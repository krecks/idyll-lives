use artkit::botany::palette;
use artkit::far::low;
use artkit::math::{TAU, cos, sin};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{cut_leaf, pinnate, revolve, twig, vessel};

/// Fern on a stand: a white pot on a round wooden plant stand with three splayed, tapering legs, long fronds of
/// leaflets arching over the rim and hanging down.
pub fn boston_fern() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#EDEBE6"),
        ("leaf", "#5E9A3E"),
        ("leaf_dark", "#558E38"),
        ("leaf_light", "#78B24E"),
        ("wood", "#7A5A40"),
    ]);
    revolve!(
        "stand_top",
        &[
            (0.0, 0.538),
            (0.19, 0.538),
            (0.2, 0.548),
            (0.2, 0.562),
            (0.192, 0.572),
            (0.0, 0.572)
        ],
        material = p.get("wood"),
        segments = low(20, 10)
    );
    for i in 0..3 {
        let a = i as f64 * TAU / 3.0 + 0.5;
        twig!(
            format!("stand_leg{i}"),
            &[
                (0.12 * cos(a), 0.12 * sin(a), 0.545),
                (0.185 * cos(a), 0.185 * sin(a), 0.0)
            ],
            0.02,
            0.013,
            p.get("wood"),
            verts = 6,
            res = 1
        );
    }
    let ys = vessel!(
        &p,
        0.145,
        0.24,
        z0 = 0.572,
        r_bottom = 0.11,
        lip = 0.0,
        foot = 0.015,
        segments = 16
    );
    let mut rng = Random::new(96);
    let cols = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    for i in 0..9 {
        let yaw = i as f64 * 40.0 + rng.uniform(-10.0, 10.0);
        let (tilt, length, droop) = (
            rng.uniform(40.0, 62.0),
            // (a Boston fern's fronds 0.6-0.9 m, arching over and hanging down past its pot)
            rng.uniform(0.43, 0.49) * 1.35,
            rng.uniform(0.92, 1.04) * 1.6,
        );
        cut_leaf!(
            format!("frond{i}"),
            (0, 0, ys + 0.02),
            yaw,
            tilt,
            length,
            &pinnate!(8, 0.14, "lance", lean = 0.5, fill = 0.75),
            cols[i % 3],
            droop = droop,
            fold = 0.12,
            ribs = 4
        );
    }
    for i in 0..3 {
        let yaw = (i * 120 + 30) as f64;
        cut_leaf!(
            format!("crown{i}"),
            (0, 0, ys + 0.02),
            yaw,
            68.0,
            // (the young fronds upright out of the crown)
            0.53,
            &pinnate!(6, 0.1, "lance", lean = 0.5, fill = 0.75),
            p.get("leaf_light"),
            droop = 0.5,
            fold = 0.12,
            ribs = 3
        );
    }
    settle(&[join_all("bostonFern")])
}
