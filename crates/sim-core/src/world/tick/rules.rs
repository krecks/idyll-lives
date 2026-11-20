//! The rules of use: who may use what, what is missing, and when things are closed.
//!
//! Split from `world/tick.rs` (see `mod.rs`).

use super::*;

/// What a Sim may use on a plot: everything at home or in public places; as a guest,
/// anything that isn't private (bed, bathroom...); nothing on other people's lots.
#[derive(PartialEq)]
pub(crate) enum Access {
    Full,
    Guest,
    None,
}

pub(crate) fn access(ctx: &Ctx, sim: &Sim, plot: Option<u32>) -> Access {
    access_in(ctx.households, ctx.plots, sim, plot)
}

/// Whether a guest may use `inter` of `def` at the host's: nothing private (sleeping, washing;
/// the toilet is fine), nothing that's the household's own (`ObjectDef::private`: a wardrobe,
/// the bath cabinet), no cooking and no caring for the host's baby.
pub(crate) fn guest_may(
    content: &Content,
    def: &crate::content::ObjectDef,
    inter: &crate::content::Interaction,
) -> bool {
    let rules = &content.social_rules;
    !def.private
        && inter.tags & rules.busy_tags & !rules.guest_tags == 0
        && inter.recipe.is_none()
        && !inter.cares()
}

/// Why someone may not take an interaction at their age, wherever it is (see `unfit`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unfit {
    /// Every stage it is for (`Interaction::stages`) is older than theirs: wine for a child.
    TooYoung,
    /// It is for younger ones only: a toddler's spring rider for a grown-up.
    TooBig,
    /// Homework is a pupil's.
    NoHomework,
    /// Looking after a baby is a grown-up's.
    TooYoungToCare,
}

/// Why someone of `age` may not take `inter` (None: they may): its life stages, homework only
/// at a school stage, and caring for a baby only grown up. Free will and the player's orders
/// keep it alike.
pub(crate) fn unfit(
    content: &Content,
    age: f32,
    inter: &crate::content::Interaction,
) -> Option<Unfit> {
    let life = &content.life;
    let stage = life.stage(age);
    if !inter.for_stage(stage) {
        return Some(if crate::content::all_older(inter.stages, stage) {
            Unfit::TooYoung
        } else {
            Unfit::TooBig
        });
    }
    if inter.homework > 0.0 && !life.stage_at(age).is_some_and(|s| s.school) {
        return Some(Unfit::NoHomework);
    }
    (inter.cares() && !life.adult(age)).then_some(Unfit::TooYoungToCare)
}

/// A condition of a use that isn't met now (`unmet`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unmet {
    /// Not at this hour (content `hours`).
    Hours,
    /// Not under a roof (content `sky`).
    Sky,
    /// Nothing to work on (content `workOn`).
    Nothing(crate::content::WorkOn),
}

/// Which of `inter`'s conditions isn't met now (None: they all are): the hours of the day it
/// starts in, open sky over `obj`, and something to work on (`work`: what the resident's
/// household has, `World::work_waiting`). Free will and the player's orders keep them alike.
pub(crate) fn unmet(
    lot: &Lot,
    hour: f32,
    obj: &ObjectInstance,
    content: &Content,
    inter: &crate::content::Interaction,
    work: u8,
) -> Option<Unmet> {
    if inter
        .hours
        .is_some_and(|h| !crate::content::in_hours(h, hour))
    {
        return Some(Unmet::Hours);
    }
    if inter.sky
        && obj
            .tiles(content)
            .any(|(x, z)| lot.room_at(x, z) != crate::lot::OUTDOORS)
    {
        return Some(Unmet::Sky);
    }
    inter
        .work_on
        .filter(|w| work & w.bit() == 0)
        .map(Unmet::Nothing)
}

/// An hour of the day (0..24) as the clock shows it: `20:00`, `5:30`.
pub(crate) fn clock_time(hour: f32) -> String {
    let minutes = (hour * 60.0).round() as u32 % 1440;
    format!("{}:{:02}", minutes / 60, minutes % 60)
}

