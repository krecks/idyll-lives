use artkit::botany::{fit_to, palette, polar};
use artkit::far::{is_low, low};
use artkit::math::{V3, v3};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{_put, cupped, cut_leaf, florets, mound, outline, rays, twig};

/// Sunflowers: four tall, hairy stalks with big, drooping heart-shaped leaves, their heavy heads turned to the
/// front: two rows of golden petals round a domed brown disc of seeds, green behind.
pub fn sunflowers() -> Vec<Obj> {
    let p = super::super::leafy_petals(
        palette(&[
            ("bloom", "#F5C518"),
            ("bloom2", "#E8A812"),
            ("centre_dark", "#5A3A1E"),
            ("stem", "#5E8C3A"),
        ]),
        &["bloom", "bloom2"],
    );
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(54);
    let top = mound!(&p);
    for (i, ((x, y), h)) in [(0.18, 0.12), (-0.17, 0.06), (0.0, -0.18), (0.04, 0.3)]
        .into_iter()
        .zip([1.85, 1.6, 1.4, 1.95])
        .enumerate()
    {
        let tip = v3(x * 1.2, y * 1.2 - 0.04, h);
        twig!(
            format!("stalk{i}"),
            &[
                v3(x, y, top - 0.01),
                v3(x, y, 0).lerp(tip, 0.5) + v3(0.01, 0, 0),
                tip
            ],
            0.02,
            0.013,
            m("stem"),
            verts = 6,
            res = 2
        );
        for k in 0..4 {
            let t = 0.25 + k as f64 * 0.16;
            let base = v3(x, y, 0).lerp(tip, t);
            let yaw = rng.uniform(0.0, 360.0) + (k * 120) as f64;
            let end = base + V3::from(polar(0.05, yaw, 0.02));
            twig!(
                format!("petiole{i}_{k}"),
                &[base, end],
                0.006,
                0.004,
                m("stem"),
                verts = 3,
                res = 1
            );
            cut_leaf!(
                format!("leaf{i}_{k}"),
                end,
                yaw,
                25.0,
                0.22 - k as f64 * 0.015,
                &outline!("heart", 0.18 - k as f64 * 0.012, n = 6, teeth = 0.08),
                [m("leaf"), m("leaf_dark"), m("leaf_light")][k % 3],
                droop = 0.42,
                fold = 0.18,
                ribs = 4
            );
        }
        let yaw = 180.0 + rng.uniform(-35.0, 35.0);
        rays!(
            format!("petals{i}"),
            tip,
            0.17,
            18,
            m("bloom"),
            yaw = yaw,
            tilt = 62,
            inner = 0.5,
            cup = 0.05,
            droop = 0.06,
            width = 0.7,
            thick = 0.012
        );
        if !is_low() {
            rays!(
                format!("petals_b{i}"),
                tip - V3::from(polar(0.006, yaw, 0.0)),
                0.15,
                18,
                m("bloom2"),
                yaw = yaw + 10.0,
                tilt = 62,
                inner = 0.5,
                cup = 0.08,
                width = 0.75,
                thick = 0.01
            );
        }
        let disc = florets!(
            format!("seeds{i}"),
            0.088,
            (0, 0, -0.012),
            m("centre_dark"),
            bumps = 6.0,
            amount = 0.05,
            flat = 0.28,
            segments = low(14, 8),
            rings = low(6, 4),
            seed = i as f64
        );
        _put!(disc, tip, yaw = yaw, tilt = 62);
        cupped!(
            format!("back{i}"),
            &[
                (0.0, -0.03),
                (0.04, -0.028),
                (0.08, -0.012),
                (0.09, 0.0),
                (0.0, 0.0)
            ],
            tip,
            m("stem"),
            yaw = yaw,
            tilt = 62,
            segments = low(10, 5)
        );
    }
    artkit::uv::box_named(&["stalk", "petiole"]);
    fit_to(&meshes(), 0.84, 2.108, 0.907);
    settle(&[join_all("sunflowers")])
}
