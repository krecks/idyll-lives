//! Where a run goes: kerbs, stands, ways in and ways out.

use super::*;

/// When rider `i` leaves home for their kerb, if they're booked for the next run to work
/// (`life`'s leaving): in time for the taxi as planned, or as late as it's running (held up on its
/// way, or still at the edge of town when it was due in, `run_on`).
pub(crate) fn leave_at(w: &World, i: usize) -> Option<u64> {
    let sim = &w.sims[i];
    let leave = booked_leave(&w.taxis, &w.cars, w.tick, sim.taxi?)?;
    Some(leave.saturating_sub(indoor_lead(
        &w.content,
        w.nav().stairs,
        &w.households,
        &w.plots,
        sim,
    )))
}

/// How much sooner than planned `sim` leaves for their kerb from where they are now: the walk
/// to their front door from further in (upstairs, say, or out in the garden) than the minutes
/// out of the house the run was timed with (`life::OUT_OF_THE_HOUSE`; a rider coming down from
/// the far end of the landing missed the taxi, and the late run of their own came hours later).
pub(crate) fn indoor_lead(
    content: &crate::content::Content,
    stairs: &crate::path::StairMap,
    households: &[crate::world::Household],
    plots: &[crate::world::Plot],
    sim: &crate::world::Sim,
) -> u64 {
    let Some(home) = households
        .get(sim.household as usize)
        .and_then(|h| h.plot)
        .and_then(|p| plots.get(p as usize))
    else {
        return 0;
    };
    // (Only in and about the house: from elsewhere, a visit's own end brings them home in time;
    // timed as a walk from a visit across the map, a cyclist was home on their bike an hour
    // early, still on it at the kerb two hours on, Swan Reach.)
    let ground = stairs.ground_pos(sim.pos);
    if !home.contains(ground[0].floor() as i32, ground[1].floor() as i32) {
        return 0;
    }
    let door = home.arrival_point();
    let storeys = stairs.storey(sim.pos[1].floor() as i32).unsigned_abs() as f32;
    let walk = crate::travel::dist(ground, door) + storeys * STOREY_WALK;
    let ticks = walk / (sim.walk_per_minute(content) * crate::MINUTES_PER_TICK).max(0.01);
    (ticks as u64)
        .saturating_sub((crate::life::OUT_OF_THE_HOUSE * crate::TICKS_PER_MINUTE as f32) as u64)
}

/// `leave_at` for booking `b` at `tick`: the planned leave, later by as much as the taxi runs late
/// (`world`'s free time before work follows it too). None: not on a run to work (or school).
pub(crate) fn booked_leave(
    taxis: &Taxis,
    cars: &[crate::cars::Car],
    tick: u64,
    b: Booking,
) -> Option<u64> {
    taxis
        .runs
        .iter()
        .any(|r| r.id == b.run && r.place.is_none())
        .then(|| leave_for(taxis, cars, tick, b))
        .flatten()
}

/// `booked_leave` for any run that picks its riders up (to work or school, to a place or back
/// from it, `leisure_leaving`).
pub(super) fn leave_for(
    taxis: &Taxis,
    cars: &[crate::cars::Car],
    tick: u64,
    b: Booking,
) -> Option<u64> {
    let run = taxis.runs.iter().find(|r| r.id == b.run && r.picks_up())?;
    let st = run.stops.get(b.stop as usize)?;
    if st.state != StopState::Booked {
        return None;
    }
    let halt = run.pickup_halt(st);
    let late = match run.car.and_then(|c| cars.get(c as usize)) {
        Some(car) => {
            let at = run.at.get(halt as usize).copied().unwrap_or(0.0);
            let to_go = (at - car.progress).max(0.0).ceil() as u64;
            let halts = (halt as u64).saturating_sub(run.next as u64) * DWELL;
            (tick + to_go + halts).saturating_sub(st.due)
        }
        None => tick.saturating_sub(run.enter),
    };
    Some(st.leave + late)
}

/// Where `sim`, booked on a taxi to work, waits for it (`world`'s way to work), if it's still to
/// come for them.
pub(crate) fn kerb_for(
    ctx: &crate::world::Ctx,
    sim: &crate::world::Sim,
    task: &TaskKind,
) -> Option<[f32; 2]> {
    let b = sim.taxi?;
    let run = ctx
        .taxis
        .runs
        .iter()
        .find(|r| r.id == b.run && r.picks_up() && r.serves(task))?;
    let st = run.stops.get(b.stop as usize)?;
    let halt = run.pickup_halt(st);
    if st.state != StopState::Booked || run.next > halt {
        return None;
    }
    let stop = run.kerbs.get(halt as usize)?;
    Some(wait_spot(ctx.travel, stop))
}

