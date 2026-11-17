//! The kids' things. Notes: tools/art/blender/families/kids.md.
//!
//! Being ported: the kids' chair is built here; the family's other models are still
//! tools/art/blender/families/kids.py's (`cargo art kids` builds them all in Blender until that script goes;
//! `--out` and `--check` build what is here).

use artkit::family::{Family, Model};
use artkit::palette;
use artkit::place::GameFrame;

mod common;
mod study;

palette! { M:
    // toys' colours, shared
    toy_red = ("#D9483B", satin),
    toy_blue = ("#3E8EDE", satin),
    toy_yellow = ("#F2C14E", satin),
    toy_green = ("#5BAE6A", satin),
    toy_pink = ("#E07BC3", satin),
    ball_red = ("#D9483B", gloss),
    ball_blue = ("#3E8EDE", gloss),
    ball_yellow = ("#F2C14E", gloss),
    ball_green = ("#5BAE6A", gloss),
    ball_pink = ("#E07BC3", gloss),
    dark = ("#2B2C2E", satin), // tyres, eyes' backs, a spoiler
    eye = ("#2A2624", gloss),
    paper = ("#F7F6F1", matte),
    // the crib and its changing table
    crib = ("#F2F0EA", satin), // the frame (Frame)
    crib_bedding = ("#CFE0EA", matte), // the mattress cover and the changing pad (Bedding)
    crib_blanket = ("#FBFBF8", matte), // the folded blanket at the foot
    crib_wood = ("#CDB08A", satin), // the changing table's top and drawer fronts
    mobile = ("#F2C14E", satin), // the mobile's arms and stars (Mobile)
    mobile_pink = ("#E4B8AE", satin),
    mobile_blue = ("#9DC6CF", satin),
    // the kids' bed
    kids_frame = ("#9DC6CF", satin),
    kids_bedding = ("#F2C14E", fabric),
    kids_drawer = ("#F4F1EA", satin), // the drawers under the bed
    mattress = ("#F7F7F5", fabric),
    sheet = ("#FFFFFF", fabric), // the sheet turned down over a duvet's top edge, pillows
    teddy_small = ("#C08A52", fabric),
    pennant_a = ("#D6A64A", satin),
    pennant_b = ("#4E7A7F", satin),
    pennant_c = ("#B5653F", satin),
    // the race car bed
    car = ("#D9483B", gloss),
    car_bedding = ("#3E8EDE", fabric),
    car_stripe = ("#F7F6F1", gloss),
    hub = ("#D0D3D6", satin),
    headlight = ("#F2C14E", gloss),
    // the high chair
    chair_white = ("#F2F2F0", satin),
    chair_legs = ("#E3CFA8", satin),
    chair_cushion = ("#9DC6CF", fabric),
    tray_mat = ("#9DC6CF", satin), // the high chair's wipe-clean mat: plain, so its sliding tray is one draw
    // the toy box and the toy chest
    toybox = ("#F2C14E", satin),
    toybox_lid = ("#E9A23B", satin),
    toybox_inside = ("#F4E2B0", satin),
    chest = ("#E9C46A", satin),
    chest_trim = ("#D9483B", satin),
    chest_inside = ("#C9A66A", satin),
    plush = ("#C08A52", satin),
    // the doll house
    house = ("#F4EBD8", satin), // its walls (Walls)
    roof = ("#D9483B", satin), // (Roof)
    house_trim = ("#FFFFFF", satin), // window frames, the gable's edge
    house_window = ("#BFE0E6", satin),
    house_door = ("#6E4A33", satin),
    house_floor = ("#A8754A", satin),
    wallpaper = ("#F2D9D5", satin),
    wallpaper_up = ("#DCEBD9", satin),
    // the rocking horse
    horse = ("#F2F0EA", satin), // (Body)
    saddle = ("#D9483B", fabric), // the saddle and the bridle (Trim)
    mane = ("#C08A52", fabric),
    rocker = ("#C9A27A", wood),
    spot = ("#C9C4BA", satin),
    handle = ("#2A2C30", metal),
    // the play tent
    canvas = ("#F2EBDD", fabric), // (Canvas)
    poles = ("#C9A27A", satin), // (Poles)
    flap = ("#E4B8AE", fabric),
    tent_door = ("#3B302A", matte),
    tent_mat = ("#9DC6CF", fabric),
    // the ball pit
    pit = ("#9DC6CF", satin),
    pit_rim = ("#F4F1EA", satin),
    pit_fill = ("#7FA9C9", satin),
    // the train table
    train_table = ("#F4F1EA", satin), // (Table)
    grass = ("#6FA36A", satin),
    track = ("#C9A27A", wood),
    track_groove = ("#8E6E52", satin),
    train = ("#D9483B", gloss), // the engine (Train)
    water = ("#9DC6CF", gloss),
    tree = ("#3F7A4A", satin),
    trunk = ("#8E6E52", satin),
    // the play mat
    mat = ("#9DC6CF", fabric),
    mat_yellow = ("#F2C14E", fabric),
    mat_pink = ("#E4B8AE", fabric),
    mat_green = ("#B5D99C", fabric),
    mat_white = ("#F2F2F0", fabric),
    // the teddy bear
    fur = ("#C08A52", fabric),
    fur_light = ("#E9D5B5", fabric),
    bow = ("#D9483B", satin),
    // the bean bag
    bag = ("#5BB3C9", fabric),
    bag_patch = ("#F2C14E", fabric),
    // the kids' desk, table and chair
    kdesk = ("#F2F2F0", satin), // legs, apron and drawer (Frame)
    kdesk_top = ("#E3CFA8", wood), // (Top)
    kdesk_chair = ("#9DC6CF", satin), // (Chair)
    ktable = ("#3E8EDE", gloss), // legs and apron (Frame)
    ktable_top = ("#F4F1EA", satin), // (Top)
    kchair = ("#3E8EDE", gloss), // (Frame)
    kchair_seat = ("#F4F1EA", satin), // (Top)
    // the picture-book shelf
    kshelf = ("#F4F1EA", satin), // (Frame)
    kshelf_top = ("#9DC6CF", satin), // the top board (Top)
    book_teal = ("#4E7A7F", matte),
    book_rust = ("#B5653F", matte),
    book_blue = ("#3E8EDE", matte),
    book_ochre = ("#D6A64A", matte),
    book_sage = ("#9DAF8C", matte),
    book_rose = ("#E4B8AE", matte),
    // the chalkboard easel and the kids' easel
    chalk_frame = ("#B5784A", wood), // (Frame)
    slate = ("#2E3532", matte), // (Board)
    chalk_white = ("#F2F2EE", matte),
    chalk_yellow = ("#F2D16B", matte),
    chalk_pink = ("#E9A0A8", matte),
    chalk_blue = ("#8EC5E8", matte),
    chalk_green = ("#9DD18B", matte),
    easel = ("#E3CFA8", wood), // (Easel)
    paint_red = ("#D9483B", matte),
    paint_blue = ("#3E8EDE", matte),
    paint_green = ("#5BAE6A", matte),
    paint_yellow = ("#F2C14E", matte),
}

const F: GameFrame = GameFrame::new(0.0);

/// The family, as far as it is built here.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "kids",
        M.palette(),
        vec![
            // KD3
            Model::new("kidsChair", study::kids_chair, (0.32, 0.57, 0.323))
                .key("model.kidsChair")
                .centre(0.0, 0.0386)
                .paint(&[("kchair", "Frame"), ("kchair_seat", "Seat")]),
        ],
        "Blender-built kids' things (tools/art/blender/families/kids.py): the crib, the beds and the \
         high chair, the toys, and the kids' study and art things. Listed in the base manifest's \
         packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group.",
    ))
}
