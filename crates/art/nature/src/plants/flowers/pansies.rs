use artkit::botany::{fit_to, palette};
use artkit::far::is_low;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{mound, rays, rosettes, spiral, stalk};

/// Pansies: a low carpet of small round leaves and flat, overlapping five-petalled faces turned outwards, each
/// with a dark blotch at its heart.
pub fn pansies() -> Vec<Obj> {
    let p = palette(&[
        ("bloom", "#5B3A8E"),
        ("bloom2", "#F2C94C"),
        ("bloom3", "#3F5FB0"),
        ("bloom4", "#8E2A3A"),
        ("bloom5", "#F4F0E8"),
        ("centre", "#F2C94C"),
        ("centre_dark", "#3A2A4E"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(57);
    let top = mound!(&p);
    rosettes!(
        &mut rng,
        8,
        0.34,
        top - 0.01,
        &[m("leaf"), m("leaf_dark"), m("leaf_light")],
        leaves = 5,
        length = 0.13,
        width = 0.08,
        shape = "oval",
        tilt = (10.0, 30.0)
    );
    let faces = [
        (m("bloom"), m("centre")),
        (m("bloom2"), m("centre_dark")),
        (m("bloom3"), m("centre")),
        (m("bloom4"), m("centre")),
        (m("bloom5"), m("centre_dark")),
    ];
    for (i, (x, y, a)) in spiral!(15, 0.36, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.17, 0.23);
        let (petal, eye) = faces[i % faces.len()];
        let (yaw, tilt) = (a + rng.uniform(-30.0, 30.0), rng.uniform(35.0, 60.0));
        // on stems: without, the faces stood in the air
        stalk!(
            format!("stem{i}"),
            (x, y, top + 0.02),
            (x, y, h),
            m("stem"),
            r = 0.004
        );
        rays!(
            format!("pansy{i}"),
            (x, y, h),
            0.045,
            5,
            petal,
            yaw = yaw,
            tilt = tilt,
            inner = 0.25,
            cup = 0.05,
            width = 1.35,
            thick = 0.006
        );
        if !is_low() {
            rays!(
                format!("eye{i}"),
                (x, y, h),
                0.02,
                3,
                eye,
                yaw = yaw,
                tilt = tilt,
                inner = 0.3,
                cup = 0.05,
                width = 1.2,
                thick = 0.012
            );
        }
    }
    fit_to(&meshes(), 0.84, 0.264, 0.84);
    settle(&[join_all("pansies")])
}
