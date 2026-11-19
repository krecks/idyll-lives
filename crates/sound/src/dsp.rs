//! The building blocks every sound is made from (most were `web/src/audio/water.ts`'s and `mono.ts`'s): a
//! seeded generator, pink noise, a two-pole filter as the browser's node computes it, slow wandering, bubbles,
//! clicks, clinks, thuds and sloshes, a small room, and the loop's seam folded and its level set; and, since,
//! tones without a `sin` or an `exp` a sample: a sine by rotation (`Osc`), overtones from one `sin`
//! (`harmonics`), a recursive decay (`Decay`), a bank of decaying modes (`Modal`), a plucked string (`Pluck`)
//! and a saw without false tones (`saw`).
//!
//! The arithmetic is JavaScript's as the sounds were written in it: f64 throughout, every buffer of samples
//! f32 (a `Float32Array`: what is read back from one is the value rounded to f32), `Math.round` rounding
//! halves up (`js_round`), and `sin`, `exp`, `pow`… from `libm` (fdlibm, as V8's `Math`), so a sound is the
//! same here and in the browser's wasm, and what the TypeScript made: to rounding where a tone is now turned
//! by rotation (a clink, something struck, a motor). The music, the knock and the knife's chop have moved on
//! from it.

/// `Math`'s functions, as fdlibm computes them (the same in a native build and in wasm).
pub mod m {
    pub use core::f64::consts::{FRAC_1_SQRT_2, PI};
    pub fn sin(x: f64) -> f64 {
        libm::sin(x)
    }
    pub fn cos(x: f64) -> f64 {
        libm::cos(x)
    }
    pub fn tan(x: f64) -> f64 {
        libm::tan(x)
    }
    pub fn exp(x: f64) -> f64 {
        libm::exp(x)
    }
    pub fn log(x: f64) -> f64 {
        libm::log(x)
    }
    pub fn log2(x: f64) -> f64 {
        libm::log2(x)
    }
    pub fn log10(x: f64) -> f64 {
        libm::log10(x)
    }
    pub fn tanh(x: f64) -> f64 {
        libm::tanh(x)
    }
    pub fn atan(x: f64) -> f64 {
        libm::atan(x)
    }
    pub fn atan2(y: f64, x: f64) -> f64 {
        libm::atan2(y, x)
    }
    /// `x ** y` and `Math.pow`.
    pub fn pow(x: f64, y: f64) -> f64 {
        libm::pow(x, y)
    }
    pub fn hypot(x: f64, y: f64) -> f64 {
        libm::hypot(x, y)
    }
    /// `Math.sqrt` (correctly rounded everywhere).
    pub fn sqrt(x: f64) -> f64 {
        x.sqrt()
    }
    /// `Math.sign` (0 and NaN as they are).
    pub fn sign(x: f64) -> f64 {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            x
        }
    }
    /// `Math.round`: the nearest integer, halves up (-2.5 is -2).
    pub fn js_round(x: f64) -> f64 {
        let f = x.floor();
        if x - f >= 0.5 { f + 1.0 } else { f }
    }
    /// `Math.round` as an index or a length (one that comes out negative is 0).
    pub fn round_usize(x: f64) -> usize {
        js_round(x).max(0.0) as usize
    }
    /// `Math.min(a, b)` / `Math.max(a, b)` (no NaN here).
    pub fn min(a: f64, b: f64) -> f64 {
        if a < b { a } else { b }
    }
    pub fn max(a: f64, b: f64) -> f64 {
        if a > b { a } else { b }
    }
    /// JavaScript's `ToUint32` (`x >>> 0`).
    pub fn to_u32(x: f64) -> u32 {
        if !x.is_finite() {
            return 0;
        }
        x.trunc().rem_euclid(4_294_967_296.0) as u32
    }
    /// JavaScript's `ToInt32` (`x | 0`).
    pub fn to_i32(x: f64) -> i32 {
        to_u32(x) as i32
    }
}

