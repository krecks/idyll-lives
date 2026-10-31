//! `cargo fetch-voice`: see `ll_tools::voicefetch`.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::voicefetch::main_with(&args));
}
