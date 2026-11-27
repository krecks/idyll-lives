//! The wall bars: see NOTES.md.

use super::common::{F, M, WALL};
use artkit::forms::smooth_outline;
use artkit::json::float_repr;
use artkit::parts::{join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, settle};

// --- the wall bars: 1x1 on a wall, beech rungs between two uprights, the top ones standing out -------------------
/// The rungs' depth at height `y`: against the wall low down, standing out over the top.
pub fn bars_z(y: f64) -> f64 {
    if y <= 2.0 {
        return WALL + 0.09;
    }
    WALL + 0.09 + (y - 2.0) / 0.5 * 0.12
}

pub fn wall_bars() -> Vec<Obj> {
    let parts = new_parts(|| {
        let up = smooth_outline(
            &[
                (WALL + 0.02, 0.0),
                (WALL + 0.16, 0.0),
                (WALL + 0.16, 2.0),
                (WALL + 0.28, 2.5),
                (WALL + 0.14, 2.5),
                (WALL + 0.02, 2.0),
            ],
            1,
            true,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fextrude!(
                F,
                format!("upright{s}"),
                &up,
                0.07,
                (sf * 0.43, 0, 0),
                M.oak,
                plane = Facing::Side,
                bevel = 0.008
            );
            for y in [0.12, 1.2, 2.3] {
                fbox!(
                    F,
                    format!("bracket{s}{}", float_repr(y)),
                    0.09,
                    0.08,
                    0.02,
                    (sf * 0.43, y, WALL + 0.01),
                    M.steel,
                    bevel = 0.003
                );
            }
        }
        for i in 0..16 {
            let y = 0.15 + i as f64 * 0.152;
            artkit::bevel::eased(fcyl!(
                F,
                format!("rung{i}"),
                0.018,
                0.82,
                (0, y, bars_z(y)),
                M.oak,
                rot = (0, 90, 0),
                verts = 12
            ));
        }
    });
    join(&parts, "bars", Some((0, 0, 0).into()));
    settle!(centre = false, floor = false);
    meshes()
}
