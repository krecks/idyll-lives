//! The game's finishes in glTF's terms (the metallic-roughness model, `KHR_materials_clearcoat`,
//! `KHR_materials_sheen`), and which of them a real surface can be.
//!
//! A finish is a manifest `material.finish.*` entry: `metallic`, `roughness`, and where it has them
//! `clearcoat`, `clearcoatRoughness`, `sheenColor`, `sheenRoughness` and `alpha`. A number anywhere between
//! 0 and 1 is easy to write and often no surface at all, so the ranges here are the ones real surfaces fall
//! in, and the three plain classes (gloss, satin, matte; chrome, brushed, cast for a metal) are kept apart:
//! a roughness between two of them is neither.
//!
//! Read by `cargo tex materials` and its test (every finish the tool writes), by `cargo review colours`
//! (the metals' colours), and there for the gate (a pack entry's own `roughness` and `metallic`: `own_faults`).

use serde_json::Value as J;
use std::path::Path;

/// A dielectric's roughness: polished (glazed ceramic, enamel, wet paint) at most this.
pub const GLOSS_MOST: f64 = 0.25;
/// Semi-gloss (eggshell paint, moulded plastic, waxed wood, skin).
pub const SATIN: [f64; 2] = [0.3, 0.55];
/// Matte (emulsion paint, plaster, rubber, cloth) at least this.
pub const MATTE_LEAST: f64 = 0.65;
/// A metal's roughness: plated or polished at most this.
pub const CHROME_MOST: f64 = 0.15;
/// Brushed or satin-finished metal.
pub const BRUSHED: [f64; 2] = [0.3, 0.5];
/// Cast, forged or weathered metal. Rougher than that it is rust or paint: a dielectric.
pub const CAST: [f64; 2] = [0.5, 0.8];
/// A textured finish's roughness, its scalar times its map's mean: satin to matte. Its map spreads over the
/// classes (foliage: waxy leaves beside dull ones), so it may lie between two; none is polished, whose
/// mirrored sky would show the texture's repeat.
pub const TEXTURED: [f64; 2] = [0.3, 1.0];
/// A metal's base colour is what it reflects face on (linear luminance): the darkest metals in use
/// (titanium 0.50, chromium 0.55, iron 0.57) are all above this. Anything darker is an oxide or a
/// coating, which is a dielectric: black iron is a dark satin, not a metal.
pub const METAL_LUMINANCE_LEAST: f64 = 0.45;
/// A clear coat is polished or it is not a coat (lacquer, varnish, car paint).
pub const COAT_ROUGHNESS_MOST: f64 = 0.3;
/// A sheen's roughness: fibres scatter widely; under 0.3 the lobe is a line at the silhouette.
pub const SHEEN_ROUGHNESS: [f64; 2] = [0.3, 1.0];
/// A see-through finish's opacity: under 0.1 the pane is not there, over 0.5 it is not glass.
pub const GLASS_ALPHA: [f64; 2] = [0.1, 0.5];

/// glTF material parameters the game does not take, each with why.
pub const REJECTED: [(&str, &str); 3] = [
    (
        "ior",
        "every dielectric reflects 4 % face on (index 1.5): the look's diffuse colour takes that off, and the plain finishes share one material",
    ),
    (
        "specular",
        "it scales the same 4 %, which the look and the shared plain material fix; a duller surface is a rougher one",
    ),
    (
        "transmission",
        "it needs a picture of the scene behind the surface every frame; a see-through finish is `alpha`",
    ),
];

/// A roughness's class (gloss, satin, matte; a metal's chrome, brushed, cast), or none between two.
pub fn class(roughness: f64, metal: bool) -> Option<&'static str> {
    let within = |[lo, hi]: [f64; 2]| (lo..=hi).contains(&roughness);
    if !(0.0..=1.0).contains(&roughness) {
        None
    } else if metal {
        if roughness <= CHROME_MOST {
            Some("chrome")
        } else if within(BRUSHED) {
            Some("brushed")
        } else {
            within(CAST).then_some("cast")
        }
    } else if roughness <= GLOSS_MOST {
        Some("gloss")
    } else if within(SATIN) {
        Some("satin")
    } else {
        (roughness >= MATTE_LEAST).then_some("matte")
    }
}

