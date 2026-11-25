//! BR3: the dressers, chests and wardrobes; each opens while a resident picks an outfit.

use super::common::{
    DOOR_SECONDS, DRAWER_SECONDS, F, M, PULL, Style, WALL, drawer, folded, front, gap, hanging,
    shell, top,
};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{P3, PI, pow, sin};
use artkit::motion::Ease;
use artkit::parts::{aside, hinged, join, new_parts};
use artkit::place::{Facing, N};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, loft, revolve, slide, soft, swing};
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

// --- the wardrobe: a white carcass, two birch doors with brass bars, a rail of shirts and shelves --------
/// A wardrobe's inside: a rail of clothes on hangers (the left half when `split`, with shelves of folded clothes
/// on the right) under a shelf.
#[allow(clippy::too_many_arguments)]
fn wardrobe_inside(w: f64, y0: f64, y1: f64, back: f64, front_z: f64, rail_y: f64, split: bool) {
    let (d, zc) = (front_z - back, (back + front_z) / 2.0);
    let hang_x1 = if split { -0.02 } else { w / 2.0 - 0.08 };
    let width = if split { w / 2.0 } else { w };
    let x = if split { N::F(-w / 4.0) } else { N::I(0) };
    fbox!(
        F,
        "hat_shelf",
        width - 0.04,
        0.02,
        d - 0.04,
        (x, rail_y + 0.12, zc - 0.01),
        M.white
    );
    fcyl!(
        F,
        "rail",
        0.012,
        width - 0.06,
        (x, rail_y, zc),
        M.chrome,
        verts = 10,
        rot = (0, 90, 0)
    );
    let n = if split { 5 } else { 3.max((w / 0.16) as usize) };
    hanging(
        "shirt",
        -w / 2.0 + 0.1,
        hang_x1 - 0.08,
        rail_y,
        zc,
        n,
        0.42_f64.min(d - 0.1),
        0.78,
    );
    // a box on the hat shelf, under the carcass's top and inside its side (it came through both)
    let bw = 0.3_f64.min(w / 2.0 - 0.06);
    storage_box(
        "folded_box",
        bw,
        0.13,
        0.3,
        (-w / 4.0, rail_y + 0.13, zc),
        M.box_,
    );
    if split {
        fbox!(
            F,
            "divider",
            0.02,
            y1 - y0 - 0.04,
            d - 0.02,
            (0, (y0 + y1) / 2.0, zc - 0.01),
            M.white
        );
        for (i, y) in [y0 + 0.45, y0 + 0.9, y0 + 1.35].into_iter().enumerate() {
            fbox!(
                F,
                format!("shelf{i}"),
                w / 2.0 - 0.04,
                0.02,
                d - 0.04,
                (w / 4.0, y, zc - 0.01),
                M.white
            );
            folded(
                &format!("stack{i}_"),
                w / 4.0 - 0.18,
                y + 0.01,
                zc,
                0.3,
                0.32,
                3,
                0.035,
            );
            folded(
                &format!("stackb{i}_"),
                w / 4.0 + 0.18,
                y + 0.01,
                zc,
                0.3,
                0.32,
                2 + i % 2,
                0.035,
            );
        }
    }
}

