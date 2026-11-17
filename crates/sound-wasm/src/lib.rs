//! Browser bindings for `sound`, used by `web/src/audio/synth.ts` (on the page and in `water.worker.ts`): each
//! sound by name at a sample rate, as samples. Keep this layer thin.

// SIMD comes from `.cargo/config.toml`'s rustflags, which a `RUSTFLAGS` in the environment replaces
// rather than adds to: such a build stops here instead of shipping a slower wasm without it.
#[cfg(all(target_arch = "wasm32", not(target_feature = "simd128")))]
compile_error!(
    "the wasm needs SIMD: a RUSTFLAGS in the environment replaces .cargo/config.toml's `-C target-feature=+simd128`, so add it there too"
);

use sound::{Filter, Made, birdsong, blips, mono, rooms, steps};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

/// Runs as the module starts. A panic aborts (`panic = "abort"`), and the trap only says "unreachable": the
/// hook first logs what panicked and where.
#[wasm_bindgen(start)]
fn start() {
    std::panic::set_hook(Box::new(|info| console_error(&info.to_string())));
}

/// A sound: its left channel, and its right (none for a sound of one channel).
#[wasm_bindgen]
pub struct Sound {
    left: Vec<f32>,
    right: Option<Vec<f32>>,
}

#[wasm_bindgen]
impl Sound {
    /// The left channel (the only one of a sound of one channel), moved out.
    #[wasm_bindgen(js_name = takeLeft)]
    pub fn take_left(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.left)
    }

    /// The right channel, moved out; none for a sound of one channel.
    #[wasm_bindgen(js_name = takeRight)]
    pub fn take_right(&mut self) -> Option<Vec<f32>> {
        self.right.take()
    }
}

fn sound(made: Made) -> Sound {
    match made {
        Made::Mono(left) => Sound { left, right: None },
        Made::Stereo(s) => Sound {
            left: s.left,
            right: Some(s.right),
        },
    }
}

/// A sound made by kind (the water's and the things' loops, the outdoors', the pieces of music); none: no such kind.
#[wasm_bindgen]
pub fn make(kind: &str, rate: f64) -> Option<Sound> {
    sound::make(kind, rate).map(sound)
}

/// A sound of one channel by name (`mono.ts`'s `MONO` once); none: no such sound.
#[wasm_bindgen]
pub fn mono(kind: &str, rate: f64) -> Option<Vec<f32>> {
    mono::make(kind, rate)
}

/// An interface blip by name.
#[wasm_bindgen]
pub fn blip(name: &str, rate: f64) -> Option<Vec<f32>> {
    blips::make(name, rate)
}

/// One footstep on `surface` in `wear`, version `version`.
#[wasm_bindgen]
pub fn step(surface: &str, wear: &str, version: u32, rate: f64) -> Option<Vec<f32>> {
    steps::make(surface, wear, version, rate)
}

/// One version of a bird's song (its voice: species, seed, pitch, pace and nearness).
#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn song(
    species: &str,
    seed: f64,
    pitch: f64,
    pace: f64,
    near: f64,
    version: u32,
    rate: f64,
) -> Vec<f32> {
    let voice = birdsong::Voice {
        species: species.to_string(),
        seed,
        pitch,
        pace,
        near,
    };
    birdsong::song(&voice, version, rate)
}

/// A room's impulse response by size (`small`, `medium`, `large`).
#[wasm_bindgen]
pub fn room(size: &str, rate: f64) -> Option<Sound> {
    rooms::response(size, rate).map(|s| sound(Made::Stereo(s)))
}

/// `x` through a two-pole filter as a Web Audio `BiquadFilterNode` computes it (`kind`: lowpass, highpass or
/// bandpass); `looped`: `x` is a loop.
#[wasm_bindgen]
pub fn filter(x: &[f32], rate: f64, kind: &str, frequency: f64, q: f64, looped: bool) -> Vec<f32> {
    let kind = match kind {
        "highpass" => Filter::Highpass,
        "bandpass" => Filter::Bandpass,
        _ => Filter::Lowpass,
    };
    sound::node_filter(x, rate, kind, frequency, q, looped)
}
