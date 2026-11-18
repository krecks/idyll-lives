//! A plant's model against its species (docs/review/reference.json: a kind's `species`), measured from the
//! file as the gate reads it (`checkglb`). `cargo review sizes` runs it on every object whose kind has one and counts
//! its findings with the size's (`reference` in tools/art/gate-baseline.json).
//!
//! What it measures, at rest (the far version aside):
//!
//! - **H**: the plant's height, from the lowest point of its plant (leaves, cards, bark, blooms, fruit: not its
//!   pot, soil or stand) to the model's top; a tree on the ground is the whole model.
//! - **Crown width**: the widest of the leaves' (foliage and leaf cards) extents across and front to back.
//! - **Crown ratio**: crown width over H, and **clear trunk**: the leaves' lowest point over H.
//! - **Bark luminance**: the bark's linear luminance as drawn (its colour times its finish's texture, as `colours`
//!   takes it), by area: the materials on the `bark` finish or named for bark.
//! - **Blossom and fruit hue**: each saturated colour that isn't a leaf's green, on the leaves or on a part named
//!   for its bloom or fruit (`bloom`, `blossom`, `flower`, `petal`, `fruit`), must be its blossom's or its fruit's.
//! - **Leaf class**: each leaf card (the `leafcards` finish) lies on a tile of the atlas of its species' leaf
//!   shape (`LEAF_SHAPES`). The atlas's tiles are listed in web/public/assets/textures/leafcards/tiles.json:
//!   `{"tiles": [{"name": "oak", "class": "lobed", "rect": [u0, v0, u1, v1]}, …]}`, each rect in a model's UVs
//!   (u right, v down from the image's top), `core` the class of the dense tile under the cards. With no such file
//!   the atlas is today's (`Atlas::today`): three clusters of one oval leaf and the dense tile.
//!
//! Leaf size on screen, the bark's pattern and the habit's likeness stay a reviewer's judgement against the kind's
//! sources (crates/art/nature/src/trees/NOTES.md, "Species review").

use crate::checkglb::glb::{Glb, accessor, key, parse_glb};
use crate::checkglb::py::{J, Num};
use crate::checkglb::{self, applied, node_matrices, primitives};
use crate::colours::{self, Finish};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// The shape classes a species' leaf may have, and a leaf-card tile.
pub const LEAF_SHAPES: [&str; 13] = [
    "oval",
    "serrate-small",
    "lobed",
    "palmate",
    "lance",
    "obovate-large",
    "cordate",
    "linear",
    "pinnate",
    "split",
    "needle",
    "scale",
    "strap",
];
/// The classes of bark.
pub const BARK_CLASSES: [&str; 6] = [
    "smooth",
    "fissured",
    "peeling",
    "plated",
    "scaly",
    "papery-white",
];
/// The leaf-card atlas's tiles, from the checkout's root.
pub const TILES: &str = "web/public/assets/textures/leafcards/tiles.json";
/// A colour this saturated (HSV) or more has a hue to judge; a paler one is white or grey (a hosta's lilac is 0.24).
pub const SATURATED: f64 = 0.2;
/// The hues (degrees) a leaf's green spans: a colour outside them is a bloom's, a fruit's or an autumn leaf's.
pub const GREEN: [f64; 2] = [50.0, 170.0];

/// A kind's `species`: what the plant is and what a garden-age specimen of it looks like. Each range is
/// [low, high]; the kind's `h` and `w` are its height and crown width, strictly.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Species {
    pub latin: String,
    #[serde(default)]
    pub habit: String,
    /// Crown width over height.
    pub crown: Option<[f64; 2]>,
    /// The clear trunk under the crown, a share of the height.
    pub clear: Option<[f64; 2]>,
    pub leaf: Option<Leaf>,
    pub bark: Option<Bark>,
    pub blossom: Option<Bloom>,
    pub fruit: Option<Bloom>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Leaf {
    /// One of `LEAF_SHAPES`.
    pub shape: String,
    /// The blade's length (m; a needle's for a conifer).
    pub length: Option<[f64; 2]>,
    /// The hues (degrees, as `Bloom::hue`) its leaves take besides green: their autumn colour, a coloured
    /// cultivar's.
    #[serde(default)]
    pub hue: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bark {
    /// One of `BARK_CLASSES`.
    pub class: String,
    /// Its reflectance in visible light (linear luminance, 0 to 1).
    pub luminance: Option<[f64; 2]>,
}

/// A blossom's or a fruit's colours and when it shows.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bloom {
    pub colour: String,
    /// The hues (degrees, sRGB HSV) of the colours it comes in, one range each; a low end over the high one wraps
    /// through red (`[345, 10]`). None: white (no hue to judge).
    #[serde(default)]
    pub hue: Vec<[f64; 2]>,
    /// The first and last month it shows (1 to 12; northern temperate gardens).
    pub months: [u8; 2],
}

