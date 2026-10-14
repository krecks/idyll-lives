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

use crate::clock;
use crate::content::{Content, Interaction, TagMask};
use crate::social::FeelingDef;
use crate::social::{self, EventKind};
use crate::world::{Layout, Phase, Sim, TaskKind, World};
use crate::Error;

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
            let i = tag_index.get(t.as_str()).ok_or_else(|| Error::new(format!("activity '{}': unknown tag '{t}'", a.id)))?;
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
        review_hour: planner.review_hour.unwrap_or(d.review_hour).clamp(0.0, 23.9),
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

/// A block on the weekly calendar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Routine {
    pub id: u16,
    /// Index into `Content::activities`.
    pub activity: usize,
    /// For activities that train a skill.
    pub skill: Option<usize>,
    /// Weekdays it starts on (bit 0 = Monday).
    pub days: u8,
    /// Minute of the day it starts.
    pub start: u16,
    pub minutes: u16,
}

impl Routine {
    /// The day (from 1) whose block is running at `day`/`minute`, if any (a block may run past
    /// midnight into the next day).
    pub fn running(&self, day: u32, minute: f32) -> Option<u32> {
        let (start, end) = (self.start as f32, self.start as f32 + self.minutes as f32);
        let on = |d: u32| d >= 1 && self.days & (1 << clock::weekday(d)) != 0;
        if on(day) && minute >= start && minute < end {
            return Some(day);
        }
        (day > 1 && on(day - 1) && end > 1440.0 && minute < end - 1440.0).then(|| day - 1)
    }

    /// Whether this routine has a block starting on `day` or still running from the day before.
    fn touches(&self, day: u32) -> bool {
        let on = |d: u32| d >= 1 && self.days & (1 << clock::weekday(d)) != 0;
        on(day) || (day > 1 && on(day - 1) && self.start as u32 + self.minutes as u32 > 1440)
    }

    /// Every week-minute range `[from, to)` the routine covers (`to` may pass the week's end).
    fn spans(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (0..7u32)
            .filter(|d| self.days & (1 << d) != 0)
            .map(|d| d * 1440 + self.start as u32)
            .map(|from| (from, from + self.minutes as u32))
    }

    /// Whether two routines would run at the same time (also across the week's end).
    pub fn overlaps(&self, other: &Routine) -> bool {
        const WEEK: i64 = 7 * 1440;
        self.spans().any(|(a0, a1)| {
            other.spans().any(|(b0, b1)| {
                let (a0, a1, b0, b1) = (a0 as i64, a1 as i64, b0 as i64, b1 as i64);
                [-WEEK, 0, WEEK].iter().any(|&s| a0 < b1 + s && b0 + s < a1)
            })
        })
    }
}

/// Why a block wasn't kept.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reason {
    /// Their trait (index into `Content::traits`) doesn't like it.
    Trait(usize),
    /// Not in the mood.
    Mood,
    /// A need (index) got urgent.
    Need(usize),
    /// At work or out visiting.
    Away,
    /// Nothing at home (or in a park) offers the activity.
    NoPlace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Kept,
    Cut,
    Skipped,
    NoPlace,
}

/// A block that is running now.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockRun {
    pub routine: u16,
    /// From the household's template.
    pub household: bool,
    pub activity: usize,
    pub skill: Option<usize>,
    /// The day it started (from 1).
    pub day: u32,
    pub start: u16,
    pub minutes: u16,
    /// Skipped from the start (with why), or nowhere to do it.
    pub skipped: Option<Reason>,
    pub no_place: bool,
    /// How strongly they go for it (boost on fitting choices).
    pub boost: f32,
    /// Minutes spent on it so far.
    pub done: f32,
    /// The urgent need that pulled them away, if any.
    pub pulled: Option<usize>,
    /// They were away when it started and came back during it.
    pub late: bool,
}

/// How a past block went.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockResult {
    pub routine: u16,
    pub household: bool,
    pub activity: usize,
    pub skill: Option<usize>,
    pub day: u32,
    pub start: u16,
    pub minutes: u16,
    pub outcome: Outcome,
    pub reason: Option<Reason>,
    pub done: f32,
}

/// A life goal (or a suggestion for one).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Goal {
    /// Index into `Content::goals`.
    pub def: usize,
    pub skill: Option<usize>,
    /// Career category (index into `Content::career_categories`), for job goals.
    pub category: Option<usize>,
    /// Object type, for things they want to have.
    pub object: Option<usize>,
    pub target: f32,
    pub since_day: u32,
    /// Where they started (job level for promotions).
    pub start: f32,
    /// Progress 0..1, and its value at the last few daily reviews (newest first).
    pub progress: f32,
    pub history: [f32; 3],
}

impl Goal {
    /// The same goal (what, which skill or field, how far), whenever it was taken on.
    pub fn same(&self, other: &Goal) -> bool {
        (self.def, self.skill, self.category, self.object, self.target)
            == (other.def, other.skill, other.category, other.object, other.target)
    }
}

/// What a resident is thinking about (shown as a thought bubble).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thought {
    pub kind: ThoughtKind,
    /// Activity or goal definition the thought is about.
    pub subject: usize,
    pub until: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThoughtKind {
    /// Skipping a planned block.
    Skipped = 1,
    /// Wants somewhere to do a planned activity.
    NoPlace = 2,
    /// Kept a planned block.
    Kept = 3,
    /// Reached a goal.
    Goal = 4,
    /// Loves the room they're in (subject: `rooms::room_subject`).
    RoomLoved = 5,
    /// Dislikes the room they're in, for its weakest factor (subject: `rooms::room_subject`).
    RoomDisliked = 6,
    /// Something at home broke (subject: object type).
    Broken = 7,
    /// Had an accident (subject: index into content `accidents`).
    Accident = 8,
    /// A baby crying for care (subject: the need that's lowest).
    Crying = 9,
    /// Dinner's ready: someone at home cooked (subject: the recipe).
    DinnerReady = 10,
    /// Would like to buy something (subject: object type).
    WantsToBuy = 11,
}

/// Something a resident asked to buy (their top `Own` goal), and how often they were told no.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PurchaseRequest {
    /// Object type.
    pub object: usize,
    /// Waiting for an answer (asked on `day`); else told no on `day`.
    pub open: bool,
    pub day: u32,
    /// Times they were told no for this thing.
    pub declines: u8,
}

/// A wish about the home itself (beyond somewhere for a planned activity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeWish {
    /// A better room of this kind (`garden`: the garden), in this factor (`rooms::FACTORS`).
    Room { kind: Option<usize>, garden: bool, factor: u8 },
    /// A broken object fixed (object type).
    Fix { def: usize },
    /// Another room of this kind (a second bathroom).
    Another { kind: usize },
}

