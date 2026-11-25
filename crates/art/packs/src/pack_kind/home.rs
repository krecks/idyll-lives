//! The gift-knitting rocker, the welcome tea table and the hearthside quilted settee.

use super::common::{M, cup, teapot};
use crate::pack_hot_headed::common::fluted;
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, TAU, cos, cosd, pow, radians, sin, sind};
use artkit::motion::{Rock, Squash, rock, squash};
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, jitter, kit, loft, ring, settle, soft, swing,
    torus,
};

// --- the gift-knitting rocker (1x1): a spindle-back rocking chair, its knitter rocking on it ----------------
const ROCK_R: f64 = 1.1; // the rockers' radius
const CZ: f64 = -0.02; // the chair's middle (depth)

pub fn knitting_rocker() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // the yarn basket beside it: a woven body with a rim and a handle, balls of wool, needles in the work
        let P3 { x: bx, y: by, .. } = f.at(0.35, 0, 0.334);
        let mut secs = vec![ring!(0.075, 0.075, z = 0.0, verts = 24, at = (bx, by))];
        secs.extend(
            [0.0, 0.33, 0.66, 1.0]
                .iter()
                .map(|&t| fluted(0.08 + 0.02 * t, 0.012 + 0.168 * t, 24, (bx, by), 12, 0.04)),
        );
        secs.push(ring!(0.09, 0.09, z = 0.18, verts = 24, at = (bx, by)));
        secs.push(ring!(0.085, 0.085, z = 0.03, verts = 24, at = (bx, by)));
        loft!("basket", &secs, material = M.wicker);
        torus!(
            "basket_rim",
            0.098,
            0.008,
            at = f.at(0.35, 0.18, 0.334),
            material = M.honey,
            major_segments = 24,
            minor_segments = 4
        );
        kit::tube!(
            f,
            "basket_handle",
            &[
                (0.26, 0.18, 0.334),
                (0.27, 0.27, 0.334),
                (0.35, 0.31, 0.334),
                (0.43, 0.27, 0.334),
                (0.44, 0.18, 0.334),
            ],
            0.007,
            M.honey,
            verts = 6,
            round = 2
        );
        for (k, (x, y, z, r, m)) in [
            (0.32, 0.18, 0.32, 0.045, M.rose_flower),
            (0.39, 0.17, 0.36, 0.04, M.sky),
            (0.35, 0.2, 0.39, 0.04, M.mustard),
        ]
        .into_iter()
        .enumerate()
        {
            fsphere!(
                f,
                numbered("yarn"),
                r,
                (x, y, z),
                m,
                segments = 12,
                rings = 8
            );
            for j in 0..2 {
                torus!(
                    numbered("yarn_wrap"),
                    r * 0.98,
                    0.0035,
                    at = f.at(x, y, z),
                    material = m,
                    rot = (60 * j + 20 * k as i32, 30 + 50 * j, 0),
                    major_segments = 14,
                    minor_segments = 3
                );
            }
        }
        kit::tube!(
            f,
            "yarn_strand",
            &[
                (0.32, 0.2, 0.36),
                (0.3, 0.17, 0.42),
                (0.25, 0.04, 0.4),
                (0.18, 0.003, 0.3),
            ],
            0.003,
            M.rose_flower,
            verts = 4,
            round = 2
        );
        for (a, b) in [
            ((0.31, 0.15, 0.37), (0.41, 0.32, 0.29)),
            ((0.33, 0.15, 0.39), (0.38, 0.33, 0.31)),
        ] {
            fsweep!(
                f,
                numbered("needle"),
                &[a, b],
                0.004,
                M.needle,
                verts = 5,
                scale = [0.5, 1.0]
            );
            fsphere!(
                f,
                numbered("needle_end"),
                0.007,
                b,
                M.mustard,
                segments = 6,
                rings = 4
            );
        }
        fbox!(
            f,
            "knitting",
            0.12,
            0.012,
            0.09,
            (0.36, 0.2, 0.33),
            M.rose_flower,
            rot = (10, 0, 20),
            bevel = 0.004,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The chair, its cushion and the throw over its back: one part turning about its rockers' centre, so they
    // roll on the floor; `rock` rocks it with its knitter (`ride`).
    let w = 0.6;
    let py = ROCK_R + 0.025; // the pivot's height: the runners' undersides are ROCK_R + 0.025 from it
    let chair = new_parts(|| {
        fextrude!(
            f,
            "seat_board",
            &smooth_outline(
                &[
                    (-0.25, -0.2),
                    (0.25, -0.2),
                    (0.26, 0.17),
                    (0.0, 0.215),
                    (-0.26, 0.17)
                ],
                2,
                true
            ),
            0.035,
            (0.0, 0.42, CZ),
            M.rocker,
            plane = Facing::Top,
            bevel = 0.008,
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            let x = s * (w / 2.0 - 0.07);
            // the runner: one curved piece, its underside on the rockers' circle, its ends rising
            let pts: Vec<(f64, f64, f64)> = (-21..22)
                .step_by(3)
                .map(|a| {
                    (
                        x,
                        py - (ROCK_R + 0.005) * cosd(a as f64),
                        CZ + (ROCK_R + 0.005) * sind(a as f64),
                    )
                })
                .collect();
            fsweep!(
                f,
                numbered("rocker"),
                &pts,
                0,
                M.rocker,
                profile = rounded_rect(0.04, 0.03, 0.01, 1),
                up = (0, 0, 1),
                resolution = 1
            );
            fsweep!(
                f,
                numbered("front_leg"),
                &[
                    (x, 0.04, CZ + 0.2),
                    (x, 0.25, CZ + 0.19),
                    (x, 0.43, CZ + 0.18)
                ],
                0.018,
                M.rocker,
                verts = 8,
                resolution = 2,
                scale = [0.85, 1.0, 0.9]
            );
            fsweep!(
                f,
                numbered("stile"),
                &[
                    (x, 0.035, CZ - 0.2),
                    (x, 0.42, CZ - 0.19),
                    (x * 1.01, 0.75, CZ - 0.25),
                    (x * 1.02, 1.0, CZ - 0.32)
                ],
                0.02,
                M.rocker,
                verts = 8,
                resolution = 2,
                scale = [0.85, 1.0, 0.95, 0.85]
            );
            kit::turned!(
                f,
                numbered("finial"),
                x * 1.02,
                0.998,
                CZ - 0.322,
                &[
                    (0.0, 0.0),
                    (0.018, 0.0),
                    (0.024, 0.012),
                    (0.018, 0.024),
                    (0.008, 0.03),
                    (0.012, 0.038),
                    (0.0, 0.048),
                ],
                M.rocker,
                segments = 10
            );
            // the arm: rising from its post and rolling over at the front into a hand-rest
            fsweep!(
                f,
                numbered("arm"),
                &[
                    (x * 1.06, 0.655, CZ - 0.24),
                    (x * 1.08, 0.665, CZ - 0.02),
                    (x * 1.08, 0.66, CZ + 0.17),
                    (x * 1.07, 0.645, CZ + 0.21)
                ],
                0,
                M.rocker,
                profile = rounded_rect(0.03, 0.055, 0.012, 1),
                up = (0, 0, 1),
                resolution = 2,
                scale = [0.8, 1.0, 1.1, 1.0]
            );
            fsweep!(
                f,
                numbered("arm_post"),
                &[
                    (x * 1.06, 0.42, CZ + 0.17),
                    (x * 1.07, 0.55, CZ + 0.175),
                    (x * 1.08, 0.65, CZ + 0.17)
                ],
                0.015,
                M.rocker,
                verts = 8,
                resolution = 2,
                scale = [1.0, 0.8, 1.0]
            );
            kit::tube!(
                f,
                numbered("side_stretcher"),
                &[(x, 0.15, CZ - 0.19), (x, 0.15, CZ + 0.19)],
                0.011,
                M.rocker,
                verts = 6,
                round = 1
            );
        }
        for i in 0..6 {
            let xx = -0.15 + i as f64 * 0.06;
            fsweep!(
                f,
                numbered("spindle"),
                &[
                    (xx, 0.44, CZ - 0.185),
                    (xx, 0.68, CZ - 0.235),
                    (xx * 1.05, 0.94, CZ - 0.3)
                ],
                0.01,
                M.rocker,
                verts = 6,
                resolution = 2,
                scale = [0.8, 1.0, 0.8]
            );
        }
        let crest: Vec<(f64, f64, f64)> = [-0.26, -0.17, -0.085, 0.0, 0.085, 0.17, 0.26]
            .iter()
            .map(|&x| {
                (
                    x,
                    0.965 + 0.03 * (1.0 - pow(x / 0.26, 2.0)),
                    CZ - 0.31 - 0.015 * (1.0 - pow(x / 0.26, 2.0)),
                )
            })
            .collect();
        fsweep!(
            f,
            "crest",
            &crest,
            0,
            M.rocker,
            profile = rounded_rect(0.08, 0.035, 0.014, 2),
            up = (0, 0, 1)
        );
        kit::tube!(
            f,
            "stretcher",
            &[(-w / 2.0 + 0.07, 0.15, CZ), (w / 2.0 - 0.07, 0.15, CZ)],
            0.011,
            M.rocker,
            verts = 6,
            round = 1
        );
        soft!(
            f,
            "seat_cushion",
            0.42,
            0.04,
            0.38,
            (0.0, 0.46, CZ + 0.01),
            M.cushion,
            round = 0.02,
            puff = 0.012,
            piping = 0.004,
            flat = 2
        );
        for s in [-1.0, 1.0] {
            kit::tube!(
                f,
                numbered("cushion_tie"),
                &[
                    (s * 0.17, 0.46, CZ - 0.17),
                    (s * 0.2, 0.45, CZ - 0.19),
                    (s * 0.21, 0.4, CZ - 0.18),
                ],
                0.004,
                M.cushion,
                verts = 4,
                round = 2
            );
        }
        // the throw folded over the crest, hanging down the back
        let pts = [
            (0.02, 0.62, -0.25),
            (0.02, 0.85, -0.305),
            (0.02, 1.0, -0.335),
            (0.02, 1.045, -0.36),
            (0.02, 1.0, -0.388),
            (0.02, 0.8, -0.4),
        ];
        let throw = fsweep!(
            f,
            "throw",
            &pts,
            0,
            M.throw,
            profile = rounded_rect(0.36, 0.014, 0.006, 1),
            up = (1, 0, 0),
            resolution = 2
        );
        jitter!(throw, 0.004, size = 0.1, seed = 3.0, up = 0.5);
    });
    let chair = hinged(join(&chair, "chair", None), f.at(0, py, CZ));
    let objs = settle!(centre = false);
    rock(
        chair,
        "rock",
        5.0,
        Rock {
            seconds: 1.6,
            axis: Axis::X,
        },
    );
    objs
}

