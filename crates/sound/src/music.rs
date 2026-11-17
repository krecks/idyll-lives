//! Music for the stereo, the radio and dancing, made in code (no sound files; was `web/src/audio/music.ts`): a
//! few pieces half a minute long, each written by a small composer from a seed (`score`: the same seed, the same
//! piece) and played on a handful of synthesised voices (`perform`), as a loop whose last notes ring on into its
//! start.
//!
//! Three plain styles:
//! - `Folk`: a plucked tune over a walking bass and an off-beat strum, 100 beats a minute, in G;
//! - `Dance`: a kick on every beat, hats between, a clap on two and four, a pumping bass and a plucked arpeggio
//!   over held chords, 120 beats a minute, in A minor;
//! - `Slow`: a waltz, soft held chords and a tune of long plucked notes, 72 beats a minute, in F.
//!
//! A piece is sixteen bars in four phrases, the first tune coming back (A A B A) and ending on the key's own note,
//! over a chord a bar from a few well-worn progressions; the tune moves by step along a five-note scale and lands
//! on a note of the chord on the strong beats.

use crate::dsp::m::*;
use crate::dsp::*;

/// A piece's style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    Folk,
    Dance,
    Slow,
}

/// What plays a note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Voice {
    Pluck,
    Strum,
    Bass,
    Pad,
    Kick,
    Clap,
    Hat,
}

/// A note of a piece: when (beats from its start) and how long (beats), its pitch (MIDI: 69 is the A at 440 Hz),
/// which voice plays it, how loud (0..1) and where (-1 left .. 1 right).
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub at: f64,
    pub len: f64,
    pub midi: i64,
    pub voice: Voice,
    pub level: f64,
    pub pan: f64,
}

/// A piece as written: its style, pace, length, key and notes.
#[derive(Clone, Debug, PartialEq)]
pub struct Score {
    pub style: Style,
    /// Beats a minute, beats a bar, bars.
    pub bpm: f64,
    pub meter: usize,
    pub bars: usize,
    /// The key's note (MIDI) and whether it is minor.
    pub key: i64,
    pub minor: bool,
    /// The chord of each bar: its root, as a step of the key's scale (0: the key's own chord).
    pub chords: Vec<i64>,
    pub notes: Vec<Note>,
}

/// What a style is made of.
struct StyleOf {
    bpm: f64,
    meter: usize,
    key: i64,
    minor: bool,
    progressions: [[i64; 4]; 3],
    bridge: [[i64; 4]; 2],
}

/// The styles (`STYLES`, in its order: the order is in each piece's seed). Chords by scale step: 0 the key's,
/// 3 the fourth, 4 the fifth, 5 the sixth.
const STYLES: [(Style, StyleOf); 3] = [
    (
        Style::Folk,
        StyleOf {
            bpm: 100.0,
            meter: 4,
            key: 55,
            minor: false,
            progressions: [[0, 3, 0, 4], [0, 5, 3, 4], [0, 3, 4, 0]],
            bridge: [[3, 0, 3, 4], [5, 3, 0, 4]],
        },
    ),
    (
        Style::Dance,
        StyleOf {
            bpm: 120.0,
            meter: 4,
            key: 57,
            minor: true,
            progressions: [[0, 5, 2, 6], [0, 3, 5, 6], [0, 5, 3, 4]],
            bridge: [[5, 6, 0, 0], [3, 5, 6, 6]],
        },
    ),
    (
        Style::Slow,
        StyleOf {
            bpm: 72.0,
            meter: 3,
            key: 53,
            minor: false,
            progressions: [[0, 5, 3, 4], [0, 3, 0, 4], [0, 4, 5, 3]],
            bridge: [[3, 4, 0, 5], [5, 3, 4, 4]],
        },
    ),
];

const MAJOR: [i64; 7] = [0, 2, 4, 5, 7, 9, 11];
const MINOR: [i64; 7] = [0, 2, 3, 5, 7, 8, 10];
/// The five notes a tune keeps to (steps of the seven-note scale): no half steps, so nothing it plays grates.
const FIVE: [i64; 5] = [0, 1, 2, 4, 5];
const FIVE_MINOR: [i64; 5] = [0, 2, 3, 4, 6];

/// The pitch (semitones over the key's note) of scale step `step` (any octave: 7 is the key's note an octave up).
fn pitch(step: i64, minor: bool) -> i64 {
    let scale = if minor { MINOR } else { MAJOR };
    let octave = step.div_euclid(7);
    octave * 12 + scale[(step - octave * 7) as usize]
}

