//! Objects as the review sees them, read from the content.

use super::*;

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
    /// How its uses are done, by animation tag (content `how`: a manner of the renderer's `MANNERS`).
    pub how: BTreeMap<String, String>,
    /// It hangs over a window, shut at dusk (content `overWindow`: curtains, blinds, shutters).
    pub over_window: bool,
    /// A seated use's hands have a top to rest on: its own (content `surface`), the desk or table it is pulled up
    /// to (`seatAt`), or the one it stands on (`layer: surface`, a laptop).
    pub top: bool,
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
            how: raw["how"]
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                .collect(),
            over_window: raw["overWindow"].as_bool().unwrap_or(false),
            top: raw.get("surface").is_some_and(|s| !s.is_null())
                || !strs(&raw["seatAt"]).is_empty()
                || raw["layer"].as_str() == Some("surface"),
        });
    }
    Ok(out)
}
