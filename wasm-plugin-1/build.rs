use std::io::Result;
fn main() -> Result<()> {
    print!("Compiling\n");
    prost_build::compile_protos(
        &["/home/christoph/envoy/source/extensions/common/wasm/ext/set_envoy_filter_state.proto"],
        &["/home/christoph/envoy/"],
    )?;
    Ok(())
}
