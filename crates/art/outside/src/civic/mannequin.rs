//! The mannequin: see NOTES.md.

use super::common::{F, M};
use artkit::math::P3;
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, soft};

// --- the mannequin: 1x1, a shop dummy in a jacket and trousers on a chrome stand ---------------------------------
pub fn mannequin() -> Vec<Obj> {
    let parts = new_parts(|| {
        artkit::bevel::eased(revolve!(
            "base",
            &[
                (0.0, 0.0),
                (0.2, 0.0),
                (0.2, 0.008),
                (0.19, 0.014),
                (0.0, 0.016)
            ],
            material = M.bell,
            segments = 32
        ));
        artkit::bevel::eased(fcyl!(
            F,
            "rod",
            0.012,
            0.6,
            (0, 0.31, -0.12),
            M.bell,
            verts = 10
        ));
        // the legs, in trousers, and shoes
        for s in [-1, 1] {
            let sf = s as f64;
            artkit::bevel::eased(fsweep!(
                F,
                format!("leg{s}"),
                &[
                    (sf * 0.085, 0.95, -0.01),
                    (sf * 0.09, 0.55, 0.0),
                    (sf * 0.1, 0.1, -0.02)
                ],
                0.07,
                M.cloth2,
                verts = 12,
                resolution = 2,
                scale = [1.0, 0.72, 0.62]
            ));
            soft!(
                F,
                format!("shoe{s}"),
                0.085,
                0.07,
                0.24,
                (sf * 0.1, 0.051, 0.03),
                M.radio,
                round = 0.03,
                puff = 0.01
            );
        }
        // the body: hips to shoulders, in a jacket
        let P3 { x: bx, y: by, .. } = F.at(0, 0, -0.01);
        let sections: Vec<Vec<P3>> = [
            (0.88, 0.17, 0.11),
            (0.98, 0.17, 0.11),
            (1.1, 0.14, 0.095),
            (1.28, 0.165, 0.11),
            (1.4, 0.19, 0.1),
            (1.44, 0.17, 0.09),
            (1.47, 0.07, 0.06),
        ]
        .iter()
        .map(|&(y, rx, rz)| ring!(rx, rz, z = y, verts = 20, square = 2.2, at = (bx, by)))
        .collect();
        artkit::bevel::eased(loft!("torso", &sections, material = M.cloth1));
        // the shirt in the jacket's open neck, its buttons
        artkit::bevel::eased(fextrude!(
            F,
            "shirt",
            &[(-0.06, 1.455), (0.06, 1.455), (0.0, 1.27)],
            0.02,
            (0, 0, 0.088),
            M.cloth4,
            plane = Facing::Front
        ));
        for k in 0..3 {
            let kf = k as f64;
            fsphere!(
                F,
                format!("button{k}"),
                0.009,
                (0, 1.2 - kf * 0.08, 0.097 - kf * 0.002),
                M.radio,
                segments = 8,
                rings = 6
            );
        }
        for s in [-1, 1] {
            let sf = s as f64;
            artkit::bevel::eased(fsweep!(
                F,
                format!("arm{s}"),
                &[
                    (sf * 0.19, 1.4, -0.01),
                    (sf * 0.23, 1.15, 0.0),
                    (sf * 0.24, 0.9, 0.04)
                ],
                0.045,
                M.cloth1,
                verts = 10,
                resolution = 2,
                scale = [1.0, 0.85, 0.75]
            ));
            fsphere!(
                F,
                format!("hand{s}"),
                0.04,
                (sf * 0.245, 0.84, 0.05),
                M.dummy,
                scale = (0.6, 0.8, 1.3),
                segments = 10,
                rings = 8
            );
        }
        artkit::bevel::eased(fcyl!(
            F,
            "neck",
            0.045,
            0.12,
            (0, 1.5, -0.01),
            M.dummy,
            verts = 12
        ));
        fsphere!(
            F,
            "head",
            0.1,
            (0, 1.64, 0.0),
            M.dummy,
            scale = (0.82, 0.95, 1.15),
            segments = 18,
            rings = 12
        );
    });
    join(&parts, "mannequin", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}
