//! What the home can fill, what is about to run out, and how far it is.
//!
//! Split from `world/objects.rs` (see `mod.rs`).

use super::*;

/// Whether their home has something working that fills need `n` (a toilet, a bed, a fridge),
/// busy or not.
pub(crate) fn home_fills(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], n: usize) -> bool {
    home_needs(sim, ctx, objects, 1 << n, false) != 0
}

/// Which of the needs in `mask` (bit per need) their home has something working for (not
/// counting what's only done with a baby there, nor a recipe the home lacks a station for: a
/// stove is no food without a fridge). `free`: only what they could go and use now, with a
/// place on it free (or theirs already), and not something they couldn't get to a moment ago
/// (every bed taken is no bed for them tonight).
pub(crate) fn home_needs(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    mask: u32,
    free: bool,
) -> u32 {
    let Some(home) = ctx.households[sim.household as usize].plot else {
        return 0;
    };
    let usable = |o: &ObjectInstance| {
        !free
            || ((o.has_free_slot(ctx.content.objects[o.def].slots) || o.slot_of(sim.id).is_some())
                && !sim.couldnt_reach(o.id, ctx.tick))
    };
    // (Busy or not: a cook waits for a station.)
    let stations = crate::cooking::home_stations(ctx, sim.household, objects);
    let possible = |r: usize| ctx.content.recipes[r].possible(stations);
    let mut found = 0;
    for o in ctx
        .index
        .on(home)
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| !o.broken() && usable(o))
    {
        let def = &ctx.content.objects[o.def];
        let startable = |&(n, _): &(usize, &crate::content::Interaction)| {
            !free || o.can_use_for(sim.id, def, n)
        };
        for (_, i) in def
            .interactions
            .iter()
            .enumerate()
            .filter(startable)
            .filter(|(_, i)| {
                i.autonomous
                    && !i.cares()
                    && i.recipe.is_none_or(possible)
                    && unfit(ctx.content, sim.age, i).is_none()
                    && requirements_met(ctx, sim, objects, o.id, i)
            })
        {
            found |= i.fills() & mask;
        }
        if found == mask {
            break;
        }
    }
    found
}

/// Essentials about to run out (`AutonomyRules::urgent`) that something at home they could use
/// now fills (bit per need): residents see to them before anything else.
pub(crate) fn urgent_needs(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> u32 {
    let content = ctx.content;
    let rules = &content.autonomy;
    let low = (0..content.needs.len())
        .filter(|&n| {
            !content.needs[n].room
                && rules.urgent & (1 << n) != 0
                && sim.needs[n] < rules.urgent_below
        })
        .fold(0, |m, n| m | 1 << n);
    if low == 0 {
        0
    } else {
        home_needs(sim, ctx, objects, low, true)
    }
}

/// Whether something they chose themselves, `inter`, with `minutes` to go, stops for one of the
/// `pressing` needs: one it doesn't fill (`fills`, bit per need) that would run out before then.
pub(crate) fn breaks_off(
    sim: &Sim,
    content: &Content,
    pressing: u32,
    fills: u32,
    minutes: f32,
    inter: &crate::content::Interaction,
    low: f32,
) -> bool {
    pressing != 0
        && pressing & fills == 0
        && runs_out(
            sim,
            content,
            pressing,
            minutes,
            false,
            &inter.gain_per_minute,
            &[0.0; MAX_NEEDS],
            low * 0.5,
        )
}

/// Whether one of the needs in `mask` would run down below `floor` in `minutes` of something
/// (asleep or not) that costs `costs` a minute (negative gains), and the walk `after` it to
/// what fills each.
#[allow(clippy::too_many_arguments)]
pub(crate) fn runs_out(
    sim: &Sim,
    content: &Content,
    mask: u32,
    minutes: f32,
    asleep: bool,
    costs: &[f32; MAX_NEEDS],
    after: &[f32; MAX_NEEDS],
    floor: f32,
) -> bool {
    let rhythm = &content.day_rhythm;
    (0..content.needs.len())
        .filter(|&n| mask & (1 << n) != 0)
        .any(|n| {
            let factor = if asleep { rhythm.asleep_decay[n] } else { 1.0 };
            let per_minute = content.needs[n].decay_per_minute * sim.mods.need_decay[n];
            let used = (per_minute * factor - costs[n].min(0.0)) * minutes + per_minute * after[n];
            sim.needs[n] - used < floor
        })
}

/// How many minutes' walk the nearest thing for each need is (one they may use, not just for
/// babies): where they'd go for it after whatever they're about to do.
pub(crate) fn reach(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    usable: &[u32],
    per_minute: f32,
) -> [f32; MAX_NEEDS] {
    let content = ctx.content;
    let mut nearest = [f32::INFINITY; MAX_NEEDS];
    for o in usable
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| !o.broken())
    {
        if access(ctx, sim, ctx.object_plot[o.id as usize]) == Access::None {
            continue;
        }
        let fills = content.objects[o.def]
            .interactions
            .iter()
            .filter(|i| {
                i.autonomous
                    && !i.baby
                    && !i.cares()
                    && requirements_met(ctx, sim, objects, o.id, i)
                    && unfit(content, sim.age, i).is_none()
            })
            .fold(0, |m, i| m | i.fills());
        if fills == 0 {
            continue;
        }
        let (fx, fz) = ctx
            .index
            .links
            .approach_of(o.id)
            .unwrap_or_else(|| o.front_tile(content));
        let minutes = ctx
            .nav
            .stairs
            .distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos)
            / per_minute;
        for n in (0..content.needs.len()).filter(|&n| fills & (1 << n) != 0) {
            nearest[n] = nearest[n].min(minutes);
        }
    }
    nearest.map(|m| if m.is_finite() { m } else { 0.0 })
}
