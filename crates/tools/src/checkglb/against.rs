//! `cargo check-glb --against <rev|dir> [keys… | --family <f>] [--dir <built>]`: whether a build of the models is
//! the build it is held against. The proof for moving the models' generator: it reads only `.glb` files, the packs
//! and centres.json, and neither the generator that wrote the old files nor the one that wrote the new.
//!
//!   cargo check-glb --against ref/blender                    every model of the working tree against that revision
//!   cargo check-glb --against HEAD model.fridge model.oak    only these keys
//!   cargo check-glb --against HEAD --family trees            the models of models/trees/ or packs/trees/
//!   cargo check-glb --against ref/blender --dir target/art   the files under target/art in place of the working tree's
//!
//! The old side is a git revision (its files read with `git cat-file`), or, if git knows no such revision, a
//! directory; the new side is the working tree, or `--dir`. A directory is a checkout's root (it has
//! `web/public/assets`, and centres.json where the repo has it) or an assets root (`models/`, `packs/` and, if it
//! has one, `centres.json` beside them). One without a `manifest.json` is a build of some models only: what it
//! lacks is the working tree's, and without keys or a family only the models it has files for are compared.
//! The content and furniture.json, which the gate reads beside the models, are the working tree's on both sides.
//!
//! Each model, and its far version, is one of:
//! - `same`: the two files' digests are equal (`digest`: the same nodes, materials, triangles and clips to the
//!   bit, in whatever order) and so are its pack entry and its centre;
//! - `near`: every row of `METRICS` is within its limit (the constants below);
//! - `CHANGED`: a row is not. Each is printed with its part, its value, its limit and the worst point (in the
//!   part's own axes, which are glTF's: x is the game's -x), and the command exits 1.
//!
//! A row that is right as it is takes a line in tools/art/port-accepted.json, `{"<key>": {"<metric>": "<why>"}}`
//! (a far version's key is `<key> far`, a manifest's its path under the assets), and is then printed as
//! `accepted`. As with the gate's `accept`, one that names no row, gives no reason or is not needed is an error.
//! But `decimated`, which says what a far version is (Blender's decimate made it, and makes another on every
//! run): where the rows fail it, it is held to that rule in their place (`METRICS`); where they pass, it passes.

use super::digest;
use super::glb::{Glb, R, parse_glb};
use super::near::{self, Across, P3, Part, Rest, Surface, Tolerance};
use super::py::{self, J, dirname, join, normpath, relpath};
use super::{Facts, Finishes, Furniture, Paths};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A node's origin may be this far off (m): a tenth of a millimetre, a quarter of what f32 keeps at 3 m, so
/// only rounding passes (a picked fruit, a door's hinge and a hand's reach are fitted to origins).
pub const ORIGIN: f64 = 1e-4;
/// A node's turn at rest may be this far off (degrees): rounding, 0.2 mm at the end of a 1 m door.
pub const REST_TURN: f64 = 0.01;
/// A node's scale, at rest or in a clip, may be this far off: 1 mm on a part a metre long.
pub const SCALE: f64 = 0.001;
/// A material's base colour may be this far off per channel: one step of the 8 bits it is drawn with.
pub const COLOUR: f64 = 1.0 / 255.0;
/// A clip's first and last keys may be this far off (s): a sixteenth of a frame at 60 Hz, so no frame shows it.
pub const CLIP_LENGTH: f64 = 0.001;
/// The clips are compared where the game samples them (modelClips.ts): 60 times a second.
pub const CLIP_HZ: f64 = 60.0;
/// A part's place in a clip may be this far off at a sample (m), as its origin at rest.
pub const CLIP_MOVE: f64 = 1e-4;
/// A part's turn in a clip may be this far off at a sample (degrees): 2 mm at the end of a 1 m door, and what two
/// ways of joining the same keys (an arc, a straight line) differ by between keys 5 degrees apart.
pub const CLIP_TURN: f64 = 0.1;
/// A perch may be this far off (m). The birds' recorded day (web/tests/birds.test.ts) takes them as written, so
/// one that is off at all is said even where it passes.
pub const PERCH: f64 = 0.001;
/// A node's box, a model's size and its functional heights may be this far off (m): half a millimetre, an
/// eighth of a pixel at the play camera, and under the gate's own slack everywhere.
pub const BOUNDS: f64 = 0.0005;
/// A material may have this share fewer or more triangles: a bevel or a hole's fill cut another way, not a part
/// gone or doubled.
pub const TRIANGLES: (f64, f64) = (-0.20, 0.10);
/// Surface samples a square metre (one a square centimetre), and the fewest and the most for a part: enough to
/// find a face of a square centimetre, bounded so the largest tree takes a second.
pub const SAMPLES: (f64, usize, usize) = (1e4, 2000, 50000);
/// A part's surface may lie this far from the other build's on average (m): a whole part moved by a quarter of a
/// millimetre fails.
pub const SURFACE_MEAN: f64 = 0.0002;
/// No point of a part may be further than this from the other build's surface (m): half a pixel at the play
/// camera, room for a bevel's band cut another way.
pub const SURFACE_MAX: f64 = 0.002;
/// The same for a moving part and for fruit (m): a hand, a hinge or a lid is fitted to them
/// (web/src/render/shared/characters/clips/uses.ts `FRUIT`, `HARVEST_APPLE`).
pub const SURFACE_FITTED: f64 = 0.0005;
/// Materials that are fruit a resident reaches for (their names start so): held to `SURFACE_FITTED`.
pub const FRUIT: [&str; 3] = ["fruit", "apple", "lemon"];
/// A normal turned by more than this (degrees) shows as another shade; a bevel's band rounded another way stays under.
pub const NORMAL_TURN: f64 = 15.0;
/// The share of a part's surface whose normal may be turned that far: the bands along its edges.
pub const NORMAL_SHARE: f64 = 0.01;
/// A normal turned by more than this (degrees) faces the other way: a face inside out. None may.
pub const NORMAL_AWAY: f64 = 90.0;
/// The counterpart of a point may be this much further than the nearest point if it faces more as the point does
/// (m): of two faces back to back or meeting in an edge, the one meant.
pub const NORMAL_REACH: f64 = 0.0002;
/// Faces whose nearest points are within this of each other are one place (m): parts laid into each other,
/// whose texture and shading there are either's.
pub const SAME_PLACE: f64 = 1e-5;
/// A material's area may differ by this share: a face gone or doubled inside another shows here first.
pub const AREA: f64 = 0.01;
/// A part's texels a metre may differ by this share: the texture's scale is the finish's look.
pub const TEXELS: f64 = 0.02;
/// A texture coordinate may be this far off, measured on the surface (m): half a pixel at the play camera.
pub const UV_OFF: f64 = 0.002;
/// The share of a part's surface whose texture must be within `UV_OFF`: all but its seams and its bevels' bands.
/// Two Blender builds of one script differ on up to 3.9 % of a part (measured 2026-11-17, the porch swing's frame:
/// a face at 45° between two sides takes its texture from either), so 5 % may be off.
pub const UV_SHARE: f64 = 0.95;
/// The baked shading is rounded to this many greys (gamekit/shade.py `LEVELS`).
pub const SHADE_LEVELS: f64 = 32.0;
/// A part's shading may differ by this many greys on average. Two Blender builds of one script differ by up to
/// 1.36 (measured 2026-11-17, the pizza oven's cup: a vertex's rays hit or miss by a hair), so 2.
pub const SHADE_MEAN: f64 = 2.0;
/// 95 % of a part's surface must be within this many greys. Two Blender builds differ by up to 8.6 there (the
/// same cup), so 10.
pub const SHADE_MOST: f64 = 10.0;
/// A far version held to `decimated` may differ from the old one's size by this share a side: the gate's own
/// rule for a far version against its model is 5 %.
pub const DECIMATED_SIZE: f64 = 0.02;
/// Its lowest point may be this far from the full model's (m): the gate's own rule, a far version stands where
/// its model stands.
pub const DECIMATED_FLOOR: f64 = 0.005;
/// It may have this share more triangles than the old one: a far version is there to be cheap.
pub const DECIMATED_TRIANGLES: f64 = 0.10;
/// No point of it may be further than this from the old one's surface (m). Two Blender builds of one script
/// were up to 55 mm apart (measured 2026-11-17, the ping-pong table's far version), so 60 mm.
pub const DECIMATED_SURFACE: f64 = 0.06;

/// What is compared, by the name a finding and a line of port-accepted.json have.
pub const METRICS: &[(&str, &str)] = &[
    ("file", "a file is missing or cannot be read"),
    (
        "nodes",
        "the nodes' names and parents, their origins, turns and scales at rest",
    ),
    (
        "materials",
        "the materials, their colours and the rest they say, each part's material and its attributes",
    ),
    ("draws", "the draw calls, as the gate counts them"),
    (
        "clips",
        "the clips' names and lengths, and each part's place, turn and size sampled at 60 Hz",
    ),
    ("pack", "the model's pack entry, or the rest of a manifest"),
    (
        "perches",
        "the perches: as many, in the same order, each within a millimetre",
    ),
    ("centre", "the model's centre in centres.json"),
    ("bounds", "each node's box"),
    ("size", "the model's size as the gate measures it"),
    (
        "heights",
        "its functional heights (seat, top, mattress, light)",
    ),
    (
        "gate",
        "the gate's errors and warnings for the key: none new",
    ),
    (
        "triangles",
        "the triangles of each material, and the budget's target",
    ),
    (
        "surface",
        "how far each part's surface is from the other build's, both ways",
    ),
    ("normals", "the normals at the nearest points"),
    ("area", "each material's area"),
    ("texels", "each part's texels a metre"),
    ("uv", "the texture coordinates at the nearest points"),
    ("shading", "the baked shading at the nearest points"),
    (
        "decimated",
        "a far version Blender's decimate made, which not even Blender repeats: where the rows above fail it is \
         held to its own rule in their place (the old size within 2 %, its lowest point within 0.5 cm of the \
         model's, at most a tenth more triangles, its surface within 6 cm of the old one's)",
    ),
];

