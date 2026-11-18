//! L2, the small seats: the recliner (it reclines), the rocking chair (it rocks), the pouf, the ottoman and the
//! floor cushion (their tops sink in) and the bean bag (it flattens).

use super::common::{Cushion, cushion, leg, panel, plain, press, round_rings, seat, settled};
use super::{F, M};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::math::{Axis, P3, V3, cos, exp, pow, radians, sin, sqrt};
use artkit::motion::{Rock, Squash, path, rock, squash};
use artkit::parts::{Settle, hinged, join, measure, new_parts, settle_with, turn};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsweep, jitter, loft, ring, soft, sphere, sweep, swing};

// The recliner: its seat and back tip back 25° about the sitter's hips and glide 8 cm forward as they do
// (`recline`, a wall-hugger's), carrying them (`ride`), and its footrest swings up 80° from the front of the
// base (`footrest_up`), for relax and doze. A high back with a head cushion (1.0 m) over a 0.80 m deep base and
// footrest: reclined, the head rests on it (it stopped at the shoulder blades: M-living-04). It stands 11 cm off
// the wall's face; the glide keeps the back's top out of the wall when tipped.
const RC_Z0: f64 = -0.32; // the base, back to front
const RC_Z1: f64 = 0.48;
const RC_HIPS: (f64, f64) = (0.44, 0.12); // (height, depth): content seat 0.44, forward 0.12
const RC_GLIDE: f64 = 0.08; // how far the seat and back come forward reclined

fn floor() {
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
}

pub fn recliner() -> Vec<Obj> {
    let (w, aw) = (0.92, 0.17);
    let mid = (RC_Z0 + RC_Z1) / 2.0;
    let still = new_parts(|| {
        panel(
            "base",
            w - 0.02,
            0.3,
            RC_Z1 - RC_Z0,
            (0.0, 0.17, mid),
            M.tan,
            0.04,
            0.0,
            Some(0.005),
            plain(0.04),
        ); // skirted to the floor
        fbox!(
            F,
            "plinth",
            w - 0.04,
            0.02,
            RC_Z1 - RC_Z0 - 0.04,
            (0, 0.01, mid),
            M.tan
        );
        for s in [-1, 1] {
            let x = f64::from(s) * (w / 2.0 - aw / 2.0);
            // a pillow-top arm: a welted side panel under a stuffed pad that rolls over its front
            panel(
                &format!("arm{s}"),
                aw,
                0.26,
                RC_Z1 - RC_Z0,
                (x, 0.45, mid),
                M.tan,
                0.05,
                0.0,
                None,
                plain(0.05),
            );
            cushion(
                &format!("arm_pad{s}"),
                aw - 0.01,
                0.1,
                RC_Z1 - RC_Z0 - 0.01,
                (x, 0.57, mid),
                M.tan,
                Cushion {
                    puff: 0.012,
                    piping: None,
                    flat: Some(2),
                    ..Cushion::default()
                },
            );
        }
    });
    join(&still, "chair", Some((0, 0, 0).into()));
    let sw = w - 2.0 * aw;
    let body = new_parts(|| {
        // the seat cushion and the back, which tip together
        cushion(
            "seat",
            sw - 0.02,
            0.13,
            RC_Z1 - RC_Z0 - 0.22,
            (0.0, 0.385, (RC_Z0 + 0.2 + RC_Z1) / 2.0),
            M.tan_cushion,
            Cushion {
                puff: 0.015,
                keep_top: true,
                ..Cushion::default()
            },
        );
        panel(
            "back",
            w - 2.0 * aw + 0.02,
            0.66,
            0.18,
            (0.0, 0.65, RC_Z0 + 0.11),
            M.tan,
            0.06,
            0.0,
            None,
            plain(0.06),
        );
        // a bustle back: a lumbar roll and a back cushion stacked under the head cushion
        let leaning = |deg: i32, puff: f64| Cushion {
            rot: P3::from((deg, 0, 0)),
            puff,
            piping: None,
            flat: Some(2),
            rs: Some(1),
            ..Cushion::default()
        };
        cushion(
            "lumbar",
            sw - 0.03,
            0.2,
            0.15,
            (0.0, 0.52, RC_Z0 + 0.27),
            M.tan_cushion,
            leaning(-6, 0.03),
        );
        cushion(
            "back_cushion",
            sw - 0.03,
            0.24,
            0.15,
            (0.0, 0.72, RC_Z0 + 0.255),
            M.tan_cushion,
            leaning(-11, 0.03),
        );
        cushion(
            "head_cushion",
            sw - 0.05,
            0.16,
            0.14,
            (0.0, 0.905, RC_Z0 + 0.24),
            M.tan_cushion,
            leaning(-9, 0.025),
        );
    });
    let seat = join(&body, "seat", Some(F.at(0, RC_HIPS.0, RC_HIPS.1)));
    // the footrest, built up (open), hinged at the front of the base
    let hinge = (0.3, RC_Z1 - 0.09); // folded down, its padded face stands 2 cm proud of the base's front
    let foot = new_parts(|| {
        cushion(
            "footrest",
            sw - 0.04,
            0.11,
            0.26,
            (0.0, hinge.0 + 0.03, hinge.1 + 0.14),
            M.tan_cushion,
            Cushion {
                puff: 0.0,
                piping: None,
                ..Cushion::default()
            },
        );
    });
    let footrest = hinged(join(&foot, "footrest", None), F.at(0, hinge.0, hinge.1));
    turn(footrest, 90.0, Axis::X); // folded down flat against the front of the base at rest
    floor();
    path(
        seat,
        "recline",
        &[
            (0.0, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),
            (1.0, (0.0, -RC_GLIDE, 0.0), (-25.0, 0.0, 0.0)),
        ],
        false,
    );
    swing!(
        footrest,
        "footrest_up",
        -80.0,
        seconds = 1.0,
        axis = Axis::X
    );
    Vec::new()
}

