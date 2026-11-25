//! The rides: the seesaw, the spring rider and the merry-go-round. Each has a part that goes round while it is
//! played on, carrying its riders (`ride`).

use super::common::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{Axis, M4, P3, TAU, V3, cos, radians, sin, v3};
use artkit::motion::{Rock, Spin, rock, spin};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, rod, settle, soft, torus};

// A 3 m plank on an A-frame stand. The content's two places are at x = ±0.78 m (`SEAT_X`), at the seat height
// 0.56 m: the seats are there, level at rest, the handles 0.28 m in from them. The plank, its seats and handles are one part
// turning about the axle (0.46 m up), so the riders go up and down with it.
const AXLE: f64 = 0.46;
pub(super) const SEAT_X: f64 = 0.78;

pub fn seesaw() -> Vec<Obj> {
    let still = new_parts(|| {
        for z in [-0.17, 0.17] {
            // each side of the stand an A of bent tube, its feet on a base plate, a boss for the axle at its apex
            fsweep!(
                F,
                format!("leg{}", float_repr(z)),
                &kit::fillet!(&[(-0.22, 0.02, z), (0.0, AXLE, z), (0.22, 0.02, z)], 0.05),
                0.028,
                M.body,
                verts = 10
            );
            fbox!(
                F,
                format!("foot{}", float_repr(z)),
                0.56,
                0.04,
                0.08,
                (0, 0.02, z),
                M.body,
                bevel = 0.01,
                bevel_segments = 1
            );
            for x in [-0.24, 0.24] {
                fcyl!(
                    F,
                    format!("bolt{}{}", float_repr(z), float_repr(x)),
                    0.012,
                    0.012,
                    (x, 0.044, z),
                    M.steel,
                    verts = 6
                );
            }
            fcyl!(
                F,
                format!("boss{}", float_repr(z)),
                0.05,
                0.05,
                (0, AXLE, z),
                M.steel,
                rot = (90, 0, 0),
                verts = 16,
                bevel = 0.006,
                bevel_segments = 1
            );
            fsweep!(
                F,
                format!("crossbar{}", float_repr(z)),
                &[(-0.12, 0.2, z), (0.12, 0.2, z)],
                0.016,
                M.body,
                verts = 8
            );
        }
        rod!(
            "axle",
            F.at(0, AXLE, -0.21),
            F.at(0, AXLE, 0.21),
            0.022,
            material = M.steel,
            verts = 12
        );
        for z in [-0.202, 0.202] {
            fcyl!(
                F,
                format!("axle_nut{}", float_repr(z)),
                0.03,
                0.012,
                (0, AXLE, z),
                M.steel,
                rot = (90, 0, 0),
                verts = 6
            );
        }
        for x in [-1.1, 1.1] {
            // half-buried tyres the low end comes down onto
            revolve!(
                format!("bumper{}", float_repr(x)),
                &[
                    (0.0, 0.0),
                    (0.12, 0.0),
                    (0.13, 0.04),
                    (0.13, 0.13),
                    (0.12, 0.165),
                    (0.09, 0.175),
                    (0.06, 0.165),
                    (0.055, 0.15),
                    (0.0, 0.15)
                ],
                at = F.at(x, 0, 0),
                material = M.rubber,
                segments = 18
            );
        }
    });
    join(&still, "stand", Some(P3::from((0, 0, 0))));
    let parts = new_parts(|| {
        // the plank: rounded at its ends, a steel channel along its middle and straps over its ends
        fextrude!(
            F,
            "plank",
            &rounded_rect(2.5, 0.24, 0.1, 4),
            0.06,
            (0, 0.48, 0),
            M.plank,
            plane = Facing::Top,
            bevel = 0.012,
            bevel_segments = 1
        );
        fbox!(
            F,
            "channel",
            0.8,
            0.03,
            0.12,
            (0, 0.44, 0),
            M.steel,
            bevel = 0.004,
            bevel_segments = 1
        );
        fcyl!(
            F,
            "sleeve",
            0.045,
            0.14,
            (0, AXLE, 0),
            M.steel,
            rot = (90, 0, 0),
            verts = 16
        );
        fbox!(
            F,
            "bracket",
            0.16,
            0.04,
            0.12,
            (0, 0.47, 0),
            M.steel,
            bevel = 0.005,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            // a moulded seat, dished, a lip rising at its outer end; its top where the rider sits (0.56 m)
            soft!(
                F,
                format!("seat{s}"),
                0.34,
                0.04,
                0.28,
                (sf * SEAT_X, 0.532, 0),
                M.seat,
                round = 0.018,
                puff = 0.008,
                flat = 2
            );
            fsweep!(
                F,
                format!("seat_lip{s}"),
                &[
                    (sf * (SEAT_X + 0.17), 0.535, -0.12),
                    (sf * (SEAT_X + 0.19), 0.6, -0.1),
                    (sf * (SEAT_X + 0.19), 0.6, 0.1),
                    (sf * (SEAT_X + 0.17), 0.535, 0.12)
                ],
                0.02,
                M.seat,
                verts = 8,
                resolution = 2
            );
            soft!(
                F,
                format!("pad{s}"),
                0.14,
                0.03,
                0.18,
                (sf * 1.12, 0.435, 0),
                M.rubber,
                round = 0.012
            );
            // the handle: one bent tube rising either side of the plank and across, its grips where the hands go
            fsweep!(
                F,
                format!("handle{s}"),
                &kit::fillet!(
                    &[
                        (sf * (SEAT_X - 0.28), 0.49, -0.135),
                        (sf * (SEAT_X - 0.28), 0.82, -0.135),
                        (sf * (SEAT_X - 0.28), 0.82, 0.135),
                        (sf * (SEAT_X - 0.28), 0.49, 0.135)
                    ],
                    0.035
                ),
                0.02,
                M.rail,
                verts = 10
            );
            for z in [-1, 1] {
                let zf = z as f64;
                fbox!(
                    F,
                    format!("handle_plate{s}{z}"),
                    0.08,
                    0.06,
                    0.008,
                    (sf * (SEAT_X - 0.28), 0.49, zf * 0.128),
                    M.steel,
                    bevel = 0.002,
                    bevel_segments = 1
                );
                fsweep!(
                    F,
                    format!("grip{s}{z}"),
                    &[
                        (sf * (SEAT_X - 0.28), 0.82, zf * 0.035),
                        (sf * (SEAT_X - 0.28), 0.82, zf * 0.1)
                    ],
                    0.026,
                    M.rubber,
                    verts = 10
                );
            }
        }
    });
    let plank = hinged(join(&parts, "plank", None), F.at(0, AXLE, 0));
    settle!(centre = false);
    rock(
        plank,
        "seesaw",
        12.0,
        Rock {
            seconds: 1.4,
            axis: Axis::Y,
        },
    );
    meshes()
}

