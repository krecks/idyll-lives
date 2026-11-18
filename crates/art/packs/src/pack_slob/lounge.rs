//! The crumb-proof beanbag, the snack-stash nightstand and the command-couch gaming recliner, and the controller
//! lying by two of them.

use super::common::{_bake, _box, _can, _chip_bag, _m, _place, _wrapper, M};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, M4, P3, PI, TAU, V3, cos, pow, radians, sin};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{
    cylinder, fbox, fcyl, fsphere, fsweep, jitter, loft, revolve, slide, soft, sphere, squash,
    swing,
};

fn settle_in_place() -> Vec<Obj> {
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    )
}

// --- the crumb-proof beanbag (1x1): sat on and napped on in the middle, a snack bag and a controller by it -----

/// A game controller lying on `y`: a lofted body with its two grips, two sticks and a button.
#[allow(clippy::too_many_arguments)]
fn _controller(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, material: Mat, turn: f64) {
    let b = f.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let outline = [
        (0.07, 0.0),
        (0.06, 0.035),
        (0.035, 0.03),
        (0.0, 0.028),
        (-0.035, 0.03),
        (-0.06, 0.035),
        (-0.07, 0.0),
        (-0.065, -0.04),
        (-0.045, -0.05),
        (-0.025, -0.02),
        (0.025, -0.02),
        (0.045, -0.05),
        (0.065, -0.04),
    ];
    let sections: Vec<Vec<P3>> = [(0.0, 0.92), (0.008, 1.0), (0.026, 1.0), (0.034, 0.9)]
        .iter()
        .map(|&(hh, k)| {
            outline
                .iter()
                .map(|&(u, v)| P3::from((bx + u * k, by + v * k, bz + hh)))
                .collect()
        })
        .collect();
    let body = loft!(name, &sections, material = material);
    let mut made = vec![body];
    for dx in [-0.03, 0.03] {
        made.push(fcyl!(
            f,
            format!("{name}_stick{dx}"),
            0.011,
            0.012,
            (x + dx, y + 0.04, z),
            M.tyre,
            verts = 8
        ));
    }
    made.push(fcyl!(
        f,
        format!("{name}_button"),
        0.007,
        0.006,
        (x + 0.045, y + 0.036, z + 0.012),
        M.pop_red,
        verts = 8
    ));
    let m = M4::translation(V3::from((bx, by, 0.0)))
        * M4::rotation(radians(turn), Axis::Z)
        * M4::translation(V3::from((-bx, -by, 0.0)));
    for o in _bake(&made) {
        o.edit(|p| p.mesh.transform(&m));
    }
}

