//! Pedestrians crossing, and the kerb they stop at.
//!
//! Split from `traffic.rs` (see `mod.rs`).

use super::*;

/// The lit junctions' crosswalks by where they are (`Travel::crossings`, `kerb_stop`): a grid of
/// `CROSS_CELL` m cells from the town's corner, each with the junctions whose crosswalks reach
/// into it.
#[derive(Debug, Clone, Default)]
pub struct Crossings {
    cols: usize,
    rows: usize,
    cells: Vec<Vec<u16>>,
}

/// `Crossings`' cells (m), and how far apart a step is looked along for crosswalks
/// (`kerb_stop`, m).
const CROSS_CELL: f32 = 16.0;
const KERB_STEP: f32 = 0.25;

impl Crossings {
    pub(crate) fn new(signals: &[Signal]) -> Self {
        let lit = || signals.iter().filter(|s| s.walk_clear > 0);
        let far = lit().fold([0.0f32; 2], |m, s| {
            let r = s.crossing_reach();
            [m[0].max(s.at[0] + r), m[1].max(s.at[1] + r)]
        });
        let cell = |v: f32| (v.max(0.0) / CROSS_CELL) as usize;
        let (cols, rows) = (cell(far[0]) + 1, cell(far[1]) + 1);
        if lit().next().is_none() {
            return Self::default();
        }
        let mut cells = vec![Vec::new(); cols * rows];
        for (i, s) in signals.iter().enumerate() {
            if s.walk_clear == 0 {
                continue;
            }
            let r = s.crossing_reach();
            for z in cell(s.at[1] - r)..=cell(s.at[1] + r).min(rows - 1) {
                for x in cell(s.at[0] - r)..=cell(s.at[0] + r).min(cols - 1) {
                    cells[z * cols + x].push(i as u16);
                }
            }
        }
        Self { cols, rows, cells }
    }
}

/// Whether lit junction `s`'s crosswalk on arm `arm` is open to those on foot: its walk phase,
/// while they may step out (`WALK_OPEN`), or a side road's on its main road's green
/// (`Signal::opens_with`).
pub(crate) fn walk_open(signals: &[Signal], lights: &[Light], s: usize, arm: usize) -> bool {
    let sig = &signals[s];
    lights.get(s).is_some_and(|l| {
        l.stage == Stage::Green
            && (sig.walk_phase() == Some(l.phase) || sig.opens_with(arm) == Some(l.phase))
    })
}

/// Where someone on foot (riding, they go on with the traffic) stepping from `from` to `to` stops
/// at a lit junction's crosswalks (`Signal::crossing`: its box too, between the kerbs, and its arms
/// out to the far side of their crosswalks) while they're closed to them (`walk_open`): the
/// junction (of those whose crosswalks are there, the nearest), the arm whose crosswalk it is, and
/// how far along the step they come (0 to 1), at the kerb. None: on (out of reach of any; one
/// whose crosswalk there is open to them; or on a junction's crosswalks already, on across, out of it and over any next to it: two junctions a
/// few metres apart, their walks apart, kept someone waiting on one for the other's). One crossing
/// a road away from the junctions goes as ever.
pub(crate) fn kerb_stop(
    travel: &crate::travel::Travel,
    lights: &[Light],
    from: [f32; 2],
    to: [f32; 2],
) -> Option<(u16, u8, f32)> {
    let c = &travel.crossings;
    if c.cells.is_empty() {
        return None;
    }
    let cell = |v: f32, n: usize| ((v.max(0.0) / CROSS_CELL) as usize).min(n - 1);
    let (x0, x1) = (
        cell(from[0].min(to[0]), c.cols),
        cell(from[0].max(to[0]), c.cols),
    );
    let (z0, z1) = (
        cell(from[1].min(to[1]), c.rows),
        cell(from[1].max(to[1]), c.rows),
    );
    // (The junctions near the step, once each.)
    let mut near: [u16; 8] = [0; 8];
    let mut count = 0;
    for z in z0..=z1 {
        for x in x0..=x1 {
            for &s in &c.cells[z * c.cols + x] {
                if !near[..count].contains(&s) && count < near.len() {
                    near[count] = s;
                    count += 1;
                }
            }
        }
    }
    let signals = &travel.signals;
    let near = &near[..count];
    if near.is_empty() || near.iter().any(|&s| signals[usize::from(s)].crossing(from)) {
        return None;
    }
    let n = (dist(from, to) / KERB_STEP).ceil().max(1.0) as usize;
    for k in 1..=n {
        let f = k as f32 / n as f32;
        let p = [
            from[0] + (to[0] - from[0]) * f,
            from[1] + (to[1] - from[1]) * f,
        ];
        let mut here = near
            .iter()
            .filter_map(|&s| Some((s, signals[usize::from(s)].crossing_arm(p)?)))
            .peekable();
        if here.peek().is_none() {
            continue;
        }
        let mut stop: Option<(u16, usize, f32)> = None;
        for (s, arm) in here {
            if walk_open(signals, lights, usize::from(s), arm) {
                return None;
            }
            let d = dist(p, signals[usize::from(s)].at);
            if stop.is_none_or(|(.., e)| d < e) {
                stop = Some((s, arm, d));
            }
        }
        return stop.map(|(s, arm, _)| (s, arm as u8, (k - 1) as f32 / n as f32));
    }
    None
}

