//! [(71, 455), (1162, 1416), (1632, 1679)]

use super::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FundsView<'a> {
    pub id: u32,
    pub funds: i64,
    /// Weekly rent for this household's home, if rent is charged.
    pub rent: Option<i64>,
    /// Weekly bills, which grow with what the household owns.
    pub bills: Option<i64>,
    /// Build and buy edits that can be undone, and undone edits that can be made again.
    pub undo: usize,
    pub redo: usize,
    /// The household's routine template (the player's household only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) routines: Option<Vec<RoutineView<'a>>>,
    /// A baby on the way: the parents and the day it's due.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expecting: Option<crate::world::Expecting>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RoutineView<'a> {
    pub id: u16,
    pub activity: &'a str,
    pub skill: Option<&'a str>,
    /// Weekdays, bit 0 = Monday.
    pub days: u8,
    pub start: u16,
    pub minutes: u16,
}

pub(crate) fn routine_view<'a>(content: &'a Content, r: &Routine) -> RoutineView<'a> {
    RoutineView {
        id: r.id,
        activity: &content.activities[r.activity].id,
        skill: r.skill.map(|s| content.skills[s].id.as_str()),
        days: r.days,
        start: r.start,
        minutes: r.minutes,
    }
}

/// Why a block wasn't kept: `trait` (with the trait id), `need` (with the need id), `mood`,
/// `away`, `noPlace`.
#[derive(Serialize)]
pub(crate) struct ReasonView<'a> {
    pub code: &'static str,
    #[serde(rename = "trait", skip_serializing_if = "Option::is_none")]
    pub trait_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub need: Option<&'a str>,
}

fn reason_view(content: &Content, r: Reason) -> ReasonView<'_> {
    let (code, trait_id, need) = match r {
        Reason::Trait(t) => ("trait", Some(content.traits[t].id.as_str()), None),
        Reason::Need(n) => ("need", None, Some(content.needs[n].id.as_str())),
        Reason::Mood => ("mood", None, None),
        Reason::Away => ("away", None, None),
        Reason::NoPlace => ("noPlace", None, None),
    };
    ReasonView {
        code,
        trait_id,
        need,
    }
}

/// A block that's running or over: `status` is `active`, `kept`, `cut`, `skipped` or `noPlace`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlockView<'a> {
    pub routine: u16,
    pub household: bool,
    pub activity: &'a str,
    pub skill: Option<&'a str>,
    pub day: u32,
    pub start: u16,
    pub minutes: u16,
    pub status: &'static str,
    pub(crate) reason: Option<ReasonView<'a>>,
    /// Minutes spent on it.
    pub done: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoalView<'a> {
    pub def: &'a str,
    pub kind: GoalKind,
    pub icon: &'a str,
    pub progress: f32,
    /// Change over the last days (positive: getting there).
    pub trend: f32,
    pub skill: Option<&'a str>,
    pub category: Option<&'a str>,
    /// Object id, for things they want.
    pub object: Option<&'a str>,
    pub target: f32,
    pub since_day: u32,
    /// What it still takes: the skill levels a career goal needs (reached ones drop off), the
    /// performance bar, the money for something they want.
    pub(crate) steps: Vec<GoalStepView<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoalStepView<'a> {
    /// `skill`, `performance` or `funds`.
    pub kind: &'static str,
    pub skill: Option<&'a str>,
    pub have: f32,
    pub need: f32,
}

