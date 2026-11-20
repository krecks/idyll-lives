//! Out for leisure: booking, timing and bringing riders back.

use super::*;

/// The most taxis on visits and outings at once, planned or under way, there or back, the town's
/// taxi firm's (`Taxis::fleet_out`): with that many out, someone setting off goes another way, or
/// somewhere nearer (one home from a place is never refused). On the
/// traffic's clock (`crate::DRIVE_SCALE`) a taxi beats any walk out of reach, and with no such
/// limit a big world's evening had some 20 out at once and 150 runs a day where 5 to 20 ran
/// before, jammed solid round a junction in view (seed base 16: Sand Quay, Eaglescar).
pub(super) const LEISURE_FLEET: usize = 6;

/// About how many minutes a taxi would take `sim` from `from` to `plot` on a visit or an outing
/// (`world::leisure_way`, after the household's car and a bike of theirs, before walking), a way
/// there or back: the ride from their kerb to the place's, the steps to and from the kerbs, and
/// half its coming in from the way out nearest them, waited for at home once for the two ways
/// (an outing's time away, `world::outing_fits`, counts each way and the wait just once). Some,
/// and they'd go by it: an open-world town, the place out of a short walk (`autonomy.reach`), and
/// the taxi, its coming in too, quicker than walking (`LEISURE_BEATS`).
pub(crate) fn leisure_eta(
    travel: &crate::travel::Travel,
    exits: &[[f32; 2]],
    content: &crate::content::Content,
    sim: &crate::world::Sim,
    from: [f32; 2],
    door: [f32; 2],
) -> Option<f32> {
    leisure_eta_via(
        travel,
        exits,
        content,
        sim,
        from,
        (None, door),
        &mut [None, None],
    )
}

/// `leisure_eta`, with where the trips from `from` get on the roads on foot and by car (`on`,
/// by `Mode`: `Travel::entry`, found as first asked for) kept by the caller for the next from
/// there (a choice of what to do times the way to every place: `world::pick_autonomous`), and
/// the door's (`Travel::door_entry`) as noted for its plot, if given.
pub(crate) fn leisure_eta_via(
    travel: &crate::travel::Travel,
    exits: &[[f32; 2]],
    content: &crate::content::Content,
    sim: &crate::world::Sim,
    from: [f32; 2],
    (plot, door): (Option<u32>, [f32; 2]),
    on: &mut [Option<Option<crate::travel::RoadPoint>>; 2],
) -> Option<f32> {
    use crate::travel::Mode;
    if exits.is_empty() || crate::travel::dist(from, door) <= content.autonomy.reach {
        return None;
    }
    let mut on_at = |mode: Mode| *on[mode as usize].get_or_insert_with(|| travel.entry(from, mode));
    let off_at = |mode: Mode| match plot {
        Some(plot) => travel.door_entry(plot, door, mode),
        None => travel.entry(door, mode),
    };
    let walk = sim.walk_per_minute(content);
    let on_foot = travel.eta_via(
        from,
        door,
        Mode::Foot,
        walk,
        || on_at(Mode::Foot),
        || off_at(Mode::Foot),
    );
    let exit = exits
        .iter()
        .copied()
        .min_by(|a, b| crate::travel::dist(*a, from).total_cmp(&crate::travel::dist(*b, from)))?;
    let come = travel.eta_via(
        exit,
        from,
        Mode::Car,
        walk,
        || travel.entry(exit, Mode::Car),
        || on_at(Mode::Car),
    );
    let ride = travel.eta_via(
        from,
        door,
        Mode::Car,
        walk,
        || on_at(Mode::Car),
        || off_at(Mode::Car),
    );
    // (`kerb_walks`, by those entries.)
    let to_road = |p: [f32; 2], entry: Option<crate::travel::RoadPoint>| {
        entry.map_or(0.0, |r| crate::travel::dist(p, travel.graph.point(r)))
    };
    let kerbs = (to_road(from, on_at(Mode::Car)) + to_road(door, off_at(Mode::Car))) * KERB_WINDING
        / walk.max(0.1)
        + LEISURE_KERBS;
    let by_taxi = ride * crate::travel::DRIVE_MINUTES + kerbs;
    let wait = come * crate::travel::DRIVE_MINUTES;
    (by_taxi.is_finite() && wait.is_finite() && by_taxi + wait < on_foot * LEISURE_BEATS)
        .then_some(by_taxi + wait / 2.0)
}

