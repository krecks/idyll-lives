use super::*;

// ---- Using the phone -------------------------------------------------------------------------

/// Gets the phone out: on a free seat close by if there's one (walking over first), else
/// standing where they are.
pub(crate) fn start(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let content = ctx.content;
    let TaskKind::Phone {
        action, contact, ..
    } = task.kind
    else {
        return;
    };
    let Some(act) = content.phone.actions.get(action) else {
        return;
    };
    if !has_phone(content, sim.age) {
        return;
    }
    let task = Task {
        kind: TaskKind::Phone {
            action,
            contact,
            seat: None,
            answer: Answer::Pending,
        },
        ..task
    };
    if let Some(seat) = seat_near(sim, ctx, objects) {
        let o = &objects[seat as usize];
        let def = &content.objects[o.def];
        let footstool = crate::world::footstool_slots(content, objects, seat);
        let slot = o.pick_slot(
            sim.id,
            0..def.slots as usize,
            Some((def, sim.pos)),
            footstool,
        );
        let (fx, fz) = slot
            .and_then(|k| crate::surface::seat_approach(content, &ctx.nav, o, k, sim.pos))
            .or_else(|| ctx.index.links.approach_of(seat))
            .unwrap_or_else(|| o.front_tile(content));
        if let Some(waypoints) = walk_route(ctx, sim, (fx, fz), [fx as f32 + 0.5, fz as f32 + 0.5])
        {
            let o = &mut objects[seat as usize];
            if o.reserve_seat(sim.id, &content.objects[o.def], sim.pos, footstool) {
                let task = Task {
                    kind: TaskKind::Phone {
                        action,
                        contact,
                        seat: Some(seat),
                        answer: Answer::Pending,
                    },
                    ..task
                };
                sim.pose = Pose::Stand;
                sim.current = Some(Activity {
                    task,
                    phase: Phase::Routing { waypoints, next: 0 },
                    tags: act.tags,
                });
                return;
            }
        }
    }
    sim.pose = Pose::Stand;
    sim.current = Some(Activity {
        task,
        phase: Phase::Using { elapsed: 0.0 },
        tags: act.tags,
    });
}

/// The nearest free seat within `seat_range` that they may sit on here.
fn seat_near(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> Option<u32> {
    let content = ctx.content;
    let rules = &content.phone;
    if rules.seat_range <= 0.0 {
        return None;
    }
    let usable = crate::world::usable_here(sim, ctx);
    usable
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| {
            let def = &content.objects[o.def];
            !o.broken()
                && o.can_use(sim.id, def.slots)
                && o.sits(
                    def,
                    content.life.stage(sim.age),
                    rules.seat_tags,
                    rules.seat_avoid,
                )
        })
        .filter(|o| access(ctx, sim, ctx.object_plot[o.id as usize]) != Access::None)
        .map(|o| {
            let (fx, fz) = ctx
                .index
                .links
                .approach_of(o.id)
                .unwrap_or_else(|| o.front_tile(content));
            (
                o.id,
                ctx.nav
                    .stairs
                    .distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos),
            )
        })
        .filter(|&(_, d)| d <= rules.seat_range)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(o, _)| o)
}

/// At the seat: sit down with the phone.
pub(crate) fn arrive(sim: &mut Sim, ctx: &Ctx, objects: &[ObjectInstance]) {
    let Some(TaskKind::Phone { seat, .. }) = sim.current.as_ref().map(|a| a.task.kind) else {
        return;
    };
    if let Some(o) = seat.map(|s| &objects[s as usize]) {
        // They stand back up where they came from (see `end_activity`).
        sim.seated_from = Some(sim.tile());
        let slot = o.slot_of(sim.id).unwrap_or(0);
        sim.pos = o.slot_position(ctx.content, slot);
        sim.yaw = o.slot_yaw(ctx.content, slot);
        sim.pose = Pose::Sit;
    }
    if let Some(act) = sim.current.as_mut() {
        act.phase = Phase::Using { elapsed: 0.0 };
    }
}

