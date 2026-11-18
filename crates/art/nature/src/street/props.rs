//! The bench, the planter, the flower tub, the door pot, the path light and the flower clump.

use super::common::{_bloom, _tub, BenchEnd, bench_end};
use super::{F, M};
use artkit::bvh::Rays;
use artkit::far::{is_low, low};
use artkit::json::float_repr;
use artkit::math::{TAU, cos, pow, sin, sqrt, v3};
use artkit::parts::{Settle, join_all, settle_all, settle_with};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{cylinder, fbox, fcone, fcyl, fsphere, fsweep, jitter, lump, revolve, sweep};

// --- the bench: four slats on two cast-iron ends, its back to the road ---------------------------------------
pub fn bench() -> Vec<Obj> {
    let full = !is_low();
    // the ends: each cast in one piece (the back leg curving up into the upright, the front leg bending back into
    // the armrest, its front curled in a scroll, the seat rail between), on pad feet
    for x in [-0.8, 0.8] {
        bench_end(
            &F,
            &format!("end{}", float_repr(x)),
            x,
            0.415,
            0.19,
            -0.21,
            (0.885, -0.272),
            M.iron,
            BenchEnd {
                arm: Some(0.68),
                t: 0.026,
                w: 0.042,
                steps: low(3, 1),
                corner: low(1, 0),
                scrolls: full,
                ..BenchEnd::default()
            },
        );
    }
    // the slats: rounded edges, held down by domed bolts over each end
    for (k, z) in [-0.1408, -0.0484, 0.044, 0.1364].into_iter().enumerate() {
        fbox!(
            F,
            format!("slat{k}"),
            1.7,
            0.035,
            0.085,
            (0, 0.4325, z),
            M.wood,
            bevel = low(0.009, 0.0),
            bevel_segments = 2
        );
        if full {
            for x in [-0.8, 0.8] {
                fsphere!(
                    F,
                    format!("bolt{k}{}", float_repr(x)),
                    0.009,
                    (x, 0.45, z),
                    M.iron,
                    segments = 6,
                    rings = 3,
                    scale = (1, 1, 0.5)
                );
            }
        }
    }
    for (k, (y, z)) in [(0.6, -0.18), (0.73, -0.225), (0.86, -0.27)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            F,
            format!("back{k}"),
            1.7,
            0.085,
            0.028,
            (0, y, z),
            M.wood,
            bevel = low(0.009, 0.0),
            bevel_segments = 2,
            rot = (-12, 0, 0)
        );
        if full {
            for x in [-0.8, 0.8] {
                fsphere!(
                    F,
                    format!("back_bolt{k}{}", float_repr(x)),
                    0.009,
                    (x, y, z + 0.016),
                    M.iron,
                    segments = 6,
                    rings = 3,
                    scale = (1, 0.5, 1)
                );
            }
        }
    }
    join_all("bench");
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    meshes()
}

