//! `cargo check-glb [keys…] [--keys] [--preview]`: the gate for the game's glTF models
//! (docs/design/objects.md, "Validation"), was tools/art/blender/check_glb.py, and prints what it printed.
//!
//! It reads every `.glb` a model entry of the manifests uses (the base manifest and its packs, later ones
//! overriding) with that entry, checks what the game needs of it, and that every model the content draws
//! has a file. Errors exit 1; warnings are printed.
//!
//!   cargo check-glb                        every .glb the manifests use
//!   cargo check-glb model.garden.gnome …   only these keys (model., item., tool.)
//!   cargo check-glb --keys                 the keys a build may write (gamekit/manifest.py `known_keys`)
//!   cargo check-glb --preview              also re-imports each file in Blender and renders it to
//!                                          tools/art/blender/previews/<family>/rt_<name>.png (gamekit/review.py)
//!
//! Run the Khronos glTF validator too (the standard says how); this checks the game's own rules.
//!
//! What it prints is check_glb.py's to the byte: JSON objects are read in file order, numbers as Python
//! has them (an int stays an int), and formatted, `repr`'d and `math.hypot`'d as Python does (`py`); and
//! since, the functional heights' warnings (`heights`: seats, tops, mattresses and lamps against the model).
//! `measures` gives `cargo review` the same numbers.

mod glb;
mod json;
mod py;

use glb::{Glb, R, accessor, first_value, image_size, index, item, key, parse_glb, values};
use py::{J, Num, dist, dirname, fixed, hypot, join, normpath, percent, relpath, repr, repr_str, signed, str_of};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::f64::consts::PI;

pub use py::repr_str as py_repr_str;

/// Triangles at full detail by the model's largest side (m): (up to, target, most). About the
/// primitive parts' median and 90th percentile in each class, so a model costs what the
/// primitive models cost today (docs/design/objects.md, "Budgets").
const TRIANGLES: [(f64, i64, i64); 4] = [(0.35, 1500, 3000), (1.0, 4000, 8000), (2.5, 8000, 16000), (f64::INFINITY, 12000, 24000)];
/// Draw calls per model (each its own in the sun's shadow map too): aim for 1-3.
const DRAWS: (usize, usize) = (3, 4);
/// Extensions the game's loader can read (no Draco: its decoder isn't shipped).
const EXTENSIONS: [&str; 7] = ["KHR_materials_emissive_strength", "KHR_mesh_quantization", "KHR_texture_transform", "KHR_texture_basisu", "EXT_meshopt_compression", "KHR_materials_ior", "KHR_materials_specular"];
/// Largest texture side (px).
const TEXTURE_MAX: u32 = 1024;
/// The wall's face lies this far into a footprint's back tile (m): a wall-standing thing's back is there.
const WALL: f64 = 0.07;
/// A house's floor is drawn 1 cm up: a flat part on the floor must reach this high (m) to show over it.
const FLOOR_TOP: f64 = 0.012;
/// A use's clip longer than this (s) reads as slow ("about 1 s"); loops and ambient clips keep their own pace.
const USE_CLIP: f64 = 3.0;
/// How far (m) a model may reach past its footprint at rest (its parts' boxes: a handle, a knob), and how far a
/// part that goes round may sweep past it (measured from its vertices, so tighter: a fan's blades whose axis
/// stood 3.3 cm off the middle swept 1.97 cm past).
const OVERHANG: f64 = 0.02;
const SWEEP: f64 = 0.01;
/// How far (m) a model's middle at rest may be off its `centre` (centres.json).
const OFF_CENTRE: f64 = 0.02;
/// A bird's `bird` (tools/art/blender/families/birds.py; the renderer flies it by code): its length bill to tail
/// tip and wingspan (m, both needed), its body's middle over its feet (`perchHeight`, m) and how far its perched
/// torso is raised from level (`pitch`, degrees).
const BIRD_KEYS: [&str; 4] = ["length", "wingspan", "perchHeight", "pitch"];
/// A bird's rigid parts, the nodes the renderer moves by name (a mesh's own node or one above it).
const BIRD_PARTS: [&str; 5] = ["body", "head", "tail", "wing.L", "wing.R"];
/// How far (a share) a bird's file may be off the length and wingspan its `bird` says.
const BIRD_FIT: f64 = 0.05;
/// How far (m) a perch (`perches`: where the game's birds sit on a tree or a hedge) may be outside the model's box.
const PERCH_OUT: f64 = 0.01;
/// What a model with `perches` is, its entry's `habitat` (families/trees.py, plants.py): a broadleaved or fruit tree,
/// a conifer, a bush or shrub, or a hedge.
const HABITATS: [&str; 4] = ["tree", "conifer", "bush", "hedge"];

/// Where the gate reads: the repo's assets, content and tools/art/blender/centres.json.
pub struct Paths {
    /// tools/art/blender
    pub here: String,
    /// web/public/assets
    pub assets: String,
    /// web/public/content
    pub content: String,
    /// web/src/render/shared/characters/furniture.json: where residents sit and lie on each object (`SEATS`,
    /// `BEDS`, `BUNKS` in characters.ts), for the height checks.
    pub furniture: String,
    /// tools/art/blender/centres.json: where each Blender-built model is centred at rest (its Model's
    /// `centre`, (0, 0) for most): {key: [x, depth]}, for this gate only (the game doesn't read it). A build
    /// writes its models' (gamekit/family.py), `cargo art --centres` every family's. A key that isn't in it
    /// isn't built here (a third-party file): its centre isn't checked.
    pub centres: String,
}

impl Paths {
    /// The paths under a checkout's root.
    pub fn at(root: &str) -> Paths {
        let root = normpath(root);
        Paths {
            here: normpath(&join(&root, "tools/art/blender")),
            assets: normpath(&join(&root, "web/public/assets")),
            content: normpath(&join(&root, "web/public/content")),
            furniture: normpath(&join(&root, "web/src/render/shared/characters/furniture.json")),
            centres: normpath(&join(&root, "tools/art/blender/centres.json")),
        }
    }

    /// This checkout's.
    pub fn repo() -> Paths {
        Paths::at(&repo_root())
    }
}

/// The checkout this crate is in.
pub fn repo_root() -> String {
    normpath(&join(env!("CARGO_MANIFEST_DIR"), "../.."))
}

fn read_json(path: &str) -> Result<J, String> {
    let data = std::fs::read(path).map_err(|e| py::os_error(&e, path))?;
    json::loads_text(&data)
}

/// {key: [x, depth]}: centres.json (empty if there's none yet).
pub fn read_centres(path: &str) -> Result<J, String> {
    match std::fs::read(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(J::Obj(Vec::new())),
        Err(e) => Err(py::os_error(&e, path)),
        Ok(data) => json::loads_text(&data),
    }
}

/// centres.json's text for `centres` ({key: (x, depth)}), as gamekit/manifest.py `write_centres` writes it:
/// one key a line, sorted, each to 0.1 mm (`round(v, 4) + 0.0`: no -0.0).
pub fn centres_text(centres: &BTreeMap<String, [f64; 2]>) -> String {
    let round4 = |v: f64| format!("{v:.4}").parse::<f64>().unwrap() + 0.0;
    let lines: Vec<String> = centres
        .iter()
        .map(|(key, c)| format!("  {}: [{}, {}]", py::json_str(key), py::float_repr(round4(c[0])), py::float_repr(round4(c[1]))))
        .collect();
    format!("{{\n{}\n}}\n", lines.join(",\n"))
}

/// [(path, manifest)] for a manifest and the packs it lists, depth first, in order. A manifest that can't
/// be read, or packs nested more than 20 deep (a pack listing itself), is an error naming it.
pub fn manifests(paths: &Paths, path: &str) -> Result<Vec<(String, J)>, String> {
    fn stack(paths: &Paths, path: &str, depth: usize, out: &mut Vec<(String, J)>) -> Result<(), String> {
        let name = || relpath(&normpath(path), &dirname(&paths.assets));
        if depth > 20 {
            return Err(format!("{}: packs nested too deep", name()));
        }
        let m = read_json(path).map_err(|e| format!("{}: {e}", name()))?;
        let packs: Vec<J> = m.get("packs").and_then(J::list).map(|l| l.to_vec()).unwrap_or_default();
        out.push((path.to_string(), m));
        for pack in packs {
            let pack = pack.str().ok_or_else(|| format!("expected str, bytes or os.PathLike object, not {}", glb::type_name(&pack)))?;
            stack(paths, &normpath(&join(&dirname(path), pack)), depth + 1, out)?;
        }
        Ok(())
    }
    let mut out = Vec::new();
    stack(paths, path, 0, &mut out)?;
    Ok(out)
}

/// `manifests` from the assets' own manifest (web/public/assets/manifest.json).
pub fn asset_manifests(paths: &Paths) -> Result<Vec<(String, J)>, String> {
    manifests(paths, &join(&paths.assets, "manifest.json"))
}

/// The game's content: base.json and the files its `include` lists, merged as the game merges them
/// (sim_core::merge_content: each file's lists after the ones before).
pub struct Content {
    pub objects: Vec<J>,
    pub items: Vec<J>,
    pub categories: Vec<J>,
}

/// Reads the content; a file that can't be read is an error naming it.
pub fn content(paths: &Paths) -> Result<Content, String> {
    let read = |name: &str| -> Result<J, String> {
        let data = read_json(&join(&paths.content, name)).map_err(|e| format!("web/public/content/{name}: {e}"))?;
        match data {
            J::Obj(_) => Ok(data),
            _ => Err(format!("web/public/content/{name}: a content file is a JSON object")),
        }
    };
    let base = read("base.json")?;
    let mut files = vec![base.clone()];
    for name in base.get("include").and_then(J::list).unwrap_or(&[]) {
        files.push(read(name.str().ok_or("expected str, bytes or os.PathLike object")?)?);
    }
    let all = |k: &str| -> Vec<J> { files.iter().flat_map(|f| f.get(k).filter(|v| v.truthy()).and_then(J::list).unwrap_or(&[]).iter().cloned()).collect() };
    Ok(Content { objects: all("objects"), items: all("items"), categories: all("buyCategories") })
}

/// The model a content object (`prefix` model) or item (item) draws, as the renderer picks it
/// (`def.model ?? `model.${id}``): its `model`, or `<prefix>.<id>`. An empty `model` stays empty (an error).
fn model_key(d: &J, prefix: &str) -> R<String> {
    match d.opt("model") {
        Some(m) => Ok(m.key()),
        None => Ok(format!("{prefix}.{}", str_of(key(d, "id")?))),
    }
}

/// {model key: what draws it ('object fridge')}: the model each content object and item draws
/// (`model_key`). The game's own models (the rock, the marker, the street's) are drawn by code and aren't here.
fn content_keys(defs: &Content) -> R<BTreeMap<String, String>> {
    let mut keys = BTreeMap::new();
    for (kind, prefix, found) in [("object", "model", &defs.objects), ("item", "item", &defs.items)] {
        for d in found {
            let what = format!("{kind} {}", str_of(key(d, "id")?));
            keys.entry(model_key(d, prefix)?).or_insert(what);
        }
    }
    Ok(keys)
}

/// The model keys a build may write (gamekit/manifest.py `known_keys`, without a pack of its own): every
/// content object's and item's model, and every model entry of the manifests.
pub fn known_keys(paths: &Paths) -> Result<BTreeSet<String>, String> {
    let mut known: BTreeSet<String> = content_keys(&content(paths)?)?.into_keys().filter(|k| !k.is_empty()).collect();
    for (_path, m) in asset_manifests(paths)? {
        for (k, e) in m.get("entries").filter(|e| e.truthy()).and_then(J::entries).unwrap_or(&[]) {
            if e.get("type") == Some(&J::Str("model".into())) {
                known.insert(k.clone());
            }
        }
    }
    Ok(known)
}

/// The facts of a content object that draws a model, for the checks against what the game does with it.
struct Facts {
    id: J,
    footprint: J,
    layer: J,
    outside: bool,
    stairs: bool,
    /// The disc a thing on a top takes (content.ts `radius`: default 0.12, kept within 0.02-0.45).
    radius: f64,
    /// The content object itself, for its functional heights (`seat`, `surface`, `light`, its uses' poses: `heights`).
    object: J,
}

/// {model key: [facts of each content object that draws it]}: where the wall's face is (a footprint's
/// back, `WALL` in), the footprint, the disc a thing on a top takes.
fn content_objects(defs: &Content) -> R<HashMap<String, Vec<Facts>>> {
    let mut out: HashMap<String, Vec<Facts>> = HashMap::new();
    let none = J::Obj(Vec::new());
    for o in &defs.objects {
        let category = o.get("category").unwrap_or(&J::Null);
        // {c.get('id'): c}: the last with that id.
        let c = defs.categories.iter().rev().find(|c| c.get("id").unwrap_or(&J::Null) == category).unwrap_or(&none);
        let group_id = o.get("group").unwrap_or(&J::Null);
        let groups = c.get("groups").filter(|g| g.truthy()).and_then(J::list).unwrap_or(&[]);
        let group = groups.iter().find(|g| g.get("id").unwrap_or(&J::Null) == group_id).unwrap_or(&none);
        let outside = group.get("outside").or(c.get("outside")).map(J::truthy).unwrap_or(false);
        let r = match o.get("radius") {
            None => 0.12,
            Some(r) => r.num().ok_or_else(|| format!("'<' not supported between instances of '{}' and 'float'", glb::type_name(r)))?.f(),
        };
        let radius = { let m = if r < 0.45 { r } else { 0.45 }; if m > 0.02 { m } else { 0.02 } };
        out.entry(model_key(o, "model")?).or_default().push(Facts {
            id: key(o, "id")?.clone(),
            footprint: o.get("footprint").cloned().unwrap_or(J::Arr(vec![J::Int(1), J::Int(1)])),
            layer: o.get("layer").cloned().unwrap_or(J::Str("floor".into())),
            outside,
            stairs: o.get("stairs").is_some_and(J::truthy),
            radius,
            object: o.clone(),
        });
    }
    Ok(out)
}

/// The finishes the manifests have: {key: plain (untextured, opaque)}, and those see-through (`alpha` under
/// 1, the glass).
struct Finishes {
    plain: HashMap<String, bool>,
    see_through: HashSet<String>,
}

impl Finishes {
    fn see_through(&self, finish: Option<&J>) -> bool {
        finish.and_then(J::str).is_some_and(|f| self.see_through.contains(f))
    }
}

type Matrix = [Num; 16];
const IDENTITY: Matrix = {
    let (o, z) = (Num::I(1), Num::I(0));
    [o, z, z, z, z, o, z, z, z, z, o, z, z, z, z, o]
};

/// A point `v` through a column-major 4x4 matrix `m` (glTF axes: the game's x is its -x).
fn apply(m: &Matrix, v: &[Num]) -> [Num; 3] {
    [
        m[0] * v[0] + m[4] * v[1] + m[8] * v[2] + m[12],
        m[1] * v[0] + m[5] * v[1] + m[9] * v[2] + m[13],
        m[2] * v[0] + m[6] * v[1] + m[10] * v[2] + m[14],
    ]
}

fn applied(m: &Matrix, v: &[Num]) -> [f64; 3] {
    apply(m, v).map(Num::f)
}

fn nums(v: &J, what: usize) -> R<Vec<Num>> {
    let l = v.list().ok_or_else(|| format!("cannot unpack non-iterable {} object", glb::type_name(v)))?;
    let out: Option<Vec<Num>> = l.iter().map(J::num).collect();
    let out = out.ok_or("unsupported operand type(s)")?;
    if out.len() != what && what != 0 {
        return Err(if out.len() > what { format!("too many values to unpack (expected {what})") } else { format!("not enough values to unpack (expected {what}, got {})", out.len()) });
    }
    Ok(out)
}

/// A node's index (`nodes[i]`, an int of the file, which must be one of its nodes).
fn node_index(nodes: &J, i: &J) -> R<usize> {
    index(nodes, i)?;
    match i {
        J::Int(i) if *i >= 0 => Ok(*i as usize),
        _ => Err("list index out of range".into()),
    }
}

/// A node an animation channel names (`c['target']['node']`), as an index.
fn target_node(node: &J) -> usize {
    match node {
        J::Int(i) if *i >= 0 => *i as usize,
        _ => usize::MAX,
    }
}

/// `nodes[i].get('name', i)`, as printed.
fn node_name(nodes: &J, i: usize) -> String {
    match nodes.at(i as i64).and_then(|n| n.get("name")) {
        Some(name) => str_of(name),
        None => i.to_string(),
    }
}

/// A mesh node of the default scene: its mesh, its world matrix, its moving part (the nearest node above
/// it, or itself, that an animation moves; None: the still body) and the node.
struct Placed {
    mesh: J,
    m: Matrix,
    part: Option<usize>,
    node: usize,
}