// (0.318: the chip bag's place as the script has it, not 1/pi)
#[allow(clippy::approx_constant)]
pub fn beanbag() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _chip_bag(
            f,
            "chip_bag",
            0.318,
            0.305,
            0.0,
            M.bag_yellow,
            30.0,
            80.0,
            0.16,
            0.22,
        );
        _controller(f, "controller", -0.34, 0.0, 0.33, M.dark, -25.0);
        _can(f, "can", -0.38, -0.32, 0.0, M.pop_blue, true, 40.0);
        _wrapper(f, "wrapper_a", 0.32, -0.36, M.bag_red, 3, 0.0);
        _wrapper(f, "wrapper_b", 0.4, 0.08, M.bag_green, 4, 0.0);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The bag: one lofted body, squat and round, rising into a back behind a soft seat (top 0.45), its vinyl
    // wrinkled, a piped seam round its foot and a carry loop: one part with its origin on the floor: `squash`
    // settles it under its sitter.
    let bag = new_parts(|| {
        let n = 28;
        let b = f.at(0, 0, 0);
        let (bx, by) = (b.x, b.y);
        // A ring `r` round at height `h`, its back half raised by up to `back`, its front lowered by `front`.
        let ring = |r: f64, h: f64, back: f64, front: f64| -> Vec<P3> {
            (0..n)
                .map(|i| {
                    let a = TAU * i as f64 / n as f64;
                    let (c, s) = (cos(a), sin(a)); // Blender: +y is the back
                    let up = if s > 0.0 { s } else { 0.0 };
                    let down = if -s > 0.0 { -s } else { 0.0 };
                    let lift = back * pow(up, 2.0) - front * pow(down, 2.0);
                    P3::from((bx + r * c, by + r * s, h + lift))
                })
                .collect()
        };
        // up the sides, over a rim (high at the back, low at the front) and down into the hollow the sitter sinks
        // into (content `seat`: 0.30 m, 5 cm forward of the middle)
        let rings = [
            ring(0.33, 0.0, 0.0, 0.0),
            ring(0.4, 0.04, 0.0, 0.0),
            ring(0.432, 0.13, 0.0, 0.0),
            ring(0.42, 0.25, 0.04, 0.0),
            ring(0.37, 0.34, 0.12, 0.02),
            ring(0.3, 0.41, 0.25, 0.04),
            ring(0.2, 0.43, 0.22, 0.05),
            ring(0.12, 0.36, 0.12, 0.02),
            ring(0.05, 0.315, 0.03, 0.0),
            ring(0.0, 0.31, 0.0, 0.0),
        ];
        let body = loft!("bag_body", &rings, material = M.bag);
        jitter!(body, 0.008, size = 0.16, seed = 2.0, up = 0.5);
        body.edit(|o| {
            // (its foot stays flat on the floor)
            for v in &mut o.mesh.verts {
                if v.z < 0.0 {
                    v.z = 0.0;
                }
            }
        });
        let seam: Vec<(f64, f64, f64)> = (0..n)
            .map(|i| {
                let a = TAU * i as f64 / n as f64;
                (0.405 * cos(a), 0.035, 0.405 * sin(a))
            })
            .collect();
        fsweep!(
            f,
            "bag_seam",
            &seam,
            0.01,
            M.bag_seam,
            verts = 5,
            closed = true
        );
        fsweep!(
            f,
            "bag_handle",
            &[
                (-0.06, 0.2, 0.41),
                (-0.05, 0.24, 0.432),
                (0.05, 0.24, 0.432),
                (0.06, 0.2, 0.41)
            ],
            0.0,
            M.bag_seam,
            profile = [
                (-0.004, -0.012),
                (0.004, -0.012),
                (0.004, 0.012),
                (-0.004, 0.012)
            ],
            up = (0, 0, 1),
            resolution = 2
        );
    });
    let bag = join(&bag, "bag", Some(P3::from((0, 0, 0))));
    let objs = settle_in_place();
    squash!(bag, "squash", (1.04, 1.04, 0.86), seconds = 0.5);
    objs
}

