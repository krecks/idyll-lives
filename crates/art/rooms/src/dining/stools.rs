//! K8: the stools. Their seats are a part of their own, swivelling on a hub.

use super::common::{bowed_pad, rail, ring_loop, turned};
use super::{F, M, SZ};
use artkit::materials::Mat;
use artkit::math::{cosd, sind};
use artkit::parts::{hinged, join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, kit, settle, swing};

// === K8: chairs and stools ===========================================================================================
/// A stool on four splayed legs in shoes and a foot ring, its seat (and back) a part of its own turning on a hub:
/// a seat board with a rounded edge under a stuffed cushion with a piped seam, its top at `seat_h`. `cap`: the
/// feet and the swivel plate (the frame's own material on a wood stool, so it draws in one group).
#[allow(clippy::too_many_arguments)]
pub(super) fn stool(
    seat_h: f64,
    z: f64,
    frame: Mat,
    seat: Mat,
    legs_r: f64,
    back: bool,
    cushion_h: f64,
    cap: Option<Mat>,
    metal: bool,
) -> Vec<Obj> {
    let verts = 24;
    let cap = cap.unwrap_or(frame);
    let base = new_parts(|| {
        for k in 0..4 {
            let a = f64::from(45 + 90 * k);
            let (c, s) = (cosd(a), sind(a));
            let (foot, top) = (
                (c * legs_r, 0.008, z + s * legs_r),
                (c * 0.12, seat_h - 0.06, z + s * 0.12),
            );
            if metal {
                kit::tube!(F, format!("leg{k}"), &[foot, top], 0.016, frame, verts = 10);
            } else {
                // turned: a long taper with a ring at the foot rest and a neck
                let pts: Vec<(f64, f64, f64)> = [0.0, 0.25, 0.33, 0.38, 0.43, 0.9, 1.0]
                    .iter()
                    .map(|&w| {
                        (
                            foot.0 + (top.0 - foot.0) * w,
                            foot.1 + (top.1 - foot.1) * w,
                            foot.2 + (top.2 - foot.2) * w,
                        )
                    })
                    .collect();
                kit::tube!(
                    F,
                    format!("leg{k}"),
                    &pts,
                    0.017,
                    frame,
                    verts = 10,
                    round = 2,
                    scale = [0.78, 0.86, 0.92, 1.18, 0.92, 1.05, 0.95]
                );
            }
            turned(
                &format!("shoe{k}"),
                &[
                    (0.0, 0.0),
                    (0.017, 0.0),
                    (0.019, 0.004),
                    (0.018, 0.014),
                    (0.015, 0.018),
                    (0.0, 0.018),
                ],
                (c * legs_r, 0.0, z + s * legs_r),
                cap,
                10,
                35.0,
            );
        }
        let ring_y = seat_h * 0.36; // the foot rest: where the renderer's `rest` puts the feet
        let k_r = legs_r - (legs_r - 0.12) * ring_y / (seat_h - 0.06);
        ring_loop("foot_ring", k_r, k_r, ring_y, 0.012, frame, 0.0, z, 8, 32);
        if metal {
            // a kick plate on the ring's front, where the shoes rest
            fbox!(
                F,
                "kick",
                0.12,
                0.004,
                0.03,
                (0, ring_y + 0.012, z + k_r - 0.006),
                frame,
                bevel = 0.0015
            );
        }
        turned(
            "hub",
            &[
                (0.0, 0.0),
                (0.135, 0.0),
                (0.15, 0.006),
                (0.148, 0.016),
                (0.11, 0.02),
                (0.0, 0.02),
            ],
            (0, seat_h - 0.075, z),
            frame,
            24,
            35.0,
        );
    });
    join(&base, "base", Some((0, 0, 0).into()));
    let top = new_parts(|| {
        turned(
            "swivel",
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.07, 0.006),
                (0.05, 0.012),
                (0.0, 0.012),
            ],
            (0, seat_h - 0.056, z),
            cap,
            16,
            35.0,
        );
        let y0 = seat_h - cushion_h - 0.025;
        turned(
            "seat_board",
            &[
                (0.0, 0.0),
                (0.18, 0.0),
                (0.192, 0.005),
                (0.197, 0.017),
                (0.194, 0.029),
                (0.186, 0.035),
                (0.0, 0.035),
            ],
            (0, y0, z),
            frame,
            verts,
            35.0,
        );
        let ch = cushion_h;
        turned(
            "cushion",
            &[
                (0.0, 0.0),
                (0.176, 0.0),
                (0.185, 0.004),
                (0.188, ch * 0.45),
                (0.185, ch - 0.01),
                (0.175, ch - 0.004),
                (0.14, ch - 0.0012),
                (0.07, ch - 0.0002),
                (0.03, ch - 0.0015),
                (0.012, ch - 0.005),
                (0.0, ch - 0.0055),
            ],
            (0, seat_h - ch, z),
            seat,
            verts,
            35.0,
        );
        turned(
            "button",
            &[
                (0.0, 0.0),
                (0.011, 0.0),
                (0.01, 0.003),
                (0.005, 0.0045),
                (0.0, 0.0048),
            ],
            (0, seat_h - 0.005, z),
            seat,
            8,
            35.0,
        );
        ring_loop(
            "piping",
            0.181,
            0.181,
            seat_h - 0.007,
            0.004,
            seat,
            0.0,
            z,
            5,
            verts,
        );
        if back {
            // the back's frame bent from one tube up behind the seat, and a curved, padded back board on it
            for s in [-1, 1] {
                let sf = f64::from(s);
                rail(
                    &format!("back_post{s}"),
                    &[
                        (sf * 0.12, seat_h - 0.04, z - 0.135),
                        (sf * 0.14, seat_h + 0.1, z - 0.17),
                        (sf * 0.14, seat_h + 0.27, z - 0.19),
                    ],
                    0.012,
                    frame,
                    0.08,
                    8,
                    3,
                    false,
                );
            }
            // a padded board, its ends rounded, bowed round the sitter and leant back 8°
            bowed_pad(
                "back",
                0.36,
                0.14,
                0.03,
                (0.0, seat_h + 0.24, z - 0.152),
                0.026,
                8.0,
                seat,
            );
        }
    });
    let seat_part = hinged(join(&top, "seat", None), F.at(0, seat_h - 0.056, z));
    settle!(centre = false);
    swing!(seat_part, "swivel", 20.0, seconds = 0.5);
    meshes()
}

/// Bar stool (catalog): an oak stool with a sage seat, top at 0.715 (sitters at 0.695), centred.
pub fn bar_stool() -> Vec<Obj> {
    // its top at 0.715: a counter stool's, a hand under the 0.95 worktops it stands at (EN 1116; sitters at 0.695)
    stool(
        0.715,
        0.0,
        M.frame_oak,
        M.seat_sage,
        0.21,
        false,
        0.05,
        None,
        false,
    )
}

/// Counter stool (catalog2): an oak stool, seat top at 0.685, 5 cm forward.
pub fn stool_counter() -> Vec<Obj> {
    stool(
        0.685,
        SZ,
        M.frame_oak,
        M.seat_sage,
        0.21,
        false,
        0.04,
        None,
        false,
    )
}

/// High bar stool (catalog2): black metal with a cognac seat and a low back, seat top at 0.805.
pub fn stool_bar_high() -> Vec<Obj> {
    stool(
        0.805,
        SZ,
        M.legs_black,
        M.leather,
        0.21,
        true,
        0.05,
        Some(M.cap),
        true,
    )
}
