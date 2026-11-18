//! The kitchen: the top-freezer fridge (`model.fridge`), five more fridges, the wine fridge and two sinks.
//! Notes: tools/art/blender/families/kitchen.md.
//!
//! A cabinet and two doors, each door its own node turning on its hinge, with a clip per door: the game
//! opens the fridge door while a resident takes food or a drink out (`motions`). At rest both are closed.
//! Built to the old part's size, in its numbers: the parts are placed in game axes (`F`).
//!
//! Run:  cargo art kitchen [--preview]

use artkit::family::{Family, Model};
use artkit::json::Json;
use artkit::palette;
use artkit::place::GameFrame;

/// `pull` as the script called it: `pull!(F, name, a, b, face, out = .., r = None, ...)`.
macro_rules! pull {
    ($f:expr, $name:expr, $a:expr, $b:expr, $face:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::kitchen::common::Pull::default();
        $(o.$k = $v;)*
        $crate::kitchen::common::pull($f, $name, $a, $b, $face, o.material, o.out, o.w, o.t, o.r, o.verts)
    }};
}

mod built_in;
pub(crate) mod common;
mod cut;
mod french;
mod fridge;
mod fridges;
mod mini_retro;
mod retro;
mod sinks;
mod sticker;
mod wine;

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
// The retro fridge: its lever turns out first, then the door swings (`delay`) while the lever springs back.
const LEVER_SECONDS: f64 = 0.3;

fn retro_open() -> Json {
    Json::Arr(vec![
        obj(vec![
            ("clip", Json::from("lever")),
            ("hold", Json::Num(0.4)),
        ]),
        obj(vec![
            ("clip", Json::from("fridge_open")),
            ("delay", Json::Num(LEVER_SECONDS)),
        ]),
    ])
}

// The fridges with a freezer: food, a meal's ingredients and a drink from the fridge door, ice cream from the freezer.
const FRIDGE_USES: [(&str, &str); 4] = [
    ("frozenTreat", "freezer_open"),
    ("fetch", "fridge_open"),
    ("cook", "fridge_open"),
    ("drink", "fridge_open"),
];

/// A JSON object from its keys and values, in order.
fn obj(pairs: Vec<(&str, Json)>) -> Json {
    Json::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

fn uses(pairs: &[(&str, &str)]) -> Json {
    Json::Obj(
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Json::from(*v)))
            .collect(),
    )
}

