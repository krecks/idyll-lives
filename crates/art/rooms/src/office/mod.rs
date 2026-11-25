//! The office: the desks (O1: the simple, corner, standing and executive desks, the computer, writing and laptop
//! desks and the gaming PC), the desk chairs (O2: the executive, desk and office chairs) and the storage and boards
//! (O3: the filing cabinet, the whiteboard and the corkboard). Notes: NOTES.md.
//!
//! The parts are placed in game axes (`F`, no shift), where the game places it; a desk's top is the content's
//! `surface` height (`DT`, 0.75 for the plain desks, 1.05 standing). Backs against the wall stop at its face (game
//! depth -0.43, -0.93 two tiles deep). The computer, writing, laptop and gaming desks keep the old part's chair in
//! the back half (`characters.ts` SEATS: seat 0.44, 17 cm back), facing the desk, as residents sit there. Moving
//! parts:
//! - the standing and computer desks' keyboard shelf slides out to the resident while they type (`tray_out`);
//! - the writing desk's drawer slides out as they sit down to write (`drawer_open`, shut after 2.5 s);
//! - the laptop opens from shut to 110 degrees while it's used (`lid_open`);
//! - the gaming PC's two case fans spin while a game is played (`fans`, a loop);
//! - a desk chair's seat swivels 15 degrees and back as a resident sits down, carrying them (`swivel`, ride);
//! - the whiteboard's marker lines grow across it while a resident plans (`scribble`).
//!
//! The executive desk's middle drawer and the filing cabinet's top drawer are parts of their own, still: no use
//! opens them yet, so a clip can be added once content gives them one.

use artkit::family::{Family, Model};
use artkit::json;
use artkit::palette;
use artkit::place::GameFrame;

mod boards;
mod cabinet;
mod catalog_desks;
mod chairs;
mod common;
mod desk_things;
mod desks;

palette! { M:
    oak = ("#CDB08A", wood),          // oak tops: the simple, computer, writing and laptop desks
    oak_panel = ("#CDB08A", wood),    // the corner desk's oak panels
    birch = ("#E3CFA8", wood),        // the standing desk's top
    walnut = ("#5A3825", wood),       // the executive desk's top
    walnut_case = ("#5A3825", wood),  // its pedestals, their plinths and the modesty panel
    walnut2 = ("#6B4630", wood),      // its drawer fronts
    laminate = ("#F1F0EC", satin),    // the corner desk's white top
    drawers = ("#F1F0EC", satin),     // painted drawer units and drawer fronts
    frame = ("#2B2C2E", metal),       // black steel: desk frames, legs, chair bases and arms
    shelf = ("#3A3D41", satin),       // the standing desk's keyboard shelf
    tray = ("#3A3D41", satin),        // the computer desk's keyboard tray
    brass = ("#B8925A", metal),       // the executive desk's handles
    chrome = ("#D0D3D6", chrome),     // gas lifts, the filing cabinet's handles
    plastic = ("#2F3236", satin),     // dark plastic: chair shells and hubs, controls, monitor stands
    castor = ("#1E1F22", satin),
    inlay = ("#2F4A3A", leather),     // the executive desk's leather writing pad
    leather = ("#3B2A20", leather),   // the executive chair
    leather2 = ("#2E2019", leather),  // its channels
    fabric = ("#4E6E7A", fabric),     // the desk chair's seat
    mesh = ("#3A3D41", fabric),       // its mesh back
    upholstery = ("#5F6E7A", fabric), // the office chair
    chair = ("#5F6E7A", satin),       // the chairs at the computer and laptop desks
    chair_sage = ("#8FA58A", satin),  // the writing desk's chair
    chair_gamer = ("#3A3D41", satin), // the gaming chair
    monitor = ("#D9DADB", satin),     // the computer desk's monitor
    screen = ("#16202B", gloss),
    wallpaper = ("#2E5470", gloss),
    keys = ("#E8E8E6", satin),        // keyboards and mice
    pad = ("#3A3D41", matte),         // mouse pads, the laptop's touchpad and keys
    paper = ("#FFFFFF", matte),
    notebook = ("#D4A843", matte),
    pen = ("#2A2C30", satin),
    mug = ("#3F8C8C", gloss),
    pot = ("#F5F3EE", gloss),
    soil = ("#4A3A2C", matte),
    cactus = ("#5E8F4E", satin),
    leaf = ("#5E8F4E", foliage),
    lamp = ("#2B2C2E", metal),        // desk lamps
    bulb = ("#FFF1CF", gloss),
    aluminium = ("#C9CBCC", metal),   // the laptop; the whiteboard's frame and tray
    // the gaming PC
    desk = ("#2F3236", satin),
    mat = ("#3A3D41", matte),
    case = ("#22252A", satin),
    accent = ("#6E5CE6", gloss),      // its lights: fan rings, the LED strip, backlit keys
    side = ("#3B2E6E", gloss),        // the tower's tinted side panel
    screen2 = ("#3B2E6E", gloss),
    blade = ("#1E1F22", satin),       // fan blades
    // the filing cabinet
    steel = ("#8E959B", metal),
    label = ("#F2F0EA", gloss),
    folder = ("#C9B79C", matte),
    // the whiteboard and the corkboard
    board = ("#FBFBF9", gloss),
    ink_blue = ("#3E8EDE", gloss),
    ink_red = ("#D9483B", gloss),
    ink_black = ("#2B2C2E", gloss),
    note = ("#F2C14E", matte),
    note2 = ("#F2F0EA", matte),
    note3 = ("#E4B8AE", matte),
    note4 = ("#9DC6CF", matte),
    cork = ("#C49A6C", matte),
    cork_frame = ("#8A5A36", wood),
    pin = ("#D9483B", gloss),
    coffee = ("#4A3121", gloss),
    reveal = ("#1E1F22", matte),      // the dark gaps round drawer fronts
    keycap = ("#2A2C30", satin),      // the laptop's keys
    keycap_light = ("#D4D5D3", satin), // the standing desk's keyboard's keys
    led = ("#5EC28B", gloss),
    leafy = ("#7FA86A", matte),       // the corkboard photo's hill
}

