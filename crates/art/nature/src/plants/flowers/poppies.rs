use artkit::botany::{fit_to, palette};
use artkit::far::{is_low, low};
use artkit::math::{V3, v3};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{_put, lump, mound, rays, rosettes, sphere, spiral, stalk};

/// Poppies: feathery grey-green leaves and red cups of four broad petals with dark middles on thin wavering
/// stems, a few nodding buds.
pub fn poppies() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#EC3A2E"),
        ("stem", "#6E9464"),
        ("leaf", "#5E8456"),
        ("leaf_light", "#6E9464"),
        ("centre_dark", "#1E1A1A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(55);
    let top = mound!(&p);
    rosettes!(
        &mut rng,
        6,
        0.3,
        top - 0.01,
        &[m("leaf"), m("leaf_light")],
        leaves = 3,
        length = 0.22,
        width = 0.1,
        tilt = (20.0, 40.0),
        pinnae = 3
    );
    for (i, (x, y, _)) in spiral!(16, 0.38, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.4, 0.61);
        let tip = (
            x + rng.uniform(-0.04, 0.04),
            y + rng.uniform(-0.04, 0.04),
            h,
        );
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(0.0, 30.0));
        stalk!(
            format!("stem{i}"),
            (x, y, top),
            tip,
            m("stem"),
            r = 0.0035,
            bow = 0.04
        );
        if i % 5 == 4 {
            // a bud, nodding
            lump!(
                format!("bud{i}"),
                0.014,
                V3::from(tip) - v3(0, 0, 0.03),
                m("stem"),
                scale = (1, 1, 1.4),
                segments = low(6, 4),
                rings = low(4, 3)
            );
            continue;
        }
        rays!(
            format!("poppy{i}"),
            tip,
            0.045,
            4,
            m("bloom"),
            yaw = yaw,
            tilt = tilt,
            inner = 0.2,
            cup = 0.75,
            width = 1.25,
            thick = 0.008
        );
        if !is_low() {
            let eye = sphere!(
                format!("eye{i}"),
                1.0,
                material = m("centre_dark"),
                segments = 5,
                rings = 3,
                scale = (0.011, 0.011, 0.012)
            );
            _put!(eye, tip, yaw = yaw, tilt = tilt, lift = 0.01);
        }
    }
    fit_to(&meshes(), 0.84, 0.657, 0.84);
    settle(&[join_all("poppies")])
}
