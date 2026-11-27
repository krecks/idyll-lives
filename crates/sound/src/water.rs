//! Water for the sounds of the world, synthesised (no sound files): a shower, a tap running into a basin,
//! washing up, a bath filling and someone soaking in it, a washing machine, a fountain and a toilet's flush;
//! and, made from the same parts, a tumble dryer and a dishwasher, running and being loaded (was
//! `web/src/audio/water.ts`).
//!
//! Each is a stereo loop several seconds long, made once and played on repeat, so it costs nothing while
//! playing. Running water is mostly two things, and both are built here (`dsp`):
//! - the jet's rush: pink noise in a broad band (most of it between about 500 Hz and 8 kHz), its centre and
//!   level wandering slowly as the flow and whatever it lands on change;
//! - drops and bubbles: thousands of tiny ticks where the spray lands, and the short rising "blip" of each
//!   air bubble the stream pushes under the surface (the gurgle of a tap), modelled as a decaying sine whose
//!   pitch rises a little as it rings (van den Doel's bubble model).
//!
//! Then the room: a short, damped reverb for a tiled stall, a basin, a bathroom. The end of each loop is
//! cross-faded into its start, so the seam can't be heard (the flush is played once: `ONCE`).

use crate::dsp::m::*;
use crate::dsp::*;

/// The water sounds, by name (`WATER`), in the TypeScript's order.
pub const WATER: [&str; 11] = [
    "shower",
    "tap",
    "washUp",
    "bathFill",
    "bathSoak",
    "washer",
    "dryer",
    "dishwasher",
    "crockery",
    "fountain",
    "flush",
];

/// The sounds played once as something happens (a flush), not looped while it lasts.
pub const ONCE: [&str; 1] = ["flush"];

/// A water sound by name (`WATER`). None: no such sound.
pub fn make(kind: &str, rate: f64) -> Option<Stereo> {
    Some(match kind {
        "shower" => shower(rate),
        "tap" => tap(rate),
        "washUp" => wash_up(rate),
        "bathFill" => bath_fill(rate),
        "bathSoak" => bath_soak(rate),
        "washer" => washer(rate),
        "dryer" => dryer(rate),
        "dishwasher" => dishwasher(rate),
        "crockery" => crockery(rate),
        "fountain" => fountain(rate),
        "flush" => flush(rate),
        _ => return None,
    })
}

/// `x ** n` for a whole `n` (0 to 1000) as the TypeScript's sounds were made (by bun: JavaScriptCore squares and
/// multiplies for such a power rather than calling `pow`), so `x ** 3` is `x * (x * x)`, which `pow(x, 3.0)` is not
/// always in its last bit (`x ** 2` is `x * x` either way).
pub fn powi(mut x: f64, mut n: u32) -> f64 {
    let mut result = 1.0;
    while n != 0 {
        if n & 1 == 1 {
            result *= x;
        }
        x *= x;
        n >>= 1;
    }
    result
}

/// Each channel through its own small room (`room`): the left with `left`'s delays, the right with `right`'s.
fn rooms(out: &mut Track, left: &[f64], right: &[f64], feedback: f64, damping: f64, wet: f64) {
    let rate = out.rate;
    room(&mut out.left, rate, left, feedback, damping, wet);
    room(&mut out.right, rate, right, feedback, damping, wet);
}

/// Inside a metal box (or shut in behind a door): the highs over `lowpass` Hz muffled, nothing under `highpass`, a
/// little ring.
fn boxed(out: &mut Track, lowpass: f64, highpass: f64) {
    let rate = out.rate;
    for c in 0..2 {
        let mut lp = Biquad::with(rate, Filter::Lowpass, lowpass, 0.7);
        let mut hp = Biquad::with(rate, Filter::Highpass, highpass, 0.7);
        let ch = out.ch(c);
        for v in ch.iter_mut() {
            *v = hp.run(lp.run(*v as f64)) as f32;
        }
        let delays: &[f64] = if c == 1 {
            &[3.7, 6.1, 8.3, 10.9]
        } else {
            &[4.1, 5.9, 8.9, 10.3]
        };
        room(ch, rate, delays, 0.55, 0.3, 0.2);
    }
}

