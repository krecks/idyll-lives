//! Running social interactions between two Sims.
//!
//! An actor calls out the target's name when they're within earshot (`within_earshot`): the
//! target turns to them, and the two walk to meet halfway. Out of earshot (only on the player's
//! orders) the actor walks over until they are; a target too busy to come is walked up to (re-
//! routing if they move) and waited for until free. Then both face each other for the
//! interaction's duration.
//! Success is rolled when the conversation starts (so the UI can show the reaction
//! halfway through); relationship changes, feelings and events apply at the end.
//!
//! Others nearby may join in on a joinable social (a group conversation, up to `MAX_GROUP`
//! people): a `Join` task walks them over, finds them a place around the pair, and they listen,
//! filling their needs like the one talked to. When a social goes well, everyone there grows a
//! little closer. They stay for the leader's next social with someone there, and go otherwise.
//! Someone joining walks to their place in the group.
//!
//! People sitting stay sitting for a talk: the one talked to on the sofa or at the table keeps
//! their seat (and carries on with what they sat down to do afterwards), and people sitting near
//! each other talk, or listen in, from their seats (`from_seat`), without getting up. Only what
//! takes them closer than talking does (a hug, a kiss, a fight: `SEATED_DISTANCE`) gets them up.

use crate::{MINUTES_PER_TICK, TICKS_PER_SECOND};
use crate::content::{Content, MAX_NEEDS, Pose, stage_in};
use crate::path::{Grounds, NavGrid, STOREY_WALK};
use crate::rng::Rng;
use crate::social::{self, Effect, EventKind, EventLog, Relationships, SocialDef};
use crate::lot::Edge;
use crate::world::{Activity, ObjectInstance, Phase, Sim, Task, TaskKind, World, end_activity, grounds, route};

/// How long an actor waits for a busy target before giving up (in all: following them about
/// counts too, see `follow`).
const WAIT_LIMIT_TICKS: u32 = (10.0 / MINUTES_PER_TICK) as u32;
/// Re-route when the target is this far from the route's end. Routes end on a tile
/// *next to* the target (up to ~1.6 m from them), so this must exceed that.
const REPATH_DISTANCE: f32 = 2.4;
/// Extra distance at which a conversation can start.
const ENGAGE_SLACK: f32 = 0.8;
/// Rejected interactions end early.
pub(crate) const FAILURE_LENGTH: f32 = 0.4;
/// The most people in one conversation: the two talking and those listening in.
pub(crate) const MAX_GROUP: usize = 4;
/// How far from the middle of a conversation someone joining in can take their place in it.
const JOIN_RANGE: f32 = 2.6;
/// How long someone listening in waits between two of the leader's socials (for the next one
/// to start with someone there) before they go: long enough for the leader to choose again.
const REGROUP_TICKS: u32 = crate::TICKS_PER_SECOND * 3;
/// How close together the people in a group stand at the least.
const GROUP_SPACING: f32 = 0.6;
/// How far a call carries (metres of open floor; see `within_earshot`): across a room, or a
/// garden.
const EARSHOT: f32 = 12.0;
/// A doorway muffles a call as much as this much more open floor: the next room is within
/// earshot near the door.
const DOOR_MUFFLE: f32 = 6.0;
/// How long after someone calls out the one called hears it and turns to them.
const HEAR_TICKS: u32 = TICKS_PER_SECOND / 2;
/// How long someone calling out stands there before the two walk over to meet.
const CALL_TICKS: u32 = TICKS_PER_SECOND * 3 / 2;
/// How far apart two people sitting can talk from their seats: along a sofa, across a table,
/// from an armchair to the sofa beside it.
pub(crate) const SEAT_RANGE: f32 = 2.5;
/// Socials done this close or closer (a hug, a kiss, a fight) need them on their feet; a talk
/// (`SocialDef::distance` 1 m) doesn't.
pub(crate) const SEATED_DISTANCE: f32 = 0.9;
/// How often someone sitting thinks of a word with those sitting near them, and the chance
/// they do, each time, if anything they'd say comes to mind (`from_seat`).
pub(crate) const SEAT_TALK_TICKS: u32 = (5.0 / MINUTES_PER_TICK) as u32;
pub(crate) const SEAT_TALK_CHANCE: f32 = 0.5;
/// Someone joining in this close to their place in the group settles into it; further off,
/// they walk there (`take_place`).
const SETTLE: f32 = 0.3;

pub fn duration(s: &SocialDef, success: bool) -> f32 {
    if success {
        s.minutes
    } else {
        s.minutes * FAILURE_LENGTH
    }
}

/// Whether `actor` can start `s` with `target` (relationship requirements, both their life
/// stages and attraction).
pub fn social_allowed(
    content: &Content,
    rels: &Relationships,
    actor: &Sim,
    target: &Sim,
    s: &SocialDef,
) -> bool {
    let rel = rels.get(actor.id as usize, target.id as usize);
    let romantic = s.tags & content.social_rules.romantic_tags != 0;
    let grown_ups = actor.adult(content) && target.adult(content);
    let stage = |sim: &Sim| content.life.stage(sim.age);
    actor.id != target.id
        && s.requires.allows(rel)
        && stage_in(s.stages, stage(actor))
        && stage_in(s.target_stages, stage(target))
        && (!romantic || (grown_ups && actor.attracted_to(target) && rel.kin.is_none()))
}

/// Probability that `target` responds well to `actor` doing `s`.
pub fn acceptance_chance(
    content: &Content,
    rels: &Relationships,
    actor: &Sim,
    target: &Sim,
    s: &SocialDef,
) -> f32 {
    let romantic = s.tags & content.social_rules.romantic_tags != 0;
    if romantic && !target.attracted_to(actor) {
        return 0.0; // not interested in the actor's gender
    }
    accepts(content, rels, actor, target, &s.acceptance, s.tags)
}

/// Probability that `target` says yes to `actor` (terms `acc`, for something with `tags`): how
/// the target feels about the actor, their chemistry, the target's mood, traits and emotion.
/// Socials use it, and so do phone calls (`phone.rs`).
pub(crate) fn accepts(
    content: &Content,
    rels: &Relationships,
    actor: &Sim,
    target: &Sim,
    acc: &social::Acceptance,
    tags: crate::content::TagMask,
) -> f32 {
    let (a, t) = (actor.id as usize, target.id as usize);
    let view = rels.get(t, a);
    let mut p = acc.base
        + acc.friendship * view.friendship
        + acc.romance * view.romance
        + acc.chemistry * rels.chemistry(a, t)
        + acc.mood * (target.mood(content) - 0.5)
        + target.mods.acceptance(tags)
        + actor.mods.success(tags);
    if let Some(e) = target.emotion(content) {
        p += content.emotions[e].mods.acceptance(tags);
    }
    if acc.taken_penalty > 0.0 && rels.partner_of(t).is_some_and(|p| p != a) {
        p -= acc.taken_penalty;
    }
    p.clamp(0.0, 1.0)
}

/// Social options `actor` has toward `target`: `(social index, success chance)`. (Goodbyes at
/// the end of a visit, `visits.farewell`, aren't on offer: guests say them as they leave.)
pub fn options(w: &World, actor: usize, target: usize) -> Vec<(usize, f32)> {
    let (Some(a), Some(t)) = (w.sims.get(actor), w.sims.get(target)) else {
        return Vec::new();
    };
    w.content
        .socials
        .iter()
        .enumerate()
        .filter(|(i, s)| !w.content.visits.farewell.contains(i) && social_allowed(&w.content, &w.relationships, a, t, s))
        .map(|(i, s)| (i, acceptance_chance(&w.content, &w.relationships, a, t, s)))
        .collect()
}

/// A free tile next to `target` (or under it), closest to `from`.
pub(crate) fn approach_tile(nav: &NavGrid, target: [f32; 2], from: [f32; 2]) -> Option<(i32, i32)> {
    let (tx, tz) = (target[0].floor() as i32, target[1].floor() as i32);
    let mut best = None;
    let mut best_d = f32::INFINITY;
    for dz in -1..=1 {
        for dx in -1..=1 {
            let (x, z) = (tx + dx, tz + dz);
            if !nav.tile_free(x, z) {
                continue;
            }
            let d = (x as f32 + 0.5 - from[0]).hypot(z as f32 + 0.5 - from[1]);
            if d < best_d {
                best_d = d;
                best = Some((x, z));
            }
        }
    }
    best
}

