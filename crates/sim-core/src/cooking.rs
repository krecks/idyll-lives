//! Cooking that makes food: recipes, the food and dishes they leave, and what happens to them.
//!
//! A recipe (content `recipes`) is a chain of steps, each done at a kind of *station* (content
//! `cooking.stations`: the fridge for ingredients, a counter to prep, the stove, oven or grill to
//! cook, the sink or dishwasher to wash up). Objects say which stations they are (`stations`),
//! and interactions start a recipe (`recipe`): clicking the stove's "Cook dinner" sends the cook
//! to the fridge first, then a counter, then back to that stove (`world::TaskKind::Cook`).
//! Between steps the cook carries what the step leaves in their hands (`holds`: ingredients, a
//! prepped bowl). The last step serves the dish (an item, `items.rs`) on a free spot of a
//! surface near the stove, else in the fridge: as many servings as there are people at home,
//! within the recipe's `servings`. Housemates are told dinner's ready; whoever is hungry takes
//! a serving, sits down to eat (at a table if there is one) and leaves a dirty plate behind,
//! which someone clears away later: scraped into the bin, washed up and put away
//! (`TaskKind::Eat`, `TaskKind::Dishes`).
//!
//! Food goes off after a while (`spoilHours`; longer in the fridge), and dirty or spoilt things
//! make their room dirty (`update`). Recipes whose stations a home doesn't have are skipped
//! (optional steps) or not offered (required ones).

use std::collections::HashMap;

use serde::Deserialize;

use crate::Error;
use crate::content::{MAX_NEEDS, TagMask};
use crate::items::{Held, ItemAt, ItemInstance};
use crate::world::{ObjectInstance, Phase, TaskKind, World};

/// Most kinds of station content can name (`cooking.stations`), so a station set fits in a
/// `StationMask`.
pub const MAX_STATIONS: usize = 16;
/// Bitmask of indices into `CookingRules::stations`.
pub type StationMask = u16;
/// Stations when content names none.
const DEFAULT_STATIONS: [&str; 8] = ["ingredients", "prep", "stove", "oven", "grill", "microwave", "dishes", "bin"];

/// How cooking, eating and washing up go (content `cooking`).
#[derive(Debug, Clone)]
pub struct CookingRules {
    /// The kinds of station, in order (objects' `stations` and recipe steps' `at` name them).
    pub stations: Vec<String>,
    /// Game minutes until food on a surface goes off, and in a fridge.
    pub spoil_minutes: f32,
    pub stored_spoil_minutes: f32,
    /// How long eating a serving takes.
    pub eat_minutes: f32,
    /// How long washing one dirty thing takes, and how many are carried at once.
    pub dishes_minutes: f32,
    pub carry_dishes: u8,
    /// Leftovers are this much more tempting than cooking something new.
    pub leftover_preference: f32,
    /// Cooking scores this much more per hungry housemate (feeding the family).
    pub feed_family: f32,
    /// A need below this is hungry (for `feed_family` and the cook eating too).
    pub hungry_below: f32,
    /// Someone told dinner's ready wants to eat this much more, for `invite_minutes`.
    pub invited_boost: f32,
    pub invite_minutes: f32,
    /// How long a cook waits for a busy station before doing without it.
    pub station_wait_minutes: f32,
    /// Dirt per game hour that a dirty dish or spoilt food adds to its tile.
    pub dirty_dirt_per_hour: f32,
    /// Each step done without its station takes this share off the dish's quality.
    pub skip_penalty: f32,
    /// Seats to eat in: sitting interactions with any of these tags and none of `seat_avoid`.
    pub seat_tags: TagMask,
    pub seat_avoid: TagMask,
    /// Height (m) of a station's top when it has no `surface` (what the renderer cooks on).
    pub default_surface_height: f32,
    /// How far (m) from the last station a dish is served.
    pub serve_range: f32,
    /// Eating: what it's called, its tags (planned meals, traits) and animation.
    pub eat_label: String,
    pub eat_tags: TagMask,
    pub eat_anim: Option<usize>,
    /// Washing up: the same, and how keen residents are on it (like tidying up).
    pub dishes_label: String,
    pub dishes_tags: TagMask,
    pub dishes_anim: Option<usize>,
    pub dishes_interest: f32,
    /// Stations that wash up (`"dishes"`), and that a resident picks up dirty things to carry
    /// there from.
    pub dishes: StationMask,
    /// Clearing away: scraping the food off one thing (into a bin, else at the sink), and
    /// putting an armful of clean things away; their animations.
    pub scrape_minutes: f32,
    pub put_away_minutes: f32,
    pub scrape_anim: Option<usize>,
    pub put_away_anim: Option<usize>,
    /// Stations food is scraped into (`"bin"`), and clean things are put away at (`putAwayAt`:
    /// the counters' cupboards).
    pub bin: StationMask,
    pub put_away: StationMask,
}

impl CookingRules {
    /// The bit of station `name`, if content has it.
    pub fn station(&self, name: &str) -> Option<StationMask> {
        self.stations.iter().position(|s| s == name).map(|i| 1 << i)
    }
}

/// A thing residents carry or put down (content `items`): ingredients, a prepped bowl, a pot of
/// stew, a plate, a dirty plate. The renderer draws it from its `model`.
#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: String,
    pub label: String,
    /// Taking a serving gives one of these (a pot's servings are plates); none: the item itself.
    pub serving: Option<usize>,
    /// What's left once its last serving is gone (a dirty pot); none: nothing.
    pub empty_as: Option<usize>,
    /// Needs washing up.
    pub dirty: bool,
    /// Radius (m) of the disc it takes on a surface.
    pub radius: f32,
}

/// One step of a recipe: done at one of the stations `at` (first preferred), for `minutes`.
#[derive(Debug, Clone)]
pub struct RecipeStep {
    pub label: String,
    /// Station indices, in order of preference.
    pub at: Vec<u8>,
    pub mask: StationMask,
    pub minutes: f32,
    pub anim: Option<usize>,
    /// Done without when the home has none of its stations (the dish turns out plainer).
    pub optional: bool,
    /// What the cook carries away from it (an item).
    pub holds: Option<usize>,
}

/// Something to cook (content `recipes`); interactions start it with `recipe`.
#[derive(Debug, Clone)]
pub struct RecipeDef {
    pub id: String,
    pub label: String,
    /// The dish it makes (an item with servings).
    pub item: usize,
    /// Servings it makes: as many as there are people at home, within these.
    pub servings: [u8; 2],
    pub cost_per_serving: i64,
    /// What eating one serving gives, per need (times its quality).
    pub effects: [f32; MAX_NEEDS],
    /// Feeling for eating a serving of good quality (1 or better).
    pub eat_feeling: Option<usize>,
    /// The cook eats it straight away (a sandwich): one plate, nothing served.
    pub eat_now: bool,
    /// How long eating a serving takes (content `eatMinutes`, else `cooking.eatMinutes`).
    pub eat_minutes: f32,
    /// Cooked outdoors (a barbecue): outdoor stations and tables first.
    pub outdoor: bool,
    pub steps: Vec<RecipeStep>,
    /// All steps' minutes.
    pub minutes: f32,
    /// Made for an interaction's `eat` (`eat_recipe`), not cooked: no steps, never offered.
    pub hidden: bool,
}

impl RecipeDef {
    /// Whether a home with stations `mask` can make it (every required step has a station).
    pub fn possible(&self, mask: StationMask) -> bool {
        self.steps.iter().all(|s| s.optional || s.mask & mask != 0)
    }

    /// Minutes of the steps a home with stations `mask` does.
    pub fn minutes_with(&self, mask: StationMask) -> f32 {
        self.steps.iter().filter(|s| s.mask & mask != 0).map(|s| s.minutes).sum()
    }

