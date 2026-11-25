//! The home gym: the treadmill (`model.treadmill`), the weight bench (`model.weightBench`), the exercise bike
//! (`model.exerciseBike`), the yoga mat (`model.yogaMat`) and the dumbbell rack (`model.dumbbells`); and wellness at
//! home: the massage table (`model.massageTable`), the meditation cushion (`model.meditationCushion`) and the sauna
//! cabin for three (`model.sauna`). Notes: NOTES.md.
//!
//! Each is built where the game places it (`F`, game axes). The treadmill's belt slats run back under the runner
//! while it is used (`belt_run`, a loop: one slat on, home unseen under the hood and the rear cap). The bike's
//! cranks and pedals turn while it is cycled (`pedals_turn`, a loop). The yoga mat lies rolled up and unrolls flat
//! for yoga (`unroll`), rolling up again at the end. The lift draws the weight in the hands (content `held`), so
//! the bench's bar (`bar_off`) and a pair of the rack's dumbbells (`pair_off`) leave them. The sauna's glass door
//! (`door_open`) lets the bathers in, shuts behind them and opens to let them out. The three wellness models have
//! palettes of their own.

use artkit::family::{Family, Model};
use artkit::json;
use artkit::palette;

mod bike;
mod common;
mod mat;
mod sauna;
mod spa;
mod treadmill;
mod weights;

palette! { M:
    body = ("#D8D9D7", satin), // a machine's frame and housings; the main part the player colours
    frame = ("#5C6168", metal), // the bench's and the rack's steel
    pads = ("#3E6E73", satin), // vinyl pads, foam grips
    steel = ("#9EA3A8", metal), // the treadmill's uprights and handlebars
    dark = ("#3A3D41", satin), // feet, caps, the flywheel
    belt = ("#26282B", matte), // the treadmill's belt
    slat = ("#4A4E54", matte), // its slats
    screen = ("#0E2230", gloss), // a console's screen
    accent = ("#3C8DBF", satin), // a console's light strip, the hood's band
    red = ("#B33A32", satin), // buttons, J-hooks, the plates' inner rings
    chrome = ("#D0D3D6", chrome), // bars, handles
    plate = ("#2F3236", satin), // weight plates, dumbbell heads
    bottle = ("#4A90C2", gloss), // water bottles
    mat = ("#7FA38A", matte), // the yoga mat
    mat_edge = ("#5F836B", matte), // its rolled layers' edges and its alignment lines
    block = ("#E8E2D6", matte), // the yoga block
}

const FRAME_PADS: &[(&str, &str)] = &[("body", "Frame"), ("pads", "Pads")];

/// The family.
pub fn family() -> Family {
    Family::new(
        "fitness",
        M.palette(),
        vec![
            Model::new("treadmill", treadmill::treadmill, (0.794, 1.439, 1.85))
                .key("model.treadmill")
                .centre(0.032, -0.005)
                .motions(json!({"loops": {"run": "belt_run"}}))
                .paint(FRAME_PADS),
            Model::new("weightBench", weights::weight_bench, (0.9, 1.2, 1.849))
                .key("model.weightBench")
                .centre(0.0, -0.006)
                .motions(
                    json!({"uses": {"lift": {"clip": "bar_off", "delay": (weights::BAR_DELAY)}}}),
                )
                .paint(&[("frame", "Frame"), ("pads", "Pads")]),
            Model::new("exerciseBike", bike::exercise_bike, (0.53, 1.107, 0.9))
                .centre(0.0, 0.02)
                .key("model.exerciseBike")
                .motions(json!({"loops": {"cycle": "pedals_turn"}}))
                .paint(FRAME_PADS),
            // its own size, rolled up as it stands: the block and bottle beside it are drawn, not measured
            Model::new("yogaMat", mat::yoga_mat, (0.623, 0.132, 0.132))
                .key("model.yogaMat")
                .centre(0.025, mat::BACK + 0.135)
                .motions(json!({"uses": {"yoga": "unroll"}}))
                .paint(&[("mat", "Mat")]),
            Model::new("dumbbells", weights::dumbbells, (0.77, 0.717, 0.44))
                .key("model.dumbbells")
                .centre(0.0, 0.035)
                .motions(json!({"uses": {"lift": "pair_off"}}))
                .paint(&[("frame", "Rack"), ("plate", "Weights")]),
            // its own size, the table's: the trolley at the right back is drawn, not measured (the centre is of both)
            Model::new("massageTable", spa::massage_table, (0.7, 0.785, 1.845))
                .key("model.massageTable")
                .centre(-0.045, -0.0025)
                .palette(spa::table_palette())
                .paint(&[("body", "Upholstery"), ("oak", "Frame")]),
            Model::new(
                "meditationCushion",
                spa::meditation_cushion,
                (0.7, 0.184, 0.7),
            )
            .key("model.meditationCushion")
            .palette(spa::cushion_palette())
            .paint(&[("body", "Cushion"), ("zabuton", "Mat")]),
            Model::new("sauna", sauna::sauna, (1.94, 2.05, 1.925))
                .key("model.sauna")
                .centre(0.0, 0.0325)
                // the door lets the bathers in, shuts behind them and opens to let them out
                .motions(json!({"uses": {"sauna": {"clip": "door_open", "hold": 1.5, "out": 1.5}}}))
                .palette(sauna::palette())
                .paint(&[("body", "Wood"), ("trim", "Trim")]),
        ],
        "The home gym and wellness at home (crates/art/things/src/fitness, built by cargo art). Listed in the base \
         manifest's packs. The materials map onto the game's finishes so a model costs one draw call per finish \
         group. The treadmill runs its belt (`belt_run`) and the bike turns its pedals (`pedals_turn`) while used; \
         the yoga mat unrolls (`unroll`) for yoga; the sauna's glass door opens (`door_open`) to let the bathers in \
         and out.",
    )
}
