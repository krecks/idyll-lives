use artkit::botany::{fit_to, palette};
use artkit::far::is_low;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{_put, cut_leaf, mound, pinnate, rays, sphere, spiral, stalk};

/// Cosmos: feathery leaves cut into threads and tall thin stems of pink, white and magenta flowers, eight broad
/// petals round a yellow eye.
pub fn cosmos() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#E77FB0"),
        ("bloom2", "#F6EEF2"),
        ("bloom3", "#C2367A"),
        ("bloom4", "#F2A6CA"),
        ("centre", "#F2C12E"),
        ("stem", "#6E9E44"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(61);
    let top = mound!(&p);
    for (i, (x, y, _)) in spiral!(9, 0.3, &mut rng).into_iter().enumerate() {
        let (yaw, t, length) = (
            rng.uniform(0.0, 360.0),
            rng.uniform(58.0, 78.0),
            rng.uniform(0.28, 0.38),
        );
        cut_leaf!(
            format!("frond{i}"),
            (x, y, top),
            yaw,
            t,
            length,
            &pinnate!(5, 0.16, "lance", lean = 0.8, fill = 0.25),
            [m("leaf_light"), m("leaf")][i % 2],
            droop = 0.3,
            fold = 0.1,
            ribs = 3
        );
    }
    let cols = [m("bloom"), m("bloom2"), m("bloom3"), m("bloom4")];
    for (i, (x, y, _)) in spiral!(14, 0.36, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.55, 0.87);
        let tip = (x * 1.1, y * 1.1, h);
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(10.0, 40.0));
        stalk!(
            format!("stem{i}"),
            (x, y, top - 0.01),
            tip,
            m("stem"),
            r = 0.0035,
            bow = 0.04
        );
        rays!(
            format!("cosmos{i}"),
            tip,
            0.05,
            8,
            cols[i % 4],
            yaw = yaw,
            tilt = tilt,
            inner = 0.25,
            cup = 0.1,
            width = 0.9,
            shoulder = 0.0
        );
        if !is_low() {
            let eye = sphere!(
                format!("eye{i}"),
                1.0,
                material = m("centre"),
                segments = 5,
                rings = 3,
                scale = (0.012, 0.012, 0.008)
            );
            _put!(eye, tip, yaw = yaw, tilt = tilt, lift = 0.004);
        }
    }
    fit_to(&meshes(), 0.885, 0.879, 0.84);
    settle(&[join_all("cosmos")])
}
