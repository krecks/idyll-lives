//! The living room's media (L4, notes: NOTES.md): the television on its media unit, the flat-screen TV on the wall,
//! the portable TV, the low TV bench with a TV lift, the home cinema, the stereo, the boombox and the electric
//! fireplace.
//!
//! Each is built where the game places it (`F`, game axes: x, height, depth; front +depth), its back on the wall
//! face (-0.43) where the old part's reached past it, then stood on the floor (`settle!(centre = false)`, its
//! `centre` on the Model). Every screen faces the front (+depth), where the game stands its watchers, and glows at
//! night (`glow`); the home cinema's projector screen is rolled away at rest, so it glows only while it is down,
//! while watched.
//!
//! What moves:
//!   tvRetro            the aerial lifts 35° off the case (`antenna`, for watch);
//!   tvStandLow         the TV rises 60 cm out of the bench's lift (`tv_up`, for watch, back down at the end);
//!   homeCinema         the projector screen unrolls from its case on the wall (`screen_down`, watch and film);
//!   stereo             the record turns on the turntable (`record`, a loop) and the woofers pulse (`pulse`, `every`);
//!   boombox            the cassette door tips open and shuts (`cassette`, hold 1.2 s);
//!   fireplaceElectric  the flames grow from the log bed (`flames`, for relax; gloss, glowing at night).

use artkit::family::{Family, Model};
use artkit::json;
use artkit::materials::{Mat, Palette};
use artkit::palette;
use artkit::place::GameFrame;

mod bench;
mod cinema;
mod common;
mod fireplace;
mod stereo;
mod tv;

palette! { M:
    // shared
    screen = ("#16202B", gloss, glow("#8DB4E2")), // every flat screen, dark by day; an LCD's cool light at night
    black_metal = ("#2B2C2E", metal), // feet, pulls, brackets
    steel = ("#C3C7CB", metal), // knobs, tonearm, ears
    grille = ("#2E3034", matte), // speaker cloth
    led = ("#5EC28B", gloss),
    // the television on its media unit (the old `tv_set`)
    tv_cabinet = ("#B08E66", wood), // the unit; Body
    tv_oak = ("#CDB08A", wood), // its top board and doors; Trim
    tv_frame = ("#2A2C30", satin), // the set's casing and stand
    soundbar = ("#3A3D41", satin),
    bowl = ("#F2F2F0", gloss),
    pebble = ("#E9E6E0", gloss),
    // the flat-screen TV on the wall
    wall_frame = ("#2A2C30", satin), // Body
    wall_bar = ("#3A3D41", satin), // the soundbar; Trim
    // the portable TV: a dark plastic case
    retro_shell = ("#43474E", satin), // Body
    retro_bezel = ("#141619", gloss), // the screen's surround, the controls, the pads; Trim
    crt = ("#26313D", gloss, glow("#A8C8E0")),
    // the low TV bench
    bench = ("#F1F0EC", satin), // Bench (and the lift's lid, on the TV)
    bench_oak = ("#CDB08A", wood), // top and legs
    bench_tv = ("#1E1F22", satin), // TV
    plant = ("#5E8F4E", satin),
    pot = ("#E8E6E1", gloss),
    // the home cinema
    cinema_unit = ("#B08E66", wood), // Body
    cinema_oak = ("#CDB08A", wood), // top and door fronts; Trim
    tower = ("#3A3D41", satin), // the tower speakers, the soundbar, the screen's case
    cone = ("#22252A", satin),
    projector = ("#E6E6E3", satin),
    lens = ("#1A1C20", gloss),
    projection = ("#F4F4F1", matte, glow("#DCE6F5")), // the pulled-down screen, lit while watched at night
    screen_border = ("#1E1F22", matte),
    // the stereo
    stereo_wood = ("#CDB08A", wood), // Body
    stereo_dark = ("#3A3D41", satin), // the amplifier, the CD player, the baffles; Trim
    stereo_inside = ("#2A2620", matte),
    display = ("#7FC8E8", gloss),
    disc = ("#D9DDE2", chrome),
    vinyl = ("#141414", satin),
    label = ("#B5653F", satin),
    rec_rust = ("#B5653F", satin),
    rec_mustard = ("#D6A64A", satin),
    rec_teal = ("#4E7A7F", satin),
    rec_cream = ("#EFE5D3", satin),
    rec_navy = ("#3E5470", satin),
    rec_clay = ("#C98F6B", satin),
    rec_black = ("#2A2C30", satin),
    // the boombox on its crate
    boombox_body = ("#3A3D41", satin), // Body
    boombox_trim = ("#C9CBCC", metal), // Trim: the top, rings, handle and aerial
    crate_dark = ("#9E6E43", wood),
    cassette = ("#1D2228", gloss),
    window = ("#3A4450", gloss),
    // the electric fireplace
    surround = ("#F4F3EF", satin), // Surround
    firebox = ("#16181B", gloss), // the firebox's back and its black frame; Firebox
    mantel = ("#CDB08A", wood),
    logs = ("#4A3B30", matte),
    ember = ("#E0672E", gloss, glow("#FF8A3C")),
    flame = ("#FF9A3C", gloss, glow("#FFB25C")),
    flame_core = ("#FFD27A", gloss, glow("#FFE0A0")),
    candle = ("#EFE5D3", gloss),
    hearth_cushion = ("#B5653F", matte), // the floor cushion residents warm up on (characters.ts SEATS)
    hearth_piping = ("#9A5433", matte),
    picture = ("#2B2C2E", satin),
    art_print = ("#C9B79C", matte),
    print_sky = ("#9DC6CF", matte),
    print_hill = ("#7FA86A", matte),
    candle_wax = ("#F4EBDC", satin),
    firebox_line = ("#24272B", gloss), // the flutes on the firebox's back
    grille_dark = ("#1F2124", matte), // ribs on speaker cloth
    led_red = ("#D9483B", gloss),
    steel_dark = ("#55595E", metal), // the amplifier's brushed face
    glass_lens = ("#2E4A66", gloss),
}

