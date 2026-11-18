//! KD2: the rocking horse.

use super::common::{P, place};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::math::{Axis, P3, TAU, cos, sin, sqrt};
use artkit::motion::{Rock, rock};
use artkit::parts::join;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fsphere, fsweep, jitter, loft, revolve, ring, soft, torus};

/// The rockers' radius: the horse rocks about their centre, 1 m up.
const ROCK_R: f64 = 1.0;

fn rocker_y(z: f64) -> f64 {
    ROCK_R - sqrt(ROCK_R * ROCK_R - z * z) + 0.022
}

/// A loop round the depth axis at game depth `zc`: an ellipse `rx` across and `ry` up round height `yc`, as
/// Blender points, for a body lofted front to back.
fn hoop(zc: f64, yc: f64, rx: f64, ry: f64, n: usize, square: f64) -> Vec<P3> {
    ring!(rx, ry, z = 0.0, verts = n, square = square)
        .iter()
        .map(|p| F.at(p.x, yc + p.y, zc))
        .collect()
}

pub fn rocking_horse() -> Vec<Obj> {
    // the rockers: two bows curling up at their ends, a stretcher between them at each end
    let zs: Vec<f64> = (0..15)
        .map(|i| 0.35 * (-1.0 + 2.0 * i as f64 / 14.0))
        .collect();
    for x in [-0.125, 0.125] {
        let e = rocker_y(0.35);
        let mut pts = vec![(x, e + 0.04, -0.371), (x, e + 0.016, -0.364)];
        pts.extend(zs.iter().map(|&z| (x, rocker_y(z), z)));
        pts.extend([(x, e + 0.016, 0.364), (x, e + 0.04, 0.371)]);
        fsweep!(
            F,
            format!("rocker{}", r(x)),
            &pts,
            0,
            M.rocker,
            profile = rounded_rect(0.045, 0.039, 0.01, 1),
            up = (0, 0, 1)
        );
    }
    for z in [-0.3, 0.3] {
        fsweep!(
            F,
            format!("stretcher{}", r(z)),
            &[
                (-0.125, rocker_y(z) + 0.01, z),
                (0.125, rocker_y(z) + 0.01, z)
            ],
            0.016,
            M.rocker,
            verts = 8
        );
    }
    // the legs, tapering to hooves on the rockers
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            let (sxf, szf) = (sx as f64, sz as f64);
            let foot = (sxf * 0.125, rocker_y(szf * 0.22) + 0.045, szf * 0.22);
            fsweep!(
                F,
                format!("leg{sx}{sz}"),
                &[
                    (sxf * 0.055, 0.44, szf * 0.14),
                    (sxf * 0.085, 0.27, szf * 0.19),
                    foot
                ],
                0.03,
                M.horse,
                verts = 10,
                resolution = 2,
                scale = [1.0, 0.72, 0.68]
            );
            fcyl!(
                F,
                format!("hoof{sx}{sz}"),
                0.019,
                0.032,
                (foot.0, foot.1 - 0.012, foot.2),
                M.dark,
                verts = 8
            );
        }
    }
    // the body: a barrel lofted from rump to chest
    let rings = [
        (-0.275, 0.475, 0.004, 0.004),
        (-0.268, 0.474, 0.05, 0.06),
        (-0.24, 0.47, 0.082, 0.092),
        (-0.17, 0.46, 0.094, 0.1),
        (-0.05, 0.452, 0.09, 0.098),
        (0.08, 0.458, 0.09, 0.1),
        (0.17, 0.475, 0.08, 0.094),
        (0.225, 0.495, 0.055, 0.066),
        (0.242, 0.505, 0.004, 0.004),
    ];
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(zc, yc, rx, ry)| hoop(zc, yc, rx, ry, 14, 2.0))
        .collect();
    loft!("body", &sections, material = M.horse);
    for (k, (x, y, z)) in [
        (0.088, 0.48, -0.12),
        (-0.089, 0.44, 0.06),
        (0.085, 0.42, 0.11),
        (-0.083, 0.49, -0.16),
        (0.075, 0.38, -0.02),
        (-0.07, 0.39, -0.08),
    ]
    .into_iter()
    .enumerate()
    {
        fsphere!(
            F,
            format!("spot{k}"),
            0.028,
            (x, y, z),
            M.spot,
            scale = (0.3, 1, 0.8),
            segments = 8,
            rings = 4
        );
    }
    // the neck and the head, swept and tapering, a muzzle on its end
    let oval = |a: f64, b: f64| -> Vec<(f64, f64)> {
        (0..12)
            .map(|i| {
                (
                    a * cos(TAU * i as f64 / 12.0),
                    b * sin(TAU * i as f64 / 12.0),
                )
            })
            .collect()
    };
    fsweep!(
        F,
        "neck",
        &[(0, 0.47, 0.16), (0, 0.6, 0.235), (0, 0.72, 0.29)],
        0,
        M.horse,
        profile = oval(0.062, 0.048),
        up = (0, 0, 1),
        resolution = 3,
        scale = [1.0, 0.85, 0.78]
    );
    fsweep!(
        F,
        "head",
        &[(0, 0.76, 0.28), (0, 0.735, 0.36), (0, 0.69, 0.43)],
        0,
        M.horse,
        profile = oval(0.05, 0.04),
        up = (0, 0, 1),
        resolution = 3,
        scale = [1.0, 0.85, 0.72]
    );
    fsphere!(
        F,
        "muzzle",
        0.034,
        (0, 0.686, 0.443),
        M.horse,
        scale = (1.0, 1.05, 1.0),
        segments = 12,
        rings = 8
    );
    fsphere!(
        F,
        "poll",
        0.045,
        (0, 0.765, 0.29),
        M.horse,
        segments = 12,
        rings = 8
    );
    for s in [-1, 1] {
        let sf = s as f64;
        fsphere!(
            F,
            format!("eye{s}"),
            0.011,
            (sf * 0.034, 0.755, 0.35),
            M.eye,
            segments = 6,
            rings = 4
        );
        fsphere!(
            F,
            format!("nostril{s}"),
            0.006,
            (sf * 0.016, 0.69, 0.474),
            M.eye,
            segments = 6,
            rings = 4
        );
        fcone!(
            F,
            format!("ear{s}"),
            0.018,
            0.003,
            0.05,
            (sf * 0.028, 0.8, 0.3),
            M.horse,
            verts = 8,
            rot = (-12, s * 10, 0)
        );
    }
    // the bridle round the muzzle, reins back to the saddle
    let bridle: Vec<P> = (0..12)
        .map(|i| {
            (
                0.04 * cos(TAU * i as f64 / 12.0),
                0.69 + 0.038 * sin(TAU * i as f64 / 12.0),
                0.42,
            )
        })
        .collect();
    fsweep!(
        F,
        "bridle",
        &bridle,
        0.005,
        M.saddle,
        verts = 5,
        closed = true
    );
    for s in [-1, 1] {
        let sf = s as f64;
        fsweep!(
            F,
            format!("rein{s}"),
            &[
                (sf * 0.04, 0.69, 0.42),
                (sf * 0.07, 0.6, 0.3),
                (sf * 0.08, 0.565, 0.13)
            ],
            0.004,
            M.saddle,
            verts = 4,
            resolution = 3
        );
    }
    // the mane down the crest and a forelock, the tail
    let mane = fsweep!(
        F,
        "mane",
        &[
            (0, 0.52, 0.1),
            (0, 0.635, 0.18),
            (0, 0.755, 0.245),
            (0, 0.8, 0.295),
            (0, 0.79, 0.34)
        ],
        0,
        M.mane,
        profile = rounded_rect(0.05, 0.026, 0.01, 1),
        up = (0, 0, 1),
        resolution = 2,
        scale = [0.7, 1.0, 1.0, 0.8, 0.5]
    );
    jitter!(mane, 0.006, size = 0.04, seed = 4.0);
    let tail = fsweep!(
        F,
        "tail",
        &[
            (0, 0.5, -0.265),
            (0, 0.49, -0.31),
            (0, 0.42, -0.345),
            (0, 0.33, -0.35)
        ],
        0.024,
        M.mane,
        verts = 8,
        resolution = 2,
        scale = [0.8, 1.15, 1.0, 0.4]
    );
    jitter!(tail, 0.005, size = 0.04, seed = 5.0);
    // the saddle on a cloth, its flaps, stirrups on leathers; the seat where the game seats the rider (0.56)
    soft!(
        F,
        "saddle",
        0.2,
        0.024,
        0.21,
        (0, 0.545, -0.02),
        M.saddle,
        round = 0.01,
        puff = 0.004,
        flat = 2,
        round_segments = 1
    );
    soft!(
        F,
        "pommel",
        0.08,
        0.03,
        0.04,
        (0, 0.56, 0.075),
        M.saddle,
        round = 0.012,
        round_segments = 1,
        flat = 1
    );
    soft!(
        F,
        "cantle",
        0.12,
        0.03,
        0.035,
        (0, 0.562, -0.115),
        M.saddle,
        round = 0.012,
        round_segments = 1,
        flat = 1
    );
    for s in [-1, 1] {
        let sf = s as f64;
        fbox!(
            F,
            format!("saddle_flap{s}"),
            0.012,
            0.1,
            0.15,
            (sf * 0.098, 0.49, -0.02),
            M.saddle,
            rot = (0, s * 8, 0),
            bevel = 0.005,
            bevel_segments = 1
        );
        fsweep!(
            F,
            format!("leather{s}"),
            &[(sf * 0.104, 0.52, 0.0), (sf * 0.107, 0.36, 0.0)],
            0,
            M.saddle,
            profile = rounded_rect(0.003, 0.016, 0.001, 1),
            up = (0, 0, 1)
        );
        torus!(
            format!("stirrup{s}"),
            0.018,
            0.004,
            at = F.at(sf * 0.107, 0.345, 0.0),
            material = M.handle,
            rot = (0, 90, 0),
            major_segments = 10,
            minor_segments = 4
        );
    }
    // the handle through the head, a turned wooden grip on each end
    fsweep!(
        F,
        "handle",
        &[(-0.085, 0.68, 0.28), (0.085, 0.68, 0.28)],
        0.008,
        M.handle,
        verts = 8
    );
    for s in [-1, 1] {
        revolve!(
            format!("grip{s}"),
            &[
                (0.0, -0.028),
                (0.011, -0.028),
                (0.016, -0.016),
                (0.017, 0.0),
                (0.015, 0.018),
                (0.009, 0.028),
                (0.0, 0.028)
            ],
            at = F.at(s as f64 * 0.1, 0.68, 0.28),
            material = M.rocker,
            segments = 8,
            rot = (0, 90, 0)
        );
    }
    // the whole horse rocks, its origin on the rockers' centre; the rider rides it
    let horse = join(&meshes(), "horse", Some(F.at(0, ROCK_R, 0)));
    place(0.0);
    rock(
        horse,
        "rock",
        12.0,
        Rock {
            seconds: 1.2,
            axis: Axis::X,
        },
    );
    meshes()
}
