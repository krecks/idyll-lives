//! Silver birch: a slim white stem with dark lenticels, black and fissured at its foot, slender white limbs up it
//! from 3 m and an airy, upright crown of small masses hanging from their twigs.

use artkit::botany::Tiles;
use artkit::botany::skeleton::{_along, Branches, ClumpAims, Heart, LimbPoints, Mass};
use artkit::botany::species::{Aims, Crown, Hearts, Perches, Seeds, Species, Stage, Tree, grow};
use artkit::far::{is_low, low};
use artkit::math::{TAU, cos, sin};
use artkit::mesh::UvWay;
use artkit::noise::noise_at;
use artkit::perch::Find;
use artkit::scene::Obj;
use artkit::{jitter, sweep};

/// The stem's line and how thick it is at each point of it, of its radius.
const LINE: [(f64, f64, f64); 6] = [
    (0.0, 0.0, 0.0),
    (0.0, 0.0, 0.6),
    (0.04, -0.02, 2.2),
    (0.02, 0.03, 4.0),
    (-0.03, 0.02, 5.8),
    (0.0, -0.02, 7.6),
];
const SCALE: [f64; 6] = [1.12, 1.0, 0.88, 0.7, 0.5, 0.28];
const CENTRE: (f64, f64, f64) = (0.0, 0.0, 5.0);
const RADII: (f64, f64, f64) = (1.1, 1.1, 2.6);

pub fn species() -> Species {
    Species {
        name: "birch",
        seeds: Seeds {
            build: 13,
            perches: 113,
            cards: Some(213),
        },
        leaves: &["#76963E", "#86A646", "#98B650", "#AAC45C", "#BCD26A"],
        tiles: Tiles::of("birch"),
        // (its white bark as light as a birch's is, the black marks and the fissured foot in it)
        colours: &[("bark", "#8E867C"), ("birch_bark", "#A29F98")],
        trunk: Stage::Own(trunk),
        // slender limbs up the stem from 3 m, rising, then their twigs hanging in long, light masses: an airy,
        // upright crown
        crown: Stage::Spec(Crown {
            centre: CENTRE,
            radii: RADII,
            tips: 22,
            limbs: 7,
            base_z: (2.7, 5.6),
            material: Some("birch_bark"),
            o: Branches {
                leader_z: Some(6.2),
                bottom: -0.55,
                spread: 0.95,
                axis: 0.3,
                limb_r: (0.055, 0.025),
                bough_r: (0.024, 0.01),
                // (round enough to read as round: four sides showed four hard edges along each white bough)
                bough_verts: 6,
                fork: (0.55, 0.7),
                lift: 0.3,
                wander: 0.25,
                rise: 0.05,
                mass: Mass {
                    k: 3,
                    size: (0.44, 0.56),
                    ring: 0.3,
                    drop: (-0.5, -0.15),
                    squash: (1.1, 1.35),
                    noise: 0.36,
                    side: 0.9,
                    ..Mass::default()
                },
                ..Branches::default()
            },
        }),
        hearts: Stage::Spec(Hearts {
            n: 3,
            centre: CENTRE,
            radii: RADII,
            size: 0.85,
            o: Heart {
                out: 0.25,
                ..Heart::default()
            },
        }),
        perches: Stage::Spec(Perches {
            n: 20,
            aims: Aims::Clumps {
                o: ClumpAims {
                    per: 2,
                    ..ClumpAims::default()
                },
                more: Vec::new(),
            },
            limbs: Some(LimbPoints::default()),
            o: Find {
                limbs_n: Some(2),
                ..Find::default()
            },
        }),
        ..Species::default()
    }
}

pub fn birch() -> Vec<Obj> {
    grow(&species())
}

/// A slim white stem with a slight lean and wave all the way up as the leader, black and fissured at its foot,
/// and its dark marks.
fn trunk(tree: &mut Tree) {
    let (p, rng) = (&tree.palette, &mut tree.rng);
    let full = !is_low();
    sweep!(
        "trunk",
        &LINE,
        0.15,
        material = p.get("birch_bark"),
        verts = low(9, 5),
        resolution = low(3, 1),
        scale = SCALE
    );
    let foot = sweep!(
        "foot",
        &[(0.0, 0.0, 0.0), (0.0, 0.0, 0.35), (0.01, 0.0, 0.75)],
        0.172,
        material = p.get("bark"),
        verts = low(10, 5),
        resolution = 1,
        scale = [1.04, 0.94, 0.86],
        caps = false
    );
    // (round its own axis: along its path, its ragged top stretched the bark over 2x)
    foot.edit(|part| part.uv = UvWay::Round);
    if full {
        jitter!(foot, 0.018, size = 0.12, seed = 3.0, up = 0.0);
        foot.edit(|part| {
            // its top edge ragged, as the black bark climbs
            for v in &mut part.mesh.verts {
                if v.z as f64 > 0.5 {
                    v.z = (v.z as f64 + 0.3 * noise_at(v.x as f64 * 14.0, v.y as f64 * 14.0, 1.7))
                        as f32;
                }
            }
        });
    }
    // the dark marks: short lenticels across the bark, and a scar under each limb
    for i in 0..22 {
        let z = 0.9 + i as f64 * 0.27 + rng.uniform(-0.1, 0.1);
        let (a, span) = (rng.uniform(0.0, TAU), rng.uniform(0.9, 1.8));
        // a thin dark bar round the stem, its faces under 12 mm across (a lenticel, too narrow to bevel), its
        // middle a little in from the stem's round so it stands proud of its flat sides and of its corners
        let rr = 0.97 * 0.15 * _along(&LINE, &SCALE, z);
        if full {
            // (a short dash, 0.5-1.1 rad round: three points keep it on the stem)
            let span = span * 0.6;
            let across: Vec<(f64, f64, f64)> = (0..3)
                .map(|k| {
                    (
                        rr * cos(a + span * (k as f64 / 2.0 - 0.5)),
                        rr * sin(a + span * (k as f64 / 2.0 - 0.5)),
                        z,
                    )
                })
                .collect();
            sweep!(
                format!("mark{i}"),
                &across,
                0.0068,
                material = p.get("birch_mark"),
                verts = 3,
                scale = [0.4, 1.0, 0.4]
            );
        }
    }
}
