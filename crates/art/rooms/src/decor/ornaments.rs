//! D2: vases, statues, the coat rack and the hanging plant.

use super::common::{F, M, heart, leaf_outline, pedestal};
use artkit::forms::{Plane, rounded_rect};
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{Axis, P3, PI, TAU, V3, atan2, cos, degrees, radians, sin, v3};
use artkit::parts::{hinged, join, join_all, new_parts, settle_all};
use artkit::scene::{Obj, RotationMode, meshes};
use artkit::{
    boulder, cone, cube, extrude, fsphere, fsweep, jitter, loft, revolve, ring, settle, sphere,
    sweep, swing, torus,
};

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

/// A marble bust on a stone pedestal: a turned socle, a chest cut off at the shoulders, a neck, a head lofted from
/// chin to crown with a nose, eyelids, lips, a chin and ears, under a cap of curly hair.
pub fn bust_marble() -> Vec<Obj> {
    let y = 1.0;
    new_parts(|| {
        pedestal(y, 0.4, M.stone);
        revolve!(
            "socle",
            &[
                (0.0, 0.0),
                (0.09, 0.0),
                (0.09, 0.012),
                (0.082, 0.02),
                (0.06, 0.03),
                (0.05, 0.05),
                (0.056, 0.07),
                (0.056, 0.08),
                (0.0, 0.08)
            ],
            at = (0, 0, y),
            material = M.marble,
            segments = 28
        );
        // the chest, cut off flat at the arms, its shoulders sloping to the neck
        let secs: Vec<Vec<P3>> = [
            (0.075, 0.055, 0.042, 0.0, 2.0),
            (0.088, 0.1, 0.058, 0.0, 2.6),
            (0.11, 0.14, 0.074, -0.004, 3.4),
            (0.18, 0.152, 0.082, -0.006, 3.6),
            (0.222, 0.15, 0.08, -0.002, 3.4),
            (0.24, 0.14, 0.074, 0.002, 3.0),
            (0.255, 0.115, 0.064, 0.004, 2.6),
            (0.267, 0.08, 0.052, 0.004, 2.2),
            (0.276, 0.045, 0.04, 0.004, 2.0),
        ]
        .iter()
        .map(|&(z, rx, ry, cy, sq)| {
            ring!(rx, ry, z = y + z, verts = 24, square = sq, at = (0.0, cy))
        })
        .collect();
        loft!("shoulders", &secs, material = M.marble);
        for s in [-1, 1] {
            // the collarbones and the chest's divide
            let sf = s as f64;
            sweep!(
                format!("collarbone{s}"),
                &[
                    (sf * 0.012, -0.05, y + 0.255),
                    (sf * 0.06, -0.06, y + 0.25),
                    (sf * 0.11, -0.05, y + 0.25)
                ],
                0.008,
                material = M.marble,
                verts = 6,
                resolution = 2,
                scale = [0.6, 1.0, 0.5]
            );
            sphere!(
                format!("chest{s}"),
                0.5,
                at = (sf * 0.055, -0.07, y + 0.19),
                material = M.marble,
                scale = (0.1, 0.03, 0.07),
                segments = 12,
                rings = 6
            );
        }
        let neck: Vec<Vec<P3>> = [
            (0.25, 0.046, 0.0),
            (0.3, 0.04, -0.004),
            (0.38, 0.039, -0.008), // its top inside the head
        ]
        .iter()
        .map(|&(z, r, cy)| ring!(r, r * 1.05, z = y + z, verts = 16, at = (0.0, cy)))
        .collect();
        loft!("neck", &neck, material = M.marble);
        let head: Vec<Vec<P3>> = [
            (0.335, 0.018, 0.016, -0.054),
            (0.345, 0.036, 0.034, -0.042),
            (0.36, 0.051, 0.056, -0.024),
            (0.385, 0.061, 0.073, -0.009),
            (0.42, 0.067, 0.083, 0.0),
            (0.45, 0.07, 0.086, 0.004),
            (0.475, 0.067, 0.083, 0.008),
            (0.495, 0.054, 0.068, 0.01),
            (0.505, 0.03, 0.04, 0.012),
            (0.508, 0.005, 0.005, 0.012),
        ]
        .iter()
        .map(|&(z, rx, ry, cy)| ring!(rx, ry, z = y + z, verts = 20, square = 2.2, at = (0.0, cy)))
        .collect();
        loft!("head", &head, material = M.marble);
        extrude!(
            "nose",
            &[
                (0.0, 0.0),
                (0.02, -0.004),
                (0.026, -0.04),
                (0.012, -0.048),
                (-0.004, -0.044)
            ],
            0.02,
            at = (0, -0.078, y + 0.452),
            material = M.marble,
            plane = Plane::YZ,
            rot = (0, 0, 180),
            bevel = 0.006,
            bevel_segments = 2
        );
        for s in [-1, 1] {
            let sf = s as f64;
            sphere!(
                format!("lid{s}"),
                0.5,
                at = (sf * 0.028, -0.074, y + 0.437),
                material = M.marble,
                scale = (0.026, 0.012, 0.012),
                segments = 10,
                rings = 5
            );
            sphere!(
                format!("ear{s}"),
                0.5,
                at = (sf * 0.07, 0.004, y + 0.425),
                material = M.marble,
                scale = (0.018, 0.03, 0.048),
                rot = (0, 0, s * 20),
                segments = 10,
                rings = 6
            );
        }
        for (k, z) in [0.384, 0.374].into_iter().enumerate() {
            sphere!(
                format!("lip{k}"),
                0.5,
                at = (0, -0.074, y + z),
                material = M.marble,
                scale = (0.03, 0.012, 0.009),
                segments = 10,
                rings = 5
            );
        }
        sphere!(
            "chin",
            0.5,
            at = (0, -0.062, y + 0.352),
            material = M.marble,
            scale = (0.036, 0.026, 0.026),
            segments = 10,
            rings = 6
        );
        // the hair: a curly cap from the brow over the crown to the nape
        let hair = boulder!(
            "hair",
            0.085,
            at = (0, 0.016, y + 0.4),
            material = M.marble,
            scale = (0.95, 1.05, 0.8),
            seed = 3.0,
            noise = 0.18,
            subdivisions = 3
        );
        jitter!(hair, 0.005, size = 0.012, seed = 1.0);
    });
    join_all("bustMarble");
    settle_all()
}