/// Translations, rotations and scales (by node and path) that put nodes elsewhere than at rest (a part grown,
/// `grown`; a bed pulled out, `open_pose`).
type Pose = HashMap<(usize, &'static str), Vec<Num>>;

/// The mesh nodes of the default scene (`Placed`), their parts where `pose` puts them; `worlds`, if given,
/// gets every node's world matrix.
fn node_matrices(gltf: &J, pose: Option<&Pose>, worlds: Option<&mut HashMap<usize, Matrix>>) -> R<Vec<Placed>> {
    placed_in(gltf, &moved_nodes(gltf)?, pose, worlds)
}

/// `node_matrices` with `parts` as the nodes that are parts (`Placed::part`: the nearest of them above a mesh,
/// or its own node), in place of the nodes the animations move (a bird's, by name).
fn placed_in(gltf: &J, parts: &HashSet<usize>, pose: Option<&Pose>, mut worlds: Option<&mut HashMap<usize, Matrix>>) -> R<Vec<Placed>> {
    let empty = J::Arr(Vec::new());
    let nodes = gltf.get("nodes").unwrap_or(&empty);
    let scenes = gltf.get("scenes").cloned().unwrap_or(J::Arr(vec![J::Obj(Vec::new())]));
    let scene = index(&scenes, gltf.get("scene").unwrap_or(&J::Int(0)))?;

    fn local(n: &J, i: usize, pose: Option<&Pose>) -> R<Matrix> {
        if let Some(m) = n.get("matrix") {
            let m = nums(m, 0)?;
            return m.get(..16).and_then(|m| m.try_into().ok()).ok_or_else(|| "list index out of range".into());
        }
        let (z, o) = (Num::I(0), Num::I(1));
        let t = match (pose.and_then(|p| p.get(&(i, "translation"))), n.get("translation")) {
            (Some(t), _) if t.len() == 3 => t.clone(),
            (_, Some(t)) => nums(t, 3)?,
            _ => vec![z, z, z],
        };
        let q = match (pose.and_then(|p| p.get(&(i, "rotation"))), n.get("rotation")) {
            (Some(q), _) if q.len() == 4 => q.clone(),
            (_, Some(q)) => nums(q, 4)?,
            _ => vec![z, z, z, o],
        };
        let s = match pose.and_then(|p| p.get(&(i, "scale"))) {
            Some(s) if s.len() != 3 => return Err(format!("too many values to unpack (expected 3, got {})", s.len())),
            Some(s) => s.clone(),
            None => match n.get("scale") { Some(s) => nums(s, 3)?, None => vec![o, o, o] },
        };
        let (qx, qy, qz, qw) = (q[0], q[1], q[2], q[3]);
        let two = Num::I(2);
        let r = [
            o - two * (qy * qy + qz * qz), two * (qx * qy + qz * qw), two * (qx * qz - qy * qw),
            two * (qx * qy - qz * qw), o - two * (qx * qx + qz * qz), two * (qy * qz + qx * qw),
            two * (qx * qz + qy * qw), two * (qy * qz - qx * qw), o - two * (qx * qx + qy * qy),
        ];
        Ok([r[0] * s[0], r[1] * s[0], r[2] * s[0], z, r[3] * s[1], r[4] * s[1], r[5] * s[1], z, r[6] * s[2], r[7] * s[2], r[8] * s[2], z, t[0], t[1], t[2], o])
    }

    fn mul(a: &Matrix, b: &Matrix) -> Matrix {
        let mut out = [Num::I(0); 16];
        for c in 0..4 {
            for r in 0..4 {
                let mut sum = Num::I(0);
                for k in 0..4 {
                    sum = sum + a[k * 4 + r] * b[c * 4 + k];
                }
                out[c * 4 + r] = sum;
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn walk(nodes: &J, i: usize, parent: &Matrix, part: Option<usize>, pose: Option<&Pose>, parts: &HashSet<usize>, worlds: &mut Option<&mut HashMap<usize, Matrix>>, out: &mut Vec<Placed>, depth: usize) -> R<()> {
        if depth > 990 {
            return Err("maximum recursion depth exceeded".into());
        }
        let n = nodes.at(i as i64).ok_or("list index out of range")?;
        let m = mul(parent, &local(n, i, pose)?);
        if let Some(w) = worlds.as_deref_mut() {
            w.insert(i, m);
        }
        let part = if parts.contains(&i) { Some(i) } else { part };
        if let Some(mesh) = n.get("mesh") {
            out.push(Placed { mesh: mesh.clone(), m, part, node: i });
        }
        for c in n.get("children").and_then(J::list).unwrap_or(&[]) {
            walk(nodes, node_index(nodes, c)?, &m, part, pose, parts, worlds, out, depth + 1)?;
        }
        Ok(())
    }

    let mut out = Vec::new();
    for i in scene.get("nodes").and_then(J::list).unwrap_or(&[]) {
        walk(nodes, node_index(nodes, i)?, &IDENTITY, None, pose, parts, &mut worlds, &mut out, 0)?;
    }
    Ok(out)
}

/// The nodes the file's animations move.
fn moved_nodes(gltf: &J) -> R<HashSet<usize>> {
    let mut out = HashSet::new();
    for a in gltf.get("animations").and_then(J::list).unwrap_or(&[]) {
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            if let Some(node) = key(c, "target")?.get("node") {
                out.insert(target_node(node));
            }
        }
    }
    Ok(out)
}

/// A JSON number (not a bool), as f64.
fn json_number(v: &J) -> Option<f64> {
    match v {
        J::Int(i) => Some(*i as f64),
        J::Float(f) => Some(*f),
        _ => None,
    }
}

/// What is wrong with a bird entry's `bird` itself (`BIRD_KEYS`): no file needed.
fn bird_fields(bird: &J) -> Vec<String> {
    let Some(fields) = bird.entries() else {
        return vec![format!("`bird` is {{\"length\", \"wingspan\", ...}} (metres), not {}", repr(bird))];
    };
    let mut errors = Vec::new();
    let mut unknown: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).filter(|k| !BIRD_KEYS.contains(k)).collect();
    unknown.sort();
    if !unknown.is_empty() {
        errors.push(format!("`bird`: unknown {} (it takes {})", unknown.join(", "), BIRD_KEYS.join(", ")));
    }
    for k in BIRD_KEYS {
        let Some(v) = bird.get(k) else {
            if k == "length" || k == "wingspan" {
                errors.push(format!("`bird` has no `{k}`"));
            }
            continue;
        };
        let ok = match (k, json_number(v)) {
            ("pitch", Some(p)) => (-90.0..=90.0).contains(&p),
            (_, Some(m)) => m > 0.0,
            _ => false,
        };
        if !ok {
            let what = if k == "pitch" { "angle (degrees, -90 to 90)" } else { "length (metres, over 0)" };
            errors.push(format!("`bird.{k}` {} is no {what}", repr(v)));
        }
    }
    errors
}

/// A bird's file as `bird_errors` needs it: which of `BIRD_PARTS` it has, each part's vertices at rest (game axes),
/// and the parts saved at scale 0.
struct BirdParts {
    names: BTreeSet<String>,
    points: HashMap<String, Vec<[f64; 3]>>,
    flat: Vec<String>,
}

fn bird_parts(g: &Glb) -> R<BirdParts> {
    let nodes = g.gltf.get("nodes").and_then(J::list).unwrap_or(&[]);
    let name = |i: usize| nodes.get(i).and_then(|n| n.get("name")).and_then(J::str).unwrap_or("");
    let mut parts = BirdParts { names: BTreeSet::new(), points: HashMap::new(), flat: Vec::new() };
    let mut named = HashSet::new();
    for (i, n) in nodes.iter().enumerate() {
        if BIRD_PARTS.contains(&name(i)) {
            named.insert(i);
            parts.names.insert(name(i).to_string());
            if n.get("scale").and_then(J::list).is_some_and(|s| s.iter().any(|v| json_number(v).is_some_and(|v| v.abs() < 1e-6))) {
                parts.flat.push(name(i).to_string());
            }
        }
    }
    // Each mesh in the part it is in: its own node, or the nearest one above it named as a part.
    for p in placed_in(&g.gltf, &named, None, None)? {
        let Some(part) = p.part else { continue };
        let out = parts.points.entry(name(part).to_string()).or_default();
        for prim in primitives(&g.gltf, &p.mesh)? {
            for v in positions(g, prim)? {
                let q = apply(&p.m, &v);
                out.push([-q[0].f(), q[1].f(), q[2].f()]);
            }
        }
    }
    Ok(parts)
}

/// What is wrong with a bird (its `bird`, already sound by `bird_fields`) against its file: its parts, its wings
/// spread to its sides at scale 1, its wingspan across them, its length from bill (`head`) to tail tip (`tail`),
/// and its body's middle within its height. `dims` is the file's size (game width, height, depth).
fn bird_errors(bird: &J, parts: &BirdParts, dims: [f64; 3]) -> Vec<String> {
    let mut errors = Vec::new();
    let missing: Vec<&str> = BIRD_PARTS.iter().copied().filter(|p| !parts.names.contains(*p)).collect();
    if !missing.is_empty() {
        errors.push(format!("a bird's parts are nodes named {}: it has no {}", BIRD_PARTS.join(", "), missing.join(", ")));
    }
    for name in &parts.flat {
        errors.push(format!("{name} is saved at scale 0: the renderer bakes each part as the file has it (the wings spread, at scale 1) and hides them itself"));
    }
    let wanted = |k: &str| bird.get(k).and_then(json_number).unwrap_or(0.0);
    let off = |got: f64, want: f64| (got - want).abs() > BIRD_FIT * want;
    let (span, length) = (wanted("wingspan"), wanted("length"));
    if off(dims[0], span) {
        errors.push(format!("{} m across its spread wings, its `bird.wingspan` says {} m (within {})", fixed(dims[0], 3), fixed(span, 3), percent(BIRD_FIT)));
    }
    let (head, tail) = (parts.points.get("head"), parts.points.get("tail"));
    if let (Some(head), Some(tail)) = (head, tail) {
        let far = head.iter().flat_map(|a| tail.iter().map(move |b| dist(a, b))).fold(0.0, f64::max);
        if off(far, length) {
            errors.push(format!("{} m from bill to tail tip, its `bird.length` says {} m (within {})", fixed(far, 3), fixed(length, 3), percent(BIRD_FIT)));
        }
    }
    for (name, side, where_) in [("wing.L", -1.0, "its left, game -x (glTF +x)"), ("wing.R", 1.0, "its right, game +x (glTF -x)")] {
        if let Some(points) = parts.points.get(name).filter(|p| !p.is_empty()) {
            let mid = points.iter().map(|p| p[0]).sum::<f64>() / points.len() as f64;
            if mid * side <= 0.0 {
                errors.push(format!("{name} reaches out the wrong way: a bird's {name} is spread to {where_}"));
            }
        }
    }
    if let Some(h) = bird.get("perchHeight").and_then(json_number)
        && h > dims[1]
    {
        errors.push(format!("`bird.perchHeight` {} m is over the bird's height, {} m", fixed(h, 3), fixed(dims[1], 3)));
    }
    errors
}

/// What is wrong with an entry's `perches` (where the game's birds sit: [[x, y, z], ...] in the model's game axes,
/// metres, before the entry's `scale`) against the model's box (`lo`, `hi`, game axes, at rest, scaled by `scale`).
fn perch_errors(perches: &J, lo: [f64; 3], hi: [f64; 3], scale: f64) -> Vec<String> {
    let Some(list) = perches.list() else {
        return vec![format!("`perches` is a list of [x, y, z] (metres), not {}", repr(perches))];
    };
    if list.is_empty() {
        return vec!["`perches` is empty: leave it out".into()];
    }
    let mut errors = Vec::new();
    let mut outside = Vec::new();
    for (i, p) in list.iter().enumerate() {
        let xyz: Vec<f64> = p.list().unwrap_or(&[]).iter().filter_map(json_number).collect();
        if xyz.len() != 3 || p.list().is_none_or(|l| l.len() != 3) {
            errors.push(format!("`perches` {i}: {} is no [x, y, z]", repr(p)));
            continue;
        }
        if (0..3).any(|k| xyz[k] * scale < lo[k] - PERCH_OUT || xyz[k] * scale > hi[k] + PERCH_OUT) {
            outside.push(i);
        }
    }
    if let Some(&first) = outside.first() {
        let box_ = (0..3).map(|k| format!("{} to {}", fixed(lo[k] / scale, 2), fixed(hi[k] / scale, 2))).collect::<Vec<_>>().join(", ");
        errors.push(format!("{} of {} perches outside the model (the first, {first}: {}; its box x, y, z: {box_})", outside.len(), list.len(), repr(&list[first])));
    }
    errors
}

/// What is wrong with an entry's `habitat` (the kind of plant its `perches` are on, which sets the birds that sit
/// there: one of `HABITATS`), if it has one.
fn habitat_errors(entry: &J) -> Vec<String> {
    let Some(habitat) = entry.get("habitat") else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    if !habitat.str().is_some_and(|h| HABITATS.contains(&h)) {
        errors.push(format!("`habitat` {} is none of {}", repr(habitat), HABITATS.join(", ")));
    }
    if entry.get("perches").is_none() {
        errors.push("`habitat` without `perches`: it says what the perches are on".into());
    }
    errors
}

/// The primitives of a mesh (`gltf['meshes'][i]['primitives']`).
fn primitives<'a>(gltf: &'a J, mesh: &J) -> R<&'a [J]> {
    Ok(key(item(gltf, "meshes", mesh)?, "primitives")?.list().unwrap_or(&[]))
}

/// The positions of a primitive: every vertex, or where they can't be read here (compressed), its box's corners.
fn positions(g: &Glb, prim: &J) -> R<Vec<Vec<Num>>> {
    let index = key(key(prim, "attributes")?, "POSITION")?;
    if let Some(pos) = accessor(g, index)? {
        return Ok(pos);
    }
    let (lo, hi) = box_of(item(&g.gltf, "accessors", index)?)?;
    let mut out = Vec::with_capacity(8);
    for x in [lo[0], hi[0]] {
        for y in [lo[1], hi[1]] {
            for z in [lo[2], hi[2]] {
                out.push(vec![x, y, z]);
            }
        }
    }
    Ok(out)
}

/// Whether a primitive's vertex colour (COLOR_0) is anything but grey. The game reads it as the shading baked into
/// the model's creases (gamekit/shade.py) and multiplies it into the part's colour, so a colour in it would tint
/// the part. One it can't read here (compressed) passes.
fn coloured(g: &Glb, prim: &J) -> R<bool> {
    let Some(index) = key(prim, "attributes")?.get("COLOR_0") else { return Ok(false) };
    let Some(rows) = accessor(g, index)? else { return Ok(false) };
    Ok(rows.iter().any(|v| v.len() >= 3 && ((v[0].f() - v[1].f()).abs() > 0.01 || (v[0].f() - v[2].f()).abs() > 0.01)))
}

/// An accessor's `min` and `max`.
fn box_of(acc: &J) -> R<(Vec<Num>, Vec<Num>)> {
    let lo = nums(key(acc, "min")?, 0)?;
    let hi = nums(key(acc, "max")?, 0)?;
    if lo.len() < 3 || hi.len() < 3 {
        return Err("list index out of range".into());
    }
    Ok((lo, hi))
}

/// Every element of a float accessor as given by `values`, as an index into a list: `a['samplers'][c['sampler']]`.
fn sampler<'a>(a: &'a J, c: &J) -> R<&'a J> {
    index(key(a, "samplers")?, key(c, "sampler")?)
}

/// Whether a sampler moves at an even speed (each key as far on from the last, per second).
fn even(g: &Glb, sampler: &J) -> R<bool> {
    let times: Vec<f64> = values(g, key(sampler, "input")?)?.iter().map(|t| t[0]).collect();
    let keys = values(g, key(sampler, "output")?)?;
    if keys.len() < 3 {
        return Ok(false);
    }
    let (t0, t1) = (*times.first().ok_or("list index out of range")?, *times.last().unwrap());
    if t1 <= t0 {
        return Ok(false);
    }
    let rate: Vec<f64> = keys[0].iter().zip(keys.last().unwrap()).map(|(a, b)| (b - a) / (t1 - t0)).collect();
    for (k, t) in keys.iter().zip(&times) {
        for i in 0..rate.len() {
            if (k.get(i).ok_or("tuple index out of range")? - keys[0][i] - rate[i] * (t - t0)).abs() > 1e-3 {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// A material named for its finish (`stone@matte`, `tunic@fabric#body`) maps itself to it (models.ts
/// `namedFinish`; a tint slot `#body`, `#skin` or `#hair` in the name aside).
fn named_finish(name: &str) -> Option<J> {
    // re.sub(r'#(body|skin|hair)(?=@|$)', '', name)
    let mut s = String::with_capacity(name.len());
    let mut rest = name;
    while !rest.is_empty() {
        let slot = ["#body", "#skin", "#hair"].iter().find(|w| rest.starts_with(**w));
        if let Some(w) = slot {
            let after = &rest[w.len()..];
            if after.is_empty() || after.starts_with('@') || after == "\n" {
                rest = after;
                continue;
            }
        }
        let c = rest.chars().next().unwrap();
        s.push(c);
        rest = &rest[c.len_utf8()..];
    }
    let (head, tail) = s.rsplit_once('@')?;
    // tail.isalpha() and tail.islower(): letters, some cased, none upper (or title) case.
    let title = |c: char| matches!(c as u32, 0x1c5 | 0x1c8 | 0x1cb | 0x1f2 | 0x1f88..=0x1f8f | 0x1f98..=0x1f9f | 0x1fa8..=0x1faf | 0x1fbc | 0x1fcc | 0x1ffc);
    let lower = !tail.is_empty() && tail.chars().all(char::is_alphabetic) && tail.chars().any(char::is_lowercase) && !tail.chars().any(|c| c.is_uppercase() || title(c));
    if !head.is_empty() && lower { Some(J::Obj(vec![("finish".into(), J::Str(format!("material.finish.{tail}")))])) } else { None }
}

/// [(use, {clip, hold, delay, every, slot, out, over})] for each clip an entry's `motions.uses` plays (a clip,
/// {clip, ...} or a list of them).
fn uses(motions: &J) -> Vec<(String, J)> {
    let mut out = Vec::new();
    for (name, value) in motions.get("uses").filter(|u| u.truthy()).and_then(J::entries).unwrap_or(&[]) {
        let list = match value {
            J::Arr(l) => l.clone(),
            v => vec![v.clone()],
        };
        for u in list {
            out.push((name.clone(), if matches!(u, J::Obj(_)) { u } else { J::Obj(vec![("clip".into(), u)]) }));
        }
    }
    out
}

/// {clip: what plays it} for an entry's `motions` (in the order they're first named): `uses`, `loops`,
/// `open`, `idle`, `night`, `growth`, `ride` (the clip whose part carries its users) and `clock` (`hour`,
/// `minute`: hands set by the time).
fn played(motions: &J) -> R<Vec<(J, String)>> {
    let mut out: Vec<(J, String)> = Vec::new();
    let mut add = |clip: &J, by: String| {
        if !out.iter().any(|(c, _)| c == clip) {
            out.push((clip.clone(), by));
        }
    };
    for (name, u) in uses(motions) {
        add(key(&u, "clip")?, format!("use {}", repr_str(&name)));
    }
    for (name, clip) in motions.get("loops").filter(|l| l.truthy()).and_then(J::entries).unwrap_or(&[]) {
        add(clip, format!("loop {}", repr_str(name)));
    }
    for kind in ["open", "idle", "night", "growth", "ride"] {
        if let Some(clip) = motions.get(kind).filter(|c| c.truthy()) {
            add(clip, kind.into());
        }
    }
    let clock = motions.get("clock").filter(|c| c.truthy());
    for hand in ["hour", "minute"] {
        if let Some(clip) = clock.and_then(|c| c.get(hand)).filter(|c| c.truthy()) {
            add(clip, format!("clock {hand}"));
        }
    }
    Ok(out)
}

/// The clips of `motions` that go round, or from rest to rest (and so must end where they start): loops,
/// `idle`, a use's clip that repeats (`every`) and a clock's hands (a whole turn). As dict keys.
fn loops(motions: &J) -> R<HashSet<String>> {
    let mut out = HashSet::new();
    for (_, clip) in motions.get("loops").filter(|l| l.truthy()).and_then(J::entries).unwrap_or(&[]) {
        out.insert(clip.key());
    }
    if let Some(idle) = motions.get("idle").filter(|c| c.truthy()) {
        out.insert(idle.key());
    }
    for (_, u) in uses(motions) {
        if u.get("every").is_some_and(J::truthy) {
            out.insert(key(&u, "clip")?.key());
        }
    }
    let clock = motions.get("clock").filter(|c| c.truthy());
    for hand in ["hour", "minute"] {
        if let Some(clip) = clock.and_then(|c| c.get(hand)).filter(|c| c.truthy()) {
            out.insert(clip.key());
        }
    }
    Ok(out)
}

/// What's wrong with the fields of an entry's `motions` (objectMotion.ts): a use's `delay` and `hold` are
/// seconds (0 or more), `every` and `out` more than 0, `slot` a slot number, `over` 1 or -1 (along with the use,
/// forward or back); `clock` names its hands' clips.
fn motions_errors(motions: &J) -> Vec<String> {
    let mut errors = Vec::new();
    for (name, u) in uses(motions) {
        let name = repr_str(&name);
        for k in ["hold", "delay"] {
            if let Some(v) = u.get(k)
                && !v.num().is_some_and(|n| n.f() >= 0.0)
            {
                errors.push(format!("`motions.uses` {name}: {k} {} is no number of seconds", repr(v)));
            }
        }
        for k in ["every", "out"] {
            if let Some(v) = u.get(k)
                && !v.num().is_some_and(|n| n.f() > 0.0)
            {
                errors.push(format!("`motions.uses` {name}: {k} {} is no number of seconds above 0", repr(v)));
            }
        }
        if let Some(over) = u.get("over")
            && !matches!(over, J::Int(1 | -1))
        {
            errors.push(format!("`motions.uses` {name}: over {} is neither 1 nor -1", repr(over)));
        }
        // (An int, not a bool.)
        if let Some(slot) = u.get("slot")
            && !matches!(slot, J::Int(i) if *i >= 0)
        {
            errors.push(format!("`motions.uses` {name}: slot {} is no slot (0, 1, ...)", repr(slot)));
        }
        let mut unknown: Vec<&str> = u.entries().unwrap_or(&[]).iter().map(|(k, _)| k.as_str()).filter(|k| !["clip", "hold", "delay", "every", "slot", "out", "over"].contains(k)).collect();
        unknown.sort();
        if !unknown.is_empty() {
            errors.push(format!("`motions.uses` {name}: unknown {}", unknown.join(", ")));
        }
    }
    if let Some(clock) = motions.opt("clock") {
        let named = |h: &str| clock.get(h).is_some_and(J::truthy);
        let odd = clock.entries().is_some_and(|e| e.iter().any(|(k, _)| !["hour", "minute", "at"].contains(&k.as_str())));
        if !matches!(clock, J::Obj(_)) || !(named("hour") || named("minute")) || odd {
            errors.push("`motions.clock` names its hands' clips: {\"hour\", \"minute\", \"at\"}".into());
        } else if let Some(at) = clock.get("at")
            && at.num().is_none()
        {
            errors.push(format!("`motions.clock.at` {} is no time (minutes past twelve)", repr(at)));
        }
    }
    errors
}

/// The product of two quaternions (x, y, z, w).
fn qmul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;
    [aw * bx + ax * bw + ay * bz - az * by, aw * by - ax * bz + ay * bw + az * bx, aw * bz + ax * by - ay * bx + az * bw, aw * bw - ax * bx - ay * by - az * bz]
}

/// [(clip, node, axis)]: the parts a clip that goes round (`loops`: a loop, `idle`, a clock's hand) turns one
/// way about one axis all through (G.spin: a fan's blades, a wheel; not G.rock, which turns back), so they
/// sweep a whole circle as it repeats. `axis`: the turn's axis in the node's own frame (unit).
fn turning(g: &Glb, motions: &J) -> R<Vec<(String, usize, [f64; 3])>> {
    let mut out = Vec::new();
    let going = loops(motions)?;
    for a in g.gltf.get("animations").and_then(J::list).unwrap_or(&[]) {
        let name = a.get("name").map(J::key).unwrap_or_default();
        if !going.contains(&name) {
            continue;
        }
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let node = target.opt("node");
            if target.get("path") != Some(&J::Str("rotation".into())) || node.is_none() {
                continue;
            }
            let node = target_node(node.unwrap());
            let keys = values(g, key(sampler(a, c)?, "output")?)?;
            let q = |k: &Vec<f64>| -> R<[f64; 4]> { k.get(..4).and_then(|k| k.try_into().ok()).ok_or_else(|| "not enough values to unpack (expected 4)".into()) };
            // Each step from one key to the next, in the part's own frame: a turn about one axis, one way.
            let mut steps = Vec::new();
            for (p, k) in keys.iter().zip(keys.iter().skip(1)) {
                let p = q(p)?;
                let st = qmul([-p[0], -p[1], -p[2], p[3]], q(k)?);
                steps.push(if st[3] >= 0.0 { st } else { st.map(|v| -v) });
            }
            let mut big: Option<[f64; 4]> = None;
            for st in &steps {
                if big.is_none_or(|b| hypot(&st[..3]) > hypot(&b[..3])) {
                    big = Some(*st);
                }
            }
            let Some(big) = big else { continue };
            let n = hypot(&big[..3]);
            if n < 1e-6 {
                continue;
            }
            let axis = [big[0] / n, big[1] / n, big[2] / n];
            let one_way = steps.iter().all(|st| {
                let along = 0.0 + st[0] * axis[0] + st[1] * axis[1] + st[2] * axis[2];
                let h = hypot(&st[..3]);
                along > -1e-6 && dist(&st[..3], &axis.map(|v| v * h)) < 1e-3
            });
            if one_way {
                out.push((name.clone(), node, axis));
            }
        }
    }
    Ok(out)
}

/// A part that goes round: its clip, its name, the floor-plan box it sweeps (x, depth), and how far it
/// reaches from its axis.
struct Swept {
    clip: String,
    name: String,
    x: (f64, f64),
    z: (f64, f64),
    radius: f64,
}

/// The floor-plan box each `turning` part sweeps, a whole circle round its pivot (in game axes), and how far
/// it reaches from its axis.
fn swept(g: &Glb, motions: &J) -> R<Vec<Swept>> {
    let turns = turning(g, motions)?;
    if turns.is_empty() {
        return Ok(Vec::new());
    }
    let mut worlds = HashMap::new();
    let meshes = node_matrices(&g.gltf, None, Some(&mut worlds))?;
    let empty = J::Arr(Vec::new());
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let mut out = Vec::new();
    for (clip, node, axis) in turns {
        let (mut below, mut todo) = (HashSet::new(), vec![node]);
        while let Some(i) = todo.pop() {
            below.insert(i);
            for c in nodes.at(i as i64).ok_or("list index out of range")?.get("children").and_then(J::list).unwrap_or(&[]) {
                todo.push(node_index(nodes, c)?);
            }
        }
        let Some(w) = worlds.get(&node) else { continue };
        let w: Vec<f64> = w.iter().map(|v| v.f()).collect();
        let pivot = [w[12], w[13], w[14]];
        let a = [w[0] * axis[0] + w[4] * axis[1] + w[8] * axis[2], w[1] * axis[0] + w[5] * axis[1] + w[9] * axis[2], w[2] * axis[0] + w[6] * axis[1] + w[10] * axis[2]];
        let n = hypot(&a);
        let a = a.map(|v| v / n);
        // A circle round the axis reaches r * sqrt(1 - a_k^2) either side of its centre along axis k.
        let spread = a.map(|v| f64::max(0.0, 1.0 - v * v).sqrt());
        let (mut lo, mut hi, mut radius) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3], 0.0f64);
        for p in &meshes {
            if !below.contains(&p.node) {
                continue;
            }
            for prim in primitives(&g.gltf, &p.mesh)? {
                for v in positions(g, prim)? {
                    let q = applied(&p.m, &v);
                    let d = [q[0] - pivot[0], q[1] - pivot[1], q[2] - pivot[2]];
                    let along = 0.0 + d[0] * a[0] + d[1] * a[1] + d[2] * a[2];
                    let r = dist(&d, &a.map(|k| along * k));
                    if r > radius {
                        radius = r;
                    }
                    for k in 0..3 {
                        let centre = pivot[k] + along * a[k];
                        let (l, h) = (centre - r * spread[k], centre + r * spread[k]);
                        if l < lo[k] {
                            lo[k] = l;
                        }
                        if h > hi[k] {
                            hi[k] = h;
                        }
                    }
                }
            }
        }
        if lo[0] <= hi[0] {
            // glTF x -> game -x
            out.push(Swept { clip, name: node_name(nodes, node), x: (-hi[0], -lo[0]), z: (lo[2], hi[2]), radius });
        }
    }
    Ok(out)
}

