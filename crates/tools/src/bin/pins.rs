//! `cargo pins`: accepted parts of the game held by their SHA-256 (see `ll_tools::pins`).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::pins::main_with(&args));
}
