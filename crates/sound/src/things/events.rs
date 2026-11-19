//! The sounds of things happening, not of things in use (`web/src/audio/events.ts` has the rules that play them):
//! a door or a gate opened and falling shut, curtains drawn along their rail, a car's door shut and its engine
//! started, the post dropping through a letterbox, a shop's till, a drawer pulled out and pushed shut, and a clock
//! ticking (the one loop: a clock keeps time, so it never drifts).
//!
//! All are made from the things' own building blocks (`strike`, `rustle`, `swish`, `motor`, `noise`) and are dry:
//! the game's rooms add their reverb (`web/src/audio/space.ts`).

use super::*;

/// The sounds of things happening, by name.
pub const EVENTS: [&str; 9] = [
    "doorOpen",
    "doorClose",
    "curtain",
    "carDoor",
    "carStart",
    "letterbox",
    "till",
    "drawer",
    "clockTick",
];

/// The ones played once as something happens (all but the clock's tick).
pub const ONCE_EVENTS: [&str; 8] = [
    "doorOpen",
    "doorClose",
    "curtain",
    "carDoor",
    "carStart",
    "letterbox",
    "till",
    "drawer",
];

/// A thing happening's sound by name (`EVENTS`). None: no such sound.
pub(super) fn make(kind: &str, rate: f64) -> Option<Stereo> {
    Some(match kind {
        "doorOpen" => door_open(rate),
        "doorClose" => door_close(rate),
        "curtain" => curtain(rate),
        "carDoor" => car_door(rate),
        "carStart" => car_start(rate),
        "letterbox" => letterbox(rate),
        "till" => till(rate),
        "drawer" => drawer(rate),
        "clockTick" => clock_tick(rate),
        _ => return None,
    })
}

/// Played once, dry, to a peak (`low`: the highs taken off above it; 0 for none).
fn dry(rate: f64, out: Ev, peak: f64, low: f64) -> Stereo {
    done_once(rate, vec![out.into()], 0.0, peak, &[], 0.0, low, 0.0)
}

/// A door opened: the handle pressed, the latch drawn back out of the keep, the leaf's air as it swings.
fn door_open(rate: f64) -> Stereo {
    let mut r = Rng::new(90.0);
    let mut out = once(rate, 0.7);
    strike(
        &mut out,
        0.01,
        0.5,
        0.0,
        &[[1800.0, 0.008, 1.0], [3400.0, 0.005, 0.5]],
        burst(2500.0, 1.0, 0.002, 0.8),
    );
    strike(
        &mut out,
        0.06,
        0.8,
        0.0,
        &[[950.0, 0.02, 1.0], [2300.0, 0.012, 0.5], [160.0, 0.03, 0.6]],
        burst(1400.0, 0.7, 0.004, 1.2),
    );
    swish(&mut out, &mut r, 0.1, 0.4, 0.2, 420.0);
    dry(rate, out, 0.8, 0.0)
}

/// A door falling shut: its air, the leaf meeting the frame, the latch snapping into the keep and the frame's
/// knock after it.
fn door_close(rate: f64) -> Stereo {
    let mut r = Rng::new(91.0);
    let mut out = once(rate, 0.7);
    swish(&mut out, &mut r, 0.0, 0.22, 0.2, 400.0);
    strike(
        &mut out,
        0.2,
        1.0,
        0.0,
        &[[110.0, 0.05, 1.0], [240.0, 0.03, 0.6], [520.0, 0.015, 0.3]],
        burst(420.0, 0.7, 0.012, 1.2),
    );
    strike(
        &mut out,
        0.215,
        0.45,
        0.0,
        &[[1100.0, 0.015, 1.0], [2600.0, 0.008, 0.5]],
        burst(1800.0, 0.8, 0.003, 1.0),
    );
    strike(
        &mut out,
        0.26,
        0.15,
        0.0,
        &[[330.0, 0.03, 1.0]],
        burst(700.0, 0.8, 0.01, 0.6),
    );
    dry(rate, out, 0.8, 0.0)
}

/// Curtains drawn (or opened): fourteen rings running along the rail, slowing as the cloth bunches, from one side
/// to the other, over the cloth's rustle.
fn curtain(rate: f64) -> Stereo {
    let mut r = Rng::new(92.0);
    let mut out = once(rate, 1.1);
    let rings = 14;
    let mut at = 0.05;
    let mut gap = 0.03;
    for k in 0..rings {
        let pan = -0.2 + 0.4 * k as f64 / (rings - 1) as f64;
        let amp = 0.12 + 0.08 * r.next();
        let hz = 3100.0 + 800.0 * r.next();
        strike(
            &mut out,
            at,
            amp,
            pan,
            &[[hz, 0.006, 1.0], [5200.0, 0.004, 0.4]],
            burst(4000.0, 1.0, 0.001, 0.8),
        );
        at += gap;
        gap *= 1.08;
    }
    rustle(&mut out, &mut r, 0.05, 0.8, 0.35, 0.0, 1500.0, 0.5);
    rustle(&mut out, &mut r, 0.05, 0.8, 0.12, 0.0, 4500.0, 0.6);
    dry(rate, out, 0.75, 0.0)
}

