//! The stainless French-door fridge: two upper doors hinged on their outer edges and a freezer drawer that
//! slides out (`slide`).

use super::common::{apple, carton, gasket, jar, kick};
use super::fridge::{bottle, door_bin};
use super::{DOOR_SECONDS, M};
use artkit::forms::rounded_rect;
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, slide, soft, swing};

/// A stainless French-door fridge: two upper doors (an ice and water dispenser set into the left one) with long
/// bar handles, a full-width freezer drawer below with frozen food in its bin. Inside, glass shelves, crispers and
/// food; the doors carry their seals, bins and bottles.
#[allow(clippy::approx_constant)] // (0.318 is a depth, not 1/π)
pub fn fridge_french_door() -> Vec<Obj> {
    let ff = GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            ff,
            "plinth",
            0.86,
            0.04,
            0.62,
            (0, 0.02, -0.07),
            M.trim,
            bevel = 0.006
        );
        kick(&ff, "kick", 0.8, 0.034, 0.024, 0.24, Some(M.dark), 8);
        // the cabinet's sides, top and bottom come forward to the doors' backs, so a shut door sits flush on them;
        // the sides rounded on their front and back edges, as pressed steel is
        let side = rounded_rect(0.03, 0.71, 0.012, 2);
        for s in [-1, 1] {
            fextrude!(
                ff,
                format!("side{s}"),
                &side,
                1.76,
                (s as f64 * 0.435, 0.92, -0.025),
                M.steel,
                plane = Facing::Top,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            ff,
            "top",
            0.87,
            0.03,
            0.71,
            (0, 1.805, -0.025),
            M.steel,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(ff, "bottom", 0.87, 0.03, 0.71, (0, 0.035, -0.025), M.steel);
        fbox!(ff, "back", 0.87, 1.76, 0.03, (0, 0.92, -0.365), M.steel);
        // the liner, the divider above the freezer, glass shelves with trims, two crispers
        fbox!(
            ff,
            "liner_back",
            0.84,
            1.70,
            0.01,
            (0, 0.92, -0.345),
            M.liner
        );
        for s in [-1, 1] {
            fbox!(
                ff,
                format!("liner{s}"),
                0.006,
                1.70,
                0.66,
                (s as f64 * 0.417, 0.92, -0.02),
                M.liner
            );
        }
        fbox!(
            ff,
            "liner_top",
            0.84,
            0.006,
            0.66,
            (0, 1.77, -0.02),
            M.liner
        );
        fbox!(ff, "divider", 0.84, 0.02, 0.64, (0, 0.68, -0.03), M.shelf);
        fbox!(
            ff,
            "divider_trim",
            0.84,
            0.05,
            0.02,
            (0, 0.68, 0.30),
            M.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        for (i, y) in [0.96, 1.24, 1.52].into_iter().enumerate() {
            fbox!(
                ff,
                format!("shelf{i}"),
                0.82,
                0.008,
                0.54,
                (0, y, -0.07),
                M.shelf
            );
            fbox!(
                ff,
                format!("shelf_trim{i}"),
                0.82,
                0.022,
                0.012,
                (0, y - 0.004, 0.2),
                M.trim,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                ff,
                format!("crisper{s}"),
                0.4,
                0.2,
                0.5,
                (sf * 0.205, 0.80, -0.07),
                M.shelf,
                bevel = 0.006,
                bevel_segments = 1
            );
            let grip: Vec<(f64, f64)> = rounded_rect(0.15, 0.022, 0.011, 2)
                .iter()
                .map(|&(u, v)| (u, v + 0.07))
                .collect();
            fextrude!(
                ff,
                format!("crisper_front{s}"),
                &rounded_rect(0.4, 0.21, 0.012, 1),
                0.012,
                (sf * 0.205, 0.80, 0.186),
                M.carton,
                holes = [grip],
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // food
        carton(&ff, "milk", -0.28, 0.964, -0.1, 0.08, 0.17, M.carton, 0);
        bottle(
            "juice",
            (-0.15, 0.964, -0.12),
            0.033,
            0.25,
            M.bottle,
            M.carton,
            10,
            &ff,
        );
        jar(
            &ff, "jam", 0.02, 0.964, -0.06, 0.04, 0.1, M.produce, M.trim, 12,
        );
        fbox!(
            ff,
            "tub",
            0.15,
            0.07,
            0.15,
            (0.22, 0.999, -0.08),
            M.shelf,
            bevel = 0.012,
            bevel_segments = 1
        );
        for (i, (x, z)) in [(-0.28, -0.02), (-0.19, -0.06), (-0.24, -0.13)]
            .into_iter()
            .enumerate()
        {
            apple(&ff, &format!("apple{i}"), x, 1.244, z, 0.036, None);
        }
        apple(&ff, "orange", -0.1, 1.244, -0.02, 0.04, Some(M.produce2));
        fbox!(
            ff,
            "egg_carton",
            0.22,
            0.03,
            0.11,
            (0.18, 1.26, -0.06),
            M.carton,
            bevel = 0.006
        );
        for i in 0..4 {
            fsphere!(
                ff,
                format!("egg{i}"),
                0.018,
                (0.105 + 0.05 * i as f64, 1.292, -0.06),
                M.egg,
                segments = 8,
                rings = 5,
                scale = (1, 1, 1.25)
            );
        }
        fbox!(
            ff,
            "cake",
            0.2,
            0.09,
            0.2,
            (0.05, 1.569, -0.1),
            M.icecream,
            bevel = 0.02,
            bevel_segments = 2
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));

    /// An upper door, `s` -1 on the left: a rounded steel slab, its handle by the middle, its seal, liner and bins.
    fn door(ff: &GameFrame, name: &str, s: i32) {
        let x = s as f64 * 0.222;
        soft!(
            ff,
            name,
            0.436,
            1.10,
            0.026,
            (x, 1.25, 0.343),
            M.steel,
            round = 0.012
        );
        pull!(
            ff,
            format!("handle_{name}"),
            (s as f64 * 0.04, 0.92),
            (s as f64 * 0.04, 1.62),
            0.356,
            out = 0.019,
            w = 0.016,
            t = 0.012,
            r = Some(0.005)
        );
        gasket(
            ff,
            &format!("{name}_seal"),
            0.41,
            1.07,
            1.25,
            0.326,
            0.007,
            0.03,
            x,
        );
        fbox!(
            ff,
            format!("{name}_liner"),
            0.36,
            1.0,
            0.008,
            (x, 1.25, 0.328),
            M.liner
        );
        for (i, y) in [0.78, 1.08, 1.38].into_iter().enumerate() {
            door_bin(
                &format!("{name}_bin{i}"),
                y,
                x - 0.17,
                x + 0.17,
                0.235,
                0.325,
                ff,
            );
        }
    }
    // the left upper door (game -x: the viewer's right), hinged on its left edge, and the right, hinged on its right
    let parts = new_parts(|| {
        door(&ff, "door_left", -1);
        for (i, x) in [-0.12, -0.2, -0.28].into_iter().enumerate() {
            jar(
                &ff,
                &format!("door_jar{i}"),
                x,
                1.09,
                0.28,
                0.032,
                0.09,
                [M.egg, M.produce, M.bottle][i],
                M.trim,
                10,
            );
        }
        fbox!(
            ff,
            "butter",
            0.12,
            0.05,
            0.06,
            (-0.2, 1.405, 0.28),
            M.egg,
            bevel = 0.008,
            bevel_segments = 1
        );
    });
    let door_left = hinged(join(&parts, "door_left", None), ff.at(-0.44, 1.25, 0.345));
    let parts = new_parts(|| {
        door(&ff, "door_right", 1);
        // the dispenser on the door the viewer sees on the left: a dark recess in a trim, a paddle and a spout, and
        // the display over it
        fextrude!(
            ff,
            "dispenser",
            &rounded_rect(0.18, 0.28, 0.025, 2),
            0.01,
            (0.24, 1.3, 0.358),
            M.trim,
            holes = [rounded_rect(0.15, 0.25, 0.016, 2)],
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            ff,
            "dispenser_back",
            0.16,
            0.26,
            0.008,
            (0.24, 1.3, 0.354),
            M.dark
        );
        fbox!(
            ff,
            "dispenser_paddle",
            0.07,
            0.08,
            0.008,
            (0.24, 1.32, 0.36),
            M.seal,
            bevel = 0.003,
            bevel_segments = 1
        );
        fcyl!(
            ff,
            "dispenser_spout",
            0.009,
            0.02,
            (0.24, 1.38, 0.352),
            M.chrome,
            verts = 8
        );
        fbox!(
            ff,
            "dispenser_tray",
            0.12,
            0.012,
            0.03,
            (0.24, 1.19, 0.358),
            M.seal,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            ff,
            "display",
            0.12,
            0.05,
            0.004,
            (0.24, 1.49, 0.357),
            M.dark,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            ff,
            "display_glow",
            0.07,
            0.012,
            0.003,
            (0.24, 1.495, 0.359),
            M.water
        );
        // the flip strip the other door's edge closes on
        fbox!(
            ff,
            "mullion",
            0.04,
            1.02,
            0.022,
            (0.025, 1.25, 0.318),
            M.liner,
            bevel = 0.006,
            bevel_segments = 1
        );
        for (i, (x, mat)) in [(0.36, M.bottle), (0.29, M.wine2), (0.22, M.produce2)]
            .into_iter()
            .enumerate()
        {
            bottle(
                &format!("door_bottle{i}"),
                (x, 0.79, 0.28),
                0.028,
                0.26 - 0.025 * i as f64,
                mat,
                M.carton,
                10,
                &ff,
            );
        }
        carton(&ff, "door_milk", 0.12, 0.79, 0.28, 0.07, 0.17, M.carton, 0);
    });
    let door_right = hinged(join(&parts, "door_right", None), ff.at(0.44, 1.25, 0.345));
    // the freezer drawer, sliding out to the front: a steel front with a long pull, its bin of frozen food
    let parts = new_parts(|| {
        soft!(
            ff,
            "drawer",
            0.88,
            0.58,
            0.026,
            (0, 0.38, 0.343),
            M.steel,
            round = 0.012
        );
        pull!(
            &ff,
            "handle_drawer",
            (-0.3, 0.6),
            (0.3, 0.6),
            0.356,
            out = 0.019,
            w = 0.016,
            t = 0.012,
            r = Some(0.005)
        );
        fbox!(
            ff,
            "drawer_floor",
            0.80,
            0.012,
            0.42,
            (0, 0.13, 0.11),
            M.liner
        );
        for s in [-1, 1] {
            fbox!(
                ff,
                format!("drawer_side{s}"),
                0.012,
                0.44,
                0.42,
                (s as f64 * 0.394, 0.35, 0.11),
                M.liner
            );
        }
        fbox!(
            ff,
            "drawer_back",
            0.80,
            0.44,
            0.012,
            (0, 0.35, -0.094),
            M.liner
        );
        fbox!(
            ff,
            "drawer_inner",
            0.80,
            0.44,
            0.012,
            (0, 0.35, 0.314),
            M.liner
        );
        fbox!(
            ff,
            "pizza",
            0.30,
            0.035,
            0.30,
            (-0.2, 0.154, 0.1),
            M.frozen,
            bevel = 0.006
        );
        fbox!(
            ff,
            "pizza2",
            0.30,
            0.035,
            0.30,
            (-0.19, 0.189, 0.11),
            M.frozen,
            bevel = 0.006
        );
        fbox!(
            ff,
            "ice_cream",
            0.14,
            0.11,
            0.14,
            (0.13, 0.191, 0.15),
            M.icecream,
            bevel = 0.012,
            bevel_segments = 1
        );
        fbox!(
            ff,
            "peas",
            0.13,
            0.06,
            0.2,
            (0.15, 0.166, -0.03),
            M.peas,
            bevel = 0.015,
            bevel_segments = 1
        );
    });
    let drawer = join(&parts, "drawer", Some(ff.at(0, 0.38, 0.345)));
    settle_all();
    swing!(
        door_left,
        "fridge_open",
        100,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        door_right,
        "fridge_open",
        -100,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(door_right, "door_right", -100, seconds = DOOR_SECONDS);
    slide!(
        drawer,
        "freezer_open",
        (0, -0.35, 0),
        seconds = DOOR_SECONDS
    );
    Vec::new()
}