use m::*;

/// A small seeded generator, so the sounds are the same every time (xorshift32: `random(seed)` in the
/// TypeScript, `seed >>> 0 || 1`): numbers in 0..1.
#[derive(Clone, Debug)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: f64) -> Rng {
        let s = to_u32(seed);
        Rng(if s == 0 { 1 } else { s })
    }

    /// The next number, 0..1 (`r()`).
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f64 {
        let mut s = self.0;
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        self.0 = s;
        s as f64 / 4_294_967_296.0
    }
}

/// Two channels of samples.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stereo {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

impl Stereo {
    /// A channel by number (0 left, 1 right).
    pub fn ch(&mut self, c: usize) -> &mut Vec<f32> {
        if c == 0 {
            &mut self.left
        } else {
            &mut self.right
        }
    }
}

/// `d[i] += v` on a `Float32Array`.
#[inline]
pub fn add(d: &mut [f32], i: usize, v: f64) {
    d[i] = (d[i] as f64 + v) as f32;
}

/// A loop being made: its two channels, its rate, its seconds (with the overlap) and the seconds after the
/// loop made to fade into its start.
#[derive(Clone, Debug)]
pub struct Track {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub rate: f64,
    /// Seconds in the loop and the overlap after it.
    pub seconds: f64,
    pub overlap: f64,
}

impl Track {
    pub fn ch(&mut self, c: usize) -> &mut Vec<f32> {
        if c == 0 {
            &mut self.left
        } else {
            &mut self.right
        }
    }
}

/// Room for a loop of `seconds`, plus a little more to cross-fade into its start (none for a sound played
/// once: `overlap` 0; the TypeScript's default is 0.6).
pub fn track(rate: f64, seconds: f64, overlap: f64) -> Track {
    let n = round_usize((seconds + overlap) * rate);
    Track {
        left: vec![0.0; n],
        right: vec![0.0; n],
        rate,
        seconds: seconds + overlap,
        overlap,
    }
}

/// What a stream is like (`stream`).
#[derive(Clone, Copy, Debug)]
pub struct StreamOpts {
    pub centre: f64,
    pub q: f64,
    pub bubbles: f64,
    pub gurgle: f64,
    pub low: f64,
    pub high: f64,
    pub rumble: f64,
}

/// Water running into water: the jet's rush (pink noise in a band, its centre and level burbling) with a
/// faint fizz above, bubbles in a range of sizes, and a low rumble in the water below.
pub fn stream(out: &mut Track, r: &mut Rng, o: StreamOpts) {
    let rate = out.rate;
    let n = out.left.len();
    let burble = wander(r, n, rate, 0.035);
    let slow = wander(r, n, rate, 0.7);
    let mut shared = Pink::new();
    for c in 0..2 {
        let mut own = Pink::new();
        let mut jet = Biquad::new(rate);
        let mut fizz = Biquad::new(rate);
        fizz.set(Filter::Bandpass, 4200.0, 0.8);
        let mut rumble = Biquad::new(rate);
        rumble.set(Filter::Lowpass, 260.0, 0.8);
        let mut deep = Pink::new();
        let seconds = out.seconds;
        let ch = out.ch(c);
        for i in 0..n {
            let g = burble[i] as f64 * 0.7 + slow[i] as f64 * 0.3;
            if i & 31 == 0 {
                jet.set(Filter::Bandpass, o.centre * pow(2.0, 0.35 * g), o.q);
            }
            let w = 0.6 * shared.next(r) + 0.75 * own.next(r);
            ch[i] = (jet.run(w) * (1.0 + 0.3 * g) * 1.3
                + fizz.run(w) * 0.12
                + rumble.run(deep.next(r)) * o.rumble) as f32;
        }
        let pan = if c == 1 { 1.0 } else { -1.0 };
        let mut t = r.next() * 0.01;
        while t < seconds {
            let i = (n - 1).min(round_usize(t * rate));
            let f = o.low * pow(o.high / o.low, pow(r.next(), 0.7));
            let a = 0.15 + pow(r.next(), 2.0);
            let b = max(0.2, 1.0 + 0.6 * burble[i] as f64);
            let amp = o.gurgle * a * b * (1.0 + 0.2 * pan * (r.next() - 0.5));
            let rise = 0.05 + r.next() * 0.25;
            bubble(ch, rate, t, f, amp, rise);
            t += -log(1.0 - r.next()) / o.bubbles;
        }
    }
}

