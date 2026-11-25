//! The tabletop things: small things that stand on tops (the `surface` layer). Notes: NOTES.md.
//!
//!   TT1 kitchenware: kettle, toaster, blender, standMixer, espressoMachineSmall, knifeBlock, utensilCrock,
//!       cuttingBoard, breadBin, spiceJars, cookieJar
//!   TT2 tableware: fruitBowl, plateSet, cakeStand, teapotSet, mugs, wineGlasses, candlesticks
//!   TT3 books and small plants: bookStack, booksUpright, vaseFlowers, vaseSmall, potSucculent, potCactus,
//!       potSmall
//!   TT4 small ornaments: photoFrame, tableClock, globe, snowGlobe, trophy, sculptureSmall
//!   TT5 bath, desk and toys: soapSet, towelStack, perfumeTray, deskOrganiser, laptopClosed, monitor,
//!       stuffedToy, toyBlocks
//!
//! Each stands on its base, centred (the renderer lifts it to its host's top), at the old part's size. Most
//! have no clips, and every finish is a plain one (wood, fabric and leaves drawn satin or matte),
//! so each costs one draw call. The exceptions: the cookie jar, the wine glasses and the snow globe are
//! see-through `glass` (two); the kettle's lid, the bread bin's roll-top and the laptop's lid are parts of
//! their own, hinged where they open, and open for their uses (two: `lid_open` for tea, `roll_open` for a
//! sandwich, the laptop's `lid_open` while it is used); the candlesticks' flames rise out of the candles
//! at night and glow (`motions.night`; two); the alarm clock's hour and minute hands tell the game's time
//! (`motions.clock`; three).
//!
//! They are placed many times on every lot, so each keeps to half its size class's target (objects.md): 750
//! triangles under 0.35 m, 2,000 for the five larger ones. Turned things are lofted rings (`turned`: few sides,
//! rims kept sharp), handles and spouts swept, slabs extruded with their holes, the plants' leaves and flowers
//! the botany's (`leaf`, `rays`, `tulip_head`).

use artkit::family::{Family, Model, motions};
use artkit::palette;

#[macro_use]
pub(crate) mod common;
mod desk;
mod kitchenware;
mod ornaments;
mod plants;
mod tableware;

