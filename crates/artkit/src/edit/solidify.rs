//! Making a surface a shell, and taking vertices out.

use super::*;
use crate::math::{V3, angle_normalized, dot_v3, normalize_v3};
use crate::mesh::{Mesh, edge};
use crate::normals;
use crate::scene::Obj;
use crate::shapes::options;
use std::collections::BTreeMap;

options! {
    /// `solidify`'s options, Blender's own defaults: `offset` -1 grows the shell inwards (against the normals), 1
    /// outwards, 0 half each way; `use_even_offset` keeps the shell as thick where the surface bends as where it is
    /// flat.
    Solidify { offset: f64 = -1.0, use_even_offset: bool = false }
}

/// Blender's SOLIDIFY modifier (its simple mode), applied: `obj`'s surface made a shell `thickness` thick: the
/// surface, a copy of it facing the other way, each moved along the vertices' normals as `offset` says, and a
/// wall round the open edges between them. The points are the surface's, then the copy's in the same order.
pub fn solidify(obj: Obj, thickness: f64, o: Solidify) -> Obj {
    obj.edit(|part| part.mesh = solidified(&part.mesh, thickness as f32, &o));
    obj
}

/// `edit::solidify` with its options by name, as the family scripts set them on the modifier:
/// `solidify!(obj, thickness, offset = 0.0, use_even_offset = true)`.
#[macro_export]
macro_rules! solidify {
    ($obj:expr, $thickness:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $thickness);
        #[allow(unused_mut)]
        let mut o = $crate::edit::Solidify::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::edit::solidify(args.0, $crate::forms::num(args.1), o)
    }};
}

fn solidified(mesh: &Mesh, thickness: f32, o: &Solidify) -> Mesh {
    let n = mesh.verts.len();
    let offset_fac = o.offset as f32;
    let ofs_orig = -(((-offset_fac + 1.0) * 0.5) * thickness);
    let ofs_new = thickness + ofs_orig;
    // (the vertices' true normals: a mesh's own corner normals are not read)
    let vert_nors = match mesh.normals {
        Some(_) => {
            let mut plain = mesh.clone();
            plain.normals = None;
            normals::vertex_normals(&plain)
        }
        None => normals::vertex_normals(mesh),
    };
    let mut out = Mesh {
        verts: mesh.verts.iter().chain(&mesh.verts).copied().collect(),
        mats: mesh.mats.clone(),
        ..Mesh::default()
    };
    // how far along its normal each vertex goes, as a share of the offset
    let share: Vec<Option<f32>> = if o.use_even_offset {
        let face_normals = mesh.face_normals();
        let (mut angles, mut accum) = (vec![0.0_f32; n], vec![0.0_f32; n]);
        for (f, &face_normal) in face_normals.iter().enumerate() {
            let c = mesh.corners(f);
            let at = |i: usize| mesh.verts[c[i] as usize];
            let mut curr = c.len() - 1;
            let mut nor_prev = normalize_v3(at(curr - 1) - at(curr)).0;
            for next in 0..c.len() {
                let nor_next = normalize_v3(at(curr) - at(next)).0;
                let angle = angle_normalized(nor_prev, nor_next).max(f32::EPSILON);
                let v = c[curr] as usize;
                accum[v] += angle;
                // (a shell is as much thicker along the vertex's normal as that leans from the face's)
                let cos = dot_v3(vert_nors[v], face_normal).abs();
                angles[v] += if cos < 1.0e-8 { 1.0 } else { 1.0 / cos } * angle;
                nor_prev = nor_next;
                curr = next;
            }
        }
        (0..n)
            .map(|v| (accum[v] != 0.0).then(|| angles[v] / accum[v]))
            .collect()
    } else {
        vec![None; n]
    };
    // the surface's own vertices take the offset that lies further along the normals, the copy's the other
    let (new_at, orig_at) = if ofs_new >= ofs_orig { (0, n) } else { (n, 0) };
    for (ofs, first) in [(ofs_new, new_at), (ofs_orig, orig_at)] {
        if ofs == 0.0 {
            continue;
        }
        for v in 0..n {
            let by = match (o.use_even_offset, share[v]) {
                (true, Some(k)) => ofs * k,
                (true, None) => continue,
                (false, _) => ofs,
            };
            let (p, nor) = (&mut out.verts[first + v], vert_nors[v]);
            p.x += nor.x * by;
            p.y += nor.y * by;
            p.z += nor.z * by;
        }
    }
    let shift = n as u32;
    for (f, face) in mesh.faces.iter().enumerate() {
        out.push_face(mesh.corners(f), face.mat, face.smooth);
    }
    // the copy wound the other way from the corner it started at, so its diagonals match the surface's
    for (f, face) in mesh.faces.iter().enumerate() {
        let c = mesh.corners(f);
        let mut turned = vec![c[0] + shift];
        turned.extend(c[1..].iter().rev().map(|&v| v + shift));
        out.push_face(&turned, face.mat, face.smooth);
    }
    // an edge one face runs along is open: a wall from it to its copy, wound against that face
    let mut users: BTreeMap<(u32, u32), Option<OpenEdge>> = BTreeMap::new();
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        let mut prev = c[c.len() - 1];
        for &v in c {
            users
                .entry(edge(prev, v))
                .and_modify(|u| *u = None)
                .or_insert(Some((f, prev, v)));
            prev = v;
        }
    }
    for (f, prev, v) in users.into_values().flatten() {
        let face = mesh.faces[f];
        out.push_face(&[v, prev, prev + shift, v + shift], face.mat, face.smooth);
    }
    out.sharp = mesh
        .sharp
        .iter()
        .flat_map(|&(a, b)| [(a, b), (a + shift, b + shift)])
        .collect();
    out
}

