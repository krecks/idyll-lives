//! `cargo tex`: the texture scripts (see `ll_tools::tex`).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::tex::main_with(&args));
}
