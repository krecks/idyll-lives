//! What `cargo check-glb --against` measures between two builds of one model that are not the same file
//! (`digest`): their parts' surfaces against each other, and their clips sampled as the game plays them. Only
//! measurements: the limits are `against`'s.
//!
//! A part is a node's triangles of one material, in the node's own space (so a part grown from nothing, saved
//! at scale 0.001, is compared at its full size). A part's surface is sampled evenly by area, in a fixed
//! sequence, and each sample is taken to the nearest point of the other build's part: how far that is, and how
//! the normal, the texture coordinate and the baked shading there differ. Every vertex is taken across too, for
//! the distance and the normal, so a triangle too small for a sample is still measured.

use super::digest::{Tree, indices, raw};
use super::glb::{Glb, R, item, key, values};
use super::py::J;
use crate::characters::V3;
use crate::characters::meshops::nearest_weights;
use std::collections::BTreeMap;

pub type P3 = [f64; 3];

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: P3, b: P3) -> P3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(a: P3) -> f64 {
    dot(a, a).sqrt()
}

/// `a` at length 1, or zero for one that has none.
fn unit(a: P3) -> P3 {
    let l = length(a);
    if l > 0.0 { a.map(|v| v / l) } else { [0.0; 3] }
}

/// The corners' values at the weights `w`.
fn mix<const N: usize>(v: [[f64; N]; 3], w: [f64; 3]) -> [f64; N] {
    std::array::from_fn(|k| v[0][k] * w[0] + v[1][k] * w[1] + v[2][k] * w[2])
}

/// The angle between two directions, in degrees (0 for one without a direction).
pub fn angle(a: P3, b: P3) -> f64 {
    let (la, lb) = (length(a), length(b));
    if la == 0.0 || lb == 0.0 {
        return 0.0;
    }
    (dot(a, b) / (la * lb)).clamp(-1.0, 1.0).acos().to_degrees()
}

/// A node's triangles of one material, in the node's own space.
pub struct Part {
    pub node: usize,
    /// The node's path (`digest::Tree`).
    pub path: String,
    pub material: String,
    pub positions: Vec<P3>,
    /// Unit normals, if every primitive of the part has them.
    pub normals: Option<Vec<P3>>,
    /// TEXCOORD_0, if it has it.
    pub uvs: Option<Vec<[f64; 2]>>,
    /// COLOR_0 as a grey from 0 to 1 (the mean of red, green and blue: the baked shading), if it has it.
    pub greys: Option<Vec<f64>>,
    pub triangles: Vec<[u32; 3]>,
}

/// The file's parts, in the order they are met (a node's primitives of one material are one part).
pub fn parts(g: &Glb, tree: &Tree) -> R<Vec<Part>> {
    let nodes = g.gltf.get("nodes").and_then(J::list).unwrap_or(&[]);
    let mut out: Vec<Part> = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        let Some(mesh) = n.get("mesh") else { continue };
        let prims = key(item(&g.gltf, "meshes", mesh)?, "primitives")?;
        for prim in prims.list().unwrap_or(&[]) {
            if prim.get("mode").is_some_and(|m| *m != J::Int(4)) {
                continue;
            }
            let material = match prim.get("material") {
                Some(m) => item(&g.gltf, "materials", m)?
                    .get("name")
                    .map(J::key)
                    .unwrap_or_default(),
                None => String::new(),
            };
            let attributes = key(prim, "attributes")?;
            let read = |name: &str, width: usize| -> R<Option<Vec<f64>>> {
                match attributes.get(name) {
                    None => Ok(None),
                    Some(index) => {
                        let a = raw(g, index)?;
                        if a.width < width {
                            return Err(format!("{name} has {} components", a.width));
                        }
                        let all = a.floats();
                        Ok(Some(if a.width == width {
                            all
                        } else {
                            all.chunks(a.width)
                                .flat_map(|e| e[..width].to_vec())
                                .collect()
                        }))
                    }
                }
            };
            let positions: Vec<P3> = read("POSITION", 3)?
                .ok_or("a primitive without POSITION")?
                .as_chunks::<3>()
                .0
                .to_vec();
            let count = positions.len();
            let sized = |v: Option<Vec<f64>>, width: usize| v.filter(|v| v.len() == count * width);
            let normals: Option<Vec<P3>> = sized(read("NORMAL", 3)?, 3)
                .map(|v| v.as_chunks::<3>().0.iter().map(|p| unit(*p)).collect());
            let uvs: Option<Vec<[f64; 2]>> =
                sized(read("TEXCOORD_0", 2)?, 2).map(|v| v.as_chunks::<2>().0.to_vec());
            let greys: Option<Vec<f64>> = sized(read("COLOR_0", 3)?, 3).map(|v| {
                v.as_chunks::<3>()
                    .0
                    .iter()
                    .map(|c| (c[0] + c[1] + c[2]) / 3.0)
                    .collect()
            });
            let idx = indices(g, prim, count)?;
            let at = match out
                .iter()
                .position(|p| p.node == i && p.material == material)
            {
                Some(at) => at,
                None => {
                    out.push(Part {
                        node: i,
                        path: tree.paths[i].clone(),
                        material,
                        positions: Vec::new(),
                        normals: Some(Vec::new()),
                        uvs: Some(Vec::new()),
                        greys: Some(Vec::new()),
                        triangles: Vec::new(),
                    });
                    out.len() - 1
                }
            };
            let part = &mut out[at];
            let base = part.positions.len() as u32;
            part.positions.extend(positions);
            // (An attribute one of the part's primitives lacks, the part lacks.)
            fn join<T>(have: &mut Option<Vec<T>>, more: Option<Vec<T>>) {
                match (have.as_mut(), more) {
                    (Some(have), Some(more)) => have.extend(more),
                    _ => *have = None,
                }
            }
            join(&mut part.normals, normals);
            join(&mut part.uvs, uvs);
            join(&mut part.greys, greys);
            part.triangles
                .extend(idx.as_chunks::<3>().0.iter().map(|t| t.map(|i| i + base)));
        }
    }
    Ok(out)
}

