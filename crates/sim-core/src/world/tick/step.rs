//! One Sim's tick: what they do next and how they get there.

use super::*;
use crate::content::Pose;
use crate::rng::Rng;
use crate::world::{
    AUTONOMY_DELAY_TICKS, BABY_CRIES_BELOW, Ctx, ObjectInstance, PERSON_UNREACHABLE_TICKS, Phase,
    Sim, Task, TaskKind, accident, crib_for, end_activity, filling, home_fills, outing_leave_by,
    pick_autonomous, progress, ready_to_leave, start_task,
};
use crate::{conversation, social};

pub(crate) fn step_sim(
    sim: &mut Sim,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut crate::items::Items,
    rng: &mut Rng,
) {
    let content = ctx.content;
    let minutes = sim.minutes();
    let rhythm = &content.day_rhythm;
    let asleep = sim.asleep(content);
    let night = !asleep && rhythm.sleep_tags != 0 && is_night(sim, ctx);
    let practising = sim.current.as_ref().is_some_and(|a| {
        matches!(a.phase, Phase::Using { .. })
            && match a.task.kind {
                TaskKind::Use {
                    object,
                    interaction,
                } => content.objects[objects[object as usize].def].interactions[interaction]
                    .trains_skills(),
                TaskKind::Cook { recipe, origin, .. } => {
                    crate::cooking::origin_interaction(content, objects, origin, recipe)
                        .is_some_and(|i| i.trains_skills())
                }
                _ => false,
            }
    });
    let skill_rules = &content.skill_rules;
    sim.practice_fatigue = if practising {
        sim.practice_fatigue + skill_rules.fatigue_per_hour * minutes / 60.0
    } else {
        (sim.practice_fatigue - skill_rules.fatigue_recovery_per_hour * minutes / 60.0).max(0.0)
    };
    for (n, def) in content.needs.iter().enumerate() {
        let rhythm_factor = if sim.away_until.is_some() {
            rhythm.work_decay[n]
        } else if asleep {
            rhythm.asleep_decay[n]
        } else if night {
            rhythm.night_decay[n]
        } else {
            1.0
        };
        if def.room {
            // Surroundings: towards how good the room is (a neutral value away from home).
            let rules = &content.room_rules;
            let target = if sim.away_until.is_some() {
                rules.away
            } else {
                ctx.room_at(sim.pos)
            };
            let k = (rules.drift * minutes / 60.0).min(1.0);
            sim.needs[n] = (sim.needs[n] + (target - sim.needs[n]) * k).clamp(0.0, 1.0);
            continue;
        }
        let decay = def.decay_per_minute * sim.mods.need_decay[n] * rhythm_factor * minutes;
        sim.needs[n] = (sim.needs[n] - decay).clamp(0.0, 1.0);
    }

    // At work: off the map until `life::update` brings them back.
    if sim.away_until.is_some() {
        return;
    }
    // On an outing: home by the hour from where they are now, looked at once a minute (wandered
    // off to the far end of the park, the way back to the bike is longer than from the way in).
    if let Some(v) = sim.visiting.filter(|v| v.home_by && v.arrived > 0)
        && (ctx.tick + u64::from(sim.id)).is_multiple_of(crate::TICKS_PER_MINUTE)
        && let Some(t) = outing_leave_by(sim, ctx, objects, v.plot, sim.pos)
        && t < v.until
        && let Some(v) = sim.visiting.as_mut()
    {
        v.until = t;
    }

    // The house changed under them since their last turn: a wall where they were going, or a
    // door opened.
    if sim.nav_seen != ctx.nav_version {
        // (Only if a change since may touch the way they're going.)
        let near = route_touched(sim, ctx);
        sim.nav_seen = ctx.nav_version;
        if near {
            reroute_after_change(sim, ctx, objects);
        }
    }

    // A baby whose needs run low cries for someone (a thought bubble).
    if content.life.baby(sim.age) {
        let lowest = (0..content.needs.len())
            .filter(|&n| !content.needs[n].room)
            .min_by(|&a, &b| sim.needs[a].total_cmp(&sim.needs[b]));
        if let Some(n) = lowest
            && sim.needs[n] < BABY_CRIES_BELOW
            && sim.planner.thought.is_none()
        {
            crate::planner::think(sim, crate::planner::ThoughtKind::Crying, n, ctx.tick);
        }
    }

    // A need ran out with nothing being done about it: an accident (a puddle, asleep on the
    // floor, takeout). Not for babies: that's what carers are for.
    let baby = content.life.baby(sim.age);
    for (a, def) in content.accidents.iter().enumerate().filter(|_| !baby) {
        if sim.needs[def.need] > 0.0 || filling(sim, content, objects, items, def.need) {
            sim.empty_since[a] = 0;
            continue;
        }
        if sim.empty_since[a] == 0 {
            sim.empty_since[a] = ctx.tick.max(1);
        }
        // When home has something for it (it's busy, or they got caught up in something), only
        // if the content says so; without (no toilet, no bed), after a short while.
        // (Kept out of a room someone is in, the bathroom, is as good as nothing.)
        let grace = if home_fills(sim, ctx, objects, def.need) && !sim.kept_out() {
            def.crowded_grace_ticks
        } else {
            Some(def.grace_ticks)
        };
        let Some(grace) = grace else { continue };
        if ctx.tick < sim.accident_at[a] || ctx.tick < sim.empty_since[a] + grace {
            continue;
        }
        sim.empty_since[a] = 0;
        accident(sim, ctx, objects, a);
        break;
    }

    // Someone is talking to this Sim: stay put unless the player gives an order.
    if sim.engaged_with.is_some() {
        if sim.queue.front().is_some_and(|t| t.directed) {
            sim.engaged_with = None;
        } else {
            return;
        }
    }
    // Someone called them: on their way to meet them, or waiting there (unless the player gives
    // an order).
    if sim.heeding.is_some() {
        if sim.queue.front().is_some_and(|t| t.directed) {
            sim.heeding = None;
        } else if sim.current.is_none() {
            return;
        }
    }

    // Climbing down from the upper bunk: nothing else until they're off its ladder.
    if sim.ladder_ticks > 0 && sim.current.is_none() {
        sim.ladder_ticks = sim.ladder_ticks.saturating_sub(sim.steps);
        return;
    }

    // Player commands interrupt whatever the Sim chose on its own.
    let autonomous_running = sim.current.as_ref().is_some_and(|a| !a.task.directed);
    if autonomous_running && sim.queue.front().is_some_and(|t| t.directed) {
        end_activity(sim, content, objects);
    }

    // Off to work or school soon: the toilet first, out of habit.
    if let Some(task) = ready_to_leave(sim, ctx, objects) {
        end_activity(sim, content, objects);
        sim.follow_up = Some(task);
    }

    // Sitting with others near (on the sofa, round a table): now and then a word from the seat,
    // without getting up (`conversation::from_seat`).
    let free_will =
        ctx.autonomy && ctx.households[sim.household as usize].free_will && !sim.steered;
    if free_will
        && sim.pose == Pose::Sit
        && sim.current.as_ref().is_some_and(|a| {
            matches!(
                (a.task.kind, &a.phase),
                (
                    TaskKind::Use { .. } | TaskKind::Eat { .. },
                    Phase::Using { .. }
                )
            )
        })
        && sim.available_for_social(content)
    {
        sim.idle_ticks = sim.idle_ticks.saturating_add(sim.steps);
        if sim.idle_ticks >= conversation::SEAT_TALK_TICKS {
            sim.idle_ticks = 0;
            if rng.next_f32() < conversation::SEAT_TALK_CHANCE
                && let Some(task) = pick_autonomous(
                    sim,
                    ctx,
                    objects,
                    items,
                    rng,
                    Some(conversation::SEAT_RANGE),
                )
            {
                conversation::from_seat(sim, task, content, ctx.briefs);
            }
        }
    }

    // Off the bike once the way to work or school, or home from it, is over or cut short. Out on
    // a visit or an outing and stopped for something else on it: off, and it's left where they
    // are, to ride on home on (`Sim::bike_parked`).
    if !crate::ride::commuting(sim) {
        if sim.visiting.is_some()
            && let Some(bike) = sim.riding.take()
        {
            sim.bound_for_bike = Some(bike);
            sim.bike_parked = Some(sim.pos);
        }
        let parked = sim.visiting.is_some() && sim.bike_parked.is_some();
        if (sim.riding.is_some() || sim.bound_for_bike.is_some()) && !parked {
            crate::ride::get_off(sim, objects);
        }
    }

    // Ticks what they're doing moves on by (fewer than `steps` if it starts partway through).
    let mut ticks = sim.steps;
    if sim.current.is_none() {
        // A routine's next step (washing hands after the toilet) before anything else they'd
        // do, unless the player asks for something; someone at the door is answered first, and
        // the step waits for after (`life::doors`). Held there rather than queued behind the
        // answer: a task the player queued meanwhile still comes before it.
        let answering = sim
            .queue
            .front()
            .is_some_and(|t| matches!(t.kind, TaskKind::Answer { .. }));
        let follow_up = if answering {
            None
        } else {
            sim.follow_up
                .take()
                .filter(|_| !sim.queue.front().is_some_and(|t| t.directed))
        };
        if let Some(task) = follow_up {
            start_task(sim, task, ctx, objects, items);
        } else if let Some(task) = sim.queue.pop_front() {
            start_task(sim, task, ctx, objects, items);
        } else {
            sim.idle_ticks = sim.idle_ticks.saturating_add(sim.steps);
            // Babies don't choose anything: they lie in a crib and are cared for (or are in
            // someone's arms, `carry.rs`).
            if content.life.baby(sim.age) {
                if sim.carried_by.is_none()
                    && let Some(task) = crib_for(sim, ctx, objects)
                {
                    start_task(sim, task, ctx, objects, items);
                }
                return;
            }
            // Standing in the road with nothing to do (set down there, or a talk there over):
            // off it first, to the pavement (free will or not).
            if sim.idle_ticks >= AUTONOMY_DELAY_TICKS
                && !sim.steered
                && let Some(task) = off_the_road(sim, ctx)
            {
                sim.idle_ticks = 0;
                start_task(sim, task, ctx, objects, items);
            }
            if sim.current.is_none() && free_will && sim.idle_ticks >= AUTONOMY_DELAY_TICKS {
                let since = sim.idle_ticks - AUTONOMY_DELAY_TICKS;
                sim.idle_ticks = 0;
                if let Some(task) = pick_autonomous(sim, ctx, objects, items, rng, None) {
                    start_task(sim, task, ctx, objects, items);
                    if sim.current.is_none() && sim.follow_up.is_none() {
                        // People move: don't hold a grudge against where they were for long.
                        let retry =
                            if matches!(task.kind, TaskKind::Social { .. } | TaskKind::Join { .. })
                            {
                                PERSON_UNREACHABLE_TICKS
                            } else {
                                social::minutes_to_ticks(content.autonomy.retry_minutes)
                            };
                        sim.remember_unreachable(task.kind, ctx.tick + retry);
                    }
                    // Catching up on a turn (`lod.rs`): they were idle until it was time to
                    // choose, so what they chose has only had the ticks since.
                    ticks = ticks.min(since + 1);
                }
            }
        }
    }

    progress(sim, ctx, objects, items, rng, ticks);
}

