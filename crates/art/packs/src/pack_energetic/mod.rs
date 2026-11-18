//! The energetic pack (P2): the home bouldering wall, the interactive spin bike, the rebound trampoline, the
//! streetball hoop, the power blender station, the contrast rain shower and the recovery pod. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre). The bike's cranks turn and the trampoline's mat dips while used (loops), the net sways,
//! the blender's lid lifts, the rain grows and the pod's canopy lowers; the bouldering wall is still.
//!
//! Run:  cargo art pack_energetic [--only spinBike] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod common;
mod home;
mod play;
mod pod;

pub use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "pack_energetic",
        M.palette(),
        vec![
            Model::new("boulderingWall", play::bouldering_wall, (1.87, 2.279, 0.877))
                .key("model.energetic.boulderingWall")
                .centre(0.0, 0.007)
                .paint(&[("climb_frame", "Frame"), ("crash_mat", "Pads"), ("plywood", "Panel")]),
            Model::new("spinBike", play::spin_bike, (0.617, 1.2, 1.6))
                .key("model.energetic.spinBike")
                .centre(0.0, 0.1)
                .motions(json!({"loops": {"ride": "pedals"}}))
                .paint(&[("bike_frame", "Frame"), ("bike_pad", "Pads"), ("flywheel", "Flywheel")]),
            Model::new("trampoline", play::trampoline, (1.9, 0.635, 1.904))
                .key("model.energetic.trampoline")
                .centre(0.0, 0.022)
                .motions(json!({"loops": {"bounce": "bounce"}}))
                .paint(&[("tramp_frame", "Frame"), ("tramp_pad", "Pads"), ("tramp_mat", "Mat")]),
            Model::new("streetballHoop", play::streetball_hoop, (0.86, 2.833, 0.886))
                .key("model.energetic.streetballHoop")
                .centre(0.0, 0.013)
                .motions(json!({"uses": {"shoot": [{"clip": "swish", "delay": 1.4, "every": 3.2}, "ball_taken"]}}))
                .paint(&[("hoop_pole", "Frame"), ("pad_red", "Pads"), ("hoop_base", "Base")]),
            Model::new("powerBlender", home::power_blender, (0.82, 1.32, 0.63))
                .key("model.energetic.powerBlender")
                // the lid comes off for the fruit and goes back on (`hold`), then it blends: the smoothie swirls, a
                // turn each second (`every` a hair under it) from the lid's back on till the use ends
                .centre(0.0, -0.115)
                .motions(json!({"uses": {"blend": ["lid", {"clip": "blades", "delay": 1.5, "every": 0.9}]}, "hold": 1.0}))
                .paint(&[("blend_cab", "Frame"), ("blend_door", "Doors"), ("motor", "Blender")]),
            Model::new("contrastShower", home::contrast_shower, (0.9, 2.15, 0.9))
                .key("model.energetic.contrastShower")
                // the door lets the bather in, shuts behind them and opens again to let them out (as the glasshouse's)
                .centre(0.0, 0.02)
                .motions(json!({"uses": {"rinse": [{"clip": "door_open", "hold": 1.5, "out": 1.5}, "water_on"]}}))
                .paint(&[("shower_frame", "Frame"), ("tile", "Tiles"), ("shower_mat", "Mat")]),
            Model::new("recoveryPod", pod::recovery_pod, (0.859, 1.175, 1.86))
                .key("model.energetic.recoveryPod")
                .centre(0.0, 0.0)
                .motions(json!({"uses": {"recover": "canopy"}}))
                .paint(&[("pod_shell", "Frame"), ("pod_pad", "Pads"), ("pod_boots", "Boots")]),
        ],
        "Energetic pack models (crates/art/packs/src/pack_energetic, built by cargo art). Listed in the base \
         manifest's packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group. The bike's cranks turn and the trampoline's mat dips while used (loops), the net sways, the \
         blender's lid lifts, the rain grows and the pod's canopy lowers; the bouldering wall is still.",
    ).switched())
}
