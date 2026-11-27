//! The bedroom's beds (BR1): the double bed (`model.bed`), the single (`model.bedSingle`), the bunk bed
//! (`model.bedBunk`), the futon (`model.bedFuton`), the upholstered king (`model.bedKing`), the oak double
//! (`model.bedDouble`), the upholstered queen (`model.bedQueenUpholstered`), the four-poster (`model.bedCanopy`)
//! and the daybed (`model.bedDaybed`). Notes: NOTES.md.
//!
//! Each bed's mattress is centred on its footprint, its top at the height the renderer lays sleepers at
//! (furniture.json `beds`; `bunks` for the bunk's two). Its head end stands on the wall's face (depth -0.93, `WALL`):
//! headboards, posts and rails stop there, thinner or a few cm forward, so the bed's bounds stand a little forward
//! of the middle (the Model's `centre`). The game lays its own quilt over each sleeper on the beds in
//! `BLANKET_BEDS`, so a bed's duvet isn't a second cover over them: its top third (`duvet_flap`) folds back over the
//! rest toward the foot while a resident sleeps or naps (`turn_down`), flattening as it goes so the quilt hides it,
//! and back up as they get up. The futon is a sofa at rest whose back folds down flat into the bed (`unfold`), held
//! open like the wall bed (content `leaves` / `when`); the daybed is static (a sofa by day). Sleepers lie centred,
//! head about 0.84 m behind the middle and feet 0.86 m in front: headboards keep their faces behind -0.87, pillows
//! and the duvet fit around that.
//!
//! Built in game coordinates (`F`: x, height, depth; front +depth).

use artkit::family::{Family, Model, motions};

mod beds;
mod bunks;
mod canopy;
mod common;
mod daybed;

use common::M;

const PAINT: [&str; 3] = ["Frame", "Headboard", "Bedding"];

fn paint(
    frame: &'static str,
    headboard: &'static str,
    bedding: &'static str,
) -> [(&'static str, &'static str); 3] {
    [
        (frame, PAINT[0]),
        (headboard, PAINT[1]),
        (bedding, PAINT[2]),
    ]
}

const TURN_DOWN: &str = r#"{"uses": {"sleep": "turn_down", "nap": "turn_down"}}"#;
// The bunk turns down the duvet of the bunk slept in: the lower one's for slot 0, the upper one's for slot 1, once
// its sleeper has climbed the ladder and crawled in (characters' `bunkUp`, 7 s: in on hands and knees by 4.9 s, sat
// up by 6), so it folds back as they turn to sit, not while they're at the ladder's foot.
const BUNK_TURN_DOWN: &str = r#"{"uses": {"sleep": [{"clip": "turn_down", "slot": 0}, {"clip": "turn_down_upper", "slot": 1, "delay": 5.0}], "nap": [{"clip": "turn_down", "slot": 0}, {"clip": "turn_down_upper", "slot": 1, "delay": 5.0}]}}"#;

/// The family.
pub fn family() -> Family {
    Family::new(
        "bedroom",
        M.palette(),
        vec![
            Model::new("bed", beds::bed, (1.9, 1.13, 1.878))
                .key("model.bed")
                .centre(0.0, 0.009)
                .motions(motions(TURN_DOWN))
                .paint(&paint("frame_linen", "headboard_linen", "bedding_cream")),
            Model::new("bedSingle", beds::bed_single, (0.96, 0.95, 1.9))
                .key("model.bedSingle")
                .centre(0.0, 0.02)
                .motions(motions(TURN_DOWN))
                .paint(&paint("frame_oak", "headboard_oak", "bedding_sage")),
            Model::new("bedBunk", bunks::bed_bunk, (0.96, 1.9, 1.89))
                .key("model.bedBunk")
                .centre(0.0, 0.015)
                .motions(motions(BUNK_TURN_DOWN))
                .paint(&paint("frame_oak", "headboard_oak", "bedding_teal")),
            // Down while the futon is open (the sim's `open`), from the moment a use that opens it starts.
            Model::new("bedFuton", bunks::bed_futon, (1.96, 0.995, 1.91))
                .key("model.bedFuton")
                .centre(0.0, 0.025)
                .motions(motions(r#"{"uses": {"sleep": "unfold", "nap": "unfold", "unfold": "unfold"}, "open": "unfold"}"#))
                .paint(&paint("frame_ash", "headboard_ash", "bedding_futon")),
            Model::new("bedKing", beds::bed_king, (1.98, 1.4, 1.9))
                .key("model.bedKing")
                .centre(0.0, 0.02)
                .motions(motions(TURN_DOWN))
                .paint(&paint("frame_linen", "headboard_linen", "bedding_cream")),
            Model::new("bedDouble", beds::bed_double, (1.62, 0.95, 1.888))
                .key("model.bedDouble")
                .centre(0.0, 0.014)
                .motions(motions(TURN_DOWN))
                .paint(&paint("frame_oak", "headboard_teak", "bedding_blue")),
            Model::new("bedQueenUpholstered", beds::bed_queen_upholstered, (1.76, 1.3, 1.89))
                .key("model.bedQueenUpholstered")
                .centre(0.0, 0.015)
                .motions(motions(TURN_DOWN))
                .paint(&paint("frame_stone", "headboard_stone", "bedding_oat")),
            Model::new("bedCanopy", canopy::bed_canopy, (1.73, 2.135, 1.895))
                .key("model.bedCanopy")
                .centre(0.0, 0.0175)
                .motions(motions(TURN_DOWN))
                .paint(&paint("frame_walnut", "headboard_walnut", "bedding_rust")),
            Model::new("bedDaybed", daybed::bed_daybed, (0.96, 0.84, 1.91))
                .key("model.bedDaybed")
                .centre(0.0, 0.025)
                .paint(&paint("frame_white", "back_white", "bedding_sage")),
        ],
        "The bedroom models (crates/art/rooms/src/bedroom, built by cargo art). Listed in the base manifest's \
         packs. The materials map onto the game's finishes so a model costs one draw call per finish group. The \
         beds turn their duvet down (`turn_down`) while a resident sleeps or naps; the futon unfolds into a bed \
         (`unfold`) and stays open.",
    )
}
