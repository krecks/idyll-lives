//! The pictures' files. `cargo check-glb --preview` draws each model the gate checked from its file as read
//! (`artkit::glb::read`, `artkit::preview`) into tools/art/blender/previews/<family>/rt_<name>.png (its clips,
//! perches and far version beside it, as `cargo art --preview` names them, and its sheet with `--sheet`), so a
//! fault in a file shows; `cargo art --preview` writes its pictures with `save`. A cut-out finish (the leaf
//! cards) is drawn on its texture and cut by its alpha (`Textures`).

use super::py::{J, dirname, join, normpath};
use super::{Paths, asset_manifests, model_entries};
use artkit::preview::{self, Maps};
use artkit::raster::{Picture, Texture};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Writes `picture` as a PNG (its folder made).
pub fn save(path: &Path, picture: &Picture) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let image = image::RgbaImage::from_raw(
        picture.width as u32,
        picture.height as u32,
        picture.rgba.clone(),
    )
    .ok_or_else(|| format!("{}: a picture of the wrong size", path.display()))?;
    image
        .save_with_format(path, image::ImageFormat::Png)
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// tools/art/blender/previews: the pictures' folder (ignored by git).
pub fn folder(paths: &Paths) -> String {
    join(&paths.here, "previews")
}

/// An entry's materials: (material, finish key), its `materials` in order.
pub fn materials(entry: &J) -> Vec<(String, String)> {
    entry
        .get("materials")
        .and_then(J::entries)
        .unwrap_or(&[])
        .iter()
        .filter_map(|(name, m)| {
            let finish = m.str().or_else(|| m.get("finish").and_then(J::str))?;
            Some((name.clone(), finish.to_string()))
        })
        .collect()
}

/// An entry's perches: [x, height, depth] each.
pub fn perches(entry: &J) -> Vec<[f64; 3]> {
    entry
        .get("perches")
        .and_then(J::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|p| {
            let p = p.list()?;
            let n = |i: usize| p.get(i).and_then(J::num).map(|n| n.f());
            Some([n(0)?, n(1)?, n(2)?])
        })
        .collect()
}

/// The textured finishes (a `texture` in the base manifest: wood, fabric, bark; with a `cutout`, the leaf cards)
/// and their textures, each read once.
pub struct Textures {
    assets: String,
    /// finish key: (texture, the alpha it is cut at, for a cut-out one)
    finishes: BTreeMap<String, (String, Option<f32>)>,
    loaded: BTreeMap<String, Texture>,
}

impl Textures {
    pub fn new(paths: &Paths) -> Result<Textures, String> {
        let mut finishes = BTreeMap::new();
        for (_, m) in asset_manifests(paths)? {
            for (key, e) in m.get("entries").and_then(J::entries).unwrap_or(&[]) {
                if let Some(texture) = e.get("texture").and_then(J::str)
                    && key.starts_with("material.finish.")
                {
                    let cut = e.get("cutout").and_then(J::num).map(|c| c.f() as f32);
                    finishes.insert(key.clone(), (texture.to_string(), cut));
                }
            }
        }
        Ok(Textures {
            assets: paths.assets.clone(),
            finishes,
            loaded: BTreeMap::new(),
        })
    }

    fn load(&mut self, texture: &str) -> Result<&Texture, String> {
        if !self.loaded.contains_key(texture) {
            let path = join(&self.assets, texture);
            let image = image::open(&path)
                .map_err(|e| format!("{path}: {e}"))?
                .to_rgba8();
            let (width, height) = (image.width() as usize, image.height() as usize);
            self.loaded.insert(
                texture.to_string(),
                Texture {
                    width,
                    height,
                    rgba: image.into_raw(),
                },
            );
        }
        Ok(&self.loaded[texture])
    }

    /// What a model's `materials` ((material, finish key) each) are drawn with: the cut-out ones' textures, the
    /// other textured ones' tints.
    pub fn maps(&mut self, materials: &[(String, String)]) -> Result<Maps, String> {
        let mut out = Maps::default();
        let mut at: BTreeMap<String, usize> = BTreeMap::new();
        for (name, finish) in materials {
            let Some((texture, cut)) = self.finishes.get(finish).cloned() else {
                continue;
            };
            let Some(cut) = cut else {
                out.tints
                    .insert(name.clone(), average(self.load(&texture)?));
                continue;
            };
            let index = match at.get(&texture) {
                Some(&i) => i,
                None => {
                    out.textures.push(self.load(&texture)?.clone());
                    at.insert(texture.clone(), out.textures.len() - 1);
                    out.textures.len() - 1
                }
            };
            out.cutouts.insert(name.clone(), (index, cut));
        }
        Ok(out)
    }
}

/// A texture's average colour, linear RGB.
fn average(t: &Texture) -> [f32; 3] {
    let lut: Vec<f64> = (0..256)
        .map(|i| artkit::raster::linear([i as u8, 0, 0])[0] as f64)
        .collect();
    let mut sum = [0.0f64; 3];
    for p in t.rgba.as_chunks::<4>().0 {
        for k in 0..3 {
            sum[k] += lut[p[k] as usize];
        }
    }
    let n = (t.rgba.len() / 4).max(1) as f64;
    sum.map(|s| (s / n) as f32)
}

