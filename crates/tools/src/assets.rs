//! `cargo assets`: compressed copies of the game's assets for the production build (was
//! `tools/assets/optimize.mjs`).
//!
//! The sources in `web/public/assets` stay as they are, and are still what `bun run dev` serves;
//! `vite build` runs this and ships the copies next to them, with an index the asset registry reads
//! (`assets/optimized.json`: source path -> compressed copy, and each source's content hash). Anything
//! not in the index, such as a modder's pack, loads as it is.
//!
//! - textures (jpg, png) -> KTX2, Basis Universal ETC1S with mipmaps (`texture.rs`);
//! - models (`.glb`) -> quantised geometry and meshopt compression (`model.rs`);
//! - character binaries -> gzip (the browser inflates them; most hosts don't compress `.bin`).
//!
//! Copies are kept by content hash in `web/.assets/cache`, so only changed files are encoded again;
//! `web/.assets/out` (`optimized.json` and the `opt/` tree) is what the build copies into `dist/assets`.
//!
//! - `cargo assets [--force] [--jobs N] [--verbose]`: builds (or reuses) every copy and writes the output.
//! - `cargo assets check [--against <dir>]`: decodes every copy in the output and compares it with its
//!   source (a model's triangles within the quantisation's steps, its shading and its animations exact;
//!   a texture's levels by their error and their mean), and sums the sizes. `--against` does the same
//!   for another output tree (the JavaScript tool's, kept from before) and sets the two side by side.

pub mod model;
pub mod texture;

use crate::download;
use serde_json::{Map, Value as J, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use texture::Role;

pub const USAGE: &str = "usage: cargo assets [--force] [--jobs N] [--verbose]    compressed copies into web/.assets/out
       cargo assets check [--against <dir>]              decodes them and compares them with their sources";

/// Bump a kind's when its encoding changes, so its cached copies are made again (and the game's own
/// caches of what it drew from them: the index carries the hashes).
const VERSION: [(&str, u32); 3] = [("model", 4), ("texture", 2), ("gzip", 1)];
/// What encodes: in every cached copy's key, so another tool's copies in the same folder are never taken.
const ENCODER: &str = "cargo assets (basisu_c_sys 0.9, meshoptimizer 1.3)";

/// Left as they are: not loaded by the game, or drawn in a way KTX2 can't be (flipped on upload).
const SKIP: [&str; 1] = ["textures/sky/clouds.png"];
/// The finishes of leaves: the tiling foliage, and the trees' leaf cards.
const FOLIAGE_FINISHES: [&str; 2] = ["material.finish.foliage", "material.finish.leafcards"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Model,
    Texture(Role),
    Gzip,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Model => "model",
            Kind::Texture(_) => "texture",
            Kind::Gzip => "gzip",
        }
    }

    fn version(self) -> u32 {
        VERSION
            .iter()
            .find(|(name, _)| *name == self.name())
            .map_or(0, |(_, v)| *v)
    }
}

fn extension(rel: &str) -> &str {
    rel.rsplit('/')
        .next()
        .and_then(|file| file.rsplit_once('.'))
        .map_or("", |(_, ext)| ext)
}

/// What a source file becomes, and where (`None`: shipped as it is).
fn plan(rel: &str) -> Option<(Kind, String)> {
    if rel.starts_with("icons/") || SKIP.contains(&rel) {
        return None;
    }
    let ext = extension(rel).to_ascii_lowercase();
    match ext.as_str() {
        "glb" => Some((Kind::Model, rel.to_string())),
        "jpg" | "jpeg" | "png" => Some((
            Kind::Texture(Role::of(rel)),
            format!("{}.ktx2", &rel[..rel.len() - ext.len() - 1]),
        )),
        "bin" if rel.starts_with("characters/") => Some((Kind::Gzip, format!("{rel}.gz"))),
        _ => None,
    }
}

struct Dirs {
    src: PathBuf,
    cache: PathBuf,
    out: PathBuf,
}

fn dirs() -> Dirs {
    let web = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web");
    Dirs {
        src: web.join("public/assets"),
        cache: web.join(".assets/cache"),
        out: web.join(".assets/out"),
    }
}