/// A concrete cat sitting up: its rump with a thigh bulging at either side, the hind feet tucked forward by the front
/// legs and paws (toes marked), a chest, a head with a muzzle, whisker pads, a chin, a nose and lidded eyes, pointed
/// ears with hollows, and its tail curling round its front paws.
pub fn statue_cat() -> Vec<Obj> {
    new_parts(|| {
        sphere!(
            "rump",
            0.5,
            at = (0, 0.07, 0.12),
            material = M.concrete,
            scale = (0.22, 0.26, 0.23),
            segments = 20,
            rings = 12
        );
        for s in [-1, 1] {
            let sf = s as f64;
            sphere!(
                format!("thigh{s}"),
                0.5,
                at = (sf * 0.085, 0.03, 0.1),
                material = M.concrete,
                scale = (0.11, 0.22, 0.17),
                rot = (-20, 0, 0),
                segments = 16,
                rings = 10
            );
            sphere!(
                format!("hind_foot{s}"),
                0.5,
                at = (sf * 0.1, -0.08, 0.018),
                material = M.concrete,
                scale = (0.055, 0.1, 0.036),
                segments = 10,
                rings = 6
            );
        }
        sphere!(
            "chest",
            0.5,
            at = (0, -0.04, 0.26),
            material = M.concrete,
            scale = (0.19, 0.18, 0.32),
            rot = (-12, 0, 0),
            segments = 18,
            rings = 12
        );
        sphere!(
            "head",
            0.5,
            at = (0, -0.07, 0.46),
            material = M.concrete,
            scale = (0.19, 0.17, 0.16),
            segments = 18,
            rings = 12
        );
        sphere!(
            "muzzle",
            0.5,
            at = (0, -0.145, 0.43),
            material = M.concrete,
            scale = (0.08, 0.05, 0.05),
            segments = 12,
            rings = 8
        );
        sphere!(
            "chin",
            0.5,
            at = (0, -0.14, 0.408),
            material = M.concrete,
            scale = (0.04, 0.03, 0.025),
            segments = 8,
            rings = 6
        );
        sphere!(
            "nose",
            0.5,
            at = (0, -0.168, 0.445),
            material = M.concrete_dark,
            scale = (0.02, 0.012, 0.012),
            segments = 8,
            rings = 4
        );
        for s in [-1, 1] {
            let sf = s as f64;
            sphere!(
                format!("pad{s}"),
                0.5,
                at = (sf * 0.018, -0.162, 0.428),
                material = M.concrete,
                scale = (0.034, 0.026, 0.026),
                segments = 8,
                rings = 6
            );
            sphere!(
                format!("eye{s}"),
                0.5,
                at = (sf * 0.04, -0.152, 0.475),
                material = M.concrete_dark,
                scale = (0.026, 0.01, 0.016),
                rot = (0, s * 12, 0),
                segments = 8,
                rings = 4
            );
            sphere!(
                format!("lid{s}"),
                0.5,
                at = (sf * 0.04, -0.15, 0.481),
                material = M.concrete,
                scale = (0.03, 0.012, 0.008),
                rot = (0, s * 12, 0),
                segments = 8,
                rings = 4
            );
            // the ear: a pointed cone, flattened front to back, its front hollowed
            let ear: Vec<Vec<P3>> = [
                (0.0, 0.53, 0.036, 0.02),
                (0.5, 0.56, 0.022, 0.013),
                (1.0, 0.6, 0.003, 0.003),
            ]
            .iter()
            .map(|&(t, z, rx, ry)| {
                ring!(
                    rx,
                    ry,
                    z = z,
                    verts = 8,
                    at = (sf * (0.052 + 0.012 * t), -0.062)
                )
            })
            .collect();
            loft!(format!("ear{s}"), &ear, material = M.concrete);
            let hollow: Vec<Vec<P3>> = [(0.0, 0.535, 0.024, 0.004), (1.0, 0.585, 0.003, 0.002)]
                .iter()
                .map(|&(t, z, rx, ry)| {
                    ring!(
                        rx,
                        ry,
                        z = z,
                        verts = 8,
                        at = (sf * (0.052 + 0.012 * t), -0.077)
                    )
                })
                .collect();
            loft!(
                format!("ear_hollow{s}"),
                &hollow,
                material = M.concrete_dark
            );
            sweep!(
                format!("leg{s}"),
                &[
                    (sf * 0.045, -0.12, 0.23),
                    (sf * 0.044, -0.13, 0.14),
                    (sf * 0.045, -0.14, 0.05),
                    (sf * 0.045, -0.145, 0.025)
                ],
                0.024,
                material = M.concrete,
                verts = 10,
                resolution = 2,
                scale = [1.15, 1.0, 0.9, 0.95]
            );
            sphere!(
                format!("paw{s}"),
                0.5,
                at = (sf * 0.045, -0.15, 0.017),
                material = M.concrete,
                scale = (0.06, 0.07, 0.035),
                segments = 10,
                rings = 6
            );
            for t in [-1, 1] {
                cube!(
                    format!("toe{s}{t}"),
                    (0.002, 0.03, 0.012),
                    at = (sf * 0.045 + t as f64 * 0.01, -0.172, 0.02),
                    material = M.concrete_dark
                );
            }
        }
        // the tail: one tapering sweep from the rump round the side to the front paws
        sweep!(
            "tail",
            &[
                (-0.08, 0.17, 0.035),
                (-0.13, 0.1, 0.028),
                (-0.148, 0.0, 0.026),
                (-0.128, -0.11, 0.026),
                (-0.08, -0.18, 0.024),
                (-0.03, -0.195, 0.024)
            ],
            0.022,
            material = M.concrete,
            verts = 10,
            resolution = 3,
            scale = [1.0, 1.0, 0.95, 0.9, 0.85, 0.7]
        );
    });
    join_all("statueCat");
    settle_all()
}