/// The three notes of the chord on scale step `root`, as scale steps.
fn triad(root: i64) -> [i64; 3] {
    [root, root + 2, root + 4]
}

/// The one of `among` nearest `to` (the first of two as near).
fn nearest(to: i64, among: &[i64]) -> i64 {
    among[1..].iter().fold(among[0], |best, &d| {
        if (d - to).abs() < (best - to).abs() {
            d
        } else {
            best
        }
    })
}

/// A note of the tune before it is placed: its bar, where it falls in the bar (in `grid`s of a beat), how long
/// (beats) and its rung of the ladder.
#[derive(Clone, Copy)]
struct Tuned {
    bar: usize,
    slot: usize,
    len: f64,
    rung: i64,
}

/// A piece in a style from a seed: the same seed, the same piece.
pub fn score(style: Style, seed: u32) -> Score {
    let index = STYLES.iter().position(|(k, _)| *k == style).unwrap();
    let s = &STYLES[index].1;
    let mut r = Rng::new(0x5c0e as f64 + seed as f64 * 977.0 + index as f64 * 131.0);
    let pick = |r: &mut Rng, n: usize| (r.next() * n as f64).floor() as usize;
    let main = s.progressions[pick(&mut r, 3)];
    let bridge = s.bridge[pick(&mut r, 2)];
    let chords: Vec<i64> = [main, main, bridge, main].concat();
    let bars = chords.len();
    let mut notes: Vec<Note> = Vec::new();
    let five = if s.minor { FIVE_MINOR } else { FIVE };
    // The tune's notes to choose from: the five-note scale over two octaves, as scale steps.
    let ladder: Vec<i64> = (0..3)
        .flat_map(|o| five.iter().map(move |d| d + 7 * o))
        .filter(|&d| d <= 16)
        .collect();
    let chord_rungs = |root: i64| -> Vec<i64> {
        ladder
            .iter()
            .copied()
            .filter(|d| triad(root).iter().any(|c| (d - c) % 7 == 0))
            .collect()
    };

    // One phrase's tune (four bars): where each note falls in its bar (in `grid`s of a beat) and its rung of the
    // ladder.
    let slow = style == Style::Slow;
    let slots = s.meter * if slow { 1 } else { 2 };
    let grid = if slow { 1.0 } else { 0.5 };
    let phrase = |r: &mut Rng, bar0: usize, from: i64| -> Vec<Tuned> {
        let mut out = Vec::new();
        let mut at = nearest(from, &ladder);
        for b in 0..4 {
            let rungs = chord_rungs(chords[bar0 + b]);
            // A rhythm for the bar: which slots start a note (the first always; a slow tune holds its notes).
            let mut starts = vec![0];
            for k in 1..slots {
                if r.next()
                    < (if slow {
                        0.45
                    } else if k % 2 == 0 {
                        0.75
                    } else {
                        0.4
                    })
                {
                    starts.push(k);
                }
            }
            for (i, &slot) in starts.iter().enumerate() {
                let strong = slot == 0 || (!slow && slot * 2 == slots);
                // On a strong beat a note of the chord near where the tune is; between, a step up or down, drawn
                // back to the middle.
                let index = ladder
                    .iter()
                    .position(|&d| d == at)
                    .map_or(-1, |p| p as i64);
                // (The draw only when the tune is in the middle.)
                let pull = if at > 9 {
                    -1
                } else if at < 5 || r.next() < 0.5 {
                    1
                } else {
                    -1
                };
                at = if strong {
                    let leap = if r.next() < 0.3 { pull * 2 } else { 0 };
                    nearest(at + leap, &rungs)
                } else {
                    let step = if r.next() < 0.75 { pull } else { -pull };
                    ladder[(index + step).clamp(0, ladder.len() as i64 - 1) as usize]
                };
                let next = starts.get(i + 1).copied().unwrap_or(slots);
                out.push(Tuned {
                    bar: b,
                    slot,
                    len: (next - slot) as f64 * grid,
                    rung: at,
                });
            }
        }
        out
    };
    let a = phrase(&mut r, 0, 7);
    let b = phrase(&mut r, 8, a[a.len() - 1].rung + 2);
    // The first phrase three times (bars 1-4, 5-8, 13-16), the bridge between; the last note of the piece is the
    // key's own.
    let mut tune: Vec<Tuned> = Vec::new();
    for (n, part) in [&a, &a, &b, &a].into_iter().enumerate() {
        for t in part {
            tune.push(Tuned {
                bar: t.bar + n * 4,
                ..*t
            });
        }
    }
    let last = tune.len() - 1;
    tune[last].rung = nearest(tune[last].rung, &[0, 7, 14]);
    for t in &tune {
        notes.push(Note {
            at: (t.bar * s.meter) as f64 + t.slot as f64 * grid,
            len: t.len,
            midi: s.key + 12 + pitch(t.rung, s.minor),
            voice: Voice::Pluck,
            level: 0.5 + if t.slot == 0 { 0.1 } else { 0.0 },
            pan: 0.15,
        });
    }

    // Under the tune, bar by bar.
    let note = |at: f64, len: f64, midi: i64, voice: Voice, level: f64, pan: f64| Note {
        at,
        len,
        midi,
        voice,
        level,
        pan,
    };
    for (bar, &root) in chords.iter().enumerate() {
        let at = (bar * s.meter) as f64;
        let low = s.key - 12 + pitch(root, s.minor) - if pitch(root, s.minor) > 7 { 12 } else { 0 };
        let chord = triad(root).map(|d| s.key + pitch(d, s.minor));
        match style {
            Style::Folk => {
                // The bass on one and three (the chord's note, then its fifth), a strum of the chord on two and four.
                notes.push(note(at, 2.0, low, Voice::Bass, 0.6, -0.1));
                notes.push(note(at + 2.0, 2.0, low + 7, Voice::Bass, 0.5, -0.1));
                for beat in [1.0, 3.0] {
                    for (i, &midi) in chord.iter().enumerate() {
                        notes.push(note(
                            at + beat + i as f64 * 0.04,
                            0.5,
                            midi,
                            Voice::Strum,
                            0.2,
                            -0.35,
                        ));
                    }
                }
            }
            Style::Dance => {
                // A kick on every beat, hats between, a clap on two and four; the bass on the off-beats; the chord
                // held, and run up and down in sixteenths.
                for beat in 0..4 {
                    let b = beat as f64;
                    notes.push(note(at + b, 0.5, 36, Voice::Kick, 0.9, 0.0));
                    notes.push(note(at + b + 0.5, 0.25, 90, Voice::Hat, 0.25, 0.3));
                    notes.push(note(
                        at + b + 0.5,
                        0.5,
                        low + if beat == 3 { 12 } else { 0 },
                        Voice::Bass,
                        0.55,
                        0.0,
                    ));
                    if beat % 2 == 1 {
                        notes.push(note(at + b, 0.5, 60, Voice::Clap, 0.4, -0.1));
                    }
                }
                for &midi in &chord {
                    notes.push(note(at, 4.0, midi, Voice::Pad, 0.16, 0.0));
                }
                let run = [0, 1, 2, 1];
                for k in 0..16 {
                    let level = 0.16 + if k % 4 == 0 { 0.05 } else { 0.0 };
                    let pan = if k % 2 == 1 { -0.4 } else { -0.2 };
                    notes.push(note(
                        at + k as f64 * 0.25,
                        0.25,
                        chord[run[k % 4]] + 12,
                        Voice::Strum,
                        level,
                        pan,
                    ));
                }
            }
            Style::Slow => {
                // A waltz: the bass on one, the chord (soft, held) under the bar, two light plucks of it on two and
                // three.
                notes.push(note(at, 3.0, low, Voice::Bass, 0.5, -0.1));
                for &midi in &chord {
                    notes.push(note(at, 3.0, midi, Voice::Pad, 0.2, 0.0));
                }
                for beat in [1, 2] {
                    notes.push(note(
                        at + beat as f64,
                        1.0,
                        chord[beat],
                        Voice::Strum,
                        0.18,
                        -0.3,
                    ));
                }
            }
        }
    }
    // (A stable sort, as the TypeScript's.)
    notes.sort_by(|x, y| x.at.total_cmp(&y.at));
    Score {
        style,
        bpm: s.bpm,
        meter: s.meter,
        bars,
        key: s.key,
        minor: s.minor,
        chords,
        notes,
    }
}

