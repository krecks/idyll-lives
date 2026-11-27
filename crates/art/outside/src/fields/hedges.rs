//! The hedgerow: a clipped hedge along a lane, grown thick enough to hold the field and to give the birds a perch.

use artkit::botany::{hedge_top, palette};
use artkit::materials::Palette;
use artkit::parts::{join_all, settle};
use artkit::scene::Obj;
use artkit::rand::Random;
use artkit::{florets, hedge_block, perches, twig};

/// The hedgerow's colours: the plants' botany palette (its finishes too: one draw on `foliage`, as the street
/// hedge), in a darker, field-side green.
pub fn colours() -> Palette {
    palette(&[
        ("leaf", "#4E6B35"),
        ("leaf_light", "#6F8A43"),
        ("leaf_dark", "#354B27"),
        ("stem", "#5C4936"),
        ("twig", "#6B5540"),
        ("blossom", "#A8322A"),
    ])
}

/// A hawthorn-and-hazel hedgerow: the garden hedge's clipped block (`hedge_block!`), in soft mounds, on bare
/// stems, with the tufts that have grown since the last trim.
pub fn hedgerow() -> Vec<Obj> {
    let p = colours();
    // an uneven, untrimmed top (`rough`), under the saplings and berried sprays that stand out of it to 1.45 m
    hedge_block!(&p, 1.95, 1.2, 0.95, seed = 211, rough = 1.0);
    let mut rng = Random::new(212);
    for i in 0..5 {
        let x = (i as f64 - 2.0) * 0.38 + rng.uniform(-0.1, 0.1);
        let y = rng.uniform(-0.2, 0.2);
        let (lx, ly) = (rng.uniform(-0.12, 0.12), rng.uniform(-0.12, 0.12));
        let top = 1.25 + rng.uniform(0.0, 0.1);
        twig!(
            format!("shoot{i}"),
            &[(x, y, 0.8), (x + lx * 0.5, y + ly * 0.5, 1.15), (x + lx, y + ly, top)],
            0.014,
            0.006,
            p.get("twig"),
            verts = 4
        );
        florets!(
            format!("spray{i}"),
            0.13,
            (x + lx, y + ly, top),
            p.get("leaf_light"),
            bumps = 3.0,
            amount = 0.15,
            flat = 0.55,
            segments = 6,
            rings = 4,
            seed = i as f64
        );
    }
    // haws among the leaves
    for i in 0..9 {
        let x = rng.uniform(-0.42, 0.42) * 1.95;
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        florets!(
            format!("haw{i}"),
            0.022,
            (x, side * 0.4, rng.uniform(0.5, 1.0)),
            p.get("blossom"),
            bumps = 2.0,
            amount = 0.05,
            flat = 1.0,
            segments = 5,
            rings = 4,
            seed = i as f64
        );
    }
    let still = join_all("hedgerow");
    perches!(8, 211, &hedge_top(1.45), per = 8, up = (0.6, 1.4));
    settle(&[still])
}