// The rocking chair rocks ±6° on its rockers (`rock`, a loop while sat in), the sitter with it (`ride`). It turns
// about the rockers' centre of curvature (1 m radius), so the rockers roll on the floor.
const RK_R: f64 = 1.0;
const RK_Z: f64 = 0.05;

pub fn rocking_chair() -> Vec<Obj> {
    let stick = rounded_rect(0.03, 0.045, 0.012, 2);
    let chair = new_parts(|| {
        // a saddled seat board, rounded at the front
        let seat = smooth_outline(
            &[
                (-0.25, -0.19),
                (0.25, -0.19),
                (0.25, 0.2),
                (0.2, 0.25),
                (-0.2, 0.25),
                (-0.25, 0.2),
            ],
            2,
            true,
        );
        fextrude!(
            F,
            "seat_board",
            &seat,
            0.04,
            (0, 0.44, 0.04),
            M.rocker,
            plane = Facing::Top,
            bevel = 0.012,
            bevel_segments = 2
        );
        cushion(
            "seat_cushion",
            0.43,
            0.05,
            0.39,
            (0.0, 0.48, 0.05),
            M.rocker_cushion,
            Cushion {
                puff: 0.012,
                piping: Some(0.005),
                keep_top: true,
                ..Cushion::default()
            },
        );
        for x in [-0.22, 0.22] {
            let xs = float_repr(x);
            // the rocker: one bent strip on its 1 m arc, its ends curling up a little
            let pts: Vec<(f64, f64, f64)> = (0..9)
                .map(|i| {
                    let a = radians(-26.0 + f64::from(i * 52) / 8.0);
                    (x, RK_R + 0.025 - RK_R * cos(a), RK_Z + RK_R * sin(a))
                })
                .collect();
            fsweep!(
                F,
                format!("rocker{xs}"),
                &pts,
                0.0,
                M.rocker,
                profile = rounded_rect(0.03, 0.045, 0.01, 1),
                up = (1, 0, 0),
                resolution = 2,
                scale = [0.7, 0.9, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9, 0.7]
            );
            // turned legs and arm posts: swelling and narrowing along their length
            let turned = [0.8, 1.0, 0.85, 1.05, 0.9, 0.8];
            let front: Vec<(f64, f64, f64)> = (0..6)
                .map(|t| {
                    let t = f64::from(t);
                    (x, 0.06 + 0.38 * t / 5.0, 0.26 - 0.04 * t / 5.0)
                })
                .collect();
            fsweep!(
                F,
                format!("front_leg{xs}"),
                &front,
                0.018,
                M.rocker,
                verts = 10,
                scale = turned
            );
            fsweep!(
                F,
                format!("back_leg{xs}"),
                &[
                    (x, 0.06, -0.18),
                    (x * 1.02, 0.45, -0.22),
                    (x * 1.04, 0.75, -0.265),
                    (x * 1.05, 1.0, -0.3)
                ],
                0.018,
                M.rocker,
                verts = 10,
                resolution = 2,
                scale = [0.9, 1.0, 0.95, 0.85]
            );
            let post: Vec<(f64, f64, f64)> = (0..5)
                .map(|t| (x * 1.08, 0.44 + 0.22 * f64::from(t) / 4.0, 0.2))
                .collect();
            fsweep!(
                F,
                format!("arm_post{xs}"),
                &post,
                0.016,
                M.rocker,
                verts = 10,
                scale = [0.85, 1.05, 0.8, 1.0, 0.85]
            );
            // the arm: a paddle, wider at the hand
            let paddle = smooth_outline(
                &[
                    (-0.018, -0.2),
                    (0.018, -0.2),
                    (0.025, 0.15),
                    (0.0, 0.235),
                    (-0.025, 0.15),
                ],
                2,
                true,
            );
            fextrude!(
                F,
                format!("arm{xs}"),
                &paddle,
                0.026,
                (x * 1.08, 0.665, 0.0),
                M.rocker,
                plane = Facing::Top,
                bevel = 0.008,
                bevel_segments = 2
            );
            // a stretcher between the legs, low down
            fsweep!(
                F,
                format!("stretcher{xs}"),
                &[(x, 0.2, 0.245), (x * 1.01, 0.2, -0.19)],
                0.012,
                M.rocker,
                verts = 8,
                scale = [1.0, 1.0]
            );
            // the cushion's ties round the back legs
            fsweep!(
                F,
                format!("tie{xs}"),
                &[
                    (x * 0.9, 0.49, -0.13),
                    (x * 1.0, 0.49, -0.17),
                    (x * 1.04, 0.485, -0.205),
                    (x * 1.04, 0.45, -0.21)
                ],
                0.004,
                M.rocker_cushion,
                verts = 5,
                resolution = 2
            );
        }
        // spindles, bowed a little to the sitter's back, and a curved crest rail over them
        for i in 0..6 {
            let xx = -0.15 + f64::from(i) * 0.06;
            fsweep!(
                F,
                format!("spindle{i}"),
                &[
                    (xx, 0.46, -0.17),
                    (xx, 0.72, -0.21),
                    (xx * 1.03, 0.98, -0.28)
                ],
                0.0095,
                M.rocker,
                verts = 8,
                resolution = 2,
                scale = [1.0, 0.85, 1.0]
            );
        }
        fsweep!(
            F,
            "crest",
            &[
                (-0.245, 0.985, -0.3),
                (-0.12, 0.996, -0.285),
                (0.0, 1.0, -0.28),
                (0.12, 0.996, -0.285),
                (0.245, 0.985, -0.3)
            ],
            0.0,
            M.rocker,
            profile = rounded_rect(0.07, 0.035, 0.014, 2),
            up = (0, 0, 1),
            resolution = 2
        );
        fsweep!(
            F,
            "back_rail",
            &[(-0.22, 0.46, -0.172), (0.22, 0.46, -0.172)],
            0.0,
            M.rocker,
            profile = &stick,
            up = (0, 0, 1)
        );
    });
    let chair = join(&chair, "chair", Some(F.at(0, RK_R + 0.025, RK_Z)));
    floor();
    rock(
        chair,
        "rock",
        6.0,
        Rock {
            seconds: 1.4,
            axis: Axis::X,
        },
    );
    Vec::new()
}

