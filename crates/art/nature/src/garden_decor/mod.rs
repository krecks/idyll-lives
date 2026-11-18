//! The garden decor (`model.garden.gnome`, `birdbath`, `fountain`, `lantern`, `wheelbarrow`, `boulder`). Notes:
//! NOTES.md.
//!
//! Built at their real size, in Blender axes (front -Y), each stood on the floor and centred by `settle`; the
//! sizes are the old parts', within each model's tolerance. The fountain's water moves all the time near the
//! camera (`flow`, ambient: the jet bubbling up, rings on the basins); the rest are still.
//!
//! Run:  cargo art garden_decor [--only gnome] [--preview]

use artkit::family::{Family, Model};
use artkit::json;
use artkit::palette;
use artkit::place::GameFrame;

pub mod common;
mod garden;
mod stonework;

palette! { M:
    stone = ("#B9B2A6", matte),
    stone_dark = ("#8F887C", matte),
    stone_warm = ("#C6BCA9", matte),
    water = ("#6FA8C0", gloss), // opaque, like the old parts' water
    moss = ("#5E8A3C", foliage),
    leaf = ("#4E7A34", foliage),
    wood = ("#9A7444", wood),
    wood_dark = ("#6E5232", satin), // the gnome's boots and belt
    metal = ("#5A6068", metal),
    metal_dark = ("#2E3236", matte),
    paint = ("#3E7A6A", metal), // the wheelbarrow's green tray
    skin = ("#F0C6A4", satin),
    nose = ("#E8A08A", satin),
    beard = ("#EDE8DE", satin),
    eye = ("#2A2622", gloss), // the gnome's eyes
    buckle = ("#C9A24A", metal), // his belt buckle and buttons (brass)
    tunic = ("#3E63B0", satin),
    hat = ("#C8322A", satin),
    soil = ("#4A3826", matte),
    bird_brown = ("#7A5A40", satin),
    beak = ("#E8A23A", satin),
    bird_breast = ("#D9703B", satin), // the bird bath's robin
    flower_pink = ("#E85A9A", satin),
    flower_yellow = ("#F2C14E", satin),
    flower_white = ("#F4F0F2", satin),
    glow = ("#FFE6A8", satin), // the lantern's fire box; the content's `light` lights it at night
    rock = ("#7A756C", matte), // the boulders: weathered greys, darker than the dressed stone
    rock_dark = ("#605C55", matte),
    rock_warm = ("#8A8174", matte),
    lichen = ("#BDB894", matte),
}

/// For the kit's parts, which take game axes.
const F: GameFrame = GameFrame::new(0.0);

/// (width, height, depth): the old parts' bounds, and how far each model may be from them.
const TOLERANCE: f64 = 0.1;

/// The family.
pub fn family() -> Option<Family> {
    Some(
        Family::new(
            "garden",
            M.palette(),
            vec![
                Model::new("gnome", garden::gnome, (0.26, 0.58, 0.18))
                    .tolerance(TOLERANCE)
                    .paint(&[("hat", "Hat"), ("tunic", "Tunic"), ("wood_dark", "Belt")]),
                Model::new("birdbath", stonework::birdbath, (0.68, 0.935, 0.68))
                    .tolerance(TOLERANCE)
                    .paint(&[("stone", "Bowl & column"), ("stone_dark", "Plinth")]),
                Model::new("fountain", stonework::fountain, (0.92, 1.17, 0.92))
                    .tolerance(TOLERANCE)
                    .motions(json!({"idle": "flow"}))
                    .paint(&[("stone", "Fountain"), ("stone_dark", "Plinth")]),
                Model::new("lantern", stonework::lantern, (0.54, 1.035, 0.468))
                    .tolerance(TOLERANCE)
                    .paint(&[("stone", "Lantern"), ("stone_dark", "Base")]),
                Model::new("wheelbarrow", garden::wheelbarrow, (0.587, 0.754, 1.036))
                    .tolerance(TOLERANCE)
                    .paint(&[
                        ("paint", "Tray"),
                        ("wood", "Handles"),
                        ("metal_dark", "Legs & tyre"),
                    ]),
                Model::new("boulder", garden::boulder, (1.008, 0.471, 0.922))
                    .tolerance(TOLERANCE)
                    .paint(&[
                        ("rock", "Large boulder"),
                        ("rock_dark", "Middle boulder"),
                        ("rock_warm", "Small boulder"),
                    ]),
            ],
            "The garden decor (crates/art/nature/src/garden_decor, built by cargo art): a gnome, a bird \
             bath, a two-tier fountain, a stone lantern, a wheelbarrow planter and boulders. Listed in \
             the base manifest's packs. Each file's materials map by name onto the game's finishes \
             (`materials`, see render/pc/models.ts), so a model costs one draw call per finish group \
             and lights like the rest of the garden.",
        )
        .pack("gardenDecor")
        ,
    )
}
