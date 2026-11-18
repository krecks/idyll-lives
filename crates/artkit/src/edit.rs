//! What a script does to a part's mesh by hand, with Blender's own tools: a modifier it adds and applies itself
//! (SOLIDIFY), a bmesh operator it runs (delete, bisect_plane, holes_fill, triangulate, inset_individual). Each is Blender's, point
//! for point in the same single-precision sums, so what the script builds on it comes out the same. The cuts and
//! their triangles are Blender's code (bmesh_bisect_plane, BLI_polyfill_2d_beautify, bmesh_polygon:
//! GPL-2.0-or-later), worked through again here.

use crate::math::{P3, V3, angle_normalized_v3v3, dot_v3, normalize_v3};
use crate::mesh::{Mesh, edge};
use crate::normals;
use crate::scene::Obj;
use crate::shapes::options;
use std::collections::BTreeMap;

options! {
    /// `solidify`'s options, Blender's own defaults: `offset` -1 grows the shell inwards (against the normals), 1
    /// outwards, 0 half each way; `use_even_offset` keeps the shell as thick where the surface bends as where it is
    /// flat; `use_rim` closes it round the surface's open edges.
    Solidify { offset: f64 = -1.0, use_even_offset: bool = false, use_rim: bool = true }
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

/// A face along an edge: (the face, the vertex it comes from, the vertex it goes to).
type OpenEdge = (usize, u32, u32);

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
                let angle = angle_normalized_v3v3(nor_prev, nor_next).max(f32::EPSILON);
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
    if o.use_rim {
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

// ---- bisect_plane, holes_fill, triangulate ------------------------------------------------------------

options! {
    /// The options of bisect_plane, Blender's own defaults: dist is how near the plane a vertex counts as on
    /// it (0: only the points the cut makes); clear_outer takes out what lies in front of the plane (the side its
    /// normal points to), clear_inner what lies behind.
    Bisect { dist: f64 = 0.0, clear_outer: bool = false, clear_inner: bool = false }
}

/// How bmesh.ops.triangulate cuts a quad in two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum QuadMethod {
    /// The diagonal that leaves the rounder triangles (Blender's default).
    #[default]
    Beauty,
    /// From the first corner to the third.
    Fixed,
    /// From the second corner to the fourth.
    Alternate,
    ShortEdge,
    LongEdge,
}

/// How bmesh.ops.triangulate cuts a polygon of five corners or more.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NgonMethod {
    /// Ears clipped, then cuts turned while that leaves rounder triangles (Blender's default).
    #[default]
    Beauty,
    EarClip,
}

/// A face while it is being cut: its corners from the one it starts at, its material and shading, and the
/// normal it had before the cut (bmesh keeps a face's normal through its splits).
#[derive(Clone)]
struct Cut {
    corners: Vec<u32>,
    mat: u16,
    smooth: bool,
    normal: V3,
}

/// bmesh's split of a face from the corner at ia to the corner at ib (not neighbours): the part that keeps the
/// face's place, from ia round to ib, and the new face, from ib round to ia, each starting where bmesh starts it.
fn split_face(c: &[u32], ia: usize, ib: usize) -> (Vec<u32>, Vec<u32>) {
    let n = c.len();
    // the corners from one place round to (not with) another
    let run = |from: usize, to: usize| -> Vec<usize> {
        let mut out = Vec::new();
        let mut i = from;
        while i != to {
            out.push(i);
            i = (i + 1) % n;
        }
        out
    };
    // the kept face: a new corner at ib's vertex, then ia .. the corner before ib
    let kept = run(ia, ib);
    let other = run(ib, ia);
    let in_kept = kept.contains(&0);
    let ids = |first: usize, new_at: usize, rest: &[usize]| -> Vec<u32> {
        // the cycle as bmesh has it: the new corner (at vertex new_at), then the run; turned to start at first
        let mut cycle: Vec<(bool, usize)> = vec![(true, new_at)];
        cycle.extend(rest.iter().map(|&i| (false, i)));
        cycle.rotate_left(first);
        cycle.into_iter().map(|(_, i)| c[i]).collect()
    };
    // (places in each cycle: 0 is its new corner, k + 1 the run's k-th)
    let (kept_first, other_first) = if in_kept {
        let at = kept.iter().position(|&i| i == 0).unwrap() + 1;
        let other_first = if at == 1 {
            other.len() // the corner before the other's new one
        } else if at == kept.len() {
            1 // the corner after it
        } else {
            0
        };
        (at, other_first)
    } else {
        let at = other.iter().position(|&i| i == 0).unwrap() + 1;
        let kept_first = if at == 1 {
            kept.len()
        } else if at == other.len() {
            1
        } else {
            0
        };
        (kept_first, at)
    };
    (ids(kept_first, ib, &kept), ids(other_first, ia, &other))
}

