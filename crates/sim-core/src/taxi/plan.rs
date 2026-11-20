//! Deciding who rides, and booking the runs that carry them.

use super::*;

/// Where a taxi would stand for each house with no other taxi about (`stop_for`): found once
/// (`warm_kerbs`), as it changes only with the roads, the driveways and the house's door, not
/// with the time (asked for by everyone who might ride, each minute, it was most of a big
/// world's step). Recomputed after a load the same as it was.
#[derive(Debug, Clone, Default)]
pub(crate) struct Kerbs {
    /// What they were found for (`kerb_stamp`): the driveways and the network.
    stamp: u64,
    /// `drive_mouths` for it.
    mouths: Vec<(u32, f32)>,
    /// By plot: its door (`Plot::arrival_point`, bits) and its stand, once found.
    plots: Vec<Option<([u32; 2], Option<Kerb>)>>,
}

/// What every house's stand (`stop_for`) depends on but its own door: the driveways, the
/// network and the public parking (a hash; 0 never: none found yet).
fn kerb_stamp(w: &World) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let g = &w.travel.graph;
    (g.roads.len(), g.edges.len(), g.nodes.len(), g.boxes.len()).hash(&mut h);
    for d in &w.drives {
        (d.plot, d.road).hash(&mut h);
        for v in [
            d.x,
            d.z,
            d.front,
            d.front_x,
            d.street,
            d.s.unwrap_or(f32::NAN),
        ] {
            v.to_bits().hash(&mut h);
        }
    }
    for p in &w.travel.file.parking {
        (p.x.to_bits(), p.z.to_bits()).hash(&mut h);
    }
    h.finish().max(1)
}

/// Finds the stand (`stop_for`) of every household's house not yet found (all again when the
/// driveways or the network have changed since, `kerb_stamp`).
pub(crate) fn warm_kerbs(w: &mut World) {
    let stamp = kerb_stamp(w);
    if w.taxis.kerbs.stamp != stamp {
        w.taxis.kerbs = Kerbs {
            stamp,
            mouths: drive_mouths(w),
            plots: Vec::new(),
        };
    }
    for h in 0..w.households.len() {
        let Some(plot) = w.households[h].plot else {
            continue;
        };
        let Some(door) = w.plots.get(plot as usize).map(|p| p.arrival_point()) else {
            continue;
        };
        let door = door.map(f32::to_bits);
        let kerbs = &w.taxis.kerbs;
        if kerbs
            .plots
            .get(plot as usize)
            .is_some_and(|k| k.is_some_and(|k| k.0 == door))
        {
            continue;
        }
        let kerb = home_way(w, plot)
            .and_then(|way| kerb_at(&w.travel, &w.meta, way, &[], &w.taxis.kerbs.mouths));
        let plots = &mut w.taxis.kerbs.plots;
        if plots.len() <= plot as usize {
            plots.resize(plot as usize + 1, None);
        }
        plots[plot as usize] = Some((door, kerb));
    }
}

fn is_zero(v: &u64) -> bool {
    *v == 0
}

/// A taxi's run: to work (or school) or home.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: u32,
    /// Home from work (drop-offs), else to it (pick-ups).
    #[serde(default)]
    pub home: bool,
    /// A leisure run (a visit or an outing, `book_leisure`, `book_back`): the place (a plot) it
    /// takes its riders to (`home` false: picked up at home, put down there) or back from (`home`
    /// true: picked up there, put down at home); and whether the player sent them. None: work or
    /// school.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub directed: bool,
    /// Its group (`group`) and the way out it comes in and goes out by (a `World::exits` index).
    pub group: (u32, u32),
    /// When it comes into town (ticks), and (there) when it's to be out of it again.
    pub enter: u64,
    #[serde(default)]
    pub due_out: u64,
    /// When it was planned to be out of town (`time_run`; `due_out` follows it as it runs late):
    /// a rider whose work begins before that and the way off the map rode a late run
    /// (`Taxis::late_arrivals`). 0: as `due_out`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub planned_out: u64,
    /// Home: it leaves work at this tick (those done by then ride it), and comes in after the way
    /// off the map.
    #[serde(default)]
    pub depart: u64,
    pub stops: Vec<Stop>,
    /// The stop it's on its way to or stands at.
    #[serde(default)]
    pub next: u32,
    /// How far along its route it is (ticks), and since when it stands at its stop (a tick; 0:
    /// not standing).
    #[serde(default)]
    pub progress: f32,
    /// Its taxi turned round on its way (`cars::unjam`): `progress` is along the way it turned
    /// round to, which a load works out again (`cars::turned_round`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detour: Option<crate::cars::Detour>,
    #[serde(default)]
    pub since: u64,
    /// Its car slot (`World::cars` index) while it's in town. Not saved.
    #[serde(skip)]
    pub car: Option<u32>,
    /// The ticks along its route of each stop. Not saved.
    #[serde(skip)]
    pub(crate) at: Vec<f32>,
    /// Where it stands at each of its halts (`kerb_at`): as planned, else worked out as it comes
    /// into town (and again on load). Not saved.
    #[serde(skip)]
    pub(crate) kerbs: Vec<Kerb>,
    /// Its way as last laid (`way_for`) and what for, as it was timed: coming into town
    /// (`route_in`) round the same places held up, it takes that rather than laying the same way
    /// again. Not saved.
    #[serde(skip)]
    pub(crate) laid: Option<Box<Laid>>,
}

