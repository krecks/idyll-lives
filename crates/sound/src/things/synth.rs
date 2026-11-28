//! The building blocks the things are made with: loops and one-shots, strikes, motors, noise, and putting a stereo sound together.

use super::*;

// Building blocks.

/// The average level of the loops of separate knocks (darts, a rally): set by their average rather than
/// their peak, as their knocks are short; the sharpest are rounded off a little (`set_level`).
pub(super) const SPARSE: f64 = 0.046;

/// A mode: its frequency (Hz), the time it takes to die to a third (s) and its level.
type Mode = [f64; 3];

/// A short burst of noise in a band (its centre in Hz, its Q), dying away over `tau` seconds, at `amp`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Burst {
    hz: f64,
    q: f64,
    tau: f64,
    amp: f64,
}

/// A burst (`{ hz, q, tau, amp }`).
pub(super) fn burst(hz: f64, q: f64, tau: f64, amp: f64) -> Burst {
    Burst { hz, q, tau, amp }
}

/// A track of events: its two channels, its rate, and whether its events wrap round its end into its start
/// (a loop of events, `looped`) or are cut off there (a sound played once, `once`). No overlap.
pub(super) struct Ev {
    pub(super) left: Vec<f32>,
    pub(super) right: Vec<f32>,
    pub(super) rate: f64,
    pub(super) wraps: bool,
}

impl From<Ev> for Stereo {
    fn from(e: Ev) -> Stereo {
        Stereo {
            left: e.left,
            right: e.right,
        }
    }
}

/// A track whose events wrap round its end into its start (a loop of events, no overlap; the TypeScript's
/// `loop`).
pub(super) fn looped(rate: f64, seconds: f64) -> Ev {
    let t = track(rate, seconds, 0.0);
    Ev {
        left: t.left,
        right: t.right,
        rate,
        wraps: true,
    }
}

/// A track for a sound played once: nothing wraps.
pub(super) fn once(rate: f64, seconds: f64) -> Ev {
    let t = track(rate, seconds, 0.0);
    Ev {
        left: t.left,
        right: t.right,
        rate,
        wraps: false,
    }
}

/// Adds `v` at sample `i`, left and right at `l` and `r`; past the end or before the start it wraps on a
/// loop of events (an event set a hair before the loop's start rings on from its end, so the seam is whole),
/// else it's dropped.
pub(super) fn put(t: &mut Ev, i: f64, v: f64, l: f64, r: f64) {
    let n = t.left.len();
    if !t.wraps && (i < 0.0 || i >= n as f64) {
        return;
    }
    let i = (i as i64).rem_euclid(n as i64) as usize;
    add(&mut t.left, i, v * l);
    add(&mut t.right, i, v * r);
}

/// Something struck: its modes ringing out over a burst of noise, from `at` (s), at `amp`, placed at `pan`.
pub(super) fn strike(t: &mut Ev, at: f64, amp: f64, pan: f64, modes: &[Mode], burst: Burst) {
    let rate = t.rate;
    let start = js_round(at * rate);
    let longest = modes.iter().fold(burst.tau, |a, m| max(a, m[1]));
    let len = js_round(longest * 7.0 * rate);
    let (l, r) = gains(pan);
    let mut band = Biquad::with(rate, Filter::Bandpass, burst.hz, burst.q);
    let mut s = to_u32(start * 2_654_435_761.0 + 977.0);
    let mut rnd = || {
        s = (s as u64 * 1_664_525 + 1_013_904_223) as u32;
        s as f64 / 4_294_967_296.0
    };
    let ringing: Vec<crate::dsp::Mode> = modes
        .iter()
        .map(|m| crate::dsp::Mode {
            hz: m[0],
            tau: m[1],
            level: m[2],
            phase: rnd() * 2.0 * PI,
        })
        .collect();
    let mut bank = Modal::new(rate, &ringing);
    let mut noise_level = Decay::new(burst.amp, burst.tau, rate);
    let attack = 0.0004 * rate;
    let mut j = 0.0;
    while j < len {
        let mut v = bank.next();
        v += band.run(rnd() * 2.0 - 1.0) * noise_level.next();
        put(t, start + j, v * amp * min(1.0, j / attack), l, r);
        j += 1.0;
    }
}