/// A shower: spray on the tiles and on someone standing in it, in a small tiled stall.
pub fn shower(rate: f64) -> Stereo {
    let mut r = Rng::new(11.0);
    let mut out = track(rate, 9.0, 0.6);
    let n = out.left.len();
    let seconds = out.seconds;
    // Someone moving under the spray: more of it lands on them (duller, a little fuller) or on the tiles (brighter).
    let body = wander(&mut r, n, rate, 0.9);
    // The flow itself, never quite steady.
    let flow = wander(&mut r, n, rate, 0.12);
    let mut shared = Pink::new();
    for c in 0..2 {
        let ch = out.ch(c);
        let mut own = Pink::new();
        let mut spray = Biquad::new(rate);
        let mut spray2 = Biquad::new(rate);
        let mut fizz = Biquad::with(rate, Filter::Highpass, 4500.0, 0.6);
        let mut fizz2 = Biquad::with(rate, Filter::Lowpass, 10000.0, 0.6);
        let mut tray = Biquad::with(rate, Filter::Lowpass, 320.0, 0.7);
        let mut patter = Biquad::with(rate, Filter::Bandpass, 3800.0, 0.6);
        let mut floor = Biquad::with(rate, Filter::Highpass, 170.0, 0.6);
        let mut sh = Pink::new();
        for i in 0..n {
            let b = 0.5 + 0.5 * tanh(body[i] as f64);
            if i & 31 == 0 {
                let centre = 3400.0 * pow(2.0, -0.7 * b + 0.08 * flow[i] as f64);
                spray.set(Filter::Bandpass, centre, 0.55);
                spray2.set(Filter::Bandpass, centre * 0.45, 0.9);
            }
            let w = 0.55 * shared.next(&mut r) + 0.8 * own.next(&mut r);
            let level = 1.0 + 0.1 * flow[i] as f64;
            // Drops landing: a dense run of tiny ticks of all sizes.
            let drop = if r.next() < 1400.0 / rate {
                (r.next() - 0.5) * 6.0 * powi(r.next(), 3)
            } else {
                0.0
            };
            let water = spray.run(w) * 1.2
                + spray2.run(w) * (0.25 + 0.35 * b)
                + (1.0 - 0.4 * b) * 0.6 * fizz2.run(fizz.run(w));
            let v = level * water
                + tray.run(sh.next(&mut r)) * (0.12 + 0.2 * b)
                + patter.run(drop) * 0.8;
            ch[i] = floor.run(v) as f32;
        }
        // Now and then a few bigger drops ringing in the water on the floor.
        let mut t = r.next() * 0.05;
        while t < seconds {
            let f = 1400.0 * pow(4.0, r.next());
            let amp = 0.15 * (0.2 + pow(r.next(), 2.0));
            let rise = 0.1 + r.next() * 0.3;
            bubble(ch, rate, t, f, amp, rise);
            t += -log(1.0 - r.next()) / 45.0;
        }
    }
    rooms(
        &mut out,
        &[4.7, 8.3, 11.3, 12.7],
        &[5.3, 7.9, 10.7, 13.1],
        0.62,
        0.35,
        0.3,
    );
    finish(out, 0.2, 1.0)
}

/// A tap running into a basin: a narrower, gurgling stream.
pub fn tap(rate: f64) -> Stereo {
    let mut r = Rng::new(12.0);
    let mut out = track(rate, 7.0, 0.6);
    stream(
        &mut out,
        &mut r,
        StreamOpts {
            centre: 1500.0,
            q: 0.9,
            bubbles: 240.0,
            gurgle: 0.15,
            low: 450.0,
            high: 2600.0,
            rumble: 0.15,
        },
    );
    rooms(
        &mut out,
        &[2.1, 3.3, 4.9, 6.1],
        &[2.3, 3.7, 5.1, 6.7],
        0.5,
        0.4,
        0.18,
    );
    finish(out, 0.2, 1.0)
}

/// A bath filling: a fuller stream plunging into deep water, in a bathroom.
pub fn bath_fill(rate: f64) -> Stereo {
    let mut r = Rng::new(13.0);
    let mut out = track(rate, 8.0, 0.6);
    stream(
        &mut out,
        &mut r,
        StreamOpts {
            centre: 950.0,
            q: 0.7,
            bubbles: 480.0,
            gurgle: 0.12,
            low: 180.0,
            high: 1400.0,
            rumble: 0.5,
        },
    );
    rooms(
        &mut out,
        &[12.7, 16.1, 24.7, 27.9],
        &[11.3, 17.9, 23.1, 29.3],
        0.7,
        0.45,
        0.28,
    );
    finish(out, 0.2, 1.0)
}

