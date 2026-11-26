//! The interface's little sounds, synthesised (was `web/src/audio/blips.ts`): picking things up, putting them
//! down, money changing hands, walls going up; in Live, a pie menu opening, the clock's speed, money in and out
//! and the household's milestones. Each is a few pitched blips and puffs of noise (`parts`), made as samples at
//! a sample rate.
//!
//! They were Web Audio graphs once (an `OscillatorNode` under a gain envelope; noise through a low-pass): the
//! waves here are the browser's own band-limited ones (`wave`), and what is made is what the graphs made
//! (compared against Chromium's rendering of them when they moved).

use crate::dsp::m::*;
use crate::dsp::{Filter, Rng, add, node_filter, sample_at, sweep};

/// The blips' names (`Blip`).
pub const NAMES: [&str; 21] = [
    "pick",
    "place",
    "sell",
    "rotate",
    "error",
    "build",
    "remove",
    "paint",
    "upgrade",
    "tab",
    "open",
    "close",
    "menu",
    "choose",
    "speed",
    "pause",
    "resume",
    "moneyIn",
    "moneyOut",
    "notify",
    "notifyLow",
];

/// A wave an oscillator plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wave {
    Sine,
    Triangle,
    Square,
}

/// A part of a blip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Part {
    /// A pitched blip gliding from `from` to `to` Hz, with a fast attack and an exponential tail; `at` and
    /// `length` in seconds.
    Tone {
        wave: Wave,
        from: f64,
        to: f64,
        at: f64,
        length: f64,
        gain: f64,
    },
    /// Noise under `cutoff` Hz dying away (dust, scrapes, the click of a knob).
    Hiss {
        cutoff: f64,
        at: f64,
        length: f64,
        gain: f64,
    },
}

impl Part {
    fn at(&self) -> f64 {
        match *self {
            Part::Tone { at, .. } | Part::Hiss { at, .. } => at,
        }
    }

    fn length(&self) -> f64 {
        match *self {
            Part::Tone { length, .. } | Part::Hiss { length, .. } => length,
        }
    }
}

fn tone(wave: Wave, from: f64, to: f64, at: f64, length: f64, gain: f64) -> Part {
    Part::Tone {
        wave,
        from,
        to,
        at,
        length,
        gain,
    }
}

fn hiss(cutoff: f64, at: f64, length: f64, gain: f64) -> Part {
    Part::Hiss {
        cutoff,
        at,
        length,
        gain,
    }
}

