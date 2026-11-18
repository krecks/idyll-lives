//! The example pack (P5): the chef's stove and the whirlpool tub of `content/packs/example.json`. Notes: NOTES.md.
//!
//! The pack people copy to make their own, so each model is built plainly, a helper per kind of part, from the
//! kernel's forms and the kit's parts: a frame-and-panel oven door (the kit's shaker) with a bar pull (its pull),
//! turned knobs and burners (`revolve`), a canopy hood lofted from its band up into the chimney (`loft` through
//! `ring`s), the cast-iron grates' frames mitred (the kit's moulding), a rail bent in one piece (the kit's tube
//! through filleted corners), the tub's acrylic shell lofted in one piece, its cedar cladding extruded from a
//! grooved outline (`extrude`), headrests and towels stuffed (`soft`).
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, and
//! `centre` on its `Model`). The stove's left oven door drops (`oven_open`) while a resident bakes, and a knob
//! turns as a ring of gas flame grows on the front burner (`burner_on`) while they cook on the hob; the tub's water
//! rises (`fill`) and bubbles well up through it while it lasts (`bubbles`, a loop).
//!
//! Run:  cargo art pack_example [--only chefStove] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod common;
mod stove;
mod tub;

pub use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "pack_example",
        M.palette(),
        vec![
            Model::new("chefStove", stove::chef_stove, (1.62, 2.55, 0.662))
                .key("model.chefStove")
                .centre(0.0, -0.099)
                // a meal on the hob lights the burner; a bake (practise, or any `bake` step) drops the oven door
                // for 3 s; by the cooking step's tag too (`stir`, `bake`), for a meal started at the fridge
                .motions(json!({"uses": {"cook": "burner_on", "stir": "burner_on",
                    "practise": {"clip": "oven_open", "hold": 3.0}, "bake": {"clip": "oven_open", "hold": 3.0}}}))
                .paint(&[("body", "Body"), ("chrome", "Handles & knobs"), ("tile", "Tiles")]),
            Model::new("whirlpoolTub", tub::whirlpool_tub, (1.86, 0.929, 1.86))
                .key("model.whirlpoolTub")
                .motions(json!({"uses": {"soak": "fill"}, "loops": {"soak": "bubbles"}}))
                .paint(&[("shell", "Tub"), ("wood", "Surround"), ("pillow", "Headrests")]),
        ],
        "Example pack models (crates/art/packs/src/pack_example, built by cargo art). Listed in the base \
         manifest's packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group. The stove's oven door drops and its front burner lights; the tub fills and bubbles while residents \
         soak.",
    ))
}
