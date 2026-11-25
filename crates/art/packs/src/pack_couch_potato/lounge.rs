//! The Cloud Nine power recliner, the sofa-side snack fridge, the living-room workout game and the lava glow lamp.

use super::common::M;
use crate::ease::{EASE, ease_ends, eased};
use artkit::forms::rounded_rect;
use artkit::math::{Axis, P3, PI, cos, cosd, radians, sin, sind};
use artkit::motion::{Ease, Spin, spin};
use artkit::parts::{aside, hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, jitter, kit, loft, revolve, ring, settle, slide, soft,
    swing,
};

// --- the Cloud Nine power recliner (1x1): deep seat between rolled arms, the back and footrest power out --------
const BACK_HINGE: (f64, f64) = (0.42, -0.21); // (height, depth) the back tips about
const FOOT_HINGE: (f64, f64) = (0.4, 0.25); // and the footrest rises about

pub fn recliner() -> Vec<Obj> {
    let f = &GameFrame::new(0.03); // 3 cm forward of the wall: room for the back to tip
    let still = new_parts(|| {
        fextrude!(
            f,
            "plinth",
            &rounded_rect(0.6, 0.6, 0.06, 2),
            0.05,
            (0, 0.025, -0.06),
            M.plinth,
            plane = Facing::Top,
            bevel = 0.01,
            bevel_segments = 1
        );
        soft!(
            f,
            "base",
            0.58,
            0.3,
            0.6,
            (0, 0.2, -0.06),
            M.plush,
            round = 0.05,
            flat = 1,
            round_segments = 1
        );
        soft!(
            f,
            "seat",
            0.56,
            0.1,
            0.54,
            (0, 0.388, -0.02),
            M.plush,
            round = 0.04,
            puff = 0.012,
            piping = 0.005,
            flat = 2
        );
        for s in [-1, 1] {
            let s = s as f64;
            // the arm: a panel rising into a fat roll, swept front to back; a plinth-coloured cap on the roll's front
            // (the roll ends tangent to the panel's inside face, not tucked under it into a lip)
            let roll: Vec<(f64, f64)> = (-50..181)
                .step_by(28)
                .map(|a| {
                    (
                        0.56 + 0.085 * sin(radians(a as f64)),
                        s * 0.085 * cos(radians(a as f64)) + s * 0.005,
                    )
                })
                .collect();
            let profile = [
                vec![(0.06, s * -0.08), (0.06, s * 0.08)],
                roll,
                vec![(0.5, s * -0.08)],
            ]
            .concat();
            let arm = fsweep!(
                f,
                numbered("arm"),
                &[(s * 0.37, 0, -0.43), (s * 0.37, 0, 0.27)],
                0,
                M.plush,
                profile = profile,
                angle = 55
            );
            ease_ends(arm);
            fcyl!(
                f,
                numbered("roll_end"),
                0.06,
                0.012,
                (s * 0.375, 0.56, 0.272),
                M.plinth,
                rot = (90, 0, 0),
                verts = 14
            );
            fcyl!(
                f,
                numbered("roll_button"),
                0.018,
                0.008,
                (s * 0.375, 0.56, 0.281),
                M.plush,
                rot = (90, 0, 0),
                verts = 8
            );
        }
        artkit::torus!(
            "cup_ring",
            0.042,
            0.006,
            at = f.at(0.37, 0.648, 0.12),
            material = M.dark,
            major_segments = 12,
            minor_segments = 3
        );
        revolve!(
            "cup",
            &eased(
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.037, 0.12),
                    (0.034, 0.12),
                    (0.03, 0.11),
                    (0.0, 0.11)
                ],
                EASE
            ),
            at = f.at(0.37, 0.64, 0.12),
            material = M.teal,
            segments = 12
        );
        fextrude!(
            f,
            "remote",
            &rounded_rect(0.05, 0.14, 0.02, 2),
            0.02,
            (-0.37, 0.655, 0.1),
            M.dark,
            plane = Facing::Top,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            f,
            "remote_keys",
            0.02,
            0.004,
            0.1,
            (-0.37, 0.666, 0.11),
            M.slat
        );
        // a throw over the left arm: across the roll and hanging down its outside, soft and creased
        let t = 0.012;
        let pts = kit::fillet!(
            &[
                (-0.29, 0.62, -0.12),
                (-0.36, 0.665, -0.12),
                (-0.45, 0.64, -0.12),
                (-0.474, 0.56, -0.12),
                (-0.474, 0.3, -0.12),
            ],
            0.04
        );
        let throw = fsweep!(
            f,
            "throw",
            &pts,
            0,
            M.throw,
            profile = rounded_rect(t, 0.4, t / 2.0, 2),
            up = (0, 0, 1)
        );
        jitter!(throw, 0.005, size = 0.1, seed = 3.0, up = 0.5);
    });
    join(&still, "body", Some(P3::ZERO));
    let (by, bz) = BACK_HINGE;
    let back = new_parts(|| {
        // the back: a shell, three pillowed rolls and a head pillow
        soft!(
            f,
            "back",
            0.6,
            0.56,
            0.13,
            (0, 0.67, -0.28),
            M.plush,
            rot = (-8, 0, 0),
            round = 0.06,
            flat = 1,
            round_segments = 1
        );
        for (y, z, hh) in [(0.55, -0.2, 0.15), (0.71, -0.22, 0.16)] {
            soft!(
                f,
                numbered("back_roll"),
                0.5,
                hh,
                0.1,
                (0, y, z),
                M.plush,
                rot = (-8, 0, 0),
                round = 0.04,
                puff = 0.022,
                flat = 2
            );
        }
        soft!(
            f,
            "head_pillow",
            0.46,
            0.13,
            0.11,
            (0, 0.88, -0.21),
            M.plush,
            rot = (-8, 0, 0),
            round = 0.05,
            puff = 0.02,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
    });
    let back = hinged(join(&back, "back", None), f.at(0, by, bz));
    let (fy, fz) = FOOT_HINGE;
    let foot = new_parts(|| {
        // the footrest, folded down under the seat's front
        soft!(
            f,
            "footrest",
            0.5,
            0.22,
            0.08,
            (0, fy - 0.12, fz + 0.05),
            M.plush,
            round = 0.035,
            flat = 1
        );
    });
    let foot = hinged(join(&foot, "footrest", None), f.at(0, fy, fz));
    let objs = settle!(centre = false);
    swing!(back, "recline", -11.0, seconds = 1.2, axis = Axis::X);
    swing!(foot, "recline", -80.0, seconds = 1.2, axis = Axis::X);
    objs
}

