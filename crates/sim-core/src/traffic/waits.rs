//! Who is waiting on whom, and keeping stopped traffic off junctions.

use super::*;

/// Whom car `key` (`cars::Driving::first`) waits for this tick: one in its way (another vehicle,
/// or those on foot or on a bike: their resident ids, below `1 << 63`), those holding
/// claims it asks for and the earlier asks it waits for. For checks (the generated worlds': a car
/// is stuck when nothing it waits for, however far round, goes on).
pub fn waits_on(w: &World, key: u64) -> Vec<u64> {
    waits_on_by(w, key, w.traffic.asks.iter().position(|a| a.key == key))
}

/// The asks by key as they are now: each key with the first of its asks, sorted (`waits_on_at`,
/// for a pass asking about many).
pub(crate) fn asks_by_key(w: &World) -> Vec<(u64, u32)> {
    let mut ix: Vec<(u64, u32)> = w
        .traffic
        .asks
        .iter()
        .enumerate()
        .map(|(i, a)| (a.key, i as u32))
        .collect();
    ix.sort_unstable();
    ix.dedup_by_key(|e| e.0);
    ix
}

/// `waits_on`, its ask found in `ix` (`asks_by_key`, made since the asks last changed).
pub(crate) fn waits_on_at(w: &World, key: u64, ix: &[(u64, u32)]) -> Vec<u64> {
    let ask = ix
        .binary_search_by_key(&key, |e| e.0)
        .ok()
        .map(|k| ix[k].1 as usize);
    debug_assert_eq!(ask, w.traffic.asks.iter().position(|a| a.key == key));
    waits_on_by(w, key, ask)
}