// --- the snack-stash nightstand (1x1): sat on (padded top at 0.45), its side drawer full of snacks -------------
pub fn snack_nightstand() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _box(
            f,
            "cabinet",
            0.56,
            0.38,
            0.46,
            (0, 0.19, -0.02),
            M.stand,
            10.0,
            24,
            0.01,
        );
        soft!(
            f,
            "pad",
            0.56,
            0.075,
            0.46,
            (0, 0.4125 - 0.006, -0.02),
            M.pad,
            round = 0.03,
            puff = 0.006,
            piping = 0.006,
            flat = 2
        );
        for k in [-1, 1] {
            // two buttons tufting the pad
            fsphere!(
                f,
                format!("pad_button{k}"),
                0.01,
                (k as f64 * 0.12, 0.445, -0.02),
                M.stand_front,
                scale = (1, 1, 0.5),
                segments = 6,
                rings = 3
            );
        }
        soft!(
            f,
            "lower_drawer",
            0.012,
            0.13,
            0.4,
            (0.284, 0.1, -0.02),
            M.stand_front,
            round = 0.004,
            round_segments = 1
        );
        fsweep!(
            f,
            "lower_pull",
            &[
                (0.29, 0.1, -0.06),
                (0.305, 0.1, -0.05),
                (0.305, 0.1, 0.01),
                (0.29, 0.1, 0.02)
            ],
            0.006,
            M.chrome,
            verts = 6,
            resolution = 2
        );
        fbox!(
            f,
            "drawer_slot",
            0.004,
            0.13,
            0.4,
            (0.279, 0.27, -0.02),
            M.dark
        );
        kit::shaker!(
            f,
            "front_panel",
            0.48,
            0.3,
            (0, 0.2, 0.212),
            0.012,
            M.stand_front,
            rail = 0.05,
            panel = M.stand
        );
        fbox!(
            f,
            "magazine",
            0.2,
            0.012,
            0.26,
            (-0.1, 0.006, 0.12),
            M.pop_purple,
            rot = (0, 0, 12)
        ); // under its front edge
        for fx in [-0.25, 0.25] {
            for fz in [-0.22, 0.18] {
                fcyl!(
                    f,
                    format!("foot{fx}{fz}"),
                    0.018,
                    0.012,
                    (fx, 0.006, fz),
                    M.dark,
                    verts = 8
                );
            }
        }
        // floor clutter: cans, wrappers, a knocked-over alarm clock with its bells, a sock
        _can(f, "can_a", -0.36, 0.26, 0.0, M.pop_blue, true, 70.0);
        _can(f, "can_b", -0.38, -0.3, 0.0, M.pop_red, false, 0.0);
        _can(f, "can_c", 0.4, 0.3, 0.0, M.pop_green, true, -20.0);
        _wrapper(f, "wrapper_a", -0.2, 0.36, M.bag_yellow, 5, 0.0);
        _wrapper(f, "wrapper_b", 0.2, 0.34, M.bag_purple, 6, 0.0);
        let mut clock = vec![
            revolve!(
                "clock",
                &[
                    (0.0, -0.03),
                    (0.06, -0.03),
                    (0.062, -0.022),
                    (0.062, 0.022),
                    (0.056, 0.03),
                    (0.0, 0.03)
                ],
                material = M.gloss_white,
                segments = 16
            ),
            cylinder!(
                "clock_face",
                0.05,
                0.004,
                at = (0, 0, 0.031),
                material = M.display,
                verts = 16
            ),
        ];
        for s in [-1, 1] {
            clock.push(sphere!(
                format!("clock_bell{s}"),
                0.025,
                at = (s as f64 * 0.04, 0.0, 0.062),
                material = M.chrome,
                scale = (1, 1, 0.7),
                segments = 10,
                rings = 5
            ));
        }
        _place(&clock, f.at(-0.34, 0.062, -0.06), -30.0, 90.0, 0.0);
        fsweep!(
            f,
            "sock",
            &[
                (0.374, 0.012, -0.32),
                (0.414, 0.012, -0.22),
                (0.424, 0.014, -0.15),
                (0.434, 0.012, -0.12)
            ],
            0.0,
            M.white,
            profile = rounded_rect(0.02, 0.07, 0.01, 2),
            up = (0, 0, 1),
            resolution = 2
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The top side drawer with its snacks, closed into the right side: `drawer_open` slides it out to the right.
    let drawer = new_parts(|| {
        fbox!(
            f,
            "drawer_box",
            0.24,
            0.1,
            0.38,
            (0.16, 0.27, -0.02),
            M.white,
            bevel = 0.004,
            bevel_segments = 1
        );
        soft!(
            f,
            "drawer_front",
            0.012,
            0.13,
            0.42,
            (0.286, 0.27, -0.02),
            M.stand_front,
            round = 0.005,
            round_segments = 1
        );
        fsweep!(
            f,
            "drawer_pull",
            &[
                (0.292, 0.29, -0.07),
                (0.31, 0.29, -0.06),
                (0.31, 0.29, 0.02),
                (0.292, 0.29, 0.03)
            ],
            0.006,
            M.chrome,
            verts = 6,
            resolution = 2
        );
        for (i, (dz, m)) in [(-0.12, M.bag_red), (0.0, M.bag_orange), (0.12, M.bag_green)]
            .into_iter()
            .enumerate()
        {
            _chip_bag(
                f,
                &format!("snack{i}"),
                0.15,
                -0.02 + dz,
                0.26,
                m,
                90.0,
                (-15 + i as i32 * 12) as f64,
                0.16,
                0.14,
            );
        }
        revolve!(
            "cookie_a",
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.046, 0.012),
                (0.04, 0.02),
                (0.0, 0.021)
            ],
            at = f.at(0.17, 0.36, 0.12),
            material = M.cookie,
            segments = 12
        );
        soft!(
            f,
            "candy_bar",
            0.04,
            0.02,
            0.14,
            (0.06, 0.335, -0.08),
            M.pop_purple,
            round = 0.006,
            round_segments = 1
        );
    });
    let drawer = hinged(join(&drawer, "drawer", None), f.at(0.286, 0.27, -0.02));
    let objs = settle_in_place();
    slide!(drawer, "drawer_open", (-0.16, 0, 0), seconds = 0.6);
    objs
}

