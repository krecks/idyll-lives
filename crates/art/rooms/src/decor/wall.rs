//! D1: wall art, the wall clock and the wall shelf.

use super::common::{
    F, M, WALL, books, clock_clips, clock_hands, disc, fan, flat, frame, leaf, mounted, oval, rect,
    shape, vine,
};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{P3, TAU, atan2, cos, degrees, pow, radians, sin, sqrt};
use artkit::parts::{join, join_all, new_parts, set_origin};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsweep, jitter, loft, revolve, ring, settle, soft, sphere};

/// A botanical print: a white moulded frame and a bevel-cut mount, a curving stem on cream paper with paired
/// pointed leaves in two greens, a bud at its tip.
pub fn print_framed() -> Vec<Obj> {
    new_parts(|| {
        let (yc, iw, ih, z) = frame(0.5, 0.71, 1.15, 0.03, 0.03, M.frame_white, None);
        let (pw, ph, z) = mounted(yc, iw, ih, z, 0.055, M.paper);
        let stem: Vec<(f64, f64)> = (0..9)
            .map(|k| {
                let t = k as f64 / 8.0;
                (0.012 * sin(2.4 * t) - 0.006, yc - 0.23 + 0.46 * t)
            })
            .collect();
        let pts: Vec<(f64, f64, f64)> = stem.iter().map(|&(x, y)| (x, y, z + 0.001)).collect();
        fsweep!(
            F,
            "stem",
            &pts,
            0.0028,
            M.olive,
            verts = 4,
            resolution = 2,
            scale = [1.3, 1.0, 0.6]
        );
        for (i, (t, s)) in [
            (0.1, -1),
            (0.2, 1),
            (0.32, -1),
            (0.42, 1),
            (0.54, -1),
            (0.64, 1),
            (0.76, -1),
            (0.85, 1),
        ]
        .into_iter()
        .enumerate()
        {
            let k = (t * 8.0) as usize;
            let (x, y) = (
                stem[k].0 + (stem[k + 1].0 - stem[k].0) * (t * 8.0 - k as f64),
                stem[k].1 + (stem[k + 1].1 - stem[k].1) * (t * 8.0 - k as f64),
            );
            let size = 1.0 - t * 0.55;
            let sf = s as f64;
            leaf(
                &format!("leaf{i}"),
                x,
                y,
                z + 0.0005 * (i % 2) as f64,
                0.034 * size,
                0.1 * size,
                -sf * (58.0 - 20.0 * t),
                [M.sage, M.olive][i % 2],
                0.003,
            );
            leaf(
                &format!("vein{i}"),
                x,
                y,
                z + 0.0025,
                0.004 * size,
                0.08 * size,
                -sf * (58.0 - 20.0 * t),
                [M.olive, M.sage][i % 2],
                0.001,
            );
        }
        let tip = stem[stem.len() - 1];
        leaf(
            "bud",
            tip.0,
            tip.1 - 0.01,
            z,
            0.026,
            0.05,
            4.0,
            M.sage,
            0.003,
        );
        shape(
            "signature",
            &rect(0.05, 0.004, yc - ph / 2.0 + 0.018, pw / 2.0 - 0.045),
            z,
            M.olive,
            0.001,
            0,
        );
    });
    join_all("printFramed");
    settle!(centre = false, floor = false)
}

