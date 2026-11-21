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
use crate::content::{Content, Interaction, Pose};
use crate::content::{MAX_NEEDS, TagMask};
use crate::items::Items;
use crate::items::{Held, ItemAt, ItemInstance};
use crate::surface::{self, SurfaceSpot};
use crate::world::{
    Access, Activity, Ctx, Sim, Task, access, affordable, end_activity, walk_route,
};
use crate::world::{ObjectInstance, Phase, TaskKind, World};

/// Most kinds of station content can name (`cooking.stations`), so a station set fits in a
/// `StationMask`.
pub const MAX_STATIONS: usize = 16;
/// Bitmask of indices into `CookingRules::stations`.
pub type StationMask = u16;
/// Stations when content names none.
const DEFAULT_STATIONS: [&str; 8] = [
    "ingredients",
    "prep",
    "stove",
    "oven",
    "grill",
    "microwave",
    "dishes",
    "bin",
];

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
    /// How far (m) from the last station the cook carries a dish to a dining table (one with a
    /// chair pulled up to it, or seats of its own) to serve it there; with none that near, it is
    /// served near the station (`serve_range`). Putting it down takes `serve_minutes`, with
    /// `serve_anim`; the action panel calls it `serve_label`.
    pub table_range: f32,
    pub serve_minutes: f32,
    pub serve_anim: Option<usize>,
    pub serve_label: String,
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
    /// Picking the armful up off the table (bent over it, `serve_anim`, as a dish is put down).
    pub pick_up_minutes: f32,
    pub scrape_anim: Option<usize>,
    pub put_away_anim: Option<usize>,
    /// Stations food is scraped into (`"bin"`), and clean things are put away at (`putAwayAt`:
    /// the cupboards, the counters' cupboards), the nearest, on a tie the first listed (a wall
    /// cabinet over a counter): `put_away_order`, as station indices.
    pub bin: StationMask,
    pub put_away: StationMask,
    pub put_away_order: Vec<u8>,
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
    /// Two or more of these carried at once are shown as this (a stack of dirty plates); none:
    /// as one of them.
    pub stack_as: Option<usize>,
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
    /// The animation at some of its stations instead (content `animAt`: a meal's Cook step put in the
    /// oven or the microwave, not stirred), by station index.
    pub anim_at: Vec<(u8, Option<usize>)>,
    /// Done without when the home has none of its stations (the dish turns out plainer).
    pub optional: bool,
    /// What the cook carries away from it (an item).
    pub holds: Option<usize>,
}

