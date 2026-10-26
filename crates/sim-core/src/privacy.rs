//! Privacy (G-72): rooms kept shut while someone is in them, the household's own things, and
//! trying for a baby in bed behind a shut door.
//!
//! **Rooms kept shut.** A room of a private kind (content `roomKinds[].private`: the bathroom)
//! is kept shut while someone is in it, and so is the room of a use done behind a shut door
//! (`Interaction::private`: a double bed's *Try for a baby*) while the two are in it at it. Not
//! a room that is two kinds at once (`mixed`: a toilet beside the bed with no wall between),
//! nor one with a door to the outside (a cabin with its toilet in the open: its door is the
//! front door), nor anything on a public place (`private_rooms`, found with the rooms). `refresh`
//! finds who keeps which shut as each tick starts.
//!
//! Someone on their way into a room kept shut by someone else stops short of its doorway
//! (`LOOK` metres before they'd step in) and waits there (`Sim::kept_out`), standing; once
//! nobody keeps it shut they go on. Whoever does the same private use as the one in there (the
//! other one trying for a baby) goes in. After `WAIT_MINUTES` they give up; one who chose it
//! themselves doesn't choose it again for a while (`autonomy.retryMinutes`). Free will doesn't
//! choose something in a room kept shut by someone else, unless it fills a need about to run
//! out: they'd rather wait at the door. Kept out, a need that runs out counts as having nothing
//! for it at home (the accident comes after its `graceMinutes`), and a toilet in a room kept
//! shut counts as taken (a second bathroom is wished for, `rooms::opinions`). The renderer keeps
//! the doors of a room kept shut shut (snapshot `sim::SHUTS`).
//!
//! **The household's own things.** An object kind marked `private` (a wardrobe, a dresser, the
//! bath cabinet) is used by nobody visiting: free will doesn't choose it and an order is refused
//! (`world::guest_may`), as with beds and showers.
//!
//! **Trying for a baby.** When partners' social with the `conceive` effect goes well and the
//! content has somewhere to try (`Interaction::conceive`, the double beds), the two go to bed
//! (`to_bed`): to a free double bed at home in a room of its own (indoors, with a door, nobody
//! else in it but a baby), where they lie under the covers for its time, the door shut. A baby
//! may come of it once it has run its time (`lifecycle::conceive`, as before). Without such a bed
//! the social isn't chosen, nor can the player ask for it.

use crate::content::{Content, Interaction};
use crate::lot::{Lot, OUTDOORS};
use crate::world::{Ctx, ObjectInstance, Phase, Sim, Task, TaskKind, World};

/// Metres ahead along their way someone stops before a room kept shut: short of its doorway.
pub const LOOK: f32 = 0.9;

/// Game minutes someone waits outside a room kept shut before giving up.
pub const WAIT_MINUTES: f32 = 60.0;

/// A room kept shut (a lot room id) and the resident in it who keeps it so.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shut {
    pub room: u16,
    pub by: u32,
    /// The private use they're at in it (an object and its interaction: a double bed's *Try for
    /// a baby*): whoever does the same one with them goes in.
    pub doing: Option<(u32, usize)>,
}

/// Whether a room is kept shut while anyone is in it: of a private kind, not mixed with another
/// (a toilet by the bed with no wall between), with no door to the outside (a cabin with its
/// toilet in the open: the front door isn't a bathroom door), not on a public place.
fn private_room(content: &Content, plots: &[crate::world::Plot], r: &crate::rooms::RoomInfo, outside: &[u16]) -> bool {
    !r.garden
        && r.id != OUTDOORS
        && !r.mixed
        && r.kind.is_some_and(|k| content.room_kinds[k].private)
        && !outside.contains(&r.id)
        && r.plot.is_none_or(|p| plots.get(p as usize).is_none_or(|p| !p.public))
}

