//! The nature lover pack (P12): the raised veggie bed, the potting bench, the bird feeder post, the garden hammock,
//! the cedar outdoor shower, the moss terrarium and the fern glasshouse. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre): a lettuce is picked from the veggie bed (`harvest`) whose crop follows its growth stage
//! (`grow`), a seedling comes up in a pot (`seedling`), a bird lands on the feeder (`bird`), the hammock sways
//! (`sway`, a loop the resident rides), the shower runs (`water_on`), the terrarium's glass lid lifts (`lid_open`)
//! and the glasshouse's door opens to let the resident in and out (`door_open`) and its roof vent airs it
//! (`vent_open`).
//!
//! The plants are leaves (`botany::leaf`: closed, bent blades, cut into leaflets with `notch`) and flower faces
//! (`botany::star`), stems and canes swept; pots turned with a rolled rim; boards and posts chamfered, rails mitred
//! round the bed (the kit's moulding), tubes bent in one piece (the kit's tube), cushions, sacks and towels stuffed
//! or draped (`soft`, `sweep`), birds the kit's songbird.
//!
//! Run:  cargo art pack_natureLover [--only veggieBed] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod bed;
mod bench;
mod common;
mod feeder;
mod ferns;
mod glasshouse;
mod hammock;
mod shower;
mod terrarium;

pub use common::M;

/// The family.
pub fn family() -> Family {
    Family::new(
        "pack_natureLover",
        M.palette(),
        vec![
            Model::new("veggieBed", bed::veggie_bed, (1.87, 1.3, 0.91))
                .centre(0.0, 0.025)
                .key("model.natureLover.veggieBed")
                .motions(json!({
                    "uses": {"pickSnack": {"clip": "harvest", "over": 1}, "tendVeggies": {"clip": "rustle", "delay": 0.6, "every": 2.4}},
                    "growth": "grow"
                }))
                .paint(&[("cedar", "Wood"), ("cedar_dark", "Posts & rails")]),
            Model::new("pottingBench", bench::potting_bench, (1.76, 1.74, 0.6))
                .key("model.natureLover.pottingBench")
                .centre(0.0, -0.13)
                .motions(json!({"uses": {"potSeedlings": "seedling", "tieBouquet": "bouquet"}}))
                .paint(&[("bench", "Wood"), ("sage", "Frame")]),
            Model::new("birdFeeder", feeder::bird_feeder, (0.705, 1.783, 0.679))
                .key("model.natureLover.birdFeeder")
                .centre(0.01, -0.05)
                .motions(json!({"uses": {"birdwatch": "bird"}}))
                .paint(&[("cedar", "Wood"), ("roof", "Roof")]),
            Model::new("gardenHammock", hammock::garden_hammock, (2.88, 1.19, 0.68))
                .key("model.natureLover.gardenHammock")
                .motions(json!({"loops": {"swing": "sway", "doze": "sway"}, "ride": "sway"}))
                .paint(&[("cedar_wood", "Wood"), ("canvas", "Fabric"), ("stripe", "Stripes")]),
            Model::new("cedarShower", shower::cedar_shower, (0.9, 2.082, 0.882))
                .key("model.natureLover.cedarShower")
                .centre(0.0, 0.011)
                .motions(json!({"uses": {"rinse": "water_on"}}))
                .paint(&[("cedar_wood", "Wood"), ("copper", "Copper")]),
            Model::new("mossTerrarium", terrarium::moss_terrarium, (0.4, 1.174, 0.4))
                .key("model.natureLover.mossTerrarium")
                .centre(0.03, 0.049)
                .motions(json!({"uses": {"mist": "lid_open"}}))
                .paint(&[("walnut", "Wood")]),
            Model::new("glasshouse", glasshouse::glasshouse, (1.887, 2.481, 1.86))
                .key("model.natureLover.glasshouse")
                // the door lets the resident in and shuts behind them (`hold`), and opens again to let them out
                // (`out`); the vent stays open while they're in
                .motions(json!({"uses": {
                    "unwind": [{"clip": "door_open", "hold": 1.5, "out": 1.5}, "vent_open"],
                    "orchids": [{"clip": "door_open", "hold": 1.5, "out": 1.5}, "vent_open"]
                }}))
                .paint(&[("cedar", "Wood"), ("frame", "Frame"), ("cushion", "Cushions")]),
        ],
        "Nature lover pack models (crates/art/packs/src/pack_nature_lover, built by cargo art). \
         Listed in the base manifest's packs. The materials map onto the game's finishes so a model \
         costs one draw call per finish group. Each moves while it is used: a lettuce is picked from \
         the veggie bed (whose crop follows its growth stage), a seedling comes up, a bird lands, the \
         hammock sways, the shower runs, the cloche lifts and the glasshouse's door and vent open.",
    )
}
