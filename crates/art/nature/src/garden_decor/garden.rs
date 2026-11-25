//! The gnome, the wheelbarrow planter and the boulders.

use super::M;
use super::common::_on_top;
use crate::ground::rock::_stone;
use artkit::forms::rounded_rect;
use artkit::kit::fillet;
use artkit::math::{P3, TAU, cos, degrees, sin};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{
    boulder, cube, cylinder, dome, jitter, leaf, loft, lump, revolve, ring, sphere, star, sweep,
    torus,
};

/// A garden gnome: boots with turned-up toes, a belted tunic with a brass buckle and two buttons, hands on his
/// belly, a round face with rosy cheeks, a nose, eyes and brows, a flowing beard and moustache, a floppy red hat.
pub fn gnome() -> Vec<Obj> {
    for i in [-1, 1] {
        let s = i as f64;
        sweep!(
            format!("boot{i}"),
            &[
                (s * 0.042, 0.03, 0.02),
                (s * 0.044, -0.03, 0.02),
                (s * 0.046, -0.065, 0.024),
                (s * 0.047, -0.08, 0.038)
            ],
            0.0,
            material = M.wood_dark,
            profile = rounded_rect(0.04, 0.052, 0.017, 2),
            up = (0, 0, 1),
            resolution = 2,
            scale = [1.0, 1.05, 0.85, 0.45]
        );
    }
    revolve!(
        "tunic",
        &[
            (0.0, 0.03),
            (0.09, 0.03),
            (0.095, 0.042),
            (0.09, 0.08),
            (0.082, 0.15),
            (0.075, 0.22),
            (0.077, 0.255),
            (0.072, 0.272),
            (0.055, 0.29),
            (0.0, 0.297)
        ],
        material = M.tunic,
        segments = 20
    );
    torus!(
        "hem",
        0.088,
        0.008,
        at = (0, 0, 0.04),
        material = M.tunic,
        major_segments = 20,
        minor_segments = 4
    );
    torus!(
        "belt",
        0.08,
        0.013,
        at = (0, 0, 0.20),
        material = M.wood_dark,
        major_segments = 20,
        minor_segments = 4
    );
    cube!(
        "buckle",
        (0.042, 0.01, 0.034),
        at = (0, -0.086, 0.2),
        material = M.buckle,
        bevel = 0.003,
        bevel_segments = 1
    );
    cube!(
        "buckle_hole",
        (0.026, 0.01, 0.018),
        at = (0, -0.088, 0.2),
        material = M.wood_dark
    );
    for (k, z) in [0.235, 0.262].into_iter().enumerate() {
        sphere!(
            format!("button{k}"),
            0.008,
            at = (0, -0.075 + (z - 0.235) * 0.1, z),
            material = M.buckle,
            segments = 8,
            rings = 5
        );
    }
    // the arms, sleeves in one piece from the shoulder, round to the belly, the hands resting on it
    for i in [-1, 1] {
        let s = i as f64;
        sweep!(
            format!("arm{i}"),
            &[
                (s * 0.062, 0.0, 0.272),
                (s * 0.095, -0.012, 0.235),
                (s * 0.09, -0.04, 0.205),
                (s * 0.066, -0.064, 0.21)
            ],
            0.027,
            material = M.tunic,
            verts = 8,
            resolution = 2,
            scale = [0.9, 1.0, 0.9, 1.05]
        );
        sphere!(
            format!("hand{i}"),
            0.022,
            at = (s * 0.045, -0.078, 0.212),
            material = M.skin,
            scale = (1.1, 0.9, 1.0),
            segments = 10,
            rings = 6
        );
    }
    // the head: a round face, ears, a big nose, rosy cheeks, eyes under bushy brows
    sphere!(
        "head",
        0.062,
        at = (0, -0.006, 0.345),
        material = M.skin,
        scale = (1, 1.02, 1),
        segments = 14,
        rings = 9
    );
    for i in [-1, 1] {
        let s = i as f64;
        sphere!(
            format!("ear{i}"),
            0.017,
            at = (s * 0.06, 0.0, 0.35),
            material = M.skin,
            scale = (0.5, 1, 1.3),
            segments = 8,
            rings = 5
        );
        sphere!(
            format!("cheek{i}"),
            0.017,
            at = (s * 0.03, -0.05, 0.332),
            material = M.nose,
            segments = 8,
            rings = 5
        );
        sphere!(
            format!("eye{i}"),
            0.0075,
            at = (s * 0.022, -0.058, 0.36),
            material = M.eye,
            segments = 8,
            rings = 5
        );
        sweep!(
            format!("brow{i}"),
            &[
                (s * 0.012, -0.06, 0.37),
                (s * 0.026, -0.062, 0.376),
                (s * 0.038, -0.056, 0.371)
            ],
            0.0055,
            material = M.beard,
            verts = 5,
            resolution = 2
        );
    }
    sphere!(
        "nose",
        0.021,
        at = (0, -0.066, 0.338),
        material = M.nose,
        segments = 10,
        rings = 6
    );
    // the beard: a long wedge from the cheeks to a point on his chest, a little wavy; a moustache over it
    let rows = [
        // (its top rounded in under the moustache: no turn of 60°)
        (0.35, 0.046, 0.026, -0.024),
        (0.348, 0.052, 0.03, -0.025),
        (0.345, 0.055, 0.032, -0.026),
        (0.315, 0.062, 0.04, -0.04),
        (0.28, 0.056, 0.038, -0.055),
        (0.245, 0.042, 0.03, -0.068),
        (0.215, 0.022, 0.018, -0.078),
        (0.195, 0.004, 0.004, -0.082),
    ];
    let sections: Vec<Vec<P3>> = rows
        .iter()
        // (14 round: at 10 its flattened sides turned 60° at the cheeks)
        .map(|&(z, rx, ry, y)| ring!(rx, ry, z = z, verts = 14, at = (0.0, y)))
        .collect();
    let beard = loft!("beard", &sections, material = M.beard);
    jitter!(beard, 0.004, size = 0.03, seed = 2.0);
    for i in [-1, 1] {
        let s = i as f64;
        sweep!(
            format!("moustache{i}"),
            &[
                (0.0, -0.07, 0.326),
                (s * 0.025, -0.07, 0.322),
                (s * 0.045, -0.062, 0.31)
            ],
            0.011,
            material = M.beard,
            verts = 8,
            resolution = 2,
            scale = [1.0, 0.9, 0.4]
        );
    }
    // the hat: a rolled brim and a tall cone that flops back towards its tip
    torus!(
        "hat_brim",
        0.064,
        0.013,
        at = (0, -0.004, 0.388),
        material = M.hat,
        major_segments = 20,
        minor_segments = 6
    );
    sweep!(
        "hat",
        &[
            (0.0, -0.004, 0.385),
            (0.0, -0.002, 0.45),
            (0.0, 0.01, 0.51),
            (0.0, 0.032, 0.56),
            (0.0, 0.058, 0.583),
            (0.0, 0.078, 0.582)
        ],
        0.07,
        material = M.hat,
        verts = 12,
        resolution = 3,
        scale = [1.0, 0.78, 0.55, 0.32, 0.14, 0.03]
    );
    settle(&[join_all("gnome")])
}

