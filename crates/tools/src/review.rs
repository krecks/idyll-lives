//! `cargo review sizes|uses [--md <dir>]`: the authenticity review's first pass over every buyable object
//! (plan/authenticity-review.md §5.1, T3 and T4).
//!
//!   cargo review sizes [--md <dir>]   each object's W x H x D and its seat, top and mattress heights against
//!                                     docs/review/reference.json: the outliers per category (`--md`: every
//!                                     object, `<dir>/sizes-<cat>.md`); a plant against its kind's `species`
//!                                     (`species.rs`); then the textures' patterns and the kerb against its
//!                                     `surfaces` (`surfaces.rs`)
//!         [--check]                   fails on a row with more outside its range than tools/art/gate-baseline.json
//!                                     holds for it (`reference`, by id), or fewer (stale)
//!         [--update-baseline]         first lowers that section to what it finds (only lowers, takes ids out)
//!         [--seed-baseline]           writes the section where the file has none (the day the check came in)
//!   cargo review colours [--all]      every model's colours against what real surfaces reflect (`colours.rs`)
//!   cargo review facets [--all]       every model's round parts whose outline shows its flat sides (`facets.rs`)
//!   cargo review edges [--all] [keys] every model's hard edges that show unbevelled at the play camera (`edges.rs`)
//!   cargo review texels [--all] [keys]
//!                                     every model's textured finishes laid too sparse, dense, stretched or smeared
//!                                     (`texels.rs`)
//!   cargo review uses [--md <dir>]    each object's uses, poses, minutes, effects, skills, slots, approach,
//!                                     motions and sounds, and what looks wrong there: a summary per category
//!                                     (`--md`: every object, `<dir>/uses-<cat>.md`)
//!
//! The content is read as the game reads it (sim-core merges base.json and its includes, so `default_anim` and
//! the defaults apply), and raw for what sim-core keeps no copy of (`model`, `category`, `group`, a use's `when`,
//! `leaves` and its tags in order). Sizes and heights are `cargo check-glb`'s (`checkglb::measures`), with what
//! the review measures besides: a top its kind has and its content hasn't, a table's H without its seats and a
//! stair's top step (`h_of`). The manifests give each model's
//! `motions`; the renderer's `ACTIONS` (render/shared/characters/actions.ts) and action sounds (audio/world.ts) are read from its
//! source. A category is the object's buy category, or `packs` for a personality
//! pack's objects.

pub mod surfaces;

use crate::checkglb::baseline::{Baseline, Counts, Verdict};
use crate::checkglb::{self, Height, Measure, Paths};
use crate::species::{self, Atlas, Shape, Species};
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
    /// Real sizes of the surfaces' patterns and the kerb (`surfaces.rs`).
    #[serde(default)]
    pub surfaces: BTreeMap<String, surfaces::SurfaceKind>,
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
    /// What a plant's kind is of (`species`): its W and H are then its crown width and height, checked strictly
    /// from its model with the rest of the block, in place of the overall W × H × D.
    pub species: Option<Species>,
}

impl Reference {
    pub fn parse(text: &str) -> Result<Reference, String> {
        serde_json::from_str(text).map_err(|e| format!("{REFERENCE}: {e}"))
    }