/// One air bubble ringing: a sine at `f0` (Hz) dying away quickly, its pitch rising by `rise` as it goes.
pub fn bubble(d: &mut [f32], rate: f64, at: f64, f0: f64, amp: f64, rise: f64) {
    let damp = 0.043 * f0 + 0.0014 * pow(f0, 1.5);
    let start = js_round(at * rate);
    let len = min(d.len() as f64 - start, js_round((4.5 / damp) * rate));
    if len <= 0.0 {
        return;
    }
    let (start, len) = (start as usize, len as usize);
    let k = exp(-damp / rate);
    let mut env = amp;
    let (mut re, mut im) = (1.0f64, 0.0f64);
    let mut j0 = 0;
    while j0 < len {
        let w = (2.0 * PI * f0 * (1.0 + (rise * damp * (j0 as f64 + 16.0)) / rate)) / rate;
        let (c, s) = (cos(w), sin(w));
        for j in j0..len.min(j0 + 32) {
            let x = re * c - im * s;
            im = re * s + im * c;
            re = x;
            add(d, start + j, env * im);
            env *= k;
        }
        j0 += 32;
    }
}

/// A very short click.
pub fn tick(d: &mut [f32], rate: f64, at: f64, amp: f64) {
    let start = js_round(at * rate);
    let len = min(d.len() as f64 - start, js_round(0.0015 * rate));
    if len <= 0.0 {
        return;
    }
    let mut s = to_u32(start * 2_654_435_761.0);
    let n = len as usize;
    for j in 0..n {
        s = (s as u64 * 1_664_525 + 1_013_904_223) as u32;
        add(
            d,
            start as usize + j,
            amp * (s as f64 / 2_147_483_648.0 - 1.0) * (1.0 - j as f64 / len),
        );
    }
}

/// Water lapping or swished about: a swell of low, filtered noise with a few big bubbles in it.
#[allow(clippy::too_many_arguments)]
pub fn slosh(
    out: &mut Track,
    rate: f64,
    r: &mut Rng,
    at: f64,
    seconds: f64,
    amp: f64,
    pan: f64,
    cutoff: f64,
) {
    let start = js_round(at * rate);
    let len = min(
        out.left.len() as f64 - start,
        js_round(seconds * 1.6 * rate),
    );
    if len <= 0.0 {
        return;
    }
    let (start, len) = (start as usize, len as usize);
    let (l, rr) = gains(pan);
    let mut lp = Biquad::new(rate);
    lp.set(Filter::Lowpass, cutoff, 0.9);
    let mut hp = Biquad::new(rate);
    hp.set(Filter::Highpass, 140.0, 0.7);
    let shape = wander(r, len, rate, 0.04);
    for (j, &wobble) in shape.iter().enumerate() {
        let t = j as f64 / rate / seconds;
        let env = if t < 0.3 {
            pow(sin(((t / 0.3) * PI) / 2.0), 2.0)
        } else {
            exp(-(t - 0.3) * 3.2)
        };
        let v =
            hp.run(lp.run(r.next() * 2.0 - 1.0)) * amp * 1.4 * env * (1.0 + 0.5 * wobble as f64);
        add(&mut out.left, start + j, v * l);
        add(&mut out.right, start + j, v * rr);
    }
    let count = js_round(seconds * (12.0 + r.next() * 20.0));
    let mut k = 0.0;
    while k < count {
        let t = at + seconds * 0.6 * pow(r.next(), 1.5);
        let f = 220.0 + 700.0 * pow(r.next(), 1.5);
        let a = 0.05 * amp * (0.3 + r.next());
        let rise_l = 0.1 + r.next() * 0.3;
        bubble(&mut out.left, rate, t, f, a * l, rise_l);
        let rise_r = 0.1 + r.next() * 0.3;
        bubble(&mut out.right, rate, t, f, a * rr, rise_r);
        k += 1.0;
    }
}

