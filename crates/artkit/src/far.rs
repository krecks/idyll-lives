//! Far versions: a model's own build run again at low detail (`Model::far_of`), so the far version has the same
//! shape, colours and size as the model, standing where it stands. The build asks `low(full, reduced)` for each
//! detail (segments, rings, counts) and `is_low()` for what to leave out (a flower's centre, the moving parts kept
//! apart: a far version is all one still part). It draws every random number the full build draws, in the same
//! order, so the far version's plants stand where the model's do.
//!
//! Or the model itself, joined at rest and decimated to a share of its triangles (`Model::far_ratio`,
//! `decimated`): for what is built of whole parts (a bench, a lamp post, a stall), which stay what they are with
//! fewer faces. Small, separate parts (flowers, leaves, a bale's twine) are better built low: decimating shrinks
//! thin parts and takes their detail first.

use crate::decimate;
use crate::math::{M4, V3};
use crate::mesh::{Mesh, edge as mesh_edge};
use crate::scene::{self, Obj};
use std::collections::BTreeSet;

/// `full` detail, or `reduced` while a far version is being built.
pub fn low<T>(full: T, reduced: T) -> T {
    if is_low() { reduced } else { full }
}

/// Whether a far version is being built.
pub fn is_low() -> bool {
    scene::with(|s| s.low)
}

/// Runs `build` as a far version.
pub fn far_of<R>(build: impl FnOnce() -> R) -> R {
    scene::with(|s| s.low = true);
    let out = build();
    scene::with(|s| s.low = false);
    out
}

/// A decimated far version keeps every vertex up to this height (m) over the model's lowest point.
pub const FEET: f64 = 0.01;
/// And every vertex within this (m) of the model's box.
pub const EDGE: f64 = 0.003;

/// Decimates `obj` (a far version, joined, at rest) to `ratio` of its triangles, in place:
/// - the vertices sitting on one another merged first (a can's turned ends meeting on its axis in many: decimated
///   unmerged, they tear into spikes);
/// - its feet (within `feet` of its bottom) and its outermost faces (within `edge` of its box) kept whole, so it
///   stands and measures as the model does;
/// - each face the collapse turned round turned back to face the way the model's surface there does, so it isn't
///   lit from behind.
///
/// Returns `obj`.
pub fn decimated(obj: Obj, ratio: f64, feet: f64, edge: f64) -> Obj {
    let (name, faces, turned) = obj.edit(|o| {
        let turned = decimate(&mut o.mesh, ratio, feet, edge);
        (o.name.clone(), o.mesh.faces.len(), turned)
    });
    scene::log(format!(
        "[gamekit] {name}: {faces} faces, {turned} turned back"
    ));
    obj
}

/// `decimated` on a mesh; how many faces were turned back.
pub fn decimate(mesh: &mut Mesh, ratio: f64, feet: f64, edge: f64) -> usize {
    merge_doubles(mesh, 1e-5);
    let reference = Surface::new(
        mesh.triangle_verts()
            .into_iter()
            .map(|t| t.map(|v| mesh.verts[v as usize]))
            .collect(),
    );
    let locked = locks(mesh, feet, edge);
    let (mut done, was) = decimate::collapse_traced(mesh, ratio, &locked);
    let turned = turn_back(&mut done, &was, &reference);
    *mesh = done;
    turned
}

/// For each of `mesh`'s vertices, whether a far version keeps it as it is: within `feet` of the mesh's bottom, or
/// within `edge` of its box.
pub fn locks(mesh: &Mesh, feet: f64, edge: f64) -> Vec<bool> {
    let Some((lo, hi)) = mesh.bounds(&M4::IDENTITY) else {
        return Vec::new();
    };
    let (lo, hi) = (lo.map(f64::from), hi.map(f64::from));
    mesh.verts
        .iter()
        .map(|v| {
            let v = v.array().map(f64::from);
            let free =
                v[2] > lo[2] + feet && (0..3).all(|i| lo[i] + edge < v[i] && v[i] < hi[i] - edge);
            !free
        })
        .collect()
}

