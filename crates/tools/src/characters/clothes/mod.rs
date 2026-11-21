//! The residents' clothes. They are painted on the body by its shader from the body's fields and
//! each garment's cut (`garments` in rig.json, see docs/characters-format.md); shells are added
//! only where the silhouette needs them:
//!
//! - `top.jacket`, `top.suit`: the body's own surface over the region the cut covers (torso and
//!   sleeves), cut exactly along the fields' iso-lines, lifted 8-10 mm (more towards the hem,
//!   which flares a little), open in a V at the front (the lapels), with a rim turned back to the
//!   skin;
//! - `top.polo`, `top.shirt`: a turned-down collar round the neckline;
//! - `top.long`: the body's own surface under the regular tops (tee, v-neck, polo, long top, shirt,
//!   tank top, blouse: one mesh), lifted off the skin and cut to the resident's sleeves, hem and
//!   neckline by the shader, as the trousers are;
//! - `bottom.trousers`: the body's own surface under the bottoms with legs (trousers, suit trousers,
//!   shorts, capri: one mesh), lifted off the skin and cut to the resident's waist and length by the
//!   shader; near the camera it is the cloth, and the body under it is not painted as cloth;
//! - `bottom.skirt`: an A-line loft from the waistband to above the knees, swinging with the thighs;
//! - `shoes.*`: an upper (the foot's skin under the collar, lifted off it: a toe box, a heel counter,
//!   a collar with its lining turned in to the skin, laces) on a sole that stands out round it.
//!
//! Seated, the jacket and the skirt lie on the lap rather than keep the flare they hang with: their
//! tone carries a seated corrective the runtime adds as the thighs turn forward (`seated_tone`).
//!
//! Contract: `garment_cuts(family, P)`: every id of `contract::garments`, in order, with its cut;
//! `build_clothes(family, stage, cuts)`: `top.jacket`, `lod.top.jacket`, ... (pos, nor, idx (CCW
//! out), dense, fields (hem distance, flag, category, 0), tone (the seated corrective, xyz)).
//!
//! Cut keys (metres on the base body's fields): top `sleeve` (sleeve parameter), `hem`, `crew` /
//! `scoop`, `over` (its hem is below the waistband: painted over the bottom); bottom `waist`,
//! `leg`, `skirt`; shoes `shoe` and `colour` (sRGB hex). Each shell is cut and laid out on the base
//! body, so it has the same vertices at every life stage (stages are deltas).

use std::sync::Arc;

use serde::Serialize;

use super::body::{NR, fields_of, region, seated_pose, surface, topology, tris};
use super::contract::garments;
use super::garments::{
    ARM_M, BOTTOM, FLAG_CUT, FLAG_MIDSOLE, FLAG_POLISHED, FLAG_SKIRT, FLAG_SOLE, Landmarks, NO_HEM,
    SHOES, TOP,
};
use super::head::head_frame;
use super::json::OMap;
use super::kdtree::KdTree;
use super::meshops::{Src, boundary_edges, clip_by_field, smooth_normals};
use super::np::{
    PI, clip, linspace, linspace_open, py_mod, py_round, radians, sign, smoothstep, unique,
};
use super::rig::{BASE_STAGE, NB, b, proportions};
use super::{Cache, Dense, M3, Mesh, V3, v3};

fn shoe_colour(name: &str) -> &'static str {
    match name {
        "sneakers" => "#EDEBE6",
        "trainers" => "#3D4B66",
        "boots" => "#5B3B28",
        _ => "#221C19",
    }
}
use super::{body, merge, np, rig};

mod apron;
mod build;
mod cut;
mod jacket;
mod mesh;
mod shoe;
mod skirt;
mod sole;
mod tops;
mod trousers;
mod upper;

use apron::*;
pub(crate) use build::*;
pub(crate) use cut::*;
use jacket::*;
use mesh::*;
use shoe::*;
use skirt::*;
use sole::*;
pub(crate) use tops::*;
pub(crate) use trousers::*;
use upper::*;

#[cfg(test)]
mod tests;
