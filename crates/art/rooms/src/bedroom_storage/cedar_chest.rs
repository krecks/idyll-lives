//! The cedar chest.

use super::common::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{Axis, P3, PI, sin};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, jitter, loft, soft, swing};
use artkit::{kit, settle};

// --- the blanket chest: a cedar box on a plinth with iron straps and side handles; the lid a part --------
/// The blanket chest.
pub fn cedar_chest() -> Vec<Obj> {
    let (w, d, y0, y1) = (0.9, 0.46, 0.06, 0.455);
    let t = 0.025;
    let still = new_parts(|| {
        // the plinth: a moulded band on bracket feet, cut away in an arch between them
        soft!(
            F,
            "plinth_band",
            0.94,
            0.025,
            0.5,
            (0, 0.0475, 0),
            M.cedar2,
            round = 0.008,
            round_segments = 1
        );
        for s in [-1, 1] {
            let curve: Vec<(f64, f64)> = (0..17)
                .map(|i| {
                    (
                        0.47 - 0.94 * i as f64 / 16.0,
                        0.012 + 0.02 * sin(PI * i as f64 / 16.0),
                    )
                })
                .collect();
            let mut arch = vec![
                (0.47, 0.0),
                (0.47, 0.036),
                (-0.47, 0.036),
                (-0.47, 0.0),
                (-0.39, 0.0),
            ];
            arch.extend(curve.iter().rev().filter(|p| p.0.abs() < 0.39));
            arch.push((0.39, 0.0));
            arch.reverse();
            fextrude!(
                F,
                format!("plinth_x{s}"),
                &arch,
                0.02,
                (0, 0, s as f64 * 0.24),
                M.cedar2,
                bevel = 0.003,
                bevel_segments = 1
            );
            let mut side = vec![
                (0.25, 0.0),
                (0.25, 0.036),
                (-0.25, 0.036),
                (-0.25, 0.0),
                (-0.17, 0.0),
                (-0.12, 0.022),
                (0.12, 0.022),
                (0.17, 0.0),
            ];
            side.reverse();
            fextrude!(
                F,
                format!("plinth_z{s}"),
                &side,
                0.02,
                (s as f64 * 0.46, 0, 0),
                M.cedar2,
                plane = Facing::Side,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("wall_z{s}"),
                w,
                y1 - y0,
                t,
                (0, (y0 + y1) / 2.0, sf * (d - t) / 2.0),
                M.cedar,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("wall_x{s}"),
                t,
                y1 - y0,
                d - 2.0 * t,
                (sf * (w - t) / 2.0, (y0 + y1) / 2.0, 0),
                M.cedar,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fbox!(
            F,
            "floor",
            w - 2.0 * t,
            0.02,
            d - 2.0 * t,
            (0, y0 + 0.01, 0),
            M.cedarin
        );
        // two raised panels in frames on the front
        for s in [-1, 1] {
            kit::shaker!(
                F,
                format!("front_panel{s}"),
                0.34,
                0.27,
                (s as f64 * 0.2, 0.255, d / 2.0 + 0.006),
                0.012,
                M.cedar2,
                rail = 0.035,
                panel = M.cedar,
                raised = true,
                recess = 0.003
            );
        }
        // iron corner brackets with rivets, and a drop handle each side on a plate
        for s in [-1, 1] {
            let sf = s as f64;
            for y in [0.09, 0.425] {
                fbox!(
                    F,
                    format!("corner_front{s}{}", float_repr(y)),
                    0.05,
                    0.05,
                    0.004,
                    (sf * (w / 2.0 - 0.025), y, d / 2.0 + 0.002),
                    M.iron,
                    bevel = 0.0015,
                    bevel_segments = 1
                );
                fbox!(
                    F,
                    format!("corner_side{s}{}", float_repr(y)),
                    0.004,
                    0.05,
                    0.05,
                    (sf * (w / 2.0 + 0.002), y, d / 2.0 - 0.025),
                    M.iron,
                    bevel = 0.0015,
                    bevel_segments = 1
                );
                fsphere!(
                    F,
                    format!("rivet{s}{}", float_repr(y)),
                    0.005,
                    (sf * (w / 2.0 - 0.02), y, d / 2.0 + 0.004),
                    M.iron,
                    segments = 6,
                    rings = 3,
                    scale = (1, 1, 0.5)
                );
            }
            let hx = sf * (w / 2.0 + 0.006);
            fbox!(
                F,
                format!("handle_plate{s}"),
                0.008,
                0.05,
                0.13,
                (hx, 0.36, 0),
                M.iron,
                bevel = 0.002,
                bevel_segments = 1
            );
            fsweep!(
                F,
                format!("handle{s}"),
                &kit::fillet(
                    &[
                        (sf * (w / 2.0 + 0.014), 0.365, -0.055),
                        (sf * (w / 2.0 + 0.024), 0.3, -0.055),
                        (sf * (w / 2.0 + 0.024), 0.3, 0.055),
                        (sf * (w / 2.0 + 0.014), 0.365, 0.055)
                    ],
                    0.014,
                    3
                ),
                0.006,
                M.iron,
                verts = 6
            );
        }
        // the lock: a brass escutcheon with a black keyhole, under the lid's front edge
        soft!(
            F,
            "latch",
            0.07,
            0.06,
            0.006,
            (0, 0.415, d / 2.0 + 0.003),
            M.brass,
            round = 0.002,
            round_segments = 1
        );
        fcyl!(
            F,
            "keyhole",
            0.006,
            0.004,
            (0, 0.42, d / 2.0 + 0.006),
            M.iron,
            rot = (90, 0, 0),
            verts = 8
        );
        fbox!(
            F,
            "keyhole_slot",
            0.004,
            0.012,
            0.004,
            (0, 0.411, d / 2.0 + 0.006),
            M.iron
        );
        let b = soft!(
            F,
            "blanket",
            w - 2.0 * t - 0.02,
            0.1,
            d - 2.0 * t - 0.02,
            (0, y0 + 0.08, 0),
            M.cloth3,
            round = 0.03,
            puff = 0.012,
            flat = 3
        );
        jitter!(b, 0.006, size = 0.1, seed = 4, up = 0.5);
    });
    join(&still, "cedarChest", Some(P3::ZERO));
    // the lid, hinged along its back edge: it lifts while a resident takes out a blanket (`lid_open`)
    let parts = new_parts(|| {
        // (its underside 2 mm below the edge band's, not in its plane: seen when the lid is up)
        soft!(
            F,
            "lid",
            0.94,
            0.047,
            0.5,
            (0, 0.4765, 0),
            M.cedar,
            round = 0.012,
            round_segments = 2
        );
        soft!(
            F,
            "lid_edge",
            0.96,
            0.012,
            0.52,
            (0, 0.461, 0),
            M.cedar2,
            round = 0.005,
            round_segments = 1
        );
        let b = 0.04;
        fextrude!(
            F,
            "lid_frame",
            &rounded_rect(0.84, 0.4, 0.01, 1),
            0.006,
            (0, 0.5, 0),
            M.cedar2,
            plane = Facing::Top,
            holes = [rounded_rect(0.74, 0.3, 0.01, 1)],
            bevel = 0.002,
            bevel_segments = 1
        );
        // the raised field inside it, chamfered up to its flat top (the chest's top, 0.505)
        let secs: Vec<Vec<P3>> = [
            (0.499, 0.369, 0.149),
            (0.5005, 0.369, 0.149),
            (0.505, 0.369 - b, 0.149 - b),
        ]
        .iter()
        .map(|&(y, k, k2)| {
            [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
                .iter()
                .map(|&(u, v)| F.at(u * k, y, v * k2))
                .collect()
        })
        .collect();
        loft!("lid_panel", &secs, material = M.cedar, smooth = false);
    });
    let lid = hinged(join(&parts, "lid", None), F.at(0, 0.455, -0.25));
    let objs = settle!(centre = false);
    swing!(lid, "lid_open", -95.0, seconds = 0.9, axis = Axis::X);
    objs
}
