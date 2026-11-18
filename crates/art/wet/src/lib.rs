//! The game's item models, generated: kitchens, bathrooms and the laundry. One folder a family, each
//! listed in `families()`.

use artkit::family::Family;

pub mod bathroom;
pub mod kitchen;
pub mod kitchen_cooking;
pub mod kitchen_counters;
pub mod laundry;

/// The group's families by their names on the command line (`cargo art <name>`).
pub fn families() -> Vec<(&'static str, Option<Family>)> {
    vec![
        ("kitchen", kitchen::family()),
        ("kitchen_cooking", kitchen_cooking::family()),
        ("kitchen_counters", kitchen_counters::family()),
        ("bathroom", bathroom::family()),
        ("laundry", laundry::family()),
    ]
}
