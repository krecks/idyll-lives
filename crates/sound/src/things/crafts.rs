//! The rest: a workbench and a bike's tools, a chemistry set bubbling, a punch bag, crockery smashed in a bin, a cello and someone singing.

use super::*;

// The rest.

/// A workbench: a nail hammered in (the steel's ring rising as it goes in, then the dull last blows on the
/// wood), a board sawn in strokes (the teeth catching, the push louder than the pull), the cut-off dropping,
/// sanding, a screwdriver's ratchet, a tool put down.
pub(crate) fn tools(rate: f64) -> Stereo {
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
pub(crate) fn saw(out: &mut Ev, r: &mut Rng, at: f64, seconds: f64, amp: f64, pan: f64) {
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
pub(crate) fn bike_tools(rate: f64) -> Stereo {
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
pub(crate) fn punch_bag(rate: f64) -> Stereo {
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
pub(crate) fn smash(rate: f64) -> Stereo {
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
pub(crate) fn cello(rate: f64) -> Stereo {
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
pub(crate) fn singing(rate: f64) -> Stereo {
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
pub(crate) fn formants(vowel: char) -> [f64; 3] {
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
pub(crate) fn sing(
    d: &mut [f32],
    rate: f64,
    r: &mut Rng,
    at: f64,
    seconds: f64,
    hz: f64,
    vowel: char,
) {
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
pub(crate) fn pad(
    d: &mut [f32],
    rate: f64,
    at: f64,
    seconds: f64,
    hz: f64,
    amp: f64,
    harmonics: usize,
) {
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
