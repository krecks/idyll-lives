//! The café table and the bentwood café chair.

use super::common::{M, fluted};
use artkit::far::{is_low, low};
use artkit::forms::{by, rounded_rect};
use artkit::math::{P3, PI, TAU, cos, radians, sin};
use artkit::parts::{join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fcyl, fsweep, kit_scroll, loft, revolve, ring, settle};

// Heights the content names (community.json): kept here and checked against it.
pub const TABLE_TOP: f64 = 0.745; // cafeTable surface.height
pub const SEAT: f64 = 0.47; // cafeChair seat.height
pub const SEAT_FORWARD: f64 = 0.02; // cafeChair seat.forward

// --- the café table: 1x1, a round marble top on a cast-iron tripod foot -----------------------------
pub fn cafe_table() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let full = !is_low(); // its far version: the same build, coarser, without the veins and curls
    let still = new_parts(|| {
        // the marble top, its edge moulded: a square fillet over a rounded nose
        revolve!(
            "top",
            &[
                (0.0, TABLE_TOP - 0.03),
                (0.29, TABLE_TOP - 0.03),
                (0.305, TABLE_TOP - 0.024),
                (0.31, TABLE_TOP - 0.012),
                (0.308, TABLE_TOP - 0.004),
                (0.3, TABLE_TOP),
                (0.0, TABLE_TOP)
            ],
            material = M.marble,
            segments = low(32, 16)
        );
        // grey veins wandering across the marble
        for (k, (x0, z0, x1, z1)) in [
            (-0.24, -0.1, 0.2, 0.17),
            (-0.05, -0.26, 0.12, 0.05),
            (0.08, 0.2, 0.26, -0.02),
        ]
        .into_iter()
        .enumerate()
        {
            if !full {
                break;
            }
            let kf = k as f64;
            let mut pts = Vec::new();
            for j in 0..6 {
                let fr = j as f64 / 5.0;
                pts.push((
                    x0 + (x1 - x0) * fr + 0.025 * sin(7.0 * fr + kf),
                    TABLE_TOP + 0.0006,
                    z0 + (z1 - z0) * fr + 0.02 * cos(5.0 * fr + (2 * k) as f64),
                ));
            }
            fsweep!(
                f,
                format!("vein{k}"),
                &pts,
                0,
                M.vein,
                profile = rounded_rect(0.001, 0.006 - 0.002 * (k % 2) as f64, 0.0, 4),
                resolution = 2,
                scale = by(|w| 0.4 + 0.6 * sin(PI * w))
            );
        }
        revolve!(
            "collar",
            &[
                (0.0, TABLE_TOP - 0.03),
                (0.07, TABLE_TOP - 0.03),
                (0.07, TABLE_TOP - 0.04),
                (0.05, TABLE_TOP - 0.05),
                (0.03, TABLE_TOP - 0.075),
                (0.0, TABLE_TOP - 0.075)
            ],
            material = M.iron,
            segments = low(14, 6)
        );
        // the spider under the top: three scrolled arms
        for i in 0..3 {
            let arm = fsweep!(
                f,
                format!("spider{i}"),
                &[
                    (0, TABLE_TOP - 0.06, 0.04),
                    (0, TABLE_TOP - 0.04, 0.11),
                    (0, TABLE_TOP - 0.031, 0.18)
                ],
                0,
                M.iron,
                profile = rounded_rect(0.012, 0.014, 0.0, 4),
                up = (1, 0, 0),
                resolution = low(2, 1)
            );
            arm.set_rotation_euler_axis(2, radians((i * 120 + 60) as f64));
            if full {
                let curl = kit_scroll!(
                    f,
                    format!("spider_curl{i}"),
                    (0, TABLE_TOP - 0.031, 0.18),
                    (TABLE_TOP - 0.05, 0.18),
                    M.iron,
                    turns = -0.9,
                    end = 0.35,
                    t = 0.012,
                    w = 0.012,
                    steps = 6,
                    profile = rounded_rect(0.012, 0.012, 0.0, 4)
                );
                curl.set_rotation_euler_axis(2, radians((i * 120 + 60) as f64));
            }
        }
        // the column: fluted, a turned knop, flaring into the hub
        if full {
            loft!(
                "column",
                &[
                    fluted(0.024, 0.2, 8, 0.15, 3),
                    fluted(0.022, TABLE_TOP - 0.07, 8, 0.15, 3)
                ],
                material = M.iron
            );
        } else {
            fcyl!(
                f,
                "column",
                0.023,
                TABLE_TOP - 0.27,
                (0, (TABLE_TOP + 0.13) / 2.0, 0),
                M.iron,
                verts = 6
            );
        }
        revolve!(
            "knop",
            &[
                (0.0, 0.29),
                (0.026, 0.29),
                (0.04, 0.31),
                (0.045, 0.34),
                (0.04, 0.37),
                (0.026, 0.39),
                (0.0, 0.39)
            ],
            material = M.iron,
            segments = low(12, 6)
        );
        revolve!(
            "hub",
            &[
                (0.0, 0.08),
                (0.07, 0.08),
                (0.075, 0.1),
                (0.06, 0.12),
                (0.04, 0.16),
                (0.03, 0.21),
                (0.0, 0.21)
            ],
            material = M.iron,
            segments = low(14, 6)
        );
        // three curved legs sweeping out and down, each ending in a scrolled toe on a round foot
        for i in 0..3 {
            let a = radians((i * 120) as f64);
            let pts: Vec<(f64, f64, f64)> = [
                (0.04, 0.16),
                (0.1, 0.13),
                (0.17, 0.07),
                (0.23, 0.03),
                (0.265, 0.018),
            ]
            .into_iter()
            .map(|(r, y)| (r * sin(a), y, r * cos(a)))
            .collect();
            fsweep!(
                f,
                format!("leg{i}"),
                &pts,
                0.018,
                M.iron,
                verts = low(6, 4),
                resolution = low(2, 1),
                scale = [1.1, 1.0, 0.85, 0.8, 0.85]
            );
            fcyl!(
                f,
                format!("foot{i}"),
                0.03,
                0.018,
                (0.27 * sin(a), 0.009, 0.27 * cos(a)),
                M.iron,
                verts = low(10, 6)
            );
        }
    });
    join(&still, "table", Some(P3::from((0, 0, 0))));
    settle!(centre = false);
    meshes()
}

