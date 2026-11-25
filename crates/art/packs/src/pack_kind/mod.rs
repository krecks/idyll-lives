//! The kind pack (P5): the neighbourly baking counter, the community giving jar, the gift-knitting rocker, the
//! welcome tea table, the songbird feeding station and the hearthside quilted settee. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre). Each moves while it is used: the oven door drops, a coin drops into the jar, the rocker
//! rocks, the teapot pours, the feeder's roof lifts and songbirds fly out, and the settee's cushions sink.
//!
//! Run:  cargo art pack_kind [--only givingJar] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod bake;
mod common;
mod garden;
mod home;

pub use common::M;

/// The family.
pub fn family() -> Family {
    Family::new(
        "pack_kind",
        M.palette(),
        vec![
            Model::new(
                "neighbourlyOven",
                bake::neighbourly_oven,
                (1.832, 1.08, 0.640),
            )
            .key("model.kind.neighbourlyOven")
            // the door drops while the dish goes in, then shuts for the bake (`hold`). Keyed by the bake step's
            // tag, not the uses: the recipes' mixing is done at its worktop too, where the oven stays shut.
            .centre(-0.004, -0.110)
            .motions(json!({"uses": {"bake": "oven_open"}, "hold": 3.0}))
            .paint(&[
                ("enamel", "Body"),
                ("towel", "Fabric"),
                ("cabinet", "Cupboards"),
            ]),
            Model::new("givingJar", bake::giving_jar, (0.44, 1.32, 0.44))
                .key("model.kind.givingJar")
                .centre(0.101, 0.057)
                .motions(json!({"uses": {"donate": "coin"}}))
                .paint(&[("pedestal", "Body"), ("ribbon", "Fabric")]),
            Model::new(
                "knittingRocker",
                home::knitting_rocker,
                (0.723, 1.046, 0.867),
            )
            .key("model.kind.knittingRocker")
            .centre(0.09, 0.007)
            .motions(json!({"loops": {"knitGift": "rock", "rock": "rock"}, "ride": "rock"}))
            .paint(&[("rocker", "Body"), ("cushion", "Fabric")]),
            Model::new(
                "welcomeTeaTable",
                home::welcome_tea_table,
                (0.403, 0.501, 0.56),
            )
            .key("model.kind.welcomeTeaTable")
            .centre(0.0, -0.005)
            .motions(json!({"uses": {"shareTea": "pour"}, "hold": 1.5}))
            .paint(&[
                ("table", "Body"),
                ("pouf", "Fabric"),
                ("pouf_rose", "Second pouf"),
                ("rug", "Rug"),
            ]),
            Model::new(
                "songbirdFeeder",
                garden::songbird_feeder,
                (0.901, 1.698, 0.884),
            )
            .key("model.kind.songbirdFeeder")
            .centre(0.0, 0.012)
            .motions(json!({"uses": {"refill": "lid_open", "watchBirds": "bird"}}))
            .paint(&[("feeder", "Body"), ("sack", "Fabric"), ("roof", "Roof")]),
            Model::new("hearthSettee", home::hearth_settee, (1.9, 0.945, 0.828))
                .key("model.kind.hearthSettee")
                .centre(0.0, -0.016)
                .motions(json!({"uses": {"curlUp": "sink_in", "quiltNap": "sink_in"}}))
                .paint(&[("body", "Body"), ("quilt", "Fabric")]),
        ],
        "Kind pack models (crates/art/packs/src/pack_kind, built by cargo art). Listed in the base manifest's packs. \
         The materials map onto the game's finishes so a model costs one draw call per finish group. Each moves \
         while it is used: the oven door drops, a coin drops into the jar, the rocker rocks, the teapot pours, the \
         feeder's roof lifts and songbirds fly out, and the settee's cushions sink.",
    )
}
