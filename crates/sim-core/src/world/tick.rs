//! The tick’s step: choosing, using and leaving things, and what can be used.
//!
//! Split from `world.rs` (see `mod.rs`).

use super::*;

/// Keeps a queue within `MAX_QUEUE` by dropping what was chosen for them from the back, but
/// never the first `keep` tasks (just put there), the player's orders, nor a recipe whose
/// ingredients are paid for: for those, it goes over the cap.
pub(crate) fn trim_queue(queue: &mut VecDeque<Task>, keep: usize) {
    while queue.len() > MAX_QUEUE {
        let paid = |t: &Task| matches!(t.kind, TaskKind::Cook { servings, .. } if servings > 0);
        let Some(k) = queue
            .iter()
            .rposition(|t| !t.directed && !paid(t))
            .filter(|&k| k >= keep)
        else {
            break;
        };
        queue.remove(k);
    }
}

/// Whether a household with `funds` can pay `cost` (free things are always affordable).
pub fn affordable(cost: i64, funds: i64) -> bool {
    cost <= 0 || funds >= cost
}

/// Hour a Sim gets up today: the usual time, or earlier for an early shift.
pub(crate) fn wake_hour(sim: &Sim, ctx: &Ctx) -> f32 {
    let rhythm = &ctx.content.day_rhythm;
    match leave_today(sim, ctx) {
        Some(leave) => (crate::clock::hour(leave) - rhythm.wake_before_work_minutes / 60.0)
            .clamp(0.0, rhythm.wake_hour),
        None => rhythm.wake_hour,
    }
}

/// Whether it is night for this Sim (time to be in bed): their planned sleep, else the usual
/// day rhythm.
pub(crate) fn is_night(sim: &Sim, ctx: &Ctx) -> bool {
    let routines = &ctx.households[sim.household as usize].routines;
    if let Some(night) =
        crate::planner::planned_night(ctx.content, sim, routines, ctx.day, ctx.minute)
    {
        return night;
    }
    let rhythm = &ctx.content.day_rhythm;
    if ctx.hour >= rhythm.bed_hour {
        return true;
    }
    if ctx.hour >= rhythm.wake_hour {
        return false;
    }
    rhythm.is_night(ctx.hour, wake_hour(sim, ctx))
}

/// What an object's quality adds to an interaction: a bed's stars make a night only a little
/// shorter (`DayRhythm::sleep_quality_bonus`), anything else's do more.
pub(crate) fn quality_boost(
    content: &Content,
    inter: &crate::content::Interaction,
    quality: u8,
) -> f32 {
    let rhythm = &content.day_rhythm;
    if inter.tags & rhythm.sleep_tags != 0 {
        rhythm.sleep_quality_factor(quality)
    } else {
        content.object_rules.quality_factor(quality)
    }
}

/// Worn out: what a night's sleep fills is nearly empty (bed, not a nap, whatever the hour).
pub(crate) fn exhausted(sim: &Sim, content: &Content) -> bool {
    let rhythm = &content.day_rhythm;
    rhythm
        .rest_need
        .is_some_and(|n| sim.needs[n] < rhythm.exhausted_below)
}

/// Time to get up for work or school (`DayRhythm::wake_before_work_minutes` before leaving).
pub(crate) fn getting_up(sim: &Sim, ctx: &Ctx) -> bool {
    let early = social::minutes_to_ticks(ctx.content.day_rhythm.wake_before_work_minutes);
    leave_today(sim, ctx).is_some_and(|leave| leave <= ctx.tick + early)
}

/// The middle of the night: a rested sleeper stays in bed until a little before they'd get up.
pub(crate) fn before_dawn(sim: &Sim, ctx: &Ctx) -> bool {
    is_night(sim, ctx) && {
        let wake = wake_hour(sim, ctx);
        !(wake - ctx.content.day_rhythm.lie_in_hours..wake).contains(&ctx.hour)
    }
}

/// Target skill levels (what probation and the next promotion need, and skill goals) and how
/// keen they are on each (the job's fully, a goal's by its place on the list).
pub(crate) fn job_goals(sim: &Sim, content: &Content) -> ([f32; MAX_SKILLS], [f32; MAX_SKILLS]) {
    let mut goals = [0.0f32; MAX_SKILLS];
    let mut keen = [0.0f32; MAX_SKILLS];
    if let Some(job) = &sim.job {
        let levels = &content.careers[job.career].levels;
        for level in levels[job.level..].iter().take(2) {
            for &(s, l) in &level.requires {
                goals[s] = goals[s].max(l);
                keen[s] = 1.0;
            }
        }
    }
    crate::planner::skill_targets(content, sim, &mut goals, &mut keen);
    (goals, keen)
}

/// How much an idle Sim wants to practise with `inter` (0 when needs come first).
pub(crate) fn training_interest(
    sim: &Sim,
    content: &Content,
    inter: &crate::content::Interaction,
) -> f32 {
    let rules = &content.skill_rules;
    let lowest = sim.needs[..content.needs.len()]
        .iter()
        .copied()
        .fold(1.0f32, f32::min);
    if lowest < 0.35 {
        return 0.0;
    }
    let (goals, keen) = job_goals(sim, content);
    let mut best = 0.0f32;
    for (s, &gain) in inter.skill_gain.iter().enumerate() {
        if gain <= 0.0 || sim.skills[s] >= rules.max_level {
            continue;
        }
        let mut want = rules.interest * sim.mods.skill_gain[s];
        if goals[s] > sim.skills[s] {
            want += rules.goal_interest * keen[s];
        }
        best = best.max(want);
    }
    best * lowest / (1.0 + sim.practice_fatigue)
}

/// Adds `minutes` of practice to a skill (a tick's, or more when catching up on a turn, see
/// `lod.rs`) and notes a new level.
pub(crate) fn practise(
    sim: &mut Sim,
    content: &Content,
    skill: usize,
    per_hour: f32,
    minutes: f32,
) {
    let rules = &content.skill_rules;
    let before = sim.skills[skill];
    let gain = rules.gain(
        per_hour * sim.mods.skill_gain[skill],
        minutes / 60.0,
        before,
    );
    let after = (before + gain).min(rules.max_level);
    sim.skills[skill] = after;
    if after.floor() > before.floor() {
        sim.skill_ups |= 1 << skill;
        sim.planner.recheck = true;
    }
}

