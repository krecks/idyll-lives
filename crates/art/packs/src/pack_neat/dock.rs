//! The steam mop dock and the robot vacuum with its dock.

use super::common::{_bottle, M, WALL};
use crate::common::_box;
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{P3, PI, TAU, cos, cosd, degrees, pow, radians, sin, sind};
use artkit::motion::{PathKey, path};
use artkit::parts::{aside, hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsphere, fsweep, leaf, loft, revolve, ring, settle, soft, squash};

// --- the steam mop dock (1x1): a slim white tower against the wall, the mop docked before it ----------------
const MOP: (f64, f64, f64) = (0.0, 0.04, -0.07); // the mop head's foot (x, height, depth): it pivots there

pub fn steam_mop_dock() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _box(
            f,
            "tower",
            0.42,
            1.25,
            0.2,
            (0, 0.625, WALL + 0.1),
            M.body,
            5.0,
            24,
            0.006,
        );
        soft!(
            f,
            "panel",
            0.34,
            0.26,
            0.012,
            (0, 1.0, WALL + 0.204),
            M.porthole,
            round = 0.006,
            round_segments = 1
        );
        for (x, w, y) in [(-0.08, 0.1, 1.06), (0.04, 0.16, 1.02), (-0.02, 0.2, 0.98)] {
            fbox!(
                f,
                format!("panel_line{y}"),
                w,
                0.012,
                0.004,
                (x, y, WALL + 0.212),
                M.teal,
                bevel = 0.002,
                bevel_segments = 1
            );
        }
        fsphere!(
            f,
            "led",
            0.012,
            (0.12, 0.92, WALL + 0.212),
            M.led,
            segments = 8,
            rings = 4
        );
        _box(
            f,
            "base_plate",
            0.44,
            0.04,
            0.36,
            (0, 0.02, WALL + 0.18),
            M.trim,
            4.0,
            20,
            0.006,
        );
        // a C-clip holding the pole, on a stub from the tower
        fbox!(
            f,
            "clip_stub",
            0.04,
            0.03,
            0.03,
            (MOP.0, 1.1, WALL + 0.215),
            M.trim,
            bevel = 0.006,
            bevel_segments = 1
        );
        let clip: Vec<(f64, f64, f64)> = (-230..51)
            .step_by(35)
            .map(|a| {
                (
                    MOP.0 + 0.022 * cos(radians(a as f64)),
                    1.1,
                    WALL + 0.26 + 0.022 * sin(radians(a as f64)),
                )
            })
            .collect();
        fsweep!(
            f,
            "clip",
            &clip,
            0.0,
            M.trim,
            profile = rounded_rect(0.006, 0.03, 0.002, 1),
            up = (0, 0, 1)
        );
        // the refill shelf on two brackets, three bottles on it
        fbox!(
            f,
            "shelf",
            0.38,
            0.02,
            0.08,
            (0, 0.7, WALL + 0.24),
            M.birch_plain,
            bevel = 0.005,
            bevel_segments = 1
        );
        for x in [-0.16, 0.16] {
            fextrude!(
                f,
                format!("shelf_bracket{x}"),
                &[(WALL + 0.2, 0.69), (WALL + 0.27, 0.69), (WALL + 0.2, 0.62)],
                0.012,
                (x, 0, 0),
                M.birch_plain,
                plane = Facing::Side
            );
        }
        for (x, m) in [(-0.13, M.bottle_a), (-0.07, M.bottle_b), (0.13, M.bottle_a)] {
            _bottle(
                f,
                &format!("refill{x}"),
                x,
                WALL + 0.24,
                0.71,
                m,
                0.14,
                0.05,
                None,
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // a bucket with its bail handle, spare pads stacked on the floor: drawn, not the dock's size
    let floor = new_parts(|| {
        revolve!(
            "bucket",
            &[
                (0.0, 0.0),
                (0.105, 0.0),
                (0.134, 0.25),
                (0.14, 0.255),
                (0.14, 0.262),
                (0.128, 0.262),
                (0.122, 0.24),
                (0.095, 0.015),
                (0.0, 0.015)
            ],
            at = f.at(-0.26, 0, 0.14),
            material = M.pastel_a,
            segments = 20
        );
        let handle: Vec<(f64, f64, f64)> = (0..181)
            .step_by(20)
            .map(|a| {
                let r = radians(a as f64);
                (
                    -0.26 + 0.134 * cos(r),
                    0.25 + 0.06 * sin(r) + 0.02 * pow(sin(r), 8.0),
                    0.14 + 0.03 * sin(r),
                )
            })
            .collect();
        fsweep!(f, "bucket_handle", &handle, 0.004, M.trim, verts = 5);
        soft!(
            f,
            "spare_pad",
            0.22,
            0.02,
            0.14,
            (0.27, 0.014, 0.16),
            M.towel_w,
            round = 0.008,
            puff = 0.004,
            flat = 2,
            round_segments = 1
        );
        soft!(
            f,
            "spare_pad_b",
            0.22,
            0.02,
            0.14,
            (0.27, 0.036, 0.16),
            M.towel_c,
            rot = (0, 0, 8),
            round = 0.008,
            puff = 0.004,
            flat = 2,
            round_segments = 1,
            pinch = 0.0
        ); // (askew: a corner is the model's outermost point)
    });
    aside(&floor, "floor");
    // The mop: pole, grip, water tank, head and pad, its origin at its head's foot: hidden while it's used (`mop_off`),
    // the game drawing it in the resident's hands (content `held`), pushed over the floor.
    let (x, y, z) = MOP;
    let top = (x, 1.12, WALL + 0.26);
    let mop = new_parts(|| {
        soft!(
            f,
            "mop_head",
            0.34,
            0.04,
            0.13,
            (x, y, z),
            M.trim,
            round = 0.018,
            round_segments = 1
        );
        soft!(
            f,
            "mop_pad",
            0.33,
            0.022,
            0.125,
            (x, 0.016, z),
            M.pad,
            round = 0.01,
            puff = 0.004,
            flat = 2,
            round_segments = 1
        );
        revolve!(
            "mop_swivel",
            &[
                (0.0, 0.0),
                (0.026, 0.0),
                (0.024, 0.02),
                (0.016, 0.03),
                (0.016, 0.06),
                (0.0, 0.06)
            ],
            at = f.at(x, y + 0.02, z),
            material = M.dark,
            segments = 10
        );
        fsweep!(
            f,
            "mop_pole",
            &[(x, y + 0.07, z), top],
            0.013,
            M.trim,
            verts = 10
        );
        fsweep!(
            f,
            "mop_grip",
            &kit::fillet!(
                &[
                    (x, 1.1, WALL + 0.262),
                    (x, 1.17, WALL + 0.255),
                    (x - 0.05, 1.2, WALL + 0.25),
                    (x, 1.23, WALL + 0.245),
                    (x + 0.05, 1.2, WALL + 0.25),
                    (x, 1.17, WALL + 0.255)
                ],
                0.02
            ),
            0.0,
            M.teal,
            profile = rounded_rect(0.024, 0.03, 0.01, 1),
            up = (0, 0, 1)
        );
        let b = f.at(x, 0.2, z - 0.032);
        let sections: Vec<Vec<P3>> = [
            (0.0, 0.0),
            (0.012, 0.8),
            (0.04, 1.0),
            (0.24, 1.0),
            (0.27, 0.8),
            (0.28, 0.0),
        ]
        .iter()
        .map(|&(h, k)| {
            ring!(
                0.05 * k,
                0.04 * k,
                z = b.z + h,
                verts = 14,
                square = 3,
                at = (b.x, b.y)
            )
        })
        .collect();
        loft!("mop_tank", &sections, material = M.teal, rot = (0, 0, 0));
    });
    let mop = hinged(join(&mop, "mop", None), f.at(x, 0.0, z));
    let objs = settle!(centre = false);
    squash!(mop, "mop_off", 0.001, seconds = 0.2);
    objs
}

// --- the robot vacuum and dock (1x1): a self-emptying dock against the wall, the robot docked on a round rug ---
const ROBOT: (f64, f64) = (0.0, -0.07); // the docked robot's middle (x, depth), against the dock

pub fn robot_vacuum_dock() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    // the rug the robot docks on and a plant beside the dock: drawn, not the dock's size
    let floor = new_parts(|| {
        revolve!(
            "rug",
            &[
                (0.0, 0.0),
                (0.43, 0.0),
                (0.432, 0.006),
                (0.425, 0.012),
                (0.36, 0.012),
                (0.352, 0.008),
                (0.344, 0.014),
                (0.0, 0.014)
            ],
            at = f.at(0, 0, 0),
            material = M.rug,
            segments = 36
        );
        revolve!(
            "rug_inner",
            &[
                (0.0, 0.008),
                (0.34, 0.008),
                (0.34, 0.014),
                (0.33, 0.016),
                (0.0, 0.016)
            ],
            at = f.at(0, 0, 0),
            material = M.rug_inner,
            segments = 32
        );
    });
    let still = new_parts(|| {
        // the dock: a rounded tower, its bin window and the bin behind it, a ramp out over the rug
        let b = f.at(0, 0, WALL + 0.095);
        let sections: Vec<Vec<P3>> = [
            (0.0, 0.98),
            (0.01, 1.0),
            (0.4, 1.0),
            (0.44, 0.9),
            (0.46, 0.7),
            (0.462, 0.0),
        ]
        .iter()
        .map(|&(h, k)| {
            ring!(
                0.19 * k,
                0.095 * k,
                z = b.z + h,
                verts = 24,
                square = 4,
                at = (b.x, b.y)
            )
        })
        .collect();
        loft!("dock", &sections, material = M.appliance);
        soft!(
            f,
            "bin_window",
            0.28,
            0.2,
            0.014,
            (0, 0.3, WALL + 0.188),
            M.porthole,
            round = 0.02,
            round_segments = 2
        );
        soft!(
            f,
            "bin",
            0.2,
            0.12,
            0.006,
            (0, 0.3, WALL + 0.194),
            M.steel,
            round = 0.012,
            round_segments = 1
        );
        fbox!(
            f,
            "bin_latch",
            0.06,
            0.015,
            0.008,
            (0, 0.415, WALL + 0.19),
            M.trim,
            bevel = 0.003,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "ramp",
            &[
                (-0.17, WALL + 0.19),
                (0.17, WALL + 0.19),
                (0.17, WALL + 0.29),
                (-0.17, WALL + 0.29)
            ],
            0.02,
            (0, 0.024, 0),
            M.trim,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 2
        );
        fsphere!(
            f,
            "led",
            0.01,
            (0.13, 0.42, WALL + 0.19),
            M.led,
            segments = 8,
            rings = 4
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    let plant = new_parts(|| {
        // a little plant beside the dock
        revolve!(
            "plant_pot",
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.09, 0.19),
                (0.0935, 0.1955), // (its lip rounded)
                (0.0935, 0.1985),
                (0.091, 0.2005),
                (0.08, 0.2),
                (0.07, 0.18),
                (0.0, 0.18)
            ],
            at = f.at(0.31, 0, WALL + 0.12),
            material = M.body,
            segments = 16
        );
        for k in 0..9 {
            // a snake plant: upright, banded strap leaves
            let a = (k * 40) as f64;
            leaf!(
                format!("plant_leaf{k}"),
                f.at(0.31 + 0.02 * cosd(a), 0.17, WALL + 0.12 + 0.02 * sind(a)),
                (k * 40 + 15) as f64,
                (82 + 2 * (k % 3)) as f64,
                0.36 - 0.04 * (k % 3) as f64,
                0.05,
                [M.green, M.stem][k % 2],
                shape = "lance",
                droop = 0.04,
                fold = 0.35,
                rows = 4,
                ring = 3,
                twist = 40.0
            );
        }
    });
    aside(&[floor, plant].concat(), "floor");
    // The robot: a disc with a teal bumper, its lid's seam, the lidar turret, a front sensor and a side brush:
    // `vacuum_round` (a loop) drives it out of its dock, round a circle in front of it, turning as it goes, and back in.
    let (x, z) = ROBOT;
    let y = 0.014;
    let robot = new_parts(|| {
        revolve!(
            "robot_body",
            &[
                (0.0, 0.0),
                (0.155, 0.0),
                (0.168, 0.012),
                (0.17, 0.07),
                (0.162, 0.082),
                (0.12, 0.085),
                (0.118, 0.083),
                (0.11, 0.085),
                (0.0, 0.085)
            ],
            at = f.at(x, y, z),
            material = M.appliance,
            segments = 28
        );
        revolve!(
            "robot_bumper",
            &[
                (0.168, 0.018),
                (0.176, 0.024),
                (0.178, 0.05),
                (0.172, 0.058),
                (0.168, 0.056)
            ],
            at = f.at(x, y, z),
            material = M.teal,
            segments = 28
        );
        revolve!(
            "lidar",
            &[
                (0.0, 0.0),
                (0.046, 0.0),
                (0.046, 0.03),
                (0.04, 0.036),
                (0.0, 0.036)
            ],
            at = f.at(x, y + 0.083, z - 0.06),
            material = M.dark,
            segments = 16
        );
        soft!(
            f,
            "robot_front",
            0.22,
            0.03,
            0.02,
            (x, y + 0.04, z + 0.165),
            M.dark,
            round = 0.01,
            round_segments = 1
        );
        for k in 0..6 {
            let (u, v) = (
                0.045 * cos(TAU * k as f64 / 6.0),
                0.045 * sin(TAU * k as f64 / 6.0),
            );
            fsweep!(
                f,
                format!("brush{k}"),
                &[
                    (x + 0.12, y + 0.004, z + 0.12),
                    (x + 0.12 + u, y + 0.002, z + 0.12 + v)
                ],
                0.002,
                M.steel,
                verts = 3
            );
        }
    });
    let robot = hinged(join(&robot, "robot", None), f.at(x, y, z));
    let objs = settle!(centre = false);
    path(robot, "vacuum_round", &_round(), true);
    objs
}

// The robot's round (game metres from where it docks): OUT forward, a circle of RADIUS to its left (seen from the
// front, game -x: clear of the plant), its middle that far ahead, and back in, at SPEED m/s; it stays a hair inside
// its tile and off the dock and the plant.
const OUT: f64 = 0.18;
const RADIUS: f64 = 0.14;
const SPEED: f64 = 0.25;

/// [(seconds, Blender offset, Blender degrees)] for `path`: the robot faces the way it goes (its front, game +z,
/// turned about Blender Z by the angle round the circle).
fn _round() -> Vec<PathKey> {
    let mut keys: Vec<PathKey> = vec![(0.0, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0))];
    let t = OUT / SPEED;
    keys.push((t, (0.0, -OUT, 0.0), (0.0, 0.0, 0.0)));
    let steps = 24;
    for k in 1..=steps {
        let a = 2.0 * PI * k as f64 / steps as f64;
        let (x, z) = (-RADIUS + RADIUS * cos(a), OUT + RADIUS * sin(a)); // game (x, depth)
        keys.push((
            t + 2.0 * PI * RADIUS * k as f64 / steps as f64 / SPEED,
            (-x, -z, 0.0),
            (0.0, 0.0, degrees(a)),
        ));
    }
    let t = keys[keys.len() - 1].0;
    keys.push((t + OUT / SPEED, (0.0, 0.0, 0.0), (0.0, 0.0, 360.0)));
    keys
}
