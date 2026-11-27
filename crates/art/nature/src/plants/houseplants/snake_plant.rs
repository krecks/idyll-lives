use artkit::botany::{palette, sword};
use artkit::far::is_low;
use artkit::math::{atan2, cos, degrees, radians, sin};
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{cut_leaf, outline, vessel};

/// Snake plant: a charcoal pot and stiff, upright, slightly twisting sword-shaped blades, banded dark green with
/// pale yellow-green edges, and a couple of young ones coming up.
pub fn snake_plant() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#2E2E30"),
        ("leaf", "#2F5A34"),
        ("leaf_dark", "#2A4E30"),
        ("leaf_light", "#B8C060"),
    ]);
    let ys = vessel!(
        &p,
        0.15,
        0.29,
        r_bottom = 0.13,
        lip = 0.012,
        foot = 0.0,
        segments = 18
    );
    let mut rng = Random::new(93);
    let cols = [p.get("leaf"), p.get("leaf_dark")];
    for i in 0..12 {
        let (x, y) = (rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08));
        let yaw = degrees(atan2(x, y)) + rng.uniform(-20.0, 20.0);
        let (tilt, mut length, width, twist) = (
            rng.uniform(78.0, 88.0),
            rng.uniform(0.46, 0.64),
            rng.uniform(0.07, 0.09),
            rng.uniform(-50.0, 50.0),
        );
        // (a grown snake plant's swords stand 1-1.5 m)
        length *= 1.65;
        if i >= 10 {
            // young ones
            length *= 0.55;
        }
        let base = (x, y, ys - 0.02);
        cut_leaf!(
            format!("blade{i}"),
            base,
            yaw,
            tilt,
            length,
            &outline!(sword, width, n = 6),
            cols[i % 2],
            fold = 0.3,
            ribs = 5,
            thick = 0.014,
            twist = twist
        );
        if !is_low() {
            // the pale edge, just wider, just behind
            cut_leaf!(
                format!("edge{i}"),
                (
                    x - 0.0015 * sin(radians(yaw)),
                    y - 0.0015 * cos(radians(yaw)),
                    ys - 0.02
                ),
                yaw,
                tilt,
                length * 1.01,
                &outline!(sword, width * 1.14, n = 6),
                p.get("leaf_light"),
                fold = 0.3,
                ribs = 2,
                thick = 0.004,
                twist = twist
            );
        }
    }
    settle(&[join_all("snakePlant")])
}