/// The tiles a tick's searches may expand (`path::nodes_searched`) before those who may look for a
/// way wait for the next tick (open-world towns: a whole hamlet setting off at once). The
/// player's household and whoever is watched never wait, nor anyone who waited last tick.
pub const SEARCH_BUDGET: u64 = 150_000;

/// Whether `sim`'s turn may look for a way somewhere: they're choosing or starting something,
/// or walking a way a change since may lie across. (Someone at something, talking or waiting
/// won't, until it ends.)
pub(crate) fn may_search(sim: &Sim, ctx: &Ctx) -> bool {
    match sim.current.as_ref().map(|a| &a.phase) {
        None => true,
        Some(Phase::Routing { .. }) => sim.nav_seen != ctx.nav_version && route_touched(sim, ctx),
        Some(_) => false,
    }
}

/// Whether a change to walking since they last looked (`Sim::nav_seen`, `World::nav_log`) may
/// lie across the rest of a walker's way: a tile round any stretch of it still ahead (from where
/// they stand to the next waypoint, and on between the waypoints), so a long walk by road isn't
/// planned again for a change far off it, even inside the box round all of it. Not walking:
/// true (nothing to keep).
pub(crate) fn route_touched(sim: &Sim, ctx: &Ctx) -> bool {
    let Some(Activity {
        phase: Phase::Routing { waypoints, next },
        ..
    }) = sim.current.as_ref()
    else {
        return true;
    };
    let Some(changes) = ctx.nav_log.changes_since(sim.nav_seen, ctx.nav_version) else {
        return true;
    };
    let changes: Vec<[i32; 4]> = changes.collect();
    let ahead = waypoints.get(*next..).unwrap_or_default();
    std::iter::once(&sim.pos)
        .chain(ahead)
        .zip(ahead)
        .any(|(a, b)| {
            let r = [
                a[0].min(b[0]).floor() as i32 - 1,
                a[1].min(b[1]).floor() as i32 - 1,
                a[0].max(b[0]).floor() as i32 + 2,
                a[1].max(b[1]).floor() as i32 + 2,
            ];
            changes.iter().any(|&c| crate::path::overlaps(c, r))
        })
}

/// A wall or something they walk through moved while they were on their way: their waypoints
/// may cross it now. Route again from where they stand, and if there's no way there any more,
/// give up (remembering it, if they chose it themselves).
pub(crate) fn reroute_after_change(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let Some(act) = sim.current.as_ref() else {
        return;
    };
    let Phase::Routing { waypoints, .. } = &act.phase else {
        return;
    };
    let Some(&target) = waypoints.last() else {
        return;
    };
    let task = act.task;
    let goal = (target[0].floor() as i32, target[1].floor() as i32);
    if let Some(waypoints) = walk_route(ctx, sim, goal, target) {
        if let Some(act) = sim.current.as_mut() {
            act.phase = Phase::Routing { waypoints, next: 0 };
        }
    } else {
        let directed = task.directed;
        // (A floor a tool was taken to: that floor, not the tool, out of reach.)
        let kind = match sim.sweep {
            Some((x, z)) if matches!(task.kind, TaskKind::Use { .. }) => TaskKind::Clean { x, z },
            _ => task.kind,
        };
        end_activity(sim, ctx.content, objects);
        if !directed {
            let retry = if matches!(task.kind, TaskKind::Social { .. } | TaskKind::Join { .. }) {
                PERSON_UNREACHABLE_TICKS
            } else {
                social::minutes_to_ticks(ctx.content.autonomy.retry_minutes)
            };
            sim.remember_unreachable(kind, ctx.tick + retry);
        }
    }
}

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
    let free_will = ctx.autonomy && ctx.households[sim.household as usize].free_will;
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

/// What a Sim may use on a plot: everything at home or in public places; as a guest,
/// anything that isn't private (bed, bathroom...); nothing on other people's lots.
#[derive(PartialEq)]
pub(crate) enum Access {
    Full,
    Guest,
    None,
}

pub(crate) fn access(ctx: &Ctx, sim: &Sim, plot: Option<u32>) -> Access {
    access_in(ctx.households, ctx.plots, sim, plot)
}

/// Whether a guest may use `inter` of `def` at the host's: nothing private (sleeping, washing;
/// the toilet is fine), nothing that's the household's own (`ObjectDef::private`: a wardrobe,
/// the bath cabinet), no cooking and no caring for the host's baby.
pub(crate) fn guest_may(
    content: &Content,
    def: &crate::content::ObjectDef,
    inter: &crate::content::Interaction,
) -> bool {
    let rules = &content.social_rules;
    !def.private
        && inter.tags & rules.busy_tags & !rules.guest_tags == 0
        && inter.recipe.is_none()
        && !inter.cares()
}

/// Why someone may not take an interaction at their age, wherever it is (see `unfit`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unfit {
    /// Every stage it is for (`Interaction::stages`) is older than theirs: wine for a child.
    TooYoung,
    /// It is for younger ones only: a toddler's spring rider for a grown-up.
    TooBig,
    /// Homework is a pupil's.
    NoHomework,
    /// Looking after a baby is a grown-up's.
    TooYoungToCare,
}

/// Why someone of `age` may not take `inter` (None: they may): its life stages, homework only
/// at a school stage, and caring for a baby only grown up. Free will and the player's orders
/// keep it alike.
pub(crate) fn unfit(
    content: &Content,
    age: f32,
    inter: &crate::content::Interaction,
) -> Option<Unfit> {
    let life = &content.life;
    let stage = life.stage(age);
    if !inter.for_stage(stage) {
        return Some(if crate::content::all_older(inter.stages, stage) {
            Unfit::TooYoung
        } else {
            Unfit::TooBig
        });
    }
    if inter.homework > 0.0 && !life.stage_at(age).is_some_and(|s| s.school) {
        return Some(Unfit::NoHomework);
    }
    (inter.cares() && !life.adult(age)).then_some(Unfit::TooYoungToCare)
}

