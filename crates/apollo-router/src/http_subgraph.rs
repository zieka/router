use apollo_router_core::prelude::*;
use bytes::BytesMut;
use derivative::Derivative;
use futures::prelude::*;
use std::pin::Pin;
use tracing::Instrument;

type BytesStream = Pin<
    Box<dyn futures::Stream<Item = Result<bytes::Bytes, graphql::FetchError>> + std::marker::Send>,
>;

/// A fetcher for subgraph data that uses http.
/// Streaming via chunking is supported.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct HttpSubgraphFetcher {
    service: String,
    url: String,
    #[derivative(Debug = "ignore")]
    http_client: reqwest_middleware::ClientWithMiddleware,
}

impl HttpSubgraphFetcher {
    /// Construct a new http subgraph fetcher that will fetch from the supplied URL.
    pub fn new(service: String, url: String) -> Self {
        HttpSubgraphFetcher {
            http_client: reqwest_middleware::ClientBuilder::new(
                reqwest::Client::builder()
                    .tcp_keepalive(Some(std::time::Duration::from_secs(5)))
                    .build()
                    .unwrap(),
            )
            .with(reqwest_tracing::TracingMiddleware)
            .with(LoggingMiddleware::new(&service))
            .build(),
            service,
            url,
        }
    }

    fn request_stream(&self, request: graphql::Request) -> BytesStream {
        // Perform the actual request and start streaming.
        // Reqwest doesn't care if there is only one response, in this case it'll be a stream of
        // one element.
        let service = self.service.clone();
        self.http_client
            .post(self.url.clone())
            .json(&request)
            .send()
            .instrument(tracing::trace_span!("http-subgraph-request"))
            // We have a future for the response, convert it to a future of the stream.
            .map_ok(|r| r.bytes_stream().boxed())
            // Convert the entire future to a stream, at this point we have a stream of a result of
            // a single stream
            .into_stream()
            // Flatten the stream
            .flat_map(|result| match result {
                Ok(s) => s.map_err(Into::into).boxed(),
                Err(err) => stream::iter(vec![Err(err)]).boxed(),
            })
            .map_err(move |err: reqwest_middleware::Error| {
                tracing::error!(fetch_error = format!("{:?}", err).as_str());

                graphql::FetchError::SubrequestHttpError {
                    service: service.to_owned(),
                    reason: err.to_string(),
                }
            })
            .boxed()
    }

    fn map_to_graphql(service_name: String, bytes_stream: BytesStream) -> graphql::ResponseStream {
        // A `BytesStream` chunk doesn't always represent a `graphql::Response`.
        // We need to accumulate the bytes until we have a deserialization that doesn't end up in EOF, and then yield a `graphql::Response`.
        // `stream::unfold` allows us to do just that: keep some state around, and yield `Some((new_item, new_state))` until we're done (at which point we'll yield `None`).
        //
        // Here's the state we're keeping around:
        //
        // `bytes_stream` is the actual response we're trying to deserialize, and we'll repeatedly await.
        //
        // `current_payload_bytes` starts empty, and will accumulate until we have a full deserialization.
        // It will then reset to an empty `BytesMut` and accumulate for the next payload.
        //
        // `service_name` is useful so we can correctly track and propagate errors.
        //
        // `is_primary` les the graphql::ResponseStream users know whether an error occured during a graphql::Response,
        // or during a subsequent HTTP patch that appends an update.
        stream::unfold(
            (bytes_stream.fuse(), BytesMut::new(), service_name, true),
            |(mut bytes_stream, mut current_payload_bytes, service_name, is_primary)| async move {
                while let Some(next_chunk) = bytes_stream.next().await {
                    match next_chunk {
                        Ok(bytes) => {
                            current_payload_bytes.extend(&bytes);
                            match serde_json::from_slice::<graphql::Response>(
                                &current_payload_bytes,
                            ) {
                                Err(e) if e.is_eof() => {
                                    // Couldn't parse a full graphql::Response. This means the message is not complete yet.
                                    continue;
                                }
                                Ok(response) => {
                                    // Yield a graphql::Response
                                    return Some((
                                        response,
                                        (bytes_stream, BytesMut::new(), service_name, false),
                                    ));
                                }
                                Err(error) => {
                                    // Yield a graphql::Response
                                    return Some((
                                        graphql::FetchError::SubrequestMalformedResponse {
                                            service: service_name.clone(),
                                            reason: error.to_string(),
                                        }
                                        .to_response(is_primary),
                                        (bytes_stream, BytesMut::new(), service_name, false),
                                    ));
                                }
                            }
                        }
                        Err(fetch_error) => {
                            return Some((
                                fetch_error.to_response(is_primary),
                                (bytes_stream, BytesMut::new(), service_name, false),
                            ));
                        }
                    }
                }

                // we're done with the `BytesStream`, yield a last response if any.
                // since `bytes_stream` is fused, reentering the fold will lead us here,
                // with an empty `current_payload_bytes` thus effectively yielding None,
                // and exiting the fold
                if !current_payload_bytes.is_empty() {
                    let last_response =
                        serde_json::from_slice::<graphql::Response>(&current_payload_bytes)
                            .unwrap_or_else(|error| {
                                graphql::FetchError::SubrequestMalformedResponse {
                                    service: service_name.clone(),
                                    reason: error.to_string(),
                                }
                                .to_response(is_primary)
                            });
                    Some((
                        last_response,
                        (bytes_stream, BytesMut::new(), service_name, false),
                    ))
                } else {
                    None
                }
            },
        )
        .boxed()
    }
}

impl graphql::Fetcher for HttpSubgraphFetcher {
    /// Using reqwest fetch a stream of graphql results.
    fn stream(
        &self,
        request: graphql::Request,
    ) -> Pin<Box<dyn Future<Output = graphql::ResponseStream> + Send>> {
        let service_name = self.service.to_string();
        let bytes_stream = self.request_stream(request);
        Box::pin(async { Self::map_to_graphql(service_name, bytes_stream) })
    }
}

struct LoggingMiddleware {
    service: String,
}

impl LoggingMiddleware {
    fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }
}

#[async_trait::async_trait]
impl reqwest_middleware::Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut task_local_extensions::Extensions,
        next: reqwest_middleware::Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        tracing::trace!("Request to service {}: {:?}", self.service, req);
        let res = next.run(req, extensions).await;
        tracing::trace!("Response from service {}: {:?}", self.service, res);
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::POST;
    use httpmock::{MockServer, Regex};
    use serde_json::json;
    use test_env_log::test;

    #[test(tokio::test)]
    async fn test_non_chunked() -> Result<(), Box<dyn std::error::Error>> {
        let response = graphql::Response::builder()
            .data(json!({
              "allProducts": [
                {
                  "variation": {
                    "id": "OSS"
                  },
                  "id": "apollo-federation"
                },
                {
                  "variation": {
                    "id": "platform"
                  },
                  "id": "apollo-studio"
                }
              ]
            }))
            .build();

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/graphql")
                .body_matches(Regex::new(".*").unwrap());
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&response);
        });
        let fetcher = HttpSubgraphFetcher::new("products".into(), server.url("/graphql"));
        let collect = fetcher
            .stream(
                graphql::Request::builder()
                    .query(r#"{allProducts{variation {id}id}}"#)
                    .build(),
            )
            .await
            .collect::<Vec<_>>()
            .await;

        assert_eq!(collect[0], response);
        mock.assert();
        Ok(())
    }
}