// --- the command-couch gaming recliner (2x1): one seat in the middle, a mini-fridge table and a console dock ----
const HINGE: (f64, f64) = (0.42, -0.3); // the back's hinge (height, depth)
const CUP: (f64, f64, f64) = (0.37, 0.6, 0.12); // the cup holder's pivot (x, height, depth), at the armrest's front

pub fn gaming_recliner() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        soft!(
            f,
            "base",
            0.82,
            0.3,
            0.66,
            (0, 0.17, -0.06),
            M.seat,
            round = 0.05,
            piping = 0.006
        );
        soft!(
            f,
            "seat",
            0.6,
            0.12,
            0.58,
            (0, 0.38, -0.02),
            M.stripe,
            round = 0.045,
            puff = 0.012,
            flat = 2
        );
        for k in [-0.1, 0.1] {
            // the seat's stitched channels
            fbox!(
                f,
                format!("seat_channel{k}"),
                0.008,
                0.004,
                0.5,
                (k, 0.452, -0.02),
                M.seat
            );
        }
        soft!(
            f,
            "footrest",
            0.5,
            0.1,
            0.24,
            (0, 0.34, 0.32),
            M.seat,
            rot = (10, 0, 0),
            round = 0.04,
            puff = 0.01,
            flat = 2
        );
        for s in [-1, 1] {
            let sf = s as f64;
            soft!(
                f,
                format!("arm{s}"),
                0.14,
                0.32,
                0.66,
                (sf * 0.36, 0.45, -0.06),
                M.seat,
                round = 0.05,
                puff = (0, 0.01, 0),
                flat = 2,
                piping = 0.005
            );
            fbox!(
                f,
                format!("led{s}"),
                0.008,
                0.02,
                0.6,
                (sf * 0.432, 0.32, -0.06),
                M.cyan,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(f, "led_front", 0.8, 0.02, 0.008, (0, 0.04, 0.27), M.cyan);
        // the right: a mini-fridge side table, cans behind its tinted door, a chip bag on top
        let fx = 0.7;
        _box(
            f,
            "fridge",
            0.38,
            0.56,
            0.42,
            (fx, 0.28, -0.12),
            M.gloss_white,
            10.0,
            24,
            0.02,
        );
        soft!(
            f,
            "fridge_door",
            0.34,
            0.5,
            0.012,
            (fx, 0.28, 0.096),
            M.window,
            round = 0.01,
            round_segments = 1
        );
        fsweep!(
            f,
            "fridge_handle",
            &[
                (fx + 0.15, 0.48, 0.1),
                (fx + 0.15, 0.47, 0.112),
                (fx + 0.15, 0.32, 0.112),
                (fx + 0.15, 0.31, 0.1)
            ],
            0.006,
            M.chrome,
            verts = 6,
            resolution = 2
        );
        fbox!(f, "fridge_led", 0.36, 0.02, 0.008, (fx, 0.05, 0.1), M.cyan);
        for (i, m) in ["pop_red", "pop_blue", "pop_green"].into_iter().enumerate() {
            _can(
                f,
                &format!("fridge_can{i}"),
                fx - 0.1 + i as f64 * 0.1,
                0.06,
                0.33,
                _m(m),
                false,
                0.0,
            );
        }
        _chip_bag(
            f,
            "chip_bag",
            fx,
            -0.12,
            0.56,
            M.bag_yellow,
            20.0,
            -60.0,
            0.16,
            0.22,
        );
        // the left: a console dock, a headset on its stand and a controller
        let lx = -0.7;
        _box(
            f,
            "dock",
            0.36,
            0.4,
            0.36,
            (lx, 0.2, -0.14),
            M.seat,
            10.0,
            24,
            0.02,
        );
        _box(
            f,
            "console",
            0.3,
            0.07,
            0.24,
            (lx, 0.435, -0.16),
            M.gloss_white,
            6.0,
            24,
            0.01,
        );
        for k in 0..4 {
            fbox!(
                f,
                format!("console_vent{k}"),
                0.2,
                0.003,
                0.008,
                (lx, 0.471, -0.22 + k as f64 * 0.03),
                M.dark
            );
        }
        fbox!(
            f,
            "console_led",
            0.08,
            0.008,
            0.002,
            (lx + 0.05, 0.44, -0.039),
            M.cyan
        );
        fsweep!(
            f,
            "headset_stand",
            &[(lx, 0.47, -0.24), (lx, 0.83, -0.24)],
            0.012,
            M.chrome,
            verts = 8
        );
        revolve!(
            "headset_foot",
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.05, 0.006),
                (0.02, 0.012),
                (0.0, 0.012)
            ],
            at = f.at(lx, 0.47, -0.24),
            material = M.chrome,
            segments = 12
        );
        let band: Vec<(f64, f64, f64)> = (0..9)
            .map(|k| {
                (
                    lx - 0.1 + 0.2 * k as f64 / 8.0,
                    0.76 + 0.09 * sin(PI * k as f64 / 8.0),
                    -0.24,
                )
            })
            .collect();
        fsweep!(
            f,
            "headset_band",
            &band,
            0.0,
            M.stripe,
            profile = rounded_rect(0.012, 0.035, 0.005, 1),
            up = (0, 0, 1)
        );
        for dx in [-0.1, 0.1] {
            revolve!(
                format!("ear_cup{dx}"),
                &[
                    (0.0, -0.025),
                    (0.042, -0.025),
                    (0.048, -0.01),
                    (0.048, 0.015),
                    (0.04, 0.025),
                    (0.0, 0.025)
                ],
                at = f.at(lx + dx, 0.74, -0.24),
                material = M.seat,
                segments = 14,
                rot = (0, 90, 0)
            );
        }
        _controller(f, "controller", lx + 0.08, 0.437, 0.0, M.gloss_white, 15.0);
        _wrapper(f, "wrapper_a", 0.3, 0.38, M.bag_orange, 8, 0.0);
        _wrapper(f, "wrapper_b", -0.45, 0.32, M.bag_purple, 9, 0.0);
        _can(f, "can", -0.3, 0.38, 0.0, M.pop_blue, true, 15.0);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The tall back with its stripe and headrest pillow, hinged at the seat's back: `recline` lays it back 20°.
    let (hy, hz) = HINGE;
    let back = new_parts(|| {
        soft!(
            f,
            "back",
            0.66,
            0.78,
            0.16,
            (0, 0.72, -0.265),
            M.seat,
            rot = (-14, 0, 0),
            round = 0.06,
            puff = 0.015,
            flat = 2
        );
        soft!(
            f,
            "back_stripe",
            0.42,
            0.62,
            0.06,
            (0, 0.74, -0.18),
            M.stripe,
            rot = (-14, 0, 0),
            round = 0.025,
            puff = 0.012,
            flat = 2
        );
        soft!(
            f,
            "headrest",
            0.3,
            0.14,
            0.1,
            (0, 1.02, -0.24),
            M.gloss_white,
            rot = (-14, 0, 0),
            round = 0.04,
            puff = 0.02,
            flat = 2
        );
    });
    let back = hinged(join(&back, "back", None), f.at(0, hy, hz));
    // The cup holder with a soda in it, tucked under the right armrest's front: `cup_out` swings it out.
    let (cx, cy, cz) = CUP;
    let cup = new_parts(|| {
        revolve!(
            "cup_ring",
            &[(0.034, -0.01), (0.046, -0.01), (0.048, 0.01), (0.036, 0.01)],
            at = f.at(cx + 0.02, cy, cz - 0.07),
            material = M.dark,
            segments = 14
        );
        fbox!(
            f,
            "cup_arm",
            0.03,
            0.015,
            0.08,
            (cx, cy - 0.005, cz - 0.03),
            M.dark,
            bevel = 0.004,
            bevel_segments = 1
        );
        _can(
            f,
            "soda",
            cx + 0.02,
            cz - 0.07,
            cy - 0.01,
            M.pop_red,
            false,
            0.0,
        );
    });
    let cup = hinged(join(&cup, "cup_holder", None), f.at(cx, cy, cz));
    let objs = settle_in_place();
    swing!(back, "recline", -20.0, seconds = 1.0, axis = Axis::X);
    swing!(cup, "cup_out", 110.0, seconds = 0.6);
    objs
}
