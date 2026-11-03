//! Getting about a big world: the road network, long trips on foot along it, how long a trip
//! takes, and the open-world fields of the town file (docs/design/open-world.md).
//!
//! **Which towns.** Only towns laid out for the open world, `normal` and `big` ones (`size`,
//! `Travel::open`), have a network, time commutes by the walk, go on outings and keep public
//! things in cells. A town or save without a size is `cosy` and plays exactly as it always did,
//! roads in its meta or not; only its cars drive the network, in a cosy town made since they
//! drive winding roads (its driveways say where they meet them: `cars::settle_all`), and nobody
//! walks it (`Travel::by_road`).
//!
//! **The network.** The town's roads (the town file's top-level `roads`; else `meta.roads`, or
//! the centre lines of `meta.streets`; none: no network) become a small graph: a node at each
//! end of every road and wherever one road's end meets another (within `width / 2 + 1` m of its
//! middle), and an edge for each stretch of road between two nodes, with its length and kind.
//! Trails are for people on foot only; country roads are where cars go fast.
//!
//! **Long walks.** A trip on foot shorter than `ROAD_TRIP_TILES` (or in a town with no network)
//! is found by A* over the tiles as ever. A longer one walks to the road (a short A* leg to a
//! point on one of the nearest stretches of road it can reach), along the roads (A* over the
//! graph, a few hundred nodes at most), and off them to where it's going (another short leg).
//! On the road they keep to the right-hand side (`walk_side`), at a waypoint every few metres
//! and at every bend (`thin`).
//!
//! **ETA.** `Travel::eta` estimates a trip's minutes on foot or by car without searching the
//! tiles: commutes, which way out of town is nearest, and how far a place is for choosing what
//! to do (`ai::falloff`).
//!
//! Nothing here is saved except the town file's fields (`OpenWorld`): the graph is built again on
//! load.

use serde::{Deserialize, Serialize};

/// Trips on foot at least this far (octile distance in tiles) go by road when there's a network.
pub const ROAD_TRIP_TILES: f32 = 96.0;
/// How many of the nearest stretches of road a trip tries to get on at (and off at).
const ENTRIES: usize = 3;
/// How far from a point a stretch of road may be to get on it there (m).
const ENTRY_REACH: f32 = 64.0;
/// The longest step between waypoints along a road (m), and how far from the road's middle line
/// a straight step between two of them may stray (m).
const THIN_STEP: f32 = 6.0;
const THIN_TOLERANCE: f32 = 0.3;
/// Walking, a detour factor over the straight line for the tile legs of an estimate.
const DETOUR: f32 = 1.25;
/// Cars: top speeds by kind of road (m/s at 1×); country roads come from content
/// (`cars.countrySpeed`).
pub const TOWN_SPEED: f32 = 6.0;
pub const COUNTRY_SPEED: f32 = 16.0;
/// Spatial cells for finding the road near a point (m).
const CELL: f32 = 16.0;

/// How big a world is (`size`; docs/design/open-world.md). A town or save without one, or with
/// one this build doesn't know, is `cosy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorldSize {
    #[default]
    Cosy,
    Normal,
    Big,
}

impl WorldSize {
    pub fn parse(size: Option<&str>) -> Self {
        match size {
            Some("normal") => WorldSize::Normal,
            Some("big") => WorldSize::Big,
            _ => WorldSize::Cosy,
        }
    }
}

/// What kind of way a road is (`RoadPlan.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoadKind {
    #[default]
    Street,
    Lane,
    /// Between areas: where cars drive fast.
    Country,
    /// Two metres wide, on foot only.
    Trail,
}

/// How someone travels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Foot,
    Car,
}

/// A road as the town file gives it (`RoadPlan`): its middle, a point about every metre (town
/// tiles), and its width with the sidewalks.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoadRaw {
    pub points: Vec<[f32; 2]>,
    pub width: f32,
    #[serde(default)]
    pub kind: RoadKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<u32>,
}

/// A way out of town (`TownExit`): a road out, or a bus stop that only people on foot use.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum ExitRaw {
    Point([f32; 2]),
    Typed {
        x: f32,
        z: f32,
        #[serde(default)]
        kind: ExitKind,
    },
}

impl ExitRaw {
    pub fn point(&self) -> [f32; 2] {
        match *self {
            ExitRaw::Point(p) => p,
            ExitRaw::Typed { x, z, .. } => [x, z],
        }
    }

