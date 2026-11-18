//! The sets against the standard of a physically based surface, as the game loads them: the committed maps
//! (`cargo tex synth` makes them byte for byte) under the manifest's materials that draw them, with each
//! material's roughness, metalness and normal strength. Each rule says why it holds. A set or material that
//! breaks one is listed in [`KNOWN`] with its numbers: the test fails on a new break, and on a known one
//! that is gone (drop its entry), so the list stays the true one.

use super::{SETS, Set, set_of_texture};
use crate::colours;
use crate::content;
use crate::tex::pil::Img;
use serde_json::Value;
use std::sync::OnceLock;

/// The breaks the sets have today: (rule, set or material, what is wrong). Findings, not fixes: the textures
/// are the art streams' to change.
const KNOWN: &[(&str, &str, &str)] = &[
    (
        "albedo",
        "bark",
        "fissures painted near black: 13 % of texels under sRGB 30, the darkest 1 % at 16 (the shade is the AO's)",
    ),
    (
        "neutral",
        "floor_tile",
        "travertine cream, a mean cast of 18 levels: blue and green tiles come out grey",
    ),
    (
        "roughness",
        "material.roof",
        "mean 0.57: shingles, asphalt (mineral granules) or weathered wood, are 0.7 and up",
    ),
    (
        "normal steep",
        "material.wall.stone",
        "18 % of texels over 45°: the deep joints between thin stones",
    ),
    (
        "normal steep",
        "material.finish.foliage",
        "18 % of texels over 45°: the leaves' edges as generated",
    ),
    (
        "normal flat",
        "material.trim",
        "steepest 1 % at 1.49° at strength 0.6: the painted wood's relief is at its storage's rounding",
    ),
    (
        "ao open",
        "road_asphalt",
        "most open 1 % at 0.84 (`ao_base` 0.8)",
    ),
    (
        "ao open",
        "sidewalk",
        "most open 1 % at 0.85 (`ao_base` 0.82)",
    ),
    (
        "density",
        "material.road",
        "arm 102 px/m: a 512 px arm map over the 5 m tile",
    ),
];

/// A set's committed maps.
pub struct Maps {
    pub albedo: Img,
    pub normal: Img,
    pub arm: Img,
}

/// A manifest material that draws a set, with the scalars the renderer applies to its maps.
pub struct Material {
    pub key: String,
    pub set: &'static Set,
    pub uv_scale: f64,
    /// Multiplies the arm map's roughness (default 1 with an arm map).
    pub roughness: f64,
    /// Multiplies the arm map's metalness (default 1 with an arm map).
    pub metallic: f64,
    /// The normal map's strength (PlayCanvas `bumpiness`, default 1).
    pub normal_strength: f64,
}

struct Loaded {
    sets: Vec<(&'static Set, Maps)>,
    materials: Vec<Material>,
    /// Textured materials that draw no set (and aren't cut out by their texture's alpha, as leaf cards are).
    strays: Vec<String>,
}

fn loaded() -> &'static Loaded {
    static LOADED: OnceLock<Loaded> = OnceLock::new();
    LOADED.get_or_init(|| {
        let assets = content::root().join("web/public/assets");
        let open = |set: &Set, map: &str| {
            let path = assets.join(set.dir()).join(map);
            Img::open(&path).unwrap_or_else(|e| panic!("{e}"))
        };
        let sets = SETS
            .iter()
            .map(|set| {
                let maps = Maps {
                    albedo: open(set, "albedo.jpg").rgb(),
                    normal: open(set, "normal.jpg").rgb(),
                    arm: open(set, "arm.jpg").rgb(),
                };
                (set, maps)
            })
            .collect();
        let text = std::fs::read_to_string(assets.join("manifest.json")).expect("manifest.json");
        let manifest: Value = serde_json::from_str(&text).expect("manifest.json parses");
        let mut materials = Vec::new();
        let mut strays = Vec::new();
        for (key, e) in manifest["entries"].as_object().expect("entries") {
            let Some(texture) = e.get("texture").and_then(Value::as_str) else {
                continue;
            };
            if e.get("type").and_then(Value::as_str) != Some("material")
                || e.get("cutout").is_some()
            {
                continue;
            }
            let Some(set) = set_of_texture(texture) else {
                strays.push(key.clone());
                continue;
            };
            let number = |name: &str| e.get(name).and_then(Value::as_f64);
            materials.push(Material {
                key: key.clone(),
                set,
                uv_scale: number("uvScale").unwrap_or(1.0),
                roughness: number("roughness").unwrap_or(1.0),
                metallic: number("metallic").unwrap_or(1.0),
                normal_strength: number("normalStrength").unwrap_or(1.0),
            });
        }
        Loaded {
            sets,
            materials,
            strays,
        }
    })
}