/// Every file under `dir`, as paths from it with `/` between their parts, in order.
fn walk(dir: &Path) -> Result<Vec<String>, String> {
    fn into(dir: &Path, prefix: &str, out: &mut Vec<String>) -> Result<(), String> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(Result::ok)
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            let rel = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            if entry.path().is_dir() {
                into(&entry.path(), &rel, out)?;
            } else {
                out.push(rel);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    into(dir, "", &mut out)?;
    Ok(out)
}

/// `a/b/../c` and `a/./c` as `a/c`.
fn normalize(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

/// The models drawn with foliage: an entry of a manifest (the base one, the packs') mapping one of its
/// file's materials to a finish of leaves, its far version too. Their geometry keeps its floats.
fn foliage_models(src: &Path, files: &[String]) -> BTreeSet<String> {
    let mut models = BTreeSet::new();
    for rel in files
        .iter()
        .filter(|f| f.rsplit('/').next() == Some("manifest.json"))
    {
        let Ok(bytes) = std::fs::read(src.join(rel)) else {
            continue;
        };
        let Ok(manifest) = serde_json::from_slice::<J>(&bytes) else {
            continue;
        };
        let Some(entries) = manifest.get("entries").and_then(J::as_object) else {
            continue;
        };
        let dir = rel.rsplit_once('/').map_or("", |(dir, _)| dir);
        for entry in entries.values() {
            let leaves = entry.get("type").and_then(J::as_str) == Some("model")
                && entry
                    .get("materials")
                    .and_then(J::as_object)
                    .is_some_and(|m| {
                        m.values().any(|v| {
                            v.get("finish")
                                .and_then(J::as_str)
                                .is_some_and(|f| FOLIAGE_FINISHES.contains(&f))
                        })
                    });
            if !leaves {
                continue;
            }
            for key in ["url", "far"] {
                if let Some(url) = entry.get(key).and_then(J::as_str) {
                    models.insert(normalize(&format!("{dir}/{url}")));
                }
            }
        }
    }
    models
}

struct Task {
    rel: String,
    kind: Kind,
    out: String,
    size: usize,
    foliage: bool,
    cached: PathBuf,
    key: String,
}

/// The models the tool makes copies of: each one's file, its path among the assets, and whether it is drawn
/// with foliage (its geometry then keeps its floats).
pub fn models() -> Result<Vec<(PathBuf, String, bool)>, String> {
    let d = dirs();
    Ok(tasks(&d)?
        .into_iter()
        .filter(|t| t.kind == Kind::Model)
        .map(|t| (d.src.join(&t.rel), t.rel, t.foliage))
        .collect())
}

fn kb(n: usize) -> String {
    format!("{} KB", (n as f64 / 1024.0).round() as usize)
}

fn tasks(d: &Dirs) -> Result<Vec<Task>, String> {
    let files = walk(&d.src)?;
    let foliage = foliage_models(&d.src, &files);
    let mut tasks = Vec::new();
    for rel in &files {
        let Some((kind, out)) = plan(rel) else {
            continue;
        };
        let bytes = std::fs::read(d.src.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
        let leaves = foliage.contains(rel);
        let mut hash = Sha256::new();
        hash.update(ENCODER);
        hash.update(kind.name());
        hash.update(kind.version().to_le_bytes());
        if let Kind::Texture(role) = kind {
            hash.update(role.name());
            hash.update(texture::settings(role));
        }
        hash.update(if leaves { "foliage" } else { "" });
        hash.update(&bytes);
        let key = format!("{:x}", hash.finalize())[..24].to_string();
        tasks.push(Task {
            cached: d.cache.join(format!("{key}.{}", extension(&out))),
            rel: rel.clone(),
            kind,
            out,
            size: bytes.len(),
            foliage: leaves,
            key,
        });
    }
    Ok(tasks)
}

fn build(force: bool, jobs: usize, verbose: bool) -> Result<(), String> {
    let started = std::time::Instant::now();
    let d = dirs();
    let tasks = tasks(&d)?;
    std::fs::create_dir_all(&d.cache).map_err(|e| format!("{}: {e}", d.cache.display()))?;
    let todo: Vec<&Task> = tasks
        .iter()
        .filter(|t| force || !t.cached.exists())
        .collect();
    if !todo.is_empty() {
        println!("[assets] encoding {} of {} files…", todo.len(), tasks.len());
    }
    let said = Mutex::new(());
    let say = |t: &Task, size: usize| {
        if verbose {
            let _one = said.lock();
            println!("[assets]   {}: {} -> {}", t.rel, kb(t.size), kb(size));
        }
    };
    // Why a model was left as it is (the game loads its source).
    let left: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
    let encode = |t: &Task| -> Result<(), String> {
        let bytes = std::fs::read(d.src.join(&t.rel)).map_err(|e| format!("{}: {e}", t.rel))?;
        let made = match t.kind {
            Kind::Gzip => download::gzip9(&bytes)?,
            Kind::Texture(role) => {
                texture::encode(&bytes, role).map_err(|e| format!("{}: {e}", t.rel))?
            }
            Kind::Model => match model::optimize(&bytes, t.foliage) {
                Ok(made) => made,
                Err(why) => {
                    left.lock().expect("no panics").push((t.rel.clone(), why));
                    return Ok(());
                }
            },
        };
        download::write(&t.cached, &made)?;
        say(t, made.len());
        Ok(())
    };
    // Side by side, the largest first (a 1024 px texture takes a few seconds, a model a few milliseconds).
    let mut rest: Vec<&&Task> = todo.iter().collect();
    rest.sort_by_key(|t| {
        (
            !matches!(t.kind, Kind::Texture(_)),
            std::cmp::Reverse(t.size),
        )
    });
    let next = AtomicUsize::new(0);
    let failed: Mutex<Option<String>> = Mutex::new(None);
    std::thread::scope(|scope| {
        for _ in 0..jobs.min(rest.len()) {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(t) = rest.get(i) else {
                        break;
                    };
                    if let Err(e) = encode(t) {
                        failed.lock().expect("no panics").get_or_insert(e);
                        break;
                    }
                }
            });
        }
    });
    if let Some(e) = failed.into_inner().expect("no panics") {
        return Err(e);
    }

    // The output tree: only copies that help (a gzip that doesn't shrink is dropped).
    let _ = std::fs::remove_dir_all(&d.out);
    let mut index = Map::new();
    // Each source's content hash (its encoding's too): what the game keys things drawn from it by
    // across releases (the catalog's pictures, `contentHash` in web/src/assets/registry.ts).
    let mut hashes = Map::new();
    let mut totals: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    for t in &tasks {
        hashes.insert(t.rel.clone(), json!(&t.key[..12]));
        let Ok(meta) = std::fs::metadata(&t.cached) else {
            continue;
        };
        let size = meta.len() as usize;
        if t.kind == Kind::Gzip && size >= t.size {
            continue;
        }
        let dest = d.out.join("opt").join(&t.out);
        if let Some(dir) = dest.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::copy(&t.cached, &dest).map_err(|e| format!("{}: {e}", dest.display()))?;
        index.insert(t.rel.clone(), json!(format!("opt/{}", t.out)));
        let group = format!(
            "{} ({})",
            t.rel.split('/').next().unwrap_or(""),
            t.kind.name()
        );
        let sum = totals.entry(group).or_default();
        sum.0 += 1;
        sum.1 += t.size;
        sum.2 += size;
    }
    let version: Map<String, J> = VERSION
        .iter()
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect();
    download::write(
        &d.out.join("optimized.json"),
        serde_json::to_string(&json!({ "version": version, "files": index, "hashes": hashes }))
            .expect("JSON")
            .as_bytes(),
    )?;
    // Forget copies of files that changed or went away (keeps the CI cache from growing).
    let live: BTreeSet<_> = tasks
        .iter()
        .filter_map(|t| t.cached.file_name().map(|n| n.to_owned()))
        .collect();
    if let Ok(entries) = std::fs::read_dir(&d.cache) {
        for entry in entries.filter_map(Result::ok) {
            if !live.contains(&entry.file_name()) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    for (rel, why) in left.into_inner().expect("no panics") {
        println!("[assets] {rel} is left as it is: {why}");
    }
    for (group, (files, before, after)) in &totals {
        println!(
            "[assets] {group:<22} {files:>3} files  {:>9} -> {:>9}",
            kb(*before),
            kb(*after)
        );
    }
    println!(
        "[assets] {} compressed copies in {:.1} s",
        index.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

/// One output tree's copies read back and compared with their sources.
#[derive(Default)]
struct Checked {
    models: usize,
    triangles: usize,
    position: f32,
    normal: f32,
    uv: f32,
    model_bytes: usize,
    textures: usize,
    texture_bytes: usize,
    /// The lowest error of a texture's largest level against its source (dB), and whose it is.
    worst: Option<(f64, String)>,
    /// Per texture: the error of its largest level against its source (dB).
    errors: BTreeMap<String, f64>,
    /// Per texture: its largest level's pixels.
    tops: BTreeMap<String, Vec<u8>>,
    /// Per texture: its width, and each level's mean of red, green and blue.
    means: BTreeMap<String, (u32, Vec<[f64; 3]>)>,
    /// Per texture: its source's mean of red, green and blue.
    sources: BTreeMap<String, [f64; 3]>,
    gzips: usize,
    gzip_bytes: usize,
    faults: Vec<String>,
}

fn check_tree(d: &Dirs, out: &Path) -> Result<Checked, String> {
    let text = std::fs::read(out.join("optimized.json"))
        .map_err(|e| format!("{}: {e}", out.join("optimized.json").display()))?;
    let index: J = serde_json::from_slice(&text).map_err(|e| format!("optimized.json: {e}"))?;
    let files = index
        .get("files")
        .and_then(J::as_object)
        .ok_or("optimized.json has no files")?;
    let mut c = Checked::default();
    for (rel, copy) in files {
        let copy_path = out.join(copy.as_str().unwrap_or(""));
        let (Ok(source), Ok(made)) = (std::fs::read(d.src.join(rel)), std::fs::read(&copy_path))
        else {
            c.faults
                .push(format!("{rel}: its source or its copy can't be read"));
            continue;
        };
        match plan(rel).map(|(kind, _)| kind) {
            Some(Kind::Model) => {
                c.models += 1;
                c.model_bytes += made.len();
                match model::read(&source)
                    .and_then(|s| model::read(&made).and_then(|m| model::compare(&s, &m)))
                {
                    Ok(drift) => {
                        c.triangles += drift.triangles;
                        c.position = c.position.max(drift.position);
                        c.normal = c.normal.max(drift.normal);
                        c.uv = c.uv.max(drift.uv);
                    }
                    Err(e) => c.faults.push(format!("{rel}: {e}")),
                }
            }
            Some(Kind::Texture(_)) => {
                c.textures += 1;
                c.texture_bytes += made.len();
                let (width, height, pixels) = match texture::rgba(&source) {
                    Ok(read) => read,
                    Err(e) => {
                        c.faults.push(format!("{rel}: its source: {e}"));
                        continue;
                    }
                };
                match texture::levels(&made) {
                    Ok(levels) => {
                        let (w, h, top) = &levels[0];
                        if (*w, *h) != (width, height) {
                            c.faults.push(format!(
                                "{rel}: {w}x{h}, its source is {:?}",
                                (width, height)
                            ));
                            continue;
                        }
                        let wanted = 32 - w.max(h).leading_zeros() as usize;
                        if levels.len() != wanted {
                            c.faults.push(format!(
                                "{rel}: {} levels, {wanted} take it down to a pixel",
                                levels.len()
                            ));
                        }
                        let error = texture::psnr(top, &pixels);
                        c.errors.insert(rel.clone(), error);
                        c.tops.insert(rel.clone(), top.clone());
                        if c.worst.as_ref().is_none_or(|(e, _)| error < *e) {
                            c.worst = Some((error, rel.clone()));
                        }
                        c.means.insert(
                            rel.clone(),
                            (
                                *w,
                                levels.iter().map(|(_, _, p)| texture::mean(p)).collect(),
                            ),
                        );
                        c.sources.insert(rel.clone(), texture::mean(&pixels));
                    }
                    Err(e) => c.faults.push(format!("{rel}: {e}")),
                }
            }
            Some(Kind::Gzip) => {
                c.gzips += 1;
                c.gzip_bytes += made.len();
                match download::gunzip(&made) {
                    Ok(back) if back == source => {}
                    Ok(_) => c.faults.push(format!("{rel}: inflates to something else")),
                    Err(e) => c.faults.push(format!("{rel}: {e}")),
                }
            }
            None => c.faults.push(format!(
                "{rel}: in the index, but not a file this tool makes a copy of"
            )),
        }
    }
    Ok(c)
}

fn report(name: &str, c: &Checked) {
    println!(
        "[assets] {name}: {} models, {:.1} MB, {} triangles; furthest from the source: position {:.3} mm, normal {:.4}, texture coordinate {:.5}",
        c.models,
        c.model_bytes as f64 / 1e6,
        c.triangles,
        c.position * 1000.0,
        c.normal,
        c.uv
    );
    let worst = c.worst.as_ref().map_or(String::new(), |(e, rel)| {
        format!("; the furthest from its source {e:.1} dB ({rel})")
    });
    println!(
        "[assets] {name}: {} textures, {:.2} MB{worst}",
        c.textures,
        c.texture_bytes as f64 / 1e6
    );
    for role in [Role::Color, Role::Normal, Role::Data] {
        let errors: Vec<f64> = c
            .errors
            .iter()
            .filter(|(rel, e)| Role::of(rel) == role && e.is_finite())
            .map(|(_, e)| *e)
            .collect();
        if !errors.is_empty() {
            println!(
                "[assets] {name}: {} maps are {:.1} dB from their sources on average (the largest level), {:.1} at worst",
                role.name(),
                errors.iter().sum::<f64>() / errors.len() as f64,
                errors.iter().copied().fold(f64::INFINITY, f64::min)
            );
        }
    }
    // A level is its source averaged down: its mean is the source's, but for what the codec and the way of
    // averaging move it (a colour map averaged in linear light is a little lighter level by level).
    for role in [Role::Color, Role::Normal, Role::Data] {
        let mut sum = 0f64;
        let mut most = 0f64;
        let mut n = 0usize;
        for (rel, (width, levels)) in c.means.iter().filter(|(rel, _)| Role::of(rel) == role) {
            let Some(source) = c.sources.get(rel) else {
                continue;
            };
            for (_, level) in levels.iter().enumerate().filter(|(i, _)| width >> i >= 16) {
                let off = (0..3).fold(0f64, |m, k| m.max((level[k] - source[k]).abs()));
                sum += off;
                most = most.max(off);
                n += 1;
            }
        }
        if n > 0 {
            println!(
                "[assets] {name}: {} maps' levels of 16 px and more are {:.2} of 255 from their source's mean on average, {most:.2} at most",
                role.name(),
                sum / n as f64
            );
        }
    }
    println!(
        "[assets] {name}: {} gzipped files, {:.2} MB",
        c.gzips,
        c.gzip_bytes as f64 / 1e6
    );
    for fault in &c.faults {
        println!("[assets] {name}: FAULT {fault}");
    }
}

fn check(against: Option<&str>) -> Result<bool, String> {
    let d = dirs();
    let ours = check_tree(&d, &d.out)?;
    report("this tool's", &ours);
    let mut ok = ours.faults.is_empty();
    // Every file this tool would make a copy of has one (a model left as it is has none: said by the build).
    let made: BTreeSet<String> = serde_json::from_slice::<J>(
        &std::fs::read(d.out.join("optimized.json")).map_err(|e| e.to_string())?,
    )
    .ok()
    .and_then(|j| {
        j.get("files")
            .and_then(J::as_object)
            .map(|m| m.keys().cloned().collect())
    })
    .unwrap_or_default();
    for t in tasks(&d)? {
        if !made.contains(&t.rel) && t.kind != Kind::Gzip {
            println!("[assets] FAULT {}: no copy (run `cargo assets`)", t.rel);
            ok = false;
        }
    }
    if let Some(dir) = against {
        let theirs = check_tree(&d, Path::new(dir))?;
        report("the other's", &theirs);
        ok &= theirs.faults.is_empty();
        // The two tools' textures against each other: the same to the pixel, or how far apart.
        for role in [Role::Color, Role::Normal, Role::Data] {
            let mut same = 0usize;
            let mut apart: Vec<f64> = Vec::new();
            for (rel, a) in ours.tops.iter().filter(|(rel, _)| Role::of(rel) == role) {
                match theirs.tops.get(rel).filter(|b| b.len() == a.len()) {
                    Some(b) if a == b => same += 1,
                    Some(b) => apart.push(texture::psnr(a, b)),
                    None => {}
                }
            }
            println!(
                "[assets] {} maps, this tool's against the other's: {same} the same to the pixel, {} others {:.1} dB apart on average",
                role.name(),
                apart.len(),
                apart.iter().sum::<f64>() / apart.len().max(1) as f64
            );
        }
        // The mip chains side by side, per kind of texture: how far the levels' means are apart, over the levels
        // of 16 px and more (a level averaged another way is lighter or darker; the last few are a handful of blocks).
        for role in [Role::Color, Role::Normal, Role::Data] {
            let mut furthest: Option<(f64, String, usize)> = None;
            let mut sum = 0f64;
            let mut n = 0usize;
            for (rel, (width, a)) in ours.means.iter().filter(|(rel, _)| Role::of(rel) == role) {
                let Some((_, b)) = theirs.means.get(rel) else {
                    continue;
                };
                for (level, (x, y)) in a
                    .iter()
                    .zip(b)
                    .enumerate()
                    .filter(|(level, _)| width >> level >= 16)
                {
                    let off = (0..3).fold(0f64, |m, k| m.max((x[k] - y[k]).abs()));
                    sum += off;
                    n += 1;
                    if furthest.as_ref().is_none_or(|(o, _, _)| off > *o) {
                        furthest = Some((off, rel.clone(), level));
                    }
                }
            }
            if let Some((off, rel, level)) = furthest {
                println!(
                    "[assets] {} maps' levels, this tool's against the other's: their means {:.2} of 255 apart on average, {off:.2} at most ({rel}, level {level})",
                    role.name(),
                    sum / n.max(1) as f64
                );
            }
        }
        println!(
            "[assets] sizes, this tool's against the other's: models {:+.1} %, textures {:+.1} %",
            (ours.model_bytes as f64 / theirs.model_bytes.max(1) as f64 - 1.0) * 100.0,
            (ours.texture_bytes as f64 / theirs.texture_bytes.max(1) as f64 - 1.0) * 100.0
        );
    }
    Ok(ok)
}

pub fn main_with(args: &[String]) -> i32 {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match a.as_slice() {
        ["-h" | "--help" | "help", ..] => {
            println!("{USAGE}");
            return 0;
        }
        ["check"] => check(None),
        ["check", "--against", dir] => check(Some(dir)),
        ["check", ..] => {
            eprintln!("{USAGE}");
            return 2;
        }
        flags => {
            let mut force = false;
            let mut verbose = false;
            let mut jobs = std::thread::available_parallelism()
                .map_or(1, |n| n.get().saturating_sub(1).max(1));
            let mut i = 0;
            while i < flags.len() {
                match flags[i] {
                    "--force" => force = true,
                    "--verbose" => verbose = true,
                    "--jobs"
                        if flags
                            .get(i + 1)
                            .and_then(|n| n.parse::<usize>().ok())
                            .is_some_and(|n| n > 0) =>
                    {
                        jobs = flags[i + 1].parse().expect("checked");
                        i += 1;
                    }
                    _ => {
                        eprintln!("{USAGE}");
                        return 2;
                    }
                }
                i += 1;
            }
            build(force, jobs, verbose).map(|()| true)
        }
    };
    match result {
        Ok(true) => 0,
        Ok(false) => 1,
        Err(e) => {
            eprintln!("assets: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_each_file_becomes() {
        assert_eq!(
            plan("models/kitchen/fridge.glb"),
            Some((Kind::Model, "models/kitchen/fridge.glb".into()))
        );
        assert_eq!(
            plan("textures/brick/albedo.jpg"),
            Some((
                Kind::Texture(Role::Color),
                "textures/brick/albedo.ktx2".into()
            ))
        );
        assert_eq!(
            plan("textures/brick/normal.jpg"),
            Some((
                Kind::Texture(Role::Normal),
                "textures/brick/normal.ktx2".into()
            ))
        );
        assert_eq!(
            plan("textures/brick/arm.jpg"),
            Some((Kind::Texture(Role::Data), "textures/brick/arm.ktx2".into()))
        );
        assert_eq!(
            plan("characters/male.bin"),
            Some((Kind::Gzip, "characters/male.bin.gz".into()))
        );
        assert_eq!(plan("other/thing.bin"), None);
        assert_eq!(plan("textures/sky/clouds.png"), None);
        assert_eq!(plan("icons/app.png"), None);
        assert_eq!(plan("manifest.json"), None);
    }

    #[test]
    fn paths_without_their_dots() {
        assert_eq!(
            normalize("packs/trees/../../models/trees/oak.glb"),
            "models/trees/oak.glb"
        );
        assert_eq!(normalize("./models/a.glb"), "models/a.glb");
    }
}
