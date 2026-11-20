//! Leaving: when a visit ends and saying goodbye.

use super::*;

/// Guests go home when the visit is over, it's late, they need something, or there's nothing
/// for them to do (saying goodbye to one of the hosts first, if one is free).
pub(super) fn end_visit_if_due(w: &mut World, i: usize, hour: f32) {
    let tick = w.tick;
    let Some(visit) = w.sims[i].visiting else {
        return;
    };
    give_up_farewell(w, i);
    // A guest left with nothing to do while the hosts are all off at work goes home (hosts
    // just out visiting come back; parks have no hosts). The hosts are the grown-ups, who let a
    // guest in (`opens_door`), as `world::hosts_about_till` has them: a baby or a child at home
    // keeps no guest.
    // (The hosts looked for only then: each tick for every guest, it was a good part of one.)
    let hosts_out = w.sims[i].current.is_none() && !w.plots[visit.plot as usize].public && {
        let content = &w.content;
        let mut household = w
            .sims
            .iter()
            .filter(|s| s.here() && w.households[s.household as usize].plot == Some(visit.plot))
            .peekable();
        household.peek().is_some() && household.all(|s| s.away_until.is_some() || !s.adult(content))
    };
    // Left standing around with nothing to do (the hosts busy, nothing they may use) for long
    // enough, a guest with free will says it's time they went (without, they're the player's
    // to send home).
    let free_will = w.autonomy && w.households[w.sims[i].household as usize].free_will;
    let content = &w.content;
    let sim = &mut w.sims[i];
    let idle = if sim.current.is_none()
        && sim.queue.is_empty()
        && sim.follow_up.is_none()
        && sim.engaged_with.is_none()
    {
        visit.idle.saturating_add(sim.steps)
    } else {
        0
    };
    if let Some(v) = sim.visiting.as_mut() {
        v.idle = idle;
    }
    let bored = free_will
        && content
            .visits
            .bored_minutes
            .is_some_and(|m| idle as f32 * MINUTES_PER_TICK >= m);
    let below = |least: f32| {
        content
            .needs
            .iter()
            .zip(&sim.needs)
            .any(|(d, &n)| !d.room && n < least)
    };
    let tired = below(LOW_NEED);
    // The place shut (its hours, `Plot::open_at`): time to go for one with nothing on there, and
    // for everyone by last orders (`staff::LAST_ORDERS`) after; one on the way there by then
    // turns back.
    let place = &w.plots[visit.plot as usize];
    let shut = place.hours.is_some()
        && !place.open_at_tick(tick)
        && (idle > 0
            || !place.open_at_tick(tick.saturating_sub(ticks(crate::staff::LAST_ORDERS * 60.0))));
    let late = !(6.0..22.0).contains(&hour) || hosts_out || shut;
    // (On the way there, and just there for `visits.leastMinutes`: they stay, unless a need is
    // about to run out, it's late or the hosts have gone; not for a low one the place can't see
    // to, nor for being bored.)
    // (Or about to go on: off the bike by the place, say.)
    let there = |k: &TaskKind| matches!(k, TaskKind::Visit { plot } if *plot == visit.plot);
    let on_the_way = visit.arrived == 0
        && (sim.current.as_ref().is_some_and(|a| there(&a.task.kind))
            || sim.queue.front().is_some_and(|t| there(&t.kind)));
    let staying = on_the_way
        || visit.arrived > 0 && tick < visit.arrived + ticks(content.visits.least_minutes);
    let due = if staying {
        below(content.autonomy.urgent_below) || late
    } else {
        tick >= visit.until || tired || late || bored
    };
    // (Queued to go within their stay for a need about to run out that's since been seen to
    // there: they stay after all. Gone 38 minutes in, for a need the park's bench had seen to.)
    if visit.leaving && !(staying && due) {
        if staying {
            sim.queue.retain(|t| {
                !(matches!(t.kind, TaskKind::GoHome) && !t.directed) && !is_farewell(content, t)
            });
        }
        if let Some(v) = sim.visiting.as_mut() {
            v.leaving = false;
        }
    }
    let busy = sim
        .current
        .as_ref()
        .is_some_and(|a| matches!(a.task.kind, TaskKind::GoHome | TaskKind::Work));
    if due
        && !busy
        && sim.engaged_with.is_none()
        && !sim.queue.iter().any(|t| matches!(t.kind, TaskKind::GoHome))
    {
        if staying && let Some(v) = sim.visiting.as_mut() {
            v.leaving = true;
        }
        if let Some((host, social)) = farewell(w, i, visit.plot) {
            w.sims[i].queue.push_back(Task {
                kind: TaskKind::Social {
                    target: host as u32,
                    social,
                },
                directed: false,
                routine: false,
            });
        }
        w.sims[i].queue.push_back(Task {
            kind: TaskKind::GoHome,
            directed: false,
            routine: false,
        });
    }
}

