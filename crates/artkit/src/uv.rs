//! UVs for the textured finishes (wood, fabric, bark, foliage), in metres, made for each part in its own frame
//! before it is joined (`parts::join`), so a texture follows the part: the wood's grain (up its picture, along v)
//! runs along a leg, a rail or a table's length, not along whichever axis of the model each face happens to face;
//! bark runs up a trunk and round it. The game tiles them by the finish's `uvScale`.
//!
//! A part's way is how it was made (`UvWay`): a box per face along its own longest sides (the default), round
//! about its own z (cylinders, cones, lathes), or along its path (a sweep, which writes its own). Parts on plain
//! finishes get none of their own; joined with textured parts, theirs are zero (one UV map for the mesh).

use crate::math::{PI, TAU, V3, atan2};
use crate::mesh::{Mesh, UvWay};
use crate::py;
use crate::scene::{self, Obj};

/// For each of `obj`'s material slots, whether its material is on a textured finish.
fn slots(obj: Obj) -> Vec<bool> {
    scene::with(|s| {
        s.get(obj)
            .mesh
            .mats
            .iter()
            .map(|m| s.finishes.get(m.finish).is_some_and(|f| f.textured))
            .collect()
    })
}

/// Whether any of `obj`'s materials is on a textured finish.
pub fn textured(obj: Obj) -> bool {
    slots(obj).iter().any(|&t| t)
}

/// Gives each of `objs` (parts about to be joined, or a model's meshes as written) its UVs, once: in metres in
/// its own frame if it is textured, zero if it isn't but one of the others is; none if none of them is. Returns
/// whether any is textured.
pub fn ensure(objs: &[Obj]) -> bool {
    if !objs.iter().any(|&o| textured(o)) {
        return false;
    }
    for &o in objs {
        let slots = slots(o);
        o.edit(|part| {
            if part.mesh.uv_metres {
                return;
            }
            let mesh = &mut part.mesh;
            if mesh.uvs.is_none() {
                mesh.uvs = Some(vec![[0.0, 0.0]; mesh.loops.len()]);
            }
            if slots.iter().any(|&t| t) {
                match part.uv {
                    UvWay::Done => {}
                    UvWay::Round => round(mesh),
                    UvWay::Box => box_(mesh),
                }
            }
            // the faces on plain finishes: nothing reads theirs, and zeros cost next to nothing compressed
            let mut uvs = mesh.uvs.take().unwrap_or_default();
            for face in &mesh.faces {
                if !slots.get(face.mat as usize).copied().unwrap_or(false) {
                    for uv in &mut uvs[face.start as usize..(face.start + face.len) as usize] {
                        *uv = [0.0, 0.0];
                    }
                }
            }
            mesh.uvs = Some(uvs);
            mesh.uv_metres = true;
        });
    }
    true
}

fn extent(co: &[V3], i: usize) -> (f64, f64) {
    let lo = co.iter().map(|c| c[i]).fold(f32::INFINITY, f32::min);
    let hi = co.iter().map(|c| c[i]).fold(f32::NEG_INFINITY, f32::max);
    (lo as f64, hi as f64)
}

/// Per face, the two axes across its normal's largest, v along the longer of them over the whole part (the grain
/// along its length), u along the other. A face keeps one projection for all its corners: no smear.
pub fn box_(mesh: &mut Mesh) {
    if mesh.verts.is_empty() {
        return;
    }
    let ext: Vec<f64> = (0..3)
        .map(|i| {
            let (lo, hi) = extent(&mesh.verts, i);
            hi - lo
        })
        .collect();
    let mut longest = [0usize, 1, 2];
    longest.sort_by(|&a, &b| ext[b].total_cmp(&ext[a]));
    let mut uvs = vec![[0.0_f32; 2]; mesh.loops.len()];
    for f in 0..mesh.faces.len() {
        let n = mesh.face_normal(f);
        let mut d = 0;
        for i in 1..3 {
            if n[i].abs() > n[d].abs() {
                d = i;
            }
        }
        let axes: Vec<usize> = longest.iter().copied().filter(|&i| i != d).collect();
        let (v_axis, u_axis) = (axes[0], axes[1]);
        let face = mesh.faces[f];
        for l in face.start..face.start + face.len {
            let c = mesh.verts[mesh.loops[l as usize] as usize];
            uvs[l as usize] = [c[u_axis], c[v_axis]];
        }
    }
    mesh.uvs = Some(uvs);
}