/// Whether someone at `b` hears a call from `a`: within `EARSHOT` of open floor (round walls and
/// furniture-free of them: furniture doesn't stop a voice), a doorway counting `DOOR_MUFFLE`
/// more and a flight of stairs `STOREY_WALK`. Walls and windows stop it; garden fences don't.
pub(crate) fn within_earshot(nav: &NavGrid, a: [f32; 2], b: [f32; 2]) -> bool {
    if nav.stairs.distance(a, b) > EARSHOT {
        return false;
    }
    let lot = nav.lot;
    let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
    let (start, goal) = (tile(a), tile(b));
    if start == goal {
        return true;
    }
    // Tenths of a metre, as whole numbers for the queue.
    let tenths = |m: f32| (m * 10.0).round() as u32;
    let (limit, muffle) = (tenths(EARSHOT), tenths(DOOR_MUFFLE));
    let storey = |z: i32| nav.stairs.storey(z);
    // Sound crosses the edge between neighbouring tiles at this extra cost, or not at all.
    let open = |(ax, az): (i32, i32), (bx, bz): (i32, i32)| -> Option<u32> {
        if !lot.in_bounds(bx, bz) || lot.diag_blocks(bx, bz) || storey(az) != storey(bz) {
            return None;
        }
        match lot.edge_between(ax, az, bx, bz) {
            Edge::Wall | Edge::Window => None,
            Edge::Door => Some(muffle),
            Edge::Open | Edge::Fence | Edge::Gate => Some(0),
        }
    };
    let mut best = std::collections::HashMap::from([(start, 0u32)]);
    let mut queue = std::collections::BinaryHeap::from([std::cmp::Reverse((0u32, start))]);
    while let Some(std::cmp::Reverse((cost, at))) = queue.pop() {
        if at == goal {
            return true;
        }
        if best.get(&at).is_some_and(|&c| c < cost) {
            continue;
        }
        let mut next: Vec<((i32, i32), u32)> = Vec::with_capacity(9);
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let to = (at.0 + dx, at.1 + dz);
            if let Some(extra) = open(at, to) {
                next.push((to, 10 + extra));
            }
        }
        // Diagonally across open floor (both ways round the corner open, no doorway).
        for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let (side_x, side_z) = ((at.0 + dx, at.1), (at.0, at.1 + dz));
            let to = (at.0 + dx, at.1 + dz);
            let clear = [open(at, side_x), open(side_x, to), open(at, side_z), open(side_z, to)].iter().all(|e| *e == Some(0));
            if clear {
                next.push((to, 14));
            }
        }
        // Up or down the stairs.
        if let Some(to) = nav.stairs.link(at.0, at.1) {
            next.push((to, tenths(STOREY_WALK)));
        }
        for (to, step) in next {
            let c = cost + step;
            if c <= limit && best.get(&to).is_none_or(|&b| c < b) {
                best.insert(to, c);
                queue.push(std::cmp::Reverse((c, to)));
            }
        }
    }
    false
}

/// Whether someone at `from` would call out to start `s` with someone at `to`: within earshot,
/// but not near enough to just start talking.
pub(crate) fn calls_out(nav: &NavGrid, s: &SocialDef, from: [f32; 2], to: [f32; 2]) -> bool {
    dist(from, to) > s.distance + ENGAGE_SLACK && within_earshot(nav, from, to)
}

#[derive(Clone, Copy)]
enum State {
    Routing { goal: [f32; 2] },
    Calling(u32),
    Waiting(u32),
    Conversing(f32, bool),
}

/// Where someone joining in is at.
#[derive(Clone, Copy)]
enum Joining {
    Routing { goal: [f32; 2] },
    /// Arrived, or between two of the leader's socials.
    Waiting(u32),
    Listening(f32),
}

