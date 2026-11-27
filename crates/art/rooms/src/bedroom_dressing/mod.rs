//! The bedroom's dressing things (BR4): the cheval mirror (`model.mirror`), the dressing table
//! (`model.dressingTable`), the wall mirror (`model.mirrorWall`) and the standing mirror on a brass stand
//! (`model.mirrorFloor`). Notes: NOTES.md.
//!
//! Each mirror's glass is one flat piece of a mirror material (E6), so the game reflects the room in it live:
//! upright, never tilted (the game lays the reflection on the glass's box). The dressing table is one still body
//! plus its right-hand drawer, which slides out to the seated resident while they get ready. The wall mirror is on
//! the wall layer: built at its real height, its back on the wall face. Built to the old parts' sizes, in their
//! numbers: the parts are placed in game axes (`F`).

use artkit::family::{Family, Model, motions};

mod common;
mod dressing_table;
mod mirrors;

use common::M;

/// The family.
pub fn family() -> Family {
    Family::new(
        "bedroom_dressing",
        M.palette(),
        vec![
            Model::new("mirror", mirrors::mirror, (0.80, 1.856, 0.56))
                .key("model.mirror")
                .paint(&[("frame", "Frame")]),
            // its own size: the pouf it carries stands out in front of the kneehole (0.86 m is the table's own depth)
            Model::new(
                "dressingTable",
                dressing_table::dressing_table,
                (0.86, 1.533, 0.418),
            )
            .key("model.dressingTable")
            .centre(0.0, -0.003)
            // getReady (the `wash` tag) slides the right drawer out to the seat, and back after a while
            .motions(motions(&format!(
                r#"{{"uses": {{"getReady": "drawer_open"}}, "hold": {}}}"#,
                artkit::json::float_repr(dressing_table::DRAWER_HOLD)
            )))
            .paint(&[("table", "Table"), ("table_frame", "Frame")]),
            Model::new("mirrorWall", mirrors::mirror_wall, (0.62, 0.97, 0.09))
                .key("model.mirrorWall")
                .centre(0.0, -0.385)
                .on_wall()
                .paint(&[("wall_frame", "Frame")]),
            Model::new("mirrorFloor", mirrors::mirror_floor, (0.56, 1.78, 0.45))
                .key("model.mirrorFloor")
                .centre(0.0, -0.155)
                .paint(&[("stand", "Frame")]),
        ],
        "The bedroom dressing models (crates/art/rooms/src/bedroom_dressing, built by cargo art): the cheval \
         mirror, the dressing table, the wall mirror and the standing mirror. Listed in the base manifest's packs. \
         Each mirror's glass maps to a `mirror` material, so the game reflects the room in it live. The dressing \
         table slides its drawer out (`drawer_open`) while a resident gets ready.",
    )
}
