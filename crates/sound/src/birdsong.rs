//! The neighbourhood birds' songs (was `web/src/audio/birdsong.ts`; `world.ts` decides when they sing): the
//! species, and how each version of a bird's song is made.

use crate::dsp::m::*;
use crate::dsp::{Rng, add};

/// One note of a song.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    /// Seconds, and seconds of silence after it.
    pub len: f64,
    pub gap: f64,
    /// Pitch through the note in multiples of the bird's own pitch, spread evenly over the note and joined in a
    /// smooth curve.
    pub pitch: Vec<f64>,
    pub level: f64,
}

fn note(len: f64, gap: f64, pitch: Vec<f64>, level: f64) -> Note {
    Note {
        len,
        gap,
        pitch,
        level,
    }
}

/// A species, as far as its song is made (the TypeScript's `Species` also held when and how loud it sings: the
/// player's).
#[derive(Clone, Copy, Debug)]
pub struct Species {
    pub name: &'static str,
    /// A fast warble on every note: rate (Hz) and depth (part of the pitch); the level dips with it by `flutter`.
    pub vibrato: (f64, f64),
    pub flutter: f64,
    /// Level of the 2nd harmonic (a third of it for the 3rd).
    pub overtone: f64,
    /// Seconds to fade each note in and out (less on short notes).
    pub fade: f64,
    /// Highest frequencies kept (Hz), before distance takes more off.
    pub bright: f64,
    /// One song; the generator varies it a little, so each bird has a few versions of its own.
    pub song: fn(&mut Rng) -> Vec<Note>,
}

/// The species (`SPECIES`).
pub const SPECIES: [Species; 5] = [
    // "Tee-cher tee-cher tee-cher", like a great tit.
    Species {
        name: "tit",
        vibrato: (34.0, 0.004),
        flutter: 0.06,
        overtone: 0.05,
        fade: 0.018,
        bright: 7000.0,
        song: tit,
    },
    // A run of quick notes falling in steps, then a flourish, like a chaffinch.
    Species {
        name: "finch",
        vibrato: (26.0, 0.006),
        flutter: 0.1,
        overtone: 0.09,
        fade: 0.008,
        bright: 8000.0,
        song: finch,
    },
    // A soft cascade of liquid notes running down the scale, like a willow warbler.
    Species {
        name: "warbler",
        vibrato: (21.0, 0.02),
        flutter: 0.3,
        overtone: 0.12,
        fade: 0.014,
        bright: 6500.0,
        song: warbler,
    },
    // A blackbird: an unhurried, fluting phrase of mellow slurred whistles, never quite the same twice, trailing
    // off into a soft, high twitter.
    Species {
        name: "blackbird",
        vibrato: (9.0, 0.008),
        flutter: 0.08,
        overtone: 0.05,
        fade: 0.03,
        bright: 5000.0,
        song: blackbird,
    },
    // "Coo-COOO-coo", low and soft, like a collared dove; now and then.
    Species {
        name: "dove",
        vibrato: (7.0, 0.005),
        flutter: 0.15,
        overtone: 0.15,
        fade: 0.06,
        bright: 1600.0,
        song: dove,
    },
];

fn tit(r: &mut Rng) -> Vec<Note> {
    let mut notes = Vec::new();
    let pairs = 3.0 + (r.next() * 3.0).floor();
    let mut i = 0.0;
    while i < pairs {
        let level = if i == pairs - 1.0 { 0.85 } else { 1.0 };
        notes.push(note(0.11, 0.04, vec![1.04, 1.01, 0.98], level));
        notes.push(note(0.13, 0.09, vec![0.8, 0.74, 0.7, 0.68], 0.8 * level));
        i += 1.0;
    }
    notes
}

