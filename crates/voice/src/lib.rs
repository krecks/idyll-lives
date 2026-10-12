//! Resident voices (see `docs/design/voices.md`).
//!
//! The model, Supertone's Supertonic 3, reads plain text ([`plain_text`]) in one of ten voices at
//! a given speed; it runs in the browser with ONNX Runtime Web on WebGPU. This crate shapes the
//! sound around it ([`dsp`]: its output brought to 24 kHz, the silence around it trimmed, a
//! resident's own pitch and size put onto each line) and gives every line a track of mouth
//! shapes ([`visemes`]) from its phonemes ([`G2p`]: English to Misaki phonemes, a port of Misaki's
//! rules, without espeak-ng).
//! Nothing here touches the simulation.

pub mod dsp;
mod g2p;
mod lexicon;
mod numbers;
mod resample;
mod rules;
pub mod visemes;

pub use g2p::{G2p, plain_text};
pub use lexicon::Lexicon;
pub use resample::resample;
pub use visemes::VisemeTrack;

/// The rate lines are finished at and played (Hz): the model's output is resampled to it.
pub const SAMPLE_RATE: u32 = 24_000;

/// Shaping on top of a voice, on a finished line.
/// Every field is a factor around 1.0 (as it is).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceParams {
    /// Speaking rate (durations are divided by it).
    pub speed: f32,
    /// Pitch, the formants kept.
    pub pitch: f32,
    /// Size of the voice: above 1 smaller (formants and pitch up together), below 1 larger.
    pub depth: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        VoiceParams { speed: 1.0, pitch: 1.0, depth: 1.0 }
    }
}

impl VoiceParams {
    /// Within what sounds right: speed 0.5–2, pitch 0.5–2, depth 0.75–1.3.
    pub fn clamped(self) -> Self {
        VoiceParams { speed: self.speed.clamp(0.5, 2.0), pitch: self.pitch.clamp(0.5, 2.0), depth: self.depth.clamp(0.75, 1.3) }
    }
}

/// One line for the model: the text it reads, the phonemes for its mouth shapes, and the voice
/// to give its output (on top of the voice the model already speaks in).
#[derive(Clone, Debug, PartialEq)]
pub struct LineInputs {
    /// Plain text (pronunciation marks dropped); empty when there is nothing to say.
    pub text: String,
    /// The line's Misaki phonemes (for its mouth shapes).
    pub phonemes: String,
    pub voice: VoiceParams,
}

impl LineInputs {
    /// The model's output (at `rate` Hz) at [`SAMPLE_RATE`], with the silence around the speech
    /// dropped, at the line's pitch, size and speed; and its mouth shapes, flat
    /// ([`VisemeTrack::flat`]).
    pub fn finish(&self, samples: &[f32], rate: u32) -> (Vec<f32>, Vec<f32>) {
        let v = self.voice;
        let at_rate = if rate == SAMPLE_RATE || rate == 0 { samples.to_vec() } else { resample(samples, rate as f32 / SAMPLE_RATE as f32) };
        let samples = dsp::shape(dsp::trim(&at_rate), v.pitch, v.depth, v.speed);
        let track = visemes::track(&self.phonemes, &samples, SAMPLE_RATE);
        (samples, track.flat())
    }
}

/// The phonemizer: `line` turns a line of English into what the model and the mouth need.
pub struct Engine {
    g2p: G2p,
}

impl Engine {
    pub fn new(g2p: G2p) -> Self {
        Engine { g2p }
    }

    /// A line in a voice (clamped to what sounds right). The phonemes come from the marked-up
    /// text, so a pronunciation fixed there shapes the mouth.
    pub fn line(&self, text: &str, voice: VoiceParams) -> LineInputs {
        let plain = plain_text(text);
        if !plain.chars().any(char::is_alphanumeric) {
            return LineInputs { text: String::new(), phonemes: String::new(), voice: VoiceParams::default() };
        }
        LineInputs { text: plain, phonemes: self.g2p.phonemize(text), voice: voice.clamped() }
    }
}
