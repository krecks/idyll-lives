//! Footsteps, synthesised (no sound files): one step on a wooden floor, tiles, carpet, grass or paving, in
//! shoes or barefoot, a few versions of each so a walk isn't one sound over and over. A step is samples made at
//! a sample rate, the same every time; `footsteps.ts` plays them as the residents in view put their feet down
//! (was `web/src/audio/steps.ts`).
//!
//! A step is the heel coming down and, a moment later, the ball of the foot: on a hard floor each a click of
//! the sole (a short band of noise) over the floor's own knock (a low tone that dies at once: a board lower and
//! longer than a tile); on carpet a dull pat with no click; on grass the blades brushed aside (a soft swell of
//! high noise) over soft ground; on paving the grit under the sole. Barefoot there is no sole to click: a soft
//! slap on a hard floor, next to nothing on carpet and grass.

use crate::dsp::m::*;
use crate::dsp::*;

/// The surfaces underfoot, in the TypeScript's order (it seeds each step).
pub const SURFACES: [&str; 5] = ["wood", "tile", "carpet", "grass", "paving"];
/// What is on the feet.
pub const FOOTWEAR: [&str; 2] = ["shoe", "bare"];
/// Versions of each step.
pub const STEP_VERSIONS: u32 = 4;
/// Seconds a step lasts at most.
const SECONDS: f64 = 0.3;

/// A low knock: a tone at `hz` (starting a third higher, as a struck thing does) gone in a few times `decay` s.
fn knock(d: &mut [f32], rate: f64, at: f64, hz: f64, decay: f64, amp: f64) {
    let s = round_usize(at * rate);
    let mut phase = 0.0;
    for i in s..d.len() {
        let t = (i - s) as f64 / rate;
        if t > decay * 7.0 {
            break;
        }
        phase += (2.0 * PI * hz * (1.0 + 0.33 * exp(-t / 0.008))) / rate;
        add(
            d,
            i,
            amp * sin(phase) * exp(-t / decay) * min(1.0, t / 0.0015),
        );
    }
}

/// A puff of noise through a filter, up in `attack` s and gone in a few times `decay` s.
#[allow(clippy::too_many_arguments)]
fn puff(
    d: &mut [f32],
    rate: f64,
    r: &mut Rng,
    at: f64,
    kind: Filter,
    hz: f64,
    q: f64,
    attack: f64,
    decay: f64,
    amp: f64,
) {
    let mut f = Biquad::with(rate, kind, hz, q);
    let s = round_usize(at * rate);
    for i in s..d.len() {
        let t = (i - s) as f64 / rate;
        if t > attack + decay * 7.0 {
            break;
        }
        add(
            d,
            i,
            amp * f.run(r.next() * 2.0 - 1.0)
                * min(1.0, t / attack)
                * exp(-max(0.0, t - attack) / decay),
        );
    }
}

