//! The planner's model: routines, goals, thoughts, wishes and the planner itself.

use super::*;

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
    pub(super) fn touches(&self, day: u32) -> bool {
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
        (
            self.def,
            self.skill,
            self.category,
            self.object,
            self.target,
        ) == (
            other.def,
            other.skill,
            other.category,
            other.object,
            other.target,
        )
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
    Room {
        kind: Option<usize>,
        garden: bool,
        factor: u8,
    },
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
pub(super) const THOUGHT_MINUTES: f32 = 60.0;

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
