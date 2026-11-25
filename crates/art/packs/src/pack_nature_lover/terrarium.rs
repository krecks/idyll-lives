//! The moss terrarium (1x1): a little forest under a glass cloche, on a walnut pedestal.

use super::common::M;
use artkit::forms::rect;
use artkit::materials::Mat;
use artkit::math::{P3, pow};
use artkit::parts::{aside, join, new_parts, numbered};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{boulder, fbox, fcyl, fsweep, jitter, leaf, loft, revolve, ring, settle, slide};

const TZ: f64 = -0.1; // the pedestal's depth

/// A little fir: a trunk under tiers of drooping branches, turned from a zigzag profile.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn _fir(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, h: f64, r: f64, material: Mat) {
    let tiers = 4;
    let mut prof = vec![(0.0, 0.0)];
    for k in 0..tiers {
        let y0 = 0.15 * h + k as f64 * 0.85 * h / tiers as f64;
        let rr = r * (1.0 - k as f64 / tiers as f64);
        prof.extend([(rr * 0.35, y0 + 0.04 * h), (rr, y0)]);
    }
    prof.push((0.0, h));
    // a lathe a tier (each from its inner point to the next's; the first open below, under its branches), so each
    // tier's texture is laid round at its own radius, not stretched to the whole tree's; its steep side in three
    // bands, each narrowing by the same share (the texture is laid at one radius round a band)
    for k in 0..tiers {
        let (rim, next) = (prof[2 * k + 2], prof[2 * k + 3]);
        let mut tier = prof[2 * k + 1..=2 * k + 2].to_vec();
        tier.extend((1..=3).map(|i| {
            let r = if next.0 > 0.0 {
                rim.0 * pow(next.0 / rim.0, i as f64 / 3.0)
            } else {
                rim.0 * (3 - i) as f64 / 3.0
            };
            let t = (rim.0 - r) / (rim.0 - next.0);
            (r, rim.1 + (next.1 - rim.1) * t)
        }));
        revolve!(
            numbered(name),
            &tier,
            at = f.at(x, y, z),
            material = material,
            segments = 8
        );
    }
    fcyl!(
        f,
        format!("{name}_trunk"),
        0.006,
        0.15 * h + 0.01,
        (x, y + 0.075 * h, z),
        M.cedar_dark,
        verts = 5
    );
}

