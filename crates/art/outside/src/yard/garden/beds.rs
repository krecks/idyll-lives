//! The raised bed: its soil, its plants and the lettuce in them.

use super::*;

const BED_TOP: f64 = 0.32; // the soil's top
// The growth stages (`grow`, stages, as the plants' edibles): ripe at rest; picked, the plants pulled and the
// crop gone; growing, the plants lower, no crop; planted, seedlings. The plants flatten and sink into the soil
// (shares of their height), so each seedling comes up where its plant stands.
const PLANTS: [(f64, f64, f64); 3] = [(1.0, 1.0, 0.3), (1.0, 1.0, 0.75), (1.0, 1.0, 0.4)]; // picked, growing, planted
const SINK: (f64, f64, f64) = (0.32, 0.03, 0.1);
const CROP: f64 = 0.001;

/// A lettuce standing on height `y`: its outer leaves ruffled round a cupped heart, lofted in one.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn lettuce(name: &str, x: f64, y: f64, z: f64, r: f64, h: f64, material: Mat, seed: f64) -> Obj {
    let b = F.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let n = low(14, 6);
    let rows = [
        (0.0, 0.6, 0.05),
        (0.12, 1.0, 0.16),
        (0.38, 0.92, 0.12),
        (0.66, 0.62, 0.07),
        (0.8, 0.32, 0.02),
        (0.7, 0.0, 0.0),
    ];
    let mut secs = Vec::new();
    for (t, k, wave) in rows {
        let mut lp = Vec::new();
        for i in 0..n {
            let a = TAU * i as f64 / n as f64;
            let w = 1.0 + wave * sin(5.0 * a + seed);
            // (as the script has it: its rings' depth is the soil's height and their height the lettuce's depth)
            lp.push(P3 {
                x: bx + r * k * w * cos(a),
                y: bz + r * k * w * sin(a),
                z: by + h * t + h * wave * 0.4 * cos(3.0 * a + seed),
            });
        }
        secs.push(lp);
    }
    loft!(name, &secs, material = material, angle = 70)
}

/// A carrot's frond from game (x, y, z): a feathery leaf `h` tall, leaning out along `a` degrees.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn frond(name: &str, x: f64, y: f64, z: f64, a: f64, h: f64, material: Mat, lean: f64) -> Obj {
    let mut out = vec![(0.0, 0.0)];
    let steps = low(3, 1);
    for i in 1..=steps {
        let v = h * i as f64 / steps as f64;
        let w = 0.016 * sin(PI * i as f64 / (steps + 1) as f64);
        out.push((w, v - h / steps as f64 * 0.5));
        out.push((w * 0.4, v));
    }
    let back: Vec<(f64, f64)> = out[1..].iter().rev().map(|&(u, v)| (-u, v)).collect();
    out.extend(back);
    let leaf = fextrude!(F, name, &out, 0.003, (x, y, z), material);
    let m = M4::translation(V3::from(F.at(x, y, z)))
        * M4::rotation(radians(a - 90.0), Axis::Z)
        * M4::rotation(lean, Axis::X);
    leaf.set_matrix_world(m)
}

