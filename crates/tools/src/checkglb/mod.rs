//! `cargo check-glb [keys…] [--keys] [--preview] [--against <rev|dir>] [--built <dir>]`: the gate for the game's
//! glTF models (docs/design/objects.md, "Validation").
//!
//! It reads every `.glb` a model entry of the manifests uses (the base manifest and its packs, later ones
//! overriding) with that entry, checks what the game needs of it, and that every model the content draws
//! has a file. Errors exit 1; warnings are printed.
//!
//!   cargo check-glb                        every .glb the manifests use
//!   cargo check-glb model.garden.gnome …   only these keys (model., item., tool.)
//!   cargo check-glb --keys                 the keys a build may write (`known_keys`)
//!   cargo check-glb --preview [--sheet]    also draws each file as read (`preview`) to
//!                                          tools/art/previews/<family>/rt_<name>.png, its clips, perches and far
//!                                          version beside it (and its sheet)
//!   cargo check-glb --validate             also runs the Khronos glTF validator over each file and its far
//!                                          version (`validate`: Bun and web/'s `gltf-validator`)
//!   cargo check-glb --update-baseline      first lowers tools/art/gate-baseline.json to what the counted checks
//!                                          find (`COUNTED`; it only lowers counts and takes keys out)
//!   cargo check-glb --against <rev|dir>    not the gate: whether this build of the models is that one (`against`)
//!   cargo check-glb --built <dir>          the same rules over what the game builds as it runs, dumped there by
//!                                          tools/built/dump.ts (`built`)
//!
//! What it prints keeps one form: JSON objects are read in file order, numbers as Python
//! has them (an int stays an int), and formatted, `repr`'d and `math.hypot`'d as Python does (`py`); and
//! since, the functional heights' warnings (`heights`: seats, tops, mattresses and lamps against the model).
//! `measures` gives `cargo review` the same numbers.

pub mod against;
pub mod baseline;
pub mod built;
pub(crate) mod digest;
pub(crate) mod glb;
pub(crate) mod json;
pub mod meshes;
pub(crate) mod near;
pub mod preview;
pub(crate) mod py;
pub mod validate;

use baseline::{Baseline, Counts, Found, Verdict};
use glb::{Glb, R, accessor, first_value, image_size, index, item, key, parse_glb, values};
use py::{
    J, Num, dirname, dist, fixed, hypot, join, normpath, percent, relpath, repr, repr_str, signed,
    str_of,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::f64::consts::PI;

pub use py::repr_str as py_repr_str;

/// Triangles at full detail by the model's largest side (m): (up to, target, most). About the
/// primitive parts' median and 90th percentile in each class, so a model costs what the
/// primitive models cost today (docs/design/objects.md, "Budgets").
const TRIANGLES: [(f64, i64, i64); 4] = [
    (0.35, 1500, 3000),
    (1.0, 4000, 8000),
    (2.5, 8000, 16000),
    (f64::INFINITY, 12000, 24000),
];
/// Draw calls per model (each its own in the sun's shadow map too): aim for 1-3.
const DRAWS: (usize, usize) = (3, 4);
/// Extensions the game's loader can read (no Draco: its decoder isn't shipped).
const EXTENSIONS: [&str; 7] = [
    "KHR_materials_emissive_strength",
    "KHR_mesh_quantization",
    "KHR_texture_transform",
    "KHR_texture_basisu",
    "EXT_meshopt_compression",
    "KHR_materials_ior",
    "KHR_materials_specular",
];
/// Largest side (px) of a texture of a model's own by the model's largest side (m): (up to, most).
const TEXTURE_MAX: [(f64, u32); 2] = [(2.5, 512), (f64::INFINITY, 1024)];
/// The wall's face lies this far into a footprint's back tile (m): a wall-standing thing's back is there.
const WALL: f64 = 0.07;
/// A house's floor is drawn 1 cm up: a flat part on the floor must reach this high (m) to show over it.
const FLOOR_TOP: f64 = 0.012;
/// A use's clip longer than this (s) reads as slow ("about 1 s"); loops and ambient clips keep their own pace.
const USE_CLIP: f64 = 3.0;
/// The game samples a clip 60 times a second and blends the samples' matrices (modelClips.ts `clipPose`), which
/// shrinks a part turning fast between them (1 - cos of half the turn): degrees a part may turn in 1/60 s (12°: two
/// turns a second, 0.5 %).
const TURN_STEP: f64 = 12.5;
/// A clip whose first key is later than this (s) starts late: the game holds it at rest until then.
const CLIP_START: f64 = 1e-4;
/// Faces smaller than this (m²) aren't checked for normals turned against their winding.
const FLIPPED_AREA: f64 = 1e-4;
/// How far (m) a model may reach past its footprint at rest (its parts' boxes: a handle, a knob), and how far a
/// part that goes round may sweep past it (measured from its vertices, so tighter: a fan's blades whose axis
/// stood 3.3 cm off the middle swept 1.97 cm past).
const OVERHANG: f64 = 0.02;

mod birds;
mod clips;
mod files;
mod furniture;
mod main;
mod measure;
mod mesh;
mod nodes;
mod paths;
mod pose;
mod report;
mod strict;
#[cfg(test)]
mod tests;

pub(crate) use birds::*;
pub(crate) use clips::*;
pub use files::*;
pub use furniture::*;
pub use main::*;
pub use measure::*;
pub(crate) use mesh::*;
pub(crate) use nodes::*;
pub use paths::*;
pub use pose::*;
pub use report::*;
pub use strict::*;
