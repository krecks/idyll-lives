//! `cargo review facets`: every model's round parts whose outline shows its flat sides.
//!
//! A round thing is drawn as flat faces with their normals averaged across the edges between them, so it
//! shades as round; its outline against what is behind it is still the polygon. How far the true curve stands
//! off that polygon (the sagitta) is what the eye sees as corners: across a smooth edge whose two faces turn
//! by an angle θ, with the narrower of them `w` wide across the edge, the arc touching both stands
//! `(w / 2) × tan(θ / 4)` off the chord (for a regular polygon that is exactly `r × (1 − cos(π / n))`). Over
//! 1.5 mm it shows in a catalogue picture and at the play camera zoomed in (a pixel there is 4 mm; an outline
//! is seen to a fraction of one).
//!
//! This lists them: for every model the manifests name, the smooth edges (two triangles sharing an edge whose
//! corners there have the same normal) over that, by how many, how long and how far, summed per family, the
//! worst models first. What is drawn as leaves (the `foliage` and `leafcards` finishes: lumps and cards, not
//! round things) is counted apart. A report, not a gate: `--check` makes it one.
//!
//! ```text
//! cargo review facets [--all] [--check]
//! ```

use serde_json::Value as J;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// How far a curve may stand off its chord before its outline shows flat sides, in metres.
pub const SAGITTA: f64 = 0.0015;
/// A pixel at the play camera zoomed all the way in (6 m), in metres: facets over this are corners anyone sees.
pub const PIXEL: f64 = 0.004;
/// Two faces turning less than this (degrees) are one flat face cut in two.
const FLAT: f64 = 0.5;

type V3 = [f64; 3];

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(a: V3) -> f64 {
    dot(a, a).sqrt()
}

/// A smooth edge whose outline stands off the curve it is drawn for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Facet {
    /// How far the arc stands off the chord, in metres.
    pub sagitta: f64,
    /// The edge's length, in metres.
    pub length: f64,
}

/// The smooth edges of a triangle mesh and how far the curve they stand for is off each: `positions` as drawn
/// (metres), `normals` as the file has them (only compared), `triangles` by vertex. An edge is smooth where
/// its two triangles meet in corners of the same place and normal (the file splits a vertex at a sharp edge).
/// Every such edge that turns is returned, whatever its sagitta.
pub fn facets(positions: &[V3], normals: &[V3], triangles: &[[u32; 3]]) -> Vec<Facet> {
    // Corners that are one vertex to the eye: the same place and normal (a UV seam splits them in the file).
    let mut ids: HashMap<[i64; 6], u32> = HashMap::new();
    let weld: Vec<u32> = positions
        .iter()
        .zip(normals)
        .map(|(p, n)| {
            let key = [
                (p[0] * 1e5).round() as i64,
                (p[1] * 1e5).round() as i64,
                (p[2] * 1e5).round() as i64,
                (n[0] * 1e3).round() as i64,
                (n[1] * 1e3).round() as i64,
                (n[2] * 1e3).round() as i64,
            ];
            let next = ids.len() as u32;
            *ids.entry(key).or_insert(next)
        })
        .collect();
    let mut place: Vec<V3> = vec![[0.0; 3]; ids.len()];
    for (i, &w) in weld.iter().enumerate() {
        place[w as usize] = positions[i];
    }
    // Each edge's triangles: (the face's normal, the corner across the edge).
    let mut edges: HashMap<(u32, u32), Vec<(V3, u32)>> = HashMap::new();
    for t in triangles {
        let Some(c) = t
            .iter()
            .map(|&i| weld.get(i as usize).copied())
            .collect::<Option<Vec<u32>>>()
        else {
            continue;
        };
        if c[0] == c[1] || c[1] == c[2] || c[0] == c[2] {
            continue;
        }
        let [a, b, d] = [
            place[c[0] as usize],
            place[c[1] as usize],
            place[c[2] as usize],
        ];
        let n = cross(sub(b, a), sub(d, a));
        let area = length(n);
        if area < 1e-12 {
            continue;
        }
        let n = [n[0] / area, n[1] / area, n[2] / area];
        for k in 0..3 {
            let (e0, e1, across) = (c[k], c[(k + 1) % 3], c[(k + 2) % 3]);
            edges
                .entry((e0.min(e1), e0.max(e1)))
                .or_default()
                .push((n, across));
        }
    }
    let mut out = Vec::new();
    for ((e0, e1), faces) in &edges {
        let [(n1, a1), (n2, a2)] = faces[..] else {
            continue;
        };
        let turn = dot(n1, n2).clamp(-1.0, 1.0).acos();
        if turn < FLAT.to_radians() {
            continue;
        }
        let (p0, p1) = (place[*e0 as usize], place[*e1 as usize]);
        let along = sub(p1, p0);
        let len = length(along);
        if len < 1e-9 {
            continue;
        }
        // How wide each face is across the edge: its far corner's distance from the edge's line.
        let width = |corner: u32| length(cross(sub(place[corner as usize], p0), along)) / len;
        let w = width(a1).min(width(a2));
        out.push(Facet {
            sagitta: w / 2.0 * (turn / 4.0).tan(),
            length: len,
        });
    }
    out
}

