//! The planter box (1x1): a plank trough on feet, full of leaves and flowers.

use super::common::{F, own};
use artkit::far::{is_low, low};
use artkit::json::float_repr;
use artkit::kit::{Moulding, moulding};
use artkit::math::{P3, TAU, V3, cos, degrees, sin, v3};
use artkit::parts::{Settle, join_all, measure, settle_with};
use artkit::place::Facing;
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{dome, fbox, fcyl, leaf, lump, star};

/// The depth it is centred on: the trough 10 cm back (as the old part's), leaves over its front.
pub const PLANTER_C: f64 = -0.09;

pub fn planter_box() -> Vec<Obj> {
    let m = own("planterBox");
    let mut rng = Random::new(58);
    let z0 = -0.1; // the old part's trough stands 10 cm back on its tile
    let (w, d, h) = (0.9, 0.42, 0.44);
    let full = !is_low();
    // corner posts, their feet chamfered off the ground
    for i in [-1, 1] {
        for j in [-1, 1] {
            let (s, e) = (i as f64, j as f64);
            fbox!(
                F,
                format!("leg{i}{j}"),
                0.06,
                h,
                0.06,
                (s * (w / 2.0 - 0.03), h / 2.0, z0 + e * (d / 2.0 - 0.03)),
                m.box_trim,
                bevel = low(0.008, 0.0),
                bevel_segments = 1
            );
        }
    }
    // three planks a side, a little gap between
    for k in 0..3 {
        let y = 0.07 + k as f64 * 0.125 + 0.06;
        for j in [-1, 1] {
            let e = j as f64;
            fbox!(
                F,
                format!("plank{k}{j}"),
                w - 0.1,
                0.115,
                0.03,
                (0, y, z0 + e * (d / 2.0 - 0.035)),
                m.body,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
        for i in [-1, 1] {
            let s = i as f64;
            fbox!(
                F,
                format!("end{k}{i}"),
                0.03,
                0.115,
                d - 0.1,
                (s * (w / 2.0 - 0.035), y, z0),
                m.body,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
    }
    fbox!(F, "floor", w - 0.08, 0.02, d - 0.08, (0, 0.08, z0), m.body);
    // a batten down the middle of each long side, screwed to the planks
    for j in [-1, 1] {
        let e = j as f64;
        fbox!(
            F,
            format!("batten{j}"),
            0.06,
            h - 0.1,
            0.02,
            (0, 0.07 + (h - 0.1) / 2.0, z0 + e * (d / 2.0 - 0.01)),
            m.box_trim,
            bevel = low(0.005, 0.0),
            bevel_segments = 1
        );
        if full {
            for k in 0..3 {
                fcyl!(
                    F,
                    format!("screw{j}{k}"),
                    0.006,
                    0.004,
                    (0, 0.13 + k as f64 * 0.125, z0 + e * (d / 2.0 + 0.001)),
                    m.bracket,
                    rot = (90, 0, 0),
                    verts = 6
                );
            }
        }
    }
    // the cap rail: one moulded frame round the top, its corners mitred, its outer edge rounded over
    let (hw, hd) = (w / 2.0 + 0.02, d / 2.0 + 0.02);
    let rail = [(hw, z0 - hd), (hw, z0 + hd), (-hw, z0 + hd), (-hw, z0 - hd)];
    moulding(
        &F,
        "rim",
        &rail,
        &[
            (0.0, 0.0),
            (0.0, low(0.02, 0.03)),
            (low(0.008, 0.0), 0.03),
            // (its inner arris eased too)
            (low(0.064, 0.07), 0.03),
            (0.07, low(0.024, 0.03)),
            (0.07, 0.0),
        ],
        m.box_trim,
        Moulding {
            at: h,
            plane: Facing::Top,
            ..Moulding::default()
        },
    );
    // steel corner brackets on the front, bolted through
    if full {
        for i in [-1, 1] {
            let s = i as f64;
            let (x, zf) = (s * (w / 2.0 - 0.03), z0 + d / 2.0 + 0.002);
            for y in [0.12, h - 0.06] {
                let at = float_repr(y);
                fbox!(
                    F,
                    format!("bracket{i}{at}"),
                    0.075,
                    0.06,
                    0.004,
                    (x, y, zf),
                    m.bracket
                );
                fbox!(
                    F,
                    format!("bracket_side{i}{at}"),
                    0.004,
                    0.06,
                    0.05,
                    (s * (w / 2.0 + 0.002), y, zf - 0.025),
                    m.bracket
                );
                for b in [-1, 1] {
                    fcyl!(
                        F,
                        format!("bolt{i}{at}{b}"),
                        0.006,
                        0.005,
                        (x + b as f64 * 0.022, y, zf + 0.003),
                        m.bracket,
                        rot = (90, 0, 0),
                        verts = 6
                    );
                }
            }
        }
    }
    fbox!(
        F,
        "soil",
        w - 0.08,
        0.03,
        d - 0.08,
        (0, h - 0.025, z0),
        m.soil
    );
    // leafy clumps along the box, flowers over them, and a few leaves trailing over the front
    let greens = [m.leaf, m.leaf_dark, m.leaf_light];
    let mut clumps: Vec<(Obj, V3)> = Vec::new();
    // two staggered rows of leafy clumps, overlapping into one mass
    for i in 0..12 {
        let row = i % 2;
        let x = -0.36 + i as f64 * 0.065 + rng.uniform(-0.015, 0.015);
        let zz = z0 + (if row != 0 { 0.065 } else { -0.06 }) + rng.uniform(-0.02, 0.02);
        let r = 0.075 * rng.uniform(0.9, 1.15);
        let clump = lump!(
            format!("clump{i}"),
            r,
            F.at(x, h - 0.04, zz),
            greens[i % 3],
            scale = (1.25, 1.15, 1.2),
            seed = i as f64 * 1.3,
            noise = 0.32
        );
        clumps.push((clump, F.at(x, 0, zz).into()));
    }
    // flowers sitting on the clumps, leaning outwards with the leaves under them, a yellow eye in each
    let cols = [m.bloom, m.bloom2, m.bloom3, m.bloom4, m.bloom5];
    for i in 0..22 {
        let (clump, c) = clumps[(i * 5) % 12];
        let (a, rr) = (rng.uniform(0.0, TAU), rng.uniform(0.0, 0.06));
        let mut at = c + v3(rr * sin(a), rr * cos(a), 0);
        let near = clump.read(|o| {
            o.mesh
                .verts
                .iter()
                .filter(|&&v| (v - at).xy_length() < 0.03)
                .map(|v| v.z as f64)
                .fold(None, |top: Option<f64>, z| {
                    Some(top.map_or(z, |t| t.max(z)))
                })
        });
        at.z = (near.unwrap_or(h + 0.15) - 0.006) as f32;
        star!(
            format!("flower{i}"),
            P3::from(at),
            0.032,
            5,
            cols[i % 5],
            yaw = degrees(a),
            tilt = rr / 0.06 * 50.0,
            inner = 0.6,
            cup = 0.15
        );
        if full && i % 2 == 0 {
            dome!(
                format!("eye{i}"),
                P3::from(at + v3(0, 0, 0.003)),
                0.008,
                0.005,
                m.eye,
                yaw = degrees(a),
                tilt = rr / 0.06 * 50.0
            );
        }
    }
    for (i, x) in [-0.3, -0.1, 0.12, 0.3].into_iter().enumerate() {
        for k in 0..2 {
            let size = rng.uniform(0.12, 0.16);
            if is_low() && k != 0 {
                continue;
            }
            leaf!(
                format!("trail{i}_{k}"),
                F.at(x + k as f64 * 0.04, h + 0.03, z0 + d / 2.0 - 0.035),
                (180 + (k as i32 * 2 - 1) * 20) as f64,
                30.0,
                size,
                0.05,
                [m.leaf_light, m.leaf][k],
                shape = "lance",
                droop = 1.1,
                fold = 0.2,
                rows = 5,
                ring = 3
            );
        }
    }
    let planter = join_all("planterBox");
    // centred on PLANTER_C deep, the far version too (its leaves are coarser)
    let (lo, hi) = measure(&[planter]);
    planter.edit(|o| o.loc.y = (o.loc.y as f64 + (-PLANTER_C - (lo[1] + hi[1]) / 2.0)) as f32);
    settle_with(
        &[planter],
        Settle {
            centre: false,
            floor: true,
        },
    )
}
