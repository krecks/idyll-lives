//! The couch potato pack (P3): the grand cinema wall, the Pixel Arcade console, the Cloud Nine power recliner, the
//! sofa-side snack fridge, the living-room workout game and the lava glow lamp. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre). The cinema's two side curtains are drawn apart (`curtains`), the console's disc tray
//! slides out (`tray`), the recliner's back tips and its footrest rises (`recline`), the fridge door swings
//! (`fridge_open`), the balance board tilts (`board`) and the lava lamp's wax rises (`blobs`) while more wax
//! drifts round all the time (`drift`, ambient). The screens the residents watch (the cinema's, the console's)
//! stand on the wall at the back, the seats in front of them, the sitters facing the object's back (content
//! `seat.facing: "back"`, as the piano): each is built as its old part first stood (the seats at the back, the
//! screen at the front edge facing them) and turned round (`turn_round`), as is its old part.
//!
//! Built with the forms and the kit: stuffed, piped and rolled upholstery (`soft`, swept arms), screens and
//! cabinets extruded from rounded outlines, speaker drivers turned in one piece, pleated curtains with a heading,
//! lofted cans, cups and glass.
//!
//! Run:  cargo art pack_couchPotato [--only recliner] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod cinema;
mod common;
mod lounge;

pub use common::M;

/// The family.
pub fn family() -> Family {
    Family::new(
        "pack_couchPotato",
        M.palette(),
        vec![
            Model::new("cinemaWall", cinema::cinema_wall, (1.88, 1.52, 0.91))
                .key("model.couchPotato.cinemaWall")
                .centre(0.0, 0.025)
                .motions(json!({"uses": {"marathon": "curtains", "channelSurf": "curtains"}}))
                .paint(&[
                    ("cinema_frame", "Body"),
                    ("leather", "Upholstery"),
                    ("curtain", "Curtains"),
                ]),
            Model::new("gameConsole", cinema::game_console, (0.89, 1.34, 0.9))
                .key("model.couchPotato.gameConsole")
                .centre(0.0, 0.02)
                .motions(json!({"uses": {"play": "tray", "speedrun": "tray"}, "hold": 1.5}))
                .paint(&[
                    ("console", "Body"),
                    ("gamer", "Upholstery"),
                    ("mustard", "Cushion"),
                ]),
            Model::new("recliner", lounge::recliner, (0.94, 0.948, 0.77))
                .key("model.couchPotato.recliner")
                .centre(-0.01, -0.015)
                .motions(
                    json!({"uses": {"recline": "recline", "doze": "recline"}, "ride": "recline"}),
                )
                .paint(&[
                    ("plinth", "Body"),
                    ("plush", "Upholstery"),
                    ("throw", "Throw"),
                ]),
            Model::new("snackFridge", lounge::snack_fridge, (0.58, 0.538, 0.539))
                .key("model.couchPotato.snackFridge")
                .centre(0.0, 0.0)
                .motions(json!({"uses": {"snack": "fridge_open", "takeout": "fridge_open"}}))
                .paint(&[("mint", "Body"), ("lid", "Upholstery")]),
            Model::new("fitnessGame", lounge::fitness_game, (0.865, 1.155, 0.88))
                .key("model.couchPotato.fitnessGame")
                .centre(0.032, 0.01)
                .motions(json!({"uses": {"workout": "board"}}))
                .paint(&[
                    ("fit_stand", "Body"),
                    ("fit_mat", "Mat"),
                    ("board", "Board"),
                ]),
            Model::new("lavaLamp", lounge::lava_lamp, (0.26, 1.155, 0.26))
                .key("model.couchPotato.lavaLamp")
                .centre(0.0019, 0.0232)
                .motions(json!({"uses": {"zoneOut": "blobs"}, "idle": "drift"}))
                .paint(&[
                    ("lamp_metal", "Body"),
                    ("pouf", "Upholstery"),
                    ("vessel", "Glass"),
                ]),
        ],
        "Couch potato pack models (crates/art/packs/src/pack_couch_potato, built by cargo art). Listed in the \
         base manifest's packs. The materials map onto the game's finishes so a model costs one draw call per \
         finish group. The cinema's curtains are drawn apart, the console's tray slides out, the recliner \
         reclines, the fridge door swings, the balance board tilts and the lava lamp's wax rises and drifts.",
    )
}
