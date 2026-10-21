//! `cargo review sizes|uses [--md <dir>]`: the authenticity review's first pass over every buyable object
//! (plan/authenticity-review.md §5.1, T3 and T4).
//!
//!   cargo review sizes [--md <dir>]   each object's W x H x D and its seat, top and mattress heights against
//!                                     docs/review/reference.json: the outliers per category (`--md`: every
//!                                     object, `<dir>/sizes-<cat>.md`)
//!   cargo review uses [--md <dir>]    each object's uses, poses, minutes, effects, skills, slots, approach,
//!                                     motions and sounds, and what looks wrong there: a summary per category
//!                                     (`--md`: every object, `<dir>/uses-<cat>.md`)
//!
//! The content is read as the game reads it (sim-core merges base.json and its includes, so `default_anim` and
//! the defaults apply), and raw for what sim-core keeps no copy of (`model`, `category`, `group`, a use's `when`,
//! `leaves` and its tags in order). Sizes and heights are `cargo check-glb`'s (`checkglb::measures`), with what
//! the review measures besides: a top its kind has and its content hasn't, a table's H without its seats and a
//! stair's top step (`h_of`; docs/review/reference.md, "How to use this"). The manifests give each model's
//! `motions`; the renderer's `ACTIONS` (characters.ts) and action sounds (audio/world.ts) are read from its
//! source. A category is the docs/objects page the object is on: its buy category, or `packs` for a personality
//! pack's objects.

use crate::checkglb::{self, Height, Measure, Paths};
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The reference table, from the checkout's root.
pub const REFERENCE: &str = "docs/review/reference.json";
/// How far (×) from the median `minutes` of uses with the same animation a use's duration may be before it is
/// flagged (owner decision: today's short durations are by design, only outliers are).
pub const DURATION_SPREAD: f32 = 3.0;

fn ten_percent() -> f64 {
    0.10
}

/// The reference table (docs/review/reference.json): real-world ranges per kind of thing, and which kind each
/// object is.
#[derive(Debug, Deserialize)]
pub struct Reference {
    /// The share overall W, H and D may stray past their range (0.10: ±10 %); functional heights are strict.
    #[serde(default = "ten_percent")]
    pub tolerance: f64,
    #[serde(default)]
    pub sources: BTreeMap<String, Value>,
    #[serde(default)]
    pub kinds: BTreeMap<String, Kind>,
    /// {content group: kind}; also "<category>/<group>", which wins (two categories may have a `seating`).
    #[serde(default)]
    pub groups: BTreeMap<String, String>,
    /// {object id or model key: kind}: wins over `groups`.
    #[serde(default)]
    pub ids: BTreeMap<String, String>,
    /// Kinds where no use is expected (decor stays decor).
    #[serde(default)]
    pub decor: Vec<String>,
}

/// A kind of thing: its real-world ranges (m; each optional), uses, minutes per use and mechanisms' angles.
#[derive(Debug, Default, Deserialize)]
pub struct Kind {
    #[serde(default)]
    pub label: String,
    pub w: Option<[f64; 2]>,
    pub h: Option<[f64; 2]>,
    pub d: Option<[f64; 2]>,
    pub seat: Option<[f64; 2]>,
    pub surface: Option<[f64; 2]>,
    pub mattress: Option<[f64; 2]>,
    #[serde(default)]
    pub uses: Vec<String>,
    #[serde(default)]
    pub minutes: BTreeMap<String, [f64; 2]>,
    #[serde(default)]
    pub mechanisms: BTreeMap<String, [f64; 2]>,
    #[serde(default)]
    pub sources: Vec<String>,
}

impl Reference {
    pub fn parse(text: &str) -> Result<Reference, String> {
        serde_json::from_str(text).map_err(|e| format!("{REFERENCE}: {e}"))
    }

    /// The kind an object is: `ids` by its id, then by its model key; else `groups` by "<category>/<group>",
    /// then by its group. None: none (an error for `sizes`). It may name a kind `kinds` doesn't have (`kind`).
    pub fn kind_of(&self, id: &str, model: &str, category: &str, group: &str) -> Option<&str> {
        let by_group = || if group.is_empty() { None } else { self.groups.get(&format!("{category}/{group}")).or_else(|| self.groups.get(group)) };
        self.ids.get(id).or_else(|| self.ids.get(model)).or_else(by_group).map(String::as_str)
    }

    /// The kind `name` names.
    pub fn kind(&self, name: &str) -> Option<&Kind> {
        self.kinds.get(name)
    }
}

/// Whether `v` lies in `[lo, hi]` widened by `tolerance` (a share of each end: 0.10 lets it be 10 % under the low
/// end or over the high one; 0 is strict), give or take half a millimetre (the models' numbers are f32).
pub fn within(v: f64, [lo, hi]: [f64; 2], tolerance: f64) -> bool {
    const SLACK: f64 = 0.0005;
    v >= lo * (1.0 - tolerance) - SLACK && v <= hi * (1.0 + tolerance) + SLACK
}

