//! KD2: the play mat.

use super::common::{block, heart, moon, place, star, still};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::math::{TAU, cos, radians, sin};
use artkit::motion::{Grow, grow};
use artkit::parts::{join, new_parts};
use artkit::place::Facing;
use artkit::py::hypot;
use artkit::scene::{Obj, meshes};
use artkit::{fextrude, fsphere, sphere};

/// Each foam tile's side; four interlock in the middle of the mat.
const MAT_TILE: f64 = 0.44;

/// An edge from p0 to p1 (game x, depth) with a jigsaw knob in its middle bulging to `side` (+1: left of the way
/// along it, -1: right), as foam tiles lock.
fn jigsaw(p0: (f64, f64), p1: (f64, f64), side: f64) -> Vec<(f64, f64)> {
    let ((x0, z0), (x1, z1)) = (p0, p1);
    let l = hypot(x1 - x0, z1 - z0);
    let (ux, uz) = ((x1 - x0) / l, (z1 - z0) / l);
    let (nx, nz) = (-uz * side, ux * side);
    let at = |t: f64, o: f64| (x0 + ux * t + nx * o, z0 + uz * t + nz * o);
    let (m, neck, r, out) = (l / 2.0, 0.02, 0.032, 0.04);
    let mut pts = vec![at(0.0, 0.0), at(m - neck, 0.0)];
    for i in 0..9 {
        // round the knob, from its near side over its far end to the other side
        let a = radians(220.0 - 260.0 * i as f64 / 8.0);
        pts.push(at(m + r * cos(a), out + r * sin(a)));
    }
    pts.extend([at(m + neck, 0.0), at(l, 0.0)]);
    pts
}

pub fn play_mat() -> Vec<Obj> {
    // four foam tiles locking in the middle, each with a shape pressed out of it, on a backing in the mat's colour
    fextrude!(
        F,
        "mat",
        &rounded_rect(0.92, 0.92, 0.02, 2),
        0.012,
        (0, 0.006, 0),
        M.mat,
        plane = Facing::Top,
        bevel = 0.003,
        bevel_segments = 1
    );
    let t = MAT_TILE;
    // the edges along depth 0, out from the middle, and along x 0
    let ez = |sx: f64| jigsaw((0.0, 0.0), (sx * t, 0.0), sx);
    let ex = |sz: f64| jigsaw((0.0, 0.0), (0.0, sz * t), -sz);
    let circle: Vec<(f64, f64)> = (0..16)
        .map(|i| {
            (
                0.08 * cos(TAU * i as f64 / 16.0),
                0.08 * sin(TAU * i as f64 / 16.0),
            )
        })
        .collect();
    let tiles = [
        (
            (-1.0, -1.0),
            M.mat_yellow,
            star(0.0, 0.0, 0.09, 0.45, 5, 90.0),
            M.mat_pink,
        ),
        (
            (1.0, -1.0),
            M.mat_pink,
            heart(0.0, 0.0, 0.075, 20),
            M.mat_yellow,
        ),
        (
            (-1.0, 1.0),
            M.mat_green,
            moon(0.0, 0.0, 0.08, 10),
            M.mat_white,
        ),
        ((1.0, 1.0), M.mat_white, circle, M.mat_green),
    ];
    for ((sx, sz), m, mark, mark_colour) in tiles {
        let mut outline = ez(sx);
        outline.push((sx * t, sz * t));
        let back: Vec<(f64, f64)> = ex(sz).into_iter().rev().collect();
        outline.extend_from_slice(&back[..back.len() - 1]);
        fextrude!(
            F,
            format!("tile{sx}{sz}"),
            &outline,
            0.01,
            (0, 0.017, 0),
            m,
            plane = Facing::Top
        );
        let (cx, cz) = (sx * t / 2.0, sz * t / 2.0);
        let shape: Vec<(f64, f64)> = mark.iter().map(|&(u, v)| (cx + u, cz - v)).collect();
        fextrude!(
            F,
            format!("mark{sx}{sz}"),
            &shape,
            0.004,
            (0, 0.0225, 0),
            mark_colour,
            plane = Facing::Top
        );
    }
    let y = 0.022;
    let blocks = [
        ((-0.3, -0.15), M.toy_red, M.toy_yellow, 10.0),
        ((-0.18, -0.24), M.toy_blue, M.paper, 35.0),
        ((0.26, 0.14), M.toy_green, M.toy_yellow, 50.0),
        ((-0.28, 0.24), M.toy_yellow, M.toy_red, 70.0),
    ];
    for (k, ((x, z), m, face, r)) in blocks.into_iter().enumerate() {
        block(&format!("block{k}"), x, y, z, 0.09, m, face, r, 0.0);
    }
    block(
        "block_top",
        -0.29,
        y + 0.09,
        -0.16,
        0.09,
        M.toy_green,
        M.paper,
        25.0,
        0.0,
    );
    sphere!(
        "ball",
        0.06,
        at = F.at(0.3, y + 0.06, -0.25),
        material = M.ball_red,
        segments = 14,
        rings = 8
    );
    fsphere!(
        F,
        "ball_band",
        0.0605,
        (0.3, y + 0.06, -0.25),
        M.ball_yellow,
        scale = (1, 0.3, 1),
        rot = (0, 30, 0),
        segments = 14,
        rings = 4
    );
    still("body", None);
    // the tower the resident builds: three blocks, grown up from the mat
    let parts = new_parts(|| {
        for (k, (m, face, r)) in [
            (M.toy_blue, M.toy_yellow, 0.0),
            (M.toy_yellow, M.toy_red, 18.0),
            (M.toy_red, M.paper, -12.0),
        ]
        .into_iter()
        .enumerate()
        {
            block(
                &format!("tower{k}"),
                0.04,
                y + 0.09 * k as f64,
                0.12,
                0.09,
                m,
                face,
                r,
                0.0,
            );
        }
    });
    // squeezed into the mat until it grows
    let tower = join(&parts, "tower", Some(F.at(0.04, 0.012, 0.12)));
    tower.edit(|o| o.scale.z = 0.001);
    place(0.03); // its back on the wall's face
    grow(
        tower,
        "blocks",
        Grow {
            seconds: 1.5,
            ..Grow::default()
        },
    );
    meshes()
}
