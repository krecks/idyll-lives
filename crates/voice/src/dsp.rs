//! Shaping a voice by its sound: pitch, size and speed by TD-PSOLA, and the silence around a
//! clip trimmed.
//!
//! Supertonic 3 speaks in ten fixed voices at a given speed; a resident's own pitch and size
//! are put onto each finished line ([`shape`]), so everyone sounds a little different.
//!
//! TD-PSOLA: the clip is cut into grains two pitch periods long, centred one period apart (5 ms
//! apart where it isn't voiced), and put back together with the grains closer (higher) or
//! further apart (lower), each grain picked for its place in time, so length and pitch change
//! independently and the spectral envelope (the formants) stays. Size is a resample first (every
//! frequency, formants included, by α), then PSOLA puts pitch and length back. All
//! arithmetic is f64, so the same clip gives the same samples everywhere.

use crate::SAMPLE_RATE;
use crate::resample::resample;

/// Pitch analysis: 40 ms windows every 10 ms, 50–500 Hz (searched at half the rate first, then
/// refined).
const HOP: usize = 240;
const WINDOW: usize = 960;
const MIN_LAG: usize = 48;
const MAX_LAG: usize = 480;
/// A frame is voiced when its normalised autocorrelation peak is above this and it is louder
/// than `VOICED_LEVEL` of the clip's loudest frame.
const VOICED_CORR: f64 = 0.6;
const VOICED_LEVEL: f64 = 0.03;
/// Grain spacing where nothing is voiced (5 ms).
const UNVOICED_PERIOD: usize = 120;

/// Silence trimming: 10 ms frames; a frame is sound when its RMS is over 5 % of the loudest
/// frame's; 50 ms are kept before the first sound and 100 ms after the last.
const FRAME: usize = 240;
const THRESHOLD: f32 = 0.05;
const LEAD: usize = 1200;
const TRAIL: usize = 2400;

/// `samples` without the silence around the sound.
pub fn trim(samples: &[f32]) -> &[f32] {
    let rms: Vec<f32> = samples
        .chunks(FRAME)
        .map(|f| (f.iter().map(|x| x * x).sum::<f32>() / f.len() as f32).sqrt())
        .collect();
    let loudest = rms.iter().copied().fold(0.0, f32::max);
    if loudest <= 0.0 {
        return samples;
    }
    let sound = |r: &f32| *r > loudest * THRESHOLD;
    let first = rms.iter().position(sound).unwrap_or(0);
    let last = rms.iter().rposition(sound).unwrap_or(rms.len() - 1);
    let start = (first * FRAME).saturating_sub(LEAD);
    let end = ((last + 1) * FRAME + TRAIL).min(samples.len());
    &samples[start..end]
}

/// Pitch × `pitch`, formants × `depth` and speed × `speed` in one pass (resample by `depth`,
/// then PSOLA puts pitch and length where they belong).
pub fn shape(samples: &[f32], pitch: f32, depth: f32, speed: f32) -> Vec<f32> {
    let (pitch, depth, speed) = (pitch as f64, depth as f64, speed as f64);
    let same = |v: f64| (v - 1.0).abs() < 1e-3;
    if samples.is_empty() || (same(pitch) && same(depth) && same(speed)) {
        return samples.to_vec();
    }
    let resized = if same(depth) {
        samples.to_vec()
    } else {
        resample(samples, depth as f32)
    };
    psola(&resized, pitch / depth, depth / speed)
}

/// The median pitch of a clip's voiced frames, Hz (0 if none is voiced).
pub fn median_pitch(samples: &[f32]) -> f32 {
    let mut voiced: Vec<f64> = periods(samples)
        .into_iter()
        .flatten()
        .map(|p| SAMPLE_RATE as f64 / p)
        .collect();
    if voiced.is_empty() {
        return 0.0;
    }
    voiced.sort_by(f64::total_cmp);
    voiced[voiced.len() / 2] as f32
}

/// TD-PSOLA: pitch × `pitch_factor`, length × `time_factor`.
pub fn psola(x: &[f32], pitch_factor: f64, time_factor: f64) -> Vec<f32> {
    let out_len = (x.len() as f64 * time_factor).round() as usize;
    if x.len() < WINDOW || out_len == 0 {
        return resample(x, (1.0 / time_factor) as f32);
    }
    let marks = marks(x);
    let mut out = vec![0.0f64; out_len];
    let mut norm = vec![0.0f64; out_len];
    // Synthesis marks: start where the analysis does, step by the (shifted) local period.
    let mut t = marks[0].0 as f64 * time_factor;
    let mut k = 0;
    while t < out_len as f64 {
        // The analysis mark nearest to this point of the original's time.
        let source = t / time_factor;
        while k + 1 < marks.len()
            && (marks[k + 1].0 as f64 - source).abs() <= (marks[k].0 as f64 - source).abs()
        {
            k += 1;
        }
        let (centre, period, voiced) = marks[k];
        let at = t.round() as isize;
        let half = period as isize;
        for d in -half..half {
            let i = centre as isize + d;
            let o = at + d;
            if i < 0 || i as usize >= x.len() || o < 0 || o as usize >= out_len {
                continue;
            }
            // Hann over the grain (two periods).
            let w = 0.5 + 0.5 * (std::f64::consts::PI * d as f64 / half as f64).cos();
            out[o as usize] += w * x[i as usize] as f64;
            norm[o as usize] += w;
        }
        let step = if voiced {
            period as f64 / pitch_factor
        } else {
            period as f64
        };
        t += step.max(1.0);
    }
    out.iter()
        .zip(&norm)
        .map(|(&v, &n)| (if n > 0.1 { v / n } else { v }) as f32)
        .collect()
}