    /// The kind an object is: `ids` by its id, then by its model key; else `groups` by "<category>/<group>",
    /// then by its group. None: none (an error for `sizes`). It may name a kind `kinds` doesn't have (`kind`).
    pub fn kind_of(&self, id: &str, model: &str, category: &str, group: &str) -> Option<&str> {
        let by_group = || {
            if group.is_empty() {
                None
            } else {
                self.groups
                    .get(&format!("{category}/{group}"))
                    .or_else(|| self.groups.get(group))
            }
        };
        self.ids
            .get(id)
            .or_else(|| self.ids.get(model))
            .or_else(by_group)
            .map(String::as_str)
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
    let side = if v < l {
        format!("{v:.3} < {l:.3}")
    } else {
        format!("{v:.3} > {h:.3}")
    };
    let allowance = if tolerance > 0.0 {
        format!(" ±{:.0} %", tolerance * 100.0)
    } else {
        String::new()
    };
    format!("{name} {side} ({lo:.2}–{hi:.2}{allowance})")
}

/// A buyable object as the review sees it.
#[derive(Debug, Clone, Default)]
pub struct Object {
    pub id: String,
    pub name: String,
    /// Its review category: its buy category; `packs` for a personality pack's objects.
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
    /// What's done at it sounds like, by animation tag (content `sounds`: world sound keys, or `none`).
    pub sounds: BTreeMap<String, String>,
    /// The world sound it makes by itself (content `ambient`: a fountain).
    pub ambient: Option<String>,
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
    /// Content `sound` (a world sound key, or `none`) and `endSound` (played once as it ends: a flush).
    pub sound: Option<String>,
    pub end_sound: Option<String>,
    /// It starts a recipe (content `recipe`): its steps done at this object (as one of its `stations`), each its
    /// animation tag and its own `sound`, else sounding by the object; the other steps are done elsewhere. None:
    /// no recipe.
    pub steps_here: Option<Vec<(String, Option<String>)>>,
}

/// The buyable objects of the content in `dir` (web/public/content), in the content's order.
pub fn objects(dir: &Path) -> Result<Vec<Object>, String> {
    let files = crate::content::load(dir)?;
    let (base, includes) = files.split_first().ok_or("no base.json")?;
    let merged = sim_core::merge_content(
        &base.text,
        &includes.iter().map(|f| f.text.as_str()).collect::<Vec<_>>(),
    )
    .map_err(|e| e.to_string())?;
    let content = sim_core::Content::from_json(&merged).map_err(|e| e.to_string())?;
    let merged: Value = serde_json::from_str(&merged).map_err(|e| e.to_string())?;
    let raw: HashMap<&str, &Value> = merged["objects"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|o| Some((o.get("id")?.as_str()?, o)))
        .collect();
    // A personality pack's objects are in `packs`, whatever their category.
    let mut packed = HashSet::new();
    for f in includes
        .iter()
        .filter(|f| f.path.starts_with(dir.join("packs")))
    {
        let file: Value =
            serde_json::from_str(&f.text).map_err(|e| format!("{}: {e}", f.path.display()))?;
        packed.extend(
            file["objects"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|o| o["id"].as_str().map(String::from)),
        );
    }
    let text = |v: Option<&Value>| v.and_then(Value::as_str).unwrap_or_default().to_string();
    let strs = |v: &Value| -> Vec<String> {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(|s| s.as_str().map(String::from))
            .collect()
    };
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
    for step in merged["recipes"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|r| r["steps"].as_array().into_iter().flatten())
    {
        // (`animAt`: another animation at some of its stations, a meal's Cook step baked in the oven.)
        for station in strs(&step["at"]) {
            let anim = step["animAt"].get(&station).unwrap_or(&step["anim"]);
            plays(vec![station], anim);
        }
    }
    let mut out = Vec::new();
    for def in content.objects.iter().filter(|d| d.price.is_some()) {
        let none = Value::Null;
        let raw = raw.get(def.id.as_str()).copied().unwrap_or(&none);
        let category = text(raw.get("category"));
        let raw_uses = raw["interactions"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let uses = def
            .interactions
            .iter()
            .map(|it| {
                let r = raw_uses
                    .iter()
                    .find(|r| r["id"].as_str() == Some(&it.id))
                    .unwrap_or(&none);
                let named = |gains: &[f32],
                             names: &mut dyn Iterator<Item = String>|
                 -> Vec<(String, f32)> {
                    names
                        .zip(gains)
                        .filter(|(_, g)| **g != 0.0)
                        .map(|(n, g)| (n, *g))
                        .collect()
                };
                Use {
                    id: it.id.clone(),
                    anim: it
                        .anim
                        .and_then(|a| content.animations.get(a))
                        .cloned()
                        .unwrap_or_default(),
                    pose: match it.pose {
                        sim_core::content::Pose::Stand => "stand",
                        sim_core::content::Pose::Sit => "sit",
                        sim_core::content::Pose::Lie => "lie",
                    },
                    minutes: it.minutes,
                    effects: named(
                        &it.total_gain,
                        &mut content.needs.iter().map(|n| n.id.clone()),
                    ),
                    skills: named(
                        &it.skill_gain,
                        &mut content.skills.iter().map(|s| s.id.clone()),
                    ),
                    tags: r["tags"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|t| t.as_str().map(String::from))
                        .collect(),
                    when: r["when"].as_str().map(String::from),
                    leaves: r["leaves"].as_str().map(String::from),
                    sound: r["sound"].as_str().map(String::from),
                    end_sound: r["endSound"].as_str().map(String::from),
                    steps_here: it.recipe.map(|i| {
                        let stations = strs(&raw["stations"]);
                        let steps = merged["recipes"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .find(|r| r["id"].as_str() == Some(&content.recipes[i].id));
                        steps
                            .into_iter()
                            .flat_map(|r| r["steps"].as_array().into_iter().flatten())
                            .filter(|s| strs(&s["at"]).iter().any(|a| stations.contains(a)))
                            .filter_map(|s| {
                                Some((
                                    s["anim"].as_str()?.to_string(),
                                    s["sound"].as_str().map(String::from),
                                ))
                            })
                            .collect()
                    }),
                }
            })
            .collect();
        out.push(Object {
            id: def.id.clone(),
            name: def.name.clone(),
            page: if packed.contains(&def.id) {
                "packs".into()
            } else {
                category.clone()
            },
            category,
            group: text(raw.get("group")),
            model: raw["model"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| format!("model.{}", def.id)),
            slots: def.slots,
            approach: match def.approach {
                sim_core::content::Approach::Front => "front",
                sim_core::content::Approach::Around => "around",
                sim_core::content::Approach::Side => "side",
                sim_core::content::Approach::Behind => "behind",
                sim_core::content::Approach::Door => "door",
            },
            uses,
            lamp: raw.get("light").is_some_and(|l| !l.is_null()),
            station_anims: strs(&raw["stations"])
                .iter()
                .flat_map(|s| at_station.get(s).into_iter().flatten().cloned())
                .collect(),
            sounds: raw["sounds"]
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                .collect(),
            ambient: raw["ambient"].as_str().map(String::from),
        });
    }
    Ok(out)
}

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
fn sound_cell(sounds: &BTreeMap<String, String>, o: &Object, u: &Use) -> String {
    let mut cell = sound_of(sounds, o, u).unwrap_or_else(|| "—".into());
    if let Some(end) = &u.end_sound {
        let _ = write!(cell, ", then {end}");
    }
    if let Some(ambient) = &o.ambient {
        let _ = write!(cell, " (+ {ambient} by itself)");
    }
    cell
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
    Duration {
        using: String,
        minutes: f32,
        like: String,
        median: f32,
    },
    /// A use's sound that every object using it is of one other kind (a sizzle away from a hob).
    OddSound {
        using: String,
        sound: String,
        kind: String,
    },
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
            Flag::OddSound { .. } => "odd sound",
        }
    }

    /// Whether the gate fails on it: a use the renderer can't play, a `motions` key no use answers to,
    /// or a model that opens without a clip. A missing use, a duration and a sound are the reviewer's to
    /// weigh — a table with no use of its own is still a table the sim uses as a station.
    pub fn hard(&self) -> bool {
        !matches!(
            self,
            Flag::NoUse | Flag::Duration { .. } | Flag::OddSound { .. }
        )
    }

    pub fn text(&self) -> String {
        match self {
            Flag::NoUse => "no use".into(),
            Flag::NoAction { using, anim } => {
                format!("{using}: anim `{anim}` has no `ACTIONS` entry")
            }
            Flag::MotionKey { key } => format!("`motions` key `{key}` is no use or anim of it"),
            Flag::NoOpen { using } => {
                format!("{using} opens or closes it, but its model has no `motions.open`")
            }
            Flag::Duration {
                using,
                minutes,
                like,
                median,
            } => format!("{using}: {minutes} min, the median for `{like}` {median} min"),
            Flag::OddSound { using, sound, kind } => {
                format!("{using}: sounds `{sound}`, which only a {kind} sounds")
            }
        }
    }
}

/// The columns of `Flag::column`, in order.
pub const FLAG_COLUMNS: [&str; 6] = [
    "no use",
    "anim not in ACTIONS",
    "motions key",
    "opens, no motions.open",
    "duration",
    "odd sound",
];

/// What a use's duration is compared with: the uses with the same animation (a night's sleep with sleeps, a nap
/// with naps; a first tag lumps them together), else with the same first tag.
pub fn like_key(u: &Use) -> Option<&str> {
    if u.anim.is_empty() {
        u.tags.first().map(String::as_str)
    } else {
        Some(u.anim.as_str())
    }
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
            (
                like,
                if n % 2 == 1 {
                    m[n / 2]
                } else {
                    (m[n / 2 - 1] + m[n / 2]) / 2.0
                },
            )
        })
        .collect()
}

