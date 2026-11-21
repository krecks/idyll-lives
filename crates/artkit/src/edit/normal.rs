//! The weighted-normal modifier.

use crate::math::{V3, normalize_v3};
use crate::mesh::{Mesh, edge};
use crate::normals;
use crate::scene::Obj;
use crate::shapes::options;
use std::collections::BTreeMap;

options! {
    /// The options of weighted_normal, Blender's own defaults: keep_sharp gives every smooth run of faces round a
    /// point (between its sharp edges) a normal of its own, where otherwise all the faces at a point share one.
    WeightedNormal { keep_sharp: bool = false }
}

/// A face's area as Blender's mesh works it out: a triangle's from two of its sides, any other's half its
/// Newell sum's length.
fn face_area(mesh: &Mesh, f: usize) -> f32 {
    let c = mesh.corners(f);
    let p = |i: usize| mesh.verts[c[i] as usize];
    if c.len() == 3 {
        return crate::math::len_v3((p(0) - p(1)).cross(p(1) - p(2))) * 0.5;
    }
    let mut n = V3::ZERO;
    let mut prev = p(c.len() - 1);
    for i in 0..c.len() {
        let cur = p(i);
        n.x += (prev.y - cur.y) * (prev.z + cur.z);
        n.y += (prev.z - cur.z) * (prev.x + cur.x);
        n.z += (prev.x - cur.x) * (prev.y + cur.y);
        prev = cur;
    }
    crate::math::len_v3(n) * 0.5
}

/// Blender's WEIGHTED_NORMAL modifier (by face area, its weight and threshold as they come), applied: the corners
/// round a point take the sum of their faces' normals, each times its face's area, the largest face first, made a
/// unit vector: a bevelled shell's broad faces stay flat and its bevels carry the turn. With keep_sharp the
/// corners of each smooth run of faces share one (a run ends at an edge marked sharp, at one with one face or
/// more than two, and at a flat face); without, all the corners at a point do. A run whose sum is nothing keeps
/// the normal it had.
///
/// The mesh is left as Blender leaves it: the normals are kept as two angles of sixteen bits in each run's own
/// frame (`Mesh::stored`), in place of whatever was kept before (a bevel's hardened normals), and are what those
/// angles read as, a ten-thousandth of a turn or so off the sums.
pub fn weighted_normal(obj: Obj, o: WeightedNormal) -> Obj {
    obj.edit(|part| {
        let mesh = &part.mesh;
        let nf = mesh.faces.len();
        let normals = mesh.face_normals();
        let areas: Vec<f32> = (0..nf).map(|f| face_area(mesh, f)).collect();
        // the item each corner adds to: its vertex, or its smooth run of faces round its vertex
        let item: Vec<usize> = if o.keep_sharp {
            fans(mesh)
        } else {
            mesh.loops.iter().map(|&v| v as usize).collect()
        };
        let items = item.iter().copied().max().map_or(0, |m| m + 1);
        // the faces largest first (of two alike, the first)
        let mut order: Vec<usize> = (0..nf).collect();
        order.sort_by(|&a, &b| areas[b].total_cmp(&areas[a]));
        let mut sums = vec![V3::ZERO; items];
        for &f in &order {
            let face = mesh.faces[f];
            for corner in face.start as usize..(face.start + face.len) as usize {
                let s = &mut sums[item[corner]];
                s.x += normals[f].x * areas[f];
                s.y += normals[f].y * areas[f];
                s.z += normals[f].z * areas[f];
            }
        }
        for s in &mut sums {
            let (unit, len) = normalize_v3(*s);
            *s = if len < 1e-6 { V3::ZERO } else { unit };
        }
        let own: Vec<V3> = if o.keep_sharp {
            let had = normals::corner_normals(mesh);
            (0..mesh.loops.len())
                .map(|corner| {
                    let s = sums[item[corner]];
                    if s == V3::ZERO { had[corner] } else { s }
                })
                .collect()
        } else {
            // (one normal a vertex, as Blender keeps the normals a script gives a mesh's points)
            mesh.loops
                .iter()
                .map(|&v| normals::unit(sums[v as usize]))
                .collect()
        };
        // Blender writes the two angles it keeps for each corner over the ones the mesh had (a bevel's), and the
        // normals are what they read as from then on
        let kept = normals::to_stored(mesh, &own);
        part.mesh.normals = Some(normals::from_stored(mesh, &kept));
        part.mesh.stored = Some(kept.into_iter().map(Some).collect());
    });
    obj
}

/// For each corner, the number of its smooth run of faces round its vertex: corners at one vertex whose faces
/// meet across an edge that is not marked sharp, has those two faces only, both smooth, wound the same way round.
fn fans(mesh: &Mesh) -> Vec<usize> {
    let n = mesh.loops.len();
    // each edge's faces: (face, whether it runs the edge from its lower vertex)
    let mut on_edge: BTreeMap<(u32, u32), Vec<(usize, bool)>> = BTreeMap::new();
    // the corners at each vertex, by the vertex at the other end of each of their two edges there
    let mut by_side: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        let start = mesh.faces[f].start as usize;
        for i in 0..c.len() {
            let (prev, next) = (c[(i + c.len() - 1) % c.len()], c[(i + 1) % c.len()]);
            on_edge
                .entry(edge(c[i], next))
                .or_default()
                .push((f, c[i] < next));
            by_side.entry((c[i], prev)).or_default().push(start + i);
            by_side.entry((c[i], next)).or_default().push(start + i);
        }
    }
    let smooth = |e: (u32, u32)| {
        let on = &on_edge[&e];
        on.len() == 2
            && on[0].1 != on[1].1
            && !mesh.sharp.contains(&e)
            && mesh.faces[on[0].0].smooth
            && mesh.faces[on[1].0].smooth
    };
    let mut root: Vec<usize> = (0..n).collect();
    fn find(root: &mut [usize], mut i: usize) -> usize {
        while root[i] != i {
            root[i] = root[root[i]];
            i = root[i];
        }
        i
    }
    for (&(v, w), corners) in &by_side {
        if corners.len() == 2 && smooth(edge(v, w)) {
            let (a, b) = (find(&mut root, corners[0]), find(&mut root, corners[1]));
            root[a.max(b)] = a.min(b);
        }
    }
    // numbered in the order their first corners come
    let mut number: BTreeMap<usize, usize> = BTreeMap::new();
    (0..n)
        .map(|corner| {
            let r = find(&mut root, corner);
            let next = number.len();
            *number.entry(r).or_insert(next)
        })
        .collect()
}

/// weighted_normal with its options by name: weighted_normal!(obj, keep_sharp = true).
#[macro_export]
macro_rules! weighted_normal {
    ($obj:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj,);
        #[allow(unused_mut)]
        let mut o = $crate::edit::WeightedNormal::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::edit::weighted_normal(args.0, o)
    }};
}