pub(crate) fn update(w: &mut World) {
    let tick = w.tick;
    let World {
        content,
        lot,
        blocked,
        objects,
        sims,
        relationships,
        events,
        rng,
        stairs,
        households,
        plots,
        tile_plot,
        reach,
        ..
    } = w;
    let nav = NavGrid { lot, blocked, stairs };
    // (Round other people's gardens, as for any walk.)
    let walk = |sim: &Sim, goal| grounds(tile_plot, plots, households, &nav, sim, goal);
    let n = sims.len();

    // Drop engagements whose actor stopped talking (cancelled, interrupted, finished).
    for i in 0..n {
        if let Some(a) = sims[i].engaged_with
            && sims[a as usize]
                .conversation(content)
                .is_none_or(|(t, ..)| t as usize != i)
        {
            sims[i].engaged_with = None;
        }
        // And whoever called them stopped coming.
        if let Some(a) = sims[i].heeding
            && !matches!(sims[a as usize].current.as_ref().map(|c| c.task.kind), Some(TaskKind::Social { target, .. }) if target as usize == i)
        {
            sims[i].heeding = None;
        }
    }

    for i in 0..n {
        let Some(act) = sims[i].current.as_ref() else {
            continue;
        };
        let TaskKind::Social { target, social } = act.task.kind else {
            continue;
        };
        let t = target as usize;
        let state = match &act.phase {
            Phase::Routing { waypoints, .. } => State::Routing {
                goal: *waypoints.last().unwrap_or(&sims[i].pos),
            },
            Phase::Calling { ticks } => State::Calling(*ticks),
            Phase::Waiting { ticks } => State::Waiting(*ticks),
            Phase::Conversing { elapsed, success } => State::Conversing(*elapsed, *success),
            Phase::Using { .. } | Phase::Knocking { .. } | Phase::Queued { .. } => continue,
        };
        let s = &content.socials[social];
        let gap = dist(sims[i].pos, sims[t].pos);
        let directed = act.task.directed;
        // The one they called is coming over to meet them (`walking`), or waits for them there,
        // turned to them.
        let coming = sims[t].heeding == Some(i as u32);
        let walking = coming && sims[t].current.is_some();
        if coming && !walking {
            sims[t].yaw = facing(sims[t].pos, sims[i].pos);
        }

        match state {
            State::Routing { goal } => {
                // (Following the target after getting there counts as waiting for them.)
                if sims[i].social_waited > 0 {
                    sims[i].social_waited += 1;
                }
                if gap <= s.distance + ENGAGE_SLACK {
                    set_phase(&mut sims[i], Phase::Waiting { ticks: 0 });
                } else if !sims[i].called && sims[t].available_for_social(content) && calls_out(&nav, s, sims[i].pos, sims[t].pos) {
                    // Near enough to call out to them (on the player's orders, from across the house).
                    sims[i].called = true;
                    set_phase(&mut sims[i], Phase::Calling { ticks: 0 });
                } else if walking {
                    // (On their way over.)
                } else if dist(goal, sims[t].pos) > REPATH_DISTANCE {
                    if coming {
                        chase(&nav, reach, &walk, sims, i, t, content, objects);
                    } else {
                        follow(&nav, reach, &walk, sims, i, t, directed, content, objects);
                    }
                }
            }
            State::Calling(ticks) => {
                sims[i].yaw = facing(sims[i].pos, sims[t].pos);
                if sims[i].pressed(content) {
                    end_activity(&mut sims[i], content, objects);
                } else if ticks == HEAR_TICKS {
                    // They hear their name: whatever they were doing (on their own), they turn to
                    // the one calling. Too busy to (in the bathroom, already talking): over to
                    // them to wait, as for anyone busy. Sitting down, they stay sitting, and the
                    // one calling comes over.
                    if sims[t].available_for_social(content) && keeps_seat(&sims[t], s) {
                        set_phase(&mut sims[i], Phase::Calling { ticks: ticks + 1 });
                    } else if sims[t].available_for_social(content) {
                        end_activity(&mut sims[t], content, objects);
                        sims[t].heeding = Some(i as u32);
                        sims[t].yaw = facing(sims[t].pos, sims[i].pos);
                        set_phase(&mut sims[i], Phase::Calling { ticks: ticks + 1 });
                    } else {
                        chase(&nav, reach, &walk, sims, i, t, content, objects);
                    }
                } else if ticks > HEAR_TICKS && !coming {
                    // (Called away since: the player's orders.)
                    follow(&nav, reach, &walk, sims, i, t, directed, content, objects);
                } else if ticks >= CALL_TICKS {
                    meet(&nav, reach, &walk, sims, i, t, content, objects);
                } else {
                    set_phase(&mut sims[i], Phase::Calling { ticks: ticks + 1 });
                }
            }
            State::Waiting(ticks) => {
                sims[i].social_waited += 1;
                // About to drop (or burst) themselves: the talk would break off at once.
                if sims[i].pressed(content) {
                    end_activity(&mut sims[i], content, objects);
                } else if sims[i].paused.is_some() {
                    // From the seat: with them sitting there, or not at all (back to what they sat
                    // down to do).
                    if gap <= SEAT_RANGE + 0.3 && sims[t].available_for_social(content) {
                        engage(content, &nav, relationships, events, rng, objects, sims, i, t, social, tick);
                    } else {
                        end_activity(&mut sims[i], content, objects);
                    }
                } else if walking && gap > s.distance + ENGAGE_SLACK {
                    // Got to the middle first: wait for them to get there too.
                    sims[i].yaw = facing(sims[i].pos, sims[t].pos);
                    set_phase(&mut sims[i], Phase::Waiting { ticks: ticks + 1 });
                } else if gap > s.distance + ENGAGE_SLACK + 0.6 {
                    if coming {
                        chase(&nav, reach, &walk, sims, i, t, content, objects);
                    } else {
                        follow(&nav, reach, &walk, sims, i, t, directed, content, objects);
                    }
                } else if coming || sims[t].available_for_social(content) {
                    engage(
                        content,
                        &nav,
                        relationships,
                        events,
                        rng,
                        objects,
                        sims,
                        i,
                        t,
                        social,
                        tick,
                    );
                } else if sims[i].social_waited >= WAIT_LIMIT_TICKS {
                    end_activity(&mut sims[i], content, objects);
                } else {
                    sims[i].yaw = facing(sims[i].pos, sims[t].pos);
                    set_phase(&mut sims[i], Phase::Waiting { ticks: ticks + 1 });
                }
            }
            State::Conversing(elapsed, success) => {
                // (Either of them bursting breaks it off.)
                if sims[t].engaged_with != Some(i as u32) || sims[i].pressed(content) || sims[t].pressed(content) {
                    if sims[t].engaged_with == Some(i as u32) {
                        sims[t].engaged_with = None;
                    }
                    end_activity(&mut sims[i], content, objects);
                    continue;
                }
                let elapsed = elapsed + MINUTES_PER_TICK;
                let (actor, other) = pair_mut(sims, i, t);
                gain(actor, &s.actor_gain, s.minutes);
                gain(other, &s.target_gain, s.minutes);
                if elapsed >= duration(s, success) {
                    if success && s.joinable {
                        group_outcome(content, relationships, events, sims, i, s, tick);
                    }
                    apply_outcome(
                        content,
                        relationships,
                        events,
                        rng,
                        sims,
                        i,
                        t,
                        social,
                        success,
                        tick,
                    );
                    sims[t].engaged_with = None;
                    end_activity(&mut sims[i], content, objects);
                } else {
                    open_up(sims, content, i, t);
                    set_phase(&mut sims[i], Phase::Conversing { elapsed, success });
                }
            }
        }
    }

    // Joining in and listening (after the socials, so they see this tick's endings).
    for j in 0..n {
        let Some(act) = sims[j].current.as_ref() else {
            continue;
        };
        let TaskKind::Join { leader } = act.task.kind else {
            continue;
        };
        let l = leader as usize;
        let state = match &act.phase {
            Phase::Routing { waypoints, .. } => Joining::Routing { goal: *waypoints.last().unwrap_or(&sims[j].pos) },
            Phase::Waiting { ticks } => Joining::Waiting(*ticks),
            Phase::Conversing { elapsed, .. } => Joining::Listening(*elapsed),
            Phase::Using { .. } | Phase::Calling { .. } | Phase::Knocking { .. } | Phase::Queued { .. } => continue,
        };
        if l == j || sims.get(l).is_none_or(|s| !s.here()) || sims[j].pressed(content) {
            end_activity(&mut sims[j], content, objects);
            continue;
        }
        // The conversation they're there for: the leader's, if it's one to join in on.
        let talk = sims[l]
            .conversation(content)
            .filter(|&(t, social, ..)| t as usize != j && content.socials[social].joinable)
            .map(|(t, social, ..)| (t as usize, social));
        match (state, talk) {
            (Joining::Routing { goal }, Some((t, _))) => {
                if sims[j].join_spot.is_some() {
                    // (On their way to their place in the group: they take it on getting there.)
                } else if dist(sims[j].pos, middle(sims, l, t)) <= JOIN_RANGE {
                    set_phase(&mut sims[j], Phase::Waiting { ticks: 0 });
                } else if dist(goal, sims[l].pos) > REPATH_DISTANCE {
                    chase(&nav, reach, &walk, sims, j, l, content, objects);
                }
            }
            (Joining::Routing { .. }, None) => {
                if !regrouping(sims, l, j) {
                    end_activity(&mut sims[j], content, objects);
                }
            }
            (Joining::Waiting(_), Some((t, _))) => {
                // (Listening in from their seat: only to a talk beside them.)
                let seated = sims[j].paused.is_some();
                let off = dist(sims[j].pos, middle(sims, l, t));
                if seated && off > SEAT_RANGE + 0.6 {
                    end_activity(&mut sims[j], content, objects);
                } else if off > JOIN_RANGE + 0.6 {
                    chase(&nav, reach, &walk, sims, j, l, content, objects);
                } else if group(sims, content, l).len() >= MAX_GROUP {
                    end_activity(&mut sims[j], content, objects);
                } else {
                    take_place(&nav, reach, &walk, relationships, events, sims, content, j, l, t, tick);
                }
            }
            (Joining::Waiting(ticks), None) => {
                if ticks < REGROUP_TICKS && regrouping(sims, l, j) {
                    set_phase(&mut sims[j], Phase::Waiting { ticks: ticks + 1 });
                } else {
                    end_activity(&mut sims[j], content, objects);
                }
            }
            (Joining::Listening(elapsed), Some((_, social))) => {
                let s = &content.socials[social];
                gain(&mut sims[j], &s.target_gain, s.minutes);
                set_phase(&mut sims[j], Phase::Conversing { elapsed: elapsed + MINUTES_PER_TICK, success: true });
            }
            (Joining::Listening(_), None) => set_phase(&mut sims[j], Phase::Waiting { ticks: 0 }),
        }
    }
}

/// Whether `sim`, talked to with `s`, stays in their seat: sitting at something they carry on
/// with afterwards (the sofa, a coffee at the table; or talking from it themselves), for a
/// social they needn't get up for.
fn keeps_seat(sim: &Sim, s: &SocialDef) -> bool {
    let sitting = sim.paused.is_some() || sim.current.as_ref().is_some_and(|a| matches!((a.task.kind, &a.phase), (TaskKind::Use { .. }, Phase::Using { .. })));
    sim.pose == Pose::Sit && sitting && s.distance >= SEATED_DISTANCE
}

/// Someone sitting talks to someone sitting near (`task`, a social), or listens in on a talk
/// beside them (a `Join`), from their seat (`World::step_sim`'s `pick_autonomous` with
/// `from_seat`): what they sat down to do waits (`Sim::paused`) until it's over.
pub(crate) fn from_seat(sim: &mut Sim, task: Task, content: &Content, briefs: &[crate::world::SimBrief]) {
    let tags = match task.kind {
        TaskKind::Social { social, .. } => content.socials[social].tags,
        TaskKind::Join { leader } => briefs.get(leader as usize).and_then(|b| b.conversing).map_or(0, |(_, social)| content.socials[social].tags),
        _ => return,
    };
    sim.paused = sim.current.take();
    // (Right there: no calling out, no walking over.)
    sim.called = true;
    sim.social_waited = 0;
    sim.current = Some(Activity { task, phase: Phase::Waiting { ticks: 0 }, tags });
}

/// Everyone in the conversation `l` leads: `l`, the one talked to, then those listening in.
pub(crate) fn group(sims: &[Sim], content: &Content, l: usize) -> Vec<usize> {
    let Some((t, ..)) = sims.get(l).and_then(|s| s.conversation(content)) else {
        return Vec::new();
    };
    let mut members = vec![l, t as usize];
    members.extend((0..sims.len()).filter(|&j| sims[j].listening_to() == Some(l as u32)));
    members
}

fn middle(sims: &[Sim], a: usize, b: usize) -> [f32; 2] {
    let (p, q) = (sims[a].pos, sims[b].pos);
    [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0]
}

