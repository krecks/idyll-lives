//! `cargo licenses`: see `ll_tools::licenses`.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::licenses::main_with(&args));
}