// --- the welcome tea table (2x1): two poufs (tops 0.45 at x = ±0.75) facing across a low table with the tea set --
const POUF_X: f64 = 0.75; // content `places` [±0.75, 0], facing ±90

pub fn welcome_tea_table() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // a braided oval rug: rings of plait in two colours to a border
        fcyl!(
            f,
            "rug",
            0.95,
            0.012,
            (0, 0.006, 0),
            M.rug,
            verts = 48,
            scale = (1, 0.44, 1)
        );
        for (k, (r, rz, m)) in [(0.95, 0.4086, M.rug_edge), (0.62, 0.26, M.rug_edge)]
            .into_iter()
            .enumerate()
        {
            let ring: Vec<(f64, f64, f64)> = (0..40)
                .map(|i| {
                    (
                        r * cos(TAU * i as f64 / 40.0),
                        0.012,
                        rz * sin(TAU * i as f64 / 40.0),
                    )
                })
                .collect();
            let (hw, ht) = if k == 0 {
                (0.02, 0.006)
            } else {
                (0.015, 0.004)
            };
            fsweep!(
                f,
                numbered("rug_braid"),
                &ring,
                0,
                m,
                profile = vec![(-ht, -hw), (ht, -hw), (ht, hw), (-ht, hw)],
                up = (0, 0, 1),
                closed = true
            );
        }
        for (x, m) in [(-POUF_X, M.pouf), (POUF_X, M.pouf_rose)] {
            let P3 { x: bx, y: by, .. } = f.at(x, 0, 0);
            // a cable-knit pouf: its side ribbed, its top a cushion with piping and a button
            loft!(
                numbered("pouf"),
                &[
                    ring!(0.18, 0.18, z = 0.0, verts = 40, at = (bx, by)),
                    fluted(0.198, 0.02, 40, (bx, by), 10, 0.035),
                    fluted(0.205, 0.12, 40, (bx, by), 10, 0.045),
                    fluted(0.205, 0.25, 40, (bx, by), 10, 0.045),
                    fluted(0.198, 0.35, 40, (bx, by), 10, 0.035),
                    ring!(0.185, 0.185, z = 0.37, verts = 40, at = (bx, by))
                ],
                material = m
            );
            kit::turned!(
                f,
                numbered("pouf_top"),
                x,
                0.0,
                0.0,
                &[
                    (0.0, 0.37),
                    (0.19, 0.37),
                    (0.205, 0.39),
                    (0.197, 0.425),
                    (0.16, 0.444),
                    (0.08, 0.45),
                    (0.0, 0.448),
                ],
                M.cushion,
                segments = 28
            );
            torus!(
                numbered("piping"),
                0.202,
                0.009,
                at = f.at(x, 0.372, 0.0),
                material = M.rug,
                major_segments = 28,
                minor_segments = 4
            );
            fsphere!(
                f,
                numbered("pouf_button"),
                0.014,
                (x, 0.448, 0.0),
                M.rug,
                scale = (1, 1, 0.45),
                segments = 8,
                rings = 4
            );
        }
        // the low table between them (top 0.36, 0.4 across: their knees stop short of its ends, their feet before
        // them), a shelf of books, a tea light
        fbox!(
            f,
            "table_top",
            0.4,
            0.04,
            0.56,
            (0, 0.34, -0.02),
            M.table,
            bevel = 0.012,
            bevel_segments = 2
        );
        for x in [-0.16, 0.16] {
            for z in [-0.25, 0.21] {
                fsweep!(
                    f,
                    numbered("table_leg"),
                    &[
                        (x, 0.32, z),
                        (
                            x + 0.008_f64.copysign(x),
                            0.0,
                            z + 0.008_f64.copysign(z + 0.02)
                        )
                    ],
                    0,
                    M.table,
                    profile = rounded_rect(0.04, 0.04, 0.01, 1),
                    up = (0, 0, 1),
                    scale = [1.0, 0.7]
                );
            }
        }
        fbox!(
            f,
            "table_shelf",
            0.32,
            0.02,
            0.48,
            (0, 0.1, -0.02),
            M.table,
            bevel = 0.005,
            bevel_segments = 1
        );
        for (k, (y, h, m, r)) in [(0.11, 0.035, M.mustard, 0), (0.145, 0.03, M.sky, 8)]
            .into_iter()
            .enumerate()
        {
            let k = k as f64;
            fbox!(
                f,
                numbered("book"),
                0.16 - 0.01 * k,
                h,
                0.12 - 0.01 * k,
                (-0.04, y + h / 2.0, -0.1),
                m,
                bevel = 0.003,
                bevel_segments = 1,
                rot = (0, 0, r)
            );
            fbox!(
                f,
                numbered("book_pages"),
                0.15 - 0.01 * k,
                h - 0.008,
                0.124 - 0.01 * k,
                (-0.035, y + h / 2.0, -0.1),
                M.candle,
                rot = (0, 0, r)
            );
        }
        kit::turned!(
            f,
            "tea_light_cup",
            0.1,
            0.36,
            -0.22,
            &[
                (0.0, 0.0),
                (0.03, 0.0),
                (0.038, 0.04),
                (0.034, 0.04),
                (0.026, 0.006),
                (0.0, 0.006),
            ],
            M.rose,
            segments = 14
        );
        fcyl!(
            f,
            "tea_light",
            0.024,
            0.016,
            (0.1, 0.372, -0.22),
            M.candle,
            verts = 12
        );
        fsphere!(
            f,
            "tea_light_flame",
            0.008,
            (0.1, 0.396, -0.22),
            M.flame,
            scale = (1, 1, 2),
            segments = 7,
            rings = 5
        );
        cup(f, -0.1, 0.36, 0.14, 200.0); // a cup before each of them
        cup(f, 0.1, 0.36, 0.1, -20.0);
        kit::turned!(
            f,
            "milk_jug",
            0.11,
            0.36,
            -0.1,
            &[
                (0.0, 0.0),
                (0.024, 0.0),
                (0.03, 0.035),
                (0.026, 0.055),
                (0.022, 0.062),
                (0.026, 0.07),
                (0.022, 0.07),
                (0.018, 0.062),
                (0.0, 0.06),
            ],
            M.porcelain,
            segments = 14
        );
        fsweep!(
            f,
            "milk_handle",
            &[
                (0.088, 0.415, -0.1),
                (0.075, 0.41, -0.1),
                (0.075, 0.385, -0.1),
                (0.086, 0.378, -0.1)
            ],
            0.003,
            M.porcelain,
            verts = 4,
            resolution = 2
        );
        fsphere!(
            f,
            "milk_spout",
            0.008,
            (0.135, 0.428, -0.1),
            M.porcelain,
            scale = (1.4, 0.6, 0.6),
            segments = 6,
            rings = 4
        );
        // potted ferns at the back, between the poufs and the table: arching fronds in two greens
        for x in [-0.44, 0.44] {
            kit::turned!(
                f,
                numbered("fern_pot"),
                x,
                0.0,
                -0.33,
                &[
                    (0.0, 0.0),
                    (0.065, 0.0),
                    (0.08, 0.17),
                    (0.09, 0.175),
                    (0.09, 0.2),
                    (0.082, 0.2),
                    (0.078, 0.19),
                    (0.0, 0.19),
                ],
                M.pot,
                segments = 16
            );
            for k in 0..6 {
                let a = (k * 60 - 30) as f64;
                let (c, s) = (cosd(a), sind(a));
                let l = 0.13 + 0.03 * (k % 2) as f64;
                let s = s * if s < 0.0 { 0.8 } else { 1.0 }; // (less towards the wall)
                fsweep!(
                    f,
                    numbered("frond"),
                    &[
                        (x, 0.185, -0.33),
                        (
                            x + 0.3 * l * c,
                            0.3 + 0.03 * (k % 2) as f64,
                            -0.33 + 0.33 * l * s
                        ),
                        (x + 0.8 * l * c, 0.33, -0.33 + 0.75 * l * s),
                        (x + l * c, 0.26, -0.33 + 0.85 * l * s)
                    ],
                    0,
                    if k % 2 != 0 { M.leaf } else { M.leaf_light },
                    profile = rounded_rect(0.003, 0.05, 0.001, 1),
                    up = (0, 0, 1),
                    resolution = 2,
                    scale = [0.3, 1.0, 0.8, 0.1]
                );
            }
        }
    });
    join(&still, "body", Some(P3::ZERO));
    let pot = new_parts(|| teapot(f, -0.07, 0.36, -0.06, 1.0));
    // `pour` tips it towards its spout about the foot's edge under the spout, over the cups.
    let pot = hinged(join(&pot, "teapot", None), f.at(-0.02, 0.36, -0.06));
    let objs = settle!(centre = false);
    swing!(pot, "pour", -30.0, seconds = 0.8, axis = Axis::Y);
    objs
}

