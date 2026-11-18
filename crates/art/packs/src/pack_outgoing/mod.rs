//! The outgoing pack (P14): the home party bar, the conversation sectional, the karaoke stage, the video-call corner,
//! the light-up dance floor and the party photo wall. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, and
//! `centre` on its `Model`), with its moving parts: the bar's beer-tap lever pulls (`tap`), the sectional's seat
//! cushions sink (`sink_in`), the karaoke mics rise on their stands (`mic_up`), the ring light swings up to face the
//! caller (`ring_light`), the dance floor's tiles pulse up and down in a checkerboard (`tiles`, a loop) and a new
//! photo grows on the board (`photo`).
//!
//! Upholstery is stuffed and piped (`soft`); the bar's fluted front one slab extruded from a scalloped outline, its
//! cabinet doors the kit's shaker, its rail and the stands' legs bent tube (the kit's tube); bottles, glasses, mics,
//! speaker cones and lamps turned (`revolve`) or lofted; the gantry's posts and beam triangular trusses swept; the
//! board's frame the kit's moulding; the balloons lofted, their strings and the bulbs' wire swept.
//!
//! Run:  cargo art pack_outgoing [--only partyBar] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod bar;
mod common;
mod floor;
mod stage;

pub use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "pack_outgoing",
        M.palette(),
        vec![
            Model::new("partyBar", bar::party_bar, (1.84, 2.0, 0.875))
                .key("model.outgoing.partyBar")
                .centre(0.0, 0.008)
                .motions(json!({"uses": {"mixDrinks": "tap", "soda": "tap"}, "hold": 2.0}))
                .paint(&[
                    ("body", "Body"),
                    ("vinyl", "Upholstery"),
                    ("wood", "Back bar"),
                ]),
            Model::new(
                "conversationSectional",
                bar::conversation_sectional,
                (1.9, 1.42, 1.86),
            )
            .key("model.outgoing.conversationSectional")
            .centre(0.0, 0.0)
            .motions(json!({"uses": {"hangOut": "sink_in"}}))
            .paint(&[("frame", "Body"), ("upholstery", "Upholstery")]),
            Model::new("karaokeStage", stage::karaoke_stage, (1.9, 1.63, 0.894))
                .key("model.outgoing.karaokeStage")
                // (centred on 0.007 before 2026-10-22: its screen's foot stood 1 cm into the wall)
                .centre(0.0, 0.012)
                .motions(json!({"uses": {"practise": "mic_up", "perform": "mic_up"}}))
                .paint(&[("body", "Body"), ("speaker", "Speakers")]),
            Model::new(
                "videoCallNook",
                stage::video_call_nook,
                (0.84, 1.283, 0.788),
            )
            .key("model.outgoing.videoCallNook")
            .centre(0.0, 0.077)
            .motions(json!({"uses": {"call": "ring_light"}}))
            .paint(&[
                ("body", "Body"),
                ("cushion", "Upholstery"),
                ("wood", "Wood"),
            ]),
            Model::new("danceFloor", floor::dance_floor, (1.92, 2.33, 1.88))
                .key("model.outgoing.danceFloor")
                .centre(0.0, 0.0)
                .motions(json!({"loops": {"dance": "tiles"}}))
                .paint(&[("base", "Body"), ("speaker", "Speakers")]),
            Model::new("photoWall", floor::photo_wall, (0.9, 2.16, 0.492))
                .key("model.outgoing.photoWall")
                .centre(0.0, -0.184)
                .motions(json!({"uses": {"reminisce": "photo"}}))
                .paint(&[("wood", "Body"), ("cork", "Board")]),
        ],
        "Outgoing pack models (crates/art/packs/src/pack_outgoing, built by cargo art). Listed in the base manifest's \
         packs. The materials map onto the game's finishes so a model costs one draw call per finish group. Each \
         moves while it is used: the bar's tap lever pulls, the sectional's cushions sink, the mics rise, the ring \
         light swings up, the dance floor's tiles pulse and a new photo grows.",
    ))
}