/// A taxi's way as laid (`way_for`): its route and the ticks along it where it stands at each of
/// `kerbs`, in at `way_in` and out at `way_out` round the places held up then (`jams`).
#[derive(Debug, Clone)]
pub(crate) struct Laid {
    pub(crate) way_in: [f32; 2],
    pub(crate) kerbs: Vec<Kerb>,
    pub(crate) way_out: [f32; 2],
    pub(crate) jams: Vec<([f32; 2], f32)>,
    pub(crate) route: crate::cars::Route,
    pub(crate) at: Vec<f32>,
}

impl Laid {
    /// Whether it's the way `way_for` lays for these (the network the same: it changes only with a
    /// load).
    pub(crate) fn is_for(
        &self,
        way_in: [f32; 2],
        kerbs: &[Kerb],
        way_out: [f32; 2],
        jams: &[([f32; 2], f32)],
    ) -> bool {
        self.way_in == way_in
            && self.way_out == way_out
            && self.jams == jams
            && self.kerbs.len() == kerbs.len()
            && self.kerbs.iter().zip(kerbs).all(|(a, b)| {
                a.road == b.road && a.s == b.s && a.mouth == b.mouth && a.axis == b.axis
            })
    }
}

/// A rider's stop on a run: their house's driveway (the plot), when the taxi is due there and
/// when they leave home for it; how far it's got.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    pub sim: u32,
    pub plot: u32,
    pub due: u64,
    #[serde(default)]
    pub leave: u64,
    pub state: StopState,
    /// Where the taxi stands for them along its run (`Run::at`): housemates share one.
    #[serde(default)]
    pub halt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopState {
    /// On their way to the kerb, or not yet.
    Booked,
    /// Called to its door as it stands for them (`door_for`): on their way to it, or getting in.
    Boarding,
    /// In it.
    Aboard,
    /// Left (not at the kerb in time), or out at home.
    Done,
}

/// The work taxi a resident is booked on (`world::Sim::taxi`): the run and their stop. Not saved:
/// worked out from the runs (`settle`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Booking {
    pub run: u32,
    pub stop: u8,
}

impl Run {
    pub(crate) fn stop_of(&self, sim: u32) -> Option<usize> {
        self.stops.iter().position(|s| s.sim == sim)
    }

    /// Whether it picks its riders up at the kerb (to work or school, out to a place, or back from
    /// one), and at which halt `st` is picked up (back from a place: its first, the place's).
    pub(crate) fn picks_up(&self) -> bool {
        !self.home || self.place.is_some()
    }

    /// Whether it's the taxi for one off on `task`: to work or school, to its place, home from it.
    pub(crate) fn serves(&self, task: &TaskKind) -> bool {
        match (self.place, self.home) {
            (None, _) => matches!(task, TaskKind::Work),
            (Some(p), false) => matches!(task, TaskKind::Visit { plot } if *plot == p),
            (Some(_), true) => matches!(task, TaskKind::GoHome),
        }
    }

    pub(crate) fn pickup_halt(&self, st: &Stop) -> u32 {
        if self.home { 0 } else { st.halt as u32 }
    }