/// The manifest's materials that draw a set.
pub fn materials() -> &'static [Material] {
    &loaded().materials
}

fn maps(set: &Set) -> &'static Maps {
    &loaded()
        .sets
        .iter()
        .find(|(s, _)| s.name == set.name)
        .expect("every set is loaded")
        .1
}

/// How the game colours a set.
#[derive(Clone, Copy, PartialEq)]
enum Tint {
    /// A grey detail map: the part's or covering's colour is the colour, the map multiplies it.
    Grey,
    /// A coloured map that coverings tint to colours of their own (floor tiles: blue, green, terracotta, slate).
    Covering,
    /// Its own colour: a tint only darkens or stains it (smoked brick, honey pine, walnut boards).
    Own,
}

fn tint(set: &Set) -> Tint {
    match set.name {
        _ if set.grey.is_some() => Tint::Grey,
        "floor_tile" => Tint::Covering,
        _ => Tint::Own,
    }
}

/// The range of mean roughness (as the renderer has it: the map's times the material's) a kind of surface has.
fn roughness_class(set: &Set) -> (&'static str, [f64; 2]) {
    // Porous or fibrous and unsealed (masonry, concrete, asphalt, plaster, raw or weathered wood or
    // shingle, cloth, soil): relief far coarser than light's wavelength, so no sheen.
    const ROUGH: (&str, [f64; 2]) = ("rough", [0.7, 1.0]);
    // A living leaf's waxy cuticle: a soft sheen.
    const LEAF: (&str, [f64; 2]) = ("leaf", [0.45, 0.85]);
    // Paint, varnish, a laminate's wear layer, finished leather: a satin to semi-gloss film.
    const COATED: (&str, [f64; 2]) = ("coated", [0.25, 0.75]);
    // Honed or glazed stone and ceramic: ground flat.
    const POLISHED: (&str, [f64; 2]) = ("polished", [0.1, 0.6]);
    match set.name {
        "foliage" | "grass_lush" => LEAF,
        "floor_laminate" | "floor_wood" | "leather" | "painted_wood" | "siding_wood"
        | "wood_oak" => COATED,
        "floor_tile" => POLISHED,
        _ => ROUGH,
    }
}

/// sRGB levels a non-metal's albedo keeps within: 30 is about 1 % reflectance, darker than charcoal; 240 is
/// 87 %, fresh snow. A texel outside is shade or glare painted into the colour.
const ALBEDO_LEVELS: [f64; 2] = [30.0, 240.0];
/// The share of texels that must keep to [`ALBEDO_LEVELS`].
const ALBEDO_SHARE: f64 = 0.99;
/// A grey detail map multiplies its tint: under half (linear) on average, the colour its author picked shows
/// over a stop darker.
const MULTIPLIER_MEAN: f64 = 0.5;
/// A tinted map's mean cast (sRGB levels, the largest channel less the smallest): a quarter of the faintest
/// coloured tint's own (sand tiles, #E6DCCB: 27), so the tint is the colour that shows.
const CAST: f64 = 6.0;
/// None of the sets is metal: a texel over 5 % metal loses its diffuse colour and mirrors the sky. At most
/// this share may.
const METAL: (f64, f64) = (0.05, 0.005);
/// 8-bit storage keeps a unit normal within 1 % of its length; past 10 % the storage bent its direction too.
/// At least this share must keep within it.
const UNIT: (f64, f64) = (0.10, 0.95);
/// A tiling height's slopes sum to nothing, so its normals average flat; a mean slope over 0.02 (1.1°) tilts
/// the whole surface's light by about 2 % where it falls at 45°, as much as the eye tells apart.
const LEVEL: f64 = 0.02;
/// Under three 8-bit steps (1.35°) even the steepest 1 % of a normal map is its storage's rounding: it carries
/// nothing.
const FLAT_DEG: f64 = 1.5;
/// Relief steeper than 45° needs the self-shadowing and parallax a normal map hasn't: over a tenth of the
/// surface that steep reads as noise.
const STEEP: (f64, f64) = (45.0, 0.10);
/// Every tile has open ground no cavity occludes, whose ambient occlusion is 1: a lower ceiling (the most
/// open 1 % under this) dims the whole surface's ambient light, which is the lighting's to do.
const OPEN_AO: f64 = 0.9;
/// Texels a metre (a map's width times its `uvScale`, the UVs being metres): the play camera's nearest view
/// shows about 250 px a metre, so under 200 is blurred there, and over 1600 (six and more texels a pixel)
/// only costs download and memory.
const DENSITY: [f64; 2] = [200.0, 1600.0];

/// Linear luminance, sRGB decoded as the renderer's materials decode a texture (`colours.rs`).
fn linear([r, g, b]: [u8; 3]) -> f64 {
    let lin = |level: u8| (level as f64 / 255.0).powf(2.2);
    colours::luminance([lin(r), lin(g), lin(b)])
}

