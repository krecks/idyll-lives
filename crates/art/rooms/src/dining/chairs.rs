//! K8: chairs and stools. The stools' seats are a part of their own, swivelling on a hub; the chairs are still.

use super::common::{apron, crest, finish, rail, ring_loop, slab, sq_leg, stretcher, turned};
use super::{F, M, SZ};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{cos, cosd, pow, radians, sin, sind};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsweep, kit, settle, soft, swing};

pub(super) const UP: (f64, f64, f64) = (0.0, 0.0, 1.0); // `crest`'s up: Blender +Z

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
            let tilt = radians(8.0);
            fsweep!(
                F,
                "back",
                &[
                    (-0.18, seat_h + 0.24, z - 0.152),
                    (0.0, seat_h + 0.24, z - 0.178),
                    (0.18, seat_h + 0.24, z - 0.152)
                ],
                0.0,
                seat,
                profile = rounded_rect(0.14, 0.03, 0.013, 2),
                up = (0.0, sin(tilt), cos(tilt)),
                resolution = 5,
                angle = 60
            );
        }
    });
    let seat_part = hinged(join(&top, "seat", None), F.at(0, seat_h - 0.056, z));
    settle!(centre = false);
    swing!(seat_part, "swivel", 20.0, seconds = 0.5);
    meshes()
}

