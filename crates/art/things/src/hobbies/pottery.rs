//! H3: the pottery wheel: a low, narrow motor body on four feet under a 0.48 m splash pan, the wheel head and a pot on
//! it, a foot pedal by the right foot (game +x: the potter faces the front, +z) and a stool behind. The potter sits
//! with the knees either side of the pan (characters.ts `activityPose`, `tinker`), the shins either side of the body, as at a
//! real wheel (review I-hobbies-10: the knees went into a 0.66 m drum). The pot is its own part and rises from a squat
//! lump while a resident throws (`clay`); the wheel head turns for as long as they do (`wheel`, a loop).

use super::M;
use super::common::stool;
use artkit::forms::smooth_outline;
use artkit::json::float_repr as r;
use artkit::kit;
use artkit::math::{Axis, P3};
use artkit::motion::{Grow, Spin, grow, spin};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, jitter, revolve, soft};

const WHEEL_Z: f64 = 0.12;
const HEAD_TOP: f64 = 0.475;
/// The splash pan's rim.
const PAN_R: f64 = 0.24;
/// The pot's height at rest, as a share of its height thrown.
const LUMP: f64 = 0.3;

pub fn pottery_wheel() -> Vec<Obj> {
    // 2 cm forward: the stool's back on the wall face
    let f = GameFrame::new(0.02);
    let still = new_parts(|| {
        // the body: a narrow rounded housing under the pan, set forward, on four rubber feet, louvres in its front
        soft!(
            f,
            "base",
            0.26,
            0.3,
            0.42,
            (0, 0.19, WHEEL_Z + 0.08),
            M.wheel_body,
            round = 0.03,
            round_segments = 2
        );
        fbox!(
            f,
            "base_plinth",
            0.3,
            0.025,
            0.46,
            (0, 0.04, WHEEL_Z + 0.08),
            M.wheel_body,
            bevel = 0.006
        );
        for sx in [-1, 1] {
            for z in [-0.1, 0.26] {
                fcyl!(
                    f,
                    format!("foot{sx}{}", r(z)),
                    0.022,
                    0.028,
                    (sx as f64 * 0.12, 0.014, WHEEL_Z + z),
                    M.foot,
                    verts = 10
                );
            }
        }
        for k in 0..4 {
            fbox!(
                f,
                format!("vent{k}"),
                0.16,
                0.008,
                0.004,
                (0, 0.17 + 0.022 * k as f64, WHEEL_Z + 0.291),
                M.foot
            );
        }
        fbox!(
            f,
            "badge",
            0.08,
            0.02,
            0.003,
            (0, 0.28, WHEEL_Z + 0.2915),
            M.chrome,
            bevel = 0.0015,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "column",
            0.06,
            0.06,
            (0, 0.37, WHEEL_Z),
            M.wheel_body,
            verts = 16
        );
        // the splash pan: a basin with a rolled rim, in two halves (the seam across it), a drain plug
        let pan = [
            (0.0, -0.5),
            (0.3, -0.5),
            (0.42, -0.2),
            (0.48, 0.42),
            (0.5, 0.5),
            (0.48, 0.56),
            (0.455, 0.5),
            (0.44, 0.45),
            (0.36, -0.15),
            (0.25, -0.35),
            (0.0, -0.35),
        ];
        let prof: Vec<(f64, f64)> = pan
            .iter()
            .map(|&(r, z)| (r * PAN_R * 2.0, z * 0.12))
            .collect();
        let prof = crate::common::round_at(&prof, 4, 0.003); // the rim's roll round
        revolve!(
            "splash_pan",
            &prof,
            at = f.at(0, 0.42, WHEEL_Z),
            material = M.pan,
            segments = 28
        );
        fbox!(
            f,
            "pan_seam",
            0.004,
            0.004,
            0.46,
            (0, 0.462, WHEEL_Z),
            M.wheel_body
        );
        fcyl!(
            f,
            "drain",
            0.012,
            0.012,
            (0.12, 0.4, WHEEL_Z - 0.14),
            M.foot,
            verts = 8
        );
        fcyl!(
            f,
            "spindle",
            0.04,
            0.03,
            (0, 0.43, WHEEL_Z),
            M.chrome,
            verts = 12
        );
        // the pedal on its cable, a speed switch, tools: a sponge, a wire, a rib
        fextrude!(
            f,
            "pedal",
            &[
                (0.1, 0.0),
                (-0.1, 0.0),
                (-0.1, 0.02),
                (-0.06, 0.05),
                (0.1, 0.05)
            ],
            0.12,
            (0.3, 0, 0.0),
            M.foot,
            plane = Facing::Side,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "pedal_tread",
            0.1,
            0.004,
            0.14,
            (0.3, 0.052, 0.02),
            M.wheel_body,
            rot = (-6, 0, 0)
        );
        kit::tube!(
            f,
            "pedal_cable",
            &[
                (0.3, 0.01, 0.1),
                (0.25, 0.012, 0.16),
                (0.15, 0.03, WHEEL_Z + 0.08),
                (0.13, 0.05, WHEEL_Z + 0.08)
            ],
            0.005,
            M.foot,
            verts = 6,
            round = 3
        );
        fbox!(
            f,
            "switch",
            0.004,
            0.04,
            0.06,
            (0.132, 0.25, WHEEL_Z + 0.2),
            M.foot,
            bevel = 0.002
        );
        fcyl!(
            f,
            "switch_knob",
            0.01,
            0.012,
            (0.138, 0.25, WHEEL_Z + 0.2),
            M.ball_red,
            rot = (0, 90, 0),
            verts = 8
        );
        fbox!(
            f,
            "sponge",
            0.06,
            0.025,
            0.04,
            (-0.17, 0.395, WHEEL_Z + 0.06),
            M.drill,
            bevel = 0.008,
            bevel_segments = 1
        );
        kit::tube!(
            f,
            "wire_tool",
            &[(0.12, 0.39, WHEEL_Z + 0.16), (0.18, 0.39, WHEEL_Z + 0.1)],
            0.0015,
            M.chrome,
            verts = 4,
            round = 1
        );
        for (k, (x, z)) in [(0.12, WHEEL_Z + 0.16), (0.18, WHEEL_Z + 0.1)]
            .into_iter()
            .enumerate()
        {
            fcyl!(
                f,
                format!("wire_toggle{k}"),
                0.005,
                0.03,
                (x, 0.392, z),
                M.stool,
                rot = (0, 90, 45),
                verts = 6
            );
        }
        fextrude!(
            f,
            "rib",
            &smooth_outline(
                &[
                    (0.0, -0.03),
                    (0.05, -0.02),
                    (0.06, 0.0),
                    (0.05, 0.02),
                    (0.0, 0.03),
                    (-0.02, 0.0)
                ],
                1,
                true
            ),
            0.004,
            (-0.15, 0.395, WHEEL_Z - 0.12),
            M.stool,
            plane = Facing::Top
        );
        stool(&f, -0.3, 0.44, 0.15, M.stool, M.stool, 0.0);
    });
    join(&still, "body", Some(P3::ZERO));
    let w = new_parts(|| {
        revolve!(
            "wheel_head",
            &[
                (0.0, 0.0),
                (0.15, 0.0),
                (0.168, 0.006),
                (0.17, 0.016),
                (0.17, 0.026),
                (0.165, 0.03),
                (0.0, 0.03)
            ],
            at = f.at(0, 0.445, WHEEL_Z),
            material = M.pan,
            segments = 32
        );
        fcyl!(
            f,
            "head_ring",
            0.135,
            0.001,
            (0, HEAD_TOP + 0.0005, WHEEL_Z),
            M.wheel_body,
            verts = 24,
            smooth = false
        );
        fcyl!(
            f,
            "slip",
            0.11,
            0.002,
            (0, HEAD_TOP + 0.001, WHEEL_Z),
            M.clay_slip,
            verts = 24,
            smooth = false
        );
        for s in [-1, 1] {
            fcyl!(
                f,
                format!("bat_pin{s}"),
                0.008,
                0.012,
                (s as f64 * 0.135, HEAD_TOP + 0.006, WHEEL_Z),
                M.chrome,
                verts = 8
            );
        }
        fbox!(
            f,
            "clay_smear",
            0.05,
            0.002,
            0.012,
            (0.0, HEAD_TOP + 0.001, WHEEL_Z + 0.135),
            M.clay_slip
        );
    });
    let wheel = hinged(join(&w, "wheel_head", None), f.at(0, HEAD_TOP, WHEEL_Z));
    let p = new_parts(|| {
        // the pot: a thrown body with throwing rings, its wall's thickness showing at the rim
        let rings = [
            (0.0, 0.0),
            (0.064, 0.0),
            (0.084, 0.02),
            (0.096, 0.05),
            (0.1, 0.09),
            (0.094, 0.12),
            (0.08, 0.15),
            (0.058, 0.176),
            (0.048, 0.186),
            (0.052, 0.2),
            (0.046, 0.2),
            (0.04, 0.188),
            (0.036, 0.18),
            (0.0, 0.176),
        ];
        let rings = crate::common::round_at(&rings, 9, 0.0015); // its lip round
        let pot = revolve!(
            "pot",
            &rings,
            at = f.at(0, HEAD_TOP, WHEEL_Z),
            material = M.clay,
            segments = 24
        );
        jitter!(pot, 0.0012, size = 0.03, seed = 1.7, up = 0.0);
    });
    let pot = join(&p, "pot", Some(f.at(0, HEAD_TOP, WHEEL_Z)));
    pot.edit(|o| o.scale.z = LUMP as f32);
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    grow(
        pot,
        "clay",
        Grow {
            seconds: 1.5,
            ..Grow::default()
        },
    );
    spin(
        wheel,
        "wheel",
        Spin {
            turns: 1.0,
            seconds: 1.0,
            axis: Axis::Z,
        },
    );
    meshes()
}
