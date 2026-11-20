//! What a finished use changes, and what is done just before leaving for work.
//!
//! Split from `world/objects.rs` (see `mod.rs`).

use super::*;

/// A use done that changed the object as the hands went (`Interaction::gradual`, a crop's
/// `picks`): now it is so. The washing is out on the line (or in); the crop is picked, and lies
/// fallow, then grows again; a machine loaded runs, one emptied is empty (`machines::use_done`).
pub(crate) fn leaves_at_end(
    sim: &mut Sim,
    content: &Content,
    o: &mut ObjectInstance,
    interaction: usize,
) {
    let Some(inter) = content.objects[o.def].interactions.get(interaction) else {
        return;
    };
    if let Some(open) = inter.leaves.filter(|_| inter.gradual)
        && o.open != open
    {
        o.open = open;
        sim.reshaped = true;
    }
    if inter.picks && o.growth >= 0.0 {
        o.growth = -1.0;
        sim.reshaped = true;
    }
    crate::machines::use_done(sim, content, o, inter);
}

/// Just before they leave for work or school (`DayRhythm::ready_minutes`): what they do out of
/// habit first (`HabitTime::Leave`), dropping what they chose themselves for it (not what
/// they were told to do, a routine, cooking or eating, or something for that habit anyway).
pub(crate) fn ready_to_leave(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> Option<Task> {
    let content = ctx.content;
    let rhythm = &content.day_rhythm;
    let low = rhythm
        .habits
        .iter()
        .filter(|h| h.at == HabitTime::Leave && sim.needs[h.need] < h.below)
        .fold(0u32, |m, h| m | 1 << h.need);
    let free_will = ctx.autonomy && ctx.households[sim.household as usize].free_will;
    if low == 0
        || !free_will
        || sim.follow_up.is_some()
        || sim.queue.front().is_some_and(|t| t.directed)
        || sim.asleep(content)
    {
        return None;
    }
    let leave = leave_today(sim, ctx)?;
    if leave > ctx.tick + social::minutes_to_ticks(rhythm.ready_minutes) {
        return None;
    }
    match sim.current.as_ref().map(|a| a.task) {
        None => {}
        Some(Task {
            kind: TaskKind::Use {
                object,
                interaction,
            },
            directed: false,
            routine: false,
        }) => {
            let inter = &content.objects[objects[object as usize].def].interactions[interaction];
            if inter.fills() & low != 0 {
                return None;
            }
        }
        Some(_) => return None,
    }
    habit(
        sim,
        ctx,
        objects,
        HabitTime::Leave,
        ctx.briefs[sim.id as usize].plot,
    )
}
