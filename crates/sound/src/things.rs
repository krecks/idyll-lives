//! The sounds of things in use (was `web/src/audio/things.ts`). PORT PENDING.

#[allow(unused_imports)]
use crate::dsp::*;

/// A thing's sound by name (`THINGS`).
pub fn make(kind: &str, rate: f64) -> Option<Stereo> {
    let _ = (kind, rate);
    None
}