fn ellipse_profile(a: f64, b: f64) -> Vec<(f64, f64)> {
    let n = 10;
    (0..n)
        .map(|k| {
            (
                a * cos(TAU * k as f64 / n as f64),
                b * sin(TAU * k as f64 / n as f64),
            )
        })
        .collect()
}

/// A bronze horse standing on a bronze base plate on an oak pedestal: a lofted barrel dipping at the back, a crested
/// neck and a tapering head with a jowl and muzzle, ears, a flowing mane and forelock, legs with knees, hocks and
/// hooves, and a full tail, in a dark patina.
pub fn statue_horse() -> Vec<Obj> {
    let y = 0.92;
    new_parts(|| {
        pedestal(y, 0.4, M.oak);
        extrude!(
            "base",
            &rounded_rect(0.34, 0.16, 0.03, 3),
            0.02,
            at = (0, 0, y + 0.01),
            material = M.bronze,
            plane = Plane::XY,
            bevel = 0.004,
            bevel_segments = 1
        );
        // the barrel along the length (Blender x, its head to -x): rings across it, lofted, turned to lie along x
        let rings = [
            (0.0, 0.03, 0.025, 0.012),
            (0.02, 0.062, 0.042, 0.012),
            (0.05, 0.072, 0.052, 0.01),
            (0.1, 0.066, 0.056, 0.0),
            (0.16, 0.064, 0.055, -0.004),
            (0.22, 0.068, 0.054, 0.004),
            (0.26, 0.064, 0.05, 0.008),
            (0.29, 0.05, 0.042, 0.004),
            (0.305, 0.022, 0.024, 0.0),
        ];
        let secs: Vec<Vec<P3>> = rings
            .iter()
            .map(|&(z, rv, rw, dz)| ring!(rv, rw, z = z, verts = 16, at = (dz, 0.0)))
            .collect();
        loft!(
            "barrel",
            &secs,
            material = M.bronze,
            at = (0.152, 0, y + 0.272),
            rot = (0, -90, 0)
        );
        sweep!(
            "neck",
            &[
                (-0.11, 0, y + 0.29),
                (-0.135, 0, y + 0.36),
                (-0.155, 0, y + 0.42),
                (-0.162, 0, y + 0.445)
            ],
            0.0,
            material = M.bronze,
            profile = ellipse_profile(0.038, 0.026),
            up = (1, 0, 0),
            resolution = 2,
            scale = [1.45, 1.1, 0.85, 0.75]
        );
        sweep!(
            "head",
            &[
                (-0.15, 0, y + 0.445),
                (-0.18, 0, y + 0.415),
                (-0.205, 0, y + 0.38),
                (-0.225, 0, y + 0.355)
            ],
            0.0,
            material = M.bronze,
            profile = ellipse_profile(0.026, 0.02),
            up = (-1, 0, 1),
            resolution = 2,
            scale = [1.0, 1.05, 0.85, 0.8]
        );
        sphere!(
            "jowl",
            0.5,
            at = (-0.17, 0, y + 0.415),
            material = M.bronze,
            scale = (0.05, 0.042, 0.045),
            segments = 10,
            rings = 6
        );
        sphere!(
            "muzzle",
            0.023,
            at = (-0.222, 0, y + 0.352),
            material = M.bronze,
            scale = (1.1, 0.95, 1),
            segments = 10,
            rings = 6
        );
        for s in [-1, 1] {
            cone!(
                format!("ear{s}"),
                0.011,
                0.002,
                0.04,
                at = (-0.14, s as f64 * 0.012, y + 0.47),
                material = M.bronze,
                rot = (s * -10, -12, 0),
                verts = 6
            );
        }
        // the mane along the crest, falling to one side, and a forelock between the ears
        sweep!(
            "mane",
            &[
                (-0.098, 0.004, y + 0.33),
                (-0.112, 0.008, y + 0.37),
                (-0.128, 0.01, y + 0.41),
                (-0.14, 0.008, y + 0.445)
            ],
            0.0,
            material = M.bronze_dark,
            profile = rounded_rect(0.036, 0.012, 0.005, 1),
            up = (1, 0, 1),
            resolution = 2,
            scale = [0.7, 1.0, 1.0, 0.7]
        );
        sweep!(
            "forelock",
            &[
                (-0.15, 0, y + 0.462),
                (-0.168, 0, y + 0.445),
                (-0.178, 0, y + 0.428)
            ],
            0.008,
            material = M.bronze_dark,
            verts = 5,
            resolution = 2,
            scale = [1.0, 0.8, 0.4]
        );
        sweep!(
            "tail",
            &[
                (0.145, 0.0, y + 0.31),
                (0.175, 0.0, y + 0.3),
                (0.19, 0.004, y + 0.24),
                (0.188, 0.008, y + 0.17),
                (0.19, 0.004, y + 0.12)
            ],
            0.015,
            material = M.bronze_dark,
            verts = 8,
            resolution = 2,
            scale = [0.7, 1.0, 1.15, 1.0, 0.6]
        );
        // legs: (hip or shoulder, knee or hock, fetlock, hoof), each tapering, a hoof under it
        for (x, joints) in [
            (
                -0.1,
                [
                    (-0.1, 0.25),
                    (-0.104, 0.135),
                    (-0.108, 0.055),
                    (-0.112, 0.03),
                ],
            ),
            (
                0.1,
                [(0.1, 0.26), (0.128, 0.13), (0.112, 0.055), (0.11, 0.03)],
            ),
        ] {
            for d in [-0.035, 0.035] {
                let pts: Vec<(f64, f64, f64)> =
                    joints.iter().map(|&(jx, jy)| (jx, d, y + jy)).collect();
                sweep!(
                    format!("leg{}{}", float_repr(x), float_repr(d)),
                    &pts,
                    0.0135,
                    material = M.bronze,
                    verts = 8,
                    resolution = 2,
                    scale = [if x > 0.0 { 2.0 } else { 1.6 }, 1.05, 0.95, 0.85]
                );
                cone!(
                    format!("hoof{}{}", float_repr(x), float_repr(d)),
                    0.017,
                    0.013,
                    0.022,
                    at = (joints[joints.len() - 1].0, d, y + 0.031),
                    material = M.bronze,
                    verts = 10
                );
            }
        }
    });
    join_all("statueHorse");
    settle_all()
}