impl Part {
    /// The part by name: its node's path and its material.
    pub fn name(&self) -> String {
        format!("{} / {}", self.path, self.material)
    }

    fn corners(&self, t: usize) -> [P3; 3] {
        self.triangles[t].map(|i| self.positions[i as usize])
    }

    /// A triangle's normal from its winding, as long as twice its area.
    fn face(&self, t: usize) -> P3 {
        let [a, b, c] = self.corners(t);
        cross(sub(b, a), sub(c, a))
    }

    /// The part's area (m²).
    pub fn area(&self) -> f64 {
        (0..self.triangles.len())
            .map(|t| length(self.face(t)) / 2.0)
            .sum()
    }

    /// The area its triangles take in the texture (UV units squared); 0 without texture coordinates.
    pub fn uv_area(&self) -> f64 {
        let Some(uvs) = &self.uvs else { return 0.0 };
        self.triangles
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|i| uvs[i as usize]);
                ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() / 2.0
            })
            .sum()
    }

    /// Its box (lowest, highest corner); None for a part without vertices.
    pub fn bounds(&self) -> Option<(P3, P3)> {
        let first = *self.positions.first()?;
        Some(self.positions.iter().fold((first, first), |(lo, hi), p| {
            (
                std::array::from_fn(|k| lo[k].min(p[k])),
                std::array::from_fn(|k| hi[k].max(p[k])),
            )
        }))
    }

    /// The normal at a point of triangle `t` (weights `w`): its corners' normals there, or its face's.
    fn normal_at(&self, t: usize, w: [f64; 3]) -> P3 {
        match &self.normals {
            Some(n) => unit(mix(self.triangles[t].map(|i| n[i as usize]), w)),
            None => unit(self.face(t)),
        }
    }

    /// The part moved through a column-major 4x4 matrix (positions and directions).
    pub fn placed(mut self, m: &[f64; 16]) -> Part {
        let point = |p: P3| -> P3 {
            std::array::from_fn(|k| m[k] * p[0] + m[4 + k] * p[1] + m[8 + k] * p[2] + m[12 + k])
        };
        let turn = |p: P3| -> P3 {
            unit(std::array::from_fn(|k| {
                m[k] * p[0] + m[4 + k] * p[1] + m[8 + k] * p[2]
            }))
        };
        self.positions = self.positions.into_iter().map(point).collect();
        self.normals = self.normals.map(|n| n.into_iter().map(turn).collect());
        self
    }
}

/// A point of a part's surface with what the part has there.
pub struct Sample {
    pub at: P3,
    pub normal: P3,
    pub uv: Option<[f64; 2]>,
    pub grey: Option<f64>,
}

/// `per_m2` points a square metre of the part's surface, no fewer than `few` and no more than `many`, spread
/// evenly by area: point `i` of `n` lies in the triangle that holds the area's (i + 0.5) / n-th part, at a place
/// in it from a fixed low-discrepancy sequence, so two runs take the same points.
pub fn samples(part: &Part, per_m2: f64, few: usize, many: usize) -> Vec<Sample> {
    let areas: Vec<f64> = (0..part.triangles.len())
        .map(|t| length(part.face(t)) / 2.0)
        .collect();
    let total: f64 = areas.iter().sum();
    if total <= 0.0 || !total.is_finite() {
        return Vec::new();
    }
    let n = ((total * per_m2).round() as usize).clamp(few, many);
    let mut out = Vec::with_capacity(n);
    let (mut t, mut before) = (0usize, 0.0f64);
    for i in 0..n {
        let target = (i as f64 + 0.5) / n as f64 * total;
        while t + 1 < areas.len() && before + areas[t] < target {
            before += areas[t];
            t += 1;
        }
        // The plastic number's sequence (R2), folded into the triangle.
        let (mut u, mut v) = (
            (0.5 + i as f64 * 0.754_877_666_246_692_7).fract(),
            (0.5 + i as f64 * 0.569_840_290_998_053_2).fract(),
        );
        if u + v > 1.0 {
            (u, v) = (1.0 - u, 1.0 - v);
        }
        let w = [1.0 - u - v, u, v];
        let corners = part.triangles[t];
        out.push(Sample {
            at: mix(part.corners(t), w),
            normal: part.normal_at(t, w),
            uv: part
                .uvs
                .as_ref()
                .map(|uv| mix(corners.map(|i| uv[i as usize]), w)),
            grey: part
                .greys
                .as_ref()
                .map(|g| mix(corners.map(|i| [g[i as usize]]), w)[0]),
        });
    }
    out
}

/// A box of the tree over a part's triangles: its corners, and its two children or (`count` over 0) its triangles.
struct Box3 {
    lo: P3,
    hi: P3,
    first: u32,
    count: u32,
}

