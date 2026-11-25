//! The items carried and set down: the bag of shopping, the prep bowl, the stew pot, the boards, platters, trays and
//! the pie dish (dirty or not).

use super::M;
use super::common::{F, loft, rrect, t3};
use super::plates::{_cookie, _cupcake};
use artkit::forms::rounded_rect;
use artkit::kit::{self, fillet};
use artkit::materials::Mat;
use artkit::math::{TAU, V3, cos, pow, radians, sin};
use artkit::place::Facing;
use artkit::py;
use artkit::scene::Obj;
use artkit::{cylinder, fextrude, fsweep, jitter};

/// A ring of the paper bag at height `y`: its front and back flat, each side folded in by `pinch` at its
/// middle (the gussets).
pub fn _bag_ring(y: f64, hw: f64, hd: f64, pinch: f64) -> Vec<(f64, f64, f64)> {
    let pts = [
        (-hw, hd),
        (-hw / 3.0, hd),
        (hw / 3.0, hd),
        (hw, hd),
        (hw - pinch, 0.0),
        (hw, -hd),
        (hw / 3.0, -hd),
        (-hw / 3.0, -hd),
        (-hw, -hd),
        (-hw + pinch, 0.0),
    ];
    pts.iter().rev().map(|&(px, pz)| (px, y, pz)).collect()
}

/// A kraft paper bag of shopping, its sides folded in at the gussets and its top turned over in a cuff: a
/// baguette, a lettuce, a carrot with its feathery tops and a tomato with its green calyx.
pub fn ingredients() -> Vec<Obj> {
    let rings: Vec<Vec<(f64, f64, f64)>> =
        [(0.0, 0.006), (0.06, 0.016), (0.13, 0.018), (0.19, 0.014)]
            .into_iter()
            .map(|(y, p)| _bag_ring(y, 0.094, 0.064, p))
            .collect();
    let bag = loft("bag", &rings, M.kraft, 25.0);
    jitter!(bag, 0.0014, size = 0.05, seed = 2.0, up = 0.0);
    fextrude!(
        F,
        "cuff",
        &rounded_rect(0.2, 0.14, 0.004, 1),
        0.024,
        (0, 0.191, 0),
        M.kraft_fold,
        plane = Facing::Top,
        holes = vec![rounded_rect(0.192, 0.132, 0.003, 1)]
    );
    box_!("inside", 0.19, 0.002, 0.13, (0, 0.1905, 0), M.kraft_inside);
    // the baguette, leaning at the back, three slashes across its crust
    let (b0, b1) = (
        V3::from((0.052, 0.09, -0.03)),
        V3::from((0.074, 0.305, -0.044)),
    );
    let dirn = (b1 - b0).normalized();
    let pts: Vec<(f64, f64, f64)> = [0.0, 0.1, 0.5, 0.9, 1.0]
        .into_iter()
        .map(|k| t3(b0 + (b1 - b0) * k))
        .collect();
    fsweep!(
        F,
        "baguette",
        &pts,
        0.021,
        M.bun,
        verts = 8,
        scale = [0.5, 0.92, 1.0, 0.92, 0.45]
    );
    let front = (V3::from((0, 0, 1)) - dirn * dirn.z).normalized();
    let side = dirn.cross(front);
    for (i, k) in [0.55, 0.72, 0.88].into_iter().enumerate() {
        let c = b0 + (b1 - b0) * k;
        let pts: Vec<(f64, f64, f64)> =
            [-1.0, 0.0, 1.0]
                .into_iter()
                .map(|s: f64| {
                    t3(c + dirn * (0.012 * s)
                        + (front * cos(0.55 * s) + side * sin(0.55 * s)) * 0.0212)
                })
                .collect();
        let up = (-front.x as f64, -front.z as f64, front.y as f64);
        fsweep!(
            F,
            format!("slash{i}"),
            &pts,
            0.0,
            M.bun_base,
            profile = [
                (-0.0008, -0.003),
                (0.0008, -0.003),
                (0.0008, 0.003),
                (-0.0008, 0.003)
            ],
            up = up,
            scale = [0.5, 1.0, 0.5]
        );
    }
    let lettuce = lump!(
        "lettuce",
        0.054,
        (0.012, 0.15, 0.0),
        M.lettuce,
        scale = (1.0, 0.95, 0.9),
        seed = 1.7,
        subdivisions = 2,
        noise = 0.18
    );
    jitter!(lettuce, 0.004, size = 0.018, seed = 1.7);
    // the carrot, out of the bag at the front left, tapering to its root, a rounded shoulder at its top
    let (c0, c1) = ([-0.0813, 0.134, 0.02], [-0.0287, 0.296, 0.02]);
    let path: Vec<(f64, f64, f64)> = [0.0, 0.2, 0.45, 0.7, 0.9, 0.97, 1.0]
        .into_iter()
        .map(|k| {
            let p = |i: usize| c0[i] + (c1[i] - c0[i]) * k;
            (p(0), p(1), p(2))
        })
        .collect();
    let carrot = fsweep!(
        F,
        "carrot",
        &path,
        1.0,
        M.carrot,
        verts = 7,
        scale = [0.002, 0.007, 0.011, 0.0145, 0.0165, 0.0155, 0.009]
    );
    jitter!(carrot, 0.0008, size = 0.01, seed = 4.0);
    let (tx, ty) = (c1[0], c1[1]);
    for (i, (dx, dz, top)) in [
        (-0.022, 0.006, 0.334),
        (0.004, -0.012, 0.336),
        (0.02, 0.01, 0.328),
    ]
    .into_iter()
    .enumerate()
    {
        fsweep!(
            F,
            format!("carrot_top{i}"),
            &[
                (tx, ty - 0.004, 0.02),
                (tx + dx * 0.3, ty + 0.012, 0.02 + dz * 0.3),
                (tx + dx * 0.75, ty + 0.03, 0.02 + dz * 0.75),
                (tx + dx, top, 0.02 + dz)
            ],
            0.0,
            M.leaf,
            profile = [
                (-0.0012, -0.004),
                (0.0012, -0.004),
                (0.0012, 0.004),
                (-0.0012, 0.004)
            ],
            scale = [0.6, 0.8, 1.2, 0.5]
        );
    }
    // the tomato: lobed, a dimple at its top, a star of a calyx and a stalk
    lathe!(
        "tomato",
        &[
            (0.0, 0.004),
            (0.02, 0.0),
            (0.033, 0.013),
            (0.035, 0.03),
            (0.027, 0.052),
            (0.009, 0.06),
            (0.0, 0.056)
        ],
        M.tomato,
        at = (0.06, 0.192, 0.035),
        segments = 12,
        wave = (6, vec![(2, 0.04), (3, 0.05), (4, 0.05)]),
        sharp = 80
    );
    let star: Vec<(f64, f64)> = (0..10)
        .map(|i| {
            let r = if i % 2 == 0 { 0.014 } else { 0.005 };
            (
                r * cos(TAU * i as f64 / 10.0),
                r * sin(TAU * i as f64 / 10.0),
            )
        })
        .collect();
    fextrude!(
        F,
        "calyx",
        &star,
        0.0015,
        (0.06, 0.2495, 0.035),
        M.leaf,
        plane = Facing::Top,
        rot = (0, 0, 10)
    );
    disc!(
        "tomato_stalk",
        0.004,
        0.009,
        (0.06, 0.25, 0.035),
        M.leaf,
        verts = 5
    );
    super::common::done("ingredients")
}

