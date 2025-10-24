use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
//use std::time::Duration;
use proxy_wasm::hostcalls::call_foreign_function;
use prost::Message;

pub mod wasm_extensions {
    include!(concat!(env!("OUT_DIR"), "/envoy.source.extensions.common.wasm.rs"));
}
use crate::wasm_extensions::LifeSpan;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_stream_context(|_, _| -> Box<dyn StreamContext> { Box::new(GrpcAuthRandom) });
}}

struct GrpcAuthRandom;

impl StreamContext for GrpcAuthRandom {
    fn on_new_connection(&mut self) -> Action {
    log::info!("On NEW connection called");

    let dynamic_port = wasm_extensions::SetEnvoyFilterStateArguments{    
        path: "envoy.upstream.dynamic_port".to_string(),
        value: "19443".to_string(),
        // TODO: change it to enum
        span: 0 /*LifeSpan::FilterChain */
    };
    let mut buf = Vec::new();
    buf.reserve(dynamic_port.encoded_len());
    dynamic_port.encode(&mut buf).unwrap();
    let _ = call_foreign_function("set_envoy_filter_state", Some(&buf));
	
    let dynamic_host = wasm_extensions::SetEnvoyFilterStateArguments{    
        path: "envoy.upstream.dynamic_host".to_string(),
        value: "127.0.0.1".to_string(),
        // TODO: change it to enum
        span: 0 /*LifeSpan::FilterChain */
    };

    buf.clear();
    buf.reserve(dynamic_host.encoded_len());
    dynamic_host.encode(&mut buf).unwrap();
    let _ = call_foreign_function("set_envoy_filter_state", Some(&buf));

                //return Action::Pause;
                Action::Continue
    }
}

impl Context for GrpcAuthRandom {}