impl Species {
    /// What is wrong with the block itself: a shape or class not in the lists, a range upside down, a month that
    /// isn't one.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.latin.trim().is_empty() {
            out.push("no `latin` name".into());
        }
        let mut range = |name: &str, r: Option<[f64; 2]>, top: f64| {
            if let Some([lo, hi]) = r
                && !(0.0 <= lo && lo <= hi && hi <= top)
            {
                out.push(format!("{name} [{lo}, {hi}] isn't a range in 0..{top}"));
            }
        };
        range("crown", self.crown, 10.0);
        range("clear", self.clear, 1.0);
        range(
            "leaf length",
            self.leaf.as_ref().and_then(|l| l.length),
            2.0,
        );
        range(
            "bark luminance",
            self.bark.as_ref().and_then(|b| b.luminance),
            1.0,
        );
        if let Some(l) = &self.leaf
            && !LEAF_SHAPES.contains(&l.shape.as_str())
        {
            out.push(format!(
                "leaf shape `{}` isn't one of {}",
                l.shape,
                LEAF_SHAPES.join(", ")
            ));
        }
        if let Some(b) = &self.bark
            && !BARK_CLASSES.contains(&b.class.as_str())
        {
            out.push(format!(
                "bark class `{}` isn't one of {}",
                b.class,
                BARK_CLASSES.join(", ")
            ));
        }
        let hues = self
            .leaf
            .iter()
            .flat_map(|l| l.hue.iter().map(|h| ("leaf", h)));
        for (name, [lo, hi]) in hues {
            if !((0.0..360.0).contains(lo) && (0.0..360.0).contains(hi)) {
                out.push(format!("{name} hue [{lo}, {hi}] isn't in 0..360"));
            }
        }
        for (name, bloom) in [("blossom", &self.blossom), ("fruit", &self.fruit)] {
            let Some(b) = bloom else { continue };
            for [lo, hi] in &b.hue {
                if !((0.0..360.0).contains(lo) && (0.0..360.0).contains(hi)) {
                    out.push(format!("{name} hue [{lo}, {hi}] isn't in 0..360"));
                }
            }
            if !b.months.iter().all(|m| (1..=12).contains(m)) {
                out.push(format!("{name} months {:?} aren't months", b.months));
            }
        }
        out
    }
}

/// What a part of the plant is, by its finish and name.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    /// Leaves drawn as lumps or blades (`foliage`).
    Leaves,
    /// Leaf cards (`leafcards`).
    Cards,
    Bark,
    /// A part named for its bloom or its fruit (or a flower's centre).
    Bloom,
    /// Anything else: a pot, its soil, a stand, a stake.
    Other,
}

impl Role {
    fn of(name: &str, finish: &str) -> Role {
        let name = name.to_ascii_lowercase();
        let named = |words: &[&str]| words.iter().any(|w| name.contains(w));
        match finish {
            _ if named(&["bark", "trunk", "twig", "branch"]) => Role::Bark,
            "leafcards" => Role::Cards,
            "foliage" => Role::Leaves,
            "bark" => Role::Bark,
            _ if named(&["bloom", "blossom", "flower", "petal", "fruit", "centre"]) => Role::Bloom,
            _ => Role::Other,
        }
    }

    /// Whether it is the plant (not its pot, soil or stand).
    fn plant(self) -> bool {
        self != Role::Other
    }

    fn leaves(self) -> bool {
        matches!(self, Role::Leaves | Role::Cards)
    }
}

