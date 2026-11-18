//! The towel rail, the towel ladder, the laundry basket, the bath mat, and what is carried doing the laundry.

use super::M;
use super::common::{WALL, fillet, folded, lump};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, cos, cosd, radians, sin, sind};
use artkit::parts::{Settle, join_all, new_parts, settle_all, settle_with};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{cube, fbox, fextrude, fsweep, loft, revolve, soft, soft_box, sphere, sweep};

/// `G.settle` with its `centre` and `floor`.
fn settle(centre: bool, floor: bool) {
    settle_with(&meshes(), Settle { centre, floor });
}

// --- the towel rail: two chrome bars on the wall, towels over them ----------------------------------------

/// A towel hung over a bar at (bar_y, bar_z) in one piece: down the back, round the bar and down the front, its
/// flaps hanging a little apart below the fold; `hem` a band across the front flap near its foot.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn drape(
    f: &GameFrame,
    name: &str,
    x: f64,
    w: f64,
    bar_y: f64,
    bar_z: f64,
    drop_front: f64,
    drop_back: f64,
    material: Mat,
    t: f64,
    r: f64,
    hem: Option<Mat>,
    sway: f64,
) {
    let big_r = r + t / 2.0;
    let mut pts = vec![
        (x, bar_y - drop_back + 0.01, bar_z - big_r - 0.004),
        (x, bar_y - drop_back * 0.5, bar_z - big_r - 0.002),
    ];
    for a in (0..=180).rev().step_by(30) {
        let a = a as f64;
        pts.push((
            x,
            bar_y + big_r * sin(radians(a)),
            bar_z + big_r * cos(radians(a)),
        ));
    }
    pts.push((x, bar_y - drop_front * 0.5, bar_z + big_r + 0.002 + sway));
    pts.push((x, bar_y - drop_front + 0.01, bar_z + big_r + 0.004 + sway));
    fsweep!(
        f,
        name,
        &pts,
        0.0,
        material,
        profile = rounded_rect(w, t, t * 0.45, 2),
        up = (1, 0, 0),
        angle = 60.0
    );
    if let Some(hem) = hem {
        let y = bar_y - drop_front + 0.06;
        fbox!(
            f,
            format!("{name}_hem"),
            w + 0.002,
            0.03,
            t + 0.002,
            (x, y, bar_z + big_r + 0.0035 + sway * 0.9),
            hem
        );
    }
}

pub fn towel_rail() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let z = -0.36;
    new_parts(|| {
        for y in [1.2, 0.95] {
            fsweep!(
                f,
                format!("bar{}", float_repr(y)),
                &fillet(
                    &[
                        (-0.34, y, WALL + 0.012),
                        (-0.34, y, z),
                        (0.34, y, z),
                        (0.34, y, WALL + 0.012)
                    ],
                    0.03,
                    3
                ),
                0.0125,
                M.chrome,
                verts = 12
            );
        }
        for s in [-1, 1] {
            fextrude!(
                f,
                format!("plate{s}"),
                &rounded_rect(0.05, 0.36, 0.025, 3),
                0.012,
                (s as f64 * 0.34, 1.075, WALL + 0.006),
                M.chrome,
                bevel = 0.004,
                bevel_segments = 1
            );
            for y in [1.2, 0.95] {
                revolve!(
                    format!("rose{s}{}", float_repr(y)),
                    &[
                        (0.0, 0.0),
                        (0.02, 0.0),
                        (0.019, 0.006),
                        (0.014, 0.012),
                        (0.0, 0.013)
                    ],
                    at = f.at(s as f64 * 0.34, y, WALL + 0.011),
                    material = M.chrome,
                    segments = 14,
                    rot = (90, 0, 0)
                );
            }
        }
        drape(
            f,
            "bath_towel",
            -0.07,
            0.42,
            1.2,
            z,
            0.5,
            0.38,
            M.towel,
            0.012,
            0.0125,
            Some(M.towel_hem),
            0.0,
        );
        drape(
            f,
            "hand_towel",
            0.17,
            0.26,
            0.95,
            z,
            0.3,
            0.22,
            M.towel2,
            0.012,
            0.0125,
            Some(M.towel_hem2),
            0.0,
        );
    });
    join_all("towelRail");
    settle(false, false); // on the wall, at its real height
    meshes()
}

// --- the towel ladder: a birch ladder leaning on the wall, towels over three rungs --------------------------
const LADDER_TILT: f64 = 10.66; // degrees back from upright
const LADDER_FOOT: f64 = -0.1; // game depth of the rails' feet