/// A part's triangles in a tree of boxes, to find the nearest point of its surface. (Not `characters::kdtree`: the
/// nearest triangle is not the one with the nearest centre, and the proof wants the nearest.)
pub struct Surface<'a> {
    pub part: &'a Part,
    boxes: Vec<Box3>,
    order: Vec<u32>,
}

/// The nearest point of a surface to a sample.
pub struct Nearest {
    /// How far the surface is (m), whichever way it faces there.
    pub distance: f64,
    /// The normal at the point taken as the sample's counterpart: the nearest, or one within `reach` further
    /// that faces more as the sample does.
    pub normal: P3,
}

/// A point of a surface: its triangle and the weights of the triangle's corners.
type On = (usize, [f64; 3]);

/// What `Surface::nearest` works in, kept from one point to the next: the points that tie with the counterpart
/// (each with what it costs), and the boxes still to look into.
#[derive(Default)]
pub struct Work {
    tied: Vec<(f64, On)>,
    stack: Vec<(u32, f64)>,
}

impl<'a> Surface<'a> {
    pub fn new(part: &'a Part) -> Surface<'a> {
        // (A triangle without area has no nearest point of its own; its neighbours cover its corners.)
        let mut order: Vec<u32> = (0..part.triangles.len() as u32)
            .filter(|t| length(part.face(*t as usize)) > 1e-18)
            .collect();
        let centres: Vec<P3> = (0..part.triangles.len())
            .map(|t| {
                let [a, b, c] = part.corners(t);
                std::array::from_fn(|k| (a[k] + b[k] + c[k]) / 3.0)
            })
            .collect();
        let mut s = Surface {
            part,
            boxes: Vec::new(),
            order: Vec::new(),
        };
        if !order.is_empty() {
            let n = order.len();
            s.build(&mut order, 0, n, &centres);
        }
        s.order = order;
        s
    }

    fn build(&mut self, order: &mut [u32], start: usize, end: usize, centres: &[P3]) -> u32 {
        let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
        let (mut clo, mut chi) = (lo, hi);
        for &t in &order[start..end] {
            for p in self.part.corners(t as usize) {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            for k in 0..3 {
                clo[k] = clo[k].min(centres[t as usize][k]);
                chi[k] = chi[k].max(centres[t as usize][k]);
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
            let axis = (0..3)
                .max_by(|a, b| (chi[*a] - clo[*a]).total_cmp(&(chi[*b] - clo[*b])))
                .unwrap();
            let mid = (start + end) / 2;
            order[start..end].select_nth_unstable_by(mid - start, |a, b| {
                centres[*a as usize][axis].total_cmp(&centres[*b as usize][axis])
            });
            self.build(order, start, mid, centres);
            let right = self.build(order, mid, end, centres);
            // (The left child is the next box; the right one is kept.)
            self.boxes[at as usize].first = right;
            self.boxes[at as usize].count = 0;
        }
        at
    }

    /// The nearest point to `p`, whose surface faces along `normal` there (`Nearest`); None for a surface without
    /// triangles. A point `d` away whose normal is turned by an angle costs d + reach × (1 − cos angle) / 2, and the
    /// cheapest is the counterpart: so of two faces back to back, or meeting in a sharp edge, the one facing as the
    /// sample does is taken, as long as it is within `reach` of the nearest. `work.tied` is left holding every
    /// point that costs within `same` of the counterpart, itself among them: where parts are laid into each
    /// other, faces of one place and one facing that may differ in their texture and shading.
    pub fn nearest(
        &self,
        p: P3,
        normal: P3,
        reach: f64,
        same: f64,
        work: &mut Work,
    ) -> Option<Nearest> {
        let Work { tied, stack } = work;
        tied.clear();
        if self.boxes.is_empty() {
            return None;
        }
        let away = |b: &Box3| -> f64 {
            let mut d2 = 0.0;
            for k in 0..3 {
                let d = (b.lo[k] - p[k]).max(p[k] - b.hi[k]).max(0.0);
                d2 += d * d;
            }
            d2.sqrt()
        };
        let pv = V3::new(p[0], p[1], p[2]);
        let (mut distance, mut cost) = (f64::INFINITY, f64::INFINITY);
        let mut found: Option<On> = None;
        stack.clear();
        stack.push((0, away(&self.boxes[0])));
        while let Some((at, d)) = stack.pop() {
            // (The cheapest so far costs no less than the nearest is far, so this keeps both.)
            if d >= cost + same {
                continue;
            }
            let b = &self.boxes[at as usize];
            if b.count == 0 {
                let (l, r) = (at + 1, b.first);
                let (dl, dr) = (away(&self.boxes[l as usize]), away(&self.boxes[r as usize]));
                // The nearer box last, to be looked into first.
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
                let t = t as usize;
                let [a, bb, c] = self.part.corners(t).map(|q| V3::new(q[0], q[1], q[2]));
                let w = nearest_weights(pv, a, bb, c);
                let d = (a * w[0] + bb * w[1] + c * w[2] - pv).norm();
                if d.is_nan() || d >= cost + same {
                    continue;
                }
                distance = distance.min(d);
                let turned = 1.0 - dot(normal, self.part.normal_at(t, w));
                let costs = d + reach * turned / 2.0;
                if costs < cost + same {
                    tied.push((costs, (t, w)));
                }
                if costs < cost {
                    cost = costs;
                    found = Some((t, w));
                }
            }
        }
        let (t, w) = found?;
        tied.retain(|(costs, _)| *costs <= cost + same);
        Some(Nearest {
            distance,
            normal: self.part.normal_at(t, w),
        })
    }

    fn uv_at(&self, (t, w): On) -> Option<[f64; 2]> {
        let uv = self.part.uvs.as_ref()?;
        Some(mix(self.part.triangles[t].map(|i| uv[i as usize]), w))
    }

    fn grey_at(&self, (t, w): On) -> Option<f64> {
        let g = self.part.greys.as_ref()?;
        Some(mix(self.part.triangles[t].map(|i| [g[i as usize]]), w)[0])
    }
}

/// What `across` takes as a difference.
pub struct Tolerance {
    /// Samples a square metre, and the fewest and the most for a part.
    pub per_m2: f64,
    pub few: usize,
    pub many: usize,
    /// How much further than the nearest point a counterpart facing as the sample does may be (m).
    pub reach: f64,
    /// Points whose costs are within this of each other are one place (m).
    pub same: f64,
    /// A normal turned by more than this (degrees) is counted as turned.
    pub turned: f64,
    /// One turned by more than this faces the other way.
    pub away: f64,
    /// A texture coordinate further off than this (m on the surface) is counted as off.
    pub uv_off: f64,
}

/// One build's part measured against the other's surface.
#[derive(Default)]
pub struct Across {
    /// The surface samples taken.
    pub samples: usize,
    /// Their mean distance to the other surface, and the largest of them and of the vertices, with where (m).
    pub mean: f64,
    pub max: f64,
    pub max_at: P3,
    /// The share of samples whose normal is turned, and the largest turn (degrees) with where.
    pub turned: f64,
    pub turn: f64,
    pub turn_at: P3,
    /// Samples and vertices that face the other way, and the first of them.
    pub away: usize,
    pub away_at: P3,
    /// The share of samples whose texture coordinate is off, and the furthest (m) with where; None where either
    /// part has none.
    pub uv: Option<(f64, f64, P3)>,
    /// The baked shading's difference in grey levels (of `levels`): its mean, the level 95 % of the samples are
    /// within, the largest and where; None where either part has none.
    pub shade: Option<(f64, f64, f64, P3)>,
}

/// Measures `from` against the surface `onto` (`Across`). `uv_metres`: how long one unit of the texture
/// coordinates is on the surface (m), `levels`: the grey levels the shading is counted in.
pub fn across(from: &Part, onto: &Surface, tol: &Tolerance, uv_metres: f64, levels: f64) -> Across {
    let mut out = Across::default();
    let points = samples(from, tol.per_m2, tol.few, tol.many);
    out.samples = points.len();
    let (mut sum, mut turned) = (0.0, 0usize);
    let (mut uv_n, mut uv_off, mut uv_worst, mut uv_at) = (0usize, 0usize, 0.0f64, [0.0; 3]);
    let mut shades: Vec<f64> = Vec::new();
    let (mut shade_worst, mut shade_at) = (0.0f64, [0.0; 3]);
    // (A surface without triangles is nowhere: the first point says so.)
    fn gone(out: &mut Across, at: P3) {
        if out.max < f64::INFINITY {
            (out.max, out.max_at) = (f64::INFINITY, at);
        }
    }
    let mut work = Work::default();
    for s in &points {
        let Some(n) = onto.nearest(s.at, s.normal, tol.reach, tol.same, &mut work) else {
            gone(&mut out, s.at);
            continue;
        };
        sum += n.distance;
        if n.distance > out.max {
            (out.max, out.max_at) = (n.distance, s.at);
        }
        let turn = angle(s.normal, n.normal);
        if turn > tol.turned {
            turned += 1;
        }
        if turn > out.turn {
            (out.turn, out.turn_at) = (turn, s.at);
        }
        if turn > tol.away {
            if out.away == 0 {
                out.away_at = s.at;
            }
            out.away += 1;
        }
        // The texture and the shading against the nearest of the faces there (`Surface::nearest`).
        if let Some(a) = s.uv {
            // (A texture repeats: a whole turn of it is no difference.)
            let off = |b: [f64; 2]| {
                let turn = |k: usize| {
                    let d = a[k] - b[k];
                    d - d.round()
                };
                turn(0).hypot(turn(1)) * uv_metres
            };
            let nearest = work
                .tied
                .iter()
                .filter_map(|(_, on)| onto.uv_at(*on))
                .map(off)
                .min_by(f64::total_cmp);
            if let Some(d) = nearest {
                uv_n += 1;
                if d > tol.uv_off {
                    uv_off += 1;
                }
                if d > uv_worst {
                    (uv_worst, uv_at) = (d, s.at);
                }
            }
        }
        if let Some(a) = s.grey {
            let nearest = work
                .tied
                .iter()
                .filter_map(|(_, on)| onto.grey_at(*on))
                .map(|b| (a - b).abs() * levels)
                .min_by(f64::total_cmp);
            if let Some(d) = nearest {
                shades.push(d);
                if d > shade_worst {
                    (shade_worst, shade_at) = (d, s.at);
                }
            }
        }
    }
    if !points.is_empty() {
        out.mean = if out.max.is_finite() {
            sum / points.len() as f64
        } else {
            f64::INFINITY
        };
        out.turned = turned as f64 / points.len() as f64;
    }
    // Every vertex of a triangle with an area (the corners of one without belong to no face).
    let mut used = vec![false; from.positions.len()];
    for (t, corners) in from.triangles.iter().enumerate() {
        if length(from.face(t)) > 1e-18 {
            corners.iter().for_each(|i| used[*i as usize] = true);
        }
    }
    for (i, p) in from.positions.iter().enumerate().filter(|(i, _)| used[*i]) {
        let normal = from.normals.as_ref().map(|n| n[i]);
        let Some(n) = onto.nearest(
            *p,
            normal.unwrap_or([0.0; 3]),
            tol.reach,
            tol.same,
            &mut work,
        ) else {
            gone(&mut out, *p);
            continue;
        };
        if n.distance > out.max {
            (out.max, out.max_at) = (n.distance, *p);
        }
        if normal.is_some_and(|normal| angle(normal, n.normal) > tol.away) {
            if out.away == 0 {
                out.away_at = *p;
            }
            out.away += 1;
        }
    }
    if uv_n > 0 {
        out.uv = Some((uv_off as f64 / uv_n as f64, uv_worst, uv_at));
    }
    if !shades.is_empty() {
        let mean = shades.iter().sum::<f64>() / shades.len() as f64;
        shades.sort_by(f64::total_cmp);
        let p95 = shades[((shades.len() as f64 * 0.95) as usize).min(shades.len() - 1)];
        out.shade = Some((mean, p95, shade_worst, shade_at));
    }
    out
}

/// A node at rest: where its origin is in its parent, how it is turned (x, y, z, w) and scaled.
#[derive(Clone, Copy)]
pub struct Rest {
    pub translation: P3,
    pub rotation: [f64; 4],
    pub scale: P3,
}

fn numbers<const N: usize>(v: Option<&J>, default: [f64; N]) -> R<[f64; N]> {
    let Some(v) = v else { return Ok(default) };
    let list: Vec<f64> = v
        .list()
        .unwrap_or(&[])
        .iter()
        .filter_map(|x| x.num().map(|n| n.f()))
        .collect();
    list.try_into()
        .map_err(|_| format!("a node's transform is no list of {N} numbers"))
}

/// Every node's rest (a `matrix` taken apart into the three).
pub fn rests(gltf: &J) -> R<Vec<Rest>> {
    let mut out = Vec::new();
    for n in gltf.get("nodes").and_then(J::list).unwrap_or(&[]) {
        if let Some(m) = n.get("matrix") {
            let m: [f64; 16] = numbers(Some(m), [0.0; 16])?;
            let col = |c: usize| [m[c * 4], m[c * 4 + 1], m[c * 4 + 2]];
            let scale = [length(col(0)), length(col(1)), length(col(2))];
            let r = [unit(col(0)), unit(col(1)), unit(col(2))];
            // (Shepperd's method, from the larger of the four.)
            let trace = r[0][0] + r[1][1] + r[2][2];
            let rotation = if trace > 0.0 {
                let s = (trace + 1.0).sqrt() * 2.0;
                [
                    (r[1][2] - r[2][1]) / s,
                    (r[2][0] - r[0][2]) / s,
                    (r[0][1] - r[1][0]) / s,
                    s / 4.0,
                ]
            } else if r[0][0] > r[1][1] && r[0][0] > r[2][2] {
                let s = (1.0 + r[0][0] - r[1][1] - r[2][2]).sqrt() * 2.0;
                [
                    s / 4.0,
                    (r[1][0] + r[0][1]) / s,
                    (r[2][0] + r[0][2]) / s,
                    (r[1][2] - r[2][1]) / s,
                ]
            } else if r[1][1] > r[2][2] {
                let s = (1.0 + r[1][1] - r[0][0] - r[2][2]).sqrt() * 2.0;
                [
                    (r[1][0] + r[0][1]) / s,
                    s / 4.0,
                    (r[2][1] + r[1][2]) / s,
                    (r[2][0] - r[0][2]) / s,
                ]
            } else {
                let s = (1.0 + r[2][2] - r[0][0] - r[1][1]).sqrt() * 2.0;
                [
                    (r[2][0] + r[0][2]) / s,
                    (r[2][1] + r[1][2]) / s,
                    s / 4.0,
                    (r[0][1] - r[1][0]) / s,
                ]
            };
            out.push(Rest {
                translation: [m[12], m[13], m[14]],
                rotation,
                scale,
            });
        } else {
            out.push(Rest {
                translation: numbers(n.get("translation"), [0.0; 3])?,
                rotation: numbers(n.get("rotation"), [0.0, 0.0, 0.0, 1.0])?,
                scale: numbers(n.get("scale"), [1.0; 3])?,
            });
        }
    }
    Ok(out)
}

/// Every node's place in the model at rest, as a column-major 4x4 matrix.
pub fn worlds(tree: &Tree, rests: &[Rest]) -> Vec<[f64; 16]> {
    let local = |r: &Rest| -> [f64; 16] {
        let [x, y, z, w] = r.rotation;
        let rot = [
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y + z * w),
                2.0 * (x * z - y * w),
            ],
            [
                2.0 * (x * y - z * w),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z + x * w),
            ],
            [
                2.0 * (x * z + y * w),
                2.0 * (y * z - x * w),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ];
        let mut m = [0.0; 16];
        for c in 0..3 {
            for k in 0..3 {
                m[c * 4 + k] = rot[c][k] * r.scale[c];
            }
            m[12 + c] = r.translation[c];
        }
        m[15] = 1.0;
        m
    };
    let mul = |a: &[f64; 16], b: &[f64; 16]| -> [f64; 16] {
        std::array::from_fn(|i| {
            let (c, r) = (i / 4, i % 4);
            (0..4).map(|k| a[k * 4 + r] * b[c * 4 + k]).sum()
        })
    };
    (0..rests.len())
        .map(|i| {
            let mut m = local(&rests[i]);
            let mut k = i;
            while let Some(p) = tree.parents[k] {
                m = mul(&local(&rests[p]), &m);
                k = p;
            }
            m
        })
        .collect()
}

/// The angle between two rotations (x, y, z, w), in degrees.
pub fn turn(a: [f64; 4], b: [f64; 4]) -> f64 {
    let norm = |q: [f64; 4]| q.iter().map(|v| v * v).sum::<f64>().sqrt();
    let (la, lb) = (norm(a), norm(b));
    if la == 0.0 || lb == 0.0 {
        return 0.0;
    }
    // (q and -q are one rotation.)
    let d = (a.iter().zip(&b).map(|(x, y)| x * y).sum::<f64>() / (la * lb)).abs();
    (2.0 * d.min(1.0).acos()).to_degrees()
}

/// One node's keys for its place, its turn or its size in a clip.
pub struct Track {
    pub times: Vec<f64>,
    pub keys: Vec<Vec<f64>>,
    /// The value holds until the next key (STEP).
    pub step: bool,
}

impl Track {
    /// The value at `t` seconds, as the glTF loader reads it: held before the first key and after the last, and
    /// between two keys a straight line, or for a turn the arc between them. None for a track without keys.
    pub fn at(&self, t: f64, rotation: bool) -> Option<Vec<f64>> {
        let n = self.times.len().min(self.keys.len());
        if n == 0 {
            return None;
        }
        if t <= self.times[0] {
            return Some(self.keys[0].clone());
        }
        if t >= self.times[n - 1] {
            return Some(self.keys[n - 1].clone());
        }
        let i = self.times[..n].partition_point(|x| *x <= t) - 1;
        let (a, b) = (&self.keys[i], &self.keys[i + 1]);
        let span = self.times[i + 1] - self.times[i];
        if self.step || span <= 0.0 || a.len() != b.len() {
            return Some(a.clone());
        }
        let u = (t - self.times[i]) / span;
        if !rotation {
            return Some(a.iter().zip(b).map(|(x, y)| x + (y - x) * u).collect());
        }
        let mut d: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let sign = if d < 0.0 { -1.0 } else { 1.0 };
        d = d.abs().min(1.0);
        let theta = d.acos();
        let (wa, wb) = if theta < 1e-6 {
            (1.0 - u, u)
        } else {
            (
                ((1.0 - u) * theta).sin() / theta.sin(),
                (u * theta).sin() / theta.sin(),
            )
        };
        Some(
            a.iter()
                .zip(b)
                .map(|(x, y)| x * wa + sign * y * wb)
                .collect(),
        )
    }
}

/// A clip: its name, when its first and last keys are (s), and its tracks by the node's path and what of it moves
/// (`translation`, `rotation`, `scale`).
pub struct Clip {
    pub name: String,
    pub start: f64,
    pub end: f64,
    pub tracks: BTreeMap<(String, String), Track>,
}

/// The file's clips, in its order.
pub fn clips(g: &Glb, tree: &Tree) -> R<Vec<Clip>> {
    let mut out = Vec::new();
    for (i, a) in g
        .gltf
        .get("animations")
        .and_then(J::list)
        .unwrap_or(&[])
        .iter()
        .enumerate()
    {
        let mut clip = Clip {
            name: match a.get("name") {
                Some(name) => name.key(),
                None => format!("animation{i}"),
            },
            start: f64::INFINITY,
            end: f64::NEG_INFINITY,
            tracks: BTreeMap::new(),
        };
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let node = match target.get("node") {
                Some(J::Int(i)) if *i >= 0 && (*i as usize) < tree.paths.len() => {
                    tree.paths[*i as usize].clone()
                }
                Some(_) => return Err("a clip moves a node the file has not".into()),
                None => continue,
            };
            let sampler = super::glb::index(key(a, "samplers")?, key(c, "sampler")?)?;
            let times: Vec<f64> = values(g, key(sampler, "input")?)?
                .iter()
                .filter_map(|t| t.first().copied())
                .collect();
            let mut keys = values(g, key(sampler, "output")?)?;
            let kind = sampler
                .get("interpolation")
                .and_then(J::str)
                .unwrap_or("LINEAR");
            if kind == "CUBICSPLINE" {
                // (In-tangent, value, out-tangent: the game plays the values as straight lines.)
                keys = keys.into_iter().skip(1).step_by(3).collect();
            }
            for t in &times {
                clip.start = clip.start.min(*t);
                clip.end = clip.end.max(*t);
            }
            clip.tracks.insert(
                (node, target.get("path").map(J::key).unwrap_or_default()),
                Track {
                    times,
                    keys,
                    step: kind == "STEP",
                },
            );
        }
        if clip.start > clip.end {
            (clip.start, clip.end) = (0.0, 0.0);
        }
        out.push(clip);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Numbers from 0 to 1 that are the same on every run.
    struct Dice(u64);

    impl Dice {
        fn roll(&mut self) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }

        fn point(&mut self) -> P3 {
            [self.roll(), self.roll(), self.roll()]
        }
    }

