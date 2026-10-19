//! Catmull-Clark subdivision of a closed polygon cage, as sparse linear operators (positions,
//! skin weights and any other per-vertex attribute go through the same matrices):
//!
//! - `subdivision(faces, n)`: `S.m` maps the cage to level 1, `S.quads` are level 1's faces;
//! - `limit_matrix(quads, n1)`: level 1 to its limit surface (every face a quad).
//!
//! Level-1 vertices are ordered: the cage's vertex points, then edge points, then face points, so
//! the first `n` of them are the cage's own vertices. Faces keep the cage's winding. (subdiv.py's
//! `fit` had no callers and is not ported: body.rs solves for the cage itself.)

use std::collections::{BTreeSet, HashMap};

use super::V3;
use super::sparse::Csr;

pub struct Subdivision {
    /// (n1, n): cage -> level 1.
    pub m: Csr,
    /// Level-1 faces.
    pub quads: Vec<[usize; 4]>,
    /// Cage edges (edge point k is level-1 vertex n + k).
    pub edges: Vec<(usize, usize)>,
    pub n: usize,
}

/// Undirected cage edges, and per face its edge ids in order (edge i joins corner i and i + 1).
fn edges_of(faces: &[Vec<usize>]) -> (Vec<(usize, usize)>, Vec<Vec<usize>>) {
    let mut key: HashMap<(usize, usize), usize> = HashMap::new();
    let mut edges = Vec::new();
    let mut face_edges = Vec::new();
    for f in faces {
        let mut ids = Vec::new();
        for i in 0..f.len() {
            let (a, b) = (f[i], f[(i + 1) % f.len()]);
            let k = if a < b { (a, b) } else { (b, a) };
            let e = *key.entry(k).or_insert_with(|| {
                edges.push(k);
                edges.len() - 1
            });
            ids.push(e);
        }
        face_edges.push(ids);
    }
    (edges, face_edges)
}

/// Every edge is shared by exactly two faces, in opposite directions (a closed, oriented 2-manifold).
pub fn check_closed(faces: &[Vec<usize>], n: usize) -> Result<(), String> {
    let mut seen: HashMap<(usize, usize), usize> = HashMap::new();
    let mut order = Vec::new();
    for (fi, f) in faces.iter().enumerate() {
        for i in 0..f.len() {
            let (a, b) = (f[i], f[(i + 1) % f.len()]);
            if let Some(&g) = seen.get(&(a, b)) {
                return Err(format!("edge {a}-{b} is used twice in the same direction (faces {g}, {fi})"));
            }
            seen.insert((a, b), fi);
            order.push((a, b));
        }
    }
    for (a, b) in order {
        if !seen.contains_key(&(b, a)) {
            return Err(format!("edge {a}-{b} of face {} is open", seen[&(a, b)]));
        }
    }
    let mut used = vec![false; n];
    for f in faces {
        for &v in f {
            used[v] = true;
        }
    }
    let unused: Vec<usize> = (0..n).filter(|&v| !used[v]).take(10).collect();
    if !unused.is_empty() {
        return Err(format!("unused cage vertices: {unused:?}"));
    }
    Ok(())
}

/// Faces wound consistently (flood fill over shared edges), counter-clockwise from outside.
pub fn orient(faces: &[Vec<usize>], pos: &[V3]) -> Vec<Vec<usize>> {
    let mut faces: Vec<Vec<usize>> = faces.to_vec();
    let mut owner: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, f) in faces.iter().enumerate() {
        for i in 0..f.len() {
            let (a, b) = (f[i], f[(i + 1) % f.len()]);
            owner.entry((a.min(b), a.max(b))).or_default().push(fi);
        }
    }
    let mut done = vec![false; faces.len()];
    for seed in 0..faces.len() {
        if done[seed] {
            continue;
        }
        done[seed] = true;
        let mut stack = vec![seed];
        let mut comp = vec![seed];
        while let Some(fi) = stack.pop() {
            let f = faces[fi].clone();
            for i in 0..f.len() {
                let (a, b) = (f[i], f[(i + 1) % f.len()]);
                for &g in &owner[&(a.min(b), a.max(b))] {
                    if done[g] {
                        continue;
                    }
                    let h = &faces[g];
                    // A neighbour must use the shared edge the other way round.
                    let same = (0..h.len()).any(|j| h[j] == a && h[(j + 1) % h.len()] == b);
                    if same {
                        faces[g].reverse();
                    }
                    done[g] = true;
                    stack.push(g);
                    comp.push(g);
                }
            }
        }
        // Outward: positive signed volume.
        let mut vol = 0.0;
        for &fi in &comp {
            let f = &faces[fi];
            let p0 = pos[f[0]];
            for j in 1..f.len() - 1 {
                vol += p0.dot(&pos[f[j]].cross(&pos[f[j + 1]]));
            }
        }
        if vol < 0.0 {
            for &fi in &comp {
                faces[fi].reverse();
            }
        }
    }
    faces
}

