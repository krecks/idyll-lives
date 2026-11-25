//! OU2: the sun lounger, the hammock, the porch swing, the swing set and the cedar hot tub; each moves while it
//! is used.

use super::common::{F, M, end_round, mesh_of, plank, side_slab, tapered_leg};
use super::seats::arm_board;
use artkit::forms::{bisect, rounded_rect};
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, TAU, V3, cos, cosd, pow, radians, sin, sind};
use artkit::motion::{PathKey, Rock, Spin, Swing, attach, last_frame, path, rock, spin, swing};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::N;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, revolve, ring, settle, soft, torus};

// --- OU2: the sun lounger: a slatted platform and cushion; the back reclines flat for sunbathing ---------------
const BACK_HINGE: (f64, f64) = (0.29, -0.36); // the back's hinge, (height, depth)

/// A side outline (depth, height) from z0 to z1, y0 to y1, its two ends rounded by `r`.
fn rounded_side(z0: f64, z1: f64, y0: f64, y1: f64, r: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for (cz, a0) in [(z1 - r, -90), (z0 + r, 90)] {
        for i in 0..5 {
            let a = radians(a0 as f64 + (180 * i) as f64 / 4.0);
            out.push((cz + r * cos(a), (y0 + y1) / 2.0 + (y1 - y0) / 2.0 * sin(a)));
        }
    }
    out
}

