//! The street furniture (CM2): the street lamp (`model.streetLamp`), the long park bench (`model.parkBench`), the
//! drinking fountain (`model.drinkingFountain`), the litter bin (`model.litterBin`), the taxi rank's shelter
//! (`model.busStop`), the post box (`model.postBox`), the notice board (`model.noticeBoard`), the phone box
//! (`model.phoneBooth`), the town fountain (`model.fountainTown`) and the founder's statue (`model.townStatue`); and
//! the traffic light (`model.trafficLight`), which isn't bought: the renderer stands one at each signal head of the
//! sim's lit junctions (render/pc/signals.ts). Notes: NOTES.md.
//!
//! Each is built in game axes (`GameFrame`) where the game places it, at the content's heights (the bench's and
//! the shelter's seats, the fountain's rim, the lamp's light), and each has a far version (a quarter): they stand on
//! the town's lots and are seen from afar. The lamp's lantern glows at night; the drinking fountain's arc of water
//! grows while someone drinks (`water_on`); the post box's flap swings in for a card (`flap`); the phone box's door
//! opens and shuts behind the caller (`door_open`); the town fountain's water runs all the time near the camera
//! (`water_run`, ambient). The traffic light's lit lens is the renderer's. The rest are still.

use artkit::family::{Family, Model, motions};

mod boxes;
pub(crate) mod common;
mod fountains;
mod lamps;
mod phone;
mod seats;
mod statue;

use common::{FAR, SF, own};

/// The family.
pub fn family() -> Option<Family> {
    // each model's materials are named in its own palette: the shared colours and its own `body`
    let mine = |model: Model| {
        let palette = own(model.id).palette();
        model.palette(palette)
    };
    Some(
        Family::new(
            "street_furniture",
            SF.palette(),
            vec![
                mine(Model::new("streetLamp", lamps::street_lamp, (0.657, 4.24, 0.467))
                    .key("model.streetLamp")
                    .far_ratio(FAR)
                    .paint(&[("body", "Lamp post")])),
                mine(Model::new("trafficLight", lamps::traffic_light, (0.52, 3.095, 0.56))
                    .key("model.trafficLight")
                    .centre(0.0, 0.15)
                    .far_low()
                    .code()),
                mine(Model::new("parkBench", seats::park_bench, (2.5, 0.907, 0.572))
                    .key("model.parkBench")
                    .centre(0.0, -0.036)
                    .far_ratio(FAR)
                    .paint(&[("body", "Frame")])),
                mine(Model::new("drinkingFountain", fountains::drinking_fountain, (0.56, 1.093, 0.56))
                    .key("model.drinkingFountain")
                    .far(fountains::drinking_fountain_far)
                    .motions(motions(r#"{"uses": {"drink": "water_on"}}"#))
                    .paint(&[("body", "Pillar")])),
                mine(Model::new("litterBin", boxes::litter_bin, (0.474, 0.97, 0.582))
                    .key("model.litterBin")
                    .centre(0.0, -0.034)
                    .far_ratio(FAR)
                    .paint(&[("body", "Bin")])),
                mine(Model::new("busStop", seats::bus_stop, (2.98, 2.89, 1.205))
                    .key("model.busStop")
                    .far_ratio(FAR)
                    .paint(&[("body", "Frame")])),
                mine(Model::new("postBox", boxes::post_box, (0.5, 1.529, 0.524))
                    .key("model.postBox")
                    .far_ratio(FAR)
                    .motions(motions(r#"{"uses": {"post": "flap"}, "hold": 1}"#))
                    .paint(&[("body", "Box")])),
                mine(Model::new("noticeBoard", boxes::notice_board, (1.02, 2.065, 0.347))
                    .key("model.noticeBoard")
                    .centre(0.0, -0.05)
                    .far_ratio(FAR)
                    .paint(&[("body", "Frame")])),
                mine(Model::new("phoneBooth", phone::phone_booth, (0.952, 2.7, 0.952))
                    .key("model.phoneBooth")
                    .far_ratio(FAR)
                    .motions(motions(
                        r#"{"uses": {"call": {"clip": "door_open", "delay": 0.3, "hold": 2.4, "out": 1.6}}}"#,
                    ))
                    .paint(&[("body", "Kiosk")])),
                mine(Model::new("fountainTown", fountains::fountain_town, (2.96, 2.473, 2.94))
                    .key("model.fountainTown")
                    .far(fountains::fountain_town_far)
                    .motions(motions(r#"{"idle": "water_run"}"#))
                    .paint(&[("body", "Stone")])),
                mine(Model::new("townStatue", statue::town_statue, (1.8, 3.39, 1.8))
                    .key("model.townStatue")
                    .far(statue::town_statue_far)
                    .paint(&[("body", "Statue")])),
            ],
            "The street furniture (crates/art/outside/src/street_furniture, built by cargo art): the street lamp, the \
             park bench, the drinking fountain, the litter bin, the taxi rank's shelter, the post box, the notice \
             board, the phone box, the town fountain and the founder's statue; and the traffic light (not buyable: \
             render/pc/signals.ts draws one at each signal head the sim has). Listed in the base manifest's packs. The \
             materials map onto the game's finishes so a model costs one draw call per finish group. Each has a far \
             version for the town overview. The lamp's lantern, the phone box's signs and the taxi rank's sign glow at \
             night; the drinking fountain's water arcs for `drink` (`water_on`), the post box's flap swings in for \
             `post` (`flap`), the phone box's door opens for `call` (`door_open`) and the town fountain's water \
             runs near the camera (`water_run`, ambient).",
        )
        .pack("street_furniture")
        .switched(),
    )
}
