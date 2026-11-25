//! `cargo art`: builds the families of item models and checks what they wrote.
//!
//!   cargo art kitchen                  build, write and register a family, then run the gate on what it wrote
//!   cargo art kitchen --only fridge    one model of it (--only a,b: several)
//!   cargo art --all                    every family
//!   cargo art kitchen --no-pack        write the files without writing the pack
//!   cargo art trees --out /tmp/trees   write the files, the pack and centres.json under another root, laid out
//!                                      as the repo is (web/public/assets/…, tools/art/centres.json)
//!   cargo art trees --check            build in memory and fail on any byte that differs from the committed
//!                                      files, pack entries or centres (--all --check: every family)
//!   cargo art --centres                write centres.json from every family's models (no build)
//!   cargo art trees --preview [--sheet]  build in memory and draw each model (`preview`) into
//!                                      tools/art/previews/trees/, writing nothing else
//!   cargo art template --preview       draw the template family (`template`), which nothing else builds
//!
//! The registry (`registry`: the group crates nature, wet, rooms, packs, outside and things) names every family,
//! and each is built here by the kernel (`artkit`), a thread a model. The content and the manifests are read here,
//! the pack and centres.json written (`ll_tools::art::take_build`), and the gate (`cargo check-glb`) run on the
//! keys built; the exit code is 1 if a build or the gate fails.

use artkit::family::{Family, Output, build_model};
use artkit::json::Json;
use artkit::materials::{Finish, Finishes};
use artkit::scene;
use ll_tools::art::{J, build_keys, take_build, write_centres};
use ll_tools::checkglb::{self, Paths};
use std::collections::BTreeSet;
use std::path::Path;

pub mod preview;
pub mod template;

/// Every family by its name on the command line (`cargo art <name>`).
pub fn registry() -> Vec<(&'static str, Family)> {
    let mut all = art_nature::families();
    all.extend(art_wet::families());
    all.extend(art_rooms::families());
    all.extend(art_packs::families());
    all.extend(art_outside::families());
    all.extend(art_things::families());
    all
}

/// The families `names` asks for, in the order asked (`all`: every family of the registry). An error for a name the
/// registry doesn't have.
pub fn pick<'a>(
    names: &[String],
    registry: &'a [(&'static str, Family)],
    all: bool,
) -> Result<Vec<&'a Family>, String> {
    let listed = || {
        let all: Vec<&str> = registry.iter().map(|(n, _)| *n).collect();
        all.join(", ")
    };
    let asked: Vec<&str> = if all {
        registry.iter().map(|(n, _)| *n).collect()
    } else if names.is_empty() {
        return Err(format!("name a family ({}) or --all", listed()));
    } else {
        names.iter().map(String::as_str).collect()
    };
    asked
        .into_iter()
        .map(|name| match registry.iter().find(|(n, _)| *n == name) {
            Some((_, family)) => Ok(family),
            None if name == template::NAME => Err(format!(
                "{name}: the template only previews (cargo art {name} --preview); copy it to make a family (crates/art/README.md)"
            )),
            None => Err(format!("no family {name:?} ({})", listed())),
        })
        .collect()
}

/// The centres of every model of `families`, by key: what `--centres` writes.
pub fn centres_of(families: &[&Family]) -> J {
    J::Obj(
        families
            .iter()
            .flat_map(|f| {
                f.models.iter().map(|m| {
                    (
                        f.key(m),
                        J::Arr(vec![J::Float(m.centre[0]), J::Float(m.centre[1])]),
                    )
                })
            })
            .collect(),
    )
}

pub(crate) fn to_j(v: &Json) -> J {
    match v {
        Json::Null => J::Null,
        Json::Bool(b) => J::Bool(*b),
        Json::Int(i) => J::Int(*i),
        Json::Num(x) => J::Float(*x),
        Json::Str(s) => J::Str(s.clone()),
        Json::Arr(items) => J::Arr(items.iter().map(to_j).collect()),
        Json::Obj(fields) => J::Obj(fields.iter().map(|(k, v)| (k.clone(), to_j(v))).collect()),
    }
}

