//! The rooms' reverb, synthesised (no sound files): what a room of a size gives back of a sound made in it,
//! as an impulse response the mix convolves the sound with (`space.ts`, a `ConvolverNode`). A response is samples
//! made at a sample rate, the same every time (was `web/src/audio/rooms.ts`).
//!
//! A response is what a furnished room at home does, not a hall: a few early echoes off the walls (their
//! times from the room's width), then a dense tail that dies away in a few tenths of a second, its highs
//! sooner than its lows (soft furniture and air take them first) and nothing under about 200 Hz (a room that
//! small has no reverb down there, only a boom). It holds the echoes only, not the sound itself (it is a
//! send: the sound goes its own way dry), and its energy is 1: a steady sound comes back from it as loud as
//! it went in, so a send's gain is how loud the room is against the sound (`RoomDef::send`).

use crate::dsp::m::*;
use crate::dsp::*;
use crate::water::powi;

/// What a room of a size is like.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomDef {
    /// A room up to this floor area (m², the game's tiles) is this size.
    pub area: f64,
    /// Metres across, for the early echoes' times.
    pub width: f64,
    /// Seconds for the tail to fall by 60 dB (its lows; the highs take `bright` of that).
    pub decay: f64,
    pub bright: f64,
    /// How loud the room is against the sound made in it (a gain: 0.2 is 14 dB under).
    pub send: f64,
}

/// The sizes, by name, smallest first: a bathroom, a hall or a pantry (tiled as often as not: short and
/// bright); a bedroom, a kitchen, a study; a living room or anything bigger. Decays as measured in furnished
/// homes (0.3 to 0.6 s), each a quiet 13 to 15 dB under the sound.
pub const ROOMS: [(&str, RoomDef); 3] = [
    (
        "small",
        RoomDef {
            area: 9.0,
            width: 2.4,
            decay: 0.32,
            bright: 0.75,
            send: 0.2,
        },
    ),
    (
        "medium",
        RoomDef {
            area: 28.0,
            width: 4.2,
            decay: 0.42,
            bright: 0.6,
            send: 0.18,
        },
    ),
    (
        "large",
        RoomDef {
            area: f64::INFINITY,
            width: 7.0,
            decay: 0.6,
            bright: 0.55,
            send: 0.22,
        },
    ),
];

/// The size of a room of `area` m² (`roomSize`).
pub fn size(area: f64) -> &'static str {
    ROOMS
        .iter()
        .find(|(_, d)| area <= d.area)
        .map_or("large", |(s, _)| s)
}

/// Metres a second.
const SOUND: f64 = 343.0;
/// The tail's highs and lows part here (Hz), and nothing comes back under `FLOOR` Hz.
const SPLIT: f64 = 2500.0;
const FLOOR: f64 = 200.0;

/// A room's impulse response by size (`roomResponse`): two channels, each its own echoes so the room is wide, its
/// energy 1 a channel on average. None: no such size.
pub fn response(size: &str, rate: f64) -> Option<Stereo> {
    let index = ROOMS.iter().position(|(s, _)| *s == size)?;
    let def = ROOMS[index].1;
    let n = round_usize(def.decay * 1.15 * rate);
    let mut out = Stereo {
        left: vec![0.0; n],
        right: vec![0.0; n],
    };
    let seed = 0x700 as f64 + index as f64 * 16.0;
    // The first echo comes back after the way to the nearest wall and back.
    let first = def.width / 2.0 / SOUND;
    for c in 0..2 {
        let ch = out.ch(c);
        let mut r = Rng::new(seed + c as f64);
        // Early echoes: six, each a little later and weaker than the last, off one wall or another.
        let mut at = first * (0.8 + 0.4 * r.next());
        for k in 0..6 {
            let i = js_round(at * rate);
            if i < n as f64 {
                let sign = if r.next() < 0.5 { -1.0 } else { 1.0 };
                add(ch, i as usize, sign * 0.9 * powi(0.72, k));
            }
            at += (def.width / SOUND) * (0.45 + 0.6 * r.next());
        }
        // The tail: noise, its lows dying away over `decay`, its highs sooner, swelling in as the echoes thicken.
        let k = 1.0 - exp((-2.0 * PI * SPLIT) / rate);
        let swell = first * 2.5;
        let mut low = 0.0;
        for i in round_usize(first * rate)..n {
            let t = i as f64 / rate;
            let w = (r.next() + r.next() + r.next() - 1.5) * 1.15;
            low += (w - low) * k;
            let rise = min(1.0, (t - first) / swell);
            add(
                ch,
                i,
                0.5 * rise
                    * (low * 1.6 * exp((-6.91 * t) / def.decay)
                        + (w - low) * exp((-6.91 * t) / (def.decay * def.bright))),
            );
        }
        // Nothing under the floor (two passes of a gentle high-pass), and the very end eased to nothing.
        let h = exp((-2.0 * PI * FLOOR) / rate);
        for _ in 0..2 {
            let (mut x1, mut y1) = (0.0, 0.0);
            for v in ch.iter_mut() {
                let x = *v as f64;
                let y = h * (y1 + x - x1);
                x1 = x;
                y1 = y;
                *v = y as f32;
            }
        }
        let ease = round_usize(0.02 * rate);
        for i in 0..ease {
            ch[n - 1 - i] = (ch[n - 1 - i] as f64 * (i as f64 / ease as f64)) as f32;
        }
    }
    let mut sum = 0.0;
    for ch in [&out.left, &out.right] {
        for &v in ch {
            sum += v as f64 * v as f64;
        }
    }
    let scale = 1.0 / sqrt(sum / 2.0);
    for ch in [&mut out.left, &mut out.right] {
        for v in ch.iter_mut() {
            *v = (*v as f64 * scale) as f32;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 48000.0;

    #[test]
    fn a_short_wide_tail_of_energy_1_the_same_every_time() {
        for (size, def) in ROOMS {
            let Stereo { left, right } = response(size, RATE).unwrap();
            assert_eq!(response(size, RATE).unwrap().left, left);
            let (mut energy, mut cross, mut late) = (0.0, 0.0, 0.0);
            for i in 0..left.len() {
                let (l, r) = (left[i] as f64, right[i] as f64);
                energy += l * l + r * r;
                cross += l * r;
                if i as f64 > left.len() as f64 * 0.75 {
                    late += l * l + r * r;
                }
            }
            assert!(f64::is_finite(energy), "{size}");
            assert!((energy / 2.0 - 1.0).abs() < 5e-6, "{size}: energy {energy}");
            // No longer than its decay says, nearly gone by its last quarter, nothing at its very end.
            assert!((left.len() as f64 / RATE) < 0.8, "{size}");
            assert!(late / energy < 0.002, "{size}");
            assert!((left[left.len() - 1] as f64).abs() < 1e-3, "{size}");
            // The two sides differ (a wide room), and it holds no copy of the sound itself (nothing at time 0).
            assert!(f64::abs(cross) / (energy / 2.0) < 0.1, "{size}");
            assert_eq!(left[0], 0.0, "{size}");
            assert_eq!(right[0], 0.0, "{size}");
            // Quiet against the sound: 13 to 15 dB under.
            let db = 20.0 * log10(def.send);
            assert!(db < -12.0 && db > -16.0, "{size}: {db} dB");
        }
        assert!(response("hall", RATE).is_none());
    }

    #[test]
    fn a_room_by_its_area() {
        assert_eq!(size(9.0), "small");
        assert_eq!(size(10.0), "medium");
        assert_eq!(size(500.0), "large");
    }
}