/// The leather pouf (the old part): dark leather welted round its top and foot on short tapered legs; the buttoned
/// top, its tufts pressed in, sinks in.
pub fn pouf() -> Vec<Obj> {
    let still = new_parts(|| {
        panel(
            "body",
            0.72,
            0.34,
            0.5,
            (0.0, 0.26, 0.0),
            M.pouf_leather,
            0.04,
            0.0,
            Some(0.006),
            plain(0.04),
        );
        for x in [-0.3, 0.3] {
            for z in [-0.19, 0.19] {
                leg(
                    &format!("leg{}{}", float_repr(x), float_repr(z)),
                    x,
                    z,
                    0.09,
                    M.walnut_warm,
                    0.02,
                    (0.006_f64.copysign(x), 0.006_f64.copysign(z)),
                    10,
                );
            }
        }
    });
    join(&still, "pouf", Some((0, 0, 0).into()));
    let cushions = || {
        soft!(
            F,
            "top",
            0.7,
            0.05,
            0.48,
            (0, 0.435, 0),
            M.pouf_leather,
            round = 0.024,
            puff = 0.012,
            piping = 0.005,
            tufts = (3, 2),
            tuft = 0.02,
            button = M.pouf_button,
            resting = true
        );
        0.46
    };
    settled(seat(cushions, 0.41, 0))
}

