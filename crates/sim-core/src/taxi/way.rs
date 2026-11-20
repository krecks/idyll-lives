//! Planning a way: the net of roads a taxi may take.
//!
//! Split from `taxi.rs` (see `mod.rs`).

use super::*;

/// A taxi's body keeps this far either side of a driveway's mouth (m, `kerb_at`: a car's length
/// and a margin; a taxi at its rider's own mouth held the household's car in the driveway).
pub(crate) const DRIVE_CLEAR: f32 = 5.5;
/// With no stand that clear within reach, at least this clear (m: half a driveway's mouth and a
/// little, so nobody's way out is across its body).
pub(crate) const DRIVE_LEAST: f32 = 2.0;
/// A taxi's stand keeps this far from a public parking spot (m, `kerb_at`): a car's length, and
/// a little to swing in or out.
pub(crate) const PARK_CLEAR: f32 = 4.0;

/// The radii of a taxi's turn round in a dead end's turning circle on a road `width` m wide of
/// `kind` (`u_turn`): the loop round its middle, the lane's offset, and how far before the end
/// the swerve into it begins (m).
fn u_turn_radii(width: f32, kind: crate::travel::RoadKind) -> (f32, f32, f32) {
    let kerb = crate::cars::kerb_of(width, kind) + TURNING;
    // (A front corner swinging over the kerb by `TURN_OVER` at most.)
    let reach = (kerb + TURN_OVER).powi(2) - (TAXI_LEN / 2.0).powi(2);
    let r = (reach.max(0.0).sqrt() - TAXI_WID / 2.0).max(2.5);
    let lane = crate::cars::lane_on(width, kind);
    let before = ((r + SWERVE).powi(2) - (lane + SWERVE).powi(2))
        .max(0.0)
        .sqrt();
    (r, lane, before)
}

/// How far before a dead end's end a taxi turning round there begins to (m).
pub(crate) fn u_turn_reach(width: f32, kind: crate::travel::RoadKind) -> f32 {
    u_turn_radii(width, kind).2
}

/// The taxis' road network as they drive it (`way_between`): the stretches of road at each node
/// a car may take, those of the nodes a car can turn round at (a dead end's turning circle with
/// its stretch long enough to swerve into it), and the stretches through a place held up now
/// (`jammed`).
pub(crate) struct Net {
    adj: Vec<Vec<u32>>,
    turn: Vec<bool>,
    jam: Vec<bool>,
}

impl Net {
    pub(crate) fn new(
        travel: &crate::travel::Travel,
        meta: &serde_json::Value,
        jammed: &[([f32; 2], f32)],
    ) -> Net {
        let g = &travel.graph;
        let mut adj = vec![Vec::new(); g.nodes.len()];
        for (k, e) in g.edges.iter().enumerate() {
            if g.roads[e.road as usize].drivable() {
                adj[e.a as usize].push(k as u32);
                adj[e.b as usize].push(k as u32);
            }
        }
        let turn = (0..g.nodes.len())
            .map(|n| {
                let [e] = adj[n][..] else { return false };
                let edge = &g.edges[e as usize];
                let road = &g.roads[edge.road as usize];
                let end = if n as u32 == edge.b && edge.s1 >= road.len() - 0.01 {
                    1
                } else if n as u32 == edge.a && edge.s0 <= 0.01 {
                    0
                } else {
                    return false;
                };
                dead_end(travel, meta, edge.road, end)
                    && edge.len() > u_turn_reach(road.width, road.kind) + 1.0
            })
            .collect();
        // (A stretch through one, about: sampled every few metres.)
        let jam = g
            .edges
            .iter()
            .map(|e| {
                if jammed.is_empty() {
                    return false;
                }
                let road = &g.roads[e.road as usize];
                let n = (e.len() / 4.0).ceil().max(1.0) as usize;
                (0..=n).any(|k| {
                    let p = road.at(e.s0 + e.len() * k as f32 / n as f32);
                    jammed
                        .iter()
                        .any(|&(m, r)| crate::travel::dist(p, m) <= r + JAM_NEAR)
                })
            })
            .collect();
        Net { adj, turn, jam }
    }
}