// --- the hearthside quilted settee (2x1): two seats at x = ±0.5 (tops 0.45), a patchwork quilt -------------

/// A rolled (English) arm in one piece: a panel rising from `y0` into a roll of radius `r` whose top is `top`,
/// leaning out by `lean`, swept from depth z0 to z1 at x `ax` (side `s`); a welt round its front.
#[allow(clippy::too_many_arguments)]
fn rolled_arm(
    f: &GameFrame,
    name: &str,
    ax: f64,
    s: f64,
    z0: f64,
    z1: f64,
    y0: f64,
    top: f64,
    r: f64,
    mat: Mat,
    lean: f64,
) {
    let (cy, inner) = (top - r, -r * 1.1);
    let roll: Vec<(f64, f64)> = (-40..231)
        .step_by(18)
        .map(|a| {
            (
                cy + r * sin(radians(a as f64)),
                s * (lean + r * cos(radians(a as f64))),
            )
        })
        .collect();
    let section = [
        vec![(y0, s * inner), (y0, s * (lean + r * 0.92))],
        roll,
        vec![(cy - r * 1.4, s * inner)],
    ]
    .concat(); // (height, x out)
    fsweep!(
        f,
        name,
        &[(ax, 0, z0), (ax, 0, z1)],
        0,
        mat,
        profile = &section,
        angle = 50
    );
    let mut pts = vec![(ax + s * (lean + r * 0.92) * 0.97, y0 + 0.03, z1 - 0.002)];
    pts.extend(
        section[2..section.len() - 1]
            .iter()
            .map(|&(y, x)| (ax + x * 0.9, cy + (y - cy) * 0.9, z1 - 0.002)),
    );
    fsweep!(
        f,
        format!("{name}_welt"),
        &pts,
        0.006,
        mat,
        verts = 5,
        resolution = 2
    );
}