// A horse on a big coil spring. The saddle's top is the content's seat (0.62 m, 4 cm back: a rider's seat for a
// toddler to a six-year-old, EN 1176-6). The horse (all above the spring) rocks about the spring's top. Its parts are
// drawn for a spring top at HORSE_DRAWN and moved down onto SPRING_TOP (it was 16 cm higher, the saddle at 0.78).
const SPRING_TOP: f64 = 0.26;
const HORSE_DRAWN: f64 = 0.42;

/// Loft rings along the horse's length: (depth, half width, half height, centre height) each, an ellipse.
fn body_rings(rows: &[(f64, f64, f64, f64)], n: usize) -> Vec<Vec<P3>> {
    rows.iter()
        .map(|&(z, hw, hh, y)| {
            (0..n)
                .map(|k| {
                    F.at(
                        hw * cos(TAU * k as f64 / n as f64),
                        y + hh * sin(TAU * k as f64 / n as f64),
                        z,
                    )
                })
                .collect()
        })
        .collect()
}

pub fn spring_rider() -> Vec<Obj> {
    let still = new_parts(|| {
        // a round steel base plate bolted down, a plate for the coil on it, the coil swept in one piece
        fcyl!(
            F,
            "base",
            0.2,
            0.03,
            (0, 0.015, 0),
            M.steel,
            verts = 24,
            bevel = 0.008,
            bevel_segments = 1
        );
        for k in 0..6 {
            let a = TAU * k as f64 / 6.0;
            fcyl!(
                F,
                format!("base_bolt{k}"),
                0.012,
                0.012,
                (0.17 * cos(a), 0.034, 0.17 * sin(a)),
                M.rubber,
                verts = 6
            );
        }
        crate::eased(fcyl!(
            F,
            "base_plate",
            0.13,
            0.02,
            (0, 0.04, 0),
            M.steel,
            verts = 20
        ));
        let (turns, r) = (3.5, 0.1);
        let steps = (turns * 12.0) as i64;
        let coil: Vec<(f64, f64, f64)> = (0..=steps)
            .map(|i| {
                (
                    r * cos(TAU * i as f64 / 12.0),
                    0.06 + (SPRING_TOP - 0.08) * i as f64 / steps as f64,
                    r * sin(TAU * i as f64 / 12.0),
                )
            })
            .collect();
        fsweep!(F, "coil", &coil, 0.018, M.spring, verts = 6);
    });
    join(&still, "spring", Some(P3::from((0, 0, 0))));
    let parts = new_parts(|| {
        crate::eased(fcyl!(
            F,
            "top_plate",
            0.12,
            0.02,
            (0, HORSE_DRAWN, 0),
            M.steel,
            verts = 20
        ));
        fcyl!(
            F,
            "mount",
            0.05,
            0.1,
            (0, HORSE_DRAWN + 0.06, 0),
            M.steel,
            verts = 12
        );
        // the body: a barrel lofted from the rump to the chest
        loft!(
            "barrel",
            &body_rings(
                &[
                    (-0.33, 0.03, 0.03, 0.62),
                    (-0.3, 0.1, 0.11, 0.615),
                    (-0.22, 0.13, 0.15, 0.6),
                    (-0.05, 0.125, 0.15, 0.59),
                    (0.12, 0.125, 0.15, 0.6),
                    (0.24, 0.11, 0.13, 0.63),
                    (0.3, 0.06, 0.07, 0.65)
                ],
                14
            ),
            material = M.horse
        );
        // the neck and head: one sweep up from the chest, arching over into the head, tapering to the muzzle
        fsweep!(
            F,
            "neck",
            &[
                (0, 0.66, 0.18),
                (0, 0.8, 0.26),
                (0, 0.95, 0.29),
                (0, 1.0, 0.32),
                (0, 0.97, 0.39),
                (0, 0.92, 0.45)
            ],
            0,
            M.horse,
            profile = rounded_rect(0.16, 0.13, 0.06, 3),
            up = (0, -1, 0),
            resolution = 3,
            scale = [1.1, 0.95, 0.95, 1.05, 0.95, 0.75],
            angle = 70
        );
        fsphere!(
            F,
            "muzzle",
            0.07,
            (0, 0.91, 0.45),
            M.mane,
            scale = (1.0, 0.85, 1.0),
            segments = 12,
            rings = 7
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                F,
                format!("ear{s}"),
                &[
                    (sf * 0.045, 1.03, 0.3),
                    (sf * 0.055, 1.08, 0.29),
                    (sf * 0.05, 1.1, 0.28)
                ],
                0.022,
                M.horse,
                verts = 6,
                resolution = 2,
                scale = [1.0, 0.7, 0.15]
            );
            fsphere!(
                F,
                format!("eye{s}"),
                0.016,
                (sf * 0.075, 1.0, 0.37),
                M.eye,
                segments = 6,
                rings = 4
            );
            fcyl!(
                F,
                format!("nostril{s}"),
                0.01,
                0.01,
                (sf * 0.03, 0.9, 0.512),
                M.eye,
                rot = (90, 0, 0),
                verts = 8
            );
            // legs: bent at the knee, tucked under as a rocking horse's are, hooves at their ends
            for (z, dz) in [(0.18, 1.0), (-0.2, -1.0)] {
                fsweep!(
                    F,
                    format!("leg{s}{}", float_repr(z)),
                    &[
                        (sf * 0.08, 0.6, z),
                        (sf * 0.095, 0.5, z + dz * 0.04),
                        (sf * 0.1, 0.44, z + dz * 0.1)
                    ],
                    0.04,
                    M.horse,
                    verts = 10,
                    resolution = 2,
                    scale = [1.1, 0.85, 0.75]
                );
                fcyl!(
                    F,
                    format!("hoof{s}{}", float_repr(z)),
                    0.032,
                    0.035,
                    (sf * 0.1, 0.43, z + dz * 0.115),
                    M.eye,
                    verts = 10
                );
            }
        }
        // the mane in locks down the neck, a forelock, the tail swept out and down
        for i in 0..6 {
            let t = i as f64 / 5.0;
            let (y, z) = (1.02 - 0.3 * t, 0.29 - 0.12 * t);
            fsweep!(
                F,
                format!("mane{i}"),
                &[
                    (0.0, y, z),
                    (0.0, y + 0.03, z - 0.05),
                    (0.0, y - 0.01, z - 0.09)
                ],
                0,
                M.mane,
                profile = vec![
                    (-0.022, -0.012),
                    (0.022, -0.012),
                    (0.022, 0.012),
                    (-0.022, 0.012)
                ],
                up = (0, 0, 1),
                resolution = 2,
                scale = [1.0, 0.8, 0.3]
            );
        }
        fsphere!(
            F,
            "forelock",
            0.03,
            (0, 1.07, 0.34),
            M.mane,
            scale = (0.9, 0.7, 1.3),
            segments = 8,
            rings = 5
        );
        fsweep!(
            F,
            "tail",
            &[
                (0, 0.66, -0.31),
                (0, 0.655, -0.365),
                (0, 0.58, -0.405),
                (0, 0.5, -0.405)
            ],
            0.035,
            M.mane,
            verts = 10,
            resolution = 2,
            scale = [0.8, 1.1, 1.0, 0.6]
        );
        // the saddle: a padded seat (its top where the rider sits), a cantle behind, flaps down the sides
        soft!(
            F,
            "saddle",
            0.27,
            0.04,
            0.26,
            (0, 0.75, -0.04),
            M.saddle,
            round = 0.018,
            puff = 0.01,
            flat = 2,
            round_segments = 1
        );
        fsweep!(
            F,
            "cantle",
            &[(-0.1, 0.77, -0.17), (0.0, 0.8, -0.18), (0.1, 0.77, -0.17)],
            0.018,
            M.saddle,
            verts = 8,
            resolution = 2
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("flap{s}"),
                0.012,
                0.12,
                0.2,
                (s as f64 * 0.132, 0.69, -0.04),
                M.saddle,
                rot = (0, 0, s * 8),
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // the handles: a bar through the head, a rubber grip at each end; the foot pegs through the body
        fsweep!(
            F,
            "handles",
            &[(-0.11, 0.93, 0.24), (0.11, 0.93, 0.24)],
            0.014,
            M.steel,
            verts = 8
        );
        fsweep!(
            F,
            "pegs",
            &[(-0.15, 0.5, 0.06), (0.15, 0.5, 0.06)],
            0.014,
            M.steel,
            verts = 8
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                F,
                format!("handle_grip{s}"),
                &[(sf * 0.1, 0.93, 0.24), (sf * 0.16, 0.93, 0.24)],
                0.022,
                M.grip,
                verts = 10,
                scale = [1.0, 1.1]
            );
            fsphere!(
                F,
                format!("handle_end{s}"),
                0.024,
                (sf * 0.16, 0.93, 0.24),
                M.grip,
                segments = 8,
                rings = 5
            );
            fsweep!(
                F,
                format!("peg{s}"),
                &[(sf * 0.13, 0.5, 0.06), (sf * 0.18, 0.5, 0.06)],
                0.022,
                M.grip,
                verts = 10
            );
            fcyl!(
                F,
                format!("peg_end{s}"),
                0.03,
                0.02,
                (sf * 0.19, 0.5, 0.06),
                M.grip,
                rot = (0, 90, 0),
                verts = 10
            );
        }
    });
    let horse = join(&parts, "horse", None);
    horse.edit(|o| {
        o.mesh
            .transform(&M4::translation(v3(0, 0, SPRING_TOP - HORSE_DRAWN)))
    });
    let horse = hinged(horse, F.at(0, SPRING_TOP, 0));
    settle!(centre = false);
    rock(
        horse,
        "rock",
        10.0,
        Rock {
            seconds: 1.0,
            axis: Axis::X,
        },
    );
    meshes()
}

