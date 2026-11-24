//! Habits at the turning points of a day, and what follows a use.

use super::*;
use crate::content::{Content, HabitTime, TagMask};
use crate::world::{Ctx, ObjectInstance, Phase, Sim, Task, TaskKind, affordable};

/// What they do out of habit at a turning point of their day (`DayRhythm::habits`): for the
/// habit for `at` whose need is lowest (for it), the nearest quick thing on `plot` that fills
/// it. None: nothing to see to, or nothing there for it.
pub(crate) fn habit(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    at: HabitTime,
    plot: Option<u32>,
) -> Option<Task> {
    let content = ctx.content;
    let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
    // (What a use costs on `plot`: a home kitchen's snack is bought on a public lot, `cost_at`.)
    let public = plot.is_some_and(|p| ctx.plots[p as usize].public);
    // (The lowest first, for its habit.)
    let mut due: Vec<&crate::content::Habit> = content
        .day_rhythm
        .habits
        .iter()
        .filter(|h| h.at == at && sim.needs[h.need] < h.below)
        .collect();
    due.sort_by(|a, b| (sim.needs[a.need] / a.below).total_cmp(&(sim.needs[b.need] / b.below)));
    due.into_iter().find_map(|h| {
        nearest(sim, ctx, objects, plot, None, |i| {
            i.autonomous
                && i.total_gain[h.need] > 0.0
                && i.minutes <= content.autonomy.quick_minutes
                && !i.baby
                && !i.needs_table
                && i.recipe.is_none()
                && affordable(i.cost_at(public), funds)
        })
    })
}

/// Whether what they're doing right now fills need `n` (on the toilet, in bed, eating). A meal
/// on its way counts all along: while it cooks, while they fetch it and sit down.
pub(crate) fn filling(
    sim: &Sim,
    content: &Content,
    objects: &[ObjectInstance],
    items: &crate::items::Items,
    n: usize,
) -> bool {
    let Some(act) = sim.current.as_ref() else {
        return false;
    };
    let using = matches!(act.phase, Phase::Using { .. });
    match act.task.kind {
        TaskKind::Use {
            object,
            interaction,
        } if using => objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .is_some_and(|i| i.total_gain[n] > 0.0),
        TaskKind::Spot { accident } if using => content.accidents[accident]
            .rest
            .as_ref()
            .is_some_and(|r| r.gain_per_minute[n] > 0.0),
        TaskKind::Cook { recipe, .. } => content.recipes[recipe].effects[n] > 0.0,
        TaskKind::Eat { .. } => {
            crate::cooking::eating(sim, items).is_some_and(|r| content.recipes[r].effects[n] > 0.0)
        }
        _ => false,
    }
}

/// Closing `o` again after sleeping or napping in it, if that opened it (`Interaction::leaves`):
/// its interaction that closes it, if `sim` may do it now. A routine, like washing hands.
pub(crate) fn closing(
    sim: &Sim,
    content: &Content,
    o: &ObjectInstance,
    object: u32,
) -> Option<Task> {
    if !o.open {
        return None;
    }
    let interactions = &content.objects[o.def].interactions;
    let interaction = interactions
        .iter()
        .position(|i| i.leaves == Some(false) && o.allows(i, sim.id))?;
    Some(Task {
        kind: TaskKind::Use {
            object,
            interaction,
        },
        directed: false,
        routine: true,
    })
}

/// What follows using `object` when its interaction has an `after` (`tags`): the nearest
/// working object on the same plot with an interaction of those tags, one they may use and can
/// get to (a guest washes their hands at the host's sink). None: there's nothing like it.
pub(crate) fn follow_up(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    object: u32,
    tags: TagMask,
) -> Option<Task> {
    let plot = ctx.object_plot.get(object as usize).copied().flatten();
    nearest(sim, ctx, objects, plot, Some(object), |i| {
        i.tags & tags != 0 && !i.baby && i.recipe.is_none()
    })
}