/// (errors, warnings) for a file's animations: moving parts turned or slid by node clips
/// (docs/design/objects.md, "Moving parts"), each starting at rest, each played by the entry's `motions`.
fn check_clips(g: &Glb, entry: &J) -> R<(Vec<String>, Vec<String>)> {
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    let empty = J::Arr(Vec::new());
    let animations = g.gltf.get("animations").and_then(J::list).unwrap_or(&[]);
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let rest = |path: &str| -> Option<Vec<J>> {
        match path {
            "rotation" => Some(vec![J::Int(0), J::Int(0), J::Int(0), J::Int(1)]),
            "translation" => Some(vec![J::Int(0), J::Int(0), J::Int(0)]),
            "scale" => Some(vec![J::Int(1), J::Int(1), J::Int(1)]),
            _ => None,
        }
    };
    let clip_name = |a: &J| a.get("name").cloned().unwrap_or(J::Str(String::new()));
    for a in animations {
        let name = repr(&clip_name(a));
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let (path, node) = (target.get("path"), target.opt("node"));
            let at_rest = path.and_then(J::str).and_then(rest);
            if at_rest.is_none() || node.is_none() {
                errors.push(format!("clip {name} animates {} (only a node's rotation, translation or scale)", path.map(str_of).unwrap_or("None".into())));
                continue;
            }
            let path = path.and_then(J::str).unwrap();
            let first = first_value(g, key(sampler(a, c)?, "output")?, false)?;
            let node = node_index(nodes, node.unwrap())?;
            let n = nodes.at(node as i64).unwrap();
            let at_rest = match n.get(path) {
                Some(v) => v.list().ok_or("object of type has no len()")?.to_vec(),
                None => at_rest.unwrap(),
            };
            let away = match &first {
                None => true,
                Some(first) => {
                    let mut away = false;
                    for (k, r) in at_rest.iter().enumerate() {
                        let f = first.get(k).ok_or("tuple index out of range")?;
                        let r = r.num().ok_or("unsupported operand type(s) for -")?.f();
                        if (f - r).abs() > 1e-3 {
                            away = true;
                            break;
                        }
                    }
                    away
                }
            };
            if away {
                errors.push(format!("clip {name} starts {} away from where the file has it (the first key is the rest pose)", node_name(nodes, node)));
            }
        }
    }
    // (A part inside one that moves, a lever on a door, moves with it and by its own clips too.)
    let names: BTreeSet<String> = animations.iter().map(|a| clip_name(a).key()).collect();
    let none = J::Obj(Vec::new());
    let motions = entry.get("motions").filter(|m| m.truthy()).unwrap_or(&none);
    errors.extend(motions_errors(motions));
    let mut plays = played(motions)?;
    plays.sort_by_key(|(clip, _)| clip.key());
    for (clip, by) in &plays {
        if !names.contains(&clip.key()) {
            errors.push(format!("`motions` plays {} ({by}), which is no clip of the file", repr(clip)));
        }
    }
    let played_keys: HashSet<String> = plays.iter().map(|(c, _)| c.key()).collect();
    for name in names.iter().filter(|n| !played_keys.contains(*n)) {
        warnings.push(format!("clip {} plays for no use (name it in the entry's `motions`)", repr_str(name)));
    }
    // A use's clip plays as the use starts: one much longer than about 1 s reads as slow. Loops, ambient,
    // night, open and growth clips keep their own pace (a merry-go-round's round, a clock's minute).
    let going = loops(motions)?;
    let by_use: HashSet<String> = plays.iter().filter(|(_, by)| by.starts_with("use ")).map(|(c, _)| c.key()).filter(|c| !going.contains(c)).collect();
    for a in animations {
        if !by_use.contains(&clip_name(a).key()) {
            continue;
        }
        let mut length: Option<f64> = None;
        for s in key(a, "samplers")?.list().unwrap_or(&[]) {
            let last = first_value(g, key(s, "input")?, true)?.map(|v| v[0]).unwrap_or(0.0);
            if length.is_none_or(|l| last > l) {
                length = Some(last);
            }
        }
        let length = length.ok_or("max() iterable argument is empty")?;
        if length > USE_CLIP {
            warnings.push(format!(
                "clip {} takes {} s, played by a use: keep a use's clip about 1 s (over 3 s, a loop or an ambient clip instead)",
                repr(key(a, "name")?),
                fixed(length, 1)
            ));
        }
    }
    // A loop goes round from rest to rest: each channel ends where it starts (a turn, either sign).
    // A turn by a whole fraction of a circle (a fan's four blades a quarter round) loops if the part
    // looks the same turned, and a move at an even speed (G.scroll: a belt's slats) if it looks the same
    // moved: warnings to check that.
    for a in animations {
        if !going.contains(&clip_name(a).key()) {
            continue;
        }
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let s = sampler(a, c)?;
            let out = key(s, "output")?;
            let (Some(first), Some(last)) = (first_value(g, out, false)?, first_value(g, out, true)?) else { continue };
            let target = key(c, "target")?;
            let node = node_name(nodes, node_index(nodes, key(target, "node")?)?);
            let clip = repr(key(a, "name")?);
            let path = target.get("path");
            if path == Some(&J::Str("rotation".into())) {
                let dot = first.iter().zip(&last).fold(0.0, |s, (x, y)| s + x * y);
                let turned = (2.0 * f64::min(1.0, dot.abs()).acos()) * (180.0 / PI);
                if turned < 0.1 {
                    continue;
                }
                let n = py::round_even(360.0 / turned);
                if (2.0..=12.0).contains(&n) && (turned * n - 360.0).abs() < 0.5 {
                    warnings.push(format!(
                        "loop {clip} ends {node} turned {}° from where it starts: right only if it looks the same turned (1/{} of a circle)",
                        fixed(turned, 0),
                        n as i64
                    ));
                    continue;
                }
            } else {
                let diffs: Vec<f64> = first.iter().zip(&last).map(|(x, y)| (x - y).abs()).collect();
                let most = diffs.iter().cloned().reduce(|m, d| if d > m { d } else { m }).ok_or("max() iterable argument is empty")?;
                if most <= 1e-3 {
                    continue;
                }
                if path == Some(&J::Str("translation".into())) && even(g, s)? {
                    let moved = dist(&first, &last);
                    warnings.push(format!(
                        "loop {clip} ends {node} moved {} cm from where it starts: right only if it looks the same moved (a belt's slats one slat on, its ends hidden)",
                        fixed(moved * 100.0, 0)
                    ));
                    continue;
                }
            }
            errors.push(format!("loop {clip} ends {node} away from where it starts ({}): a loop goes from rest to rest (G.spin, G.rock, G.scroll)", str_of(key(target, "path")?)));
        }
    }
    Ok((errors, warnings))
}

/// The parts a clip scales up from rest (G.grow: darts, a pot thrown, water), at their largest key, so what
/// is only small or flattened at rest is measured as it shows. Not by the clips `held` names (`open`,
/// `night`: a state of its own, a sofa bed pulled out; the model is at rest).
fn grown(g: &Glb, held: &[Option<J>]) -> R<Pose> {
    let empty = J::Arr(Vec::new());
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let mut out: Pose = HashMap::new();
    let abs_sum = |k: &[Num]| k.iter().fold(Num::I(0), |s, v| s + if v.f() < 0.0 { -*v } else { *v }).f();
    for a in g.gltf.get("animations").and_then(J::list).unwrap_or(&[]) {
        if held.contains(&a.opt("name").cloned()) {
            continue;
        }
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let node = target.opt("node");
            if target.get("path") != Some(&J::Str("scale".into())) || node.is_none() {
                continue;
            }
            let node = node_index(nodes, node.unwrap())?;
            let rest = match nodes.at(node as i64).unwrap().get("scale") {
                Some(s) => nums(s, 0)?,
                None => vec![Num::I(1); 3],
            };
            let keys: Vec<Vec<Num>> = values(g, key(sampler(a, c)?, "output")?)?.into_iter().map(|k| k.into_iter().map(Num::F).collect()).collect();
            let mut best: Option<&Vec<Num>> = None;
            for k in &keys {
                if best.is_none_or(|b| abs_sum(k) > abs_sum(b)) {
                    best = Some(k);
                }
            }
            let best = best.ok_or("max() iterable argument is empty")?;
            if best.iter().zip(&rest).any(|(b, r)| b.f().abs() > 1.5 * r.f().abs() + 1e-6) && out.get(&(node, "scale")).is_none_or(|have| abs_sum(best) > abs_sum(have)) {
                out.insert((node, "scale"), best.clone());
            }
        }
    }
    Ok(out)
}

type Bounds = ([Num; 3], [Num; 3]);
/// Each moving part's bounds (None: the still body), in the order they're met.
type PartBounds = Vec<(Option<usize>, Bounds)>;

/// The file's bounds in game axes (glTF x -> game -x, as `render/pc/models.ts` bakes a file), from
/// each primitive's box through its node, and each moving part's (None: the still body), in the order
/// they're met.
fn bounds(gltf: &J, pose: &Pose) -> R<(Bounds, PartBounds)> {
    let accessors = gltf.get("accessors").cloned().unwrap_or(J::Arr(Vec::new()));
    let (inf, ninf) = (Num::F(f64::INFINITY), Num::F(f64::NEG_INFINITY));
    let (mut lo, mut hi) = ([inf; 3], [ninf; 3]);
    let mut parts: PartBounds = Vec::new();
    for p in node_matrices(gltf, Some(pose), None)? {
        let at = match parts.iter().position(|(part, _)| *part == p.part) {
            Some(at) => at,
            None => {
                parts.push((p.part, ([inf; 3], [ninf; 3])));
                parts.len() - 1
            }
        };
        for prim in primitives(gltf, &p.mesh)? {
            let (bl, bh) = box_of(index(&accessors, key(key(prim, "attributes")?, "POSITION")?)?)?;
            for cx in [bl[0], bh[0]] {
                for cy in [bl[1], bh[1]] {
                    for cz in [bl[2], bh[2]] {
                        let mut q = apply(&p.m, &[cx, cy, cz]);
                        q[0] = -q[0];
                        let (plo, phi) = &mut parts[at].1;
                        for k in 0..3 {
                            lo[k] = py::min(lo[k], q[k]);
                            hi[k] = py::max(hi[k], q[k]);
                            plo[k] = py::min(plo[k], q[k]);
                            phi[k] = py::max(phi[k], q[k]);
                        }
                    }
                }
            }
        }
    }
    Ok(((lo, hi), parts))
}

/// How far the model reaches from its origin across the floor (m), its parts where `pose` puts them (`grown`:
/// as it shows): its farthest vertex's hypot(x, depth) (a round tray reaches its radius, a square one its
/// corners), or where the positions can't be read here (compressed), its boxes' farthest corner.
fn reach(g: &Glb, pose: &Pose) -> R<f64> {
    let mut far = 0.0f64;
    for p in node_matrices(&g.gltf, Some(pose), None)? {
        for prim in primitives(&g.gltf, &p.mesh)? {
            for v in positions(g, prim)? {
                let q = applied(&p.m, &v);
                let r = hypot(&[q[0], q[2]]);
                if r > far {
                    far = r;
                }
            }
        }
    }
    Ok(far)
}

/// `x ** 2` as Python has it: the C library's `pow`, which isn't always `x * x` (macOS's differs in the last
/// bit for about 1 in 1000). (`black_box`: LLVM would make `pow(x, 2.0)` a multiplication.)
fn square(x: f64) -> f64 {
    x.abs().powf(std::hint::black_box(2.0))
}

/// An opaque triangle of the model at rest, with the node it belongs to.
struct Tri {
    node: usize,
    a: [f64; 3],
    e1: [f64; 3],
    e2: [f64; 3],
    nrm: [f64; 3],
}

/// Every face the line through p along d crosses (Möller-Trumbore), in the order given: (how far along d,
/// negative behind p; the face's normal).
fn hits<'a>(p: &'a [f64; 3], d: &'a [f64; 3], faces: impl IntoIterator<Item = &'a Tri>) -> impl Iterator<Item = (f64, &'a [f64; 3])> {
    faces.into_iter().filter_map(move |f| {
        let (a, e1, e2) = (&f.a, &f.e1, &f.e2);
        let h = [d[1] * e2[2] - d[2] * e2[1], d[2] * e2[0] - d[0] * e2[2], d[0] * e2[1] - d[1] * e2[0]];
        let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
        if det.abs() < 1e-12 {
            return None;
        }
        let s = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
        let u = (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]) / det;
        #[allow(clippy::manual_range_contains)] // (not the same for a NaN)
        if u < 0.0 || u > 1.0 {
            return None;
        }
        let q = [s[1] * e1[2] - s[2] * e1[1], s[2] * e1[0] - s[0] * e1[2], s[0] * e1[1] - s[1] * e1[0]];
        let v = (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]) / det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        Some(((e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]) / det, &f.nrm))
    })
}

/// The nearest face each way along d from p: [forward, backward], each (distance, whether it is a face leaving
/// a solid that way).
fn cast(p: &[f64; 3], d: &[f64; 3], faces: &[&Tri]) -> [(f64, bool); 2] {
    let mut near = [(f64::INFINITY, false), (f64::INFINITY, false)];
    for (t, nrm) in hits(p, d, faces.iter().copied()) {
        let (slot, way) = if t > 0.0 { (0, 1.0) } else { (1, -1.0) };
        if 1e-4 < t.abs() && t.abs() < near[slot].0 {
            near[slot] = (t.abs(), (nrm[0] * d[0] + nrm[1] * d[1] + nrm[2] * d[2]) * way > 0.0);
        }
    }
    near
}