fn goal_view<'a>(world: &'a World, s: &Sim, g: &Goal) -> GoalView<'a> {
    let content = &world.content;
    let def = &content.goals[g.def];
    let mut steps: Vec<GoalStepView> = crate::planner::goal_skills(content, s, g)
        .into_iter()
        .filter(|&(k, l)| s.skills[k] < l)
        .map(|(k, l)| GoalStepView {
            kind: "skill",
            skill: Some(content.skills[k].id.as_str()),
            have: round_to(s.skills[k], PERCENT),
            need: l,
        })
        .collect();
    match def.kind {
        GoalKind::Promoted => {
            if let Some(j) = &s.job {
                steps.push(GoalStepView {
                    kind: "performance",
                    skill: None,
                    have: j.performance.round(),
                    need: 100.0,
                });
            }
        }
        GoalKind::Own => {
            let funds = world.households[s.household as usize].funds;
            steps.push(GoalStepView {
                kind: "funds",
                skill: None,
                have: funds as f32,
                need: g.target,
            });
        }
        _ => {}
    }
    GoalView {
        def: &def.id,
        kind: def.kind,
        icon: &def.icon,
        progress: round_to(g.progress, PERCENT),
        // Rounded away from zero: it passes ±0.01 (where the UI's arrow turns) just when the
        // trend itself does.
        trend: away_from_zero(g.progress - g.history[2], PERCENT),
        skill: g.skill.map(|s| content.skills[s].id.as_str()),
        category: g.category.map(|c| content.career_categories[c].id.as_str()),
        object: g.object.map(|o| content.objects[o].id.as_str()),
        target: g.target,
        since_day: g.since_day,
        steps,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlanView<'a> {
    pub(crate) routines: Vec<RoutineView<'a>>,
    /// Household template blocks this resident doesn't follow.
    pub skip_household: &'a [u16],
    pub(crate) current: Option<BlockView<'a>>,
    /// Blocks of the last week, oldest first.
    pub(crate) history: Vec<BlockView<'a>>,
    pub(crate) goals: Vec<GoalView<'a>>,
    pub(crate) suggestions: Vec<GoalView<'a>>,
    /// Activities they'd like a place for (`skill` for training).
    pub wishes: Vec<(&'a str, Option<&'a str>)>,
    /// What they'd like for the home (as saved: `room` + `factor`, `fix` or `another`).
    pub home_wishes: Vec<crate::save::HomeWishSave>,
    /// How well they stick to plans now (1 = average).
    pub adherence: f32,
    /// What they're asking to buy (waiting for a yes or no).
    pub(crate) purchase: Option<PurchaseView<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PurchaseView<'a> {
    pub object: &'a str,
    pub price: i64,
    /// Times they were told no for it before.
    pub declines: u8,
}

pub(crate) fn plan_view<'a>(world: &'a World, s: &'a Sim) -> PlanView<'a> {
    let content = &world.content;
    let p = &s.planner;
    let today = clock::day(world.tick);
    let block = |routine,
                 household,
                 activity: usize,
                 skill: Option<usize>,
                 day,
                 start,
                 minutes,
                 status,
                 reason: Option<Reason>,
                 done| BlockView {
        routine,
        household,
        activity: &content.activities[activity].id,
        skill: skill.map(|k| content.skills[k].id.as_str()),
        day,
        start,
        minutes,
        status,
        reason: reason.map(|r| reason_view(content, r)),
        // Ten-minute steps: it grows every tick while the block runs, and nothing shows it.
        done: round_to(done, 0.1),
    };
    PlanView {
        routines: p
            .routines
            .iter()
            .map(|r| routine_view(content, r))
            .collect(),
        skip_household: &p.skip_household,
        current: p.run.as_ref().map(|r| {
            let status = if r.no_place {
                "noPlace"
            } else if r.skipped.is_some() {
                "skipped"
            } else {
                "active"
            };
            let reason = if r.no_place {
                Some(Reason::NoPlace)
            } else {
                r.skipped
            };
            block(
                r.routine,
                r.household,
                r.activity,
                r.skill,
                r.day,
                r.start,
                r.minutes,
                status,
                reason,
                r.done,
            )
        }),
        history: p
            .history
            .iter()
            .filter(|b| b.day + 7 >= today)
            .map(|b| {
                let status = match b.outcome {
                    Outcome::Kept => "kept",
                    Outcome::Cut => "cut",
                    Outcome::Skipped => "skipped",
                    Outcome::NoPlace => "noPlace",
                };
                block(
                    b.routine,
                    b.household,
                    b.activity,
                    b.skill,
                    b.day,
                    b.start,
                    b.minutes,
                    status,
                    b.reason,
                    b.done,
                )
            })
            .collect(),
        goals: p.goals.iter().map(|g| goal_view(world, s, g)).collect(),
        suggestions: p
            .suggestions
            .iter()
            .map(|g| goal_view(world, s, g))
            .collect(),
        purchase: p.purchase.filter(|r| r.open).map(|r| PurchaseView {
            object: &content.objects[r.object].id,
            price: content.objects[r.object].price.unwrap_or(0),
            declines: r.declines,
        }),
        wishes: p
            .wishes
            .iter()
            .map(|&(a, k)| {
                (
                    content.activities[a].id.as_str(),
                    k.map(|k| content.skills[k].id.as_str()),
                )
            })
            .collect(),
        home_wishes: p
            .home_wishes
            .iter()
            .map(|w| crate::save::home_wish_save(content, w))
            .collect(),
        // It follows the mood; the UI words it at 0.65, 0.9 and 1.15 (rounding down keeps those).
        adherence: floor_to(crate::planner::adherence(content, s), 20.0),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct JobView<'a> {
    /// Index into the career catalog.
    pub index: usize,
    pub career: &'a str,
    pub career_label: &'a str,
    pub category: Option<&'a str>,
    pub title: &'a str,
    pub level: usize,
    pub levels: usize,
    pub grade: Option<&'a str>,
    pub grade_label: Option<&'a str>,
    pub performance: f32,
    /// What a shift adds to the bar at this pace (the mood they last left for work in), and how
    /// many more it takes to fill it (none at the top of the career, or when shifts don't).
    pub shift_performance: f32,
    pub shifts_to_full_bar: Option<u32>,
    /// The mood they last left for work in (0..1).
    pub shift_mood: f32,
    /// Pay per shift on this Sim's schedule.
    pub pay: i64,
    pub pay_per_hour: f32,
    pub weekly_pay: i64,
    pub start_hour: f32,
    pub hours: f32,
    /// This Sim's workdays (0 = Monday), after skill adjustments.
    pub days: Vec<u8>,
    pub standard_days: Vec<u8>,
    /// Probation / standard / flexible...
    pub workweek: Option<&'a str>,
    /// Skill levels above (+) or below (-) the requirements.
    pub fit: f32,
    pub requires: Vec<Requirement<'a>>,
    pub next: Option<NextLevel<'a>>,
    pub next_title: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NextLevel<'a> {
    pub title: &'a str,
    pub grade: Option<&'a str>,
    pub pay_per_hour: f32,
    pub requires: Vec<Requirement<'a>>,
}

#[derive(Serialize)]
pub(crate) struct Requirement<'a> {
    pub skill: &'a str,
    pub level: f32,
    pub have: f32,
}

