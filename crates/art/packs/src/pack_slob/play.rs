//! The spritz & go station and the pixel-blaster arcade cabinet.

use super::common::{_box, _can, _wrapper, M, POP, WALL};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{Axis, P3};
use artkit::parts::{aside, hinged, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, jitter, loft, revolve, ring, rock, settle, soft, swing,
};

// --- the spritz & go station (1x1): a narrow shelf unit with a round mirror and an arsenal of sprays ----------
const SPRAY: (f64, f64, f64) = (0.13, 0.935, -0.28); // the spray bottle's foot (x, height, depth)
const BOARD_TOP: f64 = 1.62; // the back board's top, over the mirror

/// A spray can: a body necked into its shoulder, a cap with its nozzle.
#[allow(clippy::too_many_arguments)]
fn _aerosol(f: &GameFrame, name: &str, x: f64, z: f64, y: f64, h: f64, material: Mat) {
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.026, 0.0),
            (0.03, 0.008),
            (0.03, h - 0.03),
            (0.022, h - 0.01),
            (0.014, h),
            (0.0, h)
        ],
        at = f.at(x, y, z),
        material = material,
        segments = 12
    );
    revolve!(
        format!("{name}_cap"),
        &[
            (0.0, 0.0),
            (0.022, 0.0),
            (0.022, 0.03),
            (0.016, 0.04),
            (0.0, 0.04)
        ],
        at = f.at(x, y + h - 0.004, z),
        material = M.white,
        segments = 10
    );
}

/// A bottle lofted through rounded rings [(height, scale)] from its foot at Blender (bx, by, bz).
fn bottle_rings(b: P3, rx: f64, ry: f64, verts: usize, rings: &[(f64, f64)]) -> Vec<Vec<P3>> {
    rings
        .iter()
        .map(|&(hh, k)| {
            ring!(
                rx * k,
                ry * k,
                z = b.z + hh,
                verts = verts,
                square = 3,
                at = (b.x, b.y)
            )
        })
        .collect()
}