/// Someone soaking in the bath: the water lapping as they move, a drip from the tap now and then.
pub fn bath_soak(rate: f64) -> Stereo {
    let mut r = Rng::new(14.0);
    let mut out = track(rate, 13.0, 0.6);
    let mut t = 0.4 + r.next();
    while t < out.seconds {
        let seconds = 0.5 + r.next() * 0.9;
        let amp = 0.5 + r.next() * 0.5;
        let pan = (r.next() - 0.5) * 0.8;
        let cutoff = 500.0 + r.next() * 600.0;
        slosh(&mut out, rate, &mut r, t, seconds, amp, pan, cutoff);
        t += 1.6 + r.next() * 3.0;
    }
    let mut t = r.next() * 2.0;
    while t < out.seconds {
        let pan = 0.3 + (r.next() - 0.5) * 0.3;
        let (l, rr) = gains(pan);
        let amp = 0.25 + 0.35 * r.next();
        // A tiny click as it lands, then the plink of the bubble it traps.
        tick(&mut out.left, rate, t, 0.08 * amp * l);
        tick(&mut out.right, rate, t, 0.08 * amp * rr);
        let f = 900.0 + 1300.0 * r.next();
        let rise = 0.5 + r.next() * 0.6;
        bubble(&mut out.left, rate, t + 0.002, f, amp * l, rise);
        let rise = 0.5 + r.next() * 0.6;
        bubble(&mut out.right, rate, t + 0.002, f, amp * rr, rise);
        t += 0.9 + r.next() * 2.6;
    }
    rooms(
        &mut out,
        &[12.7, 16.1, 24.7, 27.9],
        &[11.3, 17.9, 23.1, 29.3],
        0.72,
        0.45,
        0.35,
    );
    finish(out, 0.0, 0.7)
}

/// Washing up: the tap turned on and off, water swished in the bowl, plates and cutlery clinking.
pub fn wash_up(rate: f64) -> Stereo {
    let mut r = Rng::new(15.0);
    let mut out = track(rate, 14.0, 0.6);
    stream(
        &mut out,
        &mut r,
        StreamOpts {
            centre: 1700.0,
            q: 0.8,
            bubbles: 200.0,
            gurgle: 0.12,
            low: 500.0,
            high: 2800.0,
            rumble: 0.12,
        },
    );
    // The tap on for a few seconds, then off while things are scrubbed.
    let mut on = vec![0.0f32; out.left.len()];
    let mut t = 0.0;
    while t < out.seconds {
        let len = 2.2 + r.next() * 3.0;
        let gap = 1.0 + r.next() * 2.2;
        let a = js_round(t * rate);
        let b = js_round((t + len) * rate);
        let end = min(on.len() as f64, b + js_round(0.25 * rate));
        let mut i = a;
        while i < end {
            let up = (i - a) / (0.08 * rate);
            let down = 1.0 - (i - b) / (0.18 * rate);
            on[i as usize] = max(0.0, min(min(1.0, up), if i < b { 1.0 } else { down })) as f32;
            i += 1.0;
        }
        t += len + gap;
    }
    for ch in [&mut out.left, &mut out.right] {
        for (v, o) in ch.iter_mut().zip(&on) {
            *v = (*v as f64 * (*o as f64 * 0.8)) as f32;
        }
    }
    // Swishing a plate through the water.
    let mut t = 0.3 + r.next();
    while t < out.seconds {
        let seconds = 0.18 + r.next() * 0.35;
        let amp = 0.4 + r.next() * 0.3;
        let pan = (r.next() - 0.5) * 0.5;
        let cutoff = 700.0 + r.next() * 900.0;
        slosh(&mut out, rate, &mut r, t, seconds, amp, pan, cutoff);
        t += 0.6 + r.next() * 1.4;
    }
    // Crockery: clinks in little runs, some of them under the water, and the odd rattle of cutlery.
    let mut t = 0.2 + r.next() * 0.5;
    while t < out.seconds {
        let pan = (r.next() - 0.5) * 0.6;
        if r.next() < 0.18 {
            let f = 3200.0 + 1800.0 * r.next();
            let (mut k, mut at) = (0.0, t);
            while k < 2.0 + (r.next() * 3.0).floor() {
                let f0 = f * (0.95 + r.next() * 0.1);
                let amp = 0.18 + 0.12 * r.next();
                clink(
                    &mut out,
                    rate,
                    at,
                    f0,
                    amp,
                    pan,
                    &[1.0, 1.53, 2.37, 3.11],
                    0.18,
                    false,
                );
                k += 1.0;
                at += 0.03 + r.next() * 0.06;
            }
        } else {
            let f = 1300.0 + 1500.0 * r.next();
            let under = r.next() < 0.3;
            let amp = if under {
                0.35
            } else {
                0.25 + 0.5 * pow(r.next(), 2.0)
            };
            clink(
                &mut out,
                rate,
                t,
                f,
                amp,
                pan,
                &[1.0, 2.76, 5.4, 8.93],
                0.08,
                under,
            );
            if r.next() < 0.4 {
                let at = t + 0.07 + r.next() * 0.15;
                let f0 = f * (0.9 + r.next() * 0.25);
                let amp = 0.15 + 0.3 * r.next();
                clink(
                    &mut out,
                    rate,
                    at,
                    f0,
                    amp,
                    pan,
                    &[1.0, 2.76, 5.4, 8.93],
                    0.08,
                    under,
                );
            }
        }
        t += 0.35 + r.next() * 1.3;
    }
    rooms(
        &mut out,
        &[2.9, 4.7, 7.9, 9.1],
        &[3.1, 5.3, 7.1, 9.7],
        0.55,
        0.4,
        0.2,
    );
    finish(out, 0.17, 1.0)
}

