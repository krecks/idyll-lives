//! Riding a bike (or an e-scooter) to work or school and back (content `vehicle`).
//!
//! A thing to ride is furniture standing on the household's lot (`ObjectDef::vehicle`). Someone
//! off to work or school with no seat in the household's car takes the fastest one at home that
//! fits them (`Vehicle::stages`), if the ride from it to the way out of town is longer than its
//! `minRide`: they walk over to it (`Sim::bound_for_bike`, its slot theirs from then on), get on,
//! and ride their way out at its `speed` (`Sim::riding`; the renderer draws it under them and not
//! at its place). It goes to work with them. Home again, they ride back to its place and get off,
//! and it stands there as before. A ride cut short (called away, sent somewhere else) ends where
//! they are: the bike is back at its place.
//!
//! Where their way to work is timed (`life::Commute`), the bike is picked as it's worked out
//! (`commute_bike`) and they set off on that one (`bike_for`): a way timed on foot is walked, and
//! a bike one housemate's way is timed by isn't another's.
//!
//! Neither is saved: after a load the bike is at its place, free, and they walk.

use crate::content::{Content, stage_in};
use crate::world::{Ctx, ObjectInstance, Sim, TaskKind};

/// A free thing to ride at home for `sim`'s way to `to` (the way out of town): the one their way
/// to work was timed by (`life::Commute::bike`), else the fastest that fits them (`fits`), not
/// one a housemate's way is timed by. None when they're not at home.
pub(crate) fn bike_for(
    ctx: &Ctx,
    sim: &Sim,
    objects: &[ObjectInstance],
    to: [f32; 2],
) -> Option<u32> {
    let content = ctx.content;
    let home = ctx.households.get(sim.household as usize)?.plot?;
    if sim.visiting.is_some() || ctx.briefs.get(sim.id as usize)?.plot != Some(home) {
        return None;
    }
    let stage = content.life.stage(sim.age);
    let planned = sim
        .commute
        .filter(|c| c.home == Some(home))
        .and_then(|c| c.bike);
    let housemates = |o: u32| {
        ctx.briefs
            .iter()
            .enumerate()
            .any(|(j, b)| j != sim.id as usize && b.household == sim.household && b.bike == Some(o))
    };
    let free = |o: u32| {
        let obj = objects.get(o as usize)?;
        let mine = obj.users.iter().all(|u| u.is_none_or(|u| u == sim.id));
        (mine && !housemates(o))
            .then(|| fits(content, obj, stage, to))
            .flatten()
    };
    if let Some(o) = planned.filter(|&o| ctx.index.on(home).contains(&o) && free(o).is_some()) {
        return Some(o);
    }
    ctx.index
        .on(home)
        .iter()
        .filter_map(|&o| Some((free(o)?, o)))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, o)| o)
}

/// `obj`'s speed if it's a thing to ride for someone at `stage`, not broken, and the ride from it
/// to `to` (the way out of town) is at least its `minRide`. (From the bike, not from where they
/// are: the same answer as their way to work is worked out and as they set off.)
fn fits(content: &Content, obj: &ObjectInstance, stage: usize, to: [f32; 2]) -> Option<f32> {
    let v = content.objects[obj.def].vehicle?;
    let at = spot(content, obj);
    let ride = (to[0] - at[0]).hypot(to[1] - at[1]);
    (!obj.broken() && stage_in(v.stages, stage) && ride >= v.min_ride).then_some(v.speed)
}

/// How many times as fast as walking `sim` goes now: their bike's `speed` while they ride it.
pub(crate) fn speed(content: &Content, sim: &Sim, objects: &[ObjectInstance]) -> f32 {
    sim.riding
        .and_then(|o| objects.get(o as usize))
        .and_then(|o| content.objects[o.def].vehicle)
        .map_or(1.0, |v| v.speed)
}

/// `sim` sets off for `bike`: it's theirs until they're back.
pub(crate) fn claim(sim: &mut Sim, objects: &mut [ObjectInstance], bike: u32) {
    sim.bound_for_bike = Some(bike);
    if let Some(o) = objects.get_mut(bike as usize)
        && !o.users.contains(&Some(sim.id))
    {
        o.users[0] = Some(sim.id);
    }
}

/// At the bike: on they get.
pub(crate) fn get_on(sim: &mut Sim) {
    sim.riding = sim.bound_for_bike.take();
}

/// Where to get on or off `bike`: in front of it.
pub(crate) fn spot(content: &Content, bike: &ObjectInstance) -> [f32; 2] {
    let (x, z) = bike.front_tile(content);
    [x as f32 + 0.5, z as f32 + 0.5]
}

/// Off the bike (home, or the ride cut short): it's at its place again, free.
pub(crate) fn get_off(sim: &mut Sim, objects: &mut [ObjectInstance]) {
    for o in [sim.riding.take(), sim.bound_for_bike.take()]
        .into_iter()
        .flatten()
    {
        if let Some(obj) = objects.get_mut(o as usize) {
            obj.release(sim.id);
        }
    }
}

/// Whether `sim` is on the way to work or school, at it, or on the way home from it: what a
/// bike is ridden for. Anything else and they get off (`get_off`).
pub(crate) fn commuting(sim: &Sim) -> bool {
    let commute = |k: &TaskKind| matches!(k, TaskKind::Work | TaskKind::GoHome);
    sim.away_until.is_some()
        || sim.transition.is_some()
        || sim.current.as_ref().is_some_and(|a| commute(&a.task.kind))
        || sim.queue.front().is_some_and(|t| commute(&t.kind))
}

/// The bike `i`'s way to work or school is timed by (`life::refresh_commute`), for the way out
/// at `to`, and how many times as fast as walking it goes: the fastest at home that fits them
/// (`fits`) and no housemate's way is timed by already (first come, first served: in household
/// order, as their ways are first worked out). None: they walk.
pub(crate) fn commute_bike(w: &crate::world::World, i: usize, to: [f32; 2]) -> Option<(u32, f32)> {
    let sim = &w.sims[i];
    let home = w.households.get(sim.household as usize)?.plot?;
    let stage = w.content.life.stage(sim.age);
    let housemates = |o: u32| {
        w.sims.iter().enumerate().any(|(j, s)| {
            j != i && s.household == sim.household && s.commute.is_some_and(|c| c.bike == Some(o))
        })
    };
    w.object_index
        .on(home)
        .iter()
        .filter(|&&o| !housemates(o))
        .filter_map(|&o| Some((o, fits(&w.content, w.objects.get(o as usize)?, stage, to)?)))
        .max_by(|a, b| a.1.total_cmp(&b.1))
}