/// How far along their way someone on foot is looked at for a zebra they're about to step onto (m,
/// `on_crossings`), a step at a time (`KERB_STEP`).
const ZEBRA_LOOK: f32 = 1.5;

impl Crossings {
    /// The junctions whose crosswalks reach into the cell `p` is in.
    fn at(&self, p: [f32; 2]) -> &[u16] {
        if self.cells.is_empty() || p[0] < 0.0 || p[1] < 0.0 {
            return &[];
        }
        let (x, z) = ((p[0] / CROSS_CELL) as usize, (p[1] / CROSS_CELL) as usize);
        if x >= self.cols || z >= self.rows {
            return &[];
        }
        &self.cells[z * self.cols + x]
    }
}

/// Those on foot at the crosswalks this tick (`Traffic::crossing`, `Traffic::zebra_walkers`): on
/// a lit junction's; and on a zebra of one giving way or a lit junction's side road's open on its
/// main road's green (`Signal::opens_with`), or walking (`Phase::Routing`) with one `ZEBRA_LOOK`
/// on along their way (a lit one's while it's open to them). Not those riding (with the traffic),
/// nor anyone at something.
fn on_crossings(w: &mut World) {
    let mut crossing = std::mem::take(&mut w.traffic.crossing);
    let mut zebra = std::mem::take(&mut w.traffic.zebra_walkers);
    crossing.clear();
    zebra.clear();
    let travel = &w.travel;
    let lights = &w.traffic.lights;
    // (A lit junction's side road's crosswalk, open to them now, or whatever zebra they're on.)
    let on_lit = |p: [f32; 2], open: bool| {
        travel.crossings.at(p).iter().find_map(|&j| {
            let sig = &travel.signals[usize::from(j)];
            let a = sig.crossing_arm(p)?;
            (sig.opens_with(a).is_some()
                && (!open || walk_open(&travel.signals, lights, usize::from(j), a)))
            .then_some((true, j, a as u8))
        })
    };
    let on_zebra = |p: [f32; 2], open: bool| {
        travel
            .zebras
            .at(p)
            .iter()
            .find_map(|&j| {
                let a = travel.give_way[usize::from(j)].crossing_arm(p)?;
                Some((false, j, a as u8))
            })
            .or_else(|| on_lit(p, open))
    };
    let lit = !travel.signals.is_empty();
    let zebras = !travel.give_way.is_empty();
    if lit || zebras {
        for s in w.sims.iter().filter(|s| s.here() && s.riding.is_none()) {
            let p = s.pos;
            let mut on_side = None;
            if lit
                && let Some((j, a)) = travel
                    .crossings
                    .at(p)
                    .iter()
                    .find_map(|&j| Some((j, travel.signals[usize::from(j)].crossing_arm(p)?)))
            {
                crossing.push((j, a as u8));
                if travel.signals[usize::from(j)].opens_with(a).is_some() {
                    on_side = Some((true, j, a as u8));
                }
            }
            if let Some((lit, junction, arm)) = on_side {
                zebra.push(OnZebra {
                    lit,
                    junction,
                    arm,
                    id: s.id,
                    on: true,
                });
                continue;
            }
            let Some(crate::world::Phase::Routing { waypoints, next }) =
                s.current().map(|a| &a.phase)
            else {
                continue;
            };
            if let Some((lit, junction, arm)) = on_zebra(p, false) {
                zebra.push(OnZebra {
                    lit,
                    junction,
                    arm,
                    id: s.id,
                    on: true,
                });
                continue;
            }
            // (Along their way, a step at a time: from where they are to the next waypoints.)
            let mut from = p;
            let mut left = ZEBRA_LOOK;
            'look: for &to in waypoints.iter().skip(*next) {
                let d = dist(from, to);
                let steps = (d.min(left) / KERB_STEP).ceil() as usize;
                for k in 1..=steps {
                    let f = (k as f32 * KERB_STEP).min(d.min(left)) / d.max(1e-4);
                    let q = [
                        from[0] + (to[0] - from[0]) * f,
                        from[1] + (to[1] - from[1]) * f,
                    ];
                    if let Some((lit, junction, arm)) = on_zebra(q, true) {
                        zebra.push(OnZebra {
                            lit,
                            junction,
                            arm,
                            id: s.id,
                            on: false,
                        });
                        break 'look;
                    }
                }
                left -= d;
                if left <= 0.0 {
                    break;
                }
                from = to;
            }
        }
    }
    crossing.sort_unstable();
    crossing.dedup();
    w.traffic.crossing = crossing;
    w.traffic.zebra_walkers = zebra;
}