/// A wide glazed bowl of chopped vegetables, its rim rolled: a heap rising over the rim, diced potato, carrot and
/// red pepper thick over it, green beans and peas.
pub fn prep_bowl() -> Vec<Obj> {
    // the heap's height at `r` from the middle: up from the bowl's inside wall to 3.5 cm over the rim
    let heap = |r: f64| {
        let k = 1.0 - pow(r / 0.15, 2.0);
        0.08 + 0.044 * if k > 0.0 { k } else { 0.0 }
    };
    let mut mats = vec![M.bowl; 7];
    mats.extend([M.heap; 3]);
    lathe!(
        "bowl",
        &[
            (0.0, 0.0),
            (0.095, 0.0),
            (0.12, 0.012),
            (0.1605, 0.087),
            (0.162, 0.0915),
            (0.157, 0.093),
            (0.152, 0.0905),
            (0.146, 0.08),
            (0.11, heap(0.11)),
            (0.06, heap(0.06)),
            (0.0, heap(0.0))
        ],
        mats,
        segments = 18
    );
    // (Plain diced boxes, not bevelled: two dozen of them in the 750 triangles.)
    let kinds = [
        M.carrot,
        M.potato,
        M.pepper_red,
        M.potato,
        M.carrot,
        M.potato,
    ];
    for i in 0..24 {
        let r = 0.135 * ((i as f64 + 0.5) / 24.0).sqrt();
        let a = i as f64 * 2.39996;
        let (x, z) = (r * cos(a), r * sin(a));
        box_!(
            format!("dice{i}"),
            0.02,
            0.017,
            0.02,
            (x, heap(r) + 0.005, z),
            kinds[i % 6],
            yaw = (37 * i + 5) as f64,
            tilt = (
                -z / 0.15 * 16.0 + (6 * (i as i64 % 3 - 1)) as f64,
                x / 0.15 * 16.0
            )
        );
    }
    for (i, (x, z, yaw)) in [(-0.05, 0.03, 15.0), (0.045, -0.035, 120.0)]
        .into_iter()
        .enumerate()
    {
        let a = radians(yaw);
        let (ca, sa) = (cos(a), sin(a));
        let p = [
            (x - 0.018 * ca, z + 0.018 * sa),
            (x + 0.018 * ca, z - 0.018 * sa),
        ];
        let pts: Vec<(f64, f64, f64)> = p
            .iter()
            .map(|&(u, w)| (u, heap(py::hypot(u, w)) + 0.005, w))
            .collect();
        fsweep!(F, format!("bean{i}"), &pts, 0.0045, M.pea, verts = 5);
    }
    for (i, (x, z)) in [(0.015_f64, 0.045), (-0.07, -0.03), (0.08, 0.04)]
        .into_iter()
        .enumerate()
    {
        ball!(
            format!("pea{i}"),
            0.006,
            (x, heap(py::hypot(x, z)) + 0.005, z),
            M.pea,
            seg = 6,
            rings = 3
        );
    }
    super::common::done("prepBowl")
}

