//! The rainfall lamp (1x1): rain running down lines over a pool, a little figure under an umbrella.

use super::common::M;
use artkit::math::{P3, TAU, cos, cosd, sin, sind};
use artkit::motion::{Grow, Squash, grow, squash};
use artkit::parts::{join, new_parts, numbered};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{boulder, fcyl, fsphere, fsweep, kit, loft, ring, settle, torus};

const LZ: f64 = -0.05; // the lamp's middle (depth)
const POOL: f64 = 0.165;
const CANOPY: f64 = 1.36;

/// A ring of `verts` Blender points of radius `r` at height `y`, pulled in by `dip` (a fraction) and down by `drop`
/// between `lobes` points: a scalloped eave, an umbrella's edge between its ribs.
fn scalloped(
    r: f64,
    y: f64,
    verts: usize,
    at: (f64, f64),
    dip: f64,
    lobes: i32,
    drop: f64,
) -> Vec<P3> {
    (0..verts)
        .map(|k| {
            let a = TAU * k as f64 / verts as f64;
            let f = 0.5 - 0.5 * cos(lobes as f64 * a);
            let rr = r * (1.0 - dip * f);
            P3 {
                x: at.0 + rr * cos(a),
                y: at.1 + rr * sin(a),
                z: y - drop * f,
            }
        })
        .collect()
}

