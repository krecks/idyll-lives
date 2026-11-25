//! The hush-shell reading pod and the shortwave radio desk.

use super::common::{_book, _chair, _desk, _mug, M, WALL};
use artkit::bevel::ease;
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{Axis, P3, PI, cos, cosd, pow, sin, sind};
use artkit::parts::{hinged, join, new_parts, numbered, settle};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, settle, slide, soft, swing, torus};

// --- the hush-shell reading pod (1x1): an upholstered shell chair on a swivel foot, a pram hood over it -------
const HOOD: (f64, f64) = (1.1, -0.02); // the hood's pivot (height, depth): it turns about a line across the pod's wings

// (3.1416: the script's own pi, to four places)
#[allow(clippy::approx_constant)]
pub fn quiet_pod() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        kit::turned!(
            f,
            "foot",
            0,
            0.0,
            -0.04,
            &[
                (0.0, 0.0),
                (0.24, 0.0),
                (0.25, 0.012),
                (0.24, 0.035),
                (0.2, 0.04),
                (0.0, 0.04),
            ],
            M.oak,
            segments = 28
        );
        kit::turned!(
            f,
            "stem",
            0,
            0.04,
            -0.04,
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.045, 0.03),
                (0.04, 0.13),
                (0.06, 0.16),
                (0.0, 0.16),
            ],
            M.chrome,
            segments = 18
        );
        // the upholstered shell: a bucket under the seat, a tall back up into the hood, wings with rolled front edges,
        // all piped where the panels meet
        soft!(
            f,
            "bucket",
            0.82,
            0.24,
            0.74,
            (0, 0.32, -0.06),
            M.shell,
            round = 0.1,
            piping = 0.006,
            flat = 1
        );
        soft!(
            f,
            "back",
            0.82,
            1.04,
            0.14,
            (0, 0.88, WALL + 0.07),
            M.shell,
            round = 0.06,
            flat = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            soft!(
                f,
                numbered("wing"),
                0.11,
                0.86,
                0.6,
                (sf * 0.355, 0.78, -0.13),
                M.shell,
                round = 0.05,
                puff = 0.01,
                flat = 2
            );
            let edge = fsweep!(
                f,
                numbered("wing_edge"),
                &[
                    (sf * 0.355, 0.35, 0.17),
                    (sf * 0.355, 0.78, 0.175),
                    (sf * 0.355, 1.21, 0.17)
                ],
                0.055,
                M.shell,
                verts = 14,
                resolution = 2
            );
            ease(edge, 60.0, 1);
            fsweep!(
                f,
                numbered("wing_piping"),
                &[(sf * 0.3, 0.36, 0.17), (sf * 0.3, 1.2, 0.17)],
                0.008,
                M.shell,
                verts = 5
            );
            soft!(
                f,
                numbered("wing_lining"),
                0.02,
                0.6,
                0.48,
                (sf * 0.292, 0.8, -0.12),
                M.cushion,
                round = 0.008,
                round_segments = 1,
                flat = 1
            );
            fcyl!(
                f,
                numbered("hood_boss"),
                0.05,
                0.03,
                (sf * 0.4, HOOD.0, HOOD.1),
                M.oak,
                rot = (0, 90, 0),
                verts = 16,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        // the cushions: the seat (top 0.45), a buttoned back cushion, a head roll
        let puff = 0.012;
        soft!(
            f,
            "seat",
            0.6,
            0.1 - puff,
            0.56,
            (0, 0.4 - puff / 2.0, 0.02),
            M.cushion,
            round = 0.045,
            puff = puff,
            piping = 0.005,
            flat = 3,
            resting = true
        );
        soft!(
            f,
            "back_cushion",
            0.58,
            0.56,
            0.1,
            (0, 0.76, -0.29),
            M.cushion,
            rot = (-6, 0, 0),
            round = 0.04,
            puff = 0.025,
            flat = 3,
            resting = true
        );
        for (u, v) in [
            (-0.14, 0.12),
            (0.14, 0.12),
            (0.0, 0.0),
            (-0.14, -0.12),
            (0.14, -0.12),
        ] {
            let out =
                0.05 + 0.025 * (1.0 - pow(u / 0.29, 2.0)) * (1.0 - pow(v / 0.28, 2.0)) - 0.008;
            fsphere!(
                f,
                numbered("button"),
                0.011,
                (u, 0.76 + v * cosd(6.0), -0.29 - v * sind(6.0) + out),
                M.headrest,
                scale = (1, 0.5, 1),
                segments = 6,
                rings = 4
            );
        }
        fsweep!(
            f,
            "head_cushion",
            &[(-0.2, 1.08, -0.31), (0.2, 1.08, -0.31)],
            0.055,
            M.headrest,
            verts = 12,
            scale = [0.9, 0.9]
        );
        for x in [-0.2, 0.2] {
            fsphere!(
                f,
                numbered("head_cushion_end"),
                0.05,
                (x, 1.08, -0.31),
                M.headrest,
                scale = (0.4, 1, 1),
                segments = 12,
                rings = 6
            );
        }
        // a book and a folded blanket on the seat, a gooseneck reading lamp on the right wing
        _book(
            f, "book", 0.18, 0.45, 0.16, 0.2, 0.03, 0.15, M.mustard, 15.0,
        );
        soft!(
            f,
            "blanket",
            0.36,
            0.03,
            0.26,
            (-0.1, 0.465, 0.13),
            M.headrest,
            rot = (0, 0, -8),
            round = 0.012,
            puff = 0.004,
            flat = 2
        );
        fsweep!(
            f,
            "blanket_fold",
            &[(-0.27, 0.468, 0.255), (0.07, 0.468, 0.21)],
            0.016,
            M.headrest,
            verts = 8
        );
        kit::tube!(
            f,
            "lamp_arm",
            &[
                (0.31, 1.0, -0.12),
                (0.31, 1.07, -0.08),
                (0.29, 1.08, -0.02),
                (0.27, 1.05, 0.01)
            ],
            0.008,
            M.chrome,
            verts = 6,
            round = 2
        );
        fcyl!(
            f,
            "lamp_clamp",
            0.016,
            0.03,
            (0.31, 1.0, -0.12),
            M.chrome,
            verts = 10
        );
        kit::turned!(
            f,
            "lamp",
            0.27,
            0.99,
            0.02,
            &[
                (0.04, 0.0),
                (0.038, 0.015),
                (0.028, 0.045),
                (0.012, 0.06),
                (0.0, 0.062),
            ],
            M.lamp,
            segments = 14
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The pram hood: an arc of padded channels over the back, about a line across the wings, a bound lip at its
    // front. `shell` swings it forward over the reader's head, half closing the pod's mouth.
    let hood = new_parts(|| {
        let (r, n) = (0.4, 6);
        for k in 0..n {
            let a = -90.0 + (k as f64 + 0.5) * 90.0 / n as f64; // from behind (-90) to straight up (0)
            let y = HOOD.0 + r * cosd(a);
            let z = HOOD.1 + r * sind(a);
            soft!(
                f,
                numbered("rib"),
                0.84,
                0.075,
                r * 3.1416 / 2.0 / n as f64 + 0.03,
                (0, y, z),
                M.shell,
                rot = (a, 0.0, 0.0),
                round = 0.03,
                puff = 0.012,
                round_segments = 2,
                flat = 2
            );
        }
        fsweep!(
            f,
            "hood_lip",
            &[
                (-0.42, HOOD.0 + r, HOOD.1 + 0.01),
                (0.42, HOOD.0 + r, HOOD.1 + 0.01)
            ],
            0.025,
            M.shell,
            verts = 10
        );
    });
    let hood = hinged(join(&hood, "hood", None), f.at(0, HOOD.0, HOOD.1));
    let objs = settle(&meshes()); // on its Model centre: the back on the wall face
    swing!(hood, "shell", 40.0, seconds = 0.9, axis = Axis::X);
    objs
}

// --- the shortwave radio desk (1x1): seated in the middle, the rig on a shallow desk before them --------------
const DIAL: (f64, f64, f64) = (0.15, 0.87, 0.243); // the tuning knob, on the receiver's face towards the operator
const ANTENNA: (f64, f64) = (0.2, 0.42); // the telescopic antenna's (x, depth), at the receiver's back corner

pub fn shortwave_radio() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _desk(f, 0.16, 0.45, 0.84, 0.74, M.oak, M.oak, 6);
        _chair(f, -0.02, M.oak, M.seatpad, 6);
        // the receiver: its face (a framed dial window with its scale, a meter, knobs, a speaker grille) towards the
        // operator, vents to the room, a carry handle on top
        fbox!(
            f,
            "receiver",
            0.5,
            0.24,
            0.2,
            (0.02, 0.86, 0.35),
            M.body,
            bevel = 0.02,
            bevel_segments = 2
        );
        fbox!(
            f,
            "face",
            0.46,
            0.2,
            0.01,
            (0.02, 0.86, 0.248),
            M.rig_face,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "dial_window",
            0.22,
            0.07,
            0.006,
            (-0.07, 0.92, 0.243),
            M.dial,
            bevel = 0.003,
            bevel_segments = 1
        );
        kit::moulding!(
            f,
            "dial_bezel",
            &[(0.04, 0.88), (0.04, 0.96), (-0.18, 0.96), (-0.18, 0.88)],
            &[(-0.008, 0.0), (-0.008, 0.004), (0.0, 0.006), (0.004, 0.0)],
            M.chrome,
            at = 0.243,
            plane = Facing::Front
        );
        for k in 0..9 {
            fbox!(
                f,
                numbered("scale_mark"),
                0.002,
                if k % 2 != 0 { 0.014 } else { 0.022 },
                0.002,
                (-0.16 + 0.02 * k as f64, 0.935, 0.2395),
                M.dark
            );
        }
        fbox!(
            f,
            "dial_needle",
            0.004,
            0.06,
            0.004,
            (-0.03, 0.92, 0.239),
            M.needle
        );
        fbox!(
            f,
            "meter",
            0.07,
            0.05,
            0.006,
            (0.15, 0.95, 0.243),
            M.cream,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "meter_needle",
            0.002,
            0.035,
            0.002,
            (0.155, 0.945, 0.2395),
            M.needle,
            rot = (0, -25, 0)
        );
        for x in [-0.15, -0.09, -0.03, 0.03] {
            kit::turned!(
                f,
                numbered("knob"),
                x,
                0.8,
                0.243,
                &[
                    (0.0, 0.0),
                    (0.016, 0.0),
                    (0.014, 0.014),
                    (0.01, 0.02),
                    (0.0, 0.021),
                ],
                M.dark,
                segments = 10,
                rot = (-90.0, 0.0, 0.0)
            );
        }
        let holes: Vec<Vec<(f64, f64)>> = (0..3)
            .map(|k| {
                rounded_rect(0.044, 0.008, 0.004, 1)
                    .iter()
                    .map(|&(u, v)| (u, v + 0.022 * k as f64 - 0.022))
                    .collect()
            })
            .collect();
        fextrude!(
            f,
            "grille",
            &rounded_rect(0.06, 0.07, 0.008, 2),
            0.006,
            (-0.2, 0.86, 0.243),
            M.dark,
            holes = holes
        );
        for k in 0..6 {
            // vents on its back, which faces the room
            fbox!(
                f,
                numbered("vent"),
                0.3,
                0.008,
                0.004,
                (0.0, 0.8 + 0.024 * k as f64, 0.452),
                M.dark
            );
        }
        fbox!(f, "foot_l", 0.04, 0.012, 0.18, (-0.19, 0.746, 0.35), M.dark);
        fbox!(f, "foot_r", 0.04, 0.012, 0.18, (0.23, 0.746, 0.35), M.dark);
        kit::tube!(
            f,
            "handle",
            &kit::fillet!(
                &[
                    (-0.12, 0.975, 0.35),
                    (-0.12, 1.02, 0.35),
                    (0.16, 1.02, 0.35),
                    (0.16, 0.975, 0.35)
                ],
                0.02
            ),
            0.009,
            M.chrome,
            verts = 6,
            round = 1
        );
        // the antenna's swivel and outer tube
        fsphere!(
            f,
            "antenna_swivel",
            0.018,
            (ANTENNA.0, 0.985, ANTENNA.1),
            M.chrome,
            segments = 8,
            rings = 5
        );
        kit::tube!(
            f,
            "antenna_tube",
            &[(ANTENNA.0, 0.99, ANTENNA.1), (ANTENNA.0, 1.24, ANTENNA.1)],
            0.008,
            M.chrome,
            verts = 6,
            round = 1
        );
        // the desk microphone, headphones with their cord, the logbook and pencil, a mug
        kit::turned!(
            f,
            "mic_base",
            -0.3,
            0.74,
            0.28,
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.048, 0.012),
                (0.02, 0.02),
                (0.0, 0.02),
            ],
            M.dark,
            segments = 14
        );
        kit::tube!(
            f,
            "mic_stem",
            &[(-0.3, 0.76, 0.28), (-0.29, 0.86, 0.275), (-0.28, 0.9, 0.27)],
            0.007,
            M.chrome,
            verts = 6,
            round = 2
        );
        kit::lathe!(
            f,
            "mic_capsule",
            -0.275,
            0.9,
            0.265,
            &[
                (0.012, 0.0),
                (0.024, 0.012),
                (0.026, 0.05),
                (0.022, 0.085),
                (0.008, 0.095),
            ],
            M.steel,
            verts = 12
        );
        torus!(
            "mic_band",
            0.026,
            0.004,
            at = f.at(-0.275, 0.94, 0.265),
            material = M.dark,
            major_segments = 12,
            minor_segments = 3
        );
        let band: Vec<(f64, f64, f64)> = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
            .iter()
            .map(|&t| (0.3, 0.77 + 0.075 * sin(PI * t), 0.25 + 0.085 * cos(PI * t)))
            .collect();
        fsweep!(
            f,
            "headphone_band",
            &band,
            0.0,
            M.dark,
            profile = rounded_rect(0.014, 0.022, 0.006, 1),
            up = (1, 0, 0),
            resolution = 2
        );
        for zz in [0.17, 0.33] {
            kit::turned!(
                f,
                numbered("earcup"),
                0.3,
                0.765,
                zz,
                &[
                    (0.0, 0.0),
                    (0.036, 0.0),
                    (0.038, 0.018),
                    (0.03, 0.036),
                    (0.0, 0.036),
                ],
                M.seatpad,
                segments = 12,
                rot = (if zz > 0.25 { 90.0 } else { -90.0 }, 0.0, 0.0)
            );
        }
        kit::tube!(
            f,
            "headphone_cord",
            &[
                (0.3, 0.77, 0.17),
                (0.25, 0.745, 0.19),
                (0.2, 0.745, 0.24),
                (0.14, 0.75, 0.3)
            ],
            0.003,
            M.dark,
            verts = 4,
            round = 2
        );
        _book(
            f, "logbook", -0.15, 0.74, 0.21, 0.16, 0.012, 0.2, M.cream, 10.0,
        );
        fsweep!(
            f,
            "pencil",
            &[(-0.13, 0.756, 0.15), (-0.07, 0.756, 0.27)],
            0.004,
            M.mustard,
            verts = 6,
            scale = [0.3, 1.0]
        );
        _mug(f, 0.33, 0.74, 0.4, M.terra, 200.0);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The tuning knob: a big ridged knob with a skirt and a pointer, turned by `dial`.
    let dial = new_parts(|| {
        kit::turned!(
            f,
            "dial_knob",
            DIAL.0,
            DIAL.1,
            DIAL.2,
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.045, 0.006),
                (0.036, 0.01),
                (0.036, 0.026),
                (0.032, 0.03),
                (0.0, 0.03),
            ],
            M.dark,
            segments = 20,
            rot = (-90.0, 0.0, 0.0)
        );
        fbox!(
            f,
            "dial_pointer",
            0.008,
            0.03,
            0.006,
            (DIAL.0, DIAL.1 + 0.02, DIAL.2 - 0.029),
            M.cream
        );
    });
    let dial = hinged(join(&dial, "dial", None), f.at(DIAL.0, DIAL.1, DIAL.2));
    // The antenna's inner section, down in the tube (and the case) at rest: `antenna` runs it up 45 cm.
    let ant = new_parts(|| {
        kit::tube!(
            f,
            "antenna_mid",
            &[(ANTENNA.0, 0.77, ANTENNA.1), (ANTENNA.0, 1.31, ANTENNA.1)],
            0.0055,
            M.chrome,
            verts = 6,
            round = 1
        );
        fsphere!(
            f,
            "antenna_ball",
            0.01,
            (ANTENNA.0, 1.315, ANTENNA.1),
            M.chrome,
            segments = 8,
            rings = 5
        );
    });
    let ant = hinged(join(&ant, "antenna", None), f.at(ANTENNA.0, 1.0, ANTENNA.1));
    let objs = settle!(centre = false);
    swing!(dial, "dial", 150.0, seconds = 1.0, axis = Axis::Y);
    slide!(ant, "antenna", (0.0, 0.0, 0.45), seconds = 1.0);
    objs
}
