//! Daily life: work shifts, pay and promotions, rent and bills, interaction costs, and households
//! visiting each other.
//!
//! Runs as a world-level phase after Sims and conversations each tick, so it can
//! move Sims between "at home", "travelling" and "away at work", pay households,
//! and record story events.

use serde::{Deserialize, Serialize};

use crate::MINUTES_PER_TICK;
use crate::clock;
use crate::content::{CareerLevel, Content, MAX_NEEDS, MAX_SKILLS, Pose, WorkweekRule};
use crate::conversation;
use crate::social::{self, EventKind};
use crate::world::{Activity, Ctx, GameMode, Phase, Sim, SimBrief, Task, TaskKind, World, affordable, end_activity, practise};

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
}

/// A change in where a Sim is (or what it finished), raised by tasks and handled by `life::update`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transition {
    /// Reached the town exit: the shift starts.
    Work,
    /// Set off to visit a plot (guest status starts now, so an interrupted trip still ends at home).
    /// Visits the player asked for last longer.
    Visit { plot: u32, directed: bool },
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
    (level.pay * standard + actual / 2) / actual
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

fn ticks(minutes: f32) -> u64 {
    (minutes / MINUTES_PER_TICK) as u64
}

pub(crate) fn update(w: &mut World) {
    let tick = w.tick;
    let day = clock::day(tick);
    let hour = clock::hour(tick);
    for i in 0..w.sims.len() {
        // (In lots nobody watches, on their turn; see `lod.rs`.)
        if !w.sims[i].here() || w.sims[i].steps == 0 {
            continue;
        }
        // Interactions started this tick are paid from the Sim's own household (guests too).
        let spend = std::mem::take(&mut w.sims[i].pending_spend);
        w.households[w.sims[i].household as usize].funds -= spend;
        // A load of washing washed (it waits to be dried), or dried and folded.
        let worked = std::mem::take(&mut w.sims[i].worked);
        let laundry = &mut w.households[w.sims[i].household as usize].laundry;
        if worked & crate::content::WorkOn::Laundry.bit() != 0 && laundry[0] >= 1.0 {
            laundry[0] -= 1.0;
            laundry[1] = (laundry[1] + 1.0).min(w.content.laundry.map_or(1.0, |l| l.most));
        }
        if worked & crate::content::WorkOn::Washed.bit() != 0 {
            laundry[1] = (laundry[1] - 1.0).max(0.0);
        }
        if let Some(transition) = w.sims[i].transition.take() {
            apply_transition(w, i, transition);
        }
        if w.sims[i].away_until.is_some() {
            at_work(w, i);
        } else {
            schedule_work(w, i, day);
            schedule_school(w, i, day);
            end_visit_if_due(w, i, hour);
        }
        childcare(w, i);
        report_skill_ups(w, i);
    }
    doors(w);
    charge_rent(w, day);
    job_market(w, day);
}

/// Once a day: unemployed residents with free will below the retirement age look for work, and
/// residents who have been unhappy at work for a while may quit.
fn job_market(w: &mut World, day: u32) {
    let Some(market) = w.content.career_rules.market.clone() else {
        return;
    };
    if !clock::at_minute(w.tick, market.hour * 60.0) || !w.autonomy {
        return;
    }
    let tick = w.tick;
    for i in 0..w.sims.len() {
        let h = w.sims[i].household as usize;
        if !w.households[h].free_will
            || w.sims[i].away_until.is_some()
            || !w.sims[i].here()
            || w.sims[i].retired
            || !w.sims[i].adult(&w.content)
        {
            continue;
        }
        if let Some(job) = &w.sims[i].job {
            let unhappy = job.shifts >= market.min_shifts_before_quit
                && job.satisfaction < market.quit_below;
            if unhappy && w.rng.next_f32() < market.quit_chance {
                let (career, level) = (job.career, job.level);
                let sim = &mut w.sims[i];
                sim.job = None;
                sim.job_search_from = day + market.cooldown_days;
                if let Some(m) = market.quit_feeling {
                    social::add_feeling(&mut sim.feelings, m, &w.content.feelings, tick);
                }
                w.events
                    .push_career(tick, EventKind::QuitJob, i, career, Some(level as i64));
            }
            continue;
        }
        // Nobody's hired from the retirement age on (they retire instead; a job the player gives
        // them is theirs to keep).
        let retirement_age = w.content.life.retire_at.is_some_and(|at| w.sims[i].age >= at);
        if day < w.sims[i].job_search_from || retirement_age {
            continue;
        }
        // Short on money, or set on finding work (a goal): look harder.
        let funds = w.households[h].funds;
        let weekly = weekly_costs(w, h).map_or(0, |(rent, bills)| rent + bills);
        let (keen, category) = crate::planner::job_search(&w.content, &w.sims[i]);
        let chance = market.daily_chance * keen * if funds < weekly * 2 { 2.0 } else { 1.0 };
        if w.rng.next_f32() >= chance {
            continue;
        }
        let Some((career, level)) = find_job(w, i, category).or_else(|| find_job(w, i, None)) else {
            continue;
        };
        let sim = &mut w.sims[i];
        sim.job = Some(Job::new(career, level, day));
        if let Some(m) = market.hired_feeling {
            social::add_feeling(&mut sim.feelings, m, &w.content.feelings, tick);
        }
        w.events
            .push_career(tick, EventKind::JobFound, i, career, Some(level as i64));
    }
}

/// A job Sim `i` could get (in `category`, if set): for each career the best level they're
/// qualified for (the entry level on probation otherwise), weighed by pay with some chance.
fn find_job(w: &mut World, i: usize, category: Option<usize>) -> Option<(usize, usize)> {
    let World {
        content, sims, rng, ..
    } = w;
    let skills = &sims[i].skills;
    let mut candidates: Vec<(f32, (usize, usize))> = Vec::new();
    for (c, career) in content.careers.iter().enumerate() {
        if category.is_some_and(|cat| career.category != Some(cat)) {
            continue;
        }
        let level = career
            .levels
            .iter()
            .rposition(|l| skill_fit(l, skills) >= 0.0)
            .or_else(|| can_join(content, &career.levels[0], skills).then_some(0));
        let Some(level) = level else { continue };
        let def = &career.levels[level];
        let weekly_pay = (def.pay * def.days.count_ones() as i64).max(1) as f32;
        let score = weekly_pay.sqrt() * rng.range(0.5, 1.5);
        candidates.push((score, (c, level)));
    }
    crate::ai::choose(&mut candidates, rng.next_f32(), 3)
}

