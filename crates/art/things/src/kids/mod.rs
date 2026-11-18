//! The kids' things. Notes: NOTES.md.
//!
//! KD1 baby and kids' beds: the crib with its changing table (`model.crib`), the kids' bed (`model.kidsBed`), the race
//!   car bed (`model.kidsBedCar`) and the high chair (`model.highChair`).
//! KD2 toys: the toy box and toy chest, the doll house, the rocking horse, the play tent, the ball pit, the train
//!   table, the play mat, the giant teddy bear and the kids' bean bag.
//! KD3 kids' study and art: the kids' desk, table and chair, the picture-book shelf, the chalkboard easel and the
//!   kids' easel.
//!
//! Each is built where the game places it (`F`, game axes: x, height, depth; front +depth), then stood on the floor
//! (`common::place`, `centre` on its `Model`), so seats, mattresses and wall backs stay where the game puts
//! residents. Seat and mattress heights are the renderer's (characters.ts `SEATS`, `BEDS`; the content's `surface`
//! and `seat`).
//!
//! Built with the forms: mattresses, pillows, duvets and cushions are soft boxes, frames and rails one swept piece,
//! shaped boards (the car's sides, arched headboards and ends, the doll house's windows, the toy box's lid) extruded
//! outlines with their cut-outs, turned parts revolved, round bodies (the horse, the teddy, the bean bag, the tent's
//! canvas) lofted.

use artkit::family::{Family, Model};
use artkit::json;
use artkit::palette;
use artkit::place::GameFrame;

mod beds;
mod boxes;
mod chair;
mod common;
mod crib;
mod desk;
mod doll;
mod easels;
mod horse;
mod mat;
mod pit;
mod plush;
mod shelf;
mod tent;
mod train;

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