/// A photo print: a thin black frame and a bevel-cut white mount round a mountain lake: two ranges of peaks with
/// snow on their ridges, pines along the shore, their reflection and a ripple on the water, a pale sun.
pub fn photo_print() -> Vec<Obj> {
    new_parts(|| {
        let profile = [
            (0.0, 0.0),
            (0.0, 0.03),
            (0.003, 0.033),
            (0.017, 0.033),
            (0.022, 0.028),
            (0.022, 0.0),
        ];
        let (yc, iw, ih, z) = frame(0.75, 0.5, 1.25, 0.033, 0.022, M.frame_black, Some(&profile));
        let (pw, ph, z) = mounted(yc, iw, ih, z, 0.05, M.sky);
        let (x0, y0) = (-pw / 2.0, yc - ph / 2.0);
        shape(
            "far",
            &[
                (x0, y0 + 0.08),
                (x0, y0 + 0.17),
                (x0 + 0.08, y0 + 0.2),
                (x0 + 0.15, y0 + 0.17),
                (x0 + 0.27, y0 + 0.22),
                (x0 + 0.42, y0 + 0.18),
                (x0 + 0.52, y0 + 0.21),
                (x0 + pw, y0 + 0.16),
                (x0 + pw, y0 + 0.08),
            ],
            z,
            M.haze,
            0.003,
            0,
        );
        shape(
            "peak0",
            &[
                (x0 + 0.02, y0 + 0.08),
                (x0 + 0.13, y0 + 0.2),
                (x0 + 0.16, y0 + 0.215),
                (x0 + 0.21, y0 + 0.29),
                (x0 + 0.26, y0 + 0.22),
                (x0 + 0.29, y0 + 0.2),
                (x0 + 0.37, y0 + 0.08),
            ],
            z + 0.001,
            M.rock,
            0.003,
            0,
        );
        shape(
            "peak1",
            &[
                (x0 + 0.24, y0 + 0.08),
                (x0 + 0.36, y0 + 0.19),
                (x0 + 0.4, y0 + 0.2),
                (x0 + 0.44, y0 + 0.25),
                (x0 + 0.5, y0 + 0.19),
                (x0 + 0.6, y0 + 0.08),
            ],
            z + 0.002,
            M.rock_dark,
            0.003,
            0,
        );
        shape(
            "snow0",
            &[
                (x0 + 0.165, y0 + 0.232),
                (x0 + 0.19, y0 + 0.24),
                (x0 + 0.205, y0 + 0.226),
                (x0 + 0.225, y0 + 0.244),
                (x0 + 0.253, y0 + 0.235),
                (x0 + 0.21, y0 + 0.29),
            ],
            z + 0.003,
            M.snow,
            0.003,
            0,
        );
        shape(
            "snow1",
            &[
                (x0 + 0.41, y0 + 0.218),
                (x0 + 0.43, y0 + 0.222),
                (x0 + 0.445, y0 + 0.212),
                (x0 + 0.475, y0 + 0.22),
                (x0 + 0.44, y0 + 0.25),
            ],
            z + 0.004,
            M.snow,
            0.003,
            0,
        );
        fbox!(
            F,
            "lake",
            pw,
            0.075,
            0.002,
            (0, y0 + 0.0425, z + 0.005),
            M.lake
        );
        for (i, (x, w)) in [(0.12, 0.08), (0.36, 0.05), (0.52, 0.09)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                F,
                format!("ripple{i}"),
                w,
                0.0025,
                0.001,
                (x0 + x, y0 + 0.03 + 0.012 * i as f64, z + 0.0065),
                M.sky
            );
        }
        fbox!(
            F,
            "shore",
            pw,
            0.012,
            0.002,
            (0, y0 + 0.081, z + 0.006),
            M.shore
        );
        for i in 0..9 {
            // pines along the far shore, and their reflection in the lake
            let (x, h) = (
                x0 + 0.03 + i as f64 * 0.06 + 0.015 * (i % 3) as f64,
                0.03 + 0.012 * ((i * 5) % 3) as f64,
            );
            flat(
                &format!("pine{i}"),
                &[
                    (x - 0.011, y0 + 0.084),
                    (x + 0.011, y0 + 0.084),
                    (x, y0 + 0.084 + h),
                ],
                z + 0.007,
                M.shore,
                0.003,
            );
            flat(
                &format!("pine_r{i}"),
                &[
                    (x - 0.009, y0 + 0.08),
                    (x, y0 + 0.08 - h * 0.7),
                    (x + 0.009, y0 + 0.08),
                ],
                z + 0.006,
                M.rock_dark,
                0.001,
            );
        }
        disc(
            "sun",
            x0 + pw - 0.08,
            y0 + ph - 0.07,
            z,
            0.026,
            M.sun,
            0.003,
            24,
        );
    });
    join_all("photoPrint");
    settle!(centre = false, floor = false)
}

const GILT: [(f64, f64); 15] = [
    (0.0, 0.0),
    (0.0, 0.028),
    (0.005, 0.034),
    (0.009, 0.042),
    (0.014, 0.047),
    (0.019, 0.042),
    (0.024, 0.036),
    (0.034, 0.034),
    (0.044, 0.038),
    (0.052, 0.046),
    (0.058, 0.05),
    (0.062, 0.047),
    (0.066, 0.036),
    (0.07, 0.03),
    (0.07, 0.0),
];