    pub fn kind(&self) -> ExitKind {
        match *self {
            ExitRaw::Point(_) => ExitKind::Road,
            ExitRaw::Typed { kind, .. } => kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExitKind {
    #[default]
    Road,
    Stop,
}

/// A hamlet (area 0: the main town) or a stretch of countryside (`TownArea`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: AreaKind,
    pub x: f32,
    pub z: f32,
    pub w: f32,
    pub d: f32,
}

impl Area {
    pub fn contains(&self, p: [f32; 2]) -> bool {
        p[0] >= self.x && p[0] < self.x + self.w && p[1] >= self.z && p[1] < self.z + self.d
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AreaKind {
    #[default]
    Hamlet,
    Countryside,
}

/// Public parking for car trips between areas.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Parking {
    pub x: f32,
    pub z: f32,
    #[serde(default)]
    pub rot: f32,
    #[serde(default)]
    pub area: u32,
}

/// Run-length masks over the ground grid (`[run, value, run, value, ...]`, row-major from tile
/// (0, 0)): water, and other ground nobody walks on.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Terrain {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked: Option<Vec<u32>>,
}

impl Terrain {
    /// The tiles of a `width × depth` ground grid either mask sets (row-major).
    pub fn mask(&self, width: usize, depth: usize) -> Vec<bool> {
        let mut out = vec![false; width * depth];
        for rle in [&self.water, &self.blocked].into_iter().flatten() {
            let mut i = 0usize;
            for pair in rle.chunks(2) {
                let run = pair[0] as usize;
                let set = pair.get(1).is_some_and(|&v| v != 0);
                let end = (i + run).min(out.len());
                if set {
                    out[i..end].iter_mut().for_each(|t| *t = true);
                }
                i = end;
            }
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.water.is_none() && self.blocked.is_none()
    }
}

/// What a plot is (`plots[].kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlotKind {
    Home,
    Farm,
    Public,
    Shop,
    Countryside,
}

/// The open-world fields of a town file (docs/design/open-world.md), all optional, read along
/// with the rest of it (`LotFile::open_world`) and saved as they came (`SaveFile::open_world`).
/// Typed exits, `plots[].area/kind` and `drives[].road/s` are read where those are.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenWorld {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roads: Vec<RoadRaw>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub areas: Vec<Area>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<Terrain>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parking: Vec<Parking>,
    /// Saves only: which of `World::exits` are bus stops (indices).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stops: Vec<u32>,
    /// Saves only: each plot's area and kind (`plots[].area/kind`), by plot index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plot_areas: Vec<Option<u32>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plot_kinds: Vec<Option<PlotKind>>,
}

impl OpenWorld {
    /// How big the world is (absent: `cosy`).
    pub fn size(&self) -> WorldSize {
        WorldSize::parse(self.size.as_deref())
    }

    pub fn is_empty(&self) -> bool {
        self.size.is_none()
            && self.roads.is_empty()
            && self.areas.is_empty()
            && self.terrain.as_ref().is_none_or(Terrain::is_empty)
            && self.parking.is_empty()
            && self.stops.is_empty()
            && self.plot_areas.iter().all(Option::is_none)
            && self.plot_kinds.iter().all(Option::is_none)
    }
}

/// A road of the network: its middle line and how far along it each point is.
#[derive(Debug, Clone)]
pub struct Road {
    pub pts: Vec<[f32; 2]>,
    /// Distance along the road to each point (m).
    pub cum: Vec<f32>,
    pub width: f32,
    pub kind: RoadKind,
    pub area: Option<u32>,
    /// Its index in the town file's `roads` (`drives[].road` refers to it).
    pub source: u32,
}

impl Road {
    fn new(raw: &RoadRaw, source: usize) -> Option<Self> {
        let mut pts: Vec<[f32; 2]> = Vec::with_capacity(raw.points.len());
        for &p in &raw.points {
            if !(p[0].is_finite() && p[1].is_finite()) {
                continue;
            }
            if pts.last().is_some_and(|q| dist(*q, p) < 1e-3) {
                continue;
            }
            pts.push(p);
        }
        if pts.len() < 2 {
            return None;
        }
        let mut cum = Vec::with_capacity(pts.len());
        let mut s = 0.0;
        cum.push(0.0);
        for w in pts.windows(2) {
            s += dist(w[0], w[1]);
            cum.push(s);
        }
        Some(Self {
            pts,
            cum,
            width: if raw.width.is_finite() && raw.width > 0.0 {
                raw.width
            } else {
                6.0
            },
            kind: raw.kind,
            area: raw.area,
            source: source as u32,
        })
    }

    pub fn len(&self) -> f32 {
        *self.cum.last().unwrap_or(&0.0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() <= 0.0
    }

    /// The point `s` metres along it (clamped to its ends).
    pub fn at(&self, s: f32) -> [f32; 2] {
        let s = s.clamp(0.0, self.len());
        let i = self
            .cum
            .partition_point(|&c| c <= s)
            .clamp(1, self.pts.len() - 1);
        let (a, b) = (self.pts[i - 1], self.pts[i]);
        let span = (self.cum[i] - self.cum[i - 1]).max(1e-6);
        lerp(a, b, ((s - self.cum[i - 1]) / span).clamp(0.0, 1.0))
    }

    /// The way it runs at `s` (a unit vector, towards increasing `s`).
    pub fn tangent(&self, s: f32) -> [f32; 2] {
        let s = s.clamp(0.0, self.len());
        let i = self
            .cum
            .partition_point(|&c| c <= s)
            .clamp(1, self.pts.len() - 1);
        let (a, b) = (self.pts[i - 1], self.pts[i]);
        let d = dist(a, b).max(1e-6);
        [(b[0] - a[0]) / d, (b[1] - a[1]) / d]
    }

    /// The nearest point of segment `i` (points `i`, `i + 1`) to `p`: how far along the road it is
    /// and how far from `p`.
    fn project_segment(&self, i: usize, p: [f32; 2]) -> (f32, f32) {
        let (a, b) = (self.pts[i], self.pts[i + 1]);
        let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
        let len2 = (dx * dx + dz * dz).max(1e-9);
        let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dz) / len2).clamp(0.0, 1.0);
        let q = [a[0] + dx * t, a[1] + dz * t];
        (self.cum[i] + t * len2.sqrt(), dist(p, q))
    }

    /// The nearest point on it to `p`: how far along it is, and how far from `p`.
    pub fn project(&self, p: [f32; 2]) -> (f32, f32) {
        (0..self.pts.len() - 1)
            .map(|i| self.project_segment(i, p))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((0.0, f32::INFINITY))
    }

