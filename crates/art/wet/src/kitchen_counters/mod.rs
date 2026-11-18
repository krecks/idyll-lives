//! The kitchen's base counters (K4): the plain counter and its corner, the counter end with open shelves, the outer
//! corner counter, the drawer counter, the open-shelf counter, the butcher block and the marble counter; the tall and
//! wall storage (K5); the islands and bars (K6). Notes: NOTES.md.
//!
//! A base counter is a plinth, a carcass and a worktop; the front carries a drawer over two doors (the plain, corner
//! and marble counters), four drawers, one door, or nothing (the open shelves). The four with a `sandwich` use slide
//! their top drawer out 25 cm. The parts are placed in game axes (`GameFrame`): the old part's depths are shifted
//! back by the old part's depth centre, so the model is centred on its tile. There is one worktop height, 0.95, the
//! content's `surface`, for the catalogue counters (`unit66`) and the catalog2 ones (`unit79`) alike. There is one
//! depth, the catalogue's 0.66: the catalog2 counters keep the old 0.79-deep numbers and are drawn 0.66 deep through
//! `COUNTER66` (as the old `shallow`), so the fronts of a mixed run line up. Every back stands on the wall face
//! (-0.43).
//!
//! The worktop's nose is rolled over and its ends cut square (`worktop`), the fronts are shaker doors and drawers
//! (`shaker`) with swept pulls (`pull`), the drawers hold cutlery; what stands on and in them is turned (`lathe`).
//! Fronts, handles and the worktop are placed at drawn depths (`plain`, `GameFrame::depth`), so they keep their real
//! thickness on a counter drawn through COUNTER66, and no pull reaches past the worktop's nose.
//!
//! Run:  cargo art kitchen_counters [--preview]

use artkit::family::{Family, Model};
use artkit::json::Json;
use artkit::{json, palette};

mod bins;
mod block;
mod cabinets;
mod common;
mod counters;
mod islands;
mod storage;
mod turned;
mod wall;

palette! { M:
    cab = ("#EDEBE6", satin),               // a cabinet carcass and its fronts; the main part the player colours
    worktop = ("#E8E6E1", gloss, roughness(0.3)), // a laminate worktop
    dark = ("#2F3236", satin),              // plinths
    chrome = ("#D0D3D6", chrome),           // handles and pulls
    oak = ("#CDB08A", wood),                // a wood end unit, open shelves and a utensil handle
    navy = ("#3E5470", satin),              // a painted cabinet (the marble counter)
    marble = ("#F4F3F0", gloss, roughness(0.28)), // a marble worktop
    vein = ("#C9CBCC", gloss),              // a vein through the marble
    brass = ("#B8925A", metal),             // brass handles
    block = ("#B98A5A", wood),              // a butcher-block top
    block2 = ("#C99A68", wood),             // the top layer of the block
    leg = ("#9C6B43", wood),                // the block's legs and shelf
    wood2 = ("#8A5A38", satin),             // the knife drawer's front (plain, so it draws with the body's group)
    spoon = ("#CDB08A", satin),             // the corner counter's wooden spoons (plain: no draw call of their own)
    basket = ("#C9A979", satin),            // a basket
    metal = ("#2B2C2E", metal),             // a dark metal handle
    porcelain = ("#F5F3EE", gloss),         // plates, a bowl, a jar
    cream = ("#EFE5D3", gloss),             // a cream jar
    rust = ("#B5653F", gloss),              // a terracotta pot
    teal = ("#4E7A7F", gloss),              // a teal pot
    sage = ("#B9C9C4", gloss),              // a sage pot
    yellow = ("#F2C14E", satin),            // a lemon
    soil = ("#3B2E24", matte),              // the dark back of an open shelf
    oakd = ("#8A5A38", wood),               // a darker wood: the brackets under a wall shelf
    steel = ("#B9BDC1", metal),             // a cooker hood, a bin: brushed stainless
    trim = ("#3A3D41", satin),              // dark trim, keys and a hood's filter
    bulb = ("#FFF1CF", gloss),              // a strip light or a hood lamp
    copper = ("#B87333", metal),            // copper pans on the pot rail
    glass = ("#DCE6EA", glass),             // glazed doors, jars and glasses
    back = ("#C9D6DA", satin),              // a cabinet's back, seen through its glass
    leaf = ("#5E8F4E", satin),              // a pot plant's leaves (plain: foliage is a draw call of its own)
    bin1 = ("#3E7CC2", satin),              // recycling bins: blue, green and yellow
    bin2 = ("#4E9A5A", satin),
    bin3 = ("#E3B23C", satin),
    lid = ("#2F3236", satin),               // a bin's lid
    walnut = ("#7A5234", satin),            // a bar's body and slats
    island = ("#9DAF8C", satin),            // the kitchen island's sage stool seats
    towel = ("#B4483C", satin),             // a red tea towel
}