/// Resident `i` sets off from home to `plot` (a visit or an outing) by the taxi (`world`'s way
/// there: `leisure_way`): booked on one coming for those off to the same place about now (a
/// housemate, a visit party), not yet in town, with a seat; else one of their own, coming at once.
/// They wait at home till it's near (`leisure_leaving`), go out to their kerb and get in as for
/// work; it puts them down at the place's kerb and they walk on in. False: no taxi can take them
/// (they walk).
pub(crate) fn book_leisure(w: &mut World, i: usize, plot: u32, directed: bool) -> bool {
    let tick = w.tick;
    let Some(home) = w
        .households
        .get(w.sims[i].household as usize)
        .and_then(|h| h.plot)
    else {
        return false;
    };
    if w.sims[i].taxi.is_some() || plot == home {
        return false;
    }
    seat_leisure(w, i, home, plot, false, directed, tick + 1)
}

/// Resident `i` (living at `home`) on a leisure run to `place` (`back`: home from it) for `due`
/// (out: as soon as it can come; back: when their visit's done): with others off there (or home
/// from there) about then, not yet in town, with a seat, their housemates' halt theirs too (else
/// a new one); else a run of their own. False: no taxi can take them.
fn seat_leisure(
    w: &mut World,
    i: usize,
    home: u32,
    place: u32,
    back: bool,
    directed: bool,
    due: u64,
) -> bool {
    let tick = w.tick;
    let seats = w.content.taxi.seats;
    let joins: Vec<usize> = (0..w.taxis.runs.len())
        .filter(|&k| {
            let r = &w.taxis.runs[k];
            r.place == Some(place)
                && r.home == back
                && r.car.is_none()
                && r.enter > tick
                && r.stops.len() < seats
                && if back {
                    r.stops
                        .iter()
                        .all(|st| st.due.abs_diff(due) <= LEISURE_WINDOW)
                } else {
                    r.enter + LEISURE_WINDOW > tick
                }
        })
        .collect();
    // (Each it could ride along on, else one of its own: the first that can be timed. Not one
    // of its own out with the fleet out, `LEISURE_FLEET`.)
    let own = back || !w.taxis.fleet_out();
    for join in joins.into_iter().map(Some).chain(own.then_some(None)) {
        let mut run = match join {
            Some(k) => w.taxis.runs[k].clone(),
            None => Run {
                id: w.taxis.next_id,
                home: back,
                place: Some(place),
                directed,
                group: (0, 0),
                enter: due,
                due_out: 0,
                planned_out: 0,
                depart: 0,
                stops: Vec::new(),
                next: 0,
                progress: 0.0,
                detour: None,
                since: 0,
                car: None,
                at: Vec::new(),
                kerbs: Vec::new(),
                laid: None,
            },
        };
        // (Their halt: their housemates', else a new one after the others; back, the place's is
        // the first.)
        let halt = match run.stops.iter().find(|st| st.plot == home) {
            Some(st) => st.halt,
            None => run
                .stops
                .iter()
                .map(|st| st.halt + 1)
                .max()
                .unwrap_or(u8::from(back)),
        };
        run.stops.push(Stop {
            sim: i as u32,
            plot: home,
            due,
            leave: 0,
            state: StopState::Booked,
            halt,
        });
        run.kerbs = Vec::new();
        if !time_leisure(w, &mut run) {
            continue;
        }
        let id = run.id;
        match join {
            Some(k) => w.taxis.runs[k] = run,
            None => {
                w.taxis.next_id = w.taxis.next_id.wrapping_add(1);
                w.taxis.runs.push(run);
            }
        }
        rebook(w, id);
        return true;
    }
    false
}