/// A condition of a use that isn't met now (`unmet`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unmet {
    /// Not at this hour (content `hours`).
    Hours,
    /// Not under a roof (content `sky`).
    Sky,
    /// Nothing to work on (content `workOn`).
    Nothing(crate::content::WorkOn),
}

/// Which of `inter`'s conditions isn't met now (None: they all are): the hours of the day it
/// starts in, open sky over `obj`, and something to work on (`work`: what the resident's
/// household has, `World::work_waiting`). Free will and the player's orders keep them alike.
pub(crate) fn unmet(
    lot: &Lot,
    hour: f32,
    obj: &ObjectInstance,
    content: &Content,
    inter: &crate::content::Interaction,
    work: u8,
) -> Option<Unmet> {
    if inter
        .hours
        .is_some_and(|h| !crate::content::in_hours(h, hour))
    {
        return Some(Unmet::Hours);
    }
    if inter.sky
        && obj
            .tiles(content)
            .any(|(x, z)| lot.room_at(x, z) != crate::lot::OUTDOORS)
    {
        return Some(Unmet::Sky);
    }
    inter
        .work_on
        .filter(|w| work & w.bit() == 0)
        .map(Unmet::Nothing)
}

/// An hour of the day (0..24) as the clock shows it: `20:00`, `5:30`.
pub(crate) fn clock_time(hour: f32) -> String {
    let minutes = (hour * 60.0).round() as u32 % 1440;
    format!("{}:{:02}", minutes / 60, minutes % 60)
}

/// `unmet` for `sim` during a step.
pub(crate) fn conditions_met(
    ctx: &Ctx,
    sim: &Sim,
    obj: &ObjectInstance,
    inter: &crate::content::Interaction,
) -> bool {
    // (With what they've just made too: the washing taken out, carried on to dry.)
    let work = ctx.work.get(sim.household as usize).copied().unwrap_or(0) | sim.made;
    unmet(ctx.nav.lot, ctx.hour, obj, ctx.content, inter, work).is_none()
}

/// The slots of seat `seat` (a mask, for `ObjectInstance::pick_slot`) with a footstool in front
/// (content `footstool`: a pouf, an ottoman), where the feet go up on it (the renderer's
/// `footstoolFor`): its middle 0.3–1.2 m ahead of the seat's place and at most 0.35 m to one
/// side. Someone sitting down alone takes that seat.
pub(crate) fn footstool_slots(content: &Content, objects: &[ObjectInstance], seat: u32) -> u32 {
    let o = &objects[seat as usize];
    let def = &content.objects[o.def];
    if def.slots < 2 || def.footstool || !def.interactions.iter().any(|i| i.pose == Pose::Sit) {
        return 0;
    }
    let a = o.angle();
    let (fwd, right) = ([a.sin(), a.cos()], [a.cos(), -a.sin()]);
    let n = (def.slots as usize).min(32);
    let places: [[f32; 2]; 32] =
        std::array::from_fn(|k| if k < n { o.place_at(def, k) } else { [0.0; 2] });
    objects
        .iter()
        .filter(|s| content.objects[s.def].footstool && !s.broken())
        .fold(0, |mask, s| {
            let c = s.centre(content);
            let s = [
                c[0] + s.offset[0] as f32 / 100.0,
                c[1] + s.offset[1] as f32 / 100.0,
            ];
            (0..n).fold(mask, |mask, k| {
                let d = [s[0] - places[k][0], s[1] - places[k][1]];
                let ahead = d[0] * fwd[0] + d[1] * fwd[1];
                let side = d[0] * right[0] + d[1] * right[1];
                if (0.3..=1.2).contains(&ahead) && side.abs() <= 0.35 {
                    mask | 1 << k
                } else {
                    mask
                }
            })
        })
}

/// Whether `sim` (of `age`) could only take the upper bunk of `obj` (the lower one someone
/// else's) and may not (`ObjectDef::upper_stages`): an elder, say.
pub(crate) fn upper_barred(content: &Content, obj: &ObjectInstance, sim: u32, age: f32) -> bool {
    let def = &content.objects[obj.def];
    def.bunk
        && !stage_in(def.upper_stages, content.life.stage(age))
        && obj.users[0].is_some_and(|u| u != sim)
}

/// Whether one of `usable` is a bed free for `sim` tonight: working, a place on it free for them
/// (or theirs) and not kept for a couple (`privacy::kept_for_couple`), for their age, and not one
/// they couldn't get to a moment ago; a spare one (`Interaction::spare`: the sofa) only if `spare`,
/// and then one somebody only sits on for now (watching TV on the sofa) counts as free: they'll get
/// up, nobody sleeps there.
pub(crate) fn has_bed(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    usable: &[u32],
    spare: bool,
) -> bool {
    let content = ctx.content;
    usable.iter().map(|&o| &objects[o as usize]).any(|o| {
        let def = &content.objects[o.def];
        !o.broken()
            && !sim.couldnt_reach(o.id, ctx.tick)
            && !crate::privacy::kept_for_couple(ctx, objects, sim, o)
            && def.interactions.iter().enumerate().any(|(i, inter)| {
                inter.tags & content.day_rhythm.sleep_tags != 0
                    && (spare || !inter.spare)
                    && !inter.baby
                    && (o.can_use_for(sim.id, def, i)
                        || (inter.spare
                            && o.users()
                                .all(|u| ctx.briefs.get(u as usize).is_none_or(|b| !b.lying))))
                    && unfit(content, sim.age, inter).is_none()
                    && !upper_barred(content, o, sim.id, sim.age)
            })
    })
}

/// What the player may have `sim` use on `plot`: as `access`, but their own home and public
/// places stay theirs to use while they're a guest somewhere.
pub(crate) fn order_access(
    households: &[Household],
    plots: &[Plot],
    sim: &Sim,
    plot: Option<u32>,
) -> Access {
    let home = households[sim.household as usize].plot;
    match plot {
        None => Access::Full,
        Some(p) if plots[p as usize].public || home == Some(p) => Access::Full,
        _ => access_in(households, plots, sim, plot),
    }
}

