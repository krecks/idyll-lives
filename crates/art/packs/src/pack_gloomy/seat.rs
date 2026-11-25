//! The brooding window seat and the moonstone soaking tub.

use super::common::{M, WALL, WALL2, book, candle, vase};
use artkit::forms::rounded_rect;
use artkit::math::{P3, cosd, sind};
use artkit::motion::{Grow, Squash, grow, squash};
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::GameFrame;
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{boulder, fbox, fextrude, fsphere, fsweep, kit, settle, soft, torus};

// --- the brooding window seat (2x1): a cushioned bench under a rain-streaked window ------------------------
const FRAME_Z: f64 = WALL + 0.08; // the window wall's front

pub fn window_seat() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(12);
    let still = new_parts(|| {
        // the bench: a box on a recessed dark plinth, two panelled drawers with brass bar pulls, a rounded nosing
        fbox!(
            f,
            "bench",
            1.84,
            0.325,
            0.58,
            (0, 0.2225, FRAME_Z + 0.29),
            M.bench,
            bevel = 0.012,
            bevel_segments = 2
        );
        fbox!(f, "kick", 1.8, 0.06, 0.54, (0, 0.03, FRAME_Z + 0.25), M.ink);
        fsweep!(
            f,
            "bench_nose",
            &[
                (-0.92, 0.372, FRAME_Z + 0.583),
                (0.92, 0.372, FRAME_Z + 0.583)
            ],
            0.012,
            M.frame,
            verts = 10
        );
        for x in [-0.45, 0.45] {
            kit::shaker!(
                f,
                numbered("drawer"),
                0.86,
                0.24,
                (x, 0.215, FRAME_Z + 0.587),
                0.014,
                M.bench_front,
                rail = 0.05
            );
            kit::pull!(
                f,
                numbered("pull"),
                x,
                0.28,
                FRAME_Z + 0.594,
                0.14,
                M.brass,
                r = 0.006,
                out = 0.03
            );
        }
        // the window wall: a panel, the rainy pane behind a sash of four lights, a moulded casing, a nosed sill
        fbox!(
            f,
            "wall_panel",
            1.84,
            1.5,
            0.08,
            (0, 1.22, WALL + 0.04),
            M.frame,
            bevel = 0.01
        );
        fbox!(
            f,
            "pane",
            1.5,
            1.04,
            0.01,
            (0, 1.24, FRAME_Z + 0.005),
            M.pane
        );
        let lights = [
            rounded_rect(0.69, 0.58, 0.004, 1),
            rounded_rect(0.69, 0.3, 0.004, 1),
        ];
        let mut holes: Vec<Vec<(f64, f64)>> = Vec::new();
        for s in [-1.0, 1.0] {
            for (k, dy) in [(0, -0.15), (1, 0.33)] {
                holes.push(
                    lights[k]
                        .iter()
                        .map(|&(u, v)| (u + s * 0.365, v + dy))
                        .collect(),
                );
            }
        }
        fextrude!(
            f,
            "sash",
            &rounded_rect(1.5, 1.04, 0.004, 1),
            0.03,
            (0, 1.24, FRAME_Z + 0.018),
            M.frame,
            holes = holes,
            bevel = 0.003,
            bevel_segments = 1
        );
        kit::moulding!(
            f,
            "casing",
            &[(0.81, 0.69), (0.81, 1.79), (-0.81, 1.79), (-0.81, 0.69)],
            &[
                (-0.045, 0.0),
                (-0.045, 0.026),
                (-0.02, 0.04),
                (0.012, 0.04),
                (0.012, 0.0),
            ],
            M.frame,
            at = FRAME_Z
        );
        fsweep!(
            f,
            "sill",
            &[(-0.84, 0.7, FRAME_Z + 0.06), (0.84, 0.7, FRAME_Z + 0.06)],
            0,
            M.frame,
            profile = rounded_rect(0.04, 0.12, 0.014, 2),
            up = (0, 0, 1)
        );
        fbox!(
            f,
            "sill_apron",
            1.5,
            0.05,
            0.016,
            (0, 0.655, FRAME_Z + 0.008),
            M.frame,
            bevel = 0.004,
            bevel_segments = 1
        );
        for i in 0..26 {
            // raindrops and runs on the glass
            let (x, y) = (rnd.uniform(-0.72, 0.72), rnd.uniform(0.76, 1.72));
            if x.abs() < 0.03 {
                continue;
            }
            if i % 3 == 0 {
                let l = rnd.uniform(0.06, 0.16);
                fsweep!(
                    f,
                    numbered("run"),
                    &[
                        (x, y, FRAME_Z + 0.011),
                        (x + 0.004, y - l * 0.5, FRAME_Z + 0.011),
                        (x + 0.001, y - l, FRAME_Z + 0.011)
                    ],
                    0.003,
                    M.drop,
                    verts = 4,
                    resolution = 2,
                    scale = [1.0, 0.7, 1.2]
                );
            }
            fsphere!(
                f,
                numbered("drop"),
                0.012,
                (x, y, FRAME_Z + 0.011),
                M.drop,
                scale = (0.8, 0.3, 1.1),
                segments = 6,
                rings = 4
            );
        }
        // on the sill: two candles on dishes and a mug
        candle(
            f,
            -0.7,
            0.72,
            FRAME_Z + 0.065,
            0.12,
            0.025,
            true,
            true,
            2,
            21,
        );
        candle(
            f,
            -0.6,
            0.72,
            FRAME_Z + 0.06,
            0.08,
            0.022,
            true,
            true,
            2,
            22,
        );
        kit::mug!(
            f,
            "mug",
            0.7,
            0.72,
            FRAME_Z + 0.065,
            0.038,
            0.09,
            M.mug,
            200.0
        );
        // pillows against the window wall
        soft!(
            f,
            "pillow_l",
            0.48,
            0.34,
            0.12,
            (-0.62, 0.62, FRAME_Z + 0.1),
            M.pillow_a,
            rot = (-10, 0, 8),
            round = 0.05,
            puff = 0.04,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
        soft!(
            f,
            "pillow_r",
            0.42,
            0.3,
            0.11,
            (0.66, 0.6, FRAME_Z + 0.1),
            M.pillow_b,
            rot = (-10, 0, -6),
            round = 0.045,
            puff = 0.035,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The seat cushion with its throw and a book left on it, joined with their origin on the cushion's underside:
    // `sink_in` presses them 2 cm while a resident broods or dozes on it.
    let zc = FRAME_Z + 0.3;
    let seat = new_parts(|| {
        let puff = 0.012;
        soft!(
            f,
            "seat_cushion",
            1.8,
            0.08 - puff,
            0.56,
            (0.0, 0.425 - puff / 2.0, zc),
            M.cushion,
            round = 0.03,
            puff = puff,
            piping = 0.006,
            flat = 3
        );
        kit::throw!(
            f,
            "throw",
            0.32,
            0.56,
            0.465,
            zc - 0.2,
            zc + 0.282,
            M.throw,
            0.14,
            seed = 2.0
        );
        book(
            f,
            "book",
            -0.25,
            0.465,
            zc + 0.08,
            0.16,
            0.03,
            0.22,
            M.plum,
            -20.0,
            None,
        );
    });
    let seat = hinged(join(&seat, "seat", None), f.at(0, 0.385, zc));
    let objs = settle!(centre = false);
    squash(
        seat,
        "sink_in",
        (1, 1, 0.78),
        Squash {
            seconds: 0.6,
            ..Squash::default()
        },
    );
    objs
}

// --- the moonstone soaking tub (2x2): a deep round stone tub, a candle ledge behind -------------------------
const TZ: f64 = 0.05; // the tub's middle (depth)

pub fn soaking_tub() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // the tub: up the outside from a rounded foot, over a rolled rim and down the inside to its floor
        kit::turned!(
            f,
            "tub",
            0.0,
            0.0,
            TZ,
            &[
                (0.0, 0.0),
                (0.64, 0.0),
                (0.675, 0.018),
                (0.735, 0.132),
                (0.75, 0.6),
                (0.742, 0.638),
                (0.722, 0.657),
                (0.7, 0.662),
                (0.674, 0.658),
                (0.652, 0.645),
                (0.636, 0.62),
                (0.57, 0.13),
                (0.53, 0.102),
                (0.0, 0.095),
            ],
            M.stone,
            segments = 56
        );
        torus!(
            "moon_band",
            0.749,
            0.012,
            at = f.at(0, 0.52, TZ),
            material = M.moon,
            major_segments = 56,
            minor_segments = 4
        );
        torus!(
            "moon_band_low",
            0.705,
            0.008,
            at = f.at(0, 0.06, TZ),
            material = M.moon,
            major_segments = 48,
            minor_segments = 3
        );
        // the ledge behind: dark wood with panelled ends, a bullnosed stone top; candles, towels, a vase with a fern
        fbox!(
            f,
            "ledge",
            1.84,
            0.49,
            0.3,
            (0, 0.245, WALL2 + 0.15),
            M.ledge,
            bevel = 0.01
        );
        for x in [-0.69, 0.69] {
            kit::shaker!(
                f,
                numbered("ledge_panel"),
                0.38,
                0.36,
                (x, 0.26, WALL2 + 0.307),
                0.016,
                M.ledge,
                rail = 0.05,
                raised = true
            );
        }
        fsweep!(
            f,
            "ledge_top",
            &[(-0.93, 0.51, WALL2 + 0.16), (0.93, 0.51, WALL2 + 0.16)],
            0,
            M.ledge_top,
            profile = rounded_rect(0.04, 0.32, 0.014, 2),
            up = (0, 0, 1)
        );
        for (i, x) in [-0.78, -0.66, -0.56, 0.6, 0.72].into_iter().enumerate() {
            candle(
                f,
                x,
                0.53,
                WALL2 + 0.17 + (i % 2) as f64 * 0.04,
                0.08 + (i % 3) as f64 * 0.05,
                0.03,
                true,
                false,
                2 + i % 2,
                30 + i as i64,
            );
        }
        soft!(
            f,
            "towel_a",
            0.36,
            0.07,
            0.24,
            (0.2, 0.567, WALL2 + 0.16),
            M.towel,
            round = 0.03,
            puff = 0.008,
            flat = 2
        );
        soft!(
            f,
            "towel_b",
            0.34,
            0.06,
            0.23,
            (0.2, 0.636, WALL2 + 0.16),
            M.mauve,
            round = 0.026,
            puff = 0.008,
            flat = 2
        );
        for (k, (y, w, d)) in [(0.567, 0.36, 0.24), (0.636, 0.34, 0.23)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                f,
                numbered("towel_band"),
                w + 0.004,
                0.012,
                0.03,
                (0.2, y, WALL2 + 0.16 + d / 2.0 - 0.035),
                [M.mauve, M.towel][k]
            );
        }
        let top = vase(f, -0.25, 0.53, WALL2 + 0.14, 1.3);
        for k in 0..7 {
            let a = (k * 360) as f64 / 7.0 + 10.0;
            let (c, s) = (cosd(a), sind(a));
            let l = 0.17 + 0.03 * (k % 3) as f64;
            let base = (-0.25, top - 0.02, WALL2 + 0.14);
            let sd = s * if s > 0.0 { 0.7 } else { 0.4 }; // (towards the wall, less: the ledge's back)
            let pts = [
                base,
                (-0.25 + 0.04 * c, top + 0.09, WALL2 + 0.14 + 0.03 * sd),
                (
                    -0.25 + l * 0.6 * c,
                    top + 0.128,
                    WALL2 + 0.14 + l * 0.6 * sd,
                ),
                (-0.25 + l * c, top + 0.04, WALL2 + 0.14 + l * sd),
            ];
            fsweep!(
                f,
                numbered("frond"),
                &pts,
                0,
                M.leaf,
                profile = rounded_rect(0.003, 0.05, 0.001, 1),
                up = (0, 0, 1),
                resolution = 2,
                scale = [0.15, 0.8, 1.0, 0.1]
            );
        }
        // the bath mat in front, a fringe at its ends
        soft!(
            f,
            "mat",
            0.8,
            0.02,
            0.4,
            (0.1, 0.01, 0.72),
            M.mauve,
            round = 0.008,
            round_segments = 1,
            flat = 1
        );
        for s in [-1.0, 1.0] {
            for k in 0..9 {
                fbox!(
                    f,
                    numbered("tassel"),
                    0.02,
                    0.012,
                    0.012,
                    (0.1 + s * 0.41, 0.006, 0.56 + 0.04 * k as f64),
                    M.mauve
                );
            }
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The water: the tub's inside up to 0.52, a little foam on it, its origin under the tub's floor and flat
    // there at rest; `fill` raises it while a resident soaks.
    let water = new_parts(|| {
        kit::turned!(
            f,
            "water",
            0.0,
            0.095,
            TZ,
            &[(0.0, 0.0), (0.565, 0.0), (0.615, 0.425), (0.0, 0.425)],
            M.water,
            segments = 48
        );
        for (k, (x, z, r)) in [
            (0.25, 0.25, 0.1),
            (0.32, 0.12, 0.07),
            (-0.3, 0.05, 0.09),
            (-0.2, 0.3, 0.06),
        ]
        .into_iter()
        .enumerate()
        {
            boulder!(
                numbered("foam"),
                r,
                at = f.at(x, 0.515, TZ + z),
                material = M.foam,
                scale = (1.3, 1.0, 0.16),
                seed = k as f64 * 2.3,
                noise = 0.3,
                subdivisions = 2
            );
        }
    });
    let water = join(&water, "water", Some(f.at(0, 0.09, TZ)));
    water.edit(|o| o.scale.z = 0.001);
    let objs = settle!(centre = false);
    grow(
        water,
        "fill",
        Grow {
            seconds: 1.5,
            ..Grow::default()
        },
    );
    objs
}
