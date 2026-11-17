//! `cargo check-glb [keys…] [--keys] [--preview] [--validate] [--update-baseline] [--against <rev|dir>]
//! [--built <dir>]`: the gate for the game's glTF models, and for what the game builds as it runs
//! (ll_tools::checkglb).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::checkglb::main(&args));
}
