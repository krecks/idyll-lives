//! The canopy bed.

use super::beds::DOUBLE_TOP;
use super::common::{
    Bedding, Board, F, M, Mattress, WALL, bedding, board, finish, mattress, pillow_at,
};
use artkit::forms::rounded_rect;
use artkit::math::{P3, PI, TAU, cos, sin};
use artkit::place::N;
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsweep, loft, revolve};

// ---------------------------------------------------------------- the double bed (`bed`)

// ---------------------------------------------------------------- the four-poster

/// A ring of `n` points (Blender x, y about 0) round a gathered curtain's cross-section: `width` along the bed's side
/// (Blender y), `depth` across it, in `folds` soft pleats.
fn folds(k: f64, n: usize, depth: f64, width: f64, folds: i32) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let a = TAU * i as f64 / n as f64;
            let r = 1.0 + k * sin(folds as f64 * a);
            (depth / 2.0 * cos(a) * r, width / 2.0 * sin(a) * r)
        })
        .collect()
}

/// A curtain gathered at a post, hanging from the top rail down the side: pleated, wide at the rail, drawn in by a
/// tie-back a metre up and spreading again to the floor. `inward`: the depth direction along the side, away from
/// the post.
fn drape(name: &str, x: f64, z: f64, inward: i32) {
    let b = F.at(x, 0, z);
    let (bx, by) = (b.x, b.y);
    let shift = -inward as f64 * 0.05; // Blender y: its middle a little along the side from the post
    let secs: Vec<Vec<P3>> = [
        (0.12, 0.16, 0.06),
        (0.75, 0.11, 0.05),
        (1.0, 0.06, 0.045),
        (1.25, 0.1, 0.045),
        (1.6, 0.14, 0.04),
        (2.05, 0.16, 0.03), // its head in the top rail, where it hangs from
    ]
    .iter()
    .map(|&(y, width, depth)| {
        folds(0.1, 20, depth, width, 5)
            .iter()
            .map(|&(u, v)| P3 {
                x: bx + u,
                y: by + shift + v,
                z: y,
            })
            .collect()
    })
    .collect();
    // (gathered cloth turns round at its edges: shaded so, and laid flat on its folds, not round its middle)
    let c = loft!(
        format!("{name}_curtain"),
        &secs,
        material = M.canopy,
        angle = 150
    );
    c.edit(|p| p.uv = artkit::mesh::UvWay::Box);
    let tie: Vec<(f64, f64, f64)> = (0..10)
        .map(|i| {
            (
                x + 0.03 * cos(TAU * i as f64 / 10.0),
                1.0,
                z + inward as f64 * 0.05 + 0.04 * sin(TAU * i as f64 / 10.0),
            )
        })
        .collect();
    fsweep!(
        F,
        format!("{name}_tie"),
        &tie,
        0.009,
        M.canopy,
        verts = 4,
        closed = true
    );
}

/// A turned post `h` tall, about `r` thick: a square-ish foot block below, then a vase, rings and a long shaft up to
/// a finial.
fn turned_post(name: &str, x: f64, z: f64, h: f64, r: f64, material: artkit::materials::Mat) {
    let prof = [
        (0.0, 0.12),
        (r * 1.15, 0.12),
        (r * 0.8, 0.16),
        (r * 1.15, 0.25),
        (r * 1.25, 0.33),
        (r * 0.85, 0.42),
        (r * 0.65, 0.46),
        (r * 1.0, 0.49),
        (r * 1.0, 0.52),
        (r * 0.62, 0.55),
        (r * 0.7, 1.05),
        (r * 0.55, h - 0.18),
        (r * 0.9, h - 0.155),
        (r * 0.9, h - 0.135),
        (r * 0.5, h - 0.115),
        (r * 0.5, h - 0.075),
        (r * 0.85, h - 0.045),
        (r * 0.6, h - 0.012),
        (0.0, h),
    ];
    revolve!(
        name,
        &prof,
        at = F.at(x, 0, z),
        material = material,
        segments = 10
    );
    fbox!(
        F,
        format!("{name}_foot"),
        2.0 * r * 1.28,
        0.12,
        2.0 * r * 1.28,
        (x, 0.06, z),
        material,
        bevel = 0.012,
        bevel_segments = 1
    );
}

