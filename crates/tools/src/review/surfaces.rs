//! `cargo review sizes`' surfaces: the sizes of the textures' patterns (a brick, a board, a tile, a shingle's
//! exposure) and the street's kerb against real ones (docs/review/reference.json `surfaces`). A pattern's size
//! is its share of its texture's tile (`tex::synth::features`) over the `uvScale` of each material that
//! draws it, the UVs being metres. A report: a size outside its range is listed, not failed.

use super::within;
use crate::tex::synth;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// The manifest, whose materials give each set its `uvScale`.
pub const MANIFEST: &str = "web/public/assets/manifest.json";
/// The street's geometry: the levels of the kerb's top and the road.
pub const ROADS: &str = "web/src/render/shared/roads.ts";
/// What a kind names in `of` for the street's geometry rather than a texture set.
const STREET: &str = "street";

/// A kind of surface: its real sizes (m) by feature, and what is held to them.
#[derive(Debug, Default, Deserialize)]
pub struct SurfaceKind {
    #[serde(default)]
    pub label: String,
    /// Texture sets by name, or `street` (the kerb, `roads.ts`).
    #[serde(default)]
    pub of: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    /// A set's features (`tex::synth::features`), or the street's `height` (the kerb's face over the road).
    #[serde(flatten)]
    pub sizes: BTreeMap<String, [f64; 2]>,
}

/// One size against its kind's range.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub kind: String,
    /// The set, or `street`.
    pub of: String,
    /// The material that shows the set ("" for the street).
    pub material: String,
    pub feature: String,
    /// Metres.
    pub size: f64,
    pub range: [f64; 2],
}

impl Row {
    pub fn inside(&self) -> bool {
        within(self.size, self.range, 0.0)
    }
}

/// The kerb's face over the road (m), from `roads.ts`' levels of the kerb's top and the road.
pub fn kerb_height(roads: &str) -> Result<f64, String> {
    let level = |name: &str| -> Result<f64, String> {
        let re =
            Regex::new(&format!(r"export const {name} = ([0-9.]+);")).map_err(|e| e.to_string())?;
        re.captures(roads)
            .and_then(|c| c[1].parse().ok())
            .ok_or_else(|| format!("{ROADS}: no `export const {name}`"))
    };
    Ok(level("KERB_Y")? - level("ROAD_Y")?)
}

/// Every kind's sizes against what the textures (through the manifest's materials) and the street draw. An
/// error for a kind that names a set or a feature there isn't, or a set no material draws.
pub fn rows(
    kinds: &BTreeMap<String, SurfaceKind>,
    manifest: &Value,
    roads: &str,
) -> Result<Vec<Row>, String> {
    let materials: Vec<(&str, &str, f64)> = manifest["entries"]
        .as_object()
        .ok_or_else(|| format!("{MANIFEST}: no entries"))?
        .iter()
        .filter_map(|(key, e)| {
            let set = synth::set_of_texture(e.get("texture")?.as_str()?)?;
            Some((
                key.as_str(),
                set.name,
                e.get("uvScale").and_then(Value::as_f64).unwrap_or(1.0),
            ))
        })
        .collect();
    let mut rows = Vec::new();
    for (kind, k) in kinds {
        for of in &k.of {
            let row = |material: &str, feature: &str, size: f64, range: [f64; 2]| Row {
                kind: kind.clone(),
                of: of.clone(),
                material: material.to_string(),
                feature: feature.to_string(),
                size,
                range,
            };
            if of == STREET {
                for (feature, &range) in &k.sizes {
                    if feature != "height" {
                        return Err(format!(
                            "{kind}: the street has a `height` (the kerb's), not `{feature}`"
                        ));
                    }
                    rows.push(row("", feature, kerb_height(roads)?, range));
                }
                continue;
            }
            if !synth::SETS.iter().any(|s| s.name == of) {
                return Err(format!("{kind}: no texture set `{of}`"));
            }
            let features = synth::features(of);
            let drawn: Vec<&(&str, &str, f64)> = materials.iter().filter(|m| m.1 == of).collect();
            if drawn.is_empty() {
                return Err(format!("{kind}: no material in {MANIFEST} draws `{of}`"));
            }
            for (feature, &range) in &k.sizes {
                let Some(&(_, uv)) = features.iter().find(|f| f.0 == feature) else {
                    return Err(format!("{kind}: `{of}` has no feature `{feature}`"));
                };
                for (material, _, scale) in &drawn {
                    rows.push(row(material, feature, uv / scale, range));
                }
            }
        }
    }
    Ok(rows)
}

