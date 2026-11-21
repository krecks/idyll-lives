//! `cargo check-glb --built <dir>`: the gate for the built world, the geometry the game makes as it runs (houses, the
//! neighbours' and the far houses, the lots' buildings closed, roads, sidewalks, junctions, bridges, fences, cars),
//! held to the rules the models are held to.
//!
//! The game builds all of it in TypeScript (web/src/render/shared), so there is no file to read as a model's is.
//! `bun tools/built/dump.ts <dir>` writes a fixed set of samples there, made by the game's own generators, a `.glb`
//! each and `built.json`, which lists them; this reads them as plain meshes and runs
//!
//! - `built.edges`: the models' hard-edge rule (`edges::hard_edges`). A house is not seen as a model is: its roof
//!   shows only from 15 m out, and nearer its walls stand, or those on the camera's side drop to stubs (the
//!   cutaway, by the way the camera looks). So a house is judged as it is drawn in each of those states, by the
//!   cameras that see it so (`house_edges`), and an edge counts once however many states show it;
//! - `built.texels`: the models' texel rule (`texels::measure`) over each part on a textured material, with the
//!   manifest's `uvScale` and the texture's size (a material projected from above takes its UVs from x and z);
//! - `built.farNear`: each far version against what it is the far version of (the neighbour's silhouette, the
//!   street's far house and the town view's box against the viewed house; a lot's building's box against it
//!   dressed): its plan, its height, its roof's kind and pitch, a chimney on both or neither, its walls' and its
//!   roof's colour (`Look`);
//!
//! and counts each sample's triangles and draw calls. Their findings are counted per sample against
//! tools/art/gate-baseline.json (`baseline`, sections by those names): at its count a sample's findings are printed,
//! over it or in a sample the file doesn't have they fail, under it they fail as stale until `--update-baseline`
//! lowers it. Far versions are seen from too far for an edge or a texel to show, and are judged by `built.farNear`
//! alone.
//!
//!   cargo check-glb --built <dir> [sample keys…]
//!   cargo check-glb --built <dir> --report            what it measures, as tables: where the edges are, each
//!                                                     material's texels, every pair's two looks
//!   cargo check-glb --built <dir> --update-baseline   first lowers the baseline to what it finds
//!   cargo check-glb --built <dir> --seed-baseline     first writes the sections the baseline has none of (the day
//!                                                     a check comes in)

use super::baseline::{self, Baseline, Counts, Found};
use super::digest::raw;
use super::glb::{Glb, R, image_size, item, key, parse_glb};
use super::py::{J, dirname, join, normpath};
use super::{Paths, asset_manifests};
use crate::edges::{self, Edge, Mesh, V3};
use crate::texels::{self, Texture, Textures};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

mod detail;
mod edge_gate;
mod far;
mod gate;
mod read;
mod report;
mod texel_gate;

pub(crate) use detail::*;
pub(crate) use edge_gate::*;
pub(crate) use far::*;
pub(crate) use gate::*;
pub use read::*;
pub(crate) use report::*;
use texel_gate::*;

#[cfg(test)]
mod tests;
