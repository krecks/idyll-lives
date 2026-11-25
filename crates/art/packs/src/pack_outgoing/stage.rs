//! The karaoke stage and the video-call corner.

use super::common::{_speaker, M, WALL};
use crate::ease::{EASE, eased};
use artkit::forms::rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, cosd, sind};
use artkit::parts::{ASIDE, aside, hinged, join, new_parts, turn};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fsphere, fsweep, leaf, loft, revolve, settle, slide, soft, swing, torus};

// --- the karaoke stage (2x1): a low stage, a lyrics screen, two speakers and two mic stands -----------------
const MICS: [f64; 2] = [-0.36, 0.36];
const MIC_Z: f64 = 0.12;
const MIC_RISE: f64 = 0.2;

/// A stand's three splayed legs, bent tube from a collar at `top` down to rubber feet on `y`.
#[allow(clippy::too_many_arguments)]
fn _tripod(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, reach: f64, top: f64, material: Mat) {
    for a in [90, 210, 330] {
        let af = a as f64;
        let (fx, fz) = (x + reach * cosd(af), z + reach * sind(af));
        fsweep!(
            f,
            format!("{name}{a}"),
            &[
                (x, top, z),
                (
                    x + reach * 0.5 * cosd(af),
                    y + 0.06,
                    z + reach * 0.5 * sind(af)
                ),
                (fx, y + 0.012, fz)
            ],
            0.007,
            material,
            verts = 5,
            resolution = 2
        );
        fsphere!(
            f,
            format!("{name}_foot{a}"),
            0.012,
            (fx, y + 0.008, fz),
            M.dark,
            scale = (1, 1, 0.7),
            segments = 6,
            rings = 3
        );
    }
    fcyl!(
        f,
        format!("{name}_collar"),
        0.016,
        0.03,
        (x, top, z),
        M.dark,
        verts = 10
    );
}