/// A material of the model: its name, finish (without `material.finish.`), role, colour (linear RGB, the file's
/// base colour) and luminance as drawn (times its finish's texture; None: a finish `colours` leaves out).
#[derive(Debug, Clone)]
pub struct Part {
    pub name: String,
    pub finish: String,
    pub role: Role,
    pub colour: [f64; 3],
    pub luminance: Option<f64>,
}

/// One triangle at rest (glTF axes, y up), its material (an index into `Plant::parts`) and its UVs.
#[derive(Debug, Clone)]
pub struct Tri {
    pub part: usize,
    pub p: [[f64; 3]; 3],
    pub uv: Option<[[f64; 2]; 3]>,
}

/// A plant's model as read.
#[derive(Debug, Clone, Default)]
pub struct Plant {
    pub parts: Vec<Part>,
    pub tris: Vec<Tri>,
}

/// A model file's material's finish: the entry's `materials` map (or its `*`), else its own name
/// (`leaf@foliage`).
fn finish_of(materials: Option<&J>, name: &str) -> String {
    materials
        .and_then(|m| m.get(name).or_else(|| m.get("*")))
        .and_then(|o| o.get("finish"))
        .and_then(J::str)
        .map(|f| f.trim_start_matches("material.finish.").to_string())
        .or_else(|| {
            let tail = name.rsplit_once('@')?.1;
            Some(tail.split('#').next().unwrap_or(tail).to_string())
        })
        .unwrap_or_default()
}

/// The plant in model file `g`, with its entry's `materials` map and the manifests' finishes (`colours::finishes`).
pub fn plant(
    g: &Glb,
    materials: Option<&J>,
    finishes: &BTreeMap<String, Finish>,
) -> Result<Plant, String> {
    let mut out = Plant::default();
    for m in g.gltf.get("materials").and_then(J::list).unwrap_or(&[]) {
        let name = m.get("name").and_then(J::str).unwrap_or("").to_string();
        let finish = finish_of(materials, &name);
        let base: Vec<f64> = m
            .get("pbrMetallicRoughness")
            .and_then(|p| p.get("baseColorFactor"))
            .and_then(J::list)
            .map(|l| l.iter().filter_map(|v| v.num().map(Num::f)).collect())
            .unwrap_or_default();
        let colour = match base[..] {
            [r, g, b, ..] => [r, g, b],
            _ => [1.0; 3],
        };
        let luminance = match finishes.get(&format!("material.finish.{finish}")) {
            // (A metal's colour is what it reflects, not a bark's or a petal's.)
            Some(Finish::Skipped | Finish::Metal) => None,
            Some(Finish::Textured(t)) => Some(colours::luminance([
                colour[0] * t[0],
                colour[1] * t[1],
                colour[2] * t[2],
            ])),
            Some(Finish::Plain) | None => Some(colours::luminance(colour)),
        };
        out.parts.push(Part {
            role: Role::of(&name, &finish),
            name,
            finish,
            colour,
            luminance,
        });
    }
    // A primitive with no material is drawn in the default one: a part of its own, not the plant.
    let default = out.parts.len();
    for p in node_matrices(&g.gltf, None, None)? {
        for prim in primitives(&g.gltf, &p.mesh)? {
            if prim
                .get("mode")
                .and_then(J::num)
                .is_some_and(|m| m.f() != 4.0)
            {
                continue;
            }
            let part = match prim.get("material").and_then(J::num) {
                Some(Num::I(i)) if (0..default as i64).contains(&i) => i as usize,
                _ => {
                    if out.parts.len() == default {
                        out.parts.push(Part {
                            name: String::new(),
                            finish: String::new(),
                            role: Role::Other,
                            colour: [1.0; 3],
                            luminance: None,
                        });
                    }
                    default
                }
            };
            let attributes = key(prim, "attributes")?;
            let Some(pos) = accessor(g, key(attributes, "POSITION")?)? else {
                return Err("its positions can't be read here (compressed)".into());
            };
            let world: Vec<[f64; 3]> = pos.iter().map(|v| applied(&p.m, v)).collect();
            let uvs: Option<Vec<[f64; 2]>> = match attributes.get("TEXCOORD_0") {
                Some(i) => accessor(g, i)?.map(|rows| {
                    rows.iter()
                        .map(|r| {
                            [
                                r.first().map_or(0.0, |v| v.f()),
                                r.get(1).map_or(0.0, |v| v.f()),
                            ]
                        })
                        .collect()
                }),
                None => None,
            };
            let idx: Vec<usize> = match prim.get("indices") {
                Some(i) => accessor(g, i)?
                    .ok_or("its indices can't be read here (compressed)")?
                    .iter()
                    .map(|r| r.first().map_or(usize::MAX, |v| v.f() as usize))
                    .collect(),
                None => (0..world.len()).collect(),
            };
            for t in idx.as_chunks::<3>().0 {
                let corner = |k: usize| world.get(t[k]).copied();
                let (Some(a), Some(b), Some(c)) = (corner(0), corner(1), corner(2)) else {
                    return Err("an index is past its positions".into());
                };
                let uv = uvs
                    .as_ref()
                    .and_then(|uv| Some([*uv.get(t[0])?, *uv.get(t[1])?, *uv.get(t[2])?]));
                out.tris.push(Tri {
                    part,
                    p: [a, b, c],
                    uv,
                });
            }
        }
    }
    Ok(out)
}

