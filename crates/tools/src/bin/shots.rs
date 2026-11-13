//! `cargo shots`: numbers from pictures (see `ll_tools::shots`).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::shots::main_with(&args));
}