/// A stretch of road driven: the network's road, from and to how far along it (m).
type Piece = (u32, f32, f32);

/// The ways for a taxi from `from`, going `dir` along its road (+1: as `s` rises; both if
/// `None`), to `to` along the roads, as a car drives them, turning round only in a dead end's
/// turning circle (`Net::turn`): by the way it comes to `to` (+1, then -1), the cost (s) and the
/// stretches driven, one after another. None: no such way.
#[allow(clippy::type_complexity)]
fn way_between(
    travel: &crate::travel::Travel,
    net: &Net,
    from: crate::travel::RoadPoint,
    dir: Option<f32>,
    to: crate::travel::RoadPoint,
) -> [Option<(f32, Vec<Piece>)>; 2] {
    use std::cmp::Reverse;
    let g = &travel.graph;
    let speed = |road: u32| {
        if g.roads[road as usize].kind == crate::travel::RoadKind::Country {
            travel.country_speed
        } else {
            crate::travel::TOWN_SPEED
        }
    };
    let slot = |d: f32| usize::from(d < 0.0);
    let mut best: [Option<(f32, Vec<Piece>)>; 2] = [None, None];
    let fe = g.edges[from.edge as usize];
    let te = g.edges[to.edge as usize];
    let dirs: Vec<f32> = dir.map_or(vec![1.0, -1.0], |d| vec![d]);
    // (Straight along the same stretch, ahead.)
    if from.edge == to.edge {
        for &d in &dirs {
            if (to.s - from.s) * d >= -0.01 {
                best[slot(d)] = Some((
                    (to.s - from.s).abs() / speed(from.road),
                    vec![(from.road, from.s, to.s)],
                ));
            }
        }
    }
    // States: the end of a stretch (edge × 2 + 1 at its `b` end) come to along it.
    let n = g.edges.len() * 2;
    let mut cost = vec![f32::INFINITY; n];
    let mut came: Vec<Option<usize>> = vec![None; n];
    let mut heap = std::collections::BinaryHeap::new();
    let key = |c: f32| (c * 1000.0) as u64;
    for &d in &dirs {
        let st = from.edge as usize * 2 + usize::from(d > 0.0);
        let c = if d > 0.0 {
            fe.s1 - from.s
        } else {
            from.s - fe.s0
        }
        .max(0.0)
            / speed(fe.road);
        if c < cost[st] {
            cost[st] = c;
            heap.push(Reverse((key(c), st)));
        }
    }
    let node_of = |st: usize| {
        let e = &g.edges[st / 2];
        if st % 2 == 1 { e.b } else { e.a }
    };
    let mut ends: [Option<(f32, usize)>; 2] = [None, None];
    while let Some(Reverse((k, st))) = heap.pop() {
        let c = cost[st];
        if key(c) < k {
            continue;
        }
        let worst = (0..2)
            .map(|i| match (&best[i], ends[i]) {
                (Some(b), _) => b.0,
                (None, Some(e)) => e.0,
                _ => f32::INFINITY,
            })
            .fold(0.0f32, f32::max);
        if c >= worst {
            break;
        }
        let u = node_of(st) as usize;
        let came_by = (st / 2) as u32;
        for &e in &net.adj[u] {
            let pass = if net.jam[e as usize] { JAM_COST } else { 0.0 };
            let back = e == came_by;
            if back && !net.turn[u] && side_turn(g, net, u as u32, came_by).is_none() {
                continue;
            }
            let edge = &g.edges[e as usize];
            let extra = pass
                + match (back, net.turn[u]) {
                    (false, _) => 0.0,
                    (true, true) => TURN_COST,
                    (true, false) => SIDE_COST,
                };
            // Onto the goal's stretch: there, coming the way it runs from here.
            if e == to.edge {
                let (s_at, d) = if edge.a as usize == u {
                    (edge.s0, 1.0)
                } else {
                    (edge.s1, -1.0)
                };
                let total = c + extra + (to.s - s_at).abs() / speed(edge.road);
                let i = slot(d);
                if best[i].as_ref().is_none_or(|b| total < b.0)
                    && ends[i].is_none_or(|x| total < x.0)
                {
                    ends[i] = Some((total, st));
                }
            }
            let next = e as usize * 2 + usize::from(edge.a as usize == u);
            let nc = c + extra + edge.len() / speed(edge.road);
            if nc < cost[next] {
                cost[next] = nc;
                came[next] = Some(st);
                heap.push(Reverse((key(nc), next)));
            }
        }
    }
    // (Back from where it came onto the goal's stretch to the start.)
    for i in 0..2 {
        let Some((total, last)) = ends[i] else {
            continue;
        };
        if best[i].as_ref().is_some_and(|b| b.0 <= total) {
            continue;
        }
        let mut chain = vec![last];
        while let Some(p) = came[*chain.last().expect("one")] {
            chain.push(p);
        }
        chain.reverse();
        let mut pieces: Vec<Piece> = Vec::with_capacity(chain.len() + 2);
        let first = chain[0];
        let s_end = if first % 2 == 1 { fe.s1 } else { fe.s0 };
        pieces.push((from.road, from.s, s_end));
        for &st in &chain[1..] {
            let e = &g.edges[st / 2];
            pieces.push(if st % 2 == 1 {
                (e.road, e.s0, e.s1)
            } else {
                (e.road, e.s1, e.s0)
            });
        }
        let u = node_of(last);
        let s_at = if te.a == u { te.s0 } else { te.s1 };
        pieces.push((to.road, s_at, to.s));
        pieces.retain(|p| (p.2 - p.1).abs() > 1e-4);
        best[i] = Some((total, pieces));
    }
    best
}

