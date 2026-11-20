use super::*;

// --- mono.ts's parts ------------------------------------------------------------------------------

/// The first sample at or after `seconds`.
pub fn sample_at(seconds: f64, rate: f64) -> usize {
    (seconds * rate).ceil().max(0.0) as usize
}

/// `seconds` of samples filled by `fill`, then brought to a peak of 1.
pub fn raw(rate: f64, seconds: f64, fill: impl FnOnce(&mut [f32])) -> Vec<f32> {
    let mut data = vec![0.0f32; round_usize(rate * seconds)];
    fill(&mut data);
    let mut peak: f64 = 0.0;
    for &v in &data {
        peak = max(peak, (v as f64).abs());
    }
    if peak > 0.0 {
        for v in data.iter_mut() {
            *v = (*v as f64 / peak) as f32;
        }
    }
    data
}

/// A value moving through `values` at `times` (seconds, rising) the way an audio parameter does under
/// `exponentialRampToValueAtTime`: each step a constant ratio a second, held at the last value after it.
pub fn sweep(values: &[f64], times: &[f64], t: f64) -> f64 {
    for k in 1..values.len() {
        if t < times[k] {
            return values[k - 1]
                * pow(
                    values[k] / values[k - 1],
                    (t - times[k - 1]) / (times[k] - times[k - 1]),
                );
        }
    }
    values[values.len() - 1]
}

/// `x` through a two-pole filter as a Web Audio `BiquadFilterNode` computes it (its `Q` in dB for a low-pass
/// and a high-pass, a plain quality factor for a band-pass). `looped`: `x` is a loop, and the filter has run
/// over its end before its start, so the result loops as smoothly as `x` did.
pub fn node_filter(
    x: &[f32],
    rate: f64,
    kind: Filter,
    frequency: f64,
    q: f64,
    looped: bool,
) -> Vec<f32> {
    let w = (2.0 * PI * min(frequency, rate / 2.0)) / rate;
    let c = cos(w);
    let alpha = sin(w)
        / (2.0
            * if kind == Filter::Bandpass {
                q
            } else {
                pow(10.0, q / 20.0)
            });
    let a0 = 1.0 + alpha;
    let edge = if kind == Filter::Lowpass {
        (1.0 - c) / 2.0
    } else {
        (1.0 + c) / 2.0
    };
    let b0 = (if kind == Filter::Bandpass {
        alpha
    } else {
        edge
    }) / a0;
    let b1 = (match kind {
        Filter::Bandpass => 0.0,
        Filter::Lowpass => 2.0 * edge,
        Filter::Highpass => -2.0 * edge,
    }) / a0;
    let b2 = (if kind == Filter::Bandpass {
        -alpha
    } else {
        edge
    }) / a0;
    let a1 = (-2.0 * c) / a0;
    let a2 = (1.0 - alpha) / a0;
    let n = x.len() as i64;
    let mut y = vec![0.0f32; x.len()];
    let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
    let from = if looped { -(n.min(rate as i64)) } else { 0 };
    for i in from..n {
        let v = x[if i < 0 { (n + i) as usize } else { i as usize }] as f64;
        let out = b0 * v + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
        x2 = x1;
        x1 = v;
        y2 = y1;
        y1 = out;
        if i >= 0 {
            y[i as usize] = out as f32;
        }
    }
    y
}
