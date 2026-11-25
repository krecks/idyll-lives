//! The hot-headed pack (P8): the heavy punching bag, the plate-smash bin, the thunder drum kit, the cool-down
//! meditation cushion, the grudge-to-gratitude writing desk and the glacier cold-plunge tub. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre): the bag swings on its chains while it is pounded (`swing`, a loop), the crate's lid flips
//! up (`lid_open`), the crash and ride cymbals rock while the kit is played (`cymbals`, a loop), the zafu sinks
//! (`sink_in`), the desk's drawer slides out (`drawer_open`) and the plunge tub's slatted cover rolls back into
//! its housing (`cover_off`).
//!
//! Built with the forms and the kit (2026-10-22): the bag's frame bent from one square tube, chains of links, drums
//! with lugs and hoops on tripod stands, a pleated zafu, hairpin desk legs, a jointed lamp, cedar boards in steel
//! bands.
//!
//! Run:  cargo art pack_hotHeaded [--only heavyBag] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod bag;
mod calm;
mod common;
mod drums;

pub use common::M;

/// The wall's face in a 1-deep footprint, game depth.
const WALL: f64 = -0.43;

/// The family.
pub fn family() -> Family {
    Family::new(
        "pack_hotHeaded",
        M.palette(),
        vec![
            Model::new("heavyBag", bag::heavy_bag, (0.86, 2.01, 0.86))
                .key("model.hotHeaded.heavyBag")
                .centre(0.0, 0.0)
                .motions(json!({"loops": {"pound": "swing"}}))
                .paint(&[("bag", "Body")]),
            Model::new("smashBin", bag::smash_bin, (0.854, 0.643, 0.876))
                .key("model.hotHeaded.smashBin")
                .centre(0.003, 0.008)
                // the lid stands open while plates are thrown in, and shuts when they're done (2026-10-21: after 1 s)
                .motions(json!({"uses": {"smash": "lid_open"}}))
                .paint(&[("crate", "Body")]),
            Model::new("thunderDrums", drums::thunder_drums, (1.89, 1.279, 0.9))
                .key("model.hotHeaded.thunderDrums")
                .centre(0.005, 0.02)
                .motions(json!({"loops": {"drum": "cymbals"}}))
                .paint(&[("shell", "Body")]),
            Model::new(
                "coolDownCushion",
                calm::cool_down_cushion,
                (0.9, 0.236, 0.9),
            )
            .centre(0.0, 0.02)
            .key("model.hotHeaded.coolDownCushion")
            .motions(json!({"uses": {"countToTen": "sink_in", "breathe": "sink_in"}}))
            .paint(&[("zafu", "Body")]),
            Model::new("grudgeJournal", calm::grudge_journal, (0.84, 1.003, 0.677))
                .key("model.hotHeaded.grudgeJournal")
                .centre(0.0, 0.111)
                .motions(json!({"uses": {"writeItOut": "drawer_open"}}))
                .paint(&[("body", "Body")]),
            Model::new("coldPlunge", calm::cold_plunge, (0.86, 0.86, 1.39))
                .key("model.hotHeaded.coldPlunge")
                .centre(0.0, -0.23)
                .motions(json!({"uses": {"plunge": "cover_off"}}))
                .paint(&[("cedar", "Body")]),
        ],
        "Hot-headed pack models (crates/art/packs/src/pack_hot_headed, built by cargo art). Listed in the base \
         manifest's packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group. Each has its moving part: the bag swings while pounded, the crate's lid flips up, the cymbals rock \
         while the kit is played, the zafu sinks, the desk drawer slides out and the plunge tub's slatted cover \
         rolls back.",
    )
}
