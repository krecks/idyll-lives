//! Motions and sounds: which animation a use plays and what it sounds like.
//!
//! Split from `review.rs` (see `mod.rs`).

use super::*;

/// {model key: its entry's `motions`} of the manifests (web/public/assets/manifest.json and the packs it lists,
/// depth first, a later entry replacing an earlier one), for the entries that have one.
pub fn motions(paths: &Paths) -> Result<BTreeMap<String, Value>, String> {
    let mut out = BTreeMap::new();
    for (_, m) in checkglb::asset_manifests(paths)? {
        for (key, entry) in m
            .get("entries")
            .and_then(|e| e.entries())
            .unwrap_or_default()
        {
            if entry.get("type").and_then(|t| t.str()) == Some("model") {
                match entry.get("motions").filter(|m| m.entries().is_some()) {
                    Some(motions) => out.insert(key.clone(), motions.to_value()),
                    None => out.remove(key),
                };
            }
        }
    }
    Ok(out)
}

/// The body of `const <name>… = {` in TypeScript `source`, up to its closing `};` at the start of a line.
fn ts_block<'a>(source: &'a str, name: &str) -> Result<&'a str, String> {
    let head = Regex::new(&format!(r"(?m)^(?:export )?const {name}\b[^=]*= \{{\n")).unwrap();
    let start = head
        .find(source)
        .ok_or_else(|| format!("no `const {name} = {{` in the source"))?
        .end();
    let end = source[start..]
        .find("\n};")
        .ok_or_else(|| format!("`{name}` has no closing `}};`"))?;
    Ok(&source[start..start + end])
}

/// The action tags the renderer animates: the top-level keys of `ACTIONS` in render/shared/characters/actions.ts (`source`).
pub fn action_keys(source: &str) -> Result<BTreeSet<String>, String> {
    let key = Regex::new(r"(?m)^  ([A-Za-z_$][\w$]*|'[^']+'): ").unwrap();
    Ok(key
        .captures_iter(ts_block(source, "ACTIONS")?)
        .map(|c| c[1].trim_matches('\'').to_string())
        .collect())
}

/// {action tag: loop}: `ACTION_SOUNDS` in audio/world.ts (`source`).
pub fn action_sounds(source: &str) -> Result<BTreeMap<String, String>, String> {
    let entry = Regex::new(r"(?m)^  ([A-Za-z_$][\w$]*): '([^']+)'").unwrap();
    Ok(entry
        .captures_iter(ts_block(source, "ACTION_SOUNDS")?)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect())
}

/// What a use sounds like (audio/world.ts `soundOf`): its own `sound`, else the object's for its animation
/// (`sounds`), else by the animation (`sounds`: `ACTION_SOUNDS`), with washing by what is washed (written out
/// here, as there). A recipe's use sounds by its steps at this object (the rest are done elsewhere).
pub fn sound_of(sounds: &BTreeMap<String, String>, o: &Object, u: &Use) -> Option<String> {
    if let Some(steps) = &u.steps_here {
        let heard: BTreeSet<String> = steps
            .iter()
            .filter_map(|(anim, own)| match own {
                Some(own) => (own != "none").then(|| own.clone()),
                None => by_anim(sounds, o, anim, None),
            })
            .collect();
        return (!heard.is_empty()).then(|| heard.into_iter().collect::<Vec<_>>().join(", "));
    }
    by_anim(sounds, o, &u.anim, Some(u))
}

/// What `anim` sounds like at `o`, in its use `u` (None: a cooking step at it as a station).
fn by_anim(
    sounds: &BTreeMap<String, String>,
    o: &Object,
    anim: &str,
    u: Option<&Use>,
) -> Option<String> {
    if let Some(own) = u
        .and_then(|u| u.sound.as_ref())
        .or_else(|| o.sounds.get(anim))
    {
        return (own != "none").then(|| own.clone());
    }
    match (anim, u) {
        ("wash", None) => Some("washUp".into()),
        ("wash", Some(u)) if u.id == "washUp" => Some("washUp".into()),
        ("wash", Some(u)) => u.tags.iter().any(|t| t == "handwash").then(|| "tap".into()),
        (anim, _) => sounds.get(anim).cloned(),
    }
}

/// The Sound column's cell for a use: what it sounds like, what it plays as it ends, and what the object
/// sounds like by itself.
pub(crate) fn sound_cell(sounds: &BTreeMap<String, String>, o: &Object, u: &Use) -> String {
    let mut cell = sound_of(sounds, o, u).unwrap_or_else(|| "—".into());
    if let Some(end) = &u.end_sound {
        let _ = write!(cell, ", then {end}");
    }
    if let Some(ambient) = &o.ambient {
        let _ = write!(cell, " (+ {ambient} by itself)");
    }
    cell
}
