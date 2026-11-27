use artkit::botany::{fit_to, palette};
use artkit::far::{is_low, low};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{cupped, cut_leaf, mound, outline, rays, spiral, stalk};

/// Daffodils: pale six-petalled flowers with ruffled orange trumpets, nodding outwards on bowed stems, and
/// blue-green strap leaves.
pub fn daffodils() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#FAE68A"),
        ("centre", "#F6AE1C"),
        ("stem", "#5E8C3A"),
        ("leaf", "#5E8A5A"),
        ("leaf_light", "#6E9A66"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(52);
    let top = mound!(&p);
    for (i, (x, y, a)) in spiral!(13, 0.34, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.34, 0.46);
        let tip = (x, y, h);
        let yaw = a + rng.uniform(-40.0, 40.0);
        stalk!(
            format!("stem{i}"),
            (x, y, top - 0.01),
            tip,
            m("stem"),
            r = 0.004
        );
        rays!(
            format!("petals{i}"),
            tip,
            0.05,
            6,
            m("bloom"),
            yaw = yaw,
            tilt = 70,
            inner = 0.3,
            cup = 0.12,
            width = 0.8,
            shoulder = 0.0
        );
        cupped!(
            format!("trumpet{i}"),
            &[
                (0.0, 0.0),
                (0.014, 0.0),
                (0.018, 0.035),
                (0.026, 0.05),
                (0.02, 0.045)
            ],
            tip,
            m("centre"),
            yaw = yaw,
            tilt = 70,
            segments = low(6, 3)
        );
        for k in 0..2 {
            let (yaw_k, t, length) = (
                yaw + 150.0 + k as f64 * 60.0,
                rng.uniform(70.0, 82.0),
                rng.uniform(0.26, 0.32),
            );
            if is_low() && k != 0 {
                continue;
            }
            cut_leaf!(
                format!("leaf{i}_{k}"),
                (x, y, top),
                yaw_k,
                t,
                length,
                &outline!("strap", 0.032, n = 3),
                [m("leaf"), m("leaf_light")][(i + k) % 2],
                droop = 0.3,
                fold = 0.3,
                ribs = 2,
                thick = 0.005
            );
        }
    }
    fit_to(&meshes(), 0.84, 0.491, 0.84);
    settle(&[join_all("daffodils")])
}
