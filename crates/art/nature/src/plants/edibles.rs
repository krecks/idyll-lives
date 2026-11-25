//! G4, the edibles: the vegetable patch, the tomatoes, the strawberries, the herb planter and the pumpkins.

use artkit::botany::{_fit_with_part, noise_at, palette, polar, raised_bed};
use artkit::far::{is_low, low};
use artkit::json::float_repr as fr;
use artkit::math::{P3, TAU, V3, cos, radians, sin, v3};
use artkit::parts::join_all;
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{
    _crop, cone, cube, cupped, cut_leaf, cylinder, florets, lobed, loft, lump, mound, outline,
    pinnate, rays, revolve, ribbed, rose_leaf, spiral, stalk, stem, sweep, twig,
};

/// Vegetable patch: a raised bed of ruffled lettuces, cabbages wrapped in broad outer leaves and feathery carrot
/// tops, grown from seedlings to ripe.
pub fn veg_patch() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#8A6446"),
        ("leaf", "#9CC45A"),
        ("leaf_light", "#8AB44E"),
        ("leaf_dark", "#7EA48A"),
        ("stem", "#5E9A3A"),
        ("bloom", "#A8C8A0"),
        ("bloom4", "#E8782A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(81);
    let top = raised_bed(&p, 0.92, 0.92, 0.18);
    // the sawn boards' arrises eased (KIT: botany's `raised_bed` leaves them sharp)
    crate::chamfer(&["board"]);
    let still = join_all("vegPatch");
    let mut spots = Vec::new();
    for x in [-0.28, 0.0, 0.28] {
        let mut row = Vec::new();
        for y in [0.27, 0.0, -0.27] {
            row.push((x + rng.uniform(-0.03, 0.03), y + rng.uniform(-0.03, 0.03)));
        }
        spots.push(row);
    }
    let mut tops = Vec::new();
    for _ in 0..3 {
        let mut row = Vec::new();
        for j in 0..3 {
            row.push((
                rng.uniform(0.22, 0.28),
                (j * 120) as f64 + rng.uniform(0.0, 40.0),
            ));
        }
        tops.push(row);
    }
    let ruffled = outline!("oval", 0.12, n = 4, wave = 0.18, waves = 5);
    let leaves = || {
        // lettuces: rosettes of ruffled leaves, cupped up
        for (k, &(x, y)) in spots[0].iter().enumerate() {
            lump!(
                format!("lettuce_heart{k}"),
                0.045,
                (x, y, top - 0.01),
                m("leaf_light"),
                scale = (1, 1, 1.1),
                noise = 0.3,
                seed = k as f64,
                segments = 6,
                rings = 4
            );
            for j in 0..6 {
                let inner = j >= 4;
                cut_leaf!(
                    format!("lettuce{k}_{j}"),
                    (x, y, top + if inner { 0.03 } else { 0.01 }),
                    (j * 360) as f64 / (if inner { 2.0 } else { 4.0 })
                        + (k * 25) as f64
                        + if inner { 40.0 } else { 0.0 },
                    if inner { 60.0 } else { 30.0 },
                    if inner { 0.11 } else { 0.13 },
                    &ruffled,
                    [m("leaf"), m("leaf_light")][(j + k) % 2],
                    droop = 0.35,
                    fold = 0.4,
                    ribs = 2
                );
            }
        }
        // cabbages' outer leaves
        for (k, &(x, y)) in spots[1].iter().enumerate() {
            for j in 0..5 {
                cut_leaf!(
                    format!("cabbage{k}_{j}"),
                    (x, y, top + 0.01),
                    (j * 72 + k * 20) as f64,
                    38.0,
                    0.15,
                    &outline!("oval", 0.14, n = 4, wave = 0.08, waves = 3),
                    m("leaf_dark"),
                    droop = 0.4,
                    fold = 0.45,
                    ribs = 2,
                    thick = 0.005
                );
            }
        }
        // carrot tops: feathery fronds
        for (k, &(x, y)) in spots[2].iter().enumerate() {
            for (j, &(length, yaw)) in tops[k].iter().enumerate() {
                cut_leaf!(
                    format!("carrot_top{k}_{j}"),
                    (x, y, top),
                    yaw,
                    (70 - j as i32 * 8) as f64,
                    length,
                    &pinnate!(3, 0.08, "lance", lean = 0.7, fill = 0.35),
                    m("stem"),
                    droop = 0.3,
                    fold = 0.1,
                    ribs = 2
                );
            }
        }
    };
    let crop = || {
        // cabbage hearts
        for (k, &(x, y)) in spots[1].iter().enumerate() {
            lump!(
                format!("heart{k}"),
                0.07,
                (x, y, top + 0.04),
                m("bloom"),
                scale = (1, 1, 0.85),
                noise = 0.1,
                seed = (k + 6) as f64,
                segments = low(9, 6),
                rings = low(6, 4)
            );
        }
        // carrot shoulders
        for (k, &(x, y)) in spots[2].iter().enumerate() {
            cone!(
                format!("carrot{k}"),
                0.022,
                0.016,
                0.03,
                at = (x, y, top + 0.01),
                material = m("bloom4"),
                verts = low(8, 5)
            );
        }
    };
    // picked, the bed is bare
    _crop!(
        still,
        (0, 0, top),
        leaves,
        crop,
        harvested = true,
        size = (0.92, 0.398, 0.92)
    )
}