/// The triangles of the placed meshes' primitives that `keep` keeps (by their material index), with the node
/// each belongs to and its normal from the winding; None if some positions can't be read here (compressed).
fn triangles(g: &Glb, placed: &[Placed], keep: &dyn Fn(Option<&J>) -> R<bool>) -> R<Option<Vec<Tri>>> {
    let det = |m: &[f64]| m[0] * (m[5] * m[10] - m[9] * m[6]) - m[4] * (m[1] * m[10] - m[9] * m[2]) + m[8] * (m[1] * m[6] - m[5] * m[2]);
    let mut tris = Vec::new();
    for p in placed {
        let mf: Vec<f64> = p.m.iter().map(|v| v.f()).collect();
        let sign = if det(&mf) >= 0.0 { 1.0 } else { -1.0 };
        for prim in primitives(&g.gltf, &p.mesh)? {
            let triangles = prim.get("mode").is_none_or(|m| m.num().is_some_and(|n| n.f() == 4.0));
            if !triangles || !keep(prim.opt("material"))? {
                continue;
            }
            let pos = accessor(g, key(key(prim, "attributes")?, "POSITION")?)?;
            let idx = match prim.get("indices") {
                Some(i) => accessor(g, i)?,
                None => Some((0..pos.as_ref().map_or(0, |p| p.len())).map(|k| vec![Num::I(k as i64)]).collect()),
            };
            let (Some(pos), Some(idx)) = (pos, idx) else { return Ok(None) };
            let world: Vec<[f64; 3]> = pos.iter().map(|v| applied(&p.m, v)).collect();
            let corner = |k: usize| -> R<[f64; 3]> {
                let i = idx[k][0];
                let i = match i { Num::I(i) => i, Num::F(_) => return Err("list indices must be integers or slices, not float".into()) };
                let i = if i < 0 { i + world.len() as i64 } else { i };
                world.get(i as usize).copied().ok_or_else(|| "list index out of range".into())
            };
            let mut k = 0;
            while k + 2 < idx.len() {
                let (a, b, c) = (corner(k)?, corner(k + 1)?, corner(k + 2)?);
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let nrm = [sign * (e1[1] * e2[2] - e1[2] * e2[1]), sign * (e1[2] * e2[0] - e1[0] * e2[2]), sign * (e1[0] * e2[1] - e1[1] * e2[0])];
                tris.push(Tri { node: p.node, a, e1, e2, nrm });
                k += 3;
            }
        }
    }
    Ok(Some(tris))
}