/// The network's place for `kerb` (its road and `s`).
fn kerb_point(g: &crate::travel::Graph, kerb: &Kerb) -> Option<crate::travel::RoadPoint> {
    g.at_source(g.roads[kerb.road as usize].source, kerb.s)
        .filter(|p| p.road == kerb.road)
}

/// Whether a taxi coming to `kerb` going `d` (+1: as `s` rises) stands on the far side of the
/// road from the house.
fn far_side(g: &crate::travel::Graph, kerb: &Kerb, d: f32) -> bool {
    let road = &g.roads[kerb.road as usize];
    let (c, t) = (road.at(kerb.s), road.tangent(kerb.s));
    let right = [-t[1] * d, t[0] * d];
    let m = g.point(kerb.mouth);
    let house = [m[0] - kerb.axis[0], m[1] - kerb.axis[1]];
    (house[0] - c[0]) * right[0] + (house[1] - c[1]) * right[1] < 0.0
}

/// A taxi's way round town (`route`): from `way_in` to each of `kerbs` in turn and out at
/// `way_out`, as a car drives (`way_between`), the way it comes to each kerb chosen for the
/// quickest way on (`FAR_SIDE` on top for standing across the road from the house). The stretches
/// driven, in legs: one up to each kerb, then out. None: no way.
pub(crate) fn plan_way(
    travel: &crate::travel::Travel,
    net: &Net,
    way_in: [f32; 2],
    kerbs: &[Kerb],
    way_out: [f32; 2],
) -> Option<Vec<Vec<Piece>>> {
    use crate::travel::Mode;
    let g = &travel.graph;
    let entry = *g
        .near(way_in, crate::cars::ROAD_REACH, 1, Mode::Car)
        .first()?;
    let exit = *g
        .near(way_out, crate::cars::ROAD_REACH, 1, Mode::Car)
        .first()?;
    let points: Vec<crate::travel::RoadPoint> = kerbs
        .iter()
        .map(|k| kerb_point(g, k))
        .collect::<Option<_>>()?;
    // Per kerb, by the way it comes there (+1, -1): the cost so far, the way it came to the one
    // before (its slot), and the leg there.
    type Best = Option<(f32, usize, Vec<Piece>)>;
    let mut table: Vec<[Best; 2]> = Vec::with_capacity(kerbs.len());
    for (k, &to) in points.iter().enumerate() {
        let mut here: [Best; 2] = [None, None];
        let starts: Vec<(f32, usize, crate::travel::RoadPoint, Option<f32>)> = if k == 0 {
            vec![(0.0, 0, entry, None)]
        } else {
            (0..2)
                .filter_map(|i| {
                    let c = table[k - 1][i].as_ref()?.0;
                    Some((c, i, points[k - 1], Some(if i == 0 { 1.0 } else { -1.0 })))
                })
                .collect()
        };
        for (c0, i0, from, dir) in starts {
            let ways = way_between(travel, net, from, dir, to);
            for (i, way) in ways.into_iter().enumerate() {
                let Some((c, pieces)) = way else { continue };
                let d = if i == 0 { 1.0 } else { -1.0 };
                let total = c0
                    + c
                    + if far_side(g, &kerbs[k], d) {
                        FAR_SIDE
                    } else {
                        0.0
                    };
                if here[i].as_ref().is_none_or(|b| total < b.0) {
                    here[i] = Some((total, i0, pieces));
                }
            }
        }
        if here.iter().all(Option::is_none) {
            return None;
        }
        table.push(here);
    }
    // Out: from the last kerb (or straight through with none).
    let mut out: Option<(f32, usize, Vec<Piece>)> = None;
    let starts: Vec<(f32, usize, crate::travel::RoadPoint, Option<f32>)> = match table.last() {
        None => vec![(0.0, 0, entry, None)],
        Some(last) => (0..2)
            .filter_map(|i| {
                let c = last[i].as_ref()?.0;
                Some((
                    c,
                    i,
                    *points.last().expect("some"),
                    Some(if i == 0 { 1.0 } else { -1.0 }),
                ))
            })
            .collect(),
    };
    for (c0, i0, from, dir) in starts {
        for (c, pieces) in way_between(travel, net, from, dir, exit)
            .into_iter()
            .flatten()
        {
            if out.as_ref().is_none_or(|b| c0 + c < b.0) {
                out = Some((c0 + c, i0, pieces));
            }
        }
    }
    let (_, mut i, pieces) = out?;
    let mut legs = vec![pieces];
    for k in (0..table.len()).rev() {
        let (_, prev, pieces) = table[k][i].clone()?;
        legs.push(pieces);
        i = prev;
    }
    legs.reverse();
    Some(legs)
}

