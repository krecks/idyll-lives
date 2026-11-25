//! Paths, manifests and the content that names what a model is for.

use super::*;

/// Where the gate reads: the repo's assets, content and tools/art/centres.json.
pub struct Paths {
    /// tools/art: the models' tools (centres.json, the gate's baseline, previews/)
    pub art: String,
    /// web/public/assets
    pub assets: String,
    /// web/public/content
    pub content: String,
    /// web/src/render/shared/characters/furniture.json: where residents sit and lie on each object (`SEATS`,
    /// `BEDS`, `BUNKS` in characters.ts), for the height checks.
    pub furniture: String,
    /// tools/art/centres.json: where each model `cargo art` builds is centred at rest (its Model's `centre`,
    /// (0, 0) for most): {key: [x, depth]}, for this gate only (the game doesn't read it). A build writes its
    /// models', `cargo art --centres` every family's. A key that isn't in it isn't built here (a third-party
    /// file): its centre isn't checked.
    pub centres: String,
    /// tools/art/gate-baseline.json: the counted checks' findings each model may keep (`baseline`).
    pub baseline: String,
}

impl Paths {
    /// The paths under a checkout's root.
    pub fn at(root: &str) -> Paths {
        let root = normpath(root);
        Paths {
            art: normpath(&join(&root, "tools/art")),
            assets: normpath(&join(&root, "web/public/assets")),
            content: normpath(&join(&root, "web/public/content")),
            furniture: normpath(&join(
                &root,
                "web/src/render/shared/characters/furniture.json",
            )),
            centres: normpath(&join(&root, "tools/art/centres.json")),
            baseline: normpath(&join(&root, baseline::FILE)),
        }
    }

    /// This checkout's.
    pub fn repo() -> Paths {
        Paths::at(&repo_root())
    }
}

/// The checkout this crate is in.
pub fn repo_root() -> String {
    crate::repo_root().to_string_lossy().into_owned()
}