/// The game depth of the ladder's front face at height `y`.
fn on_ladder(y: f64) -> f64 {
    LADDER_FOOT - y * sind(LADDER_TILT) / cosd(LADDER_TILT)
}

pub fn towel_ladder() -> Vec<Obj> {
    let f = &GameFrame::new(0.03); // its top rail on the wall face, not 3 cm into it
    let top = 1.7;
    new_parts(|| {
        for s in [-1, 1] {
            let s = s as f64;
            // a squared rail, its edges and its top rounded off
            let (a, b) = (
                (s * 0.24, 0.012, LADDER_FOOT - 0.02),
                (s * 0.24, top - 0.012, on_ladder(top - 0.012) - 0.02),
            );
            fsweep!(
                f,
                format!("rail{}", s as i32),
                &[a, b],
                0.0,
                M.birch,
                profile = rounded_rect(0.04, 0.04, 0.01, 2),
                up = (1, 0, 0),
                angle = 50.0
            );
            sphere!(
                format!("rail_cap{}", s as i32),
                0.02,
                at = f.at(s * 0.24, top - 0.012, on_ladder(top - 0.012) - 0.02),
                material = M.birch,
                segments = 10,
                rings = 6
            );
        }
        for (k, y) in [0.3, 0.63, 0.96, 1.29, 1.62].into_iter().enumerate() {
            let zz = on_ladder(y) - 0.02;
            fsweep!(
                f,
                format!("rung{k}"),
                &[(-0.24, y, zz), (0.24, y, zz)],
                0.0125,
                M.birch,
                verts = 10
            );
        }
        for (k, (y, m, hem)) in [
            (0.63, M.towel3, None),
            (0.96, M.towel, Some(M.towel_hem)),
            (1.29, M.towel2, Some(M.towel_hem2)),
        ]
        .into_iter()
        .enumerate()
        {
            drape(
                f,
                &format!("towel{k}"),
                0.01 * (k as f64 - 1.0),
                0.4,
                y,
                on_ladder(y) - 0.02,
                0.3,
                0.26,
                m,
                0.012,
                0.0125,
                hem,
                0.0,
            );
        }
    });
    join_all("towelLadder");
    settle(false, true);
    meshes()
}

// --- the laundry basket: a wicker tub of clean washing -----------------------------------------------------
pub fn laundry_basket() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let z = -0.2;
    let at = f.at(0, 0, z);
    let (bx, by) = (at.x, at.y);
    new_parts(|| {
        // the tub: woven rows (each a ring of over-and-under ripples, the next row's turned half a step), lofted up the
        // outside to the rim and down the inside to the floor
        let (n, rows) = (40, 10);
        let mut secs: Vec<Vec<P3>> = Vec::new();
        for i in 0..2 * rows + 1 {
            let t = i as f64 / (2 * rows) as f64;
            let r = 0.23 + 0.025 * t;
            let row = i / 2;
            let mut ring = Vec::new();
            for k in 0..n {
                let a = TAU * k as f64 / n as f64;
                // a row's middle bulges where its strand passes over a stake, sinks where it passes under; the next row
                // the other way round, so the weave checks; the rows' seams sink a little
                let rr = r * if i % 2 != 0 {
                    1.0 + 0.032 * cos(10.0 * a + PI * row as f64)
                } else {
                    1.0 - 0.01 * (0 < i && i < 2 * rows) as i32 as f64
                };
                ring.push(P3 {
                    x: bx + rr * cos(a),
                    y: by + 0.78 * rr * sin(a),
                    z: 0.33 * t,
                });
            }
            secs.push(ring);
        }
        for (y, r) in [(0.33, 0.236), (0.03, 0.222)] {
            secs.push(
                (0..n)
                    .map(|k| P3 {
                        x: bx + r * cos(TAU * k as f64 / n as f64),
                        y: by + 0.78 * r * sin(TAU * k as f64 / n as f64),
                        z: y,
                    })
                    .collect(),
            );
        }
        // (the lining inside, plain: only its top shows)
        loft!("basket", &secs, material = M.wicker, angle = 25.0);
        let rim: Vec<(f64, f64, f64)> = (0..40)
            .map(|k| {
                (
                    0.25 * cos(TAU * k as f64 / 40.0),
                    0.335,
                    z - 0.78 * 0.248 * sin(TAU * k as f64 / 40.0),
                )
            })
            .collect();
        fsweep!(
            f,
            "rim",
            &rim,
            0.0,
            M.wicker_rim,
            profile = rounded_rect(0.034, 0.034, 0.012, 2),
            closed = true,
            up = (0, 0, 1),
            angle = 60.0
        );
        // a band of plaited rim at the foot
        let foot: Vec<(f64, f64, f64)> = (0..32)
            .map(|k| {
                (
                    0.232 * cos(TAU * k as f64 / 32.0),
                    0.012,
                    z - 0.78 * 0.232 * sin(TAU * k as f64 / 32.0),
                )
            })
            .collect();
        fsweep!(
            f,
            "foot_band",
            &foot,
            0.012,
            M.wicker_rim,
            verts = 5,
            closed = true
        );
        for s in [-1, 1] {
            let s = s as f64;
            // a loop handle on each end
            let pts = [
                (s * 0.256, 0.24, z - 0.06),
                (s * 0.263, 0.29, z - 0.035),
                (s * 0.264, 0.305, z),
                (s * 0.263, 0.29, z + 0.035),
                (s * 0.256, 0.24, z + 0.06),
            ];
            fsweep!(
                f,
                format!("handle{}", s as i32),
                &pts,
                0.008,
                M.wicker_rim,
                verts = 6,
                resolution = 2
            );
        }
        // clean washing heaped in it: folded towels and a crumpled shirt
        folded(
            f,
            "wash0",
            -0.06,
            0.3,
            z - 0.02,
            0.3,
            0.24,
            0.06,
            M.towel,
            8.0,
        );
        folded(
            f,
            "wash1",
            0.07,
            0.33,
            z + 0.03,
            0.26,
            0.2,
            0.05,
            M.towel2,
            -14.0,
        );
        lump(
            f,
            "wash2",
            0.0,
            0.345,
            z - 0.07,
            0.12,
            M.wash_b,
            (1.1, 0.8, 0.35),
            3.0,
            0.0,
        );
    });
    join_all("laundryBasket");
    settle(false, true);
    meshes()
}