/// The plant in the model file at `path` (`plant`).
pub fn read(
    path: &str,
    materials: Option<&J>,
    finishes: &BTreeMap<String, Finish>,
) -> Result<Plant, String> {
    let data = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    plant(&parse_glb(&data)?, materials, finishes).map_err(|e| format!("{path}: {e}"))
}

fn area(p: &[[f64; 3]; 3]) -> f64 {
    let (a, b, c) = (p[0], p[1], p[2]);
    let (u, v) = (
        [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
    );
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    0.5 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()
}

/// An sRGB-encoded channel of a linear one.
fn encode(v: f64) -> f64 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// The hue (degrees) and saturation (HSV) of a linear colour as sRGB shows it.
pub fn hue(linear: [f64; 3]) -> (f64, f64) {
    let [r, g, b] = linear.map(encode);
    let (hi, lo) = (r.max(g).max(b), r.min(g).min(b));
    let d = hi - lo;
    let sat = if hi > 0.0 { d / hi } else { 0.0 };
    if d <= 0.0 {
        return (0.0, sat);
    }
    let h = if hi == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if hi == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    (h, sat)
}

/// Whether hue `h` is in `[lo, hi]`, through red when lo is over hi.
pub fn hue_in(h: f64, [lo, hi]: [f64; 2]) -> bool {
    if lo <= hi {
        (lo..=hi).contains(&h)
    } else {
        h >= lo || h <= hi
    }
}

/// A leaf-card tile of the atlas: its class (`LEAF_SHAPES`, or `core`) and where it is in a model's UVs.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Tile {
    #[serde(default)]
    pub name: String,
    pub class: String,
    pub rect: [f64; 4],
}

/// The leaf-card atlas's tiles.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Atlas {
    pub tiles: Vec<Tile>,
}

impl Atlas {
    /// Today's atlas (artkit's `botany::cards` `CARD_TILES`, `CORE_TILE`; v turned round as the file
    /// has it): three clusters of one generic oval leaf, a quarter each, and the dense tile.
    pub fn today() -> Atlas {
        let tile = |class: &str, u: f64, v: f64| Tile {
            name: String::new(),
            class: class.into(),
            rect: [u, v, u + 0.5, v + 0.5],
        };
        Atlas {
            tiles: vec![
                tile("oval", 0.0, 0.0),
                tile("oval", 0.5, 0.0),
                tile("oval", 0.0, 0.5),
                tile("core", 0.5, 0.5),
            ],
        }
    }

    pub fn parse(text: &str) -> Result<Atlas, String> {
        let atlas: Atlas = serde_json::from_str(text).map_err(|e| format!("{TILES}: {e}"))?;
        for t in &atlas.tiles {
            if t.class != "core" && !LEAF_SHAPES.contains(&t.class.as_str()) {
                return Err(format!(
                    "{TILES}: tile `{}`'s class `{}` isn't `core` or one of {}",
                    t.name,
                    t.class,
                    LEAF_SHAPES.join(", ")
                ));
            }
        }
        Ok(atlas)
    }