/// One Catmull-Clark step of a closed cage of `n` vertices (faces: lists of vertex ids).
pub fn subdivision(faces: &[Vec<usize>], n: usize) -> Subdivision {
    let (edges, face_edges) = edges_of(faces);
    let (ne, nf) = (edges.len(), faces.len());
    let mut t: Vec<(usize, usize, f64)> = Vec::new();
    // Face points: centroids.
    let fp0 = n + ne;
    for (fi, f) in faces.iter().enumerate() {
        for &v in f {
            t.push((fp0 + fi, v, 1.0 / f.len() as f64));
        }
    }
    // Edge points: (a + b + the two face points) / 4, written in cage terms.
    let mut edge_faces = vec![Vec::new(); ne];
    for (fi, ids) in face_edges.iter().enumerate() {
        for &e in ids {
            edge_faces[e].push(fi);
        }
    }
    for (e, &(a, b)) in edges.iter().enumerate() {
        let fs = &edge_faces[e];
        assert_eq!(fs.len(), 2, "open or non-manifold edge {a}-{b}");
        t.push((n + e, a, 0.25));
        t.push((n + e, b, 0.25));
        for &fi in fs {
            let f = &faces[fi];
            for &v in f {
                t.push((n + e, v, 0.25 / f.len() as f64));
            }
        }
    }
    // Vertex points: (F + 2R + (k - 3) P) / k.
    let mut vert_faces = vec![Vec::new(); n];
    for (fi, f) in faces.iter().enumerate() {
        for &v in f {
            vert_faces[v].push(fi);
        }
    }
    let mut vert_edges = vec![Vec::new(); n];
    for (e, &(a, b)) in edges.iter().enumerate() {
        vert_edges[a].push(e);
        vert_edges[b].push(e);
    }
    for v in 0..n {
        let k = vert_edges[v].len();
        assert!(k >= 3 && vert_faces[v].len() == k, "vertex {v}: valence {k}, {} faces", vert_faces[v].len());
        let kf = k as f64;
        t.push((v, v, (kf - 3.0) / kf));
        for &e in &vert_edges[v] {
            let (a, b) = edges[e];
            t.push((v, a, 2.0 / kf * 0.5 / kf));
            t.push((v, b, 2.0 / kf * 0.5 / kf));
        }
        for &fi in &vert_faces[v] {
            let f = &faces[fi];
            for &u in f {
                t.push((v, u, 1.0 / kf / kf / f.len() as f64));
            }
        }
    }
    let m = Csr::from_triplets(n + ne + nf, n, &t);
    // Level-1 quads: corner, next edge point, face point, previous edge point.
    let mut quads = Vec::new();
    for (fi, f) in faces.iter().enumerate() {
        let ids = &face_edges[fi];
        let mm = f.len();
        for i in 0..mm {
            quads.push([f[i], n + ids[i], fp0 + fi, n + ids[(i + mm - 1) % mm]]);
        }
    }
    Subdivision { m, quads, edges, n }
}

/// Limit positions of an all-quad mesh: (k^2 v + 4 sum(edge neighbours) + sum(diagonals)) / (k (k + 5)).
pub fn limit_matrix(quads: &[[usize; 4]], n: usize) -> Csr {
    let mut nbr = vec![BTreeSet::new(); n];
    let mut diag = vec![Vec::new(); n];
    for q in quads {
        for i in 0..4 {
            let v = q[i];
            nbr[v].insert(q[(i + 1) % 4]);
            nbr[v].insert(q[(i + 3) % 4]);
            diag[v].push(q[(i + 2) % 4]);
        }
    }
    let mut t = Vec::new();
    for v in 0..n {
        let k = nbr[v].len();
        let d = (k * (k + 5)) as f64;
        t.push((v, v, (k * k) as f64 / d));
        for &u in &nbr[v] {
            t.push((v, u, 4.0 / d));
        }
        for &u in &diag[v] {
            t.push((v, u, 1.0 / d));
        }
    }
    Csr::from_triplets(n, n, &t)
}

/// Quads split along their shorter diagonal (counter-clockwise kept).
pub fn triangles(quads: &[[usize; 4]], pos: &[V3]) -> Vec<[usize; 3]> {
    let mut out = Vec::with_capacity(quads.len() * 2);
    for q in quads {
        let d02 = (pos[q[0]] - pos[q[2]]).norm();
        let d13 = (pos[q[1]] - pos[q[3]]).norm();
        if d02 <= d13 {
            out.push([q[0], q[1], q[2]]);
            out.push([q[0], q[2], q[3]]);
        } else {
            out.push([q[1], q[2], q[3]]);
            out.push([q[1], q[3], q[0]]);
        }
    }
    out
}

/// Fan triangles of polygons (for the cage-level mesh).
pub fn polygon_triangles(faces: &[Vec<usize>]) -> Vec<[usize; 3]> {
    let mut out = Vec::new();
    for f in faces {
        for j in 1..f.len() - 1 {
            out.push([f[0], f[j], f[j + 1]]);
        }
    }
    out
}
