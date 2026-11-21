//! The candlelit bistro table and the heart-shaped petal tub.

use super::common::{_candle, _flame, _rose, _rose_leaves, _velvet_chair, _wine_glass, M};
use artkit::forms::rect;
use artkit::kit;
use artkit::math::{Axis, M4, P3, PI, V3, atan2, cos, pow, radians, sin, sqrt};
use artkit::parts::{join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fsphere, fsweep, grow, leaf, loft, revolve, ring, settle, torus};

// --- the candlelit bistro table (2x1): a diner at each end (x = +-0.62, z = 0.1) facing in across the top -------
const TZ: f64 = 0.1; // the table's and the diners' depth (content `places`)
// (x, depth, candle height): either side of their sightline
const CANDLES: [(f64, f64, f64); 2] = [(0.0, TZ - 0.2, 0.26), (0.0, TZ + 0.2, 0.22)];

/// A slipper chair at the table's end, seat centred on (x, z), its sitter facing in (+x from the left end, -x from the
/// right): built facing the front, then turned a quarter about its seat's middle.
fn _end_chair(f: &GameFrame, x: f64, z: f64) {
    let parts = new_parts(|| _velvet_chair(f, &format!("chair{x}"), 0.0, 0.0, None, None, false));
    let turn = M4::translation(V3::from(f.at(x, 0, z)))
        * M4::rotation(radians(if x < 0.0 { -90.0 } else { 90.0 }), Axis::Z);
    for o in parts {
        o.set_matrix_world(turn * o.matrix_world());
    }
}

/// The cloth over the 0.8 x 0.72 top, lofted in one piece: its flat top, the edge rolling over, a skirt falling
/// `drop` in soft folds that flare at the hem, then up its inside to under the table top (hidden).
fn _tablecloth(f: &GameFrame, top: f64, hw: f64, hd: f64, drop: f64) {
    let b = f.at(0, top, TZ);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let n = 48;
    let ring = |rx: f64, ry: f64, y: f64, fold: f64| -> Vec<P3> {
        let pts = ring!(rx, ry, z = bz + y, verts = n, square = 9, at = (bx, by));
        if fold == 0.0 {
            return pts;
        }
        pts.iter()
            .map(|p| {
                let k = 1.0 + fold * sin(10.0 * atan2(p.y - by, p.x - bx));
                P3::from((bx + (p.x - bx) * k, by + (p.y - by) * k, p.z))
            })
            .collect()
    };
    loft!(
        "cloth",
        &[
            ring(0.0, 0.0, 0.0, 0.0),
            ring(hw - 0.01, hd - 0.01, 0.0, 0.0),
            ring(hw + 0.004, hd + 0.004, -0.006, 0.0),
            ring(hw + 0.006, hd + 0.006, -0.03, 0.006),
            ring(hw + 0.009, hd + 0.009, -drop * 0.6, 0.014),
            ring(hw + 0.014, hd + 0.014, -drop, 0.02),
            ring(hw + 0.004, hd + 0.004, -drop, 0.02),
            ring(hw, hd, -0.03, 0.0)
        ],
        material = M.cloth
    );
}

