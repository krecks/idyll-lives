//! The living room: a fold-down wall bed (`model.wallBed`, content `wallBed` in catalog2/living.json), a sofa bed
//! (`model.sofaBed`, content `sofaBed`), the L1 sofas and sectionals, the L2 armchairs and small seats, the L3
//! coffee and side tables, the L5 shelves and the L6 staircases (`model.stairs`, content `stairs` in base.json).
//! Notes: NOTES.md.
//!
//! The wall bed: a cabinet at the back of a 2 × 3 footprint, with open shelves on both sides. The bed stands
//! folded up in it at rest, its underside dressed as two door panels with a handle. It turns down on a hinge at
//! the cabinet's foot to fill the footprint in front (`bed_down`) and stays down while the object is open:
//! residents fold it down ("Fold down", or going to bed) and up ("Fold up", or after a night's sleep; content
//! `when`/`leaves`). The game lays sleepers centred on the footprint, so the mattress is. Built in game
//! coordinates (`F`: x, height, depth; front +depth), the bed in its down pose, then turned up to rest.
//!
//! The sofa bed is a boxy two-seater against the wall (the old part's size), whose mattress, folded away inside
//! the base under the seat at rest, pulls out forward over the seat and onto the floor in front (`bed_out`), held
//! out while the object is open (dozing on it, "Pull out the bed"; "Fold the bed away"). Sleepers lie with their
//! heads on the seat and the rest on the mattress (furniture.json beds.sofaBed: its top and how far forward).
//!
//! The sofas and sectionals copy the old primitive parts, where they stand. Their seat cushions are a part of
//! their own that sinks 2 cm while sat or dozed on (`sink_in`). The armchairs too, but for the recliner (it
//! reclines), the rocking chair (it rocks) and the bean bag.

use artkit::family::{Family, Model};
use artkit::json;
use artkit::json::Json;
use artkit::palette;
use artkit::place::GameFrame;

mod armchairs;
mod beds;
mod bookcases;
mod common;
mod seats;
mod sectionals;
mod shelves;
mod side_tables;
mod sofas;
mod stairs;
mod tables;