/// `bmesh.ops.remove_doubles` on a mesh that has what a joined one has: the points within `dist` of one another
/// become one, a face that loses a side by it loses a corner (or goes, under three, or as another face's double),
/// and the corners left keep their UVs and normals, the edges left their sharpness, the points their shading.
fn merge_doubles(mesh: &mut Mesh, dist: f32) {
    let target = crate::weld::doubles(&mesh.verts, dist);
    let to = |v: u32| match target[v as usize] {
        t if t >= 0 => t as u32,
        _ => v,
    };
    if (0..mesh.verts.len() as u32).all(|v| to(v) == v) {
        return;
    }
    let mut index = vec![u32::MAX; mesh.verts.len()];
    let mut out = Mesh {
        mats: mesh.mats.clone(),
        uv_metres: mesh.uv_metres,
        ..Mesh::default()
    };
    let mut kept_points = Vec::new();
    for v in 0..mesh.verts.len() {
        if to(v as u32) == v as u32 {
            index[v] = out.verts.len() as u32;
            out.verts.push(mesh.verts[v]);
            kept_points.push(v);
        }
    }
    let per_point = |have: &Option<Vec<f32>>| {
        have.as_ref()
            .map(|h| kept_points.iter().map(|&v| h[v]).collect::<Vec<f32>>())
    };
    out.card_shade = per_point(&mesh.card_shade);
    out.ao = per_point(&mesh.ao);
    let sorted = |c: &[u32]| {
        let mut key = c.to_vec();
        key.sort_unstable();
        key
    };
    // the faces no merge touches are there first: one that becomes the double of any of them goes
    let mut seen: BTreeSet<Vec<u32>> = BTreeSet::new();
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        if c.iter().all(|&v| to(v) == v) {
            seen.insert(sorted(
                &c.iter().map(|&v| index[v as usize]).collect::<Vec<_>>(),
            ));
        }
    }
    let mut corners: Vec<usize> = Vec::new();
    for f in 0..mesh.faces.len() {
        let face = mesh.faces[f];
        let c: Vec<u32> = mesh.corners(f).iter().map(|&v| to(v)).collect();
        let changed = mesh.corners(f).iter().zip(&c).any(|(a, b)| a != b);
        // a corner whose side has closed up goes: the one before the side
        let kept: Vec<usize> = (0..c.len())
            .filter(|&i| c[i] != c[(i + 1) % c.len()])
            .collect();
        let verts: Vec<u32> = kept.iter().map(|&i| index[c[i] as usize]).collect();
        let distinct: BTreeSet<u32> = verts.iter().copied().collect();
        if verts.len() < 3 || distinct.len() != verts.len() {
            continue;
        }
        if changed && !seen.insert(sorted(&verts)) {
            continue;
        }
        out.push_face(&verts, face.mat, face.smooth);
        corners.extend(kept.iter().map(|&i| face.start as usize + i));
    }
    out.uvs = mesh
        .uvs
        .as_ref()
        .map(|uv| corners.iter().map(|&l| uv[l]).collect());
    out.normals = mesh
        .normals
        .as_ref()
        .map(|n| corners.iter().map(|&l| n[l]).collect());
    let mut left: BTreeSet<(u32, u32)> = BTreeSet::new();
    for f in 0..out.faces.len() {
        let c = out.corners(f);
        for i in 0..c.len() {
            left.insert(mesh_edge(c[i], c[(i + 1) % c.len()]));
        }
    }
    for &(a, b) in &mesh.sharp {
        let e = mesh_edge(index[to(a) as usize], index[to(b) as usize]);
        if e.0 != e.1 && left.contains(&e) {
            out.sharp.insert(e);
        }
    }
    *mesh = out;
}

/// Turns each of `mesh`'s triangles that the collapses turned round: one that faces against the `reference`
/// surface nearest its middle, and against the way it faced itself before (`was`: a triangle on a slat or a pane
/// thinner than its vertices moved is nearer the other side's faces, and is as it should be). How many. One of
/// no area is left, as is one as near a reference face that looks its way as any that looks the other.
fn turn_back(mesh: &mut Mesh, was: &[[f64; 3]], reference: &Surface) -> usize {
    let mut turned = 0;
    for f in 0..mesh.faces.len() {
        let face = mesh.faces[f];
        if face.len != 3 {
            continue;
        }
        let p: Vec<[f64; 3]> = mesh
            .corners(f)
            .iter()
            .map(|&v| mesh.verts[v as usize].array().map(f64::from))
            .collect();
        let normal = cross(sub(p[1], p[0]), sub(p[2], p[0]));
        if dot(normal, normal).sqrt() / 2.0 < 1e-8 || dot(normal, was[f]) >= 0.0 {
            continue;
        }
        let middle: [f64; 3] = std::array::from_fn(|k| (p[0][k] + p[1][k] + p[2][k]) / 3.0);
        let Some(nearest) = reference.distance(middle) else {
            continue;
        };
        let near = reference.within(middle, nearest + SAME_PLACE);
        if near.iter().all(|&t| dot(normal, reference.normal(t)) < 0.0) {
            let at = face.start as usize;
            mesh.loops.swap(at + 1, at + 2);
            if let Some(uvs) = &mut mesh.uvs {
                uvs.swap(at + 1, at + 2);
            }
            turned += 1;
        }
    }
    turned
}

