//! Cutting a mesh along a plane, and closing what the cut leaves open.

use super::*;
use crate::math::{P3, V3, dot_v3, normalize_v3};
use crate::mesh::{Mesh, edge};
use crate::scene::Obj;
use crate::shapes::options;
use std::collections::BTreeMap;

options! {
    /// The options of bisect_plane, Blender's own defaults: dist is how near the plane a vertex counts as on
    /// it (0: only the points the cut makes); clear_outer takes out what lies in front of the plane (the side its
    /// normal points to).
    Bisect { dist: f64 = 0.0, clear_outer: bool = false }
}

/// bmesh.ops.bisect_plane(bm, geom=everything, plane_co, plane_no, dist, clear_outer) on the mesh of obj as it is
/// (in its own frame: a script that cuts in the world moves the mesh there first, as with bm.transform): every edge
/// that crosses the plane gets a point where it does, every face is split along the plane, and what lies in front
/// of it is taken out. Returns the cut's edges (geom_cut), each
/// its two points, for holes_fill.
///
/// The points are Blender's (a crossing is the edge's lower-numbered end plus the way to the other times
/// dist / (dist - other dist), in single precision); the new ones come after the old in the order the edges are
/// first met round the faces, which is not always the order of Blender's own edge list.
pub fn bisect_plane(
    obj: Obj,
    plane_co: impl Into<P3>,
    plane_no: impl Into<P3>,
    o: Bisect,
) -> Vec<(u32, u32)> {
    let (co, no): (V3, V3) = (plane_co.into().into(), plane_no.into().into());
    obj.edit(|part| {
        let (mesh, cut) = bisected(&part.mesh, co, no, &o);
        part.mesh = mesh;
        cut
    })
}

/// bisect_plane with its options by name: bisect_plane!(obj, co, no, clear_outer = true).
#[macro_export]
macro_rules! bisect_plane {
    ($obj:expr, $co:expr, $no:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $co, $no);
        #[allow(unused_mut)]
        let mut o = $crate::edit::Bisect::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::edit::bisect_plane(args.0, args.1, args.2, o)
    }};
}

