//! The washer and the dryer: a still cabinet, the round door hinged on its left and the drum's face behind the
//! glass, each its own node with its clip.

use super::M;
use super::common::{DOOR_R, DOOR_Y, DOOR_Z, folded, front_disc, lump, rbox};
use artkit::forms::eased;
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit::fillet;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, TAU, cos, cosd, degrees, radians, sin, sind};
use artkit::motion::{Ease, Spin, spin};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsweep, revolve, settle, sphere, swing};

const ZERO: P3 = P3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

/// The still cabinet: the body under a top panel, a recessed plinth, the control strip with a soap drawer or a
/// filter, a program knob in its ring of settings, a display and buttons, the rubber bellows round the porthole.
fn cabinet(f: &GameFrame, led: Mat, dryer: bool, _ring: Mat) {
    artkit::soft!(
        f,
        "cabinet",
        0.6,
        0.84,
        0.6,
        (0, 0.44, -0.16),
        M.body,
        round = 0.03,
        round_segments = 2
    );
    rbox(
        f,
        "top",
        0.6,
        0.014,
        0.6,
        (0, 0.853, -0.16),
        M.body,
        0.006,
        1,
        ZERO,
    );
    rbox(
        f,
        "plinth",
        0.56,
        0.02,
        0.56,
        (0, 0.01, -0.16),
        M.dark,
        0.006,
        1,
        ZERO,
    );
    fbox!(f, "kick", 0.5, 0.012, 0.002, (0, 0.035, 0.1405), M.dark);
    rbox(
        f,
        "strip",
        0.6,
        0.13,
        0.012,
        (0, 0.78, 0.144),
        M.panel,
        0.006,
        1,
        ZERO,
    );
    // the soap drawer (washer) or the water tank (dryer): a front with a scoop grip, its compartments' marks
    rbox(
        f,
        "drawer",
        0.14,
        0.07,
        0.012,
        (-0.18, 0.78, 0.152),
        M.body,
        0.005,
        1,
        ZERO,
    );
    rbox(
        f,
        "drawer_grip",
        0.08,
        0.012,
        0.006,
        (-0.18, 0.759, 0.159),
        M.dark,
        0.003,
        1,
        ZERO,
    );
    for k in 0..3 {
        fbox!(
            f,
            format!("drawer_mark{k}"),
            0.012,
            0.004,
            0.001,
            (-0.225 + 0.03 * k as f64, 0.8, 0.1585),
            M.dark
        );
    }
    // the display: the time left, the light by it, three buttons under
    rbox(
        f,
        "display",
        0.12,
        0.05,
        0.006,
        (0.02, 0.785, 0.153),
        M.dark,
        0.004,
        1,
        ZERO,
    );
    for (k, w) in [0.012, 0.004, 0.012, 0.012].into_iter().enumerate() {
        fbox!(
            f,
            format!("time{k}"),
            w,
            0.018,
            0.001,
            (-0.005 + 0.016 * k as f64, 0.79, 0.1562),
            led
        );
    }
    fbox!(f, "light", 0.03, 0.006, 0.002, (0.02, 0.769, 0.157), led);
    for k in 0..3 {
        front_disc(
            f,
            format!("button{k}"),
            &[
                (0.0, 0.0),
                (0.008, 0.0),
                (0.008, 0.004),
                (0.006, 0.006),
                (0.0, 0.006),
            ],
            -0.02 + 0.04 * k as f64,
            0.738,
            0.15,
            M.body,
            8,
        );
    }
    // the program knob's ring of settings
    front_disc(
        f,
        "knob_skirt",
        &[
            (0.0, 0.0),
            (0.046, 0.0),
            (0.046, 0.004),
            (0.04, 0.006),
            (0.0, 0.006),
        ],
        0.18,
        0.78,
        0.15,
        M.panel,
        24,
    );
    for k in 0..10 {
        let a = radians(-150.0 + k as f64 * 300.0 / 9.0);
        fbox!(
            f,
            format!("setting{k}"),
            0.004,
            0.004,
            0.001,
            (0.18 + 0.056 * sin(a), 0.78 + 0.056 * cos(a), 0.1505),
            if k % 3 != 0 { M.dark } else { led }
        );
    }
    // the bellows: a rubber lip round the opening, behind the door
    front_disc(
        f,
        "seal",
        &[
            (0.205, -0.004),
            (0.205, 0.0),
            (0.19, 0.004),
            (0.172, 0.0),
            (0.168, -0.01),
            (0.15, -0.012),
            (0.205, -0.004),
        ],
        0,
        DOOR_Y,
        0.146,
        M.dark,
        24,
    );
    if dryer {
        // the vents along the foot, the condenser's hatch beside them
        for (k, y) in [0.075, 0.1, 0.125].into_iter().enumerate() {
            rbox(
                f,
                format!("vent{k}"),
                0.4,
                0.008,
                0.01,
                (0, y, 0.142),
                M.door_dryer,
                0.003,
                1,
                ZERO,
            );
        }
    } else {
        rbox(
            f,
            "filter",
            0.12,
            0.06,
            0.008,
            (0.2, 0.08, 0.142),
            M.panel,
            0.004,
            1,
            ZERO,
        );
        fbox!(
            f,
            "filter_slot",
            0.04,
            0.006,
            0.002,
            (0.2, 0.1, 0.1465),
            M.dark
        );
    }
}

