//! Italian cypress: a pencil of dark leaves about a metre across and eight tall, a lofted flame of foliage covered
//! in upright sprays, a little ragged, tapering to a point.

use artkit::botany::conifer::top;
use artkit::botany::skeleton::Stem;
use artkit::botany::species::{Seeds, Species, Stage, Tree, Trunk, grow};
use artkit::far::{is_low, low};
use artkit::math::{P3, PI, TAU, cos, p3, pow, sin, sqrt};
use artkit::noise::noise_at;
use artkit::perch;
use artkit::py;
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{find, loft, lump};

pub fn species() -> Species {
    Species {
        name: "cypress",
        seeds: Seeds {
            build: 21,
            perches: 121,
            cards: None,
        },
        leaves: &["#2E5034", "#385C3A", "#426842", "#4C744A"],
        colours: &[("bark", "#8E7A6A")],
        trunk: Stage::Spec(Trunk {
            pts: vec![(0.0, 0.0, 0.0), (0.0, 0.0, 0.4), (0.0, 0.01, 1.0)],
            r: 0.1,
            scale: vec![1.3, 1.0, 0.85],
            seed: 13.0,
            o: Stem {
                verts: 7,
                wobble: 0.01,
                ..Stem::default()
            },
        }),
        crown: Stage::Own(crown),
        perches: Stage::Own(perches),
        ..Species::default()
    }
}

pub fn cypress() -> Vec<Obj> {
    grow(&species())
}

/// How far round the column is at height `z`.
fn radius(z: f64) -> f64 {
    if z < 1.4 {
        return 0.2 + 0.2 * (z - 0.75) / 0.65;
    }
    0.4 * (1.0 - pow((z - 1.4) / 6.55, 1.6)) + 0.02
}

/// The column and the sprays round it.
fn crown(tree: &mut Tree) {
    let (rng, cols) = (&mut tree.rng, &tree.cols);
    let full = !is_low();
    // the column: a dark flame of foliage, fullest at 1.5-3 m and tapering to a point, its sides a little ragged
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
        let mut ring = Vec::new();
        for v in 0..round {
            let a = TAU * v as f64 / round as f64;
            let wob = 1.0
                + if full {
                    0.12 * noise_at(cos(a) * 2.0, sin(a) * 2.0, z * 1.3)
                } else {
                    0.0
                };
            ring.push(p3(r * wob * cos(a), r * wob * sin(a), z));
        }
        rings.push(ring);
    }
    let last = rings.len() - 1;
    rings[last] = vec![p3(0.0, 0.0, 7.95); round]; // (a point at the top)
    loft!("column", &rings, material = cols[0]);
    // upright sprays round it in a loose spiral, each a slim flame, lighter towards the top
    let ga = PI * (3.0 - sqrt(5.0));
    for i in 0..44 {
        let z = 1.0 + 6.2 * (i as f64 + 0.5) / 44.0 + rng.uniform(-0.1, 0.1);
        let a = i as f64 * ga + rng.uniform(-0.3, 0.3);
        let r = radius(z + 0.3);
        let size = 0.21_f64.min(0.08 + r * 0.38) * rng.uniform(0.85, 1.15);
        let col = cols[3.min(py::int((z - 1.0) / 6.2 * 3.0 + rng.uniform(0.0, 1.2)) as usize)];
        let seed = i as f64 * 2.1 + rng.random();
        if full || i % 3 == 0 {
            lump!(
                format!("spray{i}"),
                size * low(1.0, 1.3),
                (r * 0.82 * cos(a), r * 0.82 * sin(a), z - size * 1.2),
                col,
                scale = (1, 1, 2.6),
                noise = 0.25,
                seed = seed,
                segments = 6,
                rings = 5
            );
        }
    }
}

/// Its perches: up the column's sides, where its sprays stand upright, and its top.
fn perches(tree: &mut Tree) {
    let mut prng = Random::new(tree.species.seeds.perches);
    let mut aims = Vec::new();
    for z in [1.9, 2.7, 3.5, 4.3, 5.1, 5.9, 6.6] {
        aims.extend(perch::all_around((0.0, 0.0, z), &mut prng, 6, (0.1, 0.6)));
    }
    find!(15, &aims, gap = 0.4, extra = top(), facing = -0.1);
}