/// What looks wrong in `object`'s uses: `motions` is its model's (None: none), `actions` the renderer's action
/// tags, `medians` the minutes per `like_key` (`tag_medians`), and `decor` whether its kind expects no use.
/// What a use's sound is weighed against: the tag map (`ACTION_SOUNDS`), the sounds only one kind makes
/// (`sound_owners`) and this object's kind.
pub struct Sounds<'a> {
    pub tag: &'a BTreeMap<String, String>,
    pub owners: &'a BTreeMap<String, String>,
    pub kind: Option<&'a str>,
}

pub fn use_flags(
    object: &Object,
    motions: Option<&Value>,
    actions: &BTreeSet<String>,
    medians: &BTreeMap<String, f32>,
    decor: bool,
    heard: &Sounds,
) -> Vec<Flag> {
    let mut out = Vec::new();
    if object.uses.is_empty() && !object.lamp && !decor {
        out.push(Flag::NoUse);
    }
    for u in &object.uses {
        if !u.anim.is_empty() && !actions.contains(&u.anim) {
            out.push(Flag::NoAction {
                using: u.id.clone(),
                anim: u.anim.clone(),
            });
        }
    }
    let named: BTreeSet<&str> = object
        .uses
        .iter()
        .flat_map(|u| [u.id.as_str(), u.anim.as_str()])
        .chain(object.station_anims.iter().map(String::as_str))
        .filter(|k| !k.is_empty())
        .collect();
    for part in ["uses", "loops"] {
        for key in motions
            .and_then(|m| m[part].as_object())
            .into_iter()
            .flat_map(|m| m.keys())
        {
            if !named.contains(key.as_str()) {
                out.push(Flag::MotionKey { key: key.clone() });
            }
        }
    }
    let opens = motions.is_some_and(|m| m.get("open").is_some_and(|o| !o.is_null()));
    for u in &object.uses {
        let open_or_closed = |s: &Option<String>| matches!(s.as_deref(), Some("open" | "closed"));
        if (open_or_closed(&u.when) || open_or_closed(&u.leaves)) && !opens {
            out.push(Flag::NoOpen {
                using: u.id.clone(),
            });
        }
    }
    for u in &object.uses {
        let Some(like) = like_key(u) else { continue };
        let Some(&median) = medians.get(like) else {
            continue;
        };
        if median > 0.0
            && (u.minutes > median * DURATION_SPREAD || u.minutes * DURATION_SPREAD < median)
        {
            out.push(Flag::Duration {
                using: u.id.clone(),
                minutes: u.minutes,
                like: like.to_string(),
                median,
            });
        }
    }
    if let Some(kind) = heard.kind {
        for u in &object.uses {
            let Some(sound) = sound_of(heard.tag, object, u) else {
                continue;
            };
            if let Some(owns) = heard.owners.get(&sound)
                && owns != kind
            {
                out.push(Flag::OddSound {
                    using: u.id.clone(),
                    sound,
                    kind: owns.clone(),
                });
            }
        }
    }
    out
}

/// The sounds only one kind of object makes: `sound` → that kind, for every sound `SOUND_OWNERS` objects of one
/// kind make and nothing else does. A sizzle belongs to the hobs; a toaster that sizzles is flagged.
pub const SOUND_OWNERS: usize = 3;

pub fn sound_owners(
    objects: &[Object],
    sounds: &BTreeMap<String, String>,
    kind_of: &dyn Fn(&Object) -> Option<String>,
) -> BTreeMap<String, String> {
    let mut by_sound: BTreeMap<String, (BTreeSet<String>, usize)> = BTreeMap::new();
    for o in objects {
        let Some(kind) = kind_of(o) else { continue };
        for u in &o.uses {
            let Some(sound) = sound_of(sounds, o, u) else {
                continue;
            };
            let entry = by_sound.entry(sound).or_default();
            entry.0.insert(kind.clone());
            entry.1 += 1;
        }
    }
    by_sound
        .into_iter()
        .filter(|(_, (kinds, n))| *n >= SOUND_OWNERS && kinds.len() == 1)
        .map(|(sound, (kinds, _))| (sound, kinds.into_iter().next().unwrap_or_default()))
        .collect()
}

/// One object against the reference (`cargo review sizes`).
#[derive(Debug, Default)]
pub struct SizeRow {
    pub id: String,
    pub name: String,
    pub page: String,
    /// The model it draws.
    pub model: String,
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
    /// Its plant as measured against its kind's `species` (`species_row`).
    pub species: Option<Shape>,
    /// What is outside its kind's ranges: each a finding the baseline counts (`--check`).
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
    heights
        .iter()
        .find(|h| whats.contains(&h.what))
        .map(|h| (h.expected, h.mesh))
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
        (None, Some(table)) if kind.surface.is_some() && kind.seat.is_none() => {
            Some(("table", table))
        }
        _ => None,
    }
}

