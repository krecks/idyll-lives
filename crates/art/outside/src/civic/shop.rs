//! The shops' clothes rail and plant table, and the cash machine.

use super::common::{F, M, WALL, tube};
use artkit::forms::smooth_outline;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{cos, sin};
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, jitter, revolve, settle};

// --- the clothes rail: 2x1, a chrome rail on castored feet, garments on hangers ----------------------------------
/// A garment on its hanger, seen edge on: a shirt, a dress, a coat or trousers over the bar.
pub fn garment(name: &str, x: f64, kind: &str, cloth: Mat) {
    let top = 1.4;
    let shape: Vec<(f64, f64)> = match kind {
        "shirt" => vec![
            (-0.2, top - 0.06),
            (-0.04, top),
            (0.04, top),
            (0.2, top - 0.06),
            (0.22, top - 0.5),
            (0.2, top - 0.6),
            (-0.2, top - 0.6),
            (-0.22, top - 0.5),
        ],
        "dress" => vec![
            (-0.18, top - 0.06),
            (-0.04, top),
            (0.04, top),
            (0.18, top - 0.06),
            (0.15, top - 0.35),
            (0.26, top - 0.85),
            (-0.26, top - 0.85),
            (-0.15, top - 0.35),
        ],
        "coat" => vec![
            (-0.23, top - 0.07),
            (-0.05, top),
            (0.05, top),
            (0.23, top - 0.07),
            (0.25, top - 0.4),
            (0.25, top - 0.8),
            (-0.25, top - 0.8),
            (-0.25, top - 0.4),
        ],
        "trousers" => vec![
            (-0.16, top - 0.03),
            (0.16, top - 0.03),
            (0.17, top - 0.5),
            (-0.17, top - 0.5),
        ],
        _ => panic!("no garment {kind:?}"),
    };
    let t = match kind {
        "coat" => 0.06,
        "trousers" => 0.03,
        _ => 0.04,
    };
    fextrude!(
        F,
        name,
        &smooth_outline(&shape, 1, true),
        t,
        (x, 0, 0),
        cloth,
        plane = Facing::Side,
        bevel = 0.008
    );
    tube(
        &format!("{name}_hook"),
        &[
            (x, top - 0.005, 0.0),
            (x, top + 0.07, 0.0),
            (x, top + 0.115, -0.02),
            (x, top + 0.1, -0.045),
        ],
        0.003,
        M.bell,
        0.02,
        6,
    );
}

