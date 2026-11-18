//! TT3 books and small plants.

use super::M;
use super::common::{disc, done};
use artkit::forms::{Plane, by, rounded_rect};
use artkit::kit::fillet;
use artkit::materials::Mat;
use artkit::math::{P3, atan2, cosd, degrees, pow, sind};
use artkit::parts::new_parts;
use artkit::scene::Obj;
use artkit::{
    cupped, cylinder, dome, grow, join, leaf, loft, rays, ribbed, rod, settle, soil_top, star,
    sweep, tulip_head, twig,
};

/// A closed hardback lying flat, its rounded spine to the front: the boards a little proud of the page block,
/// which shows at the head, tail and fore-edge; two gilt bands across the spine.
fn _book(
    name: &str,
    size: (f64, f64, f64),
    at: (f64, f64, f64),
    cover: Mat,
    turn: f64,
    bands: bool,
) {
    let (w, d, t) = size;
    let (x, y, z) = at;
    let (c, s) = (cosd(turn), sind(turn));
    let p = |u: f64, v: f64, h: f64| (x + u * c - v * s, y + u * s + v * c, z + h);
    for k in [-1, 1] {
        box_!(
            format!("{name}_board{k}"),
            (w, d - 0.004, 0.003),
            p(0.0, 0.002, k as f64 * (t / 2.0 - 0.0015)),
            cover,
            rot = (0.0, 0.0, turn)
        );
    }
    cylinder!(
        format!("{name}_spine"),
        t / 2.0,
        w,
        at = p(0.0, -d / 2.0 + t * 0.21, 0.0),
        material = cover,
        verts = 8,
        rot = (0.0, 90.0, turn),
        scale = (1, 0.42, 1)
    );
    box_!(
        format!("{name}_pages"),
        (w - 0.006, d - 0.008, t - 0.005),
        p(0.0, 0.0025, 0.0),
        M.pages,
        rot = (0.0, 0.0, turn)
    );
    if bands {
        for u in [-0.3, 0.3] {
            box_!(
                format!("{name}_band{u}"),
                (0.005, 0.002, t * 0.7),
                p(u * w, -d / 2.0 - 0.0002, 0.0),
                M.brass,
                rot = (0.0, 0.0, turn)
            );
        }
    }
}

/// Four hardbacks stacked a little askew, their rounded spines to the front with gilt bands, the pages showing
/// between the boards, a ribbon marker hanging from the top one.
pub fn book_stack() -> Vec<Obj> {
    let mut z = 0.0;
    for (i, (w, d, t, cover, turn, dx)) in [
        (0.24, 0.17, 0.035, M.book_a, 0.0, 0.0),
        (0.22, 0.16, 0.03, M.book_b, 10.0, 0.008),
        (0.2, 0.15, 0.04, M.book_c, -7.0, -0.008),
        (0.18, 0.13, 0.028, M.book_d, 18.0, 0.004),
    ]
    .into_iter()
    .enumerate()
    {
        _book(
            &format!("book{i}"),
            (w, d, t),
            (dx, 0.0, z + t / 2.0),
            cover,
            turn,
            true,
        );
        z += t;
    }
    sweep!(
        "ribbon",
        &[
            (0.03, 0.06, z - 0.0005),
            (0.034, 0.069, z - 0.003),
            (0.035, 0.07, z - 0.022)
        ],
        0.0,
        profile = [
            (-0.0004, -0.003),
            (0.0004, -0.003),
            (0.0004, 0.003),
            (-0.0004, 0.003)
        ],
        material = M.red,
        up = (0, 1, 0)
    );
    done("bookStack", false)
}

