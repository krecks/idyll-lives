//! The tick's planning: starting, following and finishing blocks, and offering work.

use super::*;

/// The block running for a resident at `day`/`minute`: their own first, then the household's
/// (unless skipped). Returns the routine, whether it's the household's, and its start day.
pub fn running(
    sim: &Sim,
    household: &[Routine],
    day: u32,
    minute: f32,
) -> Option<(Routine, bool, u32)> {
    let p = &sim.planner;
    p.routines
        .iter()
        .find_map(|r| r.running(day, minute).map(|d| (*r, false, d)))
        .or_else(|| {
            household
                .iter()
                .filter(|r| !p.skip_household.contains(&r.id))
                .find_map(|r| r.running(day, minute).map(|d| (*r, true, d)))
        })
}

/// For residents who plan their sleep: whether it's their night now (inside a sleep block), or
/// `None` when they have no sleep block today (the usual day rhythm applies).
pub fn planned_night(
    content: &Content,
    sim: &Sim,
    household: &[Routine],
    day: u32,
    minute: f32,
) -> Option<bool> {
    let p = &sim.planner;
    let sleep = |r: &&Routine| content.activities.get(r.activity).is_some_and(|a| a.sleep);
    let own = p.routines.iter().filter(sleep);
    let shared = household
        .iter()
        .filter(|r| !p.skip_household.contains(&r.id))
        .filter(sleep);
    let mut any = false;
    for r in own.chain(shared) {
        if r.running(day, minute).is_some() {
            return Some(true);
        }
        any |= r.touches(day);
    }
    any.then_some(false)
}

/// How well a resident sticks to plans: discipline from traits times mood.
pub fn adherence(content: &Content, sim: &Sim) -> f32 {
    let discipline: f32 = sim
        .traits
        .iter()
        .filter_map(|t| content.planner.discipline.get(t))
        .product();
    discipline * (0.6 + 0.8 * sim.mood(content))
}

/// Chance a resident skips a block of `activity` when it starts: less with discipline and a
/// good mood, more when their traits dislike the activity.
pub fn skip_chance(content: &Content, sim: &Sim, activity: &ActivityDef) -> f32 {
    let liking = if activity.tags != 0 {
        sim.mods.preference(activity.tags)
    } else {
        1.0
    };
    let base = content.planner.skip_chance / adherence(content, sim).max(0.05);
    (base + (1.0 - liking).max(0.0) * 0.5).min(0.9)
}

// ---- Per tick ----------------------------------------------------------------------------

pub(crate) fn update(w: &mut World) {
    refresh_offers(w);
    let tick = w.tick;
    let day = clock::day(tick);
    let minute = clock::minute_of_day(tick);
    let review = clock::at_minute(tick, w.content.planner.review_hour * 60.0);
    for i in 0..w.sims.len() {
        if !w.sims[i].here() {
            continue;
        }
        // Planned blocks: in lots nobody watches, on their turn (see `lod.rs`).
        if w.sims[i].steps > 0 {
            let h = w.sims[i].household as usize;
            let now = running(&w.sims[i], &w.households[h].routines, day, minute);
            let current = w.sims[i].planner.run;
            let same = |r: &BlockRun| {
                now.is_some_and(|(n, household, d)| {
                    n.id == r.routine && household == r.household && d == r.day
                })
            };
            if let Some(run) = current
                && !same(&run)
            {
                finish(w, i, run);
            }
            if let Some((routine, household, block_day)) = now
                && !current.as_ref().is_some_and(same)
            {
                begin(w, i, routine, household, block_day);
            }
            follow(w, i);
        }
        if w.sims[i].planner.thought.is_some_and(|t| t.until <= tick) {
            w.sims[i].planner.thought = None;
        }
        // Goals: progress every game ten minutes (staggered) or straight after a promotion, a
        // new skill level or a purchase; reviews once a day.
        if (tick + i as u64).is_multiple_of(10 * crate::TICKS_PER_MINUTE)
            || std::mem::take(&mut w.sims[i].planner.recheck)
        {
            check_goals(w, i);
        }
        if review && w.sims[i].planner.reviewed != day {
            w.sims[i].planner.reviewed = day;
            review_goals(w, i, day);
        }
    }
}