fn bisected(mesh: &Mesh, co: V3, no: V3, o: &Bisect) -> (Mesh, Vec<(u32, u32)>) {
    let eps = o.dist as f32;
    let plane_d = -dot_v3(no, co);
    let side_of = |p: V3, d: f32| dot_v3(p, no) + d;
    let mut verts = mesh.verts.clone();
    let old = verts.len();
    let mut dist: Vec<f32> = verts.iter().map(|&p| side_of(p, plane_d)).collect();
    let mut dir: Vec<i8> = dist
        .iter()
        .map(|&f| {
            if f <= -eps {
                -1
            } else if f >= eps {
                1
            } else {
                0
            }
        })
        .collect();
    let mut faces: Vec<Cut> = (0..mesh.faces.len())
        .map(|f| Cut {
            corners: mesh.corners(f).to_vec(),
            mat: mesh.faces[f].mat,
            smooth: mesh.faces[f].smooth,
            normal: mesh.face_normal(f),
        })
        .collect();
    // a point on every edge that goes through the plane
    let mut crossing: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    let mut cut_edges: Vec<(u32, u32)> = Vec::new();
    for face in &mut faces {
        let c = face.corners.clone();
        let mut with = Vec::with_capacity(c.len() + 2);
        for (i, &a) in c.iter().enumerate() {
            let b = c[(i + 1) % c.len()];
            with.push(a);
            let (sa, sb) = (dir[a as usize], dir[b as usize]);
            if sa != 0 && sb != 0 && sa != sb {
                let key = edge(a, b);
                let at = *crossing.entry(key).or_insert_with(|| {
                    let (v, other) = (verts[key.0 as usize], verts[key.1 as usize]);
                    let (d0, d1) = (dist[key.0 as usize], dist[key.1 as usize]);
                    let fac = d0 / (d0 - d1);
                    let d = other - v;
                    verts.push(V3::new(v.x + d.x * fac, v.y + d.y * fac, v.z + d.z * fac));
                    dist.push(0.0);
                    dir.push(0);
                    verts.len() as u32 - 1
                });
                with.push(at);
            } else if sa == 0 && sb == 0 {
                // an edge lying on the plane is part of the cut
                if !cut_edges.contains(&edge(a, b)) {
                    cut_edges.push(edge(a, b));
                }
            }
        }
        face.corners = with;
    }
    // each face split along the plane, between its points on it
    let plane_no = no;
    let whole = faces.len();
    for f in 0..whole {
        let made = bisect_face(&faces[f], &verts, &dir, plane_no, &mut cut_edges);
        if let Some(mut parts) = made {
            faces[f] = parts.remove(0);
            faces.extend(parts);
        }
    }
    // what lies in front of the plane goes: the points the cut made stay
    let outer = plane_d - eps;
    let gone = |i: usize| i < old && o.clear_outer && side_of(mesh.verts[i], outer) > 0.0;
    let mut kept = vec![u32::MAX; verts.len()];
    let mut out = Mesh {
        mats: mesh.mats.clone(),
        ..Mesh::default()
    };
    for (i, &p) in verts.iter().enumerate() {
        if !gone(i) {
            kept[i] = out.verts.len() as u32;
            out.verts.push(p);
        }
    }
    for face in &faces {
        if face.corners.iter().all(|&v| kept[v as usize] != u32::MAX) {
            let corners: Vec<u32> = face.corners.iter().map(|&v| kept[v as usize]).collect();
            out.push_face(&corners, face.mat, face.smooth);
        }
    }
    // a sharp edge the cut splits stays sharp on both sides of its point
    let mut sharp = mesh.sharp.clone();
    for (&(a, b), &c) in &crossing {
        if sharp.remove(&(a, b)) {
            sharp.insert(edge(a, c));
            sharp.insert(edge(c, b));
        }
    }
    let both = |e: &(u32, u32)| kept[e.0 as usize] != u32::MAX && kept[e.1 as usize] != u32::MAX;
    out.sharp = sharp
        .iter()
        .filter(|e| both(e))
        .map(|e| edge(kept[e.0 as usize], kept[e.1 as usize]))
        .collect();
    let cut = cut_edges
        .iter()
        .filter(|e| both(e))
        .map(|e| edge(kept[e.0 as usize], kept[e.1 as usize]))
        .collect();
    (out, cut)
}

