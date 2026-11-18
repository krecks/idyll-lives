//! The plants: the G1 houseplants, the G2 flowers, the G3 shrubs and the G4 edibles. Notes: NOTES.md.
//!
//! Each model takes the family's colours with some of its own (`botany::palette`; the finishes stay the
//! family's) and is built from artkit's botany: leaves cut from outlines, stems, pots, flower faces, the shrubs'
//! masses and their perches, the beds and the crops' growth clips. The outdoor ones have a far version, the same
//! build at low detail (`far_low`), every random number drawn.
//!
//! Run:  cargo art plants [--only hosta] [--preview]

use artkit::botany::BASE;
use artkit::family::{Family, Model};
use artkit::json;

mod boxes;
pub mod common;
mod edibles;
mod flowers;
mod grasses;
mod houseplants;
mod shrubs;

/// (width, height, depth) as measured; each within 10 % of the old part's bounds (`cargo check-glb` warns
/// beyond).
const TOLERANCE: f64 = 0.03;

/// The family.
pub fn family() -> Option<Family> {
    let pot = |model: Model| model.tolerance(TOLERANCE).paint(&[("body", "Pot")]);
    let bed = |model: Model, more: &[(&'static str, &'static str)]| {
        let paint = [&[("bed", "Bed")][..], more].concat();
        model.tolerance(TOLERANCE).far_low().paint(&paint)
    };
    let shrub = |model: Model| model.tolerance(TOLERANCE).far_low();
    Some(Family::new(
        "plants",
        BASE.palette(),
        vec![
            pot(Model::new("plant", houseplants::plant, (0.772, 0.825, 0.791)).key("model.plant")),
            pot(
                Model::new("monstera", houseplants::monstera, (0.784, 0.911, 0.791))
                    .key("model.garden.monstera"),
            ),
            pot(
                Model::new("fiddleFig", houseplants::fiddle_fig, (0.528, 1.626, 0.527))
                    .key("model.garden.fiddleFig"),
            ),
            pot(
                Model::new("snakePlant", houseplants::snake_plant, (0.341, 0.905, 0.32))
                    .key("model.garden.snakePlant"),
            ),
            pot(
                Model::new("cacti", houseplants::cacti, (0.568, 0.521, 0.465))
                    .key("model.garden.cacti"),
            ),
            pot(
                Model::new("succulents", houseplants::succulents, (0.62, 0.17, 0.62))
                    .key("model.garden.succulents"),
            ),
            pot(Model::new(
                "bostonFern",
                houseplants::boston_fern,
                (0.734, 0.979, 0.792),
            )
            .key("model.garden.bostonFern")),
            pot(Model::new("palm", houseplants::palm, (0.770, 0.893, 0.693))
                .key("model.garden.palm")),
            pot(
                Model::new("orchid", houseplants::orchid, (0.431, 1.198, 0.381))
                    .key("model.garden.orchid"),
            ),
            pot(
                Model::new("bonsai", houseplants::bonsai, (0.62, 1.354, 0.407))
                    .key("model.garden.bonsai"),
            )
            .motions(json!({"uses": {"prune": "trim"}})),
            // G2: flowers (with G3 and G4, each with a far version: they stand outside, seen in the town overview)
            Model::new("flowerbed", boxes::flowerbed, (0.94, 0.475, 0.94))
                .key("model.flowerbed")
                .tolerance(TOLERANCE)
                .far_low()
                .paint(&[("wood", "Box")]),
            bed(
                Model::new("tulips", flowers::tulips, (0.84, 0.622, 0.843))
                    .key("model.garden.tulips"),
                &[],
            ),
            bed(
                Model::new("daffodils", flowers::daffodils, (0.84, 0.491, 0.84))
                    .key("model.garden.daffodils"),
                &[("bloom", "Petals")],
            ),
            bed(
                Model::new("daisies", flowers::daisies, (0.84, 0.376, 0.84))
                    .key("model.garden.daisies"),
                &[("bloom", "Petals")],
            ),
            bed(
                Model::new("sunflowers", flowers::sunflowers, (0.84, 2.108, 0.907))
                    .key("model.garden.sunflowers"),
                &[("bloom", "Petals")],
            ),
            bed(
                Model::new("poppies", flowers::poppies, (0.84, 0.657, 0.84))
                    .key("model.garden.poppies"),
                &[("bloom", "Petals")],
            ),
            bed(
                Model::new("marigolds", flowers::marigolds, (0.885, 0.34, 0.84))
                    .key("model.garden.marigolds"),
                &[],
            ),
            bed(
                Model::new("pansies", flowers::pansies, (0.84, 0.264, 0.84))
                    .key("model.garden.pansies"),
                &[],
            ),
            bed(
                Model::new("lilies", flowers::lilies, (0.84, 0.939, 0.869))
                    .key("model.garden.lilies"),
                &[],
            ),
            bed(
                Model::new("lupins", flowers::lupins, (0.84, 0.987, 0.84))
                    .key("model.garden.lupins"),
                &[],
            ),
            bed(
                Model::new("forgetMeNots", flowers::forget_me_nots, (0.866, 0.28, 0.84))
                    .key("model.garden.forgetMeNots"),
                &[],
            ),
            bed(
                Model::new("cosmos", flowers::cosmos, (0.885, 0.879, 0.84))
                    .key("model.garden.cosmos"),
                &[],
            ),
            Model::new("meadow", boxes::meadow, (0.872, 0.575, 0.91))
                .key("model.garden.meadow")
                .tolerance(TOLERANCE)
                .far_low()
                .motions(json!({"uses": {"pickFlower": "pick"}})),
            Model::new("geraniums", boxes::geraniums, (0.424, 0.65, 0.468))
                .key("model.garden.geraniums")
                .tolerance(TOLERANCE)
                .far_low()
                .paint(&[("body", "Pot"), ("bloom", "Flowers")]),
            Model::new("flowerBox", boxes::flower_box, (0.945, 0.732, 0.505))
                .key("model.garden.flowerBox")
                .tolerance(TOLERANCE)
                .far_low()
                .paint(&[("body", "Box")]),
            // G3: shrubs (they dress every lot)
            shrub(Model::new("bush", shrubs::bush, (1.043, 0.91, 1.095)).key("model.bush"))
                .habitat("bush"),
            shrub(
                Model::new("bushSmall", shrubs::bush_small, (0.832, 0.707, 0.782))
                    .key("model.bush.small"),
            )
            .habitat("bush"),
            shrub(
                Model::new(
                    "bushFlowering",
                    shrubs::bush_flowering,
                    (1.208, 1.016, 1.268),
                )
                .key("model.bush.flowering"),
            )
            .habitat("bush"),
            shrub(
                Model::new("streetHedge", shrubs::street_hedge, (1.547, 1.019, 0.85))
                    .key("model.hedge"),
            )
            .habitat("hedge"),
            shrub(
                Model::new("boxwood", shrubs::boxwood, (0.696, 0.752, 0.722))
                    .key("model.garden.boxwood"),
            )
            .habitat("bush")
            .motions(json!({"uses": {"trim": "trim"}})),
            shrub(
                Model::new("topiaryCone", shrubs::topiary_cone, (0.643, 1.805, 0.66))
                    .key("model.garden.topiaryCone"),
            )
            .habitat("bush")
            .paint(&[("body", "Pot")])
            .motions(json!({"uses": {"trim": "trim"}})),
            shrub(
                Model::new("hedge", shrubs::garden_hedge, (1.546, 1.167, 0.92))
                    .key("model.garden.hedge"),
            )
            .habitat("hedge")
            .motions(json!({"uses": {"trim": "trim"}})),
            shrub(
                Model::new("hydrangea", shrubs::hydrangea, (1.196, 0.957, 1.182))
                    .key("model.garden.hydrangea"),
            )
            .habitat("bush")
            .paint(&[("blossom", "Flowers")]),
            shrub(
                Model::new("rosebush", shrubs::rosebush, (0.771, 0.903, 0.722))
                    .key("model.garden.rosebush"),
            )
            .habitat("bush")
            .paint(&[("bloom", "Roses")])
            .motions(json!({"uses": {"prune": "cut"}})),
            shrub(
                Model::new("azalea", shrubs::azalea, (1.158, 1.013, 1.152))
                    .key("model.garden.azalea"),
            )
            .habitat("bush")
            .paint(&[("blossom", "Flowers")])
            .motions(json!({"uses": {"trim": "trim"}})),
            shrub(
                Model::new("lavender", grasses::lavender, (0.607, 0.865, 0.63))
                    .key("model.garden.lavender"),
            )
            .paint(&[("bloom", "Flowers")])
            .motions(json!({"uses": {"cut": "cut"}})),
            shrub(
                Model::new("fern", grasses::garden_fern, (0.793, 0.478, 0.932))
                    .key("model.garden.fern"),
            ),
            shrub(
                Model::new("grass", grasses::ornamental_grass, (1.218, 1.21, 1.28))
                    .key("model.garden.grass"),
            ),
            shrub(
                Model::new("hosta", grasses::hosta, (0.621, 0.842, 0.675))
                    .key("model.garden.hosta"),
            ),
            // G4: edibles
            shrub(
                Model::new("vegPatch", edibles::veg_patch, (0.92, 0.398, 0.92))
                    .key("model.garden.vegPatch"),
            )
            .paint(&[("body", "Bed")])
            .motions(json!({"growth": "grow"})),
            shrub(
                Model::new("tomatoes", edibles::tomatoes, (0.8, 1.19, 0.8))
                    .key("model.garden.tomatoes"),
            )
            .paint(&[("bed", "Bed")])
            .motions(json!({"growth": "grow"})),
            shrub(
                Model::new("strawberries", edibles::strawberries, (0.88, 0.21, 0.88))
                    .key("model.garden.strawberries"),
            )
            .paint(&[("bed", "Bed")])
            .motions(json!({"growth": "grow"})),
            shrub(
                Model::new("herbs", edibles::herbs, (0.84, 1.01, 0.46)).key("model.garden.herbs"),
            )
            .paint(&[("body", "Bed")])
            .motions(json!({"uses": {"pickHerbs": "harvest"}})),
            shrub(
                Model::new("pumpkins", edibles::pumpkins, (1.112, 0.269, 0.947))
                    .key("model.garden.pumpkins"),
            )
            .paint(&[("bed", "Bed")])
            .motions(json!({"growth": "grow"})),
        ],
        "The plants (crates/art/nature/src/plants, built by cargo art): the G1 houseplants, G2 flowers, \
         G3 shrubs and G4 edibles. Listed in the base manifest's packs. Leaves and stems are on the \
         `foliage` finish, so the nature shader sways and tints them. The outdoor ones have a far \
         version (`far`). Clips: the bonsai's sprig and the clipped shrubs' sprigs shrink away while \
         they are trimmed, a rose and lavender stems while they are cut, a meadow flower and basil \
         while they are picked; the crops grow from seedlings to ripe and lose their crop when \
         harvested (`motions.growth`).",
    ))
}