/// One step on `surface` in `wear`, version `version` (0 to `STEP_VERSIONS` - 1: each a little different in pitch,
/// weight and the time between heel and ball), as samples of one channel with a peak of 1 (`stepSamples`). None: no
/// such surface or footwear.
pub fn make(surface: &str, wear: &str, version: u32, rate: f64) -> Option<Vec<f32>> {
    use Filter::{Bandpass as Band, Highpass as High, Lowpass as Low};
    let si = SURFACES.iter().position(|s| *s == surface)?;
    let wi = FOOTWEAR.iter().position(|w| *w == wear)?;
    let mut r = Rng::new(0x57e9 as f64 + si as f64 * 64.0 + wi as f64 * 16.0 + version as f64);
    let mut d = vec![0.0f32; round_usize(SECONDS * rate)];
    // This version's own pitch and weight, and when the ball of the foot follows the heel.
    let tune = 0.9 + 0.2 * r.next();
    let heel = 0.85 + 0.3 * r.next();
    let ball = 0.065 + 0.03 * r.next();
    let at = 0.004;
    let (d, r) = (&mut d[..], &mut r);
    if wear == "bare" {
        match surface {
            "wood" => {
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Low,
                    900.0 * tune,
                    0.7,
                    0.003,
                    0.022,
                    0.8 * heel,
                );
                knock(d, rate, at, 95.0 * tune, 0.04, 0.5 * heel);
                puff(
                    d,
                    rate,
                    r,
                    at + ball,
                    Low,
                    1200.0 * tune,
                    0.7,
                    0.003,
                    0.016,
                    0.45,
                );
            }
            "tile" => {
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Low,
                    1500.0 * tune,
                    0.7,
                    0.002,
                    0.018,
                    0.9 * heel,
                );
                knock(d, rate, at, 130.0 * tune, 0.02, 0.25 * heel);
                puff(
                    d,
                    rate,
                    r,
                    at + ball,
                    Band,
                    1900.0 * tune,
                    0.8,
                    0.002,
                    0.014,
                    0.6,
                );
            }
            "carpet" => {
                puff(d, rate, r, at, Low, 320.0 * tune, 0.7, 0.005, 0.035, heel);
                puff(
                    d,
                    rate,
                    r,
                    at + ball,
                    Low,
                    420.0 * tune,
                    0.7,
                    0.005,
                    0.025,
                    0.4,
                );
            }
            "grass" => {
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Band,
                    2600.0 * tune,
                    0.5,
                    0.014,
                    0.06,
                    0.5 * heel,
                );
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Low,
                    260.0 * tune,
                    0.7,
                    0.005,
                    0.04,
                    0.8 * heel,
                );
            }
            _ => {
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Low,
                    750.0 * tune,
                    0.7,
                    0.003,
                    0.02,
                    0.9 * heel,
                );
                knock(d, rate, at, 100.0 * tune, 0.02, 0.3 * heel);
                puff(
                    d,
                    rate,
                    r,
                    at + ball,
                    Low,
                    950.0 * tune,
                    0.7,
                    0.003,
                    0.015,
                    0.45,
                );
            }
        }
    } else {
        match surface {
            "wood" => {
                knock(d, rate, at, 110.0 * tune, 0.045, 0.9 * heel);
                knock(d, rate, at, 215.0 * tune, 0.028, 0.4 * heel);
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Band,
                    1700.0 * tune,
                    1.2,
                    0.001,
                    0.012,
                    1.6 * heel,
                );
                knock(d, rate, at + ball, 145.0 * tune, 0.03, 0.35);
                puff(
                    d,
                    rate,
                    r,
                    at + ball,
                    Band,
                    2200.0 * tune,
                    1.2,
                    0.001,
                    0.008,
                    0.8,
                );
            }
            "tile" => {
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Band,
                    3200.0 * tune,
                    1.5,
                    0.0008,
                    0.01,
                    2.4 * heel,
                );
                puff(d, rate, r, at, High, 5000.0, 0.7, 0.0005, 0.004, 0.3 * heel);
                knock(d, rate, at, 165.0 * tune, 0.02, 0.35 * heel);
                puff(
                    d,
                    rate,
                    r,
                    at + ball,
                    Band,
                    3800.0 * tune,
                    1.5,
                    0.0008,
                    0.007,
                    1.1,
                );
                knock(d, rate, at + ball, 195.0 * tune, 0.015, 0.15);
            }
            "carpet" => {
                puff(d, rate, r, at, Low, 420.0 * tune, 0.7, 0.004, 0.04, heel);
                knock(d, rate, at, 80.0 * tune, 0.04, 0.5 * heel);
                puff(
                    d,
                    rate,
                    r,
                    at + ball,
                    Low,
                    600.0 * tune,
                    0.7,
                    0.004,
                    0.03,
                    0.4,
                );
            }
            "grass" => {
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Band,
                    3500.0 * tune,
                    0.6,
                    0.012,
                    0.07,
                    0.9 * heel,
                );
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Low,
                    300.0 * tune,
                    0.7,
                    0.004,
                    0.04,
                    0.6 * heel,
                );
                puff(
                    d,
                    rate,
                    r,
                    at + ball,
                    Band,
                    4200.0 * tune,
                    0.6,
                    0.01,
                    0.045,
                    0.45,
                );
            }
            _ => {
                puff(
                    d,
                    rate,
                    r,
                    at,
                    Band,
                    1400.0 * tune,
                    0.8,
                    0.002,
                    0.03,
                    1.4 * heel,
                );
                puff(d, rate, r, at, High, 3500.0, 0.7, 0.001, 0.012, 0.25 * heel);
                knock(d, rate, at, 120.0 * tune, 0.025, 0.5 * heel);
                puff(
                    d,
                    rate,
                    r,
                    at + ball,
                    Band,
                    1800.0 * tune,
                    0.8,
                    0.002,
                    0.02,
                    0.7,
                );
            }
        }
    }
    // Eased to nothing at its end, then a peak of 1.
    let n = d.len();
    let ease = round_usize(0.03 * rate);
    for i in 0..ease {
        d[n - 1 - i] = (d[n - 1 - i] as f64 * (i as f64 / ease as f64)) as f32;
    }
    let mut peak: f64 = 0.0;
    for &v in d.iter() {
        peak = max(peak, (v as f64).abs());
    }
    if peak > 0.0 {
        for v in d.iter_mut() {
            *v = (*v as f64 / peak) as f32;
        }
    }
    Some(d.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 48000.0;

    /// How bright a sound is: the root mean square of its sample-to-sample change over that of its samples, as
    /// a frequency (Hz).
    fn brightness(d: &[f32]) -> f64 {
        let (mut change, mut level) = (0.0, 0.0);
        for i in 1..d.len() {
            let (a, b) = (d[i - 1] as f64, d[i] as f64);
            change += (b - a) * (b - a);
            level += b * b;
        }
        (change / level).sqrt() * RATE / (2.0 * PI)
    }

    fn bright(surface: &str, wear: &str) -> f64 {
        (0..STEP_VERSIONS)
            .map(|v| brightness(&make(surface, wear, v, RATE).unwrap()))
            .sum::<f64>()
            / STEP_VERSIONS as f64
    }

    #[test]
    fn whole_short_starts_and_ends_at_nothing_and_the_same_every_time() {
        for surface in SURFACES {
            for wear in FOOTWEAR {
                for v in 0..STEP_VERSIONS {
                    let d = make(surface, wear, v, RATE).unwrap();
                    assert_eq!(make(surface, wear, v, RATE).unwrap(), d);
                    assert!(d.len() as f64 / RATE <= 0.3);
                    let (mut peak, mut sum) = (0.0f64, 0.0);
                    for &x in &d {
                        peak = peak.max((x as f64).abs());
                        sum += x as f64 * x as f64;
                    }
                    assert!(sum.is_finite());
                    assert!(
                        (peak - 1.0).abs() < 5e-6,
                        "{surface} {wear} {v}: peak {peak}"
                    );
                    assert!(d[0].abs() < 0.01);
                    assert!(d[d.len() - 1].abs() < 0.001);
                    // Most of it is over in the first 0.2 s.
                    let late: f64 = d[round_usize(0.2 * RATE)..]
                        .iter()
                        .map(|&x| x as f64 * x as f64)
                        .sum();
                    assert!(late / sum < 0.02, "{surface} {wear} {v}");
                }
                // The versions differ.
                assert_ne!(make(surface, wear, 0, RATE), make(surface, wear, 1, RATE));
            }
        }
        assert!(make("sand", "shoe", 0, RATE).is_none());
    }

    #[test]
    fn each_surface_sounds_its_own() {
        // Carpet dull, a board a knock, paving gritty, tiles a click, grass a rustle.
        let shoe = |s| bright(s, "shoe");
        let (carpet, wood, paving, tile, grass) = (
            shoe("carpet"),
            shoe("wood"),
            shoe("paving"),
            shoe("tile"),
            shoe("grass"),
        );
        assert!(carpet < 400.0, "{carpet}");
        assert!(wood > carpet * 2.0);
        assert!(paving > wood * 2.0);
        assert!(tile > paving);
        assert!(grass > 3000.0, "{grass}");
        // Bare feet have no sole to click: duller on every hard floor.
        for s in ["wood", "tile", "paving"] {
            assert!(bright(s, "bare") < shoe(s) * 0.6, "{s}");
        }
    }
}
