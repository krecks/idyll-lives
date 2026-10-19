//! `cargo check-glb [keys…] [--keys] [--preview]`: the gate for the game's glTF models (ll_tools::checkglb).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::checkglb::main(&args));
}
