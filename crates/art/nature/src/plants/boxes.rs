//! G2, the rest of the flowers: the wildflower meadow, the potted geraniums, the flower box and the raised flower
//! bed.

use artkit::botany::{_fit_with_part, MEADOW_FLOWERS, fit_to, palette, polar};
use artkit::far::{is_low, low};
use artkit::json::float_repr as fr;
use artkit::math::{PI, V3, pow, sin, v3};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{
    _put, cube, cut_leaf, florets, leaf, lump, outline, rays, rosettes, sphere, spiral, stalk,
    stem, twig, vessel,
};

/// Wildflower meadow: tussocks and long grass with flowers of every colour; one flower, at the front, is picked
/// (it shrinks away while a resident picks it, and is back after).
pub fn meadow() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#7FA04A"),
        ("leaf_dark", "#6E9240"),
        ("leaf_light", "#A2B45E"),
        ("stem", "#8EAE52"),
        ("bloom", "#EC3A2E"),
        ("bloom2", "#F6C21E"),
        ("bloom3", "#A880E0"),
        ("bloom4", "#8CB4F8"),
        ("bloom5", "#FFFFFF"),
        ("centre", "#E8B436"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(62);
    let grass = [m("leaf"), m("stem"), m("leaf_dark"), m("leaf_light")];
    // low tussocks at the grass's foot
    for (i, (x, y, _)) in spiral!(3, 0.3, &mut rng).into_iter().enumerate() {
        lump!(
            format!("tussock{i}"),
            0.12,
            (x, y, 0),
            grass[i % 4],
            scale = (1, 1, 0.4),
            seed = i as f64 * 1.1,
            segments = 8,
            rings = 5
        );
    }
    // Long grass, the most of it: tall, thin blades leaning every way, and stalks gone to seed (a pale, thin head);
    // a far version builds every third blade (drawing every random number, so the flowers stand where they do).
    for (i, (x, y, a)) in spiral!(48, 0.36, &mut rng).into_iter().enumerate() {
        let length = rng.uniform(0.28, 0.48);
        let tilt = rng.uniform(70.0, 87.0);
        let yaw = a + rng.uniform(-50.0, 50.0);
        let lean = rng.uniform(-0.06, 0.06);
        if is_low() && i % 3 != 0 {
            continue;
        }
        leaf!(
            format!("grass{i}"),
            (x * 0.85, y * 0.85, 0),
            yaw,
            tilt,
            length,
            0.022,
            grass[i % 4],
            shape = "strap",
            droop = 0.24,
            thick = 0.004,
            fold = 0.35,
            rows = 2,
            ring = 3
        );
        if i % 6 == 0 {
            let head = (x * 0.8 + lean, y * 0.8 - lean, length * 1.02);
            stem!(
                format!("seed_stalk{i}"),
                (x * 0.85, y * 0.85, 0),
                head,
                m("leaf_light"),
                r = 0.003
            );
            florets!(
                format!("seed{i}"),
                0.01,
                head,
                m("leaf_light"),
                bumps = 2.0,
                amount = 0.25,
                flat = 4.5,
                segments = low(5, 3),
                rings = low(5, 3),
                seed = i as f64
            );
        }
    }
    for (i, (x, y, _)) in spiral!(14, 0.4, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.26, 0.48);
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(0.0, 30.0));
        stalk!(
            format!("stem{i}"),
            (x, y, 0),
            (x, y, h),
            m("stem"),
            r = 0.0035
        );
        rays!(
            format!("flower{i}"),
            (x, y, h),
            0.04,
            5 + i % 3,
            m(MEADOW_FLOWERS[i % 5]),
            yaw = yaw,
            tilt = tilt,
            inner = 0.3,
            cup = 0.12,
            width = 0.85,
            centre = 0.005
        );
    }
    let (x, y, h) = (0.06, -0.22, 0.48);
    // the one picked: at the front, standing tall
    let picked = || {
        stem!("picked_stem", (x, y, 0), (x, y, h), m("stalk"), r = 0.005);
        rays!(
            "picked_flower",
            (x, y, h),
            0.06,
            12,
            m("bloom5"),
            yaw = 180,
            tilt = 30,
            inner = 0.3,
            cup = 0.08,
            width = 0.75
        );
        let eye = sphere!(
            "picked_eye",
            1.0,
            material = m("centre"),
            segments = low(6, 4),
            rings = 3,
            scale = (0.018, 0.018, 0.011)
        );
        _put!(eye, (x, y, h), yaw = 180, tilt = 30, lift = 0.005);
    };
    _fit_with_part(
        "picked",
        "meadow",
        picked,
        (x, y, 0),
        "pick",
        (0.872, 0.575, 0.91),
    )
}

