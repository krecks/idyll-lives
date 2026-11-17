//! Potted lemon tree: a banded terracotta pot, a slim stem forking into a round, clipped crown of small masses hung
//! with lemons, two metres tall in all (the ones picked at the front 1.62 m up for a grown-up, 1.42 for a teen and
//! 1.1 for a child), all picked at once (they shrink away, `ripen`); picked bare, no lemons until it fruits again.

use artkit::botany::skeleton::{Branches, ClumpAims, Heart, Mass};
use artkit::botany::species::{Aims, Crown, Hearts, Perches, Seeds, Species, Stage, Tree, grow};
use artkit::far::low;
use artkit::math::{TAU, cos, sin};
use artkit::perch::Find;
use artkit::scene::Obj;
use artkit::{fruiting, hang, lump, pot, sweep, trunk};

const CENTRE: (f64, f64, f64) = (0.0, 0.0, 1.5);
const RADII: (f64, f64, f64) = (0.4, 0.4, 0.34);

pub fn species() -> Species {
    Species {
        name: "lemonTree",
        seeds: Seeds {
            build: 86,
            perches: 122,
            cards: Some(222),
        },
        leaves: &["#3A6A30", "#467838", "#548640", "#62944A"],
        colours: &[
            ("fruit", "#F2D23C"),
            ("fruit2", "#F5DC50"),
            ("fruit3", "#E8C42E"),
        ],
        trunk: Stage::Own(potted),
        crown: Stage::Spec(Crown {
            centre: CENTRE,
            radii: RADII,
            tips: 12,
            limbs: 4,
            base_z: (0.95, 1.02),
            material: None,
            o: Branches {
                leader_z: Some(1.0),
                bottom: -0.6,
                spread: 0.95,
                limb_r: (0.02, 0.011),
                bough_r: (0.011, 0.005),
                fork: (0.4, 0.5),
                lift: 0.1,
                wander: 0.2,
                rise: 0.1,
                mass: Mass {
                    k: 3,
                    size: (0.17, 0.21),
                    ring: 0.13,
                    drop: (-0.08, 0.02),
                    squash: (0.8, 0.95),
                    noise: 0.3,
                    ..Mass::default()
                },
                ..Branches::default()
            },
        }),
        hearts: Stage::Spec(Hearts {
            n: 3,
            centre: CENTRE,
            radii: RADII,
            size: 0.24,
            o: Heart {
                out: 0.3,
                ..Heart::default()
            },
        }),
        // (on the leaves, before the lemons)
        perches: Stage::Spec(Perches {
            n: 12,
            aims: Aims::Clumps {
                o: ClumpAims {
                    per: 2,
                    ..ClumpAims::default()
                },
                more: Vec::new(),
            },
            limbs: None,
            o: Find {
                inset: 0.02,
                gap: 0.2,
                ..Find::default()
            },
        }),
        fruit: Some(fruit),
        ..Species::default()
    }
}

pub fn lemon_tree() -> Vec<Obj> {
    grow(&species())
}

/// The pot with its band, and the stem out of it.
fn potted(tree: &mut Tree) {
    let p = &tree.palette;
    let top = pot!(p, 0.3, 0.5, r_bottom = 0.22, rim = 0.016);
    // a raised band round the pot under its rim
    let (zb, rb) = (
        0.36,
        0.22 + 0.004 + (0.3 - 0.226) * (0.36 - 0.012) / (0.47 - 0.012),
    );
    let band: Vec<(f64, f64, f64)> = (0..24)
        .map(|k| {
            (
                rb * cos(TAU * k as f64 / 24.0),
                rb * sin(TAU * k as f64 / 24.0),
                zb,
            )
        })
        .collect();
    sweep!(
        "pot_band",
        &band,
        0.012,
        material = p.get("body"),
        verts = low(5, 3),
        closed = true
    );
    // a slim stem with a little bend, forking at a metre into four boughs under a round, clipped crown
    trunk!(
        p,
        &[
            (0.0, 0.0, top - 0.03),
            (0.01, 0.0, 0.62),
            (0.04, 0.02, 0.85),
            (0.03, 0.03, 1.02)
        ],
        0.032,
        &[1.2, 1.0, 0.9, 0.8],
        14.0,
        verts = 6,
        wobble = 0.0
    );
}

/// The lemons hung over the crown, and the three at the front that are picked.
fn fruit(tree: &mut Tree, still: Obj) -> Vec<Obj> {
    let (p, rng) = (&tree.palette, &mut tree.rng);
    let at = (0.1, -0.6, 1.62);
    // a teen's and a child's (I-garden-11): lower on the crown's front, where they reach (`FRUIT.lemon`)
    // (each 0.03 m in front of where a line in from the front meets the leaves: at 1.51 m up for the teen's, at
    // 1.29 m, the crown's underside, for the child's, which hangs under it)
    let teen_at = (0.15, -0.566, 1.42);
    let low_at = (-0.12, -0.4, 1.1);
    let fruit = || {
        hang!(
            rng,
            12,
            CENTRE,
            RADII,
            0.13,
            &[p.get("fruit"), p.get("fruit2"), p.get("fruit3")],
            "lemon",
            band = (-0.6, 0.6),
            flat = 1.25,
            segments = low(6, 5),
            rings = low(4, 3)
        );
        for (name, at, material) in [
            ("picked_lemon", at, "fruit"),
            ("teen_lemon", teen_at, "fruit3"),
            ("low_lemon", low_at, "fruit2"),
        ] {
            lump!(
                name,
                0.07,
                at,
                p.get(material),
                scale = (1, 1, 1.3),
                segments = 8,
                rings = 5,
                noise = 0.05
            );
        }
    };
    let seed = tree
        .species
        .seeds
        .cards
        .expect("the lemon tree wears leaf cards");
    fruiting!(
        p,
        "lemon",
        still,
        seed,
        CENTRE,
        fruit,
        tiles = tree.species.tiles.clone()
    )
}
