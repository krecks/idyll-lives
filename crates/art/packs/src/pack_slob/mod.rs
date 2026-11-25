//! The slob pack (P16): the late-night microwave cart, the crumb-proof beanbag, the snack-stash nightstand, the
//! spritz & go station, the pixel-blaster arcade cabinet and the command-couch gaming recliner. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre), with its moving parts: the microwave's door opens and shuts (`door_open`), the beanbag
//! squashes under its sitter (`squash`), the nightstand's side drawer of snacks slides out (`drawer_open`), the
//! spray bottle tips (`spritz`), the arcade joystick waggles (`joystick`, a loop), and the recliner's back
//! reclines (`recline`) and its cup holder swings out (`cup_out`).
//!
//! Cabinets and appliances are lofted through rounded rectangles; the arcade's sides cut from the classic outline
//! (`extrude`); cushions, pads, snack bags and the bag itself stuffed (`soft`) or lofted and wrinkled (`jitter`);
//! cans, buttons and wheels turned; tubes and handles bent in one piece (the kit's tube and fillet).
//!
//! Run:  cargo art pack_slob [--only beanbag] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod cart;
mod common;
mod lounge;
mod play;

pub use common::M;

/// The family.
pub fn family() -> Family {
    Family::new(
        "pack_slob",
        M.palette(),
        vec![
            Model::new("microwaveCart", cart::microwave_cart, (0.722, 1.099, 0.783))
                .key("model.slob.microwaveCart")
                .centre(0.011, -0.0386)
                .motions(json!({"uses": {"nuke": "door_open", "invent": "door_open"}, "hold": 1.5}))
                .paint(&[("body", "Body")]),
            Model::new("beanbag", lounge::beanbag, (0.888, 0.663, 0.875))
                .key("model.slob.beanbag")
                .centre(0.0, 0.007)
                .motions(json!({"uses": {"flop": "squash", "snooze": "squash"}}))
                .paint(&[("bag", "Body")]),
            Model::new("snackNightstand", lounge::snack_nightstand, (0.897, 0.45, 0.728))
                .key("model.slob.snackNightstand")
                .centre(0.021, 0.031)
                .motions(json!({"uses": {"midnightSnack": "drawer_open"}}))
                .paint(&[("stand", "Body"), ("pad", "Top")]),
            Model::new("spritzStation", play::spritz_station, (0.692, 1.62, 0.798))
                .key("model.slob.spritzStation")
                .centre(0.0325, -0.031)
                .motions(json!({"uses": {"spritz": "spritz"}}))
                .paint(&[("shelf", "Body")]),
            Model::new("arcadeCabinet", play::arcade_cabinet, (0.765, 1.88, 0.784))
                .key("model.slob.arcadeCabinet")
                .centre(0.031, -0.039)
                .motions(json!({"loops": {"chaseHighScore": "joystick", "quickRound": "joystick"}}))
                .paint(&[("cabinet", "Body"), ("side", "Sides")]),
            Model::new("gamingRecliner", lounge::gaming_recliner, (1.77, 1.103, 0.874))
                .key("model.slob.gamingRecliner")
                .centre(0.005, 0.01)
                .motions(json!({"uses": {"marathon": "recline", "armrestSoda": "cup_out"}, "ride": "recline"}))
                .paint(&[("seat", "Body"), ("stripe", "Stripes")]),
        ],
        "Slob pack models (crates/art/packs/src/pack_slob, built by cargo art). Listed in the base manifest's packs. \
         The materials map onto the game's finishes so a model costs one draw call per finish group. Each moves while \
         it is used: the microwave's door opens, the beanbag squashes, the snack drawer slides out, the spray tips, the \
         joystick waggles, and the recliner reclines and swings its cup holder out.",
    )
}