    /// Its middle line from `s0` to `s1` (either way round), ends included.
    pub fn slice(&self, s0: f32, s1: f32) -> Vec<[f32; 2]> {
        let (lo, hi) = (s0.min(s1), s0.max(s1));
        let mut out = vec![self.at(lo)];
        for (i, &c) in self.cum.iter().enumerate() {
            if c > lo + 1e-3 && c < hi - 1e-3 {
                out.push(self.pts[i]);
            }
        }
        out.push(self.at(hi));
        if s0 > s1 {
            out.reverse();
        }
        out
    }

    fn walkable(&self) -> bool {
        true
    }

    fn drivable(&self) -> bool {
        self.kind != RoadKind::Trail
    }
}

/// A stretch of road between two nodes: road `road` from `s0` to `s1` (s0 < s1), from node `a`
/// to node `b`.
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub road: u32,
    pub s0: f32,
    pub s1: f32,
    pub a: u32,
    pub b: u32,
}

impl Edge {
    pub fn len(&self) -> f32 {
        self.s1 - self.s0
    }

    pub fn is_empty(&self) -> bool {
        self.len() <= 0.0
    }
}

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
    adj: Vec<Vec<u32>>,
    /// Per road, its edges in order along it.
    road_edges: Vec<Vec<u32>>,
    /// Spatial cells (`CELL` m): the road segments `(road, segment)` near each.
    cells: Vec<Vec<(u32, u32)>>,
    cols: usize,
    rows: usize,
}

