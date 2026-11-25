//! The residents' bodies: a control cage of superellipse rings (cage.rs) fitted so its limit
//! surface passes through the ring points, subdivided once (subdiv.rs, about 5k triangles) for
//! `body`; the cage at its limit (about 1.1k triangles) is `lod.body`. Contract:
//!
//! - `build_body(family, stage)`: parts `body` and `lod.body` (pos, nor, idx (CCW out), dense,
//!   arm), meta (`seatDrop`, `eyeY`, `grip`) and the joints;
//! - `body_fields_and_tone(family, mesh, P, head)`: fields (n, 4) and tone (n, 4), base stage only.
//!
//! Proportions come from rig.rs (`proportions`: height, head size, hips, shoulders, limb lengths
//! and girth, waist and belly) plus the shapes below, per family. Every stage of both families
//! builds the same cage, so the topology is fixed and stages are deltas.
//!
//! Skin weights are set per ring (a ring's bone, blended 50/50 or so with the neighbour across a
//! joint; the shoulders blend spine_03, clavicle and upperarm, the hips pelvis and thigh, the neck
//! spine_03, neck_01 and Head) and subdivide with the positions, so they fall off smoothly.
//!
//! Fields (docs/characters-format.md): x = sleeve parameter (-1 torso, 0 shoulder .. 1 wrist, over
//! 1 on the hand: the arm's share of a vertex blends from the torso's -1 at the armhole), y = height,
//! z / w = the crew and scoop (male: V) necklines, metres below a smooth curve round the neck
//! (positive: covered); on the head -1 - scalp weight (the ears: -1). Tone: x = blush (cheeks, nose, ears), z =
//! the face's age folds (`HeadFrame::fold`); the shade in the creases is baked into w afterwards
//! (ao.rs).
//!
//! The head's cage is finer over the face (head.rs) than the LOD's: the LOD keeps the coarse grid
//! (`Cage::lod_faces`), so `lod.body` has the face's extra cage vertices unused. On the head the drawn
//! surface is snapped onto the smooth head between the cage's vertices (`Topology::snapped`), and the
//! eye sockets are carved into every point of it (`head::sockets`); the lids cover them, so the skin is
//! shaded as it is without them. The ears are pieces set into the head (ear.rs), after the skin in the
//! `body` part as the lids and lips are; no vertex of the cage is an ear's (the "ear" region is theirs).

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use nalgebra::{DMatrix, DVector, Dyn, LU};

use super::cage::{Cage, Per, RingTags, Tags, Weights, se, turn, unit};
use super::head::{HeadFrame, head_cage, onto_head, sockets, youth};
use super::identity::Identity;
use super::meshops::smooth_normals;
use super::np::{clip, lstsq, mean, mean_rows, radians, sign, smoothstep, unique};
use super::rig::{BASE_STAGE, NB, b, joints, maturity, proportions};
use super::sparse::Csr;
use super::subdiv::{
    Subdivision, check_closed, limit_matrix, orient, polygon_triangles, subdivision, triangles,
};
use super::{Cache, Dense, M3, Mesh, V3, key, v3};
use super::{body, ear, prims, rig};

mod build;
mod cage;
mod cage_of;
mod fields;
mod pose;
mod regions;
mod surface;
mod topology;

pub use build::*;
pub use cage::*;
use cage_of::*;
pub use fields::*;
pub use pose::*;
pub use regions::*;
pub use surface::*;
pub use topology::*;
