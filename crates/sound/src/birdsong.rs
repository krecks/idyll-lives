//! The birds' songs (was `web/src/audio/birdsong.ts`). PORT PENDING.

#[allow(unused_imports)]
use crate::dsp::*;

/// What makes one bird's song its own (`Voice`).
#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    /// Its species' name.
    pub species: String,
    pub seed: f64,
    /// Its own pitch (Hz), pace (1 = as written) and nearness (1 = nearest).
    pub pitch: f64,
    pub pace: f64,
    pub near: f64,
}

/// One version of a bird's song at a sample rate (`songSamples`).
pub fn song(bird: &Voice, version: u32, rate: f64) -> Vec<f32> {
    let _ = (bird, version, rate);
    Vec::new()
}
