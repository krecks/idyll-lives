//! The shape a model is, as its faces and the edges between them.
//!
//! Split from `crates/tools/src/edges.rs` (see `mod.rs`).

use super::*;

impl<'a> Shape<'a> {
    pub(crate) fn new(meshes: &'a [Mesh]) -> Shape<'a> {
        let mut ids: Map<[i64; 3], u32> = Map::default();
        let mut shape = Shape {
            meshes,
            place: Vec::new(),
            faces: Vec::new(),
            edges: Map::default(),
            order: Vec::new(),
        };
        for (m, mesh) in meshes.iter().enumerate() {
            let weld: Vec<u32> = mesh
                .positions
                .iter()
                .map(|p| {
                    let key = p.map(|c| (c / WELD).round() as i64);
                    *ids.entry(key).or_insert_with(|| {
                        shape.place.push(*p);
                        shape.place.len() as u32 - 1
                    })
                })
                .collect();
            for t in &mesh.triangles {
                let Some(v) = t
                    .iter()
                    .map(|&i| weld.get(i as usize).copied())
                    .collect::<Option<Vec<u32>>>()
                else {
                    continue;
                };
                let v = [v[0], v[1], v[2]];
                if v[0] == v[1] || v[1] == v[2] || v[0] == v[2] {
                    continue;
                }
                let [a, b, c] = v.map(|i| shape.place[i as usize]);
                let n = cross(sub(b, a), sub(c, a));
                if length(n) < 1e-12 {
                    continue;
                }
                let face = shape.faces.len() as u32;
                shape.faces.push(Face {
                    mesh: m as u32,
                    v,
                    c: *t,
                    n: unit(n),
                });
                for k in 0..3 {
                    let key = edge_key(v[k], v[(k + 1) % 3]);
                    let sides = shape.edges.entry(key).or_insert_with(|| {
                        shape.order.push(key);
                        Sides::default()
                    });
                    if let Some(slot) = sides.at.get_mut(sides.count as usize) {
                        *slot = (face, k as u8);
                    }
                    sides.count += 1;
                }
            }
        }
        shape
    }

    /// The normal of `face`'s corner `k` as the file has it (the face's own where the mesh has none).
    fn normal(&self, face: &Face, k: usize) -> V3 {
        self.meshes[face.mesh as usize]
            .normals
            .get(face.c[k] as usize)
            .map_or(face.n, |n| unit(*n))
    }

    /// The two faces of an edge as (face, corner the edge starts at), if it has just two wound the same way
    /// round (each runs along it the other way).
    pub(crate) fn pair(&self, sides: &Sides) -> Option<[(usize, usize); 2]> {
        if sides.count != 2 {
            return None;
        }
        let [(a, ka), (b, kb)] = sides.at.map(|(f, k)| (f as usize, k as usize));
        (self.faces[a].v[ka] == self.faces[b].v[(kb + 1) % 3]).then_some([(a, ka), (b, kb)])
    }

    /// Whether the two faces of an edge are split there in the file: their corners' normals `SPLIT` apart at
    /// both of its ends.
    pub(crate) fn split(&self, [(a, ka), (b, kb)]: [(usize, usize); 2]) -> bool {
        let (fa, fb) = (&self.faces[a], &self.faces[b]);
        let apart =
            |i: usize, j: usize| dot(self.normal(fa, i), self.normal(fb, j)) <= SPLIT + SLACK;
        apart(ka, (kb + 1) % 3) && apart((ka + 1) % 3, kb)
    }

    /// Whether a surface ends at an edge: it has not just two faces, or they turn there and are split.
    fn ends(&self, sides: &Sides) -> bool {
        match self.pair(sides) {
            None => true,
            Some(pair) => {
                dot(self.faces[pair[0].0].n, self.faces[pair[1].0].n) < FLAT && self.split(pair)
            }
        }
    }

    /// Whether the surface on `face`'s side of its edge `k` is `limit` wide across the edge's middle: a line drawn
    /// from there straight over the surface (unfolded from triangle to triangle) gets that far before the surface
    /// ends (`ends`).
    pub(crate) fn wide(&self, mut face: usize, mut k: usize, limit: f64) -> bool {
        let f = &self.faces[face];
        let (a, b) = (
            self.place[f.v[k] as usize],
            self.place[f.v[(k + 1) % 3] as usize],
        );
        let mut at = mul(add(a, b), 0.5);
        let mut way = unit(cross(f.n, sub(b, a)));
        let mut gone = 0.0;
        for _ in 0..256 {
            let f = &self.faces[face];
            // Out through one of the triangle's other two edges: (how far, the corner it starts at).
            let mut out: Option<(f64, usize)> = None;
            for j in [(k + 1) % 3, (k + 2) % 3] {
                let p = self.place[f.v[j] as usize];
                let pq = sub(self.place[f.v[(j + 1) % 3] as usize], p);
                let det = dot(cross(way, pq), f.n);
                if det.abs() < 1e-15 {
                    continue;
                }
                let to = sub(p, at);
                let far = dot(cross(to, pq), f.n) / det;
                let on = dot(cross(to, way), f.n) / det;
                if far > 1e-9 && (-1e-6..=1.0 + 1e-6).contains(&on) && out.is_none_or(|o| far < o.0)
                {
                    out = Some((far, j));
                }
            }
            let Some((far, j)) = out else {
                return false;
            };
            gone += far;
            if gone >= limit - SLACK {
                return true;
            }
            let (p, q) = (f.v[j], f.v[(j + 1) % 3]);
            let Some(sides) = self.edges.get(&edge_key(p, q)).filter(|s| !self.ends(s)) else {
                return false;
            };
            // Over the edge, turned into the next triangle's plane about it.
            let edge = unit(sub(self.place[q as usize], self.place[p as usize]));
            let outward = cross(edge, f.n);
            let (next, nk) = sides.at.map(|(f, k)| (f as usize, k as usize))
                [usize::from(sides.at[0].0 as usize == face)];
            at = add(at, mul(way, far));
            way = unit(add(
                mul(edge, dot(way, edge)),
                mul(cross(edge, self.faces[next].n), dot(way, outward)),
            ));
            (face, k) = (next, nk);
        }
        false
    }

    /// The largest side (m) of the box round each piece (the places joined through triangles), by place.
    pub(crate) fn pieces(&self) -> Vec<f64> {
        let mut parent: Vec<u32> = (0..self.place.len() as u32).collect();
        fn root(parent: &mut [u32], mut i: u32) -> u32 {
            while parent[i as usize] != i {
                parent[i as usize] = parent[parent[i as usize] as usize];
                i = parent[i as usize];
            }
            i
        }
        for f in &self.faces {
            let a = root(&mut parent, f.v[0]);
            for &v in &f.v[1..] {
                let b = root(&mut parent, v);
                parent[b as usize] = a;
            }
        }
        let mut boxes: Vec<(V3, V3)> =
            vec![([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]); self.place.len()];
        let roots: Vec<u32> = (0..self.place.len() as u32)
            .map(|i| root(&mut parent, i))
            .collect();
        for (i, p) in self.place.iter().enumerate() {
            let (lo, hi) = &mut boxes[roots[i] as usize];
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        roots
            .iter()
            .map(|&r| {
                let (lo, hi) = boxes[r as usize];
                (0..3).map(|k| hi[k] - lo[k]).fold(0.0, f64::max)
            })
            .collect()
    }
}
