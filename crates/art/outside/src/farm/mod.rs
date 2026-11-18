//! The farm (CM5): the red barn (`model.farm.barn`), the grain silo (`model.farm.silo`), the little tractor
//! (`model.farm.tractor`), the round hay bale (`model.farm.hayBale`), the stack of hay bales (`model.farm.hayStack`),
//! the scarecrow (`model.farm.scarecrow`), the water trough (`model.farm.trough`) and the chicken coop
//! (`model.farm.coop`). They stand on the farm lots of farm worlds (web/src/game/town.ts `farmstead`) and are seen
//! from afar in the town overview, so each has a far version (E7), built by hand (the model's plain shapes) but the
//! scarecrow's, decimated. Notes: NOTES.md.
//!
//! Built in the game's axes (`F`: x, height, depth; front +depth), then stood on the floor and centred on the
//! footprint (`settle`), the sizes the old parts'.

use artkit::family::{Family, Model};
use artkit::json::{Json, loads};

mod barn;
mod common;
mod coop;
mod hay;
mod scarecrow;
mod silo;
mod tractor;
mod trough;

use common::M;

/// (width, height, depth): the old parts' bounds, give or take (each model's tolerance)
const TOLERANCE: f64 = 0.03;

fn motions(text: &str) -> Json {
    loads(text).expect("the motions are JSON")
}

/// The family.
pub fn family() -> Option<Family> {
    Some(
        Family::new(
        "farm",
        M.palette(),
        vec![
            Model::new("barn", barn::barn, (6.04, 6.0, 5.94))
                .tolerance(TOLERANCE)
                .far(barn::barn_far)
                .paint(&[("barn", "Paint"), ("barn_roof", "Roof"), ("barn_trim", "Trim")]),
            Model::new("silo", silo::silo, (2.77, 6.9, 2.72))
                .tolerance(TOLERANCE)
                .far(silo::silo_far)
                .paint(&[("silo", "Paint"), ("silo_cap", "Roof")]),
            Model::new("tractor", tractor::tractor, (1.62, 2.5, 3.28))
                .tolerance(TOLERANCE)
                .far(tractor::tractor_far)
                .paint(&[("tractor", "Paint"), ("rims", "Wheels")]),
            Model::new("hayBale", hay::hayBale, (1.224, 1.21, 1.21))
                .tolerance(TOLERANCE)
                .far(hay::hayBale_far)
                .paint(&[("twine", "Twine")]),
            Model::new("hayStack", hay::hayStack, (1.9, 0.915, 0.9))
                .tolerance(TOLERANCE)
                .far(hay::hayStack_far)
                .paint(&[("twine", "Twine")]),
            Model::new("scarecrow", scarecrow::scarecrow, (0.985, 2.13, 0.54))
                .tolerance(TOLERANCE)
                .far_ratio(0.25)
                .motions(motions(r#"{"idle": "sway"}"#))
                .paint(&[("shirt", "Shirt"), ("trousers", "Trousers"), ("hat", "Hat")]),
            Model::new("trough", trough::trough, (1.84, 0.655, 0.74))
                .tolerance(TOLERANCE)
                .far(trough::trough_far)
                .paint(&[("trough", "Paint")]),
            Model::new("coop", coop::coop, (1.74, 1.88, 1.97))
                .tolerance(TOLERANCE)
                .far(coop::coop_far)
                .motions(motions(r#"{"night": "close"}"#))
                .paint(&[("coop", "Paint"), ("coop_roof", "Roof"), ("coop_trim", "Trim")]),
        ],
        "The farm (crates/art/outside/src/farm, built by cargo art): the red barn, the grain silo, the little tractor, \
         the round hay bale, the stack of hay bales, the scarecrow (an ambient sway), the water trough and the chicken \
         coop (its pop-hole hatch shut at night, `close`). Each has a far version (`far`, E7) for the town overview. \
         Listed in the base manifest's packs.",
        )
        .switched(),
    )
}
