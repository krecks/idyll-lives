//! A host and their guest (`visits.offer`): a while after a guest comes in, a host who is free
//! asks them if they'd like something (the offer's social, `"effect": "serve"` on success; a
//! hungry guest says yes more readily, `social::Acceptance::needs`). Taken up, the host goes to
//! what home has for a bite or a treat (an `eat` use a guest could have themselves: the fridge's
//! snack, the toaster's toast), takes it out (stage 0), carries it in the hands to the guest and
//! hands it over (stage 1 on, `world::TaskKind::Serve`), after the guest if they move on; the
//! guest gets nothing to eat of their own meanwhile (`Sim::expecting`). They sit down with it, at a
//! table if there is one, as with anything they'd fetched themselves (`cooking::eat_now`), and
//! the plate is the host household's to wash up.
//!
//! A host offers once in a while (`againMinutes`), only with something at home to bring, and
//! never to a guest who is busy, leaving, eating or holding something already. The guest
//! leaving, or the host called away, it's dropped: what the host holds is put down
//! (`cooking::settle`), and the guest may get something of their own again (`nothing_coming`).
//! Nothing of it is saved: a load finds it over.

use crate::content::{Content, Pose};
use crate::social::EventKind;
use crate::world::{
    Activity, Ctx, ObjectInstance, Phase, Sim, Task, TaskKind, World, end_activity, walk_route,
};

/// Game minutes handing it over.
pub const HAND_MINUTES: f32 = 1.0;
/// How long a host with it in hand waits by a guest busy with something else (a conversation,
/// the toilet) before putting it down.
pub const WAIT_MINUTES: f32 = 15.0;
/// How near (m) the host is to the guest to hand it over.
const REACH: f32 = 1.7;
/// How many times a host sets off after a guest who moved on (stages past the first).
const TRIES: u8 = 20;
/// No offer with less of the visit than this left (minutes).
const LAST_MINUTES: f32 = 20.0;
/// How long a guest who said yes waits for it before they'd get something themselves.
const EXPECT_MINUTES: f32 = 30.0;
/// No host free to ask, or nothing at home to bring: asked again after this long (minutes).
const RETRY_MINUTES: f32 = 1.0;

/// Whether `s` is on their way home, or saying goodbye to go (`visits.farewell`).
fn leaving(content: &Content, s: &Sim) -> bool {
    let going = |t: &Task| match t.kind {
        TaskKind::GoHome => true,
        TaskKind::Social { social, .. } => !t.directed && content.visits.farewell.contains(&social),
        _ => false,
    };
    s.current.as_ref().is_some_and(|a| going(&a.task)) || s.queue.iter().any(going)
}

/// Whether `s` has free will (the household's and the town's).
fn free_will(w: &World, s: &Sim) -> bool {
    w.autonomy && w.households[s.household as usize].free_will
}

/// The plot `s` stands on (if they're still in town).
fn plot_of(w: &World, s: &Sim) -> Option<u32> {
    let (x, z) = s.tile();
    w.plot_at(x, z).filter(|_| s.here())
}

/// Whether guest `g` is a guest on `plot`, there now, and not on their way home.
fn guest_there(w: &World, g: usize, plot: u32) -> bool {
    let Some(s) = w.sims.get(g) else {
        return false;
    };
    s.here()
        && s.away_until.is_none()
        && s.visiting.is_some_and(|v| v.plot == plot)
        && plot_of(w, s) == Some(plot)
        && !leaving(&w.content, s)
}

/// Whether guest `s` could take something handed to them now: free for a word (not busy,
/// lying down, in a conversation or doing what the player asked), holding nothing, not eating.
fn free_to_take(content: &Content, s: &Sim) -> bool {
    s.held.is_none()
        && s.paused.is_none()
        && s.listening_to().is_none()
        && s.available_for_social(content)
        && !s
            .current
            .as_ref()
            .is_some_and(|a| matches!(a.task.kind, TaskKind::Eat { .. }))
}