    fn part(positions: Vec<P3>, triangles: Vec<[u32; 3]>) -> Part {
        Part {
            node: 0,
            path: "body".into(),
            material: "wood".into(),
            positions,
            normals: None,
            uvs: None,
            greys: None,
            triangles,
        }
    }

    /// A square of side 1 in the plane z = `z`, facing up or down, each corner with `grey`.
    fn square(z: f64, up: bool, grey: f64) -> Part {
        let mut p = part(
            vec![[0.0, 0.0, z], [1.0, 0.0, z], [1.0, 1.0, z], [0.0, 1.0, z]],
            if up {
                vec![[0, 1, 2], [0, 2, 3]]
            } else {
                vec![[0, 2, 1], [0, 3, 2]]
            },
        );
        p.normals = Some(vec![[0.0, 0.0, if up { 1.0 } else { -1.0 }]; 4]);
        p.uvs = Some(p.positions.iter().map(|q| [q[0], q[1]]).collect());
        p.greys = Some(vec![grey; 4]);
        p
    }

    /// Two parts as one.
    fn joined(mut a: Part, b: Part) -> Part {
        let base = a.positions.len() as u32;
        a.positions.extend(b.positions);
        a.triangles
            .extend(b.triangles.iter().map(|t| t.map(|i| i + base)));
        let both =
            |x: &mut Option<Vec<P3>>, y: Option<Vec<P3>>| x.as_mut().unwrap().extend(y.unwrap());
        both(&mut a.normals, b.normals);
        a.uvs.as_mut().unwrap().extend(b.uvs.unwrap());
        a.greys.as_mut().unwrap().extend(b.greys.unwrap());
        a
    }