/// The base manifest's finishes (`material.finish.*`), as far as a build needs them.
pub fn finishes(paths: &Paths) -> Result<Finishes, String> {
    let path = Path::new(&paths.assets).join("manifest.json");
    let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let manifest = ll_tools::art::loads(&data)?;
    let mut out = Finishes::new();
    for (key, e) in manifest.get("entries").and_then(J::entries).unwrap_or(&[]) {
        if let Some(name) = key.strip_prefix("material.finish.") {
            let number = |k: &str, or: f64| e.get(k).and_then(J::num).map(|n| n.f()).unwrap_or(or);
            let finish = Finish {
                roughness: number("roughness", 0.6),
                metallic: number("metallic", 0.0),
                textured: e.get("texture").is_some_and(J::truthy),
            };
            out.insert(name.to_string(), finish);
        }
    }
    if out.is_empty() {
        return Err(format!("{}: no material.finish.* entries", path.display()));
    }
    Ok(out)
}

/// What a run was asked for.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Just these ids (the pack keeps the others' entries).
    pub only: Option<Vec<String>>,
    pub no_pack: bool,
    /// Write under this root, not the repo.
    pub out: Option<String>,
    /// Write nothing: compare with what is committed.
    pub check: bool,
}

/// What a family's build made: each model's key, id and files, with what it said.
pub(crate) struct Built {
    pub(crate) key: String,
    pub(crate) id: &'static str,
    pub(crate) output: Result<Output, String>,
    pub(crate) log: Vec<String>,
}

/// Builds `models` of `family`, a thread a model (each has a scene of its own), as many at once as there are
/// processors.
pub(crate) fn build_all(family: &Family, ids: &[usize], finishes: &Finishes) -> Vec<Built> {
    build_many(&[(family, ids)], finishes)
        .pop()
        .unwrap_or_default()
}

/// Builds the models of every family in `work` from one queue, so a slow model holds up no batch behind it: each
/// model on a thread of its own (a fresh scene), as many at once as there are processors. Each family's models,
/// in the order asked.
pub(crate) fn build_many(work: &[(&Family, &[usize])], finishes: &Finishes) -> Vec<Vec<Built>> {
    let jobs: Vec<(usize, usize)> = work
        .iter()
        .enumerate()
        .flat_map(|(f, (_, ids))| (0..ids.len()).map(move |k| (f, k)))
        .collect();
    let at_once = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(1, jobs.len().max(1));
    let slots: Vec<Vec<std::sync::Mutex<Option<Built>>>> = work
        .iter()
        .map(|(_, ids)| ids.iter().map(|_| std::sync::Mutex::new(None)).collect())
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|workers| {
        for _ in 0..at_once {
            workers.spawn(|| {
                while let Some(&(f, k)) =
                    jobs.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                {
                    let (family, ids) = work[f];
                    let model = &family.models[ids[k]];
                    // (a thread a model: the scene, its finishes and its log are the thread's own)
                    let joined = std::thread::scope(|s| {
                        s.spawn(|| {
                            scene::set_finishes(finishes.clone());
                            let output = build_model(family, model);
                            (output, scene::take_log())
                        })
                        .join()
                    });
                    let (output, log) = match joined {
                        Ok(r) => r,
                        // (a build that panicked said why on the way: an assertion of the kernel's or its own)
                        Err(p) => {
                            let why = p
                                .downcast_ref::<String>()
                                .cloned()
                                .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()));
                            (
                                Err(why.unwrap_or_else(|| "the build panicked".into())),
                                Vec::new(),
                            )
                        }
                    };
                    *slots[f][k].lock().unwrap_or_else(|e| e.into_inner()) = Some(Built {
                        key: family.key(model),
                        id: model.id,
                        output,
                        log,
                    });
                }
            });
        }
    });
    slots
        .into_iter()
        .map(|family| {
            family
                .into_iter()
                .map(|slot| {
                    slot.into_inner()
                        .unwrap_or_else(|e| e.into_inner())
                        .expect("every model was built")
                })
                .collect()
        })
        .collect()
}

