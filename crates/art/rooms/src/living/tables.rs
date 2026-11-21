//! L3, the coffee and side tables: still, but for the coffee table's drawer (`drawer_open`, a magazine in it, for
//! the magazine use) and the small nesting table. Their tops stay at the content `surface` heights (the old `TOPS`).

use super::common::{disc, knob, round_rings, top};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{TAU, cos, sin};
use artkit::parts::{join, join_all, new_parts};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fsweep, loft, settle, slide};

/// A leg turned below a square block where the aprons meet it: a collar, a long taper to a small foot.
pub(super) fn turned_leg(name: &str, x: f64, z: f64, h: f64, mat: Mat) {
    let (r, block) = (0.022, 0.06);
    fbox!(
        F,
        format!("{name}_block"),
        r * 2.1,
        block,
        r * 2.1,
        (x, h - block / 2.0, z),
        mat,
        bevel = 0.003,
        bevel_segments = 1
    );
    let y = h - block;
    fsweep!(
        F,
        name,
        &[
            (x, y, z),
            (x, y - 0.012, z),
            (x, y - 0.024, z),
            (x, y * 0.25, z),
            (x, 0.012, z),
            (x, 0.0, z)
        ],
        r,
        mat,
        verts = 10,
        scale = [0.95, 1.12, 0.92, 0.72, 0.62, 0.7]
    );
}

