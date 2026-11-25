//! O1 continued: the catalog2 desks: the simple, corner, standing and executive desks.

use super::chairs::desk_chair;
use super::common::{bar_pull, drawer_box, front, top, turned};
use super::desk_things::{cactus, keyboard, laptop_open, monitor, mouse, mug, notebook, pen};
use super::{DT, F, M};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::json::float_repr;
use artkit::math::P3;
use artkit::parts::{aside, join, join_all, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, join, kit, settle, slide, soft};

// --- the catalog2 desks --------------------------------------------------------------------------------------
pub fn desk_simple() -> Vec<Obj> {
    new_parts(|| {
        top(1.5, 0.69, -0.085, M.oak, DT);
        // a drawer box hanging under the right, its slab front with a bar pull
        fbox!(
            F,
            "drawer_box",
            0.42,
            0.12,
            0.6,
            (0.45, DT - 0.095, -0.12),
            M.drawers,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "drawer_reveal",
            0.39,
            0.096,
            0.002,
            (0.45, DT - 0.095, 0.1805),
            M.reveal
        );
        front(
            "drawer_front",
            0.4,
            0.1,
            0.45,
            DT - 0.095,
            0.188,
            M.drawers,
            0.012,
        );
        bar_pull(
            "drawer_pull",
            0.45,
            DT - 0.095,
            0.194,
            0.14,
            M.frame,
            0.022,
            0.005,
        );
        // hairpin legs: each bent from one rod, from a plate under the top down to its foot and up again
        for x in [-0.68, 0.68] {
            for z in [-0.33, 0.17] {
                let (xs, zs) = (float_repr(x), float_repr(z));
                fbox!(
                    F,
                    format!("plate{xs}{zs}"),
                    0.06,
                    0.006,
                    0.18,
                    (x, DT - 0.038, z),
                    M.frame,
                    bevel = 0.002,
                    bevel_segments = 1
                );
                for dz in [-0.07, 0.07] {
                    fcyl!(
                        F,
                        format!("screw{xs}{zs}{}", float_repr(dz)),
                        0.005,
                        0.003,
                        (x, DT - 0.0345, z + dz * 1.15),
                        M.chrome,
                        verts = 6
                    );
                }
                let pin = [
                    (x, DT - 0.04, z - 0.07),
                    (x, 0.012, z - 0.004),
                    (x, 0.012, z + 0.004),
                    (x, DT - 0.04, z + 0.07),
                ];
                fsweep!(
                    F,
                    format!("pin{xs}{zs}"),
                    &kit::fillet!(&pin, 0.006, 3),
                    0.0065,
                    M.frame,
                    verts = 8
                );
                fsphere!(
                    F,
                    format!("pin_foot{xs}{zs}"),
                    0.012,
                    (x, 0.012, z),
                    M.plastic,
                    segments = 10,
                    rings = 6
                );
            }
        }
    });
    join_all("deskSimple");
    settle!(centre = false);
    meshes()
}

// The corner desk: an L of two 0.7 m wings along the back and the left (the walls of a room's corner), the inner
// corner cut on the diagonal, and its own swivel chair in the inner corner facing it, both wings in reach (content
// `places`: `CORNER_SEAT`, turned to face the corner). Review M-office-01: it was a 1.86 m square slab with one
// corner cut, read as an island, its middle a metre from any edge.
pub(super) const CORNER_WING: f64 = 0.7;
pub(super) const CORNER_CUT: f64 = -0.06; // the diagonal edge: game x + depth = this
pub(super) const CORNER_SEAT: (f64, f64) = (0.11, 0.11); // the chair's seat (game x, depth): 0.2 m back from the diagonal edge

