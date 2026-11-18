//! The bathroom sinks: the base sink (`model.sink`), the pedestal (`model.sinkPedestal`), the double vanity
//! (`model.vanityDouble`), the vessel sink (`model.sinkVessel`) and the console sink (`model.sinkConsole`); and the
//! five toilets (`model.toilet`, `model.toiletCompact`, `model.toiletSmart`, `model.toiletClassic`,
//! `model.toiletWallHung`) and the four showers (`model.shower`, `model.showerStall`, `model.showerWalkIn`,
//! `model.showerRain`) and the three baths (`model.bathtub`, `model.bathtubClawfoot`, `model.bathtubFreestanding`);
//! and the mirror cabinet, the round mirror and the tall cabinet. Notes: NOTES.md.
//!
//! Each sink is one still body (cabinet / basin / pedestal / legs) plus one moving part: a stream of water
//! (`water`) that grows from the tap's spout into the basin while a resident washes (`motions`). Each toilet is
//! one still body plus its lid, which lifts while a resident uses it; each shower one still body plus its rain,
//! which grows down from the head while a resident showers (and the base shower's sliding glass door, the stall's
//! curtain front, each a part of its own); each bath one still body plus its tap's stream and its water, which
//! fills while a resident bathes. Built to the old part's size, in its numbers: the parts are placed in game axes
//! (`GameFrame`).
//!
//! Run:  cargo art bathroom [--only sink] [--preview]

use artkit::family::{Family, Model};
use artkit::json::Json;
use artkit::materials::Mirror;
use artkit::palette;

mod bath_parts;
mod baths;
mod common;
mod open_showers;
mod shower_parts;
mod showers;
mod sinks;
mod storage;
mod toilets;
mod vanities;

palette! { M:
    body = ("#EDEBE6", satin),       // cabinet / vanity / pedestal / basin; the main part the player colours
    porcelain = ("#F6F6F4", gloss),  // white ceramic
    stone = ("#DCDDDA", gloss),      // worktops and vanity tops
    wood = ("#B98A5A", wood),        // wooden tops, shelves and legs
    trim = ("#B9BCBF", satin),       // plinths and seals
    metal = ("#8E9196", metal),      // basin interiors, grey metal
    chrome = ("#D0D3D6", chrome),    // chrome taps
    brass = ("#C9A54A", metal),      // brass taps and legs
    blackmetal = ("#2F3236", metal), // black taps and legs
    fabric = ("#E4B8AE", fabric),    // towels (a woven texture: its own draw call)
    towel = ("#E4B8AE", satin),      // towels on the vessel sink, kept plain to save a draw call
    dark = ("#2F3236", satin),       // drains and seals
    teal = ("#4E8A7A", gloss),       // soap and cup accents
    water = ("#9EC9DD", gloss),      // the stream from the tap
    // the toilets
    seat = ("#EFEFEC", gloss),       // a toilet's seat and lid
    seat_wood = ("#8A5A36", wood),   // the classic toilet's wooden seat and lid
    cistern = ("#F6F6F4", gloss),    // a cistern on the bowl
    panel = ("#E4E3DF", satin),      // the smart toilet's boxed-in cistern
    boxing = ("#C9C6C0", matte),     // the wall-hung toilet's boxed-in cistern
    screen = ("#2A2C30", gloss),     // flush plates and the smart toilet's display
    led = ("#5EC28B", gloss),        // the smart toilet's ready light
    paper = ("#FBFBF8", matte),      // toilet rolls
    // the showers
    tray = ("#F4F4F2", gloss),       // a shower tray
    tile = ("#E4ECEC", satin),       // wall tiles
    grout = ("#C8D3D4", satin),      // the grout between them
    glass = ("#DCE6E8", glass),      // screens and doors (see-through: its own draw call)
    grip = ("#7E9096", glass),       // the shower door's grip, smoked glass (drawn with the door's glass)
    frame = ("#D0D3D6", chrome),     // a screen's frame, a curtain rail
    fittings = ("#D0D3D6", chrome),  // heads, risers and mixers
    frame_dark = ("#2B2C2E", metal),    // a black screen frame
    fittings_dark = ("#2B2C2E", metal), // a black rain head and mixer
    curtain = ("#CFE0E4", satin),    // the stall's curtain: polyester, plain (the moving front costs no extra draw)
    slate = ("#C9C6C0", matte),      // the walk-in shower's stone floor
    slate_wall = ("#C9C6C0", matte), // and its stone wall
    groove = ("#AFACA6", matte),     // the joints in the stone wall
    tray_stone = ("#E8E6E1", matte), // the rain shower's stone tray
    wall_stone = ("#D7D3CC", matte), // and its walls
    bottle = ("#F2F1EE", gloss),     // shampoo bottles
    sage = ("#9DAF8C", gloss),
    // the baths
    bath_panel = ("#ECECE8", gloss), // the built-in bath's front and end panels
    feet = ("#B8925A", metal),       // the clawfoot tub's feet
    duck = ("#F2C14E", gloss),       // a rubber duck
    beak = ("#E9853B", gloss),
    // the bathroom storage and mirrors
    cab = ("#F2F1EE", satin),        // a bathroom cabinet's carcass
    cab_door = ("#E4E3DF", satin),   // its doors and the mirror's frame
    mirror = ("#CFDCE2", gloss, mirror(Mirror::Front)), // mirror glass: reflects the room live on High, plain gloss elsewhere
    bulb = ("#FFF1CF", gloss),       // the mirror cabinet's light strip
    frame_brass = ("#B8925A", metal), // the round mirror's frame and hook
    leather = ("#6B4A33", satin),    // its strap
    towel_blue = ("#9DC6CF", satin),
    towel_cream = ("#F2F0EA", satin),
}

