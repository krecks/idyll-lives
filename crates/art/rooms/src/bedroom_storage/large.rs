//! BR3: the dressers and chests; each opens while a resident picks an outfit.

use super::common::{DRAWER_SECONDS, F, M, PULL, Style, WALL, drawer, folded, front, gap, top};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{P3, PI, pow, sin};
use artkit::parts::{aside, join, new_parts};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fextrude, revolve, slide, soft};
use artkit::{kit, settle};

// --- BR3: the dresser: oak, two small drawers over two wide ones; the middle (wide) one slides out -------
/// Folded clothes in a drawer: stacks across its width, bottom at `y`.
fn drawer_clothes(x: f64, y: f64, z: f64, w: f64, d: f64) {
    let n = 2.max((w / 0.3) as usize);
    let step = (w - 0.1) / n as f64;
    for i in 0..n {
        folded(
            &format!("folded{i}_"),
            x - (w - 0.1) / 2.0 + step * (i as f64 + 0.5),
            y,
            z,
            step - 0.03,
            d,
            2 + (i % 2),
            0.035,
        );
    }
}

/// A bracket foot at the front corner on the side of `x` (its outer edge at x): an ogee-cut plate on the front and
/// one down the side, mitred round the corner.
fn bracket(name: &str, x: f64, front_z: f64, material: Mat) {
    let (h, w) = (0.09, 0.14);
    let s = if x > 0.0 { 1.0 } else { -1.0 };
    let curve = [
        (w - 0.012, h - 0.004),
        (w - 0.035, h - 0.022),
        (w - 0.06, h - 0.04),
        (w - 0.085, h - 0.05),
        (w - 0.1, h - 0.062),
        (w - 0.11, 0.0),
    ];
    let mut out = vec![(0.0, 0.0), (0.0, h)];
    out.extend(curve);
    let mut front_outline: Vec<(f64, f64)> = out.iter().map(|&(u, v)| (x - s * u, v)).collect();
    if s < 0.0 {
        front_outline.reverse();
    }
    fextrude!(
        F,
        format!("{name}_front"),
        &front_outline,
        0.02,
        (0, 0, front_z),
        material,
        bevel = 0.003,
        bevel_segments = 1
    );
    let mut side: Vec<(f64, f64)> = out.iter().map(|&(u, v)| (front_z - 0.01 - u, v)).collect();
    side.reverse();
    fextrude!(
        F,
        format!("{name}_side"),
        &side,
        0.02,
        (x - s * 0.01, 0, 0),
        material,
        plane = Facing::Side,
        bevel = 0.003,
        bevel_segments = 1
    );
}