/// `waits_on` with car `key`'s ask, the first of its in `Traffic::asks` (None: none).
fn waits_on_by(w: &World, key: u64, ask: Option<usize>) -> Vec<u64> {
    let car = w.cars.get((key & !(1 << 63)) as usize);
    let mut out: Vec<u64> = car.and_then(|c| c.waits_for).into_iter().collect();
    // (Those on foot or on a bike in its way: their resident ids, no vehicle's key.)
    out.extend(
        car.into_iter()
            .flat_map(|c| c.on_foot.iter().map(|&i| u64::from(i))),
    );
    if let Some(a) = ask.and_then(|i| w.traffic.asks.get(i)) {
        out.extend(a.by.iter().chain(&a.fifo));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Whether vehicle `key` waits at a red light (its ask through lights not on its phase's green): it
/// isn't held up for good, the light changes (for checks).
pub fn at_red(w: &World, key: u64) -> bool {
    w.traffic.red_for(key)
}

/// How far along its way car `key` is (ticks); none, not on its way. For checks, with `waits_on`.
pub fn on_its_way(w: &World, key: u64) -> Option<f32> {
    let car = w.cars.get((key & !(1 << 63)) as usize)?;
    matches!(
        car.state,
        crate::cars::CarState::Leaving | crate::cars::CarState::Arriving
    )
    .then_some(car.progress)
}

/// The tiles the lit junctions keep those on foot off (`Signal::off_limits`, `Travel::
/// junction_tiles`), as `signals` have them: blocked, those freed since the last time given back,
/// and walkers finding their way again round where that changed. Never ground cut off: a way
/// over them left open from any they'd shut in (`left_open`).
fn keep_off(w: &mut World, signals: &[Signal]) {
    let mut tiles: Vec<usize> = Vec::new();
    let mut rects: Vec<[i32; 4]> = Vec::new();
    let mut taken = std::collections::HashSet::new();
    let gates: Vec<(i32, i32)> = w
        .plots
        .iter()
        .map(|p| {
            let a = p.arrival_point();
            (a[0].floor() as i32, a[1].floor() as i32)
        })
        .collect();
    for sig in signals.iter().filter(|s| !s.roundabout) {
        let r = sig
            .arms
            .iter()
            .map(|a| (a.stop + KEEP_OFF).max(dist(sig.at, a.span[1])) + a.kerb)
            .fold(0.0, f32::max)
            .ceil();
        let (x0, z0) = (
            (sig.at[0] - r).floor() as i32,
            (sig.at[1] - r).floor() as i32,
        );
        let (x1, z1) = ((sig.at[0] + r).ceil() as i32, (sig.at[1] + r).ceil() as i32);
        let mut mine = Vec::new();
        // (Its arms' roads' middle lines near it: a tile kept off is on one's carriageway, within
        // its kerb of it, whatever the arms' straight ways out say. Out along a way off its road,
        // the box took in a garden's planter, Applestead.)
        let g = &w.travel.graph;
        let lines: Vec<([f32; 2], [f32; 2], f32)> = sig
            .arms
            .iter()
            .flat_map(|a| {
                let pts = &g.roads[a.road as usize].pts;
                pts.windows(2)
                    .filter(|q| dist(q[0], sig.at).min(dist(q[1], sig.at)) <= r + 2.0)
                    .map(|q| (q[0], q[1], a.kerb))
                    .collect::<Vec<_>>()
            })
            .collect();
        let on_road = |q: [f32; 2]| {
            lines
                .iter()
                .any(|&(a, b, kerb)| crate::cars::to_segment(q, a, b) <= kerb)
        };
        for z in z0..z1 {
            for x in x0..x1 {
                let q = [x as f32 + 0.5, z as f32 + 0.5];
                if w.lot.in_bounds(x, z) && sig.off_limits(q) && on_road(q) {
                    mine.push(w.lot.tile_index(x, z));
                }
            }
        }
        if mine.is_empty() {
            continue;
        }
        mine.sort_unstable();
        let open = left_open(w, [x0, z0, x1, z1], &mine, &taken, &gates);
        mine.retain(|i| open.binary_search(i).is_err());
        taken.extend(mine.iter().copied());
        tiles.extend(mine);
        rects.push([x0, z0, x1, z1]);
    }
    tiles.sort_unstable();
    tiles.dedup();
    let old = std::mem::replace(&mut w.travel.junction_tiles, tiles);
    if old == w.travel.junction_tiles {
        return;
    }
    for &i in &old {
        if w.travel.junction_tiles.binary_search(&i).is_err()
            && let Some(b) = w.blocked.get_mut(i)
        {
            *b = w.travel.base_blocked.binary_search(&i).is_ok();
        }
    }
    for &i in &w.travel.junction_tiles {
        if let Some(b) = w.blocked.get_mut(i) {
            *b = true;
        }
    }
    for &i in &old {
        let (x, z) = w.lot.tile_xz(i);
        rects.push([x, z, x + 1, z + 1]);
    }
    for r in rects {
        w.nav_changed_in(r);
    }
}

/// Of `mine` (a lit junction's tiles to keep those on foot off, sorted, within `rect`), those left
/// open so that it cuts no ground off. Of the ground in `rect` and `KEEP_OFF_ROOM` round it,
/// walked as ever (`NavGrid` on the ground, but for the tiles the junctions keep them off: those
/// looked at before it, `taken`, and not the ones they kept them off till now): a
/// piece of it once `mine` are kept off that was joined to the edge's ground and no longer is (a
/// pocket of pavement walled round but for its side onto the road by the stop line, a gate
/// opening onto it) gets the shortest way over `mine` back to it; a place's gate on one of them
/// (`gates`), the same from there.
fn left_open(
    w: &World,
    rect: [i32; 4],
    mine: &[usize],
    taken: &std::collections::HashSet<usize>,
    gates: &[(i32, i32)],
) -> Vec<usize> {
    let [x0, z0, x1, z1] = rect;
    let (x0, z0) = ((x0 - KEEP_OFF_ROOM).max(0), (z0 - KEEP_OFF_ROOM).max(0));
    let (x1, z1) = (
        (x1 + KEEP_OFF_ROOM).min(w.lot.width as i32),
        (z1 + KEEP_OFF_ROOM).min(w.lot.depth as i32),
    );
    if x1 <= x0 || z1 <= z0 {
        return Vec::new();
    }
    let (cols, rows) = ((x1 - x0) as usize, (z1 - z0) as usize);
    let blocked = w.blocked();
    let lot = &w.lot;
    let xz = |k: usize| (x0 + (k % cols) as i32, z0 + (k / cols) as i32);
    let tile = |k: usize| {
        let (x, z) = xz(k);
        lot.tile_index(x, z)
    };
    // (As `NavGrid::tile_free` has it on the ground, but for the junctions' tiles.)
    let wall: Vec<bool> = (0..cols * rows)
        .map(|k| {
            let (x, z) = xz(k);
            let i = lot.tile_index(x, z);
            !lot.allocated(x, z)
                || lot.diag_blocks(x, z)
                || taken.contains(&i)
                || if w.travel.junction_tiles.binary_search(&i).is_ok() {
                    w.travel.base_blocked.binary_search(&i).is_ok()
                } else {
                    blocked.get(i).copied().unwrap_or(true)
                }
        })
        .collect();
    let kept: Vec<bool> = (0..cols * rows)
        .map(|k| mine.binary_search(&tile(k)).is_ok())
        .collect();
    let next = |k: usize| {
        let (x, z) = (k % cols, k / cols);
        let (ax, az) = xz(k);
        [
            (x > 0).then(|| k - 1),
            (x + 1 < cols).then(|| k + 1),
            (z > 0).then(|| k - cols),
            (z + 1 < rows).then(|| k + cols),
        ]
        .into_iter()
        .flatten()
        .filter(move |&m| {
            let (bx, bz) = xz(m);
            lot.edge_walkable(ax, az, bx, bz)
        })
    };
    // (Each tile's piece of ground, numbered from 1; 0 shut.)
    let pieces = |shut: &dyn Fn(usize) -> bool| {
        let mut piece = vec![0u32; cols * rows];
        let mut n = 0;
        let mut todo = Vec::new();
        for start in 0..piece.len() {
            if piece[start] != 0 || shut(start) {
                continue;
            }
            n += 1;
            piece[start] = n;
            todo.push(start);
            while let Some(k) = todo.pop() {
                for m in next(k) {
                    if piece[m] == 0 && !shut(m) {
                        piece[m] = n;
                        todo.push(m);
                    }
                }
            }
        }
        (piece, n as usize)
    };
    let (before, nb) = pieces(&|k| wall[k]);
    let (after, na) = pieces(&|k| wall[k] || kept[k]);
    let edge = |k: usize| {
        let (x, z) = (k % cols, k / cols);
        x == 0 || z == 0 || x + 1 == cols || z + 1 == rows
    };
    // (Of each piece before, the pieces after on the edge, and those that aren't.)
    let mut out = vec![false; na + 1];
    let mut joined = vec![false; nb + 1];
    for k in 0..cols * rows {
        if edge(k) && after[k] != 0 {
            out[after[k] as usize] = true;
            joined[before[k] as usize] = true;
        }
    }
    let mut cut: Vec<usize> = (0..cols * rows)
        .filter(|&k| after[k] != 0 && !out[after[k] as usize] && joined[before[k] as usize])
        .collect();
    // (A gate on the junction's tiles: a piece of its own.)
    cut.extend(gates.iter().filter_map(|&(x, z)| {
        ((x0..x1).contains(&x) && (z0..z1).contains(&z))
            .then(|| (z - z0) as usize * cols + (x - x0) as usize)
            .filter(|&k| kept[k])
    }));
    let mut open = Vec::new();
    let mut from = vec![usize::MAX; cols * rows];
    while let Some(&start) = cut.first() {
        // (The piece, all of it, and the shortest way over the junction's tiles to the edge's
        // ground.)
        let piece = after[start];
        let seeds: Vec<usize> = if piece == 0 {
            vec![start]
        } else {
            cut.iter().copied().filter(|&k| after[k] == piece).collect()
        };
        cut.retain(|k| !seeds.contains(k));
        from.iter_mut().for_each(|f| *f = usize::MAX);
        let mut todo = std::collections::VecDeque::new();
        for &k in &seeds {
            from[k] = k;
            todo.push_back(k);
        }
        let mut found = None;
        while let Some(k) = todo.pop_front() {
            if after[k] != 0 && out[after[k] as usize] {
                found = Some(k);
                break;
            }
            for m in next(k) {
                if from[m] == usize::MAX && !wall[m] && (kept[m] || after[m] != 0) {
                    from[m] = k;
                    todo.push_back(m);
                }
            }
        }
        let Some(mut k) = found else { continue };
        while from[k] != k {
            if kept[k] {
                open.push(tile(k));
            }
            k = from[k];
        }
        if kept[k] {
            open.push(tile(k));
        }
        // (Joined now: the piece is the edge's.)
        for &k in &seeds {
            if after[k] != 0 {
                out[after[k] as usize] = true;
            }
        }
    }
    open.sort_unstable();
    open.dedup();
    open
}

/// Per road of `g`, whether it's a roundabout's ring: the town file's road (`meta.roads`) has an
/// island in the middle.
fn rings(g: &Graph, meta: &serde_json::Value) -> Vec<bool> {
    let roads = meta.get("roads").and_then(|r| r.as_array());
    g.roads
        .iter()
        .map(|r| {
            roads
                .and_then(|list| list.get(r.source as usize))
                .is_some_and(|m| m.get("island").is_some_and(|i| !i.is_null()))
        })
        .collect()
}

/// The junctions (every one where `SIGNAL_ARMS` roads or more meet, `signal_at`): the big ones'
/// traffic lights (`Travel::signals`), the others giving way with their zebras
/// (`Travel::give_way`), and every car's claims worked out again with them (`Claim::gate`,
/// `zebra_stops`). Once the cars are made or loaded. Every head has a row in the snapshot
/// (`snapshot::MAX_LIGHTS`, made room for the biggest generated worlds' and then some); past that,
/// a junction is still lit, its heads only not drawn, and a debug build says so.
pub(crate) fn settle(w: &mut World) {
    let g = &w.travel.graph;
    let ring = rings(g, &w.meta);
    let (signals, give_way): (Vec<Signal>, Vec<Signal>) = (0..g.nodes.len())
        .filter_map(|n| signal_at(g, n, &ring))
        .partition(|s| s.lit);
    let heads: usize = signals.iter().map(|s| s.arms.len()).sum();
    debug_assert!(
        heads <= crate::snapshot::MAX_LIGHTS,
        "{heads} signal heads, past the snapshot's {} rows",
        crate::snapshot::MAX_LIGHTS
    );
    w.travel.crossings = Crossings::new(&signals);
    w.travel.zebras = Crossings::new(&give_way);
    keep_off(w, &signals);
    // (Each junction's box taken out to the far side of its crosswalks, a lit one's and a zebra:
    // nobody goes onto one without room to clear it beyond, and a way between two lit junctions too
    // near each other for a body to stand between them is one group, through both lights: `gate`.)
    let mut zones = boxes(&w.travel.graph);
    for sig in signals.iter().chain(&give_way) {
        for a in sig.arms.iter().filter(|a| a.crosswalk) {
            let mut d = BOX_LEAST;
            while d < a.walk_to {
                d = (d + ARM_STEP).min(a.walk_to);
                let q = [
                    sig.at[0] + a.way[0] * (d - ARM_REACH),
                    sig.at[1] + a.way[1] * (d - ARM_REACH),
                ];
                zones.push((q, ARM_REACH));
            }
        }
    }
    w.travel.graph.boxes = zones;
    // (Green on the phase each rests on before the first tick changes them: `change_lights`.)
    let had = w.traffic.lights.len();
    w.traffic.lights.truncate(signals.len());
    w.traffic
        .lights
        .extend(signals[had.min(signals.len())..].iter().map(|s| Light {
            phase: s.rest,
            ..Light::default()
        }));
    w.travel.signals = signals;
    w.travel.give_way = give_way;
    for car in &mut w.cars {
        car.reclaim(&w.travel);
    }
}

impl World {
    /// The junctions' traffic lights worked out again (`traffic::settle`), and the cars' claims
    /// with them: for a test that changes the roads after the world is made.
    #[doc(hidden)]
    pub fn settle_lights(&mut self) {
        settle(self);
    }
}