/// Home wishes kept at once (the oldest goes).
pub const MAX_HOME_WISHES: usize = 4;
/// Finding the bathroom taken on two days this close together brings a wish for another; the
/// wish goes after `WAIT_FORGET_DAYS` without it happening again.
pub const WAIT_REPEAT_DAYS: u32 = 7;
pub const WAIT_FORGET_DAYS: u32 = 14;

/// Results kept for the planner's look back.
pub const HISTORY: usize = 80;
/// How long a thought shows (game minutes).
const THOUGHT_MINUTES: f32 = 60.0;

#[derive(Debug, Clone, Default)]
pub struct Planner {
    pub routines: Vec<Routine>,
    /// Household template blocks this resident doesn't follow.
    pub skip_household: Vec<u16>,
    pub goals: Vec<Goal>,
    pub suggestions: Vec<Goal>,
    pub run: Option<BlockRun>,
    pub history: VecDeque<BlockResult>,
    /// Activities they'd like a place for (from blocks with nowhere to do them), and skills
    /// they'd like to train.
    pub wishes: Vec<(usize, Option<usize>)>,
    /// What they'd like for the home itself (better rooms, fixes, another bathroom).
    pub home_wishes: Vec<HomeWish>,
    /// The last day they badly needed something that was taken (see `rooms::opinions`).
    pub waited_day: Option<u32>,
    pub thought: Option<Thought>,
    /// The day goals were last reviewed.
    pub reviewed: u32,
    /// Something a goal may hang on just changed: check goals on the next update.
    pub recheck: bool,
    /// What they asked to buy (the player says yes or no).
    pub purchase: Option<PurchaseRequest>,
}

/// What a household's home (plus the public places) offers: activity bits and trainable skills.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HouseOffer {
    pub activities: u64,
    pub skills: u32,
}

impl HouseOffer {
    pub fn has(&self, a: &ActivityDef, index: usize, skill: Option<usize>) -> bool {
        if a.social || a.visit {
            return true;
        }
        match skill {
            Some(s) if a.skill => self.skills & (1 << s) != 0,
            _ => self.activities & (1 << index) != 0,
        }
    }
}

// ---- The weekly calendar --------------------------------------------------------------------

/// The block running for a resident at `day`/`minute`: their own first, then the household's
/// (unless skipped). Returns the routine, whether it's the household's, and its start day.
pub fn running(sim: &Sim, household: &[Routine], day: u32, minute: f32) -> Option<(Routine, bool, u32)> {
    let p = &sim.planner;
    p.routines
        .iter()
        .find_map(|r| r.running(day, minute).map(|d| (*r, false, d)))
        .or_else(|| {
            household
                .iter()
                .filter(|r| !p.skip_household.contains(&r.id))
                .find_map(|r| r.running(day, minute).map(|d| (*r, true, d)))
        })
}

/// For residents who plan their sleep: whether it's their night now (inside a sleep block), or
/// `None` when they have no sleep block today (the usual day rhythm applies).
pub fn planned_night(content: &Content, sim: &Sim, household: &[Routine], day: u32, minute: f32) -> Option<bool> {
    let p = &sim.planner;
    let sleep = |r: &&Routine| content.activities.get(r.activity).is_some_and(|a| a.sleep);
    let own = p.routines.iter().filter(sleep);
    let shared = household
        .iter()
        .filter(|r| !p.skip_household.contains(&r.id))
        .filter(sleep);
    let mut any = false;
    for r in own.chain(shared) {
        if r.running(day, minute).is_some() {
            return Some(true);
        }
        any |= r.touches(day);
    }
    any.then_some(false)
}

/// How well a resident sticks to plans: discipline from traits times mood.
pub fn adherence(content: &Content, sim: &Sim) -> f32 {
    let discipline: f32 = sim
        .traits
        .iter()
        .filter_map(|t| content.planner.discipline.get(t))
        .product();
    discipline * (0.6 + 0.8 * sim.mood(content))
}

/// Chance a resident skips a block of `activity` when it starts: less with discipline and a
/// good mood, more when their traits dislike the activity.
pub fn skip_chance(content: &Content, sim: &Sim, activity: &ActivityDef) -> f32 {
    let liking = if activity.tags != 0 { sim.mods.preference(activity.tags) } else { 1.0 };
    let base = content.planner.skip_chance / adherence(content, sim).max(0.05);
    (base + (1.0 - liking).max(0.0) * 0.5).min(0.9)
}

// ---- Per tick ----------------------------------------------------------------------------

pub(crate) fn update(w: &mut World) {
    refresh_offers(w);
    let tick = w.tick;
    let day = clock::day(tick);
    let minute = clock::minute_of_day(tick);
    let review = clock::at_minute(tick, w.content.planner.review_hour * 60.0);
    for i in 0..w.sims.len() {
        if !w.sims[i].here() {
            continue;
        }
        // Planned blocks: in lots nobody watches, on their turn (see `lod.rs`).
        if w.sims[i].steps > 0 {
            let h = w.sims[i].household as usize;
            let now = running(&w.sims[i], &w.households[h].routines, day, minute);
            let current = w.sims[i].planner.run;
            let same = |r: &BlockRun| now.is_some_and(|(n, household, d)| n.id == r.routine && household == r.household && d == r.day);
            if let Some(run) = current
                && !same(&run)
            {
                finish(w, i, run);
            }
            if let Some((routine, household, block_day)) = now
                && !current.as_ref().is_some_and(same)
            {
                begin(w, i, routine, household, block_day);
            }
            follow(w, i);
        }
        if w.sims[i].planner.thought.is_some_and(|t| t.until <= tick) {
            w.sims[i].planner.thought = None;
        }
        // Goals: progress every game ten minutes (staggered) or straight after a promotion, a
        // new skill level or a purchase; reviews once a day.
        if (tick + i as u64).is_multiple_of(200) || std::mem::take(&mut w.sims[i].planner.recheck) {
            check_goals(w, i);
        }
        if review && w.sims[i].planner.reviewed != day {
            w.sims[i].planner.reviewed = day;
            review_goals(w, i, day);
        }
    }
}

