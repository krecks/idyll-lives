//! Games for two (`Interaction::partner`, G-65): the seesaw, ping-pong, park chess, pool and
//! foosball are played with someone.
//!
//! Whoever sets off for one (by free will, or told to) starts it with both its places free; the
//! step after, `update` finds someone to play with: a resident on the same plot who may use it
//! (at home, as a guest there, or anyone in a public place), whose age it's for (`world::unfit`),
//! free (doing nothing, or something they picked themselves that isn't busy: not asleep, eating,
//! cooking, talking or on their way out, nothing the player asked for next, not pressed) and not
//! on bad terms with them; the nearest. One the player told to play the same game is fetched
//! first. The partner drops what they're doing and goes to the other place (`Sim::joins`: the
//! nearest free one, the other end). Each waits at their place for the other (`hold`: the use
//! doesn't run, and is drawn as waiting); once both are there it runs for both, each getting
//! what it gives (`social` too: they play together), and it's over for both as soon as either
//! stops (done, cancelled, called away). It goes on to its end, as a job does, rather than
//! stopping the moment the needs it fills are full: one stopping would end it for the other.
//!
//! With nobody free, free will doesn't choose it (`free_near`); if the one picked can't come
//! after all, it's given up. The player's order to play is refused (`world::apply_command`),
//! unless the game has a use for one (`Interaction::alone`: trick shots at the pool table, a
//! foosball practice), which they do instead; the story says so.

use crate::content::{Content, Interaction, ObjectDef};
use crate::social::{EventKind, LEVEL_DISLIKED};
use crate::world::{Access, Ctx, ObjectInstance, Phase, Sim, Task, TaskKind, World};

/// Ticks someone waits at their place for the one they play with to get to theirs (half an
/// hour of game time): one who still isn't there by then isn't coming.
pub(crate) const PARTNER_WAIT_TICKS: u32 = 30 * crate::TICKS_PER_MINUTE as u32;

/// The game for two `sim` is at or on their way to next (`Sim::game`).
fn game_of(content: &Content, objects: &[ObjectInstance], sim: &Sim) -> Option<(u32, usize, bool)> {
    let game = |kind: TaskKind| match kind {
        TaskKind::Use { object, interaction } => objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .filter(|i| i.partner)
            .map(|_| (object, interaction)),
        _ => None,
    };
    match sim.current() {
        Some(a) => game(a.task.kind).map(|(o, i)| (o, i, matches!(a.phase, Phase::Using { .. }))),
        None => sim.follow_up.or_else(|| sim.queue().next().copied()).and_then(|t| game(t.kind)).map(|(o, i)| (o, i, false)),
    }
}

/// The game for two resident `sim` is at or on their way to next, as things are now.
pub(crate) fn game(world: &World, sim: u32) -> Option<(u32, usize, bool)> {
    world.sims.get(sim as usize).and_then(|s| game_of(&world.content, &world.objects, s))
}

/// Sets each resident's `Sim::game` as a tick starts.
pub(crate) fn refresh(world: &mut World) {
    let World { content, objects, sims, .. } = world;
    for s in sims.iter_mut() {
        s.game = if s.here() { game_of(content, objects, s) } else { None };
    }
}

/// Whether `me` may take a place at `obj` for its game `i` (of `def`): with both its places free
/// (starting it), or joining the one they were fetched for (`joins`), or the one at the other
/// place who plays the same game (`along`: two told to play it, both games after a load).
pub(crate) fn may_take(obj: &ObjectInstance, def: &ObjectDef, i: usize, me: u32, joins: Option<u32>, along: impl Fn(u32) -> bool) -> bool {
    let range = def.slots_for(i);
    if obj.slot_of(me).is_some_and(|k| range.contains(&k)) {
        return true;
    }
    let users = &obj.users[range];
    let free = users.iter().any(Option::is_none);
    let mut others = users.iter().flatten().filter(|&&u| u != me);
    let first = others.next().copied();
    let more = others.next().is_some();
    match (joins, first) {
        (_, None) => joins.is_none(),
        (Some(a), Some(u)) => u == a && !more && free,
        (None, Some(u)) => along(u) && !more && free,
    }
}

/// Whether someone looks free to play game `inter` at `obj` with `sim` (who chooses it by free
/// will), as the tick's start had them (`update` makes sure); not while they found nobody a
/// moment ago (`Sim::alone_until`).
pub(crate) fn free_near(ctx: &Ctx, sim: &Sim, obj: &ObjectInstance, inter: &Interaction) -> bool {
    if ctx.tick < sim.alone_until {
        return false;
    }
    let plot = ctx.object_plot.get(obj.id as usize).copied().flatten();
    ctx.briefs.iter().enumerate().any(|(j, b)| {
        j as u32 != sim.id
            && b.free
            && b.plot == plot
            && crate::world::unfit(ctx.content, b.age, inter).is_none()
            && crate::social::friend_level(ctx.rels.get(sim.id as usize, j).friendship) > LEVEL_DISLIKED
    })
}