/// The coffee table (the old `coffee_table`, classic): an oak top at 0.42, its edges eased and corners rounded, on
/// turned legs with square blocks under aprons, a slatted shelf on stretchers; a drawer in the front apron with a
/// panelled front and a turned brass knob.
pub fn coffee_table() -> Vec<Obj> {
    let (t, h) = (0.04, 0.38);
    let still = new_parts(|| {
        top("top", 1.3, 0.74, 0.03, t, 0.42, M.oak, 0.0, 0.0, 0.008);
        fbox!(
            F,
            "apron_back",
            1.12,
            0.06,
            0.02,
            (0, h - 0.03, -0.28),
            M.oak,
            bevel = 0.005,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = f64::from(s);
            fbox!(
                F,
                format!("apron_front{s}"),
                0.3,
                0.06,
                0.02,
                (sf * 0.41, h - 0.03, 0.28),
                M.oak,
                bevel = 0.005,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("apron_side{s}"),
                0.02,
                0.06,
                0.52,
                (sf * 0.58, h - 0.03, 0.0),
                M.oak,
                bevel = 0.005,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("runner{s}"),
                0.02,
                0.06,
                0.56,
                (sf * 0.26, h - 0.03, 0.0),
                M.oak
            );
            // the stretchers the shelf's slats rest on
            fbox!(
                F,
                format!("stretcher{s}"),
                0.03,
                0.035,
                0.56,
                (sf * 0.565, 0.1, 0.0),
                M.oak,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for x in [-0.58, 0.58] {
            for z in [-0.3, 0.3] {
                turned_leg(
                    &format!("leg{}{}", float_repr(x), float_repr(z)),
                    x,
                    z,
                    h,
                    M.oak,
                );
            }
        }
        for z in [-0.2, -0.07, 0.07, 0.2] {
            fbox!(
                F,
                format!("slat{}", float_repr(z)),
                1.1,
                0.02,
                0.09,
                (0, 0.127, z),
                M.oak,
                bevel = 0.006,
                bevel_segments = 2
            );
        }
    });
    join(&still, "table", Some((0, 0, 0).into()));
    let parts = new_parts(|| {
        fbox!(
            F,
            "drawer_front",
            0.5,
            0.055,
            0.02,
            (0, h - 0.03, 0.28),
            M.oak,
            bevel = 0.004,
            bevel_segments = 1
        );
        // a raised panel on the front, its edges chamfered
        fbox!(
            F,
            "drawer_panel",
            0.42,
            0.034,
            0.008,
            (0, h - 0.03, 0.292),
            M.oak,
            bevel = 0.004,
            bevel_segments = 1
        );
        knob("knob", 0.0, h - 0.03, 0.296, M.brass);
        fbox!(
            F,
            "drawer_bottom",
            0.48,
            0.01,
            0.5,
            (0, h - 0.055, 0.02),
            M.oak
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("drawer_side{s}"),
                0.01,
                0.045,
                0.5,
                (f64::from(s) * 0.235, h - 0.03, 0.02),
                M.oak
            );
        }
        fbox!(
            F,
            "drawer_back",
            0.48,
            0.045,
            0.01,
            (0, h - 0.03, -0.225),
            M.oak
        );
        fbox!(
            F,
            "magazine",
            0.3,
            0.012,
            0.22,
            (0.05, h - 0.044, 0.08),
            M.magazine_teal,
            rot = (0, 0, 8),
            bevel = 0.002
        );
        fbox!(
            F,
            "magazine2",
            0.28,
            0.008,
            0.21,
            (-0.08, h - 0.034, 0.05),
            M.magazine_navy,
            rot = (0, 0, -6),
            bevel = 0.002
        );
    });
    let drawer = join(&parts, "drawer", None);
    settle!(centre = false);
    slide!(drawer, "drawer_open", (0, -0.3, 0), seconds = 0.8);
    Vec::new()
}

/// A pot of `r` and `h` on `y`, its rim rolled, a leafy plant in it: leaves as small swept blades fanning out.
#[allow(clippy::too_many_arguments)]
pub(super) fn plant_bowl(
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    pot: Mat,
    leaves: f64,
    seed: i128,
) {
    disc(
        "pot",
        &[
            (y, r * 0.72),
            (y + h * 0.1, r * 0.82),
            (y + h * 0.85, r),
            (y + h * 0.92, r * 1.06),
            (y + h, r * 1.04),
            (y + h, r * 0.9),
            (y + h * 0.8, r * 0.88),
        ],
        pot,
        x,
        z,
        20,
    );
    let mut rnd = Random::new(seed);
    for i in 0..9 {
        let a = TAU * f64::from(i) / 9.0 + rnd.uniform(-0.2, 0.2);
        let k = leaves * rnd.uniform(0.8, 1.15);
        let lift = rnd.uniform(0.5, 1.0);
        let (c, s) = (cos(a), sin(a));
        let base = (x + c * r * 0.2, y + h * 0.8, z + s * r * 0.2);
        fsweep!(
            F,
            format!("leaf{i}"),
            &[
                base,
                (x + c * k * 0.55, y + h + k * lift * 0.9, z + s * k * 0.55),
                (x + c * k * 1.05, y + h + k * lift * 0.6, z + s * k * 1.05)
            ],
            0.0,
            if i % 2 != 0 { M.plant } else { M.leaf_light },
            profile = [(0.0, -0.022), (0.004, 0.0), (0.0, 0.022), (-0.002, 0.0)],
            up = (0, 0, 1),
            resolution = 2,
            scale = [0.3, 1.0, 0.25]
        );
    }
}

/// The glass coffee table (the old `glass_table`): a glass top and shelf in a chrome frame of bent tube, each level
/// one closed loop with rounded corners on four posts; a plant, a vase and books on the shelf.
pub fn coffee_table_glass() -> Vec<Obj> {
    fbox!(
        F,
        "top",
        1.3,
        0.02,
        0.74,
        (0, 0.41, 0),
        M.glass,
        bevel = 0.008
    );
    fbox!(
        F,
        "shelf",
        1.2,
        0.012,
        0.64,
        (0, 0.12, 0),
        M.glass,
        bevel = 0.005
    );
    for x in [-0.6, 0.6] {
        for z in [-0.32, 0.32] {
            let at = format!("{}{}", float_repr(x), float_repr(z));
            fcyl!(
                F,
                format!("post{at}"),
                0.014,
                0.39,
                (x, 0.2, z),
                M.chrome,
                verts = 12
            );
            fcyl!(
                F,
                format!("foot{at}"),
                0.018,
                0.012,
                (x, 0.006, z),
                M.chrome,
                verts = 12,
                bevel = 0.003
            );
        }
    }
    for y in [0.105, 0.39] {
        let lp: Vec<(f64, f64, f64)> = rounded_rect(1.2, 0.64, 0.04, 3)
            .iter()
            .map(|&(u, v)| (u, y, v))
            .collect();
        fsweep!(
            F,
            format!("frame{}", float_repr(y)),
            &lp,
            0.011,
            M.chrome,
            verts = 8,
            closed = true
        );
    }
    plant_bowl(-0.3, 0.42, 0.0, 0.07, 0.08, M.pot_white, 0.1, 3);
    disc(
        "vase",
        &[
            (0.126, 0.03),
            (0.13, 0.045),
            (0.2, 0.05),
            (0.26, 0.04),
            (0.29, 0.024),
            (0.3, 0.026),
            (0.304, 0.02),
        ],
        M.vase,
        0.36,
        0.05,
        20,
    );
    for (i, (t, w, d, mat, turn)) in [
        (0.035, 0.3, 0.22, M.magazine_navy, 4),
        (0.028, 0.26, 0.2, M.magazine_teal, -7),
    ]
    .into_iter()
    .enumerate()
    {
        fbox!(
            F,
            format!("book{i}"),
            w,
            t,
            d,
            (-0.3, 0.126 + 0.035 * i as f64 + t / 2.0, 0.02),
            mat,
            rot = (0, 0, turn),
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    join_all("coffeeTableGlass");
    settle!(centre = false);
    Vec::new()
}

/// The marble side table (the old part): a round marble top with a moulded edge on a brass tulip foot that flares
/// out of a slim waist; a vase, two magazines, veins in the stone.
pub fn coffee_table_marble() -> Vec<Obj> {
    disc(
        "top",
        &[
            (0.38, 0.425),
            (0.383, 0.437),
            (0.392, 0.44),
            (0.408, 0.44),
            (0.414, 0.435),
            (0.418, 0.426),
            (0.42, 0.42),
            (0.42, 0.006),
        ],
        M.marble,
        0.0,
        0.0,
        56,
    );
    let foot: [&[f64]; 11] = [
        &[0.0, 0.25],
        &[0.008, 0.28],
        &[0.02, 0.278],
        &[0.04, 0.24],
        &[0.09, 0.15],
        &[0.16, 0.085],
        &[0.23, 0.062],
        &[0.3, 0.064],
        &[0.34, 0.085],
        &[0.37, 0.11],
        &[0.38, 0.12],
    ];
    loft!(
        "foot",
        &round_rings(&foot, 40, 2.0, (0.0, 0.0)),
        material = M.brass
    );
    let veins: [&[(f64, f64)]; 2] = [
        &[
            (-0.25, 0.2),
            (-0.12, 0.1),
            (0.0, 0.07),
            (0.12, -0.04),
            (0.2, -0.16),
        ],
        &[(0.06, 0.28), (0.12, 0.16), (0.2, 0.1), (0.3, 0.06)],
    ];
    for (i, pts) in veins.into_iter().enumerate() {
        let path: Vec<(f64, f64, f64)> = pts.iter().map(|&(px, pz)| (px, 0.4202, pz)).collect();
        fsweep!(
            F,
            format!("vein{i}"),
            &path,
            0.0,
            M.marble_vein,
            profile = [(0.0012, 0.0), (0.0, 0.004), (-0.0012, 0.0), (0.0, -0.004)],
            up = (0, 1, 0),
            resolution = 3,
            scale = [0.4, 1.0, 0.7, 1.0, 0.3][..pts.len()].to_vec()
        );
    }
    disc(
        "vase",
        &[
            (0.42, 0.032),
            (0.425, 0.045),
            (0.47, 0.052),
            (0.51, 0.038),
            (0.565, 0.022),
            (0.575, 0.025),
            (0.58, 0.02),
        ],
        M.vase,
        0.12,
        0.05,
        20,
    );
    fbox!(
        F,
        "magazine",
        0.24,
        0.012,
        0.18,
        (-0.12, 0.426, -0.08),
        M.magazine_navy,
        rot = (0, 0, -14),
        bevel = 0.002
    );
    fbox!(
        F,
        "magazine2",
        0.22,
        0.01,
        0.17,
        (-0.1, 0.437, -0.07),
        M.magazine_teal,
        rot = (0, 0, -4),
        bevel = 0.002
    );
    join_all("coffeeTableMarble");
    settle!(centre = false);
    Vec::new()
}

/// The side table (the old `side_table`, classic, which the catalog pack draws over the base one): an oak top at
/// 0.55, its edges eased, over a drawer box with a panelled front and a turned brass knob, a lower shelf, square
/// legs tapering on their inside faces.
pub fn side_table() -> Vec<Obj> {
    top("top", 0.55, 0.45, 0.02, 0.03, 0.55, M.oak, 0.0, 0.0, 0.007);
    fbox!(
        F,
        "drawer_box",
        0.46,
        0.08,
        0.36,
        (0, 0.48, 0),
        M.oak,
        bevel = 0.006,
        bevel_segments = 1
    );
    fbox!(
        F,
        "drawer_front",
        0.3,
        0.06,
        0.012,
        (0, 0.48, 0.185),
        M.oak,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "drawer_panel",
        0.24,
        0.038,
        0.006,
        (0, 0.48, 0.192),
        M.oak,
        bevel = 0.003,
        bevel_segments = 1
    );
    knob("knob", 0.0, 0.48, 0.195, M.brass);
    fbox!(
        F,
        "shelf",
        0.46,
        0.02,
        0.36,
        (0, 0.16, 0),
        M.oak,
        bevel = 0.005,
        bevel_segments = 2
    );
    for x in [-0.23, 0.23] {
        for z in [-0.18, 0.18] {
            let (sx, sz) = (1.0_f64.copysign(x), 1.0_f64.copysign(z));
            // tapering towards the inside, so the outer faces stay plumb
            fsweep!(
                F,
                format!("leg{}{}", float_repr(x), float_repr(z)),
                &[(x, 0.52, z), (x + sx * 0.005, 0.0, z + sz * 0.005)],
                0.0,
                M.oak,
                profile = rounded_rect(0.034, 0.034, 0.005, 1),
                up = (1, 0, 0),
                scale = [1.0, 0.65]
            );
        }
    }
    join_all("sideTable");
    settle!(centre = false);
    Vec::new()
}
