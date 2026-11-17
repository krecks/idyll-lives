//! The interface's blips (was `web/src/audio/blips.ts`). PORT PENDING.

#[allow(unused_imports)]
use crate::dsp::*;

/// A blip by name (`BLIPS`).
pub fn make(name: &str, rate: f64) -> Option<Vec<f32>> {
    let _ = (name, rate);
    None
}
