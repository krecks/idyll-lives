//! The kitchen's cookers (K2): the electric cooker, the gas range, the oven tower, the range cooker, the hob counter
//! and the microwave counter; and the small appliances (K3). Notes: NOTES.md.
//!
//! A cooker is a box on the floor with a hob on top and an oven below; its door drops on a bottom hinge, and a knob
//! turns (with a ring of flame on the gas ones) while a resident cooks. The parts are placed in game axes (`F`): the
//! old depths are shifted back by their depth centre, so the model is centred on its tile (see GameFrame).
//!
//! Run:  cargo art kitchen_cooking [--preview]

use artkit::family::{Family, Model};
use artkit::json::Json;
use artkit::{json, palette};

mod appliances;
pub(crate) mod common;
mod cookers;
mod ovens;

palette! { M:
    enamel = ("#F4F3EF", gloss),     // a white cooker body; the main part the player colours (`paint`)
    enamel2 = ("#EFE5D3", gloss),    // a cream range cooker
    cab = ("#EDEBE6", satin),        // cabinet fronts (the oven tower, the hob counter)
    worktop = ("#E8E6E1", gloss, roughness(0.3)), // a counter worktop
    dark = ("#2B2C2E", satin),       // oven fronts, plinths and control strips
    hob = ("#1B1E22", gloss),        // a ceramic/induction hob and an oven window (opaque, glossy black)
    glass = ("#1D2228", glass),      // the microwave's see-through window
    plate = ("#E9EAEC", gloss),      // the microwave's turntable plate; plates in a dishwasher
    chrome = ("#D0D3D6", chrome),    // handles and knobs
    grate = ("#22252A", satin),      // burner heads and grates
    flame = ("#3E7CC2", gloss),      // a gas flame ring (gloss, not emissive)
    heat = ("#D2402A", gloss),       // a ceramic ring glowing hot (gloss red, so it reads by day)
    charcoal = ("#33363A", satin),   // the microwave's body
    display = ("#5EC28B", gloss),    // the microwave's display
    tray = ("#8E9196", metal),       // an oven tray and racks
    pot = ("#6E7377", metal),        // a stock pot
    pan = ("#4A4E52", satin),        // burner caps, a pot's knob
    steel = ("#C9CBCC", metal),      // a stainless appliance front
    toaster = ("#C05B4D", gloss),    // a toaster's body
    jug = ("#DCE6EA", gloss),        // a blender's jug (opaque, so it costs no extra draw call)
    bean = ("#5A3A26", gloss),       // a coffee machine's bean hopper
    cup = ("#F2F1EE", gloss),        // a cup
    coffee = ("#9A6B3F", gloss),     // the stream of coffee
    fruit = ("#E0A33E", gloss),      // a piece of fruit in the bowl
    bread = ("#D29A55", satin),      // toast's crust
    crumb = ("#EED9A8", satin),      // toast's face
    banana = ("#E6C65A", satin),     // a banana in the fruit bowl
    towel = ("#CFDCE6", satin),      // the range cooker's tea towel (plain: no draw call of its own)
}

const SECONDS: f64 = 1.4; // a door drops, a drawer slides or a knob turns in this long
// catalog2's 0.79-deep counter numbers drawn 0.66 deep against the wall (the counters' COUNTER66)
const COUNTER66: [(f64, f64); 4] = [(-0.48, -0.43), (-0.46, -0.41), (0.24, 0.19), (0.31, 0.23)];
const HOLD: f64 = 3.0; // seconds a door stays open once a use starts

/// The microwave's plate turns once its door has shut (`door_open` opens in 1.0 s and plays back from its hold,
/// 1.2 s), a hair under its 2 s turn each time round: no pause between turns.
fn turntable() -> Json {
    json!({"clip": "turntable", "delay": 2.2, "every": 1.9})
}

/// `bake` drops the oven door for HOLD s.
fn bake() -> Json {
    json!({"clip": "oven_open", "hold": (HOLD)})
}