/// The stew pot: dark iron, a rolled rim and a loop handle each side, riveted on; full of stew (beef, carrot
/// coins, potato and peas in gravy, parsley), or scraped empty with a tidemark and burnt bits.
fn _pot(dirty: bool) {
    let body = vec![
        (0.0, 0.0),
        (0.126, 0.0),
        (0.132, 0.006),
        (0.132, 0.141),
        (0.1348, 0.144),
        (0.134, 0.1495),
        (0.1295, 0.1495),
        (0.127, 0.145),
    ];
    if dirty {
        let mut profile = body.clone();
        profile.extend([(0.127, 0.125), (0.127, 0.113), (0.127, 0.03), (0.0, 0.034)]);
        let mut mats = vec![M.iron; 6];
        mats.extend([
            M.iron_inside,
            M.iron_inside,
            M.residue,
            M.iron_inside,
            M.residue,
        ]);
        lathe!("pot", &profile, mats, segments = 24);
        for (i, (x, z, r, sx, m)) in [
            (0.04, 0.03, 0.022, 1.2, M.burnt),
            (-0.05, -0.02, 0.016, 1.3, M.residue),
            (-0.01, 0.07, 0.012, 1.4, M.burnt),
            (0.07, -0.06, 0.01, 1.2, M.residue),
        ]
        .into_iter()
        .enumerate()
        {
            lump!(
                format!("burnt{i}"),
                r,
                (x, 0.031, z),
                m,
                scale = (sx, 0.35, 1.0),
                seed = 2.0 * i as f64 + 2.0,
                rough = 0.15
            );
        }
        blob!(
            "film",
            (0.0, 0.0335, 0.0),
            0.085,
            0.075,
            0.0012,
            M.residue,
            seed = 1.1,
            n = 9
        );
    } else {
        let mut profile = body.clone();
        profile.extend([(0.127, 0.135), (0.07, 0.1385), (0.0, 0.139)]);
        let mut mats = vec![M.iron; 6];
        mats.extend([M.iron_inside, M.iron_inside, M.stew, M.stew]);
        lathe!("pot", &profile, mats, segments = 24);
        for (i, (x, z)) in [
            (0.045, 0.03),
            (-0.05, -0.025),
            (0.0, -0.065),
            (-0.03, 0.06),
            (0.075, -0.035),
        ]
        .into_iter()
        .enumerate()
        {
            dice!(
                format!("beef{i}"),
                0.021,
                (x, 0.1405, z),
                M.beef,
                yaw = (25 * i) as f64,
                tilt = ((14 * (i as i64 % 3 - 1)) as f64, 10.0)
            );
        }
        for (i, (x, z, t)) in [
            (0.01, 0.0, 8.0),
            (-0.07, 0.01, -10.0),
            (0.06, 0.06, 12.0),
            (0.03, -0.075, -6.0),
        ]
        .into_iter()
        .enumerate()
        {
            coin!(
                format!("carrot{i}"),
                0.024,
                0.005,
                (x, 0.1395, z),
                (M.carrot, M.carrot_core),
                tilt = (t, t * 0.6)
            );
        }
        for (i, (x, z, yaw)) in [
            (-0.02, -0.04, 20.0),
            (0.08, 0.02, 60.0),
            (-0.075, -0.06, 35.0),
        ]
        .into_iter()
        .enumerate()
        {
            dice!(
                format!("potato{i}"),
                0.019,
                (x, 0.1445, z),
                M.potato,
                yaw = yaw,
                tilt = (10.0, (6 * i) as f64)
            );
        }
        for (i, (x, z)) in [
            (-0.045, 0.03),
            (0.035, 0.07),
            (0.09, -0.07),
            (-0.095, -0.02),
        ]
        .into_iter()
        .enumerate()
        {
            ball!(
                format!("pea{i}"),
                0.006,
                (x, 0.1405, z),
                M.pea,
                seg = 6,
                rings = 3
            );
        }
        for (i, (x, z, yaw)) in [(0.0, 0.035, 30.0), (0.012, 0.045, 100.0)]
            .into_iter()
            .enumerate()
        {
            leaf!(
                format!("parsley{i}"),
                (x, 0.1435, z),
                0.018,
                0.011,
                yaw,
                M.leaf,
                tilt = 8.0
            );
        }
    }
    // the handles: a loop of iron each side, its ends riveted to the wall under the rim
    for s in [-1, 1] {
        let (k, y) = (s as f64, 0.124);
        kit::tube!(
            F,
            format!("handle{s}"),
            &fillet(
                &[
                    (k * 0.129, y, -0.034),
                    (k * 0.189, y, -0.03),
                    (k * 0.189, y, 0.03),
                    (k * 0.129, y, 0.034)
                ],
                0.014,
                3
            ),
            0.0058,
            M.iron,
            verts = 6,
            round = 1
        );
        for t in [-1, 1] {
            cylinder!(
                format!("rivet{s}{t}"),
                0.0045,
                0.004,
                at = F.at(k * 0.133, y, t as f64 * 0.034),
                material = M.iron,
                rot = (0, 90, 0),
                verts = 6
            );
        }
    }
}