/// The door: a turned ring with its window domed in it and a grip on the right, hinged on its left edge.
fn door(f: &GameFrame, ring: Mat) -> Obj {
    let parts = new_parts(|| {
        front_disc(
            f,
            "door_ring",
            &[
                (DOOR_R - 0.035, DOOR_Z - 0.006),
                (DOOR_R + 0.008, DOOR_Z - 0.006), // its back edge eased
                (DOOR_R + 0.012, DOOR_Z - 0.002),
                (DOOR_R + 0.012, DOOR_Z + 0.006),
                (DOOR_R + 0.004, DOOR_Z + 0.022),
                (DOOR_R - 0.012, DOOR_Z + 0.03),
                (DOOR_R - 0.026, DOOR_Z + 0.026),
                (DOOR_R - 0.035, DOOR_Z + 0.014),
                (DOOR_R - 0.035, DOOR_Z - 0.006),
            ],
            0,
            DOOR_Y,
            0,
            ring,
            32,
        );
        // the grip: a bar standing out of the ring's right side
        fsweep!(
            f,
            "handle",
            &fillet(
                &[
                    (DOOR_R - 0.005, DOOR_Y - 0.045, DOOR_Z + 0.012),
                    (DOOR_R + 0.012, DOOR_Y - 0.045, DOOR_Z + 0.03),
                    (DOOR_R + 0.012, DOOR_Y + 0.045, DOOR_Z + 0.03),
                    (DOOR_R - 0.005, DOOR_Y + 0.045, DOOR_Z + 0.012),
                ],
                0.012,
                3
            ),
            0.0,
            ring,
            profile = rounded_rect(0.012, 0.022, 0.005, 1),
            up = (0, 1, 0),
            angle = 50.0
        );
        front_disc(
            f,
            "window",
            &[
                (0.0, DOOR_Z + 0.028),
                (DOOR_R - 0.08, DOOR_Z + 0.024),
                (DOOR_R - 0.03, DOOR_Z + 0.012),
                (DOOR_R - 0.025, DOOR_Z),
                (DOOR_R - 0.04, DOOR_Z - 0.004),
                (0.0, DOOR_Z - 0.004),
            ],
            0,
            DOOR_Y,
            0,
            M.glass,
            32,
        );
    });
    let door = join(&parts, "door", None);
    hinged(door, f.at(-DOOR_R - 0.02, DOOR_Y, DOOR_Z))
}