/// The rooms of `rooms` kept shut while anyone is in them (`private_room`), sorted.
pub(crate) fn private_rooms(content: &Content, plots: &[crate::world::Plot], rooms: &[crate::rooms::RoomInfo], lot: &Lot) -> Vec<u16> {
    if !content.room_kinds.iter().any(|k| k.private) {
        return Vec::new();
    }
    // Rooms with a door to the outside.
    let mut outside = Vec::new();
    let room = |x: i32, z: i32| lot.room_at(x, z);
    let (w, d) = (lot.width as i32, lot.depth as i32);
    let mut door = |a: u16, b: u16| {
        for (r, across) in [(a, b), (b, a)] {
            if r != OUTDOORS && across == OUTDOORS && !outside.contains(&r) {
                outside.push(r);
            }
        }
    };
    for z in 0..=d {
        for x in 0..w {
            if lot.h_edge(x as usize, z as usize) == crate::lot::Edge::Door {
                door(room(x, z - 1), room(x, z));
            }
        }
    }
    for z in 0..d {
        for x in 0..=w {
            if lot.v_edge(x as usize, z as usize) == crate::lot::Edge::Door {
                door(room(x - 1, z), room(x, z));
            }
        }
    }
    let mut ids: Vec<u16> = rooms.iter().filter(|r| private_room(content, plots, r, &outside)).map(|r| r.id).collect();
    ids.sort_unstable();
    ids
}

/// The private use `s` is at or on their way to (`Interaction::private`), if any.
pub(crate) fn private_use(content: &Content, objects: &[ObjectInstance], s: &Sim) -> Option<(u32, usize)> {
    match s.current()?.task.kind {
        TaskKind::Use { object, interaction } => objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .filter(|i| i.private)
            .map(|_| (object, interaction)),
        _ => None,
    }
}

/// The room a point is in (`OUTDOORS`: none).
fn room_of(lot: &Lot, p: [f32; 2]) -> u16 {
    lot.room_at(p[0].floor() as i32, p[1].floor() as i32)
}

/// The room an object stands in (by the middle of its footprint).
pub(crate) fn room_of_object(content: &Content, lot: &Lot, o: &ObjectInstance) -> u16 {
    room_of(lot, o.centre(content))
}

/// Finds the rooms kept shut as a tick starts (`World::shut`).
pub(crate) fn refresh(w: &mut World) {
    let mut shut = std::mem::take(&mut w.shut);
    shut.clear();
    for s in &w.sims {
        if !s.here() || s.away_until.is_some() || s.carried_by.is_some() || w.content.life.baby(s.age) {
            continue;
        }
        let room = room_of(&w.lot, s.pos);
        if room == OUTDOORS {
            continue;
        }
        let doing = private_use(&w.content, &w.objects, s).filter(|&(o, _)| room_of_object(&w.content, &w.lot, &w.objects[o as usize]) == room);
        if doing.is_some() || w.private_rooms.binary_search(&room).is_ok() {
            shut.push(Shut { room, by: s.id, doing });
        }
    }
    w.shut = shut;
}

/// Whether `room` is kept shut to resident `me` (doing private use `mine`, if any): by someone
/// else in it, not at the same use.
pub(crate) fn shut_to(shut: &[Shut], me: u32, mine: Option<(u32, usize)>, room: u16) -> bool {
    room != OUTDOORS && shut.iter().any(|s| s.room == room && s.by != me && (s.doing.is_none() || s.doing != mine))
}

/// Whether `sim` keeps a room shut now.
pub fn keeps_shut(world: &World, sim: u32) -> bool {
    world.shut.iter().any(|s| s.by == sim)
}

/// Whether the thing at tile `at` (where it's used from) is in a room kept shut to `sim` by
/// someone else, and they aren't in that room already.
pub(crate) fn shut_away(ctx: &Ctx, sim: &Sim, at: (i32, i32)) -> bool {
    if ctx.shut.is_empty() {
        return false;
    }
    let lot = ctx.nav.lot;
    let room = lot.room_at(at.0, at.1);
    room != room_of(lot, sim.pos) && shut_to(ctx.shut, sim.id, None, room)
}