fn finch(r: &mut Rng) -> Vec<Note> {
    let mut notes = Vec::new();
    // (count, pitch, len, gap)
    let a = 5.0 + (r.next() * 3.0).floor();
    let b = 4.0 + (r.next() * 2.0).floor();
    let c = 3.0 + (r.next() * 2.0).floor();
    let steps = [
        (a, 1.3, 0.042, 0.032),
        (b, 1.1, 0.05, 0.03),
        (c, 0.94, 0.06, 0.032),
    ];
    for (s, &(count, pitch, len, gap)) in steps.iter().enumerate() {
        let mut i = 0.0;
        while i < count {
            let p = pitch * (1.0 - 0.01 * i);
            notes.push(note(
                len,
                gap,
                vec![p * 1.12, p, p * 0.9],
                0.65 + 0.12 * s as f64,
            ));
            i += 1.0;
        }
    }
    notes.push(note(0.09, 0.03, vec![1.0, 0.86, 0.8], 0.9));
    notes.push(note(0.26, 0.0, vec![0.9, 1.2, 1.3, 1.1, 0.78], 1.0));
    notes
}

fn warbler(r: &mut Rng) -> Vec<Note> {
    let count = 11.0 + (r.next() * 4.0).floor();
    (0..count as usize)
        .map(|i| {
            let i = i as f64;
            let x = i / (count - 1.0);
            let p = 1.5 - 0.6 * x;
            let last = i >= count - 2.0;
            let level = min(1.0, 0.45 + 1.2 * x) * if last { 0.9 } else { 1.0 };
            if last {
                note(0.12, 0.04, vec![p, p * 1.08, p * 0.92], level)
            } else {
                note(0.075, 0.035, vec![p * 0.97, p * 1.04, p * 0.98], level)
            }
        })
        .collect()
}

fn blackbird(r: &mut Rng) -> Vec<Note> {
    // Slurs it puts its phrases together from: rising, falling, a dip, a turn.
    let slurs: [&[f64]; 8] = [
        &[1.0, 1.12, 1.06],
        &[0.84, 0.94, 1.0],
        &[1.22, 1.1, 0.9],
        &[0.9, 1.24, 1.2, 1.02],
        &[1.02, 0.8, 0.82],
        &[1.34, 1.3, 1.18],
        &[0.95, 1.06, 0.98, 1.1],
        &[0.76, 0.86, 0.8],
    ];
    let mut notes = Vec::new();
    let count = 3.0 + (r.next() * 4.0).floor();
    let mut i = 0.0;
    while i < count {
        let slur = slurs[(r.next() * slurs.len() as f64).floor() as usize];
        let key = 0.9 + r.next() * 0.25;
        let last = i == count - 1.0;
        let len = 0.14 + r.next() * 0.18;
        let gap = if last { 0.05 } else { 0.03 + r.next() * 0.07 };
        let pitch = slur.iter().map(|p| p * key).collect();
        let level = 0.75 + r.next() * 0.25;
        notes.push(note(len, gap, pitch, level));
        i += 1.0;
    }
    // The twitter: a few quick, thin high notes, quieter.
    let twitter = (r.next() * 6.0).floor();
    let mut i = 0.0;
    while i < twitter {
        let p = 1.8 + r.next() * 0.5;
        let len = 0.03 + r.next() * 0.02;
        let pitch = vec![p * 1.08, p, p * 1.12];
        let level = 0.3 + r.next() * 0.12;
        notes.push(note(len, 0.018, pitch, level));
        i += 1.0;
    }
    notes
}

fn dove(_: &mut Rng) -> Vec<Note> {
    vec![
        note(0.2, 0.08, vec![0.96, 1.0, 0.97], 0.8),
        note(0.55, 0.12, vec![0.98, 1.06, 1.06, 1.02, 0.94], 1.0),
        note(0.3, 0.0, vec![0.95, 0.97, 0.88], 0.75),
    ]
}

/// What makes one bird's song its own (`Voice`).
#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    /// Its species' name.
    pub species: String,
    pub seed: f64,
    /// Its own pitch (Hz), pace (1 = as written) and nearness (1 = nearest).
    pub pitch: f64,
    pub pace: f64,
    pub near: f64,
}

/// Versions of its song each bird has.
pub const VERSIONS: f64 = 3.0;

