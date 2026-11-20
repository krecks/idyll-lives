//! The graph's index: merging junctions, the grid of stretches and the edges near a point.

use super::*;

impl Graph {
    /// Junctions of roads for cars closer than `MERGE` along a stretch between them made one
    /// (`build`): two side roads meeting a road a metre or a few apart, either side of it, are a
    /// crossroads, not two T's, each with its lights in the middle of the other. The node goes
    /// to their middle with every arm of them; the stretches between them go, the road's
    /// stretches either side meeting half way between (its geometry as it was). Not on a
    /// roundabout's ring (a road whose ends are one node: its entries stay apart).
    pub(crate) fn merge_junctions(&mut self) {
        let n = self.nodes.len();
        let car = |g: &Graph, e: &Edge| g.roads[e.road as usize].drivable();
        let ring = |g: &Graph, r: u32| {
            let pts = &g.roads[r as usize].pts;
            dist(pts[0], pts[pts.len() - 1]) < 0.5
        };
        let junction = |g: &Graph, v: usize| {
            g.adj[v]
                .iter()
                .filter(|&&e| car(g, &g.edges[e as usize]))
                .count()
                >= 3
                && g.adj[v].iter().all(|&e| !ring(g, g.edges[e as usize].road))
        };
        fn find(parent: &mut [usize], mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]];
                i = parent[i];
            }
            i
        }
        // (A short stretch from a junction back to itself: two roads' ends joined there, each
        // meeting the road a little apart.)
        let looped = |g: &Graph, e: &Edge| {
            e.a == e.b
                && car(g, e)
                && e.len() < MERGE
                && !ring(g, e.road)
                && junction(g, e.a as usize)
        };
        let mut parent: Vec<usize> = (0..n).collect();
        let mut merged = self.edges.iter().any(|e| looped(self, e));
        for e in &self.edges {
            let (a, b) = (e.a as usize, e.b as usize);
            if a != b
                && car(self, e)
                && e.len() < MERGE
                && dist(self.nodes[a], self.nodes[b]) < MERGE
                && junction(self, a)
                && junction(self, b)
            {
                let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
                if ra != rb {
                    parent[ra.max(rb)] = ra.min(rb);
                    merged = true;
                }
            }
        }
        if !merged {
            return;
        }
        // (Each group in the middle of its members.)
        let mut sum: Vec<([f32; 2], u32)> = vec![([0.0, 0.0], 0); n];
        for (v, node) in self.nodes.iter().enumerate() {
            let r = find(&mut parent, v);
            sum[r].0[0] += node[0];
            sum[r].0[1] += node[1];
            sum[r].1 += 1;
        }
        for v in 0..n {
            if parent[v] == v && sum[v].1 > 1 {
                let k = sum[v].1 as f32;
                self.nodes[v] = [sum[v].0[0] / k, sum[v].0[1] / k];
            }
        }
        // (The stretches inside a group go; along each road, the ones either side of a run of
        // them meet half way across it.)
        let inside: Vec<bool> = (0..self.edges.len())
            .map(|k| {
                let e = self.edges[k];
                looped(self, &e)
                    || e.a != e.b
                        && find(&mut parent, e.a as usize) == find(&mut parent, e.b as usize)
            })
            .collect();
        for list in &self.road_edges {
            let mut k = 0;
            while k < list.len() {
                if !inside[list[k] as usize] {
                    k += 1;
                    continue;
                }
                let first = k;
                while k < list.len() && inside[list[k] as usize] {
                    k += 1;
                }
                let (from, to) = (
                    self.edges[list[first] as usize].s0,
                    self.edges[list[k - 1] as usize].s1,
                );
                let before = first.checked_sub(1).map(|j| list[j] as usize);
                let after = list.get(k).map(|&j| j as usize);
                match (before, after) {
                    (Some(p), Some(q)) => {
                        let m = (from + to) / 2.0;
                        self.edges[p].s1 = m;
                        self.edges[q].s0 = m;
                    }
                    (Some(p), None) => self.edges[p].s1 = to,
                    (None, Some(q)) => self.edges[q].s0 = from,
                    (None, None) => {}
                }
            }
        }
        let edges: Vec<Edge> = self
            .edges
            .iter()
            .zip(&inside)
            .filter(|(_, inside)| !**inside)
            .map(|(e, _)| Edge {
                a: find(&mut parent, e.a as usize) as u32,
                b: find(&mut parent, e.b as usize) as u32,
                ..*e
            })
            .collect();
        self.edges = edges;
        self.adj = vec![Vec::new(); n];
        self.road_edges = vec![Vec::new(); self.roads.len()];
        for (k, e) in self.edges.iter().enumerate() {
            self.adj[e.a as usize].push(k as u32);
            if e.a != e.b {
                self.adj[e.b as usize].push(k as u32);
            }
            self.road_edges[e.road as usize].push(k as u32);
        }
    }

    pub(crate) fn index(&mut self) {
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

    pub(crate) fn cell(&self, v: f32, n: usize) -> usize {
        ((v.max(0.0) / CELL) as usize).min(n.saturating_sub(1))
    }

    /// The edge of road `road` that `s` is on.
    /// The edge of `road` that `s` m along it is on. None: a road too short for an edge (under
    /// half a metre end to end, as an old save or a hand-built town may have): not in the
    /// network.
    pub(crate) fn edge_at(&self, road: u32, s: f32) -> Option<u32> {
        let list = self.road_edges.get(road as usize)?;
        let i = list.partition_point(|&e| self.edges[e as usize].s1 < s);
        list.get(i.min(list.len().checked_sub(1)?)).copied()
    }
}
