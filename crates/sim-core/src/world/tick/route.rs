//! Searches and routes: what a tick may search, and re-routing after a change.

use super::*;

/// The tiles a tick's searches may expand (`path::nodes_searched`) before those who may look for a
/// way wait for the next tick (open-world towns: a whole hamlet setting off at once). The
/// player's household and whoever is watched never wait, nor anyone who waited last tick.
pub const SEARCH_BUDGET: u64 = 150_000;

/// Whether `sim`'s turn may look for a way somewhere: they're choosing or starting something,
/// or walking a way a change since may lie across. (Someone at something, talking or waiting
/// won't, until it ends.)
pub(crate) fn may_search(sim: &Sim, ctx: &Ctx) -> bool {
    match sim.current.as_ref().map(|a| &a.phase) {
        None => true,
        Some(Phase::Routing { .. }) => sim.nav_seen != ctx.nav_version && route_touched(sim, ctx),
        Some(_) => false,
    }
}

/// Whether a change to walking since they last looked (`Sim::nav_seen`, `World::nav_log`) may
/// lie across the rest of a walker's way: a tile round any stretch of it still ahead (from where
/// they stand to the next waypoint, and on between the waypoints), so a long walk by road isn't
/// planned again for a change far off it, even inside the box round all of it. Not walking:
/// true (nothing to keep).
pub(crate) fn route_touched(sim: &Sim, ctx: &Ctx) -> bool {
    let Some(Activity {
        phase: Phase::Routing { waypoints, next },
        ..
    }) = sim.current.as_ref()
    else {
        return true;
    };
    let Some(changes) = ctx.nav_log.changes_since(sim.nav_seen, ctx.nav_version) else {
        return true;
    };
    let changes: Vec<[i32; 4]> = changes.collect();
    let ahead = waypoints.get(*next..).unwrap_or_default();
    std::iter::once(&sim.pos)
        .chain(ahead)
        .zip(ahead)
        .any(|(a, b)| {
            let r = [
                a[0].min(b[0]).floor() as i32 - 1,
                a[1].min(b[1]).floor() as i32 - 1,
                a[0].max(b[0]).floor() as i32 + 2,
                a[1].max(b[1]).floor() as i32 + 2,
            ];
            changes.iter().any(|&c| crate::path::overlaps(c, r))
        })
}

/// A wall or something they walk through moved while they were on their way: their waypoints
/// may cross it now. Route again from where they stand, and if there's no way there any more,
/// give up (remembering it, if they chose it themselves).
pub(crate) fn reroute_after_change(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let Some(act) = sim.current.as_ref() else {
        return;
    };
    let Phase::Routing { waypoints, .. } = &act.phase else {
        return;
    };
    let Some(&target) = waypoints.last() else {
        return;
    };
    let task = act.task;
    let goal = (target[0].floor() as i32, target[1].floor() as i32);
    if let Some(waypoints) = walk_route(ctx, sim, goal, target) {
        if let Some(act) = sim.current.as_mut() {
            act.phase = Phase::Routing { waypoints, next: 0 };
        }
    } else {
        let directed = task.directed;
        // (A floor a tool was taken to: that floor, not the tool, out of reach.)
        let kind = match sim.sweep {
            Some((x, z)) if matches!(task.kind, TaskKind::Use { .. }) => TaskKind::Clean { x, z },
            _ => task.kind,
        };
        end_activity(sim, ctx.content, objects);
        if !directed {
            let retry = if matches!(task.kind, TaskKind::Social { .. } | TaskKind::Join { .. }) {
                PERSON_UNREACHABLE_TICKS
            } else {
                social::minutes_to_ticks(ctx.content.autonomy.retry_minutes)
            };
            sim.remember_unreachable(kind, ctx.tick + retry);
        }
    }
}
