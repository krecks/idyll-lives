//! KD2: the toy box and the toy chest.

use super::beds::bun;
use super::common::{P, block, place, star, still, teddy, walls};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::kit;
use artkit::math::{Axis, PI, TAU, cos, sin};
use artkit::parts::{aside, hinged, join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, sphere, swing};

/// Toys heaped inside a box, seen with the lid open: a ball, two blocks, a teddy and a toy car's wheels.
fn toys_inside(top: f64, z: f64) {
    sphere!(
        "in_ball",
        0.075,
        at = F.at(0.16, top - 0.07, z + 0.05),
        material = M.ball_blue,
        segments = 14,
        rings = 8
    );
    fsphere!(
        F,
        "in_ball_band",
        0.0755,
        (0.16, top - 0.07, z + 0.05),
        M.ball_yellow,
        scale = (1, 0.3, 1),
        segments = 14,
        rings = 4
    );
    block(
        "in_block",
        -0.2,
        top - 0.145,
        z + 0.02,
        0.09,
        M.toy_red,
        M.toy_yellow,
        20.0,
        0.0,
    );
    block(
        "in_block2",
        -0.08,
        top - 0.14,
        z - 0.08,
        0.08,
        M.toy_green,
        M.paper,
        -15.0,
        10.0,
    );
    teddy(0.0, top - 0.17, z + 0.07, M.plush, M.eye, 0.05);
}

