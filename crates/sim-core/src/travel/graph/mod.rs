//! The road network: its nodes, edges and the points along them.
//!
//! Split from `travel.rs` (see `mod.rs`); the index, the seeking and the paths are its own
//! submodules, so the graph's fields stay private to it.

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
    pub(crate) adj: Vec<Vec<u32>>,
    /// Per road, its edges in order along it.
    pub(crate) road_edges: Vec<Vec<u32>>,
    /// Spatial cells (`CELL` m): the road segments `(road, segment)` near each.
    cells: Vec<Vec<(u32, u32)>>,
    cols: usize,
    rows: usize,
    /// Its junctions' boxes (`traffic::boxes`): their middles and how far round them they reach
    /// (m). Cars take turns driving through them (`traffic.rs`).
    pub boxes: Vec<([f32; 2], f32)>,
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
    /// The edges it goes along (`Graph::path_by` only).
    pub edges: Vec<u32>,
    /// Cost (seconds of drive time for cars, metres for walkers).
    pub cost: f32,
    /// Its length (m).
    pub len: f32,
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
        let mut g = Graph {
            roads,
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
        g.merge_junctions();
        g.index();
        g.boxes = crate::traffic::boxes(&g);
        g
    }

    /// The roads of a town file: its `roads`; else `meta.roads`; else the middle lines of
    /// `meta.streets` (straight, along x).
    pub fn from_town(open: &OpenWorld, meta: &serde_json::Value) -> Self {
        if !open.roads.is_empty() {
            return Self::build(&open.roads);
        }
        if let Some(roads) = meta.get("roads").and_then(|r| {
            serde_json::from_value::<Vec<RoadRaw>>(r.clone())
                .ok()
                .filter(|r| !r.is_empty())
        }) {
            return Self::build(&roads);
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

    /// Where three or more stretches of road meet.
    pub fn junctions(&self) -> impl Iterator<Item = [f32; 2]> + '_ {
        self.adj
            .iter()
            .zip(&self.nodes)
            .filter(|(a, _)| a.len() >= 3)
            .map(|(_, &p)| p)
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