/// Recomputes what each household's home offers when walls or objects changed: what each kind
/// of object offers (its autonomous uses), then each plot's objects that aren't broken, then
/// each household's home with the public places (a pass over the objects, not one per
/// household).
fn refresh_offers(w: &mut World) {
    if w.offers_version == w.structure_version && w.offers.len() == w.households.len() {
        return;
    }
    w.offers_version = w.structure_version;
    let content = &w.content;
    let by_def: Vec<HouseOffer> = content
        .objects
        .iter()
        .map(|def| {
            let mut offer = HouseOffer::default();
            for inter in def.interactions.iter().filter(|i| i.autonomous) {
                for (a, act) in content.activities.iter().enumerate() {
                    if inter.tags & act.tags != 0 {
                        offer.activities |= 1 << a;
                    }
                }
                for (s, &g) in inter.skill_gain.iter().enumerate() {
                    if g > 0.0 {
                        offer.skills |= 1 << s;
                    }
                }
            }
            offer
        })
        .collect();
    let mut on_plot = vec![HouseOffer::default(); w.plots.len()];
    let mut public = HouseOffer::default();
    for (o, plot) in w.objects.iter().zip(&w.object_plot) {
        let Some(p) = plot.map(|p| p as usize) else {
            continue;
        };
        if o.broken() {
            continue;
        }
        let d = by_def[o.def];
        let to = if w.plots[p].public {
            &mut public
        } else {
            &mut on_plot[p]
        };
        to.activities |= d.activities;
        to.skills |= d.skills;
    }
    // There is always something to tidy at home.
    let mut tidy = 0u64;
    for (a, def) in content.activities.iter().enumerate() {
        if def.tags & content.room_rules.clean.tags != 0 && !def.social && !def.visit {
            tidy |= 1 << a;
        }
    }
    w.offers = w
        .households
        .iter()
        .map(|h| {
            let mut offer = public;
            if let Some(p) = h.plot {
                let own = on_plot[p as usize];
                offer.activities |= own.activities | tidy;
                offer.skills |= own.skills;
            }
            offer
        })
        .collect();
    // Wishes come true once the house has what they wished for.
    for s in &mut w.sims {
        let offer = w.offers[s.household as usize];
        s.planner
            .wishes
            .retain(|&(a, skill)| !offer.has(&content.activities[a], a, skill));
    }
}

pub(crate) fn think(sim: &mut Sim, kind: ThoughtKind, subject: usize, tick: u64) {
    sim.planner.thought = Some(Thought {
        kind,
        subject,
        until: tick + social::minutes_to_ticks(THOUGHT_MINUTES),
    });
}

/// A block starts: maybe skipped (traits, mood, away, nowhere to do it), else they drop what
/// they chose themselves and go for it.
fn begin(w: &mut World, i: usize, routine: Routine, household: bool, day: u32) {
    let tick = w.tick;
    let content = &w.content;
    let rules = &content.planner;
    let Some(activity) = content.activities.get(routine.activity).cloned() else {
        return;
    };
    let offer = w
        .offers
        .get(w.sims[i].household as usize)
        .copied()
        .unwrap_or_default();
    let sim = &w.sims[i];
    let away = sim.away_until.is_some() || (sim.visiting.is_some() && !activity.visit);
    let no_place = !offer.has(&activity, routine.activity, routine.skill);
    let adherence = adherence(content, sim);
    let skip = skip_chance(content, sim, &activity);
    let roll = w.rng.next_f32();
    let skipped = if away {
        Some(Reason::Away)
    } else if !no_place && roll < skip && !activity.sleep {
        let dislike = sim
            .traits
            .iter()
            .filter_map(|t| content.traits.iter().position(|d| &d.id == t))
            .map(|t| (t, content.traits[t].mods.preference(activity.tags)))
            .filter(|(_, p)| *p < 0.95)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        Some(dislike.map_or(Reason::Mood, |(t, _)| Reason::Trait(t)))
    } else {
        None
    };
    let run = BlockRun {
        routine: routine.id,
        household,
        activity: routine.activity,
        skill: routine.skill,
        day,
        start: routine.start,
        minutes: routine.minutes,
        skipped,
        no_place,
        boost: rules.boost * adherence.clamp(0.4, 1.6),
        done: 0.0,
        pulled: None,
        late: false,
    };
    let World {
        sims,
        objects,
        content,
        ..
    } = w;
    let sim = &mut sims[i];
    sim.planner.run = Some(run);
    if no_place && !away {
        let wish = (routine.activity, routine.skill.filter(|_| activity.skill));
        if !sim.planner.wishes.contains(&wish) {
            sim.planner.wishes.push(wish);
            if sim.planner.wishes.len() > 4 {
                sim.planner.wishes.remove(0);
            }
        }
        if let Some(f) = content.planner.no_place_feeling {
            social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        }
        think(sim, ThoughtKind::NoPlace, routine.activity, tick);
        return;
    }
    if matches!(skipped, Some(Reason::Trait(_) | Reason::Mood)) {
        think(sim, ThoughtKind::Skipped, routine.activity, tick);
        return;
    }
    if skipped.is_some() {
        return;
    }
    // Time for it: stop what they picked for themselves (not a needed sleep, a conversation,
    // work or a trip, and not something the player asked for).
    let rested = sim.needs[..content.needs.len()].iter().all(|&n| n >= 0.5);
    let interruptible = sim.engaged_with.is_none()
        && sim.current().is_some_and(|a| {
            !a.task.directed
                && (a.tags & content.day_rhythm.sleep_tags == 0 || rested)
                && matches!(
                    a.task.kind,
                    TaskKind::Use { .. } | TaskKind::MoveTo { .. } | TaskKind::Clean { .. }
                )
                && !fits(content, objects, &run, sim)
        });
    if interruptible {
        crate::world::end_activity(sim, content, objects);
    }
}