/// What `within` found wrong, as a table cell, to the millimetre (to the centimetre, 1.2104 over 1.21 read "1.21 >
/// 1.21"): `H 1.215 > 1.100 (0.78–1.00 ±10 %)`, `top 0.766 > 0.760 (0.71–0.76)`.
fn outside(name: &str, v: f64, [lo, hi]: [f64; 2], tolerance: f64) -> String {
    let (l, h) = (lo * (1.0 - tolerance), hi * (1.0 + tolerance));
    let side = if v < l { format!("{v:.3} < {l:.3}") } else { format!("{v:.3} > {h:.3}") };
    let allowance = if tolerance > 0.0 { format!(" ±{:.0} %", tolerance * 100.0) } else { String::new() };
    format!("{name} {side} ({lo:.2}–{hi:.2}{allowance})")
}

/// A buyable object as the review sees it.
#[derive(Debug, Clone, Default)]
pub struct Object {
    pub id: String,
    pub name: String,
    /// The docs/objects page it is on (`<page>.md`): its buy category; `packs` for a personality pack's objects.
    pub page: String,
    pub category: String,
    pub group: String,
    /// The model it draws: content `model`, else `model.<id>` (as the renderer picks it).
    pub model: String,
    pub slots: u8,
    /// Where it is used from: `front`, or `around` (any free tile beside it).
    pub approach: &'static str,
    pub uses: Vec<Use>,
    /// It gives light (content `light`): switching it on and off is a use of its own, by the player and residents.
    pub lamp: bool,
    /// The animation tags cooking plays at it as a station (content `stations`): its recipe steps', and washing
    /// up, scraping plates and putting things away (`cooking.dishesAnim`, `scrapeAnim`, `putAwayAnim`).
    pub station_anims: BTreeSet<String>,
}

/// One of an object's interactions.
#[derive(Debug, Clone, Default)]
pub struct Use {
    pub id: String,
    /// Its animation tag: content `anim`, else sim-core's `default_anim` ("": none).
    pub anim: String,
    /// `stand`, `sit` or `lie`.
    pub pose: &'static str,
    pub minutes: f32,
    /// What it gives over its whole length, per need (sim-core `total_gain`), and the skills it trains (per hour).
    pub effects: Vec<(String, f32)>,
    pub skills: Vec<(String, f32)>,
    /// Its tags, in the content's order.
    pub tags: Vec<String>,
    /// Content `when` and `leaves` (`open`, `closed`; `ripe`, `picked` for crops).
    pub when: Option<String>,
    pub leaves: Option<String>,
}

/// The buyable objects of the content in `dir` (web/public/content), in the content's order.
pub fn objects(dir: &Path) -> Result<Vec<Object>, String> {
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).map_err(|e| format!("{}: {e}", dir.join(name).display()));
    let base = read("base.json")?;
    let includes: Vec<String> = serde_json::from_str::<Value>(&base).ok().and_then(|v| v.get("include").cloned()).and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default();
    let parts = includes.iter().map(|n| read(n)).collect::<Result<Vec<_>, _>>()?;
    let merged = sim_core::merge_content(&base, &parts.iter().map(String::as_str).collect::<Vec<_>>()).map_err(|e| e.to_string())?;
    let content = sim_core::Content::from_json(&merged).map_err(|e| e.to_string())?;
    let merged: Value = serde_json::from_str(&merged).map_err(|e| e.to_string())?;
    let raw: HashMap<&str, &Value> = merged["objects"].as_array().into_iter().flatten().filter_map(|o| Some((o.get("id")?.as_str()?, o))).collect();
    // A personality pack's objects are on docs/objects/packs.md, whatever their category.
    let mut packed = HashSet::new();
    for (name, text) in includes.iter().zip(&parts) {
        if name.starts_with("packs/") {
            let file: Value = serde_json::from_str(text).map_err(|e| format!("{name}: {e}"))?;
            packed.extend(file["objects"].as_array().into_iter().flatten().filter_map(|o| o["id"].as_str().map(String::from)));
        }
    }
    let text = |v: Option<&Value>| v.and_then(Value::as_str).unwrap_or_default().to_string();
    let strs = |v: &Value| -> Vec<String> { v.as_array().into_iter().flatten().filter_map(|s| s.as_str().map(String::from)).collect() };
    // {station: the animation tags cooking plays there}.
    let mut at_station: HashMap<String, BTreeSet<String>> = HashMap::new();
    let mut plays = |stations: Vec<String>, anim: &Value| {
        if let Some(anim) = anim.as_str() {
            for s in stations {
                at_station.entry(s).or_default().insert(anim.to_string());
            }
        }
    };
    let cooking = &merged["cooking"];
    plays(vec!["dishes".into()], &cooking["dishesAnim"]);
    plays(vec!["bin".into()], &cooking["scrapeAnim"]);
    plays(strs(&cooking["putAwayAt"]), &cooking["putAwayAnim"]);
    for step in merged["recipes"].as_array().into_iter().flatten().flat_map(|r| r["steps"].as_array().into_iter().flatten()) {
        plays(strs(&step["at"]), &step["anim"]);
    }
    let mut out = Vec::new();
    for def in content.objects.iter().filter(|d| d.price.is_some()) {
        let none = Value::Null;
        let raw = raw.get(def.id.as_str()).copied().unwrap_or(&none);
        let category = text(raw.get("category"));
        let raw_uses = raw["interactions"].as_array().map(Vec::as_slice).unwrap_or(&[]);
        let uses = def
            .interactions
            .iter()
            .map(|it| {
                let r = raw_uses.iter().find(|r| r["id"].as_str() == Some(&it.id)).unwrap_or(&none);
                let named = |gains: &[f32], names: &mut dyn Iterator<Item = String>| -> Vec<(String, f32)> { names.zip(gains).filter(|(_, g)| **g != 0.0).map(|(n, g)| (n, *g)).collect() };
                Use {
                    id: it.id.clone(),
                    anim: it.anim.and_then(|a| content.animations.get(a)).cloned().unwrap_or_default(),
                    pose: match it.pose {
                        sim_core::content::Pose::Stand => "stand",
                        sim_core::content::Pose::Sit => "sit",
                        sim_core::content::Pose::Lie => "lie",
                    },
                    minutes: it.minutes,
                    effects: named(&it.total_gain, &mut content.needs.iter().map(|n| n.id.clone())),
                    skills: named(&it.skill_gain, &mut content.skills.iter().map(|s| s.id.clone())),
                    tags: r["tags"].as_array().into_iter().flatten().filter_map(|t| t.as_str().map(String::from)).collect(),
                    when: r["when"].as_str().map(String::from),
                    leaves: r["leaves"].as_str().map(String::from),
                }
            })
            .collect();
        out.push(Object {
            id: def.id.clone(),
            name: def.name.clone(),
            page: if packed.contains(&def.id) { "packs".into() } else { category.clone() },
            category,
            group: text(raw.get("group")),
            model: raw["model"].as_str().map(String::from).unwrap_or_else(|| format!("model.{}", def.id)),
            slots: def.slots,
            approach: match def.approach {
                sim_core::content::Approach::Front => "front",
                sim_core::content::Approach::Around => "around",
            },
            uses,
            lamp: raw.get("light").is_some_and(|l| !l.is_null()),
            station_anims: strs(&raw["stations"]).iter().flat_map(|s| at_station.get(s).into_iter().flatten().cloned()).collect(),
        });
    }
    Ok(out)
}

