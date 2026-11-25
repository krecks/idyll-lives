//! The game's sounds, synthesised (no sound files): the water's and the things' loops (`water`, `things`), the
//! street and the weather (`outdoors`), the music (`music`), the sounds of one channel (`mono`: a sizzling pan,
//! the TV through a wall, the knock at the door…), the birds' songs (`birdsong`), the interface's blips
//! (`blips`), footsteps (`steps`) and the rooms' reverb (`rooms`). Each is samples at a sample rate, made
//! the same every time; the browser plays them (`web/src/audio/`, through `crates/sound-wasm`).
//!
//! Was `web/src/audio/*.ts`; the arithmetic is the TypeScript's, so most sounds are what they were (`dsp`), the
//! struck things and the motors to rounding (their tones now turned by rotation). The music (plucked strings,
//! saws and shaped drums), the knock at the door and the knife's chop (rung from their modes) have moved on.

pub mod birdsong;
pub mod blips;
pub mod dsp;
pub mod mono;
pub mod music;
pub mod outdoors;
pub mod rooms;
pub mod steps;
pub mod things;
pub mod water;

pub use dsp::{Filter, Stereo, node_filter, sweep};

/// A sound: one channel, or two.
#[derive(Clone, Debug, PartialEq)]
pub enum Made {
    Mono(Vec<f32>),
    Stereo(Stereo),
}

/// A sound made by kind off the main thread (`made.ts` `makeSound`): the water's and the things' loops, the
/// outdoors' and the pieces of music. None: no such kind.
pub fn make(kind: &str, rate: f64) -> Option<Made> {
    if let Some(s) = music::make(kind, rate) {
        return Some(Made::Stereo(s));
    }
    if let Some(m) = outdoors::make(kind, rate) {
        return Some(m);
    }
    water::make(kind, rate)
        .or_else(|| things::make(kind, rate))
        .map(Made::Stereo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsp::check::whole;
    use things::events::{EVENTS, ONCE_EVENTS};
    use things::{HOME, ONCE_HOME, ONCE_THINGS, THINGS};

    /// The rates a sound card runs at, a headset's 16 kHz to 96 kHz: every length and place in a sound is
    /// reckoned from it, and a panic in the wasm stops every sound until the page is loaded again.
    const RATES: [f64; 5] = [16000.0, 22050.0, 44100.0, 48000.0, 96000.0];

    /// Some samples, all finite.
    fn finite(what: &str, d: &[f32]) {
        assert!(!d.is_empty(), "{what}: empty");
        assert!(d.iter().all(|v| v.is_finite()), "{what}: not finite");
    }

    /// The water's and the things' sounds whole (`check::whole`), the outdoors' and the music finite, at each
    /// rate.
    #[test]
    fn every_sound_is_made_whole_at_every_rate() {
        let checked: Vec<&str> = [&water::WATER[..], &THINGS, &HOME, &EVENTS].concat();
        let once: Vec<&str> = [&water::ONCE[..], &ONCE_THINGS, &ONCE_HOME, &ONCE_EVENTS].concat();
        let music = music::PIECES.map(|(name, _, _)| name);
        let outdoors = ["carEngine", "carTyres", "town", "leaves"];
        let all: Vec<&str> = [&checked[..], &outdoors, &music].concat();
        std::thread::scope(|s| {
            for kind in all {
                let (checked, once) = (&checked, &once);
                s.spawn(move || {
                    for rate in RATES {
                        let what = format!("{kind} at {rate}");
                        match make(kind, rate) {
                            Some(Made::Stereo(st)) if checked.contains(&kind) => {
                                whole(&what, &st, rate, !once.contains(&kind))
                            }
                            Some(Made::Stereo(st)) => {
                                finite(&what, &st.left);
                                finite(&what, &st.right);
                            }
                            Some(Made::Mono(d)) => finite(&what, &d),
                            None => panic!("{what}: not made"),
                        }
                    }
                });
            }
        });
        assert!(make("nothing", 48000.0).is_none());
    }

    /// The sounds made on the page (one channel, a bird's song, a room's echo), finite at each rate.
    #[test]
    fn every_short_sound_is_made_at_every_rate() {
        for rate in RATES {
            for kind in mono::KINDS {
                finite(kind, &mono::make(kind, rate).unwrap());
            }
            for name in blips::NAMES {
                finite(name, &blips::make(name, rate).unwrap());
            }
            for surface in steps::SURFACES {
                for wear in steps::FOOTWEAR {
                    for v in 0..steps::STEP_VERSIONS {
                        let what = format!("{surface} {wear} {v} at {rate}");
                        finite(&what, &steps::make(surface, wear, v, rate).unwrap());
                    }
                }
            }
            for sp in birdsong::SPECIES {
                for (pace, near) in [(0.92, 0.3), (1.08, 1.0)] {
                    let bird = birdsong::Voice {
                        species: sp.name.to_string(),
                        seed: 7.0,
                        pitch: 3000.0,
                        pace,
                        near,
                    };
                    for v in 0..birdsong::VERSIONS as u32 {
                        let what = format!("{} {v} at {rate}", sp.name);
                        finite(
                            &what,
                            &birdsong::song(&bird, v, rate).expect("a known species sings"),
                        );
                    }
                }
            }
            for (size, _) in rooms::ROOMS {
                let s = rooms::response(size, rate).unwrap();
                finite(size, &s.left);
                finite(size, &s.right);
            }
        }
    }
}