/// An oil painting in a deep gilt frame: a moulding with a bead, a cove and a raised sight edge, a carved shell at
/// each corner and the top's middle, a dark bead round the canvas; a cottage with a chimney and windows by a tree,
/// a path and a fence, hills, a field, clouds.
pub fn painting_classic() -> Vec<Obj> {
    new_parts(|| {
        let (yc, iw, ih, mut z) = frame(0.84, 0.65, 1.15, 0.05, 0.07, M.gilt, Some(&GILT));
        for sx in [-1, 1] {
            for sy in [-1, 1] {
                fan(
                    &format!("corner{sx}{sy}"),
                    sx as f64 * 0.37,
                    yc + sy as f64 * 0.275,
                    WALL + 0.039,
                    0.045,
                    degrees(atan2(sy as f64, sx as f64)),
                    M.gilt,
                    0.011,
                );
            }
        }
        fan(
            "crest",
            0,
            yc + 0.282,
            WALL + 0.039,
            0.05,
            90.0,
            M.gilt,
            0.011,
        );
        kit::moulding!(
            F,
            "bead",
            &rect(iw + 0.016, ih + 0.016, yc, 0.0),
            &[
                (0.0, 0.0),
                (0.0, 0.008),
                (0.004, 0.012),
                (0.008, 0.008),
                (0.008, 0.0)
            ],
            M.gilt_dark,
            at = z
        );
        z += 0.003;
        fbox!(F, "haze", iw, ih, 0.003, (0, yc, z + 0.0015), M.haze);
        z += 0.003;
        let y0 = yc - ih / 2.0;
        shape(
            "cloud0",
            &oval(-0.1, yc + 0.13, 0.1, 0.022, 16),
            z,
            M.cloud,
            0.003,
            0,
        );
        shape(
            "cloud0b",
            &oval(-0.06, yc + 0.145, 0.05, 0.02, 12),
            z + 0.0005,
            M.cloud,
            0.003,
            0,
        );
        shape(
            "cloud1",
            &oval(0.13, yc + 0.16, 0.075, 0.018, 16),
            z,
            M.cloud,
            0.003,
            0,
        );
        let rx = iw / 2.0 + 0.12;
        let xs: Vec<f64> = (0..17).map(|k| iw / 2.0 - iw * k as f64 / 16.0).collect();
        let mut far: Vec<(f64, f64)> = xs
            .iter()
            .map(|&x| {
                (
                    x,
                    y0 + 0.16
                        + 0.08 * sin(3.0 * x + 1.0) * sqrt(0.0_f64.max(1.0 - pow(x / rx, 2.0))),
                )
            })
            .collect();
        far.extend([(-iw / 2.0, y0 + 0.12), (iw / 2.0, y0 + 0.12)]);
        shape("hills_far", &far, z + 0.0005, M.olive, 0.003, 0);
        let hills: Vec<(f64, f64)> = xs
            .iter()
            .map(|&x| {
                (
                    x,
                    y0 + 0.12 + 0.15 * sqrt(0.0_f64.max(1.0 - pow((x + 0.06) / rx, 2.0))),
                )
            })
            .collect();
        flat("hills", &hills, z + 0.001, M.hills, 0.003);
        fbox!(
            F,
            "field",
            iw,
            0.15,
            0.002,
            (0, y0 + 0.075, z + 0.003),
            M.field
        );
        shape(
            "path",
            &[
                (0.11, y0 + 0.14),
                (0.15, y0 + 0.14),
                (0.06, y0 + 0.07),
                (0.02, y0),
                (-0.06, y0),
                (0.05, y0 + 0.08),
            ],
            z + 0.004,
            M.cottage,
            0.003,
            1,
        );
        for i in 0..7 {
            fbox!(
                F,
                format!("fence{i}"),
                0.006,
                0.03,
                0.001,
                (0.2 + i as f64 * 0.03, y0 + 0.1, z + 0.005),
                M.trunk
            );
        }
        fbox!(
            F,
            "fence_rail",
            0.2,
            0.004,
            0.001,
            (0.29, y0 + 0.108, z + 0.006),
            M.trunk
        );
        fbox!(
            F,
            "tree_trunk",
            0.018,
            0.1,
            0.002,
            (-0.17, y0 + 0.17, z + 0.004),
            M.trunk
        );
        let tree: Vec<(f64, f64)> = (0..24)
            .map(|k| {
                let a = TAU * k as f64 / 24.0;
                (
                    -0.17 + 0.075 * (1.0 + 0.1 * sin(5.0 * a)) * cos(a),
                    y0 + 0.27 + 0.085 * (1.0 + 0.1 * sin(5.0 * a)) * sin(a),
                )
            })
            .collect();
        shape("tree", &tree, z + 0.005, M.tree, 0.003, 0);
        shape(
            "tree_light",
            &oval(-0.19, y0 + 0.3, 0.035, 0.03, 12),
            z + 0.006,
            M.shore,
            0.001,
            0,
        );
        fbox!(
            F,
            "cottage",
            0.08,
            0.05,
            0.002,
            (0.13, y0 + 0.165, z + 0.004),
            M.cottage
        );
        flat(
            "roof",
            &[(0.08, y0 + 0.19), (0.18, y0 + 0.19), (0.13, y0 + 0.235)],
            z + 0.005,
            M.roof,
            0.003,
        );
        fbox!(
            F,
            "chimney",
            0.012,
            0.03,
            0.002,
            (0.158, y0 + 0.22, z + 0.0045),
            M.roof
        );
        for (i, x) in [0.105, 0.155].into_iter().enumerate() {
            fbox!(
                F,
                format!("window{i}"),
                0.014,
                0.014,
                0.001,
                (x, y0 + 0.17, z + 0.0055),
                M.trunk
            );
        }
        fbox!(
            F,
            "door",
            0.012,
            0.024,
            0.001,
            (0.13, y0 + 0.152, z + 0.0055),
            M.roof
        );
    });
    join_all("paintingClassic");
    settle!(centre = false, floor = false)
}