/// Today's school day for a pupil (a stage that goes to school, no job): when they leave, when
/// it starts, when it ends.
pub fn school_today(content: &Content, sim: &crate::world::Sim, tick: u64) -> Option<(u64, u64, u64)> {
    let rules = content.life.school.as_ref()?;
    if sim.job.is_some() || !content.life.stage_at(sim.age).is_some_and(|s| s.school) {
        return None;
    }
    let day = clock::day(tick);
    if rules.days & (1 << clock::weekday(day)) == 0 {
        return None;
    }
    let start = clock::tick_at(day, rules.start * 60.0)?;
    let leave = start.saturating_sub(ticks(content.career_rules.commute_minutes));
    Some((leave, start, start + ticks(rules.hours * 60.0)))
}

/// Off to school when it's time (once a day; too late, and it's skipped for today).
fn schedule_school(w: &mut World, i: usize, day: u32) {
    let World { content, sims, objects, tick, .. } = w;
    let tick = *tick;
    let sim = &mut sims[i];
    if sim.school_day == day {
        return;
    }
    let Some((leave, start, _)) = school_today(content, sim, tick) else { return };
    if tick > start + ticks(content.career_rules.late_minutes) {
        // A day missed.
        sim.school_day = day;
        if let Some(school) = &content.life.school {
            sim.grade = (sim.grade - school.missed).clamp(0.0, 100.0);
        }
        return;
    }
    if tick < leave {
        return;
    }
    if !heading_out(sim) {
        end_activity(sim, content, objects);
        sim.engaged_with = None;
        sim.queue.push_front(Task { kind: TaskKind::Work, directed: true, routine: false });
    }
}

/// Leave for work when it's time; count a shift as missed if the Sim never left. The last
/// grown-up at home with a baby and no money for a babysitter stays home instead.
fn schedule_work(w: &mut World, i: usize, day: u32) {
    let tick = w.tick;
    let sim = &w.sims[i];
    let Some(job) = sim.job.as_ref() else { return };
    if job.last_shift_day == day {
        return;
    }
    let Some((leave, start, _)) = shift_today(&w.content, job, &sim.skills, tick) else {
        return;
    };
    if tick > start + ticks(w.content.career_rules.late_minutes) {
        miss_shift(w, i, day);
        return;
    }
    if tick < leave || heading_out(sim) {
        return;
    }
    if let Some(baby) = stays_home_for(w, i) {
        // No shift today, and no pay: not held against them at work.
        if let Some(job) = w.sims[i].job.as_mut() {
            job.last_shift_day = day;
        }
        w.events.push(tick, EventKind::StayedHome, i, baby, None);
        return;
    }
    // Work comes first: drop what they're doing and go.
    let World { content, sims, objects, .. } = w;
    let sim = &mut sims[i];
    end_activity(sim, content, objects);
    sim.engaged_with = None;
    sim.queue.push_front(Task {
        kind: TaskKind::Work,
        directed: true,
        routine: false,
    });
}

/// On their way to work or school (or about to set off).
fn heading_out(sim: &Sim) -> bool {
    matches!(sim.current.as_ref().map(|a| a.task.kind), Some(TaskKind::Work)) || matches!(sim.queue.front().map(|t| t.kind), Some(TaskKind::Work))
}

/// A shift they never left for.
fn miss_shift(w: &mut World, i: usize, day: u32) {
    let World { content, sims, events, tick, .. } = w;
    let tick = *tick;
    let sim = &mut sims[i];
    let Some(job) = sim.job.as_mut() else { return };
    job.last_shift_day = day;
    job.performance = (job.performance - content.career_rules.missed_penalty).max(0.0);
    job.missed = job.missed.saturating_add(1);
    if let Some(m) = content.career_rules.missed_feeling {
        social::add_feeling(&mut sim.feelings, m, &content.feelings, tick);
    }
    events.push(tick, EventKind::MissedWork, i, i, None);
    if let Some(market) = &content.career_rules.market
        && job.missed >= market.fire_after_missed
    {
        let (career, level) = (job.career, job.level);
        sim.job = None;
        sim.job_search_from = day + market.cooldown_days;
        if let Some(m) = market.fired_feeling {
            social::add_feeling(&mut sim.feelings, m, &content.feelings, tick);
        }
        events.push_career(tick, EventKind::Fired, i, career, Some(level as i64));
    }
}

// ---- Childcare ----------------------------------------------------------------------------

/// Whether `task` takes someone whose home is `home` off that plot: to work, on a visit, over
/// to theirs after a call, or to use, repair or tidy something, walk somewhere, or chat with
/// or join someone anywhere else (a public place, a neighbour's, the street).
fn takes_off(w: &World, task: &TaskKind, home: u32) -> bool {
    let away = |plot: Option<u32>| plot != Some(home);
    let of = |sim: u32| w.sims.get(sim as usize).and_then(|t| w.plot_at(t.tile().0, t.tile().1));
    match *task {
        TaskKind::Work => true,
        TaskKind::Visit { plot } => plot != home,
        TaskKind::Use { object, .. } | TaskKind::Repair { object } => away(w.object_plot.get(object as usize).copied().flatten()),
        TaskKind::MoveTo { x, z } => away(w.plot_at(x.floor() as i32, z.floor() as i32)),
        TaskKind::Clean { x, z } => away(w.plot_at(x, z)),
        TaskKind::Social { target, .. } => away(of(target)),
        TaskKind::Join { leader } => away(of(leader)),
        TaskKind::Phone { action, answer, .. } => {
            w.content.phone.actions.get(action).is_some_and(|a| a.meet == Some(crate::phone::Meet::There))
                && matches!(answer, crate::phone::Answer::Pending | crate::phone::Answer::Yes)
        }
        TaskKind::GoHome | TaskKind::Answer { .. } | TaskKind::Spot { .. } | TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. } | TaskKind::Carry { .. } => false,
    }
}

