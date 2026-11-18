//! The seats: the pupil's desk, the pew, the waiting bench and the couch.

use super::common::{F, M, SEAT, book, rod, tube};
use artkit::forms::smooth_outline;
use artkit::json::float_repr;
use artkit::math::{atan2, cos, degrees, radians, sin};
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, revolve, soft};

// --- the pupil's desk: 1x1, a lidded desk with an inkwell joined to its seat on a cast-iron frame -------------
pub fn school_desk() -> Vec<Obj> {
    let parts = new_parts(|| {
        // the cast-iron sides: a runner on the floor, the desk's leg, the seat's leg rising into the back post
        for s in [-1, 1] {
            let x = s as f64 * 0.27;
            tube(
                &format!("runner{s}"),
                &[(x, 0.016, -0.25), (x, 0.016, 0.46)],
                0.016,
                M.iron,
                0.03,
                8,
            );
            tube(
                &format!("desk_leg{s}"),
                &[(x, 0.016, 0.42), (x, 0.56, 0.42)],
                0.016,
                M.iron,
                0.03,
                8,
            );
            tube(
                &format!("seat_leg{s}"),
                &[(x, 0.016, -0.2), (x, 0.42, -0.2), (x, 0.82, -0.245)],
                0.016,
                M.iron,
                0.05,
                8,
            );
            tube(
                &format!("seat_arm{s}"),
                &[(x, 0.42, -0.2), (x, 0.42, 0.12)],
                0.013,
                M.iron,
                0.03,
                8,
            );
            tube(
                &format!("brace{s}"),
                &[(x, 0.016, 0.05), (x, 0.3, 0.2), (x, 0.56, 0.3)],
                0.012,
                M.iron,
                0.08,
                8,
            );
            fsphere!(
                F,
                format!("foot{s}a"),
                0.022,
                (x, 0.016, -0.25),
                M.iron,
                segments = 8,
                rings = 6
            );
            fsphere!(
                F,
                format!("foot{s}b"),
                0.022,
                (x, 0.016, 0.46),
                M.iron,
                segments = 8,
                rings = 6
            );
        }
        rod(
            "stretcher",
            (-0.27, 0.2, 0.2),
            (0.27, 0.2, 0.2),
            0.012,
            M.iron,
            8,
        );
        // the seat and the back's two slats
        fbox!(
            F,
            "seat",
            0.58,
            0.025,
            0.32,
            (0, 0.4325 + 0.0, -0.04),
            M.pine,
            bevel = 0.008
        );
        for (k, y) in [0.6, 0.75].into_iter().enumerate() {
            fbox!(
                F,
                format!("slat{k}"),
                0.58,
                0.08,
                0.022,
                (0, y, -0.225 - (y - 0.42) * 0.11 - 0.012),
                M.pine,
                bevel = 0.006,
                rot = (-6, 0, 0)
            );
        }
        // the desk: a box with a sloping lid towards the pupil, a flat strip at the far edge with the inkwell and a
        // groove for the pen
        let side = [
            (0.18, 0.56),
            (0.48, 0.56),
            (0.48, 0.7),
            (0.4, 0.7),
            (0.18, 0.665),
        ];
        for s in [-1, 1] {
            fextrude!(
                F,
                format!("box_side{s}"),
                &side,
                0.02,
                (s as f64 * 0.29, 0, 0),
                M.pine,
                plane = Facing::Side,
                bevel = 0.003
            );
        }
        fbox!(
            F,
            "box_front",
            0.6,
            0.14,
            0.02,
            (0, 0.63, 0.47),
            M.pine,
            bevel = 0.003
        );
        fbox!(
            F,
            "box_back",
            0.56,
            0.1,
            0.02,
            (0, 0.61, 0.19),
            M.pine,
            bevel = 0.003
        );
        fbox!(F, "box_floor", 0.56, 0.015, 0.29, (0, 0.567, 0.33), M.pine);
        fbox!(
            F,
            "strip",
            0.62,
            0.022,
            0.09,
            (0, 0.711, 0.445),
            M.pine,
            bevel = 0.005
        );
        let slope = degrees(atan2(0.035, 0.22));
        fbox!(
            F,
            "lid",
            0.62,
            0.02,
            0.235,
            (0, 0.692, 0.287),
            M.pine,
            bevel = 0.005,
            rot = (-slope, 0, 0)
        );
        fbox!(
            F,
            "groove",
            0.3,
            0.004,
            0.018,
            (-0.08, 0.7225, 0.43),
            M.soil
        );
        revolve!(
            "inkwell",
            &[
                (0.0, 0.0),
                (0.022, 0.0),
                (0.024, 0.006),
                (0.024, 0.02),
                (0.016, 0.026),
                (0.0, 0.026)
            ],
            at = F.at(0.2, 0.712, 0.445),
            material = M.pot,
            segments = 12
        );
        fcyl!(
            F,
            "ink",
            0.009,
            0.004,
            (0.2, 0.7385, 0.445),
            M.ink,
            verts = 10
        );
        rod(
            "pen",
            (-0.2, 0.7275, 0.43),
            (0.02, 0.7275, 0.432),
            0.004,
            M.ink,
            6,
        );
        book(
            "exercise",
            0.17,
            0.006,
            0.23,
            (-0.05, 0.705, 0.28),
            M.leaflet1,
            (-slope, 0.0, 8.0),
        );
    });
    join(&parts, "desk", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the pew: 3x1, an oak bench with a back, shaped ends and a book ledge behind ---------------------------------
pub fn pew() -> Vec<Obj> {
    let parts = new_parts(|| {
        fbox!(
            F,
            "seat",
            2.84,
            0.04,
            0.42,
            (0, SEAT - 0.02, -0.01),
            M.pew_oak,
            bevel = 0.012
        );
        fbox!(
            F,
            "seat_rail",
            2.84,
            0.08,
            0.03,
            (0, SEAT - 0.08, 0.18),
            M.pew_oak,
            bevel = 0.006
        );
        fbox!(
            F,
            "back",
            2.84,
            0.5,
            0.03,
            (0, SEAT + 0.27, -0.255),
            M.pew_oak,
            bevel = 0.006,
            rot = (-8, 0, 0)
        );
        fbox!(
            F,
            "back_rail",
            2.84,
            0.04,
            0.07,
            (0, SEAT + 0.53, -0.29),
            M.pew_oak,
            bevel = 0.015
        );
        fbox!(
            F,
            "stretcher",
            2.84,
            0.1,
            0.03,
            (0, 0.16, -0.1),
            M.pew_oak,
            bevel = 0.006
        );
        // the ledge for the books behind, for the row at the back
        fbox!(
            F,
            "ledge",
            2.84,
            0.02,
            0.12,
            (0, 0.8, -0.36),
            M.pew_oak,
            bevel = 0.004
        );
        fbox!(
            F,
            "ledge_lip",
            2.84,
            0.04,
            0.015,
            (0, 0.82, -0.415),
            M.pew_oak,
            bevel = 0.004
        );
        for k in 0..7 {
            let x = -1.25 + k as f64 * 0.4 + 0.03 * (k % 3) as f64;
            book(
                &format!("hymnal{k}"),
                0.13,
                0.03,
                0.1,
                (x, 0.825, -0.36),
                if k % 3 != 0 { M.book } else { M.mag1 },
                (0.0, 0.0, (90 - 6 * (k % 2)) as f64),
            );
        }
        // the ends: shaped boards rising to a scrolled top at the back, a raised field in each
        let end = smooth_outline(
            &[
                (-0.38, 0.0),
                (0.24, 0.0),
                (0.24, 0.6),
                (0.2, 0.66),
                (0.08, 0.68),
                (-0.04, 0.74),
                (-0.14, 0.86),
                (-0.2, 0.98),
                (-0.26, 1.04),
                (-0.34, 1.04),
                (-0.38, 0.98),
            ],
            2,
            true,
        );
        let field = smooth_outline(
            &[
                (-0.3, 0.12),
                (0.16, 0.12),
                (0.16, 0.56),
                (0.06, 0.6),
                (-0.06, 0.66),
                (-0.16, 0.78),
                (-0.24, 0.9),
                (-0.3, 0.9),
            ],
            2,
            true,
        );
        for s in [-1, 1] {
            fextrude!(
                F,
                format!("end{s}"),
                &end,
                0.06,
                (s as f64 * 1.45, 0, 0),
                M.pew_oak,
                plane = Facing::Side,
                bevel = 0.008
            );
            fextrude!(
                F,
                format!("field{s}"),
                &field,
                0.076,
                (s as f64 * 1.45, 0, 0),
                M.pew_oak,
                plane = Facing::Side,
                bevel = 0.006
            );
        }
    });
    join(&parts, "pew", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the waiting bench: 3x1, three moulded seats on a steel beam, tables between with magazines ------------------
pub fn waiting_bench() -> Vec<Obj> {
    let shell_side = smooth_outline(
        &[
            (0.23, SEAT - 0.035),
            (0.22, SEAT - 0.005),
            (0.05, SEAT),
            (-0.12, SEAT - 0.01),
            (-0.19, SEAT + 0.03),
            (-0.22, SEAT + 0.15),
            (-0.25, SEAT + 0.32),
            (-0.27, SEAT + 0.4),
            (-0.29, SEAT + 0.39),
            (-0.27, SEAT + 0.15),
            (-0.24, SEAT + 0.02),
            (-0.15, SEAT - 0.03),
            (0.05, SEAT - 0.02),
            (0.2, SEAT - 0.025),
        ],
        1,
        true,
    );
    let parts = new_parts(|| {
        fbox!(
            F,
            "beam",
            2.7,
            0.08,
            0.06,
            (0, 0.3, -0.06),
            M.steel,
            bevel = 0.006
        );
        for x in [-1.25, 1.25] {
            let xs = float_repr(x);
            fbox!(
                F,
                format!("post{xs}"),
                0.05,
                0.28,
                0.05,
                (x, 0.16, -0.06),
                M.steel,
                bevel = 0.005
            );
            fbox!(
                F,
                format!("foot{xs}"),
                0.06,
                0.03,
                0.52,
                (x, 0.015, -0.04),
                M.steel,
                bevel = 0.006
            );
            for z in [-0.28, 0.2] {
                fcyl!(
                    F,
                    format!("glide{xs}{}", float_repr(z)),
                    0.022,
                    0.012,
                    (x, 0.006, z),
                    M.radio,
                    verts = 10
                );
            }
        }
        for (i, x) in [-1.0, 0.0, 1.0].into_iter().enumerate() {
            fextrude!(
                F,
                format!("shell{i}"),
                &shell_side,
                0.5,
                (x, 0, 0),
                M.shell,
                plane = Facing::Side,
                bevel = 0.012
            );
            fbox!(
                F,
                format!("bracket{i}"),
                0.3,
                0.1,
                0.24,
                (x, 0.37, -0.04),
                M.steel,
                bevel = 0.006
            );
        }
        for (k, x) in [-0.5, 0.5].into_iter().enumerate() {
            fbox!(
                F,
                format!("table{k}"),
                0.4,
                0.025,
                0.44,
                (x, SEAT - 0.01, -0.02),
                M.laminate,
                bevel = 0.006
            );
            fbox!(
                F,
                format!("table_arm{k}"),
                0.06,
                0.12,
                0.2,
                (x, 0.38, -0.06),
                M.steel,
                bevel = 0.006
            );
            for j in 0..3 + k {
                fbox!(
                    F,
                    format!("mag{k}{j}"),
                    0.21,
                    0.006,
                    0.28,
                    (
                        x + 0.02 * (((j * 7) % 3) as i32 - 1) as f64,
                        SEAT + 0.006 + j as f64 * 0.0065,
                        -0.02
                    ),
                    [M.mag1, M.mag2, M.mag3, M.mag4][(j + 2 * k) % 4],
                    rot = (0, 0, (j as i32 * 37 % 30) - 15)
                );
            }
        }
    });
    join(&parts, "bench", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the couch: 1x2, a padded couch on a white frame, its head raised at the back, a paper sheet over it ---------
pub fn exam_couch() -> Vec<Obj> {
    let pad = 0.45; // the pad's top: the default seat's height
    let hinge = -0.3; // where the head rest rises
    let parts = new_parts(|| {
        for x in [-0.25, 0.25] {
            let xs = float_repr(x);
            for z in [-0.82, 0.82] {
                let zs = float_repr(z);
                rod(
                    &format!("leg{xs}{zs}"),
                    (x, 0.03, z),
                    (x, pad - 0.09, z),
                    0.02,
                    M.white,
                    10,
                );
                fcyl!(
                    F,
                    format!("foot{xs}{zs}"),
                    0.026,
                    0.03,
                    (x, 0.015, z),
                    M.radio,
                    verts = 10
                );
            }
            rod(
                &format!("rail{xs}"),
                (x, pad - 0.09, -0.84),
                (x, pad - 0.09, 0.84),
                0.016,
                M.white,
                10,
            );
            rod(
                &format!("low_rail{xs}"),
                (x, 0.14, -0.82),
                (x, 0.14, 0.82),
                0.012,
                M.white,
                8,
            );
        }
        for z in [-0.82, 0.82] {
            rod(
                &format!("cross{}", float_repr(z)),
                (-0.25, pad - 0.09, z),
                (0.25, pad - 0.09, z),
                0.016,
                M.white,
                10,
            );
        }
        fbox!(F, "shelf", 0.5, 0.012, 1.5, (0, 0.15, 0.0), M.white);
        fbox!(
            F,
            "board",
            0.6,
            0.02,
            1.88 - 0.62,
            (0, pad - 0.07, 0.31),
            M.white,
            bevel = 0.004
        );
        soft!(
            F,
            "pad",
            0.62,
            0.07,
            1.22,
            (0, pad - 0.035, 0.32),
            M.vinyl,
            round = 0.025,
            puff = 0.004,
            resting = true
        );
        // the head rest, raised 30 degrees from its hinge
        let a = radians(30.0);
        let l = 0.64;
        let (cz, cy) = (hinge - l / 2.0 * cos(a), pad - 0.035 + l / 2.0 * sin(a));
        soft!(
            F,
            "head",
            0.62,
            0.07,
            l,
            (0, cy, cz),
            M.vinyl,
            round = 0.025,
            puff = 0.004,
            resting = true,
            rot = (30, 0, 0)
        );
        rod(
            "strut",
            (0, pad - 0.09, hinge - 0.05),
            (0, cy - 0.06, cz - 0.05),
            0.014,
            M.white,
            8,
        );
        // the paper: a roll on brackets at the head, a sheet down the head rest and along the pad
        let (ez, ey) = (hinge - l * cos(a), pad - 0.035 + l * sin(a));
        for x in [-0.27, 0.27] {
            fbox!(
                F,
                format!("roll_bracket{}", float_repr(x)),
                0.012,
                0.08,
                0.03,
                (x, ey + 0.0, ez - 0.03),
                M.white
            );
        }
        fcyl!(
            F,
            "roll",
            0.045,
            0.5,
            (0, ey + 0.03, ez - 0.03),
            M.paper,
            rot = (0, 90, 0),
            verts = 14
        );
        fbox!(
            F,
            "sheet_head",
            0.5,
            0.003,
            l - 0.02,
            (0, cy + 0.041 / cos(a), cz + 0.041 * sin(a)),
            M.paper,
            rot = (30, 0, 0)
        );
        fbox!(
            F,
            "sheet",
            0.5,
            0.003,
            1.2,
            (0, pad + 0.0055, 0.32),
            M.paper
        );
    });
    join(&parts, "couch", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}
