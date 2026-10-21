//! Carrying something somewhere and putting it there (`world::TaskKind::Carry`).
//!
//! **A baby to the high chair and back (G-66).** An object with a place for a baby (`baby`) and a
//! care of theirs to do there with the baby carried in (`Interaction::carried`: the high chair's
//! *Feed the baby*) takes over the crib's care of the same id by day: the grown-up who sets off to
//! do it at the crib goes over to the crib (stage 0), lifts the baby out (a moment, `LIFT_MINUTES`,
//! the baby still in the crib), carries them in the arms to the high chair (stage 1), puts them in
//! it (another moment) and does that care there (`TaskKind::Use`, from beside it). Done, they
//! carry the baby back to a crib the same way. At night the crib's care stays at the crib (a feed
//! in the arms). While carried the baby is in no place (`Sim::carried_by`) and goes where the one
//! carrying them goes (`after_step`); dropped (the carer called away, the high chair taken), or
//! left in the high chair when its time is up, a baby is back in their crib at once, as a baby with
//! nowhere to be always is (`world::crib_for`).
//!
//! **A harvest to the fridge (G-68).** A harvest's produce (`Interaction::yields`) is carried in
//! the hands (`Sim::held`) to the home's nearest fridge (stage 1) and put away there as an item
//! (`ItemAt::Stored`), eaten later as leftovers are; a harvester hungry for it has some then and
//! there. With no fridge at home they sit down and eat a serving (as `eat`); cut short on the way,
//! what they carry is put down where they are (`cooking::settle`: on a surface, else the floor).

use crate::content::{Content, Pose, TagMask};
use crate::items::{Held, ItemAt, ItemInstance, Items};
use crate::world::{Activity, Ctx, ObjectInstance, Phase, Sim, Task, TaskKind, World};

/// Game minutes lifting a baby out of their place, or putting them in.
pub const LIFT_MINUTES: f32 = 1.5;
/// Game minutes putting a harvest away in the fridge.
pub const STOW_MINUTES: f32 = 1.0;

/// What a carer did to a baby this tick (applied after the step, `after_step`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CarryOut {
    /// Lifted them out of their place into the arms.
    Lift(u32),
    /// Put them in `object` for its `interaction` (a baby place).
    Put { baby: u32, object: u32, interaction: usize },
}

/// The baby place of `def` (its `baby` interaction).
fn baby_place(content: &Content, obj: &ObjectInstance) -> Option<usize> {
    content.objects[obj.def].interactions.iter().position(|i| i.baby)
}

/// The object on the plot `object` stands on with `id` as a `carried` care, free to take baby
/// `baby` and a carer `sim` (not broken, both places free), nearest first.
fn carried_to(ctx: &Ctx, objects: &[ObjectInstance], sim: &Sim, object: u32, id: &str, baby: u32) -> Option<u32> {
    let content = ctx.content;
    let plot = ctx.object_plot.get(object as usize).copied().flatten()?;
    let at = objects[object as usize].centre(content);
    ctx.index
        .on(plot)
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| o.id != object && !o.broken())
        .filter(|o| {
            let def = &content.objects[o.def];
            let care = def.interactions.iter().position(|i| i.carried && i.id == id);
            let place = baby_place(content, o);
            care.zip(place).is_some_and(|(c, p)| o.can_use_for(sim.id, def, c) && o.can_use_for(baby, def, p))
                && crate::world::unfit(content, ctx.briefs[baby as usize].age, &def.interactions[place.unwrap_or(0)]).is_none()
        })
        .min_by(|a, b| {
            let d = |o: &ObjectInstance| {
                let c = o.centre(content);
                (c[0] - at[0]).hypot(c[1] - at[1])
            };
            d(a).total_cmp(&d(b))
        })
        .map(|o| o.id)
}

/// What `task` becomes when it is a crib's care done elsewhere with the baby carried there (see
/// the module notes): by day, with the baby lying in it, and a place for that care free.
pub(crate) fn instead(sim: &Sim, task: &Task, ctx: &Ctx, objects: &[ObjectInstance]) -> Option<Task> {
    let TaskKind::Use { object, interaction } = task.kind else { return None };
    let content = ctx.content;
    let obj = objects.get(object as usize)?;
    let inter = content.objects[obj.def].interactions.get(interaction)?;
    if !inter.cares() || inter.carried || crate::world::is_night(sim, ctx) {
        return None;
    }
    let lying = baby_place(content, obj).filter(|&p| content.objects[obj.def].interactions[p].baby_bed())?;
    let baby = obj.users[content.objects[obj.def].slots_for(lying).start].filter(|&u| ctx.briefs.get(u as usize).is_some_and(|b| b.baby))?;
    let to = carried_to(ctx, objects, sim, object, &inter.id, baby)?;
    Some(Task { kind: TaskKind::Carry { baby: Some(baby), from: object, to, stage: 0 }, ..*task })
}

