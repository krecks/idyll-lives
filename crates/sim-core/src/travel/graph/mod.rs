//! The road network: its nodes, edges and the points along them.
//!
//! The index, the seeking and the paths are its own submodules, so the graph's fields stay
//! private to it.

use super::*;

mod index;
mod path;
mod seek;

/// A place on the network: `s` metres along road `road`, on edge `edge`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoadPoint {
    pub edge: u32,
    pub road: u32,
    pub s: f32,
    /// How far it is from the point it was found for (m).
    pub off: f32,
}

/// The road network (see the module notes).
#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub roads: Vec<Road>,
    pub nodes: Vec<[f32; 2]>,
    pub edges: Vec<Edge>,
    /// Edges at each node.
    pub(super) adj: Vec<Vec<u32>>,
    /// Per road, its edges in order along it.
    pub(super) road_edges: Vec<Vec<u32>>,
    /// Spatial cells (`CELL` m): the road segments `(road, segment)` near each.
    cells: Vec<Vec<(u32, u32)>>,
    cols: usize,
    rows: usize,
    /// The top speed of its fastest road for cars but a country road (`kind_speed`; a country
    /// road's comes with each search): `top_speed`.
    fast: f32,
    /// Its junctions' boxes (`traffic::boxes`): their middles and how far round them they reach
    /// (m). Cars take turns driving through them (`traffic/`).
    pub boxes: Vec<([f32; 2], f32)>,
    /// Per road, a motorway carriageway's inside (`motorway_insides`): +1, the left of the way its
    /// points run; -1, the right; 0, not a motorway.
    pub(crate) inside: Vec<i8>,
    /// Per road, a motorway carriageway's decks (`note_decks`): where it stands raised over the
    /// land or on a bridge over the water (m along it, from and to; `RoadPlan.raised`,
    /// `RoadPlan.bridge`, `RoadPlan.bridges`). Empty for every other road.
    pub(crate) decks: Vec<Vec<[f32; 2]>>,
    /// The merges (`is_merge`) whose carriageway is on a deck within `MERGE_ZONE` of them: there
    /// those on the deck and those about it count as one level (`layer_at`).
    decked_merges: Vec<[f32; 2]>,
}

/// Which level of the roads something is on (`Graph::layer_at`): the ground (every road, those
/// on foot and on bikes, cars parked), a motorway's deck (its raised stretches and its bridges,
/// over the roads and the water below), or both (about a merge on a deck: the ramp's cars and the
/// carriageway's meet there). Two things on the roads are in each other's way only where their
/// levels meet (`Layer::meets`): a car on the deck and a car or someone on foot under it never
/// wait for each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layer(u8);

impl Layer {
    pub const GROUND: Layer = Layer(1);
    pub const DECK: Layer = Layer(2);
    pub const BOTH: Layer = Layer(3);

    /// Whether the two meet: something on one is in the way of something on the other.
    pub fn meets(self, o: Layer) -> bool {
        self.0 & o.0 != 0
    }

    /// Whether it meets the ground (those on foot, on bikes and parked are on it).
    pub fn ground(self) -> bool {
        self.meets(Layer::GROUND)
    }
}

impl Default for Layer {
    fn default() -> Self {
        Layer::GROUND
    }
}

/// How far from a merge on a deck (m) the ramp and the carriageway count as both levels
/// (`Graph::layer_at`): the ramp's taper (`cars::TAPER`), its claim up the ramp and past the merge
/// (`cars::MERGE_UP`, `cars::MERGE_ON`), and a car's length to spare.
pub const MERGE_ZONE: f32 = 60.0;

/// How far a motorway carriageway's pair may be from its edge (m: `motorway.ts` `MOTORWAY.apart`).
const PAIR_APART: f32 = 24.0;

