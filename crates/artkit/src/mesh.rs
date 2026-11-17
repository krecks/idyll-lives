//! A mesh as Blender keeps one: single-precision points, polygons of any number of corners, a smooth flag a face
//! and a sharp flag an edge, and what is stored at its corners and points (UVs, shading). The operations a build
//! needs are Blender's, down to which corner a face starts at: that decides the diagonal a quad is drawn and hit by.

use crate::materials::Mat;
use crate::math::{M4, V3, dot_v3, len_v3, mul_m4_v3, mul_v3_fl, normalize_v3};
use std::collections::{BTreeMap, BTreeSet};

/// One polygon: its corners are `loops[start..start + len]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    pub start: u32,
    pub len: u32,
    /// Which of the mesh's materials it wears.
    pub mat: u16,
    pub smooth: bool,
}

/// How a part's UVs are made when it is joined (`uv`): a box per face, round about its own z, or made already.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum UvWay {
    #[default]
    Box,
    Round,
    Done,
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub verts: Vec<V3>,
    pub faces: Vec<Face>,
    /// Each corner's vertex.
    pub loops: Vec<u32>,
    /// The edges kept sharp on a smooth mesh, each as (lower vertex, higher vertex).
    pub sharp: BTreeSet<(u32, u32)>,
    /// A normal a corner where they are given and not worked out (leaf cards, bevels).
    pub normals: Option<Vec<V3>>,
    /// A UV a corner, in metres, once made.
    pub uvs: Option<Vec<[f32; 2]>>,
    /// Whether the UVs are final (made in the part's own frame before it was joined).
    pub uv_metres: bool,
    /// A leaf card's own shading at each vertex (0: not a card's).
    pub card_shade: Option<Vec<f32>>,
    /// The baked shading at each vertex, 0..1, once baked.
    pub ao: Option<Vec<f32>>,
    pub mats: Vec<Mat>,
}

