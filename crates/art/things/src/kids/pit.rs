//! KD2: the ball pit.

use super::common::{P, place, still};
use super::{F, M};
use artkit::forms::by;
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, cos, radians, sin};
use artkit::motion::{Squash, squash};
use artkit::parts::{join, new_parts};
use artkit::py::{floordiv, imod, int};
use artkit::scene::{Obj, meshes};
use artkit::{fcyl, fsweep, loft, sphere};

/// The balls' five colours (the palette's materials are made afresh for each model).
fn balls() -> [Mat; 5] {
    [
        M.ball_red,
        M.ball_yellow,
        M.ball_blue,
        M.ball_green,
        M.ball_pink,
    ]
}

fn ball_grid(r_max: f64, step: f64, y: f64, offset: f64) -> Vec<P> {
    let mut pts = Vec::new();
    let rows = int(r_max / (step * 0.866)) + 1;
    for j in -rows..rows + 1 {
        for i in -rows - 1..rows + 2 {
            let x = (i as f64 + 0.5 * imod(j, 2) as f64 + offset) * step;
            let z = j as f64 * step * 0.866;
            if x * x + z * z <= r_max * r_max {
                pts.push((x, y, z));
            }
        }
    }
    pts
}

/// The foam wall's padded channels round the pit.
const PADS: f64 = 12.0;

/// A loop round the pit at height `y`: radius `r`, padded out by `puff` in each channel, creased between.
fn pad_ring(r: f64, y: f64, puff: f64, n: usize) -> Vec<P3> {
    (0..n)
        .map(|i| {
            let i = i as f64;
            let rr = r + puff * sin(PADS / 2.0 * TAU * i / n as f64).abs();
            F.at(
                rr * cos(TAU * i / n as f64),
                y,
                rr * sin(TAU * i / n as f64),
            )
        })
        .collect()
}

pub fn ball_pit() -> Vec<Obj> {
    let balls = balls();
    // the pit: a foam wall in padded channels on a floor, a puffed roll round the top, the balls' level inside
    loft!(
        "pit",
        &[
            pad_ring(0.445, 0.0, 0.0, 48),
            pad_ring(0.452, 0.012, 0.008, 48),
            pad_ring(0.452, 0.33, 0.008, 48),
            pad_ring(0.42, 0.345, 0.0, 48)
        ],
        material = M.pit
    );
    let n = 48;
    let rim: Vec<P> = (0..n)
        .map(|i| {
            (
                0.43 * cos(TAU * i as f64 / n as f64),
                0.36,
                0.43 * sin(TAU * i as f64 / n as f64),
            )
        })
        .collect();
    fsweep!(
        F,
        "rim",
        &rim,
        0.035,
        M.pit_rim,
        verts = 8,
        closed = true,
        scale = by(|w| 0.86 + 0.14 * sin(PADS * PI * w).abs())
    );
    for k in 0..6 {
        let a = radians((60 * k + 15) as f64);
        fcyl!(
            F,
            format!("dot{k}"),
            0.045,
            0.004,
            (0.461 * cos(a), 0.17, 0.461 * sin(a)),
            balls[k % 5],
            rot = (90, 0, (60 * k + 15) as i32 - 90),
            verts = 10
        );
    }
    fcyl!(F, "fill", 0.42, 0.01, (0, 0.335, 0), M.pit_fill, verts = 24);
    let lower = ball_grid(0.37, 0.085, 0.37, 0.0);
    for (k, &(x, y, z)) in lower.iter().enumerate() {
        // jiggled off the grid, so the heap doesn't look laid out
        let kf = k as f64;
        let (jx, jy, jz) = (
            sin(kf * 12.9898) * 0.016,
            sin(kf * 78.233) * 0.012,
            sin(kf * 37.719) * 0.016,
        );
        let colour = balls[imod(k as i64 * 7 + floordiv(k as i64, 3), 5) as usize];
        sphere!(
            format!("ball{k}"),
            0.04,
            at = F.at(x + jx, y + jy, z + jz),
            material = colour,
            segments = 6,
            rings = 4
        );
    }
    still("body", None);
    let parts = new_parts(|| {
        for (k, (x, z)) in [
            (-0.17, -0.1),
            (0.04, 0.2),
            (0.2, -0.12),
            (-0.06, -0.24),
            (0.24, 0.12),
            (-0.22, 0.14),
        ]
        .into_iter()
        .enumerate()
        {
            sphere!(
                format!("top{k}"),
                0.04,
                at = F.at(x, 0.43, z),
                material = balls[k % 5],
                segments = 6,
                rings = 4
            );
        }
    });
    let balls = join(&parts, "balls", Some(P3::ZERO));
    place(0.035); // its back on the wall's face, its front on the tile's edge
    squash(
        balls,
        "bounce",
        (1, 1, 1.18),
        Squash {
            seconds: 0.7,
            back: true,
            ..Squash::default()
        },
    );
    meshes()
}
