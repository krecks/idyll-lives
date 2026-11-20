//! `cargo check-glb --built <dir>`: the gate for the built world, the geometry the game makes as it runs (houses, the
//! neighbours' and the far houses, the lots' buildings closed, roads, sidewalks, junctions, bridges, fences, cars),
//! held to the rules the models are held to.
//!
//! The game builds all of it in TypeScript (web/src/render/shared), so there is no file to read as a model's is.
//! `bun tools/built/dump.ts <dir>` writes a fixed set of samples there, made by the game's own generators, a `.glb`
//! each and `built.json`, which lists them; this reads them as plain meshes and runs
//!
//! - `built.edges`: the models' hard-edge rule (`edges::hard_edges`). A house is not seen as a model is: its roof
//!   shows only from 15 m out, and nearer its walls stand, or those on the camera's side drop to stubs (the
//!   cutaway, by the way the camera looks). So a house is judged as it is drawn in each of those states, by the
//!   cameras that see it so (`house_edges`), and an edge counts once however many states show it;
//! - `built.texels`: the models' texel rule (`texels::measure`) over each part on a textured material, with the
//!   manifest's `uvScale` and the texture's size (a material projected from above takes its UVs from x and z);
//! - `built.farNear`: each far version against what it is the far version of (the neighbour's silhouette, the
//!   street's far house and the town view's box against the viewed house; a lot's building's box against it
//!   dressed): its plan, its height, its roof's kind and pitch, a chimney on both or neither, its walls' and its
//!   roof's colour (`Look`);
//!
//! and counts each sample's triangles and draw calls. Their findings are counted per sample against
//! tools/art/gate-baseline.json (`baseline`, sections by those names): at its count a sample's findings are printed,
//! over it or in a sample the file doesn't have they fail, under it they fail as stale until `--update-baseline`
//! lowers it. Far versions are seen from too far for an edge or a texel to show, and are judged by `built.farNear`
//! alone.
//!
//!   cargo check-glb --built <dir> [sample keys…]
//!   cargo check-glb --built <dir> --report            what it measures, as tables: where the edges are, each
//!                                                     material's texels, every pair's two looks
//!   cargo check-glb --built <dir> --update-baseline   first lowers the baseline to what it finds
//!   cargo check-glb --built <dir> --seed-baseline     first writes the sections the baseline has none of (the day
//!                                                     a check comes in)

use super::baseline::{self, Baseline, Counts, Found};
use super::digest::raw;
use super::glb::{Glb, R, image_size, item, key, parse_glb};
use super::py::{J, dirname, join, normpath};
use super::{Paths, asset_manifests};
use crate::edges::{self, Edge, Mesh, V3};
use crate::texels::{self, Texture, Textures};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

/// The checks' names: their sections in the baseline.
pub const EDGES: &str = "built.edges";
pub const TEXELS: &str = "built.texels";
pub const FAR_NEAR: &str = "built.farNear";
/// The baseline's sections that are this gate's (`checkglb::COUNTED` are the models').
pub const CHECKS: [&str; 3] = [EDGES, TEXELS, FAR_NEAR];

/// The sample kinds that are far versions: judged against what they are the far version of, not for edges or texels.
const FAR: [&str; 4] = ["neighbour", "far", "town", "buildingFar"];
/// The far versions drawn from so far that no edge or texel of theirs shows (a neighbour's silhouette is across
/// the street: it is judged for both).
const DISTANT: [&str; 3] = ["far", "town", "buildingFar"];

/// What `built.json` says of a sample.
#[derive(serde::Deserialize, Clone, Debug, Default)]
pub struct Listed {
    pub key: String,
    pub kind: String,
    /// A far version: the sample it is the far version of.
    #[serde(default)]
    pub of: Option<String>,
    #[serde(default)]
    pub storeys: Option<u32>,
    #[serde(default)]
    pub about: String,
}

/// `built.json`: the samples, a storey's height and the height walls drop to in the cutaway (m).
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Index {
    pub storey: f64,
    pub stub: f64,
    pub samples: Vec<Listed>,
}

/// A part of a sample: one draw call's mesh (its `material` what it is drawn with), and what the dump says of it.
#[derive(Clone, Debug, Default)]
pub struct Part {
    pub mesh: Mesh,
    /// Its vertices' colours (sRGB, with the alpha as dumped).
    pub colors: Vec<[u8; 4]>,
    /// The cutaway attribute where it follows the cutaway: mask A, kind (1 drops to the stub, 2 goes), mask B.
    pub cut: Vec<[u8; 4]>,
    /// The storey of a house it is on.
    pub storey: u32,
    /// A roof or a gable: hidden while the camera is near.
    pub roof: bool,
    /// Only the sun's shadow map draws it.
    pub shadow_only: bool,
}

/// What a material is to the rules: a manifest's, or one of the dump's own (tools/built/samples.ts `BUILT_MATERIALS`).
#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    /// How often its texture repeats over a metre of UV, and the texture's file (None: no texture).
    pub uv_scale: f64,
    pub texture: Option<String>,
    /// Its UVs are x and z, whatever the mesh has.
    pub planar: bool,
    /// Its colour, the vertex colours' multiplier (sRGB, 0..1).
    pub color: [f64; 3],
    pub alpha: f64,
}

impl Default for Surface {
    fn default() -> Surface {
        Surface {
            uv_scale: 1.0,
            texture: None,
            planar: false,
            color: [1.0; 3],
            alpha: 1.0,
        }
    }
}

/// The manifests' materials by key.
pub type Surfaces = BTreeMap<String, Surface>;

/// Whether a material is glass: its edges are not judged (as the models' glass).
fn glass(material: &str) -> bool {
    material.starts_with("material.window")
}

/// Whether a material is laid on what is under it and blended (contact shadows, pools of lamplight): it hides
/// nothing and has no edges.
fn laid(material: &str) -> bool {
    matches!(material, "built.shadow" | "built.lightPool")
}

/// Whether a material's edges are judged: not glass, nothing laid or painted on the ground, nothing that glows.
fn edged(material: &str) -> bool {
    !glass(material) && !laid(material) && !matches!(material, "built.glow" | "built.roadPaint")
}

/// Whether a material hides what is behind it.
fn opaque(material: &str, surfaces: &Surfaces) -> bool {
    !laid(material) && surfaces.get(material).is_none_or(|s| s.alpha >= 1.0)
}

/// The manifests' materials (`entries` of type `material`, a pack's over the base's).
pub fn surfaces(paths: &Paths) -> Result<Surfaces, String> {
    let mut out = Surfaces::new();
    for (path, m) in asset_manifests(paths)? {
        for (k, entry) in m.get("entries").and_then(J::entries).unwrap_or(&[]) {
            if entry.get("type") != Some(&J::Str("material".into())) {
                continue;
            }
            let number = |name: &str, default: f64| {
                entry.get(name).and_then(J::num).map_or(default, |n| n.f())
            };
            let color = entry
                .get("color")
                .and_then(J::str)
                .and_then(hex)
                .map_or([1.0; 3], |c| c.map(|v| v as f64 / 255.0));
            out.insert(
                k.clone(),
                Surface {
                    uv_scale: number("uvScale", 1.0),
                    texture: entry
                        .get("texture")
                        .and_then(J::str)
                        .filter(|t| !t.is_empty())
                        .map(|t| normpath(&join(&dirname(&path), t))),
                    planar: entry.get("projection").and_then(J::str) == Some("xz"),
                    color,
                    alpha: number("alpha", 1.0),
                },
            );
        }
    }
    Ok(out)
}

