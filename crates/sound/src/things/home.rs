//! The things' sounds at home and in the garden, made with the same building blocks as the others (`things`):
//! - cleaning: a vacuum cleaner, an iron on its board, a broom and dustpan, a mop and bucket;
//! - the garden: a petrol push mower, a hose's spray, a watering can, shears;
//! - a kettle boiling (and switching itself off: played once), someone typing, a sauna's stove with water
//!   ladled on, an electric fire's quiet hum, a knife and fork on a plate.

use super::*;
use crate::water::powi;

/// The sounds at home and in the garden, by name (`HOME`).
pub const HOME: [&str; 14] = [
    "vacuum",
    "iron",
    "sweep",
    "mop",
    "mower",
    "hose",
    "wateringCan",
    "shears",
    "kettle",
    "typing",
    "saunaStove",
    "fireplaceElectric",
    "cutlery",
    "kettleOff",
];

/// The one played once (a use's `endSound`): the kettle switching itself off.
pub const ONCE_HOME: [&str; 1] = ["kettleOff"];

/// A sound at home or in the garden by name (`HOME`). None: no such sound.
pub(super) fn make(kind: &str, rate: f64) -> Option<Stereo> {
    Some(match kind {
        "vacuum" => vacuum(rate),
        "iron" => iron(rate),
        "sweep" => sweep(rate),
        "mop" => mop(rate),
        "mower" => mower(rate),
        "hose" => hose(rate),
        "wateringCan" => watering_can(rate),
        "shears" => shears(rate),
        "kettle" => kettle(rate),
        "kettleOff" => kettle_off(rate),
        "typing" => typing(rate),
        "saunaStove" => sauna_stove(rate),
        "fireplaceElectric" => fireplace_electric(rate),
        "cutlery" => cutlery(rate),
        _ => return None,
    })
}

// Cleaning.

