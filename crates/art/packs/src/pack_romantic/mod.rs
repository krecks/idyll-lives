//! The romantic pack (P15): the candlelit bistro table, the heart-shaped petal tub, the rose-gold vanity, the
//! moonlight gramophone, the love-letter bureau and the rose-arbour swing. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, and
//! `centre` on its `Model`), with its moving parts: the candle flames grow (`candles`), the tub fills with petals on
//! the water (`fill`), the vanity's drawer opens (`drawer_open`), the gramophone's tone arm swings onto the record
//! (`needle`) while the platter turns (`platter`, a loop), the bureau's writing flap drops (`fall_front`) and the
//! arbour's swing sways with its riders (`swing`, a loop) while a bloom is cut (`cut`).
//!
//! Roses are turned cups of petals round a bud (`_rose`), their leaves botany's `leaf`; the tablecloth is lofted and
//! falls in folds; the heart tub's shell is lofted through heart-shaped rings, its rim rolled; chairs and the pouf are
//! stuffed (`soft`), the vanity stands on the kit's cabriole legs, its oval mirror in a swept frame; the arbour's arch
//! and the swing's arms are swept in one piece, its lattice too.
//!
//! Run:  cargo art pack_romantic [--only heartTub] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod bureau;
mod common;
mod dining;
mod vanity;

pub use common::M;

/// The family.
pub fn family() -> Family {
    Family::new(
        "pack_romantic",
        M.palette(),
        vec![
            Model::new("candlelitTable", dining::candlelit_table, (1.802, 1.096, 0.8))
                .key("model.romantic.candlelitTable")
                .centre(0.0, 0.1)
                .motions(json!({"uses": {"candlelitDinner": "candles", "shareDessert": "candles"}}))
                .paint(&[("body", "Body"), ("fabric", "Fabric"), ("cloth", "Tablecloth")]),
            Model::new("heartTub", dining::heart_tub, (1.865, 0.784, 1.817))
                .key("model.romantic.heartTub")
                .centre(-0.017, 0.004)
                .motions(json!({"uses": {"petalBath": "fill"}}))
                .paint(&[("tub", "Body"), ("cloth", "Fabric")]),
            Model::new("vanity", vanity::vanity, (0.8, 1.64, 0.822))
                .key("model.romantic.vanity")
                .centre(0.0, -0.019)
                .motions(json!({"uses": {"primp": "drawer_open"}, "hold": 2.0}))
                .paint(&[("ivory", "Body"), ("velvet", "Fabric"), ("rosegold", "Rose gold")]),
            Model::new("gramophone", vanity::gramophone, (0.742, 1.581, 0.702))
                .key("model.romantic.gramophone")
                .centre(0.041, -0.079)
                .motions(json!({"uses": {"listen": "needle", "waltz": "needle"}, "loops": {"listen": "platter", "waltz": "platter"}}))
                .paint(&[("cabinet", "Body"), ("horn", "Horn")]),
            Model::new("letterBureau", bureau::letter_bureau, (0.84, 1.18, 0.911))
                .key("model.romantic.letterBureau")
                .centre(0.0, 0.0255)
                .motions(json!({"uses": {"writeLetter": "fall_front", "writePoem": "fall_front"}}))
                .paint(&[("ivory", "Body"), ("fabric", "Fabric")]),
            Model::new("roseArbourSwing", bureau::rose_arbour_swing, (1.844, 2.63, 0.821))
                .key("model.romantic.roseArbourSwing")
                .centre(0.0, 0.0)
                .motions(json!({"uses": {"tendRoses": {"clip": "cut", "delay": 0.6, "slot": 0}}, "loops": {"swing": "swing"}, "ride": "swing"}))
                .paint(&[("frame", "Body"), ("cushion", "Fabric")]),
        ],
        "Romantic pack models (crates/art/packs/src/pack_romantic, built by cargo art). Listed in the base manifest's \
         packs. The materials map onto the game's finishes so a model costs one draw call per finish group. Each \
         moves while it is used: the candles light, the tub fills with petals, the vanity's drawer opens, the \
         gramophone's arm drops onto the turning record, the bureau's flap drops and the arbour swing sways while a \
         bloom is cut.",
    )
}
