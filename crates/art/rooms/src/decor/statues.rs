//! D2: the marble bust and the cat and horse statues.

use super::common::{M, pedestal};
use artkit::forms::{Plane, rounded_rect};
use artkit::json::float_repr;
use artkit::math::{P3, TAU, cos, sin};
use artkit::parts::{join_all, new_parts, settle_all};
use artkit::scene::Obj;
use artkit::{boulder, cone, extrude, jitter, loft, revolve, ring, sphere, sweep};

/// A marble bust on a stone pedestal: a turned socle, a chest cut off at the shoulders, a neck, a head lofted from
/// chin to crown with a nose, eyelids, lips, a chin and ears, under a cap of curly hair.
pub fn bust_marble() -> Vec<Obj> {
    let y = 0.975; // (the head is taller, so the pedestal is lower: the bust keeps its height)
    // a head of life proportions (about 0.15 wide, 0.23 tall): the face's heights stretched up from the chin
    let hz = |z: f64| 0.335 + (z - 0.335) * 1.3;
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
        .map(|&(z, rx, ry, cy)| {
            ring!(rx * 1.12, ry * 1.04, z = y + hz(z), verts = 24, square = 2.2, at = (0.0, cy))
        })
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
            at = (0, -0.082, y + hz(0.452)),
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
                at = (sf * 0.032, -0.078, y + hz(0.437)),
                material = M.marble,
                scale = (0.026, 0.012, 0.012),
                segments = 10,
                rings = 5
            );
            sphere!(
                format!("ear{s}"),
                0.5,
                at = (sf * 0.078, 0.004, y + hz(0.425)),
                material = M.marble,
                scale = (0.018, 0.03, 0.048),
                rot = (0, 0, s * 20),
                segments = 10,
                rings = 6
            );
        }
        for (k, z) in [hz(0.384), hz(0.374)].into_iter().enumerate() {
            sphere!(
                format!("lip{k}"),
                0.5,
                at = (0, -0.078, y + z),
                material = M.marble,
                scale = (0.03, 0.012, 0.009),
                segments = 10,
                rings = 5
            );
        }
        sphere!(
            "chin",
            0.5,
            at = (0, -0.066, y + hz(0.352)),
            material = M.marble,
            scale = (0.036, 0.026, 0.026),
            segments = 10,
            rings = 6
        );
        // the hair: a curly cap from the brow over the crown to the nape
        let hair = boulder!(
            "hair",
            0.085,
            at = (0, 0.016, y + hz(0.4)),
            material = M.marble,
            scale = (1.08, 1.12, 1.0),
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
        // the body: one loft from the haunches up the back to the neck (rings of (half width, half depth, z, y of
        // the middle)), the chest leaning forward
        let body: Vec<Vec<P3>> = [
            (0.015, 0.105, 0.13, 0.04),
            (0.06, 0.115, 0.145, 0.045),
            (0.14, 0.105, 0.135, 0.035),
            (0.24, 0.088, 0.105, 0.0),
            (0.32, 0.072, 0.08, -0.035),
            (0.38, 0.06, 0.07, -0.055),
            (0.42, 0.045, 0.05, -0.065),
        ]
        .iter()
        .map(|&(z, rx, ry, y)| ring!(rx, ry, z = z, verts = 20, at = (0.0, y)))
        .collect();
        loft!("body", &body, material = M.concrete);
        for s in [-1, 1] {
            let sf = s as f64;
            // the haunch, folded against the body, the hind foot tucked under its front
            sphere!(
                format!("thigh{s}"),
                0.5,
                at = (sf * 0.09, 0.07, 0.1),
                material = M.concrete,
                scale = (0.1, 0.22, 0.2),
                segments = 16,
                rings = 10
            );
            sphere!(
                format!("hind_foot{s}"),
                0.5,
                at = (sf * 0.09, -0.04, 0.02),
                material = M.concrete,
                scale = (0.065, 0.13, 0.04),
                segments = 10,
                rings = 6
            );
        }
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
                format!("eye{s}"),
                0.5,
                at = (sf * 0.04, -0.152, 0.475),
                material = M.concrete_dark,
                scale = (0.026, 0.01, 0.016),
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
            // the front leg grows out of the chest and runs straight down into its paw
            sweep!(
                format!("leg{s}"),
                &[
                    (sf * 0.05, -0.04, 0.30),
                    (sf * 0.05, -0.10, 0.20),
                    (sf * 0.05, -0.115, 0.10),
                    (sf * 0.05, -0.12, 0.035)
                ],
                0.034,
                material = M.concrete,
                verts = 10,
                resolution = 2,
                scale = [1.1, 1.0, 0.8, 0.85]
            );
            sphere!(
                format!("paw{s}"),
                0.5,
                at = (sf * 0.05, -0.135, 0.02),
                material = M.concrete,
                scale = (0.07, 0.11, 0.045),
                segments = 10,
                rings = 6
            );
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