/// The rows as a section of `cargo review sizes`' report, in millimetres.
pub fn report(rows: &[Row]) -> String {
    let outside = rows.iter().filter(|r| !r.inside()).count();
    let mut out = format!(
        "\n## Surfaces: {outside} of {} outside\n\nEach texture's pattern at the `uvScale` of a material that draws it, and the kerb, against its kind's real sizes (strictly).\n\n| Kind | Of | Material | Feature | Size (mm) | Range (mm) | Status |\n|---|---|---|---|---|---|---|\n",
        rows.len()
    );
    for r in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {:.1} | {:.1}–{:.1} | {} |",
            r.kind,
            r.of,
            if r.material.is_empty() {
                "–"
            } else {
                &r.material
            },
            r.feature,
            r.size * 1000.0,
            r.range[0] * 1000.0,
            r.range[1] * 1000.0,
            if r.inside() { "ok" } else { "outside" }
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::{REFERENCE, Reference};
    use serde_json::json;

    fn kinds(v: Value) -> BTreeMap<String, SurfaceKind> {
        serde_json::from_value(v).unwrap()
    }

    const ROADS_TS: &str =
        "export const KERB = 0.14;\nexport const KERB_Y = 0.15;\nexport const ROAD_Y = 0.01;\n";

    #[test]
    fn a_size_is_its_share_of_the_tile_over_each_material_s_uv_scale() {
        let manifest = json!({"entries": {
            "material.wall.brick": {"type": "material", "texture": "textures/brick/albedo.jpg", "uvScale": 0.5},
            "material.other": {"type": "material", "texture": "textures/brick/albedo.jpg", "uvScale": 1.0},
            "material.plain": {"type": "material", "color": "#FFFFFF"},
        }});
        let k = kinds(json!({"brick": {"of": ["brick"], "length": [0.18, 0.2]}}));
        let rows = rows(&k, &manifest, ROADS_TS).unwrap();
        assert_eq!(rows.len(), 2);
        let at = |m: &str| rows.iter().find(|r| r.material == m).unwrap();
        // 190 mm at the 1 m tile it is drawn at; shown at 2 m, 380.
        assert!((at("material.wall.brick").size - 0.38).abs() < 1e-12);
        assert!(!at("material.wall.brick").inside());
        assert!(at("material.other").inside());
        let text = report(&rows);
        assert!(text.contains("## Surfaces: 1 of 2 outside"), "{text}");
        assert!(
            text.contains(
                "| brick | brick | material.wall.brick | length | 380.0 | 180.0–200.0 | outside |"
            ),
            "{text}"
        );
    }

    #[test]
    fn the_kerb_is_its_top_over_the_road() {
        assert!((kerb_height(ROADS_TS).unwrap() - 0.14).abs() < 1e-12);
        assert!(kerb_height("export const KERB = 0.14;").is_err());
        let k = kinds(json!({"kerb": {"of": ["street"], "height": [0.1, 0.15]}}));
        let rows = rows(&k, &json!({"entries": {}}), ROADS_TS).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].inside() && rows[0].material.is_empty());
    }

    #[test]
    fn a_set_or_feature_there_isnt_is_an_error() {
        let manifest = json!({"entries": {
            "material.wall.brick": {"type": "material", "texture": "textures/brick/albedo.jpg", "uvScale": 1.0},
        }});
        for (bad, why) in [
            (
                json!({"k": {"of": ["bricks"], "length": [0.1, 0.2]}}),
                "no texture set",
            ),
            (
                json!({"k": {"of": ["brick"], "width": [0.1, 0.2]}}),
                "no feature",
            ),
            (
                json!({"k": {"of": ["floor_tile"], "size": [0.3, 0.6]}}),
                "no material",
            ),
            (
                json!({"k": {"of": ["street"], "width": [0.1, 0.2]}}),
                "the street has a",
            ),
        ] {
            let e = rows(&kinds(bad), &manifest, ROADS_TS).unwrap_err();
            assert!(e.contains(why), "{e}");
        }
    }

    /// Every set with a pattern of real size is held to a kind, and the shipped reference reads against the
    /// shipped manifest and street.
    #[test]
    fn the_shipped_surfaces_read() {
        let root = crate::content::root();
        let read = |rel: &str| std::fs::read_to_string(root.join(rel)).unwrap();
        let reference = Reference::parse(&read(REFERENCE)).unwrap();
        let manifest: Value = serde_json::from_str(&read(MANIFEST)).unwrap();
        let rows = rows(&reference.surfaces, &manifest, &read(ROADS)).unwrap();
        for set in synth::SETS {
            for (feature, _) in synth::features(set.name) {
                let held = reference
                    .surfaces
                    .values()
                    .any(|k| k.of.iter().any(|o| o == set.name) && k.sizes.contains_key(feature));
                assert!(held, "{} {feature}: no surface kind holds it", set.name);
            }
        }
        assert!(rows.iter().any(|r| r.of == STREET));
        for (name, k) in &reference.surfaces {
            assert!(
                !k.label.is_empty() && !k.sources.is_empty(),
                "{name}: a label and sources"
            );
            for s in &k.sources {
                assert!(
                    reference.sources.contains_key(s),
                    "{name}: source `{s}` isn't in `sources`"
                );
            }
            for (feature, [lo, hi]) in &k.sizes {
                assert!(0.0 < *lo && lo <= hi, "{name} {feature}: [{lo}, {hi}]");
            }
        }
    }
}