/// What host `h` would bring guest `g` on `plot` (their home): the `eat` use (an object and its
/// interaction) that does the guest the most good, one they may have there (as a guest may), for
/// their age and costing nothing; working, free for the host and within their reach. On a tie,
/// the nearest the host. None: nothing to bring.
fn fare(w: &World, h: usize, g: usize, plot: u32) -> Option<(u32, usize)> {
    let content = &w.content;
    let (host, guest) = (&w.sims[h], &w.sims[g]);
    let hour = crate::clock::hour(w.tick);
    let reach = content.life.reach(host.age);
    let guest_access = crate::world::access_in(&w.households, &w.plots, guest, Some(plot));
    if guest_access == crate::world::Access::None {
        return None;
    }
    let mut best: Option<(f32, f32, u32, usize)> = None;
    for &o in w.object_index.on(plot) {
        let obj = &w.objects[o as usize];
        let def = &content.objects[obj.def];
        if obj.broken() || def.reach > reach {
            continue;
        }
        for (k, inter) in def.interactions.iter().enumerate() {
            let Some(recipe) = inter.eats else { continue };
            // (Nothing to work on is asked of food.)
            if inter.cost > 0
                || (guest_access == crate::world::Access::Guest
                    && !crate::world::guest_may(content, def, inter))
                || crate::world::unfit(content, guest.age, inter).is_some()
                || crate::world::unmet(&w.lot, hour, obj, content, inter, u8::MAX).is_some()
                || crate::world::missing_requirement(
                    content,
                    &w.object_plot,
                    Some(&w.object_index),
                    &w.objects,
                    o,
                    inter,
                )
                .is_some()
                || !obj.can_use_for(host.id, def, k)
                || !obj.allows(inter, host.id)
            {
                continue;
            }
            let effects = &content.recipes[recipe].effects;
            let good: f32 = (0..content.needs.len())
                .filter(|&n| !content.needs[n].room)
                .map(|n| effects[n].max(0.0).min(1.0 - guest.needs[n]))
                .sum();
            let c = obj.centre(content);
            let d = (c[0] - host.pos[0]).hypot(c[1] - host.pos[1]);
            if best.is_none_or(|b| good > b.0 || (good == b.0 && d < b.1)) {
                best = Some((good, d, o, k));
            }
        }
    }
    best.map(|(.., o, k)| (o, k))
}

/// Whether host `s` is free to ask a guest: at home, awake, up and about, holding nothing,
/// not in a conversation, and doing nothing or only something they picked themselves that
/// isn't busy (the TV, tidying up), with nothing the player asked for next.
fn free_to_offer(w: &World, s: &Sim, plot: u32) -> bool {
    let content = &w.content;
    let busy = content.social_rules.busy_tags | content.day_rhythm.sleep_tags;
    s.here()
        && s.away_until.is_none()
        && s.visiting.is_none()
        && s.held.is_none()
        && s.paused.is_none()
        && s.carried_by.is_none()
        && plot_of(w, s) == Some(plot)
        && free_will(w, s)
        && s.available_for_social(content)
        && !crate::life::leaving_home(w, s)
        && !s.queue.front().is_some_and(|t| t.directed)
        && s.current.as_ref().is_none_or(|a| {
            !a.task.directed
                && a.tags & busy == 0
                && matches!(
                    a.task.kind,
                    TaskKind::Use { .. } | TaskKind::MoveTo { .. } | TaskKind::Clean { .. }
                )
        })
}

/// After everyone's step and the conversations: what was handed over changes hands, offers
/// taken up send the host off for it, serves for guests who are leaving are dropped, guests
/// nothing is coming for any more stop waiting for it, and free hosts offer their guests
/// something.
pub(crate) fn update(w: &mut World) {
    hand_over(w);
    fetch(w);
    drop_stale(w);
    nothing_coming(w);
    offer(w);
}

/// The guest a serve is for.
fn serving(t: &Task) -> Option<u32> {
    match t.kind {
        TaskKind::Serve { guest, .. } => Some(guest),
        _ => None,
    }
}

/// A guest waiting for something to eat (`Sim::expecting`) that no host is bringing them any
/// more (the serve dropped, ended short, the host called away, sent somewhere else or gone): they
/// may get something of their own again.
fn nothing_coming(w: &mut World) {
    let tick = w.tick;
    for g in 0..w.sims.len() {
        if w.sims[g].expecting <= tick {
            continue;
        }
        let me = Some(g as u32);
        let coming = w.sims.iter().any(|s| {
            s.current
                .iter()
                .chain(&s.paused)
                .map(|a| &a.task)
                .chain(&s.queue)
                .any(|t| serving(t) == me)
        });
        if !coming {
            w.sims[g].expecting = 0;
        }
    }
}

