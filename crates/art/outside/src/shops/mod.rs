//! The town's shop fittings and café furniture (CM4): the shop counter with its till, the bakery case, the
//! grocery shelves, the café table and the bentwood café chair. Notes: NOTES.md.
//!
//! They stand on town lots, seen from far and by the dozen, so each has a far version (a quarter of the triangles,
//! E7). The parts are placed in game axes (`GameFrame`), where the game places them: the counter, the case and the
//! shelves sit back on their 2 x 1 tiles as theirs do (`settle!(centre = false)` and `centre` on the Model),
//! the café chair's seat 2 cm forward. Two clips: the till's cash drawer slides out to the back, to the shopkeeper,
//! for `treat` (`till`), and the bakery case's back glass door, on the baker's side, slides aside for `pastry`
//! and `takeCake` (`door`), each as the game's vendor reaches for it.

use artkit::family::{Family, Model, motions};

mod bakery;
mod cafe;
pub(crate) mod common;
mod counter;
mod shelves;

use common::M;

const TILL_DELAY: f64 = 3.9; // as the shopkeeper takes the money (the game's `sell` clip, clips/serve.ts: paid at 4.2 s)
const TILL_HOLD: f64 = 1.1; // and shuts again this long after it opened
const DOOR_DELAY: f64 = 1.0; // as the baker reaches in (`sell`: 1.5-2.1 s)
const DOOR_HOLD: f64 = 1.4; // and shuts again this long after it opened

const FAR: f64 = 0.25; // placed on town lots, seen from afar: a quarter of the triangles (E7)

/// The family.
pub fn family() -> Option<Family> {
    let door = format!(r#"{{"clip": "door", "delay": {DOOR_DELAY:?}, "hold": {DOOR_HOLD:?}}}"#);
    Some(Family::new(
        "shops",
        M.palette(),
        vec![
            Model::new("shopCounter", counter::shop_counter, (1.96, 1.276, 0.78))
                .key("model.shopCounter")
                .centre(0.0, -0.04)
                // buying a little something: the shopkeeper's till drawer pops as they take the money
                .motions(motions(&format!(
                    r#"{{"uses": {{"treat": {{"clip": "till", "delay": {TILL_DELAY:?}, "hold": {TILL_HOLD:?}}}}}}}"#
                )))
                .paint(&[("counter", "Counter"), ("walnut", "Trim")])
                .far_ratio(FAR),
            Model::new("bakeryCase", bakery::bakery_case, (1.94, 1.29, 0.86))
                .key("model.bakeryCase")
                .centre(0.0, 0.0)
                // buying a pastry or a slice of cake: the baker slides the back door aside to reach in
                .motions(motions(&format!(r#"{{"uses": {{"pastry": {door}, "takeCake": {door}}}}}"#)))
                .paint(&[("case", "Counter"), ("rim", "Trim")])
                .far_ratio(FAR),
            Model::new("groceryShelf", shelves::grocery_shelf, (1.92, 1.72, 0.48))
                .key("model.groceryShelf")
                .centre(0.0, -0.19)
                .paint(&[("shelf", "Shelves"), ("trim", "Trim")])
                .far_ratio(FAR),
            Model::new("cafeTable", cafe::cafe_table, (0.62, 0.745, 0.62))
                .key("model.cafeTable")
                .paint(&[("marble", "Top"), ("iron", "Frame")])
                .far_low(),
            Model::new("cafeChair", cafe::cafe_chair, (0.446, 0.952, 0.472))
                .key("model.cafeChair")
                .centre(0.0, -0.008)
                .paint(&[("bentwood", "Frame"), ("cane", "Seat")])
                .far_ratio(FAR),
        ],
        "The shop fittings and café furniture for the town (crates/art/outside/src/shops, built by cargo art). \
         Listed in the base manifest's packs. Each has a far version for the town overview. The till's cash drawer \
         slides out (`till`) while a resident buys a treat; the bakery case's back glass door slides aside (`door`) \
         for a pastry.",
    ))
}

#[cfg(test)]
mod tests {
    use super::cafe::{SEAT, SEAT_FORWARD, TABLE_TOP};
    use artkit::json::{Json, loads};

    /// The heights the content names are the ones the models are built to.
    #[test]
    fn content_facts() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../web/public/content/community.json"
        );
        let text = std::fs::read_to_string(path).expect("community.json reads");
        let content = loads(&text).expect("community.json is JSON");
        let Some(Json::Arr(objects)) = content.get("objects") else {
            panic!("community.json: no objects")
        };
        let find = |id: &str| {
            objects
                .iter()
                .find(|o| o.get("id").and_then(Json::str) == Some(id))
                .unwrap_or_else(|| panic!("{id} in community.json"))
        };
        let surface = find("cafeTable")
            .get("surface")
            .and_then(|s| s.get("height"));
        assert_eq!(
            surface.and_then(Json::num),
            Some(TABLE_TOP),
            "cafeTable: surface height moved"
        );
        let seat = find("cafeChair").get("seat").expect("cafeChair seat");
        assert_eq!(
            seat.get("height").and_then(Json::num),
            Some(SEAT),
            "cafeChair: seat moved"
        );
        assert_eq!(
            seat.get("forward").and_then(Json::num),
            Some(SEAT_FORWARD),
            "cafeChair: seat moved"
        );
        let Json::Obj(fields) = seat else {
            panic!("cafeChair: seat")
        };
        assert_eq!(fields.len(), 2, "cafeChair: seat moved");
    }
}
