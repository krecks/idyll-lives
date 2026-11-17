//! The sounds of one channel (was `web/src/audio/mono.ts`). PORT PENDING.

#[allow(unused_imports)]
use crate::dsp::*;

/// A sound of one channel by name (`MONO`).
pub fn make(kind: &str, rate: f64) -> Option<Vec<f32>> {
    let _ = (kind, rate);
    None
}