// The parts the player may colour, the biggest first (the cabinet, vanity, pedestal, console basin or vessel
// vanity: the one the quick colour row paints). Drains, water, soap and the basins' metal insides keep their own
// material.

/// `{'uses': {<tag>: <clip>}}`.
fn uses(tag: &str, clip: Json) -> Json {
    Json::Obj(vec![("uses".into(), Json::Obj(vec![(tag.into(), clip)]))])
}

/// A clip in a use's list, with its timings: `{'clip': .., 'hold': .., ...}`.
fn clip(name: &str, timings: &[(&str, f64)]) -> Json {
    let mut fields = vec![("clip".to_string(), Json::from(name))];
    fields.extend(timings.iter().map(|&(k, v)| (k.to_string(), Json::Num(v))));
    Json::Obj(fields)
}

fn wash() -> Json {
    uses("wash", Json::from("water_on"))
}

fn toilet() -> Json {
    uses("toilet", Json::from("lid_up"))
}

fn bath() -> Json {
    uses(
        "bath",
        Json::Arr(vec![
            clip(
                "tap_on",
                &[("hold", bath_parts::BATH_TAP + bath_parts::BATH_FILL)],
            ),
            Json::from("fill"),
        ]),
    )
}

fn shower() -> Json {
    uses("shower", Json::from("water_on"))
}

/// The base shower's door slides open as the resident steps in, shuts behind them, and opens again to let them
/// out; the water comes on once it's shut. The stall's curtain is drawn across once they're in, back at the end.
fn shower_door() -> Json {
    uses(
        "shower",
        Json::Arr(vec![
            clip("door_open", &[("hold", 1.5), ("out", 1.5)]),
            clip("water_on", &[("delay", 2.0)]),
        ]),
    )
}

fn shower_curtain() -> Json {
    uses(
        "shower",
        Json::Arr(vec![
            clip("curtain_draw", &[("delay", 1.0)]),
            clip("water_on", &[("delay", 1.8)]),
        ]),
    )
}

