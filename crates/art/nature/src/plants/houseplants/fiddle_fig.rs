use artkit::botany::{fiddle, fit_to, palette, polar};
use artkit::far::{is_low, low};
use artkit::forms::by;
use artkit::math::V3;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{cut_leaf, jitter, outline, sweep, twig, vessel};

/// Fiddle-leaf fig: a cream pot with a rolled rim, a slim, slightly crooked trunk in bark, and big leathery,
/// wavy-edged fiddle-shaped leaves spiralling up it on short stalks.
pub fn fiddle_fig() -> Vec<Obj> {
    let p = super::super::plain_leaves(palette(&[
        ("body", "#D9CDB4"),
        ("leaf", "#2E5A2A"),
        ("leaf_dark", "#2A5226"),
        ("leaf_light", "#386832"),
        ("bark", "#8A7058"),
    ]));
    let ys = vessel!(
        &p,
        0.2,
        0.38,
        r_bottom = 0.15,
        lip = 0.016,
        foot = 0.025,
        segments = 20
    );
    let mut rng = Random::new(92);
    let trunk_pts = [
        (0.0, 0.0, ys - 0.02),
        (0.01, 0.0, 0.8),
        (-0.012, 0.01, 1.2),
        (0.0, 0.0, 1.5),
    ];
    let stem = sweep!(
        "trunk",
        &trunk_pts,
        0.022,
        material = p.get("bark"),
        verts = low(7, 4),
        resolution = low(3, 1),
        scale = by(|w| 1.0 - 0.55 * w)
    );
    if !is_low() {
        jitter!(stem, 0.004, size = 0.08, seed = 2.0, up = 0.0);
    }
    let n = 22;
    for i in 0..n {
        let t = i as f64 / n as f64;
        let z = ys + 0.28 + t * (1.46 - ys - 0.28);
        let yaw = i as f64 * 137.5 + rng.uniform(-10.0, 10.0);
        let s = 1.05 - t * 0.3;
        let (tilt, length, width) = (
            rng.uniform(15.0, 40.0),
            rng.uniform(0.26, 0.3) * s,
            rng.uniform(0.2, 0.23) * s,
        );
        let r_at = 0.02 * (1.0 - 0.5 * t);
        let base = V3::from(polar(r_at, yaw, z));
        let end = base + V3::from(polar(0.035, yaw, 0.02));
        twig!(
            format!("stalk{i}"),
            &[base, end],
            0.005,
            0.004,
            p.get("leaf_light"),
            verts = 3,
            res = 1
        );
        cut_leaf!(
            format!("leaf{i}"),
            end,
            yaw,
            tilt,
            length,
            &outline!(fiddle, width, n = 7, wave = 0.05, waves = 5),
            [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")][i % 3],
            droop = 0.22,
            fold = 0.16,
            ribs = 5,
            thick = 0.008
        );
    }
    cut_leaf!(
        "top",
        (0, 0, 1.49),
        20.0,
        70.0,
        0.14,
        &outline!(fiddle, 0.09, n = 5),
        p.get("leaf_light"),
        droop = 0.1,
        ribs = 3
    );
    fit_to(&meshes(), 0.528, 1.626, 0.527);
    settle(&[join_all("fiddleFig")])
}