/// Each rider of run `id` booked on it at their stop (`Sim::taxi`).
fn rebook(w: &mut World, id: u32) {
    let Some(r) = w.taxis.runs.iter().find(|r| r.id == id) else {
        return;
    };
    for (s, st) in r.stops.iter().enumerate() {
        if st.state != StopState::Done
            && let Some(sim) = w.sims.get_mut(st.sim as usize)
        {
            sim.taxi = Some(Booking {
                run: id,
                stop: s as u8,
            });
        }
    }
}

/// Times leisure run `run` (out to its place, or back from it), coming into town as soon as it can
/// (or, back from the place, so as to be there at its riders' `due`): its kerbs, route, when it's at
/// each halt, and when each rider leaves for their kerb (the walk there and `KERB_EARLY`). False:
/// no way.
fn time_leisure(w: &World, run: &mut Run) -> bool {
    let tick = w.tick;
    let halts = halts_of(run);
    // (Clear of other taxis standing about when it's at each, `stands_and_way`: back from the
    // place, there when the first of them is done; else as it's timed to come.)
    let done = run.stops.iter().filter(|_| run.home).map(|st| st.due).min();
    let enter_at = |_: &crate::cars::Route, at: &[f32]| {
        match done {
            Some(due) => due.saturating_sub(at.first().map_or(0, |t| t.ceil() as u64)),
            None => run.enter,
        }
        .max(tick + 1)
    };
    let Some((kerbs, laid, later)) = stands_and_way(w, &halts, Some(run.id), None, None, enter_at)
    else {
        return false;
    };
    let (route, at) = (&laid.route, &laid.at);
    let offset = |h: usize| at.get(h).map_or(0.0, |t| t.ceil()) as u64 + h as u64 * DWELL;
    // (Each one's way from where they are out to their kerb, and a moment there.)
    let halt = |st: &Stop| if run.home { 0 } else { st.halt as usize };
    let lead: Vec<u64> = run
        .stops
        .iter()
        .map(|st| {
            let sim = &w.sims[st.sim as usize];
            // (From over the ground: upstairs, a walk timed across the map from the storey's
            // place on it had them out at their kerb two hours early, Swan Reach.)
            let to_kerb = ((w.travel.eta(
                w.stairs.ground_pos(sim.pos),
                wait_spot(&w.travel, &kerbs[halt(st)]),
                crate::travel::Mode::Foot,
                sim.walk_per_minute(&w.content),
            ) + crate::life::OUT_OF_THE_HOUSE)
                * crate::TICKS_PER_MINUTE as f32) as u64;
            to_kerb.max(TO_KERB_LEAST) + KERB_EARLY
        })
        .collect();
    // (Back from the place: there when the first of them is done; else at once. Either way, not
    // before each of them can be at their kerb for it.)
    let want = run.stops.iter().filter(|_| run.home).map(|st| st.due).min();
    let mut enter = match want {
        Some(due) => due.saturating_sub(offset(0)),
        None => run.enter,
    }
    .max(tick + 1);
    for (st, &l) in run.stops.iter().zip(&lead) {
        enter = enter.max((tick + l).saturating_sub(offset(halt(st))));
    }
    // (Later, a stand of its another's till then: `stands_and_way`.)
    if later > 0 {
        enter = enter.max(enter_at(route, at) + later);
    }
    run.enter = enter;
    for (k, st) in run.stops.iter_mut().enumerate() {
        let h = if run.home { 0 } else { st.halt as usize };
        st.due = enter + offset(h);
        st.leave = st.due.saturating_sub(lead[k]);
    }
    run.due_out = run.enter + route.ticks().ceil() as u64 + halts.len() as u64 * DWELL;
    run.at = laid.at.clone();
    run.kerbs = kerbs;
    run.laid = Some(Box::new(laid));
    true
}