/// A dish or a fork knocking another: a few inharmonic partials ringing out after a click; under water,
/// duller and shorter.
#[allow(clippy::too_many_arguments)]
pub fn clink(
    out: &mut Track,
    rate: f64,
    at: f64,
    f0: f64,
    amp: f64,
    pan: f64,
    ratios: &[f64],
    ring: f64,
    under: bool,
) {
    let start = js_round(at * rate);
    let (l, rr) = gains(pan);
    let mut s = to_u32(start * 40503.0 + 7.0);
    let mut rnd = || {
        s = (s as u64 * 1_664_525 + 1_013_904_223) as u32;
        s as f64 / 4_294_967_296.0
    };
    let mut partials = Vec::new();
    for (i, k) in ratios.iter().enumerate() {
        let i = i as f64;
        let f = f0 * k * (0.97 + 0.06 * rnd());
        let a = (if under { 0.6 } else { 1.0 }) / (1.0 + i * if under { 3.0 } else { 0.9 });
        let tau = (ring * if under { 0.35 } else { 1.0 }) / (1.0 + i * 0.8);
        // (6.28 as the TypeScript has it, not 2π: the phases are what they were.)
        #[allow(clippy::approx_constant)]
        let phase = rnd() * 6.28;
        if f < if under { 2500.0 } else { 16000.0 } {
            partials.push(Mode {
                hz: f,
                tau,
                level: a,
                phase,
            });
        }
    }
    let len = min(out.left.len() as f64 - start, js_round(ring * 6.0 * rate));
    if len <= 0.0 {
        return;
    }
    let (start, len) = (start as usize, len as usize);
    let mut bank = Modal::new(rate, &partials);
    for j in 0..len {
        let mut v = bank.next();
        if (j as f64) < 0.002 * rate && !under {
            v += (rnd() * 2.0 - 1.0) * 0.6 * (1.0 - j as f64 / (0.002 * rate));
        }
        v *= amp * min(1.0, j as f64 / (0.0004 * rate));
        add(&mut out.left, start + j, v * l);
        add(&mut out.right, start + j, v * rr);
    }
}

/// A soft, low thud (wet washing landing in the drum).
pub fn thud(out: &mut Track, rate: f64, at: f64, amp: f64) {
    let start = js_round(at * rate);
    let len = min(out.left.len() as f64 - start, js_round(0.25 * rate));
    if len <= 0.0 {
        return;
    }
    let (start, len) = (start as usize, len as usize);
    let mut phase = 0.0;
    for j in 0..len {
        let t = j as f64 / rate;
        phase += (2.0 * PI * (70.0 - 25.0 * min(1.0, t / 0.1))) / rate;
        let v = amp * sin(phase) * exp(-t / 0.06) * min(1.0, t / 0.006);
        add(&mut out.left, start + j, v);
        add(&mut out.right, start + j, v);
    }
}

