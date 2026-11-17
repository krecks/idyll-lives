//! The kitchen: the top-freezer fridge (`model.fridge`). Notes: tools/art/blender/families/kitchen.md.
//!
//! Being ported: the fridge is built here; the family's other models are still tools/art/blender/families/kitchen.py's
//! (`cargo art kitchen` builds them all in Blender until that script goes; `--out` and `--check` build what is here).

use artkit::family::{Family, Model};
use artkit::json::Json;
use artkit::palette;
use artkit::place::GameFrame;

mod fridge;
mod soft;

palette! { M:
    body = ("#F3F3F1", gloss),       // white cabinet and doors; the main part the player colours (`paint`)
    trim = ("#B9BCBF", satin),       // plinth and badge
    seal = ("#3A3D41", satin),       // the gap between the doors
    gasket = ("#C9CDD0", satin),     // the doors' rubber seals, pale grey as on a white fridge
    chrome = ("#D0D3D6", chrome),    // bar handles
    liner = ("#DCE0E3", matte),      // inside walls
    shelf = ("#E9ECEE", gloss),      // shelves, divider, tub
    carton = ("#EAF0F4", satin),     // milk and egg cartons
    bottle = ("#4E8A54", gloss),     // juice bottle
    produce = ("#C0392B", satin),    // apples
    produce2 = ("#E08A2E", satin),   // orange
    egg = ("#F2E6C9", matte),        // eggs
    frozen = ("#8FB6D6", satin),     // frozen pizza box
    icecream = ("#E8B4C8", satin),   // ice cream tub
    peas = ("#6FA96F", satin),       // frozen peas
    porcelain = ("#F6F6F4", gloss),  // sink basins and the farmhouse apron
    pot = ("#B5653F", satin),        // a terracotta herb pot
    stone = ("#DCDDDA", gloss),      // sink worktops
    wood = ("#B98A5A", wood),        // the farmhouse worktop
    board = ("#B98A5A", satin),      // the mini fridge's chopping board (plain: no draw call of its own)
    metal = ("#8E9196", metal),      // basin interiors
    dark = ("#2F3236", satin),       // drains
    plant = ("#5C8A4A", satin),      // a potted herb on the counter
    water = ("#9EC9DD", gloss),      // the stream from the tap
    // the rest of the fridge family (K1)
    steel = ("#C3C7CB", metal),      // a stainless French-door fridge
    panel = ("#EDEBE6", satin),      // built-in cabinet fronts and the wine fridge's body
    mint = ("#BFE0D2", gloss),       // a retro fridge's rounded body
    mint2 = ("#A9CFC0", gloss),      // its trim (the plinth and the door seam)
    pink = ("#F4B9CF", gloss),       // a retro mini fridge's pastel body
    pink2 = ("#E9A2BD", gloss),      // its trim (the base band and the door seam)
    gold = ("#D2AE4E", metal),       // its brass handle, feet and letters
    vent = ("#3A3D41", satin),       // its side vents
    uni_white = ("#FFFFFF", satin),  // the unicorn sticker: its die-cut border, a streak and the horn
    uni_face = ("#FCEEF3", satin),   // its head and ear
    uni_pink = ("#F2A6C6", satin),   // its mane and inner ear
    uni_rose = ("#E07FAA", satin),   // a deep streak, the nostril and the eyelid
    uni_blush = ("#F6C6D9", satin),  // the muzzle
    uni_blue = ("#B9D3EE", satin),   // a streak of the mane
    uni_lilac = ("#D4C1EC", satin),  // a streak and the horn's spiral
    uni_dark = ("#3A2B33", matte),   // its eye
    doorframe = ("#2B2C2E", satin),  // the wine fridge's dark door frame
    smoked = ("#3A2A2E", glass),     // its smoked glass
    wine = ("#5A1E2A", gloss),       // red wine bottles
    wine2 = ("#E3D9A0", gloss),      // white wine bottles
}

// The old part's back sits 9.4 cm behind the middle of its tile (the old `BACK = -0.455`);
// the model is centred on its tile, so every game depth is shifted back by that much.
const F: GameFrame = GameFrame::new(0.094);

// Both doors open at one speed; the short freezer door swings wider than the tall fridge door.
const DOOR_SECONDS: f64 = 1.75;
const FREEZER_ANGLE: f64 = -118.0;
const FRIDGE_ANGLE: f64 = -102.0;
// Seconds (the residents' animation time) a door stays opening or open once a use starts: it opens in
// 1.75 s, stays open while they reach in, then closes.
const HOLD: f64 = 3.5;

// The fridges with a freezer: food, a meal's ingredients and a drink from the fridge door, ice cream from the freezer.
const FRIDGE_USES: [(&str, &str); 4] = [
    ("frozenTreat", "freezer_open"),
    ("fetch", "fridge_open"),
    ("cook", "fridge_open"),
    ("drink", "fridge_open"),
];

fn uses(pairs: &[(&str, &str)]) -> Json {
    Json::Obj(
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Json::from(*v)))
            .collect(),
    )
}

/// The family, as far as it is built here.
pub fn family() -> Option<Family> {
    // A floor thing against a wall is centred where the game places it (`centre`): its back on the wall,
    // its top under its content `surface`.
    Some(Family::new(
        "kitchen",
        M.palette(),
        vec![
            Model::new("fridge", fridge::fridge, (0.74, 1.8, 0.722))
                .key("model.fridge")
                .centre(0.0, -0.069)
                // Taking food or a drink out opens the fridge door (snacks, a meal's ingredients (cook), a cold
                // drink, putting dishes away); ice cream (`frozenTreat`) opens the freezer door.
                .motions(Json::Obj(vec![
                    ("uses".into(), uses(&FRIDGE_USES)),
                    ("hold".into(), Json::Num(HOLD)),
                ]))
                // The player colours the cabinet and doors, the handles, the plinth and the inside; the
                // food keeps its own colours.
                .paint(&[
                    ("body", "Body"),
                    ("chrome", "Handles"),
                    ("trim", "Plinth & badge"),
                    ("liner", "Inside"),
                ]),
        ],
        "Blender-built kitchen models (tools/art/blender/families/kitchen.py). Listed in the base \
         manifest's packs. The materials map onto the game's finishes so a model costs one draw call \
         per finish group. Each sink has a `water_on` clip (the stream from the tap) played while a \
         resident washes up. The fridges (K1) each hinge a door built closed and swing it open while \
         a resident takes food or a drink (`motions`); the French-door freezer slides and the wine \
         fridge's top shelf of bottles slides out once its glass door is open; every freezer door and \
         the French freezer drawer open for 'Get some ice cream'.",
    ))
}
