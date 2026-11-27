//! BR4: the shoe rack — three slatted shelves between four legs.

use super::common::M;
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, settle};

pub fn shoe_rack() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, d) = (0.90, 0.30);
    for (i, (sx, sz)) in [(-1.0f64, -1.0f64), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .enumerate()
    {
        fbox!(
            f,
            format!("leg{i}"),
            0.035,
            0.54,
            0.035,
            (sx * (w / 2.0 - 0.02), 0.27, sz * (d / 2.0 - 0.02)),
            M.oak,
            bevel = 0.004
        );
    }
    for (s, y) in [0.10, 0.28, 0.46].into_iter().enumerate() {
        for i in 0..3 {
            let z = -0.10 + 0.10 * i as f64;
            fbox!(
                f,
                format!("slat{s}{i}"),
                w - 0.08,
                0.014,
                0.085,
                (0, y, z),
                M.oak
            );
        }
    }
    fbox!(f, "top", w, 0.020, d, (0, 0.55, 0), M.oak, bevel = 0.004);
    settle!();
    meshes()
}
