//! Magnolia: a smooth grey trunk parting low into upswept stems, a rounded pale pink crown of masses with big
//! upright white and pink goblets on them.

use artkit::botany::skeleton::{Branches, Buttress, ClumpAims, Heart, Mass, Stem};
use artkit::botany::species::{
    Aims, Crown, Hearts, Perches, Roots, Seeds, Species, Stage, Tree, Trunk, grow,
};
use artkit::bvh::Rays;
use artkit::far::{is_low, low};
use artkit::math::{atan2, cos, degrees, sin, v3};
use artkit::perch::Find;
use artkit::scene::Obj;
use artkit::{cupped, leaves};

const CENTRE: (f64, f64, f64) = (0.0, 0.0, 3.45);
const RADII: (f64, f64, f64) = (1.42, 1.22, 1.3);

/// A magnolia's bloom: a tall cup of petals, open at the top (its inside turned back down to the middle).
const GOBLET: [(f64, f64); 7] = [
    (0.0, 0.0),
    (0.045, 0.01),
    (0.1, 0.08),
    (0.12, 0.18),
    (0.1, 0.26),
    (0.078, 0.235),
    (0.0, 0.13),
];

pub fn species() -> Species {
    Species {
        name: "magnolia",
        seeds: Seeds {
            build: 17,
            perches: 117,
            cards: Some(217),
        },
        leaves: &["#EAAACA", "#F2BED6", "#F8D2E2", "#FCE2EE", "#FFF2F8"],
        colours: &[
            ("bark", "#D2CBC0"),
            ("fruit", "#FFFFFF"),
            ("fruit2", "#F2A2C4"),
            ("fruit3", "#E888B0"),
            ("green", "#6E9A48"),
            ("green2", "#628E40"),
        ],
        // a smooth grey trunk parting low into upswept stems, a rounded crown on them
        trunk: Stage::Spec(Trunk {
            pts: vec![
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 0.3),
                (0.04, 0.02, 0.9),
                (0.02, 0.0, 1.3),
            ],
            r: 0.18,
            scale: vec![1.4, 1.05, 0.98, 0.92],
            seed: 9.0,
            o: Stem {
                wobble: 0.015,
                ..Stem::default()
            },
        }),
        roots: Stage::Spec(Roots {
            n: 4,
            r: 0.25,
            h: 0.35,
            reach: 0.45,
            o: Buttress::default(),
        }),
        crown: Stage::Spec(Crown {
            centre: CENTRE,
            radii: RADII,
            tips: 11,
            limbs: 5,
            base_z: (1.0, 1.35),
            material: None,
            o: Branches {
                leader_z: Some(1.3),
                bottom: -0.55,
                spread: 0.95,
                limb_r: (0.085, 0.04),
                bough_r: (0.036, 0.014),
                fork: (0.55, 0.68),
                lift: 0.12,
                wander: 0.3,
                rise: 0.1,
                mass: Mass {
                    k: 3,
                    size: (0.46, 0.56),
                    ring: 0.38,
                    drop: (-0.28, 0.05),
                    squash: (0.78, 0.9),
                    noise: 0.3,
                    ..Mass::default()
                },
                ..Branches::default()
            },
        }),
        hearts: Stage::Spec(Hearts {
            n: 2,
            centre: CENTRE,
            radii: RADII,
            size: 0.75,
            o: Heart {
                out: 0.3,
                ..Heart::default()
            },
        }),
        extras: Some(blooms),
        perches: Stage::Spec(Perches {
            n: 20,
            aims: Aims::Clumps {
                o: ClumpAims {
                    per: 2,
                    ..ClumpAims::default()
                },
                // the two green clumps
                more: vec![
                    ((1.5, 0.8, 2.95), None, 0.0, true),
                    ((-1.3, -0.9, 3.65), None, 0.0, true),
                ],
            },
            limbs: None,
            o: Find {
                gap: 0.35,
                ..Find::default()
            },
        }),
        ..Species::default()
    }
}

pub fn magnolia() -> Vec<Obj> {
    grow(&species())
}

/// Two green clumps among the blossom, and the blooms.
fn blooms(tree: &mut Tree) {
    let p = &tree.palette;
    leaves!(
        "leaves",
        0.3,
        (1.5, 0.8, 2.72),
        p.get("green"),
        scale = (1, 1, 0.75),
        seed = 1.0
    );
    leaves!(
        "leaves2",
        0.28,
        (-1.3, -0.9, 3.44),
        p.get("green2"),
        scale = (1, 1, 0.73),
        seed = 2.0
    );
    // Blooms: upright goblets set into the crown's top and outer shoulders over each clump of its upper part (the
    // branch tips), two to a mass (one on the side of a lower one), white, pink and deep pink as a saucer
    // magnolia's. Each stands where a line out from its clump's middle leaves the crown, sunk a little into it, so
    // none stands off it (a far version: over the clumps it draws, on its smooth core).
    let cols = [
        p.get("fruit"),
        p.get("fruit2"),
        p.get("fruit"),
        p.get("fruit3"),
    ];
    let rays = Rays::of_scene();
    let mut blooms = Vec::new();
    // each mass's tip clump
    for (i, &((mx, my, mz), _, t, drawn)) in tree.clumps.iter().step_by(3).enumerate() {
        if t < -0.7 || !drawn {
            continue;
        }
        let heading = atan2(my, mx);
        let turns: &[(f64, f64)] = if t >= -0.25 {
            &[(0.3, 0.92), [(0.9, 0.55), (-1.0, 0.6)][i % 2]]
        } else {
            &[(0.0, 0.42)]
        };
        for &(turn, up) in turns {
            let a = heading + turn;
            let lean = 1.0 - up; // how far down the clump's side: 0 on top
            let d = v3(cos(a) * lean, sin(a) * lean, up).normalized();
            if let Some(hit) = rays.cast(v3(mx, my, mz) + d * 4, -d, 6.0) {
                blooms.push((hit.at - d * 0.05, a, lean));
            }
        }
    }
    for (i, &(at, a, lean)) in blooms.iter().enumerate() {
        if is_low() && i % 2 != 0 {
            continue; // far: half of them, on the tops
        }
        cupped!(
            format!("bloom{i}"),
            &GOBLET,
            at,
            cols[i % 4],
            yaw = 90.0 - degrees(a),
            tilt = 8.0 + 40.0 * lean,
            segments = low(6, 4)
        );
    }
}
