//! The sounds of things in use, synthesised like the water (\`water\`, whose building blocks they share; no
//! sound files):
//! - machines: a treadmill, the bikes, a sewing machine, a pottery wheel, coffee machines, a blender and a
//!   stand mixer, a microwave, a toaster, an oven's fan, a cooker hood;
//! - games: pool, foosball, ping-pong, darts, an arcade cabinet, basketball;
//! - a workbench's tools and a bike's, a fire, a chemistry set bubbling, a punch bag, crockery smashed in a
//!   bin, a cello played with a bow, someone singing to a karaoke track;
//! - played once: a fridge door, a pedal bin, the vending machines, a phone box's call and its hanging up,
//!   the microwave's ping, toast popping up, coffee poured into a cup.
//!
//! Most are made the same few ways:
//! - a motor or a fan: a tone at its speed with its harmonics (a transformer's hum at twice the mains,
//!   100 Hz; a fan's blades passing), over the rush of air or the rumble of what it drives (the harmonics
//!   turned from the first, \`harmonics\`: one \`sin\` a sample, not one each);
//! - something struck (\`strike\`): a few modes ringing out over a short burst of noise, at the frequencies of
//!   the thing's size and stuff (a basketball's air rings at about 950 Hz), rung as a bank of decaying
//!   rotations (\`Modal\`: no \`sin\` or \`exp\` a sample);
//! - events in a loop (darts, a rally) wrap round its end into its start, so nothing is cut off at the
//!   seam; steady sounds under them are cross-faded as the water's are.
//!
//! Each is a stereo loop several seconds long (or a sound played once: \`ONCE_THINGS\`), made off the main
//! thread the first time it's wanted. Was \`web/src/audio/things.ts\`; the arithmetic is the TypeScript's
//! (\`dsp\`), to rounding where a tone is turned by rotation rather than a \`sin\` a sample.

use crate::dsp::m::*;
use crate::dsp::*;

/// The sounds of things, by name (\`THINGS\`).
pub const THINGS: [&str; 34] = [
    "treadmill",
    "bike",
    "tools",
    "bikeTools",
    "pool",
    "foosball",
    "pingPong",
    "darts",
    "arcade",
    "sewing",
    "potteryWheel",
    "bubbling",
    "espresso",
    "blender",
    "mixer",
    "microwave",
    "toaster",
    "ovenFan",
    "hood",
    "fire",
    "basketball",
    "singing",
    "cello",
    "punchBag",
    "smash",
    "fridgeDoor",
    "binLid",
    "vendSnack",
    "vendCan",
    "phoneBox",
    "hangUp",
    "ping",
    "toastPop",
    "pour",
];

/// The ones played once as something happens (a use starting or ending), not looped while it lasts.
pub const ONCE_THINGS: [&str; 9] = [
    "fridgeDoor",
    "binLid",
    "vendSnack",
    "vendCan",
    "phoneBox",
    "hangUp",
    "ping",
    "toastPop",
    "pour",
];

/// A thing's sound by name (\`THINGS\`). None: no such thing.
pub fn make(kind: &str, rate: f64) -> Option<Stereo> {
    Some(match kind {
        "treadmill" => treadmill(rate),
        "bike" => bike(rate),
        "tools" => tools(rate),
        "bikeTools" => bike_tools(rate),
        "pool" => pool(rate),
        "foosball" => foosball(rate),
        "pingPong" => ping_pong(rate),
        "darts" => darts(rate),
        "arcade" => arcade(rate),
        "sewing" => sewing(rate),
        "potteryWheel" => pottery_wheel(rate),
        "bubbling" => bubbling(rate),
        "espresso" => espresso(rate),
        "blender" => blender(rate),
        "mixer" => mixer(rate),
        "microwave" => microwave(rate),
        "toaster" => toaster(rate),
        "ovenFan" => oven_fan(rate),
        "hood" => hood(rate),
        "fire" => fire(rate),
        "basketball" => basketball(rate),
        "singing" => singing(rate),
        "cello" => cello(rate),
        "punchBag" => punch_bag(rate),
        "smash" => smash(rate),
        "fridgeDoor" => fridge_door(rate),
        "binLid" => bin_lid(rate),
        "vendSnack" => vend_snack(rate),
        "vendCan" => vend_can(rate),
        "phoneBox" => phone_box(rate),
        "hangUp" => hang_up(rate),
        "ping" => ping(rate),
        "toastPop" => toast_pop(rate),
        "pour" => pour(rate),
        _ => return None,
    })
}

// The machines.