/// `unmet` for `sim` during a step.
pub(crate) fn conditions_met(
    ctx: &Ctx,
    sim: &Sim,
    obj: &ObjectInstance,
    inter: &crate::content::Interaction,
) -> bool {
    // (With what they've just made too: the washing taken out, carried on to dry.)
    let work = ctx.work.get(sim.household as usize).copied().unwrap_or(0) | sim.made;
    unmet(ctx.nav.lot, ctx.hour, obj, ctx.content, inter, work).is_none()
}

/// The slots of seat `seat` (a mask, for `ObjectInstance::pick_slot`) with a footstool in front
/// (content `footstool`: a pouf, an ottoman), where the feet go up on it (the renderer's
/// `footstoolFor`): its middle 0.3–1.2 m ahead of the seat's place and at most 0.35 m to one
/// side. Someone sitting down alone takes that seat.
pub(crate) fn footstool_slots(content: &Content, objects: &[ObjectInstance], seat: u32) -> u32 {
    let o = &objects[seat as usize];
    let def = &content.objects[o.def];
    if def.slots < 2 || def.footstool || !def.interactions.iter().any(|i| i.pose == Pose::Sit) {
        return 0;
    }
    let a = o.angle();
    let (fwd, right) = ([a.sin(), a.cos()], [a.cos(), -a.sin()]);
    let n = (def.slots as usize).min(32);
    let places: [[f32; 2]; 32] =
        std::array::from_fn(|k| if k < n { o.place_at(def, k) } else { [0.0; 2] });
    objects
        .iter()
        .filter(|s| content.objects[s.def].footstool && !s.broken())
        .fold(0, |mask, s| {
            let c = s.centre(content);
            let s = [
                c[0] + s.offset[0] as f32 / 100.0,
                c[1] + s.offset[1] as f32 / 100.0,
            ];
            (0..n).fold(mask, |mask, k| {
                let d = [s[0] - places[k][0], s[1] - places[k][1]];
                let ahead = d[0] * fwd[0] + d[1] * fwd[1];
                let side = d[0] * right[0] + d[1] * right[1];
                if (0.3..=1.2).contains(&ahead) && side.abs() <= 0.35 {
                    mask | 1 << k
                } else {
                    mask
                }
            })
        })
}

/// Whether `sim` (of `age`) could only take the upper bunk of `obj` (the lower one someone
/// else's) and may not (`ObjectDef::upper_stages`): an elder, say.
pub(crate) fn upper_barred(content: &Content, obj: &ObjectInstance, sim: u32, age: f32) -> bool {
    let def = &content.objects[obj.def];
    def.bunk
        && !stage_in(def.upper_stages, content.life.stage(age))
        && obj.users[0].is_some_and(|u| u != sim)
}

/// Whether one of `usable` is a bed free for `sim` tonight: working, a place on it free for them
/// (or theirs) and not kept for a couple (`privacy::kept_for_couple`), for their age, and not one
/// they couldn't get to a moment ago; a spare one (`Interaction::spare`: the sofa) only if `spare`,
/// and then one somebody only sits on for now (watching TV on the sofa) counts as free: they'll get
/// up, nobody sleeps there.
pub(crate) fn has_bed(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    usable: &[u32],
    spare: bool,
) -> bool {
    let content = ctx.content;
    usable.iter().map(|&o| &objects[o as usize]).any(|o| {
        let def = &content.objects[o.def];
        !o.broken()
            && !sim.couldnt_reach(o.id, ctx.tick)
            && !crate::privacy::kept_for_couple(ctx, objects, sim, o)
            && def.interactions.iter().enumerate().any(|(i, inter)| {
                inter.tags & content.day_rhythm.sleep_tags != 0
                    && (spare || !inter.spare)
                    && !inter.baby
                    && (o.can_use_for(sim.id, def, i)
                        || (inter.spare
                            && o.users()
                                .all(|u| ctx.briefs.get(u as usize).is_none_or(|b| !b.lying))))
                    && unfit(content, sim.age, inter).is_none()
                    && !upper_barred(content, o, sim.id, sim.age)
            })
    })
}