/// About its own z: round its sides u is the arc (metres at its mean radius), v the height; its caps (faces
/// looking along z) flat. Each face is unwrapped where it crosses the seam behind.
pub fn round(mesh: &mut Mesh) {
    if mesh.verts.is_empty() {
        return;
    }
    let (x0, x1) = extent(&mesh.verts, 0);
    let (y0, y1) = extent(&mesh.verts, 1);
    let (cx, cy) = ((x1 + x0) / 2.0, (y1 + y0) / 2.0);
    let total = py::sum(
        mesh.verts
            .iter()
            .map(|c| py::hypot(c.x as f64 - cx, c.y as f64 - cy)),
    );
    let r = (total / mesh.verts.len() as f64).max(1e-4);
    let mut uvs = vec![[0.0_f32; 2]; mesh.loops.len()];
    for f in 0..mesh.faces.len() {
        let face = mesh.faces[f];
        let corners: Vec<V3> = mesh
            .corners(f)
            .iter()
            .map(|&v| mesh.verts[v as usize])
            .collect();
        let at = face.start as usize;
        if mesh.face_normal(f).z.abs() >= 0.7 {
            for (i, c) in corners.iter().enumerate() {
                uvs[at + i] = [c.x, c.y];
            }
            continue;
        }
        let mut angles: Vec<f64> = corners
            .iter()
            .map(|c| atan2(c.y as f64 - cy, c.x as f64 - cx))
            .collect();
        let (lo, hi) = (
            angles.iter().copied().fold(f64::INFINITY, f64::min),
            angles.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        );
        if hi - lo > PI {
            for a in &mut angles {
                if *a < 0.0 {
                    *a += TAU;
                }
            }
        }
        for (i, (c, a)) in corners.iter().zip(&angles).enumerate() {
            uvs[at + i] = [(a * r) as f32, c.z];
        }
    }
    mesh.uvs = Some(uvs);
}

/// A sweep's UVs (`forms::sweep`): u round its profile (metres, times the ring's scale), v along its path
/// (metres), its caps flat in the profile's own plane. `rings` rings of `profile` (2D), at `points`.
pub fn along(
    obj: Obj,
    rings: usize,
    profile: &[(f64, f64)],
    scales: &[f64],
    points: &[V3],
    closed: bool,
) -> Obj {
    let m = profile.len();
    let mut round_ = vec![0.0_f64];
    for k in 1..=m {
        let (a, b) = (profile[k - 1], profile[k % m]);
        round_.push(round_[k - 1] + py::hypot(b.0 - a.0, b.1 - a.1));
    }
    let mut path = vec![0.0_f64];
    for i in 1..if closed { rings + 1 } else { rings } {
        let (a, b) = (points[i - 1], points[i % rings]);
        path.push(path[i - 1] + (b - a).length());
    }
    obj.edit(|part| {
        let mesh = &mut part.mesh;
        let mut uvs = vec![[0.0_f32; 2]; mesh.loops.len()];
        for f in 0..mesh.faces.len() {
            let at = mesh.faces[f].start as usize;
            let verts: Vec<usize> = mesh.corners(f).iter().map(|&v| v as usize).collect();
            if verts.iter().all(|v| v / m == verts[0] / m) {
                // a cap: flat in the profile's plane
                for (i, v) in verts.iter().enumerate() {
                    let (k, ring) = (v % m, (v / m).min(rings - 1));
                    uvs[at + i] = [
                        (profile[k].0 * scales[ring]) as f32,
                        (profile[k].1 * scales[ring]) as f32,
                    ];
                }
                continue;
            }
            let ks: Vec<usize> = verts.iter().map(|v| v % m).collect();
            let is: Vec<usize> = verts.iter().map(|v| v / m).collect();
            let wrap_k = ks.iter().any(|&k| k == m - 1) && ks.contains(&0);
            let wrap_i = closed && is.iter().any(|&i| i == rings - 1) && is.contains(&0);
            for (j, (&k, &i)) in ks.iter().zip(&is).enumerate() {
                let kk = if wrap_k && k == 0 { m } else { k };
                let ii = if wrap_i && i == 0 { rings } else { i };
                uvs[at + j] = [(round_[kk] * scales[i % rings]) as f32, path[ii] as f32];
            }
        }
        mesh.uvs = Some(uvs);
        part.uv = UvWay::Done;
    });
    obj
}