pub fn meal_pot() -> Vec<Obj> {
    _pot(false);
    super::common::done("mealPot")
}

pub fn pot_dirty() -> Vec<Obj> {
    _pot(true);
    super::common::done("potDirty")
}

/// A serving board 35 by 24 cm, its corners rounded, with a rounded tab handle at its right end and a hole to
/// hang it by: one outline, extruded `t` thick from height `y`.
pub fn _paddle(name: &str, material: Mat, y: f64, t: f64) -> Obj {
    let quarter = |cx: f64, cz: f64, r: f64, a0: f64| -> Vec<(f64, f64)> {
        let n = 3;
        (0..=n)
            .map(|i| {
                let a = radians(a0 + (90 * i) as f64 / n as f64);
                (cx + r * cos(a), cz + r * sin(a))
            })
            .collect()
    };
    let r = 0.015;
    let mut pts = quarter(-0.185, -0.105, r, 180.0);
    pts.extend(quarter(0.135, -0.105, r, 270.0));
    pts.extend([(0.15, -0.04), (0.16, -0.04)]);
    pts.extend(
        [-60.0, -30.0, 0.0, 30.0, 60.0]
            .into_iter()
            .map(|a| (0.16 + 0.04 * cos(radians(a)), 0.04 * sin(radians(a)))),
    );
    pts.extend([(0.16, 0.04), (0.15, 0.04)]);
    pts.extend(quarter(0.135, 0.105, r, 0.0));
    pts.extend(quarter(-0.185, 0.105, r, 90.0));
    let hole: Vec<(f64, f64)> = (0..8)
        .map(|i| {
            (
                0.173 + 0.0095 * cos(TAU * i as f64 / 8.0),
                0.0095 * sin(TAU * i as f64 / 8.0),
            )
        })
        .collect();
    fextrude!(
        F,
        name,
        &pts,
        t,
        (0, y + t / 2.0, 0),
        material,
        plane = Facing::Top,
        holes = vec![hole],
        bevel = 0.0035,
        bevel_segments = 1
    )
}