const COAT_AT: (f64, f64) = (0.1, 0.07); // where the coat hangs (game x, depth)
const COAT_HEM: f64 = 0.97; // its hem's height
const COAT_TURN: i32 = 30; // its turn

/// A coat hung by its collar: a body flaring a little to the hem, lapels and three buttons down its front, pocket
/// flaps, sleeves hanging at its sides with turned-back cuffs, a collar rolled round its neck.
fn coat() {
    let (c, turn) = (F.at(COAT_AT.0, 0, COAT_AT.1), radians(COAT_TURN as f64));
    // a point in the coat's own frame (Blender x across it, y through it, h up from the hem)
    let at = |x: f64, y: f64, h: f64| {
        (
            c.x + x * cos(turn) - y * sin(turn),
            c.y + x * sin(turn) + y * cos(turn),
            COAT_HEM + h,
        )
    };
    // lofted through flat oval rings (not a lathe squashed after its UVs were laid): the cloth's weave even all round
    let body: Vec<Vec<P3>> = [
        (0.125, 0.0),
        (0.13, 0.015),
        (0.12, 0.1),
        (0.108, 0.3),
        (0.112, 0.45),
        (0.105, 0.53),
        (0.075, 0.585),
        (0.035, 0.6),
    ]
    .iter()
    .map(|&(r, h)| ring!(r, r * 0.45, z = h, verts = 20))
    .collect();
    loft!(
        "coat",
        &body,
        material = M.coat,
        at = at(0.0, 0.0, 0.0),
        rot = (0, 0, COAT_TURN)
    );
    let collar: Vec<(f64, f64, f64)> = (0..11)
        .map(|k| {
            let a = radians((-120 + 24 * k) as f64);
            at(0.07 * sin(a), 0.035 * cos(a), 0.585 + 0.01 * cos(a))
        })
        .collect();
    sweep!(
        "coat_collar",
        &collar,
        0.014,
        material = M.coat,
        verts = 6,
        resolution = 2
    );
    for s in [-1, 1] {
        let sf = s as f64;
        extrude!(
            format!("lapel{s}"),
            &[
                (0.0, 0.0),
                (sf * 0.05, 0.11),
                (sf * 0.03, 0.135),
                (sf * 0.006, 0.13)
            ],
            0.006,
            at = at(sf * 0.006, -0.05, 0.43),
            material = M.coat,
            plane = Plane::XZ,
            rot = (-6, 0, COAT_TURN)
        );
        extrude!(
            format!("pocket{s}"),
            &rounded_rect(0.06, 0.022, 0.004, 1),
            0.006,
            at = at(sf * 0.065, -0.048, 0.2),
            material = M.coat,
            plane = Plane::XZ,
            rot = (0, 0, COAT_TURN + s * 18)
        );
        sweep!(
            format!("sleeve{s}"),
            &[
                at(sf * 0.1, -0.005, 0.53),
                at(sf * 0.118, -0.012, 0.36),
                at(sf * 0.122, -0.022, 0.15)
            ],
            0.027,
            material = M.coat,
            verts = 10,
            resolution = 2,
            scale = [1.0, 0.95, 0.9]
        );
        torus!(
            format!("cuff{s}"),
            0.024,
            0.006,
            at = at(sf * 0.122, -0.022, 0.16),
            material = M.coat,
            major_segments = 10,
            minor_segments = 4,
            rot = (0, 0, COAT_TURN)
        );
    }
    for k in 0..3 {
        sphere!(
            format!("button{k}"),
            0.007,
            at = at(0.0, -0.054 + 0.004 * k as f64, 0.36 - 0.09 * k as f64),
            material = M.brass,
            segments = 8,
            rings = 4
        );
    }
}

