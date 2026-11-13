//! `cargo check-chunks`: the PlayCanvas internals the renderer relies on are still there (see `ll_tools::chunks`).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::chunks::main_with(&args));
}