/// bmesh.ops.bisect_plane(bm, geom=everything, plane_co, plane_no, dist, clear_outer, clear_inner) on the mesh of
/// obj as it is (in its own frame: a script that cuts in the world moves the mesh there first, as with
/// bm.transform): every edge that crosses the plane gets a point where it does, every face is split along the
/// plane, and what lies in front of it or behind it is taken out. Returns the cut's edges (geom_cut), each
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
    // what lies in front of the plane, or behind it, goes: the points the cut made stay
    let outer = plane_d - eps;
    let inner = plane_d + eps;
    let gone = |i: usize| {
        i < old
            && ((o.clear_outer && side_of(mesh.verts[i], outer) > 0.0)
                || (o.clear_inner && side_of(mesh.verts[i], inner) < 0.0))
    };
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
    let both = |e: &(u32, u32)| kept[e.0 as usize] != u32::MAX && kept[e.1 as usize] != u32::MAX;
    out.sharp = mesh
        .sharp
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

/// Two vectors square to the unit vector n and to each other (Blender's ortho_basis_v3v3_v3).
fn ortho_basis(n: V3) -> (V3, V3) {
    let f = n.x * n.x + n.y * n.y;
    if f > f32::EPSILON {
        let d = 1.0 / f.sqrt();
        let n1 = V3::new(n.y * d, -n.x * d, 0.0);
        let n2 = V3::new(-n.z * n1.y, n.z * n1.x, n.x * n1.y - n.y * n1.x);
        (n1, n2)
    } else {
        (
            V3::new(if n.z < 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0),
            V3::new(0.0, 1.0, 0.0),
        )
    }
}

/// A point flat on the plane square to n (axis_dominant_v3_to_m3, then mul_v2_m3v3).
fn flat(p: V3, (n1, n2): (V3, V3)) -> [f32; 2] {
    [
        n1.x * p.x + n1.y * p.y + n1.z * p.z,
        n2.x * p.x + n2.y * p.y + n2.z * p.z,
    ]
}

fn cross_tri_v2(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2]) -> f32 {
    (v1[0] - v2[0]) * (v2[1] - v3[1]) + (v1[1] - v2[1]) * (v3[0] - v2[0])
}

fn len_v2v2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (x, y) = (a[0] - b[0], a[1] - b[1]);
    (x * x + y * y).sqrt()
}

/// Blender's BLI_polyfill_beautify_quad_rotate_calc_ex: what turning the cut 2-4 of the flat quad 1, 2, 3, 4 into
/// 1-3 would gain (under zero: the triangles either side of 1-3 are rounder, area over perimeter), and the quad's
/// area.
fn quad_rotate(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2], v4: [f32; 2], lock: bool) -> (f32, f32) {
    let eps_zero_area = 1e-12_f32;
    let a234 = cross_tri_v2(v2, v3, v4);
    let a241 = cross_tri_v2(v2, v4, v1);
    let a123 = cross_tri_v2(v1, v2, v3);
    let a134 = cross_tri_v2(v1, v3, v4);
    let area = (a234.abs() + a241.abs() + a123.abs() + a134.abs()) / 8.0;
    if (a123 >= 0.0) != (a134 >= 0.0) {
        return (f32::MAX, area);
    }
    if a123.abs() <= eps_zero_area || a134.abs() <= eps_zero_area {
        return (f32::MAX, area);
    }
    if (a234 >= 0.0) != (a241 >= 0.0) {
        return (if lock { f32::MAX } else { -f32::MAX }, area);
    }
    if a234.abs() <= eps_zero_area || a241.abs() <= eps_zero_area {
        return (-f32::MAX, area);
    }
    let (len_12, len_23, len_34, len_41) = (
        len_v2v2(v1, v2),
        len_v2v2(v2, v3),
        len_v2v2(v3, v4),
        len_v2v2(v4, v1),
    );
    let (len_13, len_24) = (len_v2v2(v1, v3), len_v2v2(v2, v4));
    let fac_24 =
        (a234.abs() / (len_23 + len_34 + len_24)) + (a241.abs() / (len_41 + len_12 + len_24));
    let fac_13 =
        (a123.abs() / (len_12 + len_23 + len_13)) + (a134.abs() / (len_34 + len_41 + len_13));
    (fac_24 - fac_13, area)
}