/// A washing machine: the drum turning one way then the other, the washing lifted and dropped back into the
/// water, the motor's hum.
pub fn washer(rate: f64) -> Stereo {
    let mut r = Rng::new(16.0);
    let mut out = track(rate, 16.0, 0.6);
    let n = out.left.len();
    // Turning between these times (seconds); paused for a moment in between to change direction.
    let turns = [(0.8, 7.3), (8.8, 15.3)];
    let mut turning = vec![0.0f32; n];
    for &(a, b) in &turns {
        let end = min(n as f64, b * rate);
        let mut i = js_round(a * rate);
        while i < end {
            turning[i as usize] = min(min(1.0, (i / rate - a) / 0.5), (b - i / rate) / 0.5) as f32;
            i += 1.0;
        }
    }
    let mut hum = Biquad::with(rate, Filter::Lowpass, 500.0, 0.7);
    let mut swish = Biquad::with(rate, Filter::Lowpass, 700.0, 0.6);
    let mut swish_hp = Biquad::with(rate, Filter::Highpass, 200.0, 0.6);
    let wobble = wander(&mut r, n, rate, 0.3);
    let mut phase = 0.0;
    let mut p = Pink::new();
    for i in 0..n {
        let s = turning[i] as f64;
        // A soft, buzzy hum that rises with the motor's speed.
        phase += (2.0 * PI * 92.0 * (0.6 + 0.4 * s) * (1.0 + 0.01 * wobble[i] as f64)) / rate;
        let saw = harmonics(phase / (2.0 * PI), &[1.0, 0.5, 0.3, 0.0, 0.15]);
        let v = hum.run(saw) * 0.025 * s
            + swish_hp.run(swish.run(p.next(&mut r))) * 0.8 * (0.3 + 0.7 * s);
        out.left[i] = v as f32;
        out.right[i] = v as f32;
    }
    // Each time round, the washing falls back into the water: a slosh and a soft thud.
    for &(a, b) in &turns {
        let mut t = a + 0.4 + r.next() * 0.3;
        while t < b - 0.3 {
            let seconds = 0.55 + r.next() * 0.2;
            let amp = 0.8 + 0.4 * r.next();
            let pan = (r.next() - 0.5) * 0.2;
            let cutoff = 800.0 + r.next() * 700.0;
            slosh(&mut out, rate, &mut r, t, seconds, amp, pan, cutoff);
            thud(&mut out, rate, t + 0.06, 0.1 + 0.06 * r.next());
            t += 1.15 + r.next() * 0.2;
        }
    }
    // Inside a metal box: the highs muffled, a little ring.
    boxed(&mut out, 3000.0, 110.0);
    finish(out, 0.16, 1.0)
}

