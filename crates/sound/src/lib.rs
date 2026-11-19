//! The game's sounds, synthesised (no sound files): the water's and the things' loops (`water`, `things`), the
//! street and the weather (`outdoors`), the music (`music`), the sounds of one channel (`mono`: a sizzling pan,
//! the TV through a wall, the knock at the door…), the birds' songs (`birdsong`), the interface's blips
//! (`blips`), footsteps (`steps`) and the rooms' reverb (`rooms`). Each is samples at a sample rate, made
//! the same every time; the browser plays them (`web/src/audio/`, through `crates/sound-wasm`).
//!
//! Was `web/src/audio/*.ts`; the arithmetic is the TypeScript's, so most sounds are what they were (`dsp`), the
//! struck things and the motors to rounding (their tones now turned by rotation). The music (plucked strings,
//! saws and shaped drums), the knock at the door and the knife's chop (rung from their modes) have moved on.

pub mod birdsong;
pub mod blips;
pub mod dsp;
pub mod mono;
pub mod music;
pub mod outdoors;
pub mod rooms;
pub mod steps;
pub mod things;
pub mod water;

pub use dsp::{Filter, Stereo, node_filter, sweep};

/// A sound: one channel, or two.
#[derive(Clone, Debug, PartialEq)]
pub enum Made {
    Mono(Vec<f32>),
    Stereo(Stereo),
}

/// A sound made by kind off the main thread (`made.ts` `makeSound`): the water's and the things' loops, the
/// outdoors' and the pieces of music. None: no such kind.
pub fn make(kind: &str, rate: f64) -> Option<Made> {
    if let Some(s) = music::make(kind, rate) {
        return Some(Made::Stereo(s));
    }
    if let Some(m) = outdoors::make(kind, rate) {
        return Some(m);
    }
    water::make(kind, rate)
        .or_else(|| things::make(kind, rate))
        .map(Made::Stereo)
}
