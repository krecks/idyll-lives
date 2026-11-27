use super::super::common::{F, M, side_slab};
use super::super::seats::arm_board;
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, sin};
use artkit::motion::{Rock, rock};
use artkit::parts::{hinged, join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, revolve, settle, soft, torus};

// --- the porch swing: a slatted swing seat hung on chains from an A-frame; it swings while sat in --------------

/// Two A-frame ends at x ±0.92, their legs `spread` m out at the foot, and the beam across them: square legs cut
/// level at the foot and meeting under the beam, or round timbers tapering up; ties across; a beam with its ends
/// cut back underneath.
pub(super) fn a_frame(prefix: &str, beam_y: f64, material: Mat, round_legs: bool, spread: f64) {
    let tie = spread * (1.0 - 0.7 / beam_y);
    for s in [-1, 1] {
        let sf = s as f64;
        let x = sf * 0.92;
        for e in [-1, 1] {
            let ef = e as f64;
            if round_legs {
                fsweep!(
                    F,
                    format!("{prefix}leg{s}{e}"),
                    &[
                        (x, 0.0, ef * spread),
                        (x, beam_y * 0.5, ef * spread * 0.5),
                        (x, beam_y - 0.02, ef * 0.02)
                    ],
                    0.04,
                    material,
                    verts = 10,
                    scale = [1.0, 0.93, 0.86]
                );
            } else {
                let hw = 0.04 * artkit::py::hypot(spread, beam_y) / beam_y;
                let outline = if e > 0 {
                    [
                        (ef * spread - hw, 0.0),
                        (ef * spread + hw, 0.0),
                        (ef * hw * 0.2, beam_y),
                        (-ef * hw * 1.8, beam_y),
                    ]
                } else {
                    [
                        (ef * spread - hw, 0.0),
                        (ef * spread + hw, 0.0),
                        (hw * 1.8, beam_y),
                        (-hw * 0.2, beam_y),
                    ]
                };
                side_slab(
                    &format!("{prefix}leg{s}{e}"),
                    x,
                    &outline,
                    0.08,
                    material,
                    0.006,
                );
            }
        }
        kit::board(
            &F,
            &format!("{prefix}tie{s}"),
            (x, 0.7, -tie - 0.03),
            (x, 0.7, tie + 0.03),
            0.06,
            0.04,
            material,
            (0, 1, 0),
            0.008,
        );
        if !round_legs {
            // a gusset plate where the legs meet the beam
            side_slab(
                &format!("{prefix}gusset{s}"),
                x - sf * 0.046,
                &[
                    (-0.14, beam_y - 0.05),
                    (0.14, beam_y - 0.05),
                    (0.06, beam_y - 0.2),
                    (-0.06, beam_y - 0.2),
                ],
                0.012,
                material,
                0.003,
            );
        }
    }
    if round_legs {
        fsweep!(
            F,
            format!("{prefix}beam"),
            &[(-0.96, beam_y, 0), (0.96, beam_y, 0)],
            0.045,
            material,
            verts = 14
        );
        for s in [-1, 1] {
            fcyl!(
                F,
                format!("{prefix}beam_end{s}"),
                0.047,
                0.012,
                (s as f64 * 0.955, beam_y, 0),
                material,
                rot = (0, 90, 0),
                verts = 14
            );
        }
    } else {
        fextrude!(
            F,
            format!("{prefix}beam"),
            &[
                (-0.96, 0.05),
                (0.96, 0.05),
                (0.96, 0.0),
                (0.92, -0.05),
                (-0.92, -0.05),
                (-0.96, 0.0)
            ],
            0.1,
            (0, beam_y, 0),
            material,
            bevel = 0.008,
            bevel_segments = 1
        );
    }
}

// The seat hung 5 cm lower than it was; the cushion's top is the content's `seat` height.
pub(super) const PORCH_DROP: f64 = 0.05;
pub(in crate::outdoor) const PORCH_SEAT: f64 = 0.52;