/// A root laid out as the repo, with the manifests and centres.json a pack is written from copied in (those it
/// hasn't yet): what `--out` writes into, and what `--check` compares in.
fn seeded(root: &str, repo: &Paths) -> Result<Paths, String> {
    let paths = Paths::at(root);
    let copy = |from: &Path, to: &Path| -> Result<(), String> {
        if to.exists() || !from.exists() {
            return Ok(());
        }
        if let Some(dir) = to.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::copy(from, to)
            .map(|_| ())
            .map_err(|e| format!("{}: {e}", to.display()))
    };
    copy(
        &Path::new(&repo.assets).join("manifest.json"),
        &Path::new(&paths.assets).join("manifest.json"),
    )?;
    copy(Path::new(&repo.centres), Path::new(&paths.centres))?;
    let packs = Path::new(&repo.assets).join("packs");
    for e in std::fs::read_dir(&packs)
        .map_err(|e| format!("{}: {e}", packs.display()))?
        .flatten()
    {
        let from = e.path().join("manifest.json");
        copy(
            &from,
            &Path::new(&paths.assets)
                .join("packs")
                .join(e.file_name())
                .join("manifest.json"),
        )?;
    }
    Ok(paths)
}

/// Builds `family` as `options` say. The keys it built (for the gate), or what failed.
pub fn run_family(repo: &Paths, family: &Family, options: &Options) -> Result<Vec<String>, String> {
    run_families(repo, &[family], options)
        .pop()
        .unwrap_or_else(|| Ok(Vec::new()))
}

/// What a family needs before its models are built: which of them, the finishes and the keys the game knows.
struct Prepared {
    ids: Vec<usize>,
    known: BTreeSet<String>,
}

/// Builds `families` as `options` say, their models all from one queue (`build_many`), then writes or checks each
/// family in turn. For each, the keys it built (for the gate), or what failed.
pub fn run_families(
    repo: &Paths,
    families: &[&Family],
    options: &Options,
) -> Vec<Result<Vec<String>, String>> {
    let finishes = match finishes(repo) {
        Ok(f) => f,
        Err(e) => return families.iter().map(|_| Err(e.clone())).collect(),
    };
    // A key the game wouldn't draw is a misspelt `key`: refused before any model is built or written.
    let known =
        build_keys(repo).map_err(|e| format!("cannot read the content or the manifests: {e}"));
    let prepared: Vec<Result<Prepared, String>> = families
        .iter()
        .map(|family| {
            let known = known.clone()?;
            prepare(family, options, &finishes, known)
        })
        .collect();
    let work: Vec<(&Family, &[usize])> = families
        .iter()
        .zip(&prepared)
        .filter_map(|(family, p)| p.as_ref().ok().map(|p| (*family, p.ids.as_slice())))
        .collect();
    let mut built = build_many(&work, &finishes).into_iter();
    families
        .iter()
        .zip(prepared)
        .map(|(family, p)| {
            let p = p?;
            let built = built.next().unwrap_or_default();
            finish(repo, family, options, &p, built)
        })
        .collect()
}

/// Whether `family` may be built as `options` say, and which of its models.
fn prepare(
    family: &Family,
    options: &Options,
    finishes: &Finishes,
    known: BTreeSet<String>,
) -> Result<Prepared, String> {
    let finishes = finishes.clone();
    family
        .palette
        .check(&finishes)
        .map_err(|e| format!("{}: {e}", family.name))?;
    for model in &family.models {
        if let Some(own) = &model.palette {
            own.check(&finishes)
                .map_err(|e| format!("{} {}: {e}", family.name, model.id))?;
        }
    }
    let mut ids: Vec<usize> = (0..family.models.len()).collect();
    if let Some(only) = &options.only {
        let unknown: Vec<&str> = only
            .iter()
            .map(String::as_str)
            .filter(|id| !family.models.iter().any(|m| m.id == *id))
            .collect();
        if !unknown.is_empty() {
            return Err(format!("{}: no model {}", family.name, unknown.join(", ")));
        }
        ids.retain(|&i| only.iter().any(|id| id == family.models[i].id));
    }
    let mut unknown: Vec<String> = ids
        .iter()
        .map(|&i| &family.models[i])
        .filter(|m| !m.code)
        .map(|m| family.key(m))
        .filter(|k| !known.contains(k))
        .collect();
    unknown.sort();
    if !unknown.is_empty() {
        return Err(format!(
            "{}: no content object or item names {} and no manifest has it: a misspelt key? (a model the game draws by code takes .code())",
            family.name,
            unknown.join(", ")
        ));
    }
    Ok(Prepared { ids, known })
}

