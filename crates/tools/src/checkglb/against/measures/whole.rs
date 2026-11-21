//! A whole model as one part.

use crate::checkglb::against::read;
use crate::checkglb::glb::{Glb, R};
use crate::checkglb::near::{self, Part};

/// A model at rest as one part: every node's triangles where the node stands.
pub(crate) fn whole(g: &Glb) -> R<Part> {
    let r = read(g)?;
    let worlds = near::worlds(&r.tree, &r.rests);
    let mut all = Part {
        node: 0,
        path: String::new(),
        material: String::new(),
        positions: Vec::new(),
        normals: Some(Vec::new()),
        uvs: None,
        greys: None,
        triangles: Vec::new(),
    };
    for p in r.parts {
        let world = worlds[p.node];
        let p = p.placed(&world);
        let base = all.positions.len() as u32;
        all.triangles
            .extend(p.triangles.iter().map(|t| t.map(|i| i + base)));
        match (all.normals.as_mut(), p.normals) {
            (Some(have), Some(more)) => have.extend(more),
            _ => all.normals = None,
        }
        all.positions.extend(p.positions);
    }
    Ok(all)
}
