//! At work: breaks, skill-ups, and the week's costs and rent.

use super::*;

/// Lunch, the toilet, a coffee: at work (paid from `funds`) or school (free), each need below
/// its break gets topped up, more than once if a long catch-up step took it far down.
fn take_breaks(sim: &mut Sim, content: &Content, mut funds: Option<&mut i64>) {
    for b in &content.day_rhythm.work_breaks {
        for _ in 0..4 {
            if sim.needs[b.need] >= b.below {
                break;
            }
            if let Some(funds) = funds.as_deref_mut() {
                if !affordable(b.cost, *funds) {
                    break;
                }
                *funds -= b.cost;
            }
            sim.needs[b.need] = (sim.needs[b.need] + b.gain).min(1.0);
        }
    }
}

/// Off the map at work: needs change and skills grow, then (true) they're done, with pay and maybe
/// a promotion: the caller sees them home.
pub(super) fn at_work(w: &mut World, i: usize) -> bool {
    let tick = w.tick;
    let World {
        content,
        sims,
        households,
        events,
        cars,
        ..
    } = w;
    let sim = &mut sims[i];
    // Still on their way: work (or school) hasn't started.
    if tick < sim.shift_from {
        return false;
    }
    let minutes = sim.minutes();
    let Some(job) = sim.job.clone() else {
        take_breaks(sim, content, None);
        // At school: a little of the school's skills, then home.
        if let Some(school) = &content.life.school {
            for &(skill, weight) in &school.skills {
                practise(
                    sim,
                    content,
                    skill,
                    content.skill_rules.work_gain_per_hour * weight,
                    minutes,
                );
            }
        }
        if sim.away_until.is_some_and(|until| tick < until) {
            return false;
        }
        // A day at school: the better they felt, the better it went.
        if let Some(school) = &content.life.school {
            let mood = sim.mood(content);
            sim.grade =
                (sim.grade + school.attend + (mood - 0.5) * 2.0 * school.mood).clamp(0.0, 100.0);
        }
        sim.away_until = None;
        return true;
    };
    let career = &content.careers[job.career];
    let level = &career.levels[job.level];
    let per_tick = minutes / (level.hours * 60.0);
    for n in 0..MAX_NEEDS.min(content.needs.len()) {
        sim.needs[n] = (sim.needs[n] + career.work_effects[n] * per_tick).clamp(0.0, 1.0);
    }
    take_breaks(
        sim,
        content,
        Some(&mut households[sim.household as usize].funds),
    );
    for &(skill, weight) in &career.skills {
        practise(
            sim,
            content,
            skill,
            content.skill_rules.work_gain_per_hour * weight,
            minutes,
        );
    }
    if sim.away_until.is_some_and(|until| tick < until) {
        return false;
    }
    sim.away_until = None;
    households[sim.household as usize].funds += shift_pay(content, &job, &sim.skills);
    let rules = &content.career_rules;
    let fit = skill_fit(level, &sim.skills);
    // Still a job: it was there above, and nothing since takes it away.
    let Some(job) = sim.job.as_mut() else {
        return true;
    };
    job.shifts += 1;
    job.missed = job.missed.saturating_sub(1);
    job.satisfaction = job.satisfaction * 0.8 + job.shift_mood * 0.2;
    let worth = shift_performance(rules, job.level, job.shift_mood, fit, job.late);
    job.performance = (job.performance + worth).clamp(0.0, 100.0);
    // A promotion needs a full bar and the skills for the next level.
    if job.performance >= 100.0
        && let Some(next) = career.levels.get(job.level + 1)
        && skill_fit(next, &sim.skills) >= 0.0
    {
        // A promotion pays a one-off bonus of half a week at the new grade, rounded up.
        // Saturating: a content pack's pay can be near `i64::MAX`, and a promotion must not panic.
        let shifts_a_week = next.days.count_ones().max(1) as i64;
        let bonus = next.pay.saturating_mul(shifts_a_week).saturating_add(1) / 2;
        households[sim.household as usize].funds += bonus;
        job.level += 1;
        job.performance = 0.0;
        sim.planner.recheck = true;
        if let Some(m) = rules.promotion_feeling {
            social::add_feeling(&mut sim.feelings, m, &content.feelings, tick);
        }
        events.push_promotion(tick, i, job.career, job.level, bonus);
        // (The car the new grade calls for: `cars/looks.rs`.)
        let household = sims[i].household;
        crate::cars::job_changed(cars, content, sims, household);
    }
    true
}

