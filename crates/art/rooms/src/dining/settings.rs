//! What the dining sets are laid with and the chairs they're drawn with: place settings, flowers, a runner, the
//! sets' ladder-back chairs.

use super::common::{apron, crest, fillet, rail, slab, sq_leg, stretcher, tube, turned, v};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{Axis, M4, radians};
use artkit::parts::new_parts;
use artkit::place::{Coord, Facing};
use artkit::scene::Obj;
use artkit::{fextrude, fsweep, soft};

// --- table settings, vases, a runner ---------------------------------------------------------------------------------
const KNIFE: [(f64, f64); 9] = [
    (0.0, -0.0065),
    (0.088, -0.006),
    (0.094, -0.0085),
    (0.185, -0.0075),
    (0.2, 0.0),
    (0.192, 0.0085),
    (0.094, 0.009),
    (0.088, 0.0065),
    (0.0, 0.0065),
];
const FORK: [(f64, f64); 20] = [
    (0.0, -0.0065),
    (0.1, -0.0045),
    (0.13, -0.0125),
    (0.2, -0.0125),
    (0.2, -0.0082),
    (0.165, -0.0082),
    (0.165, -0.0058),
    (0.2, -0.0058),
    (0.2, -0.0012),
    (0.165, -0.0012),
    (0.165, 0.0012),
    (0.2, 0.0012),
    (0.2, 0.0058),
    (0.165, 0.0058),
    (0.165, 0.0082),
    (0.2, 0.0082),
    (0.2, 0.0125),
    (0.13, 0.0125),
    (0.1, 0.0045),
    (0.0, 0.0065),
];

/// A knife or a fork lying flat on a top at height `y`, its middle at (x, z), its tip towards `d` (±1 along x).
fn cutlery(name: &str, outline: &[(f64, f64)], x: f64, y: f64, z: f64, d: i32) -> Obj {
    let pts: Vec<(f64, f64)> = outline
        .iter()
        .map(|&(u, w)| (f64::from(d) * (u - 0.1), w))
        .collect();
    fextrude!(
        F,
        name,
        &pts,
        0.003,
        (x, y + 0.0015, z),
        M.cutlery,
        plane = Facing::Top
    )
}

/// A place setting at (x, z) on a top at height `y`, for the diner at the `s` end (-1: the left one, facing
/// +x): a rimmed plate with food on it, a knife at their right hand and a fork at their left (on a folded `napkin`),
/// a glass beyond the knife.
pub fn place(prefix: &str, x: f64, z: f64, y: f64, s: i32, napkin: Option<Mat>) {
    turned(
        &format!("{prefix}plate"),
        &[
            (0.0, 0.0),
            (0.07, 0.0),
            (0.075, 0.004),
            (0.106, 0.011),
            (0.12, 0.016),
            (0.118, 0.0185),
            (0.104, 0.0138),
            (0.073, 0.0072),
            (0.0, 0.0068),
        ],
        (x, y, z),
        M.porcelain,
        18,
        35.0,
    );
    turned(
        &format!("{prefix}food"),
        &[
            (0.0, 0.0),
            (0.066, 0.0),
            (0.062, 0.009),
            (0.045, 0.019),
            (0.02, 0.025),
            (0.0, 0.026),
        ],
        (x, y + 0.006, z),
        M.food,
        12,
        35.0,
    );
    let right = f64::from(-s); // facing +x (s -1), the right hand is towards +z
    let sf = f64::from(s);
    turned(
        &format!("{prefix}glass"),
        &[
            (0.0, 0.0),
            (0.025, 0.0),
            (0.026, 0.004),
            (0.034, 0.1),
            (0.031, 0.1),
            (0.0235, 0.012),
            (0.0, 0.012),
        ],
        (x - sf * 0.1, y, z + right * 0.17),
        M.tumbler,
        12,
        35.0,
    );
    cutlery(
        &format!("{prefix}knife"),
        &KNIFE,
        x,
        y,
        z + right * 0.15,
        -s,
    );
    let mut fy = y;
    if let Some(napkin) = napkin {
        soft!(
            F,
            format!("{prefix}napkin"),
            0.15,
            0.008,
            0.09,
            (x - sf * 0.005, y + 0.004, z - right * 0.155),
            napkin,
            round = 0.003,
            round_segments = 1
        );
        fy = y + 0.008;
    }
    cutlery(
        &format!("{prefix}fork"),
        &FORK,
        x,
        fy,
        z - right * 0.155,
        -s,
    );
}

