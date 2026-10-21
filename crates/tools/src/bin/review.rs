//! `cargo review`: the authenticity review's checks (`ll_tools::review`).
//!
//!     cargo review sizes [--md <dir>]   sizes and functional heights against docs/review/reference.json
//!     cargo review uses [--md <dir>]    uses, motions and sounds per object, and what looks wrong

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(ll_tools::review::main(&args));
}