/// The record crate's wood (a name the palette's fields can't take: `crate` is Rust's).
pub const CRATE: Mat = Mat::new("crate", "#A8754A", "wood");

const F: GameFrame = GameFrame::new(0.0);

/// The family's palette, `crate` in its place after the boombox's trim.
fn colours() -> Palette {
    let mut p = M.palette();
    let at = p
        .colours
        .iter()
        .position(|m| m.name == "boombox_trim")
        .map_or(p.colours.len(), |i| i + 1);
    p.colours.insert(at, CRATE);
    p
}

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "media",
        colours(),
        vec![
            Model::new("tv", tv::tv, (0.91, 1.07, 0.445))
                .key("model.tv")
                .centre(0.0, -0.21)
                .paint(&[("tv_cabinet", "Body"), ("tv_oak", "Trim")]),
            Model::new("tvWall", tv::tv_wall, (1.24, 0.87, 0.102))
                .key("model.tvWall")
                .centre(0.0, -0.379)
                .on_wall()
                .paint(&[("wall_frame", "Body"), ("wall_bar", "Trim")]),
            Model::new("tvRetro", tv::tv_retro, (0.886, 0.786, 0.426))
                .key("model.tvRetro")
                .centre(0.013, -0.217)
                .motions(json!({"uses": {"watch": "antenna"}}))
                .paint(&[("retro_shell", "Body"), ("retro_bezel", "Trim")]),
            Model::new("tvStandLow", bench::tv_stand_low, (1.82, 0.8, 0.464))
                .key("model.tvStandLow")
                .centre(0.0, -0.198)
                .motions(json!({"uses": {"watch": "tv_up"}}))
                .paint(&[("bench", "Bench"), ("bench_tv", "TV")]),
            Model::new("homeCinema", cinema::home_cinema, (1.88, 1.65, 0.448))
                .key("model.homeCinema")
                .centre(0.0, -0.206)
                .motions(json!({"uses": {"watch": "screen_down", "film": "screen_down"}}))
                .paint(&[("cinema_unit", "Body"), ("cinema_oak", "Trim")]),
            Model::new("stereo", stereo::stereo, (0.92, 0.84, 0.42))
                .key("model.stereo")
                .centre(0.0, -0.22)
                // while the music plays the record turns (a loop) and the woofers pulse (`every` a hair under their beat)
                .motions(json!({"uses": {"listen": {"clip": "pulse", "every": 0.45},
                                         "dance": {"clip": "pulse", "every": 0.45}},
                                "loops": {"listen": "record", "dance": "record"}}))
                .paint(&[("stereo_wood", "Body"), ("stereo_dark", "Trim")]),
            Model::new("boombox", stereo::boombox, (0.6, 0.95, 0.44))
                .key("model.boombox")
                .motions(json!({"uses": {"listen": "cassette"}, "hold": 1.2}))
                .paint(&[("boombox_body", "Body"), ("boombox_trim", "Trim")]),
            Model::new(
                "fireplaceElectric",
                fireplace::fireplace_electric,
                (1.76, 1.31, 0.92),
            )
            .key("model.fireplaceElectric")
            .centre(0.0, 0.03)
            .motions(json!({"uses": {"relax": "flames"}}))
            .paint(&[("surround", "Surround"), ("firebox", "Firebox")]),
        ],
        "The living room's media (crates/art/things/src/media, built by cargo art): the TVs, the home cinema, the \
         stereo, the boombox and the electric fireplace. The materials map onto the game's finishes so a model costs \
         one draw call per finish group.",
    )
    .switched())
}
