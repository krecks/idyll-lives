//! Bevelled edges, as Blender's bevel modifier rounds them the way gamekit set it (shapes.py `bevel_edges`: by
//! angle, an even offset, a round profile, hardened normals): the method is Blender's own (its bevel tool's,
//! bmesh_bevel.cc, GPL-2.0-or-later), worked through again here for the meshes the builders make: a box, a
//! cylinder or prism, an outline's slab, and those after a build has moved their points. Every edge between two
//! faces that turn more than the angle is cut back by the width on both faces and bridged in `segments` strips;
//! where three or more such edges meet, the corner is a polygon (one segment) or a patch (more).
//!
//! What a part's look hangs on is kept as Blender has it, in single precision and in its order:
//! - the width is cut back for the whole part where a face has no room for it (a box thinner than two widths, and
//!   a slab's face by the triangles it is cut into: `tri::fill`);
//! - a new point between two bevelled edges with one other edge between them slides along that edge, and the
//!   widths along a run of such points are then evened (least squares), so a slab's rim is not one width all round;
//! - a corner of three edges at right angles, all cut back alike, takes an eighth of a sphere; any other takes
//!   a patch subdivided from its sides' profiles;
//! - the new points come in the order the modifier leaves them (the baked shading twists its rays by a point's
//!   number), on a box, a cylinder and a slab: each corner's from the edge Blender starts it at, which hangs on
//!   how the mesh's edges are numbered (`EdgeOrder`).
//!
//! The normals are hardened: the faces the bevel joins stay flat, and each strip's corners take the normal of
//! the face they touch (or of the strips either side, between two), so the bevel shades as a rounded edge.
//! Blender stores each of those normals in the frame of its corner's fan on the mesh the bevel was working on (two
//! angles, 16 bits each) and the mesh it leaves reads them back in its own fans' frames, the mean of them where
//! a fan's corners were given unlike ones: so are they here, which is why a normal is now and then not the one
//! given (round a profile's middle point it leans to one side).
//!
//! A bevel is asked for (`bevel_edges`, a builder's `bevel`) and given later, as Blender's modifier is: the part
//! keeps its mesh, which a build can still push about, scale or cut, and is rounded as it is when it is joined
//! (`parts::join`), measured or written. A textured part's UVs are made on the mesh as it is then and carried
//! onto the bevel as Blender carries them (`carried`).
//!
//! What isn't built says so: an edge whose bevel ends at a vertex no other bevelled edge reaches, and a patch of
//! five segments or more.

use crate::math::{PI, V3, angle_normalized, cosf, dot_v3, len_v3, mul_v3_fl, normalize_v3, sinf};
use crate::mesh::{Mesh, edge, normal_newell, normal_tri};
use crate::scene::Obj;
use crate::shapes::{SHARP, shade};
use std::collections::{BTreeMap, BTreeSet};

const EPSILON: f32 = 1e-6;
const EPSILON_BIG: f32 = 1e-4;
/// Two edges within two degrees of one line are parallel.
const EPSILON_ANG: f32 = 0.034_906_585;
/// Under ten degrees between two edges, the face's own normal says which plane they meet in.
const SMALL_ANG: f32 = 0.174_532_92;
const GOOD_ANGLE: f32 = 0.1;
/// How far towards the corner a patch's middle stands, by its segments (a round profile).
const FULLNESS: [f32; 4] = [0.0, 0.559, 0.642, 0.551];

mod bevelled;
mod charts;
mod corner;
mod entry;
mod fans;
mod geometry;
mod harden;
mod offsets;
mod patch;
mod profile;
mod space;
mod types;
mod work;

pub use bevelled::*;
use charts::*;
use corner::*;
pub use entry::*;
use fans::*;
use geometry::*;
use harden::*;
use offsets::*;
pub use patch::*;
use profile::*;
use space::*;
pub use types::*;
use work::*;
