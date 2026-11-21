//! The world's sounds of one channel, synthesised (was `web/src/audio/mono.ts`): a sizzling pan, a knife on a
//! board, a television's murmur, drums, the outdoor bed of wind, crickets and a knock at the door. Each is made
//! whole here, its filter included, as samples at a sample rate; `world.ts` gives each a place left to right (a
//! stereo panner) and its level (`catalog.ts`).
//!
//! They were Web Audio graphs once (a buffer through a `BiquadFilterNode`; the knock oscillators and a burst of
//! noise): `node_filter` and `sweep` (in `dsp`) are those nodes' own arithmetic, and what the sizzle, the TV,
//! the drums, the bed and the crickets make here is what the graphs made (compared sample by sample against
//! Chromium's rendering of them when they moved). The knife's chop and the knock have moved on: each is now
//! rung from the modes of what is struck (a blade and a board, a door's panel: `Modal`).

use crate::dsp::m::*;
use crate::dsp::{Decay, Filter, Modal, Mode, Rng, add, node_filter, raw, sample_at, sweep};

/// The sounds of one channel, by name (`MONO`).
pub const KINDS: [&str; 7] = ["sizzle", "chop", "tv", "drums", "bed", "crickets", "knock"];

/// A sound of one channel by name (`MONO`); none: no such sound.
pub fn make(kind: &str, rate: f64) -> Option<Vec<f32>> {
    Some(match kind {
        "sizzle" => sizzle(rate),
        "chop" => chop(rate),
        "tv" => murmur(rate),
        "drums" => drums(rate),
        "bed" => bed(rate),
        "crickets" => crickets(rate),
        "knock" => knock(rate),
        _ => return None,
    })
}

/// A hot pan: hiss with crackles.
fn sizzle(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(4.0);
    let d = raw(rate, 2.0, |d| {
        let mut pop = 0.0;
        let mut pop_level = 0.0;
        for v in d.iter_mut() {
            if pop <= 0.0 && r.next() < 60.0 / rate {
                pop = js_round((0.002 + r.next() * 0.006) * rate);
                pop_level = 0.3 + r.next() * 0.7;
            }
            let crackle = if pop > 0.0 {
                let left = pop;
                pop -= 1.0;
                (r.next() * 2.0 - 1.0) * pop_level * (left / (0.008 * rate))
            } else {
                0.0
            };
            *v = ((r.next() * 2.0 - 1.0) * 0.18 + crackle) as f32;
        }
    });
    node_filter(&d, rate, Filter::Highpass, 1800.0, 0.5, true)
}

/// A knife on a chopping board: short woody knocks in uneven runs. Each chop is the blade's ring (its tone and
/// an overtone over it, gone in a few hundredths of a second), the board's thump under it and a hiss of noise
/// as the blade lands, rung as a bank of modes (`Modal`); the noise drawn from a generator of its own, so the
/// rhythm is the runs' alone.
fn chop(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(9.0);
    let mut grit = Rng::new(109.0);
    let d = raw(rate, 3.0, |d| {
        let n = d.len();
        let len = round_usize(0.06 * rate);
        let fade = 0.005 * rate;
        let mut i = 0;
        while i < n {
            // A run of quick chops, then a pause to move the food along.
            let chops = 3.0 + (r.next() * 6.0).floor();
            let mut c = 0.0;
            while c < chops && i < n {
                let level = 0.6 + r.next() * 0.4;
                let tone = 900.0 + r.next() * 500.0;
                let mode = |hz, tau, level| Mode {
                    hz,
                    tau,
                    level,
                    phase: 0.0,
                };
                // (The overtone and the thump kept low, the noise's share as the old chop's: the whole is set
                // by its peak before the band, so what the band then takes off only makes the chop quieter.)
                let mut ring = Modal::new(
                    rate,
                    &[
                        mode(tone, 0.011, 1.0),
                        mode(tone * 2.76, 0.006, 0.2),
                        mode(210.0, 0.018, 0.12),
                    ],
                );
                let mut land = Decay::new(0.33, 0.011, rate);
                let mut j = 0;
                while j < len && i + j < n {
                    // (Out over the last 5 ms, so the board's thump is not cut off.)
                    let out = min(1.0, (len - j) as f64 / fade);
                    d[i + j] = ((ring.next() + (grit.next() * 2.0 - 1.0) * land.next())
                        * level
                        * out) as f32;
                    j += 1;
                }
                i += round_usize((0.16 + r.next() * 0.08) * rate);
                c += 1.0;
            }
            i += round_usize((0.4 + r.next() * 0.6) * rate);
        }
    });
    node_filter(&d, rate, Filter::Bandpass, 1400.0, 0.8, true)
}

