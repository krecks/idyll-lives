//! The farm (CM5): the red barn (`model.farm.barn`), the grain silo (`model.farm.silo`), the little tractor
//! (`model.farm.tractor`), the round hay bale (`model.farm.hayBale`), the stack of hay bales (`model.farm.hayStack`),
//! the scarecrow (`model.farm.scarecrow`), the water trough (`model.farm.trough`) and the chicken coop
//! (`model.farm.coop`). They stand on the farm lots of farm worlds (web/src/game/town.ts `farmstead`) and are seen
//! from afar in the town overview, so each has a far version (E7), built by hand (the model's plain shapes) but the
//! scarecrow's, decimated. Notes: NOTES.md.
//!
//! Built in the game's axes (`F`: x, height, depth; front +depth), then stood on the floor and centred on the
//! footprint (`settle`), the sizes the old parts'.

use artkit::family::{Family, Model, motions};
use artkit::far::far_of;

mod barn;
mod barn_doors;
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

/// The family.
pub fn family() -> Family {
    Family::new(
        "farm",
        M.palette(),
        vec![
            Model::new("barn", barn::barn, (6.04, 6.0, 5.94))
                .tolerance(TOLERANCE)
                .far(|| far_of(barn::barn_far))
                .paint(&[
                    ("barn", "Paint"),
                    ("barn_roof", "Roof"),
                    ("barn_trim", "Trim"),
                ]),
            Model::new("silo", silo::silo, (2.77, 6.9, 2.72))
                .tolerance(TOLERANCE)
                .far(|| far_of(silo::silo_far))
                .paint(&[("silo", "Paint"), ("silo_cap", "Roof")]),
            Model::new("tractor", tractor::tractor, (1.62, 2.5, 3.28))
                .tolerance(TOLERANCE)
                .far(|| far_of(tractor::tractor_far))
                .paint(&[("tractor", "Paint"), ("rims", "Wheels")]),
            Model::new("hayBale", hay::hayBale, (1.224, 1.21, 1.21))
                .tolerance(TOLERANCE)
                .far(|| far_of(hay::hayBale_far))
                .paint(&[("twine", "Twine")]),
            Model::new("hayStack", hay::hayStack, (1.9, 0.915, 0.9))
                .tolerance(TOLERANCE)
                .far(|| far_of(hay::hayStack_far))
                .paint(&[("twine", "Twine")]),
            Model::new("scarecrow", scarecrow::scarecrow, (0.985, 2.13, 0.54))
                .tolerance(TOLERANCE)
                .far_ratio(0.25)
                .motions(motions(r#"{"idle": "sway"}"#))
                .paint(&[("shirt", "Shirt"), ("trousers", "Trousers"), ("hat", "Hat")]),
            Model::new("trough", trough::trough, (1.84, 0.655, 0.74))
                .tolerance(TOLERANCE)
                .far(|| far_of(trough::trough_far))
                .paint(&[("trough", "Paint")]),
            Model::new("coop", coop::coop, (1.74, 1.88, 1.97))
                .tolerance(TOLERANCE)
                .far(|| far_of(coop::coop_far))
                .motions(motions(r#"{"night": "close"}"#))
                .paint(&[
                    ("coop", "Paint"),
                    ("coop_roof", "Roof"),
                    ("coop_trim", "Trim"),
                ]),
        ],
        "The farm (crates/art/outside/src/farm, built by cargo art): the red barn, the grain silo, the little tractor, \
         the round hay bale, the stack of hay bales, the scarecrow (an ambient sway), the water trough and the chicken \
         coop (its pop-hole hatch shut at night, `close`). Each has a far version (`far`, E7) for the town overview. \
         Listed in the base manifest's packs.",
    )
}

#[cfg(test)]
mod tests {
    use artkit::family::{Far, make, make_far};
    use artkit::json::Json;

    /// The hay stack's seat is the one the content names: its two slots spread across the width (x = ±0.5), in
    /// front of the top bale.
    #[test]
    fn content_facts() {
        let o = crate::content_object("community.json", "farm.hayStack");
        let seat = o.get("seat").expect("a seat");
        let (height, forward) = super::hay::STACK_SEAT;
        assert_eq!(
            seat.get("height").and_then(Json::num),
            Some(height),
            "hayStack: seat moved"
        );
        assert_eq!(
            seat.get("forward").and_then(Json::num),
            Some(forward),
            "hayStack: seat moved"
        );
        assert_eq!(
            o.get("slots").and_then(Json::num),
            Some(2.0),
            "hayStack: slots moved"
        );
        assert!(
            o.get("places").is_none(),
            "hayStack: places now name the seats"
        );
    }

    /// A far version built by hand stands where its model stands. Each is settled (centred on its own box) as
    /// one part whose origin is its build's first part's, the same in both, so their places after settle agree
    /// only if their boxes' middles did: a far version leaving out an outermost part (the silo's hopper) is off.
    #[test]
    fn far_versions_stand_where_the_models_do() {
        for model in super::family().models {
            let Some(far @ Far::Own(_)) = model.far else {
                continue;
            };
            let at = make(&model).expect("builds")[0].location();
            let far_at = make_far(&model, far).expect("far builds")[0].location();
            assert!(
                (at.x - far_at.x).abs() < 0.005 && (at.y - far_at.y).abs() < 0.005,
                "{}: far at {far_at:?}, model at {at:?}",
                model.id
            );
        }
    }
}
