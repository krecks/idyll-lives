//! The rooms' reverb (was `web/src/audio/rooms.ts`). PORT PENDING.

#[allow(unused_imports)]
use crate::dsp::*;

/// A room's impulse response by size (`roomResponse`).
pub fn response(size: &str, rate: f64) -> Option<Stereo> {
    let _ = (size, rate);
    None
}