/// Eight books standing between brass bookends (each an arched plate with a round cut-out on a foot): the boards
/// proud of the page block at the top, rounded spines to the front, paper bands on some.
pub fn books_upright() -> Vec<Obj> {
    for s in [-1, 1] {
        let plate: Vec<(f64, f64)> = rounded_rect(0.12, 0.16, 0.035, 2)
            .into_iter()
            .map(|(u, v)| (u, v + 0.08))
            .collect();
        let hole: Vec<(f64, f64)> = rounded_rect(0.036, 0.036, 0.018, 2)
            .into_iter()
            .map(|(u, v)| (u, v + 0.118))
            .collect();
        slab!(
            format!("bookend{s}"),
            &plate,
            0.008,
            (s as f64 * 0.156, 0.0, 0.0),
            M.brass,
            plane = Plane::YZ,
            holes = vec![hole]
        );
        box_!(
            format!("bookend_foot{s}"),
            (0.06, 0.12, 0.006),
            (s as f64 * 0.13, 0.0, 0.003),
            M.brass
        );
    }
    let covers = [
        M.book_a, M.book_c, M.book_b, M.book_e, M.book_d, M.book_c, M.book_a, M.book_b, M.book_e,
    ];
    let (mut x, mut k) = (-0.146, 6);
    for (i, cover) in covers.into_iter().enumerate() {
        k = (k * 7 + 3) % 11;
        let w = 0.022 + (k % 4) as f64 * 0.007;
        let h = 0.2 * (0.72 + (k % 5) as f64 * 0.07);
        if x + w > 0.146 {
            break;
        }
        let cx = x + w / 2.0;
        box_!(
            format!("book{i}_pages"),
            (w - 0.003, 0.12, h - 0.004),
            (cx, 0.0025, 0.006 + (h - 0.004) / 2.0),
            M.pages
        );
        for e in [-1, 1] {
            box_!(
                format!("book{i}_board{e}"),
                (0.0016, 0.124, h),
                (cx + e as f64 * (w / 2.0 - 0.0008), 0.0005, 0.006 + h / 2.0),
                cover
            );
        }
        cylinder!(
            format!("book{i}_spine"),
            w / 2.0,
            h,
            at = (cx, -0.0625 + w * 0.2, 0.006 + h / 2.0),
            material = cover,
            verts = 6,
            scale = (1, 0.4, 1)
        );
        if i % 3 != 1 {
            box_!(
                format!("book{i}_band"),
                (w * 0.8, 0.002, 0.008),
                (cx, -0.0626, 0.006 + h * 0.8),
                M.paper
            );
        }
        x += w + 0.002;
    }
    done("booksUpright", false)
}

/// The yaw (the botany's: degrees round from the back) that faces out from the middle towards (x, y).
fn _facing(x: f64, y: f64) -> f64 {
    if x != 0.0 || y != 0.0 {
        90.0 - degrees(atan2(y, x))
    } else {
        0.0
    }
}

// The bunch of a vase (x, depth, height, petal colour, kind): the same heads in the vase of flowers and in the empty
// vase once flowers are put in it, so a bunch cut in the garden looks like one bought arranged (G-10, Q-18).
const VASE_HEADS: [(f64, f64, f64, &str, &str); 10] = [
    (0.0, 0.0, 0.392, "white", "daisy"),
    (-0.066, 0.03, 0.345, "blush", "tulip"),
    (0.064, -0.02, 0.355, "yellow", "daisy"),
    (0.03, 0.06, 0.37, "rose", "tulip"),
    (-0.042, -0.05, 0.335, "white", "daisy"),
    (0.05, 0.042, 0.325, "blush", "cosmos"),
    (-0.022, 0.05, 0.372, "white", "small"),
    (0.022, -0.052, 0.352, "rose", "tulip"),
    (-0.068, -0.008, 0.31, "yellow", "small"),
    (0.066, 0.01, 0.3, "white", "small"),
];
const VASE_BODY: [(f64, f64); 9] = [
    (0.04, 0.0),
    (0.044, 0.004),
    (0.06, 0.05),
    (0.065, 0.11),
    (0.052, 0.18),
    (0.032, 0.215),
    (0.036, 0.24),
    (0.03, 0.24),
    (0.027, 0.225),
];

