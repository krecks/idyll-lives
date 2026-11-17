//! Rays against triangles: a tree of boxes over a mesh's triangles, and the ray-triangle test Blender casts with
//! (watertight, single precision), so a ray meets a built mesh where it met the one Blender built: the perches
//! found on a tree, and the baked shading, come from where rays land.

use crate::math::{V3, normalize_v3};
use crate::mesh::normal_tri;
use crate::scene::{self, Obj};

struct Node {
    lo: [f32; 3],
    hi: [f32; 3],
    /// A leaf's triangles (`first..first + count` of `order`), or a branch's two children (`first`, `first + 1`).
    first: u32,
    count: u32,
}

pub struct Bvh {
    nodes: Vec<Node>,
    order: Vec<u32>,
    pub tris: Vec<[V3; 3]>,
}

/// A ray's direction prepared for the triangle test: its largest axis and the shear onto it.
struct Shear {
    kx: usize,
    ky: usize,
    kz: usize,
    sx: f32,
    sy: f32,
    sz: f32,
}

impl Shear {
    fn new(dir: V3) -> Shear {
        let (x, y, z) = (dir.x.abs(), dir.y.abs(), dir.z.abs());
        let kz = if x > y {
            if x > z { 0 } else { 2 }
        } else if y > z {
            1
        } else {
            2
        };
        let mut kx = if kz != 2 { kz + 1 } else { 0 };
        let mut ky = if kx != 2 { kx + 1 } else { 0 };
        if dir[kz] < 0.0 {
            std::mem::swap(&mut kx, &mut ky);
        }
        let inv = 1.0 / dir[kz];
        Shear {
            kx,
            ky,
            kz,
            sx: dir[kx] * inv,
            sy: dir[ky] * inv,
            sz: inv,
        }
    }

    /// How far along the ray from `origin` it meets the triangle, if it does (either side of it).
    fn hit(&self, origin: V3, t: &[V3; 3]) -> Option<f32> {
        let (a, b, c) = (t[0] - origin, t[1] - origin, t[2] - origin);
        let (a_kx, a_ky, a_kz) = (a[self.kx], a[self.ky], a[self.kz]);
        let (b_kx, b_ky, b_kz) = (b[self.kx], b[self.ky], b[self.kz]);
        let (c_kx, c_ky, c_kz) = (c[self.kx], c[self.ky], c[self.kz]);
        let ax = a_kx - self.sx * a_kz;
        let ay = a_ky - self.sy * a_kz;
        let bx = b_kx - self.sx * b_kz;
        let by = b_ky - self.sy * b_kz;
        let cx = c_kx - self.sx * c_kz;
        let cy = c_ky - self.sy * c_kz;
        let u = cx * by - cy * bx;
        let v = ax * cy - ay * cx;
        let w = bx * ay - by * ax;
        if (u < 0.0 || v < 0.0 || w < 0.0) && (u > 0.0 || v > 0.0 || w > 0.0) {
            return None;
        }
        let det = u + v + w;
        if det == 0.0 || !det.is_finite() {
            return None;
        }
        let t = (u * a_kz + v * b_kz + w * c_kz) * self.sz;
        let signed = f32::from_bits(t.to_bits() ^ (det.to_bits() & 0x8000_0000));
        if signed < 0.0 {
            return None;
        }
        Some(t * (1.0 / det))
    }
}

impl Bvh {
    pub fn new(tris: Vec<[V3; 3]>) -> Bvh {
        let mut bvh = Bvh {
            nodes: Vec::new(),
            order: (0..tris.len() as u32).collect(),
            tris,
        };
        if !bvh.tris.is_empty() {
            let mid: Vec<[f32; 3]> = bvh
                .tris
                .iter()
                .map(|t| [0, 1, 2].map(|i| (t[0][i] + t[1][i] + t[2][i]) / 3.0))
                .collect();
            bvh.nodes.push(Node {
                lo: [0.0; 3],
                hi: [0.0; 3],
                first: 0,
                count: 0,
            });
            let n = bvh.order.len();
            bvh.build(0, 0, n, &mid);
        }
        bvh
    }

    fn build(&mut self, node: usize, start: usize, end: usize, mid: &[[f32; 3]]) {
        let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for &t in &self.order[start..end] {
            for p in &self.tris[t as usize] {
                for i in 0..3 {
                    lo[i] = lo[i].min(p[i]);
                    hi[i] = hi[i].max(p[i]);
                }
            }
        }
        (self.nodes[node].lo, self.nodes[node].hi) = (lo, hi);
        if end - start <= 4 {
            (self.nodes[node].first, self.nodes[node].count) = (start as u32, (end - start) as u32);
            return;
        }
        let axis = (0..3)
            .max_by(|&a, &b| (hi[a] - lo[a]).total_cmp(&(hi[b] - lo[b])))
            .unwrap_or(0);
        self.order[start..end].sort_by(|&a, &b| {
            mid[a as usize][axis]
                .total_cmp(&mid[b as usize][axis])
                .then(a.cmp(&b))
        });
        let half = (start + end) / 2;
        let first = self.nodes.len();
        self.nodes.push(Node {
            lo: [0.0; 3],
            hi: [0.0; 3],
            first: 0,
            count: 0,
        });
        self.nodes.push(Node {
            lo: [0.0; 3],
            hi: [0.0; 3],
            first: 0,
            count: 0,
        });
        (self.nodes[node].first, self.nodes[node].count) = (first as u32, 0);
        self.build(first, start, half, mid);
        self.build(first + 1, half, end, mid);
    }

