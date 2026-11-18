//! OU5, the pond, the small garden things and the garden lights: the garden pond (`model.garden.pond`), the
//! planter box (`model.planterBox`), the birdhouse (`model.birdhouse`), the bird feeders
//! (`model.birdFeederPole`), the dog house (`model.dogHouse`), the sundial (`model.sundial`), the stepping stones
//! (`model.steppingStones`), the garden lamp post (`model.lampPost`) and the string lights
//! (`model.stringLights`). Notes: tools/art/blender/families/garden_things.md.
//!
//! Each is built in game axes (`F`) where the game places it. The pond's water is opaque `gloss`, as the
//! fountains'; its ripples spread all the time near the camera (`ripple`, ambient) and a koi comes up out of it to
//! be fed or watched (`fish`, `surface`). A bird lands on the feeders while a resident watches (`bird`). The
//! planter box's leaves are on `foliage`, which the nature shader sways and tints. The lamp post's panes and the
//! string lights' bulbs glow at night (E8), each bulb near its content `light.height`; the string sways (`sway`,
//! ambient). The planter box and the lamp post have far versions (the town overview). The rest are still.
//!
//! Being ported: every model is built here, but the family is still tools/art/blender/families/garden_things.py's
//! (`cargo art garden_things` builds it in Blender until that script goes; `--out` and `--check` build what is
//! here): the lamp post's far version is Blender's decimation of it, and each model's `body` has a finish of its
//! own, which the pack's entries can't yet say.
//!
//! Run:  cargo art garden_things [--only sundial] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod birds;
pub mod common;
mod lights;
mod pond;
mod yard;

use common::SPEC;

/// The family, as far as it is built here.
pub fn family() -> Option<Family> {
    Some(
        Family::new(
            "garden_things",
            SPEC.palette(),
            vec![
                Model::new("pond", pond::pond, (1.986, 0.8, 1.823))
                    .key("model.garden.pond")
                    .motions(json!({
                        "idle": "ripple",
                        "uses": {
                            "feedFish": "fish",
                            "fish": {"clip": "surface", "delay": 1.5, "every": 7}
                        }
                    }))
                    .paint(&[("body", "Stones")]),
                Model::new("planterBox", birds::planterBox, (0.94, 0.623, 0.504))
                    .centre(0.0, birds::PLANTER_C)
                    .key("model.planterBox")
                    .far_low()
                    .paint(&[("body", "Box")]),
                Model::new("birdhouse", birds::birdhouse, (0.313, 1.915, 0.32))
                    .key("model.birdhouse")
                    .paint(&[("body", "House"), ("roof", "Roof")]),
                Model::new("birdFeederPole", birds::birdFeederPole, (0.85, 1.912, 0.271))
                    .centre(0.0, -0.1694)
                    .key("model.birdFeederPole")
                    .motions(json!({"uses": {"birdwatch": "bird"}}))
                    .paint(&[("body", "Pole"), ("feeder", "Feeders")]),
                Model::new("dogHouse", yard::dogHouse, (0.983, 0.975, 0.994))
                    .centre(0.0, -0.003)
                    .key("model.dogHouse")
                    .paint(&[("body", "House"), ("roof", "Roof")]),
                Model::new("sundial", yard::sundial, (0.36, 0.962, 0.36))
                    .key("model.sundial")
                    .paint(&[("body", "Plinth"), ("dial", "Dial")]),
                Model::new("steppingStones", yard::steppingStones, (0.592, 0.04, 2.537))
                    .key("model.steppingStones")
                    .paint(&[("body", "Stones")]),
                Model::new("lampPost", lights::lampPost, (0.32, 2.735, 0.32))
                    .key("model.lampPost")
                    .paint(&[("body", "Post"), ("lantern", "Lantern")]),
                Model::new("stringLights", lights::stringLights, (1.97, 2.37, 0.246))
                    .key("model.stringLights")
                    .motions(json!({"idle": "sway"}))
                    .paint(&[("body", "Posts"), ("bulb", "Bulbs")]),
            ],
            "Blender-built garden things and lights (tools/art/blender/families/garden_things.py): the \
             garden pond, the planter box, the birdhouse, the bird feeders, the dog house, the sundial, \
             the stepping stones, the garden lamp post and the string lights. Listed in the base \
             manifest's packs. The materials map onto the game's finishes so a model costs one draw call \
             per finish group. The pond's ripples spread near the camera (`ripple`, ambient) and a koi \
             comes up for `feedFish` (`fish`) and now and then while watched (`surface`); a blue tit \
             lands on the feeders for `birdwatch` (`bird`); the lamp's panes and the bulbs glow at night, \
             and the string sways near the camera (`sway`, ambient). The planter box and the lamp post \
             have far versions.",
        )
        .pack("garden_things"),
    )
}