/// `bmesh.ops.delete(bm, geom=[the vertices `gone` says], context='VERTS')` on `obj`'s mesh: those vertices and
/// every face that has one for a corner are taken out; the others keep their order.
pub fn delete_verts(obj: Obj, gone: impl Fn(V3) -> bool) -> Obj {
    obj.edit(|part| {
        let mesh = &part.mesh;
        let mut kept = vec![u32::MAX; mesh.verts.len()];
        let mut out = Mesh {
            mats: mesh.mats.clone(),
            uv_metres: mesh.uv_metres,
            ..Mesh::default()
        };
        for (i, &v) in mesh.verts.iter().enumerate() {
            if !gone(v) {
                kept[i] = out.verts.len() as u32;
                out.verts.push(v);
            }
        }
        let per_vert = |values: &Option<Vec<f32>>| {
            values.as_ref().map(|all| {
                (0..all.len())
                    .filter(|&i| kept[i] != u32::MAX)
                    .map(|i| all[i])
                    .collect::<Vec<f32>>()
            })
        };
        (out.card_shade, out.ao) = (per_vert(&mesh.card_shade), per_vert(&mesh.ao));
        let (mut uvs, mut own) = (Vec::new(), Vec::new());
        for (f, face) in mesh.faces.iter().enumerate() {
            let c = mesh.corners(f);
            if c.iter().any(|&v| kept[v as usize] == u32::MAX) {
                continue;
            }
            let corners: Vec<u32> = c.iter().map(|&v| kept[v as usize]).collect();
            out.push_face(&corners, face.mat, face.smooth);
            let at = face.start as usize..(face.start + face.len) as usize;
            if let Some(all) = &mesh.uvs {
                uvs.extend_from_slice(&all[at.clone()]);
            }
            if let Some(all) = &mesh.normals {
                own.extend_from_slice(&all[at]);
            }
        }
        out.uvs = mesh.uvs.as_ref().map(|_| uvs);
        out.normals = mesh.normals.as_ref().map(|_| own);
        out.sharp = mesh
            .sharp
            .iter()
            .filter(|e| kept[e.0 as usize] != u32::MAX && kept[e.1 as usize] != u32::MAX)
            .map(|e| edge(kept[e.0 as usize], kept[e.1 as usize]))
            .collect();
        part.mesh = out;
    });
    obj
}
