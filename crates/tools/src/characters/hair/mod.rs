//! The residents' hair, built from primitives over the head interface. Contract:
//!
//! - `build_hair(family, stage, head, joints)`: `hair.<style>` for every style but `none`, and
//!   `hair.beard` where the family has it; each pos, nor, idx (CCW out), dense, fields, tone (x:
//!   how loose it hangs, 0 on the head .. 1 at the tips of a long style's curtain, a tail or a
//!   braid, `Comp::sway`; y: shade deep in the hair, z: clump bands 0..1; w: the occlusion, baked
//!   later in build.rs). After each, its soft edges: `hair.<style>.wisps` (`Strip`, `wisps`).
//!
//! Every style starts from a skull cap: the scalp inside the hair's edge (the head interface's
//! hairline, smoothed round and brought a little lower over the forehead and temples, going round
//! each ear in an arc), cast onto the head's distance field, rounded part way out to the skull
//! ellipsoid (down to the skin at the edge) and pushed out by a thickness that varies over the head (volume on top, thin at the edge), in locks that leave the
//! crown in a whorl, with a parting and short tips past the hairline where the style has them. On
//! it go curtains (sheets with a thickness hanging from the scalp's widest, closed round with
//! rounded edges and hem, in locks with tips), fringes that follow the forehead as drawn (the
//! same), lofted locks over the face and scalp, bent capsules, braided strands, puffs, twisted
//! buns and tori. The hair is in closed shapes and locks so it breaks its outline and shades in
//! clumps (tone.z, light in a lock's middle). The beard is a shell over the jaw, chin and upper
//! lip with a hole round the lips, in locks; its morphs copy the lips' (face::follow_mouth). Each
//! style has one fixed layout, so the same calls give the same topology at every life stage.
//!
//! The solid hair ends in hard edges (a cap's rim, a hem, a lock's tip, a shell's outline). Past
//! them go wisps: thin quads growing from the edge's own vertices (`Strip`), in a part of their
//! own that is drawn only close up, on which the shader draws strands and cuts the gaps between them
//! away (fields: x = metres along the edge, y = 0 at the root .. 1 at the tip, z = how loose the
//! strands are, w = how far along they stay one; `character.ts` `RESIDENT_WISP_OPACITY`).
//!
//! Skinning: Head, blending below the jaw to neck_01 and spine_03 by height (tails and braids
//! along their length), so hair that falls past the jaw follows the shoulders when the head turns.
//! Nothing comes within 2.6 mm of the head, or 3 mm of the ears (`Ears`: the ear pieces' faces, ear.rs,
//! which the head's `sdf` leaves out): caps go round them in an arc (the afro's stands off them and closes
//! under itself above them), curtains go out over them, the beard stops in front.

use std::sync::Arc;

use super::contract::{BEARD_FAMILIES, HAIR_STYLES, wisps_part};
use super::head::{HeadFrame, head_frame};
use super::meshops::{nearest_point, ray_hit, smooth_normals};
use super::np::{
    PI, Periodic, TAU, clip, interp, linspace, linspace_open, mean_rows, pairwise_sum, py_mod,
    radians, smoothstep,
};
use super::prims::{caps, ellipsoid, grid, loft, noise3, orient, rings_mesh, spline, torus};
use super::rig::{BASE_STAGE, NB, b};
use super::{Cache, Mesh, V3, v3};
use crate::pcg64::Pcg64;

/// Nearest the hair comes to the skull (m); the check wants more than 2 mm.
const CLEAR: f64 = 0.0026;
/// How far the scalp the hair sits on is rounded out from the skull to its ellipsoid (0..1).
const ROUND: f64 = 0.5;
/// How far a cap's locks turn round the crown (rad): they leave it in a whorl.
const WHORL: f64 = 0.6;
use super::{ear, np, prims};

mod beard;
mod build;
mod cap;
mod curtain;
mod ears;
mod finish;
mod locks;
mod scalp;
mod tuft;

use beard::*;
pub(crate) use build::*;
pub(crate) use cap::*;
use curtain::*;
pub(crate) use ears::*;
use finish::*;
use locks::*;
pub(crate) use scalp::*;
use tuft::*;