/// A treadmill with someone running on it: the motor's hum (about 58 Hz, turning the belt at a run), the
/// belt's hiss over the deck, the rollers' rumble, the feet landing at 165 steps a minute (each a thump in
/// the deck and a slap on the belt, one foot a little heavier) and the belt's seam ticking past once a turn.
fn treadmill(rate: f64) -> Stereo {
    let mut r = Rng::new(31.0);
    let seconds = 8.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let wobble = wander(&mut r, n, rate, 0.5);
    let hum = motor(
        n,
        rate,
        |i| 58.0 * (1.0 + 0.006 * wobble[i] as f64),
        &[0.9, 1.0, 0.5, 0.45, 0.2, 0.25, 0.1, 0.12],
    );
    let whine = motor(
        n,
        rate,
        |i| 1240.0 * (1.0 + 0.004 * wobble[i] as f64),
        &[1.0, 0.2],
    );
    let hiss = noise(n, rate, &mut r, Filter::Bandpass, 900.0, 0.6);
    let rumble = noise(n, rate, &mut r, Filter::Lowpass, 140.0, 0.7);
    for i in 0..n {
        let v = hum[i] as f64 * 0.06
            + whine[i] as f64 * 0.004
            + hiss[i] as f64 * 0.5
            + rumble[i] as f64 * 0.5;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    // 22 steps in the 8 seconds, so the rhythm runs on round the loop.
    let step = seconds / 22.0;
    for k in 0..22 {
        let at = k as f64 * step + (r.next() - 0.5) * 0.012;
        let heavy = if k % 2 == 1 { 0.85 } else { 1.0 };
        strike(
            &mut ev,
            at,
            0.9 * heavy,
            if k % 2 == 1 { 0.08 } else { -0.08 },
            &[[72.0, 0.045, 1.0], [150.0, 0.025, 0.4]],
            burst(900.0, 0.7, 0.012, 1.6),
        );
        // The shoe leaving the belt.
        strike(
            &mut ev,
            at + 0.12,
            0.15,
            0.0,
            &[],
            burst(2600.0, 0.8, 0.02, 1.0),
        );
    }
    for k in 0..9 {
        strike(
            &mut ev,
            k as f64 * (seconds / 9.0) + 0.05,
            0.12,
            0.0,
            &[[2100.0, 0.004, 1.0]],
            burst(3000.0, 1.0, 0.002, 1.0),
        );
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.14,
        0.0,
        &[5.3, 7.9, 10.7, 13.1],
        0.15,
        0.0,
        0.0,
    )
}

/// An exercise bike pedalled at 80 a minute: the flywheel's whirr against its brake swelling with each push
/// of a pedal (two a turn), the drive's hum under it and a pedal creaking once a turn.
fn bike(rate: f64) -> Stereo {
    let mut r = Rng::new(32.0);
    let seconds = 6.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let cadence = 80.0 / 60.0;
    let whirr = noise(n, rate, &mut r, Filter::Bandpass, 520.0, 0.8);
    let air = noise(n, rate, &mut r, Filter::Bandpass, 1500.0, 0.5);
    let drive = motor(
        n,
        rate,
        |i| 210.0 * (1.0 + 0.02 * sin((2.0 * PI * 2.0 * cadence * i as f64) / rate)),
        &[1.0, 0.4, 0.2],
    );
    for i in 0..n {
        let push = pow(sin((PI * 2.0 * cadence * i as f64) / rate), 2.0);
        let v = (whirr[i] as f64 * 0.8 + air[i] as f64 * 0.25) * (0.75 + 0.25 * push)
            + drive[i] as f64 * 0.012 * (0.8 + 0.2 * push);
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    // A pedal's bearing creaking at the bottom of each turn.
    let mut k = 0.0;
    while k < seconds * cadence {
        let at = k / cadence + 0.3 + (r.next() - 0.5) * 0.02;
        strike(
            &mut ev,
            at,
            0.06,
            0.1,
            &[[1750.0, 0.02, 1.0], [2830.0, 0.012, 0.5]],
            burst(2200.0, 3.0, 0.03, 0.4),
        );
        k += 1.0;
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.14,
        0.0,
        &[5.3, 7.9, 10.7, 13.1],
        0.15,
        0.0,
        0.0,
    )
}

/// A sewing machine: runs of stitching (the motor speeding up to about 13 stitches a second, its whirr, and
/// the needle bar and the hook clacking each stitch), pauses to turn the cloth, the odd snip of scissors.
fn sewing(rate: f64) -> Stereo {
    let mut r = Rng::new(33.0);
    let seconds = 10.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let runs = [[0.3, 2.6, 13.0], [3.3, 4.4, 9.0], [5.1, 8.3, 14.0]];
    // How fast it's going (stitches a second), with speeding up and slowing down.
    let mut speed = vec![0.0f32; n];
    for [a, b, top] in runs {
        let mut i = js_round(a * rate);
        while i < min(n as f64, b * rate + 0.2 * rate) {
            speed[i as usize] =
                (top * ramp(a, a + 0.25, i / rate) * (1.0 - ramp(b, b + 0.15, i / rate))) as f32;
            i += 1.0;
        }
    }
    let whirr = motor(
        n,
        rate,
        |i| 5.5 * speed[i] as f64,
        &[
            1.0, 0.8, 0.6, 0.5, 0.4, 0.35, 0.3, 0.25, 0.2, 0.18, 0.15, 0.12,
        ],
    );
    let brush = noise(n, rate, &mut r, Filter::Bandpass, 3200.0, 0.7);
    let mut lp = Biquad::with(rate, Filter::Lowpass, 2500.0, 0.7);
    for i in 0..n {
        let s = speed[i] as f64 / 14.0;
        let v = lp.run(whirr[i] as f64) * 0.05 * s + brush[i] as f64 * 0.12 * s;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    // Each stitch: the needle bar and hook, then the take-up lever half a turn on.
    let mut phase = 0.0;
    for (i, &sp) in speed.iter().enumerate() {
        let sp = sp as f64;
        let before = phase;
        phase += sp / rate;
        let at = i as f64 / rate;
        if phase.floor() > before.floor() {
            let s = sp / 14.0;
            strike(
                &mut ev,
                at,
                0.35 * s,
                0.0,
                &[
                    [2300.0, 0.006, 1.0],
                    [3700.0, 0.004, 0.6],
                    [180.0, 0.015, 0.4],
                ],
                burst(2600.0, 0.8, 0.002, 1.2),
            );
        }
        if (phase + 0.5).floor() > (before + 0.5).floor() {
            strike(
                &mut ev,
                at,
                0.15 * (sp / 14.0),
                0.0,
                &[[1600.0, 0.005, 1.0]],
                burst(1900.0, 1.0, 0.002, 0.8),
            );
        }
    }
    // The cloth turned and smoothed between runs, and a snip.
    for at in [2.9, 4.7] {
        rustle(&mut ev, &mut r, at, 0.3, 0.05, 0.1, 4000.0, 0.5);
    }
    strike(
        &mut ev,
        8.9,
        0.25,
        -0.1,
        &[[5200.0, 0.012, 1.0], [7400.0, 0.008, 0.6]],
        burst(6000.0, 0.8, 0.003, 1.5),
    );
    strike(
        &mut ev,
        8.96,
        0.15,
        -0.1,
        &[[4800.0, 0.01, 1.0]],
        burst(5500.0, 0.8, 0.002, 1.0),
    );
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.0,
        0.75,
        &[3.1, 5.3, 7.1, 9.7],
        0.15,
        0.0,
        0.0,
    )
}

/// A potter's wheel: the motor's quiet whine and the hum of its controller, the head turning about 150 a
/// minute, wet hands on the spinning clay (a swish that comes round with each turn and squelches when they
/// press), water squeezed from a sponge.
fn pottery_wheel(rate: f64) -> Stereo {
    let mut r = Rng::new(34.0);
    let seconds = 9.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let wobble = wander(&mut r, n, rate, 0.8);
    let press = wander(&mut r, n, rate, 0.6);
    let whine = motor(
        n,
        rate,
        |i| 640.0 * (1.0 + 0.003 * wobble[i] as f64),
        &[1.0, 0.3, 0.15],
    );
    let mains = motor(n, rate, |_| 100.0, &[1.0, 0.4, 0.2]);
    let swish = noise(n, rate, &mut r, Filter::Bandpass, 1100.0, 0.6);
    let squelch = noise(n, rate, &mut r, Filter::Lowpass, 450.0, 0.9);
    let rumble = noise(n, rate, &mut r, Filter::Lowpass, 110.0, 0.7);
    let turn = 2.5;
    for i in 0..n {
        let t = i as f64 / rate;
        let p = max(0.0, press[i] as f64);
        let round = 0.75 + 0.25 * sin(2.0 * PI * turn * t + 0.4 * wobble[i] as f64);
        let v = whine[i] as f64 * 0.004
            + mains[i] as f64 * 0.01
            + swish[i] as f64 * 0.55 * round * (0.6 + 0.4 * p)
            + squelch[i] as f64 * 0.9 * p * p * round
            + rumble[i] as f64 * 0.4;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    // The sponge squeezed over the clay: a trickle of drops.
    for at in [2.2, 6.4] {
        let mut t = at;
        while t < at + 0.7 {
            for c in 0..2 {
                let f = 700.0 + 1600.0 * r.next();
                let a = 0.04 * (0.3 + r.next());
                let rise = 0.2 + r.next() * 0.3;
                bubble(bed.ch(c), rate, t, f, a, rise);
            }
            t += -log(1.0 - r.next()) / 40.0;
        }
    }
    done(
        rate,
        vec![fold(bed)],
        0.13,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.15,
        0.0,
        0.0,
    )
}

/// A chemistry set: a flask boiling over a burner (small bubbles crowding up the glass, now and then a big
/// one), the burner's soft roar, a glass rod stirred against the beaker.
fn bubbling(rate: f64) -> Stereo {
    let mut r = Rng::new(35.0);
    let seconds = 8.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let roar = noise(n, rate, &mut r, Filter::Lowpass, 380.0, 0.7);
    let hiss = noise(n, rate, &mut r, Filter::Bandpass, 3200.0, 0.5);
    for i in 0..n {
        let v = roar[i] as f64 * 0.35 + hiss[i] as f64 * 0.06;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let surge = wander(&mut r, n, rate, 0.5);
    let mut t = r.next() * 0.01;
    while t < bed.seconds {
        let s = max(
            0.2,
            1.0 + 0.6 * surge[(n - 1).min(round_usize(t * rate))] as f64,
        );
        let (l, rr) = gains((r.next() - 0.5) * 0.3);
        let f = 1500.0 * pow(2.6, r.next());
        let a = 0.08 * s * (0.2 + pow(r.next(), 2.0));
        let rise = 0.3 + r.next() * 0.4;
        bubble(&mut bed.left, rate, t, f, a * l, rise);
        let rise = 0.3 + r.next() * 0.4;
        bubble(&mut bed.right, rate, t, f, a * rr, rise);
        t += -log(1.0 - r.next()) / 90.0;
    }
    let mut t = 0.6 + r.next();
    while t < bed.seconds {
        for c in 0..2 {
            let f = 520.0 + 400.0 * r.next();
            bubble(bed.ch(c), rate, t, f, 0.12, 0.4);
        }
        t += 0.8 + r.next() * 1.6;
    }
    let mut ev = looped(rate, seconds);
    for at in [3.1, 3.5, 3.85] {
        strike(
            &mut ev,
            at + r.next() * 0.05,
            0.12,
            0.2,
            &[
                [3150.0, 0.12, 1.0],
                [7300.0, 0.06, 0.5],
                [11900.0, 0.03, 0.3],
            ],
            burst(5000.0, 1.0, 0.001, 0.6),
        );
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.12,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.12,
        0.0,
        0.0,
    )
}

/// An espresso machine making a coffee: beans ground (the grinder's whine and the crunch of beans thinning
/// out), the grounds tamped and the handle locked in, the pump's buzz (a vibration pump: 50 Hz and its
/// harmonics) with the coffee trickling into the cup, the steam wand hissing into milk (a tearing hiss as it
/// froths, then a deeper roll as it heats), and the old grounds knocked out.
fn espresso(rate: f64) -> Stereo {
    let mut r = Rng::new(36.0);
    let seconds = 16.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let grinding = |t: f64| ramp(0.2, 0.4, t) * (1.0 - ramp(2.6, 2.9, t));
    let pumping = |t: f64| ramp(4.6, 4.7, t) * (1.0 - ramp(9.5, 9.6, t));
    let steaming = |t: f64| ramp(10.5, 10.55, t) * (1.0 - ramp(13.4, 13.5, t));
    let grinder = motor(
        n,
        rate,
        |i| 160.0 * (0.7 + 0.3 * grinding(i as f64 / rate)),
        &[
            1.0, 0.7, 0.6, 0.5, 0.45, 0.4, 0.3, 0.25, 0.2, 0.2, 0.15, 0.12,
        ],
    );
    let pump = motor(
        n,
        rate,
        |_| 50.0,
        &[
            1.0, 0.9, 0.8, 0.7, 0.5, 0.45, 0.4, 0.3, 0.25, 0.2, 0.15, 0.1, 0.08, 0.06,
        ],
    );
    let mut pump_lp = Biquad::with(rate, Filter::Lowpass, 1800.0, 0.7);
    let grit = noise(n, rate, &mut r, Filter::Bandpass, 3000.0, 0.5);
    let steam = noise(n, rate, &mut r, Filter::Highpass, 2500.0, 0.6);
    let roll = noise(n, rate, &mut r, Filter::Bandpass, 900.0, 0.6);
    let trickle = noise(n, rate, &mut r, Filter::Bandpass, 2300.0, 1.2);
    let flutter = wander(&mut r, n, rate, 0.015);
    for i in 0..n {
        let t = i as f64 / rate;
        let g = grinding(t);
        let p = pumping(t);
        let s = steaming(t);
        // The steam: a purge, then frothing (fluttering, tearing) for a second and a half, then rolling.
        let froth = ramp(10.9, 11.0, t) * (1.0 - ramp(12.2, 12.5, t));
        let tear = 1.0 + 0.6 * froth * flutter[i] as f64;
        // The pump labours as the pressure builds against the puck.
        let load = 0.8 + 0.2 * ramp(4.7, 5.6, t);
        let v = grinder[i] as f64 * 0.03 * g
            + grit[i] as f64 * 0.35 * g * (1.0 - 0.5 * ramp(0.4, 2.7, t))
            + pump_lp.run(pump[i] as f64) * 0.05 * p * load
            + steam[i] as f64 * 0.7 * s * tear * (1.0 - 0.4 * ramp(12.2, 12.5, t))
            + roll[i] as f64 * 0.5 * s * ramp(12.0, 12.6, t)
            + trickle[i] as f64 * 0.08 * p * ramp(5.6, 6.2, t);
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    // Beans crunching between the burrs, fewer as they're ground.
    let mut t = 0.3;
    while t < 2.7 {
        let amp = 0.05 * (0.3 + r.next());
        let pan = (r.next() - 0.5) * 0.2;
        let hz = 2000.0 + 3000.0 * r.next();
        strike(&mut ev, t, amp, pan, &[], burst(hz, 1.0, 0.0015, 1.0));
        t += -log(1.0 - r.next()) / (400.0 * (1.0 - ramp(0.4, 2.7, t)) + 40.0);
    }
    // Tamped, the handle locked in; knocked out at the end.
    strike(
        &mut ev,
        3.4,
        0.5,
        0.0,
        &[[210.0, 0.03, 1.0], [520.0, 0.015, 0.4]],
        burst(800.0, 0.7, 0.006, 0.8),
    );
    strike(
        &mut ev,
        4.05,
        0.35,
        0.0,
        &[
            [1700.0, 0.025, 1.0],
            [3100.0, 0.015, 0.6],
            [5200.0, 0.008, 0.3],
        ],
        burst(3000.0, 0.7, 0.003, 1.0),
    );
    for at in [14.2, 14.55] {
        strike(
            &mut ev,
            at,
            0.6,
            0.1,
            &[[160.0, 0.05, 1.0], [620.0, 0.02, 0.5], [1850.0, 0.012, 0.3]],
            burst(1000.0, 0.6, 0.008, 1.0),
        );
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.13,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.15,
        0.0,
        0.0,
    )
}

/// A blender: a universal motor at about 18,000 turns a minute (300 Hz and its many harmonics, with the
/// commutator's whine high above), its fan's rush, the fruit and ice churning in the jug, the speed dipping
/// as lumps hit the blades and knocking the glass.
fn blender(rate: f64) -> Stereo {
    let mut r = Rng::new(37.0);
    let seconds = 5.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let load = wander(&mut r, n, rate, 0.12);
    let churn = wander(&mut r, n, rate, 0.03);
    let speed =
        |i: usize| 300.0 * (1.0 - 0.04 * max(0.0, load[i] as f64) - 0.015 * churn[i] as f64);
    let hum = motor(
        n,
        rate,
        speed,
        &[
            1.0, 0.8, 0.7, 0.6, 0.5, 0.45, 0.4, 0.35, 0.3, 0.25, 0.22, 0.2, 0.18, 0.16, 0.14, 0.12,
        ],
    );
    let comm = motor(n, rate, |i| 12.0 * speed(i), &[1.0, 0.3]);
    let fan = noise(n, rate, &mut r, Filter::Bandpass, 2600.0, 0.45);
    // The brushes sparking on the commutator: a hiss above.
    let hiss = noise(n, rate, &mut r, Filter::Highpass, 5000.0, 0.7);
    let slosh = noise(n, rate, &mut r, Filter::Lowpass, 520.0, 0.8);
    for i in 0..n {
        let v = hum[i] as f64 * 0.05
            + comm[i] as f64 * 0.02
            + fan[i] as f64 * 1.1
            + hiss[i] as f64 * 0.25
            + slosh[i] as f64 * 0.45 * (1.0 + 0.6 * churn[i] as f64);
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    let mut t = r.next() * 0.2;
    while t < seconds {
        let amp = 0.12 * (0.3 + r.next());
        let pan = (r.next() - 0.5) * 0.2;
        let a = 2500.0 + 600.0 * r.next();
        let b = 4300.0 + 900.0 * r.next();
        strike(
            &mut ev,
            t,
            amp,
            pan,
            &[[a, 0.02, 1.0], [b, 0.01, 0.6]],
            burst(3500.0, 0.8, 0.002, 1.0),
        );
        t += 0.08 + r.next() * 0.5;
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.16,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.12,
        0.0,
        0.0,
    )
}

/// A stand mixer: a slower motor (about 160 Hz) with its gears' whine, and the beater slapping the batter
/// round the bowl.
fn mixer(rate: f64) -> Stereo {
    let mut r = Rng::new(38.0);
    let seconds = 6.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let wobble = wander(&mut r, n, rate, 0.3);
    let hum = motor(
        n,
        rate,
        |i| 160.0 * (1.0 + 0.01 * wobble[i] as f64),
        &[1.0, 0.7, 0.5, 0.4, 0.3, 0.25, 0.2, 0.15],
    );
    let gears = motor(
        n,
        rate,
        |i| 1440.0 * (1.0 + 0.01 * wobble[i] as f64),
        &[1.0, 0.3],
    );
    let air = noise(n, rate, &mut r, Filter::Bandpass, 1800.0, 0.5);
    let batter = noise(n, rate, &mut r, Filter::Lowpass, 700.0, 0.8);
    // The beater goes round the bowl about four times a second.
    for i in 0..n {
        let t = i as f64 / rate;
        let slap = pow(max(0.0, sin(2.0 * PI * 4.0 * t)), 6.0);
        let v = hum[i] as f64 * 0.04
            + gears[i] as f64 * 0.008
            + air[i] as f64 * 0.25
            + batter[i] as f64 * (0.15 + 0.8 * slap);
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    done(
        rate,
        vec![fold(bed)],
        0.12,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.15,
        0.0,
        0.0,
    )
}

/// A microwave cooking: the transformer's hum (twice the 50 Hz mains, 100 Hz, with its harmonics), the
/// cooling fan's rush and its blades, the turntable's motor, the odd spit from the food; all inside a metal
/// box.
fn microwave(rate: f64) -> Stereo {
    let mut r = Rng::new(39.0);
    let seconds = 8.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let wobble = wander(&mut r, n, rate, 1.2);
    let hum = motor(n, rate, |_| 100.0, &[1.0, 0.55, 0.4, 0.25, 0.15, 0.1]);
    let blades = motor(
        n,
        rate,
        |i| 245.0 * (1.0 + 0.005 * wobble[i] as f64),
        &[1.0, 0.3],
    );
    let fan = noise(n, rate, &mut r, Filter::Bandpass, 750.0, 0.5);
    for i in 0..n {
        let v = hum[i] as f64 * 0.07 + blades[i] as f64 * 0.012 + fan[i] as f64 * 0.45;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    for at in [1.3, 4.2, 4.35, 6.8] {
        let at = at + r.next() * 0.1;
        let hz = 3000.0 + 2000.0 * r.next();
        strike(&mut ev, at, 0.08, 0.0, &[], burst(hz, 0.9, 0.003, 1.0));
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.12,
        0.0,
        &[3.7, 6.1, 8.3, 10.9],
        0.15,
        3000.0,
        0.0,
    )
}

/// A toaster browning: the elements ticking as they heat, a faint buzz and the hiss of the heat.
fn toaster(rate: f64) -> Stereo {
    let mut r = Rng::new(40.0);
    let seconds = 5.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let buzz = motor(n, rate, |_| 100.0, &[1.0, 0.5, 0.3]);
    let hiss = noise(n, rate, &mut r, Filter::Highpass, 4000.0, 0.6);
    for i in 0..n {
        let v = buzz[i] as f64 * 0.02 + hiss[i] as f64 * 0.08;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    let mut t = r.next() * 0.3;
    while t < seconds {
        let amp = 0.2 * (0.3 + r.next());
        let pan = (r.next() - 0.5) * 0.2;
        let a = 2400.0 + 2600.0 * r.next();
        let b = 6100.0 + 1000.0 * r.next();
        strike(
            &mut ev,
            t,
            amp,
            pan,
            &[[a, 0.006, 1.0], [b, 0.003, 0.5]],
            burst(4000.0, 1.0, 0.001, 0.8),
        );
        t += 0.25 + r.next() * 0.7;
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.06,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.1,
        0.0,
        0.0,
    )
}

/// A fan oven: its motor's hum (50 Hz mains, 100 Hz), the fan at about 2,700 turns a minute with eight
/// blades (a tone near 360 Hz), the hot air's rush inside the oven, a tick of the metal now and then.
fn oven_fan(rate: f64) -> Stereo {
    let mut r = Rng::new(41.0);
    let seconds = 8.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let wobble = wander(&mut r, n, rate, 1.5);
    let hum = motor(n, rate, |_| 50.0, &[0.5, 1.0, 0.3, 0.3, 0.1]);
    let blades = motor(
        n,
        rate,
        |i| 360.0 * (1.0 + 0.004 * wobble[i] as f64),
        &[1.0, 0.35, 0.1],
    );
    let air = noise(n, rate, &mut r, Filter::Bandpass, 600.0, 0.5);
    for i in 0..n {
        let v = hum[i] as f64 * 0.05 + blades[i] as f64 * 0.03 + air[i] as f64 * 0.5;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    for at in [2.1, 5.7] {
        strike(
            &mut ev,
            at + r.next() * 0.2,
            0.15,
            0.0,
            &[[2900.0, 0.01, 1.0], [5300.0, 0.006, 0.5]],
            burst(3500.0, 1.0, 0.002, 1.0),
        );
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.1,
        0.0,
        &[3.7, 6.1, 8.3, 10.9],
        0.15,
        2600.0,
        0.0,
    )
}

/// A cooker hood drawing the steam off a hob: a centrifugal fan's broad rush, its blades' whine and the
/// motor's hum.
fn hood(rate: f64) -> Stereo {
    let mut r = Rng::new(42.0);
    let seconds = 8.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let wobble = wander(&mut r, n, rate, 0.8);
    let hum = motor(n, rate, |_| 100.0, &[1.0, 0.5, 0.3]);
    let blades = motor(
        n,
        rate,
        |i| 690.0 * (1.0 + 0.004 * wobble[i] as f64),
        &[1.0, 0.25],
    );
    let rush = noise(n, rate, &mut r, Filter::Bandpass, 520.0, 0.45);
    let high = noise(n, rate, &mut r, Filter::Bandpass, 1900.0, 0.6);
    for i in 0..n {
        let v = hum[i] as f64 * 0.03
            + blades[i] as f64 * 0.012
            + rush[i] as f64 * (0.7 + 0.04 * wobble[i] as f64)
            + high[i] as f64 * 0.15;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    done(
        rate,
        vec![fold(bed)],
        0.14,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.12,
        0.0,
        0.0,
    )
}

/// A wood fire: the flames' low roar flickering, crackles all the time (most tiny, some sharp pops as sap
/// bursts) and now and then the hiss of steam from damp wood.
fn fire(rate: f64) -> Stereo {
    let mut r = Rng::new(43.0);
    let seconds = 10.0;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let flicker = wander(&mut r, n, rate, 0.25);
    let slow = wander(&mut r, n, rate, 1.5);
    let roar = noise(n, rate, &mut r, Filter::Lowpass, 320.0, 0.7);
    let lick = noise(n, rate, &mut r, Filter::Bandpass, 850.0, 0.6);
    let steam = noise(n, rate, &mut r, Filter::Bandpass, 4200.0, 0.7);
    for i in 0..n {
        let t = i as f64 / rate;
        let s = ramp(5.8, 6.4, t) * (1.0 - ramp(7.2, 8.2, t));
        let v = roar[i] as f64 * (0.9 + 0.35 * flicker[i] as f64 + 0.2 * slow[i] as f64)
            + lick[i] as f64 * 0.12 * max(0.0, 1.0 + flicker[i] as f64)
            + steam[i] as f64 * 0.06 * s;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    let mut t = r.next() * 0.01;
    while t < seconds {
        let a = 0.25 * pow(r.next(), 3.0);
        let pan = (r.next() - 0.5) * 0.6;
        let hz = 1500.0 + 5000.0 * r.next();
        let tau = 0.0006 + 0.002 * r.next();
        strike(&mut ev, t, a, pan, &[], burst(hz, 1.0, tau, 1.5));
        t += -log(1.0 - r.next()) / 28.0;
    }
    let mut t = 0.5 + r.next();
    while t < seconds - 0.2 {
        let pan = (r.next() - 0.5) * 0.5;
        let amp = 0.5 + 0.4 * r.next();
        let low = 300.0 + 200.0 * r.next();
        let hz = 1200.0 + 1500.0 * r.next();
        strike(
            &mut ev,
            t,
            amp,
            pan,
            &[[low, 0.01, 0.6]],
            burst(hz, 0.7, 0.006, 1.5),
        );
        // A sharp pop often throws off a few small sparks.
        for _ in 0..3 {
            let at = t + 0.02 + r.next() * 0.15;
            let amp = 0.1 * r.next();
            let hz = 4000.0 + 3000.0 * r.next();
            strike(&mut ev, at, amp, pan, &[], burst(hz, 1.0, 0.0008, 1.5));
        }
        t += 1.0 + r.next() * 2.5;
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        0.14,
        0.0,
        &[7.1, 9.3, 11.7, 13.9],
        0.1,
        0.0,
        0.0,
    )
}

// The games.

/// Pool: a shot every few seconds. The cue's leather tip on the cue ball (a dull tock), the balls clacking
/// (phenolic resin: a click of a few milliseconds, most of it 3 to 6 kHz), a cushion's thump, the balls
/// rolling on the cloth, now and then one dropping into a pocket; a cue chalked.
fn pool(rate: f64) -> Stereo {
    let mut r = Rng::new(44.0);
    let seconds = 20.0;
    let mut ev = looped(rate, seconds);
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let rumble = noise(n, rate, &mut r, Filter::Bandpass, 220.0, 0.9);
    let mut rolling = vec![0.0f32; n];
    let shots = [0.6, 3.8, 7.1, 10.2, 13.6, 16.8];
    for (k, at) in shots.into_iter().enumerate() {
        let pan = (r.next() - 0.5) * 0.6;
        let power = 0.5 + 0.5 * r.next();
        // Chalking the cue before some shots.
        if k % 2 == 0 {
            for c in 0..3 {
                strike(
                    &mut ev,
                    at - 1.6 + c as f64 * 0.18,
                    0.04,
                    pan,
                    &[],
                    burst(3200.0, 1.5, 0.04, 1.0),
                );
            }
        }
        strike(
            &mut ev,
            at,
            0.4 * power,
            pan,
            &[[1100.0, 0.006, 1.0]],
            burst(1500.0, 0.6, 0.003, 1.2),
        );
        let hit = at + 0.18 + 0.3 * r.next() * (1.0 - power);
        ball(&mut ev, hit, 0.9 * power, pan);
        ball(&mut ev, hit + 0.2 + 0.3 * r.next(), 0.5 * power, -pan);
        if r.next() < 0.5 {
            ball(&mut ev, hit + 0.55 + 0.4 * r.next(), 0.3 * power, pan);
        }
        let cushion = hit + 0.4 + 0.5 * r.next();
        strike(
            &mut ev,
            cushion,
            0.3 * power,
            pan * 0.5,
            &[[260.0, 0.03, 1.0], [520.0, 0.015, 0.3]],
            burst(600.0, 0.7, 0.008, 1.0),
        );
        if k % 2 == 1 {
            // Into the pocket: a knock on the pocket's edge, the drop and a clack against the balls already in.
            let drop = cushion + 0.5 + 0.3 * r.next();
            strike(
                &mut ev,
                drop,
                0.4,
                pan,
                &[[320.0, 0.04, 1.0], [810.0, 0.02, 0.4]],
                burst(900.0, 0.6, 0.006, 1.0),
            );
            ball(&mut ev, drop + 0.18, 0.35, pan);
            ball(&mut ev, drop + 0.27, 0.15, pan);
        }
        let mut i = js_round(at * rate);
        while i < min(n as f64, (at + 3.5) * rate) {
            let t = i / rate - at;
            add(
                &mut rolling,
                i as usize,
                power * exp(-t / 1.1) * ramp(0.1, 0.3, t),
            );
            i += 1.0;
        }
    }
    for i in 0..n {
        bed.left[i] = (rumble[i] as f64 * 0.3 * rolling[i] as f64) as f32;
        bed.right[i] = (rumble[i] as f64 * 0.3 * rolling[i] as f64) as f32;
    }
    done(
        rate,
        vec![fold(bed), ev.into()],
        SPARSE,
        0.0,
        &[5.3, 7.9, 10.7, 13.1],
        0.15,
        0.0,
        0.0,
    )
}

/// Two pool balls clacking: a very short, hard click.
fn ball(out: &mut Ev, at: f64, amp: f64, pan: f64) {
    strike(
        out,
        at,
        amp,
        pan,
        &[[3800.0, 0.003, 0.6], [6200.0, 0.002, 0.4]],
        burst(4500.0, 0.5, 0.0015, 1.6),
    );
}

/// Foosball: the hard ball whacked by the plastic men and knocking round the wooden box, the rods slid and
/// spun, quick flurries between quieter build-ups, and a goal (the ball's clunk into the goal and its rumble
/// down the return).
fn foosball(rate: f64) -> Stereo {
    let mut r = Rng::new(45.0);
    let seconds = 12.0;
    let mut ev = looped(rate, seconds);
    let mut t = 0.2;
    while t < seconds - 1.0 {
        // A flurry: shots and passes, the ball off the walls, a rod slid.
        let hits = 3.0 + (r.next() * 6.0).floor();
        let mut k = 0.0;
        while k < hits && t < seconds - 1.0 {
            let pan = (r.next() - 0.5) * 0.7;
            let shot = r.next() < 0.3;
            let amp = if shot { 0.7 } else { 0.35 + 0.2 * r.next() };
            let f = 1800.0 + 300.0 * r.next();
            strike(
                &mut ev,
                t,
                amp,
                pan,
                &[[f, 0.01, 1.0], [3100.0, 0.006, 0.5], [190.0, 0.04, 0.5]],
                burst(2500.0, 0.7, 0.003, 1.4),
            );
            if r.next() < 0.5 {
                let at = t + 0.05 + 0.1 * r.next();
                strike(
                    &mut ev,
                    at,
                    0.3,
                    -pan,
                    &[[420.0, 0.02, 1.0], [1100.0, 0.01, 0.4]],
                    burst(1200.0, 0.7, 0.004, 1.0),
                );
            }
            if r.next() < 0.3 {
                rustle(&mut ev, &mut r, t + 0.02, 0.15, 0.06, pan, 4000.0, 0.5);
            }
            t += if shot {
                0.25 + 0.2 * r.next()
            } else {
                0.12 + 0.3 * r.next()
            };
            k += 1.0;
        }
        t += 0.6 + r.next() * 1.2;
    }
    // A goal: in, and down the return to the end.
    let goal = 7.4;
    strike(
        &mut ev,
        goal,
        0.8,
        0.4,
        &[[220.0, 0.05, 1.0], [560.0, 0.02, 0.5]],
        burst(900.0, 0.6, 0.008, 1.2),
    );
    for k in 0..12 {
        let f = 300.0 + 100.0 * r.next();
        strike(
            &mut ev,
            goal + 0.15 + k as f64 * 0.05,
            0.08,
            0.5,
            &[[f, 0.02, 1.0]],
            burst(600.0, 1.0, 0.004, 0.6),
        );
    }
    strike(
        &mut ev,
        goal + 0.85,
        0.4,
        0.6,
        &[[260.0, 0.04, 1.0], [700.0, 0.02, 0.4]],
        burst(800.0, 0.7, 0.005, 1.0),
    );
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

/// A planned moment of a rally (\`pingPongSound\`'s plan).
enum Play {
    /// The serve, by the server (-1 or 1).
    Serve { at: f64, server: f64 },
    /// A shot: the ball bouncing, then the bat a \`gap\` later at \`loud\`, from the side \`from\`.
    Shot {
        at: f64,
        gap: f64,
        loud: f64,
        from: f64,
    },
    /// The ball bouncing away on the floor.
    Floor { at: f64, level: f64, pan: f64 },
}

/// Ping-pong: rallies of the ball off the bats and the table, a hit then a bounce about a quarter of a
/// second later, back and forth; a rally ends with the ball bouncing away on the floor, a pause, and a serve.
fn ping_pong(rate: f64) -> Stereo {
    let mut r = Rng::new(46.0);
    // Two rallies, planned first: the loop is as long as they take.
    let mut plan = Vec::new();
    let mut t = 0.3;
    for _ in 0..2 {
        // The serve bounces on the server's side first.
        let mut side: f64 = if r.next() < 0.5 { -1.0 } else { 1.0 };
        plan.push(Play::Serve {
            at: t,
            server: side,
        });
        t += 0.45;
        let shots = 4.0 + (r.next() * 8.0).floor();
        let mut k = 0.0;
        while k < shots {
            let at = t;
            let gap = 0.2 + 0.12 * r.next();
            let loud = 0.5 + 0.3 * r.next();
            plan.push(Play::Shot {
                at,
                gap,
                loud,
                from: side,
            });
            side = -side;
            t += gap + 0.24 + 0.12 * r.next();
            k += 1.0;
        }
        // Missed: off the end and away across the floor, each bounce lower and sooner.
        let mut gap = 0.4;
        let mut a = 0.4;
        for _ in 0..7 {
            plan.push(Play::Floor {
                at: t,
                level: a,
                pan: side * 0.8,
            });
            t += gap;
            gap *= 0.78;
            a *= 0.75;
        }
        t += 1.5 + r.next();
    }
    let mut ev = looped(rate, t);
    for p in plan {
        match p {
            Play::Serve { at, server } => {
                bat(&mut ev, at, 0.5, server * 0.5);
                bounce(&mut ev, at + 0.18, 0.5, server * 0.25);
            }
            Play::Shot {
                at,
                gap,
                loud,
                from,
            } => {
                bounce(&mut ev, at, 0.6, -from * 0.25);
                bat(&mut ev, at + gap, loud, -from * 0.5);
            }
            Play::Floor { at, level, pan } => strike(
                &mut ev,
                at,
                level,
                pan,
                &[[2200.0, 0.006, 1.0], [5700.0, 0.003, 0.4]],
                burst(2000.0, 0.8, 0.002, 0.8),
            ),
        }
    }
    done(
        rate,
        vec![ev.into()],
        SPARSE,
        0.0,
        &[5.3, 7.9, 10.7, 13.1],
        0.18,
        0.0,
        0.0,
    )
}

/// A ping-pong ball off a bat: its shell's pock over the bat's wooden knock.
fn bat(out: &mut Ev, at: f64, amp: f64, pan: f64) {
    strike(
        out,
        at,
        amp,
        pan,
        &[
            [1500.0, 0.012, 1.0],
            [5700.0, 0.004, 0.3],
            [900.0, 0.008, 0.5],
        ],
        burst(1800.0, 0.8, 0.004, 1.0),
    );
}

/// A ping-pong ball bouncing on the table: the pock and the table top's thump under it.
fn bounce(out: &mut Ev, at: f64, amp: f64, pan: f64) {
    strike(
        out,
        at,
        amp,
        pan,
        &[
            [1500.0, 0.012, 1.0],
            [5700.0, 0.004, 0.3],
            [240.0, 0.035, 0.6],
            [520.0, 0.02, 0.3],
        ],
        burst(1600.0, 0.8, 0.003, 0.8),
    );
}

/// Darts: three thrown about a second apart, each a dull thud into the bristle board (with the board's
/// backing knocking on the wall), then a walk to the board and the three pulled out; now and then one hits
/// the wire and falls to the floor.
fn darts(rate: f64) -> Stereo {
    let mut r = Rng::new(47.0);
    let seconds = 12.0;
    let mut ev = looped(rate, seconds);
    for (round, start) in [0.4, 6.4].into_iter().enumerate() {
        for k in 0..3 {
            let at = start + k as f64 * (1.0 + 0.25 * r.next());
            if round == 1 && k == 1 {
                // Off the wire, and clattering on the floor.
                strike(
                    &mut ev,
                    at,
                    0.25,
                    0.1,
                    &[[3800.0, 0.012, 1.0], [6100.0, 0.008, 0.5]],
                    burst(4000.0, 0.8, 0.002, 1.0),
                );
                strike(
                    &mut ev,
                    at + 0.42,
                    0.2,
                    0.2,
                    &[[2600.0, 0.01, 1.0], [4100.0, 0.006, 0.6]],
                    burst(3000.0, 0.8, 0.002, 1.0),
                );
                strike(
                    &mut ev,
                    at + 0.55,
                    0.08,
                    0.25,
                    &[[2900.0, 0.008, 1.0]],
                    burst(3000.0, 0.8, 0.002, 1.0),
                );
                continue;
            }
            let amp = 0.6 + 0.2 * r.next();
            strike(
                &mut ev,
                at,
                amp,
                0.05 * (k as f64 - 1.0),
                &[[140.0, 0.05, 0.6], [320.0, 0.02, 0.3]],
                burst(950.0, 0.6, 0.008, 1.5),
            );
        }
        // Pulled out of the board: a short scratch each, and the darts knocking together in the hand.
        for k in 0..3 {
            strike(
                &mut ev,
                start + 4.1 + k as f64 * 0.4 + r.next() * 0.1,
                0.18,
                0.0,
                &[],
                burst(2600.0, 2.0, 0.03, 1.5),
            );
        }
        strike(
            &mut ev,
            start + 5.4,
            0.1,
            0.0,
            &[[4400.0, 0.03, 1.0], [7100.0, 0.02, 0.5]],
            burst(5000.0, 1.0, 0.001, 0.5),
        );
    }
    done(
        rate,
        vec![ev.into()],
        SPARSE,
        0.0,
        &[5.3, 7.9, 10.7, 13.1],
        0.15,
        0.0,
        0.0,
    )
}

/// An arcade cabinet: a chiptune (square-wave lead and a triangle bass in A minor, a noise hi-hat), the
/// game's own zaps, booms and a bonus jingle, and the player's buttons and joystick clicking; through the
/// cabinet's small speaker.
fn arcade(rate: f64) -> Stereo {
    let mut r = Rng::new(48.0);
    let beat = 60.0 / 140.0;
    let seconds = 16.0 * beat;
    let mut out = looped(rate, seconds);
    let mut music = vec![0.0f32; out.left.len()];
    let note = |midi: f64| 440.0 * pow(2.0, (midi - 69.0) / 12.0);
    // The lead: eighth notes walking the minor pentatonic, a rest now and then.
    let scale = [57.0, 60.0, 62.0, 64.0, 67.0, 69.0, 72.0, 74.0, 76.0];
    let mut at = 3.0;
    for e in 0..32 {
        at = max(
            0.0,
            min(
                scale.len() as f64 - 1.0,
                at + js_round((r.next() - 0.5) * 3.0),
            ),
        );
        if r.next() < 0.12 {
            continue;
        }
        chip(
            &mut music,
            rate,
            e as f64 * (beat / 2.0),
            beat / 2.0 - 0.02,
            note(scale[at as usize] + 12.0),
            0.25,
            0.22,
        );
    }
    // The bass: root and fifth, a quarter note each, round A, F, C, G.
    for b in 0..16 {
        let root = [45.0, 41.0, 48.0, 43.0][b / 4];
        tri(
            &mut music,
            rate,
            b as f64 * beat,
            beat * 0.9,
            note(root + (if b % 2 == 1 { 7.0 } else { 0.0 }) - 12.0),
            0.35,
        );
    }
    for e in 0..32 {
        if e % 2 == 1 {
            strike(
                &mut out,
                e as f64 * (beat / 2.0),
                0.08,
                0.0,
                &[],
                burst(7000.0, 0.7, 0.01, 1.5),
            );
        }
    }
    for (i, &m) in music.iter().enumerate() {
        add(&mut out.left, i, m as f64);
        add(&mut out.right, i, m as f64);
    }
    // The game: zaps falling in pitch, a boom, the bonus jingle.
    let mut t = 0.3;
    while t < seconds - 0.3 {
        zap(&mut out, t, 1800.0 + 600.0 * r.next(), 0.12);
        t += 0.35 + r.next() * 0.9;
    }
    for t in [2.4, 5.1] {
        strike(
            &mut out,
            t,
            0.6,
            0.0,
            &[[70.0, 0.12, 1.0]],
            burst(300.0, 0.5, 0.12, 2.0),
        );
    }
    chip(&mut out.left, rate, 4.2, 0.07, 988.0, 0.5, 0.2);
    chip(&mut out.left, rate, 4.27, 0.25, 1319.0, 0.5, 0.2);
    chip(&mut out.right, rate, 4.2, 0.07, 988.0, 0.5, 0.2);
    chip(&mut out.right, rate, 4.27, 0.25, 1319.0, 0.5, 0.2);
    // Buttons (microswitches) in bursts, and the joystick's knocks.
    let mut t = 0.1;
    while t < seconds - 0.1 {
        strike(
            &mut out,
            t,
            0.1,
            0.15,
            &[[4200.0, 0.003, 1.0], [6800.0, 0.002, 0.6]],
            burst(3000.0, 0.8, 0.001, 1.0),
        );
        if r.next() < 0.4 {
            strike(
                &mut out,
                t + 0.05,
                0.06,
                -0.1,
                &[[900.0, 0.006, 1.0], [2300.0, 0.004, 0.5]],
                burst(1500.0, 0.8, 0.002, 1.0),
            );
        }
        t += 0.12 + r.next() * 0.5;
    }
    done(
        rate,
        vec![out.into()],
        0.12,
        0.0,
        &[3.1, 5.3, 7.1, 9.7],
        0.1,
        5000.0,
        180.0,
    )
}

/// A square-ish note (a pulse of \`duty\`, its harmonics added up to 8 kHz), with a short attack and release.
fn chip(d: &mut [f32], rate: f64, at: f64, seconds: f64, hz: f64, duty: f64, amp: f64) {
    let start = round_usize(at * rate);
    let len = round_usize(seconds * rate);
    let top = (8000.0 / hz).floor() as usize;
    let k: Vec<f64> = (0..top)
        .map(|i| (2.0 / ((i + 1) as f64 * PI)) * sin((i + 1) as f64 * PI * duty))
        .collect();
    for j in 0..len {
        let i = (start + j) % d.len();
        let t = j as f64 / rate;
        let mut v = 0.0;
        for (h, kh) in k.iter().enumerate() {
            v += kh * sin(2.0 * PI * hz * (h + 1) as f64 * t);
        }
        add(
            d,
            i,
            v * amp
                * min(
                    min(1.0, j as f64 / (0.003 * rate)),
                    (len - j) as f64 / (0.01 * rate),
                ),
        );
    }
}

/// A triangle-wave note (the chip's bass).
fn tri(d: &mut [f32], rate: f64, at: f64, seconds: f64, hz: f64, amp: f64) {
    let start = round_usize(at * rate);
    let len = round_usize(seconds * rate);
    for j in 0..len {
        let x = (hz * j as f64) / rate;
        let v = 4.0 * (x - (x + 0.5).floor()).abs() - 1.0;
        let i = (start + j) % d.len();
        add(
            d,
            i,
            v * amp
                * min(
                    min(1.0, j as f64 / (0.003 * rate)),
                    (len - j) as f64 / (0.01 * rate),
                ),
        );
    }
}

/// A game's zap: a square wave falling fast in pitch.
fn zap(out: &mut Ev, at: f64, from: f64, amp: f64) {
    let rate = out.rate;
    let start = round_usize(at * rate);
    let len = round_usize(0.14 * rate);
    let mut phase = 0.0;
    for j in 0..len {
        let t = j as f64 / rate;
        phase += (from * exp(-t / 0.05)) / rate;
        let v =
            (sin(2.0 * PI * phase) + sin(6.0 * PI * phase) / 3.0 + sin(10.0 * PI * phase) / 5.0)
                * amp
                * (1.0 - j as f64 / len as f64);
        let i = (start + j) % out.left.len();
        add(&mut out.left, i, v);
        add(&mut out.right, i, v);
    }
}

/// Basketball on an outdoor court: dribbling (the ball's slap on the court and the ring of the air inside
/// it, about 950 and 1500 Hz for a ball 24 cm across), a shot that goes in with a swish of the net or clangs
/// off the rim and the board, and the ball bouncing away under the hoop, each bounce lower and sooner (it
/// keeps about 80 % of its speed).
fn basketball(rate: f64) -> Stereo {
    let mut r = Rng::new(49.0);
    // Two shots, the second after more dribbling, and the ball picked up again as the loop comes round.
    let seconds = 21.5;
    let mut ev = looped(rate, seconds);
    let mut t = 0.2;
    for (k, made) in [true, false].into_iter().enumerate() {
        let dribbles = 6 + k * 2;
        for _ in 0..dribbles {
            let amp = 0.7 + 0.15 * r.next();
            let pan = (r.next() - 0.5) * 0.2;
            dribble(&mut ev, t, amp, pan);
            t += 0.5 + 0.04 * r.next();
        }
        // The shot: up for about a second.
        t += 1.1;
        if made {
            for j in 0..2 {
                strike(
                    &mut ev,
                    t + j as f64 * 0.04,
                    0.25,
                    0.0,
                    &[],
                    burst(4200.0, 0.6, 0.06, 1.0),
                );
            }
        } else {
            strike(
                &mut ev,
                t,
                0.5,
                0.05,
                &[
                    [420.0, 0.22, 1.0],
                    [1100.0, 0.16, 0.8],
                    [1960.0, 0.1, 0.6],
                    [3100.0, 0.07, 0.4],
                ],
                burst(1500.0, 0.6, 0.004, 1.0),
            );
            strike(
                &mut ev,
                t + 0.14,
                0.4,
                -0.05,
                &[[180.0, 0.05, 1.0], [430.0, 0.02, 0.4]],
                burst(700.0, 0.6, 0.006, 1.0),
            );
        }
        // Down from the hoop (3 m: about 0.8 s), then bouncing away.
        t += 0.78;
        let mut gap = 2.0 * 0.78 * 0.8;
        let mut a = 1.0;
        for _ in 0..6 {
            dribble(&mut ev, t, a, 0.3);
            t += gap;
            a *= 0.8;
            gap *= 0.8;
        }
        // Picked up again.
        t += 0.4;
    }
    done(rate, vec![ev.into()], SPARSE, 0.0, &[], 0.0, 0.0, 0.0)
}

/// A basketball hitting the court: the slap of its skin and the ring of the air inside.
fn dribble(out: &mut Ev, at: f64, amp: f64, pan: f64) {
    strike(
        out,
        at,
        amp,
        pan,
        &[[110.0, 0.03, 0.8], [950.0, 0.05, 0.5], [1520.0, 0.03, 0.3]],
        burst(1500.0, 0.6, 0.003, 0.8),
    );
}

// The rest.

/// A workbench: a nail hammered in (the steel's ring rising as it goes in, then the dull last blows on the
/// wood), a board sawn in strokes (the teeth catching, the push louder than the pull), the cut-off dropping,
/// sanding, a screwdriver's ratchet, a tool put down.
fn tools(rate: f64) -> Stereo {
    let mut r = Rng::new(50.0);
    let seconds = 20.0;
    let mut ev = looped(rate, seconds);
    for k in 0..6 {
        let at = 0.4 + k as f64 * 0.42;
        let rise = 1.0 + 0.03 * k as f64;
        if k < 5 {
            strike(
                &mut ev,
                at,
                0.6,
                0.1,
                &[
                    [2600.0 * rise, 0.05, 0.6],
                    [4100.0 * rise, 0.03, 0.4],
                    [6900.0 * rise, 0.02, 0.2],
                    [180.0, 0.04, 0.8],
                    [420.0, 0.03, 0.4],
                ],
                burst(2000.0, 0.7, 0.004, 1.2),
            );
        } else {
            strike(
                &mut ev,
                at,
                0.8,
                0.1,
                &[[180.0, 0.05, 1.0], [420.0, 0.03, 0.5], [900.0, 0.015, 0.3]],
                burst(1200.0, 0.7, 0.006, 1.2),
            );
        }
    }
    strike(
        &mut ev,
        3.6,
        0.35,
        0.2,
        &[[250.0, 0.03, 1.0], [1700.0, 0.03, 0.4]],
        burst(900.0, 0.7, 0.005, 1.0),
    );
    // Sawing: eight strokes.
    let mut t = 4.6;
    for k in 0..8 {
        let push = k % 2 == 0;
        let len = if push { 0.55 } else { 0.45 };
        saw(&mut ev, &mut r, t, len, if push { 0.5 } else { 0.3 }, -0.1);
        t += len + 0.04;
    }
    strike(
        &mut ev,
        t + 0.1,
        0.6,
        -0.2,
        &[[150.0, 0.06, 1.0], [380.0, 0.03, 0.4], [820.0, 0.015, 0.2]],
        burst(700.0, 0.6, 0.008, 1.0),
    );
    // Sanding.
    t += 1.5;
    for _ in 0..10 {
        rustle(&mut ev, &mut r, t, 0.35, 0.3, 0.0, 1900.0, 0.5);
        t += 0.4;
    }
    // A screwdriver's ratchet: three turns.
    t += 0.8;
    for _ in 0..3 {
        for c in 0..8 {
            strike(
                &mut ev,
                t + c as f64 * 0.07,
                0.12,
                0.15,
                &[[3500.0, 0.006, 1.0], [6100.0, 0.004, 0.5]],
                burst(4000.0, 1.0, 0.001, 0.8),
            );
        }
        t += 0.9;
    }
    strike(
        &mut ev,
        t + 0.3,
        0.3,
        0.1,
        &[[2100.0, 0.06, 0.5], [3700.0, 0.04, 0.4], [300.0, 0.02, 0.6]],
        burst(2000.0, 0.7, 0.003, 1.0),
    );
    done(
        rate,
        vec![ev.into()],
        SPARSE,
        0.0,
        &[7.1, 9.3, 11.7, 13.9],
        0.18,
        0.0,
        0.0,
    )
}

/// A hand saw's stroke: the teeth catching about 70 times a second, in a high band, swelling and fading with
/// the stroke.
fn saw(out: &mut Ev, r: &mut Rng, at: f64, seconds: f64, amp: f64, pan: f64) {
    let rate = out.rate;
    let start = js_round(at * rate);
    let len = js_round(seconds * rate);
    let (l, rr) = gains(pan);
    let mut teeth = Biquad::with(rate, Filter::Bandpass, 3600.0, 0.9);
    let mut board = Biquad::with(rate, Filter::Bandpass, 420.0, 1.5);
    let mut j = 0.0;
    while j < len {
        let t = j / rate;
        let x = r.next() * 2.0 - 1.0;
        let env = pow(sin((PI * j) / len), 0.7);
        let catchy = 0.5 + 0.5 * sin(PI * 70.0 * t).abs();
        let v = (teeth.run(x) * catchy * 1.5 + board.run(x) * 0.6) * env * amp;
        put(out, start + j, v, l, rr);
        j += 1.0;
    }
}

/// A bike looked after on its stand: a wheel spun and its freewheel ticking slower and slower, a spanner's
/// ratchet, the gears changed, the tyres pumped up.
fn bike_tools(rate: f64) -> Stereo {
    let mut r = Rng::new(51.0);
    let seconds = 14.0;
    let mut ev = looped(rate, seconds);
    // The freewheel: pawls ticking from about 28 a second down to a few as the wheel slows.
    for start in [0.3, 7.5] {
        let mut t = start;
        while t < start + 3.5 {
            let rate_now = 28.0 * exp(-(t - start) / 1.6);
            strike(
                &mut ev,
                t,
                0.1,
                -0.2,
                &[[5200.0, 0.004, 1.0], [7800.0, 0.003, 0.5]],
                burst(5000.0, 1.0, 0.001, 0.6),
            );
            t += 1.0 / max(3.0, rate_now);
        }
    }
    // A spanner's ratchet: three swings.
    for k in 0..3 {
        for c in 0..7 {
            strike(
                &mut ev,
                4.2 + k as f64 * 0.8 + c as f64 * 0.07,
                0.15,
                0.2,
                &[[3400.0, 0.006, 1.0], [5900.0, 0.004, 0.5]],
                burst(3800.0, 1.0, 0.001, 0.8),
            );
        }
    }
    // The gears changed: the derailleur's clack.
    strike(
        &mut ev,
        6.8,
        0.3,
        -0.1,
        &[[1200.0, 0.03, 1.0], [2900.0, 0.02, 0.6]],
        burst(2000.0, 0.7, 0.003, 1.0),
    );
    // The tyre pumped: four strokes of rising hiss.
    for k in 0..4 {
        let at = 11.4 + k as f64 * 0.6;
        let start = js_round(at * rate);
        let len = js_round(0.4 * rate);
        let mut bp = Biquad::new(rate);
        let mut j = 0.0;
        while j < len {
            if (j as u64) & 31 == 0 {
                bp.set(Filter::Bandpass, 1200.0 + 800.0 * (j / len), 1.2);
            }
            let v = bp.run(r.next() * 2.0 - 1.0) * 0.4 * sin((PI * j) / len);
            put(&mut ev, start + j, v, 0.7, 0.7);
            j += 1.0;
        }
    }
    done(
        rate,
        vec![ev.into()],
        SPARSE,
        0.0,
        &[5.3, 7.9, 10.7, 13.1],
        0.12,
        0.0,
        0.0,
    )
}

/// A punch bag worked over: combinations of punches (each the glove's slap on the leather and the thump of
/// the heavy bag, about 85 Hz), the chain jingling above as it swings, the mount creaking, a sharp breath out
/// with each punch.
fn punch_bag(rate: f64) -> Stereo {
    let mut r = Rng::new(52.0);
    let seconds = 12.0;
    let mut ev = looped(rate, seconds);
    let combos: [&[f64]; 6] = [
        &[0.4, 0.65, 0.95],
        &[2.7, 3.05],
        &[4.6, 4.8, 5.05, 5.4],
        &[7.3],
        &[8.6, 8.9, 9.2],
        &[10.9, 11.1],
    ];
    for combo in combos {
        for (k, &at) in combo.iter().enumerate() {
            let power = if k == combo.len() - 1 {
                1.0
            } else {
                0.7 + 0.15 * r.next()
            };
            let pan = (r.next() - 0.5) * 0.2;
            strike(
                &mut ev,
                at,
                power,
                pan,
                &[[85.0, 0.07, 1.0], [140.0, 0.04, 0.5]],
                burst(1300.0, 0.7, 0.006, 1.6),
            );
            strike(
                &mut ev,
                at - 0.02,
                0.12 * power,
                pan,
                &[],
                burst(2600.0, 0.8, 0.04, 1.0),
            );
            for c in 0..4 {
                let when = at + 0.04 + 0.03 * c as f64 + 0.02 * r.next();
                let a = 3000.0 + 3000.0 * r.next();
                let b = 7000.0 + 2000.0 * r.next();
                strike(
                    &mut ev,
                    when,
                    0.05 * power,
                    0.1,
                    &[[a, 0.05, 1.0], [b, 0.03, 0.4]],
                    burst(5000.0, 1.0, 0.001, 0.4),
                );
            }
        }
        // Swinging back: the mount creaks.
        let last = combo[combo.len() - 1];
        strike(
            &mut ev,
            last + 0.55,
            0.08,
            0.1,
            &[[600.0, 0.05, 1.0]],
            burst(900.0, 4.0, 0.05, 1.0),
        );
    }
    done(
        rate,
        vec![ev.into()],
        0.0,
        0.75,
        &[5.3, 7.9, 10.7, 13.1],
        0.15,
        0.0,
        0.0,
    )
}

/// Plates and glasses smashed in a metal bin: each the crash of the impact (the bin's body ringing under it),
/// then the shards: dozens of small pieces clinking down, the bigger ones landing last, then a tinkle as they
/// settle. Glass rings higher and longer than china.
fn smash(rate: f64) -> Stereo {
    let mut r = Rng::new(53.0);
    let seconds = 12.0;
    let mut ev = looped(rate, seconds);
    for (k, at) in [0.3, 3.4, 6.5, 9.2].into_iter().enumerate() {
        let glass = k % 2 == 1;
        let pan = (r.next() - 0.5) * 0.3;
        strike(
            &mut ev,
            at,
            0.8,
            pan,
            &[
                [310.0, 0.3, 0.4],
                [740.0, 0.25, 0.35],
                [1350.0, 0.18, 0.3],
                [2200.0, 0.12, 0.2],
                [90.0, 0.05, 0.8],
            ],
            burst(3000.0, 0.4, 0.012, 2.4),
        );
        let pieces = 40.0 + (r.next() * 30.0).floor();
        let mut p = 0.0;
        while p < pieces {
            let dt = -log(1.0 - r.next()) * 0.09;
            let f = if glass {
                3000.0 + 6000.0 * r.next()
            } else {
                2000.0 + 4000.0 * r.next()
            };
            let ring = if glass {
                0.02 + 0.06 * r.next()
            } else {
                0.01 + 0.03 * r.next()
            };
            let amp = 0.25 * exp(-dt / 0.25) * (0.3 + r.next());
            let to = pan + (r.next() - 0.5) * 0.4;
            strike(
                &mut ev,
                at + 0.01 + dt,
                amp,
                to,
                &[[f, ring, 1.0], [f * 2.3, ring * 0.6, 0.5]],
                burst(f, 1.0, 0.001, 0.6),
            );
            p += 1.0;
        }
        for _ in 0..4 {
            let dt = 0.25 + 0.4 * r.next();
            let f = 1500.0 + 1500.0 * r.next();
            strike(
                &mut ev,
                at + dt,
                0.25,
                pan,
                &[[f, 0.04, 1.0], [4200.0, 0.02, 0.4]],
                burst(2000.0, 0.8, 0.002, 1.0),
            );
        }
    }
    done(
        rate,
        vec![ev.into()],
        0.0,
        0.75,
        &[5.3, 7.9, 10.7, 13.1],
        0.15,
        0.0,
        0.0,
    )
}

/// A cello played with a bow, a slow lament in D minor: each note a bowed string (a sawtooth, with the bow's
/// scratch as it bites and its hiss along the string), vibrato coming in as it's held, through the body's
/// resonances (the air near 100 Hz, the wood near 190 and 290 Hz, the bridge near 2.2 kHz); in a living room.
fn cello(rate: f64) -> Stereo {
    let mut r = Rng::new(54.0);
    let notes = [
        [50.0, 2.0],
        [53.0, 1.0],
        [52.0, 1.0],
        [50.0, 1.5],
        [48.0, 0.5],
        [45.0, 2.0],
        [46.0, 1.5],
        [45.0, 0.5],
        [43.0, 1.0],
        [45.0, 1.0],
        [38.0, 3.0],
    ];
    let seconds = notes.iter().fold(0.0, |s, [_, d]| s + d);
    let mut ev = looped(rate, seconds);
    let n = ev.left.len();
    let mut dry = vec![0.0f32; n];
    let mut at = 0.0;
    for (k, [midi, dur]) in notes.into_iter().enumerate() {
        let hz = 440.0 * pow(2.0, (midi - 69.0) / 12.0);
        let start = round_usize(at * rate);
        let len = round_usize((dur + 0.15) * rate);
        let top = min(40.0, (6000.0 / hz).floor()) as usize;
        let weights: Vec<f64> = (1..=top).map(|h| 1.0 / h as f64).collect();
        let mut scratch = Biquad::with(rate, Filter::Highpass, 1500.0, 0.7);
        let swell = if k % 2 == 1 { 0.85 } else { 1.0 };
        let mut phase = r.next();
        for j in 0..len {
            let t = j as f64 / rate;
            // Vibrato: about 5.5 a second, a sixth of a semitone, coming in after a third of a second.
            let vib = 0.01 * ramp(0.3, 0.7, t) * sin(2.0 * PI * 5.5 * t);
            phase += (hz * (1.0 + vib + 0.0008 * (r.next() - 0.5))) / rate;
            let v = harmonics(phase, &weights);
            // The bow: biting in, a swell along the stroke, lifting off.
            let env = ramp(0.0, 0.1, t)
                * (1.0 - ramp(dur, dur + 0.15, t))
                * (0.75 + 0.25 * sin((PI * min(t, dur)) / dur))
                * swell;
            let bite = exp(-t / 0.05);
            let x = v * env * 0.3 + scratch.run(r.next() * 2.0 - 1.0) * (0.02 + 0.12 * bite) * env;
            add(&mut dry, (start + j) % n, x);
        }
        at += dur;
    }
    // The body.
    let body = [
        [100.0, 4.0, 0.8],
        [190.0, 5.0, 1.0],
        [290.0, 4.0, 0.8],
        [450.0, 3.0, 0.5],
        [600.0, 3.0, 0.4],
        [1100.0, 2.0, 0.5],
        [2200.0, 1.5, 0.7],
    ];
    let mut filters: Vec<Biquad> = body
        .iter()
        .map(|&[hz, q, _]| Biquad::with(rate, Filter::Bandpass, hz, q))
        .collect();
    // Twice round, so the resonances have settled at the loop's start.
    for pass in 0..2 {
        for i in 0..n {
            let x = dry[i] as f64;
            let mut v = x * 0.5;
            for (f, b) in filters.iter_mut().zip(&body) {
                v += f.run(x) * b[2];
            }
            if pass == 1 {
                ev.left[i] = v as f32;
                ev.right[i] = v as f32;
            }
        }
    }
    done(
        rate,
        vec![ev.into()],
        0.12,
        0.0,
        &[17.3, 23.9, 29.1, 34.7],
        0.3,
        0.0,
        0.0,
    )
}

/// Someone singing to a karaoke track: a pop backing (pad chords round C, G, A minor and F, a bass, a soft
/// kick, snare and hi-hat at 120 bpm) and over it a voice (a glottal buzz through the formants of the vowel
/// being sung, vibrato on the held notes, the consonants' hiss), a little echo on the microphone.
fn singing(rate: f64) -> Stereo {
    let mut r = Rng::new(55.0);
    let beat = 0.5;
    let seconds = 16.0 * beat;
    let mut out = looped(rate, seconds);
    let n = out.left.len();
    let mut backing = vec![0.0f32; n];
    let mut voice = vec![0.0f32; n];
    let hz = |midi: f64| 440.0 * pow(2.0, (midi - 69.0) / 12.0);
    let chords = [
        [48.0, 52.0, 55.0],
        [43.0, 47.0, 50.0],
        [45.0, 48.0, 52.0],
        [41.0, 45.0, 48.0],
    ];
    for (bar, chord) in chords.iter().enumerate() {
        let bar = bar as f64;
        for &m in chord {
            pad(
                &mut backing,
                rate,
                bar * 4.0 * beat,
                4.0 * beat,
                hz(m + 12.0),
                0.06,
                6,
            );
        }
        for b in 0..4 {
            pad(
                &mut backing,
                rate,
                (bar * 4.0 + b as f64) * beat,
                beat * 0.9,
                hz(chord[0] - 12.0),
                0.12,
                3,
            );
        }
    }
    for b in 0..16 {
        let t = b as f64 * beat;
        if b % 2 == 0 {
            strike(
                &mut out,
                t,
                0.5,
                0.0,
                &[[60.0, 0.08, 1.0]],
                burst(200.0, 0.7, 0.01, 0.6),
            );
        } else {
            strike(
                &mut out,
                t,
                0.25,
                0.0,
                &[[190.0, 0.04, 0.6]],
                burst(2500.0, 0.6, 0.06, 1.2),
            );
        }
        strike(
            &mut out,
            t + beat / 2.0,
            0.05,
            0.1,
            &[],
            burst(8000.0, 0.7, 0.02, 1.5),
        );
    }
    // The tune: (midi, beats, vowel), a rest as 0.
    let tune = [
        (64.0, 1.0, 'a'),
        (64.0, 0.5, 'e'),
        (67.0, 0.5, 'i'),
        (69.0, 1.5, 'o'),
        (0.0, 0.5, 'a'),
        (67.0, 1.0, 'a'),
        (66.0, 0.5, 'e'),
        (67.0, 0.5, 'u'),
        (62.0, 2.0, 'o'),
        (64.0, 1.0, 'e'),
        (64.0, 0.5, 'a'),
        (67.0, 0.5, 'i'),
        (72.0, 1.5, 'a'),
        (71.0, 0.5, 'e'),
        (69.0, 2.0, 'o'),
        (0.0, 2.0, 'a'),
    ];
    let mut at = 0.0;
    for (midi, beats, vowel) in tune {
        if midi > 0.0 {
            sing(&mut voice, rate, &mut r, at, beats * beat, hz(midi), vowel);
        }
        at += beats * beat;
    }
    for i in 0..n {
        let v = backing[i] as f64 + voice[i] as f64;
        add(&mut out.left, i, v);
        add(&mut out.right, i, v);
    }
    done(
        rate,
        vec![out.into()],
        0.13,
        0.0,
        &[23.1, 29.3, 37.9, 41.3],
        0.25,
        7000.0,
        90.0,
    )
}

/// The first three formants of a vowel (Hz), a sung voice's.
fn formants(vowel: char) -> [f64; 3] {
    match vowel {
        'a' => [800.0, 1150.0, 2900.0],
        'e' => [480.0, 1720.0, 2520.0],
        'i' => [300.0, 2200.0, 3000.0],
        'o' => [450.0, 800.0, 2830.0],
        _ => [325.0, 700.0, 2530.0],
    }
}

/// A sung note: a glottal buzz (harmonics falling 9 dB an octave) through the vowel's formants, with a
/// consonant's hiss at the start and vibrato as it's held.
#[allow(clippy::too_many_arguments)]
fn sing(d: &mut [f32], rate: f64, r: &mut Rng, at: f64, seconds: f64, hz: f64, vowel: char) {
    let start = round_usize(at * rate);
    let len = round_usize(seconds * rate);
    let [f1, f2, f3] = formants(vowel);
    let mut filters = [
        Biquad::with(rate, Filter::Bandpass, f1, f1 / 90.0),
        Biquad::with(rate, Filter::Bandpass, f2, f2 / 110.0),
        Biquad::with(rate, Filter::Bandpass, f3, f3 / 160.0),
    ];
    let weights = [1.0, 0.9, 0.6];
    let mut hiss = Biquad::with(rate, Filter::Highpass, 4500.0, 0.7);
    let top = (5000.0 / hz).floor() as usize;
    let buzz: Vec<f64> = (1..=top).map(|h| 1.0 / pow(h as f64, 1.5)).collect();
    let consonant = r.next() < 0.6;
    let mut phase = 0.0;
    for j in 0..len {
        let t = j as f64 / rate;
        let vib = 0.017 * ramp(0.25, 0.5, t) * sin(2.0 * PI * 5.5 * t);
        phase += (hz * (1.0 + vib)) / rate;
        let mut src = harmonics(phase, &buzz);
        src += (r.next() * 2.0 - 1.0) * 0.05;
        let mut v = 0.0;
        for (f, w) in filters.iter_mut().zip(weights) {
            v += f.run(src) * w;
        }
        let env = ramp(0.02, 0.06, t) * (1.0 - ramp(seconds - 0.06, seconds, t));
        let s = if consonant {
            exp(-t / 0.03) * (1.0 - ramp(0.05, 0.06, t))
        } else {
            0.0
        };
        let i = (start + j) % d.len();
        add(
            d,
            i,
            v * env * 2.2 + hiss.run(r.next() * 2.0 - 1.0) * s * 0.25,
        );
    }
}

/// A soft synth note: a few harmonics, slowly attacked and released (the backing's pad and bass; the
/// TypeScript's default \`harmonics\` was 6).
fn pad(d: &mut [f32], rate: f64, at: f64, seconds: f64, hz: f64, amp: f64, harmonics: usize) {
    let start = round_usize(at * rate);
    let len = round_usize(seconds * rate);
    let weights: Vec<f64> = (1..=harmonics).map(|h| 1.0 / (h as f64 * 1.5)).collect();
    for j in 0..len {
        let t = j as f64 / rate;
        let v = crate::dsp::harmonics(hz * t, &weights);
        let i = (start + j) % d.len();
        add(
            d,
            i,
            v * amp * ramp(0.0, 0.05, t) * (1.0 - ramp(seconds - 0.08, seconds, t)),
        );
    }
}

// Played once.

/// A fridge door opened and shut: the handle, the seal letting go with a soft suck, the bottles in the door
/// clinking as it swings, a jar taken out, and about a second and a half later the door pushed shut (a rush
/// of air and the seal's thump), the bottles rattling again.
fn fridge_door(rate: f64) -> Stereo {
    let mut r = Rng::new(60.0);
    let mut out = once(rate, 2.6);
    strike(
        &mut out,
        0.02,
        0.15,
        0.0,
        &[[1700.0, 0.006, 1.0]],
        burst(2500.0, 1.0, 0.002, 0.6),
    );
    strike(
        &mut out,
        0.08,
        0.6,
        0.0,
        &[[90.0, 0.02, 0.8]],
        burst(380.0, 0.8, 0.02, 1.0),
    );
    for at in [0.22, 0.3, 0.46] {
        let at = at + 0.03 * r.next();
        let amp = 0.15 + 0.1 * r.next();
        let hz = 1800.0 + 800.0 * r.next();
        glass(&mut out, at, amp, 0.15, hz);
    }
    glass(&mut out, 0.95, 0.12, 0.1, 1400.0);
    swish(&mut out, &mut r, 1.42, 0.16, 0.25, 500.0);
    strike(
        &mut out,
        1.58,
        1.0,
        0.0,
        &[[70.0, 0.06, 1.0], [160.0, 0.03, 0.5]],
        burst(300.0, 0.7, 0.015, 1.2),
    );
    for at in [1.6, 1.65] {
        let at = at + 0.02 * r.next();
        let hz = 2000.0 + 600.0 * r.next();
        glass(&mut out, at, 0.12, 0.15, hz);
    }
    done(
        rate,
        vec![out.into()],
        0.0,
        0.8,
        &[3.1, 5.3, 7.1, 9.7],
        0.2,
        0.0,
        0.0,
    )
}

/// A pedal bin: the pedal pressed and the steel lid flying up against its stop, the plate scraped with a
/// fork (a squeaky scratch, three strokes) and the leftovers dropping in, the lid let fall shut.
fn bin_lid(rate: f64) -> Stereo {
    let mut r = Rng::new(61.0);
    let mut out = once(rate, 2.8);
    strike(
        &mut out,
        0.0,
        0.3,
        0.0,
        &[[900.0, 0.02, 1.0], [2300.0, 0.015, 0.6]],
        burst(1500.0, 0.8, 0.003, 1.0),
    );
    lid(&mut out, 0.09, 0.6);
    for k in 0..3 {
        let at = 0.45 + k as f64 * 0.3;
        let start = js_round(at * rate);
        let len = js_round(0.22 * rate);
        let mut bp = Biquad::with(rate, Filter::Bandpass, 3400.0, 1.5);
        let mut j = 0.0;
        while j < len {
            let t = j / rate;
            let stick = 0.5 + 0.5 * sin(2.0 * PI * (2700.0 + 300.0 * sin(t * 40.0)) * t).abs();
            let v = bp.run(r.next() * 2.0 - 1.0) * stick * 0.5 * sin((PI * j) / len);
            put(&mut out, start + j, v, 0.7, 0.7);
            j += 1.0;
        }
    }
    strike(
        &mut out,
        0.8,
        0.5,
        0.0,
        &[[120.0, 0.04, 1.0]],
        burst(400.0, 0.6, 0.04, 1.0),
    );
    lid(&mut out, 1.75, 0.9);
    done(
        rate,
        vec![out.into()],
        0.0,
        0.8,
        &[3.1, 5.3, 7.1, 9.7],
        0.2,
        0.0,
        0.0,
    )
}

/// A thin steel lid hitting its stop or the bin's rim: a quick metal ring.
fn lid(out: &mut Ev, at: f64, amp: f64) {
    strike(
        out,
        at,
        amp,
        0.0,
        &[
            [620.0, 0.12, 0.6],
            [1530.0, 0.09, 0.5],
            [2780.0, 0.06, 0.4],
            [4100.0, 0.04, 0.25],
        ],
        burst(2500.0, 0.6, 0.003, 1.2),
    );
}

/// A snack from a vending machine: the coins in and rolling down the chute, a button, the spiral's motor
/// turning for a second and a half, the packet dropping into the tray and the flap pushed.
fn vend_snack(rate: f64) -> Stereo {
    let mut r = Rng::new(62.0);
    let mut out = once(rate, 4.2);
    coins(&mut out, &mut r, 0.0);
    coins(&mut out, &mut r, 0.45);
    strike(
        &mut out,
        1.2,
        0.3,
        0.0,
        &[[3000.0, 0.004, 1.0]],
        burst(2500.0, 1.0, 0.002, 0.8),
    );
    beep(&mut out, 1.25, 0.12, 1000.0, 0.04);
    let start = js_round(1.5 * rate);
    let len = round_usize(1.5 * rate);
    let whine = motor(len, rate, |_| 140.0, &[1.0, 0.7, 0.5, 0.3, 0.2]);
    let gears = motor(len, rate, |_| 1120.0, &[1.0, 0.3]);
    for j in 0..len {
        let env = ramp(0.0, 0.08, j as f64 / rate) * (1.0 - ramp(1.4, 1.5, j as f64 / rate));
        put(
            &mut out,
            start + j as f64,
            (whine[j] as f64 * 0.06 + gears[j] as f64 * 0.01) * env,
            0.7,
            0.7,
        );
    }
    let mut t = 1.55;
    while t < 2.95 {
        let f = 1800.0 + 400.0 * r.next();
        strike(
            &mut out,
            t,
            0.04,
            0.0,
            &[[f, 0.004, 1.0]],
            burst(2000.0, 1.0, 0.001, 0.5),
        );
        t += 0.065;
    }
    strike(
        &mut out,
        3.05,
        0.5,
        0.0,
        &[[110.0, 0.03, 1.0], [330.0, 0.02, 0.4]],
        burst(600.0, 0.6, 0.008, 1.0),
    );
    for _ in 0..14 {
        let at = 3.05 + r.next() * 0.15;
        let hz = 3500.0 + 3000.0 * r.next();
        strike(&mut out, at, 0.04, 0.0, &[], burst(hz, 1.0, 0.001, 1.0));
    }
    flap(&mut out, 3.7);
    done(
        rate,
        vec![out.into()],
        0.0,
        0.8,
        &[3.1, 5.3, 7.1, 9.7],
        0.15,
        0.0,
        0.0,
    )
}

/// A can from a vending machine: the coins, a button, the latch letting go and the can tumbling down inside
/// (a full can's dull clank) into the metal tray, the flap; then the can opened, a crack and a hiss.
fn vend_can(rate: f64) -> Stereo {
    let mut r = Rng::new(63.0);
    let mut out = once(rate, 4.8);
    coins(&mut out, &mut r, 0.0);
    coins(&mut out, &mut r, 0.45);
    strike(
        &mut out,
        1.2,
        0.3,
        0.0,
        &[[3000.0, 0.004, 1.0]],
        burst(2500.0, 1.0, 0.002, 0.8),
    );
    beep(&mut out, 1.25, 0.12, 1000.0, 0.04);
    strike(
        &mut out,
        1.55,
        0.4,
        0.0,
        &[[400.0, 0.03, 1.0], [1900.0, 0.01, 0.4]],
        burst(1200.0, 0.8, 0.003, 1.0),
    );
    for (at, a) in [(1.7, 0.5), (1.86, 0.6), (1.98, 0.35)] {
        strike(
            &mut out,
            at,
            a,
            0.0,
            &[
                [1100.0, 0.04, 0.5],
                [2600.0, 0.03, 0.4],
                [4200.0, 0.02, 0.3],
                [180.0, 0.02, 0.6],
            ],
            burst(1500.0, 0.7, 0.003, 1.0),
        );
    }
    strike(
        &mut out,
        2.1,
        0.9,
        0.0,
        &[[500.0, 0.08, 0.6], [1250.0, 0.05, 0.4], [180.0, 0.03, 0.8]],
        burst(1000.0, 0.6, 0.004, 1.0),
    );
    flap(&mut out, 2.7);
    strike(
        &mut out,
        3.8,
        0.5,
        0.0,
        &[[3900.0, 0.004, 1.0]],
        burst(4000.0, 0.8, 0.002, 2.0),
    );
    let start = js_round(3.81 * rate);
    let len = js_round(0.7 * rate);
    let mut hp = Biquad::with(rate, Filter::Bandpass, 6000.0, 0.8);
    let mut j = 0.0;
    while j < len {
        let v = hp.run(r.next() * 2.0 - 1.0) * 0.5 * exp(-j / rate / 0.12);
        put(&mut out, start + j, v, 0.7, 0.7);
        j += 1.0;
    }
    for _ in 0..40 {
        let at = 3.85 + r.next() * 0.6;
        let hz = 5000.0 + 4000.0 * r.next();
        strike(&mut out, at, 0.015, 0.0, &[], burst(hz, 1.0, 0.0008, 1.0));
    }
    done(
        rate,
        vec![out.into()],
        0.0,
        0.8,
        &[3.1, 5.3, 7.1, 9.7],
        0.15,
        0.0,
        0.0,
    )
}

/// A coin into a machine: its clink at the slot and the rattle down the chute into the box.
fn coins(out: &mut Ev, r: &mut Rng, at: f64) {
    strike(
        out,
        at,
        0.2,
        0.0,
        &[
            [3800.0, 0.05, 1.0],
            [9100.0, 0.03, 0.5],
            [8500.0, 0.02, 0.2],
        ],
        burst(4000.0, 1.0, 0.001, 0.6),
    );
    for k in 0..5 {
        let when = at + 0.08 + k as f64 * 0.025 + 0.01 * r.next();
        let f = 3200.0 + 600.0 * r.next();
        strike(
            out,
            when,
            0.06,
            0.0,
            &[[f, 0.01, 1.0]],
            burst(3500.0, 1.0, 0.001, 0.5),
        );
    }
    strike(
        out,
        at + 0.25,
        0.15,
        0.0,
        &[[1800.0, 0.03, 1.0], [4300.0, 0.02, 0.4]],
        burst(2500.0, 0.8, 0.002, 0.6),
    );
}

/// A machine's flap pushed in and swinging back.
fn flap(out: &mut Ev, at: f64) {
    strike(
        out,
        at,
        0.3,
        0.0,
        &[[1200.0, 0.02, 1.0], [2700.0, 0.012, 0.4]],
        burst(1500.0, 0.7, 0.003, 1.0),
    );
    strike(
        out,
        at + 0.25,
        0.18,
        0.0,
        &[[1250.0, 0.02, 1.0], [2800.0, 0.012, 0.4]],
        burst(1500.0, 0.7, 0.003, 1.0),
    );
}

/// A call from a phone box: the receiver lifted, the dial tone (350 and 440 Hz) faint from the earpiece, a
/// coin dropped in, the number pressed (each button's click and its pair of tones), and the ringing at the
/// other end (400 and 450 Hz, twice in a row, then a pause).
fn phone_box(rate: f64) -> Stereo {
    let mut r = Rng::new(64.0);
    let mut out = once(rate, 7.5);
    strike(
        &mut out,
        0.0,
        0.5,
        0.0,
        &[[900.0, 0.02, 1.0], [2100.0, 0.012, 0.5]],
        burst(1500.0, 0.7, 0.003, 1.0),
    );
    strike(
        &mut out,
        0.06,
        0.2,
        0.0,
        &[[3200.0, 0.004, 1.0]],
        burst(3000.0, 1.0, 0.001, 0.6),
    );
    let mut earpiece = vec![0.0f32; out.left.len()];
    tones(&mut earpiece, rate, 0.2, 1.1, &[350.0, 440.0]);
    coins(&mut out, &mut r, 0.65);
    let keys = [
        [697.0, 1209.0],
        [770.0, 1336.0],
        [852.0, 1477.0],
        [697.0, 1336.0],
        [941.0, 1336.0],
        [770.0, 1209.0],
        [852.0, 1336.0],
    ];
    for (k, pair) in keys.iter().enumerate() {
        let at = 1.5 + k as f64 * 0.22 + 0.04 * r.next();
        strike(
            &mut out,
            at,
            0.25,
            0.0,
            &[[1500.0, 0.01, 1.0], [3300.0, 0.006, 0.4]],
            burst(2000.0, 0.8, 0.002, 1.0),
        );
        tones(&mut earpiece, rate, at + 0.01, 0.12, pair);
    }
    for at in [3.7, 4.3, 6.7, 7.3] {
        tones(&mut earpiece, rate, at, 0.4, &[400.0, 450.0]);
    }
    // The earpiece is small and held to an ear: faint, and only the telephone's band.
    let mut hp = Biquad::with(rate, Filter::Highpass, 300.0, 0.7);
    let mut lp = Biquad::with(rate, Filter::Lowpass, 3400.0, 0.7);
    for (i, &e) in earpiece.iter().enumerate() {
        put(
            &mut out,
            i as f64,
            lp.run(hp.run(e as f64)) * 0.05,
            0.7,
            0.7,
        );
    }
    done(
        rate,
        vec![out.into()],
        0.0,
        0.8,
        &[2.3, 3.7, 5.1, 6.7],
        0.2,
        0.0,
        0.0,
    )
}

/// The receiver put back on its hook: a plastic clack, a bounce, and the hook's switch.
fn hang_up(rate: f64) -> Stereo {
    let mut out = once(rate, 0.8);
    strike(
        &mut out,
        0.01,
        0.8,
        0.0,
        &[
            [850.0, 0.025, 1.0],
            [2100.0, 0.015, 0.5],
            [180.0, 0.03, 0.6],
        ],
        burst(1500.0, 0.7, 0.004, 1.2),
    );
    strike(
        &mut out,
        0.05,
        0.3,
        0.0,
        &[[900.0, 0.02, 1.0], [2200.0, 0.012, 0.4]],
        burst(1500.0, 0.7, 0.003, 1.0),
    );
    strike(
        &mut out,
        0.07,
        0.2,
        0.0,
        &[[3200.0, 0.004, 1.0]],
        burst(3000.0, 1.0, 0.001, 0.6),
    );
    done(
        rate,
        vec![out.into()],
        0.0,
        0.8,
        &[2.3, 3.7, 5.1, 6.7],
        0.2,
        0.0,
        0.0,
    )
}

/// A microwave done: the timer's click and its bell's ping, then the door's latch let go and the door swung
/// open.
fn ping(rate: f64) -> Stereo {
    let mut out = once(rate, 2.4);
    strike(
        &mut out,
        0.0,
        0.2,
        0.0,
        &[[2400.0, 0.005, 1.0]],
        burst(2500.0, 1.0, 0.002, 0.6),
    );
    strike(
        &mut out,
        0.03,
        0.7,
        0.0,
        &[
            [2350.0, 0.6, 1.0],
            [5450.0, 0.3, 0.35],
            [9990.0, 0.15, 0.15],
        ],
        burst(4000.0, 1.0, 0.001, 0.4),
    );
    strike(
        &mut out,
        1.4,
        0.4,
        0.0,
        &[[1100.0, 0.02, 1.0], [2600.0, 0.012, 0.5]],
        burst(1800.0, 0.7, 0.003, 1.0),
    );
    strike(
        &mut out,
        1.55,
        0.2,
        0.0,
        &[[420.0, 0.04, 1.0]],
        burst(600.0, 4.0, 0.08, 0.4),
    );
    done(
        rate,
        vec![out.into()],
        0.0,
        0.75,
        &[3.1, 5.3, 7.1, 9.7],
        0.15,
        0.0,
        0.0,
    )
}

/// Toast popping up: the spring letting go, the carriage's clank at the top (and a bounce), the slices
/// knocking, then lifted out.
fn toast_pop(rate: f64) -> Stereo {
    let mut r = Rng::new(65.0);
    let mut out = once(rate, 1.6);
    strike(
        &mut out,
        0.0,
        0.9,
        0.0,
        &[
            [1300.0, 0.03, 0.6],
            [2900.0, 0.02, 0.5],
            [4700.0, 0.012, 0.3],
            [190.0, 0.08, 0.25],
        ],
        burst(2000.0, 0.7, 0.003, 1.2),
    );
    strike(
        &mut out,
        0.05,
        0.35,
        0.0,
        &[[1350.0, 0.02, 0.6], [3000.0, 0.015, 0.4]],
        burst(2000.0, 0.7, 0.002, 1.0),
    );
    for at in [0.03, 0.07] {
        strike(
            &mut out,
            at + 0.01 * r.next(),
            0.2,
            0.0,
            &[[700.0, 0.01, 1.0]],
            burst(1000.0, 0.8, 0.003, 0.8),
        );
    }
    rustle(&mut out, &mut r, 0.75, 0.2, 0.06, 0.0, 2500.0, 0.8);
    done(
        rate,
        vec![out.into()],
        0.0,
        0.75,
        &[3.1, 5.3, 7.1, 9.7],
        0.15,
        0.0,
        0.0,
    )
}

/// Coffee poured into a cup: the stream's splash, and the ring of the air left in the cup rising as it fills
/// (a quarter wave in the space above the coffee: about 950 Hz with 9 cm left, near 2.4 kHz with 3.5 cm).
fn pour(rate: f64) -> Stereo {
    let mut r = Rng::new(66.0);
    let mut out = once(rate, 2.6);
    let n = out.left.len();
    let mut splash = Biquad::with(rate, Filter::Bandpass, 2600.0, 0.5);
    let mut air = Biquad::new(rate);
    let mut pink = Pink::new();
    let burble = wander(&mut r, n, rate, 0.03);
    for (i, &b) in burble.iter().enumerate() {
        let t = i as f64 / rate;
        let filled = ramp(0.1, 2.1, t);
        if i & 31 == 0 {
            air.set(
                Filter::Bandpass,
                343.0 / (4.0 * (0.09 - 0.055 * filled)),
                6.0,
            );
        }
        let on = ramp(0.05, 0.12, t) * (1.0 - ramp(2.05, 2.2, t));
        let w = pink.next(&mut r);
        put(
            &mut out,
            i as f64,
            (air.run(w) * 2.2 + splash.run(w) * 0.35) * on * (1.0 + 0.3 * b as f64),
            0.7,
            0.7,
        );
    }
    let mut t = 0.1;
    while t < 2.1 {
        for c in 0..2 {
            let f = 900.0 + 1800.0 * r.next();
            bubble(
                if c == 0 {
                    &mut out.left
                } else {
                    &mut out.right
                },
                rate,
                t,
                f,
                0.03,
                0.3,
            );
        }
        t += -log(1.0 - r.next()) / 60.0;
    }
    done(
        rate,
        vec![out.into()],
        0.0,
        0.7,
        &[2.3, 3.7, 5.1, 6.7],
        0.15,
        0.0,
        0.0,
    )
}

/// A glass bottle or jar knocking another: a clink.
fn glass(out: &mut Ev, at: f64, amp: f64, pan: f64, hz: f64) {
    strike(
        out,
        at,
        amp,
        pan,
        &[
            [hz, 0.06, 1.0],
            [hz * 2.32, 0.04, 0.5],
            [hz * 4.25, 0.025, 0.3],
        ],
        burst(hz * 2.0, 1.0, 0.001, 0.5),
    );
}

/// A short electronic beep.
fn beep(out: &mut Ev, at: f64, seconds: f64, hz: f64, amp: f64) {
    let rate = out.rate;
    let start = js_round(at * rate);
    let len = round_usize(seconds * rate);
    for j in 0..len {
        let t = j as f64 / rate;
        let v = (sin(2.0 * PI * hz * t) + 0.3 * sin(6.0 * PI * hz * t))
            * amp
            * min(
                min(1.0, j as f64 / (0.003 * rate)),
                (len - j) as f64 / (0.003 * rate),
            );
        put(out, start + j as f64, v, 0.7, 0.7);
    }
}

/// A pair of tones (a telephone's), on for \`seconds\`.
fn tones(d: &mut [f32], rate: f64, at: f64, seconds: f64, hz: &[f64]) {
    let start = js_round(at * rate);
    let len = min(d.len() as f64 - start, js_round(seconds * rate));
    let mut j = 0.0;
    while j < len {
        let t = j / rate;
        let mut v = 0.0;
        for f in hz {
            v += sin(2.0 * PI * f * t);
        }
        add(
            d,
            (start + j) as usize,
            v * min(min(1.0, j / (0.005 * rate)), (len - j) / (0.005 * rate)),
        );
        j += 1.0;
    }
}

// Building blocks.

/// The average level of the loops of separate knocks (darts, a rally): set by their average rather than
/// their peak, as their knocks are short; the sharpest are rounded off a little (\`set_level\`).
const SPARSE: f64 = 0.046;

/// A mode: its frequency (Hz), the time it takes to die to a third (s) and its level.
type Mode = [f64; 3];

/// A short burst of noise in a band (its centre in Hz, its Q), dying away over \`tau\` seconds, at \`amp\`.
#[derive(Clone, Copy, Debug)]
struct Burst {
    hz: f64,
    q: f64,
    tau: f64,
    amp: f64,
}

/// A burst (\`{ hz, q, tau, amp }\`).
fn burst(hz: f64, q: f64, tau: f64, amp: f64) -> Burst {
    Burst { hz, q, tau, amp }
}

/// A track of events: its two channels, its rate, and whether its events wrap round its end into its start
/// (a loop of events, \`looped\`) or are cut off there (a sound played once, \`once\`). No overlap.
struct Ev {
    left: Vec<f32>,
    right: Vec<f32>,
    rate: f64,
    wraps: bool,
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
/// \`loop\`).
fn looped(rate: f64, seconds: f64) -> Ev {
    let t = track(rate, seconds, 0.0);
    Ev {
        left: t.left,
        right: t.right,
        rate,
        wraps: true,
    }
}

thread_local! {
    /// Whether the sound being made is played once (\`once\` made its track): \`done\` then runs its filters and
    /// room from silence rather than from the loop's end, and clears it.
    static PLAYED_ONCE: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
}

/// A track for a sound played once: nothing wraps.
fn once(rate: f64, seconds: f64) -> Ev {
    PLAYED_ONCE.with(|o| o.set(true));
    let t = track(rate, seconds, 0.0);
    Ev {
        left: t.left,
        right: t.right,
        rate,
        wraps: false,
    }
}

/// Adds \`v\` at sample \`i\`, left and right at \`l\` and \`r\`; past the end or before the start it wraps on a
/// loop of events (an event set a hair before the loop's start rings on from its end, so the seam is whole),
/// else it's dropped.
fn put(t: &mut Ev, i: f64, v: f64, l: f64, r: f64) {
    let n = t.left.len();
    if !t.wraps && (i < 0.0 || i >= n as f64) {
        return;
    }
    let i = (i as i64).rem_euclid(n as i64) as usize;
    add(&mut t.left, i, v * l);
    add(&mut t.right, i, v * r);
}

/// Something struck: its modes ringing out over a burst of noise, from \`at\` (s), at \`amp\`, placed at \`pan\`.
fn strike(t: &mut Ev, at: f64, amp: f64, pan: f64, modes: &[Mode], burst: Burst) {
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

/// A rustle or a rub: noise in a band (\`hz\`, \`q\`; the TypeScript's defaults 4000 and 0.5), swelling and
/// fading over \`seconds\`.
#[allow(clippy::too_many_arguments)]
fn rustle(t: &mut Ev, r: &mut Rng, at: f64, seconds: f64, amp: f64, pan: f64, hz: f64, q: f64) {
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
fn swish(t: &mut Ev, r: &mut Rng, at: f64, seconds: f64, amp: f64, hz: f64) {
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

/// A motor's (or a fan's) tone: harmonics at \`partials\` levels (the fundamental first) of a speed \`hz(i)\`
/// (Hz at sample \`i\`).
fn motor(n: usize, rate: f64, hz: impl Fn(usize) -> f64, partials: &[f64]) -> Vec<f32> {
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
fn noise(n: usize, rate: f64, r: &mut Rng, kind: Filter, hz: f64, q: f64) -> Vec<f32> {
    let mut p = Pink::new();
    let mut f = Biquad::with(rate, kind, hz, q);
    (0..n).map(|_| f.run(p.next(r)) as f32).collect()
}

/// Puts the parts together: the steady ones (already folded into their start: \`fold\`) and the events
/// added; then a room (\`delays\` in ms, none for out of doors, mixed in at \`wet\`), the highs and lows taken
/// off (\`low\`, \`high\`: a box or a small speaker; 0 for none), and the level set to an average of \`rms\`, or
/// with 0 to a peak of \`peak\`.
#[allow(clippy::too_many_arguments)]
fn done(
    rate: f64,
    parts: Vec<Stereo>,
    rms: f64,
    peak: f64,
    delays: &[f64],
    wet: f64,
    low: f64,
    high: f64,
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
    // played once (\`once\`) starts from silence.
    let lead = if PLAYED_ONCE.with(|o| o.replace(false)) || n == 0 {
        0
    } else {
        round_usize(rate)
    };
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

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 48000.0;

    /// The biggest step between neighbouring samples of a channel, and the step from its last sample round
    /// to its first.
    fn steps(ch: &[f32]) -> (f64, f64) {
        let inside = ch
            .windows(2)
            .map(|w| (w[1] as f64 - w[0] as f64).abs())
            .fold(0.0, f64::max);
        (inside, (ch[0] as f64 - ch[ch.len() - 1] as f64).abs())
    }

    /// Each thing as made (before the game's level): long enough, never past full scale (rounded off above
    /// 0.7: \`set_level\`), with sound in it, and a loop's end running on into its start.
    #[test]
    fn world_sounds() {
        for kind in THINGS {
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
            // (A NaN or an infinity anywhere carries into the sum.)
            assert!(sum.is_finite(), "{kind}: not finite");
            assert!(peak <= 1.0 + 1e-6, "{kind}: peak {peak}");
            let rms = (sum / (2.0 * left.len() as f64)).sqrt();
            assert!(rms > 0.004, "{kind}: rms {rms}");
            if !ONCE_THINGS.contains(&kind) {
                // The step from the last sample to the first is no bigger than the ones inside the loop.
                for ch in [&left, &right] {
                    let (inside, seam) = steps(ch);
                    assert!(seam <= inside, "{kind}: seam {seam} over {inside}");
                }
            }
        }
    }

    /// A loop's end runs on into its start at the browser's other rates too (the filters and the room in \`done\`
    /// already ringing at its start).
    #[test]
    fn every_loop_is_seamless_at_44_1_and_96_khz() {
        let mut off = Vec::new();
        for rate in [44100.0, 96000.0] {
            for kind in THINGS.iter().filter(|k| !ONCE_THINGS.contains(k)) {
                let s = make(kind, rate).unwrap();
                for ch in [&s.left, &s.right] {
                    let (inside, seam) = steps(ch);
                    if seam > inside {
                        off.push(format!("{kind} at {rate}: seam {seam} over {inside}"));
                    }
                }
            }
        }
        assert!(off.is_empty(), "{off:#?}");
    }

    #[test]
    fn only_the_things_are_made_here() {
        assert!(make("shower", RATE).is_none());
        assert!(make("nothing", RATE).is_none());
        assert!(ONCE_THINGS.iter().all(|k| THINGS.contains(k)));
    }
}