    /// What `servings` servings cost.
    pub fn cost(&self, servings: u8) -> i64 {
        self.cost_per_serving * i64::from(servings)
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct CookingRaw {
    stations: Option<Vec<String>>,
    spoil_hours: Option<f32>,
    stored_spoil_hours: Option<f32>,
    eat_minutes: Option<f32>,
    dishes_minutes: Option<f32>,
    carry_dishes: Option<u8>,
    leftover_preference: Option<f32>,
    feed_family: Option<f32>,
    hungry_below: Option<f32>,
    invited_boost: Option<f32>,
    invite_minutes: Option<f32>,
    station_wait_minutes: Option<f32>,
    dirty_dirt_per_hour: Option<f32>,
    skip_penalty: Option<f32>,
    seat_tags: Option<Vec<String>>,
    seat_avoid: Option<Vec<String>>,
    default_surface_height: Option<f32>,
    serve_range: Option<f32>,
    eat_label: Option<String>,
    eat_tags: Option<Vec<String>>,
    eat_anim: Option<String>,
    dishes_label: Option<String>,
    dishes_tags: Option<Vec<String>>,
    dishes_anim: Option<String>,
    dishes_interest: Option<f32>,
    scrape_minutes: Option<f32>,
    put_away_minutes: Option<f32>,
    scrape_anim: Option<String>,
    put_away_anim: Option<String>,
    put_away_at: Option<Vec<String>>,
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ItemRaw {
    id: String,
    label: String,
    #[serde(default)]
    serving: Option<String>,
    #[serde(default)]
    empty_as: Option<String>,
    #[serde(default)]
    dirty: bool,
    #[serde(default)]
    radius: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecipeRaw {
    id: String,
    label: String,
    item: String,
    #[serde(default = "one_serving")]
    servings: [u8; 2],
    #[serde(default)]
    cost_per_serving: i64,
    #[serde(default)]
    effects: HashMap<String, f32>,
    #[serde(default)]
    eat_feeling: Option<String>,
    #[serde(default)]
    eat_now: bool,
    #[serde(default)]
    eat_minutes: Option<f32>,
    #[serde(default)]
    outdoor: bool,
    steps: Vec<StepRaw>,
}

fn one_serving() -> [u8; 2] {
    [1, 1]
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StepRaw {
    #[serde(default)]
    label: Option<String>,
    at: Vec<String>,
    minutes: f32,
    #[serde(default)]
    anim: Option<String>,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    holds: Option<String>,
}

/// What `parse` needs to look things up.
pub(crate) struct Lookups<'a> {
    pub needs: &'a HashMap<&'a str, usize>,
    pub tags: &'a HashMap<&'a str, usize>,
    pub feelings: &'a HashMap<String, usize>,
    pub animations: &'a [String],
}

/// Cooking rules, items and recipes from content.
pub(crate) fn parse(
    raw: &CookingRaw,
    items: &[ItemRaw],
    recipes: &[RecipeRaw],
    ix: &Lookups,
) -> Result<(CookingRules, Vec<ItemDef>, Vec<RecipeDef>), Error> {
    let stations: Vec<String> = raw
        .stations
        .clone()
        .unwrap_or_else(|| DEFAULT_STATIONS.iter().map(|s| (*s).to_owned()).collect());
    if stations.len() > MAX_STATIONS {
        return Err(Error::new(format!("cooking: at most {MAX_STATIONS} stations")));
    }
    if stations.iter().enumerate().any(|(i, s)| stations[..i].contains(s)) {
        return Err(Error::new("cooking: duplicate station"));
    }
    let tag_mask = |tags: &[String]| tags.iter().filter_map(|t| ix.tags.get(t.as_str())).fold(0, |m: TagMask, i| m | (1 << i));
    let anim = |a: &Option<String>, fallback: &str, ctx: &str| -> Result<Option<usize>, Error> {
        match a {
            Some(a) => ix
                .animations
                .iter()
                .position(|x| x == a)
                .map(Some)
                .ok_or_else(|| Error::new(format!("{ctx}: unknown anim '{a}' (declare it in \"animations\")"))),
            None => Ok(ix.animations.iter().position(|x| x == fallback)),
        }
    };
    let station_bit = |name: &str| stations.iter().position(|s| s == name).map(|i| 1 << i).unwrap_or(0);
    let mut put_away = 0;
    for name in raw.put_away_at.clone().unwrap_or_else(|| vec!["prep".into()]) {
        if !stations.contains(&name) {
            return Err(Error::new(format!("cooking.putAwayAt: unknown station '{name}'")));
        }
        put_away |= station_bit(&name);
    }
    let rules = CookingRules {
        spoil_minutes: raw.spoil_hours.unwrap_or(8.0).max(0.1) * 60.0,
        stored_spoil_minutes: raw.stored_spoil_hours.unwrap_or(72.0).max(0.1) * 60.0,
        eat_minutes: raw.eat_minutes.unwrap_or(25.0).max(1.0),
        dishes_minutes: raw.dishes_minutes.unwrap_or(5.0).max(1.0),
        carry_dishes: raw.carry_dishes.unwrap_or(3).max(1),
        leftover_preference: raw.leftover_preference.unwrap_or(1.3).max(0.0),
        feed_family: raw.feed_family.unwrap_or(0.15).max(0.0),
        hungry_below: raw.hungry_below.unwrap_or(0.5),
        invited_boost: raw.invited_boost.unwrap_or(2.5).max(0.0),
        invite_minutes: raw.invite_minutes.unwrap_or(60.0).max(0.0),
        station_wait_minutes: raw.station_wait_minutes.unwrap_or(10.0).max(0.0),
        dirty_dirt_per_hour: raw.dirty_dirt_per_hour.unwrap_or(0.02).max(0.0),
        skip_penalty: raw.skip_penalty.unwrap_or(0.05).clamp(0.0, 0.5),
        seat_tags: tag_mask(&raw.seat_tags.clone().unwrap_or_else(|| vec!["lounge".into(), "food".into()])),
        seat_avoid: tag_mask(&raw.seat_avoid.clone().unwrap_or_else(|| vec!["spa".into(), "hygiene".into(), "bathroom".into()])),
        default_surface_height: raw.default_surface_height.unwrap_or(0.9).max(0.0),
        serve_range: raw.serve_range.unwrap_or(6.0).max(1.0),
        eat_label: raw.eat_label.clone().unwrap_or_else(|| "Eat".into()),
        // (Tags content doesn't have are left out: eating then has none.)
        eat_tags: tag_mask(&raw.eat_tags.clone().unwrap_or_else(|| vec!["food".into()])),
        eat_anim: anim(&raw.eat_anim, "eat", "cooking.eatAnim")?,
        dishes_label: raw.dishes_label.clone().unwrap_or_else(|| "Wash the dishes".into()),
        dishes_tags: tag_mask(&raw.dishes_tags.clone().unwrap_or_else(|| vec!["chores".into(), "cleaning".into()])),
        dishes_anim: anim(&raw.dishes_anim, "wash", "cooking.dishesAnim")?,
        dishes_interest: raw.dishes_interest.unwrap_or(0.6).max(0.0),
        dishes: station_bit("dishes"),
        scrape_minutes: raw.scrape_minutes.unwrap_or(1.0).max(0.0),
        put_away_minutes: raw.put_away_minutes.unwrap_or(2.0).max(0.0),
        scrape_anim: anim(&raw.scrape_anim, "clean", "cooking.scrapeAnim")?,
        put_away_anim: anim(&raw.put_away_anim, "fetch", "cooking.putAwayAnim")?,
        bin: station_bit("bin"),
        put_away,
        stations: stations.clone(),
    };

    let item_index: HashMap<&str, usize> = items.iter().enumerate().map(|(i, it)| (it.id.as_str(), i)).collect();
    if item_index.len() != items.len() {
        return Err(Error::new("duplicate item id"));
    }
    let item = |id: &str, ctx: &str| {
        item_index.get(id).copied().ok_or_else(|| Error::new(format!("{ctx}: unknown item '{id}'")))
    };
    let mut item_defs = Vec::with_capacity(items.len());
    for it in items {
        let ctx = format!("item '{}'", it.id);
        item_defs.push(ItemDef {
            id: it.id.clone(),
            label: it.label.clone(),
            serving: it.serving.as_deref().map(|s| item(s, &ctx)).transpose()?,
            empty_as: it.empty_as.as_deref().map(|s| item(s, &ctx)).transpose()?,
            dirty: it.dirty,
            radius: it.radius.unwrap_or(crate::content::DEFAULT_RADIUS).clamp(0.02, 0.45),
        });
    }

    let mut recipe_defs = Vec::with_capacity(recipes.len());
    for r in recipes {
        let ctx = format!("recipe '{}'", r.id);
        if recipe_defs.iter().any(|d: &RecipeDef| d.id == r.id) {
            return Err(Error::new(format!("duplicate recipe '{}'", r.id)));
        }
        if r.steps.is_empty() {
            return Err(Error::new(format!("{ctx}: no steps")));
        }
        let [lo, hi] = r.servings;
        if lo == 0 || hi < lo {
            return Err(Error::new(format!("{ctx}: servings are [fewest, most], at least 1")));
        }
        if r.cost_per_serving < 0 {
            return Err(Error::new(format!("{ctx}: negative cost")));
        }
        let mut effects = [0.0; MAX_NEEDS];
        for (need, g) in &r.effects {
            effects[*ix.needs.get(need.as_str()).ok_or_else(|| Error::new(format!("{ctx}: unknown need '{need}'")))?] = *g;
        }
        let mut steps = Vec::with_capacity(r.steps.len());
        for (k, s) in r.steps.iter().enumerate() {
            let sctx = format!("{ctx} step {}", k + 1);
            if s.at.is_empty() || s.minutes <= 0.0 {
                return Err(Error::new(format!("{sctx}: needs a station (`at`) and minutes > 0")));
            }
            let mut at = Vec::with_capacity(s.at.len());
            for name in &s.at {
                let i = stations
                    .iter()
                    .position(|x| x == name)
                    .ok_or_else(|| Error::new(format!("{sctx}: unknown station '{name}'")))?;
                at.push(i as u8);
            }
            steps.push(RecipeStep {
                label: s.label.clone().unwrap_or_else(|| capitalised(&s.at[0])),
                mask: at.iter().fold(0, |m, &i| m | 1 << i),
                at,
                minutes: s.minutes,
                anim: anim(&s.anim, "cook", &sctx)?,
                optional: s.optional,
                holds: s.holds.as_deref().map(|h| item(h, &sctx)).transpose()?,
            });
        }
        let dish = item(&r.item, &ctx)?;
        recipe_defs.push(RecipeDef {
            id: r.id.clone(),
            label: r.label.clone(),
            item: dish,
            servings: [lo, hi],
            cost_per_serving: r.cost_per_serving,
            effects,
            eat_feeling: r
                .eat_feeling
                .as_deref()
                .map(|f| ix.feelings.get(f).copied().ok_or_else(|| Error::new(format!("{ctx}: unknown feeling '{f}'"))))
                .transpose()?,
            eat_now: r.eat_now,
            eat_minutes: r.eat_minutes.unwrap_or(rules.eat_minutes).max(1.0),
            outdoor: r.outdoor,
            minutes: steps.iter().map(|s| s.minutes).sum(),
            steps,
            hidden: false,
        });
    }
    Ok((rules, item_defs, recipe_defs))
}

/// The hidden recipe of interaction `ctx`'s `eat`: `item` (one serving, nothing to buy: the
/// interaction's own cost is paid), eaten in `minutes` for `effects`. Its id is the
/// interaction's (`object.interaction`), so a plate in someone's hands survives a reload.
pub(crate) fn eat_recipe(
    ctx: &str,
    label: &str,
    items: &[ItemDef],
    item: &str,
    minutes: f32,
    effects: [f32; MAX_NEEDS],
) -> Result<RecipeDef, Error> {
    let item = items
        .iter()
        .position(|i| i.id == item)
        .ok_or_else(|| Error::new(format!("{ctx}: eat: unknown item '{item}'")))?;
    if minutes <= 0.0 {
        return Err(Error::new(format!("{ctx}: eat: minutes must be > 0")));
    }
    Ok(RecipeDef {
        id: ctx.to_owned(),
        label: label.to_owned(),
        item,
        servings: [1, 1],
        cost_per_serving: 0,
        effects,
        eat_feeling: None,
        eat_now: true,
        eat_minutes: minutes,
        outdoor: false,
        steps: Vec::new(),
        minutes: 0.0,
        hidden: true,
    })
}

fn capitalised(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
}

// ---- Cooking, eating, washing up (a resident's step) --------------------------------------

use crate::content::{Content, Interaction, Pose};
use crate::items::Items;
use crate::surface::{self, SurfaceSpot};
use crate::world::{Access, Activity, Ctx, Sim, Task, access, affordable, end_activity, walk_route};

/// The tile a resident uses `obj` from (`surface::Links`, cached for the tick).
fn approach(ctx: &Ctx, obj: &ObjectInstance) -> (i32, i32) {
    ctx.index.links.approach_of(obj.id).unwrap_or_else(|| obj.front_tile(ctx.content))
}

/// Whether a task is one of the cooking chain's (they go on from one step to the next).
pub(crate) fn chain(kind: &TaskKind) -> bool {
    matches!(kind, TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. })
}

/// The interaction a recipe was started with (its skills, feeling and own effects count).
pub(crate) fn origin_interaction<'a>(
    content: &'a Content,
    objects: &[ObjectInstance],
    origin: Option<u32>,
    recipe: usize,
) -> Option<&'a Interaction> {
    let o = objects.get(origin? as usize)?;
    content.objects[o.def].interactions.iter().find(|i| i.recipe == Some(recipe))
}

/// The stations a resident has at home: working objects there. (A grill in the park is a
/// station only for a recipe started at it; see `find_station`.)
pub(crate) fn station_mask(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> StationMask {
    if sim.visiting.is_some() {
        return 0;
    }
    home_objects(ctx, sim.household)
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| !o.broken())
        .fold(0, |m, o| m | ctx.content.objects[o.def].stations)
}

/// Who's at home in the household (babies aside), the resident too.
fn at_home<'a>(sim: &Sim, ctx: &'a Ctx) -> impl Iterator<Item = usize> + 'a {
    let home = ctx.households[sim.household as usize].plot;
    let household = sim.household;
    ctx.briefs.iter().enumerate().filter_map(move |(j, b)| {
        (b.here && !b.away && !b.baby && b.household == household && home.is_some() && b.plot == home).then_some(j)
    })
}

