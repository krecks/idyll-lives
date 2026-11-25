//! What the romantic pack is built from: its palette, candles and their flames, roses and their leaves, a wine glass
//! and the velvet slipper chair.

use artkit::forms::smooth_outline;
use artkit::materials::{Mat, Mirror};
use artkit::math::{Axis, M4, V3, radians};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fcyl, fextrude, fsweep, leaf, palette, revolve, soft};
use artkit::{forms::eased, shapes::AUTO_WIDTH};

palette! { M:
    body = ("#C9A54A", metal),       // gold: the table's and chairs' legs; the main part of the table
    fabric = ("#E8B4B8", fabric),    // blush velvet: chair seats and backs
    cloth = ("#F4EDE4", fabric),     // the tablecloth, towels
    runner = ("#9E2F44", fabric),
    ivory = ("#F4EDE4", satin),      // the bureau, the vanity, the tub's plinth
    tub = ("#E8B4B8", gloss),        // the heart tub's shell
    blush = ("#E8B4B8", satin),
    blushd = ("#D99AA0", satin),
    velvet = ("#E8B4B8", satin),     // the vanity's pouf (plain)
    rose = ("#C8475A", satin),
    rose_pink = ("#E07A8A", satin),
    rosed = ("#9E2F44", gloss),      // wax seal, wine
    rosegold = ("#D4A28C", metal),
    gold = ("#C9A54A", metal),
    oak = ("#CDB08A", satin),
    wax = ("#FBF6EC", satin),
    flame = ("#F28C28", gloss, glow("#FFB050")),        // candle flames: they glow at night
    flame_core = ("#FFC24A", gloss, glow("#FFD27A")),
    porcelain = ("#FBFAF7", gloss),
    glassy = ("#E4EEEE", gloss),
    dessert = ("#C77D4F", matte),
    water = ("#C7E6EE", gloss),
    bubble = ("#FBFAF7", gloss),
    chrome = ("#C9CCD0", chrome),
    bottle = ("#2E5B3A", gloss),
    leaf = ("#5E8F4E", satin),
    leaf_dark = ("#4F7F3E", satin),
    mirror = ("#DCE6EA", gloss, mirror(Mirror::Front)),
    perfume_a = ("#F2B8C6", gloss),
    perfume_b = ("#F2C77F", gloss),
    perfume_c = ("#E8C9F0", gloss),
    cabinet = ("#A8754A", satin),    // the gramophone's record cabinet; its main part
    walnut = ("#8A5E3C", satin),
    door = ("#B5835A", satin),
    horn = ("#C9A54A", metal),
    record = ("#2A2422", gloss),
    sleeve_a = ("#C8475A", matte),
    sleeve_b = ("#3E6F9E", matte),
    paper = ("#FBF7EE", matte),
    envelope = ("#F2D7C6", matte),
    ink = ("#2E3C6E", gloss),
    frame = ("#F2EFE8", satin),      // the arbour's paint; its main part
    cushion = ("#E8B4B8", satin),    // the swing's seat cushion
}

/// The wall's face in a 1-deep footprint, game depth.
pub use artkit::kit::WALL;

/// A candlestick standing on `y`, its candle a little melted at the top; returns the wick's top (x, height, z).
#[allow(clippy::too_many_arguments)]
pub fn _candle(
    f: &GameFrame,
    name: &str,
    x: f64,
    z: f64,
    y: f64,
    h: f64,
    holder: Option<Mat>,
) -> (f64, f64, f64) {
    revolve!(
        format!("{name}_stick"),
        &[
            (0.0, 0.0),
            (0.038, 0.0),
            (0.036, 0.01),
            (0.016, 0.022),
            (0.011, 0.04),
            (0.015, 0.05),
            (0.024, 0.056),
            (0.024, 0.062),
            (0.0, 0.062)
        ],
        at = f.at(x, y, z),
        material = holder.unwrap_or(M.gold),
        segments = 12
    );
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.014, 0.0),
            (0.014, h - 0.006),
            (0.011, h),
            (0.006, h - 0.004),
            (0.0, h - 0.003)
        ],
        at = f.at(x, y + 0.06, z),
        material = M.wax,
        segments = 10
    );
    fcyl!(
        f,
        format!("{name}_wick"),
        0.0015,
        0.012,
        (x, y + 0.06 + h + 0.003, z),
        M.ink,
        verts = 4
    );
    (x, y + 0.06 + h, z)
}

/// A candle flame: a teardrop turned round, a brighter core inside it.
pub fn _flame(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, s: f64) {
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.008 * s, 0.006 * s),
            (0.011 * s, 0.016 * s),
            (0.008 * s, 0.03 * s),
            (0.0, 0.048 * s)
        ],
        at = f.at(x, y, z),
        material = M.flame,
        segments = 8
    );
    revolve!(
        format!("{name}_core"),
        &[
            (0.0, 0.002),
            (0.006 * s, 0.008 * s),
            (0.007 * s, 0.016 * s),
            (0.0, 0.03 * s)
        ],
        at = f.at(x, y - 0.001, z),
        material = M.flame_core,
        segments = 6
    );
}

