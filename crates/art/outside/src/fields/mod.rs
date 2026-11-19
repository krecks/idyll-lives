//! Fields and lanes between villages: the dry-stone walls, hedgerows, gates, stiles and fingerposts that hold the
//! boundaries and point the way. Built in the game's axes through `F`, like the farm and outdoor things.

use artkit::family::{Family, Model};
use artkit::palette;
use artkit::place::GameFrame;

mod gates;
mod hedges;
mod walls;

/// Built in the game's axes (x right, height, depth; front +depth).
pub const F: GameFrame = GameFrame::new(0.0);

palette! { M:
    stone = ("#8B8983", matte),         // the wall's face stones
    stone_dark = ("#6F6D68", matte),    // the shaded stones
    stone_light = ("#A6A39A", matte),   // the sunned stones and coping
    lichen = ("#7F855F", matte),        // the patches on the wall
    oak = ("#8A6642", wood),            // the gate's and fingerpost's timber
    oak_dark = ("#6E4F33", wood),       // its shaded rails and posts
    iron = ("#2B302E", metal),          // hinges, latch and the fingerpost
    paint = ("#EFEBDD", satin),         // the fingerpost's arms
}

/// The five things that divide and mark the fields.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "fields",
        M.palette(),
        vec![
            Model::new("dryStoneWall", walls::dry_stone_wall, (2.0, 1.15, 0.5))
                .tolerance(0.03)
                .paint(&[("stone", "Stone")]),
            Model::new("hedgerow", hedges::hedgerow, (1.95, 1.45, 0.95))
                .tolerance(0.03)
                .palette(hedges::colours())
                .habitat("hedge")
                .paint(&[("leaf", "Leaves")]),
            Model::new("gate", gates::gate, (1.9, 1.35, 0.21))
                .tolerance(0.03)
                .paint(&[("oak", "Wood")]),
            Model::new("stile", walls::stile, (0.95, 1.05, 0.8))
                .tolerance(0.03)
                .paint(&[("stone", "Stone")]),
            Model::new("fingerpost", gates::fingerpost, (0.85, 2.38, 0.43))
                .tolerance(0.03)
                .paint(&[("paint", "Paint")]),
        ],
        "Fields and lanes between villages (crates/art/outside/src/fields, built by cargo art fields): dry-stone walls, hedgerows, gates, stiles and fingerposts.",
    ))
}