/// Whether `sim`, walking, is about to step into a room kept shut to them: within `LOOK` metres
/// along their way, into another room than the one they're in. Not on the way to someone to
/// talk to (`conversation.rs` walks them), nor on their way out to work, home, a visit or the
/// door (a house whose only way out is through the bathroom: they go through rather than miss
/// the shift).
pub(crate) fn shut_ahead(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> bool {
    if ctx.shut.is_empty() {
        return false;
    }
    let Some(act) = sim.current() else { return false };
    let Phase::Routing { waypoints, next } = &act.phase else { return false };
    if matches!(act.task.kind, TaskKind::Social { .. } | TaskKind::Join { .. } | TaskKind::Work | TaskKind::GoHome | TaskKind::Visit { .. } | TaskKind::Answer { .. }) {
        return false;
    }
    let lot = ctx.nav.lot;
    let stairs = &ctx.nav.stairs;
    let here = room_of(lot, sim.pos);
    let storey = stairs.storey(sim.pos[1].floor() as i32);
    let mine = private_use(ctx.content, objects, sim);
    let (mut at, mut left) = (sim.pos, LOOK);
    for &wp in waypoints.iter().skip(*next) {
        if stairs.storey(wp[1].floor() as i32) != storey {
            break;
        }
        let d = (wp[0] - at[0]).hypot(wp[1] - at[1]);
        let along = d.min(left);
        // (Every quarter metre: no doorway is passed between two looks.)
        let steps = (along / 0.25).ceil().max(1.0) as u32;
        for k in 1..=steps {
            let t = if d > 1e-4 { (along * k as f32 / steps as f32) / d } else { 1.0 };
            let p = [at[0] + (wp[0] - at[0]) * t, at[1] + (wp[1] - at[1]) * t];
            let room = room_of(lot, p);
            if room != here && shut_to(ctx.shut, sim.id, mine, room) {
                return true;
            }
        }
        if d >= left {
            break;
        }
        left -= d;
        at = wp;
    }
    false
}

/// A walker's step outside a room kept shut (`world::progress`, before they move): true while
/// they wait there (`ticks` more), false to walk on. After `WAIT_MINUTES` they give up what
/// they were on their way to.
pub(crate) fn wait_outside(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], ticks: u32) -> bool {
    if !shut_ahead(sim, ctx, objects) {
        sim.kept_out = 0;
        return false;
    }
    sim.kept_out = sim.kept_out.saturating_add(ticks.max(1));
    if sim.kept_out as f32 * crate::MINUTES_PER_TICK < WAIT_MINUTES {
        return true;
    }
    sim.kept_out = 0;
    let Some(task) = sim.current().map(|a| a.task) else { return true };
    crate::world::end_activity(sim, ctx.content, objects);
    if !task.directed {
        let retry = crate::social::minutes_to_ticks(ctx.content.autonomy.retry_minutes);
        sim.remember_unreachable(task.kind, ctx.tick + retry);
    }
    true
}

/// Whether the content has somewhere to try for a baby (`Interaction::conceive`): then the
/// social goes on in bed, and needs one.
pub(crate) fn beds_in_content(content: &Content) -> bool {
    content.conceive_beds
}