/// A serving board with a tab handle: a wedge of cheese and cubes cut from it, a fan of salami, crackers, a
/// ramekin of fig jam and a bunch of grapes on its stalk.
pub fn tasting_board() -> Vec<Obj> {
    _paddle("board", M.board_dark, 0.0, 0.028);
    prism!(
        "cheese",
        &[(-0.17, -0.07), (-0.07, -0.035), (-0.165, 0.055)],
        0.028,
        0.07,
        M.cheese_cut,
        M.cheese_cut,
        sides = vec![M.cheese_cut, M.cheese, M.cheese_cut]
    );
    for (i, (x, z, yaw)) in [(-0.075, 0.0, 20.0), (-0.095, 0.03, 55.0)]
        .into_iter()
        .enumerate()
    {
        dice!(
            format!("cube{i}"),
            0.018,
            (x, 0.0368, z),
            M.cheese_cut,
            yaw = yaw
        );
    }
    for i in 0..6 {
        let (x, z) = (-0.045 + 0.016 * i as f64, -0.06 + 0.012 * i as f64);
        coin!(
            format!("salami{i}"),
            0.048,
            0.004,
            (x, 0.028 + 0.0045 * (i % 2) as f64, z),
            M.salami,
            segments = 10,
            cup = 0.25,
            tilt = ((10 + 4 * (i % 2)) as f64, -14.0)
        );
    }
    for (i, (x, z, yaw)) in [
        (-0.125, 0.075, 10.0),
        (-0.095, 0.08, 25.0),
        (-0.065, 0.083, 40.0),
    ]
    .into_iter()
    .enumerate()
    {
        box_!(
            format!("cracker{i}"),
            0.042,
            0.005,
            0.042,
            (x, 0.031 + 0.005 * i as f64, z),
            M.cracker,
            yaw = yaw,
            bevel = 0.0012
        );
    }
    let mut mats = vec![M.dish; 5];
    mats.push(M.jam);
    lathe!(
        "ramekin",
        &[
            (0.0, 0.0),
            (0.026, 0.0),
            (0.0285, 0.004),
            (0.0285, 0.033),
            (0.026, 0.034),
            (0.024, 0.028),
            (0.0, 0.0295)
        ],
        mats,
        at = (0.03, 0.028, 0.065),
        segments = 14
    );
    for (i, p) in [
        (0.09, 0.042, -0.02),
        (0.115, 0.042, -0.035),
        (0.11, 0.042, 0.0),
        (0.135, 0.042, -0.015),
        (0.105, 0.06, -0.017),
        (0.07, 0.042, -0.045),
        (0.095, 0.042, -0.06),
        (0.125, 0.059, -0.04),
        (0.085, 0.058, -0.04),
    ]
    .into_iter()
    .enumerate()
    {
        ball!(
            format!("grape{i}"),
            0.015,
            p,
            M.grape,
            scale = (1.0, 1.0, 0.92),
            seg = 7,
            rings = 5
        );
    }
    kit::tube!(
        F,
        "stalk",
        &[
            (0.15, 0.064, -0.002),
            (0.13, 0.073, -0.022),
            (0.1, 0.073, -0.04),
            (0.075, 0.064, -0.055)
        ],
        0.0018,
        M.leaf,
        verts = 4,
        round = 2
    );
    super::common::done("tastingBoard")
}

/// An oval serving platter, 40 by 28 cm, its rim rolled.
fn _platter() {
    let mut mats = vec![M.china; 6];
    mats.push(M.china_well);
    lathe!(
        "platter",
        &[
            (0.0, 0.0),
            (0.15, 0.0),
            (0.198, 0.0162),
            (0.2, 0.0186),
            (0.196, 0.02),
            (0.188, 0.0192),
            (0.15, 0.01),
            (0.0, 0.01)
        ],
        mats,
        segments = 28,
        scale = (1.0, 0.7)
    );
}

/// A burger off the grill: a thick round patty, its edge rounded and rough, three bar marks across its top.
pub fn _patty(name: &str, at: (f64, f64, f64), d: f64, h: f64, marks: bool) {
    let (x, y, z) = at;
    let r = d / 2.0;
    let p = lathe!(
        name,
        &[
            (0.0, 0.0),
            (r * 0.94, 0.0),
            (r, h * 0.35),
            (r * 0.95, h * 0.85),
            (r * 0.7, h),
            (0.0, h * 1.02)
        ],
        M.patty,
        at = at,
        segments = 14
    );
    jitter!(p, 0.0012, size = 0.012, seed = x * 31.0, up = 0.0);
    let dxs: &[f64] = if marks { &[-0.02, 0.0, 0.02] } else { &[] };
    for (i, &dx) in dxs.iter().enumerate() {
        box_!(
            format!("{name}_mark{i}"),
            0.0065,
            0.0016,
            d * 0.8 - dx.abs() * 1.2,
            (x + dx, y + h + 0.0002, z),
            M.char,
            yaw = 20.0
        );
    }
}

/// A cob of corn lying along game x turned `yaw` degrees: `rows` rings of kernels (ridged along and round it), its
/// rounded ends; `gnawed`: eaten to the cob in the middle, kernels left at its ends.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn _corn(
    name: &str,
    r: f64,
    length: f64,
    at: (f64, f64, f64),
    yaw: f64,
    kernels: bool,
    gnawed: bool,
    rows: usize,
    segments: usize,
) -> Obj {
    let half = length / 2.0;
    let stops: Vec<f64> = (0..rows)
        .map(|k| -1.0 + (2 * k) as f64 / (rows - 1) as f64)
        .collect();
    let mut prof = vec![(0.0, -half - r * 0.2), (r * 0.7, -half)];
    for (i, &t) in stops.iter().enumerate() {
        let mut bump = if i % 2 == 0 { 1.0 } else { 0.88 };
        if gnawed {
            bump = if t.abs() > 0.55 { 1.0 } else { 0.6 };
        }
        prof.push((r * bump, t * (half - r * 0.3)));
    }
    prof.extend([(r * 0.7, half), (0.0, half + r * 0.2)]);
    let mut mats = vec![if kernels { M.corn } else { M.cob }; prof.len() - 1];
    if gnawed {
        mats = (0..prof.len() - 1)
            .map(|i| {
                let k = (i as i64 - 1).max(0).min(stops.len() as i64 - 1) as usize;
                if stops[k].abs() > 0.55 { M.corn } else { M.cob }
            })
            .collect();
    }
    let wave = if kernels {
        Some((7, (2..prof.len() - 2).map(|k| (k, 0.1)).collect()))
    } else {
        None
    };
    lathe!(
        name,
        &prof,
        mats,
        at = at,
        segments = segments,
        rot = (0.0, 90.0, -yaw),
        wave = wave
    )
}

