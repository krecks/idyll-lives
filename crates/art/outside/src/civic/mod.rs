//! The named places' fittings (content/civic.json): the reception, hotel and duty desks, the bank's teller
//! counter, the teacher's desk, the church's lectern, pews and altar, the pub's bar, the pupil's desk, the waiting
//! room's bench, the consulting room's couch, the shops' clothes rail, plant table and mannequin, the cash machine,
//! the sports hall's wall bars, the changing room's lockers, the hotel's luggage trolley and the bank's vault door.
//! Notes: NOTES.md.
//!
//! The parts are placed in game axes (`F`), where the game places them. Desks and counters with `staff` face the
//! customer at the front (+z); whoever serves stands behind them (content `staff.at`, 0.85 m back), so their
//! monitors, drawers and tills face the back. Seats keep the content's heights: the pew's and the waiting bench's
//! 0.45 m, slots spread across their three tiles; the pupil's desk and the couch the default seat's (0.45 m, at
//! the footprint's middle). Things against a wall (the lockers, the cash machine, the wall bars, the vault door)
//! have their backs on the wall face (-0.43). Nothing moves: none of their uses opens anything.

use artkit::family::{Family, Model};

pub(crate) mod common;
mod counters;
mod desks;
mod rest;
mod seats;
mod shop;

use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "civic",
        M.palette(),
        vec![
            Model::new("receptionDesk", desks::reception_desk, (1.92, 1.374, 0.85))
                .key("model.receptionDesk")
                .paint(&[("desk", "Desk"), ("oak", "Ledge")]),
            Model::new(
                "hotelReception",
                desks::hotel_reception,
                (1.94, 1.481, 0.858),
            )
            .key("model.hotelReception")
            .paint(&[("walnut", "Desk"), ("marble", "Top")]),
            Model::new("dutyDesk", desks::duty_desk, (1.92, 1.93, 0.85))
                .key("model.dutyDesk")
                .paint(&[("counter", "Counter"), ("laminate", "Top")]),
            Model::new("tellerCounter", desks::teller_counter, (1.92, 1.995, 0.85))
                .key("model.tellerCounter")
                .paint(&[("teller", "Counter"), ("laminate", "Worktop")]),
            Model::new("teacherDesk", counters::teacher_desk, (1.5, 1.124, 0.71))
                .key("model.teacherDesk")
                .paint(&[("oak", "Desk")]),
            Model::new("lectern", counters::lectern, (0.66, 1.25, 0.56))
                .key("model.lectern")
                .paint(&[("pew_oak", "Wood")]),
            Model::new("barCounter", counters::bar_counter, (1.96, 1.47, 0.82))
                .key("model.barCounter")
                .paint(&[("mahogany", "Bar"), ("brass", "Rail")]),
            Model::new("schoolDesk", seats::school_desk, (0.62, 0.828, 0.778))
                .key("model.schoolDesk")
                .paint(&[("pine", "Wood"), ("iron", "Frame")]),
            Model::new("pew", seats::pew, (2.976, 1.04, 0.67))
                .key("model.pew")
                .paint(&[("pew_oak", "Wood")]),
            Model::new("waitingBench", seats::waiting_bench, (2.7, 0.85, 0.53))
                .key("model.waitingBench")
                .paint(&[("shell", "Seats"), ("steel", "Frame")]),
            Model::new("examCouch", seats::exam_couch, (0.62, 0.809, 1.869))
                .key("model.examCouch")
                .paint(&[("vinyl", "Pad"), ("white", "Frame")]),
            Model::new("clothesRack", shop::clothes_rack, (1.76, 1.514, 0.52))
                .key("model.clothesRack")
                .paint(&[("bell", "Rail")]),
            Model::new("plantTable", shop::plant_table, (1.88, 1.206, 0.801))
                .key("model.plantTable")
                .far_ratio(0.25)
                .paint(&[("slat", "Slats"), ("terracotta", "Pots")]),
            Model::new("atm", shop::atm, (0.7, 1.68, 0.641))
                .key("model.atm")
                .centre(0.0, -0.11)
                .paint(&[("atm", "Cabinet"), ("fascia", "Fascia")]),
            Model::new("wallBars", rest::wall_bars, (0.95, 2.5, 0.25))
                .key("model.wallBars")
                .centre(0.0, -0.305)
                .on_wall()
                .paint(&[("oak", "Wood")]),
            Model::new("altar", rest::altar, (1.92, 1.692, 0.85))
                .key("model.altar")
                .paint(&[("stone", "Stone"), ("frontal", "Frontal")]),
            Model::new("lockerRow", rest::locker_row, (1.92, 1.845, 0.454))
                .key("model.lockerRow")
                .centre(0.0, -0.203)
                .paint(&[("locker", "Doors"), ("frame", "Frame")]),
            Model::new("mannequin", rest::mannequin, (0.547, 1.755, 0.4))
                .key("model.mannequin")
                .paint(&[
                    ("cloth1", "Jacket"),
                    ("cloth2", "Trousers"),
                    ("dummy", "Dummy"),
                ]),
            Model::new("luggageTrolley", rest::luggage_trolley, (0.9, 1.874, 0.54))
                .key("model.luggageTrolley")
                .paint(&[("brass", "Frame"), ("carpet", "Carpet")]),
            Model::new("vaultDoor", rest::vault_door, (1.92, 1.92, 0.316))
                .key("model.vaultDoor")
                .centre(0.0, -0.272)
                .on_wall()
                .paint(&[("vault", "Door"), ("vault_dark", "Frame")]),
        ],
        "The fittings for the town's named places (crates/art/outside/src/civic, built by cargo art; \
         content/civic.json): front desks and counters, the church's pews, lectern and altar, the pub's bar, \
         school desks, the waiting room's bench, the couch, shop rails and the rest. Listed in the base \
         manifest's packs. Nothing moves.",
    ))
}

#[cfg(test)]
mod tests {
    use super::common::SEAT;
    use artkit::json::{Json, loads};

    fn num(j: Option<&Json>) -> Option<f64> {
        match j {
            Some(Json::Num(x)) => Some(*x),
            Some(Json::Int(i)) => Some(*i as f64),
            _ => None,
        }
    }

    /// The seats the content names are the ones the pew and the waiting bench are built to.
    #[test]
    fn content_facts() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../web/public/content/civic.json"
        );
        let text = std::fs::read_to_string(path).expect("civic.json reads");
        let content = loads(&text).expect("civic.json is JSON");
        let Some(Json::Arr(objects)) = content.get("objects") else {
            panic!("civic.json: no objects")
        };
        for id in ["pew", "waitingBench"] {
            let o = objects
                .iter()
                .find(|o| o.get("id").and_then(Json::str) == Some(id))
                .unwrap_or_else(|| panic!("{id} in civic.json"));
            let seat = o.get("seat").expect("a seat");
            assert_eq!(num(seat.get("height")), Some(SEAT), "{id}: seat moved");
            assert_eq!(num(seat.get("forward")), Some(0.0), "{id}: seat moved");
            let Json::Obj(fields) = seat else {
                panic!("{id}: seat")
            };
            assert_eq!(fields.len(), 2, "{id}: seat moved");
            assert_eq!(num(o.get("slots")), Some(3.0), "{id}: slots moved");
        }
    }
}
