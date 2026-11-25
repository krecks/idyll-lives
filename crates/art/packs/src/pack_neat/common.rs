//! What the neat pack is built from: its palette, folded towels, rounded boxes, bottles and a flat strip's profile.

use artkit::materials::Mat;
use artkit::math::P3;
use artkit::place::GameFrame;
use artkit::{fcyl, loft, palette, ring, soft};

palette! { M:
    body = ("#F4F3EF", satin),       // crisp white: docks, cabinets, cubbies, frames; the main part
    appliance = ("#F7F7F5", gloss),  // the washer and dryer, the robot's dock
    trim = ("#D5D7D8", satin),
    birch = ("#E3CFA8", wood),       // textured birch: the desk, the console
    birch_plain = ("#E3CFA8", satin),
    oak = ("#CDB08A", satin),
    teak = ("#B07A4A", satin),       // the shower bench, plain
    chrome = ("#D0D3D6", chrome),
    steel = ("#9EA3A8", metal),
    brass = ("#C9A24A", metal),
    dark = ("#3A4250", satin),
    screen = ("#2E3A46", gloss),
    screen_glow = ("#7FC8E8", gloss),
    porthole = ("#C9E0E8", gloss),   // porthole and bin windows, plain (no glass draw)
    drum = ("#8C949B", satin),
    teal = ("#4E7A7F", gloss),
    led = ("#6CC27A", gloss),
    tile = ("#F1F0EC", satin),
    joint = ("#D6D4CE", satin),
    glass = ("#DCE8EA", glass),
    water = ("#BFE3F2", gloss),
    pad = ("#F7F7F7", satin),
    rug = ("#D5D7D8", fabric),
    rug_inner = ("#E6E5E0", fabric),
    paper = ("#F6F3EA", matte),
    ledger = ("#3E6E73", satin),
    coral = ("#E8846B", gloss),
    wicker = ("#D8C29C", satin),
    wicker_b = ("#CDB48A", satin),
    green = ("#6FA35A", satin),
    stem = ("#5E8F4E", satin),
    mirror = ("#DCE6EA", chrome),
    shoe_a = ("#B33A32", satin),
    shoe_b = ("#3E6E73", satin),
    shoe_c = ("#E9D58A", satin),
    // soft pastels: towels and shirts (fabric), bins and bottles (satin, gloss)
    towel_a = ("#9FD3C7", fabric),
    towel_b = ("#F2C6A0", fabric),
    towel_c = ("#B8C9E8", fabric),
    towel_d = ("#E9D58A", fabric),
    towel_e = ("#D9B8D6", fabric),
    towel_f = ("#A8D5A2", fabric),
    towel_w = ("#F7F7F7", fabric),
    pastel_a = ("#9FD3C7", satin),
    pastel_b = ("#F2C6A0", satin),
    pastel_c = ("#B8C9E8", satin),
    pastel_d = ("#E9D58A", satin),
    pastel_e = ("#D9B8D6", satin),
    pastel_f = ("#A8D5A2", satin),
    bottle_a = ("#9FD3C7", gloss),
    bottle_b = ("#B8C9E8", gloss),
    bottle_c = ("#F2C6A0", gloss),
}

/// The towels' colours.
pub const TOWELS: [Mat; 6] = [
    M.towel_a, M.towel_b, M.towel_c, M.towel_d, M.towel_e, M.towel_f,
];
pub const PASTELS: [Mat; 6] = [
    M.pastel_a, M.pastel_b, M.pastel_c, M.pastel_d, M.pastel_e, M.pastel_f,
];
/// The wall's face in a 1-deep footprint, game depth.
pub use artkit::kit::WALL;

/// `n` folded towels stacked on `y`, each plump with its fold rounded; returns the stack's top.
#[allow(clippy::too_many_arguments)]
pub fn _towels(
    f: &GameFrame,
    name: &str,
    x: f64,
    z: f64,
    y: f64,
    n: usize,
    w: f64,
    d: f64,
    colours: &[Mat],
) -> f64 {
    for i in 0..n {
        soft!(
            f,
            format!("{name}{i}"),
            w,
            0.044,
            d,
            (x + 0.004 * (i % 2) as f64, y + 0.022 + i as f64 * 0.046, z),
            colours[i % colours.len()],
            round = 0.018,
            puff = (0, 0.004, 0),
            flat = 1,
            round_segments = 1
        );
    }
    y + n as f64 * 0.046
}

/// A bottle: a rounded-square body, its shoulder, a neck and a cap.
#[allow(clippy::too_many_arguments)]
pub fn _bottle(
    f: &GameFrame,
    name: &str,
    x: f64,
    z: f64,
    y: f64,
    material: Mat,
    h: f64,
    d: f64,
    cap: Option<Mat>,
) {
    let r = d / 2.0;
    let b = f.at(x, y, z);
    let sections: Vec<Vec<P3>> = [
        (0.0, 0.92, 3),
        (0.03, 1.0, 3),
        (0.7, 1.0, 3),
        (0.84, 0.6, 2),
        (0.88, 0.42, 2),
        (1.0, 0.42, 2),
    ]
    .iter()
    .map(|&(t, k, sq)| {
        ring!(
            r * k,
            r * k,
            z = b.z + h * t,
            verts = 12,
            square = sq,
            at = (b.x, b.y)
        )
    })
    .collect();
    loft!(name, &sections, material = material);
    fcyl!(
        f,
        format!("{name}_cap"),
        r * 0.48,
        0.03,
        (x, y + h + 0.012, z),
        cap.unwrap_or(M.body),
        verts = 10,
        bevel = 0.003,
        bevel_segments = 1
    );
}

/// A flat strip's profile, `w` wide and `t` thick (across, through).
pub fn _flat(w: f64, t: f64) -> Vec<(f64, f64)> {
    vec![
        (-t / 2.0, -w / 2.0),
        (t / 2.0, -w / 2.0),
        (t / 2.0, w / 2.0),
        (-t / 2.0, w / 2.0),
    ]
}