/// Whether `sim` is out by leisure taxi (`book_leisure`): booked on one out or back, riding it,
/// or at the place it took them to (`Taxis::out_at`). The taxi brings them home by the end of their
/// stay, and they set off for work or school from there (`life::firm_leave`).
pub(crate) fn out_by_taxi(taxis: &Taxis, sim: &crate::world::Sim) -> bool {
    sim.taxi.is_some_and(|b| {
        taxis
            .runs
            .iter()
            .any(|r| r.id == b.run && r.place.is_some())
    }) || sim
        .visiting
        .is_some_and(|v| taxis.out_at.contains(&(sim.id, v.plot)))
}

/// Whether `sim` waits at home for a leisure taxi out booked for them (`book_leisure`): they don't
/// go off on another visit or an outing meanwhile (`world`'s choices); off on one, the taxi's
/// time came while they were there, and it took them away twenty minutes in (Ottertarn).
pub(crate) fn waiting_out(taxis: &Taxis, sim: &crate::world::Sim) -> bool {
    sim.taxi.is_some_and(|b| {
        taxis.runs.iter().any(|r| {
            r.id == b.run
                && r.place.is_some()
                && !r.home
                && r.stops
                    .get(b.stop as usize)
                    .is_some_and(|st| st.state == StopState::Booked)
        })
    })
}

/// Those booked on a leisure run whose time to go out to their kerb has come: off they go, there
/// (their visit or outing, `world`'s way there: to the kerb, `kerb_for`) or home from the place,
/// their stay over (the taxi back comes when the first of those sharing it is done, `book_back`).
pub(super) fn leisure_leaving(w: &mut World) {
    let tick = w.tick;
    let mut go: Vec<(usize, u32, bool, bool)> = Vec::new();
    for r in &w.taxis.runs {
        let Some(place) = r.place else { continue };
        for st in r.stops.iter().filter(|st| st.state == StopState::Booked) {
            let i = st.sim as usize;
            let leave = leave_for(
                &w.taxis,
                &w.cars,
                tick,
                Booking {
                    run: r.id,
                    stop: r.stop_of(st.sim).unwrap_or(0) as u8,
                },
            );
            let Some(sim) = w.sims.get(i) else { continue };
            let on_way = |t: &TaskKind| match t {
                TaskKind::GoHome => r.home,
                TaskKind::Visit { plot } => !r.home && *plot == place,
                _ => false,
            };
            let off = sim.current().is_some_and(|a| on_way(&a.task.kind))
                || sim.queue.front().is_some_and(|t| on_way(&t.kind));
            let ready = if r.home {
                sim.visiting.is_some_and(|v| v.plot == place)
            } else {
                sim.here() && sim.away_until.is_none()
            };
            if leave.is_some_and(|l| tick >= l) && !off && ready {
                go.push((i, place, r.directed, r.home));
            }
        }
    }
    for (i, plot, directed, home) in go {
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        let sim = &mut sims[i];
        if sim.current.is_some() {
            crate::world::end_activity(sim, content, objects);
            // (Talking from a seat: that ended the talk and took them back to what they sat
            // down to do; that ends too. Dropped, the seat stayed theirs.)
            if sim.current.is_some() {
                crate::world::end_activity(sim, content, objects);
            }
        }
        sim.current = None;
        let kind = if home {
            if let Some(v) = sim.visiting.as_mut() {
                v.until = v.until.min(tick);
            }
            TaskKind::GoHome
        } else {
            TaskKind::Visit { plot }
        };
        sim.queue.push_front(Task {
            kind,
            directed,
            routine: false,
        });
    }
}

