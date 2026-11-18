//! `cargo art`: builds the families of item models and checks what they wrote.
//!
//!   cargo art kitchen                  build, write and register a family
//!   cargo art kitchen --only fridge    one model of it (--only a,b: several)
//!   cargo art --all                    every family (--all --check: every switched one)
//!   cargo art kitchen --no-pack        write the files without writing the pack
//!   cargo art trees --out /tmp/trees   write the files, the pack and centres.json under another root, laid out
//!                                      as the repo is (web/public/assets/…, tools/art/blender/centres.json), for
//!                                      comparing with what is committed
//!   cargo art trees --check            build in memory and fail on any byte that differs from the committed
//!                                      files, pack entries or centres
//!   cargo art --centres                write centres.json from every family (no build)
//!   cargo art trees --preview [--sheet]  build in memory and draw each model (`preview`) into
//!                                      tools/art/blender/previews/trees/, writing nothing else
//!
//! The registry says where a family is built (`route`; the group crates here: nature, wet, rooms, packs, outside,
//! things), not whether a script of its name is left. A switched family (`Family::switched`: its files in the repo
//! are the ones built here) is built in this process by the kernel (`artkit`), a thread a model, whatever is asked,
//! and though its script stays as the other scripts' library (plants.py). One ported and not switched is still its
//! script run in Blender (`ll_tools::art`, which also keeps its `--preview`, `--lit` and `--leafcards`), but for
//! `--out` and `--check`, which build the models it has here: how a port is proved before the switch. Any other is
//! its script. `--all` builds every switched family here and every other that has a script in Blender;
//! `--all --check` holds the switched ones to the committed files and leaves the rest out. Either way the content
//! and the manifests are read here, the pack and centres.json written by the same code
//! (`ll_tools::art::take_build`), and the gate (`cargo check-glb`) run on the keys built; the exit code is 1 if a
//! build or the gate fails.

use artkit::family::{Family, Output, build_model};
use artkit::json::Json;
use artkit::materials::{Finish, Finishes};
use artkit::scene;
use ll_tools::art::{J, build_keys, take_build, write_centres};
use ll_tools::checkglb::{self, Paths};
use std::collections::BTreeSet;
use std::path::Path;

pub mod preview;

/// Every family by its name on the command line, each once it is ported to Rust.
pub fn registry() -> Vec<(&'static str, Option<Family>)> {
    let mut all = art_nature::families();
    all.extend(art_wet::families());
    all.extend(art_rooms::families());
    all.extend(art_packs::families());
    all.extend(art_outside::families());
    all.extend(art_things::families());
    all
}

/// How far a family's port has come: what the registry says of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Port {
    /// Its module builds nothing yet.
    Unported,
    /// Its module builds it, and the files in the repo are still Blender's.
    Ported,
    /// The files in the repo are the ones built here (`Family::switched`).
    Switched,
}

impl Port {
    pub fn of(family: Option<&Family>) -> Port {
        match family {
            None => Port::Unported,
            Some(f) if f.switched => Port::Switched,
            Some(_) => Port::Ported,
        }
    }
}

/// What of the command line says where a family is built (`--only`, `--no-pack` and `--preview` say nothing of it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Asked {
    pub all: bool,
    pub out: bool,
    pub check: bool,
}

/// What a run builds where.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Route {
    /// The families built in this process, in the order asked.
    pub here: Vec<String>,
    /// The families whose scripts are run in Blender.
    pub blender: Vec<String>,
    /// What to say of it, a line each.
    pub notes: Vec<String>,
}