/// Writes (or `--check`s) what `family`'s build made: its files, its pack and its centres.
fn finish(
    repo: &Paths,
    family: &Family,
    options: &Options,
    prepared: &Prepared,
    built: Vec<Built>,
) -> Result<Vec<String>, String> {
    let (ids, known) = (&prepared.ids, &prepared.known);
    let mut failed = Vec::new();
    for b in &built {
        b.log.iter().for_each(|l| println!("{l}"));
        if let Err(why) = &b.output {
            println!("[art] {}: {why}", b.id);
            failed.push(b.id);
        }
    }
    if !failed.is_empty() {
        return Err(format!(
            "{}: {} did not build",
            family.name,
            failed.join(", ")
        ));
    }
    // What the build hands the pack's writer.
    let entry = |f: &dyn Fn(&Built, &Output) -> J| -> J {
        J::Obj(
            built
                .iter()
                .filter_map(|b| b.output.as_ref().ok().map(|o| (b.key.clone(), f(b, o))))
                .collect(),
        )
    };
    let mut code: Vec<String> = ids
        .iter()
        .map(|&i| &family.models[i])
        .filter(|m| m.code)
        .map(|m| family.key(m))
        .collect();
    code.sort();
    let centre = |b: &Built| {
        let c = family
            .models
            .iter()
            .find(|m| m.id == b.id)
            .map(|m| m.centre)
            .unwrap_or([0.0, 0.0]);
        J::Arr(vec![J::Float(c[0]), J::Float(c[1])])
    };
    let result = J::Obj(vec![
        ("pack".into(), J::Str(family.pack.to_string())),
        ("comment".into(), J::Str(family.comment.to_string())),
        (
            "code".into(),
            J::Arr(code.into_iter().map(J::Str).collect()),
        ),
        ("entries".into(), entry(&|_, o| to_j(&o.entry))),
        ("centres".into(), entry(&|b, _| centre(b))),
        (
            "clips".into(),
            entry(&|_, o| J::Arr(o.clips.iter().cloned().map(J::Str).collect())),
        ),
    ]);
    let files = |assets: &str| -> Vec<(String, &Vec<u8>)> {
        let mut out = Vec::new();
        for b in &built {
            if let Ok(o) = &b.output {
                out.push((
                    format!("{assets}/models/{}/{}.glb", family.name, b.id),
                    &o.glb,
                ));
                if let Some(far) = &o.far {
                    out.push((
                        format!("{assets}/models/{}/{}.far.glb", family.name, b.id),
                        far,
                    ));
                }
            }
        }
        out
    };
    let keys: Vec<String> = built.iter().map(|b| b.key.clone()).collect();
    if options.check {
        return check(repo, family, &result, &files(&repo.assets), known).map(|()| Vec::new());
    }
    let paths = match &options.out {
        Some(root) => seeded(root, repo)?,
        None => Paths::repo(),
    };
    for (path, bytes) in files(&paths.assets) {
        if let Some(dir) = Path::new(&path).parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::write(&path, bytes).map_err(|e| format!("{path}: {e}"))?;
    }
    take_build(&paths, &result, options.no_pack, known)?
        .iter()
        .for_each(|l| println!("{l}"));
    // (Under another root there is no content for the gate to read the keys' objects from: it is run on the repo's.)
    Ok(if options.out.is_some() {
        Vec::new()
    } else {
        keys
    })
}