/// Whether the leader `l` is between socials, about to start the next one with someone near
/// `j` (or choosing what to do next), so it's worth `j` staying.
fn regrouping(sims: &[Sim], l: usize, j: usize) -> bool {
    let s = &sims[l];
    if s.away_until.is_some() || s.pose == Pose::Lie {
        return false;
    }
    match s.current.as_ref() {
        None => true,
        Some(a) => match (a.task.kind, &a.phase) {
            (_, Phase::Conversing { .. }) => false,
            (TaskKind::Social { target, .. }, _) => sims.get(target as usize).is_some_and(|t| dist(t.pos, sims[j].pos) <= JOIN_RANGE * 2.0),
            _ => false,
        },
    }
}

/// `j` joins in on the conversation `l` is having with `t`: a free spot around the two (to one
/// side of them first, then all round), walked to (`join_spot`) unless they're already there;
/// there, everyone faces the middle of the group. Sitting (listening in from their seat), they
/// stay where they are.
#[allow(clippy::too_many_arguments)]
fn take_place<'a>(
    nav: &NavGrid,
    reach: &crate::path::Reach,
    walk: &impl Fn(&Sim, (i32, i32)) -> Grounds<'a>,
    rels: &mut Relationships,
    events: &mut EventLog,
    sims: &mut [Sim],
    content: &Content,
    j: usize,
    l: usize,
    t: usize,
    tick: u64,
) {
    let others: Vec<usize> = group(sims, content, l).into_iter().filter(|&m| m != j).collect();
    let (a, b) = (sims[l].pos, sims[t].pos);
    let mid = middle(sims, l, t);
    let d = dist(a, b).max(0.5);
    let across = [-(b[1] - a[1]) / d, (b[0] - a[0]) / d];
    // (Beside the pair, the three of them in a triangle; then round them, a little further out.)
    let side = (d * 0.87).max(0.8);
    let round = (d * 0.5 + 0.5).max(side);
    let from = sims[j].pos;
    let by_distance = |p: &[f32; 2], q: &[f32; 2]| dist(*p, from).total_cmp(&dist(*q, from));
    let mut spots: Vec<[f32; 2]> = [1.0, -1.0].iter().map(|k| [mid[0] + across[0] * side * k, mid[1] + across[1] * side * k]).collect();
    spots.sort_by(by_distance);
    let mut ring: Vec<[f32; 2]> = (0..12)
        .map(|k| {
            let angle = k as f32 * std::f32::consts::PI / 6.0;
            [mid[0] + round * angle.sin(), mid[1] + round * angle.cos()]
        })
        .collect();
    ring.sort_by(by_distance);
    spots.extend(ring);
    let spot = spots.into_iter().find(|&p| {
        nav.tile_free(p[0].floor() as i32, p[1].floor() as i32)
            && nav.line_walkable(from, p)
            && others.iter().all(|&m| dist(sims[m].pos, p) >= GROUP_SPACING)
    });
    // (Nowhere free, or sitting: they listen from where they are.)
    let spot = spot.filter(|_| sims[j].pose == Pose::Stand);
    if let Some(p) = spot {
        if dist(from, p) > SETTLE {
            // Over to it on foot; they take it on getting there (`update`, again here).
            let tile = (p[0].floor() as i32, p[1].floor() as i32);
            if let Some(waypoints) = route(nav, Some(reach), from, sims[j].tile(), tile, p, Some(&walk(&sims[j], tile))) {
                sims[j].join_spot = Some(p);
                return set_phase(&mut sims[j], Phase::Routing { waypoints, next: 0 });
            }
        }
        sims[j].pos = p;
    }
    sims[j].join_spot = None;
    for &m in &others {
        if !rels.get(j, m).met {
            rels.get_mut(j, m).met = true;
            rels.get_mut(m, j).met = true;
            events.push(tick, EventKind::Met, j, m, None);
        }
    }
    set_phase(&mut sims[j], Phase::Conversing { elapsed: 0.0, success: true });
    open_up(sims, content, l, t);
}

/// The two talking (`l` and `t`) face each other, or, with others listening in, open up to the
/// group: everyone there faces its middle (where they all are, on average), so nobody has their
/// back turned. Called as people join and every tick of the conversation, so it follows people
/// leaving too.
fn open_up(sims: &mut [Sim], content: &Content, l: usize, t: usize) {
    let members = group(sims, content, l);
    // (Those sitting stay turned the way their seat faces: their heads turn, `characters.ts`.)
    let standing = |sims: &[Sim], m: usize| sims[m].pose == Pose::Stand;
    if members.len() <= 2 {
        if standing(sims, l) {
            sims[l].yaw = facing(sims[l].pos, sims[t].pos);
        }
        if standing(sims, t) {
            sims[t].yaw = facing(sims[t].pos, sims[l].pos);
        }
        return;
    }
    let count = members.len() as f32;
    let sum = members.iter().fold([0.0, 0.0], |c, &m| [c[0] + sims[m].pos[0], c[1] + sims[m].pos[1]]);
    let centre = [sum[0] / count, sum[1] / count];
    for m in members {
        // (Someone right on the middle, as can't happen with everyone apart: they stay as they are.)
        if standing(sims, m) && dist(sims[m].pos, centre) > 0.05 {
            sims[m].yaw = facing(sims[m].pos, centre);
        }
    }
}

/// A joinable social `s` led by `l` went well: those listening in get the feeling the one talked
/// to does, and everyone there grows a little closer to everyone else (the pair talking to each
/// other through their social's outcome).
fn group_outcome(content: &Content, rels: &mut Relationships, events: &mut EventLog, sims: &mut [Sim], l: usize, s: &SocialDef, tick: u64) {
    let members = group(sims, content, l);
    if members.len() <= 2 {
        return;
    }
    let f = content.social_rules.join_friendship;
    for (x, &a) in members.iter().enumerate() {
        for &b in &members[(x + 1).max(2)..] {
            let before = (*rels.get(a, b), *rels.get(b, a));
            rels.adjust(a, b, f, 0.0);
            rels.adjust(b, a, f, 0.0);
            for (kind, reversed) in social::milestone_events((&before.0, &before.1), (rels.get(a, b), rels.get(b, a))) {
                let (p, q) = if reversed { (b, a) } else { (a, b) };
                events.push(tick, kind, p, q, None);
            }
        }
    }
    if let Some(m) = s.success.target_feeling {
        for &j in &members[2..] {
            social::add_feeling(&mut sims[j].feelings, m, &content.feelings, tick);
        }
    }
}

fn set_phase(sim: &mut Sim, phase: Phase) {
    if let Some(act) = sim.current.as_mut() {
        act.phase = phase;
    }
}

/// The target of `i`'s social moved off: after them (`chase`) while they're someone to talk to
/// and somewhere `i` would chat (not into someone else's house or garden, as when choosing; the
/// player's orders go anywhere). Off home or to work, sitting down to eat, busy with what they
/// were told to do: `i` gives up rather than follow them across town.
#[allow(clippy::too_many_arguments)]
fn follow<'a>(
    nav: &NavGrid,
    reach: &crate::path::Reach,
    walk: &impl Fn(&Sim, (i32, i32)) -> Grounds<'a>,
    sims: &mut [Sim],
    i: usize,
    t: usize,
    directed: bool,
    content: &Content,
    objects: &mut [ObjectInstance],
) {
    let (x, z) = sims[t].tile();
    let open = walk(&sims[i], sims[i].tile());
    let allowed = directed || open.plot(nav, x, z).is_none_or(|p| open.open.get(p as usize).copied().unwrap_or(true));
    if allowed && sims[t].available_for_social(content) {
        chase(nav, reach, walk, sims, i, t, content, objects);
    } else {
        end_activity(&mut sims[i], content, objects);
    }
}

/// Re-route toward a target that moved; give up if they're unreachable.
#[allow(clippy::too_many_arguments)]
fn chase<'a>(
    nav: &NavGrid,
    reach: &crate::path::Reach,
    walk: &impl Fn(&Sim, (i32, i32)) -> Grounds<'a>,
    sims: &mut [Sim],
    i: usize,
    t: usize,
    content: &Content,
    objects: &mut [ObjectInstance],
) {
    let target = sims[t].pos;
    let rerouted = approach_tile(nav, target, sims[i].pos).and_then(|goal| {
        route(
            nav,
            Some(reach),
            sims[i].pos,
            sims[i].tile(),
            goal,
            [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5],
            Some(&walk(&sims[i], goal)),
        )
    });
    match rerouted {
        Some(waypoints) => set_phase(&mut sims[i], Phase::Routing { waypoints, next: 0 }),
        None => end_activity(&mut sims[i], content, objects),
    }
}

