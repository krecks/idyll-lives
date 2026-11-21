use super::*;

// --- Tones without a `sin` or an `exp` a sample -------------------------------------------------------
//
// Not the TypeScript's: a sine turned by rotation, an envelope by a ratio a sample, a bank of decaying modes
// and a plucked string. A note pays one `sin`, `cos` or `exp` where the sum it replaces paid one a sample (the
// software `libm` is most of a sound's time in wasm).

/// A sine by rotation: a point on the unit circle turned by `w` a sample, `w = 2π·hz/rate` (one `sin` and one
/// `cos` a note, or a glide's step).
#[derive(Clone, Debug)]
pub struct Osc {
    re: f64,
    im: f64,
    c: f64,
    s: f64,
    count: u32,
}

impl Osc {
    /// At `hz` from `phase` (radians): its first sample is `sin(phase)`.
    pub fn new(hz: f64, rate: f64, phase: f64) -> Osc {
        let mut o = Osc {
            re: cos(phase),
            im: sin(phase),
            c: 1.0,
            s: 0.0,
            count: 0,
        };
        o.set(hz, rate);
        o
    }

    /// A new pitch (a glide's step: every 8 to 32 samples, not every one).
    pub fn set(&mut self, hz: f64, rate: f64) {
        let w = (2.0 * PI * hz) / rate;
        self.c = cos(w);
        self.s = sin(w);
    }

    /// This sample, then a turn to the next.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f64 {
        let v = self.im;
        let re = self.re * self.c - self.im * self.s;
        self.im = self.re * self.s + self.im * self.c;
        self.re = re;
        self.count += 1;
        if self.count == 1024 {
            // Back onto the circle (one Newton step to a length of 1; the drift it takes off is about 1e-13).
            self.count = 0;
            let g = 1.5 - 0.5 * (self.re * self.re + self.im * self.im);
            self.re *= g;
            self.im *= g;
        }
        v
    }
}

/// A tone's overtones at `turns` (its phase, in turns) from one `sin` and one `cos`: `Σ weights[k]·sin((k+1)·θ)`,
/// `θ = 2π·turns`, each overtone turned from the one under it. For a phase that wanders a sample at a time (a
/// motor's speed, a bow's vibrato), where an `Osc` does not fit.
#[inline]
pub fn harmonics(turns: f64, weights: &[f64]) -> f64 {
    let Some((&first, rest)) = weights.split_first() else {
        return 0.0;
    };
    let theta = 2.0 * PI * turns;
    let (c1, s1) = (cos(theta), sin(theta));
    let (mut c, mut s) = (c1, s1);
    let mut v = first * s;
    for &w in rest {
        let next = c * c1 - s * s1;
        s = s * c1 + c * s1;
        c = next;
        v += w * s;
    }
    v
}

/// An exponential fall, by a ratio a sample (one `exp` a note): `level·e^(−n/(tau·rate))` at sample `n`.
#[derive(Clone, Debug)]
pub struct Decay {
    v: f64,
    k: f64,
}

impl Decay {
    pub fn new(level: f64, tau: f64, rate: f64) -> Decay {
        Decay {
            v: level,
            k: exp(-1.0 / (tau * rate)),
        }
    }

    /// This sample's level, then the next's.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f64 {
        let v = self.v;
        self.v *= self.k;
        v
    }
}

/// A mode of something struck: its frequency (Hz), the time it takes to die to a third (s), its level and the
/// phase it starts from (radians).
#[derive(Clone, Copy, Debug)]
pub struct Mode {
    pub hz: f64,
    pub tau: f64,
    pub level: f64,
    pub phase: f64,
}

/// A bank of decaying modes (something struck ringing out): each a point turned and drawn in a little every
/// sample, so `Σ level·e^(−t/tau)·sin(w·n + phase)` with no `sin` or `exp` a sample. Nothing keeps it on its
/// circle: it dies away by design.
#[derive(Clone, Debug, Default)]
pub struct Modal {
    /// Each mode's point (`re`, `im`) and its step (`c`, `s`: the turn and the fall).
    modes: Vec<[f64; 4]>,
}

impl Modal {
    /// The modes at `rate` (a frequency over 0.45 of the rate is held there).
    pub fn new(rate: f64, modes: &[Mode]) -> Modal {
        Modal {
            modes: modes
                .iter()
                .map(|m| {
                    let rho = exp(-1.0 / (m.tau * rate));
                    let w = (2.0 * PI * min(m.hz, 0.45 * rate)) / rate;
                    [
                        m.level * cos(m.phase),
                        m.level * sin(m.phase),
                        rho * cos(w),
                        rho * sin(w),
                    ]
                })
                .collect(),
        }
    }

    /// This sample (every mode's), then every mode turned on.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f64 {
        let mut v = 0.0;
        for m in self.modes.iter_mut() {
            let [re, im, c, s] = *m;
            v += im;
            m[0] = re * c - im * s;
            m[1] = re * s + im * c;
        }
        v
    }
}

