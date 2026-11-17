//! `cargo art <family…> | --all [--only id] [--no-pack] [--out dir] [--check]`, `cargo art --centres`: builds
//! the object families, then runs the gate on them (ll_art).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_art::main(&args));
}
