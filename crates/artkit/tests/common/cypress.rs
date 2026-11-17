//! The Italian cypress as families/trees.py builds it (with the few helpers of trees.py and plants.py it needs),
//! line for line: the kernel's proof from end to end on a model that needs no leaf cards.
#![allow(dead_code)]

use artkit::family::{Family, Model};
use artkit::far::{is_low, low};
use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::{P3, PI, TAU, V3, cos, p3, pow, sin, v3};
use artkit::noise::noise_at;
use artkit::parts::{join_all, settle};
use artkit::perch;
use artkit::py;
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{find, jitter, loft, palette, sphere, sweep};

palette! { M: bark = ("#8E7A6A", bark), leaf = ("#2E5034", foliage), leaf2 = ("#385C3A", foliage), leaf3 = ("#426842", foliage), leaf4 = ("#4C744A", foliage) }

/// The finishes of the base manifest the cypress wears.
pub fn finishes() -> Finishes {
    let f = |roughness, textured| Finish {
        roughness,
        metallic: 0.0,
        textured,
    };
    [("bark", f(0.6, true)), ("foliage", f(0.9, true))]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}

/// The trunk in one swept, tapering piece through Blender `pts`, its bark wobbled by noise sideways (not on a far
/// version: its foot stays where the model's is).
fn trunk(
    pts: &[(f64, f64, f64)],
    r: f64,
    scale: &[f64],
    seed: f64,
    verts: usize,
    wobble: f64,
) -> Obj {
    let stem = sweep!(
        "trunk",
        pts,
        r,
        material = M.bark,
        verts = low(verts, 5),
        resolution = low(3, 1),
        scale = scale
    );
    if !is_low() && wobble != 0.0 {
        jitter!(stem, wobble, size = 0.45, seed = seed, up = 0.0);
    }
    stem
}

/// A soft, lumpy ball (a leafy clump): a smooth sphere pushed about by noise, its bottom at `at`.
#[allow(clippy::too_many_arguments)]
pub fn lump(
    name: &str,
    r: f64,
    at: (f64, f64, f64),
    material: Mat,
    scale: (f64, f64, f64),
    seed: f64,
    noise: f64,
    segments: usize,
    rings: usize,
) -> Obj {
    let segments = low(segments, 6.max(py::round(segments as f64 * 0.6) as usize));
    let rings = low(rings, 3.max(py::round(rings as f64 * 0.6) as usize));
    let obj = sphere!(
        name,
        r,
        material = material,
        segments = segments,
        rings = rings,
        scale = scale
    );
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            let p = *v;
            let n = noise_at(
                p.x as f64 / r * 1.3 + seed,
                p.y as f64 / r * 1.3 - seed,
                p.z as f64 / r * 1.3 + seed * 0.5,
            );
            *v = p * (1.0 + noise * n);
        }
        let bottom = o
            .mesh
            .verts
            .iter()
            .map(|v| v.z)
            .fold(f32::INFINITY, f32::min);
        o.mesh.translate(V3::from(at) - v3(0, 0, bottom));
    });
    obj
}

/// Italian cypress: a pencil of dark leaves about a metre across and eight tall, a lofted flame of foliage covered
/// in upright sprays, a little ragged, tapering to a point.
pub fn cypress() -> Vec<Obj> {
    let mut rng = Random::new(21);
    let cols = [M.leaf, M.leaf2, M.leaf3, M.leaf4];
    let full = !is_low();
    trunk(
        &[(0.0, 0.0, 0.0), (0.0, 0.0, 0.4), (0.0, 0.01, 1.0)],
        0.1,
        &[1.3, 1.0, 0.85],
        13.0,
        7,
        0.01,
    );
    // the column: a dark flame of foliage, fullest at 1.5-3 m and tapering to a point, its sides a little ragged
    let radius = |z: f64| {
        if z < 1.4 {
            0.2 + 0.2 * (z - 0.75) / 0.65
        } else {
            0.4 * (1.0 - pow((z - 1.4) / 6.55, 1.6)) + 0.02
        }
    };
    let zs: Vec<f64> = if full {
        (0..14)
            .map(|k| 0.75 + (7.95 - 0.75) * pow(k as f64 / 13.0, 0.9))
            .collect()
    } else {
        (0..8).map(|k| 0.75 + 7.2 * k as f64 / 7.0).collect()
    };
    let round = low(10, 6);
    let mut rings: Vec<Vec<P3>> = Vec::new();
    for &z in &zs {
        let r = radius(z);
        rings.push(
            (0..round)
                .map(|v| {
                    let a = TAU * v as f64 / round as f64;
                    let wob = 1.0
                        + if full {
                            0.12 * noise_at(cos(a) * 2.0, sin(a) * 2.0, z * 1.3)
                        } else {
                            0.0
                        };
                    p3(r * wob * cos(a), r * wob * sin(a), z)
                })
                .collect(),
        );
    }
    let last = rings.len() - 1;
    rings[last] = vec![p3(0.0, 0.0, 7.95); round]; // (a point at the top)
    loft!("column", &rings, material = cols[0]);
    // upright sprays round it in a loose spiral, each a slim flame, lighter towards the top
    let ga = PI * (3.0 - 5.0_f64.sqrt());
    for i in 0..44 {
        let z = 1.0 + 6.2 * (i as f64 + 0.5) / 44.0 + rng.uniform(-0.1, 0.1);
        let a = i as f64 * ga + rng.uniform(-0.3, 0.3);
        let r = radius(z + 0.3);
        let size = 0.21_f64.min(0.08 + r * 0.38) * rng.uniform(0.85, 1.15);
        let col = cols[3.min(py::int((z - 1.0) / 6.2 * 3.0 + rng.uniform(0.0, 1.2)) as usize)];
        let seed = i as f64 * 2.1 + rng.random();
        if full || i % 3 == 0 {
            lump(
                &format!("spray{i}"),
                size * low(1.0, 1.3),
                (r * 0.82 * cos(a), r * 0.82 * sin(a), z - size * 1.2),
                col,
                (1.0, 1.0, 2.6),
                seed,
                0.25,
                6,
                5,
            );
        }
    }
    let still = join_all("cypress");
    let mut prng = Random::new(121);
    let mut aims = Vec::new();
    for z in [1.9, 2.7, 3.5, 4.3, 5.1, 5.9, 6.6] {
        aims.extend(perch::all_around((0.0, 0.0, z), &mut prng, 6, (0.1, 0.6)));
    }
    // the cypress's top, from straight above (a blackbird's song post)
    let top = vec![(v3(0.02, 0.0, 6.0), v3(0, 0, 1))];
    find!(15, &aims, gap = 0.4, extra = top, facing = -0.1);
    settle(&[still])
}

/// The trees family with only the cypress in it.
pub fn family() -> Family {
    Family::new(
        "trees",
        M.palette(),
        vec![
            Model::new("cypress", cypress, (1.012, 7.95, 1.025))
                .key("model.cypress")
                .tolerance(0.1)
                .far_low()
                .habitat("conifer"),
        ],
        "the cypress alone",
    )
}