/// A vase on a top at height `y`, with a lip and a foot, and `n` tulips (up to five) on curving stems, two
/// leaves (`leaves`).
#[allow(clippy::too_many_arguments)]
pub fn flowers<X: Coord>(
    prefix: &str,
    x: X,
    z: f64,
    y: f64,
    h: f64,
    n: usize,
    leaves: bool,
    segments: usize,
) {
    turned(
        &format!("{prefix}vase"),
        &[
            (0.0, 0.0),
            (0.032, 0.0),
            (0.036, 0.006),
            (0.034, 0.012),
            (0.05, 0.05),
            (0.046, 0.11),
            (0.03, h - 0.035),
            (0.027, h - 0.012),
            (0.033, h - 0.002),
            (0.031, h),
            (0.024, h),
            (0.022, h - 0.02),
            (0.0, h - 0.03),
        ],
        (x, y, z),
        M.vase,
        segments,
        35.0,
    );
    let xf = x.f();
    let blooms = [
        (-0.03, 0.1, 0.0, M.bloom_red),
        (0.035, 0.11, -0.02, M.bloom_yellow),
        (0.0, 0.09, 0.04, M.bloom_white),
        (0.02, 0.06, 0.03, M.bloom_red),
        (-0.04, 0.07, -0.03, M.bloom_white),
    ];
    for (k, &(dx, dh, dz, m)) in blooms[..n].iter().enumerate() {
        let (base, tip) = ((xf, y + h - 0.04, z), (xf + dx, y + h + dh - 0.02, z + dz));
        let mid = (xf + dx * 0.35, y + h + dh * 0.45, z + dz * 0.35);
        tube(
            &F,
            &format!("{prefix}stem{k}"),
            &[base, mid, tip],
            0.0035,
            M.stem,
            Some(5),
            2,
            1.0,
        );
        // a tulip: a cup of six petals, closed inside
        turned(
            &format!("{prefix}bloom{k}"),
            &[
                (0.0, 0.0),
                (0.012, 0.004),
                (0.021, 0.016),
                (0.024, 0.03),
                (0.019, 0.042),
                (0.015, 0.038),
                (0.0, 0.03),
            ],
            (tip.0, tip.1 - 0.002, tip.2),
            m,
            6,
            35.0,
        );
    }
    let leaf: &[(f64, f64)] = if leaves {
        &[(0.05, 0.01), (-0.045, -0.015)]
    } else {
        &[]
    };
    for (k, &(dx, dz)) in leaf.iter().enumerate() {
        fsweep!(
            F,
            format!("{prefix}leaf{k}"),
            &[
                (xf + dx * 0.2, y + h - 0.03, z + dz * 0.2),
                (xf + dx * 0.7, y + h + 0.03, z + dz * 0.7),
                (xf + dx * 1.1, y + h + 0.02, z + dz * 1.1)
            ],
            0.0,
            M.stem,
            profile = rounded_rect(0.002, 0.02, 0.001, 1),
            up = (0, 0, 1),
            resolution = 3,
            scale = [0.4, 1.0, 0.15]
        );
    }
}

/// A table runner `w` wide across a top `d` deep at height `y` (its top 4 mm up), hanging down over the front
/// and back edges by 7 cm.
pub fn runner(name: &str, w: f64, y: f64, z: f64, d: f64, material: Mat) -> Obj {
    let (drop, x) = (0.07, 0.0);
    let e = d / 2.0 + 0.003;
    let path = fillet(
        &[
            (x, y - drop, z - e),
            (x, y + 0.002, z - e),
            (x, y + 0.002, z + e),
            (x, y - drop, z + e),
        ],
        0.012,
        3,
        false,
    );
    fsweep!(
        F,
        name,
        &path,
        0.0,
        material,
        profile = rounded_rect(0.004, w, 0.0015, 1),
        up = (0, 1, 0)
    )
}