/// What a trip along the roads looks like: the points it passes (on the middle line), and how
/// long it is.
#[derive(Debug, Clone, Default)]
pub struct RoadPath {
    /// (point, road width, kind) along the way, from the first road point to the last.
    pub pts: Vec<([f32; 2], f32, RoadKind)>,
    /// Cost (seconds for cars, metres for walkers).
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
                    }
                }
            }
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
        // Number the sets in the order they first appear.
        let mut id = vec![u32::MAX; parent.len()];
        for i in 0..parent.len() {
            let root = find(&mut parent, i);
            if id[root] == u32::MAX {
                id[root] = g.nodes.len() as u32;
                g.nodes.push(pos[root]);
            }
            id[i] = id[root];
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
        g.index();
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
            edge: self.edge_at(road, s),
            road,
            s,
            off: 0.0,
        })
    }

    /// The point on the road at `p`.
    pub fn point(&self, p: RoadPoint) -> [f32; 2] {
        self.roads[p.road as usize].at(p.s)
    }

    fn index(&mut self) {
        let (mut max_x, mut max_z) = (0.0f32, 0.0f32);
        for r in &self.roads {
            for p in &r.pts {
                max_x = max_x.max(p[0]);
                max_z = max_z.max(p[1]);
            }
        }
        self.cols = ((max_x.max(0.0) / CELL) as usize + 2).min(1 << 12);
        self.rows = ((max_z.max(0.0) / CELL) as usize + 2).min(1 << 12);
        self.cells = vec![Vec::new(); self.cols * self.rows];
        for (r, road) in self.roads.iter().enumerate() {
            let pad = road.width / 2.0;
            for i in 0..road.pts.len() - 1 {
                let (a, b) = (road.pts[i], road.pts[i + 1]);
                let (x0, x1) = (a[0].min(b[0]) - pad, a[0].max(b[0]) + pad);
                let (z0, z1) = (a[1].min(b[1]) - pad, a[1].max(b[1]) + pad);
                for cz in self.cell(z0, self.rows)..=self.cell(z1, self.rows) {
                    for cx in self.cell(x0, self.cols)..=self.cell(x1, self.cols) {
                        self.cells[cz * self.cols + cx].push((r as u32, i as u32));
                    }
                }
            }
        }
    }

    fn cell(&self, v: f32, n: usize) -> usize {
        ((v.max(0.0) / CELL) as usize).min(n.saturating_sub(1))
    }

    /// The edge of road `road` that `s` is on.
    fn edge_at(&self, road: u32, s: f32) -> u32 {
        let list = &self.road_edges[road as usize];
        let i = list.partition_point(|&e| self.edges[e as usize].s1 < s);
        list[i.min(list.len() - 1)]
    }

    /// The nearest points on up to `n` different roads within `reach` m of `p` that `mode` may
    /// use, nearest first.
    pub fn near(&self, p: [f32; 2], reach: f32, n: usize, mode: Mode) -> Vec<RoadPoint> {
        if self.is_empty() {
            return Vec::new();
        }
        let mut best: Vec<RoadPoint> = Vec::new();
        let (x0, x1) = (
            self.cell(p[0] - reach, self.cols),
            self.cell(p[0] + reach, self.cols),
        );
        let (z0, z1) = (
            self.cell(p[1] - reach, self.rows),
            self.cell(p[1] + reach, self.rows),
        );
        for cz in z0..=z1 {
            for cx in x0..=x1 {
                for &(r, i) in &self.cells[cz * self.cols + cx] {
                    let road = &self.roads[r as usize];
                    if mode == Mode::Car && !road.drivable() || !road.walkable() {
                        continue;
                    }
                    let (s, d) = road.project_segment(i as usize, p);
                    if d > reach {
                        continue;
                    }
                    match best.iter_mut().find(|b| b.road == r) {
                        Some(b) if d < b.off || (d == b.off && s < b.s) => {
                            b.s = s;
                            b.off = d;
                        }
                        Some(_) => {}
                        None => best.push(RoadPoint {
                            edge: 0,
                            road: r,
                            s,
                            off: d,
                        }),
                    }
                }
            }
        }
        best.sort_by(|a, b| a.off.total_cmp(&b.off).then(a.road.cmp(&b.road)));
        best.truncate(n);
        for b in &mut best {
            b.edge = self.edge_at(b.road, b.s);
        }
        best
    }

    /// Cost of `len` metres of a road of `kind` for `mode` (metres on foot, seconds by car).
    fn cost(kind: RoadKind, len: f32, mode: Mode, country: f32) -> f32 {
        match mode {
            Mode::Foot => len,
            Mode::Car => {
                len / if kind == RoadKind::Country {
                    country
                } else {
                    TOWN_SPEED
                }
            }
        }
    }

    /// The cheapest way along the roads from `from` to `to` for `mode` (`country`: cars' speed on
    /// country roads, m/s), or None if there's none.
    pub fn path(
        &self,
        from: RoadPoint,
        to: RoadPoint,
        mode: Mode,
        country: f32,
    ) -> Option<RoadPath> {
        self.search(from, to, mode, country, |pieces| {
            let (len, cost) = self.measure(pieces, mode, country);
            let mut out = RoadPath {
                len,
                cost,
                ..Default::default()
            };
            self.each_point(pieces, |p| out.pts.push(p));
            out
        })
    }

    /// `path`'s cost, and the way it sets off and the way it comes in (`PathEnds`), without the
    /// points between: to weigh many ways and lay out only the one taken.
    pub fn path_ends(
        &self,
        from: RoadPoint,
        to: RoadPoint,
        mode: Mode,
        country: f32,
    ) -> Option<(f32, PathEnds)> {
        self.search(from, to, mode, country, |pieces| {
            let (_, cost) = self.measure(pieces, mode, country);
            let (mut n, mut first, mut last) = (0, [[0.0; 2]; 2], [[0.0; 2]; 2]);
            self.each_point(pieces, |(p, ..)| {
                if n < 2 {
                    first[n] = p;
                }
                last = [last[1], p];
                n += 1;
            });
            (cost, (n >= 2).then_some((first, last)))
        })
    }

    /// The points along the stretches of road `pieces`, one after another, as `path` has them
    /// (one within 0.05 m of the one before left out), each with its road's width and kind.
    fn each_point(&self, pieces: &[Piece], mut f: impl FnMut(([f32; 2], f32, RoadKind))) {
        let mut last: Option<[f32; 2]> = None;
        for &(r, s0, s1) in pieces {
            let road = &self.roads[r as usize];
            for p in road.slice(s0, s1) {
                if last.is_some_and(|q| dist(q, p) < 0.05) {
                    continue;
                }
                last = Some(p);
                f((p, road.width, road.kind));
            }
        }
    }

    /// `path`'s length and cost only (as `path` gives them), without the points along it: for
    /// an estimate (`Travel::eta`), asked for every public place in town as a resident weighs an
    /// outing.
    pub fn path_cost(
        &self,
        from: RoadPoint,
        to: RoadPoint,
        mode: Mode,
        country: f32,
    ) -> Option<(f32, f32)> {
        self.search(from, to, mode, country, |pieces| {
            self.measure(pieces, mode, country)
        })
    }

    /// The length and cost of the stretches of road `pieces`, one after another.
    fn measure(&self, pieces: &[Piece], mode: Mode, country: f32) -> (f32, f32) {
        let len = pieces.iter().map(|p| (p.2 - p.1).abs()).sum();
        let cost = pieces.iter().fold(0.0, |cost, &(r, s0, s1)| {
            cost + Self::cost(self.roads[r as usize].kind, (s1 - s0).abs(), mode, country)
        });
        (len, cost)
    }

    /// Finds the cheapest way from `from` to `to` and hands `found` the stretches of road it goes
    /// along, in order (None: no way).
    fn search<R>(
        &self,
        from: RoadPoint,
        to: RoadPoint,
        mode: Mode,
        country: f32,
        found: impl FnOnce(&[Piece]) -> R,
    ) -> Option<R> {
        use std::cmp::Reverse;
        let allowed = |e: &Edge| {
            let r = &self.roads[e.road as usize];
            match mode {
                Mode::Foot => r.walkable(),
                Mode::Car => r.drivable(),
            }
        };
        let fe = self.edges[from.edge as usize];
        let te = self.edges[to.edge as usize];
        if !allowed(&fe) || !allowed(&te) {
            return None;
        }
        let kind = |e: &Edge| self.roads[e.road as usize].kind;
        // The same stretch of road: straight along it.
        if from.road == to.road && from.edge == to.edge {
            return Some(found(&[(from.road, from.s, to.s)]));
        }
        let top = match mode {
            Mode::Foot => 1.0,
            Mode::Car => country.max(TOWN_SPEED),
        };
        let h = |n: u32| {
            let d = dist(
                self.nodes[n as usize],
                self.roads[to.road as usize].at(to.s),
            );
            match mode {
                Mode::Foot => d,
                Mode::Car => d / top,
            }
        };
        ROAD_SCRATCH.with_borrow_mut(|scratch| {
            let RoadScratch { search, pieces } = scratch;
            let stamp = search.start(self.nodes.len());
            let crate::path::Scratch {
                g,
                came,
                stamp: stamps,
                open: heap,
                ..
            } = search;
            // (An entry counts only if stamped by this search: unset, no way there yet.)
            let get = |g: &[f32], stamps: &[u32], v: usize| {
                if stamps[v] == stamp {
                    g[v]
                } else {
                    f32::INFINITY
                }
            };
            let key = |v: f32| (v * 1000.0) as u64;
            // Off the start's stretch at either end.
            for (node, len) in [(fe.a, from.s - fe.s0), (fe.b, fe.s1 - from.s)] {
                let c = Self::cost(kind(&fe), len.max(0.0), mode, country);
                if c < get(g, stamps, node as usize) {
                    g[node as usize] = c;
                    came[node as usize] = None;
                    stamps[node as usize] = stamp;
                    heap.push(Reverse((key(c + h(node)), node)));
                }
            }
            let mut best_end: Option<(f32, u32)> = None;
            while let Some(Reverse((f, u))) = heap.pop() {
                if best_end.is_some_and(|(b, _)| (f as f32) / 1000.0 >= b) {
                    break;
                }
                let gu = g[u as usize];
                // Onto the goal's stretch from here.
                for (end, len) in [(te.a, to.s - te.s0), (te.b, te.s1 - to.s)] {
                    if end == u {
                        let total = gu + Self::cost(kind(&te), len.max(0.0), mode, country);
                        if best_end.is_none_or(|(b, _)| total < b) {
                            best_end = Some((total, u));
                        }
                    }
                }
                for &e in &self.adj[u as usize] {
                    if e == from.edge || e == to.edge {
                        continue;
                    }
                    let edge = &self.edges[e as usize];
                    if !allowed(edge) {
                        continue;
                    }
                    let v = if edge.a == u { edge.b } else { edge.a };
                    let c = gu + Self::cost(kind(edge), edge.len(), mode, country);
                    if c < get(g, stamps, v as usize) {
                        g[v as usize] = c;
                        came[v as usize] = Some(e);
                        stamps[v as usize] = stamp;
                        heap.push(Reverse((key(c + h(v)), v)));
                    }
                }
            }
            heap.clear();
            let (_, last) = best_end?;
            // Back from the last node to the first, by edge, then the right way round.
            pieces.clear();
            pieces.push((to.road, if last == te.a { te.s0 } else { te.s1 }, to.s));
            let mut at = last;
            while let Some(e) = came[at as usize] {
                let edge = &self.edges[e as usize];
                let prev = if edge.a == at { edge.b } else { edge.a };
                pieces.push(if edge.a == prev {
                    (edge.road, edge.s0, edge.s1)
                } else {
                    (edge.road, edge.s1, edge.s0)
                });
                at = prev;
            }
            pieces.push((from.road, from.s, if at == fe.a { fe.s0 } else { fe.s1 }));
            pieces.reverse();
            Some(found(pieces))
        })
    }
}