/// A taxi's route (`len` by `wid`) along `legs` (`plan_way`): in from off the map, standing at the
/// kerb at the end of each leg but the last (in its lane, eased in to clear the kerb by
/// `KERB_KEEP` where its lane comes nearer, slowing to a stand; a claim of its stretch held as it
/// stands, `KERB_CLAIM`), turning round where it goes back the way it came (in a dead end's
/// turning circle, `u_turn`; else nose first into a side street at a junction and backing out,
/// `side_turn`; each a claim of its own), and off the map at the end. Its claims worked out
/// (`Route::claim`); with the ticks along it where it stands at each kerb.
pub(crate) fn lay_route(
    travel: &crate::travel::Travel,
    net: &Net,
    legs: &[Vec<Piece>],
) -> Option<(crate::cars::Route, Vec<f32>)> {
    use crate::cars::{Legs, kerb_of, lane_on, pull_room};
    use crate::travel::{RoadKind, dist};
    let g = &travel.graph;
    let junctions: Vec<[f32; 2]> = g.junctions().collect();
    let mut out = Legs::default();
    let mut at_stops = Vec::new();
    // (A stand's claim: from the point where it stands, `KERB_CLAIM` past its body either way.)
    let mut stood: Option<usize> = None;
    // (Turned round into a side street: its bay, the way it goes on, where its turn out of it is
    // along the road and how much it takes, and where the turn began, for its claim.)
    let mut leaving: Option<(crate::cars::Bay, f32, f32, f32, usize)> = None;
    let last_leg = legs.len() - 1;
    for (k, leg) in legs.iter().enumerate() {
        // Its runs: split where it turns round (a stretch back along the one before).
        let mut runs: Vec<Vec<Piece>> = vec![Vec::new()];
        for &p in leg {
            let turned = runs.last().and_then(|r| r.last()).is_some_and(|q| {
                q.0 == p.0 && (q.2 - q.1) * (p.2 - p.1) < 0.0 && (q.2 - p.1).abs() < 0.01
            });
            if turned {
                runs.push(Vec::new());
            }
            runs.last_mut().expect("one").push(p);
        }
        let n_runs = runs.len();
        // (How each run but the last ends: turning round at a dead end, else in a side street.)
        let rounds: Vec<Option<(u32, Option<SideTurn>)>> = (0..n_runs)
            .map(|j| {
                let q = *runs[j].last()?;
                (j + 1 < n_runs).then(|| {
                    let (e, u) = edge_end(g, q)?;
                    Some((
                        u,
                        (!net.turn[u as usize])
                            .then(|| side_turn(g, net, u, e))
                            .flatten(),
                    ))
                })?
            })
            .collect();
        let firsts: Vec<Option<Piece>> = runs.iter().map(|r| r.first().copied()).collect();
        for (j, run) in runs.iter_mut().enumerate() {
            // (Turning round at a dead end at its end, or after one: the swerve's way off the
            // stretch.)
            let dead = |j: usize| {
                rounds
                    .get(j)
                    .copied()
                    .flatten()
                    .is_some_and(|r| r.1.is_none())
            };
            if j + 1 < n_runs && dead(j) {
                let last = run.last_mut()?;
                let road = &g.roads[last.0 as usize];
                let before = u_turn_reach(road.width, road.kind);
                let d = (last.2 - last.1).signum();
                last.2 -= d * before;
            }
            if j > 0 && dead(j - 1) {
                let first = run.first_mut()?;
                let road = &g.roads[first.0 as usize];
                let before = u_turn_reach(road.width, road.kind);
                let d = (first.2 - first.1).signum();
                first.1 += d * before;
            }
            let mut centre: Vec<[f32; 2]> = Vec::new();
            let mut along: Vec<(f32, RoadKind)> = Vec::new();
            for &(r, s0, s1) in run.iter() {
                let road = &g.roads[r as usize];
                for p in road.slice(s0, s1) {
                    if centre.last().is_some_and(|q| dist(*q, p) < 0.05) {
                        continue;
                    }
                    centre.push(p);
                    along.push((road.width, road.kind));
                }
            }
            if centre.len() < 2 {
                // (A stand right where the last one was: it stands there again.)
                continue;
            }
            let mut offs: Vec<f32> = along.iter().map(|&(w, kind)| lane_on(w, kind)).collect();
            let first_run = k == 0 && j == 0;
            // (Eased to where it stands, clear of its kerb, and out again after; its body's
            // length by the stand already there, so it stands straight along its lane: eased
            // right up to it, it stood as much as 9 degrees across a country road's.)
            let flat = TAXI_LEN / 2.0 + STAND_ROOM;
            let ease = |offs: &mut [f32], order: &mut dyn Iterator<Item = usize>| {
                let mut gone = 0.0;
                let mut was: Option<usize> = None;
                for i in order {
                    if let Some(q) = was {
                        gone += dist(centre[q], centre[i]);
                    }
                    was = Some(i);
                    if gone > flat + STAND_EASE {
                        break;
                    }
                    let (w, kind) = along[i];
                    let lane = lane_on(w, kind);
                    let off = lane.min(kerb_of(w, kind) - TAXI_WID / 2.0 - KERB_KEEP);
                    // (Two stands close along one road: whichever's nearer the kerb, so it keeps
                    // to the kerb between them; else easing out of one undid easing into the
                    // next, and it stood 30 degrees across its lane.)
                    let eased = off + (lane - off) * ((gone - flat) / STAND_EASE).clamp(0.0, 1.0);
                    offs[i] = offs[i].min(eased);
                }
            };
            if j + 1 == n_runs && k < last_leg {
                ease(&mut offs, &mut (0..centre.len()).rev());
            }
            if j == 0 && k > 0 {
                ease(&mut offs, &mut (0..centre.len()));
            }
            if first_run {
                // (In from off the map.)
                centre.insert(0, crate::cars::extend_past(centre[1], centre[0]));
                offs.insert(0, offs[0]);
                along.insert(0, along[0]);
            }
            if k == last_leg && j + 1 == n_runs {
                let n = centre.len();
                centre.push(crate::cars::extend_past(centre[n - 2], centre[n - 1]));
                offs.push(offs[n - 1]);
                along.push(along[n - 1]);
            }
            let full = crate::cars::offset_right_by(&centre, &offs);
            let mut lane = crate::travel::thin_with(
                &full,
                crate::cars::ROUTE_STEP,
                crate::cars::ROUTE_TOLERANCE,
            );
            // (On from a stand: from a point just ahead on the straight, facing along it as it
            // stands; not its own first, where it stands again but for the lane's offset worked
            // out along this way rather than the last, a hair's breadth across.)
            let from_stand = j == 0 && k > 0 && out.len() > 0;
            if from_stand && full.len() > 2 {
                if lane.get(1) != Some(&full[1]) {
                    lane.insert(1, full[1]);
                }
                lane.remove(0);
            }
            let lane = crate::cars::round_corners(&lane);
            let near_full = crate::travel::NearestPoint::new(full.clone());
            let nearest = |p: [f32; 2]| near_full.nearest(p).unwrap_or(0);
            let way = |i: usize| {
                let (a, b) = (lane[i.saturating_sub(1)], lane[(i + 1).min(lane.len() - 1)]);
                let l = dist(a, b).max(1e-6);
                [(b[0] - a[0]) / l, (b[1] - a[1]) / l]
            };
            let along_bay = |bay: &crate::cars::Bay, i: usize, d: f32| {
                let w = way(i);
                (w[0] * bay.t[0] + w[1] * bay.t[1]) * d > 0.7
            };
            // (Into a side street at its end: it leaves the lane short of its turn in.)
            let into = rounds[j].and_then(|r| r.1).and_then(|st| {
                let bay = st.bay(g)?;
                let n = lane.len();
                let (a, b) = (lane[n.saturating_sub(2)], lane[n - 1]);
                let go = (b[0] - a[0]) * bay.t[0] + (b[1] - a[1]) * bay.t[1];
                let d = if go >= 0.0 { -1.0 } else { 1.0 };
                let (turn, at) = bay.entry(d);
                Some((bay, d, turn, at))
            });
            let tops =
                crate::cars::lane_tops(&lane, travel, &junctions, |k| along[nearest(lane[k])].1);
            let mut on = false;
            for (i, &p) in lane.iter().enumerate() {
                let top = tops[i];
                if !on
                    && let Some((bay, d, at, taken, _)) = &leaving
                    && bay.near(p, *at, *taken, *d)
                    && along_bay(bay, i, *d)
                {
                    continue;
                }
                if let Some((bay, d, turn, at)) = &into
                    && !(from_stand && i == 0)
                    && bay.near(p, *at, turn.taken, *d)
                    && along_bay(bay, i, -*d)
                {
                    break;
                }
                if !on && let Some((.., from)) = leaving.take() {
                    // (Back in the lane out of the side street: its claim from where it turned.)
                    let last = out.last().unwrap_or(p);
                    out.line(last, p, false, crate::cars::JUNCTION_SPEED);
                    out.mouths.push((from, Some(out.len().saturating_sub(1))));
                }
                on = true;
                let k = i;
                let i = nearest(p);
                let (w, kind) = along[i];
                // (Less on the outside of a bend to the left, where its corners stand out.)
                let room = pull_room(w, kind, TAXI_WID) - crate::cars::bend_out(&lane, k, TAXI_LEN);
                out.push_lane(p, top, room.max(0.0), offs[i]);
            }
            // (The claim of the stand before, closed once there's way on to close it at.)
            if let Some(s) = stood.take() {
                let end = reach_along(&out, s, TAXI_LEN / 2.0 + KERB_CLAIM, true);
                let begin = reach_along(&out, s, TAXI_LEN / 2.0 + KERB_CLAIM, false);
                out.mouths.push((begin, Some(end)));
            }
            if j + 1 < n_runs {
                let from = out.len().saturating_sub(1);
                match into {
                    Some((bay, _, turn, _)) => {
                        // Nose first into the side street, and back out the way it came.
                        bay.enter(&mut out, &turn);
                        let next = firsts.get(j + 1).copied().flatten()?;
                        let road = &g.roads[next.0 as usize];
                        let (a, b) = (
                            road.at(next.1),
                            road.at(next.1 + (next.2 - next.1).signum() * 2.0),
                        );
                        let ahead = (b[0] - a[0]) * bay.t[0] + (b[1] - a[1]) * bay.t[1];
                        let d = if ahead >= 0.0 { 1.0 } else { -1.0 };
                        let (at, taken) = bay.leave(&mut out, d);
                        leaving = Some((bay, d, at, taken, from));
                    }
                    None => {
                        // Round in the turning circle, and on back the way it came.
                        let &(r, _, s1) = run.last().expect("pieces");
                        let road = &g.roads[r as usize];
                        let end_s = if s1 > road.len() / 2.0 {
                            road.len()
                        } else {
                            0.0
                        };
                        let t = road.tangent(end_s);
                        let t = if end_s > 0.0 { t } else { [-t[0], -t[1]] };
                        u_turn(&mut out, road.at(end_s), t, road.width, road.kind);
                        let to = out.len() - 1;
                        out.mouths.push((from, Some(to)));
                    }
                }
            }
        }
        if k < last_leg {
            // At the kerb: it slows to a stand at the lane's end there.
            if let Some(p) = out.pts.last_mut() {
                p.2 = 0.0;
            }
            let s = out.len().checked_sub(1)?;
            at_stops.push(s);
            stood = Some(s);
        }
    }
    if out.len() < 2 || leaving.is_some() {
        return None;
    }
    let mut route = out.timed(false, false);
    let ticks = at_stops.iter().map(|&i| route.points[i].2).collect();
    route.claim(TAXI_LEN, TAXI_WID, Some(travel));
    Some((route, ticks))
}