/// What a host handed over (`Sim::hand_to`) is the guest's now: they drop what they picked to do
/// and sit down with it (`cooking::queued` starts the meal on their step). A guest who can't
/// take it after all (busy, leaving, holding something): the host keeps it, and puts it down
/// (`cooking::settle`).
fn hand_over(w: &mut World) {
    for h in 0..w.sims.len() {
        let Some(g) = w.sims[h].hand_to.take() else {
            continue;
        };
        let g = g as usize;
        let Some(held) = w.sims[h].held else {
            continue;
        };
        let Some(plot) = plot_of(w, &w.sims[h]) else {
            continue;
        };
        if !guest_there(w, g, plot) || !free_to_take(&w.content, &w.sims[g]) {
            continue;
        }
        let tick = w.tick;
        let World {
            sims,
            content,
            objects,
            events,
            ..
        } = &mut *w;
        sims[h].held = None;
        let guest = &mut sims[g];
        if guest.current.is_some() {
            end_activity(guest, content, objects);
        }
        guest.held = Some(held);
        guest.expecting = 0;
        guest.pose = Pose::Stand;
        guest.current = Some(Activity {
            task: Task {
                kind: TaskKind::Eat {
                    item: None,
                    seat: None,
                    plate: None,
                    stage: 1,
                },
                directed: false,
                routine: false,
            },
            phase: Phase::Queued { ticks: 0 },
            tags: content.cooking.eat_tags,
        });
        events.push_amount(tick, EventKind::Served, h, g, held.def as i64);
    }
}

/// Offers taken up (`Sim::serve_for`): the host sets off for what they'll bring (nothing to
/// bring after all: nothing happens).
fn fetch(w: &mut World) {
    for h in 0..w.sims.len() {
        let Some(g) = w.sims[h].serve_for.take() else {
            continue;
        };
        let g = g as usize;
        let s = &w.sims[h];
        let home = w.households[s.household as usize].plot;
        let Some(plot) = plot_of(w, s).filter(|&p| home == Some(p)) else {
            continue;
        };
        if !guest_there(w, g, plot) || s.held.is_some() {
            continue;
        }
        let Some((from, interaction)) = fare(w, h, g, plot) else {
            continue;
        };
        let tick = w.tick;
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        // (The guest waits for it: nothing to eat of their own meanwhile.)
        sims[g].expecting = tick + crate::social::minutes_to_ticks(EXPECT_MINUTES);
        let s = &mut sims[h];
        if s.current.as_ref().is_some_and(|a| !a.task.directed) {
            end_activity(s, content, objects);
        }
        s.queue.push_front(Task {
            kind: TaskKind::Serve {
                guest: g as u32,
                from,
                interaction,
                stage: 0,
            },
            directed: false,
            routine: false,
        });
    }
}

/// Serves for guests who are leaving (or gone, or no longer guests here) are given up: the host
/// stops, and what they hold is put down (`cooking::settle`).
fn drop_stale(w: &mut World) {
    for h in 0..w.sims.len() {
        let s = &w.sims[h];
        let serving = |t: &Task| serving(t).map(|g| g as usize);
        let current = s.current.as_ref().and_then(|a| serving(&a.task));
        if current.is_none() && !s.queue.iter().any(|t| serving(t).is_some()) {
            continue;
        }
        let home = w.households[s.household as usize].plot;
        let stale = |g: usize| home.is_none_or(|p| !guest_there(w, g, p));
        let drop_current = current.is_some_and(stale);
        let gone: Vec<usize> = s
            .queue
            .iter()
            .filter_map(serving)
            .filter(|&g| stale(g))
            .collect();
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        let s = &mut sims[h];
        if drop_current {
            end_activity(s, content, objects);
        }
        s.queue
            .retain(|t| serving(t).is_none_or(|g| !gone.contains(&g)));
    }
}