/// Housemates at home who'd be glad of a serving of `r`.
pub(crate) fn hungry_housemates(sim: &Sim, ctx: &Ctx, r: &RecipeDef) -> usize {
    let below = ctx.content.cooking.hungry_below;
    let needs = ctx.content.needs.len();
    at_home(sim, ctx)
        .filter(|&j| j != sim.id as usize)
        .filter(|&j| (0..needs).any(|n| r.effects[n] > 0.1 && ctx.briefs[j].needs[n] < below))
        .count()
}

/// Whether a serving of `r` would do them good (at least half of it).
fn hungry_for(sim: &Sim, content: &Content, r: &RecipeDef) -> bool {
    (0..content.needs.len()).any(|n| r.effects[n] > 0.1 && sim.needs[n] + r.effects[n] * 0.5 <= 1.0)
}

/// The recipe of what they're eating (or on their way to eat).
pub(crate) fn eating(sim: &Sim, items: &Items) -> Option<usize> {
    let TaskKind::Eat { item, plate, .. } = sim.current.as_ref()?.task.kind else {
        return None;
    };
    sim.held
        .and_then(|h| h.recipe)
        .or_else(|| plate.and_then(|p| items.get(p)).and_then(|it| it.recipe))
        .or_else(|| item.and_then(|i| items.get(i)).and_then(|it| it.recipe))
}

/// A station for a step.
enum Found {
    At(u32),
    /// There is one, but someone's using it.
    Busy,
    Missing,
}

/// The station to go to for `mask`: a working one they may use with a free slot, the one the
/// recipe was started from first, then (for outdoor recipes) outdoor ones or (otherwise)
/// indoor ones, then by the step's order of stations, then the nearest.
fn find_station(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    mask: StationMask,
    order: &[u8],
    origin: Option<u32>,
    outdoor: bool,
) -> Found {
    let content = ctx.content;
    let lot = ctx.nav.lot;
    let mut best: Option<((u8, u8, usize), f32, u32)> = None;
    let mut busy = false;
    if sim.visiting.is_some() {
        return Found::Missing;
    }
    for &o in home_objects(ctx, sim.household).iter().chain(origin.iter()) {
        let obj = &objects[o as usize];
        let def = &content.objects[obj.def];
        if def.stations & mask == 0 || obj.broken() || access(ctx, sim, ctx.object_plot[o as usize]) != Access::Full {
            continue;
        }
        if !obj.can_use(sim.id, def.slots) {
            busy = true;
            continue;
        }
        let (fx, fz) = approach(ctx, obj);
        let outside = lot.in_bounds(fx, fz) && lot.room_at(fx, fz) == crate::lot::OUTDOORS;
        let rank = order.iter().position(|&i| def.stations & (1 << i) != 0).unwrap_or(order.len());
        let key = ((origin != Some(o)) as u8, (outside != outdoor) as u8, rank);
        let distance = ctx.nav.stairs.distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos);
        if best.is_none_or(|(k, d, _)| key < k || (key == k && distance < d)) {
            best = Some((key, distance, o));
        }
    }
    match best {
        Some((_, _, o)) => Found::At(o),
        None if busy => Found::Busy,
        None => Found::Missing,
    }
}

/// Starts (or carries on with) a cooking, eating or washing-up task.
pub(crate) fn start(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items) {
    match task.kind {
        TaskKind::Cook { .. } => cook_next(sim, task, ctx, objects, items, None, None),
        TaskKind::Eat { .. } => eat_next(sim, task, ctx, objects, items),
        TaskKind::Dishes { .. } => dishes_next(sim, task, ctx, objects, items, None),
        _ => {}
    }
}

