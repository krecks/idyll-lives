//! How strongly plans, objects, people and work pull a choice.

use super::*;

/// How strongly a goal steers by its place on the list (0 = the top): the top one fully, each
/// one below a little less.
pub fn goal_strength(rules: &PlannerRules, rank: usize) -> f32 {
    1.0 / (1.0 + rules.goal_falloff * rank as f32)
}

/// Strength of the highest goal of one of these kinds (0: none). A kind counts once, however
/// many goals of it there are.
fn kind_strength(content: &Content, sim: &Sim, kinds: &[GoalKind]) -> f32 {
    sim.planner
        .goals
        .iter()
        .position(|g| kinds.contains(&content.goals[g.def].kind))
        .map_or(0.0, |rank| goal_strength(&content.planner, rank))
}

/// A goal's multiplier `full` at `strength` (1 at none, `full` at the top of the list).
fn scaled(full: f32, strength: f32) -> f32 {
    1.0 + (full - 1.0) * strength
}

/// Multiplier for an object interaction while planning: (factor, floor score). `night` is
/// `planned_night` (residents who plan their sleep don't go to bed outside it unless exhausted);
/// `cost`, what the use costs where the object stands (`Interaction::cost_at`: a home kitchen's
/// snack is bought on a public lot, and saving up weighs against it there).
pub(crate) fn object_factor(
    content: &Content,
    sim: &Sim,
    inter: &Interaction,
    cost: i64,
    night: Option<bool>,
) -> (f32, f32) {
    let mut factor = 1.0;
    if night == Some(false) && inter.tags & content.day_rhythm.sleep_tags != 0 {
        let exhausted = (0..content.needs.len())
            .any(|n| inter.total_gain[n] > 0.0 && sim.needs[n] < content.planner.urgent_below);
        if !exhausted {
            return (0.0, 0.0);
        }
    }
    if cost > 0 {
        factor *= scaled(
            0.3,
            kind_strength(content, sim, &[GoalKind::Funds, GoalKind::Own]),
        );
    }
    let Some(run) = active(sim) else {
        return (factor, 0.0);
    };
    let a = &content.activities[run.activity];
    if a.fits(inter, run.skill) && !a.social && !a.visit {
        return (factor * run.boost, content.planner.floor);
    }
    let urgent = (0..content.needs.len())
        .any(|n| sim.needs[n] < content.planner.urgent_below && inter.total_gain[n] > 0.0);
    (
        if urgent {
            factor
        } else {
            factor * content.planner.off_block
        },
        0.0,
    )
}

/// `(factor, floor)` for an action without an object (tidying up) with these tags: favoured
/// in a planned block whose activity it fits (the chores), held back outside one.
pub(crate) fn tags_factor(content: &Content, sim: &Sim, tags: TagMask) -> (f32, f32) {
    let Some(run) = active(sim) else {
        return (1.0, 0.0);
    };
    let a = &content.activities[run.activity];
    if a.tags & tags != 0 && !a.social && !a.visit {
        return (run.boost, content.planner.floor);
    }
    (content.planner.off_block, 0.0)
}

/// Multiplier for a social with someone (`romantic`: it's a romantic social; `friendly`: it
/// builds friendship; `friendship` toward them; `single`: they're free to date).
pub(crate) fn social_factor(
    content: &Content,
    sim: &Sim,
    romantic: bool,
    friendly: bool,
    friendship: f32,
    single: bool,
) -> f32 {
    let mut factor = 1.0;
    if friendly && friendship < 50.0 {
        factor *= scaled(1.5, kind_strength(content, sim, &[GoalKind::Friends]));
    }
    if romantic && single {
        factor *= scaled(2.0, kind_strength(content, sim, &[GoalKind::Partner]));
    }
    match active(sim) {
        Some(run) if content.activities[run.activity].social => factor * run.boost,
        Some(_)
            if content
                .needs
                .iter()
                .zip(&sim.needs)
                .all(|(d, &n)| d.room || n >= content.planner.urgent_below) =>
        {
            factor * content.planner.off_block
        }
        _ => factor,
    }
}

/// Whether a block other than socialising or visiting starts within `minutes` from now (a
/// resident with plans doesn't set off to visit just before them).
pub(crate) fn plans_soon(
    content: &Content,
    sim: &Sim,
    household: &[Routine],
    day: u32,
    minute: f32,
    minutes: f32,
) -> bool {
    let mut m = 15.0;
    while m <= minutes {
        let (d, t) = if minute + m >= 1440.0 {
            (day + 1, minute + m - 1440.0)
        } else {
            (day, minute + m)
        };
        if let Some((r, ..)) = running(sim, household, d, t)
            && content
                .activities
                .get(r.activity)
                .is_some_and(|a| !a.social && !a.visit)
        {
            return true;
        }
        m += 15.0;
    }
    false
}

