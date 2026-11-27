//! The sounds of things in use, synthesised like the water (\`water\`, whose building blocks they share; no
//! sound files):
//! - machines: a treadmill, the bikes, a rowing machine, a sewing machine, a pottery wheel, coffee machines, a blender and a
//!   stand mixer, a microwave, a toaster, an oven's fan, a cooker hood;
//! - games: pool, foosball, ping-pong, darts, an arcade cabinet, basketball;
//! - a workbench's tools and a bike's, a fire, a chemistry set bubbling, a punch bag, crockery smashed in a
//!   bin, a cello played with a bow, someone singing to a karaoke track;
//! - played once: a fridge door, a chest freezer's lid, a pedal bin, the vending machines, a phone box's call and its hanging up,
//!   the microwave's ping, a doorbell's chime, toast popping up, coffee poured into a cup;
//! - at home and in the garden (\`home\`, \`HOME\`): a vacuum cleaner, an iron, a broom, a mop, a mower, a hose,
//!   a watering can, shears, a kettle boiling (and switching itself off, played once), typing, a sauna's
//!   stove, an electric fire, a knife and fork on a plate;
//! - things happening (\`events\`): a door or a gate opened and shut, curtains drawn (blinds and shutters too), a car's door and its
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
pub const THINGS: [&str; 37] = [
    "treadmill",
    "bike",
    "rower",
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
    "lidThud",
    "binLid",
    "vendSnack",
    "vendCan",
    "phoneBox",
    "hangUp",
    "ping",
    "doorbell",
    "toastPop",
    "pour",
];

/// The ones played once as something happens (a use starting or ending), not looped while it lasts.
pub const ONCE_THINGS: [&str; 11] = [
    "fridgeDoor",
    "lidThud",
    "binLid",
    "vendSnack",
    "vendCan",
    "phoneBox",
    "hangUp",
    "ping",
    "doorbell",
    "toastPop",
    "pour",
];

/// A thing's sound by name (\`THINGS\`). None: no such thing.
pub fn make(kind: &str, rate: f64) -> Option<Stereo> {
    Some(match kind {
        "treadmill" => treadmill(rate),
        "bike" => bike(rate),
        "rower" => rower(rate),
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
        "lidThud" => lid_thud(rate),
        "binLid" => bin_lid(rate),
        "vendSnack" => vend_snack(rate),
        "vendCan" => vend_can(rate),
        "phoneBox" => phone_box(rate),
        "hangUp" => hang_up(rate),
        "ping" => ping(rate),
        "doorbell" => doorbell(rate),
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

use crafts::*;
use games::*;
use machines::*;
use once::*;
use synth::*;

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 48000.0;

    #[test]
    fn only_the_things_are_made_here() {
        assert!(make("shower", RATE).is_none());
        assert!(make("nothing", RATE).is_none());
        assert!(ONCE_THINGS.iter().all(|k| THINGS.contains(k)));
    }
}
