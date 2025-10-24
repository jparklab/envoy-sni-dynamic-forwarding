use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
//use std::time::Duration;
use proxy_wasm::hostcalls::call_foreign_function;
use proxy_wasm::hostcalls::get_property;
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


    // Get SNI from downstream connection.
    let path = vec!["connection", "requested_server_name"];

    match get_property(path) {
        Ok(sni) => {
            log::info!("Found SNI");
            match sni {
                Some(s) => {
                    log::info!("There is a vector in SNI option");
                    if s.is_empty() {
                        log::info!("SNI vector is empty");
                    } else {
                        log::info!("SNI vector is NOT empty");
            let s = match str::from_utf8(&s) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            log::info!("SNI of requested server name is: {}", s);
                    }
                },
                None => {
                    log::info!("There is NO vector in SNI option");
                },
            };
/*
*/
        },
        Error => {log::info!("NOT found SNI");},
    }; 

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
