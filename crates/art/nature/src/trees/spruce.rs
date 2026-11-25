//! Blue spruce: a dense blue-green cone of nine whorls of drooping sprays, their tips turned up, limbed up to a
//! bare trunk to 2.2 m, as a garden spruce by a path is: residents walk under its skirt.

use artkit::botany::conifer::{TierAims, top};
use artkit::botany::skeleton::{Buttress, Stem};
use artkit::botany::species::{Aims, Perches, Roots, Seeds, Species, Stage, Tree, Trunk, grow};
use artkit::far::low;
use artkit::perch::Find;
use artkit::scene::Obj;
use artkit::{cone, conifer};

pub fn species() -> Species {
    Species {
        name: "spruce",
        seeds: Seeds {
            build: 20,
            perches: 120,
            cards: None,
        },
        leaves: &["#3A5E52", "#446A5C", "#4E7666", "#5A8270"],
        colours: &[("bark", "#FFF2E8")],
        trunk: Stage::Spec(Trunk {
            pts: vec![
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 0.3),
                (0.02, 0.0, 2.2),
                (0.0, 0.02, 6.0),
                (0.0, 0.0, 9.6),
            ],
            r: 0.18,
            scale: vec![1.35, 1.0, 0.9, 0.55, 0.12],
            seed: 12.0,
            o: Stem {
                verts: 7,
                wobble: 0.02,
                ..Stem::default()
            },
        }),
        roots: Stage::Spec(Roots {
            n: 4,
            r: 0.24,
            h: 0.32,
            reach: 0.4,
            o: Buttress::default(),
        }),
        crown: Stage::Own(crown),
        perches: Stage::Spec(Perches {
            n: 22,
            aims: Aims::Tiers(TierAims { per: 3 }),
            limbs: None,
            o: Find {
                inset: 0.04,
                extra: top(),
                ..Find::default()
            },
        }),
        ..Species::default()
    }
}

pub fn spruce() -> Vec<Obj> {
    grow(&species())
}

/// Nine whorls of drooping sprays, and the tip over them.
fn crown(tree: &mut Tree) {
    let (p, rng, cols) = (&tree.palette, &mut tree.rng, &tree.cols);
    tree.feet = conifer!(
        p,
        rng,
        9,
        5.5,
        2.2,
        0.82,
        1.85,
        cols,
        &[9, 9, 8, 8, 7, 6, 5, 4, 4],
        top_d = 0.7,
        droop = 0.45,
        lift = 0.05,
        thick = 0.42,
        offset = 0.06,
        bare = 1
    );
    cone!(
        "tip",
        0.21,
        0.02,
        1.2,
        at = (0, 0, 9.9),
        material = p.get("leaf4"),
        verts = low(8, 5)
    );
    // (the sprays a box a face: along their path, a bent spray stretched the needles over 2x)
    artkit::uv::box_named(&["spray"]);
}