/// Reference faces this much further (m) than the nearest are as near.
const SAME_PLACE: f64 = 1e-5;

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// The point of the triangle `t` nearest `p`.
fn nearest_on(p: [f64; 3], t: &[[f64; 3]; 3]) -> [f64; 3] {
    let [a, b, c] = *t;
    let along = |from: [f64; 3], d: [f64; 3], k: f64| -> [f64; 3] {
        std::array::from_fn(|i| from[i] + d[i] * k)
    };
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, bp));
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return along(a, ab, d1 / (d1 - d3));
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, cp), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return along(a, ac, d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return along(b, sub(c, b), (d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let sum = va + vb + vc;
    if sum == 0.0 {
        return a;
    }
    let (v, w) = (vb / sum, vc / sum);
    std::array::from_fn(|i| a[i] + ab[i] * v + ac[i] * w)
}

/// A box of the tree over a surface's triangles: its two children (the next box and `first`), or `count` of the
/// triangles from `first`.
struct Box3 {
    lo: [f64; 3],
    hi: [f64; 3],
    first: u32,
    count: u32,
}

/// Triangles in a tree of boxes, to ask how far a point is from their surface and which of them are there.
pub struct Surface {
    tris: Vec<[[f64; 3]; 3]>,
    boxes: Vec<Box3>,
    order: Vec<u32>,
}

impl Surface {
    /// (A triangle of no area has no nearest point of its own; its neighbours cover its corners.)
    pub fn new(tris: Vec<[V3; 3]>) -> Surface {
        let tris: Vec<[[f64; 3]; 3]> = tris
            .iter()
            .map(|t| t.map(|v| v.array().map(f64::from)))
            .collect();
        let middles: Vec<[f64; 3]> = tris
            .iter()
            .map(|t| std::array::from_fn(|k| (t[0][k] + t[1][k] + t[2][k]) / 3.0))
            .collect();
        let mut order: Vec<u32> = (0..tris.len() as u32)
            .filter(|&t| {
                let [a, b, c] = tris[t as usize];
                let n = cross(sub(b, a), sub(c, a));
                dot(n, n) > 0.0
            })
            .collect();
        let mut surface = Surface {
            tris,
            boxes: Vec::new(),
            order: Vec::new(),
        };
        let n = order.len();
        if n > 0 {
            surface.build(&mut order, 0, n, &middles);
        }
        surface.order = order;
        surface
    }

    fn build(&mut self, order: &mut [u32], start: usize, end: usize, middles: &[[f64; 3]]) -> u32 {
        let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
        let (mut mlo, mut mhi) = (lo, hi);
        for &t in &order[start..end] {
            for p in self.tris[t as usize] {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            for k in 0..3 {
                mlo[k] = mlo[k].min(middles[t as usize][k]);
                mhi[k] = mhi[k].max(middles[t as usize][k]);
            }
        }
        let at = self.boxes.len() as u32;
        self.boxes.push(Box3 {
            lo,
            hi,
            first: start as u32,
            count: (end - start) as u32,
        });
        if end - start > 4 {
            // halved across its middles' longest side, the triangles in order of their middles there (of two
            // as far along, the lower first)
            let mut axis = 0;
            for k in 1..3 {
                if mhi[k] - mlo[k] > mhi[axis] - mlo[axis] {
                    axis = k;
                }
            }
            order[start..end].sort_unstable_by(|&a, &b| {
                middles[a as usize][axis]
                    .total_cmp(&middles[b as usize][axis])
                    .then(a.cmp(&b))
            });
            let mid = (start + end) / 2;
            self.build(order, start, mid, middles);
            let right = self.build(order, mid, end, middles);
            self.boxes[at as usize].first = right;
            self.boxes[at as usize].count = 0;
        }
        at
    }

    fn away(b: &Box3, p: [f64; 3]) -> f64 {
        let mut d2 = 0.0;
        for k in 0..3 {
            let d = (b.lo[k] - p[k]).max(p[k] - b.hi[k]).max(0.0);
            d2 += d * d;
        }
        d2.sqrt()
    }

    /// The triangles within `reach` of `p`, each with how far it is, as the tree is walked.
    fn reached(&self, p: [f64; 3], mut reach: f64, nearest_only: bool) -> Vec<(u32, f64)> {
        let mut out = Vec::new();
        if self.boxes.is_empty() {
            return out;
        }
        let mut stack = vec![(0u32, Surface::away(&self.boxes[0], p))];
        while let Some((at, d)) = stack.pop() {
            if d > reach {
                continue;
            }
            let b = &self.boxes[at as usize];
            if b.count == 0 {
                let (l, r) = (at + 1, b.first);
                let (dl, dr) = (
                    Surface::away(&self.boxes[l as usize], p),
                    Surface::away(&self.boxes[r as usize], p),
                );
                // the nearer box last, to be looked into first
                if dl < dr {
                    stack.push((r, dr));
                    stack.push((l, dl));
                } else {
                    stack.push((l, dl));
                    stack.push((r, dr));
                }
                continue;
            }
            for &t in &self.order[b.first as usize..(b.first + b.count) as usize] {
                let off = sub(nearest_on(p, &self.tris[t as usize]), p);
                let d = dot(off, off).sqrt();
                if d <= reach {
                    out.push((t, d));
                    if nearest_only {
                        reach = d;
                    }
                }
            }
        }
        out
    }

    /// How far `p` is from the surface; None for a surface without triangles.
    pub fn distance(&self, p: [f64; 3]) -> Option<f64> {
        self.reached(p, f64::INFINITY, true).last().map(|&(_, d)| d)
    }

    /// The triangles within `reach` of `p`, by their numbers among those the surface was made of.
    pub fn within(&self, p: [f64; 3], reach: f64) -> Vec<u32> {
        let mut out: Vec<u32> = self
            .reached(p, reach, false)
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        out.sort_unstable();
        out
    }

    /// A triangle's normal, not of unit length.
    pub fn normal(&self, t: u32) -> [f64; 3] {
        let [a, b, c] = self.tris[t as usize];
        cross(sub(b, a), sub(c, a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A square of two triangles a metre up, looking up.
    fn square() -> Mesh {
        let verts = [(0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)].map(V3::from);
        Mesh::from_pydata(verts.to_vec(), &[vec![0, 1, 2], vec![0, 2, 3]])
    }

    fn surface(mesh: &Mesh) -> Surface {
        Surface::new(
            mesh.triangle_verts()
                .into_iter()
                .map(|t| t.map(|v| mesh.verts[v as usize]))
                .collect(),
        )
    }

    #[test]
    fn a_face_turned_round_is_turned_back() {
        let reference = surface(&square());
        let up = [0.0, 0.0, 1.0];
        let mut mesh = square();
        mesh.loops.swap(4, 5);
        assert_eq!(turn_back(&mut mesh, &[up, up], &reference), 1);
        assert_eq!(mesh.loops, square().loops);
        assert_eq!(turn_back(&mut mesh, &[up, up], &reference), 0);
    }

    #[test]
    fn a_face_of_a_thin_part_nearer_its_other_side_is_left() {
        // a sheet with two sides a millimetre apart; a face of its upper side that has sunk below the lower one
        // is nearest a face that looks down, and still looks up as it did
        let mut sheet = square();
        let base = sheet.verts.len() as u32;
        sheet.verts.extend(
            square()
                .verts
                .iter()
                .map(|v| V3::new(v.x, v.y, v.z - 0.001)),
        );
        sheet.push_face(&[base, base + 2, base + 1], 0, false);
        sheet.push_face(&[base, base + 3, base + 2], 0, false);
        let reference = surface(&sheet);
        let mut sunk = square();
        for v in &mut sunk.verts {
            v.z -= 0.002;
        }
        let up = [0.0, 0.0, 1.0];
        assert_eq!(turn_back(&mut sunk, &[up, up], &reference), 0);
        // (one that looked down at first and looks up now is turned)
        let down = [0.0, 0.0, -1.0];
        assert_eq!(turn_back(&mut sunk, &[down, down], &reference), 2);
    }

    #[test]
    fn the_nearest_of_a_surface_is_found() {
        let s = surface(&square());
        assert_eq!(s.distance([0.5, 0.5, 3.0]), Some(2.0));
        assert_eq!(s.distance([2.0, 0.5, 1.0]), Some(1.0));
        assert_eq!(s.within([0.5, 0.5, 1.0], 1e-9), vec![0, 1]);
        assert_eq!(s.within([0.9, 0.1, 1.5], 0.6), vec![0]);
        assert!(Surface::new(Vec::new()).distance([0.0; 3]).is_none());
    }
}