pub(crate) fn read_json(path: &str) -> Result<J, String> {
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

/// centres.json's text for `centres` ({key: (x, depth)}): one key a line, sorted, each to 0.1 mm
/// (`round(v, 4) + 0.0`: no -0.0).
pub fn centres_text(centres: &BTreeMap<String, [f64; 2]>) -> String {
    let round4 = |v: f64| format!("{v:.4}").parse::<f64>().unwrap() + 0.0;
    let lines: Vec<String> = centres
        .iter()
        .map(|(key, c)| {
            format!(
                "  {}: [{}, {}]",
                py::json_str(key),
                py::float_repr(round4(c[0])),
                py::float_repr(round4(c[1]))
            )
        })
        .collect();
    format!("{{\n{}\n}}\n", lines.join(",\n"))
}

/// [(path, manifest)] for a manifest and the packs it lists, depth first, in order. A manifest that can't
/// be read, or packs nested more than 20 deep (a pack listing itself), is an error naming it.
pub fn manifests(paths: &Paths, path: &str) -> Result<Vec<(String, J)>, String> {
    fn stack(
        paths: &Paths,
        path: &str,
        depth: usize,
        out: &mut Vec<(String, J)>,
    ) -> Result<(), String> {
        let name = || relpath(&normpath(path), &dirname(&paths.assets));
        if depth > 20 {
            return Err(format!("{}: packs nested too deep", name()));
        }
        let m = read_json(path).map_err(|e| format!("{}: {e}", name()))?;
        let packs: Vec<J> = m
            .get("packs")
            .and_then(J::list)
            .map(|l| l.to_vec())
            .unwrap_or_default();
        out.push((path.to_string(), m));
        for pack in packs {
            let pack = pack.str().ok_or_else(|| {
                format!(
                    "expected str, bytes or os.PathLike object, not {}",
                    glb::type_name(&pack)
                )
            })?;
            stack(
                paths,
                &normpath(&join(&dirname(path), pack)),
                depth + 1,
                out,
            )?;
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
        let data = read_json(&join(&paths.content, name))
            .map_err(|e| format!("web/public/content/{name}: {e}"))?;
        match data {
            J::Obj(_) => Ok(data),
            _ => Err(format!(
                "web/public/content/{name}: a content file is a JSON object"
            )),
        }
    };
    let base = read("base.json")?;
    let mut files = vec![base.clone()];
    for name in base.get("include").and_then(J::list).unwrap_or(&[]) {
        files.push(read(
            name.str()
                .ok_or("expected str, bytes or os.PathLike object")?,
        )?);
    }
    let all = |k: &str| -> Vec<J> {
        files
            .iter()
            .flat_map(|f| {
                f.get(k)
                    .filter(|v| v.truthy())
                    .and_then(J::list)
                    .unwrap_or(&[])
                    .iter()
                    .cloned()
            })
            .collect()
    };
    Ok(Content {
        objects: all("objects"),
        items: all("items"),
        categories: all("buyCategories"),
    })
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
pub(super) fn content_keys(defs: &Content) -> R<BTreeMap<String, String>> {
    let mut keys = BTreeMap::new();
    for (kind, prefix, found) in [
        ("object", "model", &defs.objects),
        ("item", "item", &defs.items),
    ] {
        for d in found {
            let what = format!("{kind} {}", str_of(key(d, "id")?));
            keys.entry(model_key(d, prefix)?).or_insert(what);
        }
    }
    Ok(keys)
}

/// The model keys the manifests and the content know: every content object's and item's model, and every
/// model entry of the manifests (a build may also write its own pack's: `art::build_keys`).
pub fn known_keys(paths: &Paths) -> Result<BTreeSet<String>, String> {
    let mut known: BTreeSet<String> = content_keys(&content(paths)?)?
        .into_keys()
        .filter(|k| !k.is_empty())
        .collect();
    for (_path, m) in asset_manifests(paths)? {
        for (k, e) in m
            .get("entries")
            .filter(|e| e.truthy())
            .and_then(J::entries)
            .unwrap_or(&[])
        {
            if e.get("type") == Some(&J::Str("model".into())) {
                known.insert(k.clone());
            }
        }
    }
    Ok(known)
}

/// The facts of a content object that draws a model, for the checks against what the game does with it.
pub(super) struct Facts {
    pub(super) id: J,
    pub(super) footprint: J,
    pub(super) layer: J,
    pub(super) outside: bool,
    pub(super) stairs: bool,
    /// The disc a thing on a top takes (content.ts `radius`: default 0.12, kept within 0.02-0.45).
    pub(super) radius: f64,
    /// The content object itself, for its functional heights (`seat`, `surface`, `light`, its uses' poses: `heights`).
    pub(super) object: J,
}

/// {model key: [facts of each content object that draws it]}: where the wall's face is (a footprint's
/// back, `WALL` in), the footprint, the disc a thing on a top takes.
pub(super) fn content_objects(defs: &Content) -> R<HashMap<String, Vec<Facts>>> {
    let mut out: HashMap<String, Vec<Facts>> = HashMap::new();
    let none = J::Obj(Vec::new());
    for o in &defs.objects {
        let category = o.get("category").unwrap_or(&J::Null);
        // {c.get('id'): c}: the last with that id.
        let c = defs
            .categories
            .iter()
            .rev()
            .find(|c| c.get("id").unwrap_or(&J::Null) == category)
            .unwrap_or(&none);
        let group_id = o.get("group").unwrap_or(&J::Null);
        let groups = c
            .get("groups")
            .filter(|g| g.truthy())
            .and_then(J::list)
            .unwrap_or(&[]);
        let group = groups
            .iter()
            .find(|g| g.get("id").unwrap_or(&J::Null) == group_id)
            .unwrap_or(&none);
        let outside = group
            .get("outside")
            .or(c.get("outside"))
            .map(J::truthy)
            .unwrap_or(false);
        let r = match o.get("radius") {
            None => 0.12,
            Some(r) => r
                .num()
                .ok_or_else(|| {
                    format!(
                        "'<' not supported between instances of '{}' and 'float'",
                        glb::type_name(r)
                    )
                })?
                .f(),
        };
        let radius = {
            let m = if r < 0.45 { r } else { 0.45 };
            if m > 0.02 { m } else { 0.02 }
        };
        out.entry(model_key(o, "model")?).or_default().push(Facts {
            id: key(o, "id")?.clone(),
            footprint: o
                .get("footprint")
                .cloned()
                .unwrap_or(J::Arr(vec![J::Int(1), J::Int(1)])),
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
pub(super) struct Finishes {
    pub(super) plain: HashMap<String, bool>,
    pub(super) see_through: HashSet<String>,
    /// The textured ones: {key: (its `uvScale`, its texture's file)}.
    pub(super) textures: HashMap<String, (f64, String)>,
}

impl Finishes {
    /// Whether `finish` is a textured one (wood, fabric, bark, foliage): neither plain nor see-through.
    pub(super) fn textured(&self, finish: &J) -> bool {
        finish
            .str()
            .is_some_and(|f| self.plain.get(f) == Some(&false) && !self.see_through.contains(f))
    }

    pub(super) fn see_through(&self, finish: Option<&J>) -> bool {
        finish
            .and_then(J::str)
            .is_some_and(|f| self.see_through.contains(f))
    }
}
