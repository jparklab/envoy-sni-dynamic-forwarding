use prost::Message;
use proxy_wasm::hostcalls::call_foreign_function;
use proxy_wasm::hostcalls::get_property;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod wasm_extensions {
    include!(concat!(
        env!("OUT_DIR"),
        "/envoy.source.extensions.common.wasm.rs"
    ));
}

// VM main routine. Sets up root context which holds a config.
// That config is passed to stream contexts.
proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(SniRootContext{
            config: Default::default(),
        })
    });
}}

struct ScaleFromZero {
    config: Config,
}

#[derive(Serialize)]
struct Sni {
    name: String,
}

#[derive(Deserialize)]
struct Backend {
    server: String,
    port: String,
}

// Config struct, which is deserialized from protobufs received from config file
// or control plane.
#[derive(Deserialize, Clone)]
#[serde(default)]
struct Config {
    cluster: String,
    timeout: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cluster: Default::default(),
            timeout: 30, // Default timeout
        }
    }
}

struct SniRootContext {
    config: Config,
}

impl Context for SniRootContext {}

impl RootContext for SniRootContext {
    fn on_configure(&mut self, config_size: usize) -> bool {
        log::info!("On CONFIG: {}", config_size);
        // TODO: bailout when no config

        let config_bytes = self.get_plugin_configuration().unwrap();
        // TODO: Unwrap and check error
        self.config = serde_json::from_slice(&config_bytes).unwrap();
        log::info!("On CONFIG: {}", self.config.cluster);
        log::info!("On CONFIG: {}", self.config.timeout);
        true
    }

    fn create_stream_context(&self, _: u32) -> Option<Box<dyn StreamContext>> {
        Some(Box::new(ScaleFromZero {
            config: self.config.clone(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::StreamContext)
    }
}

impl StreamContext for ScaleFromZero {
    fn on_new_connection(&mut self) -> Action {
        log::info!("On NEW connection called");

        // Get SNI from the connection.
        let server = match ScaleFromZero::get_sni() {
            Some(sni) => sni,
            None => return Action::Continue,
        };

        // build the json payload
        let server_name = Sni { name: server };
        let server_name_json = serde_json::to_string(&server_name).unwrap();

        match self.dispatch_http_call(
            &self.config.cluster,
            vec![
                (":method", "POST"),
                (":path", "/scale_from_zero"),
                (":authority", "waiter_service"),
                ("Content-Type", "application/json"),
            ],
            Some(server_name_json.as_bytes()),
            vec![],
            Duration::from_secs(self.config.timeout as u64),
        ) {
            Ok(_) => return Action::Pause,
            Err(e) => {
                log::info!("Error dispatching HTTP call: {:?}", e);
                return Action::Continue;
            }
        }
    }
}

impl ScaleFromZero {
    // Function extracts sni from the stream info related to the connection.
    fn get_sni() -> Option<String> {
        // Get SNI from downstream connection.
        let path = vec!["connection", "requested_server_name"];

        match get_property(path) {
            Ok(sni) => {
                match sni {
                    Some(s) => {
                        if s.is_empty() {
                            log::info!("SNI found but is empty - continuing.");
                            return None;
                        } else {
                            let _server_name = match str::from_utf8(&s) {
                                Ok(server_name) => {
                                    log::info!("SNI of requested server name is: {}", server_name);
                                    return Some(server_name.to_string());
                                }
                                Err(_) => {
                                    log::info!(
                                        "SNI found but contains wrong characters - continuing."
                                    );
                                    return None;
                                }
                            };
                        }
                    }
                    None => {
                        log::info!("SNI not found - continuing.");
                        return None;
                    }
                };
            }
            Err(_) => {
                log::info!("Error obtaining SNI - continuing.");
                return None;
            }
        };
    }

    fn set_upstream(&mut self, server: String, port: String) {
        let dynamic_port = wasm_extensions::SetEnvoyFilterStateArguments {
            path: "envoy.upstream.dynamic_port".to_string(),
            value: port,
            span: wasm_extensions::LifeSpan::FilterChain.into(),
        };
        let mut buf = Vec::new();
        buf.reserve(dynamic_port.encoded_len());
        dynamic_port.encode(&mut buf).unwrap();
        match call_foreign_function("set_envoy_filter_state", Some(&buf)) {
            Ok(_) => {
                log::info!("Foreign function OK");
            }
            Err(e) => {
                log::info!("Foreign function ERROR: {:?}", e);
            }
        }

        let dynamic_host = wasm_extensions::SetEnvoyFilterStateArguments {
            path: "envoy.upstream.dynamic_host".to_string(),
            value: server,
            span: wasm_extensions::LifeSpan::FilterChain.into(),
        };

        buf.clear();
        buf.reserve(dynamic_host.encoded_len());
        dynamic_host.encode(&mut buf).unwrap();
        let _ = call_foreign_function("set_envoy_filter_state", Some(&buf));
    }
}

impl Context for ScaleFromZero {
    fn on_http_call_response(&mut self, _: u32, _: usize, body_size: usize, _: usize) {
        log::info!("RECEIVED HTTP CALL RESPONSE");

        // Extract the body and read server and port values.
        if let Some(body) = self.get_http_call_response_body(0, body_size) {
            #[allow(unknown_lints, clippy::manual_is_multiple_of)]
            if !body.is_empty() {
                // Deserialize json payload
                // TODO: handle case when is not u8.
                let body_string = String::from_utf8(body).ok();
                let backend: Backend = serde_json::from_str(&body_string.unwrap()).unwrap();

                log::info!("Received server values {}:{}", backend.server, backend.port);
                self.set_upstream(backend.server, backend.port);
            }
        }

        // Continue processing the request.
        self.resume_downstream();
    }
}
