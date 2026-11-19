//! The reeds: a clump of strap blades with a few bulrushes standing above them.

use super::{F, M};
use artkit::math::P3;
use artkit::parts::{join_all, settle_all};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{fsweep, leaf, loft, ring};

pub fn reeds() -> Vec<Obj> {
    let mut rng = Random::from_float(0.83);
    // blades: mostly upright, the shorter ones leaning out
    for i in 0..24 {
        let x = rng.uniform(-0.18, 0.18);
        let z = rng.uniform(-0.18, 0.18);
        let (length, tilt) = if i % 3 == 0 {
            (rng.uniform(0.45, 0.62), rng.uniform(52.0, 66.0))
        } else {
            (rng.uniform(0.85, 1.52), rng.uniform(76.0, 88.0))
        };
        leaf!(
            format!("blade{i}"),
            F.at(x, 0.0, z),
            rng.uniform(0.0, 360.0),
            tilt,
            length,
            rng.uniform(0.035, 0.055),
            M.blade,
            shape = "strap",
            droop = rng.uniform(0.10, 0.25),
            thick = 0.005,
            fold = 0.3,
            rows = 4,
            ring = 3
        );
    }
    // bulrushes: a slim stem and a brown seed head near the top
    for i in 0..7 {
        let x = rng.uniform(-0.20, 0.20);
        let z = rng.uniform(-0.20, 0.20);
        let h = rng.uniform(1.1, 1.5);
        let (lx, lz) = (rng.uniform(-0.05, 0.05), rng.uniform(-0.05, 0.05));
        fsweep!(
            F,
            format!("stem{i}"),
            &[(x, 0.0, z), (x + lx * 0.5, h * 0.55, z + lz * 0.5), (x + lx, h, z + lz)],
            0.008,
            M.stem,
            verts = 5,
            resolution = 2,
            scale = [1.0, 0.85, 0.6]
        );
        let top = F.at(x + lx, h, z + lz);
        let sections: Vec<Vec<P3>> = [
            (-0.075, 0.006),
            (-0.055, 0.02),
            (-0.02, 0.026),
            (0.02, 0.026),
            (0.055, 0.02),
            (0.075, 0.006),
        ]
        .iter()
        .map(|&(dy, r)| ring!(r, r, z = h + dy, verts = 8, at = (top.x, top.y)))
        .collect();
        loft!(format!("head{i}"), &sections, material = M.head);
    }
    join_all("reeds");
    settle_all()
}