/// bmesh's bm_face_bisect_verts: the face split between its corners on the plane, where it has corners on both
/// sides of it. The faces it becomes (the first in its place), or None where it stays whole.
fn bisect_face(
    face: &Cut,
    verts: &[V3],
    dir: &[i8],
    plane_no: V3,
    cut_edges: &mut Vec<(u32, u32)>,
) -> Option<Vec<Cut>> {
    let c = &face.corners;
    let n = c.len();
    let d = |i: usize| dir[c[i % n] as usize];
    let centre: Vec<usize> = (0..n).filter(|&i| d(i) == 0).collect();
    let has = |side: i8| (0..n).any(|i| d(i) == side);
    if centre.len() < 2 || !(has(-1) && has(1)) {
        return None;
    }
    let part = |corners: Vec<u32>| Cut {
        corners,
        ..face.clone()
    };
    if centre.len() == 2 {
        let (ia, ib) = (centre[0], centre[1]);
        if ib == ia + 1 || (ia == 0 && ib == n - 1) {
            return None;
        }
        let (kept, new) = split_face(c, ia, ib);
        cut_edges.push(edge(c[ia], c[ib]));
        return Some(vec![part(kept), part(new)]);
    }
    // more than two: sorted across the face, and split between each pair that has the face's inside between them
    let mut skip: Vec<bool> = (0..n)
        .map(|i| d(i) == 0 && (d(i + n - 1) ^ d(i + 1)) == 0)
        .collect();
    let centre_edge = centre.iter().any(|&i| d(i + n - 1) == 0);
    if centre_edge {
        // a run of corners on the plane between two corners on the same side of it is not a crossing
        let start = (0..n).find(|&i| d(i) != 0).unwrap();
        let mut k = 0;
        while k < n {
            let i = (start + k) % n;
            if !skip[i] && d(i) == 0 && d(i + 1) == 0 {
                let prev = (i + n - 1) % n;
                let mut next = (i + 2) % n;
                let mut len = 2;
                while d(next) == 0 {
                    next = (next + 1) % n;
                    len += 1;
                }
                if (d(prev) ^ d(next)) == 0 {
                    for s in 0..len {
                        skip[(i + s) % n] = true;
                    }
                }
                k += len;
            } else {
                k += 1;
            }
        }
    }
    let sort_dir = {
        let (unit, len) = normalize_v3(face.normal.cross(plane_no));
        if len == 0.0 {
            let first = verts[c[centre[0]] as usize];
            let other = centre
                .iter()
                .map(|&i| verts[c[i] as usize])
                .rfind(|&p| p != first)?;
            normalize_v3(first - other).0
        } else {
            unit
        }
    };
    let mut sorted = centre.clone();
    sorted.sort_by(|&a, &b| {
        let key = |i: usize| dot_v3(sort_dir, verts[c[i] as usize]);
        key(a).total_cmp(&key(b))
    });
    let mut parts: Vec<Vec<u32>> = vec![c.clone()];
    let mut inside = false;
    for pair in sorted.windows(2) {
        let (ia, ib) = (pair[0], pair[1]);
        if centre_edge {
            let delta = ia.abs_diff(ib);
            if delta == 1 || delta == n - 1 {
                continue;
            }
        }
        if !skip[ia] {
            inside = !inside;
        }
        if !inside {
            continue;
        }
        let (va, vb) = (c[ia], c[ib]);
        let found = parts
            .iter()
            .position(|p| p.contains(&va) && p.contains(&vb));
        if let Some(j) = found {
            let p = parts[j].clone();
            let (pa, pb) = (
                p.iter().position(|&v| v == va).unwrap(),
                p.iter().position(|&v| v == vb).unwrap(),
            );
            let m = p.len();
            if pa.abs_diff(pb) == 1 || pa.abs_diff(pb) == m - 1 {
                continue;
            }
            let (kept, new) = split_face(&p, pa, pb);
            cut_edges.push(edge(va, vb));
            parts[j] = kept;
            parts.push(new);
        }
    }
    if parts.len() == 1 {
        return None;
    }
    Some(parts.into_iter().map(part).collect())
}

/// bmesh.ops.holes_fill(bm, edges=edges) on the mesh of obj: each closed run of edges that have a face on one side
/// only (a cut left open by bisect_plane) is closed with one face, wound to go with the faces round it and given
/// the material and shading of the face along its first edge. Returns the faces made, as their places in the mesh.
pub fn holes_fill(obj: Obj, edges: &[(u32, u32)]) -> Vec<usize> {
    obj.edit(|part| {
        let mesh = &mut part.mesh;
        // the faces along each edge, with the way each runs it
        let mut along: BTreeMap<(u32, u32), Vec<OpenEdge>> = BTreeMap::new();
        for f in 0..mesh.faces.len() {
            let c = mesh.corners(f);
            for (i, &a) in c.iter().enumerate() {
                let b = c[(i + 1) % c.len()];
                along.entry(edge(a, b)).or_default().push((f, a, b));
            }
        }
        // the hole's side of each open edge runs against its face: from b back to a
        let mut next: BTreeMap<u32, (u32, usize)> = BTreeMap::new();
        for e in edges {
            if let Some(on) = along.get(&edge(e.0, e.1))
                && on.len() == 1
            {
                let (f, a, b) = on[0];
                next.insert(b, (a, f));
            }
        }
        let mut made = Vec::new();
        while let Some((&start, _)) = next.iter().next() {
            let mut ring = vec![start];
            let mut from = None;
            let mut v = start;
            let mut closed = false;
            while let Some((w, f)) = next.remove(&v) {
                from.get_or_insert(f);
                if w == start {
                    closed = true;
                    break;
                }
                ring.push(w);
                v = w;
            }
            if closed && ring.len() >= 3 {
                let like = mesh.faces[from.unwrap()];
                mesh.push_face(&ring, like.mat, like.smooth);
                made.push(mesh.faces.len() - 1);
            }
        }
        if !made.is_empty() {
            // (what was stored a corner is no longer a corner's: the script's UVs and normals are made after)
            (mesh.uvs, mesh.normals) = (None, None);
        }
        made
    })
}