/// The tufted ottoman (the old part): a navy drum bellied a touch, welted at its top and foot, on walnut bun feet;
/// its domed top, buttoned into seven tufts, sinks in.
pub fn ottoman() -> Vec<Obj> {
    let still = new_parts(|| {
        let drum: [&[f64]; 7] = [
            &[0.04, 0.29],
            &[0.048, 0.302],
            &[0.08, 0.307],
            &[0.22, 0.31],
            &[0.36, 0.307],
            &[0.392, 0.302],
            &[0.4, 0.29],
        ];
        loft!(
            "drum",
            &round_rings(&drum, 48, 2.0, (0.0, 0.0)),
            material = M.navy
        );
        for y in [0.046, 0.394] {
            sweep!(
                format!("welt{}", float_repr(y)),
                &ring!(0.3, 0.3, z = y, verts = 48),
                0.007,
                material = M.navy,
                verts = 5,
                closed = true
            );
        }
        for (x, z) in [(-0.2, -0.2), (0.2, -0.2), (-0.2, 0.2), (0.2, 0.2)] {
            let foot: [&[f64]; 5] = [
                &[0.0, 0.018],
                &[0.008, 0.026],
                &[0.022, 0.03],
                &[0.036, 0.026],
                &[0.045, 0.02],
            ];
            loft!(
                format!("foot{}{}", float_repr(x), float_repr(z)),
                &round_rings(&foot, 12, 2.0, (-x, -z)),
                material = M.walnut_warm
            );
        }
    });
    join(&still, "ottoman", Some((0, 0, 0).into()));
    let cushions = || {
        let rows: [&[f64]; 12] = [
            &[0.38, 0.296],
            &[0.392, 0.302],
            &[0.415, 0.3],
            &[0.428, 0.288],
            &[0.436, 0.262],
            &[0.44, 0.23],
            &[0.442, 0.19],
            &[0.443, 0.15],
            &[0.443, 0.11],
            &[0.443, 0.07],
            &[0.443, 0.035],
            &[0.443, 0.006],
        ];
        let top = loft!(
            "top",
            &round_rings(&rows, 48, 2.0, (0.0, 0.0)),
            material = M.navy
        );
        let mut tufts = vec![(0.0, 0.0)];
        tufts.extend((0..360).step_by(60).map(|a| {
            let a = radians(f64::from(a));
            (0.15 * cos(a), 0.15 * sin(a))
        }));
        press(top, &tufts, 0.014, 0.035, 0.42);
        for (i, &(x, y)) in tufts.iter().enumerate() {
            sphere!(
                format!("button{i}"),
                0.0125,
                at = (x, y, 0.432),
                material = M.navy_button,
                scale = (1, 1, 0.7),
                segments = 8,
                rings = 5
            );
        }
        0.44
    };
    settled(seat(cushions, 0.38, 0))
}

/// The floor cushion (the old part): a round mustard cushion, boxed, welted round both seams, its domed top tufted to
/// a button in the middle; all of it sinks.
pub fn floor_cushion() -> Vec<Obj> {
    let cushions = || {
        let rows: [&[f64]; 14] = [
            &[0.0, 0.27],
            &[0.006, 0.296],
            &[0.016, 0.306],
            &[0.05, 0.31],
            &[0.09, 0.306],
            &[0.102, 0.296],
            &[0.114, 0.27],
            &[0.124, 0.23],
            &[0.132, 0.18],
            &[0.137, 0.13],
            &[0.139, 0.085],
            &[0.139, 0.045],
            &[0.139, 0.015],
            &[0.139, 0.004],
        ];
        let body = loft!(
            "cushion",
            &round_rings(&rows, 40, 2.0, (0.0, 0.0)),
            material = M.mustard
        );
        press(body, &[(0.0, 0.0)], 0.024, 0.06, 0.1);
        for y in [0.014, 0.1] {
            sweep!(
                format!("piping{}", float_repr(y)),
                &ring!(0.299, 0.299, z = y, verts = 40),
                0.008,
                material = M.mustard_piping,
                verts = 6,
                closed = true
            );
        }
        sphere!(
            "button",
            0.02,
            at = (0, 0, 0.117),
            material = M.mustard_button,
            scale = (1, 1, 0.6),
            segments = 10,
            rings = 6
        );
        0.145
    };
    settled(seat(cushions, 0.0, 0))
}