/// A turned oak coat stand: four legs curving down from a hub to round pads, a pole with beads under each ring of
/// brass S-hooks and a ball finial; a coat, a fringed scarf over a hook and a hat with a band.
pub fn coat_rack() -> Vec<Obj> {
    new_parts(|| {
        let prof = [
            (0.0, 0.0),
            (0.03, 0.0),
            (0.042, 0.02),
            (0.044, 0.05),
            (0.036, 0.07),
            (0.026, 0.085),
            (0.023, 0.12),
            (0.021, 1.2),
            (0.026, 1.215),
            (0.026, 1.235),
            (0.02, 1.25),
            (0.019, 1.39),
            (0.024, 1.405),
            (0.024, 1.425),
            (0.019, 1.44),
            (0.017, 1.54),
            (0.013, 1.565),
            (0.0, 1.57),
        ];
        revolve!(
            "pole",
            &prof,
            at = F.at(0, 0.17, 0),
            material = M.oak,
            segments = 14
        );
        fsphere!(
            F,
            "top",
            0.038,
            (0, 1.75, 0),
            M.oak,
            segments = 14,
            rings = 8
        );
        for k in 0..4 {
            let a = radians((45 + k * 90) as f64);
            let (c, s) = (cos(a), sin(a));
            let pts: Vec<(f64, f64, f64)> =
                [(0.02, 0.24), (0.08, 0.205), (0.15, 0.1), (0.19, 0.03)]
                    .iter()
                    .map(|&(r, h)| (r * c, h, r * s))
                    .collect();
            kit::tube!(
                F,
                format!("leg{k}"),
                &pts,
                0.016,
                M.oak,
                verts = 8,
                round = 3,
                scale = [1.25, 1.05, 0.85, 0.75]
            );
            revolve!(
                format!("foot{k}"),
                &[
                    (0.0, 0.0),
                    (0.022, 0.0),
                    (0.024, 0.008),
                    (0.016, 0.022),
                    (0.0, 0.024)
                ],
                at = F.at(0.2 * c, 0.0, 0.2 * s),
                material = M.oak,
                segments = 10
            );
        }
        for (k, y) in [1.62, 1.62, 1.62, 1.62, 1.44, 1.44].into_iter().enumerate() {
            let a = radians(
                (k * 90 + if k > 3 { 45 } else { 0 } + if k == 5 { 180 } else { 0 }) as f64,
            );
            let (c, s) = (cos(a), sin(a));
            let pts: Vec<(f64, f64, f64)> =
                [(0.015, 0.0), (0.06, 0.004), (0.1, 0.03), (0.117, 0.06)]
                    .iter()
                    .map(|&(r, h)| (r * c, y + h, r * s))
                    .collect();
            kit::tube!(
                F,
                format!("hook{k}"),
                &pts,
                0.0055,
                M.brass,
                verts = 6,
                round = 3
            );
            fsphere!(
                F,
                format!("hook_tip{k}"),
                0.011,
                (0.12 * c, y + 0.07, 0.12 * s),
                M.brass,
                segments = 8,
                rings = 6
            );
        }
        // a coat hanging from a hook on the right-front, a scarf over the lower left one, a hat on the top hook
        coat();
        let (sx, sy, sz) = (0.085, 1.5, -0.085);
        for (side, d) in [("front", 1.0), ("back", -1.0)] {
            fsweep!(
                F,
                format!("scarf_{side}"),
                &[
                    (sx, sy + 0.012, sz),
                    (sx + 0.02 * d, sy - 0.02, sz + 0.028 * d),
                    (sx + 0.035 * d, sy - 0.2, sz + 0.035 * d),
                    (sx + 0.03 * d, sy - 0.42 + 0.06 * d, sz + 0.03 * d)
                ],
                0,
                M.scarf,
                profile = rounded_rect(0.07, 0.012, 0.005, 1),
                up = (1, 0, -1),
                resolution = 3
            );
            let end = (sx + 0.03 * d, sy - 0.42 + 0.06 * d, sz + 0.03 * d);
            for i in 0..5 {
                let o = (i as f64 - 2.0) * 0.012 * 0.7;
                kit::tube!(
                    F,
                    format!("fringe_{side}{i}"),
                    &[
                        (end.0 + o, end.1, end.2 + o),
                        (end.0 + o, end.1 - 0.035, end.2 + o)
                    ],
                    0.0025,
                    M.scarf,
                    verts = 4,
                    round = 1
                );
            }
        }
        revolve!(
            "hat",
            &[
                (0.0, 0.0),
                (0.12, 0.0),
                (0.122, 0.006),
                (0.11, 0.012),
                (0.072, 0.012),
                (0.068, 0.03),
                (0.066, 0.07),
                (0.058, 0.088),
                (0.03, 0.092),
                (0.012, 0.084),
                (0.0, 0.086)
            ],
            at = F.at(-0.09, 1.684, -0.06),
            material = M.hat,
            segments = 24
        );
        torus!(
            "hat_band",
            0.069,
            0.007,
            at = F.at(-0.09, 1.703, -0.06),
            material = M.scarf,
            major_segments = 24,
            minor_segments = 4
        );
    });
    join_all("coatRack");
    settle_all()
}

