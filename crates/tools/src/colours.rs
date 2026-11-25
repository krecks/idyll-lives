//! `cargo review colours`: every model's colours against what real surfaces reflect.
//!
//! A surface that isn't metal reflects between about 2 % (soot, black velvet) and 80 % (fresh snow is 90 %,
//! white paint about 80 %) of the light that falls on it. A colour outside that range is one no lighting was
//! tuned for: a white at 0.9 clips in the midday sun and reads as glare beside a wall at 0.6, a black at 0.01
//! is a hole that no lamp lifts. This lists them: each model's materials as the game draws them (the file's
//! colour, times its finish's texture where it has one), by their luminance in linear light.
//!
//! A metal's colour is what it reflects face on, and no metal in use reflects less than 45 % (`pbr`): one
//! darker is listed too, as a dark metal (black iron is an oxide: a dark satin, not a metal).
//!
//! Left out: see-through finishes, materials that glow, and models the manifests don't name. A report, not
//! a gate: `--check` makes it one once the palette is in range.
//!
//! ```text
//! cargo review colours [--all] [--check]
//! ```

use crate::checkglb::glb::parse_glb;
use crate::checkglb::json::loads_bytes;
use crate::checkglb::override_of;
use crate::checkglb::py::{J, Num};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The share of light a surface that isn't metal reflects, at least and at most (linear luminance).
pub const RANGE: [f64; 2] = [0.02, 0.80];
/// The least a metal reflects face on.
pub const METAL_LEAST: f64 = crate::pbr::METAL_LUMINANCE_LEAST;

/// sRGB-encoded 0..1 as the game's materials decode a texture (`pow(c, 2.2)`).
fn decode(v: f64) -> f64 {
    v.powf(2.2)
}

pub fn luminance(rgb: [f64; 3]) -> f64 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

/// What a finish does to a colour.
#[derive(Clone, Debug, PartialEq)]
pub enum Finish {
    /// Drawn as the colour is.
    Plain,
    /// The colour times a tiling texture, of this mean (linear red, green, blue).
    Textured([f64; 3]),
    /// A metal: the colour is what it reflects.
    Metal,
    /// Left out: see-through, or cut out by its texture's alpha.
    Skipped,
}

/// A manifest's finishes (`material.finish.*`), by key.
pub fn finishes(manifest: &J, dir: &Path) -> BTreeMap<String, Finish> {
    let mut out = BTreeMap::new();
    let Some(entries) = manifest.get("entries").and_then(J::entries) else {
        return out;
    };
    for (key, e) in entries
        .iter()
        .filter(|(k, _)| k.starts_with("material.finish."))
    {
        let number =
            |name: &str, default: f64| e.get(name).and_then(J::num).map_or(default, Num::f);
        let finish = if number("alpha", 1.0) < 1.0 || e.get("cutout").is_some() {
            Finish::Skipped
        } else if number("metallic", 0.0) >= 0.5 {
            Finish::Metal
        } else if let Some(texture) = e.get("texture").and_then(J::str) {
            match image::open(dir.join(texture)) {
                Ok(image) => {
                    let image = image.to_rgb8();
                    let mut sum = [0f64; 3];
                    for p in image.pixels() {
                        for (sum, v) in sum.iter_mut().zip(p.0) {
                            *sum += decode(v as f64 / 255.0);
                        }
                    }
                    let n = (image.width() * image.height()).max(1) as f64;
                    Finish::Textured([sum[0] / n, sum[1] / n, sum[2] / n])
                }
                Err(_) => Finish::Plain,
            }
        } else {
            Finish::Plain
        };
        out.insert(key.clone(), finish);
    }
    out
}

/// One material of one model out of range.
#[derive(Debug, Clone)]
pub struct Found {
    pub key: String,
    pub family: String,
    pub material: String,
    pub finish: String,
    pub luminance: f64,
    /// A metal under `METAL_LEAST` (the others are out of `RANGE`).
    pub metal: bool,
}