/// A dishwasher's rack rolled on its rails: a low rumble with the wheels ticking over the joins, then the
/// knock as it stops.
pub fn roll(out: &mut Track, rate: f64, r: &mut Rng, at: f64, seconds: f64, pan: f64) {
    let start = js_round(at * rate);
    let len = min(out.left.len() as f64 - start, js_round(seconds * rate));
    if len <= 0.0 {
        return;
    }
    let (start_i, n) = (start as usize, len as usize);
    let (l, rr) = gains(pan);
    let mut band = Biquad::new(rate);
    band.set(Filter::Bandpass, 420.0, 1.2);
    for j in 0..n {
        let v = band.run(r.next() * 2.0 - 1.0) * 0.5 * pow(sin((PI * j as f64) / len), 0.5);
        add(&mut out.left, start_i + j, v * l);
        add(&mut out.right, start_i + j, v * rr);
    }
    let mut t = at + 0.02;
    while t < at + seconds {
        tick(&mut out.left, rate, t, 0.08 * l);
        tick(&mut out.right, rate, t, 0.08 * rr);
        t += 0.03 + r.next() * 0.03;
    }
    let f = 900.0 + 300.0 * r.next();
    clink(
        out,
        rate,
        at + seconds,
        f,
        0.25,
        pan,
        &[1.0, 2.76, 5.4],
        0.05,
        false,
    );
}

/// 0 before `a`, 1 after `b`, a smooth step between.
pub fn ramp(a: f64, b: f64, x: f64) -> f64 {
    let u = max(0.0, min(1.0, (x - a) / (b - a)));
    u * u * (3.0 - 2.0 * u)
}

/// Left and right levels for `pan` from -1 (left) to 1 (right), keeping the loudness.
pub fn gains(pan: f64) -> (f64, f64) {
    let a = ((max(-1.0, min(1.0, pan)) + 1.0) * PI) / 4.0;
    (cos(a), sin(a))
}

/// A small room: four damped echoes going round (delays in ms) and a diffuser, mixed in at `wet`. Different
/// delays per channel make it wide.
pub fn room(d: &mut [f32], rate: f64, delays: &[f64], feedback: f64, damping: f64, wet: f64) {
    struct Comb {
        buf: Vec<f32>,
        i: usize,
        lp: f64,
    }
    struct Pass {
        buf: Vec<f32>,
        i: usize,
    }
    let mut combs: Vec<Comb> = delays
        .iter()
        .map(|ms| Comb {
            buf: vec![0.0; round_usize((ms / 1000.0) * rate)],
            i: 0,
            lp: 0.0,
        })
        .collect();
    let mut passes: Vec<Pass> = [5.1, 1.7]
        .iter()
        .map(|ms| Pass {
            buf: vec![0.0; round_usize((ms / 1000.0) * rate)],
            i: 0,
        })
        .collect();
    let count = combs.len() as f64;
    for s in d.iter_mut() {
        let x = *s as f64;
        let mut y = 0.0;
        for c in combs.iter_mut() {
            let v = c.buf[c.i] as f64;
            c.lp = v * (1.0 - damping) + c.lp * damping;
            c.buf[c.i] = (x + c.lp * feedback) as f32;
            c.i = (c.i + 1) % c.buf.len();
            y += v;
        }
        y /= count;
        for p in passes.iter_mut() {
            let v = p.buf[p.i] as f64;
            let u = y + v * 0.5;
            p.buf[p.i] = u as f32;
            p.i = (p.i + 1) % p.buf.len();
            y = v - u * 0.5;
        }
        *s = (x * (1.0 - wet * 0.5) + y * wet) as f32;
    }
}

/// Folds the overlap at the end into the start (an equal-power cross-fade, so the loop runs on without a
/// seam or a dip), cuts the overlap off, then sets the level: to an average of `rms`, or with 0 to a peak of
/// `peak` (the TypeScript's default 1); any peaks over 0.7 are rounded off.
pub fn finish(t: Track, rms: f64, peak: f64) -> Stereo {
    set_level(fold(t), rms, peak)
}

