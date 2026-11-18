//! The town's food stalls and vending machines (CM3): the ice cream cart (`model.iceCreamCart`), the hot dog
//! stand (`model.hotDogStand`), the snack machine (`model.vendingSnack`), the drinks machine (`model.vendingDrink`)
//! and the market stall (`model.marketStall`). Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor, not centred, and `centre`
//! on its `Model`): the carts and the stall across their 2 x 1 tiles, the machines against the back of their tile.
//! The moving parts: the cart's freezer lid slides across (`lid_open`), the stand's hot-plate lid lifts
//! (`lid_open`), each a second into the sale as the vendor behind reaches in, for 1.4 s; a snack slides down to the
//! machine's tray and a can drops to it (`drop`). The machines' lit signs (and the drinks machine's front) glow at
//! night. They are placed on town lots and seen from far, so each has a far version (a quarter of the triangles).

use artkit::family::{Family, Model, motions};

pub(crate) mod common;
mod drink;
mod hot_dog;
mod ice_cream;
mod machines;
mod market;

use common::M;

const FAR: f64 = 0.25;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "stalls",
        M.palette(),
        vec![
            Model::new(
                "iceCreamCart",
                ice_cream::ice_cream_cart,
                (1.729, 2.355, 0.966),
            )
            .key("model.iceCreamCart")
            .centre(-0.0235, 0.0023)
            // the seller slides the freezer's lid open to scoop (the game's `sell` clip: 1.5-2.1 s in)
            .motions(motions(
                r#"{"uses": {"iceCream": {"clip": "lid_open", "delay": 1.0, "hold": 1.4}}}"#,
            ))
            .far_ratio(FAR)
            .paint(&[
                ("cart", "Body"),
                ("parasol", "Parasol"),
                ("stripe", "Bands"),
            ]),
            Model::new("hotDogStand", hot_dog::hot_dog_stand, (1.577, 2.387, 0.981))
                .key("model.hotDogStand")
                .centre(-0.0915, 0.0033)
                // the vendor lifts the hot plate's lid for a sausage
                .motions(motions(
                    r#"{"uses": {"hotDog": {"clip": "lid_open", "delay": 1.0, "hold": 1.4}}}"#,
                ))
                .far_ratio(FAR)
                .paint(&[("stand", "Body"), ("awning", "Awning")]),
            Model::new("vendingSnack", machines::vending_snack, (0.9, 1.88, 0.748))
                .key("model.vendingSnack")
                .centre(0.0, -0.056)
                .motions(motions(r#"{"uses": {"snack": "drop"}}"#))
                .far_ratio(FAR)
                .paint(&[("snack_red", "Body")]),
            Model::new("vendingDrink", drink::vending_drink, (0.9, 1.88, 0.785))
                .key("model.vendingDrink")
                .centre(0.0, -0.0375)
                .motions(motions(r#"{"uses": {"drink": "drop"}}"#))
                .far_ratio(FAR)
                .paint(&[("drink_blue", "Body")]),
            Model::new("marketStall", market::market_stall, (1.841, 2.148, 0.983))
                .key("model.marketStall")
                .centre(0.0005, 0.02)
                .far_ratio(FAR)
                .paint(&[("timber", "Body"), ("awning_green", "Awning")]),
        ],
        "The food stalls and vending machines (crates/art/outside/src/stalls, built by cargo art). Listed in the \
         base manifest's packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group. The ice cream cart slides its freezer lid open and the hot dog stand lifts its hot-plate lid \
         (`lid_open`); a snack and a can drop to the machines' trays (`drop`). Each has a far version for the town \
         overview.",
    ).switched())
}