pub fn raised_bed() -> Vec<Obj> {
    let mut rnd = Random::new(7);
    let still = new_parts(|| {
        for z in [-0.4, 0.4] {
            for y in [0.085, 0.255] {
                fbox!(
                    F,
                    numbered("board"),
                    1.84,
                    0.162,
                    0.06,
                    (0, y, z),
                    M.bed,
                    bevel = low(0.01, 0.0),
                    bevel_segments = 1
                );
            }
        }
        for x in [-0.89, 0.89] {
            for y in [0.085, 0.255] {
                fbox!(
                    F,
                    numbered("board"),
                    0.06,
                    0.162,
                    0.74,
                    (x, y, 0),
                    M.bed,
                    bevel = low(0.01, 0.0),
                    bevel_segments = 1
                );
            }
            for z in [-0.395, 0.395] {
                fbox!(
                    F,
                    numbered("post"),
                    0.09,
                    0.36,
                    0.09,
                    (x * 0.995, 0.18, z),
                    M.bed_post,
                    bevel = low(0.01, 0.0),
                    bevel_segments = 1
                );
                if !is_low() {
                    // a pyramid cap on each post
                    cone!(
                        numbered("post_cap"),
                        0.064,
                        0.0,
                        0.024,
                        at = F.at(x * 0.995, 0.372, z),
                        material = M.bed_post,
                        verts = 4,
                        rot = (0, 0, 45)
                    );
                }
            }
        }
        ground(
            "soil",
            1.74,
            0.76,
            BED_TOP - 0.005,
            M.soil,
            low((14, 6), (2, 1)),
            0.012,
            5.0,
            (0.0, 0.0),
            0.0,
        );
        fbox!(
            F,
            "label",
            0.07,
            0.05,
            0.006,
            (-0.8, BED_TOP + 0.1, 0.34),
            M.trim,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            F,
            "label_stick",
            0.012,
            0.12,
            0.004,
            (-0.8, BED_TOP + 0.02, 0.34),
            M.trim
        );
        for x in [-0.7, -0.14, 0.42] {
            // the tomatoes' canes, one up each bush (`alt` below)
            rod(
                &numbered("cane"),
                (x + 0.14, BED_TOP - 0.02, 0.22),
                (x + 0.15, BED_TOP + 0.2, 0.22),
                0.005,
                M.haft,
                4,
            );
        }
    });
    if !is_low() {
        join(&still, "bed", Some(P3::from((0, 0, 0))));
    }
    let xs: Vec<f64> = (0..6).map(|i| -0.7 + i as f64 * 0.28).collect();
    let jit: Vec<(f64, f64)> = (0..18)
        .map(|_| (rnd.uniform(-0.03, 0.03), rnd.uniform(-0.03, 0.03)))
        .collect();
    let alt: Vec<f64> = xs.iter().step_by(2).copied().collect();
    let plants = new_parts(|| {
        for (i, &x) in xs.iter().enumerate() {
            // lettuces, at the back
            let (dx, dz) = jit[i];
            lettuce(
                &numbered("lettuce"),
                x + dx,
                BED_TOP - 0.01,
                -0.24 + dz,
                0.11,
                0.15,
                [M.lettuce, M.lettuce_dark][i % 2],
                i as f64 * 1.7,
            );
        }
        for (i, &x) in xs.iter().enumerate() {
            // carrot tops, in the middle
            let (dx, dz) = jit[6 + i];
            for k in 0..low(4, 2) {
                frond(
                    &numbered("frond"),
                    x + dx,
                    BED_TOP,
                    dz,
                    (k * 90 + i * 25) as f64,
                    0.17 + 0.02 * (k % 2) as f64,
                    M.carrot_top,
                    0.22,
                );
            }
        }
        for (i, &x) in alt.iter().enumerate() {
            // bushy tomato plants up their canes, at the front
            let (dx, dz) = jit[12 + i];
            for k in 0..3 {
                let a = (k * 120 + i * 40) as f64;
                blob(
                    x + 0.14 + dx + 0.06 * cosd(a),
                    BED_TOP - 0.01 + k as f64 * 0.04,
                    0.22 + dz + 0.06 * sind(a),
                    0.09 - 0.012 * k as f64,
                    M.leaf,
                    (20 + i * 3 + k) as f64,
                    (1.0, 1.0, 0.9),
                    0.45,
                );
            }
        }
    });
    let crop = new_parts(|| {
        for (i, &x) in xs.iter().enumerate() {
            // carrot shoulders
            let (dx, dz) = jit[6 + i];
            fcone!(
                F,
                numbered("carrot"),
                0.028,
                0.02,
                0.035,
                (x + dx, BED_TOP + 0.012, dz),
                M.carrot,
                verts = low(8, 5)
            );
        }
        for (i, &x) in alt.iter().enumerate() {
            // tomatoes on the bushes, each with its green calyx
            let (dx, dz) = jit[12 + i];
            for k in 0..4 {
                let a = (k * 90 + 20 + i * 30) as f64;
                let p = (
                    x + 0.14 + dx + 0.12 * cosd(a),
                    BED_TOP + 0.08 + 0.04 * (k % 2) as f64,
                    0.22 + dz + 0.12 * sind(a),
                );
                fsphere!(
                    F,
                    numbered("tomato"),
                    0.035,
                    p,
                    M.tomato,
                    scale = (1, 1, 0.85),
                    segments = low(8, 6),
                    rings = low(5, 4)
                );
                if !is_low() {
                    fcone!(
                        F,
                        numbered("calyx"),
                        0.016,
                        0.004,
                        0.012,
                        (p.0, p.1 + 0.03, p.2),
                        M.green,
                        verts = 5
                    );
                }
            }
        }
    });
    if is_low() {
        // a far version: all one still part, ripe
        return settle!(&[join_all("raisedBed")], centre = false);
    }
    let origin = F.at(0, BED_TOP, 0);
    let plants = join(&plants, "plants", Some(origin));
    let crop = join(&crop, "crop", Some(origin));
    let objs = settle!(centre = false);
    let height = measure(&[plants]).1[2] - plants.location().z as f64; // the plants' top over the soil
    let sink = (SINK.0 * height, SINK.1 * height, SINK.2 * height);
    stages(
        plants,
        "grow",
        PLANTS[0],
        PLANTS[1],
        PLANTS[2],
        Stages {
            sink: Some(sink),
            ..Stages::default()
        },
    );
    stages(crop, "grow", CROP, CROP, CROP, Stages::default());
    squash(
        crop,
        "harvest",
        0.001,
        Squash {
            seconds: 1.0,
            ..Squash::default()
        },
    ); // the ripe crop picked away (and back after, if not picked)
    // Tending: the plants and their crop rustle, 3° back and forth about the soil line through the bed's middle, on
    // top of the growth stage (objectMotion.ts plays a use clip after it).
    for part in [plants, crop] {
        rock(
            part,
            "rustle",
            3.0,
            Rock {
                seconds: 0.8,
                axis: Axis::X,
            },
        );
    }
    objs
}