/// Tomato plants: three plants tied to canes on a soil mound, a stem climbing each cane with leaves of toothed
/// leaflets off it, hung with trusses of red, orange and green tomatoes, each with its green star of a calyx.
pub fn tomatoes() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#4A7A36"),
        ("leaf_dark", "#5A8A3C"),
        ("leaf_light", "#6A9A44"),
        ("stem", "#7AA84C"),
        ("flesh", "#C9A86A"),
        ("bloom", "#D63A2A"),
        ("bloom2", "#E04A30"),
        ("bloom3", "#C8302A"),
        ("bloom4", "#E8902E"),
        ("flesh_light", "#8DB04A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(82);
    let top = mound!(&p, r = 0.4);
    let canes = [(0.2, 0.15), (-0.2, 0.12), (0.0, -0.2)];
    for (i, (x, y)) in canes.into_iter().enumerate() {
        cylinder!(
            format!("cane{i}"),
            0.011,
            1.18,
            at = (x, y, 0.6),
            material = m("flesh"),
            verts = low(6, 4)
        );
        // ties
        for k in 0..low(3, 0) {
            cylinder!(
                format!("tie{i}_{k}"),
                0.016,
                0.012,
                at = (x, y, 0.35 + k as f64 * 0.28),
                material = m("flesh"),
                verts = 6
            );
        }
    }
    let still = join_all("tomatoes");
    let mut clumps_at = Vec::new();
    for (x, y) in canes {
        let mut cane = Vec::new();
        for k in 0..4 {
            cane.push((
                x + rng.uniform(-0.06, 0.06),
                y + rng.uniform(-0.06, 0.06),
                0.14 + k as f64 * 0.24 + rng.uniform(-0.04, 0.04),
            ));
        }
        clumps_at.push(cane);
    }
    let mut fruit_at = Vec::new();
    for _ in 0..15 {
        let (x, y) = rng.choice(&canes);
        let a = rng.uniform(0.0, TAU);
        fruit_at.push((x + 0.13 * cos(a), y + 0.13 * sin(a), rng.uniform(0.2, 0.92)));
    }
    let cols = [m("leaf"), m("leaf_dark"), m("leaf_light"), m("stem")];
    let leaves = || {
        let mut r2 = Random::new(182);
        for (i, cane) in clumps_at.iter().enumerate() {
            let (x, y) = canes[i];
            twig!(
                format!("stem{i}"),
                &[
                    (x + 0.02, y, top - 0.01),
                    (x + 0.025, y + 0.01, 0.4),
                    (x + 0.02, y - 0.01, 0.8),
                    (x + 0.015, y, 1.08)
                ],
                0.012,
                0.006,
                m("stem"),
                verts = 4
            );
            for (k, &at) in cane.iter().enumerate() {
                lump!(
                    format!("leaves{i}_{k}"),
                    0.1,
                    at,
                    cols[(k + 1) % 4],
                    scale = (1, 1, 0.75),
                    noise = 0.35,
                    seed = (i * 4 + k) as f64,
                    segments = low(7, 6),
                    rings = low(5, 3)
                );
                // leaves of leaflets standing out of the clump
                for j in 0..2 {
                    let (yaw, t) = (r2.uniform(0.0, 360.0), r2.uniform(-5.0, 25.0));
                    let base = V3::from(at) + v3(0, 0, 0.06);
                    if is_low() {
                        continue;
                    }
                    rose_leaf!(
                        format!("leaf{i}_{k}_{j}"),
                        base,
                        yaw,
                        t,
                        0.17,
                        cols[(j + k) % 4],
                        pairs = 2,
                        droop = 0.35
                    );
                }
            }
        }
    };
    let crop = || {
        let cols_f = [
            m("bloom"),
            m("bloom2"),
            m("bloom3"),
            m("bloom4"),
            m("flesh_light"),
        ];
        for (i, &at) in fruit_at.iter().enumerate() {
            lump!(
                format!("tomato{i}"),
                0.035,
                at,
                cols_f[i % 5],
                noise = 0.05,
                segments = low(7, 5),
                rings = low(5, 4),
                scale = (1, 1, 0.85)
            );
            if !is_low() {
                rays!(
                    format!("calyx{i}"),
                    V3::from(at) + v3(0, 0, 0.058),
                    0.022,
                    5,
                    m("stalk"),
                    inner = 0.2,
                    cup = -0.2,
                    width = 0.5,
                    thick = 0.003
                );
            }
        }
    };
    _crop!(still, (0, 0, top), leaves, crop, size = (0.8, 1.19, 0.8))
}

