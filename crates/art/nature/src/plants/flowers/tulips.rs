use artkit::botany::{fit_to, palette};
use artkit::math::{V3, v3};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{cut_leaf, mound, outline, spiral, stalk, tulip_head};

/// Tulips: a soil mound of tulips in five colours, closed cups of petals on straight, slightly bowed stems, broad
/// grey-green leaves at their feet.
pub fn tulips() -> Vec<Obj> {
    let p = palette(&[("leaf", "#5E8A5A"), ("leaf_light", "#6E9A66")]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(51);
    let top = mound!(&p);
    let cols = [
        m("bloom"),
        m("bloom2"),
        m("bloom3"),
        m("bloom4"),
        m("bloom"),
        m("bloom5"),
    ];
    for (i, (x, y, a)) in spiral!(17, 0.34, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.42, 0.55);
        let tip = (
            x + rng.uniform(-0.03, 0.03),
            y + rng.uniform(-0.03, 0.03),
            h,
        );
        stalk!(
            format!("stem{i}"),
            (x, y, top - 0.01),
            tip,
            m("stem"),
            r = 0.005
        );
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(0.0, 10.0));
        tulip_head!(
            format!("tulip{i}"),
            V3::from(tip) - v3(0, 0, 0.01),
            0.032,
            0.075,
            cols[i % cols.len()],
            yaw = yaw,
            tilt = tilt
        );
        for k in 0..2 {
            let (yaw, t, length) = (
                a + 40.0 + k as f64 * 160.0,
                rng.uniform(55.0, 70.0),
                rng.uniform(0.2, 0.26),
            );
            if k != 0 && i % 2 != 0 {
                continue;
            }
            cut_leaf!(
                format!("leaf{i}_{k}"),
                (x, y, top - 0.005),
                yaw,
                t,
                length,
                &outline!("lance", 0.075, n = 4),
                [m("leaf"), m("leaf_light")][k],
                droop = 0.32,
                fold = 0.35,
                ribs = 3,
                thick = 0.005
            );
        }
    }
    fit_to(&meshes(), 0.84, 0.622, 0.843);
    settle(&[join_all("tulips")])
}