pub const SECONDS: f64 = 1.4; // a drawer slides in this long
const HOLD: f64 = 1.5; // seconds it stays open once a use starts
pub const DOOR_SECONDS: f64 = 1.6; // a cupboard door swings in this long
const DOOR_HOLD: f64 = 2.5; // seconds a pantry's doors stay open once a use starts
pub const WALL_DOOR_SECONDS: f64 = 0.8; // a wall cabinet's door swings open in this long, while clean things are put away
// catalog2's 0.79-deep counter numbers (worktop -0.48..0.31, carcass -0.46..0.24) as drawn: 0.66 deep, the
// worktop -0.43..0.23 and the carcass -0.41..0.19 (the old `COUNTER66`; GameFrame `depths`)
pub const COUNTER66: [(f64, f64); 4] = [(-0.48, -0.43), (-0.46, -0.41), (0.24, 0.19), (0.31, 0.23)];

/// Sorting the recycling lifts the three lids one after another.
fn sort_recycling() -> Json {
    let lids: Vec<Json> = (0..3)
        .map(|i| json!({"clip": (format!("lid{i}_open")), "delay": (0.6 * i as f64)}))
        .collect();
    json!({"uses": {"sortRecycling": (Json::Arr(lids))}, "hold": 0.9})
}

const BASE_PAINT: [(&str, &str); 3] = [
    ("cab", "Cabinet"),
    ("worktop", "Worktop"),
    ("chrome", "Handles"),
];