/// What the player may have `sim` use on `plot`: as `access`, but their own home and public
/// places stay theirs to use while they're a guest somewhere.
pub(crate) fn order_access(
    households: &[Household],
    plots: &[Plot],
    sim: &Sim,
    plot: Option<u32>,
) -> Access {
    let home = households[sim.household as usize].plot;
    match plot {
        None => Access::Full,
        Some(p) if plots[p as usize].public || home == Some(p) => Access::Full,
        _ => access_in(households, plots, sim, plot),
    }
}

/// `access` outside a step.
pub(crate) fn access_in(
    households: &[Household],
    plots: &[Plot],
    sim: &Sim,
    plot: Option<u32>,
) -> Access {
    let home = households[sim.household as usize].plot;
    // A guest stays at the host's place until it's time to go home.
    if let Some(visit) = sim.visiting {
        return if plot == Some(visit.plot) {
            Access::Guest
        } else {
            Access::None
        };
    }
    match plot {
        None => Access::Full,
        Some(p) if plots[p as usize].public || home == Some(p) => Access::Full,
        Some(p) if sim.visiting.is_some_and(|v| v.plot == p) => Access::Guest,
        Some(_) => Access::None,
    }
}

/// The first thing `inter` requires that no working object on `object`'s plot provides, if any
/// (content `requires` / `provides`): a table's coffee needs a coffee machine. With `index`
/// (current during a tick) only that plot's objects are looked at.
pub(crate) fn missing_requirement<'a>(
    content: &'a Content,
    object_plot: &[Option<u32>],
    index: Option<&ObjectIndex>,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> Option<&'a str> {
    if inter.requires == 0 {
        return None;
    }
    let plot = object_plot.get(object as usize).copied().flatten();
    let provides = |o: &ObjectInstance| {
        if o.broken() {
            0
        } else {
            content.objects[o.def].provides
        }
    };
    let provided = match (plot, index) {
        (Some(p), Some(index)) => index
            .on(p)
            .iter()
            .fold(0, |m, &o| m | provides(&objects[o as usize])),
        _ => objects
            .iter()
            .filter(|o| object_plot.get(o.id as usize).copied().flatten() == plot)
            .fold(0, |m, o| m | provides(o)),
    };
    let missing = inter.requires & !provided;
    if missing == 0 {
        return None;
    }
    (0..content.tags.len())
        .find(|&i| missing & (1 << i) != 0)
        .map(|i| content.tags[i].as_str())
}

/// Whether `sim` could start `inter` on `object` now as far as what it requires and its
/// conditions go (`missing_requirement`, `unmet`).
pub(crate) fn requirements_met(
    ctx: &Ctx,
    sim: &Sim,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> bool {
    missing_requirement(
        ctx.content,
        ctx.object_plot,
        Some(ctx.index),
        objects,
        object,
        inter,
    )
    .is_none()
        && conditions_met(ctx, sim, &objects[object as usize], inter)
        && !closed_now(ctx, objects, object, inter)
}

/// Whether it's sold at a post that's closed now, or needs fetching from one (`staff::closed`).
pub(crate) fn closed_now(
    ctx: &Ctx,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> bool {
    closed_at(ctx, objects, object, inter, ctx.hour)
}

/// `closed_now` at `hour` of the day.
pub(crate) fn closed_at(
    ctx: &Ctx,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
    hour: f32,
) -> bool {
    let plot = ctx.object_plot.get(object as usize).copied().flatten();
    let on = plot.map_or(&[][..], |p| ctx.index.on(p));
    crate::staff::closed(ctx.content, ctx.staff, objects, on, object, inter, hour).is_some()
}