/// How a game for two goes for `me` at their place this step (`world::progress`).
pub(crate) enum Hold {
    /// Both there: it runs.
    Play,
    /// Waiting for the other to get to their place.
    Wait,
    /// The other stopped: it's over.
    Over,
    /// Nobody's coming: given up (or, told to, the game's use for one instead).
    Alone,
}

/// How `me`'s game (`interaction` at `object`) goes this step (`begun`: it has run already;
/// `finding`: someone to play with is being found), as the tick's start had the one they play
/// with (`briefs`); `wait` counts the ticks waited.
pub(crate) fn hold(ctx: &Ctx, me: u32, (object, interaction): (u32, usize), begun: bool, finding: bool, wait: &mut u32, ticks: u32) -> Hold {
    let other = ctx
        .briefs
        .iter()
        .enumerate()
        .find(|&(j, b)| j as u32 != me && b.game.is_some_and(|(o, i, _)| (o, i) == (object, interaction)))
        .map(|(_, b)| b.game.is_some_and(|g| g.2));
    match other {
        Some(true) => {
            *wait = 0;
            Hold::Play
        }
        _ if begun => {
            *wait = 0;
            if other.is_some() { Hold::Play } else { Hold::Over }
        }
        // (Someone being found for them this tick, or on their way.)
        Some(false) if *wait < PARTNER_WAIT_TICKS => {
            *wait = wait.saturating_add(ticks.max(1));
            Hold::Wait
        }
        None if finding => {
            *wait = wait.saturating_add(ticks.max(1));
            Hold::Wait
        }
        _ => {
            *wait = 0;
            Hold::Alone
        }
    }
}

/// Whether `s` is free to be fetched for a game for two at all (`SimBrief::free`, as a tick
/// starts; see the module notes): here, past the crib, not carried, off the bunk's ladder,
/// nothing to do next out of habit, nothing told next, not about to leave for work or school,
/// up, and doing nothing or something they picked that they can drop (`available_for_social`
/// says it isn't busy; not on their way to someone, on the phone or carrying something).
pub(crate) fn free_now(content: &Content, s: &Sim, tick: u64) -> bool {
    let ready = crate::social::minutes_to_ticks(content.day_rhythm.ready_minutes);
    s.here()
        && !content.life.baby(s.age)
        && s.carried_by.is_none()
        && s.ladder_ticks == 0
        && s.follow_up.is_none()
        && !s.queue().any(|t| t.directed)
        && s.available_for_social(content)
        && !s.asleep(content)
        && s.current().is_none_or(|c| matches!(c.task.kind, TaskKind::Use { .. } | TaskKind::MoveTo { .. } | TaskKind::Clean { .. }))
        && !crate::world::leave_at(s, content, tick).is_some_and(|t| t <= tick + ready)
}

/// Whether resident `j` is free to play `inter` at an object on `plot` with `a` now (`free_now`,
/// there, its age for them, allowed to use it, not on bad terms), able to get to it (`front`:
/// the tile it's used from; while the walkable regions are known).
#[allow(clippy::too_many_arguments)]
fn free_for(world: &World, j: usize, a: usize, plot: Option<u32>, at: [f32; 2], front: (i32, i32), def: &ObjectDef, inter: &Interaction) -> bool {
    let content = &world.content;
    let s = &world.sims[j];
    if j == a || !free_now(content, s, world.tick) {
        return false;
    }
    let (x, z) = s.tile();
    let here = match plot {
        Some(_) => world.plot_at(x, z) == plot,
        None => (s.pos[0] - at[0]).hypot(s.pos[1] - at[1]) <= OFF_PLOT_RANGE,
    };
    let access = crate::world::access_in(&world.households, &world.plots, s, plot);
    here
        && crate::world::unfit(content, s.age, inter).is_none()
        && (access == Access::Full || (access == Access::Guest && crate::world::guest_may(content, def, inter)))
        && crate::social::friend_level(world.relationships.get(a, j).friendship) > LEVEL_DISLIKED
        && (world.reach_version != world.nav_version || world.reach.reaches(&world.nav(), (x, z), front))
}

/// How far (m) someone off every plot (on the street) looks for someone to play with.
const OFF_PLOT_RANGE: f32 = 12.0;