pub fn clothes_rack() -> Vec<Obj> {
    let rail = 1.5;
    let parts = new_parts(|| {
        for s in [-1, 1] {
            let x = s as f64 * 0.86;
            fbox!(
                F,
                format!("foot{s}"),
                0.04,
                0.03,
                0.52,
                (x, 0.075, 0),
                M.bell,
                bevel = 0.006
            );
            for z in [-0.23, 0.23] {
                let zs = float_repr(z);
                fcyl!(
                    F,
                    format!("castor{s}{zs}"),
                    0.03,
                    0.025,
                    (x, 0.032, z),
                    M.radio,
                    rot = (0, 90, 0),
                    verts = 12
                );
                fbox!(
                    F,
                    format!("fork{s}{zs}"),
                    0.035,
                    0.03,
                    0.03,
                    (x, 0.07, z),
                    M.bell
                );
            }
        }
        tube(
            "frame",
            &[
                (-0.86, 0.09, 0.0),
                (-0.86, rail, 0.0),
                (0.86, rail, 0.0),
                (0.86, 0.09, 0.0),
            ],
            0.016,
            M.bell,
            0.06,
            10,
        );
        let kinds = [
            "coat", "shirt", "shirt", "dress", "shirt", "trousers", "dress", "shirt", "coat",
            "shirt", "dress", "trousers", "shirt", "shirt", "dress",
        ];
        let cloths = [
            M.cloth2, M.cloth1, M.cloth4, M.cloth3, M.cloth5, M.cloth2, M.cloth1, M.cloth6,
            M.cloth6, M.cloth4, M.cloth5, M.cloth6, M.cloth3, M.cloth1, M.cloth2,
        ];
        for (k, (kind, cloth)) in kinds.into_iter().zip(cloths).enumerate() {
            garment(
                &format!("garment{k}"),
                -0.72 + k as f64 * 0.103,
                kind,
                cloth,
            );
        }
    });
    join(&parts, "rail", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the plant table: 2x1, slatted staging on a galvanised frame, potted plants and seed trays ------------------
pub fn pot_plant(name: &str, x: f64, y: f64, z: f64, k: usize) {
    let r = 0.08;
    revolve!(
        format!("{name}_pot"),
        &[
            (0.0, 0.0),
            (r * 0.72, 0.0),
            (r * 0.95, r * 1.6),
            (r * 1.02, r * 1.62),
            (r * 1.02, r * 1.8),
            (r * 0.95, r * 1.8),
            (0.0, r * 1.7)
        ],
        at = F.at(x, y, z),
        material = M.terracotta,
        segments = 12
    );
    let top = y + r * 1.75;
    let kf = k as f64;
    if k % 3 == 2 {
        // an upright plant, leaves up a stem
        fcyl!(
            F,
            format!("{name}_stem"),
            0.008,
            0.3,
            (x, top + 0.15, z),
            M.foliage,
            verts = 6
        );
        for j in 0..5 {
            let a = j as f64 * 2.4;
            jitter!(
                fsphere!(
                    F,
                    format!("{name}_leaf{j}"),
                    0.05,
                    (
                        x + 0.05 * cos(a),
                        top + 0.08 + j as f64 * 0.05,
                        z + 0.05 * sin(a)
                    ),
                    M.foliage,
                    scale = (1.3, 0.6, 0.6),
                    segments = 8,
                    rings = 5
                ),
                0.01,
                size = 0.04,
                seed = (k + j) as f64
            );
        }
        return;
    }
    for (j, (dx, dy, dz, s)) in [
        (0.0, 0.06, 0.0, 1.0),
        (-0.05, 0.04, 0.03, 0.7),
        (0.05, 0.045, -0.03, 0.75),
        (0.02, 0.04, 0.05, 0.6),
    ]
    .into_iter()
    .enumerate()
    {
        jitter!(
            fsphere!(
                F,
                format!("{name}_mound{j}"),
                0.08 * s,
                (x + dx, top + dy, z + dz),
                M.foliage,
                scale = (1, 1, 0.75),
                segments = 9,
                rings = 5
            ),
            0.015,
            size = 0.05,
            seed = kf * 2.1 + j as f64
        );
    }
    if k % 3 == 1 {
        // in flower
        for j in 0..6 {
            let a = j as f64 * 1.05 + kf;
            fsphere!(
                F,
                format!("{name}_bloom{j}"),
                0.018,
                (
                    x + 0.06 * cos(a),
                    top + 0.1 + 0.02 * (j % 2) as f64,
                    z + 0.06 * sin(a)
                ),
                [M.bloom1, M.bloom2, M.bloom3][(k / 3) % 3],
                segments = 6,
                rings = 4
            );
        }
    }
    fbox!(
        F,
        format!("{name}_tag"),
        0.03,
        0.05,
        0.003,
        (x + 0.05, top + 0.02, z + 0.07),
        M.label,
        rot = (-15, 0, 0)
    );
}

pub fn plant_table() -> Vec<Obj> {
    let top = 0.75;
    let parts = new_parts(|| {
        for x in [-0.88, 0.88] {
            let xs = float_repr(x);
            for z in [-0.36, 0.36] {
                fbox!(
                    F,
                    format!("leg{xs}{}", float_repr(z)),
                    0.04,
                    top - 0.025,
                    0.04,
                    (x, (top - 0.025) / 2.0, z),
                    M.galv,
                    bevel = 0.004
                );
            }
            fbox!(
                F,
                format!("end_rail{xs}"),
                0.03,
                0.04,
                0.72,
                (x, top - 0.045, 0),
                M.galv
            );
            fbox!(
                F,
                format!("low_end{xs}"),
                0.03,
                0.03,
                0.72,
                (x, 0.22, 0),
                M.galv
            );
        }
        for z in [-0.36, 0.36] {
            let zs = float_repr(z);
            fbox!(
                F,
                format!("rail{zs}"),
                1.76,
                0.04,
                0.03,
                (0, top - 0.045, z),
                M.galv
            );
            fbox!(
                F,
                format!("low_rail{zs}"),
                1.76,
                0.03,
                0.03,
                (0, 0.22, z),
                M.galv
            );
        }
        for i in 0..8 {
            fbox!(
                F,
                format!("slat{i}"),
                1.88,
                0.022,
                0.08,
                (0, top - 0.011, -0.36 + i as f64 * 0.103),
                M.slat,
                bevel = 0.004
            );
        }
        for i in 0..5 {
            fbox!(
                F,
                format!("low_slat{i}"),
                1.76,
                0.02,
                0.08,
                (0, 0.245, -0.32 + i as f64 * 0.16),
                M.slat,
                bevel = 0.004
            );
        }
        let mut k = 0;
        for z in [-0.19, 0.19] {
            for i in 0..5 {
                pot_plant(
                    &format!("plant{k}"),
                    -0.72 + i as f64 * 0.36,
                    top,
                    z + 0.02 * ((i % 2) * 2 - 1) as f64,
                    k,
                );
                k += 1;
            }
        }
        // seed trays on the shelf below
        for (t, x) in [-0.45, 0.4].into_iter().enumerate() {
            fbox!(
                F,
                format!("tray{t}"),
                0.56,
                0.05,
                0.36,
                (x, 0.28, 0.0),
                M.tray,
                bevel = 0.004
            );
            fbox!(
                F,
                format!("tray{t}_soil"),
                0.54,
                0.004,
                0.34,
                (x, 0.305, 0.0),
                M.soil
            );
            for r in 0..3 {
                for c in 0..5 {
                    fsphere!(
                        F,
                        format!("seedling{t}{r}{c}"),
                        0.03,
                        (x - 0.2 + c as f64 * 0.1, 0.32, -0.11 + r as f64 * 0.11),
                        M.foliage,
                        scale = (1, 1, 0.6),
                        segments = 5,
                        rings = 3
                    );
                }
            }
        }
    });
    join(&parts, "table", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the cash machine: 1x1 against a wall, a lit band over the screen and keypad ----------------------------------
pub fn atm() -> Vec<Obj> {
    let b = WALL;
    let parts = new_parts(|| {
        fbox!(F, "plinth", 0.62, 0.06, 0.46, (0, 0.03, b + 0.23), M.plinth);
        fbox!(
            F,
            "lower",
            0.66,
            0.95,
            0.5,
            (0, 0.06 + 0.475, b + 0.25),
            M.atm,
            bevel = 0.008
        );
        fbox!(
            F,
            "upper",
            0.66,
            0.62,
            0.36,
            (0, 1.01 + 0.31, b + 0.18),
            M.atm,
            bevel = 0.008
        );
        // the fascia: the screen in a sloping recess, the keypad on a shelf, the slots
        fbox!(
            F,
            "fascia",
            0.56,
            0.42,
            0.02,
            (0, 1.3, b + 0.36),
            M.fascia,
            bevel = 0.004,
            rot = (10, 0, 0)
        );
        fbox!(
            F,
            "screen",
            0.36,
            0.26,
            0.006,
            (0, 1.32, b + 0.373),
            M.atm_screen,
            rot = (10, 0, 0)
        );
        fbox!(
            F,
            "shelf",
            0.6,
            0.04,
            0.16,
            (0, 1.0, b + 0.56),
            M.fascia,
            bevel = 0.008,
            rot = (-12, 0, 0)
        );
        for r in 0..4 {
            for c in 0..3 {
                let rf = r as f64;
                fbox!(
                    F,
                    format!("key{r}{c}"),
                    0.04,
                    0.012,
                    0.028,
                    (
                        -0.12 + c as f64 * 0.05,
                        1.026 - rf * 0.0075,
                        b + 0.52 + rf * 0.035 - 0.0
                    ),
                    if c < 3 { M.keys } else { M.leaflet4 },
                    bevel = 0.003,
                    rot = (-12, 0, 0)
                );
            }
        }
        for (k, m) in [M.leaflet4, M.leaflet2, M.leaflet3].into_iter().enumerate() {
            let kf = k as f64;
            fbox!(
                F,
                format!("act{k}"),
                0.06,
                0.012,
                0.028,
                (0.07, 1.026 - kf * 0.0075, b + 0.52 + kf * 0.035),
                m,
                bevel = 0.003,
                rot = (-12, 0, 0)
            );
        }
        fbox!(
            F,
            "card_slot",
            0.1,
            0.04,
            0.03,
            (0.2, 1.08, b + 0.515),
            M.fascia,
            bevel = 0.004
        );
        fbox!(
            F,
            "card_mouth",
            0.07,
            0.006,
            0.004,
            (0.2, 1.08, b + 0.531),
            M.slot
        );
        fbox!(
            F,
            "card_light",
            0.07,
            0.01,
            0.004,
            (0.2, 1.096, b + 0.531),
            M.sign
        );
        fbox!(
            F,
            "cash_slot",
            0.3,
            0.05,
            0.02,
            (0, 0.88, b + 0.508),
            M.fascia,
            bevel = 0.004
        );
        fbox!(
            F,
            "cash_mouth",
            0.26,
            0.012,
            0.004,
            (0, 0.88, b + 0.519),
            M.slot
        );
        fbox!(
            F,
            "receipt_slot",
            0.08,
            0.008,
            0.004,
            (-0.2, 1.08, b + 0.502),
            M.slot
        );
        // privacy wings beside the keypad, and the hood with its lit band
        for s in [-1, 1] {
            fextrude!(
                F,
                format!("wing{s}"),
                &[
                    (b + 0.36, 0.95),
                    (b + 0.62, 0.95),
                    (b + 0.62, 1.05),
                    (b + 0.4, 1.5),
                    (b + 0.36, 1.5)
                ],
                0.015,
                (s as f64 * 0.325, 0, 0),
                M.atm,
                plane = Facing::Side,
                bevel = 0.004
            );
        }
        fbox!(
            F,
            "hood",
            0.7,
            0.05,
            0.52,
            (0, 1.655, b + 0.26),
            M.atm,
            bevel = 0.01
        );
        fbox!(F, "band", 0.66, 0.1, 0.012, (0, 1.6, b + 0.512), M.sign);
        fbox!(
            F,
            "band_rim",
            0.7,
            0.012,
            0.03,
            (0, 1.545, b + 0.505),
            M.fascia
        );
    });
    join(&parts, "atm", Some((0, 0, 0).into()));
    settle!(centre = false);
    meshes()
}
