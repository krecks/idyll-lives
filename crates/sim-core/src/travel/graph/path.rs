//! Paths over the graph: the A*, the ends and the cost.

use super::*;

impl Graph {
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

    /// `path`, setting off only towards `way.toward` and avoiding `way.avoid` (`Way`): a car turned
    /// round to go another way (`cars::Detour`).
    pub fn path_by(
        &self,
        from: RoadPoint,
        to: RoadPoint,
        mode: Mode,
        country: f32,
        way: Way,
    ) -> Option<RoadPath> {
        self.search_by(from, to, mode, country, way, |pieces| {
            let (len, cost) = self.measure(pieces, mode, country);
            let mut out = RoadPath {
                len,
                cost,
                ..Default::default()
            };
            self.each_point(pieces, |p| out.pts.push(p));
            out.edges = pieces
                .iter()
                .filter_map(|&(r, s0, s1)| self.edge_at(r, (s0 + s1) / 2.0))
                .collect();
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
        self.search_by(from, to, mode, country, Way::default(), found)
    }

    /// `search`, setting off from `from` only towards `way.toward` (a node at an end of its edge),
    /// and each edge of `way.avoid` costing that much more. A car keeps to one-way roads' way
    /// (`Graph::may_go`) setting off, on the way and coming in.
    fn search_by<R>(
        &self,
        from: RoadPoint,
        to: RoadPoint,
        mode: Mode,
        country: f32,
        way: Way,
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
        // (A car goes along a one-way road its way only, `forward` the way its points run;
        // someone on foot either way.)
        let goes = |e: &Edge, forward: bool| {
            mode == Mode::Foot || self.roads[e.road as usize].oneway.may_go(forward)
        };
        let kind = |e: &Edge| self.roads[e.road as usize].kind;
        // The same stretch of road: straight along it (the way it may set off).
        let ahead = |s: f32| match way.toward {
            Some(n) if n == fe.b => s >= from.s,
            Some(n) if n == fe.a => s <= from.s,
            _ => true,
        };
        if from.road == to.road
            && from.edge == to.edge
            && ahead(to.s)
            && (to.s == from.s || goes(&fe, to.s > from.s))
        {
            return Some(found(&[(from.road, from.s, to.s)]));
        }
        let extra = |e: u32| way.avoid.iter().find(|a| a.0 == e).map_or(0.0, |a| a.1);
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
            for (node, len, forward) in
                [(fe.a, from.s - fe.s0, false), (fe.b, fe.s1 - from.s, true)]
            {
                if way.toward.is_some_and(|n| n != node) || !goes(&fe, forward) {
                    continue;
                }
                let c = Self::cost(kind(&fe), len.max(0.0), mode, country) + extra(from.edge);
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
                for (end, len, forward) in [(te.a, to.s - te.s0, true), (te.b, te.s1 - to.s, false)]
                {
                    if end == u && goes(&te, forward) {
                        let total = gu
                            + Self::cost(kind(&te), len.max(0.0), mode, country)
                            + extra(to.edge);
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
                    if !allowed(edge) || !goes(edge, edge.a == u) {
                        continue;
                    }
                    let v = if edge.a == u { edge.b } else { edge.a };
                    let c = gu + Self::cost(kind(edge), edge.len(), mode, country) + extra(e);
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
