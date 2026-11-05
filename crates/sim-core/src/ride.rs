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
//! **Out on one.** In an open-world town a visit or an outing beyond walking range goes by bike
//! too, where the household's car doesn't (`world::leisure_way`): they ride there on the
//! fastest free one at home that fits them (`leisure_bike`), get off by the place and leave it
//! there (`Sim::bike_parked`: still theirs, gone from its place), and ride it home when they go.
//! One sent off somewhere else meanwhile leaves it: it's back at its place, as a ride cut short.
//!
//! None of it is saved: after a load the bike is at its place, free, and they walk.

use crate::content::{Content, stage_in};
use crate::index::ObjectIndex;
use crate::world::{Ctx, Household, ObjectInstance, Sim, SimBrief, TaskKind};

/// A free thing to ride at home for `sim`'s way to `to` (the way out of town): the one their way
/// to work was timed by (`life::Commute::bike`), else the fastest that fits them (`fits`), not
/// one a housemate's way is timed by. None when they're not at home.
pub(crate) fn bike_for(
    ctx: &Ctx,
    sim: &Sim,
    objects: &[ObjectInstance],
    to: [f32; 2],
) -> Option<u32> {
    let (content, index, briefs) = (ctx.content, ctx.index, ctx.briefs);
    let home = ctx.households.get(sim.household as usize)?.plot?;
    if sim.visiting.is_some() || briefs.get(sim.id as usize)?.plot != Some(home) {
        return None;
    }
    let planned = sim
        .commute
        .filter(|c| c.home == Some(home))
        .and_then(|c| c.bike);
    let free = |o: u32| free_at(content, briefs, objects, sim, o, Some(to));
    if let Some(o) = planned.filter(|&o| index.on(home).contains(&o) && free(o).is_some()) {
        return Some(o);
    }
    fastest(index, home, free).map(|(o, _)| o)
}

/// `o`'s speed if it's a thing at home `sim` may ride now: free (or theirs already), no
/// housemate's way to work is timed by it nor are they out on it, and it fits them (`fits`; with
/// `to`, a ride there of its `minRide` at least).
fn free_at(
    content: &Content,
    briefs: &[SimBrief],
    objects: &[ObjectInstance],
    sim: &Sim,
    o: u32,
    to: Option<[f32; 2]>,
) -> Option<f32> {
    let obj = objects.get(o as usize)?;
    let mine = obj.users.iter().all(|u| u.is_none_or(|u| u == sim.id));
    let housemates = briefs
        .iter()
        .enumerate()
        .any(|(j, b)| j != sim.id as usize && b.household == sim.household && b.bike == Some(o));
    (mine && !housemates)
        .then(|| fits(content, obj, content.life.stage(sim.age), to))
        .flatten()
}

/// Of the things on `plot` that `speed` gives a speed for, the fastest, and its speed.
fn fastest(
    index: &ObjectIndex,
    plot: u32,
    speed: impl Fn(u32) -> Option<f32>,
) -> Option<(u32, f32)> {
    index
        .on(plot)
        .iter()
        .filter_map(|&o| Some((o, speed(o)?)))
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// The thing to ride `sim` would take out to a visit or an outing from home (`world::leisure_way`),
/// its speed (times walking) and its `minRide`: the fastest free one at home that fits them, not
/// a housemate's. From the world's parts, so it's asked outside a tick too (`World::visit_fits_now`).
/// None: nothing of theirs to ride, or they aren't at home.
pub(crate) fn leisure_bike(
    content: &Content,
    index: &ObjectIndex,
    briefs: &[SimBrief],
    households: &[Household],
    objects: &[ObjectInstance],
    sim: &Sim,
) -> Option<(u32, f32, f32)> {
    let home = households.get(sim.household as usize)?.plot?;
    if sim.visiting.is_some() || briefs.get(sim.id as usize)?.plot != Some(home) {
        return None;
    }
    let (o, speed) = fastest(index, home, |o| {
        free_at(content, briefs, objects, sim, o, None)
    })?;
    let v = content.objects[objects[o as usize].def].vehicle?;
    Some((o, speed, v.min_ride))
}

/// `obj`'s speed if it's a thing to ride for someone at `stage`, not broken, and the ride from it
/// to `to` (the way out of town; None: anywhere) is at least its `minRide`. (From the bike, not
/// from where they are: the same answer as their way to work is worked out and as they set off.)
fn fits(
    content: &Content,
    obj: &ObjectInstance,
    stage: usize,
    to: Option<[f32; 2]>,
) -> Option<f32> {
    let v = content.objects[obj.def].vehicle?;
    let at = spot(content, obj);
    let far = to.is_none_or(|to| (to[0] - at[0]).hypot(to[1] - at[1]) >= v.min_ride);
    (!obj.broken() && stage_in(v.stages, stage) && far).then_some(v.speed)
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

/// At the bike: on they get (at home, or where they left it out, `park`).
pub(crate) fn get_on(sim: &mut Sim) {
    sim.riding = sim.bound_for_bike.take();
    sim.bike_parked = None;
}

/// Where to get on or off `bike`: in front of it.
pub(crate) fn spot(content: &Content, bike: &ObjectInstance) -> [f32; 2] {
    let (x, z) = bike.front_tile(content);
    [x as f32 + 0.5, z as f32 + 0.5]
}

/// Off the bike (home, or the ride cut short): it's at its place again, free.
pub(crate) fn get_off(sim: &mut Sim, objects: &mut [ObjectInstance]) {
    sim.bike_parked = None;
    for o in [sim.riding.take(), sim.bound_for_bike.take()]
        .into_iter()
        .flatten()
    {
        if let Some(obj) = objects.get_mut(o as usize) {
            obj.release(sim.id);
        }
    }
}

/// Whether `sim` is on the way to work or school, at it, or on the way home from it, or on the
/// way to a visit or an outing: what a bike is ridden for. Anything else and they get off
/// (`get_off`; out on a visit, it's left where they are, `Sim::bike_parked`).
pub(crate) fn commuting(sim: &Sim) -> bool {
    let way = |k: &TaskKind| {
        matches!(
            k,
            TaskKind::Work | TaskKind::GoHome | TaskKind::Visit { .. }
        )
    };
    sim.away_until.is_some()
        || sim.transition.is_some()
        || sim.current.as_ref().is_some_and(|a| way(&a.task.kind))
        || sim.queue.front().is_some_and(|t| way(&t.kind))
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
        .filter_map(|&o| {
            Some((
                o,
                fits(&w.content, w.objects.get(o as usize)?, stage, Some(to))?,
            ))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
}
