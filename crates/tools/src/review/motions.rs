//! Motions and sounds: which animation a use plays and what it sounds like.

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

/// The renderer's tables a use is weighed against (render/shared/characters/actions.ts, `Renderer::parse`).
#[derive(Debug, Clone, Default)]
pub struct Renderer {
    /// `ACTIONS`' tags (`action_keys`).
    pub actions: BTreeSet<String>,
    /// {tag: its `sit`} of `ACTIONS`, as TypeScript: a clip's name, or a function of the object's id (`sit_clips`).
    pub sits: BTreeMap<String, String>,
    /// {manner: its `sit`, if it has one} of `MANNERS` (content `how` names them; none in the source: empty).
    pub manners: BTreeMap<String, Option<String>>,
    /// `RIDES`' object ids: sat astride, the hands on its grips, whatever the tag's `sit` (residents.ts).
    pub rides: BTreeSet<String>,
    /// The source's `const NAME = new Set([…])` of strings, for a `sit` that asks `NAME.has(def)`.
    pub sets: BTreeMap<String, BTreeSet<String>>,
}

impl Renderer {
    /// The tables of actions.ts (`source`): `ACTIONS` must be there, `MANNERS` and `RIDES` may not.
    pub fn parse(source: &str) -> Result<Renderer, String> {
        let sit = |body: &str| field(body, "sit").map(String::from);
        let optional = |name| ts_block(source, name).map(ts_entries).unwrap_or_default();
        let literal = Regex::new(r"'([^']*)'").unwrap();
        let set =
            Regex::new(r"(?s)\bconst (\w+)(?::[^=]*)? = new Set(?:<[^>]*>)?\(\[(.*?)\]\)").unwrap();
        Ok(Renderer {
            actions: action_keys(source)?,
            sits: ts_entries(ts_block(source, "ACTIONS")?)
                .into_iter()
                .filter_map(|(k, body)| Some((k, sit(&body)?)))
                .collect(),
            manners: optional("MANNERS")
                .into_iter()
                .map(|(k, body)| (k, sit(&body)))
                .collect(),
            rides: optional("RIDES").into_iter().map(|(k, _)| k).collect(),
            sets: set
                .captures_iter(source)
                .map(|c| {
                    let names = literal.captures_iter(&c[2]).map(|l| l[1].to_string());
                    (c[1].to_string(), names.collect())
                })
                .collect(),
        })
    }
}

/// The top-level entries of a TypeScript object's body (`ts_block`): (key, its value's text), comment lines left out.
fn ts_entries(block: &str) -> Vec<(String, String)> {
    let key = Regex::new(r"^  ([A-Za-z_$][\w$]*|'[^']+'): (.*)$").unwrap();
    let mut out: Vec<(String, String)> = Vec::new();
    for line in block.lines().filter(|l| !l.trim_start().starts_with("//")) {
        match (key.captures(line), out.last_mut()) {
            (Some(c), _) => out.push((c[1].trim_matches('\'').to_string(), c[2].to_string())),
            (None, Some((_, body))) => {
                body.push('\n');
                body.push_str(line);
            }
            (None, None) => {}
        }
    }
    out
}

/// The value of `name` in an object literal's text (`{ stand: […], sit: 'sitHands' }`): up to the `,` or `}` that
/// ends it at the literal's own depth, strings skipped.
fn field<'a>(object: &'a str, name: &str) -> Option<&'a str> {
    let b = object.as_bytes();
    let (mut depth, mut quote, mut start) = (0i32, None::<u8>, None::<usize>);
    for (i, &c) in b.iter().enumerate() {
        if let Some(q) = quote {
            if c == q && b[i - 1] != b'\\' {
                quote = None;
            }
            continue;
        }
        match c {
            b'\'' | b'"' | b'`' => quote = Some(c),
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' | b',' if depth == 1 && start.is_some() => {
                return Some(object[start?..i].trim());
            }
            b'}' | b']' | b')' => depth -= 1,
            _ if depth == 1
                && start.is_none()
                && object[i..].starts_with(name)
                && object[i + name.len()..].starts_with(':')
                && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_')) =>
            {
                start = Some(i + name.len() + 1);
            }
            _ => {}
        }
    }
    None
}

/// The clips a `sit` (`Renderer::sits`, `manners`) plays at object `def`: a name; a function of `def` that is one
/// test (`def.includes('x')`, `def === 'x'`, `SET.has(def)`, or `!` one) choosing between two names; else (or a
/// set it can't read) every name it holds.
pub fn sit_clips(
    sit: &str,
    def: &str,
    sets: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let literal = Regex::new(r"'([^']*)'").unwrap();
    let arrow = Regex::new(r"^\(?\s*\w+(?:\s*:\s*\w+)?\s*\)?\s*=>\s*").unwrap();
    let test = Regex::new(r"^(!?)(?:def\.includes\('([^']*)'\)|def === '([^']*)'|(\w+)\.has\(def\))\s*\?\s*'([^']*)'\s*:\s*'([^']*)'$").unwrap();
    let body = arrow.replace(sit.trim(), "");
    let mut e = body.trim();
    if e.starts_with('(') && e.ends_with(')') {
        e = e[1..e.len() - 1].trim();
    }
    let every = || literal.captures_iter(e).map(|c| c[1].to_string()).collect();
    let Some(c) = test.captures(e) else {
        return every();
    };
    let truth = match (c.get(2), c.get(3), c.get(4)) {
        (Some(part), _, _) => Some(def.contains(part.as_str())),
        (_, Some(id), _) => Some(def == id.as_str()),
        (_, _, Some(set)) => sets.get(set.as_str()).map(|s| s.contains(def)),
        _ => None,
    };
    let negated = !c[1].is_empty();
    match truth.map(|t| t != negated) {
        Some(true) => [c[5].to_string()].into(),
        Some(false) => [c[6].to_string()].into(),
        None => every(),
    }
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
pub(super) fn sound_cell(sounds: &BTreeMap<String, String>, o: &Object, u: &Use) -> String {
    let mut cell = sound_of(sounds, o, u).unwrap_or_else(|| "—".into());
    if let Some(end) = &u.end_sound {
        let _ = write!(cell, ", then {end}");
    }
    if let Some(ambient) = &o.ambient {
        let _ = write!(cell, " (+ {ambient} by itself)");
    }
    cell
}