pub fn planter() -> Vec<Obj> {
    let full = !is_low();
    _tub(true);
    // a young street tree: a slim trunk (plain satin), tapering, forking under a crown of leafy lumps
    sweep!(
        "trunk",
        &[
            F.at(0, 0.37, 0),
            F.at(0.008, 0.9, 0),
            F.at(-0.006, 1.4, 0.006),
            F.at(0.0, 1.85, 0.0)
        ],
        0.038,
        material = M.trunk,
        verts = low(8, 4),
        resolution = low(2, 1),
        scale = [1.0, 0.8, 0.68, 0.55]
    );
    fcone!(
        F,
        "flare",
        0.06,
        0.036,
        0.06,
        (0, 0.42, 0),
        M.trunk,
        verts = low(8, 4)
    );
    for (i, (x, z)) in [(0.16, 0.06), (-0.15, -0.1), (0.02, -0.14), (-0.05, 0.15)]
        .into_iter()
        .enumerate()
    {
        let y0 = 1.45 + i as f64 * 0.07;
        sweep!(
            format!("branch{i}"),
            &[
                F.at(0, y0, 0),
                F.at(x * 0.5, y0 + 0.2, z * 0.5),
                F.at(x, y0 + 0.38, z)
            ],
            0.017,
            material = M.trunk,
            verts = low(5, 3),
            resolution = low(2, 1),
            scale = [1.0, 0.8, 0.55]
        );
    }
    // two stakes either side of the trunk, tied to it with a rubber band on a spacer
    for i in [-1, 1] {
        let s = i as f64;
        fcyl!(
            F,
            format!("stake{i}"),
            0.018,
            1.0,
            (s * 0.12, 0.89, 0.02 * s),
            M.stake,
            verts = low(8, 4)
        );
        if full {
            fcone!(
                F,
                format!("stake_top{i}"),
                0.018,
                0.012,
                0.012,
                (s * 0.12, 1.396, 0.02 * s),
                M.stake,
                verts = 8
            );
        }
    }
    fbox!(
        F,
        "tie",
        0.24,
        0.03,
        0.012,
        (0, 1.2, 0.0),
        M.tie,
        bevel = low(0.003, 0.0),
        bevel_segments = 1,
        rot = (0, 0, 9)
    );
    // radius, bottom (game x, height, depth), material, height share
    let crown = [
        (0.36, (0.0, 1.64, 0.0), M.leaf, 0.86),
        (0.24, (0.15, 1.98, 0.08), M.leaf_light, 0.9),
        (0.22, (-0.16, 1.9, -0.12), M.leaf_dark, 0.9),
        (0.2, (0.2, 1.76, -0.13), M.leaf_dark, 0.85),
        (0.2, (-0.2, 1.72, 0.13), M.leaf_light, 0.85),
        (0.18, (0.0, 2.1, -0.02), M.leaf_light, 0.9),
        (0.16, (0.1, 1.62, 0.1), M.leaf_dark, 0.8), // small lumps filling the crown's underside and its gaps
        (0.15, (-0.14, 1.62, -0.08), M.leaf, 0.8),
        (0.15, (-0.04, 2.0, 0.08), M.leaf, 0.85),
    ];
    for (i, (r, (x, y, z), mat, h)) in crown.into_iter().enumerate() {
        lump!(
            format!("crown{i}"),
            r,
            F.at(x, y, z),
            mat,
            scale = (1, 1, h),
            noise = 0.14,
            seed = i as f64 * 1.7,
            segments = 12,
            rings = 7
        );
    }
    join_all("planter");
    settle_all();
    meshes()
}

pub fn flower_tub() -> Vec<Obj> {
    let full = !is_low();
    _tub(false);
    let mut rng = Random::new(81);
    // a leafy mound over the soil, and flowers in five colours on it, the old part's five where it had them
    lump!(
        "mound",
        0.27,
        F.at(0, 0.39, 0),
        M.leaf,
        scale = (1, 1, 0.25),
        noise = 0.16,
        segments = 12,
        rings = 6
    );
    for (i, (x, z)) in [(0.2, 0.2), (-0.21, 0.19), (0.21, -0.2), (-0.2, -0.2)]
        .into_iter()
        .enumerate()
    {
        lump!(
            format!("leaves{i}"),
            0.11,
            F.at(x, 0.4, z),
            [M.leaf_dark, M.leaf_light][i % 2],
            scale = (1, 1, 0.6),
            noise = 0.2,
            seed = i as f64 * 2.3,
            segments = 7,
            rings = 4
        );
    }
    let big = [
        (-0.15, -0.12, M.red),
        (0.14, -0.1, M.yellow),
        (0.0, 0.14, M.pink),
        (-0.16, 0.16, M.blue),
        (0.17, 0.15, M.white),
    ];
    let cols = [M.red, M.yellow, M.pink, M.blue, M.white, M.violet];
    let mut spots = big.to_vec();
    for i in 0..12 {
        let (x, z) = (rng.uniform(-0.24, 0.24), rng.uniform(-0.24, 0.24));
        spots.push((x, z, cols[i % cols.len()]));
    }
    for (i, (x, z, mat)) in spots.into_iter().enumerate() {
        let drop = if i < big.len() {
            0.0
        } else {
            rng.uniform(0.0, 0.015)
        };
        let tilt = (
            rng.uniform(-12.0, 12.0),
            rng.uniform(-12.0, 12.0),
            rng.uniform(0.0, 72.0),
        );
        let y = (if i < big.len() { 0.562 } else { 0.55 - drop }) - 0.06 * (x * x + z * z) / 0.08;
        let r = if i < big.len() { 0.07 } else { 0.045 };
        if full {
            _bloom(
                &F,
                &format!("bloom{i}"),
                x,
                y,
                z,
                r,
                if i % 3 != 0 { 5 } else { 6 },
                mat,
                if mat.name != M.yellow.name {
                    M.eye
                } else {
                    M.eye_dark
                },
                tilt,
                if i < big.len() { 5 } else { 2 },
            );
        } else if i < big.len() {
            // far off, the big blooms only, as lumps
            lump!(
                format!("bloom{i}"),
                r * 0.8,
                F.at(x, y - 0.045, z),
                mat,
                scale = (1, 1, 0.5),
                noise = 0.2,
                seed = i as f64 * 0.9,
                segments = 5,
                rings = 3
            );
        }
    }
    join_all("flowerTub");
    settle_all();
    meshes()
}

