//! Footsteps (was `web/src/audio/steps.ts`). PORT PENDING.

#[allow(unused_imports)]
use crate::dsp::*;

/// One step on `surface` in `wear`, version `version` (`stepSamples`).
pub fn make(surface: &str, wear: &str, version: u32, rate: f64) -> Option<Vec<f32>> {
    let _ = (surface, wear, version, rate);
    None
}