/// The vase's bunch: `VASE_HEADS` on curving stems from the water at `water`, leaves at the neck.
fn _bunch(water: f64, neck: f64) {
    for (i, &(x, y, z, colour, kind)) in VASE_HEADS.iter().enumerate() {
        let petal = M.palette().get(colour);
        let (yaw, tilt) = (_facing(x, y), if x != 0.0 || y != 0.0 { 25.0 } else { 0.0 });
        twig!(
            format!("stem{i}"),
            &[
                (x * 0.15, y * 0.15, water),
                (x * 0.55, y * 0.55, (neck + z) / 2.0),
                (x, y, z - 0.004)
            ],
            0.0032,
            0.0026,
            M.leaf,
            verts = 4,
            res = 1
        );
        if kind == "tulip" {
            tulip_head!(
                format!("flower{i}"),
                (x, y, z - 0.012),
                0.017,
                0.042,
                petal,
                yaw = yaw,
                tilt = tilt * 0.6
            );
        } else {
            let (n, r, width) = match kind {
                "daisy" => (12, 0.03, 0.7),
                "small" => (10, 0.021, 0.7),
                "cosmos" => (8, 0.028, 0.95),
                other => panic!("no flower {other}"),
            };
            rays!(
                format!("flower{i}"),
                (x, y, z),
                r,
                n,
                petal,
                yaw = yaw,
                tilt = tilt,
                inner = 0.25,
                cup = 0.12,
                width = width,
                thick = 0.004
            );
            dome!(
                format!("eye{i}"),
                (x, y, z),
                r * 0.3,
                0.006,
                if colour == "yellow" {
                    M.orange
                } else {
                    M.yellow
                },
                yaw = yaw,
                tilt = tilt
            );
        }
        if i % 2 != 0 {
            // a leaf part way up the stem
            let (mx, my) = (x * 0.55, y * 0.55);
            leaf!(
                format!("stem_leaf{i}"),
                (mx, my, (neck + z) / 2.0 - 0.01),
                _facing(x, y) + 40.0,
                35.0,
                0.05,
                0.018,
                M.leaf_light,
                shape = "lance",
                droop = 0.3,
                thick = 0.003,
                rows = 3,
                ring = 4
            );
        }
    }
    for (i, a) in [20.0, 140.0, 260.0].into_iter().enumerate() {
        let (bx, by) = (0.022 * cosd(a), 0.022 * sind(a));
        leaf!(
            format!("leaf{i}"),
            (bx, by, neck),
            _facing(bx, by),
            40.0,
            0.075,
            0.03,
            M.leaf,
            shape = "lance",
            droop = 0.3,
            thick = 0.004,
            rows = 4,
            ring = 4
        );
    }
}

/// A glazed vase swelling at its belly with a flared lip, a mixed bunch on curving stems: white and yellow daisies
/// (ray petals round a raised eye), two tulips and a pink cosmos, leaves at the neck.
pub fn vase_flowers() -> Vec<Obj> {
    turned!("vase", &VASE_BODY, M.vase, verts = 16);
    _bunch(0.2, 0.232);
    done("vaseFlowers", false)
}

/// The same glazed vase, empty: a mouth 6 cm across and no flowers. The bunch it takes is a part of its own,
/// built small in the water and hidden inside the vase, which `flowers_open` grows up out of the mouth over 0.9 s
/// (the game holds it there while the vase is open: `motions.open`), so the flowers come from the garden and not
/// from the shop. The bunch is the vase of flowers' own (`VASE_HEADS`).
pub fn vase_empty() -> Vec<Obj> {
    let body = new_parts(|| {
        turned!("vase", &VASE_BODY, M.vase, verts = 16);
    });
    join!(&body, "vase", origin = (0, 0, 0));
    let part = new_parts(|| _bunch(0.2, 0.232));
    let flowers = join!(&part, "flowers", origin = (0, 0, 0.2));
    flowers.set_scale((0.001, 0.001, 0.001));
    let objs = settle!(centre = false);
    for axis in [
        artkit::math::Axis::X,
        artkit::math::Axis::Y,
        artkit::math::Axis::Z,
    ] {
        grow!(flowers, "flowers_open", seconds = 0.9, axis = axis);
    }
    objs
}