/// Where partners `a` and `b` could try for a baby now: a working double bed at home with its
/// use for it (`Interaction::conceive`), nobody else on it, in a room indoors with a door,
/// kept shut by nobody and with nobody else in it (`others_in`: whether someone but the two,
/// and babies, is in a room); the nearest to `from`. None: nowhere.
pub(crate) fn bed_for(
    content: &Content,
    lot: &Lot,
    rooms: &[crate::rooms::RoomInfo],
    objects: &[ObjectInstance],
    on_home: &[u32],
    shut: &[Shut],
    (a, b): (u32, u32),
    from: [f32; 2],
    others_in: impl Fn(u16) -> bool,
) -> Option<(u32, usize)> {
    let mut best: Option<(f32, u32, usize)> = None;
    for &id in on_home {
        // (Free, or lain on by one of the two: a nap they got up from to be asked.)
        let Some(o) = objects.get(id as usize).filter(|o| !o.broken() && o.users().all(|u| u == a || u == b)) else { continue };
        let def = &content.objects[o.def];
        let Some(i) = def.interactions.iter().position(|i: &Interaction| i.conceive && o.allows(i, a)) else { continue };
        let room = room_of_object(content, lot, o);
        let walled = room != OUTDOORS && rooms.iter().any(|r| !r.garden && r.id == room && r.doors > 0);
        if !walled || shut.iter().any(|s| s.room == room && s.by != a && s.by != b) || others_in(room) {
            continue;
        }
        let c = o.centre(content);
        let d = (c[0] - from[0]).hypot(c[1] - from[1]);
        if best.is_none_or(|(e, ..)| d < e) {
            best = Some((d, id, i));
        }
    }
    best.map(|(_, o, i)| (o, i))
}

/// `bed_for` partners `a` and `b` as a tick's step sees them (free will choosing the social).
pub(crate) fn bed_in_step(ctx: &Ctx, objects: &[ObjectInstance], a: usize, b: usize) -> Option<(u32, usize)> {
    let home = ctx.households[ctx.briefs[a].household as usize].plot?;
    let lot = ctx.nav.lot;
    let others_in = |room: u16| ctx.briefs.iter().enumerate().any(|(j, s)| j != a && j != b && s.here && !s.away && !s.baby && room_of(lot, s.pos) == room);
    bed_for(ctx.content, lot, ctx.rooms, objects, ctx.index.on(home), ctx.shut, (a as u32, b as u32), ctx.briefs[a].pos, others_in)
}

/// `bed_for` partners `a` and `b` in `world` as it is now.
pub(crate) fn bed_in_world(w: &World, a: usize, b: usize) -> Option<(u32, usize)> {
    let home = w.households[w.sims[a].household as usize].plot?;
    let others_in = |room: u16| {
        w.sims.iter().enumerate().any(|(j, s)| {
            j != a && j != b && s.here() && s.away_until.is_none() && !w.content.life.baby(s.age) && room_of(&w.lot, s.pos) == room
        })
    };
    let on: Vec<u32> = (0..w.objects.len() as u32).filter(|&o| w.object_plot.get(o as usize).copied().flatten() == Some(home)).collect();
    bed_for(&w.content, &w.lot, &w.rooms, &w.objects, &on, &w.shut, (a as u32, b as u32), w.sims[a].pos, others_in)
}

/// After the step: partners whose *Try for a baby* went well (`Sim::to_bed`) go to bed together
/// (with the content's beds for it; without, a baby may come of the social itself, as it did
/// before beds had a use for it). With no bed free for them now, nothing comes of it.
pub(crate) fn after_step(w: &mut World) {
    for a in 0..w.sims.len() {
        let Some(b) = w.sims[a].to_bed.take() else { continue };
        let b = b as usize;
        if !beds_in_content(&w.content) {
            w.sims[a].conceived = Some(b as u32);
            continue;
        }
        let Some((object, interaction)) = bed_in_world(w, a, b) else { continue };
        let World { content, objects, sims, .. } = w;
        for s in [a, b] {
            let sim = &mut sims[s];
            if sim.current().is_some() {
                crate::world::end_activity(sim, content, objects);
            }
            sim.follow_up = None;
            sim.queue.push_front(Task { kind: TaskKind::Use { object, interaction }, directed: false, routine: false });
            sim.game = Some((object, interaction, false));
        }
    }
}