    const TOLERANCE: Tolerance = Tolerance {
        per_m2: 1e4,
        few: 2000,
        many: 50000,
        reach: 0.0002,
        same: 1e-5,
        turned: 15.0,
        away: 90.0,
        uv_off: 0.002,
    };

    #[test]
    fn the_nearest_point_is_the_nearest_of_every_triangle() {
        let mut dice = Dice(7);
        let positions: Vec<P3> = (0..600).map(|_| dice.point()).collect();
        let triangles: Vec<[u32; 3]> = (0..200).map(|t| [t * 3, t * 3 + 1, t * 3 + 2]).collect();
        let p = part(positions, triangles);
        let surface = Surface::new(&p);
        let mut work = Work::default();
        for _ in 0..500 {
            let at = dice.point().map(|v| v * 3.0 - 1.0);
            let found = surface.nearest(at, [0.0; 3], 0.0, 0.0, &mut work).unwrap();
            let brute = (0..p.triangles.len())
                .map(|t| {
                    let [a, b, c] = p.corners(t).map(|q| V3::new(q[0], q[1], q[2]));
                    let v = V3::new(at[0], at[1], at[2]);
                    let w = nearest_weights(v, a, b, c);
                    (a * w[0] + b * w[1] + c * w[2] - v).norm()
                })
                .fold(f64::INFINITY, f64::min);
            assert!(
                (found.distance - brute).abs() < 1e-12,
                "{} {brute}",
                found.distance
            );
        }
        assert!(
            Surface::new(&part(Vec::new(), Vec::new()))
                .nearest([0.0; 3], [0.0; 3], 0.0, 0.0, &mut work)
                .is_none()
        );
    }

