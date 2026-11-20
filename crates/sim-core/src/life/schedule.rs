//! The day: the job market, school and shift schedules, setting off and missing a shift.

use super::*;

pub(crate) fn update(w: &mut World) {
    let tick = w.tick;
    let day = clock::day(tick);
    let hour = clock::hour(tick);
    let mut refreshed = false;
    let residents = crate::profile::start();
    // (Only those taking a turn: `World::turns`, unless someone came to town since.)
    let turns = std::mem::take(&mut w.turns);
    let fresh = w.turns_fresh;
    debug_assert!(
        !fresh
            || (0..w.sims.len()).all(|i| turns.binary_search(&(i as u32)).is_ok()
                || !w.sims[i].here()
                || w.sims[i].steps == 0)
    );
    let all = if fresh { 0 } else { w.sims.len() };
    let listed = if fresh { &turns[..] } else { &[] };
    for i in listed.iter().map(|&i| i as usize).chain(0..all) {
        // (In lots nobody watches, on their turn; see `lod.rs`.)
        if !w.sims[i].here() || w.sims[i].steps == 0 {
            continue;
        }
        // A bike left out with no room by the place (`ride::park_spot`): where they got off,
        // facing as they did (a placement faces a quarter turn off its rider).
        if w.sims[i].bike_parked.is_some() && w.sims[i].bike_spot.is_none() {
            let sim = &mut w.sims[i];
            sim.bike_spot = sim
                .bike_parked
                .map(|at| (at, sim.yaw - std::f32::consts::FRAC_PI_2));
        }
        // Interactions started this tick are paid from the Sim's own household (guests too).
        let spend = std::mem::take(&mut w.sims[i].pending_spend);
        w.households[w.sims[i].household as usize].funds -= spend;
        // A load of washing taken up (put in the washer, the dryer, hung out, folded), or one
        // left to work on next (taken out washed, or dry).
        let worked = std::mem::take(&mut w.sims[i].worked);
        let made = std::mem::take(&mut w.sims[i].made);
        let most = w.content.laundry.map_or(1.0, |l| l.most);
        crate::machines::settle(
            &mut w.households[w.sims[i].household as usize].laundry,
            worked,
            made,
            most,
        );
        // (Pressed clothes put on: an outfit's share of the ironed washing.)
        if std::mem::take(&mut w.sims[i].wore_pressed)
            && let Some(p) = w.content.laundry.and_then(|l| l.pressed)
        {
            crate::machines::wear_pressed(
                &mut w.households[w.sims[i].household as usize].laundry,
                p.outfits,
            );
        }
        if let Some(transition) = w.sims[i].transition.take() {
            apply_transition(w, i, transition);
        }
        if w.sims[i].away_until.is_some() {
            // (Done, and waiting for their car home or in it: `cars.rs` brings them back; or the
            // work taxi, `taxi.rs`.) Done now: home in the car (`cars::off_work`), by taxi
            // (`taxi::off_work`), or on their own.
            if !crate::cars::homeward(w, i)
                && w.sims[i].taxi.is_none()
                && at_work(w, i)
                && !crate::cars::off_work(w, i)
                && !crate::taxi::off_work(w, i)
            {
                w.sims[i].queue.push_back(Task {
                    kind: TaskKind::GoHome,
                    directed: false,
                    routine: false,
                });
            }
        } else {
            // (Commutes worked out for the first time or after a move, one a tick at least:
            // those with one to go by can wait a tick when the searches are over budget, and
            // those without one yet whose work or school is hours off (`unhurried`) wait for a
            // tick nobody else's is worked out in. The town's hires at the job market's hour
            // each worked theirs out in the same tick, a search across the town each: 35 of them
            // in 70 ms, on a fast machine.)
            let waits = refreshed
                && if w.sims[i].commute.is_some() {
                    w.over_search_budget()
                } else {
                    unhurried(w, i)
                };
            let mut fresh = None;
            if !waits {
                let before = w.sims[i].commute;
                let searched = crate::path::nodes_searched();
                fresh =
                    refresh_commute(w, i, day).then(|| crate::path::nodes_searched() - searched);
                refreshed |= w.sims[i].commute != before;
            }
            schedule_work(w, i, day, fresh);
            schedule_school(w, i, day, fresh);
            end_visit_if_due(w, i, hour);
        }
        childcare(w, i);
        report_skill_ups(w, i);
    }
    w.turns = turns;
    crate::profile::end("life::update/residents", residents);
    crate::profile::phase("life::update/doors", || doors(w));
    crate::profile::phase("life::update/rent_and_jobs", || {
        charge_rent(w, day);
        job_market(w, day);
    });
}

