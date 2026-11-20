//! The small judgements a choice leans on: how much a relationship invites a social,
//! what cleaning skips, where to sweep, and a goal at a point.

use crate::content::TagMask;
use crate::social::{self, Prefer};
use crate::world::{Ctx, Sim, TaskKind};

/// How much a relationship invites a social of the given kind.
pub(crate) fn prefer_factor(
    prefer: Prefer,
    rel: &social::Relationship,
    chemistry: f32,
    spark: f32,
) -> f32 {
    match prefer {
        Prefer::Any => 1.0,
        Prefer::Liked => (0.5 + rel.friendship / 100.0).clamp(0.1, 1.5),
        Prefer::Disliked => (0.4 - rel.friendship / 100.0).clamp(0.0, 1.4),
        Prefer::Romance => (rel.romance / 50.0 + chemistry * 0.4 + spark).clamp(0.0, 1.6),
        Prefer::Unloved => {
            if rel.partners && rel.romance < 25.0 {
                (25.0 - rel.romance) / 25.0
            } else {
                0.0
            }
        }
    }
}

/// Where `sim` won't clean (tidying up or with a tool): where they couldn't get to a moment ago,
/// and where someone else is cleaning (the briefs' `cleaning`).
pub(crate) fn cleaning_skip(sim: &Sim, ctx: &Ctx) -> Vec<(i32, i32)> {
    let mut skip: Vec<(i32, i32)> = sim
        .unreachable_now(ctx.tick)
        .filter_map(|k| {
            if let TaskKind::Clean { x, z } = k {
                Some((x, z))
            } else {
                None
            }
        })
        .collect();
    skip.extend(
        ctx.briefs
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != sim.id as usize)
            .filter_map(|(_, b)| b.cleaning),
    );
    skip
}

/// Where `sim` would take a tool that cleans as `sweep` (`sweep::spot`): the dirtiest floor at
/// their home it cleans, dirtier than `threshold`, but where they won't clean (`cleaning_skip`).
pub(crate) fn sweep_spot(
    sim: &Sim,
    ctx: &Ctx,
    sweep: &crate::content::Sweep,
    threshold: f32,
) -> Option<crate::sweep::Spot> {
    let plot = ctx.households[sim.household as usize].plot?;
    crate::sweep::spot(
        ctx.content,
        &ctx.nav,
        ctx.rooms,
        ctx.dirt,
        &ctx.plots[plot as usize],
        sweep,
        threshold,
        &cleaning_skip(sim, ctx),
    )
}

/// What dirt a floor needs for the player's order to clean it with a tool (free will waits for
/// `roomRules.clean.threshold`).
pub(crate) const SWEEP_ORDERED: f32 = 0.01;

/// A walk's goal at `at`: the point, its tile, and no tags (`start_task`).
pub(crate) fn goal_at(at: [f32; 2]) -> ([f32; 2], (i32, i32), TagMask) {
    (at, (at[0].floor() as i32, at[1].floor() as i32), 0)
}
