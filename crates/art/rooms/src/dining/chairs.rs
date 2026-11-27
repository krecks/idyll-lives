//! K8: the dining and wooden chairs, still.

use super::common::{apron, crest, finish, ring_loop, slab, sq_leg, stretcher, turned};
use super::{F, M, SZ};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::pow;
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fsweep, kit, soft};

pub(super) const UP: (f64, f64, f64) = (0.0, 0.0, 1.0); // `crest`'s up: Blender +Z

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
