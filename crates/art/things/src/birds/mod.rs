//! The garden birds the game flies about by code (`model.bird.*`): the blue tit, chaffinch, willow warbler,
//! blackbird and collared dove, at their real sizes and in their own plumage, perched (standing on a twig or the
//! lawn), each in five rigid parts the renderer moves (NOTES.md):
//!
//!   body     the torso with the wings folded on it and the legs; origin between the feet, on the perch
//!   head     the head, bill and eyes; origin at the neck joint
//!   tail     origin at its base, under the rump
//!   wing.L   the wings spread for flight, level, origin at the shoulder, reaching out to the bird's own left
//!   wing.R   (game -X, Blender +X) and right (game +X); the renderer flaps them about the body's forward axis
//!            and hides them (scale 0) while the bird is perched. At rest they are spread, at scale 1.
//!
//! No clips: the game moves the parts. A bird is a handful of pixels from the game's camera (6-95 m away, mostly
//! 15-30), and up to 40 fly about at once, so each is a few hundred triangles (at most half its size's target: 750
//! for the songbirds, more rings round for the blackbird and the dove): lofted rings coloured by plumage zone, every
//! material plain (matte feathers, satin bill and legs, a gloss eye): one draw call for the lot.

use artkit::family::{Family, Model};
use artkit::json;
use artkit::materials::Palette;
use artkit::palette;
use artkit::parts::{Settle, settle_with};
use artkit::py::round_to;
use artkit::scene::Obj;

mod bird;
mod common;
mod species;

use species::*;

// The plumage zones, one material each; every species colours them its own way (`palette`). Feathers are matte, the
// bill and legs satin, the eye gloss: all plain, so a bird is one draw call.
palette! { M:
    back = ("#8A9A5A", matte), // the mantle and the back of the body
    rump = ("#8A9A5A", matte), // the lower back over the tail (a chaffinch's olive-green)
    cap = ("#3F7FC8", matte), // the crown
    brow = ("#F2F2EE", matte), // over the eye (a tit's white band, a warbler's pale eyebrow)
    mask = ("#1E2430", matte), // the stripe through the eye (a chaffinch's black forehead)
    face = ("#F4F4F0", matte), // the cheek
    throat = ("#1E2430", matte), // chin and throat
    nape = ("#3F6FB0", matte), // the back of the head and neck (a tit's dark blue collar)
    collar = ("#1A1A1A", matte), // a dove's black half-collar
    breast = ("#F2D23A", matte),
    belly = ("#E8D050", matte),
    wing = ("#4A82C8", matte), // the coverts, folded and spread
    bar = ("#F2F2EE", matte), // wing bars and the chaffinch's shoulder patch
    flight = ("#4F6E9A", matte), // the flight feathers
    edge = ("#4F6E9A", matte), // the flight feathers' pale edges, folded (a chaffinch's yellowish panel)
    tip = ("#3E5578", matte), // the primaries' ends (a dove's dark wingtips)
    under = ("#C8CCC8", matte), // under the spread wing
    tail = ("#3F6FB0", matte),
    outer = ("#3F6FB0", matte), // the outer tail feathers (a chaffinch's white)
    tailtip = ("#EEEAE4", matte), // a dove's pale tail end
    undertail = ("#3F6FB0", matte), // under the tail (a dove's black base)
    bill = ("#2A2A2A", satin),
    eye = ("#121212", gloss),
    ring = ("#F2B828", satin), // a blackbird's eye ring
    leg = ("#6A7A90", satin),
}

/// The zones added later, and the zone each stands for where a species doesn't colour it.
const SAME: [(&str, &str); 4] = [
    ("rump", "back"),
    ("edge", "flight"),
    ("outer", "tail"),
    ("undertail", "tail"),
];

/// `palette`: the family's palette in one species' colours (the finishes stay the family's). A zone a species doesn't
/// colour takes the colour of the zone it stands for (`SAME`), so it never shows the family's placeholder.
fn palette(colours: &[(&'static str, &'static str)]) -> Palette {
    let mut colours = colours.to_vec();
    for (zone, like) in SAME {
        let has = |z: &str| colours.iter().any(|(k, _)| *k == z);
        if !has(zone)
            && let Some(&(_, hex)) = colours.iter().find(|(k, _)| *k == like)
        {
            colours.push((zone, hex));
        }
    }
    M.palette().recoloured(&colours)
}

/// `bird`: the build of a bird of `s` in its colours.
fn bird(s: &Species, colours: &[(&'static str, &'static str)]) -> Vec<Obj> {
    let objs = bird::Bird::new(s, palette(colours)).build();
    settle_with(
        &objs,
        Settle {
            centre: false,
            floor: false,
        },
    )
}

fn blue_tit() -> Vec<Obj> {
    bird(&BLUE_TIT, BLUE_TIT_COLOURS)
}

fn chaffinch() -> Vec<Obj> {
    bird(&CHAFFINCH, CHAFFINCH_COLOURS)
}

fn willow_warbler() -> Vec<Obj> {
    bird(&WARBLER, WARBLER_COLOURS)
}

