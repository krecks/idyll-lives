//! Pine: a tall bare trunk and six whorls of drooping sprays narrowing to a point, the lowest from 2.35 m (its bare
//! boughs showing from below): residents walk under them.

use artkit::botany::conifer::{TierAims, top};
use artkit::botany::skeleton::{Buttress, Stem};
use artkit::botany::species::{Aims, Perches, Roots, Seeds, Species, Stage, Tree, Trunk, grow};
use artkit::far::low;
use artkit::perch::Find;
use artkit::scene::Obj;
use artkit::{cone, conifer};

pub fn species() -> Species {
    Species {
        name: "pine",
        seeds: Seeds {
            build: 19,
            perches: 119,
            cards: None,
        },
        leaves: &["#335C38", "#3D683E", "#477444", "#51804A"],
        colours: &[("bark", "#FFFFFF")],
        trunk: Stage::Spec(Trunk {
            pts: vec![
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 0.3),
                (0.03, 0.02, 2.4),
                (-0.02, 0.0, 5.0),
                (0.0, 0.02, 8.4),
            ],
            r: 0.19,
            scale: vec![1.35, 1.0, 0.9, 0.6, 0.15],
            seed: 11.0,
            o: Stem {
                verts: 8,
                wobble: 0.02,
                ..Stem::default()
            },
        }),
        roots: Stage::Spec(Roots {
            n: 4,
            r: 0.26,
            h: 0.35,
            reach: 0.42,
            o: Buttress::default(),
        }),
        crown: Stage::Own(crown),
        perches: Stage::Spec(Perches {
            n: 20,
            aims: Aims::Tiers(TierAims::default()),
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

pub fn pine() -> Vec<Obj> {
    grow(&species())
}

/// Six whorls of drooping sprays, and the tip over them.
fn crown(tree: &mut Tree) {
    let (p, rng, cols) = (&tree.palette, &mut tree.rng, &tree.cols);
    tree.feet = conifer!(
        p,
        rng,
        6,
        6.5,
        2.35,
        1.0,
        2.1,
        cols,
        &[11, 10, 9, 8, 7, 6],
        top_d = 0.9,
        droop = 0.4,
        lift = 0.04,
        thick = 0.36,
        offset = 0.15
    );
    cone!(
        "tip",
        0.25,
        0.02,
        1.3,
        at = (0, 0, 8.55),
        material = p.get("leaf3"),
        verts = low(8, 5)
    );
    // (the sprays a box a face: along their path, a bent spray stretched the needles over 2x)
    crate::box_uvs(&["spray"]);
}