/// The taxi `sim` is called to (`stand`), standing at their halt: the car, their seat, the door
/// they get in by and where they stand for it (`world`'s way to work: over to it, and in).
pub(crate) fn door_for(
    ctx: &crate::world::Ctx,
    sim: &crate::world::Sim,
    task: &TaskKind,
) -> Option<(u32, usize, usize, [f32; 2])> {
    let b = sim.taxi?;
    let run = ctx
        .taxis
        .runs
        .iter()
        .find(|r| r.id == b.run && r.picks_up() && r.serves(task))?;
    let st = run.stops.get(b.stop as usize)?;
    let c = run.car?;
    let car = ctx.cars.get(c as usize)?;
    if st.state != StopState::Boarding || !car.standing || run.next != run.pickup_halt(st) {
        return None;
    }
    let seat = usize::from(sim.car_seat);
    let (door, at) = car.taxi_door(&ctx.nav, seat);
    Some((c, seat, door, at))
}

/// Where a rider waits for the taxi standing at `stop`: at the back of the pavement on their
/// house's side, by the end of where it stands nearer their driveway (a step to its doors, or
/// across the road to them; `WAIT_CLEAR` past its end, clear of where it parks up on the pavement,
/// `cars::park_beside`: waiting by its middle, they were in its way as it pulled in), not in their
/// driveway's mouth (`WAIT_ASIDE` along from it at least).
pub(super) fn wait_spot(travel: &crate::travel::Travel, stop: &Kerb) -> [f32; 2] {
    let g = &travel.graph;
    let road = &g.roads[stop.road as usize];
    let mouth = g.point(stop.mouth);
    let off = stop.s - stop.mouth.s;
    let way = if off < 0.0 { -1.0 } else { 1.0 };
    let end = off - way * (TAXI_LEN / 2.0 + WAIT_CLEAR);
    let s = if end * way >= WAIT_ASIDE {
        stop.mouth.s + end
    } else {
        stop.mouth.s + if off < 0.0 { -WAIT_ASIDE } else { WAIT_ASIDE }
    };
    let (c, t) = (road.at(s), road.tangent(s));
    let right = crate::cars::right_of(t);
    let house = [mouth[0] - stop.axis[0], mouth[1] - stop.axis[1]];
    let k = if (house[0] - c[0]) * right[0] + (house[1] - c[1]) * right[1] >= 0.0 {
        1.0
    } else {
        -1.0
    };
    // (A road with no pavement, a country road: on its edge.)
    let back = road.width / 2.0 - 0.3;
    [c[0] + right[0] * back * k, c[1] + right[1] * back * k]
}

/// A slot for a taxi in `World::cars`: one free (a taxi done with its run), else a new one (none
/// past `cars::MAX_CARS`).
pub(super) fn slot(w: &mut World) -> Option<u32> {
    let busy: Vec<u32> = w.taxis.runs.iter().filter_map(|r| r.car).collect();
    if let Some(c) = (0..w.cars.len())
        .find(|&c| w.cars[c].taxi && !w.cars[c].off_run && !busy.contains(&(c as u32)))
    {
        return Some(c as u32);
    }
    if w.cars.len() >= crate::cars::MAX_CARS {
        return None;
    }
    let at = w.exits.first().copied().unwrap_or([0.0, 0.0]);
    w.cars.push(crate::cars::Car::taxi(at, TAXI_LEN, TAXI_WID));
    Some(w.cars.len() as u32 - 1)
}

/// The plots of run `k`'s halts, in order (housemates' one; a leisure run's place, `Run::place`,
/// last going there, first coming back).
pub(super) fn halts_of(run: &Run) -> Vec<u32> {
    let n = run.halts() as usize;
    let mut halts = vec![u32::MAX; n];
    for st in &run.stops {
        if let Some(h) = halts.get_mut(st.halt as usize) {
            *h = st.plot;
        }
    }
    if let Some(place) = run.place {
        let k = if run.home { 0 } else { n - 1 };
        halts[k] = place;
    }
    halts
}