    #[test]
    fn of_two_faces_back_to_back_the_one_facing_as_the_point_does_is_its_counterpart() {
        // A sheet a tenth of a millimetre thick: its top at z = 0.0001, its underside at 0.
        let sheet = joined(square(0.0001, true, 1.0), square(0.0, false, 0.5));
        let surface = Surface::new(&sheet);
        let mut work = Work::default();
        // A point on the underside's plane that faces up belongs to the top.
        let found = surface
            .nearest(
                [0.5, 0.5, 0.0],
                [0.0, 0.0, 1.0],
                TOLERANCE.reach,
                TOLERANCE.same,
                &mut work,
            )
            .unwrap();
        assert_eq!((found.distance, found.normal), (0.0, [0.0, 0.0, 1.0]));
        let found = surface
            .nearest(
                [0.5, 0.5, 0.0],
                [0.0, 0.0, -1.0],
                TOLERANCE.reach,
                TOLERANCE.same,
                &mut work,
            )
            .unwrap();
        assert_eq!(found.normal, [0.0, 0.0, -1.0]);
        // A millimetre apart the top is out of reach: the underside is all there is.
        let apart = joined(square(0.001, true, 1.0), square(0.0, false, 0.5));
        let found = Surface::new(&apart)
            .nearest(
                [0.5, 0.5, 0.0],
                [0.0, 0.0, 1.0],
                TOLERANCE.reach,
                TOLERANCE.same,
                &mut work,
            )
            .unwrap();
        assert_eq!(found.normal, [0.0, 0.0, -1.0]);
    }

