//! The play and yard things. Notes: NOTES.md.
//!
//! The sandpit (`model.sandbox`), whose sand castle grows while one is built (`castle`); the bike rack
//! (`model.bikeRack`), whose bike lifts its back wheel onto the stand (`wheel_up`) and turns it (`wheel_turn`, a loop)
//! for a tune-up; the raised vegetable bed (`model.raisedBed`), whose crop follows its growth stage (`grow`), rustles
//! while tended and is picked away for the harvest; the garden shed (`model.shedSmall`) and its door (`door_open`);
//! the greenhouse (`model.greenhouse`), whose door and roof vent open (`door_open`); the compost bin
//! (`model.compostBin`) and its lid (`lid_open`); the rain barrel (`model.rainBarrel`); the washing line
//! (`model.clothesline`), whose washing is pegged out line by line (`washing`) and stays out until taken in
//! (`open`); the log store (`model.firewoodStack`); the garden letterbox on its post (`model.letterbox`), whose door
//! swings open while the post is fetched (`door_open`) and whose post sticks out of its slot while it waits
//! (`post_in`, `open`); the hose
//! reel cart (`model.hoseReel`) and the push lawnmower (`model.lawnMower`). The rides, with no clips (the renderer turns their parts by code,
//! each a top-level node on its pivot: `frame`, `wheel_front`, `wheel_rear`, `cranks`, `stand`): the step-through city
//! bike with its basket, carrier and centre stand (`model.bicycle`), the kids' bike on stabilisers (`model.bicycleChild`)
//! and the e-scooter on its side stand (`model.eScooter`), the two bikes the rack's bike with their own numbers (`Bike`).
//!
//! Each is built in place where the game places it (`F`, game axes), and each has a far version for the town
//! overview: its own build at low detail.

use artkit::family::{Family, Model, motions};

mod chores;
mod common;
mod garden;
mod greenhouse;
mod line;
mod play;
mod rides;