pub fn porch_swing() -> Vec<Obj> {
    let frame = new_parts(|| a_frame("", 2.0, M.porch_frame, false, 0.42));
    join(&frame, "frame", Some(P3::from((0, 0, 0))));
    let d = PORCH_DROP;
    let seat = new_parts(|| {
        for k in 0..4 {
            fbox!(
                F,
                format!("seat{k}"),
                1.3,
                0.03,
                0.105,
                (0, 0.48 - d, -0.13 + 0.115 * k as f64),
                M.porch_seat,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        for k in 0..4 {
            fbox!(
                F,
                format!("back{k}"),
                1.3,
                0.08,
                0.025,
                (0, 0.6 - d + 0.1 * k as f64, -0.19 - 0.021 * k as f64),
                M.porch_seat,
                rot = (-12, 0, 0),
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        // the back's top rail, its top edge arched; the seat's front rail
        let mut arch = vec![(-0.7, -0.03), (0.7, -0.03)];
        arch.extend((0..11).map(|i| {
            (
                0.7 - 1.4 * i as f64 / 10.0,
                0.03 + 0.025 * sin(PI * i as f64 / 10.0),
            )
        }));
        fextrude!(
            F,
            "top_rail",
            &arch,
            0.03,
            (0, 0.98 - d, -0.275),
            M.porch_seat,
            rot = (-12, 0, 0),
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "front_rail",
            1.4,
            0.06,
            0.03,
            (0, 0.44 - d, 0.235),
            M.porch_seat,
            bevel = 0.008,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            arm_board(
                &format!("arm{s}"),
                sf * 0.68,
                0.7 - d,
                -0.21,
                0.25,
                0.06,
                0.08,
                M.porch_seat,
                0.03,
            );
            revolve!(
                format!("arm_post{s}"),
                &[
                    (0.0, 0.0),
                    (0.024, 0.0),
                    (0.024, 0.03),
                    (0.017, 0.05),
                    (0.02, 0.12),
                    (0.014, 0.18),
                    (0.022, 0.2),
                    (0.022, 0.22),
                    (0.0, 0.22)
                ],
                at = F.at(sf * 0.68, 0.47 - d, 0.21),
                material = M.porch_seat,
                segments = 10
            );
            fbox!(
                F,
                format!("side{s}"),
                0.05,
                0.06,
                0.48,
                (sf * 0.68, 0.45 - d, 0.03),
                M.porch_seat,
                bevel = 0.008,
                bevel_segments = 1
            );
            // two chains from the seat's corners to a ring, one chain up to the hook under the beam
            let ring = (sf * 0.67, 1.42, 0.02);
            for z in [-0.18, 0.24] {
                fsweep!(
                    F,
                    format!("chain{s}{}", float_repr(z)),
                    &[(sf * 0.66, 0.5 - d, z), ring],
                    0.006,
                    M.chain,
                    verts = 5
                );
                fcyl!(
                    F,
                    format!("eye{s}{}", float_repr(z)),
                    0.012,
                    0.03,
                    (sf * 0.66, 0.5 - d, z),
                    M.chain,
                    verts = 6
                );
            }
            torus!(
                format!("ring{s}"),
                0.022,
                0.006,
                at = F.at(ring.0, ring.1, ring.2),
                material = M.chain,
                rot = (0, 90, 0),
                major_segments = 10,
                minor_segments = 4
            );
            fsweep!(
                F,
                format!("chain_top{s}"),
                &[ring, (sf * 0.68, 1.93, 0.0)],
                0.006,
                M.chain,
                verts = 5
            );
            torus!(
                format!("hook{s}"),
                0.02,
                0.005,
                at = F.at(sf * 0.68, 1.95, 0.0),
                material = M.chain,
                rot = (0, 90, 0),
                major_segments = 10,
                minor_segments = 4
            );
        }
        soft!(
            F,
            "cushion",
            1.28,
            0.08,
            0.42,
            (0, PORCH_SEAT - 0.04, 0.05),
            M.porch_cushion,
            round = 0.03,
            puff = 0.01,
            flat = 2,
            piping = 0.005,
            round_segments = 1
        );
    });
    let seat = hinged(join(&seat, "seat", None), F.at(0, 1.98, 0));
    settle!(centre = false);
    rock(
        seat,
        "swing",
        10.0,
        Rock {
            seconds: 2.0,
            axis: Axis::X,
        },
    );
    meshes()
}