/// When run `r`'s taxi stands at its halt `h` (ticks, from and to): about when it's there as it's
/// timed, `STAND_NEAR` either side; in town, from where it's got to (there now: now; gone on past
/// it: None, it's free). With no times for it yet, whenever.
fn stand_window(w: &World, r: &Run, h: usize) -> Option<(u64, u64)> {
    let Some(&at) = r.at.get(h) else {
        return Some((0, u64::MAX));
    };
    let t = if r.car.is_some() {
        let next = r.next as usize;
        if h < next {
            return None;
        }
        w.tick + (at - r.progress).max(0.0).ceil() as u64 + (h - next) as u64 * DWELL
    } else {
        r.enter.max(w.tick) + at.ceil() as u64 + h as u64 * DWELL
    };
    Some((t.saturating_sub(STAND_NEAR), t + STAND_NEAR))
}

/// Where a taxi stands for each of `halts` (their plots, in order; `kerb_at`) at about `when` each
/// (ticks): clear of every other run's taxi (but `skip`'s) standing then (`stand_window`; `when`
/// none: clear of none); and one stand for two halts as near each other as a taxi's length and a
/// gap.
fn kerbs_for(
    w: &World,
    halts: &[u32],
    skip: Option<u32>,
    when: Option<&[u64]>,
) -> Option<Vec<Kerb>> {
    let others: Vec<((u64, u64), (u32, f32))> = match when {
        Some(_) => w
            .taxis
            .runs
            .iter()
            .filter(|r| Some(r.id) != skip)
            .flat_map(|r| {
                r.kerbs
                    .iter()
                    .enumerate()
                    .filter_map(move |(h, k)| Some((stand_window(w, r, h)?, (k.road, k.s))))
            })
            .collect(),
        None => Vec::new(),
    };
    let mouths = drive_mouths(w);
    let mut out: Vec<Kerb> = Vec::with_capacity(halts.len());
    for (h, &plot) in halts.iter().enumerate() {
        let t = when.and_then(|when| when.get(h)).copied().unwrap_or(0);
        let taken: Vec<(u32, f32)> = others
            .iter()
            .filter(|(span, _)| span.0 <= t && t <= span.1)
            .map(|&(_, k)| k)
            .collect();
        let mut kerb = kerb_at(&w.travel, &w.meta, home_way(w, plot)?, &taken, &mouths)?;
        if let Some(last) = out.last()
            && last.road == kerb.road
            && (last.s - kerb.s).abs() < TAXI_LEN + STAND_GAP
        {
            kerb.s = last.s;
        }
        out.push(kerb);
    }
    Some(out)
}

/// The stands for `halts` (`kerbs_for`) and the way by them (`way_for`): in by `way_in` and out
/// by `way_out`, else (a leisure run) the ways out nearest its first stand and its last; the run
/// coming into town at `enter` (of its route and the ticks along it of each stand). Timed first
/// by where it'd stand with no other taxi about, then clear of the other runs' taxis standing at
/// about the time it's at each (a run's stands held for all its hours in town left a house in a
/// row of driveways with none, and its riders walked: the run home didn't come).
pub(super) fn stands_and_way(
    w: &World,
    halts: &[u32],
    skip: Option<u32>,
    way_in: Option<[f32; 2]>,
    way_out: Option<[f32; 2]>,
    enter: impl Fn(&crate::cars::Route, &[f32]) -> u64,
) -> Option<(Vec<Kerb>, Laid, u64)> {
    let ways = |kerbs: &[Kerb]| {
        Some((
            way_in.or_else(|| nearest_way_out(w, kerbs.first()?))?,
            way_out.or_else(|| nearest_way_out(w, kerbs.last()?))?,
        ))
    };
    let alone = kerbs_for(w, halts, skip, None)?;
    let (a, b) = ways(&alone)?;
    let laid = way_for(w, a, &alone, b, None)?;
    let start = enter(&laid.route, &laid.at);
    // (A stand another taxi has then, with none other clear of the driveways near: as much later
    // as clears it, `STAND_LATER` at most; neighbours' late runs as a game begins all come at once.)
    let (later, kerbs) = (0..=STAND_LATER / STAND_NEAR).find_map(|k| {
        let later = k * STAND_NEAR;
        let when: Vec<u64> = (0..halts.len())
            .map(|h| {
                start + later + laid.at.get(h).map_or(0, |t| t.ceil() as u64) + h as u64 * DWELL
            })
            .collect();
        Some((later, kerbs_for(w, halts, skip, Some(&when))?))
    })?;
    if kerbs
        .iter()
        .zip(&alone)
        .all(|(k, l)| k.road == l.road && k.s == l.s)
    {
        return Some((kerbs, laid, later));
    }
    let (a, b) = ways(&kerbs)?;
    let laid = way_for(w, a, &kerbs, b, None)?;
    Some((kerbs, laid, later))
}