/// `#rrggbb` as bytes.
fn hex(s: &str) -> Option<[u8; 3]> {
    let v = u32::from_str_radix(s.strip_prefix('#').filter(|h| h.len() == 6)?, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

/// The surfaces' textures as the rules need them, by file, each read once: a texture's side (px, the root of its
/// width times its height) and its mean colour (linear, 0..1); None where the file can't be read.
#[derive(Default)]
pub struct Pictures {
    sides: HashMap<String, Option<f64>>,
    means: HashMap<String, Option<[f64; 3]>>,
}

impl Pictures {
    fn side(&mut self, file: &str) -> Option<f64> {
        *self.sides.entry(file.to_string()).or_insert_with(|| {
            let (w, h) = image_size(&std::fs::read(file).ok()?)?;
            Some((w as f64 * h as f64).sqrt())
        })
    }

    fn mean(&mut self, file: &str) -> Option<[f64; 3]> {
        *self.means.entry(file.to_string()).or_insert_with(|| {
            let rgb = image::load_from_memory(&std::fs::read(file).ok()?)
                .ok()?
                .to_rgb8();
            let mut sum = [0.0; 3];
            for p in rgb.pixels() {
                for k in 0..3 {
                    sum[k] += linear(p.0[k] as f64 / 255.0);
                }
            }
            let n = (rgb.width() as f64 * rgb.height() as f64).max(1.0);
            Some(sum.map(|s| s / n))
        })
    }
}

/// sRGB (0..1) as linear light, and back.
fn linear(v: f64) -> f64 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb(v: f64) -> f64 {
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// A sample's parts, read from its `.glb` as the dump writes one (tools/built/glb.ts): a node a part, its mesh's one
/// primitive, the node's `extras`.
pub fn read(g: &Glb) -> R<Vec<Part>> {
    let gltf = &g.gltf;
    let none = J::Arr(Vec::new());
    let mut out = Vec::new();
    for node in gltf.get("nodes").and_then(J::list).unwrap_or(&[]) {
        let Some(mesh) = node.get("mesh") else {
            continue;
        };
        let name = node.get("name").and_then(J::str).unwrap_or_default();
        let extra = |k: &str| node.get("extras").and_then(|e| e.get(k));
        for prim in key(item(gltf, "meshes", mesh)?, "primitives")?
            .list()
            .unwrap_or(&[])
        {
            let attributes = key(prim, "attributes")?;
            let rows = |name: &str, width: usize| -> R<Vec<f64>> {
                match attributes.get(name) {
                    Some(i) => {
                        let a = raw(g, i)?;
                        if a.width != width {
                            return Err(format!("{name} of {} numbers, not {width}", a.width));
                        }
                        Ok(a.floats())
                    }
                    None => Ok(Vec::new()),
                }
            };
            let positions: Vec<V3> = rows("POSITION", 3)?.as_chunks::<3>().0.to_vec();
            let bytes = |v: Vec<f64>, scale: f64| -> Vec<[u8; 4]> {
                v.as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| c.map(|x| (x * scale).round().clamp(0.0, 255.0) as u8))
                    .collect()
            };
            let material = match prim.get("material") {
                Some(i) => super::glb::index(gltf.get("materials").unwrap_or(&none), i)?
                    .get("name")
                    .and_then(J::str)
                    .unwrap_or_default()
                    .to_string(),
                None => String::new(),
            };
            let triangles = super::digest::indices(g, prim, positions.len())?
                .as_chunks::<3>()
                .0
                .to_vec();
            out.push(Part {
                mesh: Mesh {
                    part: name.to_string(),
                    finish: material.clone(),
                    edged: edged(&material),
                    opaque: true,
                    material,
                    normals: rows("NORMAL", 3)?.as_chunks::<3>().0.to_vec(),
                    uvs: rows("TEXCOORD_0", 2)?.as_chunks::<2>().0.to_vec(),
                    positions,
                    triangles,
                },
                colors: bytes(rows("COLOR_0", 4)?, 255.0),
                cut: bytes(rows("_CUT", 4)?, 1.0),
                storey: extra("storey").and_then(J::num).map_or(0, |n| n.f() as u32),
                roof: extra("roof").is_some_and(J::truthy),
                shadow_only: extra("shadowOnly").is_some_and(J::truthy),
            });
        }
    }
    Ok(out)
}

// ---- edges --------------------------------------------------------------------------------------------

/// The parts a camera draws (not the shadow map's own), as the edge rule reads them: what hides what behind it.
fn drawn(parts: &[Part], surfaces: &Surfaces) -> Vec<Mesh> {
    parts
        .iter()
        .filter(|p| !p.shadow_only)
        .map(|p| Mesh {
            opaque: opaque(&p.mesh.material, surfaces),
            ..p.mesh.clone()
        })
        .collect()
}

/// The cutaway's look bits (web/src/render/shared/geometry.ts `LOOK`) for a camera the way `way` from what it looks at,
/// as the renderer sets them (`PlayCanvasRenderer.updateCutaway`: a bit where the camera looks more than 0.3 along
/// an axis). The dump's z is the game's mirrored.
fn look_bits(way: V3) -> u8 {
    let flat = (way[0] * way[0] + way[2] * way[2]).sqrt().max(1e-12);
    let (dx, dz) = (-way[0] / flat, way[2] / flat);
    u8::from(dx > 0.3)
        | u8::from(dx < -0.3) << 1
        | u8::from(dz > 0.3) << 2
        | u8::from(dz < -0.3) << 3
}

/// A house's parts as drawn with the camera near (no roofs) on storey `storey` (those above it hidden), the walls
/// cut away for a camera looking `look` (0: every wall standing): a vertex whose two masks both have a bit of
/// `look` drops to the stub's height over its storey's floor, or goes with its triangles (`shaders/cutaway.ts`).
fn cut_away(
    parts: &[Part],
    surfaces: &Surfaces,
    index: &Index,
    storey: u32,
    look: u8,
) -> Vec<Mesh> {
    parts
        .iter()
        .filter(|p| !p.shadow_only && !p.roof && p.storey <= storey)
        .map(|p| {
            let mut mesh = Mesh {
                opaque: opaque(&p.mesh.material, surfaces),
                ..p.mesh.clone()
            };
            if look == 0 || p.cut.len() != mesh.positions.len() {
                return mesh;
            }
            let cut = |i: usize| p.cut[i][0] & look != 0 && p.cut[i][2] & look != 0;
            let top = p.storey as f64 * index.storey + index.stub;
            for (i, at) in mesh.positions.iter_mut().enumerate() {
                if cut(i) && p.cut[i][1] == 1 {
                    at[1] = at[1].min(top);
                }
            }
            mesh.triangles.retain(|t| {
                !t.iter()
                    .any(|&i| cut(i as usize) && p.cut[i as usize][1] == 2)
            });
            mesh
        })
        .collect()
}

/// The line an edge lies on, to the millimetre: its part and material, its direction and where it passes
/// nearest the origin. The same edge found in two states of a house is on the same line.
type Line = (String, String, [i64; 3], [i64; 3]);

fn line_of(e: &Edge) -> Line {
    let first = e
        .along
        .iter()
        .find(|c| c.abs() > 1e-6)
        .copied()
        .unwrap_or(1.0);
    let along = e.along.map(|c| if first < 0.0 { -c } else { c });
    let on = (0..3).fold(0.0, |sum, k| sum + e.at[k] * along[k]);
    let foot = [0, 1, 2].map(|k| e.at[k] - along[k] * on);
    (
        e.part.clone(),
        e.material.clone(),
        along.map(|c| (c * 1e3).round() as i64),
        foot.map(|c| (c * 1e3).round() as i64),
    )
}

/// A sample's hard edges that show unbevelled, by the line they lie on: how many, how long together, and the
/// longest of them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Edges(pub BTreeMap<Line, (usize, f64, Option<Edge>)>);

impl Edges {
    /// One state's edges by line.
    fn of(edges: Vec<Edge>) -> Edges {
        let mut out = Edges::default();
        for e in edges {
            let slot = out.0.entry(line_of(&e)).or_default();
            slot.0 += 1;
            slot.1 += e.length;
            if slot.2.as_ref().is_none_or(|l| e.length > l.length) {
                slot.2 = Some(e);
            }
        }
        out
    }

    /// With another state's: a line both have keeps the one that shows more of it.
    fn or(&mut self, other: Edges) {
        for (line, found) in other.0 {
            match self.0.get_mut(&line) {
                Some(mine) if mine.1 >= found.1 => {}
                Some(mine) => *mine = found,
                None => {
                    self.0.insert(line, found);
                }
            }
        }
    }

    pub fn count(&self) -> usize {
        self.0.values().map(|v| v.0).sum()
    }

    /// Their length together (m).
    pub fn length(&self) -> f64 {
        self.0.values().fold(0.0, |sum, v| sum + v.1)
    }

    /// The longest edge of each line, the longest first.
    fn longest(&self) -> Vec<&Edge> {
        let mut out: Vec<&Edge> = self.0.values().filter_map(|v| v.2.as_ref()).collect();
        out.sort_by(|a, b| {
            b.length
                .total_cmp(&a.length)
                .then_with(|| a.at.map(f64::to_bits).cmp(&b.at.map(f64::to_bits)))
        });
        out
    }
}

/// The hard edges a house shows (`edges::hard_edges`'s rule) in the states the play camera sees it in:
///
/// - from 15 m out and further, whole, its roof on (`ROOF_MIN_DISTANCE`);
/// - nearer, for each storey in view (those above it hidden, no roof): with its walls standing, which the steep
///   views look down into; and with the walls on the camera's side cut away, each of the eight ways the cutaway
///   stands (`look_bits`) seen from the views that make it stand so.
///
/// An edge on the same line in several states counts once, by the state that shows most of it.
pub fn house_edges(parts: &[Part], surfaces: &Surfaces, index: &Index, storeys: u32) -> Edges {
    let mut out = Edges::of(edges::hard_edges(&drawn(parts, surfaces)));
    for storey in 0..storeys.max(1) {
        out.or(Edges::of(edges::hard_edges(&cut_away(
            parts, surfaces, index, storey, 0,
        ))));
        // (The looks the play camera's views make: one bit square on to a wall, two from a corner.)
        for look in [1, 2, 4, 8, 5, 6, 9, 10] {
            let meshes = cut_away(parts, surfaces, index, storey, look);
            out.or(Edges::of(edges::hard_edges_from(&meshes, &|way| {
                look_bits(way) == look
            })));
        }
    }
    out
}

/// A sample's hard edges: a house's by `house_edges`, anything else's as a model's.
pub fn sample_edges(sample: &Listed, parts: &[Part], surfaces: &Surfaces, index: &Index) -> Edges {
    if sample.kind == "house" {
        house_edges(parts, surfaces, index, sample.storeys.unwrap_or(1))
    } else {
        Edges::of(edges::hard_edges(&drawn(parts, surfaces)))
    }
}

/// Which way an edge runs: upright, level or slanted.
fn runs(e: &Edge) -> &'static str {
    match e.along[1].abs() {
        y if y > 0.985 => "upright",
        y if y < 0.17 => "level",
        _ => "slanted",
    }
}

/// Where an edge is, in the game's own x and z (the dump's z mirrored back), to the centimetre.
fn place(e: &Edge) -> String {
    let cm = |v: f64| format!("{:.2}", if v.abs() < 0.005 { 0.0 } else { v });
    format!(
        "{} ({}) {:.2} m {} at ({}, {}, {})",
        e.part,
        e.material,
        e.length,
        runs(e),
        cm(e.at[0]),
        cm(e.at[1]),
        cm(-e.at[2])
    )
}

/// The gate's line for a sample's hard edges (none: no line).
fn edges_line(edges: &Edges) -> Option<String> {
    let longest = edges.longest();
    let first = longest.first()?;
    let n = edges.count();
    Some(format!(
        "{n} hard edge{} of 5 cm or more show{} unbevelled at the play camera, {:.1} m (`--report` says where; x, height, z): the longest {}",
        if n == 1 { "" } else { "s" },
        if n == 1 { "s" } else { "" },
        edges.length(),
        place(first)
    ))
}

// ---- texels -------------------------------------------------------------------------------------------

/// The textured materials among `surfaces` as the texel rule takes them, each with its texture's size.
fn textures(surfaces: &Surfaces, pictures: &mut Pictures) -> Textures {
    surfaces
        .iter()
        .filter_map(|(k, s)| {
            let file = s.texture.as_ref()?;
            let texture = Texture {
                uv_scale: s.uv_scale,
                px: pictures.side(file),
            };
            Some((k.clone(), texture))
        })
        .collect()
}

/// A sample's parts on textured materials as the texel rule reads them: a part is the rule's material (what the
/// findings name), its material the rule's finish. A material projected from above has x and z for UVs.
fn textured(parts: &[Part], surfaces: &Surfaces) -> Vec<Mesh> {
    parts
        .iter()
        .filter(|p| {
            !p.shadow_only
                && surfaces
                    .get(&p.mesh.material)
                    .is_some_and(|s| s.texture.is_some())
        })
        .map(|p| {
            let planar = surfaces[&p.mesh.material].planar;
            Mesh {
                material: p.mesh.part.clone(),
                uvs: if planar {
                    p.mesh.positions.iter().map(|at| [at[0], at[2]]).collect()
                } else {
                    p.mesh.uvs.clone()
                },
                ..p.mesh.clone()
            }
        })
        .collect()
}

// ---- a far version against its near one ---------------------------------------------------------------

/// A triangle as a camera far off sees it: its corners, its normal and its colour as drawn (linear, 0..1: its
/// vertices' mean times its material's colour and texture).
struct Seen {
    p: [V3; 3],
    n: V3,
    colour: [f64; 3],
}

/// What shows of a sample from far off along `w` (towards the camera): its triangles filed by the cells of a grid
/// across the view, to ask which of them is nearest the camera at a point.
struct View<'a> {
    seen: &'a [Seen],
    /// Each triangle's corners in the view's own axes (across, up, towards the camera); None: not drawn from here.
    flat: Vec<Option<[V3; 3]>>,
    lo: [f64; 2],
    size: [usize; 2],
    cells: Vec<Vec<u32>>,
}

/// The grid's cells (m): the triangles a point may lie in are a cell's.
const CELL: f64 = 0.5;
/// How far apart the views are sampled (m): a tenth of a metre shows a corner board, and a house is ten thousand points.
const STEP: f64 = 0.1;

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl<'a> View<'a> {
    /// The view of `seen` along `w`, `u` across and `v` up the picture (the three at right angles).
    fn new(seen: &'a [Seen], u: V3, v: V3, w: V3) -> View<'a> {
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in seen.iter().flat_map(|t| t.p) {
            for (k, axis) in [u, v].into_iter().enumerate() {
                lo[k] = lo[k].min(dot(p, axis));
                hi[k] = hi[k].max(dot(p, axis));
            }
        }
        if seen.is_empty() {
            (lo, hi) = ([0.0; 2], [0.0; 2]);
        }
        let size = [0, 1].map(|k| ((hi[k] - lo[k]) / CELL).floor() as usize + 1);
        let mut view = View {
            seen,
            // (One seen from behind isn't drawn; one seen edge on covers no point.)
            flat: seen
                .iter()
                .map(|t| {
                    (dot(t.n, w) > 1e-6).then(|| t.p.map(|p| [dot(p, u), dot(p, v), dot(p, w)]))
                })
                .collect(),
            lo,
            size,
            cells: vec![Vec::new(); size[0] * size[1]],
        };
        for (i, q) in view.flat.iter().enumerate() {
            let Some(q) = q else {
                continue;
            };
            let cell = |k: usize, most: bool| {
                let reach = q.iter().map(|p| p[k]);
                let far = if most {
                    reach.fold(f64::NEG_INFINITY, f64::max)
                } else {
                    reach.fold(f64::INFINITY, f64::min)
                };
                (((far - lo[k]) / CELL).floor().max(0.0) as usize).min(size[k] - 1)
            };
            for b in cell(1, false)..=cell(1, true) {
                for a in cell(0, false)..=cell(0, true) {
                    view.cells[b * size[0] + a].push(i as u32);
                }
            }
        }
        view
    }

    /// The triangle nearest the camera at (`a`, `b`) of the picture, and how far along `w` it is there.
    fn first(&self, a: f64, b: f64) -> Option<(&'a Seen, f64)> {
        let (ca, cb) = (
            ((a - self.lo[0]) / CELL).floor(),
            ((b - self.lo[1]) / CELL).floor(),
        );
        if ca < 0.0 || cb < 0.0 || ca as usize >= self.size[0] || cb as usize >= self.size[1] {
            return None;
        }
        let mut best: Option<(&Seen, f64)> = None;
        for &i in &self.cells[cb as usize * self.size[0] + ca as usize] {
            let Some(q) = self.flat[i as usize] else {
                continue;
            };
            let d = (q[1][0] - q[0][0]) * (q[2][1] - q[0][1])
                - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1]);
            if d.abs() < 1e-12 {
                continue;
            }
            let b1 =
                ((a - q[0][0]) * (q[2][1] - q[0][1]) - (q[2][0] - q[0][0]) * (b - q[0][1])) / d;
            let b2 =
                ((q[1][0] - q[0][0]) * (b - q[0][1]) - (a - q[0][0]) * (q[1][1] - q[0][1])) / d;
            if b1 < -1e-9 || b2 < -1e-9 || b1 + b2 > 1.0 + 1e-9 {
                continue;
            }
            let depth = q[0][2] + b1 * (q[1][2] - q[0][2]) + b2 * (q[2][2] - q[0][2]);
            if best.is_none_or(|(_, far)| depth > far) {
                best = Some((&self.seen[i as usize], depth));
            }
        }
        best
    }

    /// Every point of the picture `STEP` apart with what shows there: `each(a, b, triangle, depth)`.
    fn points(&self, mut each: impl FnMut(f64, f64, &'a Seen, f64)) {
        let count = [0, 1].map(|k| (self.size[k] as f64 * CELL / STEP).ceil() as usize);
        for j in 0..count[1] {
            for i in 0..count[0] {
                let (a, b) = (
                    self.lo[0] + (i as f64 + 0.5) * STEP,
                    self.lo[1] + (j as f64 + 0.5) * STEP,
                );
                if let Some((t, depth)) = self.first(a, b) {
                    each(a, b, t, depth);
                }
            }
        }
    }
}

