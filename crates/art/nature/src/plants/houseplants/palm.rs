use artkit::botany::{fit_to, palette, polar};
use artkit::math::{V3, cos, radians, sin, v3};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{cut_leaf, pinnate, twig, vessel};

/// Parlour palm: a cream pot with a rolled rim, slender arching stalks, each a feathery frond of narrow leaflets,
/// and a young frond still upright in the middle.
pub fn palm() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#D9CDB4"),
        ("leaf", "#4A8A3A"),
        ("leaf_dark", "#3E7A34"),
        ("leaf_light", "#5A9A42"),
        ("stem", "#5E7E3A"),
    ]);
    let ys = vessel!(
        &p,
        0.19,
        0.33,
        r_bottom = 0.14,
        lip = 0.014,
        foot = 0.0,
        segments = 18
    );
    let mut rng = Random::new(97);
    let cols = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    for i in 0..9 {
        let young = i == 8;
        let yaw = i as f64 * 45.0 + rng.uniform(-15.0, 15.0);
        let mut up = rng.uniform(72.0, 86.0);
        let mut length = rng.uniform(0.3, 0.48);
        let (mut flen, mut droop) = (rng.uniform(0.3, 0.36), rng.uniform(0.75, 0.95));
        if young {
            (up, length, flen, droop) = (88.0, 0.32, 0.22, 0.2);
        }
        let tip = V3::from(polar(
            length * cos(radians(up)),
            yaw,
            ys + length * sin(radians(up)),
        ));
        twig!(
            format!("stalk{i}"),
            &[
                v3(0, 0, ys - 0.01),
                v3(0, 0, ys).lerp(tip, 0.5) + V3::from(polar(0.01, yaw, 0.0)),
                tip
            ],
            0.006,
            0.004,
            p.get("stem"),
            verts = 3,
            res = 1
        );
        cut_leaf!(
            format!("frond{i}"),
            tip,
            yaw,
            up - 18.0,
            flen,
            &pinnate!(10, 0.26, "lance", lean = 0.85, fill = 0.3),
            cols[i % 3],
            droop = droop,
            fold = 0.1,
            ribs = 5
        );
    }
    fit_to(&meshes(), 0.770, 0.905, 0.693);
    settle(&[join_all("palm")])
}
