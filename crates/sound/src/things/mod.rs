//! The sounds of things in use, synthesised like the water (\`water\`, whose building blocks they share; no
//! sound files):
//! - machines: a treadmill, the bikes, a sewing machine, a pottery wheel, coffee machines, a blender and a
//!   stand mixer, a microwave, a toaster, an oven's fan, a cooker hood;
//! - games: pool, foosball, ping-pong, darts, an arcade cabinet, basketball;
//! - a workbench's tools and a bike's, a fire, a chemistry set bubbling, a punch bag, crockery smashed in a
//!   bin, a cello played with a bow, someone singing to a karaoke track;
//! - played once: a fridge door, a pedal bin, the vending machines, a phone box's call and its hanging up,
//!   the microwave's ping, toast popping up, coffee poured into a cup;
//! - at home and in the garden (\`home\`, \`HOME\`): a vacuum cleaner, an iron, a broom, a mop, a mower, a hose,
//!   a watering can, shears, a kettle boiling (and switching itself off, played once), typing, a sauna's
//!   stove, an electric fire, a knife and fork on a plate;
//! - things happening (\`events\`): a door or a gate opened and shut, curtains drawn, a car's door and its
//!   start, the post through a letterbox, a till, a drawer.
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

mod home;
pub use home::{HOME, ONCE_HOME};
pub mod events;

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
        _ => return home::make(kind, rate).or_else(|| events::make(kind, rate)),
    })
}

mod crafts;
mod games;
mod machines;
mod once;
mod synth;

pub(crate) use crafts::*;
pub(crate) use games::*;
pub(crate) use machines::*;
pub(crate) use once::*;
pub(crate) use synth::*;

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
    /// already ringing at its start): the things', the home's and the events' loops.
    #[test]
    fn every_loop_is_seamless_at_44_1_and_96_khz() {
        let mut off = Vec::new();
        for rate in [44100.0, 96000.0] {
            let loops = THINGS
                .iter()
                .filter(|k| !ONCE_THINGS.contains(k))
                .chain(HOME.iter().filter(|k| !ONCE_HOME.contains(k)))
                .chain(
                    events::EVENTS
                        .iter()
                        .filter(|k| !events::ONCE_EVENTS.contains(k)),
                );
            for kind in loops {
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