/// How long a piece lasts (s).
pub fn seconds(piece: &Score) -> f64 {
    (piece.bars * piece.meter) as f64 * 60.0 / piece.bpm
}

/// A note's frequency (Hz).
fn hz(midi: i64) -> f64 {
    440.0 * exp2((midi - 69) as f64 / 12.0)
}

/// `2 ** y` as the TypeScript's sounds were made (`tools/audio/dump.ts` under Bun: JavaScriptCore's `Math.pow`,
/// which is correctly rounded here where `libm`'s `pow` is a last bit off for a third of the notes, and takes
/// `y` = ±0.5 as `sqrt`, `1 / sqrt`). Worked in double-double precision: 2^k times exp(r ln 2) by its series.
fn exp2(y: f64) -> f64 {
    if y == 0.5 {
        return sqrt(2.0);
    }
    if y == -0.5 {
        return 1.0 / sqrt(2.0);
    }
    // ln 2, as a double and what it leaves over.
    const LN2: (f64, f64) = (core::f64::consts::LN_2, 2.3190468138462996e-17);
    let fast = |a: f64, b: f64| {
        let s = a + b;
        (s, b - (s - a))
    };
    let sum = |a: (f64, f64), b: (f64, f64)| {
        let s = a.0 + b.0;
        let v = s - a.0;
        let e = (a.0 - (s - v)) + (b.0 - v);
        fast(s, e + a.1 + b.1)
    };
    let times = |a: (f64, f64), b: (f64, f64)| {
        let p = a.0 * b.0;
        fast(p, libm::fma(a.0, b.0, -p) + a.0 * b.1 + a.1 * b.0)
    };
    let over = |a: (f64, f64), n: f64| {
        let q = a.0 / n;
        fast(q, (libm::fma(-q, n, a.0) + a.1) / n)
    };
    let k = y.round();
    let r = y - k;
    let z = times((r, 0.0), LN2);
    let (mut total, mut term) = ((1.0, 0.0), (1.0, 0.0));
    for n in 1..=30 {
        term = over(times(term, z), n as f64);
        total = sum(total, term);
    }
    total.0 * pow(2.0, k)
}