    #[test]
    fn a_part_is_sampled_evenly_and_the_same_every_time() {
        // Two triangles, one three times the other's area.
        let p = part(
            vec![
                [0.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0],
                [-1.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 4, 3]],
        );
        assert_eq!(p.area(), 2.0);
        let points = samples(&p, 1e4, 2000, 50000);
        assert_eq!(points.len(), 20000);
        let large = points.iter().filter(|s| s.at[0] > 0.0).count();
        assert_eq!(large, 15000);
        assert!(
            points
                .iter()
                .all(|s| s.at[2] == 0.0 && s.normal[2].abs() == 1.0)
        );
        let again = samples(&p, 1e4, 2000, 50000);
        assert!(points.iter().zip(&again).all(|(a, b)| a.at == b.at));
        // The fewest and the most.
        assert_eq!(samples(&p, 1.0, 2000, 50000).len(), 2000);
        assert_eq!(samples(&p, 1e6, 2000, 50000).len(), 50000);
        assert!(
            samples(&part(vec![[0.0; 3]; 3], vec![[0, 1, 2]]), 1e4, 2000, 50000).is_empty(),
            "no area"
        );
    }

    #[test]
    fn a_part_is_measured_against_another() {
        let (a, b) = (square(0.0, true, 1.0), square(0.0005, true, 0.75));
        let found = across(&a, &Surface::new(&b), &TOLERANCE, 1.0, 32.0);
        assert_eq!(found.samples, 10000);
        assert!((found.mean - 0.0005).abs() < 1e-12 && (found.max - 0.0005).abs() < 1e-12);
        assert_eq!((found.turned, found.away), (0.0, 0));
        let (off, furthest, _) = found.uv.unwrap();
        assert!(off == 0.0 && furthest < 1e-12);
        let (mean, most, largest, _) = found.shade.unwrap();
        assert!(
            (mean - 8.0).abs() < 1e-9 && (most - 8.0).abs() < 1e-9 && (largest - 8.0).abs() < 1e-9
        );
        // Facing the other way: every sample and every corner.
        let found = across(
            &a,
            &Surface::new(&square(0.0, false, 1.0)),
            &TOLERANCE,
            1.0,
            32.0,
        );
        assert_eq!((found.turned, found.away), (1.0, 10004));
        assert!(found.max < 1e-12);
        // Against nothing.
        let nothing = part(Vec::new(), Vec::new());
        assert_eq!(
            across(&a, &Surface::new(&nothing), &TOLERANCE, 1.0, 32.0).max,
            f64::INFINITY
        );
        // Where two faces lie in one place, the texture and the shading are either's.
        let two = joined(square(0.0, true, 0.25), square(0.0, true, 1.0));
        let found = across(&a, &Surface::new(&two), &TOLERANCE, 1.0, 32.0);
        assert_eq!(found.shade.unwrap().0, 0.0);
    }

