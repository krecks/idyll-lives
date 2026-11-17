//! Maple (a Norway maple): a straight trunk with a leader and six steep limbs forking into boughs under one dense,
//! even dome of eighteen leaf masses round a dark heart, in a cool, deep green; not the oak's few big billows.

use artkit::botany::skeleton::{Branches, Buttress, ClumpAims, Heart, Mass, Stem};
use artkit::botany::species::{
    Aims, Crown, Hearts, Perches, Roots, Seeds, Species, Stage, Trunk, grow,
};
use artkit::perch::Find;
use artkit::scene::Obj;

const CENTRE: (f64, f64, f64) = (0.0, 0.0, 4.95);
const RADII: (f64, f64, f64) = (1.9, 1.9, 1.85);

pub fn species() -> Species {
    Species {
        name: "maple",
        seeds: Seeds {
            build: 12,
            perches: 112,
            cards: Some(212),
        },
        leaves: &["#3E6E34", "#4A7C3A", "#588A42", "#68984A", "#7AA654"],
        // a straight trunk, a little flared, carrying on as the leader into the crown
        trunk: Stage::Spec(Trunk {
            pts: vec![
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 0.3),
                (0.02, -0.01, 1.4),
                (-0.01, 0.02, 2.6),
                (0.0, 0.0, 3.5),
            ],
            r: 0.24,
            scale: vec![1.4, 1.05, 0.98, 0.9, 0.78],
            seed: 5.0,
            o: Stem::default(),
        }),
        roots: Stage::Spec(Roots {
            n: 5,
            r: 0.33,
            h: 0.45,
            reach: 0.55,
            o: Buttress::default(),
        }),
        // steep limbs all round from 2.5-3.4 m, forking into boughs that fill an even dome; a dark heart closes it
        crown: Stage::Spec(Crown {
            centre: CENTRE,
            radii: RADII,
            tips: 18,
            limbs: 6,
            base_z: (2.4, 3.2),
            material: None,
            o: Branches {
                leader_z: Some(3.3),
                bottom: -0.5,
                spread: 0.98,
                limb_r: (0.1, 0.045),
                bough_r: (0.042, 0.015),
                fork: (0.45, 0.55),
                lift: 0.1,
                rise: 0.12,
                mass: Mass {
                    k: 3,
                    size: (0.5, 0.62),
                    ring: 0.4,
                    drop: (-0.25, 0.1),
                    squash: (0.82, 0.95),
                    noise: 0.26,
                    ..Mass::default()
                },
                ..Branches::default()
            },
        }),
        hearts: Stage::Spec(Hearts {
            n: 6,
            centre: CENTRE,
            radii: RADII,
            size: 1.15,
            o: Heart {
                out: 0.36,
                ..Heart::default()
            },
        }),
        perches: Stage::Spec(Perches {
            n: 24,
            aims: Aims::Clumps {
                o: ClumpAims {
                    per: 2,
                    ..ClumpAims::default()
                },
                more: Vec::new(),
            },
            limbs: None,
            o: Find::default(),
        }),
        ..Species::default()
    }
}

pub fn maple() -> Vec<Obj> {
    grow(&species())
}
