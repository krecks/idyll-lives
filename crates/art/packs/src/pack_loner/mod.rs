//! The loner pack (P11): the hush-shell reading pod, the shortwave radio desk, the jigsaw puzzle table, the bonsai
//! potting bench, the cedar sauna for one and the no-knock delivery hatch. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre): the pod's hood swings half closed (`shell`), the radio's tuning dial turns (`dial`) and its
//! antenna extends (`antenna`), a patch of the puzzle grows (`pieces`), a twig of the bonsai shrinks away (`trim`),
//! the sauna's glass door opens and shuts (`door_open`) and the delivery hatch opens (`hatch`).
//!
//! Built with the forms and the kit: an upholstered shell of soft panels under a hood of padded channels, spindle
//! chairs swept in one piece, jigsaw pieces that nest, a bonsai's tapering trunk under jittered foliage pads, a cedar
//! cabin of boards, the hatch's moulded frame.
//!
//! Run:  cargo art pack_loner [--only quietPod] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod cabin;
mod common;
mod pod;
mod puzzle;

pub use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "pack_loner",
        M.palette(),
        vec![
            Model::new("quietPod", pod::quiet_pod, (0.84, 1.539, 0.769))
                .key("model.loner.quietPod")
                .centre(0.0, -0.0455)
                .motions(json!({"uses": {"retreat": "shell", "read": "shell"}}))
                .paint(&[("shell", "Body"), ("oak", "Wood"), ("cushion", "Cushions")]),
            Model::new("shortwaveRadio", pod::shortwave_radio, (0.84, 1.325, 0.682))
                .key("model.loner.shortwaveRadio")
                .centre(0.0, 0.113)
                .motions(json!({"uses": {"tune": "dial", "airwaves": ["antenna", "dial"]}}))
                .paint(&[("body", "Body"), ("oak", "Wood"), ("seatpad", "Seat")]),
            Model::new("puzzleTable", puzzle::puzzle_table, (0.867, 1.008, 0.718))
                .key("model.loner.puzzleTable")
                .centre(0.003, 0.091)
                .motions(json!({"uses": {"piece": "pieces", "newPuzzle": "pieces"}}))
                .paint(&[("body", "Body"), ("wood", "Wood")]),
            Model::new("bonsaiBench", puzzle::bonsai_bench, (0.86, 1.31, 0.75))
                .key("model.loner.bonsaiBench")
                .centre(0.0, -0.055)
                .motions(json!({"uses": {"shape": "trim"}, "loops": {"tend": "mist"}}))
                .paint(&[("body", "Body"), ("teak", "Wood"), ("glaze", "Pot")]),
            Model::new("soloSauna", cabin::solo_sauna, (0.94, 2.085, 1.705))
                .key("model.loner.soloSauna")
                .centre(0.0, -0.0775)
                // the door lets the bather in, shuts behind them and opens again to let them out
                .motions(json!({"uses": {"sweat": {"clip": "door_open", "hold": 1.5, "out": 1.5}}}))
                .paint(&[("cedar", "Wood"), ("cedar_dark", "Trim")]),
            Model::new("deliveryHatch", cabin::delivery_hatch, (0.84, 1.95, 0.84))
                .key("model.loner.deliveryHatch")
                .centre(0.0, -0.01)
                .motions(json!({"uses": {"order": "hatch", "treat": "hatch"}, "hold": 2}))
                .paint(&[("slate", "Body"), ("wood", "Wood")]),
        ],
        "Loner pack models (crates/art/packs/src/pack_loner, built by cargo art). Listed in the base manifest's packs. \
         The materials map onto the game's finishes so a model costs one draw call per finish group. Each moves a part \
         while it is used: the pod's hood, the radio's dial and antenna, a patch of the puzzle, a twig of the bonsai, \
         the sauna's door and the delivery hatch.",
    ))
}