/// Whether `s` is on their way off their home plot (or about to be): what they're doing or
/// anything they're yet to do takes them off it (`takes_off`). The one test of who's leaving
/// for childcare, whether at work time, for a player's order, on the phone or by their own
/// choice.
pub(crate) fn leaving_home(w: &World, s: &Sim) -> bool {
    let Some(home) = w.households.get(s.household as usize).and_then(|h| h.plot) else { return false };
    s.current.as_ref().map(|a| &a.task).into_iter().chain(&s.queue).any(|t| takes_off(w, &t.kind, home))
}

/// Whether a grown-up of household `h` other than `except` is at home on `plot` to look after
/// a baby (not on their way out: `leaving_home`; back from a visit once they're on the lot).
fn carer_home(w: &World, h: usize, plot: u32, except: Option<usize>) -> bool {
    w.sims.iter().enumerate().any(|(j, s)| {
        Some(j) != except
            && s.here()
            && s.household as usize == h
            && s.away_until.is_none()
            && !leaving_home(w, s)
            && s.adult(&w.content)
            && w.plot_at(s.tile().0, s.tile().1) == Some(plot)
    })
}

/// Who `i` would stay home for if they left for work now: a baby at home with no other
/// grown-up there, no babysitter booked for today and no money for one (content
/// `life.childcare`; without it, nobody stays home).
fn stays_home_for(w: &World, i: usize) -> Option<usize> {
    let rules = w.content.life.childcare?;
    let h = w.sims[i].household as usize;
    let household = &w.households[h];
    let plot = household.plot?;
    if household.sitter_day == Some(clock::day(w.tick)) || affordable(rules.cost, household.funds) {
        return None;
    }
    let baby = w.sims.iter().position(|s| s.here() && s.household as usize == h && w.content.life.baby(s.age))?;
    (!carer_home(w, h, plot, Some(i))).then_some(baby)
}

/// Who `i` would leave alone at home by going out now (visiting, say): a baby there with no
/// other grown-up at home, no babysitter booked for today and no money for one. Only the last
/// grown-up at home can.
pub(crate) fn no_cover_for(w: &World, i: usize) -> Option<usize> {
    let s = &w.sims[i];
    let plot = w.households[s.household as usize].plot?;
    let home = s.away_until.is_none() && s.visiting.is_none() && w.plot_at(s.tile().0, s.tile().1) == Some(plot);
    home.then(|| stays_home_for(w, i)).flatten()
}

/// Why going out would leave a baby home alone (`homebound`), short for the UI.
pub const NO_SITTER: &str = "Nobody to mind the baby";

/// Why `i` can't be sent out (off the home lot: visiting, the park, over to a friend's) now:
/// `NO_SITTER` when that would leave a baby alone at home (`no_cover_for`, as `check_cover`
/// refuses), else None. For the UI, next to what would take them out.
pub fn homebound(w: &World, i: usize) -> Option<&'static str> {
    no_cover_for(w, i).map(|_| NO_SITTER)
}

/// Refuses a player's order that takes `i` off the home lot (a visit, the park, the shops,
/// over to a friend's) when it would leave a baby alone at home: `no_cover_for`.
pub(crate) fn check_cover(w: &World, i: usize) -> Result<(), crate::Error> {
    let Some(baby) = no_cover_for(w, i) else { return Ok(()) };
    let cost = w.content.life.childcare.map_or(0, |c| c.cost);
    Err(crate::Error::new(format!("nobody to look after {}: not enough money for a babysitter (${cost})", w.sims[baby].name)))
}

/// Refuses a player's order that takes `i` to `plot` (None: off every plot) when that's away
/// from home and would leave a baby alone there (`check_cover`); orders at home go ahead.
pub(crate) fn check_cover_at(w: &World, i: usize, plot: Option<u32>) -> Result<(), crate::Error> {
    if plot != w.households[w.sims[i].household as usize].plot { check_cover(w, i) } else { Ok(()) }
}

/// Whether `sim` is the last grown-up at home with a baby of theirs there (content
/// `life.childcare`): they don't go out on their own (visits, outings, meeting up, a chat off
/// the lot) and leave the baby alone; sent out by the player, a babysitter comes (see
/// `childcare`). Anyone else on their way out (`SimBrief::leaving`) isn't home for long.
pub(crate) fn minding_baby(sim: &Sim, ctx: &Ctx) -> bool {
    let content = ctx.content;
    if content.life.childcare.is_none() || !sim.adult(content) {
        return false;
    }
    let Some(home) = ctx.households[sim.household as usize].plot else { return false };
    let me = sim.id as usize;
    let at_home = |b: &SimBrief| b.here && !b.away && b.household == sim.household && b.plot == Some(home);
    at_home(&ctx.briefs[me]) && ctx.briefs.iter().any(|b| b.baby && at_home(b)) && !ctx.briefs.iter().enumerate().any(|(j, b)| j != me && b.adult && !b.leaving && at_home(b))
}