pub fn sun_lounger() -> Vec<Obj> {
    let still = new_parts(|| {
        for s in [-1, 1] {
            let sf = s as f64;
            // the side rails, rounded at their ends; tapered legs with a cross rail between each pair
            side_slab(
                &format!("rail{s}"),
                sf * 0.29,
                &rounded_side(-0.38, 0.94, 0.22, 0.29, 0.03),
                0.04,
                M.lounger,
                0.005,
            );
            for z in [-0.3, 0.86] {
                tapered_leg(
                    &format!("leg{s}{}", float_repr(z)),
                    sf * 0.275,
                    z,
                    0.0,
                    0.235,
                    0.04,
                    0.05,
                    M.lounger,
                    12,
                );
            }
        }
        for z in [-0.3, 0.86] {
            fbox!(
                F,
                format!("leg_rail{}", float_repr(z)),
                0.5,
                0.04,
                0.03,
                (0, 0.1, z),
                M.lounger,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        for k in 0..9 {
            fbox!(
                F,
                format!("slat{k}"),
                0.54,
                0.02,
                0.11,
                (0, 0.27, -0.3 + 0.145 * k as f64),
                M.lounger,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        // the cushions: the long one and the seat's end under the back, piped, their middles at the old tops (0.35)
        soft!(
            F,
            "cushion",
            0.56,
            0.052,
            0.95,
            (0, 0.32, 0.445),
            M.lounger_pad,
            round = 0.025,
            puff = 0.008,
            flat = 2,
            piping = 0.004,
            round_segments = 1
        );
        soft!(
            F,
            "seat_end",
            0.56,
            0.052,
            0.32,
            (0, 0.32, -0.198),
            M.lounger_pad,
            round = 0.025,
            puff = 0.008,
            flat = 1,
            piping = 0.004,
            round_segments = 1
        );
        // a beach towel laid along the cushion and over its foot, and one rolled up
        let drape = kit::fillet!(
            &[(0.0, 0.355, 0.55), (0.0, 0.356, 0.915), (0.0, 0.24, 0.93)],
            0.02,
            steps = 2
        );
        fsweep!(
            F,
            "towel",
            &drape,
            0,
            M.towel,
            profile = rounded_rect(0.4, 0.008, 0.003, 1),
            up = (1, 0, 0)
        );
        let roll: Vec<(f64, f64, f64)> = [-0.2, 0.2].iter().map(|&x| (x, 0.375, 0.5)).collect();
        let prof: Vec<(f64, f64)> = (0..10)
            .map(|i| {
                (
                    0.02 * cos(TAU * i as f64 / 10.0) * (1.0 + 0.08 * (i % 2) as f64),
                    0.02 * sin(TAU * i as f64 / 10.0),
                )
            })
            .collect();
        fsweep!(
            F,
            "towel_roll",
            &roll,
            0,
            M.towel,
            profile = prof,
            up = (0, 0, 1)
        );
    });
    join(&still, "lounger", Some(P3::from((0, 0, 0))));
    let (y0, z0) = BACK_HINGE;
    let back = new_parts(|| {
        let a = 35;
        let along = (0, sind(a as f64), -cosd(a as f64)); // up the back
        let face = (0, cosd(a as f64), sind(a as f64));
        let c = (
            0,
            y0 + 0.345 * along.1 + 0.012 * face.1,
            z0 + 0.345 * along.2 + 0.012 * face.2,
        );
        soft!(
            F,
            "back_pad",
            0.56,
            0.68,
            0.07,
            c,
            M.lounger_pad,
            rot = (-(90 - a), 0, 0),
            round = 0.028,
            puff = 0.01,
            flat = 2,
            piping = 0.004,
            round_segments = 1,
            pinch = 0.0
        );
        let mid = (
            0,
            y0 + 0.585 * along.1 + 0.07 * face.1,
            z0 + 0.585 * along.2 + 0.07 * face.2,
        );
        // (the pillow's top edge is the lounger's highest point: neither pinched nor drawn in, nor is the pad)
        soft!(
            F,
            "pillow",
            0.42,
            0.18,
            0.08,
            mid,
            M.pillow,
            rot = (-(90 - a), 0, 0),
            round = 0.035,
            puff = 0.02,
            pinch = 0.0
        );
    });
    let back = hinged(join(&back, "back", None), F.at(0, y0, z0));
    settle!(centre = false);
    swing(
        back,
        "recline",
        -30.0,
        Swing {
            seconds: 1.0,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    meshes()
}

// --- the hammock: a striped sling between spreader bars on a wooden stand; it sways while lain in -------------
const HOOK_Y: f64 = 1.1; // the hooks, height: the sling sways about the line through them

/// A sagging sling `width` wide from depth -half to +half, `low` in the middle and `high` at the ends, `t` thick,
/// in bands of alternating materials; its sides curl up by `curl` where it sags (flat at the spreader bars), so
/// the middle line keeps its heights.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn sling(
    name: &str,
    width: f64,
    half: f64,
    low: f64,
    high: f64,
    t: f64,
    materials: &[Mat],
    bands: usize,
    cols: usize,
    curl: f64,
) -> Obj {
    let n = bands * 3;
    let xs: Vec<f64> = (0..=cols)
        .map(|i| -width / 2.0 + width * i as f64 / cols as f64)
        .collect();
    let (mut verts, mut faces, mut mats) = (Vec::new(), Vec::new(), Vec::new());
    let y_at = |x: f64, z: f64| {
        low + (high - low) * pow(z / half, 2.0)
            + curl * pow(2.0 * x / width, 2.0) * (1.0 - pow(z / half, 2.0))
    };
    for i in 0..=n {
        let z = -half + 2.0 * half * i as f64 / n as f64;
        for &x in &xs {
            let y = y_at(x, z);
            verts.push(V3::from(F.at(x, y, z)));
            verts.push(V3::from(F.at(x, y - t, z)));
        }
    }
    let row = 2 * (cols + 1) as u32;
    for i in 0..n as u32 {
        let m = ((i / 3) as usize % materials.len()) as u16;
        let (a, b) = (i * row, (i + 1) * row);
        for c in 0..cols as u32 {
            let (p, q) = (2 * c, 2 * (c + 1));
            for f in [
                vec![a + p, b + p, b + q, a + q],
                vec![a + p + 1, a + q + 1, b + q + 1, b + p + 1],
            ] {
                faces.push(f);
                mats.push(m);
            }
        }
        for p in [0, 2 * cols as u32] {
            // the two side edges
            faces.push(vec![a + p, a + p + 1, b + p + 1, b + p]);
            mats.push(m);
        }
    }
    for a in [0, n as u32 * row] {
        // the two ends
        for c in 0..cols as u32 {
            let (p, q) = (2 * c, 2 * (c + 1));
            faces.push(vec![a + p, a + q, a + q + 1, a + p + 1]);
            mats.push(0);
        }
    }
    mesh_of(name, verts, &faces, materials, &mats, true)
}

// A stand hammock on its 1 x 3 tiles: a 2.8 m stand, a 1.96 m bed between the spreader bars, long enough to lie in
// flat (`beds.hammock` `sling`: `low`, `end`, `half`, and the pillow at `head`).
const SLING_LOW: f64 = 0.52;
const SLING_END: f64 = 0.84;
const SLING_HALF: f64 = 0.98;
const SLING_HEAD: f64 = 0.78;
const HAMMOCK_HOOK: f64 = 1.245; // the hooks on the posts' inner faces, either side of the middle

pub fn hammock() -> Vec<Obj> {
    let stand = new_parts(|| {
        side_slab(
            "base",
            0,
            &[(-1.4, 0.0), (1.4, 0.0), (1.37, 0.06), (-1.37, 0.06)],
            0.1,
            M.hammock_frame,
            0.006,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            // the feet across, rounded at their ends; the posts leaning in, cut level at the foot; braces
            plank(
                &format!("foot{s}"),
                (-0.5, 0.02, sf * 1.36),
                (0.5, 0.02, sf * 1.36),
                0.04,
                M.hammock_frame,
                &end_round(0.08, 1.0, Some(0.03), 2, true),
                (0, 1, 0),
                0.006,
            );
            let hw = 0.04 / cosd(2.9);
            side_slab(
                &format!("post{s}"),
                0,
                &[
                    (sf * 1.36 - hw, 0.0),
                    (sf * 1.36 + hw, 0.0),
                    (sf * 1.3 + hw, 1.2),
                    (sf * 1.3 - hw, 1.2),
                ],
                0.08,
                M.hammock_frame,
                0.006,
            );
            kit::board(
                &F,
                &format!("brace{s}"),
                (0, 0.06, sf * 1.0),
                (0, 0.5, sf * 1.34),
                0.05,
                0.05,
                M.hammock_frame,
                (1, 0, 0),
                0.008,
            );
            fcyl!(
                F,
                format!("cap{s}"),
                0.045,
                0.02,
                (0, 1.19, sf * 1.3),
                M.hammock_frame,
                verts = 10,
                bevel = 0.006,
                bevel_segments = 1
            );
            // the hook: an eye screwed into the post and a ring hanging from it
            torus!(
                format!("hook{s}"),
                0.018,
                0.005,
                at = F.at(0, HOOK_Y, sf * HAMMOCK_HOOK),
                material = M.hammock_frame,
                rot = (0, 90, 0),
                major_segments = 10,
                minor_segments = 4
            );
        }
    });
    join(&stand, "stand", Some(P3::from((0, 0, 0))));
    let bed = new_parts(|| {
        sling(
            "sling",
            0.74,
            SLING_HALF,
            SLING_LOW,
            SLING_END,
            0.02,
            &[M.sling, M.sling_stripe],
            10,
            4,
            0.05,
        );
        let bar = SLING_HALF + 0.01;
        for s in [-1, 1] {
            let sf = s as f64;
            revolve!(
                format!("spreader{s}"),
                &[
                    (0.0, -0.4),
                    (0.012, -0.4),
                    (0.016, -0.39),
                    (0.016, 0.39),
                    (0.012, 0.4),
                    (0.0, 0.4)
                ],
                at = F.at(0, SLING_END + 0.005, sf * bar),
                material = M.hammock_frame,
                segments = 8,
                rot = (0, 90, 0)
            );
            // the cords fanning from the bar to a ring under the hook, and the rope up to it
            let knot = (N::I(0), HOOK_Y - 0.09, sf * (HAMMOCK_HOOK - 0.06));
            for x in [-0.36, -0.18, 0.0, 0.18, 0.36] {
                fsweep!(
                    F,
                    format!("cord{s}{}", float_repr(x)),
                    &[(N::F(x), SLING_END + 0.01, sf * bar), knot],
                    0.0035,
                    M.sling_stripe,
                    verts = 4
                );
            }
            torus!(
                format!("ring{s}"),
                0.02,
                0.006,
                at = F.at(knot.0, knot.1, knot.2),
                material = M.sling_stripe,
                rot = (90, 0, 0),
                major_segments = 10,
                minor_segments = 4
            );
            fsweep!(
                F,
                format!("rope{s}"),
                &[knot, (N::I(0), HOOK_Y, sf * HAMMOCK_HOOK)],
                0.006,
                M.sling_stripe,
                verts = 5
            );
            // the sling's edges whipped round the bar
            let hem: Vec<(f64, f64, f64)> = [-0.37, 0.37]
                .iter()
                .map(|&x| (x, SLING_END - 0.002, sf * (bar - 0.012)))
                .collect();
            fsweep!(F, format!("hem{s}"), &hem, 0.012, M.sling, verts = 6);
        }
        let head = SLING_LOW + (SLING_END - SLING_LOW) * pow(SLING_HEAD / SLING_HALF, 2.0);
        soft!(
            F,
            "pillow",
            0.4,
            0.08,
            0.26,
            (0, head + 0.03, -SLING_HEAD),
            M.pillow,
            rot = (12, 0, 0),
            round = 0.035,
            puff = 0.02,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
    });
    let bed = hinged(join(&bed, "sling", None), F.at(0, HOOK_Y, 0));
    settle!(centre = false);
    rock(
        bed,
        "sway",
        8.0,
        Rock {
            seconds: 2.0,
            axis: Axis::Y,
        },
    );
    meshes()
}

// --- the porch swing: a slatted swing seat hung on chains from an A-frame; it swings while sat in --------------

/// Two A-frame ends at x ±0.92, their legs `spread` m out at the foot, and the beam across them: square legs cut
/// level at the foot and meeting under the beam, or round timbers tapering up; ties across; a beam with its ends
/// cut back underneath.
fn a_frame(prefix: &str, beam_y: f64, material: Mat, round_legs: bool, spread: f64) {
    let tie = spread * (1.0 - 0.7 / beam_y);
    for s in [-1, 1] {
        let sf = s as f64;
        let x = sf * 0.92;
        for e in [-1, 1] {
            let ef = e as f64;
            if round_legs {
                fsweep!(
                    F,
                    format!("{prefix}leg{s}{e}"),
                    &[
                        (x, 0.0, ef * spread),
                        (x, beam_y * 0.5, ef * spread * 0.5),
                        (x, beam_y - 0.02, ef * 0.02)
                    ],
                    0.04,
                    material,
                    verts = 10,
                    scale = [1.0, 0.93, 0.86]
                );
            } else {
                let hw = 0.04 * artkit::py::hypot(spread, beam_y) / beam_y;
                let outline = if e > 0 {
                    [
                        (ef * spread - hw, 0.0),
                        (ef * spread + hw, 0.0),
                        (ef * hw * 0.2, beam_y),
                        (-ef * hw * 1.8, beam_y),
                    ]
                } else {
                    [
                        (ef * spread - hw, 0.0),
                        (ef * spread + hw, 0.0),
                        (hw * 1.8, beam_y),
                        (-hw * 0.2, beam_y),
                    ]
                };
                side_slab(
                    &format!("{prefix}leg{s}{e}"),
                    x,
                    &outline,
                    0.08,
                    material,
                    0.006,
                );
            }
        }
        kit::board(
            &F,
            &format!("{prefix}tie{s}"),
            (x, 0.7, -tie - 0.03),
            (x, 0.7, tie + 0.03),
            0.06,
            0.04,
            material,
            (0, 1, 0),
            0.008,
        );
        if !round_legs {
            // a gusset plate where the legs meet the beam
            side_slab(
                &format!("{prefix}gusset{s}"),
                x - sf * 0.046,
                &[
                    (-0.14, beam_y - 0.05),
                    (0.14, beam_y - 0.05),
                    (0.06, beam_y - 0.2),
                    (-0.06, beam_y - 0.2),
                ],
                0.012,
                material,
                0.003,
            );
        }
    }
    if round_legs {
        fsweep!(
            F,
            format!("{prefix}beam"),
            &[(-0.96, beam_y, 0), (0.96, beam_y, 0)],
            0.045,
            material,
            verts = 14
        );
        for s in [-1, 1] {
            fcyl!(
                F,
                format!("{prefix}beam_end{s}"),
                0.047,
                0.012,
                (s as f64 * 0.955, beam_y, 0),
                material,
                rot = (0, 90, 0),
                verts = 14
            );
        }
    } else {
        fextrude!(
            F,
            format!("{prefix}beam"),
            &[
                (-0.96, 0.05),
                (0.96, 0.05),
                (0.96, 0.0),
                (0.92, -0.05),
                (-0.92, -0.05),
                (-0.96, 0.0)
            ],
            0.1,
            (0, beam_y, 0),
            material,
            bevel = 0.008,
            bevel_segments = 1
        );
    }
}

// The seat hung 5 cm lower than it was; the cushion's top is the content's `seat` height.
const PORCH_DROP: f64 = 0.05;
pub(super) const PORCH_SEAT: f64 = 0.52;

pub fn porch_swing() -> Vec<Obj> {
    let frame = new_parts(|| a_frame("", 2.0, M.porch_frame, false, 0.42));
    join(&frame, "frame", Some(P3::from((0, 0, 0))));
    let d = PORCH_DROP;
    let seat = new_parts(|| {
        for k in 0..4 {
            fbox!(
                F,
                format!("seat{k}"),
                1.3,
                0.03,
                0.105,
                (0, 0.48 - d, -0.13 + 0.115 * k as f64),
                M.porch_seat,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        for k in 0..4 {
            fbox!(
                F,
                format!("back{k}"),
                1.3,
                0.08,
                0.025,
                (0, 0.6 - d + 0.1 * k as f64, -0.19 - 0.021 * k as f64),
                M.porch_seat,
                rot = (-12, 0, 0),
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        // the back's top rail, its top edge arched; the seat's front rail
        let mut arch = vec![(-0.7, -0.03), (0.7, -0.03)];
        arch.extend((0..11).map(|i| {
            (
                0.7 - 1.4 * i as f64 / 10.0,
                0.03 + 0.025 * sin(PI * i as f64 / 10.0),
            )
        }));
        fextrude!(
            F,
            "top_rail",
            &arch,
            0.03,
            (0, 0.98 - d, -0.275),
            M.porch_seat,
            rot = (-12, 0, 0),
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "front_rail",
            1.4,
            0.06,
            0.03,
            (0, 0.44 - d, 0.235),
            M.porch_seat,
            bevel = 0.008,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            arm_board(
                &format!("arm{s}"),
                sf * 0.68,
                0.7 - d,
                -0.21,
                0.25,
                0.06,
                0.08,
                M.porch_seat,
                0.03,
            );
            revolve!(
                format!("arm_post{s}"),
                &[
                    (0.0, 0.0),
                    (0.024, 0.0),
                    (0.024, 0.03),
                    (0.017, 0.05),
                    (0.02, 0.12),
                    (0.014, 0.18),
                    (0.022, 0.2),
                    (0.022, 0.22),
                    (0.0, 0.22)
                ],
                at = F.at(sf * 0.68, 0.47 - d, 0.21),
                material = M.porch_seat,
                segments = 10
            );
            fbox!(
                F,
                format!("side{s}"),
                0.05,
                0.06,
                0.48,
                (sf * 0.68, 0.45 - d, 0.03),
                M.porch_seat,
                bevel = 0.008,
                bevel_segments = 1
            );
            // two chains from the seat's corners to a ring, one chain up to the hook under the beam
            let ring = (sf * 0.67, 1.42, 0.02);
            for z in [-0.18, 0.24] {
                fsweep!(
                    F,
                    format!("chain{s}{}", float_repr(z)),
                    &[(sf * 0.66, 0.5 - d, z), ring],
                    0.006,
                    M.chain,
                    verts = 5
                );
                fcyl!(
                    F,
                    format!("eye{s}{}", float_repr(z)),
                    0.012,
                    0.03,
                    (sf * 0.66, 0.5 - d, z),
                    M.chain,
                    verts = 6
                );
            }
            torus!(
                format!("ring{s}"),
                0.022,
                0.006,
                at = F.at(ring.0, ring.1, ring.2),
                material = M.chain,
                rot = (0, 90, 0),
                major_segments = 10,
                minor_segments = 4
            );
            fsweep!(
                F,
                format!("chain_top{s}"),
                &[ring, (sf * 0.68, 1.93, 0.0)],
                0.006,
                M.chain,
                verts = 5
            );
            torus!(
                format!("hook{s}"),
                0.02,
                0.005,
                at = F.at(sf * 0.68, 1.95, 0.0),
                material = M.chain,
                rot = (0, 90, 0),
                major_segments = 10,
                minor_segments = 4
            );
        }
        soft!(
            F,
            "cushion",
            1.28,
            0.08,
            0.42,
            (0, PORCH_SEAT - 0.04, 0.05),
            M.porch_cushion,
            round = 0.03,
            puff = 0.01,
            flat = 2,
            piping = 0.005,
            round_segments = 1
        );
    });
    let seat = hinged(join(&seat, "seat", None), F.at(0, 1.98, 0));
    settle!(centre = false);
    rock(
        seat,
        "swing",
        10.0,
        Rock {
            seconds: 2.0,
            axis: Axis::X,
        },
    );
    meshes()
}

// --- the swing set: two seats on chains from a round-timber A-frame; they swing while played on ---------------
pub fn swing_set() -> Vec<Obj> {
    let frame = new_parts(|| a_frame("", 1.92, M.swing_frame, true, 0.72)); // splayed for seats swinging 20°
    join(&frame, "frame", Some(P3::from((0, 0, 0))));
    let seats = new_parts(|| {
        for x in [-0.48, 0.48] {
            // a belt seat: a strap sagging between its chains, edges rolled
            let belt: Vec<(f64, f64, i32)> = [-0.21, -0.14, -0.07, 0.0, 0.07, 0.14, 0.21]
                .iter()
                .map(|&u| (x + u, 0.48 + 0.035 * pow(u / 0.2, 2.0), 0))
                .collect();
            crate::eased(fsweep!(
                F,
                format!("seat{}", float_repr(x)),
                &belt,
                0,
                M.swing_seat,
                profile = rounded_rect(0.012, 0.2, 0.005, 1),
                up = (0, 0, 1),
                resolution = 2
            ));
            for dx in [-0.2, 0.2] {
                fsweep!(
                    F,
                    format!("chain{}{}", float_repr(x), float_repr(dx)),
                    &[(x + dx, 0.53, 0), (x + dx, 1.86, 0)],
                    0.006,
                    M.chain,
                    verts = 5
                );
                fbox!(
                    F,
                    format!("clip{}{}", float_repr(x), float_repr(dx)),
                    0.03,
                    0.04,
                    0.03,
                    (x + dx, 0.515, 0),
                    M.chain,
                    bevel = 0.008,
                    bevel_segments = 1
                );
                // the hanger round the beam: a ring the chain hangs from (it turns on its own axis as it swings)
                torus!(
                    format!("hanger{}{}", float_repr(x), float_repr(dx)),
                    0.052,
                    0.008,
                    at = F.at(x + dx, 1.92, 0),
                    material = M.chain,
                    rot = (0, 90, 0),
                    major_segments = 12,
                    minor_segments = 4
                );
            }
        }
    });
    let seats = hinged(join(&seats, "seats", None), F.at(0, 1.92, 0));
    settle!(centre = false);
    rock(
        seats,
        "swing",
        20.0,
        Rock {
            seconds: 1.6,
            axis: Axis::X,
        },
    );
    meshes()
}

// --- the cedar hot tub: banded staves, a step, water; the cover folds back for a soak and the water bubbles ----
const TUB_R: f64 = 0.92;
const TUB_SEAT: f64 = 0.35; // the bench's top (`seats.hotTubCedar`) and its inner edge's radius
const TUB_BENCH: f64 = 0.44;

pub fn hot_tub_cedar() -> Vec<Obj> {
    let tub = new_parts(|| {
        // the staves: one wall, a V between each of 32, the inside round; a floor; the rim round their tops
        let mut out = Vec::new();
        let n = 32;
        for k in 0..n {
            for (f, r) in [(0.0, TUB_R - 0.006), (0.12, TUB_R), (0.88, TUB_R)] {
                let a = TAU * (k as f64 + f) / n as f64;
                out.push((r * cos(a), r * sin(a)));
            }
        }
        let hole: Vec<(f64, f64)> = (0..n)
            .map(|k| {
                (
                    (TUB_R - 0.06) * cos(TAU * k as f64 / n as f64),
                    (TUB_R - 0.06) * sin(TAU * k as f64 / n as f64),
                )
            })
            .collect();
        fextrude!(
            F,
            "staves",
            &out,
            0.78,
            (0, 0.41, 0),
            M.cedar,
            plane = artkit::place::Facing::Top,
            holes = [hole]
        );
        revolve!(
            "floor",
            &[
                (0.0, 0.0),
                (TUB_R - 0.05, 0.0),
                (TUB_R - 0.05, 0.12),
                (0.0, 0.12)
            ],
            at = F.at(0, 0, 0),
            material = M.cedar,
            segments = 24,
            smooth = false
        );
        revolve!(
            "rim",
            &[
                (TUB_R - 0.075, 0.79),
                (TUB_R + 0.005, 0.79),
                (TUB_R + 0.01, 0.81),
                (TUB_R + 0.002, 0.826),
                (TUB_R - 0.07, 0.826),
                (TUB_R - 0.078, 0.81)
            ],
            at = F.at(0, 0, 0),
            material = M.cedar,
            segments = 32
        );
        for y in [0.2, 0.6] {
            // the bands, a lug where each is drawn tight
            let band: Vec<(f64, f64, f64)> = (0..32)
                .map(|k| {
                    (
                        (TUB_R + 0.004) * cos(TAU * k as f64 / 32.0),
                        y,
                        (TUB_R + 0.004) * sin(TAU * k as f64 / 32.0),
                    )
                })
                .collect();
            fsweep!(
                F,
                format!("band{}", float_repr(y)),
                &band,
                0,
                M.cedar_band,
                profile = rounded_rect(0.04, 0.008, 0.003, 1),
                up = (0, 1, 0),
                closed = true
            );
            fbox!(
                F,
                format!("lug{}", float_repr(y)),
                0.05,
                0.05,
                0.03,
                (0.62, y, 0.68),
                M.cedar_band,
                rot = (0, 0, 0),
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        // the bench round the inside wall, under the water: the soakers' seat (`seats.hotTubCedar`)
        revolve!(
            "bench",
            &[
                (TUB_BENCH, 0.12),
                (TUB_BENCH, TUB_SEAT - 0.01),
                (TUB_BENCH + 0.01, TUB_SEAT),
                (TUB_R - 0.06, TUB_SEAT)
            ],
            at = F.at(0, 0, 0),
            material = M.cedar,
            segments = 32,
            smooth = false
        );
        // the step: a tread with a dark nosing on two shaped sides and a riser
        fbox!(
            F,
            "tread",
            0.6,
            0.03,
            0.29,
            (0, 0.185, 0.81),
            M.cedar,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "tread_edge",
            0.6,
            0.022,
            0.03,
            (0, 0.188, 0.944),
            M.cedar_band,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            F,
            "riser",
            0.54,
            0.14,
            0.02,
            (0, 0.09, 0.93),
            M.cedar,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            side_slab(
                &format!("step_side{s}"),
                s as f64 * 0.285,
                &[(0.66, 0.0), (0.955, 0.0), (0.955, 0.17), (0.66, 0.17)],
                0.03,
                M.cedar,
                0.004,
            );
        }
    });
    join(&tub, "tub", Some(P3::from((0, 0, 0))));
    let water = new_parts(|| {
        revolve!(
            "water",
            &[
                (0.0, 0.69),
                (0.3, 0.688),
                (0.6, 0.692),
                (TUB_R - 0.06, 0.69),
                (TUB_R - 0.06, 0.67),
                (0.0, 0.67)
            ],
            at = F.at(0, 0, 0),
            material = M.water,
            segments = 32
        );
        for k in 0..14 {
            let a = 360.0 / 14.0 * k as f64 + ((k % 3) * 9) as f64;
            let r = 0.2 + 0.5 * ((k * 7) % 5) as f64 / 5.0;
            fsphere!(
                F,
                format!("bubble{k}"),
                0.025 + 0.01 * (k % 3) as f64,
                (r * cosd(a), 0.69, r * sind(a)),
                M.foam,
                scale = (1, 1, 0.4),
                segments = 8,
                rings = 4
            );
        }
        for k in 0..4 {
            // the jets' foam against the wall
            let a = (90 * k + 30) as f64;
            fsphere!(
                F,
                format!("jet{k}"),
                0.06,
                ((TUB_R - 0.11) * cosd(a), 0.69, (TUB_R - 0.11) * sind(a)),
                M.foam,
                scale = (1, 0.7, 0.3),
                segments = 8,
                rings = 4
            );
        }
    });
    let water = hinged(join(&water, "water", None), F.at(0, 0.68, 0));
    // The cover in two halves meeting at the seam across its middle (a fourth draw, as the shower's door, Q-05): one
    // lofted disc, each half cut from a copy of it, a hair apart at the seam, where a vinyl hinge strip joins them.
    let rings = [
        (TUB_R + 0.0, 0.84),
        (TUB_R + 0.02, 0.85),
        (TUB_R + 0.02, 0.895),
        (TUB_R + 0.005, 0.91),
        (TUB_R - 0.03, 0.912),
    ];
    // The half of the cover on side `s` of the seam (1: the front, game +z; -1: the back), cut from the disc: what
    // the cutting box (2.2 x 0.2 x 1.2 at game depth -0.597 s) covers goes, up to its face a hair past the seam.
    let half = |name: &str, s: i32| -> Obj {
        let sections: Vec<Vec<P3>> = rings
            .iter()
            .map(|&(r, y)| ring!(r, r, z = y, verts = 32))
            .collect();
        let disc = loft!(name, &sections, material = M.tub_cover, angle = 30);
        let sf = s as f32;
        let face = (sf * 0.597_f32 - sf * 0.6_f32) as f64;
        bisect(disc, (0.0, face, 0.876), (0, s, 0), true)
    };
    let back = new_parts(|| {
        half("cover", -1);
        fbox!(
            F,
            "cover_seam",
            1.84,
            0.008,
            0.02,
            (0, 0.912, -0.003),
            M.tub_cover,
            bevel = 0.003,
            bevel_segments = 1
        );
    });
    let front = new_parts(|| {
        half("cover_front", 1);
        for x in [-0.3, 0.3] {
            // the straps down to their clips, and the handles
            fbox!(
                F,
                format!("strap{}", float_repr(x)),
                0.05,
                0.06,
                0.008,
                (x, 0.865, TUB_R + 0.024),
                M.tub_cover,
                bevel = 0.002,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("clip{}", float_repr(x)),
                0.04,
                0.025,
                0.012,
                (x, 0.83, TUB_R + 0.028),
                M.tub_cover,
                bevel = 0.003,
                bevel_segments = 1
            );
            fsweep!(
                F,
                format!("handle{}", float_repr(x)),
                &[
                    (x - 0.05, 0.91, 0.6),
                    (x - 0.03, 0.919, 0.6),
                    (x + 0.03, 0.919, 0.6),
                    (x + 0.05, 0.91, 0.6)
                ],
                0.006,
                M.tub_cover,
                verts = 5,
                resolution = 2
            );
        }
    });
    // the back half's origin on its back edge, at the bottom; the front half's on the seam, at the top, about which
    // it folds over onto the back half
    let cover = hinged(join(&back, "cover", None), F.at(0, 0.84, -TUB_R - 0.02));
    let flap = hinged(join(&front, "cover_front", None), F.at(0, 0.912, 0));
    settle!(centre = false);
    attach(flap, cover);
    // Off for a soak: the front half folds back over the back half on the seam, then the two together are lifted,
    // turned down behind the tub and lowered to hang there on their folded edge, as on a cover lifter, the round
    // edge just off the ground (Blender offsets: +y is back).
    swing(
        flap,
        "cover_off",
        -180.0,
        Swing {
            seconds: 0.5,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    let keys: [PathKey; 5] = [
        (0.0, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),
        (0.5, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),
        (0.8, (0.0, 0.0, 0.25), (0.0, 0.0, 0.0)),
        (1.1, (0.0, 0.16, 1.16), (90.0, 0.0, 0.0)),
        (1.5, (0.0, 0.16, 0.11), (90.0, 0.0, 0.0)),
    ];
    path(cover, "cover_off", &keys, false);
    // previewed hung up, at its end
    let end = last_frame(1.5);
    cover.edit(|o| {
        if let Some(t) = o.tracks.iter_mut().find(|t| t.name == "cover_off") {
            t.peak = Some(end);
        }
    });
    spin(
        water,
        "bubbles",
        Spin {
            turns: 1.0,
            seconds: 2.0,
            axis: Axis::Z,
        },
    );
    meshes()
}
