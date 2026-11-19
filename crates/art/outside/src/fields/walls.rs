//! The dry-stone wall and the stone stile: courses of irregular stones, a coping on edge, and two worn steps over
//! the wall.

use super::{F, M};
use artkit::fbox;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::Obj;

/// A course-built wall body, `length` long, `depth` thick, standing at `z`, with the given course heights.
fn courses(name: &str, length: f64, depth: f64, z: f64, heights: &[f64], seed: i128) {
    let mut rng = Random::new(seed);
    let end = length / 2.0 - 0.005;
    let mut y = 0.0;
    for (row, &h) in heights.iter().enumerate() {
        let mut x = -length / 2.0 + 0.005;
        let mut k = 0;
        while x < end - 0.001 {
            let remain = end - x;
            let w = if remain < 0.30 {
                remain
            } else {
                rng.uniform(0.12, 0.38).min(remain)
            };
            if w < 0.05 {
                break;
            }
            let d = depth - rng.uniform(0.0, 0.015);
            let zz = z + (rng.random() - 0.5) * 0.006;
            let pick = rng.random();
            let mat = if pick < 0.55 {
                M.stone
            } else if pick < 0.80 {
                M.stone_light
            } else {
                M.stone_dark
            };
            fbox!(
                F,
                format!("{name}{row}_{k}"),
                w - 0.006,
                h,
                d,
                (x + w / 2.0, y + h / 2.0, zz),
                mat,
                bevel = 0.012,
                bevel_segments = 2
            );
            x += w;
            k += 1;
        }
        y += h;
    }
}

/// The coping: stones set on edge along the top, each a little taller or shorter, so the wall's back is uneven.
fn coping(
    name: &str,
    length: f64,
    depth: f64,
    z: f64,
    y0: f64,
    seed: i128,
    count: usize,
    hmin: f64,
    hmax: f64,
) {
    let mut rng = Random::new(seed);
    let unit = length / count as f64;
    for i in 0..count {
        let h = rng.uniform(hmin, hmax);
        let pick = rng.random();
        let mat = if pick < 0.45 {
            M.stone_light
        } else if pick < 0.80 {
            M.stone
        } else {
            M.stone_dark
        };
        fbox!(
            F,
            format!("{name}{i}"),
            unit - 0.008,
            h,
            depth,
            (
                -length / 2.0 + (i as f64 + 0.5) * unit,
                y0 + h / 2.0,
                z + (rng.random() - 0.5) * 0.006
            ),
            mat,
            bevel = 0.010,
            bevel_segments = 1
        );
    }
}

/// A dry-stone wall: six courses of stones, a coping on edge and a few lichen patches on the lane side.
pub fn dry_stone_wall() -> Vec<Obj> {
    courses(
        "stone",
        2.0,
        0.48,
        0.0,
        &[0.20, 0.18, 0.17, 0.16, 0.14, 0.10],
        41,
    );
    coping("coping", 1.98, 0.48, 0.0, 0.95, 43, 16, 0.14, 0.19);
    // lichen on the front face, in the shelter of the stones
    let mut rng = Random::new(45);
    for i in 0..7 {
        let w = rng.uniform(0.03, 0.07);
        let h = rng.uniform(0.015, 0.035);
        let x = rng.uniform(-0.82, 0.82);
        let y = rng.uniform(0.16, 0.88);
        fbox!(
            F,
            format!("lichen{i}"),
            w,
            h,
            0.005,
            (x, y, 0.245),
            M.lichen,
            bevel = 0.002,
            bevel_segments = 1
        );
    }
    settle(&[join_all("dryStoneWall")])
}

/// A stone stile: a short length of wall with two worn steps on the lane side, so a walker can climb over.
pub fn stile() -> Vec<Obj> {
    courses(
        "stileStone",
        0.95,
        0.45,
        -0.1,
        &[0.20, 0.18, 0.18, 0.17, 0.17],
        47,
    );
    coping("stileCoping", 0.95, 0.48, -0.1, 0.90, 49, 7, 0.12, 0.15);
    // the lower step stands out furthest; the upper one is set back against the wall
    fbox!(
        F,
        "lowerStep",
        0.82,
        0.32,
        0.35,
        (0.0, 0.16, 0.30),
        M.stone_light,
        bevel = 0.018,
        bevel_segments = 2
    );
    fbox!(
        F,
        "upperStep",
        0.82,
        0.30,
        0.35,
        (0.0, 0.47, 0.20),
        M.stone,
        bevel = 0.018,
        bevel_segments = 2
    );
    settle(&[join_all("stile")])
}
