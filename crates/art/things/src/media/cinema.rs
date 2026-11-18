//! The home cinema (2x1): an oak media unit, tower speakers, a soundbar and a short-throw projector on it, and a
//! projector screen rolled up in its case on the wall above.

use super::common::call::rbox;
use super::common::{WALL, capsule, driver, grille, py};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{Axis, P3};
use artkit::motion::flat;
use artkit::parts::{join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsweep, grow, settle, soft};

const CASE_Y: f64 = 1.6;
const SCREEN_W: f64 = 1.54;
const SCREEN_H: f64 = 0.92;

pub fn home_cinema() -> Vec<Obj> {
    let still = new_parts(|| {
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                F,
                format!("sled{s}"),
                &kit::fillet(
                    &[
                        (sf * 0.86, 0.04, -0.405),
                        (sf * 0.86, 0.02, -0.38),
                        (sf * 0.86, 0.02, -0.06),
                        (sf * 0.86, 0.04, -0.035)
                    ],
                    0.02,
                    2
                ),
                0,
                M.black_metal,
                profile = rounded_rect(0.03, 0.04, 0.008, 1),
                up = (1, 0, 0),
                angle = 50.0
            );
        }
        soft!(
            F,
            "unit",
            1.86,
            0.42,
            0.43,
            (0, 0.25, -0.215),
            M.cinema_unit,
            round = 0.012,
            round_segments = 2
        );
        soft!(
            F,
            "top",
            1.88,
            0.03,
            0.44,
            (0, 0.475, -0.21),
            M.cinema_oak,
            round = 0.009,
            round_segments = 2
        );
        fbox!(F, "reveal", 1.82, 0.31, 0.002, (0, 0.25, 0.0005), M.grille);
        for x in [-0.62, 0.0, 0.62] {
            // a slab front with a finger grip routed along its top
            soft!(
                F,
                format!("front{}", py(x)),
                0.58,
                0.3,
                0.012,
                (x, 0.25, 0.006),
                M.cinema_oak,
                round = 0.004,
                round_segments = 2
            );
            capsule(
                format!("grip{}", py(x)),
                x - 0.08,
                x + 0.08,
                0.385,
                0.012,
                0.012,
                0.012,
                M.cinema_unit,
                Some(0.004),
            );
        }
        // the soundbar at the front, the short-throw projector behind it, a player on the right
        capsule(
            "soundbar",
            -0.5,
            0.5,
            0.525,
            -0.07,
            0.07,
            0.09,
            M.tower,
            Some(0.015),
        );
        grille(
            "soundbar_grille",
            0,
            0.525,
            -0.0255,
            0.94,
            0.04,
            M.grille,
            Some(5),
            1,
        );
        rbox!(
            "projector",
            0.5,
            0.1,
            0.3,
            (0, 0.54, -0.27),
            M.projector,
            r = 0.025,
            segs = 3
        );
        rbox!(
            "lens",
            0.32,
            0.006,
            0.07,
            (0, 0.591, -0.37),
            M.lens,
            r = 0.002
        );
        fbox!(
            F,
            "lens_glass",
            0.28,
            0.002,
            0.04,
            (0, 0.5945, -0.37),
            M.glass_lens
        );
        for k in 0..6 {
            fbox!(
                F,
                format!("projector_vent{k}"),
                0.004,
                0.06,
                0.002,
                (-0.12 + 0.012 * k as f64, 0.53, -0.1195),
                M.grille
            );
        }
        rbox!(
            "player",
            0.34,
            0.05,
            0.24,
            (0.55, 0.515, -0.25),
            M.tower,
            r = 0.01,
            segs = 2
        );
        fbox!(
            F,
            "player_slot",
            0.16,
            0.005,
            0.002,
            (0.52, 0.522, -0.1295),
            M.cone
        );
        fbox!(
            F,
            "player_led",
            0.02,
            0.008,
            0.004,
            (0.66, 0.515, -0.128),
            M.led
        );
        // the towers: rounded cabinets on plinths, two woofers and a tweeter each behind rings
        for s in [-1, 1] {
            let x = s as f64 * 0.86;
            soft!(
                F,
                format!("tower{s}"),
                0.15,
                0.88,
                0.2,
                (x, 0.95, -0.3),
                M.tower,
                round = 0.018,
                round_segments = 2
            );
            rbox!(
                format!("tower_plinth{s}"),
                0.155,
                0.02,
                0.205,
                (x, 0.5, -0.3),
                M.black_metal,
                r = 0.004
            );
            for y in [0.78, 1.0] {
                driver(
                    &format!("woofer{s}{}", py(y)),
                    x,
                    y,
                    -0.2,
                    0.055,
                    None,
                    Some(M.black_metal),
                    20,
                );
            }
            driver(&format!("tweeter{s}"), x, 1.22, -0.2, 0.025, None, None, 14);
            fbox!(
                F,
                format!("tower_badge{s}"),
                0.03,
                0.006,
                0.002,
                (x, 0.56, -0.199),
                M.steel
            );
        }
        // the screen's case, on the wall above, with its end caps and brackets
        capsule(
            "case",
            -SCREEN_W / 2.0 - 0.04,
            SCREEN_W / 2.0 + 0.04,
            CASE_Y,
            WALL + 0.045,
            0.09,
            0.09,
            M.tower,
            Some(0.035),
        );
        for s in [-1, 1] {
            rbox!(
                format!("cap{s}"),
                0.02,
                0.1,
                0.1,
                (s as f64 * (SCREEN_W / 2.0 + 0.05), CASE_Y, WALL + 0.05),
                M.black_metal,
                r = 0.008,
                segs = 2
            );
        }
    });
    join(&still, "homeCinema", Some(P3::ZERO));
    // the screen, its origin in the case, rolled up to nothing at rest: `screen_down` pulls it down
    let top = CASE_Y - 0.045;
    let z = WALL + 0.045;
    let screen = new_parts(|| {
        fbox!(
            F,
            "projection",
            SCREEN_W - 0.06,
            SCREEN_H - 0.1,
            0.004,
            (0, top - 0.06 - (SCREEN_H - 0.1) / 2.0 - 0.02, z),
            M.projection
        );
        fbox!(
            F,
            "screen_back",
            SCREEN_W,
            SCREEN_H,
            0.003,
            (0, top - SCREEN_H / 2.0, z - 0.0035),
            M.screen_border
        );
        capsule(
            "screen_bar",
            -SCREEN_W / 2.0 - 0.01,
            SCREEN_W / 2.0 + 0.01,
            top - SCREEN_H,
            z,
            0.02,
            0.016,
            M.black_metal,
            Some(0.007),
        );
        fsweep!(
            F,
            "screen_pull",
            &kit::fillet(
                &[
                    (-0.03, top - SCREEN_H - 0.008, z),
                    (-0.03, top - SCREEN_H - 0.04, z),
                    (0.03, top - SCREEN_H - 0.04, z),
                    (0.03, top - SCREEN_H - 0.008, z)
                ],
                0.01,
                3
            ),
            0.003,
            M.black_metal,
            verts = 5
        );
    });
    let screen = join(&screen, "screen", Some(F.at(0, CASE_Y - 0.01, z)));
    flat(screen, Axis::Z);
    settle!(centre = false);
    grow!(screen, "screen_down", seconds = 1.4);
    meshes()
}