/// Voices through a wall: noise shaped into syllables, phrases and pauses (the TV).
fn murmur(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(6.0);
    let d = raw(rate, 5.0, |d| {
        let n = d.len();
        let mut i = 0;
        while i < n {
            // A phrase of a few syllables, then a pause.
            let syllables = 3.0 + (r.next() * 7.0).floor();
            let mut s = 0.0;
            while s < syllables && i < n {
                let len = round_usize((0.09 + r.next() * 0.14) * rate);
                let level = 0.5 + r.next() * 0.5;
                let mut j = 0;
                while j < len && i < n {
                    d[i] =
                        ((r.next() * 2.0 - 1.0) * level * sin((PI * j as f64) / len as f64)) as f32;
                    j += 1;
                    i += 1;
                }
                s += 1.0;
            }
            i += round_usize((0.15 + r.next() * 0.5) * rate);
        }
    });
    node_filter(&d, rate, Filter::Bandpass, 700.0, 1.1, true)
}

/// A drum kit played: a rock beat (kick, snare on two and four, the hi-hat on the eighths) at 110 bpm, four bars
/// with a crash at the top and a fill on the toms at the end.
fn drums(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(10.0);
    let beat = 60.0 / 110.0;
    let d = raw(rate, 16.0 * beat, |d| {
        let mut kit = Kit { d, r: &mut r, rate };
        kit.cymbal(0.0, 0.8, 0.35);
        for bar in 0..4 {
            let b = (bar * 4) as f64 * beat;
            let fill = bar == 3;
            for e in 0..(if fill { 4 } else { 8 }) {
                kit.cymbal(
                    b + e as f64 * (beat / 2.0),
                    0.03,
                    if e % 2 == 1 { 0.12 } else { 0.2 },
                );
            }
            let kicks: &[f64] = if fill { &[0.0] } else { &[0.0, 2.0, 2.5] };
            for &k in kicks {
                kit.skin(b + k * beat, 120.0, 48.0, 0.16, 0.9);
            }
            let snares: &[f64] = if fill { &[1.0] } else { &[1.0, 3.0] };
            for &s in snares {
                kit.snare(b + s * beat, 0.7);
            }
        }
        // The fill: sixteenths down the toms.
        for n in 0..8 {
            let tom = if n < 3 {
                210.0
            } else if n < 6 {
                160.0
            } else {
                115.0
            };
            kit.skin(
                14.0 * beat + n as f64 * (beat / 4.0),
                tom * 1.3,
                tom,
                0.12,
                0.5 + 0.1 * (n % 2) as f64,
            );
        }
    });
    node_filter(&d, rate, Filter::Lowpass, 7000.0, 0.7, true)
}

/// The drums' loop being made.
struct Kit<'a> {
    d: &'a mut [f32],
    r: &'a mut Rng,
    rate: f64,
}

impl Kit<'_> {
    /// A hit: `f` of its seconds and its noise's change each sample; its tail runs on past the loop's end into its
    /// start.
    fn hit(&mut self, at: f64, seconds: f64, level: f64, mut f: impl FnMut(f64, f64) -> f64) {
        let rate = self.rate;
        let n = self.d.len();
        let s = round_usize(at * rate);
        let mut last = 0.0;
        let mut j = 0;
        while (j as f64) < seconds * rate {
            let noise = self.r.next() * 2.0 - 1.0;
            let v = level * (1.0 + 0.1 * (self.r.next() - 0.5)) * f(j as f64 / rate, noise - last);
            add(self.d, (s + j) % n, v);
            last = noise;
            j += 1;
        }
    }

    /// A pitched skin: a sine falling from `from` to `to` Hz as it dies away, with a little noise in its attack.
    fn skin(&mut self, at: f64, from: f64, to: f64, decay: f64, level: f64) {
        let rate = self.rate;
        let mut phase = 0.0;
        self.hit(at, decay * 5.0, level, |t, noise| {
            phase += (2.0 * PI * (to + (from - to) * exp(-t / 0.03))) / rate;
            exp(-t / decay) * sin(phase) + noise * 0.15 * exp(-t / 0.004)
        });
    }

    fn snare(&mut self, at: f64, level: f64) {
        let rate = self.rate;
        let mut phase = 0.0;
        self.hit(at, 0.4, level, |t, noise| {
            phase += (2.0 * PI * 190.0) / rate;
            0.5 * noise * exp(-t / 0.08) + 0.6 * sin(phase) * exp(-t / 0.04)
        });
    }

    /// Cymbals: high noise (each sample's change, so only the highs), short for the hi-hat, long for the crash.
    fn cymbal(&mut self, at: f64, decay: f64, level: f64) {
        self.hit(at, decay * 5.0, level, |t, noise| noise * exp(-t / decay));
    }
}