/// Where each of `names` (with `--all`: each of the registry's) is built. The registry decides, not whether a
/// script of that name is left:
///
///   switched               here, whatever is asked (its script may stay as the other scripts' library)
///   ported, not switched   here with --out or --check (how a port is proved before the switch); else its script
///                          in Blender, with a line saying so
///   not ported             its script in Blender; none with --out or --check
///   --all                  every switched family here, every other that has a script in Blender
///   --all --check          every switched family; the others left out, with a line saying how many (a port's
///                          files aren't expected to be the Blender-built ones)
///   --all --out            every ported family, switched or not; the others left out
///
/// An error for a name that is neither ported nor a script, and for one nothing can build as asked.
pub fn route(
    names: &[String],
    registry: &[(&str, Port)],
    scripts: &[String],
    asked: Asked,
) -> Result<Route, String> {
    let port = |name: &str| {
        registry
            .iter()
            .find(|(n, _)| *n == name)
            .map_or(Port::Unported, |(_, p)| *p)
    };
    let script = |name: &str| scripts.iter().any(|s| s == name);
    let listed = || {
        let all: Vec<&str> = registry.iter().map(|(n, _)| *n).collect();
        all.join(", ")
    };
    let in_blender = |name: &str| {
        format!(
            "[gamekit] {name}: ported, not switched: built in Blender (--out or --check build it here)"
        )
    };
    const GONE: &str = "ported, not switched, and no script of that name is left";
    const SWITCH: &str = "--out or --check build it here; `.switched()` in its module once its files are the committed ones";
    let proving = asked.out || asked.check;
    let mut route = Route::default();
    if asked.all {
        let mut left = 0;
        for &(name, port) in registry {
            if port == Port::Switched || (port == Port::Ported && asked.out && !asked.check) {
                route.here.push(name.to_string());
            } else if proving {
                left += usize::from(port == Port::Ported || script(name));
            } else if script(name) {
                if port == Port::Ported {
                    route.notes.push(in_blender(name));
                }
                route.blender.push(name.to_string());
            } else if port == Port::Ported {
                route
                    .notes
                    .push(format!("[gamekit] {name}: {GONE}: not built ({SWITCH})"));
            }
        }
        if left > 0 {
            route.notes.push(if asked.check {
                format!(
                    "[gamekit] not switched, left out: {left} (--check holds a switched family to the committed files)"
                )
            } else {
                format!("[gamekit] still built in Blender, left out: {left}")
            });
        }
        return Ok(route);
    }
    if names.is_empty() {
        return Err(format!("name a family ({}) or --all", listed()));
    }
    if let Some(odd) = names
        .iter()
        .find(|n| port(n) == Port::Unported && !script(n))
    {
        return Err(format!("no family {odd:?} ({})", listed()));
    }
    let (mut theirs, mut gone) = (Vec::new(), Vec::new());
    for name in names {
        match port(name) {
            Port::Switched => route.here.push(name.clone()),
            // (only some of its models may be built here yet)
            Port::Ported if proving => route.here.push(name.clone()),
            Port::Ported if script(name) => {
                route.notes.push(in_blender(name));
                route.blender.push(name.clone());
            }
            Port::Ported => gone.push(name.as_str()),
            Port::Unported if proving => theirs.push(name.as_str()),
            Port::Unported => route.blender.push(name.clone()),
        }
    }
    if !theirs.is_empty() {
        return Err(format!(
            "{}: still built in Blender (--out and --check are for the families built here)",
            theirs.join(", ")
        ));
    }
    if !gone.is_empty() {
        return Err(format!("{}: {GONE} ({SWITCH})", gone.join(", ")));
    }
    Ok(route)
}

/// The centres of every model of `families`, by key: what `--centres` writes over the scripts' own.
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
    let at_once = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .max(1);
    let mut out = Vec::with_capacity(ids.len());
    for chunk in ids.chunks(at_once) {
        let built: Vec<Built> = std::thread::scope(|s| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|&i| {
                    let model = &family.models[i];
                    s.spawn(move || {
                        scene::set_finishes(finishes.clone());
                        let output = build_model(family, model);
                        (output, scene::take_log())
                    })
                })
                .collect();
            handles
                .into_iter()
                .zip(chunk)
                .map(|(h, &i)| {
                    let model = &family.models[i];
                    let (output, log) = match h.join() {
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
                    Built {
                        key: family.key(model),
                        id: model.id,
                        output,
                        log,
                    }
                })
                .collect()
        });
        out.extend(built);
    }
    out
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

