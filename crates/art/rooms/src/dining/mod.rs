//! The kitchen's dining things. Notes: NOTES.md.
//!
//! Each is built in place where the game places it (`F`, game axes): table tops at the content's `surface`
//! heights (`TT` 0.75), seats where the game seats residents.
//!
//! The detail is built from the forms: tops and seat boards lofted with a moulded edge (`slab`), turned legs
//! (`turned`), arched aprons extruded (`apron`).

use artkit::family::{Family, Model};
use artkit::json;
use artkit::palette;
use artkit::place::GameFrame;

mod chairs;
mod chairs_more;
mod china;
mod common;
mod pedestals;
mod sets;
mod settings;
mod shells;
mod storage;
mod tables;

palette! { M:
    // table tops and legs (each its own material, so the player colours them apart)
    top_oak = ("#CDB08A", wood), // white oak tops; the dining sets' chair seats
    legs_oak = ("#CDB08A", wood),
    legs_oakd = ("#B08E66", wood), // darker oak legs and frames
    top_walnut = ("#7A5234", wood),
    legs_walnut = ("#7A5234", wood),
    top_banquet = ("#8A5A36", wood),
    legs_banquet = ("#6B4A33", wood),
    top_birch = ("#E3CFA8", wood),
    legs_birch = ("#E3CFA8", wood),
    top_farm = ("#B48A5E", wood), // the farmhouse table's planks
    legs_farm = ("#8E6A48", wood),
    groove = ("#7A5A3A", wood), // the joints between its planks
    top_white = ("#F1F0EC", gloss), // the round bistro table
    top_white_satin = ("#F1F0EC", satin), // the square table for eight
    top_black = ("#2B2C2E", satin), // the high café table
    legs_black = ("#2B2C2E", metal), // black metal frames
    legs_chrome = ("#D0D3D6", chrome),
    glass = ("#DCE6E8", glass), // the glass table's top
    // the sets' chairs and the table settings
    seat_sage = ("#9DAF8C", fabric), // seat pads
    chair_dark = ("#3A3C40", metal), // the glass set's chair frames
    seat_grey = ("#C9CBCC", satin), // and their seats
    runner = ("#E8DCC0", fabric), // a table runner
    porcelain = ("#F5F3EE", gloss), // plates, a cup
    food = ("#E7D9BF", gloss),
    tumbler = ("#DCE6EA", gloss), // drinking glasses (gloss, as the old parts')
    cutlery = ("#B9BDC1", satin),
    vase = ("#D8D2C4", gloss),
    stem = ("#7E9A62", satin),
    bloom_red = ("#D9483B", satin),
    bloom_yellow = ("#F2C14E", satin),
    bloom_white = ("#F0EDE6", satin),
    // chairs and stools
    frame_cream = ("#E4E0D6", satin), // the painted dining chair
    frame_honey = ("#A8754A", wood), // the wooden chair
    seat_honey = ("#A8754A", wood),
    upholstery = ("#B7A48A", fabric), // the upholstered chair
    shell = ("#E8E4DC", gloss), // the moulded chair
    frame_windsor = ("#6B4A33", wood),
    seat_windsor = ("#6B4A33", wood),
    frame_bent = ("#3B2A20", wood), // the bentwood chair
    seat_bent = ("#8E2A2A", fabric),
    frame_steel = ("#C3C7CB", metal), // the folding chair
    seat_teal = ("#4E7A7F", satin),
    cap = ("#2B2C2E", satin), // rubber feet
    frame_oak = ("#CDB08A", wood), // the carver chair, the bar stool
    seat_oak = ("#CDB08A", wood), // the bench's seat
    leather = ("#C08A52", leather), // the high bar stool's seat
    // storage
    body_walnut = ("#7A5234", wood), // the sideboard
    top_sideboard = ("#7A5234", wood),
    flute = ("#5E3F28", wood), // the fluting on its doors
    brass = ("#B8925A", metal),
    cream = ("#EFE5D3", gloss), // a vase
    body_sage = ("#9DAF8C", satin), // the buffet
    body_china = ("#F1EDE4", satin), // the china cabinet
    top_china = ("#F1EDE4", satin),
    back_blue = ("#9DB7C9", satin), // its back, behind the glass
    plinth = ("#B9A890", satin),
    plate_blue = ("#9DC6CF", gloss),
    // the bar cart
    frame_brass = ("#B8925A", metal),
    tray = ("#DCE6EA", glass),
    castor = ("#2B2C2E", satin),
    bottle_green = ("#2F5A3A", glass),
    bottle_red = ("#7A2E2E", glass),
    bottle_amber = ("#C9A64A", glass),
    shaker = ("#D0D3D6", chrome),
    basket = ("#C9A979", satin),
    lemon = ("#F2C14E", satin),
}

