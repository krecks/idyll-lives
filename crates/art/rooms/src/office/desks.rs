//! O1: the desks. The computer, writing, laptop and gaming desks have their own chair: a top at the front of the
//! tile, a frame, the chair behind; the catalog2 desks (simple, corner, standing, executive) stand alone.

use super::chairs::desk_chair;
use super::common::{bar_pull, drawer_box, front, top, turned};
use super::desk_things::{
    cactus, desk_frame, desk_lamp, keyboard, laptop_keys, monitor, mouse, mug, notebook, pen,
};
use super::{CHAIR_Z, F, FT, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::Axis;
use artkit::motion::{Spin, spin};
use artkit::parts::{aside, hinged, join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, join, kit, revolve, settle, slide, swing, torus,
};

// --- the desks with their own chair: a top at the front of the tile, a frame, the chair behind -----------------
pub fn computer_desk() -> Vec<Obj> {
    let still = new_parts(|| {
        top(1.54, 0.38, 0.26, M.oak, FT);
        // the sides: a sled frame each, bent from one square tube round its foot, legs and top rail
        let sq = rounded_rect(0.04, 0.04, 0.007, 1);
        for s in [-1, 1] {
            let x = f64::from(s) * 0.72;
            let lp = kit::fillet!(
                &[
                    (x, 0.02, 0.275),
                    (x, 0.02, 0.43),
                    (x, 0.7, 0.43),
                    (x, 0.7, 0.12),
                    (x, 0.02, 0.12),
                    (x, 0.02, 0.275),
                ],
                0.03,
                3
            );
            fsweep!(
                F,
                format!("sled{s}"),
                &lp[..lp.len() - 1],
                0.0,
                M.frame,
                profile = &sq,
                up = (1, 0, 0),
                closed = true,
                angle = 50
            );
            for z in [0.12, 0.43] {
                fbox!(
                    F,
                    format!("glide{s}{}", float_repr(z)),
                    0.04,
                    0.006,
                    0.04,
                    (x, 0.003, z),
                    M.plastic,
                    bevel = 0.002,
                    bevel_segments = 1
                );
            }
        }
        fbox!(
            F,
            "rail_back",
            1.4,
            0.04,
            0.02,
            (0, 0.7, 0.435),
            M.frame,
            bevel = 0.005,
            bevel_segments = 1
        );
        // the drawer pedestal hanging under the left, its drawers to the resident, a reveal between them
        fbox!(
            F,
            "pedestal",
            0.36,
            0.42,
            0.34,
            (-0.52, 0.48, 0.27),
            M.drawers,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(
            F,
            "pedestal_reveal",
            0.33,
            0.39,
            0.002,
            (-0.52, 0.48, 0.0995),
            M.reveal
        );
        for (k, (y, h)) in [(0.6, 0.12), (0.42, 0.2)].into_iter().enumerate() {
            front(
                &format!("pedestal_front{k}"),
                0.34,
                h - 0.006,
                -0.52,
                y,
                0.092,
                M.drawers,
                0.014,
            );
            bar_pull(
                &format!("pedestal_pull{k}"),
                -0.52,
                y + h / 2.0 - 0.035,
                0.085,
                0.11,
                M.frame,
                -0.022,
                0.0045,
            );
        }
        // the runners the keyboard tray hangs from
        for x in [-0.18, 0.5] {
            fbox!(
                F,
                format!("runner{}", float_repr(x)),
                0.015,
                0.055,
                0.32,
                (x, 0.69, 0.25),
                M.frame,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    });
    join!(&still, "computerDesk", origin = (0, 0, 0));
    // the monitor on it, facing the resident, the mug, lamp, cactus, notebook and pen, and the chair drawn at
    // it: not part of the desk's size (the monitor stands 1.21 m up, the desk's own top 0.75)
    let carried = new_parts(|| {
        monitor(
            "monitor",
            0.05,
            0.4,
            0.62,
            0.37,
            1.03,
            M.monitor,
            M.screen,
            Some(M.wallpaper),
            -1.0,
            FT,
            (0.22, 0.14),
        );
        fsweep!(
            F,
            "monitor_cable",
            &[
                (0.05, 0.86, 0.44),
                (0.08, 0.78, 0.443),
                (0.2, FT + 0.004, 0.435),
                (0.3, FT + 0.004, 0.43)
            ],
            0.003,
            M.pen,
            verts = 5,
            resolution = 3
        );
        mug(0.58, 0.3, FT);
        desk_lamp(-0.6, 0.38);
        cactus(0.62, 0.38, FT, 0.045, 0.08);
        notebook(
            "notebook", -0.3, FT, 0.26, 0.2, 0.26, 0.015, M.notebook, 12.0,
        );
        pen("pen", -0.27, FT + 0.015, 0.24, 60.0);
        desk_chair(M.chair, CHAIR_Z, 0.5, false, None);
    });
    aside(&carried, "carried");
    // the keyboard tray: under the top, sliding 15 cm out to the resident
    let parts = new_parts(|| {
        fbox!(
            F,
            "tray",
            0.66,
            0.012,
            0.3,
            (0.16, 0.655, 0.25),
            M.tray,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "tray_lip",
            0.66,
            0.03,
            0.012,
            (0.16, 0.668, 0.106),
            M.tray,
            bevel = 0.004,
            bevel_segments = 1
        );
        keyboard(
            "keyboard", 0.06, 0.661, 0.22, 0.42, 0.13, M.keys, M.keys, -1.0, 13,
        );
        fbox!(
            F,
            "mouse_pad",
            0.18,
            0.003,
            0.18,
            (0.38, 0.6625, 0.22),
            M.pad,
            bevel = 0.0014,
            bevel_segments = 1
        );
        mouse("mouse", 0.38, 0.664, 0.22, M.keys, -1.0);
        fsweep!(
            F,
            "mouse_cable",
            &[
                (0.38, 0.666, 0.272),
                (0.37, 0.664, 0.33),
                (0.3, 0.663, 0.37),
                (0.15, 0.663, 0.38)
            ],
            0.002,
            M.keys,
            verts = 4,
            resolution = 3
        );
    });
    let tray = join(&parts, "tray", None);
    settle!(centre = false);
    slide!(tray, "tray_out", (0, 0.15, 0), seconds = 0.6);
    meshes()
}

pub fn desk_writing() -> Vec<Obj> {
    let still = new_parts(|| {
        top(1.54, 0.42, 0.24, M.oak, FT);
        desk_frame(1.54, None, Some((-0.1, 0.4)), None);
        fbox!(
            F,
            "drawer_rails",
            0.52,
            0.01,
            0.38,
            (0.15, 0.662, 0.24),
            M.frame
        );
    });
    join!(&still, "deskWriting", origin = (0, 0, 0));
    // what lies on it and the chair drawn at it: part of the picture, not of the desk's size (a desk is
    // 1.54 x 0.75 x 0.87 m with its chair's back and the lamp standing over it)
    let carried = new_parts(|| {
        for (k, (x, z, turn, dy)) in [
            (-0.12, 0.2, 8, 0.0015),
            (-0.1, 0.22, -4, 0.0045),
            (-0.13, 0.19, 15, 0.0075),
        ]
        .into_iter()
        .enumerate()
        {
            fbox!(
                F,
                format!("paper{k}"),
                0.3,
                0.003,
                0.21,
                (x, FT + dy, z),
                M.paper,
                rot = (0, 0, turn)
            );
        }
        for j in 0..5 {
            // writing on the top sheet
            fbox!(
                F,
                format!("line{j}"),
                0.2 - 0.03 * f64::from(j % 2),
                0.0004,
                0.004,
                (-0.13, FT + 0.0092, 0.15 + 0.024 * f64::from(j)),
                M.pen,
                rot = (0, 0, 15)
            );
        }
        pen("pen", 0.08, FT, 0.2, 85.0);
        for (k, m) in [M.notebook, M.mug].into_iter().enumerate() {
            let k = k as i32;
            notebook(
                &format!("notebook{k}"),
                0.45,
                FT + 0.025 * f64::from(k),
                0.26,
                0.2,
                0.27,
                0.025,
                m,
                f64::from(6 * k - 3),
            );
        }
        desk_lamp(-0.57, 0.36);
        desk_chair(M.chair_sage, CHAIR_Z, 0.5, false, None);
    });
    aside(&carried, "carried");
    // the drawer: a front with a turned knob and a box of letters and pencils, sliding out to the resident
    let parts = new_parts(|| {
        front("drawer", 0.5, 0.05, 0.15, 0.69, 0.051, M.drawers, 0.018);
        revolve!(
            "drawer_knob",
            &[
                (0.0, 0.0),
                (0.008, 0.0),
                (0.007, 0.004),
                (0.004, 0.008),
                (0.005, 0.012),
                (0.011, 0.016),
                (0.012, 0.02),
                (0.009, 0.024),
                (0.0, 0.025)
            ],
            at = F.at(0.15, 0.69, 0.042),
            material = M.brass,
            segments = 12,
            rot = (-90, 0, 0)
        );
        drawer_box(
            "drawer_box",
            0.48,
            0.042,
            0.36,
            0.15,
            0.668,
            0.06,
            M.drawers,
        );
        for (k, (x, turn)) in [(0.04, 5), (0.07, -3)].into_iter().enumerate() {
            fbox!(
                F,
                format!("letter{k}"),
                0.22,
                0.006,
                0.12,
                (x, 0.681 + 0.006 * k as f64, 0.2),
                M.paper,
                rot = (0, 0, turn)
            );
            fbox!(
                F,
                format!("stamp{k}"),
                0.022,
                0.001,
                0.026,
                (x + 0.08, 0.6845 + 0.006 * k as f64, 0.24),
                M.note3,
                rot = (0, 0, turn)
            );
        }
        for k in 0..3 {
            let (px, pz) = (0.3, 0.16 + 0.02 * f64::from(k));
            fcyl!(
                F,
                format!("pencil{k}"),
                0.0042,
                0.13,
                (px, 0.68, pz),
                [M.notebook, M.pen, M.mug][k as usize],
                rot = (0, 90, 0),
                verts = 6
            );
            fcone!(
                F,
                format!("pencil_tip{k}"),
                0.0042,
                0.0008,
                0.018,
                (px - 0.074, 0.68, pz),
                M.folder,
                rot = (0, -90, 0),
                verts = 6
            );
        }
    });
    let drawer = join(&parts, "drawer", None);
    settle!(centre = false);
    slide!(drawer, "drawer_open", (0, 0.22, 0), seconds = 0.6);
    meshes()
}

pub fn desk_laptop() -> Vec<Obj> {
    let still = new_parts(|| {
        top(0.9, 0.42, 0.24, M.oak, FT);
        desk_frame(0.9, None, Some((-0.38, -0.02)), None);
        front("drawer", 0.34, 0.05, -0.2, 0.69, 0.052, M.drawers, 0.016);
        revolve!(
            "drawer_knob",
            &[
                (0.0, 0.0),
                (0.008, 0.0),
                (0.007, 0.004),
                (0.004, 0.008),
                (0.005, 0.012),
                (0.01, 0.015),
                (0.011, 0.019),
                (0.008, 0.022),
                (0.0, 0.023)
            ],
            at = F.at(-0.2, 0.69, 0.044),
            material = M.frame,
            segments = 12,
            rot = (-90, 0, 0)
        );
        fbox!(
            F,
            "drawer_back",
            0.36,
            0.055,
            0.3,
            (-0.2, 0.69, 0.21),
            M.drawers
        );
        // the laptop's base: a rounded slab, the keys sunk in a well towards the hinge, the touchpad to the resident
        fbox!(
            F,
            "laptop",
            0.32,
            0.012,
            0.22,
            (0.02, FT + 0.006, 0.2),
            M.aluminium,
            bevel = 0.004,
            bevel_segments = 1
        );
        laptop_keys(0.02, FT + 0.012, 0.25);
        fbox!(
            F,
            "laptop_pad",
            0.1,
            0.0006,
            0.06,
            (0.02, FT + 0.0123, 0.14),
            M.pad
        );
        fcyl!(
            F,
            "laptop_hinge",
            0.004,
            0.24,
            (0.02, FT + 0.012, 0.309),
            M.pad,
            rot = (0, 90, 0),
            verts = 8
        );
    });
    join!(&still, "deskLaptop", origin = (0, 0, 0));
    // the mug, notebook, pen and charger on it, and the chair drawn at it: not part of the desk's size
    let carried = new_parts(|| {
        mug(0.32, 0.36, FT);
        notebook(
            "notebook", -0.28, FT, 0.28, 0.18, 0.24, 0.012, M.notebook, -10.0,
        );
        pen("pen", -0.27, FT + 0.012, 0.27, 70.0);
        fsweep!(
            F,
            "charger",
            &[
                (0.18, FT + 0.006, 0.26),
                (0.24, FT + 0.004, 0.3),
                (0.3, FT + 0.004, 0.42),
                (0.33, FT + 0.004, 0.45)
            ],
            0.0025,
            M.pen,
            verts = 4,
            resolution = 3
        );
        desk_chair(M.chair, CHAIR_Z, 0.5, false, None);
    });
    aside(&carried, "carried");
    // the lid, shut on the base, hinged along the base's far edge
    let parts = new_parts(|| {
        fbox!(
            F,
            "lid",
            0.32,
            0.007,
            0.22,
            (0.02, FT + 0.0165, 0.2),
            M.aluminium,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            F,
            "lid_bezel",
            0.305,
            0.0006,
            0.205,
            (0.02, FT + 0.013, 0.2),
            M.pad
        );
        fbox!(
            F,
            "lid_screen",
            0.29,
            0.0008,
            0.18,
            (0.02, FT + 0.0127, 0.205),
            M.screen
        );
        fcyl!(
            F,
            "lid_logo",
            0.014,
            0.0008,
            (0.02, FT + 0.0201, 0.2),
            M.chrome,
            verts = 12
        );
    });
    let lid = hinged(join(&parts, "lid", None), F.at(0.02, FT + 0.013, 0.31));
    settle!(centre = false);
    swing!(lid, "lid_open", 110.0, seconds = 0.8, axis = Axis::X);
    meshes()
}

pub fn computer_gaming() -> Vec<Obj> {
    let accent = M.accent;
    let still = new_parts(|| {
        top(1.7, 0.44, 0.24, M.desk, FT);
        desk_frame(1.7, Some(M.desk), None, Some(M.desk));
        fbox!(
            F,
            "desk_mat",
            1.6,
            0.004,
            0.36,
            (0, FT + 0.002, 0.22),
            M.mat,
            bevel = 0.0018,
            bevel_segments = 1
        );
        fbox!(
            F,
            "led_strip",
            1.62,
            0.01,
            0.006,
            (0, FT - 0.012, 0.463),
            accent
        );
        // two monitors on stands, turned in to the resident
        for (s, scr) in [(-1, M.screen2), (1, M.wallpaper)] {
            let x = f64::from(s) * 0.36;
            let parts = monitor(
                &format!("monitor{s}"),
                x,
                0.36,
                0.62,
                0.36,
                1.03,
                M.case,
                scr,
                None,
                -1.0,
                FT,
                (0.22, 0.15),
            );
            let glow = new_parts(|| {
                fbox!(
                    F,
                    format!("monitor_glow{s}"),
                    0.4,
                    0.006,
                    0.006,
                    (x, 0.95, 0.385),
                    accent
                );
            });
            turned(
                &[parts, glow].concat(),
                &format!("monitor{s}"),
                (x, 1.0, 0.36),
                0.0,
                f64::from(-s) * 15.0,
            );
        }
        keyboard(
            "keyboard",
            0,
            FT + 0.004,
            0.17,
            0.46,
            0.15,
            M.case,
            accent,
            -1.0,
            14,
        );
        fbox!(
            F,
            "wrist_rest",
            0.44,
            0.012,
            0.05,
            (0, FT + 0.01, 0.075),
            M.mat,
            bevel = 0.005,
            bevel_segments = 1
        );
        mouse("mouse", 0.36, FT + 0.004, 0.17, M.case, -1.0);
        fbox!(
            F,
            "mouse_light",
            0.012,
            0.002,
            0.03,
            (0.36, FT + 0.0235, 0.135),
            accent
        );
        // the speaker: a rounded box, its woofer and tweeter behind rings
        fbox!(
            F,
            "speaker",
            0.16,
            0.2,
            0.14,
            (-0.66, FT + 0.1, 0.36),
            M.case,
            bevel = 0.014,
            bevel_segments = 2
        );
        for (k, (y, r)) in [(FT + 0.075, 0.048), (FT + 0.155, 0.02)]
            .into_iter()
            .enumerate()
        {
            fcyl!(
                F,
                format!("driver{k}"),
                r,
                0.006,
                (-0.66, y, 0.288),
                M.blade,
                rot = (90, 0, 0),
                verts = 18
            );
            torus!(
                format!("driver_ring{k}"),
                r + 0.003,
                0.0035,
                at = F.at(-0.66, y, 0.2885),
                material = accent,
                rot = (90, 0, 0),
                major_segments = 18,
                minor_segments = 4
            );
        }
        fsphere!(
            F,
            "woofer",
            0.016,
            (-0.66, FT + 0.075, 0.29),
            M.case,
            scale = (1, 0.5, 1),
            segments = 10,
            rings = 5
        );
        // headphones hung on a hook under the desk's edge
        fsweep!(
            F,
            "headset",
            &[
                (0.8, FT - 0.05, 0.07),
                (0.8, FT - 0.01, 0.06),
                (0.8, FT - 0.01, 0.0),
                (0.8, FT - 0.05, -0.01)
            ],
            0.006,
            M.case,
            verts = 6,
            resolution = 3
        );
        for z in [0.07, -0.01] {
            fbox!(
                F,
                format!("earcup{}", float_repr(z)),
                0.03,
                0.07,
                0.06,
                (0.8, FT - 0.08, z),
                M.case,
                bevel = 0.012,
                bevel_segments = 2
            );
        }
        // the tower under the desk: a tinted side to the resident, two fans in the front behind a mesh grille
        fbox!(
            F,
            "tower",
            0.24,
            0.5,
            0.48,
            (0.62, 0.27, 0.2),
            M.case,
            bevel = 0.01,
            bevel_segments = 2
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("tower_foot{s}"),
                0.2,
                0.02,
                0.04,
                (0.62, 0.01, 0.2 + f64::from(s) * 0.19),
                M.plastic,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        fbox!(
            F,
            "tower_side",
            0.004,
            0.4,
            0.38,
            (0.499, 0.28, 0.2),
            M.side
        );
        for k in 0..2 {
            // what shows through the tinted side: the graphics card and the board's glow
            fbox!(
                F,
                format!("tower_card{k}"),
                0.004,
                0.035,
                0.26,
                (0.503, 0.24 + 0.08 * f64::from(k), 0.21),
                M.blade
            );
        }
        fbox!(
            F,
            "tower_strip",
            0.012,
            0.42,
            0.004,
            (0.53, 0.27, 0.441),
            accent
        );
        fbox!(
            F,
            "tower_top_vent",
            0.18,
            0.002,
            0.3,
            (0.62, 0.5205, 0.2),
            M.blade
        );
        for k in 0..2 {
            fcyl!(
                F,
                format!("tower_button{k}"),
                0.008,
                0.004,
                (0.66 - 0.04 * f64::from(k), 0.522, 0.4),
                if k == 0 { accent } else { M.chrome },
                verts = 10
            );
        }
        for (k, y) in [0.38, 0.17].into_iter().enumerate() {
            torus!(
                format!("fan_ring{k}"),
                0.074,
                0.008,
                at = F.at(0.62, y, 0.442),
                material = accent,
                rot = (90, 0, 0),
                major_segments = 20,
                minor_segments = 4
            );
        }
        desk_chair(M.chair_gamer, CHAIR_Z, 0.62, true, Some(accent));
    });
    join!(&still, "computerGaming", origin = (0, 0, 0));
    let mut fans = Vec::new();
    let blade = [
        (0.0, -0.006),
        (0.03, -0.013),
        (0.052, -0.002),
        (0.05, 0.011),
        (0.026, 0.012),
    ];
    for (k, y) in [0.38, 0.17].into_iter().enumerate() {
        let parts = new_parts(|| {
            fcyl!(
                F,
                format!("fan_hub{k}"),
                0.022,
                0.012,
                (0.62, y, 0.446),
                M.blade,
                rot = (90, 0, 0),
                verts = 16,
                bevel = 0.003,
                bevel_segments = 1
            );
            fcyl!(
                F,
                format!("fan_badge{k}"),
                0.012,
                0.012,
                (0.62, y, 0.447),
                accent,
                rot = (90, 0, 0),
                verts = 12
            );
            for b in 0..7 {
                let a = f64::from(b * 360) / 7.0;
                fextrude!(
                    F,
                    format!("blade{k}{b}"),
                    &blade,
                    0.003,
                    (0.62, y, 0.444),
                    M.blade,
                    rot = (0.0, a, 0.0)
                );
            }
        });
        fans.push(hinged(
            join(&parts, &format!("fan{k}"), None),
            F.at(0.62, y, 0.446),
        ));
    }
    settle!(centre = false);
    for fan in fans {
        spin(
            fan,
            "fans",
            Spin {
                turns: 1.0,
                seconds: 1.0,
                axis: Axis::Y,
            },
        );
    }
    meshes()
}
