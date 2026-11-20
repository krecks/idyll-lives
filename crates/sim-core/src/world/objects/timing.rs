//! When work or school takes them away today.

use crate::MINUTES_PER_TICK;
use crate::content::Content;
use crate::world::{Ctx, Sim};

/// Minutes before `sim` must leave for work or school today, for a visit or an outing to fit
/// (`leaves_for_work`). Infinite with none today.
pub(crate) fn free_before_work(sim: &Sim, ctx: &Ctx) -> f32 {
    leaves_for_work(sim, ctx).map_or(f32::INFINITY, |leave| {
        leave.saturating_sub(ctx.tick) as f32 * MINUTES_PER_TICK
    })
}

/// When they leave for work or school today (if they haven't yet).
pub(crate) fn leave_today(sim: &Sim, ctx: &Ctx) -> Option<u64> {
    leave_at(sim, ctx.content, ctx.tick)
}

/// When `sim` (the one being updated, or one of `ctx.others`) leaves home for work or school
/// today at the latest, as things stand: the one answer for a visit's guest (`free_before_work`)
/// and its hosts (`hosts_about_till`), as `life`'s leaving has it. On the work taxi they're booked
/// on, at its time (`taxi::booked_leave`, as late as it runs), sooner from further in than the
/// run was timed with (`taxi::indoor_lead`). Else as their way is timed on foot
/// (`life::leave_for`), or by the drive's time if that's sooner (`cars::lead_ticks`: a driver
/// sets off by it, later than on foot as a rule, so never before this). Or now, that time gone by
/// with work still to start (off any moment: a host past her walk's time drove off to her night
/// shift an hour on). None: nothing still to go to today (none, done, or a day stayed home).
pub(crate) fn leaves_for_work(sim: &Sim, ctx: &Ctx) -> Option<u64> {
    let tick = ctx.tick;
    if let Some(leave) = sim
        .taxi
        .and_then(|b| crate::taxi::booked_leave(ctx.taxis, ctx.cars, tick, b))
    {
        let indoors =
            crate::taxi::indoor_lead(ctx.content, ctx.nav.stairs, ctx.households, ctx.plots, sim);
        return Some(leave.saturating_sub(indoors).max(tick));
    }
    let day = crate::clock::day(tick);
    let (leave, start, _) = match &sim.job {
        Some(job) if job.last_shift_day != day => {
            crate::life::shift_today(ctx.content, job, &sim.skills, tick)?
        }
        Some(_) => return None,
        None if sim.school_day != day => crate::life::school_today(ctx.content, sim, tick)?,
        None => return None,
    };
    if start <= tick {
        return None;
    }
    let walk = crate::life::leave_for(sim, leave, start);
    let leave = match crate::cars::lead_ticks_in(ctx, sim, start) {
        0 => walk,
        lead => walk.min(start.saturating_sub(lead)),
    };
    Some(leave.max(tick))
}

/// When they leave for work or school later today (`leave_today`).
pub(crate) fn leave_at(sim: &Sim, content: &Content, tick: u64) -> Option<u64> {
    sim.job
        .as_ref()
        .and_then(|job| crate::life::shift_today(content, job, &sim.skills, tick))
        .or_else(|| crate::life::school_today(content, sim, tick))
        .map(|(leave, start, _)| crate::life::leave_for(sim, leave, start))
        .filter(|&leave| leave > tick)
}

/// Whether they leave for work or school later today and by tick `by` (`leave_at`): for a job,
/// the time asked before whether today is a workday (`life::work_days`, the dearer question, and
/// most of the time the answer is no anyway: asked of everyone every tick, `SimBrief::free`).
pub(crate) fn leaves_by(sim: &Sim, content: &Content, tick: u64, by: u64) -> bool {
    let leaves = match &sim.job {
        Some(job) => {
            // (`life::shift_today`, its parts the other way round.)
            let level = &content.careers[job.career].levels[job.level];
            let day = crate::clock::day(tick);
            crate::clock::tick_at(day, level.start_hour * 60.0).is_some_and(|start| {
                let leave =
                    start.saturating_sub(crate::life::ticks(content.career_rules.commute_minutes));
                let leave = crate::life::leave_for(sim, leave, start);
                leave > tick
                    && leave <= by
                    && crate::life::work_days(content, job, &sim.skills)
                        & (1 << crate::clock::weekday(day))
                        != 0
            })
        }
        None => crate::life::school_today(content, sim, tick)
            .map(|(leave, start, _)| crate::life::leave_for(sim, leave, start))
            .is_some_and(|leave| leave > tick && leave <= by),
    };
    debug_assert_eq!(
        leaves,
        leave_at(sim, content, tick).is_some_and(|t| t <= by)
    );
    leaves
}
