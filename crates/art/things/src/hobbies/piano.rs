//! H1: the piano. An upright with its back to the wall and its bench in front, the pianist facing the back (content
//! `seat.facing`). Built keys to the back, then turned round (`turn_round`) and stood on the wall's face. Its fallboard
//! lies over the keys and lifts back against the case; its music desk, hinged at the top, swings its foot out so the
//! sheet music leans back for reading.

use super::M;
use super::common::{flat_quads, sheet_music};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr as r;
use artkit::kit::turn_round;
use artkit::kit::{self};
use artkit::math::{Axis, P3, PI, TAU, cos, sin};
use artkit::parts::{Settle, hinged, join, new_parts, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, soft, swing};

/// How far forward of where it was built, turned round, it stands: its back on the wall.
pub const PIANO_FORWARD: f64 = 0.027;

fn black_keys() -> Vec<f64> {
    (0..8)
        .flat_map(|o| [0.0, 0.023, 0.069, 0.092, 0.115].map(move |d| -0.577 + 0.161 * o as f64 + d))
        .collect()
}

pub fn piano() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    // (built keys to the back: the player's side is -z here, the wall's +z; turned round at the end)
    let still = new_parts(|| {
        fbox!(
            f,
            "case",
            1.5,
            0.53,
            0.15,
            (0, 0.995, 0.375),
            M.piano,
            bevel = 0.01
        );
        // the lid, its front edge moulded to a rounded nosing over a cove, overhanging the case
        fextrude!(
            f,
            "lid",
            &[
                (0.455, 1.26),
                (0.455, 1.29),
                (0.296, 1.29),
                (0.286, 1.287),
                (0.28, 1.279),
                (0.28, 1.272),
                (0.285, 1.266),
                (0.294, 1.264),
                (0.298, 1.26)
            ],
            1.54,
            (0, 0, 0),
            M.piano_trim,
            plane = Facing::Side,
            bevel = 0.003,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "lid_cove",
            &[(-0.75, 1.256, 0.3), (0.75, 1.256, 0.3)],
            0,
            M.piano_trim,
            profile = rounded_rect(0.012, 0.012, 0.005, 1)
        );
        // the upper front: a raised panel either side of the music desk, a moulded rail under them
        for s in [-1, 1] {
            kit::shaker!(
                f,
                format!("upper_panel{s}"),
                0.36,
                0.3,
                (s as f64 * 0.52, 1.05, 0.296),
                0.012,
                M.piano,
                rail = 0.045,
                panel = M.piano_trim,
                raised = true
            );
        }
        fsweep!(
            f,
            "upper_rail",
            &[(-0.74, 0.86, 0.296), (0.74, 0.86, 0.296)],
            0,
            M.piano_trim,
            profile = rounded_rect(0.026, 0.014, 0.005, 1)
        );
        fbox!(
            f,
            "keybed",
            1.46,
            0.08,
            0.32,
            (0, 0.69, 0.29),
            M.piano,
            bevel = 0.01
        );
        fsweep!(
            f,
            "key_slip",
            &[(-0.68, 0.72, 0.124), (0.68, 0.72, 0.124)],
            0,
            M.piano_trim,
            profile = vec![
                (-0.02, -0.006),
                (0.02, -0.006),
                (0.02, 0.002),
                (0.016, 0.006),
                (-0.02, 0.006)
            ]
        );
        fbox!(f, "keys", 1.3, 0.025, 0.15, (0, 0.735, 0.205), M.ivory);
        let gaps: Vec<(f64, f64, f64, f64, f64)> = (1..57)
            .map(|k| {
                (
                    -0.65 + 0.023 * k as f64 - 0.00075,
                    -0.65 + 0.023 * k as f64 + 0.00075,
                    0.131,
                    0.279,
                    0.7478,
                )
            })
            .collect();
        flat_quads(&f, "gaps", &gaps, M.key_gap);
        for (k, x) in black_keys().into_iter().filter(|&x| x <= 0.56).enumerate() {
            fbox!(
                f,
                format!("black{k}"),
                0.012,
                0.018,
                0.09,
                (x, 0.756, 0.23),
                M.ebony
            );
        }
        fbox!(
            f,
            "key_felt",
            1.3,
            0.012,
            0.012,
            (0, 0.74, 0.288),
            M.cushion
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                f,
                format!("side{s}"),
                0.06,
                0.78,
                0.18,
                (sf * 0.74, 0.39, 0.36),
                M.piano,
                bevel = 0.01
            );
            // the key cheeks: rounded at the front, a scroll under
            fextrude!(
                f,
                format!("cheek{s}"),
                &[
                    (0.32, 0.73),
                    (0.32, 0.87),
                    (0.17, 0.87),
                    (0.135, 0.86),
                    (0.12, 0.84),
                    (0.12, 0.79),
                    (0.13, 0.765),
                    (0.15, 0.75),
                    (0.16, 0.73)
                ],
                0.06,
                (sf * 0.74, 0, 0),
                M.piano,
                plane = Facing::Side,
                bevel = 0.008,
                bevel_segments = 2
            );
            // a turned console leg under each cheek, on a scrolled toe block
            kit::turned!(
                f,
                format!("leg{s}"),
                sf * 0.7,
                0.03,
                0.16,
                &[
                    (0.0, 0.0),
                    (0.024, 0.0),
                    (0.024, 0.02),
                    (0.018, 0.04),
                    (0.017, 0.3),
                    (0.024, 0.38),
                    (0.03, 0.46),
                    (0.03, 0.5),
                    (0.022, 0.55),
                    (0.028, 0.58),
                    (0.03, 0.62),
                    (0.0, 0.62),
                ],
                M.piano,
                segments = 12
            );
            fextrude!(
                f,
                format!("toe{s}"),
                &[
                    (0.26, 0.0),
                    (0.26, 0.035),
                    (0.17, 0.035),
                    (0.145, 0.03),
                    (0.13, 0.015),
                    (0.13, 0.0)
                ],
                0.07,
                (sf * 0.7, 0, 0),
                M.piano,
                plane = Facing::Side,
                bevel = 0.005,
                bevel_segments = 1
            );
            // brass candlesticks on the lid
            kit::turned!(
                f,
                format!("sconce{s}"),
                sf * 0.6,
                1.29,
                0.38,
                &[
                    (0.0, 0.0),
                    (0.034, 0.0),
                    (0.035, 0.005),
                    (0.024, 0.01),
                    (0.009, 0.022),
                    (0.007, 0.045),
                    (0.012, 0.05),
                    (0.026, 0.06),
                    (0.026, 0.064),
                    (0.019, 0.07),
                    (0.0, 0.07),
                ],
                M.brass,
                segments = 10
            );
            fcyl!(
                f,
                format!("candle{s}"),
                0.0175,
                0.12,
                (sf * 0.6, 1.42, 0.38),
                M.candle,
                verts = 12,
                bevel = 0.003
            );
            fcyl!(
                f,
                format!("wick{s}"),
                0.002,
                0.01,
                (sf * 0.6, 1.475, 0.38),
                M.ink,
                verts = 4
            );
            // the lid's hinges on the back edge
            fbox!(
                f,
                format!("hinge{s}"),
                0.06,
                0.006,
                0.02,
                (sf * 0.45, 1.293, 0.445),
                M.brass
            );
        }
        // the lower case: a framed kneeboard in two raised panels, the back, plinth, the lyre and three pedals
        for s in [-1, 1] {
            kit::shaker!(
                f,
                format!("kneeboard{s}"),
                0.71,
                0.6,
                (s as f64 * 0.355, 0.35, 0.36),
                0.025,
                M.piano,
                rail = 0.06,
                panel = M.piano,
                raised = true
            );
        }
        fbox!(f, "back_lower", 1.42, 0.65, 0.02, (0, 0.325, 0.44), M.piano);
        fsweep!(
            f,
            "plinth",
            &[(-0.71, 0.018, 0.345), (0.71, 0.018, 0.345)],
            0,
            M.piano,
            profile = vec![
                (-0.018, -0.03),
                (0.018, -0.03),
                (0.018, 0.035),
                (0.012, 0.043),
                (0.0, 0.045),
                (-0.018, 0.045)
            ]
        );
        let lyre = smooth_outline(
            &[
                (0.1, 0.0),
                (0.1, 0.03),
                (0.06, 0.06),
                (0.05, 0.11),
                (0.09, 0.15),
                (0.11, 0.17),
                (-0.11, 0.17),
                (-0.09, 0.15),
                (-0.05, 0.11),
                (-0.06, 0.06),
                (-0.1, 0.03),
                (-0.1, 0.0),
            ],
            1,
            true,
        );
        fextrude!(
            f,
            "lyre",
            &lyre,
            0.02,
            (0, 0.03, 0.335),
            M.piano_trim,
            bevel = 0.004,
            bevel_segments = 1
        );
        for x in [-0.07, 0.0, 0.07] {
            // (the middle pedal's name is the script's whole 0)
            let name = if x == 0.0 {
                "pedal0".to_string()
            } else {
                format!("pedal{}", r(x))
            };
            fsweep!(
                f,
                name,
                &[
                    (x, 0.05, 0.345),
                    (x, 0.049, 0.3),
                    (x, 0.044, 0.262),
                    (x, 0.04, 0.252)
                ],
                0,
                M.brass,
                profile = rounded_rect(0.008, 0.024, 0.003, 1),
                resolution = 2,
                scale = [0.8, 0.9, 1.15, 1.1]
            );
        }
        // the bench: a buttoned cushion on a moulded frame, turned legs and stretchers (its top at 0.49 m)
        fbox!(
            f,
            "bench",
            0.86,
            0.06,
            0.34,
            (0, 0.42, -0.2),
            M.piano,
            bevel = 0.012
        );
        soft!(
            f,
            "bench_top",
            0.82,
            0.042,
            0.3,
            (0, 0.461, -0.2),
            M.cushion,
            round = 0.016,
            puff = 0.008,
            flat = 2,
            round_segments = 2,
            piping = 0.004,
            piping_material = M.piano_trim
        );
        for sx in [-1, 1] {
            for z in [-0.33, -0.07] {
                kit::turned!(
                    f,
                    format!("bench_leg{sx}{}", r(z)),
                    sx as f64 * 0.38,
                    0.0,
                    z,
                    &[
                        (0.0, 0.0),
                        (0.016, 0.0),
                        (0.017, 0.02),
                        (0.013, 0.05),
                        (0.016, 0.2),
                        (0.022, 0.29),
                        (0.024, 0.33),
                        (0.024, 0.39),
                        (0.0, 0.39),
                    ],
                    M.piano,
                    segments = 10
                );
            }
            fbox!(
                f,
                format!("bench_rail{sx}"),
                0.02,
                0.025,
                0.22,
                (sx as f64 * 0.38, 0.12, -0.2),
                M.piano,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "bench_stretcher",
            0.74,
            0.025,
            0.02,
            (0, 0.12, -0.2),
            M.piano,
            bevel = 0.005,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some(P3::ZERO));
    let fb = new_parts(|| {
        // the fallboard: a board with a rounded lip over the key slip, its maker's name in gold
        fextrude!(
            f,
            "fallboard",
            &[
                (0.3, 0.785),
                (0.3, 0.805),
                (0.142, 0.805),
                (0.132, 0.8),
                (0.128, 0.79),
                (0.13, 0.781),
                (0.142, 0.785)
            ],
            1.32,
            (0, 0, 0),
            M.piano,
            plane = Facing::Side,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            f,
            "fallboard_name",
            0.12,
            0.001,
            0.012,
            (0, 0.8055, 0.2),
            M.brass
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("fallboard_hinge{s}"),
                0.05,
                0.003,
                0.012,
                (s as f64 * 0.5, 0.8055, 0.293),
                M.brass
            );
        }
    });
    let fallboard = hinged(join(&fb, "fallboard", None), f.at(0, 0.805, 0.3));
    let dk = new_parts(|| {
        // the music desk: a board with an arched top, pierced with fretwork, and a ledge for the music
        let mut outline = vec![(0.31, 0.985)];
        outline.extend((0..13).map(|i| {
            (
                0.31 - 0.62 * i as f64 / 12.0,
                1.19 + 0.02 * sin(PI * i as f64 / 12.0),
            )
        }));
        outline.push((-0.31, 0.985));
        let holes: Vec<Vec<(f64, f64)>> = [-0.265, 0.265]
            .iter()
            .map(|&x| {
                (0..10)
                    .map(|i| {
                        (
                            x + 0.026 * cos(TAU * i as f64 / 10.0),
                            1.165 + 0.012 * sin(TAU * i as f64 / 10.0),
                        )
                    })
                    .collect()
            })
            .collect();
        fextrude!(
            f,
            "desk",
            &outline,
            0.012,
            (0, 0, 0.292),
            M.piano_trim,
            holes = holes,
            bevel = 0.003,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "desk_ledge",
            &[(-0.31, 0.994, 0.282), (0.31, 0.994, 0.282)],
            0,
            M.piano_trim,
            profile = vec![
                (-0.012, -0.012),
                (0.012, -0.012),
                (0.012, 0.006),
                (0.006, 0.012),
                (-0.012, 0.012)
            ]
        );
        sheet_music(
            &f,
            "music",
            0.4,
            0.19,
            (0.0, 1.1, 0.285),
            (0.0, 0.0, 0.0),
            0.002,
        );
    });
    let desk = hinged(join(&dk, "music_desk", None), f.at(0, 1.205, 0.298));
    turn_round(&meshes(), PIANO_FORWARD);
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    swing!(fallboard, "fallboard", -90.0, seconds = 0.9, axis = Axis::X);
    swing!(desk, "fallboard", -18.0, seconds = 0.9, axis = Axis::X);
    meshes()
}