/// A taxi's way in at `exit`, to stand at each of `kerbs` and out at `exit` again: its route and
/// the ticks along it where it stands at each (`lay_route`), clear of places held up where it can
/// be (`jammed`); `had`, if it's the way for these (`Laid::is_for`). None: no way.
fn way_for(
    w: &World,
    way_in: [f32; 2],
    kerbs: &[Kerb],
    way_out: [f32; 2],
    had: Option<&Laid>,
) -> Option<Laid> {
    let jams = jammed(w);
    if let Some(had) = had.filter(|l| l.is_for(way_in, kerbs, way_out, &jams)) {
        return Some(had.clone());
    }
    let (route, at) = lay_way(w, way_in, kerbs, way_out, &jams)?;
    Some(Laid {
        way_in,
        kerbs: kerbs.to_vec(),
        way_out,
        jams,
        route,
        at,
    })
}

/// `way_for`'s way round `jams`, laid.
fn lay_way(
    w: &World,
    way_in: [f32; 2],
    kerbs: &[Kerb],
    way_out: [f32; 2],
    jams: &[([f32; 2], f32)],
) -> Option<(crate::cars::Route, Vec<f32>)> {
    let net = Net::new(&w.travel, &w.meta, jams);
    let round = plan_way(&w.travel, &net, way_in, kerbs, way_out)
        .and_then(|legs| lay_route(&w.travel, &net, &legs));
    if jams.is_empty() {
        return round;
    }
    // (A way round a hold-up only when it's no longer than waiting it out would be,
    // `DETOUR_MOST`: else the straight way, the hold-up waited out at the edge of town if it's
    // still there, `run_on`.)
    let plain = Net::new(&w.travel, &w.meta, &[]);
    let straight = plan_way(&w.travel, &plain, way_in, kerbs, way_out)
        .and_then(|legs| lay_route(&w.travel, &plain, &legs));
    match (round, straight) {
        (Some(r), Some(s)) if r.0.ticks() > s.0.ticks() + DETOUR_MOST as f32 => Some(s),
        (Some(r), _) => Some(r),
        (None, s) => s,
    }
}

/// Run `k`'s route and the ticks along it where it stands at each halt, and where it stands there
/// (its kerbs as planned; worked out again, after a load). None: no way.
pub(super) fn route_of(
    w: &World,
    k: usize,
) -> Option<(crate::cars::Route, Vec<f32>, Vec<Kerb>, u64)> {
    route_via(w, k, None)
}

/// `route_of`, coming in by the way out `by` (a point of `World::exits`) if one is given.
pub(super) fn route_via(
    w: &World,
    k: usize,
    by: Option<[f32; 2]>,
) -> Option<(crate::cars::Route, Vec<f32>, Vec<Kerb>, u64)> {
    let run = &w.taxis.runs[k];
    // (A leisure run: in by the way out nearest its first halt, out by the one nearest its last.)
    let exit = match run.place {
        Some(_) => None,
        None => Some(*w.exits.get(run.group.1 as usize)?),
    };
    let way_in = by.or(exit);
    if run.kerbs.len() != run.halts() as usize {
        // (Coming into town now, or when it's due.)
        let enter = w.tick.max(run.enter);
        let (kerbs, laid, later) =
            stands_and_way(w, &halts_of(run), Some(run.id), way_in, exit, |_, _| enter)?;
        return Some((laid.route, laid.at, kerbs, later));
    }
    let kerbs = run.kerbs.clone();
    let way_in = match way_in {
        Some(p) => p,
        None => nearest_way_out(w, kerbs.first()?)?,
    };
    let way_out = match exit {
        Some(p) => p,
        None => nearest_way_out(w, kerbs.last()?)?,
    };
    let laid = way_for(w, way_in, &kerbs, way_out, run.laid.as_deref())?;
    Some((laid.route, laid.at, kerbs, 0))
}

/// The way in at `at` (a point of `World::exits`): how many cars wait out of sight to come in by
/// it (`Car::waits_to_come_in_at`), and stand still on their way near it (`QUEUE_NEAR`, a second
/// of drive time or more): a queue into its first junction.
pub(super) fn queued(w: &World, at: [f32; 2]) -> usize {
    let outside = w.cars.iter().filter(|c| c.waits_to_come_in_at(at)).count();
    let held = w
        .cars
        .iter()
        .zip(&w.taxis.still)
        .filter(|(_, s)| {
            s.0.is_some_and(|p| crate::travel::dist(p, at) < QUEUE_NEAR)
                && w.tick >= s.1 + crate::drive_secs(1.0)
        })
        .count();
    outside + held
}