/// Each road's inside as a motorway carriageway (`Graph::inside`), as the renderer draws its lanes
/// (`motorway.ts` `insideOf`): towards its pair, the other carriageway nearest it at its middle
/// running alongside it (within `PAIR_APART` of its edge); with none, the left of the way its cars
/// go, as they keep to the right. Its lanes run out from there, its shoulder outside them.
fn motorway_insides(roads: &[Road]) -> Vec<i8> {
    let ways: Vec<usize> = (0..roads.len())
        .filter(|&i| roads[i].kind == RoadKind::Motorway)
        .collect();
    let mut out = vec![0i8; roads.len()];
    for &i in &ways {
        let a = &roads[i];
        let mid = a.len() / 2.0;
        let (p, t) = (a.at(mid), a.tangent(mid));
        let mut best: Option<(f32, i8)> = None;
        for &j in &ways {
            let b = &roads[j];
            if j == i {
                continue;
            }
            let (s, d) = b.project(p);
            if d > (a.width + b.width) / 2.0 + PAIR_APART || best.is_some_and(|x| d >= x.0) {
                continue;
            }
            let (q, u) = (b.at(s), b.tangent(s));
            if (u[0] * t[0] + u[1] * t[1]).abs() < 0.9 {
                continue;
            }
            // (The left of the way along: (t.z, -t.x).)
            let side = (q[0] - p[0]) * t[1] - (q[1] - p[1]) * t[0];
            best = Some((d, if side >= 0.0 { 1 } else { -1 }));
        }
        out[i] = best.map_or(if a.oneway == OneWay::Against { -1 } else { 1 }, |b| b.1);
    }
    out
}

/// Whether a road of kind `end`'s end meeting a road of kind `along` part way along it joins it
/// there (`Graph::build`): not where either is a motorway (grade separated: one passes over the
/// other), but a ramp's end on a motorway (where it merges into its carriageway, or leaves it).
fn level_join(end: RoadKind, along: RoadKind) -> bool {
    match (end, along) {
        (RoadKind::Ramp, RoadKind::Motorway) | (RoadKind::Motorway, RoadKind::Motorway) => true,
        (RoadKind::Motorway, _) | (_, RoadKind::Motorway) => false,
        _ => true,
    }
}

/// What `Graph::seek` shows the road segments near a point to.
trait Seeker {
    /// A segment `d` m off, its nearest point `s` m along road `road`.
    fn visit(&mut self, road: u32, s: f32, d: f32);
    /// How far off a segment may be and still count, as found so far.
    fn enough(&self) -> f32;
}

/// `Graph::nearest`'s search: the least (how far, road, how far along) of a road in the network.
struct Nearest<'a> {
    graph: &'a Graph,
    best: Option<(f32, u32, f32)>,
}

impl Seeker for Nearest<'_> {
    fn visit(&mut self, r: u32, s: f32, d: f32) {
        let better = self
            .best
            .is_none_or(|(bd, br, bs)| d < bd || (d == bd && (r < br || (r == br && s < bs))));
        // (Not a road too short for an edge: not in the network, `Graph::edge_at`.)
        if better
            && self
                .graph
                .road_edges
                .get(r as usize)
                .is_some_and(|l| !l.is_empty())
        {
            self.best = Some((d, r, s));
        }
    }

    fn enough(&self) -> f32 {
        self.best.map_or(f32::INFINITY, |(d, ..)| d)
    }
}

/// `Graph::near`'s search: each road's nearest point (the least of how far, then how far along).
/// Once `n` roads in the network are found, one further off than the `n`th nearest doesn't count.
struct Near<'a> {
    graph: &'a Graph,
    n: usize,
    best: Vec<RoadPoint>,
}

impl Seeker for Near<'_> {
    fn visit(&mut self, r: u32, s: f32, d: f32) {
        match self.best.iter_mut().find(|b| b.road == r) {
            Some(b) if d < b.off || (d == b.off && s < b.s) => {
                b.s = s;
                b.off = d;
            }
            Some(_) => {}
            None => self.best.push(RoadPoint {
                edge: 0,
                road: r,
                s,
                off: d,
            }),
        }
    }

    fn enough(&self) -> f32 {
        if self.n == 0 || self.best.len() < self.n {
            return f32::INFINITY;
        }
        // (The `n`th nearest of the roads in the network.)
        let mut offs: Vec<f32> = self
            .best
            .iter()
            .filter(|b| {
                self.graph
                    .road_edges
                    .get(b.road as usize)
                    .is_some_and(|l| !l.is_empty())
            })
            .map(|b| b.off)
            .collect();
        if offs.len() < self.n {
            return f32::INFINITY;
        }
        offs.sort_by(f32::total_cmp);
        offs[self.n - 1]
    }
}