/// Story events for new skill levels (the player's household only: across a whole town they
/// would crowd out everything else).
pub(super) fn report_skill_ups(w: &mut World, i: usize) {
    let sim = &mut w.sims[i];
    let mut ups = std::mem::take(&mut sim.skill_ups);
    if ups == 0 || !w.households[sim.household as usize].player {
        return;
    }
    while ups != 0 {
        let s = ups.trailing_zeros() as usize;
        ups &= ups - 1;
        let level = sim.skills[s].floor() as i64;
        w.events
            .push_detail(w.tick, EventKind::SkillUp, i, Some(level), Some(s));
    }
}

/// What household `h` pays each week: `(rent, bills)`, or `None` if it has no home or
/// nothing is charged (nobody lives there yet, or it's the player's home in a Creative game). Bills grow with the value of
/// everything the household owns and with how many live there.
pub fn weekly_costs(w: &World, h: usize) -> Option<(i64, i64)> {
    let rent = w.content.rent.as_ref()?;
    let household = w.households.get(h)?;
    if household.player && w.mode == GameMode::Creative {
        return None;
    }
    let plot = household.plot?;
    let p = &w.plots[plot as usize];
    let home_value: i64 = w
        .objects
        .iter()
        .zip(&w.object_plot)
        .filter(|(_, on)| **on == Some(plot))
        .map(|(o, _)| o.value)
        .sum();
    let residents = w
        .sims
        .iter()
        .filter(|s| s.here() && s.household as usize == h)
        .count();
    if residents == 0 {
        return None;
    }
    Some((rent.amount(p.w * p.d), rent.bills(home_value, residents)))
}

/// Weekly rent and bills; households that can't cover them go into debt and worry about it
/// every day.
pub(super) fn charge_rent(w: &mut World, day: u32) {
    let Some(rent) = w.content.rent.clone() else {
        return;
    };
    if !clock::at_minute(w.tick, rent.hour * 60.0) {
        return;
    }
    let tick = w.tick;
    let due = clock::weekday(day) == rent.weekday;
    for h in 0..w.households.len() {
        let Some(member) = w
            .sims
            .iter()
            .position(|s| s.here() && s.household as usize == h)
        else {
            continue;
        };
        if due {
            // Pensions come in on the same day.
            let pensions: i64 = w
                .sims
                .iter()
                .filter(|s| s.here() && s.household as usize == h)
                .map(|s| s.pension)
                .sum();
            w.households[h].funds += pensions;
        }
        if due && let Some((rent, bills)) = weekly_costs(w, h) {
            let amount = rent + bills;
            let household = &mut w.households[h];
            let kind = if household.funds >= amount {
                EventKind::PaidRent
            } else {
                EventKind::RentDebt
            };
            household.funds -= amount;
            // Everyone's debts are news; paying on time only for the player's household.
            if household.player || kind == EventKind::RentDebt {
                w.events.push_detail(tick, kind, member, Some(amount), None);
            }
        }
        if w.households[h].funds < 0
            && let Some(m) = rent.debt_feeling
        {
            let World { sims, content, .. } = &mut *w;
            for s in sims
                .iter_mut()
                .filter(|s| s.here() && s.household as usize == h)
            {
                social::add_feeling(&mut s.feelings, m, &content.feelings, tick);
            }
        }
    }
}