// --- what's carried: the basket of washing, a piece held up to fold ------------------------------------------

/// An oval wicker basket carried in both arms, heaped with washing: a tub widening to its rim, woven bands round
/// it, a hand hole at each end and the washing crumpled above the rim. Its base is where it rests on the palms.
pub fn washing_basket() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    new_parts(|| {
        revolve!(
            "basket",
            &[
                (0.0, 0.0),
                (0.165, 0.0),
                (0.2, 0.17),
                (0.19, 0.175),
                (0.158, 0.012),
                (0.0, 0.012)
            ],
            at = f.at(0, 0, 0),
            material = M.wicker_plain,
            segments = 24,
            scale = (1, 0.7, 1)
        );
        for (k, (y, r)) in [(0.05, 0.177), (0.1, 0.187), (0.15, 0.197)]
            .into_iter()
            .enumerate()
        {
            let lp: Vec<(f64, f64, f64)> = (0..24)
                .map(|i| {
                    (
                        r * cos(TAU * i as f64 / 24.0),
                        y,
                        0.7 * r * sin(TAU * i as f64 / 24.0),
                    )
                })
                .collect();
            fsweep!(
                f,
                format!("band{k}"),
                &lp,
                0.004,
                M.wicker_band,
                verts = 4,
                closed = true
            );
        }
        let rim: Vec<(f64, f64, f64)> = (0..28)
            .map(|i| {
                (
                    0.2 * cos(TAU * i as f64 / 28.0),
                    0.172,
                    0.7 * 0.2 * sin(TAU * i as f64 / 28.0),
                )
            })
            .collect();
        fsweep!(
            f,
            "rim",
            &rim,
            0.01,
            M.wicker_band,
            verts = 5,
            closed = true
        );
        for s in [-1, 1] {
            // the hand hole: a dark slot under the rim at each end
            fbox!(
                f,
                format!("hole{s}"),
                0.004,
                0.026,
                0.07,
                (s as f64 * 0.193, 0.135, 0),
                M.dark,
                rot = (0, 0, s * 9)
            );
        }
        // the washing heaped in it
        sphere!(
            "heap",
            0.17,
            at = f.at(0, 0.17, 0),
            material = M.wash_c,
            scale = (1, 0.66, 0.42),
            segments = 14,
            rings = 7
        );
        for (k, (m, x, z, r)) in [
            (M.wash_a, -0.07, 0.03, 0.08),
            (M.wash_b, 0.08, -0.02, 0.07),
            (M.wash_a, 0.03, 0.05, 0.06),
            (M.wash_b, -0.1, -0.05, 0.05),
        ]
        .into_iter()
        .enumerate()
        {
            lump(
                f,
                format!("wash{k}"),
                x,
                0.2 + 0.012 * k as f64,
                z,
                r,
                m,
                (1.1, 0.8, 0.5),
                (11 + k) as f64,
                (40 * k) as f64,
            );
        }
    });
    join_all("washingBasket");
    settle(false, true);
    meshes()
}