/// A tumble dryer: the drum turning steadily one way, the motor's hum and the fan's rush, the dry washing
/// lifted and dropped (softer than a wash), now and then a button or a zip ticking on the drum.
pub fn dryer(rate: f64) -> Stereo {
    let mut r = Rng::new(18.0);
    let mut out = track(rate, 12.0, 0.6);
    let n = out.left.len();
    let mut hum = Biquad::with(rate, Filter::Lowpass, 450.0, 0.7);
    let mut fan = Biquad::with(rate, Filter::Bandpass, 650.0, 0.5);
    let mut fan_hp = Biquad::with(rate, Filter::Highpass, 180.0, 0.6);
    let wobble = wander(&mut r, n, rate, 0.4);
    let mut phase = 0.0;
    let mut p = Pink::new();
    for i in 0..n {
        phase += (2.0 * PI * 100.0 * (1.0 + 0.008 * wobble[i] as f64)) / rate;
        let saw =
            sin(phase) + 0.5 * sin(2.0 * phase) + 0.3 * sin(3.0 * phase) + 0.15 * sin(5.0 * phase);
        let v = hum.run(saw) * 0.03
            + fan_hp.run(fan.run(p.next(&mut r))) * 0.55 * (1.0 + 0.1 * wobble[i] as f64);
        out.left[i] = v as f32;
        out.right[i] = v as f32;
    }
    // The washing tumbling over, about twice a turn: soft thuds.
    let mut t = 0.3 + r.next() * 0.2;
    while t < out.seconds - 0.3 {
        thud(&mut out, rate, t, 0.05 + 0.05 * r.next());
        t += 0.5 + r.next() * 0.45;
    }
    // A button or a zip knocking on the drum.
    let mut t = 0.8 + r.next();
    while t < out.seconds - 0.3 {
        let f = 2400.0 + 1600.0 * r.next();
        let amp = 0.05 + 0.06 * r.next();
        let pan = (r.next() - 0.5) * 0.3;
        clink(
            &mut out,
            rate,
            t,
            f,
            amp,
            pan,
            &[1.0, 2.32, 4.25, 6.63],
            0.05,
            false,
        );
        t += 1.2 + r.next() * 2.5;
    }
    // Inside a metal box, as the washing machine.
    boxed(&mut out, 3000.0, 110.0);
    finish(out, 0.14, 1.0)
}

/// A dishwasher running: the spray arm going round, its jets sweeping over the racks and the door (a hiss
/// that swells as they pass, about once a second), the pump's hum, the water gurgling in the sump.
pub fn dishwasher(rate: f64) -> Stereo {
    let mut r = Rng::new(21.0);
    let mut out = track(rate, 12.0, 0.6);
    let n = out.left.len();
    let mut hum = Biquad::with(rate, Filter::Lowpass, 400.0, 0.7);
    let wobble = wander(&mut r, n, rate, 0.3);
    let mut phase = 0.0;
    let mut arm = r.next() * 2.0 * PI;
    let mut shared = Pink::new();
    for c in 0..2 {
        let ch = out.ch(c);
        let mut own = Pink::new();
        let mut spray = Biquad::with(rate, Filter::Bandpass, 1900.0, 0.5);
        let mut spray_hp = Biquad::with(rate, Filter::Highpass, 500.0, 0.6);
        let mut gurgle = Biquad::with(rate, Filter::Bandpass, 260.0, 1.2);
        for i in 0..n {
            // (The hum and the arm turn with the left; the right hears them where the left left them.)
            if c == 0 {
                phase += (2.0 * PI * 100.0 * (1.0 + 0.006 * wobble[i] as f64)) / rate;
                arm += (2.0 * PI * 0.95) / rate;
            }
            let saw = sin(phase) + 0.5 * sin(2.0 * phase) + 0.25 * sin(3.0 * phase);
            // The jets nearer one side as the arm turns.
            let pass = 0.55 + 0.45 * sin(arm + if c == 1 { 0.6 } else { 0.0 });
            let noise = 0.7 * shared.next(&mut r) + 0.3 * own.next(&mut r);
            ch[i] = (hum.run(saw) * 0.02
                + spray_hp.run(spray.run(noise)) * 0.6 * pass
                + gurgle.run(noise) * 0.25 * (0.6 + 0.4 * wobble[i] as f64))
                as f32;
        }
    }
    // Now and then a jet catches a plate's edge or a cup: a short splash.
    let mut t = 0.5 + r.next();
    while t < out.seconds - 0.4 {
        let seconds = 0.2 + r.next() * 0.15;
        let amp = 0.4 + 0.3 * r.next();
        let pan = (r.next() - 0.5) * 0.3;
        let cutoff = 1400.0 + r.next() * 900.0;
        slosh(&mut out, rate, &mut r, t, seconds, amp, pan, cutoff);
        t += 0.9 + r.next() * 1.6;
    }
    // Shut in, behind the door and the worktop: the highs muffled.
    boxed(&mut out, 2600.0, 90.0);
    finish(out, 0.13, 1.0)
}

