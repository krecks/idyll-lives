//! K7: the dining sets: the dining table with its two chairs (base manifest), the bistro set, the glass and the
//! farmhouse dining sets.

use super::common::{apron, finish, rail, ring_loop, round_slab, slab, sq_leg, turned};
use super::settings::{SET_X, SET_Z, end_chair, flowers, place, runner};
use super::tables::turned_leg;
use super::{F, M};
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{TAU, cos, radians, sin};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fsweep, kit};

// --- the dining table (base manifest): an oak top to 0.76 on turned legs, a chair at each end facing in -----------
pub fn dining_table() -> Vec<Obj> {
    let (y, z) = (0.76, SET_Z);
    slab(
        "top",
        0.84,
        0.75,
        y,
        0.04,
        M.top_oak,
        0.02,
        None,
        3,
        (0.0, z),
    );
    for s in [-1, 1] {
        let sf = f64::from(s);
        apron(
            &format!("apron{s}"),
            0.67,
            0.07,
            0.022,
            (0.0, y - 0.04, z + sf * 0.304),
            M.legs_oakd,
            Facing::Front,
            0.018,
            10,
            0.003,
        );
        apron(
            &format!("apron_end{s}"),
            0.58,
            0.07,
            0.022,
            (sf * 0.349, y - 0.04, z),
            M.legs_oakd,
            Facing::Side,
            0.016,
            10,
            0.003,
        );
    }
    for x in [-0.36, 0.36] {
        for dz in [-0.315, 0.315] {
            turned(
                &format!("leg{}{}", float_repr(x), float_repr(dz)),
                &turned_leg(y - 0.04, 0.026, 0.018, None),
                (x, 0, z + dz),
                M.legs_oakd,
                12,
                35.0,
            );
        }
    }
    runner("runner", 0.2, y, z, 0.75, M.runner);
    flowers("", 0, z - 0.2, y + 0.004, 0.13, 5, true, 12);
    for s in [-1, 1] {
        place(
            &format!("p{s}"),
            f64::from(s) * 0.25,
            z,
            y,
            s,
            Some(M.runner),
        );
        end_chair(
            &format!("c{s}"),
            f64::from(s) * SET_X,
            z,
            M.legs_oakd,
            M.top_oak,
            M.seat_sage,
            false,
        );
    }
    finish("diningTable")
}