/// A model's materials out of range: how many that aren't metal were looked at, how many metals, and those
/// found. `materials` is its manifest entry's map, `gltf` its file's JSON.
pub fn of_model(
    key: &str,
    family: &str,
    materials: Option<&J>,
    gltf: &J,
    finishes: &BTreeMap<String, Finish>,
) -> ([usize; 2], Vec<Found>) {
    let mut checked = [0; 2];
    let mut found = Vec::new();
    for m in gltf.get("materials").and_then(J::list).unwrap_or(&[]) {
        let name = m.get("name").and_then(J::str).unwrap_or("");
        // As the game maps it: its own entry, else the finish in its name, else the `*` one.
        let mapped = override_of(materials.unwrap_or(&J::Null), name);
        let mapped = Some(&mapped).filter(|o| **o != J::Null);
        let finish_key = mapped
            .and_then(|o| o.get("finish"))
            .and_then(J::str)
            .map(String::from);
        let pbr = m.get("pbrMetallicRoughness");
        let number =
            |o: Option<&J>, name: &str| o.and_then(|o| o.get(name)).and_then(J::num).map(Num::f);
        let finish = match &finish_key {
            Some(k) => finishes.get(k).cloned().unwrap_or(Finish::Plain),
            None => Finish::Plain,
        };
        // A colour's own metal (the palette's `metallic`), else its finish's, or the file's where nothing maps
        // the material.
        let metal = number(mapped, "metallic").unwrap_or(if finish_key.is_some() {
            f64::from(finish == Finish::Metal)
        } else {
            number(pbr, "metallicFactor").unwrap_or(1.0)
        }) >= 0.5;
        let glows = mapped.is_some_and(|o| o.get("glow").is_some())
            || m.get("emissiveFactor")
                .and_then(J::list)
                .is_some_and(|e| e.iter().any(|v| v.num().map_or(0.0, Num::f) > 0.0));
        if glows
            || finish == Finish::Skipped
            || m.get("alphaMode").and_then(J::str) == Some("BLEND")
        {
            continue;
        }
        let base: Vec<f64> = pbr
            .and_then(|p| p.get("baseColorFactor"))
            .and_then(J::list)
            .map(|a| a.iter().map(|v| v.num().map_or(1.0, Num::f)).collect())
            .unwrap_or_else(|| vec![1.0; 4]);
        let tint = match finish {
            Finish::Textured(mean) => mean,
            _ => [1.0; 3],
        };
        let l = luminance([base[0] * tint[0], base[1] * tint[1], base[2] * tint[2]]);
        checked[usize::from(metal)] += 1;
        if if metal {
            l < METAL_LEAST
        } else {
            l < RANGE[0] || l > RANGE[1]
        } {
            found.push(Found {
                key: key.to_string(),
                family: family.to_string(),
                material: name.to_string(),
                finish: finish_key.map_or("unmapped".into(), |k| {
                    k.trim_start_matches("material.finish.").to_string()
                }),
                luminance: l,
                metal,
            });
        }
    }
    (checked, found)
}

/// Every `manifest.json` under `dir`, in order.
pub(crate) fn manifests(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            manifests(&path, out);
        } else if path.file_name().is_some_and(|n| n == "manifest.json") {
            out.push(path);
        }
    }
}