    /// The halt where `st` gets out: home (`st.halt`), or out to a place, its last; none on to
    /// work.
    pub(crate) fn drop_halt(&self, st: &Stop) -> Option<u32> {
        match (self.home, self.place) {
            (true, _) => Some(st.halt as u32),
            (false, Some(_)) => Some(self.halts() - 1),
            (false, None) => None,
        }
    }

    /// How many halts it makes: one for each house (housemates' one), and the place it goes to or
    /// back from.
    pub(crate) fn halts(&self) -> u32 {
        let most = self
            .stops
            .iter()
            .map(|s| s.halt as u32 + 1)
            .max()
            .unwrap_or(0);
        most + u32::from(self.place.is_some() && !self.home)
    }
}

/// Resident `i`'s group for the taxi: the way out they leave by (whatever their work or school:
/// the taxi's shared; 0, then the way out's index).
pub(crate) fn group(w: &World, i: usize) -> Option<(u32, u32)> {
    let c = w.sims[i].commute?;
    Some((0, c.exit as u32))
}

/// Where the taxi would stand for resident `i` at home (`kerb_at`), no other taxi about: as found
/// by `warm_kerbs`, for the same door, driveways and network (`kerb_stamp`); else worked out (not
/// found yet, or found for a town since changed: whoever asks first, `warm_kerbs` or not).
pub(crate) fn stop_for(w: &World, i: usize) -> Option<Kerb> {
    let plot = w.households.get(w.sims[i].household as usize)?.plot?;
    let door = w
        .plots
        .get(plot as usize)?
        .arrival_point()
        .map(f32::to_bits);
    let kerbs = &w.taxis.kerbs;
    if let Some(Some((at, kerb))) = kerbs.plots.get(plot as usize)
        && *at == door
        && kerbs.stamp == kerb_stamp(w)
    {
        return *kerb;
    }
    kerb_at(
        &w.travel,
        &w.meta,
        home_way(w, plot)?,
        &[],
        &drive_mouths(w),
    )
}

/// Where each driveway meets its road (the network's road, and how far along it): a taxi stands
/// clear of them (`kerb_at`, `DRIVE_CLEAR`).
pub(crate) fn drive_mouths(w: &World) -> Vec<(u32, f32)> {
    let g = &w.travel.graph;
    w.drives
        .iter()
        .filter_map(|d| {
            let (r, s) = d.road.zip(d.s.filter(|s| s.is_finite()))?;
            g.at_source(r, s).map(|p| (p.road, p.s))
        })
        .collect()
}

/// Whether `i` would ride a taxi to work or school starting at `start`, and if so the stop at
/// home: an open-world commute a long way on foot, no car seat to go in, no bike to ride.
fn would_ride(w: &World, i: usize, start: u64) -> Option<Kerb> {
    let s = &w.sims[i];
    let c = s.commute?;
    let home = w.households.get(s.household as usize)?.plot;
    if !s.here()
        || c.home != home
        || c.bike.is_some()
        || c.walk - crate::life::OUT_OF_THE_HOUSE <= w.content.taxi.min_walk
        || crate::cars::lead_ticks(w, i, start) > 0
    {
        return None;
    }
    stop_for(w, i)
}

/// Once a tick (after the cars): plans the runs due (`plan`), drives those under way, the riders
/// in and out at their stops, and frees a taxi done with its run.
pub(crate) fn update(w: &mut World) {
    if w.travel.graph.is_empty() {
        return;
    }
    crate::profile::phase("taxi::update/note_still", || note_still(w));
    if w.tick.is_multiple_of(TICKS_PER_MINUTE) {
        crate::profile::phase("taxi::update/plan", || plan(w));
        let booking = crate::profile::start();
        close_home_runs(w);
        leisure_leaving(w);
        bring_backs_forward(w);
        book_backs(w);
        crate::profile::end("taxi::update/home_runs", booking);
        if w.tick.is_multiple_of(60 * TICKS_PER_MINUTE) {
            w.taxis.no_leisure.clear();
            w.taxis.left.clear();
            w.taxis.early.clear();
        }
    }
    crate::profile::phase("taxi::update/seat_riders", || seat_riders(w));
    let running = crate::profile::start();
    let mut k = 0;
    while k < w.taxis.runs.len() {
        if !run_on(w, k) {
            finish(w, k);
            w.taxis.runs.remove(k);
            continue;
        }
        k += 1;
    }
    crate::profile::end("taxi::update/runs", running);
}