    /// The atlas under the checkout at `root`: its tiles.json, else today's.
    pub fn read(root: &str) -> Result<Atlas, String> {
        let path = format!("{root}/{TILES}");
        match std::fs::read_to_string(&path) {
            Ok(text) => Atlas::parse(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Atlas::today()),
            Err(e) => Err(format!("{TILES}: {e}")),
        }
    }

    /// The class of the tile a UV is on (the first that has it).
    fn class_at(&self, [u, v]: [f64; 2]) -> Option<&str> {
        self.tiles
            .iter()
            .find(|t| t.rect[0] <= u && u <= t.rect[2] && t.rect[1] <= v && v <= t.rect[3])
            .map(|t| t.class.as_str())
    }

    fn has(&self, class: &str) -> bool {
        self.tiles.iter().any(|t| t.class == class)
    }
}

/// A colour on the plant with a hue to judge: its material, hue (degrees), saturation, and whether it is on its
/// leaves.
#[derive(Debug, Clone, PartialEq)]
pub struct Colour {
    pub material: String,
    pub hue: f64,
    pub saturation: f64,
    pub leaf: bool,
}

/// A plant as measured (`measure`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Shape {
    /// The plant's height (m).
    pub height: f64,
    /// The crown's widest extent (m); None: no leaves.
    pub crown: Option<f64>,
    /// The leaves' lowest point over the height.
    pub clear: Option<f64>,
    /// The bark's luminance, by area.
    pub bark: Option<f64>,
    /// The saturated colours that aren't a leaf's green.
    pub colours: Vec<Colour>,
    /// The leaf cards' triangles by the class of the tile they are on ("" for none).
    pub tiles: BTreeMap<String, usize>,
}

/// Measures `plant` against the leaf-card `atlas`.
pub fn measure(plant: &Plant, atlas: &Atlas) -> Shape {
    let role = |t: &Tri| plant.parts[t.part].role;
    let lowest = |keep: &dyn Fn(Role) -> bool| {
        plant
            .tris
            .iter()
            .filter(|t| keep(role(t)))
            .flat_map(|t| t.p.iter().map(|p| p[1]))
            .reduce(f64::min)
    };
    let top = plant
        .tris
        .iter()
        .flat_map(|t| t.p.iter().map(|p| p[1]))
        .reduce(f64::max)
        .unwrap_or(0.0);
    let foot = lowest(&Role::plant).unwrap_or(top);
    let height = top - foot;
    let mut shape = Shape {
        height,
        ..Shape::default()
    };
    let leaves: Vec<&Tri> = plant.tris.iter().filter(|t| role(t).leaves()).collect();
    if !leaves.is_empty() {
        let extent = |k: usize| {
            let v = leaves.iter().flat_map(|t| t.p.iter().map(move |p| p[k]));
            v.clone().reduce(f64::max).unwrap_or(0.0) - v.reduce(f64::min).unwrap_or(0.0)
        };
        shape.crown = Some(extent(0).max(extent(2)));
        if height > 0.0 {
            shape.clear = lowest(&Role::leaves).map(|y| (y - foot) / height);
        }
    }
    let (mut lum, mut bark_area) = (0.0, 0.0);
    for t in plant.tris.iter().filter(|t| role(t) == Role::Bark) {
        if let Some(l) = plant.parts[t.part].luminance {
            let a = area(&t.p);
            lum += l * a;
            bark_area += a;
        }
    }
    shape.bark = (bark_area > 0.0).then(|| lum / bark_area);
    let shown: BTreeSet<usize> = plant
        .tris
        .iter()
        .filter(|t| matches!(role(t), Role::Leaves | Role::Cards | Role::Bloom))
        .map(|t| t.part)
        .collect();
    for i in shown {
        let p = &plant.parts[i];
        let (h, s) = hue(p.colour);
        if s >= SATURATED && (p.role == Role::Bloom || !hue_in(h, GREEN)) {
            shape.colours.push(Colour {
                material: p.name.clone(),
                hue: h,
                saturation: s,
                leaf: p.role.leaves(),
            });
        }
    }
    for t in plant.tris.iter().filter(|t| role(t) == Role::Cards) {
        let class = t.uv.and_then(|uv| {
            let mid = [
                (uv[0][0] + uv[1][0] + uv[2][0]) / 3.0,
                (uv[0][1] + uv[1][1] + uv[2][1]) / 3.0,
            ];
            atlas.class_at(mid)
        });
        *shape
            .tiles
            .entry(class.unwrap_or_default().to_string())
            .or_default() += 1;
    }
    shape
}

