//! The garden's trees (G5): the oak (`model.tree`), maple, silver birch, cherry blossom, apple, Japanese maple,
//! magnolia and weeping willow (`model.tree.*`), the pine, blue spruce and Italian cypress (`model.pine`,
//! `model.spruce`, `model.cypress`) and the potted lemon tree (`model.garden.lemonTree`).
//!
//! Broadleaves are real branching skeletons: a swept, tapering trunk on root buttresses, limbs found from where the
//! crown's leaf masses are, each forking into curving boughs, and a mass of leaves at each bough's tip, lighter on
//! top, with gaps between (artkit's `botany::skeleton`; the oak first, by hand): leaf cards over a dark core
//! (`botany::leaves`, `clad`: cut-out clusters of leaves from one atlas, on `leafcards`). Conifers are whorls of
//! drooping sprays of needles round a trunk (`botany::conifer`), the cypress a flame of upright sprays, on
//! `foliage`; trunks are on `bark`. The nature shader sways and tints all three. Trees stand on every lot and by the
//! thousand in the woods, so each keeps under 4,500 triangles, and each has a far version (the same build at low
//! detail, every random number drawn, one leafy lump (`botany::lump`, on `foliage`) a mass, no boughs; about a fifth
//! of the triangles), which the far woods and the town overview draw.
//!
//! A species is a module: its numbers as a `botany::species::Species` (its seeds, its colours, a spec a stage where
//! it grows as the others do) and the stages that are its own.
//!
//! Run:  cargo art trees [--only oak]

use artkit::botany::skeleton::base;
use artkit::family::{Family, Model};
use artkit::json;

pub mod apple;
pub mod birch;
pub mod cherry;
pub mod cypress;
pub mod japanese_maple;
pub mod lemon_tree;
pub mod magnolia;
pub mod maple;
pub mod oak;
pub mod pine;
pub mod spruce;
pub mod willow;

/// A resident picking fruit (`pickApple*`, `pickLemon*` in web/src/render/shared/characters/clips/uses.ts, fitted
/// to where the picked fruit hangs): it shrinks away as the hand closes on it, this long into the use.
const PICK_DELAY: f64 = 1.6;

/// (width, height, depth) as measured. A tree's far version may differ from it by up to `TOLERANCE`.
const TOLERANCE: f64 = 0.1;

/// The family.
pub fn family() -> Family {
    let tree = |model: Model| model.tolerance(TOLERANCE).far_low().habitat("tree");
    let conifer = |model: Model| model.tolerance(TOLERANCE).far_low().habitat("conifer");
    Family::new(
        "trees",
        base(),
        vec![
            tree(Model::new("oak", oak::oak, (5.488, 7.473, 5.23)).key("model.tree")),
            tree(Model::new("maple", maple::maple, (4.983, 7.312, 5.335)).key("model.tree.maple")),
            tree(Model::new("birch", birch::birch, (3.446, 8.184, 3.293)).key("model.tree.birch")),
            tree(
                Model::new("cherry", cherry::cherry, (5.251, 5.454, 5.14))
                    .key("model.tree.blossom"),
            ),
            tree(
                Model::new("apple", apple::apple, (4.29, 4.106, 4.642))
                    .key("model.tree.fruit")
                    .motions(json!({
                        "uses": {
                            "pick": {"clip": "pick", "delay": PICK_DELAY},
                            "harvest": {"clip": "pick", "delay": PICK_DELAY}
                        },
                        "growth": "ripen"
                    })),
            ),
            tree(
                Model::new(
                    "japaneseMaple",
                    japanese_maple::japanese_maple,
                    (3.723, 4.153, 3.828),
                )
                .key("model.tree.japaneseMaple")
                .motions(json!({"uses": {"shape": "trim"}})),
            ),
            tree(
                Model::new("magnolia", magnolia::magnolia, (4.1, 5.076, 3.756))
                    .key("model.tree.magnolia"),
            ),
            tree(
                Model::new("willow", willow::willow, (4.947, 6.1, 5.052)).key("model.tree.willow"),
            ),
            conifer(Model::new("pine", pine::pine, (6.031, 9.2, 6.054)).key("model.pine")),
            conifer(Model::new("spruce", spruce::spruce, (5.188, 10.5, 5.154)).key("model.spruce")),
            conifer(
                Model::new("cypress", cypress::cypress, (1.012, 7.95, 1.025)).key("model.cypress"),
            ),
            tree(
                Model::new("lemonTree", lemon_tree::lemon_tree, (1.288, 1.994, 1.344))
                    .key("model.garden.lemonTree")
                    .paint(&[("body", "Pot")])
                    .motions(json!({"growth": "ripen"})),
            ),
        ],
        "The trees (crates/art/nature/src/trees, built by cargo art): the G5 garden trees, the woods' oaks, maples, \
         birches, pines and spruces, and the potted lemon tree. Listed in the base manifest's packs. Leaves are on \
         `foliage` and trunks on `bark`, which the nature shader sways and tints. Each has a far version (`far`), \
         which the far woods and the town overview draw. Clips: an apple shrinks away while it is picked, the fruit \
         follows the tree's ripeness (picked bare, the lemons all go), the Japanese maple's stray sprigs while it is \
         shaped.",
    )
}