/// The tick's claims, before the cars go on (`Traffic::begin`).
pub(crate) fn begin(w: &mut World) {
    // (Those on foot waiting at a lit junction's crosswalks, `kerb_stop`: those who took a turn
    // this tick (`lod.rs`) only if they waited this tick, the others as on their last turn. Gone
    // on, or off doing something else, they no longer ask for the walk: a request standing for a
    // turn's length after brought round walk phases nobody waited for.)
    let walking = crate::profile::start();
    let mut walkers = std::mem::take(&mut w.traffic.walkers);
    walkers.clear();
    if !w.travel.signals.is_empty() {
        let tick = w.tick;
        let signals = &w.travel.signals;
        walkers.extend(w.sims.iter().filter_map(|s| {
            let (j, arm, _) = s.kerb_wait.filter(|&(.., t)| {
                if s.steps > 0 {
                    t == tick
                } else {
                    t + crate::lod::FAR_TICKS >= tick
                }
            })?;
            Some((j, signals.get(usize::from(j))?.wants(usize::from(arm))?))
        }));
        walkers.sort_unstable();
        walkers.dedup();
    }
    w.traffic.walkers = walkers;
    crate::profile::end("traffic::begin/walkers", walking);
    crate::profile::phase("traffic::begin/on_crossings", || on_crossings(w));
    crate::profile::phase("traffic::begin/track_stall", || crate::cars::track_stall(w));
    crate::profile::phase("traffic::begin/sight", || crate::cars::sight(w));
    crate::profile::phase("traffic::begin/pass_stands", || crate::cars::pass_stands(w));
    crate::profile::phase("traffic::begin/unjam", || crate::cars::unjam(w));
    let moving = crate::profile::start();
    let mut movers = Vec::new();
    let mut places = Vec::new();
    let mut approaching = std::mem::take(&mut w.traffic.approaching);
    approaching.clear();
    crate::cars::movers(w, &mut movers, &mut places, &mut approaching);
    w.traffic.approaching = approaching;
    crate::profile::end("traffic::begin/movers", moving);
    let tick = w.tick;
    crate::profile::phase("traffic::begin/claims", || {
        w.traffic.begin(tick, movers, &places, &w.travel.signals)
    });
}
