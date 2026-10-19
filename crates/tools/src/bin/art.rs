//! `cargo art <family…> | --all [--only id] [--preview] [--no-pack]`, `cargo art --centres`: runs Blender on
//! the object families, then the gate (ll_tools::art).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::art::main(&args));
}