/// Once a day: unemployed residents with free will below the retirement age look for work, and
/// residents who have been unhappy at work for a while may quit.
fn job_market(w: &mut World, day: u32) {
    let Some(market) = w.content.career_rules.market.clone() else {
        return;
    };
    if !clock::at_minute(w.tick, market.hour * 60.0) || !w.autonomy {
        return;
    }
    let tick = w.tick;
    for i in 0..w.sims.len() {
        let h = w.sims[i].household as usize;
        if !w.households[h].free_will
            || w.sims[i].away_until.is_some()
            || !w.sims[i].here()
            || w.sims[i].retired
            || !w.sims[i].adult(&w.content)
        {
            continue;
        }
        if let Some(job) = &w.sims[i].job {
            let unhappy =
                job.shifts >= market.min_shifts_before_quit && job.satisfaction < market.quit_below;
            if unhappy && w.rng.next_f32() < market.quit_chance {
                let (career, level) = (job.career, job.level);
                let sim = &mut w.sims[i];
                sim.job = None;
                sim.job_search_from = day + market.cooldown_days;
                if let Some(m) = market.quit_feeling {
                    social::add_feeling(&mut sim.feelings, m, &w.content.feelings, tick);
                }
                w.events
                    .push_career(tick, EventKind::QuitJob, i, career, Some(level as i64));
                crate::cars::job_lost(&mut w.cars, i as u32);
            }
            continue;
        }
        // Nobody's hired from the retirement age on (they retire instead; a job the player gives
        // them is theirs to keep).
        let retirement_age = w
            .content
            .life
            .retire_at
            .is_some_and(|at| w.sims[i].age >= at);
        if day < w.sims[i].job_search_from || retirement_age {
            continue;
        }
        // Short on money, or set on finding work (a goal): look harder.
        let funds = w.households[h].funds;
        let weekly = weekly_costs(w, h).map_or(0, |(rent, bills)| rent + bills);
        let (keen, category) = crate::planner::job_search(&w.content, &w.sims[i]);
        let chance = market.daily_chance * keen * if funds < weekly * 2 { 2.0 } else { 1.0 };
        if w.rng.next_f32() >= chance {
            continue;
        }
        let Some((career, level)) = find_job(w, i, category).or_else(|| find_job(w, i, None))
        else {
            continue;
        };
        let sim = &mut w.sims[i];
        sim.job = Some(Job::new(career, level, day));
        if let Some(m) = market.hired_feeling {
            social::add_feeling(&mut sim.feelings, m, &w.content.feelings, tick);
        }
        w.events
            .push_career(tick, EventKind::JobFound, i, career, Some(level as i64));
        let household = w.sims[i].household;
        crate::cars::job_changed(&mut w.cars, &w.content, &w.sims, household);
    }
}

/// A job Sim `i` could get (in `category`, if set): for each career the best level they're
/// qualified for (the entry level on probation otherwise), weighed by pay with some chance.
fn find_job(w: &mut World, i: usize, category: Option<usize>) -> Option<(usize, usize)> {
    let World {
        content, sims, rng, ..
    } = w;
    let skills = &sims[i].skills;
    let mut candidates: Vec<(f32, (usize, usize))> = Vec::new();
    for (c, career) in content.careers.iter().enumerate() {
        if category.is_some_and(|cat| career.category != Some(cat)) {
            continue;
        }
        let level = career
            .levels
            .iter()
            .rposition(|l| skill_fit(l, skills) >= 0.0)
            .or_else(|| can_join(content, &career.levels[0], skills).then_some(0));
        let Some(level) = level else { continue };
        let def = &career.levels[level];
        let weekly_pay = (def.pay * def.days.count_ones() as i64).max(1) as f32;
        let score = weekly_pay.sqrt() * rng.range(0.5, 1.5);
        candidates.push((score, (c, level)));
    }
    crate::ai::choose(&mut candidates, rng.next_f32(), 3)
}

/// Today's school day for a pupil (a stage that goes to school, no job): when they leave, when
/// it starts, when it ends.
pub fn school_today(
    content: &Content,
    sim: &crate::world::Sim,
    tick: u64,
) -> Option<(u64, u64, u64)> {
    let rules = content.life.school.as_ref()?;
    if sim.job.is_some() || !content.life.stage_at(sim.age).is_some_and(|s| s.school) {
        return None;
    }
    let day = clock::day(tick);
    if rules.days & (1 << clock::weekday(day)) == 0 {
        return None;
    }
    let start = clock::tick_at(day, rules.start * 60.0)?;
    let leave = start.saturating_sub(ticks(content.career_rules.commute_minutes));
    Some((leave, start, start + ticks(rules.hours * 60.0)))
}

