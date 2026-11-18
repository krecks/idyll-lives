//! The plants' colours (families/plants.py's `BASE`): every plant model's materials with their finishes; each model
//! takes its own colours (`palette`).

use crate::materials::Palette;

crate::palette! { BASE:
    body = ("#C0683F", satin),          // the pot (the bonsai's tray); the main part the player colours
    soil = ("#5A4030", matte),
    leaf = ("#4F8A3E", foliage),
    leaf_dark = ("#3B7034", foliage),
    leaf_light = ("#6FAA52", foliage),
    stem = ("#6E9248", foliage),
    bark = ("#8A7058", bark),
    wood = ("#7A5A40", wood),           // plant stands
    flesh = ("#4E8A4A", satin),         // cacti and succulents: still, no sway
    flesh_light = ("#7EA88A", satin),
    flesh_pink = ("#C99AA8", satin),
    bloom = ("#F6F0F4", satin),         // flowers
    bloom_accent = ("#C2367A", satin),
    sand = ("#C8B894", matte),
    moss = ("#5E7A34", matte),
    // a flower bed's soil mound: the part the player colours ("Bed"); dark earth (in a mid brown, #6A4C38, it was
    // a salmon disc in the game's midday sun)
    bed = ("#3C2C1E", matte),
    bloom2 = ("#F2C12E", satin),
    bloom3 = ("#EF6F9C", satin),
    bloom4 = ("#F08A24", satin),
    bloom5 = ("#F4F1EA", satin),
    centre = ("#F6C21E", satin),        // flower centres
    centre_dark = ("#3A2A1E", matte),
    blossom = ("#F8901C", foliage),     // flowers drawn leafy, as the old parts' (marigolds, lupins, forget-me-nots)
    blossom2 = ("#FCBA1E", foliage),
    blossom3 = ("#F4F0F2", foliage),
    blossom4 = ("#6C8CF0", foliage),
    stalk = ("#5E8C3A", satin),         // the meadow's picked flower's stem: one draw call with its bloom
    twig = ("#5C4936", foliage),        // a shrub's woody stems: on foliage, so they sway with the leaves in one draw
}

/// The family's palette with some colours changed, (name, #rrggbb) each (finishes stay the family's).
pub fn palette(colours: &[(&str, &str)]) -> Palette {
    BASE.palette().recoloured(colours)
}