/// [(part, share of it in view)] for the parts flattened to nothing at rest (a node scaled under 0.01
/// along an axis, G.grow from 0.001): it still shows as a flat shape, if it's 3 cm across or more, where its
/// flattened plane isn't inside something opaque (`opaque(material index)`), as the dartboard's flights did
/// as crosses on its face. Each of its points is looked from along the flattened axis both ways: inside a
/// solid, the nearest face either way is one leaving it (its normal, from the winding, pointing on).
fn flattened_in_view(g: &Glb, opaque: &dyn Fn(Option<&J>) -> R<bool>) -> R<Vec<(String, f64)>> {
    let empty = J::Arr(Vec::new());
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let mut flat: BTreeMap<usize, usize> = BTreeMap::new();
    for i in moved_nodes(&g.gltf)? {
        let n = nodes.at(i as i64).ok_or("list index out of range")?;
        let scale = match n.get("scale") { Some(s) => nums(s, 0)?, None => vec![Num::I(1); 3] };
        let smallest = scale.iter().map(|v| v.f().abs()).fold(f64::INFINITY, |m, v| if v < m { v } else { m });
        if n.has("mesh") && smallest < 0.01 {
            if scale.len() < 3 {
                return Err("list index out of range".into());
            }
            let mut axis = 0;
            for k in 1..3 {
                if scale[k].f().abs() < scale[axis].f().abs() {
                    axis = k;
                }
            }
            flat.insert(i, axis);
        }
    }
    if flat.is_empty() {
        return Ok(Vec::new());
    }
    let placed = node_matrices(&g.gltf, None, None)?;
    // Every opaque triangle of the model at rest, with the node it belongs to.
    let Some(tris) = triangles(g, &placed, opaque)? else { return Ok(Vec::new()) };

    let mut out = Vec::new();
    for (&node, &axis) in &flat {
        let Some(p) = placed.iter().find(|p| p.node == node) else { return Err("StopIteration".into()) };
        let m: Vec<f64> = p.m.iter().map(|v| v.f()).collect();
        let d = [m[axis * 4], m[axis * 4 + 1], m[axis * 4 + 2]];
        let length = (0.0 + d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let length = if length == 0.0 { 1.0 } else { length };
        let d = d.map(|v| v / length);
        let mut points = Vec::new();
        for prim in primitives(&g.gltf, key(nodes.at(node as i64).unwrap(), "mesh")?)? {
            let Some(pos) = accessor(g, key(key(prim, "attributes")?, "POSITION")?)? else { return Ok(out) };
            points.extend(pos.iter().map(|v| applied(&p.m, v)));
        }
        if points.is_empty() {
            continue;
        }
        // Its flat shape's two sides (across its plane, along its own main lines): one under 3 cm across, or
        // a sliver under 1 cm (a stream's end, a painting's edge before it spreads), hardly shows.
        let u0 = if d[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
        let along = 0.0 + u0[0] * d[0] + u0[1] * d[1] + u0[2] * d[2];
        let u = [u0[0] - d[0] * along, u0[1] - d[1] * along, u0[2] - d[2] * along];
        let ul = (0.0 + u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let u = u.map(|v| v / ul);
        let w = [d[1] * u[2] - d[2] * u[1], d[2] * u[0] - d[0] * u[2], d[0] * u[1] - d[1] * u[0]];
        let flat2: Vec<[f64; 2]> = points.iter().map(|q| [0.0 + q[0] * u[0] + q[1] * u[1] + q[2] * u[2], 0.0 + q[0] * w[0] + q[1] * w[1] + q[2] * w[2]]).collect();
        let count = flat2.len() as f64;
        let mu = [flat2.iter().fold(0.0, |s, c| s + c[0]) / count, flat2.iter().fold(0.0, |s, c| s + c[1]) / count];
        let cxx = flat2.iter().fold(0.0, |s, c| s + square(c[0] - mu[0]));
        let cyy = flat2.iter().fold(0.0, |s, c| s + square(c[1] - mu[1]));
        let cxy = flat2.iter().fold(0.0, |s, c| s + (c[0] - mu[0]) * (c[1] - mu[1]));
        let angle = 0.5 * (2.0 * cxy).atan2(cxx - cyy);
        let axes = [(angle.cos(), angle.sin()), (-angle.sin(), angle.cos())];
        let mut side: Vec<f64> = axes
            .iter()
            .map(|a| {
                let along = flat2.iter().map(|c| c[0] * a.0 + c[1] * a.1);
                let hi = along.clone().reduce(|m, v| if v > m { v } else { m }).unwrap();
                let lo = along.reduce(|m, v| if v < m { v } else { m }).unwrap();
                hi - lo
            })
            .collect();
        side.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if side[0] < 0.01 || side[1] < 0.03 {
            continue;
        }
        let step = (points.len() / 24).max(1);
        let points: Vec<[f64; 3]> = points.into_iter().step_by(step).collect();
        let others: Vec<&Tri> = tris.iter().filter(|t| t.node != node).collect();
        let mut shown = 0;
        for p in &points {
            // Hidden inside a solid (the nearest face either way along its flattened axis leaves one), or,
            // lying level, under something (the game looks from above: glTF +y; a shower head's rain).
            let near = cast(p, &d, &others);
            if near[0].1 && near[1].1 {
                continue;
            }
            if d[1].abs() > 0.7 && cast(p, &[0.0, 1.0, 0.0], &others)[0].0 < f64::INFINITY {
                continue;
            }
            shown += 1;
        }
        let share = shown as f64 / points.len() as f64;
        if share > 0.25 {
            out.push((node_name(nodes, node), share));
        }
    }
    Ok(out)
}

/// `overrides.get(name, named_finish(name) or overrides.get('*'))`: a material's entry in `materials`
/// (None: unmapped).
fn override_of(overrides: &J, name: &str) -> J {
    match overrides.get(name) {
        Some(ov) => ov.clone(),
        None => named_finish(name).or_else(|| overrides.get("*").cloned()).unwrap_or(J::Null),
    }
}

/// Warnings for a model (at rest `lo`; as it shows, its parts where `pose` puts them: `slo`, `shi`,
/// `sparts`) against one content object's facts (None: none, a floor thing).
#[allow(clippy::too_many_arguments)]
fn content_warnings(facts: Option<&Facts>, lo: &[Num; 3], (slo, shi): &Bounds, sparts: &[(Option<usize>, Bounds)], scale: Num, g: &Glb, pose: &Pose, entry: &J) -> R<Vec<String>> {
    let mut warnings = Vec::new();
    let floor = J::Str("floor".into());
    let layer = facts.map(|f| &f.layer).unwrap_or(&floor);
    let is = |l: &str| layer.str() == Some(l);
    let foot = |f: &Facts| -> R<(Num, Num)> {
        let l = nums(&f.footprint, 2)?;
        Ok((l[0], l[1]))
    };
    // Against a wall: the wall's face is `WALL` into the footprint's back tile. A wall thing, or a floor
    // thing indoors that stands against one, has its back on it.
    // (A rug lies under the wall: it may reach in. Stairs reach their footprint's back edge, where the floor
    // above goes on from the top step: no wall.)
    if let Some(f) = facts
        && (is("wall") || (is("floor") && !f.outside && !f.stairs))
    {
        let depth = f.footprint.at(1).ok_or("list index out of range")?.num().ok_or("bad operand type for unary -")?;
        let face = ((-depth) / Num::I(2)).f() + WALL;
        if slo[2].f() < face - 0.01 {
            warnings.push(format!(
                "reaches {} cm into the wall (its back at depth {}, the wall's face at {})",
                fixed((face - slo[2].f()) * 100.0, 1),
                fixed(slo[2].f(), 3),
                fixed(face, 2)
            ));
        }
        // (Whether it should stand against the wall, its back on the face, nothing in the content says: a
        // counter and an island have the same top, and `fillsWall` only keeps the wall over it clear.)
    }
    // Within its footprint: a thing indoors on the floor, a wall or the ceiling that reaches past its tiles
    // overlaps what stands next to it (a fan's blades another's). Outside, trees, hedges and parasols
    // spread over their neighbours, and a model with `vary` is drawn larger or smaller anyway; a rug
    // may reach under the wall, and a thing on a top takes its disc (below).
    if let Some(f) = facts
        && (is("floor") || is("wall") || is("ceiling"))
        && !f.outside
        && !entry.has("vary")
    {
        let (w, d) = foot(f)?;
        let half = |n: Num| (n / Num::I(2)).f();
        let (nw, nd) = (((-w) / Num::I(2)).f(), ((-d) / Num::I(2)).f());
        let over = [nw - slo[0].f(), shi[0].f() - half(w), nd - slo[2].f(), shi[2].f() - half(d)].into_iter().reduce(|m, v| if v > m { v } else { m }).unwrap();
        if over > OVERHANG {
            warnings.push(format!(
                "reaches {} cm past its {} x {} footprint (x {} to {}, depth {} to {}): it overlaps what stands next to it",
                fixed(over * 100.0, 1),
                py::num_str(w),
                py::num_str(d),
                fixed(slo[0].f(), 2),
                fixed(shi[0].f(), 2),
                fixed(slo[2].f(), 2),
                fixed(shi[2].f(), 2)
            ));
        }
        // A part that goes round (a fan's blades) sweeps a whole circle about its pivot, not just its rest pose.
        let none = J::Obj(Vec::new());
        for s in swept(g, entry.get("motions").filter(|m| m.truthy()).unwrap_or(&none))? {
            let sc = scale.f();
            let (xlo, xhi, zlo, zhi, radius) = (s.x.0 * sc, s.x.1 * sc, s.z.0 * sc, s.z.1 * sc, s.radius * sc);
            let over = [nw - xlo, xhi - half(w), nd - zlo, zhi - half(d)].into_iter().reduce(|m, v| if v > m { v } else { m }).unwrap();
            if over > SWEEP {
                warnings.push(format!(
                    "turning ({}), {} sweeps {} cm past its {} x {} footprint (x {} to {}, depth {} to {}; {} m from its axis): it overlaps what stands next to it (keep its axis on the middle, or make it smaller)",
                    repr_str(&s.clip),
                    s.name,
                    fixed(over * 100.0, 1),
                    py::num_str(w),
                    py::num_str(d),
                    fixed(xlo, 3),
                    fixed(xhi, 3),
                    fixed(zlo, 3),
                    fixed(zhi, 3),
                    fixed(radius, 3)
                ));
            }
        }
    }
    // On a top: the game keeps the disc of its content `radius` round its origin clear on its host, so the
    // model reaches no farther (a square tray's corners, a model off its origin).
    if let Some(f) = facts
        && is("surface")
    {
        let out = reach(g, pose)? * scale.f();
        if out > 1.1 * f.radius {
            warnings.push(format!(
                "reaches {} m from its origin, past the {} m disc its content `radius` keeps on a top: it overlaps what stands next to it (give it a larger radius)",
                fixed(out, 3),
                fixed(f.radius, 2)
            ));
        }
    }
    // Thin on the floor: a house's floor hides what lies under `FLOOR_TOP` (a mat, a belt), the whole
    // model or a moving part (as it shows, grown).
    if (is("floor") || is("rug")) && lo[1].f() <= 0.005 {
        let mut flat: Vec<(Option<usize>, f64)> = Vec::new();
        if shi[1].f() < FLOOR_TOP {
            flat.push((None, shi[1].f()));
        }
        for (part, b) in sparts {
            let top = (b.1[1] * scale).f();
            if part.is_some() && top < FLOOR_TOP {
                flat.push((*part, top));
            }
        }
        let empty = J::Arr(Vec::new());
        let nodes = g.gltf.get("nodes").unwrap_or(&empty);
        for (part, top) in flat {
            let what = match part {
                None => "it".to_string(),
                Some(p) => format!("moving part {}", node_name(nodes, p)),
            };
            warnings.push(format!("{what} reaches only {} cm up: a house's floor is drawn 1 cm up and hides it (make it reach 1.2 cm)", fixed(top * 100.0, 1)));
        }
    }
    Ok(warnings)
}

/// How far (m) a mesh may be from a functional height (a seat, a top, a mattress) before the gate warns; and
/// how far two sources of one may differ before it calls them different.
const HEIGHT_OFF: f64 = 0.02;
const SAME: f64 = 0.005;
/// A seat's front edge (furniture.json `seats` `front`, `Sampler::front`): where its top has dropped this far (m)
/// below the seat's height (a cushion's rounded front), and how far the model's may be from what the table says.
const FRONT_DROP: f64 = 0.06;
const FRONT_OFF: f64 = 0.03;
/// A seat this low (m) is the ground (furniture.json `seats` `shade`: against a tree's trunk), not the model's to measure.
const GROUND: f64 = 0.02;

/// furniture.json: where residents sit and lie on each object (characters.ts `SEATS`, `BEDS`, `BUNKS` and their
/// defaults). An object in `seats` takes that seat over the content's `seat` (characters.ts `seatOf`).
pub struct Furniture {
    seats: J,
    default_seat: J,
    beds: J,
    default_bed: J,
    bunks: J,
}

impl Furniture {
    /// Reads `paths.furniture`; a file that can't be read is an error naming it.
    pub fn read(paths: &Paths) -> Result<Furniture, String> {
        let f = read_json(&paths.furniture).map_err(|e| format!("{}: {e}", relpath(&paths.furniture, &repo_root())))?;
        let part = |k: &str| f.get(k).cloned().unwrap_or(J::Obj(Vec::new()));
        Ok(Furniture { seats: part("seats"), default_seat: part("defaultSeat"), beds: part("beds"), default_bed: part("defaultBed"), bunks: part("bunks") })
    }
}

/// A number of the content's (`d[k]`), or `default`.
fn number(d: Option<&J>, k: &str, default: f64) -> f64 {
    d.and_then(|d| d.get(k)).and_then(J::num).map_or(default, Num::f)
}

/// A model as the game draws it, its parts where a pose puts them, to measure heights on: its faces (hidden
/// materials, and parts flattened to nothing at rest, left out) and the entry's `scale`.
struct Sampler {
    tris: Vec<Tri>,
    scale: f64,
}

impl Sampler {
    /// None if its positions can't be read here (compressed).
    fn new(g: &Glb, entry: &J, pose: Option<&Pose>) -> R<Option<Sampler>> {
        let materials = g.gltf.get("materials").and_then(J::list).unwrap_or(&[]);
        let no_overrides = J::Obj(Vec::new());
        let overrides = entry.get("materials").unwrap_or(&no_overrides);
        let shown = |i: Option<&J>| -> R<bool> {
            let name = match i {
                Some(i) => index(&J::Arr(materials.to_vec()), i)?.get("name").map(J::key).unwrap_or_default(),
                None => String::new(),
            };
            let ov = override_of(overrides, &name);
            Ok(!(ov.truthy() && ov.get("hide").is_some_and(J::truthy)))
        };
        // (A part flattened to nothing: one of its axes under a hundredth of its longest, G.grow from 0.001.)
        let placed: Vec<Placed> = node_matrices(&g.gltf, pose, None)?
            .into_iter()
            .filter(|p| {
                let axis = |k: usize| hypot(&[p.m[k * 4].f(), p.m[k * 4 + 1].f(), p.m[k * 4 + 2].f()]);
                let (short, long) = (0..3).map(axis).fold((f64::INFINITY, 0.0f64), |(s, l), a| (s.min(a), l.max(a)));
                short >= 0.01 * long
            })
            .collect();
        let fit = entry.get("fit").is_some_and(J::truthy);
        let scale = if fit { 1.0 } else { entry.get("scale").and_then(J::num).map_or(1.0, Num::f) };
        Ok(triangles(g, &placed, &shown)?.map(|tris| Sampler { tris, scale }))
    }

    /// The heights (m) of the faces on the vertical line at (x, depth) (game axes, m from the model's origin):
    /// those looking up, or every face.
    fn line(&self, x: f64, z: f64, up: bool) -> Vec<f64> {
        // glTF x -> game -x; the game's model is the file's times `scale`.
        let (p, d) = ([-x / self.scale, 0.0, z / self.scale], [0.0, -1.0, 0.0]);
        hits(&p, &d, &self.tris).filter(|(_, n)| !up || n[1] > 0.0).map(|(t, _)| -t * self.scale).collect()
    }

    /// The top (m) at (x, depth) nearest `near`: of the faces looking up there, the one nearest that height (what
    /// lies on a top, a vase or a pillow, and what is under it aren't its top). If none looks up (inside out),
    /// any face. None: nothing there.
    fn top(&self, x: f64, z: f64, near: f64) -> Option<f64> {
        self.top_in(x, z, [near, near])
    }

    /// `top`, nearest the range `[lo, hi]` (of those in it, nearest its middle).
    fn top_in(&self, x: f64, z: f64, [lo, hi]: [f64; 2]) -> Option<f64> {
        let off = |h: f64| ((lo - h).max(h - hi).max(0.0), (h - (lo + hi) / 2.0).abs());
        self.tops_at(x, z).into_iter().min_by(|a, b| off(*a).partial_cmp(&off(*b)).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// The heights `top` chooses from at (x, depth): the faces looking up, or every face if none does.
    fn tops_at(&self, x: f64, z: f64) -> Vec<f64> {
        let found = self.line(x, z, true);
        if found.is_empty() { self.line(x, z, false) } else { found }
    }

    /// Where a seat ends in front (m forward of the footprint's middle): from the hips' point (x, depth) forward a
    /// centimetre at a time while the model reaches up to within `FRONT_DROP` of the seat's height `h` there (a
    /// cushion's top, a bean bag's bulge), the last such depth; None: nothing that high under the hips.
    fn front(&self, x: f64, z: f64, h: f64) -> Option<f64> {
        let on = |z: f64| self.span(x, z).is_some_and(|(_, hi)| hi >= h - FRONT_DROP);
        if !on(z) {
            return None;
        }
        let mut at = z;
        while at < z + 1.5 && on(at + 0.01) {
            at += 0.01;
        }
        Some(at)
    }

    /// The lowest and the highest face (m) on the vertical line at (x, depth); None: nothing there.
    fn span(&self, x: f64, z: f64) -> Option<(f64, f64)> {
        let found = self.line(x, z, false);
        let lo = found.iter().copied().reduce(f64::min)?;
        Some((lo, found.into_iter().fold(lo, f64::max)))
    }

    /// A face's corners (m, game axes).
    fn corners(&self, t: &Tri) -> [[f64; 3]; 3] {
        let game = |v: [f64; 3]| [-v[0] * self.scale, v[1] * self.scale, v[2] * self.scale];
        let at = |e: &[f64; 3]| game([t.a[0] + e[0], t.a[1] + e[1], t.a[2] + e[2]]);
        [game(t.a), at(&t.e1), at(&t.e2)]
    }

    /// The box of its faces (m, game axes): (lowest corner, highest); None: no face.
    fn extent(&self) -> Option<([f64; 3], [f64; 3])> {
        let mut corners = self.tris.iter().flat_map(|t| self.corners(t));
        let first = corners.next()?;
        Some(corners.fold((first, first), |(lo, hi), c| ([0, 1, 2].map(|k| lo[k].min(c[k])), [0, 1, 2].map(|k| hi[k].max(c[k])))))
    }

    /// How high it reaches (m) without what stands beside its top `top` higher than it (a dining set's chairs'
    /// backs): the highest of its faces that are no more than `OVER_TOP` over the top, or whose middle, seen from
    /// above, is over the top (the box of the flat faces looking up within 1 cm of it: a vase, a screen on it).
    /// None: no face looks up there.
    fn height_by(&self, top: f64) -> Option<f64> {
        const OVER_TOP: f64 = 0.02;
        let flat_at = |t: &Tri, c: &[[f64; 3]; 3]| t.nrm[1] > 0.9 * hypot(&t.nrm) && c.iter().all(|v| (v[1] - top).abs() <= 0.01);
        let mut area: Option<([f64; 2], [f64; 2])> = None;
        for t in &self.tris {
            let c = self.corners(t);
            if flat_at(t, &c) {
                for v in c {
                    let (lo, hi) = area.get_or_insert(([v[0], v[2]], [v[0], v[2]]));
                    *lo = [lo[0].min(v[0]), lo[1].min(v[2])];
                    *hi = [hi[0].max(v[0]), hi[1].max(v[2])];
                }
            }
        }
        let (lo, hi) = area?;
        let over = |c: &[[f64; 3]; 3]| {
            let (x, z) = ((c[0][0] + c[1][0] + c[2][0]) / 3.0, (c[0][2] + c[1][2] + c[2][2]) / 3.0);
            lo[0] <= x && x <= hi[0] && lo[1] <= z && z <= hi[1]
        };
        self.tris
            .iter()
            .map(|t| self.corners(t))
            .filter(|c| c.iter().all(|v| v[1] <= top + OVER_TOP) || over(c))
            .flat_map(|c| c.map(|v| v[1]))
            .reduce(f64::max)
    }

    /// A stair's steps: on the line up its middle (across), front to back every centimetre, the highest face
    /// looking up (a tread, not the handrails at its sides); the heights of those 2 cm or more up, a centimetre
    /// apart or more, are its steps. (The top step's height, how many); None: none.
    fn steps(&self) -> Option<(f64, usize)> {
        let (lo, hi) = self.extent()?;
        let x = (lo[0] + hi[0]) / 2.0;
        let mut found: Vec<f64> = (0..=((hi[2] - lo[2]) * 100.0) as usize)
            .filter_map(|k| self.line(x, lo[2] + k as f64 / 100.0, true).into_iter().reduce(f64::max))
            .filter(|h| *h >= 0.02)
            .collect();
        found.sort_by(f64::total_cmp);
        found.dedup_by(|a, b| *a - *b < 0.01);
        Some((*found.last()?, found.len()))
    }
}

/// Where the clip `clip` leaves the model's parts: each channel's last key (a bed pulled out or folded down,
/// held open: `motions.open`).
fn open_pose(g: &Glb, clip: &J) -> R<Pose> {
    let empty = J::Arr(Vec::new());
    let nodes = g.gltf.get("nodes").unwrap_or(&empty);
    let mut pose: Pose = HashMap::new();
    for a in g.gltf.get("animations").and_then(J::list).unwrap_or(&[]).iter().filter(|a| a.get("name") == Some(clip)) {
        for c in key(a, "channels")?.list().unwrap_or(&[]) {
            let target = key(c, "target")?;
            let (Some(node), Some(path)) = (target.opt("node"), target.get("path").and_then(J::str)) else { continue };
            let Some(path) = ["translation", "rotation", "scale"].into_iter().find(|p| *p == path) else { continue };
            if let Some(last) = first_value(g, key(sampler(a, c)?, "output")?, true)? {
                pose.insert((node_index(nodes, node)?, path), last.into_iter().map(Num::F).collect());
            }
        }
    }
    Ok(pose)
}

/// A functional height of a content object (where residents sit or lie on it, where things stand on its top,
/// where its lamp shines from) and what its model has there (`heights`).
pub struct Height {
    /// `seat`, `surface`, `mattress`, `lower bunk`, `upper bunk` or `light`.
    pub what: &'static str,
    /// The height the game uses (m), and what says so (content `seat`, furniture.json `seats`, a default).
    pub expected: f64,
    pub source: String,
    /// Where it was measured: m to the right of the footprint's middle and forward of it (game axes).
    pub at: [f64; 2],
    /// The clip it was measured held open at (`motions.open`: a bed pulled out), if any.
    pub open: Option<String>,
    /// The model's top there (m; None: nothing there); for a light, its lowest and highest faces there.
    pub mesh: Option<f64>,
    pub span: Option<(f64, f64)>,
    /// What a second source of it says, when it says otherwise (content `seat` against furniture.json `seats`).
    pub disagrees: Option<String>,
}

impl Height {
    /// How far (m) the model is from it (for a light, how far outside the lamp it shines from); None: nothing there.
    pub fn off(&self) -> Option<f64> {
        if let Some((lo, hi)) = self.span {
            return Some(if self.expected < lo { lo - self.expected } else { f64::max(0.0, self.expected - hi) });
        }
        self.mesh.map(|m| (m - self.expected).abs())
    }

    /// Whether the model is within `HEIGHT_OFF` of it.
    pub fn fits(&self) -> bool {
        self.off().is_some_and(|off| off <= HEIGHT_OFF)
    }

    /// The gate's warnings about it: the model off it, and a second source that says otherwise.
    fn warnings(&self) -> Vec<String> {
        let mut out = Vec::new();
        let open = self.open.as_ref().map(|c| format!(", held open ({})", repr_str(c))).unwrap_or_default();
        let at = format!("x {}, depth {}", fixed(self.at[0], 2), fixed(self.at[1], 2));
        if self.what == "seat front" {
            if self.off().is_none_or(|off| off > FRONT_OFF) {
                out.push(match self.mesh {
                    Some(m) => format!(
                        "seat front {} m forward ({}), the model's seat ends {} m forward there ({at}): {} cm {}",
                        fixed(self.expected, 2),
                        self.source,
                        fixed(m, 2),
                        fixed((m - self.expected).abs() * 100.0, 1),
                        if m < self.expected { "further back" } else { "further forward" }
                    ),
                    None => format!("seat front {} m forward ({}): no seat under the hips ({at})", fixed(self.expected, 2), self.source),
                });
            }
            return out;
        }
        if !self.fits() {
            out.push(match (self.mesh, self.span) {
                (_, Some((lo, hi))) => format!("{} {} m ({}), outside the lamp there (its model {} to {} m at {at})", self.what, fixed(self.expected, 2), self.source, fixed(lo, 2), fixed(hi, 2)),
                (Some(m), _) => format!(
                    "{} {} m ({}{open}), the model {} m there ({at}): {} cm {}",
                    self.what,
                    fixed(self.expected, 2),
                    self.source,
                    fixed(m, 3),
                    fixed((m - self.expected).abs() * 100.0, 1),
                    if m < self.expected { "lower" } else { "higher" }
                ),
                (None, None) => format!("{} {} m ({}{open}): no model under it ({at})", self.what, fixed(self.expected, 2), self.source),
            });
        }
        if let Some(d) = &self.disagrees {
            out.push(format!("{}: {d}", self.what));
        }
        out
    }
}

/// Where residents sit on a content object (`seat_of`): the seat's height (m), what says so, what a second source
/// says when it says otherwise, and the hips' points (m, game axes).
struct Seat {
    h: f64,
    source: &'static str,
    disagrees: Option<String>,
    points: Vec<[f64; 2]>,
    /// Where its top ends in front (furniture.json `seats` `front`, m forward of the middle), if the table says.
    front: Option<f64>,
}

/// Each slot's place of content object `object` (sim-core `slot_position`, `slot_yaw`), `fwd` m ahead of it the
/// way its user faces (m right of the footprint's middle and forward of it): its `places`, else spread across
/// its width, facing its front.
fn slot_points(object: &J, fwd: f64) -> Vec<[f64; 2]> {
    if let Some(places) = object.get("places").and_then(J::list) {
        return places
            .iter()
            .map(|p| {
                let at = |k| p.get("at").and_then(|a| a.at(k)).and_then(J::num).map_or(0.0, Num::f);
                let facing = number(Some(p), "facing", 0.0).to_radians();
                [at(0) + facing.sin() * fwd, at(1) + facing.cos() * fwd]
            })
            .collect();
    }
    let n = object.get("slots").and_then(J::num).map_or(1, |n| n.f().max(1.0) as usize);
    let width = object.get("footprint").and_then(|f| f.at(0)).and_then(J::num).map_or(1.0, Num::f);
    (0..n).map(|k| [(k as f64 - (n as f64 - 1.0) / 2.0) * width / n as f64, fwd]).collect()
}

/// The seat of content object `object` (furniture.json `seats`, else the content's `seat`, else `defaultSeat` for
/// one sat on), the hips' point `fwd` ahead of each slot's place (`slot_points`; along the object for a bath, `along`).
/// None: no one sits on it.
fn seat_of(object: &J, furniture: &Furniture) -> Option<Seat> {
    let id = object.get("id").map(J::key).unwrap_or_default();
    let own = object.opt("seat");
    let table = furniture.seats.get(&id);
    let sits = || object.get("interactions").and_then(J::list).unwrap_or(&[]).iter().any(|i| i.get("pose").is_some_and(|p| p.key() == "sit"));
    if own.is_none() && table.is_none() && !sits() {
        return None;
    }
    let (seat, source) = match (table, own) {
        (Some(t), _) => ((number(Some(t), "h", 0.45), number(Some(t), "fwd", 0.0)), "furniture.json `seats`"),
        (None, Some(o)) => ((number(Some(o), "height", 0.45), number(Some(o), "forward", 0.0)), "content `seat`"),
        (None, None) => ((number(Some(&furniture.default_seat), "h", 0.45), number(Some(&furniture.default_seat), "fwd", 0.0)), "furniture.json `defaultSeat`"),
    };
    let disagrees = match (table, own) {
        (Some(_), Some(o)) => {
            let theirs = (number(Some(o), "height", 0.0), number(Some(o), "forward", 0.0));
            ((theirs.0 - seat.0).abs() > SAME || (theirs.1 - seat.1).abs() > SAME).then(|| {
                format!(
                    "content `seat` says {} m, {} forward; furniture.json `seats` says {} m, {} forward (furniture.json wins: characters.ts `seatOf`)",
                    fixed(theirs.0, 2),
                    fixed(theirs.1, 2),
                    fixed(seat.0, 2),
                    fixed(seat.1, 3)
                )
            })
        }
        _ => None,
    };
    let along = table.and_then(|t| t.get("along")).and_then(J::num).map(Num::f);
    let points = match along {
        Some(a) => vec![[a * seat.1, 0.0]],
        None => slot_points(object, seat.1),
    };
    let front = table.and_then(|t| t.get("front")).and_then(J::num).map(Num::f);
    Some(Seat { h: seat.0, source, disagrees, points, front })
}

/// The functional heights of content object `object` (`Facts::object`) drawn by the model `g` (its `entry`):
/// - its seat (`seat_of`), at each of its hips' points (the worst reported);
/// - its top (content `surface`: at its `at`, else the middle);
/// - its mattress (furniture.json `bunks`, `beds`, else `defaultBed` for one lain on; a hammock's sling at its
///   low point), `fwd` forward, at each slot (in the middle for one lain on across, but a baby's at its slot), held open if it opens;
///   with `every_slot` false at slot 0 only (`cargo review sizes`: a crib's slot 1 is its changing table);
/// - its lamp's light (content `light`, only with an `at`: the shade).
fn heights(g: &Glb, entry: &J, object: &J, furniture: &Furniture, every_slot: bool) -> R<Vec<Height>> {
    let mut out = Vec::new();
    let id = object.get("id").map(J::key).unwrap_or_default();
    let poses: HashSet<String> = object.get("interactions").and_then(J::list).unwrap_or(&[]).iter().filter_map(|i| i.get("pose").map(J::key)).collect();
    let seat = seat_of(object, furniture);
    let lain = furniture.beds.get(&id).is_some() || furniture.bunks.get(&id).is_some() || poses.contains("lie");
    let lamp = object.opt("light").is_some_and(|l| l.opt("at").is_some());
    if !(seat.is_some() || lain || lamp || object.opt("surface").is_some()) {
        return Ok(out);
    }
    // At rest (not `grown`: a tub's water grown to full would be its seat; so a yoga mat, rolled up at rest, has
    // nothing under its seat).
    let rest = Sampler::new(g, entry, None)?;
    // The worst of the points: the farthest off, nothing there worst of all.
    let worst = |s: &Sampler, points: &[[f64; 2]], near: f64| -> ([f64; 2], Option<f64>) {
        let mut best: Option<([f64; 2], Option<f64>)> = None;
        for p in points {
            let m = s.top(p[0], p[1], near);
            let off = |m: Option<f64>| m.map_or(f64::INFINITY, |m| (m - near).abs());
            if best.is_none_or(|(_, b)| off(m) > off(b)) {
                best = Some((*p, m));
            }
        }
        best.unwrap_or(([0.0, 0.0], None))
    };
    let mut height = |what: &'static str, expected: f64, source: String, points: &[[f64; 2]], s: &Sampler, open: Option<String>, disagrees: Option<String>| {
        let (at, mesh) = worst(s, points, expected);
        out.push(Height { what, expected, source, at, open, mesh, span: None, disagrees });
    };

    // Seated (not on the ground: against a tree's trunk, nothing of the model's is sat on).
    let mut seat_front = None;
    if let Some(seat) = seat
        && let Some(s) = &rest
        && seat.h > GROUND
    {
        height("seat", seat.h, seat.source.into(), &seat.points, s, None, seat.disagrees);
        // Where its top ends in front (the shins clear of a deep cushion: characters.ts `perch`), the worst point.
        if let Some(front) = seat.front {
            let mut worst: Option<([f64; 2], Option<f64>)> = None;
            for p in &seat.points {
                let m = s.front(p[0], p[1], seat.h);
                let off = |m: Option<f64>| m.map_or(f64::INFINITY, |m| (m - front).abs());
                if worst.is_none_or(|(_, w)| off(m) > off(w)) {
                    worst = Some((*p, m));
                }
            }
            let (at, mesh) = worst.unwrap_or(([0.0, 0.0], None));
            seat_front = Some(Height { what: "seat front", expected: front, source: "furniture.json `seats`".into(), at, open: None, mesh, span: None, disagrees: None });
        }
    }

    // On a top.
    if let Some(top) = object.opt("surface")
        && let Some(s) = &rest
    {
        let point = match top.get("at").and_then(J::list) {
            Some([x, z, ..]) => [x.num().map_or(0.0, Num::f), z.num().map_or(0.0, Num::f)],
            _ => [0.0, 0.0],
        };
        height("surface", number(Some(top), "height", 0.0), "content `surface`".into(), &[point], s, None, None);
    }

    // Lain on: held open if it opens (a sofa bed's mattress pulled out, a wall bed folded down).
    let bed = furniture.beds.get(&id);
    let bunk = furniture.bunks.get(&id).and_then(J::list);
    if lain {
        let none = J::Obj(Vec::new());
        let open = entry.get("motions").filter(|m| m.truthy()).unwrap_or(&none).opt("open").cloned();
        let opened = match &open {
            Some(clip) => Sampler::new(g, entry, Some(&open_pose(g, clip)?))?,
            None => None,
        };
        let s = match &opened {
            Some(s) => Some(s),
            None => rest.as_ref(),
        };
        if let Some(s) = s {
            let fwd = number(bed, "fwd", 0.0);
            let open = open.map(|c| c.key());
            if let Some(tops) = bunk {
                for (what, h) in ["lower bunk", "upper bunk"].into_iter().zip(tops) {
                    height(what, h.num().map_or(0.0, Num::f), "furniture.json `bunks`".into(), &[[0.0, fwd]], s, open.clone(), None);
                }
            } else {
                let sling = bed.and_then(|b| b.get("sling"));
                let (h, source) = match (bed, sling) {
                    (Some(_), Some(sl)) => (number(Some(sl), "low", 0.5), "furniture.json `beds`, its sling's low point"),
                    (Some(b), None) => (number(Some(b), "h", 0.5), "furniture.json `beds`"),
                    (None, _) => (number(Some(&furniture.default_bed), "h", 0.5), "furniture.json `defaultBed`"),
                };
                // (A baby lies at its slot, on its back along the bed, even on one lain on across: characters.ts.)
                let baby = object.get("interactions").and_then(J::list).is_some_and(|l| l.iter().any(|i| i.get("baby").is_some_and(J::truthy)));
                let across = !baby && bed.and_then(|b| b.get("across")).is_some_and(J::truthy);
                let slots = slot_points(object, fwd);
                let n = if every_slot { slots.len() } else { 1 };
                let points: Vec<[f64; 2]> = if across { vec![[0.0, fwd]] } else { slots.into_iter().take(n).collect() };
                height("mattress", h, source.into(), &points, s, open, None);
            }
        }
    }

    // A lamp's light, at its shade.
    if let Some(light) = object.opt("light").filter(|_| lamp)
        && let Some([x, z, ..]) = light.get("at").and_then(J::list)
        && let Some(s) = &rest
    {
        let at = [x.num().map_or(0.0, Num::f), z.num().map_or(0.0, Num::f)];
        let span = s.span(at[0], at[1]);
        out.push(Height { what: "light", expected: number(Some(light), "height", 1.55), source: "content `light`".into(), at, open: None, mesh: None, span, disagrees: None });
    }
    out.extend(seat_front);
    Ok(out)
}

/// The numbers of one file, for the summary and the far version's checks.
struct Info {
    size: usize,
    tris: i64,
    draws: usize,
    materials: usize,
    dims: [Num; 3],
    lo: [Num; 3],
    clips: usize,
    moving: usize,
}

fn dims_text(dims: &[Num; 3]) -> String {
    dims.iter().map(|d| fixed(d.f(), 2)).collect::<Vec<_>>().join(" x ")
}

/// (errors, warnings, summary) for one model entry and its file (and its far version's, at `far`);
/// `facts`: those of each content object that draws it (`content_objects`); `centre`: where it's built
/// centred (centres.json; None: not built here, not checked).
fn check(fin: &Finishes, entry: &J, path: &str, far: Option<&str>, facts: Option<&Vec<Facts>>, centre: Option<&J>, furniture: Option<&Furniture>) -> R<(Vec<String>, Vec<String>, String)> {
    let (mut errors, mut warnings, info) = check_file(fin, entry, path, facts, centre, furniture)?;
    let mut summary = format!("{:>6} KB {:>6} tris {} draws {:>2} materials  {} m", fixed(info.size as f64 / 1024.0, 1), info.tris, info.draws, info.materials, dims_text(&info.dims));
    if info.clips > 0 {
        summary += &format!("  {} moving parts, {} clips", info.moving, info.clips);
    }
    if let Some(far) = far {
        // Its far version (`far`): the same model, still, with far fewer triangles.
        // (Without what the full model is checked for: its clips, colours and centre, a bird's parts and the perches.)
        let far_entry = J::Obj(entry.entries().unwrap_or(&[]).iter().filter(|(k, _)| !["motions", "paint", "perches", "habitat", "bird"].contains(&k.as_str())).cloned().collect());
        let (mut far_errors, mut far_warnings, far_info) = match check_file(fin, &far_entry, far, None, None, None) {
            Ok((e, w, i)) => (e, w, Some(i)),
            Err(err) => (vec![format!("cannot read {far}: {err}")], Vec::new(), None),
        };
        if let Some(fi) = far_info {
            if fi.clips > 0 {
                far_errors.push("has clips: a far version stands still".into());
            }
            if fi.tris as f64 > info.tris as f64 / 2.0 {
                far_warnings.push(format!("{} triangles, over half the full model's {} (aim for a quarter)", fi.tris, info.tris));
            }
            // It stands where the model stands: decimating can drop the lowest points and lift it off the floor.
            let off = fi.lo[1] - info.lo[1];
            if off.f().abs() > 0.005 {
                far_errors.push(format!(
                    "its lowest point is {} cm off the full model's: a far version stands on the floor as the model does (G.far_of, or far= keeps the floor)",
                    signed((off * Num::I(100)).f(), 1)
                ));
            }
            if fi.dims.iter().zip(&info.dims).any(|(a, b)| (a.f() - b.f()).abs() > f64::max(0.05 * b.f(), 0.02)) {
                far_warnings.push(format!("{} m, the full model {} m", dims_text(&fi.dims), dims_text(&info.dims)));
            }
            summary += &format!("  (far: {} tris)", fi.tris);
        }
        errors.extend(far_errors.into_iter().map(|e| format!("far: {e}")));
        // (What it says of the far version, it says of the full model too.)
        warnings.extend(far_warnings.into_iter().map(|w| format!("far: {w}")));
    }
    Ok((errors, warnings, summary))
}

/// (errors, warnings, numbers) for one model entry and a file of it; `facts`: those of each content object
/// that draws it (`content_objects`), each checked; `centre`: where it's built centred (centres.json), or None.
fn check_file(fin: &Finishes, entry: &J, path: &str, facts: Option<&Vec<Facts>>, centre: Option<&J>, furniture: Option<&Furniture>) -> R<(Vec<String>, Vec<String>, Info)> {
    let (mut errors, mut warnings) = (Vec::new(), Vec::new());
    let data = std::fs::read(path).map_err(|e| py::os_error(&e, path))?;
    let g = parse_glb(&data)?;
    let gltf = &g.gltf;
    let empty = J::Arr(Vec::new());
    let required: BTreeSet<String> = gltf.get("extensionsRequired").and_then(J::list).unwrap_or(&[]).iter().map(J::key).collect();
    for ext in required.iter().filter(|e| !EXTENSIONS.contains(&e.as_str())) {
        errors.push(format!("requires {ext}, which the game cannot read"));
    }
    if gltf.get("skins").is_some_and(J::truthy) {
        errors.push("has a skin: placed things are drawn as thin instances, without bones".into());
    }
    let meshes = gltf.get("meshes").and_then(J::list).unwrap_or(&[]);
    let mut morphs = false;
    'm: for mesh in meshes {
        for prim in key(mesh, "primitives")?.list().unwrap_or(&[]) {
            if prim.get("targets").is_some_and(J::truthy) {
                morphs = true;
                break 'm;
            }
        }
    }
    if morphs {
        errors.push("has morph targets: placed things are drawn as thin instances, without them".into());
    }
    if gltf.get("cameras").is_some_and(J::truthy) {
        warnings.push("has cameras; the game drops them".into());
    }
    let (clip_errors, clip_warnings) = check_clips(&g, entry)?;
    errors.extend(clip_errors);
    warnings.extend(clip_warnings);
    // The parts the player may colour (`paint`: a material's name, or {material, label}): materials of the file.
    let materials = gltf.get("materials").and_then(J::list).unwrap_or(&[]);
    let names: Vec<J> = materials.iter().map(|m| m.get("name").cloned().unwrap_or(J::Str(String::new()))).collect();
    for slot in entry.get("paint").and_then(J::list).unwrap_or(&[]) {
        let name = if matches!(slot, J::Obj(_)) { slot.get("material").cloned().unwrap_or(J::Null) } else { slot.clone() };
        if matches!(slot, J::Obj(_)) && !slot.get("label").is_some_and(J::truthy) {
            errors.push(format!("`paint` part {} has no label", repr(&name)));
        }
        if !names.contains(&name) {
            errors.push(format!("`paint` names {}, which is no material of the file", repr(&name)));
        }
    }
    if gltf.get("extensionsUsed").and_then(J::list).unwrap_or(&[]).contains(&J::Str("KHR_lights_punctual".into())) {
        warnings.push("has lights; the game drops them (a lamp's light is the content's `light`)".into());
    }

    let accessors = gltf.get("accessors").unwrap_or(&empty);
    let no_overrides = J::Obj(Vec::new());
    let overrides = entry.get("materials").unwrap_or(&no_overrides);
    // Mirror glass (a material with `mirror`): flat, on one part (models.ts `fileMirror`).
    let mut glass: BTreeMap<String, (Bounds, Option<usize>)> = BTreeMap::new();
    for (name, ov) in overrides.entries().unwrap_or(&[]) {
        let mirror = ov.opt("mirror");
        let truthy = mirror.is_some_and(J::truthy);
        if !matches!(mirror.and_then(J::str), None | Some("front") | Some("back")) || mirror.is_some_and(|m| m.str().is_none()) {
            errors.push(format!("material {}: `mirror` is front or back, not {}", repr_str(name), repr(mirror.unwrap())));
        } else if truthy && !names.contains(&J::Str(name.clone())) {
            errors.push(format!("material {} is mirror glass, but no material of the file", repr_str(name)));
        } else if truthy && fin.see_through(ov.opt("finish")) {
            warnings.push(format!(
                "mirror glass {} is see-through ({}): where nothing reflects (Medium, Low) it shows what is behind it; map it to gloss",
                repr_str(name),
                str_of(key(ov, "finish")?)
            ));
        }
        if let Some(glow) = ov.opt("glow") {
            let ok = glow.str().is_some_and(|s| s.chars().count() == 7 && s.starts_with('#') && s.chars().skip(1).all(|c| c.is_ascii_hexdigit()));
            if !ok {
                errors.push(format!("material {}: `glow` is a colour, #rrggbb, not {}", repr_str(name), repr(glow)));
            }
        }
        if let Some(screen) = ov.get("screen") {
            if !matches!(screen, J::Bool(_)) {
                errors.push(format!("material {}: `screen` is true or false, not {}", repr_str(name), repr(screen)));
            } else if screen.truthy() && ov.opt("glow").is_none() {
                errors.push(format!("material {}: `screen` needs a `glow` colour", repr_str(name)));
            }
        }
    }
    let (inf, ninf) = (Num::F(f64::INFINITY), Num::F(f64::NEG_INFINITY));
    let (mut lo, mut hi) = ([inf; 3], [ninf; 3]);
    let mut tris = 0i64;
    let mut groups: BTreeSet<(Option<usize>, String)> = BTreeSet::new();
    // Per problem, the materials that have it.
    let mut flags: HashMap<&str, BTreeSet<String>> = HashMap::new();
    // Draw calls: one per finish group, the still body's and each moving part's apart.
    for p in node_matrices(gltf, None, None)? {
        for prim in primitives(gltf, &p.mesh)? {
            if !prim.get("mode").is_none_or(|m| m.num().is_some_and(|n| n.f() == 4.0)) {
                errors.push("a primitive that is not triangles".into());
                continue;
            }
            let pos = index(accessors, key(key(prim, "attributes")?, "POSITION")?)?;
            let counted = match prim.get("indices") {
                Some(i) => index(accessors, i)?,
                None => pos,
            };
            let n = key(counted, "count")?.num().map_or(0, |n| n.f() as i64);
            tris += n.div_euclid(3);
            // Bounds: the accessor's box through the node matrix, then glTF x -> game -x (as `render/pc/models.ts` bakes it).
            let (bl, bh) = box_of(pos)?;
            for cx in [bl[0], bh[0]] {
                for cy in [bl[1], bh[1]] {
                    for cz in [bl[2], bh[2]] {
                        let mut q = apply(&p.m, &[cx, cy, cz]);
                        q[0] = -q[0];
                        for k in 0..3 {
                            lo[k] = py::min(lo[k], q[k]);
                            hi[k] = py::max(hi[k], q[k]);
                        }
                    }
                }
            }
            let unnamed = J::Obj(vec![("name".into(), J::Str(String::new()))]);
            let mat = match prim.get("material") {
                Some(i) => index(&J::Arr(materials.to_vec()), i)?.clone(),
                None => unnamed,
            };
            let name = mat.get("name").map(J::key).unwrap_or_default();
            let ov = override_of(overrides, &name);
            let ov_on = ov.truthy();
            if ov_on && ov.get("mirror").is_some_and(J::truthy) {
                let ((glo, ghi), part) = glass.entry(name.clone()).or_insert((([inf; 3], [ninf; 3]), p.part));
                let (bl, bh) = box_of(pos)?;
                for k in 0..3 {
                    let mut l = Num::I(0);
                    let mut h = Num::I(0);
                    for j in 0..3 {
                        let mj = p.m[j * 4 + k];
                        l = l + mj * if mj.f() >= 0.0 { bl[j] } else { bh[j] };
                        h = h + mj * if mj.f() >= 0.0 { bh[j] } else { bl[j] };
                    }
                    glo[k] = py::min(glo[k], l + p.m[12 + k]);
                    ghi[k] = py::max(ghi[k], h + p.m[12 + k]);
                }
                if *part != p.part {
                    errors.push(format!("mirror glass {} is on two parts: one material per mirror", repr_str(&name)));
                }
            }
            let pbr = mat.get("pbrMetallicRoughness");
            let textured = ["baseColorTexture", "metallicRoughnessTexture"].iter().any(|k| pbr.is_some_and(|p| p.has(k)))
                || ["normalTexture", "occlusionTexture", "emissiveTexture"].iter().any(|k| mat.has(k));
            let finish = if ov_on { ov.get("finish").filter(|f| f.truthy()) } else { None };
            if key(prim, "attributes")?.has("TEXCOORD_0") && (!textured || finish.is_some()) {
                flags.entry("uvs").or_default().insert(name.clone());
            }
            if coloured(&g, prim)? {
                flags.entry("coloured").or_default().insert(name.clone());
            }
            if ov_on && ov.get("hide").is_some_and(J::truthy) {
                continue;
            }
            if let Some(finish) = finish {
                let f = finish.key();
                if !fin.plain.contains_key(&f) {
                    errors.push(format!("material {} maps to {f}, which is no finish in the manifests", repr_str(&name)));
                }
                groups.insert((p.part, if fin.plain.get(&f) == Some(&true) { "plain".into() } else { f }));
                continue;
            }
            groups.insert((p.part, format!("file:{name}")));
            flags.entry("unmapped").or_default().insert(name.clone());
            if mat.get("doubleSided").is_some_and(J::truthy) {
                flags.entry("double").or_default().insert(name.clone());
            }
            if mat.get("alphaMode") == Some(&J::Str("BLEND".into())) {
                flags.entry("blend").or_default().insert(name.clone());
            }
            if mat.get("emissiveFactor").map(|e| e.list().unwrap_or(&[]).iter().any(J::truthy)).unwrap_or(false) {
                flags.entry("emissive").or_default().insert(name.clone());
            }
        }
    }
    for (name, ((glo, ghi), _part)) in &glass {
        // (not `size`: that is the file's, for the summary)
        let mut extent: Vec<Num> = (0..3).map(|k| ghi[k] - glo[k]).collect();
        extent.sort_by(|a, b| a.f().partial_cmp(&b.f()).unwrap_or(std::cmp::Ordering::Equal));
        if extent[0].f() > 0.1 * extent[1].f() {
            warnings.push(format!(
                "mirror glass {} isn't flat ({} cm thick, {} cm across): the reflection lies on its box's thinnest side",
                repr_str(name),
                fixed((extent[0] * Num::I(100)).f(), 1),
                fixed((extent[1] * Num::I(100)).f(), 0)
            ));
        }
    }
    for (flag, text) in [
        ("unmapped", "not mapped to a finish in the entry's `materials` (each its own draw call and look)"),
        ("double", "double-sided (every back face drawn too)"),
        ("blend", "see-through (BLEND: drawn after everything else, unsorted between instances)"),
        ("emissive", "glowing by day too (emissive; map it to a finish, the lamp lights it at night)"),
        ("uvs", "carrying UVs nothing reads (export without them)"),
    ] {
        if let Some(names) = flags.get(flag).filter(|n| !n.is_empty()) {
            warnings.push(format!("materials {text}: {}", names.iter().cloned().collect::<Vec<_>>().join(", ")));
        }
    }
    if let Some(names) = flags.get("coloured").filter(|n| !n.is_empty()) {
        let names = names.iter().cloned().collect::<Vec<_>>().join(", ");
        errors.push(format!("materials with a coloured COLOR_0: {names} (the game multiplies it in as baked shading, grey: gamekit/shade.py)"));
    }

    // Units and origin: on the floor, centred on the footprint (models.ts lines files up by neither).
    let fit = entry.get("fit").is_some_and(J::truthy);
    let scale = if fit { Num::I(1) } else { entry.get("scale").map(|s| s.num().ok_or("can't multiply sequence by non-int")).transpose()?.unwrap_or(Num::I(1)) };
    let lo = lo.map(|v| v * scale);
    let hi = hi.map(|v| v * scale);
    let dims = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    // As it shows: the parts a clip grows (hidden or small at rest: darts, a pot thrown) grown, for what
    // is compared with the content (the footprint, the wall, what the floor hides, a top's disc).
    let none = J::Obj(Vec::new());
    let motions = entry.get("motions").filter(|m| m.truthy()).unwrap_or(&none);
    let pose = grown(&g, &[motions.opt("open").cloned(), motions.opt("night").cloned()])?;
    let ((slo, shi), sparts) = bounds(gltf, &pose)?;
    let (slo, shi) = (slo.map(|v| v * scale), shi.map(|v| v * scale));
    if !fit && !entry.get("offset").is_some_and(J::truthy) {
        // On the floor (y = 0). A model the game sinks may reach below it by its `sink` (the woods' rock, the marker).
        let sink = entry.get("sink").map(|s| s.num().ok_or("unsupported operand type(s) for -")).transpose()?.unwrap_or(Num::I(0));
        if lo[1].f() < (Num::F(-0.005) - sink).f() {
            errors.push(format!("reaches {} cm below the floor (origin on the floor, y = 0)", fixed((-lo[1] * Num::I(100)).f(), 1)));
        } else if lo[1].f() > 0.005 {
            warnings.push(format!("floats {} cm above its origin (fine only on a wall or ceiling)", fixed((lo[1] * Num::I(100)).f(), 1)));
        }
        // Centred where its `Model` says (centres.json, which the build writes: the middle, or its `centre`, a
        // fridge against the wall, a wall bed at the back of its tiles), at rest as `G.settle` centres it (grown,
        // the dartboard's darts and the unrolled yoga mat reach forward). A file not built here isn't checked.
        if let Some(centre) = centre.filter(|c| **c != J::Null) {
            let at = (((lo[0] + hi[0]) / Num::I(2)).f(), ((lo[2] + hi[2]) / Num::I(2)).f());
            let c0 = centre.at(0).ok_or("list index out of range")?.num().ok_or("unsupported operand type(s) for -")?.f();
            let c1 = centre.at(1).ok_or("list index out of range")?.num().ok_or("unsupported operand type(s) for -")?.f();
            if f64::max((at.0 - c0).abs(), (at.1 - c1).abs()) > OFF_CENTRE {
                warnings.push(format!(
                    "centred on (x {}, depth {}), its Model's `centre` is ({}, {}) (centres.json): rebuild it, or `cargo art --centres` if the Model changed",
                    fixed(at.0, 3),
                    fixed(at.1, 3),
                    fixed(c0, 3),
                    fixed(c1, 3)
                ));
            }
        }
        // Against what each content object that draws it says of it (two objects may share a model).
        let mut seen = HashSet::new();
        let each: Vec<Option<&Facts>> = match facts.filter(|f| !f.is_empty()) {
            Some(f) => f.iter().map(Some).collect(),
            None => vec![None],
        };
        let many = facts.is_some_and(|f| f.len() > 1);
        for fact in each {
            // Its functional heights against the model (`heights`: warnings only).
            let mut found = content_warnings(fact, &lo, &(slo, shi), &sparts, scale, &g, &pose, entry)?;
            if let (Some(f), Some(furniture)) = (fact, furniture) {
                found.extend(heights(&g, entry, &f.object, furniture, true)?.iter().flat_map(Height::warnings));
            }
            for w in found {
                let w = if many { format!("{w} (object {})", str_of(&fact.unwrap().id)) } else { w };
                if seen.insert(w.clone()) {
                    warnings.push(w);
                }
            }
        }
    }
    // Parts flattened to nothing at rest whose flat shape shows (README trap "Grown from nothing").
    let opaque = |index_: Option<&J>| -> R<bool> {
        let mat = match index_ {
            Some(i) => index(&J::Arr(materials.to_vec()), i)?.clone(),
            None => J::Obj(vec![("name".into(), J::Str(String::new()))]),
        };
        let ov = override_of(overrides, &mat.get("name").map(J::key).unwrap_or_default());
        if ov.truthy() && ov.get("hide").is_some_and(J::truthy) {
            return Ok(false);
        }
        Ok(match ov.get("finish").filter(|f| ov.truthy() && f.truthy()) {
            Some(f) => !fin.see_through(Some(f)),
            None => mat.get("alphaMode") != Some(&J::Str("BLEND".into())),
        })
    };
    for (name, share) in flattened_in_view(&g, &opaque)? {
        warnings.push(format!(
            "{name} lies flattened at rest with {} of it outside anything opaque, so its flat shape shows: put its origin inside something opaque (README trap \"Grown from nothing\")",
            percent(share)
        ));
    }

    // A bird the renderer flies (`bird`): its parts and size against the file; where birds sit on the model
    // (`perches`): inside it, and what they are on (`habitat`).
    if let Some(bird) = entry.get("bird") {
        let fields = bird_fields(bird);
        if fields.is_empty() {
            errors.extend(bird_errors(bird, &bird_parts(&g)?, dims.map(|d| d.f())));
        }
        errors.extend(fields);
    }
    if let Some(perches) = entry.get("perches") {
        errors.extend(perch_errors(perches, lo.map(|v| v.f()), hi.map(|v| v.f()), scale.f()));
    }
    errors.extend(habitat_errors(entry));

    // Budgets.
    let side = dims.iter().copied().reduce(|m, v| if v.f() > m.f() { v } else { m }).unwrap();
    let (target, most) = TRIANGLES.iter().find(|(up, _, _)| side.f() <= *up).map(|(_, t, m)| (*t, *m)).ok_or("StopIteration")?;
    if tris > most {
        errors.push(format!("{tris} triangles, at most {most} for a model {} m across (aim for {target})", fixed(side.f(), 2)));
    } else if tris > target {
        warnings.push(format!("{tris} triangles, aim for {target} for a model {} m across (at most {most})", fixed(side.f(), 2)));
    }
    if groups.len() > DRAWS.1 {
        errors.push(format!("{} draw calls, at most {} (one per finish group: map materials to finishes)", groups.len(), DRAWS.1));
    } else if groups.len() > DRAWS.0 {
        warnings.push(format!("{} draw calls, aim for {} at most", groups.len(), DRAWS.0));
    }
    for (i, img) in gltf.get("images").and_then(J::list).unwrap_or(&[]).iter().enumerate() {
        let wh = match img.get("bufferView") {
            Some(v) => {
                let view = item(gltf, "bufferViews", v)?;
                let start = glb::int(view, "byteOffset", 0)?.max(0) as usize;
                let len = key(view, "byteLength")?.num().map_or(0, |n| n.f() as i64).max(0) as usize;
                image_size(&g.bin[start.min(g.bin.len())..(start + len).min(g.bin.len())])
            }
            None => None,
        };
        if let Some((w, h)) = wh
            && w.max(h) > TEXTURE_MAX
        {
            errors.push(format!("texture {i} is {w}x{h}, at most {TEXTURE_MAX} px a side"));
        }
        let label = img.get("name").or(img.get("uri")).map(str_of).unwrap_or("embedded".into());
        warnings.push(format!("texture {i} ({label}): a texture of its own, prefer the game's finishes"));
    }

    let info = Info {
        size: g.size,
        tris,
        draws: groups.len(),
        materials: materials.len(),
        dims,
        lo,
        clips: gltf.get("animations").and_then(J::list).map_or(0, |a| a.len()),
        moving: moved_nodes(gltf)?.len(),
    };
    Ok((errors, warnings, info))
}

/// What the gate found: its text, the keys it checked (in order) and whether anything failed.
pub struct Report {
    pub text: String,
    pub checked: Vec<String>,
    pub failed: usize,
}

/// Runs the gate over `keys` (every model if none), as check_glb.py's `main` did, into a report.
pub fn gate(paths: &Paths, keys: &[String]) -> Report {
    let mut text = String::new();
    let fail = |text: String| Report { text, checked: Vec::new(), failed: 1 };
    let read = || -> Result<_, String> {
        let stack = asset_manifests(paths)?;
        let defs = content(paths)?;
        Ok((stack, content_objects(&defs)?, content_keys(&defs)?, Furniture::read(paths)?))
    };
    let (stack, facts, named, furniture) = match read() {
        Ok(r) => r,
        Err(err) => return fail(format!("FAIL content and manifests\n       error: cannot read them: {err}\n")),
    };
    let (entries, fin) = model_entries(&stack);
    let centres = match read_centres(&paths.centres) {
        Ok(c) => c,
        Err(err) => return fail(format!("FAIL centres.json\n       error: cannot read it: {err}\n")),
    };
    let mut failed = 0;
    let mut checked = Vec::new();
    // What can't be read at all: a model the content draws (or the command line names) that no manifest has,
    // or whose entry has no file once the packs are stacked (its pack missing from `packs`), and a file that
    // is no .glb. The game draws a box for each.
    let mut problems: BTreeMap<String, String> = BTreeMap::new();
    let wanted = |k: &str| keys.is_empty() || keys.iter().any(|x| x == k);
    let all: BTreeSet<&String> = named.keys().chain(keys.iter()).collect();
    for k in all {
        if !wanted(k) {
            continue;
        }
        let what = named.get(k).map(String::as_str).unwrap_or("the command line");
        let url = entries.get(k).and_then(|(e, _)| e.get("url")).is_some_and(J::truthy);
        if k.is_empty() {
            problems.insert(k.clone(), format!("{what} has an empty `model`: the game looks up \"\" and draws a box (leave it out to draw model.<id>)"));
        } else if !entries.contains_key(k) {
            problems.insert(k.clone(), format!("{what} names it, but no manifest has a model entry {k}"));
        } else if !url {
            problems.insert(k.clone(), format!("{what} names it, but its entry has no `url` once the packs are stacked (is its pack in the base manifest's `packs`?)"));
        }
    }
    for (k, (entry, _)) in &entries {
        if let Some(url) = entry.get("url").and_then(J::str).filter(|u| !u.is_empty())
            && !url.to_lowercase().ends_with(".glb")
            && !url.contains("://")
            && wanted(k)
        {
            problems.insert(k.clone(), format!("its file {url} is no .glb: the game's models are glTF binaries"));
        }
    }
    for (k, problem) in &problems {
        failed += 1;
        text += &format!("FAIL {}\n       error: {problem}\n", if k.is_empty() { "(empty model)" } else { k });
    }
    for (k, (entry, manifest)) in &entries {
        let url = entry.get("url").filter(|u| u.truthy()).map(J::key).unwrap_or_default();
        if !wanted(k) || problems.contains_key(k) || url.is_empty() || url.contains("://") {
            continue;
        }
        let path = normpath(&join(&dirname(manifest), &url));
        let far = entry.get("far").filter(|f| f.truthy()).map(|f| normpath(&join(&dirname(manifest), &f.key())));
        let (errors, warnings, summary) = check(&fin, entry, &path, far.as_deref(), facts.get(k), centres.get(k), Some(&furniture)).unwrap_or_else(|err| (vec![format!("cannot read {path}: {err}")], Vec::new(), String::new()));
        failed += !errors.is_empty() as usize;
        checked.push(k.clone());
        text += &format!("{} {k:28} {summary}\n", if errors.is_empty() { "ok  " } else { "FAIL" });
        for e in errors {
            text += &format!("       error: {e}\n");
        }
        for w in warnings {
            text += &format!("       warning: {w}\n");
        }
    }
    text += &format!("{} models, {failed} failed", checked.len());
    if !problems.is_empty() {
        text += &format!(" ({} without a file it can read)", problems.len());
    }
    text += "\n";
    Report { text, checked, failed }
}

/// The model entries of the manifests once stacked ({key: (entry, the manifest it's in)}), and their finishes.
fn model_entries(stack: &[(String, J)]) -> (BTreeMap<String, (J, String)>, Finishes) {
    let mut entries: BTreeMap<String, (J, String)> = BTreeMap::new();
    let mut fin = Finishes { plain: HashMap::new(), see_through: HashSet::new() };
    for (path, m) in stack {
        for (k, entry) in m.get("entries").filter(|e| e.truthy()).and_then(J::entries).unwrap_or(&[]) {
            let kind = entry.get("type");
            if kind == Some(&J::Str("model".into())) {
                // A pack's entry replaces the entry, keeping its `vary` unless it gives its own
                // (registry.ts `extended`).
                let mut entry = entry.clone();
                if let Some((old, _)) = entries.get(k).filter(|(old, _)| old.truthy())
                    && let (Some(vary), J::Obj(fields)) = (old.get("vary"), &mut entry)
                {
                    match fields.iter().position(|(f, _)| f == "vary") {
                        Some(at) => {
                            let mine = fields.remove(at);
                            fields.insert(0, mine);
                        }
                        None => fields.insert(0, ("vary".into(), vary.clone())),
                    }
                }
                entries.insert(k.clone(), (entry, path.clone()));
            } else if kind == Some(&J::Str("material".into())) && k.starts_with("material.finish.") {
                let alpha = entry.get("alpha").and_then(J::num).map_or(1.0, |a| a.f());
                fin.plain.insert(k.clone(), !entry.get("texture").is_some_and(J::truthy) && alpha >= 1.0);
                if alpha < 1.0 {
                    fin.see_through.insert(k.clone());
                }
            }
        }
    }
    (entries, fin)
}

/// A model file read: its size at rest (m, game axes: across its front x, up, front to back, as the gate's
/// summary has it), the file and its entry (for `heights`).
struct Model {
    size: [f64; 3],
    file: Glb,
    entry: J,
}

/// A content object's model, its size (`Model`), its functional heights (`heights`, a mattress at slot 0 only)
/// and what `cargo review sizes` measures besides.
#[derive(Default)]
pub struct Measure {
    pub model: String,
    pub size: [f64; 3],
    pub heights: Vec<Height>,
    /// Its top where the content has no `surface` but its kind has one (`top_near`): where (m, game axes: right
    /// of the footprint's middle, forward of it) and how high.
    pub top: Option<([f64; 2], f64)>,
    /// How high it reaches without its seats, for one sat at that has a top (`Sampler::height_by`).
    pub table: Option<f64>,
    /// A stair's (content `stairs`) top step's height and how many steps it has (`Sampler::steps`).
    pub steps: Option<(f64, usize)>,
}

/// The top of an object with no content `surface` whose kind has one, `range` (m), measured as `heights`
/// measures a seat: on vertical lines near the middle, the face looking up nearest the range. The lines are at
/// the middle of its faces' box seen from above (a thing against a wall stands off its footprint's middle) and
/// at the eight points a quarter of its width and depth from there: a worktop beside a basin or a hob in the
/// middle, while a quarter in is still on a table's top, inside its rim or on its board. Lines within
/// `SEAT_CLEAR` of a seat's hips (`seats`) are left out (a chair's seat or back is no top), unless none of the
/// others finds a face (a stash box sat on: its top is its seat). Of the faces found, the highest that doesn't
/// stand on another: one with a face 2 cm or more under it on its line that is what another line found is a
/// thing on the top (a vase, a tool, a plate: the top goes on under it), while a basin's or a hob's floor is
/// lower than the worktop round it, which has no top under it. (Where, how high); None: no face looks up there.
fn top_near(s: &Sampler, range: [f64; 2], seats: &[[f64; 2]]) -> Option<([f64; 2], f64)> {
    // (line, the face nearest the range there, every face there).
    let on = |lines: Vec<[f64; 2]>| -> Vec<([f64; 2], f64, Vec<f64>)> { lines.iter().filter_map(|p| Some((*p, s.top_in(p[0], p[1], range)?, s.tops_at(p[0], p[1])))).collect() };
    let mut found = on(top_lines(s, seats));
    if found.is_empty() {
        found = on(top_lines(s, &[]));
    }
    let stands = |(_, h, faces): &([f64; 2], f64, Vec<f64>)| faces.iter().any(|q| *q <= h - 0.02 && found.iter().any(|(_, other, _)| (other - q).abs() <= 0.01));
    found.iter().filter(|f| !stands(f)).map(|(p, h, _)| (*p, *h)).max_by(|a, b| a.1.total_cmp(&b.1))
}

/// The vertical lines `top_near` looks along, those within `SEAT_CLEAR` of `seats` left out (m, game axes;
/// across, then front to back).
fn top_lines(s: &Sampler, seats: &[[f64; 2]]) -> Vec<[f64; 2]> {
    const SEAT_CLEAR: f64 = 0.3;
    let Some((lo, hi)) = s.extent() else { return Vec::new() };
    let mid = [(lo[0] + hi[0]) / 2.0, (lo[2] + hi[2]) / 2.0];
    let quarter = [(hi[0] - lo[0]) / 4.0, (hi[2] - lo[2]) / 4.0];
    let grid = [-1.0, 0.0, 1.0].iter().flat_map(|i| [-1.0, 0.0, 1.0].map(|k| [mid[0] + i * quarter[0], mid[1] + k * quarter[1]]));
    grid.filter(|p| seats.iter().all(|s| dist(p, s) >= SEAT_CLEAR)).collect()
}

/// Measures content object `f` drawn by the model `m` (its `key`) for `cargo review sizes`; `top` is its kind's
/// top range.
fn measure(key: &str, m: &Model, f: &Facts, furniture: &Furniture, top: Option<[f64; 2]>) -> R<Measure> {
    let mut out = Measure { model: key.into(), size: m.size, heights: heights(&m.file, &m.entry, &f.object, furniture, false)?, ..Measure::default() };
    let seats = seat_of(&f.object, furniture).map(|s| s.points).unwrap_or_default();
    let top = top.filter(|_| f.object.opt("surface").is_none());
    if top.is_none() && seats.is_empty() && !f.stairs {
        return Ok(out);
    }
    let Some(s) = Sampler::new(&m.file, &m.entry, None)? else { return Ok(out) };
    out.top = top.and_then(|range| top_near(&s, range, &seats));
    let on = out.heights.iter().find(|h| h.what == "surface").and_then(|h| h.mesh).or(out.top.map(|(_, h)| h));
    if !seats.is_empty()
        && let Some(on) = on
    {
        out.table = s.height_by(on);
    }
    if f.stairs {
        out.steps = s.steps();
    }
    Ok(out)
}

/// Every model the manifests have a file for, measured ({key: its size, or why it can't be read}), and every
/// content object's measure ({id: …}; an object whose model can't be read has the reason).
pub struct Measures {
    pub models: BTreeMap<String, Result<[f64; 3], String>>,
    pub objects: BTreeMap<String, Result<Measure, String>>,
}

/// Reads and measures a model file of the manifests (`path`) with its entry: its size at rest, times `scale`.
fn model(entry: &J, path: &str) -> R<Model> {
    let file = parse_glb(&std::fs::read(path).map_err(|e| py::os_error(&e, path))?)?;
    let ((lo, hi), _) = bounds(&file.gltf, &HashMap::new())?;
    let fit = entry.get("fit").is_some_and(J::truthy);
    let scale = if fit { 1.0 } else { entry.get("scale").and_then(J::num).map_or(1.0, Num::f) };
    Ok(Model { size: [0, 1, 2].map(|k| (hi[k].f() - lo[k].f()) * scale), file, entry: entry.clone() })
}

/// Measures every model and content object (`Measures`), for `cargo review sizes`; `tops`: {object id: its
/// kind's top range}, for those whose content has no `surface` (`top_near`).
pub fn measures(paths: &Paths, tops: &BTreeMap<String, [f64; 2]>) -> Result<Measures, String> {
    let stack = asset_manifests(paths)?;
    let defs = content(paths)?;
    let facts = content_objects(&defs)?;
    let furniture = Furniture::read(paths)?;
    let (entries, _) = model_entries(&stack);
    let mut read: BTreeMap<String, Result<Model, String>> = BTreeMap::new();
    for (k, (entry, manifest)) in &entries {
        let url = entry.get("url").filter(|u| u.truthy()).map(J::key).unwrap_or_default();
        let found = if url.is_empty() || url.contains("://") {
            Err(format!("its entry has no file here ({})", if url.is_empty() { "no `url`" } else { &url }))
        } else {
            let path = normpath(&join(&dirname(manifest), &url));
            model(entry, &path).map_err(|e| format!("cannot read {path}: {e}"))
        };
        read.insert(k.clone(), found);
    }
    let mut objects = BTreeMap::new();
    for (k, each) in &facts {
        for f in each {
            let measured = match read.get(k) {
                None => Err(format!("no manifest has a model entry {k}")),
                Some(Err(e)) => Err(e.clone()),
                Some(Ok(m)) => measure(k, m, f, &furniture, tops.get(&str_of(&f.id)).copied()),
            };
            objects.insert(str_of(&f.id), measured);
        }
    }
    let models = read.into_iter().map(|(k, m)| (k, m.map(|m| m.size))).collect();
    Ok(Measures { models, objects })
}

/// `cargo check-glb [keys…] [--keys] [--preview]`: the exit code.
pub fn main(args: &[String]) -> i32 {
    let paths = Paths::repo();
    if args.iter().any(|a| a == "--keys") {
        return match known_keys(&paths) {
            Ok(keys) => {
                let mut out = String::new();
                for k in keys {
                    out += &k;
                    out.push('\n');
                }
                print!("{out}");
                0
            }
            Err(err) => {
                eprintln!("cannot read the content or the manifests: {err}");
                1
            }
        };
    }
    let keys: Vec<String> = args.iter().filter(|a| a.starts_with("model.") || a.starts_with("item.") || a.starts_with("tool.")).cloned().collect();
    let report = gate(&paths, &keys);
    print!("{}", report.text);
    let mut code = if report.failed > 0 { 1 } else { 0 };
    if args.iter().any(|a| a == "--preview") && !report.checked.is_empty() {
        // Re-imports each file in Blender and renders it to previews/<family>/rt_<name>.png, so export problems show.
        let review = join(&paths.here, "gamekit/review.py");
        let ok = crate::art::blender().map(|exe| crate::art::echo(&crate::art::run(&exe, &review, &report.checked)));
        match ok {
            Ok(true) => {}
            Ok(false) => code = 1,
            Err(err) => {
                eprintln!("{err}");
                code = 1;
            }
        }
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centres_one_key_a_line_sorted_to_a_tenth_of_a_millimetre() {
        let mut c = BTreeMap::new();
        c.insert("model.b".to_string(), [0.0, -0.068_96]);
        c.insert("model.a".to_string(), [-0.000_01, 1.234_56]);
        c.insert("model.c".to_string(), [0.5, 2.0]);
        assert_eq!(centres_text(&c), "{\n  \"model.a\": [0.0, 1.2346],\n  \"model.b\": [0.0, -0.069],\n  \"model.c\": [0.5, 2.0]\n}\n");
    }

    #[test]
    fn the_committed_centres_are_in_that_format() {
        let paths = Paths::repo();
        let text = std::fs::read_to_string(&paths.centres).unwrap();
        let read = read_centres(&paths.centres).unwrap();
        let c: BTreeMap<String, [f64; 2]> = read.entries().unwrap().iter().map(|(k, v)| (k.clone(), [v.at(0).unwrap().num().unwrap().f(), v.at(1).unwrap().num().unwrap().f()])).collect();
        assert_eq!(c.len(), read.entries().unwrap().len(), "a key twice");
        assert_eq!(centres_text(&c), text);
    }

    #[test]
    fn named_finishes() {
        let finish = |n: &str| named_finish(n).map(|f| f.get("finish").unwrap().key());
        assert_eq!(finish("stone@matte").as_deref(), Some("material.finish.matte"));
        assert_eq!(finish("tunic@fabric#body").as_deref(), Some("material.finish.fabric"));
        assert_eq!(finish("tunic#body@fabric").as_deref(), Some("material.finish.fabric"));
        assert_eq!(finish("tunic#bodyx@fabric").as_deref(), Some("material.finish.fabric"));
        assert_eq!(finish("a@b#bodyx"), None, "only a slot before @ or at the end goes");
        assert_eq!(finish("x@matte#hair#skin"), None);
        assert_eq!(finish("x@mätte").as_deref(), Some("material.finish.mätte"));
        assert_eq!(finish("@matte"), None);
        assert_eq!(finish("stone@Matte"), None);
        assert_eq!(finish("stone@matte2"), None);
        assert_eq!(finish("stone"), None);
    }

    /// One triangle whose vertex colour is `colour` at each corner (normalised u16 RGBA, as Blender writes it).
    fn shaded(colour: [u16; 4]) -> Glb {
        let mut bin: Vec<u8> = [[0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]].iter().flatten().flat_map(|c| c.to_le_bytes()).collect();
        for _ in 0..3 {
            bin.extend(colour.iter().flat_map(|c| c.to_le_bytes()));
        }
        let json = r#"{"asset": {"version": "2.0"}, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
            "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "COLOR_0": 1}}]}],
            "bufferViews": [{"byteOffset": 0, "byteLength": 36}, {"byteOffset": 36, "byteLength": 24}],
            "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 3, "min": [0, 0, 0], "max": [1, 0, 1]},
                {"bufferView": 1, "componentType": 5123, "normalized": true, "type": "VEC4", "count": 3}]}"#;
        parse_glb(&glb::tests::glb(json, &bin)).unwrap()
    }

    #[test]
    fn baked_shading_is_grey() {
        let prim = |g: &Glb| key(item(&g.gltf, "meshes", &J::Int(0)).unwrap(), "primitives").unwrap().list().unwrap()[0].clone();
        let grey = shaded([40000, 40000, 40000, 65535]);
        assert!(!coloured(&grey, &prim(&grey)).unwrap());
        let tinted = shaded([40000, 20000, 40000, 65535]);
        assert!(coloured(&tinted, &prim(&tinted)).unwrap());
    }

    /// A sofa bed of two quads facing up: the seat (glTF x -0.5 to 0.1, so game x -0.1 to 0.5) 0.41 m up, and the
    /// mattress 0.40 m up under it, which `bed_out` slides 0.6 m forward.
    fn sofa_bed() -> Glb {
        let mut bin = Vec::new();
        for (y, x1) in [(0.41f32, 0.1f32), (0.40, 0.5)] {
            for v in [[-0.5, y, -0.5], [-0.5, y, 0.5], [x1, y, 0.5], [x1, y, -0.5]] {
                bin.extend(v.iter().flat_map(|c: &f32| c.to_le_bytes()));
            }
        }
        bin.extend([0u16, 1, 2, 0, 2, 3].iter().flat_map(|i| i.to_le_bytes()));
        bin.extend([0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.6].iter().flat_map(|c| c.to_le_bytes()));
        let json = r#"{"asset": {"version": "2.0"}, "scenes": [{"nodes": [0, 1]}],
            "nodes": [{"name": "seat", "mesh": 0}, {"name": "mattress", "mesh": 1}],
            "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 2}]}, {"primitives": [{"attributes": {"POSITION": 1}, "indices": 2}]}],
            "bufferViews": [{"byteOffset": 0, "byteLength": 48}, {"byteOffset": 48, "byteLength": 48}, {"byteOffset": 96, "byteLength": 12},
                {"byteOffset": 108, "byteLength": 8}, {"byteOffset": 116, "byteLength": 24}],
            "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 4, "min": [-0.5, 0.41, -0.5], "max": [0.1, 0.41, 0.5]},
                {"bufferView": 1, "componentType": 5126, "type": "VEC3", "count": 4, "min": [-0.5, 0.4, -0.5], "max": [0.5, 0.4, 0.5]},
                {"bufferView": 2, "componentType": 5123, "type": "SCALAR", "count": 6},
                {"bufferView": 3, "componentType": 5126, "type": "SCALAR", "count": 2}, {"bufferView": 4, "componentType": 5126, "type": "VEC3", "count": 2}],
            "animations": [{"name": "bed_out", "channels": [{"sampler": 0, "target": {"node": 1, "path": "translation"}}], "samplers": [{"input": 3, "output": 4}]}]}"#;
        parse_glb(&glb::tests::glb(json, &bin)).unwrap()
    }

    #[test]
    fn the_sampler_reads_game_axes_and_the_open_pose() {
        let g = sofa_bed();
        let rest = Sampler::new(&g, &J::Obj(Vec::new()), None).unwrap().unwrap();
        // glTF x -> game -x: the seat reaches game x 0.5, not -0.5.
        assert!((rest.top(0.3, 0.0, 0.45).unwrap() - 0.41).abs() < 1e-6);
        assert_eq!(rest.top(-0.3, 0.0, 0.41).map(|h| (h * 100.0).round()), Some(40.0), "only the mattress under it there");
        assert_eq!(rest.top(0.0, 0.8, 0.4), None, "nothing in front at rest");
        let open = Sampler::new(&g, &J::Obj(Vec::new()), Some(&open_pose(&g, &J::Str("bed_out".into())).unwrap())).unwrap().unwrap();
        assert!((open.top(0.0, 0.8, 0.44).unwrap() - 0.40).abs() < 1e-6, "pulled out");
        // Scaled: the game's model is the file's times `scale`.
        let big = Sampler::new(&g, &json::loads_text(br#"{"scale": 2}"#).unwrap(), None).unwrap().unwrap();
        assert!((big.top(0.6, 0.0, 0.85).unwrap() - 0.82).abs() < 1e-6);
    }

    #[test]
    fn heights_warn_off_the_model_and_when_their_sources_disagree() {
        let g = sofa_bed();
        let furniture = Furniture {
            seats: json::loads_text(br#"{"sofaBed": {"h": 0.44, "fwd": 0}}"#).unwrap(),
            default_seat: json::loads_text(br#"{"h": 0.45, "fwd": 0}"#).unwrap(),
            beds: json::loads_text(br#"{"sofaBed": {"h": 0.4, "fwd": 0.6}}"#).unwrap(),
            default_bed: json::loads_text(br#"{"h": 0.5}"#).unwrap(),
            bunks: J::Obj(Vec::new()),
        };
        let object = json::loads_text(br#"{"id": "sofaBed", "footprint": [1, 1], "seat": {"height": 0.41, "forward": 0.08}, "interactions": [{"pose": "lie"}]}"#).unwrap();
        let entry = json::loads_text(br#"{"motions": {"open": "bed_out"}}"#).unwrap();
        let found = heights(&g, &entry, &object, &furniture, true).unwrap();
        let warnings: Vec<String> = found.iter().flat_map(Height::warnings).collect();
        assert_eq!(warnings, [
            "seat 0.44 m (furniture.json `seats`), the model 0.410 m there (x 0.00, depth 0.00): 3.0 cm lower",
            "seat: content `seat` says 0.41 m, 0.08 forward; furniture.json `seats` says 0.44 m, 0.000 forward (furniture.json wins: characters.ts `seatOf`)",
        ]);
        let mattress = found.iter().find(|h| h.what == "mattress").unwrap();
        assert_eq!((mattress.open.as_deref(), mattress.at), (Some("bed_out"), [0.0, 0.6]));
        assert!(mattress.fits());
        // Not held open (no `motions.open`): nothing under the sleepers in front.
        let shut = heights(&g, &J::Obj(Vec::new()), &object, &furniture, true).unwrap();
        let shut: Vec<String> = shut.iter().filter(|h| h.what == "mattress").flat_map(Height::warnings).collect();
        assert_eq!(shut, ["mattress 0.40 m (furniture.json `beds`): no model under it (x 0.00, depth 0.60)"]);
    }

    /// A model of flat quads facing up, a node each: `[height, x from, x to, depth from, depth to]` (glTF axes: game
    /// x is the file's -x).
    #[test]
    fn a_seat_front_is_where_the_seat_ends_in_front() {
        // A deep sofa's seat at 0.44 m from 0.4 m back to 0.38 m forward, its back at 0.8 m behind it, and a
        // cushion's rounded front lip 3 cm lower up to 0.4 m: the seat ends at 0.4 m.
        let sofa = quads(&[[0.44, -0.4, 0.4, -0.4, 0.38], [0.41, -0.4, 0.4, 0.38, 0.4], [0.8, -0.4, 0.4, -0.5, -0.4], [0.1, -0.4, 0.4, 0.4, 0.45]]);
        let s = Sampler::new(&sofa, &J::Obj(Vec::new()), None).unwrap().unwrap();
        assert_eq!(s.front(0.0, -0.09, 0.44).map(|f| (f * 100.0).round()), Some(40.0));
        assert_eq!(s.front(0.0, 0.6, 0.44), None, "nothing under hips in front of it");
        let furniture = |front: f64| Furniture {
            seats: json::loads_text(format!(r#"{{"sofaThree": {{"h": 0.44, "fwd": -0.09, "front": {front}}}}}"#).as_bytes()).unwrap(),
            default_seat: json::loads_text(br#"{"h": 0.45, "fwd": 0}"#).unwrap(),
            beds: J::Obj(Vec::new()),
            default_bed: json::loads_text(br#"{"h": 0.5}"#).unwrap(),
            bunks: J::Obj(Vec::new()),
        };
        let object = json::loads_text(br#"{"id": "sofaThree", "footprint": [1, 1], "interactions": [{"pose": "sit"}]}"#).unwrap();
        let warn = |front: f64| -> Vec<String> { heights(&sofa, &J::Obj(Vec::new()), &object, &furniture(front), true).unwrap().iter().flat_map(Height::warnings).collect() };
        assert!(warn(0.41).is_empty(), "{:?}", warn(0.41));
        assert_eq!(warn(0.3), ["seat front 0.30 m forward (furniture.json `seats`), the model's seat ends 0.40 m forward there (x 0.00, depth -0.09): 10.0 cm further forward"]);
    }

    fn quads(quads: &[[f32; 5]]) -> Glb {
        let mut bin = Vec::new();
        for [y, x0, x1, z0, z1] in quads {
            for v in [[*x0, *y, *z0], [*x0, *y, *z1], [*x1, *y, *z1], [*x1, *y, *z0]] {
                bin.extend(v.iter().flat_map(|c: &f32| c.to_le_bytes()));
            }
        }
        let at = bin.len();
        bin.extend([0u16, 1, 2, 0, 2, 3].iter().flat_map(|i| i.to_le_bytes()));
        let n = quads.len();
        let nodes: Vec<String> = (0..n).map(|k| format!(r#"{{"name": "q{k}", "mesh": {k}}}"#)).collect();
        let meshes: Vec<String> = (0..n).map(|k| format!(r#"{{"primitives": [{{"attributes": {{"POSITION": {k}}}, "indices": {n}}}]}}"#)).collect();
        let mut views: Vec<String> = (0..n).map(|k| format!(r#"{{"byteOffset": {}, "byteLength": 48}}"#, k * 48)).collect();
        views.push(format!(r#"{{"byteOffset": {at}, "byteLength": 12}}"#));
        let mut accessors: Vec<String> = quads
            .iter()
            .enumerate()
            .map(|(k, [y, x0, x1, z0, z1])| format!(r#"{{"bufferView": {k}, "componentType": 5126, "type": "VEC3", "count": 4, "min": [{x0}, {y}, {z0}], "max": [{x1}, {y}, {z1}]}}"#))
            .collect();
        accessors.push(format!(r#"{{"bufferView": {n}, "componentType": 5123, "type": "SCALAR", "count": 6}}"#));
        let json = format!(
            r#"{{"asset": {{"version": "2.0"}}, "scenes": [{{"nodes": [{}]}}], "nodes": [{}], "meshes": [{}], "bufferViews": [{}], "accessors": [{}]}}"#,
            (0..n).map(|k| k.to_string()).collect::<Vec<_>>().join(", "),
            nodes.join(", "),
            meshes.join(", "),
            views.join(", "),
            accessors.join(", ")
        );
        parse_glb(&glb::tests::glb(&json, &bin)).unwrap()
    }

    #[test]
    fn a_top_the_content_hasnt_a_table_without_its_chairs_and_a_stairs_steps() {
        // A table: its top 0.74 m up (1.2 × 0.8), a shelf under it, a vase on it; a chair behind it, its seat 0.46
        // m up and its back's top rail 1.0 m.
        let table = quads(&[[0.74, -0.6, 0.6, -0.4, 0.4], [0.3, -0.6, 0.6, -0.4, 0.4], [0.95, -0.05, 0.05, -0.05, 0.05], [0.46, -0.25, 0.25, -0.7, -0.3], [1.0, -0.25, 0.25, -0.72, -0.68]]);
        let s = Sampler::new(&table, &J::Obj(Vec::new()), None).unwrap().unwrap();
        // The vase on the middle line and the shelf under every one don't move it.
        let (at, top) = top_near(&s, [0.72, 0.76], &[]).unwrap();
        assert!((top - 0.74).abs() < 1e-6, "{at:?} {top}");
        // Out of the range, it is still the top (the review calls it a top if near enough): the vase, in the
        // range, stands on what the other lines found.
        assert!((top_near(&s, [0.85, 0.95], &[]).unwrap().1 - 0.74).abs() < 1e-6);
        // A sink: its worktop 0.9 m up at the back, the basin's floor 0.75 m in front (on six lines of nine).
        let sink = quads(&[[0.9, -0.5, 0.5, -0.5, -0.2], [0.75, -0.5, 0.5, -0.2, 0.5]]);
        let (at, top) = top_near(&Sampler::new(&sink, &J::Obj(Vec::new()), None).unwrap().unwrap(), [0.76, 0.91], &[]).unwrap();
        assert!((top - 0.9).abs() < 1e-6 && at[1] < -0.2, "{at:?} {top}");
        // The lines: its middle (0, -0.16) and a quarter of its 1.2 × 1.12 m round it; those within 0.3 m of a
        // seat's hips are left out (over the chair's seat, and the middle), unless the others find nothing.
        let cm = |lines: Vec<[f64; 2]>| -> Vec<[i64; 2]> { lines.iter().map(|p| p.map(|v| (v * 100.0).round() as i64)).collect() };
        let all = top_lines(&s, &[]);
        assert_eq!(cm(all), [[-30, -44], [-30, -16], [-30, 12], [0, -44], [0, -16], [0, 12], [30, -44], [30, -16], [30, 12]]);
        assert_eq!(cm(top_lines(&s, &[[0.0, -0.4]])), [[-30, -44], [-30, -16], [-30, 12], [0, 12], [30, -44], [30, -16], [30, 12]]);
        let seated = [[0.3, -0.1], [-0.05, -0.44], [-0.3, -0.16], [0.0, 0.0]];
        assert_eq!(cm(top_lines(&s, &seated)), [[30, -44]], "the one line left, off the table");
        assert!((top_near(&s, [0.72, 0.76], &seated).unwrap().1 - 0.74).abs() < 1e-6, "so all the lines");
        // Without the chair's back: the vase, over the top, counts.
        assert!((s.height_by(0.74).unwrap() - 0.95).abs() < 1e-6);
        assert_eq!(s.height_by(0.5), None, "nothing flat there");

        // A stair of three treads, a rail at its side.
        let stair = quads(&[[0.2, -0.4, 0.4, 0.2, 0.6], [0.4, -0.4, 0.4, -0.2, 0.2], [0.6, -0.4, 0.4, -0.6, -0.2], [1.5, 0.45, 0.5, -0.6, 0.6]]);
        let s = Sampler::new(&stair, &J::Obj(Vec::new()), None).unwrap().unwrap();
        let (top, n) = s.steps().unwrap();
        assert_eq!(((top * 100.0).round(), n), (60.0, 3));
    }

    #[test]
    fn the_review_takes_a_cribs_mattress_at_slot_0() {
        // A crib's mattress 0.5 m up at slot 0 (game x -1 to 0: the file's 0 to 1), its changing pad 0.8 m at slot 1.
        let g = quads(&[[0.5, 0.0, 1.0, -0.5, 0.5], [0.8, -1.0, 0.0, -0.5, 0.5]]);
        let furniture = Furniture {
            seats: J::Obj(Vec::new()),
            default_seat: json::loads_text(br#"{"h": 0.45, "fwd": 0}"#).unwrap(),
            beds: json::loads_text(br#"{"crib": {"h": 0.5}}"#).unwrap(),
            default_bed: json::loads_text(br#"{"h": 0.5}"#).unwrap(),
            bunks: J::Obj(Vec::new()),
        };
        let object = json::loads_text(br#"{"id": "crib", "footprint": [2, 1], "slots": 2, "interactions": [{"pose": "lie"}]}"#).unwrap();
        let mattress = |every: bool| heights(&g, &J::Obj(Vec::new()), &object, &furniture, every).unwrap().into_iter().find(|h| h.what == "mattress").map(|h| (h.at, h.mesh)).unwrap();
        assert_eq!(mattress(false), ([-0.5, 0.0], Some(0.5)));
        let (at, mesh) = mattress(true);
        assert_eq!(at, [0.5, 0.0], "the gate takes the worst slot");
        assert!((mesh.unwrap() - 0.8).abs() < 1e-6);
    }

    #[test]
    fn motions_played_in_order_and_checked() {
        let m = json::loads_text(br#"{"uses": {"fetch": ["door_open", {"clip": "door_open", "hold": -1, "slot": true, "x": 1}], "b": "lid"},
            "loops": {"relax": "rock"}, "idle": "rock", "clock": {"hour": "h", "at": "noon"}}"#).unwrap();
        let p: Vec<(String, String)> = played(&m).unwrap().into_iter().map(|(c, by)| (c.key(), by)).collect();
        assert_eq!(p, [("door_open", "use 'fetch'"), ("lid", "use 'b'"), ("rock", "loop 'relax'"), ("h", "clock hour")].map(|(a, b)| (a.to_string(), b.to_string())));
        assert_eq!(motions_errors(&m), [
            "`motions.uses` 'fetch': hold -1 is no number of seconds",
            "`motions.uses` 'fetch': slot True is no slot (0, 1, ...)",
            "`motions.uses` 'fetch': unknown x",
            "`motions.clock.at` 'noon' is no time (minutes past twelve)",
        ]);
        assert_eq!(loops(&m).unwrap(), HashSet::from(["rock".to_string(), "h".to_string()]));
    }

    fn j(text: &str) -> J {
        json::loads_text(text.as_bytes()).unwrap()
    }

    #[test]
    fn a_birds_fields_are_its_sizes() {
        assert!(bird_fields(&j(r#"{"length": 0.12, "wingspan": 0.185, "perchHeight": 0.03, "pitch": 34}"#)).is_empty());
        assert!(bird_fields(&j(r#"{"length": 0.12, "wingspan": 0.185}"#)).is_empty(), "the rest is optional");
        assert_eq!(bird_fields(&j(r#"{"length": 0, "span": 1, "pitch": 95, "perchHeight": true}"#)), [
            "`bird`: unknown span (it takes length, wingspan, perchHeight, pitch)",
            "`bird.length` 0 is no length (metres, over 0)",
            "`bird` has no `wingspan`",
            "`bird.perchHeight` True is no length (metres, over 0)",
            "`bird.pitch` 95 is no angle (degrees, -90 to 90)",
        ]);
        assert_eq!(bird_fields(&j("[0.12]")), ["`bird` is {\"length\", \"wingspan\", ...} (metres), not [0.12]"]);
    }

    /// The blue tit as built (families/birds.py) against its `bird`, and against wrong ones.
    #[test]
    fn a_bird_is_checked_against_its_file() {
        let path = join(&repo_root(), "web/public/assets/models/birds/blueTit.glb");
        let g = parse_glb(&std::fs::read(&path).unwrap()).unwrap();
        let parts = bird_parts(&g).unwrap();
        assert_eq!(parts.names.iter().map(String::as_str).collect::<Vec<_>>(), ["body", "head", "tail", "wing.L", "wing.R"]);
        assert!(parts.flat.is_empty());
        let dims = [0.185, 0.072, 0.106];
        assert!(bird_errors(&j(r#"{"length": 0.12, "wingspan": 0.185, "perchHeight": 0.03, "pitch": 34}"#), &parts, dims).is_empty());
        let wrong = bird_errors(&j(r#"{"length": 0.2, "wingspan": 0.3, "perchHeight": 0.5}"#), &parts, dims);
        assert_eq!(wrong.len(), 3, "{wrong:?}");
        assert!(wrong[0].starts_with("0.185 m across its spread wings, its `bird.wingspan` says 0.300 m"), "{}", wrong[0]);
        assert!(wrong[1].starts_with("0.12") && wrong[1].contains("from bill to tail tip, its `bird.length` says 0.200 m"), "{}", wrong[1]);
        assert_eq!(wrong[2], "`bird.perchHeight` 0.500 m is over the bird's height, 0.072 m");
        // Its wings swapped over, one part gone and one saved flat.
        let mut points = parts.points;
        let left = points.remove("wing.L").unwrap();
        let right = points.insert("wing.R".into(), left).unwrap();
        points.insert("wing.L".into(), right);
        let mut names = parts.names;
        names.remove("tail");
        let broken = BirdParts { names, points, flat: vec!["wing.L".into()] };
        let e = bird_errors(&j(r#"{"length": 0.12, "wingspan": 0.185}"#), &broken, dims);
        assert_eq!(e[0], "a bird's parts are nodes named body, head, tail, wing.L, wing.R: it has no tail");
        assert!(e[1].starts_with("wing.L is saved at scale 0"), "{}", e[1]);
        assert_eq!(&e[2..], [
            "wing.L reaches out the wrong way: a bird's wing.L is spread to its left, game -x (glTF +x)",
            "wing.R reaches out the wrong way: a bird's wing.R is spread to its right, game +x (glTF -x)",
        ]);
    }

    #[test]
    fn perches_are_points_inside_the_model() {
        let (lo, hi) = ([-1.0, 0.0, -1.0], [1.0, 4.0, 1.0]);
        assert!(perch_errors(&j("[[0.5, 3.9, -0.5], [-1.005, 0, 1]]"), lo, hi, 1.0).is_empty(), "a centimetre out is in");
        assert_eq!(perch_errors(&j("[[0, 5, 0], [0.5, 1], [1, 1, \"x\"], [0, 2, 0], [3, 1, 0]]"), lo, hi, 1.0), [
            "`perches` 1: [0.5, 1] is no [x, y, z]",
            "`perches` 2: [1, 1, 'x'] is no [x, y, z]",
            "2 of 5 perches outside the model (the first, 0: [0, 5, 0]; its box x, y, z: -1.00 to 1.00, 0.00 to 4.00, -1.00 to 1.00)",
        ]);
        // Before the entry's `scale`: a tree drawn twice its size has its box twice as big, its perches as written.
        assert!(perch_errors(&j("[[0, 3, 0]]"), [-2.0, 0.0, -2.0], [2.0, 8.0, 2.0], 2.0).is_empty());
        assert_eq!(perch_errors(&j("[[0, 5, 0]]"), [-2.0, 0.0, -2.0], [2.0, 8.0, 2.0], 2.0).len(), 1);
        assert_eq!(perch_errors(&j("[]"), lo, hi, 1.0), ["`perches` is empty: leave it out"]);
        assert_eq!(perch_errors(&j("{\"a\": 1}"), lo, hi, 1.0), ["`perches` is a list of [x, y, z] (metres), not {'a': 1}"]);
    }

    #[test]
    fn a_habitat_is_one_of_four_and_goes_with_perches() {
        for h in HABITATS {
            assert!(habitat_errors(&j(&format!(r#"{{"perches": [[0, 1, 0]], "habitat": "{h}"}}"#))).is_empty(), "{h}");
        }
        assert!(habitat_errors(&j(r#"{"perches": [[0, 1, 0]]}"#)).is_empty(), "optional");
        assert!(habitat_errors(&j(r#"{"url": "a.glb"}"#)).is_empty());
        assert_eq!(habitat_errors(&j(r#"{"perches": [[0, 1, 0]], "habitat": "shrub"}"#)), ["`habitat` 'shrub' is none of tree, conifer, bush, hedge"]);
        assert_eq!(habitat_errors(&j(r#"{"perches": [[0, 1, 0]], "habitat": ["tree"]}"#)), ["`habitat` ['tree'] is none of tree, conifer, bush, hedge"]);
        assert_eq!(habitat_errors(&j(r#"{"habitat": "Tree"}"#)), [
            "`habitat` 'Tree' is none of tree, conifer, bush, hedge",
            "`habitat` without `perches`: it says what the perches are on",
        ]);
    }
}