/// Heads for `obj` (reserving a slot) for `task`; false if it can't be reached or is taken.
fn go_to(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance], obj: u32, tags: TagMask) -> bool {
    let content = ctx.content;
    let o = &objects[obj as usize];
    let (fx, fz) = approach(ctx, o);
    let Some(waypoints) = walk_route(ctx, sim, (fx, fz), [fx as f32 + 0.5, fz as f32 + 0.5]) else {
        return false;
    };
    let o = &mut objects[obj as usize];
    if !o.reserve(sim.id, content.objects[o.def].slots) {
        return false;
    }
    sim.pose = Pose::Stand;
    sim.current = Some(Activity { task, phase: Phase::Routing { waypoints, next: 0 }, tags });
    true
}

/// Walks to tile `goal` for `task`.
fn walk_to(sim: &mut Sim, task: Task, ctx: &Ctx, goal: (i32, i32), tags: TagMask) -> bool {
    let target = [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5];
    let Some(waypoints) = walk_route(ctx, sim, goal, target) else {
        return false;
    };
    sim.pose = Pose::Stand;
    sim.current = Some(Activity { task, phase: Phase::Routing { waypoints, next: 0 }, tags });
    true
}

/// The tiles to stand on to reach an item, best first: in front of what it's on or in, then
/// the free tiles round it nearest the item (a table with chairs pulled up to its front); on
/// the floor, its tile, then the free ones beside it.
fn item_access(ctx: &Ctx, objects: &[ObjectInstance], at: &ItemAt) -> Vec<(i32, i32)> {
    let nav = &ctx.nav;
    let free = |(x, z): (i32, i32)| nav.tile_free(x, z) && nav.stairs.step(x, z).is_none();
    let Some(o) = crate::items::holder(ctx.content, objects, at) else {
        let (x, z) = at.tile();
        return [(x, z), (x, z + 1), (x + 1, z), (x, z - 1), (x - 1, z)].into_iter().filter(|&t| free(t)).collect();
    };
    let obj = &objects[o as usize];
    let front = approach(ctx, obj);
    let footprint: Vec<(i32, i32)> = obj.tiles(ctx.content).collect();
    let storey = nav.stairs.storey(obj.z);
    let mut tiles: Vec<(i32, i32)> = if free(front) { vec![front] } else { Vec::new() };
    let mut round = Vec::new();
    for &(x, z) in &footprint {
        for t in [(x, z + 1), (x + 1, z), (x, z - 1), (x - 1, z)] {
            let open = nav.lot.in_bounds(t.0, t.1) && nav.lot.edge_between(x, z, t.0, t.1) == crate::lot::Edge::Open;
            if open && free(t) && nav.stairs.storey(t.1) == storey && !footprint.contains(&t) && t != front && !round.contains(&t) {
                round.push(t);
            }
        }
    }
    let c = at.centre();
    let d = |t: &(i32, i32)| (t.0 as f32 + 0.5 - c[0]).hypot(t.1 as f32 + 0.5 - c[1]);
    round.sort_by(|a, b| d(a).total_cmp(&d(b)));
    tiles.extend(round);
    tiles
}

/// Walks for `task` to the first of `goals` (a few at most) there's a way to.
fn walk_to_any(sim: &mut Sim, task: Task, ctx: &Ctx, goals: &[(i32, i32)], tags: TagMask) -> bool {
    goals.iter().take(4).any(|&goal| walk_to(sim, task, ctx, goal, tags))
}

/// Tags of a recipe being made (its interaction's, for "busy" checks and plans).
fn cook_tags(content: &Content, objects: &[ObjectInstance], origin: Option<u32>, recipe: usize) -> TagMask {
    origin_interaction(content, objects, origin, recipe).map_or(0, |i| i.tags)
}

/// Goes on to the recipe's next step that can be done (or serves it): to its station, or
/// waiting for a busy one (`waited` ticks so far). `last` is the station of the step just done.
/// The minutes of optional steps done without are carried in the task's `minutes` to the next
/// step that has a station (no counter: they chop by the stove).
fn cook_next(
    sim: &mut Sim,
    mut task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
    waited: Option<u32>,
    last: Option<u32>,
) {
    let content = ctx.content;
    sim.current = None;
    let TaskKind::Cook { recipe, step, origin, servings, skipped, minutes: carried, .. } = task.kind else {
        return;
    };
    let r = &content.recipes[recipe];
    let tags = cook_tags(content, objects, origin, recipe);
    // Before anything is bought: the home has to have every station the recipe needs.
    let at_origin = origin.and_then(|o| objects.get(o as usize)).map_or(0, |o| content.objects[o.def].stations);
    if step == 0 && servings == 0 && !r.possible(station_mask(sim, ctx, objects) | at_origin) {
        return cannot_cook(sim, ctx, tags);
    }
    let give_up = waited.is_some_and(|w| w as f32 * crate::MINUTES_PER_TICK >= content.cooking.station_wait_minutes);
    let (mut step, mut skipped, mut carried) = (step, skipped, carried);
    while let Some(s) = r.steps.get(step as usize) {
        task.kind = TaskKind::Cook { recipe, step, station: None, origin, servings, skipped, minutes: carried };
        match find_station(sim, ctx, objects, s.mask, &s.at, origin, r.outdoor) {
            Found::At(o) => {
                let at = Task { kind: TaskKind::Cook { recipe, step, station: Some(o), origin, servings, skipped, minutes: carried }, ..task };
                if go_to(sim, at, ctx, objects, o, tags) {
                    return;
                }
            }
            Found::Busy if !give_up => {
                sim.current = Some(Activity { task, phase: Phase::Queued { ticks: waited.unwrap_or(0) }, tags });
                return;
            }
            _ => {}
        }
        if !s.optional {
            return cannot_cook(sim, ctx, tags);
        }
        // Done without (no counter: chopped at the next station).
        carried = carried.saturating_add(s.minutes.round() as u16);
        step += 1;
        skipped += 1;
    }
    task.kind = TaskKind::Cook { recipe, step, station: None, origin, servings, skipped, minutes: 0 };
    serve(sim, task, ctx, objects, items, last);
}

/// Nowhere to make it: they think about wanting somewhere to cook.
fn cannot_cook(sim: &mut Sim, ctx: &Ctx, tags: TagMask) {
    let content = ctx.content;
    sim.current = None;
    let eat = content.cooking.eat_tags;
    let fits = |a: &crate::planner::ActivityDef, t: TagMask| a.tags & t != 0 && !a.social && !a.visit;
    let activity = content
        .activities
        .iter()
        .position(|a| fits(a, tags & !eat))
        .or_else(|| content.activities.iter().position(|a| fits(a, tags)));
    if let Some(a) = activity {
        crate::planner::think(sim, crate::planner::ThoughtKind::NoPlace, a, ctx.tick);
    }
}

/// How many servings to make: one each for whoever's at home, within the recipe's.
fn servings_for(sim: &Sim, ctx: &Ctx, r: &RecipeDef) -> u8 {
    (at_home(sim, ctx).count().clamp(1, 255) as u8).clamp(r.servings[0], r.servings[1])
}

/// Pays for the ingredients (how many servings); None if the money ran out.
fn buy_ingredients(sim: &mut Sim, ctx: &Ctx, r: &RecipeDef) -> Option<u8> {
    let n = servings_for(sim, ctx, r);
    let cost = r.cost(n);
    let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
    if !affordable(cost, funds) {
        return None;
    }
    sim.pending_spend += cost.max(0);
    Some(n)
}

/// The household's objects that things can be put on or in.
fn home_objects<'a>(ctx: &Ctx<'a>, household: u32) -> &'a [u32] {
    ctx.households[household as usize].plot.map_or(&[], |p| ctx.index.on(p))
}

/// Somewhere to put things: the objects that may carry them.
pub(crate) struct Place<'a> {
    pub content: &'a Content,
    pub objects: &'a [ObjectInstance],
    pub lot: &'a crate::lot::Lot,
    pub candidates: &'a [u32],
}