/// The start of resident `i`'s work (or school) a taxi would take them to: today's still to come,
/// or begun no longer than `late_minutes` ago with them not yet gone (a game begun after they'd
/// have set off: late, but sooner than on foot); else the next.
pub(crate) fn start_for(w: &World, i: usize) -> Option<u64> {
    let s = &w.sims[i];
    let day = crate::clock::day(w.tick);
    let gone = match &s.job {
        Some(job) => job.last_shift_day == day,
        None => s.school_day == day,
    };
    let late = (w.content.career_rules.late_minutes * TICKS_PER_MINUTE as f32) as u64;
    crate::life::start_on(&w.content, s, w.tick)
        .filter(|&start| start + late > w.tick && !gone)
        .or_else(|| crate::life::next_start(w, i))
}

/// One who'd ride (`plan`): their group, the start they're grouped by, their start, who, and when
/// they'd set off on foot.
type Want = ((u32, u32), u64, u64, usize, u64);

/// The runs to work due to be planned: everyone who'd ride, by group (their way out), those
/// starting within `taxi.window` of the first of a group (all too late on foot already, together),
/// once one of them would set off on foot within `PLAN_LEAD` (and not yet on their way).
fn plan(w: &mut World) {
    warm_kerbs(w);
    let tick = w.tick;
    let window = (w.content.taxi.window * TICKS_PER_MINUTE as f32) as u64;
    w.taxis.declined.retain(|d| d.1 + window > tick);
    w.taxis.missed.retain(|d| d.1 + window > tick);
    let mut want: Vec<Want> = Vec::new();
    for i in 0..w.sims.len() {
        let Some(start) = start_for(w, i) else {
            continue;
        };
        let s = &w.sims[i];
        if start > tick + PLAN_HORIZON
            || s.taxi.is_some()
            || s.away_until.is_some()
            || crate::life::heading_out(s) && !setting_off(w, i)
            || w.taxis.declined.contains(&(i as u32, start))
        {
            continue;
        }
        let Some(g) = group(w, i) else { continue };
        let on_foot = start.saturating_sub(s.commute.map_or(0, |c| c.ticks));
        // (Too late to be there in time on foot already: one late run for all of them going out
        // that way, whenever their work starts.)
        let key = if on_foot <= tick { 0 } else { start };
        if would_ride(w, i, start).is_some() {
            want.push((g, key, start, i, on_foot));
        }
    }
    want.sort_by_key(|&(g, key, start, i, _)| (g, key, start, i));
    let mut k = 0;
    while k < want.len() {
        let (g, first, ..) = want[k];
        let n = want[k..]
            .iter()
            .take_while(|m| m.0 == g && m.1 <= first + window)
            .count();
        let members: Vec<(usize, u64)> = want[k..k + n].iter().map(|m| (m.3, m.2)).collect();
        let soonest = want[k..k + n].iter().map(|m| m.4).min().unwrap_or(0);
        k += n;
        // (Only once one of them would soon set off on foot.)
        if soonest > tick + PLAN_LEAD {
            continue;
        }
        plan_group(w, g, &members);
    }
}

