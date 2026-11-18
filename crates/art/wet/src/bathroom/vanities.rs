//! The basins on stands: the double vanity, the vessel sink and the console sink. Each is one still body plus
//! its stream of water (`water_on`).

use super::M;
use super::common::{
    LATHE, Lathe, Ring, bends, bowl_shell, drain, gooseneck, lathe, pull, shaker, tube,
};
use crate::common::water;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsweep, grow, soft};

// --- double vanity: a two-basin cabinet ----------------------------------------------------------------
// No mirrors: the player hangs the ones they like above it (the wall mirror, the mirror cabinet).

pub fn vanity_double() -> Vec<Obj> {
    let f = GameFrame::new(0.152);
    let mut tips = Vec::new();
    let still = new_parts(|| {
        // the worktop at 0.76 m, so the bowls standing on it have their rims at 0.88 (a basin's 0.76-0.91)
        fbox!(
            f,
            "plinth",
            1.82,
            0.06,
            0.46,
            (0, 0.03, -0.19),
            M.trim,
            bevel = 0.006
        );
        fbox!(
            f,
            "cabinet",
            1.86,
            0.66,
            0.52,
            (0, 0.39, -0.17),
            M.body,
            bevel = 0.006,
            bevel_segments = 1
        );
        soft!(
            f,
            "top",
            1.9,
            0.04,
            0.56,
            (0, 0.74, -0.15),
            M.stone,
            round = 0.008,
            round_segments = 2
        );
        fbox!(
            f,
            "upstand",
            1.9,
            0.08,
            0.02,
            (0, 0.8, -0.42),
            M.stone,
            bevel = 0.004,
            bevel_segments = 1
        );
        // two drawers across the top, four shaker doors under them, bar pulls on each
        for s in [-1, 1] {
            soft!(
                f,
                format!("drawer{s}"),
                0.915,
                0.085,
                0.02,
                (s as f64 * 0.4625, 0.672, 0.1),
                M.body,
                round = 0.004,
                round_segments = 1
            );
            pull(
                &f,
                format!("drawer_pull{s}"),
                s as f64 * 0.4625,
                0.672,
                0.11,
                0.2,
                M.chrome,
                false,
                0.0065,
                0.018,
            );
        }
        for (i, x) in [-0.6975, -0.2325, 0.2325, 0.6975].into_iter().enumerate() {
            shaker(
                &f,
                &format!("door{i}"),
                0.455,
                0.53,
                x,
                0.36,
                0.09,
                M.body,
                0.02,
                0.06,
            );
            pull(
                &f,
                format!("handle{i}"),
                x + (if i % 2 == 0 { 0.18 } else { -0.18 }),
                0.535,
                0.11,
                0.13,
                M.chrome,
                true,
                0.0065,
                0.018,
            );
        }
        for s in [-1, 1] {
            let x = s as f64 * 0.48;
            bowl_shell(
                &f,
                &format!("basin{s}"),
                x,
                -0.12,
                0.76,
                0.88,
                0.5,
                0.38,
                M.porcelain,
                0.016,
                0.46,
                0.8,
                32,
                2.0,
            );
            drain(&f, x, 0.776, -0.12, 0.026, None);
            tips.push(gooseneck(
                &f,
                &format!("tap{s}"),
                x,
                0.76,
                -0.35,
                -0.22,
                0.98,
                M.chrome,
                0.02,
                Some(0.92),
                true,
            ));
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let waters: Vec<Obj> = tips
        .iter()
        .map(|&tip| water(&f, tip, 0.785, M.water))
        .collect();
    settle_all();
    for water in waters {
        grow!(water, "water_on", seconds = 0.4);
    }
    meshes()
}

// --- vessel sink: a bowl on a painted vanity on a wooden stand, towels on its shelf ------------------------
pub fn sink_vessel() -> Vec<Obj> {
    let f = GameFrame::new(0.2);
    let mut spout = (0, 0.0, 0.0);
    let still = new_parts(|| {
        // the worktop at 0.76 m, so the bowl on it has its rim at 0.88 (a basin's 0.76-0.91)
        fbox!(
            f,
            "vanity",
            0.8,
            0.42,
            0.48,
            (0, 0.51, -0.22),
            M.body,
            bevel = 0.006,
            bevel_segments = 1
        );
        soft!(
            f,
            "drawer",
            0.78,
            0.38,
            0.022,
            (0, 0.51, 0.03),
            M.body,
            round = 0.006,
            round_segments = 2
        );
        pull(
            &f,
            "pull",
            0.0,
            0.64,
            0.041,
            0.32,
            M.blackmetal,
            false,
            0.006,
            0.024,
        );
        soft!(
            f,
            "top",
            0.84,
            0.04,
            0.54,
            (0, 0.74, -0.2),
            M.stone,
            round = 0.008,
            round_segments = 2
        );
        // the stand: four tapered legs from the floor up to the vanity, a slatted shelf between them
        for x in [-0.38, 0.38] {
            for z in [-0.43, 0.01] {
                tube(
                    &f,
                    format!("leg{}{}", float_repr(x), float_repr(z)),
                    &[(x, 0.0, z), (x, 0.305, z)],
                    0.016,
                    M.wood,
                    8,
                    1,
                    [0.7, 1.0],
                );
            }
            fbox!(
                f,
                format!("rail{}", float_repr(x)),
                0.03,
                0.03,
                0.44,
                (x, 0.18, -0.21),
                M.wood,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for k in 0..5 {
            fbox!(
                f,
                format!("slat{k}"),
                0.73,
                0.02,
                0.072,
                (0, 0.205, -0.385 + 0.088 * k as f64),
                M.wood,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        bowl_shell(
            &f,
            "bowl",
            0,
            -0.18,
            0.76,
            0.88,
            0.44,
            0.36,
            M.porcelain,
            0.014,
            0.45,
            1.0,
            32,
            2.0,
        );
        drain(&f, 0, 0.774, -0.18, 0.026, Some(M.blackmetal));
        spout = gooseneck(
            &f,
            "tap",
            0,
            0.76,
            -0.42,
            -0.26,
            1.04,
            M.blackmetal,
            0.02,
            Some(0.98),
            true,
        );
        // two folded towels on the shelf, each two soft layers
        for (i, x) in [-0.17, 0.17].into_iter().enumerate() {
            for k in 0..2 {
                soft!(
                    f,
                    format!("towel{i}_{k}"),
                    0.3,
                    0.036,
                    0.24,
                    (x, 0.233 + 0.036 * k as f64, -0.13 + 0.008 * k as f64),
                    M.towel,
                    round = 0.017,
                    puff = (0, 0.006, 0),
                    round_segments = 2,
                    flat = 2
                );
            }
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let water = water(&f, spout, 0.785, M.water);
    settle_all();
    grow!(water, "water_on", seconds = 0.4);
    meshes()
}

// --- console sink: a basin on a brass frame, a towel over the bar between its legs -----------------------
pub fn sink_console() -> Vec<Obj> {
    let f = GameFrame::new(0.2);
    let z = -0.21;
    let mut spout = (0, 0.0, 0.0);
    let still = new_parts(|| {
        // the basin in one shell: a rounded slab, a flat deck round the bowl, the bowl set forward
        let bz = z + 0.03;
        lathe(
            &f,
            "basin",
            0.0,
            z,
            &[
                Ring::from((0.765, 0.29, 0.22, 7.0)),
                Ring::from((0.77, 0.297, 0.227, 7.0)),
                Ring::from((0.87, 0.3, 0.23, 7.0)),
                Ring::from((0.88, 0.296, 0.226, 7.0)),
                Ring::from((0.88, 0.25, 0.168, 5.0, 0.168, bz - z)),
                Ring::from((0.875, 0.245, 0.163, 5.0, 0.163, bz - z)),
                Ring::from((0.83, 0.225, 0.145, 4.0, 0.145, bz - z)),
                Ring::from((0.805, 0.17, 0.1, 3.0, 0.1, bz - z)),
                Ring::from((0.8, 0.1, 0.06, 2.4, 0.06, bz - z)),
            ],
            M.body,
            Lathe {
                verts: 36,
                angle: 50.0,
                ..LATHE
            },
        );
        drain(&f, 0, 0.8, bz, 0.026, Some(M.brass));
        // the frame: each side a leg from a foot up and back under the basin to the wall, and a towel bar
        for s in [-1, 1] {
            let x = s as f64 * 0.27;
            tube(
                &f,
                format!("leg{s}"),
                &bends(
                    &[(x, 0.012, 0.0), (x, 0.752, 0.0), (x, 0.752, -0.41)],
                    0.05,
                    5,
                ),
                0.016,
                M.brass,
                10,
                1,
                1.0,
            );
            lathe(
                &f,
                format!("foot{s}"),
                x,
                0.0,
                &[
                    (0.0, 0.024, 0.024),
                    (0.012, 0.022, 0.022),
                    (0.03, 0.017, 0.017),
                ],
                M.brass,
                Lathe { verts: 12, ..LATHE },
            );
            fcyl!(
                f,
                format!("wall_plate{s}"),
                0.03,
                0.012,
                (x, 0.752, -0.42),
                M.brass,
                rot = (90, 0, 0),
                verts = 14
            );
            fcyl!(
                f,
                format!("bar_collar{s}"),
                0.02,
                0.02,
                (s as f64 * 0.255, 0.6, 0.0),
                M.brass,
                rot = (0, 90, 0),
                verts = 12
            );
        }
        fcyl!(
            f,
            "bar",
            0.011,
            0.5,
            (0, 0.6, 0),
            M.brass,
            rot = (0, 90, 0),
            verts = 10
        );
        // the towel hangs over the bar, longer at the front
        fsweep!(
            f,
            "towel",
            &[
                (0, 0.36, 0.036),
                (0, 0.5, 0.03),
                (0, 0.585, 0.021),
                (0, 0.617, 0.0),
                (0, 0.585, -0.022),
                (0, 0.5, -0.03),
                (0, 0.42, -0.036)
            ],
            0.0,
            M.fabric,
            profile = rounded_rect(0.4, 0.012, 0.005, 2),
            up = (1, 0, 0),
            resolution = 2
        );
        spout = gooseneck(
            &f,
            "tap",
            0,
            0.88,
            -0.4,
            -0.26,
            1.04,
            M.brass,
            0.02,
            Some(0.98),
            true,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let water = water(&f, spout, 0.81, M.water);
    settle_all();
    grow!(water, "water_on", seconds = 0.4);
    meshes()
}