/// Analysis marks `(sample, period, voiced)`, one period apart (5 ms where unvoiced), over the
/// whole clip.
fn marks(x: &[f32]) -> Vec<(usize, usize, bool)> {
    let periods = periods(x);
    let period_at = |i: usize| periods.get(i / HOP).copied().flatten();
    let mut out = Vec::new();
    let mut at = 0usize;
    while at < x.len() {
        let (period, voiced) = match period_at(at) {
            Some(p) => (p.round() as usize, true),
            None => (UNVOICED_PERIOD, false),
        };
        out.push((at, period, voiced));
        at += period.max(1);
    }
    out
}

/// The pitch period (samples) of each analysis frame, `None` where unvoiced; median-smoothed
/// over five frames, so a stray octave error doesn't move a grain. The lag is searched on the
/// signal at half the rate (a quarter of the work), then refined at the full rate.
fn periods(x: &[f32]) -> Vec<Option<f64>> {
    let frames = x.len().div_ceil(HOP);
    let half: Vec<f32> = x
        .chunks(2)
        .map(|p| p.iter().sum::<f32>() / p.len() as f32)
        .collect();
    let energy = |i: usize| {
        let a = (i * HOP).saturating_sub(WINDOW / 2);
        let b = (a + WINDOW).min(x.len());
        (x[a..b].iter().map(|&v| v as f64 * v as f64).sum::<f64>() / (b - a).max(1) as f64).sqrt()
    };
    let levels: Vec<f64> = (0..frames).map(energy).collect();
    let loudest = levels.iter().copied().fold(0.0, f64::max);
    // Normalised correlation of `s` at `a` with itself `lag` later, over `n` samples.
    let ncorr = |s: &[f32], a: usize, lag: usize, n: usize| {
        let (frame, other) = (&s[a..a + n], &s[a + lag..a + lag + n]);
        let (mut c, mut e0, mut e1) = (0.0f64, 0.0f64, 0.0f64);
        for (&p, &q) in frame.iter().zip(other) {
            c += p as f64 * q as f64;
            e0 += p as f64 * p as f64;
            e1 += q as f64 * q as f64;
        }
        c / ((e0 * e1).sqrt() + 1e-12)
    };
    let raw: Vec<Option<f64>> = (0..frames)
        .map(|i| {
            if loudest <= 0.0 || levels[i] < loudest * VOICED_LEVEL {
                return None;
            }
            let a = (i * HOP).saturating_sub(WINDOW / 2);
            if a + WINDOW + MAX_LAG + 2 > x.len() {
                return None;
            }
            let (ha, hn) = (a / 2, WINDOW / 2);
            let r: Vec<f64> = (MIN_LAG / 2..=MAX_LAG / 2)
                .map(|lag| ncorr(&half, ha, lag, hn))
                .collect();
            let top = r.iter().copied().fold(0.0, f64::max);
            // The shortest lag that peaks nearly as high as the best: a multiple of the period
            // can match a little better (whole samples), but it is not the pitch.
            let peak = (1..r.len() - 1)
                .find(|&k| r[k] >= 0.9 * top && r[k] >= r[k - 1] && r[k] >= r[k + 1]);
            let coarse = 2
                * (MIN_LAG / 2
                    + peak.unwrap_or_else(|| r.iter().position(|&v| v == top).unwrap_or(0)));
            // Refined at the full rate around it, then a parabola through the peak.
            let (best, lag) = (coarse.saturating_sub(2).max(MIN_LAG)..=(coarse + 2).min(MAX_LAG))
                .map(|l| (ncorr(x, a, l, WINDOW), l))
                .fold((f64::MIN, coarse), |b, c| if c.0 > b.0 { c } else { b });
            (best > VOICED_CORR).then(|| {
                if lag > MIN_LAG && lag < MAX_LAG {
                    let (l, m, r) = (
                        ncorr(x, a, lag - 1, WINDOW),
                        best,
                        ncorr(x, a, lag + 1, WINDOW),
                    );
                    let den = l - 2.0 * m + r;
                    if den.abs() > 1e-12 {
                        return lag as f64 + (0.5 * (l - r) / den).clamp(-0.5, 0.5);
                    }
                }
                lag as f64
            })
        })
        .collect();
    (0..frames)
        .map(|i| {
            raw[i]?;
            let mut near: Vec<f64> = raw[i.saturating_sub(2)..(i + 3).min(frames)]
                .iter()
                .flatten()
                .copied()
                .collect();
            near.sort_by(f64::total_cmp);
            Some(near[near.len() / 2])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f64 = SAMPLE_RATE as f64;

    /// A buzz at `f0` (harmonics to 4 kHz) through two fixed resonances, like a vowel.
    fn vowel(f0: f64, seconds: f64) -> Vec<f32> {
        let n = (seconds * SR) as usize;
        let mut x = vec![0.0f32; n];
        let mut h = 1.0;
        while h * f0 < 4000.0 {
            let f = h * f0;
            let gain = 1.0 / (1.0 + ((f - 700.0) / 150.0).powi(2))
                + 0.6 / (1.0 + ((f - 1800.0) / 200.0).powi(2));
            for (i, v) in x.iter_mut().enumerate() {
                *v += (gain * (2.0 * std::f64::consts::PI * f * i as f64 / SR).sin()) as f32 * 0.1;
            }
            h += 1.0;
        }
        x
    }

    /// Mean spectral centroid below 4 kHz (a direct DFT on a few frames).
    fn centroid(x: &[f32]) -> f64 {
        let n = 1024;
        let (mut sum, mut count) = (0.0, 0);
        for at in (n..x.len().saturating_sub(2 * n)).step_by(4 * n) {
            let (mut num, mut den) = (0.0, 0.0);
            for k in 1..(4000 * n / SR as usize) {
                let (mut re, mut im) = (0.0, 0.0);
                for i in 0..n {
                    let w = 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos();
                    let p = 2.0 * std::f64::consts::PI * (k * i) as f64 / n as f64;
                    re += w * x[at + i] as f64 * p.cos();
                    im += w * x[at + i] as f64 * p.sin();
                }
                let m = (re * re + im * im).sqrt();
                num += m * k as f64 * SR / n as f64;
                den += m;
            }
            sum += num / den;
            count += 1;
        }
        sum / count as f64
    }

    #[test]
    fn finds_the_pitch() {
        for f0 in [100.0, 150.0, 220.0, 300.0] {
            let p = median_pitch(&vowel(f0, 0.5)) as f64;
            assert!((p - f0).abs() < f0 * 0.01, "{f0} Hz read as {p}");
        }
        assert_eq!(median_pitch(&vec![0.0; 24_000]), 0.0);
    }

    #[test]
    fn pitch_moves_alone() {
        let x = vowel(150.0, 1.0);
        let c0 = centroid(&x);
        for factor in [0.8, 1.2] {
            let y = shape(&x, factor, 1.0, 1.0);
            let p = median_pitch(&y) as f64;
            assert!(
                (p - 150.0 * factor as f64).abs() < 150.0 * 0.03,
                "x{factor}: {p} Hz"
            );
            assert!(
                (y.len() as f64 - x.len() as f64).abs() < x.len() as f64 * 0.02,
                "x{factor}: length {}",
                y.len()
            );
            // The resonances stay where they were: the centroid moves far less than the pitch.
            let c = centroid(&y);
            assert!(
                (c / c0 - 1.0).abs() < 0.08,
                "x{factor}: centroid {c0:.0} to {c:.0}"
            );
        }
    }

    #[test]
    fn size_moves_the_formants_not_the_pitch() {
        let x = vowel(150.0, 1.0);
        let c0 = centroid(&x);
        for depth in [0.85f32, 1.15] {
            let y = shape(&x, 1.0, depth, 1.0);
            let p = median_pitch(&y) as f64;
            assert!((p - 150.0).abs() < 150.0 * 0.03, "depth {depth}: {p} Hz");
            assert!((y.len() as f64 - x.len() as f64).abs() < x.len() as f64 * 0.02);
            let c = centroid(&y);
            assert!(
                (c / c0 - depth as f64).abs() < 0.06,
                "depth {depth}: centroid {c0:.0} to {c:.0}"
            );
        }
    }

    #[test]
    fn speed_changes_length_not_pitch() {
        let x = vowel(200.0, 1.0);
        for speed in [0.85f32, 1.2] {
            let y = shape(&x, 1.0, 1.0, speed);
            assert!((y.len() as f64 - x.len() as f64 / speed as f64).abs() < 2.0);
            let p = median_pitch(&y) as f64;
            assert!((p - 200.0).abs() < 4.0, "speed {speed}: {p} Hz");
        }
        assert_eq!(shape(&x, 1.0, 1.0, 1.0), x);
    }

    #[test]
    fn trims_silence() {
        let mut x = vec![0.0f32; 24_000];
        for (i, v) in x[9_600..14_400].iter_mut().enumerate() {
            *v = (i as f32 * 0.1).sin() * 0.5;
        }
        let t = trim(&x);
        assert_eq!(t.len(), 4_800 + LEAD + TRAIL);
        assert_eq!(t[LEAD], x[9_600]);
    }

    #[test]
    fn deterministic() {
        let x = vowel(130.0, 0.6);
        assert_eq!(shape(&x, 1.1, 0.9, 1.05), shape(&x, 1.1, 0.9, 1.05));
    }
}
