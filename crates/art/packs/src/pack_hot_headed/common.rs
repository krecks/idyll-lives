//! What the hot-headed pack is built from: its palette, fluted rings and chains.

use artkit::materials::Mat;
use artkit::math::{Axis, Quat, V3};
use artkit::parts::numbered;
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{palette, torus};

palette! { M:
    body = ("#C49A6C", wood),        // the grudge desk's oak top and its chair; the desk's main part
    bag = ("#C8402F", leather),      // the heavy bag's leather
    crate_ = ("#A87547", wood, named("crate")), // the smash crate and its lid (`crate` is Rust's)
    shell = ("#C8402F", gloss),      // the drum shells
    zafu = ("#5F7FA0", fabric),      // the meditation cushion
    cedar = ("#C98B58", wood),       // the plunge tub's cladding
    fire = ("#C8402F", satin),       // red details: weight plate, gloves, lamp shade
    ember = ("#E07A2E", satin),
    flame = ("#F2B84B", gloss),      // logos, hazard tape
    steel = ("#AEB4BA", metal),
    steel_dark = ("#7E858C", metal),
    chrome = ("#D0D3D6", chrome),
    cymbal = ("#C9A24A", metal),
    brass = ("#C9A54A", metal),
    dark = ("#2F3236", satin),       // straps, castors, the drum throne, pen
    rubber = ("#3E4A55", satin),     // floor mats, the drum rug
    rug = ("#6E3A2E", satin),
    head = ("#F4F1EA", satin),       // drum heads
    porcelain = ("#F4F1EA", gloss),  // plates and shards
    plate_blue = ("#9CC3DB", gloss),
    plate_gold = ("#F2D7A0", gloss),
    plate_green = ("#C8E0C0", gloss),
    crate_dark = ("#7C5432", wood),  // the crate's posts and floor
    straw = ("#D2B872", matte),
    oak = ("#B88A5C", satin),        // plain wood: stool, bowl stand, drum sticks, mallet
    calm = ("#5F7FA0", satin),       // the chair's cushion
    calm_light = ("#A9C2D6", fabric),  // the meditation mat
    calm_deep = ("#4E6E8E", fabric),   // the zafu's bands and the mat's edge
    oat = ("#E6DCC8", fabric),       // the zabuton
    candle = ("#FBF6EC", satin),
    candle_flame = ("#FFD27A", gloss),
    leaf = ("#5E8F4E", satin),
    pot = ("#F4F1EA", satin),
    paper = ("#F4EFE2", matte),
    journal = ("#7A3A2A", satin),
    drawer = ("#E6E3DC", satin),     // the desk's drawer case and the chiller
    cactus = ("#6E9E58", satin),
    shell_white = ("#F2F2F0", satin),  // the plunge tub's liner and rims
    water = ("#BFE6F0", gloss),
    ice = ("#F4FAFC", gloss),
    cover = ("#5F7FA0", satin),      // the plunge cover's slats
    screen = ("#1F3A40", gloss),
    readout = ("#7FD1F0", gloss),
    hose = ("#5F6368", satin),
    towel = ("#F4EFE2", satin),
    amp = ("#E4E1DA", satin),
    speaker = ("#3A3D41", satin),
    ash = ("#8E8A84", matte),        // the incense's ash
}

/// A chain of oval links from game point `a` to `b`, each turned a quarter to the last.
#[allow(clippy::too_many_arguments)]
pub fn chain(
    f: &GameFrame,
    name_: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    material: Mat,
    pitch: f64,
    r: f64,
    t: f64,
) -> Vec<Obj> {
    let (a, b) = (V3::from(f.at(a.0, a.1, a.2)), V3::from(f.at(b.0, b.1, b.2)));
    let d = b - a;
    let n = 2.max(artkit::py::round(d.length() / pitch));
    let q0 = d.to_track_quat(Axis::X, Axis::Z);
    let mut made = Vec::new();
    for k in 0..n {
        let c = a + d * ((k as f64 + 0.5) / n as f64);
        let link = torus!(
            numbered(name_),
            r,
            t,
            at = c,
            material = material,
            major_segments = 8,
            minor_segments = 3
        );
        let q = q0
            * Quat::from_axis_angle((1.0, 0.0, 0.0), std::f64::consts::PI / 2.0 * (k % 2) as f64);
        link.edit(|o| o.quat = Some(q));
        link.set_scale((d.length() / n as f64 / (2.0 * r) * 1.25, 0.75, 1.0));
        made.push(link);
    }
    made
}
