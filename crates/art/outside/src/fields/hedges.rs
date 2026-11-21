//! The hedgerow: a clipped hedge along a lane, grown thick enough to hold the field and to give the birds a perch.

use artkit::botany::{hedge_top, palette};
use artkit::materials::Palette;
use artkit::parts::{join_all, settle};
use artkit::scene::Obj;
use artkit::{hedge_block, perches};

/// The hedgerow's colours: the plants' botany palette (its finishes too: one draw on `foliage`, as the street
/// hedge), in a darker, field-side green.
pub fn colours() -> Palette {
    palette(&[
        ("leaf", "#4E6B35"),
        ("leaf_light", "#6F8A43"),
        ("leaf_dark", "#354B27"),
        ("stem", "#5C4936"),
        ("twig", "#6B5540"),
    ])
}

/// A hawthorn-and-hazel hedgerow: the garden hedge's clipped block (`hedge_block!`), in soft mounds, on bare
/// stems, with the tufts that have grown since the last trim.
pub fn hedgerow() -> Vec<Obj> {
    hedge_block!(&colours(), 1.95, 1.45, 0.95, seed = 211);
    let still = join_all("hedgerow");
    perches!(8, 211, &hedge_top(1.45), per = 8, up = (0.6, 1.4));
    settle(&[still])
}