/// Someone on foot standing in a road's carriageway (`Travel::on_carriageway`, a lit junction's
/// box: `NavGrid::kept_off`): a walk to the nearest ground off it they can get to, the pavement
/// (by the nearest way, over tiles kept off too: `NavGrid::way_out`). None: not in the road, in a
/// car or on their way to one, or nowhere off it within `OFF_ROAD_MOST` tiles. (Canalside's Mila,
/// saved in a T's box, stood there a day, a taxi and two cars waiting round her.)
pub(crate) fn off_the_road(sim: &Sim, ctx: &Ctx) -> Option<Task> {
    let travel = ctx.travel;
    let nav = &ctx.nav;
    if travel.graph.is_empty()
        || sim.bound_for_car.is_some()
        || sim.taxi.is_some()
        || sim.riding.is_some()
    {
        return None;
    }
    let road = |(x, z): (i32, i32)| {
        nav.kept_off_at(x, z) || travel.on_carriageway([x as f32 + 0.5, z as f32 + 0.5], 0.0)
    };
    let start = sim.tile();
    if !nav.kept_off_at(start.0, start.1) && !travel.on_carriageway(sim.pos, OFF_ROAD_IN) {
        return None;
    }
    // (Nearest first, over what they may step onto, or kept off but for stepping onto: they're
    // on it already.)
    let mut seen = std::collections::HashSet::from([start]);
    let mut todo = std::collections::VecDeque::from([start]);
    while let Some((x, z)) = todo.pop_front() {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let t = (x + dx, z + dz);
            if seen.len() > OFF_ROAD_MOST || !seen.insert(t) {
                continue;
            }
            let free = nav.can_step(x, z, t.0, t.1);
            if !free && !(nav.kept_off_at(t.0, t.1) && nav.lot.edge_walkable(x, z, t.0, t.1)) {
                continue;
            }
            if free && !road(t) && ctx.reach.reaches(nav, start, t) {
                return Some(Task {
                    kind: TaskKind::MoveTo {
                        x: t.0 as f32 + 0.5,
                        z: t.1 as f32 + 0.5,
                    },
                    directed: false,
                    routine: false,
                });
            }
            todo.push_back(t);
        }
    }
    None
}

/// How far in from its kerbs someone stands in a road for `off_the_road` to walk them off it (m),
/// and how many tiles round it looks for ground off it at most.
pub(crate) const OFF_ROAD_IN: f32 = 0.3;
pub(crate) const OFF_ROAD_MOST: usize = 1024;