/// Blender's BLI_polyfill_edge_calc_rotate_beauty__area for a quad in space.
fn quad_rotate_v3(v1: V3, v2: V3, v3: V3, v4: V3) -> f32 {
    let cross_tri = |a: V3, b: V3, c: V3| (a - b).cross(b - c);
    let (no, scale) = normalize_v3(cross_tri(v2, v3, v4) + cross_tri(v2, v4, v1));
    if scale == 0.0 {
        return f32::MAX;
    }
    let basis = ortho_basis(no);
    let (a, b, c, d) = (
        flat(v1, basis),
        flat(v2, basis),
        flat(v3, basis),
        flat(v4, basis),
    );
    let sign = |x: f32| {
        if x > 1e-5 {
            1
        } else if x < -1e-5 {
            -1
        } else {
            0
        }
    };
    if sign(cross_tri_v2(b, c, d) / scale) + sign(cross_tri_v2(b, d, a) / scale) == 0 {
        return f32::MAX;
    }
    quad_rotate(a, b, c, d, false).0
}

/// Blender's is_quad_flip_v3: bit 0 where the quad folds over cut 1-3, bit 1 where over 2-4.
fn quad_flip(v1: V3, v2: V3, v3: V3, v4: V3) -> u8 {
    let (d_12, d_23, d_34, d_41) = (v1 - v2, v2 - v3, v3 - v4, v4 - v1);
    let mut ret = 0;
    if dot_v3(d_12.cross(d_23), d_34.cross(d_41)) < 0.0 {
        ret |= 1;
    }
    if dot_v3(d_23.cross(d_34), d_41.cross(d_12)) < 0.0 {
        ret |= 2;
    }
    ret
}

/// Blender's BLI_polyfill_beautify: the triangles of a flat polygon (its corners coords, in order) with their
/// cuts turned, the best gain first, until none leaves rounder triangles.
fn beautify(coords: &[[f32; 2]], tris: &mut [[u32; 3]]) {
    let last = coords.len() as u32 - 1;
    let n = tris.len();
    if n < 2 {
        return;
    }
    const NONE: usize = usize::MAX;
    // half edges: (the vertex it starts at, the next round its triangle, its twin, its edge's number)
    let mut v: Vec<u32> = vec![0; 3 * n];
    let mut next: Vec<usize> = vec![0; 3 * n];
    let mut radial: Vec<usize> = vec![NONE; 3 * n];
    let mut base: Vec<usize> = vec![NONE; 3 * n];
    let mut inner: Vec<((u32, u32), usize)> = Vec::new();
    for (i, t) in tris.iter().enumerate() {
        for (j_prev, j_curr) in [(2, 0), (0, 1), (1, 2)] {
            let e = i * 3 + j_prev;
            v[e] = t[j_prev];
            next[e] = i * 3 + j_curr;
            let pair = edge(t[j_prev], t[j_curr]);
            let boundary = pair.0 + 1 == pair.1 || (pair.0 == 0 && pair.1 == last);
            if !boundary {
                inner.push((pair, e));
            }
        }
    }
    inner.sort();
    for (number, pair) in inner.chunks(2).enumerate() {
        let (a, b) = (pair[0].1, pair[1].1);
        (radial[a], radial[b]) = (b, a);
        (base[a], base[b]) = (number, number);
    }
    let cost_of = |e: usize, v: &[u32], next: &[usize], radial: &[usize]| -> (f32, f32) {
        let b = radial[e];
        let a_other = next[next[e]];
        let b_other = next[next[b]];
        quad_rotate(
            coords[v[a_other] as usize],
            coords[v[e] as usize],
            coords[v[b_other] as usize],
            coords[v[b] as usize],
            false,
        )
    };
    // the cuts worth turning, each by its edge's number: (gain, the half edge)
    let mut heap: Vec<Option<(f32, usize)>> = vec![None; inner.len() / 2];
    for e in 0..3 * n {
        if radial[e] != NONE && radial[e] < e {
            let (cost, _) = cost_of(e, &v, &next, &radial);
            if cost < 0.0 {
                heap[base[e]] = Some((cost, e));
            }
        }
    }
    loop {
        let mut best: Option<(f32, usize, usize)> = None;
        for (number, slot) in heap.iter().enumerate() {
            if let Some((cost, e)) = *slot
                && best.is_none_or(|(c, _, _)| cost < c)
            {
                best = Some((cost, e, number));
            }
        }
        let Some((_, e, number)) = best else { break };
        heap[number] = None;
        // the cut turned: the two triangles either side of it made over
        let ed = {
            let e1 = next[e];
            let e2 = next[e1];
            let e3 = radial[e];
            let e4 = next[e3];
            let e5 = next[e4];
            [e, e1, e2, e3, e4, e5]
        };
        next[ed[0]] = ed[2];
        next[ed[1]] = ed[3];
        next[ed[2]] = ed[4];
        next[ed[3]] = ed[5];
        next[ed[4]] = ed[0];
        next[ed[5]] = ed[1];
        v[ed[0]] = v[ed[5]];
        v[ed[3]] = v[ed[2]];
        // the four cuts round it weighed again
        let a0 = next[e];
        let a1 = next[a0];
        let twin = radial[e];
        let b0 = next[twin];
        let b1 = next[b0];
        for other in [a0, a1, b0, b1] {
            if base[other] == NONE {
                continue;
            }
            let (cost, area) = cost_of(other, &v, &next, &radial);
            heap[base[other]] = (cost < -1e-6 * area.max(1.0)).then_some((cost, other));
        }
    }
    let mut at = 0;
    for e in 0..3 * n {
        if v[e] != u32::MAX {
            let (e1, e2) = (next[e], next[next[e]]);
            tris[at] = [v[e], v[e1], v[e2]];
            (v[e], v[e1], v[e2]) = (u32::MAX, u32::MAX, u32::MAX);
            at += 1;
        }
    }
}