/// The dresser.
pub fn dresser() -> Vec<Obj> {
    let front_z = 0.07;
    let still = new_parts(|| {
        // bracket feet at the front corners, block feet at the back, a shaped apron between the front ones
        for sx in [-1, 1] {
            bracket(&format!("foot{sx}"), sx as f64 * 0.7, front_z, M.oak);
            fbox!(
                F,
                format!("foot_back{sx}"),
                0.05,
                0.06,
                0.05,
                (sx as f64 * 0.665, 0.03, WALL + 0.03),
                M.oak,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        let edge: Vec<(f64, f64)> = (0..25)
            .map(|i| {
                (
                    0.56 - 1.12 * i as f64 / 24.0,
                    0.055 - 0.02 * pow(sin(PI * i as f64 / 24.0), 4.0),
                )
            })
            .collect();
        let mut apron = vec![(0.56, 0.09), (-0.56, 0.09)];
        apron.extend(edge.iter().rev());
        fextrude!(
            F,
            "apron",
            &apron,
            0.02,
            (0, 0, front_z - 0.01),
            M.oak,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            F,
            "plinth",
            1.4,
            0.03,
            0.48,
            (0, 0.075, (WALL + front_z) / 2.0),
            M.oak,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "case",
            1.4,
            0.77,
            0.5,
            (0, 0.475, (WALL + front_z) / 2.0),
            M.oak,
            bevel = 0.008,
            bevel_segments = 1
        );
        top("top", 1.44, 0.03, 0.52, 0.89, -0.17, M.oak, 0.012);
        for s in [-1, 1] {
            let sf = s as f64;
            front(
                &format!("drawer_top{s}"),
                0.66,
                0.2,
                sf * 0.35,
                0.72,
                front_z + 0.01,
                M.oak,
                Style::Plain,
                None,
            );
            kit::pull!(
                F,
                &format!("pull_top{s}"),
                sf * 0.35,
                0.75,
                front_z + 0.02,
                0.1,
                M.brass,
                out = 0.014
            );
            kit::pull!(
                F,
                &format!("pull_low{s}"),
                sf * 0.35,
                0.27,
                front_z + 0.02,
                0.1,
                M.brass,
                out = 0.014
            );
        }
        front(
            "drawer_low",
            1.36,
            0.2,
            0,
            0.24,
            front_z + 0.01,
            M.oak,
            Style::Plain,
            None,
        );
        gap("drawer", 1.36, 0.2, 0, 0.48, front_z, M.ebony);
    });
    join(&still, "dresser", Some(P3::ZERO));
    // what stands on the top: a framed picture on its easel, a turned vase and a box with a lid. Drawn with the
    // dresser, not part of its size (the chest of drawers is 0.89 m tall; the picture stands 0.46 m above it).
    let carried = new_parts(|| {
        revolve!(
            "vase",
            &[
                (0.0, 0.0),
                (0.042, 0.0),
                (0.046, 0.006),
                (0.06, 0.06),
                (0.05, 0.15),
                (0.027, 0.195),
                (0.026, 0.21),
                (0.034, 0.22),
                (0.022, 0.222),
                (0.02, 0.17),
                (0.0, 0.17)
            ],
            at = F.at(-0.45, 0.89, -0.2),
            material = M.vase,
            segments = 18
        );
        soft!(
            F,
            "jewellery_box",
            0.22,
            0.055,
            0.16,
            (-0.12, 0.9175, -0.16),
            M.pull,
            round = 0.008,
            round_segments = 1
        );
        soft!(
            F,
            "jewellery_lid",
            0.226,
            0.018,
            0.166,
            (-0.12, 0.951, -0.16),
            M.pull,
            round = 0.007,
            round_segments = 1
        );
        fbox!(
            F,
            "jewellery_clasp",
            0.024,
            0.02,
            0.004,
            (-0.12, 0.94, -0.078),
            M.brass,
            bevel = 0.0015,
            bevel_segments = 1
        );
        fextrude!(
            F,
            "picture_frame",
            &rounded_rect(0.36, 0.46, 0.004, 1),
            0.03,
            (0.32, 1.12, -0.3),
            M.frame,
            holes = [rounded_rect(0.27, 0.37, 0.004, 1)],
            bevel = 0.006,
            bevel_segments = 2,
            rot = (-6, 0, 0)
        );
        fbox!(
            F,
            "picture",
            0.268,
            0.368,
            0.006,
            (0.32, 1.12, -0.29),
            M.print1,
            rot = (-6, 0, 0)
        );
        fbox!(
            F,
            "picture_easel",
            0.04,
            0.3,
            0.01,
            (0.32, 1.0, -0.36),
            M.frame,
            rot = (18, 0, 0)
        );
    });
    aside(&carried, "onTop");
    let d = drawer(
        "drawer",
        1.36,
        0.2,
        0,
        0.48,
        front_z,
        0.44,
        M.oak,
        &|| {
            for s in [-1, 1] {
                kit::pull!(
                    F,
                    &format!("pull_mid{s}"),
                    s as f64 * 0.35,
                    0.51,
                    front_z + 0.02,
                    0.1,
                    M.brass,
                    out = 0.014
                );
            }
        },
        Some(&|x, y, z| drawer_clothes(x, y, z, 1.2, 0.36)),
        Style::Plain,
    );
    let objs = settle!(centre = false);
    slide!(d, "drawer_open", (0, -PULL, 0), seconds = DRAWER_SECONDS);
    objs
}

/// A lidded storage box standing at game `at` (its bottom), a label on its front.
pub(super) fn storage_box(name: &str, w: f64, h: f64, d: f64, at: (f64, f64, f64), material: Mat) {
    let (x, y, z) = at;
    soft!(
        F,
        name,
        w,
        h - 0.02,
        d,
        (x, y + (h - 0.02) / 2.0, z),
        material,
        round = 0.008,
        round_segments = 1
    );
    soft!(
        F,
        format!("{name}_lid"),
        w + 0.008,
        0.028,
        d + 0.008,
        (x, y + h - 0.014, z),
        material,
        round = 0.008,
        round_segments = 1
    );
    fbox!(
        F,
        format!("{name}_label"),
        w * 0.35,
        0.04,
        0.002,
        (x, y + h * 0.45, z + d / 2.0 + 0.001),
        M.paper
    );
}

// --- the tall chest of drawers: linen-white, five drawers with brass bars, on a dark plinth --------------
// 0.51 m deep (it was 0.66, as deep as its tile: M-bedroom-02).
const CHEST_FRONT: f64 = 0.04;

/// The tall chest of drawers.
pub fn chest_of_drawers() -> Vec<Obj> {
    let front_z = CHEST_FRONT;
    let still = new_parts(|| {
        fbox!(
            F,
            "plinth",
            0.82,
            0.06,
            front_z - WALL - 0.04,
            (0, 0.03, (WALL + front_z) / 2.0 - 0.01),
            M.plinth,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "case",
            0.86,
            1.07,
            front_z - WALL,
            (0, 0.595, (WALL + front_z) / 2.0),
            M.linen,
            bevel = 0.008,
            bevel_segments = 1
        );
        top(
            "top",
            0.88,
            0.03,
            front_z + 0.015 - WALL,
            1.16,
            (WALL + front_z + 0.015) / 2.0,
            M.linen,
            0.012,
        );
        for (i, y) in [0.179, 0.387, 0.803, 1.011].into_iter().enumerate() {
            front(
                &format!("drawer{i}"),
                0.815,
                0.194,
                0,
                y,
                front_z + 0.01,
                M.linen,
                Style::Shaker,
                Some(0.04),
            );
            kit::pull!(
                F,
                &format!("pull{i}"),
                0.0,
                y + 0.031,
                front_z + 0.02,
                0.14,
                M.brass,
                r = 0.005,
                out = 0.016
            );
        }
        gap("drawer", 0.815, 0.194, 0, 0.595, front_z, M.plinth);
    });
    join(&still, "chestOfDrawers", Some(P3::ZERO));
    let d = drawer(
        "drawer",
        0.815,
        0.194,
        0,
        0.595,
        front_z,
        0.42,
        M.linen,
        &|| {
            kit::pull!(
                F,
                "pull_mid",
                0.0,
                0.626,
                front_z + 0.02,
                0.14,
                M.brass,
                r = 0.005,
                out = 0.016
            );
        },
        Some(&|x, y, z| drawer_clothes(x, y, z, 0.72, 0.34)),
        Style::Shaker,
    );
    let objs = settle!(centre = false);
    slide!(d, "drawer_open", (0, -PULL, 0), seconds = DRAWER_SECONDS);
    objs
}

// --- the low dresser: oak, nine drawers in three columns on tapered legs; the middle one slides out ------
/// The low dresser.
pub fn dresser_low() -> Vec<Obj> {
    let front_z = 0.165;
    let (ys, xs) = ([0.2217, 0.435, 0.6483], [-0.59, 0.0, 0.59]);
    let still = new_parts(|| {
        for sx in [-1, 1] {
            let sf = sx as f64;
            for z in [-0.39, 0.12] {
                kit::tube!(
                    F,
                    format!("leg{sx}{}", float_repr(z)),
                    &[
                        (sf * 0.84, 0.1, z),
                        (sf * 0.852, 0.002, z + 0.012_f64.copysign(z + 0.1))
                    ],
                    0.022,
                    M.oakleg,
                    verts = 10,
                    round = 1,
                    scale = [1.0, 0.7]
                );
            }
        }
        soft!(
            F,
            "case",
            1.8,
            0.67,
            front_z - WALL,
            (0, 0.435, (WALL + front_z) / 2.0),
            M.oak,
            round = 0.014,
            round_segments = 2
        );
        top(
            "top",
            1.82,
            0.03,
            0.61,
            0.8,
            (WALL + 0.18) / 2.0,
            M.oak,
            0.012,
        );
        fbox!(
            F,
            "reveal",
            1.78,
            0.008,
            0.58,
            (0, 0.768, (WALL + 0.18) / 2.0),
            M.ebony
        );
        for x in xs {
            for y in ys {
                if (x, y) == (0.0, 0.435) {
                    continue;
                }
                front(
                    &format!("drawer{}{}", float_repr(x), float_repr(y)),
                    0.575,
                    0.1993,
                    x,
                    y,
                    front_z + 0.01,
                    M.oak,
                    Style::Plain,
                    None,
                );
                kit::pull!(
                    F,
                    &format!("pull{}{}", float_repr(x), float_repr(y)),
                    x,
                    y + 0.032,
                    front_z + 0.02,
                    0.14,
                    M.iron,
                    r = 0.005,
                    out = 0.016
                );
            }
        }
        gap("drawer", 0.575, 0.1993, 0, 0.435, front_z, M.ebony);
    });
    join(&still, "dresserLow", Some(P3::ZERO));
    let d = drawer(
        "drawer",
        0.575,
        0.1993,
        0,
        0.435,
        front_z,
        0.5,
        M.oak,
        &|| {
            kit::pull!(
                F,
                "pull_mid",
                0.0,
                0.467,
                front_z + 0.02,
                0.14,
                M.iron,
                r = 0.005,
                out = 0.016
            );
        },
        Some(&|x, y, z| drawer_clothes(x, y, z, 0.5, 0.4)),
        Style::Plain,
    );
    let objs = settle!(centre = false);
    slide!(d, "drawer_open", (0, -PULL, 0), seconds = DRAWER_SECONDS);
    objs
}