/// Multiplier for visiting someone (`soon`: plans start before a visit would be over).
pub(crate) fn visit_factor(content: &Content, sim: &Sim, soon: bool) -> f32 {
    if soon && active(sim).is_none() {
        return content.planner.off_block;
    }
    let factor = scaled(
        1.5,
        kind_strength(content, sim, &[GoalKind::Friends, GoalKind::Partner]),
    );
    match active(sim) {
        Some(run) if content.activities[run.activity].visit => factor * run.boost,
        Some(_) => factor * content.planner.off_block,
        None => factor,
    }
}

/// Whether using `inter` is what the resident's running block is for.
pub(crate) fn planned_now(content: &Content, sim: &Sim, inter: &Interaction) -> bool {
    active(sim).is_some_and(|run| {
        let a = &content.activities[run.activity];
        !a.social && !a.visit && a.fits(inter, run.skill)
    })
}

/// The block steering choices now (not skipped, somewhere to do it).
fn active(sim: &Sim) -> Option<&BlockRun> {
    sim.planner
        .run
        .as_ref()
        .filter(|r| r.skipped.is_none() && !r.no_place)
}

/// Skill targets from goals, and how keen they are on each (folded into practice interest
/// with the job's).
pub(crate) fn skill_targets(content: &Content, sim: &Sim, targets: &mut [f32], keen: &mut [f32]) {
    for (rank, g) in sim.planner.goals.iter().enumerate() {
        let strength = goal_strength(&content.planner, rank);
        match content.goals[g.def].kind {
            GoalKind::Skill => {
                if let Some(s) = g.skill {
                    targets[s] = targets[s].max(g.target);
                    keen[s] = keen[s].max(strength);
                }
            }
            // A career goal takes its skills: keener on them than the job alone makes them.
            GoalKind::Promoted | GoalKind::JobLevel => {
                for (s, l) in goal_skills(content, sim, g) {
                    targets[s] = targets[s].max(l);
                    keen[s] = keen[s].max(1.0 + strength);
                }
            }
            _ => {}
        }
    }
}

/// Job search: how much harder they look (a job goal), and in which category.
pub(crate) fn job_search(content: &Content, sim: &Sim) -> (f32, Option<usize>) {
    sim.planner
        .goals
        .iter()
        .position(|g| content.goals[g.def].kind == GoalKind::HasJob)
        .map_or((1.0, None), |rank| {
            let g = &sim.planner.goals[rank];
            (
                scaled(3.0, goal_strength(&content.planner, rank)),
                g.category,
            )
        })
}

// ---- Commands -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;

    #[test]
    fn saving_up_weighs_against_what_a_use_costs_where_it_stands() {
        // (A home kitchen's snack, free at home and bought on a public lot: the price there counts.)
        let content = r#"{"needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
            "goals":[{"id":"savings","label":"Save $N","kind":"funds"}],
            "objects":[{"id":"fridge","name":"Fridge","interactions":[
                {"id":"snack","label":"Snack","minutes":2,"effects":{"hunger":0.3},"publicCost":4}]}],
            "economy":{"startingFunds":500}}"#;
        let town = r#"{"width":10,"depth":10,"plots":[{"name":"Home","x":0,"z":0,"w":10,"d":10}],
            "households":[{"name":"H","plot":0,"player":true}],
            "objects":[{"def":"fridge","x":1,"z":1,"rot":0}],
            "sims":[{"name":"Ada","x":5.5,"z":5.5}]}"#;
        let mut w = World::from_json(content, town, 1).unwrap();
        w.sims[0].planner.goals.push(Goal {
            def: 0,
            skill: None,
            category: None,
            object: None,
            target: 1000.0,
            since_day: 1,
            start: 0.0,
            progress: 0.0,
            history: [0.0; 3],
        });
        let inter = &w.content.objects[0].interactions[0];
        let (home, _) = object_factor(&w.content, &w.sims[0], inter, inter.cost_at(false), None);
        let (out, _) = object_factor(&w.content, &w.sims[0], inter, inter.cost_at(true), None);
        assert_eq!(home, 1.0, "free at home: nothing to weigh");
        assert!(
            out < 1.0,
            "bought at a place, saving up weighs against it: {out}"
        );
    }
}
