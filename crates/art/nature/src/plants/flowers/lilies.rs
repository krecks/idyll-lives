use artkit::botany::{fit_to, palette};
use artkit::far::{is_low, low};
use artkit::math::v3;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{cupped, cut_leaf, lump, mound, outline, rays, spiral, stalk};

/// Lilies: six tall stems clothed in narrow leaves, each with two open lilies, six petals curling back from a
/// trumpet, and a bud.
pub fn lilies() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#F07A2A"),
        ("bloom2", "#F5EFE4"),
        // (a rose-red, as Asiatic lilies come: orange, red or yellow)
        ("bloom3", "#E8586C"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(58);
    let top = mound!(&p);
    let cols = [m("bloom"), m("bloom2"), m("bloom3")];
    for (i, (x, y, a)) in spiral!(6, 0.27, &mut rng).into_iter().enumerate() {
        // (Asiatic lilies flower at 1-1.5 m)
        let h = rng.uniform(0.71, 0.87) * 1.2;
        let tip = v3(x * 1.15, y * 1.15, h);
        stalk!(
            format!("stem{i}"),
            (x, y, top - 0.01),
            tip,
            m("stem"),
            r = 0.007,
            bow = 0.02
        );
        for k in 0..4 {
            let (t, yaw) = (0.15 + k as f64 * 0.17, rng.uniform(0.0, 360.0));
            if is_low() && k % 2 != 0 {
                // a far version: three leaves a stem
                continue;
            }
            cut_leaf!(
                format!("leaf{i}_{k}"),
                v3(x, y, 0).lerp(tip, t),
                yaw,
                35.0,
                0.17,
                &outline!("lance", 0.03, n = 3),
                m("leaf"),
                droop = 0.3,
                ribs = 2,
                thick = 0.004
            );
        }
        for k in 0..2 {
            let yaw = a + (k as f64 * 2.0 - 1.0) * 50.0 + rng.uniform(-15.0, 15.0);
            let at = tip - v3(0, 0, 0.03 * k as f64);
            rays!(
                format!("lily{i}_{k}"),
                at,
                0.075,
                6,
                cols[i % 3],
                yaw = yaw,
                tilt = 55,
                inner = 0.18,
                cup = 1.1,
                droop = 0.55,
                width = 0.7,
                shoulder = 0.4
            );
            if !is_low() {
                cupped!(
                    format!("throat{i}_{k}"),
                    &[
                        (0.0, 0.0),
                        (0.008, 0.0),
                        (0.016, 0.03),
                        (0.02, 0.04),
                        (0.0, 0.02)
                    ],
                    at,
                    cols[i % 3],
                    yaw = yaw,
                    tilt = 55,
                    segments = 4
                );
            }
        }
        lump!(
            format!("bud{i}"),
            0.012,
            tip + v3(0, 0, 0.005),
            m("stem"),
            scale = (1, 1, 2.4),
            segments = low(5, 4),
            rings = 3
        );
    }
    fit_to(&meshes(), 0.84, 1.08, 0.869);
    settle(&[join_all("lilies")])
}
