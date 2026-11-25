//! H3: the telescope: a refractor on a white tripod with an eyepiece tray. The tube, with its dew shield, finder and
//! eyepiece, is its own part about the mount's axle, and tilts up 35° while a resident stargazes (`aim`).

use super::M;
use super::common::{P, axis};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit;
use artkit::math::{Axis, P3, TAU, cos, cosd, sin, sind};
use artkit::parts::{hinged, join, new_parts, settle};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, revolve, swing};

/// Up the tube, towards the objective (back and up).
fn tube_dir() -> P {
    axis(55.0, 180.0, 0.0)
}
const TUBE_C: P = (0.0, 1.18, -0.012);
const AXLE: P = (0.0, 1.12, 0.0);

/// Game point `t` along the tube from its middle (towards the objective) and `off` above its axis.
fn on_tube(t: f64, off: f64) -> P {
    let d = tube_dir();
    // square to the tube, upwards (towards the front)
    let normal = (0.0, -d.2, d.1);
    let k = if d.2 < 0.0 { 1.0 } else { -1.0 };
    (
        TUBE_C.0 + t * d.0 + off * normal.0 * k,
        TUBE_C.1 + t * d.1 + off * normal.1 * k,
        TUBE_C.2 + t * d.2 + off * normal.2 * k,
    )
}