fn texels(img: &Img) -> impl Iterator<Item = [u8; 3]> + '_ {
    img.px.as_chunks::<3>().0.iter().copied()
}

/// sRGB-encoded luminance in levels.
fn luma([r, g, b]: [u8; 3]) -> f64 {
    colours::luminance([r as f64, g as f64, b as f64])
}

/// The value `p` (0..1) of the way up the sorted values.
fn percentile(mut v: Vec<f64>, p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * p).round() as usize]
}

fn share(img: &Img, f: impl Fn([u8; 3]) -> bool) -> f64 {
    texels(img).filter(|&t| f(t)).count() as f64 / (img.w * img.h) as f64
}

fn mean(img: &Img, f: impl Fn([u8; 3]) -> f64) -> f64 {
    texels(img).map(f).sum::<f64>() / (img.w * img.h) as f64
}

/// A normal map's texel as a vector (the renderer's `2 c - 1`).
fn normal([r, g, b]: [u8; 3]) -> [f64; 3] {
    [r, g, b].map(|c| c as f64 / 255.0 * 2.0 - 1.0)
}

/// A texel's tilt from the surface's normal in degrees, at `strength` (the renderer blends the map's normal
/// with the flat one by it, then normalises).
fn tilt(t: [u8; 3], strength: f64) -> f64 {
    let [x, y, z] = normal(t);
    let n = [x * strength, y * strength, 1.0 - strength + z * strength];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    (n[2] / len).clamp(-1.0, 1.0).acos().to_degrees()
}

/// Fails on a break [`KNOWN`] doesn't list for `rule`, and on a listed one that is gone.
fn verdict(rule: &str, breaks: Vec<(String, String)>) {
    let known: Vec<&str> = KNOWN.iter().filter(|k| k.0 == rule).map(|k| k.1).collect();
    let new: Vec<String> = breaks
        .iter()
        .filter(|(s, _)| !known.contains(&s.as_str()))
        .map(|(s, d)| format!("{s}: {d}"))
        .collect();
    let gone: Vec<&str> = known
        .iter()
        .copied()
        .filter(|k| !breaks.iter().any(|(s, _)| s == k))
        .collect();
    assert!(
        new.is_empty() && gone.is_empty(),
        "{rule}: breaks not in KNOWN: {new:#?}\nin KNOWN, no longer broken (drop them): {gone:?}"
    );
}

fn per_set(rule: &str, f: impl Fn(&Set, &Maps) -> Option<String>) {
    let breaks = loaded()
        .sets
        .iter()
        .filter_map(|(set, m)| f(set, m).map(|d| (set.name.to_string(), d)))
        .collect();
    verdict(rule, breaks);
}

fn per_material(rule: &str, f: impl Fn(&Material, &Maps) -> Option<String>) {
    let breaks = materials()
        .iter()
        .filter_map(|m| f(m, maps(m.set)).map(|d| (m.key.clone(), d)))
        .collect();
    verdict(rule, breaks);
}

#[test]
fn every_set_is_drawn_and_every_textured_material_is_a_set() {
    for set in SETS {
        assert!(
            materials().iter().any(|m| m.set.name == set.name),
            "{}: no material in the manifest draws it",
            set.name
        );
    }
    assert!(
        loaded().strays.is_empty(),
        "textured materials no set makes: {:?}",
        loaded().strays
    );
}

#[test]
fn a_coloured_albedo_keeps_to_what_real_surfaces_reflect() {
    per_set("albedo", |set, m| {
        if tint(set) == Tint::Grey {
            return None;
        }
        let inside = share(&m.albedo, |t| {
            (ALBEDO_LEVELS[0]..=ALBEDO_LEVELS[1]).contains(&luma(t))
        });
        let mean = mean(&m.albedo, linear);
        let [lo, hi] = colours::RANGE;
        (inside < ALBEDO_SHARE || !(lo..=hi).contains(&mean)).then(|| {
            format!(
                "{:.1} % of texels in sRGB {}–{}, mean {mean:.3} linear (darkest 1 %: {:.0})",
                inside * 100.0,
                ALBEDO_LEVELS[0],
                ALBEDO_LEVELS[1],
                percentile(texels(&m.albedo).map(luma).collect(), 0.01)
            )
        })
    });
}

#[test]
fn a_detail_map_keeps_its_tint() {
    per_set("multiplier", |set, m| {
        if tint(set) != Tint::Grey {
            return None;
        }
        let mean = mean(&m.albedo, linear);
        let dark = percentile(texels(&m.albedo).map(luma).collect(), 0.01);
        (mean < MULTIPLIER_MEAN || dark < ALBEDO_LEVELS[0])
            .then(|| format!("mean {mean:.3} linear, darkest 1 % sRGB {dark:.0}"))
    });
}

