//! The recovery pod (1x2): a padded lounger (top 0.5) in a shell, a tinted canopy over the head end.

use super::common::{M, lofted};
use artkit::forms::rounded_rect;
use artkit::math::{Axis, P3, TAU, cos, sin};
use artkit::parts::{hinged, join, new_parts, numbered, turn};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, settle, soft, swing, torus};

const CANOPY: (f64, f64, f64, f64, f64) = (0.42, -0.35, 0.4, 0.4, 0.5); // the canopy's rim height, middle depth, half-width, height, half-length
const CANOPY_UP: f64 = 40.0; // how far it stands open at rest (degrees)

/// A compression boot lying along the mattress, toe to the front: a lofted leg sleeve, quilted in rings.
fn boot(f: &GameFrame, x: f64, y: f64, z: f64) {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    let rows = [
        (-0.31, 0.04, 0.035),
        (-0.29, 0.1, 0.055),
        (-0.1, 0.11, 0.06),
        (0.12, 0.1, 0.06),
        (0.24, 0.09, 0.07),
        (0.29, 0.07, 0.065),
        (0.31, 0.02, 0.03),
    ];
    let secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(u, rx, rz)| {
            (0..12)
                .map(|k| P3 {
                    x: bx + rx * cos(TAU * k as f64 / 12.0),
                    y: by - u,
                    z: bz + rz * sin(TAU * k as f64 / 12.0),
                })
                .collect()
        })
        .collect();
    loft!(numbered("boot"), &secs, material = M.pod_boots);
    for u in [-0.2, -0.06, 0.08] {
        torus!(
            numbered("boot_quilt"),
            1.0,
            0.006,
            at = (bx, by - u, bz),
            material = M.pod_boots,
            rot = (90, 0, 0),
            major_segments = 12,
            minor_segments = 3
        )
        .set_scale((0.112, 0.062, 1.0));
    }
}

pub fn recovery_pod() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // the shell: a tub lofted through rounded rings, a glowing strip round its foot
        lofted(
            f,
            "shell",
            0.0,
            0.04,
            0.0,
            &[
                (0.0, 0.39, 0.91),
                (0.02, 0.405, 0.925),
                (0.16, 0.41, 0.93),
                (0.27, 0.4, 0.92),
                (0.29, 0.385, 0.905),
            ],
            M.pod_shell,
            32,
            7.0,
        );
        let strip: Vec<(f64, f64, f64)> = rounded_rect(0.78, 1.82, 0.08, 2)
            .iter()
            .map(|&(x, z)| (x, 0.025, z))
            .collect();
        fsweep!(
            f,
            "glow_strip",
            &strip,
            0.015,
            M.pod_glow,
            verts = 5,
            closed = true
        );
        soft!(
            f,
            "mattress",
            0.68,
            0.132,
            1.66,
            (0, 0.426, 0.04),
            M.pod_pad,
            round = 0.05,
            puff = 0.008,
            piping = 0.006,
            flat = 2
        );
        soft!(
            f,
            "headrest",
            0.5,
            0.1,
            0.3,
            (0, 0.53, -0.7),
            M.pod_head,
            round = 0.04,
            puff = 0.012,
            flat = 2
        );
        for x in [-0.13, 0.13] {
            boot(f, x, 0.56, 0.5);
        }
        fextrude!(
            f,
            "control",
            &rounded_rect(0.3, 0.3, 0.05, 2),
            0.06,
            (0.38, 0.5, 0.55),
            M.pod_shell,
            plane = Facing::Side,
            bevel = 0.015,
            bevel_segments = 1
        );
        fbox!(
            f,
            "control_screen",
            0.004,
            0.16,
            0.2,
            (0.412, 0.54, 0.55),
            M.pod_screen
        );
        fbox!(
            f,
            "control_line",
            0.004,
            0.03,
            0.12,
            (0.414, 0.6, 0.55),
            M.pod_line
        );
        for k in 0..3 {
            fcyl!(
                f,
                numbered("control_key"),
                0.012,
                0.004,
                (0.412, 0.44, 0.49 + 0.06 * k as f64),
                M.pod_line,
                rot = (0, 90, 0),
                verts = 8
            );
        }
        for x in [-0.33, 0.33] {
            fextrude!(
                f,
                numbered("hinge_block"),
                &rounded_rect(0.06, 0.07, 0.02, 2),
                0.06,
                (x, 0.42, -0.86),
                M.pod_shell,
                plane = Facing::Side,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The canopy: a tinted glass half-shell with a white rim, built lowered, hinged at the head end and turned up
    // to stand open at rest; `canopy` lowers it over the resident.
    let (ry, cz, hw, hh, hl) = CANOPY;
    let can = new_parts(|| {
        let shell = fsphere!(
            f,
            "canopy",
            1.0,
            (0, ry, cz),
            M.canopy_glass,
            scale = (hw, hl, hh),
            segments = 28,
            rings = 14
        );
        artkit::edit::delete_verts(shell, |co| co.z < -1e-4);
        artkit::solidify!(shell, 0.008);
        let rim = torus!(
            "canopy_rim",
            1.0,
            0.06,
            at = f.at(0, ry, cz),
            material = M.canopy_rim,
            major_segments = 32,
            minor_segments = 5
        );
        rim.set_scale((hw + 0.005, hl + 0.005, 0.4));
        for x in [-0.33, 0.33] {
            fbox!(
                f,
                numbered("canopy_arm"),
                0.02,
                0.03,
                0.12,
                (x, ry + 0.01, cz - hl + 0.05),
                M.canopy_rim,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
    });
    let can = hinged(join(&can, "canopy", None), f.at(0, ry, cz - hl));
    turn(can, -CANOPY_UP, Axis::X);
    let objs = settle!(centre = false);
    swing!(can, "canopy", CANOPY_UP, seconds = 1.2, axis = Axis::X);
    objs
}