/// What a trip along the roads looks like: the points it passes (on the middle line), and how
/// long it is.
#[derive(Debug, Clone, Default)]
pub struct RoadPath {
    /// (point, road width, kind) along the way, from the first road point to the last.
    pub pts: Vec<([f32; 2], f32, RoadKind)>,
    /// At each of its points, the road it's on (by index in the network, `Graph::roads`) and
    /// whether it goes along it the way the road's points run (a point where one stretch meets
    /// the next, the one it comes in by).
    pub roads: Vec<(u32, bool)>,
    /// The edges it goes along (`Graph::path_by` only).
    pub edges: Vec<u32>,
    /// Cost (seconds of drive time for cars, metres for walkers).
    pub cost: f32,
    /// Its length (m).
    pub len: f32,
}

impl RoadPath {
    /// The same way driven the other way round: its points, roads and edges from the last to the
    /// first (each road gone along the other way).
    pub fn reverse(&mut self) {
        self.pts.reverse();
        self.edges.reverse();
        self.roads.reverse();
        for r in &mut self.roads {
            r.1 = !r.1;
        }
    }
}

/// The way a trip along the roads sets off and the way it comes in: its first two points and its
/// last two (as `Graph::path` has them). None for one of fewer than two points.
pub type PathEnds = Option<([[f32; 2]; 2], [[f32; 2]; 2])>;