/// A rustle or a rub: noise in a band (`hz`, `q`; the TypeScript's defaults 4000 and 0.5), swelling and
/// fading over `seconds`.
#[allow(clippy::too_many_arguments)]
pub(super) fn rustle(
    t: &mut Ev,
    r: &mut Rng,
    at: f64,
    seconds: f64,
    amp: f64,
    pan: f64,
    hz: f64,
    q: f64,
) {
    let rate = t.rate;
    let start = js_round(at * rate);
    let len = js_round(seconds * rate);
    let (l, rr) = gains(pan);
    let mut bp = Biquad::with(rate, Filter::Bandpass, hz, q);
    let mut j = 0.0;
    while j < len {
        let x = bp.run(r.next() * 2.0 - 1.0) * amp;
        let v = x * (0.6 + 0.4 * r.next()) * sin((PI * j) / len);
        put(t, start + j, v, l, rr);
        j += 1.0;
    }
}

/// A rush of air (a door swung): low noise swelling and fading.
pub(super) fn swish(t: &mut Ev, r: &mut Rng, at: f64, seconds: f64, amp: f64, hz: f64) {
    let rate = t.rate;
    let start = js_round(at * rate);
    let len = js_round(seconds * rate);
    let mut lp = Biquad::with(rate, Filter::Lowpass, hz, 0.7);
    let mut j = 0.0;
    while j < len {
        let v = lp.run(r.next() * 2.0 - 1.0) * amp * pow(sin((PI * j) / len), 2.0);
        put(t, start + j, v, 0.7, 0.7);
        j += 1.0;
    }
}

/// A machine's steady sound, the same on both sides: a loop of `seconds` and its overlap (0.6 s), its layers
/// summed as each is made (`add`), so the layers are not all kept until the end.
pub(super) struct Bed {
    sum: Vec<f64>,
    rate: f64,
    seconds: f64,
}

impl Bed {
    pub(super) fn new(rate: f64, seconds: f64) -> Bed {
        Bed {
            sum: vec![0.0; round_usize((seconds + 0.6) * rate)],
            rate,
            seconds,
        }
    }

    /// Its samples, the overlap's with them.
    pub(super) fn len(&self) -> usize {
        self.sum.len()
    }

    /// A layer added: `v(i, x)` for its sample `x` at `i`.
    pub(super) fn add(&mut self, layer: Vec<f32>, mut v: impl FnMut(usize, f64) -> f64) {
        for (i, (s, &x)) in self.sum.iter_mut().zip(&layer).enumerate() {
            *s += v(i, x as f64);
        }
    }

    /// The loop being made, both sides (to add to each, then `fold`).
    pub(super) fn track(self) -> Track {
        let mut t = track(self.rate, self.seconds, 0.6);
        for (i, &v) in self.sum.iter().enumerate() {
            t.left[i] = v as f32;
            t.right[i] = v as f32;
        }
        t
    }

    /// The loop, its overlap folded into its start.
    pub(super) fn fold(self) -> Stereo {
        fold(self.track())
    }
}

/// A motor's (or a fan's) tone: harmonics at `partials` levels (the fundamental first) of a speed `hz(i)`
/// (Hz at sample `i`).
pub(super) fn motor(n: usize, rate: f64, hz: impl Fn(usize) -> f64, partials: &[f64]) -> Vec<f32> {
    let mut d = vec![0.0f32; n];
    let mut phase = 0.0;
    for (i, o) in d.iter_mut().enumerate() {
        let f = hz(i);
        phase += (2.0 * PI * f) / rate;
        if phase > 2.0 * PI {
            phase -= 2.0 * PI;
        }
        // The harmonics under 0.45 of the rate (`k·f < 0.45·rate`), turned from the first.
        let mut top = 0;
        while top < partials.len() && (top + 1) as f64 * f < rate * 0.45 {
            top += 1;
        }
        *o = harmonics(phase / (2.0 * PI), &partials[..top]) as f32;
    }
    d
}