/// The classes' ranges as a fault's text.
fn classes(metal: bool) -> String {
    if metal {
        format!(
            "chrome up to {CHROME_MOST}, brushed {}-{}, cast {}-{}",
            BRUSHED[0], BRUSHED[1], CAST[0], CAST[1]
        )
    } else {
        format!(
            "gloss up to {GLOSS_MOST}, satin {}-{}, matte from {MATTE_LEAST}",
            SATIN[0], SATIN[1]
        )
    }
}

/// `metallic` is 0 or 1: a surface is a metal or it is not (a blend is a texture's business, at an edge).
fn metal_of(metallic: f64) -> Result<bool, String> {
    if metallic == 0.0 {
        Ok(false)
    } else if metallic == 1.0 {
        Ok(true)
    } else {
        Err(format!("`metallic` {metallic}: 0 or 1"))
    }
}

/// A plain surface's roughness against its classes.
fn plain_fault(roughness: f64, metal: bool) -> Option<String> {
    class(roughness, metal).is_none().then(|| {
        format!(
            "{} roughness {roughness}: {}",
            if metal { "metal" } else { "dielectric" },
            classes(metal)
        )
    })
}

/// A colour as the manifests write it (`#RRGGBB`, sRGB): its linear red, green and blue.
pub fn linear(hex: &str) -> Option<[f64; 3]> {
    let hex = hex
        .strip_prefix('#')
        .filter(|h| h.len() == 6 && h.is_ascii())?;
    let mut out = [0.0; 3];
    for (i, c) in out.iter_mut().enumerate() {
        let v = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()? as f64 / 255.0;
        *c = if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        };
    }
    Some(out)
}

/// The mean roughness (green) and metalness (blue) of a packed `arm` map.
pub fn arm_mean(file: &Path) -> Result<[f64; 2], String> {
    let image = image::open(file)
        .map_err(|e| format!("{}: {e}", file.display()))?
        .to_rgb8();
    let n = (image.width() as f64 * image.height() as f64).max(1.0) * 255.0;
    let mut sum = [0u64; 2];
    for p in image.pixels() {
        sum[0] += p.0[1] as u64;
        sum[1] += p.0[2] as u64;
    }
    Ok([sum[0] as f64 / n, sum[1] as f64 / n])
}

