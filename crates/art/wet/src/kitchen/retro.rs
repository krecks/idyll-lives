//! The 1950s mint fridge: one rounded body cut open at the front, a rounded door, and a chrome lever that turns out
//! first and unlatches the door (`attach`: the lever rides on the door).

use super::common::{apple, gasket, kick, soft};
use super::cut::cut_shell;
use super::fridge::bottle;
use super::{DOOR_SECONDS, LEVER_SECONDS, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{Axis, P3};
use artkit::motion::{Ease, attach};
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, ring, soft, swing};

/// A 1950s mint fridge: one rounded body, its top rolling over, on a plinth with a chrome kick grille; a single
/// rounded door on chrome hinges with a trim line, a chrome nameplate and a chrome lever handle. Inside, an ice box
/// with a flap, shelves with chrome rails, a crisper and food; on the door's inside, railed shelves with bottles.
#[allow(clippy::approx_constant)] // (0.318 is a depth, not 1/π)
pub fn fridge_retro() -> Vec<Obj> {
    let fr = GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            fr,
            "plinth",
            0.66,
            0.08,
            0.60,
            (0, 0.04, -0.05),
            M.mint2,
            bevel = 0.010
        );
        kick(&fr, "kick", 0.56, 0.05, 0.042, 0.255, Some(M.chrome), 7);
        // the body: rounded-square rings, full from the plinth up to where the top rolls over towards the back
        let b = fr.at(0, 0, -0.04);
        let (bx, by) = (b.x, b.y);
        let rings = [
            (0.08, 0.35, 0.31),
            (0.1, 0.36, 0.32),
            (1.54, 0.36, 0.32),
            (1.585, 0.354, 0.314),
            (1.615, 0.34, 0.3),
            (1.633, 0.315, 0.275),
            (1.64, 0.27, 0.23),
        ];
        let sections: Vec<Vec<P3>> = rings
            .iter()
            .map(|&(y, rx, rz)| ring!(rx, rz, z = y, verts = 32, square = 7.0, at = (bx, by)))
            .collect();
        let shell = loft!("shell", &sections, material = M.mint);
        // the inside cut out through the front, lined (the cut's faces take the liner)
        let cavity = fbox!(fr, "cavity", 0.62, 1.42, 0.62, (0, 0.85, -0.01), M.liner);
        cut_shell(shell, cavity);
        soft(shell);
        cavity.remove();
        // a breaker strip lining the opening's mouth: the cut's rim, where the shell's front rounds back, is eased
        fextrude!(
            fr,
            "breaker",
            &rounded_rect(0.62, 1.42, 0.003, 1),
            0.04,
            (0, 0.85, 0.259),
            M.liner,
            holes = [rounded_rect(0.586, 1.386, 0.003, 1)],
            bevel = 0.003,
            bevel_segments = 1
        );
        // chrome hinges on the door's hinge side
        for y in [0.2, 1.47] {
            fcyl!(
                fr,
                format!("hinge{}", float_repr(y)),
                0.011,
                0.07,
                (-0.348, y, 0.30),
                M.chrome,
                verts = 10
            );
        }
        // the ice box across the top with its flap and latch, its drip tray, two shelves with chrome rails, a crisper
        fbox!(
            fr,
            "icebox",
            0.5,
            0.2,
            0.4,
            (0, 1.42, -0.11),
            M.liner,
            bevel = 0.01,
            bevel_segments = 1
        );
        soft!(
            fr,
            "icebox_flap",
            0.48,
            0.18,
            0.014,
            (0, 1.42, 0.097),
            M.shelf,
            round = 0.006,
            round_segments = 1
        );
        fbox!(
            fr,
            "icebox_latch",
            0.06,
            0.02,
            0.016,
            (0, 1.35, 0.108),
            M.chrome,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            fr,
            "drip_tray",
            0.56,
            0.02,
            0.4,
            (0, 1.29, -0.11),
            M.shelf,
            bevel = 0.004,
            bevel_segments = 1
        );
        for (i, y) in [0.45, 0.80].into_iter().enumerate() {
            fbox!(
                fr,
                format!("shelf{i}"),
                0.6,
                0.008,
                0.4,
                (0, y, -0.11),
                M.shelf
            );
            fsweep!(
                fr,
                format!("shelf_rail{i}"),
                &[(-0.3, y + 0.004, 0.09), (0.3, y + 0.004, 0.09)],
                0.006,
                M.chrome,
                verts = 6
            );
        }
        fbox!(
            fr,
            "crisper",
            0.56,
            0.2,
            0.4,
            (0, 0.25, -0.11),
            M.shelf,
            bevel = 0.008,
            bevel_segments = 1
        );
        let grip: Vec<(f64, f64)> = rounded_rect(0.16, 0.024, 0.012, 2)
            .iter()
            .map(|&(u, v)| (u, v + 0.07))
            .collect();
        fextrude!(
            fr,
            "crisper_front",
            &rounded_rect(0.56, 0.21, 0.014, 1),
            0.012,
            (0, 0.25, 0.096),
            M.carton,
            holes = [grip]
        );
        // glass milk bottles, a pie and apples, butter
        for (i, x) in [-0.2, -0.12].into_iter().enumerate() {
            bottle(
                &format!("milk{i}"),
                (x, 0.804, -0.16),
                0.034,
                0.22,
                M.carton,
                M.trim,
                10,
                &fr,
            );
        }
        fcyl!(
            fr,
            "pie_dish",
            0.11,
            0.03,
            (0.12, 0.47, -0.12),
            M.porcelain,
            verts = 16,
            bevel = 0.006,
            bevel_segments = 1
        );
        fcyl!(
            fr,
            "pie",
            0.1,
            0.025,
            (0.12, 0.49, -0.12),
            M.board,
            verts = 16,
            bevel = 0.01,
            bevel_segments = 2
        );
        for (i, (x, z)) in [(-0.2, -0.05), (-0.12, -0.08), (0.02, -0.06)]
            .into_iter()
            .enumerate()
        {
            apple(
                &fr,
                &format!("apple{i}"),
                x,
                0.804,
                z + 0.02 * i as f64,
                0.036,
                None,
            );
        }
        fbox!(
            fr,
            "butter_dish",
            0.14,
            0.03,
            0.09,
            (0.14, 0.82, -0.08),
            M.porcelain,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            fr,
            "butter",
            0.1,
            0.04,
            0.06,
            (0.14, 0.85, -0.08),
            M.egg,
            bevel = 0.006,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the rounded door, hinged on its left edge: a trim line, a chrome nameplate, the lever's escutcheon; on its
    // inside, a seal, a liner and two shelves with chrome guard rails holding bottles and eggs
    let parts = new_parts(|| {
        let door = fextrude!(
            fr,
            "door",
            &rounded_rect(0.68, 1.50, 0.075, 4),
            0.04,
            (0, 0.84, 0.30),
            M.mint,
            bevel = 0.014,
            bevel_segments = 3
        );
        soft(door);
        fsweep!(
            fr,
            "seam",
            &[(-0.33, 1.18, 0.318), (0.33, 1.18, 0.318)],
            0.006,
            M.mint2,
            verts = 6
        );
        fextrude!(
            fr,
            "nameplate",
            &rounded_rect(0.2, 0.05, 0.025, 4),
            0.006,
            (0, 1.46, 0.321),
            M.chrome,
            bevel = 0.002,
            bevel_segments = 1
        );
        fextrude!(
            fr,
            "nameplate_inlay",
            &rounded_rect(0.17, 0.028, 0.014, 3),
            0.004,
            (0, 1.46, 0.324),
            M.mint2
        );
        soft!(
            fr,
            "escutcheon",
            0.07,
            0.09,
            0.01,
            (0.27, 1.11, 0.322),
            M.chrome,
            round = 0.005,
            round_segments = 2
        );
        gasket(&fr, "door_seal", 0.64, 1.46, 0.84, 0.276, 0.008, 0.06, 0.0);
        fbox!(
            fr,
            "door_liner",
            0.58,
            1.36,
            0.008,
            (0, 0.84, 0.278),
            M.liner
        );
        for (i, y) in [0.38, 0.86].into_iter().enumerate() {
            fbox!(
                fr,
                format!("door_shelf{i}"),
                0.52,
                0.012,
                0.08,
                (0, y, 0.234),
                M.shelf,
                bevel = 0.004,
                bevel_segments = 1
            );
            fsweep!(
                fr,
                format!("door_rail{i}"),
                &[
                    (-0.26, y, 0.27),
                    (-0.26, y + 0.07, 0.205),
                    (0.26, y + 0.07, 0.205),
                    (0.26, y, 0.27)
                ],
                0.005,
                M.chrome,
                verts = 6,
                resolution = 2
            );
        }
        for (i, (x, mat)) in [(-0.2, M.bottle), (-0.12, M.wine2), (-0.04, M.produce2)]
            .into_iter()
            .enumerate()
        {
            bottle(
                &format!("door_bottle{i}"),
                (x, 0.386, 0.234),
                0.03,
                0.26 - 0.03 * i as f64,
                mat,
                M.carton,
                10,
                &fr,
            );
        }
        for i in 0..5 {
            fsphere!(
                fr,
                format!("egg{i}"),
                0.02,
                (-0.16 + 0.07 * i as f64, 0.886, 0.234),
                M.egg,
                segments = 8,
                rings = 5,
                scale = (1, 1, 1.25)
            );
        }
    });
    let door = hinged(join(&parts, "door", None), fr.at(-0.36, 0.84, 0.30));
    // the chrome lever on the door, pivoting on its top: pulled out first, it unlatches the door
    let parts = new_parts(|| {
        fsweep!(
            fr,
            "handle",
            &[
                (0.27, 1.14, 0.328),
                (0.27, 1.07, 0.348),
                (0.27, 0.95, 0.353),
                (0.27, 0.89, 0.348)
            ],
            0.0,
            M.chrome,
            profile = rounded_rect(0.048, 0.02, 0.009, 2),
            up = (1, 0, 0),
            resolution = 3,
            scale = [1.0, 0.85, 0.9, 1.05],
            angle = 60
        );
    });
    let lever = hinged(join(&parts, "lever", None), fr.at(0.27, 1.12, 0.33));
    settle_all();
    attach(lever, door);
    swing!(lever, "lever", -30, seconds = LEVER_SECONDS, axis = Axis::X); // its foot out to the front
    swing!(
        door,
        "fridge_open",
        100,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    Vec::new()
}
