//! The lazy pack (P10): the snoozecloud adjustable bed, the daydream nap pod, the heated quilt table, the doorstep
//! delivery locker, the effortless vibration plate and the smart-home command hub. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre): the bed's head section rises (`head_up`), the pod's hood swings half over (`hood`), the
//! quilt's edge lifts (`quilt`), a locker door opens (`door_open`), the plate's deck shakes (`shake`, a loop) and
//! the tablet tilts up on its dock (`screen_up`).
//!
//! Built with the forms and the kit: soft mattresses, pillows and cushions with piping, a cloud headboard cut from
//! one outline, the pod's egg and a piped hood, a quilt hanging in folds to the floor, lockers with vents and
//! recessed pulls, a handlebar bent from one tube, a mid-century side table on splayed legs.
//!
//! Run:  cargo art pack_lazy [--only napPod] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod common;
mod home;
mod sleep;
mod table;

pub use common::M;

/// The family.
pub fn family() -> Family {
    Family::new(
        "pack_lazy",
        M.palette(),
        vec![
            Model::new(
                "snoozecloudBed",
                sleep::snoozecloud_bed,
                (1.86, 1.292, 1.86),
            )
            .key("model.lazy.snoozecloudBed")
            .motions(json!({"uses": {"sleepIn": "head_up"}, "hold": 2}))
            .paint(&[
                ("body", "Body"),
                ("upholstery", "Upholstery"),
                ("throw", "Throw"),
            ]),
            Model::new("napPod", sleep::nap_pod, (0.918, 1.539, 0.908))
                .key("model.lazy.napPod")
                .centre(0.0, 0.0243)
                .motions(json!({"uses": {"powerNap": "hood", "curlUp": "hood"}}))
                .paint(&[("shell", "Body"), ("seat", "Upholstery"), ("hood", "Hood")]),
            Model::new("quiltTable", table::quilt_table, (1.81, 0.77, 0.929))
                .key("model.lazy.quiltTable")
                .centre(0.0, 0.0343)
                .motions(json!({"uses": {"tuckIn": "quilt", "doze": "quilt"}}))
                .paint(&[("oak", "Body"), ("quilt", "Upholstery"), ("pouf", "Poufs")]),
            Model::new("deliveryLocker", home::delivery_locker, (0.86, 1.96, 0.81))
                .key("model.lazy.deliveryLocker")
                .centre(0.0, -0.025)
                .motions(
                    json!({"uses": {"takeout": "door_open", "noodles": "door_open"}, "hold": 2}),
                )
                .paint(&[("locker", "Body"), ("door", "Doors"), ("accent", "Trim")]),
            Model::new("vibePlate", home::vibe_plate, (0.67, 1.255, 0.771))
                .key("model.lazy.vibePlate")
                .centre(0.02, 0.0193)
                .motions(json!({"loops": {"shake": "shake"}}))
                .paint(&[("plate", "Body"), ("lime", "Trim"), ("pad", "Deck")]),
            Model::new("smartHub", table::smart_hub, (0.6, 0.806, 0.842))
                .key("model.lazy.smartHub")
                .centre(0.0, -0.009)
                .motions(json!({"uses": {"program": "screen_up", "story": "screen_up"}}))
                .paint(&[("wood", "Body"), ("speaker", "Speaker")]),
        ],
        "Lazy pack models (crates/art/packs/src/pack_lazy, built by cargo art). Listed in the base manifest's packs. \
         The materials map onto the game's finishes so a model costs one draw call per finish group. Each moves while \
         it is used: the bed's head rises, the pod's hood swings over, the quilt lifts, a locker door opens, the plate \
         shakes and the tablet tilts up.",
    )
}
