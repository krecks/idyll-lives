//! The game's item models, generated: trait packs. One folder a family, each
//! listed in `families()`.

use artkit::family::Family;

pub mod pack_bookworm;
pub mod pack_cheerful;
pub mod pack_couch_potato;
pub mod pack_energetic;
pub mod pack_example;
pub mod pack_foodie;
pub mod pack_gloomy;
pub mod pack_hot_headed;
pub mod pack_kind;
pub mod pack_lazy;
pub mod pack_loner;
pub mod pack_nature_lover;
pub mod pack_neat;
pub mod pack_outgoing;
pub mod pack_romantic;
pub mod pack_slob;

/// The group's families by their names on the command line (`cargo art <name>`).
pub fn families() -> Vec<(&'static str, Option<Family>)> {
    vec![
        ("pack_bookworm", pack_bookworm::family()),
        ("pack_cheerful", pack_cheerful::family()),
        ("pack_couchPotato", pack_couch_potato::family()),
        ("pack_energetic", pack_energetic::family()),
        ("pack_foodie", pack_foodie::family()),
        ("pack_gloomy", pack_gloomy::family()),
        ("pack_hotHeaded", pack_hot_headed::family()),
        ("pack_kind", pack_kind::family()),
        ("pack_example", pack_example::family()),
        ("pack_lazy", pack_lazy::family()),
        ("pack_loner", pack_loner::family()),
        ("pack_natureLover", pack_nature_lover::family()),
        ("pack_neat", pack_neat::family()),
        ("pack_outgoing", pack_outgoing::family()),
        ("pack_romantic", pack_romantic::family()),
        ("pack_slob", pack_slob::family()),
    ]
}