impl Graph {
    /// The network of `roads` (see the module notes); empty without any.
    pub fn build(raw: &[RoadRaw]) -> Self {
        let roads: Vec<Road> = raw
            .iter()
            .enumerate()
            .filter_map(|(i, r)| Road::new(r, i))
            .collect();
        let fast = roads
            .iter()
            .filter(|r| r.drivable() && r.kind != RoadKind::Country)
            .map(|r| kind_speed(r.kind, 0.0))
            .fold(0.0, f32::max);
        let mut g = Graph {
            roads,
            fast,
            ..Default::default()
        };
        if g.roads.is_empty() {
            return g;
        }
        // Nodes: each road's two ends, and where another road's end meets it. Joined nodes share
        // a set (union-find).
        let mut parent: Vec<usize> = Vec::new();
        let mut pos: Vec<[f32; 2]> = Vec::new();
        // (Where a road's end meets another part way along it, the place on that one's middle
        // line: the node's, so that a T whose side road begins at the road's kerb has its box,
        // lights and slow zone in the middle of the road, not off towards the side road.)
        let mut on_road: Vec<Option<[f32; 2]>> = Vec::new();
        let mut splits: Vec<Vec<(f32, usize)>> = vec![Vec::new(); g.roads.len()];
        let mut ends: Vec<[usize; 2]> = Vec::with_capacity(g.roads.len());
        for (r, road) in g.roads.iter().enumerate() {
            let a = parent.len();
            parent.push(a);
            pos.push(road.pts[0]);
            let last = *road.pts.last().expect("two points");
            // A ring: both ends one node.
            let b = if dist(road.pts[0], last) < 0.5 {
                a
            } else {
                parent.push(a + 1);
                pos.push(last);
                a + 1
            };
            splits[r].push((0.0, a));
            splits[r].push((road.len(), b));
            ends.push([a, b]);
        }
        fn find(parent: &mut [usize], mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]];
                i = parent[i];
            }
            i
        }
        fn union(parent: &mut [usize], a: usize, b: usize) {
            let (ra, rb) = (find(parent, a), find(parent, b));
            if ra != rb {
                // The lower id stays the root: the same town makes the same graph.
                let (lo, hi) = (ra.min(rb), ra.max(rb));
                parent[hi] = lo;
            }
        }
        for r in 0..g.roads.len() {
            for (k, &end) in ends[r].iter().enumerate() {
                let p = if k == 0 {
                    g.roads[r].pts[0]
                } else {
                    *g.roads[r].pts.last().expect("two points")
                };
                for q in 0..g.roads.len() {
                    if q == r {
                        continue;
                    }
                    let other = &g.roads[q];
                    let snap = other.width.max(g.roads[r].width) / 2.0 + 1.0;
                    let (s, d) = other.project(p);
                    if d > snap {
                        continue;
                    }
                    let near_end = |s: f32| s < snap || s > other.len() - snap;
                    if !near_end(s) && !level_join(g.roads[r].kind, other.kind) {
                        continue;
                    }
                    if near_end(s) {
                        let e = if s < other.len() / 2.0 {
                            ends[q][0]
                        } else {
                            ends[q][1]
                        };
                        union(&mut parent, end, e);
                    } else {
                        splits[q].push((s, end));
                        if on_road.len() < parent.len() {
                            on_road.resize(parent.len(), None);
                        }
                        on_road[end].get_or_insert(other.at(s));
                    }
                }
            }
        }
        // Where two roads for cars cross or touch away from their ends: a node on both.
        for (r, q, sr, sq, p) in crossings(&g.roads) {
            let n = parent.len();
            parent.push(n);
            pos.push(p);
            splits[r].push((sr, n));
            splits[q].push((sq, n));
        }
        // Splits along each road close together are one node.
        for list in &mut splits {
            list.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            for i in 1..list.len() {
                if list[i].0 - list[i - 1].0 < 0.5 {
                    union(&mut parent, list[i].1, list[i - 1].1);
                }
            }
        }
        // Number the sets in the order they first appear; each where a road's end meets another
        // part way along it, if one does (the first), else where its first member is.
        on_road.resize(parent.len(), None);
        let mut id = vec![u32::MAX; parent.len()];
        for i in 0..parent.len() {
            let root = find(&mut parent, i);
            if id[root] == u32::MAX {
                id[root] = g.nodes.len() as u32;
                g.nodes.push(pos[root]);
            }
            id[i] = id[root];
        }
        let mut placed = vec![false; g.nodes.len()];
        for i in 0..parent.len() {
            let n = id[i] as usize;
            if let Some(p) = on_road[i].filter(|_| !placed[n]) {
                g.nodes[n] = p;
                placed[n] = true;
            }
        }
        g.adj = vec![Vec::new(); g.nodes.len()];
        g.road_edges = vec![Vec::new(); g.roads.len()];
        for (r, list) in splits.iter().enumerate() {
            let mut prev: Option<(f32, u32)> = None;
            for &(s, n) in list {
                let n = id[n];
                if let Some((ps, pn)) = prev {
                    if s - ps < 0.5 {
                        continue;
                    }
                    let e = g.edges.len() as u32;
                    g.edges.push(Edge {
                        road: r as u32,
                        s0: ps,
                        s1: s,
                        a: pn,
                        b: n,
                    });
                    g.adj[pn as usize].push(e);
                    if pn != n {
                        g.adj[n as usize].push(e);
                    }
                    g.road_edges[r].push(e);
                }
                prev = Some((s, n));
            }
        }
        g.inside = motorway_insides(&g.roads);
        g.decks = vec![Vec::new(); g.roads.len()];
        g.merge_junctions();
        g.index();
        g.boxes = crate::traffic::boxes(&g);
        g
    }

    /// The roads of a town file: its `roads`; else `meta.roads`; else the middle lines of
    /// `meta.streets` (straight, along x).
    pub fn from_town(open: &OpenWorld, meta: &serde_json::Value) -> Self {
        if !open.roads.is_empty() {
            let mut g = Self::build(&open.roads);
            g.note_decks(meta);
            return g;
        }
        if let Some(roads) = meta.get("roads").and_then(|r| {
            serde_json::from_value::<Vec<RoadRaw>>(r.clone())
                .ok()
                .filter(|r| !r.is_empty())
        }) {
            let mut g = Self::build(&roads);
            g.note_decks(meta);
            return g;
        }
        let streets: Vec<RoadRaw> = meta
            .get("streets")
            .and_then(|s| s.as_array())
            .map(|list| {
                list.iter()
                    .filter_map(|s| {
                        let f = |k: &str| s.get(k).and_then(|v| v.as_f64()).map(|v| v as f32);
                        let (x, z, w, d) = (f("x")?, f("z")?, f("w")?, f("d")?);
                        let mid = z + d / 2.0;
                        Some(RoadRaw {
                            points: vec![[x, mid], [x + w, mid]],
                            width: d,
                            kind: RoadKind::Street,
                            area: None,
                            oneway: OneWay::Both,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self::build(&streets)
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Each motorway carriageway's decks (`decks`) from the town's `meta.roads` (the same roads as
    /// the town file's, in its order: `Road::source`), its `raised` stretches and its bridges, and
    /// the merges on a deck (`decked_merges`). The sim's own roads (`RoadRaw`) leave them out, so
    /// a town saves as it did; `meta` is saved whole.
    pub(crate) fn note_decks(&mut self, meta: &serde_json::Value) {
        let Some(list) = meta.get("roads").and_then(|r| r.as_array()) else {
            return;
        };
        let span = |v: &serde_json::Value| -> Option<[f32; 2]> {
            let a = v.as_array()?;
            let (s0, s1) = (a.first()?.as_f64()? as f32, a.get(1)?.as_f64()? as f32);
            (s0.is_finite() && s1.is_finite() && s1 > s0).then_some([s0, s1])
        };
        for (r, road) in self.roads.iter().enumerate() {
            if road.kind != RoadKind::Motorway {
                continue;
            }
            let Some(m) = list.get(road.source as usize) else {
                continue;
            };
            if m.get("kind").and_then(|k| k.as_str()) != Some("motorway") {
                continue;
            }
            let mut decks: Vec<[f32; 2]> = Vec::new();
            for key in ["raised", "bridges"] {
                if let Some(a) = m.get(key).and_then(|v| v.as_array()) {
                    decks.extend(a.iter().filter_map(span));
                }
            }
            decks.extend(m.get("bridge").and_then(span));
            decks.sort_by(|a, b| a[0].total_cmp(&b[0]));
            self.decks[r] = decks;
        }
        let decked: Vec<[f32; 2]> = self
            .merge_nodes()
            .filter(|&n| {
                self.adj[n].iter().any(|&e| {
                    let e = &self.edges[e as usize];
                    let s = if e.a as usize == n { e.s0 } else { e.s1 };
                    self.decks[e.road as usize]
                        .iter()
                        .any(|d| d[0] - MERGE_ZONE <= s && s <= d[1] + MERGE_ZONE)
                })
            })
            .map(|n| self.nodes[n])
            .collect();
        self.decked_merges = decked;
    }

    /// Whether any road stands on a deck (`decks`): every level then the ground's but where one
    /// does.
    pub fn has_decks(&self) -> bool {
        self.decks.iter().any(|d| !d.is_empty())
    }

    /// The level (`Layer`) of the place `s` m along road `road`: a motorway carriageway's deck
    /// where it stands on one; about a merge on a deck (`MERGE_ZONE`), on the carriageway or its
    /// ramp, both; else the ground.
    pub fn layer_at(&self, road: u32, s: f32) -> Layer {
        let Some(r) = self.roads.get(road as usize) else {
            return Layer::GROUND;
        };
        if !matches!(r.kind, RoadKind::Motorway | RoadKind::Ramp) {
            return Layer::GROUND;
        }
        let p = r.at(s);
        if self.decked_merges.iter().any(|&m| dist(m, p) < MERGE_ZONE) {
            return Layer::BOTH;
        }
        let decked = self
            .decks
            .get(road as usize)
            .is_some_and(|d| d.iter().any(|d| d[0] <= s && s <= d[1]));
        if decked { Layer::DECK } else { Layer::GROUND }
    }

    /// How many networks the roads make, none reaching another (0 without roads): a town whose
    /// roads all join up is one.
    pub fn networks(&self) -> usize {
        let mut seen = vec![false; self.nodes.len()];
        let mut count = 0;
        for start in 0..self.nodes.len() {
            if seen[start] || self.adj[start].is_empty() {
                continue;
            }
            count += 1;
            seen[start] = true;
            let mut stack = vec![start];
            while let Some(n) = stack.pop() {
                for &e in &self.adj[n] {
                    let edge = &self.edges[e as usize];
                    for m in [edge.a as usize, edge.b as usize] {
                        if !seen[m] {
                            seen[m] = true;
                            stack.push(m);
                        }
                    }
                }
            }
        }
        count
    }

    /// How many stretches of road meet at node `n`.
    pub(crate) fn degree(&self, n: usize) -> usize {
        self.adj.get(n).map_or(0, Vec::len)
    }

    /// Where three or more stretches of road meet, but a motorway's merges (`is_merge`): no
    /// lights, no box and no slowing down there.
    pub fn junctions(&self) -> impl Iterator<Item = [f32; 2]> + '_ {
        (0..self.nodes.len())
            .filter(|&n| self.adj[n].len() >= 3 && !self.is_merge(n))
            .map(|n| self.nodes[n])
    }

    /// Whether node `n` is where ramps join a motorway's carriageway: every stretch there a
    /// motorway's or a ramp's, a motorway's among them, and three or more. Not a junction: no box,
    /// no lights, nobody slowing for it.
    pub fn is_merge(&self, n: usize) -> bool {
        let Some(adj) = self.adj.get(n) else {
            return false;
        };
        let kind = |&e: &u32| self.roads[self.edges[e as usize].road as usize].kind;
        adj.len() >= 3
            && adj
                .iter()
                .all(|e| matches!(kind(e), RoadKind::Motorway | RoadKind::Ramp))
            && adj.iter().any(|e| kind(e) == RoadKind::Motorway)
    }

    /// The motorway's merges (`is_merge`), by node.
    pub fn merge_nodes(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.nodes.len()).filter(|&n| self.is_merge(n))
    }

    /// Whether a car may drive along stretch `edge` from its node `from` (the road's `OneWay`):
    /// from its `a` end is the way its road's points run. A stretch from a node back to itself
    /// goes the way its points run.
    pub fn may_go(&self, edge: u32, from: u32) -> bool {
        let e = &self.edges[edge as usize];
        self.roads[e.road as usize].oneway.may_go(e.a == from)
    }

    /// Whether, in each network of roads for cars (`networks`, by roads for cars only), every node
    /// a car can come onto the roads at (one on a road for cars other than the motorway and its
    /// ramps, `Road::enterable`) can be driven to from every other, one-way roads gone along
    /// their way only. True without any. A one-way road must never cut a home off.
    pub fn strongly_connected(&self) -> bool {
        let n = self.nodes.len();
        let enterable: Vec<bool> = (0..n)
            .map(|v| {
                self.adj[v]
                    .iter()
                    .any(|&e| self.roads[self.edges[e as usize].road as usize].enterable())
            })
            .collect();
        // (From `start` along roads for cars: either way (`any`), the way cars go, or the way
        // back, against it.)
        let reach = |start: usize, any: bool, back: bool| {
            let mut seen = vec![false; n];
            seen[start] = true;
            let mut stack = vec![start];
            while let Some(u) = stack.pop() {
                for &e in &self.adj[u] {
                    let edge = &self.edges[e as usize];
                    if !self.roads[edge.road as usize].drivable() {
                        continue;
                    }
                    let v = if edge.a as usize == u { edge.b } else { edge.a } as usize;
                    // (Back: from `v` to `u` must be a way cars go.)
                    let ok = any || self.may_go(e, if back { v } else { u } as u32);
                    if ok && !seen[v] {
                        seen[v] = true;
                        stack.push(v);
                    }
                }
            }
            seen
        };
        let mut done = vec![false; n];
        for start in 0..n {
            if done[start] || !enterable[start] {
                continue;
            }
            let network = reach(start, true, false);
            let (ahead, back) = (reach(start, false, false), reach(start, false, true));
            for v in 0..n {
                if network[v] {
                    done[v] = true;
                    if enterable[v] && !(ahead[v] && back[v]) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// The place `s` m along the town file's road `source` (`drives[].road/s`), if it's in the
    /// network.
    pub fn at_source(&self, source: u32, s: f32) -> Option<RoadPoint> {
        let road = self.roads.iter().position(|r| r.source == source)? as u32;
        let s = s.clamp(0.0, self.roads[road as usize].len());
        Some(RoadPoint {
            edge: self.edge_at(road, s)?,
            road,
            s,
            off: 0.0,
        })
    }

    /// The point on the road at `p`.
    pub fn point(&self, p: RoadPoint) -> [f32; 2] {
        self.roads[p.road as usize].at(p.s)
    }
}