const F: GameFrame = GameFrame::new(0.0);
const TT: f64 = 0.75; // a dining table's top (content surface)
const BT: f64 = 1.06; // a bar table's top (the old `BT`)
const SZ: f64 = 0.05; // catalog2's chairs: the seat's middle sits 5 cm forward, towards the table

const DINING_TOP: [(&str, &str); 2] = [("top_oak", "Top"), ("legs_oakd", "Legs")];
const SEAT: [(&str, &str); 2] = [("frame_oak", "Frame"), ("seat_sage", "Seat")];

/// The family, as far as it is built here.
pub fn family() -> Option<Family> {
    let stool =
        json!({"uses": {"sit": "swivel", "drink": "swivel"}, "hold": 0.6, "ride": "swivel"});
    // putting clean dishes away (a `cupboard` station, cooking putAwayAt, anim fetch) opens a door
    let put_away = json!({"uses": {"fetch": "door_open"}, "hold": 1.5});
    Some(Family::new(
        "dining",
        M.palette(),
        vec![
            // K7: dining tables
            Model::new("diningTable", sets::dining_table, (1.674, 1.027, 0.75))
                .centre(0.0, 0.1)
                .key("model.diningTable")
                .paint(&DINING_TOP),
            Model::new("bistroSet", sets::bistro_set, (0.62, 1.004, 0.912))
                .key("model.bistroSet")
                .centre(0.0, 0.024)
                .paint(&[
                    ("top_oak", "Top"),
                    ("legs_black", "Frame"),
                    ("seat_sage", "Seats"),
                ]),
            Model::new("diningGlass", sets::dining_glass, (1.666, 1.073, 0.8))
                .centre(0.0, 0.1)
                .key("model.diningGlass")
                .paint(&[("glass", "Top"), ("legs_chrome", "Legs")]),
            Model::new("diningOak", sets::dining_oak, (1.674, 1.077, 0.8))
                .centre(0.0, 0.1)
                .key("model.diningOak")
                .paint(&DINING_TOP),
            Model::new("tableSquare2", tables::table_square2, (0.82, 0.75, 0.82))
                .key("model.tableSquare2")
                .paint(&DINING_TOP),
            Model::new(
                "tableBistroRound",
                pedestals::table_bistro_round,
                (0.72, 0.75, 0.72),
            )
            .key("model.tableBistroRound")
            .paint(&[("top_white", "Top"), ("legs_black", "Legs")]),
            Model::new("tableDining4", tables::table_dining4, (1.84, 0.75, 0.88))
                .key("model.tableDining4")
                .paint(&[("top_oak", "Top"), ("legs_oak", "Legs")]),
            Model::new("tableDining6", tables::table_dining6, (2.84, 0.75, 0.88))
                .key("model.tableDining6")
                .paint(&[("top_walnut", "Top"), ("legs_walnut", "Legs")]),
            Model::new("tableBanquet8", tables::table_banquet8, (3.84, 0.751, 0.88))
                .key("model.tableBanquet8")
                .paint(&[("top_banquet", "Top"), ("legs_banquet", "Legs")]),
            Model::new("tableRound4", pedestals::table_round4, (1.52, 0.75, 1.52))
                .key("model.tableRound4")
                .paint(&[("top_birch", "Top"), ("legs_birch", "Legs")]),
            Model::new("tableSquare8", tables::table_square8, (1.86, 0.75, 1.86))
                .key("model.tableSquare8")
                .paint(&[("top_white_satin", "Top"), ("legs_black", "Legs")]),
            Model::new(
                "tableFarmhouse6",
                tables::table_farmhouse6,
                (2.86, 0.751, 0.88),
            )
            .key("model.tableFarmhouse6")
            .paint(&[("top_farm", "Top"), ("legs_farm", "Legs")]),
            Model::new("tableBarHigh", pedestals::table_bar_high, (0.7, 1.06, 0.7))
                .key("model.tableBarHigh")
                .paint(&[("top_black", "Top"), ("legs_black", "Legs")]),
            // K8: chairs and stools
            Model::new("barStool", chairs::bar_stool, (0.39, 0.742, 0.39))
                .key("model.barStool")
                .motions(stool.clone())
                .paint(&SEAT),
            Model::new("stoolCounter", chairs::stool_counter, (0.39, 0.687, 0.39))
                .centre(0.0, 0.05)
                .key("model.stoolCounter")
                .motions(stool.clone())
                .paint(&SEAT),
            Model::new("stoolBarHigh", chairs::stool_bar_high, (0.39, 1.117, 0.398))
                .centre(0.0, 0.046)
                .key("model.stoolBarHigh")
                .motions(stool)
                .paint(&[("legs_black", "Frame"), ("leather", "Seat")]),
            Model::new("diningChair", chairs::dining_chair, (0.44, 1.002, 0.463))
                .centre(0.0, -0.002)
                .key("model.diningChair")
                .paint(&[("frame_cream", "Frame"), ("seat_sage", "Seat")]),
            Model::new("chairWood", chairs::chair_wood, (0.44, 0.902, 0.45))
                .centre(0.0, 0.035)
                .key("model.chairWood")
                .paint(&[("frame_honey", "Frame"), ("seat_honey", "Seat")]),
            Model::new(
                "chairUpholstered",
                chairs_more::chair_upholstered,
                (0.46, 0.992, 0.496),
            )
            .centre(0.0, 0.032)
            .key("model.chairUpholstered")
            .paint(&[("legs_walnut", "Frame"), ("upholstery", "Upholstery")]),
            Model::new("chairModern", shells::chair_modern, (0.44, 0.789, 0.496))
                .centre(0.0, 0.032)
                .key("model.chairModern")
                .paint(&[("legs_black", "Frame"), ("shell", "Seat")]),
            Model::new("chairWindsor", shells::chair_windsor, (0.475, 0.975, 0.474))
                .centre(0.0, 0.041)
                .key("model.chairWindsor")
                .paint(&[("frame_windsor", "Frame"), ("seat_windsor", "Seat")]),
            Model::new(
                "chairBentwood",
                chairs_more::chair_bentwood,
                (0.434, 0.968, 0.452),
            )
            .centre(0.0, 0.041)
            .key("model.chairBentwood")
            .paint(&[("frame_bent", "Frame"), ("seat_bent", "Seat")]),
            Model::new(
                "chairFolding",
                chairs_more::chair_folding,
                (0.46, 0.892, 0.48),
            )
            .centre(0.0, 0.06)
            .key("model.chairFolding")
            .paint(&[("frame_steel", "Frame"), ("seat_teal", "Seat")]),
            Model::new("chairArm", chairs_more::chair_arm, (0.57, 0.922, 0.47))
                .centre(0.0, 0.035)
                .key("model.chairArm")
                .paint(&SEAT),
            Model::new("benchDining", chairs_more::bench_dining, (1.6, 0.468, 0.36))
                .centre(0.0, 0.05)
                .key("model.benchDining")
                .paint(&[("legs_oakd", "Frame"), ("seat_oak", "Seat")]),
            // K9: dining storage
            Model::new("sideboard", storage::sideboard, (1.82, 1.1, 0.668))
                .centre(0.0, -0.096)
                .key("model.sideboard")
                .motions(put_away.clone())
                .paint(&[
                    ("body_walnut", "Body"),
                    ("top_sideboard", "Top"),
                    ("brass", "Handles"),
                ]),
            Model::new("buffet", storage::buffet, (1.88, 0.9, 0.668))
                .centre(0.0, -0.096)
                .key("model.buffet")
                .motions(put_away.clone())
                .paint(&[
                    ("body_sage", "Body"),
                    ("top_oak", "Top"),
                    ("brass", "Handles"),
                ]),
            Model::new("chinaCabinet", china::china_cabinet, (0.98, 2.1, 0.548))
                .centre(0.0, -0.156)
                .key("model.chinaCabinet")
                .motions(put_away)
                .paint(&[
                    ("body_china", "Body"),
                    ("top_china", "Top"),
                    ("brass", "Handles"),
                ]),
            Model::new("barCart", china::bar_cart, (0.769, 1.09, 0.45))
                .centre(-0.017, 0.0)
                .key("model.barCart")
                .motions(json!({"uses": {"mix": "shake"}}))
                .paint(&[("frame_brass", "Frame"), ("tray", "Trays")]),
        ],
        "The dining tables, chairs, stools and storage (crates/art/rooms/src/dining, built by cargo art). Listed in the \
         base manifest's packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group. Table tops stand at the content's surface heights and seats where the game seats residents; the \
         stools' seats swivel 20 degrees as a resident sits (`swivel`, carrying them), and the bar cart's cocktail \
         shaker is taken off it while a drink is mixed (`shake`).",
    ).switched())
}