/// Loading a dishwasher: plates and mugs set into the rack, a handful of cutlery dropped in the basket, a
/// rack rolled out or in; no water.
pub fn crockery(rate: f64) -> Stereo {
    let mut r = Rng::new(20.0);
    let mut out = track(rate, 12.0, 0.6);
    let mut t = 0.3 + r.next() * 0.4;
    while t < out.seconds {
        let pan = (r.next() - 0.5) * 0.5;
        let what = r.next();
        if what < 0.15 {
            // Cutlery into the basket: a quick rattle.
            let f = 3200.0 + 1800.0 * r.next();
            let (mut k, mut at) = (0.0, t);
            while k < 3.0 + (r.next() * 4.0).floor() {
                let f0 = f * (0.9 + r.next() * 0.2);
                let amp = 0.12 + 0.12 * r.next();
                clink(
                    &mut out,
                    rate,
                    at,
                    f0,
                    amp,
                    pan,
                    &[1.0, 1.53, 2.37, 3.11],
                    0.15,
                    false,
                );
                k += 1.0;
                at += 0.02 + r.next() * 0.05;
            }
            t += 0.8 + r.next() * 1.2;
        } else if what < 0.25 {
            let seconds = 0.45 + r.next() * 0.3;
            roll(&mut out, rate, &mut r, t, seconds, pan);
            t += 1.0 + r.next();
        } else {
            // A plate or a mug knocking the rack as it goes in, sometimes again as it settles.
            let f = 1300.0 + 1500.0 * r.next();
            let amp = 0.2 + 0.35 * pow(r.next(), 2.0);
            clink(
                &mut out,
                rate,
                t,
                f,
                amp,
                pan,
                &[1.0, 2.76, 5.4, 8.93],
                0.07,
                false,
            );
            if r.next() < 0.4 {
                let at = t + 0.06 + r.next() * 0.1;
                let f0 = f * (0.9 + r.next() * 0.2);
                let amp = 0.1 + 0.2 * r.next();
                clink(
                    &mut out,
                    rate,
                    at,
                    f0,
                    amp,
                    pan,
                    &[1.0, 2.76, 5.4, 8.93],
                    0.07,
                    false,
                );
            }
            t += 0.7 + r.next() * 1.3;
        }
    }
    rooms(
        &mut out,
        &[2.9, 4.7, 7.9, 9.1],
        &[3.1, 5.3, 7.1, 9.7],
        0.55,
        0.4,
        0.2,
    );
    finish(out, 0.0, 0.7)
}

/// A fountain heard from beside it: the jet falling back into the basin as a broad, splashing rush, drops
/// pattering all over the water; out of doors, so no echo.
pub fn fountain(rate: f64) -> Stereo {
    let mut r = Rng::new(17.0);
    let mut out = track(rate, 10.0, 0.6);
    stream(
        &mut out,
        &mut r,
        StreamOpts {
            centre: 1100.0,
            q: 0.45,
            bubbles: 600.0,
            gurgle: 0.1,
            low: 500.0,
            high: 3500.0,
            rumble: 0.3,
        },
    );
    // Drops landing all over the basin, each somewhere of its own left to right.
    let mut t = r.next() * 0.01;
    while t < out.seconds {
        let (l, rr) = gains((r.next() - 0.5) * 1.4);
        let amp = 0.06 * (0.2 + powi(r.next(), 3));
        let f = 1000.0 * pow(4.0, r.next());
        tick(&mut out.left, rate, t, amp * 0.5 * l);
        tick(&mut out.right, rate, t, amp * 0.5 * rr);
        let rise = 0.2 + r.next() * 0.4;
        bubble(&mut out.left, rate, t + 0.001, f, amp * l, rise);
        let rise = 0.2 + r.next() * 0.4;
        bubble(&mut out.right, rate, t + 0.001, f, amp * rr, rise);
        t += -log(1.0 - r.next()) / 500.0;
    }
    finish(out, 0.18, 1.0)
}

