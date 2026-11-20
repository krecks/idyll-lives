//! Goals: progress, reviewing, suggesting and the object they ask for.

use super::*;

/// Progress toward a goal, 0..1.
pub fn progress(w: &World, i: usize, g: &Goal) -> f32 {
    let content = &w.content;
    let sim = &w.sims[i];
    let target = g.target.max(1e-3);
    let job_ok = |c: usize| {
        g.category
            .is_none_or(|cat| content.careers[c].category == Some(cat))
    };
    let p = match content.goals[g.def].kind {
        GoalKind::HasJob => sim.job.as_ref().is_some_and(|j| job_ok(j.career)) as u8 as f32,
        GoalKind::JobLevel => sim
            .job
            .as_ref()
            .filter(|j| job_ok(j.career))
            .map_or(0.0, |j| (j.level + 1) as f32 / target),
        // The skills the next level needs count as much as the performance bar.
        GoalKind::Promoted => sim.job.as_ref().map_or(0.0, |j| {
            if j.level as f32 > g.start {
                1.0
            } else {
                let needs = goal_skills(content, sim, g);
                let ready = if needs.is_empty() {
                    1.0
                } else {
                    needs
                        .iter()
                        .map(|&(s, l)| (sim.skills[s] / l).min(1.0))
                        .sum::<f32>()
                        / needs.len() as f32
                };
                (ready + j.performance / 100.0) * 0.5 * 0.9
            }
        }),
        GoalKind::Skill => g.skill.map_or(0.0, |s| sim.skills[s] / target),
        GoalKind::Friends => {
            let friends = (0..w.sims.len())
                .filter(|&j| j != i && w.relationships.get(i, j).friendship >= 50.0)
                .count();
            friends as f32 / target
        }
        GoalKind::Partner => {
            if w.relationships.partner_of(i).is_some() {
                1.0
            } else {
                let best = (0..w.sims.len())
                    .filter(|&j| j != i)
                    .map(|j| w.relationships.get(i, j).romance)
                    .fold(0.0f32, f32::max);
                (best / 100.0 * 0.9).max(0.0)
            }
        }
        GoalKind::Funds => w.households[sim.household as usize].funds as f32 / target,
        GoalKind::Own => match g.object {
            Some(d) if has_at_home(w, sim.household as usize, d) => 1.0,
            // Saving up for it: it's only had once it's home.
            _ => (w.households[sim.household as usize].funds as f32 / target).min(1.0) * 0.9,
        },
    };
    p.clamp(0.0, 1.0)
}

/// Skill levels a career goal takes (the next level's for a promotion; every level's up to the
/// target for a job level, in their current job), highest per skill.
pub fn goal_skills(content: &Content, sim: &Sim, g: &Goal) -> Vec<(usize, f32)> {
    let Some(job) = &sim.job else {
        return Vec::new();
    };
    let career = &content.careers[job.career];
    let levels = match content.goals[g.def].kind {
        GoalKind::Promoted => job.level + 1..job.level + 2,
        GoalKind::JobLevel if g.category.is_none_or(|c| career.category == Some(c)) => {
            job.level + 1..g.target.max(0.0) as usize
        }
        _ => return Vec::new(),
    };
    let mut out: Vec<(usize, f32)> = Vec::new();
    for level in career.levels.get(levels).unwrap_or_default() {
        for &(s, l) in &level.requires {
            match out.iter_mut().find(|o| o.0 == s) {
                Some(o) => o.1 = o.1.max(l),
                None if l > 0.0 => out.push((s, l)),
                None => {}
            }
        }
    }
    out
}

pub(super) fn check_goals(w: &mut World, i: usize) {
    let tick = w.tick;
    // Ideas that came true meanwhile just go.
    let mut k = 0;
    while k < w.sims[i].planner.suggestions.len() {
        let goal = w.sims[i].planner.suggestions[k];
        if progress(w, i, &goal) >= 1.0 {
            w.sims[i].planner.suggestions.remove(k);
        } else {
            k += 1;
        }
    }
    let mut k = 0;
    while k < w.sims[i].planner.goals.len() {
        let goal = w.sims[i].planner.goals[k];
        let p = progress(w, i, &goal);
        w.sims[i].planner.goals[k].progress = p;
        if p < 1.0 {
            k += 1;
            continue;
        }
        let content = &w.content;
        let sim = &mut w.sims[i];
        sim.planner.goals.remove(k);
        let feeling = content.goals[goal.def]
            .feeling
            .or(content.planner.goal_feeling);
        if let Some(f) = feeling {
            social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        }
        // Got what they wanted (whoever bought it).
        if goal.object.is_some() {
            if let Some(f) = content.planner.wish_feeling {
                social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
            }
            if sim
                .planner
                .purchase
                .is_some_and(|p| Some(p.object) == goal.object)
            {
                sim.planner.purchase = None;
            }
        }
        think(sim, ThoughtKind::Goal, goal.def, tick);
        w.events.push_goal(tick, EventKind::GoalReached, i, &goal);
    }
}

