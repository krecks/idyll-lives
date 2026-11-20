//! The planner: each resident's weekly routines and life goals, the player's way to steer
//! residents without giving orders.
//!
//! **Routines** are blocks on a weekly calendar ("train strength 1 h at 18:00 Mon/Wed/Fri",
//! "sleep 23:00–07:00"). A block names an *activity* (tags, or "whatever trains this skill"),
//! never an object, so plans keep working when the house changes. While a block runs, its
//! activity becomes the resident's strong favourite: choices that fit are boosted, others are
//! damped, but urgent needs still win (`World::pick_autonomous`). Whether they stick to it
//! depends on their traits ("discipline"), mood and how much they like the activity: each block
//! may be skipped when it starts. Every block ends as kept, cut short, skipped or "no place
//! for it" (nothing at home or in a park offers the activity), with a reason; missing places
//! become *wishes* the player can build for. Sleep blocks decide when a resident's night is.
//!
//! **Goals** (get a job, reach a skill level, find love, make friends, save money...) bias
//! what residents choose and how hard they look for work, report their progress, and end with
//! a story event and a feeling. Goals higher on the list steer more, and goals of one kind count
//! once. Residents suggest goals of their own from their traits.
//!
//! Households have routine templates every member follows (each member may skip one); a
//! resident's own block wins where the two overlap.

use std::collections::{HashMap, VecDeque};

use serde::Deserialize;

use crate::Error;
use crate::clock;
use crate::content::{Content, Interaction, TagMask};
use crate::social::FeelingDef;
use crate::social::{self, EventKind};
use crate::world::{Layout, Phase, Sim, TaskKind, World};

// ---- Content ------------------------------------------------------------------------------

/// Something a routine block can be for.
#[derive(Debug, Clone)]
pub struct ActivityDef {
    pub id: String,
    pub label: String,
    pub icon: String,
    /// Interactions with any of these tags count (also: training a skill, below).
    pub tags: TagMask,
    /// A block names a skill; interactions that train it count.
    pub skill: bool,
    /// Socialising counts.
    pub social: bool,
    /// Visiting someone counts.
    pub visit: bool,
    /// The block decides when the resident's night is.
    pub sleep: bool,
}

impl ActivityDef {
    /// Whether using `inter` is doing this activity (with the block's `skill`, if any).
    pub fn fits(&self, inter: &Interaction, skill: Option<usize>) -> bool {
        match skill {
            Some(s) if self.skill => inter.skill_gain[s] > 0.0,
            _ => inter.tags & self.tags != 0,
        }
    }
}

/// How routines and goals behave (`"planner"` in content).
#[derive(Debug, Clone)]
pub struct PlannerRules {
    /// Multiplier on choices that fit the current block (at full adherence).
    pub boost: f32,
    /// Multiplier on other choices while a block runs (urgent needs excepted).
    pub off_block: f32,
    /// A need below this is urgent: choices that fill it are never damped.
    pub urgent_below: f32,
    /// Lowest score a fitting choice gets, so a planned workout happens even with full needs.
    pub floor: f32,
    /// Chance a block is skipped at average discipline and mood.
    pub skip_chance: f32,
    /// Share of a block's minutes spent on it to count as kept.
    pub kept_share: f32,
    /// Longest block (minutes); sleep blocks may be longer.
    pub max_minutes: u16,
    pub max_sleep_minutes: u16,
    /// Trait id → how well residents with it stick to plans (1 = average).
    pub discipline: HashMap<String, f32>,
    pub kept_feeling: Option<usize>,
    pub no_place_feeling: Option<usize>,
    pub goal_feeling: Option<usize>,
    /// Hour of the day goals are reviewed and new ones suggested.
    pub review_hour: f32,
    /// Most goals and suggestions a resident has at once.
    pub max_goals: usize,
    pub max_suggestions: usize,
    /// How much weaker each goal steers than the one above it on the list (see `goal_strength`).
    pub goal_falloff: f32,
    /// Days before an idea nobody answered is taken on anyway (residents with free will).
    pub suggestion_days: u32,
    /// Days a resident waits after being told no before asking for the same thing again.
    pub ask_again_days: u32,
    /// Getting something they wanted.
    pub wish_feeling: Option<usize>,
    /// Told no again for the same thing: the first on the second no, the next on the third,
    /// and so on (the last one from then on).
    pub denied_feelings: Vec<usize>,
}

