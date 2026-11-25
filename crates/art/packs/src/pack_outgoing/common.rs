//! What the outgoing pack is built from: its palette, the bottles, glasses, stools and speakers more than one of its
//! models stands about.

use artkit::materials::Mat;
use artkit::math::{P3, cosd, sind};
use artkit::place::GameFrame;
use artkit::{fcone, fcyl, fsweep, loft, palette, revolve, ring, soft, torus};
use artkit::{forms::eased, shapes::AUTO_WIDTH};

palette! { M:
    body = ("#E8735A", satin),       // coral: the bar's counter, the desk frame, the stage, the floor's frame
    upholstery = ("#3FA9A5", fabric), // the sectional's cushions (fabric); see the per-model notes
    vinyl = ("#E8735A", satin),      // bar stool tops
    cushion = ("#3FA9A5", satin),    // the desk chair's seat and back
    frame = ("#3FA9A5", fabric),     // the sectional's frame, back and arms
    wood = ("#8A5E3C", wood),        // the back bar, the stage deck, the desk top, the photo board's frame
    oak = ("#CDB08A", wood),         // the bar's counter top and shelves
    oak_plain = ("#CDB08A", satin),  // the sectional's console (plain: the sofa takes the textured draw)
    brass = ("#B8925A", metal),
    chrome = ("#D0D3D6", chrome),
    steel = ("#C9CCD0", metal),      // the dance floor's gantry, mic bases
    dark = ("#3A3D41", satin),       // speaker cones, the ring light's stand
    speaker = ("#E4E1DA", satin),    // speaker cabinets
    screen = ("#26305A", gloss, glow("#5A78D8"), screen()),   // the karaoke and laptop screens: lit while used
    lyric = ("#F2F2F0", gloss, glow("#FFFFFF"), screen()),
    lyric_hi = ("#F2C14E", gloss, glow("#FFD45A"), screen()),   // the line being sung
    sunny = ("#F2C14E", satin),
    coral = ("#E8735A", gloss),
    magenta = ("#D64B8A", gloss),
    turq = ("#3FA9A5", gloss),
    plum = ("#6B4A7E", satin),
    pouf = ("#F2C14E", fabric),
    cream = ("#F4EDE4", satin),
    pot = ("#E9E6E0", satin),
    leaf = ("#5E8F4E", satin),       // plants plain, so no foliage draw
    lime = ("#9CCB4A", satin),
    porcelain = ("#F5F3EE", gloss),
    glassy = ("#DDEBEC", gloss),     // glasses, plain gloss (no see-through draw)
    beer = ("#E8B04A", gloss),
    laptop = ("#D9DADB", metal),
    face = ("#E8B892", gloss),
    ring = ("#FFF6E2", gloss, glow("#FFF3D8"), screen()),   // the ring light: on while the call lasts
    cork = ("#C9A27A", matte),
    paper = ("#FAFAF7", satin),
    bulb = ("#FFE9A8", gloss, glow("#FFDC8A")),   // the photo wall's string of bulbs: lit at night
    string = ("#3A3D41", satin),
    base = ("#E4E1DA", satin),       // the dance floor's base
    mirror = ("#E8EEF2", chrome),
    tile_a = ("#D64B8A", gloss, glow("#FF5AA8"), screen()),   // the floor's tiles light up while it's danced on
    tile_b = ("#3FA9A5", gloss, glow("#4FE0D8"), screen()),
    tile_c = ("#F2C14E", gloss, glow("#FFD860"), screen()),
    tile_d = ("#8E6BD6", gloss, glow("#B08CFF"), screen()),
    tile_e = ("#58C7E8", gloss, glow("#70DCFF"), screen()),
    tile_f = ("#E8735A", gloss, glow("#FF8A6A"), screen()),
    bottle_a = ("#2E7D5B", gloss),
    bottle_b = ("#B5372E", gloss),
    bottle_c = ("#E8B04A", gloss),
    bottle_d = ("#6B4A7E", gloss),
    bottle_e = ("#2E5B8E", gloss),
    bottle_f = ("#E07A2E", gloss),
    wine_a = ("#2E4A30", gloss),
    wine_b = ("#5A1F2A", gloss),
    label = ("#EFE5D3", satin),
    photo_a = ("#7FB069", gloss),
    photo_b = ("#8E6BD6", gloss),
    photo_c = ("#58C7E8", gloss),
    pin_a = ("#D9483B", gloss),
    pin_b = ("#3E8EDE", gloss),
}

// Colours in turn.
pub const TILES: [Mat; 6] = [M.tile_a, M.tile_b, M.tile_c, M.tile_d, M.tile_e, M.tile_f];
pub const BOTTLES: [Mat; 8] = [
    M.bottle_a, M.bottle_b, M.bottle_c, M.bottle_d, M.bottle_e, M.bottle_f, M.magenta, M.turq,
];
pub const PHOTOS: [Mat; 8] = [
    M.coral, M.turq, M.sunny, M.magenta, M.photo_b, M.photo_a, M.photo_c, M.beer,
];
/// The wall's face in a 1-deep footprint, game depth.
pub use artkit::kit::WALL;

