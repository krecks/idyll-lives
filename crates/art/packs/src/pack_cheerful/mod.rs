//! The cheerful pack (P6): the sunburst jukebox, the spotlight karaoke machine, the fizz & sunshine mocktail cart,
//! the rainbow hammock, the sunrise stretch lamp and the confetti party booth. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built). Each
//! has one moving part: the record swings up, the microphone rises, the shaker is taken off the cart, the hammock
//! sways (a loop its sleeper rides), the sun rises (its sky glowing at night) and the curtain is drawn.
//!
//! Run:  cargo art pack_cheerful [--only jukebox] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod cart;
mod common;
mod music;
mod play;

pub use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "pack_cheerful",
        M.palette(),
        vec![
            Model::new("jukebox", music::jukebox, (0.75, 1.43, 0.495))
                .key("model.cheerful.jukebox")
                .centre(0.0, -0.183)
                .motions(json!({"uses": {"dance": "record"}, "loops": {"dance": "spin"}}))
                .paint(&[("cherry", "Body"), ("amber", "Trim")]),
            Model::new(
                "karaokeMachine",
                music::karaoke_machine,
                (0.76, 1.763, 0.851),
            )
            .key("model.cheerful.karaokeMachine")
            .centre(-0.06, -0.005)
            .motions(json!({"uses": {"sing": "mic_up"}}))
            .paint(&[
                ("speaker", "Body"),
                ("speaker_trim", "Trim"),
                ("stage", "Rug"),
            ]),
            Model::new("mocktailCart", cart::mocktail_cart, (0.84, 1.635, 0.57))
                .key("model.cheerful.mocktailCart")
                .centre(0.0, -0.045)
                .motions(json!({"uses": {"mix": "shake"}}))
                .paint(&[("mint", "Body"), ("canopy", "Trim"), ("brass", "Frame")]),
            Model::new("rainbowHammock", play::rainbow_hammock, (2.844, 1.172, 0.7))
                .key("model.cheerful.rainbowHammock")
                .centre(0.0, 0.0)
                .motions(json!({"loops": {"swing": "sway", "doze": "sway"}, "ride": "sway"}))
                .paint(&[("wood", "Body"), ("spreader", "Trim"), ("pillow", "Pillow")]),
            Model::new("sunriseLamp", play::sunrise_lamp, (0.784, 1.642, 0.87))
                .key("model.cheerful.sunriseLamp")
                .centre(0.0, 0.005)
                .motions(json!({"uses": {"stretch": "sunrise"}}))
                .paint(&[("housing", "Body"), ("lamp_metal", "Trim"), ("yoga", "Mat")]),
            Model::new("partyBooth", play::party_booth, (1.9, 1.64, 1.83))
                .key("model.cheerful.partyBooth")
                .centre(0.0, -0.015)
                .motions(json!({"uses": {"party": "curtain"}, "hold": 1.5}))
                .paint(&[
                    ("booth", "Body"),
                    ("piping", "Trim"),
                    ("curtain", "Curtain"),
                ]),
        ],
        "Cheerful pack models (crates/art/packs/src/pack_cheerful, built by cargo art). Listed in the base \
         manifest's packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group. Each has one moving part: the record swings up, the microphone rises, the shaker is taken off the \
         cart, the hammock sways (a loop its sleeper rides), the sun rises (its sky glowing at night) and the \
         curtain is drawn.",
    ))
}