/// The family.
pub fn family() -> Family {
    // A floor thing against a wall is centred where the game places it (`centre`): its back on the wall,
    // its top under its content `surface`.
    Family::new(
        "kitchen_cooking",
        M.palette(),
        vec![
            Model::new(
                "stoveElectric",
                cookers::stove_electric,
                (0.60, 0.98, 0.661),
            )
            .key("model.stoveElectric")
            .centre(0.0, -0.0995)
            // cook turns the front-left knob and its ring glows for the whole use; bake drops the oven door
            // for HOLD s. By the cooking step's tag too (`stir` on the hob, `bake` in the oven), for a meal
            // started at the fridge (snapshot.rs: no interaction at a station that isn't the recipe's origin).
            .motions(json!({"uses": {"cook": "burner_on", "stir": "burner_on", "bake": (bake())}}))
            .paint(&[("enamel", "Body"), ("chrome", "Knobs"), ("hob", "Hob")]),
            Model::new("stoveGas", cookers::stove_gas, (0.92, 1.111, 0.694))
                .key("model.stoveGas")
                .centre(0.0, -0.089)
                // cook and gourmet (and any `stir` step) light the front left burner; bake drops the oven door
                .motions(json!({"uses": {"cook": "burner_on", "gourmet": "burner_on", "stir": "burner_on", "bake": (bake())}}))
                .paint(&[("enamel", "Body"), ("chrome", "Knobs"), ("grate", "Hob")]),
            Model::new("wallOven", cookers::wall_oven, (0.70, 2.11, 0.696))
                .key("model.wallOven")
                .centre(0.0, -0.077)
                // roast and bake (the interactions, and any `bake` step) drop the upper oven door and pull its tray out
                .motions(json!({"uses": {"roast": "oven_open", "bake": "oven_open"}, "hold": (HOLD)}))
                .paint(&[("cab", "Tower"), ("chrome", "Handles")]),
            Model::new("rangeCooker", cookers::range_cooker, (1.58, 1.13, 0.755))
                .key("model.rangeCooker")
                .centre(0.0, -0.0525)
                // cook and gourmet (and any `stir` step) light a burner; bake drops the left oven door
                .motions(json!({"uses": {"cook": "burner_on", "gourmet": "burner_on", "stir": "burner_on", "bake": (bake())}}))
                .paint(&[
                    ("enamel2", "Body"),
                    ("chrome", "Knobs & rail"),
                    ("grate", "Hob"),
                ]),
            Model::new(
                "cooktopCounter",
                cookers::cooktop_counter,
                (1.0, 0.955, 0.66),
            )
            .key("model.cooktopCounter")
            .centre(0.0, -0.10)
            // cook (and any `stir` step) turns the outer knob on its side and the front ring at +x glows red
            .motions(json!({"uses": {"cook": "burner_on", "stir": "burner_on"}}))
            .paint(&[("cab", "Cabinet"), ("worktop", "Worktop"), ("hob", "Hob")]),
            Model::new("microwave", appliances::microwave, (1.0, 1.25, 0.66))
                .key("model.microwave")
                .centre(0.0, -0.095)
                // reheat (and a meal's `bake` step) opens the door to put the dish in, shuts it, then turns the
                // plate while it heats: from the door shut (turntable), round and round (`every`, a turn each 2 s)
                .motions(json!({"uses": {"reheat": ["door_open", (turntable())], "bake": ["door_open", (turntable())]}, "hold": 1.2}))
                .paint(&[
                    ("cab", "Cabinet"),
                    ("worktop", "Worktop"),
                    ("charcoal", "Microwave"),
                ]),
            Model::new(
                "coffeeMachine",
                appliances::coffee_machine,
                (1.0, 1.40, 0.66),
            )
            .key("model.coffeeMachine")
            .centre(0.0, -0.097)
            // espresso slides a cup under the spout and grows a stream of coffee; pour too, a second in (at a café
            // as the barista turns to the machine: the game's `sell` clip)
            .motions(json!({"uses": {"espresso": "brew", "pour": {"clip": "brew", "delay": 1.0}}, "hold": 2.0}))
            .paint(&[
                ("cab", "Cabinet"),
                ("worktop", "Worktop"),
                ("charcoal", "Machine"),
            ]),
            Model::new(
                "breakfastCounter",
                appliances::breakfast_counter,
                (1.0, 1.38, 0.66),
            )
            .key("model.breakfastCounter")
            .centre(0.0, -0.097)
            // toast presses the lever and the toast down (back up, popping, after `hold`); smoothie lifts the
            // lid and swirls the smoothie for as long as it blends
            .motions(json!({"uses": {"toast": "toast_down", "smoothie": "blend"}, "loops": {"smoothie": "blades"}, "hold": 2.0}))
            .paint(&[
                ("cab", "Counter"),
                ("toaster", "Toaster"),
                ("charcoal", "Blender"),
            ]),
            Model::new("dishwasher", appliances::dishwasher, (1.0, 0.95, 0.685))
                .key("model.dishwasher")
                .centre(0.0, -0.088)
                // dishes (loading it), clearing away at it (the `wash` tag) and emptying it (unload) drop the door
                // and slide the lower rack out, held open while it is loaded or emptied
                .motions(json!({"uses": {"dishes": "door_open", "wash": "door_open", "unload": "door_open"}}))
                .paint(&[("cab", "Body"), ("chrome", "Handle"), ("steel", "Panel")]),
            Model::new(
                "dishwasherCompact",
                appliances::dishwasher_compact,
                (1.0, 0.95, 0.66),
            )
            .key("model.dishwasherCompact")
            .centre(0.0, -0.10)
            // dishes, clearing away and emptying it, as the dishwasher
            .motions(json!({"uses": {"dishes": "door_open", "wash": "door_open", "unload": "door_open"}}))
            .paint(&[("cab", "Body"), ("chrome", "Handle"), ("steel", "Panel")]),
        ],
        "The kitchen's cookers (crates/art/wet/src/kitchen_cooking, built by cargo art). Listed in the \
         base manifest's packs. Each cooker is a box on the floor with a hob on top and an oven \
         below: its door drops on a bottom hinge (`oven_open`) while a resident bakes, and a knob \
         turns with a ring of flame on the gas ones (`burner_on`) while they cook. The microwave \
         swings its door (`door_open`) and turns its turntable (`turntable`).",
    )
}
