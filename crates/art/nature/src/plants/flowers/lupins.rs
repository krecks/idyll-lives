use artkit::botany::{fit_to, lupin_spike, palette};
use artkit::math::v3;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{mound, rays, spiral, stalk};

/// Lupins: hand-shaped leaves of narrow leaflets on stalks and tall tapering spikes of pea flowers in purples,
/// pinks, white and blue.
pub fn lupins() -> Vec<Obj> {
    let p = palette(&[
        ("blossom", "#8E6CE0"),
        ("blossom2", "#F08CC4"),
        ("blossom3", "#F4F0F2"),
        ("blossom4", "#6C8CF0"),
    ]);
    let m = |name: &str| p.get(name);
    let mut rng = Random::new(59);
    let top = mound!(&p);
    let cols = [m("leaf"), m("leaf_dark"), m("leaf_light")];
    // the leaves: leaflets fanned round a stalk's end
    for (i, (x, y, a)) in spiral!(10, 0.3, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.12, 0.22);
        let end = v3(x * 1.15, y * 1.15, h);
        stalk!(
            format!("petiole{i}"),
            (x * 0.6, y * 0.6, top),
            end,
            m("stem"),
            r = 0.004
        );
        let tilt = rng.uniform(5.0, 25.0);
        rays!(
            format!("leaf{i}"),
            end,
            0.085,
            9,
            cols[i % 3],
            yaw = a,
            tilt = tilt,
            inner = 0.08,
            cup = 0.15,
            droop = 0.2,
            width = 0.42,
            thick = 0.004
        );
    }
    let cols = [m("blossom"), m("blossom2"), m("blossom3"), m("blossom4")];
    for (i, (x, y, _)) in spiral!(9, 0.3, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.66, 0.99);
        let tip = (
            x + rng.uniform(-0.03, 0.03),
            y + rng.uniform(-0.03, 0.03),
            h,
        );
        stalk!(
            format!("stem{i}"),
            (x, y, top + 0.02),
            (tip.0, tip.1, h - 0.34),
            m("stem"),
            r = 0.007
        );
        lupin_spike(
            format!("spike{i}"),
            (tip.0, tip.1, h - 0.36),
            0.36,
            0.045,
            cols[i % 4],
        );
    }
    fit_to(&meshes(), 0.84, 0.987, 0.84);
    settle(&[join_all("lupins")])
}
