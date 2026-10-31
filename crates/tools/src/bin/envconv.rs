//! `cargo envconv`: see `ll_tools::envconv`.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::envconv::main_with(&args));
}