palette! { M:
    body = ("#E9E4DA", satin), // the cabinet and the bed's underside; the main part the player colours
    inside = ("#D8D1C4", matte), // the cabinet's back and the shelves
    frame = ("#B9A58A", satin), // the bed frame around the mattress
    metal = ("#8D8F92", metal), // handle
    mattress = ("#F2EFEA", fabric),
    throw = ("#C98B6B", fabric), // folded at the foot; the game puts a blanket over each sleeper
    pillow = ("#F6F3EE", fabric),
    book_a = ("#9C4A3C", satin),
    book_b = ("#3F5E73", satin),
    book_c = ("#C9A65A", satin),
    book_d = ("#5D7A54", satin),
    // L6, the staircase: oak treads, white risers and strings, a black iron railing
    treads = ("#B98B5E", wood),
    risers = ("#EFEBE3", satin),
    railing = ("#3C3A38", metal),
    spine = ("#2B2D30", metal), // the spiral's newel: dark steel, not a colour part
    // the sofa bed
    upholstery = ("#6F7F8C", fabric), // its body: arms, back and base; the main part the player colours
    cushion = ("#7A8A97", fabric), // the seat and back cushions
    legs = ("#2B2C2E", metal),
    pillow_rose = ("#E4B8AE", fabric),
    pillow_cream = ("#EFE5D3", fabric),
    bedding = ("#F2EFEA", fabric), // the pulled-out mattress, made up
    // L1, the sofas and sectionals: per model its upholstery (the main part), its cushions, its legs and
    // any show-wood or metal frame, the colours of its old part
    linen = ("#DCD5CA", fabric), // the classic camel-back sofa
    linen_cushion = ("#DCD5CA", fabric),
    walnut = ("#6E4E36", wood), // its show-wood: the back rail, the apron, the scroll faces
    walnut_legs = ("#6E4E36", wood),
    pillow_rust = ("#B5653F", fabric),
    pillow_sage = ("#9DAF8C", fabric),
    sage = ("#9DAF8C", fabric), // the loveseat
    sage_cushion = ("#9DAF8C", fabric),
    oak_legs = ("#CDB08A", wood),
    stone = ("#C9C6BE", fabric), // the three-seat sofa
    stone_cushion = ("#C9C6BE", fabric),
    oak_dark_legs = ("#B08E66", wood),
    throw_sage = ("#9DAF8C", fabric),
    teal = ("#4E7A7F", fabric), // the designer sofa
    teal_cushion = ("#4E7A7F", fabric),
    brass = ("#B8925A", metal), // its front rail and arm caps
    brass_legs = ("#B8925A", metal),
    moss = ("#7A8C6E", fabric), // the three-seater
    moss_cushion = ("#7A8C6E", fabric),
    pillow_mustard = ("#D6A64A", fabric),
    greige = ("#A9A49A", fabric), // the sectional's modules and the chaise
    greige_cushion = ("#A9A49A", fabric),
    espresso_legs = ("#3B302A", wood),
    throw_oat = ("#D9CFC0", fabric),
    // L2, the armchairs and small seats
    forest = ("#2F5A4E", fabric), // the wingback
    forest_cushion = ("#2F5A4E", fabric),
    walnut_warm = ("#7A5234", wood), // the wingback's, the pouf's and the ottoman's legs
    tan = ("#7A5A44", fabric), // the recliner
    tan_cushion = ("#7A5A44", fabric),
    black_satin = ("#2B2C2E", satin), // the designer lounge chair's glides
    leather = ("#2B2826", leather), // the leather armchair's cushions
    teak = ("#9A6B45", wood), // and its open frame
    ply = ("#6B4630", wood), // the designer lounge chair's shells
    hide = ("#5A3A2A", leather), // its cushions
    alu = ("#C9CBCC", metal), // its swivel base
    rocker = ("#8A5A36", wood), // the rocking chair
    rocker_cushion = ("#C9B79C", fabric),
    pouf_leather = ("#3A2E2A", leather), // the leather pouf
    pouf_button = ("#2A201D", leather), // covered in the same hide (plain, they would cost the top a draw call)
    navy = ("#3E5470", fabric), // the tufted ottoman
    navy_button = ("#2E3F56", fabric),
    mustard = ("#D6A64A", fabric), // the floor cushion
    mustard_piping = ("#C9963D", fabric),
    mustard_button = ("#B5853A", fabric),
    bag = ("#C25B3E", fabric), // the bean bag
    bag_seat = ("#B5523A", fabric),
    // L3, the coffee and side tables (and what stands on them: plain, so it costs no draw call)
    oak = ("#CDB08A", wood),
    glass = ("#DCE6E8", glass),
    chrome = ("#D0D3D6", chrome),
    marble = ("#EEECE8", gloss, roughness(0.28)),
    marble_vein = ("#C9CBCC", gloss),
    cream_gloss = ("#E8E2D6", gloss),
    white_gloss = ("#F1F0EC", gloss),
    stone_top = ("#E7E3DC", gloss),
    black_metal = ("#2B2C2E", metal),
    basket = ("#C9A979", fabric),
    magazine_teal = ("#4E7A7F", matte),
    magazine_navy = ("#3E5470", matte),
    box_sand = ("#C9B79C", matte),
    vase = ("#E8E6E1", gloss),
    pot_white = ("#F2F2F0", gloss),
    plant = ("#5E8F4E", satin),
    // L5, the shelves: frames and backs (the colour parts), and what stands on them. The books are plain
    // satin, so a wall of books costs no draw call of its own; the plants on them are plain too.
    shelf_back = ("#8E6A48", wood), // the bookshelf's back and kick
    oak_back = ("#B99A72", wood), // the low and library bookcases' backs
    walnut_back = ("#5A3F2C", wood), // the tall bookcase's
    oak_shelf = ("#CDB08A", wood), // the ladder shelf's boards and the industrial shelving's
    shelf_white = ("#F1F0EC", satin), // the corner bookcase and the cube shelf
    shelf_teal = ("#4E7A7F", matte), // the corner bookcase's back
    cube_back = ("#D9D6CF", matte),
    bin_sage = ("#9DAF8C", fabric), // the cube shelf's fabric bins
    bin_clay = ("#C98F6B", fabric),
    bin_hole = ("#3A3530", matte),
    vitrine = ("#7A5234", wood), // the display cabinet
    vitrine_back = ("#E8E2D6", wood),
    shelf_glass = ("#CFDCDF", gloss), // its shelves: plain, so its door is the only glass
    book_e = ("#2B2B2E", satin),
    book_f = ("#D8D2C4", satin),
    book_g = ("#4A3B5A", satin),
    pages = ("#EFE8D8", satin),
    gilt = ("#D8B86A", satin), // the bands on the spines
    clay = ("#C98F6B", matte),
    navy_gloss = ("#3E5470", gloss),
    red_gloss = ("#C25B3E", gloss),
    blue_gloss = ("#9DC6CF", gloss),
    porcelain = ("#F5F3EE", gloss),
    photo = ("#F2F0EA", matte),
    leaf_light = ("#7FA35A", satin),
}