/// A model to draw: its key (for the log), the path its pictures are named after (the folder and the name, which
/// each picture's suffix and ".png" follow), its file, its far version's and its pack entry (its materials'
/// finishes, its perches).
pub struct Subject {
    pub key: String,
    pub stem: PathBuf,
    pub glb: Vec<u8>,
    pub far: Option<Vec<u8>>,
    pub entry: J,
}

/// Draws one: its pictures (`preview::previews`).
fn draw(subject: &Subject, maps: &Maps, sheet: bool) -> Result<Vec<(String, Picture)>, String> {
    let model = artkit::glb::read(&subject.glb)?;
    let far = subject
        .far
        .as_ref()
        .map(|f| artkit::glb::read(f).map_err(|e| format!("its far version: {e}")))
        .transpose()?;
    Ok(preview::previews(
        &model,
        &perches(&subject.entry),
        far.as_ref(),
        sheet,
        maps,
    ))
}

/// Draws `subjects` and writes their pictures (`<stem><suffix>.png`), as many at once as there are processors,
/// saying where each went (its path from `shown`). What failed, each with why.
pub fn draw_all(paths: &Paths, subjects: &[Subject], sheet: bool, shown: &Path) -> Vec<String> {
    let started = std::time::Instant::now();
    let mut failed = Vec::new();
    let mut textures = match Textures::new(paths) {
        Ok(t) => t,
        Err(err) => return vec![err],
    };
    let mut jobs: Vec<(&Subject, Maps)> = Vec::new();
    for s in subjects {
        match textures.maps(&materials(&s.entry)) {
            Ok(maps) => jobs.push((s, maps)),
            Err(err) => failed.push(format!("{}: {err}", s.key)),
        }
    }
    let at_once = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut pictures = 0;
    for chunk in jobs.chunks(at_once) {
        let results: Vec<_> = std::thread::scope(|s| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|(subject, maps)| s.spawn(move || draw(subject, maps, sheet)))
                .collect();
            handles
                .into_iter()
                .map(|h| {
                    h.join()
                        .unwrap_or_else(|_| Err("drawing it panicked".into()))
                })
                .collect()
        });
        for ((subject, _), result) in chunk.iter().zip(results) {
            let drawn = match result {
                Ok(r) => r,
                Err(err) => {
                    failed.push(format!("{}: {err}", subject.key));
                    continue;
                }
            };
            for (suffix, picture) in &drawn {
                let mut name = subject.stem.clone().into_os_string();
                name.push(format!("{suffix}.png"));
                if let Err(err) = save(Path::new(&name), picture) {
                    failed.push(err);
                }
            }
            pictures += drawn.len();
            let stem = subject.stem.strip_prefix(shown).unwrap_or(&subject.stem);
            println!(
                "[gamekit] previewed {} -> {}.png",
                subject.key,
                stem.display()
            );
        }
    }
    println!(
        "[gamekit] {pictures} pictures of {} models in {:.1} s",
        jobs.len(),
        started.elapsed().as_secs_f64()
    );
    failed
}

/// `cargo check-glb --preview` in Rust: each of `checked` (keys) whose file (`files`, in step) is there, drawn
/// from its file into previews/<family>/rt_<name>…png. The exit code.
pub fn review(paths: &Paths, checked: &[String], files: &[String], sheet: bool) -> i32 {
    let stack = match asset_manifests(paths) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    let (entries, _) = model_entries(&stack);
    let root = PathBuf::from(folder(paths));
    let mut subjects = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut failed = Vec::new();
    for (key, file) in checked.iter().zip(files) {
        if !Path::new(file).exists() || !seen.insert(file.clone()) {
            continue;
        }
        let (entry, manifest) = entries
            .get(key)
            .cloned()
            .unwrap_or((J::Null, String::new()));
        let glb = match std::fs::read(file) {
            Ok(b) => b,
            Err(e) => {
                failed.push(format!("{file}: {e}"));
                continue;
            }
        };
        let far = entry
            .get("far")
            .and_then(J::str)
            .map(|url| normpath(&join(&dirname(&manifest), url)))
            .and_then(|path| std::fs::read(path).ok());
        let file = Path::new(file);
        // (by family, as the build's pictures: an id may be in two)
        let dir = match file.parent().and_then(Path::file_name) {
            Some(f) if f != "models" => root.join(f),
            _ => root.clone(),
        };
        let name = file
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        subjects.push(Subject {
            key: key.clone(),
            stem: dir.join(format!("rt_{name}")),
            glb,
            far,
            entry,
        });
    }
    failed.extend(draw_all(paths, &subjects, sheet, Path::new(&paths.here)));
    for f in &failed {
        eprintln!("{f}");
    }
    if failed.is_empty() { 0 } else { 1 }
}