/// Plans the runs to work for `members` of group `g` (each with when their work starts): in
/// taxis of `taxi.seats` in a short order from the way in. Those no run fits (no way by the roads,
/// or the taxi no sooner than the walk) are declined for that start.
fn plan_group(w: &mut World, g: (u32, u32), members: &[(usize, u64)]) {
    let Some(&exit) = w.exits.get(g.1 as usize) else {
        return;
    };
    let mut left: Vec<(usize, u64, Kerb)> = members
        .iter()
        .filter_map(|&(i, start)| Some((i, start, would_ride(w, i, start)?)))
        .collect();
    for &(i, start) in members {
        if !left.iter().any(|m| m.0 == i) {
            w.taxis.declined.push((i as u32, start));
        }
    }
    let seats = w.content.taxi.seats;
    let pos = |w: &World, st: &Kerb| w.travel.graph.point(st.mouth);
    while !left.is_empty() {
        // (A short order: nearest the way in first, then on nearest to the last.)
        let mut chunk: Vec<(usize, u64, Kerb)> = Vec::new();
        let mut at = exit;
        while chunk.len() < seats && !left.is_empty() {
            let j = (0..left.len())
                .min_by(|&a, &b| {
                    let d = |m: &(usize, u64, Kerb)| crate::travel::dist(pos(w, &m.2), at);
                    d(&left[a])
                        .total_cmp(&d(&left[b]))
                        .then(left[a].0.cmp(&left[b].0))
                })
                .expect("some left");
            let m = left.remove(j);
            at = pos(w, &m.2);
            chunk.push(m);
        }
        // (No way by the roads, or no sooner than the walk for some: without them, once.)
        let mut tries = 0;
        while !chunk.is_empty() && tries < 3 {
            tries += 1;
            match time_run(w, g, exit, &chunk) {
                Ok(run) => {
                    book(w, run);
                    break;
                }
                Err(slow) if !slow.is_empty() => {
                    for i in slow {
                        if let Some(p) = chunk.iter().position(|m| m.0 == i) {
                            let m = chunk.remove(p);
                            w.taxis.declined.push((m.0 as u32, m.1));
                        }
                    }
                }
                Err(_) => break,
            }
        }
        for m in chunk {
            if w.sims[m.0].taxi.is_none() {
                w.taxis.declined.push((m.0 as u32, m.1));
            }
        }
    }
}

/// The run to work for `chunk` (each rider with their work's start and their stop), timed back
/// from the first one's start: Err with the riders the taxi gets out no sooner than their walk
/// (none: no way by the roads at all).
fn time_run(
    w: &World,
    g: (u32, u32),
    exit: [f32; 2],
    chunk: &[(usize, u64, Kerb)],
) -> Result<Run, Vec<usize>> {
    // (Housemates, next to each other in the order: one halt.)
    let plot_of = |i: usize| {
        w.households[w.sims[i].household as usize]
            .plot
            .unwrap_or(u32::MAX)
    };
    let mut halts: Vec<u32> = Vec::new();
    let mut halt_of = Vec::with_capacity(chunk.len());
    for m in chunk {
        let plot = plot_of(m.0);
        if halts.last() != Some(&plot) {
            halts.push(plot);
        }
        halt_of.push(halts.len() - 1);
    }
    let off_map = crate::cars::off_map_ticks(w);
    let first = chunk.iter().map(|m| m.1).min().unwrap_or(0);
    let due_out = first.saturating_sub(off_map);
    let tick = w.tick;
    // (Spare time for the way: `SLACK` and a share of it, those in its way on the roads.)
    let ride_of = |route: &crate::cars::Route| {
        let ride = route.ticks().ceil() as u64 + halts.len() as u64 * DWELL;
        (ride, SLACK + ride * SLACK_SHARE / 100)
    };
    // (Clear of other taxis standing about when it's at each: `stands_and_way`, it coming in to
    // be out of town in time, or now.)
    let (kerbs, laid, later) = stands_and_way(w, &halts, None, Some(exit), Some(exit), |r, _| {
        let (ride, spare) = ride_of(r);
        due_out.saturating_sub(ride + spare).max(tick + 1)
    })
    .ok_or(vec![])?;
    let (ride, spare) = ride_of(&laid.route);
    let at = laid.at.clone();
    // Each rider's walk to their kerb (ticks: from home, or from wherever they are) and when the
    // taxi's there.
    let lead: Vec<u64> = chunk
        .iter()
        .enumerate()
        .map(|(k, &(i, _, _))| {
            let sim = &w.sims[i];
            let walk = sim.walk_per_minute(&w.content);
            // (From their door: home by then.)
            let from = w
                .households
                .get(sim.household as usize)
                .and_then(|h| h.plot)
                .map_or(sim.pos, |p| w.plots[p as usize].arrival_point());
            let to_kerb = ((w.travel.eta(
                from,
                wait_spot(&w.travel, &kerbs[halt_of[k]]),
                crate::travel::Mode::Foot,
                walk,
            ) + crate::life::OUT_OF_THE_HOUSE)
                * crate::TICKS_PER_MINUTE as f32) as u64;
            to_kerb.max(TO_KERB_LEAST) + KERB_EARLY
        })
        .collect();
    let offset = |h: usize| at[h].ceil() as u64 + h as u64 * DWELL;
    // Out of town in time for the first of them; else, too late for that (a game just begun, a
    // rider missed by a taxi before), as soon as it can come with each of them at their kerb by
    // then: late, but out sooner than on foot.
    let mut enter = due_out.saturating_sub(ride + spare).max(tick + 1);
    let mut least = tick + 1;
    for (k, &l) in lead.iter().enumerate() {
        least = least.max((tick + 1 + l).saturating_sub(offset(halt_of[k])));
    }
    // (Later, its stand another's till then: `stands_and_way`.)
    if later > 0 {
        least = least.max(enter + later);
    }
    enter = enter.max(least);
    // (Not into town by the same way at the same time as another run: a slot before it, as far
    // as `ENTRY_SHIFT` sooner.)
    let crowded = |e: u64| {
        w.taxis
            .runs
            .iter()
            .any(|r| !r.home && r.group.1 == g.1 && r.enter.abs_diff(e) < ENTRY_GAP)
    };
    let latest = enter;
    while crowded(enter) && enter >= least + ENTRY_GAP && latest - enter < ENTRY_SHIFT {
        enter -= ENTRY_GAP;
    }
    if crowded(enter) {
        enter = latest;
    }
    let due_out = enter + ride + spare;
    let mut slow = Vec::new();
    let mut out = Vec::new();
    for (k, &(i, start, _)) in chunk.iter().enumerate() {
        let h = halt_of[k];
        let due = enter + offset(h);
        let sim = &w.sims[i];
        let leave = due.saturating_sub(lead[k]);
        // (Sooner than on foot: from leaving home to out of town no longer than the walk out, and
        // out of town no later than setting off on foot now would have them.)
        let ticks = sim.commute.map_or(0, |c| c.ticks);
        let walk_out = ticks.saturating_sub(off_map);
        let by_taxi = due_out.saturating_sub(leave);
        let on_foot_out = start.saturating_sub(ticks).max(tick) + walk_out;
        if by_taxi >= walk_out || due_out > on_foot_out {
            slow.push(i);
        }
        out.push(Stop {
            sim: i as u32,
            plot: w.households[sim.household as usize]
                .plot
                .unwrap_or(u32::MAX),
            due,
            leave,
            state: StopState::Booked,
            halt: h as u8,
        });
    }
    if !slow.is_empty() {
        return Err(slow);
    }
    Ok(Run {
        id: 0,
        home: false,
        place: None,
        directed: false,
        group: g,
        enter,
        due_out,
        planned_out: due_out,
        depart: 0,
        stops: out,
        next: 0,
        progress: 0.0,
        detour: None,
        since: 0,
        car: None,
        at,
        kerbs,
        laid: Some(Box::new(laid)),
    })
}