/// A dining set's chair at the origin, its sitter facing +z: a seat board with a rounded edge under a piped pad,
/// and a ladder back. Wood: square tapered front legs, back legs bending back into the posts, a bowed crest rail
/// and two bowed ladder rails, arched aprons and turned stretchers. Metal (`metal`): the back legs and posts bent
/// from one tube over the top, each front leg bent back into the seat's side rail, flat bars for the ladder.
fn ladder_chair(prefix: &str, frame: Mat, seat: Mat, pad: Mat, metal: bool) {
    let (seat_h, back_h) = (0.46, 0.92);
    slab(
        &format!("{prefix}seat"),
        0.42,
        0.4,
        seat_h - 0.02,
        0.04,
        seat,
        0.025,
        None,
        3,
        (0.0, 0.0),
    );
    soft!(
        F,
        format!("{prefix}pad"),
        0.38,
        0.03,
        0.36,
        (0, seat_h - 0.013, 0.01),
        pad,
        round = 0.012,
        puff = 0.008,
        round_segments = 1,
        flat = 2
    );
    let post_z = |y: f64| -0.15 - 0.05 * y / back_h;
    let up = (0.0, 0.0, 1.0);
    if metal {
        rail(
            &format!("{prefix}back_frame"),
            &[
                (-0.18, 0.0, -0.15),
                (-0.18, back_h, -0.2),
                (0.18, back_h, -0.2),
                (0.18, 0.0, -0.15),
            ],
            0.012,
            frame,
            0.05,
            8,
            3,
            false,
        );
        for s in [-1, 1] {
            let sx = f64::from(s) * 0.18;
            rail(
                &format!("{prefix}front{s}"),
                &[
                    (sx, 0.0, 0.17),
                    (sx, seat_h - 0.06, 0.165),
                    (sx, seat_h - 0.06, post_z(seat_h)),
                ],
                0.012,
                frame,
                0.03,
                8,
                3,
                false,
            );
        }
        tube(
            &F,
            &format!("{prefix}stretcher"),
            &[(-0.18, 0.16, 0.17), (0.18, 0.16, 0.17)],
            0.009,
            frame,
            Some(8),
            3,
            1.0,
        );
        for (k, (y, h)) in [(back_h - 0.25, 0.04), (back_h - 0.38, 0.04)]
            .into_iter()
            .enumerate()
        {
            crest(
                &format!("{prefix}rail{}", k + 1),
                0.18,
                y,
                post_z(y),
                h,
                0.008,
                frame,
                0.015,
                1.0,
                up,
                3,
            );
        }
    } else {
        for s in [-1, 1] {
            let sx = f64::from(s) * 0.18;
            sq_leg(
                &format!("{prefix}front{s}"),
                sx,
                0.17,
                0.0,
                seat_h - 0.06,
                0.024,
                0.034,
                frame,
                0.006,
                0.003,
                None,
            );
            fsweep!(
                F,
                format!("{prefix}back_leg{s}"),
                &[
                    (sx, 0.0, -0.15),
                    (sx, 0.03, -0.15),
                    (sx, 0.4, post_z(0.4) - 0.006),
                    (sx, 0.66, post_z(0.66) - 0.004),
                    (sx, back_h, -0.2)
                ],
                0.0,
                frame,
                profile = rounded_rect(0.034, 0.032, 0.008, 1),
                up = (-1, 0, 0),
                resolution = 2,
                scale = [0.92, 0.92, 1.0, 1.0, 0.9],
                angle = 60
            );
            stretcher(
                &format!("{prefix}side{s}"),
                (sx, 0.16, 0.17),
                (sx, 0.16, -0.152),
                0.009,
                frame,
                1.3,
                6,
            );
            apron(
                &format!("{prefix}apron_s{s}"),
                0.3,
                0.05,
                0.018,
                (f64::from(s) * 0.175, seat_h - 0.06, 0.01),
                frame,
                Facing::Side,
                0.012,
                6,
                0.003,
            );
        }
        apron(
            &format!("{prefix}apron_f"),
            0.33,
            0.05,
            0.018,
            (0.0, seat_h - 0.06, 0.165),
            frame,
            Facing::Front,
            0.014,
            6,
            0.003,
        );
        stretcher(
            &format!("{prefix}stretcher"),
            (-0.18, 0.2, 0.17),
            (0.18, 0.2, 0.17),
            0.009,
            frame,
            1.3,
            6,
        );
        for (k, (y, h)) in [(back_h - 0.25, 0.05), (back_h - 0.38, 0.05)]
            .into_iter()
            .enumerate()
        {
            crest(
                &format!("{prefix}rail{}", k + 1),
                0.172,
                y,
                post_z(y),
                h,
                0.02,
                frame,
                0.015,
                1.0,
                up,
                3,
            );
        }
    }
    let y = back_h - 0.07;
    crest(
        &format!("{prefix}rail0"),
        if metal { 0.19 } else { 0.185 },
        y,
        post_z(y) - if metal { 0.004 } else { 0.0 },
        0.1,
        0.022,
        seat,
        0.015,
        1.1,
        up,
        3,
    );
}

pub const SET_X: f64 = 0.62; // the sets for two: each chair's seat this far out from the middle, at its end of the top
pub const SET_Z: f64 = 0.1; // and this far forward, with the top's middle (content `places` [±0.62, 0.1], facing ±90)

/// A set's chair at the end of its top, seat centred on (x, z), its sitter facing in across the top (+x from
/// the left end, -x from the right): built facing the front, then turned a quarter about its seat's middle.
#[allow(clippy::too_many_arguments)]
pub fn end_chair(prefix: &str, x: f64, z: f64, frame: Mat, seat: Mat, pad: Mat, metal: bool) {
    let parts = new_parts(|| ladder_chair(prefix, frame, seat, pad, metal));
    let turn = M4::translation(v((x, 0, z)))
        * M4::rotation(radians(if x < 0.0 { -90.0 } else { 90.0 }), Axis::Z);
    for o in parts {
        super::common::set_matrix_world(o, &(turn * o.matrix_world()));
    }
}