/// Those booked on a taxi back from a place who've given their stay up early and are off home
/// now (`life::end_visit_if_due`: a need the place can't see to, the hosts gone, bored): the taxi
/// was booked for when their stay was to end, and they'd stand at the kerb till then (three hours
/// and more, a visit to a house with nobody in). Off that run (still to come into town) and
/// booked on one for now, shared as ever (`book_back`), if it's sooner; else as they were. Once
/// for each run they're on (`Taxis::early`).
pub(super) fn bring_backs_forward(w: &mut World) {
    let tick = w.tick;
    let mut early: Vec<(usize, u32)> = Vec::new();
    for r in &w.taxis.runs {
        if !r.home || r.place.is_none() || r.car.is_some() {
            continue;
        }
        for (s, st) in r.stops.iter().enumerate() {
            let i = st.sim as usize;
            let b = Booking {
                run: r.id,
                stop: s as u8,
            };
            let off = w.sims.get(i).is_some_and(|x| {
                x.taxi == Some(b)
                    && x.current()
                        .is_some_and(|a| matches!(a.task.kind, TaskKind::GoHome))
            });
            // (Out to the kerb more than the few minutes early a rider is anyway.)
            if off
                && st.state == StopState::Booked
                && st.leave > tick + KERB_EARLY
                && !w.taxis.early.contains(&(st.sim, r.id))
            {
                early.push((i, r.id));
            }
        }
    }
    for (i, id) in early {
        w.taxis.early.push((i as u32, id));
        let Some(k) = w.taxis.runs.iter().position(|r| r.id == id) else {
            continue;
        };
        let mut run = w.taxis.runs[k].clone();
        let (Some(place), Some(s)) = (run.place, run.stop_of(i as u32)) else {
            continue;
        };
        let was = run.stops.remove(s);
        // (Theirs alone, their halt goes, those after it one on.)
        if !run.stops.iter().any(|st| st.halt == was.halt) {
            for st in &mut run.stops {
                if st.halt > was.halt {
                    st.halt -= 1;
                }
            }
        }
        // (The others' run timed again without them; it can't be: as they were.)
        run.kerbs = Vec::new();
        if !run.stops.is_empty() && !time_leisure(w, &mut run) {
            continue;
        }
        let (runs, next_id) = (w.taxis.runs.clone(), w.taxis.next_id);
        if run.stops.is_empty() {
            w.taxis.runs.remove(k);
        } else {
            w.taxis.runs[k] = run;
            rebook(w, id);
        }
        w.sims[i].taxi = None;
        let due = |w: &World| {
            let b = w.sims[i].taxi?;
            let r = w.taxis.runs.iter().find(|r| r.id == b.run)?;
            Some(r.stops.get(b.stop as usize)?.due)
        };
        if !(book_back(w, i, place, tick) && due(w).is_some_and(|d| d < was.due)) {
            w.taxis.runs = runs;
            w.taxis.next_id = next_id;
            w.sims[i].taxi = None;
            rebook(w, id);
            continue;
        }
        // (On their way to the kerb: to the new run's, as it stands for them.)
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        let sim = &mut sims[i];
        if let Some(task) = sim.current().map(|a| a.task) {
            crate::world::end_activity(sim, content, objects);
            sim.current = None;
            sim.queue.push_front(task);
        }
    }
}

