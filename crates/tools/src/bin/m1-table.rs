//! `cargo m1-table <run>`: the M1 estimate's report (see `ll_tools::m1table`).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::m1table::main(&args));
}
