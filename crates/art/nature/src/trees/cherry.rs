//! Cherry blossom: a short, leaning trunk opening into wide, low limbs, and a broad, flat crown of pink blossom
//! clouds at their boughs' tips.

use artkit::botany::Tiles;
use artkit::botany::skeleton::{Branches, Buttress, ClumpAims, Heart, Mass, Stem};
use artkit::botany::species::{
    Aims, Crown, Hearts, Perches, Roots, Seeds, Species, Stage, Tree, Trunk, grow,
};
use artkit::leaves;
use artkit::perch::Find;
use artkit::scene::Obj;

const CENTRE: (f64, f64, f64) = (0.0, 0.0, 3.85);
const RADII: (f64, f64, f64) = (1.95, 1.95, 1.15);

pub fn species() -> Species {
    Species {
        name: "cherry",
        seeds: Seeds {
            build: 14,
            perches: 114,
            cards: Some(214),
        },
        leaves: &["#E07AAE", "#EA90BE", "#F2A8CE", "#F8BEDC", "#FCD4E8"],
        tiles: Tiles::of("cherry").green_of("apple"),
        colours: &[("bark", "#B09A8A")],
        // a short trunk, leaning a little, that opens at 1.6-2 m into wide, low limbs reaching out nearly level
        // and turning up at their ends: a broad vase under a flat crown of blossom clouds
        trunk: Stage::Spec(Trunk {
            pts: vec![
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 0.3),
                (0.05, 0.02, 1.0),
                (0.1, 0.0, 1.8),
            ],
            r: 0.225,
            scale: vec![1.4, 1.05, 0.98, 0.88],
            seed: 6.0,
            o: Stem::default(),
        }),
        roots: Stage::Spec(Roots {
            n: 5,
            r: 0.3,
            h: 0.4,
            reach: 0.55,
            o: Buttress::default(),
        }),
        crown: Stage::Spec(Crown {
            centre: CENTRE,
            radii: RADII,
            tips: 16,
            limbs: 5,
            base_z: (1.55, 1.95),
            material: None,
            o: Branches {
                leader_z: None,
                bottom: -0.45,
                spread: 0.95,
                limb_r: (0.11, 0.05),
                bough_r: (0.045, 0.016),
                fork: (0.5, 0.62),
                lift: 0.22,
                wander: 0.3,
                rise: 0.2,
                mass: Mass {
                    k: 3,
                    size: (0.5, 0.62),
                    ring: 0.42,
                    drop: (-0.25, 0.05),
                    squash: (0.6, 0.72),
                    noise: 0.36,
                    ..Mass::default()
                },
                ..Branches::default()
            },
        }),
        hearts: Stage::Spec(Hearts {
            n: 3,
            centre: CENTRE,
            radii: RADII,
            size: 0.75,
            o: Heart {
                out: 0.35,
                ..Heart::default()
            },
        }),
        extras: Some(green),
        perches: Stage::Spec(Perches {
            n: 22,
            aims: Aims::Clumps {
                o: ClumpAims::default(),
                // the two green clumps
                more: vec![
                    ((2.0, -0.9, 3.5), None, 0.0, true),
                    ((-1.8, 1.2, 3.6), None, 0.0, true),
                ],
            },
            limbs: None,
            o: Find::default(),
        }),
        ..Species::default()
    }
}

pub fn cherry() -> Vec<Obj> {
    grow(&species())
}

/// Two green clumps among the blossom.
fn green(tree: &mut Tree) {
    let p = &tree.palette;
    leaves!(
        "leaves",
        0.4,
        (2.0, -0.9, 3.25),
        p.get("green"),
        scale = (1, 1, 0.62),
        seed = 1.0
    );
    leaves!(
        "leaves2",
        0.35,
        (-1.8, 1.2, 3.38),
        p.get("green2"),
        scale = (1, 1, 0.64),
        seed = 2.0
    );
}