/// An edge's key: its two vertices, the lower first.
pub fn edge(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

impl Mesh {
    /// `mesh.from_pydata(verts, [], faces)`.
    pub fn from_pydata(verts: Vec<V3>, faces: &[Vec<u32>]) -> Mesh {
        let mut m = Mesh {
            verts,
            ..Mesh::default()
        };
        for f in faces {
            m.push_face(f, 0, false);
        }
        m
    }

    pub fn push_face(&mut self, corners: &[u32], mat: u16, smooth: bool) {
        self.faces.push(Face {
            start: self.loops.len() as u32,
            len: corners.len() as u32,
            mat,
            smooth,
        });
        self.loops.extend_from_slice(corners);
    }

    /// A face's corners' vertices.
    pub fn corners(&self, f: usize) -> &[u32] {
        let face = &self.faces[f];
        &self.loops[face.start as usize..(face.start + face.len) as usize]
    }

    fn points(&self, f: usize) -> Vec<V3> {
        self.corners(f)
            .iter()
            .map(|&v| self.verts[v as usize])
            .collect()
    }

    /// The slot of `mat` among the mesh's materials, added if it is new.
    pub fn slot(&mut self, mat: Mat) -> u16 {
        match self.mats.iter().position(|m| m.name == mat.name) {
            Some(i) => i as u16,
            None => {
                self.mats.push(mat);
                (self.mats.len() - 1) as u16
            }
        }
    }

    /// `mesh.materials.append(material)`: every face is on the first.
    pub fn set_material(&mut self, mat: Option<Mat>) {
        if let Some(mat) = mat {
            self.slot(mat);
        }
    }

    /// A face's normal as Blender works it out: a triangle's from two sides, a quad's from its diagonals, any
    /// other's by Newell's sum.
    pub fn face_normal(&self, f: usize) -> V3 {
        normal_of(&self.points(f))
    }

    pub fn face_normals(&self) -> Vec<V3> {
        (0..self.faces.len()).map(|f| self.face_normal(f)).collect()
    }

    /// `mesh.transform(matrix)`: every point through it, single precision; the corners' own normals (a
    /// bevel's) turn with it.
    pub fn transform(&mut self, m: &M4) {
        for v in &mut self.verts {
            *v = mul_m4_v3(m, *v);
        }
        if let Some(normals) = &mut self.normals {
            let a = m.0;
            // (a normal goes through the matrix's cofactors: its inverse transposed, but for the length)
            let cof = |r: usize, c: usize| {
                let (r1, r2, c1, c2) = ((r + 1) % 3, (r + 2) % 3, (c + 1) % 3, (c + 2) % 3);
                a[c1][r1] * a[c2][r2] - a[c2][r1] * a[c1][r2]
            };
            let k: [[f32; 3]; 3] = std::array::from_fn(|r| std::array::from_fn(|c| cof(r, c)));
            if k != [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
                for n in normals {
                    let turned = V3::new(
                        k[0][0] * n.x + k[0][1] * n.y + k[0][2] * n.z,
                        k[1][0] * n.x + k[1][1] * n.y + k[1][2] * n.z,
                        k[2][0] * n.x + k[2][1] * n.y + k[2][2] * n.z,
                    );
                    *n = normalize_v3(turned).0;
                }
            }
        }
    }

    /// Every point moved by `by`.
    pub fn translate(&mut self, by: V3) {
        self.transform(&M4::translation(by));
    }

    /// `mesh.flip_normals()`: every face wound the other way, from the corner it started at.
    pub fn flip_normals(&mut self) {
        for f in 0..self.faces.len() {
            self.flip_face(f);
        }
    }

    fn flip_face(&mut self, f: usize) {
        let face = self.faces[f];
        let (a, b) = (face.start as usize + 1, (face.start + face.len) as usize);
        self.loops[a..b].reverse();
        if let Some(uvs) = &mut self.uvs {
            uvs[a..b].reverse();
        }
        if let Some(n) = &mut self.normals {
            n[a..b].reverse();
        }
    }

    /// `mesh.shade_flat()`.
    pub fn shade_flat(&mut self) {
        for f in &mut self.faces {
            f.smooth = false;
        }
    }

    /// `mesh.shade_smooth()` then `mesh.set_sharp_from_angle(angle)`: smooth, but for the edges between two faces
    /// that turn more than `degrees` as the mesh is now (a half turn or more: none).
    pub fn shade_smooth(&mut self, degrees: f64) {
        for f in &mut self.faces {
            f.smooth = true;
        }
        self.sharp.clear();
        let angle = crate::math::radians(degrees) as f32;
        if angle as f64 >= crate::math::PI {
            return;
        }
        let cos = libm::cos(angle as f64) as f32;
        let normals = self.face_normals();
        // each edge's first face, then whether a second or third came
        let mut first: BTreeMap<(u32, u32), (usize, u32, bool)> = BTreeMap::new();
        for f in 0..self.faces.len() {
            let c = self.corners(f).to_vec();
            for (i, &v) in c.iter().enumerate() {
                let e = edge(v, c[(i + 1) % c.len()]);
                match first.get_mut(&e) {
                    None => {
                        first.insert(e, (f, v, false));
                    }
                    Some((g, _, done)) if !*done => {
                        if dot_v3(normals[*g], normals[f]) < cos {
                            self.sharp.insert(e);
                        }
                        *done = true;
                    }
                    // a third face on the edge: it splits the shading anyway, and isn't marked
                    Some(_) => {
                        self.sharp.remove(&e);
                    }
                }
            }
        }
    }

    /// The mesh's triangles as Blender draws and casts rays at them: (face, its three corners' places among the
    /// face's corners). A quad is cut from its first corner to its third, unless that folds it; a polygon of
    /// more by clipping ears.
    pub fn triangles(&self) -> Vec<(u32, [u32; 3])> {
        let mut out = Vec::with_capacity(self.loops.len());
        for f in 0..self.faces.len() {
            let p = self.points(f);
            match p.len() {
                0..=2 => {}
                3 => out.push((f as u32, [0, 1, 2])),
                4 => {
                    if quad_flips(p[0], p[1], p[2], p[3]) {
                        out.push((f as u32, [0, 1, 3]));
                        out.push((f as u32, [1, 2, 3]));
                    } else {
                        out.push((f as u32, [0, 1, 2]));
                        out.push((f as u32, [0, 2, 3]));
                    }
                }
                _ => out.extend(
                    crate::tri::polygon(&p, normal_of(&p))
                        .into_iter()
                        .map(|t| (f as u32, t)),
                ),
            }
        }
        out
    }

    /// The triangles as vertices.
    pub fn triangle_verts(&self) -> Vec<[u32; 3]> {
        self.triangles()
            .into_iter()
            .map(|(f, t)| {
                let c = self.corners(f as usize);
                [c[t[0] as usize], c[t[1] as usize], c[t[2] as usize]]
            })
            .collect()
    }

    /// `normals_make_consistent(inside=False)`: each connected piece's faces wound one way, the way that has the
    /// face at its outermost point look outwards.
    pub fn recalc_normals(&mut self) {
        let nf = self.faces.len();
        // the faces on each edge, with whether the face runs the edge from its lower vertex to its higher
        let mut on_edge: BTreeMap<(u32, u32), Vec<(usize, bool)>> = BTreeMap::new();
        for f in 0..nf {
            let c = self.corners(f);
            for (i, &v) in c.iter().enumerate() {
                let w = c[(i + 1) % c.len()];
                on_edge.entry(edge(v, w)).or_default().push((f, v < w));
            }
        }
        let mut group = vec![usize::MAX; nf];
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for f0 in 0..nf {
            if group[f0] != usize::MAX {
                continue;
            }
            let g = groups.len();
            let mut faces = vec![f0];
            group[f0] = g;
            let mut at = 0;
            while at < faces.len() {
                let f = faces[at];
                at += 1;
                let c = self.corners(f).to_vec();
                for (i, &v) in c.iter().enumerate() {
                    let on = &on_edge[&edge(v, c[(i + 1) % c.len()])];
                    if on.len() == 2 {
                        for &(o, _) in on {
                            if group[o] == usize::MAX {
                                group[o] = g;
                                faces.push(o);
                            }
                        }
                    }
                }
            }
            faces.sort_unstable();
            groups.push(faces);
        }
        for faces in groups {
            let (start, flip) = self.outermost(&faces);
            // walk out from the start face: a neighbour running the shared edge the same way is the other way up
            let mut turned: BTreeMap<usize, bool> = BTreeMap::new();
            turned.insert(start, flip);
            let mut stack = vec![start];
            while let Some(f) = stack.pop() {
                let c = self.corners(f).to_vec();
                for (i, &v) in c.iter().enumerate() {
                    let w = c[(i + 1) % c.len()];
                    let on = &on_edge[&edge(v, w)];
                    if on.len() != 2 {
                        continue;
                    }
                    let (o, o_way) = if on[0].0 == f && on[1].0 != f {
                        on[1]
                    } else {
                        on[0]
                    };
                    if o == f || turned.contains_key(&o) {
                        continue;
                    }
                    let same_way = o_way == (v < w);
                    turned.insert(o, turned[&f] != same_way);
                    stack.push(o);
                }
            }
            for (f, flip) in turned {
                if flip {
                    self.flip_face(f);
                }
            }
        }
    }

    /// Of a connected piece's `faces`: the one at the corner farthest from the piece's middle, and whether it
    /// looks inwards there.
    fn outermost(&self, faces: &[usize]) -> (usize, bool) {
        let eps = f32::EPSILON;
        let k = 1.0 / faces.len() as f32;
        let (mut cent, mut area_sum) = (V3::ZERO, 0.0_f32);
        for &f in faces {
            let p = self.points(f);
            let area = area_of(&p);
            cent += mul_v3_fl(centre_weighted(&p), k * area);
            area_sum += area;
        }
        if area_sum != 0.0 {
            cent = mul_v3_fl(cent, 1.0 / area_sum);
        }
        let (mut best_dist, mut best_edge, mut best_face) = (eps, f32::MIN, f32::MIN);
        let (mut start, mut flip) = (faces[0], false);
        for &f in faces {
            let p = self.points(f);
            let n = p.len();
            for i in 0..n {
                let dir = p[i] - cent;
                let dist = dot_v3(dir, dir);
                let far = dist > best_dist;
                if !(far || dist == best_dist) {
                    continue;
                }
                let dir = mul_v3_fl(dir, 1.0 / dist.sqrt());
                let (e0, l0) = normalize_v3(p[(i + 1) % n] - p[i]);
                let (e1, l1) = normalize_v3(p[(i + n - 1) % n] - p[i]);
                if !(l0 > eps && l1 > eps) {
                    continue;
                }
                let edge_dot = dot_v3(dir, e0).max(dot_v3(dir, e1));
                let better_edge = edge_dot > best_edge;
                if !(far || better_edge || edge_dot == best_edge) {
                    continue;
                }
                let (cross, lc) = normalize_v3(e0.cross(e1));
                if lc > eps {
                    let face_dot = dot_v3(dir, cross).abs();
                    if far || better_edge || face_dot > best_face {
                        (best_dist, best_edge, best_face) = (dist, edge_dot, face_dot);
                        start = f;
                        flip = dot_v3(cross, dir) < 0.0;
                    }
                }
            }
        }
        (start, flip)
    }

    /// `bmesh.ops.remove_doubles(verts=all, dist)`: the points within `dist` of one another become one, and the
    /// faces that lose a side by it lose a corner (or go, under three).
    pub fn remove_doubles(&mut self, dist: f32) {
        let target = crate::weld::doubles(&self.verts, dist);
        if target
            .iter()
            .enumerate()
            .all(|(i, &t)| t < 0 || t as usize == i)
        {
            return;
        }
        self.weld(&target);
    }

    /// Merges each vertex with a `target` other than itself (or none: -1) into it.
    pub(crate) fn weld(&mut self, target: &[i32]) {
        let to = |v: u32| {
            if target[v as usize] >= 0 {
                target[v as usize] as u32
            } else {
                v
            }
        };
        // the vertices left, in order
        let mut new_index = vec![u32::MAX; self.verts.len()];
        let mut verts = Vec::new();
        for (i, v) in self.verts.iter().enumerate() {
            if to(i as u32) == i as u32 {
                new_index[i] = verts.len() as u32;
                verts.push(*v);
            }
        }
        let mut out = Mesh {
            verts,
            mats: self.mats.clone(),
            ..Mesh::default()
        };
        let mut seen: BTreeSet<Vec<u32>> = BTreeSet::new();
        for f in 0..self.faces.len() {
            if self.corners(f).iter().all(|&v| to(v) == v) {
                let mut key: Vec<u32> = self
                    .corners(f)
                    .iter()
                    .map(|&v| new_index[v as usize])
                    .collect();
                key.sort_unstable();
                seen.insert(key);
            }
        }
        for f in 0..self.faces.len() {
            let c: Vec<u32> = self.corners(f).iter().map(|&v| to(v)).collect();
            let changed = self.corners(f).iter().zip(&c).any(|(a, b)| a != b);
            // a corner whose side has closed up goes: the one before the side
            let kept: Vec<u32> = (0..c.len())
                .filter(|&i| c[i] != c[(i + 1) % c.len()])
                .map(|i| new_index[c[i] as usize])
                .collect();
            let distinct: BTreeSet<u32> = kept.iter().copied().collect();
            if kept.len() < 3 || distinct.len() != kept.len() {
                continue;
            }
            if changed {
                // (one that has become another face's double goes too)
                let mut key = kept.clone();
                key.sort_unstable();
                if !seen.insert(key) {
                    continue;
                }
            }
            out.push_face(&kept, self.faces[f].mat, self.faces[f].smooth);
        }
        *self = out;
    }

    /// The lowest and highest of each coordinate, through `m`.
    pub fn bounds(&self, m: &M4) -> Option<([f32; 3], [f32; 3])> {
        let mut it = self
            .verts
            .iter()
            .map(|v| if m.is_identity() { *v } else { *m * *v });
        let first = it.next()?;
        let (mut lo, mut hi) = (first.array(), first.array());
        for p in it {
            for i in 0..3 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        Some((lo, hi))
    }
}

/// A polygon's normal as Blender works it out.
pub fn normal_of(p: &[V3]) -> V3 {
    match p.len() {
        3 => normal_tri(p[0], p[1], p[2]),
        4 => normalize_v3((p[0] - p[2]).cross(p[1] - p[3])).0,
        _ => {
            let n = newell(p);
            let (unit, len) = normalize_v3(n);
            if len == 0.0 { V3::Z } else { unit }
        }
    }
}

/// A triangle's normal.
pub fn normal_tri(a: V3, b: V3, c: V3) -> V3 {
    normalize_v3((a - b).cross(b - c)).0
}

/// A polygon's normal by Newell's sum, whatever its corners (straight up for one of no area).
pub fn normal_newell(p: &[V3]) -> V3 {
    let (unit, len) = normalize_v3(newell(p));
    if len == 0.0 { V3::Z } else { unit }
}

fn newell(p: &[V3]) -> V3 {
    let mut n = V3::ZERO;
    let mut prev = p[p.len() - 1];
    for &cur in p {
        n.x += (prev.y - cur.y) * (prev.z + cur.z);
        n.y += (prev.z - cur.z) * (prev.x + cur.x);
        n.z += (prev.x - cur.x) * (prev.y + cur.y);
        prev = cur;
    }
    n
}

fn area_of(p: &[V3]) -> f32 {
    let mut n = V3::ZERO;
    for (i, &cur) in p.iter().enumerate() {
        let next = p[(i + 1) % p.len()];
        n.x += (cur.y - next.y) * (cur.z + next.z);
        n.y += (cur.z - next.z) * (cur.x + next.x);
        n.z += (cur.x - next.x) * (cur.y + next.y);
    }
    len_v3(n) * 0.5
}

/// A polygon's middle, each corner weighed by the length of its two sides.
fn centre_weighted(p: &[V3]) -> V3 {
    let n = p.len();
    let (mut c, mut total) = (V3::ZERO, 0.0_f32);
    let mut w_prev = len_v3(p[n - 1] - p[0]);
    for i in 0..n {
        let w_cur = len_v3(p[i] - p[(i + 1) % n]);
        let w = w_cur + w_prev;
        c += mul_v3_fl(p[i], w);
        total += w;
        w_prev = w_cur;
    }
    if total != 0.0 {
        mul_v3_fl(c, 1.0 / total)
    } else {
        c
    }
}

/// Whether a quad cut from its first corner to its third would fold over (it is then cut the other way): its
/// second and fourth corners on one side of that cut.
fn quad_flips(v1: V3, v2: V3, v3: V3, v4: V3) -> bool {
    let d13 = v3 - v1;
    dot_v3((v2 - v1).cross(d13), (v4 - v1).cross(d13)) > 0.0
}
