//! The bedroom's storage: the nightstands and small storage (BR2: the nightstand, the modern, drum and three-drawer
//! nightstands, the nightstand with a lamp, the blanket chest, the laundry hamper and the bed-end bench) and the
//! wardrobes and drawers (BR3: the dresser, the wardrobe, the tall chest of drawers, the low dresser, the single
//! wardrobe and the armoire). Notes: NOTES.md.
//!
//! The parts are placed in game axes (`F`, no shift), where the game places it: most stand against the back of
//! their tile, their backs on the wall face (game depth -0.43), so they're built in place and settled with
//! `centre = false` (a `centre` on each Model). Their tops are the content's `surface` heights. The BR3 pieces open
//! for a resident choosing an outfit: a drawer slides out 30 cm (`drawer_open`), or the doors swing out 100 degrees
//! (`doors_open`, `door_open`) onto a rail of clothes. Of the BR2 pieces, the three-drawer nightstand's top drawer
//! slides out for a midnight snack, the chest's lid lifts for a blanket and the hamper's while the washing is
//! sorted; the rest are static.

use artkit::family::{Family, Model, motions};

mod common;
mod large;
mod small;

// Open while the resident changes into another outfit (content `changesOutfit`), shut as they leave.
const CHANGE_DRAWER: &str = r#"{"uses": {"outfit": "drawer_open"}}"#;
const CHANGE_DOORS: &str = r#"{"uses": {"outfit": "doors_open"}}"#;

/// The family.
pub fn family() -> Option<Family> {
    // the three-drawer nightstand's snack
    let snack_drawer = format!(
        r#"{{"uses": {{"snack": "drawer_open"}}, "hold": {}}}"#,
        artkit::json::float_repr(common::DRAWER_HOLD)
    );
    Some(Family::new(
        "bedroom_storage",
        common::M.palette(),
        vec![
            // BR2: the three-drawer nightstand's top drawer slides out for a snack, the chest's lid lifts for a
            // blanket and the hamper's while the washing is sorted
            Model::new("nightstand", small::nightstand, (0.54, 0.66, 0.468))
                .key("model.nightstand")
                .centre(0.0, -0.1958)
                .paint(&[("oak", "Body"), ("brass", "Handles")]),
            Model::new(
                "nightstandModern",
                small::nightstand_modern,
                (0.58, 0.56, 0.468),
            )
            .key("model.nightstandModern")
            .centre(0.0, -0.196)
            .paint(&[("white", "Body"), ("iron", "Handles")]),
            Model::new(
                "nightstandRound",
                small::nightstand_round,
                (0.54, 0.57, 0.56),
            )
            .key("model.nightstandRound")
            .centre(0.0, -0.0498)
            .paint(&[("clay", "Body"), ("brass", "Handles")]),
            Model::new(
                "nightstandDrawers",
                small::nightstand_drawers,
                (0.58, 0.6, 0.468),
            )
            .key("model.nightstandDrawers")
            .centre(0.0, -0.196)
            .motions(motions(&snack_drawer))
            .paint(&[("walnut", "Body"), ("brass", "Handles")]),
            Model::new(
                "nightstandLamp",
                small::nightstand_lamp,
                (0.54, 1.08, 0.468),
            )
            .key("model.nightstandLamp")
            .centre(0.0, -0.1958)
            .paint(&[("oak", "Body"), ("shade", "Shade")]),
            Model::new("cedarChest", small::cedar_chest, (0.974, 0.505, 0.52))
                .key("model.cedarChest")
                .motions(motions(
                    r#"{"uses": {"takeBlanket": "lid_open"}, "hold": 2.0}"#,
                ))
                .paint(&[("cedar", "Body"), ("iron", "Handles")]),
            Model::new(
                "laundryHamper",
                small::laundry_hamper,
                (0.476, 0.631, 0.476),
            )
            .key("model.laundryHamper")
            .centre(0.0, -0.2)
            .motions(motions(r#"{"uses": {"sortWashing": "lid_open"}}"#))
            .paint(&[("wicker", "Hamper")]),
            Model::new("benchBedEnd", small::bench_bed_end, (1.6, 0.471, 0.42))
                .key("model.benchBedEnd")
                .paint(&[("brass", "Frame"), ("olive", "Cushion")]),
            // BR3: each opens while a resident picks an outfit
            // its own size: the picture, vase and jewellery box it carries stand on it (0.89 m is the chest itself)
            Model::new("dresser", large::dresser, (1.44, 0.89, 0.54))
                .key("model.dresser")
                .centre(0.0, -0.16)
                .motions(motions(CHANGE_DRAWER))
                .paint(&[("oak", "Body"), ("brass", "Handles")]),
            Model::new("wardrobe", large::wardrobe, (1.84, 2.41, 0.626))
                .key("model.wardrobe")
                .centre(0.0, -0.117)
                .motions(motions(CHANGE_DOORS))
                // (the white carcass is its body; the birch doors are their own part)
                .paint(&[("white", "Body"), ("birch", "Doors"), ("brass", "Handles")]),
            Model::new(
                "chestOfDrawers",
                large::chest_of_drawers,
                (0.88, 1.16, 0.51),
            )
            .key("model.chestOfDrawers")
            .centre(0.0, -0.175)
            .motions(motions(CHANGE_DRAWER))
            .paint(&[("linen", "Body"), ("brass", "Handles")]),
            Model::new("dresserLow", large::dresser_low, (1.82, 0.8, 0.635))
                .key("model.dresserLow")
                .centre(0.0, -0.1125)
                .motions(motions(CHANGE_DRAWER))
                .paint(&[("oak", "Body"), ("iron", "Handles")]),
            Model::new("wardrobeSingle", large::wardrobe_single, (0.6, 2.28, 0.617))
                .key("model.wardrobeSingle")
                .centre(0.0, -0.1215)
                .motions(motions(r#"{"uses": {"outfit": "door_open"}}"#))
                .paint(&[("white", "Body"), ("brass", "Handles")]),
            Model::new("armoire", large::armoire, (1.86, 2.109, 0.628))
                .key("model.armoire")
                .centre(0.0, -0.1162)
                .motions(motions(CHANGE_DOORS))
                .paint(&[("mahogany", "Body"), ("brass", "Handles")]),
        ],
        "The bedroom storage (crates/art/rooms/src/bedroom_storage, built by cargo art): the nightstands, the \
         blanket chest, the laundry hamper and the bed-end bench (BR2), and the dressers, chests and wardrobes (BR3). \
         Listed in the base manifest's packs. A dresser or chest slides its middle drawer out 30 cm (`drawer_open`), \
         a wardrobe swings its doors open (`doors_open`, `door_open`) while a resident picks an outfit.",
    ))
}