/// A toilet flushing, played once: the lever's clunk, the cistern emptying into the bowl in a rush that
/// swirls away, the gulp as the trap empties, then the cistern filling again with a thin hiss and a trickle
/// into the bowl until the valve shuts.
pub fn flush(rate: f64) -> Stereo {
    let mut r = Rng::new(19.0);
    let mut out = track(rate, 9.0, 0.0);
    let n = out.left.len();
    let swirl = wander(&mut r, n, rate, 0.25);
    let burble = wander(&mut r, n, rate, 0.04);
    let mut shared = Pink::new();
    // How the rush goes: on at once, falling away as the cistern empties; and the refill after it.
    let rush = |t: f64| {
        if t < 0.15 {
            0.0
        } else {
            min(1.0, (t - 0.15) / 0.12) * exp(-max(0.0, t - 0.9) / 1.1)
        }
    };
    let refill = |t: f64| ramp(2.2, 3.2, t) * (1.0 - ramp(7.4, 8.6, t));
    for c in 0..2 {
        let ch = out.ch(c);
        let mut own = Pink::new();
        let mut jet = Biquad::new(rate);
        let mut low = Biquad::with(rate, Filter::Lowpass, 300.0, 0.8);
        let mut hiss = Biquad::with(rate, Filter::Bandpass, 3400.0, 3.0);
        let mut hp = Biquad::with(rate, Filter::Highpass, 110.0, 0.7);
        let mut deep = Pink::new();
        for i in 0..n {
            let t = i as f64 / rate;
            if i & 31 == 0 {
                jet.set(
                    Filter::Bandpass,
                    900.0 * pow(2.0, 0.5 * swirl[i] as f64 + 0.25 * burble[i] as f64),
                    0.7,
                );
            }
            let w = 0.6 * shared.next(&mut r) + 0.75 * own.next(&mut r);
            let a = rush(t);
            let v = jet.run(w) * 1.6 * a * (1.0 + 0.3 * burble[i] as f64)
                + low.run(deep.next(&mut r)) * 0.9 * a
                + hiss.run(w) * 0.5 * refill(t);
            ch[i] = hp.run(v) as f32;
        }
        // Bubbles in the swirl...
        let mut t = 0.3 + r.next() * 0.01;
        while t < 3.2 {
            let f = 300.0 * pow(2600.0 / 300.0, pow(r.next(), 0.8));
            let amp = 0.18 * rush(t) * (0.2 + pow(r.next(), 2.0));
            let rise = 0.1 + r.next() * 0.3;
            bubble(ch, rate, t, f, amp, rise);
            t += -log(1.0 - r.next()) / 260.0;
        }
        // ...the gulp as the trap empties: a run of big, low bubbles...
        let mut t = 2.3 + r.next() * 0.1;
        for k in 0..7 {
            let f = 160.0 + 260.0 * r.next();
            let rise = 0.4 + r.next() * 0.4;
            bubble(ch, rate, t, f, 0.5 * (1.0 - k as f64 / 9.0), rise);
            t += 0.07 + r.next() * 0.08;
        }
        // ...and the trickle back into the bowl as the cistern fills.
        let mut t = 3.0;
        while t < 8.4 {
            let f = 900.0 + 1500.0 * r.next();
            bubble(ch, rate, t, f, 0.05 * refill(t), 0.3);
            t += -log(1.0 - r.next()) / 30.0;
        }
    }
    rooms(
        &mut out,
        &[12.7, 16.1, 24.7, 27.9],
        &[11.3, 17.9, 23.1, 29.3],
        0.62,
        0.45,
        0.25,
    );
    // The lever: a plastic clunk.
    thud(&mut out, rate, 0.02, 0.3);
    tick(&mut out.left, rate, 0.02, 0.25);
    tick(&mut out.right, rate, 0.021, 0.25);
    finish(out, 0.0, 0.8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_whole_power_is_squared_and_multiplied() {
        assert_eq!(powi(0.3, 0), 1.0);
        assert_eq!(powi(0.3, 2), 0.3 * 0.3);
        assert_eq!(powi(0.3, 3), 0.3 * (0.3 * 0.3));
        assert_eq!(powi(0.72, 5), 0.72 * ((0.72 * 0.72) * (0.72 * 0.72)));
    }
}