pub(crate) fn weekdays(mask: u8) -> Vec<u8> {
    (0..7).filter(|d| mask & (1 << d) != 0).collect()
}

fn requirements<'a>(
    content: &'a Content,
    req: &[(usize, f32)],
    skills: &[f32],
) -> Vec<Requirement<'a>> {
    req.iter()
        .map(|&(s, level)| Requirement {
            skill: &content.skills[s].id,
            level,
            have: skills[s].floor(),
        })
        .collect()
}

/// One of a resident's actions (`held`: what they carry, for the current one).
pub(crate) fn task_view(
    world: &World,
    task: &Task,
    phase: Option<&Phase>,
    held: Option<crate::items::Held>,
) -> ActionView {
    let content = &world.content;
    // The view names objects by stable id.
    let (label, object, target, minutes) = match task.kind {
        TaskKind::Use {
            object,
            interaction,
        } => {
            let inter =
                &content.objects[world.objects[object as usize].def].interactions[interaction];
            let served = world.staff.server_of(object).is_some();
            (
                inter.label.clone(),
                world.stable_of(object),
                None,
                inter.minutes_where(served),
            )
        }
        TaskKind::MoveTo { .. } => ("Walk here".to_owned(), None, None, 1.0),
        TaskKind::Social { target, social } => {
            let s = &content.socials[social];
            let minutes = match phase {
                Some(Phase::Conversing { success, .. }) => conversation::duration(s, *success),
                _ => s.minutes,
            };
            (s.label.clone(), None, Some(target), minutes)
        }
        // Listening in: the conversation's label and progress ("Chat · Bo", the one who started it).
        TaskKind::Join { leader } => {
            let talk = world
                .sims
                .get(leader as usize)
                .and_then(|l| l.conversation(content))
                .filter(|_| matches!(phase, Some(Phase::Conversing { .. })));
            let label = talk.map_or_else(
                || "Join in".to_owned(),
                |(_, social, ..)| content.socials[social].label.clone(),
            );
            return ActionView {
                label,
                object: None,
                target: Some(leader),
                progress: round_to(talk.map_or(0.0, |t| t.3), PERCENT),
                directed: task.directed,
                active: phase.is_some(),
            };
        }
        TaskKind::Work => ("Go to work".to_owned(), None, None, 1.0),
        TaskKind::Visit { plot } => (format!("Visit {}", host_name(world, plot)), None, None, 1.0),
        TaskKind::GoHome => ("Go home".to_owned(), None, None, 1.0),
        TaskKind::Answer { guest } => ("Answer the door".to_owned(), None, Some(guest), 1.0),
        TaskKind::Clean { .. } => {
            let rules = &content.room_rules.clean;
            (rules.label.clone(), None, None, rules.minutes)
        }
        TaskKind::Spot { accident } => {
            let rest = content.accidents[accident].rest.as_ref();
            (
                rest.map_or_else(String::new, |r| r.label.clone()),
                None,
                None,
                rest.map_or(1.0, |r| r.minutes),
            )
        }
        // "Set up the new cookbook lectern".
        TaskKind::SetUp { object, stage, .. } => {
            let rules = &content.object_rules.set_up;
            let name = content
                .objects
                .get(object as usize)
                .map_or_else(String::new, |d| d.name.to_lowercase());
            // (Picking the parcel up is the start of it: no progress yet.)
            let minutes = if stage == 0 {
                f32::INFINITY
            } else {
                rules.minutes
            };
            (
                format!("{} the new {name}", rules.label),
                None,
                None,
                minutes,
            )
        }
        TaskKind::Repair { object } => {
            let rules = &content.object_rules.repair;
            let name = &content.objects[world.objects[object as usize].def].name;
            (
                format!("{} the {}", rules.label, name.to_lowercase()),
                world.stable_of(object),
                None,
                rules.minutes,
            )
        }
        // "Make a meal · Prep (2/3)": the whole recipe's progress.
        TaskKind::Cook {
            recipe,
            step,
            station,
            origin,
            minutes,
            ..
        } => {
            let r = &content.recipes[recipe];
            let label = crate::cooking::origin_interaction(content, &world.objects, origin, recipe)
                .map_or(&r.label, |i| &i.label);
            let i = (step as usize).min(r.steps.len() - 1);
            // (Past the last step: the dish carried to the table, all but done.)
            let serving = step as usize >= r.steps.len();
            let label = match serving {
                true => format!("{label} · {}", content.cooking.serve_label),
                false => format!(
                    "{label} · {} ({}/{})",
                    r.steps[i].label,
                    i + 1,
                    r.steps.len()
                ),
            };
            let done: f32 = r.steps[..i].iter().map(|s| s.minutes).sum();
            let elapsed = match phase {
                Some(Phase::Using { elapsed }) => {
                    elapsed.min(f32::from(minutes)) / f32::from(minutes.max(1)) * r.steps[i].minutes
                }
                _ => 0.0,
            };
            let progress = if serving {
                1.0
            } else {
                ((done + elapsed) / r.minutes).min(1.0)
            };
            return ActionView {
                label,
                object: station.or(origin).and_then(|o| world.stable_of(o)),
                target: None,
                progress: round_to(progress, PERCENT),
                directed: task.directed,
                active: phase.is_some(),
            };
        }
        TaskKind::Eat { item, plate, .. } => {
            let rules = &content.cooking;
            let r = dish(world, held, item, plate).map(|r| &content.recipes[r]);
            let food = r.map_or("something", |r| &content.items[r.item].label);
            (
                format!("{} {food}", rules.eat_label),
                None,
                None,
                r.map_or(rules.eat_minutes, |r| r.eat_minutes),
            )
        }
        TaskKind::Dishes { stage, scraps, .. } => {
            let rules = &content.cooking;
            (
                rules.dishes_label.clone(),
                None,
                None,
                crate::cooking::clearing_minutes(rules, stage, scraps, held).max(1.0),
            )
        }
        // "Call · Bo" (the UI adds who).
        TaskKind::Phone {
            action,
            contact,
            answer,
            ..
        } => {
            let label = content
                .phone
                .actions
                .get(action)
                .map_or_else(String::new, |a| a.label.clone());
            (
                label,
                None,
                contact,
                crate::phone::length(content, action, answer),
            )
        }
        // "Carry the baby to the high chair" (the UI adds who), "Put the garden vegetables in the
        // fridge".
        TaskKind::Carry { baby, to, .. } => {
            let name = world
                .objects
                .get(to as usize)
                .map_or_else(String::new, |o| content.objects[o.def].name.to_lowercase());
            let label = match baby {
                Some(_) => format!("Carry the baby to the {name}"),
                None => {
                    let what = held.map_or_else(
                        || "harvest".to_owned(),
                        |h| content.items[h.def].label.to_lowercase(),
                    );
                    format!("Put the {what} in the {name}")
                }
            };
            let minutes = if baby.is_some() {
                crate::carry::LIFT_MINUTES
            } else {
                crate::carry::STOW_MINUTES
            };
            (label, world.stable_of(to), baby, minutes)
        }
        // "Bring a snack" (the UI adds who): what they take out for a guest, or have in hand.
        TaskKind::Serve {
            guest,
            from,
            interaction,
            stage,
        } => {
            let item = held.map(|h| h.def).or_else(|| {
                let o = world.objects.get(from as usize)?;
                crate::staff::handed(
                    content,
                    content.objects[o.def].interactions.get(interaction)?,
                )
            });
            let what = item.map_or_else(
                || "something".to_owned(),
                |d| some_of(&content.items[d].label),
            );
            let minutes = if stage == 0 {
                world
                    .objects
                    .get(from as usize)
                    .and_then(|o| content.objects[o.def].interactions.get(interaction))
                    .map_or(1.0, |i| i.minutes.max(0.5))
            } else {
                crate::hosting::HAND_MINUTES
            };
            let object = (stage == 0).then(|| world.stable_of(from)).flatten();
            (format!("Bring {what}"), object, Some(guest), minutes)
        }
    };
    let progress = match phase {
        Some(Phase::Using { elapsed }) | Some(Phase::Conversing { elapsed, .. }) => {
            (elapsed / minutes).min(1.0)
        }
        _ => 0.0,
    };
    ActionView {
        label,
        object,
        target,
        progress: round_to(progress, PERCENT),
        directed: task.directed,
        active: phase.is_some(),
    }
}