/// A babysitter for baby `b` (content `life.childcare`): booked (and paid) for the day once
/// the baby is left with no grown-up at home while they're out (at work, visiting, wherever);
/// while nobody's home, the babysitter does the care interactions of the baby's crib whenever
/// a need they fill is low.
fn childcare(w: &mut World, b: usize) {
    let Some(rules) = w.content.life.childcare else { return };
    if !w.content.life.baby(w.sims[b].age) {
        return;
    }
    let h = w.sims[b].household as usize;
    let Some(plot) = w.households[h].plot else { return };
    if carer_home(w, h, plot, None) {
        return;
    }
    // The baby's crib (or the home's), for what caring for them takes.
    let content = &w.content;
    let is_crib = |o: &crate::world::ObjectInstance| content.objects[o.def].interactions.iter().any(|i| i.baby_bed());
    let crib = w
        .objects
        .iter()
        .find(|o| o.users().any(|u| u as usize == b))
        .or_else(|| w.objects.iter().zip(&w.object_plot).find(|&(o, &p)| p == Some(plot) && is_crib(o)).map(|(o, _)| o))
        .map(|o| o.def);
    let Some(crib) = crib else { return };
    let day = clock::day(w.tick);
    if w.households[h].sitter_day != Some(day) {
        // Booked by a grown-up out at work, or else whoever else is out (with no grown-up in
        // the household at all, there's nobody to book one).
        let out = |s: &&Sim| s.here() && s.household as usize == h && s.adult(&w.content);
        let at_work = w.sims.iter().position(|s| out(&s) && s.job.is_some() && s.away_until.is_some());
        let parent = at_work.or_else(|| w.sims.iter().position(|s| out(&s)));
        let Some(parent) = parent.filter(|_| affordable(rules.cost, w.households[h].funds)) else { return };
        w.households[h].funds -= rules.cost;
        w.households[h].sitter_day = Some(day);
        w.events.push_amount(w.tick, EventKind::Babysitter, parent, b, rules.cost);
    }
    let content = &w.content;
    let baby = &mut w.sims[b];
    for inter in content.objects[crib].interactions.iter().filter(|i| i.cares()) {
        let low = (0..content.needs.len()).any(|n| inter.care[n] > 0.0 && baby.needs[n] < rules.below);
        if low {
            for (n, g) in inter.care.iter().enumerate().take(content.needs.len()) {
                baby.needs[n] = (baby.needs[n] + g).clamp(0.0, 1.0);
            }
        }
    }
}

fn apply_transition(w: &mut World, i: usize, transition: Transition) {
    let tick = w.tick;
    match transition {
        Transition::Work => {
            let content = &w.content;
            let mood = w.sims[i].mood(content);
            let sim = &mut w.sims[i];
            let skills = sim.skills;
            // A pupil: school until the afternoon.
            if sim.job.is_none() {
                if let Some((_, start, end)) = school_today(content, sim, tick) {
                    sim.school_day = clock::day(tick);
                    sim.away_until = Some(end.max(start).max(tick + 1));
                    sim.queue.clear();
                }
                return;
            }
            let Some(job) = sim.job.as_mut() else { return };
            let Some((_, start, end)) = shift_today(content, job, &skills, tick) else {
                return;
            };
            job.last_shift_day = clock::day(tick);
            job.shift_mood = mood;
            // Early arrivals still work until the shift's end; late ones lose that time.
            sim.away_until = Some(end.max(start).max(tick + 1));
            sim.queue.clear();
        }
        Transition::Visit { plot, directed } => {
            let visits = &w.content.visits;
            let hours = if directed {
                visits.directed_hours
            } else {
                visits.hours
            };
            let until = tick + ticks(hours * 60.0);
            w.sims[i].visiting = Some(Visit { plot, until, idle: 0, farewell: 0 });
            // With door rules the story is told when they're let in (`answer`).
            if w.content.visits.door.is_some() && !w.plots[plot as usize].public {
                return;
            }
            // Tell the story if the hosts are home.
            let host = w
                .households
                .iter()
                .find(|h| h.plot == Some(plot))
                .map(|h| h.id);
            if let Some(host) = host
                && let Some(j) = w.sims.iter().position(|s| {
                    s.here() && s.household == host && w.plot_at(s.tile().0, s.tile().1) == Some(plot)
                })
            {
                w.events.push(tick, EventKind::Visited, i, j, None);
            }
        }
        Transition::Home => w.sims[i].visiting = None,
        Transition::Answered { guest } => answer(w, i, guest as usize),
    }
}

/// The plot a visitor is knocking at, and how long they've waited and who is coming, if they are.
fn knocking(sim: &crate::world::Sim) -> Option<(u32, u32, Option<u32>)> {
    match sim.current.as_ref()? {
        Activity { task: Task { kind: TaskKind::Visit { plot }, .. }, phase: Phase::Knocking { ticks, host }, .. } => Some((*plot, *ticks, *host)),
        _ => None,
    }
}

/// Visitors at front doors (`visits.door`): someone at home is sent to answer; with nobody to
/// answer, the visitor gives up after a while and goes home.
fn doors(w: &mut World) {
    let Some(rules) = w.content.visits.door.clone() else { return };
    let tick = w.tick;
    let wait = ticks(rules.wait_minutes);
    for i in 0..w.sims.len() {
        let Some((plot, waited, host)) = knocking(&w.sims[i]) else { continue };
        let waited = u64::from(waited);
        // Still coming? (They may have been called away, gone to bed or to work.)
        let host = host.filter(|&h| {
            let s = &w.sims[h as usize];
            let answering = |t: &Task| matches!(t.kind, TaskKind::Answer { guest } if guest as usize == i);
            s.current.as_ref().is_some_and(|a| answering(&a.task)) || s.queue.iter().any(answering)
        });
        let host = host.or_else(|| {
            let h = door_host(w, plot, i)?;
            let World { sims, content, objects, .. } = &mut *w;
            let s = &mut sims[h];
            if s.current.is_some() {
                end_activity(s, content, objects);
            }
            s.queue.push_front(Task { kind: TaskKind::Answer { guest: i as u32 }, directed: false, routine: false });
            Some(h as u32)
        });
        if let Some(Activity { phase: Phase::Knocking { host: h, .. }, .. }) = w.sims[i].current.as_mut() {
            *h = host;
        }
        // Nobody comes (or whoever was coming never makes it): home again.
        if (host.is_none() && waited >= wait) || waited >= wait * 3 {
            let sim = &mut w.sims[i];
            sim.current = None;
            if !sim.queue.iter().any(|t| matches!(t.kind, TaskKind::GoHome)) {
                sim.queue.push_front(Task { kind: TaskKind::GoHome, directed: false, routine: false });
            }
            let household = w.households.iter().position(|h| h.plot == Some(plot));
            if let Some(member) = household.and_then(|h| w.sims.iter().position(|s| s.here() && s.household as usize == h)) {
                w.events.push(tick, EventKind::NobodyHome, i, member, None);
            }
        }
    }
}

