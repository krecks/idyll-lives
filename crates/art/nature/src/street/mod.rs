//! The street's dressing props (`model.street.*`): the kerbside bench, the planter with its little tree, the
//! flower tub, the street-name sign, the taxi rank's sign, the car park's P sign, the pot by a front door, the garden path light, and the
//! foliage of the street (`flowers`). Notes: NOTES.md.
//!
//! render/pc/street.ts draws them as thin instances along the kerbs and in the neighbours' front gardens, dozens
//! to hundreds of copies, and the town overview draws them again from afar: each is small (a few hundred to a
//! thousand triangles), takes one or two draw calls, and has a far version, its own build at low detail
//! (`far_low`). They are still; the leaves and flowers are on the `foliage` finish, so the nature shader sways
//! them in the wind as it did the old parts'. The path light's lens glows at night.
//!
//! Run:  cargo art street [--only bench] [--preview]

use artkit::family::{Family, Model};
use artkit::palette;
use artkit::place::GameFrame;

pub mod common;
mod props;
mod signs;

palette! { M:
    iron = ("#2B302E", metal), // the bench's cast-iron ends
    wood = ("#9A6B45", wood), // its slats
    concrete = ("#9A958C", matte), // the planter and the tub
    soil = ("#4A3526", matte),
    trunk = ("#6B4F3A", satin), // the planter's tree: plain, so the planter stays two draws
    stake = ("#8A6A48", satin), // its stakes, plain too
    tie = ("#2B2C2E", satin), // the rubber tie round the trunk and stakes
    leaf = ("#5E8F4E", foliage),
    leaf_light = ("#6E9C58", foliage),
    leaf_dark = ("#527F45", foliage),
    bed = ("#4E7432", foliage), // the flowers' leafy clumps
    bed_light = ("#5C8438", foliage),
    red = ("#D9483B", foliage), // flowers: on foliage, as the old parts', so they sway with the leaves
    yellow = ("#F2C14E", foliage),
    pink = ("#E9A6B4", foliage),
    rose = ("#E8678A", foliage),
    blue = ("#9FC5E8", foliage),
    violet = ("#B07AD8", foliage),
    white = ("#F2F0EA", foliage),
    eye = ("#F2B935", foliage), // the flowers' eyes
    eye_dark = ("#7A4E2A", foliage),
    pole = ("#5C6168", metal), // the signs' poles
    sign = ("#2F5D3A", gloss), // the street-name sign's green blades and lettering
    plate = ("#F2F2F0", gloss), // white panels
    ink = ("#3A3F45", matte), // the fares' lines
    // the taxi rank's sign: the taxi's yellow, its chequers and lettering, the light box lit at night (E8)
    taxi = ("#F2B81C", gloss),
    black = ("#16181A", matte),
    sign_light = ("#FFF4C8", satin, glow("#FFE7A0")),
    sign_blue = ("#1F4F9A", gloss), // the car park's P sign's panel
    terracotta = ("#B5653F", satin), // the door pot
    lamp = ("#2B2C2E", metal), // the path light's post and cap
    // the path light's frosted lens: glows at night (E8), where the old part's glass bulb was
    lens = ("#FFF1CF", satin, glow("#FFD9A0")),
}

const F: GameFrame = GameFrame::new(0.0);

/// The family.
pub fn family() -> Family {
    Family::new(
        "street",
        M.palette(),
        vec![
            Model::new("bench", props::bench, (1.7, 0.905, 0.514))
                .centre(0.0, -0.037)
                .far_low(),
            Model::new("planter", props::planter, (0.8, 2.445, 0.72)).far_low(),
            Model::new("flowerTub", props::flower_tub, (0.7, 0.57, 0.7)).far_low(),
            Model::new("sign", signs::sign, (0.9, 2.665, 0.9)).far_low(),
            Model::new("taxiSign", signs::taxi_sign, (0.5, 2.57, 0.112))
                .centre(0.0, 0.0141)
                .code()
                .far_low(),
            Model::new("parkingSign", signs::parking_sign, (0.46, 2.417, 0.15))
                .code()
                .far_low(),
            Model::new("pot", props::pot, (0.432, 0.88, 0.421)).far_low(),
            Model::new("pathLight", props::path_light, (0.18, 0.56, 0.18)).far_low(),
            Model::new("flowers", props::flowers, (0.6, 0.343, 0.52)).far_low(),
        ],
        "The street's dressing (crates/art/nature/src/street, built by cargo art): the kerbside bench, \
         the planter with its little tree, the flower tub, the street-name sign, the taxi rank's sign, \
         the car park's P sign, \
         the door pot, the garden path light and the flower clump that \
         render/pc/street.ts draws as thin instances along the kerbs and front gardens. Listed \
         in the base manifest's packs. Each has a far version for the town overview. Leaves and \
         flowers are on the foliage finish, so they sway in the wind; the path light's lens glows at \
         night.",
    )
}