/// The stretch of road (`Graph::edges` index) piece `q` ends on, and the node it ends at.
fn edge_end(g: &crate::travel::Graph, q: Piece) -> Option<(u32, u32)> {
    let (r, s0, s1) = q;
    let d = (s1 - s0).signum();
    let mid = s1 - d * 0.25;
    g.edges.iter().enumerate().find_map(|(k, e)| {
        (e.road == r && e.s0 <= mid && mid <= e.s1).then(|| {
            let u = if d > 0.0 { e.b } else { e.a };
            (k as u32, u)
        })
    })
}

/// Turning round at a junction by nosing into a side street (`side_turn`): its bay's place there
/// (as a driveway's: `cars::Bay`), along the road come by.
#[derive(Debug, Clone, Copy)]
struct SideTurn {
    spot: [f32; 2],
    yaw: f32,
    axis: [f32; 2],
    road: u32,
    s: f32,
}

impl SideTurn {
    fn bay(&self, g: &crate::travel::Graph) -> Option<crate::cars::Bay> {
        let road = &g.roads[self.road as usize];
        crate::cars::Bay::new(
            self.spot,
            self.yaw,
            Some(self.axis),
            road.at(self.s),
            road,
            self.s,
        )
    }
}

/// How far into a side street a taxi turning round in it noses in, past the road come by's edge
/// and half its length (m).
const SIDE_IN: f32 = 0.5;