/// A cylinder vacuum cleaner pushed back and forth: its motor at about 26,000 turns a minute (440 Hz and its
/// harmonics, the fan's blades whining high above), the air's rush through the hose, its hiss at the nozzle,
/// the wheels' rumble. Each push the nozzle seals on the carpet: the motor labours and slows a little and the
/// rush swells (five pushes a loop).
fn vacuum(rate: f64) -> Stereo {
    let mut r = Rng::new(70.0);
    let seconds = 8.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let load = wander(&mut r, n, rate, 0.4);
    let stroke = |i: usize| 0.5 + 0.5 * sin(2.0 * PI * 0.625 * i as f64 / rate);
    let speed = |i: usize| 440.0 * (1.0 - 0.03 * stroke(i) - 0.01 * load[i] as f64);
    let hum = motor(n, rate, speed, &[1.0, 0.6, 0.45, 0.3, 0.25, 0.2, 0.15, 0.1]);
    let whine = motor(n, rate, |i| 9.0 * speed(i), &[1.0, 0.2]);
    let rush = noise(n, rate, &mut r, Filter::Bandpass, 1400.0, 0.4);
    let hiss = noise(n, rate, &mut r, Filter::Highpass, 3800.0, 0.6);
    let wheels = noise(n, rate, &mut r, Filter::Lowpass, 220.0, 0.7);
    for i in 0..n {
        let s = stroke(i);
        let v = hum[i] as f64 * 0.05
            + whine[i] as f64 * 0.006
            + rush[i] as f64 * (0.9 + 0.25 * s)
            + hiss[i] as f64 * 0.22
            + wheels[i] as f64 * 0.3 * s;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    done(
        rate,
        vec![fold(bed)],
        0.16,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.12,
        0.0,
        0.0,
    )
}

/// Ironing: the iron's soleplate gliding over the cloth in long strokes, a puff of steam now and then, the
/// thermostat clicking, the board creaking under a push.
fn iron(rate: f64) -> Stereo {
    let mut r = Rng::new(71.0);
    let seconds = 10.0;
    let mut ev = looped(rate, seconds);
    for k in 0..8 {
        let at = 0.2 + k as f64 * 1.25;
        let pan = if k % 2 == 0 { -0.15 } else { 0.15 };
        rustle(&mut ev, &mut r, at, 0.9, 0.10, pan, 1100.0, 0.5);
    }
    for at in [1.6, 4.9, 8.1] {
        let at = at + 0.2 * r.next();
        rustle(&mut ev, &mut r, at, 0.7, 0.45, 0.0, 5200.0, 0.6);
        rustle(&mut ev, &mut r, at, 0.35, 0.2, 0.0, 8000.0, 0.8);
    }
    strike(
        &mut ev,
        6.4,
        0.2,
        0.0,
        &[[2100.0, 0.008, 1.0]],
        burst(2600.0, 1.0, 0.002, 0.8),
    );
    for at in [3.2, 7.4] {
        strike(
            &mut ev,
            at,
            0.25,
            0.0,
            &[[140.0, 0.05, 1.0], [310.0, 0.03, 0.4]],
            burst(500.0, 0.7, 0.01, 0.8),
        );
    }
    done(
        rate,
        vec![ev.into()],
        SPARSE,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.15,
        0.0,
        0.0,
    )
}

/// A floor swept: strokes of the broom's bristles in short runs across the floor, then the dust swept onto
/// the dustpan (its edge knocking down, two little brushes).
fn sweep(rate: f64) -> Stereo {
    let mut r = Rng::new(72.0);
    let seconds = 9.0;
    let mut ev = looped(rate, seconds);
    let mut t = 0.3;
    for _ in 0..4 {
        let strokes = if r.next() < 0.5 { 3 } else { 4 };
        for k in 0..strokes {
            let pan = -0.2 + 0.4 * k as f64 / (strokes - 1) as f64;
            let amp = if k % 2 == 0 { 0.5 } else { 0.42 };
            rustle(&mut ev, &mut r, t, 0.35, amp, pan, 3200.0, 0.6);
            rustle(&mut ev, &mut r, t, 0.38, 0.2, pan, 700.0, 0.6);
            t += 0.38;
        }
        t += 0.5;
    }
    strike(
        &mut ev,
        t,
        0.4,
        0.1,
        &[
            [1300.0, 0.02, 1.0],
            [2900.0, 0.012, 0.5],
            [420.0, 0.03, 0.4],
        ],
        burst(1800.0, 0.7, 0.004, 1.0),
    );
    for k in 0..2 {
        let at = t + 0.15 + 0.25 * k as f64;
        rustle(&mut ev, &mut r, at, 0.18, 0.3, 0.1, 4200.0, 0.7);
    }
    done(
        rate,
        vec![ev.into()],
        SPARSE,
        0.0,
        &[7.1, 9.3, 11.7, 13.9],
        0.12,
        0.0,
        0.0,
    )
}

/// A floor mopped: the mop dunked in its bucket, wrung out in the wringer (a creak, the water dripping back),
/// six wet strokes over the floor, the handle knocking the bucket as it goes back in; then again.
fn mop(rate: f64) -> Stereo {
    let mut r = Rng::new(73.0);
    let seconds = 12.0;
    let mut wet = track(rate, seconds, 0.6);
    let mut ev = looped(rate, seconds);
    slosh(&mut wet, rate, &mut r, 0.4, 0.7, 0.5, -0.2, 900.0);
    slosh(&mut wet, rate, &mut r, 0.9, 0.5, 0.35, -0.2, 900.0);
    slosh(&mut wet, rate, &mut r, 9.6, 0.6, 0.45, -0.2, 900.0);
    for (from, to) in [(1.8, 3.0), (10.2, 11.2)] {
        strike(
            &mut ev,
            from,
            0.3,
            -0.2,
            &[[240.0, 0.05, 1.0], [610.0, 0.03, 0.4]],
            burst(800.0, 0.8, 0.01, 0.8),
        );
        for c in 0..2 {
            let mut t = from + r.next() * 0.05;
            while t < to {
                let f = 700.0 + 1100.0 * r.next();
                bubble(wet.ch(c), rate, t, f, 0.05, 0.3);
                t += -log(1.0 - r.next()) / 14.0;
            }
        }
    }
    for k in 0..6 {
        let at = 3.6 + 0.95 * k as f64;
        let pan = if k % 2 == 0 { -0.25 } else { 0.25 };
        rustle(&mut ev, &mut r, at, 0.85, 0.45, pan, 1300.0, 0.5);
        rustle(&mut ev, &mut r, at, 0.85, 0.15, pan, 3800.0, 0.6);
    }
    strike(
        &mut ev,
        11.5,
        0.25,
        -0.2,
        &[[820.0, 0.05, 0.6], [1900.0, 0.03, 0.4]],
        burst(1500.0, 0.7, 0.003, 1.0),
    );
    done(
        rate,
        vec![fold(wet), ev.into()],
        SPARSE,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.18,
        0.0,
        0.0,
    )
}

// The garden.

/// A petrol push mower: a four-stroke single at about 3,000 turns a minute (firing 25 times a second, its
/// harmonics up through the crank's 50 Hz and the blade's 100 Hz), the exhaust's rasp chopped at the firing,
/// the deck's rattle and the grass cut under it, the engine dipping as it bites into thicker grass. Out of
/// doors.
fn mower(rate: f64) -> Stereo {
    let mut r = Rng::new(74.0);
    let seconds = 8.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let load = wander(&mut r, n, rate, 0.5);
    let churn = wander(&mut r, n, rate, 0.05);
    let speed = |i: usize| 50.0 * (1.0 - 0.04 * max(0.0, load[i] as f64) - 0.01 * churn[i] as f64);
    let firing = motor(
        n,
        rate,
        |i| 0.5 * speed(i),
        &[
            0.6, 1.0, 0.7, 0.8, 0.5, 0.45, 0.35, 0.3, 0.22, 0.18, 0.14, 0.1, 0.08, 0.06,
        ],
    );
    let exhaust = noise(n, rate, &mut r, Filter::Bandpass, 260.0, 0.7);
    let deck = noise(n, rate, &mut r, Filter::Bandpass, 900.0, 0.5);
    let grass = noise(n, rate, &mut r, Filter::Bandpass, 2600.0, 0.6);
    let mut phase = 0.0;
    for i in 0..n {
        phase += (2.0 * PI * 0.5 * speed(i)) / rate;
        if phase > 2.0 * PI {
            phase -= 2.0 * PI;
        }
        let b = 0.5 + 0.5 * sin(phase);
        let v = firing[i] as f64 * 0.09
            + exhaust[i] as f64 * 1.4 * (0.4 + 0.6 * b * b)
            + deck[i] as f64 * 0.5
            + grass[i] as f64 * 0.35 * (0.5 + 0.5 * max(0.0, load[i] as f64));
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    done(rate, vec![fold(bed)], 0.18, 0.0, &[], 0.0, 0.0, 0.0)
}

/// A garden hose's spray swept over the beds: a broad hiss on the leaves, brighter or duller as it moves, its
/// fizz above, the soft thud of the water on the soil and drops pattering off the leaves. Out of doors.
fn hose(rate: f64) -> Stereo {
    let mut r = Rng::new(75.0);
    let mut out = track(rate, 8.0, 0.6);
    let n = out.left.len();
    let sweep = wander(&mut r, n, rate, 0.8);
    let mut shared = Pink::new();
    for c in 0..2 {
        let ch = out.ch(c);
        let mut own = Pink::new();
        let mut spray = Biquad::new(rate);
        let mut fizz = Biquad::with(rate, Filter::Highpass, 5000.0, 0.6);
        let mut soil = Biquad::with(rate, Filter::Lowpass, 380.0, 0.7);
        let mut patter = Biquad::with(rate, Filter::Bandpass, 2600.0, 0.7);
        let mut sh = Pink::new();
        for i in 0..n {
            let s = sweep[i] as f64;
            if i & 31 == 0 {
                spray.set(Filter::Bandpass, 3000.0 * pow(2.0, 0.25 * s), 0.5);
            }
            let w = 0.55 * shared.next(&mut r) + 0.8 * own.next(&mut r);
            let drop = if r.next() < 900.0 / rate {
                (r.next() - 0.5) * 5.0 * powi(r.next(), 3)
            } else {
                0.0
            };
            let v = spray.run(w) * 1.1 * (1.0 + 0.25 * s)
                + fizz.run(w) * 0.4
                + soil.run(sh.next(&mut r)) * 0.18
                + patter.run(drop) * 0.7;
            ch[i] = v as f32;
        }
    }
    finish(out, 0.2, 1.0)
}

/// A watering can: three pours from its rose, each a gentle rush of water on leaves and soil with drops
/// ticking off them and a glug of air back into the can as it tips. Out of doors.
fn watering_can(rate: f64) -> Stereo {
    let mut r = Rng::new(76.0);
    let seconds = 9.0;
    let mut out = track(rate, seconds, 0.6);
    stream(
        &mut out,
        &mut r,
        StreamOpts {
            centre: 2400.0,
            q: 0.6,
            bubbles: 90.0,
            gurgle: 0.06,
            low: 600.0,
            high: 2400.0,
            rumble: 0.05,
        },
    );
    let pours = [(0.3, 2.6), (3.4, 5.6), (6.3, 8.5)];
    for (a, b) in pours {
        for c in 0..2 {
            let mut t = a + r.next() * 0.005;
            while t < b {
                let amp = 0.04 * (0.2 + powi(r.next(), 3));
                tick(out.ch(c), rate, t, amp);
                t += -log(1.0 - r.next()) / 400.0;
            }
        }
        let f = 180.0 + 60.0 * r.next();
        for c in 0..2 {
            bubble(out.ch(c), rate, a + 0.6, f, 0.12, 0.2);
        }
    }
    // The pours (the overlap after the loop as its start, so it folds in whole).
    let n = out.left.len();
    let on: Vec<f64> = (0..n)
        .map(|i| {
            let t = (i as f64 / rate) % seconds;
            pours
                .iter()
                .map(|&(a, b)| ramp(a, a + 0.15, t) * (1.0 - ramp(b - 0.2, b, t)))
                .sum()
        })
        .collect();
    for c in 0..2 {
        for (v, g) in out.ch(c).iter_mut().zip(&on) {
            *v = (*v as f64 * g) as f32;
        }
    }
    finish(out, 0.13, 1.0)
}

/// Shears cutting back a bush: runs of snips (the blades sliding, then meeting with a ring; now and then a
/// twig cracking between them), the leaves rustling as they're pulled aside between runs. Out of doors.
fn shears(rate: f64) -> Stereo {
    let mut r = Rng::new(77.0);
    let seconds = 9.0;
    let mut ev = looped(rate, seconds);
    let mut t = 0.3;
    while t < seconds - 0.6 {
        let snips = 3 + (r.next() * 3.0).floor() as usize;
        let mut last = t;
        for _ in 0..snips {
            if t >= seconds - 0.45 {
                break;
            }
            let j = 0.96 + 0.08 * r.next();
            let amp = 0.45 + 0.25 * r.next();
            let pan = (r.next() - 0.5) * 0.4;
            rustle(&mut ev, &mut r, t, 0.07, 0.25, pan, 5600.0, 1.2);
            strike(
                &mut ev,
                t + 0.06,
                amp,
                pan,
                &[
                    [1900.0 * j, 0.03, 1.0],
                    [3300.0 * j, 0.02, 0.6],
                    [5200.0 * j, 0.012, 0.35],
                    [310.0, 0.02, 0.4],
                ],
                burst(3000.0, 0.8, 0.002, 1.0),
            );
            if r.next() < 0.3 {
                strike(
                    &mut ev,
                    t + 0.05,
                    0.3,
                    pan,
                    &[[620.0, 0.008, 1.0]],
                    burst(1500.0, 0.8, 0.004, 1.2),
                );
            }
            last = t;
            t += 0.45 + 0.15 * r.next();
        }
        let gap = 0.8 + 0.6 * r.next();
        let pan = (r.next() - 0.5) * 0.4;
        rustle(&mut ev, &mut r, last + 0.35, 0.4, 0.12, pan, 3500.0, 0.5);
        t = last + gap;
    }
    done(rate, vec![ev.into()], SPARSE, 0.0, &[], 0.0, 0.0, 0.0)
}

// In the kitchen, at a desk, in the sauna, by the fire, at the table.

/// A kettle at a rolling boil: the element's faint hum, the water's rumble and seethe, small bubbles bursting
/// and bigger ones rolling up, all inside its body (a loop starts anywhere, so it cannot rise to the boil).
fn kettle(rate: f64) -> Stereo {
    let mut r = Rng::new(78.0);
    let seconds = 8.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let hum = motor(n, rate, |_| 100.0, &[1.0, 0.4, 0.2]);
    let rumble = noise(n, rate, &mut r, Filter::Lowpass, 420.0, 0.8);
    let seethe = noise(n, rate, &mut r, Filter::Bandpass, 1300.0, 0.5);
    let roll = wander(&mut r, n, rate, 0.2);
    let swell = wander(&mut r, n, rate, 0.5);
    for i in 0..n {
        let v = hum[i] as f64 * 0.015
            + rumble[i] as f64 * 0.7 * (1.0 + 0.3 * roll[i] as f64)
            + seethe[i] as f64 * 0.45 * (0.8 + 0.2 * swell[i] as f64);
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let length = bed.seconds;
    for c in 0..2 {
        let mut t = r.next() * 0.01;
        while t < length {
            let f = 1200.0 + 2800.0 * r.next();
            bubble(bed.ch(c), rate, t, f, 0.02, 0.3);
            t += -log(1.0 - r.next()) / 80.0;
        }
        let mut t = r.next() * 0.05;
        while t < length {
            let f = 250.0 + 450.0 * r.next();
            bubble(bed.ch(c), rate, t, f, 0.06, 0.2);
            t += -log(1.0 - r.next()) / 12.0;
        }
        // Inside its body: the highs muffled before the loop is folded, so its end runs on into its start.
        muffle(bed.ch(c), rate, 2600.0, false);
    }
    done(
        rate,
        vec![fold(bed)],
        0.12,
        0.0,
        &[2.3, 3.7, 5.1, 6.7],
        0.15,
        0.0,
        0.0,
    )
}

/// A kettle switching itself off: the switch flicking up with a clack (and its spring), the boil dying away,
/// the last few bubbles.
fn kettle_off(rate: f64) -> Stereo {
    let mut r = Rng::new(79.0);
    let mut out = once(rate, 1.4);
    let n = out.left.len();
    let mut band = Biquad::with(rate, Filter::Bandpass, 1400.0, 0.5);
    let mut pink = Pink::new();
    for i in 0..n {
        let t = i as f64 / rate;
        put(
            &mut out,
            i as f64,
            band.run(pink.next(&mut r)) * 0.6 * exp(-t / 0.35),
            0.7,
            0.7,
        );
    }
    strike(
        &mut out,
        0.12,
        0.4,
        0.0,
        &[
            [1500.0, 0.012, 1.0],
            [3100.0, 0.008, 0.6],
            [240.0, 0.03, 0.5],
        ],
        burst(2200.0, 0.8, 0.003, 1.2),
    );
    strike(
        &mut out,
        0.17,
        0.3,
        0.0,
        &[[900.0, 0.015, 1.0]],
        burst(1200.0, 0.8, 0.003, 0.8),
    );
    for _ in 0..6 {
        let at = 0.2 + 0.7 * r.next();
        let f = 300.0 + 400.0 * r.next();
        bubble(&mut out.left, rate, at, f, 0.08, 0.2);
        bubble(&mut out.right, rate, at, f, 0.08, 0.2);
    }
    done_once(
        rate,
        vec![out.into()],
        0.0,
        0.8,
        &[2.3, 3.7, 5.1, 6.7],
        0.15,
        0.0,
        0.0,
    )
}

/// Someone typing: bursts of keys (each a click as it bottoms out and a lighter one as it's let go; now and
/// then the space bar's duller thump), pauses to think, sometimes a mouse's two clicks.
fn typing(rate: f64) -> Stereo {
    let mut r = Rng::new(80.0);
    let seconds = 10.0;
    let mut ev = looped(rate, seconds);
    let mut t = 0.3;
    while t < seconds - 0.5 {
        let keys = 4 + (r.next() * 9.0).floor() as usize;
        let pace = 6.0 + 3.0 * r.next();
        for _ in 0..keys {
            if t >= seconds - 0.15 {
                break;
            }
            if r.next() < 0.15 {
                strike(
                    &mut ev,
                    t,
                    0.4,
                    0.0,
                    &[[900.0, 0.01, 1.0], [1700.0, 0.006, 0.5], [220.0, 0.02, 0.6]],
                    burst(1400.0, 0.7, 0.003, 1.0),
                );
            } else {
                let amp = 0.3 * (0.6 + 0.4 * r.next());
                let pan = (r.next() - 0.5) * 0.5;
                let a = 2300.0 + 900.0 * r.next();
                let b = 4200.0 + 800.0 * r.next();
                strike(
                    &mut ev,
                    t,
                    amp,
                    pan,
                    &[[a, 0.004, 1.0], [b, 0.003, 0.5], [420.0, 0.012, 0.35]],
                    burst(3200.0, 0.8, 0.0015, 1.0),
                );
                let up = t + 0.07 + 0.03 * r.next();
                strike(
                    &mut ev,
                    up,
                    0.1,
                    pan,
                    &[[2900.0, 0.003, 1.0]],
                    burst(3600.0, 1.0, 0.001, 0.7),
                );
            }
            t += (0.75 + 0.5 * r.next()) / pace;
        }
        let pause = 0.4 + 0.8 * r.next();
        if r.next() < 0.4 {
            for k in 0..2 {
                strike(
                    &mut ev,
                    t + 0.4 * pause + 0.09 * k as f64,
                    0.2,
                    0.4,
                    &[[3400.0, 0.003, 1.0]],
                    burst(4000.0, 1.0, 0.001, 0.8),
                );
            }
        }
        t += pause;
    }
    done(
        rate,
        vec![ev.into()],
        SPARSE,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.12,
        0.0,
        0.0,
    )
}

/// A sauna's stove: the heat's low roar, the stones ticking and popping as they heat, and once a loop water
/// ladled on (the ladle knocking the bucket, the splash, the stones sizzling and a gust of steam).
fn sauna_stove(rate: f64) -> Stereo {
    let mut r = Rng::new(81.0);
    let seconds = 14.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let drift = wander(&mut r, n, rate, 1.2);
    let roar = noise(n, rate, &mut r, Filter::Lowpass, 260.0, 0.7);
    let steam = noise(n, rate, &mut r, Filter::Bandpass, 4200.0, 0.6);
    let hiss = noise(n, rate, &mut r, Filter::Highpass, 6000.0, 0.6);
    for i in 0..n {
        let t = i as f64 / rate;
        let h = ramp(5.0, 5.08, t) * exp(-max(0.0, t - 5.08) / 1.5);
        let v = roar[i] as f64 * 0.5 * (1.0 + 0.2 * drift[i] as f64)
            + steam[i] as f64 * 1.2 * h
            + hiss[i] as f64 * 0.5 * h;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    let mut t = r.next() * 0.1;
    while t < seconds {
        let amp = 0.12 * powi(r.next(), 3);
        let pan = (r.next() - 0.5) * 0.8;
        let hz = 2000.0 + 4000.0 * r.next();
        let tau = 0.001 + 0.002 * r.next();
        strike(&mut ev, t, amp, pan, &[], burst(hz, 1.0, tau, 1.5));
        t += -log(1.0 - r.next()) / 5.0;
    }
    let mut t = 0.8 + r.next();
    while t < seconds - 0.2 {
        let pan = (r.next() - 0.5) * 0.8;
        let low = 700.0 + 500.0 * r.next();
        strike(
            &mut ev,
            t,
            0.3,
            pan,
            &[[low, 0.008, 0.6]],
            burst(1800.0, 0.8, 0.004, 1.2),
        );
        t += 1.5 + 2.0 * r.next();
    }
    strike(
        &mut ev,
        4.6,
        0.3,
        -0.2,
        &[[520.0, 0.06, 1.0], [1340.0, 0.04, 0.5]],
        burst(900.0, 0.7, 0.006, 0.8),
    );
    rustle(&mut ev, &mut r, 5.0, 0.25, 0.5, 0.1, 1800.0, 0.6);
    let mut t = 5.05;
    while t < 6.8 {
        let amp = 0.2 * powi(r.next(), 2);
        let pan = (r.next() - 0.5) * 0.8;
        let hz = 3000.0 + 4000.0 * r.next();
        let tau = 0.0008 + 0.001 * r.next();
        strike(&mut ev, t, amp, pan, &[], burst(hz, 1.0, tau, 1.5));
        t += -log(1.0 - r.next()) / 40.0;
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.1,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.12,
        0.0,
        0.0,
    )
}

/// An electric fire: its transformer's quiet hum, the fan blowing the warm air out, the flame effect's drum
/// ticking round twice a second, the metal ticking now and then as it warms.
fn fireplace_electric(rate: f64) -> Stereo {
    let mut r = Rng::new(82.0);
    let seconds = 8.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let wobble = wander(&mut r, n, rate, 1.5);
    let hum = motor(n, rate, |_| 100.0, &[1.0, 0.4, 0.25, 0.1]);
    let fan = noise(n, rate, &mut r, Filter::Bandpass, 700.0, 0.5);
    for i in 0..n {
        let v = hum[i] as f64 * 0.03 + fan[i] as f64 * 0.5 * (1.0 + 0.1 * wobble[i] as f64);
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    // Inside its casing: the highs muffled before the loop is folded.
    for c in 0..2 {
        muffle(bed.ch(c), rate, 2400.0, false);
    }
    let mut ev = looped(rate, seconds);
    for k in 0..16 {
        strike(
            &mut ev,
            0.25 + 0.5 * k as f64,
            0.05,
            0.0,
            &[[1800.0, 0.004, 1.0]],
            burst(2400.0, 1.0, 0.001, 0.6),
        );
    }
    for at in [2.3, 6.1] {
        strike(
            &mut ev,
            at + 0.2 * r.next(),
            0.15,
            0.0,
            &[[2900.0, 0.01, 1.0], [5300.0, 0.006, 0.5]],
            burst(3500.0, 1.0, 0.002, 1.0),
        );
    }
    // The ticks muffled too, round the loop (they wrap).
    muffle(&mut ev.left, rate, 2400.0, true);
    muffle(&mut ev.right, rate, 2400.0, true);
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.1,
        0.0,
        &[3.7, 6.1, 8.3, 10.9],
        0.12,
        0.0,
        0.0,
    )
}

/// A meal eaten with a knife and fork: the fork on the china, the knife and the fork touching, the knife
/// cutting with a soft scrape; a few seconds between.
fn cutlery(rate: f64) -> Stereo {
    let mut r = Rng::new(83.0);
    let seconds = 12.0;
    let mut out = track(rate, seconds, 0.6);
    let mut ev = looped(rate, seconds);
    let china = [1.0, 2.76, 5.4, 8.93];
    let steel = [1.0, 1.53, 2.37, 3.11];
    let mut t = 0.3 + r.next() * 0.4;
    while t < seconds {
        let pan = (r.next() - 0.5) * 0.4;
        let what = r.next();
        if what < 0.5 {
            let f = 2200.0 + 1400.0 * r.next();
            let amp = 0.1 + 0.2 * powi(r.next(), 2);
            clink(&mut out, rate, t, f, amp, pan, &china, 0.06, false);
            if r.next() < 0.4 {
                let at = t + 0.08 + 0.12 * r.next();
                let f = 2200.0 + 1400.0 * r.next();
                clink(&mut out, rate, at, f, amp * 0.5, pan, &china, 0.06, false);
            }
        } else if what < 0.8 {
            let touches = 2 + (r.next() * 2.0).floor() as usize;
            let mut at = t;
            for _ in 0..touches {
                let f = 3200.0 + 1800.0 * r.next();
                let amp = 0.08 + 0.08 * r.next();
                clink(&mut out, rate, at, f, amp, pan, &steel, 0.12, false);
                at += 0.03 + 0.04 * r.next();
            }
        } else {
            rustle(&mut ev, &mut r, t, 0.15, 0.06, pan, 4500.0, 1.5);
            let f = 2200.0 + 1400.0 * r.next();
            clink(&mut out, rate, t + 0.18, f, 0.08, pan, &china, 0.06, false);
        }
        t += 0.9 + 1.3 * r.next();
    }
    done(
        rate,
        vec![fold(out), ev.into()],
        0.0,
        0.7,
        &[2.9, 4.7, 7.9, 9.1],
        0.15,
        0.0,
        0.0,
    )
}

/// The highs over `hz` muffled (inside a box: a kettle's body, an electric fire's casing). Done before a loop
/// is put together, so its seam stays whole: on a track yet to be folded, or with `round` on a loop of events,
/// the filter first run once round it so it starts where the loop ends.
fn muffle(d: &mut [f32], rate: f64, hz: f64, round: bool) {
    let mut lp = Biquad::with(rate, Filter::Lowpass, hz, 0.7);
    if round {
        for &v in d.iter() {
            lp.run(v as f64);
        }
    }
    for v in d.iter_mut() {
        *v = lp.run(*v as f64) as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 48000.0;
    /// The rates a sound card runs at: a loop's seam is checked at each (a filter's state at the seam differs).
    const RATES: [f64; 2] = [48000.0, 44100.0];

    /// The biggest step between neighbouring samples of a channel, and the step from its last sample round
    /// to its first.
    fn steps(ch: &[f32]) -> (f64, f64) {
        let inside = ch
            .windows(2)
            .map(|w| (w[1] as f64 - w[0] as f64).abs())
            .fold(0.0, f64::max);
        (inside, (ch[0] as f64 - ch[ch.len() - 1] as f64).abs())
    }

    /// Each as made (before the game's level), as the other things' (`things` `world_sounds`): long enough,
    /// never past full scale, with sound in it, and a loop's end running on into its start; at each rate.
    #[test]
    fn home_sounds() {
        for rate in RATES {
            for kind in HOME {
                let Stereo { left, right } =
                    make(kind, rate).unwrap_or_else(|| panic!("{kind}: not made"));
                assert_eq!(left.len(), right.len(), "{kind} at {rate}");
                assert!(
                    left.len() as f64 > rate / 2.0,
                    "{kind} at {rate}: {} samples",
                    left.len()
                );
                let (mut peak, mut sum) = (0.0f64, 0.0f64);
                for ch in [&left, &right] {
                    for &v in ch.iter() {
                        peak = peak.max((v as f64).abs());
                        sum += v as f64 * v as f64;
                    }
                }
                assert!(sum.is_finite(), "{kind} at {rate}: not finite");
                assert!(peak <= 1.0 + 1e-6, "{kind} at {rate}: peak {peak}");
                let rms = (sum / (2.0 * left.len() as f64)).sqrt();
                assert!(rms > 0.004, "{kind} at {rate}: rms {rms}");
                if !ONCE_HOME.contains(&kind) {
                    for ch in [&left, &right] {
                        let (inside, seam) = steps(ch);
                        assert!(
                            seam <= inside,
                            "{kind} at {rate}: seam {seam} over {inside}"
                        );
                    }
                }
            }
        }
    }

    /// Reached through the things' own `make`, and nothing else is made here.
    #[test]
    fn made_through_the_things() {
        assert!(HOME.iter().all(|k| super::super::make(k, RATE).is_some()));
        assert!(ONCE_HOME.iter().all(|k| HOME.contains(k)));
        assert!(make("shower", RATE).is_none());
        assert!(make("blender", RATE).is_none());
    }
}