/// A stretch of a road: the road, and from and to how far along it (m).
type Piece = (u32, f32, f32);

/// Road searches' state kept between them (`Graph::search`): a big world's network has
/// thousands of nodes, too many to allocate for every estimate.
#[derive(Default)]
struct RoadScratch {
    search: crate::path::Scratch<f32, Option<u32>, (u64, u32)>,
    /// The stretches of road the way found goes along.
    pieces: Vec<Piece>,
}

thread_local! {
    static ROAD_SCRATCH: std::cell::RefCell<RoadScratch> = std::cell::RefCell::default();
}

/// Everything the sim knows about getting about (`World::travel`).
#[derive(Debug, Clone, Default)]
pub struct Travel {
    /// The town file's open-world fields, as they came (saved).
    pub file: OpenWorld,
    pub graph: Graph,
    /// Per exit (`World::exits`): a bus stop, for people on foot only.
    pub stops: Vec<bool>,
    /// How big the world is (`OpenWorld::size`).
    pub size: WorldSize,
    /// The town is laid out for the open world (its `size` is `normal` or `big`): it has a road
    /// network, commutes are timed by the walk (`commuteMinutes` is only the part off the map),
    /// residents go on outings. Cosy towns, and towns and saves without a size, play as before
    /// (no network, even with roads in their meta; but for their cars in towns made since cars
    /// drive winding roads, `cars::settle_all`).
    pub open: bool,
    /// Cars' top speed on country roads (m/s; content `cars.countrySpeed`).
    pub country_speed: f32,
    /// The tiles the town file's `terrain` masks (water, rock): blocked under everything, so a
    /// tile freed of what stood on it stays blocked (`World::set_travel`). Sorted tile indices.
    pub base_blocked: Vec<usize>,
    /// By plot, for the public places: their door and where it gets on the roads on foot and by
    /// car (`entry`), kept as each tick starts (`note_doors`): a resident weighing an outing asks
    /// how long the way to every one is. Not saved.
    doors: Vec<Option<Door>>,
}

/// A public place's door (its bits) and where it gets on the roads on foot and by car
/// (`Travel::doors`).
type Door = ([u32; 2], [Option<RoadPoint>; 2]);

impl Travel {
    pub fn new(file: OpenWorld, meta: &serde_json::Value, exits: usize) -> Self {
        let size = file.size();
        let open = size != WorldSize::Cosy;
        let graph = if open {
            Graph::from_town(&file, meta)
        } else {
            Graph::default()
        };
        let mut stops = vec![false; exits];
        for &i in &file.stops {
            if let Some(s) = stops.get_mut(i as usize) {
                *s = true;
            }
        }
        Self {
            file,
            graph,
            stops,
            size,
            open,
            country_speed: COUNTRY_SPEED,
            base_blocked: Vec::new(),
            doors: Vec::new(),
        }
    }

    /// Where a trip from or to `p` gets on the roads for `mode` (the nearest road within
    /// `ENTRY_REACH`), as `eta` finds it.
    pub fn entry(&self, p: [f32; 2], mode: Mode) -> Option<RoadPoint> {
        self.graph.near(p, ENTRY_REACH, 1, mode).first().copied()
    }