/// Whether a taxi come to junction node `u` along stretch `came` can turn round there, nose first
/// into a side street square to its road (that road going on past it) and backing out the way it
/// came, and where: None, it can't.
fn side_turn(g: &crate::travel::Graph, net: &Net, u: u32, came: u32) -> Option<SideTurn> {
    let e_in = &g.edges[came as usize];
    let road = &g.roads[e_in.road as usize];
    let adj = &net.adj[u as usize];
    if adj.len() < 3
        || !adj
            .iter()
            .any(|&e| e != came && g.edges[e as usize].road == e_in.road)
    {
        return None;
    }
    let s = if e_in.b == u { e_in.s1 } else { e_in.s0 };
    let at = road.at(s);
    let t = road.tangent(s);
    adj.iter().find_map(|&e| {
        let arm = &g.edges[e as usize];
        if arm.road == e_in.road {
            return None;
        }
        let side = &g.roads[arm.road as usize];
        let depth = road.width / 2.0 + TAXI_LEN / 2.0 + SIDE_IN;
        if arm.len() < depth + TAXI_LEN {
            return None;
        }
        let (s_u, sign) = if arm.a == u {
            (arm.s0, 1.0)
        } else {
            (arm.s1, -1.0)
        };
        let spot = side.at(s_u + sign * depth);
        let l = crate::travel::dist(spot, at).max(1e-3);
        let a = [(spot[0] - at[0]) / l, (spot[1] - at[1]) / l];
        if (a[0] * t[0] + a[1] * t[1]).abs() > 0.5 {
            return None;
        }
        let turn = SideTurn {
            spot,
            yaw: a[0].atan2(a[1]),
            axis: [-a[0], -a[1]],
            road: e_in.road,
            s,
        };
        turn.bay(g).map(|_| turn)
    })
}