impl Default for PlannerRules {
    fn default() -> Self {
        Self {
            boost: 6.0,
            off_block: 0.35,
            urgent_below: 0.15,
            floor: 0.15,
            skip_chance: 0.1,
            kept_share: 0.75,
            max_minutes: 240,
            max_sleep_minutes: 720,
            discipline: HashMap::new(),
            kept_feeling: None,
            no_place_feeling: None,
            goal_feeling: None,
            review_hour: 7.0,
            max_goals: 10,
            max_suggestions: 2,
            goal_falloff: 0.15,
            suggestion_days: 2,
            ask_again_days: 3,
            wish_feeling: None,
            denied_feelings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GoalKind {
    /// Have a job (in `category`, if set).
    HasJob,
    /// Reach job level `target` (1 = the first grade).
    JobLevel,
    /// One promotion from where they are now.
    Promoted,
    /// Reach `target` in `skill`.
    Skill,
    /// Have `target` friends (friendship 50+).
    Friends,
    /// Have a partner.
    Partner,
    /// Household funds of `target` dollars.
    Funds,
    /// Have `object` at home (saving up to `target`, its price).
    Own,
}

#[derive(Debug, Clone)]
pub struct GoalDef {
    pub id: String,
    /// With `{skill}`, `{n}` (the target) and `{category}` filled in.
    pub label: String,
    pub icon: String,
    pub kind: GoalKind,
    pub feeling: Option<usize>,
    /// How likely residents suggest it: base weight plus per-trait weights.
    pub weight: f32,
    pub traits: HashMap<String, f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActivityRaw {
    id: String,
    label: String,
    #[serde(default)]
    icon: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    skill: bool,
    #[serde(default)]
    social: bool,
    #[serde(default)]
    visit: bool,
    #[serde(default)]
    sleep: bool,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct PlannerRaw {
    boost: Option<f32>,
    off_block: Option<f32>,
    urgent_below: Option<f32>,
    floor: Option<f32>,
    skip_chance: Option<f32>,
    kept_share: Option<f32>,
    max_minutes: Option<u16>,
    max_sleep_minutes: Option<u16>,
    discipline: HashMap<String, f32>,
    kept_feeling: Option<String>,
    no_place_feeling: Option<String>,
    goal_feeling: Option<String>,
    review_hour: Option<f32>,
    max_goals: Option<usize>,
    max_suggestions: Option<usize>,
    goal_falloff: Option<f32>,
    suggestion_days: Option<u32>,
    ask_again_days: Option<u32>,
    wish_feeling: Option<String>,
    denied_feelings: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoalRaw {
    id: String,
    label: String,
    #[serde(default)]
    icon: String,
    kind: GoalKind,
    #[serde(default)]
    feeling: Option<String>,
    #[serde(default = "one")]
    weight: f32,
    #[serde(default)]
    traits: HashMap<String, f32>,
}

fn one() -> f32 {
    1.0
}

/// Builds the planner's content (after tags and feelings are known).
pub(crate) fn parse(
    activities: &[ActivityRaw],
    planner: &PlannerRaw,
    goals: &[GoalRaw],
    tag_index: &HashMap<&str, usize>,
    feelings: &[FeelingDef],
) -> Result<(Vec<ActivityDef>, PlannerRules, Vec<GoalDef>), Error> {
    let feeling = |id: &Option<String>, ctx: &str| -> Result<Option<usize>, Error> {
        id.as_deref()
            .map(|f| {
                feelings
                    .iter()
                    .position(|d| d.id == f)
                    .ok_or_else(|| Error::new(format!("{ctx}: unknown feeling '{f}'")))
            })
            .transpose()
    };
    let mut out = Vec::with_capacity(activities.len());
    for a in activities {
        if out.iter().any(|o: &ActivityDef| o.id == a.id) {
            return Err(Error::new(format!("duplicate activity '{}'", a.id)));
        }
        let tags = a.tags.iter().try_fold(0, |m: TagMask, t| {
            let i = tag_index
                .get(t.as_str())
                .ok_or_else(|| Error::new(format!("activity '{}': unknown tag '{t}'", a.id)))?;
            Ok::<_, Error>(m | (1 << i))
        })?;
        out.push(ActivityDef {
            id: a.id.clone(),
            label: a.label.clone(),
            icon: a.icon.clone(),
            tags,
            skill: a.skill,
            social: a.social,
            visit: a.visit,
            sleep: a.sleep,
        });
    }
    if out.len() > 64 {
        return Err(Error::new("at most 64 activities"));
    }
    let d = PlannerRules::default();
    let rules = PlannerRules {
        boost: planner.boost.unwrap_or(d.boost),
        off_block: planner.off_block.unwrap_or(d.off_block),
        urgent_below: planner.urgent_below.unwrap_or(d.urgent_below),
        floor: planner.floor.unwrap_or(d.floor),
        skip_chance: planner.skip_chance.unwrap_or(d.skip_chance).clamp(0.0, 1.0),
        kept_share: planner.kept_share.unwrap_or(d.kept_share).clamp(0.0, 1.0),
        max_minutes: planner.max_minutes.unwrap_or(d.max_minutes),
        max_sleep_minutes: planner.max_sleep_minutes.unwrap_or(d.max_sleep_minutes),
        discipline: planner.discipline.clone(),
        kept_feeling: feeling(&planner.kept_feeling, "planner")?,
        no_place_feeling: feeling(&planner.no_place_feeling, "planner")?,
        goal_feeling: feeling(&planner.goal_feeling, "planner")?,
        review_hour: planner
            .review_hour
            .unwrap_or(d.review_hour)
            .clamp(0.0, 23.9),
        max_goals: planner.max_goals.unwrap_or(d.max_goals),
        max_suggestions: planner.max_suggestions.unwrap_or(d.max_suggestions),
        goal_falloff: planner.goal_falloff.unwrap_or(d.goal_falloff).max(0.0),
        suggestion_days: planner.suggestion_days.unwrap_or(d.suggestion_days),
        ask_again_days: planner.ask_again_days.unwrap_or(d.ask_again_days),
        wish_feeling: feeling(&planner.wish_feeling, "planner")?,
        denied_feelings: planner
            .denied_feelings
            .iter()
            .map(|f| feeling(&Some(f.clone()), "planner.deniedFeelings").map(|f| f.expect("given")))
            .collect::<Result<_, _>>()?,
    };
    let mut defs = Vec::with_capacity(goals.len());
    for g in goals {
        if defs.iter().any(|o: &GoalDef| o.id == g.id) {
            return Err(Error::new(format!("duplicate goal '{}'", g.id)));
        }
        defs.push(GoalDef {
            id: g.id.clone(),
            label: g.label.clone(),
            icon: g.icon.clone(),
            kind: g.kind,
            feeling: feeling(&g.feeling, &g.id)?,
            weight: g.weight,
            traits: g.traits.clone(),
        });
    }
    Ok((out, rules, defs))
}

// ---- State --------------------------------------------------------------------------------

mod factors;
mod goals;
mod model;
mod parse;
mod purchases;
mod update;

pub use factors::*;
pub use goals::*;
pub use model::*;
pub(crate) use parse::*;
pub(crate) use purchases::*;
pub use update::*;