/// What a model's faceted edges add up to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Sum {
    /// Smooth edges over `SAGITTA`.
    pub edges: usize,
    /// Their length together, in metres.
    pub length: f64,
    /// Of that, the length over `PIXEL`.
    pub coarse: f64,
    /// The worst sagitta, in metres.
    pub worst: f64,
}

impl Sum {
    fn add(&mut self, f: &Facet) {
        if f.sagitta > SAGITTA {
            self.edges += 1;
            self.length += f.length;
            if f.sagitta > PIXEL {
                self.coarse += f.length;
            }
            self.worst = self.worst.max(f.sagitta);
        }
    }

    fn merge(&mut self, o: &Sum) {
        self.edges += o.edges;
        self.length += o.length;
        self.coarse += o.coarse;
        self.worst = self.worst.max(o.worst);
    }
}

/// One model's facets: its round parts, and what is drawn as leaves apart.
#[derive(Debug, Clone, Default)]
pub struct Found {
    pub key: String,
    pub family: String,
    pub round: Sum,
    pub leaves: Sum,
    /// The part and material of the worst round edge.
    pub where_: String,
    pub triangles: usize,
}

type Matrix = [[f64; 4]; 4];

const IDENTITY: Matrix = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn mul(a: &Matrix, b: &Matrix) -> Matrix {
    let mut m = [[0.0; 4]; 4];
    for (r, row) in m.iter_mut().enumerate() {
        for (c, v) in row.iter_mut().enumerate() {
            *v = (0..4).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    m
}

fn floats(v: Option<&J>, n: usize) -> Option<Vec<f64>> {
    let a = v?.as_array()?;
    (a.len() == n).then(|| a.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect())
}

/// A node's own matrix (row-major): its `matrix`, or its translation, rotation and scale.
fn local(node: &J) -> Matrix {
    if let Some(m) = floats(node.get("matrix"), 16) {
        let mut out = IDENTITY;
        for (r, row) in out.iter_mut().enumerate() {
            for (c, v) in row.iter_mut().enumerate() {
                *v = m[c * 4 + r]; // (glTF writes columns first)
            }
        }
        return out;
    }
    let t = floats(node.get("translation"), 3).unwrap_or_else(|| vec![0.0; 3]);
    let q = floats(node.get("rotation"), 4).unwrap_or_else(|| vec![0.0, 0.0, 0.0, 1.0]);
    let s = floats(node.get("scale"), 3).unwrap_or_else(|| vec![1.0; 3]);
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let rot = [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - x * w),
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ];
    let mut out = IDENTITY;
    for r in 0..3 {
        for c in 0..3 {
            out[r][c] = rot[r][c] * s[c];
        }
        out[r][3] = t[r];
    }
    out
}

/// A `.glb`'s JSON and binary chunks.
fn chunks(file: &Path) -> Result<(J, Vec<u8>), String> {
    let data = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let word = |at: usize| {
        data.get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
    };
    let (mut json, mut bin, mut at) = (None, Vec::new(), 12);
    while let (Some(n), Some(kind)) = (word(at), word(at + 4)) {
        let chunk = data
            .get(at + 8..at + 8 + n)
            .ok_or_else(|| format!("{}: a chunk ends early", file.display()))?;
        match kind {
            0x4E4F534A => {
                json = Some(
                    serde_json::from_slice(chunk)
                        .map_err(|e| format!("{}: {e}", file.display()))?,
                )
            }
            0x004E4942 => bin = chunk.to_vec(),
            _ => {}
        }
        at += 8 + n;
    }
    Ok((
        json.ok_or_else(|| format!("{}: not a .glb", file.display()))?,
        bin,
    ))
}

/// An accessor's elements as numbers, `width` to each (floats and unsigned integers, as a source file has them).
fn accessor(gltf: &J, bin: &[u8], index: &J, width: usize) -> Option<Vec<f64>> {
    let acc = gltf.get("accessors")?.get(index.as_u64()? as usize)?;
    let view = gltf
        .get("bufferViews")?
        .get(acc.get("bufferView")?.as_u64()? as usize)?;
    let size = match acc.get("componentType")?.as_u64()? {
        5126 | 5125 => 4,
        5123 => 2,
        5121 => 1,
        _ => return None,
    };
    let float = acc.get("componentType")?.as_u64()? == 5126;
    let offset = |o: &J| o.get("byteOffset").and_then(J::as_u64).unwrap_or(0) as usize;
    let stride = view
        .get("byteStride")
        .and_then(J::as_u64)
        .filter(|&s| s > 0)
        .map_or(size * width, |s| s as usize);
    let count = acc.get("count")?.as_u64()? as usize;
    let start = offset(view) + offset(acc);
    let mut out = Vec::with_capacity(count * width);
    for i in 0..count {
        for k in 0..width {
            let at = start + i * stride + k * size;
            let b = bin.get(at..at + size)?;
            out.push(match (size, float) {
                (4, true) => f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
                (4, false) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
                (2, _) => u16::from_le_bytes([b[0], b[1]]) as f64,
                _ => b[0] as f64,
            });
        }
    }
    Some(out)
}

/// A material's finish, by the entry's `materials` map or its own name (`leaf@foliage`), without the prefix.
fn finish_of(materials: Option<&J>, name: &str) -> String {
    materials
        .and_then(|map| map.get(name).or_else(|| map.get("*")))
        .and_then(|o| o.get("finish"))
        .and_then(J::as_str)
        .map(|k| k.trim_start_matches("material.finish.").to_string())
        .or_else(|| {
            let tail = name.split_once('@')?.1;
            Some(tail.split('#').next().unwrap_or(tail).to_string())
        })
        .unwrap_or_default()
}

/// A model's facets: `materials` is its manifest entry's map, `gltf` and `bin` its file's chunks.
pub fn of_model(key: &str, family: &str, materials: Option<&J>, gltf: &J, bin: &[u8]) -> Found {
    let mut found = Found {
        key: key.to_string(),
        family: family.to_string(),
        ..Found::default()
    };
    let nodes = gltf
        .get("nodes")
        .and_then(J::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let list = |o: &J, k: &str| -> Vec<usize> {
        o.get(k)
            .and_then(J::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_u64().map(|v| v as usize))
                    .collect()
            })
            .unwrap_or_default()
    };
    let scene = gltf.get("scene").and_then(J::as_u64).unwrap_or(0) as usize;
    let mut todo: Vec<(usize, Matrix)> = gltf
        .get("scenes")
        .and_then(|s| s.get(scene))
        .map(|s| list(s, "nodes"))
        .unwrap_or_default()
        .into_iter()
        .map(|i| (i, IDENTITY))
        .collect();
    let mut steps = 0;
    while let Some((i, parent)) = todo.pop() {
        steps += 1;
        let Some(node) = nodes.get(i).filter(|_| steps < 100_000) else {
            continue;
        };
        let world = mul(&parent, &local(node));
        todo.extend(list(node, "children").into_iter().map(|c| (c, world)));
        let Some(mesh) = node
            .get("mesh")
            .and_then(J::as_u64)
            .and_then(|m| gltf.get("meshes")?.get(m as usize))
        else {
            continue;
        };
        let part = node.get("name").and_then(J::as_str).unwrap_or("");
        for prim in mesh
            .get("primitives")
            .and_then(J::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
        {
            if prim.get("mode").and_then(J::as_u64).unwrap_or(4) != 4 {
                continue;
            }
            let attribute = |name: &str| {
                accessor(gltf, bin, prim.get("attributes")?.get(name)?, 3)
                    .map(|v| v.as_chunks::<3>().0.to_vec())
            };
            let (Some(positions), Some(normals)) = (attribute("POSITION"), attribute("NORMAL"))
            else {
                continue;
            };
            let positions: Vec<V3> = positions
                .iter()
                .map(|p| {
                    let mut q = [0.0; 3];
                    for (r, v) in q.iter_mut().enumerate() {
                        *v = world[r][0] * p[0]
                            + world[r][1] * p[1]
                            + world[r][2] * p[2]
                            + world[r][3];
                    }
                    q
                })
                .collect();
            let indices: Vec<u32> = match prim.get("indices") {
                Some(i) => accessor(gltf, bin, i, 1)
                    .unwrap_or_default()
                    .iter()
                    .map(|&v| v as u32)
                    .collect(),
                None => (0..positions.len() as u32).collect(),
            };
            let triangles: Vec<[u32; 3]> = indices.as_chunks::<3>().0.to_vec();
            found.triangles += triangles.len();
            let material = prim
                .get("material")
                .and_then(J::as_u64)
                .and_then(|m| {
                    gltf.get("materials")?
                        .get(m as usize)?
                        .get("name")?
                        .as_str()
                })
                .unwrap_or("");
            let finish = finish_of(materials, material);
            let leaves = finish == "foliage" || finish == "leafcards";
            let mut sum = Sum::default();
            for f in facets(&positions, &normals, &triangles) {
                sum.add(&f);
            }
            if leaves {
                found.leaves.merge(&sum);
            } else {
                if sum.worst > found.round.worst {
                    found.where_ = format!("{part} ({material})");
                }
                found.round.merge(&sum);
            }
        }
    }
    found
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every model the manifests under `assets` name (not their far versions), measured.
pub fn survey(assets: &Path) -> Result<Vec<Found>, String> {
    let mut files = Vec::new();
    crate::colours::manifests(assets, &mut files);
    let mut out = Vec::new();
    for file in files {
        let manifest: J = serde_json::from_slice(
            &std::fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?,
        )
        .map_err(|e| format!("{}: {e}", file.display()))?;
        let dir = file.parent().unwrap_or(assets);
        let Some(entries) = manifest.get("entries").and_then(J::as_object) else {
            continue;
        };
        for (key, entry) in entries {
            let Some(url) = entry
                .get("url")
                .and_then(J::as_str)
                .filter(|_| entry.get("type").and_then(J::as_str) == Some("model"))
            else {
                continue;
            };
            let glb = dir.join(url);
            let family = glb
                .parent()
                .and_then(|p| p.file_name())
                .map_or(String::new(), |n| n.to_string_lossy().into_owned());
            let (gltf, bin) = chunks(&glb)?;
            out.push(of_model(key, &family, entry.get("materials"), &gltf, &bin));
        }
    }
    Ok(out)
}

fn mm(v: f64) -> String {
    format!("{:.1}", v * 1000.0)
}

/// The report's text.
pub fn report(found: &[Found], all: bool) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    let mut total = Sum::default();
    let mut leaves = Sum::default();
    let mut families: BTreeMap<&str, (usize, usize, Sum)> = BTreeMap::new();
    for f in found {
        total.merge(&f.round);
        leaves.merge(&f.leaves);
        let family = families.entry(&f.family).or_default();
        family.0 += 1;
        family.1 += usize::from(f.round.edges > 0);
        family.2.merge(&f.round);
    }
    let faceted = found.iter().filter(|f| f.round.edges > 0).count();
    let _ = writeln!(
        text,
        "{} models, {faceted} with a round part whose outline stands more than {} mm off its curve: {} smooth edges, {:.0} m of them ({:.0} m over {} mm, a pixel at the play camera zoomed in), the worst {} mm\n(leaves apart, lumps and cards rather than round things: {} edges, {:.0} m)\n",
        found.len(),
        mm(SAGITTA),
        total.edges,
        total.length,
        total.coarse,
        mm(PIXEL),
        mm(total.worst),
        leaves.edges,
        leaves.length,
    );
    let _ = writeln!(
        text,
        "| family | models | faceted | edges | metres | over {} mm | worst mm |\n|---|---|---|---|---|---|---|",
        mm(PIXEL)
    );
    let mut rows: Vec<_> = families.iter().collect();
    rows.sort_by(|a, b| b.1.2.length.total_cmp(&a.1.2.length).then(a.0.cmp(b.0)));
    for (family, (models, faceted, sum)) in rows {
        let _ = writeln!(
            text,
            "| {family} | {models} | {faceted} | {} | {:.1} | {:.1} | {} |",
            sum.edges,
            sum.length,
            sum.coarse,
            mm(sum.worst)
        );
    }
    let mut worst: Vec<&Found> = found.iter().filter(|f| f.round.edges > 0).collect();
    worst.sort_by(|a, b| {
        b.round
            .length
            .total_cmp(&a.round.length)
            .then(a.key.cmp(&b.key))
    });
    let n = if all { usize::MAX } else { 25 };
    let _ = writeln!(
        text,
        "\nThe most faceted{}:\n\n| model | triangles | edges | metres | over {} mm | worst mm | worst at |\n|---|---|---|---|---|---|---|",
        if all {
            ""
        } else {
            " (25; `--all` for every one)"
        },
        mm(PIXEL)
    );
    for f in worst.iter().take(n) {
        let _ = writeln!(
            text,
            "| {} | {} | {} | {:.1} | {:.1} | {} | {} |",
            f.key,
            f.triangles,
            f.round.edges,
            f.round.length,
            f.round.coarse,
            mm(f.round.worst),
            f.where_
        );
    }
    text
}

/// `cargo review facets [--all] [--check]`: the exit code (1 with `--check` when a round part is faceted).
pub fn main(args: &[String]) -> i32 {
    let (all, check) = (
        args.iter().any(|a| a == "--all"),
        args.iter().any(|a| a == "--check"),
    );
    let found = match survey(&root().join("web/public/assets")) {
        Ok(found) => found,
        Err(e) => {
            eprintln!("review facets: {e}");
            return 1;
        }
    };
    print!("{}", report(&found, all));
    i32::from(check && found.iter().any(|f| f.round.edges > 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// An upright cylinder's side, `sides` round: quads cut in two, their normals pointing out from the axis.
    fn cylinder(radius: f64, height: f64, sides: u32) -> (Vec<V3>, Vec<V3>, Vec<[u32; 3]>) {
        let (mut positions, mut normals, mut triangles) = (Vec::new(), Vec::new(), Vec::new());
        for i in 0..sides {
            let a = std::f64::consts::TAU * i as f64 / sides as f64;
            for y in [0.0, height] {
                positions.push([radius * a.cos(), y, radius * a.sin()]);
                normals.push([a.cos(), 0.0, a.sin()]);
            }
        }
        for i in 0..sides {
            let (a, b) = (2 * i, 2 * ((i + 1) % sides));
            triangles.push([a, a + 1, b + 1]);
            triangles.push([a, b + 1, b]);
        }
        (positions, normals, triangles)
    }

    fn sum(mesh: &(Vec<V3>, Vec<V3>, Vec<[u32; 3]>)) -> Sum {
        let mut sum = Sum::default();
        for f in facets(&mesh.0, &mesh.1, &mesh.2) {
            sum.add(&f);
        }
        sum
    }

    #[test]
    fn a_leg_of_eight_sides_shows_and_one_of_twenty_four_does_not() {
        // 24 mm round: 8 sides stand 1.8 mm off the circle, 24 sides 0.2 mm.
        let coarse = sum(&cylinder(0.024, 0.4, 8));
        assert_eq!(coarse.edges, 8);
        assert!((coarse.length - 8.0 * 0.4).abs() < 1e-9);
        let exact = 0.024 * (1.0 - (std::f64::consts::PI / 8.0).cos());
        assert!((coarse.worst - exact).abs() < 1e-9, "{}", coarse.worst);
        assert!(coarse.worst > SAGITTA && coarse.worst < PIXEL);
        assert_eq!(coarse.coarse, 0.0);
        assert_eq!(sum(&cylinder(0.024, 0.4, 24)), Sum::default());
    }

    #[test]
    fn the_sagitta_is_the_polygon_s() {
        // Every side edge of a regular polygon: r × (1 − cos(π / n)); the cuts across its quads are flat.
        for (radius, sides) in [(0.1, 12), (0.5, 32), (0.02, 6)] {
            let mesh = cylinder(radius, 1.0, sides);
            let found = facets(&mesh.0, &mesh.1, &mesh.2);
            assert_eq!(found.len(), sides as usize);
            let exact = radius * (1.0 - (std::f64::consts::PI / sides as f64).cos());
            for f in found {
                assert!((f.sagitta - exact).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn a_sharp_edge_is_not_a_facet() {
        // The same cylinder with each side's corners its own (flat shaded, as a file writes a sharp edge).
        let (positions, _, triangles) = cylinder(0.024, 0.4, 8);
        let (mut p, mut n, mut t) = (Vec::new(), Vec::new(), Vec::new());
        for tri in &triangles {
            let [a, b, c] = tri.map(|i| positions[i as usize]);
            let face = cross(sub(b, a), sub(c, a));
            let at = p.len() as u32;
            p.extend([a, b, c]);
            n.extend([face; 3].map(|f| {
                let l = length(f);
                [f[0] / l, f[1] / l, f[2] / l]
            }));
            t.push([at, at + 1, at + 2]);
        }
        assert_eq!(sum(&(p, n, t)), Sum::default());
    }

    #[test]
    fn a_seam_in_the_file_is_still_smooth() {
        // A vertex written twice (a UV seam) with the same place and normal joins its two faces.
        let (mut positions, mut normals, mut triangles) = cylinder(0.024, 0.4, 8);
        let at = positions.len() as u32;
        positions.extend([positions[0], positions[1]]);
        normals.extend([normals[0], normals[1]]);
        triangles[0] = [at, at + 1, 3];
        triangles[1] = [at, 3, 2];
        assert_eq!(sum(&(positions, normals, triangles)).edges, 8);
    }

    #[test]
    fn a_narrow_bevel_beside_a_wide_face_is_measured_by_the_bevel() {
        // A 3 mm face turning 30° from a 1 m one: 0.2 mm, whatever the wide face's width.
        let turn = 30f64.to_radians();
        let positions = vec![
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.003 * turn.cos(), 0.0, -0.003 * turn.sin()],
        ];
        let normals = vec![[0.0, 0.0, 1.0]; 4];
        let found = facets(&positions, &normals, &[[0, 1, 2], [1, 0, 3]]);
        assert_eq!(found.len(), 1);
        assert!((found[0].sagitta - 0.0015 * (turn / 4.0).tan()).abs() < 1e-9);
        assert!(found[0].sagitta < SAGITTA);
    }

    #[test]
    fn a_model_is_read_through_its_nodes_and_leaves_are_apart() {
        // One cylinder's side, 8 round at 24 mm, as two primitives of a node drawn at twice its size.
        let (positions, normals, triangles) = cylinder(0.024, 0.4, 8);
        let mut bin = Vec::new();
        for v in positions.iter().chain(&normals) {
            for c in v {
                bin.extend((*c as f32).to_le_bytes());
            }
        }
        for t in &triangles {
            for i in t {
                bin.extend((*i as u16).to_le_bytes());
            }
        }
        let n = positions.len();
        let gltf = json!({
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "name": "top", "scale": [2.0, 2.0, 2.0], "children": [1] }, { "name": "leg", "mesh": 0 }],
            "meshes": [{ "primitives": [
                { "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "material": 0 },
                { "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "material": 1 },
            ] }],
            "materials": [{ "name": "oak" }, { "name": "leaf" }],
            "accessors": [
                { "bufferView": 0, "componentType": 5126, "count": n, "type": "VEC3" },
                { "bufferView": 1, "componentType": 5126, "count": n, "type": "VEC3" },
                { "bufferView": 2, "componentType": 5123, "count": triangles.len() * 3, "type": "SCALAR" },
            ],
            "bufferViews": [
                { "buffer": 0, "byteOffset": 0, "byteLength": n * 12 },
                { "buffer": 0, "byteOffset": n * 12, "byteLength": n * 12 },
                { "buffer": 0, "byteOffset": n * 24, "byteLength": triangles.len() * 6 },
            ],
        });
        let materials = json!({ "*": { "finish": "material.finish.wood" }, "leaf": { "finish": "material.finish.foliage" } });
        let found = of_model("model.x", "living", Some(&materials), &gltf, &bin);
        assert_eq!(found.triangles, 32);
        assert_eq!(found.round.edges, 8);
        assert_eq!(found.leaves.edges, 8);
        assert_eq!(found.where_, "leg (oak)");
        // Twice the size: twice the sagitta, twice the length.
        let exact = 0.048 * (1.0 - (std::f64::consts::PI / 8.0).cos());
        assert!((found.round.worst - exact).abs() < 1e-6);
        assert!((found.round.length - 8.0 * 0.8).abs() < 1e-6);
        let text = report(&[found], false);
        assert!(
            text.contains("| living | 1 | 1 | 8 | 6.4 | 0.0 | 3.7 |"),
            "{text}"
        );
        assert!(
            text.contains("| model.x | 32 | 8 | 6.4 | 0.0 | 3.7 | leg (oak) |"),
            "{text}"
        );
    }

    #[test]
    fn the_shipped_models_are_read() {
        let found = survey(&root().join("web/public/assets")).unwrap();
        assert!(found.len() > 600, "{} models", found.len());
        assert!(found.iter().all(|f| f.triangles > 0));
        // (A report for now: what the kit's rounder defaults and the soft forms bring down.)
        assert!(found.iter().any(|f| f.round.edges == 0));
    }
}