    /// Notes the public places' doors and where they get on the roads (`doors`), again only for
    /// a door that moved.
    pub(crate) fn note_doors(&mut self, plots: &[crate::world::Plot]) {
        if !self.open {
            return;
        }
        self.doors.resize(plots.len(), None);
        for (i, p) in plots.iter().enumerate() {
            let door = p.arrival_point();
            let bits = door.map(f32::to_bits);
            if !p.public {
                self.doors[i] = None;
            } else if self.doors[i].is_none_or(|(at, _)| at != bits) {
                let entries = [self.entry(door, Mode::Foot), self.entry(door, Mode::Car)];
                self.doors[i] = Some((bits, entries));
            }
        }
    }

    /// `entry` for the door `door` of public place `plot`, as noted (`note_doors`), or found.
    /// (The sim tests check it against `entry`: `common::check_lookups`.)
    pub fn door_entry(&self, plot: u32, door: [f32; 2], mode: Mode) -> Option<RoadPoint> {
        match self.doors.get(plot as usize) {
            Some(Some((at, entries))) if *at == door.map(f32::to_bits) => entries[mode as usize],
            _ => self.entry(door, mode),
        }
    }

    /// The most residents the town grows to (`World::population_cap`).
    pub fn population_cap(&self) -> usize {
        match self.size {
            WorldSize::Big => crate::world::MAX_SIMS,
            WorldSize::Cosy | WorldSize::Normal => crate::world::TOWN_SIMS,
        }
    }

    /// Whether tile `i` (`Lot::tile_index`) is blocked by the land itself (`base_blocked`).
    pub fn ground_blocked(&self, i: usize) -> bool {
        self.base_blocked.binary_search(&i).is_ok()
    }

    /// Whether exit `i` is a bus stop.
    pub fn is_stop(&self, i: usize) -> bool {
        self.stops.get(i).copied().unwrap_or(false)
    }

    /// The area a point is in, if any.
    pub fn area_at(&self, p: [f32; 2]) -> Option<u32> {
        self.file
            .areas
            .iter()
            .position(|a| a.contains(p))
            .map(|i| i as u32)
    }

    /// Whether a trip from `a` to `b` on foot goes by road.
    pub fn by_road(&self, a: [f32; 2], b: [f32; 2]) -> bool {
        self.open && !self.graph.is_empty() && octile(a, b) >= ROAD_TRIP_TILES
    }

    /// About how many minutes a trip from `from` to `to` takes: on foot at `walk` m a minute,
    /// or by car. No search of the tiles: the roads for long trips, the straight line (with a
    /// detour) for the rest.
    pub fn eta(&self, from: [f32; 2], to: [f32; 2], mode: Mode, walk: f32) -> f32 {
        self.eta_via(
            from,
            to,
            mode,
            walk,
            || self.entry(from, mode),
            || self.entry(to, mode),
        )
    }

    /// `eta`, with where the trip gets on and off the roads (`entry` of `from` and of `to`, asked
    /// for only if it goes by road) found by the caller: the same for many trips from one place.
    pub fn eta_via(
        &self,
        from: [f32; 2],
        to: [f32; 2],
        mode: Mode,
        walk: f32,
        on: impl FnOnce() -> Option<RoadPoint>,
        off: impl FnOnce() -> Option<RoadPoint>,
    ) -> f32 {
        let walk = walk.max(1e-3);
        match mode {
            Mode::Foot => {
                if !self.by_road(from, to) {
                    return octile(from, to) * DETOUR / walk;
                }
                self.road_estimate(on, off, Mode::Foot)
                    .map_or(octile(from, to) * DETOUR, |(m, _)| m)
                    / walk
            }
            Mode::Car => {
                let straight = dist(from, to) / TOWN_SPEED / 60.0;
                self.road_estimate(on, off, Mode::Car)
                    .map_or(straight, |(_, s)| s / 60.0)
            }
        }
    }

    /// At least as many minutes as `eta` says the trip takes: the straight line at the fastest
    /// way along it (no search at all). For finding the quickest of several without working
    /// out every one (`world::quickest_exit`).
    pub fn eta_at_least(&self, from: [f32; 2], to: [f32; 2], mode: Mode, walk: f32) -> f32 {
        let d = dist(from, to);
        match mode {
            Mode::Foot => d / walk.max(1e-3),
            Mode::Car => d / TOWN_SPEED.max(self.country_speed) / 60.0,
        }
    }

    /// Along the roads from where a trip gets on them (`on`) to where it gets off: the metres on foot (tile legs with a detour), and the
    /// seconds by car (`mode`).
    fn road_estimate(
        &self,
        on: impl FnOnce() -> Option<RoadPoint>,
        off: impl FnOnce() -> Option<RoadPoint>,
        mode: Mode,
    ) -> Option<(f32, f32)> {
        let a = on()?;
        let b = off()?;
        let (len, cost) = self.graph.path_cost(a, b, mode, self.country_speed)?;
        let legs = (a.off + b.off) * DETOUR;
        Some((len + legs, cost + legs / TOWN_SPEED.min(1.5)))
    }