/// A valance's outline `w` across and `h` deep, hanging from its top (0) in `n` scallops.
fn scallops(w: f64, h: f64, n: usize) -> Vec<(f64, f64)> {
    let mut bottom: Vec<(f64, f64)> = (0..=n * 4)
        .map(|i| {
            let u = w / 2.0 - w * i as f64 / (n * 4) as f64;
            (u, -h + 0.04 * sin(PI * i as f64 / 4.0).abs())
        })
        .collect();
    bottom.reverse();
    let mut out = vec![(w / 2.0, 0.0), (-w / 2.0, 0.0)];
    out.extend(bottom);
    out
}

/// The four-poster.
pub fn bed_canopy() -> Vec<Obj> {
    let top = DOUBLE_TOP;
    // the posts' depths: the head end's feet, crest and valance stand on the wall's face
    let posts = |s: i32| if s < 0 { WALL + 0.045 } else { 0.92 };
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            turned_post(
                &format!("post{sx}{sz}"),
                sx as f64 * 0.82,
                posts(sz),
                2.135,
                0.035,
                M.frame_walnut,
            );
        }
        fbox!(
            F,
            format!("rail_top{sx}"),
            0.06,
            0.06,
            1.77,
            (sx as f64 * 0.82, 2.06, 0),
            M.frame_walnut,
            bevel = 0.012,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("rail_end{sx}"),
            1.57,
            0.06,
            0.06,
            (0, 2.06, posts(sx)),
            M.frame_walnut,
            bevel = 0.012,
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        "frame",
        1.6,
        0.22,
        1.78,
        (0, 0.21, 0),
        M.frame_walnut,
        bevel = 0.016,
        bevel_segments = 2
    );
    fbox!(
        F,
        "frame_bead",
        1.612,
        0.016,
        1.792,
        (0, 0.29, 0),
        M.frame_walnut,
        bevel = 0.006
    );
    // a headboard of three raised panels in a frame, under an arched crest rail
    let holes: Vec<Vec<(f64, f64)>> = [-0.52, 0.0, 0.52]
        .iter()
        .map(|&x| {
            rounded_rect(0.42, 0.4, 0.02, 2)
                .iter()
                .map(|&(u, v)| (x + u, v + 0.02))
                .collect()
        })
        .collect();
    fextrude!(
        F,
        "headboard",
        &rounded_rect(1.57, 0.6, 0.004, 1),
        0.04,
        (0, 0.68, WALL + 0.02),
        M.headboard_walnut,
        holes = holes,
        bevel = 0.006,
        bevel_segments = 1
    );
    for (i, x) in [N::F(-0.52), N::I(0), N::F(0.52)].into_iter().enumerate() {
        fbox!(
            F,
            format!("panel{i}"),
            0.418,
            0.398,
            0.024,
            (x, 0.7, WALL + 0.02),
            M.headboard_walnut,
            bevel = 0.009,
            bevel_segments = 1
        );
    }
    board(
        "crest",
        1.64,
        0.965,
        1.1,
        WALL + 0.035,
        0.07,
        M.headboard_walnut,
        Board {
            arch: 0.065,
            bevel: 0.012,
            n: 12,
            ..Board::default()
        },
    );
    // the canopy: a scalloped linen valance round the open top (sleepers stay in view from above), a curtain
    // gathered at each post
    for s in [-1, 1] {
        fextrude!(
            F,
            format!("valance_side{s}"),
            &scallops(1.77, 0.2, 5),
            0.008, // lined linen, not a board
            (s as f64 * 0.85, 2.09, 0),
            M.canopy,
            plane = artkit::place::Facing::Side
        );
        fextrude!(
            F,
            format!("valance_end{s}"),
            &scallops(1.57, 0.2, 4),
            0.008,
            (0, 2.09, if s > 0 { 0.95 } else { WALL + 0.006 }),
            M.canopy
        );
    }
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            drape(
                &format!("drape{sx}{sz}"),
                sx as f64 * 0.83,
                sz as f64 * 0.8,
                -sz,
            );
        }
    }
    mattress("mattress", 1.56, 0.2, 1.76, top, 0, Mattress::default());
    for s in [-1, 1] {
        pillow_at(
            &format!("pillow{s}"),
            0.69,
            0.14,
            0.34,
            s as f64 * 0.405,
            top,
            -0.68,
        );
    }
    // (the duvet's foot 1.5 cm past the mattress's end, not in its plane)
    let (z_top, z_foot) = (-0.36, 0.87);
    let z_fold = bedding(
        top,
        0.775,
        z_top,
        z_foot,
        0.2,
        M.bedding_rust,
        Bedding::default(),
    );
    finish("bedCanopy", (top, 0.775, z_top, z_fold, M.bedding_rust), 2)
}