/// Once a day: progress history, and maybe a new goal suggested from their traits (the
/// neighbours, who have no player, take theirs on straight away).
pub(super) fn review_goals(w: &mut World, i: usize, day: u32) {
    for k in 0..w.sims[i].planner.goals.len() {
        let g = w.sims[i].planner.goals[k];
        let p = progress(w, i, &g);
        let g = &mut w.sims[i].planner.goals[k];
        g.progress = p;
        g.history = [p, g.history[0], g.history[1]];
    }
    ask_to_buy(w, i, day);
    let rules = &w.content.planner;
    let player = w.households[w.sims[i].household as usize].player;
    let free = w.autonomy && w.households[w.sims[i].household as usize].free_will;
    // Ideas the player let be for a while: they go ahead with them.
    if free {
        let (max, wait) = (rules.max_goals, rules.suggestion_days);
        let p = &mut w.sims[i].planner;
        while p.goals.len() < max
            && let Some(k) = p.suggestions.iter().position(|g| day >= g.since_day + wait)
        {
            let mut goal = p.suggestions.remove(k);
            goal.since_day = day;
            if !p.goals.iter().any(|g| g.same(&goal)) {
                p.goals.push(goal);
            }
        }
    }
    let rules = &w.content.planner;
    let p = &w.sims[i].planner;
    let room = if player {
        p.suggestions.len() < rules.max_suggestions && p.goals.len() < rules.max_goals
    } else {
        p.goals.len() < rules.max_goals.min(2)
    };
    if !room || !w.autonomy || !w.households[w.sims[i].household as usize].free_will {
        return;
    }
    // About every other day.
    if w.rng.next_f32() < 0.5 {
        return;
    }
    let Some(goal) = suggest(w, i, day) else {
        return;
    };
    let tick = w.tick;
    let sim = &mut w.sims[i];
    if player {
        sim.planner.suggestions.push(goal);
        w.events.push_goal(tick, EventKind::GoalSuggested, i, &goal);
    } else {
        sim.planner.goals.push(goal);
    }
}

/// A goal that suits the resident now, weighted by their traits.
fn suggest(w: &mut World, i: usize, day: u32) -> Option<Goal> {
    let wanted = if w.content.goals.iter().any(|d| d.kind == GoalKind::Own) {
        wanted_object(w, i)
    } else {
        None
    };
    let content = &w.content;
    let sim = &w.sims[i];
    let taken = |kind: GoalKind, skill: Option<usize>| {
        sim.planner
            .goals
            .iter()
            .chain(&sim.planner.suggestions)
            .any(|g| {
                content.goals[g.def].kind == kind && (kind != GoalKind::Skill || g.skill == skill)
            })
    };
    let friends = (0..w.sims.len())
        .filter(|&j| j != i && w.relationships.get(i, j).friendship >= 50.0)
        .count() as f32;
    let funds = w.households[sim.household as usize].funds;
    // Skills a career goal of theirs takes already (no separate goal for those).
    let implied: Vec<usize> = sim
        .planner
        .goals
        .iter()
        .chain(&sim.planner.suggestions)
        .flat_map(|g| goal_skills(content, sim, g))
        .map(|(s, _)| s)
        .collect();
    // The skill they're best at learning (traits), else their strongest.
    let skill = (0..content.skills.len())
        .filter(|&s| sim.skills[s] < content.skill_rules.max_level - 1.0 && !implied.contains(&s))
        .max_by(|&a, &b| {
            let key = |s: usize| sim.mods.skill_gain[s] * 2.0 + sim.skills[s] * 0.1;
            key(a).total_cmp(&key(b))
        });
    let mut options: Vec<(f32, Goal)> = Vec::new();
    for (d, def) in content.goals.iter().enumerate() {
        let make = |skill: Option<usize>, target: f32, start: f32| Goal {
            def: d,
            skill,
            category: None,
            object: None,
            target,
            since_day: day,
            start,
            progress: 0.0,
            history: [0.0; 3],
        };
        let goal = match def.kind {
            GoalKind::HasJob if sim.job.is_none() => make(None, 1.0, 0.0),
            GoalKind::JobLevel => match &sim.job {
                Some(j) if j.level + 2 < content.careers[j.career].levels.len() => {
                    make(None, (j.level + 3) as f32, 0.0)
                }
                _ => continue,
            },
            GoalKind::Promoted => match &sim.job {
                Some(j) if j.level + 1 < content.careers[j.career].levels.len() => {
                    make(None, 1.0, j.level as f32)
                }
                _ => continue,
            },
            GoalKind::Skill => match skill {
                Some(s) => make(
                    Some(s),
                    (sim.skills[s].floor() + 2.0).min(content.skill_rules.max_level),
                    0.0,
                ),
                None => continue,
            },
            GoalKind::Friends => make(None, friends + 2.0, 0.0),
            GoalKind::Partner if w.relationships.partner_of(i).is_none() => make(None, 1.0, 0.0),
            GoalKind::Funds if funds < 50_000 => {
                make(None, (((funds.max(0) + 2500) / 500) * 500) as f32, 0.0)
            }
            GoalKind::Own => match wanted {
                Some(o) => Goal {
                    object: Some(o),
                    ..make(None, content.objects[o].price.unwrap_or(0) as f32, 0.0)
                },
                None => continue,
            },
            _ => continue,
        };
        if taken(def.kind, goal.skill) {
            continue;
        }
        let weight = def.weight
            + sim
                .traits
                .iter()
                .filter_map(|t| def.traits.get(t))
                .sum::<f32>();
        if weight > 0.0 {
            options.push((weight, goal));
        }
    }
    let total: f32 = options.iter().map(|o| o.0).sum();
    let mut pick = w.rng.next_f32() * total;
    for (weight, goal) in options {
        if pick < weight {
            return Some(goal);
        }
        pick -= weight;
    }
    None
}

