//! The music (was `web/src/audio/music.ts`). PORT PENDING.

#[allow(unused_imports)]
use crate::dsp::*;

/// A piece by name (`PIECES`).
pub fn make(kind: &str, rate: f64) -> Option<Stereo> {
    let _ = (kind, rate);
    None
}