/// The tags a carry counts as (a baby's: the care it's for, so it's as busy as caring; a
/// harvest's: none).
fn tags(content: &Content, objects: &[ObjectInstance], baby: Option<u32>, to: u32) -> TagMask {
    baby.and(objects.get(to as usize))
        .and_then(|o| content.objects[o.def].interactions.iter().find(|i| i.cares()).map(|i| i.tags))
        .unwrap_or(0)
}

/// Where a carer stands to do a `carried` care at `obj`: beside it (a high chair at a table has
/// the table in front and its back to the room), else in front, else behind.
pub(crate) fn care_spot(ctx: &Ctx, obj: &ObjectInstance) -> (i32, i32) {
    let content = ctx.content;
    let c = obj.centre(content);
    let yaw = obj.yaw();
    let ahead = |t: &(i32, i32)| ((t.0 as f32 + 0.5 - c[0]) * yaw.sin() + (t.1 as f32 + 0.5 - c[1]) * yaw.cos()).abs();
    let mut tiles: Vec<(i32, i32)> = crate::surface::approach_tiles(content, &ctx.nav, obj).collect();
    tiles.sort_by(|a, b| ahead(a).total_cmp(&ahead(b)));
    tiles.into_iter().find(|&(x, z)| ctx.nav.tile_free(x, z)).unwrap_or_else(|| obj.front_tile(content))
}

/// The tile in front of place `slot` on `obj` (`ObjectInstance::slot_position`): a crib's changing
/// table beside the cot (`Interaction::at_slot`); none if it can't be stood on.
pub(crate) fn slot_front(ctx: &Ctx, obj: &ObjectInstance, slot: usize) -> Option<(i32, i32)> {
    let content = ctx.content;
    let p = obj.slot_position(content, slot);
    let yaw = obj.yaw();
    let deep = f32::from(content.objects[obj.def].footprint[1]) / 2.0 + 0.5;
    let t = ((p[0] + yaw.sin() * deep).floor() as i32, (p[1] + yaw.cos() * deep).floor() as i32);
    ctx.nav.tile_free(t.0, t.1).then_some(t)
}

/// Sets off on a carry (or its next stage): over to the baby's place, to where the baby goes, or
/// with the harvest to the fridge.
pub(crate) fn start(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let content = ctx.content;
    let TaskKind::Carry { baby, from, to, stage } = task.kind else { return };
    sim.current = None;
    let (Some(f), Some(t)) = (objects.get(from as usize), objects.get(to as usize)) else { return };
    let goal = match (baby, stage) {
        (Some(b), 0) => {
            // The baby still there, and a place beside them for the carer.
            let def = &content.objects[f.def];
            if f.broken() || f.slot_of(b).is_none() || sim.id == b {
                return;
            }
            let carers = 1.min(def.slots as usize - 1)..def.slots as usize;
            if !f.can_use_in(sim.id, carers.clone()) {
                return;
            }
            // (Lifted out of the crib from in front of it, out of the high chair from beside it.)
            let goal = match baby_place(content, f).filter(|&p| def.interactions[p].baby_bed()) {
                Some(_) => ctx.index.links.approach_of(from).unwrap_or_else(|| f.front_tile(content)),
                None => care_spot(ctx, f),
            };
            let Some(waypoints) = crate::world::walk_route(ctx, sim, goal, [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5]) else { return };
            objects[from as usize].reserve_in(sim.id, carers);
            sim.pose = Pose::Stand;
            sim.current = Some(Activity { task, phase: Phase::Routing { waypoints, next: 0 }, tags: tags(content, objects, baby, to) });
            return;
        }
        (Some(_), _) if t.broken() => return,
        (Some(_), _) if baby_place(content, t).is_some_and(|p| content.objects[t.def].interactions[p].baby_bed()) => {
            ctx.index.links.approach_of(to).unwrap_or_else(|| t.front_tile(content))
        }
        (Some(_), _) => care_spot(ctx, t),
        (None, _) => {
            if sim.held.is_none() || t.broken() {
                return;
            }
            ctx.index.links.approach_of(to).unwrap_or_else(|| t.front_tile(content))
        }
    };
    let Some(waypoints) = crate::world::walk_route(ctx, sim, goal, [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5]) else { return };
    sim.pose = Pose::Stand;
    sim.current = Some(Activity { task, phase: Phase::Routing { waypoints, next: 0 }, tags: tags(content, objects, baby, to) });
}

