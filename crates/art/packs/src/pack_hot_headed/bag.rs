//! The heavy punching bag and the plate-smash bin.

use super::WALL;
use super::common::{M, chain};
use artkit::forms::{rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{Axis, P3, TAU, cos, cosd, radians, sin, sind};
use artkit::motion::{Rock, rock};
use artkit::parts::{aside, hinged, join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::rand::Random;
use artkit::scene::Obj;
use artkit::{bevel::ease, forms::eased, shapes::AUTO_WIDTH};
use artkit::{
    boulder, fbox, fcyl, fextrude, fsphere, fsweep, jitter, kit, settle, soft, swing, torus,
};

// --- the heavy punching bag (1x1): a free-standing frame with a boom, the bag on chains --------------------
const HOOK: (f64, f64, f64) = (0.0, 1.9, 0.1); // the swivel the bag hangs from (x, height, depth)

pub fn heavy_bag() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (hx, hy, hz) = HOOK;
    let sq = rounded_rect(0.07, 0.07, 0.014, 2);
    // the rubber floor mat it stands on: drawn, not the stand's size
    let mat = new_parts(|| {
        fbox!(
            f,
            "mat",
            0.86,
            0.012,
            0.86,
            (0, 0.006, 0),
            M.rubber,
            bevel = 0.004,
            bevel_segments = 1
        );
    });
    aside(&mat, "mat");
    let still = new_parts(|| {
        // the T foot, its ends capped in rubber
        let foot = fsweep!(
            f,
            "foot_long",
            &[(0, 0.042, -0.42), (0, 0.042, 0.335)],
            0,
            M.steel_dark,
            profile = rounded_rect(0.06, 0.08, 0.012, 2),
            up = (0, 0, 1)
        );
        ease(foot, 60.0, 1);
        let foot = fsweep!(
            f,
            "foot_cross",
            &[(-0.35, 0.042, -0.37), (0.35, 0.042, -0.37)],
            0,
            M.steel_dark,
            profile = rounded_rect(0.06, 0.08, 0.012, 2),
            up = (0, 0, 1)
        );
        ease(foot, 60.0, 1);
        for (x, z) in [(0.0, 0.34), (-0.355, -0.37), (0.355, -0.37)] {
            fbox!(
                f,
                numbered("foot_cap"),
                if x == 0.0 { 0.084 } else { 0.012 },
                0.064,
                if x == 0.0 { 0.012 } else { 0.084 },
                (x, 0.042, z),
                M.dark,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        // the upright and the boom bent from one square tube, a gusset in the bend, a round brace to the foot
        fsweep!(
            f,
            "frame",
            &kit::fillet!(
                &[(0.0, 0.07, -0.37), (0.0, 1.975, -0.37), (0.0, 1.975, 0.13)],
                0.07,
                steps = 4
            ),
            0,
            M.steel,
            profile = &sq,
            up = (0, 1, 0)
        );
        for s in [-1.0, 1.0] {
            fextrude!(
                f,
                numbered("gusset"),
                &[(-0.335, 1.94), (-0.2, 1.94), (-0.335, 1.8)],
                0.006,
                (s * 0.038, 0, 0),
                M.steel_dark,
                plane = Facing::Side
            );
            fsphere!(
                f,
                numbered("bolt"),
                0.008,
                (s * 0.041, 1.9, -0.3),
                M.chrome,
                scale = (0.5, 1, 1),
                segments = 6,
                rings = 4
            );
        }
        fbox!(
            f,
            "boom_cap",
            0.074,
            0.074,
            0.008,
            (0, 1.975, 0.134),
            M.dark,
            bevel = 0.003,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "brace",
            &[(0, 0.52, -0.335), (0, 0.075, 0.1)],
            0.02,
            M.steel,
            verts = 10
        );
        for (y, z) in [(0.52, -0.335), (0.075, 0.1)] {
            fbox!(
                f,
                numbered("brace_flange"),
                0.06,
                if y < 0.1 { 0.012 } else { 0.06 },
                if y < 0.1 { 0.06 } else { 0.012 },
                (0, y, z),
                M.steel_dark,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fcyl!(
            f,
            "swivel_eye",
            0.025,
            0.05,
            (hx, 1.93, hz),
            M.chrome,
            verts = 12
        );
        // weight plates on pegs at the foot
        for (s, m) in [(-1, M.fire), (1, M.dark)] {
            kit::tube!(
                f,
                numbered("peg"),
                &[
                    (0.0, 0.072, -0.29),
                    (0.0, 0.17, -0.29),
                    (s as f64 * 0.24, 0.17, -0.29),
                ],
                0.015,
                M.chrome,
                verts = 10,
                round = 1
            );
            // (closed from the middle of its back face, so every corner of it is eased)
            let prof = [
                (0.07, 0.0),
                (0.13, 0.0),
                (0.13, 0.045),
                (0.115, 0.045),
                (0.105, 0.037),
                (0.05, 0.037),
                (0.04, 0.045),
                (0.018, 0.045),
                (0.018, 0.0),
                (0.07, 0.0),
            ];
            kit::turned!(
                f,
                numbered("plate"),
                s as f64 * 0.177 + (s > 0) as i32 as f64 * 0.045,
                0.17,
                -0.29,
                &eased(&prof, AUTO_WIDTH),
                m,
                segments = 22,
                rot = (0.0, 90.0, 0.0)
            );
        }
        // a hook on the upright with a pair of gloves hanging by their laces
        fbox!(
            f,
            "glove_hook",
            0.2,
            0.02,
            0.03,
            (-0.13, 1.32, -0.35),
            M.steel,
            bevel = 0.006,
            bevel_segments = 1
        );
        for x in [-0.24, -0.12] {
            fsphere!(
                f,
                numbered("glove"),
                0.07,
                (x, 1.18, -0.3),
                M.fire,
                scale = (0.9, 0.85, 1.1),
                segments = 14,
                rings = 10
            );
            fsweep!(
                f,
                numbered("glove_thumb"),
                &[
                    (x + 0.04, 1.14, -0.27),
                    (x + 0.062, 1.18, -0.265),
                    (x + 0.05, 1.22, -0.27)
                ],
                0.022,
                M.fire,
                verts = 8,
                resolution = 2,
                scale = [0.9, 1.0, 0.7]
            );
            kit::turned!(
                f,
                numbered("cuff"),
                x,
                1.04,
                -0.3,
                &[
                    (0.0, 0.0),
                    (0.048, 0.0),
                    (0.05, 0.03),
                    (0.045, 0.08),
                    (0.0, 0.08),
                ],
                M.head,
                segments = 12
            );
            fcyl!(
                f,
                numbered("strap"),
                0.052,
                0.026,
                (x, 1.075, -0.3),
                M.dark,
                verts = 12
            );
            kit::tube!(
                f,
                numbered("lace"),
                &[
                    (x, 1.1, -0.29),
                    (x + 0.03, 1.24, -0.31),
                    (-0.13, 1.31, -0.34),
                ],
                0.004,
                M.dark,
                verts = 4,
                round = 2
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The bag, its straps, seams and logo, the chains and swivel: one part hung from the swivel, `swing` rocks it
    // back and forth while it is pounded.
    let bag = new_parts(|| {
        kit::turned!(
            f,
            "bag",
            hx,
            0.66,
            hz,
            &[
                (0.0, 0.0),
                (0.08, 0.002),
                (0.14, 0.018),
                (0.175, 0.05),
                (0.19, 0.1),
                (0.19, 0.9),
                (0.175, 0.95),
                (0.14, 0.982),
                (0.08, 0.998),
                (0.0, 1.0),
            ],
            M.bag,
            segments = 28
        );
        for y in [0.86, 1.5] {
            kit::turned!(
                f,
                numbered("strap"),
                hx,
                y - 0.025,
                hz,
                &[(0.189, 0.0), (0.195, 0.004), (0.195, 0.046), (0.189, 0.05)],
                M.dark,
                segments = 28
            );
        }
        for y in [0.72, 1.6] {
            torus!(
                numbered("seam"),
                0.186,
                0.003,
                at = f.at(hx, y, hz),
                material = M.dark,
                major_segments = 28,
                minor_segments = 3
            );
        }
        let flame = smooth_outline(
            &[
                (0.0, -0.1),
                (0.05, -0.07),
                (0.055, 0.0),
                (0.03, 0.05),
                (0.035, 0.1),
                (0.0, 0.06),
                (-0.02, 0.11),
                (-0.04, 0.04),
                (-0.055, -0.01),
                (-0.045, -0.07),
            ],
            2,
            true,
        );
        fextrude!(f, "logo", &flame, 0.008, (hx, 1.18, hz + 0.188), M.flame);
        kit::turned!(
            f,
            "swivel",
            hx,
            1.84,
            hz,
            &[
                (0.0, 0.0),
                (0.012, 0.0),
                (0.02, 0.012),
                (0.02, 0.05),
                (0.012, 0.06),
                (0.0, 0.06),
            ],
            M.chrome,
            segments = 10
        );
        for k in 0..4 {
            let a = (45 + 90 * k) as f64;
            let ring = (hx + 0.12 * cosd(a), 1.66, hz + 0.12 * sind(a));
            torus!(
                numbered("d_ring"),
                0.014,
                0.003,
                at = f.at(ring.0, ring.1, ring.2),
                material = M.chrome,
                major_segments = 8,
                minor_segments = 3,
                rot = (90.0, 0.0, -a)
            );
            chain(
                f,
                "chain",
                (hx, 1.84, hz),
                (ring.0, ring.1 + 0.012, ring.2),
                M.chrome,
                0.03,
                0.011,
                0.0028,
            );
        }
    });
    let bag = hinged(join(&bag, "bag", None), f.at(hx, hy, hz));
    let objs = settle!(centre = false);
    rock(
        bag,
        "swing",
        8.0,
        Rock {
            seconds: 1.4,
            axis: Axis::X,
        },
    );
    objs
}

// --- the plate-smash bin (1x1): a slatted crate with a lid, a stool stacked with old plates ----------------
const CRATE: (f64, f64, f64, f64, f64) = (-0.08, 0.66, 0.6, WALL, 0.56); // x, width, height, back depth, depth

/// A broken piece of plate lying flat: a jagged outline, its glaze side up.
#[allow(clippy::too_many_arguments)]
fn shard(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    s: f64,
    turn: f64,
    seed: i128,
    material: Mat,
) -> Obj {
    let mut rnd = Random::new(seed);
    let mut pts = Vec::new();
    for k in 0..5 {
        let a = TAU * k as f64 / 5.0 + rnd.uniform(-0.3, 0.3);
        let r = s * rnd.uniform(0.45, 1.0);
        pts.push((r * cos(a), r * sin(a)));
    }
    let o = fextrude!(
        f,
        name,
        &pts,
        0.006,
        (x, y + 0.003, z),
        material,
        plane = Facing::Top
    );
    o.edit(|p| p.rot.z = radians(turn) as f32);
    o
}

pub fn smash_bin() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let mut rnd = Random::new(4);
    let (x, w, h, z0, d) = CRATE;
    let (z1, zc) = (z0 + d, z0 + d / 2.0);
    let still = new_parts(|| {
        fbox!(
            f,
            "mat",
            0.84,
            0.012,
            0.84,
            (0, 0.006, -0.01),
            M.rubber,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            f,
            "floor",
            w - 0.04,
            0.03,
            d - 0.04,
            (x, 0.04, zc),
            M.crate_dark
        );
        let straw = boulder!(
            "straw",
            0.3,
            at = f.at(x, 0.05, zc),
            material = M.straw,
            scale = (1.0, 0.8, 0.5),
            seed = 2.0,
            noise = 0.15,
            subdivisions = 2
        );
        jitter!(straw, 0.012, size = 0.05, seed = 1.0);
        for px in [x - w / 2.0 + 0.03, x + w / 2.0 - 0.03] {
            for pz in [z0 + 0.03, z1 - 0.03] {
                fbox!(
                    f,
                    numbered("post"),
                    0.06,
                    h,
                    0.06,
                    (px, h / 2.0, pz),
                    M.crate_dark,
                    bevel = 0.004,
                    bevel_segments = 2
                );
            }
        }
        let sh = (h - 0.02) / 3.0;
        for k in 0..3 {
            let y = 0.01 + sh * (k as f64 + 0.5);
            for zz in [z1 - 0.011, z0 + 0.011] {
                let dy = rnd.uniform(-0.004, 0.004);
                let ry = rnd.uniform(-0.8, 0.8);
                fbox!(
                    f,
                    numbered("slat"),
                    w - 0.02,
                    sh - 0.025,
                    0.022,
                    (x, y + dy, zz),
                    M.crate_,
                    rot = (0.0, ry, 0.0)
                );
            }
            for xx in [x - w / 2.0 + 0.011, x + w / 2.0 - 0.011] {
                let dy = rnd.uniform(-0.004, 0.004);
                let rx = rnd.uniform(-0.8, 0.8);
                fbox!(
                    f,
                    numbered("slat"),
                    0.022,
                    sh - 0.025,
                    d - 0.02,
                    (xx, y + dy, zc),
                    M.crate_,
                    rot = (rx, 0.0, 0.0)
                );
            }
        }
        // steel corner brackets at the top, the lid's hinges on the back
        for px in [x - w / 2.0 + 0.03, x + w / 2.0 - 0.03] {
            for pz in [z0 + 0.03, z1 - 0.03] {
                fbox!(
                    f,
                    numbered("bracket"),
                    0.062,
                    0.06,
                    0.06,
                    (px + 0.001_f64.copysign(px - x), h - 0.04, pz),
                    M.steel_dark,
                    bevel = 0.002,
                    bevel_segments = 1
                );
            }
        }
        // hazard tape across the front, a stencilled arrow
        fbox!(f, "tape", 0.6, 0.05, 0.004, (x, 0.36, z1 + 0.002), M.dark);
        for i in 0..5 {
            fbox!(
                f,
                numbered("tape_stripe"),
                0.06,
                0.05,
                0.005,
                (x - 0.24 + i as f64 * 0.12, 0.36, z1 + 0.003),
                M.flame,
                rot = (0, 0, 30)
            );
        }
        fextrude!(
            f,
            "arrow",
            &[
                (0.0, 0.05),
                (0.04, 0.0),
                (0.015, 0.0),
                (0.015, -0.05),
                (-0.015, -0.05),
                (-0.015, 0.0),
                (-0.04, 0.0)
            ],
            0.003,
            (x, 0.5, z1 + 0.002),
            M.dark
        );
        // broken plates on the straw and the mat
        for (k, (dx, dz, r)) in [
            (-0.12, -0.05, 20.0),
            (0.1, 0.06, -35.0),
            (0.02, -0.12, 70.0),
            (-0.05, 0.1, 140.0),
        ]
        .into_iter()
        .enumerate()
        {
            shard(
                f,
                &numbered("shard"),
                x + dx,
                0.19 + 0.01 * k as f64,
                zc + dz,
                0.05,
                r,
                k as i128,
                if k % 2 != 0 {
                    M.porcelain
                } else {
                    M.plate_blue
                },
            );
        }
        for (k, (sx, sz, r, sr)) in [
            (-0.3, 0.32, 20.0, 0.04),
            (-0.15, 0.36, -40.0, 0.04),
            (0.06, 0.408, 70.0, 0.05),
            (-0.38, 0.24, 10.0, 0.04),
        ]
        .into_iter()
        .enumerate()
        {
            shard(
                f,
                &numbered("shard"),
                sx,
                0.012,
                sz,
                sr,
                r,
                10 + k as i128,
                [M.porcelain, M.plate_gold][k % 2],
            );
        }
        // the stool and its stack of thrift-store plates
        let (sx, sz) = (0.28, 0.25);
        kit::turned!(
            f,
            "stool_top",
            sx,
            0.43,
            sz,
            &[
                (0.0, 0.0),
                (0.14, 0.0),
                (0.15, 0.008),
                (0.15, 0.024),
                (0.14, 0.03),
                (0.0, 0.03),
            ],
            M.oak,
            segments = 20
        );
        for a in [90.0, 210.0, 330.0] {
            fsweep!(
                f,
                numbered("stool_leg"),
                &[
                    (sx + 0.08 * cosd(a), 0.43, sz + 0.08 * sind(a)),
                    (sx + 0.13 * cosd(a), 0.0, sz + 0.13 * sind(a))
                ],
                0.016,
                M.oak,
                verts = 8,
                scale = [1.0, 0.75]
            );
        }
        torus!(
            "stool_ring",
            0.108,
            0.008,
            at = f.at(sx, 0.15, sz),
            material = M.oak,
            major_segments = 16,
            minor_segments = 4
        );
        let mut y = 0.46;
        for m in [
            M.porcelain,
            M.plate_blue,
            M.porcelain,
            M.plate_gold,
            M.porcelain,
            M.plate_green,
        ] {
            let n = numbered("plate");
            let px = sx + rnd.uniform(-0.008, 0.008);
            let pz = sz + rnd.uniform(-0.008, 0.008);
            kit::turned!(
                f,
                &n,
                px,
                y,
                pz,
                // (the rim a 4 mm band: too narrow for its edges to show hard)
                &[
                    (0.0, 0.0),
                    (0.065, 0.0),
                    (0.12, 0.013),
                    (0.12, 0.017),
                    (0.113, 0.0185),
                    (0.06, 0.006),
                    (0.0, 0.006),
                ],
                m,
                segments = 18
            );
            y += 0.02;
        }
        // safety goggles hung on the crate's front corner post: two lenses in a frame, the strap round the post
        let (gx, gz) = (x + w / 2.0 - 0.03, z1 + 0.035);
        soft!(
            f,
            "goggles",
            0.16,
            0.055,
            0.04,
            (gx + 0.03, 0.5, gz),
            M.flame,
            round = 0.018,
            round_segments = 1,
            flat = 1
        );
        for s in [-1.0, 1.0] {
            fbox!(
                f,
                numbered("lens"),
                0.055,
                0.032,
                0.004,
                (gx + 0.03 + s * 0.037, 0.5, gz + 0.021),
                M.porcelain,
                bevel = 0.01,
                bevel_segments = 1
            );
        }
        kit::tube!(
            f,
            "goggle_strap",
            &[
                (gx + 0.1, 0.505, gz - 0.005),
                (gx + 0.04, 0.56, gz - 0.04),
                (gx - 0.035, 0.56, gz - 0.05),
                (gx - 0.04, 0.505, gz - 0.005),
            ],
            0.006,
            M.dark,
            verts = 4,
            round = 2
        );
    });
    join(&still, "body", Some(P3::ZERO));
    // The lid: three boards on two battens, a rope grip, hinged on the crate's back top edge: `lid_open` flips it up.
    let lz0 = z0 + 0.03;
    let lid = new_parts(|| {
        let bd = (z1 - lz0) / 3.0;
        for k in 0..3 {
            fbox!(
                f,
                numbered("lid_board"),
                w + 0.02,
                0.022,
                bd - 0.006,
                (x, h + 0.011, lz0 + bd * (k as f64 + 0.5)),
                M.crate_,
                bevel = 0.004,
                bevel_segments = 1,
                rot = (0.0, (k - 1) as f64 * 0.6, 0.0)
            );
        }
        for xx in [x - w / 2.0 + 0.08, x + w / 2.0 - 0.08] {
            fbox!(
                f,
                numbered("lid_batten"),
                0.05,
                0.02,
                z1 - lz0 - 0.04,
                (xx, h + 0.032, (lz0 + z1) / 2.0),
                M.crate_dark,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "lid_grip",
            0.16,
            0.03,
            0.035,
            (x, h + 0.015, z1 + 0.015),
            M.crate_dark,
            bevel = 0.01,
            bevel_segments = 1
        );
    });
    let lid = hinged(join(&lid, "lid", None), f.at(x, h, lz0));
    let objs = settle!(centre = false);
    swing!(lid, "lid_open", -80.0, seconds = 0.6, axis = Axis::X);
    objs
}