/// Who'd play game `interaction` at `object` with `a` now: one the player told to play it next
/// (their next task, nothing they were told before it under way), else the nearest one free
/// (`free_for`).
pub(crate) fn partner_for(world: &World, a: usize, object: u32, interaction: usize) -> Option<usize> {
    let content = &world.content;
    let obj = world.objects.get(object as usize)?;
    let inter = content.objects[obj.def].interactions.get(interaction)?;
    let plot = world.object_plot.get(object as usize).copied().flatten();
    let at = obj.centre(content);
    let front = obj.front_tile(content);
    let told = (0..world.sims.len()).find(|&j| {
        let s = &world.sims[j];
        j != a
            && s.here()
            && s.current().is_none_or(|c| !c.task.directed)
            && s.queue().next().is_some_and(|t| t.directed && matches!(t.kind, TaskKind::Use { object: o, interaction: i } if (o, i) == (object, interaction)))
    });
    told.or_else(|| {
        let far = |j: usize| {
            let p = world.sims[j].pos;
            (p[0] - at[0]).hypot(p[1] - at[1])
        };
        (0..world.sims.len()).filter(|&j| free_for(world, j, a, plot, at, front, &content.objects[obj.def], inter)).min_by(|&x, &y| far(x).total_cmp(&far(y)))
    })
}

/// After the residents' step: finds someone to play with for each who set off for a game for two
/// this tick (`Sim::wants_partner`), and sends them; with nobody, the game is given up (the
/// player's order: its use for one instead, if it has one).
pub(crate) fn update(world: &mut World) {
    for a in 0..world.sims.len() {
        if !std::mem::take(&mut world.sims[a].wants_partner) {
            continue;
        }
        let Some((object, interaction, directed)) = world.sims[a].current().and_then(|c| match c.task.kind {
            TaskKind::Use { object, interaction } => Some((object, interaction, c.task.directed)),
            _ => None,
        }) else {
            continue;
        };
        let content = &world.content;
        let Some(def) = world.objects.get(object as usize).map(|o| &content.objects[o.def]) else { continue };
        if !def.interactions.get(interaction).is_some_and(|i| i.partner) {
            continue;
        }
        // Someone is at it or on their way already (told to play it too).
        let same = |s: &Sim| game_of(content, &world.objects, s).is_some_and(|(o, i, _)| (o, i) == (object, interaction));
        if world.sims.iter().enumerate().any(|(j, s)| j != a && s.here() && same(s)) {
            continue;
        }
        let range = def.slots_for(interaction);
        // (Trying for a baby is with the one it was asked of, sent along with them
        // (`Sim::to_bed`, `privacy::after_step`): nobody else is fetched.)
        let room = world.objects[object as usize].users[range].iter().any(Option::is_none) && !def.interactions[interaction].conceive;
        match room.then(|| partner_for(world, a, object, interaction)).flatten() {
            Some(b) => fetch(world, b, a, object, interaction),
            None => alone(world, a, object, interaction, directed),
        }
    }
}

/// Sends `b` to play game `interaction` at `object` with `a`: they drop what they're doing (an
/// order of the player's to play it stays theirs).
fn fetch(world: &mut World, b: usize, a: usize, object: u32, interaction: usize) {
    let World { content, objects, sims, .. } = world;
    let sim = &mut sims[b];
    let task = match sim.queue.front() {
        Some(t) if t.directed && matches!(t.kind, TaskKind::Use { object: o, interaction: i } if (o, i) == (object, interaction)) => sim.queue.pop_front(),
        _ => None,
    };
    if sim.current().is_some() {
        crate::world::end_activity(sim, content, objects);
    }
    sim.follow_up = None;
    sim.queue.push_front(task.unwrap_or(Task { kind: TaskKind::Use { object, interaction }, directed: false, routine: false }));
    crate::world::trim_queue(&mut sim.queue, 1);
    sim.joins = Some(a as u32);
    sim.game = Some((object, interaction, false));
}

/// Nobody to play game `interaction` at `object` with `a`: the player's order goes on as the
/// game's use for one, if it has one (or is dropped), and the story says so; what they chose
/// themselves is given up, and not chosen again for a while.
fn alone(world: &mut World, a: usize, object: u32, interaction: usize, directed: bool) {
    let World { content, objects, sims, events, tick, .. } = world;
    let def = objects[object as usize].def;
    let solo = content.objects[def].interactions[interaction].alone.filter(|_| directed);
    let sim = &mut sims[a];
    match (solo, sim.current.as_mut()) {
        (Some(k), Some(act)) => {
            act.task.kind = TaskKind::Use { object, interaction: k };
            act.tags = content.objects[def].interactions[k].tags;
        }
        _ => {
            let kind = TaskKind::Use { object, interaction };
            crate::world::end_activity(sim, content, objects);
            // (No game for two for a while: they choose something else.)
            if !directed {
                sim.alone_until = *tick + crate::social::minutes_to_ticks(content.autonomy.retry_minutes);
                sim.remember_unreachable(kind, sim.alone_until);
            }
        }
    }
    sim.partner_wait = 0;
    if directed {
        let kind = if solo.is_some() { EventKind::PlayedAlone } else { EventKind::NoPartner };
        events.push_object(*tick, kind, a, def, None);
    }
}