/// Got there: facing it, a moment lifting the baby out (stage 0), putting them in (stage 1: in
/// their place from now), or putting the harvest away.
pub(crate) fn arrive(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let content = ctx.content;
    let Some(act) = sim.current.as_mut() else { return };
    let TaskKind::Carry { baby, from, to, stage } = act.task.kind else { return };
    let at = if stage == 0 { from } else { to };
    let o = &objects[at as usize];
    if let Some(b) = baby.filter(|_| stage == 1) {
        // (The place taken meanwhile, or broken: they put the baby down, back in the crib.)
        let def = &content.objects[o.def];
        let Some(place) = baby_place(content, o).filter(|&p| !o.broken() && o.can_use_for(b, def, p)) else {
            crate::world::end_activity(sim, content, objects);
            return;
        };
        sim.carry_out = Some(CarryOut::Put { baby: b, object: to, interaction: place });
    }
    let c = o.centre(content);
    sim.yaw = (c[0] - sim.pos[0]).atan2(c[1] - sim.pos[1]);
    sim.pose = Pose::Stand;
    act.phase = Phase::Using { elapsed: 0.0 };
}

/// A moment of lifting, putting in or putting away: whether it's done.
pub(crate) fn using(sim: &Sim, elapsed: f32) -> bool {
    let Some(TaskKind::Carry { baby, .. }) = sim.current.as_ref().map(|a| a.task.kind) else { return true };
    elapsed >= if baby.is_some() { LIFT_MINUTES } else { STOW_MINUTES }
}

/// The moment is over: the baby lifted out, they're off with them (stage 1); the baby put in, the
/// care follows there (`Interaction::carried`); the harvest put in the fridge.
pub(crate) fn step_done(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items) {
    let content = ctx.content;
    let Some(act) = sim.current.take() else { return };
    let task = act.task;
    let TaskKind::Carry { baby, from, to, stage } = task.kind else { return };
    match baby {
        Some(b) if stage == 0 => {
            sim.carry_out = Some(CarryOut::Lift(b));
            objects[from as usize].release(sim.id);
            start(sim, Task { kind: TaskKind::Carry { baby, from, to, stage: 1 }, ..task }, ctx, objects);
        }
        Some(_) => {
            let o = &objects[to as usize];
            let def = &content.objects[o.def];
            if let Some(care) = def.interactions.iter().position(|i| i.carried) {
                sim.follow_up = Some(Task { kind: TaskKind::Use { object: to, interaction: care }, routine: false, ..task });
            }
        }
        None => {
            let Some(held) = sim.held.take() else { return };
            let o = &objects[to as usize];
            let id = items.add(ItemInstance {
                id: 0,
                def: held.def,
                at: ItemAt::Stored { x: o.x, z: o.z },
                servings: held.servings,
                quality: held.quality,
                made_at: held.made_at,
                household: sim.household,
                spoiled: false,
                recipe: held.recipe,
            });
            // Hungry for it: some now.
            if held.recipe.is_some_and(|r| crate::cooking::hungry_for(sim, content, &content.recipes[r])) {
                let eat = Task { kind: TaskKind::Eat { item: Some(id), seat: None, plate: None, stage: 0 }, routine: false, ..task };
                crate::cooking::start(sim, eat, ctx, objects, items);
            }
        }
    }
}

/// A harvest done (`Interaction::yields`, `quality` as the dish of a cook): its produce in the
/// harvester's hands, off to the fridge with it; with no fridge at home, they sit down and eat a
/// serving.
pub(crate) fn harvested(sim: &mut Sim, task: Task, y: crate::content::Yield, quality: f32, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut Items) {
    let content = ctx.content;
    let r = &content.recipes[y.recipe];
    let home = ctx.households[sim.household as usize].plot;
    let fridge = home.and_then(|p| {
        ctx.index
            .on(p)
            .iter()
            .map(|&o| &objects[o as usize])
            .filter(|o| content.objects[o.def].stores && !o.broken())
            .min_by(|a, b| {
                let d = |o: &ObjectInstance| {
                    let c = o.centre(content);
                    (c[0] - sim.pos[0]).hypot(c[1] - sim.pos[1])
                };
                d(a).total_cmp(&d(b))
            })
            .map(|o| o.id)
    });
    let Some(fridge) = fridge else {
        return crate::cooking::eat_now(sim, task, y.recipe, quality, ctx, objects, items);
    };
    sim.held = Some(Held { def: r.item, servings: y.servings, quality, made_at: ctx.tick, recipe: Some(y.recipe) });
    start(sim, Task { kind: TaskKind::Carry { baby: None, from: fridge, to: fridge, stage: 1 }, routine: false, ..task }, ctx, objects);
}