/// One version of a bird's song at a sample rate (`songSamples`): each note a sine following its pitch curve
/// with a fast warble and a quiet overtone, faded in and out, then softened and given a faint echo as if from a
/// garden away. None: no such species.
pub fn song(bird: &Voice, version: u32, rate: f64) -> Option<Vec<f32>> {
    let sp = SPECIES.iter().find(|s| s.name == bird.species)?;
    let seed =
        to_i32(bird.seed * VERSIONS + version as f64 + 1.0).wrapping_mul(0x9e37_79b1_u32 as i32);
    let mut r = Rng::new(seed as f64);
    let notes = (sp.song)(&mut r);
    let shift = 1.0 + (r.next() - 0.5) * 0.03;
    let mut seconds = 0.3;
    for n in &notes {
        seconds += (n.len * 1.04 + n.gap) * bird.pace;
    }
    let mut d = vec![0.0f32; (seconds * rate).ceil() as usize];
    let (wobble, depth) = sp.vibrato;
    let mut at = 0.0;
    for n in &notes {
        let len = n.len * bird.pace * (1.0 + (r.next() - 0.5) * 0.08);
        let count = round_usize(len * rate);
        let fade = min(sp.fade, len / 3.0);
        let tune = bird.pitch * shift * (1.0 + (r.next() - 0.5) * 0.02);
        let speed = (2.0 * PI * wobble * (0.9 + r.next() * 0.2)) / rate;
        let start = round_usize(at * rate);
        let mut phase = 0.0;
        let mut vib = r.next() * 2.0 * PI;
        let mut j = 0;
        while j < count && start + j < d.len() {
            let time = j as f64 / rate;
            let w = sin(vib);
            phase +=
                (2.0 * PI * tune * curve(&n.pitch, j as f64 / count as f64) * (1.0 + depth * w))
                    / rate;
            vib += speed;
            let edge = min(min(1.0, time / fade), (len - time) / fade);
            let amp = (0.5 - 0.5 * cos(PI * edge)) * n.level * (1.0 - sp.flutter * (0.5 + 0.5 * w));
            add(
                &mut d,
                start + j,
                amp * (sin(phase) + sp.overtone * (sin(2.0 * phase) + sin(3.0 * phase) / 3.0)),
            );
            j += 1;
        }
        at += len + n.gap * bird.pace;
    }
    distance(
        &mut d,
        rate,
        sp.bright * (0.7 + 0.3 * bird.near),
        1.0 - bird.near,
    );
    Some(d)
}

/// A smooth curve through evenly spaced points, at `x` from 0 to 1.
fn curve(points: &[f64], x: f64) -> f64 {
    let n = points.len();
    if n == 1 {
        return points[0];
    }
    let u = x * (n - 1) as f64;
    let i = ((n - 2) as f64).min(u.floor()) as usize;
    let f = u - i as f64;
    let p0 = points[i.saturating_sub(1)];
    let p1 = points[i];
    let p2 = points[i + 1];
    let p3 = points[(n - 1).min(i + 2)];
    p1 + 0.5
        * f
        * (p2 - p0 + f * (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3 + f * (3.0 * (p1 - p2) + p3 - p0)))
}

/// Softens a sound as if from across the gardens: highs rolled off, and faint echoes off nearby walls.
fn distance(d: &mut [f32], rate: f64, cutoff: f64, far: f64) {
    let a = 1.0 - exp((-2.0 * PI * cutoff) / rate);
    let mut lp = 0.0;
    for v in d.iter_mut() {
        lp += (*v as f64 - lp) * a;
        *v = lp as f32;
    }
    let taps: Vec<(usize, f64)> = [0.023, 0.037, 0.053, 0.071, 0.097, 0.127, 0.163, 0.211]
        .iter()
        .enumerate()
        .map(|(i, &delay)| {
            (
                round_usize(delay * rate),
                (if i % 2 == 1 { -0.07 } else { 0.08 }) * exp(-delay / 0.09) * (0.6 + far),
            )
        })
        .collect();
    // From the end back, so echoes don't echo again.
    for i in (0..d.len()).rev() {
        for &(k, level) in &taps {
            if i >= k {
                let v = d[i - k] as f64 * level;
                add(d, i, v);
            }
        }
    }
}