/// A car's door shut: its air, the heavy door meeting the body, the latch catching and the panel's ring after.
fn car_door(rate: f64) -> Stereo {
    let mut r = Rng::new(93.0);
    let mut out = once(rate, 0.8);
    swish(&mut out, &mut r, 0.0, 0.1, 0.2, 300.0);
    strike(
        &mut out,
        0.02,
        1.0,
        0.0,
        &[[75.0, 0.06, 1.0], [150.0, 0.035, 0.7], [310.0, 0.02, 0.35]],
        burst(280.0, 0.7, 0.02, 1.4),
    );
    strike(
        &mut out,
        0.025,
        0.35,
        0.0,
        &[[1300.0, 0.01, 1.0], [2700.0, 0.006, 0.4]],
        burst(2000.0, 0.8, 0.003, 1.0),
    );
    strike(
        &mut out,
        0.06,
        0.12,
        0.0,
        &[[480.0, 0.03, 1.0], [900.0, 0.02, 0.5]],
        burst(900.0, 0.8, 0.01, 0.5),
    );
    dry(rate, out, 0.8, 0.0)
}

/// A parked car started: the starter motor's whine, chugging with the engine's turns, then the engine catching,
/// flaring and settling to an idle, fading as the driving engine's loop comes in (`outdoors.rs` `car_engine`, whose
/// overtones it has).
fn car_start(rate: f64) -> Stereo {
    let mut r = Rng::new(94.0);
    let seconds = 2.2;
    let mut out = once(rate, seconds);
    let n = out.left.len();
    let time = |i: usize| i as f64 / rate;
    let starter = motor(n, rate, |_| 190.0, &[1.0, 0.6, 0.4, 0.3, 0.2]);
    let grind = noise(n, rate, &mut r, Filter::Bandpass, 900.0, 0.7);
    let firing = |t: f64| 30.0 + 28.0 * ramp(0.7, 1.0, t) - 14.0 * ramp(1.1, 1.7, t);
    let partials: Vec<f64> = (0..12)
        .map(|k| (if k % 2 == 1 { 0.7 } else { 1.0 }) / pow((k + 1) as f64, 1.15))
        .collect();
    let engine = motor(n, rate, |i| firing(time(i)), &partials);
    let exhaust = noise(n, rate, &mut r, Filter::Bandpass, 260.0, 0.7);
    // The firing's own phase (as `motor` turns it), for the exhaust's beat.
    let mut phase = 0.0;
    for i in 0..n {
        let t = time(i);
        phase += (2.0 * PI * firing(t)) / rate;
        if phase > 2.0 * PI {
            phase -= 2.0 * PI;
        }
        let crank = (0.6 + 0.4 * sin(2.0 * PI * 4.0 * t).abs())
            * ramp(0.0, 0.03, t)
            * (1.0 - ramp(0.7, 0.78, t));
        let b = 0.5 + 0.5 * sin(phase);
        let run = ramp(0.7, 0.85, t) * (1.0 - ramp(1.7, 2.2, t));
        let v = (starter[i] as f64 * 0.25 + grind[i] as f64 * 0.5) * crank
            + (engine[i] as f64 * 0.5 + exhaust[i] as f64 * 0.8 * (0.4 + 0.6 * b * b)) * run;
        put(&mut out, i as f64, v, 0.7, 0.7);
    }
    dry(rate, out, 0.8, 1800.0)
}

/// The post through a letterbox: the paper pushed through the slot, landing, and the flap snapping shut with its
/// bounce.
fn letterbox(rate: f64) -> Stereo {
    let mut r = Rng::new(95.0);
    let mut out = once(rate, 1.0);
    rustle(&mut out, &mut r, 0.08, 0.35, 0.3, 0.0, 2600.0, 0.6);
    strike(
        &mut out,
        0.42,
        0.2,
        0.0,
        &[[220.0, 0.02, 1.0]],
        burst(900.0, 0.6, 0.01, 0.8),
    );
    lid(&mut out, 0.5, 0.8);
    lid(&mut out, 0.6, 0.25);
    dry(rate, out, 0.8, 0.0)
}

