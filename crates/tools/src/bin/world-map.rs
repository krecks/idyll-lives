//! `cargo world-map`: the sample worlds drawn from above (see `ll_tools::worldmap`).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::worldmap::main_with(&args));
}
