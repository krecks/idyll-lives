//! The electric fireplace (2x1): a white surround with an oak mantel, a log bed glowing in its firebox.

use super::common::call::rbox;
use super::common::{P, WALL};
use super::{F, M};
use artkit::forms::smooth_outline;
use artkit::kit;
use artkit::math::{Axis, P3, TAU, cos, radians, sin};
use artkit::motion::{Ease, Path, flat, keys};
use artkit::parts::{join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, grow, jitter, loft, revolve, ring, settle, soft,
};

/// The surround's back and front.
const FP_Z0: f64 = WALL;
const FP_Z1: f64 = -0.11;
const BOX_Y0: f64 = 0.16;
const BOX_Y1: f64 = 0.68;
const BOX_X: f64 = 0.48;
const BOX_BACK: f64 = -0.24;
const LOG_Y: f64 = 0.2;
const LOG_Z: f64 = -0.165;
/// The floor cushion's middle.
const HEARTH_Z: f64 = 0.29;
/// A tongue of flame, (radius, height) of a unit one, round at the foot and drawn to a point.
const TONGUE: [(f64, f64); 7] = [
    (0.0, 0.0),
    (0.3, 0.06),
    (0.48, 0.2),
    (0.46, 0.38),
    (0.34, 0.58),
    (0.18, 0.8),
    (0.0, 1.0),
];

/// `_log`: a split log from game point `a` to `b`: a lumpy barked round, its sawn ends showing.
fn log(name: &str, a: P, b: P, r: f64, seed: i32) -> Obj {
    let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0, (a.2 + b.2) / 2.0);
    let log = fsweep!(
        F,
        name,
        &[a, mid, b],
        r,
        M.logs,
        verts = 9,
        scale = [0.95, 1.05, 0.9]
    );
    jitter!(log, r * 0.12, size = 0.04, seed = seed)
}

