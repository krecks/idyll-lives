//! The fire station's fire pole and hose rack.

use super::common::{F, WALL, own_palette};
use artkit::far::{is_low, low};
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{TAU, cos, radians, sin};
use artkit::parts::{join_all, new_parts, settle_all};
use artkit::scene::{Obj, meshes};
use artkit::shapes::sides;
use artkit::{fbox, fcyl, fsweep, revolve, settle};

own_palette! { POLE:
    body = ("#C9A04A", metal, roughness(0.16)),   // the polished brass pole
    mat = ("#2E2F33", matte),         // the rubber landing mat
}

own_palette! { HOSE:
    body = ("#B8221E", gloss),        // the rack's red steel
    hose = ("#C9473A", satin),        // the red hose
    hose2 = ("#E6DFCC", satin),       // the white one
}

// --- the fire pole: 1x1, a brass pole from floor to ceiling on a round landing mat -------------------------------
pub const CEILING: f64 = 2.8; // where the ceiling things hang from: the pole runs up to it, through the watch room's floor

pub fn fire_pole() -> Vec<Obj> {
    let m = &POLE;
    // the landing mat: a thick round of rubber, its edge bevelled, rings moulded in its top, a hole for the pole
    let mut prof = vec![
        (0.04, 0.0),
        (0.4, 0.0),
        (0.42, 0.012),
        (0.417, 0.028),
        (0.4, 0.036),
    ];
    if !is_low() {
        for r in [0.35, 0.28, 0.21, 0.14] {
            prof.extend([
                (r + 0.012, 0.036),
                (r + 0.006, 0.042),
                (r - 0.006, 0.042),
                (r - 0.012, 0.036),
            ]);
        }
    }
    prof.extend([(0.06, 0.036), (0.04, 0.03), (0.04, 0.0)]);
    revolve!(
        "mat",
        &prof,
        at = F.at(0, 0, 0),
        material = m.mat,
        segments = low(44, 16)
    );
    // the pole on its floor flange, through the ceiling's collar
    let r = 0.0375;
    revolve!(
        "flange",
        &[
            (0.0, 0.03),
            (0.08, 0.03),
            (0.08, 0.04),
            (0.06, 0.05),
            (0.045, 0.08),
            (0.0, 0.08)
        ],
        at = F.at(0, 0, 0),
        material = m.brass,
        segments = low(24, 10)
    );
    fcyl!(
        F,
        "pole",
        r,
        CEILING - 0.1,
        (0, 0.05 + (CEILING - 0.1) / 2.0, 0),
        m.body,
        verts = low(sides(r, 16, 64), 8)
    );
    revolve!(
        "collar",
        &[
            (0.0, CEILING - 0.12),
            (0.045, CEILING - 0.12),
            (0.06, CEILING - 0.08),
            (0.06, CEILING - 0.03),
            (0.13, CEILING - 0.02),
            (0.13, CEILING),
            (0.0, CEILING)
        ],
        at = F.at(0, 0, 0),
        material = m.brass,
        segments = low(24, 10)
    );
    join_all(if is_low() { "firePole_far" } else { "firePole" });
    settle_all();
    meshes()
}

// --- the hose rack: 1x1 on the wall, two hoses coiled over its saddles -------------------------------------------
type Point = (f64, f64, f64);

/// A hose coiled over a saddle: `turns` loops hanging from the saddle's top at height `top`, `rx` by `ry` round,
/// each a hose's width further from the wall than the last, from depth z0; returns the coil's (start, end) points.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn coil(
    name: &str,
    cx: f64,
    top: f64,
    rx: f64,
    ry: f64,
    turns: f64,
    r: f64,
    material: Mat,
    z0: f64,
    steps: usize,
) -> (Point, Point) {
    let mut pts = Vec::new();
    let n = artkit::py::int(turns * steps as f64);
    for i in 0..n + 1 {
        let a = TAU * i as f64 / steps as f64;
        pts.push((
            cx + rx * sin(a),
            top - ry * (1.0 - cos(a)),
            z0 + 2.05 * r * i as f64 / steps as f64,
        ));
    }
    fsweep!(F, name, &pts, r, material, verts = low(6, 4));
    (pts[0], pts[pts.len() - 1])
}

