//! Daily life: work shifts, pay and promotions, rent and bills, interaction costs, and
//! households visiting each other.
//!
//! Runs as a world-level phase after Sims and conversations each tick, so it can move Sims
//! between "at home", "travelling" and "away at work", pay households, and record story events.
//!
//! Split from `life.rs` (see `mod.rs`).

mod commute;
mod farewell;
mod schedule;
mod visits;
mod work;

pub use commute::*;
pub(crate) use farewell::*;
pub use schedule::*;
pub use visits::*;
pub use work::*;

use serde::{Deserialize, Serialize};

use crate::MINUTES_PER_TICK;
use crate::clock;
use crate::content::{
    CareerLevel, CareerRules, Content, MAX_NEEDS, MAX_SKILLS, Pose, WorkweekRule,
};
use crate::conversation;
use crate::social::{self, EventKind};
use crate::world::{
    Activity, Ctx, GameMode, Phase, Sim, SimBrief, Task, TaskKind, World, affordable, end_activity,
    practise,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub career: usize,
    pub level: usize,
    /// 0..100; a full bar earns a promotion (once the skills are there too).
    pub performance: f32,
    /// Last day a shift was worked or missed (so each day counts once).
    pub last_shift_day: u32,
    /// Mood when leaving for the current shift.
    pub shift_mood: f32,
    /// How happy the Sim is in this job: a running average of the mood they leave for work in.
    pub satisfaction: f32,
    /// Shifts worked in this job.
    pub shifts: u32,
    /// Recent missed shifts (one worked shift makes up for one missed); too many and they're let go.
    pub missed: u8,
    /// Arrived late for the current shift (`careerRules.lateGraceMinutes`; saved as `JobSave::late`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub late: bool,
}