/// One row of `METRICS` that is over its limit.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub metric: &'static str,
    pub text: String,
}

fn found(metric: &'static str, text: String) -> Finding {
    debug_assert!(METRICS.iter().any(|(m, _)| *m == metric), "{metric}");
    Finding { metric, text }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Class {
    Same,
    Near,
    Changed,
}

/// The largest differences measured between two files that are not the same, for a `near` line.
#[derive(Clone, Debug, Default)]
pub struct Numbers {
    /// Which parts of the digest differ (`digest::Parts::differing`).
    pub differs: Vec<&'static str>,
    /// The worst part's mean and largest distance (m).
    pub surface_mean: f64,
    pub surface_max: f64,
    /// The worst part's share of turned normals, of texture coordinates off, and mean shading difference (greys).
    pub turned: f64,
    pub uv_off: f64,
    pub shade: f64,
    /// The material whose triangle count changed most, as a share (signed).
    pub triangles: f64,
}

impl Numbers {
    fn text(&self) -> String {
        let mut out = Vec::new();
        if !self.differs.is_empty() {
            out.push(format!("differs in {}", self.differs.join(", ")));
        }
        out.push(format!(
            "surface {} mean, {} at most",
            mm(self.surface_mean),
            mm(self.surface_max)
        ));
        if self.turned > 0.0 {
            out.push(format!("normals {} turned", share(self.turned)));
        }
        if self.uv_off > 0.0 {
            out.push(format!("texture {} off", share(self.uv_off)));
        }
        if self.shade > 0.0 {
            out.push(format!("shading {:.2} greys", self.shade));
        }
        if self.triangles != 0.0 {
            out.push(format!("triangles {:+.1} %", self.triangles * 100.0));
        }
        out.join("; ")
    }
}

fn mm(v: f64) -> String {
    if v.is_finite() {
        format!("{:.3} mm", v * 1000.0)
    } else {
        "no surface".into()
    }
}

fn share(v: f64) -> String {
    format!("{:.2} %", v * 100.0)
}

fn point(p: P3) -> String {
    format!("({:.4}, {:.4}, {:.4})", p[0], p[1], p[2])
}

/// Two files compared: whether they are the same, else what is over its limit and the numbers.
#[derive(Debug, Default)]
pub struct Compared {
    pub same: bool,
    pub findings: Vec<Finding>,
    pub numbers: Numbers,
}

/// Compares two `.glb` files' bytes: `same` if their digests are equal, else every row of `METRICS` a file alone
/// answers (the pack, the centre and the gate's rows need its entry: `against`).
pub fn compare(old: &[u8], new: &[u8]) -> Compared {
    compare_by(old, new, &measure)
}

/// `compare` for a far version held to `decimated` (the `DECIMATED_` limits); `model` is the new build's full
/// model, whose lowest point the far version's is held to (None: not held to it).
pub fn compare_decimated(old: &[u8], new: &[u8], model: Option<&[u8]>) -> Compared {
    let floor = model
        .and_then(|data| parse_glb(data).ok())
        .and_then(|g| whole(&g).ok())
        .and_then(|part| part.bounds())
        .map(|(lo, _)| lo[1]);
    compare_by(old, new, &|o, n, out, numbers| {
        measure_decimated(o, n, floor, out, numbers)
    })
}

/// What two files that are not the same are measured by: their findings and numbers.
type Measure<'a> = &'a dyn Fn(&Glb, &Glb, &mut Vec<Finding>, &mut Numbers) -> R<()>;

fn compare_by(old: &[u8], new: &[u8], measure: Measure) -> Compared {
    let mut out = Compared::default();
    if old == new {
        out.same = true;
        return out;
    }
    let read = |data: &[u8], which: &str| {
        parse_glb(data).map_err(|e| found("file", format!("the {which} file is no .glb: {e}")))
    };
    let (o, n) = match (read(old, "old"), read(new, "new")) {
        (Ok(o), Ok(n)) => (o, n),
        (o, n) => {
            out.findings.extend(o.err().into_iter().chain(n.err()));
            return out;
        }
    };
    let measured = (|| -> R<()> {
        if digest::digest(&o)? == digest::digest(&n)? {
            out.same = true;
            return Ok(());
        }
        out.numbers.differs = digest::parts(&o)?.differing(&digest::parts(&n)?);
        measure(&o, &n, &mut out.findings, &mut out.numbers)
    })();
    if let Err(e) = measured {
        out.findings
            .push(found("file", format!("cannot be compared: {e}")));
    }
    out
}

/// A list of names for a finding: the first six, then how many more.
fn names(list: &[String]) -> String {
    let mut out = list.iter().take(6).cloned().collect::<Vec<_>>().join(", ");
    if list.len() > 6 {
        out += &format!(" and {} more", list.len() - 6);
    }
    out
}

/// What is only in one of two sets, as findings.
fn only<'a>(
    metric: &'static str,
    what: &str,
    old: impl Iterator<Item = &'a String>,
    new: impl Iterator<Item = &'a String>,
    out: &mut Vec<Finding>,
) {
    let (old, new): (BTreeSet<&String>, BTreeSet<&String>) = (old.collect(), new.collect());
    for (side, list) in [("old", old.difference(&new)), ("new", new.difference(&old))] {
        let list: Vec<String> = list.map(|s| s.to_string()).collect();
        if !list.is_empty() {
            out.push(found(
                metric,
                format!("{what} only in the {side} file: {}", names(&list)),
            ));
        }
    }
}

/// A file read for the rows of `METRICS`: its nodes, their rests, its clips and its parts.
struct Model {
    tree: digest::Tree,
    rests: Vec<Rest>,
    clips: Vec<near::Clip>,
    parts: Vec<Part>,
    meshes: HashSet<String>,
}

fn read(g: &Glb) -> R<Model> {
    let tree = digest::tree(&g.gltf)?;
    let nodes = g.gltf.get("nodes").and_then(J::list).unwrap_or(&[]);
    Ok(Model {
        rests: near::rests(&g.gltf)?,
        clips: near::clips(g, &tree)?,
        parts: near::parts(g, &tree)?,
        meshes: nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.has("mesh"))
            .map(|(i, _)| tree.paths[i].clone())
            .collect(),
        tree,
    })
}

impl Model {
    fn rest(&self, path: &str) -> Option<&Rest> {
        self.tree
            .paths
            .iter()
            .position(|p| p == path)
            .map(|i| &self.rests[i])
    }
}

fn measure(o: &Glb, n: &Glb, out: &mut Vec<Finding>, numbers: &mut Numbers) -> R<()> {
    let (old, new) = (read(o)?, read(n)?);
    nodes_differ(&old, &new, out);
    materials_differ(o, n, out);
    clips_differ(&old, &new, out);
    parts_differ(&old, &new, out, numbers);
    Ok(())
}

/// The nodes: the same paths (names under the same parents), each with or without a mesh as before, at rest where
/// it was.
fn nodes_differ(old: &Model, new: &Model, out: &mut Vec<Finding>) {
    only(
        "nodes",
        "nodes",
        old.tree.paths.iter(),
        new.tree.paths.iter(),
        out,
    );
    for (i, path) in old.tree.paths.iter().enumerate() {
        let (a, Some(b)) = (&old.rests[i], new.rest(path)) else {
            continue;
        };
        if old.meshes.contains(path) != new.meshes.contains(path) {
            let side = if old.meshes.contains(path) {
                "old"
            } else {
                "new"
            };
            out.push(found(
                "nodes",
                format!("{path}: has a mesh only in the {side} file"),
            ));
        }
        let moved = py::dist(&a.translation, &b.translation);
        if moved > ORIGIN {
            out.push(found(
                "nodes",
                format!(
                    "{path}: its origin moved {}, at most {} (from {} to {})",
                    mm(moved),
                    mm(ORIGIN),
                    point(a.translation),
                    point(b.translation)
                ),
            ));
        }
        let turned = near::turn(a.rotation, b.rotation);
        if turned > REST_TURN {
            out.push(found(
                "nodes",
                format!("{path}: turned {turned:.3}° at rest, at most {REST_TURN}"),
            ));
        }
        let scaled = (0..3)
            .map(|k| (a.scale[k] - b.scale[k]).abs())
            .fold(0.0, f64::max);
        if scaled > SCALE {
            out.push(found(
                "nodes",
                format!(
                    "{path}: its scale at rest {} is {}, at most {SCALE} off",
                    point(a.scale),
                    point(b.scale)
                ),
            ));
        }
    }
}

/// The differences between two JSON values, each with its path: a number by its value, an object whatever the
/// order of its keys.
fn json_differ(a: Option<&J>, b: Option<&J>, path: &str, out: &mut Vec<String>) {
    let short = |v: &J| {
        let text = digest::canon(v);
        if text.chars().count() > 60 {
            format!("{}…", text.chars().take(60).collect::<String>())
        } else {
            text
        }
    };
    match (a, b) {
        (None, None) => {}
        (Some(a), None) => out.push(format!("{path} ({}) is gone", short(a))),
        (None, Some(b)) => out.push(format!("{path} ({}) is new", short(b))),
        (Some(J::Obj(a)), Some(J::Obj(b))) => {
            let mut keys: Vec<&String> = a.iter().map(|(k, _)| k).collect();
            keys.extend(
                b.iter()
                    .map(|(k, _)| k)
                    .filter(|k| !a.iter().any(|(x, _)| x == *k)),
            );
            for k in keys {
                let of =
                    |o: &'_ [(String, J)]| o.iter().find(|(x, _)| x == k).map(|(_, v)| v.clone());
                let inner = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                json_differ(of(a).as_ref(), of(b).as_ref(), &inner, out);
            }
        }
        (Some(J::Arr(a)), Some(J::Arr(b))) if a.len() == b.len() => {
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                json_differ(Some(x), Some(y), &format!("{path}[{i}]"), out);
            }
        }
        (Some(a), Some(b)) => {
            if digest::canon(a) != digest::canon(b) {
                out.push(format!("{path}: {} is now {}", short(a), short(b)));
            }
        }
    }
}

