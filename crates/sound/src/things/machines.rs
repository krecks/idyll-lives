//! The machines: a treadmill, bikes, a rowing machine, a sewing machine, a pottery wheel, coffee machines, a blender, a mixer, a microwave, a toaster, an oven fan, a cooker hood, a fire, and the things played once at the fridge, bin, vending machines and phone box.

use super::*;

// The machines.

/// A treadmill with someone running on it: the motor's hum (about 58 Hz, turning the belt at a run), the
/// belt's hiss over the deck, the rollers' rumble, the feet landing at 165 steps a minute (each a thump in
/// the deck and a slap on the belt, one foot a little heavier) and the belt's seam ticking past once a turn.
pub(super) fn treadmill(rate: f64) -> Stereo {
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
pub(super) fn bike(rate: f64) -> Stereo {
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

/// A rowing machine at an easy 21 strokes a minute (2.8 s a stroke: three in the loop): the drive pulls
/// the fan flywheel up to speed, its whoosh rising and brightening, the chain rattling out over its
/// sprocket; on the recovery the wheel coasts down, the strap's spring reels the chain back in more
/// lightly, and the seat rolls along its rail both ways, bumping softly at the catch.
pub(super) fn rower(rate: f64) -> Stereo {
    let mut r = Rng::new(34.0);
    let stroke = 2.8;
    let drive = 0.32;
    let seconds = 3.0 * stroke;
    let mut bed = track(rate, seconds, 0.6);
    let n = bed.left.len();
    let whoosh = noise(n, rate, &mut r, Filter::Bandpass, 480.0, 0.7);
    let rush = noise(n, rate, &mut r, Filter::Bandpass, 1400.0, 0.6);
    let rail = noise(n, rate, &mut r, Filter::Lowpass, 220.0, 0.7);
    for i in 0..n {
        let p = (i as f64 / rate % stroke) / stroke;
        // The wheel's speed: pulled up through the drive, coasting down over the recovery.
        let wheel = if p < drive {
            0.5 + 0.5 * sin(0.5 * PI * p / drive)
        } else {
            0.5 + 0.5 * exp(-2.2 * (p - drive) / (1.0 - drive))
        };
        // The seat's speed along the rail: fast out on the drive, slower back on the recovery.
        let seat = if p < drive {
            sin(PI * p / drive)
        } else {
            0.55 * sin(PI * (p - drive) / (1.0 - drive))
        };
        let v = whoosh[i] as f64 * 0.9 * wheel * wheel
            + rush[i] as f64 * 0.35 * pow(wheel, 4.0)
            + rail[i] as f64 * 0.5 * seat;
        bed.left[i] = v as f32;
        bed.right[i] = v as f32;
    }
    let mut ev = looped(rate, seconds);
    for k in 0..3 {
        let at = k as f64 * stroke + (r.next() - 0.5) * 0.02;
        // The seat meeting the catch.
        strike(
            &mut ev,
            at,
            0.25,
            0.0,
            &[[95.0, 0.04, 1.0], [210.0, 0.02, 0.4]],
            burst(400.0, 0.8, 0.01, 1.0),
        );
        // The chain out over the sprocket on the drive, then reeled back in.
        let runs = [
            (0.04, drive * stroke * 0.9, 16, 0.07),
            (drive * stroke + 0.1, 1.3, 12, 0.035),
        ];
        for (from, length, links, amp) in runs {
            for j in 0..links {
                let a = at + from + length * j as f64 / links as f64 + (r.next() - 0.5) * 0.02;
                let hz = 3100.0 + 900.0 * r.next();
                strike(
                    &mut ev,
                    a,
                    amp * (0.6 + 0.4 * r.next()),
                    0.05,
                    &[[hz, 0.005, 1.0], [5400.0, 0.003, 0.4]],
                    burst(4200.0, 1.0, 0.001, 0.7),
                );
            }
        }
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
pub(super) fn sewing(rate: f64) -> Stereo {
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
pub(super) fn pottery_wheel(rate: f64) -> Stereo {
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
pub(super) fn bubbling(rate: f64) -> Stereo {
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
pub(super) fn espresso(rate: f64) -> Stereo {
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
pub(super) fn blender(rate: f64) -> Stereo {
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
pub(super) fn mixer(rate: f64) -> Stereo {
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
pub(super) fn microwave(rate: f64) -> Stereo {
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
pub(super) fn toaster(rate: f64) -> Stereo {
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
pub(super) fn oven_fan(rate: f64) -> Stereo {
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
pub(super) fn hood(rate: f64) -> Stereo {
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
pub(super) fn fire(rate: f64) -> Stereo {
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
