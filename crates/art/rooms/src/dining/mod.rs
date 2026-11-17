//! The kitchen's dining things. Notes: tools/art/blender/families/dining.md.
//!
//! Each is built in place where the game places it (`F`, game axes): table tops at the content's `surface`
//! heights (`TT` 0.75), seats where the game seats residents.
//!
//! The detail is built from the forms: tops and seat boards lofted with a moulded edge (`slab`), turned legs
//! (`turned`), arched aprons extruded (`apron`).
//!
//! Being ported: the square table for two is built here; the family's other models are still
//! tools/art/blender/families/dining.py's (`cargo art dining` builds them all in Blender until that script goes;
//! `--out` and `--check` build what is here).

use artkit::family::{Family, Model};
use artkit::palette;
use artkit::place::GameFrame;

mod common;
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

const DINING_TOP: [(&str, &str); 2] = [("top_oak", "Top"), ("legs_oakd", "Legs")];

/// The family, as far as it is built here.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "dining",
        M.palette(),
        vec![
            // K7: dining tables
            Model::new("tableSquare2", tables::table_square2, (0.82, 0.75, 0.82))
                .key("model.tableSquare2")
                .paint(&DINING_TOP),
        ],
        "Blender-built dining tables, chairs, stools and storage \
         (tools/art/blender/families/dining.py). Listed in the base manifest's packs. The materials \
         map onto the game's finishes so a model costs one draw call per finish group. Table tops \
         stand at the content's surface heights and seats where the game seats residents; the stools' \
         seats swivel 20 degrees as a resident sits (`swivel`, carrying them), and the bar cart's \
         cocktail shaker is taken off it while a drink is mixed (`shake`).",
    ))
}