/// Off to school when it's time (once a day; too late, and it's skipped for today).
fn schedule_school(w: &mut World, i: usize, day: u32, fresh: Option<u64>) {
    if w.sims[i].school_day == day {
        return;
    }
    let Some((leave, start, _)) = school_today(&w.content, &w.sims[i], w.tick) else {
        return;
    };
    let leave = firm_leave(w, i, leave, start, day, fresh);
    // By car, when the drive takes it (`cars::lead_ticks`); by the work taxi, in time for it at the
    // kerb (`taxi::leave_at`).
    let leave = match crate::cars::lead_ticks(w, i, start) {
        0 => leave,
        // (By the drive's time, not the walk's: a way out on foot of hours had drivers off in the
        // night.)
        lead => start.saturating_sub(lead),
    };
    let by_taxi = crate::taxi::leave_at(w, i);
    let leave = by_taxi.unwrap_or(leave);
    let World {
        content,
        sims,
        objects,
        tick,
        ..
    } = w;
    let tick = *tick;
    let sim = &mut sims[i];
    // (Booked on a work taxi still to come, late as it may be: not a day missed.)
    if tick > start + ticks(content.career_rules.late_minutes) && by_taxi.is_none() {
        // A day missed.
        sim.school_day = day;
        if let Some(school) = &content.life.school {
            sim.grade = (sim.grade - school.missed).clamp(0.0, 100.0);
        }
        return;
    }
    if tick < leave {
        return;
    }
    if !heading_out(sim) && !in_a_car(sim) {
        set_off(sim, content, objects, tick);
    }
}

/// Leave for work when it's time; count a shift as missed if the Sim never left. The last
/// grown-up at home with a baby and no money for a babysitter stays home instead.
fn schedule_work(w: &mut World, i: usize, day: u32, fresh: Option<u64>) {
    let tick = w.tick;
    let sim = &w.sims[i];
    let Some(job) = sim.job.as_ref() else { return };
    if job.last_shift_day == day {
        return;
    }
    let Some((leave, start, _)) = shift_today(&w.content, job, &sim.skills, tick) else {
        return;
    };
    // (A little early, `early`: the walk firmed up as that comes near.)
    let early = early(sim.household);
    let leave = firm_leave(
        w,
        i,
        leave.saturating_sub(early),
        start.saturating_sub(early),
        day,
        fresh,
    );
    let sim = &w.sims[i];
    // By car, when the drive takes it (`cars::lead_ticks`); by the work taxi, in time for it at the
    // kerb (`taxi::leave_at`).
    let leave = match crate::cars::lead_ticks(w, i, start) {
        0 => leave,
        // (By the drive's time, not the walk's: a way out on foot of hours had drivers off in the
        // night.)
        lead => start.saturating_sub(lead + early),
    };
    let by_taxi = crate::taxi::leave_at(w, i);
    let leave = by_taxi.unwrap_or(leave);
    // (Booked on a work taxi still to come, late as it may be: not a shift missed.)
    if tick > start + ticks(w.content.career_rules.late_minutes) && by_taxi.is_none() {
        miss_shift(w, i, day);
        return;
    }
    if tick < leave || heading_out(sim) || in_a_car(sim) {
        return;
    }
    if let Some(baby) = stays_home_for(w, i) {
        // No shift today, and no pay: not held against them at work.
        if let Some(job) = w.sims[i].job.as_mut() {
            job.last_shift_day = day;
        }
        w.events.push(tick, EventKind::StayedHome, i, baby, None);
        return;
    }
    // Work comes first: drop what they're doing and go.
    let World {
        content,
        sims,
        objects,
        ..
    } = w;
    set_off(&mut sims[i], content, objects, tick);
}

/// Off to work or school now: whatever they're doing dropped, and a visit or an outing they're
/// on over, as coming home ends one (`Transition::Home`); not ended, it went to work with them,
/// and a cyclist rode on with a visit hours past its end (Snowdale, Hollyshaw, Stationgate).
fn set_off(
    sim: &mut Sim,
    content: &Content,
    objects: &mut [crate::world::ObjectInstance],
    tick: u64,
) {
    end_activity(sim, content, objects);
    sim.engaged_with = None;
    if sim.visiting.take().is_some() {
        sim.back_at = tick;
    }
    sim.queue.push_front(Task {
        kind: TaskKind::Work,
        directed: true,
        routine: false,
    });
}