/// Whether `task` is a guest's goodbye to a host (`visits.farewell`) rather than something the
/// player asked for.
fn is_farewell(content: &Content, task: &Task) -> bool {
    matches!(task.kind, TaskKind::Social { social, .. } if content.visits.farewell.contains(&social))
        && !task.directed
}

/// Who guest `i` leaving `plot` says goodbye to, and how (`visits.farewell`): the nearest
/// member of the household living there who is around, awake and free to talk (or already
/// talking with the guest), close enough to get to in `farewellMinutes`. Nobody wakes up, comes
/// out of the bathroom or leaves something the player asked for to see a guest off.
pub(crate) fn farewell(w: &World, i: usize, plot: u32) -> Option<(usize, usize)> {
    let content = &w.content;
    let rules = &content.visits;
    let guest = &w.sims[i];
    if rules.farewell.is_empty() || guest.pressed(content) || w.plots[plot as usize].public {
        return None;
    }
    let household = w.households.iter().position(|h| h.plot == Some(plot))?;
    let reach = guest.walk_per_minute(content) * rules.farewell_minutes;
    w.sims
        .iter()
        .enumerate()
        .filter(|&(j, s)| {
            j != i
                && s.here()
                && s.household as usize == household
                && s.away_until.is_none()
                && s.visiting.is_none()
                && !content.life.baby(s.age)
                && w.plot_at(s.tile().0, s.tile().1) == Some(plot)
                && (s.available_for_social(content) || s.engaged_with == Some(i as u32))
                && !s.queue.front().is_some_and(|t| t.directed)
        })
        .filter_map(|(j, s)| {
            let social = rules.farewell.iter().copied().find(|&f| {
                conversation::social_allowed(
                    content,
                    &w.relationships,
                    guest,
                    s,
                    &content.socials[f],
                )
            })?;
            Some((
                j,
                social,
                (s.pos[0] - guest.pos[0]).hypot(s.pos[1] - guest.pos[1]),
            ))
        })
        .filter(|&(.., d)| d <= reach)
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(j, social, _)| (j, social))
}

/// A goodbye that hasn't started within `visits.farewellMinutes` (the host went off, or got
/// busy) is skipped: the guest just leaves.
fn give_up_farewell(w: &mut World, i: usize) {
    let World {
        sims,
        content,
        objects,
        ..
    } = &mut *w;
    let sim = &mut sims[i];
    let starting = sim.current.as_ref().is_some_and(|a| {
        is_farewell(content, &a.task) && !matches!(a.phase, Phase::Conversing { .. })
    });
    let Some(visit) = sim.visiting.as_mut() else {
        return;
    };
    if !starting {
        visit.farewell = 0;
        return;
    }
    visit.farewell = visit.farewell.saturating_add(sim.steps);
    if visit.farewell as f32 * MINUTES_PER_TICK >= content.visits.farewell_minutes {
        visit.farewell = 0;
        end_activity(sim, content, objects);
    }
}