/// A big abstract canvas in an oak float frame (an L-section standing off the canvas): the canvas wrapped round its
/// stretcher with soft edges, a sage block with a torn edge, a mustard circle, a rust band, a brushed dark line and a
/// few small marks.
pub fn canvas_abstract() -> Vec<Obj> {
    let (w, h, y0) = (1.5, 0.9, 1.05);
    let yc = y0 + h / 2.0;
    new_parts(|| {
        kit::moulding!(
            F,
            "frame",
            &rect(w, h, yc, 0.0),
            &[
                (0.0, 0.0),
                (0.0, 0.048),
                (0.003, 0.05),
                (0.011, 0.05),
                (0.014, 0.047),
                (0.014, 0.008),
                (0.03, 0.008),
                (0.03, 0.0)
            ],
            M.oak,
            at = WALL
        );
        let (cw, ch) = (w - 0.06, h - 0.06);
        soft!(
            F,
            "canvas",
            cw,
            ch,
            0.036,
            (0, yc, WALL + 0.02),
            M.canvas,
            round = 0.006,
            round_segments = 2,
            flat = 1
        );
        let z = WALL + 0.038;
        let (bx, by, bw, bh) = (-cw * 0.17, y0 + 0.03 + ch * 0.46, cw * 0.48, ch * 0.72);
        let torn: Vec<(f64, f64)> = (0..11)
            .map(|i| {
                (
                    bx - bw / 2.0 + 0.006 * sin(i as f64 * 1.7),
                    by - bh / 2.0 + bh * i as f64 / 10.0,
                )
            })
            .collect();
        let mut block = vec![
            (bx + bw / 2.0, by - bh / 2.0),
            (bx + bw / 2.0, by + bh / 2.0),
        ];
        block.extend(torn.iter().rev());
        shape("block", &block, z, M.sage, 0.002, 0);
        disc(
            "circle",
            cw * 0.2,
            y0 + 0.03 + ch * 0.6,
            z + 0.001,
            ch * 0.24,
            M.mustard,
            0.003,
            40,
        );
        shape(
            "band",
            &[
                (cw * 0.07, y0 + 0.03 + ch * 0.12),
                (cw * 0.37, y0 + 0.03 + ch * 0.115),
                (cw * 0.372, y0 + 0.03 + ch * 0.29),
                (cw * 0.068, y0 + 0.03 + ch * 0.285),
            ],
            z + 0.003,
            M.rust,
            0.002,
            0,
        );
        let line: Vec<(f64, f64, f64)> = (0..7)
            .map(|k| {
                let t = k as f64 / 6.0;
                (
                    -cw * 0.02 + 0.01 * sin(3.0 * t),
                    y0 + 0.03 + ch * (0.14 + 0.72 * t),
                    z + 0.005,
                )
            })
            .collect();
        fsweep!(
            F,
            "line",
            &line,
            0,
            M.ink,
            profile = rounded_rect(0.004, 0.018, 0.0015, 1),
            up = (0, 1, 0),
            resolution = 2,
            scale = [0.7, 1.0, 1.1, 1.0, 0.9, 1.0, 0.6]
        );
        for (i, (x, y, r)) in [(0.4, 0.86, 0.022), (0.46, 0.8, 0.014), (-0.4, 0.1, 0.018)]
            .into_iter()
            .enumerate()
        {
            disc(
                &format!("mark{i}"),
                cw * x,
                y0 + 0.03 + ch * y,
                z + 0.001,
                r,
                [M.ink, M.rust, M.mustard][i],
                0.002,
                16,
            );
        }
        let arc: Vec<(f64, f64, f64)> = (0..8)
            .map(|k| {
                let a = radians((200 + 18 * k) as f64);
                (
                    cw * (0.3 + 0.12 * cos(a)),
                    y0 + 0.03 + ch * (0.6 + 0.2 * sin(a)),
                    z + 0.004,
                )
            })
            .collect();
        fsweep!(
            F,
            "arc",
            &arc,
            0,
            M.rust,
            profile = rounded_rect(0.003, 0.012, 0.001, 1),
            up = (0, 1, 0),
            resolution = 2
        );
    });
    join_all("canvasAbstract");
    settle!(centre = false, floor = false)
}