/// The loop's overlap folded into its start and cut off (`finish` without the level).
pub fn fold(t: Track) -> Stereo {
    let f = round_usize(t.overlap * t.rate);
    let n = t.left.len() - f;
    let one = |src: &[f32]| -> Vec<f32> {
        let mut dst = src[..n].to_vec();
        for i in 0..f {
            let x = i as f64 / f as f64;
            dst[i] = (src[i] as f64 * sin((x * PI) / 2.0) + src[n + i] as f64 * cos((x * PI) / 2.0))
                as f32;
        }
        dst
    };
    Stereo {
        left: one(&t.left),
        right: one(&t.right),
    }
}

/// Sets the level, as `finish`.
pub fn set_level(mut result: Stereo, rms: f64, peak: f64) -> Stereo {
    let n = result.left.len();
    let mut sum = 0.0;
    let mut top: f64 = 0.0;
    for d in [&result.left, &result.right] {
        for &v in d.iter().take(n) {
            let v = v as f64;
            sum += v * v;
            top = max(top, v.abs());
        }
    }
    let or1 = |x: f64| if x == 0.0 || x.is_nan() { 1.0 } else { x };
    let scale = if rms > 0.0 {
        rms / or1(sqrt(sum / (2.0 * n as f64)))
    } else {
        peak / or1(top)
    };
    for d in [&mut result.left, &mut result.right] {
        for v in d.iter_mut().take(n) {
            let x = *v as f64 * scale;
            let a = x.abs();
            *v = (if a <= 0.7 {
                x
            } else {
                sign(x) * (0.7 + 0.3 * tanh((a - 0.7) / 0.3))
            }) as f32;
        }
    }
    result
}

/// Slow random wandering around 0 (about ±1), changing over roughly `tau` seconds.
pub fn wander(r: &mut Rng, n: usize, rate: f64, tau: f64) -> Vec<f32> {
    let mut out = vec![0.0f32; n];
    let k = 1.0 / (tau * rate);
    let step = sqrt(2.0 * k) * sqrt(3.0);
    let a = min(1.0, 4.0 * k);
    let (mut x, mut y) = (0.0, 0.0);
    for o in out.iter_mut() {
        x += -x * k + step * (r.next() * 2.0 - 1.0);
        y += (x - y) * a;
        *o = y as f32;
    }
    out
}

/// Pink noise (equal energy per octave, like most of nature's noises), about ±1. It draws from the
/// generator it is given each time (the TypeScript's `new Pink(r)` kept `r`).
#[derive(Clone, Debug, Default)]
pub struct Pink {
    b0: f64,
    b1: f64,
    b2: f64,
}

impl Pink {
    pub fn new() -> Pink {
        Pink::default()
    }

    pub fn next(&mut self, r: &mut Rng) -> f64 {
        let w = r.next() * 2.0 - 1.0;
        self.b0 = 0.99765 * self.b0 + w * 0.099046;
        self.b1 = 0.963 * self.b1 + w * 0.2965164;
        self.b2 = 0.57 * self.b2 + w * 1.0526913;
        (self.b0 + self.b1 + self.b2 + w * 0.1848) * 0.25
    }
}

/// A filter's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    Lowpass,
    Highpass,
    Bandpass,
}