/// An object without some of its keys.
fn without(v: &J, keys: &[&str]) -> J {
    match v {
        J::Obj(o) => J::Obj(
            o.iter()
                .filter(|(k, _)| !keys.contains(&k.as_str()))
                .cloned()
                .collect(),
        ),
        other => other.clone(),
    }
}

/// The materials: the same names, each with its base colour within a step and everything else it says.
fn materials_differ(o: &Glb, n: &Glb, out: &mut Vec<Finding>) {
    let named = |g: &Glb| -> BTreeMap<String, J> {
        g.gltf
            .get("materials")
            .and_then(J::list)
            .unwrap_or(&[])
            .iter()
            .map(|m| (m.get("name").map(J::key).unwrap_or_default(), m.clone()))
            .collect()
    };
    let (old, new) = (named(o), named(n));
    only("materials", "materials", old.keys(), new.keys(), out);
    for (name, a) in &old {
        let Some(b) = new.get(name) else { continue };
        let colour = |m: &J| -> [f64; 4] {
            let list: Vec<f64> = m
                .get("pbrMetallicRoughness")
                .and_then(|p| p.get("baseColorFactor"))
                .and_then(J::list)
                .unwrap_or(&[])
                .iter()
                .filter_map(|v| v.num().map(|x| x.f()))
                .collect();
            list.try_into().unwrap_or([1.0; 4])
        };
        let (ca, cb) = (colour(a), colour(b));
        let off = (0..4).map(|k| (ca[k] - cb[k]).abs()).fold(0.0, f64::max);
        if off > COLOUR {
            let text =
                |c: [f64; 4]| format!("({:.3}, {:.3}, {:.3}, {:.3})", c[0], c[1], c[2], c[3]);
            out.push(found(
                "materials",
                format!(
                    "material {name}: its base colour {} is now {}, {:.1} of 255 off, at most 1",
                    text(ca),
                    text(cb),
                    off * 255.0
                ),
            ));
        }
        let rest = |m: &J| {
            let mut m = without(m, &["name"]);
            if let Some(pbr) = m
                .get("pbrMetallicRoughness")
                .map(|p| without(p, &["baseColorFactor"]))
            {
                m.set("pbrMetallicRoughness", pbr);
            }
            m
        };
        let mut differs = Vec::new();
        json_differ(Some(&rest(a)), Some(&rest(b)), "", &mut differs);
        if !differs.is_empty() {
            out.push(found(
                "materials",
                format!("material {name}: {}", names(&differs)),
            ));
        }
    }
}

/// The clips: the same names, as long, and every part they move where it was at each sixtieth of a second (a part
/// one build's clip leaves alone is at rest there). How many keys a clip has is free.
fn clips_differ(old: &Model, new: &Model, out: &mut Vec<Finding>) {
    only(
        "clips",
        "clips",
        old.clips.iter().map(|c| &c.name),
        new.clips.iter().map(|c| &c.name),
        out,
    );
    for a in &old.clips {
        let Some(b) = new.clips.iter().find(|c| c.name == a.name) else {
            continue;
        };
        for (what, x, y) in [("starts", a.start, b.start), ("ends", a.end, b.end)] {
            if (x - y).abs() > CLIP_LENGTH {
                out.push(found(
                    "clips",
                    format!(
                        "clip {}: {what} at {y:.4} s, was {x:.4} (at most 1 ms off)",
                        a.name
                    ),
                ));
            }
        }
        let end = a.end.max(b.end);
        let mut times: Vec<f64> = (0..=(end * CLIP_HZ).floor() as usize)
            .map(|k| k as f64 / CLIP_HZ)
            .collect();
        times.push(end);
        let tracks: BTreeSet<&(String, String)> = a.tracks.keys().chain(b.tracks.keys()).collect();
        for track in tracks {
            let (node, path) = track;
            let (Some(ra), Some(rb)) = (old.rest(node), new.rest(node)) else {
                continue;
            };
            let rest = |r: &Rest| -> Vec<f64> {
                match path.as_str() {
                    "translation" => r.translation.to_vec(),
                    "rotation" => r.rotation.to_vec(),
                    "scale" => r.scale.to_vec(),
                    _ => Vec::new(),
                }
            };
            let rotation = path == "rotation";
            // (off by, at, limit, unit)
            let mut worst: Option<(f64, f64)> = None;
            let (limit, unit) = match path.as_str() {
                "translation" => (CLIP_MOVE * 1000.0, "mm"),
                "rotation" => (CLIP_TURN, "°"),
                _ => (SCALE, ""),
            };
            for &t in &times {
                let va = a
                    .tracks
                    .get(track)
                    .and_then(|k| k.at(t, rotation))
                    .unwrap_or_else(|| rest(ra));
                let vb = b
                    .tracks
                    .get(track)
                    .and_then(|k| k.at(t, rotation))
                    .unwrap_or_else(|| rest(rb));
                let off = if va.len() != vb.len() {
                    f64::INFINITY
                } else {
                    match path.as_str() {
                        "translation" => py::dist(&va, &vb) * 1000.0,
                        "rotation" if va.len() == 4 => {
                            near::turn([va[0], va[1], va[2], va[3]], [vb[0], vb[1], vb[2], vb[3]])
                        }
                        _ => va
                            .iter()
                            .zip(&vb)
                            .map(|(x, y)| (x - y).abs())
                            .fold(0.0, f64::max),
                    }
                };
                if worst.is_none_or(|(w, _)| off > w) {
                    worst = Some((off, t));
                }
            }
            if let Some((off, t)) = worst.filter(|(off, _)| *off > limit) {
                out.push(found(
                    "clips",
                    format!(
                        "clip {}: {node}'s {path} is {off:.3}{unit} off at {t:.3} s, at most {limit}{unit}",
                        a.name
                    ),
                ));
            }
        }
    }
}

/// Whether a material is fruit a resident reaches for (`FRUIT`).
fn fruit(material: &str) -> bool {
    FRUIT.iter().any(|f| material.starts_with(f))
}

fn tolerance() -> Tolerance {
    Tolerance {
        per_m2: SAMPLES.0,
        few: SAMPLES.1,
        many: SAMPLES.2,
        reach: NORMAL_REACH,
        same: SAME_PLACE,
        turned: NORMAL_TURN,
        away: NORMAL_AWAY,
        uv_off: UV_OFF,
    }
}

/// Two builds of a part measured against each other, both ways: (old against new, new against old).
fn both_ways(a: &Part, b: &Part) -> (Across, Across) {
    let (sa, sb) = (Surface::new(a), Surface::new(b));
    let (area, uv) = (a.area(), a.uv_area());
    let uv_metres = if uv > 0.0 { (area / uv).sqrt() } else { 0.0 };
    let tol = tolerance();
    (
        near::across(a, &sb, &tol, uv_metres, SHADE_LEVELS),
        near::across(b, &sa, &tol, uv_metres, SHADE_LEVELS),
    )
}

