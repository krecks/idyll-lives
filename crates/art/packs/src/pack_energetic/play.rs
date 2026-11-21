//! The home bouldering wall, the interactive spin bike, the rebound trampoline and the streetball hoop.

use super::common::{HOLDS, M, WALL, lofted};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, cos, cosd, radians, sin, sind};
use artkit::motion::{Rock, Spin, Squash, rock, spin, squash};
use artkit::parts::{hinged, join, join_all, new_parts, numbered, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::py::hypot;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, jitter, kit, revolve, settle, soft, torus};

// --- the home bouldering wall (2x1): an overhanging panel with holds on an A-frame, a crash mat before it ------
const LEAN: f64 = 12.0; // the panel's top leans forward by this much (an overhang)
const FOOT: (f64, f64) = (0.02, -0.38); // the panel's foot (height, depth)
const PANEL_H: f64 = 2.3;

/// A game point `up` metres up the panel's face from its foot and `out` metres out of it.
fn on_panel(x: f64, up: f64, out: f64) -> (f64, f64, f64) {
    let a = radians(LEAN);
    (
        x,
        FOOT.0 + up * cos(a) - out * sin(a),
        FOOT.1 + up * sin(a) + out * cos(a),
    )
}

/// A climbing hold on the panel: a lumpy, flattened lump (a jug, a crimp or a sloper by its shape), its bolt.
fn hold(f: &GameFrame, i: i32, x: f64, up: f64, s: f64, material: Mat) {
    let shape = [
        (1.3, 0.55, 0.8),
        (0.8, 0.5, 1.2),
        (1.0, 0.7, 1.0),
        (1.5, 0.45, 0.6),
    ][(i % 4) as usize];
    let h = fsphere!(
        f,
        numbered("hold"),
        s,
        on_panel(x, up, 0.012),
        material,
        scale = shape,
        rot = (LEAN, 0.0, ((i * 47) % 90 - 45) as f64),
        segments = 8,
        rings = 5
    );
    jitter!(h, s * 0.18, size = s * 1.1, seed = i as f64 * 0.77);
    fcyl!(
        f,
        numbered("bolt"),
        0.008,
        0.004,
        on_panel(x, up, 0.012 + s * shape.1),
        M.bolt,
        rot = (LEAN + 90.0, 0.0, 0.0),
        verts = 6
    );
}