/// A glazed bud vase with a slim neck and a flared lip, one rose on a curving stem: a cup of outer petals round a
/// furled bud, green sepals under it, a leaf on the stem.
pub fn vase_small() -> Vec<Obj> {
    turned!(
        "vase",
        &[
            (0.026, 0.0),
            (0.03, 0.004),
            (0.039, 0.035),
            (0.04, 0.06),
            (0.025, 0.12),
            (0.013, 0.148),
            (0.016, 0.16),
            (0.011, 0.16),
            (0.009, 0.15)
        ],
        M.bud_vase,
        verts = 16
    );
    twig!(
        "stem",
        &[(0.0, 0.0, 0.14), (0.004, 0.002, 0.2), (0.012, 0.004, 0.25)],
        0.003,
        0.0026,
        M.leaf,
        verts = 4,
        res = 1
    );
    leaf!(
        "leaf",
        (0.004, 0.002, 0.205),
        250.0,
        35.0,
        0.04,
        0.022,
        M.leaf,
        shape = "oval",
        droop = 0.25,
        thick = 0.003,
        rows = 3,
        ring = 4
    );
    star!(
        "sepals",
        (0.012, 0.004, 0.249),
        0.013,
        5,
        M.leaf,
        inner = 0.3,
        depth = 0.002,
        cup = -0.3
    );
    cupped!(
        "rose",
        &[
            (0.0, 0.0),
            (0.011, 0.004),
            (0.017, 0.013),
            (0.0165, 0.022),
            (0.014, 0.0215)
        ],
        (0.012, 0.004, 0.25),
        M.white,
        segments = 8
    );
    cupped!(
        "rose_inner",
        &[
            (0.0, 0.004),
            (0.009, 0.008),
            (0.0125, 0.018),
            (0.011, 0.025),
            (0.009, 0.0245)
        ],
        (0.012, 0.004, 0.25),
        M.white,
        yaw = 22.0,
        segments = 7
    );
    tulip_head!("rose_bud", (0.012, 0.004, 0.253), 0.0105, 0.024, M.white);
    done("vaseSmall", false)
}

/// An echeveria in a pale pot with a rim, on grit: three tiers of fat, upturned leaves round a tight heart.
pub fn pot_succulent() -> Vec<Obj> {
    turned!(
        "pot",
        &[
            (0.054, 0.0),
            (0.058, 0.004),
            (0.066, 0.056),
            (0.07, 0.06),
            (0.07, 0.07),
            (0.064, 0.07),
            (0.062, 0.058)
        ],
        M.pot_pale,
        verts = 16
    );
    soil_top!(
        None,
        0.062,
        0.066,
        segments = 12,
        mound = 0.006,
        seed = 1.0,
        material = M.sand
    );
    for (tier, (n, base, length, width, tilt, z, colour, rows)) in [
        (8, 0.014, 0.072, 0.044, 20.0, 0.074, M.succulent, 3),
        (6, 0.009, 0.054, 0.036, 44.0, 0.084, M.succulent, 2),
        (5, 0.004, 0.044, 0.028, 72.0, 0.096, M.succulent_light, 2),
    ]
    .into_iter()
    .enumerate()
    {
        for i in 0..n {
            let a = (i * 360) as f64 / n as f64 + (tier * 22) as f64;
            let (bx, by) = (base * cosd(a), base * sind(a));
            leaf!(
                format!("leaf{tier}_{i}"),
                (bx, by, z),
                90.0 - a,
                tilt,
                length,
                width,
                colour,
                shape = "oval",
                thick = 0.016,
                fold = 0.18,
                rows = rows,
                ring = 4,
                droop = -0.12
            );
        }
    }
    dome!(
        "heart",
        (0, 0, 0.098),
        0.012,
        0.018,
        M.succulent_light,
        lift = 0.0
    );
    done("potSucculent", false)
}