/// Who answers the door of `plot`: the grown-up of the household living there who is nearest to
/// the door, at home, awake, and not busy with something the player asked for or a conversation.
fn door_host(w: &World, plot: u32, guest: usize) -> Option<usize> {
    let household = w.households.iter().position(|h| h.plot == Some(plot))?;
    let door = w.plots[plot as usize].arrival_point();
    let free = |s: &crate::world::Sim| {
        s.current.as_ref().is_none_or(|a| {
            !a.task.directed
                && !matches!(a.phase, Phase::Conversing { .. })
                && !matches!(a.task.kind, TaskKind::Work | TaskKind::GoHome | TaskKind::Visit { .. } | TaskKind::Answer { .. })
        }) && !s.queue.front().is_some_and(|t| t.directed)
    };
    w.sims
        .iter()
        .enumerate()
        .filter(|&(j, s)| {
            j != guest
                && s.here()
                && s.household as usize == household
                && s.away_until.is_none()
                && s.visiting.is_none()
                && s.pose != Pose::Lie
                && s.engaged_with.is_none()
                && s.adult(&w.content)
                && w.plot_at(s.tile().0, s.tile().1) == Some(plot)
                && free(s)
        })
        .map(|(j, s)| (j, (s.pos[0] - door[0]).hypot(s.pos[1] - door[1])))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(j, _)| j)
}

/// Host `h` opened the door to `guest`: friends (and acquaintances) come in, people the host
/// can't stand and visitors at an unreasonable hour are sent away.
fn answer(w: &mut World, h: usize, guest: usize) {
    let Some(rules) = w.content.visits.door.clone() else { return };
    if w.sims.get(guest).and_then(knocking).is_none() {
        return;
    }
    let tick = w.tick;
    let hour = clock::hour(tick);
    let visits = &w.content.visits;
    let sensible_hour = (visits.earliest_hour - 2.0..=visits.latest_hour + 2.0).contains(&hour);
    let welcome = sensible_hour && w.relationships.get(h, guest).friendship >= rules.welcome_friendship;
    w.sims[guest].current = None;
    if welcome {
        w.events.push(tick, EventKind::Visited, guest, h, None);
        if let Some(greet) = rules.greet {
            w.sims[h].queue.push_front(Task { kind: TaskKind::Social { target: guest as u32, social: greet }, directed: false, routine: false });
        }
        return;
    }
    let World { sims, content, events, .. } = &mut *w;
    let sim = &mut sims[guest];
    sim.queue.push_front(Task { kind: TaskKind::GoHome, directed: false, routine: false });
    if let Some(m) = rules.turned_away {
        social::add_feeling(&mut sim.feelings, m, &content.feelings, tick);
    }
    events.push(tick, EventKind::TurnedAway, guest, h, None);
}

/// Lunch, the toilet, a coffee: at work (paid from `funds`) or school (free), each need below
/// its break gets topped up, more than once if a long catch-up step took it far down.
fn take_breaks(sim: &mut Sim, content: &Content, mut funds: Option<&mut i64>) {
    for b in &content.day_rhythm.work_breaks {
        for _ in 0..4 {
            if sim.needs[b.need] >= b.below {
                break;
            }
            if let Some(funds) = funds.as_deref_mut() {
                if !affordable(b.cost, *funds) {
                    break;
                }
                *funds -= b.cost;
            }
            sim.needs[b.need] = (sim.needs[b.need] + b.gain).min(1.0);
        }
    }
}

/// Off the map at work: needs change and skills grow, then come home with pay and maybe a promotion.
fn at_work(w: &mut World, i: usize) {
    let tick = w.tick;
    let World {
        content,
        sims,
        households,
        events,
        ..
    } = w;
    let sim = &mut sims[i];
    let minutes = sim.minutes();
    let Some(job) = sim.job.clone() else {
        take_breaks(sim, content, None);
        // At school: a little of the school's skills, then home.
        if let Some(school) = &content.life.school {
            for &(skill, weight) in &school.skills {
                practise(sim, content, skill, content.skill_rules.work_gain_per_hour * weight, minutes);
            }
        }
        if sim.away_until.is_some_and(|until| tick < until) {
            return;
        }
        // A day at school: the better they felt, the better it went.
        if let Some(school) = &content.life.school {
            let mood = sim.mood(content);
            sim.grade = (sim.grade + school.attend + (mood - 0.5) * 2.0 * school.mood).clamp(0.0, 100.0);
        }
        sim.away_until = None;
        sim.queue.push_back(Task { kind: TaskKind::GoHome, directed: false, routine: false });
        return;
    };
    let career = &content.careers[job.career];
    let level = &career.levels[job.level];
    let per_tick = minutes / (level.hours * 60.0);
    for n in 0..MAX_NEEDS.min(content.needs.len()) {
        sim.needs[n] = (sim.needs[n] + career.work_effects[n] * per_tick).clamp(0.0, 1.0);
    }
    take_breaks(sim, content, Some(&mut households[sim.household as usize].funds));
    for &(skill, weight) in &career.skills {
        practise(
            sim,
            content,
            skill,
            content.skill_rules.work_gain_per_hour * weight,
            minutes,
        );
    }
    if sim.away_until.is_some_and(|until| tick < until) {
        return;
    }
    sim.away_until = None;
    households[sim.household as usize].funds += shift_pay(content, &job, &sim.skills);
    let rules = &content.career_rules;
    let fit = skill_fit(level, &sim.skills).clamp(-3.0, 3.0);
    let job = sim.job.as_mut().expect("checked above");
    job.shifts += 1;
    job.missed = job.missed.saturating_sub(1);
    job.satisfaction = job.satisfaction * 0.8 + job.shift_mood * 0.2;
    job.performance = (job.performance
        + rules.performance_per_shift
        + (job.shift_mood - 0.5) * 2.0 * rules.performance_per_mood
        + fit * rules.performance_per_fit)
        .clamp(0.0, 100.0);
    // A promotion needs a full bar and the skills for the next level.
    if job.performance >= 100.0
        && let Some(next) = career.levels.get(job.level + 1)
        && skill_fit(next, &sim.skills) >= 0.0
    {
        job.level += 1;
        job.performance = 0.0;
        sim.planner.recheck = true;
        if let Some(m) = rules.promotion_feeling {
            social::add_feeling(&mut sim.feelings, m, &content.feelings, tick);
        }
        events.push(tick, EventKind::Promoted, i, i, None);
    }
    sim.queue.push_back(Task {
        kind: TaskKind::GoHome,
        directed: false,
        routine: false,
    });
}

