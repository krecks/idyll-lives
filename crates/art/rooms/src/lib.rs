//! The game's item models, generated: living and dining rooms, offices, bedrooms, decor and lighting. One folder a family; a family not yet ported has
//! none to give, and `cargo art` builds it in Blender from tools/art/blender/families/ as before.

use artkit::family::Family;

pub mod bedroom;
pub mod bedroom_dressing;
pub mod bedroom_storage;
pub mod decor;
pub mod dining;
pub mod lighting;
pub mod living;
pub mod office;

/// The group's families by their names on the command line (`cargo art <name>`), each once it is ported.
pub fn families() -> Vec<(&'static str, Option<Family>)> {
    vec![
        ("living", living::family()),
        ("dining", dining::family()),
        ("office", office::family()),
        ("decor", decor::family()),
        ("lighting", lighting::family()),
        ("bedroom", bedroom::family()),
        ("bedroom_storage", bedroom_storage::family()),
        ("bedroom_dressing", bedroom_dressing::family()),
    ]
}