impl RecipeStep {
    /// The animation at a station object that is stations `mask`: `animAt`'s for the first of the
    /// step's stations (`at`, in order) it is (a cooker is a stove before an oven), else the step's own.
    pub fn anim_for(&self, mask: StationMask) -> Option<usize> {
        let at = self.at.iter().find(|&&i| mask & (1 << i) != 0);
        at.and_then(|i| self.anim_at.iter().find(|(s, _)| s == i))
            .map_or(self.anim, |&(_, a)| a)
    }
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
        self.steps
            .iter()
            .filter(|s| s.mask & mask != 0)
            .map(|s| s.minutes)
            .sum()
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
    table_range: Option<f32>,
    serve_minutes: Option<f32>,
    serve_anim: Option<String>,
    serve_label: Option<String>,
    eat_label: Option<String>,
    eat_tags: Option<Vec<String>>,
    eat_anim: Option<String>,
    dishes_label: Option<String>,
    dishes_tags: Option<Vec<String>>,
    dishes_anim: Option<String>,
    dishes_interest: Option<f32>,
    scrape_minutes: Option<f32>,
    put_away_minutes: Option<f32>,
    pick_up_minutes: Option<f32>,
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
    stack_as: Option<String>,
    #[serde(default)]
    radius: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecipeRaw {
    pub(crate) id: String,
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
    /// Life-stage ids that may cook it, for every interaction that starts it without `stages`
    /// of its own (see `Interaction::stages`): cooking at a hot stove is for teens and up.
    #[serde(default)]
    pub(crate) stages: Option<Vec<String>>,
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
    anim_at: HashMap<String, String>,
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
        return Err(Error::new(format!(
            "cooking: at most {MAX_STATIONS} stations"
        )));
    }
    if stations
        .iter()
        .enumerate()
        .any(|(i, s)| stations[..i].contains(s))
    {
        return Err(Error::new("cooking: duplicate station"));
    }
    // Tags content gives must exist; of the defaults, those content has (eating then may have none).
    let tag_mask =
        |tags: &Option<Vec<String>>, default: &[&str], ctx: &str| -> Result<TagMask, Error> {
            match tags {
                Some(tags) => tags.iter().try_fold(0, |m: TagMask, t| {
                    let i = ix
                        .tags
                        .get(t.as_str())
                        .ok_or_else(|| Error::new(format!("{ctx}: unknown tag '{t}'")))?;
                    Ok(m | (1 << i))
                }),
                None => Ok(default
                    .iter()
                    .filter_map(|t| ix.tags.get(t))
                    .fold(0, |m: TagMask, i| m | (1 << i))),
            }
        };
    let finite = |v: f32, ctx: &str| {
        if v.is_finite() {
            Ok(v)
        } else {
            Err(Error::new(format!("{ctx} must be a number")))
        }
    };
    let anim = |a: &Option<String>, fallback: &str, ctx: &str| -> Result<Option<usize>, Error> {
        match a {
            Some(a) => ix
                .animations
                .iter()
                .position(|x| x == a)
                .map(Some)
                .ok_or_else(|| {
                    Error::new(format!(
                        "{ctx}: unknown anim '{a}' (declare it in \"animations\")"
                    ))
                }),
            None => Ok(ix.animations.iter().position(|x| x == fallback)),
        }
    };
    let station_bit = |name: &str| {
        stations
            .iter()
            .position(|s| s == name)
            .map(|i| 1 << i)
            .unwrap_or(0)
    };
    let mut put_away = 0;
    let mut put_away_order = Vec::new();
    for name in raw
        .put_away_at
        .clone()
        .unwrap_or_else(|| vec!["prep".into()])
    {
        let Some(i) = stations.iter().position(|s| *s == name) else {
            return Err(Error::new(format!(
                "cooking.putAwayAt: unknown station '{name}'"
            )));
        };
        put_away |= station_bit(&name);
        put_away_order.push(i as u8);
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
        invite_minutes: finite(raw.invite_minutes.unwrap_or(60.0), "cooking.inviteMinutes")?
            .max(0.0),
        station_wait_minutes: finite(
            raw.station_wait_minutes.unwrap_or(10.0),
            "cooking.stationWaitMinutes",
        )?
        .max(0.0),
        dirty_dirt_per_hour: raw.dirty_dirt_per_hour.unwrap_or(0.02).max(0.0),
        skip_penalty: raw.skip_penalty.unwrap_or(0.05).clamp(0.0, 0.5),
        seat_tags: tag_mask(&raw.seat_tags, &["lounge", "food"], "cooking.seatTags")?,
        seat_avoid: tag_mask(
            &raw.seat_avoid,
            &["spa", "hygiene", "bathroom"],
            "cooking.seatAvoid",
        )?,
        default_surface_height: raw.default_surface_height.unwrap_or(0.9).max(0.0),
        serve_range: raw.serve_range.unwrap_or(6.0).max(1.0),
        table_range: finite(raw.table_range.unwrap_or(12.0), "cooking.tableRange")?.max(0.0),
        serve_minutes: finite(raw.serve_minutes.unwrap_or(1.0), "cooking.serveMinutes")?
            .clamp(1.0, 30.0),
        serve_anim: anim(&raw.serve_anim, "serve", "cooking.serveAnim")?,
        serve_label: raw.serve_label.clone().unwrap_or_else(|| "Serve it".into()),
        eat_label: raw.eat_label.clone().unwrap_or_else(|| "Eat".into()),
        eat_tags: tag_mask(&raw.eat_tags, &["food"], "cooking.eatTags")?,
        eat_anim: anim(&raw.eat_anim, "eat", "cooking.eatAnim")?,
        dishes_label: raw
            .dishes_label
            .clone()
            .unwrap_or_else(|| "Wash the dishes".into()),
        dishes_tags: tag_mask(
            &raw.dishes_tags,
            &["chores", "cleaning"],
            "cooking.dishesTags",
        )?,
        dishes_anim: anim(&raw.dishes_anim, "wash", "cooking.dishesAnim")?,
        dishes_interest: raw.dishes_interest.unwrap_or(0.6).max(0.0),
        dishes: station_bit("dishes"),
        scrape_minutes: raw.scrape_minutes.unwrap_or(1.0).max(0.0),
        put_away_minutes: raw.put_away_minutes.unwrap_or(2.0).max(0.0),
        pick_up_minutes: finite(raw.pick_up_minutes.unwrap_or(0.75), "cooking.pickUpMinutes")?
            .max(0.0),
        scrape_anim: anim(&raw.scrape_anim, "clean", "cooking.scrapeAnim")?,
        put_away_anim: anim(&raw.put_away_anim, "fetch", "cooking.putAwayAnim")?,
        bin: station_bit("bin"),
        put_away,
        put_away_order,
        stations: stations.clone(),
    };

