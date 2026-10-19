//! `cargo content`: moves catalog items into the Validate category and back (see `ll_tools::content`).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::content::main_with(&args));
}