pub fn rain_lamp() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let P3 { x: bx, y: by, .. } = f.at(0, 0, LZ);
    let still = new_parts(|| {
        // the base: a moulded slate bowl round the pool, its lip stepped in
        kit::turned!(
            f,
            "base",
            0.0,
            0.0,
            LZ,
            &[
                (0.0, 0.0),
                (0.205, 0.0),
                (0.21, 0.008),
                (0.205, 0.016),
                (0.196, 0.022),
                (0.196, 0.085),
                (0.206, 0.092),
                (0.21, 0.1),
                (0.205, 0.12),
                (0.19, 0.14),
                (0.172, 0.142),
                (0.17, 0.15),
                (0.0, 0.15),
            ],
            M.slate,
            segments = 40
        );
        fcyl!(
            f,
            "pool",
            0.17,
            0.015,
            (0.0, POOL - 0.0075, LZ),
            M.pool,
            verts = 32
        );
        for k in 0..3 {
            let a = (90 + 120 * k) as f64;
            let (px, pz) = (0.195 * cosd(a), LZ + 0.195 * sind(a));
            fsweep!(
                f,
                numbered("post"),
                &[(px, 0.13, pz), (px, CANOPY + 0.01, pz)],
                0.0075,
                M.slate,
                verts = 8,
                scale = [1.25, 0.9]
            );
            fcyl!(
                f,
                numbered("post_foot"),
                0.014,
                0.02,
                (px, 0.14, pz),
                M.slate,
                verts = 10,
                bevel = 0.004,
                bevel_segments = 1
            );
            for y in [0.55, 0.95] {
                fcyl!(
                    f,
                    numbered("post_collar"),
                    0.011,
                    0.014,
                    (px, y, pz),
                    M.brass,
                    verts = 8,
                    bevel = 0.003,
                    bevel_segments = 1
                );
            }
        }
        // the canopy: a pagoda roof with a scalloped eave, its light underneath (content `light`), a brass finial
        let rings = [
            scalloped(0.218, CANOPY, 48, (bx, by), 0.05, 8, 0.012),
            scalloped(0.22, CANOPY + 0.016, 48, (bx, by), 0.05, 8, 0.01),
            scalloped(0.19, CANOPY + 0.045, 48, (bx, by), 0.02, 8, 0.0),
            ring!(0.11, 0.11, z = CANOPY + 0.075, verts = 48, at = (bx, by)),
            ring!(0.05, 0.05, z = CANOPY + 0.092, verts = 48, at = (bx, by)),
            ring!(0.02, 0.02, z = CANOPY + 0.1, verts = 48, at = (bx, by)),
        ];
        loft!("canopy", &rings, material = M.slate);
        kit::turned!(
            f,
            "finial",
            0.0,
            CANOPY + 0.1,
            LZ,
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.02, 0.008),
                (0.012, 0.015),
                (0.007, 0.02),
                (0.016, 0.028),
                (0.006, 0.036),
                (0.0, 0.038),
            ],
            M.brass,
            segments = 14
        );
        fcyl!(
            f,
            "canopy_under",
            0.2,
            0.004,
            (0.0, CANOPY - 0.002, LZ),
            M.lamp_light,
            verts = 32
        ); // the light (content `light`)
        // the rock in the pool, a cushion of moss on it, the little figure in a coat under an umbrella
        boulder!(
            "rock",
            0.055,
            at = f.at(0, POOL - 0.012, LZ),
            material = M.pebble_dark,
            scale = (1.35, 1.15, 0.58),
            seed = 3.0,
            noise = 0.18,
            subdivisions = 2
        );
        boulder!(
            "moss",
            0.03,
            at = f.at(-0.04, POOL + 0.03, LZ + 0.02),
            material = M.moss,
            scale = (1.4, 1.1, 0.45),
            seed = 5.0,
            noise = 0.25,
            subdivisions = 1
        );
        let (fx, fz) = (0.0, LZ);
        kit::lathe!(
            f,
            "figure",
            fx,
            POOL + 0.03,
            fz,
            &[
                (0.03, 0.0),
                (0.031, 0.01),
                (0.024, 0.05),
                (0.019, 0.085),
                (0.017, 0.1),
                (0.012, 0.108),
                (0.008, 0.112),
            ],
            M.figure,
            verts = 12
        );
        fsphere!(
            f,
            "figure_head",
            0.018,
            (fx, POOL + 0.162, fz),
            M.page,
            segments = 10,
            rings = 6
        );
        fcyl!(
            f,
            "figure_hat",
            0.022,
            0.006,
            (fx, POOL + 0.176, fz),
            M.umbrella,
            verts = 12
        );
        fcyl!(
            f,
            "figure_crown",
            0.013,
            0.014,
            (fx, POOL + 0.185, fz),
            M.umbrella,
            verts = 10
        );
        let (ux, uz) = (0.02, LZ + 0.01);
        kit::tube!(
            f,
            "umbrella_handle",
            &kit::fillet!(
                &[
                    (ux, POOL + 0.26, uz),
                    (ux, POOL + 0.1, uz),
                    (ux + 0.014, POOL + 0.09, uz),
                    (ux + 0.024, POOL + 0.1, uz),
                ],
                0.007
            ),
            0.003,
            M.wick,
            verts = 5,
            round = 1
        );
        let P3 { x: ubx, y: uby, .. } = f.at(ux, 0, uz);
        let uy = POOL + 0.218;
        loft!(
            "umbrella",
            &[
                scalloped(0.09, uy, 32, (ubx, uby), 0.06, 8, 0.006),
                scalloped(0.075, uy + 0.018, 32, (ubx, uby), 0.03, 8, 0.0),
                ring!(0.04, 0.04, z = uy + 0.036, verts = 32, at = (ubx, uby)),
                ring!(0.006, 0.006, z = uy + 0.045, verts = 32, at = (ubx, uby))
            ],
            material = M.umbrella
        );
        fcyl!(
            f,
            "umbrella_tip",
            0.003,
            0.016,
            (ux, uy + 0.05, uz),
            M.wick,
            verts = 5
        );
        // pebbles round the base
        for (i, (x, z, s)) in [
            (0.3, 0.2, 1.0),
            (0.362, 0.08, 0.8),
            (-0.3, 0.22, 1.1),
            (0.22, 0.3, 0.7),
            (-0.358, 0.08, 0.9),
            (-0.22, 0.324, 0.6),
        ]
        .into_iter()
        .enumerate()
        {
            boulder!(
                numbered("pebble"),
                0.04 * s,
                at = f.at(x, 0, z),
                material = [M.pebble, M.pebble_dark][i % 2],
                scale = (1.3, 0.95, 0.55),
                rot = (0, 0, if i == 1 || i == 4 { 0 } else { 30 * i as i32 }),
                seed = 1.3 * i as f64,
                noise = 0.14,
                subdivisions = 2
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The rain: lines from the canopy to the pool, built full length and squashed to nothing under the canopy:
    // `rain` runs them down while a resident watches.
    let rain = new_parts(|| {
        for i in 0..18 {
            let a = i as f64 * 137.5;
            let r = 0.1 + (i * 7 % 6) as f64 * 0.015;
            let (x, z) = (r * cosd(a), LZ + r * sind(a));
            fcyl!(
                f,
                numbered("strand"),
                0.0035,
                CANOPY - POOL,
                (x, (CANOPY + POOL) / 2.0, z),
                M.rain,
                verts = 5
            );
            for k in 0..3 {
                fsphere!(
                    f,
                    numbered("drop"),
                    0.007,
                    (x, POOL + 0.15 + ((i * 3 + k * 5) % 11) as f64 * 0.09, z),
                    M.rain,
                    scale = (1, 1, 1.6),
                    segments = 6,
                    rings = 4
                );
            }
        }
    });
    let rain = join(&rain, "rain", Some(f.at(0, CANOPY - 0.004, LZ)));
    rain.edit(|o| o.scale.z = 0.001);
    // The ripples on the pool: `ripple` swells them out and back, all the time (ambient).
    let rip = new_parts(|| {
        for i in 0..4 {
            torus!(
                numbered("ripple"),
                0.06 + i as f64 * 0.03,
                0.0025,
                at = f.at(0, POOL + 0.002, LZ),
                material = M.ripple,
                major_segments = 32,
                minor_segments = 4
            );
        }
    });
    let rip = join(&rip, "ripples", Some(f.at(0, POOL, LZ)));
    let objs = settle!(centre = false);
    grow(
        rain,
        "rain",
        Grow {
            seconds: 0.6,
            ..Grow::default()
        },
    );
    squash(
        rip,
        "ripple",
        (1.12, 1.12, 1),
        Squash {
            seconds: 2.0,
            back: true,
            ..Squash::default()
        },
    );
    objs
}
