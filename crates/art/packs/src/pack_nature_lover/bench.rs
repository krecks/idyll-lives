//! The potting bench (2x1): a sage bench against the wall, pots and seedlings, tools on the back board.

use super::common::{_flower, _pot, _rosette, M, WALL};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{P3, cosd, sind};
use artkit::parts::{aside, join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{
    fbox, fcyl, fextrude, fsweep, grow, jitter, leaf, loft, revolve, ring, settle, soft, torus,
};
use artkit::{forms::eased, shapes::AUTO_WIDTH};

const TOP: f64 = 0.88;
const SEEDLING: (f64, f64) = (-0.62, -0.02); // the pot the seedling comes up in (x, depth)
const BOUQUET: (f64, f64) = (-0.02, 0.09); // where the tied bouquet lies on the worktop's front (x, depth: its twine)

/// The frame and boards: chamfered legs, rails, a worktop of three planks, a slatted shelf below, a tongue-and-
/// groove back board with a peg rail, and the top shelf on shaped brackets.
fn _bench_frame(f: &GameFrame) {
    for x in [-0.84, 0.84] {
        for z in [-0.39, 0.12] {
            fbox!(
                f,
                format!("leg{}{}", float_repr(x), float_repr(z)),
                0.06,
                TOP - 0.05,
                0.06,
                (x, (TOP - 0.05) / 2.0, z),
                M.sage,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
        for y in [0.12, TOP - 0.09] {
            fbox!(
                f,
                format!("side_rail{}{}", float_repr(x), float_repr(y)),
                0.04,
                0.06,
                0.45,
                (x, y, -0.135),
                M.sage,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
    }
    for y in [0.12, TOP - 0.09] {
        fbox!(
            f,
            format!("front_rail{}", float_repr(y)),
            1.62,
            0.06,
            0.04,
            (0, y, 0.12),
            M.sage,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
    for k in 0..3 {
        // the worktop's planks, a hair apart
        fbox!(
            f,
            format!("plank{k}"),
            1.76,
            0.05,
            0.198,
            (0, TOP - 0.025, -0.331 + k as f64 * 0.2),
            M.bench,
            bevel = 0.006,
            bevel_segments = 1
        );
    }
    for k in 0..4 {
        fbox!(
            f,
            format!("slat{k}"),
            1.7,
            0.025,
            0.11,
            (0, 0.2, -0.37 + k as f64 * 0.125),
            M.bench,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
    // the back board: upright boards, a groove between each, extruded upright from a grooved outline
    let (n, w) = (12, 1.76);
    let mut pts = vec![(-w / 2.0, WALL)];
    for k in 0..n {
        let x0 = -w / 2.0 + k as f64 * w / n as f64;
        pts.extend([
            (x0 + 0.005, WALL + 0.04),
            (x0 + w / n as f64 - 0.005, WALL + 0.04),
            (x0 + w / n as f64, WALL + 0.032),
        ]);
    }
    let last = pts.len() - 1;
    pts[last] = (w / 2.0, WALL);
    pts.reverse();
    fextrude!(
        f,
        "back_board",
        &pts,
        0.86,
        (0, TOP + 0.43, 0),
        M.sage,
        plane = Facing::Top,
        bevel = AUTO_WIDTH,
        bevel_segments = 1
    );
    fbox!(
        f,
        "peg_rail",
        1.7,
        0.05,
        0.025,
        (0, TOP + 0.36, WALL + 0.052),
        M.bench,
        bevel = 0.006,
        bevel_segments = 1
    );
    for x in [-0.75, -0.6, 0.72] {
        fcyl!(
            f,
            format!("peg{}", float_repr(x)),
            0.008,
            0.05,
            (x, TOP + 0.36, WALL + 0.085),
            M.cedar_dark,
            rot = (90, 0, 0),
            verts = 6
        );
    }
    fbox!(
        f,
        "top_shelf",
        1.76,
        0.03,
        0.18,
        (0, TOP + 0.62, -0.33),
        M.bench,
        bevel = 0.006,
        bevel_segments = 1
    );
    let bracket = smooth_outline(
        &[
            (0.0, 0.0),
            (0.0, 0.12),
            (0.15, 0.12),
            (0.15, 0.1),
            (0.05, 0.07),
            (0.02, 0.0),
        ],
        2,
        true,
    );
    for x in [-0.6, 0.0, 0.6] {
        let outline: Vec<(f64, f64)> = bracket
            .iter()
            .map(|&(u, v)| (WALL + 0.04 + u, TOP + 0.485 + v))
            .collect();
        fextrude!(
            f,
            format!("bracket{}", float_repr(x)),
            &outline,
            0.025,
            (x, 0, 0),
            M.sage,
            plane = Facing::Side
        );
    }
}

/// A trowel and a hand fork hanging from the peg rail, a reel of twine on its peg.
fn _hand_tools(f: &GameFrame) {
    let z = WALL + 0.07;
    for (x, kind) in [(-0.75, "trowel"), (-0.6, "fork")] {
        fsweep!(
            f,
            format!("{kind}_handle"),
            &[(x, TOP + 0.35, z), (x, TOP + 0.2, z)],
            0.015,
            M.cedar_dark,
            verts = 8,
            scale = [0.85, 1.0]
        );
        fsweep!(
            f,
            format!("{kind}_ferrule"),
            &[(x, TOP + 0.2, z), (x, TOP + 0.16, z)],
            0.012,
            M.steel,
            verts = 8,
            scale = [1.0, 0.6]
        );
        if kind == "trowel" {
            let blade = smooth_outline(
                &[
                    (-0.035, 0.16),
                    (0.035, 0.16),
                    (0.03, 0.06),
                    (0.0, 0.0),
                    (-0.03, 0.06),
                ],
                1,
                true,
            );
            let outline: Vec<(f64, f64)> = blade.iter().map(|&(u, v)| (x + u, TOP + v)).collect();
            fextrude!(
                f,
                "trowel_blade",
                &outline,
                0.004,
                (0, 0, z),
                M.steel,
                bevel = 0.0015,
                bevel_segments = 1
            );
        } else {
            for dx in [-0.022, 0.0, 0.022] {
                fsweep!(
                    f,
                    format!("fork_tine{}", float_repr(dx)),
                    &[
                        (x, TOP + 0.165, z),
                        (x + dx, TOP + 0.12, z),
                        (x + dx * 1.1, TOP + 0.05, z + 0.012)
                    ],
                    0.0035,
                    M.steel,
                    verts = 4,
                    resolution = 2
                );
            }
        }
    }
    revolve!(
        "twine",
        &eased(
            &[
                (0.012, -0.05),
                (0.03, -0.05),
                (0.03, -0.04),
                (0.024, -0.035),
                (0.024, 0.035),
                (0.03, 0.04),
                (0.03, 0.05),
                (0.012, 0.05)
            ],
            AUTO_WIDTH
        ),
        at = f.at(0.72, TOP + 0.36, WALL + 0.11),
        material = M.cedar_dark,
        segments = 14,
        rot = (90, 0, 0)
    );
    revolve!(
        "twine_wound",
        &eased(
            &[
                (0.024, -0.034),
                (0.046, -0.03),
                (0.05, 0.0),
                (0.046, 0.03),
                (0.024, 0.034)
            ],
            AUTO_WIDTH
        ),
        at = f.at(0.72, TOP + 0.36, WALL + 0.11),
        material = M.green_dark,
        segments = 14,
        rot = (90, 0, 0)
    );
}

/// A galvanised can: an oval body, a long spout with its rose, an arching handle over the top.
fn _watering_can(f: &GameFrame, x: f64, y: f64, z: f64) {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    let sections: Vec<Vec<P3>> = [
        (0.0, 0.95),
        (0.01, 1.0),
        (0.15, 1.0),
        (0.17, 0.92),
        (0.18, 0.6),
        (0.185, 0.0),
    ]
    .iter()
    .map(|&(h, k)| ring!(0.075 * k, 0.06 * k, z = bz + h, verts = 16, at = (bx, by)))
    .collect();
    loft!("can_body", &sections, material = M.can);
    fsweep!(
        f,
        "can_spout",
        &[
            (x + 0.06, y + 0.03, z),
            (x + 0.14, y + 0.1, z),
            (x + 0.2, y + 0.19, z)
        ],
        0.012,
        M.can,
        verts = 8,
        scale = [1.3, 0.8, 0.7]
    );
    revolve!(
        "can_rose",
        &[
            (0.0, 0.0),
            (0.012, 0.0),
            (0.03, 0.03),
            (0.03, 0.036),
            (0.0, 0.036)
        ],
        at = f.at(x + 0.2, y + 0.19, z),
        material = M.can,
        segments = 10,
        rot = (0, -50, 0)
    );
    fsweep!(
        f,
        "can_handle",
        &kit::fillet!(
            &[
                (x - 0.06, y + 0.17, z),
                (x - 0.03, y + 0.26, z),
                (x + 0.05, y + 0.26, z),
                (x + 0.07, y + 0.18, z)
            ],
            0.03
        ),
        0.0,
        M.can,
        profile = rounded_rect(0.012, 0.02, 0.004, 1),
        up = (0, 0, 1)
    );
}

pub fn potting_bench() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        _bench_frame(f);
        _hand_tools(f);
        // on the worktop: a seedling tray, a flowering pot, the watering can, the empty pot for the seedling
        fbox!(
            f,
            "tray",
            0.5,
            0.05,
            0.3,
            (-0.15, TOP + 0.025, -0.14),
            M.dark,
            bevel = 0.006,
            bevel_segments = 1
        );
        for i in 0..4 {
            for j in 0..2 {
                let (cx, cz) = (-0.33 + i as f64 * 0.12, -0.2 + j as f64 * 0.13);
                fbox!(
                    f,
                    format!("cell{i}{j}"),
                    0.105,
                    0.012,
                    0.115,
                    (cx, TOP + 0.046, cz),
                    M.soil
                );
                for s in [-1, 1] {
                    leaf!(
                        format!("sprout{i}{j}{s}"),
                        f.at(cx, TOP + 0.05, cz),
                        (90 * s + 30 * j) as f64,
                        35.0,
                        0.045,
                        0.03,
                        [M.green_pale, M.green_light][((i + j) % 2) as usize],
                        droop = 0.2,
                        rows = 2,
                        ring = 3
                    );
                }
            }
        }
        let sy = _pot(
            f,
            "flower_pot",
            0.25,
            TOP,
            -0.15,
            0.2,
            0.18,
            None,
            true,
            None,
        );
        _rosette(
            f,
            "flower_leaf",
            0.25,
            sy,
            -0.15,
            9,
            0.14,
            0.07,
            &[M.green_light, M.green],
            (30.0, 60.0),
            0.3,
            "oval",
            0.3,
            None,
            3,
            3,
            21,
            0.0,
        );
        for (k, m) in [M.pink, M.yellow, M.pink, M.lilac, M.yellow, M.lilac]
            .into_iter()
            .enumerate()
        {
            let a = (k * 60 + 15) as f64;
            let (fx, fz) = (0.25 + 0.075 * cosd(a), -0.15 + 0.075 * sind(a));
            fsweep!(
                f,
                format!("flower_stem{k}"),
                &[
                    (0.25 + 0.02 * cosd(a), sy, -0.15 + 0.02 * sind(a)),
                    (fx, sy + 0.17 + 0.03 * (k % 2) as f64, fz)
                ],
                0.003,
                M.stem,
                verts = 3
            );
            _flower(
                f,
                &format!("flower{k}"),
                fx,
                sy + 0.17 + 0.03 * (k % 2) as f64,
                fz,
                0.035,
                m,
                Some(if m != M.yellow { M.yellow } else { M.carrot }),
                5,
                20.0,
                -a + 90.0,
            );
        }
        _watering_can(f, 0.62, TOP, -0.1);
        _pot(
            f,
            "seedling_pot",
            SEEDLING.0,
            TOP,
            SEEDLING.1,
            0.14,
            0.12,
            None,
            true,
            None,
        );
        // below: sacks of compost, their tops rolled over, and a stack of pots
        for i in 0..3 {
            let sack = soft!(
                f,
                format!("sack{i}"),
                0.32,
                0.3,
                0.26,
                (-0.55 + i as f64 * 0.36, 0.362, -0.16),
                [M.sack, M.seed, M.sack][i as usize],
                rot = (0, 0, (i - 1) * 12),
                round = 0.06,
                puff = (0.02, 0.0, 0.03),
                flat = 2,
                round_segments = 1
            );
            jitter!(sack, 0.01, size = 0.12, seed = i as f64 * 1.3);
            fsweep!(
                f,
                format!("sack_roll{i}"),
                &[
                    (-0.55 + i as f64 * 0.36 - 0.15, 0.5, -0.16),
                    (-0.55 + i as f64 * 0.36 + 0.15, 0.5, -0.16)
                ],
                0.025,
                [M.sack, M.seed, M.sack][i as usize],
                verts = 6
            );
        }
        for (k, (yy, d)) in [(0.2125, 0.24), (0.3, 0.23), (0.385, 0.22)]
            .into_iter()
            .enumerate()
        {
            _pot(
                f,
                &format!("stack_pot{k}"),
                0.55,
                yy,
                -0.12,
                d,
                0.15,
                None,
                false,
                None,
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // pots of herbs on the top shelf: drawn with the bench, not part of its size
    let herbs = new_parts(|| {
        for (i, x) in [-0.6, -0.3, 0.0, 0.3, 0.6].into_iter().enumerate() {
            let sy = _pot(
                f,
                &format!("herb_pot{i}"),
                x,
                TOP + 0.635,
                -0.32,
                0.13 - (i % 2) as f64 * 0.02,
                0.11,
                None,
                true,
                None,
            );
            let mats: &[_] = [
                &[M.green, M.green_light][..],
                &[M.green_pale][..],
                &[M.green_light, M.green_dark][..],
            ][i % 3];
            _rosette(
                f,
                &format!("herb{i}_"),
                x,
                sy,
                -0.32,
                8,
                0.19 - 0.02 * (i % 2) as f64,
                0.05,
                mats,
                (68.0, 82.0),
                0.15,
                if i % 2 != 0 { "lance" } else { "oval" },
                0.3,
                None,
                3,
                3,
                20 + i as i128,
                0.0,
            );
        }
    });
    aside(&herbs, "herbs");
    // The seedling: a stem, two seed leaves and a tuft of true leaves, on the pot's soil: `seedling` grows it up. Its
    // origin is under the soil (and the pot's rolled rim), where it flattens out of sight: on the soil, its leaves
    // lay flat over it. Plain satin: a foliage part would grow as well since the renderer's fixes (see the notes).
    let (sx, sz) = SEEDLING;
    let sy = TOP + 0.106;
    let seedling = new_parts(|| {
        fsweep!(
            f,
            "seedling_stem",
            &[
                (sx, sy, sz),
                (sx + 0.005, sy + 0.08, sz),
                (sx + 0.01, sy + 0.15, sz)
            ],
            0.005,
            M.green,
            verts = 5,
            resolution = 2
        );
        for s in [-1, 1] {
            leaf!(
                format!("seed_leaf{s}"),
                f.at(sx + 0.006, sy + 0.1, sz),
                (90 * s) as f64,
                25.0,
                0.07,
                0.035,
                M.green_light,
                droop = 0.2,
                rows = 3,
                ring = 3
            );
        }
        for k in 0..4 {
            leaf!(
                format!("true_leaf{k}"),
                f.at(sx + 0.01, sy + 0.15, sz),
                (45 + k * 90) as f64,
                55.0,
                0.05,
                0.03,
                M.green,
                shape = "lance",
                droop = 0.15,
                rows = 2,
                ring = 3
            );
        }
    });
    let seedling = join(&seedling, "seedling", Some(f.at(sx, TOP + 0.09, sz)));
    seedling.set_scale_axis(2, 0.001);
    // The bouquet a skilled gardener ties: stems lying along the worktop's front, flower heads, a twine wrap.
    // Its origin is inside the worktop, flattened out of sight: `bouquet` grows it while it is tied.
    let (bx, bz) = BOUQUET;
    let bouquet = new_parts(|| {
        for (k, dz) in [-0.025, -0.008, 0.008, 0.025].into_iter().enumerate() {
            fsweep!(
                f,
                format!("bouquet_stem{k}"),
                &[
                    (bx - 0.15, TOP + 0.008, bz + dz * 0.3),
                    (bx + 0.1, TOP + 0.012 + 0.005 * (k % 2) as f64, bz + dz)
                ],
                0.004,
                M.green,
                verts = 4
            );
        }
        for (k, (dx, dy, dz, m)) in [
            (0.12, 0.03, -0.03, M.pink),
            (0.135, 0.045, 0.005, M.yellow),
            (0.115, 0.032, 0.035, M.lilac),
            (0.1, 0.05, -0.005, M.pink),
        ]
        .into_iter()
        .enumerate()
        {
            _flower(
                f,
                &format!("bouquet_flower{k}"),
                bx + dx,
                TOP + dy,
                bz + dz,
                0.03,
                m,
                Some(M.yellow),
                6,
                70.0,
                270.0,
            );
        }
        leaf!(
            "bouquet_leaf",
            f.at(bx + 0.05, TOP + 0.02, bz),
            280.0,
            10.0,
            0.08,
            0.035,
            M.green_light,
            rows = 3,
            ring = 3
        );
        torus!(
            "bouquet_twine",
            0.016,
            0.004,
            at = f.at(bx - 0.02, TOP + 0.017, bz),
            material = M.green_dark,
            rot = (0, 90, 0),
            major_segments = 8,
            minor_segments = 3
        );
    });
    let bouquet = join(&bouquet, "bouquet", Some(f.at(bx, TOP - 0.01, bz)));
    bouquet.set_scale_axis(2, 0.001);
    let objs = settle!(centre = false);
    grow!(seedling, "seedling", seconds = 1.0);
    grow!(bouquet, "bouquet", seconds = 0.8);
    objs
}