/// {model key: its entry's `motions`} of the manifests (web/public/assets/manifest.json and the packs it lists,
/// depth first, a later entry replacing an earlier one), for the entries that have one.
pub fn motions(assets: &Path) -> Result<BTreeMap<String, Value>, String> {
    fn stack(path: &Path, out: &mut BTreeMap<String, Value>, depth: usize) -> Result<(), String> {
        if depth > 20 {
            return Err(format!("{}: packs nested too deep", path.display()));
        }
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let m: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        for (key, entry) in m["entries"].as_object().into_iter().flatten() {
            if entry["type"] == "model" {
                match entry.get("motions").filter(|m| m.is_object()) {
                    Some(motions) => out.insert(key.clone(), motions.clone()),
                    None => out.remove(key),
                };
            }
        }
        for pack in m["packs"].as_array().into_iter().flatten().filter_map(Value::as_str) {
            stack(&path.parent().unwrap_or(Path::new(".")).join(pack), out, depth + 1)?;
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    stack(&assets.join("manifest.json"), &mut out, 0)?;
    Ok(out)
}

/// The body of `const <name>… = {` in TypeScript `source`, up to its closing `};` at the start of a line.
fn ts_block<'a>(source: &'a str, name: &str) -> Result<&'a str, String> {
    let head = Regex::new(&format!(r"(?m)^const {name}\b[^=]*= \{{\n")).unwrap();
    let start = head.find(source).ok_or_else(|| format!("no `const {name} = {{` in the source"))?.end();
    let end = source[start..].find("\n};").ok_or_else(|| format!("`{name}` has no closing `}};`"))?;
    Ok(&source[start..start + end])
}

/// The action tags the renderer animates: the top-level keys of `ACTIONS` in characters.ts (`source`).
pub fn action_keys(source: &str) -> Result<BTreeSet<String>, String> {
    let key = Regex::new(r"(?m)^  ([A-Za-z_$][\w$]*|'[^']+'): ").unwrap();
    Ok(key.captures_iter(ts_block(source, "ACTIONS")?).map(|c| c[1].trim_matches('\'').to_string()).collect())
}

/// {action tag: loop}: `ACTION_SOUNDS` in audio/world.ts (`source`).
pub fn action_sounds(source: &str) -> Result<BTreeMap<String, String>, String> {
    let entry = Regex::new(r"(?m)^  ([A-Za-z_$][\w$]*): '([^']+)'").unwrap();
    Ok(entry.captures_iter(ts_block(source, "ACTION_SOUNDS")?).map(|c| (c[1].to_string(), c[2].to_string())).collect())
}

/// What a use sounds like (audio/world.ts `soundOf`): by its animation, and for washing and cleaning by what
/// is washed or cleaned (those two cases are written out here, as there).
pub fn sound_of(sounds: &BTreeMap<String, String>, object: &str, u: &Use) -> Option<String> {
    match u.anim.as_str() {
        "wash" if u.id == "washUp" => Some("washUp".into()),
        "wash" => u.tags.iter().any(|t| t == "handwash").then(|| "tap".into()),
        "clean" => (u.id == "laundry" && Regex::new(r"(?i)washer|laundryCent").unwrap().is_match(object)).then(|| "washer".into()),
        anim => sounds.get(anim).cloned(),
    }
}