/// The parts: the same nodes' triangles of the same materials with the same attributes, about as many triangles
/// and as much area a material, each node's box, and each part's surface, normals, texture and shading against the
/// other build's.
fn parts_differ(old: &Model, new: &Model, out: &mut Vec<Finding>, numbers: &mut Numbers) {
    // (A part of a node the other file has not is the nodes' finding.)
    let names_of = |r: &Model, other: &Model| -> Vec<String> {
        r.parts
            .iter()
            .filter(|p| other.tree.paths.contains(&p.path))
            .map(Part::name)
            .collect()
    };
    let (on, nn) = (names_of(old, new), names_of(new, old));
    only(
        "materials",
        "parts (node / material) drawn",
        on.iter(),
        nn.iter(),
        out,
    );

    // A material's triangles and area, over every node.
    let by_material = |r: &Model| -> BTreeMap<String, (usize, f64)> {
        let mut m: BTreeMap<String, (usize, f64)> = BTreeMap::new();
        for p in &r.parts {
            let e = m.entry(p.material.clone()).or_default();
            e.0 += p.triangles.len();
            e.1 += p.area();
        }
        m
    };
    let (om, nm) = (by_material(old), by_material(new));
    for (material, (ta, aa)) in &om {
        let Some((tb, ab)) = nm.get(material) else {
            continue;
        };
        let delta = if *ta == 0 {
            0.0
        } else {
            *tb as f64 / *ta as f64 - 1.0
        };
        if delta.abs() > numbers.triangles.abs() {
            numbers.triangles = delta;
        }
        if delta < TRIANGLES.0 || delta > TRIANGLES.1 || (*ta == 0) != (*tb == 0) {
            out.push(found(
                "triangles",
                format!(
                    "material {material}: {tb} triangles, was {ta} ({:+.1} %; {:+.0} % to {:+.0} %)",
                    delta * 100.0,
                    TRIANGLES.0 * 100.0,
                    TRIANGLES.1 * 100.0
                ),
            ));
        }
        let off = (aa - ab).abs() / aa.max(*ab).max(1e-12);
        if off > AREA && aa.max(*ab) > 1e-8 {
            out.push(found(
                "area",
                format!(
                    "material {material}: {:.4} m², was {:.4} ({} off, at most {})",
                    ab,
                    aa,
                    share(off),
                    share(AREA)
                ),
            ));
        }
    }

    // Each node's box.
    let boxes = |r: &Model| -> BTreeMap<String, (P3, P3)> {
        let mut m: BTreeMap<String, (P3, P3)> = BTreeMap::new();
        for p in &r.parts {
            let Some((lo, hi)) = p.bounds() else { continue };
            m.entry(p.path.clone())
                .and_modify(|(l, h)| {
                    *l = std::array::from_fn(|k| l[k].min(lo[k]));
                    *h = std::array::from_fn(|k| h[k].max(hi[k]));
                })
                .or_insert((lo, hi));
        }
        m
    };
    let (ob, nb) = (boxes(old), boxes(new));
    for (path, (alo, ahi)) in &ob {
        let Some((blo, bhi)) = nb.get(path) else {
            continue;
        };
        let mut worst = (0.0, String::new());
        for k in 0..3 {
            for (side, a, b) in [("lowest", alo[k], blo[k]), ("highest", ahi[k], bhi[k])] {
                if (a - b).abs() > worst.0 {
                    worst = (
                        (a - b).abs(),
                        format!("its {side} {} is {b:.4} m, was {a:.4}", ["x", "y", "z"][k]),
                    );
                }
            }
        }
        if worst.0 > BOUNDS {
            out.push(found(
                "bounds",
                format!(
                    "{path}: {} ({} off, at most {})",
                    worst.1,
                    mm(worst.0),
                    mm(BOUNDS)
                ),
            ));
        }
    }

    // The parts a clip of either build moves: a node it names and all under it.
    let moved: BTreeSet<&String> = old
        .clips
        .iter()
        .chain(&new.clips)
        .flat_map(|c| c.tracks.keys().map(|(node, _)| node))
        .collect();
    let moving = |path: &str| {
        moved.iter().any(|m| {
            path == m.as_str()
                || path
                    .strip_prefix(m.as_str())
                    .is_some_and(|rest| rest.starts_with('/'))
        })
    };
    for a in &old.parts {
        let Some(b) = new
            .parts
            .iter()
            .find(|p| p.path == a.path && p.material == a.material)
        else {
            continue;
        };
        let name = a.name();
        for (what, x, y) in [
            ("NORMAL", a.normals.is_some(), b.normals.is_some()),
            ("TEXCOORD_0", a.uvs.is_some(), b.uvs.is_some()),
            ("COLOR_0", a.greys.is_some(), b.greys.is_some()),
        ] {
            if x != y {
                let side = if x { "old" } else { "new" };
                out.push(found(
                    "materials",
                    format!("{name}: has {what} only in the {side} file"),
                ));
            }
        }
        // Texels a metre: the texture's length over the surface's.
        let density = |p: &Part| {
            let area = p.area();
            if area > 0.0 {
                (p.uv_area() / area).sqrt()
            } else {
                0.0
            }
        };
        let (da, db) = (density(a), density(b));
        if da.max(db) > 1e-9 {
            let off = (da - db).abs() / da.max(db);
            if off > TEXELS {
                out.push(found(
                    "texels",
                    format!(
                        "{name}: its texture runs {db:.4} of itself a metre, was {da:.4} ({} off, at most {})",
                        share(off),
                        share(TEXELS)
                    ),
                ));
            }
        }
        let (ab, ba) = both_ways(a, b);
        let fitted = moving(&a.path) || fruit(&a.material);
        let limit = if fitted { SURFACE_FITTED } else { SURFACE_MAX };
        let mean = ab.mean.max(ba.mean);
        numbers.surface_mean = numbers.surface_mean.max(mean);
        if mean > SURFACE_MEAN {
            out.push(found(
                "surface",
                format!(
                    "{name}: {} from the other build's on average, at most {}",
                    mm(mean),
                    mm(SURFACE_MEAN)
                ),
            ));
        }
        let (max, at, of, from) = if ab.max >= ba.max {
            (ab.max, ab.max_at, "old", "new")
        } else {
            (ba.max, ba.max_at, "new", "old")
        };
        numbers.surface_max = numbers.surface_max.max(max);
        if max > limit {
            out.push(found(
                "surface",
                format!(
                    "{name}: the {of} file's point {} is {} from the {from} surface, at most {}{}",
                    point(at),
                    mm(max),
                    mm(limit),
                    if !fitted {
                        ""
                    } else if moving(&a.path) {
                        " (a moving part)"
                    } else {
                        " (fruit)"
                    }
                ),
            ));
        }
        let worse = if ab.turned >= ba.turned { &ab } else { &ba };
        numbers.turned = numbers.turned.max(worse.turned);
        if worse.turned > NORMAL_SHARE {
            out.push(found(
                "normals",
                format!(
                    "{name}: {} of its surface has its normal turned over {NORMAL_TURN}° (the most {:.1}° at {}), at most {}",
                    share(worse.turned),
                    worse.turn,
                    point(worse.turn_at),
                    share(NORMAL_SHARE)
                ),
            ));
        }
        if ab.away + ba.away > 0 {
            let (w, of) = if ab.away > 0 {
                (&ab, "old")
            } else {
                (&ba, "new")
            };
            out.push(found(
                "normals",
                format!(
                    "{name}: faces the other way at {} points (the {of} file's {}): a normal turned over {NORMAL_AWAY}°",
                    ab.away + ba.away,
                    point(w.away_at)
                ),
            ));
        }
        let worse = [&ab, &ba]
            .into_iter()
            .filter_map(|w| w.uv)
            .max_by(|x, y| x.0.total_cmp(&y.0));
        if let Some((off, furthest, at)) = worse {
            numbers.uv_off = numbers.uv_off.max(off);
            if off > 1.0 - UV_SHARE {
                out.push(found(
                    "uv",
                    format!(
                        "{name}: {} of its surface has its texture over {} off (the most {} at {}), at most {}",
                        share(off),
                        mm(UV_OFF),
                        mm(furthest),
                        point(at),
                        share(1.0 - UV_SHARE)
                    ),
                ));
            }
        }
        let worse = [&ab, &ba]
            .into_iter()
            .filter_map(|w| w.shade)
            .max_by(|x, y| x.0.total_cmp(&y.0));
        if let Some((mean, most, largest, at)) = worse {
            numbers.shade = numbers.shade.max(mean);
            if mean > SHADE_MEAN || most > SHADE_MOST {
                out.push(found(
                    "shading",
                    format!(
                        "{name}: its baked shading is {mean:.2} greys of {SHADE_LEVELS} off on average and {most:.2} over 95 % of it \
                         (the most {largest:.1} at {}), at most {SHADE_MEAN} and {SHADE_MOST}",
                        point(at)
                    ),
                ));
            }
        }
    }
}

/// A model at rest as one part: every node's triangles where the node stands.
fn whole(g: &Glb) -> R<Part> {
    let r = read(g)?;
    let worlds = near::worlds(&r.tree, &r.rests);
    let mut all = Part {
        node: 0,
        path: String::new(),
        material: String::new(),
        positions: Vec::new(),
        normals: Some(Vec::new()),
        uvs: None,
        greys: None,
        triangles: Vec::new(),
    };
    for p in r.parts {
        let world = worlds[p.node];
        let p = p.placed(&world);
        let base = all.positions.len() as u32;
        all.triangles
            .extend(p.triangles.iter().map(|t| t.map(|i| i + base)));
        match (all.normals.as_mut(), p.normals) {
            (Some(have), Some(more)) => have.extend(more),
            _ => all.normals = None,
        }
        all.positions.extend(p.positions);
    }
    Ok(all)
}

/// A far version held to `decimated`, the whole of it at rest as one surface: each side of its box within
/// `DECIMATED_SIZE` of the old one's, its lowest point within `DECIMATED_FLOOR` of `floor` (the full model's),
/// at most `DECIMATED_TRIANGLES` more triangles, and every point of it within `DECIMATED_SURFACE` of the old one.
fn measure_decimated(
    o: &Glb,
    n: &Glb,
    floor: Option<f64>,
    out: &mut Vec<Finding>,
    numbers: &mut Numbers,
) -> R<()> {
    let (a, b) = (whole(o)?, whole(n)?);
    numbers.triangles = if a.triangles.is_empty() {
        0.0
    } else {
        b.triangles.len() as f64 / a.triangles.len() as f64 - 1.0
    };
    if b.triangles.len() as f64 > a.triangles.len() as f64 * (1.0 + DECIMATED_TRIANGLES) {
        out.push(found(
            "triangles",
            format!(
                "{} triangles, the decimated one had {} ({:+.1} %, at most {:+.0} %)",
                b.triangles.len(),
                a.triangles.len(),
                numbers.triangles * 100.0,
                DECIMATED_TRIANGLES * 100.0
            ),
        ));
    }
    if let (Some((alo, ahi)), Some((blo, bhi))) = (a.bounds(), b.bounds()) {
        let size = |lo: P3, hi: P3| -> P3 { std::array::from_fn(|k| hi[k] - lo[k]) };
        let (was, is) = (size(alo, ahi), size(blo, bhi));
        if (0..3).any(|k| (was[k] - is[k]).abs() > DECIMATED_SIZE * was[k] + 1e-6) {
            let text = |s: P3| format!("{:.4} x {:.4} x {:.4} m", s[0], s[1], s[2]);
            out.push(found(
                "size",
                format!(
                    "{}, the decimated one was {} (each side within {})",
                    text(is),
                    text(was),
                    share(DECIMATED_SIZE)
                ),
            ));
        }
        if let Some(floor) = floor.filter(|floor| (blo[1] - floor).abs() > DECIMATED_FLOOR) {
            out.push(found(
                "bounds",
                format!(
                    "its lowest point is at {:.4} m, the model's at {floor:.4} (at most {} apart)",
                    blo[1],
                    mm(DECIMATED_FLOOR)
                ),
            ));
        }
    }
    let (ab, ba) = both_ways(&a, &b);
    let (max, at, of, from) = if ab.max >= ba.max {
        (ab.max, ab.max_at, "old", "new")
    } else {
        (ba.max, ba.max_at, "new", "old")
    };
    numbers.surface_mean = ab.mean.max(ba.mean);
    numbers.surface_max = max;
    if max > DECIMATED_SURFACE {
        out.push(found(
            "surface",
            format!(
                "the {of} file's point {} is {} from the {from} surface, at most {}",
                point(at),
                mm(max),
                mm(DECIMATED_SURFACE)
            ),
        ));
    }
    Ok(())
}

/// Where a side's files come from.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    /// The working tree of the checkout.
    Tree,
    /// A git revision of the checkout.
    Rev(String),
    /// A directory: a checkout's root or an assets root, whole or a build of some models (the module's note).
    Dir(String),
}