/// Builds a ported `family` as `options` say. The keys it built (for the gate), or what failed.
pub fn run_family(repo: &Paths, family: &Family, options: &Options) -> Result<Vec<String>, String> {
    let finishes = finishes(repo)?;
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
    // A key the game wouldn't draw is a misspelt `key`: refused before any model is built or written.
    let known =
        build_keys(repo).map_err(|e| format!("cannot read the content or the manifests: {e}"))?;
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
    let built = build_all(family, &ids, &finishes);
    let mut failed = Vec::new();
    for b in &built {
        b.log.iter().for_each(|l| println!("{l}"));
        if let Err(why) = &b.output {
            println!("[gamekit] {}: {why}", b.id);
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
    // What the build hands the pack's writer, as the Blender builds hand it.
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
        return check(repo, family, &result, &files(&repo.assets), &known).map(|()| Vec::new());
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
    take_build(&paths, &result, options.no_pack, &known)?
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
            "[gamekit] {}: {} files, its pack and its centres as committed",
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

/// `cargo art …`: the exit code.
pub fn main(argv: &[String]) -> i32 {
    let has = |flag: &str| argv.iter().any(|a| a == flag);
    let value = |flag: &str| {
        argv.iter()
            .position(|a| a == flag)
            .map(|i| argv.get(i + 1).cloned())
    };
    let registry = registry();
    let ported: Vec<&Family> = registry.iter().filter_map(|(_, f)| f.as_ref()).collect();
    let repo = Paths::repo();
    if has("--centres") {
        // Blender's scripts first (it writes the file anew; none left, and it isn't run), then every ported
        // family's over them, written last: a switched family whose script stays as a library (plants.py) has
        // the centres its module gives, not those its script reports.
        let code = if ll_tools::art::families(&repo.here).is_empty() {
            0
        } else {
            ll_tools::art::main(argv)
        };
        if code == 0
            && !ported.is_empty()
            && let Err(err) = write_centres(&repo, &centres_of(&ported), false)
        {
            return exit(err);
        }
        return code;
    }
    if has("--leafcards") {
        return ll_tools::art::main(argv);
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
    let ports: Vec<(&str, Port)> = registry
        .iter()
        .map(|(n, f)| (*n, Port::of(f.as_ref())))
        .collect();
    let asked = Asked {
        all: has("--all"),
        out: out.is_some(),
        check: has("--check"),
    };
    let Route {
        here,
        blender,
        notes,
    } = match route(&names, &ports, &ll_tools::art::families(&repo.here), asked) {
        Ok(route) => route,
        Err(err) => return exit(err),
    };
    notes.iter().for_each(|l| println!("{l}"));
    let mut here: Vec<&Family> = here
        .iter()
        .filter_map(|name| {
            registry
                .iter()
                .find(|(n, _)| *n == name.as_str())?
                .1
                .as_ref()
        })
        .collect();
    let options = Options {
        only: only
            .as_ref()
            .map(|o| o.split(',').map(String::from).collect()),
        no_pack: has("--no-pack"),
        out,
        check: has("--check"),
    };
    let (mut keys, mut failed) = (Vec::new(), Vec::new());
    if has("--preview") {
        // Pictures only: the families built here are drawn (no file, pack or gate), Blender's render their own.
        if has("--lit") && !here.is_empty() {
            println!("{}", preview::LIT);
        }
        let into = std::path::PathBuf::from(ll_tools::checkglb::preview::folder(&repo));
        for family in here.drain(..) {
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
    }
    for family in here {
        match run_family(&repo, family, &options) {
            Ok(built) => keys.extend(built),
            Err(err) => {
                println!("{err}");
                failed.push(family.name.to_string());
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
    if !blender.is_empty() {
        let mut args = blender;
        args.extend(
            argv.iter()
                .filter(|a| ["--preview", "--lit", "--sheet", "--no-pack"].contains(&a.as_str()))
                .cloned(),
        );
        if let Some(only) = &only {
            args.extend(["--only".to_string(), only.clone()]);
        }
        if ll_tools::art::main(&args) != 0 {
            failed.push("the Blender builds".into());
        }
    }
    if !failed.is_empty() {
        return exit(format!("failed: {}", failed.join(", ")));
    }
    0
}