pub fn candlelit_table() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let top = 0.775;
    let still = new_parts(|| {
        // a 0.8 x 0.72 top under a cloth that drops 12 cm all round, clear of the diners' knees
        artkit::fbox!(f, "table_top", 0.8, 0.03, 0.72, (0, 0.745, TZ), M.oak);
        _tablecloth(f, 0.772, 0.41, 0.38, 0.12);
        fsweep!(
            f,
            "runner",
            &kit::fillet!(
                &[
                    (0.0, 0.67, TZ - 0.396),
                    (0.0, 0.775, TZ - 0.393),
                    (0.0, 0.775, TZ + 0.393),
                    (0.0, 0.67, TZ + 0.396)
                ],
                0.012
            ),
            0.0,
            M.runner,
            profile = rect(0.003, 0.12),
            up = (0, 0, 1)
        );
        for x in [-0.33, 0.33] {
            for dz in [-0.27, 0.27] {
                fsweep!(
                    f,
                    format!("table_leg{x}{dz}"),
                    &[(x, 0.73, TZ + dz), (x, 0.0, TZ + dz)],
                    0.02,
                    M.body,
                    verts = 10,
                    scale = [1.0, 0.65]
                );
            }
        }
        for s in [-1, 1] {
            let sf = s as f64;
            let x = sf * 0.22;
            revolve!(
                format!("plate{s}"),
                &[
                    (0.0, 0.0),
                    (0.08, 0.0),
                    (0.09, 0.006),
                    (0.128, 0.014),
                    (0.13, 0.016),
                    (0.124, 0.016),
                    (0.085, 0.01),
                    (0.0, 0.009)
                ],
                at = f.at(x, top, TZ),
                material = M.porcelain,
                segments = 22
            );
            torus!(
                format!("plate_rim{s}"),
                0.126,
                0.0025,
                at = f.at(x, top + 0.016, TZ),
                material = M.gold,
                major_segments = 22,
                minor_segments = 3
            );
            revolve!(
                format!("dessert{s}"),
                &[
                    (0.0, 0.0),
                    (0.05, 0.0),
                    (0.05, 0.035),
                    (0.045, 0.045),
                    (0.02, 0.052),
                    (0.0, 0.053)
                ],
                at = f.at(x, top + 0.009, TZ),
                material = M.dessert,
                segments = 12
            );
            revolve!(
                format!("cream{s}"),
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.026, 0.01),
                    (0.012, 0.02),
                    (0.0, 0.03)
                ],
                at = f.at(x, top + 0.06, TZ),
                material = M.wax,
                segments = 8
            );
            fsphere!(
                f,
                format!("berry{s}"),
                0.012,
                (x, top + 0.092, TZ),
                M.rose,
                segments = 8,
                rings = 5
            );
            for (k, dz) in [-0.16, 0.16].into_iter().enumerate() {
                // a fork and a knife either side of the plate
                fsweep!(
                    f,
                    format!("cutlery{s}{k}"),
                    &[
                        (x - 0.08, top + 0.004, TZ + dz),
                        (x + 0.08, top + 0.004, TZ + dz)
                    ],
                    0.0,
                    M.gold,
                    profile = rect(0.003, if k != 0 { 0.014 } else { 0.018 }),
                    up = (0, 0, 1),
                    scale = if k != 0 { [1.0, 0.7] } else { [1.0, 1.3] }
                );
            }
            _wine_glass(
                f,
                &format!("wine_glass{s}"),
                x - sf * 0.1,
                top,
                TZ - sf * 0.22,
                true,
            ); // at the diner's right hand
            _end_chair(f, sf * 0.62, TZ);
        }
        for (k, (x, z, h)) in CANDLES.into_iter().enumerate() {
            _candle(f, &format!("candle{k}"), x, z, top, h, None);
        }
        revolve!(
            "bud_vase",
            &[
                (0.0, 0.0),
                (0.025, 0.0),
                (0.03, 0.05),
                (0.012, 0.13),
                (0.015, 0.18),
                (0.011, 0.18),
                (0.0, 0.17)
            ],
            at = f.at(0, top, TZ),
            material = M.glassy,
            segments = 12
        );
        fsweep!(
            f,
            "stem",
            &[(0.0, 0.86, TZ), (0.006, 0.96, TZ), (0.01, 1.04, TZ)],
            0.004,
            M.leaf,
            verts = 5,
            resolution = 2
        );
        _rose(f, "vase_rose", 0.01, 1.03, TZ, 0.045, None, 7, 0.0, 0.0);
        _rose_leaves(f, "vase_leaf", 0.005, 0.98, TZ, 2, 0.045, 0.0, None);
        for (k, (x, z)) in [
            (-0.08, TZ + 0.3),
            (0.1, TZ - 0.32),
            (-0.3, TZ - 0.25),
            (0.28, TZ + 0.27),
            (0.05, TZ + 0.28),
        ]
        .into_iter()
        .enumerate()
        {
            leaf!(
                format!("petal{k}"),
                f.at(x, top + 0.004, z),
                (k * 70) as f64,
                4.0,
                0.04,
                0.035,
                M.rose,
                shape = "petal",
                fold = 0.4,
                rows = 2,
                ring = 3
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The two flames, joined with their origin on the shorter wick and built full size, then flattened: `candles`
    // grows them while the two dine; they glow at night (`flame`'s `glow`).
    let flames = new_parts(|| {
        for (k, (x, z, h)) in CANDLES.into_iter().enumerate() {
            _flame(f, &format!("flame{k}"), x, top + 0.06 + h + 0.002, z, 1.1);
        }
    });
    let low = CANDLES.iter().map(|c| c.2).fold(f64::INFINITY, f64::min);
    let flames = join(&flames, "flames", Some(f.at(0, top + 0.05 + low, TZ))); // flattened inside the wax at rest
    flames.set_scale_axis(2, 0.001);
    let objs = settle!(centre = false);
    grow!(flames, "candles", seconds = 0.6);
    objs
}

// --- the heart-shaped petal tub (2x2): two bathers sit in it at x = +-0.5 -----------------------------------

/// The heart's outline in game (x, depth), its point to the front, scaled by `k` about its middle.
fn _heart(k: f64, n: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let t = 2.0 * PI * i as f64 / n as f64;
            let hx = 16.0 * pow(sin(t), 3.0);
            let hy = 13.0 * cos(t) - 5.0 * cos(2.0 * t) - 2.0 * cos(3.0 * t) - cos(4.0 * t);
            (hx * 0.055 * k, -(hy + 2.5) * 0.0552 * k + 0.08)
        })
        .collect()
}

