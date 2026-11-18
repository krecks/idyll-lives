//! The cedar sauna for one and the no-knock delivery hatch.

use super::common::{M, WALL, WALL2};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::kit;
use artkit::math::P3;
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{boulder, fbox, fcyl, fextrude, fsweep, jitter, settle, soft, swing, torus};

// --- the cedar sauna for one (1x2): seated in the middle on the bench, a glass door at the front ----------------
// the door's depth: the seated bather's feet inside it (2026-10-21: 0.42, the shoes at and past the glass: I-packs-17)
const FRONT: f64 = 0.7;

pub fn solo_sauna() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (zb, zf) = (WALL2, FRONT + 0.01); // the cabin's back and front
    let (zc, d) = ((zb + zf) / 2.0, zf - zb);
    let still = new_parts(|| {
        fbox!(
            f,
            "floor",
            0.9,
            0.06,
            d,
            (0, 0.03, zc),
            M.cedar,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "back_wall",
            0.86,
            1.96,
            0.05,
            (0, 1.04, zb + 0.025),
            M.cedar,
            bevel = 0.01,
            bevel_segments = 1
        );
        // the roof: its deck, a fascia round the front and sides, a drip edge
        fbox!(
            f,
            "roof",
            0.94,
            0.05,
            d + 0.05,
            (0, 2.06, zc + 0.025),
            M.cedar,
            bevel = 0.012,
            bevel_segments = 1
        ); // its back on the wall face
        fbox!(
            f,
            "roof_fascia",
            0.94,
            0.05,
            0.02,
            (0, 2.0, zf + 0.04),
            M.cedar_dark,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            fbox!(
                f,
                numbered("roof_fascia_side"),
                0.02,
                0.05,
                d + 0.04,
                (s as f64 * 0.46, 2.0, zc + 0.02),
                M.cedar_dark,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for s in [-1, 1] {
            let x = s as f64 * 0.425;
            // the side wall in horizontal boards, each a hair proud or shy of the next, between corner posts
            for k in 0..14 {
                let y = 0.08 + 0.137 * k as f64 + 0.0685;
                fbox!(
                    f,
                    numbered("board"),
                    0.04 + 0.002 * (k % 2) as f64,
                    0.133,
                    d - 0.1,
                    (x, y, zc - 0.01),
                    M.cedar,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
            fbox!(
                f,
                numbered("post"),
                0.05,
                1.96,
                0.06,
                (x, 1.04, zf - 0.02),
                M.cedar,
                bevel = 0.01,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("corner"),
                0.05,
                1.96,
                0.05,
                (x, 1.04, zb + 0.025),
                M.cedar,
                bevel = 0.01,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("base_board"),
                0.05,
                0.08,
                d - 0.08,
                (x, 0.1, zc - 0.01),
                M.cedar_dark,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "lintel",
            0.82,
            0.08,
            0.06,
            (0, 1.97, zf - 0.02),
            M.cedar,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            f,
            "sill",
            0.82,
            0.06,
            0.06,
            (0, 0.06, zf - 0.02),
            M.cedar,
            bevel = 0.008,
            bevel_segments = 1
        );
        // the bench (top 0.45) of rounded slats on its skirt of upright boards, the upper tier, backrest slats
        for z in [-0.16, -0.06, 0.04, 0.14] {
            fbox!(
                f,
                numbered("slat"),
                0.78,
                0.035,
                0.085,
                (0, 0.432, z),
                M.cedar,
                bevel = 0.01,
                bevel_segments = 2
            );
        }
        for k in 0..6 {
            fbox!(
                f,
                numbered("skirt_board"),
                0.125,
                0.38,
                0.03,
                (-0.325 + 0.13 * k as f64, 0.22, 0.17),
                M.cedar,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for z in [-0.54, -0.44, -0.34] {
            fbox!(
                f,
                numbered("tier_slat"),
                0.78,
                0.035,
                0.085,
                (0, 0.82, z),
                M.cedar,
                bevel = 0.01,
                bevel_segments = 2
            );
        }
        for k in 0..6 {
            fbox!(
                f,
                numbered("tier_board"),
                0.125,
                0.4,
                0.03,
                (-0.325 + 0.13 * k as f64, 0.62, -0.31),
                M.cedar,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for y in [0.98, 1.12] {
            fbox!(
                f,
                numbered("back_slat"),
                0.74,
                0.06,
                0.025,
                (0, y, -0.86),
                M.cedar,
                bevel = 0.01,
                bevel_segments = 2
            );
        }
        // the heater with its stones behind a cedar guard, a bucket and ladle, the lamp and the thermometer
        fbox!(
            f,
            "heater",
            0.26,
            0.42,
            0.22,
            (0.22, 1.07, -0.76),
            M.iron,
            bevel = 0.015,
            bevel_segments = 1
        );
        fbox!(
            f,
            "heater_top",
            0.28,
            0.03,
            0.24,
            (0.22, 1.295, -0.76),
            M.dark,
            bevel = 0.008,
            bevel_segments = 1
        );
        for k in 0..4 {
            fbox!(
                f,
                numbered("heater_vent"),
                0.18,
                0.012,
                0.004,
                (0.22, 0.95 + 0.04 * k as f64, -0.648),
                M.dark
            );
        }
        for (k, (dx, dz)) in [
            (-0.07, -0.06),
            (0.06, -0.05),
            (0.0, 0.05),
            (-0.08, 0.06),
            (0.08, 0.06),
            (0.0, -0.01),
            (-0.03, 0.0),
            (0.04, 0.01),
        ]
        .into_iter()
        .enumerate()
        {
            boulder!(
                numbered("stone"),
                0.045,
                at = f.at(0.22 + dx, 1.305, -0.76 + dz),
                material = M.stones,
                seed = k as f64 * 1.7,
                subdivisions = 1,
                scale = (1, 1, 0.75)
            );
        }
        for k in 0..3 {
            fbox!(
                f,
                numbered("guard"),
                0.025,
                0.36,
                0.025,
                (0.07 + 0.11 * k as f64, 1.07, -0.625),
                M.cedar,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        kit::turned!(
            f,
            "bucket",
            -0.22,
            0.837,
            -0.44,
            &[
                (0.0, 0.0),
                (0.08, 0.0),
                (0.1, 0.16),
                (0.092, 0.16),
                (0.073, 0.015),
                (0.0, 0.015),
            ],
            M.cedar_dark,
            segments = 16
        );
        for y in [0.87, 0.95] {
            torus!(
                numbered("bucket_band"),
                0.087 + (y - 0.837) * 0.12,
                0.005,
                at = f.at(-0.22, y, -0.44),
                material = M.copper,
                major_segments = 16,
                minor_segments = 3
            );
        }
        kit::tube!(
            f,
            "ladle",
            &[
                (-0.22, 0.86, -0.44),
                (-0.19, 0.97, -0.42),
                (-0.16, 1.08, -0.4)
            ],
            0.008,
            M.cedar_dark,
            verts = 6,
            round = 1
        );
        kit::turned!(
            f,
            "ladle_cup",
            -0.225,
            0.86,
            -0.445,
            &[
                (0.0, 0.0),
                (0.03, 0.01),
                (0.034, 0.03),
                (0.026, 0.03),
                (0.0, 0.016),
            ],
            M.cedar_dark,
            segments = 12
        );
        fbox!(
            f,
            "lamp",
            0.3,
            0.1,
            0.06,
            (0, 1.8, zb + 0.08),
            M.lamp,
            bevel = 0.02,
            bevel_segments = 1
        );
        for k in 0..5 {
            fbox!(
                f,
                numbered("lamp_louvre"),
                0.34,
                0.012,
                0.02,
                (0, 1.75 + 0.025 * k as f64, zb + 0.115),
                M.cedar,
                rot = (25, 0, 0)
            );
        }
        fbox!(
            f,
            "lamp_shade",
            0.34,
            0.03,
            0.08,
            (0, 1.86, zb + 0.08),
            M.cedar,
            bevel = 0.006,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "thermometer",
            0.06,
            0.02,
            (-0.3, 1.5, zb + 0.06),
            M.cedar_dark,
            rot = (90, 0, 0),
            verts = 18,
            bevel = 0.006,
            bevel_segments = 1
        );
        fcyl!(
            f,
            "thermometer_face",
            0.05,
            0.004,
            (-0.3, 1.5, zb + 0.072),
            M.cream,
            rot = (90, 0, 0),
            verts = 18
        );
        fbox!(
            f,
            "thermometer_needle",
            0.004,
            0.04,
            0.002,
            (-0.292, 1.512, zb + 0.075),
            M.needle,
            rot = (0, -30, 0)
        );
        // a towel over the bench's front edge
        let pts = [
            (0.0, 0.45, -0.12),
            (0.0, 0.456, 0.08),
            (0.0, 0.45, 0.16),
            (0.0, 0.425, 0.192),
            (0.0, 0.3, 0.2),
        ];
        let towel = fsweep!(
            f,
            "towel",
            &pts,
            0.0,
            M.cream,
            profile = rounded_rect(0.5, 0.012, 0.005, 1),
            up = (1, 0, 0),
            resolution = 2
        );
        jitter!(towel, 0.003, size = 0.1, seed = 1.0, up = 0.5);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The glass door, hinged on its left post, its cedar pull outside on two stand-offs: `door_open` swings it out.
    let hx = -0.39;
    let door = new_parts(|| {
        fbox!(
            f,
            "door_glass",
            0.78,
            1.84,
            0.016,
            (0, 1.02, FRONT),
            M.glass
        );
        kit::tube!(
            f,
            "pull",
            &kit::fillet!(
                &[
                    (0.3, 0.8, FRONT + 0.008),
                    (0.3, 0.8, FRONT + 0.06),
                    (0.3, 1.3, FRONT + 0.06),
                    (0.3, 1.3, FRONT + 0.008)
                ],
                0.03
            ),
            0.014,
            M.cedar_dark,
            verts = 8,
            round = 1
        );
        for y in [0.8, 1.3] {
            fcyl!(
                f,
                numbered("pull_rose"),
                0.02,
                0.006,
                (0.3, y, FRONT + 0.011),
                M.cedar_dark,
                rot = (90, 0, 0),
                verts = 10
            );
        }
    });
    let door = hinged(join(&door, "door", None), f.at(hx, 1.0, FRONT));
    let objs = settle!(centre = false);
    swing!(door, "door_open", 60.0, seconds = 0.8, ease = Ease::BackOut);
    objs
}

// --- the no-knock delivery hatch (1x1): a wall cabinet with a hatch over a ledge, a parcel drawer below ---------
const HZ: f64 = -0.19; // the cabinet's front face
const HATCH: (f64, f64, f64) = (0.52, 0.42, 1.18); // the hatch's width, height and middle height

pub fn delivery_hatch() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (hw, hh, hy) = HATCH;
    let (zc, d) = ((WALL + HZ) / 2.0, HZ - WALL);
    let (lo, hi) = (hy - hh / 2.0, hy + hh / 2.0);
    let still = new_parts(|| {
        // the cabinet, round the hatch's recess, a moulded frame round the opening
        fbox!(
            f,
            "cabinet_low",
            0.82,
            lo,
            d,
            (0, lo / 2.0, zc),
            M.slate,
            bevel = 0.015,
            bevel_segments = 1
        );
        fbox!(
            f,
            "cabinet_high",
            0.82,
            1.9 - hi,
            d,
            (0, (1.9 + hi) / 2.0, zc),
            M.slate,
            bevel = 0.015,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            fbox!(
                f,
                numbered("cabinet_side"),
                (0.82 - hw) / 2.0,
                hh + 0.02,
                d,
                (s as f64 * (0.41 + hw / 2.0) / 2.0, hy, zc),
                M.slate
            );
        }
        kit::moulding!(
            f,
            "hatch_frame",
            &[
                (hw / 2.0 + 0.005, lo - 0.005),
                (hw / 2.0 + 0.005, hi + 0.005),
                (-hw / 2.0 - 0.005, hi + 0.005),
                (-hw / 2.0 - 0.005, lo - 0.005)
            ],
            &[
                (-0.03, 0.0),
                (-0.03, 0.006),
                (-0.012, 0.012),
                (0.0, 0.012),
                (0.0, 0.0)
            ],
            M.slate,
            at = HZ
        );
        fbox!(f, "recess", hw, hh, 0.02, (0, hy, WALL + 0.07), M.dark);
        fbox!(
            f,
            "recess_floor",
            hw,
            0.01,
            d - 0.08,
            (0, lo + 0.005, zc + 0.04),
            M.steel
        );
        fbox!(
            f,
            "parcel",
            0.2,
            0.12,
            0.14,
            (0.08, lo + 0.07, zc + 0.03),
            M.kraft,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "parcel_tape",
            0.2,
            0.004,
            0.03,
            (0.08, lo + 0.132, zc + 0.03),
            M.mustard
        );
        fbox!(
            f,
            "parcel_label",
            0.06,
            0.003,
            0.05,
            (0.13, lo + 0.1315, zc + 0.06),
            M.cream
        );
        fbox!(
            f,
            "cap",
            0.84,
            0.05,
            d + 0.03,
            (0, 1.925, zc + 0.015),
            M.wood,
            bevel = 0.01,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "cap_nose",
            &[(-0.42, 1.905, HZ + 0.03), (0.42, 1.905, HZ + 0.03)],
            0.012,
            M.wood,
            verts = 8
        );
        // the ledge on two scrolled brackets, a bag and a drink waiting on it
        fbox!(
            f,
            "ledge",
            0.66,
            0.035,
            0.26,
            (0, 0.94, HZ + 0.13),
            M.wood,
            bevel = 0.01,
            bevel_segments = 2
        );
        for x in [-0.28, 0.28] {
            fextrude!(
                f,
                numbered("bracket"),
                &smooth_outline(
                    &[
                        (0.0, 0.0),
                        (0.0, 0.12),
                        (0.2, 0.12),
                        (0.18, 0.1),
                        (0.08, 0.07),
                        (0.03, 0.02)
                    ],
                    1,
                    true
                ),
                0.04,
                (x, 0.802, HZ),
                M.wood,
                plane = Facing::Side,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        soft!(
            f,
            "bag",
            0.2,
            0.26,
            0.14,
            (-0.14, 1.09, -0.06),
            M.kraft,
            round = 0.012,
            puff = 0.008,
            flat = 2
        );
        fbox!(
            f,
            "bag_fold",
            0.204,
            0.04,
            0.12,
            (-0.14, 1.235, -0.06),
            M.kraft,
            rot = (0, 4, 0),
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            f,
            "sticker",
            0.08,
            0.06,
            0.004,
            (-0.14, 1.1, 0.012),
            M.terra,
            bevel = 0.002,
            bevel_segments = 1
        );
        kit::turned!(
            f,
            "cup",
            0.14,
            0.9575,
            -0.04,
            &[
                (0.0, 0.0),
                (0.032, 0.0),
                (0.045, 0.15),
                (0.047, 0.155),
                (0.047, 0.16),
                (0.0, 0.16),
            ],
            M.porcelain,
            segments = 16
        );
        kit::turned!(
            f,
            "cup_lid",
            0.14,
            1.1175,
            -0.04,
            &[
                (0.0, 0.0),
                (0.049, 0.0),
                (0.049, 0.008),
                (0.03, 0.016),
                (0.0, 0.016),
            ],
            M.porcelain,
            segments = 16
        );
        fbox!(
            f,
            "cup_sleeve",
            0.002,
            0.05,
            0.002,
            (0.14, 1.03, -0.04),
            M.porcelain
        );
        kit::turned!(
            f,
            "cup_band",
            0.14,
            1.0,
            -0.04,
            &[(0.039, 0.0), (0.042, 0.0), (0.044, 0.05), (0.041, 0.05)],
            M.kraft,
            segments = 16
        );
        kit::tube!(
            f,
            "straw",
            &[(0.14, 1.1, -0.04), (0.16, 1.2, -0.05)],
            0.005,
            M.needle,
            verts = 6,
            round = 1
        );
        // the order screen in its bezel and the "no knock" plaque
        fbox!(
            f,
            "screen_bezel",
            0.24,
            0.17,
            0.02,
            (0.24, 1.55, HZ + 0.01),
            M.dark,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(
            f,
            "screen",
            0.2,
            0.13,
            0.004,
            (0.24, 1.55, HZ + 0.021),
            M.screen,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "screen_line_a",
            0.16,
            0.02,
            0.004,
            (0.24, 1.58, HZ + 0.023),
            M.screen_a
        );
        fbox!(
            f,
            "screen_line_b",
            0.1,
            0.02,
            0.004,
            (0.21, 1.53, HZ + 0.023),
            M.dial
        );
        fbox!(
            f,
            "plaque",
            0.26,
            0.09,
            0.012,
            (-0.18, 1.55, HZ + 0.006),
            M.cream,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "plaque_bar",
            0.2,
            0.012,
            0.004,
            (-0.18, 1.55, HZ + 0.013),
            M.needle
        );
        for x in [-0.29, -0.07] {
            fcyl!(
                f,
                numbered("plaque_screw"),
                0.006,
                0.004,
                (x, 1.55, HZ + 0.013),
                M.steel,
                rot = (90, 0, 0),
                verts = 8
            );
        }
        // the parcel drawer with its pull and letter slot, the mat
        fbox!(
            f,
            "drawer",
            0.62,
            0.3,
            0.02,
            (0, 0.4, HZ + 0.01),
            M.steel,
            bevel = 0.008,
            bevel_segments = 2
        );
        kit::pull!(
            f,
            "drawer_pull",
            0,
            0.48,
            HZ + 0.02,
            0.2,
            M.chrome,
            r = 0.007,
            out = 0.025,
            verts = 8
        );
        fbox!(
            f,
            "drawer_slot",
            0.3,
            0.012,
            0.004,
            (0, 0.33, HZ + 0.021),
            M.dark
        );
        fbox!(
            f,
            "drawer_flap",
            0.32,
            0.03,
            0.006,
            (0, 0.352, HZ + 0.022),
            M.steel,
            bevel = 0.002,
            bevel_segments = 1
        );
        soft!(
            f,
            "mat",
            0.62,
            0.014,
            0.42,
            (0, 0.007, 0.2),
            M.mat,
            round = 0.006,
            round_segments = 1,
            flat = 1
        );
        let border: Vec<(f64, f64, f64)> = rounded_rect(0.54, 0.34, 0.02, 2)
            .iter()
            .map(|&(u, v)| (u, 0.014, v + 0.2))
            .collect();
        fsweep!(
            f,
            "mat_border",
            &border,
            0.0,
            M.dark,
            profile = [
                (-0.0015, -0.008),
                (0.0015, -0.008),
                (0.0015, 0.008),
                (-0.0015, 0.008)
            ],
            up = (0, 0, 1),
            closed = true
        );
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The hatch door, hinged on its right edge: a steel door with its window framed, a bar pull; `hatch` swings it open.
    let door = new_parts(|| {
        fbox!(
            f,
            "hatch_door",
            hw,
            hh,
            0.03,
            (0, hy, HZ + 0.015),
            M.steel,
            bevel = 0.012,
            bevel_segments = 2
        );
        fbox!(
            f,
            "hatch_window",
            0.3,
            0.16,
            0.004,
            (0, hy + 0.06, HZ + 0.031),
            M.screen,
            bevel = 0.004,
            bevel_segments = 1
        );
        kit::moulding!(
            f,
            "window_frame",
            &[
                (0.155, hy - 0.025),
                (0.155, hy + 0.145),
                (-0.155, hy + 0.145),
                (-0.155, hy - 0.025)
            ],
            &[(-0.01, 0.0), (-0.01, 0.004), (0.0, 0.006), (0.004, 0.0)],
            M.steel,
            at = HZ + 0.03
        );
        kit::pull!(
            f,
            "hatch_handle",
            0,
            hy - 0.12,
            HZ + 0.03,
            0.3,
            M.chrome,
            r = 0.008,
            out = 0.03,
            verts = 8
        );
    });
    let door = hinged(join(&door, "hatch", None), f.at(hw / 2.0, hy, HZ + 0.03));
    let objs = settle!(centre = false);
    swing!(door, "hatch", -105.0, seconds = 0.8);
    objs
}