/// The platter from the grill: two sausages, bent and marked by the bars, a marked burger, a cob of corn with
/// a pat of butter melting on it, and a sprig of parsley.
pub fn grill_platter() -> Vec<Obj> {
    _platter();
    // (Sausages 2.8 cm thick, as a butcher's are: they were 5.)
    for (i, (z, yaw, m, b)) in [
        (-0.04, 12.0, M.sausage, -0.008),
        (0.03, -8.0, M.sausage_b, 0.008),
    ]
    .into_iter()
    .enumerate()
    {
        roll!(
            format!("sausage{i}"),
            0.014,
            0.15,
            (m, M.char),
            (-0.08, 0.01 + 0.014, z),
            yaw = yaw,
            segments = 10,
            marks = vec![(-0.55, -0.4), (-0.05, 0.1), (0.45, 0.6)],
            bend = b
        );
    }
    _patty("patty", (0.075, 0.01, 0.035), 0.1, 0.028, true);
    _corn(
        "corn",
        0.022,
        0.13,
        (0.075, 0.01 + 0.022, -0.075),
        -12.0,
        true,
        false,
        11,
        14,
    );
    box_!(
        "butter",
        0.016,
        0.006,
        0.014,
        (0.07, 0.054, -0.075),
        M.butter,
        yaw = 20.0,
        bevel = 0.002
    );
    for (i, (x, z, yaw)) in [(0.145, 0.0, 20.0), (0.155, -0.02, 80.0)]
        .into_iter()
        .enumerate()
    {
        leaf!(
            format!("parsley{i}"),
            (x, 0.0105, z),
            0.022,
            0.014,
            yaw,
            M.leaf,
            tilt = 6.0
        );
    }
    super::common::done("grillPlatter")
}

/// The platter after the barbecue: a smear of grease and a streak of sauce, crumbs, a gnawed cob and a paper
/// napkin screwed up on its rim.
pub fn platter_dirty() -> Vec<Obj> {
    _platter();
    blob!(
        "smear",
        (0.02, 0.01, 0.0),
        0.12,
        0.075,
        0.0016,
        M.smear,
        seed = 0.7,
        yaw = 8.0,
        n = 10
    );
    blob!(
        "sauce",
        (-0.06, 0.0112, 0.03),
        0.045,
        0.02,
        0.0016,
        M.smear_sauce,
        seed = 2.1,
        yaw = -20.0
    );
    _corn(
        "cob",
        0.012,
        0.11,
        (0.06, 0.01 + 0.012, -0.04),
        20.0,
        true,
        true,
        11,
        14,
    );
    for (i, (x, z)) in [
        (-0.02, -0.05),
        (0.09, 0.04),
        (-0.1, -0.02),
        (0.02, 0.06),
        (-0.04, 0.07),
    ]
    .into_iter()
    .enumerate()
    {
        chunk!(
            format!("crumb{i}"),
            0.008,
            (x, 0.0135, z),
            M.crumb,
            yaw = (30 * i) as f64
        );
    }
    lump!(
        "napkin",
        0.028,
        (-0.12, 0.009, -0.05),
        M.napkin,
        scale = (1.25, 0.42, 1.0),
        seed = 3.3,
        subdivisions = 2,
        noise = 0.3,
        rough = 0.22
    );
    super::common::done("platterDirty")
}