/// A lidded storage box standing at game `at` (its bottom), a label on its front.
fn storage_box(name: &str, w: f64, h: f64, d: f64, at: (f64, f64, f64), material: Mat) {
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

/// The knuckles of a door's hinges on its hinge line (x, z), on the still carcass.
fn hinges(name: &str, x: f64, ys: &[f64], z: f64) {
    for &y in ys {
        fcyl!(
            F,
            format!("{name}{}", float_repr(y)),
            0.006,
            0.06,
            (x, y, z),
            M.chrome,
            verts = 8
        );
    }
}

/// The wardrobe.
pub fn wardrobe() -> Vec<Obj> {
    let (back, front_z) = (WALL, 0.15);
    let still = new_parts(|| {
        fbox!(
            F,
            "plinth",
            1.80,
            0.06,
            0.54,
            (0, 0.03, (back + front_z) / 2.0 - 0.01),
            M.plinth,
            bevel = 0.004,
            bevel_segments = 1
        );
        shell(1.84, 0.06, 2.11, back, front_z, M.white, 0.025, "case");
        wardrobe_inside(1.84, 0.06, 2.11, back, front_z, 1.8, true);
        storage_box("box_big", 0.4, 0.3, 0.32, (-0.5, 2.11, -0.18), M.box_);
        storage_box("box_small", 0.32, 0.22, 0.3, (-0.08, 2.11, -0.18), M.box2);
        for s in [-1, 1] {
            hinges(
                &format!("hinge{s}_"),
                s as f64 * 0.913,
                &[0.3, 1.085, 1.86],
                front_z + 0.01,
            );
        }
    });
    join(&still, "wardrobe", Some(P3::ZERO));
    let mut doors = Vec::new();
    for s in [-1, 1] {
        let sf = s as f64;
        let parts = new_parts(|| {
            front(
                &format!("door{s}"),
                0.905,
                2.04,
                sf * 0.4575,
                1.085,
                front_z + 0.01,
                M.birch,
                Style::Shaker,
                Some(0.09),
            );
            kit::pull!(
                F,
                &format!("handle{s}"),
                sf * 0.06,
                1.1,
                front_z + 0.02,
                0.36,
                M.brass,
                r = 0.008,
                out = 0.018,
                upright = true
            );
        });
        let name = if s < 0 { "door_left" } else { "door_right" };
        doors.push(hinged(
            join(&parts, name, None),
            F.at(sf * 0.91, 1.085, front_z + 0.01),
        ));
    }
    let objs = settle!(centre = false);
    swing!(
        doors[0],
        "doors_open",
        100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        doors[1],
        "doors_open",
        -100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    objs
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

// --- the single wardrobe: white, one door hinged on the left with a brass bar, a rail of shirts ----------
// 0.60 m wide, its door 0.58 (it was 0.90, with a door as wide as a room's: M-bedroom-03).
const WS_W: f64 = 0.6;

/// The single wardrobe.
pub fn wardrobe_single() -> Vec<Obj> {
    let (back, front_z) = (WALL, 0.14);
    let w = WS_W;
    let still = new_parts(|| {
        fbox!(
            F,
            "plinth",
            w - 0.04,
            0.05,
            0.53,
            (0, 0.025, (back + front_z) / 2.0 - 0.01),
            M.plinth,
            bevel = 0.004,
            bevel_segments = 1
        );
        shell(w, 0.05, 2.04, back, front_z, M.white, 0.022, "case");
        wardrobe_inside(w, 0.05, 2.04, back, front_z, 1.74, false);
        storage_box("box_top", 0.3, 0.24, 0.3, (-0.1, 2.04, -0.2), M.box_);
        hinges(
            "hinge",
            -(w - 0.02) / 2.0,
            &[0.3, 1.045, 1.79],
            front_z + 0.01,
        );
    });
    join(&still, "wardrobeSingle", Some(P3::ZERO));
    let parts = new_parts(|| {
        front(
            "door",
            w - 0.02,
            1.97,
            0,
            1.045,
            front_z + 0.01,
            M.white,
            Style::Shaker,
            Some(0.08),
        );
        kit::pull!(
            F,
            "handle",
            w / 2.0 - 0.06,
            1.1,
            front_z + 0.02,
            0.44,
            M.brass,
            r = 0.008,
            out = 0.018,
            upright = true
        );
    });
    let door = hinged(
        join(&parts, "door", None),
        F.at(-(w - 0.02) / 2.0, 1.045, front_z + 0.01),
    );
    let objs = settle!(centre = false);
    swing!(
        door,
        "door_open",
        100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    objs
}

// --- the armoire: mahogany, a cornice, raised-panel doors with brass knobs, on bun feet -------------------
/// A crown moulding round the front and sides at height `y` (its bottom), its path `x` either side and at game depth
/// `front`: a cove rising and swelling out `out` to a flat top, its top's corners eased and its ends (on the wall's
/// face) chamfered. Its back edge reaches only 1 cm in from the path, inside the path's 1.2 cm corners: reaching
/// farther, it folded back on itself round each front corner (faces turned inside out).
#[allow(clippy::too_many_arguments)]
fn crown(name: &str, x: f64, front_z: f64, back: f64, y: f64, material: Mat, h: f64, out: f64) {
    // the cove as drawn for the 6.9 cm crown that reaches 5 cm, scaled to this one
    let (ku, kv) = (h / 0.069, out / 0.05);
    let mut prof: Vec<(f64, f64)> = [
        (0.0, 0.0),
        (0.0, -0.006),
        (0.012, -0.012),
        (0.024, -0.02),
        (0.036, -0.032),
        (0.046, -0.044),
        (0.054, -0.05),
    ]
    .iter()
    .map(|&(u, v)| (u * ku, v * kv))
    .collect();
    prof.extend([(h, -out), (h, 0.01), (0.0, 0.01)]);
    // swept about its middle (`c`), so the rings at its ends can shrink into it: a chamfer
    let c = (h / 2.0, (0.01 - out) / 2.0);
    let centred: Vec<(f64, f64)> = prof.iter().map(|&(u, v)| (v - c.1, u - c.0)).collect();
    // (eased with height as its v: what it leaves sharp is its bottom, on the cornice)
    let prof: Vec<(f64, f64)> = crate::common::eased(&centred, 0.004)
        .iter()
        .map(|&(v, u)| (u, v))
        .collect();
    let (x, front_z, y) = (x - c.1, front_z - c.1, y + c.0); // the path through its middle, out by -c.1
    let (z0, z1) = (back, back + 0.004);
    let mut path = vec![(-x, y, z0)];
    path.extend(kit::fillet(
        &[(-x, y, z1), (-x, y, front_z), (x, y, front_z), (x, y, z1)],
        0.012 - c.1,
        2,
    ));
    path.push((x, y, z0));
    let end = 1.0 - 0.004 / (h / 2.0).min((0.01 + out) / 2.0); // each end's ring 4 mm in all round
    fsweep!(
        F,
        name,
        &path,
        0,
        material,
        profile = prof,
        up = (0, 0, 1),
        angle = 30,
        scale = artkit::forms::by(move |w| if w <= 0.0 || w >= 1.0 { end } else { 1.0 })
    );
}

/// The armoire.
pub fn armoire() -> Vec<Obj> {
    let (back, front_z) = (WALL, 0.115);
    let still = new_parts(|| {
        for sx in [-1, 1] {
            for z in [-0.38, 0.08] {
                revolve!(
                    format!("foot{sx}{}", float_repr(z)),
                    &[
                        (0.0, 0.0),
                        (0.028, 0.0),
                        (0.044, 0.018),
                        (0.048, 0.045),
                        (0.04, 0.07),
                        (0.026, 0.085),
                        (0.026, 0.12),
                        (0.0, 0.12)
                    ],
                    at = F.at(sx as f64 * 0.8, 0, z),
                    material = M.mahogany2,
                    segments = 14
                );
            }
        }
        soft!(
            F,
            "base",
            1.74,
            0.1,
            0.59,
            (0, 0.17, (back + 0.16) / 2.0),
            M.mahogany2,
            round = 0.012,
            round_segments = 1
        );
        crown(
            "base_moulding",
            0.85,
            front_z + 0.005,
            back,
            0.22,
            M.mahogany2,
            0.03,
            0.018,
        );
        shell(1.7, 0.22, 1.97, back, front_z, M.mahogany, 0.03, "case");
        fbox!(
            F,
            "cornice",
            1.76,
            0.07,
            0.57,
            (0, 2.005, (back + 0.14) / 2.0),
            M.mahogany,
            bevel = 0.006,
            bevel_segments = 1
        );
        crown("crown", 0.88, 0.148, back, 2.04, M.mahogany2, 0.069, 0.05);
        // the face frame round the doors: rails above and below, stiles with a bead
        fbox!(
            F,
            "rail_top",
            1.7,
            0.07,
            0.02,
            (0, 1.935, front_z - 0.01),
            M.mahogany,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "rail_low",
            1.7,
            0.04,
            0.02,
            (0, 0.23, front_z - 0.01),
            M.mahogany,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("stile{s}"),
                0.04,
                1.69,
                0.02,
                (sf * 0.83, 1.09, front_z - 0.006),
                M.mahogany,
                bevel = 0.004,
                bevel_segments = 1
            );
            fcyl!(
                F,
                format!("stile_bead{s}"),
                0.006,
                1.6,
                (sf * 0.836, 1.09, front_z + 0.004),
                M.mahogany,
                verts = 6
            );
        }
        // inside: a shelf, a rail of clothes on hangers and folded linen above
        let zc = (back + front_z) / 2.0;
        fbox!(F, "shelf", 1.64, 0.02, 0.5, (0, 1.7, zc - 0.01), M.mahogany);
        fcyl!(
            F,
            "rail",
            0.012,
            1.6,
            (0, 1.6, zc),
            M.chrome,
            verts = 10,
            rot = (0, 90, 0)
        );
        hanging("shirt", -0.72, 0.72, 1.6, zc, 9, 0.4, 0.8);
        for (i, x) in [-0.5, 0.0, 0.5].into_iter().enumerate() {
            folded(&format!("linen{i}_"), x, 1.71, zc, 0.36, 0.34, 3, 0.035);
        }
    });
    join(&still, "armoire", Some(P3::ZERO));
    let mut doors = Vec::new();
    for s in [-1, 1] {
        let sf = s as f64;
        let parts = new_parts(|| {
            let x = sf * 0.4125;
            let holes: Vec<Vec<(f64, f64)>> = [0.34, -0.36]
                .iter()
                .map(|&dy| {
                    rounded_rect(0.58, 0.56, 0.004, 1)
                        .iter()
                        .map(|&(u, v)| (u, v + dy))
                        .collect()
                })
                .collect();
            fextrude!(
                F,
                format!("door{s}"),
                &rounded_rect(0.815, 1.66, 0.004, 1),
                0.025,
                (x, 1.08, front_z + 0.0125),
                M.mahogany,
                holes = holes,
                bevel = 0.004,
                bevel_segments = 1
            );
            for dy in [0.34, -0.36] {
                let b = 0.07;
                let secs: Vec<Vec<P3>> = [
                    (0.289, 0.279, front_z + 0.004),
                    (0.289, 0.279, front_z + 0.017),
                    (0.289 - b, 0.279 - b, front_z + 0.03),
                ]
                .iter()
                .map(|&(hw, hh, d)| {
                    [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
                        .iter()
                        .map(|&(u, v)| F.at(x + u * hw, 1.08 + dy + v * hh, d))
                        .collect()
                })
                .collect();
                loft!(
                    format!("panel{s}{}", float_repr(dy)),
                    &secs,
                    material = M.walnut,
                    smooth = false
                );
            }
            kit::knob!(
                F,
                format!("knob{s}"),
                sf * 0.09,
                1.08,
                front_z + 0.025,
                0.024,
                M.brass
            );
            if s > 0 {
                soft!(
                    F,
                    "escutcheon",
                    0.026,
                    0.05,
                    0.004,
                    (0.045, 1.0, front_z + 0.027),
                    M.brass,
                    round = 0.0015,
                    round_segments = 1
                );
                fbox!(
                    F,
                    "keyhole",
                    0.005,
                    0.016,
                    0.003,
                    (0.045, 1.0, front_z + 0.029),
                    M.mahogany2
                );
            }
        });
        let name = if s < 0 { "door_left" } else { "door_right" };
        doors.push(hinged(
            join(&parts, name, None),
            F.at(sf * 0.82, 1.08, front_z + 0.0125),
        ));
    }
    let objs = settle!(centre = false);
    swing!(
        doors[0],
        "doors_open",
        100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        doors[1],
        "doors_open",
        -100.0,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    objs
}