/// The one `i` called turned to them: both walk to meet halfway along the way between them,
/// `i` to the middle of it and `t` to a free tile next to that. No way to the middle for `t`:
/// they wait where they are and `i` walks all the way.
#[allow(clippy::too_many_arguments)]
fn meet<'a>(
    nav: &NavGrid,
    reach: &crate::path::Reach,
    walk: &impl Fn(&Sim, (i32, i32)) -> Grounds<'a>,
    sims: &mut [Sim],
    i: usize,
    t: usize,
    content: &Content,
    objects: &mut [ObjectInstance],
) {
    let centre = |(x, z): (i32, i32)| [x as f32 + 0.5, z as f32 + 0.5];
    let way = |sims: &[Sim], who: usize, to: [f32; 2]| {
        approach_tile(nav, to, sims[who].pos).and_then(|goal| {
            route(nav, Some(reach), sims[who].pos, sims[who].tile(), goal, centre(goal), Some(&walk(&sims[who], goal)))
        })
    };
    let Some(all) = way(sims, i, sims[t].pos) else {
        sims[t].heeding = None;
        return end_activity(&mut sims[i], content, objects);
    };
    let (mut mine, middle) = halfway(sims[i].pos, &all);
    match way(sims, t, middle) {
        Some(waypoints) => {
            let goal = *waypoints.last().unwrap_or(&sims[t].pos);
            let task = Task { kind: TaskKind::MoveTo { x: goal[0], z: goal[1] }, directed: false, routine: false };
            sims[t].current = Some(Activity { task, phase: Phase::Routing { waypoints, next: 0 }, tags: 0 });
        }
        None => mine = all,
    }
    set_phase(&mut sims[i], Phase::Routing { waypoints: mine, next: 0 });
}

/// The waypoints from `from` along `path` to its middle, and the middle.
fn halfway(from: [f32; 2], path: &[[f32; 2]]) -> (Vec<[f32; 2]>, [f32; 2]) {
    let legs = || std::iter::once(from).chain(path.iter().copied()).zip(path.iter().copied());
    let mut left = legs().map(|(a, b)| dist(a, b)).sum::<f32>() / 2.0;
    let mut out = Vec::new();
    for (a, b) in legs() {
        let d = dist(a, b);
        if d >= left {
            let k = if d > 0.0 { left / d } else { 0.0 };
            let middle = [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k];
            out.push(middle);
            return (out, middle);
        }
        left -= d;
        out.push(b);
    }
    let end = *path.last().unwrap_or(&from);
    (out, end)
}

#[allow(clippy::too_many_arguments)]
fn engage(
    content: &Content,
    nav: &NavGrid,
    rels: &mut Relationships,
    events: &mut EventLog,
    rng: &mut Rng,
    objects: &mut [ObjectInstance],
    sims: &mut [Sim],
    i: usize,
    t: usize,
    social: usize,
    tick: u64,
) {
    let s = &content.socials[social];
    let success = rng.next_f32() < acceptance_chance(content, rels, &sims[i], &sims[t], s);
    // The target drops whatever (autonomous, interruptible) thing it was doing (walking over to
    // meet them, if they called); sitting down, they stay sitting, turned the way they sit, and
    // carry on afterwards (what they're doing waits while they talk: `step_sim`).
    let seated = keeps_seat(&sims[t], s);
    // (Talking to someone themselves from their seat: they're back at what they sat down to do.)
    if !seated || sims[t].paused.is_some() {
        end_activity(&mut sims[t], content, objects);
    }
    sims[t].heeding = None;
    sims[t].engaged_with = Some(i as u32);

    if !rels.get(i, t).met {
        rels.get_mut(i, t).met = true;
        rels.get_mut(t, i).met = true;
        events.push(tick, EventKind::Met, i, t, None);
    }

    // Stand at the interaction's distance, facing each other.
    let (ap, tp) = (sims[i].pos, sims[t].pos);
    let d = dist(ap, tp);
    let at = |dir: [f32; 2]| [tp[0] + dir[0] * s.distance, tp[1] + dir[1] * s.distance];
    let open = |spot: [f32; 2]| nav.tile_free(spot[0].floor() as i32, spot[1].floor() as i32) && nav.line_walkable(ap, spot);
    let spot = if d > 0.05 {
        Some(at([(ap[0] - tp[0]) / d, (ap[1] - tp[1]) / d])).filter(|&p| open(p))
    } else {
        // On top of each other (someone answering the door from right where the visitor
        // stands): the actor steps back, or to one side if there's no room behind them.
        let back = sims[i].yaw + std::f32::consts::PI;
        [0.0f32, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0]
            .iter()
            .map(|k| back + k * std::f32::consts::FRAC_PI_4)
            .map(|a| at([a.sin(), a.cos()]))
            .find(|&p| open(p))
    };
    // (Talking from their seat, the actor stays on it, turned the way it faces.)
    if sims[i].pose != Pose::Sit {
        if let Some(spot) = spot {
            sims[i].pos = spot;
        }
        sims[i].yaw = facing(sims[i].pos, tp);
    }
    if !seated {
        sims[t].yaw = facing(tp, sims[i].pos);
    }
    set_phase(
        &mut sims[i],
        Phase::Conversing {
            elapsed: 0.0,
            success,
        },
    );
}

#[allow(clippy::too_many_arguments)]
fn apply_outcome(
    content: &Content,
    rels: &mut Relationships,
    events: &mut EventLog,
    rng: &mut Rng,
    sims: &mut [Sim],
    a: usize,
    t: usize,
    social: usize,
    success: bool,
    tick: u64,
) {
    let s = &content.socials[social];
    let o = if success { &s.success } else { &s.failure };
    outcome(content, rels, events, rng, sims, a, t, o, s.tags, success, tick);
}

