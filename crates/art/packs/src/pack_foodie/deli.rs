//! The deli pantry (1x1): a cream cupboard under a marble ledge, open shelves of deli things above.

use super::common::{_lofted, _name, _names_from, _paint, M, WALL};
use super::garden::settle_in_place;
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, cosd, sind};
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fbox, fcone, fcyl, fextrude, fsphere, fsweep, jitter, loft, revolve, ring, swing};

/// Where the pantry's count of names starts: the script names its parts from one count that runs on through a whole
/// run of the family, and the herb garden, built before it, names 95. The doors' own names come from it.
const AFTER_HERB_GARDEN: u32 = 95;

/// A preserving jar standing on `y`: a lofted body with shoulders, its label band, a brass lid.
#[allow(clippy::too_many_arguments)]
fn _jar(f: &GameFrame, x: f64, y: f64, z: f64, m: Mat, r: f64, h: f64) {
    let jar = _lofted(
        f,
        &_name("jar"),
        x,
        y,
        z,
        &[
            (0.0, r * 0.9),
            (0.006, r),
            (h * 0.8, r),
            (h * 0.92, r * 0.86),
            (h, r * 0.86),
        ],
        m,
        12,
    );
    let base = f.at(x, y, z).z;
    _paint(jar, M.chalk, |c| {
        0.3 * h < c.z as f64 - base && c.z as f64 - base < 0.6 * h
    });
    revolve!(
        _name("jar_lid"),
        &[
            (0.0, 0.0),
            (r * 0.9, 0.0),
            (r * 0.92, 0.02),
            (r * 0.8, 0.026),
            (0.0, 0.026)
        ],
        at = f.at(x, y + h, z),
        material = M.brass,
        segments = 12
    );
}

/// A loaf lying on `y`: a lumpy, domed body with three slashes across its top.
#[allow(clippy::too_many_arguments)]
pub fn _loaf(f: &GameFrame, x: f64, y: f64, z: f64, length: f64, r: f64, turn: f64, seed: f64) {
    let loaf = fsphere!(
        f,
        _name("loaf"),
        r,
        (x, y + r * 0.8, z),
        M.bread,
        scale = (length / (2.0 * r), 1.0, 0.8),
        rot = (0.0, 0.0, turn),
        segments = 12,
        rings = 7
    );
    jitter!(loaf, r * 0.06, size = r, seed = seed);
    let (c, s) = (cosd(turn), sind(turn));
    for k in [-1, 0, 1] {
        let u = k as f64 * length * 0.25;
        fsweep!(
            f,
            _name("loaf_slash"),
            &[
                (
                    x + u * c - 0.6 * r * s,
                    y + r * 1.48,
                    z - u * s - 0.6 * r * c
                ),
                (
                    x + u * c + 0.6 * r * s,
                    y + r * 1.48,
                    z - u * s + 0.6 * r * c
                )
            ],
            r * 0.12,
            M.crust,
            verts = 4
        );
    }
}

/// A cheese wheel on `y`, its rind round it, a wedge `cut` degrees wide taken out of its front.
#[allow(clippy::too_many_arguments)]
fn _wheel(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    m: Mat,
    cut: f64,
) -> Obj {
    let a0 = 90.0 + cut / 2.0;
    let mut pts = if cut != 0.0 { vec![(0.0, 0.0)] } else { vec![] };
    let n = if cut != 0.0 { 21 } else { 20 };
    pts.extend((0..n).map(|k| {
        let a = a0 + (360.0 - cut) * k as f64 / 20.0;
        (r * cosd(a), r * sind(a))
    }));
    let w = fextrude!(
        f,
        name,
        &pts,
        h,
        (x, y + h / 2.0, z),
        m,
        plane = Facing::Top,
        bevel = 0.006,
        bevel_segments = 1
    );
    if m.name != M.cheese.name {
        return _paint(w, M.cheese, |c| {
            artkit::py::hypot(c.x as f64, c.y as f64) > r * 0.92 || (c.z as f64).abs() > h * 0.45
        });
    }
    w
}

/// A salami hanging from its string at `y`: a tapered, lofted sausage tied at the top.
fn _salami(f: &GameFrame, x: f64, y: f64, z: f64, length: f64) {
    let b = f.at(x, y, z);
    let rows = [
        (0.0, 0.006),
        (0.012, 0.02),
        (0.03, 0.03),
        (length * 0.6, 0.032),
        (length - 0.02, 0.026),
        (length, 0.008),
    ];
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(hh, r)| ring!(r, r, z = b.z - hh, verts = 10, at = (b.x, b.y)))
        .collect();
    loft!(_name("salami"), &sections, material = M.salami);
    kit::tube!(
        f,
        _name("string"),
        &[(x, y, z), (x, y + 0.05, z)],
        0.0025,
        M.chalk,
        verts = 4,
        round = 1
    );
}