/// The carried care is done: the baby back to a crib (the nearest free one on the plot).
pub(crate) fn carry_back(sim: &Sim, task: Task, ctx: &Ctx, objects: &[ObjectInstance], object: u32) -> Option<Task> {
    let content = ctx.content;
    let o = objects.get(object as usize)?;
    let place = baby_place(content, o)?;
    let baby = o.users[content.objects[o.def].slots_for(place).start].filter(|&u| u != sim.id && ctx.briefs.get(u as usize).is_some_and(|b| b.baby))?;
    let plot = ctx.object_plot.get(object as usize).copied().flatten()?;
    let at = o.centre(content);
    let crib = ctx
        .index
        .on(plot)
        .iter()
        .map(|&c| &objects[c as usize])
        .filter(|c| !c.broken())
        .filter(|c| {
            let def = &content.objects[c.def];
            def.interactions.iter().position(|i| i.baby_bed()).is_some_and(|p| c.can_use_for(baby, def, p))
        })
        .min_by(|a, b| {
            let d = |c: &ObjectInstance| {
                let p = c.centre(content);
                (p[0] - at[0]).hypot(p[1] - at[1])
            };
            d(a).total_cmp(&d(b))
        })?;
    Some(Task { kind: TaskKind::Carry { baby: Some(baby), from: object, to: crib.id, stage: 0 }, routine: false, ..task })
}

/// Who `s` is carrying in the arms now (lifting them out, or on the way with them), for the
/// renderer (`snapshot::sim::CARRY`) and `after_step`.
pub fn carrying(s: &Sim) -> Option<u32> {
    let a = s.current()?;
    match (a.task.kind, &a.phase) {
        (TaskKind::Carry { baby: Some(b), stage: 0, .. }, Phase::Using { .. }) => Some(b),
        (TaskKind::Carry { baby: Some(b), stage: 1, .. }, Phase::Routing { .. }) => Some(b),
        _ => None,
    }
}

/// After everyone's step: babies lifted out of their place into someone's arms, or put in a new
/// one; and carried babies go where the one carrying them goes. One no longer carried by them
/// (put down anywhere else) has no place, and is back in their crib on their next step.
pub(crate) fn after_step(w: &mut World) {
    for i in 0..w.sims.len() {
        let Some(out) = w.sims[i].carry_out.take() else { continue };
        match out {
            CarryOut::Lift(b) => {
                let World { sims, content, objects, .. } = &mut *w;
                let Some(baby) = sims.get_mut(b as usize) else { continue };
                if !baby.here() || !content.life.baby(baby.age) || baby.carried_by.is_some() {
                    continue;
                }
                crate::world::end_activity(baby, content, objects);
                baby.queue.clear();
                baby.carried_by = Some(i as u32);
                baby.pose = Pose::Lie;
            }
            CarryOut::Put { baby: b, object, interaction } => {
                let World { sims, content, objects, .. } = &mut *w;
                let Some(baby) = sims.get_mut(b as usize).filter(|s| s.carried_by == Some(i as u32)) else { continue };
                baby.carried_by = None;
                let o = &mut objects[object as usize];
                let def = &content.objects[o.def];
                if !o.reserve_for(b, def, interaction) {
                    continue;
                }
                let inter = &def.interactions[interaction];
                baby.seated_from = Some((baby.pos[0].floor() as i32, baby.pos[1].floor() as i32));
                baby.pos = o.slot_position(content, o.slot_of(b).unwrap_or(0));
                baby.yaw = o.yaw();
                baby.pose = inter.pose;
                baby.current = Some(Activity {
                    task: Task { kind: TaskKind::Use { object, interaction }, directed: false, routine: false },
                    phase: Phase::Using { elapsed: 0.0 },
                    tags: inter.tags,
                });
            }
        }
    }
    for b in 0..w.sims.len() {
        let Some(c) = w.sims[b].carried_by else { continue };
        let carer = &w.sims[c as usize];
        let held = carer.here() && carrying(carer) == Some(b as u32) && w.content.life.baby(w.sims[b].age) && w.sims[b].here();
        let (pos, yaw) = (carer.pos, carer.yaw);
        let baby = &mut w.sims[b];
        if held {
            baby.pos = pos;
            baby.yaw = yaw;
        } else {
            baby.carried_by = None;
            baby.pose = Pose::Stand;
        }
    }
}