/// What looks wrong in an object's uses (`cargo review uses`).
#[derive(Debug, Clone, PartialEq)]
pub enum Flag {
    /// No use at all, it isn't a lamp, and its kind isn't decor.
    NoUse,
    /// A use's animation that `ACTIONS` has no entry for (the resident stands idle at it).
    NoAction { using: String, anim: String },
    /// A `motions.uses` or `motions.loops` key that is neither a use's id nor an animation tag of the object.
    MotionKey { key: String },
    /// A use that opens or closes the object (`when`, `leaves`) on a model without `motions.open`.
    NoOpen { using: String },
    /// A use's minutes more than `DURATION_SPREAD` times from the median of the uses like it (`like_key`).
    Duration { using: String, minutes: f32, like: String, median: f32 },
}

impl Flag {
    /// The column it is counted in.
    pub fn column(&self) -> &'static str {
        match self {
            Flag::NoUse => "no use",
            Flag::NoAction { .. } => "anim not in ACTIONS",
            Flag::MotionKey { .. } => "motions key",
            Flag::NoOpen { .. } => "opens, no motions.open",
            Flag::Duration { .. } => "duration",
        }
    }

    pub fn text(&self) -> String {
        match self {
            Flag::NoUse => "no use".into(),
            Flag::NoAction { using, anim } => format!("{using}: anim `{anim}` has no `ACTIONS` entry"),
            Flag::MotionKey { key } => format!("`motions` key `{key}` is no use or anim of it"),
            Flag::NoOpen { using } => format!("{using} opens or closes it, but its model has no `motions.open`"),
            Flag::Duration { using, minutes, like, median } => format!("{using}: {minutes} min, the median for `{like}` {median} min"),
        }
    }
}

/// The columns of `Flag::column`, in order.
pub const FLAG_COLUMNS: [&str; 5] = ["no use", "anim not in ACTIONS", "motions key", "opens, no motions.open", "duration"];

/// What a use's duration is compared with: the uses with the same animation (a night's sleep with sleeps, a nap
/// with naps; a first tag lumps them together), else with the same first tag.
pub fn like_key(u: &Use) -> Option<&str> {
    if u.anim.is_empty() { u.tags.first().map(String::as_str) } else { Some(u.anim.as_str()) }
}

/// The median minutes of the uses of each `like_key`, over `objects`.
pub fn tag_medians(objects: &[Object]) -> BTreeMap<String, f32> {
    let mut by_like: BTreeMap<String, Vec<f32>> = BTreeMap::new();
    for u in objects.iter().flat_map(|o| &o.uses) {
        if let Some(like) = like_key(u) {
            by_like.entry(like.to_string()).or_default().push(u.minutes);
        }
    }
    by_like
        .into_iter()
        .map(|(like, mut m)| {
            m.sort_by(f32::total_cmp);
            let n = m.len();
            (like, if n % 2 == 1 { m[n / 2] } else { (m[n / 2 - 1] + m[n / 2]) / 2.0 })
        })
        .collect()
}

/// What looks wrong in `object`'s uses: `motions` is its model's (None: none), `actions` the renderer's action
/// tags, `medians` the minutes per `like_key` (`tag_medians`), and `decor` whether its kind expects no use.
pub fn use_flags(object: &Object, motions: Option<&Value>, actions: &BTreeSet<String>, medians: &BTreeMap<String, f32>, decor: bool) -> Vec<Flag> {
    let mut out = Vec::new();
    if object.uses.is_empty() && !object.lamp && !decor {
        out.push(Flag::NoUse);
    }
    for u in &object.uses {
        if !u.anim.is_empty() && !actions.contains(&u.anim) {
            out.push(Flag::NoAction { using: u.id.clone(), anim: u.anim.clone() });
        }
    }
    let named: BTreeSet<&str> = object.uses.iter().flat_map(|u| [u.id.as_str(), u.anim.as_str()]).chain(object.station_anims.iter().map(String::as_str)).filter(|k| !k.is_empty()).collect();
    for part in ["uses", "loops"] {
        for key in motions.and_then(|m| m[part].as_object()).into_iter().flat_map(|m| m.keys()) {
            if !named.contains(key.as_str()) {
                out.push(Flag::MotionKey { key: key.clone() });
            }
        }
    }
    let opens = motions.is_some_and(|m| m.get("open").is_some_and(|o| !o.is_null()));
    for u in &object.uses {
        let open_or_closed = |s: &Option<String>| matches!(s.as_deref(), Some("open" | "closed"));
        if (open_or_closed(&u.when) || open_or_closed(&u.leaves)) && !opens {
            out.push(Flag::NoOpen { using: u.id.clone() });
        }
    }
    for u in &object.uses {
        let Some(like) = like_key(u) else { continue };
        let Some(&median) = medians.get(like) else { continue };
        if median > 0.0 && (u.minutes > median * DURATION_SPREAD || u.minutes * DURATION_SPREAD < median) {
            out.push(Flag::Duration { using: u.id.clone(), minutes: u.minutes, like: like.to_string(), median });
        }
    }
    out
}