    /// The first triangle the ray from `origin` along `dir` (a unit vector) meets within `reach`: (which, how
    /// far). Of two as near as each other, the earlier.
    pub fn ray_cast(&self, origin: V3, dir: V3, reach: f32) -> Option<(u32, f32)> {
        if self.nodes.is_empty() {
            return None;
        }
        let shear = Shear::new(dir);
        let o = [origin.x as f64, origin.y as f64, origin.z as f64];
        let inv = [1.0 / dir.x as f64, 1.0 / dir.y as f64, 1.0 / dir.z as f64];
        let mut best: Option<(u32, f32)> = None;
        let mut limit = reach;
        let mut stack = vec![0u32];
        while let Some(n) = stack.pop() {
            let node = &self.nodes[n as usize];
            // the box, a hair larger: no triangle is passed over for the rounding of this test
            let (mut t0, mut t1) = (0.0_f64, limit as f64 * 1.000_001 + 1e-6);
            let mut miss = false;
            for i in 0..3 {
                let (lo, hi) = (node.lo[i] as f64 - 1e-5, node.hi[i] as f64 + 1e-5);
                if inv[i].is_infinite() {
                    miss |= o[i] < lo || o[i] > hi;
                    continue;
                }
                let (a, b) = ((lo - o[i]) * inv[i], (hi - o[i]) * inv[i]);
                t0 = t0.max(a.min(b));
                t1 = t1.min(a.max(b));
            }
            if miss || t0 > t1 {
                continue;
            }
            if node.count == 0 {
                stack.push(node.first);
                stack.push(node.first + 1);
                continue;
            }
            for &t in &self.order[node.first as usize..(node.first + node.count) as usize] {
                if let Some(dist) = shear.hit(origin, &self.tris[t as usize]) {
                    let nearer = match best {
                        None => dist >= 0.0 && dist < limit,
                        Some((b, d)) => dist >= 0.0 && (dist < d || (dist == d && t < b)),
                    };
                    if nearer {
                        best = Some((t, dist));
                        limit = dist;
                    }
                }
            }
        }
        best
    }
}

/// Where a ray met the scene.
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub at: V3,
    /// The normal of the triangle it met.
    pub normal: V3,
    pub obj: Obj,
    /// The face it met, among the part's.
    pub face: u32,
}

/// The scene's parts as they stand now, to cast rays at (`scene.ray_cast`).
pub struct Rays {
    parts: Vec<(Obj, Bvh, Vec<u32>)>,
}

impl Rays {
    /// Every part in the scene, where it is.
    pub fn of_scene() -> Rays {
        Rays::of(&scene::meshes())
    }

    pub fn of(objs: &[Obj]) -> Rays {
        let parts = objs
            .iter()
            .map(|&o| {
                let points = o.world_points();
                let (tris, faces) = o.read(|p| {
                    let mut tris = Vec::new();
                    let mut faces = Vec::new();
                    for (f, t) in p.mesh.triangles() {
                        let c = p.mesh.corners(f as usize);
                        tris.push([
                            points[c[t[0] as usize] as usize],
                            points[c[t[1] as usize] as usize],
                            points[c[t[2] as usize] as usize],
                        ]);
                        faces.push(f);
                    }
                    (tris, faces)
                });
                (o, Bvh::new(tris), faces)
            })
            .collect();
        Rays { parts }
    }

    /// The first face the ray from `origin` along `direction` meets within `distance`.
    pub fn cast(&self, origin: V3, direction: V3, distance: f64) -> Option<Hit> {
        let distance = distance as f32;
        let direction = normalize_v3(direction).0;
        let mut depth = distance;
        let mut out = None;
        for (obj, bvh, faces) in &self.parts {
            // (each part takes the ray into its own frame, and makes it a unit vector again)
            let (dir, scale) = normalize_v3(direction);
            if let Some((t, dist)) = bvh.ray_cast(origin, dir, depth * scale) {
                let far = dist / scale;
                if far <= depth {
                    depth = far;
                    let tri = &bvh.tris[t as usize];
                    let at = V3::new(
                        origin.x + dir.x * dist,
                        origin.y + dir.y * dist,
                        origin.z + dir.z * dist,
                    );
                    let normal = normalize_v3(normal_tri(tri[0], tri[1], tri[2])).0;
                    out = Some(Hit {
                        at,
                        normal,
                        obj: *obj,
                        face: faces[t as usize],
                    });
                }
            }
        }
        out
    }
}
