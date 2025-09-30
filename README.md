# Overview

This repository is for an experiment where we use [SNI dynamic forward proxy](https://www.envoyproxy.io/docs/envoy/v1.35.3/api-v3/extensions/filters/network/sni_dynamic_forward_proxy/v3/sni_dynamic_forward_proxy.proto.html) with [Network external authorization](https://www.envoyproxy.io/docs/envoy/v1.35.3/api-v3/extensions/filters/network/ext_authz/v3/ext_authz.proto.html) filter to handle scale from zero for services at L4(TCP) level.

# Notes

In order to run the experiment, we need to use a modified Envoy proxy because `sni_dynamic_forward_proxy` is applied before `ext_authz` filter as described below

- sni_dynamic_forward_proxy is applied when a connection is established
- ext_authz is applied when the first data is received

## Updating upstream Envoy to support applying ext_authz when connections are establishes

This ticket is somewhat related, and could give a reason to update sni_dynamic_forwarding_cluster

https://github.com/envoyproxy/envoy/issues/9023

# How to run experiment

### 0. Prerequsite

`run-test` uses `nginx` to run a backend server. Please install nginx or download nginx before running the test

### 1. Build an envoy with a patch

In order to fully test sni_dynamic_forwarding_proxy filter with ext_authz, we need to build an Envoy proxy with a change to make ext_authz filter be applied on new connections. Below is the modification I made to `ext_authz` filter

You can run tests with a vanilla envoy in case you want to check where Envoy fails to route requests.

```
$ git diff source/extensions/filters/network/ext_authz/ext_authz.cc
diff --git a/source/extensions/filters/network/ext_authz/ext_authz.cc b/source/extensions/filters/network/ext_authz/ext_authz.cc
index 2a8276a1f6..9baa93a26a 100644
--- a/source/extensions/filters/network/ext_authz/ext_authz.cc
+++ b/source/extensions/filters/network/ext_authz/ext_authz.cc
@@ -38,6 +38,7 @@ void Filter::callCheck() {
 }
 
 Network::FilterStatus Filter::onData(Buffer::Instance&, bool /* end_stream */) {
+#if 0 // JPARK
   if (!filterEnabled(filter_callbacks_->connection().streamInfo().dynamicMetadata())) {
     config_->stats().disabled_.inc();
     return Network::FilterStatus::Continue;
@@ -50,11 +51,29 @@ Network::FilterStatus Filter::onData(Buffer::Instance&, bool /* end_stream */) {
   }
   return filter_return_ == FilterReturn::Stop ? Network::FilterStatus::StopIteration
                                               : Network::FilterStatus::Continue;
+#else
+  return Network::FilterStatus::Continue;
+#endif
 }
 
 Network::FilterStatus Filter::onNewConnection() {
+#if 1 // JPARK
+  if (!filterEnabled(filter_callbacks_->connection().streamInfo().dynamicMetadata())) {
+    config_->stats().disabled_.inc();
+    return Network::FilterStatus::Continue;
+  }
+
+  if (status_ == Status::NotStarted) {
+    // By waiting to invoke the check at onData() the call to authorization service will have
+    // sufficient information to fill out the checkRequest_.
+    callCheck();
+  }
+  return filter_return_ == FilterReturn::Stop ? Network::FilterStatus::StopIteration
+                                              : Network::FilterStatus::Continue;
+#else
   // Wait till onData() happens.
   return Network::FilterStatus::Continue;
+#endif
 }
 ```

### 2. Build Wasm module

Wasm module is used to set filter state with a metadata value that is set by the ext_authz filter.
You can use the Wasm module from the repo. In case you need to rebuild the wasm module, follow instructions below.

```
cd wasm-plugin
make
```

### 3. Run the test

```
./run-test -o output --nginx-bin <path to nginx binary> --envoy-bin <path to envoy binary>
```

With a unmodified Envoy, we will see an error message like below

```
------------------------------------------------------------
curl output:

curl: (35) Recv failure: Connection reset by peer

------------------------------------------------------------
```

and we can find errors like below in `output/envoy.stderr`

```
[2025-09-30 15:41:25.599][2474150][info][wasm] [source/extensions/common/wasm/context.cc:1137] wasm log tcp_plugin: failed to get property for [metadata filter_metadata envoy.filters.network.ext_authz envoy.upstream.dynamic_port]: error status returned by host: not found
```

With an Envoy patched for `ext_authz` filter, we will see `curl` succeed with the following output

```
------------------------------------------------------------
curl output:

Hello from fake-service.mydomain.com

------------------------------------------------------------
```