/// What came of something `a` did with `t` (a social, a phone call): relationship changes both
/// ways, needs, feelings, the outcome's effect, milestones and jealousy.
#[allow(clippy::too_many_arguments)]
pub(crate) fn outcome(
    content: &Content,
    rels: &mut Relationships,
    events: &mut EventLog,
    rng: &mut Rng,
    sims: &mut [Sim],
    a: usize,
    t: usize,
    o: &social::Outcome,
    tags: crate::content::TagMask,
    success: bool,
    tick: u64,
) {
    let defs = &content.feelings;
    let rules = &content.social_rules;
    let before = (*rels.get(a, t), *rels.get(t, a));

    rels.adjust(a, t, o.friendship, o.romance);
    rels.adjust(t, a, o.target_friendship, o.target_romance);
    {
        let (actor, target) = pair_mut(sims, a, t);
        nudge(actor, &o.needs);
        nudge(target, &o.target_needs);
        if let Some(m) = o.feeling {
            social::add_feeling(&mut actor.feelings, m, defs, tick);
        }
        if let Some(m) = o.target_feeling {
            social::add_feeling(&mut target.feelings, m, defs, tick);
        }
    }

    match o.effect {
        Effect::None => {}
        Effect::Conceive => {
            if success {
                sims[a].conceived = Some(t as u32);
            }
        }
        Effect::Kiss => {
            if success && !rels.get(a, t).kissed {
                rels.get_mut(a, t).kissed = true;
                rels.get_mut(t, a).kissed = true;
                events.push(tick, EventKind::FirstKiss, a, t, None);
            }
        }
        Effect::BecomePartners => {
            if success {
                // Starting something new ends any existing relationship, painfully for the ex.
                for x in [a, t] {
                    if let Some(ex) = rels.partner_of(x).filter(|&p| p != a && p != t) {
                        break_up(
                            rels,
                            events,
                            sims,
                            defs,
                            rules.heartbreak_feeling,
                            x,
                            ex,
                            tick,
                        );
                    }
                }
                rels.set_partners(a, t, true);
                events.push(tick, EventKind::StartedDating, a, t, None);
            } else {
                events.push(tick, EventKind::ProposalRejected, a, t, None);
            }
        }
        Effect::BreakUp => {
            if rels.get(a, t).partners {
                break_up(
                    rels,
                    events,
                    sims,
                    defs,
                    rules.heartbreak_feeling,
                    a,
                    t,
                    tick,
                );
            }
        }
        Effect::Fight => {
            let strength = |sim: &Sim| 0.3 + sim.mood(content) * 0.5 + sim.mods.walk_speed * 0.2;
            let (sa, st) = (strength(&sims[a]), strength(&sims[t]));
            let (winner, loser) = if rng.next_f32() * (sa + st) < sa {
                (a, t)
            } else {
                (t, a)
            };
            if let Some(m) = o.winner_feeling {
                social::add_feeling(&mut sims[winner].feelings, m, defs, tick);
            }
            if let Some(m) = o.loser_feeling {
                social::add_feeling(&mut sims[loser].feelings, m, defs, tick);
            }
            events.push(tick, EventKind::Fight, winner, loser, None);
        }
    }

    for (kind, reversed) in
        social::milestone_events((&before.0, &before.1), (rels.get(a, t), rels.get(t, a)))
    {
        let (x, y) = if reversed { (t, a) } else { (a, t) };
        events.push(tick, kind, x, y, None);
    }

    // Romance in front of a partner causes jealousy.
    if success && tags & rules.romantic_tags != 0 {
        for b in 0..sims.len() {
            if b == a || b == t || sims[b].pose == Pose::Lie {
                continue;
            }
            for (x, y) in [(a, t), (t, a)] {
                if rels.get(b, x).partners && dist(sims[b].pos, sims[x].pos) <= rules.jealousy_range
                {
                    rels.adjust(b, x, rules.jealousy_friendship, rules.jealousy_romance);
                    if let Some(m) = rules.jealousy_feeling {
                        social::add_feeling(&mut sims[b].feelings, m, defs, tick);
                    }
                    events.push(tick, EventKind::Jealous, b, x, Some(y));
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn break_up(
    rels: &mut Relationships,
    events: &mut EventLog,
    sims: &mut [Sim],
    defs: &[social::FeelingDef],
    heartbreak: Option<usize>,
    leaver: usize,
    left: usize,
    tick: u64,
) {
    rels.set_partners(leaver, left, false);
    rels.adjust(left, leaver, -20.0, -40.0);
    rels.adjust(leaver, left, -5.0, -20.0);
    if let Some(m) = heartbreak {
        social::add_feeling(&mut sims[left].feelings, m, defs, tick);
    }
    events.push(tick, EventKind::BrokeUp, leaver, left, None);
}

fn gain(sim: &mut Sim, total: &[f32; MAX_NEEDS], minutes: f32) {
    for (n, g) in total.iter().enumerate() {
        if *g != 0.0 {
            sim.needs[n] = (sim.needs[n] + g / minutes * MINUTES_PER_TICK * sim.mods.need_gain[n])
                .clamp(0.0, 1.0);
        }
    }
}

fn nudge(sim: &mut Sim, delta: &[f32; MAX_NEEDS]) {
    for (n, d) in delta.iter().enumerate() {
        sim.needs[n] = (sim.needs[n] + d).clamp(0.0, 1.0);
    }
}

fn pair_mut(sims: &mut [Sim], a: usize, b: usize) -> (&mut Sim, &mut Sim) {
    assert_ne!(a, b);
    if a < b {
        let (l, r) = sims.split_at_mut(b);
        (&mut l[a], &mut r[0])
    } else {
        let (l, r) = sims.split_at_mut(a);
        (&mut r[0], &mut l[b])
    }
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn facing(from: [f32; 2], to: [f32; 2]) -> f32 {
    (to[0] - from[0]).atan2(to[1] - from[1])
}

#[cfg(test)]
mod tests {
    use crate::social::EventKind;
    use crate::world::{Phase, Task, TaskKind, TICKS_PER_DAY};
    use crate::{Command, TICKS_PER_SECOND, World};

    const CONTENT: &str = r#"{
        "needs":[{"id":"social","label":"Social","decayPerHour":0.1},{"id":"fun","label":"Fun","decayPerHour":0.05}],
        "objects":[],
        "genders":[{"id":"female","label":"Female"},{"id":"male","label":"Male"}],
        "emotions":[{"id":"angry","label":"Angry","effects":{"tagPreference":{"mean":4}}},
                    {"id":"happy","label":"Happy"}],
        "feelings":[{"id":"goodChat","label":"Good chat","emotion":"happy","mood":0.05,"hours":2},
                    {"id":"jealous","label":"Jealous","emotion":"angry","mood":-0.15,"hours":6},
                    {"id":"heartbroken","label":"Heartbroken","mood":-0.25,"hours":24},
                    {"id":"lost","label":"Lost a fight","mood":-0.1,"hours":4}],
        "socials":[
          {"id":"chat","label":"Chat","minutes":10,"tags":["social","friendly"],"prefer":"liked","joinable":true,
           "acceptance":{"base":1.0},"needs":{"social":0.4},"targetNeeds":{"social":0.4},
           "success":{"friendship":6,"targetFriendship":6,"feeling":"goodChat","targetFeeling":"goodChat"}},
          {"id":"flirt","label":"Flirt","minutes":10,"tags":["social","romantic"],"animation":"flirt","prefer":"romance",
           "acceptance":{"base":1.0},"success":{"romance":10,"targetRomance":10}},
          {"id":"askPartner","label":"Ask to be partners","minutes":5,"tags":["social","romantic"],"autonomous":false,
           "requires":{"minRomance":30,"partners":false},"acceptance":{"base":1.0},"success":{"effect":"becomePartners"}},
          {"id":"fight","label":"Fight","minutes":5,"tags":["social","mean"],"animation":"fight","autonomous":false,
           "acceptance":{"base":1.0},"success":{"friendship":-20,"targetFriendship":-20,"effect":"fight","loserFeeling":"lost"}},
          {"id":"hug","label":"Hug","minutes":4,"tags":["social","friendly"],"animation":"hug","autonomous":false,
           "acceptance":{"base":0.0}}],
        "bondPresets":{"roommates":{"friendship":20},"partners":{"friendship":50,"romance":70,"partners":true}},
        "socialRules":{"defaultBond":"roommates","romanticTags":["romantic"],"jealousyFeeling":"jealous","heartbreakFeeling":"heartbroken"}}"#;

    #[test]
    fn a_call_carries_across_a_room_and_through_a_doorway_but_not_a_wall() {
        // Two rooms side by side (x 1..13 and 13..23), a doorway between them at z 6 and a window
        // at z 3; outdoors below them.
        let lot = r#"{"width":30,"depth":14,
            "walls":[[1,1,23,1],[1,11,23,11],[1,1,1,11],[23,1,23,11],[13,1,13,11]],
            "doors":[{"x":13,"z":6,"axis":"z"},{"x":5,"z":11,"axis":"x"}],
            "windows":[{"x":13,"z":3,"axis":"z"}]}"#;
        let w = World::from_json(CONTENT, lot, 1).unwrap();
        let heard = |a: [f32; 2], b: [f32; 2]| w.within_earshot(a, b) && w.within_earshot(b, a);
        assert!(heard([3.5, 6.5], [11.5, 6.5]), "across a room");
        assert!(heard([3.5, 2.5], [10.5, 9.5]), "diagonally across a room");
        assert!(heard([10.5, 6.5], [14.5, 6.5]), "through the doorway");
        assert!(!heard([7.5, 6.5], [15.5, 6.5]), "through the doorway, from afar");
        assert!(!heard([11.5, 2.5], [14.5, 2.5]), "through a wall");
        assert!(!heard([12.5, 3.5], [13.5, 3.5]), "through a window");
        assert!(heard([2.5, 12.5], [13.5, 12.5]), "out in the garden");
        assert!(!heard([2.5, 12.5], [16.5, 12.5]), "too far out in the garden");
    }

    fn town(sims: &str, relationships: &str) -> World {
        let lot = format!(
            r#"{{"width":16,"depth":16,"sims":[{sims}],"relationships":[{relationships}]}}"#
        );
        World::from_json(CONTENT, &lot, 5).unwrap()
    }

    fn ticks(w: &mut World, n: u64) {
        for _ in 0..n {
            w.tick_once();
        }
    }

    #[test]
    fn lonely_housemates_chat_and_grow_closer() {
        let mut w = town(
            r#"{"name":"A","gender":"female","x":3.5,"z":3.5},{"name":"B","gender":"male","x":8.5,"z":8.5}"#,
            "",
        );
        w.sims[0].needs[0] = 0.1;
        w.sims[1].needs[0] = 0.1;
        let before = w.relationships.get(0, 1).friendship;
        ticks(&mut w, TICKS_PER_SECOND as u64 * 120);
        assert!(
            w.relationships.get(0, 1).friendship > before,
            "they never talked"
        );
        assert!(w.sims[0].needs[0] > 0.1);
    }

    #[test]
    fn romance_respects_attraction() {
        let w = town(
            r#"{"name":"A","gender":"female","attractedTo":["female"],"x":3.5,"z":3.5},
               {"name":"B","gender":"male","attractedTo":["female"],"x":4.5,"z":3.5}"#,
            "",
        );
        let flirt = w.content.social_index("flirt").unwrap();
        // A isn't attracted to men: no romantic options toward B.
        assert!(!super::options(&w, 0, 1).iter().any(|(i, _)| *i == flirt));
        // B is attracted to women and may flirt, but A isn't interested.
        let chance = super::options(&w, 1, 0)
            .into_iter()
            .find(|(i, _)| *i == flirt)
            .unwrap()
            .1;
        assert_eq!(chance, 0.0);
    }

    #[test]
    fn directed_flirt_runs_as_a_conversation() {
        let mut w = town(
            r#"{"name":"A","gender":"female","x":3.5,"z":3.5},{"name":"B","gender":"male","x":9.5,"z":3.5}"#,
            "",
        );
        w.autonomy = false;
        let flirt = w.content.social_index("flirt").unwrap();
        w.apply(Command::Social {
            sim: 0,
            target: 1,
            social: flirt,
        })
        .unwrap();
        let mut talked = false;
        for _ in 0..TICKS_PER_SECOND * 30 {
            w.tick_once();
            if matches!(
                w.sims[0].current.as_ref().map(|a| &a.phase),
                Some(Phase::Conversing { .. })
            ) {
                talked = true;
                assert_eq!(w.sims[1].engaged_with, Some(0));
            }
        }
        assert!(talked);
        assert_eq!(w.relationships.get(0, 1).romance, 10.0);
        assert_eq!(w.relationships.get(1, 0).romance, 10.0);
        assert_eq!(
            w.sims[1].engaged_with, None,
            "released after the conversation"
        );
    }

    #[test]
    fn new_partner_breaks_old_relationship_and_flirting_makes_jealous() {
        let mut w = town(
            r#"{"name":"A","gender":"female","x":3.5,"z":3.5},{"name":"B","gender":"male","x":4.5,"z":3.5},
               {"name":"C","gender":"male","x":5.5,"z":3.5}"#,
            r#"{"a":0,"b":1,"preset":"partners"}"#,
        );
        w.autonomy = false;
        let flirt = w.content.social_index("flirt").unwrap();
        let ask = w.content.social_index("askPartner").unwrap();
        w.apply(Command::Social {
            sim: 0,
            target: 2,
            social: flirt,
        })
        .unwrap();
        ticks(&mut w, TICKS_PER_SECOND as u64 * 20);
        assert!(
            w.events
                .iter()
                .any(|e| e.kind == EventKind::Jealous && e.a == 1 && e.b == 0)
        );

        w.relationships.get_mut(0, 2).romance = 50.0;
        w.apply(Command::Social {
            sim: 0,
            target: 2,
            social: ask,
        })
        .unwrap();
        ticks(&mut w, TICKS_PER_SECOND as u64 * 20);
        assert!(w.relationships.get(0, 2).partners);
        assert!(!w.relationships.get(0, 1).partners);
        assert!(
            w.events
                .iter()
                .any(|e| e.kind == EventKind::BrokeUp && e.b == 1)
        );
        assert!(
            w.sims[1]
                .feelings
                .iter()
                .any(|m| w.content.feelings[m.def].id == "heartbroken")
        );
    }

    #[test]
    fn fights_have_a_loser_and_hurt_the_relationship() {
        let mut w = town(
            r#"{"name":"A","x":3.5,"z":3.5},{"name":"B","x":5.5,"z":3.5}"#,
            "",
        );
        w.autonomy = false;
        let fight = w.content.social_index("fight").unwrap();
        let before = w.relationships.get(0, 1).friendship;
        w.apply(Command::Social {
            sim: 0,
            target: 1,
            social: fight,
        })
        .unwrap();
        ticks(&mut w, TICKS_PER_SECOND as u64 * 20);
        assert!(w.relationships.get(0, 1).friendship < before);
        let e = w
            .events
            .iter()
            .find(|e| e.kind == EventKind::Fight)
            .expect("fight event");
        assert!(
            w.sims[e.b as usize]
                .feelings
                .iter()
                .any(|m| w.content.feelings[m.def].id == "lost")
        );
    }

    #[test]
    fn relationships_drift_daily_and_feelings_expire() {
        let mut w = town(
            r#"{"name":"A","x":3.5,"z":3.5},{"name":"B","x":12.5,"z":12.5}"#,
            "",
        );
        w.autonomy = false;
        w.relationships.get_mut(0, 1).friendship = 60.0;
        crate::social::add_feeling(&mut w.sims[0].feelings, 0, &w.content.feelings, w.tick);
        ticks(&mut w, TICKS_PER_DAY);
        assert!(w.relationships.get(0, 1).friendship < 60.0);
        assert!(w.sims[0].feelings.is_empty());
    }

    /// A and B chatting (A started it, the player's order), the others elsewhere on the lot,
    /// nobody choosing anything for themselves yet.
    fn chatting(others: &str) -> World {
        let mut w = town(&format!(r#"{{"name":"A","x":5.5,"z":5.5}},{{"name":"B","x":7.5,"z":5.5}}{others}"#), "");
        w.autonomy = false;
        let chat = w.content.social_index("chat").unwrap();
        w.apply(Command::Social { sim: 0, target: 1, social: chat }).unwrap();
        for _ in 0..TICKS_PER_SECOND * 5 {
            w.tick_once();
            if w.sims[0].conversation(&w.content).is_some() {
                return w;
            }
        }
        panic!("A and B never started talking");
    }

    fn join(w: &mut World, sim: usize, leader: u32) {
        w.sims[sim].queue.push_back(Task { kind: TaskKind::Join { leader }, directed: false, routine: false });
    }

    fn listeners(w: &World, leader: u32) -> usize {
        w.sims.iter().filter(|s| s.listening_to() == Some(leader)).count()
    }

    fn apart(p: [f32; 2], q: [f32; 2]) -> f32 {
        (p[0] - q[0]).hypot(p[1] - q[1])
    }

    #[test]
    fn a_lonely_third_joins_in_listens_and_is_released_afterwards() {
        let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
        w.sims[2].needs[0] = 0.05;
        // C chooses for themselves: A is talking and B is being talked to, so joining in is all
        // there is.
        w.autonomy = true;
        let mut joined = false;
        for _ in 0..TICKS_PER_SECOND * 6 {
            w.tick_once();
            if w.sims[2].listening_to() == Some(0) {
                joined = true;
                break;
            }
        }
        assert!(joined, "C never joined in");
        w.autonomy = false;
        let before = w.sims[2].needs[0];
        let (c, a, b) = (w.sims[2].pos, w.sims[0].pos, w.sims[1].pos);
        assert!(apart(c, a) < 2.0 && apart(c, b) < 2.0, "C listens from {c:?}, A at {a:?}, B at {b:?}");
        assert!(apart(c, a) >= 0.6 && apart(c, b) >= 0.6, "C stands on someone");
        // The snapshot: C listens to A's chat (role 3, A as partner).
        let mut out = vec![0.0; crate::snapshot::CAPACITY];
        crate::snapshot::write(&w, &mut out);
        let row = |i: usize| crate::snapshot::HEADER_LEN + i * crate::snapshot::SIM_STRIDE;
        use crate::snapshot::sim;
        assert_eq!(out[row(2) + sim::ROLE], 3.0);
        assert_eq!(out[row(2) + sim::PARTNER], 0.0);
        assert_eq!(out[row(2) + sim::SOCIAL], out[row(0) + sim::SOCIAL]);
        assert!(!w.sims[2].available_for_social(&w.content));

        let friends = (w.relationships.get(2, 0).friendship, w.relationships.get(1, 2).friendship);
        let mut ended = false;
        for _ in 0..TICKS_PER_SECOND * 15 {
            w.tick_once();
            if w.sims[0].conversation(&w.content).is_none() {
                ended = true;
                break;
            }
        }
        assert!(ended);
        assert!(w.sims[2].needs[0] > before, "listening filled C's social need");
        let gain = w.content.social_rules.join_friendship;
        assert_eq!(w.relationships.get(2, 0).friendship, friends.0 + gain);
        assert_eq!(w.relationships.get(1, 2).friendship, friends.1 + gain);
        assert!(w.sims[2].feelings.iter().any(|f| w.content.feelings[f.def].id == "goodChat"));
        // A doesn't talk again (nobody chooses): C goes after a little while.
        ticks(&mut w, super::REGROUP_TICKS as u64 + 2);
        assert!(w.sims[2].current.is_none(), "C still {:?}", w.sims[2].current.as_ref().map(|a| a.task.kind));
        assert!(w.sims[2].available_for_social(&w.content));
    }

    #[test]
    fn listeners_stay_for_the_next_social_with_the_group() {
        let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
        join(&mut w, 2, 0);
        ticks(&mut w, TICKS_PER_SECOND as u64 * 5);
        assert_eq!(listeners(&w, 0), 1);
        // The player has A chat with B again right after: C stays for it.
        let chat = w.content.social_index("chat").unwrap();
        w.apply(Command::Social { sim: 0, target: 1, social: chat }).unwrap();
        let mut gap = false;
        for _ in 0..TICKS_PER_SECOND * 30 {
            w.tick_once();
            if w.sims[0].conversation(&w.content).is_none() {
                gap = true;
                assert!(
                    matches!(w.sims[2].current.as_ref().map(|a| a.task.kind), Some(TaskKind::Join { leader: 0 })),
                    "C left between the two"
                );
            } else if gap {
                break;
            }
        }
        assert!(gap && w.sims[0].conversation(&w.content).is_some());
        ticks(&mut w, 2);
        assert_eq!(listeners(&w, 0), 1, "C listens to the second one too");
    }

    #[test]
    fn cancelling_or_leaving_releases_those_listening() {
        // The leader's conversation cancelled: the listener goes.
        let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
        join(&mut w, 2, 0);
        ticks(&mut w, TICKS_PER_SECOND as u64 * 5);
        assert_eq!(listeners(&w, 0), 1);
        w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
        ticks(&mut w, super::REGROUP_TICKS as u64 + 2);
        assert!(w.sims[2].current.is_none());
        assert_eq!(w.sims[1].engaged_with, None);

        // The listener called away: the others talk on.
        let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
        join(&mut w, 2, 0);
        ticks(&mut w, TICKS_PER_SECOND as u64 * 5);
        assert_eq!(listeners(&w, 0), 1);
        w.apply(Command::Cancel { sim: 2, index: 0 }).unwrap();
        ticks(&mut w, 2);
        assert_eq!(listeners(&w, 0), 0);
        assert!(w.sims[2].current.is_none());
        assert!(w.sims[0].conversation(&w.content).is_some());
        assert_eq!(w.sims[1].engaged_with, Some(0));

        // The one who started it leaves town: nobody waits for them.
        let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
        join(&mut w, 2, 0);
        ticks(&mut w, TICKS_PER_SECOND as u64 * 5);
        assert_eq!(listeners(&w, 0), 1);
        w.depart(0, crate::world::GoneWhy::MovedAway);
        assert!(w.sims[2].current.is_none());
        ticks(&mut w, 2);
        assert!(w.sims[2].current.is_none() && w.sims[1].engaged_with.is_none());
    }

    #[test]
    fn a_conversation_takes_four_at_most() {
        let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5},{"name":"D","x":2.5,"z":9.5},{"name":"E","x":5.5,"z":1.5}"#);
        for k in 2..5 {
            join(&mut w, k, 0);
        }
        for _ in 0..TICKS_PER_SECOND * 6 {
            w.tick_once();
            assert!(listeners(&w, 0) <= 2);
        }
        assert_eq!(listeners(&w, 0), 2);
        let group = super::group(&w.sims, &w.content, 0);
        assert_eq!(group.len(), super::MAX_GROUP);
        // The one left out does something else.
        let out = (2..5).find(|&k| w.sims[k].listening_to().is_none()).unwrap();
        assert!(w.sims[out].current.is_none());
        // Everyone in the group stands apart.
        for (x, &a) in group.iter().enumerate() {
            for &b in &group[x + 1..] {
                assert!(apart(w.sims[a].pos, w.sims[b].pos) >= 0.59, "{a} and {b} too close");
            }
        }
    }

    /// How far (radians) `sim`'s yaw is from facing `to`.
    fn off(w: &World, sim: usize, to: [f32; 2]) -> f32 {
        let p = w.sims[sim].pos;
        let want = (to[0] - p[0]).atan2(to[1] - p[1]);
        let d = (w.sims[sim].yaw - want).rem_euclid(std::f32::consts::TAU);
        d.min(std::f32::consts::TAU - d)
    }

    fn centroid(w: &World, members: &[usize]) -> [f32; 2] {
        let n = members.len() as f32;
        let sum = members.iter().fold([0.0, 0.0], |c, &m| [c[0] + w.sims[m].pos[0], c[1] + w.sims[m].pos[1]]);
        [sum[0] / n, sum[1] / n]
    }

    #[test]
    fn the_two_talking_open_up_to_those_listening() {
        let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
        // Just the two: face to face.
        ticks(&mut w, 2);
        let (a, b) = (w.sims[0].pos, w.sims[1].pos);
        assert!(off(&w, 0, b) < 0.01 && off(&w, 1, a) < 0.01, "A and B don't face each other");

        join(&mut w, 2, 0);
        ticks(&mut w, TICKS_PER_SECOND as u64 * 5);
        assert_eq!(listeners(&w, 0), 1);
        let group = super::group(&w.sims, &w.content, 0);
        assert_eq!(group.len(), 3);
        let centre = centroid(&w, &group);
        for m in group {
            assert!(off(&w, m, centre) < 0.05, "{m} doesn't face the middle of the group");
        }
        // Not straight at each other any more: both turned part way towards C (about 30 degrees,
        // standing in a triangle).
        let (a, b, c) = (w.sims[0].pos, w.sims[1].pos, w.sims[2].pos);
        assert!(off(&w, 0, b) > 0.3 && off(&w, 1, a) > 0.3, "A and B still face each other");
        let between = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| {
            let d = ((r[0] - p[0]).atan2(r[1] - p[1]) - (q[0] - p[0]).atan2(q[1] - p[1])).rem_euclid(std::f32::consts::TAU);
            d.min(std::f32::consts::TAU - d)
        };
        assert!(off(&w, 0, c) < between(a, b, c) && off(&w, 1, c) < between(b, a, c), "they don't turn towards C");

        // C goes: back to face to face.
        w.apply(Command::Cancel { sim: 2, index: 0 }).unwrap();
        ticks(&mut w, 2);
        assert_eq!(listeners(&w, 0), 0);
        let (a, b) = (w.sims[0].pos, w.sims[1].pos);
        assert!(off(&w, 0, b) < 0.01 && off(&w, 1, a) < 0.01, "A and B don't face each other again");
    }

    #[test]
    fn listening_in_is_not_saved() {
        let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
        join(&mut w, 2, 0);
        ticks(&mut w, TICKS_PER_SECOND as u64 * 5);
        assert_eq!(listeners(&w, 0), 1);
        let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
        assert!(loaded.sims[2].current.is_none() && loaded.sims[2].queue.is_empty());
    }

    /// The snapshot's `OUTCOME` for Sim `i`, ticking on until the two are in the social.
    fn outcome_shown(w: &World, i: usize) -> f32 {
        let mut out = vec![0.0; crate::snapshot::CAPACITY];
        crate::snapshot::write(w, &mut out);
        out[crate::snapshot::HEADER_LEN + i * crate::snapshot::SIM_STRIDE + crate::snapshot::sim::OUTCOME]
    }

    #[test]
    fn a_refused_hug_shows_at_once_a_chat_halfway_through() {
        // A hug is answered by the body, from the start (the renderer's step back, `characters.ts`).
        let mut w = town(r#"{"name":"A","x":5.5,"z":5.5},{"name":"B","x":7.5,"z":5.5}"#, "");
        w.autonomy = false;
        let hug = w.content.social_index("hug").unwrap();
        w.apply(Command::Social { sim: 0, target: 1, social: hug }).unwrap();
        let mut seen = None;
        for _ in 0..TICKS_PER_SECOND * 5 {
            w.tick_once();
            if let Some((_, _, success, progress)) = w.sims[0].conversation(&w.content) {
                seen = Some((success, progress, outcome_shown(&w, 0), outcome_shown(&w, 1)));
                break;
            }
        }
        let (success, progress, a, b) = seen.expect("A never started the hug");
        assert!(!success && progress < 0.1, "refused, just begun: {progress}");
        assert_eq!((a, b), (2.0, 2.0), "the refusal shows from the start");
        // A chat's outcome shows only halfway through (the answer after the opener).
        let w = chatting("");
        assert!(w.sims[0].conversation(&w.content).is_some_and(|c| c.3 < 0.5));
        assert_eq!(outcome_shown(&w, 1), 0.0);
    }
}