/// The family.
pub fn family() -> Option<Family> {
    // A floor thing against a wall is centred where the game places it (`centre`): its back on the wall,
    // its top under its content `surface`.
    Some(Family::new(
        "kitchen_counters",
        M.palette(),
        vec![
            Model::new("counter", counters::counter, (1.0, 0.95, 0.66))
                .key("model.counter")
                .centre(0.0, -0.10)
                // sandwich slides the top drawer out
                .motions(json!({"uses": {"sandwich": "drawer_open"}, "hold": (HOLD)}))
                .paint(&BASE_PAINT),
            Model::new("counterCorner", counters::counter_corner, (1.0, 1.26, 0.93))
                .key("model.counterCorner")
                .centre(0.0, 0.035)
                // sandwich, as the counter
                .motions(json!({"uses": {"sandwich": "drawer_open"}, "hold": (HOLD)}))
                .paint(&BASE_PAINT),
            Model::new("counterEnd", counters::counter_end, (1.0, 0.95, 0.66))
                .key("model.counterEnd")
                .centre(0.0, -0.10)
                .paint(&BASE_PAINT),
            Model::new(
                "counterCornerOuter",
                counters::counter_corner_outer,
                (0.81, 1.035, 0.66),
            )
            .key("model.counterCornerOuter")
            .centre(-0.095, -0.10)
            .paint(&BASE_PAINT),
            Model::new(
                "counterDrawers",
                counters::counter_drawers,
                (1.0, 0.95, 0.66),
            )
            .key("model.counterDrawers")
            .centre(0.0, -0.10)
            // putting clean dishes away here (cooking putAwayAt prep, anim fetch) slides the top drawer out
            .motions(json!({"uses": {"fetch": "drawer_open"}, "hold": (HOLD)}))
            .paint(&BASE_PAINT),
            Model::new(
                "counterOpenShelf",
                counters::counter_open_shelf,
                (1.0, 0.95, 0.66),
            )
            .key("model.counterOpenShelf")
            .centre(0.0, -0.10)
            .paint(&[("cab", "Cabinet"), ("worktop", "Worktop")]),
            Model::new("butcherBlock", block::butcher_block, (0.9, 0.95, 0.813))
                .key("model.butcherBlock")
                // sandwich slides the knife drawer out
                .motions(json!({"uses": {"sandwich": "drawer_open"}, "hold": (HOLD)}))
                .paint(&[("leg", "Base"), ("block", "Block"), ("block2", "Top")]),
            Model::new("counterMarble", block::counter_marble, (1.0, 0.953, 0.66))
                .key("model.counterMarble")
                .centre(0.0, -0.10)
                // sandwich slides the top drawer out
                .motions(json!({"uses": {"sandwich": "drawer_open"}, "hold": (HOLD)}))
                .paint(&[
                    ("navy", "Cabinet"),
                    ("marble", "Marble top"),
                    ("brass", "Handles"),
                ]),
            // K5: tall and wall storage
            Model::new("pantry", storage::pantry, (0.94, 2.27, 0.63))
                .key("model.pantry")
                .centre(0.0, -0.115)
                // snack fetches from the larder: both doors swing open
                .motions(json!({"uses": {"snack": "doors_open"}, "hold": (DOOR_HOLD)}))
                .paint(&[("cab", "Cabinet"), ("chrome", "Handles")]),
            Model::new("pantryTall", storage::pantry_tall, (0.96, 2.38, 0.71))
                .key("model.pantryTall")
                .centre(0.0, -0.08)
                // snack, as the larder
                .motions(json!({"uses": {"snack": "doors_open"}, "hold": (DOOR_HOLD)}))
                .paint(&[("cab", "Cabinet"), ("brass", "Handles")]),
            // putting clean dishes away (a `cupboard` station, cooking putAwayAt, anim fetch) opens the doors
            Model::new("wallCabinet", cabinets::wall_cabinet, (0.98, 0.732, 0.379))
                .key("model.wallCabinet")
                .centre(0.0, -0.2405)
                .on_wall()
                .motions(json!({"uses": {"fetch": "doors_open"}, "hold": (HOLD)}))
                .paint(&[("cab", "Cabinet"), ("chrome", "Handles")]),
            Model::new(
                "wallCabinetGlass",
                cabinets::wall_cabinet_glass,
                (0.96, 0.72, 0.406),
            )
            .key("model.wallCabinetGlass")
            .centre(0.0, -0.227)
            .on_wall()
            .motions(json!({"uses": {"fetch": "door_open"}, "hold": (HOLD)}))
            .paint(&[("cab", "Cabinet"), ("chrome", "Handles")]),
            Model::new(
                "wallCabinetCorner",
                cabinets::wall_cabinet_corner,
                (0.96, 0.722, 0.406),
            )
            .key("model.wallCabinetCorner")
            .centre(0.0, -0.227)
            .on_wall()
            .motions(json!({"uses": {"fetch": "door_open"}, "hold": (HOLD)}))
            .paint(&[("cab", "Cabinet"), ("chrome", "Handles")]),
            Model::new("rangeHood", wall::range_hood, (0.9, 0.712, 0.515))
                .key("model.rangeHood")
                .centre(0.0, -0.1725)
                .on_wall()
                .paint(&[("steel", "Hood"), ("trim", "Trim")]),
            Model::new("spiceRack", wall::spice_rack, (0.52, 0.44, 0.106))
                .key("model.spiceRack")
                .centre(0.0, -0.377)
                .on_wall()
                .paint(&[("oak", "Rack")]),
            Model::new("potRail", wall::pot_rail, (0.91, 0.545, 0.22))
                .key("model.potRail")
                .centre(0.0, -0.32)
                .on_wall()
                .paint(&[("metal", "Rail"), ("copper", "Pots")]),
            Model::new(
                "kitchenShelfWall",
                wall::kitchen_shelf_wall,
                (0.9, 0.76, 0.24),
            )
            .key("model.kitchenShelfWall")
            .centre(0.0, -0.31)
            .on_wall()
            .paint(&[("oak", "Shelves"), ("oakd", "Brackets")]),
            Model::new("trashCan", bins::trash_can, (0.31, 0.685, 0.36))
                .key("model.trashCan")
                .centre(0.0, -0.247)
                // scraping a plate into the bin (the cooking `bin` station, anim clean) lifts the lid
                .motions(json!({"uses": {"clean": "lid_open"}, "hold": 1.0}))
                .paint(&[("steel", "Bin"), ("lid", "Lid")]),
            Model::new("recyclingBins", bins::recycling_bins, (0.92, 0.66, 0.455))
                .key("model.recyclingBins")
                .centre(0.0, -0.212)
                .motions(sort_recycling())
                // as seen from the front
                .paint(&[
                    ("bin3", "Left bin"),
                    ("bin2", "Middle bin"),
                    ("bin1", "Right bin"),
                    ("lid", "Handles"),
                ]),
            // K6: islands and bars
            Model::new(
                "kitchenIsland",
                islands::kitchen_island,
                (1.94, 0.95, 0.839),
            )
            .key("model.kitchenIsland")
            .centre(0.0, 0.0495)
            // sandwich slides the drawer out (breakfast is eaten standing, or on a stool at the island)
            .motions(json!({"uses": {"sandwich": "drawer_open"}, "hold": (HOLD)}))
            .paint(&[("cab", "Cabinet"), ("worktop", "Worktop")]),
            Model::new("islandSmall", islands::island_small, (0.96, 0.95, 0.911))
                .key("model.islandSmall")
                .centre(0.0, 0.0255)
                .paint(&[("cab", "Cabinet"), ("worktop", "Worktop")]),
            Model::new("islandLong", islands::island_long, (2.96, 0.95, 0.911))
                .key("model.islandLong")
                .centre(0.0, 0.0255)
                .paint(&[("cab", "Cabinet"), ("worktop", "Worktop")]),
            Model::new("barSegment", islands::bar_segment, (1.0, 1.06, 0.89))
                .key("model.barSegment")
                .centre(0.0, 0.015)
                .paint(&[("walnut", "Cabinet"), ("oak", "Worktop")]),
            Model::new("barEnd", islands::bar_end, (1.0, 1.06, 0.89))
                .key("model.barEnd")
                .centre(0.0, 0.015)
                .paint(&[("walnut", "Cabinet"), ("oak", "Worktop")]),
        ],
        "The kitchen base counters (crates/art/wet/src/kitchen_counters, built by cargo art). Listed \
         in the base manifest's packs. Each counter is a plinth, a carcass and a worktop (to 0.95): \
         the plain, corner and marble counters carry a drawer over two doors and slide it out 25 cm \
         (`drawer_open`) while a resident makes a sandwich, as does the butcher block's knife drawer. \
         The tall and wall storage (K5) adds a larder whose doors swing open for a snack; the islands \
         and bars (K6) add freestanding islands with their cupboards at the front and a worktop \
         overhanging the back for stools, and slatted bars with a top at 1.06, the kitchen island \
         sliding a drawer out between its stools for a sandwich.",
    ).switched())
}