// ---- Things they want ---------------------------------------------------------------------

/// Whether household `h` has an object of type `def` at home.
pub fn has_at_home(w: &World, h: usize, def: usize) -> bool {
    let Some(plot) = w.households[h].plot else {
        return false;
    };
    w.objects
        .iter()
        .zip(&w.object_plot)
        .any(|(o, p)| *p == Some(plot) && o.def == def)
}

/// Two object types for the same thing (a better bed for a bed): the same catalog group and
/// some use in common.
fn same_kind(content: &Content, a: usize, b: usize) -> bool {
    let (a, b) = (&content.objects[a], &content.objects[b]);
    !a.group.is_empty()
        && a.group == b.group
        && a.interactions
            .iter()
            .any(|x| b.interactions.iter().any(|y| x.id == y.id))
}

/// Whether object type `new` does all `old` does: every use of it (by id), the cooking stations
/// it is, and keeping leftovers. A range cooker shares the gourmet dinner with a fridge but is
/// no fridge: selling the fridge for it would leave no snacks and no ingredients.
fn covers(content: &Content, new: usize, old: usize) -> bool {
    let (new, old) = (&content.objects[new], &content.objects[old]);
    old.interactions
        .iter()
        .all(|x| new.interactions.iter().any(|y| x.id == y.id))
        && old.stations & !new.stations == 0
        && (new.stores || !old.stores)
}

/// The cheaper thing at home that `def` would replace (an upgrade that does all it did), if any.
pub(crate) fn replaces(w: &World, h: usize, def: usize) -> Option<u32> {
    let plot = w.households[h].plot?;
    let price = w.content.objects[def].price?;
    w.objects
        .iter()
        .zip(&w.object_plot)
        .filter(|(o, p)| {
            **p == Some(plot)
                && o.def != def
                && same_kind(&w.content, o.def, def)
                && covers(&w.content, def, o.def)
        })
        .filter(|(o, _)| w.content.objects[o.def].price.is_some_and(|p| p < price))
        .min_by_key(|(o, _)| w.content.objects[o.def].price)
        .map(|(o, _)| o.id)
}

