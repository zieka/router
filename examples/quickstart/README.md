# quickstart supergraph demonstration

This configuration allows you to quickly start a router, connected to subgraphs exposed in apollo studio.

## Prerequisites:

- Rust: Head over to [the rustup website](https://rustup.rs/) to install rust

## Environment setup

This project will need only one port on your machine:

- 4100: The Apollo federation router

If you would like to change the server's used port, change the `examples/quickstart/configuration.yaml` file's `listen` entry:

```yml
listen: 127.0.0.1:<YOUR_PORT>
```

You should be good to go!

## Running the Apollo federation router

In this project's root directory, you can run the following command to build and run the Apollo federation router:

```bash
ignition@ignition-apollo router % cargo run -- -p ./examples/quickstart
   Compiling router-bridge v0.1.0 (https://github.com/apollographql/federation.git)
   Compiling apollo-router-core v0.1.0-prealpha.3 (/Users/ignition/projects/apollo/router/crates/apollo-router-core)
   Compiling apollo-router v0.1.0-prealpha.3 (/Users/ignition/projects/apollo/router/crates/apollo-router)
    Finished dev [unoptimized + debuginfo] target(s) in 5.38s
     Running `target/debug/router -p ./examples/quickstart`
Nov 02 17:08:09.926  INFO router: Starting Apollo Router
Nov 02 17:08:10.279  INFO router: Listening on http://127.0.0.1:4100 🚀
```

Go to http://127.0.0.1:4100 to open the [Apollo studio explorer](https://www.apollographql.com/docs/studio/explorer/) and inspect the graph, and run your first queries using the Apollo federation router!