use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "yard",
        M.palette(),
        vec![
            Model::new("sandbox", play::sandbox, (1.98, 0.355, 1.98))
                .key("model.sandbox")
                .far_low()
                .motions(motions(r#"{"uses": {"sandcastle": "castle"}}"#))
                .paint(&[("frame", "Frame")]),
            Model::new("bikeRack", play::bike_rack, (1.8, 0.832, 0.58))
                .centre(0.0, -0.07)
                .key("model.bikeRack")
                .far_low()
                .motions(motions(r#"{"uses": {"tuneUp": "wheel_up"}, "loops": {"tuneUp": "wheel_turn"}}"#))
                .paint(&[("rack", "Rack"), ("bike", "Bike")]),
            Model::new("bicycle", rides::bicycle, (1.839, 1.008, 0.645))
                .centre(-0.0046, 0.0)
                .key("model.bicycle")
                .far_low()
                .paint(&[("city", "Frame")]),
            Model::new("mountainBike", rides::mountain_bike, (1.832, 0.998, 0.605))
                .tolerance(0.02)
                .key("model.mountainBike")
                .far_low()
                .paint(&[("mtb", "Frame")]),
            Model::new("roadBike", rides::road_bike, (1.810, 0.961, 0.447))
                .tolerance(0.02)
                .key("model.roadBike")
                .far_low()
                .paint(&[("road", "Frame")]),
            Model::new("cargoBike", rides::cargo_bike, (2.727, 1.003, 0.644))
                .tolerance(0.02)
                .key("model.cargoBike")
                .far_low()
                .paint(&[("cargo", "Frame")]),
            Model::new("eBike", rides::e_bike, (1.879, 1.020, 0.624))
                .centre(-0.0046, 0.0)
                .tolerance(0.02)
                .key("model.eBike")
                .far_low()
                .paint(&[("ebike", "Frame")]),
            Model::new("bicycleChild", rides::bicycle_child, (0.949, 0.679, 0.573))
                .centre(-0.0062, 0.0)
                .key("model.bicycleChild")
                .far_low()
                .paint(&[("kid", "Frame")]),
            Model::new("eScooter", rides::e_scooter, (0.952, 1.067, 0.596))
                .centre(-0.0009, 0.0)
                .key("model.eScooter")
                .far_low()
                .paint(&[("scooter", "Body")]),
            Model::new("raisedBed", garden::raised_bed, (1.861, 0.536, 0.88))
                .key("model.raisedBed")
                .far_low()
                .motions(motions(
                    r#"{"growth": "grow", "uses": {"harvest": {"clip": "harvest", "over": 1}, "tend": {"clip": "rustle", "delay": 0.6, "every": 2.4}}}"#,
                ))
                .paint(&[("bed", "Bed")]),
            Model::new("shedSmall", garden::shed_small, (1.981, 2.37, 1.913))
                .centre(0.0, -0.056)
                .key("model.shedSmall")
                .far_low()
                .motions(motions(r#"{"uses": {"potter": "door_open"}}"#))
                .paint(&[("shed", "Wood"), ("roof", "Roof"), ("shed_door", "Door")]),
            Model::new("greenhouse", greenhouse::greenhouse, (1.805, 2.398, 2.84))
                .key("model.greenhouse")
                .far_low()
                .motions(motions(r#"{"uses": {"tend": "door_open", "harvest": "door_open"}}"#))
                .paint(&[("gh_frame", "Frame"), ("gh_wall", "Wall")]),
            Model::new("compostBin", garden::compost_bin, (0.84, 0.775, 0.792))
                .centre(0.0, -0.08)
                .key("model.compostBin")
                .far_low()
                .motions(motions(r#"{"uses": {"compost": "lid_open"}, "hold": 2}"#))
                .paint(&[("bin", "Bin")]),
            Model::new("rainBarrel", garden::rain_barrel, (0.606, 0.97, 0.686))
                .centre(0.0, 0.04)
                .key("model.rainBarrel")
                .far_low()
                .paint(&[("barrel", "Barrel")]),
            Model::new("clothesline", line::clothesline, (2.76, 1.805, 0.763))
                .centre(0.0, 0.031)
                .key("model.clothesline")
                .far_low()
                .motions(motions(
                    r#"{"uses": {"washing": {"clip": "washing", "over": 1}, "takeIn": {"clip": "washing", "over": -1}}, "open": "washing"}"#,
                ))
                .paint(&[("posts", "Posts"), ("washing", "Washing")]),
            Model::new("firewoodStack", line::firewood_stack, (1.0, 1.095, 0.377))
                .centre(0.0, -0.238)
                .key("model.firewoodStack")
                .far_low()
                .paint(&[("iron", "Shelter")]),
            Model::new("letterbox", chores::letterbox, (0.3, 1.35, 0.21))
                .key("model.letterbox")
                .far_low()
                .motions(motions(r#"{"uses": {"fetchPost": {"clip": "door_open", "hold": 2.5}}, "open": "post_in"}"#))
                .paint(&[("mailbox", "Box"), ("mail_post", "Post")]),
            Model::new("hoseReel", chores::hose_reel, (0.571, 0.848, 0.479))
                .key("model.hoseReel")
                .far_low()
                .paint(&[("reel", "Frame"), ("hose", "Hose")]),
            Model::new("lawnMower", chores::lawn_mower, (0.553, 0.992, 1.417))
                .centre(0.0, 0.2626)
                .key("model.lawnMower")
                .far_low()
                .paint(&[("mower", "Body")]),
        ],
        "The play and yard things (crates/art/outside/src/yard, built by cargo art). Listed in the base manifest's \
         packs. The materials map onto the game's finishes so a model costs one draw call per finish group. A sand \
         castle grows in the sandpit, the bike lifts its back wheel and turns it, the raised bed's crop grows and is \
         picked, the shed's and the greenhouse's doors open (and the vent), the compost bin's lid lifts, the \
         washing goes out on the line a line at a time and stays out, the post waits half out of the letterbox's \
         slot and its door swings open to fetch it.",
    ))
}