/// Story events for new skill levels (the player's household only: across a whole town they
/// would crowd out everything else).
fn report_skill_ups(w: &mut World, i: usize) {
    let sim = &mut w.sims[i];
    let mut ups = std::mem::take(&mut sim.skill_ups);
    if ups == 0 || !w.households[sim.household as usize].player {
        return;
    }
    while ups != 0 {
        let s = ups.trailing_zeros() as usize;
        ups &= ups - 1;
        let level = sim.skills[s].floor() as i64;
        w.events
            .push_detail(w.tick, EventKind::SkillUp, i, Some(level), Some(s));
    }
}

/// What household `h` pays each week: `(rent, bills)`, or `None` if it has no home or
/// nothing is charged (nobody lives there yet, or it's the player's home in a Creative game). Bills grow with the value of
/// everything the household owns and with how many live there.
pub fn weekly_costs(w: &World, h: usize) -> Option<(i64, i64)> {
    let rent = w.content.rent.as_ref()?;
    let household = w.households.get(h)?;
    if household.player && w.mode == GameMode::Creative {
        return None;
    }
    let plot = household.plot?;
    let p = &w.plots[plot as usize];
    let home_value: i64 = w
        .objects
        .iter()
        .zip(&w.object_plot)
        .filter(|(_, on)| **on == Some(plot))
        .map(|(o, _)| o.value)
        .sum();
    let residents = w.sims.iter().filter(|s| s.here() && s.household as usize == h).count();
    if residents == 0 {
        return None;
    }
    Some((rent.amount(p.w * p.d), rent.bills(home_value, residents)))
}

/// Weekly rent and bills; households that can't cover them go into debt and worry about it
/// every day.
fn charge_rent(w: &mut World, day: u32) {
    let Some(rent) = w.content.rent.clone() else {
        return;
    };
    if !clock::at_minute(w.tick, rent.hour * 60.0) {
        return;
    }
    let tick = w.tick;
    let due = clock::weekday(day) == rent.weekday;
    for h in 0..w.households.len() {
        let Some(member) = w.sims.iter().position(|s| s.here() && s.household as usize == h) else {
            continue;
        };
        if due {
            // Pensions come in on the same day.
            let pensions: i64 = w.sims.iter().filter(|s| s.here() && s.household as usize == h).map(|s| s.pension).sum();
            w.households[h].funds += pensions;
        }
        if due && let Some((rent, bills)) = weekly_costs(w, h) {
            let amount = rent + bills;
            let household = &mut w.households[h];
            let kind = if household.funds >= amount {
                EventKind::PaidRent
            } else {
                EventKind::RentDebt
            };
            household.funds -= amount;
            // Everyone's debts are news; paying on time only for the player's household.
            if household.player || kind == EventKind::RentDebt {
                w.events.push_detail(tick, kind, member, Some(amount), None);
            }
        }
        if w.households[h].funds < 0
            && let Some(m) = rent.debt_feeling
        {
            let World { sims, content, .. } = &mut *w;
            for s in sims.iter_mut().filter(|s| s.here() && s.household as usize == h) {
                social::add_feeling(&mut s.feelings, m, &content.feelings, tick);
            }
        }
    }
}

/// Guests go home when the visit is over, it's late, they need something, or there's nothing
/// for them to do (saying goodbye to one of the hosts first, if one is free).
fn end_visit_if_due(w: &mut World, i: usize, hour: f32) {
    let tick = w.tick;
    let Some(visit) = w.sims[i].visiting else { return };
    give_up_farewell(w, i);
    // A guest left with nothing to do while the hosts are all off at work goes home (hosts
    // just out visiting come back; parks have no hosts).
    let hosts: Vec<&crate::world::Sim> = w
        .sims
        .iter()
        .filter(|s| s.here() && w.households[s.household as usize].plot == Some(visit.plot))
        .collect();
    let hosts_out = w.sims[i].current.is_none()
        && !w.plots[visit.plot as usize].public
        && !hosts.is_empty()
        && hosts.iter().all(|s| s.away_until.is_some());
    // Left standing around with nothing to do (the hosts busy, nothing they may use) for long
    // enough, a guest with free will says it's time they went (without, they're the player's
    // to send home).
    let free_will = w.autonomy && w.households[w.sims[i].household as usize].free_will;
    let content = &w.content;
    let sim = &mut w.sims[i];
    let idle = if sim.current.is_none() && sim.queue.is_empty() && sim.follow_up.is_none() && sim.engaged_with.is_none() {
        visit.idle.saturating_add(sim.steps)
    } else {
        0
    };
    if let Some(v) = sim.visiting.as_mut() {
        v.idle = idle;
    }
    let bored = free_will && content.visits.bored_minutes.is_some_and(|m| idle as f32 * MINUTES_PER_TICK >= m);
    let tired = content.needs.iter().zip(&sim.needs).any(|(d, &n)| !d.room && n < 0.15);
    let late = !(6.0..22.0).contains(&hour) || hosts_out;
    let busy = sim
        .current
        .as_ref()
        .is_some_and(|a| matches!(a.task.kind, TaskKind::GoHome | TaskKind::Work));
    if (tick >= visit.until || tired || late || bored)
        && !busy
        && sim.engaged_with.is_none()
        && !sim.queue.iter().any(|t| matches!(t.kind, TaskKind::GoHome))
    {
        if let Some((host, social)) = farewell(w, i, visit.plot) {
            w.sims[i].queue.push_back(Task { kind: TaskKind::Social { target: host as u32, social }, directed: false, routine: false });
        }
        w.sims[i].queue.push_back(Task {
            kind: TaskKind::GoHome,
            directed: false,
            routine: false,
        });
    }
}

