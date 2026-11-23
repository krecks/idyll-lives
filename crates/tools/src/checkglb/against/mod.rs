//! `cargo check-glb --against <rev|dir> [keys… | --family <f>] [--dir <built>] [--accept <file>]`: whether a
//! build of the models is the build it is held against: what a change to a family or the kernel moved. It reads
//! only `.glb` files, the packs and centres.json, and neither the generator that wrote the old files nor the one
//! that wrote the new.
//!
//!   cargo check-glb --against HEAD                           every model of the working tree against that revision
//!   cargo check-glb --against HEAD model.fridge model.oak    only these keys
//!   cargo check-glb --against HEAD --family trees            the models of models/trees/ or packs/trees/
//!   cargo check-glb --against main --dir target/art          the files under target/art in place of the working tree's
//!
//! The old side is a git revision (its files read with `git cat-file`), or, if git knows no such revision, a
//! directory; the new side is the working tree, or `--dir`. A directory is a checkout's root (it has
//! `web/public/assets`, and centres.json where the repo has it) or an assets root (`models/`, `packs/` and, if it
//! has one, `centres.json` beside them). One without a `manifest.json` is a build of some models only: what it
//! lacks is the working tree's, and without keys or a family only the models it has files for are compared.
//! The content and furniture.json, which the gate reads beside the models, are the working tree's on both sides.
//!
//! Each model, and its far version, is one of:
//! - `same`: the two files' digests are equal (`digest`: the same nodes, materials, triangles and clips to the
//!   bit, in whatever order) and so are its pack entry and its centre;
//! - `near`: every row of `METRICS` is within its limit (the constants below);
//! - `CHANGED`: a row is not. Each is printed with its part, its value, its limit and the worst point (in the
//!   part's own axes, which are glTF's: x is the game's -x), and the command exits 1.
//!
//! A row that is right as it is takes a line in an accept list, `--accept <file>`, `{"<key>": {"<metric>": "<why>"}}`
//! (a far version's key is `<key> far`, a manifest's its path under the assets), and is then printed as
//! `accepted`; there is none by default. As with the gate's `accept`, one that names no row, gives no reason or is not needed is an error.
//! But `decimated`, which says what a far version is (a decimation made it, which another build need not
//! repeat): where the rows fail it, it is held to that rule in their place (`METRICS`); where they pass, it passes.
/// A node's origin may be this far off (m): a tenth of a millimetre, a quarter of what f32 keeps at 3 m, so
/// only rounding passes (a picked fruit, a door's hinge and a hand's reach are fitted to origins).
pub const ORIGIN: f64 = 1e-4;
/// A node's turn at rest may be this far off (degrees): rounding, 0.2 mm at the end of a 1 m door.
pub const REST_TURN: f64 = 0.01;
/// A node's scale, at rest or in a clip, may be this far off: 1 mm on a part a metre long.
pub const SCALE: f64 = 0.001;
/// A material's base colour may be this far off per channel: one step of the 8 bits it is drawn with.
pub const COLOUR: f64 = 1.0 / 255.0;
/// A clip's first and last keys may be this far off (s): a sixteenth of a frame at 60 Hz, so no frame shows it.
pub const CLIP_LENGTH: f64 = 0.001;
/// The clips are compared where the game samples them (modelClips.ts): 60 times a second.
pub const CLIP_HZ: f64 = 60.0;
/// A part's place in a clip may be this far off at a sample (m), as its origin at rest.
pub const CLIP_MOVE: f64 = 1e-4;
/// A part's turn in a clip may be this far off at a sample (degrees): 2 mm at the end of a 1 m door, and what two
/// ways of joining the same keys (an arc, a straight line) differ by between keys 5 degrees apart.
pub const CLIP_TURN: f64 = 0.1;
/// A perch may be this far off (m). The birds' recorded day (web/tests/birds.test.ts) takes them as written, so
/// one that is off at all is said even where it passes.
pub const PERCH: f64 = 0.001;
/// A node's box, a model's size and its functional heights may be this far off (m): half a millimetre, an
/// eighth of a pixel at the play camera, and under the gate's own slack everywhere.
pub const BOUNDS: f64 = 0.0005;
/// A material may have this share fewer or more triangles: a bevel or a hole's fill cut another way, not a part
/// gone or doubled.
pub const TRIANGLES: (f64, f64) = (-0.20, 0.10);
/// Surface samples a square metre (one a square centimetre), and the fewest and the most for a part: enough to
/// find a face of a square centimetre, bounded so the largest tree takes a second.
pub const SAMPLES: (f64, usize, usize) = (1e4, 2000, 50000);
/// A part's surface may lie this far from the other build's on average (m): a whole part moved by a quarter of a
/// millimetre fails.
pub const SURFACE_MEAN: f64 = 0.0002;
/// No point of a part may be further than this from the other build's surface (m): half a pixel at the play
/// camera, room for a bevel's band cut another way.
pub const SURFACE_MAX: f64 = 0.002;
/// The same for a moving part and for fruit (m): a hand, a hinge or a lid is fitted to them
/// (web/src/render/shared/characters/clips/uses.ts `FRUIT`, `HARVEST_APPLE`).
pub const SURFACE_FITTED: f64 = 0.0005;
/// Materials that are fruit a resident reaches for (their names start so): held to `SURFACE_FITTED`.
pub const FRUIT: [&str; 3] = ["fruit", "apple", "lemon"];
/// A normal turned by more than this (degrees) shows as another shade; a bevel's band rounded another way stays under.
pub const NORMAL_TURN: f64 = 15.0;
/// The share of a part's surface whose normal may be turned that far: the bands along its edges.
pub const NORMAL_SHARE: f64 = 0.01;
/// A normal turned by more than this (degrees) faces the other way: a face inside out. None may.
pub const NORMAL_AWAY: f64 = 90.0;
/// The counterpart of a point may be this much further than the nearest point if it faces more as the point does
/// (m): of two faces back to back or meeting in an edge, the one meant.
pub const NORMAL_REACH: f64 = 0.0002;
/// Faces whose nearest points are within this of each other are one place (m): parts laid into each other,
/// whose texture and shading there are either's.
pub const SAME_PLACE: f64 = 1e-5;
/// A material's area may differ by this share: a face gone or doubled inside another shows here first.
pub const AREA: f64 = 0.01;
/// A part's texels a metre may differ by this share: the texture's scale is the finish's look.
pub const TEXELS: f64 = 0.02;
/// A texture coordinate may be this far off, measured on the surface (m): half a pixel at the play camera.
pub const UV_OFF: f64 = 0.002;
/// The share of a part's surface whose texture must be within `UV_OFF`: all but its seams and its bevels' bands.
/// Two Blender builds of one script differ on up to 3.9 % of a part (measured 2026-11-17, the porch swing's frame:
/// a face at 45° between two sides takes its texture from either), so 5 % may be off.
pub const UV_SHARE: f64 = 0.95;
/// The baked shading is rounded to this many greys (artkit's shade.rs `LEVELS`).
pub const SHADE_LEVELS: f64 = 32.0;
/// A part's shading may differ by this many greys on average. Two Blender builds of one script differ by up to
/// 1.36 (measured 2026-11-17, the pizza oven's cup: a vertex's rays hit or miss by a hair), so 2.
pub const SHADE_MEAN: f64 = 2.0;
/// 95 % of a part's surface must be within this many greys. Two Blender builds differ by up to 8.6 there (the
/// same cup), so 10.
pub const SHADE_MOST: f64 = 10.0;
/// A far version held to `decimated` may differ from the old one's size by this share a side: the gate's own
/// rule for a far version against its model is 5 %.
pub const DECIMATED_SIZE: f64 = 0.02;
/// Its lowest point may be this far from the full model's (m): the gate's own rule, a far version stands where
/// its model stands.
pub const DECIMATED_FLOOR: f64 = 0.005;
/// It may have this share more triangles than the old one: a far version is there to be cheap.
pub const DECIMATED_TRIANGLES: f64 = 0.10;
/// No point of it may be further than this from the old one's surface (m). Two Blender builds of one script
/// were up to 55 mm apart (measured 2026-11-17, the ping-pong table's far version), so 60 mm.
pub const DECIMATED_SURFACE: f64 = 0.06;
/// What is compared, by the name a finding and a line of the accept list have.
pub const METRICS: &[(&str, &str)] = &[
    ("file", "a file is missing or cannot be read"),
    (
        "nodes",
        "the nodes' names and parents, their origins, turns and scales at rest",
    ),
    (
        "materials",
        "the materials, their colours and the rest they say, each part's material and its attributes",
    ),
    ("draws", "the draw calls, as the gate counts them"),
    (
        "clips",
        "the clips' names and lengths, and each part's place, turn and size sampled at 60 Hz",
    ),
    ("pack", "the model's pack entry, or the rest of a manifest"),
    (
        "perches",
        "the perches: as many, in the same order, each within a millimetre",
    ),
    ("centre", "the model's centre in centres.json"),
    ("bounds", "each node's box"),
    ("size", "the model's size as the gate measures it"),
    (
        "heights",
        "its functional heights (seat, top, mattress, light)",
    ),
    (
        "gate",
        "the gate's errors and warnings for the key: none new",
    ),
    (
        "triangles",
        "the triangles of each material, and the budget's target",
    ),
    (
        "surface",
        "how far each part's surface is from the other build's, both ways",
    ),
    ("normals", "the normals at the nearest points"),
    ("area", "each material's area"),
    ("texels", "each part's texels a metre"),
    ("uv", "the texture coordinates at the nearest points"),
    ("shading", "the baked shading at the nearest points"),
    (
        "decimated",
        "a far version a decimation made, which another build need not repeat: where the rows above fail it is \
         held to its own rule in their place (the old size within 2 %, its lowest point within 0.5 cm of the \
         model's, at most a tenth more triangles, its surface within 6 cm of the old one's)",
    ),
];

mod compare;
mod differ;
mod findings;
mod main;
mod measures;

pub(crate) use compare::*;
pub(crate) use differ::*;
pub(crate) use findings::*;
pub(crate) use main::*;
pub(crate) use measures::*;

pub use compare::{compare, compare_decimated};
pub use findings::{Class, Compared};
pub use measures::{Options, Outcome, Row, Source, against};

#[cfg(test)]
mod tests;