impl Place<'_> {
    /// The room at `near` (None off the lot).
    fn room_of(&self, near: [f32; 2]) -> Option<u16> {
        let (x, z) = (near[0].floor() as i32, near[1].floor() as i32);
        self.lot.in_bounds(x, z).then(|| self.lot.room_at(x, z))
    }

    /// Whether object `o` stands in `room` (None: anywhere).
    fn in_room(&self, o: &ObjectInstance, room: Option<u16>) -> bool {
        room.is_none_or(|r| self.lot.in_bounds(o.x, o.z) && self.lot.room_at(o.x, o.z) == r)
    }

    /// A free spot for something of `radius` near `near` (within `range`), on what stands in
    /// `room` (None: anywhere): on an outdoor surface first if `outdoor`, then on a table, bar or
    /// counter, else any.
    fn spot(&self, items: &Items, near: [f32; 2], range: f32, radius: f32, outdoor: bool, room: Option<u16>) -> Option<SurfaceSpot> {
        use crate::content::SurfaceKind;
        let occupied = items.occupancy(self.content);
        let candidates: Vec<u32> = self.candidates.iter().copied().filter(|&o| self.in_room(&self.objects[o as usize], room)).collect();
        let near_on = |candidates: &[u32], kinds: u8| {
            surface::free_surface_spot_near(self.content, self.objects, candidates, near, range, kinds, radius, &occupied)
        };
        if outdoor {
            let outside: Vec<u32> = candidates
                .iter()
                .copied()
                .filter(|&o| {
                    let c = self.objects[o as usize].centre(self.content);
                    let (x, z) = (c[0].floor() as i32, c[1].floor() as i32);
                    self.lot.in_bounds(x, z) && self.lot.room_at(x, z) == crate::lot::OUTDOORS
                })
                .collect();
            if let Some(s) = near_on(&outside, 0) {
                return Some(s);
            }
        }
        let tops = SurfaceKind::Table.bit() | SurfaceKind::Bar.bit() | SurfaceKind::Counter.bit();
        near_on(&candidates, tops).or_else(|| near_on(&candidates, 0))
    }

    /// The nearest object in `room` (None: anywhere) that keeps food (a fridge).
    fn store(&self, near: [f32; 2], room: Option<u16>) -> Option<&ObjectInstance> {
        let d = |o: &ObjectInstance| {
            let c = o.centre(self.content);
            (c[0] - near[0]).hypot(c[1] - near[1])
        };
        self.candidates
            .iter()
            .map(|&o| &self.objects[o as usize])
            .filter(|o| self.content.objects[o.def].stores && self.in_room(o, room))
            .min_by(|a, b| d(a).total_cmp(&d(b)))
    }

    /// Puts down what someone carries: food on a free surface near `near` (else in the fridge,
    /// else on the floor), dirty dishes on a surface or the floor; ingredients and bowls are
    /// used up. A surface or fridge in the same room as `near` comes first (not one through the
    /// wall that happens to be nearer). `prefer` is a spot to try first. Returns the (first) item
    /// put down.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn put_down(
        &self,
        items: &mut Items,
        held: Held,
        household: u32,
        near: [f32; 2],
        range: f32,
        outdoor: bool,
        mut prefer: Option<SurfaceSpot>,
        tick: u64,
    ) -> Option<u32> {
        let def = &self.content.items[held.def];
        let food = held.recipe.is_some() && held.servings > 0;
        if !food && !def.dirty {
            return None;
        }
        // In this room first, then anywhere.
        let here = self.room_of(near);
        let rooms = [here, None];
        let rooms = &rooms[..if here.is_some() { 2 } else { 1 }];
        // Dirty things one by one (a stack of plates is so many plates).
        let count = if food { 1 } else { held.servings.max(1) };
        let mut first = None;
        for _ in 0..count {
            let at = prefer
                .take()
                .map(|spot| ItemAt::on(&spot))
                .or_else(|| {
                    rooms.iter().find_map(|&room| {
                        self.spot(items, near, range, def.radius, outdoor, room).map(|spot| ItemAt::on(&spot)).or_else(|| {
                            self.store(near, room).filter(|_| food).map(|o| ItemAt::Stored { x: o.x, z: o.z })
                        })
                    })
                })
                .unwrap_or_else(|| ItemAt::Floor { x: 0, z: 0, offset: [0, 0] }.moved_to(near));
            let id = items.add(ItemInstance {
                id: 0,
                def: held.def,
                at,
                servings: if food { held.servings } else { 0 },
                quality: held.quality,
                made_at: if food { held.made_at } else { tick },
                household,
                spoiled: false,
                recipe: held.recipe.filter(|_| food),
            });
            first.get_or_insert(id);
        }
        first
    }
}

fn place<'a>(ctx: &Ctx<'a>, objects: &'a [ObjectInstance], household: u32) -> Place<'a> {
    Place { content: ctx.content, objects, lot: ctx.nav.lot, candidates: home_objects(ctx, household) }
}

/// The last step is done: the dish is served near the cook (or, made to eat at once, they keep
/// the plate and sit down with it).
fn serve(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items, last: Option<u32>) {
    let content = ctx.content;
    let TaskKind::Cook { recipe, origin, servings, skipped, .. } = task.kind else {
        return;
    };
    sim.current = None;
    let r = &content.recipes[recipe];
    // (Every step done without its station: the ingredients are bought now.)
    let servings = match servings {
        0 => match buy_ingredients(sim, ctx, r) {
            Some(n) => n,
            None => return,
        },
        n => n,
    };
    let inter = origin_interaction(content, objects, origin, recipe);
    let skill = inter.map_or(1.0, |i| i.skill_factor(&content.skill_rules, &sim.skills));
    let tools = last.and_then(|o| objects.get(o as usize)).map_or(1.0, |o| content.object_rules.quality_factor(o.quality));
    let quality = (skill * tools * (1.0 - content.cooking.skip_penalty * f32::from(skipped))).max(0.3);
    if let Some(i) = inter {
        if let Some(f) = i.earns_feeling(i.minutes, &sim.skills) {
            crate::social::add_feeling(&mut sim.feelings, f, &content.feelings, ctx.tick);
        }
        if let Some(o) = last.and_then(|o| objects.get(o as usize)).filter(|_| i.dirt > 0.0) {
            let (fx, fz) = approach(ctx, o);
            sim.mess = Some((fx, fz, i.dirt));
        }
    }
    if r.eat_now {
        return eat_now(sim, task, recipe, quality, ctx, objects, items);
    }
    let made = Held { def: r.item, servings, quality, made_at: ctx.tick, recipe: Some(recipe) };
    let at = place(ctx, objects, sim.household);
    let Some(id) = at.put_down(items, made, sim.household, sim.pos, content.cooking.serve_range, r.outdoor, None, ctx.tick) else {
        return;
    };
    sim.served = Some(id);
    // Hungry themselves: they have some too.
    if hungry_for(sim, content, r) {
        let eat = Task { kind: TaskKind::Eat { item: Some(id), seat: None, plate: None, stage: 0 }, ..task };
        eat_next(sim, eat, ctx, objects, items);
    }
}

/// Made to be eaten at once (a sandwich, a snack from the fridge; `quality` as a cooked
/// dish's): they hold a plate of `recipe` and sit down with it, at a table if there is one.
pub(crate) fn eat_now(
    sim: &mut Sim,
    task: Task,
    recipe: usize,
    quality: f32,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut Items,
) {
    let content = ctx.content;
    let r = &content.recipes[recipe];
    let plate = content.items[r.item].serving.unwrap_or(r.item);
    sim.held = Some(Held { def: plate, servings: 1, quality, made_at: ctx.tick, recipe: Some(recipe) });
    let eat = Task { kind: TaskKind::Eat { item: None, seat: None, plate: None, stage: 1 }, ..task };
    eat_next(sim, eat, ctx, objects, items);
}

/// Where someone eats: on the plot they're on (at home, at a host's, in a park). Its objects,
/// and whose plates are left there: the household living there (a guest's dirty plate is the
/// host's to wash up), else nobody's (`None`: nothing is left behind).
fn dining<'a>(sim: &Sim, ctx: &Ctx<'a>) -> (&'a [u32], Option<u32>) {
    let home = ctx.households[sim.household as usize].plot;
    let Some(here) = ctx.briefs.get(sim.id as usize).and_then(|b| b.plot) else {
        return (home_objects(ctx, sim.household), Some(sim.household));
    };
    if home == Some(here) {
        return (ctx.index.on(here), Some(sim.household));
    }
    (ctx.index.on(here), ctx.households.iter().position(|h| h.plot == Some(here)).map(|h| h as u32))
}

/// The best free place to sit and eat: a chair at a table (`surface::dining_seat`), else a
/// table with seats of its own, else any seat.
fn eat_seat(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> Option<u32> {
    let content = ctx.content;
    let rules = &content.cooking;
    let candidates: Vec<u32> = dining(sim, ctx)
        .0
        .iter()
        .copied()
        .filter(|&o| {
            let obj = &objects[o as usize];
            !obj.broken() && obj.can_use(sim.id, content.objects[obj.def].slots)
        })
        .collect();
    if let Some(d) = surface::dining_seat(content, objects, &ctx.index.links, &candidates, sim.pos) {
        return Some(d.object);
    }
    let sits = |o: u32, tags: TagMask| {
        content.objects[objects[o as usize].def]
            .interactions
            .iter()
            .any(|i| i.pose == Pose::Sit && i.tags & tags != 0 && i.tags & rules.seat_avoid == 0 && !i.baby)
    };
    let distance = |o: u32| {
        let (fx, fz) = approach(ctx, &objects[o as usize]);
        ctx.nav.stairs.distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos)
    };
    let nearest = |pick: &dyn Fn(u32) -> bool| {
        candidates.iter().copied().filter(|&o| pick(o)).min_by(|&a, &b| distance(a).total_cmp(&distance(b)))
    };
    nearest(&|o| content.objects[objects[o as usize].def].surface.is_some() && sits(o, rules.eat_tags))
        .or_else(|| nearest(&|o| sits(o, rules.seat_tags)))
}

