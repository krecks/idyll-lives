//! The pronunciation dictionary for the tests (`web/public/voice/en-us.lexz`, from
//! `cargo fetch-voice`).

use std::io::Read;
use std::path::PathBuf;

use voice::{G2p, Lexicon};

/// The phonemizer, or None (the test skips) when the dictionary is missing and `VOICE_TEST_SKIP=1`
/// asks for that: otherwise a missing dictionary fails, so a green run means it was tested.
pub fn g2p() -> Option<G2p> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/public/voice/en-us.lexz");
    let Ok(file) = std::fs::File::open(&path) else {
        assert!(
            std::env::var_os("VOICE_TEST_SKIP").is_some_and(|v| v == "1"),
            "{} missing: run `cargo fetch-voice` first (or VOICE_TEST_SKIP=1 to skip these tests)",
            path.display()
        );
        eprintln!("skipped: {} missing (VOICE_TEST_SKIP=1)", path.display());
        return None;
    };
    let mut tsv = String::new();
    flate2::read::GzDecoder::new(file)
        .read_to_string(&mut tsv)
        .unwrap();
    Some(G2p::new(Lexicon::parse(&tsv)))
}