// --- D2: the hanging plant --------------------------------------------------------------------------------
const CEILING: f64 = 2.8;

/// A pot with a rolled rim in a macramé hanger: a cord from the ceiling hook to a ring, a gathering knot, three
/// knotted strands that cross into a net round the pot and gather again in a tasselled knot below it; leafy, its
/// vines trailing with heart leaves. It sways when watered: the whole hanger swings about the hook.
pub fn hanging_plant() -> Vec<Obj> {
    let still = new_parts(|| {
        revolve!(
            "hook_plate",
            &[
                (0.0, 0.0),
                (0.008, 0.0),
                (0.012, 0.004),
                (0.03, 0.008),
                (0.03, 0.012),
                (0.0, 0.012)
            ],
            at = F.at(0, CEILING - 0.012, 0),
            material = M.cord,
            segments = 14
        );
    });
    join(&still, "body", Some(P3::ZERO));
    let parts = new_parts(|| {
        kit::tube!(
            F,
            "cord",
            &[(0, CEILING - 0.01, 0), (0, 1.84, 0)],
            0.005,
            M.cord,
            verts = 6,
            round = 1
        );
        torus!(
            "ring",
            0.016,
            0.004,
            at = F.at(0, 1.825, 0),
            material = M.cord,
            rot = (90, 0, 0),
            major_segments = 10,
            minor_segments = 4
        );
        fsphere!(
            F,
            "gather_knot",
            0.016,
            (0, 1.79, 0),
            M.cord,
            scale = (1, 1, 1.4),
            segments = 8,
            rings = 6
        );
        let corners: Vec<(f64, f64)> = (0..3)
            .map(|k| {
                (
                    cos(radians((90 + k * 120) as f64)),
                    sin(radians((90 + k * 120) as f64)),
                )
            })
            .collect();
        for (k, &(c, s)) in corners.iter().enumerate() {
            kit::tube!(
                F,
                format!("hanger{k}"),
                &[
                    (0.008 * c, 1.78, 0.008 * s),
                    (0.1 * c, 1.6, 0.1 * s),
                    (0.162 * c, 1.44, 0.162 * s)
                ],
                0.0045,
                M.cord,
                verts = 5,
                round = 2
            );
            for (i, h) in [1.62, 1.5].into_iter().enumerate() {
                let r = 0.008 + (0.162 - 0.008) * (1.78 - h) / 0.34;
                fsphere!(
                    F,
                    format!("knot{k}_{i}"),
                    0.009,
                    (r * c, h, r * s),
                    M.cord,
                    scale = (1, 1, 1.3),
                    segments = 6,
                    rings = 4
                );
            }
            // the strand splits: each half crosses to meet its neighbour's under the pot's belly
            let (c2, s2) = corners[(k + 1) % 3];
            let (mut mc, mut ms) = ((c + c2) / 2.0, (s + s2) / 2.0);
            let ml = artkit::py::hypot(mc, ms);
            (mc, ms) = (mc / ml, ms / ml);
            for (j, (ec, es)) in [(c, s), (c2, s2)].into_iter().enumerate() {
                let (dc, ds) = (ec + mc, es + ms);
                let dl = artkit::py::hypot(dc, ds);
                kit::tube!(
                    F,
                    format!("net{k}_{j}"),
                    &[
                        (0.162 * ec, 1.44, 0.162 * es),
                        (0.157 * dc / dl, 1.37, 0.157 * ds / dl),
                        (0.146 * mc, 1.3, 0.146 * ms)
                    ],
                    0.0035,
                    M.cord,
                    verts = 4,
                    round = 2
                );
            }
            fsphere!(
                F,
                format!("net_knot{k}"),
                0.008,
                (0.146 * mc, 1.3, 0.146 * ms),
                M.cord,
                segments = 6,
                rings = 4
            );
            kit::tube!(
                F,
                format!("strand{k}"),
                &[
                    (0.146 * mc, 1.3, 0.146 * ms),
                    (0.125 * mc, 1.235, 0.125 * ms),
                    (0.07 * mc, 1.19, 0.07 * ms),
                    (0.01 * mc, 1.165, 0.01 * ms)
                ],
                0.004,
                M.cord,
                verts = 4,
                round = 2
            );
        }
        fsphere!(
            F,
            "base_knot",
            0.02,
            (0, 1.155, 0),
            M.cord,
            scale = (1, 1, 1.3),
            segments = 8,
            rings = 6
        );
        for i in 0..6 {
            let a = radians((i * 60) as f64);
            kit::tube!(
                F,
                format!("tassel{i}"),
                &[
                    (0.006 * cos(a), 1.14, 0.006 * sin(a)),
                    (0.016 * cos(a), 1.06, 0.016 * sin(a))
                ],
                0.0035,
                M.cord,
                verts = 4,
                round = 1
            );
        }
        revolve!(
            "pot",
            &[
                (0.0, 0.0),
                (0.1, 0.0),
                (0.104, 0.006),
                (0.142, 0.12),
                (0.15, 0.2),
                (0.16, 0.205),
                (0.163, 0.218),
                (0.16, 0.24),
                (0.148, 0.24),
                (0.14, 0.215),
                (0.13, 0.13),
                (0.0, 0.13)
            ],
            at = F.at(0, 1.2, 0),
            material = M.pot_white,
            segments = 28
        );
        torus!(
            "pot_band",
            0.148,
            0.007,
            at = F.at(0, 1.33, 0),
            material = M.cord,
            major_segments = 28,
            minor_segments = 4
        );
        for (k, (dx, dy, dz, r)) in [
            (0.0, 1.5, 0.0, 0.17),
            (-0.12, 1.47, 0.06, 0.11),
            (0.12, 1.46, -0.05, 0.11),
            (0.04, 1.47, 0.13, 0.1),
            (-0.04, 1.46, -0.13, 0.1),
        ]
        .into_iter()
        .enumerate()
        {
            let l = sphere!(
                format!("leaves{k}"),
                r,
                at = F.at(dx, dy, dz),
                material = M.leaf,
                scale = (1, 1, 0.75),
                segments = 14,
                rings = 8
            );
            jitter!(l, 0.02, size = 0.05, seed = k as f64 * 0.9);
        }
        for (k, (x, z, length)) in [
            (0.18, 0.08, 0.45),
            (-0.17, -0.06, 0.38),
            (0.05, -0.17, 0.3),
            (-0.06, 0.16, 0.34),
        ]
        .into_iter()
        .enumerate()
        {
            let mut pts: Vec<(f64, f64, f64)> = (0..7)
                .map(|i| {
                    let t = i as f64 / 6.0;
                    (
                        x * (0.8 + 0.2 * t) + 0.012 * sin(7.0 * t + k as f64),
                        1.42 - length * t,
                        z * (0.8 + 0.2 * t),
                    )
                })
                .collect();
            let last = pts[pts.len() - 1];
            *pts.last_mut().expect("a vine has points") = (last.0, 1.42 - length + 0.042, last.2);
            fsweep!(
                F,
                format!("vine{k}"),
                &pts,
                0.004,
                M.leaf_light,
                verts = 4,
                resolution = 2
            );
            for (i, &p) in pts.iter().enumerate().skip(1) {
                heart(
                    &format!("vine{k}_leaf{i}"),
                    p,
                    0.05 - 0.004 * i as f64,
                    50 * ((i * 3 + k) % 7) as i64 - 150,
                    150 + 10 * (i % 2) as i64,
                    if (i + k) % 2 == 1 {
                        M.leaf
                    } else {
                        M.leaf_light
                    },
                );
            }
        }
    });
    let plant = hinged(join(&parts, "plant", None), F.at(0, CEILING, 0));
    settle!(floor = false);
    // one sway as it's watered: out (eased) and, played back after the hold, home; a rock (out, past rest and home)
    // played back would swing it twice
    swing!(plant, "sway", 5.0, seconds = 0.8, axis = Axis::X);
    meshes()
}