/// Whether resident `i` has only just set off to work or school on foot: on their way out of
/// town walking (no car, bike or taxi), still by their door (`SETTING_OFF`). As at a game's start,
/// when those with a long walk set off at once.
fn setting_off(w: &World, i: usize) -> bool {
    let s = &w.sims[i];
    let walking = s
        .current()
        .is_none_or(|a| matches!(a.phase, Phase::Routing { .. }));
    let door = w
        .households
        .get(s.household as usize)
        .and_then(|h| h.plot)
        .map(|p| w.plots[p as usize].arrival_point());
    crate::life::heading_out(s)
        && walking
        && s.taxi.is_none()
        && s.bound_for_car.is_none()
        && s.bound_for_bike.is_none()
        && s.riding.is_none()
        && door.is_some_and(|d| crate::travel::dist(d, s.pos) < SETTING_OFF)
}

/// Books `run`'s riders on it and adds it; any just off on foot stop, to leave again for their
/// kerb in time for it (`leave_at`).
fn book(w: &mut World, mut run: Run) {
    run.id = w.taxis.next_id;
    w.taxis.next_id = w.taxis.next_id.wrapping_add(1);
    for (k, st) in run.stops.iter().enumerate() {
        let sim = &mut w.sims[st.sim as usize];
        sim.taxi = Some(Booking {
            run: run.id,
            stop: k as u8,
        });
        if sim
            .current()
            .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
        {
            sim.current = None;
        }
        if sim
            .queue
            .front()
            .is_some_and(|t| matches!(t.kind, TaskKind::Work))
        {
            sim.queue.pop_front();
        }
    }
    w.taxis.runs.push(run);
}
