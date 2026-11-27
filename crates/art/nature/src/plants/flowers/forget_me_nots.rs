use artkit::botany::{clumps, fit_to, palette};
use artkit::far::is_low;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{florets, mound, rays, spiral};

/// Forget-me-nots: leafy clumps under drifts of tiny sky-blue flowers, a few open ones standing out.
pub fn forget_me_nots() -> Vec<Obj> {
    let p = palette(&[
        ("blossom", "#8CB4F8"),
        ("blossom2", "#A4C4FC"),
        ("blossom3", "#7AA0F0"),
        ("centre", "#F6E27A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(60);
    let top = mound!(&p);
    clumps(
        &mut rng,
        7,
        0.33,
        0.28,
        0.15,
        top + 0.05,
        &[m("leaf"), m("leaf_dark"), m("leaf_light")],
    );
    let cols = [m("blossom"), m("blossom2"), m("blossom3")];
    for (i, (x, y, _)) in spiral!(16, 0.36, &mut rng).into_iter().enumerate() {
        let z = rng.uniform(0.17, 0.23);
        florets!(
            format!("drift{i}"),
            0.06,
            (x, y, z),
            cols[i % 3],
            bumps = 6.0,
            amount = 0.22,
            flat = 0.4,
            segments = 8,
            rings = 4,
            seed = i as f64 * 0.9
        );
    }
    // single flowers standing out of the drifts
    for (i, (x, y, _)) in spiral!(10, 0.34, &mut rng).into_iter().enumerate() {
        let (h, yaw, tilt) = (
            rng.uniform(0.22, 0.27),
            rng.uniform(0.0, 360.0),
            rng.uniform(0.0, 30.0),
        );
        if is_low() {
            continue;
        }
        rays!(
            format!("flower{i}"),
            (x, y, h),
            0.02,
            5,
            cols[i % 3],
            yaw = yaw,
            tilt = tilt,
            inner = 0.3,
            cup = 0.1,
            width = 1.1,
            centre = 0.003
        );
    }
    fit_to(&meshes(), 0.866, 0.28, 0.84);
    settle(&[join_all("forgetMeNots")])
}
