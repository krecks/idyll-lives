//! What a part asks for: a pending bevel, and applying it to an object.

use super::*;

/// A bevel a part is to be given: Blender's modifier, which rounds the mesh as it is when the part is joined,
/// measured or written, not as it was when the bevel was asked for.
#[derive(Clone, Debug)]
pub struct Pending {
    pub width: f64,
    pub segments: usize,
    pub angle: f64,
    pub order: EdgeOrder,
    /// A slab's side faces, from this one on: Blender numbers them by where its memory put their edges, an
    /// order that can't be told from here (`Charts::groups` reads it for the textures).
    pub sides_from: Option<usize>,
}

/// Rounds `obj`'s edges turning more than `angle` degrees by `width` (metres), in `segments` strips, shaded
/// smooth with hardened normals, so the faces it joins stay flat and the bevel reads as a rounded edge, not a
/// facet. As Blender's modifier, it is only asked for here: the part keeps its mesh, which a build can still move,
/// scale or cut, and is rounded when it is joined (`parts::join`), measured or written (`apply`).
pub fn bevel_edges(obj: Obj, width: f64, segments: usize, angle: f64, order: &EdgeOrder) -> Obj {
    if obj.read(|o| o.mesh.faces.first().is_none_or(|f| !f.smooth)) {
        shade(obj, true, SHARP);
    }
    // (a second bevel rounds what the first made)
    apply(obj);
    obj.edit(|o| {
        o.bevel = Some(Pending {
            width,
            segments,
            angle,
            order: order.clone(),
            sides_from: None,
        })
    });
    obj
}

/// The mesh of `obj` with the bevel it was to be given (`None`: it was to be given none), and what each of its
/// faces came from.
pub(super) fn rounded(o: &crate::scene::Object) -> Option<Bevelled> {
    let p = o.bevel.as_ref()?;
    match bevelled(&o.mesh, p.width, p.segments, p.angle, &p.order) {
        Ok(made) => Some(made),
        Err(why) => panic!("{}: {why}", o.name),
    }
}

/// The points of `obj` as it will be written: with its bevel, if it is still to be given one.
pub fn points(obj: Obj) -> Vec<V3> {
    obj.read(|o| rounded(o).map_or_else(|| o.mesh.verts.clone(), |made| made.mesh.verts))
}

/// Gives `obj` the bevel it was to be given, on its mesh as it is now (once: nothing is left to give after).
pub fn apply(obj: Obj) -> Obj {
    if obj.read(|o| o.bevel.is_none()) {
        return obj;
    }
    let textured = crate::uv::textured(obj);
    obj.edit(|o| {
        // (a textured part's UVs are made on the mesh as it was, as the modifier carried them)
        if textured && !o.mesh.uv_metres {
            match o.uv {
                crate::mesh::UvWay::Box => crate::uv::box_(&mut o.mesh),
                crate::mesh::UvWay::Round => crate::uv::round(&mut o.mesh),
                crate::mesh::UvWay::Done => {}
            }
        }
        let Some(made) = rounded(o) else { return };
        let mut made = made;
        if let Some(uvs) = &o.mesh.uvs {
            made.mesh.uvs = Some(if textured {
                let sides_from = o.bevel.as_ref().and_then(|b| b.sides_from);
                carried(&o.mesh, uvs, &made.mesh, &made, sides_from)
            } else {
                // (a part on plain finishes: its UVs are zero, or will be before they are read)
                vec![[0.0, 0.0]; made.mesh.loops.len()]
            });
            // the mesh is still the one whose UVs were made: joined first, it says so for the whole
            made.mesh.uv_metres = o.mesh.uv_metres;
        }
        o.mesh = made.mesh;
        o.bevel = None;
    });
    obj
}

/// Gives each of `objs` (parts about to be joined, or a model's as written) its UVs and then its bevel, as Blender
/// applies a part's modifiers.
pub fn apply_all(objs: &[Obj]) {
    crate::uv::ensure(objs);
    for &o in objs {
        apply(o);
    }
}