    /// Where a trip on foot from `pos` to `target` gets on and off the roads, and the way along
    /// them in between (on the walkers' side, thinned): `ons` and `offs` are the candidates to
    /// try in turn (the caller finds the tile legs). None: no network, or no way along it.
    pub fn walk_plan(&self, pos: [f32; 2], target: [f32; 2]) -> Option<WalkPlan> {
        let ons = self.graph.near(pos, ENTRY_REACH, ENTRIES, Mode::Foot);
        let offs = self.graph.near(target, ENTRY_REACH, ENTRIES, Mode::Foot);
        if ons.is_empty() || offs.is_empty() {
            return None;
        }
        Some(WalkPlan { ons, offs })
    }

    /// The way along the roads from `on` to `off` on foot, as waypoints.
    pub fn walk_between(&self, on: RoadPoint, off: RoadPoint) -> Option<(Vec<[f32; 2]>, f32)> {
        let path = self.graph.path(on, off, Mode::Foot, self.country_speed)?;
        Some((thin(&walk_side(&path.pts)), path.len))
    }
}

/// Where a long walk may get on and off the roads (`Travel::walk_plan`).
#[derive(Debug, Clone)]
pub struct WalkPlan {
    pub ons: Vec<RoadPoint>,
    pub offs: Vec<RoadPoint>,
}

/// How far to the right of a road's middle line people walk: on the pavement of a street (its
/// width takes in the sidewalks), on the verge of a country road, down the middle of a trail.
/// Half a metre in from its edge, 2.5 m out at most: on a 6 m lane, a bike's handlebars clear a
/// car or bus going by (`cars::BESIDE`). Beside a country road there's open ground: half a
/// metre out on the verge, clear of a bus pulled in at a stop by its edge (`bus::bay_out`).
pub(crate) fn side_offset(width: f32, kind: RoadKind) -> f32 {
    match kind {
        RoadKind::Trail => 0.0,
        RoadKind::Country => width / 2.0 + 0.5,
        _ => (width / 2.0 - 0.5).clamp(0.0, 2.5),
    }
}

/// `pts` moved to the walkers' side (the right, looking the way they go).
fn walk_side(pts: &[([f32; 2], f32, RoadKind)]) -> Vec<[f32; 2]> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let (a, b) = (pts[i.saturating_sub(1)].0, pts[(i + 1).min(n - 1)].0);
            let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
            let d = dx.hypot(dz).max(1e-6);
            let off = side_offset(pts[i].1, pts[i].2);
            // Right of the way (dx, dz): (-dz, dx) in town coordinates (x right, z down the map).
            [pts[i].0[0] - dz / d * off, pts[i].0[1] + dx / d * off]
        })
        .collect()
}

/// `pts` moved `off` m to the right (looking the way they run).
pub fn offset_right(pts: &[[f32; 2]], off: f32) -> Vec<[f32; 2]> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let (a, b) = (pts[i.saturating_sub(1)], pts[(i + 1).min(n - 1)]);
            let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
            let d = dx.hypot(dz).max(1e-6);
            [pts[i][0] - dz / d * off, pts[i][1] + dx / d * off]
        })
        .collect()
}

/// A line of points about a metre apart, cut down to one every `THIN_STEP` m at most and at
/// every bend (where a straight step would stray more than `THIN_TOLERANCE` from it).
pub fn thin(pts: &[[f32; 2]]) -> Vec<[f32; 2]> {
    thin_with(pts, THIN_STEP, THIN_TOLERANCE)
}

/// `thin` with its own longest step and tolerance (m).
pub fn thin_with(pts: &[[f32; 2]], step: f32, tolerance: f32) -> Vec<[f32; 2]> {
    if pts.len() <= 2 {
        return pts.to_vec();
    }
    let mut out = vec![pts[0]];
    let mut from = 0;
    let mut i = 1;
    while i < pts.len() {
        let a = pts[from];
        let b = pts[i];
        let strays = (from + 1..i).any(|k| line_distance(pts[k], a, b) > tolerance);
        if strays || dist(a, b) > step {
            from = (i - 1).max(from + 1);
            out.push(pts[from]);
            i = from + 1;
            continue;
        }
        i += 1;
    }
    let last = *pts.last().expect("points");
    if out.last().is_none_or(|p| dist(*p, last) > 1e-3) {
        out.push(last);
    }
    out
}

/// Distance from `p` to the segment `a`–`b`.
pub fn line_distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
    let len2 = dx * dx + dz * dz;
    if len2 < 1e-9 {
        return dist(p, a);
    }
    let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dz) / len2).clamp(0.0, 1.0);
    dist(p, [a[0] + dx * t, a[1] + dz * t])
}

pub fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// Octile distance (tiles): diagonal steps where both ways differ.
pub fn octile(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dz) = ((a[0] - b[0]).abs(), (a[1] - b[1]).abs());
    dx.max(dz) + (std::f32::consts::SQRT_2 - 1.0) * dx.min(dz)
}

