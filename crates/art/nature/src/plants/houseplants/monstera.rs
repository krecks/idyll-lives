use artkit::botany::{fit_to, palette, polar};
use artkit::far::{is_low, low};
use artkit::math::V3;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{arc, cut_leaf, cylinder, jitter, slits, twig, vessel};

/// Monstera: a white pot, a moss pole, and big glossy leaves split from their edges almost to the midrib, on long
/// arching stems, with a couple of aerial roots reaching down.
pub fn monstera() -> Vec<Obj> {
    let p = super::super::plain_leaves(palette(&[
        ("body", "#EDEBE6"),
        ("leaf", "#2F6B35"),
        ("leaf_dark", "#28602E"),
        ("leaf_light", "#3B7A3C"),
        ("stem", "#4E7A3A"),
        ("moss", "#6E6A3E"),
    ]));
    let ys = vessel!(
        &p,
        0.18,
        0.33,
        r_bottom = 0.14,
        lip = 0.0,
        foot = 0.02,
        segments = 18
    );
    let pole = cylinder!(
        "pole",
        0.024,
        0.62,
        at = (0.02, 0.04, ys + 0.28),
        material = p.get("moss"),
        verts = low(8, 5)
    );
    if !is_low() {
        jitter!(pole, 0.006, size = 0.05, seed = 3.0);
    }
    let mut rng = Random::new(91);
    let cols = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    for i in 0..8 {
        let yaw = i as f64 / 8.0 * 720.0 + rng.uniform(-15.0, 15.0);
        let out = rng.uniform(0.07, 0.13);
        let h = rng.uniform(0.3, 0.6);
        let (tilt, length, width) = (
            rng.uniform(5.0, 25.0),
            rng.uniform(0.29, 0.33),
            rng.uniform(0.27, 0.31),
        );
        let tip = V3::from(polar(out, yaw, ys + h));
        twig!(
            format!("stem{i}"),
            &arc!((0, 0, ys), tip, 0.06, steps = 3),
            0.009,
            0.006,
            p.get("stem"),
            verts = 4,
            res = 1
        );
        cut_leaf!(
            format!("leaf{i}"),
            tip,
            yaw,
            tilt,
            length,
            &slits!("heart", width, 5, depth = 0.62, gap = 0.22),
            cols[i % 3],
            droop = 0.5,
            fold = 0.1,
            ribs = 5,
            thick = 0.006
        );
    }
    // aerial roots
    for i in 0..low(2, 0) {
        let a = (40 + i * 150) as f64;
        let top = V3::from(polar(0.05, a, ys + 0.22));
        twig!(
            format!("root{i}"),
            &[
                top,
                top + V3::from(polar(0.08, a, -0.06)),
                V3::from(polar(0.15, a, ys - 0.005))
            ],
            0.005,
            0.004,
            p.get("stem"),
            verts = 3
        );
    }
    fit_to(&meshes(), 0.784, 0.911, 0.791);
    settle(&[join_all("monstera")])
}