palette! { M:
    // shared details
    dark = ("#2B2C2E", satin),
    charcoal = ("#3A3D41", satin),
    steel = ("#C3C7CB", metal),
    chrome = ("#D0D3D6", chrome),
    brass = ("#C9A54A", metal),
    wood = ("#9C6B43", satin),         // plain, not `wood`: a textured finish would be a second draw call
    wood_dark = ("#7A5234", satin),
    porcelain = ("#F5F3EE", gloss),
    paper = ("#F7F7F5", matte),
    leaf = ("#5E8F4E", satin),         // plain, not `foliage`, for the same reason
    leaf_light = ("#7FA35A", satin),
    soil = ("#4A3A2C", matte),
    glass = ("#DCE6EA", glass),        // see-through: its own draw call (cookie jar, wine glasses, snow globe)
    glass_pale = ("#DCE6EA", gloss),   // glass drawn opaque, where seeing through it doesn't matter
    red = ("#D9483B", gloss),
    orange = ("#F29B38", satin),
    yellow = ("#F2C14E", satin),
    apple_green = ("#9BC24A", satin),
    stalk = ("#5A4632", matte),
    white = ("#F2F0EA", satin),
    blush = ("#E4B8AE", satin),
    icecream = ("#8FD1A6", satin),     // churned mint: green, so it isn't the blender's pink smoothie
    rose = ("#E07B7B", satin),
    pink = ("#E07BC3", satin),
    bread = ("#D9A55B", matte),
    crumb = ("#F0DDB5", matte),
    coffee = ("#5A3A26", gloss),
    wine = ("#6E1E2A", gloss),
    // TT1 kitchenware
    kettle = ("#F2F0EA", gloss),
    toaster = ("#E3D9C6", gloss),
    blender_base = ("#3A3D41", satin),
    jug = ("#DCE6EA", gloss),          // opaque, so the smoothie shows as the jug's lower half
    smoothie = ("#E9A3B5", gloss),
    ice_cream_maker = ("#C3C7CB", metal), // its plinth and head, painted; nothing else wears it
    mixer = ("#C0473A", gloss),
    espresso = ("#C3C7CB", metal),
    knife_block = ("#9C6B43", satin),
    crock = ("#E8E2D6", gloss),
    crock_band = ("#4E6E8A", gloss),
    board = ("#C99A68", satin),
    bread_bin = ("#E8E2D6", gloss),
    rack = ("#9C6B43", satin),
    spice_a = ("#C98F4B", matte),
    spice_b = ("#7A8C3A", matte),
    spice_c = ("#B5452E", matte),
    spice_d = ("#E3C66A", matte),
    spice_e = ("#5E4B3A", matte),
    jar_lid = ("#C25B3E", gloss),
    cookie = ("#C98F4B", matte),
    cookie_dark = ("#8A5A36", matte),
    tomato_flesh = ("#EE7A63", satin),
    // TT2 tableware
    fruit_bowl = ("#E8E2D6", gloss),
    plate = ("#F5F3EE", gloss),
    plate_rim = ("#3E5470", gloss),
    napkin = ("#E7E0D4", matte),
    cake_stand = ("#F2F0EA", gloss),
    sponge = ("#E9C8A0", matte),
    icing = ("#F7E7EC", gloss),
    tray = ("#9C6B43", satin),
    teapot = ("#F2F0EA", gloss),
    teapot_trim = ("#3E5470", gloss),
    mug_a = ("#3E5470", gloss),
    mug_b = ("#E9C46A", gloss),
    mug_c = ("#E4B8AE", gloss),
    bottle = ("#2F4A33", gloss),
    candle = ("#F4EBD8", satin),
    flame = ("#FFD27A", gloss, glow("#FFC46A")),   // gloss, never emissive; it glows at night (lit)
    // TT3 books and small plants
    book_a = ("#3E5470", matte),
    book_b = ("#D6A64A", matte),
    book_c = ("#B5653F", matte),
    book_d = ("#9DAF8C", matte),
    book_e = ("#4E7A7F", matte),
    pages = ("#F2F0EA", matte),
    vase = ("#9DC6CF", gloss),
    bud_vase = ("#C25B3E", gloss),
    pot_pale = ("#E8E2D6", matte),
    terracotta = ("#C26E4A", matte),
    pot_white = ("#F2F0EA", gloss),
    succulent = ("#8FB59A", satin),
    succulent_light = ("#A9C9A4", satin),
    cactus = ("#4F8A55", satin),
    sand = ("#D9C7A7", matte),
    // TT4 small ornaments
    frame = ("#2B2C2E", satin),
    photo_sky = ("#9DB7C9", matte),
    photo_hill = ("#8FB27A", matte),
    clock = ("#C0473A", gloss),
    clock_face = ("#F7F4EC", matte),
    globe = ("#4E8FB8", gloss),
    land = ("#8EB46A", matte),
    snow = ("#F7F7F5", satin),
    fir = ("#3F7A4A", satin),
    trophy = ("#D6A64A", metal),
    trophy_base = ("#2B2C2E", gloss),
    plinth = ("#E7E3DC", matte),
    sculpture = ("#2B2C2E", satin),
    // TT5 bath, desk and toys
    soap_tray = ("#E7E3DC", matte),
    dispenser = ("#F2F0EA", gloss),
    tumbler = ("#9DC6CF", gloss),
    soap = ("#E9DCC4", matte),
    towel_a = ("#9DC6CF", matte),
    towel_b = ("#F2F0EA", matte),
    towel_c = ("#E4B8AE", matte),
    perfume_tray = ("#D7E2E6", gloss),
    perfume_a = ("#E9A3B5", gloss),
    perfume_b = ("#E9C46A", gloss),
    perfume_c = ("#9DC6CF", gloss),
    organiser = ("#2B2C2E", satin),
    pen_red = ("#D9483B", gloss),
    pen_blue = ("#3E8EDE", gloss),
    pen_yellow = ("#F2C14E", gloss),
    laptop = ("#B9BCBF", metal),
    logo = ("#E7E7E5", gloss),
    laptop_screen = ("#1F2A36", gloss, glow("#A9C8E8"), screen()),   // the laptop's screen: lit while it's used
    monitor = ("#1E1F22", satin),
    screen = ("#16202B", gloss),
    keys = ("#3A3D41", satin),
    bear = ("#B98656", matte),
    bear_light = ("#E9D5B5", matte),
    block_a = ("#D9483B", satin),
    block_b = ("#3E8EDE", satin),
    block_c = ("#F2C14E", satin),
    block_d = ("#5BAE6A", satin),
    block_e = ("#E07BC3", satin),
}

const BODY: &str = "Body";