pub fn hearth_settee() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        soft!(
            f,
            "base",
            1.66,
            0.28,
            0.78,
            (0, 0.22, -0.03),
            M.body,
            round = 0.05,
            piping = 0.006,
            flat = 2
        );
        soft!(
            f,
            "back",
            1.66,
            0.5,
            0.2,
            (0, 0.62, -0.33),
            M.body,
            round = 0.07,
            flat = 2
        );
        for x in [-0.4, 0.4] {
            soft!(
                f,
                numbered("back_cushion"),
                0.74,
                0.36,
                0.14,
                (x, 0.7, -0.2),
                M.body,
                rot = (-10, 0, 0),
                round = 0.05,
                puff = 0.03,
                piping = 0.005,
                flat = 3,
                resting = true
            );
        }
        for s in [-1.0, 1.0] {
            rolled_arm(
                f,
                &numbered("arm"),
                s * 0.82,
                s,
                -0.42,
                0.37,
                0.08,
                0.67,
                0.11,
                M.body,
                0.02,
            );
            for z in [-0.36, 0.3] {
                kit::turned!(
                    f,
                    numbered("leg"),
                    s * 0.82,
                    0.0,
                    z,
                    &[
                        (0.0, 0.0),
                        (0.018, 0.0),
                        (0.022, 0.012),
                        (0.03, 0.04),
                        (0.035, 0.07),
                        (0.028, 0.09),
                        (0.032, 0.1),
                        (0.0, 0.1),
                    ],
                    M.honey,
                    segments = 12
                );
            }
        }
        soft!(
            f,
            "pillow_l",
            0.34,
            0.3,
            0.12,
            (-0.62, 0.66, -0.12),
            M.pillow_a,
            rot = (-12, 0, 18),
            round = 0.05,
            puff = 0.035,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
        soft!(
            f,
            "pillow_r",
            0.3,
            0.28,
            0.12,
            (0.66, 0.66, -0.12),
            M.pillow_b,
            rot = (-12, 0, -18),
            round = 0.05,
            puff = 0.035,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
        for (k, (x, y, z)) in [(-0.62, 0.66, -0.06), (0.66, 0.66, -0.06)]
            .into_iter()
            .enumerate()
        {
            fsphere!(
                f,
                numbered("pillow_button"),
                0.012,
                (x + 0.01 * (1 - 2 * k as i32) as f64, y, z - 0.004),
                M.brass,
                scale = (1, 0.5, 1),
                segments = 6,
                rings = 4
            );
        }
        // a brass lantern on the left arm: a base, four posts round its light, a peaked cap, a ring
        let (lx, ly, lz) = (-0.84, 0.68, 0.12);
        kit::turned!(
            f,
            "lantern_base",
            lx,
            ly,
            lz,
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.07, 0.012),
                (0.06, 0.02),
                (0.0, 0.02),
            ],
            M.brass,
            segments = 4,
            rot = (0.0, 0.0, 45.0)
        );
        fbox!(
            f,
            "lantern_light",
            0.1,
            0.15,
            0.1,
            (lx, ly + 0.095, lz),
            M.lantern,
            bevel = 0.01,
            bevel_segments = 1
        );
        for dx in [-0.052, 0.052] {
            for dz in [-0.052, 0.052] {
                fbox!(
                    f,
                    numbered("lantern_post"),
                    0.012,
                    0.16,
                    0.012,
                    (lx + dx, ly + 0.095, lz + dz),
                    M.brass
                );
            }
        }
        for y in [ly + 0.03, ly + 0.165] {
            fbox!(
                f,
                numbered("lantern_band"),
                0.116,
                0.01,
                0.116,
                (lx, y, lz),
                M.brass
            );
        }
        fcone!(
            f,
            "lantern_cap",
            0.085,
            0.02,
            0.06,
            (lx, ly + 0.19, lz),
            M.brass,
            verts = 4,
            rot = (0, 0, 45)
        );
        torus!(
            "lantern_ring",
            0.025,
            0.005,
            at = f.at(lx, ly + 0.235, lz),
            material = M.brass,
            rot = (90, 0, 0),
            major_segments = 12,
            minor_segments = 4
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The seat cushions and the quilt over the right one, origin on their underside: `sink_in` presses them. The quilt
    // is a strip quilt, its strips running over the cushion and down its front, stitched across.
    let seats = new_parts(|| {
        let puff = 0.016;
        for x in [-0.41, 0.41] {
            soft!(
                f,
                numbered("seat_cushion"),
                0.8,
                0.12 - puff,
                0.62,
                (x, 0.39 - puff / 2.0, 0.06),
                M.body,
                round = 0.04,
                puff = puff,
                piping = 0.006,
                flat = 3,
                resting = true
            );
        }
        let (z0, z1) = (-0.21, 0.37);
        let strips = [M.patch_a, M.quilt, M.patch_b, M.patch_c, M.patch_d];
        let sw = 0.76 / strips.len() as f64;
        for (k, m) in strips.into_iter().enumerate() {
            let qx = 0.42 - 0.38 + sw * (k as f64 + 0.5);
            let pts = [
                (qx, 0.454, z0),
                (qx, 0.46, (z0 + z1) / 2.0),
                (qx, 0.449, z1 - 0.03),
                (qx, 0.433, z1 - 0.004),
                (qx, 0.405, z1 + 0.012),
                (qx, 0.21, z1 + 0.016),
            ];
            let strip = fsweep!(
                f,
                numbered("quilt_strip"),
                &pts,
                0,
                m,
                profile = rounded_rect(sw + 0.002, 0.014, 0.005, 1),
                up = (1, 0, 0),
                resolution = 2
            );
            jitter!(strip, 0.003, size = 0.12, seed = 4.0, up = 0.5);
        }
        for k in 0..4 {
            let t = (k as f64 + 0.5) / 4.0;
            fbox!(
                f,
                numbered("quilt_stitch"),
                0.762,
                0.003,
                0.006,
                (0.42, 0.464 + 0.002 * sin(PI * t), z0 + (z1 - 0.06 - z0) * t),
                M.quilt
            );
        }
    });
    let seats = hinged(join(&seats, "seats", None), f.at(0, 0.33, 0.06));
    let objs = settle!(centre = false);
    squash(
        seats,
        "sink_in",
        (1, 1, 0.84),
        Squash {
            seconds: 0.6,
            ..Squash::default()
        },
    );
    objs
}