/// The point of `legs` `reach` m along from point `from` (forward, else back), or its last (first).
fn reach_along(legs: &crate::cars::Legs, from: usize, reach: f32, forward: bool) -> usize {
    let pts = &legs.pts;
    let mut d = 0.0;
    let mut i = from;
    loop {
        let j = if forward {
            if i + 1 >= pts.len() {
                return i;
            }
            i + 1
        } else {
            if i == 0 {
                return 0;
            }
            i - 1
        };
        d += crate::travel::dist(pts[i].0, pts[j].0);
        i = j;
        if d >= reach {
            return i;
        }
    }
}

/// Turning round in the turning circle of a dead end at `end` (its middle), come to going `t` (a
/// unit vector) on a road `width` m wide of `kind`: a swerve right out of the lane onto a loop
/// round the circle's middle the far way round (left), and a swerve right back into the lane the
/// other way (`u_turn_radii`: the loop's front corners over the kerb by `TURN_OVER` at most).
fn u_turn(
    legs: &mut crate::cars::Legs,
    end: [f32; 2],
    t: [f32; 2],
    width: f32,
    kind: crate::travel::RoadKind,
) {
    let (r, lane, before) = u_turn_radii(width, kind);
    let right = [-t[1], t[0]];
    // (x along the way it comes, y to its right; the dead end's middle at 0.)
    let world = |x: f32, y: f32| {
        [
            end[0] + t[0] * x + right[0] * y,
            end[1] + t[1] * x + right[1] * y,
        ]
    };
    let xa = -before;
    let arc = |legs: &mut crate::cars::Legs, c: (f32, f32), rad: f32, a0: f32, a1: f32| {
        let n = (((a1 - a0).abs() * rad) / 0.5).ceil().max(1.0) as usize;
        for i in 0..=n {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            legs.push(
                world(c.0 + rad * a.cos(), c.1 + rad * a.sin()),
                false,
                TURN_SPEED,
            );
        }
    };
    let ya = lane + SWERVE;
    // The swerve out: round (xa, ya) from below it to where it meets the loop.
    let beta = (-ya).atan2(-xa);
    arc(legs, (xa, ya), SWERVE, -std::f32::consts::FRAC_PI_2, beta);
    // The loop: round the middle the far way, from that side to the other.
    let alpha = ya.atan2(xa);
    arc(legs, (0.0, 0.0), r, alpha, -alpha);
    // The swerve back in: round (xa, -ya) to the lane the other way.
    arc(legs, (xa, -ya), SWERVE, -beta, std::f32::consts::FRAC_PI_2);
}