/// `--check`: the files in memory against the committed ones, and the pack and centres a build would write
/// against the committed ones, written in a scratch root.
fn check(
    repo: &Paths,
    family: &Family,
    result: &J,
    files: &[(String, &Vec<u8>)],
    known: &BTreeSet<String>,
) -> Result<(), String> {
    let mut differ = Vec::new();
    for (path, bytes) in files {
        if std::fs::read(path).ok().as_ref() != Some(*bytes) {
            differ.push(
                path.strip_prefix(&format!("{}/", repo.assets))
                    .unwrap_or(path)
                    .to_string(),
            );
        }
    }
    let root = std::env::temp_dir().join(format!(
        "ll-art-check-{}-{}",
        std::process::id(),
        family.name
    ));
    let _ = std::fs::remove_dir_all(&root);
    let scratch = seeded(&root.to_string_lossy(), repo)?;
    let taken = take_build(&scratch, result, false, known);
    let same = |a: &str, b: &str| std::fs::read(a).ok() == std::fs::read(b).ok();
    let pack = format!("packs/{}/manifest.json", family.pack);
    if taken.is_ok() {
        if !same(
            &format!("{}/{pack}", scratch.assets),
            &format!("{}/{pack}", repo.assets),
        ) {
            differ.push(pack);
        }
        if !same(&scratch.centres, &repo.centres) {
            differ.push("centres.json".into());
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    taken?;
    if differ.is_empty() {
        println!(
            "[art] {}: {} files, its pack and its centres as committed",
            family.name,
            files.len()
        );
        Ok(())
    } else {
        Err(format!(
            "{}: not as committed: {}",
            family.name,
            differ.join(", ")
        ))
    }
}

fn exit(msg: String) -> i32 {
    eprintln!("{msg}");
    1
}

/// The options `cargo art` takes (`--only` and `--out` with a value).
const FLAGS: [&str; 8] = [
    "--all",
    "--only",
    "--no-pack",
    "--out",
    "--check",
    "--centres",
    "--preview",
    "--sheet",
];

/// `cargo art …`: the exit code.
pub fn main(argv: &[String]) -> i32 {
    let has = |flag: &str| argv.iter().any(|a| a == flag);
    let value = |flag: &str| {
        argv.iter()
            .position(|a| a == flag)
            .map(|i| argv.get(i + 1).cloned())
    };
    if let Some(odd) = argv
        .iter()
        .find(|a| a.starts_with('-') && !FLAGS.contains(&a.as_str()))
    {
        return exit(format!("no option {odd} ({})", FLAGS.join(", ")));
    }
    let registry = registry();
    let repo = Paths::repo();
    if has("--centres") {
        // Every family's models and no others: a key the families don't build isn't centred here.
        let families = match pick(&[], &registry, true) {
            Ok(f) => f,
            Err(err) => return exit(err),
        };
        if let Err(err) = write_centres(&repo, &centres_of(&families), true) {
            return exit(err);
        }
        let models: usize = families.iter().map(|f| f.models.len()).sum();
        println!(
            "[art] wrote tools/art/centres.json: {models} models of {} families",
            families.len()
        );
        return 0;
    }
    let (only, out) = match (value("--only"), value("--out")) {
        (Some(None), _) => return exit("--only names a model: --only <id>[,<id>…]".into()),
        (_, Some(None)) => return exit("--out names a folder: --out <dir>".into()),
        (only, out) => (only.flatten(), out.flatten()),
    };
    let mut names: Vec<String> = argv
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();
    for taken in [&only, &out].into_iter().flatten() {
        if let Some(i) = names.iter().position(|n| n == taken) {
            names.remove(i);
        }
    }
    let options = Options {
        only: only
            .as_ref()
            .map(|o| o.split(',').map(String::from).collect()),
        no_pack: has("--no-pack"),
        out,
        check: has("--check"),
    };
    let into = std::path::PathBuf::from(ll_tools::checkglb::preview::folder(&repo));
    if has("--preview") && names == [template::NAME] {
        let family = template::family();
        return match preview::run(
            &repo,
            &family,
            options.only.as_deref(),
            has("--sheet"),
            &into,
        ) {
            Ok(()) => 0,
            Err(err) => exit(err),
        };
    }
    let families = match pick(&names, &registry, has("--all")) {
        Ok(f) => f,
        Err(err) => return exit(err),
    };
    let (mut keys, mut failed) = (Vec::new(), Vec::new());
    if has("--preview") {
        // Pictures only: no file, pack or gate.
        for family in families {
            if let Err(err) = preview::run(
                &repo,
                family,
                options.only.as_deref(),
                has("--sheet"),
                &into,
            ) {
                println!("{err}");
                failed.push(family.name.to_string());
            }
        }
    } else {
        for (family, result) in families
            .iter()
            .zip(run_families(&repo, &families, &options))
        {
            match result {
                Ok(built) => keys.extend(built),
                Err(err) => {
                    println!("{err}");
                    failed.push(family.name.to_string());
                }
            }
        }
    }
    let keys: Vec<String> = keys
        .into_iter()
        .filter(|k| k.starts_with("model.") || k.starts_with("item.") || k.starts_with("tool."))
        .collect();
    if !keys.is_empty() {
        let report = checkglb::gate(&repo, &keys);
        print!("{}", report.text);
        if report.failed > 0 {
            failed.push("check-glb".into());
        }
    }
    if !failed.is_empty() {
        return exit(format!("failed: {}", failed.join(", ")));
    }
    0
}
