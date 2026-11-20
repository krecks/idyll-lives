//! The games: pool, foosball, ping-pong, darts, an arcade cabinet and basketball.

use super::*;

// The games.

/// Pool: a shot every few seconds. The cue's leather tip on the cue ball (a dull tock), the balls clacking
/// (phenolic resin: a click of a few milliseconds, most of it 3 to 6 kHz), a cushion's thump, the balls
/// rolling on the cloth, now and then one dropping into a pocket; a cue chalked.
pub(super) fn pool(rate: f64) -> Stereo {
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
pub(super) fn ball(out: &mut Ev, at: f64, amp: f64, pan: f64) {
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
pub(super) fn foosball(rate: f64) -> Stereo {
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
pub(super) enum Play {
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
pub(super) fn ping_pong(rate: f64) -> Stereo {
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
pub(super) fn bat(out: &mut Ev, at: f64, amp: f64, pan: f64) {
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
pub(super) fn bounce(out: &mut Ev, at: f64, amp: f64, pan: f64) {
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
pub(super) fn darts(rate: f64) -> Stereo {
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
pub(super) fn arcade(rate: f64) -> Stereo {
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
pub(super) fn chip(d: &mut [f32], rate: f64, at: f64, seconds: f64, hz: f64, duty: f64, amp: f64) {
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
pub(super) fn tri(d: &mut [f32], rate: f64, at: f64, seconds: f64, hz: f64, amp: f64) {
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
pub(super) fn zap(out: &mut Ev, at: f64, from: f64, amp: f64) {
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
pub(super) fn basketball(rate: f64) -> Stereo {
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
pub(super) fn dribble(out: &mut Ev, at: f64, amp: f64, pan: f64) {
    strike(
        out,
        at,
        amp,
        pan,
        &[[110.0, 0.03, 0.8], [950.0, 0.05, 0.5], [1520.0, 0.03, 0.3]],
        burst(1500.0, 0.6, 0.003, 0.8),
    );
}