/// The bean bag (the old part): a slumped, wrinkled bag rising into a back, a dent to sit in with its seat panel, a
/// carry strap; it flattens 15 % when flopped into (`squash`).
pub fn bean_bag() -> Vec<Obj> {
    let parts = new_parts(|| {
        // rings from the floor up (height, half-width, half-depth, centre's way back), the top slumped back
        let rows: [[f64; 4]; 13] = [
            [0.0, 0.33, 0.33, 0.0],
            [0.025, 0.395, 0.4, 0.0],
            [0.08, 0.42, 0.425, 0.0],
            [0.16, 0.415, 0.42, 0.005],
            [0.25, 0.4, 0.405, 0.005],
            [0.29, 0.39, 0.38, 0.03],
            [0.34, 0.36, 0.33, 0.07],
            [0.43, 0.32, 0.245, 0.14],
            [0.52, 0.28, 0.185, 0.185],
            [0.61, 0.22, 0.13, 0.215],
            [0.675, 0.14, 0.08, 0.23],
            [0.706, 0.05, 0.03, 0.235],
            [0.712, 0.008, 0.006, 0.235],
        ];
        let mut all: Vec<[f64; 4]> = rows
            .windows(2)
            .map(|w| [0, 1, 2, 3].map(|i| (w[0][i] + w[1][i]) / 2.0))
            .collect();
        all.extend(rows);
        all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let all: Vec<&[f64]> = all.iter().map(|r| r.as_slice()).collect();
        let bag = loft!(
            "bag",
            &round_rings(&all, 48, 2.0, (0.0, 0.0)),
            material = M.bag
        );
        bag.edit(|o| {
            // the dent the sitter has left, in front of the back
            for v in &mut o.mesh.verts {
                let (x, y, z) = (v.x as f64, v.y as f64, v.z as f64);
                let d = exp(-(pow(x / 0.24, 2.0) + pow((y + 0.05) / 0.2, 2.0)));
                if z > 0.18 {
                    v.z = (z - 0.2 * d * 1.0_f64.min((z - 0.18) / 0.12)) as f32;
                }
            }
            // the seat panel sewn into the dent, in its own colour
            let slot = o.mesh.slot(M.bag_seat);
            let oval = |x: f64, y: f64| pow(x / 0.2, 2.0) + pow((y + 0.07) / 0.17, 2.0);
            let mut seat = vec![false; o.mesh.faces.len()];
            for f in 0..o.mesh.faces.len() {
                let corners = o.mesh.corners(f).to_vec();
                let mut c = V3::ZERO;
                for &i in &corners {
                    c += o.mesh.verts[i as usize];
                }
                let c = c * (1.0 / corners.len() as f32);
                let n = o.mesh.face_normal(f);
                if c.z as f64 > 0.15 && n.z as f64 > 0.45 && oval(c.x as f64, c.y as f64) < 1.0 {
                    o.mesh.faces[f].mat = slot;
                    seat[f] = true;
                }
            }
            // the panel's edge: each vertex it shares with the bag drawn out onto the oval, so the seam runs smooth
            let (mut on_seat, mut on_bag) = (
                vec![false; o.mesh.verts.len()],
                vec![false; o.mesh.verts.len()],
            );
            for (f, &is_seat) in seat.iter().enumerate() {
                for &i in o.mesh.corners(f) {
                    if is_seat {
                        on_seat[i as usize] = true;
                    } else {
                        on_bag[i as usize] = true;
                    }
                }
            }
            for (i, v) in o.mesh.verts.iter_mut().enumerate() {
                if !(on_seat[i] && on_bag[i]) {
                    continue;
                }
                let (x, y) = (v.x as f64, v.y as f64);
                let k = 1.0 / 1e-6_f64.max(sqrt(oval(x, y)));
                (v.x, v.y) = ((x * k) as f32, ((y + 0.07) * k - 0.07) as f32);
            }
        });
        jitter!(bag, 0.012, size = 0.14, seed = 2.0, up = 0.5);
        fsweep!(
            F,
            "strap",
            &[
                (-0.06, 0.48, -0.355),
                (-0.045, 0.5, -0.38),
                (0.0, 0.505, -0.392),
                (0.045, 0.5, -0.38),
                (0.06, 0.48, -0.355)
            ],
            0.0,
            M.bag_seat,
            profile = rounded_rect(0.03, 0.006, 0.003, 1),
            up = (0, 0, 1),
            resolution = 2
        );
    });
    let (lo, hi) = measure(&parts);
    for &o in &parts {
        // its middle where the old one's was (game depth 0.018)
        o.edit(|p| {
            p.loc.x = (p.loc.x as f64 - (lo[0] + hi[0]) / 2.0) as f32;
            p.loc.y = (p.loc.y as f64 + (-0.018 - (lo[1] + hi[1]) / 2.0)) as f32;
        });
    }
    let bag = join(&parts, "bag", Some(F.at(0, 0, 0.02)));
    floor();
    squash(
        bag,
        "squash",
        (1.06, 1.06, 0.85),
        Squash {
            seconds: 0.6,
            ..Squash::default()
        },
    );
    Vec::new()
}