/// Recomputes what each household's home offers when walls or objects changed.
fn refresh_offers(w: &mut World) {
    if w.offers_version == w.structure_version && w.offers.len() == w.households.len() {
        return;
    }
    w.offers_version = w.structure_version;
    let content = &w.content;
    let public: Vec<bool> = w.plots.iter().map(|p| p.public).collect();
    w.offers = w
        .households
        .iter()
        .map(|h| {
            let mut offer = HouseOffer::default();
            // There is always something to tidy at home.
            if h.plot.is_some() {
                for (a, def) in content.activities.iter().enumerate() {
                    if def.tags & content.room_rules.clean.tags != 0 && !def.social && !def.visit {
                        offer.activities |= 1 << a;
                    }
                }
            }
            for (o, plot) in w.objects.iter().zip(&w.object_plot) {
                let here = plot.is_some_and(|p| Some(p) == h.plot || public[p as usize]);
                if !here || o.broken() {
                    continue;
                }
                for inter in content.objects[o.def].interactions.iter().filter(|i| i.autonomous) {
                    for (a, def) in content.activities.iter().enumerate() {
                        if inter.tags & def.tags != 0 {
                            offer.activities |= 1 << a;
                        }
                    }
                    for (s, &g) in inter.skill_gain.iter().enumerate() {
                        if g > 0.0 {
                            offer.skills |= 1 << s;
                        }
                    }
                }
            }
            offer
        })
        .collect();
    // Wishes come true once the house has what they wished for.
    for s in &mut w.sims {
        let offer = w.offers[s.household as usize];
        s.planner
            .wishes
            .retain(|&(a, skill)| !offer.has(&content.activities[a], a, skill));
    }
}

pub(crate) fn think(sim: &mut Sim, kind: ThoughtKind, subject: usize, tick: u64) {
    sim.planner.thought = Some(Thought {
        kind,
        subject,
        until: tick + social::minutes_to_ticks(THOUGHT_MINUTES),
    });
}

/// A block starts: maybe skipped (traits, mood, away, nowhere to do it), else they drop what
/// they chose themselves and go for it.
fn begin(w: &mut World, i: usize, routine: Routine, household: bool, day: u32) {
    let tick = w.tick;
    let content = &w.content;
    let rules = &content.planner;
    let Some(activity) = content.activities.get(routine.activity).cloned() else {
        return;
    };
    let offer = w.offers.get(w.sims[i].household as usize).copied().unwrap_or_default();
    let sim = &w.sims[i];
    let away = sim.away_until.is_some() || (sim.visiting.is_some() && !activity.visit);
    let no_place = !offer.has(&activity, routine.activity, routine.skill);
    let adherence = adherence(content, sim);
    let skip = skip_chance(content, sim, &activity);
    let roll = w.rng.next_f32();
    let skipped = if away {
        Some(Reason::Away)
    } else if !no_place && roll < skip && !activity.sleep {
        let dislike = sim
            .traits
            .iter()
            .filter_map(|t| content.traits.iter().position(|d| &d.id == t))
            .map(|t| (t, content.traits[t].mods.preference(activity.tags)))
            .filter(|(_, p)| *p < 0.95)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        Some(dislike.map_or(Reason::Mood, |(t, _)| Reason::Trait(t)))
    } else {
        None
    };
    let run = BlockRun {
        routine: routine.id,
        household,
        activity: routine.activity,
        skill: routine.skill,
        day,
        start: routine.start,
        minutes: routine.minutes,
        skipped,
        no_place,
        boost: rules.boost * adherence.clamp(0.4, 1.6),
        done: 0.0,
        pulled: None,
        late: false,
    };
    let World {
        sims,
        objects,
        content,
        ..
    } = w;
    let sim = &mut sims[i];
    sim.planner.run = Some(run);
    if no_place && !away {
        let wish = (routine.activity, routine.skill.filter(|_| activity.skill));
        if !sim.planner.wishes.contains(&wish) {
            sim.planner.wishes.push(wish);
            if sim.planner.wishes.len() > 4 {
                sim.planner.wishes.remove(0);
            }
        }
        if let Some(f) = content.planner.no_place_feeling {
            social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        }
        think(sim, ThoughtKind::NoPlace, routine.activity, tick);
        return;
    }
    if matches!(skipped, Some(Reason::Trait(_) | Reason::Mood)) {
        think(sim, ThoughtKind::Skipped, routine.activity, tick);
        return;
    }
    if skipped.is_some() {
        return;
    }
    // Time for it: stop what they picked for themselves (not a needed sleep, a conversation,
    // work or a trip, and not something the player asked for).
    let rested = sim.needs[..content.needs.len()]
        .iter()
        .all(|&n| n >= 0.5);
    let interruptible = sim.engaged_with.is_none()
        && sim.current().is_some_and(|a| {
            !a.task.directed
                && (a.tags & content.day_rhythm.sleep_tags == 0 || rested)
                && matches!(a.task.kind, TaskKind::Use { .. } | TaskKind::MoveTo { .. } | TaskKind::Clean { .. })
                && !fits(content, objects, &run, sim)
        });
    if interruptible {
        crate::world::end_activity(sim, content, objects);
    }
}

/// While a block runs: count the minutes spent on it, and note what pulled them away. Back
/// from work or a visit with time left, they pick it up again.
fn follow(w: &mut World, i: usize) {
    let content = &w.content;
    let sim = &w.sims[i];
    let Some(run) = sim.planner.run else { return };
    let minutes = sim.minutes();
    if run.skipped == Some(Reason::Away) && sim.away_until.is_none() && sim.visiting.is_none() {
        let run = w.sims[i].planner.run.as_mut().expect("checked above");
        run.skipped = None;
        run.late = true;
        return;
    }
    if run.skipped.is_some() || run.no_place {
        return;
    }
    let on_it = fits(content, &w.objects, &run, sim);
    let mut pulled = run.pulled;
    if !on_it
        && let Some(n) = (0..content.needs.len())
            .filter(|&n| !content.needs[n].room && sim.needs[n] < content.planner.urgent_below)
            .min_by(|&a, &b| sim.needs[a].total_cmp(&sim.needs[b]))
    {
        pulled = Some(n);
    }
    let run = w.sims[i].planner.run.as_mut().expect("checked above");
    if on_it {
        run.done += minutes;
    }
    run.pulled = pulled;
}

/// Whether what the resident is doing right now is the block's activity.
fn fits(content: &Content, objects: &[crate::world::ObjectInstance], run: &BlockRun, sim: &Sim) -> bool {
    let Some(a) = content.activities.get(run.activity) else {
        return false;
    };
    if a.visit && sim.visiting.is_some() {
        return true;
    }
    if a.social && sim.engaged_with.is_some() {
        return true;
    }
    let Some(act) = sim.current() else {
        return false;
    };
    match (act.task.kind, &act.phase) {
        (TaskKind::Social { .. } | TaskKind::Join { .. }, Phase::Conversing { .. }) => a.social,
        // Walking over to it counts too.
        (TaskKind::Use { object, interaction }, Phase::Using { .. } | Phase::Routing { .. }) => objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .is_some_and(|inter| a.fits(inter, run.skill)),
        // Tidying up is chores.
        (TaskKind::Clean { .. }, Phase::Using { .. } | Phase::Routing { .. }) => {
            run.skill.is_none() && a.tags & content.room_rules.clean.tags != 0
        }
        // Cooking is what the recipe's interaction is (all along, between stations too); eating
        // and washing up are what their tags say.
        (TaskKind::Cook { recipe, origin, .. }, _) => {
            crate::cooking::origin_interaction(content, objects, origin, recipe).is_some_and(|inter| a.fits(inter, run.skill))
        }
        (TaskKind::Eat { .. }, _) => run.skill.is_none() && a.tags & content.cooking.eat_tags != 0,
        (TaskKind::Dishes { .. }, _) => run.skill.is_none() && a.tags & content.cooking.dishes_tags != 0,
        // On the phone with someone is socialising.
        (TaskKind::Phone { contact: Some(_), .. }, Phase::Using { .. }) => a.social,
        _ => false,
    }
}