/// A piece played, as a loop of two channels at a sample rate: every note on its voice at its place, a note's
/// tail past the end ringing on into the start, then the whole through the cloth of a loudspeaker (a low-pass)
/// and brought to an even level.
pub fn perform(piece: &Score, rate: f64) -> Stereo {
    let n = round_usize(seconds(piece) * rate);
    let mut out = Stereo {
        left: vec![0.0; n],
        right: vec![0.0; n],
    };
    let beat = 60.0 / piece.bpm;
    let mut r = Rng::new(0xbea7 as f64 + piece.notes.len() as f64);
    for note in &piece.notes {
        let (l, rr) = gains(note.pan);
        let start = round_usize(note.at * beat * rate);
        let held = note.len * beat;
        let f = hz(note.midi);
        // Adds the voice's sound, `sample(t)` for `secs`, round the loop's end.
        let mut add_voice = |secs: f64, mut sample: Box<dyn FnMut(f64) -> f64 + '_>| {
            let count = round_usize(secs * rate);
            for j in 0..count {
                let v = note.level * sample(j as f64 / rate);
                let i = (start + j) % n;
                add(&mut out.left, i, v * l);
                add(&mut out.right, i, v * rr);
            }
        };
        match note.voice {
            Voice::Pluck | Voice::Strum => {
                // A plucked string: its tone and the octave over it, dying away (sooner for a strum's).
                let decay = if note.voice == Voice::Pluck {
                    0.16 + 0.12 * min(2.0, held)
                } else {
                    0.14
                };
                add_voice(
                    decay * 5.0,
                    Box::new(|t| {
                        exp(-t / decay)
                            * min(1.0, t / 0.002)
                            * (sin(2.0 * PI * f * t)
                                + 0.3 * sin(4.0 * PI * f * t)
                                + 0.08 * sin(6.0 * PI * f * t))
                    }),
                );
            }
            Voice::Bass => add_voice(
                min(2.5, held + 0.6),
                Box::new(|t| {
                    exp(-t / 0.5)
                        * min(1.0, t / 0.004)
                        * min(1.0, (held + 0.6 - t) / 0.1)
                        * (sin(2.0 * PI * f * t) + 0.3 * sin(4.0 * PI * f * t))
                }),
            ),
            // A soft held tone: three voices a hair apart, in over a third of a second and out again.
            Voice::Pad => add_voice(
                held + 0.5,
                Box::new(|t| {
                    min(1.0, t / 0.35)
                        * min(1.0, max(0.0, held + 0.5 - t) / 0.5)
                        * (sin(2.0 * PI * f * t)
                            + sin(2.0 * PI * f * 1.004 * t)
                            + sin(2.0 * PI * f * 0.996 * t)
                            + 0.25 * sin(4.0 * PI * f * t))
                        * 0.33
                }),
            ),
            Voice::Kick => {
                let mut phase = 0.0;
                add_voice(
                    0.5,
                    Box::new(move |t| {
                        phase += (2.0 * PI * (48.0 + 72.0 * exp(-t / 0.03))) / rate;
                        exp(-t / 0.14) * sin(phase)
                    }),
                );
            }
            Voice::Clap => {
                let mut last_noise = 0.0;
                let r = &mut r;
                add_voice(
                    0.3,
                    Box::new(move |t| {
                        let w = r.next() * 2.0 - 1.0;
                        let high = w - last_noise;
                        last_noise = w;
                        0.5 * high * exp(-t / 0.06)
                            + 0.3 * sin(2.0 * PI * 190.0 * t) * exp(-t / 0.03)
                    }),
                );
            }
            Voice::Hat => {
                let mut last_noise = 0.0;
                let r = &mut r;
                add_voice(
                    0.15,
                    Box::new(move |t| {
                        let w = r.next() * 2.0 - 1.0;
                        let high = w - last_noise;
                        last_noise = w;
                        high * exp(-t / 0.025)
                    }),
                );
            }
        }
    }
    // Through a loudspeaker's cloth: the highs softened (a dance's less, for its hats), run round the loop.
    let cutoff = if piece.style == Style::Dance {
        5200.0
    } else {
        3200.0
    };
    let played = Stereo {
        left: node_filter(&out.left, rate, Filter::Lowpass, cutoff, -3.01, true),
        right: node_filter(&out.right, rate, Filter::Lowpass, cutoff, -3.01, true),
    };
    set_level(played, 0.12, 1.0)
}