/// Where the plate goes for someone sitting at `seat`: on the table in front of them, or on
/// the seat's own table.
fn plate_spot(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], items: &Items, seat: u32, radius: f32) -> Option<SurfaceSpot> {
    let content = ctx.content;
    let occupied = items.occupancy(content);
    let slot = objects[seat as usize].slot_of(sim.id).unwrap_or(0);
    surface::spot_for_seat(content, objects, &ctx.index.links, seat, slot, radius, &occupied).or_else(|| {
        content.objects[objects[seat as usize].def].surface?;
        surface::free_surface_spot_near(content, objects, &[seat], sim.pos, 2.5, 0, radius, &occupied)
    })
}

/// Eating, step by step: fetch a serving, then sit down with it (or eat standing).
fn eat_next(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items) {
    let content = ctx.content;
    let tags = content.cooking.eat_tags;
    let TaskKind::Eat { item, stage, .. } = task.kind else {
        return;
    };
    sim.current = None;
    if stage == 0 {
        let Some(it) = item.and_then(|i| items.get(i)).filter(|it| it.edible() && it.household == sim.household) else {
            return;
        };
        walk_to_any(sim, task, ctx, &item_access(ctx, objects, &it.at), tags);
        return;
    }
    if sim.held.is_none_or(|h| h.recipe.is_none()) {
        return;
    }
    if let Some(seat) = eat_seat(sim, ctx, objects) {
        let at = Task { kind: TaskKind::Eat { item, seat: Some(seat), plate: None, stage: 1 }, ..task };
        if go_to(sim, at, ctx, objects, seat, tags) {
            return;
        }
    }
    // Nowhere to sit: standing up.
    sim.pose = Pose::Stand;
    sim.current = Some(Activity {
        task: Task { kind: TaskKind::Eat { item, seat: None, plate: None, stage: 1 }, ..task },
        phase: Phase::Using { elapsed: 0.0 },
        tags,
    });
}

/// Takes a serving of `id` into their hands (false: it's gone or spoilt).
fn take_serving(sim: &mut Sim, content: &Content, items: &mut Items, id: u32) -> bool {
    let household = sim.household;
    let Some(it) = items.get_mut(id).filter(|it| it.edible() && it.household == household) else {
        return false;
    };
    let def = &content.items[it.def];
    it.servings -= 1;
    sim.held = Some(Held {
        def: def.serving.unwrap_or(it.def),
        servings: 1,
        quality: it.quality,
        made_at: it.made_at,
        recipe: it.recipe,
    });
    if it.servings == 0 {
        match def.empty_as.filter(|_| def.serving.is_some()) {
            // The empty pot stays to be washed up.
            Some(e) => {
                it.def = e;
                it.recipe = None;
            }
            // A plate of its own: it's in their hands now.
            None => {
                items.remove(id);
            }
        }
    }
    true
}

/// Clearing away, step by step: fetch what's to be cleared (stage 0); scrape the food off
/// into a bin (1; with none, it's done at the sink); wash up (2; `waited`: ticks waited for a
/// busy sink); put the clean things away at a counter (3; with none, from the sink).
fn dishes_next(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items, waited: Option<u32>) {
    let content = ctx.content;
    let rules = &content.cooking;
    let tags = rules.dishes_tags;
    let TaskKind::Dishes { item, stage, scraps, .. } = task.kind else {
        return;
    };
    sim.current = None;
    let at = |o: u32, stage: u8| Task { kind: TaskKind::Dishes { item, station: Some(o), stage, scraps }, ..task };
    match stage {
        0 => {
            let Some(it) = items.get(item).filter(|it| it.household == sim.household) else {
                return;
            };
            walk_to_any(sim, task, ctx, &item_access(ctx, objects, &it.at), tags);
        }
        1 => {
            if scraps > 0
                && let Found::At(o) = find_station(sim, ctx, objects, rules.bin, &[], None, false)
                && go_to(sim, at(o, 1), ctx, objects, o, tags)
            {
                return;
            }
            let sink = Task { kind: TaskKind::Dishes { item, station: None, stage: 2, scraps }, ..task };
            dishes_next(sim, sink, ctx, objects, items, None);
        }
        // Everything was thrown away: nothing to wash.
        2 if sim.held.is_none() && scraps == 0 => {}
        2 => {
            let give_up = waited.is_some_and(|w| w as f32 * crate::MINUTES_PER_TICK >= rules.station_wait_minutes);
            match find_station(sim, ctx, objects, rules.dishes, &[], None, false) {
                Found::At(o) => {
                    go_to(sim, at(o, 2), ctx, objects, o, tags);
                }
                Found::Busy if !give_up => {
                    sim.current = Some(Activity { task, phase: Phase::Queued { ticks: waited.unwrap_or(0) }, tags });
                }
                // Nowhere to wash up: what they carry is put down again (after the step).
                _ => {}
            }
        }
        _ => {
            if let Found::At(o) = find_station(sim, ctx, objects, rules.put_away, &[], None, false) {
                go_to(sim, at(o, 3), ctx, objects, o, tags);
            }
        }
    }
}

/// What clearing away `def` leaves to wash up: itself if it's dirty, else what it is empty (a
/// pot of stew, a dirty pot); none, it's thrown away whole.
fn washed(content: &Content, def: usize) -> Option<usize> {
    let d = &content.items[def];
    if d.dirty { Some(def) } else { d.empty_as }
}

/// How long a step of clearing away takes: scraping the food off `scraps` things (at the sink
/// too, if there was no bin), washing what they carry, or putting it away.
pub(crate) fn clearing_minutes(rules: &CookingRules, stage: u8, scraps: u8, held: Option<Held>) -> f32 {
    let scraping = rules.scrape_minutes * f32::from(scraps);
    match stage {
        1 => scraping,
        3 => rules.put_away_minutes,
        _ => rules.dishes_minutes * f32::from(held.map_or(0, |h| h.servings.max(1))) + scraping,
    }
}

/// Waiting for a busy station: try again (or, after a while, do without).
pub(crate) fn queued(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items, waited: u32) {
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return;
    };
    match task.kind {
        TaskKind::Cook { .. } => cook_next(sim, task, ctx, objects, items, Some(waited), None),
        TaskKind::Dishes { .. } => dishes_next(sim, task, ctx, objects, items, Some(waited)),
        _ => sim.current = None,
    }
}

/// Minutes of the recipe's optional steps done without that are spent at step `step` instead:
/// `carried`, those skipped since the step before (see `cook_next`), and when no later step has
/// a station in `mask`, all the later ones too (a fridge-only kitchen makes the whole meal at
/// the fridge).
fn extra_minutes(r: &RecipeDef, step: usize, mask: StationMask, carried: u16) -> f32 {
    let later = &r.steps[(step + 1).min(r.steps.len())..];
    let trailing = if later.iter().all(|s| s.optional && s.mask & mask == 0) { later.iter().map(|s| s.minutes).sum() } else { 0.0 };
    f32::from(carried) + trailing
}

/// Turns to face `o`, standing.
fn face(sim: &mut Sim, content: &Content, o: &ObjectInstance) {
    let c = o.centre(content);
    sim.yaw = (c[0] - sim.pos[0]).atan2(c[1] - sim.pos[1]);
    sim.pose = Pose::Stand;
}