/// A bloom `s` round, its foot at (x, y, z): a tight bud in two cups of petals, each a thick turned cup (seen into
/// from above, so it has an inside), the outer one flaring; turned `tilt` towards `yaw`.
#[allow(clippy::too_many_arguments)]
pub fn _rose(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    s: f64,
    material: Option<Mat>,
    segments: usize,
    tilt: f64,
    yaw: f64,
) -> Vec<Obj> {
    let m = material.unwrap_or(M.rose);
    // the outer petals' edge rolled over on a bloom big enough for its edge to show (each side 5 cm or more)
    let edge: &[(f64, f64)] = if s >= 0.055 {
        &[(0.99, 0.6), (0.985, 0.625), (0.95, 0.632)]
    } else {
        &[(1.0, 0.62)]
    };
    let mut outer = vec![(0.0, 0.0), (s * 0.55, s * 0.1), (s * 0.95, s * 0.45)];
    outer.extend(edge.iter().map(|&(r, h)| (s * r, s * h)));
    outer.extend([(s * 0.86, s * 0.58), (s * 0.5, s * 0.3), (0.0, s * 0.22)]);
    let made = vec![
        revolve!(
            format!("{name}_outer"),
            &outer,
            material = m,
            segments = segments
        ),
        revolve!(
            format!("{name}_inner"),
            &[
                (0.0, s * 0.18),
                (s * 0.5, s * 0.3),
                (s * 0.66, s * 0.7),
                (s * 0.56, s * 0.74),
                (s * 0.3, s * 0.52),
                (0.0, s * 0.5)
            ],
            material = m,
            segments = segments - 1
        ),
        revolve!(
            format!("{name}_bud"),
            &[
                (0.0, s * 0.45),
                (s * 0.3, s * 0.55),
                (s * 0.28, s * 0.8),
                (0.0, s * 0.86)
            ],
            material = m,
            segments = 5
        ),
    ];
    // (each bloom turned about its own axis by its name's length, so no two in a bunch show the same petals)
    let turn = M4::translation(V3::from(f.at(x, y, z)))
        * M4::rotation(radians(-yaw), Axis::Z)
        * M4::rotation(radians(-tilt), Axis::X)
        * M4::rotation(radians((name.len() * 23) as f64), Axis::Z);
    for &o in &made {
        o.edit(|p| p.mesh.transform(&turn));
    }
    made
}

/// A few rose leaves round a bloom's foot.
#[allow(clippy::too_many_arguments)]
pub fn _rose_leaves(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    n: usize,
    size: f64,
    yaw0: f64,
    material: Option<Mat>,
) {
    for k in 0..n {
        leaf!(
            format!("{name}{k}"),
            f.at(x, y, z),
            yaw0 + k as f64 * 360.0 / n as f64,
            -10.0,
            size,
            size * 0.6,
            material.unwrap_or(M.leaf),
            droop = 0.25,
            rows = 3,
            ring = 3
        );
    }
}

pub fn _wine_glass(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, wine: bool) {
    revolve!(
        name,
        &eased(
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.03, 0.005),
                (0.004, 0.01),
                (0.004, 0.08),
                (0.032, 0.11),
                (0.035, 0.17),
                (0.031, 0.17),
                (0.0, 0.09)
            ],
            AUTO_WIDTH
        ),
        at = f.at(x, y, z),
        material = M.glassy,
        segments = 10
    );
    if wine {
        revolve!(
            format!("{name}_wine"),
            &[(0.0, 0.088), (0.022, 0.1), (0.031, 0.125), (0.0, 0.125)],
            at = f.at(x, y, z),
            material = M.rosed,
            segments = 10
        );
    }
}

/// A slipper chair centred on (x, z): a stuffed seat (top 0.45) and a tall rounded back in velvet, on tapered legs,
/// facing the object's front, or its back (`back`: its back on the far side).
#[allow(clippy::too_many_arguments)]
pub fn _velvet_chair(
    f: &GameFrame,
    name: &str,
    x: f64,
    z: f64,
    frame: Option<Mat>,
    seat: Option<Mat>,
    back: bool,
) {
    let (frame, seat) = (frame.unwrap_or(M.body), seat.unwrap_or(M.fabric));
    let s = if back { -1 } else { 1 };
    let sf = s as f64;
    soft!(
        f,
        format!("{name}_seat"),
        0.44,
        0.1,
        0.42,
        (x, 0.392, z),
        seat,
        round = 0.04,
        puff = 0.012,
        flat = 2
    );
    fextrude!(
        f,
        format!("{name}_back"),
        &smooth_outline(
            &[
                (-0.21, 0.0),
                (0.21, 0.0),
                (0.21, 0.36),
                (0.12, 0.46),
                (-0.12, 0.46),
                (-0.21, 0.36)
            ],
            2,
            true
        ),
        0.08,
        (x, 0.48, z - sf * 0.177),
        seat,
        rot = (-8 * s, 0, 0),
        bevel = 0.03,
        bevel_segments = 2
    );
    for dx in [-0.18, 0.18] {
        for dz in [0.16, -0.17] {
            fsweep!(
                f,
                format!("{name}_leg{dx}{dz}"),
                &[
                    (x + dx, 0.36, z + sf * dz),
                    (x + dx * 1.08, 0.0, z + sf * dz * 1.1)
                ],
                0.015,
                frame,
                verts = 8,
                scale = [1.0, 0.6]
            );
        }
    }
}