// --- the café chair: 1x1, a bentwood chair with a cane seat, the sitter facing the front ------------
pub fn cafe_chair() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let c = SEAT_FORWARD; // the seat's centre, 2 cm forward
    let still = new_parts(|| {
        // the seat: a bentwood ring, the woven cane dished a little inside it
        fcyl!(
            f,
            "seat",
            0.2,
            0.03,
            (0, SEAT - 0.042, c),
            M.bentwood,
            verts = 24
        );
        let P3 { x: bx, y: by_, .. } = f.at(0, 0, c);
        loft!(
            "cane",
            &[
                ring!(0.19, 0.19, z = SEAT - 0.03, verts = 24, at = (bx, by_)),
                ring!(0.19, 0.19, z = SEAT - 0.012, verts = 24, at = (bx, by_)),
                ring!(0.12, 0.12, z = SEAT - 0.016, verts = 24, at = (bx, by_)),
                ring!(0.001, 0.001, z = SEAT - 0.018, verts = 24, at = (bx, by_))
            ],
            material = M.cane
        );
        let rim: Vec<(f64, f64, f64)> = (0..24)
            .map(|i| {
                let i = i as f64;
                (
                    0.198 * cos(TAU * i / 24.0),
                    SEAT - 0.012,
                    c + 0.198 * sin(TAU * i / 24.0),
                )
            })
            .collect();
        fsweep!(
            f,
            "seat_rim",
            &rim,
            0.012,
            M.bentwood,
            verts = 6,
            closed = true
        );
        // the front legs, tapering and splaying a little, and the ring that braces all four
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                format!("leg{s}"),
                &[
                    (sf * 0.13, SEAT - 0.04, c + 0.13),
                    (sf * 0.14, 0.22, c + 0.148),
                    (sf * 0.145, 0.0, c + 0.16)
                ],
                0.017,
                M.bentwood,
                verts = 8,
                resolution = 2,
                scale = [1.0, 0.9, 0.75]
            );
            // the little bent brackets under the seat, from each front leg to the seat ring
            fsweep!(
                f,
                format!("knee{s}"),
                &[
                    (sf * 0.133, SEAT - 0.2, c + 0.135),
                    (sf * 0.11, SEAT - 0.12, c + 0.1),
                    (sf * 0.09, SEAT - 0.06, c + 0.09)
                ],
                0.008,
                M.bentwood,
                verts = 6,
                resolution = 2
            );
        }
        let ring_pts: Vec<(f64, f64, f64)> = (0..24)
            .map(|i| {
                let i = i as f64;
                (
                    0.212 * cos(TAU * i / 24.0),
                    0.22,
                    -0.0124 + 0.212 * sin(TAU * i / 24.0),
                )
            })
            .collect();
        fsweep!(
            f,
            "leg_ring",
            &ring_pts,
            0.011,
            M.bentwood,
            verts = 6,
            closed = true
        );
        // one bent piece: the back legs rising past the seat into the bow of the back, tapering to the feet
        let side = [
            (0.152, 0.0, -0.192),
            (0.15, 0.22, -0.178),
            (0.152, 0.44, -0.165),
            (0.148, 0.66, -0.19),
            (0.14, 0.82, -0.215),
            (0.09, 0.915, -0.226),
        ];
        let bow: Vec<(f64, f64, f64)> = side
            .iter()
            .map(|&(x, y, z)| (-x, y, z))
            .chain([(0.0, 0.935, -0.23)])
            .chain(side.iter().rev().copied())
            .collect();
        fsweep!(
            f,
            "bow",
            &bow,
            0.016,
            M.bentwood,
            verts = 8,
            resolution = 3,
            scale = by(|w| 0.75 + 0.25 * 1.0_f64.min(4.0 * w.min(1.0 - w)))
        );
        // the inner loop of the back
        let side = [
            (0.075, SEAT - 0.01, -0.17),
            (0.08, 0.6, -0.186),
            (0.065, 0.71, -0.2),
        ];
        let lp: Vec<(f64, f64, f64)> = side
            .iter()
            .map(|&(x, y, z)| (-x, y, z))
            .chain([(0.0, 0.75, -0.206)])
            .chain(side.iter().rev().copied())
            .collect();
        fsweep!(f, "loop", &lp, 0.012, M.bentwood, verts = 6, resolution = 3);
    });
    join(&still, "chair", Some(P3::from((0, 0, 0))));
    settle!(centre = false);
    meshes()
}