pub fn moss_terrarium() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        revolve!(
            "pedestal",
            &[
                (0.0, 0.0),
                (0.17, 0.0),
                (0.17, 0.012),
                (0.155, 0.03),
                (0.09, 0.05),
                (0.07, 0.075),
                (0.077, 0.084), // its bead's crown cut across: two turns under the gate's 60°
                (0.0765, 0.0935),
                (0.05, 0.12),
                (0.042, 0.3),
                (0.048, 0.42),
                (0.04, 0.54),
                (0.06, 0.56),
                (0.09, 0.6),
                (0.15, 0.65),
                (0.17, 0.67),
                (0.17, 0.68),
                (0.0, 0.68)
            ],
            at = f.at(0, 0, TZ),
            material = M.walnut,
            segments = 20
        );
        revolve!(
            "plate",
            &[
                (0.0, 0.0),
                (0.19, 0.0),
                (0.2, 0.008),
                (0.2, 0.03),
                (0.192, 0.04),
                (0.185, 0.04),
                (0.185, 0.03),
                (0.0, 0.03)
            ],
            at = f.at(0, 0.68, TZ),
            material = M.walnut,
            segments = 28
        );
        let mound = revolve!(
            "moss_mound",
            &[
                (0.0, 0.71),
                (0.18, 0.71),
                (0.18, 0.74),
                (0.15, 0.765),
                (0.08, 0.79),
                (0.0, 0.8)
            ],
            at = f.at(0, 0, TZ),
            material = M.moss,
            segments = 20
        );
        jitter!(mound, 0.008, size = 0.05, seed = 1.3);
        boulder!(
            "rock",
            0.05,
            at = f.at(0.08, 0.77, TZ + 0.05),
            material = M.pebble,
            seed = 2,
            subdivisions = 1,
            scale = (1.2, 1, 0.8)
        );
        _fir(f, "fir", -0.07, 0.77, TZ - 0.02, 0.24, 0.055, M.leaf_dark);
        _fir(f, "fir_b", 0.06, 0.765, TZ - 0.07, 0.16, 0.04, M.leaf);
        for k in 0..5 {
            // a little fern at the front
            leaf!(
                format!("fern{k}"),
                f.at(-0.06, 0.78, TZ + 0.08),
                (150 + k * 35) as f64,
                40.0,
                0.09,
                0.035,
                M.leaf_light,
                shape = "lance",
                droop = 0.4,
                notch = (2usize, 0.35),
                rows = 6,
                ring = 3
            );
        }
        for (k, (x, z, h)) in [(-0.02, TZ + 0.1, 0.035), (0.01, TZ + 0.12, 0.022)]
            .into_iter()
            .enumerate()
        {
            // toadstools
            fcyl!(
                f,
                format!("toadstool_stem{k}"),
                0.005,
                h,
                (x, 0.77 + h / 2.0, z),
                M.white,
                verts = 6
            );
            revolve!(
                format!("toadstool{k}"),
                &[
                    (0.0, 0.0),
                    (0.022, 0.0),
                    (0.024, 0.006),
                    (0.016, 0.016),
                    (0.0, 0.02)
                ],
                at = f.at(x, 0.77 + h - 0.002, z),
                material = M.toadstool,
                segments = 10
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // the mister on the floor beside it, and a round braided rug: drawn with it, not part of its size
    let floor = new_parts(|| {
        let P3 {
            x: bx,
            y: by,
            z: bz,
        } = f.at(0.24, 0.012, 0.15);
        let sections: Vec<Vec<P3>> = [
            (0.0, 0.9),
            (0.01, 1.0),
            (0.11, 1.0),
            (0.14, 0.45),
            (0.165, 0.42),
        ]
        .iter()
        .map(|&(h, k)| ring!(0.03 * k, 0.03 * k, z = bz + h, verts = 12, at = (bx, by)))
        .collect();
        loft!("mister", &sections, material = M.mister);
        fbox!(
            f,
            "mister_head",
            0.035,
            0.035,
            0.05,
            (0.24, 0.195, 0.16),
            M.white,
            bevel = 0.008,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "mister_trigger",
            &[(0.24, 0.19, 0.18), (0.24, 0.15, 0.19)],
            0.0,
            M.white,
            profile = rect(0.012, 0.008),
            up = (1, 0, 0)
        );
        let mut braid = vec![(0.0, 0.0), (0.21, 0.0), (0.21, 0.006)]; // rings of braid, from the edge in
        for k in (0..=4).rev() {
            braid.extend([
                (0.21 * (k as f64 + 0.5) / 5.0, 0.012),
                (0.21 * k as f64 / 5.0 + 0.002, 0.008),
            ]);
        }
        braid.pop();
        braid.push((0.0, 0.012));
        revolve!(
            "rug",
            &braid,
            at = f.at(0, 0.0, 0.24),
            material = M.rug,
            segments = 28,
            scale = (1, 0.75, 1)
        );
    });
    aside(&floor, "floor");
    // The glass cloche and its knob, resting on the plate: `lid_open` lifts it.
    let lid = new_parts(|| {
        revolve!(
            "cloche",
            &[
                (0.185, 0.0),
                (0.192, 0.006),
                (0.192, 0.015),
                (0.19, 0.23),
                (0.16, 0.33),
                (0.1, 0.39),
                (0.02, 0.41),
                (0.02, 0.4),
                (0.098, 0.384),
                (0.156, 0.326),
                (0.182, 0.23),
                (0.182, 0.01),
                (0.178, 0.0)
            ],
            at = f.at(0, 0.72, TZ),
            material = M.glass,
            segments = 28
        );
        revolve!(
            "cloche_knob",
            &[
                (0.0, 0.0),
                (0.014, 0.0),
                (0.01, 0.012),
                (0.026, 0.026),
                (0.022, 0.044),
                (0.0, 0.048)
            ],
            at = f.at(0, 1.126, TZ),
            material = M.glass,
            segments = 12
        );
    });
    let lid = join(&lid, "cloche", Some(f.at(0, 0.72, TZ)));
    let objs = settle!(centre = false);
    slide!(lid, "lid_open", (0, 0, 0.22), seconds = 0.7);
    objs
}