/// Those put down at a place by a leisure taxi (`alight_at`), there now on their visit or outing:
/// a taxi back home booked for when it ends (`crate::world::Visit::until`), shared with others
/// there going home about then.
pub(super) fn book_backs(w: &mut World) {
    let mut k = 0;
    while k < w.taxis.out_at.len() {
        let (i, place) = w.taxis.out_at[k];
        let Some(sim) = w.sims.get(i as usize) else {
            w.taxis.out_at.remove(k);
            continue;
        };
        let visit = sim.visiting.filter(|v| v.plot == place);
        // (Not there yet: getting out, walking in from the kerb, knocking. There, their visit's
        // end is known.)
        let coming = |t: &TaskKind| matches!(t, TaskKind::Visit { plot } if *plot == place);
        if visit.is_some()
            && (sim.current().is_some_and(|a| coming(&a.task.kind))
                || sim.queue.front().is_some_and(|t| coming(&t.kind)))
        {
            k += 1;
            continue;
        }
        let Some(v) = visit.filter(|_| sim.taxi.is_none()) else {
            w.taxis.out_at.remove(k);
            continue;
        };
        // (No stand free for it at the place yet, the taxi that brought them still there, say:
        // again next tick.)
        let ok = book_back(w, i as usize, place, v.until.max(w.tick) + BACK_SLACK);
        if ok {
            w.taxis.out_at.remove(k);
            // (Already off home on foot: out to the kerb for it instead.)
            let World {
                sims,
                content,
                objects,
                ..
            } = &mut *w;
            let sim = &mut sims[i as usize];
            if sim
                .current()
                .is_some_and(|a| matches!(a.task.kind, TaskKind::GoHome))
            {
                crate::world::end_activity(sim, content, objects);
                sim.current = None;
                sim.queue.push_front(Task {
                    kind: TaskKind::GoHome,
                    directed: false,
                    routine: false,
                });
            }
        } else {
            k += 1;
        }
    }
}

/// Resident `i`, at `place` on a visit or an outing they came to by taxi, books one home for
/// `due` (when the visit ends): with others there going home about then, else one of its own.
/// False: none can come for them now.
pub(super) fn book_back(w: &mut World, i: usize, place: u32, due: u64) -> bool {
    let Some(home) = w
        .households
        .get(w.sims[i].household as usize)
        .and_then(|h| h.plot)
    else {
        return false;
    };
    seat_leisure(w, i, home, place, true, false, due)
}

/// Minutes `sim`, on a visit or an outing at `place` they came to by taxi (or booked on one back
/// from it), takes from `pos` to `door` at home by the taxi back: its coming once they're done
/// (`BACK_SLACK`), the ride, the steps to and from the kerbs (`world::outing_leave_by`). None:
/// they didn't come by taxi.
pub(crate) fn back_minutes(
    ctx: &crate::world::Ctx,
    sim: &crate::world::Sim,
    place: u32,
    pos: [f32; 2],
    (home, door): (u32, [f32; 2]),
) -> Option<f32> {
    let came = ctx.taxis.out_at.contains(&(sim.id, place))
        || sim.taxi.is_some_and(|b| {
            ctx.taxis
                .runs
                .iter()
                .any(|r| r.id == b.run && r.home && r.place == Some(place))
        });
    came.then(|| {
        // (Where the ride gets on and off the roads, for it and the kerbs alike; home's door as
        // noted, `Travel::door_entry`.)
        let car = crate::travel::Mode::Car;
        let on = ctx.travel.entry(pos, car);
        let off = ctx.travel.door_entry(home, door, car);
        let walk = sim.walk_per_minute(ctx.content);
        BACK_SLACK as f32 * crate::MINUTES_PER_TICK
            + ctx.travel.eta_via(pos, door, car, walk, || on, || off) * crate::travel::DRIVE_MINUTES
            + kerb_walks(ctx.travel, (pos, on), (door, off), walk)
    })
}

/// Minutes on foot by taxi from `from` to `to` besides the ride (`leisure_eta`, `back_minutes`):
/// out to the road nearest the one and in from the road nearest the other (where each gets on
/// the roads by car, `Travel::entry`), at `walk` m a minute, and the taxi's stands and the wait
/// (`LEISURE_KERBS`).
fn kerb_walks(
    travel: &crate::travel::Travel,
    (from, on): ([f32; 2], Option<crate::travel::RoadPoint>),
    (to, off): ([f32; 2], Option<crate::travel::RoadPoint>),
    walk: f32,
) -> f32 {
    let to_road = |p: [f32; 2], entry: Option<crate::travel::RoadPoint>| {
        entry.map_or(0.0, |r| crate::travel::dist(p, travel.graph.point(r)))
    };
    (to_road(from, on) + to_road(to, off)) * KERB_WINDING / walk.max(0.1) + LEISURE_KERBS
}