pub fn spritz_station() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _box(
            f,
            "top",
            0.6,
            0.03,
            0.26,
            (0, 0.92, WALL + 0.13),
            M.gloss_white,
            10.0,
            24,
            0.006,
        );
        _box(
            f,
            "lower_shelf",
            0.56,
            0.03,
            0.24,
            (0, 0.42, WALL + 0.13),
            M.gloss_white,
            10.0,
            24,
            0.006,
        );
        for x in [-0.28, 0.28] {
            soft!(
                f,
                format!("side{x}"),
                0.04,
                0.92,
                0.24,
                (x, 0.46, WALL + 0.13),
                M.shelf,
                round = 0.012,
                round_segments = 1
            );
        }
        // a back board rising from the top carries the round mirror, standing free or on the wall (2026-10-21: the
        // mirror hung on the air over a free-standing station)
        fbox!(
            f,
            "backboard",
            0.6,
            BOARD_TOP - 0.935,
            0.018,
            (0, (0.935 + BOARD_TOP) / 2.0, WALL + 0.009),
            M.shelf,
            bevel = 0.008,
            bevel_segments = 1
        );
        revolve!(
            "mirror_frame",
            &[
                (0.19, 0.0),
                (0.23, 0.0),
                (0.234, 0.01),
                (0.226, 0.022),
                (0.2, 0.024),
                (0.19, 0.016)
            ],
            at = f.at(0, 1.35, WALL + 0.018),
            material = M.shelf,
            segments = 32,
            rot = (90, 0, 0)
        );
        fcyl!(
            f,
            "mirror",
            0.2,
            0.01,
            (0, 1.35, WALL + 0.032),
            M.mirror,
            rot = (90, 0, 0),
            verts = 32
        );
        for (i, (m, hh)) in [
            (M.pop_blue, 0.22),
            (M.pop_red, 0.18),
            (M.pop_yellow, 0.24),
            (M.pop_green, 0.2),
        ]
        .into_iter()
        .enumerate()
        {
            _aerosol(
                f,
                &format!("aerosol{i}"),
                -0.24 + i as f64 * 0.09,
                -0.3,
                0.935,
                hh,
                m,
            );
        }
        for (i, m) in [M.pop_purple, M.pop_blue, M.pop_red]
            .into_iter()
            .enumerate()
        {
            let b = f.at(-0.18 + i as f64 * 0.12, 0.435, -0.3);
            let rings = bottle_rings(
                b,
                0.035,
                0.03,
                12,
                &[
                    (0.0, 0.95),
                    (0.01, 1.0),
                    (0.12, 1.0),
                    (0.15, 0.5),
                    (0.18, 0.4),
                ],
            );
            loft!(format!("bottle{i}"), &rings, material = m);
            fcyl!(
                f,
                format!("bottle_cap{i}"),
                0.016,
                0.025,
                (-0.18 + i as f64 * 0.12, 0.627, -0.3),
                M.white,
                verts = 8
            );
        }
        for (k, m) in [M.towel, M.towel_green].into_iter().enumerate() {
            soft!(
                f,
                format!("towels{k}"),
                0.24,
                0.04,
                0.2,
                (0.16, 0.457 + 0.042 * k as f64, -0.3),
                m,
                round = 0.016,
                puff = 0.004,
                flat = 1,
                round_segments = 1
            );
        }
        // the towel over a bar under the top's front edge (it hung on nothing at the side)
        fsweep!(
            f,
            "towel_bar",
            &[
                (-0.02, 0.88, -0.165),
                (-0.02, 0.87, -0.155),
                (0.26, 0.87, -0.155),
                (0.26, 0.88, -0.165)
            ],
            0.007,
            M.chrome,
            verts = 6
        );
        fsweep!(
            f,
            "towel_hung",
            &[
                (0.12, 0.47, -0.162),
                (0.12, 0.86, -0.162),
                (0.12, 0.882, -0.155),
                (0.12, 0.872, -0.143),
                (0.12, 0.62, -0.14)
            ],
            0.0,
            M.towel_green,
            profile = rounded_rect(0.008, 0.26, 0.004, 1),
            up = (0, 1, 0),
            resolution = 6 // (enough steps that its fold over the bar rolls round)
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the washing heaped on the floor in front, a sleeve trailing, a can: drawn, not the station's size
    let heaped = new_parts(|| {
        for (k, (x, z, r, m)) in [
            (-0.112, 0.14, 0.2, M.laundry),
            (0.08, 0.234, 0.13, M.laundry_red),
        ]
        .into_iter()
        .enumerate()
        {
            let heap = soft!(
                f,
                format!("laundry{k}"),
                r * 1.75,
                r * 0.9,
                r * 1.7,
                (x, r * 0.45 + 0.03, z),
                m,
                round = r * 0.4,
                puff = (0.02, 0.03, 0.02),
                flat = 3,
                round_segments = 1,
                pinch = 0.0
            ); // (a heap, not a cushion)
            jitter!(heap, 0.02, size = 0.08, seed = k as f64 + 1.0, up = 0.5);
            let floor = -r * 0.45 - 0.03 + 0.001;
            heap.edit(|o| {
                // (flat on the floor)
                for v in &mut o.mesh.verts {
                    if (v.z as f64) < floor {
                        v.z = floor as f32;
                    }
                }
            });
        }
        fsweep!(
            f,
            "laundry_sleeve",
            &[(-0.2, 0.12, 0.22), (-0.24, 0.03, 0.3), (-0.26, 0.022, 0.34)],
            0.02,
            M.laundry,
            verts = 6,
            resolution = 2
        );
        _can(f, "can", 0.32, 0.3, 0.0, M.pop_yellow, true, -30.0);
    });
    aside(&heaped, "heaped");
    // The trigger spray bottle, hinged on its foot: `spritz` tips it 35 degrees towards the front.
    let (sx, sy, sz) = SPRAY;
    let spray = new_parts(|| {
        let b = f.at(sx, sy, sz);
        let rings = bottle_rings(
            b,
            0.036,
            0.03,
            14,
            &[
                (0.0, 0.95),
                (0.008, 1.0),
                (0.13, 1.0),
                (0.16, 0.55),
                (0.18, 0.4),
                (0.19, 0.4),
            ],
        );
        loft!("spray_bottle", &rings, material = M.pop_orange);
        soft!(
            f,
            "spray_head",
            0.032,
            0.045,
            0.075,
            (sx, sy + 0.21, sz + 0.015),
            M.white,
            round = 0.012,
            round_segments = 1
        );
        fsweep!(
            f,
            "spray_trigger",
            &[
                (sx, sy + 0.2, sz + 0.03),
                (sx, sy + 0.17, sz + 0.045),
                (sx, sy + 0.15, sz + 0.04)
            ],
            0.0,
            M.white,
            profile = [
                (-0.005, -0.006),
                (0.005, -0.006),
                (0.005, 0.006),
                (-0.005, 0.006)
            ],
            up = (1, 0, 0),
            resolution = 2
        );
        fcyl!(
            f,
            "spray_nozzle",
            0.008,
            0.02,
            (sx, sy + 0.215, sz + 0.055),
            M.pop_red,
            rot = (90, 0, 0),
            verts = 8
        );
    });
    let spray = hinged(join(&spray, "spray", None), f.at(sx, sy, sz + 0.035));
    let objs = settle!(centre = false);
    swing!(spray, "spritz", 35.0, seconds = 0.4, axis = Axis::X);
    objs
}