fn blackbird() -> Vec<Obj> {
    bird(&BLACKBIRD, BLACKBIRD_COLOURS)
}

fn collared_dove() -> Vec<Obj> {
    bird(&COLLARED_DOVE, COLLARED_DOVE_COLOURS)
}

/// `facts`: the entry's `bird`: what the renderer needs of its size (`perchHeight`: the body's middle over the feet).
fn facts(s: &Species) -> json::Json {
    json!({"length": (s.length), "wingspan": (s.wingspan),
           "perchHeight": (round_to(bird::perch_height(s), 3)), "pitch": (s.pitch)})
}

/// The family. (wingspan, height, length over the ground: bill to tail tip as it sits) as measured; `centre` its
/// box's middle (the feet are on the origin, the tail reaches back further than the bill forward); `sink` how far
/// the tail hangs below the perch.
pub fn family() -> Family {
    Family::new(
        "birds",
        M.palette(),
        vec![
            Model::new("blueTit", blue_tit, (0.185, 0.072, 0.106))
                .centre(0.0, -0.0021)
                .key("model.bird.blueTit")
                .code()
                .on_wall()
                .sink(0.01)
                .bird(facts(&BLUE_TIT)),
            Model::new("chaffinch", chaffinch, (0.26, 0.072, 0.135))
                .centre(0.0, -0.0059)
                .key("model.bird.chaffinch")
                .code()
                .on_wall()
                .sink(0.01)
                .bird(facts(&CHAFFINCH)),
            Model::new("warbler", willow_warbler, (0.19, 0.054, 0.103))
                .centre(0.0, -0.0019)
                .key("model.bird.warbler")
                .code()
                .on_wall()
                .bird(facts(&WARBLER)),
            Model::new("blackbird", blackbird, (0.36, 0.11, 0.241))
                .centre(0.0, -0.0077)
                .key("model.bird.blackbird")
                .code()
                .on_wall()
                .bird(facts(&BLACKBIRD)),
            Model::new("collaredDove", collared_dove, (0.51, 0.11, 0.312))
                .centre(0.0, -0.0313)
                .key("model.bird.collaredDove")
                .code()
                .on_wall()
                .bird(facts(&COLLARED_DOVE)),
        ],
        "The garden birds (crates/art/things/src/birds, built by cargo art): the blue tit, chaffinch, willow \
             warbler, blackbird and collared dove the game flies about by code, perched, in five rigid parts the \
             renderer moves (`body`, `head`, `tail`, `wing.L`, `wing.R`: the wings spread, which it flaps, and hides \
             while perched). `bird`: length (bill to tail tip) and wingspan, the body's middle over the feet \
             (`perchHeight`) and the perched torso's rise from level (`pitch`, degrees), in metres.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use artkit::py;

    /// What the renderer (`render/shared/birds.ts`) reads of each bird: its five parts by name (`BIRD_PARTS`), its
    /// length bill tip to tail tip and the body's middle over the feet (`perchHeight`), and every zone coloured.
    #[test]
    fn every_bird_is_its_length_on_its_perch_height_in_five_named_parts() {
        let birds = [
            (&BLUE_TIT, BLUE_TIT_COLOURS, blue_tit as fn() -> Vec<Obj>),
            (&CHAFFINCH, CHAFFINCH_COLOURS, chaffinch),
            (&WARBLER, WARBLER_COLOURS, willow_warbler),
            (&BLACKBIRD, BLACKBIRD_COLOURS, blackbird),
            (&COLLARED_DOVE, COLLARED_DOVE_COLOURS, collared_dove),
        ];
        for (s, colours, build) in birds {
            let p = palette(colours);
            for (zone, like) in SAME {
                let own = colours.iter().find(|(z, _)| *z == zone);
                assert!(
                    own.is_some() || p.get(zone).colour == p.get(like).colour,
                    "{} {zone}",
                    s.name
                );
            }
            artkit::scene::reset();
            let parts = build();
            let names: Vec<String> = parts.iter().map(|o| o.name()).collect();
            assert_eq!(
                names,
                ["body", "head", "tail", "wing.L", "wing.R"],
                "{}",
                s.name
            );
            // the bill's tip is the head's foremost point (forward is Blender -y), the tail's tip its farthest from it
            let head = parts[1].world_points();
            let bill = *head.iter().min_by(|a, b| a.y.total_cmp(&b.y)).unwrap();
            let tail = parts[2].world_points();
            let long = tail
                .iter()
                .map(|&q| (q - bill).length())
                .fold(0.0, f64::max);
            assert!((long - s.length).abs() < 0.002, "{}: {long} m long", s.name);
            // the torso's middle (the body over the legs: the folded wings lie on top of it)
            let torso: Vec<f64> = parts[0]
                .world_points()
                .iter()
                .map(|q| q.z as f64)
                .filter(|&z| z > s.leg + 0.002)
                .collect();
            let (lo, hi) = (py::min_of(&torso), py::max_of(&torso));
            let mid = (lo + hi) / 2.0;
            assert!(
                (mid - bird::perch_height(s)).abs() < 0.005,
                "{}: middle {mid} m up",
                s.name
            );
        }
    }
}