pub(crate) fn job_view<'a>(content: &'a Content, s: &Sim) -> Option<JobView<'a>> {
    let j = s.job.as_ref()?;
    let career = &content.careers[j.career];
    let level = &career.levels[j.level];
    let grade = level.grade.map(|g| &content.grades[g]);
    let fit = life::skill_fit(level, &s.skills);
    let next = career.levels.get(j.level + 1);
    Some(JobView {
        index: j.career,
        career: &career.id,
        career_label: &career.label,
        category: career
            .category
            .map(|c| content.career_categories[c].id.as_str()),
        title: &level.title,
        level: j.level,
        levels: career.levels.len(),
        grade: grade.map(|g| g.id.as_str()),
        grade_label: grade.map(|g| g.label.as_str()),
        performance: j.performance,
        shift_performance: round_to(
            life::shift_performance(&content.career_rules, j.level, j.shift_mood, fit, false),
            PERCENT,
        ),
        shifts_to_full_bar: life::shifts_to_full_bar(content, j, &s.skills),
        shift_mood: round_to(j.shift_mood, PERCENT),
        pay: life::shift_pay(content, j, &s.skills),
        pay_per_hour: level.pay as f32 / level.hours,
        weekly_pay: level.pay * level.days.count_ones() as i64,
        start_hour: level.start_hour,
        hours: level.hours,
        days: weekdays(life::work_days(content, j, &s.skills)),
        standard_days: weekdays(level.days),
        workweek: life::workweek_rule(content, fit).map(|r| r.label.as_str()),
        fit,
        requires: requirements(content, &level.requires, &s.skills),
        next: next.map(|n| NextLevel {
            title: &n.title,
            grade: n.grade.map(|g| content.grades[g].id.as_str()),
            pay_per_hour: n.pay as f32 / n.hours,
            requires: requirements(content, &n.requires, &s.skills),
        }),
        next_title: next.map(|l| l.title.as_str()),
    })
}
