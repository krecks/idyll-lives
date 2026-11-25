//! The street and the weather, synthesised (no sound files; was `web/src/audio/outdoors.ts`): a car's engine
//! and its tyres on the road, the town heard from above, and the wind in the leaves and the grass. Each a loop
//! made once, its end running on into its start; `outside.ts` plays them by what the camera sees: a car by its
//! speed and how near it passes, the town by how far out the camera is, the leaves by the gusts the grass shows.
//!
//! - An engine heard from the kerb is a low hum: the firing's tone (55 Hz at the loop's own speed; played
//!   faster as the car goes faster) and its overtones, each wavering a little, with the rasp of the exhaust
//!   under it. Nothing much over 1 kHz gets out from under a bonnet.
//! - Tyres on a road are a broad hiss round 1 kHz that never quite holds still.
//! - A town from above is traffic too far off to tell apart: a rumble, and the swell of a car now and then.
//! - Leaves are thousands of tiny taps: bright noise that flutters, each side its own.

use crate::Made;
use crate::dsp::m::*;
use crate::dsp::*;

/// The average level each is made to (as the water's and the things': a plain gain on it is its level in the
/// game).
const RMS: f64 = 0.2;

/// An outdoor loop by name (`OUTDOORS`): one channel for what has a place (a car), two for what is all round.
pub fn make(kind: &str, rate: f64) -> Option<Made> {
    Some(match kind {
        "carEngine" => Made::Mono(car_engine(rate)),
        "carTyres" => Made::Mono(car_tyres(rate)),
        "town" => Made::Stereo(town(rate)),
        "leaves" => Made::Stereo(leaves(rate)),
        _ => return None,
    })
}

/// One channel of a loop made on both: the left, levelled.
fn mono(s: Stereo) -> Vec<f32> {
    let right = s.left.clone();
    set_level(
        Stereo {
            left: s.left,
            right,
        },
        RMS,
        1.0,
    )
    .left
}

/// A car's engine at a steady pull (4 s: 220 turns of its firing, so its hum meets itself at the seam).
pub fn car_engine(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(0xca51 as f64);
    let seconds = 4.0;
    let firing = 55.0;
    // The exhaust's rasp: low noise (its end run on into its start), chopped at the firing's pace below.
    let mut noise = track(rate, seconds, 0.5);
    let mut rasp = Biquad::with(rate, Filter::Bandpass, 260.0, 0.7);
    let mut pink = Pink::new();
    for v in noise.left.iter_mut() {
        *v = rasp.run(pink.next(&mut r)) as f32;
    }
    let mut d = fold(noise).left;
    // The overtones, each a little weaker than the last, each wavering at its own slow pace (whole turns a loop).
    struct Part {
        k: f64,
        amp: f64,
        wobble: f64,
        depth: f64,
        phase: f64,
    }
    let parts: Vec<Part> = (0..12)
        .map(|k| {
            let amp = (if k % 2 == 1 { 0.7 } else { 1.0 }) / pow((k + 1) as f64, 1.15);
            let wobble = 1.0 + (r.next() * 5.0).floor();
            let depth = 0.1 + 0.2 * r.next();
            let phase = r.next() * 2.0 * PI;
            Part {
                k: (k + 1) as f64,
                amp,
                wobble,
                depth,
                phase,
            }
        })
        .collect();
    for (i, s) in d.iter_mut().enumerate() {
        let t = i as f64 / rate;
        let mut v = 0.0;
        for p in &parts {
            v += p.amp
                * (1.0 + p.depth * sin((2.0 * PI * p.wobble * t) / seconds + p.phase))
                * sin(2.0 * PI * firing * p.k * t + p.phase * p.k);
        }
        let beat = 0.5 + 0.5 * sin(2.0 * PI * firing * t);
        *s = (v + 1.6 * *s as f64 * (0.4 + 0.6 * beat * beat)) as f32;
    }
    // From under the bonnet: the highs stay in (the filter run round the loop, so the seam stays whole).
    let hum = node_filter(&d, rate, Filter::Lowpass, 900.0, -3.01, true);
    let right = hum.clone();
    set_level(Stereo { left: hum, right }, RMS, 1.0).left
}

/// A car's tyres on the road at a steady speed.
pub fn car_tyres(rate: f64) -> Vec<f32> {
    let mut r = Rng::new(0x7e5 as f64);
    let mut out = track(rate, 4.0, 0.5);
    let n = out.left.len();
    let drift = wander(&mut r, n, rate, 0.7);
    let mut pink = Pink::new();
    let mut body = Biquad::with(rate, Filter::Bandpass, 1000.0, 0.5);
    let mut hiss = Biquad::with(rate, Filter::Highpass, 2800.0, 0.6);
    let mut top = Biquad::with(rate, Filter::Lowpass, 6000.0, 0.7);
    for i in 0..n {
        let g = drift[i] as f64;
        if i & 63 == 0 {
            body.set(Filter::Bandpass, 1000.0 * pow(2.0, 0.12 * g), 0.5);
        }
        let w = pink.next(&mut r);
        out.left[i] = ((body.run(w) + 0.25 * top.run(hiss.run(w))) * (1.0 + 0.1 * g)) as f32;
    }
    mono(fold(out))
}