/// What to compare.
pub struct Options {
    /// The checkout: its working tree is `Source::Tree`, its git the revisions, its content and furniture.json
    /// what the gate reads on both sides.
    pub root: String,
    pub old: Source,
    pub new: Source,
    /// Only these keys (none: every model, or `family`'s).
    pub keys: Vec<String>,
    /// Only the models of this folder of `models/` or `packs/` (case and underscores aside, `pack_` or not).
    pub family: Option<String>,
    /// port-accepted.json (None: the checkout's tools/art/port-accepted.json).
    pub accepted: Option<String>,
}

/// One model, far version or manifest compared.
#[derive(Debug)]
pub struct Row {
    /// The model's key, `<key> far` for its far version, or a manifest's path under the assets.
    pub name: String,
    pub class: Class,
    pub findings: Vec<Finding>,
    /// Findings port-accepted.json accepts, each with its reason.
    pub accepted: Vec<(Finding, String)>,
    /// What is worth saying of a row that passes (a perch that is off within its limit).
    pub notes: Vec<String>,
    pub numbers: Option<Numbers>,
}

/// What `against` found.
pub struct Outcome {
    pub rows: Vec<Row>,
    /// What is wrong with port-accepted.json.
    pub errors: Vec<String>,
    /// The report: a line a row that is not `same` (every row when keys or a family were named) and the counts.
    pub text: String,
}

impl Outcome {
    pub fn count(&self, class: Class) -> usize {
        self.rows.iter().filter(|r| r.class == class).count()
    }

    pub fn failed(&self) -> bool {
        self.count(Class::Changed) > 0 || !self.errors.is_empty()
    }
}

/// A folder for this run's copies of a revision's files, gone when the run is.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Result<Scratch, String> {
        // (One a run: two runs of one process, as its tests are, don't share one.)
        static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let run = RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("ll-against-{}-{run}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        Ok(Scratch(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn git(root: &str, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

/// Whether git knows `rev` as a commit of the checkout.
fn is_rev(root: &str, rev: &str) -> bool {
    !rev.starts_with('-')
        && git(
            root,
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{rev}^{{commit}}"),
            ],
        )
        .is_ok()
}

/// Writes the files `paths` (from the checkout's root) as revision `rev` has them under `into`; one it has not is
/// left out.
fn checkout(root: &str, rev: &str, paths: &[String], into: &str) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot run git: {e}"))?;
    let mut ask = child.stdin.take().ok_or("git: no input")?;
    let mut answers = BufReader::new(child.stdout.take().ok_or("git: no output")?);
    let broken = |e: std::io::Error| format!("git cat-file: {e}");
    for path in paths {
        ask.write_all(format!("{rev}:{path}\n").as_bytes())
            .map_err(broken)?;
        ask.flush().map_err(broken)?;
        let mut head = String::new();
        answers.read_line(&mut head).map_err(broken)?;
        // "<object> blob <size>", or "<name> missing".
        let fields: Vec<&str> = head.split_whitespace().collect();
        let size = match fields.as_slice() {
            [.., "blob", size] => size
                .parse::<usize>()
                .map_err(|_| format!("git cat-file: {head}"))?,
            [.., "missing"] => continue,
            _ => {
                return Err(format!(
                    "git cat-file: {rev}:{path} is no file ({})",
                    head.trim()
                ));
            }
        };
        let mut data = vec![0u8; size + 1];
        answers.read_exact(&mut data).map_err(broken)?;
        data.pop();
        let to = Path::new(into).join(path);
        if let Some(dir) = to.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::write(&to, data).map_err(|e| format!("{}: {e}", to.display()))?;
    }
    drop(ask);
    let _ = child.wait();
    Ok(())
}

/// How a side's model files get where its manifests name them.
enum Origin {
    /// They are there.
    Here,
    /// Read from a revision into the scratch folder `at`, laid out as the checkout.
    Rev { rev: String, at: String },
    /// A build of some models (its assets root `built`) over the working tree's (`tree`), copied into the scratch.
    Over { built: String, tree: String },
}

/// One side of the comparison: where it is read from, its manifests stacked and its centres.
struct Side {
    paths: Paths,
    origin: Origin,
    root: String,
    stack: Vec<(String, J)>,
    entries: BTreeMap<String, (J, String)>,
    fin: Finishes,
    centres: J,
}

const ASSETS: &str = "web/public/assets";
const CENTRES: &str = "tools/art/blender/centres.json";

fn copy(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::copy(from, to)
        .map(|_| ())
        .map_err(|e| format!("{} to {}: {e}", from.display(), to.display()))
}

/// The manifests of the packs under an assets root: `packs/<name>/manifest.json`, from the root.
fn pack_manifests(assets: &str) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(Path::new(assets).join("packs"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .map(|name| format!("packs/{name}/manifest.json"))
        .filter(|rel| Path::new(assets).join(rel).is_file())
        .collect();
    out.sort();
    out
}

impl Side {
    fn open(what: &str, source: &Source, root: &str, scratch: &Scratch) -> Result<Side, String> {
        let tree = Paths::at(root);
        let at = scratch.0.join(what).to_string_lossy().into_owned();
        let under = |base: &str, assets: String, centres: String| Paths {
            here: normpath(&join(base, "tools/art/blender")),
            assets,
            content: tree.content.clone(),
            furniture: tree.furniture.clone(),
            centres,
        };
        let laid_out = |base: &str| {
            under(
                base,
                normpath(&join(base, ASSETS)),
                normpath(&join(base, CENTRES)),
            )
        };
        let (paths, origin) = match source {
            Source::Tree => (Paths::at(root), Origin::Here),
            Source::Rev(rev) => {
                let listed = git(
                    root,
                    &[
                        "ls-tree",
                        "-r",
                        "--name-only",
                        "-z",
                        rev,
                        "--",
                        &format!("{ASSETS}/manifest.json"),
                        &format!("{ASSETS}/packs"),
                        CENTRES,
                    ],
                )?;
                let files: Vec<String> = String::from_utf8_lossy(&listed)
                    .split('\0')
                    .filter(|p| p.ends_with(".json"))
                    .map(str::to_string)
                    .collect();
                checkout(root, rev, &files, &at)?;
                (
                    laid_out(&at),
                    Origin::Rev {
                        rev: rev.clone(),
                        at: at.clone(),
                    },
                )
            }
            Source::Dir(dir) => {
                if !Path::new(dir).is_dir() {
                    return Err(format!("{dir} is no directory"));
                }
                let dir = normpath(dir);
                let found = if Path::new(&dir).join(ASSETS).is_dir() {
                    laid_out(&dir)
                } else {
                    under(&dir, dir.clone(), normpath(&join(&dir, "centres.json")))
                };
                if Path::new(&found.assets).join("manifest.json").is_file() {
                    (found, Origin::Here)
                } else {
                    // A build of some models: the working tree's manifests with the build's packs over them, and
                    // its centres over the working tree's.
                    let view = laid_out(&at);
                    for (from, rels) in [
                        (
                            &tree.assets,
                            [
                                vec!["manifest.json".to_string()],
                                pack_manifests(&tree.assets),
                            ]
                            .concat(),
                        ),
                        (&found.assets, pack_manifests(&found.assets)),
                    ] {
                        for rel in rels {
                            copy(
                                &Path::new(from).join(&rel),
                                &Path::new(&view.assets).join(&rel),
                            )?;
                        }
                    }
                    let mut centres = super::read_centres(&tree.centres)?;
                    for (k, c) in super::read_centres(&found.centres)?
                        .entries()
                        .unwrap_or(&[])
                    {
                        centres.set(k, c.clone());
                    }
                    let to = Path::new(&view.centres);
                    if let Some(dir) = to.parent() {
                        std::fs::create_dir_all(dir)
                            .map_err(|e| format!("{}: {e}", dir.display()))?;
                    }
                    std::fs::write(to, py::dumps_indent(&centres))
                        .map_err(|e| format!("{}: {e}", to.display()))?;
                    (
                        view,
                        Origin::Over {
                            built: found.assets,
                            tree: tree.assets.clone(),
                        },
                    )
                }
            }
        };
        let stack = super::asset_manifests(&paths)?;
        let (entries, fin) = super::model_entries(&stack);
        let centres = super::read_centres(&paths.centres)?;
        Ok(Side {
            paths,
            origin,
            root: root.to_string(),
            stack,
            entries,
            fin,
            centres,
        })
    }

    /// The file an entry's `url` or `far` names, if it is one of this side's files.
    fn file(&self, key: &str, field: &str) -> Option<String> {
        let (entry, manifest) = self.entries.get(key)?;
        let url = entry
            .get(field)
            .and_then(J::str)
            .filter(|u| !u.is_empty() && !u.contains("://"))?;
        Some(normpath(&join(&dirname(manifest), url)))
    }

    /// Whether a build of some models (`Origin::Over`) has the file an entry of this side names.
    fn built(&self, file: &str) -> bool {
        match &self.origin {
            Origin::Over { built, .. } => Path::new(built)
                .join(relpath(file, &self.paths.assets))
                .is_file(),
            _ => true,
        }
    }

    /// Puts the model files `files` where this side's manifests name them.
    fn fetch(&self, files: &[String]) -> Result<(), String> {
        match &self.origin {
            Origin::Here => Ok(()),
            Origin::Rev { rev, at } => {
                let paths: Vec<String> = files.iter().map(|f| relpath(f, at)).collect();
                checkout(&self.root, rev, &paths, at)
            }
            Origin::Over { built, tree } => {
                for file in files {
                    let rel = relpath(file, &self.paths.assets);
                    let from = [built, tree]
                        .into_iter()
                        .map(|base| Path::new(base).join(&rel))
                        .find(|p| p.is_file());
                    if let Some(from) = from {
                        copy(&from, Path::new(file))?;
                    }
                }
                Ok(())
            }
        }
    }

    /// A manifest's path from the assets root.
    fn manifest_name(&self, path: &str) -> String {
        relpath(path, &self.paths.assets)
    }
}

/// The folders an entry's model is of: its file's under `models/` and its manifest's under `packs/`.
fn families(entry: &J, manifest: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(url) = entry.get("url").and_then(J::str) {
        let dir = dirname(url);
        if let Some(name) = dir
            .rsplit('/')
            .next()
            .filter(|n| !n.is_empty() && *n != "models")
        {
            out.push(name.to_string());
        }
    }
    let dir = dirname(manifest);
    if dirname(&dir).ends_with("packs") {
        out.push(dir.rsplit('/').next().unwrap_or("").to_string());
    }
    out
}

/// A family's name as `--family` compares it: case and underscores aside, with or without `pack` before it.
fn family_name(name: &str) -> String {
    let plain: String = name
        .chars()
        .filter(|c| *c != '_')
        .collect::<String>()
        .to_lowercase();
    plain
        .strip_prefix("pack")
        .filter(|p| !p.is_empty())
        .unwrap_or(&plain)
        .to_string()
}

/// What the rows are compared with: the two sides, and the working tree's content for the gate.
struct Context {
    old: Side,
    new: Side,
    facts: HashMap<String, Vec<Facts>>,
    furniture: Furniture,
    /// port-accepted.json: {row: {metric: why}}.
    accepted: BTreeMap<String, BTreeMap<String, String>>,
}

/// A digit run as `#`, so two findings that differ in their numbers alone are one finding.
fn masked(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut digits = false;
    for c in text.chars() {
        let digit = c.is_ascii_digit() || (digits && c == '.');
        if digit && !digits {
            out.push('#');
        } else if !digit {
            out.push(c);
        }
        digits = digit;
    }
    out
}

/// The texts of `new` that `old` has not, numbers aside (each of `old` answers for one of `new`).
fn newly<'a>(old: &[String], new: &'a [String]) -> Vec<&'a String> {
    let mut have: HashMap<String, usize> = HashMap::new();
    for text in old {
        *have.entry(masked(text)).or_default() += 1;
    }
    new.iter()
        .filter(|text| match have.get_mut(&masked(text)) {
            Some(n) if *n > 0 => {
                *n -= 1;
                false
            }
            _ => true,
        })
        .collect()
}