/// A tick on the phone (`minutes` long; more when catching up on a turn, see `lod.rs`): needs
/// fill while connected. Returns whether they're done (the ending is applied after the step).
pub(crate) fn using(
    sim: &mut Sim,
    content: &Content,
    task: Task,
    elapsed: f32,
    minutes: f32,
) -> bool {
    let TaskKind::Phone {
        action,
        contact,
        answer,
        ..
    } = task.kind
    else {
        return true;
    };
    let Some(act) = content.phone.actions.get(action) else {
        return true;
    };
    // With someone: once it's decided whether they answer (`update`).
    if contact.is_some() && answer == Answer::Pending {
        return false;
    }
    let connected = contact.is_none() || answer == Answer::Yes;
    if connected {
        for (n, g) in act.gains.iter().enumerate().take(content.needs.len()) {
            if *g != 0.0 {
                sim.needs[n] = (sim.needs[n] + g / act.minutes * minutes * sim.mods.need_gain[n])
                    .clamp(0.0, 1.0);
            }
        }
    }
    // Scrolling stops once fun is full, like any pastime.
    let full = contact.is_none()
        && (0..content.needs.len())
            .filter(|&n| act.gains[n] > 0.0)
            .all(|n| sim.needs[n] >= 0.999)
        && act.gains.iter().any(|&g| g > 0.0);
    let done = elapsed >= length(content, action, answer) || full;
    if done {
        sim.hung_up = Some(HungUp {
            action,
            contact,
            answer,
            directed: task.directed,
        });
    }
    done
}