// --- D1: the wall clock -----------------------------------------------------------------------------------
const CLOCK_Y: f64 = 1.9;

/// A round wall clock: a steel case with a rolled bezel, a white face with a minute track (sixty ticks, the hours
/// bolder), and shaped hands that tell the game's time (ten past ten at rest, the catalogue's picture).
pub fn wall_clock() -> Vec<Obj> {
    let r = 0.18;
    let z = WALL + 0.054;
    let still = new_parts(|| {
        // (its well's floor behind the face: at 0.056 the case's front hid the face, so "Face" coloured nothing)
        revolve!(
            "case",
            &[
                (0.0, 0.0),
                (r - 0.006, 0.0),
                (r - 0.001, 0.004),
                (r, 0.012),
                (r, 0.052),
                (r - 0.002, 0.06),
                (r - 0.007, 0.0655),
                (r - 0.013, 0.066),
                (r - 0.018, 0.0625),
                (r - 0.02, 0.055),
                (r - 0.021, 0.046),
                (0.0, 0.046)
            ],
            at = F.at(0, CLOCK_Y, WALL),
            material = M.clock_case,
            rot = (90, 0, 0),
            segments = 48
        );
        disc(
            "face",
            0,
            CLOCK_Y,
            WALL + 0.05,
            r - 0.022,
            M.face,
            0.004,
            48,
        );
        for k in 0..60 {
            let a = radians((k * 6) as f64);
            let big = k % 5 == 0;
            let d = if big { r - 0.045 } else { r - 0.034 };
            fbox!(
                F,
                format!("tick{k}"),
                if big {
                    if k % 15 == 0 { 0.009 } else { 0.006 }
                } else {
                    0.0022
                },
                if big { 0.03 } else { 0.008 },
                0.0015,
                (sin(a) * d, CLOCK_Y + cos(a) * d, z + 0.00075),
                M.ink,
                rot = (0, -k * 6, 0)
            );
        }
        disc("cap", 0, CLOCK_Y, z + 0.007, 0.009, M.second, 0.004, 12);
    });
    join(&still, "body", Some(P3::ZERO));
    let hands = clock_hands(
        CLOCK_Y,
        z,
        ("hour", 0.075, 0.012, 0.003, 0.015),
        ("minute", 0.115, 0.008, 0.005, 0.018),
    );
    settle!(centre = false, floor = false);
    clock_clips(hands);
    meshes()
}