pub fn telescope() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        // the tripod: three legs, each an outer tube with a lock where the inner one slides out, a rubber foot
        for (k, yaw) in [180.0, 60.0, -60.0].into_iter().enumerate() {
            let d = (sind(yaw), cosd(yaw));
            let (top, foot) = (
                (0.045 * d.0, 0.97, 0.045 * d.1),
                (0.36 * d.0, 0.02, 0.36 * d.1),
            );
            let along = |t: f64| {
                (
                    top.0 + (foot.0 - top.0) * t,
                    top.1 + (foot.1 - top.1) * t,
                    top.2 + (foot.2 - top.2) * t,
                )
            };
            let mid = along(0.55);
            fsweep!(
                f,
                format!("leg{k}"),
                &[top, mid],
                0,
                M.tripod,
                profile = rounded_rect(0.034, 0.024, 0.008, 1),
                up = (1, 0, 0)
            );
            kit::tube!(
                f,
                format!("leg_inner{k}"),
                &[mid, foot],
                0.01,
                M.tripod,
                verts = 8,
                round = 1
            );
            fsweep!(
                f,
                format!("leg_lock{k}"),
                &[along(0.53), mid],
                0,
                M.foot,
                profile = rounded_rect(0.042, 0.032, 0.01, 1),
                up = (1, 0, 0)
            );
            fsphere!(
                f,
                format!("foot{k}"),
                0.018,
                foot,
                M.foot,
                segments = 8,
                rings = 5,
                scale = (1, 1, 0.8)
            );
            kit::tube!(
                f,
                format!("spreader{k}"),
                &[(0.0, 0.45, 0.0), (0.17 * d.0, 0.5, 0.17 * d.1)],
                0.006,
                M.tripod,
                verts = 6,
                round = 1
            );
        }
        // the eyepiece tray on the spreaders: a three-lobed plate with holes, two eyepieces in it
        let lobes: Vec<(f64, f64)> = (90..450)
            .step_by(60)
            .map(|a| (0.13 * cosd(a as f64), 0.13 * sind(a as f64)))
            .collect();
        let tray = smooth_outline(&lobes, 2, true);
        let holes: Vec<Vec<(f64, f64)>> = [90.0, 210.0]
            .iter()
            .map(|&a| {
                (0..8)
                    .map(|i| {
                        (
                            0.07 * cosd(a) + 0.017 * cos(TAU * i as f64 / 8.0),
                            0.07 * sind(a) + 0.017 * sin(TAU * i as f64 / 8.0),
                        )
                    })
                    .collect()
            })
            .collect();
        fextrude!(
            f,
            "tray",
            &tray,
            0.012,
            (0, 0.456, 0),
            M.tripod,
            holes = holes,
            plane = Facing::Top,
            bevel = 0.003,
            bevel_segments = 1
        );
        for a in [330, 210] {
            let af = a as f64;
            revolve!(
                format!("eyepiece{a}"),
                &[
                    (0.0, 0.0),
                    (0.015, 0.0),
                    (0.015, 0.03),
                    (0.019, 0.032),
                    (0.019, 0.05),
                    (0.016, 0.055),
                    (0.0, 0.055)
                ],
                at = f.at(0.07 * cosd(af), 0.45, 0.07 * sind(af)),
                material = M.foot,
                segments = 10
            );
        }
        // the head: a hub the legs hinge from, an alt-azimuth mount, a fork, the axle's lock knob
        fcyl!(
            f,
            "hub",
            0.065,
            0.05,
            (0, 0.98, 0),
            M.tripod,
            verts = 16,
            bevel = 0.01,
            bevel_segments = 1
        );
        for (k, yaw) in [180.0, 60.0, -60.0].into_iter().enumerate() {
            let d = (sind(yaw), cosd(yaw));
            fcyl!(
                f,
                format!("hinge{k}"),
                0.01,
                0.05,
                (0.06 * d.0, 0.965, 0.06 * d.1),
                M.chrome,
                rot = (0.0, 90.0, -yaw),
                verts = 8
            );
        }
        fbox!(
            f,
            "head",
            0.1,
            0.08,
            0.1,
            (0, 1.04, 0),
            M.tripod,
            bevel = 0.015
        );
        fextrude!(
            f,
            "fork",
            &[
                (-0.035, 0.0),
                (0.035, 0.0),
                (0.035, 0.09),
                (0.0, 0.12),
                (-0.035, 0.09)
            ],
            0.03,
            (0.065, 1.05, 0),
            M.tripod,
            plane = Facing::Side,
            bevel = 0.006,
            bevel_segments = 1
        );
        revolve!(
            "axle_knob",
            &[
                (0.0, 0.0),
                (0.012, 0.0),
                (0.012, 0.01),
                (0.022, 0.014),
                (0.022, 0.026),
                (0.016, 0.03),
                (0.0, 0.03)
            ],
            at = f.at(0.08, 1.12, 0),
            material = M.foot,
            segments = 12,
            rot = (0, -90, 0)
        );
        // the slow-motion cable and its knob
        kit::tube!(
            f,
            "slow_motion",
            &[(0.03, 1.02, 0.05), (0.05, 0.94, 0.11), (0.06, 0.85, 0.16)],
            0.004,
            M.chrome,
            verts = 6,
            round = 3
        );
        fcyl!(
            f,
            "slow_knob",
            0.012,
            0.03,
            (0.06, 0.835, 0.168),
            M.foot,
            rot = (-30, 0, 0),
            verts = 10
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // Blender: the tube's axis, up 35° towards the back
    let tube_rot = (-55.0, 0.0, 0.0);
    let t = new_parts(|| {
        // the optical tube: a lofted body, its dew shield, the lens cell; tube rings on a dovetail
        fcyl!(
            f,
            "tube",
            0.06,
            0.84,
            (0, TUBE_C.1, TUBE_C.2),
            M.tube,
            rot = tube_rot,
            verts = 24,
            bevel = 0.003
        );
        let at = |t: f64, off: f64| {
            let p = on_tube(t, off);
            f.at(p.0, p.1, p.2)
        };
        revolve!(
            "dew_shield",
            &[
                (0.058, -0.08),
                (0.066, -0.08),
                (0.0685, -0.075),
                (0.0685, 0.08),
                (0.065, 0.08),
                (0.063, 0.074),
                (0.063, -0.074),
                (0.058, -0.074)
            ],
            at = at(0.42, 0.0),
            material = M.foot,
            segments = 24,
            rot = (180.0 + tube_rot.0, 0.0, 0.0)
        );
        fcyl!(
            f,
            "objective",
            0.058,
            0.004,
            on_tube(0.44, 0.0),
            M.optic,
            rot = tube_rot,
            verts = 24,
            smooth = false
        );
        for (k, t) in [0.1, -0.15].into_iter().enumerate() {
            revolve!(
                format!("ring{k}"),
                &[
                    (0.06, -0.015),
                    (0.068, -0.015),
                    (0.07, -0.01),
                    (0.07, 0.01),
                    (0.068, 0.015),
                    (0.06, 0.015)
                ],
                at = at(t, 0.0),
                material = M.chrome,
                segments = 20,
                rot = (180.0 + tube_rot.0, 0.0, 0.0)
            );
            fbox!(
                f,
                format!("ring_bolt{k}"),
                0.018,
                0.02,
                0.018,
                on_tube(t, 0.075),
                M.chrome,
                rot = tube_rot,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "dovetail",
            0.04,
            0.32,
            0.016,
            on_tube(-0.025, -0.075),
            M.foot,
            rot = tube_rot,
            bevel = 0.003,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "cell",
            0.05,
            0.05,
            on_tube(-0.445, 0.0),
            M.foot,
            rot = tube_rot,
            verts = 18,
            bevel = 0.003
        );
        // the focuser: a drawtube with two knobs, the diagonal and the eyepiece
        fcyl!(
            f,
            "focuser",
            0.022,
            0.07,
            on_tube(-0.5, 0.0),
            M.chrome,
            rot = tube_rot,
            verts = 12
        );
        fbox!(
            f,
            "focus_box",
            0.04,
            0.04,
            0.035,
            on_tube(-0.47, -0.04),
            M.foot,
            rot = tube_rot,
            bevel = 0.005,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let p = on_tube(-0.47, -0.045);
            fcyl!(
                f,
                format!("focus_knob{s}"),
                0.014,
                0.014,
                (s as f64 * 0.032, p.1, p.2),
                M.foot,
                rot = (0, 90, 0),
                verts = 10,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fcyl!(
            f,
            "diagonal",
            0.0175,
            0.07,
            (0, 0.946, 0.41),
            M.foot,
            rot = (35.0, 0.0, 0.0),
            verts = 12
        );
        fcyl!(
            f,
            "eyecup",
            0.02,
            0.02,
            (0, 0.975, 0.43),
            M.foot,
            rot = (35.0, 0.0, 0.0),
            verts = 12
        );
        // the finder scope on its two-ring bracket
        fcyl!(
            f,
            "finder",
            0.0175,
            0.22,
            on_tube(0.05, 0.11),
            M.tube,
            rot = tube_rot,
            verts = 12,
            bevel = 0.002
        );
        fcyl!(
            f,
            "finder_lens",
            0.02,
            0.025,
            on_tube(0.16, 0.11),
            M.foot,
            rot = tube_rot,
            verts = 12
        );
        for (k, t) in [0.0, 0.1].into_iter().enumerate() {
            fbox!(
                f,
                format!("finder_foot{k}"),
                0.016,
                0.04,
                0.012,
                on_tube(t, 0.08),
                M.chrome,
                rot = tube_rot,
                bevel = 0.003,
                bevel_segments = 1
            );
            revolve!(
                format!("finder_ring{k}"),
                &[
                    (0.0175, -0.005),
                    (0.023, -0.005),
                    (0.023, 0.005),
                    (0.0175, 0.005)
                ],
                at = at(t, 0.11),
                material = M.chrome,
                segments = 12,
                rot = (180.0 + tube_rot.0, 0.0, 0.0)
            );
        }
        fbox!(
            f,
            "saddle",
            0.06,
            0.05,
            0.1,
            (0, 1.12, -0.01),
            M.foot,
            rot = tube_rot,
            bevel = 0.006
        );
    });
    let tube = hinged(join(&t, "tube", None), f.at(AXLE.0, AXLE.1, AXLE.2));
    // on its Model's `centre`: the back leg on the wall face
    settle(&meshes());
    swing!(tube, "aim", 35.0, seconds = 1.2, axis = Axis::X);
    meshes()
}
