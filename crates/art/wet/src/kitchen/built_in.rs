//! The tall fridge-freezer (K1) behind shaker cabinet fronts: a fridge door over a freezer door, each hinged on its
//! left edge with a clip of its own.

use super::common::{apple, carton, gasket, jar, kick};
use super::fridge::{bottle, door_bin};
use super::{DOOR_SECONDS, M};
use crate::common::shaker;
use artkit::forms::rounded_rect;
use artkit::math::{cos, radians, sin};
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, swing};

/// A tall fridge-freezer behind shaker cabinet fronts under a moulded cornice: a fridge door (a water dispenser
/// set into it) over a freezer door, long bar handles. Inside, glass shelves, crispers and food over two freezer
/// baskets; the doors carry their seals, bins and bottles.
pub fn fridge_built_in() -> Vec<Obj> {
    let fb = GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            fb,
            "plinth",
            0.78,
            0.10,
            0.54,
            (0, 0.05, -0.05),
            M.dark,
            bevel = 0.004
        );
        kick(&fb, "kick", 0.66, 0.05, 0.065, 0.236, None, 7);
        for s in [-1, 1] {
            fbox!(
                fb,
                format!("side{s}"),
                0.03,
                1.88,
                0.58,
                (s as f64 * 0.395, 1.06, -0.03),
                M.panel,
                bevel = 0.004
            );
        }
        fbox!(fb, "top", 0.76, 0.03, 0.58, (0, 1.985, -0.03), M.panel);
        fbox!(fb, "bottom", 0.76, 0.03, 0.58, (0, 0.115, -0.03), M.panel);
        fbox!(fb, "back", 0.76, 1.88, 0.03, (0, 1.06, -0.30), M.panel);
        // the cornice: a slab with a cove moulding swept round its front and sides
        fbox!(fb, "cornice", 0.80, 0.05, 0.60, (0, 2.0, -0.015), M.panel);
        // (up, out) from the slab's edge: a cove rising from the panel face to a square lip
        let mut cove: Vec<(f64, f64)> = vec![(0.0, -0.01), (0.0, 0.004)];
        cove.extend((0..91).step_by(18).map(|a| {
            (
                0.004 + 0.036 * sin(radians(a as f64)),
                0.004 + 0.016 * (1.0 - cos(radians(a as f64))),
            )
        }));
        cove.extend([(0.05, 0.02), (0.05, -0.01)]);
        fsweep!(
            fb,
            "cornice_moulding",
            &[
                (0.4, 1.975, -0.315),
                (0.4, 1.975, 0.285),
                (-0.4, 1.975, 0.285),
                (-0.4, 1.975, -0.315)
            ],
            0.0,
            M.panel,
            profile = cove,
            up = (0, 0, 1),
            angle = 50
        );
        // the liner, glass shelves with trims, two crispers, the divider and the freezer's two baskets
        fbox!(
            fb,
            "liner_back",
            0.76,
            1.72,
            0.01,
            (0, 1.06, -0.28),
            M.liner
        );
        for s in [-1, 1] {
            fbox!(
                fb,
                format!("liner{s}"),
                0.006,
                1.72,
                0.54,
                (s as f64 * 0.377, 1.06, -0.02),
                M.liner
            );
        }
        fbox!(
            fb,
            "liner_top",
            0.76,
            0.006,
            0.54,
            (0, 1.917, -0.02),
            M.liner
        );
        fbox!(fb, "divider", 0.76, 0.02, 0.44, (0, 0.80, -0.06), M.shelf);
        fbox!(
            fb,
            "divider_trim",
            0.76,
            0.05,
            0.02,
            (0, 0.80, 0.17),
            M.liner,
            bevel = 0.006,
            bevel_segments = 1
        );
        for (i, y) in [1.08, 1.38, 1.68].into_iter().enumerate() {
            fbox!(
                fb,
                format!("shelf{i}"),
                0.74,
                0.008,
                0.42,
                (0, y, -0.06),
                M.shelf
            );
            fbox!(
                fb,
                format!("shelf_trim{i}"),
                0.74,
                0.022,
                0.012,
                (0, y - 0.004, 0.15),
                M.trim,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                fb,
                format!("crisper{s}"),
                0.36,
                0.22,
                0.40,
                (sf * 0.185, 0.94, -0.06),
                M.shelf,
                bevel = 0.006,
                bevel_segments = 1
            );
            let grip: Vec<(f64, f64)> = rounded_rect(0.14, 0.022, 0.011, 2)
                .iter()
                .map(|&(u, v)| (u, v + 0.08))
                .collect();
            fextrude!(
                fb,
                format!("crisper_front{s}"),
                &rounded_rect(0.36, 0.23, 0.012, 1),
                0.012,
                (sf * 0.185, 0.94, 0.146),
                M.carton,
                holes = [grip],
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        for (i, y) in [0.16, 0.48].into_iter().enumerate() {
            fbox!(
                fb,
                format!("basket{i}"),
                0.70,
                0.02,
                0.42,
                (0, y, -0.06),
                M.liner
            );
            let grip: Vec<(f64, f64)> = rounded_rect(0.16, 0.026, 0.013, 2)
                .iter()
                .map(|&(u, v)| (u, v + 0.06))
                .collect();
            fextrude!(
                fb,
                format!("basket_front{i}"),
                &rounded_rect(0.70, 0.2, 0.012, 1),
                0.014,
                (0, y + 0.09, 0.152),
                M.shelf,
                holes = [grip],
                bevel = 0.003,
                bevel_segments = 1
            );
            for s in [-1, 1] {
                fbox!(
                    fb,
                    format!("basket_side{i}{s}"),
                    0.012,
                    0.18,
                    0.42,
                    (s as f64 * 0.344, y + 0.09, -0.06),
                    M.liner
                );
            }
        }
        // food, fresh above and frozen below
        carton(&fb, "milk", -0.24, 1.084, -0.06, 0.08, 0.17, M.carton, 0);
        bottle(
            "juice",
            (-0.1, 1.084, -0.08),
            0.032,
            0.24,
            M.bottle,
            M.carton,
            10,
            &fb,
        );
        jar(
            &fb, "jam", 0.08, 1.084, -0.04, 0.04, 0.1, M.produce, M.trim, 12,
        );
        for (i, (x, z)) in [(-0.24, -0.02), (-0.15, -0.06), (-0.2, -0.13)]
            .into_iter()
            .enumerate()
        {
            apple(&fb, &format!("apple{i}"), x, 1.384, z, 0.036, None);
        }
        apple(&fb, "orange", -0.06, 1.384, -0.03, 0.04, Some(M.produce2));
        fbox!(
            fb,
            "egg_carton",
            0.22,
            0.03,
            0.11,
            (0.16, 1.40, -0.05),
            M.carton,
            bevel = 0.006
        );
        for i in 0..4 {
            fsphere!(
                fb,
                format!("egg{i}"),
                0.018,
                (0.085 + 0.05 * i as f64, 1.432, -0.05),
                M.egg,
                segments = 8,
                rings = 5,
                scale = (1, 1, 1.25)
            );
        }
        fbox!(
            fb,
            "tub",
            0.14,
            0.07,
            0.14,
            (-0.16, 1.719, -0.06),
            M.shelf,
            bevel = 0.012,
            bevel_segments = 1
        );
        fbox!(
            fb,
            "pizza",
            0.26,
            0.03,
            0.26,
            (-0.15, 0.505, -0.06),
            M.frozen,
            bevel = 0.006
        );
        fbox!(
            fb,
            "peas",
            0.12,
            0.16,
            0.06,
            (0.14, 0.57, -0.1),
            M.peas,
            bevel = 0.012,
            bevel_segments = 1
        );
        fbox!(
            fb,
            "ice_cream",
            0.12,
            0.10,
            0.12,
            (0.16, 0.22, -0.06),
            M.icecream,
            bevel = 0.01
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the tall fridge door, hinged on its left edge: a shaker front with a water dispenser set into it and a bar
    // handle; its seal, liner and bins with bottles on its inside
    let parts = new_parts(|| {
        shaker(
            &fb,
            "fridge_door",
            0.78,
            1.14,
            (0, 1.39, 0.30),
            M.panel,
            0.025,
            0.07,
            0.004,
        );
        fextrude!(
            fb,
            "dispenser",
            &rounded_rect(0.16, 0.24, 0.02, 2),
            0.014,
            (-0.24, 1.42, 0.31),
            M.trim,
            holes = [rounded_rect(0.13, 0.21, 0.012, 2)],
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            fb,
            "dispenser_back",
            0.14,
            0.22,
            0.01,
            (-0.24, 1.42, 0.305),
            M.dark
        );
        fbox!(
            fb,
            "dispenser_screen",
            0.08,
            0.025,
            0.004,
            (-0.24, 1.505, 0.311),
            M.water
        );
        fbox!(
            fb,
            "dispenser_paddle",
            0.06,
            0.07,
            0.008,
            (-0.24, 1.43, 0.311),
            M.seal,
            bevel = 0.003,
            bevel_segments = 1
        );
        fcyl!(
            fb,
            "dispenser_spout",
            0.008,
            0.02,
            (-0.24, 1.475, 0.302),
            M.chrome,
            verts = 8
        );
        fextrude!(
            fb,
            "drip_tray",
            &rounded_rect(0.12, 0.03, 0.008, 1),
            0.012,
            (-0.24, 1.33, 0.312),
            M.seal,
            plane = Facing::Front
        );
        pull!(
            &fb,
            "handle_fridge",
            (0.33, 1.08),
            (0.33, 1.70),
            0.3125,
            out = 0.03,
            w = 0.018,
            r = None
        );
        gasket(
            &fb,
            "fridge_seal",
            0.75,
            1.11,
            1.39,
            0.282,
            0.008,
            0.03,
            0.0,
        );
        fbox!(
            fb,
            "fridge_liner",
            0.70,
            1.04,
            0.008,
            (0, 1.39, 0.284),
            M.liner
        );
        for (i, y) in [0.88, 1.18, 1.5].into_iter().enumerate() {
            door_bin(&format!("fridge_bin{i}"), y, -0.33, 0.33, 0.18, 0.28, &fb);
        }
        for (i, (x, mat)) in [(-0.28, M.bottle), (-0.2, M.wine2), (-0.12, M.produce2)]
            .into_iter()
            .enumerate()
        {
            bottle(
                &format!("door_bottle{i}"),
                (x, 0.89, 0.23),
                0.03,
                0.27 - 0.025 * i as f64,
                mat,
                M.carton,
                10,
                &fb,
            );
        }
        carton(&fb, "door_milk", 0.2, 0.89, 0.23, 0.07, 0.17, M.carton, 0);
        for (i, x) in [-0.2, -0.11].into_iter().enumerate() {
            jar(
                &fb,
                &format!("door_jar{i}"),
                x,
                1.19,
                0.23,
                0.033,
                0.09,
                if i != 0 { M.egg } else { M.produce },
                M.trim,
                10,
            );
        }
        fbox!(
            fb,
            "butter",
            0.12,
            0.05,
            0.06,
            (0.18, 1.215, 0.23),
            M.egg,
            bevel = 0.008,
            bevel_segments = 1
        );
    });
    let fridge_door = hinged(join(&parts, "fridge_door", None), fb.at(-0.39, 1.39, 0.30));
    // the freezer door below, same hinge, a clip of its own (unused until a use takes frozen food)
    let parts = new_parts(|| {
        shaker(
            &fb,
            "freezer_door",
            0.78,
            0.62,
            (0, 0.50, 0.30),
            M.panel,
            0.025,
            0.07,
            0.004,
        );
        pull!(
            &fb,
            "handle_freezer",
            (-0.25, 0.72),
            (0.25, 0.72),
            0.3125,
            out = 0.03,
            w = 0.018,
            r = None
        );
        gasket(
            &fb,
            "freezer_seal",
            0.75,
            0.59,
            0.50,
            0.282,
            0.008,
            0.03,
            0.0,
        );
        fbox!(
            fb,
            "freezer_liner",
            0.70,
            0.52,
            0.008,
            (0, 0.50, 0.284),
            M.liner
        );
    });
    let freezer = hinged(join(&parts, "freezer_door", None), fb.at(-0.39, 0.50, 0.30));
    settle_all();
    swing!(
        fridge_door,
        "fridge_open",
        100,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    swing!(
        freezer,
        "freezer_open",
        100,
        seconds = DOOR_SECONDS,
        ease = Ease::BackOut
    );
    Vec::new()
}