/// Every named model's materials: how many models, how many colours were looked at (those that aren't
/// metal, then the metals), and those out of range.
pub fn survey(assets: &Path) -> Result<(usize, [usize; 2], Vec<Found>), String> {
    let read = |path: &Path| -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
    };
    let json = |path: &Path| -> Result<J, String> {
        loads_bytes(&read(path)?).map_err(|e| format!("{}: {e}", path.display()))
    };
    let finishes = finishes(&json(&assets.join("manifest.json"))?, assets);
    let mut files = Vec::new();
    manifests(assets, &mut files);
    let (mut models, mut checked, mut found) = (0, [0; 2], Vec::new());
    for file in files {
        let manifest = json(&file)?;
        let dir = file.parent().unwrap_or(assets);
        let Some(entries) = manifest.get("entries").and_then(J::entries) else {
            continue;
        };
        for (key, entry) in entries {
            let Some(url) = entry
                .get("url")
                .and_then(J::str)
                .filter(|_| entry.get("type").and_then(J::str) == Some("model"))
            else {
                continue;
            };
            let glb = dir.join(url);
            let family = glb
                .parent()
                .and_then(|p| p.file_name())
                .map_or(String::new(), |n| n.to_string_lossy().into_owned());
            let (n, mut out) = of_model(
                key,
                &family,
                entry.get("materials"),
                &parse_glb(&read(&glb)?)
                    .map_err(|e| format!("{}: {e}", glb.display()))?
                    .gltf,
                &finishes,
            );
            models += 1;
            checked[0] += n[0];
            checked[1] += n[1];
            found.append(&mut out);
        }
    }
    Ok((models, checked, found))
}

