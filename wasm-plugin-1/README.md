# Overview

This rust library uses rust proxy sdk to compile into WASM module which can be executed by Envoy proxy as network filter.

# The logic flow

In the final form the the module should:

- when on_new_connection is called, the module should extract requested_server_name from Envoy's connection.
  That value (SNI), if present, should be included in a call to external server, which will perform Scale From Zero 
  based on that SNI value. The call can be either GRPC or HTTP with json payload.

- When callout is successful, the callback function on_http_call_response or on_grpc_call_response is called. The callback
  should extract from either GRPC or HTTP payload two values: server name and server port which should be set in Envoy
  as upstream_dynamic_host and upstream_dynamic_port respectively.

# Building

Tools necessary for building WASM module:
- rust compiler (can be setup using rustup)
- protoc (protobuf compiler)
- wasm extension. Add it by:
     rustup target add wasm32-wasip1
     
Then to build run:

cargo build --target wasm32-wasip1 --release     

The module will be placed in target subdirectory.