/// Soft, low noise: wind and a distant street.
fn bed(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(2.0);
    let d = raw(rate, 4.0, |d| {
        let mut b = 0.0;
        for v in d.iter_mut() {
            b = (b + 0.02 * (r.next() * 2.0 - 1.0)) / 1.02;
            *v = b as f32;
        }
        // Ease the loop's seam.
        let len = d.len();
        let n = 2000.min(len >> 2);
        for i in 0..n {
            let x = i as f64 / n as f64;
            d[len - n + i] = (d[len - n + i] as f64 * (1.0 - x) + d[i] as f64 * x) as f32;
        }
    });
    node_filter(&d, rate, Filter::Lowpass, 650.0, 0.5, true)
}

/// Two crickets, each chirping in short trills.
fn crickets(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(3.0);
    raw(rate, 3.0, |d| {
        for (freq, level, offset) in [(4600.0, 1.0, 0.0), (4250.0, 0.6, 0.21)] {
            let mut start = offset;
            while start < 2.8 {
                for p in 0..3 {
                    let s = round_usize((start + p as f64 * 0.032) * rate);
                    let len = round_usize(0.014 * rate);
                    let mut j = 0;
                    while j < len && s + j < d.len() {
                        let env = sin((PI * j as f64) / len as f64);
                        add(
                            d,
                            s + j,
                            level * env * sin((2.0 * PI * freq * j as f64) / rate),
                        );
                        j += 1;
                    }
                }
                start += 0.42 + r.next() * 0.12;
            }
        }
    })
}

/// Where in a second of noise each of the knock's three clicks is read from.
const CLICKS: [f64; 3] = [0.25, 0.5, 0.75];

/// A door's panel struck: its modes (Hz, the time to die to a third in s, level), the lowest the panel
/// flexing whole.
const PANEL: [[f64; 3]; 6] = [
    [98.0, 0.040, 1.0],
    [171.0, 0.032, 0.7],
    [247.0, 0.026, 0.5],
    [388.0, 0.018, 0.32],
    [571.0, 0.012, 0.2],
    [840.0, 0.008, 0.1],
];