/// Potted geraniums: a terracotta pot with a rolled rim, round, scalloped leaves on stalks, and round trusses of
/// red florets held up on stems over them (all `bloom`: "Flowers").
pub fn geraniums() -> Vec<Obj> {
    let p = palette(&[("body", "#C0683F"), ("bloom", "#D6283A")]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(63);
    let top = vessel!(
        &p,
        0.19,
        0.33,
        r_bottom = 0.15,
        lip = 0.014,
        segments = low(18, 10)
    );
    let round_leaf = |t: f64| pow(sin(PI * 1.0_f64.min(0.1 + t * 0.9)), 0.5);
    let cols = [m("leaf"), m("leaf_dark"), m("leaf_light")];
    // the leaves, in two rings
    for i in 0..16 {
        let yaw = i as f64 * 137.5 + rng.uniform(-10.0, 10.0);
        let up = i % 2 != 0;
        let reach = rng.uniform(0.06, 0.13);
        let end = V3::from(polar(
            reach,
            yaw,
            top + (if up { 0.1 } else { 0.05 }) + rng.uniform(0.0, 0.04),
        ));
        let t = if up {
            rng.uniform(15.0, 35.0)
        } else {
            rng.uniform(0.0, 15.0)
        };
        stalk!(
            format!("petiole{i}"),
            (0, 0, top - 0.01),
            end,
            m("stem"),
            r = 0.004,
            bow = 0.02
        );
        cut_leaf!(
            format!("leaf{i}"),
            end,
            yaw,
            t,
            0.11,
            &outline!(round_leaf, 0.12, n = 6, wave = 0.07, waves = 7),
            cols[i % 3],
            droop = 0.25,
            fold = 0.15,
            ribs = 3
        );
    }
    for (i, (x, y, _)) in spiral!(6, 0.15, &mut rng).into_iter().enumerate() {
        let h = top + rng.uniform(0.2, 0.27);
        stalk!(
            format!("stem{i}"),
            (x * 0.5, y * 0.5, top),
            (x, y, h),
            m("stem"),
            r = 0.004
        );
        florets!(
            format!("flowers{i}"),
            0.05,
            (x, y, h - 0.02),
            m("bloom"),
            bumps = 3.5,
            amount = 0.2,
            flat = 0.85,
            segments = 9,
            rings = 6,
            seed = i as f64 * 0.6
        );
    }
    fit_to(&meshes(), 0.424, 0.65, 0.468);
    settle(&[join_all("geraniums")])
}

/// Flower box: a white window box of boards on square legs, full of leafy plants and bright flowers, ivy trailing
/// over the front.
pub fn flower_box() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#ECE9E1"),
        ("bloom", "#E85A9A"),
        ("bloom2", "#F2C14E"),
        ("bloom3", "#9A5AC8"),
        ("bloom5", "#F4F0F2"),
    ]);
    let m = |name: &str| p.get(name);
    let body = m("body");
    let mut rng = Random::new(64);
    // the box: boards (two to a side, a gap between), a rim round the top, square legs
    for y in [-0.1, 0.16] {
        for k in 0..2 {
            cube!(
                format!("board{}_{k}", fr(y)),
                (0.9, 0.025, 0.104),
                at = (
                    0,
                    y + if y > 0.0 { 0.0125 } else { -0.0125 },
                    0.355 + k as f64 * 0.106
                ),
                material = body
            );
        }
    }
    for x in [-0.44, 0.44] {
        cube!(
            format!("end{}", fr(x)),
            (0.025, 0.26, 0.21),
            at = (x, 0.03, 0.408),
            material = body
        );
    }
    cube!(
        "bottom",
        (0.86, 0.24, 0.02),
        at = (0, 0.03, 0.31),
        material = body
    );
    for y in [-0.122, 0.182] {
        cube!(
            format!("rim{}", fr(y)),
            (0.94, 0.035, 0.022),
            at = (0, y, 0.522),
            material = body,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
    }
    for x in [-0.462, 0.462] {
        cube!(
            format!("rim_end{}", fr(x)),
            (0.035, 0.27, 0.022),
            at = (x, 0.03, 0.522),
            material = body,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
    }
    for x in [-0.4, 0.4] {
        for y in [-0.08, 0.14] {
            cube!(
                format!("leg{}{}", fr(x), fr(y)),
                (0.05, 0.05, 0.34),
                at = (x, y, 0.17),
                material = body,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
    }
    cube!(
        "soil",
        (0.84, 0.22, 0.02),
        at = (0, 0.03, 0.5),
        material = m("soil")
    );
    for i in 0..6 {
        let x = -0.36 + i as f64 * 0.144;
        let y = 0.03 + rng.uniform(-0.03, 0.03);
        lump!(
            format!("leaves{i}"),
            0.1,
            (x, y, 0.49),
            [m("leaf"), m("leaf_dark"), m("leaf_light")][i % 3],
            scale = (1, 1.1, 0.7),
            seed = i as f64 * 1.3,
            segments = 8,
            rings = 5
        );
    }
    let cols = [
        m("bloom"),
        m("bloom5"),
        m("bloom3"),
        m("bloom"),
        m("bloom2"),
    ];
    for i in 0..14 {
        let (x, y) = (rng.uniform(-0.4, 0.4), rng.uniform(-0.06, 0.12));
        let (h, yaw, tilt) = (
            rng.uniform(0.6, 0.66),
            rng.uniform(0.0, 360.0),
            rng.uniform(0.0, 30.0),
        );
        stalk!(
            format!("stem{i}"),
            (x, y, 0.5),
            (x, y, h),
            m("stem"),
            r = 0.003
        );
        rays!(
            format!("flower{i}"),
            (x, y, h),
            0.035,
            5,
            cols[i % 5],
            yaw = yaw,
            tilt = tilt,
            inner = 0.3,
            cup = 0.25,
            width = 1.0,
            centre = 0.005
        );
    }
    // ivy trailing over the front
    for (i, x) in [-0.3, -0.12, 0.05, 0.2, 0.32].into_iter().enumerate() {
        let top = v3(x, -0.13, 0.53);
        let pts = [
            top + v3(0, 0.04, -0.01),
            top,
            top + v3(0.02, -0.07, -0.06),
            top + v3(0.01, -0.12, -0.2),
            top + v3(0.0, -0.15, -0.3),
        ];
        twig!(
            format!("trail{i}"),
            &pts,
            0.003,
            0.002,
            m("stem"),
            verts = 3,
            res = 1
        );
        for k in 0..low(4, 2) {
            let at = pts[1].lerp(pts[4], (k as f64 + 0.5) / low(4, 2) as f64);
            cut_leaf!(
                format!("ivy{i}_{k}"),
                at,
                (180 + (k as i32 % 2 * 2 - 1) * 50) as f64,
                -10.0,
                0.05,
                &outline!(
                    |t: f64| pow(sin(PI * t), 0.6) * (1.15 - 0.6 * t),
                    0.05,
                    n = 3,
                    wave = 0.2,
                    waves = 3
                ),
                [m("leaf_light"), m("leaf")][k % 2],
                droop = 0.2,
                fold = 0.2,
                ribs = 1
            );
        }
    }
    fit_to(&meshes(), 0.945, 0.732, 0.505);
    settle(&[join_all("flowerBox")])
}

/// Raised flower bed: a box of boards, two to a side, with corner posts and a cap rail, full of soil, low rosettes
/// of leaves and flowers in six colours on short stems.
pub fn flowerbed() -> Vec<Obj> {
    let p = palette(&[
        ("wood", "#8A6446"),
        ("bloom", "#E8873A"),
        ("bloom2", "#8E6BBF"),
        ("bloom3", "#F0EDE6"),
        ("bloom4", "#D9483B"),
        ("bloom5", "#F2C14E"),
    ]);
    let m = |name: &str| p.get(name);
    let wood = m("wood");
    let mut rng = Random::new(65);
    // boards, two to a side
    for i in [-1, 1] {
        let s = i as f64;
        for k in 0..2 {
            cube!(
                format!("side{i}_{k}"),
                (0.88, 0.04, 0.105),
                at = (0, s * 0.44, 0.055 + k as f64 * 0.11),
                material = wood
            );
            cube!(
                format!("end{i}_{k}"),
                (0.04, 0.84, 0.105),
                at = (s * 0.44, 0, 0.055 + k as f64 * 0.11),
                material = wood
            );
        }
    }
    // corner posts
    for x in [-0.44, 0.44] {
        for y in [-0.44, 0.44] {
            cube!(
                format!("post{}{}", fr(x), fr(y)),
                (0.06, 0.06, 0.24),
                at = (x, y, 0.12),
                material = wood,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
    }
    // the cap rail round the top
    for i in [-1, 1] {
        let s = i as f64;
        cube!(
            format!("cap{i}"),
            (0.94, 0.06, 0.02),
            at = (0, s * 0.44, 0.235),
            material = wood
        );
        cube!(
            format!("cap_end{i}"),
            (0.06, 0.82, 0.02),
            at = (s * 0.44, 0, 0.235),
            material = wood
        );
    }
    cube!(
        "soil",
        (0.84, 0.84, 0.03),
        at = (0, 0, 0.21),
        material = m("soil")
    );
    rosettes!(
        &mut rng,
        8,
        0.3,
        0.225,
        &[m("leaf"), m("leaf_dark"), m("leaf_light")],
        leaves = 4,
        length = 0.15,
        width = 0.07,
        shape = "oval",
        tilt = (15.0, 40.0)
    );
    let cols = [
        m("bloom"),
        m("bloom2"),
        m("bloom3"),
        m("bloom4"),
        m("bloom5"),
        m("bloom3"),
    ];
    for (i, (x, y, _)) in spiral!(18, 0.34, &mut rng).into_iter().enumerate() {
        let (h, yaw, tilt) = (
            rng.uniform(0.36, 0.45),
            rng.uniform(0.0, 360.0),
            rng.uniform(0.0, 30.0),
        );
        stalk!(
            format!("stem{i}"),
            (x, y, 0.22),
            (x, y, h),
            m("stem"),
            r = 0.003
        );
        rays!(
            format!("flower{i}"),
            (x, y, h),
            0.055,
            6,
            cols[i % 6],
            yaw = yaw,
            tilt = tilt,
            inner = 0.3,
            cup = 0.2,
            width = 0.95,
            centre = 0.006
        );
    }
    fit_to(&meshes(), 0.94, 0.475, 0.94);
    settle(&[join_all("flowerbed")])
}