/// Colours by how much of a picture they cover: each (sRGB bytes) with its area, to find the one most of it is.
#[derive(Default)]
struct Palette(BTreeMap<[u8; 3], (f64, [f64; 3])>);

impl Palette {
    /// `area` more of `colour` (linear).
    fn add(&mut self, colour: [f64; 3], area: f64) {
        // (Sixteen steps a channel: a wall's shading by vertex stays one colour, another paint is another.)
        let bin = colour.map(|c| ((srgb(c.clamp(0.0, 1.0)) * 255.0).round() as u8) >> 4);
        let slot = self.0.entry(bin).or_default();
        slot.0 += area;
        for k in 0..3 {
            slot.1[k] += colour[k] * area;
        }
    }

    /// The colour most of it is (sRGB bytes: the mean of its bin and the bins beside it), and its share.
    fn most(&self) -> Option<([u8; 3], f64)> {
        let total: f64 = self.0.values().map(|v| v.0).sum();
        let (bin, _) = self
            .0
            .iter()
            .max_by(|a, b| a.1.0.total_cmp(&b.1.0).then(b.0.cmp(a.0)))?;
        let (mut area, mut sum) = (0.0, [0.0; 3]);
        for (other, (a, s)) in &self.0 {
            if (0..3).all(|k| other[k].abs_diff(bin[k]) <= 1) {
                area += a;
                for k in 0..3 {
                    sum[k] += s[k];
                }
            }
        }
        Some((
            sum.map(|s| (srgb(s / area) * 255.0).round() as u8),
            area / total,
        ))
    }
}

/// A roof's kind, as its faces show it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Roof {
    Flat,
    Gable,
    Hip,
}

/// How a building looks from far off, measured from its meshes: what a far version has to agree with.
#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    /// What its roofs cover in plan: the box round them (x0, z0, x1, z1, m) and their area (m²).
    pub plan: [f64; 4],
    pub area: f64,
    /// The top of its roof (m).
    pub top: f64,
    pub roof: Roof,
    /// Its roof's pitch (degrees): the pitch half its sloped faces are under. 0: flat.
    pub pitch: f64,
    /// The pieces of chimney stacks standing over its roof (a stack's brick, its cap).
    pub chimneys: usize,
    /// The colour most of its walls are and most of its roof is, as drawn (sRGB: None where it has none).
    pub wall: Option<[u8; 3]>,
    pub roof_colour: Option<[u8; 3]>,
}

/// What a roof is at least this high over the ground (m): a porch's landing and a sill look up too.
const ROOFS_FROM: f64 = 1.2;
/// A face looking up less steeply than this (the y of its normal) is a wall's, not a roof's.
const UP: f64 = 0.2;
/// A roof pitched under this (rise over run) is flat.
const LEVEL: f64 = 0.1;
/// A stack is no wider than this (m), and stands this far over the roof's top at least.
const STACK: f64 = 1.6;
const STACK_OVER: f64 = 0.3;
/// Gable ends are at least this share of a gable roof's plan (a porch's little gable is under it).
const GABLES: f64 = 0.04;