// --- the pixel-blaster arcade cabinet (1x1): an upright cabinet, lit marquee, tilted screen, stick and buttons ---
const STICK: (f64, f64, f64) = (-0.14, 1.04, 0.15); // the joystick's pivot

/// The cabinet's side, in (depth, height): the kick at the foot, the control deck jutting out, the screen's slope
/// back, the marquee's hood overhanging, the back straight down.
fn side() -> Vec<(f64, f64)> {
    vec![
        (WALL + 0.02, 0.0),
        (0.12, 0.0),
        (0.12, 0.84),
        (0.27, 0.96),
        (0.27, 1.05),
        (0.07, 1.08),
        (-0.02, 1.52),
        (0.13, 1.6),
        (0.13, 1.82),
        (WALL + 0.12, 1.88),
        (WALL + 0.02, 1.88),
    ]
}

pub fn arcade_cabinet() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        for x in [-0.33, 0.33] {
            fextrude!(
                f,
                format!("side{x}"),
                &smooth_outline(&side(), 1, true),
                0.04,
                (x, 0, 0),
                M.side,
                plane = Facing::Side,
                bevel = 0.008,
                bevel_segments = 1
            );
            // the side art: two bands following the deck's line
            fextrude!(
                f,
                format!("side_stripe{x}"),
                &[(-0.4, 0.42), (0.118, 0.42), (0.118, 0.6), (-0.4, 0.6)],
                0.044,
                (x, 0, 0),
                M.pop_orange,
                plane = Facing::Side
            );
            fextrude!(
                f,
                format!("side_stripe_b{x}"),
                &[(-0.4, 0.64), (0.118, 0.64), (0.118, 0.69), (-0.4, 0.69)],
                0.044,
                (x, 0, 0),
                M.pop_red,
                plane = Facing::Side
            );
        }
        fbox!(
            f,
            "lower_body",
            0.62,
            0.84,
            0.52,
            (0, 0.42, -0.15),
            M.cabinet,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "back",
            0.62,
            1.04,
            0.1,
            (0, 1.36, WALL + 0.07),
            M.cabinet,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "top",
            0.66,
            0.06,
            0.6,
            (0, 1.85, -0.13),
            M.side,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(
            f,
            "marquee",
            0.62,
            0.22,
            0.06,
            (0, 1.71, 0.1),
            M.marquee,
            rot = (-10, 0, 0),
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "marquee_logo",
            0.5,
            0.12,
            0.004,
            (0, 1.715, 0.133),
            M.pop_red,
            rot = (-10, 0, 0)
        );
        fbox!(
            f,
            "marquee_hood",
            0.62,
            0.02,
            0.2,
            (0, 1.82, 0.03),
            M.cabinet,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "bezel",
            0.62,
            0.48,
            0.03,
            (0, 1.29, 0.03),
            M.screen,
            rot = (-20, 0, 0)
        );
        soft!(
            f,
            "screen",
            0.48,
            0.38,
            0.01,
            (0, 1.29, 0.045),
            M.screen_lit,
            rot = (-20, 0, 0),
            round = 0.004,
            puff = 0.004,
            flat = 2,
            round_segments = 1
        );
        fbox!(
            f,
            "sprite_a",
            0.12,
            0.1,
            0.004,
            (-0.1, 1.31, 0.05),
            M.pop_yellow,
            rot = (-20, 0, 0)
        );
        fbox!(
            f,
            "sprite_b",
            0.08,
            0.06,
            0.004,
            (0.12, 1.21, 0.084),
            M.pop_red,
            rot = (-20, 0, 0)
        );
        soft!(
            f,
            "deck",
            0.62,
            0.08,
            0.3,
            (0, 0.995, 0.13),
            M.cabinet,
            rot = (12, 0, 0),
            round = 0.02,
            round_segments = 1
        );
        fbox!(
            f,
            "deck_art",
            0.58,
            0.004,
            0.24,
            (0, 1.037, 0.14),
            M.pop_orange,
            rot = (12, 0, 0)
        );
        for (i, (dx, dz)) in [
            (0.06, 0.12),
            (0.13, 0.11),
            (0.2, 0.12),
            (0.08, 0.2),
            (0.15, 0.19),
        ]
        .into_iter()
        .enumerate()
        {
            revolve!(
                format!("button{i}"),
                &[
                    (0.0, 0.0),
                    (0.026, 0.0),
                    (0.026, 0.006),
                    (0.019, 0.008),
                    (0.019, 0.016),
                    (0.014, 0.022),
                    (0.0, 0.023)
                ],
                at = f.at(dx, 1.04 + (0.15 - dz) * 0.2, dz),
                material = POP[i],
                segments = 12,
                rot = (-12, 0, 0)
            );
        }
        soft!(
            f,
            "coin_door",
            0.22,
            0.26,
            0.012,
            (0, 0.5, 0.126),
            M.tyre,
            round = 0.006,
            round_segments = 1
        );
        for (k, dx) in [-0.05, 0.05].into_iter().enumerate() {
            soft!(
                f,
                format!("coin_slot{k}"),
                0.032,
                0.062,
                0.008,
                (dx, 0.55, 0.134),
                M.pop_red,
                round = 0.004,
                round_segments = 1
            );
            fbox!(
                f,
                format!("coin_slit{k}"),
                0.004,
                0.03,
                0.004,
                (dx, 0.55, 0.139),
                M.dark
            );
        }
        fbox!(f, "kick", 0.6, 0.05, 0.54, (0, 0.025, -0.15), M.dark);
        _can(f, "can", 0.38, 0.2, 0.0, M.pop_green, false, 0.0);
        _wrapper(f, "wrapper", -0.3, 0.32, M.bag_yellow, 7, 0.0);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The joystick (shaft and ball), its origin on the pivot in the deck: `joystick` waggles it while played.
    let (sx, sy, sz) = STICK;
    let stick = new_parts(|| {
        revolve!(
            "stick_boot",
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.03, 0.004),
                (0.016, 0.014),
                (0.0, 0.016)
            ],
            at = f.at(sx, sy, sz),
            material = M.dark,
            segments = 12
        );
        fsweep!(
            f,
            "stick_shaft",
            &[(sx, sy, sz), (sx, sy + 0.075, sz)],
            0.007,
            M.chrome,
            verts = 8
        );
        fsphere!(
            f,
            "stick_ball",
            0.03,
            (sx, sy + 0.09, sz),
            M.pop_red,
            segments = 12,
            rings = 8
        );
    });
    let stick = hinged(join(&stick, "joystick", None), f.at(sx, sy, sz));
    let objs = settle!(centre = false);
    rock!(stick, "joystick", 18.0, seconds = 0.6, axis = Axis::Y);
    objs
}