/// What is wrong with a finish (a manifest `material.finish.*` entry); `arm` is its packed map's mean
/// roughness and metalness (`arm_mean`), which its own multiply, where it has one.
pub fn finish_faults(entry: &J, arm: Option<[f64; 2]>) -> Vec<String> {
    let mut out = Vec::new();
    let number = |name: &str| entry.get(name).and_then(J::as_f64);
    for (name, why) in REJECTED {
        if entry.get(name).is_some() {
            out.push(format!("`{name}` is not taken: {why}"));
        }
    }
    // (The game's defaults: `MaterialLibrary.create`.)
    let textured = entry.get("texture").is_some();
    let metallic = number("metallic").unwrap_or(if arm.is_some() { 1.0 } else { 0.0 });
    let roughness = number("roughness").unwrap_or(if arm.is_some() { 1.0 } else { 0.8 });
    let metal = match metal_of(metallic) {
        Ok(metal) => metal,
        Err(e) => {
            out.push(e);
            return out;
        }
    };
    if textured {
        // The scalars multiply the map: metal only where the map says so, and no tiling set is one.
        let [rough, metal] = arm.unwrap_or([1.0, 1.0]);
        let (rough, metal) = (roughness * rough, metallic * metal);
        if metal > 0.05 {
            out.push(format!(
                "its map is {:.0} % metal: a textured finish is a dielectric",
                metal * 100.0
            ));
        }
        if !(TEXTURED[0]..=TEXTURED[1]).contains(&rough) {
            out.push(format!(
                "roughness {rough:.2} (its own times its map's mean): {}-{}",
                TEXTURED[0], TEXTURED[1]
            ));
        }
    } else {
        out.extend(plain_fault(roughness, metal));
    }
    if metal
        && !textured
        && let Some(l) = entry
            .get("color")
            .and_then(J::as_str)
            .and_then(linear)
            .map(crate::colours::luminance)
        && l < METAL_LUMINANCE_LEAST
    {
        out.push(format!(
            "a metal reflecting {:.0} % face on: at least {:.0} % (a darker one is a dielectric)",
            l * 100.0,
            METAL_LUMINANCE_LEAST * 100.0
        ));
    }
    if let Some(alpha) = number("alpha")
        && !(GLASS_ALPHA[0]..=GLASS_ALPHA[1]).contains(&alpha)
    {
        out.push(format!(
            "`alpha` {alpha}: {}-{} (glass), or none",
            GLASS_ALPHA[0], GLASS_ALPHA[1]
        ));
    }
    let coat = number("clearcoat");
    if let Some(c) = coat
        && c != 0.0
        && c != 1.0
    {
        out.push(format!(
            "`clearcoat` {c}: 0 or 1 (a surface is coated or it is not)"
        ));
    }
    let coated = coat.is_some_and(|c| c > 0.0);
    match number("clearcoatRoughness") {
        Some(r) if coated && !(0.0..=COAT_ROUGHNESS_MOST).contains(&r) => out.push(format!(
            "`clearcoatRoughness` {r}: at most {COAT_ROUGHNESS_MOST}"
        )),
        Some(_) if !coated => out.push("`clearcoatRoughness` without a `clearcoat`".into()),
        _ => {}
    }
    let sheen = entry.get("sheenColor");
    if let Some(colour) = sheen
        && colour.as_str().and_then(linear).is_none()
    {
        out.push(format!("`sheenColor` {colour}: a colour, `#RRGGBB`"));
    }
    match (sheen, number("sheenRoughness")) {
        (Some(_), Some(r)) if !(SHEEN_ROUGHNESS[0]..=SHEEN_ROUGHNESS[1]).contains(&r) => {
            out.push(format!(
                "`sheenRoughness` {r}: {}-{}",
                SHEEN_ROUGHNESS[0], SHEEN_ROUGHNESS[1]
            ))
        }
        (Some(_), None) => out.push("`sheenColor` without a `sheenRoughness`".into()),
        (None, Some(_)) => out.push("`sheenRoughness` without a `sheenColor`".into()),
        _ => {}
    }
    if sheen.is_some() && metal {
        out.push("a sheen on a metal: sheen is cloth's".into());
    }
    out
}

