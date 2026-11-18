//! The game's item models, generated: hobbies, the children's things, what is held or stands on a table, media, fitness and the birds. One folder a family, each
//! listed in `families()`.

use artkit::family::Family;

pub mod birds;
mod common;
pub mod fitness;
pub mod held;
pub mod hobbies;
pub mod kids;
pub mod media;
pub mod tabletop;

/// The group's families by their names on the command line (`cargo art <name>`).
pub fn families() -> Vec<(&'static str, Option<Family>)> {
    vec![
        ("hobbies", hobbies::family()),
        ("kids", kids::family()),
        ("held", held::family()),
        ("tabletop", tabletop::family()),
        ("media", media::family()),
        ("fitness", fitness::family()),
        ("birds", birds::family()),
    ]
}
