//! The dry-stone wall and the stone stile: courses of irregular stones narrowing to the top (the batter), a coping
//! on edge, and the stile's step stones and its oak post.

use super::{F, M};
use artkit::fbox;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::Obj;

/// A course-built wall body, `length` long, standing at depth `z`, `base` thick at the foot and `top` thick at
/// the last course's top (the batter: each course a little narrower than the one under it), with the given course
/// heights. Each stone is its own length, a little lower than its course or set back, so the courses don't read as
/// brick.
fn courses(name: &str, length: f64, base: f64, top: f64, z: f64, heights: &[f64], seed: i128) {
    let mut rng = Random::new(seed);
    let end = length / 2.0 - 0.005;
    let total: f64 = heights.iter().sum();
    let mut y = 0.0;
    for (row, &h) in heights.iter().enumerate() {
        // the course's thickness where its middle is
        let depth = base + (top - base) * (y + h / 2.0) / total;
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
            let d = depth - rng.uniform(0.0, 0.03);
            let zz = z + (rng.random() - 0.5) * 0.006;
            let hs = h - rng.uniform(0.0, 0.04);
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
                w - 0.008,
                hs,
                d,
                (x + w / 2.0, y + hs / 2.0, zz),
                mat,
                bevel = 0.012,
                bevel_segments = 1
            );
            x += w;
            k += 1;
        }
        y += h;
    }
}

/// The coping: `count` stones set on edge along the top, standing at depth `z` on height `y0`, each between
/// `hmin` and `hmax` tall, so the wall's back is uneven.
fn coping(
    name: &str,
    length: f64,
    depth: f64,
    (z, y0): (f64, f64),
    seed: i128,
    (count, hmin, hmax): (usize, f64, f64),
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

/// A dry-stone wall: six courses battered from 0.6 m at the foot to 0.36 m under a coping on edge, and a few lichen
/// patches on the lane side (front).
pub fn dry_stone_wall() -> Vec<Obj> {
    let heights = [0.20, 0.18, 0.17, 0.16, 0.14, 0.10];
    let (base, top) = (0.60, 0.36);
    courses("stone", 2.0, base, top, 0.0, &heights, 41);
    coping("coping", 1.98, top, (0.0, 0.95), 43, (16, 0.14, 0.19));
    // lichen on the front face, in the shelter of the stones: a patch 2.2 cm deep from the face of a stone set
    // back the most (1.5 cm) to 1 mm past one set out the most, so it is on the stone wherever the stone is
    let mut rng = Random::new(45);
    for i in 0..7 {
        let w = rng.uniform(0.03, 0.07);
        let h = rng.uniform(0.015, 0.035);
        let x = rng.uniform(-0.82, 0.82);
        let y = rng.uniform(0.16, 0.88);
        // the course it is on, and that course's thickness, as `courses` laid it
        let (mut below, mut mid) = (0.0, heights[0] / 2.0);
        for &hc in &heights {
            mid = below + hc / 2.0;
            if y < below + hc {
                break;
            }
            below += hc;
        }
        let face = (base + (top - base) * mid / 0.95) / 2.0;
        fbox!(
            F,
            format!("lichen{i}"),
            w,
            h,
            0.022,
            (x, y, face - 0.007),
            M.lichen
        );
    }
    settle(&[join_all("dryStoneWall")])
}

/// A step stile over a low stone wall: the wall (two courses and a coping), an oak post at each end of it standing
/// to hand height, and an oak tread on short legs against each face, a step of 0.25 m up to the coping and 0.25 m
/// down again.
pub fn stile() -> Vec<Obj> {
    let heights = [0.22, 0.20];
    courses("stileStone", 0.95, 0.52, 0.42, 0.0, &heights, 47);
    coping("stileCoping", 0.95, 0.42, (0.0, 0.42), 49, (7, 0.12, 0.15));
    for (side, s) in [("front", 1.0), ("back", -1.0)] {
        // the tread, its two legs under the outer corners
        let zc = s * (0.225 + 0.11);
        fbox!(F, format!("{side}Tread"), 0.50, 0.05, 0.22, (0.0, 0.25, zc), M.oak, bevel = 0.008);
        for (i, x) in [-0.22, 0.22].iter().enumerate() {
            fbox!(
                F,
                format!("{side}Leg{i}"),
                0.06,
                0.225,
                0.06,
                (*x, 0.1125, s * (0.225 + 0.19)),
                M.oak_dark,
                bevel = 0.006
            );
        }
    }
    for (i, x) in [-0.40, 0.40].iter().enumerate() {
        fbox!(F, format!("post{i}"), 0.10, 0.96, 0.10, (*x, 0.48, 0.0), M.oak, bevel = 0.01);
    }
    settle(&[join_all("stile")])
}