pub fn karaoke_stage() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (zc, d) = ((WALL + 0.45) / 2.0, 0.45 - WALL);
    let still = new_parts(|| {
        fbox!(
            f,
            "stage",
            1.9,
            0.12,
            d,
            (0, 0.06, zc),
            M.body,
            bevel = 0.012,
            bevel_segments = 2
        );
        for k in 0..8 {
            // the deck's boards
            fbox!(
                f,
                format!("deck{k}"),
                1.86,
                0.016,
                (d - 0.04) / 8.0 - 0.004,
                (
                    0,
                    0.128,
                    zc - (d - 0.04) / 2.0 + (k as f64 + 0.5) * (d - 0.04) / 8.0
                ),
                M.wood,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "light_strip",
            1.86,
            0.022,
            0.012,
            (0, 0.07, 0.448),
            M.magenta,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the lyrics screen on its stand at the back: a slim bezel, the lyrics, a column on a foot
        soft!(
            f,
            "screen_case",
            0.96,
            0.56,
            0.05,
            (0, 1.35, -0.34),
            M.speaker,
            round = 0.015,
            round_segments = 1
        );
        fbox!(f, "screen", 0.9, 0.5, 0.004, (0, 1.35, -0.314), M.screen);
        for (i, (w, y)) in [(0.6, 1.47), (0.7, 1.38), (0.5, 1.29), (0.62, 1.2)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                f,
                format!("lyric{i}"),
                w,
                0.032,
                0.003,
                (0, y, -0.311),
                if i == 1 { M.lyric_hi } else { M.lyric }
            );
        }
        fbox!(
            f,
            "screen_post",
            0.05,
            1.0,
            0.05,
            (0, 0.6, -0.39),
            M.steel,
            bevel = 0.01,
            bevel_segments = 1
        );
        soft!(
            f,
            "screen_foot",
            0.5,
            0.03,
            0.16,
            (0, 0.151, -0.348),
            M.steel,
            round = 0.012,
            round_segments = 1
        );
        for s in [-1, 1] {
            let sf = s as f64;
            _speaker(
                f,
                &format!("speaker{s}"),
                sf * 0.74,
                0.47,
                -0.24,
                0.34,
                0.64,
                0.3,
                &[0.38, 0.62],
                1,
            );
            // floor spots at the stage's front corners, angled up at the singers, in their yokes
            revolve!(
                format!("spot{s}"),
                &eased(
                    &[
                        (0.0, -0.07),
                        (0.05, -0.07),
                        (0.06, 0.03),
                        (0.066, 0.05),
                        (0.058, 0.05),
                        (0.0, 0.04)
                    ],
                    EASE
                ),
                at = f.at(sf * 0.83, 0.23, 0.33),
                material = M.dark,
                segments = 14,
                rot = (-55, 0, 0)
            );
            fcyl!(
                f,
                format!("spot_lens{s}"),
                0.056,
                0.006,
                (sf * 0.83, 0.262, 0.374),
                M.sunny,
                rot = (-55, 0, 0),
                verts = 14
            );
            fsweep!(
                f,
                format!("spot_yoke{s}"),
                &[
                    (sf * 0.83 - 0.07, 0.23, 0.33),
                    (sf * 0.83 - 0.07, 0.15, 0.33),
                    (sf * 0.83 + 0.07, 0.15, 0.33),
                    (sf * 0.83 + 0.07, 0.23, 0.33)
                ],
                0.0,
                M.dark,
                profile = rect(0.008, 0.025),
                up = (0, 1, 0)
            );
        }
        for x in MICS {
            // the mic stands' tripods and lower tubes
            _tripod(
                f,
                &format!("mic_leg{x}"),
                x,
                0.136,
                MIC_Z,
                0.16,
                0.34,
                M.steel,
            );
            fsweep!(
                f,
                format!("mic_tube{x}"),
                &[(x, 0.32, MIC_Z), (x, 1.03, MIC_Z)],
                0.013,
                M.steel,
                verts = 10
            );
            fcyl!(
                f,
                format!("mic_clutch{x}"),
                0.02,
                0.04,
                (x, 1.04, MIC_Z),
                M.dark,
                verts = 10
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The upper tubes, booms and mics of both stands, lowered at rest: `mic_up` raises them 20 cm.
    let mics = new_parts(|| {
        for (i, x) in MICS.into_iter().enumerate() {
            let top = 1.33 - MIC_RISE;
            fsweep!(
                f,
                format!("mic_upper{x}"),
                &[(x, top - 0.46, MIC_Z), (x, top, MIC_Z)],
                0.009,
                M.chrome,
                verts = 8
            );
            fsweep!(
                f,
                format!("mic_boom{x}"),
                &[
                    (x, top, MIC_Z),
                    (x, top + 0.05, MIC_Z + 0.04),
                    (x, top + 0.07, MIC_Z + 0.08)
                ],
                0.008,
                M.chrome,
                verts = 6,
                resolution = 2
            );
            revolve!(
                format!("mic_body{x}"),
                &[
                    (0.0, 0.0),
                    (0.012, 0.0),
                    (0.018, 0.08),
                    (0.022, 0.09),
                    (0.0, 0.092)
                ],
                at = f.at(x, top + 0.06, MIC_Z + 0.07),
                material = if i == 0 { M.magenta } else { M.turq },
                segments = 10,
                rot = (45, 0, 0)
            );
            fsphere!(
                f,
                format!("mic_head{x}"),
                0.034,
                (x, top + 0.135, MIC_Z + 0.145),
                M.steel,
                segments = 10,
                rings = 6
            );
        }
    });
    let mics = hinged(join(&mics, "mics", None), f.at(0, 1.04, MIC_Z));
    let objs = settle!(centre = false);
    slide!(mics, "mic_up", (0.0, 0.0, MIC_RISE), seconds = 0.8);
    objs
}

// --- the video-call corner (1x1): the caller sits in the middle facing the desk, a ring light beside the laptop
const RING: (f64, f64, f64) = (0.27, 1.21, 0.34); // the ring light's middle and pivot (x, height, depth)
const RING_R: f64 = 0.11;
const RING_TILT: f64 = 60.0; // tipped back at rest, its face to the ceiling

pub fn video_call_nook() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (x, y, z) = RING;
    let yoke = RING_R + 0.03;
    let still = new_parts(|| {
        fbox!(
            f,
            "desk_top",
            0.84,
            0.03,
            0.27,
            (0, 0.74, 0.31),
            M.wood,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(
            f,
            "desk_apron",
            0.78,
            0.05,
            0.22,
            (0, 0.7, 0.31),
            M.body,
            bevel = 0.004,
            bevel_segments = 1
        );
        for sx in [-0.37, 0.37] {
            // hairpin legs: a bent rod each pair of feet
            fsweep!(
                f,
                format!("hairpin{sx}"),
                &kit::fillet!(
                    &[
                        (sx, 0.675, 0.22),
                        (sx, 0.007, 0.25),
                        (sx, 0.007, 0.37),
                        (sx, 0.675, 0.4)
                    ],
                    0.03
                ),
                0.007,
                M.body,
                verts = 6
            );
        }
        // the laptop, open towards the caller (-depth), a friend on its screen
        soft!(
            f,
            "laptop_base",
            0.34,
            0.015,
            0.23,
            (0, 0.763, 0.29),
            M.laptop,
            round = 0.006,
            round_segments = 1
        );
        fbox!(f, "laptop_keys", 0.29, 0.002, 0.1, (0, 0.771, 0.3), M.dark);
        fbox!(
            f,
            "laptop_pad",
            0.1,
            0.002,
            0.06,
            (0, 0.771, 0.22),
            M.porcelain
        );
        soft!(
            f,
            "laptop_lid",
            0.34,
            0.22,
            0.012,
            (0, 0.88, 0.41),
            M.laptop,
            rot = (-12, 0, 0),
            round = 0.006,
            round_segments = 1
        );
        fbox!(
            f,
            "laptop_screen",
            0.31,
            0.19,
            0.003,
            (0, 0.88, 0.402),
            M.screen,
            rot = (-12, 0, 0)
        );
        fsphere!(
            f,
            "friend",
            0.045,
            (0, 0.875, 0.398),
            M.face,
            scale = (1, 0.08, 1.1),
            rot = (-12, 0, 0),
            segments = 10,
            rings = 6
        );
    });
    let stand = new_parts(|| {
        // the ring light's stand: a weighted foot, a pole and the yoke the ring turns in
        revolve!(
            "ring_foot",
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.06, 0.008),
                (0.04, 0.016),
                (0.012, 0.02),
                (0.0, 0.02)
            ],
            at = f.at(x, 0.755, z),
            material = M.dark,
            segments = 14
        );
        fsweep!(
            f,
            "ring_pole",
            &[(x, 0.77, z), (x, y - RING_R - 0.02, z)],
            0.009,
            M.dark,
            verts = 8
        );
        kit::tube!(
            f,
            "yoke",
            &kit::fillet!(
                &[
                    (x - yoke, y, z),
                    (x - yoke, y - RING_R - 0.02, z),
                    (x + yoke, y - RING_R - 0.02, z),
                    (x + yoke, y, z)
                ],
                0.03
            ),
            0.008,
            M.dark,
            verts = 6,
            round = 1
        );
    });
    let rest = new_parts(|| {
        // a mug, a notebook with a pen, a little plant
        revolve!(
            "mug",
            &[
                (0.0, 0.0),
                (0.034, 0.0),
                (0.037, 0.09),
                (0.032, 0.09),
                (0.03, 0.01),
                (0.0, 0.01)
            ],
            at = f.at(-0.28, 0.755, 0.24),
            material = M.coral,
            segments = 14
        );
        fsweep!(
            f,
            "mug_handle",
            &[
                (-0.244, 0.82, 0.24),
                (-0.222, 0.81, 0.24),
                (-0.222, 0.785, 0.24),
                (-0.244, 0.775, 0.24)
            ],
            0.006,
            M.coral,
            verts = 5,
            resolution = 2
        );
        soft!(
            f,
            "notebook",
            0.14,
            0.012,
            0.2,
            (-0.3, 0.761, 0.33),
            M.sunny,
            rot = (0, 0, -10),
            round = 0.004,
            round_segments = 1
        );
        fsweep!(
            f,
            "pen",
            &[(-0.25, 0.769, 0.28), (-0.24, 0.769, 0.42)],
            0.004,
            M.dark,
            verts = 5
        );
        revolve!(
            "plant_pot",
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.054, 0.097),
                (0.0515, 0.1), // (its rim's outer corner cut)
                (0.05, 0.1),
                (0.0, 0.09)
            ],
            at = f.at(-0.33, 0.755, 0.4),
            material = M.pot,
            segments = 12
        );
        for k in 0..7 {
            leaf!(
                format!("plant_leaf{k}"),
                f.at(-0.33, 0.845, 0.4),
                (k * 51) as f64,
                (45 + 20 * (k % 2)) as f64,
                0.08,
                0.05,
                M.leaf,
                droop = 0.35,
                rows = 3,
                ring = 3
            );
        }
        // the chair: a moulded shell (seat top 0.45, centred on the caller) on four oak legs, a padded seat
        soft!(
            f,
            "chair_seat",
            0.44,
            0.06,
            0.42,
            (0, 0.415, -0.02),
            M.cushion,
            round = 0.025,
            puff = 0.008,
            flat = 2
        );
        let b = f.at(0, 0.36, -0.02);
        let shell: Vec<Vec<P3>> = [
            (
                0.0,
                [(0.21, -0.2), (0.21, 0.2), (-0.21, 0.2), (-0.21, -0.2)],
            ),
            (
                0.021,
                [(0.22, -0.21), (0.22, 0.21), (-0.22, 0.21), (-0.22, -0.21)],
            ),
            // (its rim chamfered 4 mm)
            (
                0.025,
                [
                    (0.216, -0.206),
                    (0.216, 0.206),
                    (-0.216, 0.206),
                    (-0.216, -0.206),
                ],
            ),
        ]
        .iter()
        .map(|(h, pts)| {
            pts.iter()
                .map(|&(px, py)| P3::from((b.x + px, b.y + py, b.z + h)))
                .collect()
        })
        .collect();
        loft!("chair_shell", &shell, material = M.wood, smooth = false);
        soft!(
            f,
            "chair_back",
            0.44,
            0.36,
            0.06,
            (0, 0.7, -0.25),
            M.cushion,
            rot = (-8, 0, 0),
            round = 0.028,
            puff = 0.008,
            flat = 2
        );
        for lx in [-0.18, 0.18] {
            for (lz, dz) in [(0.15, 0.03), (-0.19, -0.03)] {
                fsweep!(
                    f,
                    format!("chair_leg{lx}{lz}"),
                    &[(lx, 0.36, lz), (lx * 1.1, 0.0, lz + dz)],
                    0.016,
                    M.wood,
                    verts = 8,
                    scale = [1.0, 0.7]
                );
            }
            fsweep!(
                f,
                format!("back_post{lx}"),
                &[(lx, 0.36, -0.22), (lx, 0.6, -0.245)],
                0.012,
                M.wood,
                verts = 6
            );
        }
    });
    join(&[still, rest].concat(), "body", Some(P3::from((0, 0, 0))));
    // the ring light's stand on the desk: drawn on it, not the desk's own size
    aside(&stand, "stand");
    // The ring and its phone clip, built facing the caller, pivoting on the yoke: tipped back to rest, then
    // `ring_light` swings it up to face them.
    let ring = new_parts(|| {
        torus!(
            "ring",
            RING_R,
            0.018,
            at = f.at(x, y, z),
            material = M.ring,
            rot = (90, 0, 0),
            major_segments = 28,
            minor_segments = 8
        );
        torus!(
            "ring_back",
            RING_R,
            0.02,
            at = f.at(x, y, z + 0.008),
            material = M.dark,
            rot = (90, 0, 0),
            major_segments = 28,
            minor_segments = 6
        );
        fsweep!(
            f,
            "phone_arm",
            &[(x, y - RING_R, z), (x, y, z)],
            0.005,
            M.dark,
            verts = 6
        );
        soft!(
            f,
            "phone",
            0.06,
            0.11,
            0.008,
            (x, y, z - 0.006),
            M.dark,
            round = 0.004,
            round_segments = 1
        );
        fbox!(
            f,
            "phone_screen",
            0.052,
            0.1,
            0.002,
            (x, y, z - 0.011),
            M.screen
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fsweep!(
                f,
                format!("pivot{s}"),
                &[(x + sf * RING_R, y, z), (x + sf * yoke, y, z)],
                0.008,
                M.dark,
                verts = 8
            );
            fcyl!(
                f,
                format!("pivot_knob{s}"),
                0.014,
                0.012,
                (x + sf * (yoke - 0.014), y, z),
                M.dark,
                rot = (0, 90, 0),
                verts = 10
            );
        }
    });
    // (set aside too: the ring on its stand)
    let ring = hinged(join(&ring, &format!("ring{ASIDE}"), None), f.at(x, y, z));
    turn(ring, RING_TILT, Axis::X);
    let objs = settle!(centre = false);
    swing!(
        ring,
        "ring_light",
        -RING_TILT,
        seconds = 0.7,
        axis = Axis::X
    );
    objs
}