pub fn fireplace_electric() -> Vec<Obj> {
    let zc = (FP_Z0 + FP_Z1) / 2.0;
    let d = FP_Z1 - FP_Z0;
    let bead = [
        (0.0, 0.0),
        (0.0, 0.004),
        (0.006, 0.008),
        (0.012, 0.004),
        (0.012, 0.0),
    ];
    let still = new_parts(|| {
        // the surround: two piers, each panelled, a lintel with a frieze panel, the hearth and a plinth
        let pier = (1.6 - 2.0 * BOX_X) / 2.0;
        for s in [-1, 1] {
            let px = s as f64 * (BOX_X + pier / 2.0);
            rbox!(
                format!("pier{s}"),
                pier,
                0.98,
                d,
                (px, 0.49, zc),
                M.surround,
                r = 0.008,
                segs = 2
            );
            kit::moulding!(
                F,
                &format!("pier_bead{s}"),
                &[
                    (px + pier / 2.0 - 0.05, 0.12),
                    (px + pier / 2.0 - 0.05, 0.86),
                    (px - pier / 2.0 + 0.05, 0.86),
                    (px - pier / 2.0 + 0.05, 0.12),
                ],
                &bead,
                M.surround,
                at = FP_Z1
            );
        }
        rbox!(
            "lintel",
            2.0 * BOX_X + 0.02,
            0.98 - BOX_Y1,
            d,
            (0, (BOX_Y1 + 0.98) / 2.0, zc),
            M.surround,
            r = 0.008,
            segs = 2
        );
        kit::moulding!(
            F,
            "frieze",
            &[
                (BOX_X - 0.04, BOX_Y1 + 0.07),
                (BOX_X - 0.04, 0.92),
                (-BOX_X + 0.04, 0.92),
                (-BOX_X + 0.04, BOX_Y1 + 0.07),
            ],
            &bead,
            M.surround,
            at = FP_Z1
        );
        rbox!(
            "hearth",
            2.0 * BOX_X + 0.02,
            BOX_Y0,
            d,
            (0, BOX_Y0 / 2.0, zc),
            M.surround,
            r = 0.008,
            segs = 2
        );
        rbox!(
            "plinth",
            1.64,
            0.05,
            d + 0.02,
            (0, 0.025, zc + 0.01),
            M.surround,
            r = 0.008,
            segs = 2
        );
        // the firebox: a black frame round the opening, its glossy back and floor, a fluted back panel
        fbox!(
            F,
            "box_back",
            2.0 * BOX_X,
            BOX_Y1 - BOX_Y0,
            0.01,
            (0, (BOX_Y0 + BOX_Y1) / 2.0, BOX_BACK),
            M.firebox
        );
        for k in 0..9 {
            fbox!(
                F,
                format!("box_flute{k}"),
                0.006,
                BOX_Y1 - BOX_Y0 - 0.04,
                0.004,
                (
                    -0.4 + 0.1 * k as f64,
                    (BOX_Y0 + BOX_Y1) / 2.0,
                    BOX_BACK + 0.007
                ),
                M.firebox_line
            );
        }
        fbox!(
            F,
            "box_floor",
            2.0 * BOX_X,
            0.01,
            FP_Z1 - BOX_BACK,
            (0, BOX_Y0 + 0.005, (BOX_BACK + FP_Z1) / 2.0),
            M.firebox
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("box_side{s}"),
                0.01,
                BOX_Y1 - BOX_Y0,
                FP_Z1 - BOX_BACK,
                (
                    s as f64 * (BOX_X - 0.005),
                    (BOX_Y0 + BOX_Y1) / 2.0,
                    (BOX_BACK + FP_Z1) / 2.0
                ),
                M.firebox
            );
        }
        fbox!(
            F,
            "box_roof",
            2.0 * BOX_X,
            0.01,
            FP_Z1 - BOX_BACK,
            (0, BOX_Y1 - 0.005, (BOX_BACK + FP_Z1) / 2.0),
            M.firebox
        );
        let fw = 0.03;
        kit::moulding!(
            F,
            "box_frame",
            &[
                (BOX_X + fw, BOX_Y0 - fw),
                (BOX_X + fw, BOX_Y1 + fw),
                (-BOX_X - fw, BOX_Y1 + fw),
                (-BOX_X - fw, BOX_Y0 - fw),
            ],
            &[
                (0.0, 0.0),
                (0.0, 0.01),
                (0.006, 0.012),
                (fw, 0.012),
                (fw, 0.0),
            ],
            M.firebox,
            at = FP_Z1
        );
        fbox!(
            F,
            "switch",
            0.03,
            0.012,
            0.004,
            (BOX_X - 0.05, BOX_Y0 - 0.015, FP_Z1 + 0.014),
            M.steel
        );
        // the log bed: logs on a strip of embers and stones, which glows at night as the fire's light comes on
        rbox!(
            "embers",
            0.84,
            0.03,
            0.08,
            (0, BOX_Y0 + 0.02, LOG_Z + 0.01),
            M.ember,
            r = 0.01,
            segs = 2
        );
        for i in 0..9 {
            let st = fsphere!(
                F,
                format!("ember_stone{i}"),
                0.018,
                (
                    -0.36 + 0.09 * i as f64,
                    BOX_Y0 + 0.032,
                    LOG_Z + 0.04 - 0.02 * (i % 2) as f64
                ),
                if i % 3 != 0 { M.logs } else { M.ember },
                scale = (1.3, 1.0, 0.6),
                segments = 8,
                rings = 5
            );
            jitter!(st, 0.004, size = 0.02, seed = i);
        }
        for (i, (x, length, turn)) in [(-0.2, 0.42, 4), (0.2, 0.4, -5), (0.0, 0.36, 0)]
            .into_iter()
            .enumerate()
        {
            let y = LOG_Y + if i == 2 { 0.03 } else { 0.0 };
            let zz = LOG_Z - if i == 2 { 0.03 } else { 0.0 };
            let (c, s) = (cos(radians(turn as f64)), sin(radians(turn as f64)));
            log(
                &format!("log{i}"),
                (x - c * length / 2.0, y, zz + s * length / 2.0),
                (x + c * length / 2.0, y, zz - s * length / 2.0),
                0.04,
                i as i32,
            );
        }
        // the floor cushion in front, where residents sit cross-legged facing the fire (furniture.json seats: 0.14 high,
        // the hips 0.36 out): plump, piped, a button tufted into its middle
        soft!(
            F,
            "cushion",
            0.56,
            0.12,
            0.4,
            (0, 0.06, HEARTH_Z),
            M.hearth_cushion,
            round = 0.045,
            puff = 0.012,
            flat = 3,
            piping = 0.008,
            piping_material = M.hearth_piping
        );
        fcyl!(
            F,
            "cushion_button",
            0.016,
            0.012,
            (0, 0.13, HEARTH_Z),
            M.hearth_piping,
            verts = 10,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the mantel: a moulded shelf on two corbels, a candle vase and a framed picture on it
        soft!(
            F,
            "mantel",
            1.76,
            0.06,
            0.37,
            (0, 1.0, WALL + 0.185),
            M.mantel,
            round = 0.012,
            round_segments = 2
        );
        for s in [-1, 1] {
            fextrude!(
                F,
                format!("corbel{s}"),
                &smooth_outline(
                    &[
                        (0.0, 0.0),
                        (0.045, 0.0),
                        (0.045, -0.03),
                        (0.012, -0.12),
                        (0.0, -0.12)
                    ],
                    1,
                    true
                ),
                0.06,
                (s as f64 * 0.7, 0.97, FP_Z1),
                M.mantel,
                plane = Facing::Side,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        let at = F.at(-0.6, 0, -0.32);
        let vase: Vec<Vec<P3>> = [
            (0.0, 0.05),
            (0.01, 0.058),
            (0.08, 0.062),
            (0.18, 0.05),
            (0.24, 0.04),
            (0.26, 0.042),
        ]
        .iter()
        .map(|&(h, r)| ring!(r, r, z = 1.03 + h, verts = 20, at = (at.x, at.y)))
        .collect();
        loft!("vase", &vase, material = M.candle);
        fcyl!(
            F,
            "candle",
            0.018,
            0.06,
            (-0.6, 1.25, -0.32),
            M.candle_wax,
            verts = 10
        );
        kit::moulding!(
            F,
            "frame_pic",
            &[(0.66, 1.03), (0.66, 1.31), (0.44, 1.31), (0.44, 1.03)],
            &[
                (0.0, 0.0),
                (0.0, 0.02),
                (0.012, 0.025),
                (0.03, 0.02),
                (0.03, 0.0),
            ],
            M.picture,
            at = -0.383
        );
        fbox!(
            F,
            "print",
            0.17,
            0.22,
            0.004,
            (0.55, 1.17, -0.381),
            M.art_print
        );
        fbox!(
            F,
            "print_sky",
            0.15,
            0.09,
            0.002,
            (0.55, 1.22, -0.378),
            M.print_sky
        );
        fbox!(
            F,
            "print_hill",
            0.15,
            0.06,
            0.002,
            (0.55, 1.12, -0.3775),
            M.print_hill
        );
    });
    join(&still, "fireplaceElectric", Some(P3::ZERO));
    // the flames: tongues on the logs, their origin inside the log bed, flattened into it at rest
    let flames = new_parts(|| {
        let tongues = [
            (-0.3, 0.2, 0.17),
            (-0.15, 0.27, 0.15),
            (0.0, 0.23, 0.18),
            (0.15, 0.29, 0.15),
            (0.3, 0.2, 0.16),
        ];
        for (i, &(x, h, w)) in tongues.iter().enumerate() {
            let lean = [-6, 4, -3, 7, -5][i];
            revolve!(
                format!("flame{i}"),
                &TONGUE,
                at = F.at(x, LOG_Y - 0.02, LOG_Z),
                material = M.flame,
                segments = 14,
                scale = (w, 0.04, h + 0.02),
                rot = (0, lean, 0)
            );
            revolve!(
                format!("core{i}"),
                &TONGUE,
                at = F.at(x, LOG_Y - 0.01, LOG_Z + 0.012),
                material = M.flame_core,
                segments = 12,
                scale = (w * 0.55, 0.03, h * 0.6),
                rot = (0, lean, 0)
            );
        }
    });
    let flames = join(&flames, "flames", Some(F.at(0, LOG_Y, LOG_Z)));
    flat(flames, Axis::Z);
    settle!(centre = false);
    grow!(flames, "flames", seconds = 0.8);

    // A lit fire never stands still: its tongues lean and sway on a loop that ends where it starts
    // (`burn`, played round while the fire burns; `motions.burn`). A lean turns the whole bed about its
    // foot, so the tongues at its ends sweep further than the middle's. (No scale: the part is flattened
    // on its height axis, and a scale clip would carry that flatness into the grown flames it plays with.)
    const PERIOD: f64 = 2.4;
    // (24 samples round the loop, one every 0.1 s, joined at an even speed: an eased key would slow the flames to a
    // stop at every sample and at the seam. The exporter resamples them at the frame rate.)
    const STEPS: usize = 24;
    const STEP: f64 = PERIOD / STEPS as f64;
    // (A sum of whole cycles of sines, and of cosines taken from 1, so the loop starts and ends at rest;
    // the cosine terms keep it from swinging back through rest at half time, which reads as a metronome.)
    let wave = |terms: &[(f64, f64, bool)], amp: f64| -> Vec<(f64, f64, Ease)> {
        (0..=STEPS)
            .map(|k| {
                let t = STEP * k as f64;
                let turn = TAU * t / PERIOD;
                let swing: f64 = terms
                    .iter()
                    .map(|&(cycles, share, cosine)| {
                        let x = cycles * turn;
                        share * if cosine { cos(x) - 1.0 } else { sin(x) }
                    })
                    .sum();
                (t, amp * swing, Ease::Linear)
            })
            .collect()
    };
    let lean = wave(
        &[(2.0, 0.6, false), (5.0, 0.2, false), (1.0, 0.2, true)],
        radians(6.0),
    );
    let nod = wave(
        &[(3.0, 0.6, false), (1.0, 0.2, false), (2.0, 0.2, true)],
        radians(2.5),
    );
    let twist = wave(
        &[(4.0, 0.7, false), (2.0, 0.15, false), (1.0, 0.15, true)],
        radians(2.0),
    );
    keys(flames, "burn", Path::Rotation, 1, &lean);
    keys(flames, "burn", Path::Rotation, 0, &nod);
    keys(flames, "burn", Path::Rotation, 2, &twist);
    meshes()
}
