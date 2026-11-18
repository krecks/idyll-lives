//! What Blender's `mesh.validate()` takes out of a mesh and mends in it (BKE_mesh_validate, Blender 5.2), for the
//! meshes a build can make. Blender's glTF exporter calls it on every mesh before it writes it, so a file never
//! has what it takes out: gamekit's export made the UVs and baked the shading first, on the mesh as it was built,
//! and what is left keeps its shading, UVs, normals, sharp edges, smooth flags and materials as they were.
//!
//! What goes, in Blender's order: a face of fewer than three corners; a face with a vertex twice among its
//! corners; a face on the corners of an earlier one, the same way round or the other, from whichever corner
//! (`cut_leaf`'s top and underside are one triangle twice wherever a piece of the outline touches no midrib point).
//! Two faces on the same points joined by other edges (a quad and its bow tie) are both kept. The first of two stays,
//! whatever their materials. What is mended: a point with a coordinate that is no number, or endless, is put at
//! the origin, and a corner's UV likewise at zero.
//!
//! Left alone, as Blender leaves them: two points in one place and the faces of no area on them, points and edges
//! no face uses (the file has only what its triangles use). The scripts' own `mesh.validate()` after
//! `from_pydata` is the same call; the kernel's builders don't make it, and a mesh they give doubled faces loses
//! them here, when it is written.

use crate::mesh::Mesh;
use crate::scene::Obj;
use std::collections::BTreeSet;

/// A face's corners as Blender's validate orders them to tell doubles: from its lowest vertex, towards the lower
/// of that one's two neighbours. Two faces with the same edges give the same, whichever way they are wound.
fn ordered(corners: &[u32]) -> Vec<u32> {
    let n = corners.len();
    let low = (1..n).fold(0, |low, i| if corners[i] < corners[low] { i } else { low });
    let forward = corners[(low + 1) % n] < corners[(low + n - 1) % n];
    (0..n)
        .map(|i| corners[if forward { low + i } else { low + n - i } % n])
        .collect()
}

/// `mesh.validate()`: whether it changed the mesh (Blender's returns the same).
pub fn validate(mesh: &mut Mesh) -> bool {
    let mut seen: BTreeSet<Vec<u32>> = BTreeSet::new();
    let keep: Vec<bool> = (0..mesh.faces.len())
        .map(|f| {
            let corners = mesh.corners(f);
            let mut distinct = corners.to_vec();
            distinct.sort_unstable();
            distinct.dedup();
            corners.len() >= 3 && distinct.len() == corners.len() && seen.insert(ordered(corners))
        })
        .collect();
    let mut changed = keep.contains(&false);
    if changed {
        // the corners of the faces that stay, in order, with what each carries
        let old: Vec<usize> = (0..mesh.faces.len())
            .filter(|&f| keep[f])
            .flat_map(|f| {
                let face = mesh.faces[f];
                face.start as usize..(face.start + face.len) as usize
            })
            .collect();
        mesh.loops = old.iter().map(|&l| mesh.loops[l]).collect();
        if let Some(uvs) = &mut mesh.uvs {
            *uvs = old.iter().map(|&l| uvs[l]).collect();
        }
        if let Some(normals) = &mut mesh.normals {
            *normals = old.iter().map(|&l| normals[l]).collect();
        }
        if let Some(stored) = &mut mesh.stored {
            *stored = old.iter().map(|&l| stored[l]).collect();
        }
        let mut kept = keep.iter();
        mesh.faces.retain(|_| *kept.next().unwrap());
        let mut start = 0;
        for face in &mut mesh.faces {
            face.start = start;
            start += face.len;
        }
    }
    for v in &mut mesh.verts {
        if !v.array().iter().all(|c| c.is_finite()) {
            *v = crate::math::V3::ZERO;
            changed = true;
        }
    }
    for uv in mesh.uvs.iter_mut().flatten() {
        if !uv.iter().all(|c| c.is_finite()) {
            *uv = [0.0, 0.0];
            changed = true;
        }
    }
    changed
}

/// Validates each of `objs`' meshes in the scene, as Blender's exporter does before it writes them (the scene
/// keeps them so: a far version decimated from the model afterwards starts from these).
pub fn meshes(objs: &[Obj]) {
    for &o in objs {
        o.edit(|part| validate(&mut part.mesh));
    }
}