pub fn bouldering_wall() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let rot = (LEAN, 0.0, 0.0);
    fbox!(
        f,
        "panel",
        1.84,
        PANEL_H,
        0.04,
        on_panel(0.0, PANEL_H / 2.0, 0.0),
        M.plywood,
        rot = rot,
        bevel = 0.006,
        bevel_segments = 1
    );
    // the timber frame round its face, one piece each side
    for x in [-0.9, 0.9] {
        fbox!(
            f,
            numbered("edge"),
            0.06,
            PANEL_H,
            0.08,
            on_panel(x, PANEL_H / 2.0, -0.015),
            M.timber,
            rot = rot,
            bevel = 0.01,
            bevel_segments = 1
        );
    }
    fbox!(
        f,
        "top_edge",
        1.86,
        0.08,
        0.08,
        on_panel(0.0, PANEL_H - 0.04, -0.015),
        M.timber,
        rot = rot,
        bevel = 0.01,
        bevel_segments = 1
    );
    fbox!(
        f,
        "foot_edge",
        1.86,
        0.06,
        0.08,
        on_panel(0.0, 0.03, -0.015),
        M.timber,
        rot = rot,
        bevel = 0.01,
        bevel_segments = 1
    );
    fbox!(
        f,
        "seam",
        1.8,
        0.006,
        0.004,
        on_panel(0.0, 1.15, 0.021),
        M.timber,
        rot = rot
    );
    // the T-nut holes in a grid over the face, where holds can be bolted
    for i in 0..10 {
        for j in 0..12 {
            let (x, up) = (
                -0.81 + 0.18 * i as f64 + 0.09 * (j % 2) as f64,
                0.12 + 0.18 * j as f64,
            );
            if x.abs() < 0.84 {
                fbox!(
                    f,
                    numbered("tnut"),
                    0.012,
                    0.012,
                    0.004,
                    on_panel(x, up, 0.0205),
                    M.bolt,
                    rot = rot
                );
            }
        }
    }
    // the steel A-frame behind it, on the wall: posts on base plates, braces bent onto the panel's back
    for x in [-0.84, 0.84] {
        fbox!(
            f,
            numbered("post"),
            0.06,
            2.2,
            0.06,
            (x, 1.1, WALL + 0.03),
            M.climb_frame,
            bevel = 0.008,
            bevel_segments = 1
        );
        for (y, up) in [(2.12, 2.18), (1.1, 1.15)] {
            kit::tube!(
                f,
                numbered("brace"),
                &[(x, y, WALL + 0.03), on_panel(x, up, -0.05)],
                0.022,
                M.climb_frame,
                verts = 8,
                round = 1
            );
            fbox!(
                f,
                numbered("brace_plate"),
                0.08,
                0.1,
                0.012,
                on_panel(x, up, -0.026),
                M.climb_frame,
                rot = rot
            );
        }
        fbox!(
            f,
            numbered("post_foot"),
            0.12,
            0.02,
            0.12,
            (x, 0.01, WALL + 0.06),
            M.climb_frame,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    // the holds, and three volumes (triangular wedges) bolted on the face
    let holds = [
        (-0.62, 0.35),
        (-0.18, 0.42),
        (0.35, 0.3),
        (0.7, 0.55),
        (-0.45, 0.75),
        (0.05, 0.8),
        (0.52, 0.95),
        (-0.75, 1.1),
        (-0.25, 1.2),
        (0.28, 1.3),
        (0.72, 1.42),
        (-0.55, 1.55),
        (0.0, 1.62),
        (0.45, 1.75),
        (-0.3, 1.95),
        (0.2, 2.05),
        (0.66, 2.1),
        (-0.7, 1.9),
        (-0.05, 0.15),
        (0.6, 1.25),
    ];
    for (i, &(x, up)) in holds.iter().enumerate() {
        let i = i as i32;
        hold(
            f,
            i,
            x,
            up,
            0.035 + (i * 37 % 5) as f64 * 0.006,
            HOLDS[i as usize % HOLDS.len()],
        );
    }
    for (x, up, m, a) in [
        (-0.1, 1.0, M.hold_b, 0.0),
        (0.5, 1.95, M.hold_c, 30.0),
        (-0.6, 0.55, M.hold_f, -20.0),
    ] {
        let tri: Vec<(f64, f64)> = (0..3)
            .map(|k| {
                (
                    0.14 * cosd((90 + 120 * k) as f64 + a),
                    0.14 * sind((90 + 120 * k) as f64 + a),
                )
            })
            .collect();
        fextrude!(
            f,
            numbered("volume"),
            &tri,
            0.08,
            on_panel(x, up, 0.06),
            m,
            rot = (LEAN, 0.0, 0.0),
            bevel = 0.012,
            bevel_segments = 1
        );
    }
    // the crash mat folded in two halves, piped in yellow, carry straps on its front; a chalk bag and a towel on it
    for s in [-1.0, 1.0] {
        soft!(
            f,
            numbered("crash_mat"),
            0.935,
            0.22,
            0.811,
            (s * 0.4675, 0.11, 0.0405),
            M.crash_mat,
            round = 0.05,
            piping = 0.008,
            piping_material = M.mat_band,
            flat = 1,
            round_segments = 1
        );
        kit::tube!(
            f,
            numbered("strap"),
            &kit::fillet!(
                &[
                    (s * 0.465 - 0.12, 0.218, 0.36),
                    (s * 0.465 - 0.1, 0.236, 0.36),
                    (s * 0.465 + 0.1, 0.236, 0.36),
                    (s * 0.465 + 0.12, 0.218, 0.36),
                ],
                0.02
            ),
            0.008,
            M.mat_band,
            verts = 5,
            round = 1
        );
    }
    lofted(
        f,
        "chalk_bag",
        0.82,
        0.22,
        0.32,
        &[
            (0.0, 0.05, 0.05),
            (0.01, 0.062, 0.062),
            (0.1, 0.065, 0.065),
            (0.12, 0.058, 0.058),
            (0.14, 0.06, 0.06),
        ],
        M.chalk,
        12,
        2.0,
    );
    torus!(
        "chalk_rim",
        0.06,
        0.008,
        at = f.at(0.82, 0.36, 0.32),
        material = M.mat_band,
        major_segments = 12,
        minor_segments = 3
    );
    fsweep!(
        f,
        "chalk_brush",
        &[(0.84, 0.33, 0.33), (0.9, 0.42, 0.36)],
        0.009,
        M.timber,
        verts = 6
    );
    soft!(
        f,
        "towel",
        0.2,
        0.03,
        0.12,
        (-0.7, 0.235, 0.3),
        M.towel,
        round = 0.012,
        puff = 0.004,
        flat = 1
    );
    join_all("boulderingWall");
    settle!(centre = false)
}

// --- the interactive spin bike (1x2): a recumbent trainer, seat at 0.45, cranks in the flywheel's hub --------------
const CRANK: (f64, f64) = (0.36, 0.66); // the crank axle (height, depth)

pub fn spin_bike() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (cy, cz) = CRANK;
    let still = new_parts(|| {
        soft!(
            f,
            "seat",
            0.44,
            0.08,
            0.42,
            (0, 0.4, 0.0),
            M.bike_pad,
            round = 0.03,
            puff = 0.01,
            piping = 0.005,
            flat = 2
        );
        soft!(
            f,
            "backrest",
            0.42,
            0.52,
            0.08,
            (0, 0.72, -0.27),
            M.bike_pad,
            rot = (-18, 0, 0),
            round = 0.035,
            puff = 0.015,
            flat = 2
        );
        fextrude!(
            f,
            "seat_pan",
            &rounded_rect(0.46, 0.3, 0.06, 2),
            0.03,
            (0, 0.345, -0.02),
            M.bike_frame,
            plane = Facing::Top,
            bevel = 0.01,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "seat_post",
            &[(0, 0.1, -0.02), (0, 0.34, -0.02)],
            0,
            M.bike_frame,
            profile = rounded_rect(0.1, 0.12, 0.035, 2),
            up = (1, 0, 0)
        );
        fextrude!(
            f,
            "back_shell",
            &rounded_rect(0.4, 0.5, 0.06, 2),
            0.02,
            (0, 0.71, -0.31),
            M.bike_frame,
            rot = (-18, 0, 0),
            bevel = 0.006,
            bevel_segments = 1
        );
        kit::tube!(
            f,
            "back_strut",
            &kit::fillet!(
                &[(0.0, 0.34, -0.14), (0.0, 0.4, -0.3), (0.0, 0.6, -0.36)],
                0.06
            ),
            0.022,
            M.bike_frame,
            verts = 8,
            round = 1
        );
        // the main beam, curving up from the rear stabiliser to the flywheel's foot
        fsweep!(
            f,
            "beam",
            &[
                (0, 0.05, -0.66),
                (0, 0.08, -0.2),
                (0, 0.13, 0.3),
                (0, 0.18, 0.62)
            ],
            0,
            M.bike_frame,
            profile = rounded_rect(0.1, 0.11, 0.04, 2),
            up = (1, 0, 0),
            resolution = 2
        );
        for (z, w) in [(-0.66, 0.6), (0.86, 0.56)] {
            fsweep!(
                f,
                numbered("stab"),
                &[(-w / 2.0 + 0.02, 0.03, z), (w / 2.0 - 0.02, 0.03, z)],
                0,
                M.bike_grey,
                profile = rounded_rect(0.05, 0.08, 0.02, 2),
                up = (0, 0, 1)
            );
            for x in [-w / 2.0 + 0.0065, w / 2.0 - 0.0065] {
                fcyl!(
                    f,
                    numbered("stab_cap"),
                    0.032,
                    0.03,
                    (x, 0.03, z),
                    M.dark,
                    rot = (0, 90, 0),
                    verts = 10
                );
            }
        }
        fsweep!(
            f,
            "front_leg",
            &[(0, 0.04, 0.86), (0, 0.12, 0.76), (0, 0.18, 0.66)],
            0,
            M.bike_frame,
            profile = rounded_rect(0.1, 0.1, 0.035, 2),
            up = (1, 0, 0),
            resolution = 2
        );
        // the flywheel's rounded housing, the orange wheel showing in each side with its spokes
        revolve!(
            "housing",
            &[
                (0.0, -0.08),
                (0.2, -0.08),
                (0.228, -0.074),
                (0.24, -0.055),
                (0.24, 0.055),
                (0.228, 0.074),
                (0.2, 0.08),
                (0.0, 0.08)
            ],
            at = f.at(0, cy, cz),
            material = M.bike_frame,
            segments = 28,
            rot = (0, 90, 0)
        );
        for s in [-1.0, 1.0] {
            fcyl!(
                f,
                numbered("flywheel"),
                0.19,
                0.006,
                (s * 0.081, cy, cz),
                M.flywheel,
                rot = (0, 90, 0),
                verts = 24
            );
            for k in 0..5 {
                let a = (72 * k + 18) as f64;
                fbox!(
                    f,
                    numbered("spoke"),
                    0.004,
                    0.022,
                    0.15,
                    (s * 0.085, cy + 0.1 * sind(a), cz + 0.1 * cosd(a)),
                    M.steel,
                    rot = (-a, 0.0, 0.0)
                );
            }
        }
        fsweep!(
            f,
            "housing_foot",
            &[(0, 0.04, cz), (0, cy - 0.12, cz)],
            0,
            M.bike_frame,
            profile = rounded_rect(0.2, 0.22, 0.05, 2),
            up = (1, 0, 0)
        );
        for s in [-1.0, 1.0] {
            // the side grips by the seat: bent tube, foam grips
            kit::tube!(
                f,
                numbered("grip_post"),
                &kit::fillet!(
                    &[
                        (s * 0.2, 0.34, 0.1),
                        (s * 0.24, 0.36, 0.12),
                        (s * 0.24, 0.5, 0.12),
                        (s * 0.24, 0.5, -0.12),
                    ],
                    0.03
                ),
                0.012,
                M.steel,
                verts = 8,
                round = 1
            );
            fsweep!(
                f,
                numbered("grip"),
                &[(s * 0.24, 0.5, 0.06), (s * 0.24, 0.5, -0.12)],
                0.017,
                M.dark,
                verts = 8
            );
        }
        // the console on its stalk, its route screen facing the rider, the handlebar and a bottle in its cage
        kit::tube!(
            f,
            "stalk",
            &[(0.0, 0.5, 0.74), (0.0, 0.75, 0.69), (0.0, 1.0, 0.63)],
            0.025,
            M.bike_frame,
            verts = 10,
            round = 2
        );
        let (cc, a) = ((0, 1.08, 0.6), radians(35.0));
        fextrude!(
            f,
            "console",
            &rounded_rect(0.42, 0.28, 0.04, 2),
            0.04,
            cc,
            M.bike_frame,
            rot = (35, 0, 0),
            bevel = 0.01,
            bevel_segments = 1
        );
        let n = (0.0, sin(a), -cos(a)); // the screen's normal, towards the rider
        fbox!(
            f,
            "route",
            0.38,
            0.23,
            0.004,
            (0.0, cc.1 + n.1 * 0.021, cc.2 + n.2 * 0.021),
            M.route,
            rot = (35, 0, 0)
        );
        fbox!(
            f,
            "route_line",
            0.2,
            0.05,
            0.003,
            (
                -0.05,
                cc.1 + n.1 * 0.024 + 0.03 * cos(a),
                cc.2 + n.2 * 0.024 + 0.03 * sin(a)
            ),
            M.route_line,
            rot = (35, 0, 0)
        );
        kit::tube!(
            f,
            "handlebar",
            &kit::fillet!(
                &[
                    (-0.24, 1.02, 0.5),
                    (-0.22, 0.98, 0.56),
                    (0.22, 0.98, 0.56),
                    (0.24, 1.02, 0.5),
                ],
                0.04
            ),
            0.014,
            M.dark,
            verts = 8,
            round = 1
        );
        lofted(
            f,
            "bottle",
            0.2,
            0.33,
            0.8,
            &[
                (0.0, 0.03, 0.03),
                (0.005, 0.035, 0.035),
                (0.14, 0.035, 0.035),
                (0.17, 0.022, 0.022),
                (0.2, 0.02, 0.02),
            ],
            M.bottle,
            10,
            2.0,
        );
        fcyl!(
            f,
            "bottle_cap",
            0.022,
            0.03,
            (0.2, 0.545, 0.8),
            M.dark,
            verts = 10
        );
        kit::tube!(
            f,
            "cage",
            &kit::fillet!(
                &[
                    (0.16, 0.36, 0.8),
                    (0.165, 0.36, 0.84),
                    (0.235, 0.36, 0.84),
                    (0.24, 0.36, 0.8),
                ],
                0.02
            ),
            0.004,
            M.steel,
            verts = 4,
            round = 1
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The cranks and pedals on the flywheel's axle: `pedals` turns them while a resident rides.
    let cranks = new_parts(|| {
        fcyl!(
            f,
            "axle",
            0.025,
            0.24,
            (0, cy, cz),
            M.steel,
            rot = (0, 90, 0),
            verts = 12
        );
        for (s, ang) in [(-1.0, 60.0), (1.0, 240.0)] {
            let (ey, ez) = (cy + 0.13 * sind(ang), cz + 0.13 * cosd(ang));
            fsweep!(
                f,
                numbered("crank"),
                &[(s * 0.12, cy, cz), (s * 0.122, ey, ez)],
                0,
                M.steel,
                profile = rounded_rect(0.014, 0.034, 0.007, 1),
                up = (1, 0, 0),
                scale = [1.0, 0.75]
            );
            fcyl!(
                f,
                numbered("spindle"),
                0.007,
                0.05,
                (s * 0.145, ey, ez),
                M.steel,
                rot = (0, 90, 0),
                verts = 6
            );
            fextrude!(
                f,
                numbered("pedal"),
                &rounded_rect(0.09, 0.06, 0.012, 2),
                0.022,
                (s * 0.2, ey, ez),
                M.dark,
                plane = Facing::Top,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("strap"),
                0.07,
                0.022,
                0.014,
                (s * 0.2, ey + 0.02, ez + 0.01),
                M.bike_grey,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
    });
    let cranks = hinged(join(&cranks, "cranks", None), f.at(0, cy, cz));
    let objs = settle!(centre = false);
    spin(
        cranks,
        "pedals",
        Spin {
            turns: 1.0,
            seconds: 1.0,
            axis: Axis::X,
        },
    );
    objs
}

// --- the rebound trampoline (2x2): a round frame on six legs, a padded rim and the mat, a ladder at the front ---
const MAT_RIM: f64 = 0.6; // the mat's edge, on the springs under the pad
const MAT_SAG: f64 = 0.02; // its middle sits this much lower at rest; `bounce` stretches the sag to 0.1 (the feet at 0.5)

pub fn trampoline() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        torus!(
            "frame",
            0.92,
            0.022,
            at = f.at(0, 0.56, 0),
            material = M.tramp_frame,
            major_segments = 40,
            minor_segments = 8
        );
        // three W legs: each down from the frame, along the grass and up again, in one bent tube
        for k in 0..3 {
            let a0 = (k * 120 + 30) as f64;
            let mut pts = Vec::new();
            for (a, top) in [(a0, true), (a0 + 60.0, false)] {
                let (c, s) = (cosd(a), sind(a));
                let rim = (0.92 * c, 0.555, 0.92 * s);
                let foot = (0.92 * c, 0.02, 0.92 * s);
                if top {
                    pts.extend([rim, foot]);
                } else {
                    pts.extend([foot, rim]);
                }
            }
            kit::tube!(
                f,
                numbered("leg"),
                &kit::fillet!(&pts, 0.06),
                0.02,
                M.tramp_frame,
                verts = 8,
                round = 1
            );
            for a in [a0, a0 + 60.0] {
                fcyl!(
                    f,
                    numbered("leg_foot"),
                    0.03,
                    0.012,
                    (0.92 * cosd(a), 0.006, 0.92 * sind(a)),
                    M.dark,
                    verts = 8
                );
            }
        }
        for i in 0..36 {
            // the springs, under the pad's edge
            let a = (i * 10) as f64;
            let (c, s) = (cosd(a), sind(a));
            kit::tube!(
                f,
                numbered("spring"),
                &[
                    (0.8 * c, MAT_RIM - 0.004, 0.8 * s),
                    (0.915 * c, 0.565, 0.915 * s),
                ],
                0.007,
                M.steel,
                verts = 4,
                round = 1
            );
        }
        let prof = [
            (0.78, 0.6),
            (0.93, 0.6),
            (0.948, 0.607),
            (0.952, 0.617),
            (0.944, 0.628),
            (0.92, 0.634),
            (0.79, 0.634),
            (0.776, 0.626),
            (0.774, 0.608),
            (0.78, 0.6),
        ];
        revolve!(
            "pad",
            &prof,
            at = f.at(0, 0, 0),
            material = M.tramp_pad,
            segments = 40
        );
        // the ladder at the front: two bent stiles hooked over the frame, two treads
        for x in [-0.13, 0.13] {
            kit::tube!(
                f,
                numbered("ladder"),
                &kit::fillet!(
                    &[
                        (x, 0.014, 0.94),
                        (x, 0.5, 0.875),
                        (x, 0.59, 0.895),
                        (x, 0.585, 0.93),
                    ],
                    0.025
                ),
                0.014,
                M.tramp_frame,
                verts = 8,
                round = 1
            );
        }
        for y in [0.18, 0.36] {
            fextrude!(
                f,
                numbered("step"),
                &rounded_rect(0.3, 0.07, 0.02, 2),
                0.025,
                (0.0, y, 0.94 - y / 0.5 * 0.065),
                M.dark,
                plane = Facing::Top,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        fsphere!(
            f,
            "ball",
            0.11,
            (0.8, 0.11, 0.76),
            M.ball,
            segments = 16,
            rings = 10
        );
        for r in [(90, 0, 30), (0, 90, 30)] {
            torus!(
                numbered("ball_seam"),
                0.11,
                0.003,
                at = f.at(0.8, 0.11, 0.76),
                material = M.ball_seam,
                rot = r,
                major_segments = 20,
                minor_segments = 3
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The mat, a shallow dish from the springs in, its origin at the rim, a stitched band round it and a target in
    // its middle: `bounce` deepens it and lets it go.
    let mat = new_parts(|| {
        let y0 = MAT_RIM;
        let mut prof = vec![(0.0, y0 - MAT_SAG - 0.008)];
        prof.extend(
            [0.08, 0.1, 0.7, 0.73, 0.8]
                .iter()
                .map(|&r| (r, y0 - 0.008 - MAT_SAG * (1.0 - r / 0.8))),
        );
        prof.push((0.8, y0));
        prof.extend(
            [0.73, 0.7, 0.1, 0.08]
                .iter()
                .map(|&r| (r, y0 - MAT_SAG * (1.0 - r / 0.8))),
        );
        prof.push((0.0, y0 - MAT_SAG));
        let dish = revolve!(
            "mat",
            &prof,
            at = f.at(0, 0, 0),
            material = M.tramp_mat,
            segments = 40
        );
        kit::paint(dish, M.dark, |c| {
            let r = hypot(c.x as f64, c.y as f64);
            c.z as f64 > y0 - MAT_SAG - 0.004
                && ((0.705 < r && r < 0.725) || (0.082 < r && r < 0.098))
        });
    });
    let mat = hinged(join(&mat, "mat", None), f.at(0, MAT_RIM, 0));
    let objs = settle_all(); // on its Model centre: the back on the wall face
    squash(
        mat,
        "bounce",
        (1, 1, 5),
        Squash {
            seconds: 0.625,
            back: true,
            ..Squash::default()
        },
    ); // one bounce (the residents bounce 1.6 a second)
    objs
}

// --- the streetball hoop (1x1): a weighted base at the back, the pole, the board and the rim over the front -------
const RIM: (f64, f64, f64) = (2.4, 0.22, 0.225); // the rim's height, depth and radius

pub fn streetball_hoop() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (ry, rz, rr) = RIM;
    let still = new_parts(|| {
        // the blow-moulded base: rounded, tapering up, its fill cap, two wheels at its back
        lofted(
            f,
            "base",
            0.0,
            0.0,
            WALL + 0.25,
            &[
                (0.0, 0.29, 0.24),
                (0.02, 0.3, 0.25),
                (0.16, 0.28, 0.23),
                (0.21, 0.25, 0.2),
                (0.23, 0.22, 0.17),
            ],
            M.hoop_base,
            24,
            4.0,
        );
        revolve!(
            "fill_cap",
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.06, 0.03),
                (0.05, 0.04),
                (0.0, 0.04)
            ],
            at = f.at(0.18, 0.21, -0.3),
            material = M.cap,
            segments = 12
        );
        for s in [-1.0, 1.0] {
            fcyl!(
                f,
                numbered("wheel"),
                0.05,
                0.04,
                (s * 0.24, 0.05, WALL + 0.05),
                M.dark,
                rot = (0, 90, 0),
                verts = 14
            );
            fcyl!(
                f,
                numbered("hub"),
                0.022,
                0.044,
                (s * 0.24, 0.05, WALL + 0.05),
                M.cap,
                rot = (0, 90, 0),
                verts = 8
            );
        }
        // the pole in two lengths, a collar where they meet, its pad, the arm braces up to the board
        kit::tube!(
            f,
            "pole",
            &[(0.0, 0.2, -0.28), (0.0, 1.3, -0.215)],
            0.048,
            M.hoop_pole,
            verts = 12,
            round = 1
        );
        kit::tube!(
            f,
            "pole_upper",
            &[(0.0, 1.26, -0.217), (0.0, 2.32, -0.16)],
            0.04,
            M.hoop_pole,
            verts = 12,
            round = 1
        );
        revolve!(
            "collar",
            &[
                (0.0, 0.0),
                (0.058, 0.0),
                (0.06, 0.01),
                (0.06, 0.05),
                (0.055, 0.06),
                (0.0, 0.06)
            ],
            at = f.at(0, 1.27, -0.216),
            material = M.dark,
            segments = 12
        );
        kit::tube!(
            f,
            "collar_knob",
            &[(0.0, 1.3, -0.16), (0.0, 1.3, -0.12)],
            0.012,
            M.dark,
            verts = 6,
            round = 1
        );
        soft!(
            f,
            "pole_pad",
            0.12,
            0.9,
            0.12,
            (0, 0.7, -0.252),
            M.pad_red,
            rot = (3.24, 0.0, 0.0),
            round = 0.03,
            flat = 1
        );
        for x in [-0.1, 0.1] {
            kit::tube!(
                f,
                numbered("arm"),
                &[(0.0, 2.1, -0.18), (x, 2.3, -0.06), (x, 2.42, -0.045)],
                0.016,
                M.hoop_pole,
                verts = 8,
                round = 1
            );
        }
        kit::tube!(
            f,
            "arm_top",
            &[(0.0, 2.25, -0.2), (0.0, 2.42, -0.05)],
            0.028,
            M.hoop_pole,
            verts = 10,
            round = 1
        );
        // the board: white, a red border round it, the target square over the rim
        fextrude!(
            f,
            "board",
            &rounded_rect(0.84, 0.54, 0.03, 2),
            0.035,
            (0, 2.555, -0.02),
            M.board,
            bevel = 0.006,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "border",
            &rounded_rect(0.86, 0.56, 0.035, 2),
            0.045,
            (0, 2.555, -0.02),
            M.pad_red,
            holes = vec![rounded_rect(0.79, 0.49, 0.01, 1)],
            bevel = 0.008,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "target",
            &rounded_rect(0.3, 0.2, 0.004, 1),
            0.044,
            (0, 2.51, -0.016),
            M.pad_red,
            holes = vec![rounded_rect(0.25, 0.155, 0.002, 1)]
        );
        // the rim on its bracket, little hooks round it for the net
        torus!(
            "rim",
            rr,
            0.011,
            at = f.at(0, ry, rz),
            material = M.rim,
            major_segments = 28,
            minor_segments = 6
        );
        fextrude!(
            f,
            "rim_bracket",
            &[(-0.06, 0.0), (0.06, 0.0), (0.04, 0.06), (-0.04, 0.06)],
            0.022,
            (0.0, ry - 0.011, 0.0),
            M.rim,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "rim_plate",
            0.14,
            0.12,
            0.012,
            (0.0, ry - 0.04, 0.0),
            M.rim,
            bevel = 0.004,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The ball at rest by the stand: a part of its own, gone while someone plays (in their hands: actions.ts
    // `PROPS.basketball`).
    let ball_parts = new_parts(|| {
        fsphere!(
            f,
            "ball",
            0.12,
            (0.28, 0.12, 0.3),
            M.ball,
            segments = 16,
            rings = 10
        );
        for r in [(90, 0, 0), (0, 90, 0), (0, 0, 0)] {
            torus!(
                numbered("ball_seam"),
                0.12,
                0.003,
                at = f.at(0.28, 0.12, 0.3),
                material = M.ball_seam,
                rot = r,
                major_segments = 20,
                minor_segments = 3
            );
        }
    });
    let ball = hinged(join(&ball_parts, "ball", None), f.at(0.28, 0.12, 0.3));
    // The net: strands spiralling down from the rim each way, crossing into diamonds, drawn in at its foot by two
    // rings; `swish` sways it under the rim.
    let net = new_parts(|| {
        let levels = [
            (ry - 0.01, rr - 0.012),
            (ry - 0.12, rr * 0.88),
            (ry - 0.23, rr * 0.74),
            (ry - 0.34, rr * 0.64),
        ];
        for k in 0..12 {
            for t in [-1, 1] {
                let pts: Vec<(f64, f64, f64)> = levels
                    .iter()
                    .enumerate()
                    .map(|(j, &(y, r))| {
                        let a = (k * 30 + t * 15 * j as i32) as f64;
                        (r * cosd(a), y, rz + r * sind(a))
                    })
                    .collect();
                fsweep!(f, numbered("strand"), &pts, 0.0035, M.net, verts = 4);
            }
        }
        for &(y, r) in &levels[1..] {
            torus!(
                numbered("net_ring"),
                r,
                0.0035,
                at = f.at(0, y, rz),
                material = M.net,
                major_segments = 24,
                minor_segments = 3
            );
        }
    });
    let net = hinged(join(&net, "net", None), f.at(0, ry, rz));
    let objs = settle!(centre = false);
    rock(
        net,
        "swish",
        9.0,
        Rock {
            seconds: 1.0,
            axis: Axis::X,
        },
    );
    squash(
        ball,
        "ball_taken",
        0.001,
        Squash {
            seconds: 0.2,
            ..Squash::default()
        },
    );
    objs
}