/// The draw calls in the gate's summary of a model (`check`'s "N draws").
fn draws(summary: &str) -> Option<usize> {
    let words: Vec<&str> = summary.split_whitespace().collect();
    let at = words.iter().position(|w| *w == "draws")?;
    words.get(at.checked_sub(1)?)?.parse().ok()
}

impl Context {
    /// What the gate says of `key` on a side; a file it cannot read is its one error.
    fn gate(&self, side: &Side, key: &str) -> super::Checked {
        let unread = |err: String| super::Checked {
            errors: vec![format!("cannot be read: {err}")],
            warnings: Vec::new(),
            accepted: Vec::new(),
            summary: String::new(),
        };
        let (Some((entry, _)), Some(path)) = (side.entries.get(key), side.file(key, "url")) else {
            return unread("its entry has no file".into());
        };
        super::check(
            key,
            &side.fin,
            entry,
            &path,
            side.file(key, "far").as_deref(),
            self.facts.get(key),
            side.centres.get(key),
            Some(&self.furniture),
        )
        .unwrap_or_else(unread)
    }

    /// The rows that need the model's entry and the content: what the gate says of it that it did not say before,
    /// its draw calls, its size and its functional heights.
    fn in_the_game(&self, key: &str, out: &mut Vec<Finding>) {
        let (old, new) = (self.gate(&self.old, key), self.gate(&self.new, key));
        for (kind, was, is) in [
            ("error", &old.errors, &new.errors),
            ("warning", &old.warnings, &new.warnings),
        ] {
            for text in newly(was, is) {
                // (The budget's target is the triangles' row: "never newly over it".)
                let budget =
                    text.contains(" triangles, aim for ") || text.contains(" triangles, at most ");
                out.push(found(
                    if budget { "triangles" } else { "gate" },
                    format!("the gate's new {kind}: {text}"),
                ));
            }
        }
        if let (Some(was), Some(is)) = (draws(&old.summary), draws(&new.summary))
            && was != is
        {
            out.push(found("draws", format!("{is} draw calls, was {was}")));
        }
        let measured = (|| -> R<()> {
            let side = |s: &Side| -> R<Option<super::Model>> {
                match (s.entries.get(key), s.file(key, "url")) {
                    (Some((entry, _)), Some(path)) => super::model(entry, &path).map(Some),
                    _ => Ok(None),
                }
            };
            let (Some(a), Some(b)) = (side(&self.old)?, side(&self.new)?) else {
                return Ok(());
            };
            let off = (0..3)
                .map(|k| (a.size[k] - b.size[k]).abs())
                .fold(0.0, f64::max);
            if off > BOUNDS {
                let text = |s: [f64; 3]| format!("{:.4} x {:.4} x {:.4} m", s[0], s[1], s[2]);
                out.push(found(
                    "size",
                    format!(
                        "{}, was {} ({} off, at most {})",
                        text(b.size),
                        text(a.size),
                        mm(off),
                        mm(BOUNDS)
                    ),
                ));
            }
            for f in self.facts.get(key).map(Vec::as_slice).unwrap_or(&[]) {
                let ha = super::heights(&a.file, &a.entry, &f.object, &self.furniture, true)?;
                let hb = super::heights(&b.file, &b.entry, &f.object, &self.furniture, true)?;
                let id = py::str_of(&f.id);
                if ha.len() != hb.len() {
                    out.push(found(
                        "heights",
                        format!(
                            "object {id}: {} heights measured, {} before",
                            hb.len(),
                            ha.len()
                        ),
                    ));
                    continue;
                }
                for (x, y) in ha.iter().zip(&hb) {
                    let values = |h: &super::Height| -> Vec<Option<f64>> {
                        vec![h.mesh, h.span.map(|s| s.0), h.span.map(|s| s.1)]
                    };
                    for (p, q) in values(x).into_iter().zip(values(y)) {
                        let off = match (p, q) {
                            (Some(p), Some(q)) => (p - q).abs(),
                            (None, None) => 0.0,
                            _ => f64::INFINITY,
                        };
                        if off > BOUNDS {
                            let text = |v: Option<f64>| {
                                v.map_or("nothing".to_string(), |v| format!("{v:.4} m"))
                            };
                            out.push(found(
                                "heights",
                                format!(
                                    "object {id}: its {} at ({}, {}) is at {}, was {} (at most {} off)",
                                    x.what,
                                    x.at[0],
                                    x.at[1],
                                    text(q),
                                    text(p),
                                    mm(BOUNDS)
                                ),
                            ));
                            break;
                        }
                    }
                }
            }
            Ok(())
        })();
        if let Err(e) = measured {
            out.push(found("file", format!("cannot be measured: {e}")));
        }
    }

    /// The pack entry and the centre of `key`: whether they are the same, with what differs as findings and what
    /// is off within its limit as notes.
    fn entry_differs(&self, key: &str, out: &mut Vec<Finding>, notes: &mut Vec<String>) -> bool {
        let (Some((a, _)), Some((b, _))) = (self.old.entries.get(key), self.new.entries.get(key))
        else {
            return true;
        };
        let mut differs = Vec::new();
        json_differ(
            Some(&without(a, &["perches"])),
            Some(&without(b, &["perches"])),
            "",
            &mut differs,
        );
        let mut same = differs.is_empty();
        if !same {
            out.push(found("pack", format!("its entry: {}", names(&differs))));
        }
        match (a.get("perches"), b.get("perches")) {
            (None, None) => {}
            (Some(p), Some(q)) => {
                let points = |v: &J| -> Vec<Vec<f64>> {
                    v.list()
                        .unwrap_or(&[])
                        .iter()
                        .map(|p| {
                            p.list()
                                .unwrap_or(&[])
                                .iter()
                                .filter_map(|x| x.num().map(|n| n.f()))
                                .collect()
                        })
                        .collect()
                };
                let (p, q) = (points(p), points(q));
                if p.len() != q.len() {
                    same = false;
                    out.push(found(
                        "perches",
                        format!("{} perches, was {}", q.len(), p.len()),
                    ));
                } else {
                    let worst = p
                        .iter()
                        .zip(&q)
                        .enumerate()
                        .map(|(i, (x, y))| {
                            (
                                if x.len() == y.len() {
                                    py::dist(x, y)
                                } else {
                                    f64::INFINITY
                                },
                                i,
                            )
                        })
                        .max_by(|x, y| x.0.total_cmp(&y.0));
                    if let Some((off, i)) = worst.filter(|(off, _)| *off > 0.0) {
                        same = false;
                        if off > PERCH {
                            out.push(found(
                                "perches",
                                format!(
                                    "perch {i} is {} off, at most {}: {:?}, was {:?}",
                                    mm(off),
                                    mm(PERCH),
                                    q[i],
                                    p[i]
                                ),
                            ));
                        } else {
                            notes.push(format!(
                                "perches up to {} off (perch {i}): the birds' recorded day reads them as written",
                                mm(off)
                            ));
                        }
                    }
                }
            }
            (p, _) => {
                same = false;
                let side = if p.is_some() { "old" } else { "new" };
                out.push(found(
                    "perches",
                    format!("only the {side} entry has perches"),
                ));
            }
        }
        let mut differs = Vec::new();
        json_differ(
            self.old.centres.get(key),
            self.new.centres.get(key),
            "its centre",
            &mut differs,
        );
        if !differs.is_empty() {
            same = false;
            out.push(found("centre", names(&differs)));
        }
        !same
    }