const F: GameFrame = GameFrame::new(0.0);

const SECTIONAL: [(&str, &str); 3] = [
    ("greige", "Upholstery"),
    ("greige_cushion", "Cushions"),
    ("espresso_legs", "Legs"),
];

fn sofa_uses() -> Json {
    json!({"uses": {"relax": "sink_in", "doze": "sink_in"}})
}

/// The family, as far as it is built here.
pub fn family() -> Family {
    let sit = json!({"uses": {"sit": "sink_in"}});
    let sit_read = json!({"uses": {"sit": "sink_in", "read": "sink_in"}});
    let book_out =
        json!({"uses": {"read": "book_out", "study": "book_out", "homework": "book_out"}});
    let read = json!({"uses": {"read": "book_out"}});
    Family::new(
        "living",
        M.palette(),
        vec![
            Model::new("wallBed", beds::wall_bed, (1.98, 2.22, 0.6))
                .key("model.wallBed")
                .centre(0.0, -1.135)
                // Down while the bed is open (the sim's `open`), from the moment a use that opens it
                // starts (`uses`, so it doesn't wait for the house to be redrawn).
                .motions(json!({"uses": {"sleep": "bed_down", "nap": "bed_down", "foldDown": "bed_down"}, "open": "bed_down"}))
                // The books keep their own colours.
                .paint(&[
                    ("body", "Cabinet"),
                    ("frame", "Bed frame"),
                    ("mattress", "Mattress"),
                    ("pillow", "Pillows"),
                    ("throw", "Throw"),
                    ("inside", "Inside & shelves"),
                    ("metal", "Handle"),
                ]),
            Model::new("sofaBed", beds::sofa_bed, (1.86, 0.83, 0.83))
                .key("model.sofaBed")
                .centre(0.0, -0.015)
                // Out while the sofa bed is open, from the moment a use that opens it starts.
                .motions(json!({"uses": {"doze": "bed_out", "sleep": "bed_out", "pullOut": "bed_out"}, "open": "bed_out"}))
                .paint(&[
                    ("upholstery", "Upholstery"),
                    ("cushion", "Cushions"),
                    ("legs", "Legs"),
                    ("bedding", "Bedding"),
                    ("pillow_rose", "Pillow"),
                    ("pillow_cream", "Second pillow"),
                ]),
            // L6: the staircase (placed by build mode's Stairs tool and the house generator)
            Model::new("stairs", stairs::stairs, (0.855, 3.68, 4.0))
                .key("model.stairs")
                .paint(&[("treads", "Treads"), ("railing", "Railing")]),
            Model::new("stairsOpen", stairs::stairs_open, (0.82, 3.73, 4.0))
                .key("model.stairsOpen")
                .paint(&[("treads", "Treads"), ("railing", "Railing")]),
            Model::new("stairsSpiral", stairs::stairs_spiral, (1.84, 3.78, 1.84))
                .key("model.stairsSpiral")
                .paint(&[("treads", "Treads"), ("railing", "Railing")]),
            // L1: the sofas and sectionals, their seat cushions sinking while sat or dozed on
            Model::new("sofa", sofas::sofa_classic, (1.88, 0.935, 0.817))
                .key("model.sofa")
                .centre(0.0, -0.0215)
                .motions(json!({"uses": {"relax": "sink_in"}}))
                .paint(&[
                    ("linen", "Upholstery"),
                    ("linen_cushion", "Cushions"),
                    ("walnut", "Frame"),
                    ("walnut_legs", "Legs"),
                    ("pillow_rust", "Pillow"),
                    ("pillow_sage", "Second pillow"),
                ]),
            Model::new("loveseat", sofas::loveseat, (1.84, 0.833, 0.84))
                .key("model.loveseat")
                .centre(0.0, -0.01)
                .motions(sofa_uses())
                .paint(&[("sage", "Upholstery"), ("sage_cushion", "Cushions"), ("oak_legs", "Legs"), ("pillow_cream", "Pillow")]),
            Model::new("sofaThree", sofas::sofa_three, (2.82, 0.87, 0.86))
                .key("model.sofaThree")
                .centre(0.0, 0.0)
                .motions(sofa_uses())
                .paint(&[
                    ("stone", "Upholstery"),
                    ("stone_cushion", "Cushions"),
                    ("oak_dark_legs", "Legs"),
                    ("pillow_sage", "Pillows"),
                    ("throw_sage", "Throw"),
                ]),
            Model::new("sofaDesign", sectionals::sofa_design, (1.87, 0.72, 0.82))
                .key("model.sofaDesign")
                .centre(0.0, -0.02)
                .motions(sofa_uses())
                .paint(&[("teal", "Upholstery"), ("teal_cushion", "Cushions"), ("brass", "Frame"), ("brass_legs", "Legs")]),
            Model::new("sofaThreeSeat", sectionals::sofa_three_seat, (2.86, 0.823, 0.88))
                .key("model.sofaThreeSeat")
                .centre(0.0, 0.01)
                .motions(sofa_uses())
                .paint(&[
                    ("moss", "Upholstery"),
                    ("moss_cushion", "Cushions"),
                    ("oak_legs", "Legs"),
                    ("pillow_cream", "Pillow"),
                    ("pillow_mustard", "Second pillow"),
                ]),
            Model::new("sectionalMiddle", sectionals::sectional_middle, (1.0, 0.823, 0.88))
                .key("model.sectionalMiddle")
                .centre(0.0, 0.01)
                .motions(sofa_uses())
                .paint(&SECTIONAL),
            Model::new("sectionalArmLeft", sectionals::sectional_arm_left, (1.0, 0.823, 0.88))
                .key("model.sectionalArmLeft")
                .centre(0.0, 0.01)
                .motions(sofa_uses())
                .paint(&SECTIONAL),
            Model::new("sectionalArmRight", sectionals::sectional_arm_right, (1.0, 0.823, 0.88))
                .key("model.sectionalArmRight")
                .centre(0.0, 0.01)
                .motions(sofa_uses())
                .paint(&SECTIONAL),
            Model::new("sectionalCorner", sectionals::sectional_corner, (1.0, 0.823, 0.89))
                .key("model.sectionalCorner")
                .centre(0.0, 0.015)
                .motions(sofa_uses())
                .paint(&[SECTIONAL[0], SECTIONAL[1], SECTIONAL[2], ("pillow_mustard", "Pillow")]),
            Model::new("chaise", sectionals::chaise, (0.96, 0.823, 1.9))
                .key("model.chaise")
                .centre(0.0, 0.02)
                .motions(sofa_uses())
                .paint(&[SECTIONAL[0], SECTIONAL[1], SECTIONAL[2], ("throw_oat", "Throw")]),
            // L2: the armchairs and small seats
            Model::new("armchair", armchairs::armchair_bergere, (0.74, 0.982, 0.797))
                .key("model.armchair")
                .centre(0.0, -0.0315)
                .motions(sit.clone())
                .paint(&[
                    ("linen", "Upholstery"),
                    ("linen_cushion", "Cushion"),
                    ("walnut", "Frame"),
                    ("walnut_legs", "Legs"),
                    ("pillow_sage", "Pillow"),
                ]),
            Model::new("armchairLounge", armchairs::armchair_lounge, (0.82, 0.83, 0.84))
                .key("model.armchairLounge")
                .centre(0.0, -0.01)
                .motions(sit.clone())
                .paint(&[("sage", "Upholstery"), ("sage_cushion", "Cushions"), ("oak_legs", "Legs")]),
            Model::new("armchairLeather", armchairs::armchair_leather, (0.73, 0.949, 0.852))
                .key("model.armchairLeather")
                .centre(0.0, -0.004)
                .motions(sit_read.clone())
                .paint(&[("teak", "Frame"), ("leather", "Upholstery")]),
            Model::new("armchairDesign", armchairs::armchair_design, (0.76, 0.959, 0.852))
                .key("model.armchairDesign")
                .centre(0.0, -0.004)
                .paint(&[("ply", "Frame"), ("hide", "Upholstery"), ("alu", "Base")]),
            Model::new("armchairWingback", armchairs::armchair_wingback, (0.84, 1.1, 0.8))
                .key("model.armchairWingback")
                .centre(0.0, -0.03)
                .motions(sit_read)
                .paint(&[("forest", "Upholstery"), ("forest_cushion", "Cushion"), ("walnut_warm", "Legs")]),
            Model::new("recliner", seats::recliner, (0.92, 0.99, 0.80))
                .key("model.recliner")
                .centre(0.0, 0.08)
                .motions(json!({"uses": {"relax": ["recline", "footrest_up"], "doze": ["recline", "footrest_up"]}, "ride": "recline"}))
                .paint(&[("tan", "Upholstery"), ("tan_cushion", "Cushions & footrest")]),
            Model::new("rockingChair", seats::rocking_chair, (0.525, 1.033, 0.9))
                .key("model.rockingChair")
                .centre(0.0, 0.05)
                .motions(json!({"loops": {"relax": "rock", "read": "rock"}, "ride": "rock"}))
                .paint(&[("rocker", "Wood"), ("rocker_cushion", "Cushion")]),
            Model::new("pouf", seats::pouf, (0.72, 0.47, 0.5))
                .key("model.pouf")
                .motions(sit.clone())
                .paint(&[("pouf_leather", "Leather"), ("pouf_button", "Buttons"), ("walnut_warm", "Legs")]),
            Model::new("ottoman", seats::ottoman, (0.62, 0.45, 0.62))
                .key("model.ottoman")
                .motions(sit.clone())
                .paint(&[("navy", "Upholstery"), ("navy_button", "Buttons"), ("walnut_warm", "Feet")]),
            Model::new("floorCushion", seats::floor_cushion, (0.62, 0.139, 0.62))
                .key("model.floorCushion")
                .motions(sit)
                .paint(&[("mustard", "Cushion"), ("mustard_piping", "Piping"), ("mustard_button", "Button")]),
            Model::new("beanBag", seats::bean_bag, (0.84, 0.696, 0.844))
                .key("model.beanBag")
                .centre(0.0, 0.018)
                .motions(json!({"uses": {"flop": "squash"}}))
                .paint(&[("bag", "Bag"), ("bag_seat", "Seat")]),
            // L3: the coffee and side tables
            Model::new("coffeeTable", tables::coffee_table, (1.3, 0.42, 0.74))
                .key("model.coffeeTable")
                .motions(json!({"uses": {"magazine": "drawer_open"}, "hold": 2.5}))
                .paint(&[("oak", "Wood"), ("brass", "Knob")]),
            Model::new("coffeeTableGlass", tables::coffee_table_glass, (1.3, 0.6, 0.74))
                .key("model.coffeeTableGlass")
                .paint(&[("chrome", "Frame"), ("glass", "Glass")]),
            Model::new("coffeeTableMarble", tables::coffee_table_marble, (0.88, 0.58, 0.88))
                .key("model.coffeeTableMarble")
                .paint(&[("marble", "Top"), ("brass", "Base")]),
            Model::new("sideTable", tables::side_table, (0.55, 0.55, 0.45))
                .key("model.sideTable")
                .paint(&[("oak", "Wood"), ("brass", "Knob")]),
            Model::new("coffeeTableRound", side_tables::coffee_table_round, (0.84, 0.42, 0.84))
                .key("model.coffeeTableRound")
                .paint(&[("cream_gloss", "Top"), ("oak", "Base")]),
            Model::new("coffeeTableLong", side_tables::coffee_table_long, (1.66, 0.42, 0.72))
                .key("model.coffeeTableLong")
                .paint(&[("walnut_warm", "Top"), ("black_metal", "Legs")]),
            Model::new("nestingTables", side_tables::nesting_tables, (0.57, 0.512, 0.55))
                .key("model.nestingTables")
                .centre(0.005, 0.025)
                .motions(json!({"uses": {"pullOut": "small_out", "pushIn": {"clip": "small_out", "over": (-1)}}, "open": "small_out"}))
                .paint(&[("white_gloss", "Top"), ("oak", "Small table"), ("black_metal", "Legs")]),
            Model::new("sideTableRound", side_tables::side_table_round, (0.52, 0.56, 0.52))
                .key("model.sideTableRound")
                .paint(&[("stone_top", "Top"), ("brass", "Base")]),
            Model::new("consoleTable", side_tables::console_table, (1.8, 0.7975, 0.56))
                .key("model.consoleTable")
                .centre(0.0, -0.15)
                .paint(&[("oak", "Top"), ("black_metal", "Legs"), ("basket", "Baskets")]),
            // L5: the shelves; on the bookcases one book tips out while read at
            Model::new("bookshelf", shelves::bookshelf, (0.9, 1.9175, 0.375))
                .key("model.bookshelf")
                .centre(0.0, -0.2425)
                .motions(book_out.clone())
                .paint(&[("oak", "Frame"), ("shelf_back", "Back")]),
            Model::new("bookcaseLow", shelves::bookcase_low, (0.92, 1.165, 0.38))
                .key("model.bookcaseLow")
                .centre(0.0, -0.24)
                .motions(read.clone())
                .paint(&[("oak", "Frame"), ("oak_back", "Back")]),
            Model::new("bookcaseWide", shelves::bookcase_wide, (1.86, 2.0, 0.47))
                .key("model.bookcaseWide")
                .centre(0.0, -0.195)
                .motions(book_out)
                .paint(&[("oak", "Frame"), ("oak_back", "Back")]),
            Model::new("shelvingIndustrial", bookcases::shelving_industrial, (1.86, 1.83, 0.53))
                .key("model.shelvingIndustrial")
                .centre(0.0, -0.165)
                .paint(&[("black_metal", "Frame"), ("oak_shelf", "Shelves")]),
            Model::new("bookcaseTall", bookcases::bookcase_tall, (0.9, 2.0, 0.34))
                .key("model.bookcaseTall")
                .centre(0.0, -0.26)
                .motions(json!({"uses": {"read": "book_out", "study": "book_out"}}))
                .paint(&[("walnut_warm", "Frame"), ("walnut_back", "Back")]),
            Model::new("bookcaseCorner", bookcases::bookcase_corner, (0.9, 1.9, 0.34))
                .key("model.bookcaseCorner")
                .centre(0.03, -0.26)
                .motions(read)
                .paint(&[("shelf_white", "Frame"), ("shelf_teal", "Back")]),
            Model::new("cubeShelf", bookcases::cube_shelf, (0.8, 0.8, 0.36))
                .key("model.cubeShelf")
                .centre(0.0, -0.25)
                .paint(&[("shelf_white", "Frame"), ("cube_back", "Back")]),
            Model::new("ladderShelf", bookcases::ladder_shelf, (0.635, 1.8, 0.356))
                .key("model.ladderShelf")
                .centre(0.0, -0.252)
                .paint(&[("oak", "Frame"), ("oak_shelf", "Shelves")]),
            Model::new("displayCabinet", bookcases::display_cabinet, (0.86, 1.91, 0.45))
                .key("model.displayCabinet")
                .centre(0.0, -0.205)
                .motions(json!({"uses": {"openDoor": "door_open", "shutDoor": {"clip": "door_open", "over": (-1)}}, "open": "door_open"}))
                .paint(&[("vitrine", "Frame"), ("vitrine_back", "Back")]),
        ],
        "The living room (crates/art/rooms/src/living, built by cargo art). The materials map onto the game's \
         finishes so a model costs one draw call per finish group.",
    )
}