/// While a block runs: count the minutes spent on it, and note what pulled them away. Back
/// from work or a visit with time left, they pick it up again.
fn follow(w: &mut World, i: usize) {
    let content = &w.content;
    let sim = &w.sims[i];
    let Some(run) = sim.planner.run else { return };
    let minutes = sim.minutes();
    if run.skipped == Some(Reason::Away) && sim.away_until.is_none() && sim.visiting.is_none() {
        let run = w.sims[i].planner.run.as_mut().expect("checked above");
        run.skipped = None;
        run.late = true;
        return;
    }
    if run.skipped.is_some() || run.no_place {
        return;
    }
    let on_it = fits(content, &w.objects, &run, sim);
    let mut pulled = run.pulled;
    if !on_it
        && let Some(n) = (0..content.needs.len())
            .filter(|&n| !content.needs[n].room && sim.needs[n] < content.planner.urgent_below)
            .min_by(|&a, &b| sim.needs[a].total_cmp(&sim.needs[b]))
    {
        pulled = Some(n);
    }
    let run = w.sims[i].planner.run.as_mut().expect("checked above");
    if on_it {
        run.done += minutes;
    }
    run.pulled = pulled;
}

/// Whether what the resident is doing right now is the block's activity.
fn fits(
    content: &Content,
    objects: &[crate::world::ObjectInstance],
    run: &BlockRun,
    sim: &Sim,
) -> bool {
    let Some(a) = content.activities.get(run.activity) else {
        return false;
    };
    if a.visit && sim.visiting.is_some() {
        return true;
    }
    if a.social && sim.engaged_with.is_some() {
        return true;
    }
    let Some(act) = sim.current() else {
        return false;
    };
    match (act.task.kind, &act.phase) {
        (TaskKind::Social { .. } | TaskKind::Join { .. }, Phase::Conversing { .. }) => a.social,
        // Walking over to it counts too.
        (
            TaskKind::Use {
                object,
                interaction,
            },
            Phase::Using { .. } | Phase::Routing { .. },
        ) => objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .is_some_and(|inter| a.fits(inter, run.skill)),
        // Tidying up is chores.
        (TaskKind::Clean { .. }, Phase::Using { .. } | Phase::Routing { .. }) => {
            run.skill.is_none() && a.tags & content.room_rules.clean.tags != 0
        }
        // Cooking is what the recipe's interaction is (all along, between stations too); eating
        // and washing up are what their tags say.
        (TaskKind::Cook { recipe, origin, .. }, _) => {
            crate::cooking::origin_interaction(content, objects, origin, recipe)
                .is_some_and(|inter| a.fits(inter, run.skill))
        }
        (TaskKind::Eat { .. }, _) => run.skill.is_none() && a.tags & content.cooking.eat_tags != 0,
        (TaskKind::Dishes { .. }, _) => {
            run.skill.is_none() && a.tags & content.cooking.dishes_tags != 0
        }
        // On the phone with someone is socialising.
        (
            TaskKind::Phone {
                contact: Some(_), ..
            },
            Phase::Using { .. },
        ) => a.social,
        _ => false,
    }
}

/// A block is over: how it went, a feeling for keeping it.
fn finish(w: &mut World, i: usize, run: BlockRun) {
    let tick = w.tick;
    let content = &w.content;
    let sim = &mut w.sims[i];
    sim.planner.run = None;
    let (outcome, reason) = if run.no_place {
        (Outcome::NoPlace, Some(Reason::NoPlace))
    } else if let Some(r) = run.skipped {
        (Outcome::Skipped, Some(r))
    } else if run.done >= run.minutes as f32 * content.planner.kept_share {
        (Outcome::Kept, None)
    } else if run.done > 0.0 {
        (
            Outcome::Cut,
            run.pulled
                .map(Reason::Need)
                .or(run.late.then_some(Reason::Away))
                .or(Some(Reason::Mood)),
        )
    } else {
        (
            Outcome::Skipped,
            run.pulled
                .map(Reason::Need)
                .or(run.late.then_some(Reason::Away))
                .or(Some(Reason::Mood)),
        )
    };
    if outcome == Outcome::Kept {
        if let Some(f) = content.planner.kept_feeling {
            social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        }
        if !content.activities[run.activity].sleep {
            think(sim, ThoughtKind::Kept, run.activity, tick);
        }
    }
    let history = &mut sim.planner.history;
    if history.len() >= HISTORY {
        history.pop_front();
    }
    history.push_back(BlockResult {
        routine: run.routine,
        household: run.household,
        activity: run.activity,
        skill: run.skill,
        day: run.day,
        start: run.start,
        minutes: run.minutes,
        outcome,
        reason,
        done: run.done,
    });
}

// ---- Goals ------------------------------------------------------------------------------