/// What each sound is made of (`BLIPS`); none: no such blip.
pub fn parts(name: &str) -> Option<Vec<Part>> {
    use Wave::*;
    Some(match name {
        "pick" => vec![
            tone(Triangle, 660.0, 990.0, 0.0, 0.09, 0.16),
            tone(Sine, 1320.0, 1320.0, 0.03, 0.06, 0.05),
        ],
        // A soft thump, then a bright "ting-ting".
        "place" => vec![
            tone(Sine, 170.0, 55.0, 0.0, 0.16, 0.45),
            hiss(900.0, 0.0, 0.06, 0.12),
            tone(Sine, 1568.0, 1568.0, 0.06, 0.12, 0.07),
            tone(Sine, 2093.0, 2093.0, 0.11, 0.18, 0.06),
        ],
        "sell" => vec![
            tone(Triangle, 880.0, 330.0, 0.0, 0.16, 0.14),
            tone(Sine, 2637.0, 2637.0, 0.1, 0.16, 0.06),
            tone(Sine, 3136.0, 3136.0, 0.15, 0.2, 0.05),
        ],
        "rotate" => vec![
            tone(Square, 1400.0, 1100.0, 0.0, 0.025, 0.035),
            hiss(3000.0, 0.0, 0.03, 0.05),
        ],
        "error" => vec![
            tone(Square, 220.0, 210.0, 0.0, 0.08, 0.05),
            tone(Square, 175.0, 165.0, 0.09, 0.12, 0.05),
        ],
        // Wood knock.
        "build" => vec![
            tone(Sine, 240.0, 110.0, 0.0, 0.09, 0.4),
            hiss(500.0, 0.0, 0.08, 0.2),
        ],
        "remove" => vec![
            hiss(280.0, 0.0, 0.28, 0.3),
            tone(Sine, 130.0, 50.0, 0.0, 0.18, 0.3),
        ],
        // A brush stroke: a soft swish, rising.
        "paint" => vec![
            hiss(1800.0, 0.0, 0.16, 0.12),
            tone(Sine, 520.0, 780.0, 0.04, 0.12, 0.05),
        ],
        "upgrade" => [1047.0, 1319.0, 1568.0, 2093.0]
            .iter()
            .enumerate()
            .map(|(i, &f)| tone(Triangle, f, f, i as f64 * 0.065, 0.14, 0.09))
            .collect(),
        "tab" => vec![tone(Sine, 1800.0, 1500.0, 0.0, 0.03, 0.04)],
        "open" => vec![
            tone(Triangle, 523.0, 523.0, 0.0, 0.1, 0.08),
            tone(Triangle, 784.0, 784.0, 0.07, 0.14, 0.08),
        ],
        "close" => vec![
            tone(Triangle, 784.0, 784.0, 0.0, 0.1, 0.07),
            tone(Triangle, 523.0, 523.0, 0.07, 0.14, 0.07),
        ],
        // Live mode's: a pie menu opening (a quick rise and a breath), an item of it chosen.
        "menu" => vec![
            tone(Sine, 587.0, 880.0, 0.0, 0.07, 0.07),
            hiss(2500.0, 0.0, 0.03, 0.03),
        ],
        "choose" => vec![
            tone(Triangle, 880.0, 880.0, 0.0, 0.05, 0.07),
            tone(Sine, 1319.0, 1319.0, 0.03, 0.08, 0.05),
        ],
        // The clock: a faster or slower speed (a tick), paused (falling), going again (rising).
        "speed" => vec![tone(Sine, 1480.0, 1760.0, 0.0, 0.04, 0.05)],
        "pause" => vec![
            tone(Triangle, 659.0, 659.0, 0.0, 0.09, 0.08),
            tone(Triangle, 440.0, 440.0, 0.08, 0.14, 0.08),
        ],
        "resume" => vec![
            tone(Triangle, 440.0, 440.0, 0.0, 0.09, 0.08),
            tone(Triangle, 659.0, 659.0, 0.08, 0.14, 0.08),
        ],
        // Money in: a bright coin; money out: a falling note and a rustle.
        "moneyIn" => vec![
            tone(Triangle, 1047.0, 1047.0, 0.0, 0.06, 0.05),
            tone(Sine, 2093.0, 2093.0, 0.0, 0.1, 0.06),
            tone(Sine, 2637.0, 2637.0, 0.07, 0.18, 0.06),
        ],
        "moneyOut" => vec![
            tone(Triangle, 784.0, 523.0, 0.0, 0.13, 0.08),
            hiss(1400.0, 0.0, 0.06, 0.05),
        ],
        // A milestone of the household: a rising chime, or a falling one for bad news.
        "notify" => vec![
            tone(Sine, 784.0, 784.0, 0.0, 0.45, 0.09),
            tone(Sine, 1175.0, 1175.0, 0.13, 0.55, 0.08),
            tone(Sine, 2349.0, 2349.0, 0.13, 0.3, 0.02),
        ],
        "notifyLow" => vec![
            tone(Sine, 587.0, 587.0, 0.0, 0.45, 0.07),
            tone(Sine, 440.0, 440.0, 0.15, 0.53, 0.07),
        ],
        _ => return None,
    })
}

/// Seconds a part sounds on after its length (its envelope held at nothing).
const TAIL: f64 = 0.02;
/// Seconds of noise the puffs read from, the same every time.
const NOISE: f64 = 0.5;

fn noise(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(0x5f as f64);
    (0..round_usize(rate * NOISE))
        .map(|_| (r.next() * 2.0 - 1.0) as f32)
        .collect()
}