    /// Two files of a row compared, by path: one side without the file is a finding.
    /// With `decimated`, as a far version held to that rule, its lowest point to the model's (the file `model`).
    fn files(
        &self,
        old: Option<&String>,
        new: Option<&String>,
        decimated: Option<Option<&String>>,
    ) -> Compared {
        let read = |path: Option<&String>, which: &str| -> Result<Vec<u8>, Finding> {
            let path =
                path.ok_or_else(|| found("file", format!("the {which} entry names no file")))?;
            std::fs::read(path).map_err(|e| {
                let name = relpath(
                    path,
                    &if which == "old" { &self.old } else { &self.new }
                        .paths
                        .assets,
                );
                found("file", format!("the {which} side has no {name}: {e}"))
            })
        };
        match (read(old, "old"), read(new, "new")) {
            (Ok(a), Ok(b)) => match decimated {
                Some(model) => {
                    let model = model.and_then(|path| std::fs::read(path).ok());
                    compare_decimated(&a, &b, model.as_deref())
                }
                None => compare(&a, &b),
            },
            (a, b) => Compared {
                same: false,
                findings: a.err().into_iter().chain(b.err()).collect(),
                numbers: Numbers::default(),
            },
        }
    }

    /// The rows of one key: the model, and its far version if either side has one.
    fn rows(&self, key: &str) -> Vec<Row> {
        let row = |name: String| Row {
            name,
            class: Class::Same,
            findings: Vec::new(),
            accepted: Vec::new(),
            notes: Vec::new(),
            numbers: None,
        };
        let mut main = row(key.to_string());
        let sides = (
            self.old.entries.contains_key(key),
            self.new.entries.contains_key(key),
        );
        if sides != (true, true) {
            main.findings.push(found(
                "pack",
                match sides {
                    (true, false) => "only the old manifests have it".into(),
                    (false, true) => "only the new manifests have it".into(),
                    _ => "no manifest of either side has it".into(),
                },
            ));
            return vec![self.classed(main, false)];
        }
        let entry_differs = self.entry_differs(key, &mut main.findings, &mut main.notes);
        let file = |field: &str| (self.old.file(key, field), self.new.file(key, field));
        let (old, new) = file("url");
        let body = if old.is_none() && new.is_none() {
            // (An entry drawn from elsewhere: nothing here to compare.)
            Compared {
                same: true,
                ..Compared::default()
            }
        } else {
            self.files(old.as_ref(), new.as_ref(), None)
        };
        let model = new;
        let far_name = format!("{key} far");
        let (old, new) = file("far");
        let far = (old.is_some() || new.is_some()).then(|| {
            let found = self.files(old.as_ref(), new.as_ref(), None);
            // A far version listed as decimated is held to its own rule, if the rows it is not held to fail.
            let decimated = self
                .accepted
                .get(&far_name)
                .is_some_and(|a| a.contains_key("decimated"));
            if decimated && !found.same && !found.findings.is_empty() {
                (
                    self.files(old.as_ref(), new.as_ref(), Some(model.as_ref())),
                    true,
                )
            } else {
                (found, false)
            }
        });
        if entry_differs || !body.same || far.as_ref().is_some_and(|(f, _)| !f.same) {
            self.in_the_game(key, &mut main.findings);
        }
        let same = !entry_differs && body.same;
        main.findings.extend(body.findings);
        main.numbers = (!body.same).then_some(body.numbers);
        let mut out = vec![self.classed(main, same)];
        if let Some((found, decimated)) = far {
            let mut far = row(far_name);
            far.numbers = (!found.same).then_some(found.numbers);
            far.findings = found.findings;
            if decimated {
                let why = self.accepted[&far.name]["decimated"].clone();
                far.accepted.push((
                    Finding {
                        metric: "decimated",
                        text: "held to the decimated far versions' rule".into(),
                    },
                    why,
                ));
            }
            out.push(self.classed(far, found.same));
        }
        out
    }

    /// A row with what port-accepted.json accepts of it taken out of its findings, and its class.
    fn classed(&self, mut row: Row, same: bool) -> Row {
        if let Some(accepted) = self.accepted.get(&row.name) {
            let (taken, left): (Vec<Finding>, Vec<Finding>) = std::mem::take(&mut row.findings)
                .into_iter()
                .partition(|f| accepted.contains_key(f.metric));
            row.findings = left;
            row.accepted.extend(
                taken
                    .into_iter()
                    .map(|f| (f.clone(), accepted[f.metric].clone())),
            );
        }
        row.class = if !row.findings.is_empty() {
            Class::Changed
        } else if same && row.accepted.is_empty() {
            Class::Same
        } else {
            Class::Near
        };
        row
    }
}

/// A manifest without its model entries: what the models' rows do not cover.
fn rest_of(manifest: &J) -> J {
    let mut out = manifest.clone();
    if let Some(entries) = manifest.get("entries").and_then(J::entries) {
        out.set(
            "entries",
            J::Obj(
                entries
                    .iter()
                    .filter(|(_, e)| e.get("type") != Some(&J::Str("model".into())))
                    .cloned()
                    .collect(),
            ),
        );
    }
    out
}

/// port-accepted.json, {row: {metric: why}}, and what is wrong with it: a metric that is none, a reason left out.
fn accepted(path: &str) -> (BTreeMap<String, BTreeMap<String, String>>, Vec<String>) {
    let (mut out, mut errors) = (BTreeMap::new(), Vec::new());
    let data = match std::fs::read(path) {
        Ok(data) => data,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (out, errors),
        Err(e) => return (out, vec![format!("{path}: {e}")]),
    };
    let json = match super::json::loads_text(&data) {
        Ok(json @ J::Obj(_)) => json,
        Ok(_) => return (out, vec![format!("{path} is {{key: {{metric: why}}}}")]),
        Err(e) => return (out, vec![format!("{path}: {e}")]),
    };
    for (row, metrics) in json.entries().unwrap_or(&[]) {
        let Some(metrics) = metrics.entries() else {
            errors.push(format!("{row}: its line is {{metric: why}}"));
            continue;
        };
        for (metric, why) in metrics {
            if !METRICS.iter().any(|(m, _)| m == metric) {
                errors.push(format!(
                    "{row}: {} is no metric ({})",
                    py::repr_str(metric),
                    METRICS
                        .iter()
                        .map(|(m, _)| *m)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            } else if !why.str().is_some_and(|w| !w.trim().is_empty()) {
                errors.push(format!(
                    "{row}: {metric} needs its reason: why the new build is right as it is"
                ));
            } else if metric == "decimated" && !row.ends_with(" far") {
                errors.push(format!("{row}: decimated is a far version's (`{row} far`)"));
            } else {
                out.entry(row.clone())
                    .or_insert_with(BTreeMap::new)
                    .insert(metric.clone(), why.str().unwrap_or("").to_string());
            }
        }
    }
    (out, errors)
}

/// Compares the new side with the old (`Options`), every row on as many threads as the machine has.
pub fn against(opts: &Options) -> Result<Outcome, String> {
    let scratch = Scratch::new()?;
    let tree = Paths::at(&opts.root);
    let old = Side::open("old", &opts.old, &opts.root, &scratch)?;
    let new = Side::open("new", &opts.new, &opts.root, &scratch)?;
    let partial = matches!(new.origin, Origin::Over { .. });
    let all: BTreeSet<&String> = old.entries.keys().chain(new.entries.keys()).collect();
    let keys: Vec<String> = if !opts.keys.is_empty() {
        let mut keys = opts.keys.clone();
        keys.sort();
        keys.dedup();
        keys
    } else if let Some(family) = &opts.family {
        let of = |side: &Side, key: &str| {
            side.entries.get(key).is_some_and(|(e, m)| {
                families(e, m)
                    .iter()
                    .any(|f| family_name(f) == family_name(family))
            })
        };
        let keys: Vec<String> = all
            .iter()
            .filter(|k| of(&old, k) || of(&new, k))
            .map(|k| k.to_string())
            .collect();
        if keys.is_empty() {
            let mut known: BTreeSet<String> = BTreeSet::new();
            for side in [&old, &new] {
                known.extend(side.entries.values().flat_map(|(e, m)| families(e, m)));
            }
            return Err(format!(
                "no model is of the family {} ({})",
                py::repr_str(family),
                known.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
        keys
    } else if partial {
        let keys: Vec<String> = all
            .iter()
            .filter(|k| {
                ["url", "far"]
                    .iter()
                    .any(|f| new.file(k, f).is_some_and(|p| new.built(&p)))
            })
            .map(|k| k.to_string())
            .collect();
        if keys.is_empty() {
            return Err(
                "the build has no model file a manifest names: name keys or a family".into(),
            );
        }
        keys
    } else {
        all.iter().map(|k| k.to_string()).collect()
    };
    for side in [&old, &new] {
        let files: Vec<String> = keys
            .iter()
            .flat_map(|k| [side.file(k, "url"), side.file(k, "far")])
            .flatten()
            .collect();
        side.fetch(&files)?;
    }

    let defs = super::content(&tree)?;
    let path = opts
        .accepted
        .clone()
        .unwrap_or_else(|| join(&opts.root, "tools/art/port-accepted.json"));
    let (accepted, mut errors) = accepted(&path);
    let context = Context {
        facts: super::content_objects(&defs)?,
        furniture: Furniture::read(&tree)?,
        old,
        new,
        accepted,
    };

    let next = std::sync::atomic::AtomicUsize::new(0);
    let done: std::sync::Mutex<Vec<Vec<Row>>> =
        std::sync::Mutex::new(keys.iter().map(|_| Vec::new()).collect());
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(keys.len().max(1));
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(key) = keys.get(i) else { break };
                    let rows = context.rows(key);
                    done.lock().unwrap_or_else(|e| e.into_inner())[i] = rows;
                }
            });
        }
    });
    let mut rows: Vec<Row> = done
        .into_inner()
        .unwrap_or_else(|e| e.into_inner())
        .into_iter()
        .flatten()
        .collect();
    let models = keys.len();
    let fars = rows.len() - models;

    // The manifests' own lines (not for keys named one by one): those the models compared are in, and the assets'
    // own when every model is compared; then the centres of what no manifest has.
    let whole = opts.keys.is_empty() && opts.family.is_none() && !partial;
    let mut manifests = 0;
    if opts.keys.is_empty() {
        let of = |side: &Side| -> BTreeMap<String, J> {
            let used: BTreeSet<&String> = keys
                .iter()
                .filter_map(|k| side.entries.get(k).map(|(_, m)| m))
                .collect();
            side.stack
                .iter()
                .filter(|(path, _)| whole || used.contains(path))
                .map(|(path, m)| (side.manifest_name(path), rest_of(m)))
                .collect()
        };
        let (a, b) = (of(&context.old), of(&context.new));
        let names: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
        manifests = names.len();
        for name in names {
            let mut differs = Vec::new();
            json_differ(a.get(name), b.get(name), "", &mut differs);
            let findings = if differs.is_empty() {
                Vec::new()
            } else {
                vec![found(
                    "pack",
                    format!("but for its models' entries: {}", self::names(&differs)),
                )]
            };
            let same = findings.is_empty();
            rows.push(context.classed(
                Row {
                    name: name.clone(),
                    class: Class::Same,
                    findings,
                    accepted: Vec::new(),
                    notes: Vec::new(),
                    numbers: None,
                },
                same,
            ));
        }
    }
    if whole {
        let loose = |side: &Side| -> J {
            J::Obj(
                side.centres
                    .entries()
                    .unwrap_or(&[])
                    .iter()
                    .filter(|(k, _)| !all_has(&context, k))
                    .cloned()
                    .collect(),
            )
        };
        let mut differs = Vec::new();
        json_differ(
            Some(&loose(&context.old)),
            Some(&loose(&context.new)),
            "",
            &mut differs,
        );
        if !differs.is_empty() {
            let row = Row {
                name: "centres.json".into(),
                class: Class::Same,
                findings: vec![found(
                    "centre",
                    format!("of keys no manifest has: {}", names(&differs)),
                )],
                accepted: Vec::new(),
                notes: Vec::new(),
                numbers: None,
            };
            rows.push(context.classed(row, false));
        }
    }

    // A line of port-accepted.json no row needs hides the next finding: an error, as the gate's `accept`.
    for (name, metrics) in &context.accepted {
        let Some(row) = rows.iter().find(|r| r.name == *name) else {
            if whole {
                errors.push(format!(
                    "{name}: no model, far version or manifest of that name is compared"
                ));
            }
            continue;
        };
        // (But `decimated`: what the far version is, whether this build needs its rule or not.)
        for metric in metrics.keys().filter(|m| *m != "decimated") {
            if !row.accepted.iter().any(|(f, _)| f.metric == metric) {
                errors.push(format!(
                    "{name}: {metric} is accepted, but the row passes without it: take it out"
                ));
            }
        }
    }

    let listed = !opts.keys.is_empty() || opts.family.is_some();
    let mut text = String::new();
    for row in &rows {
        let class = match row.class {
            Class::Same => "same",
            Class::Near => "near",
            Class::Changed => "CHANGED",
        };
        if row.class == Class::Same && !listed {
            continue;
        }
        text += &format!("{class:8} {}", row.name);
        if let Some(numbers) = row.numbers.as_ref().filter(|_| row.class != Class::Same) {
            text += &format!("  ({})", numbers.text());
        }
        text.push('\n');
        for f in &row.findings {
            text += &format!("         {}: {}\n", f.metric, f.text);
        }
        for (f, why) in &row.accepted {
            text += &format!("         accepted: {}: {} ({why})\n", f.metric, f.text);
        }
        for note in &row.notes {
            text += &format!("         note: {note}\n");
        }
    }
    for e in &errors {
        text += &format!("error: port-accepted.json: {e}\n");
    }
    let outcome = Outcome {
        rows,
        errors,
        text: String::new(),
    };
    text += &format!(
        "{models} models, {fars} far versions and {manifests} manifests: {} same, {} near, {} CHANGED\n",
        outcome.count(Class::Same),
        outcome.count(Class::Near),
        outcome.count(Class::Changed)
    );
    Ok(Outcome { text, ..outcome })
}