/// A block is over: how it went, a feeling for keeping it.
fn finish(w: &mut World, i: usize, run: BlockRun) {
    let tick = w.tick;
    let content = &w.content;
    let sim = &mut w.sims[i];
    sim.planner.run = None;
    let (outcome, reason) = if run.no_place {
        (Outcome::NoPlace, Some(Reason::NoPlace))
    } else if let Some(r) = run.skipped {
        (Outcome::Skipped, Some(r))
    } else if run.done >= run.minutes as f32 * content.planner.kept_share {
        (Outcome::Kept, None)
    } else if run.done > 0.0 {
        (Outcome::Cut, run.pulled.map(Reason::Need).or(run.late.then_some(Reason::Away)).or(Some(Reason::Mood)))
    } else {
        (Outcome::Skipped, run.pulled.map(Reason::Need).or(run.late.then_some(Reason::Away)).or(Some(Reason::Mood)))
    };
    if outcome == Outcome::Kept {
        if let Some(f) = content.planner.kept_feeling {
            social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        }
        if !content.activities[run.activity].sleep {
            think(sim, ThoughtKind::Kept, run.activity, tick);
        }
    }
    let history = &mut sim.planner.history;
    if history.len() >= HISTORY {
        history.pop_front();
    }
    history.push_back(BlockResult {
        routine: run.routine,
        household: run.household,
        activity: run.activity,
        skill: run.skill,
        day: run.day,
        start: run.start,
        minutes: run.minutes,
        outcome,
        reason,
        done: run.done,
    });
}

// ---- Goals ------------------------------------------------------------------------------

/// Progress toward a goal, 0..1.
pub fn progress(w: &World, i: usize, g: &Goal) -> f32 {
    let content = &w.content;
    let sim = &w.sims[i];
    let target = g.target.max(1e-3);
    let job_ok = |c: usize| g.category.is_none_or(|cat| content.careers[c].category == Some(cat));
    let p = match content.goals[g.def].kind {
        GoalKind::HasJob => sim.job.as_ref().is_some_and(|j| job_ok(j.career)) as u8 as f32,
        GoalKind::JobLevel => sim
            .job
            .as_ref()
            .filter(|j| job_ok(j.career))
            .map_or(0.0, |j| (j.level + 1) as f32 / target),
        // The skills the next level needs count as much as the performance bar.
        GoalKind::Promoted => sim.job.as_ref().map_or(0.0, |j| {
            if j.level as f32 > g.start {
                1.0
            } else {
                let needs = goal_skills(content, sim, g);
                let ready = if needs.is_empty() {
                    1.0
                } else {
                    needs.iter().map(|&(s, l)| (sim.skills[s] / l).min(1.0)).sum::<f32>() / needs.len() as f32
                };
                (ready + j.performance / 100.0) * 0.5 * 0.9
            }
        }),
        GoalKind::Skill => g.skill.map_or(0.0, |s| sim.skills[s] / target),
        GoalKind::Friends => {
            let friends = (0..w.sims.len())
                .filter(|&j| j != i && w.relationships.get(i, j).friendship >= 50.0)
                .count();
            friends as f32 / target
        }
        GoalKind::Partner => {
            if w.relationships.partner_of(i).is_some() {
                1.0
            } else {
                let best = (0..w.sims.len())
                    .filter(|&j| j != i)
                    .map(|j| w.relationships.get(i, j).romance)
                    .fold(0.0f32, f32::max);
                (best / 100.0 * 0.9).max(0.0)
            }
        }
        GoalKind::Funds => w.households[sim.household as usize].funds as f32 / target,
        GoalKind::Own => match g.object {
            Some(d) if has_at_home(w, sim.household as usize, d) => 1.0,
            // Saving up for it: it's only had once it's home.
            _ => (w.households[sim.household as usize].funds as f32 / target).min(1.0) * 0.9,
        },
    };
    p.clamp(0.0, 1.0)
}

/// Skill levels a career goal takes (the next level's for a promotion; every level's up to the
/// target for a job level, in their current job), highest per skill.
pub fn goal_skills(content: &Content, sim: &Sim, g: &Goal) -> Vec<(usize, f32)> {
    let Some(job) = &sim.job else { return Vec::new() };
    let career = &content.careers[job.career];
    let levels = match content.goals[g.def].kind {
        GoalKind::Promoted => job.level + 1..job.level + 2,
        GoalKind::JobLevel if g.category.is_none_or(|c| career.category == Some(c)) => {
            job.level + 1..g.target.max(0.0) as usize
        }
        _ => return Vec::new(),
    };
    let mut out: Vec<(usize, f32)> = Vec::new();
    for level in career.levels.get(levels).unwrap_or_default() {
        for &(s, l) in &level.requires {
            match out.iter_mut().find(|o| o.0 == s) {
                Some(o) => o.1 = o.1.max(l),
                None if l > 0.0 => out.push((s, l)),
                None => {}
            }
        }
    }
    out
}

fn check_goals(w: &mut World, i: usize) {
    let tick = w.tick;
    // Ideas that came true meanwhile just go.
    let mut k = 0;
    while k < w.sims[i].planner.suggestions.len() {
        let goal = w.sims[i].planner.suggestions[k];
        if progress(w, i, &goal) >= 1.0 {
            w.sims[i].planner.suggestions.remove(k);
        } else {
            k += 1;
        }
    }
    let mut k = 0;
    while k < w.sims[i].planner.goals.len() {
        let goal = w.sims[i].planner.goals[k];
        let p = progress(w, i, &goal);
        w.sims[i].planner.goals[k].progress = p;
        if p < 1.0 {
            k += 1;
            continue;
        }
        let content = &w.content;
        let sim = &mut w.sims[i];
        sim.planner.goals.remove(k);
        let feeling = content.goals[goal.def].feeling.or(content.planner.goal_feeling);
        if let Some(f) = feeling {
            social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        }
        // Got what they wanted (whoever bought it).
        if goal.object.is_some() {
            if let Some(f) = content.planner.wish_feeling {
                social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
            }
            if sim.planner.purchase.is_some_and(|p| Some(p.object) == goal.object) {
                sim.planner.purchase = None;
            }
        }
        think(sim, ThoughtKind::Goal, goal.def, tick);
        w.events.push_goal(tick, EventKind::GoalReached, i, &goal);
    }
}