/// `cargo review colours [--all] [--check]`: the exit code (1 with `--check` when a colour is out of range).
pub fn main(args: &[String]) -> i32 {
    let (all, check) = (
        args.iter().any(|a| a == "--all"),
        args.iter().any(|a| a == "--check"),
    );
    let (models, checked, mut found) = match survey(&crate::repo_root().join("web/public/assets")) {
        Ok(survey) => survey,
        Err(e) => {
            eprintln!("review colours: {e}");
            return 1;
        }
    };
    // What a finding is: too light, too dark, or a dark metal.
    let kind = |f: &Found| {
        if f.metal {
            2
        } else {
            usize::from(f.luminance < RANGE[0])
        }
    };
    let mut families: BTreeMap<&str, [usize; 3]> = BTreeMap::new();
    let mut totals = [0; 3];
    for f in &found {
        families.entry(&f.family).or_default()[kind(f)] += 1;
        totals[kind(f)] += 1;
    }
    println!(
        "{models} models, {} colours that aren't metal, glass or glowing: {} reflect more than {:.0} %, {} less than {:.0} %; {} metals: {} reflect less than {:.0} %\n",
        checked[0],
        totals[0],
        RANGE[1] * 100.0,
        totals[1],
        RANGE[0] * 100.0,
        checked[1],
        totals[2],
        METAL_LEAST * 100.0
    );
    println!("| family | too light | too dark | dark metal |\n|---|---|---|---|");
    for (family, [light, dark, metal]) in &families {
        println!("| {family} | {light} | {dark} | {metal} |");
    }
    found.sort_by(|a, b| b.luminance.total_cmp(&a.luminance));
    let show = |title: &str, rows: &[&Found]| {
        println!("\n{title}\n\n| model | material | finish | reflects |\n|---|---|---|---|");
        for f in rows {
            println!(
                "| {} | {} | {} | {:.1} % |",
                f.key,
                f.material,
                f.finish,
                f.luminance * 100.0
            );
        }
    };
    let lights: Vec<&Found> = found.iter().filter(|f| kind(f) == 0).collect();
    let darks: Vec<&Found> = found.iter().rev().filter(|f| kind(f) == 1).collect();
    let metals: Vec<&Found> = found.iter().rev().filter(|f| kind(f) == 2).collect();
    let n = if all { usize::MAX } else { 15 };
    show(
        &format!(
            "The lightest{}:",
            if all {
                ""
            } else {
                " (15; `--all` for every one)"
            }
        ),
        &lights[..lights.len().min(n)],
    );
    show("The darkest:", &darks[..darks.len().min(n)]);
    show(
        "The darkest metals (a metal's colour is what it reflects; darker, it is a dielectric):",
        &metals[..metals.len().min(n)],
    );
    i32::from(check && !found.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn j(v: serde_json::Value) -> J {
        loads_bytes(v.to_string().as_bytes()).unwrap()
    }

    #[test]
    fn a_model_s_colours_in_and_out_of_range() {
        let mut finishes = BTreeMap::new();
        finishes.insert("material.finish.matte".to_string(), Finish::Plain);
        finishes.insert("material.finish.metal".to_string(), Finish::Metal);
        finishes.insert(
            "material.finish.wood".to_string(),
            Finish::Textured([0.5, 0.5, 0.5]),
        );
        let colour = |name: &str, v: f64| json!({ "name": name, "pbrMetallicRoughness": { "baseColorFactor": [v, v, v, 1.0] } });
        let gltf = json!({ "materials": [
            colour("white", 0.9), colour("grey", 0.5), colour("soot", 0.01), colour("steel", 0.95),
            colour("pale@wood", 0.9), colour("lamp", 0.95), colour("brass", 0.9),
            { "name": "raw", "pbrMetallicRoughness": { "baseColorFactor": [0.9, 0.9, 0.9, 1.0], "metallicFactor": 0.0 } },
        ] });
        let materials = json!({
            "white": { "finish": "material.finish.matte" }, "grey": { "finish": "material.finish.matte" },
            "soot": { "finish": "material.finish.matte" }, "steel": { "finish": "material.finish.metal" },
            "lamp": { "finish": "material.finish.matte", "glow": "#FFD9A0" },
            "brass": { "finish": "material.finish.matte", "metallic": 1.0 },
        });
        let (checked, found) = of_model(
            "model.x",
            "kitchen",
            Some(&j(materials)),
            &j(gltf),
            &finishes,
        );
        // (The lamp is left out; steel and brass are metals; the wood's 0.9 is 0.45 under its texture.)
        assert_eq!(checked, [5, 2]);
        let names: Vec<&str> = found.iter().map(|f| f.material.as_str()).collect();
        assert_eq!(names, ["white", "soot", "raw"]);
        assert_eq!(found[2].finish, "unmapped");
        assert!((found[0].luminance - 0.9).abs() < 1e-9);
        assert!(found.iter().all(|f| !f.metal));
    }

    #[test]
    fn a_dark_metal_is_found() {
        let mut finishes = BTreeMap::new();
        finishes.insert("material.finish.matte".to_string(), Finish::Plain);
        finishes.insert("material.finish.metal".to_string(), Finish::Metal);
        let colour = |name: &str, v: f64| json!({ "name": name, "pbrMetallicRoughness": { "baseColorFactor": [v, v, v, 1.0] } });
        let gltf = json!({ "materials": [
            colour("steel", 0.6), colour("iron", 0.05), colour("gunmetal", 0.2), colour("painted", 0.05),
            colour("grate", 0.44),
        ] });
        let materials = json!({
            "steel": { "finish": "material.finish.metal" }, "iron": { "finish": "material.finish.metal" },
            "gunmetal": { "finish": "material.finish.matte", "metallic": 1 },
            "painted": { "finish": "material.finish.metal", "metallic": 0 },
            "grate": { "finish": "material.finish.metal" },
        });
        let (checked, found) = of_model(
            "model.x",
            "kitchen",
            Some(&j(materials)),
            &j(gltf),
            &finishes,
        );
        // (Painted over, the metal finish's colour is a dielectric's: 5 % is in range.)
        assert_eq!(checked, [1, 4]);
        let names: Vec<&str> = found.iter().map(|f| f.material.as_str()).collect();
        assert_eq!(names, ["iron", "gunmetal", "grate"]);
        assert!(found.iter().all(|f| f.metal));
    }

    #[test]
    fn the_shipped_models_are_read() {
        let (models, checked, found) =
            survey(&crate::repo_root().join("web/public/assets")).unwrap();
        assert!(models > 600, "{models} models");
        assert!(checked[0] > 2000 && checked[1] > 300, "{checked:?} colours");
        // (A report for now: how many are out of range is what the palette pass brings down.)
        assert!(found.len() < checked[0] + checked[1]);
    }
}