/// The family.
pub fn family() -> Option<Family> {
    let turn_down = || json!({"uses": {"sleep": "turn_down", "nap": "turn_down"}});
    // open while the child plays (the toys are out), shut when they're done
    let lid = || json!({"uses": {"play": "lid_open"}});
    Some(Family::new(
        "kids",
        M.palette(),
        vec![
            // KD1
            Model::new("crib", crib::crib, (1.935, 1.008, 0.69))
                .key("model.crib")
                .centre(-0.0125, -0.095)
                // the mobile turns for an awake baby, while someone plays with them; not all night
                .motions(json!({"uses": {"feed": "side_down", "change": "side_down", "cuddle": "side_down"},
                                "loops": {"play": "mobile"}}))
                .paint(&[("crib", "Frame"), ("crib_bedding", "Bedding"), ("mobile", "Mobile")]),
            Model::new("kidsBed", beds::kids_bed, (0.97, 0.764, 1.88))
                .key("model.kidsBed")
                .centre(0.005, 0.01)
                .motions(turn_down())
                .paint(&[("kids_frame", "Frame"), ("kids_bedding", "Bedding")]),
            Model::new("kidsBedCar", beds::kids_bed_car, (0.985, 0.775, 1.916))
                .key("model.kidsBedCar")
                .centre(0.0, 0.028)
                .motions(turn_down())
                .paint(&[("car", "Car body"), ("car_bedding", "Bedding")]),
            Model::new("highChair", chair::high_chair, (0.533, 0.945, 0.554))
                .key("model.highChair")
                .centre(0.0, 0.06)
                .motions(json!({"uses": {"sit": {"clip": "tray_out", "hold": 1.0, "out": 1.2}}}))
                .paint(&[("chair_white", "Frame"), ("chair_cushion", "Bedding")]),
            // KD2
            // its own size: the blocks left out in front of it are drawn with it, not measured (the box is 0.5 m deep)
            Model::new("toyBox", boxes::toy_box, (0.82, 0.51, 0.518))
                .key("model.toyBox")
                .centre(0.0, -0.0389)
                .motions(lid())
                .paint(&[("toybox", "Body"), ("toybox_lid", "Lid")]),
            Model::new("toyChest", boxes::toy_chest, (0.846, 0.534, 0.513))
                .key("model.toyChest")
                .centre(0.0, -0.1735)
                .motions(lid())
                .paint(&[("chest", "Body"), ("chest_trim", "Trim")]),
            Model::new("dollHouse", doll::doll_house, (0.84, 0.908, 0.456))
                .key("model.dollHouse")
                .centre(0.0, -0.202)
                .motions(json!({"uses": {"dolls": "front_open"}}))
                .paint(&[("house", "Walls"), ("roof", "Roof")]),
            Model::new("rockingHorse", horse::rocking_horse, (0.289, 0.825, 0.877))
                .key("model.rockingHorse")
                .centre(0.0, 0.0426)
                .motions(json!({"loops": {"ride": "rock"}, "ride": "rock"}))
                .paint(&[("horse", "Body"), ("saddle", "Trim")]),
            Model::new("playTent", tent::play_tent, (0.952, 1.574, 0.827))
                .key("model.playTent")
                .centre(0.0, 0.0)
                .motions(json!({"uses": {"play": "flap_open"}}))
                .paint(&[("canvas", "Canvas"), ("poles", "Poles")]),
            Model::new("ballPit", pit::ball_pit, (0.93, 0.47, 0.93))
                .key("model.ballPit")
                .centre(0.0, 0.035)
                .motions(json!({"loops": {"jump": "bounce"}}))
                .paint(&[("pit", "Pit"), ("pit_rim", "Rim")]),
            Model::new("trainTable", train::train_table, (1.24, 0.68, 0.84))
                .key("model.trainTable")
                .centre(0.0, 0.0)
                .motions(json!({"loops": {"trains": "train"}}))
                .paint(&[("train_table", "Table"), ("train", "Train")]),
            Model::new("playMat", mat::play_mat, (0.92, 0.204, 0.92))
                .key("model.playMat")
                .centre(0.0, 0.03)
                .motions(json!({"uses": {"blocks": "blocks"}}))
                .paint(&[("mat", "Mat")]),
            Model::new("teddyBear", plush::teddy_bear, (0.635, 0.912, 0.575))
                .key("model.teddyBear")
                .centre(0.0, 0.0575)
                .motions(json!({"uses": {"hug": "squish"}}))
                .paint(&[("fur", "Fur"), ("bow", "Bow")]),
            Model::new("beanBagKids", plush::bean_bag_kids, (0.62, 0.528, 0.645))
                .key("model.beanBagKids")
                .centre(0.0, 0.0187)
                .motions(json!({"uses": {"flop": "squash"}}))
                .paint(&[("bag", "Bag")]),
            // KD3
            Model::new("kidsDesk", desk::kids_desk, (0.76, 0.691, 0.82))
                .key("model.kidsDesk")
                .centre(0.0, 0.01)
                .motions(json!({"uses": {"homework": "drawer_open", "drawing": "drawer_open"}, "hold": 2}))
                .paint(&[("kdesk", "Frame"), ("kdesk_top", "Top"), ("kdesk_chair", "Chair")]),
            Model::new("kidsTable", desk::kids_table, (0.64, 0.5, 0.64))
                .key("model.kidsTable")
                .centre(0.0, 0.0)
                .paint(&[("ktable", "Frame"), ("ktable_top", "Top")]),
            Model::new("kidsChair", desk::kids_chair, (0.32, 0.57, 0.323))
                .key("model.kidsChair")
                .centre(0.0, 0.0386)
                .paint(&[("kchair", "Frame"), ("kchair_seat", "Seat")]),
            Model::new("kidsBookshelf", shelf::kids_bookshelf, (0.77, 0.925, 0.4))
                .key("model.kidsBookshelf")
                .centre(0.0, -0.23)
                .motions(json!({"uses": {"read": "book_out"}}))
                .paint(&[("kshelf", "Frame"), ("kshelf_top", "Top")]),
            Model::new("kidsChalkboard", easels::kids_chalkboard, (0.66, 0.956, 0.528))
                .key("model.kidsChalkboard")
                .centre(0.0, 0.0259)
                .motions(json!({"uses": {"draw": "drawing"}}))
                .paint(&[("chalk_frame", "Frame"), ("slate", "Board")]),
            Model::new("kidsEasel", easels::kids_easel, (0.66, 1.208, 0.634))
                .key("model.kidsEasel")
                .centre(0.0, 0.0)
                .motions(json!({"uses": {"paint": "painting"}}))
                .paint(&[("easel", "Easel")]),
        ],
        "The kids' things (crates/art/things/src/kids, built by cargo art): the crib, the beds and the \
         high chair, the toys, and the kids' study and art things. Listed in the base manifest's \
         packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group.",
    ).switched())
}
