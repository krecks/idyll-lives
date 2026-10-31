//! `cargo wasm-opt`: see `ll_tools::wasmopt`.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::wasmopt::main_with(&args));
}