/// Whether either side's manifests have a model entry `key`.
fn all_has(context: &Context, key: &str) -> bool {
    context.old.entries.contains_key(key) || context.new.entries.contains_key(key)
}

/// `cargo check-glb --against <rev|dir> [keys… | --family <f>] [--dir <built>]`: the exit code.
pub fn main(args: &[String]) -> i32 {
    let root = super::repo_root();
    let mut opts = Options {
        root: root.clone(),
        old: Source::Tree,
        new: Source::Tree,
        keys: Vec::new(),
        family: None,
        accepted: None,
    };
    let mut args = args.iter();
    let mut held = false;
    while let Some(arg) = args.next() {
        let mut value = |what: &str| {
            args.next()
                .filter(|v| !v.starts_with("--"))
                .cloned()
                .ok_or_else(|| format!("{arg} takes {what}"))
        };
        let taken = match arg.as_str() {
            "--against" => value("a revision or a directory").and_then(|v| {
                held = true;
                // (A revision first: a directory of a revision's name is written ./name.)
                opts.old = if is_rev(&root, &v) {
                    Source::Rev(v)
                } else if Path::new(&v).is_dir() {
                    Source::Dir(v)
                } else {
                    return Err(format!(
                        "{v} is neither a revision git knows nor a directory"
                    ));
                };
                Ok(())
            }),
            "--dir" => value("a directory").map(|v| opts.new = Source::Dir(v)),
            "--family" => value("a family").map(|v| opts.family = Some(v)),
            key if key.starts_with("model.")
                || key.starts_with("item.")
                || key.starts_with("tool.") =>
            {
                opts.keys.push(key.to_string());
                Ok(())
            }
            other => Err(format!(
                "{other}: --against takes keys (model., item., tool.), --family <f> and --dir <built>"
            )),
        };
        if let Err(e) = taken {
            eprintln!("{e}");
            return 2;
        }
    }
    if !held {
        eprintln!("--against takes a revision or a directory");
        return 2;
    }
    if !opts.keys.is_empty() && opts.family.is_some() {
        eprintln!("name keys or a family, not both");
        return 2;
    }
    let started = std::time::Instant::now();
    match against(&opts) {
        Ok(outcome) => {
            print!("{}", outcome.text);
            eprintln!("compared in {:.1} s", started.elapsed().as_secs_f64());
            outcome.failed() as i32
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn findings_that_differ_in_their_numbers_alone_are_one() {
        assert_eq!(
            masked("4321 triangles, aim for 4000 (1.25 m)"),
            "# triangles, aim for # (# m)"
        );
        assert_eq!(
            masked("wing.L is saved at scale 0."),
            "wing.L is saved at scale #"
        );
        let old = vec!["4 draw calls, aim for 3 at most".to_string(), "a".into()];
        let new = vec![
            "5 draw calls, aim for 3 at most".to_string(),
            "b".into(),
            "a".into(),
            "a".into(),
        ];
        assert_eq!(newly(&old, &new), [&"b".to_string(), &"a".to_string()]);
        assert!(newly(&new, &old).is_empty());
    }

    /// The draw calls are read from the gate's own line: the coin's, as the gate writes it today.
    #[test]
    fn the_draw_calls_are_the_gate_s() {
        assert_eq!(
            draws(
                "  305.0 KB   7616 tris 4 draws 26 materials  1.89 x 1.90 x 0.92 m  3 moving parts, 2 clips"
            ),
            Some(4)
        );
        assert_eq!(draws(""), None);
        let fin = Finishes {
            plain: HashMap::new(),
            see_through: HashSet::new(),
        };
        let path = join(
            &super::super::repo_root(),
            "web/public/assets/models/held/coin.glb",
        );
        let entry =
            super::super::json::loads_text(br#"{"type": "model", "url": "coin.glb"}"#).unwrap();
        let checked =
            super::super::check("tool.coin", &fin, &entry, &path, None, None, None, None).unwrap();
        assert_eq!(draws(&checked.summary), Some(1), "{}", checked.summary);
    }

    #[test]
    fn a_family_is_a_folder_of_models_or_packs_however_it_is_written() {
        let j = |text: &str| super::super::json::loads_text(text.as_bytes()).unwrap();
        assert_eq!(
            families(
                &j(r#"{"url": "../../models/garden/gnome.glb"}"#),
                "/a/assets/packs/gardenDecor/manifest.json"
            ),
            ["garden", "gardenDecor"]
        );
        assert!(
            families(
                &j(r#"{"url": "models/rock.glb"}"#),
                "/a/assets/manifest.json"
            )
            .is_empty()
        );
        for (written, folder) in [
            ("garden_decor", "gardenDecor"),
            ("gloomy", "pack_gloomy"),
            ("pack_gloomy", "pack_gloomy"),
        ] {
            assert_eq!(family_name(written), family_name(folder));
        }
        assert_ne!(family_name("kitchen"), family_name("kitchen_cooking"));
        assert_eq!(family_name("pack"), "pack");
    }

    #[test]
    fn two_values_differ_where_their_paths_say() {
        let j = |text: &str| super::super::json::loads_text(text.as_bytes()).unwrap();
        let differ = |a: &str, b: &str| {
            let mut out = Vec::new();
            json_differ(Some(&j(a)), Some(&j(b)), "", &mut out);
            out
        };
        assert!(differ(r#"{"a": 1, "b": [1, 2.0]}"#, r#"{"b": [1.0, 2], "a": 1.0}"#).is_empty());
        assert_eq!(
            differ(
                r#"{"a": {"x": 1, "y": [1, 2]}, "gone": true}"#,
                r#"{"a": {"x": 2, "y": [1, 3]}, "new": "n"}"#
            ),
            [
                "a.x: 1.0 is now 2.0",
                "a.y[1]: 2.0 is now 3.0",
                "gone (true) is gone",
                "new (\"n\") is new"
            ]
        );
        assert_eq!(
            differ("[1, 2]", "[1, 2, 3]"),
            [": [1.0,2.0] is now [1.0,2.0,3.0]"]
        );
    }

    #[test]
    fn every_metric_has_its_name_once_and_fruit_is_fitted() {
        let names: HashSet<&str> = METRICS.iter().map(|(m, _)| *m).collect();
        assert_eq!(names.len(), METRICS.len());
        assert!(fruit("fruit2") && fruit("apple_blush") && fruit("lemon") && !fruit("leaf"));
    }
}