/// Three knocks on a wooden door: each the door's panel ringing (a bank of its modes, `Modal`) with a click of
/// noise (round 1400 Hz) on top where the knuckle lands; the third a touch late.
fn knock(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(1.0);
    let white = raw(rate, 1.0, |d| {
        for v in d.iter_mut() {
            *v = (r.next() * 2.0 - 1.0) as f32;
        }
    });
    let mut d = vec![0.0f32; round_usize(0.6 * rate)];
    // One knock of the panel, rung once, its peak 0.45; each knock a copy at its own level, ringing on under the
    // next.
    let modes = PANEL.map(|[hz, tau, level]| Mode {
        hz,
        tau,
        level,
        phase: 0.0,
    });
    let mut bank = Modal::new(rate, &modes);
    let mut panel: Vec<f64> = (0..d.len()).map(|_| bank.next()).collect();
    let peak = panel.iter().fold(0.0f64, |p, v| max(p, v.abs()));
    for v in panel.iter_mut() {
        *v *= 0.45 / peak;
    }
    for (n, click) in CLICKS.iter().enumerate() {
        let at = 0.02 + n as f64 * 0.17 + if n == 2 { 0.02 } else { 0.0 };
        let first = sample_at(at, rate);
        let level = [1.0, 0.85, 0.9][n];
        for (j, v) in panel.iter().take(d.len() - first).enumerate() {
            add(&mut d, first + j, v * level);
        }
        // The click: a moment of the noise through a band, gone in 30 ms.
        let len = sample_at(at + 0.04, rate) - first;
        let from = round_usize(click * rate).min(white.len());
        let band = node_filter(
            &white[from..(from + len).min(white.len())],
            rate,
            Filter::Bandpass,
            1400.0,
            1.2,
            false,
        );
        let mut j = 0;
        while j < len && first + j < d.len() {
            let fade = sweep(
                &[0.25, 0.0001],
                &[0.0, 0.03],
                (first + j) as f64 / rate - at,
            );
            add(&mut d, first + j, band[j] as f64 * fade);
            j += 1;
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::m::FRAC_1_SQRT_2;

    const RATE: f64 = 48000.0;

    /// The biggest step between neighbouring samples, and the step from the last sample round to the first.
    fn steps(ch: &[f32]) -> (f64, f64) {
        let inside = ch
            .windows(2)
            .map(|w| (w[1] as f64 - w[0] as f64).abs())
            .fold(0.0, max);
        (inside, (ch[0] as f64 - ch[ch.len() - 1] as f64).abs())
    }

    #[test]
    fn the_sounds_of_one_channel_are_the_same_every_time() {
        for kind in KINDS {
            let a = make(kind, RATE).unwrap();
            assert!(!a.is_empty());
            assert_eq!(a, make(kind, RATE).unwrap(), "{kind}");
        }
        assert_eq!(make("nothing", RATE), None);
    }

    #[test]
    fn a_filter_as_the_browsers_node_computes_it() {
        // A low-pass passes the lows, stops the highs, and a loop comes out a loop.
        let tone = |hz: f64| -> Vec<f32> {
            (0..RATE as usize)
                .map(|i| sin((2.0 * PI * hz * i as f64) / RATE) as f32)
                .collect()
        };
        let rms =
            |d: &[f32]| sqrt(d.iter().map(|&v| v as f64 * v as f64).sum::<f64>() / d.len() as f64);
        let db = |d: &[f32]| 20.0 * log10(rms(d) / FRAC_1_SQRT_2);
        let low = node_filter(&tone(100.0), RATE, Filter::Lowpass, 1000.0, 0.0, true);
        let high = node_filter(&tone(8000.0), RATE, Filter::Lowpass, 1000.0, 0.0, true);
        assert!((rms(&low) - FRAC_1_SQRT_2).abs() < 0.005);
        // Two poles: 12 dB an octave, three octaves up.
        assert!(db(&high) < -34.0);
        // At its cutoff a low-pass is as far up as its Q says in dB (the node's way; 0: level with the lows); a
        // band-pass passes its centre whole.
        assert!(
            db(&node_filter(
                &tone(1000.0),
                RATE,
                Filter::Lowpass,
                1000.0,
                0.0,
                true
            ))
            .abs()
                < 0.005
        );
        assert!(
            (db(&node_filter(
                &tone(1000.0),
                RATE,
                Filter::Highpass,
                1000.0,
                6.0,
                true
            )) - 6.0)
                .abs()
                < 0.005
        );
        assert!(
            (rms(&node_filter(
                &tone(1000.0),
                RATE,
                Filter::Bandpass,
                1000.0,
                1.0,
                true
            )) - FRAC_1_SQRT_2)
                .abs()
                < 0.0005
        );
        // Whole periods in a second: the loop's last sample runs on into its first.
        let (inside, seam) = steps(&low);
        assert!(seam <= inside);
    }

    #[test]
    fn a_sweep_moves_by_a_constant_ratio_and_holds_its_last_value() {
        let f = |t| sweep(&[220.0, 95.0], &[0.0, 0.08], t);
        assert_eq!(f(0.0), 220.0);
        assert!((f(0.04) - sqrt(220.0 * 95.0)).abs() < 5e-10);
        assert_eq!(f(0.08), 95.0);
        assert_eq!(f(1.0), 95.0);
    }

    #[test]
    fn the_knock_is_three_knocks() {
        let d = make("knock", RATE).unwrap();
        // Loud within 20 ms of each knock's start, quiet just before it.
        let level = |from: f64, to: f64| {
            d[round_usize(from * RATE)..round_usize(to * RATE)]
                .iter()
                .map(|v| v.abs() as f64)
                .fold(f64::NEG_INFINITY, max)
        };
        for at in [0.02, 0.19, 0.38] {
            assert!(level(at, at + 0.02) > 0.3, "{at}");
            assert!(level(at - 0.015, at - 0.001) < 0.05, "{at}");
        }
    }
}