/// A free host asks a guest who has nothing and isn't eating whether they'd like something: a
/// while after they came in, and again a good while after (`visits.offer`), with something at
/// home to bring.
fn offer(w: &mut World) {
    let Some(rules) = w.content.visits.offer.clone() else {
        return;
    };
    let tick = w.tick;
    let ticks = |m: f32| crate::social::minutes_to_ticks(m);
    for g in 0..w.sims.len() {
        let Some(visit) = w.sims[g].visiting else {
            continue;
        };
        if tick < visit.offer_at {
            continue;
        }
        let plot = visit.plot;
        let guest = &w.sims[g];
        // In (not on the way, nor at the door), at a home.
        let inside = guest_there(w, g, plot)
            && !w.plots[plot as usize].public
            && !guest
                .current
                .as_ref()
                .is_some_and(|a| matches!(a.task.kind, TaskKind::Visit { .. }));
        if !inside {
            continue;
        }
        if visit.offer_at == 0 {
            if let Some(v) = w.sims[g].visiting.as_mut() {
                v.offer_at = tick + ticks(rules.after_minutes).max(1);
            }
            continue;
        }
        let eating = |t: &Task| matches!(t.kind, TaskKind::Eat { .. });
        let content = &w.content;
        if visit.until < tick + ticks(LAST_MINUTES)
            || !free_will(w, guest)
            || guest.held.is_some()
            || !guest.available_for_social(content)
            || guest.current.as_ref().is_some_and(|a| eating(&a.task))
            || guest.queue.iter().any(eating)
        {
            continue;
        }
        let social = &content.socials[rules.social];
        let retry = |w: &mut World| {
            if let Some(v) = w.sims[g].visiting.as_mut() {
                v.offer_at = tick + ticks(RETRY_MINUTES).max(1);
            }
        };
        let Some(household) = w.households.iter().position(|h| h.plot == Some(plot)) else {
            retry(w);
            continue;
        };
        // The nearest of the household free to ask, with something to bring.
        let host = (0..w.sims.len())
            .filter(|&h| {
                let s = &w.sims[h];
                h != g
                    && s.household as usize == household
                    && free_to_offer(w, s, plot)
                    && crate::conversation::social_allowed(
                        content,
                        &w.relationships,
                        s,
                        guest,
                        social,
                    )
            })
            .min_by(|&a, &b| {
                let d = |h: usize| {
                    let p = w.sims[h].pos;
                    (p[0] - guest.pos[0]).hypot(p[1] - guest.pos[1])
                };
                d(a).total_cmp(&d(b))
            })
            .filter(|&h| fare(w, h, g, plot).is_some());
        let Some(h) = host else {
            retry(w);
            continue;
        };
        if let Some(v) = w.sims[g].visiting.as_mut() {
            v.offer_at = tick + ticks(rules.again_minutes);
        }
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        let s = &mut sims[h];
        if s.current.is_some() {
            end_activity(s, content, objects);
        }
        s.queue.push_front(Task {
            kind: TaskKind::Social {
                target: g as u32,
                social: rules.social,
            },
            directed: false,
            routine: false,
        });
    }
}

// ---- The host's step ----------------------------------------------------------------------

/// The guest of a serve, if they're a guest on the plot `sim` lives on and there now (the
/// tick's briefs).
fn guest_here(sim: &Sim, ctx: &Ctx, guest: u32) -> bool {
    let home = ctx.households[sim.household as usize].plot;
    ctx.briefs.get(guest as usize).is_some_and(|b| {
        b.here && !b.away && home.is_some() && b.guest_at == home && b.plot == home
    })
}

/// Whether `sim` is near enough to hand `guest` something, in sight of them (the tick's briefs).
fn near(sim: &Sim, ctx: &Ctx, guest: u32) -> bool {
    let at = ctx.briefs[guest as usize].pos;
    (at[0] - sim.pos[0]).hypot(at[1] - sim.pos[1]) <= REACH
        && crate::conversation::in_sight(&ctx.nav, sim.pos, at)
}

/// Sets off on a serve (or its next stage): over to what it's taken from, or with it in hand
/// to the guest.
pub(crate) fn start(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let content = ctx.content;
    let TaskKind::Serve {
        guest,
        from,
        interaction,
        stage,
    } = task.kind
    else {
        return;
    };
    sim.current = None;
    if !guest_here(sim, ctx, guest) {
        return;
    }
    if stage == 0 {
        let Some(o) = objects.get(from as usize) else {
            return;
        };
        let def = &content.objects[o.def];
        let Some(inter) = def
            .interactions
            .get(interaction)
            .filter(|i| i.eats.is_some())
        else {
            return;
        };
        if sim.held.is_some()
            || o.broken()
            || !o.can_use_for(sim.id, def, interaction)
            || !o.allows(inter, sim.id)
        {
            return;
        }
        let tags = inter.tags;
        let (fx, fz) = ctx
            .index
            .links
            .approach_of(from)
            .unwrap_or_else(|| o.front_tile(content));
        let Some(waypoints) = walk_route(ctx, sim, (fx, fz), [fx as f32 + 0.5, fz as f32 + 0.5])
        else {
            return;
        };
        let o = &mut objects[from as usize];
        if !o.reserve_for(sim.id, &content.objects[o.def], interaction, sim.pos, 0) {
            return;
        }
        sim.pose = Pose::Stand;
        sim.current = Some(Activity {
            task,
            phase: Phase::Routing { waypoints, next: 0 },
            tags,
        });
        return;
    }
    if sim.held.is_none() {
        return;
    }
    let at = ctx.briefs[guest as usize].pos;
    let Some(goal) = crate::conversation::approach_tile(&ctx.nav, at, sim.pos) else {
        return;
    };
    let Some(waypoints) = walk_route(ctx, sim, goal, [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5])
    else {
        return;
    };
    sim.pose = Pose::Stand;
    sim.current = Some(Activity {
        task,
        phase: Phase::Routing { waypoints, next: 0 },
        tags: 0,
    });
}

