//! The grand cinema wall and the Pixel Arcade console: a screen on the wall, the seats in front of it. Each is
//! built with the seats at the back and the screen at the front edge facing them, then turned round.

use super::common::{M, cup, driver, pleats, popcorn, screen};
use artkit::bevel::ease;
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit::turn_round;
use artkit::math::{P3, cos, radians, sin};
use artkit::motion::{Squash, squash};
use artkit::parts::{hinged, join, new_parts, numbered, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fextrude, fsweep, slide, soft, torus};

pub const SCREEN_Z: f64 = 0.42;

// --- the grand cinema wall (2x1): the screen on the wall, twin recliners in front of it, a console between --------

/// One recliner of the pair centred on (x, 0), facing the screen (before the turn, in front): a stuffed, piped
/// seat cushion (top 0.45), a back of three pillowed rolls leaning back, a padded arm outside, on a dark plinth.
fn cinema_seat(f: &GameFrame, x: f64, w: f64, outer: f64) {
    soft!(
        f,
        numbered("seat_base"),
        w,
        0.28,
        0.62,
        (x, 0.2, -0.06),
        M.leather,
        round = 0.06,
        flat = 1,
        round_segments = 1
    );
    soft!(
        f,
        numbered("seat"),
        w - 0.18,
        0.09,
        0.54,
        (x, 0.393, -0.01),
        M.leather,
        round = 0.035,
        puff = 0.012,
        flat = 2,
        resting = true
    );
    let a = radians(14.0);
    let (up, back) = ((cos(a), sin(a)), (0.32, -0.23));
    soft!(
        f,
        numbered("seat_shell"),
        w,
        0.68,
        0.12,
        (x, back.0 + 0.33 * up.0, back.1 - 0.33 * up.1 - 0.03),
        M.leather,
        rot = (-14, 0, 0),
        round = 0.05,
        flat = 1,
        round_segments = 1
    );
    for (t, hh) in [(0.12, 0.22), (0.33, 0.2), (0.53, 0.18)] {
        soft!(
            f,
            numbered("seat_roll"),
            w - 0.16,
            hh,
            0.13,
            (x, back.0 + t * up.0, back.1 - t * up.1 + 0.02),
            M.leather,
            rot = (-14, 0, 0),
            round = 0.05,
            puff = 0.025,
            flat = 2,
            round_segments = 1,
            resting = true
        );
    }
    soft!(
        f,
        numbered("arm"),
        0.11,
        0.3,
        0.62,
        (outer, 0.47, -0.06),
        M.leather,
        round = 0.045,
        flat = 1,
        round_segments = 1
    );
    soft!(
        f,
        numbered("arm_pad"),
        0.13,
        0.05,
        0.5,
        (outer, 0.63, -0.03),
        M.leather,
        round = 0.022,
        flat = 1,
        round_segments = 1,
        resting = true
    );
    fextrude!(
        f,
        numbered("seat_foot"),
        &rounded_rect(w - 0.1, 0.56, 0.04, 2),
        0.05,
        (x, 0.025, -0.06),
        M.dark,
        plane = Facing::Top,
        bevel = 0.004,
        bevel_segments = 1
    );
}