/// A two-pole filter (the Web Audio `BiquadFilterNode`'s formulas, its frequency kept under 0.45 of the
/// rate), whose settings can change as it runs.
#[derive(Clone, Debug)]
pub struct Biquad {
    rate: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl Biquad {
    pub fn new(rate: f64) -> Biquad {
        Biquad {
            rate,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// A new filter already set (`new Biquad(rate).set(type, frequency, q)`).
    pub fn with(rate: f64, kind: Filter, frequency: f64, q: f64) -> Biquad {
        let mut b = Biquad::new(rate);
        b.set(kind, frequency, q);
        b
    }

    pub fn set(&mut self, kind: Filter, frequency: f64, q: f64) -> &mut Self {
        let w = (2.0 * PI * min(frequency, self.rate * 0.45)) / self.rate;
        let c = cos(w);
        let alpha = sin(w) / (2.0 * q);
        let a0 = 1.0 + alpha;
        if kind == Filter::Bandpass {
            self.b0 = alpha / a0;
            self.b1 = 0.0;
            self.b2 = -alpha / a0;
        } else {
            let k = if kind == Filter::Lowpass {
                1.0 - c
            } else {
                1.0 + c
            };
            self.b0 = k / 2.0 / a0;
            self.b2 = self.b0;
            self.b1 = (if kind == Filter::Lowpass { k } else { -k }) / a0;
        }
        self.a1 = (-2.0 * c) / a0;
        self.a2 = (1.0 - alpha) / a0;
        self
    }

    pub fn run(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

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
        let s = self.dull;
        // The loss filter's delay and gain at the note.
        let d_lp = atan2(s * sin(w), (1.0 - s) + s * cos(w)) / w;
        let gain = sqrt((1.0 - s) * (1.0 - s) + s * s + 2.0 * s * (1.0 - s) * cos(w));
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
                last = (1.0 - self.soft) * (r.next() * 2.0 - 1.0) + self.soft * last;
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
            let (mut u1, mut v1, mut w1) = (0.0, 0.0, 0.0);
            let mut i = 0;
            for o in out.iter_mut() {
                let y = line[i];
                *o = y as f32;
                let v = rho * ((1.0 - s) * y + s * u1);
                let x = ap * v + v1 - ap * w1;
                u1 = y;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_generator_is_the_typescripts() {
        // random(11)'s first numbers, as the TypeScript gives them.
        let mut r = Rng::new(11.0);
        let first: Vec<f64> = (0..3).map(|_| r.next()).collect();
        assert_eq!(
            first,
            [
                0.0006924520712345839,
                0.1720386769156903,
                0.07115283189341426
            ]
        );
        assert_eq!(Rng::new(0.0).next(), Rng::new(1.0).next());
        assert_eq!(Rng::new(-1.0).0, u32::MAX);
    }

    #[test]
    fn math_round_rounds_halves_up() {
        assert_eq!(js_round(2.5), 3.0);
        assert_eq!(js_round(-2.5), -2.0);
        assert_eq!(js_round(0.49999999999999994), 0.0);
        assert_eq!(js_round(-0.6), -1.0);
    }

    #[test]
    fn a_sine_by_rotation_stays_a_sine_for_ten_seconds() {
        let rate = 48000.0;
        for (hz, phase) in [(440.0, 0.0), (61.7, 1.3), (5123.4, 4.0)] {
            let mut o = Osc::new(hz, rate, phase);
            let w = (2.0 * PI * hz) / rate;
            for n in 0..(10.0 * rate) as usize {
                let want = sin(w * n as f64 + phase);
                let got = o.next();
                assert!(
                    (got - want).abs() < 1e-9,
                    "{hz} Hz, sample {n}: {got} {want}"
                );
            }
        }
    }

    #[test]
    fn a_decay_is_an_exponential() {
        let (rate, tau, level) = (48000.0, 0.13, 0.7);
        let mut d = Decay::new(level, tau, rate);
        for n in 0..round_usize(5.0 * tau * rate) {
            let want = level * exp(-(n as f64) / (tau * rate));
            assert!((d.next() - want).abs() < 1e-9 * level, "{n}");
        }
    }

    #[test]
    fn a_modal_bank_is_its_modes_dying_away() {
        let rate = 48000.0;
        let modes = [
            Mode {
                hz: 98.0,
                tau: 0.04,
                level: 1.0,
                phase: 0.0,
            },
            Mode {
                hz: 1234.5,
                tau: 0.02,
                level: 0.6,
                phase: 2.1,
            },
            Mode {
                hz: 7000.0,
                tau: 0.008,
                level: 0.3,
                phase: 5.9,
            },
            // Over 0.45 of the rate: held there.
            Mode {
                hz: 30000.0,
                tau: 0.01,
                level: 0.2,
                phase: 1.0,
            },
        ];
        let mut bank = Modal::new(rate, &modes);
        for n in 0..round_usize(7.0 * 0.04 * rate) {
            let t = n as f64 / rate;
            let want: f64 = modes
                .iter()
                .map(|m| {
                    let w = (2.0 * PI * min(m.hz, 0.45 * rate)) / rate;
                    m.level * exp(-t / m.tau) * sin(w * n as f64 + m.phase)
                })
                .sum();
            let got = bank.next();
            assert!((got - want).abs() < 1e-9, "{n}: {got} {want}");
        }
    }

    #[test]
    fn harmonics_are_the_sum_of_their_sines() {
        let weights: Vec<f64> = (1..=40).map(|h| 1.0 / h as f64).collect();
        for turns in [0.0, 0.013, 0.25, 0.4999, 0.77, 3.3, 1234.567] {
            let want: f64 = weights
                .iter()
                .enumerate()
                .map(|(k, w)| w * sin(2.0 * PI * (k + 1) as f64 * turns))
                .sum();
            let got = harmonics(turns, &weights);
            assert!((got - want).abs() < 1e-9, "{turns}: {got} {want}");
        }
        assert_eq!(harmonics(0.3, &[]), 0.0);
    }

    #[test]
    fn a_plucked_string_is_in_tune() {
        let rate = 48000.0;
        // The pitch where the note is strongest, of 61 a cent apart round where it should be.
        let found = |x: &[f32], hz: f64| -> f64 {
            let n = rate as usize;
            let mut best = (0.0, f64::NEG_INFINITY);
            for cents in -30..=30 {
                let f = hz * pow(2.0, cents as f64 / 1200.0);
                let w = (2.0 * PI * f) / rate;
                let (mut re, mut im) = (0.0, 0.0);
                for (i, &v) in x[..n].iter().enumerate() {
                    let h = 0.5 - 0.5 * cos((2.0 * PI * i as f64) / (n - 1) as f64);
                    re += v as f64 * h * cos(w * i as f64);
                    im -= v as f64 * h * sin(w * i as f64);
                }
                let size = re * re + im * im;
                if size > best.1 {
                    best = (cents as f64, size);
                }
            }
            best.0
        };
        for (hz, within) in [(110.0, 10.0), (440.0, 5.0), (1760.0, 5.0), (1975.0, 10.0)] {
            for dull in [0.3, 0.5] {
                let mut r = Rng::new(7.0);
                let note = Pluck {
                    hz,
                    tau: 2.0,
                    dull,
                    soft: 0.4,
                    place: 0.18,
                }
                .play(rate, 1.2, &mut r);
                let off = found(&note, hz);
                assert!(off.abs() <= within, "{hz} Hz, dull {dull}: {off} cents off");
            }
        }
        // Too high for the rate: a sine, still a note.
        let mut r = Rng::new(7.0);
        let high = Pluck {
            hz: 3000.0,
            tau: 0.3,
            dull: 0.3,
            soft: 0.4,
            place: 0.18,
        }
        .play(8000.0, 0.5, &mut r);
        assert!(high.iter().any(|&v| v.abs() > 0.2));
        assert!(high.iter().all(|v| v.is_finite() && v.abs() <= 1.0));
    }

    #[test]
    fn a_saw_without_false_tones_averages_nothing_and_stays_in_bounds() {
        let rate = 48000.0;
        for hz in [55.0, 440.0, 1003.0] {
            let dt = hz / rate;
            let periods = 20.0;
            let n = round_usize(periods * rate / hz);
            let mut phase = 0.0;
            let mut sum = 0.0;
            for _ in 0..n {
                let v = saw(phase, dt);
                assert!(v.abs() <= 1.05, "{hz}: {v}");
                sum += v;
                phase += dt;
                if phase >= 1.0 {
                    phase -= 1.0;
                }
            }
            assert!(
                (sum / n as f64).abs() < 2e-3,
                "{hz}: mean {}",
                sum / n as f64
            );
        }
    }
}
