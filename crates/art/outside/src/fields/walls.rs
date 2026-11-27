//! The dry-stone wall and the stone stile: courses of irregular stones narrowing to the top (the batter), a coping
//! on edge, and the stile's step stones and its oak post.

use super::{F, M};
use artkit::fbox;
use artkit::parts::{join_all, settle};
use artkit::rand::Random;
use artkit::scene::Obj;

const STONE_W: (f64, f64) = (0.22, 0.55); // a stone's length along the wall, least and most
const LAST_STONE: f64 = 0.30; // what is left of a course below this is one stone: no sliver at the end
const MIN_STONE: f64 = 0.05; // a remainder shorter than this is left a chink
const COURSE_END: f64 = 0.005; // a course starts and ends this far in from the wall's ends
const STONE_SET: f64 = 0.03; // most a stone is set back from the course's face
const STONE_DROP: f64 = 0.06; // most a stone is lower than its course, its gap left dark
const STONE_BEVEL: f64 = 0.022; // a stone's rounded edges
const CAP_WIDTH: (f64, f64) = (0.65, 1.35); // a cap stone's width, as a share of an even one
const COPING_BEVEL: f64 = 0.03; // a cap stone's rounder ones
const LICHEN_REACH: f64 = 0.95; // the height the batter is measured to (the coping's foot)

/// A course-built wall body, `length` long, standing at depth `z`, `base` thick at the foot and `top` thick at
/// the last course's top (the batter: each course a little narrower than the one under it), with the given course
/// heights. Each stone is its own length, a little lower than its course or set back, so the courses don't read as
/// brick.
fn courses(name: &str, length: f64, base: f64, top: f64, z: f64, heights: &[f64], seed: i128) {
    let mut rng = Random::new(seed);
    let end = length / 2.0 - COURSE_END;
    let total: f64 = heights.iter().sum();
    let mut y = 0.0;
    for (row, &h) in heights.iter().enumerate() {
        // the course's thickness where its middle is
        let depth = base + (top - base) * (y + h / 2.0) / total;
        let mut x = -length / 2.0 + COURSE_END;
        let mut k = 0;
        while x < end - 0.001 {
            let remain = end - x;
            let w = if remain < LAST_STONE {
                remain
            } else {
                rng.uniform(STONE_W.0, STONE_W.1).min(remain)
            };
            if w < MIN_STONE {
                break;
            }
            let d = depth - rng.uniform(0.0, STONE_SET);
            let zz = z + (rng.random() - 0.5) * 0.006;
            let hs = h - rng.uniform(0.0, STONE_DROP);
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
                bevel = STONE_BEVEL,
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
    // each cap stone its own width, the row filling the length
    let shares: Vec<f64> = (0..count)
        .map(|_| rng.uniform(CAP_WIDTH.0, CAP_WIDTH.1))
        .collect();
    let unit = length / shares.iter().sum::<f64>();
    let mut x = -length / 2.0;
    for (i, share) in shares.into_iter().enumerate() {
        let w = share * unit;
        x += w;
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
            w - 0.008,
            h,
            depth,
            (x - w / 2.0, y0 + h / 2.0, z + (rng.random() - 0.5) * 0.006),
            mat,
            bevel = COPING_BEVEL,
            bevel_segments = 1
        );
    }
}

/// A dry-stone wall: five courses battered from 0.6 m at the foot to 0.36 m under a coping on edge, and a few lichen
/// patches on the lane side (front).
pub fn dry_stone_wall() -> Vec<Obj> {
    let heights = [0.26, 0.22, 0.20, 0.16, 0.11];
    let (base, top) = (0.60, 0.36);
    courses("stone", 2.0, base, top, 0.0, &heights, 41);
    // the dark heart of the wall, seen in the chinks between stones
    fbox!(
        F,
        "heart",
        1.96,
        0.92,
        top - 0.06,
        (0.0, 0.46, 0.0),
        M.stone_dark
    );
    coping("coping", 1.98, top, (0.0, LICHEN_REACH), 43, (7, 0.12, 0.2));
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
        let face = (base + (top - base) * mid / LICHEN_REACH) / 2.0;
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

/// A step stile in a dry-stone wall: the wall (three courses, 0.75 m) with a flat cap stone on each end pier and a
/// wide flat slab between them, the crossing; two stone steps built up against each face (0.27 m a step) to the
/// slab, and an oak hand post standing from the ground at one pier.
pub fn stile() -> Vec<Obj> {
    let heights = [0.26, 0.25, 0.24];
    courses("stileStone", 0.95, 0.50, 0.44, 0.0, &heights, 47);
    // the dark heart, so the chinks and the gaps under the cap stones show no light through
    fbox!(
        F,
        "stileHeart",
        0.92,
        0.74,
        0.38,
        (0.0, 0.37, 0.0),
        M.stone_dark
    );
    for (i, x) in [-0.36, 0.36].iter().enumerate() {
        fbox!(
            F,
            format!("pierCap{i}"),
            0.23,
            0.10,
            0.46,
            (*x, 0.80, 0.0),
            M.stone_light,
            bevel = COPING_BEVEL,
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        "crossSlab",
        0.5,
        0.07,
        0.5,
        (0.0, 0.775, 0.0),
        M.stone,
        bevel = COPING_BEVEL,
        bevel_segments = 1
    );
    for (side, s) in [("front", 1.0), ("back", -1.0)] {
        fbox!(
            F,
            format!("{side}Step0"),
            0.5,
            0.27,
            0.30,
            (0.0, 0.135, s * 0.30),
            M.stone_light,
            bevel = STONE_BEVEL,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("{side}Step1"),
            0.5,
            0.27,
            0.16,
            (0.0, 0.405, s * 0.24),
            M.stone,
            bevel = STONE_BEVEL,
            bevel_segments = 1
        );
    }
    // the hand post, up through the right pier
    fbox!(
        F,
        "post",
        0.09,
        1.25,
        0.09,
        (0.36, 0.625, 0.0),
        M.oak,
        bevel = 0.012
    );
    settle(&[join_all("stile")])
}