/// `access` outside a step.
pub(crate) fn access_in(
    households: &[Household],
    plots: &[Plot],
    sim: &Sim,
    plot: Option<u32>,
) -> Access {
    let home = households[sim.household as usize].plot;
    // A guest stays at the host's place until it's time to go home.
    if let Some(visit) = sim.visiting {
        return if plot == Some(visit.plot) {
            Access::Guest
        } else {
            Access::None
        };
    }
    match plot {
        None => Access::Full,
        Some(p) if plots[p as usize].public || home == Some(p) => Access::Full,
        Some(p) if sim.visiting.is_some_and(|v| v.plot == p) => Access::Guest,
        Some(_) => Access::None,
    }
}

/// The first thing `inter` requires that no working object on `object`'s plot provides, if any
/// (content `requires` / `provides`): a table's coffee needs a coffee machine. With `index`
/// (current during a tick) only that plot's objects are looked at.
pub(crate) fn missing_requirement<'a>(
    content: &'a Content,
    object_plot: &[Option<u32>],
    index: Option<&ObjectIndex>,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> Option<&'a str> {
    if inter.requires == 0 {
        return None;
    }
    let plot = object_plot.get(object as usize).copied().flatten();
    let provides = |o: &ObjectInstance| {
        if o.broken() {
            0
        } else {
            content.objects[o.def].provides
        }
    };
    let provided = match (plot, index) {
        (Some(p), Some(index)) => index
            .on(p)
            .iter()
            .fold(0, |m, &o| m | provides(&objects[o as usize])),
        _ => objects
            .iter()
            .filter(|o| object_plot.get(o.id as usize).copied().flatten() == plot)
            .fold(0, |m, o| m | provides(o)),
    };
    let missing = inter.requires & !provided;
    if missing == 0 {
        return None;
    }
    (0..content.tags.len())
        .find(|&i| missing & (1 << i) != 0)
        .map(|i| content.tags[i].as_str())
}