/// bmesh.ops.triangulate(bm, faces=all, quad_method, ngon_method) on the mesh of obj: every face of four corners
/// or more cut into triangles as Blender cuts it; the last triangle of a face takes its place, the others come
/// after the mesh's faces.
pub fn triangulate(obj: Obj, quad: QuadMethod, ngon: NgonMethod) -> Obj {
    obj.edit(|part| {
        let mesh = &part.mesh;
        let mut faces: Vec<(Vec<u32>, u16, bool)> = (0..mesh.faces.len())
            .map(|f| {
                (
                    mesh.corners(f).to_vec(),
                    mesh.faces[f].mat,
                    mesh.faces[f].smooth,
                )
            })
            .collect();
        let whole = faces.len();
        let mut changed = false;
        for f in 0..whole {
            let c = faces[f].0.clone();
            if c.len() < 4 {
                continue;
            }
            changed = true;
            let p: Vec<V3> = c.iter().map(|&v| mesh.verts[v as usize]).collect();
            let tris: Vec<[u32; 3]> = if c.len() == 4 {
                // (bmesh names the corners from the second: 1, 2, 3 and the first as 4)
                let split_24 = match quad {
                    QuadMethod::Fixed => true,
                    QuadMethod::Alternate => false,
                    QuadMethod::ShortEdge | QuadMethod::LongEdge => {
                        let sq = |a: V3, b: V3| dot_v3(a - b, a - b);
                        let (d1, d2) = (sq(p[0], p[2]), sq(p[1], p[3]));
                        if quad == QuadMethod::ShortEdge {
                            (d2 - d1) > 0.0
                        } else {
                            (d2 - d1) < 0.0
                        }
                    }
                    QuadMethod::Beauty => {
                        let flip = quad_flip(p[1], p[2], p[3], p[0]);
                        if flip & 1 != 0 {
                            true
                        } else if flip & 2 != 0 {
                            false
                        } else {
                            quad_rotate_v3(p[1], p[2], p[3], p[0]) > 0.0
                        }
                    }
                };
                if split_24 {
                    vec![[0, 1, 2], [0, 2, 3]]
                } else {
                    vec![[1, 2, 3], [1, 3, 0]]
                }
            } else {
                let normal = crate::mesh::normal_of(&p);
                let basis = ortho_basis(-normal);
                let coords: Vec<[f32; 2]> = p.iter().map(|&q| flat(q, basis)).collect();
                let mut tris = crate::botany::polyfill::fill(&coords);
                if ngon == NgonMethod::Beauty {
                    beautify(&coords, &mut tris);
                }
                tris
            };
            let of = |t: &[u32; 3]| vec![c[t[0] as usize], c[t[1] as usize], c[t[2] as usize]];
            let (mat, smooth) = (faces[f].1, faces[f].2);
            let (last, rest) = tris.split_last().unwrap();
            for t in rest {
                faces.push((of(t), mat, smooth));
            }
            faces[f].0 = of(last);
        }
        if !changed {
            return;
        }
        let mut out = Mesh {
            verts: mesh.verts.clone(),
            mats: mesh.mats.clone(),
            sharp: mesh.sharp.clone(),
            card_shade: mesh.card_shade.clone(),
            ao: mesh.ao.clone(),
            ..Mesh::default()
        };
        for (corners, mat, smooth) in &faces {
            out.push_face(corners, *mat, *smooth);
        }
        part.mesh = out;
    });
    obj
}

