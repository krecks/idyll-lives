//! Cutting faces into triangles, as Blender's polyfill does.

use crate::math::{V3, dot_v3};
use crate::mesh::{Mesh, edge};
use crate::polyfill::{fill, flat, ortho_basis, split_24, turn_gain};
use crate::scene::Obj;

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
        turn_gain(
            coords[v[a_other] as usize],
            coords[v[e] as usize],
            coords[v[b_other] as usize],
            coords[v[b] as usize],
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
                    QuadMethod::Beauty => split_24(&p),
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
                let mut tris = fill(&coords);
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