/// Something from the catalog that suits the resident and is within reach of the household's
/// money: what their traits like, trains the skills they're after, or gives a planned activity
/// a place; or a better one of something they have. One of the best few, at random.
fn wanted_object(w: &mut World, i: usize) -> Option<usize> {
    let pick = w.rng.next_f32();
    let content = &w.content;
    let sim = &w.sims[i];
    let h = sim.household as usize;
    let plot = w.households[h].plot?;
    let reach = w.households[h].funds.max(0) as f32 * 1.5 + 1000.0;
    let home: Vec<usize> = w
        .objects
        .iter()
        .zip(&w.object_plot)
        .filter(|(_, p)| **p == Some(plot))
        .map(|(o, _)| o.def)
        .collect();
    let (targets, keen) = crate::world::job_goals(sim, content);
    let mut options: Vec<(f32, usize)> = Vec::new();
    for (d, def) in content.objects.iter().enumerate() {
        let Some(price) = def.price else { continue };
        if price <= 0
            || price as f32 > reach
            || home.contains(&d)
            || sim
                .planner
                .goals
                .iter()
                .chain(&sim.planner.suggestions)
                .any(|g| g.object == Some(d))
        {
            continue;
        }
        let uses: Vec<&Interaction> = def
            .interactions
            .iter()
            .filter(|it| it.autonomous && !it.baby)
            .collect();
        if uses.is_empty() {
            continue;
        }
        // Something as good for the same thing at home already: no need.
        let same: Vec<usize> = home
            .iter()
            .copied()
            .filter(|&o| same_kind(content, o, d))
            .collect();
        if same
            .iter()
            .any(|&o| content.objects[o].price.unwrap_or(0) >= price)
        {
            continue;
        }
        let liking = uses
            .iter()
            .map(|it| sim.mods.preference(it.tags))
            .fold(0.0f32, f32::max);
        let mut score = (liking - 1.0).max(0.0) * 2.0;
        for it in &uses {
            for (s, &gain) in it.skill_gain.iter().enumerate() {
                if gain > 0.0 && targets[s] > sim.skills[s] {
                    score += keen[s].min(1.0);
                }
            }
            for &(a, skill) in &sim.planner.wishes {
                if content.activities[a].fits(it, skill) {
                    score += 3.0;
                }
            }
        }
        if !same.is_empty() {
            score += 0.5;
        }
        if score > 0.0 {
            options.push((score, d));
        }
    }
    options.sort_by(|a, b| b.0.total_cmp(&a.0));
    options.truncate(3);
    let total: f32 = options.iter().map(|o| o.0).sum();
    let mut pick = pick * total;
    for (score, d) in options {
        if pick < score {
            return Some(d);
        }
        pick -= score;
    }
    None
}

/// Once a day: a resident whose top `Own` goal they can afford (with a week's rent and bills
/// left over) asks the player for it, or, in a household nobody plays, just buys it. After a
/// no, they wait a few days before asking for the same thing again.
fn ask_to_buy(w: &mut World, i: usize, day: u32) {
    let h = w.sims[i].household as usize;
    if !w.autonomy || !w.households[h].free_will || w.sims[i].away_until.is_some() {
        return;
    }
    let content = &w.content;
    let sim = &w.sims[i];
    let Some(object) = sim
        .planner
        .goals
        .iter()
        .find(|g| content.goals[g.def].kind == GoalKind::Own)
        .and_then(|g| g.object)
    else {
        return;
    };
    // (Bought already, and waiting to be set up: `setup.rs`.)
    if has_at_home(w, h, object)
        || w.setups
            .iter()
            .any(|s| s.household as usize == h && s.object == object)
    {
        return;
    }
    let price = content.objects[object].price.unwrap_or(0);
    // (The bills go up with what the home is worth: counted with it in there.)
    let reserve = crate::life::weekly_costs(w, h).map_or(0, |(rent, bills)| rent + bills)
        + content
            .rent
            .as_ref()
            .map_or(0, |r| r.bills(price, 0) - r.bills(0, 0));
    if w.households[h].funds - w.build_price(price) < reserve {
        return;
    }
    let mut request = match sim.planner.purchase {
        Some(p) if p.object == object => p,
        _ => PurchaseRequest {
            object,
            open: false,
            day,
            declines: 0,
        },
    };
    // Waiting for an answer since today, or after a no.
    if request.open && request.day == day
        || !request.open
            && request.declines > 0
            && day < request.day + content.planner.ask_again_days
    {
        return;
    }
    if !w.households[h].player {
        // Nobody to ask: they go ahead (and try again in a few days if there's no room).
        if buy_wanted(w, i, object).is_err() {
            w.sims[i].planner.purchase = Some(PurchaseRequest {
                object,
                open: false,
                day,
                declines: 1,
            });
        }
        return;
    }
    request.open = true;
    request.day = day;
    let tick = w.tick;
    let sim = &mut w.sims[i];
    sim.planner.purchase = Some(request);
    think(sim, ThoughtKind::WantsToBuy, object, tick);
    w.events
        .push_object(tick, EventKind::WantsToBuy, i, object, Some(price));
}