/// `{'uses': {<tag>: 'door_open'}, 'hold': 1.5}`.
fn door_held(tag: &str) -> Json {
    let Json::Obj(mut fields) = uses(tag, Json::from("door_open")) else {
        unreachable!()
    };
    fields.push(("hold".into(), Json::Num(1.5)));
    Json::Obj(fields)
}

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "bathroom",
        M.palette(),
        vec![
            Model::new("sink", sinks::sink, (0.86, 1.262, 0.621))
                .key("model.sink")
                .centre(0.0, -0.121)
                .motions(wash())
                .paint(&[
                    ("body", "Cabinet"),
                    ("stone", "Worktop"),
                    ("metal", "Basin"),
                    ("chrome", "Tap & handles"),
                    ("trim", "Plinth"),
                ]),
            Model::new("sinkPedestal", sinks::sink_pedestal, (0.56, 1.060, 0.464))
                .key("model.sinkPedestal")
                .centre(0.0, -0.195)
                .motions(wash())
                .paint(&[
                    ("body", "Pedestal"),
                    ("porcelain", "Basin & cup"),
                    ("chrome", "Tap"),
                ]),
            Model::new(
                "vanityDouble",
                vanities::vanity_double,
                (1.90, 1.002, 0.562),
            )
            .key("model.vanityDouble")
            .centre(0.0, -0.149)
            .motions(wash())
            .paint(&[
                ("body", "Cabinet"),
                ("stone", "Worktop"),
                ("porcelain", "Basins"),
                ("chrome", "Taps & handles"),
                ("trim", "Plinth"),
            ]),
            Model::new("sinkVessel", vanities::sink_vessel, (0.84, 1.060, 0.540))
                .key("model.sinkVessel")
                .centre(0.0, -0.16)
                .motions(wash())
                .paint(&[
                    ("body", "Vanity"),
                    ("stone", "Worktop"),
                    ("porcelain", "Bowl"),
                    ("blackmetal", "Tap"),
                    ("wood", "Shelf & legs"),
                    ("towel", "Towels"),
                ]),
            Model::new("sinkConsole", vanities::sink_console, (0.60, 1.062, 0.480))
                .key("model.sinkConsole")
                .centre(0.0, -0.188)
                .motions(wash())
                .paint(&[
                    ("body", "Basin"),
                    ("brass", "Legs & tap"),
                    ("fabric", "Towel"),
                ]),
            Model::new("toilet", toilets::toilet, (0.44, 0.815, 0.676))
                .key("model.toilet")
                .centre(0.0, -0.092)
                .motions(toilet())
                .paint(&[
                    ("porcelain", "Bowl"),
                    ("seat", "Seat & lid"),
                    ("cistern", "Cistern"),
                ]),
            Model::new(
                "toiletCompact",
                toilets::toilet_compact,
                (0.59, 0.809, 0.631),
            )
            .key("model.toiletCompact")
            .centre(0.09, -0.1145)
            .motions(toilet())
            .paint(&[
                ("porcelain", "Bowl"),
                ("seat", "Seat & lid"),
                ("cistern", "Cistern"),
            ]),
            Model::new("toiletSmart", toilets::toilet_smart, (0.9, 1.2, 0.714))
                .key("model.toiletSmart")
                .centre(0.0, -0.073)
                .motions(toilet())
                .paint(&[
                    ("porcelain", "Bowl"),
                    ("seat", "Seat & lid"),
                    ("panel", "Cistern"),
                ]),
            Model::new(
                "toiletClassic",
                toilets::toilet_classic,
                (0.47, 0.875, 0.712),
            )
            .key("model.toiletClassic")
            .centre(0.0, -0.074)
            .motions(toilet())
            .paint(&[
                ("porcelain", "Bowl"),
                ("seat_wood", "Seat & lid"),
                ("cistern", "Cistern"),
            ]),
            Model::new(
                "toiletWallHung",
                toilets::toilet_wall_hung,
                (0.6, 1.1, 0.665),
            )
            .key("model.toiletWallHung")
            .centre(0.0, -0.0975)
            .motions(toilet())
            .paint(&[
                ("porcelain", "Bowl"),
                ("seat", "Seat & lid"),
                ("boxing", "Cistern"),
            ]),
            Model::new("shower", showers::shower, (0.9, 2.15, 0.9))
                .key("model.shower")
                .centre(0.0, 0.02)
                .motions(shower_door())
                .paint(&[
                    ("tray", "Tray"),
                    ("frame", "Frame"),
                    ("fittings", "Head & tap"),
                    ("tile", "Tiles"),
                ]),
            Model::new("showerStall", showers::shower_stall, (0.9, 2.08, 0.9))
                .key("model.showerStall")
                .centre(0.0, 0.02)
                .motions(shower_curtain())
                .paint(&[
                    ("tray", "Tray"),
                    ("frame", "Rail"),
                    ("fittings", "Head & tap"),
                    ("curtain", "Curtain"),
                    ("tile", "Tiles"),
                ]),
            Model::new(
                "showerWalkIn",
                open_showers::shower_walk_in,
                (0.92, 2.26, 0.92),
            )
            .key("model.showerWalkIn")
            .centre(0.0, 0.03)
            .motions(shower())
            .paint(&[
                ("slate", "Floor"),
                ("frame_dark", "Frame"),
                ("fittings_dark", "Head & tap"),
                ("slate_wall", "Wall"),
            ]),
            Model::new("showerRain", open_showers::shower_rain, (0.94, 2.192, 0.93))
                .key("model.showerRain")
                .centre(0.0, 0.035)
                .motions(shower())
                .paint(&[
                    ("tray_stone", "Tray"),
                    ("frame_dark", "Frame"),
                    ("fittings_dark", "Head & tap"),
                    ("wall_stone", "Walls"),
                ]),
            Model::new("bathtub", baths::bathtub, (1.82, 0.69, 0.86))
                .key("model.bathtub")
                .motions(bath())
                .paint(&[
                    ("porcelain", "Tub"),
                    ("bath_panel", "Panel"),
                    ("chrome", "Taps"),
                    ("towel", "Towel"),
                ]),
            Model::new(
                "bathtubClawfoot",
                baths::bathtub_clawfoot,
                (1.73, 1.02, 0.829),
            )
            .key("model.bathtubClawfoot")
            .motions(bath())
            .paint(&[
                ("porcelain", "Tub"),
                ("feet", "Feet"),
                ("brass", "Taps"),
                ("towel", "Towel"),
            ]),
            Model::new(
                "bathtubFreestanding",
                baths::bathtub_freestanding,
                (1.895, 0.919, 0.837),
            )
            .key("model.bathtubFreestanding")
            .motions(bath())
            .paint(&[("porcelain", "Tub"), ("chrome", "Taps"), ("towel", "Towel")]),
            Model::new("mirrorCabinet", storage::mirror_cabinet, (0.56, 0.77, 0.2))
                .key("model.mirrorCabinet")
                .centre(0.0, -0.33)
                .on_wall()
                .motions(door_held("brushTeeth"))
                .paint(&[("cab", "Cabinet"), ("cab_door", "Frame")]),
            Model::new("mirrorRound", storage::mirror_round, (0.64, 0.772, 0.046))
                .key("model.mirrorRound")
                .centre(0.0, -0.407)
                .on_wall()
                .paint(&[("frame_brass", "Frame"), ("leather", "Strap")]),
            Model::new(
                "bathCabinetTall",
                storage::bath_cabinet_tall,
                (0.42, 1.82, 0.38),
            )
            .key("model.bathCabinetTall")
            .centre(0.0, -0.24)
            .motions(door_held("takeTowel"))
            .paint(&[
                ("cab", "Cabinet"),
                ("cab_door", "Doors"),
                ("towel_blue", "Towels"),
            ]),
        ],
        "The bathroom (crates/art/wet/src/bathroom, built by cargo art). Listed in the base manifest's packs. \
         The materials map onto the game's finishes so a model costs one draw call per finish group. Each \
         sink has a `water_on` clip (the stream from the tap) played while a resident washes.",
    ).switched())
}