pub fn desk_corner() -> Vec<Obj> {
    let e = 0.93;
    let i = -e + CORNER_WING; // the wings' inner edges
    let desk = new_parts(|| {
        // an L of white laminate, its edges rounded
        let cut = [(CORNER_CUT - i, i), (i, CORNER_CUT - i)];
        let outline = [(-e, -e), (e, -e), (e, i), cut[0], cut[1], (i, e), (-e, e)];
        fextrude!(
            F,
            "top",
            &outline,
            0.035,
            (0, DT - 0.0175, 0),
            M.laminate,
            plane = Facing::Top,
            bevel = 0.006,
            bevel_segments = 2
        );
        fcyl!(
            F,
            "grommet",
            0.03,
            0.002,
            (-0.75, DT + 0.0005, -0.75),
            M.frame,
            verts = 16
        );
        fcyl!(
            F,
            "grommet_hole",
            0.024,
            0.0025,
            (-0.75, DT + 0.0006, -0.75),
            M.reveal,
            verts = 16
        );
        // oak panels at the outer ends and along the back and the left, and a drawer pedestal under each wing
        let h = DT - 0.035;
        fbox!(
            F,
            "panel_left",
            0.03,
            h,
            1.8,
            (-0.89, h / 2.0, 0),
            M.oak_panel,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "panel_right",
            0.03,
            h,
            CORNER_WING - 0.04,
            (0.89, h / 2.0, -e + CORNER_WING / 2.0),
            M.oak_panel,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "panel_front",
            CORNER_WING - 0.04,
            h,
            0.03,
            (-e + CORNER_WING / 2.0, h / 2.0, 0.89),
            M.oak_panel,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "panel_back",
            1.74,
            0.4,
            0.02,
            (0, 0.45, -0.89),
            M.oak_panel,
            bevel = 0.004,
            bevel_segments = 1
        );
        for (k, (x, z, turn)) in [(0.5, -0.6, 0), (-0.6, 0.5, 1)].into_iter().enumerate() {
            let (w, d) = if turn == 0 {
                (0.42, 0.55)
            } else {
                (0.55, 0.42)
            };
            fbox!(
                F,
                format!("pedestal{k}"),
                w,
                0.6,
                d,
                (x, 0.3, z),
                M.drawers,
                bevel = 0.008,
                bevel_segments = 1
            );
            // the plinth, set back in shadow, and three slab fronts with reveals and bar pulls
            if turn != 0 {
                fbox!(
                    F,
                    format!("pedestal{k}_reveal"),
                    0.002,
                    0.53,
                    0.39,
                    (x + 0.276, 0.3, z),
                    M.reveal
                );
            } else {
                fbox!(
                    F,
                    format!("pedestal{k}_reveal"),
                    0.39,
                    0.53,
                    0.002,
                    (x, 0.3, z + 0.276),
                    M.reveal
                );
            }
            for j in 0..3 {
                let y = 0.5 - f64::from(j) * 0.18;
                if turn != 0 {
                    // its drawers to the inner side, towards the chair
                    fbox!(
                        F,
                        format!("pedestal{k}_front{j}"),
                        0.014,
                        0.17,
                        0.4,
                        (x + 0.282, y, z),
                        M.drawers,
                        bevel = 0.004,
                        bevel_segments = 1
                    );
                    let pts = [
                        (x + 0.286, y + 0.045, z - 0.07),
                        (x + 0.31, y + 0.045, z - 0.07),
                        (x + 0.31, y + 0.045, z + 0.07),
                        (x + 0.286, y + 0.045, z + 0.07),
                    ];
                    fsweep!(
                        F,
                        format!("pedestal{k}_pull{j}"),
                        &kit::fillet!(&pts, 0.009),
                        0.005,
                        M.frame,
                        verts = 8
                    );
                } else {
                    fbox!(
                        F,
                        format!("pedestal{k}_front{j}"),
                        0.4,
                        0.17,
                        0.014,
                        (x, y, z + 0.282),
                        M.drawers,
                        bevel = 0.004,
                        bevel_segments = 1
                    );
                    bar_pull(
                        &format!("pedestal{k}_pull{j}"),
                        x,
                        y + 0.045,
                        z + 0.289,
                        0.14,
                        M.frame,
                        0.024,
                        0.005,
                    );
                }
            }
        }
    });
    // on the top: an open laptop on the back wing turned to the chair, a mug, a tray of papers, a cactus and a
    // notebook; the chair at the corner: drawn, not the desk's size
    let carried = new_parts(|| {
        laptop_open(0.05, -0.62, DT, -20.0);
        mug(0.42, -0.45, DT);
        fbox!(
            F,
            "paper_tray",
            0.25,
            0.018,
            0.32,
            (-0.62, DT + 0.009, -0.15),
            M.plastic,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "paper_tray_papers",
            0.215,
            0.012,
            0.29,
            (-0.62, DT + 0.022, -0.15),
            M.paper,
            rot = (0, 0, 3)
        );
        cactus(-0.78, -0.78, DT, 0.045, 0.08);
        notebook(
            "notebook", -0.58, DT, 0.35, 0.18, 0.24, 0.014, M.notebook, 20.0,
        );
        pen("pen", -0.5, DT + 0.014, 0.32, 40.0);
        let chair = new_parts(|| desk_chair(M.chair, 0.0, 0.5, false, None));
        let seat = turned(&chair, "chair", (0, 0, 0), 0.0, 135.0);
        seat.edit(|o| {
            o.loc.x = (f64::from(o.loc.x) - CORNER_SEAT.0) as f32;
            o.loc.y = (f64::from(o.loc.y) - CORNER_SEAT.1) as f32;
        });
    });
    aside(&carried, "carried");
    join(&desk, "deskCorner", Some(P3::ZERO));
    settle!(centre = false);
    meshes()
}

