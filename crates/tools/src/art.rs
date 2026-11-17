//! `cargo art`: builds Blender families and checks what they wrote (was tools/art/blender/build.py).
//!
//!   cargo art kitchen                  build, export and register a family
//!   cargo art kitchen --only fridge    one model of it (--only a,b: several)
//!   cargo art --all                    every family
//!   cargo art garden_decor --preview   render previews/ instead of exporting
//!   cargo art garden_decor --preview --lit   and <id>.lit.png, lit and textured as the game draws it
//!   cargo art garden_decor --preview --sheet   and <id>.sheet.png: four sides, the baked shading, the clips along
//!   cargo art kitchen --no-pack        export without writing the pack
//!   cargo art --centres                write centres.json from every family (gamekit/centres.py, no build)
//!   cargo art --leafcards              render the trees' leaf-card atlas (leafcards.py: textures/leafcards/albedo.png)
//!
//! A family is a script in tools/art/blender/families/ (those starting with `_` are templates). Blender
//! comes from $BLENDER, the macOS app, or PATH; the models are made with Blender 5.2 LTS. After exporting
//! it runs the gate (`cargo check-glb`) on the keys it built and exits 1 if a build or the gate fails.
//!
//! The Python in Blender builds and exports; this side reads the content and the manifests and writes the
//! pack and centres.json (gamekit/runner.py). Before a build it hands Blender the keys a build may write
//! (`$GAMEKIT_KNOWN`, a JSON list: a misspelt `key=` is refused before anything is built); after it, it
//! reads what the build made (`$GAMEKIT_RESULT`: the pack, its entries, each model's clips and centre),
//! checks every clip `motions` plays is one the model has, and writes centres.json and the pack.
//!
//! `--centres` writes centres.json (where each model is centred at rest, for the gate) for every model
//! of every family, as a build writes it for the models it builds: Blender imports the family scripts
//! without building anything. Run it after changing a Model's `centre` without a rebuild (the gate then
//! says which files are off it), or after adding or removing models.
//!
//! `--leafcards` renders the one atlas every tree's leaf cards are cut from (the `leafcards` finish), from
//! leaves the script generates: run it after changing the script; the trees needn't be rebuilt for it.

use crate::checkglb::py::{J, dumps_indent, join, json_str, normpath, relpath};
use crate::checkglb::{self, Paths};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;

/// Blender: `$BLENDER`, the macOS app, or `blender` on PATH.
pub fn blender() -> Result<String, String> {
    let on_path = || -> Option<String> {
        std::env::split_paths(&std::env::var_os("PATH")?)
            .map(|d| d.join("blender"))
            .find(|p| is_executable(p))
            .map(|p| p.to_string_lossy().into_owned())
    };
    let env = std::env::var("BLENDER").ok().filter(|p| !p.is_empty());
    for path in [
        env,
        Some("/Applications/Blender.app/Contents/MacOS/Blender".into()),
        on_path(),
    ]
    .into_iter()
    .flatten()
    {
        if Path::new(&path).exists() {
            return Ok(path);
        }
    }
    Err("Blender not found: install Blender 5.2 LTS or set $BLENDER".into())
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata()
        .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// The family scripts (not the `_` templates), sorted.
pub fn families(here: &str) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(Path::new(here).join("families"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".py") && !n.starts_with('_') && !n.starts_with('.'))
        .map(|n| n.trim_end_matches(".py").to_string())
        .collect();
    out.sort();
    out
}

