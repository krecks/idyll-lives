use artkit::botany::{fit_to, palette};
use artkit::far::is_low;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{_put, mound, rays, rosettes, sphere, spiral, stalk};

/// Daisies: low rosettes of leaves and white daisies of separate ray petals round a raised yellow eye, on thin
/// stems.
pub fn daisies() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#FFFFFF"),
        ("centre", "#F6C21E"),
        ("stem", "#5E8C3A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(53);
    let top = mound!(&p);
    rosettes!(
        &mut rng,
        5,
        0.3,
        top - 0.01,
        &[m("leaf"), m("leaf_dark"), m("leaf_light")],
        leaves = 4,
        // (a lawn daisy's spoon-shaped leaves, under its flowers)
        length = 0.12,
        width = 0.05,
        tilt = (15.0, 35.0)
    );
    for (i, (x, y, _)) in spiral!(15, 0.38, &mut rng).into_iter().enumerate() {
        // (lawn daisies, Bellis perennis: a carpet flowering 8-15 cm up, a large-flowered kind's heads)
        let h = rng.uniform(0.25, 0.36) * 0.4;
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(0.0, 30.0));
        stalk!(
            format!("stem{i}"),
            (x, y, top),
            (x, y, h),
            m("stem"),
            r = 0.004
        );
        rays!(
            format!("daisy{i}"),
            (x, y, h),
            0.03,
            10,
            m("bloom"),
            yaw = yaw,
            tilt = tilt,
            inner = 0.32,
            cup = 0.06,
            width = 0.75,
            shoulder = 0.0
        );
        if !is_low() {
            let eye = sphere!(
                format!("eye{i}"),
                1.0,
                material = m("centre"),
                segments = 5,
                rings = 3,
                scale = (0.01, 0.01, 0.006)
            );
            _put!(eye, (x, y, h), yaw = yaw, tilt = tilt, lift = 0.004);
        }
    }
    // (short stems: along their path, their ends stretched the texture over 2x)
    artkit::uv::box_named(&["stem"]);
    fit_to(&meshes(), 0.84, 0.15, 0.84);
    settle(&[join_all("daisies")])
}
