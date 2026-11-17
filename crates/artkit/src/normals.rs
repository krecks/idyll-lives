//! Shading normals. A corner's normal is its face's on a flat face; on a smooth one, the mean of the faces round
//! its vertex that it joins without crossing a sharp edge (`Mesh::sharp`, a flat face, an open edge), each weighed
//! by its angle there. A vertex's normal is that mean over all its faces (what the baked shading looks along).

use crate::math::{V3, dot_v3, normalize_v3};
use crate::mesh::{Mesh, edge};
use std::collections::BTreeMap;

/// The arc cosine as Blender approximates it for a vertex's normal (to a few hundred-thousandths of a radian).
fn acos_approx(x: f32) -> f32 {
    let f = x.abs();
    let m = if f < 1.0 { 1.0 - (1.0 - f) } else { 1.0 };
    let a = (1.0 - m).sqrt()
        * (std::f32::consts::FRAC_PI_2
            + m * (-0.213_300_99 + m * (0.077_980_48 + m * -0.021_640_95)));
    if x < 0.0 { std::f32::consts::PI - a } else { a }
}

/// Each corner's angle weight: (face, place among the face's corners) in the mesh's corner order.
fn corner_angles(mesh: &Mesh, approx: bool) -> Vec<f32> {
    let mut out = Vec::with_capacity(mesh.loops.len());
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        let n = c.len();
        for i in 0..n {
            let p = mesh.verts[c[i] as usize];
            let prev = normalize_v3(mesh.verts[c[(i + n - 1) % n] as usize] - p).0;
            let next = normalize_v3(mesh.verts[c[(i + 1) % n] as usize] - p).0;
            let d = dot_v3(prev, next);
            out.push(if approx {
                acos_approx(d)
            } else {
                libm::acos(d.clamp(-1.0, 1.0) as f64) as f32
            });
        }
    }
    out
}

/// A normal a vertex: its faces' normals (or its corners' own, where the mesh has them) weighed by their angles
/// at it (a vertex of no face: its direction).
pub fn vertex_normals(mesh: &Mesh) -> Vec<V3> {
    let face_normals = mesh.face_normals();
    let angles = corner_angles(mesh, true);
    // (a mesh with normals of its own at its corners, a bevel's: Blender means those round each vertex instead)
    let own = mesh.normals.as_ref();
    let mut sum = vec![V3::ZERO; mesh.verts.len()];
    let mut at = 0;
    for (f, normal) in face_normals.iter().enumerate() {
        for &v in mesh.corners(f) {
            let k = angles[at];
            let normal = own.map_or(*normal, |n| n[at]);
            let s = &mut sum[v as usize];
            *s = V3::new(s.x + normal.x * k, s.y + normal.y * k, s.z + normal.z * k);
            at += 1;
        }
    }
    sum.iter()
        .zip(&mesh.verts)
        .map(|(s, p)| {
            let (n, len) = normalize_v3(*s);
            if len == 0.0 { normalize_v3(*p).0 } else { n }
        })
        .collect()
}

/// A normal a corner, as the file is written with: the mesh's own where it has them (`Mesh::normals`).
pub fn corner_normals(mesh: &Mesh) -> Vec<V3> {
    if let Some(n) = &mesh.normals {
        return n.clone();
    }
    let face_normals = mesh.face_normals();
    let angles = corner_angles(mesh, false);
    let nl = mesh.loops.len();
    // the corners on each edge: (corner, whether the face runs the edge from its lower vertex up)
    let mut on_edge: BTreeMap<(u32, u32), Vec<(u32, bool)>> = BTreeMap::new();
    let mut face_of = vec![0u32; nl];
    for f in 0..mesh.faces.len() {
        let face = mesh.faces[f];
        let c = mesh.corners(f);
        for (i, &v) in c.iter().enumerate() {
            let l = face.start + i as u32;
            face_of[l as usize] = f as u32;
            let w = c[(i + 1) % c.len()];
            on_edge.entry(edge(v, w)).or_default().push((l, v < w));
        }
    }
    // corners of one vertex joined across each smooth edge
    let mut group: Vec<u32> = (0..nl as u32).collect();
    fn find(g: &mut [u32], mut a: u32) -> u32 {
        while g[a as usize] != a {
            g[a as usize] = g[g[a as usize] as usize];
            a = g[a as usize];
        }
        a
    }
    for (e, on) in &on_edge {
        if on.len() != 2 || mesh.sharp.contains(e) || on[0].1 == on[1].1 {
            continue;
        }
        let (fa, fb) = (
            face_of[on[0].0 as usize] as usize,
            face_of[on[1].0 as usize] as usize,
        );
        if !mesh.faces[fa].smooth || !mesh.faces[fb].smooth {
            continue;
        }
        // the edge's two ends: at each, the corner of one face and the corner of the other there
        let next = |l: u32| {
            let face = mesh.faces[face_of[l as usize] as usize];
            face.start + (l - face.start + 1) % face.len
        };
        let (a, b) = (on[0].0, on[1].0);
        for (x, y) in [(a, next(b)), (next(a), b)] {
            let (rx, ry) = (find(&mut group, x), find(&mut group, y));
            if rx != ry {
                group[rx as usize] = ry;
            }
        }
    }
    let mut sum = vec![V3::ZERO; nl];
    for l in 0..nl {
        let f = face_of[l] as usize;
        let r = find(&mut group, l as u32) as usize;
        let (n, k) = (face_normals[f], angles[l]);
        sum[r] = V3::new(sum[r].x + n.x * k, sum[r].y + n.y * k, sum[r].z + n.z * k);
    }
    (0..nl)
        .map(|l| {
            let f = face_of[l] as usize;
            if !mesh.faces[f].smooth {
                return face_normals[f];
            }
            let r = find(&mut group, l as u32) as usize;
            let (n, len) = normalize_v3(sum[r]);
            if len == 0.0 { face_normals[f] } else { n }
        })
        .collect()
}