/// Arrived where a cooking, eating or washing-up step happens.
pub(crate) fn arrive(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items) {
    let content = ctx.content;
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return;
    };
    match task.kind {
        TaskKind::Cook { recipe, step, station: Some(o), origin, servings, skipped, minutes: carried } => {
            let r = &content.recipes[recipe];
            // The ingredients are paid for at the first step; if the money ran out, never mind.
            let servings = match servings {
                0 => match buy_ingredients(sim, ctx, r) {
                    Some(n) => n,
                    None => return end_activity(sim, content, objects),
                },
                n => n,
            };
            let at_origin = origin.and_then(|o| objects.get(o as usize)).map_or(0, |o| content.objects[o.def].stations);
            let mask = station_mask(sim, ctx, objects) | at_origin;
            let minutes = r.steps[step as usize].minutes + extra_minutes(r, step as usize, mask, carried);
            face(sim, content, &objects[o as usize]);
            let act = sim.current.as_mut().expect("checked above");
            act.task.kind = TaskKind::Cook {
                recipe,
                step,
                station: Some(o),
                origin,
                servings,
                skipped,
                minutes: minutes.round().clamp(1.0, f32::from(u16::MAX)) as u16,
            };
            act.phase = Phase::Using { elapsed: 0.0 };
        }
        TaskKind::Eat { item: Some(id), stage: 0, .. } => {
            if !take_serving(sim, content, items, id) {
                sim.current = None;
                return;
            }
            // The last of it from the fridge: the empty pot comes out too, to be washed up.
            if items.get(id).is_some_and(|it| matches!(it.at, ItemAt::Stored { .. }) && it.to_clear(content))
                && let Some(empty) = items.remove(id)
            {
                let dirty = Held { servings: 1, ..Held::of(empty.def) };
                let at = place(ctx, objects, empty.household);
                at.put_down(items, dirty, empty.household, sim.pos, content.cooking.serve_range, false, None, ctx.tick);
            }
            let next = Task { kind: TaskKind::Eat { item: Some(id), seat: None, plate: None, stage: 1 }, ..task };
            eat_next(sim, next, ctx, objects, items);
        }
        TaskKind::Eat { item, seat: Some(seat), stage: 1, .. } => {
            let o = &objects[seat as usize];
            // They stand back up where they came from (see `end_activity`).
            sim.seated_from = Some(sim.tile());
            sim.pos = o.slot_position(content, o.slot_of(sim.id).unwrap_or(0));
            sim.yaw = o.yaw();
            sim.pose = Pose::Sit;
            // The plate goes on the table in front of them.
            let mut plate = None;
            if let Some(held) = sim.held
                && let Some(spot) = plate_spot(sim, ctx, objects, items, seat, content.items[held.def].radius)
            {
                let (candidates, household) = dining(sim, ctx);
                let at = Place { content, objects, lot: ctx.nav.lot, candidates };
                plate = at.put_down(items, held, household.unwrap_or(sim.household), sim.pos, 1.0, false, Some(spot), ctx.tick);
                sim.held = None;
            }
            let act = sim.current.as_mut().expect("checked above");
            act.task.kind = TaskKind::Eat { item, seat: Some(seat), plate, stage: 1 };
            act.phase = Phase::Using { elapsed: 0.0 };
        }
        TaskKind::Dishes { item, stage: 0, .. } => {
            // Everything to clear from the same place, up to an armful.
            let Some(target) = items.get(item).filter(|it| it.household == sim.household).cloned() else {
                sim.current = None;
                return;
            };
            let from = crate::items::holder(content, objects, &target.at);
            let same = |it: &ItemInstance| match from {
                Some(h) => crate::items::holder(content, objects, &it.at) == Some(h),
                None => it.at.tile() == target.at.tile(),
            };
            let mut picked: Vec<u32> = vec![item];
            for it in &items.list {
                if picked.len() >= usize::from(content.cooking.carry_dishes) {
                    break;
                }
                if it.id != item && it.household == sim.household && it.to_clear(content) && same(it) {
                    picked.push(it.id);
                }
            }
            // Food on them (eaten or not, gone off or not) is scraped off first; what's left is
            // carried dirty.
            let (mut scraps, mut dirty, mut carried) = (0u8, 0u8, None);
            for &id in &picked {
                let Some(it) = items.remove(id) else { continue };
                scraps += u8::from(it.servings > 0 || it.spoiled);
                if let Some(def) = washed(content, it.def) {
                    dirty += 1;
                    carried.get_or_insert(def);
                }
            }
            sim.held = carried.map(|def| Held { servings: dirty, ..Held::of(def) });
            let next = Task { kind: TaskKind::Dishes { item, station: None, stage: 1, scraps }, ..task };
            dishes_next(sim, next, ctx, objects, items, None);
        }
        TaskKind::Dishes { station: Some(o), .. } => {
            face(sim, content, &objects[o as usize]);
            if let Some(act) = sim.current.as_mut() {
                act.phase = Phase::Using { elapsed: 0.0 };
            }
        }
        _ => sim.current = None,
    }
}

/// A tick of cooking, eating or washing up (`minutes` long). Returns whether the step is done.
pub(crate) fn using(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items, elapsed: f32, minutes: f32) -> bool {
    let content = ctx.content;
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return true;
    };
    match task.kind {
        TaskKind::Cook { recipe, station, origin, minutes: total, .. } => {
            // What cooking does for the cook (fun, a skill), better with good tools.
            if let Some(inter) = origin_interaction(content, objects, origin, recipe) {
                let tools = station.and_then(|o| objects.get(o as usize)).map_or(1.0, |o| content.object_rules.quality_factor(o.quality));
                let boost = tools * inter.skill_factor(&content.skill_rules, &sim.skills);
                for n in 0..content.needs.len() {
                    let g = inter.gain_per_minute[n];
                    let g = if g > 0.0 { g * sim.mods.need_gain[n] * boost } else { g };
                    sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
                }
                for s in 0..content.skills.len() {
                    if inter.skill_gain[s] > 0.0 {
                        crate::world::practise(sim, content, s, inter.skill_gain[s] * boost, minutes);
                    }
                }
            }
            elapsed >= f32::from(total)
        }
        TaskKind::Eat { plate, .. } => {
            let food = sim.held.map(|h| (h.recipe, h.quality)).or_else(|| plate.and_then(|p| items.get(p)).map(|it| (it.recipe, it.quality)));
            let Some((Some(r), quality)) = food else {
                return true;
            };
            let r = &content.recipes[r];
            for n in 0..content.needs.len() {
                let g = r.effects[n] * quality / r.eat_minutes;
                let g = if g > 0.0 { g * sim.mods.need_gain[n] } else { g };
                sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
            }
            elapsed >= r.eat_minutes
        }
        TaskKind::Dishes { stage, scraps, .. } => elapsed >= clearing_minutes(&content.cooking, stage, scraps, sim.held),
        _ => true,
    }
}

/// Wears a station a little for a finished step (as a finished use does).
fn wear(sim: &mut Sim, content: &Content, objects: &mut [ObjectInstance], o: u32) {
    let Some(obj) = objects.get_mut(o as usize) else { return };
    let def = &content.objects[obj.def];
    let was = obj.broken();
    let w = def.wear_per_use * (1.0 - content.object_rules.wear_per_quality * f32::from(obj.quality));
    obj.wear = (obj.wear + w.max(0.0)).min(1.0);
    if !was && obj.broken() {
        sim.news = Some(crate::world::News::Broke(o));
    }
}

/// A step is done: on to the next one (another station, sitting down, the sink) or finished.
pub(crate) fn step_done(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items) {
    let content = ctx.content;
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return;
    };
    match task.kind {
        TaskKind::Cook { recipe, step, station, origin, servings, skipped, .. } => {
            if let Some(o) = station {
                objects[o as usize].release(sim.id);
                wear(sim, content, objects, o);
            }
            sim.held = content.recipes[recipe].steps[step as usize].holds.map(Held::of);
            let next = TaskKind::Cook { recipe, step: step + 1, station: None, origin, servings, skipped, minutes: 0 };
            cook_next(sim, Task { kind: next, ..task }, ctx, objects, items, None, station);
        }
        TaskKind::Eat { seat, plate, .. } => {
            let food = sim.held.map(|h| (h.recipe, h.quality)).or_else(|| plate.and_then(|p| items.get(p)).map(|it| (it.recipe, it.quality)));
            if let Some((Some(r), quality)) = food
                && quality >= 1.0
                && let Some(f) = content.recipes[r].eat_feeling
            {
                crate::social::add_feeling(&mut sim.feelings, f, &content.feelings, ctx.tick);
            }
            // A plate on the table is left there dirty; one in hand is put down.
            if let Some(p) = plate
                && let Some(it) = items.get_mut(p)
            {
                match content.items[it.def].empty_as {
                    Some(e) => {
                        it.def = e;
                        it.servings = 0;
                        it.recipe = None;
                        it.made_at = ctx.tick;
                    }
                    None => {
                        items.remove(p);
                    }
                }
            }
            let held = sim.held.take();
            let spot = held
                .and_then(|h| content.items[h.def].empty_as)
                .and_then(|e| seat.and_then(|s| plate_spot(sim, ctx, objects, items, s, content.items[e].radius)));
            // (Standing up first: a plate left on the floor goes where they stand.)
            end_activity(sim, content, objects);
            let (candidates, household) = dining(sim, ctx);
            if let Some(e) = held.and_then(|h| content.items[h.def].empty_as)
                && let Some(household) = household
            {
                let dirty = Held { servings: 1, ..Held::of(e) };
                let range = content.cooking.serve_range;
                let at = Place { content, objects, lot: ctx.nav.lot, candidates };
                at.put_down(items, dirty, household, sim.pos, range, false, spot, ctx.tick);
            }
        }
        TaskKind::Dishes { item, station, stage, .. } => {
            if stage >= 3 {
                return end_activity(sim, content, objects);
            }
            if let Some(o) = station {
                objects[o as usize].release(sim.id);
            }
            // Scraped: on to the sink. Washed: the clean things are put away (out of hand now:
            // called away, they're away already).
            if stage == 2 {
                sim.held = None;
                if let Some(o) = station {
                    wear(sim, content, objects, o);
                }
            }
            let next = Task { kind: TaskKind::Dishes { item, station: None, stage: stage + 1, scraps: 0 }, ..task };
            dishes_next(sim, next, ctx, objects, items, None);
        }
        _ => end_activity(sim, content, objects),
    }
}

