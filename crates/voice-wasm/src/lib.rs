//! Browser bindings for `voice`, used by `web/src/voice/tts.worker.ts`: a line's plain text for
//! the model and its phonemes, and the model's output at 24 kHz, trimmed, at the voice's pitch
//! and size and with its mouth shapes (the worker runs Supertonic 3 with ONNX Runtime Web in
//! between). Keep this layer thin.

// SIMD comes from `.cargo/config.toml`'s rustflags, which a `RUSTFLAGS` in the environment replaces
// rather than adds to: such a build stops here instead of shipping a slower wasm without it.
#[cfg(all(target_arch = "wasm32", not(target_feature = "simd128")))]
compile_error!("the wasm needs SIMD: a RUSTFLAGS in the environment replaces .cargo/config.toml's `-C target-feature=+simd128`, so add it there too");

use voice::{Engine, G2p, Lexicon, LineInputs, VoiceParams, dsp};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// The worker's `__wasmPanic` (`tts.worker.ts`), which passes the message on.
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = __wasmPanic)]
    fn report_panic(message: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

/// Runs as the module starts. A panic aborts (`panic = "abort"`), and the trap only says
/// "unreachable": the hook first passes on what panicked and where.
#[wasm_bindgen(start)]
fn start() {
    std::panic::set_hook(Box::new(|info| {
        let message = info.to_string();
        if report_panic(&message).is_err() {
            console_error(&message);
        }
    }));
}

#[wasm_bindgen]
pub struct Voice {
    engine: Engine,
}

/// One line for the model, and what to do with its output.
#[wasm_bindgen]
pub struct Line {
    inner: LineInputs,
    visemes: Vec<f32>,
}

#[wasm_bindgen]
impl Line {
    /// What the model reads (pronunciation marks dropped); empty when there is nothing to say.
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.inner.text.clone()
    }

    /// The model's output (at `rate` Hz) at 24 kHz, trimmed and at the line's pitch, size and
    /// speed; its mouth shapes are then in `visemes`.
    pub fn finish(&mut self, samples: &[f32], rate: u32) -> Vec<f32> {
        let (samples, visemes) = self.inner.finish(samples, rate);
        self.visemes = visemes;
        samples
    }

    /// The finished clip's mouth shapes, flat: `[segments, energy frames, times…, visemes…,
    /// energy…]` (see `voice::VisemeTrack::flat`); empty before `finish`.
    #[wasm_bindgen(getter)]
    pub fn visemes(&self) -> Vec<f32> {
        self.visemes.clone()
    }
}

#[wasm_bindgen]
impl Voice {
    /// `lexicon`: `en-us.lex` (decompressed).
    #[wasm_bindgen(constructor)]
    pub fn new(lexicon: &str) -> Voice {
        Voice { engine: Engine::new(G2p::new(Lexicon::parse(lexicon))) }
    }

    /// A line with pitch, size and speed for its output (clamped to what sounds right).
    pub fn line(&self, text: &str, speed: f32, pitch: f32, depth: f32) -> Line {
        Line { inner: self.engine.line(text, VoiceParams { speed, pitch, depth }), visemes: Vec::new() }
    }
}

/// The median pitch of a clip's voiced parts, Hz (0 if none): for the checks and the creator.
#[wasm_bindgen]
pub fn median_pitch(samples: &[f32]) -> f32 {
    dsp::median_pitch(samples)
}