/// A whole pizza on its round board: a puffed, golden crust, tomato showing inside it, melted cheese browned
/// in spots, pepperoni curled at their edges, basil, and the cuts between eight slices.
pub fn pizza() -> Vec<Obj> {
    lathe!(
        "board",
        &[
            (0.0, 0.0),
            (0.197, 0.0),
            (0.2, 0.004),
            (0.2, 0.016),
            (0.197, 0.02),
            (0.0, 0.02)
        ],
        M.peel,
        segments = 28
    );
    let top = lathe!(
        "pizza",
        &[
            (0.0, 0.02),
            (0.166, 0.02),
            (0.166, 0.028),
            (0.147, 0.0345),
            (0.138, 0.0375),
            (0.07, 0.0395),
            (0.0, 0.04)
        ],
        vec![
            M.dough,
            M.dough,
            M.sauce,
            M.sauce,
            M.mozzarella,
            M.mozzarella
        ],
        segments = 28
    );
    jitter!(top, 0.0008, size = 0.03, seed = 0.4, up = 0.6);
    let ring: Vec<(f64, f64)> = (0..24)
        .map(|i| {
            (
                0.156 * cos(TAU * i as f64 / 24.0),
                0.156 * sin(TAU * i as f64 / 24.0),
            )
        })
        .collect();
    let c = crust!(
        "crust",
        &ring,
        0.0405,
        0.0115,
        M.dough_baked,
        flat = 0.95,
        crimp = 5,
        bulge = 0.06,
        verts = 6,
        closed = true
    );
    jitter!(c, 0.0012, size = 0.02, seed = 1.9, up = 0.4);
    for (i, (x, z)) in [
        (0.06, 0.05),
        (-0.07, 0.03),
        (0.0, -0.085),
        (-0.04, -0.04),
        (0.08, -0.04),
        (-0.02, 0.095),
        (0.1, 0.03),
    ]
    .into_iter()
    .enumerate()
    {
        coin!(
            format!("pepperoni{i}"),
            0.042,
            0.003,
            (x, 0.039, z),
            M.pepperoni,
            segments = 10
        );
    }
    for (i, (x, z, yaw)) in [
        (0.02, 0.0, 20.0),
        (-0.09, -0.06, 70.0),
        (0.1, -0.08, 40.0),
        (0.03, -0.1, 10.0),
        (-0.06, 0.09, 60.0),
    ]
    .into_iter()
    .enumerate()
    {
        leaf!(
            format!("basil{i}"),
            (x, 0.0415, z),
            0.026,
            0.016,
            yaw,
            M.leaf,
            tilt = 5.0
        );
    }
    for (i, (x, z, s)) in [
        (-0.045, 0.065, 0.012),
        (0.045, -0.02, 0.01),
        (-0.1, -0.02, 0.009),
        (0.03, 0.09, 0.008),
    ]
    .into_iter()
    .enumerate()
    {
        blob!(
            format!("brown{i}"),
            (x, 0.0398, z),
            s,
            s * 0.7,
            0.0006,
            M.cheese_brown,
            seed = i as f64,
            yaw = (40 * i) as f64,
            n = 6
        );
    }
    for i in 0..4 {
        box_!(
            format!("cut{i}"),
            0.29,
            0.0008,
            0.0016,
            (0, 0.0405, 0),
            M.sauce,
            yaw = (45 * i) as f64
        );
    }
    super::common::done("pizza")
}

/// A steel baking tray 38 by 28 cm: its floor and sides pressed in one, the edge rolled round a wire.
fn _tray() {
    let ring = |y: f64, hw: f64, hd: f64| -> Vec<(f64, f64, f64)> {
        rrect(hw, hd, 24, 12.0)
            .into_iter()
            .map(|(x, z)| (x, y, z))
            .collect()
    };
    loft(
        "tray",
        &[
            ring(0.0, 0.18, 0.13),
            ring(0.022, 0.1865, 0.1365),
            ring(0.022, 0.1835, 0.1335),
            ring(0.006, 0.177, 0.127),
        ],
        M.tray,
        30.0,
    );
    let rim: Vec<(f64, f64, f64)> = rounded_rect(0.374, 0.274, 0.012, 3)
        .into_iter()
        .map(|(x, z)| (x, 0.022, z))
        .collect();
    fsweep!(F, "rim", &rim, 0.003, M.tray, verts = 6, closed = true);
}

/// A tray of what `treatPlate` serves, fresh from the oven: six chocolate-chip cookies and three iced cupcakes
/// in their pleated cases, each with a cherry.
pub fn baking_tray() -> Vec<Obj> {
    _tray();
    for (i, (x, z)) in [
        (-0.12, -0.085),
        (0.0, -0.085),
        (0.12, -0.085),
        (-0.12, 0.0),
        (0.0, 0.0),
        (0.12, 0.0),
        (-0.12, 0.085),
        (0.0, 0.085),
        (0.12, 0.085),
    ]
    .into_iter()
    .enumerate()
    {
        if [0, 4, 8].contains(&i) {
            _cupcake(&format!("cupcake{i}"), (x, 0.006, z));
        } else {
            _cookie(
                &format!("cookie{i}"),
                (x, 0.006, z),
                (0.0, 0.0),
                i as f64 * 1.3,
                2,
                (40 * i) as f64,
            );
        }
    }
    super::common::done("bakingTray")
}