pub fn cinema_wall() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        for s in [-1, 1] {
            cinema_seat(f, s as f64 * 0.5, 0.78, s as f64 * 0.86);
        }
        // the console between the seats: a padded lid, two cup holders, a soda, a bucket of popcorn
        fextrude!(
            f,
            "console",
            &rounded_rect(0.22, 0.6, 0.04, 2),
            0.5,
            (0, 0.25, -0.06),
            M.cinema_frame,
            plane = Facing::Top,
            bevel = 0.02,
            bevel_segments = 1
        );
        soft!(
            f,
            "console_lid",
            0.2,
            0.04,
            0.42,
            (0, 0.515, -0.14),
            M.leather,
            round = 0.018,
            flat = 1,
            round_segments = 1,
            resting = true
        );
        for z in [0.08, 0.17] {
            torus!(
                numbered("cup_ring"),
                0.045,
                0.006,
                at = f.at(0, 0.503, z),
                material = M.graphite,
                major_segments = 12,
                minor_segments = 3
            );
            fcyl!(
                f,
                numbered("cup_well"),
                0.042,
                0.004,
                (0, 0.5, z),
                M.dark,
                verts = 12
            );
        }
        cup(f, 0.0, 0.47, 0.08, 0.045, 0.16, M.soda);
        popcorn(f, 0.0, 0.535, -0.18, 0.075, 0.12, M.soda, 7);
        fextrude!(
            f,
            "remote",
            &rounded_rect(0.045, 0.16, 0.018, 2),
            0.018,
            (-0.3, 0.465, 0.1),
            M.dark,
            plane = Facing::Top,
            rot = (0, 0, 20),
            bevel = 0.004,
            bevel_segments = 1
        );
        // the screen (at the front edge until turned round), facing the seats; a pelmet over it for the curtains
        screen(f, -0.92, 0.92, 0.88, 1.42, SCREEN_Z, M.picture);
        fextrude!(
            f,
            "pelmet",
            &rounded_rect(1.6, 0.1, 0.02, 2),
            0.1,
            (0, 1.47, SCREEN_Z - 0.06),
            M.pelmet,
            bevel = 0.012,
            bevel_segments = 1
        );
        fbox!(
            f,
            "pelmet_bead",
            1.56,
            0.012,
            0.012,
            (0, 1.425, SCREEN_Z - 0.11),
            M.graphite,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the surround speakers at the screen's corners: rounded towers, two drivers and a port each, on spikes
        for x in [-0.86, 0.86] {
            fextrude!(
                f,
                numbered("speaker"),
                &rounded_rect(0.16, 0.16, 0.03, 2),
                0.8,
                (x, 0.44, 0.33),
                M.speaker_grey,
                plane = Facing::Top,
                bevel = 0.012,
                bevel_segments = 1
            );
            driver(f, &numbered("woofer"), x, 0.6, 0.25, 0.05, -1, 12);
            driver(f, &numbered("tweeter"), x, 0.76, 0.25, 0.028, -1, 12);
            fcyl!(
                f,
                numbered("port"),
                0.014,
                0.01,
                (x, 0.3, 0.248),
                M.dark,
                rot = (90, 0, 0),
                verts = 10
            );
            for dx in [-0.05, 0.05] {
                for dz in [0.28, 0.38] {
                    fcone!(
                        f,
                        numbered("spike"),
                        0.012,
                        0.004,
                        0.04,
                        (x + dx, 0.02, dz),
                        M.graphite,
                        verts = 6
                    );
                }
            }
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The two curtains, closed over the picture, each gathered by `curtains` to its own side.
    let zc = SCREEN_Z - 0.06;
    let mut parts = Vec::new();
    for (s, cname) in [(-1, "curtain_left"), (1, "curtain_right")] {
        let s = s as f64;
        let cur = new_parts(|| {
            pleats(
                f,
                cname,
                s * 0.01,
                s * 0.78,
                0.9,
                1.42,
                zc - 0.012,
                6,
                0.018,
                M.curtain,
                0.01,
            );
        });
        parts.push(hinged(
            join(&cur, cname, None),
            f.at(s * 0.78, 1.42, zc - 0.012),
        ));
    }
    turn_round(&meshes(), 0.0);
    let objs = settle_all(); // on its Model centre: the screen's back on the wall face
    for cur in parts {
        squash(
            cur,
            "curtains",
            (0.22, 1, 1),
            Squash {
                seconds: 1.2,
                ..Squash::default()
            },
        );
    }
    objs
}

// --- the Pixel Arcade console (1x1): the screen and console on the wall, a gaming loveseat for two in front -------

/// A game controller lying on `y` at (x, z), facing +z: a body with two grips, two sticks, a cross and four
/// buttons.
fn pad(f: &GameFrame, x: f64, y: f64, z: f64, turn: f64) {
    let body = smooth_outline(
        &[
            (-0.07, 0.03),
            (0.07, 0.03),
            (0.075, 0.0),
            (0.065, -0.05),
            (0.04, -0.055),
            (0.025, -0.02),
            (-0.025, -0.02),
            (-0.04, -0.055),
            (-0.065, -0.05),
            (-0.075, 0.0),
        ],
        2,
        true,
    );
    fextrude!(
        f,
        numbered("pad"),
        &body,
        0.026,
        (x, y + 0.013, z),
        M.console,
        plane = Facing::Top,
        rot = (0.0, 0.0, turn),
        bevel = 0.006,
        bevel_segments = 1
    );
    for dx in [-0.025, 0.025] {
        fcyl!(
            f,
            numbered("stick"),
            0.009,
            0.012,
            (x + dx, y + 0.032, z - 0.005),
            M.dark,
            verts = 8
        );
    }
    fbox!(
        f,
        numbered("cross"),
        0.024,
        0.004,
        0.008,
        (x - 0.045, y + 0.027, z + 0.008),
        M.dark
    );
    fbox!(
        f,
        numbered("cross"),
        0.008,
        0.004,
        0.024,
        (x - 0.045, y + 0.027, z + 0.008),
        M.dark
    );
    for (k, (dx, dz)) in [(0.045, 0.0), (0.045, 0.016), (0.037, 0.008), (0.053, 0.008)]
        .into_iter()
        .enumerate()
    {
        fcyl!(
            f,
            numbered("button"),
            0.004,
            0.005,
            (x + dx, y + 0.0285, z + dz),
            [M.soda, M.led, M.mustard, M.box_blue][k],
            verts = 6
        );
    }
}

pub fn game_console() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // the loveseat: a piped base, two stuffed seat cushions, a back with two cushions, rolled arms, splayed legs
        soft!(
            f,
            "base",
            0.88,
            0.24,
            0.56,
            (0, 0.17, -0.12),
            M.gamer,
            round = 0.06,
            piping = 0.005,
            flat = 1
        );
        for x in [-0.21, 0.21] {
            soft!(
                f,
                numbered("seat"),
                0.4,
                0.1,
                0.5,
                (x, 0.388, -0.08),
                M.gamer,
                round = 0.035,
                puff = 0.012,
                piping = 0.005,
                flat = 2
            );
            soft!(
                f,
                numbered("back_cushion"),
                0.38,
                0.36,
                0.13,
                (x, 0.6, -0.31),
                M.gamer,
                rot = (-10, 0, 0),
                round = 0.045,
                puff = 0.025,
                flat = 2
            );
        }
        soft!(
            f,
            "back",
            0.88,
            0.52,
            0.1,
            (0, 0.55, -0.355),
            M.gamer,
            rot = (-10, 0, 0),
            round = 0.045,
            flat = 1,
            round_segments = 1
        );
        for s in [-1, 1] {
            let s = s as f64;
            let roll: Vec<(f64, f64)> = (-40..230)
                .step_by(30)
                .map(|a| {
                    (
                        0.48 + 0.045 * sin(radians(a as f64)),
                        s * 0.04 * cos(radians(a as f64)) + s * 0.005,
                    )
                })
                .collect();
            let profile = [
                vec![(0.27, s * -0.035), (0.27, s * 0.035)],
                roll,
                vec![(0.44, s * -0.035)],
            ]
            .concat();
            let arm = fsweep!(
                f,
                numbered("arm"),
                &[(s * 0.4, 0, -0.39), (s * 0.4, 0, 0.15)],
                0,
                M.gamer,
                profile = profile,
                angle = 50
            );
            ease(arm, 60.0, 1);
            for lz in [-0.37, 0.13] {
                let pts = [
                    (s * 0.37, 0.06, lz),
                    (
                        s * 0.385,
                        0.0,
                        lz + 0.01 * (if lz > 0.0 { 1.0 } else { -1.0 }),
                    ),
                ];
                fsweep!(
                    f,
                    numbered("foot"),
                    &pts,
                    0.016,
                    M.dark,
                    verts = 8,
                    scale = [1.0, 0.7]
                );
            }
        }
        soft!(
            f,
            "pillow",
            0.3,
            0.28,
            0.1,
            (0.24, 0.64, -0.24),
            M.mustard,
            rot = (-10, 0, 20),
            round = 0.04,
            puff = 0.03,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
        for (x, t) in [(-0.2, 10.0), (0.2, -15.0)] {
            // the controllers on the seats
            pad(f, x, 0.45, 0.06, t);
        }
        // the screen (at the front edge until turned round) and the shelf between its posts, with the console and two
        // games
        screen(f, -0.4, 0.4, 0.85, 1.32, SCREEN_Z, M.mustard);
        fextrude!(
            f,
            "shelf",
            &rounded_rect(0.74, 0.12, 0.01, 1),
            0.03,
            (0, 0.62, 0.39),
            M.screen_back,
            plane = Facing::Top,
            bevel = 0.005,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "console",
            &rounded_rect(0.3, 0.1, 0.02, 2),
            0.06,
            (0, 0.665, 0.39),
            M.console,
            plane = Facing::Top,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "console_vent",
            0.24,
            0.004,
            0.06,
            (0, 0.6955, 0.39),
            M.dark
        );
        fbox!(f, "tray_slot", 0.14, 0.012, 0.004, (0, 0.67, 0.339), M.dark);
        fbox!(f, "power", 0.05, 0.006, 0.003, (0.1, 0.65, 0.339), M.led);
        fextrude!(
            f,
            "game_a",
            &rounded_rect(0.14, 0.18, 0.008, 1),
            0.02,
            (-0.27, 0.725, 0.43),
            M.box_red,
            rot = (0, 4, 0),
            bevel = 0.003,
            bevel_segments = 1
        );
        fextrude!(
            f,
            "game_b",
            &rounded_rect(0.14, 0.18, 0.008, 1),
            0.02,
            (-0.24, 0.725, 0.405),
            M.box_blue,
            rot = (0, -3, 0),
            bevel = 0.003,
            bevel_segments = 1
        );
        for (k, z) in [0.43, 0.405].into_iter().enumerate() {
            let k = k as i32;
            fbox!(
                f,
                numbered("game_art"),
                0.1,
                0.1,
                0.002,
                (-0.27 + 0.03 * k as f64, 0.74, z - 0.011),
                [M.mustard, M.led][k as usize],
                rot = (0, 4 - 7 * k, 0)
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The disc tray in the console's slot, facing the players: `tray` slides it out with a disc on it.
    let tray = new_parts(|| {
        fbox!(
            f,
            "tray",
            0.13,
            0.008,
            0.09,
            (0, 0.67, 0.385),
            M.dark,
            bevel = 0.002,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "tray_disc",
            0.04,
            0.003,
            (0, 0.6755, 0.385),
            M.disc,
            verts = 24
        );
        fcyl!(
            f,
            "tray_hub",
            0.008,
            0.004,
            (0, 0.677, 0.385),
            M.dark,
            verts = 8
        );
    });
    let tray = join(&tray, "tray", None);
    turn_round(&meshes(), 0.0);
    let objs = settle_all(); // on its Model centre: the screen's back on the wall face
    slide!(tray, "tray", (0, -0.09, 0), seconds = 0.6); // 9 cm towards the players (+Z game, once turned round)
    objs
}