// ---- The household's food (after every step) ------------------------------------------------

/// After residents stepped: dinner's ready for the households of those who served, things
/// carried by people who stopped cooking, eating or washing up are put down, and (every
/// `ROOM_REFRESH_TICKS`) food goes off, dirty things dirty their room and things whose place
/// is gone go too.
pub(crate) fn after_step(w: &mut World) {
    if w.sims.iter().any(|s| s.served.is_some() || s.held.is_some()) {
        w.refresh_index();
        invite(w);
        settle(w);
    }
    if w.tick.is_multiple_of(crate::world::ROOM_REFRESH_TICKS) {
        update(w);
    }
}

/// "Dinner's ready": everyone else at home, awake and not busy with what the player asked for,
/// is keener to eat for a while (and drops what they chose themselves if they're hungry).
fn invite(w: &mut World) {
    let tick = w.tick;
    for i in 0..w.sims.len() {
        let Some(item) = w.sims[i].served.take() else { continue };
        let Some(recipe) = w.items.get(item).and_then(|it| it.recipe) else { continue };
        let household = w.sims[i].household;
        let Some(home) = w.households[household as usize].plot else { continue };
        let World { sims, objects, content, plots, stairs, .. } = &mut *w;
        let until = tick + crate::social::minutes_to_ticks(content.cooking.invite_minutes);
        let r = &content.recipes[recipe];
        for (j, s) in sims.iter_mut().enumerate() {
            let at_home = s.tile();
            let at_home = plots[home as usize].contains(at_home.0, stairs.ground_z(at_home.1));
            if j == i
                || s.household != household
                || !s.here()
                || s.away_until.is_some()
                || !at_home
                || content.life.baby(s.age)
                || s.asleep(content)
                || s.queue.front().is_some_and(|t| t.directed)
                || s.current.as_ref().is_some_and(|a| a.task.directed)
            {
                continue;
            }
            s.invited_until = until;
            crate::planner::think(s, crate::planner::ThoughtKind::DinnerReady, recipe, tick);
            // Hungry, and only doing something they picked themselves: they come at once.
            let picked = s.current.as_ref().is_some_and(|a| {
                matches!(a.task.kind, TaskKind::Use { .. } | TaskKind::MoveTo { .. } | TaskKind::Clean { .. })
                    && a.tags & content.day_rhythm.sleep_tags == 0
            });
            if picked && hungry_for(s, content, r) {
                end_activity(s, content, objects);
            }
        }
    }
}

/// Puts down what people carry who are no longer cooking, eating or washing up (cancelled,
/// called away, off to work, gone). A recipe about to go on (loaded from a save) keeps it.
pub(crate) fn settle(w: &mut World) {
    w.refresh_index();
    for i in 0..w.sims.len() {
        let s = &w.sims[i];
        let Some(held) = s.held else { continue };
        let going_on = match &s.current {
            Some(a) => chain(&a.task.kind),
            None => s.queue.front().is_some_and(|t| matches!(t.kind, TaskKind::Cook { .. })),
        };
        if s.here() && going_on {
            continue;
        }
        w.sims[i].held = None;
        if !w.sims[i].here() {
            continue;
        }
        let (household, pos) = (w.sims[i].household, w.sims[i].pos);
        let candidates = w.households[household as usize].plot.map_or(&[][..], |p| w.object_index.on(p));
        let at = Place { content: &w.content, objects: &w.objects, lot: &w.lot, candidates };
        at.put_down(&mut w.items, held, household, pos, 4.0, false, None, w.tick);
    }
}

/// Every `ROOM_REFRESH_TICKS`: food goes off, dirty things make their tile dirtier, and things
/// whose place is gone (their table sold, their household moved away) go too.
fn update(w: &mut World) {
    let content = &w.content;
    let rules = &content.cooking;
    let tick = w.tick;
    let mut changed = false;
    let minutes = crate::world::ROOM_REFRESH_TICKS as f32 * crate::MINUTES_PER_TICK;
    let mut keep = Vec::with_capacity(w.items.list.len());
    for mut it in std::mem::take(&mut w.items.list) {
        let household = w.households.get(it.household as usize);
        let home = household.and_then(|h| h.plot);
        let (x, z) = it.at.tile();
        let placed = match it.at {
            ItemAt::Floor { .. } => w.lot.in_bounds(x, z),
            _ => crate::items::holder(content, &w.objects, &it.at).is_some(),
        };
        let lives_here = w.sims.iter().any(|s| s.here() && s.household == it.household);
        let on_home = home.is_some_and(|p| w.plots[p as usize].contains(x, w.stairs.ground_z(z)));
        if !placed || !lives_here || !on_home {
            changed = true;
            continue;
        }
        if it.edible() {
            let limit = if matches!(it.at, ItemAt::Stored { .. }) { rules.stored_spoil_minutes } else { rules.spoil_minutes };
            if (tick.saturating_sub(it.made_at)) as f32 * crate::MINUTES_PER_TICK >= limit {
                it.spoiled = true;
                changed = true;
            }
        }
        if it.to_clear(content) && !matches!(it.at, ItemAt::Stored { .. }) && w.lot.in_bounds(x, z) {
            let t = w.lot.tile_index(x, z);
            w.dirt[t] = (w.dirt[t] + rules.dirty_dirt_per_hour * minutes / 60.0).min(1.0);
            w.room_tiles.touch(t);
        }
        keep.push(it);
    }
    w.items.list = keep;
    if changed {
        w.items.version += 1;
    }
}

impl World {
    /// Items on or in `old` follow it to where it stands now (`new`, the same object moved or
    /// turned), turning with it.
    pub(crate) fn carry_items(&mut self, old: &ObjectInstance, new: &ObjectInstance) {
        let content = &self.content;
        let (from, to) = (old.centre(content), new.centre(content));
        let turn = (i32::from(new.rot) - i32::from(old.rot)).rem_euclid(4);
        let tiles: Vec<(i32, i32)> = old.tiles(content).collect();
        let mut moved = false;
        for it in &mut self.items.list {
            if matches!(it.at, ItemAt::Floor { .. }) || !tiles.contains(&it.at.tile()) {
                continue;
            }
            let c = it.at.centre();
            // Where it stood on the old top, on the new one (as things standing on it move).
            let p = match (surface::Top::of(content, old), surface::Top::of(content, new)) {
                (Some(a), Some(b)) => b.world(a.local(c)),
                _ => {
                    let (dx, dz) = (c[0] - from[0], c[1] - from[1]);
                    // A quarter turn (rot + 1) takes local +z (the front) to +x.
                    let (rx, rz) = match turn {
                        0 => (dx, dz),
                        1 => (dz, -dx),
                        2 => (-dx, -dz),
                        _ => (-dz, dx),
                    };
                    [to[0] + rx, to[1] + rz]
                }
            };
            it.at = it.at.moved_to(p);
            if let ItemAt::Stored { .. } = it.at {
                it.at = ItemAt::Stored { x: new.x, z: new.z };
            }
            moved = true;
        }
        if moved {
            self.items.version += 1;
        }
    }

    /// Items on or in `obj` go with it (it's being sold).
    pub(crate) fn drop_items_with(&mut self, obj: &ObjectInstance) {
        let tiles: Vec<(i32, i32)> = obj.tiles(&self.content).collect();
        let before = self.items.list.len();
        self.items.list.retain(|it| matches!(it.at, ItemAt::Floor { .. }) || !tiles.contains(&it.at.tile()));
        if self.items.list.len() != before {
            self.items.version += 1;
        }
    }

    /// Items on `tiles` move by `(dx, dz)` with the room they're in (`World::move_room`).
    pub(crate) fn shift_items(&mut self, tiles: &[(i32, i32)], dx: i32, dz: i32) {
        let mut moved = false;
        for it in &mut self.items.list {
            if tiles.contains(&it.at.tile()) {
                let c = it.at.centre();
                it.at = it.at.moved_to([c[0] + dx as f32, c[1] + dz as f32]);
                moved = true;
            }
        }
        if moved {
            self.items.version += 1;
        }
    }
}
