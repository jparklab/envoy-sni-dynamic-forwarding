use std::env;
use std::io::Result;

fn main() -> Result<()> {
    print!("Compiling protos\n");
    env::var("ENVOY_DIR")
        .expect("Environment variable ENVOY_DIR not set. It is needed to find WASM protos.");

    let root_dir = env::var("ENVOY_DIR").unwrap() + "/";
    prost_build::compile_protos(
        &[root_dir.clone() + "source/extensions/common/wasm/ext/set_envoy_filter_state.proto"],
        &[root_dir],
    )?;
    Ok(())
}