/// Once a day: progress history, and maybe a new goal suggested from their traits (the
/// neighbours, who have no player, take theirs on straight away).
fn review_goals(w: &mut World, i: usize, day: u32) {
    for k in 0..w.sims[i].planner.goals.len() {
        let g = w.sims[i].planner.goals[k];
        let p = progress(w, i, &g);
        let g = &mut w.sims[i].planner.goals[k];
        g.progress = p;
        g.history = [p, g.history[0], g.history[1]];
    }
    ask_to_buy(w, i, day);
    let rules = &w.content.planner;
    let player = w.households[w.sims[i].household as usize].player;
    let free = w.autonomy && w.households[w.sims[i].household as usize].free_will;
    // Ideas the player let be for a while: they go ahead with them.
    if free {
        let (max, wait) = (rules.max_goals, rules.suggestion_days);
        let p = &mut w.sims[i].planner;
        while p.goals.len() < max
            && let Some(k) = p.suggestions.iter().position(|g| day >= g.since_day + wait)
        {
            let mut goal = p.suggestions.remove(k);
            goal.since_day = day;
            if !p.goals.iter().any(|g| g.same(&goal)) {
                p.goals.push(goal);
            }
        }
    }
    let rules = &w.content.planner;
    let p = &w.sims[i].planner;
    let room = if player {
        p.suggestions.len() < rules.max_suggestions && p.goals.len() < rules.max_goals
    } else {
        p.goals.len() < rules.max_goals.min(2)
    };
    if !room || !w.autonomy || !w.households[w.sims[i].household as usize].free_will {
        return;
    }
    // About every other day.
    if w.rng.next_f32() < 0.5 {
        return;
    }
    let Some(goal) = suggest(w, i, day) else { return };
    let tick = w.tick;
    let sim = &mut w.sims[i];
    if player {
        sim.planner.suggestions.push(goal);
        w.events.push_goal(tick, EventKind::GoalSuggested, i, &goal);
    } else {
        sim.planner.goals.push(goal);
    }
}

/// A goal that suits the resident now, weighted by their traits.
fn suggest(w: &mut World, i: usize, day: u32) -> Option<Goal> {
    let wanted = if w.content.goals.iter().any(|d| d.kind == GoalKind::Own) { wanted_object(w, i) } else { None };
    let content = &w.content;
    let sim = &w.sims[i];
    let taken = |kind: GoalKind, skill: Option<usize>| {
        sim.planner
            .goals
            .iter()
            .chain(&sim.planner.suggestions)
            .any(|g| content.goals[g.def].kind == kind && (kind != GoalKind::Skill || g.skill == skill))
    };
    let friends = (0..w.sims.len())
        .filter(|&j| j != i && w.relationships.get(i, j).friendship >= 50.0)
        .count() as f32;
    let funds = w.households[sim.household as usize].funds;
    // Skills a career goal of theirs takes already (no separate goal for those).
    let implied: Vec<usize> = sim
        .planner
        .goals
        .iter()
        .chain(&sim.planner.suggestions)
        .flat_map(|g| goal_skills(content, sim, g))
        .map(|(s, _)| s)
        .collect();
    // The skill they're best at learning (traits), else their strongest.
    let skill = (0..content.skills.len())
        .filter(|&s| sim.skills[s] < content.skill_rules.max_level - 1.0 && !implied.contains(&s))
        .max_by(|&a, &b| {
            let key = |s: usize| sim.mods.skill_gain[s] * 2.0 + sim.skills[s] * 0.1;
            key(a).total_cmp(&key(b))
        });
    let mut options: Vec<(f32, Goal)> = Vec::new();
    for (d, def) in content.goals.iter().enumerate() {
        let make = |skill: Option<usize>, target: f32, start: f32| Goal {
            def: d,
            skill,
            category: None,
            object: None,
            target,
            since_day: day,
            start,
            progress: 0.0,
            history: [0.0; 3],
        };
        let goal = match def.kind {
            GoalKind::HasJob if sim.job.is_none() => make(None, 1.0, 0.0),
            GoalKind::JobLevel => match &sim.job {
                Some(j) if j.level + 2 < content.careers[j.career].levels.len() => {
                    make(None, (j.level + 3) as f32, 0.0)
                }
                _ => continue,
            },
            GoalKind::Promoted => match &sim.job {
                Some(j) if j.level + 1 < content.careers[j.career].levels.len() => {
                    make(None, 1.0, j.level as f32)
                }
                _ => continue,
            },
            GoalKind::Skill => match skill {
                Some(s) => make(Some(s), (sim.skills[s].floor() + 2.0).min(content.skill_rules.max_level), 0.0),
                None => continue,
            },
            GoalKind::Friends => make(None, friends + 2.0, 0.0),
            GoalKind::Partner if w.relationships.partner_of(i).is_none() => make(None, 1.0, 0.0),
            GoalKind::Funds if funds < 50_000 => {
                make(None, (((funds.max(0) + 2500) / 500) * 500) as f32, 0.0)
            }
            GoalKind::Own => match wanted {
                Some(o) => Goal {
                    object: Some(o),
                    ..make(None, content.objects[o].price.unwrap_or(0) as f32, 0.0)
                },
                None => continue,
            },
            _ => continue,
        };
        if taken(def.kind, goal.skill) {
            continue;
        }
        let weight = def.weight + sim.traits.iter().filter_map(|t| def.traits.get(t)).sum::<f32>();
        if weight > 0.0 {
            options.push((weight, goal));
        }
    }
    let total: f32 = options.iter().map(|o| o.0).sum();
    let mut pick = w.rng.next_f32() * total;
    for (weight, goal) in options {
        if pick < weight {
            return Some(goal);
        }
        pick -= weight;
    }
    None
}

// ---- Things they want ---------------------------------------------------------------------

/// Whether household `h` has an object of type `def` at home.
pub fn has_at_home(w: &World, h: usize, def: usize) -> bool {
    let Some(plot) = w.households[h].plot else { return false };
    w.objects.iter().zip(&w.object_plot).any(|(o, p)| *p == Some(plot) && o.def == def)
}

/// Two object types for the same thing (a better bed for a bed): the same catalog group and
/// some use in common.
fn same_kind(content: &Content, a: usize, b: usize) -> bool {
    let (a, b) = (&content.objects[a], &content.objects[b]);
    !a.group.is_empty()
        && a.group == b.group
        && a.interactions.iter().any(|x| b.interactions.iter().any(|y| x.id == y.id))
}

/// Whether object type `new` does all `old` does: every use of it (by id), the cooking stations
/// it is, and keeping leftovers. A range cooker shares the gourmet dinner with a fridge but is
/// no fridge: selling the fridge for it would leave no snacks and no ingredients.
fn covers(content: &Content, new: usize, old: usize) -> bool {
    let (new, old) = (&content.objects[new], &content.objects[old]);
    old.interactions.iter().all(|x| new.interactions.iter().any(|y| x.id == y.id))
        && old.stations & !new.stations == 0
        && (new.stores || !old.stores)
}

