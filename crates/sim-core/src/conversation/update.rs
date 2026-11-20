//! The tick's conversations: the states, joining, places and grouping.
//!
//! Split from `conversation.rs` (see `mod.rs`).

use super::*;
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
    Routing {
        goal: [f32; 2],
    },
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
        travel,
        ..
    } = w;
    let nav = NavGrid {
        lot,
        blocked,
        stairs,
        kept_off: &travel.junction_tiles,
        steps: None,
    };
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

    // Who has someone listening in, counted once: nobody starts listening before the joining
    // below, so whoever has nobody now talks only with the one they talk to (`open_up`), and
    // isn't looked for among the whole town every tick of their talk.
    let mut listened_to = vec![false; n];
    for l in sims.iter().filter_map(Sim::listening_to) {
        if let Some(heard) = listened_to.get_mut(l as usize) {
            *heard = true;
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
            Phase::Using { .. }
            | Phase::Knocking { .. }
            | Phase::Queued { .. }
            | Phase::Boarding { .. }
            | Phase::Alighting { .. }
            | Phase::Driving
            | Phase::AtKerb { .. } => continue,
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
                // (Near, but the other side of a wall: on round to them.)
                if gap <= s.distance + ENGAGE_SLACK && in_sight(&nav, sims[i].pos, sims[t].pos) {
                    set_phase(&mut sims[i], Phase::Waiting { ticks: 0 });
                } else if !sims[i].called
                    && sims[t].available_for_social(content)
                    && calls_out(&nav, s, sims[i].pos, sims[t].pos)
                {
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
                // Near enough to talk is only where they see each other (`in_sight`).
                let (from, to) = (sims[i].pos, sims[t].pos);
                let mut sight = None;
                let mut seen = || *sight.get_or_insert_with(|| in_sight(&nav, from, to));
                // About to drop (or burst) themselves: the talk would break off at once.
                if sims[i].pressed(content) {
                    end_activity(&mut sims[i], content, objects);
                } else if sims[i].paused.is_some() {
                    // From the seat: with them sitting there, or not at all (back to what they sat
                    // down to do).
                    if gap <= SEAT_RANGE + 0.3 && seen() && sims[t].available_for_social(content) {
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
                    } else {
                        end_activity(&mut sims[i], content, objects);
                    }
                } else if walking && (gap > s.distance + ENGAGE_SLACK || !seen()) {
                    // Got to the middle first: wait for them to get there too (where they see it).
                    sims[i].yaw = facing(sims[i].pos, sims[t].pos);
                    set_phase(&mut sims[i], Phase::Waiting { ticks: ticks + 1 });
                } else if gap > s.distance + ENGAGE_SLACK + 0.6 || !seen() {
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
                if sims[t].engaged_with != Some(i as u32)
                    || sims[i].pressed(content)
                    || sims[t].pressed(content)
                {
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
                    open_up(sims, content, i, t, listened_to[i]);
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
            Phase::Routing { waypoints, .. } => Joining::Routing {
                goal: *waypoints.last().unwrap_or(&sims[j].pos),
            },
            Phase::Waiting { ticks } => Joining::Waiting(*ticks),
            Phase::Conversing { elapsed, .. } => Joining::Listening(*elapsed),
            Phase::Using { .. }
            | Phase::Calling { .. }
            | Phase::Knocking { .. }
            | Phase::Queued { .. }
            | Phase::Boarding { .. }
            | Phase::Alighting { .. }
            | Phase::Driving
            | Phase::AtKerb { .. } => continue,
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
                } else if dist(sims[j].pos, middle(sims, l, t)) <= JOIN_RANGE
                    && in_sight(&nav, sims[j].pos, sims[l].pos)
                {
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
                // (Listening in from their seat: only to a talk beside them, in sight of it.)
                let seated = sims[j].paused.is_some();
                let off = dist(sims[j].pos, middle(sims, l, t));
                let (from, to) = (sims[j].pos, sims[l].pos);
                let mut sight = None;
                let mut seen = || *sight.get_or_insert_with(|| in_sight(&nav, from, to));
                if seated && (off > SEAT_RANGE + 0.6 || !seen()) {
                    end_activity(&mut sims[j], content, objects);
                } else if off > JOIN_RANGE + 0.6 || !seen() {
                    chase(&nav, reach, &walk, sims, j, l, content, objects);
                } else if group(sims, content, l).len() >= MAX_GROUP {
                    end_activity(&mut sims[j], content, objects);
                } else {
                    take_place(
                        &nav,
                        reach,
                        &walk,
                        relationships,
                        events,
                        sims,
                        content,
                        objects,
                        j,
                        l,
                        t,
                        tick,
                    );
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
                set_phase(
                    &mut sims[j],
                    Phase::Conversing {
                        elapsed: elapsed + MINUTES_PER_TICK,
                        success: true,
                    },
                );
            }
            (Joining::Listening(_), None) => set_phase(&mut sims[j], Phase::Waiting { ticks: 0 }),
        }
    }
}

/// Whether `sim`, talked to with `s`, stays in their seat: sitting at something they carry on
/// with afterwards (the sofa, a coffee at the table, a meal; or talking from it themselves), for
/// a social they needn't get up for.
pub(crate) fn keeps_seat(sim: &Sim, s: &SocialDef) -> bool {
    let sitting = sim.paused.is_some()
        || sim.current.as_ref().is_some_and(|a| {
            matches!(
                (a.task.kind, &a.phase),
                (
                    TaskKind::Use { .. } | TaskKind::Eat { .. },
                    Phase::Using { .. }
                )
            )
        });
    sim.pose == Pose::Sit && sitting && s.distance >= SEATED_DISTANCE
}

/// Someone sitting talks to someone sitting near (`task`, a social), or listens in on a talk
/// beside them (a `Join`), from their seat (`World::step_sim`'s `pick_autonomous` with
/// `from_seat`): what they sat down to do waits (`Sim::paused`) until it's over.
pub(crate) fn from_seat(
    sim: &mut Sim,
    task: Task,
    content: &Content,
    briefs: &[crate::world::SimBrief],
) {
    let tags = match task.kind {
        TaskKind::Social { social, .. } => content.socials[social].tags,
        TaskKind::Join { leader } => briefs
            .get(leader as usize)
            .and_then(|b| b.conversing)
            .map_or(0, |(_, social)| content.socials[social].tags),
        _ => return,
    };
    sim.paused = sim.current.take();
    // (Right there: no calling out, no walking over.)
    sim.called = true;
    sim.social_waited = 0;
    sim.current = Some(Activity {
        task,
        phase: Phase::Waiting { ticks: 0 },
        tags,
    });
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
    // (Nor taken into someone else's social: hugged, they aren't choosing the next one, and
    // nobody stands by a hug waiting for it to end.)
    if s.away_until.is_some() || s.pose == Pose::Lie || s.engaged_with.is_some() {
        return false;
    }
    match s.current.as_ref() {
        None => true,
        Some(a) => match (a.task.kind, &a.phase) {
            (_, Phase::Conversing { .. }) => false,
            (TaskKind::Social { target, .. }, _) => sims
                .get(target as usize)
                .is_some_and(|t| dist(t.pos, sims[j].pos) <= JOIN_RANGE * 2.0),
            _ => false,
        },
    }
}

/// `j` joins in on the conversation `l` is having with `t`: a free spot around the two (to one
/// side of them first, then all round) in sight of both, walked to (`join_spot`) unless they're
/// already there; there, everyone faces the middle of the group. Sitting (listening in from their
/// seat), they stay where they are. Nowhere to go and the two not both in sight from where they
/// are (one round the door frame): there's no place for them, and they go.
#[allow(clippy::too_many_arguments)]
fn take_place<'a>(
    nav: &NavGrid,
    reach: &crate::path::Reach,
    walk: &impl Fn(&Sim, (i32, i32)) -> Grounds<'a>,
    rels: &mut Relationships,
    events: &mut EventLog,
    sims: &mut [Sim],
    content: &Content,
    objects: &mut [ObjectInstance],
    j: usize,
    l: usize,
    t: usize,
    tick: u64,
) {
    let others: Vec<usize> = group(sims, content, l)
        .into_iter()
        .filter(|&m| m != j)
        .collect();
    let (a, b) = (sims[l].pos, sims[t].pos);
    let mid = middle(sims, l, t);
    let d = dist(a, b).max(0.5);
    let across = [-(b[1] - a[1]) / d, (b[0] - a[0]) / d];
    // (Beside the pair, the three of them in a triangle; then round them, a little further out.)
    let side = (d * 0.87).max(0.8);
    let round = (d * 0.5 + 0.5).max(side);
    let from = sims[j].pos;
    let by_distance = |p: &[f32; 2], q: &[f32; 2]| dist(*p, from).total_cmp(&dist(*q, from));
    let mut spots: Vec<[f32; 2]> = [1.0, -1.0]
        .iter()
        .map(|k| [mid[0] + across[0] * side * k, mid[1] + across[1] * side * k])
        .collect();
    spots.sort_by(by_distance);
    let mut ring: Vec<[f32; 2]> = (0..12)
        .map(|k| {
            let angle = k as f32 * std::f32::consts::PI / 6.0;
            [mid[0] + round * angle.sin(), mid[1] + round * angle.cos()]
        })
        .collect();
    ring.sort_by(by_distance);
    spots.extend(ring);
    let seen = |p: [f32; 2]| in_sight(nav, p, a) && in_sight(nav, p, b);
    let spot = spots.into_iter().find(|&p| {
        nav.tile_free(p[0].floor() as i32, p[1].floor() as i32)
            && nav.line_walkable(from, p)
            && seen(p)
            && others
                .iter()
                .all(|&m| dist(sims[m].pos, p) >= GROUP_SPACING)
    });
    // (Nowhere free, or sitting: they listen from where they are.)
    let spot = spot.filter(|_| sims[j].pose == Pose::Stand);
    if spot.is_none() && !seen(from) {
        return end_activity(&mut sims[j], content, objects);
    }
    if let Some(p) = spot {
        if dist(from, p) > SETTLE {
            // Over to it on foot; they take it on getting there (`update`, again here).
            let tile = (p[0].floor() as i32, p[1].floor() as i32);
            if let Some(waypoints) = route(
                nav,
                Some(reach),
                from,
                sims[j].tile(),
                tile,
                p,
                Some(&walk(&sims[j], tile)),
            ) {
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
    set_phase(
        &mut sims[j],
        Phase::Conversing {
            elapsed: 0.0,
            success: true,
        },
    );
    open_up(sims, content, l, t, true);
}

/// The two talking (`l` and `t`) face each other, or, with others listening in, open up to the
/// group: everyone there faces its middle (where they all are, on average), so nobody has their
/// back turned. Called as people join and every tick of the conversation, so it follows people
/// leaving too. `listened`: whether anyone might be listening in (if not, only the two are
/// there, without looking).
fn open_up(sims: &mut [Sim], content: &Content, l: usize, t: usize, listened: bool) {
    let members = if listened {
        group(sims, content, l)
    } else {
        vec![l, t]
    };
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
    let sum = members.iter().fold([0.0, 0.0], |c, &m| {
        [c[0] + sims[m].pos[0], c[1] + sims[m].pos[1]]
    });
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
fn group_outcome(
    content: &Content,
    rels: &mut Relationships,
    events: &mut EventLog,
    sims: &mut [Sim],
    l: usize,
    s: &SocialDef,
    tick: u64,
) {
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
            for (kind, reversed) in
                social::milestone_events((&before.0, &before.1), (rels.get(a, b), rels.get(b, a)))
            {
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

pub(crate) fn set_phase(sim: &mut Sim, phase: Phase) {
    if let Some(act) = sim.current.as_mut() {
        act.phase = phase;
    }
}