// ---- inset_individual ---------------------------------------------------------------------------------

options! {
    /// The options of inset_individual, Blender's own defaults: thickness is how far in from its edges each face
    /// is drawn, depth how far along its normal; use_even_offset pushes a corner further in the sharper it is,
    /// so the strip is as wide all round; use_relative_offset scales both by the lengths of the edges at a corner.
    InsetIndividual { thickness: f64 = 0.0, depth: f64 = 0.0, use_even_offset: bool = false, use_relative_offset: bool = false }
}

/// bmesh.ops.inset_individual(bm, faces=all, thickness, depth, use_even_offset, use_relative_offset) on the mesh of
/// obj: each face drawn in from its own edges and moved along its normal, joined to where its edges were by a strip
/// of quads (a mosaic's tiles, each proud of the grout between them). Every face keeps its place and takes new
/// points, in its corners' order; the old points stay as the strips' outer side; the strips come after the faces,
/// each face's in its edges' order.
pub fn inset_individual(obj: Obj, o: InsetIndividual) -> Obj {
    let (thickness, depth) = (o.thickness as f32, o.depth as f32);
    obj.edit(|part| {
        let mesh = &part.mesh;
        let mut out = Mesh {
            verts: mesh.verts.clone(),
            mats: mesh.mats.clone(),
            sharp: mesh.sharp.clone(),
            ..Mesh::default()
        };
        let mut strips: Vec<(Vec<u32>, u16, bool)> = Vec::new();
        for (f, face) in mesh.faces.iter().enumerate() {
            let c = mesh.corners(f);
            let n = c.len();
            let p: Vec<V3> = c.iter().map(|&v| mesh.verts[v as usize]).collect();
            let no = mesh.face_normal(f);
            // each edge's way into the face, and its length
            let edge_nors: Vec<V3> = (0..n)
                .map(|i| normalize_v3((p[i] - p[(i + 1) % n]).cross(no)).0)
                .collect();
            let length = |i: usize| crate::math::len_v3(p[(i + 1) % n] - p[i]);
            let mut length_prev = if depth != 0.0 { length(n - 1) } else { 0.0 };
            let first = out.verts.len() as u32;
            for i in 0..n {
                let (eno_prev, eno_next) =
                    (edge_nors[(if i != 0 { i } else { n }) - 1], edge_nors[i]);
                let mut tvec = normalize_v3(eno_prev + eno_next).0;
                if o.use_even_offset {
                    // (Blender's shell_v3v3_mid_normalized_to_dist: further in the sharper the corner)
                    let (mid, len) = normalize_v3(eno_prev + eno_next);
                    let cos = if len != 0.0 {
                        dot_v3(eno_prev, mid).abs()
                    } else {
                        0.0
                    };
                    tvec = crate::math::mul_v3_fl(tvec, if cos < 1.0e-8 { 1.0 } else { 1.0 / cos });
                }
                if o.use_relative_offset {
                    tvec =
                        crate::math::mul_v3_fl(tvec, (length(i) + length((i + n - 1) % n)) / 2.0);
                }
                let mut co = p[i];
                co.x += tvec.x * thickness;
                co.y += tvec.y * thickness;
                co.z += tvec.z * thickness;
                if depth != 0.0 {
                    let e_length = length(i);
                    let fac = depth
                        * if o.use_relative_offset {
                            (length_prev + e_length) * 0.5
                        } else {
                            1.0
                        };
                    length_prev = e_length;
                    co.x += no.x * fac;
                    co.y += no.y * fac;
                    co.z += no.z * fac;
                }
                out.verts.push(co);
            }
            let inner: Vec<u32> = (0..n as u32).map(|i| first + i).collect();
            out.push_face(&inner, face.mat, face.smooth);
            for i in 0..n {
                let j = (i + 1) % n;
                strips.push((vec![c[i], c[j], inner[j], inner[i]], face.mat, face.smooth));
            }
        }
        for (corners, mat, smooth) in &strips {
            out.push_face(corners, *mat, *smooth);
        }
        part.mesh = out;
    });
    obj
}

/// inset_individual with its options by name, as the scripts gave them to the operator:
/// inset_individual!(obj, thickness = grout, depth = raised, use_even_offset = true).
#[macro_export]
macro_rules! inset_individual {
    ($obj:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj,);
        #[allow(unused_mut)]
        let mut o = $crate::edit::InsetIndividual::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::edit::inset_individual(args.0, o)
    }};
}