/// Strawberry bed: a bed of straw, plants of three-lobed, toothed leaves on stalks round a dark heart, white
/// flowers and red berries with green caps tucked in.
pub fn strawberries() -> Vec<Obj> {
    let p = palette(&[
        ("bed", "#D8C27A"),
        ("leaf", "#4A7A36"),
        ("leaf_dark", "#5A8A3C"),
        ("leaf_light", "#6A9A44"),
        ("bloom", "#D8263A"),
        ("bloom5", "#F6F4EE"),
        ("centre", "#F2C12E"),
        ("stalk", "#4A7A36"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(83);
    let straw = revolve!(
        "bed",
        &[
            (0.0, 0.0),
            (0.44, 0.0),
            (0.43, 0.012),
            (0.4, 0.026),
            (0.3, 0.045),
            (0.0, 0.05)
        ],
        material = m("bed"),
        segments = low(18, 10)
    );
    // loose straw: the same on a far version
    straw.edit(|o| {
        for v in &mut o.mesh.verts {
            if v.z as f64 > 0.004 {
                v.z = (v.z as f64 + 0.006 * noise_at(v.x as f64 * 14.0, v.y as f64 * 14.0, 0.3))
                    as f32;
            }
        }
    });
    let still = join_all("strawberries");
    let crowns = spiral!(7, 0.32, &mut rng);
    let berries = spiral!(16, 0.4, &mut rng);
    let flowers = spiral!(4, 0.3, &mut rng);
    let leaves = || {
        // Each plant a dense rosette round its crown: leaves of three toothed leaflets spreading on stalks, two
        // standing up in the middle, over a dark heart, so it closes even flattened to a seedling.
        for (i, &(x, y, a)) in crowns.iter().enumerate() {
            if !is_low() {
                // a far version: hidden under the leaves
                lump!(
                    format!("heart{i}"),
                    0.04,
                    (x, y, 0.03),
                    m("leaf"),
                    scale = (1, 1, 0.8),
                    noise = 0.2,
                    seed = i as f64 * 1.7,
                    segments = 6,
                    rings = 4
                );
            }
            for j in 0..4 {
                let yaw = a + (j * 90) as f64;
                let end = v3(x, y, 0.045) + V3::from(polar(0.06, yaw, 0.04));
                if is_low() && j % 2 != 0 {
                    // a far version: two leaves a plant, bigger
                    continue;
                }
                rays!(
                    format!("crown{i}_{j}"),
                    end,
                    low(0.065, 0.08),
                    3,
                    [m("leaf"), m("leaf_dark"), m("leaf_light")][(i + j) % 3],
                    yaw = yaw + 90.0,
                    tilt = 20,
                    inner = 0.12,
                    cup = 0.2,
                    droop = 0.12,
                    width = 0.95,
                    thick = 0.004
                );
            }
            for j in 0..low(2, 0) {
                let yaw = a + 36.0 + (j * 180) as f64;
                rays!(
                    format!("inner{i}_{j}"),
                    v3(x, y, 0.07) + V3::from(polar(0.02, yaw, 0.03)),
                    0.05,
                    3,
                    m("leaf_light"),
                    yaw = yaw + 90.0,
                    tilt = 55,
                    inner = 0.12,
                    cup = 0.3,
                    width = 0.95,
                    thick = 0.004
                );
            }
        }
    };
    let crop = || {
        for (i, &(x, y, _)) in berries.iter().enumerate() {
            cone!(
                format!("berry{i}"),
                0.006,
                0.024,
                0.05,
                at = (x, y, 0.07),
                material = m("bloom"),
                verts = low(6, 4)
            );
            if !is_low() {
                rays!(
                    format!("cap{i}"),
                    (x, y, 0.094),
                    0.02,
                    3,
                    m("stalk"),
                    yaw = (i * 30) as f64,
                    tilt = 0,
                    inner = 0.2,
                    cup = 0.1,
                    width = 0.6,
                    thick = 0.003
                );
            }
        }
        for (i, &(x, y, _)) in flowers.iter().enumerate() {
            // plain, as the berries
            stem!(
                format!("flower_stem{i}"),
                (x, y, 0.04),
                (x, y, 0.2),
                m("stalk"),
                r = 0.003
            );
            rays!(
                format!("flower{i}"),
                (x, y, 0.2),
                0.024,
                5,
                m("bloom5"),
                yaw = 0,
                tilt = 10,
                inner = 0.3,
                cup = 0.12,
                width = 1.1,
                centre = 0.005
            );
        }
    };
    // seedlings: flat on the straw
    _crop!(
        still,
        (0, 0, 0.05),
        leaves,
        crop,
        sink = (0.0, 0.04, 0.12),
        size = (0.88, 0.21, 0.88)
    )
}

/// Herb planter: a planter of boards on square legs, with a rim, of basil, upright rosemary, chives (some in
/// flower) and grey sage; a resident picking herbs takes a few sprigs of basil.
pub fn herbs() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#8A6446"),
        ("leaf", "#6AAE3E"),
        ("leaf_dark", "#5E9E38"),
        ("leaf_light", "#8EA088"),
        ("stem", "#3E5E3A"),
        ("blossom", "#4E8A3A"),
        ("bloom", "#B88AD0"),
    ]);
    let m = |name: &str| p.get(name);
    let body = m("body");
    let mut rng = Random::new(84);
    // boards, two to a side
    for y in [-0.21, 0.21] {
        for k in 0..2 {
            cube!(
                format!("board{}_{k}", fr(y)),
                (0.84, 0.04, 0.106),
                at = (0, y, 0.564 + k as f64 * 0.11),
                material = body
            );
        }
    }
    for x in [-0.4, 0.4] {
        cube!(
            format!("board_end{}", fr(x)),
            (0.04, 0.38, 0.216),
            at = (x, 0, 0.62),
            material = body
        );
    }
    cube!(
        "bottom",
        (0.76, 0.38, 0.02),
        at = (0, 0, 0.52),
        material = body
    );
    for x in [-0.38, 0.38] {
        for y in [-0.19, 0.19] {
            cube!(
                format!("leg{}{}", fr(x), fr(y)),
                (0.055, 0.055, 0.52),
                at = (x, y, 0.26),
                material = body,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
    }
    // stretchers between the legs
    for x in [-0.36, 0.36] {
        cube!(
            format!("stretcher{}", fr(x)),
            (0.03, 0.33, 0.04),
            at = (x, 0, 0.12),
            material = body
        );
    }
    cube!(
        "soil",
        (0.76, 0.38, 0.02),
        at = (0, 0, 0.715),
        material = m("soil")
    );
    lump!(
        "basil2",
        0.07,
        (-0.25, -0.1, 0.71),
        m("leaf_dark"),
        scale = (1, 1, 0.75),
        noise = 0.35,
        seed = 1,
        segments = 7,
        rings = 5
    );
    let mut r2 = Random::new(184);
    // basil leaves on it
    for i in 0..8 {
        let a = (i * 45) as f64;
        let (yaw, t) = (a + r2.uniform(-15.0, 15.0), r2.uniform(10.0, 40.0));
        if is_low() && i % 3 != 0 {
            continue;
        }
        cut_leaf!(
            format!("basil2_leaf{i}"),
            v3(-0.25, -0.1, 0.74) + V3::from(polar(0.045, a, 0.02)),
            yaw,
            t,
            0.07,
            &outline!("oval", 0.045, n = 3),
            m("leaf_dark"),
            droop = 0.4,
            fold = 0.5,
            ribs = 2
        );
    }
    // rosemary
    for i in 0..7 {
        let base = (
            -0.06 + rng.uniform(-0.05, 0.05),
            rng.uniform(-0.1, 0.1),
            0.72,
        );
        let (yaw, t, length) = (
            rng.uniform(0.0, 360.0),
            rng.uniform(72.0, 90.0),
            rng.uniform(0.2, 0.28),
        );
        cut_leaf!(
            format!("rosemary{i}"),
            base,
            yaw,
            t,
            length,
            &pinnate!(7, 0.045, "strap", lean = 0.9, fill = 0.25),
            m("stem"),
            droop = 0.08,
            fold = 0.1,
            ribs = 3
        );
    }
    // chives, some in flower
    for i in 0..6 {
        let base = (
            0.14 + rng.uniform(-0.04, 0.04),
            rng.uniform(-0.1, 0.1),
            0.72,
        );
        let h = rng.uniform(0.2, 0.26);
        let (yaw, t) = (rng.uniform(0.0, 360.0), rng.uniform(78.0, 90.0));
        for j in 0..low(3, 1) {
            let d = V3::from(polar(
                cos(radians(t)) * (1.0 + j as f64 * 0.4),
                yaw + (j * 120) as f64,
                sin(radians(t)),
            ));
            twig!(
                format!("chive{i}_{j}"),
                &[
                    V3::from(base),
                    V3::from(base) + d * h * (0.9 - j as f64 * 0.1)
                ],
                0.004,
                0.0015,
                m("blossom"),
                verts = 3,
                res = 1
            );
        }
        if i % 2 == 0 {
            florets!(
                format!("chive_flower{i}"),
                0.02,
                (base.0, base.1, base.2 + h - 0.01),
                m("bloom"),
                bumps = 3.0,
                amount = 0.25,
                segments = low(6, 4),
                rings = low(5, 3),
                seed = i as f64
            );
        }
    }
    // sage
    lump!(
        "sage",
        0.085,
        (0.31, 0.04, 0.71),
        m("leaf_light"),
        scale = (1, 1, 0.65),
        noise = 0.3,
        seed = 3,
        segments = 7,
        rings = 5
    );
    lump!(
        "sage2",
        0.065,
        (0.3, -0.12, 0.71),
        m("leaf_light"),
        scale = (1, 1, 0.7),
        noise = 0.3,
        seed = 4,
        segments = 6,
        rings = 4
    );
    for i in 0..9 {
        let a = (i * 40) as f64;
        let (yaw, t) = (a + r2.uniform(-15.0, 15.0), r2.uniform(15.0, 45.0));
        if is_low() && i % 3 != 0 {
            continue;
        }
        let at = (if i < 6 {
            v3(0.31, 0.04, 0.72)
        } else {
            v3(0.3, -0.12, 0.72)
        }) + V3::from(polar(0.05, a, 0.03));
        cut_leaf!(
            format!("sage_leaf{i}"),
            at,
            yaw,
            t,
            0.08,
            &outline!("lance", 0.035, n = 3),
            m("leaf_light"),
            droop = 0.2,
            fold = 0.2,
            ribs = 2,
            thick = 0.005
        );
    }
    let at = (-0.29, 0.05, 0.72);
    let basil = || {
        lump!(
            "basil",
            0.085,
            at,
            m("leaf"),
            scale = (1, 1, 0.75),
            noise = 0.35,
            seed = 2,
            segments = 7,
            rings = 5
        );
        for i in 0..low(8, 3) {
            let a = (i * 45 + 20) as f64;
            cut_leaf!(
                format!("basil_leaf{i}"),
                V3::from(at) + V3::from(polar(0.05, a, 0.04)),
                a,
                (25 + (i % 3) * 10) as f64,
                0.075,
                &outline!("oval", 0.05, n = 3),
                m("leaf"),
                droop = 0.4,
                fold = 0.5,
                ribs = 2
            );
        }
    };
    _fit_with_part("sprigs", "herbs", basil, at, "harvest", (0.84, 1.01, 0.46))
}

/// Pumpkin patch: a patch of soil, vines with curling tendrils, big lobed leaves on long stalks and a yellow
/// flower, and three ribbed pumpkins on their stalks, grown from seedlings to ripe.
pub fn pumpkins() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#4A7A36"),
        ("leaf_dark", "#5A8A3C"),
        ("leaf_light", "#6A9A44"),
        ("stem", "#7AA84C"),
        ("bloom", "#E8822A"),
        ("stalk", "#5E6A2E"),
        ("blossom", "#F2C12E"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(85);
    revolve!(
        "bed",
        &[
            (0.0, 0.0),
            (0.46, 0.0),
            (0.42, 0.02),
            (0.3, 0.035),
            (0.0, 0.04)
        ],
        material = m("bed"),
        segments = low(20, 12),
        scale = (1.1, 1.0, 1)
    );
    let still = join_all("pumpkins");
    let mut spots = Vec::new();
    for i in 0..9 {
        let a = i as f64 / 9.0 * 360.0 + rng.uniform(-15.0, 15.0);
        let rr = rng.uniform(0.12, 0.34);
        spots.push((polar(rr, a, 0.03), a, rng.uniform(5.0, 20.0)));
    }
    let cols = [m("leaf"), m("leaf_dark"), m("leaf_light"), m("stem")];
    let leaves = || {
        // the vines, wandering over the soil
        for i in 0..3 {
            let a = (i * 120 + 30) as f64;
            let pts: Vec<V3> = (0..4)
                .map(|k| {
                    V3::from(polar(
                        0.02 + 0.13 * k as f64,
                        a + 25.0 * sin(k as f64 * 1.3 + i as f64),
                        0.045,
                    ))
                })
                .collect();
            twig!(format!("vine{i}"), &pts, 0.008, 0.005, m("stem"), verts = 4);
            if !is_low() {
                // a tendril curling off its end
                let end = pts[3];
                let curl: Vec<V3> = (0..7)
                    .map(|k| {
                        let t = k as f64 / 6.0;
                        end + v3(0.03 * cos(t * 2.5), 0.03 * sin(t * 2.5), 0.03 * t)
                            * (1.0 - t * 0.5)
                    })
                    .collect();
                sweep!(
                    format!("tendril{i}"),
                    &curl,
                    0.002,
                    material = m("stem"),
                    verts = 3
                );
            }
        }
        for (i, &(base, a, tilt)) in spots.iter().enumerate() {
            let end = V3::from(base) + V3::from(polar(0.05, a, 0.1));
            stalk!(
                format!("petiole{i}"),
                base,
                end,
                m("stem"),
                r = 0.005,
                bow = 0.02
            );
            cut_leaf!(
                format!("leaf{i}"),
                end,
                a,
                tilt,
                0.24,
                &outline!(lobed!(5), 0.27, n = 10, teeth = 0.05),
                cols[i % 4],
                droop = 0.22,
                fold = 0.3,
                ribs = 4,
                thick = 0.008
            );
        }
        cupped!(
            "flower",
            &[
                (0.0, 0.0),
                (0.01, 0.0),
                (0.02, 0.04),
                (0.045, 0.07),
                (0.035, 0.07)
            ],
            (-0.05, -0.3, 0.04),
            m("blossom"),
            yaw = 200,
            tilt = 55,
            segments = low(6, 3)
        );
        // (turned, then tilted: round its own axis no longer)
        crate::box_uvs(&["flower"]);
    };
    let crop = || {
        for (i, (x, y, r)) in [(0.18, -0.12, 0.17), (-0.2, 0.05, 0.14), (-0.02, 0.3, 0.11)]
            .into_iter()
            .enumerate()
        {
            let n = low(18, 8);
            let secs: Vec<Vec<P3>> = [
                (0.0, r * 0.3),
                (r * 0.12, r * 0.75),
                (r * 0.45, r),
                (r * 0.85, r * 0.9),
                (r * 1.08, r * 0.55),
                (r * 1.15, r * 0.18),
            ]
            .iter()
            .map(|&(z, rr)| {
                ribbed!(rr, n / 2, depth = 0.1, verts = n)
                    .iter()
                    .map(|&(u, v)| P3::from((x + u, y + v, 0.02 + z)))
                    .collect()
            })
            .collect();
            loft!(format!("pumpkin{i}"), &secs, material = m("bloom"));
            twig!(
                format!("stalk{i}"),
                &[
                    (x, y, 0.02 + r * 1.1),
                    (x + 0.008, y, 0.02 + r * 1.1 + 0.04),
                    (x + 0.025, y, 0.02 + r * 1.1 + 0.06)
                ],
                0.013,
                0.009,
                m("stalk"),
                verts = 5,
                res = 1
            );
        }
    };
    _crop!(
        still,
        (0, 0, 0.04),
        leaves,
        crop,
        size = (1.112, 0.269, 0.947)
    )
}
