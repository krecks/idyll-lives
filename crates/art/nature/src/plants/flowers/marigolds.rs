use artkit::botany::{clumps, fit_to, palette};
use artkit::far::is_low;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{cut_leaf, florets, mound, pinnate, spiral};

/// Marigolds: bushy clumps of dark, finely cut leaves covered in ruffled orange and yellow pompoms.
pub fn marigolds() -> Vec<Obj> {
    let p = palette(&[
        ("leaf", "#2F5A28"),
        ("leaf_dark", "#3A6A2E"),
        ("leaf_light", "#467634"),
        ("blossom", "#F8901C"),
        ("blossom2", "#FCBA1E"),
        ("blossom3", "#F0761A"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(56);
    let top = mound!(&p);
    clumps(
        &mut rng,
        7,
        0.33,
        0.27,
        0.17,
        top + 0.07,
        &[m("leaf"), m("leaf_dark"), m("leaf_light")],
    );
    // cut leaves sticking out round the clumps
    for (i, (x, y, a)) in spiral!(7, 0.38, &mut rng).into_iter().enumerate() {
        let (yaw, t) = (a + rng.uniform(-30.0, 30.0), rng.uniform(15.0, 40.0));
        if is_low() {
            continue;
        }
        cut_leaf!(
            format!("leaf{i}"),
            (x * 0.8, y * 0.8, top + 0.06),
            yaw,
            t,
            0.13,
            &pinnate!(3, 0.06, "lance", lean = 0.6, fill = 0.6),
            m("leaf_light"),
            droop = 0.3,
            ribs = 2
        );
    }
    for (i, (x, y, _)) in spiral!(16, 0.38, &mut rng).into_iter().enumerate() {
        let z = rng.uniform(0.19, 0.27);
        florets!(
            format!("pom{i}"),
            0.05,
            (x, y, z),
            [m("blossom"), m("blossom2"), m("blossom3")][i % 3],
            bumps = 4.5,
            amount = 0.18,
            flat = 0.75,
            segments = 8,
            rings = 5,
            seed = i as f64 * 0.7
        );
    }
    fit_to(&meshes(), 0.885, 0.334, 0.84);
    settle(&[join_all("marigolds")])
}