/// The pieces the game plays, by name: each its style and seed.
pub const PIECES: [(&str, Style, u32); 3] = [
    ("musicFolk", Style::Folk, 3),
    ("musicDance", Style::Dance, 5),
    ("musicSlow", Style::Slow, 2),
];

/// A piece by name (`PIECES`), played (`MUSIC`).
pub fn make(kind: &str, rate: f64) -> Option<Stereo> {
    let &(_, style, seed) = PIECES.iter().find(|(name, _, _)| *name == kind)?;
    Some(perform(&score(style, seed), rate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const RATE: f64 = 48000.0;
    const STYLES: [Style; 3] = [Style::Folk, Style::Dance, Style::Slow];

    fn tune_of(s: &Score) -> Vec<&Note> {
        s.notes.iter().filter(|n| n.voice == Voice::Pluck).collect()
    }

    // A piece as written.

    #[test]
    fn the_same_seed_the_same_piece_another_seed_another_tune() {
        for style in STYLES {
            assert_eq!(score(style, 4), score(style, 4));
            let tunes: HashSet<Vec<(u64, i64)>> = (1..=6)
                .map(|seed| {
                    tune_of(&score(style, seed))
                        .iter()
                        .map(|n| (n.at.to_bits(), n.midi))
                        .collect()
                })
                .collect();
            assert!(tunes.len() >= 5, "{style:?}: {} tunes", tunes.len());
        }
    }

    #[test]
    fn sixteen_bars_a_chord_a_bar_every_note_on_the_grid_and_inside_the_piece() {
        for style in STYLES {
            for seed in [1, 2, 3] {
                let s = score(style, seed);
                assert_eq!(s.bars, 16);
                assert_eq!(s.chords.len(), 16);
                let beats = (s.bars * s.meter) as f64;
                for n in &s.notes {
                    assert!(n.at >= 0.0 && n.at < beats);
                    assert!(n.len > 0.0);
                    assert!(n.level > 0.0 && n.level <= 1.0);
                    assert!(n.pan.abs() <= 1.0);
                    // Sixteenths at the finest (a strummed chord's strings a hair apart).
                    if n.voice != Voice::Strum {
                        assert!((n.at * 4.0 - js_round(n.at * 4.0)).abs() < 1e-9);
                    }
                }
                // The first phrase's chords come back for the second and the last.
                assert_eq!(s.chords[4..8], s.chords[0..4]);
                assert_eq!(s.chords[12..16], s.chords[0..4]);
                assert_eq!(s.chords[0], 0);
            }
        }
    }

    #[test]
    fn the_tune_keeps_to_the_keys_five_notes_comes_back_and_ends_on_the_keys_own_note() {
        for style in STYLES {
            for seed in [1, 2, 3, 4] {
                let s = score(style, seed);
                let tune = tune_of(&s);
                assert!(tune.len() > 16);
                let five = if s.minor {
                    [0, 3, 5, 7, 10]
                } else {
                    [0, 2, 4, 7, 9]
                };
                for n in &tune {
                    assert!(five.contains(&(n.midi - s.key).rem_euclid(12)));
                }
                // Within an octave and a half of where it starts: singable.
                let high = tune.iter().map(|n| n.midi).max().unwrap();
                let low = tune.iter().map(|n| n.midi).min().unwrap();
                assert!(high - low <= 24);
                // Bars 5-8 are bars 1-4 again.
                let bar = s.meter as f64;
                let phrase = |from: f64| -> Vec<(f64, i64)> {
                    tune.iter()
                        .filter(|n| n.at >= from * bar && n.at < (from + 4.0) * bar)
                        .map(|n| (n.at - from * bar, n.midi))
                        .collect()
                };
                assert_eq!(phrase(4.0), phrase(0.0));
                assert_ne!(phrase(8.0), phrase(0.0));
                assert_eq!((tune[tune.len() - 1].midi - s.key) % 12, 0);
            }
        }
    }

    #[test]
    fn under_the_tune_a_dance_has_its_kick_on_every_beat_and_its_clap_on_two_and_four_a_waltz_three_beats_a_bar()
     {
        let dance = score(Style::Dance, 1);
        let kicks: Vec<f64> = dance
            .notes
            .iter()
            .filter(|n| n.voice == Voice::Kick)
            .map(|n| n.at)
            .collect();
        assert_eq!(kicks, (0..64).map(f64::from).collect::<Vec<_>>());
        assert!(
            dance
                .notes
                .iter()
                .filter(|n| n.voice == Voice::Clap)
                .all(|n| n.at % 2.0 == 1.0)
        );
        assert_eq!(score(Style::Slow, 1).meter, 3);
        assert!(
            !score(Style::Folk, 1)
                .notes
                .iter()
                .any(|n| n.voice == Voice::Kick)
        );
        // The bass plays the chord's own note at the start of each bar of a folk tune.
        let folk = score(Style::Folk, 2);
        for bar in 0..folk.bars {
            let bass = folk
                .notes
                .iter()
                .find(|n| n.voice == Voice::Bass && n.at == (bar * folk.meter) as f64)
                .unwrap();
            assert_eq!(
                (bass.midi - folk.key).rem_euclid(12),
                MAJOR[folk.chords[bar] as usize % 7]
            );
        }
    }

    // A piece played.

    #[test]
    fn a_loop_of_its_written_length_both_sides_whole_the_same_every_time() {
        let piece = score(Style::Slow, 2);
        let a = perform(&piece, RATE);
        assert_eq!(a.left.len(), round_usize(seconds(&piece) * RATE));
        assert_eq!(a.right.len(), a.left.len());
        assert!((seconds(&piece) - (16.0 * 3.0 * 60.0) / 72.0).abs() < 5e-10);
        assert_eq!(perform(&piece, RATE).left, a.left);
        let (mut sum, mut differ) = (0.0, 0.0);
        for (&l, &r) in a.left.iter().zip(&a.right) {
            sum += (l as f64).powi(2);
            differ += (l as f64 - r as f64).powi(2);
        }
        assert!(sum.is_finite());
        assert!(sum > 0.0);
        // The voices sit a little left and right: the two sides are not the same.
        assert!(differ / sum > 0.001);
    }

    #[test]
    fn the_pieces_the_game_plays_are_half_a_minute_and_more_each_in_its_style() {
        for (kind, style, seed) in PIECES {
            assert!(seconds(&score(style, seed)) >= 30.0);
            assert!(make(kind, 8000.0).is_some());
        }
        assert!(make("musicJazz", 8000.0).is_none());
        assert_eq!(PIECES.iter().map(|p| p.1).collect::<HashSet<_>>().len(), 3);
    }

    #[test]
    fn two_to_a_power_is_the_typescripts() {
        // `2 ** ((midi - 69) / 12)` as Bun gave it, where libm's pow is a last bit off (79, 80) and at -0.5 (63).
        assert_eq!(exp2(10.0 / 12.0), 1.7817974362806785);
        assert_eq!(hz(79), 783.9908719634985);
        assert_eq!(exp2(-0.5), 0.7071067811865475);
        assert_eq!(exp2(11.0 / 12.0), 1.8877486253633868);
        assert_eq!(exp2(-2.0), 0.25);
    }
}
