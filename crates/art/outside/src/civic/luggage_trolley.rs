//! The luggage trolley: see NOTES.md.

use super::common::{F, M, rod, tube};
use artkit::json::float_repr;
use artkit::math::{cos, radians, sin};
use artkit::parts::{join, new_parts, settle_all};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, fsweep, soft};

// --- the luggage trolley: 1x1, a carpeted deck on castors under a brass arch, two cases on it ---------------------
pub fn luggage_trolley() -> Vec<Obj> {
    let deck = 0.17;
    let parts = new_parts(|| {
        fbox!(
            F,
            "deck",
            0.88,
            0.05,
            0.52,
            (0, deck - 0.025, 0),
            M.carpet,
            bevel = 0.006
        );
        fbox!(
            F,
            "deck_rim",
            0.9,
            0.03,
            0.54,
            (0, deck - 0.06, 0),
            M.brass,
            bevel = 0.006
        );
        for x in [-0.38, 0.38] {
            let xs = float_repr(x);
            for z in [-0.2, 0.2] {
                let zs = float_repr(z);
                fbox!(
                    F,
                    format!("fork{xs}{zs}"),
                    0.03,
                    0.06,
                    0.05,
                    (x, deck - 0.09, z),
                    M.brass
                );
                artkit::bevel::eased(fcyl!(
                    F,
                    format!("wheel{xs}{zs}"),
                    0.045,
                    0.03,
                    (x, 0.045, z),
                    M.rubber,
                    rot = (0, 90, 0),
                    verts = 14
                ));
            }
        }
        // the uprights and the arch, a hanging rail across
        for s in [-1, 1] {
            let sf = s as f64;
            artkit::bevel::eased(fcyl!(
                F,
                format!("upright{s}"),
                0.016,
                1.45,
                (sf * 0.42, deck + 0.725, 0),
                M.brass,
                verts = 10
            ));
            fsphere!(
                F,
                format!("knob{s}"),
                0.03,
                (sf * 0.42, deck + 1.46, 0),
                M.brass,
                segments = 12,
                rings = 8
            );
            rod(
                &format!("bumper{s}"),
                (-0.42, deck + 0.08, sf * 0.25),
                (0.42, deck + 0.08, sf * 0.25),
                0.012,
                M.brass,
                8,
            );
        }
        let arch: Vec<(f64, f64, i32)> = (0..181)
            .step_by(15)
            .map(|d| radians(d as f64))
            .map(|a| (0.42 * cos(a), deck + 1.45 + 0.2 * sin(a), 0))
            .collect();
        artkit::bevel::eased(fsweep!(F, "arch", &arch, 0.016, M.brass, verts = 10));
        rod(
            "hanging_rail",
            (-0.42, deck + 1.3, 0),
            (0.42, deck + 1.3, 0),
            0.013,
            M.brass,
            8,
        );
        fsphere!(
            F,
            "crown",
            0.035,
            (0, deck + 1.67, 0),
            M.brass,
            segments = 12,
            rings = 8
        );
        // a case standing, another lying on the deck, a garment bag on the rail
        soft!(
            F,
            "case_big",
            0.5,
            0.62,
            0.24,
            (-0.12, deck + 0.31, 0.06),
            M.case1,
            round = 0.03,
            puff = 0.01
        );
        tube(
            "case_big_handle",
            &[
                (-0.2, deck + 0.62, 0.06),
                (-0.2, deck + 0.66, 0.06),
                (-0.04, deck + 0.66, 0.06),
                (-0.04, deck + 0.62, 0.06),
            ],
            0.01,
            M.rubber,
            0.015,
            8,
        );
        soft!(
            F,
            "case_small",
            0.36,
            0.18,
            0.42,
            (0.22, deck + 0.09, -0.02),
            M.case2,
            round = 0.03,
            puff = 0.01
        );
        tube(
            "case_small_handle",
            &[
                (0.4, deck + 0.07, -0.08),
                (0.43, deck + 0.07, -0.08),
                (0.43, deck + 0.07, 0.04),
                (0.4, deck + 0.07, 0.04),
            ],
            0.009,
            M.rubber,
            0.012,
            8,
        );
        soft!(
            F,
            "garment_bag",
            0.08,
            0.7,
            0.42,
            (0.28, deck + 1.3 - 0.36, -0.03),
            M.radio,
            round = 0.03,
            puff = 0.012
        );
        tube(
            "bag_hook",
            &[
                (0.28, deck + 1.28, -0.03),
                (0.28, deck + 1.34, -0.03),
                (0.28, deck + 1.33, -0.0),
            ],
            0.004,
            M.bell,
            0.015,
            6,
        );
    });
    join(&parts, "trolley", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}
