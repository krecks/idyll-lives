//! The things played once: a fridge door, a pedal bin, the vending machines, a phone box, the microwave's ping, toast popping up, coffee poured.
//!
//! Split from `things.rs` (see `mod.rs`).

use super::*;

// Played once.

/// A fridge door opened and shut: the handle, the seal letting go with a soft suck, the bottles in the door
/// clinking as it swings, a jar taken out, and about a second and a half later the door pushed shut (a rush
/// of air and the seal's thump), the bottles rattling again.
pub(crate) fn fridge_door(rate: f64) -> Stereo {
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
    done_once(
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
pub(crate) fn bin_lid(rate: f64) -> Stereo {
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
    done_once(
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
pub(crate) fn lid(out: &mut Ev, at: f64, amp: f64) {
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
pub(crate) fn vend_snack(rate: f64) -> Stereo {
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
    done_once(
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
pub(crate) fn vend_can(rate: f64) -> Stereo {
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
    done_once(
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
pub(crate) fn coins(out: &mut Ev, r: &mut Rng, at: f64) {
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
pub(crate) fn flap(out: &mut Ev, at: f64) {
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
pub(crate) fn phone_box(rate: f64) -> Stereo {
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
    done_once(
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
pub(crate) fn hang_up(rate: f64) -> Stereo {
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
    done_once(
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
pub(crate) fn ping(rate: f64) -> Stereo {
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
    done_once(
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
pub(crate) fn toast_pop(rate: f64) -> Stereo {
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
    done_once(
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
pub(crate) fn pour(rate: f64) -> Stereo {
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
    done_once(
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
pub(crate) fn glass(out: &mut Ev, at: f64, amp: f64, pan: f64, hz: f64) {
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
pub(crate) fn beep(out: &mut Ev, at: f64, seconds: f64, hz: f64, amp: f64) {
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
pub(crate) fn tones(d: &mut [f32], rate: f64, at: f64, seconds: f64, hz: &[f64]) {
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
