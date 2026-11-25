//! The pronunciation dictionary for the tests (`web/public/voice/en-us.lexz`, from
//! `cargo fetch-voice`).

use std::io::Read;
use std::path::PathBuf;

use voice::{G2p, Lexicon};

/// The phonemizer, or None (the test skips) when the dictionary is missing outside CI.
pub fn g2p() -> Option<G2p> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/public/voice/en-us.lexz");
    let Ok(file) = std::fs::File::open(&path) else {
        // CI must test the phonemizer: there a missing dictionary fails instead of skipping.
        assert!(
            std::env::var_os("CI").is_none(),
            "{} missing in CI (run `cargo fetch-voice` first)",
            path.display()
        );
        eprintln!(
            "skipped: {} missing (run `cargo fetch-voice`)",
            path.display()
        );
        return None;
    };
    let mut tsv = String::new();
    flate2::read::GzDecoder::new(file)
        .read_to_string(&mut tsv)
        .unwrap();
    Some(G2p::new(Lexicon::parse(&tsv)))
}
