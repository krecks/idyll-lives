use super::super::common::{Bar, F, M, bar, cup, end_round, finish, jug, plank, rod, v};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{P3, cosd, sind};
use artkit::scene::Obj;
use artkit::{cylinder, fsweep};

// --- the folding bistro set: a slatted table between two folding chairs --------------------------------------

/// A folding chair centred at game depth `c`, its front towards `facing` (+1: +z): the back's frame one bent tube
/// from the front feet over the top, the back legs another under the seat's front, a tube round the seat, bowed back
/// slats and seat slats with rounded ends.
pub(super) fn bistro_chair(c: f64, facing: i32) {
    let fc = facing as f64;
    let at = |x: f64, y: f64, lz: f64| (x * fc, y, c + lz * fc);
    for (k, lz) in [-0.14, -0.045, 0.05, 0.145].into_iter().enumerate() {
        plank(
            &format!("chair{}_seat{k}", float_repr(c)),
            at(-0.205, 0.43, lz),
            at(0.205, 0.43, lz),
            0.02,
            M.bistro,
            &end_round(0.085, 0.41, Some(0.025), 2, true),
            (0, 1, 0),
            0.004,
        );
    }
    let lean = (0, cosd(10.0), -sind(10.0) * fc); // up the back, leaning 10°
    for (k, y) in [0.62, 0.74, 0.86].into_iter().enumerate() {
        let lz = 0.2 - 0.45 * y / 0.93 - 0.013; // just behind the frame's uprights
        let bow: Vec<(f64, f64, f64)> = [-0.205, -0.1, 0.0, 0.1, 0.205]
            .iter()
            .map(|&x| at(x, y, lz - 0.015 * (1.0 - artkit::math::pow(x / 0.205, 2.0))))
            .collect();
        let up = v(lean);
        artkit::bevel::eased(fsweep!(
            F,
            format!("chair{}_back{k}", float_repr(c)),
            &bow,
            0,
            M.bistro,
            profile = rounded_rect(0.07, 0.018, 0.007, 1),
            up = (up.x as f64, up.y as f64, up.z as f64),
            angle = 60
        ));
    }
    let x = 0.21;
    super::super::common::bent(
        &format!("chair{}_frame", float_repr(c)),
        &[
            at(-x, 0.0, 0.2),
            at(-x, 0.93, -0.25),
            at(x, 0.93, -0.25),
            at(x, 0.0, 0.2),
        ],
        0.012,
        M.iron,
        0.06,
        6,
        3,
        1.0,
    );
    super::super::common::bent(
        &format!("chair{}_legs", float_repr(c)),
        &[
            at(-x + 0.012, 0.0, -0.2),
            at(-x + 0.012, 0.405, 0.16),
            at(x - 0.012, 0.405, 0.16),
            at(x - 0.012, 0.0, -0.2),
        ],
        0.012,
        M.iron,
        0.03,
        6,
        3,
        1.0,
    );
    let ring: Vec<(f64, f64, f64)> = rounded_rect(0.42, 0.37, 0.03, 2)
        .iter()
        .map(|&(u, w)| at(u, 0.414, w))
        .collect();
    fsweep!(
        F,
        format!("chair{}_seat_ring", float_repr(c)),
        &ring,
        0.009,
        M.iron,
        verts = 6,
        closed = true
    );
    rod(
        &format!("chair{}_stretcher", float_repr(c)),
        at(-x, 0.12, 0.13),
        at(x, 0.12, 0.13),
        0.008,
        M.iron,
        6,
    );
    for s in [-1, 1] {
        // the pivots where the legs cross the frame, glides under
        let p = at(s as f64 * (x + 0.006), 0.2, 0.06);
        cylinder!(
            format!("chair{}_pivot{s}", float_repr(c)),
            0.014,
            0.012,
            at = F.at(p.0, p.1, p.2),
            material = M.iron,
            rot = (0, 90, 0),
            verts = 8
        );
    }
}

// The two chairs face each other across the table, their seats' fronts 4 cm under its edges (content `places`).
pub(super) const BISTRO_CHAIRS: (f64, f64) = (-0.45, 0.45);

pub fn patio_set() -> Vec<Obj> {
    for (k, z) in [-0.24, -0.12, 0.0, 0.12, 0.24].into_iter().enumerate() {
        plank(
            &format!("top{k}"),
            (-0.3, 0.71, z),
            (0.3, 0.71, z),
            0.02,
            M.bistro,
            &end_round(0.11, 0.6, Some(0.02), 2, true),
            (0, 1, 0),
            0.004,
        );
    }
    // the frame under the top, and the folding legs: a U each, crossing at the table's sides, clear of the
    // sitters' knees between them
    let ring: Vec<(f64, f64, f64)> = rounded_rect(0.56, 0.56, 0.04, 2)
        .iter()
        .map(|&(u, w)| (u, 0.69, w))
        .collect();
    bar(
        "top_frame",
        &ring,
        0.03,
        0.012,
        M.iron,
        Bar {
            bend: 0.0,
            r: Some(0.003),
            up: P3::from((0, 0, 1)),
            closed: true,
            ..Bar::default()
        },
    );
    for e in [-1, 1] {
        let ef = e as f64;
        let x = 0.27 - (if e > 0 { 0.014 } else { 0.0 });
        super::super::common::bent(
            &format!("legs{e}"),
            &[
                (-x, 0.0, ef * 0.23),
                (-x, 0.678, -ef * 0.23),
                (x, 0.678, -ef * 0.23),
                (x, 0.0, ef * 0.23),
            ],
            0.0125,
            M.iron,
            0.03,
            6,
            3,
            1.0,
        );
    }
    for s in [-1, 1] {
        cylinder!(
            format!("hinge{s}"),
            0.016,
            0.04,
            at = F.at(s as f64 * 0.263, 0.339, 0),
            material = M.iron,
            rot = (0, 90, 0),
            verts = 8
        );
    }
    cup("cup", 0.12, 0.72, 0.06, 0.038, 0.085, M.cup, true, 0.0, 12);
    jug("jug", -0.14, 0.72, -0.08, 0.05, 0.16, M.jug);
    bistro_chair(BISTRO_CHAIRS.0, 1);
    bistro_chair(BISTRO_CHAIRS.1, -1);
    finish("patioSet", false)
}
