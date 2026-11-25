//! The bathroom sinks: the base sink in its cabinet and the pedestal. Each is one still body plus its stream of
//! water (`water_on`).

use super::M;
use super::common::{LATHE, Lathe, Ring, bottle, drain, gooseneck, lathe, pull, shaker};
use crate::common::water;
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, grow, soft};

// --- the base sink: a shaker cabinet, a worktop with an undermount basin and a gooseneck tap -------------
pub fn sink() -> Vec<Obj> {
    let f = GameFrame::new(0.145);
    let (w, h, d, t) = (0.86, 0.92, 0.61, 0.04);
    let (z0, zt) = (-0.455, -0.455 + d); // worktop back and front
    let zc = zt - 0.045; // carcass front
    let (hole, hd) = (0.48, 0.35);
    let (hz0, hz1) = (z0 + 0.17, zt - 0.09); // the basin opening
    let bz = (hz0 + hz1) / 2.0;
    let by = h - 0.17; // the basin floor
    let mut spout = (0, 0.0, 0.0);
    let still = new_parts(|| {
        // the carcass on a toe kick set back under it
        fbox!(
            f,
            "plinth",
            w - 0.04,
            0.09,
            d - 0.10,
            (0, 0.045, z0 + (d - 0.10) / 2.0),
            M.trim,
            bevel = 0.006
        );
        // (its hidden top 3 cm under the basin's floor: `cargo review` tells a worktop from what stands on it by the
        // faces under it, and a face within 1 cm of the basin's floor made the worktop look like something on it)
        let cy = by - 0.03;
        fbox!(
            f,
            "carcass",
            w - 0.01,
            cy - 0.09,
            zc - z0,
            (0, (0.09 + cy) / 2.0, (z0 + zc) / 2.0),
            M.body,
            bevel = 0.01
        );
        // the cabinet's sides in one piece up to the worktop, and its shoulders round the basin
        for s in [-1, 1] {
            fbox!(
                f,
                format!("side{s}"),
                0.018,
                h - t - 0.09,
                zc - z0,
                (
                    s as f64 * (w / 2.0 - 0.013),
                    (0.09 + h - t) / 2.0,
                    (z0 + zc) / 2.0
                ),
                M.body,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        let sw = (w - hole) / 2.0;
        let sx = (hole / 2.0 + w / 2.0) / 2.0;
        fbox!(
            f,
            "shoulder_back",
            w - 0.01,
            h - t - cy,
            hz0 - z0,
            (0, (cy + h - t) / 2.0, (z0 + hz0) / 2.0),
            M.body
        );
        fbox!(
            f,
            "shoulder_front",
            w - 0.01,
            h - t - cy,
            zc - hz1,
            (0, (cy + h - t) / 2.0, (hz1 + zc) / 2.0),
            M.body
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("shoulder{s}"),
                sw - 0.005,
                h - t - cy,
                hz1 - hz0,
                (
                    s as f64 * (sx - 0.0025),
                    (cy + h - t) / 2.0,
                    (hz0 + hz1) / 2.0
                ),
                M.body
            );
        }
        // the worktop in one slab, its opening's corners rounded, its edges eased
        let opening: Vec<(f64, f64)> = rounded_rect(hole, hd, 0.035, 3)
            .into_iter()
            .map(|(u, v)| (u, v + bz - (z0 + zt) / 2.0))
            .collect();
        fextrude!(
            f,
            "top",
            &rounded_rect(w, d, 0.006, 1),
            t,
            (0, h - t / 2.0, (z0 + zt) / 2.0),
            M.stone,
            plane = Facing::Top,
            holes = vec![opening],
            bevel = 0.005,
            bevel_segments = 2
        );
        // the undermount basin: one steel shell, its rim under the worktop, its inside rounding into the floor
        lathe(
            &f,
            "basin",
            0.0,
            bz,
            &[
                (by - 0.012, 0.236, 0.166, 6.0),
                (h - t - 0.001, 0.262, 0.192, 9.0),
                (h - t - 0.001, 0.25, 0.18, 9.0),
                (by + 0.04, 0.246, 0.176, 8.0),
                (by + 0.01, 0.232, 0.162, 6.0),
                (by, 0.214, 0.144, 5.0),
            ],
            M.metal,
            Lathe {
                verts: 40,
                angle: 50.0,
                ..LATHE
            },
        );
        drain(&f, 0, by, bz + 0.03, 0.036, None);
        // a false drawer front under the worktop and two shaker doors under it, bar pulls on each
        let dw = (w - 0.03) / 2.0;
        let (dy0, dy1) = (0.11, 0.71);
        soft!(
            f,
            "false_front",
            w - 0.02,
            0.15,
            0.02,
            (0, 0.795, zc + 0.01),
            M.body,
            round = 0.004,
            round_segments = 1
        );
        pull(
            &f,
            "pull_top",
            0.0,
            0.795,
            zc + 0.02,
            0.16,
            M.chrome,
            false,
            0.0065,
            0.024,
        );
        for s in [-1, 1] {
            let cx = s as f64 * (dw / 2.0 + 0.005);
            shaker(
                &f,
                &format!("door{s}"),
                dw,
                dy1 - dy0,
                cx,
                (dy0 + dy1) / 2.0,
                zc,
                M.body,
                0.02,
                0.06,
            );
            pull(
                &f,
                format!("handle{s}"),
                cx - s as f64 * (dw / 2.0 - 0.045),
                dy1 - 0.11,
                zc + 0.02,
                0.14,
                M.chrome,
                true,
                0.0065,
                0.024,
            );
        }
        // the tap, and a soap pump beside it
        spout = gooseneck(
            &f,
            "tap",
            0,
            h,
            -0.37,
            -0.23,
            h + 0.32,
            M.chrome,
            0.02,
            None,
            true,
        );
        bottle(
            &f,
            "soap",
            (0.25, h, -0.36),
            0.03,
            0.17,
            M.teal,
            M.chrome,
            12,
            true,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let water = water(&f, spout, by, M.water);
    settle_all();
    grow!(water, "water_on", seconds = 0.4);
    meshes()
}

// --- pedestal sink: a tapered column under a basin with a tap deck at its back -----------------------------
pub fn sink_pedestal() -> Vec<Obj> {
    let f = GameFrame::new(0.222);
    let z = -0.22;
    let (rim, floor_y) = (0.88, 0.735);
    let bz = z + 0.03; // the bowl sits forward of the deck
    let mut spout = (0, 0.0, 0.0);
    let still = new_parts(|| {
        // the column: an oval that waists in and flares into its foot and under the basin
        lathe(
            &f,
            "column",
            0.0,
            z - 0.02,
            &[
                (0.0, 0.122, 0.1),
                (0.012, 0.122, 0.1),
                (0.03, 0.112, 0.092),
                (0.08, 0.098, 0.08),
                (0.3, 0.084, 0.069),
                (0.52, 0.08, 0.066),
                (0.62, 0.095, 0.078),
                (0.69, 0.14, 0.115),
            ],
            M.body,
            Lathe {
                verts: 28,
                angle: 60.0,
                ..LATHE
            },
        );
        // the basin in one shell: up the outside, over a rolled rim, across the flat deck at the back and down
        // the bowl (set forward) to its floor
        lathe(
            &f,
            "basin",
            0.0,
            z,
            &[
                Ring::from((0.66, 0.2, 0.15, 2.4)),
                Ring::from((0.7, 0.25, 0.195, 2.6)),
                Ring::from((0.8, 0.274, 0.222, 2.8)),
                Ring::from((0.865, 0.28, 0.229, 3.0, 0.233)),
                Ring::from((rim, 0.276, 0.225, 3.0, 0.229)),
                Ring::from((rim, 0.255, 0.2, 3.0, 0.21)),
                Ring::from((rim, 0.236, 0.16, 2.4, 0.16, bz - z)),
                Ring::from((rim - 0.02, 0.228, 0.152, 2.2, 0.15, bz - z)),
                Ring::from((0.8, 0.19, 0.125, 2.1, 0.12, bz - z)),
                Ring::from((0.755, 0.12, 0.08, 2.0, 0.08, bz - z)),
                Ring::from((floor_y, 0.06, 0.042, 2.0, 0.042, bz - z)),
            ],
            M.porcelain,
            Lathe {
                verts: 36,
                angle: 50.0,
                ..LATHE
            },
        );
        drain(&f, 0, floor_y, bz, 0.028, None);
        // an overflow slot in the bowl's front wall
        fbox!(
            f,
            "overflow",
            0.05,
            0.012,
            0.01,
            (0, 0.85, bz + 0.147),
            M.dark,
            bevel = 0.004,
            bevel_segments = 1,
            rot = (-25, 0, 0)
        );
        spout = gooseneck(
            &f,
            "tap",
            0,
            rim,
            -0.405,
            -0.28,
            1.04,
            M.chrome,
            0.018,
            Some(1.0),
            true,
        );
        // a soap on its dish, a tooth mug with a brush
        lathe(
            &f,
            "dish",
            0.15,
            -0.408,
            &[
                (rim, 0.035, 0.026),
                (rim + 0.008, 0.045, 0.034),
                (rim + 0.012, 0.043, 0.032),
                (rim + 0.006, 0.036, 0.026),
            ],
            M.porcelain,
            Lathe {
                verts: 16,
                square: 3.0,
                ..LATHE
            },
        );
        soft!(
            f,
            "soap",
            0.06,
            0.02,
            0.04,
            (0.15, rim + 0.017, -0.408),
            M.teal,
            round = 0.008,
            round_segments = 2
        );
        lathe(
            &f,
            "cup",
            -0.15,
            -0.41,
            &[
                (rim, 0.028, 0.028),
                (rim + 0.004, 0.032, 0.032),
                (rim + 0.09, 0.038, 0.038),
                (rim + 0.09, 0.034, 0.034),
                (rim + 0.02, 0.029, 0.029),
            ],
            M.porcelain,
            Lathe { verts: 16, ..LATHE },
        );
        kit::tube!(
            f,
            "brush",
            &[
                (-0.15, rim + 0.03, -0.41),
                (-0.143, rim + 0.12, -0.405),
                (-0.137, rim + 0.16, -0.4),
            ],
            0.006,
            M.teal,
            verts = 6,
            round = 1
        );
        fbox!(
            f,
            "brush_head",
            0.01,
            0.03,
            0.014,
            (-0.136, rim + 0.17, -0.396),
            M.paper,
            bevel = 0.003,
            bevel_segments = 1
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let water = water(&f, spout, floor_y + 0.01, M.water);
    settle_all();
    grow!(water, "water_on", seconds = 0.4);
    meshes()
}
