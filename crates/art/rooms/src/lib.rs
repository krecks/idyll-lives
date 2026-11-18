//! The game's item models, generated: living and dining rooms, offices, bedrooms, decor and lighting. One folder a family, each
//! listed in `families()`.

use artkit::family::Family;

pub mod bedroom;
pub mod bedroom_dressing;
pub mod bedroom_storage;
pub mod decor;
pub mod dining;
pub mod lighting;
pub mod living;
pub mod office;

/// The group's families by their names on the command line (`cargo art <name>`).
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