/// Whether `task` is a guest's goodbye to a host (`visits.farewell`) rather than something the
/// player asked for.
fn is_farewell(content: &Content, task: &Task) -> bool {
    matches!(task.kind, TaskKind::Social { social, .. } if content.visits.farewell.contains(&social)) && !task.directed
}

/// Who guest `i` leaving `plot` says goodbye to, and how (`visits.farewell`): the nearest
/// member of the household living there who is around, awake and free to talk (or already
/// talking with the guest), close enough to get to in `farewellMinutes`. Nobody wakes up, comes
/// out of the bathroom or leaves something the player asked for to see a guest off.
pub(crate) fn farewell(w: &World, i: usize, plot: u32) -> Option<(usize, usize)> {
    let content = &w.content;
    let rules = &content.visits;
    let guest = &w.sims[i];
    if rules.farewell.is_empty() || guest.pressed(content) || w.plots[plot as usize].public {
        return None;
    }
    let household = w.households.iter().position(|h| h.plot == Some(plot))?;
    let reach = crate::world::WALK_SPEED * guest.mods.walk_speed / MINUTES_PER_TICK * rules.farewell_minutes;
    w.sims
        .iter()
        .enumerate()
        .filter(|&(j, s)| {
            j != i
                && s.here()
                && s.household as usize == household
                && s.away_until.is_none()
                && s.visiting.is_none()
                && !content.life.baby(s.age)
                && w.plot_at(s.tile().0, s.tile().1) == Some(plot)
                && (s.available_for_social(content) || s.engaged_with == Some(i as u32))
                && !s.queue.front().is_some_and(|t| t.directed)
        })
        .filter_map(|(j, s)| {
            let social = rules.farewell.iter().copied().find(|&f| conversation::social_allowed(content, &w.relationships, guest, s, &content.socials[f]))?;
            Some((j, social, (s.pos[0] - guest.pos[0]).hypot(s.pos[1] - guest.pos[1])))
        })
        .filter(|&(.., d)| d <= reach)
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(j, social, _)| (j, social))
}

/// A goodbye that hasn't started within `visits.farewellMinutes` (the host went off, or got
/// busy) is skipped: the guest just leaves.
fn give_up_farewell(w: &mut World, i: usize) {
    let World { sims, content, objects, .. } = &mut *w;
    let sim = &mut sims[i];
    let starting = sim.current.as_ref().is_some_and(|a| is_farewell(content, &a.task) && !matches!(a.phase, Phase::Conversing { .. }));
    let Some(visit) = sim.visiting.as_mut() else { return };
    if !starting {
        visit.farewell = 0;
        return;
    }
    visit.farewell = visit.farewell.saturating_add(sim.steps);
    if visit.farewell as f32 * MINUTES_PER_TICK >= content.visits.farewell_minutes {
        visit.farewell = 0;
        end_activity(sim, content, objects);
    }
}

#[cfg(test)]
mod tests {
    use crate::social::EventKind;
    use crate::{Command, TICKS_PER_SECOND, World, clock};