// --- the bistro set: a round oak café table and a black metal café chair behind it ----------------------------------
/// A café chair facing +z, its seat centred on (x, z): a round, stuffed seat on a ring, four legs bowed a little
/// out with a ring stretcher, back posts curving up into a looped back with a smaller loop inside it.
fn cafe_chair(prefix: &str, x: f64, z: f64, frame: Mat, pad: Mat) {
    let seat_h = 0.46;
    ring_loop(
        &format!("{prefix}seat_ring"),
        0.2,
        0.2,
        seat_h - 0.045,
        0.012,
        frame,
        x,
        z,
        5,
        20,
    );
    turned(
        &format!("{prefix}seat"),
        &[
            (0.0, 0.0),
            (0.196, 0.0),
            (0.205, 0.008),
            (0.207, 0.026),
            (0.2, 0.037),
            (0.18, 0.043),
            (0.12, 0.0447),
            (0.0, 0.045),
        ],
        (x, seat_h - 0.045, z),
        pad,
        20,
        35.0,
    );
    for (k, (sx, sz)) in [(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
        .into_iter()
        .enumerate()
    {
        let top = (x + sx * 0.13, seat_h - 0.045, z + sz * 0.13);
        let foot = (
            x + sx * 0.17,
            0.006,
            z + sz * 0.17 + if sz < 0.0 { 0.03 } else { 0.0 },
        );
        let mid = (
            (top.0 + foot.0) / 2.0,
            (top.1 + foot.1) / 2.0,
            (top.2 + foot.2) / 2.0,
        );
        let mid = (mid.0 + sx * 0.012, mid.1, mid.2 + sz * 0.012);
        kit::tube!(
            F,
            format!("{prefix}leg{k}"),
            &[foot, mid, top],
            0.011,
            frame,
            verts = 6,
            round = 2
        );
        turned(
            &format!("{prefix}foot{k}"),
            &[
                (0.0, 0.0),
                (0.012, 0.0),
                (0.014, 0.006),
                (0.011, 0.012),
                (0.0, 0.012),
            ],
            (foot.0, 0, foot.2),
            frame,
            6,
            35.0,
        );
    }
    ring_loop(
        &format!("{prefix}stretcher"),
        0.165,
        0.165,
        0.2,
        0.008,
        frame,
        x,
        z + 0.01,
        5,
        20,
    );
    let lean = radians(8.0);
    let hoop = |name: String, rx: f64, ry: f64, minor: f64, n: usize| {
        let pts: Vec<(f64, f64, f64)> = (0..n)
            .map(|i| {
                let a = TAU * i as f64 / n as f64;
                (
                    x + rx * cos(a),
                    0.75 + ry * sin(a) * cos(lean),
                    z - 0.21 - ry * sin(a) * sin(lean),
                )
            })
            .collect();
        fsweep!(F, name, &pts, minor, frame, verts = 5, closed = true)
    };
    hoop(format!("{prefix}back"), 0.16, 0.15, 0.012, 22);
    hoop(format!("{prefix}back_in"), 0.1, 0.09, 0.008, 16);
    for s in [-1, 1] {
        let sf = f64::from(s);
        kit::tube!(
            F,
            format!("{prefix}post{s}"),
            &[
                (x + sf * 0.13, seat_h - 0.045, z - 0.13),
                (x + sf * 0.15, 0.62, z - 0.175),
                (x + sf * 0.159, 0.75, z - 0.21),
            ],
            0.011,
            frame,
            verts = 8,
            round = 3
        );
        // the loops tied together at the sides
        kit::tube!(
            F,
            format!("{prefix}tie{s}"),
            &[
                (x + sf * 0.1, 0.75, z - 0.21),
                (x + sf * 0.16, 0.75, z - 0.21),
            ],
            0.006,
            frame,
            verts = 5
        );
    }
}

pub fn bistro_set() -> Vec<Obj> {
    let y = 0.75;
    let z = 0.17;
    round_slab(
        "top",
        0.31,
        y,
        0.035,
        M.top_oak,
        32,
        Some(&[(0.004, 0.0), (0.0, 0.4), (0.0015, 0.85), (0.004, 1.0)]),
        (0.0, z),
    );
    turned(
        "hub",
        &[
            (0.0, 0.0),
            (0.06, 0.0),
            (0.08, 0.012),
            (0.08, 0.03),
            (0.0, 0.03),
        ],
        (0, y - 0.065, z),
        M.legs_black,
        16,
        35.0,
    );
    turned(
        "column",
        &[
            (0.0, 0.0),
            (0.03, 0.0),
            (0.025, 0.03),
            (0.024, y - 0.15),
            (0.03, y - 0.13),
            (0.03, y - 0.12),
            (0.024, y - 0.11),
            (0.024, y - 0.08),
            (0.0, y - 0.08),
        ],
        (0, 0.01, z),
        M.legs_black,
        14,
        35.0,
    );
    turned(
        "foot",
        &[
            (0.0, 0.0),
            (0.205, 0.0),
            (0.21, 0.006),
            (0.205, 0.014),
            (0.16, 0.024),
            (0.09, 0.036),
            (0.05, 0.044),
            (0.04, 0.056),
            (0.0, 0.056),
        ],
        (0, 0, z),
        M.legs_black,
        24,
        35.0,
    );
    // a cup on its saucer and a little vase
    turned(
        "saucer",
        &[
            (0.0, 0.0),
            (0.04, 0.0),
            (0.045, 0.003),
            (0.068, 0.008),
            (0.07, 0.011),
            (0.066, 0.011),
            (0.045, 0.007),
            (0.0, 0.007),
        ],
        (0.12, y, 0.24),
        M.porcelain,
        14,
        35.0,
    );
    turned(
        "cup",
        &[
            (0.0, 0.0),
            (0.026, 0.0),
            (0.028, 0.004),
            (0.033, 0.03),
            (0.043, 0.07),
            (0.04, 0.07),
            (0.03, 0.03),
            (0.024, 0.012),
            (0.0, 0.012),
        ],
        (0.12, y + 0.007, 0.24),
        M.porcelain,
        12,
        35.0,
    );
    rail(
        "cup_handle",
        &[
            (0.152, y + 0.065, 0.24),
            (0.172, y + 0.06, 0.24),
            (0.172, y + 0.03, 0.24),
            (0.148, y + 0.025, 0.24),
        ],
        0.005,
        M.porcelain,
        0.012,
        5,
        3,
        false,
    );
    flowers("", -0.15, 0.1, y, 0.12, 3, false, 10);
    cafe_chair("c", 0.0, -0.19, M.legs_black, M.seat_sage); // its back on the wall face, as the old part's
    finish("bistroSet")
}

// --- the glass dining set: a glass top on two chrome sleds along its sides, a dark chair at each end ----------------
pub fn dining_glass() -> Vec<Obj> {
    let (y, z) = (0.76, SET_Z);
    slab(
        "top",
        0.9,
        0.8,
        y,
        0.02,
        M.glass,
        0.03,
        Some(&[(0.003, 0.0), (0.0, 0.3), (0.0, 0.7), (0.003, 1.0)]),
        3,
        (0.0, z),
    );
    for dz in [-0.3, 0.3] {
        // the sleds stand along the sides, the ends open for the knees
        rail(
            &format!("sled{}", float_repr(dz)),
            &[
                (-0.38, 0.018, z + dz),
                (0.38, 0.018, z + dz),
                (0.38, y - 0.032, z + dz),
                (-0.38, y - 0.032, z + dz),
            ],
            0.018,
            M.legs_chrome,
            0.06,
            12,
            3,
            true,
        );
        for x in [-0.3, 0.3] {
            turned(
                &format!("pad{}{}", float_repr(x), float_repr(dz)),
                &[
                    (0.0, 0.0),
                    (0.016, 0.0),
                    (0.02, 0.006),
                    (0.02, 0.012),
                    (0.0, 0.012),
                ],
                (x, y - 0.032, z + dz),
                M.legs_chrome,
                10,
                35.0,
            );
        }
        for x in [-0.3, 0.3] {
            // glides under the runners
            fbox!(
                F,
                format!("glide{}{}", float_repr(x), float_repr(dz)),
                0.05,
                0.004,
                0.03,
                (x, 0.002, z + dz),
                M.legs_chrome,
                bevel = 0.0015
            );
        }
    }
    kit::tube!(
        F,
        "rail",
        &[(0, y - 0.05, z - 0.3), (0, y - 0.05, z + 0.3)],
        0.012,
        M.legs_chrome,
        verts = 10
    );
    flowers("", 0, z - 0.22, y, 0.18, 5, true, 12);
    for s in [-1, 1] {
        place(&format!("p{s}"), f64::from(s) * 0.25, z, y, s, None);
        end_chair(
            &format!("c{s}"),
            f64::from(s) * SET_X,
            z,
            M.chair_dark,
            M.seat_grey,
            M.seat_grey,
            true,
        );
    }
    finish("diningGlass")
}

// --- the farmhouse dining set: a thick planked oak top with pegged breadboard ends on square legs, a runner ---------
pub fn dining_oak() -> Vec<Obj> {
    let (y, z) = (0.76, SET_Z);
    let edge = [
        (0.003, 0.0),
        (0.0, 0.25),
        (0.0, 0.8),
        (0.002, 0.95),
        (0.004, 1.0),
    ];
    for k in 0..5 {
        // five planks between the breadboard ends, rounded where they meet
        slab(
            &format!("plank{k}"),
            0.78,
            0.16,
            y,
            0.05,
            M.top_oak,
            0.004,
            Some(&edge),
            3,
            (0.0, z - 0.32 + 0.16 * f64::from(k)),
        );
    }
    for s in [-1, 1] {
        let sf = f64::from(s);
        slab(
            &format!("cleat{s}"),
            0.06,
            0.8,
            y,
            0.05,
            M.top_oak,
            0.012,
            None,
            3,
            (sf * 0.42, z),
        ); // breadboard ends
        for k in [-1, 0, 1] {
            fcyl!(
                F,
                format!("peg{s}{k}"),
                0.007,
                0.01,
                (sf * 0.42, y - 0.0045, z + f64::from(k) * 0.28),
                M.legs_oakd,
                verts = 8
            );
        }
        apron(
            &format!("apron{s}"),
            0.66,
            0.08,
            0.025,
            (0.0, y - 0.05, z + sf * 0.32),
            M.legs_oakd,
            Facing::Front,
            0.018,
            10,
            0.003,
        );
        apron(
            &format!("apron_end{s}"),
            0.6,
            0.08,
            0.025,
            (sf * 0.34, y - 0.05, z),
            M.legs_oakd,
            Facing::Side,
            0.018,
            10,
            0.003,
        );
    }
    for x in [-0.37, 0.37] {
        for dz in [-0.33, 0.33] {
            sq_leg(
                &format!("leg{}{}", float_repr(x), float_repr(dz)),
                x,
                z + dz,
                0.0,
                y - 0.05,
                0.06,
                0.08,
                M.legs_oakd,
                0.008,
                0.004,
                None,
            );
        }
    }
    runner("runner", 0.24, y, z, 0.8, M.runner);
    flowers("", 0, z - 0.2, y + 0.004, 0.18, 5, true, 12);
    for s in [-1, 1] {
        place(
            &format!("p{s}"),
            f64::from(s) * 0.27,
            z,
            y,
            s,
            Some(M.runner),
        );
        end_chair(
            &format!("c{s}"),
            f64::from(s) * SET_X,
            z,
            M.legs_oakd,
            M.top_oak,
            M.seat_sage,
            false,
        );
    }
    finish("diningOak")
}
