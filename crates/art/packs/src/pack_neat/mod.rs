//! The neat pack (P13): the steam mop dock, the stacked laundry centre, the rainfall steam shower, the label-and-sort
//! organizer desk, the robot vacuum and its dock and the tidy entry console. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, or on
//! its `Model`'s centre), one moving part each: the mop leaves its dock for the hands (`mop_off`), the washer's door
//! swings open (`door_open`), the rain falls (`water_on`) and the shower's glass door swings open (`door_open`), a
//! bin slides out of the desk's hutch (`drawer_open`), the robot rolls out of its dock and round its rug
//! (`vacuum_round`) and the console's key tray drawer slides out (`tray`).
//!
//! Appliances are lofted through rounded rectangles (`loft`, `ring`), their portholes turned (`revolve`); bins,
//! bottles and baskets lofted too (the baskets' weave in ridges); towels, pads and cushions stuffed (`soft`); poles,
//! hangers, hooks and rails swept in one piece; the mirror framed with the kit's moulding, the drawer's knob the
//! kit's knob.
//!
//! Run:  cargo art pack_neat [--only steamMopDock] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod common;
mod desk;
mod dock;
mod laundry;

pub use common::M;

/// The family.
// (0.318: the console's depth, not 1/pi)
#[allow(clippy::approx_constant)]
pub fn family() -> Option<Family> {
    Some(Family::new(
        "pack_neat",
        M.palette(),
        vec![
            Model::new("steamMopDock", dock::steam_mop_dock, (0.788, 1.25, 0.708))
                .key("model.neat.steamMopDock")
                .centre(-0.006, -0.076)
                .motions(json!({"uses": {"mop": "mop_off"}}))
                .paint(&[("body", "Body"), ("teal", "Mop")]),
            Model::new("laundryCenter", laundry::laundry_center, (1.82, 1.738, 0.625))
                .key("model.neat.laundryCenter")
                .centre(0.01, -0.1175) // (-0.127 before 2026-10-22: its shirts stood 1 cm into the wall)
                // the porthole opens to load the washing and to take it out, and shuts for the wash and the dry
                .motions(json!({"uses": {"laundry": "door_open", "unload": "door_open"}, "hold": 2.5}))
                .paint(&[("body", "Body"), ("appliance", "Washer & dryer")]),
            Model::new("rainShowerTower", laundry::rain_shower_tower, (0.9, 2.18, 0.9))
                .key("model.neat.rainShowerTower")
                .centre(0.0, 0.02)
                // the door lets the bather in, shuts behind them and opens again to let them out (as the glasshouse's)
                .motions(json!({"uses": {"steamShower": [{"clip": "door_open", "hold": 1.5, "out": 1.5}, "water_on"]}}))
                .paint(&[("tile", "Body"), ("teak", "Bench")]),
            Model::new("organizerDesk", desk::organizer_desk, (1.6, 1.25, 0.905))
                .key("model.neat.organizerDesk")
                .centre(0.0, 0.0225)
                .motions(json!({"uses": {"sort": "drawer_open", "ledger": "drawer_open"}}))
                .paint(&[("body", "Body"), ("birch", "Wood"), ("pastel_c", "Chair")]),
            Model::new("robotVacuumDock", dock::robot_vacuum_dock, (0.86, 0.519, 0.86))
                .key("model.neat.robotVacuumDock")
                // once programmed the robot drives out of its dock, round its rug and back, again and again
                .motions(json!({"loops": {"program": "vacuum_round"}}))
                .paint(&[("appliance", "Body"), ("teal", "Trim"), ("rug", "Rug")]),
            Model::new("entryConsole", desk::entry_console, (0.87, 1.805, 0.318))
                .key("model.neat.entryConsole")
                .centre(-0.005, -0.262)
                .motions(json!({"uses": {"tidy": "tray"}}))
                .paint(&[("body", "Body"), ("birch", "Wood")]),
        ],
        "Neat pack models (crates/art/packs/src/pack_neat, built by cargo art). Listed in the base manifest's packs. \
         The materials map onto the game's finishes so a model costs one draw call per finish group. Each moves one \
         part while it is used: the mop leaves its dock for the resident's hands, the washer's door opens, the rain \
         falls, a bin slides out, the robot rolls out and the key tray drawer opens.",
    ))
}
