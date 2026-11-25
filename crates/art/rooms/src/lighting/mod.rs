//! The lamps: the ceiling lights (LT1: `ceilingPendant`, `ceilingLight`, `chandelier`, `pendantIndustrial`,
//! `ceilingFan`, `trackLights`, `paperLantern`), the wall lights (LT2: `wallSconce`, `wallLampArm`,
//! `wallLightOutdoor`), the floor lamps (LT3: `lamp`, `floorLampSimple`, `lampFloorArc`, `lampFloorTripod`,
//! `lampTorchiere`) and the table lamps (LT4: `tableLamp`, `lampTableRound`, `lampTableSquare`, `lampBanker`,
//! `lampMushroom`, `lampLantern`). Notes: NOTES.md.
//!
//! The light itself is the content's `light`; each lamp's bulb is placed near its `light.height`, and once built at
//! its `light.at` where it gives one (both asserted against the content JSON). The shades, diffusers and bulbs glow
//! at night (`glow`, E8); nothing glows by day. Ceiling things hang from 2.8 m and wall things have their back on
//! the wall face, both at their real height (not on the floor). One clip, ambient (`idle`): the ceiling fan's blades
//! turn while it's on. The rest are still (the paper lantern's sway went, 2026-10-21: a paper globe indoors hangs
//! still).

use artkit::family::{Family, Model, motions};

mod ceiling;
mod common;
mod standing;
mod table;

use common::M;

/// A lamp keyed `model.<id>`.
macro_rules! model {
    ($id:literal, $build:expr, $size:expr, $paint:expr $(,)?) => {
        Model::new($id, $build, $size)
            .key(concat!("model.", $id))
            .paint($paint)
    };
}

/// The family.
pub fn family() -> Family {
    Family::new(
        "lighting",
        M.palette(),
        vec![
            model!(
                "ceilingPendant",
                ceiling::ceiling_pendant,
                (0.43, 0.688, 0.43),
                &[("brass", "Body"), ("dome", "Shade")]
            )
            .on_wall(),
            model!(
                "ceilingLight",
                ceiling::ceiling_light,
                (0.36, 0.6, 0.36),
                &[("iron", "Body"), ("panel", "Shade")]
            )
            .on_wall(),
            model!(
                "chandelier",
                ceiling::chandelier,
                (0.94, 0.763, 0.897),
                &[("brass", "Body"), ("bulb", "Globes")]
            )
            .on_wall(),
            model!(
                "pendantIndustrial",
                ceiling::pendant_industrial,
                (0.38, 0.874, 0.38),
                &[("iron", "Body"), ("industrial", "Shade")]
            )
            .on_wall(),
            // its box's middle; the blades' axis is on the tile's
            model!(
                "ceilingFan",
                ceiling::ceiling_fan,
                (0.905, 0.385, 0.951),
                &[("white", "Body"), ("blade", "Blades")]
            )
            .on_wall()
            .centre(-0.0374, 0.0)
            .motions(motions(r#"{"idle": "blades_turn"}"#)),
            model!(
                "trackLights",
                ceiling::track_lights,
                (0.94, 0.296, 0.19),
                &[("white", "Body"), ("iron", "Shade")]
            )
            .on_wall(),
            model!(
                "paperLantern",
                ceiling::paper_lantern,
                (0.543, 0.681, 0.543),
                &[("paper", "Paper"), ("rib", "Ribs")]
            )
            .on_wall(),
            model!(
                "wallSconce",
                standing::wall_sconce,
                (0.2, 0.25, 0.27),
                &[("brass", "Body"), ("shade", "Shade")]
            )
            .on_wall()
            .centre(0.0, -0.295),
            model!(
                "wallLampArm",
                standing::wall_lamp_arm,
                (0.2, 0.296, 0.424),
                &[("iron", "Body"), ("shade_metal", "Shade")]
            )
            .on_wall()
            .centre(0.0, -0.218),
            model!(
                "wallLightOutdoor",
                standing::wall_light_outdoor,
                (0.22, 0.327, 0.11),
                &[("iron", "Body"), ("lens", "Shade")]
            )
            .on_wall()
            .centre(0.0, -0.375),
            model!(
                "lamp",
                standing::lamp,
                (0.45, 1.808, 0.45),
                &[
                    ("stand", "Stand"),
                    ("lamp_shade", "Shade"),
                    ("brass", "Fittings")
                ]
            ),
            model!(
                "floorLampSimple",
                standing::floor_lamp_simple,
                (0.28, 1.6, 0.28),
                &[("iron", "Stand"), ("shade", "Shade")]
            ),
            // its box's middle: the base's back on the wall face, the shade over the next tile
            model!(
                "lampFloorArc",
                standing::lamp_floor_arc,
                (0.611, 2.055, 1.611),
                &[("arc", "Stand"), ("arc_shade", "Shade"), ("marble", "Base")],
            )
            .centre(-0.1245, 0.3755),
            model!(
                "lampFloorTripod",
                standing::lamp_floor_tripod,
                (0.554, 1.614, 0.557),
                &[("walnut", "Stand"), ("linen", "Shade")]
            ),
            model!(
                "lampTorchiere",
                standing::lamp_torchiere,
                (0.4, 1.85, 0.4),
                &[("brass", "Stand"), ("bowl", "Shade"), ("iron", "Base")],
            ),
            model!(
                "tableLamp",
                table::table_lamp,
                (0.66, 0.92, 0.42),
                &[("ceramic", "Base"), ("shade", "Shade"), ("oak", "Table")]
            ),
            model!(
                "lampTableRound",
                table::lamp_table_round,
                (0.34, 0.577, 0.34),
                &[("jar", "Base"), ("shade", "Shade")]
            ),
            model!(
                "lampTableSquare",
                table::lamp_table_square,
                (0.212, 0.54, 0.212),
                &[("stone", "Base"), ("square_shade", "Shade")]
            ),
            model!(
                "lampBanker",
                table::lamp_banker,
                (0.3, 0.39, 0.16),
                &[("brass", "Base"), ("banker", "Shade")]
            ),
            model!(
                "lampMushroom",
                table::lamp_mushroom,
                (0.3, 0.36, 0.3),
                &[("porcelain", "Base"), ("cap", "Shade")]
            ),
            model!(
                "lampLantern",
                table::lamp_lantern,
                (0.16, 0.373, 0.16),
                &[("iron", "Base"), ("glass", "Glass")]
            ),
        ],
        "The lamps (crates/art/rooms/src/lighting, built by cargo art). Listed in the base manifest's packs. The \
         materials map onto the game's finishes so a model costs one draw call per finish group. Shades, diffusers \
         and bulbs glow at night (`glow`); the ceiling fan's blades turn while it is on, near the camera (`idle`).",
    )
}