/// A plucked string (Karplus-Strong, tuned between samples): a line of noise going round through a loss filter
/// and a tuning all-pass, at `hz`, its own note falling to a third in `tau` s. `dull` (0.1 bright .. 0.5 dull)
/// is how much the loss filter takes off the highs, `soft` (0..1) how soft the pluck (the noise smoothed), `place`
/// where along the string it is plucked (a fraction of it: the overtones with a node there are missing).
///
/// `dull` and `soft` are as heard at 48 kHz; at another rate the note is made to sound the same in Hz: the pluck's
/// smoothing as long in seconds, and the loss filter taking as much off each overtone a pass round the string
/// (its `s(1 − s)` by the square of the rate's ratio, run twice or more over above about 52 kHz, where one pass
/// cannot take that much).
#[derive(Clone, Copy, Debug)]
pub struct Pluck {
    pub hz: f64,
    pub tau: f64,
    pub dull: f64,
    pub soft: f64,
    pub place: f64,
}

impl Pluck {
    /// The note for `seconds` at `rate`, its pluck's noise drawn from `r`: in over 1 ms, out over its last 5 ms.
    pub fn play(&self, rate: f64, seconds: f64, r: &mut Rng) -> Vec<f32> {
        let len = round_usize(seconds * rate);
        let mut out = vec![0.0f32; len];
        let period = rate / self.hz;
        let w = (2.0 * PI * self.hz) / rate;
        // The loss filter: as much off each overtone at this rate as `dull` takes at 48 kHz (its `s(1 − s)` by the
        // square of the rates' ratio), in `passes` two-tap filters where one cannot take that much.
        let at48 = self.dull * (1.0 - self.dull);
        let wanted = if rate == 48000.0 {
            at48
        } else {
            at48 * (rate / 48000.0) * (rate / 48000.0)
        };
        let passes = max(1.0, (wanted / 0.25).ceil());
        let mut each = wanted / passes;
        // Where the filter alone would take more off the note itself than it should lose a period, less of it.
        let g = exp(-period / (self.tau * rate)) / 0.99995;
        if g < 1.0 {
            each = min(each, (1.0 - pow(g, 2.0 / passes)) / (2.0 * (1.0 - cos(w))));
        } else {
            each = 0.0;
        }
        let s = if passes == 1.0 && each >= at48 {
            self.dull
        } else {
            (1.0 - sqrt(max(0.0, 1.0 - 4.0 * each))) / 2.0
        };
        let passes = passes as usize;
        // Its delay and gain at the note.
        let d_lp = passes as f64 * atan2(s * sin(w), (1.0 - s) + s * cos(w)) / w;
        let gain = pow(
            sqrt((1.0 - s) * (1.0 - s) + s * s + 2.0 * s * (1.0 - s) * cos(w)),
            passes as f64,
        );
        let soft = if rate == 48000.0 {
            self.soft
        } else {
            pow(self.soft, 48000.0 / rate)
        };
        let whole = (period - d_lp - 0.1).floor();
        if whole < 2.0 {
            // Too high for the rate: a sine dying away.
            let mut osc = Osc::new(self.hz, rate, 0.0);
            let mut fall = Decay::new(1.0, self.tau, rate);
            for o in out.iter_mut() {
                *o = (osc.next() * fall.next()) as f32;
            }
        } else {
            let n = whole as usize;
            let rho = min(0.99995, exp(-period / (self.tau * rate)) / gain);
            let d = period - d_lp - whole;
            let ap = sin(((1.0 - d) * w) / 2.0) / sin(((1.0 + d) * w) / 2.0);
            // The pluck: noise, softened, less itself `place` along the string, its mean off, its peak 1.
            let mut e = vec![0.0f64; n];
            let mut last = 0.0;
            for v in e.iter_mut() {
                last = (1.0 - soft) * (r.next() * 2.0 - 1.0) + soft * last;
                *v = last;
            }
            let m = (js_round(self.place * n as f64) as usize).clamp(1, n);
            let mut line: Vec<f64> = (0..n).map(|i| e[i] - e[(i + n - m) % n]).collect();
            let mean = line.iter().sum::<f64>() / n as f64;
            let peak = line.iter().fold(0.0f64, |p, &v| max(p, (v - mean).abs()));
            let scale = if peak > 0.0 { 1.0 / peak } else { 0.0 };
            for v in line.iter_mut() {
                *v = (*v - mean) * scale;
            }
            // Each pass's last input.
            let mut before = vec![0.0f64; passes];
            let (mut v1, mut w1) = (0.0, 0.0);
            let mut i = 0;
            for o in out.iter_mut() {
                let y = line[i];
                *o = y as f32;
                let mut u = y;
                for b in before.iter_mut() {
                    let next = (1.0 - s) * u + s * *b;
                    *b = u;
                    u = next;
                }
                let v = rho * u;
                let x = ap * v + v1 - ap * w1;
                v1 = v;
                w1 = x;
                line[i] = x;
                i += 1;
                if i == n {
                    i = 0;
                }
            }
        }
        let attack = 0.001 * rate;
        let fade = 0.005 * rate;
        for (j, o) in out.iter_mut().enumerate() {
            let g = min(min(1.0, j as f64 / attack), (len - j) as f64 / fade);
            if g < 1.0 {
                *o = (*o as f64 * g) as f32;
            }
        }
        out
    }
}

/// A saw at `phase` (0..1) without false tones above the rate (PolyBLEP: the jump rounded off over a sample
/// either side), `dt = hz/rate`.
#[inline]
pub fn saw(phase: f64, dt: f64) -> f64 {
    let blep = if phase < dt {
        let x = phase / dt;
        2.0 * x - x * x - 1.0
    } else if phase > 1.0 - dt {
        let x = (phase - 1.0) / dt;
        x * x + 2.0 * x + 1.0
    } else {
        0.0
    };
    2.0 * phase - 1.0 - blep
}