    const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02},{"id":"social","label":"Social","decayPerHour":0.02}],
        "objects":[{"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"effects":{"social":0.01},"tags":["lounge"]}]}],
        "feelings":[{"id":"promoted","label":"Promoted","mood":0.2,"hours":24}],
        "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},"needs":{"social":0.3}}],
        "bondPresets":{"friends":{"friendship":60}},
        "careers":[{"id":"office","label":"Office","workEffects":{"hunger":-0.2},"levels":[
            {"title":"Intern","pay":100,"start":9,"hours":4,"days":[0,1,2,3,4,5,6]},
            {"title":"Clerk","pay":200,"start":9,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
        "careerRules":{"promotionFeeling":"promoted","performancePerShift":60},
        "visits":{"needs":{"social":0.5},"hours":2,"minFriendship":10,"earliestHour":0,"latestHour":24},
        "economy":{"startingFunds":500}}"#;

    /// Two houses on one street with an exit at the west end.
    const TOWN: &str = r#"{"width":40,"depth":14,"exits":[[0.5,7.5]],
        "plots":[{"name":"West","x":0,"z":0,"w":20,"d":6,"entry":[5.5,5.5]},{"name":"East","x":20,"z":0,"w":20,"d":6,"entry":[25.5,5.5]}],
        "households":[{"name":"Player","plot":0,"player":true},{"name":"Npc","plot":1}],
        "objects":[{"def":"chair","x":3,"z":2,"rot":0},{"def":"chair","x":24,"z":2,"rot":0}],
        "sims":[{"name":"Worker","household":0,"job":{"career":"office"},"x":5.5,"z":3.5},
                {"name":"Guest","household":1,"x":25.5,"z":3.5}],
        "relationships":[{"a":0,"b":1,"preset":"friends"}]}"#;

    fn ticks(w: &mut World, n: u64) {
        for _ in 0..n {
            w.tick_once();
        }
    }

    fn until_hour(w: &mut World, day: u32, hour: f32) {
        let target = clock::tick_at(day, hour * 60.0).unwrap();
        while w.tick < target {
            w.tick_once();
        }
    }

    #[test]
    fn workers_leave_get_paid_and_come_home() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        assert_eq!(w.households[0].funds, 500);
        until_hour(&mut w, 2, 10.0);
        assert!(w.sims[0].away_until.is_some(), "should be at work at 10:00");
        until_hour(&mut w, 2, 14.5);
        assert!(w.sims[0].away_until.is_none());
        assert_eq!(w.households[0].funds, 700, "paid for day 1 and day 2");
        until_hour(&mut w, 2, 16.0);
        let home = w.plot_at(w.sims[0].tile().0, w.sims[0].tile().1);
        assert_eq!(home, Some(0), "walked back home");
        // 60 performance per shift at neutral mood -> promoted after two shifts.
        assert_eq!(w.sims[0].job.as_ref().unwrap().level, 1);
        assert!(w.events.iter().any(|e| e.kind == EventKind::Promoted));
    }

    #[test]
    fn being_kept_busy_means_a_missed_shift() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        w.autonomy = false;
        // Make the exit unreachable by walling the worker in.
        for x in 0..40 {
            w.lot.set_h(x, 6, crate::lot::Edge::Wall);
        }
        until_hour(&mut w, 2, 11.0);
        assert!(w.events.iter().any(|e| e.kind == EventKind::MissedWork));
        assert!(w.sims[0].away_until.is_none());
    }

    #[test]
    fn friends_visit_and_go_home() {
        let mut w = World::from_json(CONTENT, TOWN, 3).unwrap();
        w.sims[0].job = None;
        w.sims[1].needs[1] = 0.1; // lonely guest
        ticks(&mut w, TICKS_PER_SECOND as u64 * 120);
        assert!(
            w.events
                .iter()
                .any(|e| e.kind == EventKind::Visited && e.a == 1),
            "guest never visited"
        );
        // The visit lasts two hours; afterwards they head home.
        ticks(&mut w, (3.0 * 60.0 / crate::MINUTES_PER_TICK) as u64);
        let (x, z) = w.sims[1].tile();
        assert_eq!(w.plot_at(x, z), Some(1), "guest went home");
        assert!(w.sims[1].visiting.is_none());
    }

    #[test]
    fn guests_stay_at_the_hosts_place() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        w.sims[0].job = None;
        w.sims[0].needs[0] = 0.2; // hungry: the home chair... and nothing to eat anywhere
        w.apply(Command::Visit { sim: 0, plot: 1 }).unwrap();
        ticks(&mut w, TICKS_PER_SECOND as u64 * 30);
        for _ in 0..60 {
            ticks(&mut w, TICKS_PER_SECOND as u64);
            let (x, z) = w.sims[0].tile();
            assert_eq!(
                w.plot_at(x, z),
                Some(1),
                "guest wandered off the host's lot"
            );
        }
    }

    #[test]
    fn player_can_send_a_sim_to_visit_and_back() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        w.sims[0].job = None;
        w.apply(Command::Visit { sim: 0, plot: 1 }).unwrap();
        ticks(&mut w, TICKS_PER_SECOND as u64 * 40);
        assert_eq!(w.sims[0].visiting.map(|v| v.plot), Some(1));
        w.apply(Command::GoHome { sim: 0 }).unwrap();
        ticks(&mut w, TICKS_PER_SECOND as u64 * 40);
        assert!(w.sims[0].visiting.is_none());
    }

    /// The test content with a job market: hired the day they look, let go after two misses.
    fn with_market() -> String {
        CONTENT.replace(
            r#""careerRules":{"promotionFeeling":"promoted","performancePerShift":60}"#,
            r#""careerRules":{"promotionFeeling":"promoted","performancePerShift":60,
                "market":{"hour":9.5,"dailyChance":1.0,"fireAfterMissed":2,"cooldownDays":1,
                          "hiredFeeling":"promoted"}}"#,
        )
    }

    #[test]
    fn the_unemployed_find_work_on_their_own() {
        let mut w = World::from_json(&with_market(), TOWN, 2).unwrap();
        assert!(w.sims[1].job.is_none());
        until_hour(&mut w, 1, 10.0);
        let job = w.sims[1].job.as_ref().expect("found a job at 9:30");
        assert_eq!(job.career, 0);
        assert!(
            w.events
                .iter()
                .any(|e| e.kind == EventKind::JobFound && e.a == 1 && e.career == Some(0))
        );
        // The first shift is tomorrow's.
        until_hour(&mut w, 2, 10.0);
        assert!(w.sims[1].away_until.is_some(), "at work on day 2");
    }

    #[test]
    fn no_free_will_no_job_search() {
        let mut w = World::from_json(&with_market(), TOWN, 2).unwrap();
        w.apply(Command::SetAutonomy {
            enabled: false,
            household: Some(1),
        })
        .unwrap();
        until_hour(&mut w, 3, 12.0);
        assert!(w.sims[1].job.is_none());
    }

    #[test]
    fn missing_shifts_gets_you_let_go() {
        let mut w = World::from_json(&with_market(), TOWN, 1).unwrap();
        // No free will: they don't look for a new job afterwards either.
        w.apply(Command::SetAutonomy {
            enabled: false,
            household: Some(0),
        })
        .unwrap();
        // Wall the worker in so they can never reach the exit.
        for x in 0..40 {
            w.lot.set_h(x, 6, crate::lot::Edge::Wall);
        }
        until_hour(&mut w, 3, 12.0);
        assert!(w.sims[0].job.is_none(), "let go after two missed shifts");
        assert!(w.events.iter().any(|e| e.kind == EventKind::Fired && e.a == 0));
        // Missed on days 1 and 2; looks again from day 3.
        assert_eq!(w.sims[0].job_search_from, 3);
    }

    #[test]
    fn a_visit_the_player_asked_for_ends() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        w.sims[0].job = None;
        w.apply(Command::Visit { sim: 0, plot: 1 }).unwrap();
        ticks(&mut w, TICKS_PER_SECOND as u64 * 40);
        assert_eq!(w.sims[0].visiting.map(|v| v.plot), Some(1));
        // Twice as long as a normal (2 h) visit, then home.
        ticks(&mut w, (5.0 * 60.0 / crate::MINUTES_PER_TICK) as u64);
        assert!(w.sims[0].visiting.is_none());
    }

    #[test]
    fn careers_can_be_joined_and_quit() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        w.apply(Command::QuitCareer { sim: 0 }).unwrap();
        assert!(w.sims[0].job.is_none());
        w.apply(Command::JoinCareer {
            sim: 1,
            career: 0,
            level: 0,
        })
        .unwrap();
        assert_eq!(w.sims[1].job.as_ref().unwrap().level, 0);
        assert!(
            w.apply(Command::JoinCareer {
                sim: 1,
                career: 9,
                level: 0
            })
            .is_err()
        );
    }
}
