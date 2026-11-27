//! The microwave counter.

use super::counter;
use crate::kitchen_cooking::M;
use crate::kitchen_cooking::common::{hollow, knob};
use artkit::forms::eased;
use artkit::forms::rounded_rect;
use artkit::math::Axis;
use artkit::motion::{Ease, Spin, spin};
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, revolve, soft, swing, torus};

// --- microwave counter: 1x1, a microwave on a small base unit ------------------------------------------

/// A microwave on a base unit (a drawer over two doors): a rounded charcoal body with vents down its side, a
/// control column with a display, buttons and a dial, a door with a window and a grip; inside, a bowl of soup on the
/// turntable over its roller ring.
pub fn microwave() -> Vec<Obj> {
    let f = &GameFrame::new(0.15);
    let still = new_parts(|| {
        counter(f, 1.0, None, None);
        // the microwave: a rounded body, hollowed through the front of its left part (the control column on the right
        // as the viewer sees it: game -x)
        let body = soft!(
            f,
            "micro_body",
            0.50,
            0.29,
            0.36,
            (-0.20, 1.105, -0.28),
            M.charcoal,
            round = 0.015,
            round_segments = 2
        );
        hollow(
            body,
            f,
            "micro_cavity",
            0.36,
            0.25,
            0.36,
            (-0.15, 1.10, -0.255),
            Some(M.plate),
        );
        for i in 0..8 {
            fbox!(
                f,
                format!("vent{i}"),
                0.003,
                0.008,
                0.16,
                (-0.451, 1.04 + 0.018 * i as f64, -0.33),
                M.dark
            );
        }
        for s in [-1, 1] {
            for k in [-1, 1] {
                fcyl!(
                    f,
                    format!("foot{s}{k}"),
                    0.012,
                    0.012,
                    (-0.2 + s as f64 * 0.2, 0.956, -0.28 + k as f64 * 0.14),
                    M.dark,
                    verts = 8
                );
            }
        }
        // the control column's face: a display, a grid of buttons, a dial and the door release
        fextrude!(
            f,
            "micro_face",
            &rounded_rect(0.11, 0.26, 0.01, 2),
            0.006,
            (-0.39, 1.105, -0.097),
            M.dark
        );
        fbox!(
            f,
            "micro_screen",
            0.08,
            0.04,
            0.004,
            (-0.39, 1.205, -0.093),
            M.hob,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "micro_digits",
            0.05,
            0.016,
            0.002,
            (-0.39, 1.205, -0.0905),
            M.display
        );
        for r in 0..3 {
            for c in 0..2 {
                fbox!(
                    f,
                    format!("button{r}{c}"),
                    0.03,
                    0.016,
                    0.004,
                    (-0.405 + 0.03 * c as f64, 1.16 - 0.025 * r as f64, -0.093),
                    M.chrome,
                    bevel = 0.0015,
                    bevel_segments = 1
                );
            }
        }
        knob(f, "dial", -0.39, 1.055, -0.087, 0.018, 0.014, None, true);
        fbox!(
            f,
            "release",
            0.07,
            0.022,
            0.006,
            (-0.39, 1.005, -0.092),
            M.chrome,
            bevel = 0.003,
            bevel_segments = 1
        );
        // the turntable's roller ring under the plate
        torus!(
            "rollers",
            0.08,
            0.005,
            at = f.at(-0.14, 0.976, -0.27),
            material = M.dark,
            major_segments = 14,
            minor_segments = 4
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the door, hinged on its right edge (seen from the front): a frame round its window, a grip by the controls
    let parts = new_parts(|| {
        fextrude!(
            f,
            "micro_door",
            &rounded_rect(0.38, 0.28, 0.012, 2),
            0.02,
            (-0.14, 1.1, -0.09),
            M.charcoal,
            holes = [rounded_rect(0.27, 0.19, 0.01, 2)],
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "micro_window",
            0.28,
            0.20,
            0.006,
            (-0.14, 1.10, -0.088),
            M.glass
        );
        fsweep!(
            f,
            "micro_grip",
            &[
                (-0.305, 1.03, -0.082),
                (-0.312, 1.04, -0.068),
                (-0.312, 1.16, -0.068),
                (-0.305, 1.17, -0.082)
            ],
            0.0,
            M.chrome,
            profile = rounded_rect(0.014, 0.01, 0.004, 1),
            up = (1, 0, 0),
            resolution = 2
        );
    });
    let door = hinged(join(&parts, "micro_door", None), f.at(0.05, 1.1, -0.09));
    // the turntable inside, with a bowl of soup on it, a mark so its turn shows
    let parts = new_parts(|| {
        let b = f.at(-0.14, 0.972, -0.27);
        revolve!(
            "turntable",
            &[
                (0.0, 0.0),
                (0.125, 0.0),
                (0.13, 0.008),
                (0.126, 0.012),
                (0.11, 0.006),
                (0.0, 0.006)
            ],
            at = (b.x, b.y, b.z),
            material = M.plate,
            segments = 20
        );
        fbox!(
            f,
            "turntable_mark",
            0.03,
            0.006,
            0.03,
            (-0.22, 0.98, -0.27),
            M.dark,
            bevel = 0.002
        );
        revolve!(
            "soup_bowl",
            &eased(
                &[
                    (0.0, 0.0),
                    (0.035, 0.0),
                    (0.055, 0.04),
                    (0.05, 0.04),
                    (0.032, 0.008),
                    (0.0, 0.008)
                ],
                0.002
            ),
            at = f.at(-0.12, 0.978, -0.26),
            material = M.cup,
            segments = 14
        );
        fcyl!(
            f,
            "soup",
            0.048,
            0.004,
            (-0.12, 1.012, -0.26),
            M.fruit,
            verts = 14
        );
    });
    let turn = join(&parts, "turntable", Some(f.at(-0.14, 0.978, -0.27)));
    settle_all();
    swing!(
        door,
        "door_open",
        -100,
        seconds = 1.0,
        axis = Axis::Z,
        ease = Ease::BackOut
    );
    spin(
        turn,
        "turntable",
        Spin {
            turns: 1.0,
            seconds: 2.0,
            axis: Axis::Z,
        },
    );
    Vec::new()
}
