//! L3 continued: the round and long coffee tables, the nesting tables, the round side table and the console table.

use super::common::{disc, top};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit::fillet;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, cos, pow, sin};
use artkit::parts::{join, join_all, new_parts, settle};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsweep, loft, ring, settle, slide, sweep};

/// The round coffee table (the old part): a cream top at 0.42 with a rounded edge on an oak drum, its sides reeded
/// all round, standing on a plinth.
pub fn coffee_table_round() -> Vec<Obj> {
    disc(
        "top",
        &[
            (0.38, 0.395),
            (0.382, 0.412),
            (0.39, 0.419),
            (0.4, 0.42),
            (0.41, 0.419),
            (0.418, 0.412),
            (0.42, 0.4),
            (0.42, 0.006),
        ],
        M.cream_gloss,
        0.0,
        0.0,
        56,
    );
    // the drum: its rings carry 28 reeds round it
    let (verts, reeds) = (112, 28);
    let mut secs = Vec::new();
    for (y, r) in [(0.02, 0.3), (0.2, 0.278), (0.38, 0.255)] {
        let ring: Vec<P3> = (0..verts)
            .map(|i| {
                let a = TAU * f64::from(i) / f64::from(verts);
                let k = 1.0
                    + 0.025
                        * pow(
                            cos(PI * f64::from(reeds) * f64::from(i) / f64::from(verts)).abs(),
                            0.5,
                        );
                P3 {
                    x: r * k * cos(a),
                    y: r * k * sin(a),
                    z: y,
                }
            })
            .collect();
        secs.push(ring);
    }
    loft!("drum", &secs, material = M.oak);
    disc(
        "plinth",
        &[
            (0.0, 0.33),
            (0.005, 0.35),
            (0.016, 0.35),
            (0.022, 0.335),
            (0.022, 0.3),
        ],
        M.oak,
        0.0,
        0.0,
        48,
    );
    join_all("coffeeTableRound");
    settle!(centre = false);
    Vec::new()
}

/// The long coffee table (the old part): a walnut top at 0.42 and a shelf, their edges eased, on two black steel
/// frames, each one bent square tube; a lidded box and a book on the shelf.
pub fn coffee_table_long() -> Vec<Obj> {
    top(
        "top",
        1.52,
        0.72,
        0.02,
        0.04,
        0.42,
        M.walnut_warm,
        0.0,
        0.0,
        0.01,
    );
    top(
        "shelf",
        1.36,
        0.58,
        0.015,
        0.025,
        0.1525,
        M.walnut_warm,
        0.0,
        0.0,
        0.006,
    );
    let tube = rounded_rect(0.04, 0.04, 0.008, 1);
    for x in [-0.69, 0.69] {
        // each end one closed loop of square tube: two legs, a foot rail on the floor, a rail under the top
        let lp: Vec<(f64, f64, f64)> = rounded_rect(0.6, 0.36, 0.03, 2)
            .iter()
            .map(|&(z, y)| (x, y + 0.2, z))
            .collect();
        fsweep!(
            F,
            format!("frame{}", float_repr(x)),
            &lp,
            0.0,
            M.black_metal,
            profile = &tube,
            up = (1, 0, 0),
            closed = true
        );
        fbox!(
            F,
            format!("shelf_bracket{}", float_repr(x)),
            0.04,
            0.012,
            0.58,
            (x * 0.99, 0.134, 0.0),
            M.black_metal
        );
    }
    fbox!(
        F,
        "box",
        0.32,
        0.05,
        0.24,
        (0.36, 0.1775, 0.0),
        M.box_sand,
        bevel = 0.008,
        bevel_segments = 1
    );
    fbox!(
        F,
        "box_lid",
        0.33,
        0.016,
        0.25,
        (0.36, 0.2105, 0.0),
        M.box_sand,
        bevel = 0.005,
        bevel_segments = 1
    );
    fbox!(
        F,
        "book",
        0.26,
        0.035,
        0.19,
        (0.36, 0.236, 0.0),
        M.magazine_teal,
        rot = (0, 0, -5),
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "pages",
        0.252,
        0.027,
        0.004,
        (0.36, 0.236, 0.093),
        M.pages,
        rot = (0, 0, -5)
    );
    join_all("coffeeTableLong");
    settle!(centre = false);
    Vec::new()
}

/// The nesting tables (the old part): a white table at 0.51 on black legs, a smaller oak one under it (a part of its
/// own, for a clip pulling it out when it gets a use). Each pair of legs is one bent rod with a rail under the top;
/// the tops have rounded corners and eased edges.
pub fn nesting_tables() -> Vec<Obj> {
    let big = new_parts(|| {
        top(
            "top",
            0.56,
            0.46,
            0.02,
            0.025,
            0.5125,
            M.white_gloss,
            -0.02,
            0.0,
            0.006,
        );
        for x in [-0.26, 0.26] {
            fsweep!(
                F,
                format!("legs{}", float_repr(x)),
                &fillet(
                    &[
                        (x, 0.0, -0.23),
                        (x, 0.49, -0.23),
                        (x, 0.49, 0.19),
                        (x, 0.0, 0.19)
                    ],
                    0.02,
                    3
                ),
                0.0,
                M.black_metal,
                profile = rounded_rect(0.02, 0.02, 0.005, 1),
                up = (1, 0, 0),
                angle = 60
            );
        }
    });
    join(&big, "table", Some((0, 0, 0).into()));
    let small = new_parts(|| {
        top(
            "small_top",
            0.42,
            0.36,
            0.016,
            0.025,
            0.3925,
            M.oak,
            0.12,
            0.08,
            0.006,
        );
        for x in [-0.11, 0.27] {
            fsweep!(
                F,
                format!("small_legs{}", float_repr(x)),
                &fillet(
                    &[
                        (x, 0.0, -0.04),
                        (x, 0.371, -0.04),
                        (x, 0.371, 0.28),
                        (x, 0.0, 0.28)
                    ],
                    0.016,
                    3
                ),
                0.0,
                M.black_metal,
                profile = rounded_rect(0.018, 0.018, 0.004, 1),
                up = (1, 0, 0),
                angle = 60
            );
        }
    });
    let small = join(&small, "small_table", None);
    settle!(centre = false);
    slide!(small, "small_out", (0, -0.42, 0), seconds = 0.9);
    Vec::new()
}

