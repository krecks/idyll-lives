//! `cargo decoders`: see `ll_tools::decoders`.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::decoders::main_with(&args));
}
