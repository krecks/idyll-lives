//! `cargo assets`: compressed copies of the game's assets for the production build (see `ll_tools::assets`).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::assets::main_with(&args));
}