    #[test]
    fn a_track_is_read_between_its_keys() {
        let half = std::f64::consts::FRAC_1_SQRT_2;
        let turn_to = Track {
            times: vec![0.0, 1.0],
            keys: vec![vec![0.0, 0.0, 0.0, 1.0], vec![0.0, half, 0.0, half]],
            step: false,
        };
        let at = |t: f64| -> [f64; 4] { turn_to.at(t, true).unwrap().try_into().unwrap() };
        assert!((turn([0.0, 0.0, 0.0, 1.0], at(0.5)) - 45.0).abs() < 1e-9);
        assert!((turn(at(0.25), at(1.0)) - 67.5).abs() < 1e-9);
        assert_eq!(at(-1.0), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(at(2.0), [0.0, half, 0.0, half]);
        // q and -q are one turn.
        assert!(turn([0.0, half, 0.0, half], [0.0, -half, 0.0, -half]) < 1e-6);
        let slide = Track {
            times: vec![1.0, 2.0, 4.0],
            keys: vec![vec![0.0], vec![1.0], vec![5.0]],
            step: false,
        };
        assert_eq!(slide.at(3.0, false), Some(vec![3.0]));
        let held = Track {
            step: true,
            ..slide
        };
        assert_eq!(held.at(3.0, false), Some(vec![1.0]));
        let none = Track {
            times: Vec::new(),
            keys: Vec::new(),
            step: false,
        };
        assert_eq!(none.at(0.0, false), None);
    }

    #[test]
    fn a_node_s_rest_is_read_from_its_matrix_too() {
        let half = std::f64::consts::FRAC_1_SQRT_2;
        let gltf = super::super::json::loads_text(
            br#"{"nodes": [{"translation": [1, 2, 3], "rotation": [0, 0.7071067811865476, 0, 0.7071067811865476], "scale": [2, 2, 2]},
                {"matrix": [0, 0, -2, 0, 0, 2, 0, 0, 2, 0, 0, 0, 1, 2, 3, 1]}, {"children": [0]}]}"#,
        )
        .unwrap();
        let rests = rests(&gltf).unwrap();
        assert_eq!(rests[1].translation, [1.0, 2.0, 3.0]);
        assert_eq!(rests[1].scale, [2.0, 2.0, 2.0]);
        assert!(turn(rests[1].rotation, [0.0, half, 0.0, half]) < 1e-6);
        assert_eq!(rests[2].rotation, [0.0, 0.0, 0.0, 1.0]);
        // A quarter turn about y takes x to -z; a parent's place is added.
        let tree = super::super::digest::tree(&gltf).unwrap();
        let world = worlds(&tree, &rests);
        let x = part(vec![[1.0, 0.0, 0.0]], Vec::new())
            .placed(&world[0])
            .positions[0];
        assert!(py_dist(x, [1.0, 2.0, 1.0]) < 1e-9, "{x:?}");
        assert!(
            world[0]
                .iter()
                .zip(&world[1])
                .all(|(a, b)| (a - b).abs() < 1e-9),
            "{world:?}"
        );
    }

    fn py_dist(a: P3, b: P3) -> f64 {
        length(sub(a, b))
    }
}
