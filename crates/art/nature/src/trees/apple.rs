//! Apple tree: a garden apple, a semi-dwarf: a short, crooked trunk, gnarled limbs and a round crown, its middle
//! 2.6 m up, hung with red and yellow apples (set where lines in meet its leaves, `_hang`), and laden boughs out of
//! it in front with apples on them from 1.12 to 1.75 m up (`LOW_BOUGHS`): one a resident picks (it shrinks away),
//! low enough for a child on tiptoe, and those the harvest's reaches close on. Picked bare, no apples until it
//! fruits again (`ripen`).

use artkit::botany::skeleton::{Branches, Buttress, ClumpAims, Heart, Mass, Stem, base};
use artkit::botany::species::{
    Aims, Crown, Hearts, Perches, Roots, Seeds, Species, Stage, Tree, Trunk, grow,
};
use artkit::far::low;
use artkit::materials::{Mat, Palette};
use artkit::perch::Find;
use artkit::scene::Obj;
use artkit::{bough, fruiting, hang, leaves, lump, tube};

const CENTRE: (f64, f64, f64) = (0.0, 0.0, 2.6);
const RADII: (f64, f64, f64) = (1.5, 1.5, 1.15);

// The apple tree's low fruit (I-garden-11, I-garden-12): a garden apple (a semi-dwarf) carries its fruit from about
// 1.1 m up, so its crown's middle stands at 2.6 m (its leaves from 1.6 m up) and laden boughs hang out of it in
// front, under the crown's edge, apples on spurs along them. Each apple is (to the right of a picker facing the
// tree, out in front of the trunk, its bottom), in metres. `PICKED` is the one a resident picks to eat (its own
// part: it shrinks away), low enough for every size of picker, a child on tiptoe too (`FRUIT.apple` and
// `pickApple*` in characters/clips/uses.ts). The harvest's reaches (`pickHigh*` in the same file, from the tile in
// front, 1 m out) close on `HARVEST`'s: a grown-up's, a teen's and a child's. Placed, the tree isn't turned or sized
// at random (no `vary` in the base manifest), so the boughs face the tile in front at these heights.
const PICKED: (f64, f64, f64) = (-0.06, 0.62, 1.13);
const HARVEST: [(&str, (f64, f64, f64)); 3] = [
    ("177", (0.16, 0.62, 1.75)),
    ("147", (0.3, 0.62, 1.45)),
    ("112", (0.24, 0.78, 1.12)),
];
/// A bough passes this far over the bottom of each apple it carries (an apple 0.2 m, its stalk).
const OVER: f64 = 0.27;

