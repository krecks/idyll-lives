//! `cargo release-version`: see `ll_tools::version`.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::version::main_with(&args));
}