/// A frame-and-panel door, its bar pull upright at the edge it opens by, hinged on its outer edge.
fn _pantry_door(f: &GameFrame, x: f64, hinge_x: f64, zf: f64) -> Obj {
    let parts = new_parts(|| {
        kit::shaker!(
            f,
            _name("door"),
            0.395,
            0.78,
            (x, 0.5, zf + 0.01),
            0.02,
            M.cream,
            rail = 0.06,
            panel = M.cream_panel,
            raised = true
        );
        let px = x + if hinge_x < x { 0.15 } else { -0.15 };
        kit::pull!(
            f,
            _name("door_pull"),
            px,
            0.62,
            zf + 0.02,
            0.1,
            M.brass,
            r = 0.006,
            out = 0.024,
            upright = true
        );
        for y in [0.2, 0.8] {
            fcyl!(
                f,
                _name("hinge"),
                0.008,
                0.06,
                (
                    hinge_x + if hinge_x > 0.0 { 0.005 } else { -0.005 },
                    y,
                    zf + 0.012
                ),
                M.brass,
                verts = 8
            );
        }
    });
    hinged(
        join(&parts, &_name("door"), None),
        f.at(hinge_x, 0.5, zf + 0.01),
    )
}

pub fn deli_pantry() -> Vec<Obj> {
    _names_from(AFTER_HERB_GARDEN);
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(2);
    let zf = 0.06; // the carcass's front
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            0.8,
            0.08,
            0.4,
            (0, 0.04, WALL + 0.2),
            M.honey,
            bevel = 0.004,
            bevel_segments = 1
        );
        for x in [-0.41, 0.41] {
            fbox!(
                f,
                _name("side"),
                0.02,
                0.84,
                zf - WALL,
                (x, 0.5, (zf + WALL) / 2.0),
                M.cream,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fbox!(f, "back", 0.8, 0.84, 0.01, (0, 0.5, WALL + 0.005), M.back);
        fbox!(
            f,
            "bottom",
            0.8,
            0.02,
            zf - WALL,
            (0, 0.09, (zf + WALL) / 2.0),
            M.cream
        );
        fbox!(
            f,
            "mid_shelf",
            0.8,
            0.02,
            0.4,
            (0, 0.5, WALL + 0.2),
            M.cream
        );
        fbox!(f, "stile", 0.02, 0.84, 0.02, (0, 0.5, zf - 0.01), M.cream);
        // inside, behind the doors: jars and a crock below, tins, oil and a cheese above
        for (x, m) in [(-0.3, M.jar_a), (-0.2, M.jar_b), (-0.1, M.jar_c)] {
            _jar(f, x, 0.1, -0.2, m, 0.045, 0.18);
        }
        _lofted(
            f,
            "crock",
            0.2,
            0.1,
            -0.22,
            &[(0.0, 0.09), (0.01, 0.1), (0.17, 0.1), (0.2, 0.09)],
            M.cream_panel,
            14,
        );
        revolve!(
            "crock_lid",
            &[
                (0.0, 0.0),
                (0.1, 0.0),
                (0.1, 0.012),
                (0.04, 0.02),
                (0.03, 0.035),
                (0.0, 0.04)
            ],
            at = f.at(0.2, 0.3, -0.22),
            material = M.honey,
            segments = 14
        );
        for x in [-0.28, -0.18] {
            fcyl!(
                f,
                _name("tin"),
                0.045,
                0.12,
                (x, 0.57, -0.2),
                M.jar_c,
                verts = 12
            );
        }
        _lofted(
            f,
            "oil",
            0.0,
            0.51,
            -0.25,
            &[
                (0.0, 0.03),
                (0.005, 0.035),
                (0.2, 0.035),
                (0.25, 0.014),
                (0.31, 0.012),
            ],
            M.oil,
            10,
        );
        _wheel(f, "cheese_in", 0.22, 0.51, -0.2, 0.11, 0.08, M.cheese, 0.0);
        // the marble ledge, its front edge rounded
        fextrude!(
            f,
            "ledge",
            &rounded_rect(0.88, 0.56, 0.02, 2),
            0.04,
            (0, 0.94, WALL + 0.28),
            M.marble,
            plane = Facing::Top,
            bevel = 0.012,
            bevel_segments = 2
        );
        // the open shelves above, a crown moulding round their top
        for x in [-0.405, 0.405] {
            fbox!(
                f,
                _name("upper_side"),
                0.03,
                0.92,
                0.4,
                (x, 1.42, WALL + 0.2),
                M.cream,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "upper_back",
            0.78,
            0.92,
            0.01,
            (0, 1.42, WALL + 0.005),
            M.back
        );
        for y in [1.2, 1.5] {
            fbox!(
                f,
                _name("shelf"),
                0.78,
                0.02,
                0.36,
                (0, y, WALL + 0.19),
                M.honey,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                f,
                _name("shelf_lip"),
                0.78,
                0.03,
                0.012,
                (0, y + 0.005, WALL + 0.374),
                M.honey
            );
        }
        fbox!(
            f,
            "upper_top",
            0.84,
            0.03,
            0.42,
            (0, 1.895, WALL + 0.21),
            M.cream
        );
        // (its back 1 cm into the wall, as the old part's crown was: the model keeps its centre)
        kit::moulding!(
            f,
            "crown",
            &[
                (-0.41, WALL + 0.02),
                (0.41, WALL + 0.02),
                (0.41, WALL + 0.43),
                (-0.41, WALL + 0.43)
            ],
            &[
                (0.02, 0.0),
                (0.0, 0.0),
                (-0.008, 0.012),
                (-0.02, 0.03),
                (-0.03, 0.046),
                (-0.03, 0.06),
                (0.02, 0.06)
            ],
            M.honey,
            at = 1.93,
            plane = Facing::Top
        );
        fbox!(
            f,
            "header",
            0.78,
            0.07,
            0.025,
            (0, 1.845, -0.045),
            M.cream,
            bevel = 0.004,
            bevel_segments = 1
        );
        for (x, m) in [(-0.3, M.jar_a), (-0.19, M.jar_b), (-0.08, M.jar_c)] {
            _jar(f, x, 1.21, -0.24, m, 0.045, 0.14);
        }
        _wheel(
            f,
            "wheel_a",
            0.2,
            1.21,
            -0.25,
            0.11,
            0.08,
            M.cheese_light,
            50.0,
        );
        _wheel(f, "wheel_b", 0.21, 1.29, -0.27, 0.095, 0.07, M.cheese, 0.0);
        _loaf(f, -0.14, 1.51, -0.24, 0.3, 0.055, 0.0, 1.0);
        _loaf(f, 0.16, 1.51, -0.22, 0.24, 0.05, 25.0, 2.0);
        for (x, m) in [(-0.33, M.wine), (-0.26, M.bottle)] {
            _lofted(
                f,
                &_name("bottle"),
                x,
                1.51,
                -0.28,
                &[
                    (0.0, 0.03),
                    (0.005, 0.032),
                    (0.19, 0.032),
                    (0.23, 0.014),
                    (0.3, 0.012),
                    (0.31, 0.013),
                ],
                m,
                10,
            );
        }
        // hanging salami on a brass rail under the header
        kit::tube!(
            f,
            "rail",
            &[(-0.37, 1.79, -0.1), (0.37, 1.79, -0.1)],
            0.008,
            M.brass,
            verts = 8,
            round = 1
        );
        for x in [-0.24, -0.08, 0.1, 0.25] {
            let length = rnd.uniform(0.16, 0.22);
            _salami(f, x, 1.74, -0.1, length);
        }
        // on the ledge: a board with cheese and bread, two tomatoes
        fextrude!(
            f,
            "board",
            &rounded_rect(0.4, 0.22, 0.03, 2),
            0.02,
            (0.08, 0.97, -0.01),
            M.board,
            plane = Facing::Top,
            holes = vec![rounded_rect(0.03, 0.012, 0.006, 2)],
            bevel = 0.004,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "wedge",
            &[(0.0, 0.0), (0.08, -0.025), (0.08, 0.025)],
            0.06,
            (-0.02, 1.01, -0.01),
            M.cheese_light,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        _loaf(f, 0.17, 0.98, 0.0, 0.18, 0.04, 70.0, 3.0);
        for (x, z) in [(-0.3, 0.02), (-0.24, 0.06)] {
            fsphere!(
                f,
                _name("tomato"),
                0.028,
                (x, 0.988, z),
                M.tomato,
                scale = (1, 1, 0.9),
                segments = 10,
                rings = 6
            );
            fcone!(
                f,
                _name("tomato_calyx"),
                0.012,
                0.0,
                0.008,
                (x, 1.016, z),
                M.basil,
                verts = 5
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The two doors, hinged on their outer edges: `doors_open` swings them out.
    let left = _pantry_door(f, -0.2025, -0.40, zf);
    let right = _pantry_door(f, 0.2025, 0.40, zf);
    let objs = settle_in_place();
    swing!(
        left,
        "doors_open",
        100.0,
        seconds = 0.8,
        ease = Ease::BackOut
    );
    swing!(
        right,
        "doors_open",
        -100.0,
        seconds = 0.8,
        ease = Ease::BackOut
    );
    objs
}