/// What a Blender run printed (newlines as Python's text mode reads them) and whether it succeeded.
pub struct Run {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Runs a script in Blender, in the background, with the factory settings (so the export settings stay
/// fixed) and an exit code on a Python error (else Blender exits 0 on one): `exe --background
/// --factory-startup --python-exit-code 1 --python <script> [-- <args>]`, with `env` set.
pub fn run(exe: &str, script: &str, args: &[String], env: &[(&str, &str)]) -> Run {
    let mut cmd = Command::new(exe);
    cmd.envs(env.iter().copied());
    cmd.args([
        "--background",
        "--factory-startup",
        "--python-exit-code",
        "1",
        "--python",
        script,
    ]);
    if !args.is_empty() {
        cmd.arg("--").args(args);
    }
    let text = |b: &[u8]| {
        String::from_utf8_lossy(b)
            .replace("\r\n", "\n")
            .replace('\r', "\n")
    };
    match cmd.output() {
        Ok(out) => Run {
            ok: out.status.success(),
            stdout: text(&out.stdout),
            stderr: text(&out.stderr),
        },
        Err(err) => Run {
            ok: false,
            stdout: String::new(),
            stderr: format!("{exe}: {err}"),
        },
    }
}

/// Python's `str.splitlines()`.
fn splitlines(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = s;
    while !rest.is_empty() {
        match rest.char_indices().find(|(_, c)| {
            matches!(
                c,
                '\n' | '\r'
                    | '\u{b}'
                    | '\u{c}'
                    | '\u{1c}'
                    | '\u{1d}'
                    | '\u{1e}'
                    | '\u{85}'
                    | '\u{2028}'
                    | '\u{2029}'
            )
        }) {
            Some((i, c)) => {
                out.push(&rest[..i]);
                let skip = if rest[i..].starts_with("\r\n") {
                    2
                } else {
                    c.len_utf8()
                };
                rest = &rest[i + skip..];
            }
            None => {
                out.push(rest);
                break;
            }
        }
    }
    out
}

/// Prints a run's `[gamekit]` lines and, when it failed, the Python traceback (Blender prints it to stdout;
/// else the last 20 lines) and the end of stderr. Whether it succeeded.
pub fn echo(run: &Run) -> bool {
    let out = splitlines(&run.stdout);
    for line in out.iter().filter(|l| l.starts_with("[gamekit]")) {
        println!("{line}");
    }
    if !run.ok {
        let start = out
            .iter()
            .position(|l| l.starts_with("Traceback"))
            .unwrap_or(out.len().saturating_sub(20));
        let chars: Vec<char> = run.stderr.chars().collect();
        let tail: String = chars[chars.len().saturating_sub(2000)..].iter().collect();
        println!("{}\n{tail}", out[start..].join("\n"));
    }
    run.ok
}

/// The model keys a build may write: `known_keys` (the content's and the manifests'), and every pack's own
/// model entries, its pack listed in the base manifest's `packs` or not yet. Any other key is a misspelt
/// `key=`, unless its `Model` says the game draws it by code (`code=True`).
pub fn build_keys(paths: &Paths) -> Result<BTreeSet<String>, String> {
    let mut known = checkglb::known_keys(paths)?;
    let listed: Vec<String> = checkglb::asset_manifests(paths)?
        .into_iter()
        .map(|(p, _)| normpath(&p))
        .collect();
    let mut packs: Vec<String> = std::fs::read_dir(join(&paths.assets, "packs"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| normpath(&join(&e.path().to_string_lossy(), "manifest.json")))
        .filter(|p| Path::new(p).exists() && !listed.contains(p))
        .collect();
    packs.sort();
    for pack in packs {
        for (_, m) in checkglb::manifests(paths, &pack)? {
            known.extend(model_keys(&m).map(|(k, _)| k.clone()));
        }
    }
    Ok(known)
}

/// A manifest's model entries.
fn model_keys(m: &J) -> impl Iterator<Item = &(String, J)> {
    m.get("entries")
        .filter(|e| e.truthy())
        .and_then(J::entries)
        .unwrap_or(&[])
        .iter()
        .filter(|(_, e)| e.get("type") == Some(&J::Str("model".into())))
}

/// An entry's `perches` with each point on one line, `[x, y, z]`, as centres.json writes its points: indent=2
/// puts every number on a line of its own (a tree's 20 perches, 80 lines).
pub fn points_on_a_line(text: &str) -> String {
    let perches = regex::Regex::new(r#""perches": \[(?:\s*\[[^\[\]]*\],?)*\s*\]"#).unwrap();
    let point = regex::Regex::new(r"\[\s+([^\[\]]*?)\s+\]").unwrap();
    let comma = regex::Regex::new(r",\s+").unwrap();
    perches
        .replace_all(text, |m: &regex::Captures| {
            point
                .replace_all(&m[0], |p: &regex::Captures| {
                    format!("[{}]", comma.replace_all(&p[1], ", "))
                })
                .into_owned()
        })
        .into_owned()
}

/// Writes the override pack `packs/<pack>/manifest.json` and lists it in the base manifest's `packs`; its path.
/// `entries` is [(key, fields)], e.g. {url, materials, motions, paint}. The game keeps the `vary` of the
/// entry it overrides (registry.ts `extended`); any other fields of that entry are copied, read from the
/// manifests now so the copy can't go stale. A key with no entry below is written as it is. Entries of the
/// pack that aren't given are kept. A key must be `known` (`build_keys`) or one of `code`.
pub fn write_pack(
    paths: &Paths,
    pack: &str,
    entries: &[(String, J)],
    comment: &J,
    code: &BTreeSet<String>,
    known: &BTreeSet<String>,
) -> Result<String, String> {
    let base_path = join(&paths.assets, "manifest.json");
    let path = join(&paths.assets, &format!("packs/{pack}/manifest.json"));
    let rel = relpath(&path, &paths.assets);
    let mut base = checkglb::read_json(&base_path)?;
    // The model entries of every manifest but this pack, later ones overriding (the game's order).
    let mut below: HashMap<String, J> = HashMap::new();
    for (p, m) in checkglb::manifests(paths, &base_path)? {
        if relpath(&p, &paths.assets) != rel {
            for (k, e) in model_keys(&m) {
                below.insert(k.clone(), e.clone());
            }
        }
    }
    let mut kept = match std::fs::read(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => J::Obj(Vec::new()),
        Err(e) => return Err(format!("{rel}: {e}")),
        Ok(data) => checkglb::json::loads_text(&data)
            .map_err(|e| format!("{rel}: {e}"))?
            .get("entries")
            .cloned()
            .ok_or_else(|| format!("{rel}: no `entries`"))?,
    };
    for (key, _) in entries {
        if !known.contains(key) && !code.contains(key) {
            return Err(format!(
                "{key}: no content object names this model and no manifest has it: a misspelt key=? (a model the game draws by code: Model(code=True))"
            ));
        }
    }
    const OWN: [&str; 7] = [
        "url",
        "far",
        "materials",
        "motions",
        "paint",
        "vary",
        "sink",
    ];
    for (key, fields) in entries {
        let mut entry = J::Obj(vec![("type".into(), J::Str("model".into()))]);
        for (k, v) in fields.entries().unwrap_or(&[]) {
            entry.set(k, v.clone());
        }
        for (k, v) in below.get(key).and_then(J::entries).unwrap_or(&[]) {
            if !entry.has(k) && !OWN.contains(&k.as_str()) && k != "centre" {
                entry.set(k, v.clone());
            }
        }
        kept.set(key, entry);
    }
    let manifest = J::Obj(vec![
        ("$comment".into(), comment.clone()),
        ("version".into(), J::Int(1)),
        ("entries".into(), kept),
    ]);
    std::fs::create_dir_all(Path::new(&path).parent().unwrap())
        .map_err(|e| format!("{rel}: {e}"))?;
    std::fs::write(&path, points_on_a_line(&dumps_indent(&manifest)) + "\n")
        .map_err(|e| format!("{rel}: {e}"))?;
    let listed = base
        .get("packs")
        .and_then(J::list)
        .is_some_and(|l| l.iter().any(|p| p.str() == Some(rel.as_str())));
    if !listed {
        if let Some(J::Arr(l)) = match &mut base {
            J::Obj(m) => m.iter_mut().find(|(k, _)| k == "packs").map(|(_, v)| v),
            _ => None,
        } {
            l.push(J::Str(rel.clone()));
        } else {
            base.set("packs", J::Arr(vec![J::Str(rel.clone())]));
        }
        std::fs::write(&base_path, dumps_indent(&base) + "\n")
            .map_err(|e| format!("manifest.json: {e}"))?;
    }
    Ok(path)
}

/// Writes `centres` over centres.json's entries for those keys, keeping the others (`replace`: dropping them).
pub fn write_centres(paths: &Paths, centres: &J, replace: bool) -> Result<(), String> {
    let point = |v: &J| -> Result<[f64; 2], String> {
        match v.list() {
            Some([x, d]) => match (x.num(), d.num()) {
                (Some(x), Some(d)) => Ok([x.f(), d.f()]),
                _ => Err(format!(
                    "a centre is two numbers, not {}",
                    checkglb::py::repr(v)
                )),
            },
            _ => Err(format!(
                "a centre is two numbers, not {}",
                checkglb::py::repr(v)
            )),
        }
    };
    let mut out: BTreeMap<String, [f64; 2]> = BTreeMap::new();
    if !replace {
        for (k, v) in checkglb::read_centres(&paths.centres)?
            .entries()
            .unwrap_or(&[])
        {
            out.insert(k.clone(), point(v)?);
        }
    }
    for (k, v) in centres.entries().unwrap_or(&[]) {
        out.insert(k.clone(), point(v)?);
    }
    std::fs::write(&paths.centres, checkglb::centres_text(&out))
        .map_err(|e| format!("centres.json: {e}"))
}

/// What a family's build handed back (`$GAMEKIT_RESULT`): checks each model's `motions` against its clips,
/// then writes centres.json and (unless `no_pack`) the pack. The lines to print, or what failed.
pub fn take_build(
    paths: &Paths,
    result: &J,
    no_pack: bool,
    known: &BTreeSet<String>,
) -> Result<Vec<String>, String> {
    let none = J::Obj(Vec::new());
    let entries = result.get("entries").unwrap_or(&none);
    let clips = result.get("clips").unwrap_or(&none);
    for (key, entry) in entries.entries().unwrap_or(&[]) {
        let Some(motions) = entry.get("motions").filter(|m| m.truthy()) else {
            continue;
        };
        let has: Vec<&str> = clips
            .get(key)
            .and_then(J::list)
            .unwrap_or(&[])
            .iter()
            .filter_map(J::str)
            .collect();
        let played = checkglb::played(motions).map_err(|e| format!("{key}: {e}"))?;
        let mut missing: Vec<String> = played
            .iter()
            .map(|(c, _)| c.key())
            .filter(|c| !has.contains(&c.as_str()))
            .collect();
        missing.sort();
        missing.dedup();
        if !missing.is_empty() {
            return Err(format!(
                "{key}: `motions` plays {}, which it has no clip for",
                missing.join(", ")
            ));
        }
    }
    let mut lines = Vec::new();
    if let Some(centres) = result.get("centres").filter(|c| c.truthy()) {
        write_centres(paths, centres, false)?;
    }
    let given = entries.entries().unwrap_or(&[]);
    if !given.is_empty() && !no_pack {
        let pack = result
            .get("pack")
            .and_then(J::str)
            .ok_or("the build named no pack")?;
        let code: BTreeSet<String> = result
            .get("code")
            .and_then(J::list)
            .unwrap_or(&[])
            .iter()
            .filter_map(|k| k.str().map(String::from))
            .collect();
        let comment = result.get("comment").cloned().unwrap_or(J::Null);
        let path = write_pack(paths, pack, given, &comment, &code, known)?;
        lines.push(format!("[gamekit] wrote {}", relpath(&path, &paths.assets)));
    }
    Ok(lines)
}

/// A file for this run's hand-overs, in a folder of its own, gone when the run is.
struct Scratch(std::path::PathBuf);

impl Scratch {
    fn new() -> Result<Scratch, String> {
        let dir = std::env::temp_dir().join(format!("ll-art-{}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        Ok(Scratch(dir))
    }

    fn file(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().into_owned()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// What Blender wrote to `path` (`$GAMEKIT_RESULT`), if it wrote anything; removed once read.
fn read_result(path: &str) -> Result<Option<J>, String> {
    if !Path::new(path).exists() {
        return Ok(None);
    }
    let r = checkglb::read_json(path).map_err(|e| format!("the build's result: {e}"));
    let _ = std::fs::remove_file(path);
    r.map(Some)
}

fn exit(msg: String) -> i32 {
    eprintln!("{msg}");
    1
}

/// `cargo art …`: the exit code.
pub fn main(argv: &[String]) -> i32 {
    let paths = Paths::repo();
    let here = paths.here.clone();
    let has = |flag: &str| argv.iter().any(|a| a == flag);
    let only = argv
        .iter()
        .position(|a| a == "--only")
        .map(|i| argv.get(i + 1).cloned());
    let scratch = match Scratch::new() {
        Ok(s) => s,
        Err(err) => return exit(err),
    };
    let result = scratch.file("result.json");
    if has("--centres") {
        // Every family imported in Blender, its models' centres collected (no build), centres.json written.
        let exe = match blender() {
            Ok(exe) => exe,
            Err(err) => return exit(err),
        };
        let script = format!("{here}/gamekit/centres.py");
        if !echo(&run(&exe, &script, &[], &[("GAMEKIT_RESULT", &result)])) {
            return 1;
        }
        let found = match read_result(&result) {
            Ok(Some(r)) => r,
            Ok(None) => return exit("centres.py handed back no centres".into()),
            Err(err) => return exit(err),
        };
        let none = J::Obj(Vec::new());
        let centres = found.get("centres").unwrap_or(&none);
        if let Err(err) = write_centres(&paths, centres, true) {
            return exit(err);
        }
        let all = centres.entries().unwrap_or(&[]);
        let off = all
            .iter()
            .filter(|(_, c)| {
                c.list()
                    .is_some_and(|l| l.iter().any(|v| v.num().is_some_and(|n| n.f() != 0.0)))
            })
            .count();
        let cwd = std::env::current_dir()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        println!(
            "[gamekit] wrote {}: {} models of {} families ({off} off the middle)",
            relpath(&paths.centres, &cwd),
            all.len(),
            found
                .get("families")
                .map(checkglb::py::repr)
                .unwrap_or_default()
        );
        return 0;
    }
    if has("--leafcards") {
        // The trees' leaf-card atlas, rendered from generated leaves (no family, no gate).
        let exe = match blender() {
            Ok(exe) => exe,
            Err(err) => return exit(err),
        };
        return if echo(&run(&exe, &format!("{here}/leafcards.py"), &[], &[])) {
            0
        } else {
            1
        };
    }
    let mut names: Vec<String> = argv
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();
    let only = match only {
        Some(Some(id)) => {
            if let Some(i) = names.iter().position(|n| *n == id) {
                names.remove(i);
            }
            Some(id)
        }
        Some(None) => return exit("--only names a model: --only <id>[,<id>…]".into()),
        None => None,
    };
    if has("--all") {
        names = families(&here);
    }
    if names.is_empty() {
        return exit(format!(
            "name a family ({}) or --all",
            families(&here).join(", ")
        ));
    }
    let mut passthrough: Vec<String> = ["--preview", "--lit", "--sheet", "--no-pack"]
        .iter()
        .filter(|f| has(f))
        .map(|f| f.to_string())
        .collect();
    if let Some(id) = only {
        passthrough.extend(["--only".to_string(), id]);
    }
    let exe = match blender() {
        Ok(exe) => exe,
        Err(err) => return exit(err),
    };
    // The keys a build may write, read once and handed to every family (not needed for previews).
    let preview = has("--preview");
    let known = if preview {
        BTreeSet::new()
    } else {
        match build_keys(&paths) {
            Ok(k) => k,
            Err(err) => return exit(format!("cannot read the content or the manifests: {err}")),
        }
    };
    let known_file = scratch.file("known.json");
    let list: Vec<String> = known.iter().map(|k| json_str(k)).collect();
    if let Err(err) = std::fs::write(&known_file, format!("[{}]", list.join(", "))) {
        return exit(format!("{known_file}: {err}"));
    }
    let env = [
        ("GAMEKIT_KNOWN", known_file.as_str()),
        ("GAMEKIT_RESULT", result.as_str()),
    ];
    let (mut built, mut failed) = (Vec::new(), Vec::new());
    let made = regex::Regex::new(r"(?m)^\[gamekit\] built (\S+)").unwrap();
    for name in &names {
        let script = format!("{here}/families/{name}.py");
        if !Path::new(&script).exists() {
            return exit(format!(
                "no family {} ({})",
                checkglb::py_repr_str(name),
                families(&here).join(", ")
            ));
        }
        let run = run(&exe, &script, &passthrough, &env);
        let ok = echo(&run);
        // (A failed build hands back nothing: it stopped before it got there.)
        let taken = match read_result(&result) {
            Ok(Some(r)) if ok && !preview => take_build(&paths, &r, has("--no-pack"), &known),
            Ok(_) => Ok(Vec::new()),
            Err(err) => Err(err),
        };
        match &taken {
            Ok(lines) => lines.iter().for_each(|l| println!("{l}")),
            Err(err) => println!("{name}: {err}"),
        }
        if !ok || taken.is_err() {
            failed.push(name.clone());
        }
        built.extend(made.captures_iter(&run.stdout).map(|c| c[1].to_string()));
    }
    if !built.is_empty() {
        // (The gate takes the keys among its arguments: model., item., tool.; none, and it checks every model.)
        let keys: Vec<String> = built
            .into_iter()
            .filter(|k| k.starts_with("model.") || k.starts_with("item.") || k.starts_with("tool."))
            .collect();
        let report = checkglb::gate(&paths, &keys);
        print!("{}", report.text);
        if report.failed > 0 {
            failed.push("check-glb".into());
        }
        println!(
            "Exports are not byte-for-byte repeatable: commit only the models you meant to change (git checkout the others)."
        );
    }
    if !failed.is_empty() {
        return exit(format!("failed: {}", failed.join(", ")));
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkglb::json::loads_text;

    #[test]
    fn every_committed_pack_reads_back_to_its_own_text() {
        // The packs were written by gamekit's write_pack (Python's json.dumps(indent=2) and the perches on a
        // line each): read and written again here they must come out byte for byte.
        let paths = Paths::repo();
        let dir = join(&paths.assets, "packs");
        let mut n = 0;
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = e.path().join("manifest.json");
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let read = loads_text(text.as_bytes()).unwrap();
            assert_eq!(
                points_on_a_line(&dumps_indent(&read)) + "\n",
                text,
                "{}",
                path.display()
            );
            n += 1;
        }
        assert!(n > 40, "{n} packs");
        let base = std::fs::read_to_string(join(&paths.assets, "manifest.json")).unwrap();
        assert_eq!(
            dumps_indent(&loads_text(base.as_bytes()).unwrap()) + "\n",
            base
        );
    }

    #[test]
    fn perches_go_one_point_a_line() {
        let j = loads_text(br#"{"a": {"perches": [[1, 2.5, -3], [0.0, 1e-05, 2]], "x": [1, 2]}}"#)
            .unwrap();
        assert_eq!(
            points_on_a_line(&dumps_indent(&j)),
            "{\n  \"a\": {\n    \"perches\": [\n      [1, 2.5, -3],\n      [0.0, 1e-05, 2]\n    ],\n    \"x\": [\n      1,\n      2\n    ]\n  }\n}"
        );
    }

    /// A checkout of its own under the temp folder: the base manifest listing `listed`, with model.a in it.
    fn scratch_assets(name: &str) -> (Paths, String) {
        let root = std::env::temp_dir()
            .join(format!("ll-art-test-{name}-{}", std::process::id()))
            .to_string_lossy()
            .into_owned();
        let _ = std::fs::remove_dir_all(&root);
        let paths = Paths::at(&root);
        std::fs::create_dir_all(join(&paths.assets, "packs/listed")).unwrap();
        std::fs::create_dir_all(&paths.here).unwrap();
        std::fs::write(
            join(&paths.assets, "manifest.json"),
            "{\n  \"version\": 1,\n  \"packs\": [\n    \"packs/listed/manifest.json\"\n  ],\n  \"entries\": {\n    \"model.a\": {\n      \"type\": \"model\",\n      \"url\": \"models/a.glb\",\n      \"vary\": {\"x\": 1},\n      \"scale\": 2,\n      \"centre\": [0, 1]\n    }\n  }\n}\n",
        )
        .unwrap();
        std::fs::write(
            join(&paths.assets, "packs/listed/manifest.json"),
            "{\n  \"entries\": {\n    \"model.b\": {\n      \"type\": \"model\",\n      \"url\": \"b.glb\"\n    }\n  }\n}\n",
        )
        .unwrap();
        (paths, root)
    }

    #[test]
    fn a_pack_copies_the_entry_below_but_its_own_fields_and_is_listed() {
        let (paths, root) = scratch_assets("pack");
        let known: BTreeSet<String> = ["model.a".to_string()].into();
        let entries = vec![(
            "model.a".to_string(),
            loads_text(br#"{"url": "../../models/f/a.glb", "materials": {"body": "gloss"}}"#)
                .unwrap(),
        )];
        let comment = J::Str("Blender-built \u{d7} models".into());
        let path = write_pack(&paths, "fam", &entries, &comment, &BTreeSet::new(), &known).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "{\n  \"$comment\": \"Blender-built \\u00d7 models\",\n  \"version\": 1,\n  \"entries\": {\n    \"model.a\": {\n      \"type\": \"model\",\n      \"url\": \"../../models/f/a.glb\",\n      \"materials\": {\n        \"body\": \"gloss\"\n      },\n      \"scale\": 2\n    }\n  }\n}\n"
        );
        let base = std::fs::read_to_string(join(&paths.assets, "manifest.json")).unwrap();
        assert!(
            base.contains("\"packs/listed/manifest.json\",\n    \"packs/fam/manifest.json\"\n  ]"),
            "{base}"
        );
        // Written again: the same pack, listed once.
        write_pack(&paths, "fam", &entries, &comment, &BTreeSet::new(), &known).unwrap();
        let again = std::fs::read_to_string(join(&paths.assets, "manifest.json")).unwrap();
        assert_eq!(again, base);
        // An unknown key is refused, unless the game draws it by code.
        let odd = vec![("model.typo".to_string(), J::Obj(Vec::new()))];
        assert!(write_pack(&paths, "fam", &odd, &comment, &BTreeSet::new(), &known).is_err());
        let code: BTreeSet<String> = ["model.typo".to_string()].into();
        write_pack(&paths, "fam", &odd, &comment, &code, &known).unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_build_knows_an_unlisted_packs_keys_and_its_clips_are_checked() {
        let (paths, root) = scratch_assets("take");
        std::fs::create_dir_all(&paths.content).unwrap();
        std::fs::write(
            join(&paths.content, "base.json"),
            "{\"objects\": [{\"id\": \"fridge\"}]}",
        )
        .unwrap();
        std::fs::create_dir_all(join(&paths.assets, "packs/unlisted")).unwrap();
        std::fs::write(
            join(&paths.assets, "packs/unlisted/manifest.json"),
            "{\"entries\": {\"model.c\": {\"type\": \"model\"}}}",
        )
        .unwrap();
        let known = build_keys(&paths).unwrap();
        for k in ["model.a", "model.b", "model.c", "model.fridge"] {
            assert!(known.contains(k), "{k}: {known:?}");
        }
        let result = loads_text(
            br#"{"pack": "fam", "comment": "c", "code": [], "entries": {"model.fridge": {"url": "f.glb", "motions": {"uses": {"fetch": "fridge_open"}, "idle": "hum"}}}, "centres": {"model.fridge": [0, -0.06896]}, "clips": {"model.fridge": ["fridge_open"]}}"#,
        )
        .unwrap();
        let err = take_build(&paths, &result, false, &known).unwrap_err();
        assert_eq!(
            err,
            "model.fridge: `motions` plays hum, which it has no clip for"
        );
        let result = loads_text(
            br#"{"pack": "fam", "comment": "c", "code": [], "entries": {"model.fridge": {"url": "f.glb", "motions": {"uses": {"fetch": "fridge_open"}}}}, "centres": {"model.fridge": [0, -0.06896]}, "clips": {"model.fridge": ["fridge_open"]}}"#,
        )
        .unwrap();
        assert_eq!(
            take_build(&paths, &result, false, &known).unwrap(),
            ["[gamekit] wrote packs/fam/manifest.json"]
        );
        assert_eq!(
            std::fs::read_to_string(&paths.centres).unwrap(),
            "{\n  \"model.fridge\": [0.0, -0.069]\n}\n"
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
