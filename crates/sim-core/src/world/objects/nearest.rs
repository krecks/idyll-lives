//! Finding the nearest thing that fits a use.

use crate::world::{
    Access, Ctx, ObjectInstance, Sim, Task, TaskKind, access, requirements_met, route, unfit,
    upper_barred, usable_here,
};

/// The nearest working object on `plot` (but `except`) with an interaction that `fits`, one
/// they may use and can get to, as a routine task.
pub(crate) fn nearest(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    plot: Option<u32>,
    except: Option<u32>,
    fits: impl Fn(&crate::content::Interaction) -> bool,
) -> Option<Task> {
    let content = ctx.content;
    if access(ctx, sim, plot) == Access::None {
        return None;
    }
    let usable = usable_here(sim, ctx);
    let usable: &[u32] = &usable;
    let approach = |o: &ObjectInstance| {
        ctx.index
            .links
            .approach_of(o.id)
            .unwrap_or_else(|| o.front_tile(content))
    };
    let mut near: Vec<(f32, u32, usize)> = usable
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| Some(o.id) != except && ctx.object_plot[o.id as usize] == plot)
        .filter(|o| !o.broken() && o.has_free_slot(content.objects[o.def].slots))
        .filter(|o| !(sim.visiting.is_some() && content.objects[o.def].private))
        .filter_map(|o| {
            let def = &content.objects[o.def];
            let i = (0..def.interactions.len()).find(|&i| {
                fits(&def.interactions[i])
                    && unfit(content, sim.age, &def.interactions[i]).is_none()
                    && !upper_barred(content, o, sim.id, sim.age)
                    && requirements_met(ctx, sim, objects, o.id, &def.interactions[i])
                    && o.allows(&def.interactions[i], sim.id)
                    && o.can_use_for(sim.id, def, i)
            })?;
            let (fx, fz) = approach(o);
            Some((
                ctx.nav
                    .stairs
                    .distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos),
                o.id,
                i,
            ))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (_, object, interaction) = near.into_iter().take(3).find(|&(_, o, _)| {
        let (fx, fz) = approach(&objects[o as usize]);
        route(
            &ctx.nav,
            Some(ctx.reach),
            sim.pos,
            sim.tile(),
            (fx, fz),
            [fx as f32 + 0.5, fz as f32 + 0.5],
            None,
        )
        .is_some()
    })?;
    Some(Task {
        kind: TaskKind::Use {
            object,
            interaction,
        },
        directed: false,
        routine: true,
    })
}