/// Whether `sim` is at the wheel of a car on its way, or getting in or out of one: off to work
/// only once they're out of it (`schedule_work`). Called away mid-drive, a driver on a trip walked
/// off to work and left the car standing on the road for good with them still its driver
/// (Mirror Reach, full).
fn in_a_car(sim: &Sim) -> bool {
    sim.current.as_ref().is_some_and(|a| {
        matches!(
            a.phase,
            Phase::Driving | Phase::Boarding { .. } | Phase::Alighting { .. }
        )
    })
}

/// On their way to work or school (or about to set off).
pub(crate) fn heading_out(sim: &Sim) -> bool {
    matches!(
        sim.current.as_ref().map(|a| a.task.kind),
        Some(TaskKind::Work)
    ) || matches!(sim.queue.front().map(|t| t.kind), Some(TaskKind::Work))
}

/// A shift they never left for.
fn miss_shift(w: &mut World, i: usize, day: u32) {
    let World {
        content,
        sims,
        events,
        tick,
        cars,
        ..
    } = w;
    let tick = *tick;
    let sim = &mut sims[i];
    let Some(job) = sim.job.as_mut() else { return };
    job.last_shift_day = day;
    let rules = &content.career_rules;
    job.performance =
        (job.performance - rules.missed_penalty * grade_pace(rules, job.level)).max(0.0);
    job.missed = job.missed.saturating_add(1);
    if let Some(m) = content.career_rules.missed_feeling {
        social::add_feeling(&mut sim.feelings, m, &content.feelings, tick);
    }
    events.push(tick, EventKind::MissedWork, i, i, None);
    if let Some(market) = &content.career_rules.market
        && job.missed >= market.fire_after_missed
    {
        let (career, level) = (job.career, job.level);
        sim.job = None;
        sim.job_search_from = day + market.cooldown_days;
        if let Some(m) = market.fired_feeling {
            social::add_feeling(&mut sim.feelings, m, &content.feelings, tick);
        }
        events.push_career(tick, EventKind::Fired, i, career, Some(level as i64));
        crate::cars::job_lost(cars, i as u32);
    }
}

// ---- Childcare ----------------------------------------------------------------------------

/// Whether `task` takes someone whose home is `home` off that plot: to work, on a visit, over
/// to theirs after a call, or to use, repair or tidy something, walk somewhere, or chat with
/// or join someone anywhere else (a public place, a neighbour's, the street).
fn takes_off(w: &World, task: &TaskKind, home: u32) -> bool {
    let away = |plot: Option<u32>| plot != Some(home);
    let of = |sim: u32| {
        w.sims
            .get(sim as usize)
            .and_then(|t| w.plot_at(t.tile().0, t.tile().1))
    };
    match *task {
        TaskKind::Work => true,
        TaskKind::Visit { plot } => plot != home,
        TaskKind::Use { object, .. } | TaskKind::Repair { object } => {
            away(w.object_plot.get(object as usize).copied().flatten())
        }
        TaskKind::MoveTo { x, z } => away(w.plot_at(x.floor() as i32, z.floor() as i32)),
        TaskKind::Clean { x, z } => away(w.plot_at(x, z)),
        TaskKind::Social { target, .. } => away(of(target)),
        TaskKind::Join { leader } => away(of(leader)),
        TaskKind::Phone { action, answer, .. } => {
            w.content
                .phone
                .actions
                .get(action)
                .is_some_and(|a| a.meet == Some(crate::phone::Meet::There))
                && matches!(
                    answer,
                    crate::phone::Answer::Pending | crate::phone::Answer::Yes
                )
        }
        TaskKind::GoHome
        | TaskKind::Answer { .. }
        | TaskKind::Spot { .. }
        | TaskKind::Cook { .. }
        | TaskKind::Eat { .. }
        | TaskKind::Dishes { .. }
        | TaskKind::Carry { .. }
        | TaskKind::SetUp { .. }
        | TaskKind::Serve { .. } => false,
    }
}

/// Whether `s` is on their way off their home plot (or about to be): what they're doing or
/// anything they're yet to do takes them off it (`takes_off`). The one test of who's leaving
/// for childcare, whether at work time, for a player's order, on the phone or by their own
/// choice.
pub(crate) fn leaving_home(w: &World, s: &Sim) -> bool {
    let Some(home) = w.households.get(s.household as usize).and_then(|h| h.plot) else {
        return false;
    };
    s.current
        .as_ref()
        .map(|a| &a.task)
        .into_iter()
        .chain(&s.queue)
        .any(|t| takes_off(w, &t.kind, home))
}
