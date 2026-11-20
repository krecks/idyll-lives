use super::*;

// ---- Autonomy --------------------------------------------------------------------------------

/// What the phone offers `sim` right now, scored like everything else in `pick_autonomous`
/// (`feel`: traits, emotion and the time of day): `(score, action, contact)`. Talking with
/// people in the same place is for face to face; meeting up is scored like a visit.
pub(crate) fn candidates(
    sim: &Sim,
    ctx: &Ctx,
    felt: &[f32],
    feel: &dyn Fn(TagMask) -> f32,
) -> Vec<(f32, usize, Option<u32>)> {
    let content = ctx.content;
    let mut out = Vec::new();
    // (Guests spend their time with their hosts.)
    if !has_phone(content, sim.age) || sim.visiting.is_some() {
        return out;
    }
    let me = sim.id as usize;
    let here = ctx.briefs[me].plot;
    let needs = &sim.needs[..content.needs.len()];
    let household = &ctx.households[sim.household as usize];
    let visits = &content.visits;
    let visiting_hours = (visits.earliest_hour..visits.latest_hour).contains(&ctx.hour);
    let mut soon = None;
    for (a, act) in content
        .phone
        .actions
        .iter()
        .enumerate()
        .filter(|(_, a)| a.autonomous)
    {
        if !act.contact {
            // Scrolling: a pastime like any other (a planned block for it counts).
            let (plan, floor) = crate::planner::tags_factor(content, sim, act.tags);
            let s = (crate::ai::score_felt(needs, felt, &act.gains, 0.0) * act.autonomy_weight)
                .max(floor)
                * feel(act.tags)
                * plan;
            out.push((s, a, None));
            continue;
        }
        if act.meet.is_some() && (!visiting_hours || here.is_none()) {
            continue;
        }
        for (j, other) in ctx.briefs.iter().enumerate() {
            if j == me
                || !other.here
                || !has_phone(content, other.age)
                || (other.plot == here && !other.away)
            {
                continue;
            }
            // A call needs them free; a message, awake (asleep, they're neither).
            let reachable = other.available || (act.message && other.away);
            let rel = ctx.rels.get(me, j);
            if !reachable || !act.requires.allows(rel) {
                continue;
            }
            let gains = match act.meet {
                None => crate::ai::score_felt(needs, felt, &act.gains, 0.0),
                Some(meet) => {
                    let their_home = ctx.households[other.household as usize].plot;
                    let ok = match meet {
                        Meet::Here => here == household.plot && other.household != sim.household,
                        // (Not leaving a baby home alone: `life::minding_baby`.)
                        Meet::There => {
                            their_home.is_some()
                                && other.plot == their_home
                                && other.household != sim.household
                                && !crate::life::minding_baby(sim, ctx)
                        }
                    };
                    if !ok || rel.friendship < visits.min_friendship {
                        continue;
                    }
                    let soon = *soon.get_or_insert_with(|| {
                        crate::planner::plans_soon(
                            content,
                            sim,
                            &household.routines,
                            ctx.day,
                            ctx.minute,
                            visits.hours * 60.0,
                        )
                    });
                    // Who travels doesn't matter much on the phone: like a visit next door.
                    crate::ai::score_felt(needs, felt, &visits.gains, 0.0)
                        * (0.4 + rel.friendship / 100.0)
                        * feel(visits.tags)
                        * crate::planner::visit_factor(content, sim, soon)
                }
            };
            let friendly = matches!(act.prefer, Prefer::Any | Prefer::Liked);
            let s = gains
                * act.autonomy_weight
                * prefer_factor(act.prefer, rel, ctx.rels.chemistry(me, j), 0.0)
                * feel(act.tags)
                * crate::planner::social_factor(
                    content,
                    sim,
                    false,
                    friendly,
                    rel.friendship,
                    false,
                );
            out.push((s, a, Some(j as u32)));
        }
    }
    out
}

/// How long a phone action lasts, as it went: a no is a short call; nobody answering, a few rings.
pub(crate) fn length(content: &Content, action: usize, answer: Answer) -> f32 {
    let Some(act) = content.phone.actions.get(action) else {
        return 1.0;
    };
    match answer {
        Answer::No => act.minutes * crate::conversation::FAILURE_LENGTH,
        Answer::NoAnswer => content.phone.no_answer_minutes.min(act.minutes),
        _ => act.minutes,
    }
}
