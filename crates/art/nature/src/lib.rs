//! The game's item models, generated: trees, plants, the garden, the ground and the street. One folder a family, each
//! listed in `families()`.

use artkit::family::Family;

pub mod garden_decor;
pub mod garden_things;
pub mod ground;
pub mod plants;
pub mod street;
pub mod trees;

/// The group's families by their names on the command line (`cargo art <name>`).
pub fn families() -> Vec<(&'static str, Option<Family>)> {
    vec![
        ("trees", trees::family()),
        ("plants", plants::family()),
        ("garden_things", garden_things::family()),
        ("garden_decor", garden_decor::family()),
        ("ground", ground::family()),
        ("street", street::family()),
    ]
}