/// Resident `i` gets out of taxi `c` (seat `seat`) at `place`'s kerb and walks on in, a visit or
/// an outing (`directed`: the player's), a taxi back booked once they're there (`book_backs`).
pub(super) fn alight_at(
    w: &mut World,
    i: usize,
    c: usize,
    seat: usize,
    place: u32,
    directed: bool,
) {
    let door = w.cars[c].taxi_door(&w.nav(), seat).0;
    let car = &w.cars[c];
    let (at, yaw) = (car.seat_at(seat, door), car.yaw);
    let sim = &mut w.sims[i];
    sim.pos = at;
    sim.yaw = yaw;
    sim.pose = crate::content::Pose::Sit;
    sim.bound_for_car = Some(c as u32);
    sim.car_seat = seat as u8;
    sim.car_door = door as u8;
    sim.current = Some(crate::world::Activity {
        task: Task {
            kind: TaskKind::Visit { plot: place },
            directed,
            routine: false,
        },
        phase: Phase::Alighting { ticks: 0 },
        tags: Default::default(),
    });
    w.taxis.out_at.retain(|o| o.0 != i as u32);
    w.taxis.out_at.push((i as u32, place));
}

/// Riders in a leisure taxi sit in their seats as it drives (as in a household's car on a trip).
pub(super) fn seat_riders(w: &mut World) {
    for r in w.taxis.runs.iter().filter(|r| r.place.is_some()) {
        let Some(c) = r.car else { continue };
        let car = &w.cars[c as usize];
        let Some((at, yaw, ..)) = car.shown else {
            continue;
        };
        let posed = car.posed(at, yaw);
        for (s, st) in r.stops.iter().enumerate() {
            if st.state != StopState::Aboard {
                continue;
            }
            let seat = 1 + s % 4;
            if let Some(sim) = w.sims.get_mut(st.sim as usize)
                && sim
                    .current()
                    .is_some_and(|a| matches!(a.phase, Phase::Driving))
            {
                sim.pos =
                    posed.seat_at(seat, usize::from(sim.car_door).min(crate::cars::DOORS - 1));
                sim.yaw = yaw;
                sim.pose = crate::content::Pose::Sit;
            }
        }
    }
}

/// Whether car `c` is a work taxi with a halt still ahead on its run (`cars::unjam`: one with none
/// left may turn round and go out of town another way).
pub(crate) fn halts_ahead(w: &World, c: usize) -> bool {
    w.taxis
        .runs
        .iter()
        .find(|r| r.car == Some(c as u32))
        .is_some_and(|r| (r.next as usize) < r.at.len())
}

/// Whether car `c`, a work taxi, has a halt still to come between `from` and `to` ticks along its
/// way (`cars::pass_stands`: it doesn't go out round a standing taxi there).
pub(crate) fn halts_within(w: &World, c: usize, from: f32, to: f32) -> bool {
    w.taxis
        .runs
        .iter()
        .find(|r| r.car == Some(c as u32))
        .is_some_and(|r| {
            r.at[(r.next as usize).min(r.at.len())..]
                .iter()
                .any(|&h| (from..=to).contains(&h))
        })
}

/// Car `c`, a work taxi, backed up out of the way on its run (`cars::unjam`): its halts still
/// ahead are at `at(old)` along its way now.
pub(crate) fn moved_halts(w: &mut World, c: usize, at: &dyn Fn(f32) -> f32) {
    let progress = w.cars[c].progress;
    if let Some(run) = w.taxis.runs.iter_mut().find(|r| r.car == Some(c as u32)) {
        for h in run.next as usize..run.at.len() {
            run.at[h] = at(run.at[h]);
        }
        run.progress = progress;
    }
}