pub fn toy_box() -> Vec<Obj> {
    let z = -0.18;
    fbox!(
        F,
        "box_floor",
        0.78,
        0.03,
        0.48,
        (0, 0.03, z),
        M.toybox_inside
    );
    let box_walls = walls(
        "box_walls",
        0.8,
        0.5,
        0.0,
        0.445,
        z,
        0.025,
        M.toybox,
        0.03,
        0.0,
    );
    // hand holds cut through the ends
    for s in [-1, 1] {
        kit::cut!(
            box_walls,
            fextrude!(
                F,
                format!("grip{s}"),
                &rounded_rect(0.13, 0.035, 0.0175, 3),
                0.1,
                (s as f64 * 0.4, 0.37, z),
                M.toybox,
                plane = Facing::Side
            )
        );
    }
    // a rounded rim along the walls' top edge, under the lid
    let rim = rounded_rect(0.775, 0.475, 0.018, 3);
    let path: Vec<P> = rim.iter().map(|&(u, v)| (u, 0.445, z + v)).collect();
    fsweep!(
        F,
        "box_rim",
        &path,
        0,
        M.toybox,
        profile = rounded_rect(0.016, 0.03, 0.007, 1),
        up = (0, 0, 1),
        closed = true
    );
    fbox!(F, "fill", 0.75, 0.01, 0.45, (0, 0.31, z), M.toybox_inside);
    // the front: a star, a circle and a triangle set into it, each in its frame
    let circle: Vec<(f64, f64)> = (0..16)
        .map(|i| {
            (
                0.055 * cos(TAU * i as f64 / 16.0),
                0.055 * sin(TAU * i as f64 / 16.0),
            )
        })
        .collect();
    let shapes = [
        star(0.0, 0.0, 0.06, 0.45, 5, 90.0),
        circle,
        vec![(0.0, 0.06), (-0.06, -0.045), (0.06, -0.045)],
    ];
    for (k, (x, m, outline)) in [
        (-0.25, M.toy_red, &shapes[0]),
        (0.0, M.toy_green, &shapes[1]),
        (0.25, M.toy_blue, &shapes[2]),
    ]
    .into_iter()
    .enumerate()
    {
        fextrude!(
            F,
            format!("tile{k}"),
            &rounded_rect(0.16, 0.15, 0.025, 2),
            0.008,
            (x, 0.24, 0.072),
            M.toybox_inside,
            bevel = 0.003,
            bevel_segments = 1
        );
        fextrude!(
            F,
            format!("shape{k}"),
            outline,
            0.012,
            (x, 0.24, 0.074),
            m,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
    toys_inside(0.46, z);
    still("body", None);
    // blocks left out on the floor in front: drawn with the toy box, not part of its size (the box is 0.5 m deep)
    let carried = new_parts(|| {
        block(
            "floor_block0",
            0.32,
            0.0,
            0.25,
            0.1,
            M.toy_green,
            M.toy_yellow,
            0.0,
            0.0,
        );
        block(
            "floor_block1",
            0.2,
            0.0,
            0.3,
            0.1,
            M.toy_red,
            M.paper,
            30.0,
            0.0,
        );
        block(
            "floor_block2",
            0.27,
            0.1,
            0.27,
            0.1,
            M.toy_blue,
            M.toy_yellow,
            12.0,
            0.0,
        );
    });
    aside(&carried, "blocks");
    let parts = new_parts(|| {
        // the lid: a rounded board, a finger notch in its front edge, a piano hinge along its back
        let notch = (0..9).map(|i| {
            (
                0.05 - 0.1 * i as f64 / 8.0,
                0.25 - 0.022 * sin(PI * i as f64 / 8.0),
            )
        });
        let mut outline = vec![(0.41, -0.258), (0.41, 0.25)];
        outline.extend(notch);
        outline.extend([(-0.41, 0.25), (-0.41, -0.258)]);
        fextrude!(
            F,
            "lid",
            &outline,
            0.042,
            (0, 0.486, z),
            M.toybox_lid,
            plane = Facing::Top,
            bevel = 0.012,
            bevel_segments = 2
        );
        fextrude!(
            F,
            "lid_panel",
            &rounded_rect(0.66, 0.36, 0.03, 3),
            0.006,
            (0, 0.508, z - 0.01),
            M.toybox_lid,
            plane = Facing::Top,
            bevel = 0.003,
            bevel_segments = 1
        );
        fcyl!(
            F,
            "hinge",
            0.006,
            0.7,
            (0, 0.462, z - 0.252),
            M.handle,
            rot = (0, 90, 0),
            verts = 8
        );
    });
    let lid = hinged(join(&parts, "lid", None), F.at(0, 0.46, z - 0.26));
    place(0.0);
    swing!(lid, "lid_open", -100.0, seconds = 1.0, axis = Axis::X);
    meshes()
}

/// A domed lid's cross-section seen from the side (game depth, height): flat from z0 to z1 at y0, arched `rise`
/// over it.
fn arch(z0: f64, z1: f64, y0: f64, rise: f64, n: usize) -> Vec<(f64, f64)> {
    let mut pts = vec![(z0, y0), (z1, y0)];
    for i in 0..=n {
        let a = PI * i as f64 / n as f64;
        pts.push((
            (z0 + z1) / 2.0 + (z1 - z0) / 2.0 * cos(a),
            y0 + 0.012 + (rise - 0.012) * sin(a),
        ));
    }
    pts
}

pub fn toy_chest() -> Vec<Obj> {
    let z = -0.18;
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            bun(
                &format!("foot{sx}{sz}"),
                sx as f64 * 0.36,
                z + sz as f64 * 0.19,
                0.045,
                0.03,
                M.chest_trim,
                8,
            );
        }
    }
    fbox!(
        F,
        "box_floor",
        0.78,
        0.03,
        0.44,
        (0, 0.055, z),
        M.chest_inside
    );
    walls(
        "box_walls",
        0.8,
        0.46,
        0.04,
        0.44,
        z,
        0.025,
        M.chest,
        0.012,
        0.0,
    );
    fbox!(F, "fill", 0.75, 0.01, 0.41, (0, 0.29, z), M.chest_inside);
    // planks: grooves across the front and the ends
    for y in [0.17, 0.3] {
        fbox!(
            F,
            format!("groove_front{}", r(y)),
            0.79,
            0.006,
            0.004,
            (0, y, z + 0.23),
            M.chest_inside
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("groove_end{}{s}", r(y)),
                0.004,
                0.006,
                0.45,
                (s as f64 * 0.4, y, z),
                M.chest_inside
            );
        }
    }
    // the bands and straps, iron corners, a bail handle on each end
    fbox!(
        F,
        "band",
        0.81,
        0.05,
        0.012,
        (0, 0.33, z + 0.235),
        M.chest_trim,
        bevel = 0.004,
        bevel_segments = 1
    );
    for s in [-1, 1] {
        let sf = s as f64;
        fbox!(
            F,
            format!("strap{s}"),
            0.05,
            0.4,
            0.012,
            (sf * 0.3, 0.24, z + 0.235),
            M.chest_trim,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("band_side{s}"),
            0.012,
            0.05,
            0.47,
            (sf * 0.405, 0.33, z),
            M.chest_trim,
            bevel = 0.004,
            bevel_segments = 1
        );
        // iron corner plates on the front
        for (k, (y0, dy)) in [(0.04, 0.075), (0.44, -0.075)].into_iter().enumerate() {
            fextrude!(
                F,
                format!("corner{s}{k}"),
                &[
                    (sf * 0.4, y0),
                    (sf * 0.4 - sf * 0.065, y0),
                    (sf * 0.4, y0 + dy)
                ],
                0.004,
                (0, 0, z + 0.232),
                M.handle
            );
        }
        fbox!(
            F,
            format!("handle_plate{s}"),
            0.006,
            0.05,
            0.12,
            (sf * 0.403, 0.4, z),
            M.handle,
            bevel = 0.003,
            bevel_segments = 1
        );
        let pts = artkit::kit::fillet(
            &[
                (sf * 0.41, 0.4, z - 0.045),
                (sf * 0.416, 0.355, z - 0.045),
                (sf * 0.416, 0.355, z + 0.045),
                (sf * 0.41, 0.4, z + 0.045),
            ],
            0.012,
            3,
        );
        fsweep!(F, format!("handle{s}"), &pts, 0.006, M.handle, verts = 6);
    }
    toys_inside(0.44, z);
    still("body", None);
    let parts = new_parts(|| {
        // the lid, domed from back to front, bound with straps over the dome, a hasp and lock plate at its front
        fextrude!(
            F,
            "lid",
            &arch(z - 0.25, z + 0.25, 0.44, 0.085, 10),
            0.84,
            (0, 0, 0),
            M.chest,
            plane = Facing::Side,
            bevel = 0.01,
            bevel_segments = 1
        );
        for (k, x) in [-0.3, 0.3, -0.416, 0.416].into_iter().enumerate() {
            let w = if k < 2 { 0.05 } else { 0.012 };
            let band: Vec<P> = (0..11)
                .map(|i| {
                    (
                        x,
                        0.452 + 0.077 * sin(PI * i as f64 / 10.0),
                        z + 0.246 * cos(PI * i as f64 / 10.0),
                    )
                })
                .collect();
            fsweep!(
                F,
                format!("lid_strap{k}"),
                &band,
                0,
                M.chest_trim,
                profile = rounded_rect(w, 0.008, 0.002, 1),
                up = (1, 0, 0)
            );
        }
        fbox!(
            F,
            "lid_band",
            0.84,
            0.03,
            0.012,
            (0, 0.455, z + 0.249),
            M.chest_trim,
            bevel = 0.004,
            bevel_segments = 1
        );
        fextrude!(
            F,
            "latch",
            &rounded_rect(0.06, 0.075, 0.012, 2),
            0.008,
            (0, 0.44, z + 0.257),
            M.handle,
            bevel = 0.002,
            bevel_segments = 1
        );
        fextrude!(
            F,
            "keyhole",
            &[
                (0.0, 0.012),
                (0.007, 0.006),
                (0.004, -0.018),
                (-0.004, -0.018),
                (-0.007, 0.006)
            ],
            0.003,
            (0, 0.43, z + 0.2615),
            M.eye
        );
    });
    let lid = hinged(join(&parts, "lid", None), F.at(0, 0.44, z - 0.25));
    place(0.0);
    swing!(lid, "lid_open", -100.0, seconds = 1.0, axis = Axis::X);
    meshes()
}
