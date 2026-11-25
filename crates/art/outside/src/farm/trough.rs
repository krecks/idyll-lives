//! The water trough: an oval galvanised stock tank full of water on two skids, with its float valve, feed pipe and
//! drain, and its far version.

use super::common::{F, M, stadium_slab, stadium_sweep};
use artkit::json::float_repr;
use artkit::parts::{join_all, settle};
use artkit::scene::Obj;
use artkit::{cylinder, fbox, fcyl, fsphere, kit_fillet, kit_tube, rod, sphere};

// --- the trough ------------------------------------------------------------------------------------------
const TR_R: f64 = 0.35; // a stock tank: round ends of 0.35 m, a straight middle 1.1 m long
const TR_HALF: f64 = 0.55;

/// A galvanised oval stock tank: corrugated sides with seams where its ends join, a rolled rim, water inside, on two
/// skids; a float valve inside one end, fed by a pipe up from the ground through the end, with its tap, and a drain
/// plug at the other.
pub fn trough() -> Vec<Obj> {
    let (y0, top) = (0.1, 0.65);
    let mut prof = vec![(TR_R - 0.02, y0), (TR_R, y0 + 0.03)];
    for y in [0.2, 0.27, 0.34, 0.41, 0.48, 0.55] {
        prof.extend([(TR_R, y - 0.025), (TR_R + 0.012, y), (TR_R, y + 0.025)]);
    }
    prof.extend([
        (TR_R, top - 0.04),
        (TR_R + 0.018, top - 0.02),
        (TR_R + 0.02, top),
        (TR_R, top + 0.004),
        (TR_R - 0.018, top - 0.015),
        (TR_R - 0.018, y0 + 0.04),
    ]);
    trough_tank(&prof, 12);
    // the seams where the round ends meet the straight sides, riveted
    for x in [-TR_HALF, TR_HALF] {
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("seam{}{s}", float_repr(x)),
                0.03,
                top - y0 - 0.08,
                0.008,
                (x, (top + y0) / 2.0, sf * (TR_R + 0.012)),
                M.trough
            );
            for k in 0..5 {
                fsphere!(
                    F,
                    format!("rivet{}{s}{k}", float_repr(x)),
                    0.007,
                    (x, y0 + 0.08 + k as f64 * 0.1, sf * (TR_R + 0.017)),
                    M.steel,
                    segments = 6,
                    rings = 4
                );
            }
        }
    }
    // the float valve hung inside the rim at one end, and its feed pipe: up from the ground, over the rim and down in
    fbox!(
        F,
        "valve",
        0.12,
        0.09,
        0.08,
        (-0.78, top - 0.04, -0.12),
        M.steel,
        bevel = 0.01,
        bevel_segments = 1
    );
    sphere!(
        "float",
        0.055,
        at = F.at(-0.6, top - 0.09, -0.12),
        material = M.steel,
        segments = 10,
        rings = 6
    );
    rod!(
        "float_arm",
        F.at(-0.74, top - 0.04, -0.12),
        F.at(-0.6, top - 0.08, -0.12),
        0.008,
        material = M.steel,
        verts = 6
    );
    kit_tube!(
        F,
        "feed",
        &kit_fillet!(
            &[
                (-0.935, 0.0, -0.2),
                (-0.935, 0.5, -0.2),
                (-0.84, 0.5, -0.12)
            ],
            0.04
        ),
        0.016,
        M.steel,
        verts = 6,
        round = 1
    );
    fcyl!(
        F,
        "feed_tap",
        0.024,
        0.05,
        (-0.935, 0.3, -0.2),
        M.steel,
        verts = 8
    );
    rod!(
        "feed_lever",
        F.at(-0.935, 0.3, -0.2),
        F.at(-0.935, 0.3, -0.11),
        0.008,
        material = M.iron,
        verts = 4
    );
    // the drain plug low on the other end
    cylinder!(
        "drain",
        0.03,
        0.03,
        at = F.at(TR_HALF + TR_R + 0.01, y0 + 0.06, 0),
        material = M.steel,
        verts = 6,
        rot = (0, 90, 0)
    );
    settle(&[join_all("trough")])
}

fn trough_tank(prof: &[(f64, f64)], n: usize) {
    let (y0, top) = (0.1, 0.65);
    stadium_sweep("tank", prof, TR_HALF, M.trough, n);
    stadium_slab(
        "tank_floor",
        TR_R - 0.01,
        TR_HALF,
        y0,
        y0 + 0.045,
        M.trough,
        n,
    );
    stadium_slab(
        "water",
        TR_R - 0.017,
        TR_HALF,
        y0 + 0.04,
        top - 0.07,
        M.water,
        n,
    );
    for x in [-0.62, 0.62] {
        fbox!(
            F,
            format!("skid{}", float_repr(x)),
            0.12,
            y0,
            0.66,
            (x, y0 / 2.0, 0),
            M.steel,
            bevel = if n > 4 { 0.01 } else { 0.0 },
            bevel_segments = 1
        );
    }
}

/// The trough from afar: plain sides and rim (no corrugations), the water, the skids, the valve, the feed pipe.
pub fn trough_far() -> Vec<Obj> {
    let (y0, top) = (0.1, 0.65);
    trough_tank(
        &[
            (TR_R - 0.02, y0),
            (TR_R, y0 + 0.03),
            (TR_R + 0.02, top),
            (TR_R - 0.018, top),
            (TR_R - 0.018, y0 + 0.04),
        ],
        4,
    );
    fbox!(
        F,
        "valve",
        0.12,
        0.09,
        0.08,
        (-0.78, top - 0.04, -0.12),
        M.steel
    );
    // the feed pipe and its tap, the farthest out at that end, so it stands where the trough does
    fcyl!(
        F,
        "feed",
        0.016,
        0.5,
        (-0.935, 0.25, -0.2),
        M.steel,
        verts = 6
    );
    fcyl!(
        F,
        "feed_tap",
        0.024,
        0.05,
        (-0.935, 0.3, -0.2),
        M.steel,
        verts = 8
    );
    settle(&[join_all("trough_far")])
}