/// A wheelbarrow planter: a pressed steel tray with a rolled rim, sloping out over the wheel, on two wooden
/// handles with rubber grips that run from the axle, two bent steel legs with stays, a spoked wheel with a treaded
/// tyre; the tray full of earth, leafy plants and flowers.
pub fn wheelbarrow() -> Vec<Obj> {
    let mut rng = Random::new(5);
    let green = M.paint;
    // the tray: its walls lofted from a rounded bottom out to the rim, deeper over the wheel; the earth in it
    let rings = [
        ring!(
            0.19,
            0.24,
            z = 0.39,
            verts = 24,
            square = 5,
            at = (0.0, -0.03)
        ),
        ring!(
            0.205,
            0.26,
            z = 0.41,
            verts = 24,
            square = 5,
            at = (0.0, -0.025)
        ),
        ring!(
            0.265,
            0.33,
            z = 0.615,
            verts = 24,
            square = 5,
            at = (0.0, 0.01)
        ),
        ring!(
            0.245,
            0.31,
            z = 0.6,
            verts = 24,
            square = 5,
            at = (0.0, 0.01)
        ),
    ];
    loft!("tray", &rings, material = green, angle = 50);
    sweep!(
        "tray_rim",
        &ring!(
            0.265,
            0.33,
            z = 0.617,
            verts = 24,
            square = 5,
            at = (0.0, 0.01)
        ),
        0.013,
        material = green,
        verts = 6,
        closed = true
    );
    loft!(
        "soil",
        &[
            ring!(
                0.25,
                0.315,
                z = 0.602,
                verts = 16,
                square = 5,
                at = (0.0, 0.01)
            ),
            ring!(
                0.2,
                0.26,
                z = 0.625,
                verts = 16,
                square = 4,
                at = (0.0, 0.01)
            ),
            ring!(
                0.1,
                0.13,
                z = 0.64,
                verts = 16,
                square = 3,
                at = (0.0, 0.01)
            )
        ],
        material = M.soil
    );
    // the handles: from the axle back under the tray and out past it, the grips on their ends
    for i in [-1, 1] {
        let s = i as f64;
        let path = fillet(
            &[
                (s * 0.1, 0.36, 0.165),
                (s * 0.15, 0.22, 0.37),
                (s * 0.17, -0.1, 0.42),
                (s * 0.205, -0.42, 0.57),
            ],
            0.06,
            3,
        );
        sweep!(
            format!("handle{i}"),
            &path,
            0.0,
            material = M.wood,
            profile = rounded_rect(0.04, 0.034, 0.01, 1),
            up = (0, 0, 1)
        );
        sweep!(
            format!("grip{i}"),
            &[(s * 0.199, -0.39, 0.555), (s * 0.212, -0.47, 0.595)],
            0.021,
            material = M.metal_dark,
            verts = 8,
            scale = [1.0, 0.9]
        );
        // a bent steel leg from under the handle down to a foot, a stay up to the tray
        sweep!(
            format!("leg{i}"),
            &fillet(
                &[
                    (s * 0.17, -0.08, 0.42),
                    (s * 0.18, -0.19, 0.03),
                    (s * 0.18, -0.26, 0.012)
                ],
                0.03,
                3
            ),
            0.014,
            material = M.metal_dark,
            verts = 8
        );
        sweep!(
            format!("stay{i}"),
            &[(s * 0.178, -0.16, 0.15), (s * 0.12, -0.1, 0.395)],
            0.008,
            material = M.metal_dark,
            verts = 6
        );
        sweep!(
            format!("foot{i}"),
            &[(s * 0.18, -0.235, 0.008), (s * 0.18, -0.285, 0.008)],
            0.009,
            material = M.metal_dark,
            verts = 6
        );
        // the axle's bracket
        cube!(
            format!("bracket{i}"),
            (0.012, 0.06, 0.05),
            at = (s * 0.1, 0.35, 0.18),
            material = M.metal,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    // the wheel: a treaded tyre, a dished steel rim on six spokes, a hub on its axle
    let wheel = (0, 0.36, 0.165);
    torus!(
        "tyre",
        0.13,
        0.035,
        at = wheel,
        material = M.metal_dark,
        rot = (0, 90, 0),
        major_segments = 20,
        minor_segments = 6
    );
    for k in 0..16 {
        let a = TAU * (k as f64 + 0.5) / 16.0;
        cube!(
            format!("tread{k}"),
            (0.055, 0.014, 0.008),
            at = (0, 0.36 + 0.161 * cos(a), 0.165 + 0.161 * sin(a)),
            material = M.metal_dark,
            rot = (degrees(a) - 90.0, 0, 0)
        );
    }
    torus!(
        "rim",
        0.1,
        0.012,
        at = wheel,
        material = M.metal,
        rot = (0, 90, 0),
        major_segments = 20,
        minor_segments = 4
    );
    for k in 0..6 {
        let a = TAU * k as f64 / 6.0;
        sweep!(
            format!("spoke{k}"),
            &[
                (0.0, 0.36 + 0.03 * cos(a), 0.165 + 0.03 * sin(a)),
                (0.0, 0.36 + 0.095 * cos(a), 0.165 + 0.095 * sin(a))
            ],
            0.006,
            material = M.metal,
            verts = 5
        );
    }
    cylinder!(
        "hub",
        0.035,
        0.08,
        at = wheel,
        material = M.metal,
        rot = (0, 90, 0),
        verts = 12,
        bevel = 0.006,
        bevel_segments = 1
    );
    cylinder!(
        "axle",
        0.012,
        0.22,
        at = wheel,
        material = M.metal,
        rot = (0, 90, 0),
        verts = 8
    );
    // in the earth: leafy rosettes, flowers over them on short stems, a few leaves trailing over the rim
    let cols = [M.flower_pink, M.flower_yellow, M.flower_white];
    for (i, (x, y, r)) in [
        (-0.11, -0.17, 0.09),
        (0.11, -0.1, 0.085),
        (-0.08, 0.08, 0.09),
        (0.1, 0.15, 0.08),
        (0.0, -0.02, 0.08),
        (-0.14, 0.22, 0.07),
        (0.15, 0.27, 0.06),
    ]
    .into_iter()
    .enumerate()
    {
        lump!(
            format!("clump{i}"),
            r,
            (x, y, 0.6),
            M.leaf,
            scale = (1.1, 1.0, 0.75),
            seed = i as f64 * 1.9,
            noise = 0.35
        );
    }
    for (i, (x, y)) in [
        (-0.12, -0.16),
        (0.12, -0.04),
        (-0.07, 0.12),
        (0.1, 0.18),
        (-0.16, 0.0),
        (0.03, -0.22),
        (0.15, -0.18),
        (0.0, 0.03),
    ]
    .into_iter()
    .enumerate()
    {
        for k in 0..3 {
            let (tilt, length) = (rng.uniform(30.0, 50.0), rng.uniform(0.07, 0.1));
            leaf!(
                format!("leaf{i}_{k}"),
                (x, y, 0.64),
                (120 * k + i * 17) as f64,
                tilt,
                length,
                0.045,
                M.leaf,
                shape = "oval",
                droop = 0.3,
                thick = 0.006,
                fold = 0.2,
                rows = 3,
                ring = 3
            );
        }
        let h = rng.uniform(0.1, 0.15);
        let tilt = rng.uniform(0.0, 25.0);
        star!(
            format!("flower{i}"),
            (x + 0.01, y, 0.62 + h),
            0.05,
            5 + i % 2,
            cols[i % 3],
            yaw = (i * 40) as f64,
            tilt = tilt,
            inner = 0.55,
            cup = 0.25
        );
        dome!(
            format!("flower_eye{i}"),
            (x + 0.01, y, 0.62 + h),
            0.011,
            0.007,
            if i % 3 != 1 {
                M.flower_yellow
            } else {
                M.wood_dark
            }
        );
        sweep!(
            format!("stem{i}"),
            &[(x, y, 0.61), (x + 0.01, y, 0.62 + h)],
            0.004,
            material = M.leaf,
            verts = 4
        );
    }
    for (i, (x, y, yaw)) in [(-0.22, -0.2, 260.0), (0.22, 0.1, 95.0), (0.1, -0.3, 180.0)]
        .into_iter()
        .enumerate()
    {
        leaf!(
            format!("trail{i}"),
            (x, y, 0.62),
            yaw,
            25.0,
            0.14,
            0.05,
            M.leaf,
            shape = "lance",
            droop = 1.0,
            thick = 0.006,
            fold = 0.2,
            rows = 5,
            ring = 3
        );
    }
    settle(&[join_all("wheelbarrow")])
}

/// Three weathered garden stones, broader than tall, in three greys: rough, faces split off flat, their feet cut
/// flat where they sink into the lawn; moss in patches on the shaded tops and in the crack between the two big
/// ones, pale lichen on the faces, tufts of long grass round them and a few pebbles.
pub fn boulder() -> Vec<Obj> {
    // The two smaller stones huddle on one side of the big one (front and back of it, they read as a head and a
    // tail).
    _stone(
        "boulder_main",
        0.34,
        (-0.06, 0.02, 0.0),
        M.rock,
        (1.12, 0.94, 0.86),
        0.4,
        0.43,
        3,
        &[
            ((0.1, -0.05, 1.0), 0.07),
            ((0.35, -0.2, 1.0), 0.05),
            ((-0.9, 0.3, 0.35), 0.06),
            ((0.2, 1.0, 0.1), 0.05),
            ((0.6, -0.8, 0.25), 0.05),
            ((-0.5, -0.9, 0.1), 0.04),
        ],
        0.0,
        (0.3, 0.1),
        0.0,
    );
    _stone(
        "boulder_side",
        0.21,
        (0.3, 0.17, 0.0),
        M.rock_dark,
        (1.0, 0.92, 0.8),
        2.1,
        0.24,
        3,
        &[((0.5, -0.6, 1.0), 0.03), ((1.0, 0.2, 0.3), 0.04)],
        0.0,
        (0.3, 0.1),
        0.0,
    );
    _stone(
        "boulder_small",
        0.13,
        (0.26, -0.27, 0.0),
        M.rock_warm,
        (1.05, 0.85, 0.75),
        4.2,
        0.12,
        2,
        &[((0.3, -0.4, 1.0), 0.02)],
        0.0,
        (0.3, 0.1),
        0.0,
    );
    let moss = _on_top(&[(-0.14, 0.12), (0.02, 0.18), (0.32, 0.24)], 0.03);
    let lichen = _on_top(&[(0.08, -0.08), (-0.28, -0.1), (0.26, -0.27)], 0.012);
    for (i, &at) in moss.iter().enumerate() {
        boulder!(
            format!("moss{i}"),
            [0.1, 0.07, 0.06][i],
            at = at,
            material = M.moss,
            scale = (1.3, 1.0, 0.45),
            seed = 7.3 + i as f64,
            noise = 0.45,
            subdivisions = 2
        );
    }
    for (i, &at) in lichen.iter().enumerate() {
        boulder!(
            format!("lichen{i}"),
            [0.06, 0.05, 0.035][i],
            at = at,
            material = M.lichen,
            scale = (1.3, 1.0, 0.25),
            seed = 3.1 + i as f64,
            noise = 0.35,
            subdivisions = 2
        );
    }
    boulder!(
        "moss_crack",
        0.08,
        at = (0.19, 0.06, 0),
        material = M.moss,
        scale = (0.8, 1.3, 0.8),
        seed = 9.1,
        noise = 0.4,
        subdivisions = 2
    );
    // tufts of long grass where the stones meet the lawn: a low clump, blades fanning out of it
    let mut rng = Random::new(17);
    for (i, (x, y)) in [
        (0.45, 0.0),
        (-0.41, 0.06),
        (0.0, 0.37),
        (-0.14, -0.33),
        (0.06, -0.4),
    ]
    .into_iter()
    .enumerate()
    {
        boulder!(
            format!("tuft{i}"),
            0.055,
            at = (x, y, 0),
            material = M.leaf,
            scale = (
                if i % 2 != 0 { 1.4 } else { 0.9 },
                if i % 2 != 0 { 0.9 } else { 1.4 },
                0.5
            ),
            seed = i as f64 * 2.3,
            noise = 0.45,
            subdivisions = 1
        );
        for k in 0..9 {
            let base = (
                x + rng.uniform(-0.03, 0.03),
                y + rng.uniform(-0.03, 0.03),
                0.0,
            );
            let (yaw, tilt, length, droop) = (
                rng.uniform(0.0, 360.0),
                rng.uniform(50.0, 80.0),
                rng.uniform(0.12, 0.2),
                rng.uniform(0.15, 0.4),
            );
            leaf!(
                format!("blade{i}_{k}"),
                base,
                yaw,
                tilt,
                length,
                0.016,
                M.leaf,
                shape = "strap",
                droop = droop,
                thick = 0.004,
                fold = 0.3,
                rows = 3,
                ring = 3
            );
        }
    }
    // a few loose pebbles at the big stone's foot
    for (i, (x, y)) in [(-0.3, -0.24), (-0.22, -0.31), (0.38, -0.16), (-0.42, -0.08)]
        .into_iter()
        .enumerate()
    {
        boulder!(
            format!("pebble{i}"),
            0.022 + 0.006 * (i % 2) as f64,
            at = (x, y, 0),
            material = if i % 2 != 0 { M.rock_warm } else { M.rock_dark },
            scale = (1.3, 1.0, 0.6),
            seed = 5.5 + i as f64,
            noise = 0.3,
            subdivisions = 1
        );
    }
    settle(&[join_all("boulder")])
}
