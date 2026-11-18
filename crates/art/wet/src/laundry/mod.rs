//! The laundry: the washing machine (`model.laundryWasher`), the tumble dryer (`model.laundryDryer`), the towel
//! rail (`model.towelRail`), the towel ladder (`model.towelLadder`), the laundry basket (`model.laundryBasket`)
//! and the bath mat (`model.bathMat`); and what residents carry doing the laundry (content `items`): the basket of
//! washing (`item.washingBasket`), an armful of it dry (`item.washingArmful`) and a towel being folded
//! (`item.washingPiece`). Notes: NOTES.md.
//!
//! The washer and the dryer stand against the wall where the game places them, built in place (game axes). Each
//! has two moving parts: the round door, hinged on its left, which swings open while a resident loads or empties
//! it (`door_open`, held 2.5 s), and the drum's face behind the glass, which turns while the machine runs on its
//! own after it's loaded (`drum_spin`, the `running` loop: content `runs`). The rail hangs on the wall at its
//! real height; the rest are still.
//!
//! Run:  cargo art laundry [--only laundryWasher] [--preview]

use artkit::family::{Family, Model};
use artkit::json::Json;
use artkit::palette;

mod common;
mod machines;
mod rest;

palette! { M:
    body = ("#F4F4F2", gloss),       // a machine's cabinet; the main part the player colours
    panel = ("#E6E6E3", gloss),      // its control strip
    door = ("#C9CBCC", metal),       // the washer's door ring, handle and knob: brushed metal
    door_dryer = ("#9AA4AB", metal), // the dryer's, and its vents
    glass = ("#BFD3DA", glass),      // the door's window (see-through: its own draw call)
    dark = ("#3A3D41", satin),       // plinths, displays, the door seal
    drum = ("#2B2C2E", satin),       // the drum's dark inside
    paddle = ("#9AA0A6", metal),     // the drum's paddles
    led = ("#5EC28B", gloss),        // the washer's ready light
    led_dryer = ("#E9853B", gloss),  // the dryer's
    // washing: plain, so it costs no draw call of its own on a machine
    wash_a = ("#9DC6CF", satin),
    wash_b = ("#E4B8AE", satin),
    wash_c = ("#F2F0EA", satin),
    wicker_plain = ("#C9B79C", satin), // the basket on the dryer
    // the rail, the ladder, the basket and the mat
    chrome = ("#D0D3D6", chrome),
    towel = ("#9DC6CF", fabric),
    towel2 = ("#F2F0EA", fabric),
    towel3 = ("#9DAF8C", fabric),
    birch = ("#E3CFA8", wood),
    wicker = ("#C9A979", fabric),
    wicker_rim = ("#B8955F", fabric),
    mat = ("#B9CCC4", fabric),
    mat_pile = ("#C9DAD3", fabric),
    towel_hem = ("#7FAEB9", fabric),  // the woven band across a towel's foot
    towel_hem2 = ("#D9D5CA", fabric),
    wicker_band = ("#B09A7C", satin), // the bands round the dryer's basket
}

/// Loading a machine and taking the washing out open its door, which shuts again (`hold`; `door_open` takes 1.0 s
/// each way); once loaded it runs on its own (content `runs`), the drum going round, a turn each 1.2 s on the
/// washer and 1.8 s on the dryer, till it's done (`running`, from rest to rest).
fn machine() -> Json {
    Json::Obj(vec![
        (
            "uses".into(),
            Json::Obj(vec![
                ("laundry".into(), Json::from("door_open")),
                ("unload".into(), Json::from("door_open")),
            ]),
        ),
        ("hold".into(), Json::Num(2.5)),
        ("running".into(), Json::from("drum_spin")),
    ])
}

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "laundry",
        M.palette(),
        vec![
            Model::new("laundryWasher", machines::laundry_washer, (0.6, 0.98, 0.65))
                .key("model.laundryWasher")
                .centre(0.0, -0.105)
                .motions(machine())
                .paint(&[("body", "Body"), ("door", "Door"), ("panel", "Panel")]),
            Model::new("laundryDryer", machines::laundry_dryer, (0.6, 1.08, 0.65))
                .key("model.laundryDryer")
                .centre(0.0, -0.105)
                .motions(machine())
                .paint(&[("body", "Body"), ("door_dryer", "Door"), ("panel", "Panel")]),
            Model::new("towelRail", rest::towel_rail, (0.73, 0.595, 0.098))
                .key("model.towelRail")
                .centre(0.0, -0.381)
                .on_wall()
                .paint(&[
                    ("chrome", "Rail"),
                    ("towel", "Towels"),
                    ("towel2", "Hand towel"),
                ]),
            Model::new("towelLadder", rest::towel_ladder, (0.52, 1.707, 0.359))
                .key("model.towelLadder")
                .centre(0.0, -0.25)
                .paint(&[
                    ("birch", "Ladder"),
                    ("towel2", "Top towel"),
                    ("towel", "Middle towel"),
                    ("towel3", "Bottom towel"),
                ]),
            Model::new("laundryBasket", rest::laundry_basket, (0.54, 0.39, 0.421))
                .key("model.laundryBasket")
                .centre(0.0, -0.2)
                .paint(&[("wicker", "Basket"), ("wicker_rim", "Rim & weave")]),
            Model::new("bathMat", rest::bath_mat, (0.76, 0.025, 0.5))
                .key("model.bathMat")
                .paint(&[("mat", "Mat"), ("mat_pile", "Pile")]),
            Model::new("washingBasket", rest::washing_basket, (0.42, 0.284, 0.3))
                .key("item.washingBasket"),
            Model::new("washingArmful", rest::washing_armful, (0.34, 0.193, 0.24))
                .key("item.washingArmful"),
            Model::new("washingPiece", rest::washing_piece, (0.402, 0.165, 0.25))
                .key("item.washingPiece")
                .centre(0.0, -0.005),
        ],
        "The laundry (crates/art/wet/src/laundry, built by cargo art). Listed in the base manifest's packs. \
         The materials map onto the game's finishes so a model costs one draw call per finish group. The \
         washer and the dryer open their door (`door_open`) while a resident loads or empties them, and \
         turn their drum (`drum_spin`) while they run on their own (`running`). The basket of washing \
         and the piece held up to fold are carried (content `items`).",
    ))
}
