//! What `cargo art` (crates/art) writes beside the models it builds: the keys a build may write (`build_keys`),
//! a family's pack (`write_pack`) and centres.json (`write_centres`), from what the build hands back
//! (`take_build`), which also checks that every clip a model's `motions` plays is one it has.

use crate::checkglb::py::{dumps_indent, join, normpath, relpath};
use crate::checkglb::{self, Paths};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

/// The JSON value a pack and a build's result are held in, and its reader.
pub use crate::checkglb::json::loads_text as loads;
pub use crate::checkglb::py::J;

/// The model keys a build may write: `known_keys` (the content's and the manifests'), and every pack's own
/// model entries, its pack listed in the base manifest's `packs` or not yet. Any other key is a misspelt
/// `key`, unless its `Model` says the game draws it by code (`.code()`).
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
                "{key}: no content object names this model and no manifest has it: a misspelt key? (a model the game draws by code takes .code())"
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

/// Whether each model of a family's build (`take_build`'s `result`) has a clip for every one its `motions` plays:
/// run before any of its files is written, so a failed build leaves none of them half new.
pub fn validate(result: &J) -> Result<(), String> {
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
    Ok(())
}

/// What a family's build hands back (its pack, comment, entries, centres, clips and the keys drawn by code): checks
/// it (`validate`), then writes centres.json and (unless `no_pack`) the pack. The lines to print, or what failed.
pub fn take_build(
    paths: &Paths,
    result: &J,
    no_pack: bool,
    known: &BTreeSet<String>,
) -> Result<Vec<String>, String> {
    validate(result)?;
    let none = J::Obj(Vec::new());
    let entries = result.get("entries").unwrap_or(&none);
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
        lines.push(format!("[art] wrote {}", relpath(&path, &paths.assets)));
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkglb::json::loads_text;

    #[test]
    fn every_committed_pack_reads_back_to_its_own_text() {
        // The packs as written (two-space indents, the perches a point a line): read and written again they must
        // come out byte for byte.
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
        std::fs::create_dir_all(&paths.art).unwrap();
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
        let comment = J::Str("Generated \u{d7} models".into());
        let path = write_pack(&paths, "fam", &entries, &comment, &BTreeSet::new(), &known).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "{\n  \"$comment\": \"Generated \\u00d7 models\",\n  \"version\": 1,\n  \"entries\": {\n    \"model.a\": {\n      \"type\": \"model\",\n      \"url\": \"../../models/f/a.glb\",\n      \"materials\": {\n        \"body\": \"gloss\"\n      },\n      \"scale\": 2\n    }\n  }\n}\n"
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
            ["[art] wrote packs/fam/manifest.json"]
        );
        assert_eq!(
            std::fs::read_to_string(&paths.centres).unwrap(),
            "{\n  \"model.fridge\": [0.0, -0.069]\n}\n"
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