/// The round side table (the old part): a stone top at 0.56 with an eased edge on a brass stem, ringed under the top
/// and at its foot, standing on a domed disc.
pub fn side_table_round() -> Vec<Obj> {
    disc(
        "top",
        &[
            (0.53, 0.25),
            (0.532, 0.258),
            (0.536, 0.26),
            (0.556, 0.26),
            (0.559, 0.258),
            (0.56, 0.252),
            (0.56, 0.005),
        ],
        M.stone_top,
        0.0,
        0.0,
        48,
    );
    disc(
        "stem",
        &[
            (0.028, 0.032),
            (0.04, 0.032),
            (0.045, 0.026),
            (0.48, 0.022),
            (0.49, 0.03),
            (0.505, 0.03),
            (0.51, 0.05),
            (0.53, 0.05),
        ],
        M.brass,
        0.0,
        0.0,
        16,
    );
    disc(
        "foot",
        &[
            (0.0, 0.178),
            (0.006, 0.18),
            (0.014, 0.176),
            (0.026, 0.13),
            (0.032, 0.06),
            (0.034, 0.03),
        ],
        M.brass,
        0.0,
        0.0,
        40,
    );
    join_all("sideTableRound");
    settle!(centre = false);
    Vec::new()
}

/// A woven basket standing on `y`: a body widening a little to a rolled rim, rope handles at its ends.
#[allow(clippy::too_many_arguments)]
pub(super) fn basket(name: &str, x: f64, y: f64, z: f64, w: f64, h: f64, d: f64, mat: Mat) {
    let ring5 = |rx: f64, ry: f64, at_y: f64| {
        ring!(rx, ry, z = at_y, verts = 24, square = 5.0, at = (-x, -z))
    };
    let mut secs: Vec<Vec<P3>> = [
        (y, w * 0.45, d * 0.45),
        (y + 0.01, w * 0.48, d * 0.48),
        (y + h * 0.95, w / 2.0, d / 2.0),
    ]
    .iter()
    .map(|&(by, rx, ry)| ring5(rx, ry, by))
    .collect();
    secs.push(ring5(w / 2.0 - 0.012, d / 2.0 - 0.012, y + h * 0.95));
    secs.push(ring5(w / 2.0 - 0.012, d / 2.0 - 0.012, y + 0.02));
    loft!(name, &secs, material = mat);
    sweep!(
        format!("{name}_rim"),
        &ring5(w / 2.0 - 0.005, d / 2.0 - 0.005, y + h * 0.96),
        0.009,
        material = mat,
        verts = 6,
        closed = true
    );
    for s in [-1, 1] {
        let sf = f64::from(s);
        fsweep!(
            F,
            format!("{name}_handle{s}"),
            &[
                (x + sf * w * 0.5, y + h * 0.7, z - 0.04),
                (x + sf * (w * 0.5 + 0.018), y + h * 0.82, z),
                (x + sf * w * 0.5, y + h * 0.7, z + 0.04)
            ],
            0.006,
            mat,
            verts = 6,
            resolution = 3
        );
    }
}

/// The console table (the old part): a narrow oak top at 0.8 against the wall, its edges eased, and a low shelf on a
/// black steel frame (legs, a rail under the top each end); two woven baskets on the shelf.
pub fn console_table() -> Vec<Obj> {
    top(
        "top", 1.6, 0.44, 0.015, 0.035, 0.7975, M.oak, -0.24, 0.0, 0.008,
    );
    top(
        "shelf", 1.5, 0.36, 0.01, 0.025, 0.2325, M.oak, -0.24, 0.0, 0.005,
    );
    let tube = rounded_rect(0.04, 0.04, 0.008, 1);
    for x in [-0.76, 0.76] {
        fsweep!(
            F,
            format!("frame{}", float_repr(x)),
            &fillet(
                &[
                    (x, 0.0, -0.42),
                    (x, 0.76, -0.42),
                    (x, 0.76, -0.06),
                    (x, 0.0, -0.06)
                ],
                0.03,
                3
            ),
            0.0,
            M.black_metal,
            profile = &tube,
            up = (1, 0, 0),
            angle = 60
        );
        fbox!(
            F,
            format!("shelf_rail{}", float_repr(x)),
            0.03,
            0.03,
            0.32,
            (x * 0.995, 0.205, -0.24),
            M.black_metal,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
    for z in [-0.42, -0.06] {
        fbox!(
            F,
            format!("back_rail{}", float_repr(z)),
            1.48,
            0.03,
            0.025,
            (0, 0.705, z),
            M.black_metal,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
    for x in [-0.5, 0.0] {
        basket(
            &format!("basket{}", float_repr(x)),
            x,
            0.2325,
            -0.24,
            0.36,
            0.14,
            0.3,
            M.basket,
        );
    }
    join_all("consoleTable");
    settle(&meshes()); // on its Model's `centre`, its back on the wall face
    Vec::new()
}
