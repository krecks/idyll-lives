//! The playground (CM1): the slide (`model.slide`), the climbing frame (`model.climbingFrame`), the seesaw
//! (`model.seesaw`), the spring rider (`model.springRider`), the merry-go-round (`model.merryGoRound`), the
//! basketball hoop (`model.basketballHoop`), the ping-pong table (`model.pingPongTable`) and the park chess
//! table (`model.parkChessTable`). Notes: NOTES.md.
//!
//! Each is built where the game places it (`F`, game axes) and stood on the floor without moving it across
//! (`settle!(centre = false)`, `centre` on its `Model`), so the seats stay where the content puts its riders:
//! `seat` heights and forward offsets in web/public/content/community.json, slots spread across the width.
//! Town things, seen from far: every model has a far version.
//!
//! Moving parts:
//!   seesaw         the plank with its seats and handles rocks ±12° about the axle (`seesaw`, a loop; riders ride)
//!   springRider    the horse rocks ±10° on top of its spring (`rock`, a loop; the rider rides)
//!   merryGoRound   the platform turns once in 3.5 s (`spin`, a loop; the riders ride round with it)
//!   basketballHoop the net sways and stretches after each shot (`swish`, 1.4 s into the use, then every 3.2 s)
//!   parkChessTable a white pawn slides one square on (`move`)
//!

use artkit::family::{Family, Model, motions};

pub(crate) mod common;
mod courts;
mod frames;
mod rides;

use common::M;

/// The family.
pub fn family() -> Family {
    Family::new(
        "playground",
        M.palette(),
        vec![
            Model::new("slide", frames::slide, (0.75, 2.805, 1.934))
                .key("model.slide")
                .far_ratio(0.25)
                .centre(0.0, 0.0024)
                .paint(&[("body", "Frame"), ("chute", "Slide"), ("roof", "Roof")]),
            // its own size: the flag on the ridge is drawn with it, not measured (the frame is 2.81 m tall)
            Model::new("climbingFrame", frames::climbing_frame, (1.814, 2.808, 1.99))
                .key("model.climbingFrame")
                .far_ratio(0.25)
                .centre(0.0, 0.015)
                .paint(&[("timber", "Frame"), ("shingle", "Roof"), ("rope", "Net")]),
            Model::new("seesaw", rides::seesaw, (2.5, 0.846, 0.42))
                .key("model.seesaw")
                .far_ratio(0.25)
                .motions(motions(r#"{"loops": {"seesaw": "seesaw"}, "ride": "seesaw"}"#))
                .paint(&[("body", "Frame"), ("seat", "Seats"), ("plank", "Plank")]),
            Model::new("springRider", rides::spring_rider, (0.4, 0.949, 0.957))
                .key("model.springRider")
                .far_ratio(0.25)
                .centre(0.0, 0.0387)
                .motions(motions(r#"{"loops": {"ride": "rock"}, "ride": "rock"}"#))
                .paint(&[("horse", "Horse"), ("saddle", "Saddle"), ("spring", "Spring")]),
            Model::new("merryGoRound", rides::merry_go_round, (1.96, 1.04, 1.96))
                .key("model.merryGoRound")
                .far_ratio(0.25)
                .motions(motions(r#"{"loops": {"spin": "spin"}, "ride": "spin"}"#))
                .paint(&[("platform", "Platform"), ("rail", "Rails"), ("deck", "Stripes")]),
            Model::new("basketballHoop", courts::basketball_hoop, (1.2, 3.7, 0.972))
                .key("model.basketballHoop")
                .far_ratio(0.25)
                .centre(0.0, 0.016)
                // The net swishes after each shot: the first 1.4 s into the use, then every 3.2 s while it lasts.
                .motions(motions(
                    r#"{"uses": {"shoot": [{"clip": "swish", "delay": 1.4, "every": 3.2}, "ball_taken"]}}"#,
                ))
                .paint(&[("post", "Frame"), ("board", "Backboard"), ("rim", "Hoop")]),
            Model::new("pingPongTable", courts::ping_pong_table, (2.74, 0.925, 1.86))
                .key("model.pingPongTable")
                .far_ratio(0.25)
                .paint(&[("table_top", "Table"), ("concrete", "Legs")]),
            Model::new("parkChessTable", courts::park_chess_table, (1.44, 0.855, 0.7))
                .key("model.parkChessTable")
                .motions(motions(r#"{"uses": {"chess": "move"}}"#))
                .far(courts::park_chess_table_far)
                .paint(&[("stone", "Table"), ("stool", "Stools")]),
        ],
        "The playground (crates/art/outside/src/playground, built by cargo art): a slide, a climbing frame, a \
         seesaw, a spring rider, a merry-go-round, a basketball hoop, a ping-pong table and a park chess table. \
         Listed in the base manifest's packs. The materials map onto the game's finishes so a model costs one \
         draw call per finish group. The seesaw's plank, the spring rider's horse and the merry-go-round's \
         platform go round while played on, carrying their riders (`ride`); the hoop's net swishes after a shot \
         and a pawn moves on the chess table. Each has a far version for the town overview.",
    )
}

#[cfg(test)]
mod tests {
    use artkit::json::Json;

    /// The seesaw's seats are where the content's places put its riders.
    #[test]
    fn content_facts() {
        let o = crate::content_object("community.json", "seesaw");
        let Some(Json::Arr(places)) = o.get("places") else {
            panic!("seesaw: no places")
        };
        let xs: Vec<Option<f64>> = places
            .iter()
            .map(|p| match p.get("at") {
                Some(Json::Arr(at)) => at[0].num(),
                _ => None,
            })
            .collect();
        let x = super::rides::SEAT_X;
        assert_eq!(xs, [Some(-x), Some(x)], "seesaw: places moved");
    }
}