impl Job {
    /// A new job; starting today doesn't count, so the first shift is tomorrow's.
    pub fn new(career: usize, level: usize, today: u32) -> Self {
        Self {
            career,
            level,
            performance: 0.0,
            last_shift_day: today,
            shift_mood: 0.5,
            satisfaction: 0.6,
            shifts: 0,
            missed: 0,
            late: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Visit {
    pub plot: u32,
    /// Guests head home after this tick.
    pub until: u64,
    /// Ticks in a row they've had nothing to do (`visits.boredMinutes`; not saved).
    #[serde(skip)]
    pub idle: u32,
    /// Ticks spent getting to a host to say goodbye (`visits.farewellMinutes`; not saved).
    #[serde(skip)]
    pub farewell: u32,
    /// No host offers them something to eat or drink before this tick (`visits.offer`,
    /// `hosting.rs`); 0: not let in yet. Not saved: after a load it's a while again.
    #[serde(skip)]
    pub offer_at: u64,
    /// When they got there (let in, or at the place: `world::arrive`); 0: not there yet. For
    /// `visits.leastMinutes` they stay whatever, unless a need runs out (`end_visit_if_due`). Not
    /// saved: after a load, as if there a while.
    #[serde(skip)]
    pub arrived: u64,
    /// Their goodbye and way home are queued, within `visits.leastMinutes`, for a need about to
    /// run out (`end_visit_if_due`): withdrawn once it no longer is (seen to there). Not saved.
    #[serde(skip)]
    pub leaving: bool,
    /// An outing that ends in time to be home by `visits.outingHomeBy` and before they leave for
    /// work or school (`world::outing_leave_by`): its end comes forward as they wander further
    /// from the way home. Not saved: after a load it keeps the end it has.
    #[serde(skip)]
    pub home_by: bool,
}

/// A change in where a Sim is (or what it finished), raised by tasks and handled by `life::update`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transition {
    /// Reached the town exit: the shift starts.
    Work,
    /// Got into a car to go to work in (`cars.rs`): off they go.
    Board { car: u32 },
    /// Set off to visit a plot (guest status starts now, so an interrupted trip still ends at home).
    /// Visits the player asked for last longer.
    Visit { plot: u32, directed: bool },
    /// At their car's door, beyond walking range of the place they're off to: they drive there
    /// (`cars::start_trip`).
    Drive { car: u32, plot: u32, directed: bool },
    /// At their car parked out on a trip: they drive home (`cars::drive_home`).
    DriveHome { car: u32 },
    /// At the door of their household's car on a trip: they get in to ride along to `plot`, or
    /// (None) home (`cars::ride`).
    Ride {
        car: u32,
        plot: Option<u32>,
        directed: bool,
    },
    /// Off to `plot` (a visit or an outing) by the taxi: booked on one (`taxi::book_leisure`).
    Taxi { plot: u32, directed: bool },
    /// Reached home.
    Home,
    /// Reached the front door to answer `guest`'s knock.
    Answered { guest: u32 },
}

/// Skill levels above (+) or below (-) what a job level asks for; the weakest skill counts.
pub fn skill_fit(level: &CareerLevel, skills: &[f32; MAX_SKILLS]) -> f32 {
    level
        .requires
        .iter()
        .map(|&(s, need)| skills[s].floor() - need)
        .fold(None, |m: Option<f32>, f| Some(m.map_or(f, |m| m.min(f))))
        .unwrap_or(0.0)
}

/// How much one shift fills the performance bar (0..100): all a shift's worth is worked out here.
/// At the entry grade, for a Sim in a middling mood (0.5) with just the skills the job asks for,
/// it's `performancePerShift`. Leaving for work in a better mood adds up to `performancePerMood`
/// (a worse one takes as much off); each skill level above the job's requirements adds
/// `performancePerFit` (each one short takes it off; three count at most). Arriving more than
/// `lateGraceMinutes` late, the shift counts `lateShare` as much. Each grade up it counts less
/// (`grade_pace`), so later promotions take longer. A shift worked never takes any off (a missed
/// one does).
pub fn shift_performance(
    rules: &CareerRules,
    level: usize,
    mood: f32,
    fit: f32,
    late: bool,
) -> f32 {
    let worth = rules.performance_per_shift
        + (mood - 0.5) * 2.0 * rules.performance_per_mood
        + fit.clamp(-3.0, 3.0) * rules.performance_per_fit;
    let late = if late { rules.late_share } else { 1.0 };
    worth.max(0.0) * late * grade_pace(rules, level)
}

/// What shifts (and missed ones) count for at a grade (0: the entry grade), as a share of the
/// entry grade's: 1 / (1 + grade × `performanceSlowdown`).
pub fn grade_pace(rules: &CareerRules, level: usize) -> f32 {
    1.0 / (1.0 + level as f32 * rules.performance_slowdown)
}

/// Shifts still to work for a full bar, each worth what one is now (in the mood they last left
/// for work in, on time); None at the top of the career, or when shifts don't fill it.
pub fn shifts_to_full_bar(content: &Content, job: &Job, skills: &[f32; MAX_SKILLS]) -> Option<u32> {
    let career = &content.careers[job.career];
    career.levels.get(job.level + 1)?;
    let fit = skill_fit(&career.levels[job.level], skills);
    let worth = shift_performance(&content.career_rules, job.level, job.shift_mood, fit, false);
    let left = (100.0 - job.performance).max(0.0);
    (worth > 0.0).then(|| (left / worth).ceil() as u32)
}

/// The workweek rule for a fit (probation, standard, flexible...).
pub fn workweek_rule(content: &Content, fit: f32) -> Option<&WorkweekRule> {
    content
        .career_rules
        .workweek
        .iter()
        .rev()
        .find(|r| r.fit <= fit)
}

/// Adds days after the last workday, or drops days from the end of the week (at least one stays).
pub fn adjust_days(days: u8, delta: i8) -> u8 {
    let mut days = days & 0x7f;
    if delta > 0 {
        for _ in 0..delta {
            let last = (0..7).rev().find(|d| days & (1 << d) != 0).unwrap_or(0);
            if let Some(d) = (1..7)
                .map(|k| (last + k) % 7)
                .find(|d| days & (1 << d) == 0)
            {
                days |= 1 << d;
            }
        }
    } else {
        for _ in 0..-delta {
            if days.count_ones() <= 1 {
                break;
            }
            let last = (0..7).rev().find(|d| days & (1 << d) != 0).unwrap_or(0);
            days &= !(1 << last);
        }
    }
    days
}

/// This Sim's workdays for its job: the standard week, shortened or lengthened by skill.
pub fn work_days(content: &Content, job: &Job, skills: &[f32; MAX_SKILLS]) -> u8 {
    let level = &content.careers[job.career].levels[job.level];
    let delta = workweek_rule(content, skill_fit(level, skills)).map_or(0, |r| r.days);
    adjust_days(level.days, delta)
}

/// Pay for one shift. The weekly salary is fixed, so a shorter week pays more per shift.
pub fn shift_pay(content: &Content, job: &Job, skills: &[f32; MAX_SKILLS]) -> i64 {
    let level = &content.careers[job.career].levels[job.level];
    let standard = level.days.count_ones().max(1) as i64;
    let actual = work_days(content, job, skills).count_ones().max(1) as i64;
    // Saturating like the promotion bonus: content's pay must not panic the week's sum.
    level
        .pay
        .saturating_mul(standard)
        .saturating_add(actual / 2)
        / actual
}

/// Whether a Sim with `skills` may take `level` (probation allows a small shortfall).
pub fn can_join(content: &Content, level: &CareerLevel, skills: &[f32; MAX_SKILLS]) -> bool {
    skill_fit(level, skills) >= -content.career_rules.probation_levels
}

/// Today's shift for a job as `(leave, start, end)` ticks, if today is a workday.
pub fn shift_today(
    content: &Content,
    job: &Job,
    skills: &[f32; MAX_SKILLS],
    tick: u64,
) -> Option<(u64, u64, u64)> {
    let level = &content.careers[job.career].levels[job.level];
    let day = clock::day(tick);
    if work_days(content, job, skills) & (1 << clock::weekday(day)) == 0 {
        return None;
    }
    let start = clock::tick_at(day, level.start_hour * 60.0)?;
    let leave = start.saturating_sub(ticks(content.career_rules.commute_minutes));
    Some((leave, start, start + ticks(level.hours * 60.0)))
}

/// Game minutes as ticks (rounded down).
pub(crate) fn ticks(minutes: f32) -> u64 {
    (minutes * crate::TICKS_PER_MINUTE as f32) as u64
}

#[cfg(test)]
mod tests;