/// Pink noise through a filter: a rush, a hiss or a rumble.
pub(super) fn noise(n: usize, rate: f64, r: &mut Rng, kind: Filter, hz: f64, q: f64) -> Vec<f32> {
    let mut p = Pink::new();
    let mut f = Biquad::with(rate, kind, hz, q);
    (0..n).map(|_| f.run(p.next(r)) as f32).collect()
}

/// Puts the parts together: the steady ones (already folded into their start: `fold`) and the events
/// added; then a room (`delays` in ms, none for out of doors, mixed in at `wet`), the highs and lows taken
/// off (`low`, `high`: a box or a small speaker; 0 for none), and the level set to an average of `rms`, or
/// with 0 to a peak of `peak`. A loop's filters and room are already ringing at its start, as they were at
/// its end (`done_once` for a sound played once, which starts from silence).
#[allow(clippy::too_many_arguments)]
pub(super) fn done(
    rate: f64,
    parts: Vec<Stereo>,
    rms: f64,
    peak: f64,
    delays: &[f64],
    wet: f64,
    low: f64,
    high: f64,
) -> Stereo {
    put_together(rate, parts, rms, peak, delays, wet, low, high, true)
}

/// `done` for a sound played once: its filters and room start from silence.
#[allow(clippy::too_many_arguments)]
pub(super) fn done_once(
    rate: f64,
    parts: Vec<Stereo>,
    rms: f64,
    peak: f64,
    delays: &[f64],
    wet: f64,
    low: f64,
    high: f64,
) -> Stereo {
    put_together(rate, parts, rms, peak, delays, wet, low, high, false)
}

/// `done`'s work; `lead`: a loop, its filters and room run over a second of its end before its start.
#[allow(clippy::too_many_arguments)]
pub(super) fn put_together(
    rate: f64,
    parts: Vec<Stereo>,
    rms: f64,
    peak: f64,
    delays: &[f64],
    wet: f64,
    low: f64,
    high: f64,
    lead: bool,
) -> Stereo {
    let n = parts.iter().map(|p| p.left.len()).min().unwrap_or(0);
    let mut result = Stereo {
        left: vec![0.0; n],
        right: vec![0.0; n],
    };
    for p in &parts {
        for i in 0..n {
            add(&mut result.left, i, p.left[i] as f64);
            add(&mut result.right, i, p.right[i] as f64);
        }
    }
    // A loop's start follows its end: the filters and the room run over a second of the loop (its end, round
    // and round if it is shorter) before its start, so they are already ringing as they were at its end. A sound
    // played once starts from silence.
    let lead = if lead && n > 0 { round_usize(rate) } else { 0 };
    for c in 0..2 {
        let ch = result.ch(c);
        let mut x: Vec<f32> = (0..lead)
            .map(|k| ch[(k + n - lead % n) % n])
            .chain(ch.iter().copied())
            .collect();
        if low > 0.0 {
            let mut lp = Biquad::with(rate, Filter::Lowpass, low, 0.7);
            for v in x.iter_mut() {
                *v = lp.run(*v as f64) as f32;
            }
        }
        if high > 0.0 {
            let mut hp = Biquad::with(rate, Filter::Highpass, high, 0.7);
            for v in x.iter_mut() {
                *v = hp.run(*v as f64) as f32;
            }
        }
        if !delays.is_empty() {
            let wide: Vec<f64> = delays.iter().map(|d| d * 1.09).collect();
            room(
                &mut x,
                rate,
                if c == 1 { &wide } else { delays },
                0.55,
                0.4,
                wet,
            );
        }
        ch.copy_from_slice(&x[lead..]);
    }
    set_level(result, rms, if peak == 0.0 { 1.0 } else { peak })
}