pub fn desk_standing() -> Vec<Obj> {
    let still = new_parts(|| {
        soft!(
            F,
            "top",
            0.96,
            0.03,
            0.63,
            (0, 1.035, -0.115),
            M.birch,
            round = 0.008,
            round_segments = 2
        );
        fbox!(
            F,
            "crossbar",
            0.84,
            0.04,
            0.06,
            (0, 0.995, -0.13),
            M.frame,
            bevel = 0.006,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let x = f64::from(s) * 0.38;
            fbox!(
                F,
                format!("bracket{s}"),
                0.06,
                0.03,
                0.56,
                (x, 1.005, -0.12),
                M.frame,
                bevel = 0.006,
                bevel_segments = 1
            );
            // the telescoping leg: a wide outer column, a slimmer one sliding out of it, a collar between
            fbox!(
                F,
                format!("column{s}"),
                0.08,
                0.55,
                0.06,
                (x, 0.315, -0.13),
                M.frame,
                bevel = 0.008,
                bevel_segments = 2
            );
            fbox!(
                F,
                format!("collar{s}"),
                0.084,
                0.012,
                0.064,
                (x, 0.594, -0.13),
                M.plastic,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("column_top{s}"),
                0.065,
                0.44,
                0.048,
                (x, 0.77, -0.13),
                M.frame,
                bevel = 0.006,
                bevel_segments = 2
            );
            // the foot: tapering to both ends, on two levellers
            fsweep!(
                F,
                format!("foot{s}"),
                &[(x, 0.032, -0.415), (x, 0.038, -0.13), (x, 0.032, 0.175)],
                0.0,
                M.frame,
                profile = rounded_rect(0.042, 0.07, 0.012, 2),
                up = (0, 0, 1),
                resolution = 3,
                scale = [0.7, 1.0, 0.7],
                angle = 50
            );
            for z in [-0.39, 0.15] {
                fcyl!(
                    F,
                    format!("leveller{s}{}", float_repr(z)),
                    0.014,
                    0.018,
                    (x, 0.009, z),
                    M.plastic,
                    verts = 10
                );
            }
            fbox!(
                F,
                format!("runner{s}"),
                0.015,
                0.035,
                0.26,
                (f64::from(s) * 0.31, 1.0, 0.06),
                M.frame,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // the handset under the front right: a display and four buttons
        fbox!(
            F,
            "controls",
            0.12,
            0.022,
            0.05,
            (0.36, 1.007, 0.165),
            M.plastic,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            F,
            "controls_display",
            0.036,
            0.012,
            0.002,
            (0.325, 1.006, 0.1905),
            M.screen
        );
        fbox!(
            F,
            "controls_height",
            0.022,
            0.005,
            0.0004,
            (0.325, 1.006, 0.1916),
            M.led
        );
        for k in 0..4 {
            fcyl!(
                F,
                format!("button{k}"),
                0.005,
                0.003,
                (0.36 + 0.014 * f64::from(k), 1.006, 0.191),
                M.chrome,
                rot = (90, 0, 0),
                verts = 8
            );
        }
    });
    // the monitor on its stand at the back, facing the resident, its cable down the left leg, a mug and a cactus:
    // drawn, not the desk's size
    let carried = new_parts(|| {
        monitor(
            "monitor",
            0,
            -0.38,
            0.56,
            0.34,
            1.37,
            M.plastic,
            M.screen,
            Some(M.wallpaper),
            1.0,
            1.05,
            (0.2, 0.14),
        );
        fsweep!(
            F,
            "cable",
            &[
                (-0.05, 1.3, -0.415),
                (-0.2, 1.06, -0.42),
                (-0.32, 1.0, -0.4),
                (-0.36, 0.9, -0.165),
                (-0.36, 0.3, -0.165),
                (-0.36, 0.05, -0.3),
                (-0.3, 0.004, -0.41)
            ],
            0.004,
            M.pen,
            verts = 5,
            resolution = 3
        );
        mug(0.32, -0.3, 1.05);
        cactus(-0.36, -0.36, 1.05, 0.04, 0.07);
    });
    aside(&carried, "carried");
    join!(&still, "deskStanding", origin = (0, 0, 0));
    // the keyboard shelf under the front, sliding 20 cm out to the resident
    let parts = new_parts(|| {
        fbox!(
            F,
            "shelf",
            0.6,
            0.012,
            0.24,
            (0, 0.976, 0.06),
            M.shelf,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "shelf_lip",
            0.6,
            0.026,
            0.012,
            (0, 0.983, 0.184),
            M.shelf,
            bevel = 0.004,
            bevel_segments = 1
        );
        keyboard(
            "keyboard",
            -0.04,
            0.982,
            0.08,
            0.42,
            0.13,
            M.keys,
            M.keycap_light,
            1.0,
            13,
        );
        mouse("mouse", 0.22, 0.983, 0.08, M.keys, 1.0);
    });
    let shelf = join(&parts, "shelf", None);
    settle!(centre = false);
    slide!(shelf, "tray_out", (0, -0.2, 0), seconds = 0.6);
    meshes()
}

/// One of the executive desk's drawer fronts: raised-panel, a brass bar pull between two roses.
#[allow(clippy::too_many_arguments)]
pub(super) fn drawer_front(name: &str, w: f64, h: f64, x: f64, y: f64, z: f64, pull_y: f64) {
    kit::shaker!(
        F,
        name,
        w,
        h,
        (x, y, z),
        0.02,
        M.walnut2,
        rail = 0.045_f64.min(h * 0.25),
        raised = true
    );
    kit::pull!(
        F,
        format!("{name}_pull"),
        x,
        y + pull_y,
        z + 0.01,
        0.12_f64.min(w * 0.3),
        M.brass,
        r = 0.005,
        out = 0.016
    );
    for p in [-1, 1] {
        fcyl!(
            F,
            format!("{name}_rose{p}"),
            0.009,
            0.003,
            (
                x + f64::from(p) * 0.06_f64.min(w * 0.15),
                y + pull_y,
                z + 0.0115
            ),
            M.brass,
            rot = (90, 0, 0),
            verts = 10
        );
    }
}

pub fn desk_executive() -> Vec<Obj> {
    let still = new_parts(|| {
        soft!(
            F,
            "top",
            1.8,
            0.05,
            0.77,
            (0, DT - 0.025, -0.045),
            M.walnut,
            round = 0.014,
            round_segments = 2
        );
        // the leather pad in the top, a tooled border stitched round it
        fbox!(
            F,
            "inlay",
            1.3,
            0.004,
            0.46,
            (0, DT + 0.001, -0.08),
            M.inlay,
            bevel = 0.0015,
            bevel_segments = 1
        );
        let border: Vec<(f64, f64, f64)> = rounded_rect(1.25, 0.41, 0.02, 2)
            .iter()
            .map(|&(u, v)| (u, DT + 0.0032, -0.08 + v))
            .collect();
        fsweep!(
            F,
            "inlay_stitch",
            &border,
            0.0012,
            M.brass,
            verts = 3,
            closed = true
        );
        for s in [-1, 1] {
            let x = f64::from(s) * 0.66;
            fbox!(
                F,
                format!("plinth{s}"),
                0.42,
                0.06,
                0.68,
                (x, 0.03, -0.07),
                M.walnut_case,
                bevel = 0.006,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("pedestal{s}"),
                0.44,
                DT - 0.11,
                0.72,
                (x, 0.06 + (DT - 0.11) / 2.0, -0.06),
                M.walnut_case,
                bevel = 0.01,
                bevel_segments = 2
            );
            // a moulding round the pedestal's foot
            kit::moulding!(
                F,
                format!("base_moulding{s}"),
                &[
                    (x + 0.22, -0.418),
                    (x + 0.22, 0.3),
                    (x - 0.22, 0.3),
                    (x - 0.22, -0.418),
                ],
                &[
                    (0.0, 0.0),
                    (-0.012, 0.0),
                    (-0.012, 0.02),
                    (-0.004, 0.035),
                    (0.0, 0.04),
                ],
                M.walnut_case,
                at = 0.06,
                plane = Facing::Top
            );
            for (i, (y, h)) in [(0.585, 0.18), (0.385, 0.18), (0.175, 0.2)]
                .into_iter()
                .enumerate()
            {
                drawer_front(
                    &format!("front{s}{i}"),
                    0.4,
                    h - 0.008,
                    x,
                    y,
                    0.31,
                    h / 2.0 - 0.045,
                );
            }
        }
        // the modesty panel at the back of the kneehole, framed and panelled
        kit::shaker!(
            F,
            "modesty",
            0.88,
            0.45,
            (0, 0.44, -0.38),
            0.025,
            M.walnut_case,
            rail = 0.07,
            raised = true
        );
        fbox!(
            F,
            "apron",
            0.88,
            0.07,
            0.025,
            (0, DT - 0.085, -0.3),
            M.walnut_case,
            bevel = 0.004,
            bevel_segments = 1
        );
        // a fountain pen and a letter opener on the pad
        pen("pen", 0.42, DT + 0.003, -0.05, 75.0);
        fcyl!(
            F,
            "pen_tip",
            0.0055,
            0.03,
            (0.4805, DT + 0.008, -0.034),
            M.brass,
            rot = (90, 0, 75),
            verts = 8
        );
        fextrude!(
            F,
            "opener",
            &smooth_outline(
                &[
                    (0.0, -0.004),
                    (0.15, -0.002),
                    (0.17, 0.0),
                    (0.15, 0.002),
                    (0.0, 0.006),
                    (-0.05, 0.006),
                    (-0.06, 0.0),
                    (-0.05, -0.006)
                ],
                1,
                true
            ),
            0.003,
            (0.25, DT + 0.0045, 0.0),
            M.brass,
            plane = Facing::Top,
            rot = (0, 0, 20)
        );
    });
    join!(&still, "deskExecutive", origin = (0, 0, 0));
    // the middle drawer, still: no use opens it yet
    let parts = new_parts(|| {
        kit::shaker!(
            F,
            "drawer",
            0.84,
            0.075,
            (0, DT - 0.09, 0.31),
            0.02,
            M.walnut2,
            rail = 0.025
        );
        kit::pull!(
            F,
            "drawer_pull",
            0,
            DT - 0.09,
            0.32,
            0.16,
            M.brass,
            r = 0.006,
            out = 0.015
        );
        drawer_box(
            "drawer_box",
            0.8,
            0.06,
            0.5,
            0.0,
            DT - 0.125,
            -0.2,
            M.walnut2,
        );
    });
    join(&parts, "drawer", None);
    settle!(centre = false);
    meshes()
}
