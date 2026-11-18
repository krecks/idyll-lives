//! The bookworm pack (P1): the window-seat reading nook, the rare-books library wall, the writer's bureau, the
//! book-club tea trolley, the fireside reading lounge and the bedside book stack. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre). Each has one moving part its use plays: the nook's cushion sinks, the ladder rolls, the
//! bureau's flap drops, the teapot pours, the fire grows and the top book opens.
//!
//! Run:  cargo art pack_bookworm [--only libraryWall] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod common;
mod desk;
mod fire;
mod rooms;

pub use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "pack_bookworm",
        M.palette(),
        vec![
            Model::new("readingNook", rooms::reading_nook, (1.88, 1.71, 0.646))
                .key("model.bookworm.readingNook")
                .centre(0.0, -0.107)
                .motions(json!({"uses": {"curlUp": "sink_in"}}))
                .paint(&[
                    ("body", "Wood"),
                    ("fabric", "Fabric"),
                    ("cushion", "Cushions"),
                ]),
            Model::new("libraryWall", rooms::library_wall, (1.88, 2.34, 0.82))
                .key("model.bookworm.libraryWall")
                .centre(0.0, -0.02)
                .motions(json!({"uses": {"research": "ladder", "browse": "ladder"}}))
                .paint(&[("body", "Wood"), ("honey", "Ladder")]),
            Model::new("writingBureau", desk::writing_bureau, (0.852, 1.35, 0.758))
                .key("model.bookworm.writingBureau")
                .centre(0.006, -0.051)
                .motions(json!({"uses": {"draft": "fall_front", "letter": "fall_front"}}))
                .paint(&[("body", "Wood"), ("seatpad", "Fabric"), ("honey", "Chair")]),
            Model::new("teaTrolley", desk::tea_trolley, (0.746, 0.939, 0.492))
                .key("model.bookworm.teaTrolley")
                .centre(0.0, -0.184)
                .motions(json!({"uses": {"brew": "pour"}, "hold": 1.5}))
                .paint(&[("honey", "Wood"), ("linen", "Fabric"), ("brass", "Frame")]),
            Model::new("firesideLounge", fire::fireside_lounge, (1.7, 2.8, 1.85))
                .key("model.bookworm.firesideLounge")
                .centre(0.0, -0.005)
                .motions(json!({"uses": {"readByFire": "flames", "readAloud": "flames"}}))
                .paint(&[("walnut", "Wood"), ("velvet", "Fabric"), ("rug", "Rug")]),
            Model::new("bedsideStack", fire::bedside_stack, (0.626, 0.931, 0.73))
                .key("model.bookworm.bedsideStack")
                .centre(0.023, -0.06)
                .motions(json!({"uses": {"chapter": "book_up"}}))
                .paint(&[("body", "Wood"), ("shade", "Fabric"), ("panel", "Drawers")]),
        ],
        "Bookworm pack models (crates/art/packs/src/pack_bookworm, built by cargo art). Listed in the base \
         manifest's packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group. Each has one moving part its use plays: the nook's cushion sinks, the ladder rolls, the bureau's \
         flap drops, the teapot pours, the fire grows and the top book opens.",
    ))
}
