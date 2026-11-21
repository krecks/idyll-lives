//! Outcomes: applying them, breaking up, the gains and the helpers.

use super::*;
#[allow(clippy::too_many_arguments)]
pub(super) fn apply_outcome(
    content: &Content,
    rels: &mut Relationships,
    events: &mut EventLog,
    rng: &mut Rng,
    sims: &mut [Sim],
    a: usize,
    t: usize,
    social: usize,
    success: bool,
    tick: u64,
) {
    let s = &content.socials[social];
    let o = if success { &s.success } else { &s.failure };
    outcome(
        content, rels, events, rng, sims, a, t, o, s.tags, success, tick,
    );
}

/// What came of something `a` did with `t` (a social, a phone call): relationship changes both
/// ways, needs, feelings, the outcome's effect, milestones and jealousy.
#[allow(clippy::too_many_arguments)]
pub(crate) fn outcome(
    content: &Content,
    rels: &mut Relationships,
    events: &mut EventLog,
    rng: &mut Rng,
    sims: &mut [Sim],
    a: usize,
    t: usize,
    o: &social::Outcome,
    tags: crate::content::TagMask,
    success: bool,
    tick: u64,
) {
    let defs = &content.feelings;
    let rules = &content.social_rules;
    let before = (*rels.get(a, t), *rels.get(t, a));

    rels.adjust(a, t, o.friendship, o.romance);
    rels.adjust(t, a, o.target_friendship, o.target_romance);
    {
        let (actor, target) = pair_mut(sims, a, t);
        nudge(actor, &o.needs);
        nudge(target, &o.target_needs);
        if let Some(m) = o.feeling {
            social::add_feeling(&mut actor.feelings, m, defs, tick);
        }
        if let Some(m) = o.target_feeling {
            social::add_feeling(&mut target.feelings, m, defs, tick);
        }
    }

    match o.effect {
        Effect::None => {}
        // (Off to bed together, where the content has beds for it: `privacy::after_step`.)
        Effect::Conceive => {
            if success {
                sims[a].to_bed = Some(t as u32);
            }
        }
        // (Off to fetch it: `hosting::update`.)
        Effect::Serve => {
            if success {
                sims[a].serve_for = Some(t as u32);
            }
        }
        Effect::Kiss => {
            if success && !rels.get(a, t).kissed {
                rels.get_mut(a, t).kissed = true;
                rels.get_mut(t, a).kissed = true;
                events.push(tick, EventKind::FirstKiss, a, t, None);
            }
        }
        Effect::BecomePartners => {
            if success {
                // Starting something new ends any existing relationship, painfully for the ex.
                for x in [a, t] {
                    if let Some(ex) = rels.partner_of(x).filter(|&p| p != a && p != t) {
                        break_up(
                            rels,
                            events,
                            sims,
                            defs,
                            rules.heartbreak_feeling,
                            x,
                            ex,
                            tick,
                        );
                    }
                }
                rels.set_partners(a, t, true);
                events.push(tick, EventKind::StartedDating, a, t, None);
            } else {
                events.push(tick, EventKind::ProposalRejected, a, t, None);
            }
        }
        Effect::BreakUp => {
            if rels.get(a, t).partners {
                break_up(
                    rels,
                    events,
                    sims,
                    defs,
                    rules.heartbreak_feeling,
                    a,
                    t,
                    tick,
                );
            }
        }
        Effect::Fight => {
            let strength = |sim: &Sim| 0.3 + sim.mood(content) * 0.5 + sim.mods.walk_speed * 0.2;
            let (sa, st) = (strength(&sims[a]), strength(&sims[t]));
            let (winner, loser) = if rng.next_f32() * (sa + st) < sa {
                (a, t)
            } else {
                (t, a)
            };
            if let Some(m) = o.winner_feeling {
                social::add_feeling(&mut sims[winner].feelings, m, defs, tick);
            }
            if let Some(m) = o.loser_feeling {
                social::add_feeling(&mut sims[loser].feelings, m, defs, tick);
            }
            events.push(tick, EventKind::Fight, winner, loser, None);
        }
    }

    for (kind, reversed) in
        social::milestone_events((&before.0, &before.1), (rels.get(a, t), rels.get(t, a)))
    {
        let (x, y) = if reversed { (t, a) } else { (a, t) };
        events.push(tick, kind, x, y, None);
    }

    // Romance in front of a partner causes jealousy.
    if success && tags & rules.romantic_tags != 0 {
        for b in 0..sims.len() {
            if b == a || b == t || sims[b].pose == Pose::Lie {
                continue;
            }
            for (x, y) in [(a, t), (t, a)] {
                if rels.get(b, x).partners && dist(sims[b].pos, sims[x].pos) <= rules.jealousy_range
                {
                    rels.adjust(b, x, rules.jealousy_friendship, rules.jealousy_romance);
                    if let Some(m) = rules.jealousy_feeling {
                        social::add_feeling(&mut sims[b].feelings, m, defs, tick);
                    }
                    events.push(tick, EventKind::Jealous, b, x, Some(y));
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn break_up(
    rels: &mut Relationships,
    events: &mut EventLog,
    sims: &mut [Sim],
    defs: &[social::FeelingDef],
    heartbreak: Option<usize>,
    leaver: usize,
    left: usize,
    tick: u64,
) {
    rels.set_partners(leaver, left, false);
    rels.adjust(left, leaver, -20.0, -40.0);
    rels.adjust(leaver, left, -5.0, -20.0);
    if let Some(m) = heartbreak {
        social::add_feeling(&mut sims[left].feelings, m, defs, tick);
    }
    events.push(tick, EventKind::BrokeUp, leaver, left, None);
}

pub(super) fn gain(sim: &mut Sim, total: &[f32; MAX_NEEDS], minutes: f32) {
    for (n, g) in total.iter().enumerate() {
        if *g != 0.0 {
            sim.needs[n] = (sim.needs[n] + g / minutes * MINUTES_PER_TICK * sim.mods.need_gain[n])
                .clamp(0.0, 1.0);
        }
    }
}

fn nudge(sim: &mut Sim, delta: &[f32; MAX_NEEDS]) {
    for (n, d) in delta.iter().enumerate() {
        sim.needs[n] = (sim.needs[n] + d).clamp(0.0, 1.0);
    }
}

pub(super) fn pair_mut(sims: &mut [Sim], a: usize, b: usize) -> (&mut Sim, &mut Sim) {
    assert_ne!(a, b);
    if a < b {
        let (l, r) = sims.split_at_mut(b);
        (&mut l[a], &mut r[0])
    } else {
        let (l, r) = sims.split_at_mut(a);
        (&mut r[0], &mut l[b])
    }
}

pub(super) fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

pub(super) fn facing(from: [f32; 2], to: [f32; 2]) -> f32 {
    (to[0] - from[0]).atan2(to[1] - from[1])
}
