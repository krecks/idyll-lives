//! The decor: wall art and clocks (D1: `model.printFramed`, `photoPrint`, `paintingClassic`, `canvasAbstract`,
//! `wallClock`, `wallShelf`), ornaments (D2: `grandfatherClock`, `vaseTall`, `urnPorcelain`, `vasePedestal`,
//! `bustMarble`, `statueCat`, `statueHorse`, `coatRack`, `hangingPlant`, `fireplace`) and textiles (D3:
//! `curtains`, `curtainsLong`, `rugRound`, `rugLarge`, `rugRunner`, `rugPattern`). Notes: NOTES.md.
//!
//! Built where the game places them (`F`, game axes): the wall things at their real height with their back on the
//! wall face (not on the floor), the hanging plant from the ceiling. What moves:
//!   - the clocks' hour and minute hands tell the game's time (`hour_hand`, `minute_hand`, `motions.clock`); the
//!     grandfather clock's pendulum swings all the time (`pendulum`, `idle`) and its trunk door opens while it is
//!     wound (`door_open`, `wind`): its case is plain satin, not wood, and the entry's `draws` of 5 allows the door;
//!   - the hanging plant sways when watered (`sway`);
//!   - the fireplace's flames grow on the logs while a resident warms up (`flames`);
//!   - the curtains are drawn closed (`close`, `motions.night`: held while the game has them drawn, sim-core
//!     `curtains.rs`), on cloth tabs over the rail; the blinds' slats shut and the shutters swing closed the same
//!     way (`close`).

use artkit::family::{Family, Model, motions};
use artkit::json::Json;

mod clock;
mod common;
mod fittings;
mod hangers;
mod hearth;
mod statues;
mod textiles;
mod vases;
mod wall;

use common::M;

/// The clocks' motions: the hands tell the game's time.
fn clock() -> Json {
    motions(&format!("{{{}}}", common::clock_json()))
}

// A window's opening is its edges less 0.12 m at each end (house.ts WIN_INSET): 0.76 m over one edge, 1.76 m over
// two. A rail runs past it, so the open curtains stack on the wall beside the glass, not over it: each pair's `w` is
// the opening plus its two gathered panels (rails 1.22 and 2.38 m, 0.1-0.2 m past the footprint each side). The
// window-height pairs hang from above the head (2.1 m) to just under the sill (0.9 m). Past their footprint, along
// the wall, by design (`cargo check-glb`'s `footprint`):
const CURTAINS_ACCEPT: &str = "the rail runs past the window, its opening plus a gathered panel each side, so the open \
                               curtains stack on the wall beside the glass (decor.md, M-decor-02)";

// Folded back by day, the shutters lie flat on the wall either side of the window, 0.39 m past their footprint each
// side, as an open curtain's panels stack beside the glass (`cargo check-glb`'s `footprint`):
const SHUTTERS_ACCEPT: &str = "folded back by day, the two panels lie flat on the wall either side of the window, past \
                               the footprint, and swing shut over it at night";