/// The cheaper thing at home that `def` would replace (an upgrade that does all it did), if any.
fn replaces(w: &World, h: usize, def: usize) -> Option<u32> {
    let plot = w.households[h].plot?;
    let price = w.content.objects[def].price?;
    w.objects
        .iter()
        .zip(&w.object_plot)
        .filter(|(o, p)| **p == Some(plot) && o.def != def && same_kind(&w.content, o.def, def) && covers(&w.content, def, o.def))
        .filter(|(o, _)| w.content.objects[o.def].price.is_some_and(|p| p < price))
        .min_by_key(|(o, _)| w.content.objects[o.def].price)
        .map(|(o, _)| o.id)
}

/// Something from the catalog that suits the resident and is within reach of the household's
/// money: what their traits like, trains the skills they're after, or gives a planned activity
/// a place; or a better one of something they have. One of the best few, at random.
fn wanted_object(w: &mut World, i: usize) -> Option<usize> {
    let pick = w.rng.next_f32();
    let content = &w.content;
    let sim = &w.sims[i];
    let h = sim.household as usize;
    let plot = w.households[h].plot?;
    let reach = w.households[h].funds.max(0) as f32 * 1.5 + 1000.0;
    let home: Vec<usize> = w
        .objects
        .iter()
        .zip(&w.object_plot)
        .filter(|(_, p)| **p == Some(plot))
        .map(|(o, _)| o.def)
        .collect();
    let (targets, keen) = crate::world::job_goals(sim, content);
    let mut options: Vec<(f32, usize)> = Vec::new();
    for (d, def) in content.objects.iter().enumerate() {
        let Some(price) = def.price else { continue };
        if price <= 0
            || price as f32 > reach
            || home.contains(&d)
            || sim.planner.goals.iter().chain(&sim.planner.suggestions).any(|g| g.object == Some(d))
        {
            continue;
        }
        let uses: Vec<&Interaction> = def.interactions.iter().filter(|it| it.autonomous && !it.baby).collect();
        if uses.is_empty() {
            continue;
        }
        // Something as good for the same thing at home already: no need.
        let same: Vec<usize> = home.iter().copied().filter(|&o| same_kind(content, o, d)).collect();
        if same.iter().any(|&o| content.objects[o].price.unwrap_or(0) >= price) {
            continue;
        }
        let liking = uses.iter().map(|it| sim.mods.preference(it.tags)).fold(0.0f32, f32::max);
        let mut score = (liking - 1.0).max(0.0) * 2.0;
        for it in &uses {
            for (s, &gain) in it.skill_gain.iter().enumerate() {
                if gain > 0.0 && targets[s] > sim.skills[s] {
                    score += keen[s].min(1.0);
                }
            }
            for &(a, skill) in &sim.planner.wishes {
                if content.activities[a].fits(it, skill) {
                    score += 3.0;
                }
            }
        }
        if !same.is_empty() {
            score += 0.5;
        }
        if score > 0.0 {
            options.push((score, d));
        }
    }
    options.sort_by(|a, b| b.0.total_cmp(&a.0));
    options.truncate(3);
    let total: f32 = options.iter().map(|o| o.0).sum();
    let mut pick = pick * total;
    for (score, d) in options {
        if pick < score {
            return Some(d);
        }
        pick -= score;
    }
    None
}

/// Once a day: a resident whose top `Own` goal they can afford (with a week's rent and bills
/// left over) asks the player for it, or, in a household nobody plays, just buys it. After a
/// no, they wait a few days before asking for the same thing again.
fn ask_to_buy(w: &mut World, i: usize, day: u32) {
    let h = w.sims[i].household as usize;
    if !w.autonomy || !w.households[h].free_will || w.sims[i].away_until.is_some() {
        return;
    }
    let content = &w.content;
    let sim = &w.sims[i];
    let Some(object) = sim
        .planner
        .goals
        .iter()
        .find(|g| content.goals[g.def].kind == GoalKind::Own)
        .and_then(|g| g.object)
    else {
        return;
    };
    if has_at_home(w, h, object) {
        return;
    }
    let price = content.objects[object].price.unwrap_or(0);
    let reserve = crate::life::weekly_costs(w, h).map_or(0, |(rent, bills)| rent + bills);
    if w.households[h].funds - w.build_price(price) < reserve {
        return;
    }
    let mut request = match sim.planner.purchase {
        Some(p) if p.object == object => p,
        _ => PurchaseRequest { object, open: false, day, declines: 0 },
    };
    // Waiting for an answer since today, or after a no.
    if request.open && request.day == day
        || !request.open && request.declines > 0 && day < request.day + content.planner.ask_again_days
    {
        return;
    }
    if !w.households[h].player {
        // Nobody to ask: they go ahead (and try again in a few days if there's no room).
        if buy_wanted(w, i, object).is_err() {
            w.sims[i].planner.purchase = Some(PurchaseRequest { object, open: false, day, declines: 1 });
        }
        return;
    }
    request.open = true;
    request.day = day;
    let tick = w.tick;
    let sim = &mut w.sims[i];
    sim.planner.purchase = Some(request);
    think(sim, ThoughtKind::WantsToBuy, object, tick);
    w.events.push_object(tick, EventKind::WantsToBuy, i, object, Some(price));
}

/// Buys `object` for resident `i`'s home: in place of the cheaper one it replaces if there is
/// one (sold, the new one where it stood if there's no other room), else at a free spot.
pub(crate) fn buy_wanted(w: &mut World, i: usize, object: usize) -> Result<u32, Error> {
    let h = w.sims[i].household;
    let id = w.content.objects[object].id.clone();
    let old = replaces(w, h as usize, object);
    let bought = match w.buy_at(h, &id, None, None, Default::default()) {
        Ok(new) => {
            // The old one goes (ids after it shift down).
            match old {
                Some(o) if w.sell(h, o).is_ok() && o < new => Ok(new - 1),
                _ => Ok(new),
            }
        }
        Err(e) => match old {
            Some(o) => swap_in(w, h, o, &id).ok_or(e),
            None => Err(e),
        },
    }?;
    w.events.push_object(w.tick, EventKind::Bought, i, object, None);
    w.sims[i].planner.recheck = true;
    Ok(bought)
}