/// One object against the reference (`cargo review sizes`).
#[derive(Debug, Default)]
pub struct SizeRow {
    pub id: String,
    pub name: String,
    pub page: String,
    pub kind: Option<String>,
    /// W x H x D (m, game axes: across its front, up, front to back).
    pub size: Option<[f64; 3]>,
    /// The seat, top and mattress the game uses and what its model has there (m; the mattress at slot 0).
    pub seat: Option<(f64, Option<f64>)>,
    pub surface: Option<(f64, Option<f64>)>,
    pub mattress: Option<(f64, Option<f64>)>,
    /// Its model's top where the content has no `surface` and its kind has one (`checkglb::Measure::top`): within
    /// `TOP_NEAR` of the kind's range, else None (and the nearest is in `outside`).
    pub top: Option<f64>,
    /// The H compared where it isn't the whole model's, and what it is (`h_of`).
    pub h: Option<(&'static str, f64)>,
    /// A stair's steps: how many, and the rise of each (m).
    pub steps: Option<(usize, f64)>,
    /// What is outside its kind's ranges.
    pub outside: Vec<String>,
    /// Why it couldn't be checked (no kind, a kind `kinds` doesn't have, an unreadable model).
    pub problem: Option<String>,
}

impl SizeRow {
    pub fn status(&self) -> String {
        match (&self.problem, self.outside.is_empty()) {
            (Some(p), _) => p.clone(),
            (None, true) => "ok".into(),
            (None, false) => format!("out: {}", self.outside.join("; ")),
        }
    }
}

/// The height `what` of `heights` (the first of `whats`): the game's and the model's.
fn height_of(heights: &[Height], whats: &[&str]) -> Option<(f64, Option<f64>)> {
    heights.iter().find(|h| whats.contains(&h.what)).map(|h| (h.expected, h.mesh))
}

/// How far from its kind's top range (a share of each end, as `within`'s tolerance) a top found where the content
/// has none may be and still count as its top: a face looking up any farther is something else (a shelf, a seat).
pub const TOP_NEAR: f64 = 0.20;

/// The H to compare where the model's whole height isn't the kind's (`kind`): a stair's top step (the storey it
/// climbs, which its kind gives, not the handrail over it); for a kind with a top and no seat (a table or a desk
/// alone: a set's kind, with a seat, has the set's ranges), one sat at whose model carries its seats, the table
/// alone (`checkglb::Measure::table`: not its chairs' backs).
fn h_of(m: &Measure, kind: &Kind) -> Option<(&'static str, f64)> {
    match (m.steps, m.table) {
        (Some((top, _)), _) => Some(("top step", top)),
        (None, Some(table)) if kind.surface.is_some() && kind.seat.is_none() => Some(("table", table)),
        _ => None,
    }
}

/// `object` (its id, model, category and group) measured (`measure`), against its kind in `reference`. The
/// functional heights are the model's where it has one there (else the game's), strictly in range; the top,
/// where the content has none, the model's top near the kind's range (`TOP_NEAR`).
pub fn size_row(object: &Object, measure: Option<&Result<Measure, String>>, reference: &Reference) -> SizeRow {
    let mut row = SizeRow { id: object.id.clone(), name: object.name.clone(), page: object.page.clone(), ..SizeRow::default() };
    let measured = measure.and_then(|m| m.as_ref().ok());
    match measure {
        Some(Ok(m)) => {
            row.size = Some(m.size);
            row.seat = height_of(&m.heights, &["seat"]);
            row.surface = height_of(&m.heights, &["surface"]);
            row.mattress = height_of(&m.heights, &["mattress", "lower bunk"]);
            row.steps = m.steps.map(|(top, n)| (n, top / n as f64));
        }
        Some(Err(e)) => row.problem = Some(format!("unmeasured: {e}")),
        None => row.problem = Some("unmeasured: no model".into()),
    }
    let Some(kind) = reference.kind_of(&object.id, &object.model, &object.category, &object.group) else {
        row.problem = Some("no kind".into());
        return row;
    };
    row.kind = Some(kind.to_string());
    let Some(k) = reference.kind(kind) else {
        row.problem = Some(format!("kind `{kind}` isn't in `kinds`"));
        return row;
    };
    row.h = measured.and_then(|m| h_of(m, k));
    if let Some(size) = row.size {
        let (h_name, h) = match row.h {
            Some((by, h)) => (format!("H ({by})"), h),
            None => ("H".into(), size[1]),
        };
        for (name, v, range) in [("W", size[0], k.w), (h_name.as_str(), h, k.h), ("D", size[2], k.d)] {
            if let Some(range) = range.filter(|r| !within(v, *r, reference.tolerance)) {
                row.outside.push(outside(name, v, range, reference.tolerance));
            }
        }
    }
    if row.problem.is_none() {
        // A top the content hasn't: the model's, if near enough the range to be one.
        let found_top = measured.and_then(|m| m.top).filter(|_| row.surface.is_none()).map(|(_, h)| h);
        let near = |h: f64, range: [f64; 2]| within(h, range, TOP_NEAR);
        row.top = found_top.zip(k.surface).filter(|(h, r)| near(*h, *r)).map(|(h, _)| h);
        let top = row.surface.map(|(game, mesh)| mesh.unwrap_or(game)).or(row.top);
        for (name, found, range) in [("seat", row.seat.map(|(game, mesh)| mesh.unwrap_or(game)), k.seat), ("top", top, k.surface), ("mattress", row.mattress.map(|(game, mesh)| mesh.unwrap_or(game)), k.mattress)] {
            let Some(range) = range else { continue };
            match found {
                None => {
                    let nearest = if name == "top" { found_top.map(|h| format!("; the nearest face looking up {h:.3}")).unwrap_or_default() } else { String::new() };
                    row.outside.push(format!("no {name} to measure ({:.2}–{:.2}{nearest})", range[0], range[1]));
                }
                Some(v) if !within(v, range, 0.0) => row.outside.push(outside(name, v, range, 0.0)),
                Some(_) => {}
            }
        }
    }
    row
}

/// A size row for a model key the reference lists in `ids` that no object draws (its size only).
fn model_row(key: &str, size: &Result<[f64; 3], String>, reference: &Reference) -> SizeRow {
    let object = Object { id: key.into(), name: String::new(), page: "models".into(), model: key.into(), ..Object::default() };
    let measure = size.clone().map(|size| Measure { model: key.into(), size, ..Measure::default() });
    size_row(&object, Some(&measure), reference)
}

fn cell_height(h: Option<(f64, Option<f64>)>) -> String {
    match h {
        None => "—".into(),
        Some((game, Some(mesh))) if (mesh - game).abs() > 0.02 => format!("{mesh:.2} (game {game:.2})"),
        Some((_, Some(mesh))) => format!("{mesh:.2}"),
        Some((game, None)) => format!("{game:.2} (no model there)"),
    }
}

/// W × H × D, with the H compared where it isn't the whole model's: `1.62 × 0.80 (table; 1.03 in all) × 0.87`,
/// `0.86 × 2.80 (top step; 14 steps of 0.200; 3.68 in all) × 4.00`.
fn size_cell(r: &SizeRow) -> String {
    let Some(s) = r.size else { return "—".into() };
    let h = match (r.h, r.steps) {
        (Some((by, h)), Some((n, rise))) => format!("{h:.2} ({by}; {n} steps of {rise:.3}; {:.2} in all)", s[1]),
        (Some((by, h)), None) => format!("{h:.2} ({by}; {:.2} in all)", s[1]),
        (None, _) => format!("{:.2}", s[1]),
    };
    format!("{:.2} × {h} × {:.2}", s[0], s[2])
}

fn size_cells(r: &SizeRow) -> String {
    let name = if r.name.is_empty() { String::new() } else { format!(" {}", r.name) };
    let top = match (r.surface, r.top) {
        (None, Some(top)) => format!("{top:.2} (no content `surface`)"),
        (surface, _) => cell_height(surface),
    };
    format!("| `{}`{name} | {} | {} | {} | {top} | {} |", r.id, r.kind.as_deref().unwrap_or("—"), size_cell(r), cell_height(r.seat), cell_height(r.mattress))
}

const SIZE_HEAD: &str = "| Object | Kind | W × H × D (m) | Seat | Top | Mattress |";
const SIZE_RULE: &str = "|---|---|---|---|---|---|";

/// The pages in the order of docs/objects/README.md, then any other.
fn page_order(pages: impl IntoIterator<Item = String>) -> Vec<String> {
    const ORDER: [&str; 16] = ["kitchen", "bathroom", "bedroom", "kids", "living", "office", "fitness", "hobbies", "decor", "tabletop", "lighting", "outdoor", "garden", "community", "packs", "models"];
    let mut all: Vec<String> = pages.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
    all.sort_by_key(|p| ORDER.iter().position(|o| o == p).unwrap_or(ORDER.len()));
    all
}

/// {object id: its kind's top range}, for each object whose kind has one: what `checkglb::measures` looks for where
/// the content has no `surface`.
pub fn tops(objects: &[Object], reference: &Reference) -> BTreeMap<String, [f64; 2]> {
    let top = |o: &Object| reference.kind_of(&o.id, &o.model, &o.category, &o.group).and_then(|k| reference.kind(k)).and_then(|k| k.surface);
    objects.iter().filter_map(|o| Some((o.id.clone(), top(o)?))).collect()
}

/// `cargo review sizes`: (stdout, the files for `--md`, exit code: 1 if an object maps to no kind).
pub fn sizes(objects: &[Object], measures: &checkglb::Measures, reference: &Reference) -> (String, BTreeMap<String, String>, i32) {
    let mut rows: Vec<SizeRow> = objects.iter().map(|o| size_row(o, measures.objects.get(&o.id), reference)).collect();
    let drawn: HashSet<&str> = objects.iter().map(|o| o.model.as_str()).collect();
    for key in reference.ids.keys().filter(|k| k.contains('.') && measures.models.contains_key(*k) && !drawn.contains(k.as_str())) {
        rows.push(model_row(key, &measures.models[key], reference));
    }
    let mut out = format!(
        "# Sizes against {REFERENCE}\n\nOverall W × H × D within ±{:.0} % of the kind's range (H: a stair's top step; a table's or a desk's without the seats its model carries); seat, top and mattress (slot 0's) heights (the model's, where it has one; a top the content hasn't, the model's near the range) strictly within it.\n",
        reference.tolerance * 100.0
    );
    let mut files = BTreeMap::new();
    for page in page_order(rows.iter().map(|r| r.page.clone())) {
        let here: Vec<&SizeRow> = rows.iter().filter(|r| r.page == page).collect();
        let odd: Vec<&&SizeRow> = here.iter().filter(|r| !r.outside.is_empty() && r.problem.is_none()).collect();
        let _ = write!(out, "\n## {page}: {} of {} outside\n", odd.len(), here.len());
        if !odd.is_empty() {
            let _ = write!(out, "\n{SIZE_HEAD} Outside |\n{SIZE_RULE}---|\n");
            for r in odd {
                let _ = writeln!(out, "{} {} |", size_cells(r), r.outside.join("; "));
            }
        }
        let mut md = format!("# Sizes: {page}\n\n`cargo review sizes --md`, against {REFERENCE} (±{:.0} % on W × H × D; functional heights strict).\n\n{SIZE_HEAD} Status |\n{SIZE_RULE}---|\n", reference.tolerance * 100.0);
        for r in &here {
            let _ = writeln!(md, "{} {} |", size_cells(r), r.status());
        }
        files.insert(format!("sizes-{page}.md"), md);
    }
    let problems: Vec<&SizeRow> = rows.iter().filter(|r| r.problem.as_deref().is_some_and(|p| !p.starts_with("unmeasured"))).collect();
    let unmeasured: Vec<&SizeRow> = rows.iter().filter(|r| r.problem.as_deref().is_some_and(|p| p.starts_with("unmeasured"))).collect();
    if !unmeasured.is_empty() {
        let _ = write!(out, "\n## Not measured ({})\n\n", unmeasured.len());
        for r in &unmeasured {
            let _ = writeln!(out, "- `{}` ({}): {}", r.id, r.page, r.problem.as_deref().unwrap_or_default());
        }
    }
    if !problems.is_empty() {
        let _ = write!(out, "\n## Error: {} objects map to no kind\n\n", problems.len());
        for r in &problems {
            let o = objects.iter().find(|o| o.id == r.id);
            let at = o.map(|o| format!("{}, group `{}`, {}", o.category, o.group, o.model)).unwrap_or_else(|| r.page.clone());
            let _ = writeln!(out, "- `{}` ({at}): {}", r.id, r.problem.as_deref().unwrap_or_default());
        }
    }
    (out, files, if problems.is_empty() { 0 } else { 1 })
}

fn short(v: f32) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// A model's `motions` as a table cell: `uses doze→bed_out; open bed_out`.
fn motions_cell(m: Option<&Value>) -> String {
    let Some(m) = m else { return "—".into() };
    let clip = |v: &Value| match v {
        Value::String(s) => s.clone(),
        Value::Array(l) => l.iter().map(|c| c.as_str().map(String::from).unwrap_or_else(|| c["clip"].as_str().unwrap_or("?").to_string())).collect::<Vec<_>>().join("+"),
        v => v["clip"].as_str().unwrap_or("?").to_string(),
    };
    let mut parts = Vec::new();
    for part in ["uses", "loops"] {
        if let Some(map) = m[part].as_object().filter(|m| !m.is_empty()) {
            parts.push(format!("{part} {}", map.iter().map(|(k, v)| format!("{k}→{}", clip(v))).collect::<Vec<_>>().join(", ")));
        }
    }
    for part in ["open", "idle", "night", "growth", "ride"] {
        if let Some(c) = m[part].as_str() {
            parts.push(format!("{part} {c}"));
        }
    }
    if m["clock"].is_object() {
        parts.push("clock".into());
    }
    if parts.is_empty() { "—".into() } else { parts.join("; ") }
}

const USES_HEAD: &str = "| Object | Kind | Uses (id:anim) | Pose | Minutes | Effects | Skills | Slots | Approach | Motions | Sound | Flags |\n|---|---|---|---|---|---|---|---|---|---|---|---|";

/// `cargo review uses`: (stdout, the files for `--md`). `motions` by model key; `reference` for the kinds (None:
/// no reference yet, so every object without a use is flagged).
pub fn uses(objects: &[Object], motions: &BTreeMap<String, Value>, actions: &BTreeSet<String>, sounds: &BTreeMap<String, String>, reference: Option<&Reference>) -> (String, BTreeMap<String, String>) {
    let medians = tag_medians(objects);
    let kind_of = |o: &Object| reference.and_then(|r| r.kind_of(&o.id, &o.model, &o.category, &o.group));
    let flagged: Vec<(&Object, Vec<Flag>)> = objects
        .iter()
        .map(|o| {
            let decor = kind_of(o).is_some_and(|k| reference.is_some_and(|r| r.decor.iter().any(|d| d == k)));
            (o, use_flags(o, motions.get(&o.model), actions, &medians, decor))
        })
        .collect();
    let mut out = String::from("# Uses: flags per category\n\n");
    if reference.is_none() {
        let _ = write!(out, "(No {REFERENCE} yet: no kind is decor, so every object without a use is flagged.)\n\n");
    }
    let _ = writeln!(out, "| Category | Objects | Flagged | {} |", FLAG_COLUMNS.join(" | "));
    let _ = writeln!(out, "|---|---|---|{}", "---|".repeat(FLAG_COLUMNS.len()));
    let mut totals = [0usize; 7];
    let mut files = BTreeMap::new();
    for page in page_order(objects.iter().map(|o| o.page.clone())) {
        let here: Vec<&(&Object, Vec<Flag>)> = flagged.iter().filter(|(o, _)| o.page == page).collect();
        let counts: Vec<usize> = FLAG_COLUMNS.iter().map(|c| here.iter().map(|(_, f)| f.iter().filter(|f| f.column() == *c).count()).sum()).collect();
        let n_flagged = here.iter().filter(|(_, f)| !f.is_empty()).count();
        let _ = writeln!(out, "| {page} | {} | {n_flagged} | {} |", here.len(), counts.iter().map(usize::to_string).collect::<Vec<_>>().join(" | "));
        totals[0] += here.len();
        totals[1] += n_flagged;
        for (t, c) in totals[2..].iter_mut().zip(&counts) {
            *t += c;
        }
        let mut md = format!("# Uses: {page}\n\n`cargo review uses --md`: per object its uses (one line each), its model's `motions` and what looks wrong.\n\n{USES_HEAD}\n");
        for (o, flags) in here {
            let lines = |f: &dyn Fn(&Use) -> String| if o.uses.is_empty() { "—".to_string() } else { o.uses.iter().map(f).collect::<Vec<_>>().join("<br>") };
            let list = |l: &[(String, f32)], unit: &str| if l.is_empty() { "—".to_string() } else { l.iter().map(|(n, g)| format!("{n} {}{unit}", short(*g))).collect::<Vec<_>>().join(", ") };
            let flags = if flags.is_empty() { "—".to_string() } else { flags.iter().map(Flag::text).collect::<Vec<_>>().join("<br>") };
            let _ = writeln!(
                md,
                "| `{}` {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {flags} |",
                o.id,
                o.name,
                kind_of(o).unwrap_or("—"),
                lines(&|u| format!("{}:{}", u.id, if u.anim.is_empty() { "—" } else { &u.anim })),
                lines(&|u| u.pose.to_string()),
                lines(&|u| short(u.minutes)),
                lines(&|u| list(&u.effects, "")),
                lines(&|u| list(&u.skills, "/h")),
                o.slots,
                o.approach,
                motions_cell(motions.get(&o.model)),
                lines(&|u| sound_of(sounds, &o.id, u).unwrap_or_else(|| "—".into())),
            );
        }
        files.insert(format!("uses-{page}.md"), md);
    }
    let _ = writeln!(out, "| **all** | {} | {} | {} |", totals[0], totals[1], totals[2..].iter().map(usize::to_string).collect::<Vec<_>>().join(" | "));
    (out, files)
}

/// Writes `files` into `dir` (made if missing).
fn write_md(dir: &Path, files: &BTreeMap<String, String>) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (name, text) in files {
        std::fs::write(dir.join(name), text).map_err(|e| format!("{}: {e}", dir.join(name).display()))?;
    }
    Ok(())
}