/// The tray after the cookies and cupcakes: a greasy stain with burnt patches, cookie crumbs, smudges of melted
/// chocolate, a dab of icing and a burnt crumb or two.
pub fn tray_dirty() -> Vec<Obj> {
    _tray();
    blob!(
        "grease",
        (0.0, 0.006, 0.0),
        0.11,
        0.075,
        0.0012,
        M.grease,
        seed = 0.3,
        yaw = 5.0,
        n = 10
    );
    for (i, (x, z, d)) in [
        (-0.07, -0.025, 0.02),
        (0.06, 0.03, 0.016),
        (0.02, -0.045, 0.013),
        (-0.11, 0.07, 0.012),
    ]
    .into_iter()
    .enumerate()
    {
        blob!(
            format!("burnt{i}"),
            (x, 0.0072, z),
            d,
            d * 0.7,
            0.0012,
            M.burnt,
            seed = i as f64 * 1.3,
            yaw = (30 * i) as f64,
            n = 6
        );
    }
    for (i, (x, z)) in [
        (-0.1, 0.06),
        (0.11, -0.05),
        (0.04, 0.08),
        (-0.03, -0.09),
        (0.13, 0.09),
        (-0.14, -0.08),
    ]
    .into_iter()
    .enumerate()
    {
        chunk!(
            format!("crumb{i}"),
            0.008,
            (x, 0.0095, z),
            M.cookie,
            yaw = (25 * i) as f64
        );
    }
    for (i, (x, z, d)) in [(0.1, 0.0, 0.012), (-0.02, 0.075, 0.009)]
        .into_iter()
        .enumerate()
    {
        blob!(
            format!("chocolate{i}"),
            (x, 0.0072, z),
            d,
            d * 0.6,
            0.0012,
            M.chocolate,
            seed = i as f64 + 0.7,
            yaw = (50 * i) as f64,
            n = 6
        );
    }
    blob!(
        "icing",
        (-0.12, 0.0072, -0.02),
        0.008,
        0.006,
        0.0014,
        M.icing,
        seed = 2.2,
        n = 6
    );
    lump!(
        "bit",
        0.007,
        (0.08, 0.0065, -0.06),
        M.burnt,
        scale = (1.3, 0.6, 1.0),
        seed = 5.0
    );
    super::common::done("trayDirty")
}

/// The glazed pie dish, 26 cm across and 4.7 cm deep (a family pie's 23-28 cm), its outside fluted: `inside` the
/// profile from its rim's inner edge.
pub fn _pie_dish(inside_mats: &[Mat], inside: &[(f64, f64)]) {
    let mut profile = vec![
        (0.0, 0.0),
        (0.1045, 0.0),
        (0.1285, 0.043),
        (0.13, 0.046),
        (0.1225, 0.047),
    ];
    profile.extend_from_slice(inside);
    let mut mats = vec![M.dish, M.dish, M.dish, M.dish];
    mats.extend_from_slice(inside_mats);
    lathe!(
        "dish",
        &profile,
        mats,
        segments = 24,
        wave = (12, vec![(1, 0.012), (2, 0.01)])
    );
}

/// A cherry pie in a fluted glazed dish: a crimped rim of pastry and a lattice woven over the red filling.
pub fn pie_dish() -> Vec<Obj> {
    _pie_dish(&[M.pastry, M.filling], &[(0.1145, 0.054), (0.0, 0.0563)]);
    let ring: Vec<(f64, f64)> = (0..26)
        .map(|i| {
            (
                0.1138 * cos(TAU * i as f64 / 26.0),
                0.1138 * sin(TAU * i as f64 / 26.0),
            )
        })
        .collect();
    let c = crust!(
        "rim",
        &ring,
        0.0545,
        0.0088,
        M.pastry_baked,
        flat = 0.95,
        crimp = 13,
        bulge = 0.12,
        verts = 5,
        closed = true
    );
    jitter!(c, 0.0005, size = 0.02, seed = 0.8);
    let prof = [
        (-0.0018, -0.009),
        (0.0018, -0.009),
        (0.0018, 0.009),
        (-0.0018, 0.009),
    ];
    for (i, o) in [-0.054, 0.0, 0.054].into_iter().enumerate() {
        let h = (pow(0.113, 2.0) - pow(o, 2.0)).sqrt();
        let (lo, hi) = (0.0583, 0.061);
        for axis in ["x", "z"] {
            let ys: Vec<f64> = (0..3)
                .map(|j| {
                    if (j + i + usize::from(axis == "z")) % 2 != 0 {
                        lo
                    } else {
                        hi
                    }
                })
                .collect();
            let pts = [
                (-h, 0.0575),
                (-0.054, ys[0]),
                (0.0, ys[1]),
                (0.054, ys[2]),
                (h, 0.0575),
            ];
            let pts: Vec<(f64, f64, f64)> = pts
                .iter()
                .map(|&(u, y)| if axis == "x" { (u, y, o) } else { (o, y, u) })
                .collect();
            fsweep!(
                F,
                format!("strip_{axis}{i}"),
                &pts,
                0.0,
                M.pastry,
                profile = prof,
                up = (0, 0, 1)
            );
        }
    }
    super::common::done("pieDish")
}