pub fn hose_rack() -> Vec<Obj> {
    let m = &HOSE;
    let z = WALL;
    new_parts(|| {
        // the rail on the wall, bolted at its ends, and the two saddles standing off it
        fbox!(
            F,
            "rail",
            0.88,
            0.1,
            0.025,
            (0, 1.62, z + 0.0125),
            m.body,
            bevel = low(0.006, 0.0),
            bevel_segments = 1
        );
        for s in [-1, 1] {
            for dy in [-0.025, 0.025] {
                fcyl!(
                    F,
                    format!("bolt{s}{}", float_repr(dy)),
                    0.008,
                    0.008,
                    (s as f64 * 0.41, 1.62 + dy, z + 0.029),
                    m.chrome,
                    rot = (90, 0, 0),
                    verts = 6
                );
            }
        }
        let mut tops: Vec<(usize, (Point, Point, f64, Mat))> = Vec::new();
        for (k, (x, hose, turns)) in [(-0.22, m.hose, 4.0), (0.22, m.hose2, 3.5)]
            .into_iter()
            .enumerate()
        {
            // a saddle: an arm out from the rail, curving up at its end, a curved plate the coil hangs on
            fbox!(
                F,
                format!("arm{k}"),
                0.05,
                0.04,
                0.26,
                (x, 1.6, z + 0.155),
                m.body,
                bevel = low(0.005, 0.0),
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("lip{k}"),
                0.05,
                0.08,
                0.02,
                (x, 1.63, z + 0.29),
                m.body,
                bevel = low(0.005, 0.0),
                bevel_segments = 1
            );
            let arc: Vec<Point> = (0..181)
                .step_by(low(30, 90))
                .map(|a| {
                    (
                        x + 0.11 * cos(radians(a as f64)),
                        1.58 + 0.04 * sin(radians(a as f64)),
                        z + 0.155,
                    )
                })
                .collect();
            fsweep!(
                F,
                format!("saddle{k}"),
                &arc,
                0,
                m.body,
                profile = [
                    (-0.004, -0.13),
                    (0.004, -0.13),
                    (0.004, 0.13),
                    (-0.004, 0.13)
                ],
                up = (0, 0, 1)
            );
            let r = 0.02;
            let (a, b) = coil(
                &format!("coil{k}"),
                x,
                1.62 + 0.004 + r,
                0.18,
                0.21,
                turns,
                r,
                hose,
                z + 0.05,
                low(16, 8),
            );
            tops.push((k, (a, b, x, hose)));
        }
        // the ends: a brass coupling hanging from the back turn, and a branch (the nozzle) from the front one
        for (k, (_a, b, _x, hose)) in tops {
            let end = (b.0, b.1, b.2);
            let drop = [
                end,
                (end.0 + 0.01, end.1 - 0.12, end.2 + 0.01),
                (end.0, end.1 - 0.26, end.2 + 0.02),
            ];
            fsweep!(
                F,
                format!("tail{k}"),
                &drop,
                0.02,
                hose,
                verts = low(6, 4),
                resolution = low(2, 1)
            );
            let (bx, by, bz) = drop[drop.len() - 1];
            if k == 0 {
                revolve!(
                    format!("branch{k}"),
                    &[
                        (0.0, 0.0),
                        (0.03, 0.0),
                        (0.03, -0.04),
                        (0.024, -0.06),
                        (0.024, -0.2),
                        (0.012, -0.26),
                        (0.012, -0.29),
                        (0.0, -0.29)
                    ],
                    at = F.at(bx, by, bz),
                    material = m.brass,
                    segments = low(12, 6)
                );
                fbox!(
                    F,
                    format!("branch_grip{k}"),
                    0.07,
                    0.1,
                    0.07,
                    (bx, by - 0.1, bz),
                    m.plastic,
                    bevel = low(0.01, 0.0),
                    bevel_segments = 1
                );
            } else {
                revolve!(
                    format!("coupling{k}"),
                    &[
                        (0.0, 0.0),
                        (0.034, 0.0),
                        (0.036, -0.02),
                        (0.036, -0.07),
                        (0.03, -0.08),
                        (0.0, -0.08)
                    ],
                    at = F.at(bx, by, bz),
                    material = m.brass,
                    segments = low(12, 6)
                );
                for j in 0..2 {
                    // its lugs
                    fbox!(
                        F,
                        format!("lug{k}{j}"),
                        0.1,
                        0.012,
                        0.02,
                        (bx, by - 0.04 - 0.02 * j as f64, bz),
                        m.brass
                    );
                }
            }
        }
    });
    join_all(if is_low() { "hoseRack_far" } else { "hoseRack" });
    settle!(centre = false, floor = false);
    meshes()
}
