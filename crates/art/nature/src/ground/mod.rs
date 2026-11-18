//! The ground cover, the woods' rocks and the selection marker: the lawn's wildflowers (`model.flowers`), drawn by
//! the thousand as thin instances on the town's tiles and round it (render/shared/nature.ts `lawns`; the grass
//! itself is built on the GPU, render/pc/grass.ts); the rock scattered in the woods (`model.rock`, nature.ts
//! `forests`, always its far version there, without a shadow); and the sparkle star over the selected resident
//! (`model.marker`, render/pc/selectionMarker.ts). Notes: NOTES.md.
//!
//! The flowers stay cheap (266 triangles in one draw call where the old part took 192 in two): every part on
//! `foliage`, which the nature shader sways and tints per instance, the heads with their stems. They have a far
//! version (`far_low`). The rock is two lumps cut flat a little under the ground, weathered, with pebbles at its
//! foot; its far version (what the woods draw) 56 triangles. The marker is one closed pillow-shaped star, which
//! the game draws in its own unlit material with an outline pushed out along its normals (inverted hull): built
//! smooth, its normals worked out from its shape, its origin at its middle, as the old part's.
//!
//! Run:  cargo art ground [--only marker] [--preview]

use artkit::family::{Family, Model};
use artkit::palette;

pub mod common;
mod flowers;
mod marker;
pub mod rock;

palette! { M:
    // the wildflowers: on `foliage` too, so the heads sway with their stems (one draw call)
    leaf = ("#5E8436", foliage),
    stem = ("#4F7A30", foliage),
    // (brighter than the old part's satin heads: the foliage finish's leaf texture darkens what it is on)
    petal_white = ("#FFFFFF", foliage),
    petal_yellow = ("#FFD84A", foliage),
    petal_violet = ("#B48CF0", foliage),
    petal_pink = ("#FFB3CC", foliage), // fallen blossom
    eye = ("#FFC72E", foliage),
    eye_orange = ("#F08A1E", foliage),
    eye_pale = ("#F2ECF6", foliage), // a cranesbill's pale centre
    // the rock
    stone = ("#8E8A82", matte),
    stone_light = ("#9A958C", matte),
    lichen = ("#9FA28E", matte), // pale grey-green lichen on the tops
    soil = ("#6F675C", matte), // the damp, earthy foot
    // the marker: the game draws it in its own unlit material, tinted by the resident's mood
    star = ("#FFFFFF", satin),
}

/// (The gate's `float`: these lie on the lawn, at the tips of its blades.)
const AT_THE_TIPS: &str =
    "lies on the tips of the lawn's blades, a few centimetres up, not under them";

/// The family.
pub fn family() -> Option<Family> {
    let drift = |model: Model| {
        model
            .far_low()
            .code()
            .ground(false)
            .on_wall()
            .accept("float", AT_THE_TIPS)
    };
    Some(Family::new(
        "ground",
        M.palette(),
        vec![
            Model::new("flowers", flowers::flowers, (0.143, 0.274, 0.165))
                .key("model.flowers")
                .far_low(),
            drift(
                Model::new("daisies", flowers::daisies, (0.578, 0.034, 0.466))
                    .key("model.flowers.daisies")
                    .tolerance(0.15),
            ),
            drift(
                Model::new("buttercups", flowers::buttercups, (0.581, 0.049, 0.436))
                    .key("model.flowers.buttercups")
                    .tolerance(0.15),
            ),
            drift(
                Model::new("violets", flowers::violets, (0.462, 0.039, 0.384))
                    .key("model.flowers.violets")
                    .tolerance(0.15),
            ),
            drift(
                Model::new("petals", flowers::petals, (4.948, 0.06, 4.948))
                    .key("model.flowers.petals"),
            ),
            Model::new("rock", rock::rock, (1.137, 0.55, 0.829))
                .key("model.rock")
                .centre(rock::ROCK_CENTRE.0, rock::ROCK_CENTRE.1)
                .on_wall()
                .far_low()
                .tolerance(0.05)
                .sink(-rock::GROUND),
            Model::new("marker", marker::marker, (0.23, 0.34, 0.064))
                .key("model.marker")
                .on_wall()
                .sink(0.17),
        ],
        "The lawn's wildflowers, the woods' rocks and the selection marker (crates/art/nature/src/ground, \
         built by cargo art).",
    ))
}