/// Got there: at what it's taken from, a moment taking it out; by the guest, turned to them
/// to hand it over (moved on meanwhile: after them again, a few times at most).
pub(crate) fn arrive(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let content = ctx.content;
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return;
    };
    let TaskKind::Serve {
        guest, from, stage, ..
    } = task.kind
    else {
        return;
    };
    let to = if stage == 0 {
        objects[from as usize].centre(content)
    } else if near(sim, ctx, guest) {
        ctx.briefs[guest as usize].pos
    } else {
        return after(sim, task, ctx, objects);
    };
    sim.yaw = (to[0] - sim.pos[0]).atan2(to[1] - sim.pos[1]);
    sim.pose = Pose::Stand;
    if let Some(act) = sim.current.as_mut() {
        act.phase = Phase::Using { elapsed: 0.0 };
    }
}

/// After a guest who moved on, if there are goes left (else it's put down).
fn after(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let TaskKind::Serve { stage, .. } = task.kind else {
        return;
    };
    sim.current = None;
    if stage < TRIES {
        start(sim, next_stage(task), ctx, objects);
    }
}

/// A serve at its next stage.
fn next_stage(task: Task) -> Task {
    let mut next = task;
    if let TaskKind::Serve { stage, .. } = &mut next.kind {
        *stage += 1;
    }
    next
}

/// A moment taking it out (the `eat` use's minutes), or handing it over: whether it's done.
/// Handing over waits for a guest busy with something else (a while at most), and is over at
/// once if they've moved off.
pub(crate) fn using(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], elapsed: f32) -> bool {
    let Some(TaskKind::Serve {
        guest,
        from,
        interaction,
        stage,
    }) = sim.current.as_ref().map(|a| a.task.kind)
    else {
        return true;
    };
    if stage == 0 {
        let minutes = objects
            .get(from as usize)
            .and_then(|o| ctx.content.objects[o.def].interactions.get(interaction))
            .map_or(1.0, |i| i.minutes);
        return elapsed >= minutes.max(0.5);
    }
    let b = &ctx.briefs[guest as usize];
    !guest_here(sim, ctx, guest)
        || !near(sim, ctx, guest)
        || (elapsed >= HAND_MINUTES && b.available && !b.eating)
        || elapsed >= WAIT_MINUTES
}

/// The moment is over: taken out, off to the guest with it in hand; handed over, it's theirs
/// (`hand_over`, after the step); the guest moved off, after them again.
pub(crate) fn step_done(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let content = ctx.content;
    let Some(act) = sim.current.take() else {
        return;
    };
    let task = act.task;
    let TaskKind::Serve {
        guest,
        from,
        interaction,
        stage,
    } = task.kind
    else {
        return;
    };
    if stage == 0 {
        objects[from as usize].release(sim.id);
        let o = &objects[from as usize];
        let Some(inter) = content.objects[o.def].interactions.get(interaction) else {
            return;
        };
        let Some(recipe) = inter.eats else {
            return;
        };
        let quality = crate::cooking::made_quality(content, o, inter, &sim.skills);
        sim.held = Some(crate::cooking::plate(content, recipe, quality, ctx.tick));
        return start(sim, next_stage(task), ctx, objects);
    }
    if !guest_here(sim, ctx, guest) {
        return;
    }
    if !near(sim, ctx, guest) {
        return after(sim, task, ctx, objects);
    }
    let b = &ctx.briefs[guest as usize];
    if b.available && !b.eating {
        sim.hand_to = Some(guest);
    }
}
