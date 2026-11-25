//! The garden hammock (3x1): a canvas sling 2 m long hung from a curved cedar stand, rocking for the resident
//! (2026-10-21: 2x1, its sling 1.44 m, grown-ups lying in it with the knees up: M-packs-06, Q-07).

use super::common::M;
use artkit::forms::{rect, rounded_rect};
use artkit::json::float_repr;
use artkit::math::{Axis, P3, atan, degrees, pow};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fsweep, rock, settle, soft, torus};

const HOOK: (f64, f64) = (1.38, 1.1); // the hooks' (x, height): the sling swings about the line between them
// the sling's low point, its ends' height and its ends' x
const LOW: f64 = 0.49;
const END: f64 = 0.9;
const XS: f64 = 1.0;
const PILLOW: f64 = 0.76; // the pillow's x (towards the left end): `furniture.json` `head`

/// The sling's height at x: a parabola from LOW in the middle to END at the spreader bars.
fn _sag(x: f64) -> f64 {
    let u = x / XS;
    LOW + (END - LOW) * u * u
}

pub fn garden_hammock() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        fsweep!(
            f,
            "beam",
            &[(-1.35, 0.05, 0), (1.35, 0.05, 0)],
            0.0,
            M.cedar_wood,
            profile = rounded_rect(0.08, 0.1, 0.015, 2),
            up = (0, 0, 1)
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                f,
                format!("foot{s}"),
                0.1,
                0.05,
                0.62,
                (sf * 1.16, 0.035, 0.0),
                M.cedar_wood_dark,
                bevel = 0.012,
                bevel_segments = 1
            );
            for z in [-0.3, 0.3] {
                fbox!(
                    f,
                    format!("foot_pad{s}{}", float_repr(z)),
                    0.1,
                    0.012,
                    0.06,
                    (sf * 1.16, 0.006, z),
                    M.cedar_wood_dark
                );
            }
            // the laminated arm, curving up in one piece from the beam's end to its hook
            fsweep!(
                f,
                format!("arm{s}"),
                &[
                    (sf * 1.08, 0.04, 0),
                    (sf * 1.24, 0.4, 0),
                    (sf * 1.35, 0.78, 0),
                    (sf * 1.41, 1.18, 0)
                ],
                0.0,
                M.cedar_wood,
                profile = rounded_rect(0.07, 0.08, 0.02, 2),
                up = (0, 0, 1),
                resolution = 4,
                scale = [1.15, 1.0, 0.95, 0.85]
            );
            // (the hook's eye in the stand's dark wood, as the old part's: steel would be a draw of its own)
            fsweep!(
                f,
                format!("eye_bolt{s}"),
                &[(sf * 1.4, 1.17, 0), (sf * HOOK.0, HOOK.1 + 0.025, 0)],
                0.006,
                M.cedar_wood_dark,
                verts = 6
            );
            torus!(
                format!("hook{s}"),
                0.025,
                0.006,
                at = f.at(sf * HOOK.0, HOOK.1, 0),
                material = M.cedar_wood_dark,
                rot = (90, 0, 0),
                major_segments = 10,
                minor_segments = 4
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The sling: striped canvas sagging between two spreader bars in one sweep a stripe, hemmed along its sides, roped
    // to a ring at each end and on to the hooks, a pillow at one end.
    let sling = new_parts(|| {
        let width = 0.62;
        let stripes = [M.canvas, M.stripe, M.canvas, M.canvas, M.stripe, M.canvas];
        let zw = width / stripes.len() as f64;
        let xs: Vec<f64> = (0..17).map(|i| -XS + 2.0 * XS * i as f64 / 16.0).collect();
        for (j, m) in stripes.into_iter().enumerate() {
            let zc = -width / 2.0 + zw * (j as f64 + 0.5);
            let pts: Vec<(f64, f64, f64)> = xs.iter().map(|&x| (x, _sag(x), zc)).collect();
            fsweep!(
                f,
                format!("sling{j}"),
                &pts,
                0.0,
                m,
                profile = rect(0.012, zw + 0.002),
                up = (0, 0, 1)
            );
        }
        for z in [-width / 2.0, width / 2.0] {
            let pts: Vec<(f64, f64, f64)> = xs.iter().map(|&x| (x, _sag(x) + 0.003, z)).collect();
            fsweep!(
                f,
                format!("hem{}", float_repr(z)),
                &pts,
                0.008,
                M.canvas,
                verts = 5
            );
        }
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                format!("spreader{s}"),
                &[(sf * XS, END, -0.34), (sf * XS, END, 0.34)],
                0.02,
                M.cedar_wood_dark,
                verts = 8
            );
            let ring = (sf * (HOOK.0 - 0.12), HOOK.1 - 0.08, 0);
            torus!(
                format!("rope_ring{s}"),
                0.025,
                0.007,
                at = f.at(ring.0, ring.1, ring.2),
                material = M.rope,
                rot = (90, 0, 0),
                major_segments = 10,
                minor_segments = 4
            );
            for k in 0..7 {
                let z = -0.3 + k as f64 * 0.1;
                fsweep!(
                    f,
                    format!("cord{s}{k}"),
                    &[
                        (sf * XS, END + 0.01, z),
                        (ring.0 - sf * 0.02, ring.1 - 0.01, z * 0.08)
                    ],
                    0.004,
                    M.rope,
                    verts = 4
                );
            }
            fsweep!(
                f,
                format!("rope{s}"),
                &[ring, (sf * HOOK.0, HOOK.1 - 0.02, 0)],
                0.008,
                M.rope,
                verts = 5
            );
        }
        let y = _sag(-PILLOW) + 0.0125; // the sling's top and slope under the pillow
        let slope = degrees(atan(2.0 * (END - LOW) * PILLOW / pow(XS, 2.0)));
        soft!(
            f,
            "pillow",
            0.36,
            0.11,
            0.3,
            (-PILLOW, y + 0.045, 0.0),
            M.pillow,
            rot = (0, slope, 0),
            round = 0.045,
            puff = 0.02,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
    });
    let sling = hinged(join(&sling, "sling", None), f.at(0, HOOK.1, 0));
    let objs = settle!(centre = false);
    rock!(sling, "sway", 8.0, seconds = 2.4, axis = Axis::X);
    objs
}
