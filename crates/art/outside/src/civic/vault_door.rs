//! The vault door: see NOTES.md.

use super::common::{F, M, WALL, rod};
use artkit::json::float_repr;
use artkit::math::{TAU, cos, sin};
use artkit::parts::{join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, revolve, settle, torus};

// --- the vault door: 2x1 on a wall, a round steel door a foot thick in its frame, its wheel and dial -------------
pub fn vault_door() -> Vec<Obj> {
    let c = 1.0; // the door's middle, its height
    let r = 0.74;
    let parts = new_parts(|| {
        fbox!(
            F,
            "frame",
            1.92,
            1.92,
            0.05,
            (0, 0.96, WALL + 0.025),
            M.vault_dark,
            bevel = 0.01
        );
        artkit::bevel::eased(revolve!(
            "door",
            &[
                (0.0, 0.0),
                (r, 0.0),
                (r, 0.13),
                (r - 0.03, 0.16),
                (r - 0.12, 0.16),
                (r - 0.14, 0.18),
                (0.0, 0.18)
            ],
            at = F.at(0, c, WALL + 0.05),
            material = M.vault,
            segments = 48,
            rot = (90, 0, 0)
        ));
        artkit::bevel::eased(revolve!(
            "door_rim",
            &[
                (r + 0.06, 0.0),
                (r + 0.06, 0.03),
                (r + 0.03, 0.06),
                (r + 0.005, 0.06),
                (r + 0.005, 0.0)
            ],
            at = F.at(0, c, WALL + 0.05),
            material = M.vault_dark,
            segments = 48,
            rot = (90, 0, 0)
        ));
        for k in 0..16 {
            let a = TAU * k as f64 / 16.0;
            artkit::bevel::eased(fcyl!(
                F,
                format!("bolt{k}"),
                0.022,
                0.02,
                (0.66 * cos(a), c + 0.66 * sin(a), WALL + 0.215), // standing 1.5 cm off the face (WALL + 0.21)
                M.bell,
                rot = (90, 0, 0),
                verts = 10
            ));
        }
        // the wheel: a hub, six spokes ending in knobs, a ring
        let wz = WALL + 0.29;
        artkit::bevel::eased(fcyl!(
            F,
            "hub",
            0.08,
            0.08,
            (0, c, WALL + 0.25),
            M.bell,
            rot = (90, 0, 0),
            verts = 20
        ));
        for k in 0..6 {
            let a = TAU * k as f64 / 6.0 + 0.26;
            rod(
                &format!("spoke{k}"),
                (0.06 * cos(a), c + 0.06 * sin(a), wz),
                (0.38 * cos(a), c + 0.38 * sin(a), wz),
                0.014,
                M.bell,
                10,
            );
            fsphere!(
                F,
                format!("spoke_knob{k}"),
                0.026,
                (0.4 * cos(a), c + 0.4 * sin(a), wz),
                M.bell,
                segments = 12,
                rings = 8
            );
        }
        torus!(
            "wheel",
            0.3,
            0.018,
            at = F.at(0, c, wz),
            material = M.bell,
            rot = (90, 0, 0),
            major_segments = 40,
            minor_segments = 8
        );
        // the dial over it
        artkit::bevel::eased(fcyl!(
            F,
            "dial",
            0.075,
            0.03,
            (0, c + 0.5, WALL + 0.245),
            M.iron,
            rot = (90, 0, 0),
            verts = 24
        ));
        artkit::bevel::eased(fcyl!(
            F,
            "dial_face",
            0.06,
            0.006,
            (0, c + 0.5, WALL + 0.262),
            M.bell,
            rot = (90, 0, 0),
            verts = 24
        ));
        fbox!(
            F,
            "dial_mark",
            0.008,
            0.03,
            0.004,
            (0, c + 0.59, WALL + 0.232),
            M.leaflet4
        );
        // the hinge on the right
        artkit::bevel::eased(fcyl!(
            F,
            "hinge",
            0.06,
            0.9,
            (-0.86, c, WALL + 0.13),
            M.vault_dark,
            verts = 16
        ));
        for y in [c - 0.3, c + 0.3] {
            fbox!(
                F,
                format!("hinge_arm{}", float_repr(y)),
                0.2,
                0.12,
                0.08,
                (-0.74, y, WALL + 0.15),
                M.vault_dark,
                bevel = 0.01
            );
        }
    });
    join(&parts, "vault", Some((0, 0, 0).into()));
    settle!(centre = false, floor = false);
    meshes()
}
