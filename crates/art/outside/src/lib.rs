//! The game's item models, generated: yards, playgrounds, farms, street furniture, stalls, shops and the named places'
//! fittings. One folder a family; a family not yet ported has none to give, and `cargo art` builds it in Blender from
//! tools/art/blender/families/ as before.

use artkit::family::Family;

pub mod civic;
pub mod civic_works;
pub mod farm;
pub mod outdoor;
pub mod playground;
pub mod shops;
pub mod stalls;
pub mod street_furniture;
pub mod yard;

/// The group's families by their names on the command line (`cargo art <name>`), each once it is ported.
pub fn families() -> Vec<(&'static str, Option<Family>)> {
    vec![
        ("yard", yard::family()),
        ("outdoor", outdoor::family()),
        ("playground", playground::family()),
        ("farm", farm::family()),
        ("street_furniture", street_furniture::family()),
        ("stalls", stalls::family()),
        ("shops", shops::family()),
        ("civic", civic::family()),
        ("civic_works", civic_works::family()),
    ]
}