/// After everyone's step: whoever just got their phone out with someone on the other end
/// learns whether they answer, and phone actions that ended have their effect on the contact,
/// the relationship and, after a yes to meeting, who goes where.
pub(crate) fn update(w: &mut World) {
    if w.content.phone.actions.is_empty() {
        return;
    }
    let tick = w.tick;
    for i in 0..w.sims.len() {
        let Some(Activity {
            task:
                Task {
                    kind:
                        TaskKind::Phone {
                            action,
                            contact: Some(c),
                            answer: Answer::Pending,
                            ..
                        },
                    ..
                },
            phase: Phase::Using { .. },
            ..
        }) = w.sims[i].current.as_ref()
        else {
            continue;
        };
        let (action, c) = (*action, *c as usize);
        let answer = if why_not(w, i, c, action).is_some() {
            Answer::NoAnswer
        } else {
            let act = &w.content.phone.actions[action];
            let chance = crate::conversation::accepts(
                &w.content,
                &w.relationships,
                &w.sims[i],
                &w.sims[c],
                &act.acceptance,
                act.tags,
            );
            if w.rng.next_f32() < chance {
                Answer::Yes
            } else {
                Answer::No
            }
        };
        if let Some(Activity {
            task:
                Task {
                    kind: TaskKind::Phone { answer: a, .. },
                    ..
                },
            ..
        }) = w.sims[i].current.as_mut()
        {
            *a = answer;
        }
    }
    // (Those asked over who give up a taxi booked out to a place of their own: `taxi::release`.)
    let mut unbook = Vec::new();
    for i in 0..w.sims.len() {
        let Some(h) = w.sims[i].hung_up.take() else {
            continue;
        };
        let Some(c) = h
            .contact
            .map(|c| c as usize)
            .filter(|&c| c != i && w.sims.get(c).is_some_and(Sim::here))
        else {
            continue;
        };
        // Whoever would set off for the other's home only goes if the way is one to make now
        // (not a walk of hours, there in visiting hours: `World::visit_fits_now`), and the one
        // asked over not if it'd leave a baby home alone (`life::no_cover_for`; the caller was
        // checked as they called).
        let meet = w.content.phone.actions[h.action].meet;
        let home_of = |s: usize| w.households[w.sims[s].household as usize].plot;
        let can_meet = match meet {
            Some(Meet::Here) => {
                home_of(i).is_some_and(|p| w.visit_fits_now(c, p))
                    && crate::life::no_cover_for(w, c).is_none()
            }
            Some(Meet::There) => h.directed || home_of(c).is_some_and(|p| w.visit_fits_now(i, p)),
            None => true,
        };
        let World {
            content,
            sims,
            objects,
            relationships,
            events,
            rng,
            households,
            taxis,
            ..
        } = &mut *w;
        let act = &content.phone.actions[h.action];
        match h.answer {
            Answer::Yes => {
                crate::conversation::outcome(
                    content,
                    relationships,
                    events,
                    rng,
                    sims,
                    i,
                    c,
                    &act.success,
                    act.tags,
                    true,
                    tick,
                );
                let t = &mut sims[c];
                for (n, g) in act
                    .contact_gains
                    .iter()
                    .enumerate()
                    .take(content.needs.len())
                {
                    t.needs[n] = (t.needs[n] + g * t.mods.need_gain[n]).clamp(0.0, 1.0);
                }
            }
            Answer::No => crate::conversation::outcome(
                content,
                relationships,
                events,
                rng,
                sims,
                i,
                c,
                &act.failure,
                act.tags,
                false,
                tick,
            ),
            Answer::NoAnswer | Answer::Pending => continue,
        }
        let Some(meet) = act.meet else { continue };
        // Invitations are news for the player's household only (across a whole town they would
        // crowd out everything else).
        let ours = households[sims[i].household as usize].player
            || households[sims[c].household as usize].player;
        if h.answer == Answer::No || !can_meet {
            if ours {
                events.push(tick, EventKind::Declined, i, c, None);
            }
            continue;
        }
        match meet {
            // The contact sets off for the caller's home (dropping what they chose themselves).
            Meet::Here => {
                let Some(home) = households[sims[i].household as usize].plot else {
                    continue;
                };
                let t = &mut sims[c];
                if t.current.as_ref().is_some_and(|a| !a.task.directed) {
                    end_activity(t, content, objects);
                }
                // (Nor an outing of their own still to go on with, queued: on a bike to a place,
                // asked over, they were let in and off to the place a moment after, Oakwood.)
                t.queue.retain(|q| {
                    q.directed || !matches!(q.kind, TaskKind::Visit { plot } if plot != home)
                });
                // (Nor one they're booked on a taxi for, or about to be: the taxi still came for
                // them and took them to the place, and they were off to the caller's a few minutes
                // after getting there, Alder Mill.)
                if matches!(
                    t.transition,
                    Some(crate::life::Transition::Taxi { plot, directed: false }) if plot != home
                ) {
                    t.transition = None;
                }
                let elsewhere = |r: &crate::taxi::Run, stop: u8| {
                    !r.home
                        && !r.directed
                        && r.place.is_some_and(|p| p != home)
                        && r.stops.get(usize::from(stop)).is_some_and(|s| {
                            s.sim == c as u32 && s.state != crate::taxi::StopState::Aboard
                        })
                };
                if t.taxi.is_some_and(|b| {
                    taxis
                        .runs
                        .iter()
                        .any(|r| r.id == b.run && elsewhere(r, b.stop))
                }) {
                    unbook.push(c);
                }
                // (Not as well as a visit there already on its way: they went twice, knocking
                // again just let in, Mirrorpool.)
                let there = |k: &TaskKind| matches!(k, TaskKind::Visit { plot } if *plot == home);
                if !t.queue.iter().any(|q| there(&q.kind))
                    && !t.current.as_ref().is_some_and(|a| there(&a.task.kind))
                {
                    t.queue.push_front(Task {
                        kind: TaskKind::Visit { plot: home },
                        directed: false,
                        routine: false,
                    });
                }
                if ours {
                    events.push(tick, EventKind::Invited, i, c, None);
                }
            }
            // Over to theirs.
            Meet::There => {
                let Some(home) = households[sims[c].household as usize].plot else {
                    continue;
                };
                let me = &mut sims[i];
                let there = |k: &TaskKind| matches!(k, TaskKind::Visit { plot } if *plot == home);
                if !me.queue.iter().any(|q| there(&q.kind))
                    && !me.current.as_ref().is_some_and(|a| there(&a.task.kind))
                {
                    me.queue.push_front(Task {
                        kind: TaskKind::Visit { plot: home },
                        directed: h.directed,
                        routine: false,
                    });
                }
            }
        }
    }
    for c in unbook {
        crate::taxi::release(w, c, false);
    }
}
