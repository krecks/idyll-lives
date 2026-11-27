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
    stone = ("#7A756C", matte),         // the wall's field stones: the boulders' weathered greys (garden_decor)
    stone_dark = ("#605C55", matte),    // the darker stones
    stone_light = ("#8A8174", matte),   // the warmer stones, coping and steps
    lichen = ("#7F855F", matte),        // the patches on the wall
    oak = ("#8A6642", wood),            // the gate's and fingerpost's timber
    oak_dark = ("#6E4F33", wood),       // its shaded rails and posts
    iron = ("#2B302E", metal),          // hinges, latch and the fingerpost
    paint = ("#EFEBDD", satin),         // the fingerpost's arms
}

/// The five things that divide and mark the fields.
pub fn family() -> Family {
    Family::new(
        "fields",
        M.palette(),
        vec![
            Model::new("dryStoneWall", walls::dry_stone_wall, (1.98, 1.14, 0.57))
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
            Model::new("stile", walls::stile, (0.94, 0.96, 0.9))
                .tolerance(0.03)
                .paint(&[("stone", "Stone"), ("oak", "Wood")]),
            Model::new("fingerpost", gates::fingerpost, (0.99, 2.70, 0.62))
                .centre(0.0, 0.17)
                .tolerance(0.03)
                .paint(&[("paint", "Paint")]),
        ],
        "Fields and lanes between villages (crates/art/outside/src/fields, built by cargo art fields): dry-stone walls, hedgerows, gates, stiles and fingerposts.",
    )
}
