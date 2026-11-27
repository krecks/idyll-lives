use artkit::botany::{fit_to, palette, sword};
use artkit::far::low;
use artkit::math::{TAU, cos, sin};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{cut_leaf, leaf, lump, outline, revolve, soil_top};

/// Succulent bowl: a shallow glazed bowl with a rolled rim, grit and pebbles, five fat-leaved rosettes in
/// blue-green and pink, and a spiky upright one.
pub fn succulents() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#7FA6B8"),
        ("flesh", "#5E8F55"),
        ("flesh_light", "#8FB5A0"),
        ("flesh_pink", "#C99AA8"),
        ("sand", "#C8B894"),
    ]);
    revolve!(
        "bowl",
        &[
            (0.0, 0.0),
            (0.17, 0.0),
            (0.19, 0.008),
            (0.27, 0.085),
            (0.3, 0.11),
            (0.312, 0.122),
            // (its rim rolled round: no turn of 60°)
            (0.311, 0.127),
            (0.306, 0.13),
            (0.292, 0.128),
            (0.276, 0.104),
            (0.0, 0.094)
        ],
        material = p.get("body"),
        segments = low(20, 12)
    );
    soil_top!(
        &p,
        0.278,
        0.104,
        segments = low(20, 10),
        mound = 0.01,
        seed = 2.0,
        material = p.get("sand")
    );
    let mut rng = Random::new(95);
    // pebbles
    for i in 0..low(7, 0) {
        let (a, rr) = (rng.uniform(0.0, TAU), rng.uniform(0.08, 0.24));
        lump!(
            format!("pebble{i}"),
            0.012,
            (rr * cos(a), rr * sin(a), 0.104),
            p.get("body"),
            scale = (1.4, 1, 0.6),
            seed = i as f64,
            noise = 0.2,
            segments = 5,
            rings = 3
        );
    }
    let cols = [
        p.get("flesh_light"),
        p.get("flesh_pink"),
        p.get("flesh"),
        p.get("flesh_light"),
        p.get("flesh_pink"),
    ];
    let mut spots = vec![(0.0, 0.0)];
    spots.extend((0..6).map(|i| {
        let a = i as f64 * TAU / 6.0 + 0.3;
        (0.17 * sin(a), 0.17 * cos(a))
    }));
    for (n, (x, y)) in spots.into_iter().enumerate() {
        let s = rng.uniform(0.85, 1.15) * if n == 0 { 1.15 } else { 1.0 };
        if n == 3 || n == 6 {
            // spiky ones: narrow upright leaves
            for i in 0..6 {
                let yaw = (i * 60 + n * 13) as f64;
                cut_leaf!(
                    format!("spike{n}_{i}"),
                    (x, y, 0.1),
                    yaw,
                    (62 + (i % 2) * 14) as f64,
                    0.1 * s,
                    &outline!(sword, 0.03 * s, n = 3),
                    p.get("flesh"),
                    fold = 0.3,
                    ribs = 2,
                    thick = 0.012
                );
            }
            continue;
        }
        for (tier, (k, length, tilt)) in [(6, 0.085, 24.0), (4, 0.064, 46.0), (3, 0.04, 70.0)]
            .into_iter()
            .enumerate()
        {
            for i in 0..k {
                let yaw = (i * 360) as f64 / k as f64 + (tier * 30 + n * 13) as f64;
                leaf!(
                    format!("r{n}_{tier}_{i}"),
                    (x, y, 0.1 + tier as f64 * 0.01),
                    yaw,
                    tilt,
                    length * s,
                    length * 0.62 * s,
                    cols[n % cols.len()],
                    shape = "oval",
                    thick = 0.024 * s,
                    fold = 0.2,
                    rows = 2,
                    ring = 4,
                    droop = -0.1
                );
            }
        }
    }
    fit_to(&meshes(), 0.62, 0.17, 0.62);
    settle(&[join_all("succulents")])
}