const F: GameFrame = GameFrame::new(0.0);
use artkit::kit::WALL;
const DT: f64 = 0.74; // the second catalogue's desk top
const FT: f64 = 0.7525; // the plain desks' top (content surface 0.75)
const CHAIR_Z: f64 = -0.17; // where a desk's own chair stands (characters.ts SEATS fwd)

const DESK: [(&str, &str); 2] = [("frame", "Frame"), ("drawers", "Drawers")];

/// The family.
pub fn family() -> Option<Family> {
    let swivel = json!({"uses": {"sit": "swivel", "write": "swivel", "homework": "swivel"}, "hold": 0.6, "ride": "swivel"});
    Some(Family::new(
        "office",
        M.palette(),
        vec![
            Model::new("deskSimple", catalog_desks::desk_simple, (1.5, 0.74, 0.69))
                .key("model.deskSimple")
                .centre(0.0, -0.085)
                .paint(&[("oak", "Top"), DESK[0], DESK[1]]),
            Model::new("deskCorner", catalog_desks::desk_corner, (1.86, 1.009, 1.86))
                .key("model.deskCorner")
                .paint(&[
                    ("laminate", "Top"),
                    ("oak_panel", "Frame"),
                    ("drawers", "Drawers"),
                    ("chair", "Chair"),
                ]),
            Model::new("deskStanding", catalog_desks::desk_standing, (0.96, 1.54, 0.63))
                .key("model.deskStanding")
                .centre(0.0, -0.115)
                .motions(json!({"uses": {"browse": "tray_out", "code": "tray_out"}}))
                .paint(&[("birch", "Top"), ("frame", "Frame"), ("shelf", "Shelf")]),
            Model::new("deskExecutive", catalog_desks::desk_executive, (1.8, 0.754, 0.771))
                .key("model.deskExecutive")
                .centre(0.0, -0.0445)
                .paint(&[
                    ("walnut", "Top"),
                    ("walnut_case", "Frame"),
                    ("walnut2", "Drawers"),
                    ("brass", "Handles"),
                    ("inlay", "Desk pad"),
                ]),
            // its own size: the monitor, the things on it and the chair at it are not measured
            Model::new("computerDesk", desks::computer_desk, (1.54, 0.752, 0.391))
                .key("model.computerDesk")
                .centre(0.0, 0.0142)
                .motions(json!({"uses": {"code": "tray_out", "write": "tray_out", "markets": "tray_out", "browse": "tray_out", "homework": "tray_out"}}))
                .paint(&[
                    ("oak", "Top"),
                    DESK[0],
                    DESK[1],
                    ("tray", "Keyboard tray"),
                    ("chair", "Chair"),
                ]),
            // its own size: the chair drawn at it and what lies on it are not measured (a desk is 0.75 m tall)
            Model::new("deskWriting", desks::desk_writing, (1.54, 0.752, 0.433))
                .key("model.deskWriting")
                .centre(0.0, 0.0142)
                .motions(json!({"uses": {"write": "drawer_open", "homework": "drawer_open"}, "hold": 2.5}))
                .paint(&[("oak", "Top"), DESK[0], DESK[1], ("chair_sage", "Chair")]),
            // its own size: the chair at it and what lies on it are not measured
            Model::new("deskLaptop", desks::desk_laptop, (0.9, 0.773, 0.429))
                .key("model.deskLaptop")
                .centre(0.0, 0.0142)
                .motions(json!({"uses": {"browse": "lid_open", "code": "lid_open", "write": "lid_open", "homework": "lid_open"}}))
                .paint(&[
                    ("oak", "Top"),
                    ("frame", "Frame"),
                    ("drawers", "Drawer"),
                    ("aluminium", "Laptop"),
                    ("chair", "Chair"),
                ]),
            Model::new("computerGaming", desks::computer_gaming, (1.7, 1.299, 0.9))
                .key("model.computerGaming")
                .centre(0.0, 0.016)
                .motions(json!({"loops": {"game": "fans"}}))
                .paint(&[
                    ("desk", "Desk"),
                    ("case", "PC & monitors"),
                    ("accent", "Lights"),
                    ("chair_gamer", "Chair"),
                ]),
            Model::new("chairExecutive", chairs::chair_executive, (0.615, 1.336, 0.637))
                .key("model.chairExecutive")
                .centre(0.0, 0.0276)
                .motions(swivel.clone())
                .paint(&[("frame", "Frame"), ("leather", "Seat")]),
            Model::new("chairTask", chairs::chair_task, (0.615, 1.048, 0.603))
                .key("model.chairTask")
                .centre(0.0, 0.0445)
                .motions(swivel)
                .paint(&[("frame", "Frame"), ("fabric", "Seat"), ("mesh", "Back")]),
            Model::new("officeChair", chairs::office_chair, (0.615, 1.157, 0.624))
                .key("model.officeChair")
                .centre(0.0, -0.0162)
                .motions(json!({"uses": {"sit": "swivel"}, "hold": 0.6, "ride": "swivel"}))
                .paint(&[("frame", "Frame"), ("upholstery", "Seat")]),
            Model::new("filingCabinet", cabinet::filing_cabinet, (0.46, 1.21, 0.64))
                .key("model.filingCabinet")
                .centre(0.0, -0.11)
                .motions(json!({"uses": {"file": "drawer_open"}, "hold": 2.0}))
                .paint(&[("steel", "Cabinet"), ("chrome", "Handles")]),
            Model::new("whiteboard", boards::whiteboard, (0.93, 0.647, 0.07))
                .key("model.whiteboard")
                .centre(0.0, -0.395)
                .on_wall()
                .motions(json!({"uses": {"plan": "scribble"}}))
                .paint(&[("board", "Board"), ("aluminium", "Frame")]),
            Model::new("corkboard", boards::corkboard, (0.74, 0.52, 0.033))
                .key("model.corkboard")
                .centre(0.0, -0.4135)
                .on_wall()
                .paint(&[("cork", "Board"), ("cork_frame", "Frame")]),
        ],
        "The office (crates/art/rooms/src/office, built by cargo art): the desks, the desk chairs, the filing cabinet \
         and the boards. Listed in the base manifest's packs. The materials map onto the game's finishes so a model \
         costs one draw call per finish group. The keyboard shelves slide out (`tray_out`), the writing desk's \
         drawer opens (`drawer_open`), the laptop opens (`lid_open`), the gaming PC's fans spin (`fans`), the \
         chairs swivel (`swivel`) and the whiteboard fills with a plan (`scribble`) while residents use them.",
    ))
}
