//! The gloomy pack (P7): the midnight record cabinet, the nocturne cello, the candlelit writing desk, the rainfall
//! lamp, the brooding window seat and the moonstone soaking tub. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built). The
//! cabinet's lid lifts and its record turns, the cello comes off its stand, a candle is lit, the rain runs down the
//! lamp, the window seat's cushion sinks and the tub fills.
//!
//! Run:  cargo art pack_gloomy [--only cello] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod common;
mod desk;
mod music;
mod rain;
mod seat;

pub use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "pack_gloomy",
        M.palette(),
        vec![
            Model::new("recordCabinet", music::record_cabinet, (0.9, 0.925, 0.814))
                .key("model.gloomy.recordCabinet")
                .centre(0.0, -0.025)
                .motions(
                    json!({"uses": {"listen": "lid_open", "newAlbum": "lid_open"},
                                "loops": {"listen": "platter_spin", "newAlbum": "platter_spin"}}),
                )
                .paint(&[("walnut", "Body"), ("ink", "Trim")]),
            Model::new("cello", music::cello, (0.78, 1.427, 0.741))
                .key("model.gloomy.cello")
                .centre(0.06, 0.091)
                .motions(json!({"uses": {"practise": "off_stand", "lament": "off_stand"}}))
                .paint(&[("rosewood", "Body"), ("plum", "Trim"), ("varnish", "Cello")]),
            Model::new("journalDesk", desk::journal_desk, (1.5, 1.198, 0.834))
                .key("model.gloomy.journalDesk")
                .centre(0.0, 0.043)
                .motions(json!({"uses": {"journal": "candle", "poetry": "candle"}}))
                .paint(&[("rosewood", "Body"), ("plum", "Trim")]),
            Model::new("rainLamp", rain::rain_lamp, (0.804, 1.498, 0.618))
                .key("model.gloomy.rainLamp")
                .centre(0.0, 0.039)
                .motions(json!({"uses": {"watchRain": "rain"}, "idle": "ripple"}))
                .paint(&[("slate", "Body"), ("umbrella", "Trim")]),
            Model::new("windowSeat", seat::window_seat, (1.84, 1.97, 0.706))
                .key("model.gloomy.windowSeat")
                .centre(0.0, -0.077)
                .motions(json!({"uses": {"brood": "sink_in", "doze": "sink_in"}}))
                .paint(&[("bench", "Body"), ("cushion", "Cushion"), ("frame", "Trim")]),
            Model::new("soakingTub", seat::soaking_tub, (1.86, 0.922, 1.85))
                .key("model.gloomy.soakingTub")
                .centre(0.0, -0.005)
                .motions(json!({"uses": {"soak": "fill"}}))
                .paint(&[("stone", "Body"), ("ledge", "Trim")]),
        ],
        "Gloomy pack models (crates/art/packs/src/pack_gloomy, built by cargo art). Listed in the base manifest's \
         packs. The materials map onto the game's finishes so a model costs one draw call per finish group. The \
         cabinet's lid lifts and its record turns, the cello comes off its stand, a candle is lit, the rain runs \
         down the lamp, the window seat's cushion sinks and the tub fills.",
    ).switched())
}