/// Bar stool (catalog): an oak stool with a sage seat, top at 0.74 (sitters at 0.72), centred.
pub fn bar_stool() -> Vec<Obj> {
    stool(
        0.74,
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

/// How far back a rail bowed by `bow` (its ends at ±xm) is at x.
pub(super) fn bowed(x: f64, xm: f64, bow: f64) -> f64 {
    bow * (1.0 - pow(x / xm, 2.0))
}

/// Dining chair (catalog): a painted ladder-back chair on turned legs with a tie-on piped pad, seat top 0.46:
/// turned front legs and back posts with finials, three bowed slats, arched aprons, turned stretchers.
pub fn dining_chair() -> Vec<Obj> {
    let z = 0.02;
    slab(
        "seat",
        0.44,
        0.42,
        0.46,
        0.04,
        M.frame_cream,
        0.03,
        None,
        3,
        (0.0, z),
    );
    soft!(
        F,
        "pad",
        0.38,
        0.025,
        0.36,
        (0, 0.4665, z + 0.01),
        M.seat_sage,
        round = 0.012,
        puff = 0.006,
        piping = 0.0035
    );
    let leg = [
        (0.0, 0.0),
        (0.013, 0.0),
        (0.0145, 0.014),
        (0.0145, 0.11),
        (0.019, 0.125),
        (0.019, 0.14),
        (0.0145, 0.155),
        (0.016, 0.27),
        (0.021, 0.29),
        (0.021, 0.305),
        (0.017, 0.32),
        (0.019, 0.42),
        (0.0, 0.42),
    ];
    for x in [-0.19, 0.19] {
        let xs = float_repr(x);
        turned(
            &format!("front{xs}"),
            &leg,
            (x, 0, 0.19),
            M.frame_cream,
            10,
            35.0,
        );
        kit::tube!(
            F,
            format!("back{xs}"),
            &[
                (x, 0.0, -0.17),
                (x, 0.42, -0.178),
                (x * 0.97, 0.7, -0.193),
                (x * 0.95, 0.955, -0.21),
            ],
            0.017,
            M.frame_cream,
            verts = 10,
            round = 3,
            scale = [0.9, 1.0, 0.95, 0.88]
        );
        turned(
            &format!("finial{xs}"),
            &[
                (0.0, 0.0),
                (0.015, 0.0),
                (0.019, 0.01),
                (0.012, 0.018),
                (0.021, 0.03),
                (0.017, 0.041),
                (0.006, 0.046),
                (0.0, 0.047),
            ],
            (x * 0.95, 0.955, -0.21),
            M.frame_cream,
            10,
            35.0,
        );
        // the pad's ties, knotted round the back post, their tails hanging
        ring_loop(
            &format!("tie{xs}"),
            0.021,
            0.021,
            0.468,
            0.004,
            M.seat_sage,
            x,
            -0.172,
            4,
            10,
        );
        for (k, dx) in [-0.012, 0.01].into_iter().enumerate() {
            fsweep!(
                F,
                format!("tail{xs}{k}"),
                &[
                    (x + dx, 0.468, -0.195),
                    (x + dx * 1.4, 0.44, -0.2),
                    (x + dx * 1.8, 0.405, -0.196)
                ],
                0.0,
                M.seat_sage,
                profile = rounded_rect(0.012, 0.003, 0.0012, 1),
                up = (0, 1, 0),
                resolution = 2
            );
        }
    }
    // three slats between the posts, bowed back round the sitter, the top one widest
    for (k, (y, h)) in [(0.885, 0.075), (0.75, 0.062), (0.615, 0.055)]
        .into_iter()
        .enumerate()
    {
        let zz = -0.17 - 0.04 * y / 0.96;
        crest(
            &format!("slat{k}"),
            0.178,
            y,
            zz,
            h,
            0.016,
            M.frame_cream,
            0.022,
            if k == 0 { 1.12 } else { 1.0 },
            UP,
            3,
        );
    }
    // aprons under the seat, arched between the legs; turned stretchers, the sides higher than the front and back
    apron(
        "apron_f",
        0.36,
        0.05,
        0.018,
        (0, 0.42, 0.185),
        M.frame_cream,
        Facing::Front,
        0.016,
        10,
        0.003,
    );
    apron(
        "apron_b",
        0.36,
        0.05,
        0.018,
        (0, 0.42, -0.165),
        M.frame_cream,
        Facing::Front,
        0.0,
        10,
        0.003,
    );
    for x in [-0.185, 0.185] {
        let xs = float_repr(x);
        apron(
            &format!("apron_s{xs}"),
            0.34,
            0.05,
            0.018,
            (x, 0.42, 0.01),
            M.frame_cream,
            Facing::Side,
            0.012,
            10,
            0.003,
        );
        stretcher(
            &format!("side{xs}"),
            (x, 0.24, 0.19),
            (x, 0.24, -0.172),
            0.009,
            M.frame_cream,
            1.3,
            8,
        );
    }
    for zz in [-0.172, 0.19] {
        stretcher(
            &format!("stretcher{}", float_repr(zz)),
            (-0.19, 0.16, zz),
            (0.19, 0.16, zz),
            0.009,
            M.frame_cream,
            1.3,
            8,
        );
    }
    finish("diningChair")
}

/// Wooden chair (catalog2): honey wood, a spindle back between two bowed rails, seat top 0.47: square tapered
/// front legs, back legs bending back into the posts, swelled spindles, arched aprons.
pub fn chair_wood() -> Vec<Obj> {
    let (x, zf, zb) = (0.19, 0.23, -0.095);
    slab(
        "seat",
        0.44,
        0.42,
        0.47,
        0.04,
        M.seat_honey,
        0.025,
        None,
        3,
        (0.0, SZ),
    );
    for s in [-1, 1] {
        let sx = f64::from(s) * x;
        sq_leg(
            &format!("front{s}"),
            sx,
            zf,
            0.0,
            0.43,
            0.026,
            0.036,
            M.frame_honey,
            0.006,
            0.003,
            None,
        );
        fsweep!(
            F,
            format!("back{s}"),
            &[
                (sx, 0.0, zb),
                (sx, 0.03, zb),
                (sx, 0.43, zb - 0.025),
                (sx, 0.66, zb - 0.045),
                (sx, 0.892, zb - 0.074)
            ],
            0.0,
            M.frame_honey,
            profile = rounded_rect(0.034, 0.03, 0.008, 1),
            up = (-1, 0, 0),
            resolution = 3,
            scale = [0.92, 0.92, 1.0, 1.0, 0.92],
            angle = 60
        );
        turned(
            &format!("cap{s}"),
            &[
                (0.0, 0.0),
                (0.018, 0.0),
                (0.018, 0.006),
                (0.012, 0.012),
                (0.0, 0.013),
            ],
            (sx, 0.889, zb - 0.074),
            M.frame_honey,
            8,
            35.0,
        );
    }
    // the rails, bowed back; five swelled spindles between them, following the bow
    let bow = 0.012;
    let (z_top, z_low) = (zb - 0.068, zb - 0.04);
    crest(
        "rail0",
        x - 0.008,
        0.835,
        z_top,
        0.1,
        0.026,
        M.frame_honey,
        bow,
        1.15,
        UP,
        3,
    );
    crest(
        "rail1",
        x - 0.008,
        0.6,
        z_low,
        0.05,
        0.026,
        M.frame_honey,
        bow,
        1.12,
        UP,
        3,
    );
    for (k, sx) in [-0.1, -0.05, 0.0, 0.05, 0.1].into_iter().enumerate() {
        let a = (sx, 0.615, z_low - bowed(sx, x, bow));
        let b = (sx, 0.79, z_top - bowed(sx, x, bow));
        stretcher(&format!("spindle{k}"), a, b, 0.008, M.frame_honey, 1.45, 8);
    }
    // aprons under the seat, arched; turned side and cross stretchers
    apron(
        "apron_f",
        2.0 * x - 0.03,
        0.06,
        0.02,
        (0, 0.45, zf - 0.005),
        M.frame_honey,
        Facing::Front,
        0.02,
        10,
        0.003,
    );
    apron(
        "apron_b",
        2.0 * x - 0.03,
        0.06,
        0.02,
        (0, 0.45, zb - 0.02),
        M.frame_honey,
        Facing::Front,
        0.0,
        10,
        0.003,
    );
    for s in [-1, 1] {
        let sf = f64::from(s);
        apron(
            &format!("apron_s{s}"),
            zf - zb - 0.03,
            0.06,
            0.02,
            (sf * (x - 0.004), 0.45, (zf + zb) / 2.0),
            M.frame_honey,
            Facing::Side,
            0.015,
            10,
            0.003,
        );
        stretcher(
            &format!("side{s}"),
            (sf * x, 0.24, zf),
            (sf * x, 0.24, zb - 0.01),
            0.011,
            M.frame_honey,
            1.3,
            8,
        );
        let zc = if s > 0 { zf } else { zb - 0.008 };
        stretcher(
            &format!("cross{s}"),
            (-x, 0.18, zc),
            (x, 0.18, zc),
            0.011,
            M.frame_honey,
            1.3,
            8,
        );
    }
    finish("chairWood")
}