    let item_index: HashMap<&str, usize> = items
        .iter()
        .enumerate()
        .map(|(i, it)| (it.id.as_str(), i))
        .collect();
    if item_index.len() != items.len() {
        return Err(Error::new("duplicate item id"));
    }
    let item = |id: &str, ctx: &str| {
        item_index
            .get(id)
            .copied()
            .ok_or_else(|| Error::new(format!("{ctx}: unknown item '{id}'")))
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
            stack_as: it.stack_as.as_deref().map(|s| item(s, &ctx)).transpose()?,
            radius: it
                .radius
                .unwrap_or(crate::content::DEFAULT_RADIUS)
                .clamp(0.02, 0.45),
        });
    }

    let mut recipe_defs = Vec::with_capacity(recipes.len());
    for r in recipes {
        let ctx = format!("recipe '{}'", r.id);
        if recipe_defs.iter().any(|d: &RecipeDef| d.id == r.id) {
            return Err(Error::new(format!("duplicate recipe '{}'", r.id)));
        }
        // (A cook counts the steps in a byte.)
        if r.steps.is_empty() || r.steps.len() > u8::MAX as usize {
            return Err(Error::new(format!("{ctx}: 1 to {} steps", u8::MAX)));
        }
        let [lo, hi] = r.servings;
        if lo == 0 || hi < lo {
            return Err(Error::new(format!(
                "{ctx}: servings are [fewest, most], at least 1"
            )));
        }
        if r.cost_per_serving < 0 {
            return Err(Error::new(format!("{ctx}: negative cost")));
        }
        let mut effects = [0.0; MAX_NEEDS];
        for (need, g) in &r.effects {
            effects[*ix
                .needs
                .get(need.as_str())
                .ok_or_else(|| Error::new(format!("{ctx}: unknown need '{need}'")))?] = *g;
        }
        let mut steps = Vec::with_capacity(r.steps.len());
        for (k, s) in r.steps.iter().enumerate() {
            let sctx = format!("{ctx} step {}", k + 1);
            if s.at.is_empty() || s.minutes <= 0.0 {
                return Err(Error::new(format!(
                    "{sctx}: needs a station (`at`) and minutes > 0"
                )));
            }
            let mut at = Vec::with_capacity(s.at.len());
            for name in &s.at {
                let i = stations
                    .iter()
                    .position(|x| x == name)
                    .ok_or_else(|| Error::new(format!("{sctx}: unknown station '{name}'")))?;
                at.push(i as u8);
            }
            let mut anim_at = Vec::with_capacity(s.anim_at.len());
            for (name, a) in &s.anim_at {
                let i = stations
                    .iter()
                    .position(|x| x == name)
                    .filter(|&i| at.contains(&(i as u8)))
                    .ok_or_else(|| {
                        Error::new(format!(
                            "{sctx}: animAt names '{name}', not one of its stations"
                        ))
                    })?;
                anim_at.push((i as u8, anim(&Some(a.clone()), "cook", &sctx)?));
            }
            steps.push(RecipeStep {
                label: s.label.clone().unwrap_or_else(|| capitalised(&s.at[0])),
                mask: at.iter().fold(0, |m, &i| m | 1 << i),
                at,
                minutes: s.minutes,
                anim: anim(&s.anim, "cook", &sctx)?,
                anim_at,
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
                .map(|f| {
                    ix.feelings
                        .get(f)
                        .copied()
                        .ok_or_else(|| Error::new(format!("{ctx}: unknown feeling '{f}'")))
                })
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
    c.next()
        .map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
}

mod cook;
mod dishes;
mod eat;
mod household;
mod serve;
mod step;

pub(crate) use cook::*;
pub(crate) use dishes::*;
pub(crate) use eat::*;
pub(crate) use household::*;
pub(crate) use serve::*;
pub(crate) use step::*;