/// Two floating oak shelves with rounded front edges: a row of bound books (boards, pages, gilt bands), a trailing
/// pothos in a lipped pot spilling over the edge, a small framed picture leaning on the wall, a bud vase and a clay
/// ball.
pub fn wall_shelf() -> Vec<Obj> {
    let zc = WALL + 0.12;
    new_parts(|| {
        let board = smooth_outline(
            &[
                (-0.12, -0.02),
                (0.11, -0.02),
                (0.12, -0.012),
                (0.12, 0.012),
                (0.11, 0.02),
                (-0.12, 0.02),
            ],
            1,
            true,
        );
        for y in [1.4, 1.78] {
            fextrude!(
                F,
                format!("shelf{}", float_repr(y)),
                &board,
                0.9,
                (0, y, zc),
                M.oak,
                plane = Facing::Side
            );
        }
        let (top0, top1) = (1.42, 1.80);
        books(-0.42, -0.06, top0, zc - 0.01, 0.17, 1);
        revolve!(
            "plant_pot",
            &[
                (0.0, 0.0),
                (0.042, 0.0),
                (0.05, 0.004),
                (0.056, 0.085),
                (0.062, 0.088),
                (0.063, 0.11),
                (0.056, 0.11),
                (0.052, 0.09),
                (0.0, 0.09)
            ],
            at = F.at(0.25, top0, zc),
            material = M.pot,
            segments = 24
        );
        revolve!(
            "soil",
            &[(0.0, 0.0), (0.052, 0.0), (0.0, 0.004)],
            at = F.at(0.25, top0 + 0.088, zc),
            material = M.clay,
            segments = 12
        );
        for (k, (dx, dy, dz, r)) in [
            (0.0, 0.14, 0.0, 0.06),
            (-0.04, 0.125, 0.02, 0.045),
            (0.045, 0.13, -0.015, 0.05),
        ]
        .into_iter()
        .enumerate()
        {
            let p = sphere!(
                format!("plant{k}"),
                r,
                at = F.at(0.25 + dx, top0 + dy, zc + dz),
                material = if k > 0 { M.leaf } else { M.leaf_light },
                scale = (1, 0.9, 0.8),
                segments = 10,
                rings = 6
            );
            jitter!(p, 0.012, size = 0.03, seed = k as f64 * 1.7);
        }
        // the pothos's vines, out of the pot and down over the shelf's front edge
        vine(
            "vine0",
            &[
                (0.21, top0 + 0.1, zc + 0.03),
                (0.17, top0 + 0.03, zc + 0.07),
                (0.13, top0 + 0.008, zc + 0.085),
                (0.09, top0 + 0.006, zc + 0.09),
                (0.07, top0 + 0.006, zc + 0.08),
            ],
            M.leaf,
            1,
            0,
        );
        vine(
            "vine1",
            &[
                (0.29, top0 + 0.09, zc + 0.03),
                (0.31, top0 + 0.03, zc + 0.06),
                (0.33, top0 + 0.006, zc + 0.08),
                (0.36, top0 + 0.005, zc + 0.085),
                (0.39, top0 + 0.005, zc + 0.07),
            ],
            M.leaf_light,
            1,
            3,
        );
        vine(
            "vine2",
            &[
                (0.3, top0 + 0.11, zc - 0.02),
                (0.34, top0 + 0.06, zc - 0.04),
                (0.37, top0 + 0.02, zc - 0.05),
                (0.39, top0 + 0.006, zc - 0.03),
            ],
            M.leaf,
            1,
            5,
        );
        // a small picture leaning on the wall
        let pic = new_parts(|| {
            kit::moulding!(
                F,
                "picture",
                &rect(0.18, 0.24, top1 + 0.12, -0.25),
                &[
                    (0.0, 0.0),
                    (0.0, 0.014),
                    (0.004, 0.016),
                    (0.02, 0.016),
                    (0.02, 0.0)
                ],
                M.frame_white,
                at = WALL + 0.036
            );
            fbox!(
                F,
                "picture_art",
                0.15,
                0.21,
                0.004,
                (-0.25, top1 + 0.12, WALL + 0.04),
                M.clay
            );
            shape(
                "picture_sun",
                &oval(-0.25, top1 + 0.15, 0.03, 0.03, 12),
                WALL + 0.042,
                M.mustard,
                0.001,
                0,
            );
        });
        let art = join(&pic, "picture", None);
        set_origin(art, F.at(-0.25, top1, WALL + 0.044));
        art.set_rotation_euler_axis(0, radians(-8.0));
        let b = F.at(0.05, 0, zc);
        let (bx, by) = (b.x, b.y);
        let secs: Vec<Vec<P3>> = [
            (0.0, 0.036),
            (0.004, 0.04),
            (0.05, 0.05),
            (0.1, 0.04),
            (0.13, 0.022),
            (0.15, 0.02),
            (0.158, 0.026),
            (0.16, 0.024),
        ]
        .iter()
        .map(|&(y, r)| ring!(r, r, z = top1 + y, verts = 18, at = (bx, by)))
        .collect();
        loft!("vase", &secs, material = M.book_cream);
        sphere!(
            "ball",
            0.055,
            at = F.at(0.3, top1 + 0.055, zc),
            material = M.clay,
            segments = 16,
            rings = 10
        );
    });
    join_all("wallShelf");
    settle!(centre = false, floor = false)
}
