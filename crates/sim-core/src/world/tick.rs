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
        end_activity(sim, ctx.content, objects);
        if !directed {
            let retry = if matches!(task.kind, TaskKind::Social { .. } | TaskKind::Join { .. }) {
                PERSON_UNREACHABLE_TICKS
            } else {
                social::minutes_to_ticks(ctx.content.autonomy.retry_minutes)
            };
            sim.remember_unreachable(task.kind, ctx.tick + retry);
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