/// The family.
pub fn family() -> Family {
    let night = || motions(r#"{"night": "close"}"#);
    Family::new(
        "decor",
        M.palette(),
        vec![
            // D1. Wall art and clocks
            Model::new("printFramed", wall::print_framed, (0.5, 0.71, 0.03))
                .key("model.printFramed")
                .centre(0.0, -0.415)
                .on_wall()
                .paint(&[("frame_white", "Frame")]),
            Model::new("photoPrint", wall::photo_print, (0.75, 0.5, 0.033))
                .key("model.photoPrint")
                .centre(0.0, -0.4135)
                .on_wall()
                .paint(&[("frame_black", "Frame")]),
            Model::new(
                "paintingClassic",
                wall::painting_classic,
                (0.84, 0.65, 0.05),
            )
            .key("model.paintingClassic")
            .centre(0.0, -0.405)
            .on_wall()
            .paint(&[("gilt", "Frame")]),
            Model::new("canvasAbstract", wall::canvas_abstract, (1.5, 0.9, 0.05))
                .key("model.canvasAbstract")
                .centre(0.0, -0.405)
                .on_wall()
                .paint(&[("oak", "Frame")]),
            Model::new("wallClock", wall::wall_clock, (0.36, 0.36, 0.066))
                .key("model.wallClock")
                .centre(0.0, -0.397)
                .on_wall()
                .motions(clock())
                .paint(&[("clock_case", "Case"), ("face", "Face")]),
            Model::new("wallShelf", wall::wall_shelf, (0.9, 0.659, 0.24))
                .key("model.wallShelf")
                .centre(0.0, -0.31)
                .on_wall()
                .paint(&[("oak", "Shelves")]),
            // D2. Ornaments
            Model::new(
                "grandfatherClock",
                clock::grandfather_clock,
                (0.6, 2.116, 0.4),
            )
            .key("model.grandfatherClock")
            .centre(0.0, -0.23)
            .motions(motions(&format!(
                r#"{{{}, "idle": "pendulum", "uses": {{"wind": "door_open"}}}}"#,
                common::clock_json()
            )))
            .draws(5)
            .paint(&[("walnut", "Case"), ("dial", "Face")]),
            Model::new("vaseTall", vases::vase_tall, (0.329, 1.27, 0.224))
                .key("model.vaseTall")
                .paint(&[("ceramic", "Body")]),
            Model::new("urnPorcelain", vases::urn_porcelain, (0.36, 0.62, 0.363))
                .key("model.urnPorcelain")
                .paint(&[("porcelain", "Body")]),
            Model::new("vasePedestal", vases::vase_pedestal, (0.4, 1.6, 0.4))
                .key("model.vasePedestal")
                .paint(&[("vase_blue", "Vase"), ("stone", "Pedestal")]),
            Model::new("bustMarble", statues::bust_marble, (0.44, 1.539, 0.44))
                .key("model.bustMarble")
                .paint(&[("marble", "Bust"), ("stone", "Plinth")]),
            Model::new("statueCat", statues::statue_cat, (0.311, 0.598, 0.40))
                .key("model.statueCat")
                .paint(&[("concrete", "Body")]),
            Model::new("statueHorse", statues::statue_horse, (0.46, 1.41, 0.44))
                .key("model.statueHorse")
                .paint(&[("bronze", "Body")]),
            Model::new("coatRack", hangers::coat_rack, (0.431, 1.788, 0.355))
                .key("model.coatRack")
                .paint(&[("oak", "Body")]),
            Model::new("hangingPlant", hangers::hanging_plant, (0.46, 1.83, 0.46))
                .key("model.hangingPlant")
                .on_wall()
                .motions(motions(r#"{"uses": {"water": "sway"}, "hold": 0.8}"#))
                .paint(&[("pot_white", "Pot"), ("cord", "Cord")]),
            Model::new("fireplace", hearth::fireplace, (1.96, 2.8, 0.88))
                .key("model.fireplace")
                .centre(0.0, 0.01)
                .motions(motions(r#"{"uses": {"relax": "flames"}}"#))
                .paint(&[("surround", "Surround"), ("oak", "Mantel")]),
            // D3. Textiles
            Model::new("curtains", textiles::curtains, (1.288, 1.674, 0.104))
                .key("model.curtains")
                .centre(0.0, -0.378)
                .on_wall()
                .motions(night())
                .paint(&[("linen", "Fabric"), ("rail", "Rail")])
                .accept("footprint", CURTAINS_ACCEPT),
            Model::new(
                "curtainsWide",
                textiles::curtains_wide,
                (2.448, 1.674, 0.104),
            )
            .key("model.curtainsWide")
            .centre(0.0, -0.378)
            .on_wall()
            .motions(night())
            .paint(&[("linen", "Fabric"), ("rail", "Rail")])
            .accept("footprint", CURTAINS_ACCEPT),
            Model::new(
                "curtainsLong",
                textiles::curtains_long,
                (2.448, 2.584, 0.104),
            )
            .key("model.curtainsLong")
            .centre(0.0, -0.378)
            .on_wall()
            .motions(night())
            .paint(&[("linen", "Fabric"), ("rail", "Rail")])
            .accept("footprint", CURTAINS_ACCEPT),
            Model::new("rugRound", textiles::rug_round, (1.9, 0.019, 1.9))
                .key("model.rugRound")
                .paint(&[("rug", "Rug")]),
            Model::new("rugLarge", textiles::rug_large, (2.85, 0.019, 1.85))
                .key("model.rugLarge")
                .paint(&[("rug", "Rug")]),
            Model::new("rugRunner", textiles::rug_runner, (0.8, 0.019, 2.8))
                .key("model.rugRunner")
                .paint(&[("rug", "Rug")]),
            Model::new("rugPattern", textiles::rug_pattern, (2.84, 0.024, 1.86))
                .key("model.rugPattern")
                .paint(&[("rug_ground", "Rug")]),
            Model::new("blinds", fittings::blinds, (0.90, 0.806, 0.070))
                .key("model.blinds")
                .centre(0.0, -0.395)
                .on_wall()
                .motions(night())
                .paint(&[("blind_slat", "Slats"), ("blind_rail", "Head rail")]),
            Model::new("coatHooks", fittings::coat_hooks, (0.90, 0.14, 0.111))
                .key("model.coatHooks")
                .centre(0.0, -0.3745)
                .on_wall()
                .paint(&[("hook_board", "Board"), ("gadget_chrome", "Hooks")]),
            Model::new("doorbell", fittings::doorbell, (0.084, 0.084, 0.036))
                .key("model.doorbell")
                .centre(0.0, -0.412)
                .on_wall()
                .paint(&[("bell_brass", "Brass"), ("bell_push", "Button")]),
            Model::new("burglarAlarm", fittings::burglar_alarm, (0.22, 0.28, 0.061))
                .key("model.burglarAlarm")
                .centre(0.0, -0.3995)
                .on_wall()
                .motions(motions(r#"{"idle": "blink"}"#))
                .paint(&[("gadget_white", "Case"), ("gadget_dark", "Keypad")]),
            Model::new("shutters", fittings::shutters, (1.765, 0.765, 0.050))
                .key("model.shutters")
                .centre(0.0, -0.395)
                .on_wall()
                .motions(night())
                .paint(&[("blind_rail", "Panels"), ("hook_board", "Frame")])
                .accept("footprint", SHUTTERS_ACCEPT),
            Model::new("hallStand", fittings::hall_stand, (0.90, 1.80, 0.35))
                .key("model.hallStand")
                .centre(0.0, -0.255)
                .on_wall()
                .paint(&[
                    ("oak", "Wood"),
                    ("gadget_chrome", "Hooks"),
                    ("hall_glass", "Mirror"),
                ]),
            Model::new("cctvCamera", fittings::cctv_camera, (0.11, 0.14, 0.351))
                .key("model.cctvCamera")
                .centre(0.0, -0.2545)
                .on_wall()
                .paint(&[("gadget_white", "Housing"), ("cam_lens", "Lens")]),
        ],
        "The decor (crates/art/rooms/src/decor, built by cargo art): wall art and clocks, ornaments, curtains and \
             rugs. Listed in the base manifest's packs. The materials map onto the game's finishes so a model costs one \
             draw call per finish group. The clocks' hands tell the game's time (`clock`); the hanging plant sways when \
             watered, the fire burns while a resident warms up by it, and the curtains, blinds and shutters close at night.",
    )
}