/// A sound of the interface by name, as samples (one channel; `blipSamples`); none: no such blip.
pub fn make(name: &str, rate: f64) -> Option<Vec<f32>> {
    let parts = parts(name)?;
    let end = parts
        .iter()
        .map(|p| p.at() + p.length() + TAIL)
        .fold(f64::NEG_INFINITY, max);
    let mut d = vec![0.0f32; sample_at(end, rate) + 1];
    let mut white: Option<Vec<f32>> = None;
    for p in &parts {
        let first = sample_at(p.at(), rate);
        let last = d.len().min(sample_at(p.at() + p.length() + TAIL, rate));
        match *p {
            Part::Hiss {
                cutoff,
                at,
                length,
                gain,
            } => {
                let white = white.get_or_insert_with(|| noise(rate));
                // (The node's own resonance: 1 dB.)
                let n = last.saturating_sub(first).min(white.len());
                let low = node_filter(&white[..n], rate, Filter::Lowpass, cutoff, 1.0, false);
                for i in first..first + n {
                    let env = sweep(&[gain, 0.0001], &[0.0, length], i as f64 / rate - at);
                    add(&mut d, i, low[i - first] as f64 * env);
                }
            }
            Part::Tone {
                wave: kind,
                from,
                to,
                at,
                length,
                gain,
            } => {
                let mut phase = 0.0;
                for i in first..last {
                    let t = i as f64 / rate - at;
                    let f = if from == to {
                        from
                    } else {
                        sweep(&[from, to], &[0.0, length], t)
                    };
                    let env = sweep(&[0.0001, gain, 0.0001], &[0.0, 0.006, length], t);
                    add(&mut d, i, env * wave(kind, phase, f, rate));
                    phase += f / rate;
                }
            }
        }
    }
    Some(d)
}

/// The browser's wave tables hold a period in this many samples (so half as many overtones at most), at the
/// sample rates of sound cards.
const TABLE: f64 = 4096.0;
/// The tables thin out their overtones in steps of a third of an octave.
const STEPS: f64 = 3.0;
/// The square's level: the browser brings each wave's fullest table to a peak of 1, and a square built from
/// overtones overshoots its corners by 18 % (Gibbs), so its flat parts sit at 0.85.
const SQUARE_LEVEL: f64 = 0.8482;

/// A wave at `phase` (in periods) played at `hz`, as a Web Audio `OscillatorNode` makes it: from its
/// overtones, only those under half the sample rate (the last third of an octave of them fading out as the
/// pitch rises), so a high square does not fold back into the band as false tones.
fn wave(kind: Wave, phase: f64, hz: f64, rate: f64) -> f64 {
    let angle = 2.0 * PI * (phase - phase.floor());
    if kind == Wave::Sine {
        return sin(angle);
    }
    // The two tables the pitch lies between: how many overtones each holds, and how far from the fuller to the
    // thinner.
    let lowest = rate / TABLE;
    let range = max(0.0, 1.0 + STEPS * log2(max(hz, lowest / 2.0) / lowest));
    let fuller = ((TABLE / 2.0) * pow(2.0, -range.floor() / STEPS)).floor();
    let thinner = ((TABLE / 2.0) * pow(2.0, -(range.floor() + 1.0) / STEPS)).floor();
    let fade = range - range.floor();
    let mut sum = 0.0;
    let mut n = 1.0;
    while n <= fuller {
        let share = if n <= thinner { 1.0 } else { 1.0 - fade };
        let k = 2.0 / (n * PI);
        let sign = if ((n as i64 - 1) >> 1) & 1 == 1 {
            -1.0
        } else {
            1.0
        };
        sum += share
            * (if kind == Wave::Square {
                2.0 * k
            } else {
                2.0 * k * k * sign
            })
            * sin(n * angle);
        n += 2.0;
    }
    if kind == Wave::Square {
        sum * SQUARE_LEVEL
    } else {
        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_blips_are_the_same_every_time() {
        for name in NAMES {
            let a = make(name, 48000.0).unwrap();
            assert!(!a.is_empty());
            assert_eq!(a, make(name, 48000.0).unwrap(), "{name}");
        }
        assert_eq!(make("nothing", 48000.0), None);
    }
}
