//! What a tick reads of a Sim: their queue, the hour, their interests and training.
//!
//! Split from `world/tick.rs` (see `mod.rs`).

use super::*;

/// Keeps a queue within `MAX_QUEUE` by dropping what was chosen for them from the back, but
/// never the first `keep` tasks (just put there), the player's orders, nor a recipe whose
/// ingredients are paid for: for those, it goes over the cap.
pub(crate) fn trim_queue(queue: &mut VecDeque<Task>, keep: usize) {
    while queue.len() > MAX_QUEUE {
        let paid = |t: &Task| matches!(t.kind, TaskKind::Cook { servings, .. } if servings > 0);
        let Some(k) = queue
            .iter()
            .rposition(|t| !t.directed && !paid(t))
            .filter(|&k| k >= keep)
        else {
            break;
        };
        queue.remove(k);
    }
}

/// Whether a household with `funds` can pay `cost` (free things are always affordable).
pub fn affordable(cost: i64, funds: i64) -> bool {
    cost <= 0 || funds >= cost
}

/// Hour a Sim gets up today: the usual time, or earlier for an early shift.
pub(crate) fn wake_hour(sim: &Sim, ctx: &Ctx) -> f32 {
    let rhythm = &ctx.content.day_rhythm;
    match leave_today(sim, ctx) {
        Some(leave) => (crate::clock::hour(leave) - rhythm.wake_before_work_minutes / 60.0)
            .clamp(0.0, rhythm.wake_hour),
        None => rhythm.wake_hour,
    }
}

/// Whether it is night for this Sim (time to be in bed): their planned sleep, else the usual
/// day rhythm.
pub(crate) fn is_night(sim: &Sim, ctx: &Ctx) -> bool {
    let routines = &ctx.households[sim.household as usize].routines;
    if let Some(night) =
        crate::planner::planned_night(ctx.content, sim, routines, ctx.day, ctx.minute)
    {
        return night;
    }
    let rhythm = &ctx.content.day_rhythm;
    if ctx.hour >= rhythm.bed_hour {
        return true;
    }
    if ctx.hour >= rhythm.wake_hour {
        return false;
    }
    rhythm.is_night(ctx.hour, wake_hour(sim, ctx))
}

/// What an object's quality adds to an interaction: a bed's stars make a night only a little
/// shorter (`DayRhythm::sleep_quality_bonus`), anything else's do more.
pub(crate) fn quality_boost(
    content: &Content,
    inter: &crate::content::Interaction,
    quality: u8,
) -> f32 {
    let rhythm = &content.day_rhythm;
    if inter.tags & rhythm.sleep_tags != 0 {
        rhythm.sleep_quality_factor(quality)
    } else {
        content.object_rules.quality_factor(quality)
    }
}

/// Worn out: what a night's sleep fills is nearly empty (bed, not a nap, whatever the hour).
pub(crate) fn exhausted(sim: &Sim, content: &Content) -> bool {
    let rhythm = &content.day_rhythm;
    rhythm
        .rest_need
        .is_some_and(|n| sim.needs[n] < rhythm.exhausted_below)
}

/// Time to get up for work or school (`DayRhythm::wake_before_work_minutes` before leaving).
pub(crate) fn getting_up(sim: &Sim, ctx: &Ctx) -> bool {
    let early = social::minutes_to_ticks(ctx.content.day_rhythm.wake_before_work_minutes);
    leave_today(sim, ctx).is_some_and(|leave| leave <= ctx.tick + early)
}

/// The middle of the night: a rested sleeper stays in bed until a little before they'd get up.
pub(crate) fn before_dawn(sim: &Sim, ctx: &Ctx) -> bool {
    is_night(sim, ctx) && {
        let wake = wake_hour(sim, ctx);
        !(wake - ctx.content.day_rhythm.lie_in_hours..wake).contains(&ctx.hour)
    }
}

/// Target skill levels (what probation and the next promotion need, and skill goals) and how
/// keen they are on each (the job's fully, a goal's by its place on the list).
pub(crate) fn job_goals(sim: &Sim, content: &Content) -> ([f32; MAX_SKILLS], [f32; MAX_SKILLS]) {
    let mut goals = [0.0f32; MAX_SKILLS];
    let mut keen = [0.0f32; MAX_SKILLS];
    if let Some(job) = &sim.job {
        let levels = &content.careers[job.career].levels;
        for level in levels[job.level..].iter().take(2) {
            for &(s, l) in &level.requires {
                goals[s] = goals[s].max(l);
                keen[s] = 1.0;
            }
        }
    }
    crate::planner::skill_targets(content, sim, &mut goals, &mut keen);
    (goals, keen)
}

/// How much an idle Sim wants to practise with `inter` (0 when needs come first).
pub(crate) fn training_interest(
    sim: &Sim,
    content: &Content,
    inter: &crate::content::Interaction,
) -> f32 {
    let rules = &content.skill_rules;
    let lowest = sim.needs[..content.needs.len()]
        .iter()
        .copied()
        .fold(1.0f32, f32::min);
    if lowest < 0.35 {
        return 0.0;
    }
    let (goals, keen) = job_goals(sim, content);
    let mut best = 0.0f32;
    for (s, &gain) in inter.skill_gain.iter().enumerate() {
        if gain <= 0.0 || sim.skills[s] >= rules.max_level {
            continue;
        }
        let mut want = rules.interest * sim.mods.skill_gain[s];
        if goals[s] > sim.skills[s] {
            want += rules.goal_interest * keen[s];
        }
        best = best.max(want);
    }
    best * lowest / (1.0 + sim.practice_fatigue)
}

/// Adds `minutes` of practice to a skill (a tick's, or more when catching up on a turn, see
/// `lod.rs`) and notes a new level.
pub(crate) fn practise(
    sim: &mut Sim,
    content: &Content,
    skill: usize,
    per_hour: f32,
    minutes: f32,
) {
    let rules = &content.skill_rules;
    let before = sim.skills[skill];
    let gain = rules.gain(
        per_hour * sim.mods.skill_gain[skill],
        minutes / 60.0,
        before,
    );
    let after = (before + gain).min(rules.max_level);
    sim.skills[skill] = after;
    if after.floor() > before.floor() {
        sim.skill_ups |= 1 << skill;
        sim.planner.recheck = true;
    }
}
