//! The rainbow hammock, the sunrise stretch lamp and the confetti party booth.

use super::common::{M, STRIPES, WALL2, bulb, pleats, slab, tumbler};
use super::music::tilt;
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, TAU, atan2, cosd, degrees, pow, sin, sind, sqrt};
use artkit::motion::{Rock, Squash, rock, squash};
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::py::fmod;
use artkit::scene::Obj;
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, kit, object_of, revolve, settle, slide, soft, torus,
};

// --- the rainbow hammock (3x1): a striped sling 2 m long between spreader bars, on a wooden stand (2026-10-21: 2x1,
// its sling 1.5 m, grown-ups lying in it with the knees up: M-packs-06, Q-07) ---------------------------------------
const SLING_END: f64 = 0.9; // the sling's ends' height
const SLING_LOW: f64 = 0.5; // its middle's
const SLING_W: f64 = 0.62; // its width
const SLING_X: f64 = 1.0; // its ends' x, at the spreader bars
const HOOK: f64 = 1.08; // the hooks' height: the sway's pivot
const POST_X: f64 = 1.38; // the posts' tops' x, the hooks on them
const PILLOW: f64 = 0.75; // the pillow's x (towards the left end): `furniture.json` `head`

/// The sling's top at a fraction `u` along it and `v` across it: sagging along, cupped across in the middle.
fn sling_y(u: f64, v: f64) -> f64 {
    let s = 2.0 * u - 1.0;
    SLING_LOW + (SLING_END - SLING_LOW) * s * s + 0.07 * pow(2.0 * v - 1.0, 2.0) * (1.0 - s * s)
}

