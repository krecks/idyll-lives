//! D2: the vases and the urn.

use super::common::{M, leaf_outline, pedestal};
use artkit::forms::Plane;
use artkit::math::{Axis, PI, V3, atan2, cos, degrees, radians, sin, v3};
use artkit::parts::{join_all, new_parts, settle_all};
use artkit::scene::{Obj, RotationMode};
use artkit::{extrude, jitter, revolve, sphere, sweep, torus};

/// A tall floor vase of pale ceramic, thrown with a foot, two grooves and a rolled lip, holding three dried twigs
/// curving out, each with a side shoot, under their pampas plumes.
pub fn vase_tall() -> Vec<Obj> {
    new_parts(|| {
        revolve!(
            "vase",
            // its lip rolled
            &artkit::forms::eased(
                &[
                    (0.0, 0.0),
                    (0.06, 0.0),
                    (0.068, 0.004),
                    (0.07, 0.014),
                    (0.064, 0.022),
                    (0.072, 0.05),
                    (0.09, 0.15),
                    (0.098, 0.2),
                    (0.096, 0.205),
                    (0.1, 0.212),
                    (0.1, 0.27),
                    (0.096, 0.33),
                    (0.093, 0.335),
                    (0.092, 0.342),
                    (0.085, 0.42),
                    (0.07, 0.49),
                    (0.052, 0.55),
                    (0.046, 0.59),
                    (0.05, 0.606),
                    (0.058, 0.614),
                    (0.059, 0.622),
                    (0.052, 0.627),
                    (0.044, 0.616),
                    (0.04, 0.59),
                    (0.0, 0.59)
                ],
                0.002
            ),
            material = M.ceramic,
            segments = 32
        );
        for (k, (x, y, top)) in [(-0.11, 0.04, 1.0), (0.1, -0.03, 1.06), (0.02, 0.07, 0.94)]
            .into_iter()
            .enumerate()
        {
            let tip = v3(-x, y, top + 0.035); // Blender: the game's x mirrored
            let foot = v3(0.0, 0.0, 0.5);
            let bend = v3(-x * 0.35, y * 0.2, 0.0);
            let pts: Vec<V3> = [0.0, 0.33, 0.66, 1.0]
                .iter()
                .map(|&t| foot + (tip - foot) * t + bend * sin(PI * t))
                .collect();
            sweep!(
                format!("twig{k}"),
                &pts,
                0.0055,
                material = M.twig,
                verts = 6,
                resolution = 3,
                scale = [1.0, 0.85, 0.7, 0.55]
            );
            let mid = pts[2];
            let shoot = mid + v3(-x * 0.15, y * 0.3, 0.12);
            sweep!(
                format!("shoot{k}"),
                &[mid, mid + (shoot - mid) * 0.5 + v3(0.0, 0.0, 0.02), shoot],
                0.0035,
                material = M.twig,
                verts = 5,
                resolution = 2,
                scale = [1.0, 0.7]
            );
            for (name, base, along, r, n) in [
                (format!("plume{k}"), tip, tip - foot, 0.045, 3.2),
                (format!("tuft{k}"), shoot, shoot - mid, 0.026, 2.4),
            ] {
                let plume = sphere!(
                    name,
                    r,
                    at = base + along.normalized() * r * n * 0.25,
                    material = M.plume,
                    scale = (1.0, 1.0, n),
                    segments = 12,
                    rings = 10
                );
                plume.set_rotation_mode(RotationMode::Quaternion);
                plume.set_rotation_quaternion(along.to_track_quat(Axis::Z, Axis::Y)); // along its twig
                jitter!(plume, 0.004, size = 0.02, seed = k as f64 * 2.3 + r);
            }
        }
    });
    join_all("vaseTall");
    settle_all()
}

/// The ginger jar's radius at heights (its profile).
const JAR_SIDE: [(f64, f64); 3] = [(0.15, 0.17), (0.27, 0.18), (0.39, 0.16)];

fn jar_radius(y: f64) -> f64 {
    for w in JAR_SIDE.windows(2) {
        let ((y0, r0), (y1, r1)) = (w[0], w[1]);
        if y <= y1 {
            return r0 + (r1 - r0) * (y - y0) / (y1 - y0);
        }
    }
    JAR_SIDE[JAR_SIDE.len() - 1].1
}

/// A sprig of blossom painted on the jar's side: a curving stem, three leaves along it and a five-petalled flower at
/// its tip, each a thin shape lying on the porcelain.
fn sprig(k: usize, a0: f64) {
    let pts: Vec<((f64, f64, f64), f64)> = (0..5)
        .map(|i| {
            let t = i as f64 / 4.0;
            let y = 0.17 + 0.15 * t;
            let a = a0 + 0.3 * t + 0.08 * sin(PI * t);
            let r = jar_radius(y) + 0.002;
            ((cos(a) * r, sin(a) * r, y), a)
        })
        .collect();
    let stem: Vec<(f64, f64, f64)> = pts.iter().map(|&(p, _)| p).collect();
    sweep!(
        format!("stem{k}"),
        &stem,
        0.0032,
        material = M.blue,
        verts = 4,
        resolution = 2,
        scale = [1.1, 1.0, 0.9, 0.8, 0.7]
    );
    // a point `along` the jar's turn and `up` from `p`, on its side
    let on_side = |p: (f64, f64, f64), a: f64, along: f64, up: f64| {
        (p.0 - sin(a) * along, p.1 + cos(a) * along, p.2 + up)
    };
    for (i, side) in [(1, 1), (2, -1), (3, 1)] {
        // each leaf from the stem, tipped 50 degrees off it
        let (p, a) = pts[i];
        let tilt = radians((side * 50) as f64);
        sphere!(
            format!("leaf{k}_{i}"),
            0.5,
            at = on_side(p, a, -sin(tilt) * 0.02, cos(tilt) * 0.02),
            material = M.blue,
            scale = (0.006, 0.014, 0.04),
            rot = (side * 50, 0, degrees(a)),
            segments = 6,
            rings = 4
        );
    }
    let (p, a) = pts[4];
    for j in 0..5 {
        let b = radians((j * 72 + 90) as f64);
        sphere!(
            format!("petal{k}_{j}"),
            0.5,
            at = on_side(p, a, cos(b) * 0.011, sin(b) * 0.011),
            material = M.blue,
            scale = (0.006, 0.013, 0.013),
            rot = (0, 0, degrees(a)),
            segments = 6,
            rings = 3
        );
    }
}