/// The checkout's files the review reads.
struct Sources {
    root: PathBuf,
    paths: Paths,
}

impl Sources {
    fn repo() -> Sources {
        let root = PathBuf::from(checkglb::repo_root());
        Sources { paths: Paths::repo(), root }
    }

    fn read(&self, rel: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(rel)).map_err(|e| format!("{rel}: {e}"))
    }
}

pub const USAGE: &str = "usage: cargo review sizes [--md <dir>] | uses [--md <dir>]";

/// `cargo review <sizes|uses> [--md <dir>]`: the exit code (2: usage, or no reference for `sizes`).
pub fn main(args: &[String]) -> i32 {
    let md = match args.iter().position(|a| a == "--md") {
        Some(i) => match args.get(i + 1) {
            Some(dir) => Some(PathBuf::from(dir)),
            None => {
                eprintln!("{USAGE}");
                return 2;
            }
        },
        None => None,
    };
    let s = Sources::repo();
    let run = || -> Result<i32, String> {
        let reference = match std::fs::read_to_string(s.root.join(REFERENCE)) {
            Ok(text) => Some(Reference::parse(&text)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(format!("{REFERENCE}: {e}")),
        };
        let objects = objects(Path::new(&s.paths.content))?;
        let (text, files, code) = match args.first().map(String::as_str) {
            Some("sizes") => {
                let Some(reference) = reference else {
                    eprintln!("no {REFERENCE} yet: `cargo review sizes` compares with it (plan/authenticity-review.md §3.4)");
                    return Ok(2);
                };
                sizes(&objects, &checkglb::measures(&s.paths, &tops(&objects, &reference))?, &reference)
            }
            Some("uses") => {
                let actions = action_keys(&s.read("web/src/render/babylon/characters.ts")?).map_err(|e| format!("characters.ts: {e}"))?;
                let sounds = action_sounds(&s.read("web/src/audio/world.ts")?).map_err(|e| format!("world.ts: {e}"))?;
                let (text, files) = uses(&objects, &motions(Path::new(&s.paths.assets))?, &actions, &sounds, reference.as_ref());
                (text, files, 0)
            }
            _ => {
                eprintln!("{USAGE}");
                return Ok(2);
            }
        };
        print!("{text}");
        if let Some(dir) = &md {
            write_md(dir, &files)?;
            eprintln!("wrote {} files to {}", files.len(), dir.display());
        }
        Ok(code)
    };
    run().unwrap_or_else(|e| {
        eprintln!("review: {e}");
        1
    })
}