/// A spirit bottle standing on `y`: a body (round, or square as a whisky's), its label, a shoulder, neck and cap.
#[allow(clippy::too_many_arguments)]
pub fn _bottle(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    h: f64,
    material: Mat,
    square: bool,
) {
    let b = f.at(x, y, z);
    // (a square one no squarer than 2.6: its eight points then turn under 60° at the corners, not edges that show)
    let sq = if square { 2.6 } else { 2.0 };
    let sections: Vec<Vec<P3>> = [
        (0.0, 0.94, sq),
        (0.02, 1.0, sq),
        (0.6, 1.0, sq),
        (0.72, 0.7, 2.0),
        (0.8, 0.34, 2.0),
        (1.0, 0.34, 2.0),
    ]
    .iter()
    .map(|&(t, k, s)| {
        ring!(
            0.036 * k,
            0.036 * k,
            z = b.z + h * t,
            verts = 8,
            square = s,
            at = (b.x, b.y)
        )
    })
    .collect();
    loft!(name, &sections, material = material);
    let label: Vec<Vec<P3>> = [0.22, 0.44]
        .iter()
        .map(|&t| {
            ring!(
                0.0375,
                0.0375,
                z = b.z + h * t,
                verts = 8,
                square = sq,
                at = (b.x, b.y)
            )
        })
        .collect();
    loft!(format!("{name}_label"), &label, material = M.label);
    fcyl!(
        f,
        format!("{name}_cap"),
        0.014,
        0.025,
        (x, y + h + 0.01, z),
        M.brass,
        verts = 8
    );
}

/// A stemmed cocktail glass standing on `y`, a drink in its bowl.
#[allow(clippy::too_many_arguments)]
pub fn _glass(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    h: f64,
    r: f64,
    drink: Option<Mat>,
) {
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (r * 0.8, 0.0),
            (r * 0.8, 0.006),
            (0.006, 0.012),
            (0.005, h * 0.5),
            (r, h),
            (r * 0.85, h),
            (0.0, h * 0.55)
        ],
        at = f.at(x, y, z),
        material = M.glassy,
        segments = 8
    );
    if let Some(drink) = drink {
        fcone!(
            f,
            format!("{name}_drink"),
            0.006,
            r * 0.8,
            h * 0.36,
            (x, y + h * 0.75, z),
            drink,
            verts = 8
        );
    }
}

/// A bar stool: a stuffed round seat on a brass ring, a column, a foot ring and a domed base.
pub fn _stool(f: &GameFrame, name: &str, x: f64, z: f64, h: f64) {
    let b = f.at(x, h - 0.07, z);
    let sections: Vec<Vec<P3>> = [
        (0.0, 0.9),
        (0.006, 1.0),
        (0.05, 1.0),
        (0.066, 0.85),
        (0.07, 0.0),
    ]
    .iter()
    .map(|&(hh, k)| {
        ring!(
            0.165 * k,
            0.165 * k,
            z = b.z + hh,
            verts = 20,
            at = (b.x, b.y)
        )
    })
    .collect();
    loft!(format!("{name}_top"), &sections, material = M.vinyl);
    fcyl!(
        f,
        format!("{name}_ring"),
        0.15,
        0.025,
        (x, h - 0.08, z),
        M.brass,
        verts = 20
    );
    fsweep!(
        f,
        format!("{name}_post"),
        &[(x, 0.02, z), (x, h - 0.09, z)],
        0.022,
        M.brass,
        verts = 10
    );
    torus!(
        format!("{name}_foot"),
        0.13,
        0.009,
        at = f.at(x, 0.28, z),
        material = M.brass,
        major_segments = 18,
        minor_segments = 4
    );
    for a in [0, 120, 240] {
        // the foot ring's struts
        let af = a as f64;
        fsweep!(
            f,
            format!("{name}_strut{a}"),
            &[
                (x, 0.28, z),
                (x + 0.13 * cosd(af), 0.28, z + 0.13 * sind(af))
            ],
            0.006,
            M.brass,
            verts = 4
        );
    }
    revolve!(
        format!("{name}_base"),
        &[
            (0.0, 0.0),
            (0.15, 0.0),
            (0.15, 0.008),
            (0.1, 0.022),
            (0.03, 0.035),
            (0.0, 0.035)
        ],
        at = f.at(x, 0, z),
        material = M.brass,
        segments = 20
    );
}

/// A speaker cabinet centred on (x, y, z), its edges rounded, its turned cones (a surround, the cone, a dust cap) on
/// the face towards `facing` (+1: the front).
#[allow(clippy::too_many_arguments)]
pub fn _speaker(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    d: f64,
    cone_ys: &[f64],
    facing: i32,
) {
    soft!(
        f,
        name,
        w,
        h,
        d,
        (x, y, z),
        M.speaker,
        round = 0.02,
        round_segments = 1
    );
    let zf = z + facing as f64 * d / 2.0;
    for (k, &cy) in cone_ys.iter().enumerate() {
        let r = w * if k == 0 { 0.36 } else { 0.2 };
        revolve!(
            format!("{name}_cone{k}"),
            &eased(
                &[
                    (0.0, -0.002),
                    (r * 1.08, -0.002),
                    (r * 1.08, 0.004),
                    (r, 0.01),
                    (r * 0.82, 0.004),
                    (r * 0.32, -0.012),
                    (r * 0.3, -0.006),
                    (r * 0.2, 0.0),
                    (0.0, 0.004)
                ],
                AUTO_WIDTH
            ),
            at = f.at(x, cy, zf),
            material = if k != 0 { M.dark } else { M.coral },
            segments = 16,
            rot = (90 * facing, 0, 0)
        );
    }
}