/// A porcelain ginger jar on a foot ring, white with blue bands and sprigs; its lid on a flange, with a turned knob.
pub fn urn_porcelain() -> Vec<Obj> {
    new_parts(|| {
        revolve!(
            "jar",
            &[
                (0.0, 0.004),
                (0.098, 0.004),
                (0.1, 0.0),
                (0.108, 0.0),
                (0.11, 0.008),
                (0.112, 0.014),
                (0.122, 0.024),
                (0.15, 0.08),
                (0.17, 0.15),
                (0.18, 0.27),
                (0.16, 0.39),
                (0.13, 0.44),
                (0.11, 0.46),
                (0.1, 0.475),
                (0.1, 0.5),
                (0.0, 0.5)
            ],
            material = M.porcelain,
            segments = 30
        );
        revolve!(
            "lid",
            &[
                (0.0, 0.0),
                (0.104, 0.0),
                (0.11, 0.004),
                (0.112, 0.014),
                (0.108, 0.022),
                (0.096, 0.036),
                (0.08, 0.058),
                (0.055, 0.075),
                (0.03, 0.085),
                (0.0, 0.087)
            ],
            at = (0, 0, 0.495),
            material = M.porcelain,
            segments = 24
        );
        revolve!(
            "knob",
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.012, 0.01),
                (0.022, 0.022),
                (0.024, 0.032),
                (0.016, 0.044),
                (0.0, 0.048)
            ],
            at = (0, 0, 0.578),
            material = M.blue,
            segments = 14
        );
        for (k, (y, r)) in [(0.012, 0.11), (0.39, 0.16), (0.5, 0.106), (0.545, 0.089)]
            .into_iter()
            .enumerate()
        {
            torus!(
                format!("band{k}"),
                r,
                0.006,
                at = (0, 0, y),
                material = M.blue,
                major_segments = 30,
                minor_segments = 4
            );
        }
        for k in 0..5 {
            sprig(k, radians((k * 72 + 20) as f64));
        }
    });
    join_all("urnPorcelain");
    settle_all()
}

/// A blue glazed vase with a foot and a rolled lip, of five white hydrangea heads on curving stems with pointed
/// leaves, on a stone pedestal with moulded foot and cap and sunk panels.
pub fn vase_pedestal() -> Vec<Obj> {
    let y = 0.86;
    new_parts(|| {
        pedestal(y, 0.36, M.stone);
        revolve!(
            "vase",
            // its lip rolled
            &artkit::forms::eased(
                &[
                    (0.0, 0.0),
                    (0.07, 0.0),
                    (0.074, 0.006),
                    (0.07, 0.014),
                    (0.1, 0.07),
                    (0.118, 0.13),
                    (0.12, 0.17),
                    (0.108, 0.23),
                    (0.085, 0.28),
                    (0.06, 0.315),
                    (0.06, 0.325),
                    (0.07, 0.334),
                    (0.07, 0.342),
                    (0.062, 0.344),
                    (0.054, 0.335),
                    (0.0, 0.3)
                ],
                0.002
            ),
            at = (0, 0, y),
            material = M.vase_blue,
            segments = 32
        );
        for (k, (x, d, top)) in [
            (-0.08, 0.02, 1.48),
            (0.06, -0.04, 1.54),
            (0.02, 0.07, 1.42),
            (-0.03, -0.06, 1.36),
            (0.1, 0.05, 1.32),
        ]
        .into_iter()
        .enumerate()
        {
            let tip = v3(-x, -d, top);
            let foot = v3(0.0, 0.0, y + 0.28);
            let pts: Vec<V3> = [0.0, 0.4, 0.75, 1.0]
                .iter()
                .map(|&t| foot + (tip - foot) * t + v3(-x * 0.25, -d * 0.25, 0.0) * sin(PI * t))
                .collect();
            sweep!(
                format!("stem{k}"),
                &pts,
                0.0055,
                material = M.stem,
                verts = 5,
                resolution = 3,
                scale = [1.0, 0.9, 0.8, 0.7]
            );
            let b = sphere!(
                format!("bloom{k}"),
                0.05,
                at = (-x, -d, top + 0.02),
                material = M.bloom,
                scale = (1, 1, 0.8),
                segments = 14,
                rings = 8
            );
            jitter!(b, 0.009, size = 0.018, seed = k as f64 * 1.3);
            // two pointed leaves off the stem, held out and turned along it
            for j in 0..2 {
                let p = if j == 0 { pts[1] } else { pts[2] };
                let turn = degrees(atan2(-d, -x)) + if (j + k) % 2 == 1 { 100.0 } else { -100.0 };
                extrude!(
                    format!("leaf{k}_{j}"),
                    &leaf_outline(0.11 - 0.02 * j as f64, 0.045, 6, 0.0),
                    0.003,
                    at = p,
                    material = M.stem,
                    plane = Plane::XZ,
                    rot = (-60.0, 0.0, turn)
                );
            }
        }
    });
    join_all("vasePedestal");
    settle_all()
}