/// A shop's till: a key, the bell, the drawer sliding out to its stop, coins counted into it and the drawer shut.
fn till(rate: f64) -> Stereo {
    let mut r = Rng::new(96.0);
    let mut out = once(rate, 1.3);
    strike(
        &mut out,
        0.0,
        0.3,
        0.0,
        &[[2400.0, 0.005, 1.0]],
        burst(3000.0, 1.0, 0.002, 0.8),
    );
    strike(
        &mut out,
        0.08,
        0.6,
        0.0,
        &[[2637.0, 0.35, 1.0], [5290.0, 0.2, 0.4], [7900.0, 0.1, 0.2]],
        burst(4500.0, 1.0, 0.001, 0.4),
    );
    rustle(&mut out, &mut r, 0.12, 0.25, 0.25, 0.0, 500.0, 0.8);
    strike(
        &mut out,
        0.36,
        0.5,
        0.0,
        &[[130.0, 0.04, 1.0], [330.0, 0.02, 0.4]],
        burst(450.0, 0.7, 0.012, 1.0),
    );
    for k in 0..4 {
        let at = 0.5 + 0.1 * (k as f64 + r.next());
        let hz = 3800.0 + 1500.0 * r.next();
        strike(
            &mut out,
            at,
            0.1,
            0.0,
            &[
                [hz, 0.03, 1.0],
                [hz * 1.53, 0.02, 0.5],
                [hz * 2.37, 0.012, 0.3],
            ],
            burst(hz * 1.5, 1.0, 0.001, 0.5),
        );
    }
    strike(
        &mut out,
        1.0,
        0.45,
        0.0,
        &[[150.0, 0.04, 1.0], [380.0, 0.02, 0.4]],
        burst(500.0, 0.7, 0.01, 1.0),
    );
    dry(rate, out, 0.75, 0.0)
}

/// A drawer: pulled out to its stop, the clothes in it rummaged, pushed shut.
fn drawer(rate: f64) -> Stereo {
    let mut r = Rng::new(97.0);
    let mut out = once(rate, 1.5);
    rustle(&mut out, &mut r, 0.02, 0.3, 0.4, 0.0, 600.0, 0.8);
    strike(
        &mut out,
        0.32,
        0.5,
        0.0,
        &[[160.0, 0.04, 1.0], [390.0, 0.025, 0.5], [900.0, 0.012, 0.2]],
        burst(600.0, 0.7, 0.01, 1.0),
    );
    rustle(&mut out, &mut r, 0.55, 0.4, 0.12, 0.1, 3000.0, 0.5);
    rustle(&mut out, &mut r, 1.05, 0.22, 0.35, 0.0, 600.0, 0.8);
    strike(
        &mut out,
        1.27,
        0.8,
        0.0,
        &[[120.0, 0.05, 1.0], [280.0, 0.03, 0.5]],
        burst(450.0, 0.7, 0.012, 1.2),
    );
    dry(rate, out, 0.8, 0.0)
}

/// A clock ticking, a second a beat: the tick, and the tock a little lower (a loop of two seconds).
fn clock_tick(rate: f64) -> Stereo {
    let mut out = looped(rate, 2.0);
    let modes = |k: f64| {
        [
            [2100.0 * k, 0.006, 1.0],
            [3900.0 * k, 0.004, 0.5],
            [650.0 * k, 0.012, 0.4],
        ]
    };
    strike(
        &mut out,
        0.0,
        0.5,
        0.0,
        &modes(1.0),
        burst(2800.0, 1.0, 0.0015, 1.0),
    );
    strike(
        &mut out,
        1.0,
        0.42,
        0.0,
        &modes(0.84),
        burst(2800.0, 1.0, 0.0015, 1.0),
    );
    done(rate, vec![out.into()], 0.0, 0.5, &[], 0.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 48000.0;

    /// Each as made (before the game's level): long enough, never past full scale, with sound in it, and the
    /// clock's end running on into its start (`things.rs` `world_sounds`).
    #[test]
    fn event_sounds() {
        for kind in EVENTS {
            let Stereo { left, right } =
                make(kind, RATE).unwrap_or_else(|| panic!("{kind}: not made"));
            assert_eq!(left.len(), right.len(), "{kind}");
            assert!(
                left.len() as f64 > RATE / 2.0,
                "{kind}: {} samples",
                left.len()
            );
            let (mut peak, mut sum) = (0.0f64, 0.0f64);
            for ch in [&left, &right] {
                for &v in ch.iter() {
                    peak = peak.max((v as f64).abs());
                    sum += v as f64 * v as f64;
                }
            }
            assert!(sum.is_finite(), "{kind}: not finite");
            assert!(peak <= 1.0 + 1e-6, "{kind}: peak {peak}");
            let rms = (sum / (2.0 * left.len() as f64)).sqrt();
            assert!(rms > 0.004, "{kind}: rms {rms}");
            if !ONCE_EVENTS.contains(&kind) {
                for ch in [&left, &right] {
                    let inside = ch
                        .windows(2)
                        .map(|w| (w[1] as f64 - w[0] as f64).abs())
                        .fold(0.0, f64::max);
                    let seam = (ch[0] as f64 - ch[ch.len() - 1] as f64).abs();
                    assert!(seam <= inside, "{kind}: seam {seam} over {inside}");
                }
            }
        }
    }

    #[test]
    fn the_clock_keeps_time() {
        let Stereo { left, .. } = make("clockTick", RATE).unwrap();
        assert_eq!(left.len(), 2 * RATE as usize);
    }

    #[test]
    fn only_the_events_are_made_here() {
        assert!(make("fridgeDoor", RATE).is_none());
        assert!(ONCE_EVENTS.iter().all(|k| EVENTS.contains(k)));
        assert!(!ONCE_EVENTS.contains(&"clockTick"));
    }
}