/// Whether `sim` could start `inter` on `object` now as far as what it requires and its
/// conditions go (`missing_requirement`, `unmet`).
pub(crate) fn requirements_met(
    ctx: &Ctx,
    sim: &Sim,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> bool {
    missing_requirement(
        ctx.content,
        ctx.object_plot,
        Some(ctx.index),
        objects,
        object,
        inter,
    )
    .is_none()
        && conditions_met(ctx, sim, &objects[object as usize], inter)
        && !closed_now(ctx, objects, object, inter)
}

/// Whether it's sold at a post that's closed now, or needs fetching from one (`staff::closed`).
pub(crate) fn closed_now(
    ctx: &Ctx,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> bool {
    closed_at(ctx, objects, object, inter, ctx.hour)
}

/// `closed_now` at `hour` of the day.
pub(crate) fn closed_at(
    ctx: &Ctx,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
    hour: f32,
) -> bool {
    let plot = ctx.object_plot.get(object as usize).copied().flatten();
    let on = plot.map_or(&[][..], |p| ctx.index.on(p));
    crate::staff::closed(ctx.content, ctx.staff, objects, on, object, inter, hour).is_some()
}

impl World {
    /// Advances one real-time step (1 / TICKS_PER_SECOND s) at the current speed; at super
    /// speed, all of its most ticks (60 while it stays quiet at home, else `SUPER_BUSY`).
    pub fn advance(&mut self) {
        self.start_step();
        while self.advance_more() {}
    }

    /// Starts a real-time step: the speed's ticks, or at super speed none yet (`advance_more`
    /// runs them, as many as the caller has time for). With `auto_fast` on, the game goes to super
    /// speed as the household falls quiet, and drops back to the speed chosen before it once it
    /// isn't quiet at home (a super speed the player picked stays). The ticks it ran.
    pub fn start_step(&mut self) -> u32 {
        self.step_ticks = 0;
        if self.speed == 0 {
            return 0;
        }
        let calm = self.calm();
        if self.speed == SUPER_SPEED && self.auto_super && !calm {
            self.speed = self.resume_speed;
            self.auto_super = false;
        } else if self.auto_fast && calm && !self.was_calm && self.speed < SUPER_SPEED {
            self.resume_speed = self.speed;
            self.speed = SUPER_SPEED;
            self.auto_super = true;
        }
        self.was_calm = calm;
        if self.speed == SUPER_SPEED {
            return 0;
        }
        let ticks = TICKS_PER_STEP[self.speed as usize];
        for _ in 0..ticks {
            self.tick_once();
        }
        self.step_ticks = ticks;
        ticks
    }

    /// One more tick of a super-speed step, if the game is at super speed and the step hasn't run
    /// its most: `TICKS_PER_STEP[SUPER_SPEED]` while it's quiet at home, else `SUPER_BUSY` (none
    /// more once the quiet ends, for a super speed gone to by itself: it drops back at the next
    /// step). Whether it ran.
    pub fn advance_more(&mut self) -> bool {
        if self.speed != SUPER_SPEED || self.step_ticks >= TICKS_PER_STEP[SUPER_SPEED as usize] {
            return false;
        }
        // (The step's start looked before its first tick; under 30 a step there is nothing to ask.)
        if (self.step_ticks >= SUPER_BUSY || (self.auto_super && self.step_ticks > 0))
            && !self.calm()
        {
            return false;
        }
        self.tick_once();
        self.step_ticks += 1;
        true
    }

    /// Ticks a real step runs now: the speed's, or at super speed what its last step ran.
    pub fn step_rate(&self) -> u32 {
        if self.speed == SUPER_SPEED {
            self.step_ticks.max(1)
        } else {
            TICKS_PER_STEP[self.speed as usize]
        }
    }

    /// A baby crying (content `dayRhythm.babyWakes`) in the crib at home with every grown-up of
    /// the household there asleep wakes one of them: family first, then someone not worn out,
    /// then whoever's turn it is (woken longest ago: partners take turns), then the most rested.
    /// At night, a grown-up there who's up and free to drop what they're doing
    /// (`free_for_baby`) comes at once instead (a cook nearly done with a step finishes it
    /// first): family first, then the nearest; with nobody up who can (in the shower, told to
    /// do something, about to leave for work), one asleep is woken. They do the crib's care for
    /// each need that's low (the woken go back to bed after: night still picks it); and the same
    /// baby calls nobody again for a while. In the daytime, with someone at home awake, they see
    /// to the baby themselves (autonomy, `pick_autonomous`).
    fn wake_for_crying_babies(&mut self) {
        let Some(rules) = self.content.day_rhythm.baby_wakes else {
            return;
        };
        for b in 0..self.sims.len() {
            let Some((carer, tasks)) = self.woken_for(b, &rules) else {
                continue;
            };
            let content = &self.content;
            let sim = &mut self.sims[carer];
            let woken = sim.asleep(content);
            // A cook halfway through a step finishes it first (`cooking::called_away`).
            if !crate::cooking::called_away(sim, content, &mut self.objects) {
                end_activity(sim, content, &mut self.objects);
            }
            sim.follow_up = None;
            let care = tasks.len();
            for task in tasks.into_iter().rev() {
                sim.queue.push_front(task);
            }
            trim_queue(&mut sim.queue, care);
            if woken {
                sim.night_woken = self.tick;
                if let Some(f) = rules.feeling {
                    social::add_feeling(&mut sim.feelings, f, &content.feelings, self.tick);
                }
            }
            self.sims[b].night_woken = self.tick;
        }
    }

    /// Who sees to baby `b`'s crying (`wake_for_crying_babies`), and the care they do.
    fn woken_for(&self, b: usize, rules: &crate::content::BabyWakes) -> Option<(usize, Vec<Task>)> {
        let content = &self.content;
        let baby = &self.sims[b];
        let cooling = baby.night_woken != 0 && self.tick < baby.night_woken + rules.cooldown_ticks;
        if !baby.here() || baby.away_until.is_some() || !content.life.baby(baby.age) || cooling {
            return None;
        }
        let wants = |n: usize, below: f32| rules.needs & (1 << n) != 0 && baby.needs[n] < below;
        if !(0..content.needs.len()).any(|n| wants(n, rules.below)) {
            return None;
        }
        let h = baby.household as usize;
        let plot = self.households[h]
            .plot
            .filter(|_| self.autonomy && self.households[h].free_will)?;
        // In a crib at home, with care for what they're crying for.
        let crib = self.objects.iter().find(|o| {
            o.users().any(|u| u as usize == b) && self.object_plot[o.id as usize] == Some(plot)
        })?;
        let def = &content.objects[crib.def];
        let care = |i: &crate::content::Interaction, below: f32| {
            i.cares() && (0..content.needs.len()).any(|n| i.care[n] > 0.0 && wants(n, below))
        };
        if !def.interactions.iter().any(|i| care(i, rules.below)) || !crib.has_free_slot(def.slots)
        {
            return None;
        }
        // The grown-ups at home.
        let home: Vec<usize> = (0..self.sims.len())
            .filter(|&j| {
                let s = &self.sims[j];
                let (x, z) = s.tile();
                s.here()
                    && s.household as usize == h
                    && s.away_until.is_none()
                    && s.visiting.is_none()
                    && s.adult(content)
                    && self.plot_at(x, z) == Some(plot)
            })
            .collect();
        // Nothing to do if someone's already doing the crib's care (or on their way to).
        let tending = |t: &Task| matches!(t.kind, TaskKind::Use { object, interaction } if object == crib.id && def.interactions[interaction].cares());
        if home.iter().any(|&j| {
            self.sims[j].current().is_some_and(|a| tending(&a.task))
                || self.sims[j].queue.iter().any(tending)
        }) {
            return None;
        }
        let carer = if home.iter().all(|&j| self.sims[j].asleep(content)) {
            self.to_wake(b, &home)?
        } else {
            // Up at night: whoever's free drops what they're doing; by day, autonomy.
            let rhythm = &content.day_rhythm;
            if !rhythm.is_night(crate::clock::hour(self.tick), rhythm.wake_hour) {
                return None;
            }
            let family = |j: usize| self.relationships.kin(j, b) != crate::social::Kin::None;
            let (cx, cz) = crib.front_tile(content);
            let far = |j: usize| {
                let p = self.sims[j].pos;
                (p[0] - cx as f32 - 0.5).powi(2) + (p[1] - cz as f32 - 0.5).powi(2)
            };
            let up = home
                .iter()
                .copied()
                .filter(|&j| self.free_for_baby(j, rules))
                .max_by(|&x, &y| family(x).cmp(&family(y)).then(far(y).total_cmp(&far(x))));
            up.or_else(|| self.to_wake(b, &home))?
        };
        let tasks = def
            .interactions
            .iter()
            .enumerate()
            .filter(|(_, i)| care(i, rules.tend_below))
            .map(|(i, _)| Task {
                kind: TaskKind::Use {
                    object: crib.id,
                    interaction: i,
                },
                directed: false,
                routine: false,
            })
            .collect();
        Some((carer, tasks))
    }

    /// Which of the grown-ups at `home` who are asleep baby `b`'s crying wakes
    /// (`wake_for_crying_babies`).
    fn to_wake(&self, b: usize, home: &[usize]) -> Option<usize> {
        let content = &self.content;
        let rhythm = &content.day_rhythm;
        let rest = |j: usize| rhythm.rest_need.map_or(1.0, |n| self.sims[j].needs[n]);
        let key = |j: usize| {
            let family = self.relationships.kin(j, b) != crate::social::Kin::None;
            (
                family,
                rest(j) >= rhythm.exhausted_below,
                std::cmp::Reverse(self.sims[j].night_woken),
            )
        };
        home.iter()
            .copied()
            .filter(|&j| self.sims[j].asleep(content))
            .max_by(|&x, &y| key(x).cmp(&key(y)).then(rest(x).total_cmp(&rest(y))))
    }

    /// Whether grown-up `j`, up at home, can see to a crying baby now: doing nothing, or
    /// something they picked themselves (on their way to bed too; eating, the plate left where
    /// it is; cooking, with at most `BabyWakes::finish_minutes` of the step left, see
    /// `cooking::called_away`; not the toilet or shower, washing up, or a conversation), with
    /// nothing the player asked for next, not pressed themselves, and not leaving for work or
    /// school within `DayRhythm::ready_minutes`.
    fn free_for_baby(&self, j: usize, rules: &crate::content::BabyWakes) -> bool {
        let content = &self.content;
        let sim = &self.sims[j];
        if sim.asleep(content) || sim.engaged_with.is_some() || sim.pressed(content) {
            return false;
        }
        let picked = sim.current().is_none_or(|a| {
            let drop = match a.task.kind {
                TaskKind::Use { .. }
                | TaskKind::MoveTo { .. }
                | TaskKind::Clean { .. }
                | TaskKind::Eat { .. } => true,
                TaskKind::Cook { .. } => {
                    crate::cooking::step_left(sim).is_some_and(|m| m <= rules.finish_minutes)
                }
                _ => false,
            };
            !a.task.directed && drop && a.tags & content.social_rules.busy_tags == 0
        });
        let next = sim
            .queue
            .front()
            .is_some_and(|t| t.directed || matches!(t.kind, TaskKind::Work));
        let ready = social::minutes_to_ticks(content.day_rhythm.ready_minutes);
        let leaving = leaves_by(sim, content, self.tick, self.tick + ready);
        picked && !next && !leaving
    }

    pub fn tick_once(&mut self) {
        self.sync_tile_arrays();
        // Time moves on: edits can no longer be taken back.
        self.undo.clear();
        self.redo.clear();
        // (An edit still under way can't be taken back now: nothing more to keep of it.)
        self.lot.log_off();
        if self.rooms_version != self.room_layout_version
            || self.rooms_dirty
            || self.rooms_things
            || self.rooms_rescore
        {
            crate::profile::phase("update_rooms", || self.update_rooms());
        } else if self.tick.is_multiple_of(ROOM_REFRESH_TICKS) {
            crate::profile::phase("rescore_rooms", || self.rescore_rooms());
        }
        crate::profile::phase("refresh_reach", || self.refresh_reach());
        crate::profile::phase("refresh_index", || self.refresh_index());
        self.travel.note_doors(&self.plots);
        // A new game's first step: the player hears of anyone without a bed (told as the world was
        // made, it would be the story so far, which the view doesn't show as news).
        if self.tick == 0 {
            for h in 0..self.households.len() {
                crate::lifecycle::notify_missing_beds(self, h);
            }
        }
        self.tick += 1;
        let tick = self.tick;
        self.search_mark = crate::path::nodes_searched();

        crate::profile::phase("partners::refresh", || crate::partners::refresh(self));
        crate::profile::phase("staff::update", || crate::staff::update(self));
        self.build_briefs(tick);
        crate::profile::phase("privacy::refresh", || crate::privacy::refresh(self));
        crate::profile::phase("lod::plan", || crate::lod::plan(self));
        let steps_start = crate::profile::start();
        // Those who go first when the tick's searches run over budget: the player's household
        // and whoever is watched (see `SEARCH_BUDGET`).
        // (At full detail, as `lod::plan` found them: the player's household always is.)
        let first = std::mem::take(&mut self.first);
        let (search_mark, budget) = (self.search_mark, self.search_budget);
        let hour = crate::clock::hour(tick);
        let mut work = std::mem::take(&mut self.work);
        self.fill_work_waiting(&mut work);
        let room_in_town = self.room_in_town();
        {
            let World {
                content,
                lot,
                objects,
                items,
                sims,
                blocked,
                rng,
                relationships,
                briefs,
                members,
                autonomy,
                households,
                plots,
                tile_plot,
                object_plot,
                object_index,
                exits,
                travel,
                cars,
                traffic,
                taxis,
                room_score,
                dirt,
                stairs,
                reach,
                nav_version,
                nav_log,
                shut,
                rooms,
                staff,
                paths,
                ..
            } = self;
            let base = Ctx {
                room_score,
                dirt,
                content,
                nav: NavGrid {
                    lot,
                    blocked,
                    stairs,
                    kept_off: &travel.junction_tiles,
                    steps: Some(reach),
                },
                reach,
                nav_version: *nav_version,
                nav_log,
                rels: relationships,
                briefs,
                members,
                households,
                plots,
                tile_plot,
                object_plot,
                index: object_index,
                exits,
                travel,
                cars,
                lights: &traffic.lights,
                taxis,
                autonomy: *autonomy,
                room_in_town,
                hour,
                day: crate::clock::day(tick),
                minute: crate::clock::minute_of_day(tick),
                tick,
                work: &work,
                shut,
                rooms,
                staff,
                paths,
                others: Others::default(),
            };
            // (In lots nobody watches, only those whose turn it is; see `lod.rs`.)
            for i in 0..sims.len() {
                let (before, rest) = sims.split_at_mut(i);
                let Some((sim, after)) = rest.split_first_mut() else {
                    break;
                };
                if !sim.here() || sim.steps == 0 {
                    continue;
                }
                let ctx = Ctx {
                    others: Others { before, after },
                    ..base
                };
                // The tick's searches over budget: one who may look for a way now waits for
                // the next tick, unless they go first or waited last tick.
                if first.get(sim.id as usize) == Some(&false)
                    && !sim.deferred
                    && crate::path::nodes_searched() - search_mark > budget
                    && may_search(sim, &ctx)
                {
                    sim.owed += sim.steps;
                    sim.steps = 0;
                    sim.deferred = true;
                    continue;
                }
                sim.deferred = false;
                step_sim(sim, &ctx, objects, items, rng);
            }
        }
        self.work = work;
        self.first = first;
        crate::profile::end("step_sim", steps_start);
        crate::profile::phase("partners::update", || crate::partners::update(self));
        crate::profile::phase("broadcast", || self.broadcast());
        crate::profile::phase("apply_messes", || self.apply_messes());
        crate::profile::phase("carry::after_step", || crate::carry::after_step(self));
        crate::profile::phase("setup::update", || crate::setup::update(self));
        crate::profile::phase("post::after_step", || crate::post::after_step(self));
        crate::profile::phase("wake_for_crying_babies", || self.wake_for_crying_babies());
        crate::profile::phase("conversation::update", || conversation::update(self));
        crate::profile::phase("privacy::after_step", || crate::privacy::after_step(self));
        crate::profile::phase("hosting::update", || crate::hosting::update(self));
        crate::profile::phase("phone::update", || crate::phone::update(self));
        crate::profile::phase("life::update", || crate::life::update(self));
        crate::profile::phase("lifecycle::update", || crate::lifecycle::update(self));
        crate::profile::phase("traffic::begin", || crate::traffic::begin(self));
        crate::profile::phase("cars::update", || crate::cars::update(self));
        crate::profile::phase("taxi::update", || crate::taxi::update(self));
        crate::profile::phase("planner::update", || crate::planner::update(self));
        crate::profile::phase("cooking::after_step", || crate::cooking::after_step(self));
        crate::profile::phase("lights::update", || crate::lights::update(self));
        crate::profile::phase("lights::update_while", || crate::lights::update_while(self));
        crate::profile::phase("curtains::update", || crate::curtains::update(self));
        if self.tick.is_multiple_of(TICKS_PER_HOUR) {
            crate::rooms::opinions(self);
        }

        self.update_after_step(tick);
    }

    /// The tick's `SimBrief` for each resident, into `self.briefs` (`members`, and those near
    /// them, `tick_once`).
    fn build_briefs(&mut self, tick: u64) {
        let briefs_start = crate::profile::start();
        let mut briefs = std::mem::take(&mut self.briefs);
        briefs.clear();
        for s in &self.sims {
            let (x, z) = s.tile();
            let plot = self.plot_at(x, z);
            let available = s.here() && s.available_for_social(&self.content);
            briefs.push(SimBrief {
                pos: s.pos,
                available,
                gender_ix: s.gender_ix,
                household: s.household,
                plot: plot.filter(|_| s.here()),
                away: s.away_until.is_some() || !s.here(),
                leaving: crate::life::leaving_home(self, s),
                baby: s.here() && self.content.life.baby(s.age),
                needs: s.needs,
                here: s.here(),
                partnered: self.relationships.partner_of(s.id as usize).is_some(),
                adult: s.adult(&self.content),
                age: s.age,
                conversing: None,
                cleaning: s.current().and_then(|a| match a.task.kind {
                    TaskKind::Clean { x, z } => Some((x, z)),
                    TaskKind::Use { .. } => s.sweep,
                    _ => None,
                }),
                seated: s.pose == Pose::Sit && !s.is_moving(),
                lying: s.pose == Pose::Lie,
                eating: s.eating_seated(),
                game: s.game,
                // (`partners::free_now`, its start just above.)
                free: {
                    let free = available && crate::partners::free_besides(&self.content, s, tick);
                    debug_assert_eq!(free, crate::partners::free_now(&self.content, s, tick));
                    free
                },
                bound_for_car: s.bound_for_car.filter(|_| {
                    s.current()
                        .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
                }),
                trip_car: s.bound_for_car.filter(|_| {
                    s.current().is_some_and(|a| {
                        matches!(a.task.kind, TaskKind::Visit { .. } | TaskKind::GoHome)
                    })
                }),
                bike: s.commute.and_then(|c| c.bike),
                bike_spot: s.bike_spot.map(|(at, _)| at),
                guest_at: s.visiting.map(|v| v.plot),
                eats_at: match s.current().map(|a| a.task.kind) {
                    Some(TaskKind::Eat { seat, stage: 1, .. }) => seat,
                    _ => None,
                },
                carried: s.carried_by.is_some(),
                private: crate::privacy::private_use(&self.content, &self.objects, s),
            });
        }
        // Who's in a conversation, and whose (for joining in).
        for (i, s) in self.sims.iter().enumerate() {
            if let Some((t, social, ..)) = s.conversation(&self.content) {
                briefs[i].conversing = Some((i as u32, social));
                briefs[t as usize].conversing = Some((i as u32, social));
            }
        }
        for (i, s) in self.sims.iter().enumerate() {
            if let Some(l) = s.listening_to() {
                briefs[i].conversing = briefs[l as usize].conversing.filter(|c| c.0 == l);
            }
        }
        self.members.build(&briefs);
        self.briefs = briefs;
        crate::profile::end("briefs", briefs_start);
    }

    /// The tick's tail: feelings fade and buffs refresh, relationships decay by the day, crops
    /// grow and machines run (`tick_once`).
    fn update_after_step(&mut self, tick: u64) {
        for s in self.sims.iter_mut().filter(|s| s.steps > 0) {
            s.feelings.retain(|m| m.expires > tick);
            if self.content.buff_feelings {
                s.refresh_buffs(&self.content);
            }
        }
        if tick.is_multiple_of(TICKS_PER_DAY) {
            self.relationships.decay(&self.content.social_rules);
        }
        if tick.is_multiple_of(GROWTH_STEP_MINUTES * TICKS_PER_MINUTE) {
            self.grow(GROWTH_STEP_MINUTES as f32);
        }
        if tick.is_multiple_of(crate::machines::STEP_MINUTES * TICKS_PER_MINUTE) {
            crate::machines::run(self, crate::machines::STEP_MINUTES as f32);
        }
    }

    /// Gives nearby residents a share of what a use gives (content `broadcast`: hearing the TV
    /// from the sofa, or while eating at the table). Only positive gains carry, and not to the
    /// resident using it, who gets the whole of it.
    pub(crate) fn broadcast(&mut self) {
        let mut scratch = std::mem::take(&mut self.broadcast_scratch);
        self.broadcast_with(&mut scratch);
        self.broadcast_scratch = scratch;
    }

    /// `broadcast_with`'s sources: what each resident using something that carries
    /// (`broadcast`) is using, where, and what it gives.
    fn broadcast_sources(&self, sources: &mut Vec<BroadcastSource>) {
        let content = &self.content;
        sources.clear();
        for sim in &self.sims {
            if !sim.here() || sim.away_until.is_some() {
                continue;
            }
            let Some(act) = sim.current() else { continue };
            if !matches!(act.phase, Phase::Using { .. }) {
                continue;
            }
            let TaskKind::Use {
                object,
                interaction,
            } = act.task.kind
            else {
                continue;
            };
            let Some(obj) = self.objects.get(object as usize) else {
                continue;
            };
            let inter = &content.objects[obj.def].interactions[interaction];
            if inter.broadcast <= 0.0 {
                continue;
            }
            let pos = obj.centre(content);
            let (x, z) = (pos[0].floor() as i32, pos[1].floor() as i32);
            let ground = self.ground_z(z);
            let room = if self.lot.in_bounds(x, ground) {
                self.lot.room_at(x, ground)
            } else {
                crate::lot::OUTDOORS
            };
            let quality = content.object_rules.quality_factor(obj.quality);
            sources.push(BroadcastSource {
                object,
                room,
                storey: self.storey_of(z),
                plot: self.object_plot.get(object as usize).copied().flatten(),
                pos,
                gains: inter.gain_per_minute.map(|g| g * quality),
                share: inter.broadcast,
                range: inter.broadcast_range,
                user: sim.id,
            });
        }
    }

    /// `broadcast` with its buffers (`BroadcastScratch`) out of `self`.
    fn broadcast_with(&mut self, scratch: &mut BroadcastScratch) {
        let BroadcastScratch {
            sources,
            rooms,
            storeys,
            plots,
            asleep,
            heard,
        } = scratch;
        self.broadcast_sources(sources);
        if sources.is_empty() {
            return;
        }
        let content = &self.content;
        rooms.clear();
        rooms.extend(self.sims.iter().map(|s| {
            let (x, z) = s.tile();
            let ground = self.ground_z(z);
            if self.lot.in_bounds(x, ground) {
                self.lot.room_at(x, ground)
            } else {
                crate::lot::OUTDOORS
            }
        }));
        storeys.clear();
        storeys.extend(self.sims.iter().map(|s| self.storey_of(s.tile().1)));
        plots.clear();
        plots.extend(self.sims.iter().map(|s| {
            let (x, z) = s.tile();
            self.plot_at(x, z)
        }));
        asleep.clear();
        asleep.extend(self.sims.iter().map(|s| s.asleep(content)));
        for (i, sim) in self.sims.iter_mut().enumerate() {
            if !sim.here() || sim.away_until.is_some() || sim.steps == 0 || asleep[i] {
                continue;
            }
            let minutes = sim.minutes();
            // Two residents watching the same screen are one source, not two.
            heard.clear();
            for src in sources.iter() {
                if src.user == sim.id
                    || heard.contains(&src.object)
                    || (src.plot.is_some() && plots[i] != src.plot)
                {
                    continue;
                }
                // Another resident watching the same screen has their own share of it already.
                if let Some(act) = sim.current()
                    && let TaskKind::Use { object, .. } = act.task.kind
                    && object == src.object
                {
                    continue;
                }
                if storeys[i] != src.storey {
                    continue;
                }
                let same_room = src.room != crate::lot::OUTDOORS && rooms[i] == src.room;
                let near = (sim.pos[0] - src.pos[0]).hypot(sim.pos[1] - src.pos[1]) <= src.range;
                if !same_room && !near {
                    continue;
                }
                heard.push(src.object);
                for n in 0..content.needs.len() {
                    let gain = src.gains[n] * src.share * minutes * sim.mods.need_gain[n];
                    if gain > 0.0 {
                        sim.needs[n] = (sim.needs[n] + gain).clamp(0.0, 1.0);
                    }
                }
            }
        }
    }

    /// Crops grow by `minutes`: fallow ground is planted again, what's planted ripens. A crop
    /// that shows another stage changes how the lot looks.
    fn grow(&mut self, minutes: f32) {
        let mut changed = false;
        for o in &mut self.objects {
            let Some(g) = self.content.objects[o.def].grows else {
                continue;
            };
            if o.growth >= 1.0 {
                continue;
            }
            let before = o.growth_stage();
            o.growth = if o.growth < 0.0 {
                // Fallow, then planted afresh (from nothing: 0).
                let fallow = o.growth + minutes / (g.fallow_days.max(1.0 / 1440.0) * 1440.0);
                if fallow >= 0.0 { 0.0 } else { fallow }
            } else {
                (o.growth + minutes / (g.days * 1440.0)).min(1.0)
            };
            changed |= o.growth_stage() != before;
        }
        if changed {
            self.structure_version = self.structure_version.wrapping_add(1);
            self.rooms_need_clean_rescore();
        }
        self.pile_laundry(minutes);
    }

    /// Dirty washing piles up over `minutes`: each household's, as much as content `laundry` says
    /// for each of its residents in town, up to the most it says.
    fn pile_laundry(&mut self, minutes: f32) {
        let Some(rules) = self.content.laundry else {
            return;
        };
        for s in self.sims.iter().filter(|s| s.here()) {
            if let Some(h) = self.households.get_mut(s.household as usize) {
                h.laundry[0] = (h.laundry[0] + rules.per_day * minutes / 1440.0).min(rules.most);
            }
        }
    }

    /// What each household has to work on (`WorkOn` bits, by household): dirty dishes or food
    /// gone off of theirs, a load of dirty washing, a load washed.
    pub(crate) fn work_waiting(&self) -> Vec<u8> {
        let mut work = Vec::new();
        self.fill_work_waiting(&mut work);
        work
    }

    /// `work_waiting`, into `work`.
    fn fill_work_waiting(&self, work: &mut Vec<u8>) {
        use crate::content::WorkOn;
        work.clear();
        work.extend(self.households.iter().map(|h| {
            let has = |kind: WorkOn, loads: f32| if loads >= 1.0 { kind.bit() } else { 0 };
            has(WorkOn::Laundry, h.laundry[0])
                | has(WorkOn::Washed, h.laundry[1])
                | has(WorkOn::Dried, h.laundry[2])
        }));
        for it in self
            .items
            .list
            .iter()
            .filter(|it| it.to_clear(&self.content))
        {
            if let Some(w) = work.get_mut(it.household as usize) {
                *w |= WorkOn::Dishes.bit();
            }
        }
    }
}
