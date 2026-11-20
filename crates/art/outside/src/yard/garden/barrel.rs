//! The rain barrel.
//!
//! Split from `yard/garden.rs` (see `mod.rs`).

use super::*;

// --- the rain barrel (1x1): a ribbed green barrel with a lid and its inlet, a brass tap ------------------------
// Free-standing, so no downpipe (Q-06): one ended in mid-air wherever the barrel stood. It stands in the middle of its
// tile (it stood 0.15 m back, the pipe against the wall).
const RB_Z: f64 = 0.0;

pub fn rain_barrel() -> Vec<Obj> {
    // the barrel: moulded in one, a foot ring, two hoop bands, ribs running up between them, a rolled top edge
    let b = F.at(0, 0, RB_Z);
    let (bx, by) = (b.x, b.y);
    let n = low(32, 12);
    let mut rows = vec![
        (0.0, 0.292, false),
        (0.008, 0.302, false),
        (0.05, 0.302, false),
        (0.07, 0.28, false),
        (0.25, 0.285, true),
        (0.285, 0.296, false),
        (0.315, 0.296, false),
        (0.34, 0.286, true),
        (0.62, 0.283, true),
        (0.645, 0.293, false),
        (0.675, 0.293, false),
        (0.7, 0.282, true),
        (0.88, 0.272, true),
        (0.9, 0.28, false),
        (0.93, 0.28, false),
        (0.935, 0.26, false),
    ];
    if is_low() {
        rows = [0, 2, 3, 5, 6, 9, 10, 12, 13, 14, 15]
            .iter()
            .map(|&i| rows[i])
            .collect();
    }
    let mut secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(y, r, _)| ring!(r, r, z = y, verts = n, at = (bx, by)))
        .collect();
    if !is_low() {
        // the ribs: every other point a little proud where `rib`
        secs = secs
            .iter()
            .zip(&rows)
            .map(|(sec, &(_, _, rib))| {
                sec.iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let k = 1.0 + if rib && i % 2 == 1 { 0.025 } else { 0.0 };
                        P3 {
                            x: bx + (p.x - bx) * k,
                            y: by + (p.y - by) * k,
                            z: p.z,
                        }
                    })
                    .collect()
            })
            .collect();
    }
    loft!("barrel", &secs, material = M.barrel, angle = 35);
    // the lid: a domed disc with a raised rim, a mesh inlet at the back and its ring
    revolve!(
        "lid",
        &[
            (0.0, 0.0),
            (0.262, 0.0),
            (0.27, 0.012),
            (0.262, 0.024),
            (0.24, 0.026),
            (0.12, 0.034),
            (0.0, 0.036)
        ],
        at = F.at(0, 0.925, RB_Z),
        material = M.barrel_dark,
        segments = low(24, 10)
    );
    fcyl!(
        F,
        "lid_inlet",
        0.065,
        0.016,
        (0, 0.962, RB_Z - 0.1),
        M.barrel_dark,
        verts = low(12, 6)
    );
    if !is_low() {
        fcyl!(
            F,
            "inlet_mesh",
            0.05,
            0.004,
            (0, 0.969, RB_Z - 0.1),
            M.tyre,
            verts = 12
        );
        for k in 0..3 {
            let a = 120 * k + 30;
            fbox!(
                F,
                numbered("lid_grip"),
                0.08,
                0.008,
                0.012,
                (0.13 * cosd(a as f64), 0.958, RB_Z + 0.13 * sind(a as f64)),
                M.barrel_dark,
                rot = (0, 0, a),
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    }
    // the brass tap at the front: a flange, a body, a spout curving down, a lever on top
    fcyl!(
        F,
        "tap_flange",
        0.032,
        0.012,
        (0, 0.17, RB_Z + 0.29),
        M.brass,
        rot = (90, 0, 0),
        verts = low(10, 6)
    );
    kit::tube!(
        F,
        "tap_spout",
        &kit::fillet!(
            &[
                (0.0, 0.17, RB_Z + 0.29),
                (0.0, 0.17, RB_Z + 0.355),
                (0.0, 0.12, RB_Z + 0.367),
            ],
            0.03,
            3
        ),
        0.016,
        M.brass,
        verts = low(10, 6),
        round = 1
    );
    fcyl!(
        F,
        "tap_body",
        0.024,
        0.05,
        (0, 0.2, RB_Z + 0.33),
        M.brass,
        verts = low(10, 6)
    );
    fsweep!(
        F,
        "tap_lever",
        &[(0, 0.23, RB_Z + 0.33), (0, 0.235, RB_Z + 0.37)],
        0,
        M.brass,
        profile = rounded_rect(0.012, 0.03, 0.004, 1),
        up = (1, 0, 0),
        scale = [1.0, 1.2]
    );
    settle!(&[join_all("rainBarrel")], centre = false)
}
