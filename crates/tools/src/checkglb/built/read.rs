//! Reading the built world: the sample list, its parts and surfaces, and pictures.

use super::*;

/// The checks' names: their sections in the baseline.
pub const EDGES: &str = "built.edges";
pub const TEXELS: &str = "built.texels";
pub const FAR_NEAR: &str = "built.farNear";
/// The baseline's sections that are this gate's (`checkglb::COUNTED` are the models').
pub const CHECKS: [&str; 3] = [EDGES, TEXELS, FAR_NEAR];

/// The sample kinds that are far versions: judged against what they are the far version of, not for edges or texels.
pub const FAR: [&str; 4] = ["neighbour", "far", "town", "buildingFar"];
/// The far versions drawn from so far that no edge or texel of theirs shows (a neighbour's silhouette is across
/// the street: it is judged for both).
pub const DISTANT: [&str; 3] = ["far", "town", "buildingFar"];

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
    #[allow(dead_code)]
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
pub fn glass(material: &str) -> bool {
    material.starts_with("material.window")
}

/// Whether a material is laid on what is under it and blended (contact shadows, pools of lamplight): it hides
/// nothing and has no edges.
pub fn laid(material: &str) -> bool {
    matches!(material, "built.shadow" | "built.lightPool")
}

/// Whether a material's edges are judged: not glass, nothing laid or painted on the ground, nothing that glows.
pub fn edged(material: &str) -> bool {
    !glass(material) && !laid(material) && !matches!(material, "built.glow" | "built.roadPaint")
}

/// Whether a material hides what is behind it.
pub fn opaque(material: &str, surfaces: &Surfaces) -> bool {
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
pub fn hex(s: &str) -> Option<[u8; 3]> {
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
    pub(crate) fn side(&mut self, file: &str) -> Option<f64> {
        *self.sides.entry(file.to_string()).or_insert_with(|| {
            let (w, h) = image_size(&std::fs::read(file).ok()?)?;
            Some((w as f64 * h as f64).sqrt())
        })
    }

    pub(crate) fn mean(&mut self, file: &str) -> Option<[f64; 3]> {
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
pub fn linear(v: f64) -> f64 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

pub fn srgb(v: f64) -> f64 {
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
                Some(i) => super::super::glb::index(gltf.get("materials").unwrap_or(&none), i)?
                    .get("name")
                    .and_then(J::str)
                    .unwrap_or_default()
                    .to_string(),
                None => String::new(),
            };
            let triangles = super::super::digest::indices(g, prim, positions.len())?
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
