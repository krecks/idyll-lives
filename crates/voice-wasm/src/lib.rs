//! Browser bindings for `voice`, used by `web/src/voice/tts.worker.ts`: a line's plain text for
//! the model and its phonemes, the model's output trimmed, at the voice's speed and with its
//! mouth shapes, and reference clips shaped to a voice's pitch and size (the worker runs Pocket
//! TTS with ONNX Runtime Web in between). Keep this layer thin.

use voice::{Engine, G2p, Lexicon, LineInputs, VoiceParams, dsp};
use wasm_bindgen::prelude::*;

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

    /// The model's output (24 kHz) trimmed and at the line's pitch, size and speed; its mouth
    /// shapes are then in `visemes`.
    pub fn finish(&mut self, samples: &[f32]) -> Vec<f32> {
        let (samples, visemes) = self.inner.finish(samples);
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

    pub fn phonemize(&self, text: &str) -> String {
        self.engine.phonemize(text)
    }

    /// A line with pitch, size and speed for its output (clamped to what sounds right).
    pub fn line(&self, text: &str, speed: f32, pitch: f32, depth: f32) -> Line {
        Line { inner: self.engine.line(text, VoiceParams { speed, pitch, depth }), visemes: Vec::new() }
    }
}

/// A voice clip from any sound at any `rate`: 24 kHz, trimmed, at most 10 s, its loudness evened
/// out, then a short pause (see `voice::dsp::clip`). For the presets at build time and for players' recordings.
#[wasm_bindgen]
pub fn clip(samples: &[f32], rate: u32) -> Vec<f32> {
    dsp::clip(samples, rate)
}

/// A clip with a voice's pitch and size, for the voice encoder (see `voice::dsp::reference`).
#[wasm_bindgen]
pub fn reference(clip: &[f32], pitch: f32, depth: f32) -> Vec<f32> {
    dsp::reference(clip, pitch.clamp(0.5, 2.0), depth.clamp(0.75, 1.3))
}

/// The median pitch of a clip's voiced parts, Hz (0 if none): for the checks and the creator.
#[wasm_bindgen]
pub fn median_pitch(samples: &[f32]) -> f32 {
    dsp::median_pitch(samples)
}