/// The heart outline scaled by `k` at height `y`, as a loft's ring (Blender points).
fn _heart_ring(f: &GameFrame, k: f64, y: f64, n: usize) -> Vec<P3> {
    _heart(k, n).iter().map(|&(x, z)| f.at(x, y, z)).collect()
}

const RIM: f64 = 0.54;
const FLOOR: f64 = 0.2;
const WATER: f64 = 0.47;

pub fn heart_tub() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let hr = |k: f64, y: f64| _heart_ring(f, k, y, 72);
    let still = new_parts(|| {
        // the plinth, its edge eased; the shell in one loft: up the outside, over a rolled rim, down to the floor
        loft!(
            "plinth",
            &[
                hr(1.03, 0.0),
                hr(1.04, 0.01),
                hr(1.04, 0.07),
                hr(1.025, 0.08)
            ],
            material = M.ivory
        );
        loft!(
            "shell",
            &[
                hr(0.95, 0.075),
                hr(0.96, 0.1),
                hr(0.96, RIM - 0.03),
                hr(0.955, RIM - 0.008),
                hr(0.945, RIM),
                hr(0.86, RIM),
                hr(0.845, RIM - 0.012),
                hr(0.84, RIM - 0.04),
                hr(0.835, FLOOR + 0.04),
                hr(0.815, FLOOR + 0.006),
                hr(0.78, FLOOR),
                hr(0.0, FLOOR)
            ],
            material = M.tub
        );
        // an ice bucket with a bottle on the rim, rolled towels, candles and roses on the floor
        revolve!(
            "ice_bucket",
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.06, 0.13),
                (0.064, 0.135),
                (0.064, 0.142),
                (0.056, 0.142),
                (0.052, 0.13),
                (0.04, 0.012),
                (0.0, 0.012)
            ],
            at = f.at(0.6, RIM, 0.3),
            material = M.chrome,
            segments = 16
        );
        for s in [-1, 1] {
            torus!(
                format!("bucket_ring{s}"),
                0.012,
                0.003,
                at = f.at(0.6 + s as f64 * 0.062, RIM + 0.11, 0.3),
                material = M.chrome,
                rot = (90, 0, 90),
                major_segments = 8,
                minor_segments = 3
            );
        }
        let b = f.at(0.6, RIM + 0.012, 0.3);
        let (bx, by, bz) = (b.x, b.y, b.z);
        let lean = M4::translation(V3::from((bx, by, bz)))
            * M4::rotation(radians(15.0), Axis::Y)
            * M4::translation(V3::from((-bx, -by, -bz)));
        let sections: Vec<Vec<P3>> = [
            (0.0, 0.95),
            (0.01, 1.0),
            (0.15, 1.0),
            (0.19, 0.42),
            (0.23, 0.38),
            (0.24, 0.0),
        ]
        .iter()
        .map(|&(h, k)| ring!(0.03 * k, 0.03 * k, z = bz + h, verts = 10, at = (bx, by)))
        .collect();
        let bottle = loft!("bottle", &sections, material = M.bottle);
        bottle.edit(|o| o.mesh.transform(&lean));
        let foil: Vec<Vec<P3>> = [0.19, 0.236]
            .iter()
            .map(|&h| ring!(0.0135, 0.0135, z = bz + h, verts = 10, at = (bx, by)))
            .collect();
        loft!("bottle_foil", &foil, material = M.gold).edit(|o| o.mesh.transform(&lean));
        for (k, (x, y, z, m)) in [
            (-0.78, 0.045, 0.74, M.cloth),
            (-0.78, 0.045, 0.83, M.fabric),
            (-0.78, 0.13, 0.785, M.cloth),
        ]
        .into_iter()
        .enumerate()
        {
            fsweep!(
                f,
                format!("towel{k}"),
                &[(x - 0.17, y, z), (x + 0.16, y, z)],
                0.045,
                m,
                verts = 10
            ); // rolled towels, stacked
            revolve!(
                format!("towel_end{k}"),
                &[
                    (0.0, 0.0),
                    (0.042, 0.0),
                    (0.044, 0.004),
                    (0.03, 0.006),
                    (0.0, 0.004)
                ],
                at = f.at(x + 0.16, y, z),
                material = m,
                segments = 10,
                rot = (0, -90, 0)
            );
        }
        for (k, (x, z, h)) in [
            (-0.85, -0.82, 0.3),
            (-0.72, -0.86, 0.2),
            (0.85, -0.82, 0.26),
            (0.73, -0.87, 0.18),
        ]
        .into_iter()
        .enumerate()
        {
            let (wx, wy, wz) = _candle(
                f,
                &format!("floor_candle{k}"),
                x,
                z,
                0.0,
                h,
                Some(M.rosegold),
            );
            revolve!(
                format!("floor_flame{k}"),
                &[(0.0, 0.0), (0.008, 0.006), (0.01, 0.016), (0.0, 0.04)],
                at = f.at(wx, wy + 0.003, wz),
                material = M.flame_core,
                segments = 6
            );
        }
        revolve!(
            "floor_vase",
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.08, 0.15),
                (0.05, 0.27),
                (0.055, 0.3),
                (0.05, 0.3),
                (0.0, 0.29)
            ],
            at = f.at(0.7, 0, 0.82),
            material = M.ivory,
            segments = 16
        );
        for (x, z, m) in [(0.68, 0.84, M.rose), (0.74, 0.8, M.rose_pink)] {
            fsweep!(
                f,
                format!("vase_stem{x}"),
                &[(0.7, 0.25, 0.82), (x, 0.42, z)],
                0.005,
                M.leaf,
                verts = 5
            );
            _rose(
                f,
                &format!("vase_rose{x}"),
                x,
                0.41,
                z,
                0.05,
                Some(m),
                7,
                0.0,
                0.0,
            );
            _rose_leaves(
                f,
                &format!("vase_leaf{x}"),
                0.5 * (0.7 + x),
                0.36,
                0.5 * (0.82 + z),
                2,
                0.05,
                0.0,
                None,
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The water with its petals and bubbles, built full and flattened onto the tub's floor: `fill` raises it.
    let water = new_parts(|| {
        loft!(
            "water",
            &[
                _heart_ring(f, 0.845, FLOOR - 0.01, 48),
                _heart_ring(f, 0.845, WATER, 48)
            ],
            material = M.water
        );
        for i in 0..26 {
            let a = i as f64 * 2.39996;
            let rr = 0.1 + 0.5 * sqrt((i as f64 + 0.5) / 26.0);
            let (x, z) = (rr * cos(a) * 0.95, -0.05 + rr * sin(a) * 0.72);
            leaf!(
                format!("petal{i}"),
                f.at(x, WATER + 0.002, z),
                (i * 37) as f64,
                6.0,
                0.05,
                0.045,
                if i % 3 != 0 { M.rose } else { M.rose_pink },
                shape = "petal",
                fold = 0.45,
                rows = 2,
                ring = 3
            );
        }
        for (x, z) in [(-0.18, -0.35), (0.25, -0.42), (0.05, 0.3), (-0.4, 0.05)] {
            for k in 0..4 {
                // foam: a little heap of bubbles
                let kf = k as f64;
                fsphere!(
                    f,
                    format!("bubble{x}{z}{k}"),
                    0.04 - 0.008 * kf,
                    (
                        x + 0.03 * cos(kf * 2.1),
                        WATER + 0.004,
                        z + 0.03 * sin(kf * 2.1)
                    ),
                    M.bubble,
                    scale = (1, 1, 0.6),
                    segments = 8,
                    rings = 4
                );
            }
        }
    });
    let water = join(&water, "water", Some(f.at(0, FLOOR - 0.01, 0))); // hidden in the floor at rest
    water.set_scale_axis(2, 0.001);
    let objs = settle!(centre = false);
    grow!(water, "fill", seconds = 1.2);
    objs
}