/// The town from above: the rumble of its streets, and a car's swell now and then, each side its own.
pub fn town(rate: f64) -> Stereo {
    let mut r = Rng::new(0x70c4 as f64);
    let mut out = track(rate, 16.0, 1.5);
    let n = out.left.len();
    for c in 0..2 {
        let swell = wander(&mut r, n, rate, 2.5);
        let far = wander(&mut r, n, rate, 6.0);
        let mut pink = Pink::new();
        let mut pass = Pink::new();
        let mut rumble = Biquad::with(rate, Filter::Lowpass, 220.0, 0.6);
        let mut road = Biquad::with(rate, Filter::Bandpass, 520.0, 0.45);
        let mut top = Biquad::with(rate, Filter::Lowpass, 1400.0, 0.7);
        let ch = out.ch(c);
        for i in 0..n {
            let car = pow(max(0.0, swell[i] as f64), 2.0);
            let low = rumble.run(pink.next(&mut r)) * (1.6 + 0.3 * far[i] as f64);
            ch[i] = (low + top.run(road.run(pass.next(&mut r))) * (0.35 + 0.9 * car)) as f32;
        }
    }
    set_level(fold(out), RMS, 1.0)
}

/// The wind in the leaves and the grass, at a full gust: bright noise that flutters, each side its own.
pub fn leaves(rate: f64) -> Stereo {
    let mut r = Rng::new(0x1eaf as f64);
    let mut out = track(rate, 10.0, 1.0);
    let n = out.left.len();
    for c in 0..2 {
        // The leaves' flutter: fast, and the boughs' slower heave under it.
        let flutter = wander(&mut r, n, rate, 0.05);
        let heave = wander(&mut r, n, rate, 0.6);
        let mut pink = Pink::new();
        let mut leaves = Biquad::with(rate, Filter::Bandpass, 5200.0, 0.5);
        let mut grass = Biquad::with(rate, Filter::Bandpass, 1500.0, 0.6);
        let mut air = Biquad::with(rate, Filter::Lowpass, 380.0, 0.6);
        let mut low = Pink::new();
        let ch = out.ch(c);
        for i in 0..n {
            let w = pink.next(&mut r);
            let f = 0.55 + 0.45 * tanh(flutter[i] as f64);
            let h = 1.0 + 0.25 * tanh(heave[i] as f64);
            let near = h * (leaves.run(w) * 2.2 * f + grass.run(w) * 0.5 * (0.6 + 0.4 * f));
            ch[i] = (near + air.run(low.next(&mut r)) * 0.5) as f32;
        }
    }
    set_level(fold(out), RMS, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::check::brightness;

    const RATE: f64 = 48000.0;

    /// How much of `d` is a tone at `hz` (the size of its part at that frequency, per sample).
    fn tone(d: &[f32], hz: f64) -> f64 {
        let (mut re, mut im) = (0.0, 0.0);
        for (i, &v) in d.iter().enumerate() {
            let a = 2.0 * PI * hz * i as f64 / RATE;
            re += v as f64 * cos(a);
            im += v as f64 * sin(a);
        }
        hypot(re, im) / d.len() as f64
    }

    struct Loops {
        engine: Vec<f32>,
        tyres: Vec<f32>,
        town: Stereo,
        leaves: Stereo,
    }

    fn loops() -> &'static Loops {
        static LOOPS: std::sync::OnceLock<Loops> = std::sync::OnceLock::new();
        LOOPS.get_or_init(|| Loops {
            engine: car_engine(RATE),
            tyres: car_tyres(RATE),
            town: town(RATE),
            leaves: leaves(RATE),
        })
    }

    #[test]
    fn the_same_every_time_whole_and_under_full_scale() {
        let l = loops();
        assert_eq!(make("carEngine", RATE), Some(Made::Mono(l.engine.clone())));
        assert_eq!(leaves(RATE).right, l.leaves.right);
        assert!(matches!(make("town", RATE), Some(Made::Stereo(_))));
        assert_eq!(make("rain", RATE), None);
        for d in [
            &l.engine,
            &l.tyres,
            &l.town.left,
            &l.town.right,
            &l.leaves.left,
            &l.leaves.right,
        ] {
            let (mut peak, mut sum) = (0.0f64, 0.0);
            for &v in d.iter() {
                peak = peak.max((v as f64).abs());
                sum += v as f64 * v as f64;
            }
            assert!(sum.is_finite());
            assert!(peak <= 1.0);
            assert!((sum / d.len() as f64).sqrt() > 0.1);
        }
    }

    #[test]
    fn an_engine_hums_on_its_firings_tone_and_its_overtones_a_low_sound() {
        let engine = &loops().engine;
        // 55 Hz and its multiples stand out of what lies between them.
        for k in [1.0, 2.0, 3.0] {
            assert!(tone(engine, 55.0 * k) > 8.0 * tone(engine, 55.0 * k + 27.0));
        }
        assert!(brightness(engine, RATE) < 700.0);
        // Whole turns in the loop: it meets itself at the seam.
        assert!((engine[0] - engine[engine.len() - 1]).abs() < 0.02);
    }

    #[test]
    fn tyres_hiss_a_town_rumbles_leaves_are_bright_what_is_all_round_differs_side_to_side() {
        let l = loops();
        assert!(brightness(&l.tyres, RATE) > 1500.0);
        assert!(brightness(&l.town.left, RATE) < 700.0);
        assert!(brightness(&l.leaves.left, RATE) > 3500.0);
        for s in [&l.town, &l.leaves] {
            let (mut cross, mut own) = (0.0, 0.0);
            for (&a, &b) in s.left.iter().zip(&s.right) {
                cross += a as f64 * b as f64;
                own += a as f64 * a as f64;
            }
            assert!((cross / own).abs() < 0.2);
        }
    }
}
