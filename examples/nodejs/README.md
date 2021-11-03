# docker-compose supergraph demonstration

This configuration allows you to quickly start a router, with a demo docker-compose.yml file.

## Prerequisites:

- Nodejs: Follow the [install instructions](https://nodejs.org/en/download/) to install nodejs and npm
- Rust: Head over to [the rustup website](https://rustup.rs/) to install rust

## Environment setup

/ ! \ Make sure your submodules are up to date if you want to experiment around the local examples / ! \

```sh
$ git submodule update --init --recursive
```

This project will need several available ports on your machine:

- 4001 to 4004: nodejs subservices exposing functionality the apollo gateway and the Apollo Federation router will expose.
- 4100: The Apollo federation router

In the federation-demo directory, install the nodejs dependencies by running:

```sh
$ npm install
```

This will install all of the dependencies for the gateway and each underlying service.

```sh
$ npm run start-services
[start-service-reviews] [nodemon] 2.0.4
[start-service-reviews] [nodemon] to restart at any time, enter `rs`
[start-service-accounts] [nodemon] 2.0.4
[start-service-reviews] [nodemon] watching path(s): *.*
[start-service-reviews] [nodemon] watching extensions: js,mjs,json
[start-service-reviews] [nodemon] starting `node services/reviews/index.js`
[start-service-products] [nodemon] 2.0.4
[start-service-products] [nodemon] to restart at any time, enter `rs`
[start-service-products] [nodemon] watching path(s): *.*
[start-service-products] [nodemon] watching extensions: js,mjs,json
[start-service-products] [nodemon] starting `node services/products/index.js`
[start-service-accounts] [nodemon] to restart at any time, enter `rs`
[start-service-accounts] [nodemon] watching path(s): *.*
[start-service-accounts] [nodemon] watching extensions: js,mjs,json
[start-service-accounts] [nodemon] starting `node services/accounts/index.js`
[start-service-inventory] [nodemon] 2.0.4
[start-service-inventory] [nodemon] to restart at any time, enter `rs`
[start-service-inventory] [nodemon] watching path(s): *.*
[start-service-inventory] [nodemon] watching extensions: js,mjs,json
[start-service-inventory] [nodemon] starting `node services/inventory/index.js`
[start-service-products] 🚀 Server ready at http://localhost:4003/
[start-service-accounts] 🚀 Server ready at http://localhost:4001/
[start-service-reviews] 🚀 Server ready at http://localhost:4002/
[start-service-inventory] 🚀 Server ready at http://localhost:4004/
```

This command will run all of the microservices at once. They can be found at http://localhost:4001, http://localhost:4002, http://localhost:4003, and http://localhost:4004.

## Running the Apollo federation router

In another terminal window, run rust gateway by running this command in the project's root directory:

```bash
ignition@ignition-apollo router % cargo run -- -p ./examples/nodejs
   Compiling router-bridge v0.1.0 (https://github.com/apollographql/federation.git)
   Compiling apollo-router-core v0.1.0-prealpha.3 (/Users/ignition/projects/apollo/router/crates/apollo-router-core)
   Compiling apollo-router v0.1.0-prealpha.3 (/Users/ignition/projects/apollo/router/crates/apollo-router)
    Finished dev [unoptimized + debuginfo] target(s) in 5.38s
     Running `target/debug/router -p ./examples/nodejs`
Nov 02 17:08:09.926  INFO router: Starting Apollo Router
Nov 02 17:08:10.279  INFO router: Listening on http://127.0.0.1:4100 🚀
```

Go to http://127.0.0.1:4100 to open the [Apollo studio explorer](https://www.apollographql.com/docs/studio/explorer/) and inspect the graph, and run your first queries using the Apollo federation router!