// --- the pot by a front door: terracotta on a saucer, a clipped leafy ball ------------------------------------
pub fn pot() -> Vec<Obj> {
    let full = !is_low();
    let n = low(20, 8);
    // the pot: a foot ring, the body flaring up to a rolled rim, and inside down to the soil
    let whole = [
        (0.0, 0.004),
        (0.142, 0.004),
        (0.15, 0.0),
        (0.156, 0.012),
        (0.152, 0.024),
        (0.178, 0.3),
        (0.184, 0.306),
        (0.2, 0.312),
        (0.205, 0.34),
        (0.2, 0.366),
        (0.186, 0.372),
        (0.172, 0.366),
        (0.168, 0.352),
        (0.0, 0.352),
    ];
    let far = [
        (0.0, 0.0),
        (0.155, 0.0),
        (0.18, 0.31),
        (0.2, 0.312),
        (0.2, 0.37),
        (0.17, 0.37),
        (0.0, 0.35),
    ];
    revolve!(
        "pot",
        if full { &whole[..] } else { &far[..] },
        material = M.terracotta,
        segments = n
    );
    if full {
        revolve!(
            "saucer",
            &[
                (0.13, 0.002),
                (0.182, 0.0),
                (0.198, 0.026),
                (0.192, 0.03),
                (0.178, 0.012),
                (0.13, 0.012)
            ],
            material = M.terracotta,
            segments = n
        );
    }
    let soil = cylinder!(
        "soil",
        0.168,
        0.012,
        at = (0, 0, 0.356),
        material = M.soil,
        verts = n
    );
    if full {
        jitter!(soil, 0.003, size = 0.05, seed = 1.0);
    }
    lump!(
        "ball",
        low(0.205, 0.218),
        F.at(0, 0.32, 0),
        M.leaf,
        scale = (low(1.01, 1.06), 1.005, 1.0),
        noise = 0.1,
        segments = 16,
        rings = 8
    );
    lump!(
        "top",
        0.13,
        F.at(0.08, 0.63, 0.04),
        M.leaf_light,
        scale = (1, 1, 0.94),
        noise = 0.15,
        seed = 2.0,
        segments = 12,
        rings = 7
    );
    lump!(
        "side",
        0.1,
        F.at(-0.12, 0.5, -0.03),
        M.leaf_dark,
        scale = (1, 1, 0.9),
        noise = 0.15,
        seed = 4.0,
        segments = 10,
        rings = 6
    );
    join_all("pot");
    settle_all();
    meshes()
}

// --- the garden path light: a dark bollard with a frosted lens under louvres and a cap ---------------------------
pub fn path_light() -> Vec<Obj> {
    let full = !is_low();
    let n = low(16, 8);
    let foot: &[(f64, f64)] = if full {
        &[(0.08, 0.008), (0.068, 0.014), (0.058, 0.04), (0.054, 0.07)]
    } else {
        &[(0.07, 0.02)]
    };
    let post = [
        &[(0.0, 0.0), (if full { 0.08 } else { 0.07 }, 0.0)],
        foot,
        &[(0.054, 0.4), (0.06, 0.41), (0.06, 0.42), (0.0, 0.42)],
    ]
    .concat();
    revolve!("post", &post, material = M.lamp, segments = n);
    fcyl!(F, "lens", 0.052, 0.1, (0, 0.47, 0), M.lens, verts = n);
    if full {
        // three louvre rings round the lens, on four thin bars
        for (k, y) in [0.444, 0.47, 0.496].into_iter().enumerate() {
            revolve!(
                format!("louvre{k}"),
                &[
                    (0.054, y - 0.004),
                    (0.068, y - 0.009),
                    (0.07, y - 0.006),
                    (0.058, y + 0.002),
                    (0.054, y + 0.002)
                ],
                material = M.lamp,
                segments = n
            );
        }
        for k in 0..4 {
            let a = TAU * (k as f64 + 0.5) / 4.0;
            fbox!(
                F,
                format!("bar{k}"),
                0.008,
                0.1,
                0.008,
                (0.066 * cos(a), 0.47, 0.066 * sin(a)),
                M.lamp
            );
        }
        // the access hatch's screw
        fcyl!(
            F,
            "screw",
            0.006,
            0.01,
            (0, 0.2, 0.054),
            M.lamp,
            verts = 6,
            rot = (90, 0, 0)
        );
    }
    let rim: &[(f64, f64)] = if full {
        &[(0.09, 0.525), (0.075, 0.54), (0.045, 0.553), (0.015, 0.559)]
    } else {
        &[(0.09, 0.525)]
    };
    let cap = [&[(0.0, 0.515), (0.09, 0.515)], rim, &[(0.0, 0.56)]].concat();
    revolve!("cap", &cap, material = M.lamp, segments = n);
    join_all("pathLight");
    settle_all();
    meshes()
}