/// An armful of dry washing carried to be folded: towels folded loosely on top of each other, a crumpled shirt
/// on them and a sock. Its base is where it rests on the palms.
pub fn washing_armful() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    new_parts(|| {
        for (k, (m, x, y, z, w, d, h)) in [
            (M.wash_c, 0.0, 0.025, 0.0, 0.34, 0.24, 0.05),
            (M.wash_a, 0.01, 0.072, -0.01, 0.3, 0.22, 0.045),
            (M.wash_b, -0.015, 0.114, 0.01, 0.28, 0.2, 0.04),
        ]
        .into_iter()
        .enumerate()
        {
            soft!(
                f,
                format!("fold{k}"),
                w,
                h,
                d,
                (x, y, z),
                m,
                round = 0.018,
                puff = 0.006,
                flat = 1,
                round_segments = 2
            );
        }
        lump(
            f,
            "shirt",
            0.02,
            0.15,
            0.0,
            0.1,
            M.wash_c,
            (1.3, 0.9, 0.45),
            21.0,
            0.0,
        );
        lump(
            f,
            "sock",
            -0.11,
            0.15,
            0.06,
            0.035,
            M.wash_b,
            (1.4, 0.6, 0.5),
            22.0,
            30.0,
        );
    });
    join_all("washingArmful");
    settle(false, true);
    meshes()
}

/// A towel being folded in the hands: its near half lying across the palms, the far half lifted up and over
/// towards them, a woven band across its end. Its base is where it rests on the palms.
pub fn washing_piece() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (w, t) = (0.4, 0.01);
    // the towel's middle, from the near edge back under the palms and up round the fold to the lifted end
    let pts = [
        (0.0, t / 2.0, 0.12),
        (0.0, t / 2.0, -0.06),
        (0.0, 0.012, -0.1),
        (0.0, 0.04, -0.125),
        (0.0, 0.085, -0.12),
        (0.0, 0.13, -0.085),
        (0.0, 0.16, -0.035),
    ];
    new_parts(|| {
        fsweep!(
            f,
            "towel",
            &pts,
            0.0,
            M.towel,
            profile = rounded_rect(w, t, t * 0.45, 2),
            up = (1, 0, 0),
            angle = 60.0
        );
        fbox!(
            f,
            "hem",
            w + 0.002,
            t + 0.002,
            0.03,
            (0, t / 2.0, 0.095),
            M.towel_hem
        );
    });
    join_all("washingPiece");
    settle(false, true);
    meshes()
}

// --- the bath mat: a soft rounded mat with a raised pile -----------------------------------------------------
pub fn bath_mat() -> Vec<Obj> {
    new_parts(|| {
        soft_box!(
            "mat",
            (0.76, 0.5, 0.016),
            at = (0, 0, 0.008),
            material = M.mat,
            round = 0.008,
            round_segments = 2
        );
        soft_box!(
            "pile",
            (0.66, 0.4, 0.012),
            at = (0, 0, 0.0165),
            material = M.mat_pile,
            round = 0.006,
            puff = 0.003,
            round_segments = 2,
            flat = 3
        );
        for s in [-1, 1] {
            cube!(
                format!("stripe{s}"),
                (0.6, 0.025, 0.004),
                at = (0.0_f64, s as f64 * 0.15, 0.0225),
                material = M.mat,
                bevel = 0.0015,
                bevel_segments = 1
            );
        }
    });
    // a stitched seam round the mat's edge
    let seam: Vec<P3> = rounded_rect(0.72, 0.46, 0.02, 2)
        .into_iter()
        .map(|(u, v)| P3 {
            x: u,
            y: v,
            z: 0.0162,
        })
        .collect();
    sweep!(
        "seam",
        &seam,
        0.0016,
        material = M.mat_pile,
        verts = 3,
        closed = true
    );
    join_all("bathMat");
    settle_all();
    meshes()
}