/// `parts`' look (`Look`): the roofs from above, the walls from the four sides.
pub fn look(parts: &[Part], surfaces: &Surfaces, pictures: &mut Pictures) -> Look {
    let mut seen = Vec::new();
    for part in parts {
        let material = &part.mesh.material;
        if part.shadow_only || !opaque(material, surfaces) || glass(material) {
            continue;
        }
        let surface = surfaces.get(material);
        let tint = surface.map_or([1.0; 3], |s| s.color.map(linear));
        let picture = surface
            .and_then(|s| s.texture.as_ref())
            .and_then(|file| pictures.mean(file))
            .unwrap_or([1.0; 3]);
        for t in &part.mesh.triangles {
            let p = t.map(|i| part.mesh.positions[i as usize]);
            let (e1, e2) = (
                [0, 1, 2].map(|k| p[1][k] - p[0][k]),
                [0, 1, 2].map(|k| p[2][k] - p[0][k]),
            );
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = dot(n, n).sqrt();
            if l < 1e-12 {
                continue;
            }
            let mut colour = [0.0; 3];
            for &i in t {
                let c = part.colors.get(i as usize).copied().unwrap_or([255; 4]);
                for k in 0..3 {
                    colour[k] += linear(c[k] as f64 / 255.0) / 3.0;
                }
            }
            seen.push(Seen {
                p,
                n: n.map(|c| c / l),
                colour: [0, 1, 2].map(|k| colour[k] * tint[k] * picture[k]),
            });
        }
    }
    // The roofs: what looks up at a camera overhead, over the ground storey's middle.
    let above = View::new(&seen, [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let mut roof = Palette::default();
    let mut cells: Vec<(f64, f64, f64, &Seen)> = Vec::new();
    above.points(|x, z, t, y| {
        if t.n[1] > UP && y > ROOFS_FROM {
            roof.add(t.colour, STEP * STEP);
            cells.push((x, z, y, t));
        }
    });
    let roof_colour = roof.most().map(|(c, _)| c);
    let like = |a: [u8; 3], colour: [f64; 3]| {
        (0..3).all(|k| {
            a[k].abs_diff((srgb(colour[k].clamp(0.0, 1.0)) * 255.0).round() as u8) <= COLOUR
        })
    };
    let mut plan = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    let (mut top, mut level, mut pitches) = (0.0_f64, 0.0, Vec::new());
    for &(x, z, y, t) in &cells {
        plan = [
            plan[0].min(x),
            plan[1].min(z),
            plan[2].max(x),
            plan[3].max(z),
        ];
        // (The roof's own top: a stack's cap is another colour.)
        if roof_colour.is_some_and(|c| like(c, t.colour)) {
            top = top.max(y);
        }
        let run = (t.n[0] * t.n[0] + t.n[2] * t.n[2]).sqrt();
        if run < LEVEL * t.n[1] {
            level += 1.0;
        } else {
            pitches.push((run.atan2(t.n[1]).to_degrees(), 1.0));
        }
    }
    let area = cells.len() as f64 * STEP * STEP;
    if cells.is_empty() {
        plan = [0.0; 4];
    }
    // Gable ends: upright faces with a side that slants and is their own (no face goes on past it in their plane).
    let mut sides: HashMap<([i64; 3], [i64; 3]), Vec<usize>> = HashMap::new();
    let mm = |p: V3| p.map(|c| (c * 1e3).round() as i64);
    for (i, t) in seen.iter().enumerate().filter(|(_, t)| t.n[1].abs() < 0.05) {
        for k in 0..3 {
            let (a, b) = (mm(t.p[k]), mm(t.p[(k + 1) % 3]));
            sides.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut gables = 0.0;
    for (i, t) in seen.iter().enumerate().filter(|(_, t)| t.n[1].abs() < 0.05) {
        let own_slant = (0..3).any(|k| {
            let (a, b) = (t.p[k], t.p[(k + 1) % 3]);
            let d = [0, 1, 2].map(|c| b[c] - a[c]);
            let l = dot(d, d).sqrt().max(1e-12);
            let rise = d[1].abs() / l;
            (0.15..0.98).contains(&rise)
                && !sides[&(mm(a).min(mm(b)), mm(a).max(mm(b)))]
                    .iter()
                    .any(|&j| j != i && dot(seen[j].n, t.n) > 0.999)
        });
        if own_slant {
            let (e1, e2) = (
                [0, 1, 2].map(|k| t.p[1][k] - t.p[0][k]),
                [0, 1, 2].map(|k| t.p[2][k] - t.p[0][k]),
            );
            let c = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            gables += dot(c, c).sqrt() / 2.0;
        }
    }
    let kind = if cells.is_empty() || level >= 0.9 * cells.len() as f64 {
        Roof::Flat
    } else if gables >= GABLES * area {
        Roof::Gable
    } else {
        Roof::Hip
    };
    // Stacks: small pieces (what a part's triangles join up) standing over the roof's top.
    let mut chimneys = 0;
    for part in parts
        .iter()
        .filter(|p| !p.shadow_only && !glass(&p.mesh.material))
    {
        let at = &part.mesh.positions;
        if cells.is_empty() || !at.iter().any(|p| p[1] > top + STACK_OVER) {
            continue;
        }
        let mut places: HashMap<[i64; 3], u32> = HashMap::new();
        let place: Vec<u32> = at
            .iter()
            .map(|p| {
                let n = places.len() as u32;
                *places.entry(mm(*p)).or_insert(n)
            })
            .collect();
        let mut parent: Vec<u32> = (0..places.len() as u32).collect();
        fn root(parent: &mut [u32], mut i: u32) -> u32 {
            while parent[i as usize] != i {
                parent[i as usize] = parent[parent[i as usize] as usize];
                i = parent[i as usize];
            }
            i
        }
        for t in &part.mesh.triangles {
            let a = root(&mut parent, place[t[0] as usize]);
            for &v in &t[1..] {
                let b = root(&mut parent, place[v as usize]);
                parent[b as usize] = a;
            }
        }
        let mut boxes: BTreeMap<u32, [f64; 5]> = BTreeMap::new();
        for (i, p) in at.iter().enumerate() {
            let b = boxes.entry(root(&mut parent, place[i])).or_insert([
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ]);
            *b = [
                b[0].min(p[0]),
                b[1].min(p[2]),
                b[2].max(p[0]),
                b[3].max(p[2]),
                b[4].max(p[1]),
            ];
        }
        chimneys += boxes
            .values()
            .filter(|b| b[4] > top + STACK_OVER && b[2] - b[0] <= STACK && b[3] - b[1] <= STACK)
            .filter(|b| inside(plan, [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0]))
            .count();
    }
    // The walls: what stands upright before a camera level with it on each of its four sides, under the roof's top.
    let mut wall = Palette::default();
    for (u, w) in [
        ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
    ] {
        for sign in [1.0, -1.0] {
            let view = View::new(&seen, u, [0.0, 1.0, 0.0], w.map(|c: f64| c * sign));
            view.points(|_, y, t, _| {
                if t.n[1].abs() <= UP && y > 0.4 && y < top.max(ROOFS_FROM) {
                    wall.add(t.colour, STEP * STEP);
                }
            });
        }
    }
    Look {
        plan,
        area,
        top,
        roof: kind,
        pitch: if kind == Roof::Flat {
            0.0
        } else {
            texels::median(&mut pitches, 0.5).unwrap_or(0.0)
        },
        chimneys,
        wall: wall.most().map(|(c, _)| c),
        roof_colour,
    }
}

fn inside(plan: [f64; 4], at: [f64; 2]) -> bool {
    at[0] >= plan[0] && at[0] <= plan[2] && at[1] >= plan[1] && at[1] <= plan[3]
}

/// How far a far version may be off its near one: a side of its plan (m; a porch stands out less), its plan's
/// area (a share: the eaves' overhang differs by less), its top (m), its pitch (degrees), a colour's channel (of
/// 255: the same paint under a texture and in the plain finish).
const PLAN: f64 = 1.5;
const AREA: f64 = 0.1;
const TOP: f64 = 0.5;
const PITCH: f64 = 4.0;
const COLOUR: u8 = 28;

fn hex_of(c: Option<[u8; 3]>) -> String {
    c.map_or("none".into(), |c| {
        format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
    })
}

fn roof_name(r: Roof) -> &'static str {
    match r {
        Roof::Flat => "flat",
        Roof::Gable => "gable",
        Roof::Hip => "hip",
    }
}

/// Where a far version's look disagrees with its near one's: a line each.
pub fn disagreements(near: &Look, far: &Look) -> Vec<String> {
    let mut out = Vec::new();
    let side = (0..4).fold(0.0_f64, |most, k| {
        most.max((near.plan[k] - far.plan[k]).abs())
    });
    let share = (far.area - near.area).abs() / near.area.max(1e-9);
    if side > PLAN || share > AREA {
        out.push(format!(
            "plan: its roofs cover {:.0} m² ({:.1} m by {:.1} m), near {:.0} m² ({:.1} m by {:.1} m): {:.0} % and {:.1} m off (at most {:.0} % and {PLAN} m)",
            far.area,
            far.plan[2] - far.plan[0],
            far.plan[3] - far.plan[1],
            near.area,
            near.plan[2] - near.plan[0],
            near.plan[3] - near.plan[1],
            share * 100.0,
            side,
            AREA * 100.0
        ));
    }
    if (near.top - far.top).abs() > TOP {
        out.push(format!(
            "height: its roof's top is at {:.2} m, near at {:.2} m (at most {TOP} m apart)",
            far.top, near.top
        ));
    }
    if near.roof != far.roof {
        out.push(format!(
            "roof: a {} roof, near a {} roof",
            roof_name(far.roof),
            roof_name(near.roof)
        ));
    } else if (near.pitch - far.pitch).abs() > PITCH {
        out.push(format!(
            "roof: pitched {:.0}°, near {:.0}° (at most {PITCH}° apart)",
            far.pitch, near.pitch
        ));
    }
    if (near.chimneys > 0) != (far.chimneys > 0) {
        out.push(format!(
            "chimney: {}, near {}",
            if far.chimneys > 0 {
                "it has one"
            } else {
                "none"
            },
            if near.chimneys > 0 { "has one" } else { "none" }
        ));
    }
    let apart = |a: Option<[u8; 3]>, b: Option<[u8; 3]>| match (a, b) {
        (Some(a), Some(b)) => (0..3).any(|k| a[k].abs_diff(b[k]) > COLOUR),
        (a, b) => a.is_some() != b.is_some(),
    };
    if apart(near.wall, far.wall) {
        out.push(format!(
            "walls: most of them are {} as drawn, near {} (a channel at most {COLOUR} of 255 apart)",
            hex_of(far.wall),
            hex_of(near.wall)
        ));
    }
    if apart(near.roof_colour, far.roof_colour) {
        out.push(format!(
            "roof colour: most of it is {} as drawn, near {} (a channel at most {COLOUR} of 255 apart)",
            hex_of(far.roof_colour),
            hex_of(near.roof_colour)
        ));
    }
    out
}

// ---- what a house has of the detail a real one has ------------------------------------------------------

/// What a house's meshes have of the detail the eye misses at the play camera: measured, to say what is there
/// before any is added.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Detail {
    /// The walls' trim (`trim`: posts, corner boards, frames, sills, glazing bars, the frieze): its area (m²), and how
    /// much of it is neither plumb nor level (a bevel's or a chamfer's face, a moulding's or a cornice's profile).
    pub trim: f64,
    pub trim_shaped: f64,
    /// Convex edges of 5 cm or more that turn 15° to 59° (an eased arris, a chamfer's two): upright ones (a diagonal
    /// wall's corners are such) and the others, in metres. Not the roofs'.
    pub eased_upright: f64,
    pub eased: f64,
    /// Each storey's roof: its ridges and its hips (m, where two slopes meet level and slanting), the cap shingles
    /// over them (m²: the roof's faces in another tone than its slopes'), and how far its slopes reach under its
    /// eaves (m, under the foot of its fascias: none on a roof that stands as one is built).
    pub roofs: BTreeMap<u32, [f64; 4]>,
    /// How much of the roofs' trim looks up (m²: a gutter's trough would).
    pub troughs: f64,
}

/// `parts`' detail (`Detail`).
pub fn detail(parts: &[Part], surfaces: &Surfaces) -> Detail {
    let mut out = Detail::default();
    let mm = |p: V3| p.map(|c| (c * 1e4).round() as i64);
    let normal = |p: [V3; 3]| -> Option<(V3, f64)> {
        let (e1, e2) = (
            [0, 1, 2].map(|k| p[1][k] - p[0][k]),
            [0, 1, 2].map(|k| p[2][k] - p[0][k]),
        );
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let l = dot(n, n).sqrt();
        (l > 1e-12).then(|| (n.map(|c| c / l), l / 2.0))
    };
    // Each welded edge's faces as (normal, the face's third corner): the walls' and what stands on them, and each
    // storey's roof slopes.
    type Sides = HashMap<([i64; 3], [i64; 3]), Vec<(V3, V3)>>;
    let mut walls = Sides::new();
    let mut roofs: BTreeMap<u32, Sides> = BTreeMap::new();
    // Each storey's roof: the foot of its fascias (the roofs' upright trim) and the lowest its shingles reach.
    let mut eaves: BTreeMap<u32, [f64; 2]> = BTreeMap::new();
    for part in parts.iter().filter(|p| !p.shadow_only) {
        let material = &part.mesh.material;
        if !part.mesh.edged || !opaque(material, surfaces) {
            continue;
        }
        let shingles = material == "material.roof";
        // The roof's own tone: the vertex colour most of its area has.
        let mut tones: BTreeMap<[u8; 4], f64> = BTreeMap::new();
        let faces: Vec<([V3; 3], V3, f64, [u8; 4])> = part
            .mesh
            .triangles
            .iter()
            .filter_map(|t| {
                let p = t.map(|i| part.mesh.positions[i as usize]);
                let (n, area) = normal(p)?;
                Some((
                    p,
                    n,
                    area,
                    part.colors.get(t[0] as usize).copied().unwrap_or([255; 4]),
                ))
            })
            .collect();
        for f in &faces {
            *tones.entry(f.3).or_default() += f.2;
        }
        let tone = tones
            .iter()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(c, _)| *c);
        for (p, n, area, colour) in faces {
            let own = Some(colour) == tone;
            if part.mesh.part == "trim" {
                out.trim += area;
                if n[1].abs() > 0.02 && n[1].abs() < 0.98 {
                    out.trim_shaped += area;
                }
            }
            if shingles && part.roof && !own {
                out.roofs.entry(part.storey).or_default()[2] += area;
            }
            if part.roof && !shingles && n[1] > 0.5 {
                out.troughs += area;
            }
            if part.roof && (shingles || n[1].abs() < 0.05) {
                let low = p.iter().fold(f64::INFINITY, |low, at| low.min(at[1]));
                let slot = &mut eaves.entry(part.storey).or_insert([f64::INFINITY; 2])
                    [usize::from(shingles)];
                *slot = slot.min(low);
            }
            // (A roof's edges are its slopes' in its own tone; the soffits and fascias under it are no wall's.)
            let sides = match (shingles, part.roof) {
                (true, true) if own && n[1] > UP => roofs.entry(part.storey).or_default(),
                (false, false) => &mut walls,
                _ => continue,
            };
            for k in 0..3 {
                let (a, b) = (mm(p[k]), mm(p[(k + 1) % 3]));
                sides
                    .entry((a.min(b), a.max(b)))
                    .or_default()
                    .push((n, p[(k + 2) % 3]));
            }
        }
    }
    let ends = |key: &([i64; 3], [i64; 3])| -> (V3, V3) {
        (key.0.map(|c| c as f64 / 1e4), key.1.map(|c| c as f64 / 1e4))
    };
    // (Convex: one face's far corner lies behind the other.)
    let convex = |a: V3, faces: &[(V3, V3)]| {
        dot(faces[0].0, [0, 1, 2].map(|k| faces[1].1[k] - a[k])) < -1e-6
    };
    for (key, faces) in walls.iter().filter(|(_, f)| f.len() == 2) {
        let (a, b) = ends(key);
        let d = [0, 1, 2].map(|k| b[k] - a[k]);
        let l = dot(d, d).sqrt();
        let turn = dot(faces[0].0, faces[1].0)
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees();
        if l >= edges::LENGTH && (15.0..60.0).contains(&turn) && convex(a, faces) {
            if d[1].abs() / l > 0.985 {
                out.eased_upright += l;
            } else {
                out.eased += l;
            }
        }
    }
    for (storey, sides) in &roofs {
        for (key, faces) in sides.iter().filter(|(_, f)| f.len() == 2) {
            let (a, b) = ends(key);
            let d = [0, 1, 2].map(|k| b[k] - a[k]);
            let l = dot(d, d).sqrt();
            if l < 1e-6 || dot(faces[0].0, faces[1].0) > 0.999 || !convex(a, faces) {
                continue;
            }
            out.roofs.entry(*storey).or_default()[usize::from(d[1].abs() / l >= 0.02)] += l;
        }
    }
    for (storey, [fascia, shingles]) in eaves {
        if fascia.is_finite() && shingles.is_finite() {
            out.roofs.entry(storey).or_default()[3] = (fascia - shingles).max(0.0);
        }
    }
    out
}

// ---- the gate -----------------------------------------------------------------------------------------

/// A sample as the gate measured it.
#[derive(Clone, Debug, Default)]
pub struct Measured {
    pub sample: Listed,
    /// What a camera draws of it, and what only the shadow map does.
    pub triangles: usize,
    pub draws: usize,
    pub shadow_triangles: usize,
    pub shadow_draws: usize,
    /// Its hard edges and its textured parts' texels (None: a far version, not judged for them).
    pub edges: Option<Edges>,
    pub materials: Option<Vec<texels::Material>>,
    /// Its look, where it is one of a pair, and where a far version's disagrees with its near one's.
    pub look: Option<Look>,
    pub far_near: Option<Vec<String>>,
    /// A house's detail.
    pub detail: Option<Detail>,
}

/// A gate run: what it prints, how many samples failed, what it counted and measured.
pub struct Report {
    pub text: String,
    pub failed: usize,
    pub counts: Counts,
    pub samples: Vec<Measured>,
    pub textures: Textures,
    /// The looks of the samples that have far versions, by key.
    pub near: HashMap<String, Look>,
}

fn fail(text: String) -> Report {
    Report {
        text,
        failed: 1,
        counts: Counts::new(),
        samples: Vec::new(),
        textures: Textures::new(),
        near: HashMap::new(),
    }
}

/// `dir`'s `built.json`.
pub fn index(dir: &str) -> Result<Index, String> {
    let path = join(dir, "built.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("{path}: {e} (`bun tools/built/dump.ts {dir}` writes it)"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))
}

/// The gate over the samples dumped in `dir` (`keys`: only these), judged by the baseline at `paths`.
pub fn gate(paths: &Paths, dir: &str, keys: &[String]) -> Report {
    let read = || -> Result<_, String> {
        Ok((
            index(dir)?,
            surfaces(paths)?,
            Baseline::read(&paths.baseline)?,
        ))
    };
    let (index, surfaces, baseline) = match read() {
        Ok(r) => r,
        Err(err) => {
            return fail(format!(
                "FAIL the built world\n       error: cannot read it: {err}\n"
            ));
        }
    };
    if let Some(key) = keys
        .iter()
        .find(|k| !index.samples.iter().any(|s| &s.key == *k))
    {
        return fail(format!(
            "FAIL {key}\n       error: built.json lists no such sample\n"
        ));
    }
    let mut pictures = Pictures::default();
    let textures = textures(&surfaces, &mut pictures);
    let parts_of = |key: &str| -> Result<Vec<Part>, String> {
        let path = join(dir, &format!("{key}.glb"));
        let data = std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
        read_parts(&data).map_err(|e| format!("{path}: {e}"))
    };
    let mut text = String::new();
    let mut failed = 0;
    let mut counts = Counts::new();
    let mut samples = Vec::new();
    // The near samples' looks, each measured once for all its far versions.
    let mut looks: HashMap<String, Look> = HashMap::new();
    for sample in index
        .samples
        .iter()
        .filter(|s| keys.is_empty() || keys.contains(&s.key))
    {
        let measure = |pictures: &mut Pictures,
                       looks: &mut HashMap<String, Look>|
         -> Result<Measured, String> {
            let parts = parts_of(&sample.key)?;
            let mut m = Measured {
                sample: sample.clone(),
                ..Measured::default()
            };
            for p in parts.iter().filter(|p| !p.mesh.triangles.is_empty()) {
                let (triangles, draws) = if p.shadow_only {
                    (&mut m.shadow_triangles, &mut m.shadow_draws)
                } else {
                    (&mut m.triangles, &mut m.draws)
                };
                *triangles += p.mesh.triangles.len();
                *draws += 1;
            }
            if !DISTANT.contains(&sample.kind.as_str()) {
                m.edges = Some(sample_edges(sample, &parts, &surfaces, &index));
                m.materials = Some(texels::measure(&textured(&parts, &surfaces), &textures));
            }
            if sample.kind == "house" {
                m.detail = Some(detail(&parts, &surfaces));
            }
            if FAR.contains(&sample.kind.as_str()) {
                let of = sample
                    .of
                    .as_ref()
                    .ok_or("a far version, but built.json doesn't say of what (`of`)")?;
                if !looks.contains_key(of) {
                    let near = look(&parts_of(of)?, &surfaces, pictures);
                    looks.insert(of.clone(), near);
                }
                let far = look(&parts, &surfaces, pictures);
                m.far_near = Some(disagreements(&looks[of], &far));
                m.look = Some(far);
            }
            Ok(m)
        };
        let m = match measure(&mut pictures, &mut looks) {
            Ok(m) => m,
            Err(err) => {
                failed += 1;
                text += &format!("FAIL {}\n       error: {err}\n", sample.key);
                continue;
            }
        };
        let mut found: Vec<(&str, Found)> = Vec::new();
        if let Some(edges) = &m.edges {
            found.push((
                EDGES,
                Found {
                    count: edges.count() as u64,
                    text: edges_line(edges).into_iter().collect(),
                },
            ));
        }
        if let Some(materials) = &m.materials {
            let lines: Vec<String> = materials.iter().flat_map(texels::findings).collect();
            found.push((
                TEXELS,
                Found {
                    count: lines.len() as u64,
                    text: lines,
                },
            ));
        }
        if let Some(lines) = &m.far_near {
            found.push((
                FAR_NEAR,
                Found {
                    count: lines.len() as u64,
                    text: lines.clone(),
                },
            ));
        }
        let (mut errors, mut warnings) = (Vec::new(), Vec::new());
        for (check, found) in found {
            counts
                .entry(check.into())
                .or_default()
                .insert(sample.key.clone(), found.count);
            let sorted = baseline::sort(
                check,
                baseline.judge(check, &sample.key, found.count),
                found,
            );
            errors.extend(sorted.strict);
            errors.extend(sorted.errors);
            warnings.extend(sorted.warnings);
        }
        failed += usize::from(!errors.is_empty());
        text += &format!(
            "{} {:28} {} triangles, {} draws{}\n",
            if errors.is_empty() { "ok  " } else { "FAIL" },
            sample.key,
            m.triangles,
            m.draws,
            if m.shadow_draws > 0 {
                format!(
                    " (and {} triangles in {} only the shadow map draws)",
                    m.shadow_triangles, m.shadow_draws
                )
            } else {
                String::new()
            }
        );
        // (The baseline's own lines name the models' command: this gate's lowers these sections.)
        let own = |line: String| {
            line.replace(
                "`cargo check-glb --update-baseline`",
                "`cargo check-glb --built <dir> --update-baseline`",
            )
        };
        for e in errors {
            text += &format!("       error: {}\n", own(e));
        }
        for w in warnings {
            text += &format!("       warning: {}\n", own(w));
        }
        samples.push(m);
    }
    // The baseline's own keys: one that is no sample a check ran on is stale (counted as none found).
    let mut stale = Vec::new();
    if keys.is_empty() {
        for check in CHECKS {
            let ran = counts.entry(check.into()).or_default();
            for key in baseline
                .keys(check)
                .filter(|k| !ran.contains_key(*k))
                .collect::<Vec<_>>()
            {
                stale.push(format!(
                    "{check}: {key} is no sample this check ran on: take it out (`cargo check-glb --built <dir> --update-baseline`)"
                ));
                ran.insert(key.into(), 0);
            }
        }
    }
    if !stale.is_empty() {
        failed += 1;
        text += "FAIL gate-baseline.json\n";
        for e in stale {
            text += &format!("       error: {e}\n");
        }
    }
    text += &format!("{} samples, {failed} failed\n", samples.len());
    Report {
        text,
        failed,
        counts,
        samples,
        textures,
        near: looks,
    }
}

/// A sample's file as its parts.
fn read_parts(data: &[u8]) -> R<Vec<Part>> {
    read(&parse_glb(data)?)
}

/// `baseline` with the sections of `CHECKS` it has none of, from what a run counted (the day a check comes in):
/// a section it has stays as it is (only `--update-baseline` changes it, and only down).
pub fn seeded(baseline: &Baseline, counts: &Counts) -> Result<(Baseline, Vec<String>), String> {
    let mut all: serde_json::Value =
        serde_json::from_str(&baseline.text()).map_err(|e| e.to_string())?;
    let mut written = Vec::new();
    for check in CHECKS {
        if baseline.checks().any(|c| c == check) {
            continue;
        }
        let section: serde_json::Map<String, serde_json::Value> = counts
            .get(check)
            .into_iter()
            .flatten()
            .filter(|(_, n)| **n > 0)
            .map(|(key, n)| (key.clone(), serde_json::Value::from(*n)))
            .collect();
        if section.is_empty() {
            continue;
        }
        written.push(format!(
            "{check}: {} samples, {} findings",
            section.len(),
            section
                .values()
                .filter_map(serde_json::Value::as_u64)
                .sum::<u64>()
        ));
        if let Some(all) = all.as_object_mut() {
            all.insert(check.into(), serde_json::Value::Object(section));
        }
    }
    Ok((Baseline::parse(&all.to_string())?, written))
}

// ---- the report ---------------------------------------------------------------------------------------

/// What a run measured, as tables (`--report`): each sample's counts, where the hard edges are, how each material's
/// texture is laid, every far version's look beside its near one's, and the houses' detail.
pub fn report(r: &Report) -> String {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "## Samples\n\n| sample | triangles | draws | hard edges | metres | texel findings | far against near |\n|---|---|---|---|---|---|---|"
    );
    let dash = || "—".to_string();
    for m in &r.samples {
        let _ = writeln!(
            text,
            "| {} | {} | {} | {} | {} | {} | {} |",
            m.sample.key,
            m.triangles,
            m.draws,
            m.edges.as_ref().map_or(dash(), |e| e.count().to_string()),
            m.edges
                .as_ref()
                .map_or(dash(), |e| format!("{:.1}", e.length())),
            m.materials.as_ref().map_or(dash(), |ms| ms
                .iter()
                .map(|x| texels::findings(x).len())
                .sum::<usize>()
                .to_string()),
            m.far_near.as_ref().map_or(dash(), |f| f.len().to_string()),
        );
    }
    // By kind: the most a sample of it has.
    let mut kinds: BTreeMap<&str, (usize, usize, usize, usize)> = BTreeMap::new();
    for m in &r.samples {
        let k = kinds.entry(&m.sample.kind).or_default();
        *k = (
            k.0 + 1,
            k.1.max(m.triangles),
            k.2.max(m.draws),
            k.3 + m.triangles,
        );
    }
    let _ = writeln!(
        text,
        "\n| kind | samples | triangles (mean) | triangles (most) | draws (most) |\n|---|---|---|---|---|"
    );
    for (kind, (n, most, draws, sum)) in &kinds {
        let _ = writeln!(
            text,
            "| {kind} | {n} | {} | {most} | {draws} |",
            sum / n.max(&1)
        );
    }

    let _ = writeln!(
        text,
        "\n## Hard edges, by where\n\n| kind | part | runs | edges | metres | the longest |\n|---|---|---|---|---|---|"
    );
    // By kind, part and the way it runs: how many edges, how long together, the longest.
    type Found<'a> = (usize, f64, Option<&'a Edge>);
    let mut places: BTreeMap<(&str, &str, &str), Found> = BTreeMap::new();
    for m in &r.samples {
        for (count, length, longest) in m.edges.iter().flat_map(|e| e.0.values()) {
            let Some(e) = longest.as_ref() else {
                continue;
            };
            let slot = places
                .entry((&m.sample.kind, &e.part, runs(e)))
                .or_default();
            slot.0 += count;
            slot.1 += length;
            if slot.2.is_none_or(|l| e.length > l.length) {
                slot.2 = Some(e);
            }
        }
    }
    let mut rows: Vec<_> = places.iter().collect();
    rows.sort_by(|a, b| a.0.0.cmp(b.0.0).then(b.1.1.total_cmp(&a.1.1)));
    for ((kind, part, way), (count, length, longest)) in rows {
        let _ = writeln!(
            text,
            "| {kind} | {part} | {way} | {count} | {length:.1} | {} |",
            longest.map_or(dash(), place)
        );
    }

    let _ = writeln!(
        text,
        "\n## Texels, by material\n\n| material | texture px | uvScale | parts | m² | px a metre (median) | 5th to 95th | stretched | smeared | findings |\n|---|---|---|---|---|---|---|---|---|---|"
    );
    for (material, texture) in &r.textures {
        let laid: Vec<&texels::Material> = r
            .samples
            .iter()
            .flat_map(|m| m.materials.iter().flatten())
            .filter(|m| &m.finish == material)
            .collect();
        if laid.is_empty() {
            continue;
        }
        let area: f64 = laid.iter().map(|m| m.area).sum();
        let mut spread: Vec<(f64, f64)> = laid.iter().flat_map(|m| m.spread.clone()).collect();
        let px = |at: f64, spread: &mut [(f64, f64)]| match (texture.px, texels::median(spread, at))
        {
            (Some(px), Some(v)) => format!("{:.0}", v * texture.uv_scale * px),
            _ => dash(),
        };
        let share = |of: &dyn Fn(&texels::Material) -> f64| {
            format!(
                "{:.1} %",
                laid.iter().map(|m| of(m) * m.area).sum::<f64>() / area.max(1e-9) * 100.0
            )
        };
        let _ = writeln!(
            text,
            "| {material} | {} | {} | {} | {area:.0} | {} | {} to {} | {} | {} | {} |",
            texture.px.map_or("unread".into(), |px| format!("{px:.0}")),
            texture.uv_scale,
            laid.len(),
            px(0.5, &mut spread),
            px(0.05, &mut spread),
            px(0.95, &mut spread),
            share(&|m| m.stretched),
            share(&|m| m.smeared),
            laid.iter()
                .map(|m| texels::findings(m).len())
                .sum::<usize>(),
        );
    }

    let _ = writeln!(
        text,
        "\n## Far against near\n\n| sample | plan m² | plan box m | top m | roof | chimney | walls | roof colour | disagreements |\n|---|---|---|---|---|---|---|---|---|"
    );
    let row = |key: &str, l: &Look, found: Option<&Vec<String>>| {
        format!(
            "| {key} | {:.0} | {:.1} × {:.1} | {:.2} | {}{} | {} | {} | {} | {} |",
            l.area,
            l.plan[2] - l.plan[0],
            l.plan[3] - l.plan[1],
            l.top,
            roof_name(l.roof),
            if l.roof == Roof::Flat {
                String::new()
            } else {
                format!(" {:.0}°", l.pitch)
            },
            if l.chimneys > 0 { "yes" } else { "no" },
            hex_of(l.wall),
            hex_of(l.roof_colour),
            found.map_or(dash(), |f| {
                if f.is_empty() {
                    "none".into()
                } else {
                    f.iter()
                        .map(|line| line.split(':').next().unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            })
        )
    };
    let mut shown: Vec<&str> = Vec::new();
    for m in &r.samples {
        let (Some(look), Some(of)) = (&m.look, &m.sample.of) else {
            continue;
        };
        if let Some(near) = r.near.get(of).filter(|_| !shown.contains(&of.as_str())) {
            shown.push(of);
            let _ = writeln!(text, "{}", row(of, near, None));
        }
        let _ = writeln!(text, "{}", row(&m.sample.key, look, m.far_near.as_ref()));
    }

    let _ = writeln!(
        text,
        "\n## The houses' detail\n\n| house | trim m² | of it neither plumb nor level | eased edges, upright m | eased edges, others m | roof trim looking up m² | each storey's roof: ridges m, hips m, cap shingles m², slopes under its eaves m |\n|---|---|---|---|---|---|---|"
    );
    for m in &r.samples {
        if let Some(d) = &m.detail {
            let roofs: Vec<String> = d
                .roofs
                .iter()
                .map(|(storey, [ridges, hips, caps, sunk])| {
                    format!("storey {storey}: {ridges:.1}, {hips:.1}, {caps:.2}, {sunk:.2}")
                })
                .collect();
            let _ = writeln!(
                text,
                "| {} | {:.1} | {:.2} | {:.1} | {:.1} | {:.2} | {} |",
                m.sample.key,
                d.trim,
                d.trim_shaped,
                d.eased_upright,
                d.eased,
                d.troughs,
                roofs.join("; ")
            );
        }
    }
    text
}

const USAGE: &str = "usage: cargo check-glb --built <dir> [sample keys…] [--report] [--update-baseline] [--seed-baseline] (the dir `bun tools/built/dump.ts <dir>` wrote)";

/// `cargo check-glb --built <dir> …`: the exit code.
pub fn main(args: &[String]) -> i32 {
    let dir = args
        .iter()
        .position(|a| a == "--built")
        .and_then(|i| args.get(i + 1))
        .filter(|d| !d.starts_with("--"));
    let Some(dir) = dir else {
        eprintln!("{USAGE}");
        return 2;
    };
    let keys: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--") && *a != dir)
        .cloned()
        .collect();
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let paths = Paths::repo();
    // A change to the baseline from what a run counted: the baseline after it, a line for each change.
    type Change = dyn Fn(&Baseline, &Counts) -> Result<(Baseline, Vec<String>), String>;
    let rewrite = |what: &str, change: &Change| -> Result<(), String> {
        let run = gate(&paths, dir, &keys);
        let (next, changes) = change(&Baseline::read(&paths.baseline)?, &run.counts)?;
        if changes.is_empty() {
            println!("{}: nothing to {what}", baseline::FILE);
        } else {
            next.write(&paths.baseline)?;
            for c in changes {
                println!("{}: {c}", baseline::FILE);
            }
        }
        Ok(())
    };
    let mut done = Ok(());
    if has("--seed-baseline") {
        done = rewrite("seed", &seeded);
    }
    if done.is_ok() && has("--update-baseline") {
        done = rewrite("lower", &|baseline, counts| Ok(baseline.lowered(counts)));
    }
    if let Err(err) = done {
        eprintln!("cannot write {}: {err}", baseline::FILE);
        return 1;
    }
    let run = gate(&paths, dir, &keys);
    print!("{}", run.text);
    if has("--report") {
        print!("\n{}", report(&run));
    }
    i32::from(run.failed > 0)
}

#[cfg(test)]
mod tests {
    use super::super::glb::tests::glb;
    use super::*;

    /// A part named `name` on `material`, nothing in it yet.
    fn part(name: &str, material: &str) -> Part {
        Part {
            mesh: Mesh {
                part: name.into(),
                material: material.into(),
                finish: material.into(),
                edged: edged(material),
                opaque: true,
                ..Mesh::default()
            },
            ..Part::default()
        }
    }

    /// A flat face of `corners` (three or four, round its edge) in `colour`, wound to look `out`.
    fn face(part: &mut Part, corners: &[V3], out: V3, colour: [u8; 3]) {
        let (a, b, c) = (corners[0], corners[1], corners[2]);
        let (e1, e2) = (
            [0, 1, 2].map(|k| b[k] - a[k]),
            [0, 1, 2].map(|k| c[k] - a[k]),
        );
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let turned = dot(n, out) < 0.0;
        let l = dot(n, n).sqrt() * if turned { -1.0 } else { 1.0 };
        let first = part.mesh.positions.len() as u32;
        let order: Vec<V3> = if turned {
            corners.iter().rev().copied().collect()
        } else {
            corners.to_vec()
        };
        for p in &order {
            part.mesh.positions.push(*p);
            part.mesh.normals.push(n.map(|c| c / l));
            part.mesh.uvs.push([p[0] + p[2], p[1]]);
            part.colors.push([colour[0], colour[1], colour[2], 255]);
        }
        for k in 1..order.len() as u32 - 1 {
            part.mesh.triangles.push([first, first + k, first + k + 1]);
        }
    }

    /// A box from `lo` to `hi`, every face but those `skip` names (`py`: its top, `ny`: its underside).
    fn boxed(part: &mut Part, lo: V3, hi: V3, colour: [u8; 3], skip: &str) {
        let at = |x: bool, y: bool, z: bool| {
            [
                if x { hi[0] } else { lo[0] },
                if y { hi[1] } else { lo[1] },
                if z { hi[2] } else { lo[2] },
            ]
        };
        for (name, out, corners) in [
            (
                "py",
                [0.0, 1.0, 0.0],
                [
                    at(false, true, false),
                    at(true, true, false),
                    at(true, true, true),
                    at(false, true, true),
                ],
            ),
            (
                "ny",
                [0.0, -1.0, 0.0],
                [
                    at(false, false, false),
                    at(true, false, false),
                    at(true, false, true),
                    at(false, false, true),
                ],
            ),
            (
                "px",
                [1.0, 0.0, 0.0],
                [
                    at(true, false, false),
                    at(true, true, false),
                    at(true, true, true),
                    at(true, false, true),
                ],
            ),
            (
                "nx",
                [-1.0, 0.0, 0.0],
                [
                    at(false, false, false),
                    at(false, true, false),
                    at(false, true, true),
                    at(false, false, true),
                ],
            ),
            (
                "pz",
                [0.0, 0.0, 1.0],
                [
                    at(false, false, true),
                    at(true, false, true),
                    at(true, true, true),
                    at(false, true, true),
                ],
            ),
            (
                "nz",
                [0.0, 0.0, -1.0],
                [
                    at(false, false, false),
                    at(true, false, false),
                    at(true, true, false),
                    at(false, true, false),
                ],
            ),
        ] {
            if !skip.contains(name) {
                face(part, &corners, out, colour);
            }
        }
    }

    const WALL: [u8; 3] = [200, 190, 170];
    const TILES: [u8; 3] = [120, 60, 50];

    /// A building 10 m by 8 m, its walls 2.8 m high, under a `roof` of the game's pitch (its ridge along x), all in
    /// the plain finish.
    fn building(roof: Roof) -> Vec<Part> {
        let mut walls = part("walls", "built.plain");
        boxed(&mut walls, [0.0, 0.0, 0.0], [10.0, 2.8, 8.0], WALL, "py ny");
        let mut top = part("roof", "built.plain");
        let rise = 4.0 * 0.6745;
        let (e0, e1, r) = (2.8, 2.8 + rise, 4.0);
        match roof {
            Roof::Flat => boxed(&mut top, [-0.1, 2.8, -0.1], [10.1, 3.0, 8.1], TILES, "ny"),
            Roof::Gable => {
                face(
                    &mut top,
                    &[[0.0, e0, 0.0], [10.0, e0, 0.0], [10.0, e1, r], [0.0, e1, r]],
                    [0.0, 1.0, -1.0],
                    TILES,
                );
                face(
                    &mut top,
                    &[[0.0, e0, 8.0], [10.0, e0, 8.0], [10.0, e1, r], [0.0, e1, r]],
                    [0.0, 1.0, 1.0],
                    TILES,
                );
                face(
                    &mut walls,
                    &[[0.0, e0, 0.0], [0.0, e0, 8.0], [0.0, e1, r]],
                    [-1.0, 0.0, 0.0],
                    WALL,
                );
                face(
                    &mut walls,
                    &[[10.0, e0, 0.0], [10.0, e0, 8.0], [10.0, e1, r]],
                    [1.0, 0.0, 0.0],
                    WALL,
                );
            }
            Roof::Hip => {
                face(
                    &mut top,
                    &[[0.0, e0, 0.0], [10.0, e0, 0.0], [6.0, e1, r], [4.0, e1, r]],
                    [0.0, 1.0, -1.0],
                    TILES,
                );
                face(
                    &mut top,
                    &[[0.0, e0, 8.0], [10.0, e0, 8.0], [6.0, e1, r], [4.0, e1, r]],
                    [0.0, 1.0, 1.0],
                    TILES,
                );
                face(
                    &mut top,
                    &[[0.0, e0, 0.0], [0.0, e0, 8.0], [4.0, e1, r]],
                    [-1.0, 1.0, 0.0],
                    TILES,
                );
                face(
                    &mut top,
                    &[[10.0, e0, 0.0], [10.0, e0, 8.0], [6.0, e1, r]],
                    [1.0, 1.0, 0.0],
                    TILES,
                );
            }
        }
        vec![walls, top]
    }

    fn looked(parts: &[Part]) -> Look {
        look(parts, &Surfaces::new(), &mut Pictures::default())
    }

    #[test]
    fn the_play_camera_s_views_make_the_eight_ways_the_cutaway_stands() {
        let mut seen = std::collections::BTreeSet::new();
        for (x, z) in [(1.0, 0.0), (2.0, 1.0), (1.0, 2.0)] {
            for (x, z) in [(x, z), (-z, x), (-x, -z), (z, -x)] {
                seen.insert(look_bits([x, 1.5, z]));
            }
        }
        assert_eq!(
            seen.into_iter().collect::<Vec<_>>(),
            [1, 2, 4, 5, 6, 8, 9, 10]
        );
        // A camera out along +x looks along -x (bit 2); one out along the dump's +z, the game's -z, looks along +z (4).
        assert_eq!(look_bits([1.0, 0.3, 0.0]), 2);
        assert_eq!(look_bits([0.0, 0.3, 1.0]), 4);
    }

    #[test]
    fn the_cutaway_drops_a_wall_to_its_stub_and_takes_what_goes_with_it() {
        let index = Index {
            storey: 2.8,
            stub: 0.35,
            samples: Vec::new(),
        };
        let mut wall = part("wallsExterior", "built.plain");
        boxed(&mut wall, [0.0, 2.8, 0.0], [4.0, 5.6, 0.14], WALL, "");
        wall.storey = 1;
        // (Drops when the camera looks along +x or +z: mask 5 on both.)
        wall.cut = vec![[5, 1, 5, 0]; wall.mesh.positions.len()];
        let mut lamp = part("trim", "built.plain");
        boxed(&mut lamp, [1.0, 4.0, 0.14], [1.2, 4.2, 0.3], WALL, "");
        lamp.storey = 1;
        lamp.cut = vec![[4, 2, 4, 0]; lamp.mesh.positions.len()];
        let mut roof = part("roof", "material.roof");
        boxed(&mut roof, [0.0, 5.6, 0.0], [4.0, 5.8, 4.0], TILES, "");
        (roof.storey, roof.roof) = (1, true);
        let mut lid = part("ceilingShadow", "built.plain");
        boxed(&mut lid, [0.0, 5.5, 0.0], [4.0, 5.6, 4.0], WALL, "");
        lid.shadow_only = true;
        let parts = [wall, lamp, roof, lid];
        let top = |m: &Mesh| {
            m.positions
                .iter()
                .fold(f64::NEG_INFINITY, |t, p| t.max(p[1]))
        };
        // Standing: the wall and the lamp as built; no roof, nothing of the shadow map's.
        let up = cut_away(&parts, &Surfaces::new(), &index, 1, 0);
        assert_eq!(
            up.iter().map(|m| m.part.as_str()).collect::<Vec<_>>(),
            ["wallsExterior", "trim"]
        );
        assert_eq!((top(&up[0]), up[1].triangles.len()), (5.6, 12));
        // Looking along +x: the wall at its stub over its own storey's floor, the lamp (mask 4) still there.
        let x = cut_away(&parts, &Surfaces::new(), &index, 1, 1);
        assert_eq!((top(&x[0]), x[1].triangles.len()), (2.8 + 0.35, 12));
        // Along +z as well: the lamp goes. Along -x: nothing drops.
        let xz = cut_away(&parts, &Surfaces::new(), &index, 1, 5);
        assert_eq!((top(&xz[0]), xz[1].triangles.len()), (2.8 + 0.35, 0));
        assert_eq!(
            top(&cut_away(&parts, &Surfaces::new(), &index, 1, 2)[0]),
            5.6
        );
        // From the storey below, the one above is hidden.
        assert!(cut_away(&parts, &Surfaces::new(), &index, 0, 0).is_empty());
        // From far off: everything a camera draws.
        assert_eq!(drawn(&parts, &Surfaces::new()).len(), 3);
    }

    #[test]
    fn an_edge_on_the_same_line_in_two_states_counts_once() {
        let edge = |x: f64, length: f64| Edge {
            part: "trim".into(),
            material: "material.trim".into(),
            length,
            at: [x, 1.0, 2.0],
            along: [1.0, 0.0, 0.0],
        };
        // Two sills on one line in one state, more of them showing in another; an edge only the first has.
        let other = Edge {
            at: [0.0, 2.0, 2.0],
            ..edge(0.0, 0.5)
        };
        let mut all = Edges::of(vec![edge(1.0, 0.6), edge(3.0, 0.4), other]);
        assert_eq!((all.count(), all.length()), (3, 1.5));
        all.or(Edges::of(vec![
            edge(1.0, 0.8),
            edge(3.0, 0.8),
            edge(5.0, 0.2),
        ]));
        assert_eq!(all.count(), 4);
        assert!((all.length() - 2.3).abs() < 1e-9);
        all.or(Edges::of(vec![edge(-1.0, 0.1)]));
        assert_eq!(all.count(), 4);
        assert_eq!(all.longest()[0].length, 0.8);
        // The same line run the other way is the same line.
        let back = Edge {
            along: [-1.0, 0.0, 0.0],
            ..edge(2.0, 9.0)
        };
        all.or(Edges::of(vec![back]));
        assert_eq!((all.count(), all.longest()[0].length), (2, 9.0));
    }

    #[test]
    fn a_house_s_edges_are_those_of_every_state_it_is_seen_in() {
        // A room 4 m square under a roof, a chest standing in it: with the roof on nothing of the chest shows.
        let index = Index {
            storey: 2.8,
            stub: 0.35,
            samples: Vec::new(),
        };
        let mut walls = part("wallsExterior", "built.plain");
        for (lo, hi) in [
            ([0.0, 0.0, 0.0], [4.0, 2.8, 0.14]),
            ([0.0, 0.0, 3.86], [4.0, 2.8, 4.0]),
            ([0.0, 0.0, 0.14], [0.14, 2.8, 3.86]),
            ([3.86, 0.0, 0.14], [4.0, 2.8, 3.86]),
        ] {
            boxed(&mut walls, lo, hi, WALL, "ny");
        }
        walls.cut = vec![[15, 1, 15, 0]; walls.mesh.positions.len()];
        let mut roof = part("roof", "material.roof");
        boxed(&mut roof, [-0.3, 2.8, -0.3], [4.3, 3.0, 4.3], TILES, "");
        roof.roof = true;
        let mut chest = part("chest", "built.plain");
        boxed(&mut chest, [1.5, 0.0, 1.5], [2.5, 0.8, 2.5], WALL, "ny");
        let parts = [walls, roof, chest];
        let of = |edges: &Edges, part: &str| edges.0.keys().filter(|line| line.0 == part).count();
        let whole = Edges::of(edges::hard_edges(&drawn(&parts, &Surfaces::new())));
        assert_eq!(of(&whole, "chest"), 0);
        assert!(of(&whole, "roof") > 0);
        let house = house_edges(&parts, &Surfaces::new(), &index, 1);
        // Its top's four edges from above, its uprights once the walls on the camera's side are down.
        assert_eq!(
            of(&house, "chest"),
            8,
            "{:?}",
            house.0.keys().collect::<Vec<_>>()
        );
        assert_eq!(of(&house, "roof"), of(&whole, "roof"));
        // The walls' stubs have edges of their own, 0.35 m up.
        assert!(
            house
                .0
                .keys()
                .any(|line| line.0 == "wallsExterior" && line.3[1] == 350)
        );
    }

    #[test]
    fn a_building_s_look_is_measured_from_its_meshes() {
        let gable = looked(&building(Roof::Gable));
        assert_eq!(gable.roof, Roof::Gable);
        assert!((gable.pitch - 34.0).abs() < 0.5, "{}", gable.pitch);
        assert!(
            (gable.top - (2.8 + 4.0 * 0.6745)).abs() < 0.05,
            "{}",
            gable.top
        );
        assert!((gable.area - 80.0).abs() < 2.0, "{}", gable.area);
        assert!(
            (gable.plan[2] - gable.plan[0] - 10.0).abs() < 0.2
                && (gable.plan[3] - gable.plan[1] - 8.0).abs() < 0.2
        );
        assert_eq!(
            (gable.wall, gable.roof_colour, gable.chimneys),
            (Some(WALL), Some(TILES), 0)
        );
        let hip = looked(&building(Roof::Hip));
        assert_eq!(
            (hip.roof, hip.wall, hip.roof_colour),
            (Roof::Hip, Some(WALL), Some(TILES))
        );
        assert!((hip.pitch - 34.0).abs() < 0.5, "{}", hip.pitch);
        let flat = looked(&building(Roof::Flat));
        assert_eq!((flat.roof, flat.pitch), (Roof::Flat, 0.0));
        assert!((flat.top - 3.0).abs() < 1e-9);
        // A stack through the roof: its brick over the ridge, a cap on it.
        let mut stacked = building(Roof::Gable);
        let mut stack = part("brick", "built.plain");
        boxed(
            &mut stack,
            [2.0, 3.0, 3.2],
            [2.6, 6.3, 3.96],
            [150, 80, 60],
            "ny",
        );
        stacked.push(stack);
        let with = looked(&stacked);
        assert_eq!((with.chimneys, with.roof), (1, Roof::Gable));
        assert!((with.top - gable.top).abs() < 1e-9);
        // Glass and what only the shadow map draws are no wall's colour.
        let mut glazed = building(Roof::Gable);
        let mut panes = part("windows", "material.window");
        boxed(
            &mut panes,
            [-0.01, 0.5, -0.01],
            [10.01, 2.6, 8.01],
            [10, 20, 30],
            "py ny",
        );
        glazed.push(panes);
        assert_eq!(looked(&glazed).wall, Some(WALL));
    }

    #[test]
    fn a_far_version_disagrees_where_it_is_off_its_near_one() {
        let near = looked(&building(Roof::Gable));
        assert_eq!(disagreements(&near, &near), Vec::<String>::new());
        // A hip for a gable, a stack the near one hasn't, walls another paint.
        let mut far = building(Roof::Hip);
        far[0].colors.fill([240, 240, 235, 255]);
        let mut stack = part("brick", "built.plain");
        boxed(
            &mut stack,
            [2.0, 3.0, 3.2],
            [2.6, 6.3, 3.96],
            [150, 80, 60],
            "ny",
        );
        far.push(stack);
        let found = disagreements(&near, &looked(&far));
        assert_eq!(found.len(), 3, "{found:?}");
        assert_eq!(found[0], "roof: a hip roof, near a gable roof");
        assert_eq!(found[1], "chimney: it has one, near none");
        assert_eq!(
            found[2],
            "walls: most of them are #F0F0EB as drawn, near #C8BEAA (a channel at most 28 of 255 apart)"
        );
        // Twice as long, a storey taller, another roof colour: its plan, its height and its roof's colour.
        let mut tall = building(Roof::Gable);
        for p in tall.iter_mut() {
            for at in &mut p.mesh.positions {
                *at = [
                    at[0] * 2.0,
                    at[1] + if at[1] > 0.0 { 2.8 } else { 0.0 },
                    at[2],
                ];
            }
        }
        tall[1].colors.fill([60, 70, 90, 255]);
        let found = disagreements(&near, &looked(&tall));
        assert_eq!(
            found
                .iter()
                .map(|f| f.split(':').next().unwrap())
                .collect::<Vec<_>>(),
            ["plan", "height", "roof colour"],
            "{found:?}"
        );
        // Within the limits: eaves a little wider, a shade off.
        let mut close = building(Roof::Gable);
        for p in close.iter_mut() {
            for at in &mut p.mesh.positions {
                at[0] *= 1.03;
            }
            for c in &mut p.colors {
                *c = [c[0] - 10, c[1] - 10, c[2] - 10, 255];
            }
        }
        assert_eq!(disagreements(&near, &looked(&close)), Vec::<String>::new());
    }

    #[test]
    fn a_house_s_detail_is_what_its_meshes_have() {
        let mut trim = part("trim", "material.trim");
        boxed(&mut trim, [0.0, 0.0, 0.0], [0.1, 2.0, 0.1], WALL, "ny");
        let plain = detail(&[trim.clone()], &Surfaces::new());
        assert!((plain.trim - (4.0 * 0.2 + 0.01)).abs() < 1e-9);
        assert_eq!(
            (plain.trim_shaped, plain.eased, plain.eased_upright),
            (0.0, 0.0, 0.0)
        );
        // A chamfer down one corner: a face neither plumb nor level it isn't, but two eased edges it has.
        face(
            &mut trim,
            &[
                [0.1, 0.0, 0.09],
                [0.11, 0.0, 0.1],
                [0.11, 2.0, 0.1],
                [0.1, 2.0, 0.09],
            ],
            [1.0, 0.0, -1.0],
            WALL,
        );
        // A weathered sill: a face that slopes.
        face(
            &mut trim,
            &[
                [0.0, 2.0, 0.0],
                [0.5, 2.0, 0.0],
                [0.5, 1.9, 0.2],
                [0.0, 1.9, 0.2],
            ],
            [0.0, 1.0, 0.5],
            WALL,
        );
        let shaped = detail(&[trim], &Surfaces::new());
        assert!(
            (shaped.trim_shaped - 0.5 * (0.05_f64).sqrt()).abs() < 1e-9,
            "{}",
            shaped.trim_shaped
        );
        // A hip roof: its ridge, its four hips, no cap shingles; with caps in another tone, their area.
        let mut roof = building(Roof::Hip).remove(1);
        (roof.mesh.material, roof.roof) = ("material.roof".into(), true);
        let bare = detail(&[roof.clone()], &Surfaces::new());
        let hip = (16.0_f64 + 16.0 + (4.0_f64 * 0.6745).powi(2)).sqrt();
        assert!(
            (bare.roofs[&0][0] - 2.0).abs() < 1e-6 && (bare.roofs[&0][1] - 4.0 * hip).abs() < 1e-6,
            "{:?}",
            bare.roofs
        );
        assert_eq!(bare.roofs[&0][2], 0.0);
        face(
            &mut roof,
            &[
                [4.0, 5.6, 3.9],
                [6.0, 5.6, 3.9],
                [6.0, 5.6, 4.1],
                [4.0, 5.6, 4.1],
            ],
            [0.0, 1.0, 0.0],
            [100, 50, 40],
        );
        let capped = detail(&[roof], &Surfaces::new());
        assert!((capped.roofs[&0][2] - 0.4).abs() < 1e-9);
        assert!((capped.roofs[&0][1] - 4.0 * hip).abs() < 1e-6);
        // A fascia at the eaves and the slopes on it; then a slope that dives 1.8 m under it.
        let mut fascia = part("roofTrim", "material.trim");
        let board = [
            [0.0, 2.8, 0.0],
            [10.0, 2.8, 0.0],
            [10.0, 2.94, 0.0],
            [0.0, 2.94, 0.0],
        ];
        face(&mut fascia, &board, [0.0, 0.0, -1.0], WALL);
        fascia.roof = true;
        let mut slopes = building(Roof::Hip).remove(1);
        (slopes.mesh.material, slopes.roof) = ("material.roof".into(), true);
        let sound = detail(&[fascia.clone(), slopes.clone()], &Surfaces::new());
        assert_eq!(sound.roofs[&0][3], 0.0);
        let dive = [[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.5, 1.0]];
        face(&mut slopes, &dive, [0.0, 1.0, 0.0], TILES);
        let sunk = detail(&[fascia, slopes], &Surfaces::new());
        assert!((sunk.roofs[&0][3] - 1.8).abs() < 1e-9, "{:?}", sunk.roofs);
    }

    /// `parts` as the dump writes a sample's file (tools/built/glb.ts).
    fn file_of(parts: &[Part]) -> Vec<u8> {
        let mut bin: Vec<u8> = Vec::new();
        let (mut views, mut accessors, mut meshes, mut nodes, mut materials) = (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::<String>::new(),
        );
        let mut accessor = |bin: &mut Vec<u8>,
                            bytes: Vec<u8>,
                            count: usize,
                            kind: &str,
                            component: u32,
                            normalized: bool| {
            views.push(format!(
                "{{\"buffer\": 0, \"byteOffset\": {}, \"byteLength\": {}}}",
                bin.len(),
                bytes.len()
            ));
            bin.extend(bytes);
            while !bin.len().is_multiple_of(4) {
                bin.push(0);
            }
            accessors.push(format!(
                "{{\"bufferView\": {}, \"componentType\": {component}, \"count\": {count}, \"type\": \"{kind}\"{}}}",
                views.len() - 1,
                if normalized { ", \"normalized\": true" } else { "" }
            ));
            accessors.len() - 1
        };
        for p in parts {
            let floats = |v: Vec<f64>| {
                v.iter()
                    .flat_map(|x| (*x as f32).to_le_bytes())
                    .collect::<Vec<u8>>()
            };
            let n = p.mesh.positions.len();
            let mut attributes = vec![
                format!(
                    "\"POSITION\": {}",
                    accessor(
                        &mut bin,
                        floats(p.mesh.positions.concat()),
                        n,
                        "VEC3",
                        5126,
                        false
                    )
                ),
                format!(
                    "\"NORMAL\": {}",
                    accessor(
                        &mut bin,
                        floats(p.mesh.normals.concat()),
                        n,
                        "VEC3",
                        5126,
                        false
                    )
                ),
                format!(
                    "\"TEXCOORD_0\": {}",
                    accessor(
                        &mut bin,
                        floats(p.mesh.uvs.concat()),
                        n,
                        "VEC2",
                        5126,
                        false
                    )
                ),
                format!(
                    "\"COLOR_0\": {}",
                    accessor(&mut bin, p.colors.concat(), n, "VEC4", 5121, true)
                ),
            ];
            if !p.cut.is_empty() {
                attributes.push(format!(
                    "\"_CUT\": {}",
                    accessor(&mut bin, p.cut.concat(), n, "VEC4", 5121, false)
                ));
            }
            let indices: Vec<u8> = p
                .mesh
                .triangles
                .concat()
                .iter()
                .flat_map(|i| i.to_le_bytes())
                .collect();
            let indices = accessor(
                &mut bin,
                indices,
                p.mesh.triangles.len() * 3,
                "SCALAR",
                5125,
                false,
            );
            let material = materials
                .iter()
                .position(|m| m == &p.mesh.material)
                .unwrap_or_else(|| {
                    materials.push(p.mesh.material.clone());
                    materials.len() - 1
                });
            meshes.push(format!(
                "{{\"primitives\": [{{\"attributes\": {{{}}}, \"indices\": {indices}, \"material\": {material}}}]}}",
                attributes.join(", ")
            ));
            nodes.push(format!(
                "{{\"name\": \"{}\", \"mesh\": {}, \"extras\": {{\"storey\": {}, \"roof\": {}, \"shadowOnly\": {}}}}}",
                p.mesh.part,
                meshes.len() - 1,
                p.storey,
                p.roof,
                p.shadow_only
            ));
        }
        let materials: Vec<String> = materials
            .iter()
            .map(|m| format!("{{\"name\": \"{m}\"}}"))
            .collect();
        let json = format!(
            "{{\"asset\": {{\"version\": \"2.0\"}}, \"nodes\": [{}], \"meshes\": [{}], \"materials\": [{}], \"accessors\": [{}], \"bufferViews\": [{}], \"buffers\": [{{\"byteLength\": {}}}]}}",
            nodes.join(", "),
            meshes.join(", "),
            materials.join(", "),
            accessors.join(", "),
            views.join(", "),
            bin.len()
        );
        glb(&json, &bin)
    }

    #[test]
    fn a_sample_s_file_reads_back_as_its_parts() {
        let mut wall = part("wallsExterior", "material.wall.brick");
        boxed(&mut wall, [0.0, 0.0, 0.0], [4.0, 2.5, 0.25], WALL, "ny");
        wall.cut = vec![[5, 1, 5, 0]; wall.mesh.positions.len()];
        let mut roof = part("roof", "material.roof");
        boxed(&mut roof, [0.0, 2.5, 0.0], [4.0, 2.75, 4.0], TILES, "");
        (roof.storey, roof.roof) = (1, true);
        let mut glass = part("windows", "material.window.clear");
        boxed(&mut glass, [1.0, 1.0, 0.0], [2.0, 2.0, 0.25], [1, 2, 3], "");
        glass.shadow_only = true;
        let parts = vec![wall, roof, glass];
        let back = read_parts(&file_of(&parts)).unwrap();
        assert_eq!(back.len(), 3);
        for (a, b) in parts.iter().zip(&back) {
            assert_eq!(a.mesh, b.mesh);
            assert_eq!(
                (&a.colors, &a.cut, a.storey, a.roof, a.shadow_only),
                (&b.colors, &b.cut, b.storey, b.roof, b.shadow_only)
            );
        }
        // Glass has no edges to judge; a file that is no .glb is an error.
        assert!(!back[2].mesh.edged && back[0].mesh.edged);
        assert!(read_parts(b"no model").is_err());
    }

    /// The gate over a dump of two samples, a chest and its far version: the baseline's verdicts, `seeded` writing
    /// the sections a baseline has none of, the lowering taking out and never adding.
    #[test]
    fn the_gate_judges_a_dump_by_the_baseline() {
        let dir = std::env::temp_dir().join(format!("ll-gate-built-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let at = |name: &str| dir.join(name).to_string_lossy().into_owned();
        let mut paths = Paths::repo();
        paths.baseline = at("gate-baseline.json");
        let dump = at("dump");
        std::fs::create_dir_all(&dump).unwrap();
        let mut chest = part("chest", "built.plain");
        boxed(&mut chest, [0.0, 0.0, 0.0], [1.0, 0.8, 1.0], WALL, "ny");
        std::fs::write(join(&dump, "car.chest.glb"), file_of(&[chest])).unwrap();
        std::fs::write(
            join(&dump, "house.box.glb"),
            file_of(&building(Roof::Gable)),
        )
        .unwrap();
        std::fs::write(join(&dump, "far.box.glb"), file_of(&building(Roof::Hip))).unwrap();
        std::fs::write(
            join(&dump, "built.json"),
            r#"{"storey": 2.8, "stub": 0.35, "samples": [
                {"key": "car.chest", "kind": "car", "about": "a chest"},
                {"key": "house.box", "kind": "house", "storeys": 1, "about": "a box"},
                {"key": "far.box", "kind": "far", "of": "house.box", "storeys": 1, "about": "a box"}]}"#,
        )
        .unwrap();
        let set = |text: &str| std::fs::write(&paths.baseline, text).unwrap();
        let run = || gate(&paths, &dump, &[]);
        // No baseline: every finding fails.
        let r = run();
        assert_eq!(r.failed, 3, "{}", r.text);
        assert!(r.text.starts_with("FAIL car.chest                    10 triangles, 1 draws\n       error: 8 hard edges of 5 cm or more show unbevelled at the play camera, 7.2 m"), "{}", r.text);
        assert!(r.text.contains("FAIL far.box                      14 triangles, 2 draws\n       error: roof: a hip roof, near a gable roof\n"), "{}", r.text);
        assert!(r.text.ends_with("3 samples, 3 failed\n"), "{}", r.text);
        assert_eq!(r.counts[EDGES]["car.chest"], 8);
        assert_eq!(
            (r.counts[FAR_NEAR]["far.box"], r.counts[FAR_NEAR].len()),
            (1, 1)
        );
        // (A far version is judged for neither edges nor texels.)
        assert!(
            !r.counts[EDGES].contains_key("far.box") && r.counts[EDGES].contains_key("house.box")
        );
        // Seeded: the sections it had none of, of the samples with findings; then it passes, its findings printed.
        let (seed, written) = seeded(&Baseline::default(), &r.counts).unwrap();
        assert_eq!(written.len(), 2, "{written:?}");
        assert_eq!(written[1], "built.farNear: 1 samples, 1 findings");
        seed.write(&paths.baseline).unwrap();
        let r = run();
        assert_eq!(r.failed, 0, "{}", r.text);
        assert!(r.text.contains("ok   car.chest                    10 triangles, 1 draws\n       warning: built.edges: 8 found, as the baseline has (tools/art/gate-baseline.json)\n"), "{}", r.text);
        // A section it has is left as it is; the models' sections too.
        let (again, written) = seeded(&seed, &r.counts).unwrap();
        assert_eq!((again, written), (seed.clone(), Vec::new()));
        // One sample alone, by its key; a key the dump hasn't.
        assert!(
            gate(&paths, &dump, &["car.chest".into()])
                .text
                .ends_with("1 samples, 0 failed\n")
        );
        assert_eq!(gate(&paths, &dump, &["car.none".into()]).failed, 1);
        // Higher than found: stale, with this gate's own command; lowered, never raised.
        set(
            "{\"built.edges\": {\"car.chest\": 9, \"house.box\": 1, \"fence.gone\": 2}, \"built.farNear\": {\"far.box\": 1}, \"edges\": {\"model.x\": 1}}\n",
        );
        let r = run();
        assert_eq!(r.failed, 3, "{}", r.text);
        assert!(r.text.contains("error: built.edges: 8 found, 9 in the baseline: lower it (`cargo check-glb --built <dir> --update-baseline`)"), "{}", r.text);
        assert!(r.text.contains("FAIL gate-baseline.json\n       error: built.edges: fence.gone is no sample this check ran on"), "{}", r.text);
        let (low, changes) = Baseline::read(&paths.baseline).unwrap().lowered(&r.counts);
        assert_eq!(
            changes,
            [
                "built.edges: car.chest 9 -> 8",
                "built.edges: fence.gone out (was 2)"
            ]
        );
        // (Over its count, a sample fails until it is fixed: the baseline is never raised.)
        assert_eq!(
            (low.get(EDGES, "house.box"), low.get("edges", "model.x")),
            (1, 1)
        );
        // A dump that isn't there.
        assert!(
            gate(&paths, &at("none"), &[])
                .text
                .starts_with("FAIL the built world\n       error: cannot read it: ")
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The report has a row for every sample and every pair.
    #[test]
    fn the_report_lists_what_a_run_measured() {
        let dir = std::env::temp_dir().join(format!("ll-gate-built-report-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dump = dir.to_string_lossy().into_owned();
        let mut paths = Paths::repo();
        paths.baseline = join(&dump, "gate-baseline.json");
        let mut near = building(Roof::Gable);
        near[0].mesh.material = "material.wall.siding".into();
        std::fs::write(join(&dump, "house.box.glb"), file_of(&near)).unwrap();
        std::fs::write(join(&dump, "far.box.glb"), file_of(&building(Roof::Gable))).unwrap();
        std::fs::write(
            join(&dump, "built.json"),
            r#"{"storey": 2.8, "stub": 0.35, "samples": [{"key": "house.box", "kind": "house", "storeys": 1}, {"key": "far.box", "kind": "far", "of": "house.box"}]}"#,
        )
        .unwrap();
        let text = report(&gate(&paths, &dump, &[]));
        assert!(text.contains("| house.box | 14 | 2 | "), "{text}");
        assert!(text.contains("| far.box | 14 | 2 | — | — | — | "), "{text}");
        assert!(
            text.contains("| house | walls | upright | 4 | 11.2 | "),
            "{text}"
        );
        // The siding as its UVs lay it: the test's are a metre a metre, so the manifest's 0.4 of 1024 px.
        assert!(
            text.contains("| material.wall.siding | 1024 | 0.4 | 1 | "),
            "{text}"
        );
        assert!(text.contains("| house.box | 80 | "), "{text}");
        assert!(text.contains("| far.box | 80 | "), "{text}");
        assert!(
            text.contains("| house.box | 0.0 | 0.00 | 0.0 | 20.0 | 0.00 |"),
            "{text}"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