/// What `shape` has outside its species `sp` (with its kind's height `h` and crown width `w`, strictly), each a
/// finding the baseline counts, and what `atlas` lacks for its leaf.
pub fn findings(
    sp: &Species,
    h: Option<[f64; 2]>,
    w: Option<[f64; 2]>,
    shape: &Shape,
    atlas: &Atlas,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut range = |name: &str, v: Option<f64>, r: Option<[f64; 2]>| {
        if let (Some(v), Some(r)) = (v, r)
            && !crate::review::within(v, r, 0.0)
        {
            let side = if v < r[0] { "<" } else { ">" };
            out.push(format!("{name} {v:.2} {side} {:.2}–{:.2}", r[0], r[1]));
        }
    };
    range("plant H", Some(shape.height), h);
    range("crown W", shape.crown, w);
    let ratio = shape
        .crown
        .filter(|_| shape.height > 0.0)
        .map(|c| c / shape.height);
    range("crown W/H", ratio, sp.crown);
    range("clear trunk", shape.clear, sp.clear);
    range(
        "bark luminance",
        shape.bark,
        sp.bark.as_ref().and_then(|b| b.luminance),
    );
    let expected: Vec<(&str, [f64; 2])> = [("blossom", &sp.blossom), ("fruit", &sp.fruit)]
        .into_iter()
        .filter_map(|(name, b)| Some((name, b.as_ref()?)))
        .flat_map(|(name, b)| b.hue.iter().map(move |h| (name, *h)))
        .collect();
    let leaf: Vec<(&str, [f64; 2])> = sp
        .leaf
        .iter()
        .flat_map(|l| l.hue.iter().map(|h| ("leaf", *h)))
        .collect();
    for c in &shape.colours {
        let allowed: Vec<&(&str, [f64; 2])> = expected
            .iter()
            .chain(leaf.iter().filter(|_| c.leaf))
            .collect();
        if !allowed.iter().any(|(_, r)| hue_in(c.hue, *r)) {
            let theirs = if allowed.is_empty() {
                "no coloured blossom or fruit".to_string()
            } else {
                allowed
                    .iter()
                    .map(|(n, [lo, hi])| format!("{n} {lo:.0}–{hi:.0}°"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            out.push(format!(
                "colour `{}` hue {:.0}° isn't its species' ({theirs})",
                c.material, c.hue
            ));
        }
    }
    if !expected.is_empty()
        && !shape
            .colours
            .iter()
            .any(|c| expected.iter().any(|(_, r)| hue_in(c.hue, *r)))
    {
        let mut names: Vec<&str> = expected.iter().map(|(n, _)| *n).collect();
        names.dedup();
        out.push(format!("shows no {} colour", names.join(" or ")));
    }
    let cards: usize = shape.tiles.values().sum();
    if let Some(leaf) = &sp.leaf
        && cards > 0
    {
        if !atlas.has(&leaf.shape) {
            out.push(format!("the atlas has no {} tile", leaf.shape));
        } else {
            let off: usize = shape
                .tiles
                .iter()
                .filter(|(class, _)| *class != &leaf.shape && *class != "core")
                .map(|(_, n)| n)
                .sum();
            if off > 0 {
                out.push(format!(
                    "{off} of {cards} leaf-card triangles aren't on a {} tile",
                    leaf.shape
                ));
            }
        }
    }
    out
}

/// The manifests' model entries: {key: (its file's path, its entry)}, a later manifest's entry replacing an
/// earlier one's (as the gate reads them).
pub fn model_files(paths: &checkglb::Paths) -> Result<BTreeMap<String, (String, J)>, String> {
    use crate::checkglb::py::{dirname, join, normpath};
    let mut out = BTreeMap::new();
    for (manifest, m) in checkglb::asset_manifests(paths)? {
        for (k, entry) in m.get("entries").and_then(J::entries).unwrap_or(&[]) {
            if entry.get("type").and_then(J::str) != Some("model") {
                continue;
            }
            match entry.get("url").and_then(J::str) {
                Some(url) if !url.is_empty() && !url.contains("://") => {
                    out.insert(
                        k.clone(),
                        (normpath(&join(&dirname(&manifest), url)), entry.clone()),
                    );
                }
                _ => {
                    out.remove(k);
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(name: &str, finish: &str, colour: [f64; 3], luminance: f64) -> Part {
        Part {
            name: name.into(),
            finish: finish.into(),
            role: Role::of(name, finish),
            colour,
            luminance: Some(luminance),
        }
    }

    /// A triangle of part `part` with corners at heights y0 and y1, `w` across x, its UVs at `uv`.
    fn tri(part: usize, y0: f64, y1: f64, w: f64, uv: [f64; 2]) -> Tri {
        Tri {
            part,
            p: [[-w / 2.0, y0, 0.0], [w / 2.0, y0, 0.0], [0.0, y1, 0.1]],
            uv: Some([uv, uv, uv]),
        }
    }

    /// A tree: a trunk from the ground to 3 m (its bark at 0.10), cards from 2 to 6 m, 4 m across, a pink card,
    /// and one on the dense tile.
    fn tree() -> Plant {
        Plant {
            parts: vec![
                part("bark", "bark", [0.3, 0.25, 0.2], 0.10),
                part("card", "leafcards", [0.06, 0.15, 0.03], 0.1),
                part("card2", "leafcards", [0.51, 0.14, 0.30], 0.2),
                part("body", "satin", [0.5, 0.14, 0.05], 0.2),
            ],
            tris: vec![
                tri(0, 0.0, 3.0, 0.4, [0.0, 0.0]),
                tri(1, 2.0, 6.0, 4.0, [0.25, 0.25]),
                tri(2, 3.0, 5.0, 1.0, [0.75, 0.25]),
                tri(1, 2.0, 2.5, 0.5, [0.75, 0.75]),
            ],
        }
    }

    #[test]
    fn a_tree_is_measured_from_its_parts() {
        let s = measure(&tree(), &Atlas::today());
        assert_eq!(s.height, 6.0);
        assert_eq!(s.crown, Some(4.0));
        assert!((s.clear.unwrap() - 2.0 / 6.0).abs() < 1e-9);
        assert!((s.bark.unwrap() - 0.10).abs() < 1e-9);
        // The pink card has a hue to judge; the green one is a leaf's.
        assert_eq!(s.colours.len(), 1);
        assert_eq!(
            (s.colours[0].material.as_str(), s.colours[0].leaf),
            ("card2", true)
        );
        assert!(
            hue_in(s.colours[0].hue, [320.0, 345.0]),
            "{}",
            s.colours[0].hue
        );
        assert_eq!(
            s.tiles,
            BTreeMap::from([("oval".to_string(), 2), ("core".to_string(), 1)])
        );
        // A pot under the plant isn't its height: a tree in a pot is measured from its trunk's foot.
        let mut potted = tree();
        potted.tris.push(tri(3, -0.5, 0.0, 0.6, [0.0, 0.0]));
        assert_eq!(measure(&potted, &Atlas::today()).height, 6.0);
    }

    #[test]
    fn findings_are_strict_and_say_what_is_off() {
        let s = measure(&tree(), &Atlas::today());
        let mut sp = Species {
            latin: "Prunus serrulata".into(),
            crown: Some([0.7, 1.0]),
            clear: Some([0.1, 0.3]),
            leaf: Some(Leaf {
                shape: "oval".into(),
                length: Some([0.05, 0.12]),
                hue: Vec::new(),
            }),
            bark: Some(Bark {
                class: "smooth".into(),
                luminance: Some([0.05, 0.12]),
            }),
            blossom: Some(Bloom {
                colour: "pink".into(),
                hue: vec![[320.0, 350.0]],
                months: [4, 5],
            }),
            ..Species::default()
        };
        assert!(sp.problems().is_empty(), "{:?}", sp.problems());
        let f = findings(&sp, Some([4.0, 6.0]), Some([3.0, 4.0]), &s, &Atlas::today());
        assert_eq!(
            f,
            ["crown W/H 0.67 < 0.70–1.00", "clear trunk 0.33 > 0.10–0.30"]
        );
        // Strict: no 10 % either side.
        let f = findings(&sp, Some([4.0, 5.9]), Some([3.0, 4.0]), &s, &Atlas::today());
        assert_eq!(f[0], "plant H 6.00 > 4.00–5.90");
        // A colour that is neither its blossom's nor its fruit's; and a species whose colour isn't shown.
        sp.blossom = None;
        sp.fruit = Some(Bloom {
            colour: "red or yellow".into(),
            hue: vec![[350.0, 15.0], [45.0, 65.0]],
            months: [8, 10],
        });
        let f = findings(&sp, None, None, &s, &Atlas::today());
        assert!(
            f.iter().any(|f| f.starts_with("colour `card2` hue ")
                && f.ends_with("(fruit 350–15°, fruit 45–65°)")),
            "{f:?}"
        );
        assert!(f.contains(&"shows no fruit colour".to_string()), "{f:?}");
        // A leaf colour its leaves take (autumn's) passes on the leaves.
        sp.fruit = None;
        if let Some(leaf) = sp.leaf.as_mut() {
            leaf.hue = vec![[320.0, 350.0]];
        }
        let f = findings(&sp, None, None, &s, &Atlas::today());
        assert!(!f.iter().any(|f| f.starts_with("colour")), "{f:?}");
        // A lobed leaf on today's atlas: it has no such tile.
        sp.fruit = None;
        if let Some(leaf) = sp.leaf.as_mut() {
            leaf.shape = "lobed".into();
        }
        let f = findings(&sp, None, None, &s, &Atlas::today());
        assert!(
            f.contains(&"the atlas has no lobed tile".to_string()),
            "{f:?}"
        );
        // An atlas that has one: the card on an oval tile is off it, the core's isn't counted.
        let atlas = Atlas::parse(
            r#"{"tiles": [{"name": "oak", "class": "lobed", "rect": [0, 0, 0.5, 0.5]},
                {"class": "oval", "rect": [0.5, 0, 1, 0.5]}, {"class": "core", "rect": [0.5, 0.5, 1, 1]}]}"#,
        )
        .unwrap();
        let s = measure(&tree(), &atlas);
        let f = findings(&sp, None, None, &s, &atlas);
        assert!(
            f.contains(&"1 of 3 leaf-card triangles aren't on a lobed tile".to_string()),
            "{f:?}"
        );
    }

    #[test]
    fn a_block_with_a_shape_or_range_that_isnt_one_is_a_problem() {
        let sp = Species {
            latin: String::new(),
            crown: Some([1.2, 0.8]),
            leaf: Some(Leaf {
                shape: "heart".into(),
                length: None,
                hue: Vec::new(),
            }),
            fruit: Some(Bloom {
                colour: "red".into(),
                hue: vec![[0.0, 400.0]],
                months: [0, 13],
            }),
            ..Species::default()
        };
        assert_eq!(sp.problems().len(), 5, "{:?}", sp.problems());
        assert!(Atlas::parse(r#"{"tiles": [{"class": "heart", "rect": [0, 0, 1, 1]}]}"#).is_err());
    }

    #[test]
    fn hues_as_srgb_shows_them() {
        assert_eq!(hue([1.0, 0.0, 0.0]), (0.0, 1.0));
        // A linear pink (the cherry's card) is a pink hue; a grey has none.
        let (h, s) = hue([0.51, 0.14, 0.30]);
        assert!((320.0..345.0).contains(&h) && s > 0.3, "{h} {s}");
        assert_eq!(hue([0.2, 0.2, 0.2]).1, 0.0);
        assert!(hue_in(5.0, [350.0, 15.0]) && hue_in(355.0, [350.0, 15.0]));
        assert!(!hue_in(30.0, [350.0, 15.0]));
    }
}