/// `object` (its id, model, category and group) measured (`measure`), against its kind in `reference`. The
/// functional heights are the model's where it has one there (else the game's), strictly in range; the top,
/// where the content has none, the model's top near the kind's range (`TOP_NEAR`).
pub fn size_row(
    object: &Object,
    measure: Option<&Result<Measure, String>>,
    reference: &Reference,
) -> SizeRow {
    let mut row = SizeRow {
        id: object.id.clone(),
        name: object.name.clone(),
        page: object.page.clone(),
        model: object.model.clone(),
        ..SizeRow::default()
    };
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
    let Some(kind) = reference.kind_of(&object.id, &object.model, &object.category, &object.group)
    else {
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
        // A plant's W and H are its crown's and its own, strictly (`species_row`).
        let (w, h_range) = if k.species.is_some() {
            (None, None)
        } else {
            (k.w, k.h)
        };
        for (name, v, range) in [
            ("W", size[0], w),
            (h_name.as_str(), h, h_range),
            ("D", size[2], k.d),
        ] {
            if let Some(range) = range.filter(|r| !within(v, *r, reference.tolerance)) {
                row.outside
                    .push(outside(name, v, range, reference.tolerance));
            }
        }
    }
    if row.problem.is_none() {
        // A top the content hasn't: the model's, if near enough the range to be one.
        let found_top = measured
            .and_then(|m| m.top)
            .filter(|_| row.surface.is_none())
            .map(|(_, h)| h);
        let near = |h: f64, range: [f64; 2]| within(h, range, TOP_NEAR);
        row.top = found_top
            .zip(k.surface)
            .filter(|(h, r)| near(*h, *r))
            .map(|(h, _)| h);
        let top = row
            .surface
            .map(|(game, mesh)| mesh.unwrap_or(game))
            .or(row.top);
        for (name, found, range) in [
            (
                "seat",
                row.seat.map(|(game, mesh)| mesh.unwrap_or(game)),
                k.seat,
            ),
            ("top", top, k.surface),
            (
                "mattress",
                row.mattress.map(|(game, mesh)| mesh.unwrap_or(game)),
                k.mattress,
            ),
        ] {
            let Some(range) = range else { continue };
            match found {
                None => {
                    let nearest = if name == "top" {
                        found_top
                            .map(|h| format!("; the nearest face looking up {h:.3}"))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    row.outside.push(format!(
                        "no {name} to measure ({:.2}–{:.2}{nearest})",
                        range[0], range[1]
                    ));
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
    let object = Object {
        id: key.into(),
        name: String::new(),
        page: "models".into(),
        model: key.into(),
        ..Object::default()
    };
    let measure = size.clone().map(|size| Measure {
        model: key.into(),
        size,
        ..Measure::default()
    });
    size_row(&object, Some(&measure), reference)
}

fn cell_height(h: Option<(f64, Option<f64>)>) -> String {
    match h {
        None => "—".into(),
        Some((game, Some(mesh))) if (mesh - game).abs() > 0.02 => {
            format!("{mesh:.2} (game {game:.2})")
        }
        Some((_, Some(mesh))) => format!("{mesh:.2}"),
        Some((game, None)) => format!("{game:.2} (no model there)"),
    }
}

/// W × H × D, with the H compared where it isn't the whole model's: `1.62 × 0.80 (table; 1.03 in all) × 0.87`,
/// `0.86 × 2.80 (top step; 14 steps of 0.200; 3.68 in all) × 4.00`.
fn size_cell(r: &SizeRow) -> String {
    let Some(s) = r.size else { return "—".into() };
    let h = match (r.h, r.steps) {
        (Some((by, h)), Some((n, rise))) => {
            format!("{h:.2} ({by}; {n} steps of {rise:.3}; {:.2} in all)", s[1])
        }
        (Some((by, h)), None) => format!("{h:.2} ({by}; {:.2} in all)", s[1]),
        (None, _) => format!("{:.2}", s[1]),
    };
    format!("{:.2} × {h} × {:.2}", s[0], s[2])
}

fn size_cells(r: &SizeRow) -> String {
    let name = if r.name.is_empty() {
        String::new()
    } else {
        format!(" {}", r.name)
    };
    let top = match (r.surface, r.top) {
        (None, Some(top)) => format!("{top:.2} (no content `surface`)"),
        (surface, _) => cell_height(surface),
    };
    format!(
        "| `{}`{name} | {} | {} | {} | {top} | {} |",
        r.id,
        r.kind.as_deref().unwrap_or("—"),
        size_cell(r),
        cell_height(r.seat),
        cell_height(r.mattress)
    )
}

const SIZE_HEAD: &str = "| Object | Kind | W × H × D (m) | Seat | Top | Mattress |";
const SIZE_RULE: &str = "|---|---|---|---|---|---|";

/// The pages in the buy catalogue's order, then any other.
fn page_order(pages: impl IntoIterator<Item = String>) -> Vec<String> {
    const ORDER: [&str; 16] = [
        "kitchen",
        "bathroom",
        "bedroom",
        "kids",
        "living",
        "office",
        "fitness",
        "hobbies",
        "decor",
        "tabletop",
        "lighting",
        "outdoor",
        "garden",
        "community",
        "packs",
        "models",
    ];
    let mut all: Vec<String> = pages
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    all.sort_by_key(|p| ORDER.iter().position(|o| o == p).unwrap_or(ORDER.len()));
    all
}

/// {object id: its kind's top range}, for each object whose kind has one: what `checkglb::measures` looks for where
/// the content has no `surface`.
pub fn tops(objects: &[Object], reference: &Reference) -> BTreeMap<String, [f64; 2]> {
    let top = |o: &Object| {
        reference
            .kind_of(&o.id, &o.model, &o.category, &o.group)
            .and_then(|k| reference.kind(k))
            .and_then(|k| k.surface)
    };
    objects
        .iter()
        .filter_map(|o| Some((o.id.clone(), top(o)?)))
        .collect()
}

/// Every object's size row (`size_row`), and one for each model key in `ids` that no object draws.
pub fn size_rows(
    objects: &[Object],
    measures: &checkglb::Measures,
    reference: &Reference,
) -> Vec<SizeRow> {
    let mut rows: Vec<SizeRow> = objects
        .iter()
        .map(|o| size_row(o, measures.objects.get(&o.id), reference))
        .collect();
    let drawn: HashSet<&str> = objects.iter().map(|o| o.model.as_str()).collect();
    for key in reference.ids.keys().filter(|k| {
        k.contains('.') && measures.models.contains_key(*k) && !drawn.contains(k.as_str())
    }) {
        rows.push(model_row(key, &measures.models[key], reference));
    }
    rows
}

/// A plant's `row` (its kind's `species` in `reference`) with its model measured (`species::measure`): what is
/// outside the block, or the kind's `h` and `w` strictly, joins its findings; a model that can't be read is its
/// problem. A row of no such kind is left as it is.
pub fn species_row(
    row: &mut SizeRow,
    reference: &Reference,
    shape: Result<Shape, String>,
    atlas: &Atlas,
) {
    let Some(k) = row.kind.as_deref().and_then(|k| reference.kind(k)) else {
        return;
    };
    let Some(sp) = &k.species else { return };
    match shape {
        Ok(shape) => {
            row.outside
                .extend(species::findings(sp, k.h, k.w, &shape, atlas));
            row.species = Some(shape);
        }
        Err(e) => row.problem = Some(format!("unmeasured: {e}")),
    }
}

/// `cargo review sizes` over `objects` (`size_rows`, no plant measured): (stdout, the files for `--md`, exit
/// code: 1 if an object maps to no kind or a kind's `species` isn't one).
pub fn sizes(
    objects: &[Object],
    measures: &checkglb::Measures,
    reference: &Reference,
) -> (String, BTreeMap<String, String>, i32) {
    report(objects, &size_rows(objects, measures, reference), reference)
}

/// `sizes`'s report of `rows`.
pub fn report(
    objects: &[Object],
    rows: &[SizeRow],
    reference: &Reference,
) -> (String, BTreeMap<String, String>, i32) {
    let mut out = format!(
        "# Sizes against {REFERENCE}\n\nOverall W × H × D within ±{:.0} % of the kind's range (H: a stair's top step; a table's or a desk's without the seats its model carries); seat, top and mattress (slot 0's) heights (the model's, where it has one; a top the content hasn't, the model's near the range) strictly within it.\n",
        reference.tolerance * 100.0
    );
    let mut files = BTreeMap::new();
    for page in page_order(rows.iter().map(|r| r.page.clone())) {
        let here: Vec<&SizeRow> = rows.iter().filter(|r| r.page == page).collect();
        let odd: Vec<&&SizeRow> = here
            .iter()
            .filter(|r| !r.outside.is_empty() && r.problem.is_none())
            .collect();
        let _ = write!(
            out,
            "\n## {page}: {} of {} outside\n",
            odd.len(),
            here.len()
        );
        if !odd.is_empty() {
            let _ = write!(out, "\n{SIZE_HEAD} Outside |\n{SIZE_RULE}---|\n");
            for r in odd {
                let _ = writeln!(out, "{} {} |", size_cells(r), r.outside.join("; "));
            }
        }
        let mut md = format!(
            "# Sizes: {page}\n\n`cargo review sizes --md`, against {REFERENCE} (±{:.0} % on W × H × D; functional heights strict).\n\n{SIZE_HEAD} Status |\n{SIZE_RULE}---|\n",
            reference.tolerance * 100.0
        );
        for r in &here {
            let _ = writeln!(md, "{} {} |", size_cells(r), r.status());
        }
        files.insert(format!("sizes-{page}.md"), md);
    }
    let plants: Vec<&SizeRow> = rows.iter().filter(|r| r.species.is_some()).collect();
    if !plants.is_empty() {
        let _ = write!(
            out,
            "\n## Species: {} of {} plants outside\n\nEach against its kind's `species` and its `h` and `w`, strictly (`species.rs`): the plant's H (from the foot of its trunk or stems), its crown's widest, crown W/H, the clear trunk under the leaves (of H), the bark's luminance, the colours on it that aren't a leaf's green and the leaf cards' triangles by the atlas tile they are on.\n\n{SPECIES_HEAD}\n",
            plants.iter().filter(|r| !r.outside.is_empty()).count(),
            plants.len()
        );
        for r in &plants {
            let _ = writeln!(out, "{}", species_cells(r, reference));
        }
    }
    let mut problems: Vec<&SizeRow> = rows
        .iter()
        .filter(|r| {
            r.problem
                .as_deref()
                .is_some_and(|p| !p.starts_with("unmeasured"))
        })
        .collect();
    problems.sort_by_key(|r| r.page != "models");
    let unmeasured: Vec<&SizeRow> = rows
        .iter()
        .filter(|r| {
            r.problem
                .as_deref()
                .is_some_and(|p| p.starts_with("unmeasured"))
        })
        .collect();
    if !unmeasured.is_empty() {
        let _ = write!(out, "\n## Not measured ({})\n\n", unmeasured.len());
        for r in &unmeasured {
            let _ = writeln!(
                out,
                "- `{}` ({}): {}",
                r.id,
                r.page,
                r.problem.as_deref().unwrap_or_default()
            );
        }
    }
    if !problems.is_empty() {
        let _ = write!(
            out,
            "\n## Error: {} objects map to no kind\n\n",
            problems.len()
        );
        for r in &problems {
            let o = objects.iter().find(|o| o.id == r.id);
            let at = o
                .map(|o| format!("{}, group `{}`, {}", o.category, o.group, o.model))
                .unwrap_or_else(|| r.page.clone());
            let _ = writeln!(
                out,
                "- `{}` ({at}): {}",
                r.id,
                r.problem.as_deref().unwrap_or_default()
            );
        }
    }
    let bad: Vec<String> = reference
        .kinds
        .iter()
        .filter_map(|(name, k)| Some((name, k.species.as_ref()?.problems())))
        .flat_map(|(name, p)| p.into_iter().map(move |p| format!("- `{name}`: {p}")))
        .collect();
    if !bad.is_empty() {
        let _ = write!(
            out,
            "\n## Error: {} `species` entries aren't what a species is\n\n{}\n",
            bad.len(),
            bad.join("\n")
        );
    }
    (
        out,
        files,
        if problems.is_empty() && bad.is_empty() {
            0
        } else {
            1
        },
    )
}

const SPECIES_HEAD: &str = "| Object | Kind | Species | Plant H | Crown W | Crown W/H | Clear trunk | Bark | Colours | Leaf cards | Outside |\n|---|---|---|---|---|---|---|---|---|---|---|";

/// A plant's row of the species table: each measure with its range.
fn species_cells(r: &SizeRow, reference: &Reference) -> String {
    let k = r.kind.as_deref().and_then(|k| reference.kind(k));
    let sp = k.and_then(|k| k.species.as_ref());
    let Some(s) = &r.species else {
        return String::new();
    };
    let cell = |v: Option<f64>, range: Option<[f64; 2]>, digits: usize| match (v, range) {
        (None, _) => "—".to_string(),
        (Some(v), None) => format!("{v:.digits$}"),
        (Some(v), Some([lo, hi])) => format!("{v:.digits$} ({lo:.digits$}–{hi:.digits$})"),
    };
    let ratio = s.crown.filter(|_| s.height > 0.0).map(|c| c / s.height);
    let colours = if s.colours.is_empty() {
        "—".to_string()
    } else {
        s.colours
            .iter()
            .map(|c| format!("{} {:.0}°", c.material, c.hue))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let tiles = if s.tiles.is_empty() {
        "—".to_string()
    } else {
        s.tiles
            .iter()
            .map(|(class, n)| {
                let class = if class.is_empty() { "none" } else { class };
                format!("{n} {class}")
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "| `{}` | {} | {} | {} | {} | {} | {} | {} | {colours} | {tiles} | {} |",
        r.id,
        r.kind.as_deref().unwrap_or("—"),
        sp.map_or("—".into(), |sp| format!("*{}*", sp.latin)),
        cell(Some(s.height), k.and_then(|k| k.h), 2),
        cell(s.crown, k.and_then(|k| k.w), 2),
        cell(ratio, sp.and_then(|sp| sp.crown), 2),
        cell(s.clear, sp.and_then(|sp| sp.clear), 2),
        cell(s.bark, sp.and_then(|sp| sp.bark.as_ref()?.luminance), 3),
        if r.outside.is_empty() {
            "ok".to_string()
        } else {
            r.outside.join("; ")
        }
    )
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
        Value::Array(l) => l
            .iter()
            .map(|c| {
                c.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| c["clip"].as_str().unwrap_or("?").to_string())
            })
            .collect::<Vec<_>>()
            .join("+"),
        v => v["clip"].as_str().unwrap_or("?").to_string(),
    };
    let mut parts = Vec::new();
    for part in ["uses", "loops"] {
        if let Some(map) = m[part].as_object().filter(|m| !m.is_empty()) {
            parts.push(format!(
                "{part} {}",
                map.iter()
                    .map(|(k, v)| format!("{k}→{}", clip(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    for part in ["open", "idle", "night", "growth", "ride", "running", "burn"] {
        if let Some(c) = m[part].as_str() {
            parts.push(format!("{part} {c}"));
        }
    }
    if m["clock"].is_object() {
        parts.push("clock".into());
    }
    if parts.is_empty() {
        "—".into()
    } else {
        parts.join("; ")
    }
}

const USES_HEAD: &str = "| Object | Kind | Uses (id:anim) | Pose | Minutes | Effects | Skills | Slots | Approach | Motions | Sound | Flags |\n|---|---|---|---|---|---|---|---|---|---|---|---|";

/// `cargo review uses`: (stdout, the files for `--md`, the hard flags `--check` fails on as `id: flag`). `motions`
/// by model key; `reference` for the kinds (None: no reference yet, so every object without a use is flagged).
pub fn uses(
    objects: &[Object],
    motions: &BTreeMap<String, Value>,
    actions: &BTreeSet<String>,
    sounds: &BTreeMap<String, String>,
    reference: Option<&Reference>,
) -> (String, BTreeMap<String, String>, Vec<String>) {
    let flagged = flagged(objects, motions, actions, sounds, reference);
    let hard: Vec<String> = flagged
        .iter()
        .flat_map(|(o, _, flags)| {
            flags
                .iter()
                .filter(|f| f.hard())
                .map(move |f| format!("{}: {}", o.id, f.text()))
        })
        .collect();
    let mut out = String::from("# Uses: flags per category\n\n");
    if reference.is_none() {
        let _ = write!(
            out,
            "(No {REFERENCE} yet: no kind is decor, so every object without a use is flagged.)\n\n"
        );
    }
    let _ = writeln!(
        out,
        "| Category | Objects | Flagged | {} |",
        FLAG_COLUMNS.join(" | ")
    );
    let _ = writeln!(out, "|---|---|---|{}", "---|".repeat(FLAG_COLUMNS.len()));
    let mut totals = [0usize; FLAG_COLUMNS.len() + 2];
    let mut files = BTreeMap::new();
    for page in page_order(objects.iter().map(|o| o.page.clone())) {
        let here: Vec<&(&Object, Option<&str>, Vec<Flag>)> =
            flagged.iter().filter(|(o, _, _)| o.page == page).collect();
        let counts: Vec<usize> = FLAG_COLUMNS
            .iter()
            .map(|c| {
                here.iter()
                    .map(|(_, _, f)| f.iter().filter(|f| f.column() == *c).count())
                    .sum()
            })
            .collect();
        let n_flagged = here.iter().filter(|(_, _, f)| !f.is_empty()).count();
        let _ = writeln!(
            out,
            "| {page} | {} | {n_flagged} | {} |",
            here.len(),
            counts
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(" | ")
        );
        totals[0] += here.len();
        totals[1] += n_flagged;
        for (t, c) in totals[2..].iter_mut().zip(&counts) {
            *t += c;
        }
        let mut md = format!(
            "# Uses: {page}\n\n`cargo review uses --md`: per object its uses (one line each), its model's `motions` and what looks wrong.\n\n{USES_HEAD}\n"
        );
        for (o, kind, flags) in here {
            let lines = |f: &dyn Fn(&Use) -> String| {
                if o.uses.is_empty() {
                    "—".to_string()
                } else {
                    o.uses.iter().map(f).collect::<Vec<_>>().join("<br>")
                }
            };
            let list = |l: &[(String, f32)], unit: &str| {
                if l.is_empty() {
                    "—".to_string()
                } else {
                    l.iter()
                        .map(|(n, g)| format!("{n} {}{unit}", short(*g)))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            };
            let flags = if flags.is_empty() {
                "—".to_string()
            } else {
                flags
                    .iter()
                    .map(Flag::text)
                    .collect::<Vec<_>>()
                    .join("<br>")
            };
            let _ = writeln!(
                md,
                "| `{}` {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {flags} |",
                o.id,
                o.name,
                kind.unwrap_or("—"),
                lines(&|u| format!("{}:{}", u.id, if u.anim.is_empty() { "—" } else { &u.anim })),
                lines(&|u| u.pose.to_string()),
                lines(&|u| short(u.minutes)),
                lines(&|u| list(&u.effects, "")),
                lines(&|u| list(&u.skills, "/h")),
                o.slots,
                o.approach,
                motions_cell(motions.get(&o.model)),
                lines(&|u| sound_cell(sounds, o, u)),
            );
        }
        files.insert(format!("uses-{page}.md"), md);
    }
    let _ = writeln!(
        out,
        "| **all** | {} | {} | {} |",
        totals[0],
        totals[1],
        totals[2..]
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(" | ")
    );
    // Why an object with no uses is not flagged: its kind is decor by the reference's decision.
    let silent = objects
        .iter()
        .filter(|o| o.uses.is_empty() && !o.lamp)
        .count();
    let _ = writeln!(
        out,
        "\n- {} objects have no use; decor by `{REFERENCE}`'s `decor` list (the decision record): {}; the rest are flagged.",
        silent,
        silent.saturating_sub(totals[2])
    );
    (out, files, hard)
}

/// Every object with its kind (the reference's) and its flags, in the content's order: the reference says which
/// kinds are decor and which kind owns a sound.
fn flagged<'a>(
    objects: &'a [Object],
    motions: &BTreeMap<String, Value>,
    actions: &BTreeSet<String>,
    sounds: &BTreeMap<String, String>,
    reference: Option<&'a Reference>,
) -> Vec<(&'a Object, Option<&'a str>, Vec<Flag>)> {
    let medians = tag_medians(objects);
    let kind_of =
        |o: &Object| reference.and_then(|r| r.kind_of(&o.id, &o.model, &o.category, &o.group));
    let owners = sound_owners(objects, sounds, &|o| kind_of(o).map(str::to_string));
    objects
        .iter()
        .map(|o| {
            let kind = kind_of(o);
            let decor =
                kind.is_some_and(|k| reference.is_some_and(|r| r.decor.iter().any(|d| d == k)));
            let heard = Sounds {
                tag: sounds,
                owners: &owners,
                kind,
            };
            let flags = use_flags(o, motions.get(&o.model), actions, &medians, decor, &heard);
            (o, kind, flags)
        })
        .collect()
}

/// Writes `files` into `dir` (made if missing).
fn write_md(dir: &Path, files: &BTreeMap<String, String>) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (name, text) in files {
        std::fs::write(dir.join(name), text)
            .map_err(|e| format!("{}: {e}", dir.join(name).display()))?;
    }
    Ok(())
}

/// The baseline's section (tools/art/gate-baseline.json) `cargo review sizes --check` judges by: how many
/// findings each measured row has (its `outside`), by its id (an object's, or a model key no object draws).
pub const CHECK: &str = "reference";

/// Each measured row's findings, by its id (a row that couldn't be measured is left out).
pub fn counts(rows: &[SizeRow]) -> BTreeMap<String, u64> {
    rows.iter()
        .filter(|r| r.problem.is_none())
        .map(|r| (r.id.clone(), r.outside.len() as u64))
        .collect()
}

/// What `--check` fails on, against `baseline`'s `reference` section: a row with more findings than it holds, or
/// any where it holds none (new objects are strict); one with fewer (stale: `--update-baseline` lowers it); and an
/// id it holds that no row has.
pub fn judged(rows: &[SizeRow], baseline: &Baseline) -> Vec<String> {
    let mut out = Vec::new();
    for r in rows.iter().filter(|r| r.problem.is_none()) {
        let count = r.outside.len() as u64;
        let what = r.outside.join("; ");
        match baseline.judge(CHECK, &r.id, count) {
            Verdict::Clean | Verdict::Held(_) => {}
            Verdict::New(n) => out.push(format!(
                "`{}`: {n} outside its kind's range, none in the baseline: {what}",
                r.id
            )),
            Verdict::Over { base, count } => out.push(format!(
                "`{}`: {count} outside its kind's range, {base} in the baseline: fix the new ones: {what}",
                r.id
            )),
            Verdict::Stale { base, count } => out.push(format!(
                "`{}`: {count} outside its kind's range, {base} in the baseline: lower it (`cargo review sizes --update-baseline`)",
                r.id
            )),
        }
    }
    let ids: HashSet<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    for key in baseline.keys(CHECK).filter(|k| !ids.contains(k)) {
        out.push(format!(
            "`{key}`: in the baseline, but no object or model the review measured: take it out (`cargo review sizes --update-baseline`)"
        ));
    }
    out
}

/// `baseline` lowered to what `rows` found (`Baseline::lowered`): never raised, nothing added; an id no row
/// has goes, one whose row couldn't be measured stays. (The baseline, a line for each change.)
pub fn lowered(rows: &[SizeRow], baseline: &Baseline) -> (Baseline, Vec<String>) {
    let mut found = counts(rows);
    let ids: HashSet<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    for key in baseline.keys(CHECK).filter(|k| !ids.contains(k)) {
        found.insert(key.to_string(), 0);
    }
    baseline.lowered(&Counts::from([(CHECK.to_string(), found)]))
}

/// `baseline` with a `reference` section of what `rows` found, the day the check comes in: refused where it has
/// one (only `--update-baseline` changes it then, and only down).
pub fn seeded(rows: &[SizeRow], baseline: &Baseline) -> Result<Baseline, String> {
    if baseline.checks().any(|c| c == CHECK) {
        return Err(format!(
            "it has a `{CHECK}` section already: `--update-baseline` lowers it"
        ));
    }
    let mut all: Value = serde_json::from_str(&baseline.text()).map_err(|e| e.to_string())?;
    let section: serde_json::Map<String, Value> = counts(rows)
        .into_iter()
        .filter(|(_, n)| *n > 0)
        .map(|(id, n)| (id, Value::from(n)))
        .collect();
    if let Some(all) = all.as_object_mut().filter(|_| !section.is_empty()) {
        all.insert(CHECK.into(), Value::Object(section));
    }
    Baseline::parse(&all.to_string())
}

/// `sizes`'s baseline flags over `rows`: `--seed-baseline` writes the section (`seeded`), `--update-baseline`
/// lowers it (`lowered`), then `--check` judges by it (`judged`). The exit code: 1 if `--check` found something.
fn baselined(paths: &Paths, rows: &[SizeRow], args: &[String]) -> Result<i32, String> {
    use crate::checkglb::baseline::FILE;
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let mut baseline = Baseline::read(&paths.baseline)?;
    if has("--seed-baseline") {
        baseline = seeded(rows, &baseline).map_err(|e| format!("{FILE}: {e}"))?;
        baseline.write(&paths.baseline)?;
        eprintln!(
            "{FILE}: `{CHECK}` holds {} rows",
            baseline.keys(CHECK).count()
        );
    }
    if has("--update-baseline") {
        let (low, changes) = lowered(rows, &baseline);
        if changes.is_empty() {
            eprintln!("{FILE}: nothing to lower");
        } else {
            low.write(&paths.baseline)?;
            for c in &changes {
                eprintln!("{FILE}: {c}");
            }
        }
        baseline = low;
    }
    if !has("--check") {
        return Ok(0);
    }
    let errors = judged(rows, &baseline);
    let held = baseline.keys(CHECK).count();
    if errors.is_empty() {
        eprintln!(
            "review sizes: every row within its kind's range or held by {FILE} ({held} held)"
        );
        return Ok(0);
    }
    eprintln!(
        "review sizes: {} rows against {FILE} ({held} held):",
        errors.len()
    );
    for e in &errors {
        eprintln!("  {e}");
    }
    Ok(1)
}

/// Measures each row whose kind has a `species` from its model (`species::read`) and adds what is off
/// (`species_row`).
fn measure_plants(s: &Sources, rows: &mut [SizeRow], reference: &Reference) -> Result<(), String> {
    let plant = |r: &SizeRow| {
        r.kind
            .as_deref()
            .and_then(|k| reference.kind(k))
            .is_some_and(|k| k.species.is_some())
            && r.problem.is_none()
    };
    if !rows.iter().any(plant) {
        return Ok(());
    }
    let files = species::model_files(&s.paths)?;
    let assets = Path::new(&s.paths.assets);
    let base: Value = serde_json::from_str(&s.read("web/public/assets/manifest.json")?)
        .map_err(|e| format!("manifest.json: {e}"))?;
    let finishes = crate::colours::finishes(&base, assets);
    let atlas = Atlas::read(&s.root.to_string_lossy())?;
    for r in rows.iter_mut().filter(|r| plant(r)) {
        let shape = match files.get(&r.model) {
            Some((path, entry)) => species::read(path, entry.get("materials"), &finishes)
                .map(|p| species::measure(&p, &atlas)),
            None => Err(format!("no manifest has a model file for {}", r.model)),
        };
        species_row(r, reference, shape, &atlas);
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
        Sources {
            paths: Paths::repo(),
            root,
        }
    }

    fn read(&self, rel: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(rel)).map_err(|e| format!("{rel}: {e}"))
    }
}

pub const USAGE: &str = "usage: cargo review sizes [--md <dir>] [--check] [--update-baseline] [--seed-baseline] | uses [--md <dir>] [--check] | colours [--all] [--check] | facets [--all] [--check] | edges [--all] [--check] [keys] | texels [--all] [--check] [keys]";

/// `cargo review <sizes|uses> [--md <dir>] [--check]`: the exit code (2: usage, or no reference for
/// `sizes`; 1: an object of no kind, a `species` that isn't one, or `--check` found a row the baseline doesn't
/// hold or a hard flag, a use the renderer can't play).
pub fn main(args: &[String]) -> i32 {
    if args.first().map(String::as_str) == Some("colours") {
        return crate::colours::main(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("facets") {
        return crate::facets::main(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("edges") {
        return crate::edges::main(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("texels") {
        return crate::texels::main(&args[1..]);
    }
    let check = args.iter().any(|a| a == "--check");
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
                    eprintln!(
                        "no {REFERENCE} yet: `cargo review sizes` compares with it (plan/authenticity-review.md §3.4)"
                    );
                    return Ok(2);
                };
                let measures = checkglb::measures(&s.paths, &tops(&objects, &reference))?;
                let mut rows = size_rows(&objects, &measures, &reference);
                measure_plants(&s, &mut rows, &reference)?;
                let (mut text, files, code) = report(&objects, &rows, &reference);
                let manifest = serde_json::from_str(&s.read(surfaces::MANIFEST)?)
                    .map_err(|e| format!("{}: {e}", surfaces::MANIFEST))?;
                let surface_rows =
                    surfaces::rows(&reference.surfaces, &manifest, &s.read(surfaces::ROADS)?)?;
                text.push_str(&surfaces::report(&surface_rows));
                (text, files, code.max(baselined(&s.paths, &rows, args)?))
            }
            Some("uses") => {
                let actions = action_keys(&s.read("web/src/render/shared/characters/actions.ts")?)
                    .map_err(|e| format!("actions.ts: {e}"))?;
                let sounds = action_sounds(&s.read("web/src/audio/world.ts")?)
                    .map_err(|e| format!("world.ts: {e}"))?;
                let motions = motions(&s.paths)?;
                let (text, files, hard) =
                    uses(&objects, &motions, &actions, &sounds, reference.as_ref());
                let hard = if check { hard } else { Vec::new() };
                let code = if hard.is_empty() {
                    0
                } else {
                    eprintln!(
                        "review uses: {} hard flag(s), a use the renderer can't play or a model that opens without a clip:",
                        hard.len()
                    );
                    for line in hard.iter().take(20) {
                        eprintln!("  {line}");
                    }
                    1
                };
                (text, files, code)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, outside: usize, problem: Option<&str>) -> SizeRow {
        SizeRow {
            id: id.into(),
            outside: (0..outside).map(|i| format!("finding {i}")).collect(),
            problem: problem.map(String::from),
            ..SizeRow::default()
        }
    }

    fn base(text: &str) -> Baseline {
        Baseline::parse(text).unwrap()
    }

    #[test]
    fn check_judges_each_measured_row_by_the_baseline() {
        let b = base(r#"{"reference": {"a": 2, "c": 3, "gone": 1}, "edges": {"model.x": 4}}"#);
        let rows = [
            row("a", 2, None),
            row("b", 0, None),
            row("c", 0, Some("unmeasured: cannot read it")),
        ];
        assert_eq!(
            counts(&rows),
            BTreeMap::from([("a".to_string(), 2), ("b".to_string(), 0)])
        );
        // At its count: held; none and none held: clean; unmeasured: not judged; an id no row has: stale.
        assert_eq!(
            judged(&rows, &b),
            [
                "`gone`: in the baseline, but no object or model the review measured: take it out (`cargo review sizes --update-baseline`)"
            ]
        );
        let rows = [
            row("a", 3, None),
            row("b", 1, None),
            row("c", 0, Some("unmeasured: x")),
            row("gone", 0, None),
        ];
        let errors = judged(&rows, &b);
        assert_eq!(errors.len(), 3, "{errors:?}");
        assert!(errors[0].starts_with(
            "`a`: 3 outside its kind's range, 2 in the baseline: fix the new ones: finding 0; "
        ));
        assert!(
            errors[1]
                .starts_with("`b`: 1 outside its kind's range, none in the baseline: finding 0")
        );
        assert!(
            errors[2]
                .starts_with("`gone`: 0 outside its kind's range, 1 in the baseline: lower it")
        );
    }

    #[test]
    fn update_only_lowers_and_seed_only_starts() {
        let b = base(r#"{"reference": {"a": 2, "c": 3, "gone": 1}, "edges": {"model.x": 4}}"#);
        let rows = [
            row("a", 1, None),
            row("c", 0, Some("unmeasured: x")),
            row("new", 5, None),
        ];
        let (low, changes) = lowered(&rows, &b);
        // a lowered, gone out, c (unmeasured) kept, new not added, the gate's section untouched.
        assert_eq!(
            low,
            base(r#"{"reference": {"a": 1, "c": 3}, "edges": {"model.x": 4}}"#)
        );
        assert_eq!(
            changes,
            ["reference: a 2 -> 1", "reference: gone out (was 1)"]
        );
        assert!(seeded(&rows, &b).unwrap_err().contains("--update-baseline"));
        let start = seeded(&rows, &base(r#"{"edges": {"model.x": 4}}"#)).unwrap();
        assert_eq!(
            start,
            base(r#"{"edges": {"model.x": 4}, "reference": {"a": 1, "new": 5}}"#)
        );
        // Nothing found: no section (every row strict).
        let clean = seeded(&[row("a", 0, None)], &Baseline::default()).unwrap();
        assert_eq!(clean, Baseline::default());
    }

    #[test]
    fn the_shipped_reference_names_its_sources_and_its_species_are_species() {
        let root = checkglb::repo_root();
        let text = std::fs::read_to_string(format!("{root}/{REFERENCE}")).unwrap();
        let r = Reference::parse(&text).unwrap();
        for (name, k) in &r.kinds {
            for s in &k.sources {
                assert!(r.sources.contains_key(s), "{name}: no source `{s}`");
            }
            if let Some(sp) = &k.species {
                assert!(sp.problems().is_empty(), "{name}: {:?}", sp.problems());
                assert!(k.h.is_some(), "{name}: a species' kind has its height");
            }
        }
        for (id, kind) in &r.ids {
            assert!(r.kinds.contains_key(kind), "{id}: no kind `{kind}`");
        }
        // The game's twelve trees are a species each.
        for id in [
            "tree",
            "garden.maple",
            "garden.birch",
            "garden.cherry",
            "garden.apple",
            "garden.japaneseMaple",
            "garden.magnolia",
            "garden.willow",
            "pine",
            "garden.spruce",
            "garden.cypress",
            "garden.lemonTree",
        ] {
            let kind = r.ids.get(id).and_then(|k| r.kind(k));
            assert!(kind.is_some_and(|k| k.species.is_some()), "{id}");
        }
    }
}