/// No free spot: sells `old` and puts `id` where it stood (or anywhere now free). If even that
/// doesn't fit, `old` goes back as it was. Only for something nothing stands on and nobody uses.
fn swap_in(w: &mut World, h: u32, old: u32, id: &str) -> Option<u32> {
    let o = w.objects.get(old as usize)?.clone();
    if o.users.iter().any(Option::is_some) || !crate::surface::riders(&w.content, &w.objects, old).is_empty() {
        return None;
    }
    let (hh, plot) = w.home_of(h).ok()?;
    let before = w.households[hh].funds;
    w.sell(h, old).ok()?;
    let refund = w.households[hh].funds - before;
    let placed = w
        .buy_at(h, id, Some([o.x, o.z, o.rot as i32]), None, Default::default())
        .or_else(|_| w.buy_at(h, id, None, None, Default::default()));
    if let Ok(new) = placed {
        return Some(new);
    }
    // Put it back (and the money).
    if let Ok(back) = w.place_at_home(plot, o.def, o.x, o.z, o.rot, Layout::of(&o), o.color, o.quality, o.value) {
        let b = &mut w.objects[back as usize];
        (b.turn, b.wear, b.open) = (o.turn, o.wear, o.open);
    }
    w.households[hh].funds -= refund;
    None
}

/// The player says yes to what resident `i` asked for: it's bought (their goal is reached).
pub(crate) fn accept_purchase(w: &mut World, i: usize) -> Result<(), Error> {
    let Some(request) = w.sims[i].planner.purchase.filter(|p| p.open) else {
        return Err(Error::new(format!("{} isn't asking for anything", w.sims[i].name)));
    };
    buy_wanted(w, i, request.object)?;
    w.sims[i].planner.purchase = None;
    check_goals(w, i);
    Ok(())
}

/// The player says no: they'll ask again in a few days, and from the second no for the same
/// thing on, they're cross about it.
pub(crate) fn decline_purchase(w: &mut World, i: usize) -> Result<(), Error> {
    let day = clock::day(w.tick);
    let tick = w.tick;
    let content = &w.content;
    let sim = &mut w.sims[i];
    let Some(request) = sim.planner.purchase.as_mut().filter(|p| p.open) else {
        return Err(Error::new(format!("{} isn't asking for anything", sim.name)));
    };
    request.open = false;
    request.day = day;
    request.declines = request.declines.saturating_add(1);
    let (object, declines) = (request.object, request.declines);
    let denied = &content.planner.denied_feelings;
    if declines >= 2
        && let Some(&f) = denied.get((declines as usize - 2).min(denied.len().saturating_sub(1)))
    {
        social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        w.events.push_object(tick, EventKind::DeniedAgain, i, object, Some(declines as i64));
    }
    Ok(())
}

// ---- How plans and goals steer choices (used by `World::pick_autonomous`) ----------------

/// How strongly a goal steers by its place on the list (0 = the top): the top one fully, each
/// one below a little less.
pub fn goal_strength(rules: &PlannerRules, rank: usize) -> f32 {
    1.0 / (1.0 + rules.goal_falloff * rank as f32)
}

/// Strength of the highest goal of one of these kinds (0: none). A kind counts once, however
/// many goals of it there are.
fn kind_strength(content: &Content, sim: &Sim, kinds: &[GoalKind]) -> f32 {
    sim.planner
        .goals
        .iter()
        .position(|g| kinds.contains(&content.goals[g.def].kind))
        .map_or(0.0, |rank| goal_strength(&content.planner, rank))
}

/// A goal's multiplier `full` at `strength` (1 at none, `full` at the top of the list).
fn scaled(full: f32, strength: f32) -> f32 {
    1.0 + (full - 1.0) * strength
}

/// Multiplier for an object interaction while planning: (factor, floor score). `night` is
/// `planned_night` (residents who plan their sleep don't go to bed outside it unless exhausted).
pub(crate) fn object_factor(content: &Content, sim: &Sim, inter: &Interaction, night: Option<bool>) -> (f32, f32) {
    let mut factor = 1.0;
    if night == Some(false) && inter.tags & content.day_rhythm.sleep_tags != 0 {
        let exhausted = (0..content.needs.len())
            .any(|n| inter.total_gain[n] > 0.0 && sim.needs[n] < content.planner.urgent_below);
        if !exhausted {
            return (0.0, 0.0);
        }
    }
    if inter.cost > 0 {
        factor *= scaled(0.3, kind_strength(content, sim, &[GoalKind::Funds, GoalKind::Own]));
    }
    let Some(run) = active(sim) else {
        return (factor, 0.0);
    };
    let a = &content.activities[run.activity];
    if a.fits(inter, run.skill) && !a.social && !a.visit {
        return (factor * run.boost, content.planner.floor);
    }
    let urgent = (0..content.needs.len())
        .any(|n| sim.needs[n] < content.planner.urgent_below && inter.total_gain[n] > 0.0);
    (if urgent { factor } else { factor * content.planner.off_block }, 0.0)
}

/// `(factor, floor)` for an action without an object (tidying up) with these tags: favoured
/// in a planned block whose activity it fits (the chores), held back outside one.
pub(crate) fn tags_factor(content: &Content, sim: &Sim, tags: TagMask) -> (f32, f32) {
    let Some(run) = active(sim) else {
        return (1.0, 0.0);
    };
    let a = &content.activities[run.activity];
    if a.tags & tags != 0 && !a.social && !a.visit {
        return (run.boost, content.planner.floor);
    }
    (content.planner.off_block, 0.0)
}

/// Multiplier for a social with someone (`romantic`: it's a romantic social; `friendly`: it
/// builds friendship; `friendship` toward them; `single`: they're free to date).
pub(crate) fn social_factor(content: &Content, sim: &Sim, romantic: bool, friendly: bool, friendship: f32, single: bool) -> f32 {
    let mut factor = 1.0;
    if friendly && friendship < 50.0 {
        factor *= scaled(1.5, kind_strength(content, sim, &[GoalKind::Friends]));
    }
    if romantic && single {
        factor *= scaled(2.0, kind_strength(content, sim, &[GoalKind::Partner]));
    }
    match active(sim) {
        Some(run) if content.activities[run.activity].social => factor * run.boost,
        Some(_) if content.needs.iter().zip(&sim.needs).all(|(d, &n)| d.room || n >= content.planner.urgent_below) => {
            factor * content.planner.off_block
        }
        _ => factor,
    }
}

/// Whether a block other than socialising or visiting starts within `minutes` from now (a
/// resident with plans doesn't set off to visit just before them).
pub(crate) fn plans_soon(content: &Content, sim: &Sim, household: &[Routine], day: u32, minute: f32, minutes: f32) -> bool {
    let mut m = 15.0;
    while m <= minutes {
        let (d, t) = if minute + m >= 1440.0 { (day + 1, minute + m - 1440.0) } else { (day, minute + m) };
        if let Some((r, ..)) = running(sim, household, d, t)
            && content.activities.get(r.activity).is_some_and(|a| !a.social && !a.visit)
        {
            return true;
        }
        m += 15.0;
    }
    false
}