// --- the sofa-side snack fridge (1x1): a mini fridge with a padded lid, a takeout screen on its door ------------
const FR: (f64, f64, f64, f64, f64) = (0.0, -0.02, 0.56, 0.37, 0.46); // x, depth, width, height, depth of the cabinet

/// A drinks can standing on `y`: a body necked in at the top and the foot, its lid and ring pull.
#[allow(clippy::too_many_arguments)]
fn can(f: &GameFrame, x: f64, y: f64, z: f64, material: artkit::materials::Mat, r: f64, h: f64) {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    let rows = [
        (0.0, 0.8),
        (0.008, 1.0),
        (h - 0.012, 1.0),
        (h - 0.002, 0.84),
        (h, 0.84),
    ];
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(hh, k)| ring!(r * k, r * k, z = bz + hh, verts = 10, at = (bx, by)))
        .collect();
    loft!(numbered("can"), &sections, material = material);
    fcyl!(
        f,
        numbered("can_top"),
        r * 0.8,
        0.002,
        (x, y + h, z),
        M.chrome,
        verts = 10
    );
}

pub fn snack_fridge() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (x, z, w, h, d) = FR;
    let (z0, z1) = (z - d / 2.0, z + d / 2.0);
    let t = 0.025;
    let still = new_parts(|| {
        // the cabinet, open at the front: back, sides, floor and top, a white liner inside with a shelf of snacks
        fbox!(
            f,
            "cab_back",
            w,
            h - 0.03,
            t,
            (x, 0.03 + (h - 0.03) / 2.0, z0 + t / 2.0),
            M.mint,
            bevel = 0.008,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            fbox!(
                f,
                numbered("cab_side"),
                t,
                h - 0.03,
                d - 0.03,
                (
                    x + s * (w / 2.0 - t / 2.0),
                    0.03 + (h - 0.03) / 2.0,
                    z - 0.015
                ),
                M.mint,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "cab_floor",
            w,
            t,
            d - 0.03,
            (x, 0.03 + t / 2.0, z - 0.015),
            M.mint,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "cab_top",
            w,
            t,
            d - 0.03,
            (x, h - t / 2.0, z - 0.015),
            M.mint,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "liner",
            w - 2.0 * t,
            h - 0.03 - 2.0 * t,
            0.004,
            (x, 0.03 + (h - 0.03) / 2.0, z0 + t + 0.002),
            M.inside
        );
        fbox!(
            f,
            "liner_floor",
            w - 2.0 * t,
            0.004,
            d - 0.06,
            (x, 0.03 + t + 0.002, z - 0.03),
            M.inside
        );
        fbox!(
            f,
            "shelf",
            w - 2.0 * t,
            0.012,
            d - 0.08,
            (x, 0.2, z - 0.04),
            M.inside
        );
        fbox!(
            f,
            "shelf_lip",
            w - 2.0 * t,
            0.02,
            0.006,
            (x, 0.206, z1 - 0.043),
            M.chrome
        );
        for (k, (cx, m)) in [
            (-0.17, M.can_a),
            (-0.1, M.can_b),
            (-0.03, M.can_c),
            (0.04, M.can_a),
        ]
        .into_iter()
        .enumerate()
        {
            can(f, cx, 0.206, -0.02 + 0.03 * (k % 2) as f64, m, 0.03, 0.11);
        }
        fbox!(
            f,
            "takeout_box",
            0.16,
            0.08,
            0.14,
            (0.14, 0.247, -0.03),
            M.takeout,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "takeout_band",
            0.161,
            0.02,
            0.141,
            (0.14, 0.25, -0.03),
            M.menu
        );
        fbox!(
            f,
            "pizza",
            0.4,
            0.035,
            0.3,
            (0, 0.075, -0.06),
            M.takeout,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "pizza_lid_line",
            0.402,
            0.003,
            0.302,
            (0, 0.085, -0.06),
            M.slat
        );
        fbox!(
            f,
            "pizza_logo",
            0.12,
            0.002,
            0.08,
            (0, 0.093, -0.02),
            M.can_a
        );
        fbox!(
            f,
            "trim",
            w - 0.08,
            0.012,
            0.02,
            (x, h - 0.03, z1 - 0.004),
            M.mint_trim,
            bevel = 0.004,
            bevel_segments = 1
        );
        for fx in [x - w / 2.0 + 0.06, x + w / 2.0 - 0.06] {
            for fz in [z0 + 0.06, z1 - 0.06] {
                revolve!(
                    numbered("foot"),
                    &[
                        (0.0, 0.0),
                        (0.018, 0.0),
                        (0.022, 0.02),
                        (0.02, 0.03),
                        (0.0, 0.03)
                    ],
                    at = f.at(fx, 0, fz),
                    material = M.mint_trim,
                    segments = 8
                );
            }
        }
        fbox!(
            f,
            "kick",
            0.5,
            0.02,
            0.04,
            (x, 0.04, z1 - 0.02),
            M.graphite,
            bevel = 0.006,
            bevel_segments = 1
        );
        // the padded lid (the ottoman's top, 0.45), piped, and a bowl of crisps on it
        soft!(
            f,
            "lid",
            0.58,
            0.07,
            0.5,
            (x, 0.405, z),
            M.lid,
            round = 0.03,
            puff = 0.01,
            piping = 0.006,
            flat = 2
        );
        for bx in [-0.12, 0.12] {
            fsphere!(
                f,
                numbered("lid_button"),
                0.01,
                (bx, 0.452, z),
                M.lid,
                scale = (1, 1, 0.5),
                segments = 6,
                rings = 4
            );
        }
        revolve!(
            "bowl",
            &[
                &[(0.0, 0.0)][..],
                // (its rim rolled over)
                &eased(
                    &[(0.06, 0.0), (0.11, 0.06), (0.105, 0.062), (0.055, 0.008)],
                    EASE
                ),
                &[(0.0, 0.008)],
            ]
            .concat(),
            at = f.at(0.13, 0.45, -0.12),
            material = M.bowl,
            segments = 16
        );
        for k in 0..8 {
            let a = (45 * k) as f64;
            let r = if k % 2 != 0 { 0.05 } else { 0.025 };
            let crisp = fsphere!(
                f,
                numbered("crisp"),
                0.03,
                (
                    0.13 + r * cosd(a),
                    0.5 + 0.006 * (k % 3) as f64,
                    -0.12 + r * sind(a)
                ),
                M.crisps,
                scale = (1, 1, 0.3),
                rot = (20 * (k % 3), 30 * k, 0),
                segments = 8,
                rings = 4
            );
            jitter!(crisp, 0.004, size = 0.02, seed = k as f64 * 1.3);
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The door with its takeout screen and handle, hinged on its right edge: `fridge_open` swings it open.
    let hx = x + w / 2.0 - 0.01;
    let door = new_parts(|| {
        fextrude!(
            f,
            "door",
            &rounded_rect(w - 0.02, h - 0.06, 0.02, 2),
            0.03,
            (x, 0.03 + (h - 0.03) / 2.0, z1 + 0.003),
            M.mint,
            bevel = 0.008,
            bevel_segments = 1
        );
        let seal = rounded_rect(w - 0.07, h - 0.1, 0.03, 2);
        let pts: Vec<(f64, f64, f64)> = seal
            .iter()
            .map(|&(u, v)| (x + u, 0.03 + (h - 0.03) / 2.0 + v, z1 - 0.014))
            .collect();
        fsweep!(
            f,
            "door_seal",
            &pts,
            0.006,
            M.graphite,
            verts = 4,
            closed = true
        );
        fbox!(
            f,
            "door_liner",
            w - 0.1,
            h - 0.13,
            0.008,
            (x, 0.03 + (h - 0.03) / 2.0, z1 - 0.014),
            M.inside
        );
        fextrude!(
            f,
            "door_screen",
            &rounded_rect(0.18, 0.12, 0.012, 2),
            0.006,
            (0.08, 0.24, z1 + 0.02),
            M.bezel,
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "door_menu",
            0.15,
            0.09,
            0.002,
            (0.08, 0.24, z1 + 0.0235),
            M.menu
        );
        for k in 0..3 {
            fbox!(
                f,
                numbered("door_item"),
                0.06,
                0.012,
                0.002,
                (0.1, 0.27 - 0.025 * k as f64, z1 + 0.0245),
                M.takeout
            );
        }
        fbox!(
            f,
            "door_pick",
            0.04,
            0.03,
            0.002,
            (0.04, 0.26, z1 + 0.0245),
            M.can_a
        );
        kit::pull!(
            f,
            "handle",
            x - w / 2.0 + 0.06,
            0.2,
            z1 + 0.018,
            0.16,
            M.chrome,
            r = 0.007,
            out = 0.0345,
            upright = true
        );
    });
    let door = hinged(join(&door, "door", None), f.at(hx, 0.2, z1 + 0.003));
    let objs = settle!(centre = false);
    swing!(
        door,
        "fridge_open",
        -100.0,
        seconds = 0.9,
        ease = Ease::BackOut
    );
    objs
}

// --- the living-room workout game (1x1): a TV on a slim stand, a step mat and a balance board before it ---------
pub fn fitness_game() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        for y in [0.2, 0.56] {
            fextrude!(
                f,
                numbered("stand_shelf"),
                &rounded_rect(0.8, 0.3, 0.02, 2),
                0.04,
                (0, y, -0.28),
                M.oak,
                plane = Facing::Top,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        for x in [-0.37, 0.37] {
            fextrude!(
                f,
                numbered("stand_side"),
                &rounded_rect(0.28, 0.58, 0.03, 2),
                0.04,
                (x, 0.29, -0.28),
                M.fit_stand,
                plane = Facing::Side,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "stand_back",
            0.7,
            0.3,
            0.012,
            (0, 0.38, -0.425),
            M.fit_stand
        );
        // the TV on its foot, a workout on the screen, the sensor bar on top
        let (y0, w, h) = (0.58, 0.8, 0.53);
        let ph = h - 0.07;
        let py = y0 + 0.07 + ph / 2.0;
        let zc = -0.35;
        fextrude!(
            f,
            "tv_foot",
            &rounded_rect(0.3, 0.14, 0.03, 2),
            0.014,
            (0, y0 + 0.007, zc),
            M.tv_frame,
            plane = Facing::Top,
            bevel = 0.004,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "tv_neck",
            &[(0, y0 + 0.01, zc - 0.03), (0, y0 + 0.08, zc - 0.03)],
            0,
            M.tv_frame,
            profile = rounded_rect(0.06, 0.03, 0.01, 1),
            up = (1, 0, 0)
        );
        fextrude!(
            f,
            "tv",
            &rounded_rect(w, ph, 0.015, 2),
            0.04,
            (0, py, zc - 0.01),
            M.tv_frame,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "tv_screen",
            w - 0.03,
            ph - 0.03,
            0.004,
            (0, py + 0.003, zc + 0.011),
            M.tv_screen
        );
        fbox!(
            f,
            "tv_band",
            w - 0.03,
            0.1,
            0.003,
            (0, py - ph / 2.0 + 0.07, zc + 0.0135),
            M.tv_floor
        );
        // the coach on the screen: a head, a body and arms raised
        fcyl!(
            f,
            "coach_head",
            0.03,
            0.003,
            (-0.12, py + 0.06, zc + 0.0145),
            M.tv_white,
            rot = (90, 0, 0),
            verts = 12
        );
        fextrude!(
            f,
            "coach",
            &[
                (-0.03, -0.09),
                (0.03, -0.09),
                (0.03, 0.0),
                (0.07, 0.07),
                (0.055, 0.08),
                (0.02, 0.03),
                (-0.02, 0.03),
                (-0.055, 0.08),
                (-0.07, 0.07),
                (-0.03, 0.0)
            ],
            0.003,
            (-0.12, py - 0.04, zc + 0.0145),
            M.tv_orange
        );
        fbox!(
            f,
            "score_a",
            0.2,
            0.04,
            0.003,
            (0.2, py + 0.14, zc + 0.014),
            M.tv_white
        );
        fbox!(
            f,
            "score_b",
            0.14,
            0.03,
            0.003,
            (0.17, py + 0.09, zc + 0.014),
            M.picture_b
        );
        fextrude!(
            f,
            "sensor",
            &rounded_rect(0.3, 0.04, 0.02, 3),
            0.05,
            (0, y0 + h + 0.025, zc - 0.005),
            M.tv_white,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "sensor_glass",
            0.2,
            0.02,
            0.004,
            (0, y0 + h + 0.025, zc + 0.021),
            M.bezel
        );
        fsphere!(
            f,
            "sensor_eye",
            0.012,
            (-0.08, y0 + h + 0.025, zc + 0.022),
            M.soda,
            scale = (1, 1, 0.4),
            segments = 8,
            rings = 4
        );
        // on the shelves: a folded towel and a water bottle; on the floor: the step mat and a kettlebell
        soft!(
            f,
            "towel",
            0.3,
            0.03,
            0.2,
            (-0.2, 0.235, -0.28),
            M.towel,
            round = 0.012,
            puff = 0.004,
            flat = 1
        );
        soft!(
            f,
            "towel_top",
            0.3,
            0.022,
            0.18,
            (-0.2, 0.26, -0.29),
            M.towel,
            round = 0.01,
            puff = 0.004,
            flat = 1
        );
        let P3 {
            x: bx,
            y: by,
            z: bz,
        } = f.at(0.25, 0.22, -0.26);
        let sections: Vec<Vec<P3>> = [
            (0.0, 0.03),
            (0.004, 0.035),
            (0.08, 0.032),
            (0.15, 0.035),
            (0.17, 0.03),
            (0.19, 0.02),
        ]
        .iter()
        .map(|&(hh, r)| ring!(r, r, z = bz + hh, verts = 12, at = (bx, by)))
        .collect();
        loft!("bottle", &sections, material = M.bottle);
        revolve!(
            "bottle_cap",
            &[
                (0.0, 0.0),
                (0.022, 0.0),
                (0.022, 0.025),
                (0.012, 0.035),
                (0.012, 0.05),
                (0.0, 0.05)
            ],
            at = (bx, by, bz + 0.19),
            material = M.tv_white,
            segments = 10
        );
        soft!(
            f,
            "mat",
            0.7,
            0.012,
            0.56,
            (0, 0.006, 0.17),
            M.fit_mat,
            round = 0.006,
            flat = 1,
            round_segments = 1
        );
        fbox!(
            f,
            "mat_inner",
            0.62,
            0.004,
            0.48,
            (0, 0.0135, 0.17),
            M.mat_inner,
            bevel = 0.002,
            bevel_segments = 1
        );
        // the step pads printed round its edge: an arrow at each side, pointing out
        let arrow = [
            (0.0, 0.05),
            (0.045, 0.0),
            (0.018, 0.0),
            (0.018, -0.04),
            (-0.018, -0.04),
            (-0.018, 0.0),
            (-0.045, 0.0),
        ];
        for (k, (ax, az, a)) in [
            (0.0, 0.37, 0.0),
            (0.0, -0.03, 180.0),
            (0.24, 0.17, -90.0),
            (-0.24, 0.17, 90.0),
        ]
        .into_iter()
        .enumerate()
        {
            let (c, s) = (cosd(a), sind(a));
            let outline: Vec<(f64, f64)> = arrow
                .iter()
                .map(|&(u, v)| (ax + u * c - v * s, az - (u * s + v * c)))
                .collect();
            fextrude!(
                f,
                numbered("arrow"),
                &outline,
                0.003,
                (0, 0.0165, 0),
                [M.tv_orange, M.board][k % 2],
                plane = Facing::Top
            );
        }
        // a pair of dumbbells on the bottom shelf
        for dz in [-0.33, -0.23] {
            fsweep!(
                f,
                numbered("dumbbell_bar"),
                &[(-0.02, 0.25, dz), (0.16, 0.25, dz)],
                0.011,
                M.grip,
                verts = 8
            );
            for dx in [-0.01, 0.15] {
                fcyl!(
                    f,
                    numbered("dumbbell_end"),
                    0.03,
                    0.04,
                    (dx, 0.25, dz),
                    M.weight,
                    rot = (0, 90, 0),
                    verts = 8,
                    bevel = 0.006,
                    bevel_segments = 1
                );
            }
        }
        // the kettlebell: a round bell, its flat foot, a handle arching over it
        let (kx, kz) = (0.38, 0.36);
        let P3 {
            x: bx,
            y: by,
            z: bz,
        } = f.at(kx, 0, kz);
        let sections: Vec<Vec<P3>> = [
            (0.0, 0.05),
            (0.01, 0.065),
            (0.04, 0.085),
            (0.08, 0.085),
            (0.11, 0.07),
            (0.13, 0.04),
            (0.136, 0.02),
        ]
        .iter()
        .map(|&(hh, r)| ring!(r, r, z = bz + hh, verts = 14, at = (bx, by)))
        .collect();
        loft!("kettlebell", &sections, material = M.weight);
        kit::tube!(
            f,
            "kettle_handle",
            &kit::fillet!(
                &[
                    (kx - 0.05, 0.11, kz),
                    (kx - 0.055, 0.19, kz),
                    (kx + 0.055, 0.19, kz),
                    (kx + 0.05, 0.11, kz),
                ],
                0.035
            ),
            0.012,
            M.weight,
            verts = 8,
            round = 1
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The balance board on its rocker, on the mat: `board` tilts it 10° to one side.
    let board = new_parts(|| {
        fextrude!(
            f,
            "board_deck",
            &rounded_rect(0.52, 0.4, 0.18, 6),
            0.025,
            (0, 0.1, 0.2),
            M.board,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 1
        );
        for dx in [-0.14, 0.14] {
            fextrude!(
                f,
                numbered("board_grip"),
                &rounded_rect(0.12, 0.16, 0.03, 2),
                0.004,
                (dx, 0.1135, 0.2),
                M.grip,
                plane = Facing::Top
            );
        }
        fcyl!(
            f,
            "board_rocker",
            0.045,
            0.06,
            (0, 0.057, 0.2),
            M.grip,
            rot = (90, 0, 0),
            verts = 16
        );
        fbox!(f, "board_mount", 0.08, 0.02, 0.07, (0, 0.082, 0.2), M.grip);
    });
    let board = hinged(join(&board, "board", None), f.at(0, 0.012, 0.2));
    let objs = settle!(centre = false);
    swing!(board, "board", 10.0, seconds = 0.8, axis = Axis::Y);
    objs
}

// --- the lava glow lamp (1x1): a floor pouf to zone out on, a side table with a big lava lamp --------------------
// The pouf is turned to face the lamp at its front right: the sitter faces it (content `places`, facing TURN°),
// coming from the tile to its left. 2026-10-21: the lamp stood behind the sitter's shoulder (I-packs-11).
const LAMP: (f64, f64) = (0.32, 0.25); // the side table's middle (x, depth)
const POUF: (f64, f64) = (-0.04, -0.03); // the pouf's middle (x, depth): the sitter's place
const TURN: f64 = 50.0; // the way the pouf and its sitter face, degrees from the front towards the right
const VESSEL: (f64, f64) = (0.65, 0.42); // the glass's foot height and height

/// The glass's inside radius at height `y` (it bulges a third of the way up).
fn vessel_r(y: f64) -> f64 {
    let (y0, h) = VESSEL;
    let u = (y - y0) / h;
    if u < 0.8 {
        // (Python's max and min: the first unless the second is strictly beyond it)
        let lo = if u > 0.0 { u } else { 0.0 };
        let k = lo / 0.8;
        0.046 + 0.03 * sin(PI * if k < 1.0 { k } else { 1.0 })
    } else {
        0.046 - (u - 0.8) * 0.02
    }
}

/// A blob of wax: a round drop stretched upright, wobbled.
#[allow(clippy::too_many_arguments)]
fn blob(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64, seed: f64, stretch: f64) -> Obj {
    let b = fsphere!(
        f,
        name,
        r,
        (x, y, z),
        M.wax,
        scale = (1.0, 1.0, stretch),
        segments = 10,
        rings = 6
    );
    jitter!(b, r * 0.12, size = r * 1.2, seed = seed)
}

pub fn lava_lamp() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (tx, tz) = LAMP;
    let (y0, h) = VESSEL;
    // the pouf the lamp is zoned out from and the rug before it: drawn, not the lamp's size (as a desk's chair)
    let pouf = new_parts(|| {
        let (px, pz) = POUF;
        let (fx, fz) = (sind(TURN), cosd(TURN)); // the way it faces (game x, depth)
        soft!(
            f,
            "pouf",
            0.62,
            0.36,
            0.6,
            (px, 0.18, pz),
            M.pouf,
            rot = (0.0, 0.0, -TURN),
            round = 0.13,
            flat = 2
        );
        soft!(
            f,
            "pouf_top",
            0.54,
            0.1,
            0.52,
            (px, 0.385, pz),
            M.pouf,
            rot = (0.0, 0.0, -TURN),
            round = 0.045,
            puff = 0.015,
            piping = 0.007,
            flat = 2,
            pinch = 0.0
        ); // (turned: its corners are the model's outermost points)
        fsphere!(
            f,
            "pouf_button",
            0.016,
            (px, 0.452, pz),
            M.pouf,
            scale = (1, 1, 0.5),
            segments = 8,
            rings = 4
        );
        soft!(
            f,
            "lean",
            0.5,
            0.36,
            0.14,
            (px - 0.26 * fx, 0.6, pz - 0.26 * fz),
            M.pouf,
            rot = (-12.0, 0.0, -TURN),
            round = 0.06,
            puff = 0.012,
            flat = 2,
            pinch = 0.0
        );
        fcyl!(
            f,
            "rug",
            0.2,
            0.012,
            (-0.22, 0.006, 0.27),
            M.rug,
            verts = 24,
            bevel = 0.004,
            bevel_segments = 1
        );
    });
    aside(&pouf, "pouf");
    let still = new_parts(|| {
        // the side table: a turned stem on a round foot, a lipped oak top
        fcyl!(
            f,
            "table_top",
            0.13,
            0.03,
            (tx, 0.5, tz),
            M.oak,
            verts = 20,
            bevel = 0.008,
            bevel_segments = 1
        );
        revolve!(
            "table_stem",
            &[
                (0.0, 0.0),
                (0.1, 0.0),
                (0.1, 0.012),
                (0.08, 0.02),
                (0.022, 0.035),
                (0.018, 0.2),
                (0.024, 0.3),
                (0.018, 0.46),
                (0.04, 0.485),
                (0.0, 0.485)
            ],
            at = f.at(tx, 0, tz),
            material = M.table_white,
            segments = 14
        );
        // the lamp: a tapered metal foot, the glass, a cone cap; a pool of wax at the bottom
        revolve!(
            "lamp_foot",
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.072, 0.008),
                (0.05, 0.12),
                (0.052, 0.13),
                (0.045, 0.14),
                (0.0, 0.14)
            ],
            at = f.at(tx, 0.515, tz),
            material = M.lamp_metal,
            segments = 16
        );
        let mut prof = vec![(0.0, 0.0)];
        prof.extend((0..13).map(|k| {
            (
                vessel_r(y0 + h * k as f64 / 12.0) + 0.003,
                h * k as f64 / 12.0,
            )
        }));
        prof.push((0.0, h));
        revolve!(
            "vessel",
            &prof,
            at = f.at(tx, y0, tz),
            material = M.vessel,
            segments = 20
        );
        revolve!(
            "lamp_cap",
            &[
                (0.0, 0.0),
                (0.046, 0.0),
                (0.044, 0.01),
                (0.02, 0.08),
                (0.014, 0.085),
                (0.0, 0.085)
            ],
            at = f.at(tx, y0 + h, tz),
            material = M.lamp_metal,
            segments = 14
        );
        blob(f, "wax_pool", tx, y0 + 0.02, tz, 0.045, 1.0, 0.35);
    });
    join(&still, "body", Some(P3::ZERO));
    // The wax that rises while a resident zones out (`blobs`, slow) ...
    let rise = new_parts(|| {
        for (k, (dx, y, r)) in [
            (0.0, 0.7, 0.03),
            (0.012, 0.79, 0.024),
            (-0.016, 0.745, 0.018),
        ]
        .into_iter()
        .enumerate()
        {
            blob(f, &numbered("blob"), tx + dx, y, tz, r, 2.0 + k as f64, 1.3);
        }
    });
    let rise = hinged(join(&rise, "blobs", None), f.at(tx, y0, tz));
    // ... and the wax that drifts round all the time near the camera (`drift`, ambient)
    let drift = new_parts(|| {
        for (k, (a, y, r)) in [(30.0, 0.9, 0.022), (200.0, 0.98, 0.017)]
            .into_iter()
            .enumerate()
        {
            blob(
                f,
                &numbered("drift"),
                tx + 0.022 * cosd(a),
                y,
                tz + 0.022 * sind(a),
                r,
                7.0 + k as f64,
                1.3,
            );
        }
    });
    let drift = hinged(join(&drift, "drift", None), f.at(tx, y0, tz));
    let objs = settle!(centre = false); // built in place: the sitter's place is content `places`
    slide!(rise, "blobs", (0, 0, 0.14), seconds = 2.5);
    spin(
        drift,
        "drift",
        Spin {
            turns: 1.0,
            seconds: 8.0,
            ..Spin::default()
        },
    );
    objs
}
