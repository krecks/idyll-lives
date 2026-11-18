//! The outdoor things. Notes: NOTES.md.
//!
//! OU1, patio seating and tables: the park bench (`model.bench`), the folding bistro set (`model.patioSet`), the
//! picnic table (`model.picnicTable`), the garden chair (`model.gardenChair`), the round mosaic table
//! (`model.patioTableRound`), the teak table (`model.patioTable4`), the patio chair (`model.patioChair`), the
//! Adirondack chair (`model.adirondackChair`), the three-seat garden bench (`model.benchGarden3`), the outdoor bar
//! (`model.outdoorBar`) and the parasol (`model.parasol`), whose canopy stirs in the wind (`stir`, ambient).
//!
//! OU2: the sun lounger, the hammock, the porch swing, the swing set and the cedar hot tub, each moving while used.
//!
//! OU3, grills and fire: the charcoal grill (`model.grill`, its lid swings up while cooking), the gas grill station
//! (`model.grillStation`, its hood swings up) and the fire pit (`model.firePit`, its flames grow while warmed by).
//!
//! Each is built in place where the game placed it (`F`, game axes), so the seat heights (content `seat`)
//! and the tops things stand on (content `surface`) stay where the game has them.

use artkit::family::{Family, Model};
use artkit::json::{Json, loads};

pub(crate) mod common;
mod fire;
mod leisure;
mod seats;
mod tables;

use common::M;

fn motions(text: &str) -> Json {
    loads(text).expect("the motions are JSON")
}

const SLATS: [(&str, &str); 2] = [("iron", "Frame"), ("park_slats", "Slats")];

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "outdoor",
        M.palette(),
        vec![
            Model::new("bench", seats::bench, (1.74, 0.886, 0.528)).centre(0.0, 0.011).key("model.bench").paint(&SLATS),
            Model::new("patioSet", seats::patio_set, (0.6, 0.933, 1.423))
                .key("model.patioSet")
                .paint(&[("iron", "Frame"), ("bistro", "Slats")]),
            Model::new("picnicTable", seats::picnic_table, (2.5, 1.058, 1.62))
                .key("model.picnicTable")
                .paint(&[("pine_dark", "Frame"), ("pine", "Planks"), ("cloth", "Cloth")]),
            Model::new("gardenChair", seats::garden_chair, (0.84, 1.041, 0.757))
                .centre(0.0, -0.052)
                .key("model.gardenChair")
                .paint(&[("garden_white", "Chair")]),
            Model::new("patioTableRound", tables::patio_table_round, (0.816, 0.725, 0.816))
                .key("model.patioTableRound")
                .paint(&[("iron", "Frame"), ("mosaic", "Top"), ("tile", "Tiles")]),
            Model::new("patioTable4", tables::patio_table4, (1.6, 0.74, 0.785))
                .centre(0.0, -0.04)
                .key("model.patioTable4")
                .paint(&[("teak_dark", "Frame"), ("teak", "Top")]),
            Model::new("patioChair", tables::patio_chair, (0.46, 0.928, 0.448))
                .centre(0.0, 0.042)
                .key("model.patioChair")
                .paint(&[("patio_green", "Frame"), ("cushion", "Cushion")]),
            Model::new("adirondackChair", seats::adirondack_chair, (0.88, 1.097, 0.785))
                .centre(0.0, -0.038)
                .key("model.adirondackChair")
                .paint(&[("adirondack", "Chair")]),
            Model::new("benchGarden3", seats::bench_garden3, (1.91, 0.906, 0.602))
                .centre(0.0, -0.021)
                .key("model.benchGarden3")
                .paint(&[("iron", "Frame"), ("bench_slats", "Slats")]),
            Model::new("outdoorBar", tables::outdoor_bar, (2.0, 1.06, 0.89))
                .centre(0.0, 0.015)
                .key("model.outdoorBar")
                .paint(&[("teak", "Body"), ("bar_top", "Counter"), ("steel", "Foot rail")]),
            Model::new("parasol", tables::parasol, (2.2, 2.46, 2.2))
                .key("model.parasol")
                .motions(motions(r#"{"idle": "stir"}"#))
                .paint(&[("canopy", "Canopy"), ("pole", "Pole"), ("valance", "Edge")]),
            Model::new("sunLounger", leisure::sun_lounger, (0.62, 0.759, 1.866))
                .centre(0.0, 0.007)
                .key("model.sunLounger")
                .motions(motions(r#"{"uses": {"sunbathe": "recline", "doze": "recline"}}"#))
                .paint(&[("lounger", "Frame"), ("lounger_pad", "Cushions"), ("towel", "Towel")]),
            Model::new("hammock", leisure::hammock, (1.0, 1.2, 2.8))
                .key("model.hammock")
                .motions(motions(r#"{"loops": {"swing": "sway", "doze": "sway"}, "ride": "sway"}"#))
                .paint(&[("hammock_frame", "Frame"), ("sling", "Fabric"), ("sling_stripe", "Stripes")]),
            Model::new("porchSwing", leisure::porch_swing, (1.92, 2.05, 0.92))
                .key("model.porchSwing")
                .motions(motions(r#"{"loops": {"swing": "swing"}, "ride": "swing"}"#))
                .paint(&[("porch_seat", "Seat"), ("porch_frame", "Frame"), ("chain", "Chains"), ("porch_cushion", "Cushion")]),
            Model::new("swingSet", leisure::swing_set, (1.92, 1.972, 1.51))
                .key("model.swingSet")
                .motions(motions(r#"{"loops": {"swing": "swing"}, "ride": "swing"}"#))
                .paint(&[("swing_frame", "Frame"), ("swing_seat", "Seats")]),
            Model::new("hotTubCedar", leisure::hot_tub_cedar, (1.886, 0.917, 1.903))
                .centre(0.0, 0.008)
                .key("model.hotTubCedar")
                .motions(motions(r#"{"uses": {"soak": "cover_off"}, "open": "cover_off", "loops": {"soak": "bubbles"}}"#))
                .paint(&[("cedar", "Cedar"), ("tub_cover", "Cover")]),
            // OU3: the grills and the fire pit
            // the lid up for a cook, and for any recipe's grilling step at it (the `flip` tag: one started at the fridge)
            Model::new("grill", fire::grill, (0.818, 1.044, 0.804))
                .centre(0.089, 0.096)
                .key("model.grill")
                .motions(motions(r#"{"uses": {"cook": "lid_open", "flip": "lid_open"}}"#))
                .paint(&[("kettle", "Body"), ("kettle_lid", "Lid")]),
            Model::new("grillStation", fire::grill_station, (1.66, 1.199, 0.66))
                .centre(0.0, -0.125)
                .key("model.grillStation")
                .motions(motions(r#"{"uses": {"cook": "hood_up", "feast": "hood_up", "flip": "hood_up"}}"#))
                .paint(&[("cart", "Cart"), ("hood", "Hood")]),
            Model::new("firePit", fire::fire_pit, (0.927, 0.619, 0.91))
                .centre(-0.001, -0.009)
                .key("model.firePit")
                .motions(motions(r#"{"uses": {"relax": "flames"}}"#))
                .paint(&[("pit", "Bowl"), ("pit_stones", "Stones")]),
        ],
        "The outdoor things (crates/art/outside/src/outdoor, built by cargo art). Listed in the base manifest's \
         packs. The materials map onto the game's finishes so a model costs one draw call per finish group. The \
         parasol's canopy stirs in the wind (`stir`, ambient); the lounger reclines, the hammock and the swings \
         sway with whoever is on them, and the hot tub's cover folds back for a soak.",
    ))
}