pub fn rainbow_hammock() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let beam = rounded_rect(0.07, 0.09, 0.02, 1);
    let still = new_parts(|| {
        // the base beam, its ends rounded, and the two posts curving up and out from it to the hooks
        fsweep!(
            f,
            "beam",
            &[(-(POST_X - 0.1), 0.05, 0.0), (POST_X - 0.1, 0.05, 0.0)],
            0,
            M.wood,
            profile = &beam,
            up = (0, 0, 1)
        );
        for s in [-1.0, 1.0] {
            fsweep!(
                f,
                numbered("post"),
                &[
                    (s * (POST_X - 0.16), 0.04, 0.0),
                    (s * (POST_X - 0.1), 0.3, 0.0),
                    (s * (POST_X - 0.04), 0.75, 0.0),
                    (s * POST_X, HOOK + 0.05, 0.0)
                ],
                0,
                M.wood,
                profile = rounded_rect(0.07, 0.07, 0.02, 1),
                up = (0, 0, 1),
                resolution = 3
            );
            revolve!(
                numbered("post_top"),
                &[
                    (0.0, 0.0),
                    (0.04, 0.0),
                    (0.042, 0.02),
                    (0.03, 0.034),
                    (0.012, 0.042),
                    (0.0, 0.042)
                ],
                at = f.at(s * POST_X, HOOK + 0.05, 0),
                material = M.wood,
                segments = 12
            );
            fextrude!(
                f,
                numbered("foot"),
                &rounded_rect(0.62, 0.045, 0.02, 2),
                0.09,
                (s * (POST_X - 0.1), 0.0225, 0.0),
                M.wood,
                plane = Facing::Side,
                bevel = 0.008,
                bevel_segments = 1
            );
            for z in [-0.18, 0.18] {
                // braces from the foot to the post
                fsweep!(
                    f,
                    numbered("brace"),
                    &[
                        (s * (POST_X - 0.1), 0.04, z * 1.5),
                        (s * (POST_X - 0.11), 0.34, 0.0)
                    ],
                    0,
                    M.wood,
                    profile = rounded_rect(0.04, 0.03, 0.01, 1),
                    up = (0, 0, 1)
                );
            }
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The sling, its spreader bars and ropes, the hooks, the pillow and the throw: `sway` rocks them about the hooks.
    let sling = new_parts(|| {
        let (nu, nv, x0, x1, t) = (26_u32, 12_u32, -SLING_X, SLING_X, 0.018);
        let (mut verts, mut faces, mut mats): (Vec<P3>, Vec<Vec<u32>>, Vec<u16>) =
            (Vec::new(), Vec::new(), Vec::new());
        for layer in 0..2 {
            for i in 0..=nu {
                for j in 0..=nv {
                    let (u, v) = (i as f64 / nu as f64, j as f64 / nv as f64);
                    verts.push(f.at(
                        x0 + (x1 - x0) * u,
                        sling_y(u, v) - t * layer as f64,
                        -SLING_W / 2.0 + SLING_W * v,
                    ));
                }
            }
        }
        let at = |layer: u32, i: u32, j: u32| layer * (nu + 1) * (nv + 1) + i * (nv + 1) + j;
        let stripe = |j: u32| 5.min(j * 6 / nv) as u16;
        for i in 0..nu {
            for j in 0..nv {
                faces.push(vec![
                    at(0, i, j),
                    at(0, i + 1, j),
                    at(0, i + 1, j + 1),
                    at(0, i, j + 1),
                ]);
                faces.push(vec![
                    at(1, i, j),
                    at(1, i, j + 1),
                    at(1, i + 1, j + 1),
                    at(1, i + 1, j),
                ]);
                mats.extend([stripe(j); 2]);
            }
        }
        for i in 0..nu {
            // the long edges
            for j in [0, nv] {
                faces.push(vec![
                    at(0, i, j),
                    at(1, i, j),
                    at(1, i + 1, j),
                    at(0, i + 1, j),
                ]);
                mats.push(stripe(j.min(nv - 1)));
            }
        }
        for j in 0..nv {
            // the ends
            for i in [0, nu] {
                faces.push(vec![
                    at(0, i, j),
                    at(0, i, j + 1),
                    at(1, i, j + 1),
                    at(1, i, j),
                ]);
                mats.push(stripe(j));
            }
        }
        object_of!(
            "sling",
            &verts,
            &faces,
            &STRIPES,
            face_mats = &mats,
            sharp = 50
        );
        // a corded edge along each long side, and a fringe of tassels hanging from it
        for (j, side) in [(0, -1.0), (nv, 1.0)] {
            let edge: Vec<(f64, f64, f64)> = (0..=nu)
                .map(|i| {
                    (
                        x0 + (x1 - x0) * i as f64 / nu as f64,
                        sling_y(i as f64 / nu as f64, j as f64 / nv as f64) - t / 2.0,
                        side * SLING_W / 2.0,
                    )
                })
                .collect();
            fsweep!(f, numbered("edge_cord"), &edge, 0.008, M.rope, verts = 5);
            for i in (2..nu - 1).step_by(2) {
                let (x, y, z) = edge[i as usize];
                kit::tube!(
                    f,
                    numbered("tassel"),
                    &[(x, y, z + side * 0.004), (x, y - 0.09, z + side * 0.012)],
                    0.005,
                    STRIPES[((i / 2) % 6) as usize],
                    verts = 4,
                    round = 1,
                    scale = [0.7, 1.2]
                );
            }
        }
        for s in [-1.0, 1.0] {
            let bx = s * (SLING_X + 0.015);
            fsweep!(
                f,
                numbered("spreader"),
                &[
                    (bx, SLING_END - 0.01, -(SLING_W / 2.0 + 0.014)),
                    (bx, SLING_END - 0.01, SLING_W / 2.0 + 0.014)
                ],
                0.022,
                M.spreader,
                verts = 10
            );
            for z in [-1.0, 1.0] {
                fsphere!(
                    f,
                    numbered("spreader_end"),
                    0.026,
                    (bx, SLING_END - 0.01, z * (SLING_W / 2.0 + 0.014)),
                    M.spreader,
                    segments = 10,
                    rings = 6
                );
            }
            let hook = (s * (POST_X - 0.02), HOOK, 0.0);
            torus!(
                numbered("ring"),
                0.03,
                0.007,
                at = f.at(hook.0, hook.1, hook.2),
                material = M.lamp_metal,
                rot = (90, 0, 0),
                major_segments = 12,
                minor_segments = 4
            );
            kit::tube!(
                f,
                numbered("hook"),
                &kit::fillet!(
                    &[
                        (s * (POST_X - 0.02), HOOK + 0.05, 0.0),
                        (s * (POST_X - 0.02), HOOK + 0.032, 0.0),
                        (s * (POST_X - 0.045), HOOK + 0.03, 0.0),
                        (s * (POST_X - 0.048), HOOK + 0.05, 0.0),
                    ],
                    0.01
                ),
                0.005,
                M.lamp_metal,
                verts = 5,
                round = 1
            );
            for z in [-0.33, -0.22, -0.11, 0.0, 0.11, 0.22, 0.33] {
                kit::tube!(
                    f,
                    numbered("rope"),
                    &[
                        (s * (POST_X - 0.04), HOOK - 0.03, 0.0),
                        (bx, SLING_END + 0.012, z),
                    ],
                    0.005,
                    M.rope,
                    verts = 4,
                    round = 1
                );
            }
        }
        // the pillow at the left end, lying on the slope
        let u = (-PILLOW - x0) / (x1 - x0);
        let slope = degrees(libm::atan(
            (SLING_END - SLING_LOW) * 4.0 * (2.0 * u - 1.0) / (x1 - x0),
        ));
        soft!(
            f,
            "pillow",
            0.36,
            0.11,
            0.3,
            (-PILLOW, sling_y(u, 0.5) + 0.05, 0.0),
            M.pillow,
            rot = (0.0, slope, 0.0),
            round = 0.04,
            puff = 0.03,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
    });
    let sling = hinged(join(&sling, "sling", None), f.at(0, HOOK, 0));
    let objs = settle!(centre = false);
    rock(
        sling,
        "sway",
        5.0,
        Rock {
            seconds: 2.4,
            axis: Axis::X,
        },
    );
    objs
}

// --- the sunrise stretch lamp (1x1): a dawn scene in a round frame on a stand, a yoga mat before it -------------
const SUN_C: (f64, f64, f64) = (0.0, 1.25, -0.3); // the frame's middle (x, height, depth)
const SUN_R: f64 = 0.33; // the scene's radius
const SUN_RISE: f64 = 0.22;

/// The part of a disc of radius `r` (centre height `cy`) below a wavy horizon at `top` (offsets from the centre).
#[allow(clippy::too_many_arguments)]
fn hills(cy: f64, r: f64, top: f64, waves: f64, amp: f64, phase: f64, n: usize) -> Vec<(f64, f64)> {
    let rim = |x: f64| {
        let q = r * r - x * x;
        sqrt(if q > 0.0 { q } else { 0.0 })
    };
    let pts: Vec<(f64, f64)> = (0..=n)
        .map(|k| {
            // the horizon, left to right
            let x = -r + 2.0 * r * k as f64 / n as f64;
            let y = top + amp * sin(waves * PI * (x + r) / (2.0 * r) + phase);
            let c = rim(x);
            (x, if c < y { c } else { y })
        })
        .collect();
    let lo: Vec<(f64, f64)> = pts.iter().map(|&(x, _)| (x, -rim(x))).collect();
    let mut out = pts.clone();
    out.extend(lo.iter().rev().skip(1).take(lo.len() - 2)); // along the top, back along the circle's foot
    out.iter().map(|&(x, y)| (x, cy + y)).collect()
}

/// A spiral line on the end of a rolled mat at game x `x` (its end face), round (y, z): the layers of the roll.
#[allow(clippy::too_many_arguments)]
fn spiral(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r0: f64,
    r1: f64,
    turns: f64,
    material: Mat,
    side: f64,
) -> Obj {
    let n = artkit::py::int(turns * 14.0);
    let pts: Vec<(f64, f64, f64)> = (0..=n)
        .map(|k| {
            let fr = k as f64 / n as f64;
            let a = TAU * turns * fr;
            let r = r0 + (r1 - r0) * fr;
            (
                x + side * 0.002,
                y + r * sin(a),
                z + r * artkit::math::cos(a),
            )
        })
        .collect();
    fsweep!(f, name, &pts, 0.0025, material, verts = 4)
}

pub fn sunrise_lamp() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (cx, cy, cz) = SUN_C;
    let still = new_parts(|| {
        revolve!(
            "base",
            &[
                (0.0, 0.0),
                (0.13, 0.0),
                (0.13, 0.012),
                (0.122, 0.02),
                (0.1, 0.028),
                (0.04, 0.036),
                (0.03, 0.05),
                (0.0, 0.05)
            ],
            at = f.at(cx, 0, cz),
            material = M.housing,
            segments = 24
        );
        kit::tube!(
            f,
            "stem",
            &[(cx, 0.04, cz - 0.03), (cx, cy - 0.37, cz - 0.03)],
            0.016,
            M.lamp_metal,
            verts = 10,
            round = 1
        );
        revolve!(
            "stem_collar",
            &[
                (0.0, 0.0),
                (0.024, 0.0),
                (0.026, 0.01),
                (0.02, 0.03),
                (0.018, 0.05),
                (0.0, 0.05)
            ],
            at = f.at(cx, cy - 0.41, cz - 0.03),
            material = M.lamp_metal,
            segments = 10
        );
        fcyl!(
            f,
            "dimmer",
            0.018,
            0.014,
            (cx + 0.07, 0.04, cz + 0.06),
            M.lamp_metal,
            verts = 10
        );
        // the case: a shallow round drum, its edge rolled, the ring round the scene's face
        revolve!(
            "case",
            &[
                (0.0, -0.06),
                (0.37, -0.06),
                (0.384, -0.05),
                (0.39, -0.03),
                (0.386, -0.008),
                (0.37, 0.0),
                (0.0, 0.0)
            ],
            at = f.at(cx, cy, cz),
            material = M.housing,
            segments = 40,
            rot = (90, 0, 0)
        );
        torus!(
            "ring",
            0.355,
            0.02,
            at = f.at(cx, cy, cz + 0.006),
            material = M.lamp_metal,
            rot = (90, 0, 0),
            major_segments = 40,
            minor_segments = 6
        );
        fcyl!(
            f,
            "dawn",
            SUN_R + 0.01,
            0.006,
            (cx, cy, cz + 0.003),
            M.dawn,
            rot = (90, 0, 0),
            verts = 40
        );
        for (dx, dy, w) in [(-0.14, 0.17, 0.16), (0.12, 0.22, 0.12), (0.2, 0.08, 0.09)] {
            let cloud = smooth_outline(
                &[
                    (-w / 2.0, 0.0),
                    (w / 2.0, 0.0),
                    (w / 2.0, 0.012),
                    (w * 0.25, 0.03),
                    (0.0, 0.022),
                    (-w * 0.2, 0.036),
                    (-w / 2.0, 0.012),
                ],
                2,
                true,
            );
            let cloud: Vec<(f64, f64)> = cloud
                .iter()
                .map(|&(u, v)| (cx + dx + u, cy + dy + v))
                .collect();
            fextrude!(
                f,
                numbered("cloud"),
                &cloud,
                0.004,
                (0.0, 0.0, cz + 0.024),
                M.cloud
            );
        }
        slab(
            f,
            "hills_far",
            &hills(cy, SUN_R, -0.06, 3.0, 0.03, 0.4, 40),
            cz + 0.032,
            cz + 0.038,
            M.hills_far,
            0.0,
        );
        slab(
            f,
            "hills",
            &hills(cy, SUN_R, -0.12, 2.0, 0.035, 2.2, 40),
            cz + 0.038,
            cz + 0.046,
            M.hills,
            0.0,
        );
        // the yoga mat, its far end rolled up (the layers drawn on its ends), and a cork block on it
        fbox!(
            f,
            "mat",
            0.6,
            0.008,
            0.38,
            (0, 0.004, 0.13),
            M.yoga,
            bevel = 0.002,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "mat_roll",
            0.06,
            0.6,
            (0, 0.06, 0.38),
            M.yoga_roll,
            rot = (0, 90, 0),
            verts = 16
        );
        for s in [-1.0, 1.0] {
            spiral(
                f,
                &numbered("roll_layers"),
                s * 0.3,
                0.06,
                0.38,
                0.012,
                0.054,
                2.6,
                M.yoga,
                s,
            );
        }
        fbox!(
            f,
            "block",
            0.22,
            0.075,
            0.14,
            (-0.2, 0.0455, 0.06),
            M.cork,
            rot = (0, 0, 15),
            bevel = 0.008,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The sun, set behind the hills: `sunrise` raises it into the sky, which glows at night (`dawn`'s `glow`).
    let sun = new_parts(|| {
        fcyl!(
            f,
            "sun",
            0.13,
            0.004,
            (cx, cy - 0.19, cz + 0.011),
            M.sun,
            rot = (90, 0, 0),
            verts = 32
        );
        torus!(
            "sun_halo",
            0.11,
            0.004,
            at = f.at(cx, cy - 0.19, cz + 0.014),
            material = M.cloud,
            rot = (90, 0, 0),
            major_segments = 24,
            minor_segments = 3
        )
        .set_scale((1, 1, 0.4));
    });
    let sun = hinged(join(&sun, "sun", None), f.at(cx, cy - 0.19, cz + 0.011));
    let objs = settle!(centre = false);
    slide!(sun, "sunrise", (0.0, 0.0, SUN_RISE), seconds = 1.5);
    objs
}

// --- the confetti party booth (2x2): a horseshoe booth for two under a bulb-lit frame, a table with a tablet ----

/// A channelled back: `n` stuffed channels side by side from x0 to x1, each rising from the seat at `y0`
/// (depth `z0`) to `top`, leaning back to `z1` and rolling over at the top. The roll starts 9 cm down and thins as
/// it goes: turned in the last 3.5 cm, tighter than the channel is thick, its back folded inside out.
#[allow(clippy::too_many_arguments)]
fn channels(
    f: &GameFrame,
    name_: &str,
    x0: f64,
    x1: f64,
    n: usize,
    y0: f64,
    top: f64,
    z0: f64,
    z1: f64,
    lean: f64,
    mat: Mat,
) {
    let cw = (x1 - x0) / n as f64;
    for i in 0..n {
        let x = x0 + cw * (i as f64 + 0.5);
        fsweep!(
            f,
            numbered(name_),
            &[
                (x, y0, z0),
                (x, (y0 + top) / 2.0, (z0 + z1) / 2.0 - 0.01),
                (x, top - 0.09, z1 + 0.01),
                (x, top - 0.02, z1 - lean)
            ],
            0,
            mat,
            profile = rounded_rect(cw * 1.02, 0.17, 0.06, 1),
            up = (1, 0, 0),
            resolution = 2,
            scale = [1.0, 1.04, 0.95, 0.75]
        );
    }
}

pub fn party_booth() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (zr, xr, yr) = (0.82, 0.88, 1.58);
    let still = new_parts(|| {
        soft!(
            f,
            "seat_base",
            1.3,
            0.36,
            0.62,
            (0, 0.18, -0.05),
            M.booth,
            round = 0.05,
            flat = 1,
            round_segments = 1
        );
        soft!(
            f,
            "seat",
            1.3,
            0.088,
            0.58,
            (0, 0.394, -0.04),
            M.booth,
            round = 0.035,
            puff = 0.012,
            piping = 0.006,
            piping_material = M.piping,
            flat = 2
        );
        fbox!(f, "kick", 1.26, 0.04, 0.02, (0, 0.02, 0.27), M.panel);
        // the channelled backrest, its rolled top piped in yellow
        channels(
            f, "channel", -0.93, 0.93, 7, 0.38, 1.05, -0.3, -0.42, 0.06, M.booth,
        );
        fsweep!(
            f,
            "backrest_piping",
            &[(-0.93, 1.04, -0.475), (0.93, 1.04, -0.475)],
            0.012,
            M.piping,
            verts = 8
        );
        fextrude!(
            f,
            "back_panel",
            &rounded_rect(1.9, 1.6, 0.04, 2),
            0.06,
            (0.0, 0.8, WALL2 + 0.03),
            M.panel,
            bevel = 0.012,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            soft!(
                f,
                numbered("wing"),
                0.3,
                0.7,
                1.0,
                (s * 0.8, 0.37, 0.4),
                M.booth,
                round = 0.06,
                piping = 0.008,
                piping_material = M.piping,
                flat = 1,
                round_segments = 1
            );
            fbox!(
                f,
                numbered("wing_base"),
                0.28,
                0.04,
                0.96,
                (s * 0.8, 0.02, 0.4),
                M.panel,
                bevel = 0.01,
                bevel_segments = 1
            );
        }
        // the bulb-lit frame over it: each side one bent brass tube, from the wing's front up and back to the panel
        for s in [-1.0, 1.0] {
            kit::tube!(
                f,
                numbered("frame"),
                &kit::fillet!(
                    &[
                        (s * xr, 0.72, zr),
                        (s * xr, yr, zr),
                        (s * xr, yr, WALL2 + 0.06),
                    ],
                    0.06
                ),
                0.016,
                M.brass,
                verts = 8,
                round = 1
            );
            revolve!(
                numbered("frame_foot"),
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.03, 0.01),
                    (0.02, 0.02),
                    (0.0, 0.02)
                ],
                at = f.at(s * xr, 0.715, zr),
                material = M.brass,
                segments = 10
            );
        }
        fextrude!(
            f,
            "marquee",
            &rounded_rect(1.76, 0.12, 0.06, 4),
            0.05,
            (0.0, yr, zr),
            M.panel,
            bevel = 0.01,
            bevel_segments = 1
        );
        for i in 0..9 {
            bulb(f, -0.72 + i as f64 * 0.18, yr, zr + 0.03, i, 0.026);
        }
        // the marquee bulbs and confetti on the back panel
        for i in 0..13 {
            let x = -0.84 + i as f64 * 0.14;
            bulb(f, x, 1.53, WALL2 + 0.065, i, 0.03);
            bulb(
                f,
                x + 0.07,
                1.28 + 0.06 * sin(i as f64),
                WALL2 + 0.063,
                i + 3,
                0.024,
            );
        }
        let wire: Vec<(f64, f64, f64)> = (0..13)
            .map(|i| {
                (
                    -0.77 + 0.14 * i as f64,
                    1.28 + 0.06 * sin(i as f64) + 0.02,
                    WALL2 + 0.062,
                )
            })
            .collect();
        kit::tube!(f, "bulb_wire", &wire, 0.0025, M.stand, verts = 4, round = 2);
        kit::tube!(
            f,
            "bulb_wire_top",
            &[(-0.86, 1.555, WALL2 + 0.062), (0.86, 1.555, WALL2 + 0.062)],
            0.0025,
            M.stand,
            verts = 4,
            round = 1
        );
        for (i, (x, y)) in [
            (-0.6, 1.08),
            (-0.3, 1.18),
            (0.1, 1.1),
            (0.45, 1.15),
            (0.7, 1.05),
            (-0.05, 1.4),
            (0.3, 1.38),
            (-0.45, 1.4),
            (0.62, 1.42),
            (-0.75, 1.22),
        ]
        .into_iter()
        .enumerate()
        {
            let m = super::common::BULBS[i % 6];
            if i % 2 != 0 {
                fbox!(
                    f,
                    numbered("confetti"),
                    0.05,
                    0.05,
                    0.008,
                    (x, y, WALL2 + 0.064),
                    m,
                    rot = (0, 30 * i as i32, 0)
                );
            } else {
                fcyl!(
                    f,
                    numbered("confetti"),
                    0.026,
                    0.008,
                    (x, y, WALL2 + 0.064),
                    m,
                    rot = (90, 0, 0),
                    verts = 8
                );
            }
        }
        // the round table with the party tablet, two drinks and a striped bowl of popcorn
        let tz = 0.52;
        fcyl!(
            f,
            "table",
            0.31,
            0.035,
            (0.0, 0.72, tz),
            M.white,
            verts = 32,
            bevel = 0.01,
            bevel_segments = 1
        );
        revolve!(
            "table_stem",
            &[
                (0.2, 0.0),
                (0.2, 0.012),
                (0.18, 0.022),
                (0.06, 0.045),
                (0.035, 0.1),
                (0.035, 0.68),
                (0.06, 0.7),
                (0.0, 0.705)
            ],
            at = f.at(0, 0, tz),
            material = M.brass,
            segments = 20
        );
        fbox!(
            f,
            "tablet_foot",
            0.12,
            0.012,
            0.08,
            (0, 0.743, 0.62),
            M.dark,
            bevel = 0.004,
            bevel_segments = 1
        );
        let tc = (0.0, 0.86, 0.6);
        fextrude!(
            f,
            "tablet",
            &rounded_rect(0.34, 0.22, 0.02, 2),
            0.02,
            tc,
            M.dark,
            rot = (12, 0, 0),
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            f,
            "tablet_face",
            0.31,
            0.19,
            0.004,
            tilt(tc, -12.0, 0.0, -0.011),
            M.tablet_face,
            rot = (12, 0, 0)
        );
        tumbler(
            f,
            -0.18,
            0.4,
            M.drink_sun,
            Some(0.7375),
            0.11,
            0.034,
            None,
            None,
        );
        tumbler(
            f,
            0.18,
            0.4,
            M.drink_teal,
            Some(0.7375),
            0.11,
            0.034,
            Some(M.piping),
            None,
        );
        let bowl = revolve!(
            "popcorn_bowl",
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.09, 0.05),
                (0.085, 0.052),
                (0.045, 0.006),
                (0.0, 0.006)
            ],
            at = f.at(0.14, 0.7375, 0.66),
            material = M.white,
            segments = 16
        );
        kit::paint(bowl, M.bulb_c, |c| {
            fmod(atan2(c.y as f64, c.x as f64), PI / 4.0) < PI / 8.0
        });
        for k in 0..7 {
            let a = (51 * k) as f64;
            let r = if k % 3 != 0 { 0.04 } else { 0.015 };
            fsphere!(
                f,
                numbered("popcorn"),
                0.022,
                (
                    0.14 + r * cosd(a),
                    0.79 + 0.008 * (k % 2) as f64,
                    0.66 + r * sind(a)
                ),
                M.popcorn,
                segments = 6,
                rings = 4
            );
        }
        // the left curtain, gathered at the back
        pleats(
            f,
            "curtain_left",
            -xr,
            0.77,
            yr - 0.02,
            WALL2 + 0.07,
            WALL2 + 0.45,
            4,
            0.03,
            M.curtain,
            0.01,
            false,
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The right curtain, hanging closed along the rail: `curtain` draws it aside to the back.
    let cur = new_parts(|| {
        pleats(
            f,
            "curtain",
            xr,
            0.77,
            yr - 0.02,
            WALL2 + 0.07,
            zr - 0.04,
            9,
            0.022,
            M.curtain,
            0.01,
            false,
        );
    });
    let cur = hinged(join(&cur, "curtain", None), f.at(xr, yr, WALL2 + 0.07));
    let objs = settle!(centre = false);
    squash(
        cur,
        "curtain",
        (1, 0.25, 1),
        Squash {
            seconds: 0.8,
            ..Squash::default()
        },
    );
    objs
}
