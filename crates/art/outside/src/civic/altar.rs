//! The altar: see NOTES.md.

use super::common::{F, M, open_book};
use artkit::parts::{join, new_parts, settle_all};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, revolve};

// --- the altar: 2x1, stone under a white linen and a frontal, two candlesticks and a cross ------------------------
pub fn candlestick(name: &str, x: f64, y: f64, z: f64) {
    artkit::bevel::eased(revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.08, 0.0),
            (0.08, 0.015),
            (0.06, 0.03),
            (0.03, 0.05),
            (0.02, 0.1),
            (0.03, 0.12),
            (0.016, 0.14),
            (0.014, 0.3),
            (0.024, 0.32),
            (0.05, 0.34),
            (0.05, 0.35),
            (0.0, 0.35)
        ],
        at = F.at(x, y, z),
        material = M.brass,
        segments = 16
    ));
    artkit::bevel::eased(fcyl!(
        F,
        format!("{name}_candle"),
        0.022,
        0.24,
        (x, y + 0.35 + 0.12, z),
        M.candle,
        verts = 12
    ));
    fsphere!(
        F,
        format!("{name}_flame"),
        0.012,
        (x, y + 0.6 + 0.018, z),
        M.flame,
        scale = (1, 1, 2.2),
        segments = 8,
        rings = 6
    );
}

pub fn altar() -> Vec<Obj> {
    let parts = new_parts(|| {
        fbox!(
            F,
            "step",
            1.92,
            0.06,
            0.84,
            (0, 0.03, 0),
            M.stone,
            bevel = 0.008
        );
        fbox!(
            F,
            "body",
            1.7,
            0.92,
            0.7,
            (0, 0.06 + 0.46, -0.02),
            M.stone,
            bevel = 0.006
        );
        fbox!(
            F,
            "mensa",
            1.82,
            0.06,
            0.8,
            (0, 1.01, -0.02),
            M.stone,
            bevel = 0.012
        );
        // the fair linen over the top, hanging down the ends
        fbox!(F, "linen", 1.86, 0.008, 0.82, (0, 1.044, -0.02), M.linen);
        for s in [-1, 1] {
            fbox!(
                F,
                format!("linen_drop{s}"),
                0.008,
                0.4,
                0.82,
                (s as f64 * 0.934, 0.85, -0.02),
                M.linen
            );
        }
        // the frontal down the front, a gold cross and fringe on it
        fbox!(
            F,
            "frontal",
            1.3,
            0.8,
            0.01,
            (0, 0.64, 0.344),
            M.frontal,
            bevel = 0.002
        );
        fbox!(
            F,
            "frontal_band",
            1.3,
            0.06,
            0.012,
            (0, 1.01, 0.346),
            M.linen
        );
        fbox!(F, "fringe", 1.3, 0.04, 0.012, (0, 0.26, 0.346), M.brass);
        fbox!(
            F,
            "frontal_cross_v",
            0.06,
            0.4,
            0.004,
            (0, 0.65, 0.351),
            M.brass
        );
        fbox!(
            F,
            "frontal_cross_h",
            0.26,
            0.06,
            0.004,
            (0, 0.72, 0.351),
            M.brass
        );
        let t = 1.048;
        for s in [-1, 1] {
            candlestick(&format!("candlestick{s}"), s as f64 * 0.62, t, -0.12);
        }
        artkit::bevel::eased(revolve!(
            "cross_base",
            &[
                (0.0, 0.0),
                (0.09, 0.0),
                (0.09, 0.02),
                (0.06, 0.04),
                (0.04, 0.08),
                (0.0, 0.08)
            ],
            at = F.at(0, t, -0.2),
            material = M.brass,
            segments = 16
        ));
        fbox!(
            F,
            "cross_v",
            0.035,
            0.5,
            0.025,
            (0, t + 0.33, -0.2),
            M.brass,
            bevel = 0.005
        );
        fbox!(
            F,
            "cross_h",
            0.26,
            0.035,
            0.025,
            (0, t + 0.45, -0.2),
            M.brass,
            bevel = 0.005
        );
        open_book("missal", 0.3, 0.22, (0.3, t, 0.08), M.book, 8.0, 0.02);
    });
    join(&parts, "altar", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}