/// The drum's face behind the window: its dark inside with a ring of holes, three lifters and the washing,
/// joined round the middle so it turns about its axis.
fn drum(f: &GameFrame, washing: &[(Mat, f64, f64, f64)], seed: i32) -> Obj {
    let parts = new_parts(|| {
        artkit::fcyl!(
            f,
            "drum_face",
            0.19,
            0.006,
            (0, DOOR_Y, 0.146),
            M.drum,
            rot = (90, 0, 0),
            verts = 32
        );
        for k in 0..12 {
            let a = TAU * k as f64 / 12.0;
            fbox!(
                f,
                format!("hole{k}"),
                0.008,
                0.008,
                0.001,
                (0.165 * cos(a), DOOR_Y + 0.165 * sin(a), 0.1492),
                M.paddle,
                rot = (0.0, degrees(a), 0.0)
            );
        }
        for k in 0..3 {
            let a = 120 * k;
            let (x, y) = (0.14 * sind(a as f64), DOOR_Y + 0.14 * cosd(a as f64));
            fextrude!(
                f,
                format!("paddle{k}"),
                &smooth_outline(
                    &[(-0.016, -0.04), (0.016, -0.04), (0.01, 0.04), (-0.01, 0.04)],
                    1,
                    true
                ),
                0.012,
                (x, y, 0.153),
                M.paddle,
                rot = (0, -a, 0),
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for (k, &(m, dx, dy, r)) in washing.iter().enumerate() {
            lump(
                f,
                format!("wash{k}"),
                dx,
                DOOR_Y + dy,
                0.153,
                r,
                m,
                (1.0, 0.18, 0.8),
                (seed + k as i32) as f64,
                (15 * k) as f64,
            );
        }
    });
    let drum = join(&parts, "drum", None);
    hinged(drum, f.at(0, DOOR_Y, 0.146))
}

fn machine(dryer: bool) -> Vec<Obj> {
    let f = &GameFrame::new(0.03); // built 3 cm into the wall (as the old part was); moved onto its face
    let (led, ring) = if dryer {
        (M.led_dryer, M.door_dryer)
    } else {
        (M.led, M.door)
    };
    let still = new_parts(|| {
        cabinet(f, led, dryer, ring);
        front_disc(
            f,
            "knob",
            &[
                (0.0, 0.0),
                (0.033, 0.0),
                (0.034, 0.014),
                (0.03, 0.026),
                (0.0, 0.028),
            ],
            0.18,
            0.78,
            0.154,
            ring,
            18,
        );
        rbox(
            f,
            "knob_pointer",
            0.006,
            0.026,
            0.006,
            (0.18, 0.795, 0.181),
            M.dark,
            0.002,
            1,
            ZERO,
        );
        if dryer {
            // a wicker basket of dry washing on the top, its weave in bands, handles at its ends
            revolve!(
                "basket",
                &eased(
                    &[
                        (0.0, 0.0),
                        (0.185, 0.0),
                        (0.21, 0.18),
                        (0.195, 0.18),
                        (0.17, 0.02),
                        (0.0, 0.02)
                    ],
                    0.004
                ),
                at = f.at(-0.06, 0.86, -0.14),
                material = M.wicker_plain,
                segments = 24,
                scale = (1, 0.76, 1)
            );
            for (k, (yy, r)) in [(0.05, 0.193), (0.1, 0.199), (0.15, 0.205)]
                .into_iter()
                .enumerate()
            {
                let lp: Vec<(f64, f64, f64)> = (0..24)
                    .map(|i| {
                        (
                            -0.06 + r * cos(TAU * i as f64 / 24.0),
                            0.86 + yy,
                            -0.14 + 0.76 * r * sin(TAU * i as f64 / 24.0),
                        )
                    })
                    .collect();
                fsweep!(
                    f,
                    format!("basket_band{k}"),
                    &lp,
                    0.004,
                    M.wicker_band,
                    verts = 4,
                    closed = true
                );
            }
            sphere!(
                "pile",
                0.15,
                at = f.at(-0.06, 1.0, -0.14),
                material = M.wash_c,
                scale = (1, 0.75, 0.38),
                segments = 16,
                rings = 8
            );
            lump(
                f,
                "pile_top",
                0.02,
                1.05,
                -0.1,
                0.07,
                M.wash_a,
                (1.1, 0.9, 0.5),
                4.0,
                0.0,
            );
            lump(
                f,
                "pile_sock",
                -0.15,
                1.04,
                -0.08,
                0.04,
                M.wash_b,
                (1.2, 0.6, 0.5),
                7.0,
                0.0,
            );
        } else {
            // folded towels on the top
            folded(f, "fold0", 0.08, 0.86, -0.1, 0.3, 0.22, 0.06, M.wash_c, 0.0);
            folded(
                f, "fold1", 0.08, 0.92, -0.1, 0.28, 0.2, 0.056, M.wash_b, 4.0,
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let washing = if dryer {
        [
            (M.wash_c, -0.05, -0.06, 0.08),
            (M.wash_b, 0.06, -0.08, 0.07),
            (M.wash_a, 0.0, 0.07, 0.06),
        ]
    } else {
        [
            (M.wash_a, -0.04, -0.07, 0.08),
            (M.wash_b, 0.07, -0.05, 0.06),
            (M.wash_c, 0.02, 0.06, 0.07),
        ]
    };
    let drum = drum(f, &washing, if dryer { 1 } else { 0 });
    let door = door(f, ring);
    settle!(centre = false);
    swing!(door, "door_open", 110, seconds = 1.0, ease = Ease::BackOut);
    // (The dryer tumbles slower than the washer turns.)
    spin(
        drum,
        "drum_spin",
        Spin {
            turns: 1.0,
            seconds: if dryer { 1.8 } else { 1.2 },
            axis: Axis::Y,
        },
    );
    meshes()
}

pub fn laundry_washer() -> Vec<Obj> {
    machine(false)
}

pub fn laundry_dryer() -> Vec<Obj> {
    machine(true)
}