/// Multiplier for visiting someone (`soon`: plans start before a visit would be over).
pub(crate) fn visit_factor(content: &Content, sim: &Sim, soon: bool) -> f32 {
    if soon && active(sim).is_none() {
        return content.planner.off_block;
    }
    let factor = scaled(1.5, kind_strength(content, sim, &[GoalKind::Friends, GoalKind::Partner]));
    match active(sim) {
        Some(run) if content.activities[run.activity].visit => factor * run.boost,
        Some(_) => factor * content.planner.off_block,
        None => factor,
    }
}

/// Whether using `inter` is what the resident's running block is for.
pub(crate) fn planned_now(content: &Content, sim: &Sim, inter: &Interaction) -> bool {
    active(sim).is_some_and(|run| {
        let a = &content.activities[run.activity];
        !a.social && !a.visit && a.fits(inter, run.skill)
    })
}

/// The block steering choices now (not skipped, somewhere to do it).
fn active(sim: &Sim) -> Option<&BlockRun> {
    sim.planner
        .run
        .as_ref()
        .filter(|r| r.skipped.is_none() && !r.no_place)
}

/// Skill targets from goals, and how keen they are on each (folded into practice interest
/// with the job's).
pub(crate) fn skill_targets(content: &Content, sim: &Sim, targets: &mut [f32], keen: &mut [f32]) {
    for (rank, g) in sim.planner.goals.iter().enumerate() {
        let strength = goal_strength(&content.planner, rank);
        match content.goals[g.def].kind {
            GoalKind::Skill => {
                if let Some(s) = g.skill {
                    targets[s] = targets[s].max(g.target);
                    keen[s] = keen[s].max(strength);
                }
            }
            // A career goal takes its skills: keener on them than the job alone makes them.
            GoalKind::Promoted | GoalKind::JobLevel => {
                for (s, l) in goal_skills(content, sim, g) {
                    targets[s] = targets[s].max(l);
                    keen[s] = keen[s].max(1.0 + strength);
                }
            }
            _ => {}
        }
    }
}

/// Job search: how much harder they look (a job goal), and in which category.
pub(crate) fn job_search(content: &Content, sim: &Sim) -> (f32, Option<usize>) {
    sim.planner
        .goals
        .iter()
        .position(|g| content.goals[g.def].kind == GoalKind::HasJob)
        .map_or((1.0, None), |rank| {
            let g = &sim.planner.goals[rank];
            (scaled(3.0, goal_strength(&content.planner, rank)), g.category)
        })
}

// ---- Commands -----------------------------------------------------------------------------

/// A routine as the UI sends it (ids are kept, new ones assigned).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineIn {
    #[serde(default)]
    pub id: Option<u16>,
    pub activity: String,
    #[serde(default)]
    pub skill: Option<String>,
    /// Weekdays as a bitmask (bit 0 = Monday).
    pub days: u8,
    pub start: u16,
    pub minutes: u16,
}

/// A goal as the UI sends it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalIn {
    pub def: String,
    #[serde(default)]
    pub skill: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub target: Option<f32>,
}

/// Checks routines (valid activity, times, no overlaps) and gives each an id.
pub(crate) fn routines_from(content: &Content, list: &[RoutineIn]) -> Result<Vec<Routine>, Error> {
    let rules = &content.planner;
    let mut out: Vec<Routine> = Vec::with_capacity(list.len());
    for r in list {
        let activity = content
            .activities
            .iter()
            .position(|a| a.id == r.activity)
            .ok_or_else(|| Error::new(format!("unknown activity '{}'", r.activity)))?;
        let def = &content.activities[activity];
        let skill = match &r.skill {
            Some(s) if def.skill => Some(
                content
                    .skill_index(s)
                    .ok_or_else(|| Error::new(format!("unknown skill '{s}'")))?,
            ),
            _ => None,
        };
        let longest = if def.sleep { rules.max_sleep_minutes } else { rules.max_minutes };
        if r.days & 0x7f == 0 || r.start >= 1440 || r.minutes < 15 || r.minutes > longest {
            return Err(Error::new(format!(
                "{}: pick at least one day and 15 min to {} h",
                def.label,
                longest / 60
            )));
        }
        let mut routine = Routine {
            id: r.id.unwrap_or(0),
            activity,
            skill,
            days: r.days & 0x7f,
            start: r.start,
            minutes: r.minutes,
        };
        if let Some(other) = out.iter().find(|o| o.overlaps(&routine)) {
            return Err(Error::new(format!(
                "{} overlaps {}",
                def.label, content.activities[other.activity].label
            )));
        }
        if r.id.is_none() || out.iter().any(|o| o.id == routine.id) {
            routine.id = 0;
        }
        out.push(routine);
    }
    // New blocks get fresh ids.
    let first = out.iter().map(|r| r.id).max().unwrap_or(0) + 1;
    for (id, r) in (first..).zip(out.iter_mut().filter(|r| r.id == 0)) {
        r.id = id;
    }
    Ok(out)
}

pub(crate) fn goal_from(content: &Content, g: &GoalIn, sim: &Sim, day: u32) -> Result<Goal, Error> {
    let def = content
        .goals
        .iter()
        .position(|d| d.id == g.def)
        .ok_or_else(|| Error::new(format!("unknown goal '{}'", g.def)))?;
    let kind = content.goals[def].kind;
    let skill = g
        .skill
        .as_deref()
        .map(|s| content.skill_index(s).ok_or_else(|| Error::new(format!("unknown skill '{s}'"))))
        .transpose()?;
    if kind == GoalKind::Skill && skill.is_none() {
        return Err(Error::new("pick a skill"));
    }
    let category = g
        .category
        .as_deref()
        .map(|c| {
            content
                .career_categories
                .iter()
                .position(|d| d.id == c)
                .ok_or_else(|| Error::new(format!("unknown career category '{c}'")))
        })
        .transpose()?;
    let object = g
        .object
        .as_deref()
        .map(|o| content.object_index(o).ok_or_else(|| Error::new(format!("unknown object '{o}'"))))
        .transpose()?;
    let price = match (kind, object) {
        (GoalKind::Own, None) => return Err(Error::new("pick an item")),
        (GoalKind::Own, Some(o)) => match content.objects[o].price {
            Some(p) => Some(p as f32),
            None => return Err(Error::new(format!("{} isn't for sale", content.objects[o].name))),
        },
        _ => None,
    };
    let start = match kind {
        GoalKind::Promoted => sim.job.as_ref().map_or(0.0, |j| j.level as f32),
        _ => 0.0,
    };
    Ok(Goal {
        def,
        skill,
        category,
        object: object.filter(|_| kind == GoalKind::Own),
        target: price.unwrap_or(g.target.unwrap_or(1.0)).max(0.0),
        since_day: day,
        start,
        progress: 0.0,
        history: [0.0; 3],
    })
}