/// Points of the quadratic curve from `a` towards `b` to `c`.
fn bezier(a: (f64, f64, f64), b: (f64, f64, f64), c: (f64, f64, f64), steps: usize) -> Vec<V3> {
    let (a, b, c) = (V3::from(a), V3::from(b), V3::from(c));
    (0..=steps)
        .map(|i| {
            let t = i as f64 / steps as f64;
            a * ((1.0 - t) * (1.0 - t)) + b * (2.0 * (1.0 - t) * t) + c * (t * t)
        })
        .collect()
}

fn tuple(p: V3) -> (f64, f64, f64) {
    (p.x as f64, p.y as f64, p.z as f64)
}

// A 1.9 m platform, its top 0.26 m up (the content's seat: the riders sit on it), four bent rails to hold on to.
// The platform turns once in 3.5 s about the hub; the riders ride round with it.
pub fn merry_go_round() -> Vec<Obj> {
    let still = new_parts(|| {
        fcyl!(
            F,
            "ground_plate",
            0.3,
            0.03,
            (0, 0.015, 0),
            M.steel,
            verts = 24,
            bevel = 0.008,
            bevel_segments = 1
        );
        for k in 0..8 {
            let a = TAU * k as f64 / 8.0;
            fcyl!(
                F,
                format!("plate_bolt{k}"),
                0.012,
                0.01,
                (0.26 * cos(a), 0.034, 0.26 * sin(a)),
                M.rubber,
                verts = 6
            );
        }
        revolve!(
            "hub",
            &[
                (0.0, 0.03),
                (0.17, 0.03),
                (0.17, 0.05),
                (0.15, 0.06),
                (0.13, 0.13),
                (0.11, 0.15),
                (0.0, 0.15)
            ],
            at = F.at(0, 0, 0),
            material = M.steel,
            segments = 20
        );
    });
    join(&still, "hub", Some(P3::from((0, 0, 0))));
    let parts = new_parts(|| {
        // the deck: a disc on a dished steel skirt, a rail round its edge, the stripes, anti-slip ribs between them
        revolve!(
            "platform",
            &[
                (0.0, 0.18),
                (0.94, 0.18),
                (0.95, 0.19),
                (0.95, 0.25),
                (0.94, 0.26),
                (0.0, 0.26)
            ],
            at = F.at(0, 0, 0),
            material = M.platform,
            segments = 48,
            smooth = false
        );
        torus!(
            "edge",
            0.95,
            0.03,
            at = F.at(0, 0.22, 0),
            material = M.rail,
            major_segments = 48,
            minor_segments = 6
        );
        revolve!(
            "skirt",
            &[
                (0.12, 0.13),
                (0.3, 0.14),
                (0.62, 0.17),
                (0.8, 0.18),
                (0.0, 0.18)
            ],
            at = F.at(0, 0, 0),
            material = M.steel,
            segments = 24
        );
        for a in [45, -45] {
            fbox!(
                F,
                format!("stripe{a}"),
                0.22,
                0.008,
                1.72,
                (0, 0.263, 0),
                M.deck,
                rot = (0, 0, a)
            );
        }
        for k in 0..4 {
            let a = 90 * k;
            for j in [-1, 1] {
                let b = radians((a + j * 22) as f64);
                fbox!(
                    F,
                    format!("rib{k}{j}"),
                    0.03,
                    0.006,
                    0.5,
                    (0.6 * sin(b), 0.262, 0.6 * cos(b)),
                    M.deck,
                    rot = (0, 0, -(a + j * 22))
                );
            }
        }
        crate::eased(fcyl!(
            F,
            "centre_disc",
            0.2,
            0.012,
            (0, 0.266, 0),
            M.deck,
            verts = 24
        ));
        // the column: a turned post, a collar where the rails meet it and a knob on top
        revolve!(
            "column",
            &[
                (0.0, 0.26),
                (0.08, 0.26),
                (0.08, 0.28),
                (0.055, 0.3),
                (0.05, 0.84),
                (0.07, 0.86),
                (0.07, 0.92),
                (0.05, 0.94),
                (0.0, 0.94)
            ],
            at = F.at(0, 0, 0),
            material = M.rail,
            segments = 14
        );
        revolve!(
            "knob",
            &[
                (0.0, 0.93),
                (0.05, 0.93),
                (0.08, 0.96),
                (0.075, 1.0),
                (0.04, 1.035),
                (0.0, 1.04)
            ],
            at = F.at(0, 0, 0),
            material = M.deck,
            segments = 14
        );
        for k in 0..4 {
            let t = radians((45 + 90 * k) as f64);
            let (c, s) = (sin(t), cos(t));
            let pts = bezier(
                (0.05 * c, 0.9, 0.05 * s),
                (0.78 * c, 0.9, 0.78 * s),
                (0.78 * c, 0.25, 0.78 * s),
                14,
            );
            let pts: Vec<(f64, f64, f64)> = pts.into_iter().map(tuple).collect();
            fsweep!(F, format!("rail{k}"), &pts, 0.022, M.rail, verts = 10);
            fcyl!(
                F,
                format!("rail_foot{k}"),
                0.045,
                0.014,
                (0.78 * c, 0.266, 0.78 * s),
                M.steel,
                verts = 10,
                bevel = 0.004,
                bevel_segments = 1
            );
            // a rubber grip where the riders' hands go, down the rail beside them
            let g = bezier(
                (0.05 * c, 0.9, 0.05 * s),
                (0.78 * c, 0.9, 0.78 * s),
                (0.78 * c, 0.25, 0.78 * s),
                20,
            );
            let (g0, g1) = (g[10], g[15]);
            fsweep!(
                F,
                format!("rail_grip{k}"),
                &[tuple(g0), tuple(g1)],
                0.027,
                M.rubber,
                verts = 10
            );
        }
    });
    let platform = hinged(join(&parts, "platform", None), (0, 0, 0));
    settle!(centre = false);
    spin(
        platform,
        "spin",
        Spin {
            turns: 1.0,
            seconds: 3.5,
            axis: Axis::Z,
        },
    );
    meshes()
}
