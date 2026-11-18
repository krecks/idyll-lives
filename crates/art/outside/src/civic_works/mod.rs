//! The big working things of the town's named places (content/civicWorks.json, the 'community' buy category): the
//! fire station's fire engine, fire pole and hose rack, the garage's two-post car lift, tyre rack and tool chest,
//! the newspaper's printing press, the server rack, and the builders' yard's timber stack, cement mixer and the
//! town's flagpole (`model.<id>`). Notes: NOTES.md.
//!
//! Built in game axes (`F`: x, height, depth; the front +depth) where the game places them, then stood on the
//! floor (`settle`). The things against a wall (the hose rack on it; the server rack, the tyre rack and the tool
//! chest on the floor) have their backs on the wall's face (-0.43) and a `centre` on their Model. Each model has its
//! own palette, its main part `body` (a red engine, a black cabinet, an orange mixer), over the shared colours
//! (`own_palette!`). The fire engine, the car lift and the outdoor things have far versions for the town overview
//! (built by hand at low detail, `far_low`). Three clips:
//! - the printing press's two cylinders turn against each other while a resident watches it run (`rollers`, a loop);
//! - the server rack's LEDs blink near the camera all the time (`blink`, ambient);
//! - the flag sways at the top of its pole near the camera (`flutter`, ambient).

use artkit::family::{Family, Model};
use artkit::json::{Json, loads};

pub(crate) mod common;
mod engine;
mod garage;
mod lift;
mod press;
mod server;
mod station;
mod yard;

use common::S;

fn motions(text: &str) -> Json {
    loads(text).expect("the motions are JSON")
}

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "civic_works",
        S.palette(),
        vec![
            Model::new("fireEngine", engine::fire_engine, (2.51, 3.17, 5.843))
                .key("model.fireEngine")
                .far_low()
                .paint(&[
                    ("body", "Paint"),
                    ("shutter", "Lockers"),
                    ("stripe", "Stripes"),
                    ("ladder", "Ladders"),
                ])
                .palette(engine::ENGINE.palette()),
            Model::new("carLift", lift::car_lift, (2.94, 2.76, 4.09))
                .key("model.carLift")
                .far_low()
                .paint(&[("body", "Lift"), ("car", "Car"), ("alloy", "Wheels")])
                .palette(lift::LIFT.palette()),
            Model::new("printingPress", press::printing_press, (1.72, 1.51, 1.864))
                .key("model.printingPress")
                .motions(motions(r#"{"loops": {"watch": "rollers"}}"#))
                .paint(&[("body", "Press")])
                .palette(press::palette()),
            Model::new("firePole", station::fire_pole, (0.84, 2.8, 0.84))
                .key("model.firePole")
                .paint(&[("body", "Pole"), ("mat", "Mat")])
                .palette(station::POLE.palette()),
            Model::new("hoseRack", station::hose_rack, (0.88, 0.786, 0.3))
                .key("model.hoseRack")
                .centre(0.0, -0.28)
                .on_wall()
                .paint(&[("body", "Rack"), ("hose", "Hose"), ("hose2", "Second hose")])
                .palette(station::HOSE.palette()),
            Model::new("serverRack", server::server_rack, (0.6, 2.003, 0.889))
                .key("model.serverRack")
                .centre(0.0, 0.0145)
                .motions(motions(r#"{"idle": "blink"}"#))
                .paint(&[("body", "Cabinet"), ("unit", "Servers")])
                .palette(server::SERVER.palette()),
            Model::new("tyreRack", garage::tyre_rack, (1.96, 2.0, 0.66))
                .key("model.tyreRack")
                .centre(0.0, -0.1)
                .paint(&[("body", "Frame"), ("beam", "Beams")])
                .palette(garage::TYRES.palette()),
            Model::new("toolChest", garage::tool_chest, (0.79, 1.429, 0.661))
                .key("model.toolChest")
                .centre(-0.0132, -0.0995)
                .paint(&[("body", "Chest")])
                .palette(garage::CHEST.palette()),
            Model::new("timberStack", yard::timber_stack, (1.86, 0.598, 0.931))
                .key("model.timberStack")
                .far_low()
                .paint(&[("body", "Tarpaulin"), ("timber", "Timber")])
                .palette(yard::TIMBER.palette()),
            Model::new("cementMixer", yard::cement_mixer, (0.8, 1.163, 0.965))
                .key("model.cementMixer")
                .far_low()
                .paint(&[("body", "Mixer"), ("frame", "Stand")])
                .palette(yard::MIXER.palette()),
            Model::new("flagPole", yard::flag_pole, (1.874, 6.17, 0.6))
                .key("model.flagPole")
                .centre(0.6368, 0.0)
                .far_low()
                .motions(motions(r#"{"idle": "flutter"}"#))
                .paint(&[("body", "Pole"), ("flag", "Flag")])
                .palette(yard::FLAG.palette()),
        ],
        "The big working things of the town's named places (crates/art/outside/src/civic_works, built by cargo art; \
         content/civicWorks.json): the fire engine, fire pole and hose rack, the two-post car lift, tyre rack and \
         tool chest, the printing press, the server rack, the timber stack, cement mixer and flagpole. Listed in the \
         base manifest's packs. The materials map onto the game's finishes so a model costs one draw call per \
         finish group. The fire engine, the car lift and the outdoor things have far versions for the town \
         overview. The press's plate and blanket cylinders turn while a resident watches it run (`rollers`); the \
         server rack's LEDs blink (`blink`) and the flag sways at the top of its pole (`flutter`), both ambient; \
         the server rack's LEDs and its UPS's screen glow at night.",
    ).switched())
}