// --- the street foliage --------------------------------------------------------------------------------------
/// A clump of leaves and little flowers in many colours, for the beds along the house fronts.
pub fn flowers() -> Vec<Obj> {
    let mut rng = Random::new(44);
    lump!(
        "clump",
        0.29,
        F.at(0, 0.0, 0),
        M.bed,
        scale = (1, 0.9, 0.45),
        noise = 0.2,
        segments = 16,
        rings = 7
    );
    lump!(
        "clump2",
        0.17,
        F.at(0.12, 0.08, -0.06),
        M.bed_light,
        scale = (1, 0.9, 0.6),
        noise = 0.2,
        seed = 1.5,
        segments = 10,
        rings = 6
    );
    lump!(
        "clump3",
        0.14,
        F.at(-0.16, 0.05, 0.08),
        M.bed_light,
        scale = (1, 0.9, 0.55),
        noise = 0.2,
        seed = 3.5,
        segments = 8,
        rings = 5
    );
    if !is_low() {
        // smaller leafy lumps breaking up its front and sides
        for (k, (x, z, r)) in [(0.06, 0.15, 0.1), (0.19, 0.05, 0.09), (-0.2, -0.08, 0.09)]
            .into_iter()
            .enumerate()
        {
            lump!(
                format!("tuft{k}"),
                r,
                F.at(x, 0.06, z),
                [M.bed_light, M.bed, M.bed_light][k],
                scale = (1, 1, 0.75),
                noise = 0.25,
                seed = 5.0 + k as f64,
                segments = 8,
                rings = 5
            );
        }
    }
    let cols = [M.rose, M.yellow, M.white, M.violet, M.red, M.pink, M.blue];
    for i in 0..14 {
        let a = i as f64 * 2.39996 + rng.uniform(-0.3, 0.3);
        let rr = 0.24 * sqrt((i as f64 + 0.5) / 14.0);
        let (x, z) = (rr * cos(a) * 1.05, rr * sin(a) * 0.95);
        let y = 0.295 - pow(rr / 0.24, 2.0) * 0.07 + rng.uniform(-0.015, 0.015);
        let tilt = (
            rng.uniform(-15.0, 15.0),
            rng.uniform(-15.0, 15.0),
            rng.uniform(0.0, 72.0),
        );
        let mat = cols[i % cols.len()];
        if is_low() {
            if i % 2 != 0 {
                // far off, half the flowers (each drawn, so the rest stand put)
                continue;
            }
            lump!(
                format!("bloom{i}"),
                0.036,
                F.at(x, y, z),
                mat,
                scale = (1, 1, 0.7),
                noise = 0.25,
                seed = i as f64 * 0.7,
                segments = 4,
                rings = 3
            );
            continue;
        }
        // each flower held a little over the leaves on its stem, from where the clump's surface is under it (or
        // a flower's made before it)
        let hit = Rays::of_scene().cast(F.at(x, 1.0, z).into(), v3(0, 0, -1), 1.70141e38);
        let ground = hit.map_or(y, |hit| hit.at.z as f64);
        let top = (ground + 0.012).max(y + 0.008 + 0.03 * (1.0 - rr / 0.24));
        fsweep!(
            F,
            format!("stem{i}"),
            &[(x, ground - 0.01, z), (x * 1.02, top, z * 1.02)],
            0.004,
            M.bed,
            verts = 4,
            caps = false
        );
        _bloom(
            &F,
            &format!("bloom{i}"),
            x * 1.02,
            top,
            z * 1.02,
            0.05,
            if i % 3 != 0 { 5 } else { 4 },
            mat,
            if mat.name != M.yellow.name {
                M.eye
            } else {
                M.eye_dark
            },
            (tilt.0 + 40.0 * z, tilt.1 - 40.0 * x, tilt.2),
            4,
        );
    }
    join_all("flowers");
    settle_all();
    meshes()
}
