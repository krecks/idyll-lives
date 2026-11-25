//! The fireside reading lounge and the bedside book stack.

use super::common::{M, WALL, books, lamp, mug, pile};
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{Axis, P3, cos, cosd, radians, sin, sind};
use artkit::motion::{Grow, grow};
use artkit::parts::{aside, hinged, join, new_parts, numbered, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::py::norm;
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::shapes::SHARP;
use artkit::{
    fbox, fcyl, fextrude, fsweep, jitter, kit, loft, revolve, ring, settle, soft, swing, torus,
};

// --- the fireside reading lounge (2x2): two wing chairs before an open fire grate under a hood ----------------
const STOVE: (f64, f64) = (0.0, 0.62); // the grate's middle (x, depth)
const HOOD_Y: f64 = 0.6; // the hood's rim, right over the flames (they reach 0.47); 2026-10-21: 1.0, over the air (M-packs-10)

/// A wing chair centred on (x, z), facing the front: a stuffed seat cushion (top 0.45) on a sprung base, a
/// tall buttoned back, wings curving forward from it, arms rolled outward, short cabriole legs.
fn wing_chair(f: &GameFrame, x: f64, z: f64) {
    let (w, d) = (0.66, 0.64);
    let z0 = z - d / 2.0;
    soft!(
        f,
        numbered("chair_base"),
        w - 0.08,
        0.2,
        d - 0.06,
        (x, 0.26, z + 0.01),
        M.velvet,
        round = 0.04,
        flat = 1,
        round_segments = 1,
        resting = true
    );
    soft!(
        f,
        numbered("chair_seat"),
        w - 0.24,
        0.088,
        d - 0.2,
        (x, 0.394, z + 0.07),
        M.velvet,
        round = 0.03,
        puff = 0.012,
        piping = 0.005,
        flat = 1,
        round_segments = 1,
        resting = true
    );
    soft!(
        f,
        numbered("chair_back"),
        w - 0.1,
        0.72,
        0.15,
        (x, 0.71, z0 + 0.1),
        M.velvet,
        rot = (7, 0, 0),
        round = 0.06,
        puff = 0.03,
        flat = 1,
        resting = true
    );
    for s in [-1.0, 1.0] {
        let ax = x + s * (w / 2.0 - 0.065);
        // the arm: a panel rising into a roll that leans out, swept front to back
        let roll: Vec<(f64, f64)> = (-40..230)
            .step_by(30)
            .map(|a| {
                (
                    0.56 + 0.06 * sin(radians(a as f64)),
                    s * 0.065 * cos(radians(a as f64)) + s * 0.01,
                )
            })
            .collect();
        let section = [
            vec![(0.3, s * -0.05), (0.3, s * 0.05)],
            roll,
            vec![(0.5, s * -0.05)],
        ]
        .concat();
        let arm = fsweep!(
            f,
            numbered("chair_arm"),
            &[(ax, 0.0, z0 + 0.1), (ax, 0.0, z + d / 2.0 - 0.035)],
            0,
            M.velvet,
            profile = section,
            angle = 50
        );
        bevel_edges(arm, 0.004, 1, SHARP, &EdgeOrder::Corners); // its ends' upholstered edges
        fcyl!(
            f,
            numbered("chair_scroll"),
            0.06,
            0.02,
            (ax + s * 0.01, 0.565, z + d / 2.0 - 0.03),
            M.velvet,
            rot = (90, 0, 0),
            verts = 12,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the wing: a shaped panel from the back's top curving down to the arm
        let wing = smooth_outline(
            &[
                (-0.3, 0.55),
                (-0.3, 1.03),
                (-0.2, 1.07),
                (-0.08, 1.02),
                (0.0, 0.9),
                (0.02, 0.74),
                (0.06, 0.6),
                (0.0, 0.56),
            ],
            1,
            true,
        );
        let wing: Vec<(f64, f64)> = wing.iter().map(|&(u, v)| (z + u, v)).collect();
        fextrude!(
            f,
            numbered("chair_wing"),
            &wing,
            0.075,
            (x + s * (w / 2.0 - 0.065), 0.0, 0.0),
            M.velvet,
            plane = Facing::Side,
            rot = (0.0, 0.0, s * 6.0),
            bevel = 0.025,
            bevel_segments = 1
        );
        for (lz, dz) in [(z0 + 0.07, -1.0), (z + d / 2.0 - 0.07, 1.0)] {
            kit::cabriole!(
                f,
                numbered("chair_leg"),
                ax,
                lz,
                0.19, // (its top inside the base, not showing at its rounded edge)
                M.walnut,
                s * 0.012,
                dz * 0.012,
                r = 0.02,
                verts = 8
            );
        }
    }
}

/// A split log from game point `a` to `b`: a barked, slightly crooked round, its cut ends pale.
fn log(
    f: &GameFrame,
    name: &str,
    a: (f64, f64, f64),
    b: (f64, f64, f64),
    r: f64,
    seed: f64,
    material: Option<Mat>,
) {
    let mid = (
        (a.0 + b.0) / 2.0 + 0.0,
        (a.1 + b.1) / 2.0 + 0.012,
        (a.2 + b.2) / 2.0 + 0.0,
    );
    let l = fsweep!(
        f,
        name,
        &[a, mid, b],
        r,
        material.unwrap_or(M.log),
        verts = 8,
        resolution = 2,
        scale = [1.0, 1.06, 0.94]
    );
    jitter!(l, r * 0.12, size = 0.03, seed = seed);
    for (k, p) in [a, b].into_iter().enumerate() {
        let dist = norm(&[p.0 - mid.0, p.1 - mid.1, p.2 - mid.2]);
        let k_ = if 1e-6 < dist { dist } else { 1e-6 };
        let q = (
            p.0 + (p.0 - mid.0) * 0.001 / k_,
            p.1 + (p.1 - mid.1) * 0.001 / k_,
            p.2 + (p.2 - mid.2) * 0.001 / k_,
        );
        fsweep!(
            f,
            format!("{name}_end{k}"),
            &[p, q],
            r * 0.88 * if k != 0 { 0.94 } else { 1.0 },
            M.log_end,
            verts = 8
        );
    }
}

/// A tongue of flame standing on `y`: a body lofted from a round foot, swelling, then drawn up into a flickering
/// tip that leans and twists.
#[allow(clippy::too_many_arguments)]
fn flame(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    turn: f64,
) -> Obj {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    let rows = [
        (0.0, 0.7),
        (0.15, 1.0),
        (0.4, 0.85),
        (0.65, 0.5),
        (0.85, 0.22),
        (1.0, 0.02),
    ];
    let secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(t, s)| {
            let lean = 0.25 * r * t * t;
            ring!(
                r * s,
                r * s * 0.7,
                z = bz + h * t,
                verts = 8,
                at = (bx - lean * cosd(turn), by - lean * sind(turn))
            )
        })
        .collect();
    loft!(name, &secs, material = material)
}

pub fn fireside_lounge() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(5);
    let (sx, sz) = STOVE;
    let still = new_parts(|| {
        // an oval rug: its border, a field and a medallion in the middle
        fcyl!(
            f,
            "rug",
            0.85,
            0.012,
            (0, 0.006, 0.2),
            M.rug,
            verts = 40,
            scale = (1, 0.76, 1),
            bevel = 0.004,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "rug_field",
            0.75,
            0.014,
            (0, 0.007, 0.2),
            M.rug_inner,
            verts = 40,
            scale = (1, 0.75, 1)
        );
        fcyl!(
            f,
            "rug_medallion",
            0.3,
            0.016,
            (0, 0.008, 0.2),
            M.rug,
            verts = 24,
            scale = (1, 0.7, 1)
        );
        // the hearth stone, the iron fire basket on its legs with logs, the hood on its flue right over them
        fcyl!(
            f,
            "hearth",
            0.3,
            0.05,
            (sx, 0.025, sz),
            M.stone,
            verts = 24,
            bevel = 0.01,
            bevel_segments = 1
        );
        revolve!(
            "grate",
            &[
                (0.0, 0.0),
                (0.15, 0.0),
                (0.159, 0.01), // (its rim rounded over)
                (0.16, 0.0125),
                (0.158, 0.015),
                (0.145, 0.015),
                (0.0, 0.015)
            ],
            at = f.at(sx, 0.12, sz),
            material = M.iron,
            segments = 16
        );
        for k in 0..12 {
            // the basket's bars, bowed out
            let a = (30 * k + 15) as f64;
            let (c, s_) = (cosd(a), sind(a));
            kit::tube!(
                f,
                numbered("grate_bar"),
                &[
                    (sx + 0.15 * c, 0.13, sz + 0.15 * s_),
                    (sx + 0.2 * c, 0.2, sz + 0.2 * s_),
                    (sx + 0.205 * c, 0.27, sz + 0.205 * s_),
                ],
                0.006,
                M.iron,
                verts = 4,
                round = 2
            );
        }
        torus!(
            "grate_ring",
            0.205,
            0.008,
            at = f.at(sx, 0.27, sz),
            material = M.iron,
            major_segments = 20,
            minor_segments = 3
        );
        for k in 0..3 {
            let a = (120 * k + 30) as f64;
            let (c, s_) = (cosd(a), sind(a));
            kit::tube!(
                f,
                numbered("grate_leg"),
                &[
                    (sx + 0.12 * c, 0.13, sz + 0.12 * s_),
                    (sx + 0.16 * c, 0.07, sz + 0.16 * s_),
                    (sx + 0.17 * c, 0.05, sz + 0.17 * s_),
                ],
                0.012,
                M.iron,
                verts = 6,
                round = 2
            );
        }
        for (k, (a, dx, dy)) in [(20.0, -0.04, 0.0), (-25.0, 0.05, 0.0), (90.0, 0.0, 0.035)]
            .into_iter()
            .enumerate()
        {
            let (c, s_) = (cosd(a) * 0.14, sind(a) * 0.14);
            log(
                f,
                &numbered("log"),
                (sx + dx - c, 0.17 + dy, sz - s_),
                (sx + dx + c, 0.17 + dy, sz + s_),
                0.035,
                k as f64 * 1.7,
                None,
            );
        }
        fcyl!(
            f,
            "embers",
            0.13,
            0.02,
            (sx, 0.15, sz),
            M.ember,
            verts = 14,
            bevel = 0.004,
            bevel_segments = 1
        );
        let at = f.at(sx, 0, sz);
        let hood: Vec<Vec<P3>> = [
            (0.25, 0.0),
            (0.25, 0.02),
            (0.2, 0.08),
            (0.12, 0.18),
            (0.075, 0.25),
            (0.062, 0.276),
            (0.058, 0.28), // (its top's edge inside the flue)
        ]
        .iter()
        .map(|&(r, b)| ring!(r, r, z = b + HOOD_Y, verts = 24, at = (at.x, at.y)))
        .collect();
        loft!("hood", &hood, material = M.iron);
        torus!(
            "hood_rim",
            0.25,
            0.01,
            at = f.at(sx, HOOD_Y + 0.005, sz),
            material = M.iron,
            major_segments = 24,
            minor_segments = 3
        );
        // the flue up to the ceiling (2.8 m), its joints banded, and its collar there: the hood hangs from it
        fcyl!(
            f,
            "flue",
            0.06,
            2.8 - HOOD_Y - 0.28,
            (sx, (2.8 + HOOD_Y + 0.28) / 2.0, sz),
            M.iron,
            verts = 16
        );
        for y in [1.3, 2.05] {
            fcyl!(
                f,
                numbered("flue_band"),
                0.066,
                0.025,
                (sx, y, sz),
                M.iron,
                verts = 16,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fcyl!(
            f,
            "flue_collar",
            0.09,
            0.03,
            (sx, 2.785, sz),
            M.iron,
            verts = 16,
            bevel = 0.005,
            bevel_segments = 1
        );
        // the chairs, a tripod table with books and tea between them, a low bookcase behind
        wing_chair(f, -0.5, 0.0);
        wing_chair(f, 0.5, 0.0);
        fextrude!(
            f,
            "table_top",
            &rounded_rect(0.3, 0.3, 0.15, 6),
            0.03,
            (0, 0.56, -0.16),
            M.walnut,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 1
        );
        revolve!(
            "table_stem",
            &[
                (0.03, 0.12),
                (0.042, 0.16),
                (0.03, 0.22),
                (0.026, 0.4),
                (0.034, 0.48),
                (0.03, 0.54),
                (0.0, 0.545),
                (0.0, 0.12)
            ],
            at = f.at(0, 0, -0.16),
            material = M.walnut,
            segments = 12
        );
        for k in 0..3 {
            let a = (120 * k + 90) as f64;
            let (c, s_) = (cosd(a), sind(a));
            fsweep!(
                f,
                numbered("table_leg"),
                &[
                    (0.03 * c, 0.2, -0.16 + 0.03 * s_),
                    (0.1 * c, 0.1, -0.16 + 0.1 * s_),
                    (0.17 * c, 0.025, -0.16 + 0.17 * s_),
                    (0.19 * c, 0.013, -0.16 + 0.19 * s_)
                ],
                0.016,
                M.walnut,
                verts = 6,
                resolution = 2,
                scale = [1.2, 1.0, 0.7, 0.8]
            );
        }
        pile(f, &mut rnd, -0.02, -0.17, 0.575, 3, 0.15, 0.11);
        mug(f, 0.07, 0.575, -0.09, M.porcelain, None);
        // the low bookcase: a moulded top, a plinth, two bays of books and a lamp on it
        fbox!(
            f,
            "case",
            1.7,
            0.56,
            0.3,
            (0, 0.3, -0.78),
            M.walnut,
            bevel = 0.008,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "case_top",
            &rounded_rect(1.7, 0.3, 0.01, 1),
            0.03,
            (0, 0.585, -0.78),
            M.walnut,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "case_plinth",
            1.7,
            0.05,
            0.3,
            (0, 0.025, -0.78),
            M.walnut,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(f, "case_mouth", 1.6, 0.46, 0.02, (0, 0.32, -0.626), M.back);
        fbox!(f, "case_shelf", 1.6, 0.02, 0.26, (0, 0.32, -0.76), M.walnut);
        fbox!(
            f,
            "case_divider",
            0.03,
            0.46,
            0.27,
            (0, 0.32, -0.76),
            M.walnut
        );
        books(
            f, &mut rnd, -0.8, 0.0, 0.33, -0.9, 0.2, 0.24, 0.9, true, false,
        );
        books(
            f, &mut rnd, 0.02, 0.8, 0.33, -0.9, 0.2, 0.24, 0.7, false, true,
        );
        books(
            f, &mut rnd, -0.8, 0.0, 0.1, -0.9, 0.2, 0.2, 0.85, false, true,
        );
        books(
            f, &mut rnd, 0.02, 0.8, 0.1, -0.9, 0.2, 0.2, 0.95, false, false,
        );
        lamp(f, 0.6, 0.6, -0.78, 0.5, None);
        // a wicker log basket by the fire: woven, a rim, two logs in it
        revolve!(
            "basket",
            &[
                (0.0, 0.0),
                (0.15, 0.0),
                (0.17, 0.24),
                (0.16, 0.24),
                (0.14, 0.02),
                (0.0, 0.02)
            ],
            at = f.at(0.68, 0, 0.62),
            material = M.wicker,
            segments = 18,
            scale = (1, 0.76, 1)
        );
        for y in [0.08, 0.16] {
            let r = 0.15 + 0.02 * y / 0.24;
            torus!(
                numbered("weave"),
                1.0,
                0.006,
                at = f.at(0.68, y, 0.62),
                material = M.wicker,
                major_segments = 18,
                minor_segments = 3
            )
            .set_scale((r + 0.003, (r + 0.003) * 0.76, 1.0));
        }
        torus!(
            "basket_rim",
            1.0,
            0.012,
            at = f.at(0.68, 0.24, 0.62),
            material = M.wicker,
            major_segments = 18,
            minor_segments = 3
        )
        .set_scale((0.165, 0.165 * 0.76, 1.0));
        for k in 0..3 {
            let a = (30 * k - 30) as f64;
            let (c, s_) = (cosd(a) * 0.16, sind(a) * 0.16);
            let kf = k as f64;
            log(
                f,
                &numbered("spare_log"),
                (0.62 + 0.05 * kf - s_ * 0.3, 0.2 + 0.03 * kf, 0.62 - c),
                (0.62 + 0.05 * kf + s_ * 0.3, 0.27 + 0.03 * kf, 0.62 + c),
                0.034,
                (5 + k) as f64,
                None,
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The fire: tongues of flame on the logs, joined with their origin on the grate and built flat, so
    // `flames` grows them up while residents read by it.
    let fire = new_parts(|| {
        for (dx, dz, r, h, m, t) in [
            (0.0, 0.0, 0.07, 0.3, M.flame, 0.0),
            (-0.07, 0.03, 0.05, 0.2, M.flame, 120.0),
            (0.07, -0.03, 0.05, 0.22, M.flame, 250.0),
            (0.02, 0.05, 0.04, 0.16, M.flame_core, 60.0),
            (-0.02, -0.04, 0.035, 0.18, M.flame_core, 200.0),
        ] {
            flame(f, &numbered("flame"), sx + dx, 0.17, sz + dz, r, h, m, t);
        }
    });
    let fire = join(&fire, "flames", Some(f.at(sx, 0.17, sz)));
    fire.edit(|o| o.scale.z = 0.001);
    let objs = settle!(centre = false);
    grow(
        fire,
        "flames",
        Grow {
            seconds: 0.8,
            ..Grow::default()
        },
    );
    objs
}

// --- the bedside book stack (1x1): a nightstand-sized chest of drawers (0.6 wide) piled with books, a lamp and tea --
pub fn bedside_stack() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(9);
    let (h, d) = (0.5, 0.43);
    let zf = WALL + d;
    let mut top = 0.0;
    let still = new_parts(|| {
        fextrude!(
            f,
            "chest_top",
            &rounded_rect(0.6, d, 0.012, 2),
            0.03,
            (0.0, h - 0.015, WALL + d / 2.0),
            M.body,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "chest",
            0.58,
            h - 0.09,
            d - 0.02,
            (0.0, 0.06 + (h - 0.09) / 2.0, WALL + (d - 0.02) / 2.0),
            M.body,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "chest_plinth",
            0.59,
            0.025,
            d - 0.01,
            (0.0, 0.0725, WALL + (d - 0.01) / 2.0),
            M.body,
            bevel = 0.005,
            bevel_segments = 1
        );
        for lx in [-0.26, 0.26] {
            for lz in [WALL + 0.04, zf - 0.04] {
                revolve!(
                    numbered("leg"),
                    &[
                        (0.0, 0.0),
                        (0.014, 0.0),
                        (0.018, 0.012),
                        (0.026, 0.035),
                        (0.024, 0.05),
                        (0.02, 0.06),
                        (0.0, 0.06)
                    ],
                    at = f.at(lx, 0, lz),
                    material = M.body,
                    segments = 8
                );
            }
        }
        let (fw, fh) = (0.54 / 2.0, (h - 0.14) / 2.0);
        for i in 0..2 {
            for j in 0..2 {
                let (cx, cy) = (-0.27 + fw * (i as f64 + 0.5), 0.095 + fh * (j as f64 + 0.5));
                kit::shaker!(
                    f,
                    numbered("drawer"),
                    fw - 0.014,
                    fh - 0.014,
                    (cx, cy, zf - 0.002),
                    0.016,
                    M.panel,
                    rail = 0.03
                );
                // a brass cup pull and a card frame with its label
                fsweep!(
                    f,
                    numbered("pull"),
                    &[
                        (cx - 0.035, cy - 0.006, zf + 0.006),
                        (cx - 0.03, cy - 0.022, zf + 0.012),
                        (cx, cy - 0.03, zf + 0.016),
                        (cx + 0.03, cy - 0.022, zf + 0.012),
                        (cx + 0.035, cy - 0.006, zf + 0.006)
                    ],
                    0,
                    M.brass,
                    profile = [
                        (-0.006, -0.002),
                        (0.006, -0.002),
                        (0.006, 0.002),
                        (-0.006, 0.002)
                    ],
                    up = (0, 0, 1)
                );
                fbox!(
                    f,
                    numbered("pull_plate"),
                    0.08,
                    0.012,
                    0.004,
                    (cx, cy - 0.004, zf + 0.008),
                    M.brass
                );
                fbox!(
                    f,
                    numbered("label_frame"),
                    0.054,
                    0.032,
                    0.004,
                    (cx, cy + 0.035, zf + 0.007),
                    M.brass
                );
                fbox!(
                    f,
                    numbered("label"),
                    0.046,
                    0.024,
                    0.002,
                    (cx, cy + 0.035, zf + 0.009),
                    M.page
                );
            }
        }
        pile(f, &mut rnd, -0.12, -0.25, h, 5, 0.22, 0.16);
        top = pile(f, &mut rnd, 0.1, -0.15, h, 2, 0.2, 0.15);
        // the open book's pages and back board under its cover, the spine rounded
        fbox!(
            f,
            "book_block",
            0.19,
            0.03,
            0.145,
            (0.1, top + 0.015, -0.15),
            M.page
        );
        fbox!(
            f,
            "book_back",
            0.2,
            0.006,
            0.15,
            (0.1, top + 0.003, -0.15),
            M.book_a
        );
        fcyl!(
            f,
            "book_spine",
            0.021,
            0.15,
            (0.198, top + 0.0195, -0.15),
            M.book_a,
            rot = (90, 0, 0),
            verts = 10,
            scale = (0.4, 1, 1)
        );
        fbox!(
            f,
            "book_ribbon",
            0.008,
            0.002,
            0.08,
            (0.06, top + 0.031, -0.075),
            M.book_d
        );
        mug(f, -0.22, h, -0.06, M.mug_a, None);
        // reading glasses folded on the top: two rims, a bridge, the arms folded behind
        let (gx, gy, gz) = (-0.07, h + 0.004, -0.05);
        for s in [-1.0, 1.0] {
            torus!(
                numbered("lens"),
                0.02,
                0.0028,
                at = f.at(gx + s * 0.024, gy, gz),
                material = M.dark,
                major_segments = 10,
                minor_segments = 3
            );
        }
        fsweep!(
            f,
            "glasses_bridge",
            &[
                (gx - 0.006, gy, gz + 0.006),
                (gx, gy + 0.004, gz + 0.008),
                (gx + 0.006, gy, gz + 0.006)
            ],
            0.0022,
            M.dark,
            verts = 4,
            resolution = 2
        );
        fsweep!(
            f,
            "glasses_arms",
            &kit::fillet!(
                &[
                    (gx - 0.044, gy, gz + 0.002),
                    (gx - 0.048, gy + 0.004, gz - 0.016),
                    (gx + 0.04, gy + 0.004, gz - 0.02),
                    (gx + 0.046, gy + 0.008, gz - 0.006)
                ],
                0.006
            ),
            0.0022,
            M.dark,
            verts = 4
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // the lamp on it and a pile on the floor in front of it: drawn with it, not the chest's own size
    let things = new_parts(|| {
        lamp(f, 0.17, h, -0.3, 0.42, None);
        pile(f, &mut rnd, 0.2, 0.2, 0.0, 6, 0.24, 0.18);
    });
    aside(&things, "things");
    // The top book's cover, hinged on the spine at its right: `book_up` opens it over to the right.
    let cover = new_parts(|| {
        fbox!(
            f,
            "cover",
            0.2,
            0.006,
            0.15,
            (0.1, top + 0.033, -0.15),
            M.book_a,
            bevel = 0.0015,
            bevel_segments = 1
        );
        fbox!(
            f,
            "cover_title",
            0.08,
            0.002,
            0.03,
            (0.1, top + 0.037, -0.15),
            M.book_d
        );
        fbox!(
            f,
            "cover_rule",
            0.16,
            0.0015,
            0.004,
            (0.1, top + 0.0365, -0.2),
            M.book_d
        );
    });
    let cover = hinged(join(&cover, "cover", None), f.at(0.2, top + 0.033, -0.15));
    let objs = settle_all(); // on its Model centre: the back on the wall face
    swing!(cover, "book_up", 170.0, seconds = 0.8, axis = Axis::Y);
    objs
}
