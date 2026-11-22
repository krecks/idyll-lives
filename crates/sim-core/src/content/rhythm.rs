//! The day's rhythm: sleep and waking, babies waking, work breaks and habits.

use super::*;

/// Day and night: when Sims go to bed and get up, and how sleep changes their needs.
#[derive(Debug, Clone)]
pub struct DayRhythm {
    pub wake_hour: f32,
    pub bed_hour: f32,
    /// Interactions with any of these tags count as sleep.
    pub sleep_tags: TagMask,
    /// Need decay multipliers while awake at night (tiredness builds up).
    pub night_decay: [f32; MAX_NEEDS],
    /// Need decay multipliers while asleep.
    pub asleep_decay: [f32; MAX_NEEDS],
    /// Need decay multipliers while at work (there's lunch and a bathroom there; the job's
    /// `workEffects` come on top).
    pub work_decay: [f32; MAX_NEEDS],
    /// Needs that wake a sleeper when they get urgent (bit per need; default: all).
    pub wake_for: u32,
    /// Sims with an early shift get up this long before they leave.
    pub wake_before_work_minutes: f32,
    /// A rested sleeper stays in bed at night until this many hours before they'd get up.
    pub lie_in_hours: f32,
    /// What each star of a bed adds to sleep (instead of `ObjectRules::quality_bonus`: a better
    /// bed makes a night a little shorter, not half as long).
    pub sleep_quality_bonus: f32,
    /// The need a night's sleep is for (the one beds fill most: energy). Sleepers get up once
    /// it's full.
    pub rest_need: Option<usize>,
    /// Interactions with any of these tags are naps (short, and not again too soon).
    pub nap_tags: TagMask,
    /// What night quiet holds back and favours in the autonomous pick (`NightQuiet`).
    pub night_quiet: NightQuiet,
    /// No nap for this long after the last one.
    pub nap_cooldown_hours: f32,
    /// Below this (in what a bed's sleep fills), daytime is no reason not to go to bed, and a
    /// nap won't do.
    pub exhausted_below: f32,
    /// What residents see to at the turning points of their day (content `habits`).
    pub habits: Vec<Habit>,
    /// Habits before leaving for work or school start this long before they leave.
    pub ready_minutes: f32,
    /// What residents see to while away at work or school (content `dayRhythm.workBreaks`).
    pub work_breaks: Vec<WorkBreak>,
    /// A crying baby wakes a grown-up asleep at home (content `dayRhythm.babyWakes`; None:
    /// nobody wakes for them).
    pub baby_wakes: Option<BabyWakes>,
}

/// Night quiet (content `dayRhythm.nightQuiet`): at night (a Sim's own, `world::is_night`), what
/// the autonomous pick holds back and favours, on top of the `schedule` window for the hour.
#[derive(Debug, Clone, Default)]
pub struct NightQuiet {
    /// `(tag, unless)`: an interaction with `tag` and none of `unless` is noisy.
    pub noisy: Vec<(TagMask, TagMask)>,
    pub noisy_factor: f32,
    /// Interactions with any of these tags are quiet.
    pub quiet: TagMask,
    pub quiet_factor: f32,
}

impl NightQuiet {
    /// What night quiet makes of an interaction with `tags` (1: nothing).
    pub fn factor(&self, tags: TagMask) -> f32 {
        let noisy = self
            .noisy
            .iter()
            .any(|&(tag, unless)| tags & tag != 0 && tags & unless == 0);
        let mut f = if noisy { self.noisy_factor } else { 1.0 };
        if tags & self.quiet != 0 {
            f *= self.quiet_factor;
        }
        f
    }
}

/// A baby crying for one of `needs` (bit per need; default: all but surroundings) below
/// `below` while every grown-up at home is asleep wakes one of them, who does the crib's care
/// interactions for each of those needs below `tend_below` and goes back to bed, with the
/// `feeling` (optional); at night, a grown-up at home who's up and free comes instead (a cook
/// with at most `finish_minutes` of a step left finishes it first). The same baby calls nobody
/// again for `cooldown_ticks`.
#[derive(Debug, Clone, Copy)]
pub struct BabyWakes {
    pub needs: u32,
    pub below: f32,
    pub tend_below: f32,
    pub cooldown_ticks: u64,
    pub finish_minutes: f32,
    pub feeling: Option<usize>,
}

/// A break at work or school: once `need` drops below `below`, it gains `gain` (lunch, the
/// toilet, a coffee). `cost` is paid from the household at work (school lunch is free); a
/// break they can't afford is skipped.
#[derive(Debug, Clone, Copy)]
pub struct WorkBreak {
    pub need: usize,
    pub below: f32,
    pub gain: f32,
    pub cost: i64,
}

/// When a habit is kept (`Habit`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HabitTime {
    /// On getting up after a night's sleep.
    Wake,
    /// On the way to bed.
    Bed,
    /// In the `ready_minutes` before leaving for work or school.
    Leave,
    /// On getting home from work or school.
    Home,
    /// After a meal (a serving eaten).
    Meal,
}

/// Something residents see to at a turning point of their day (content `dayRhythm.habits`):
/// unless `need` is at `below` or better already, they first go to the nearest quick thing
/// that fills it (the toilet on getting up, before bed, before leaving, on getting home and
/// after a meal).
#[derive(Debug, Clone)]
pub struct Habit {
    pub at: HabitTime,
    pub need: usize,
    pub below: f32,
}

impl DayRhythm {
    /// Night for a Sim who gets up at `wake`.
    pub fn is_night(&self, hour: f32, wake: f32) -> bool {
        !(wake..self.bed_hour).contains(&hour)
    }

    /// Multiplier on a night's sleep in a bed of `quality` stars.
    pub fn sleep_quality_factor(&self, quality: u8) -> f32 {
        1.0 + self.sleep_quality_bonus * quality as f32
    }
}
