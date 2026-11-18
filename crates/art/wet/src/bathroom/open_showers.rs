//! The open showers: the walk-in shower behind one glass screen, and the rain shower in its stone corner.

use super::M;
use super::common::{bends, bottle, tube, wall_rose};
use super::shower_parts::{frame, grate, head, post, rain};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsweep, grow, soft};

pub fn shower_walk_in() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        soft!(
            f,
            "floor",
            0.92,
            0.04,
            0.92,
            (0, 0.02, 0),
            M.slate,
            round = 0.008,
            round_segments = 1
        );
        grate(
            &f,
            "drain",
            0,
            0.04,
            0.2,
            0.3,
            0.06,
            M.fittings_dark,
            12,
            false,
        );
        fbox!(
            f,
            "wall",
            0.92,
            2.2,
            0.04,
            (0, 1.14, -0.44),
            M.slate_wall,
            bevel = 0.004
        );
        for y in [0.6, 1.15, 1.7] {
            fbox!(
                f,
                format!("joint{}", float_repr(y)),
                0.92,
                0.006,
                0.004,
                (0, y, -0.419),
                M.groove
            );
        }
        fbox!(
            f,
            "joint_v",
            0.006,
            2.2,
            0.004,
            (0.15, 1.14, -0.419),
            M.groove
        );
        // one glass screen on the right in a wall channel and a floor channel, a post at its end, and a stay bar from
        // its top to the wall
        fbox!(
            f,
            "glass_screen",
            0.01,
            2.0,
            0.6,
            (0.42, 1.04, -0.11),
            M.glass
        );
        fbox!(
            f,
            "channel",
            0.025,
            2.0,
            0.02,
            (0.42, 1.04, -0.41),
            M.frame_dark,
            bevel = 0.003
        );
        fbox!(
            f,
            "floor_channel",
            0.025,
            0.02,
            0.6,
            (0.42, 0.05, -0.11),
            M.frame_dark,
            bevel = 0.003
        );
        post(&f, "post", 0.42, 0.2, 0.04, 2.04, M.frame_dark, 0.022);
        tube(
            &f,
            "bar",
            &[(0.42, 2.05, 0.2), (0.42, 2.05, -0.38), (0.42, 2.05, -0.415)],
            0.01,
            M.frame_dark,
            8,
            1,
            1.0,
        );
        wall_rose(
            &f,
            "bar_rose",
            0.42,
            2.05,
            -0.42,
            0.018,
            M.frame_dark,
            0.01,
            12,
        );
        // a riser up the wall, bent over into an arm out over the middle, and the wide rain head
        tube(
            &f,
            "riser",
            &bends(
                &[(0.0, 1.74, -0.4), (0.0, 2.245, -0.4), (0.0, 2.245, -0.08)],
                0.045,
                5,
            ),
            0.015,
            M.fittings_dark,
            10,
            1,
            1.0,
        );
        wall_rose(
            &f,
            "riser_rose",
            0,
            1.74,
            -0.42,
            0.03,
            M.fittings_dark,
            0.012,
            14,
        );
        head(&f, "head", 0, 2.208, -0.08, 0.17, M.fittings_dark, 0.02);
        soft!(
            f,
            "mixer",
            0.08,
            0.16,
            0.012,
            (-0.25, 1.1, -0.414),
            M.fittings_dark,
            round = 0.004,
            round_segments = 1
        );
        for (k, y) in [1.14, 1.06].into_iter().enumerate() {
            fcyl!(
                f,
                format!("mixer_knob{k}"),
                0.02,
                0.03,
                (-0.25, y, -0.393),
                M.fittings_dark,
                rot = (90, 0, 0),
                verts = 14,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        // a stone ledge with two bottles
        soft!(
            f,
            "ledge",
            0.4,
            0.04,
            0.12,
            (-0.12, 1.3, -0.36),
            M.slate_wall,
            round = 0.006,
            round_segments = 1
        );
        bottle(
            &f,
            "bottle0",
            (-0.22, 1.32, -0.36),
            0.025,
            0.16,
            M.bottle,
            M.fittings_dark,
            12,
            true,
        );
        bottle(
            &f,
            "bottle1",
            (-0.12, 1.32, -0.36),
            0.025,
            0.12,
            M.sage,
            M.bottle,
            12,
            false,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let rain = rain(&f, (0.0, 2.207, -0.08), 0.14, 0.04, 3, None, None);
    settle_all();
    grow!(rain, "water_on", seconds = 0.5);
    meshes()
}

pub fn shower_rain() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let still = new_parts(|| {
        // 0.93 deep: its back on the wall's face, its front on the tile's edge (the Model's centre)
        soft!(
            f,
            "tray",
            0.94,
            0.08,
            0.93,
            (0, 0.04, 0.005),
            M.tray_stone,
            round = 0.01,
            round_segments = 1
        );
        grate(
            &f,
            "drain",
            0.15,
            0.08,
            0.15,
            0.1,
            0.1,
            M.fittings_dark,
            4,
            false,
        );
        fbox!(
            f,
            "wall_back",
            0.94,
            2.1,
            0.05,
            (0, 1.13, -0.435),
            M.wall_stone,
            bevel = 0.008
        );
        fbox!(
            f,
            "wall_left",
            0.06,
            2.0,
            0.88,
            (-0.44, 1.08, 0.03),
            M.wall_stone,
            bevel = 0.008
        );
        // a niche in the left wall with two bottles
        fbox!(
            f,
            "niche_back",
            0.004,
            0.3,
            0.36,
            (-0.408, 1.25, -0.05),
            M.groove
        );
        for (k, y) in [1.1, 1.4].into_iter().enumerate() {
            fbox!(
                f,
                format!("niche_sill{k}"),
                0.03,
                0.012,
                0.38,
                (-0.395, y, -0.05),
                M.wall_stone,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        bottle(
            &f,
            "bottle0",
            (-0.385, 1.106, -0.12),
            0.024,
            0.15,
            M.bottle,
            M.fittings_dark,
            12,
            true,
        );
        bottle(
            &f,
            "bottle1",
            (-0.385, 1.106, -0.04),
            0.022,
            0.12,
            M.sage,
            M.bottle,
            12,
            false,
        );
        // glass along the right and the left half of the front: the way in is the front's right half
        fbox!(
            f,
            "glass_side",
            0.012,
            1.9,
            0.86,
            (0.45, 1.03, 0.0),
            M.glass
        );
        fbox!(
            f,
            "glass_front",
            0.46,
            1.9,
            0.012,
            (-0.18, 1.03, 0.45),
            M.glass
        );
        // the frame: the corner post bent into the top rails in one piece, a foot channel
        frame(
            &f,
            "frame",
            &[(0.45, 0.08, 0.45), (0.45, 2.02, 0.45), (0.45, 2.02, -0.41)],
            M.frame_dark,
            0.02,
            0.012,
            (1, 0, 0).into(),
        );
        fsweep!(
            f,
            "rail_front",
            &[(0.44, 2.02, 0.45), (-0.41, 2.02, 0.45)],
            0.0,
            M.frame_dark,
            profile = rounded_rect(0.02, 0.02, 0.004, 1)
        );
        fbox!(
            f,
            "foot",
            0.03,
            0.012,
            0.88,
            (0.45, 0.086, 0.0),
            M.frame_dark,
            bevel = 0.003
        );
        // the square rain head on an arm from the back wall, and a mixer plate with its lever
        wall_rose(
            &f,
            "arm_rose",
            0,
            2.12,
            -0.41,
            0.025,
            M.fittings_dark,
            0.012,
            14,
        );
        tube(
            &f,
            "arm",
            &bends(
                &[(0.0, 2.12, -0.405), (0.0, 2.12, -0.03), (0.0, 2.17, -0.03)],
                0.03,
                4,
            ),
            0.0125,
            M.fittings_dark,
            10,
            1,
            1.0,
        );
        soft!(
            f,
            "head",
            0.3,
            0.02,
            0.3,
            (0, 2.17, 0),
            M.fittings_dark,
            round = 0.008,
            round_segments = 2
        );
        for i in 0..4 {
            fbox!(
                f,
                format!("nozzle_row{i}"),
                0.24,
                0.002,
                0.006,
                (0, 2.159, -0.09 + 0.06 * i as f64),
                M.dark
            );
        }
        soft!(
            f,
            "mixer",
            0.06,
            0.18,
            0.012,
            (0.3, 1.1, -0.404),
            M.fittings_dark,
            round = 0.004,
            round_segments = 1
        );
        tube(
            &f,
            "mixer_lever",
            &[(0.3, 1.14, -0.4), (0.3, 1.14, -0.37), (0.3, 1.1, -0.362)],
            0.008,
            M.fittings_dark,
            8,
            2,
            1.0,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let rain = rain(&f, (0.0, 2.158, 0.0), 0.0, 0.08, 2, Some((5, 0.055)), None);
    settle_all();
    grow!(rain, "water_on", seconds = 0.5);
    meshes()
}