/// What is wrong with a material's own `roughness` and `metallic` over its plain finish (a model entry's
/// `materials`: polished marble over `gloss`), given the finish's entry.
pub fn own_faults(finish: &J, own: &J) -> Vec<String> {
    let number = |name: &str, default: f64| {
        own.get(name)
            .or_else(|| finish.get(name))
            .and_then(J::as_f64)
            .unwrap_or(default)
    };
    match metal_of(number("metallic", 0.0)) {
        Ok(metal) => plain_fault(number("roughness", 0.8), metal)
            .into_iter()
            .collect(),
        Err(e) => vec![e],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classes_and_the_gaps_between() {
        for (roughness, metal, class) in [
            (0.1, false, Some("gloss")),
            (0.25, false, Some("gloss")),
            (0.28, false, None),
            (0.42, false, Some("satin")),
            (0.6, false, None),
            (0.75, false, Some("matte")),
            (1.0, false, Some("matte")),
            (1.2, false, None),
            (0.1, true, Some("chrome")),
            (0.2, true, None),
            (0.32, true, Some("brushed")),
            (0.65, true, Some("cast")),
            (0.9, true, None),
        ] {
            assert_eq!(super::class(roughness, metal), class, "{roughness} {metal}");
        }
    }

    #[test]
    fn a_finish_s_faults() {
        let faults = |e: J| finish_faults(&e, None);
        assert!(faults(json!({ "color": "#FFFFFF", "roughness": 0.42 })).is_empty());
        assert!(faults(json!({ "color": "#FFFFFF", "roughness": 0.1, "metallic": 1 })).is_empty());
        assert!(faults(json!({ "color": "#DCE6E8", "roughness": 0.05, "alpha": 0.28 })).is_empty());
        for (entry, fault) in [
            (json!({ "roughness": 0.28 }), "dielectric roughness 0.28"),
            (
                json!({ "roughness": 0.2, "metallic": 1 }),
                "metal roughness 0.2",
            ),
            (
                json!({ "roughness": 0.4, "metallic": 0.5 }),
                "`metallic` 0.5",
            ),
            (
                json!({ "color": "#3A3A3A", "roughness": 0.65, "metallic": 1 }),
                "a metal reflecting 4 %",
            ),
            (json!({ "roughness": 0.05, "alpha": 0.8 }), "`alpha` 0.8"),
            (
                json!({ "roughness": 0.4, "ior": 1.33 }),
                "`ior` is not taken",
            ),
            (
                json!({ "roughness": 0.4, "specular": 0.5 }),
                "`specular` is not taken",
            ),
            (
                json!({ "roughness": 0.05, "transmission": 1 }),
                "`transmission` is not taken",
            ),
            (
                json!({ "roughness": 0.4, "clearcoat": 0.5 }),
                "`clearcoat` 0.5",
            ),
            (
                json!({ "roughness": 0.4, "clearcoat": 1, "clearcoatRoughness": 0.5 }),
                "`clearcoatRoughness` 0.5",
            ),
            (
                json!({ "roughness": 0.4, "clearcoatRoughness": 0.1 }),
                "without a `clearcoat`",
            ),
            (
                json!({ "roughness": 0.8, "sheenColor": "#FFFFFF", "sheenRoughness": 0.1 }),
                "`sheenRoughness` 0.1",
            ),
            (
                json!({ "roughness": 0.8, "sheenColor": "white", "sheenRoughness": 0.5 }),
                "`sheenColor`",
            ),
            (
                json!({ "roughness": 0.8, "sheenRoughness": 0.5 }),
                "without a `sheenColor`",
            ),
            (
                json!({ "roughness": 0.4, "metallic": 1, "sheenColor": "#FFFFFF", "sheenRoughness": 0.5 }),
                "a sheen on a metal",
            ),
        ] {
            let found = faults(entry.clone());
            assert!(
                found.len() == 1 && found[0].contains(fault),
                "{entry}: {found:?}"
            );
        }
    }

    #[test]
    fn a_textured_finish_is_its_map_s() {
        let wood = json!({ "texture": "textures/wood_oak/albedo.jpg", "arm": "arm.jpg", "roughness": 0.8, "metallic": 0 });
        assert!(finish_faults(&wood, Some([0.545, 0.01])).is_empty());
        // Between satin and matte is a texture's to be; polished is not.
        assert!(finish_faults(&wood, Some([0.75, 0.01])).is_empty());
        assert!(finish_faults(&wood, Some([0.3, 0.01]))[0].contains("roughness 0.24"));
        // Without its own `metallic` the map's blue is the metal.
        let bare = json!({ "texture": "t.jpg", "arm": "arm.jpg" });
        assert!(finish_faults(&bare, Some([0.9, 0.0])).is_empty());
        assert!(finish_faults(&bare, Some([0.9, 0.4]))[0].contains("40 % metal"));
    }

    #[test]
    fn a_material_s_own_over_its_finish() {
        let gloss = json!({ "roughness": 0.12 });
        assert!(own_faults(&gloss, &json!({ "roughness": 0.2 })).is_empty());
        assert!(own_faults(&gloss, &json!({ "roughness": 0.28 }))[0].contains("0.28"));
        // The finish's roughness as a metal's: 0.12 is chrome.
        assert!(own_faults(&gloss, &json!({ "metallic": 1 })).is_empty());
        assert!(
            own_faults(&json!({ "roughness": 0.9 }), &json!({ "metallic": 1 }))[0]
                .contains("metal roughness")
        );
        assert!(own_faults(&gloss, &json!({ "metallic": 0.3 }))[0].contains("0 or 1"));
    }

    #[test]
    fn colours_from_hex() {
        assert_eq!(linear("#FFFFFF"), Some([1.0; 3]));
        assert_eq!(linear("#000000"), Some([0.0; 3]));
        assert!((linear("#808080").unwrap()[0] - 0.2159).abs() < 1e-3);
        assert_eq!(linear("FFFFFF"), None);
        assert_eq!(linear("#FFF"), None);
        assert_eq!(linear("#GGGGGG"), None);
    }
}