/// A point a laden bough runs through: (r, out, h) as the apples are, or over the apple named.
enum Via {
    At(f64, f64, f64),
    Over(&'static str),
}

// The boughs: from the trunk out over their apples (named) to a tip, through points (r, out, h) as above, leaves
// near the trunk and at the tip. They end within 0.85 m of the trunk, short of where a picker stands (1 m out).
const LOW_BOUGHS: [&[Via]; 4] = [
    // straight out in front over the picked apple, dipping
    &[
        Via::At(-0.02, 0.1, 1.48),
        Via::At(-0.07, 0.36, 1.62),
        Via::Over("picked"),
        Via::At(-0.12, 0.74, 1.38),
    ],
    // to the right, up under the crown: the grown-up's apple and another
    &[
        Via::At(0.06, 0.1, 1.74),
        Via::At(0.12, 0.34, 1.98),
        Via::Over("177"),
        Via::Over("a"),
        Via::At(0.4, 0.84, 1.9),
    ],
    // to the right and low, bowed down by its fruit: the teen's and the child's
    &[
        Via::At(0.08, 0.1, 1.44),
        Via::At(0.27, 0.32, 1.64),
        Via::Over("147"),
        Via::Over("112"),
        Via::At(0.2, 0.84, 1.36),
    ],
    // to the left, two more
    &[
        Via::At(-0.06, 0.1, 1.6),
        Via::At(-0.34, 0.28, 1.84),
        Via::Over("b"),
        Via::Over("c"),
        Via::At(-0.68, 0.58, 1.6),
    ],
];
const LOW_APPLES: [(&str, (f64, f64, f64)); 3] = [
    ("a", (0.34, 0.78, 1.66)),
    ("b", (-0.45, 0.4, 1.55)),
    ("c", (-0.6, 0.5, 1.42)),
];

pub fn species() -> Species {
    Species {
        name: "apple",
        seeds: Seeds {
            build: 15,
            perches: 115,
            cards: Some(215),
        },
        leaves: &["#52803A", "#629040", "#74A048", "#86AE52", "#98BC5E"],
        // a short, crooked trunk opening low into gnarled limbs that spread wide, a round crown on them
        trunk: Stage::Spec(Trunk {
            pts: vec![
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 0.25),
                (0.07, 0.04, 0.75),
                (0.02, 0.08, 1.15),
                (0.05, 0.05, 1.5),
            ],
            r: 0.2,
            scale: vec![1.4, 1.05, 1.0, 0.92, 0.85],
            seed: 7.0,
            o: Stem {
                wobble: 0.04,
                ..Stem::default()
            },
        }),
        roots: Stage::Spec(Roots {
            n: 4,
            r: 0.28,
            h: 0.4,
            reach: 0.5,
            o: Buttress::default(),
        }),
        crown: Stage::Spec(Crown {
            centre: CENTRE,
            radii: RADII,
            tips: 13,
            limbs: 5,
            base_z: (1.2, 1.5),
            material: None,
            o: Branches {
                leader_z: Some(1.5),
                bottom: -0.4,
                spread: 0.95,
                limb_r: (0.1, 0.045),
                bough_r: (0.042, 0.015),
                fork: (0.5, 0.62),
                lift: 0.22,
                wander: 0.45,
                rise: 0.15,
                mass: Mass {
                    k: 3,
                    size: (0.5, 0.62),
                    ring: 0.4,
                    drop: (-0.15, 0.05),
                    squash: (0.78, 0.9),
                    noise: 0.32,
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
        extras: Some(|tree| _low_boughs(&tree.palette, &tree.cols)),
        // (on the leaves, before the fruit)
        perches: Stage::Spec(Perches {
            n: 20,
            aims: Aims::Clumps {
                o: ClumpAims {
                    per: 3,
                    ..ClumpAims::default()
                },
                more: Vec::new(),
            },
            limbs: None,
            o: Find::default(),
        }),
        fruit: Some(fruit),
        ..Species::default()
    }
}

pub fn apple() -> Vec<Obj> {
    grow(&species())
}

/// The laden boughs out of the apple tree's crown in front (`LOW_BOUGHS`): each a bark sweep from the trunk over
/// its apples, with leaves where it leaves the crown and a sprig at its tip.
fn _low_boughs(p: &Palette, cols: &[Mat]) {
    let named = |name: &str| {
        let all = [("picked", PICKED)]
            .into_iter()
            .chain(HARVEST)
            .chain(LOW_APPLES);
        match all.into_iter().find(|(n, _)| *n == name) {
            Some((_, at)) => at,
            None => panic!("no apple {name:?}"),
        }
    };
    for (i, path) in LOW_BOUGHS.iter().enumerate() {
        let mut pts = Vec::new();
        for via in *path {
            let (r, out, h) = match *via {
                Via::Over(name) => (named(name).0, named(name).1, named(name).2 + OVER),
                Via::At(r, out, h) => (r, out, h),
            };
            pts.push((-r, -out, h));
        }
        bough!(
            format!("low_bough{i}"),
            &pts,
            0.034,
            0.012,
            p,
            verts = 4,
            res = 2
        );
        let (near, tip) = (pts[1], pts[pts.len() - 1]);
        leaves!(
            format!("low_leaves{i}"),
            0.2,
            (near.0, near.1, near.2 - 0.08),
            cols[1 + i % 2],
            scale = (1.15, 1.0, 0.75),
            noise = 0.3,
            seed = 4.2 + i as f64,
            segments = 6,
            rings = 5,
            cards = 6
        );
        leaves!(
            format!("low_tip{i}"),
            0.1,
            (tip.0, tip.1, tip.2 - 0.06),
            cols[2 + i % 2],
            scale = (1.2, 1.0, 0.7),
            noise = 0.3,
            seed = 7.7 + i as f64,
            segments = 5,
            rings = 3,
            cards = 3,
            core = false
        );
    }
}

/// `_low_apple`'s options.
struct LowApple {
    r: f64,
    segments: usize,
    rings: usize,
}

impl Default for LowApple {
    fn default() -> LowApple {
        LowApple {
            r: 0.1,
            segments: 6,
            rings: 4,
        }
    }
}

/// An apple (radius `r`) hanging from a bough on its stalk, its bottom at `at` (r, out, h as `LOW_BOUGHS`).
fn _low_apple(name: &str, at: (f64, f64, f64), material: Mat, o: LowApple) {
    let (x, y, z) = (-at.0, -at.1, at.2);
    lump!(
        name,
        o.r,
        (x, y, z),
        material,
        segments = o.segments,
        rings = o.rings,
        noise = 0.05
    );
    tube!(
        format!("{name}_stalk"),
        &[
            (x, y, z + 2.0 * o.r - 0.02),
            (x + 0.004, y - 0.004, z + OVER + 0.01)
        ],
        0.011,
        0.008,
        base().get("stalk"),
        verts = 4
    );
}

/// The apples hung over the crown and on the low boughs, and the one a resident picks.
fn fruit(tree: &mut Tree, still: Obj) -> Vec<Obj> {
    let (p, rng) = (&tree.palette, &mut tree.rng);
    let fruit = || {
        hang!(
            rng,
            low(9, 6),
            CENTRE,
            RADII,
            0.2,
            &[
                p.get("fruit"),
                p.get("fruit2"),
                p.get("fruit3"),
                p.get("fruit4")
            ],
            "apple",
            band = (-0.7, 0.45),
            segments = low(6, 5),
            rings = low(4, 3)
        );
        for (i, (_, at)) in HARVEST.into_iter().chain(LOW_APPLES).enumerate() {
            _low_apple(
                &format!("low_apple{i}"),
                at,
                [p.get("fruit"), p.get("fruit2"), p.get("fruit4")][i % 3],
                LowApple {
                    segments: low(6, 5),
                    rings: low(4, 3),
                    ..LowApple::default()
                },
            );
        }
    };
    let picked = || {
        _low_apple(
            "picked_apple",
            PICKED,
            p.get("fruit"),
            LowApple {
                r: 0.11,
                segments: 8,
                rings: 5,
            },
        )
    };
    let (x, out, h) = PICKED;
    let seed = tree
        .species
        .seeds
        .cards
        .expect("the apple wears leaf cards");
    fruiting!(
        p,
        "apple",
        still,
        seed,
        CENTRE,
        fruit,
        picked = picked,
        at = (-x, -out, h + 0.11),
        tiles = tree.species.tiles.clone()
    )
}