/// Length of a line of waypoints from `from`.
pub fn length(from: [f32; 2], pts: &[[f32; 2]]) -> f32 {
    let mut at = from;
    let mut len = 0.0;
    for &p in pts {
        len += dist(at, p);
        at = p;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    fn road(points: &[[f32; 2]], width: f32, kind: RoadKind) -> RoadRaw {
        RoadRaw {
            points: points.to_vec(),
            width,
            kind,
            area: None,
        }
    }

    /// A line from `a` to `b` with a point every metre.
    fn line(a: [f32; 2], b: [f32; 2]) -> Vec<[f32; 2]> {
        let n = dist(a, b).ceil() as usize;
        (0..=n).map(|i| lerp(a, b, i as f32 / n as f32)).collect()
    }

    #[test]
    fn ends_and_joins_become_nodes() {
        // A street along x, a lane from its middle going +z (ending 2 m off its middle line,
        // within reach of its edge), and a trail on from the lane's far end.
        let g = Graph::build(&[
            road(&line([0.0, 10.0], [100.0, 10.0]), 6.0, RoadKind::Street),
            road(&line([50.0, 12.0], [50.0, 60.0]), 5.0, RoadKind::Lane),
            road(&line([50.0, 60.0], [90.0, 90.0]), 2.0, RoadKind::Trail),
        ]);
        // Street ends 2, the T (the lane's near end), the lane's far end shared with the trail,
        // the trail's far end.
        assert_eq!(g.nodes.len(), 5, "{:?}", g.nodes);
        // The street in two at the T, the lane, the trail.
        assert_eq!(g.edges.len(), 4);
        let degree: Vec<usize> = g.adj.iter().map(Vec::len).collect();
        assert_eq!(degree.iter().filter(|&&d| d == 3).count(), 1, "{degree:?}");
        assert_eq!(g.networks(), 1);
        // A lane ending 5 m short of the street is a network of its own.
        let apart = Graph::build(&[
            road(&line([0.0, 10.0], [100.0, 10.0]), 6.0, RoadKind::Street),
            road(&line([50.0, 18.0], [50.0, 60.0]), 6.0, RoadKind::Lane),
        ]);
        assert_eq!(apart.networks(), 2);
        assert_eq!(Graph::default().networks(), 0);
    }

    #[test]
    fn a_path_goes_round_by_the_roads_and_cars_keep_off_trails() {
        let g = Graph::build(&[
            road(&line([0.0, 10.0], [100.0, 10.0]), 6.0, RoadKind::Street),
            road(&line([100.0, 10.0], [100.0, 100.0]), 6.0, RoadKind::Country),
            road(&line([0.0, 10.0], [0.0, 100.0]), 2.0, RoadKind::Trail),
            road(&line([0.0, 100.0], [100.0, 100.0]), 6.0, RoadKind::Street),
        ]);
        let from = g.near([2.0, 60.0], 10.0, 1, Mode::Foot)[0];
        let to = g.near([50.0, 100.0], 10.0, 1, Mode::Foot)[0];
        let walk = g.path(from, to, Mode::Foot, COUNTRY_SPEED).expect("a way");
        // Down the trail and along the far street: 40 + 50.
        assert!((walk.len - 90.0).abs() < 1.0, "{}", walk.len);
        // By car: no trail, so round by the near street and the country road.
        let from = g.near([20.0, 12.0], 10.0, 1, Mode::Car)[0];
        let drive = g.path(from, to, Mode::Car, COUNTRY_SPEED).expect("a drive");
        assert!(
            (drive.len - (80.0 + 90.0 + 50.0)).abs() < 1.0,
            "{}",
            drive.len
        );
        assert!(g.near([1.0, 60.0], 3.0, 1, Mode::Car).is_empty());
    }

    /// `path_cost` is `path`'s length and cost, to the bit, from and to anywhere on the roads
    /// (a search after another reuses the last one's state, `RoadScratch`).
    #[test]
    fn a_paths_cost_alone_is_the_paths() {
        let g = Graph::build(&[
            road(&line([0.0, 10.0], [100.0, 10.0]), 6.0, RoadKind::Street),
            road(&line([100.0, 10.0], [100.0, 100.0]), 6.0, RoadKind::Country),
            road(&line([0.0, 10.0], [0.0, 100.0]), 2.0, RoadKind::Trail),
            road(&line([0.0, 100.0], [100.0, 100.0]), 6.0, RoadKind::Street),
            road(&line([50.0, 10.0], [50.0, 100.0]), 4.0, RoadKind::Lane),
        ]);
        let points: Vec<[f32; 2]> = (0..12)
            .flat_map(|i| (0..12).map(move |j| [i as f32 * 9.0 + 1.0, j as f32 * 9.0 + 1.0]))
            .collect();
        let mut compared = 0;
        for mode in [Mode::Foot, Mode::Car] {
            for &a in &points {
                for &b in points.iter().step_by(7) {
                    let (Some(&from), Some(&to)) = (
                        g.near(a, 30.0, 1, mode).first(),
                        g.near(b, 30.0, 1, mode).first(),
                    ) else {
                        continue;
                    };
                    let path = g.path(from, to, mode, COUNTRY_SPEED);
                    let cost = g.path_cost(from, to, mode, COUNTRY_SPEED);
                    assert_eq!(
                        path.map(|p| (p.len.to_bits(), p.cost.to_bits())),
                        cost.map(|(l, c)| (l.to_bits(), c.to_bits()))
                    );
                    compared += 1;
                }
            }
        }
        assert!(compared > 100);
    }

    #[test]
    fn thinning_keeps_bends() {
        let mut pts = line([0.0, 0.0], [20.0, 0.0]);
        pts.extend(line([20.0, 0.0], [20.0, 20.0]).into_iter().skip(1));
        let t = thin(&pts);
        assert!(t.contains(&[20.0, 0.0]), "{t:?}");
        assert!(t.len() <= 10, "{t:?}");
        assert!(t.windows(2).all(|w| dist(w[0], w[1]) <= THIN_STEP + 1e-3));
    }

    #[test]
    fn rle_masks() {
        let t = Terrain {
            water: Some(vec![3, 0, 2, 1, 10, 0]),
            blocked: Some(vec![14, 0, 1, 1]),
        };
        let m = t.mask(5, 3);
        let set: Vec<usize> = (0..15).filter(|&i| m[i]).collect();
        assert_eq!(set, [3, 4, 14]);
    }
}
