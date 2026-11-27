//! The still wall things (K5): the cooker hood, the spice rack, the pot rail and the wall shelf, each on the wall face
//! at its real height.

use super::M;
use super::storage::{WALL, jar, lathe_at};
use super::turned::{canister, lathe, plates};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, TAU, cos, radians, sin};
use artkit::parts::{join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, settle, torus};

/// A rectangle in plan at height `y`, game corners (x0, z0) to (x1, z1), as Blender points (for a loft).
fn rect_ring(f: &GameFrame, x0: f64, x1: f64, z0: f64, z1: f64, y: f64) -> Vec<P3> {
    // its back (on the wall) 4 mm lower, 4 mm behind the rest: the slope's edge along the wall eased at 45°
    let c = 0.004;
    [
        (x0, y - c, z0),
        (x1, y - c, z0),
        (x1, y, z0 + c),
        (x1, y, z1),
        (x0, y, z1),
        (x0, y, z0 + c),
    ]
    .into_iter()
    .map(|(x, y, z)| f.at(x, y, z))
    .collect()
}

// --- the cooker hood: 1x1, a canopy and a chimney with a lamp and a filter ------------------------------
pub fn range_hood() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // one flue tapering from the canopy's neck to a narrower top, its corners rounded, its back on the wall
        let ring = |w: f64, d: f64, y: f64, back: f64| -> Vec<P3> {
            rounded_rect(w, d, 0.03, 2)
                .into_iter()
                .map(|(u, v)| f.at(u, y, back + d / 2.0 + v))
                .collect()
        };
        loft!(
            "flue",
            &[
                ring(0.31, 0.305, 1.67, -0.43),
                ring(0.27, 0.27, 1.96, -0.43),
                ring(0.23, 0.23, 2.235, -0.43), // its top edge eased
                ring(0.206, 0.206, 2.25, -0.418)
            ],
            material = M.steel,
            angle = 20
        );
        // the canopy: an upright band, then sloping up to the chimney, its back on the wall
        fbox!(
            f,
            "canopy",
            0.9,
            0.05,
            0.5,
            (0, 1.575, -0.18),
            M.steel,
            bevel = 0.006,
            bevel_segments = 2
        );
        loft!(
            "canopy_slope",
            &[
                rect_ring(f, -0.45, 0.45, -0.43, 0.07, 1.599),
                rect_ring(f, -0.45, 0.45, -0.43, 0.07, 1.605),
                rect_ring(f, -0.447, 0.447, -0.43, 0.067, 1.608), // the rim eased onto the slope
                rect_ring(f, -0.155, 0.155, -0.43, -0.125, 1.675)
            ],
            material = M.steel,
            angle = 20
        );
        // the trim along its front, its buttons and a little display
        fbox!(
            f,
            "trim",
            0.9,
            0.025,
            0.02,
            (0, 1.58, 0.075),
            M.trim,
            bevel = 0.004
        );
        for i in 0..4 {
            fcyl!(
                f,
                format!("key{i}"),
                0.0065,
                0.004,
                (0.2 + 0.04 * i as f64, 1.58, 0.083),
                M.steel,
                rot = (90, 0, 0),
                verts = 10
            );
        }
        fbox!(
            f,
            "display",
            0.05,
            0.012,
            0.004,
            (0.13, 1.58, 0.083),
            M.bulb
        );
        // under it: the grease filter with its slots, two round lamps
        let slots: Vec<Vec<(f64, f64)>> = (0..10)
            .map(|i| {
                rounded_rect(0.05, 0.36, 0.008, 1)
                    .into_iter()
                    .map(|(u, v)| (u + (i as f64 - 4.5) * 0.08, v))
                    .collect()
            })
            .collect();
        fextrude!(
            f,
            "filter",
            &rounded_rect(0.84, 0.44, 0.01, 1),
            0.01,
            (0, 1.551, -0.18),
            M.dark,
            holes = slots,
            plane = Facing::Top
        );
        fbox!(
            f,
            "filter_back",
            0.82,
            0.004,
            0.42,
            (0, 1.556, -0.18),
            M.trim
        );
        for s in [-1, 1] {
            fcyl!(
                f,
                format!("lamp{s}"),
                0.028,
                0.008,
                (s as f64 * 0.3, 1.542, -0.0),
                M.bulb,
                verts = 12
            );
            fcyl!(
                f,
                format!("lamp_ring{s}"),
                0.034,
                0.004,
                (s as f64 * 0.3, 1.548, -0.0),
                M.steel,
                verts = 12
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle!(centre = false, floor = false);
    meshes()
}

// --- the spice rack: 1x1 wall, three rails of little jars ---------------------------------------------
pub fn spice_rack() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let colours = [M.rust, M.teal, M.sage, M.cream, M.yellow, M.porcelain];
    let still = new_parts(|| {
        // the board: an arched crest with a hole to hang it by
        let mut crest = vec![(0.26, -0.22), (0.26, 0.17)];
        crest.extend((15..180).step_by(15).map(|a| {
            (
                0.26 * cos(radians(a as f64)),
                0.17 + 0.05 * sin(radians(a as f64)),
            )
        }));
        crest.extend([(-0.26, 0.17), (-0.26, -0.22)]);
        let hole: Vec<(f64, f64)> = (0..10)
            .map(|i| {
                (
                    0.011 * cos(TAU * i as f64 / 10.0),
                    0.19 + 0.011 * sin(TAU * i as f64 / 10.0),
                )
            })
            .collect();
        fextrude!(
            f,
            "board",
            &crest,
            0.02,
            (0, 1.52, WALL + 0.01),
            M.oak,
            holes = vec![hole],
            bevel = 0.003,
            bevel_segments = 1
        );
        for (i, y) in [1.33, 1.47, 1.61].into_iter().enumerate() {
            fbox!(
                f,
                format!("shelf{i}"),
                0.5,
                0.015,
                0.09,
                (0, y, WALL + 0.055),
                M.oak,
                bevel = 0.003
            );
            // a brass rail on two posts along the shelf's front, in one piece
            kit::tube!(
                f,
                format!("rail{i}"),
                &[
                    (-0.244, y, WALL + 0.1),
                    (-0.244, y + 0.05, WALL + 0.1),
                    (-0.236, y + 0.058, WALL + 0.1),
                    (0.236, y + 0.058, WALL + 0.1),
                    (0.244, y + 0.05, WALL + 0.1),
                    (0.244, y, WALL + 0.1),
                ],
                0.005,
                M.brass,
                verts = 6,
                round = 1
            );
            for j in 0..6 {
                jar(
                    f,
                    &format!("jar{i}{j}"),
                    -0.2 + j as f64 * 0.08,
                    y + 0.0075,
                    WALL + 0.055,
                    colours[(i + j) % 6],
                    0.075,
                    0.024,
                    true,
                    Some(M.dark),
                    8,
                );
            }
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle!(centre = false, floor = false);
    meshes()
}

/// An S-hook over a rail whose middle is at (x, y, z): over the rail, down its front, curling up at the bottom.
fn s_hook(f: &GameFrame, name: &str, x: f64, y: f64, z: f64) {
    let pts = [
        (-0.019, -0.01),
        (-0.021, 0.012),
        (0.0, 0.024),
        (0.021, 0.012),
        (0.018, -0.02),
        (0.006, -0.045),
        (-0.006, -0.062),
        (0.002, -0.076),
        (0.016, -0.071),
    ];
    let path: Vec<(f64, f64, f64)> = pts.iter().map(|&(dz, dy)| (x, y + dy, z + dz)).collect();
    fsweep!(f, name, &path, 0.0035, M.metal, verts = 6, resolution = 2);
}

// --- the pot rail: 1x1 wall, a rail of hooks with pans, a pot and ladles -------------------------------
pub fn pot_rail() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (z, y) = (WALL + 0.08, 1.6);
    let still = new_parts(|| {
        // the rail with ball ends, on two arms from round wall plates
        kit::tube!(
            f,
            "rail",
            &[(-0.435, y, z), (0.435, y, z)],
            0.014,
            M.metal,
            verts = 12,
            round = 1
        );
        for s in [-1, 1] {
            fsphere!(
                f,
                format!("finial{s}"),
                0.02,
                (s as f64 * 0.435, y, z),
                M.metal,
                segments = 10,
                rings = 6
            );
            fcyl!(
                f,
                format!("plate{s}"),
                0.024,
                0.01,
                (s as f64 * 0.38, y, WALL + 0.005),
                M.metal,
                rot = (90, 0, 0),
                verts = 12
            );
            fsweep!(
                f,
                format!("arm{s}"),
                &[
                    (s as f64 * 0.38, y, WALL + 0.005),
                    (s as f64 * 0.38, y - 0.004, z - 0.03),
                    (s as f64 * 0.38, y, z)
                ],
                0.008,
                M.metal,
                verts = 8,
                resolution = 2
            );
        }
        for (x, kind) in [
            (-0.32, "pan"),
            (-0.12, "ladle"),
            (0.06, "pot"),
            (0.28, "pan"),
            (0.4, "ladle"),
        ] {
            let xs = float_repr(x);
            s_hook(f, &format!("hook{xs}"), x, y, z);
            let low = y - 0.075; // where the hook's curl holds what hangs from it
            if kind == "pan" {
                // a frying pan facing the front, its handle up to the hook
                lathe_at(
                    format!("pan{xs}"),
                    &[
                        (0.0, 0.0),
                        (0.1, 0.0),
                        (0.118, 0.008),
                        (0.13, 0.04),
                        (0.124, 0.04),
                        (0.11, 0.012),
                        (0.0, 0.006),
                    ],
                    f.at(x, 1.21, WALL + 0.04),
                    M.copper,
                    18,
                    (90, 0, 0),
                );
                fsweep!(
                    f,
                    format!("handle{xs}"),
                    &[
                        (x, 1.33, WALL + 0.06),
                        (x, 1.42, WALL + 0.07),
                        (x, low - 0.01, z)
                    ],
                    0.0,
                    M.metal,
                    profile = rounded_rect(0.022, 0.008, 0.003, 1),
                    up = (1, 0, 0),
                    resolution = 2
                );
                torus!(
                    format!("eye{xs}"),
                    0.009,
                    0.0035,
                    at = f.at(x, low - 0.002, z),
                    material = M.metal,
                    rot = (0, 90, 0),
                    major_segments = 8,
                    minor_segments = 4
                );
            } else if kind == "pot" {
                // a saucepan hung by its bail handle, its lid on
                lathe(
                    f,
                    format!("pot{xs}"),
                    &[
                        (0.0, 0.0),
                        (0.086, 0.0),
                        (0.096, 0.012),
                        (0.096, 0.14),
                        (0.1, 0.146),
                        (0.0, 0.146),
                    ],
                    x,
                    1.29,
                    WALL + 0.12,
                    M.copper,
                    16,
                );
                lathe(
                    f,
                    format!("pot_lid{xs}"),
                    &[
                        (0.0, 0.144),
                        (0.097, 0.144),
                        (0.06, 0.158),
                        (0.01, 0.16),
                        (0.014, 0.17),
                        (0.0, 0.172),
                    ],
                    x,
                    1.29,
                    WALL + 0.12,
                    M.copper,
                    16,
                );
                fsweep!(
                    f,
                    format!("bail{xs}"),
                    &[
                        (x - 0.1, 1.4, WALL + 0.12),
                        (x - 0.092, 1.47, WALL + 0.11),
                        (x - 0.03, low - 0.01, z),
                        (x + 0.03, low - 0.01, z),
                        (x + 0.092, 1.47, WALL + 0.11),
                        (x + 0.1, 1.4, WALL + 0.12)
                    ],
                    0.004,
                    M.metal,
                    verts = 6,
                    resolution = 2
                );
            } else {
                // a ladle: its bowl, its handle up to a curl round the hook
                lathe(
                    f,
                    format!("bowl{xs}"),
                    &[
                        (0.0, 0.0),
                        (0.03, 0.004),
                        (0.046, 0.025),
                        (0.05, 0.05),
                        (0.045, 0.05),
                        (0.03, 0.012),
                        (0.0, 0.009),
                    ],
                    x,
                    1.236,
                    WALL + 0.09,
                    M.steel,
                    12,
                );
                fsweep!(
                    f,
                    format!("handle{xs}"),
                    &[
                        (x, 1.286, WALL + 0.045),
                        (x, 1.33, WALL + 0.06),
                        (x, 1.45, WALL + 0.075),
                        (x, low - 0.012, z - 0.004),
                        (x, low + 0.004, z + 0.008)
                    ],
                    0.0,
                    M.steel,
                    profile = rounded_rect(0.014, 0.005, 0.002, 1),
                    up = (1, 0, 0),
                    resolution = 2
                );
            }
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle!(centre = false, floor = false);
    meshes()
}

/// A shelf bracket in profile: its back on the wall at `z`, its top under the shelf at `y`, `d` deep and `h` tall,
/// the brace curving from the wall out under the shelf.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn corbel(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, h: f64, d: f64, material: Mat) {
    let k = 0.025_f64.copysign(d); // a negative `d` reaches back (under an island's overhang)
    let mut outline = vec![(z, y), (z + d, y)];
    outline.extend((0..91).step_by(15).map(|a| {
        (
            z + d - (d - k) * sin(radians(a as f64)),
            y - 0.025 - (h - 0.045) * (1.0 - cos(radians(a as f64))),
        )
    }));
    outline.extend([(z + k, y - h + 0.01), (z, y - h)]);
    fextrude!(
        f,
        name,
        &outline,
        0.03,
        (x, 0, 0),
        material,
        plane = Facing::Side,
        bevel = 0.003,
        bevel_segments = 1
    );
}

// --- the wall shelf: 1x1 wall, two shelves of jars, plates, pots and a plant ---------------------------
pub fn kitchen_shelf_wall() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        for (i, y) in [1.4, 1.75].into_iter().enumerate() {
            fbox!(
                f,
                format!("shelf{i}"),
                0.9,
                0.03,
                0.24,
                (0, y, WALL + 0.12),
                M.oak,
                bevel = 0.006,
                bevel_segments = 2
            );
        }
        for s in [-1, 1] {
            corbel(
                f,
                &format!("bracket{s}"),
                s as f64 * 0.36,
                1.385,
                WALL,
                0.095,
                0.2,
                M.oakd,
            );
            corbel(
                f,
                &format!("bracket_top{s}"),
                s as f64 * 0.36,
                1.735,
                WALL,
                0.08,
                0.16,
                M.oakd,
            );
        }
        jar(
            f,
            "jar0",
            -0.32,
            1.415,
            WALL + 0.12,
            M.yellow,
            0.13,
            0.045,
            true,
            None,
            10,
        );
        jar(
            f,
            "jar1",
            -0.2,
            1.415,
            WALL + 0.12,
            M.porcelain,
            0.16,
            0.045,
            true,
            None,
            10,
        );
        jar(
            f,
            "jar2",
            -0.08,
            1.415,
            WALL + 0.12,
            M.rust,
            0.13,
            0.045,
            true,
            None,
            10,
        );
        plates(
            f,
            "plates",
            0.22,
            1.415,
            WALL + 0.12,
            0.11,
            4,
            M.porcelain,
            0.014,
            14,
        );
        canister(
            f,
            "pot0",
            -0.25,
            1.765,
            WALL + 0.12,
            0.05,
            0.11,
            M.teal,
            M.spoon,
            14,
        );
        kit::mug!(
            f,
            "pot1",
            -0.12,
            1.765,
            WALL + 0.12,
            0.04,
            0.09,
            M.cream,
            -30.0,
            segments = 12
        );
        // a trailing plant in a pot: a crown of leaves, two vines trailing over the shelf's front
        lathe(
            f,
            "plant_pot",
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.058, 0.085),
                (0.062, 0.088),
                (0.062, 0.1),
                (0.054, 0.1),
                (0.052, 0.09),
                (0.0, 0.09),
            ],
            0.25,
            1.765,
            WALL + 0.12,
            M.porcelain,
            14,
        );
        fcyl!(
            f,
            "soil",
            0.053,
            0.006,
            (0.25, 1.858, WALL + 0.12),
            M.soil,
            verts = 12
        );
        for (i, (dx, dy, dz, a, t)) in [
            (0.0, 0.19, 0.0, 0, 10),
            (0.05, 0.13, 0.03, 60, 35),
            (-0.05, 0.14, 0.01, 130, 35),
            (0.02, 0.12, -0.05, 200, 40),
            (-0.03, 0.1, 0.05, 280, 45),
            (0.07, 0.07, -0.02, 20, 60),
            (-0.07, 0.075, -0.03, 160, 55),
        ]
        .into_iter()
        .enumerate()
        {
            fsphere!(
                f,
                format!("leaf{i}"),
                0.045,
                (0.25 + dx, 1.85 + dy, WALL + 0.12 + dz),
                M.leaf,
                scale = (1.0, 0.5, 0.18),
                rot = (t, 0, a),
                segments = 8,
                rings = 4
            );
            // its stem, up from the soil
            fsweep!(
                f,
                format!("stem{i}"),
                &[
                    (0.25 + dx * 0.15, 1.858, WALL + 0.12 + dz * 0.15),
                    (0.25 + dx * 0.6, 1.85 + dy * 0.6, WALL + 0.12 + dz * 0.6),
                    (0.25 + dx * 0.9, 1.85 + dy - 0.012, WALL + 0.12 + dz * 0.9)
                ],
                0.0025,
                M.leaf,
                verts = 4,
                resolution = 2
            );
        }
        for (k, (x1, z1)) in [(0.33, 0.212), (0.18, 0.215)].into_iter().enumerate() {
            let sign = (1 - 2 * k as i32) as f64;
            let vine = [
                (0.25 + (x1 - 0.25) * 0.4, 1.86, WALL + 0.15),
                (x1, 1.79, WALL + z1 - 0.01),
                (x1 + 0.01 * sign, 1.69, WALL + z1 + 0.005),
                (x1 + 0.02 * sign, 1.6, WALL + z1),
            ];
            fsweep!(
                f,
                format!("vine{k}"),
                &vine,
                0.0025,
                M.leaf,
                verts = 4,
                resolution = 2
            );
            for (j, &(px, py, pz)) in vine[1..].iter().enumerate() {
                let side = 1 - 2 * (j % 2) as i32;
                fsphere!(
                    f,
                    format!("vine_leaf{k}{j}"),
                    0.028,
                    (px + 0.012 * side as f64, py - 0.015, pz),
                    M.leaf,
                    scale = (1.0, 0.6, 0.18),
                    rot = (70, 0, 30 * side),
                    segments = 6,
                    rings = 4
                );
            }
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    settle!(centre = false, floor = false);
    meshes()
}
