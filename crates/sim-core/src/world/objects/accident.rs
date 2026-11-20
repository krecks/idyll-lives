//! An accident happening where they are.
//!
//! Split from `world/objects.rs` (see `mod.rs`).

use super::*;

/// Accident `a` happens to `sim` where they are (see `Content::accidents`).
pub(crate) fn accident(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], a: usize) {
    let content = ctx.content;
    let def = &content.accidents[a];
    sim.accident_at[a] = ctx.tick + def.cooldown_ticks;
    for (n, e) in def.effects.iter().enumerate().take(content.needs.len()) {
        sim.needs[n] = (sim.needs[n] + e).clamp(0.0, 1.0);
    }
    if def.dirt > 0.0 {
        let (x, z) = sim.tile();
        sim.mess = Some((x, z, def.dirt));
    }
    if let Some(f) = def.feeling {
        social::add_feeling(&mut sim.feelings, f, &content.feelings, ctx.tick);
    }
    sim.pending_spend += def.cost;
    if def.story {
        sim.news = Some(News::Accident(a));
        crate::planner::think(sim, crate::planner::ThoughtKind::Accident, a, ctx.tick);
    }
    if let Some(rest) = &def.rest {
        // (Asleep on the way to a car: they don't go on to it, and it doesn't wait for them.)
        if sim
            .current()
            .is_some_and(|a| matches!(a.phase, Phase::Routing { .. }))
        {
            sim.bound_for_car = None;
        }
        end_activity(sim, content, objects);
        sim.queue.retain(|t| t.directed);
        sim.pose = rest.pose;
        sim.current = Some(Activity {
            task: Task {
                kind: TaskKind::Spot { accident: a },
                directed: false,
                routine: false,
            },
            phase: Phase::Using { elapsed: 0.0 },
            tags: rest.tags,
        });
    }
}