/// A ribbed column cactus in a terracotta pot with a rolled rim, on sand: two ribbed arms bent up from its sides,
/// a pink flower on top.
pub fn pot_cactus() -> Vec<Obj> {
    turned!(
        "pot",
        &[
            (0.048, 0.0),
            (0.052, 0.005),
            (0.06, 0.075),
            (0.066, 0.078),
            (0.067, 0.1),
            (0.058, 0.1),
            (0.056, 0.092)
        ],
        M.terracotta,
        verts = 14
    );
    disc("sand", 0.0565, (0.0, 0.0, 0.094), M.sand, 14, None);
    let secs: Vec<Vec<P3>> = [
        (0.09, 0.034),
        (0.2, 0.036),
        (0.27, 0.031),
        (0.298, 0.02),
        (0.31, 0.006),
    ]
    .into_iter()
    .map(|(z, r)| {
        ribbed!(r, 7, depth = 0.16, verts = 21)
            .into_iter()
            .map(|(u, v)| P3::from((u, v, z)))
            .collect()
    })
    .collect();
    loft!("cactus", &secs, material = M.cactus);
    for (s, (h, z)) in [(1.0, (0.06, 0.16)), (-1.0, (0.05, 0.195))] {
        let pts = fillet(
            &[
                (s * 0.02, 0.0, z),
                (s * 0.077, 0.0, z + 0.004),
                (s * 0.081, 0.0, z + h),
            ],
            0.022,
            2,
        );
        sweep!(
            format!("arm{}", s as i32),
            &pts,
            0.0,
            profile = ribbed!(0.016, 5, depth = 0.2),
            material = M.cactus,
            angle = 60,
            scale = by(|w| {
                let f = (w - 0.82) / 0.18;
                1.0 - 0.55 * pow(if f > 0.0 { f } else { 0.0 }, 1.5)
            })
        );
    }
    rays!(
        "flower",
        (0, 0, 0.308),
        0.022,
        10,
        M.pink,
        inner = 0.3,
        cup = 0.45,
        width = 0.9,
        centre = 0.003
    );
    dome!("flower_eye", (0, 0, 0.308), 0.006, 0.007, M.yellow);
    done("potCactus", false)
}

/// A peace lily in a glossy pot with a rolled rim: arching, pointed leaves on their stalks and two white spathes
/// on tall stems.
pub fn pot_small() -> Vec<Obj> {
    turned!(
        "pot",
        &[
            (0.05, 0.0),
            (0.054, 0.004),
            (0.062, 0.09),
            (0.068, 0.096),
            (0.068, 0.11),
            (0.06, 0.11),
            (0.058, 0.1)
        ],
        M.pot_white,
        verts = 16
    );
    soil_top!(
        None,
        0.058,
        0.106,
        segments = 12,
        mound = 0.008,
        seed = 2.0,
        material = M.soil
    );
    for i in 0..7 {
        let a = (i * 360) as f64 / 7.0 + 10.0;
        let (r0, lift) = (0.012, 0.03 + (i % 3) as f64 * 0.012);
        let (bx, by) = (r0 * cosd(a), r0 * sind(a));
        let (tx, ty) = ((r0 + 0.012) * cosd(a), (r0 + 0.012) * sind(a));
        twig!(
            format!("stalk{i}"),
            &[(bx * 0.5, by * 0.5, 0.104), (tx, ty, 0.104 + lift)],
            0.0028,
            0.0022,
            M.leaf,
            verts = 3,
            res = 1
        );
        let reach = 1.0 + 0.19 * cosd(2.0 * a); // longer to the sides than to the front and back (its footprint)
        leaf!(
            format!("leaf{i}"),
            (tx, ty, 0.104 + lift),
            90.0 - a,
            (70 - (i % 3) * 7) as f64,
            (0.088 + (i % 2) as f64 * 0.012) * reach,
            0.04,
            if i % 2 != 0 { M.leaf } else { M.leaf_light },
            shape = "lance",
            droop = 0.22,
            thick = 0.004,
            fold = 0.2,
            rows = 4,
            ring = 4
        );
    }
    for (i, (x, y, top, yaw)) in [(0.012, -0.01, 0.261, 200.0), (-0.014, 0.012, 0.235, 30.0)]
        .into_iter()
        .enumerate()
    {
        twig!(
            format!("spathe_stem{i}"),
            &[
                (x * 0.3, y * 0.3, 0.104),
                (x, y, top - 0.06),
                (x, y, top - 0.052)
            ],
            0.0026,
            0.0022,
            M.leaf,
            verts = 3,
            res = 1
        );
        leaf!(
            format!("spathe{i}"),
            (x, y, top - 0.055),
            yaw,
            78.0,
            0.055,
            0.032,
            M.white,
            shape = "oval",
            thick = 0.003,
            fold = 0.7,
            rows = 3,
            ring = 4,
            droop = -0.05
        );
        rod!(
            format!("spadix{i}"),
            (x, y, top - 0.054),
            (x, y, top - 0.026),
            0.0035,
            material = M.crumb,
            verts = 5
        );
    }
    done("potSmall", false)
}
