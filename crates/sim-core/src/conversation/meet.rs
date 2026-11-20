//! Meeting: following, chasing, the halfway point and engaging.

use super::*;
/// The target of `i`'s social moved off: after them (`chase`) while they're someone to talk to
/// and somewhere `i` would chat (not into someone else's house or garden, as when choosing; the
/// player's orders go anywhere). Off home or to work, sitting down to eat, busy with what they
/// were told to do: `i` gives up rather than follow them across town.
#[allow(clippy::too_many_arguments)]
pub(crate) fn follow<'a>(
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
    let allowed = directed
        || open
            .plot(nav, x, z)
            .is_none_or(|p| open.open.get(p as usize).copied().unwrap_or(true));
    if allowed && sims[t].available_for_social(content) {
        chase(nav, reach, walk, sims, i, t, content, objects);
    } else {
        end_activity(&mut sims[i], content, objects);
    }
}

/// Re-route toward a target that moved; give up if they're unreachable.
#[allow(clippy::too_many_arguments)]
pub(crate) fn chase<'a>(
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
pub(crate) fn meet<'a>(
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
            route(
                nav,
                Some(reach),
                sims[who].pos,
                sims[who].tile(),
                goal,
                centre(goal),
                Some(&walk(&sims[who], goal)),
            )
        })
    };
    let Some(all) = way(sims, i, sims[t].pos) else {
        sims[t].heeding = None;
        return end_activity(&mut sims[i], content, objects);
    };
    let (mut mine, middle) = halfway(nav.stairs, sims[i].pos, &all);
    match way(sims, t, middle) {
        Some(waypoints) => {
            // What `t` was still at (back on the sofa after listening in, say) ends as they set
            // off: its slot freed (else the seat stays theirs while they walk away).
            if sims[t].current.is_some() {
                end_activity(&mut sims[t], content, objects);
            }
            let goal = *waypoints.last().unwrap_or(&sims[t].pos);
            let task = Task {
                kind: TaskKind::MoveTo {
                    x: goal[0],
                    z: goal[1],
                },
                directed: false,
                routine: false,
            };
            sims[t].current = Some(Activity {
                task,
                phase: Phase::Routing { waypoints, next: 0 },
                tags: 0,
            });
        }
        None => mine = all,
    }
    set_phase(
        &mut sims[i],
        Phase::Routing {
            waypoints: mine,
            next: 0,
        },
    );
}

/// The waypoints from `from` along `path` to its middle, and the middle. Legs are measured as
/// they're walked (`StairMap::distance`: a flight of stairs is a few metres, not the rows between
/// the storeys' blocks), and a middle on the stairs is the nearer end of them, never a point
/// between two storeys' rows (somewhere else in town).
pub(crate) fn halfway(
    stairs: &StairMap,
    from: [f32; 2],
    path: &[[f32; 2]],
) -> (Vec<[f32; 2]>, [f32; 2]) {
    let legs = || {
        std::iter::once(from)
            .chain(path.iter().copied())
            .zip(path.iter().copied())
    };
    let storey = |p: [f32; 2]| stairs.storey(p[1].floor() as i32);
    let mut left = legs().map(|(a, b)| stairs.distance(a, b)).sum::<f32>() / 2.0;
    let mut out = Vec::new();
    for (a, b) in legs() {
        let d = stairs.distance(a, b);
        if d >= left {
            let k = if d > 0.0 { left / d } else { 0.0 };
            if storey(a) != storey(b) && k < 0.5 {
                // (`a` is where the waypoints so far end, or where they start from.)
                return (out, a);
            }
            let middle = if storey(a) != storey(b) {
                b
            } else {
                [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k]
            };
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
pub(crate) fn engage(
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
    // (Never round a wall, or out of sight behind the front door.)
    let open = |spot: [f32; 2]| {
        nav.tile_free(spot[0].floor() as i32, spot[1].floor() as i32)
            && nav.line_walkable(ap, spot)
            && in_sight(nav, spot, tp)
    };
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