/// The family. The retro mini fridge (`fridgeMiniRetro`) is not in it yet: its raised letters wait for the
/// kernel's block letters (`mini_retro::letters`), so `cargo art` builds it in Blender until they land.
pub fn family() -> Option<Family> {
    let s = Json::from;
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
                .motions(obj(vec![
                    ("uses", uses(&FRIDGE_USES)),
                    ("hold", Json::Num(HOLD)),
                ]))
                // The player colours the cabinet and doors, the handles, the plinth and the inside; the
                // food keeps its own colours.
                .paint(&[
                    ("body", "Body"),
                    ("chrome", "Handles"),
                    ("trim", "Plinth & badge"),
                    ("liner", "Inside"),
                ]),
            Model::new("fridgeMini", fridges::fridge_mini, (0.60, 1.00, 0.60))
                .key("model.fridgeMini")
                .centre(0.0, -0.128)
                .motions(obj(vec![
                    (
                        "uses",
                        uses(&[("fetch", "fridge_open"), ("cook", "fridge_open")]),
                    ),
                    ("hold", Json::Num(3.0)),
                ]))
                .paint(&[("body", "Body"), ("chrome", "Handles"), ("trim", "Plinth")]),
            Model::new(
                "fridgeBuiltIn",
                built_in::fridge_built_in,
                (0.84, 2.025, 0.656),
            )
            .key("model.fridgeBuiltIn")
            .centre(0.0, -0.099)
            // the tall door serves every use but ice cream, which opens the freezer door
            .motions(obj(vec![
                ("uses", uses(&FRIDGE_USES)),
                ("hold", Json::Num(HOLD)),
            ]))
            .paint(&[("panel", "Panels"), ("chrome", "Handles")]),
            Model::new(
                "fridgeFrenchDoor",
                french::fridge_french_door,
                (0.90, 1.82, 0.76),
            )
            .key("model.fridgeFrenchDoor")
            .centre(0.0, -0.052)
            // both doors for a snack or a meal; the right door alone for a cold drink
            .motions(obj(vec![
                (
                    "uses",
                    uses(&[
                        ("frozenTreat", "freezer_open"),
                        ("fetch", "fridge_open"),
                        ("cook", "fridge_open"),
                        ("drink", "door_right"),
                    ]),
                ),
                ("hold", Json::Num(HOLD)),
            ]))
            .paint(&[("steel", "Body"), ("chrome", "Handles"), ("trim", "Plinth")]),
            Model::new("fridgeRetro", retro::fridge_retro, (0.72, 1.64, 0.715))
                .key("model.fridgeRetro")
                .centre(0.0, -0.067)
                .motions(obj(vec![
                    (
                        "uses",
                        obj(["fetch", "cook", "drink"]
                            .into_iter()
                            .map(|u| (u, retro_open()))
                            .collect()),
                    ),
                    ("hold", Json::Num(HOLD)),
                ]))
                .paint(&[("mint", "Body"), ("chrome", "Handle"), ("mint2", "Trim")]),
            Model::new("wineFridge", wine::wine_fridge, (1.00, 0.95, 0.66))
                .key("model.wineFridge")
                .centre(0.0, -0.10)
                // pouring a glass opens the door, then slides the top shelf of bottles out (back in before it shuts)
                .motions(obj(vec![
                    (
                        "uses",
                        obj(vec![(
                            "drink",
                            Json::Arr(vec![
                                s("door_open"),
                                obj(vec![
                                    ("clip", s("rack_out")),
                                    ("delay", Json::Num(0.9)),
                                    ("hold", Json::Num(0.9)),
                                ]),
                            ]),
                        )]),
                    ),
                    ("hold", Json::Num(2.5)),
                ]))
                .paint(&[("panel", "Body"), ("doorframe", "Door frame")]),
            Model::new("farmhouseSink", sinks::farmhouse_sink, (1.0, 1.253, 0.67))
                .key("model.farmhouseSink")
                .centre(0.0, -0.095)
                .motions(obj(vec![("uses", obj(vec![("wash", s("water_on"))]))]))
                .paint(&[
                    ("body", "Cabinet"),
                    ("wood", "Worktop"),
                    ("porcelain", "Basin"),
                    ("chrome", "Tap & handles"),
                    ("trim", "Plinth"),
                ]),
            Model::new(
                "counterSinkDouble",
                sinks::counter_sink_double,
                (2.0, 1.265, 0.66),
            )
            .key("model.counterSinkDouble")
            .centre(0.0, -0.10)
            .motions(obj(vec![(
                "uses",
                obj(vec![(
                    "wash",
                    Json::Arr(
                        [0, 1]
                            .into_iter()
                            .map(|slot| {
                                obj(vec![
                                    ("clip", Json::from(format!("water_on_{slot}").as_str())),
                                    ("slot", Json::Int(slot)),
                                ])
                            })
                            .collect(),
                    ),
                )]),
            )]))
            .paint(&[
                ("body", "Cabinets"),
                ("stone", "Worktop"),
                ("metal", "Basins"),
                ("chrome", "Tap & handles"),
                ("trim", "Plinth"),
            ]),
        ],
        "The kitchen (crates/art/wet/src/kitchen, built by cargo art). Listed in the base \
         manifest's packs. The materials map onto the game's finishes so a model costs one draw call \
         per finish group. Each sink has a `water_on` clip (the stream from the tap) played while a \
         resident washes up. The fridges (K1) each hinge a door built closed and swing it open while \
         a resident takes food or a drink (`motions`); the French-door freezer slides and the wine \
         fridge's top shelf of bottles slides out once its glass door is open; every freezer door and \
         the French freezer drawer open for 'Get some ice cream'.",
    ))
}