#[test]
fn a_tinted_map_is_neutral() {
    per_set("neutral", |set, m| {
        if tint(set) == Tint::Own {
            return None;
        }
        let cast = mean(&m.albedo, |[r, g, b]| {
            (r.max(g).max(b) - r.min(g).min(b)) as f64
        });
        (cast > CAST).then(|| format!("mean cast {cast:.1} sRGB levels"))
    });
}

#[test]
fn roughness_is_its_kind_s() {
    per_material("roughness", |mat, m| {
        let (class, [lo, hi]) = roughness_class(mat.set);
        let mean = mean(&m.arm, |[_, g, _]| g as f64 / 255.0 * mat.roughness);
        (!(lo..=hi).contains(&mean)).then(|| format!("mean {mean:.3}, {class} surfaces {lo}–{hi}"))
    });
}

#[test]
fn nothing_is_metal() {
    per_material("metal", |mat, m| {
        let metal = share(&m.arm, |[_, _, b]| {
            b as f64 / 255.0 * mat.metallic > METAL.0
        });
        (metal > METAL.1).then(|| {
            format!(
                "{:.1} % of texels over {:.0} % metal (most {:.0} %)",
                metal * 100.0,
                METAL.0 * 100.0,
                texels(&m.arm).map(|t| t[2]).max().unwrap_or(0) as f64 / 2.55 * mat.metallic
            )
        })
    });
}

#[test]
fn normals_face_out_of_the_surface() {
    per_set("normal faces out", |_, m| {
        let inward = share(&m.normal, |[_, _, b]| b < 128);
        (inward > 0.001).then(|| format!("{:.3} % of texels point in", inward * 100.0))
    });
}

#[test]
fn normals_are_unit_length() {
    per_set("normal unit", |_, m| {
        let unit = share(&m.normal, |t| {
            let [x, y, z] = normal(t);
            ((x * x + y * y + z * z).sqrt() - 1.0).abs() <= UNIT.0
        });
        (unit < UNIT.1).then(|| {
            format!(
                "{:.1} % of texels within {:.0} % of unit length",
                unit * 100.0,
                UNIT.0 * 100.0
            )
        })
    });
}

#[test]
fn normals_average_level() {
    per_set("normal level", |_, m| {
        let slopes: Vec<[f64; 2]> = texels(&m.normal)
            .map(normal)
            .filter(|n| n[2] > 0.05)
            .map(|[x, y, z]| [x / z, y / z])
            .collect();
        let n = slopes.len().max(1) as f64;
        let [sx, sy] = [0, 1].map(|i| slopes.iter().map(|s| s[i]).sum::<f64>() / n);
        (sx.abs() > LEVEL || sy.abs() > LEVEL).then(|| format!("mean slope ({sx:+.3}, {sy:+.3})"))
    });
}

#[test]
fn normals_carry_relief() {
    per_material("normal flat", |mat, m| {
        let p99 = percentile(
            texels(&m.normal)
                .map(|t| tilt(t, mat.normal_strength))
                .collect(),
            0.99,
        );
        (p99 < FLAT_DEG).then(|| {
            format!(
                "steepest 1 % tilted {p99:.2}° (strength {})",
                mat.normal_strength
            )
        })
    });
}

#[test]
fn normals_are_not_steeper_than_a_normal_map_can_show() {
    per_material("normal steep", |mat, m| {
        let steep = share(&m.normal, |t| tilt(t, mat.normal_strength) > STEEP.0);
        (steep > STEEP.1).then(|| {
            format!(
                "{:.1} % of texels tilted over {}° (strength {})",
                steep * 100.0,
                STEEP.0,
                mat.normal_strength
            )
        })
    });
}

#[test]
fn ambient_occlusion_opens_to_one() {
    per_set("ao open", |_, m| {
        let open = percentile(texels(&m.arm).map(|t| t[0] as f64 / 255.0).collect(), 0.99);
        (open < OPEN_AO).then(|| format!("most open 1 % at {open:.3}"))
    });
}

#[test]
fn texel_density_suits_the_play_camera() {
    per_material("density", |mat, m| {
        let out: Vec<String> = [
            ("albedo", &m.albedo),
            ("normal", &m.normal),
            ("arm", &m.arm),
        ]
        .iter()
        .map(|(map, img)| (map, img.w as f64 * mat.uv_scale))
        .filter(|(_, d)| !(DENSITY[0]..=DENSITY[1]).contains(d))
        .map(|(map, d)| format!("{map} {d:.0} px/m"))
        .collect();
        (!out.is_empty()).then(|| out.join(", "))
    });
}
