//! What a script does to a part's mesh by hand, with Blender's own tools: a modifier it adds and applies itself
//! (SOLIDIFY, WEIGHTED_NORMAL), a bmesh operator it runs (delete, bisect_plane, holes_fill, triangulate,
//! inset_individual, subdivide_edges). Each is Blender's, point for point in the same single-precision sums, so
//! what the script builds on it comes out the same. The cuts and their triangles are Blender's code
//! (bmesh_bisect_plane, bmo_subdivide, BLI_polyfill_2d_beautify, bmesh_polygon: GPL-2.0-or-later), worked through
//! again here.

mod bisect;
mod cut;
mod inset;
mod normal;
mod solidify;
mod subdivide;
mod triangulate;

pub use bisect::*;
use cut::*;
pub use inset::*;
pub use normal::*;
pub use solidify::*;
pub use subdivide::*;
pub use triangulate::*;