/// One of the family's models, its key `model.<id>`.
fn t(
    id: &'static str,
    key: &'static str,
    build: artkit::family::Build,
    size: (f64, f64, f64),
    paint: &[(&'static str, &'static str)],
) -> Model {
    Model::new(id, build, size).key(key).paint(paint)
}

/// The family.
pub fn family() -> Family {
    use desk::*;
    use kitchenware::*;
    use ornaments::*;
    use plants::*;
    use tableware::*;
    Family::new(
        "tabletop",
        M.palette(),
        vec![
            // TT1 kitchenware
            t("kettle", "model.kettle", kettle, (0.228, 0.238, 0.172), &[("kettle", BODY), ("dark", "Base, handle & lid")])
                .motions(motions(r#"{"uses": {"tea": {"clip": "lid_open", "hold": 1.6}}}"#)),
            // the toast down while it's made, popping up as the use ends
            t("toaster", "model.toaster", toaster, (0.28, 0.23, 0.15), &[("toaster", BODY)])
                .motions(motions(r#"{"uses": {"toast": "toast_down"}}"#)),
            // the jug shaking on its base, the beater turning round, for as long as each runs
            t("blender", "model.blender", blender, (0.18, 0.356, 0.146), &[("blender_base", BODY), ("smoothie", "Smoothie")])
                .motions(motions(r#"{"loops": {"smoothie": "whirr"}}"#)),
            // the tub twisting on its collar for as long as it churns
            t("iceCreamMaker", "model.iceCreamMaker", ice_cream_maker, (0.237, 0.403, 0.224), &[("ice_cream_maker", BODY)])
                .motions(motions(r#"{"loops": {"churn": "whirr"}}"#)),
            t("standMixer", "model.standMixer", stand_mixer, (0.2, 0.356, 0.3), &[("mixer", BODY)])
                .motions(motions(r#"{"loops": {"bake": "beat"}}"#)),
            // the coffee running into the cup, then stopped while it's drunk
            t("espressoMachineSmall", "model.espressoMachineSmall", espresso_machine_small, (0.231, 0.31, 0.391), &[("espresso", BODY)])
                .motions(motions(r#"{"uses": {"espresso": {"clip": "brew", "hold": 2.5}}}"#)),
            t("knifeBlock", "model.knifeBlock", knife_block, (0.12, 0.297, 0.232), &[("knife_block", BODY), ("dark", "Handles")]),
            t("utensilCrock", "model.utensilCrock", utensil_crock, (0.132, 0.316, 0.124), &[("crock", BODY)]),
            t("cuttingBoard", "model.cuttingBoard", cutting_board, (0.34, 0.022, 0.23), &[("board", BODY)]),
            t("breadBin", "model.breadBin", bread_bin, (0.34, 0.206, 0.202), &[("bread_bin", BODY), ("wood", "Ends & handle")])
                .motions(motions(r#"{"uses": {"sandwich": {"clip": "roll_open", "hold": 2.4}}}"#)),
            t("spiceJars", "model.spiceJars", spice_jars, (0.3, 0.112, 0.12), &[("rack", BODY)]),
            t("cookieJar", "model.cookieJar", cookie_jar, (0.16, 0.248, 0.158), &[("jar_lid", "Lid")]),
            // TT2 tableware
            t("fruitBowl", "model.fruitBowl", fruit_bowl, (0.28, 0.194, 0.28), &[("fruit_bowl", BODY)]),
            t("plateSet", "model.plateSet", plate_set, (0.378, 0.1, 0.321), &[("plate", BODY), ("plate_rim", "Rim"), ("napkin", "Napkin")]),
            t("cakeStand", "model.cakeStand", cake_stand, (0.28, 0.263, 0.28), &[("cake_stand", BODY), ("icing", "Icing")]),
            t("teapotSet", "model.teapotSet", teapot_set, (0.4, 0.167, 0.26), &[("teapot", BODY), ("tray", "Tray")]),
            t("mugs", "model.mugs", mugs, (0.216, 0.09, 0.212), &[("mug_a", "Mug 1"), ("mug_b", "Mug 2"), ("mug_c", "Mug 3")]),
            t("wineGlasses", "model.wineGlasses", wine_glasses, (0.214, 0.301, 0.197), &[("bottle", BODY)]),
            t("candlesticks", "model.candlesticks", candlesticks, (0.18, 0.33, 0.076), &[("brass", BODY), ("candle", "Candles")])
                .motions(motions(r#"{"night": "lit"}"#)),
            // TT3 books and small plants
            t("bookStack", "model.bookStack", book_stack, (0.25, 0.133, 0.196),
                &[("book_a", "Book 1"), ("book_b", "Book 2"), ("book_c", "Book 3"), ("book_d", "Book 4")]),
            // each cover colour, left to right as they first show (eight books: Book 1-3 colour two each)
            t("booksUpright", "model.booksUpright", books_upright, (0.32, 0.206, 0.126),
                &[("brass", "Bookends"), ("book_a", "Book 1"), ("book_c", "Book 2"), ("book_b", "Book 3"),
                  ("book_e", "Book 4"), ("book_d", "Book 5")]),
            t("vaseFlowers", "model.vaseFlowers", vase_flowers, (0.179, 0.41, 0.172), &[("vase", BODY)]),
            // the flowers are a part of their own, hidden until they are put in (two draw calls once they are);
            // taken out, the same clip goes back along the use, so they come out as they're lifted
            t("vaseEmpty", "model.vaseEmpty", vase_empty, (0.13, 0.24, 0.13), &[("vase", BODY)]).motions(motions(
                r#"{"uses": {"putFlowers": "flowers_open", "takeOut": {"clip": "flowers_open", "over": -1}}, "open": "flowers_open"}"#,
            )),
            t("vaseSmall", "model.vaseSmall", vase_small, (0.08, 0.277, 0.08), &[("bud_vase", BODY)]),
            t("potSucculent", "model.potSucculent", pot_succulent, (0.164, 0.136, 0.164), &[("pot_pale", BODY)]),
            t("potCactus", "model.potCactus", pot_cactus, (0.184, 0.324, 0.132), &[("terracotta", BODY)]),
            t("potSmall", "model.potSmall", pot_small, (0.16, 0.261, 0.154), &[("pot_white", BODY)]),
            // TT4 small ornaments
            t("photoFrame", "model.photoFrame", photo_frame, (0.18, 0.231, 0.114), &[("frame", BODY)]),
            t("tableClock", "model.tableClock", table_clock, (0.11, 0.164, 0.048), &[("clock", BODY), ("brass", "Bells & legs")])
                .motions(motions(r#"{"clock": {"hour": "hour_hand", "minute": "minute_hand", "at": 610}}"#)),
            t("globe", "model.globe", globe, (0.212, 0.31, 0.24), &[("globe", BODY), ("wood_dark", "Stand")]),
            t("snowGlobe", "model.snowGlobe", snow_globe, (0.128, 0.176, 0.128), &[("wood_dark", BODY)]),
            t("trophy", "model.trophy", trophy, (0.148, 0.226, 0.126), &[("trophy", BODY)]),
            t("sculptureSmall", "model.sculptureSmall", sculpture_small, (0.154, 0.219, 0.12), &[("sculpture", BODY), ("brass", "Ball")]),
            // TT5 bath, desk and toys
            t("soapSet", "model.soapSet", soap_set, (0.26, 0.166, 0.12), &[("soap_tray", BODY), ("dispenser", "Dispenser")]),
            t("towelStack", "model.towelStack", towel_stack, (0.267, 0.15, 0.204),
                &[("towel_a", "Towel 1"), ("towel_b", "Towel 2"), ("towel_c", "Towel 3")]),
            t("perfumeTray", "model.perfumeTray", perfume_tray, (0.276, 0.143, 0.176), &[("perfume_tray", BODY)]),
            t("deskOrganiser", "model.deskOrganiser", desk_organiser, (0.265, 0.16, 0.2), &[("organiser", BODY)]),
            // open while it is used (browse, code, write, homework)
            t("laptopClosed", "model.laptopClosed", laptop_closed, (0.32, 0.0205, 0.223), &[("laptop", BODY)])
                .motions(motions(r#"{"uses": {"type": "lid_open"}}"#)),
            t("monitor", "model.monitor", monitor, (0.543, 0.46, 0.324), &[("monitor", BODY)]),
            t("stuffedToy", "model.stuffedToy", stuffed_toy, (0.197, 0.282, 0.192), &[("bear", BODY)]),
            t("toyBlocks", "model.toyBlocks", toy_blocks, (0.222, 0.2, 0.171),
                &[("block_a", "Block 1"), ("block_b", "Block 2"), ("block_c", "Block 3")]),
        ],
        "The tabletop things (crates/art/things/src/tabletop, built by cargo art). Listed in the base manifest's \
         packs. The materials map onto the game's finishes, so each costs one draw call (the see-through ones and \
         the parts kept for a use to come one more, the alarm clock's two hands two more).",
    )
}
