//! The small under-counter fridge (K1), built to the old part's size with the classic fridge's mechanics: a door
//! hinged on one edge (`hinged`) with `swing`, and a `hold` so it shuts while the resident is busy.

use super::M;
use super::common::{apple, carton, gasket, jar, kick};
use super::fridge::{bottle, door_bin};
use artkit::forms::{circle, rounded_rect};
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, soft, swing};

/// A small under-counter fridge: a white cabinet on a vented plinth, one door over its whole front (its seal,
/// two bins and their bottles inside), an ice box with a flap at the top, a shelf and a crisper below, and a
/// chopping board, a jar and an apple on top.
pub fn fridge_mini() -> Vec<Obj> {
    let fm = GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            fm,
            "plinth",
            0.56,
            0.04,
            0.50,
            (0, 0.02, -0.035),
            M.trim,
            bevel = 0.006
        );
        kick(&fm, "kick", 0.52, 0.03, 0.02, 0.215, None, 5);
        // the cabinet: sides rounded on their front and back edges, a top rounded all round, a back and a floor
        let side = rounded_rect(0.03, 0.54, 0.012, 2);
        for s in [-1, 1] {
            fextrude!(
                fm,
                format!("side{s}"),
                &side,
                0.80,
                (s as f64 * 0.285, 0.44, -0.015),
                M.body,
                plane = Facing::Top,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            fm,
            "top",
            0.60,
            0.03,
            0.54,
            (0, 0.855, -0.015),
            M.body,
            bevel = 0.012,
            bevel_segments = 2
        );
        fbox!(
            fm,
            "bottom",
            0.57,
            0.03,
            0.54,
            (0, 0.055, -0.015),
            M.body,
            bevel = 0.004
        );
        fbox!(
            fm,
            "back",
            0.57,
            0.80,
            0.03,
            (0, 0.44, -0.27),
            M.body,
            bevel = 0.004
        );
        // the liner, the ice box across the top (a flap with a grip), a shelf with a trim, a crisper on the floor
        fbox!(
            fm,
            "liner_back",
            0.54,
            0.77,
            0.006,
            (0, 0.455, -0.252),
            M.liner
        );
        for s in [-1, 1] {
            fbox!(
                fm,
                format!("liner{s}"),
                0.006,
                0.77,
                0.5,
                (s as f64 * 0.267, 0.455, -0.005),
                M.liner
            );
        }
        fbox!(
            fm,
            "icebox",
            0.52,
            0.13,
            0.42,
            (0, 0.765, -0.04),
            M.liner,
            bevel = 0.01,
            bevel_segments = 1
        );
        soft!(
            fm,
            "icebox_flap",
            0.50,
            0.12,
            0.014,
            (0, 0.765, 0.176),
            M.shelf,
            round = 0.006,
            round_segments = 1
        );
        fbox!(
            fm,
            "icebox_grip",
            0.14,
            0.014,
            0.012,
            (0, 0.715, 0.188),
            M.trim,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(fm, "shelf", 0.52, 0.008, 0.40, (0, 0.40, -0.05), M.shelf);
        fbox!(
            fm,
            "shelf_trim",
            0.52,
            0.02,
            0.01,
            (0, 0.397, 0.15),
            M.trim,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            fm,
            "crisper",
            0.50,
            0.14,
            0.40,
            (0, 0.145, -0.05),
            M.shelf,
            bevel = 0.006,
            bevel_segments = 1
        );
        let grip: Vec<(f64, f64)> = rounded_rect(0.13, 0.022, 0.011, 2)
            .iter()
            .map(|&(u, v)| (u, v + 0.045))
            .collect();
        fextrude!(
            fm,
            "crisper_front",
            &rounded_rect(0.50, 0.15, 0.012, 1),
            0.012,
            (0, 0.145, 0.156),
            M.carton,
            holes = [grip]
        );
        // food: milk, a bottle and apples on the shelf
        carton(&fm, "milk", -0.16, 0.404, -0.08, 0.07, 0.14, M.carton, 0);
        bottle(
            "juice",
            (-0.05, 0.404, -0.1),
            0.028,
            0.2,
            M.bottle,
            M.carton,
            10,
            &fm,
        );
        for (i, (x, z)) in [(0.07, -0.02), (0.15, -0.05), (0.11, -0.12)]
            .into_iter()
            .enumerate()
        {
            apple(&fm, &format!("apple{i}"), x, 0.404, z, 0.034, None);
        }
        // a chopping board (a hanging hole at one end), a jar and an apple on top
        let hang: Vec<(f64, f64)> = circle(0.012, 8)
            .iter()
            .map(|&(u, v)| (u + 0.2, v))
            .collect();
        fextrude!(
            fm,
            "board",
            &rounded_rect(0.50, 0.40, 0.03, 3),
            0.02,
            (0, 0.88, -0.02),
            M.board,
            plane = Facing::Top,
            holes = [hang],
            bevel = 0.004,
            bevel_segments = 1
        );
        jar(
            &fm,
            "jar",
            -0.14,
            0.89,
            -0.04,
            0.05,
            0.107,
            M.porcelain,
            M.trim,
            12,
        );
        apple(&fm, "apple_top", 0.12, 0.89, 0.0, 0.042, None);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the door over the whole front, hinged on its left edge: its seal, a liner and two bins with bottles and a can
    let parts = new_parts(|| {
        soft!(
            fm,
            "door",
            0.58,
            0.77,
            0.03,
            (0, 0.475, 0.27),
            M.body,
            round = 0.012
        );
        soft!(
            fm,
            "badge",
            0.08,
            0.016,
            0.005,
            (-0.18, 0.80, 0.286),
            M.trim,
            round = 0.003,
            round_segments = 1
        );
        pull!(&fm, "handle", (0.24, 0.36), (0.24, 0.62), 0.285);
        gasket(&fm, "door_seal", 0.55, 0.74, 0.475, 0.252, 0.008, 0.03, 0.0);
        fbox!(
            fm,
            "door_liner",
            0.50,
            0.68,
            0.008,
            (0, 0.475, 0.25),
            M.liner
        );
        for (i, y) in [0.16, 0.50].into_iter().enumerate() {
            door_bin(&format!("bin{i}"), y, -0.23, 0.23, 0.17, 0.246, &fm);
        }
        for (i, (x, mat)) in [(-0.18, M.bottle), (-0.11, M.wine2)]
            .into_iter()
            .enumerate()
        {
            bottle(
                &format!("door_bottle{i}"),
                (x, 0.17, 0.205),
                0.027,
                0.24 - 0.03 * i as f64,
                mat,
                M.carton,
                10,
                &fm,
            );
        }
        fcyl!(
            fm,
            "can",
            0.026,
            0.1,
            (0.15, 0.56, 0.205),
            M.produce,
            verts = 10,
            bevel = 0.003,
            bevel_segments = 1
        );
        jar(
            &fm, "door_jar", 0.07, 0.51, 0.205, 0.03, 0.08, M.egg, M.trim, 10,
        );
    });
    let door = hinged(join(&parts, "door", None), fm.at(-0.29, 0.40, 0.27));
    settle_all();
    swing!(
        door,
        "fridge_open",
        100,
        seconds = 1.6,
        ease = Ease::BackOut
    );
    Vec::new()
}
