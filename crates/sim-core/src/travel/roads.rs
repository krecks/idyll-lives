//! Roads and their ends: the stretches, the edges, and where they cross.

use super::*;
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
    pub(crate) fn new(raw: &RoadRaw, source: usize) -> Option<Self> {
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
    pub(super) fn project_segment(&self, i: usize, p: [f32; 2]) -> (f32, f32) {
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

    pub(super) fn walkable(&self) -> bool {
        true
    }

    pub(crate) fn drivable(&self) -> bool {
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

/// Junctions this near (m), with a stretch of road for cars between them, are one (`Graph::
/// merge_junctions`): each within the other's box (`traffic::boxes`, 6 m round its middle at
/// least) and a little more. Two side roads meeting a road a few metres apart either side are a
/// crossroads: as two junctions, each one's lights stood in the middle of the other.
pub(super) const MERGE: f32 = 8.0;

/// Two roads' middle lines this near (m) away from their ends meet there (`crossings`): two cars
/// in their lanes on them would touch.
const CROSS_NEAR: f32 = 2.0;
/// Two roads that near along no further than this (m) meet once, where they come nearest; along
/// further (one laid on the other), where they come together and where they part (`crossings`).
const CROSS_RUN: f32 = 20.0;
/// A gap this long (m along the first road) between places where two roads are that near is
/// two stretches.
const CROSS_GAP: f32 = 4.0;

/// Where two roads for cars (`Road::drivable`) cross, or their middle lines touch (`CROSS_NEAR`),
/// away from either's ends (where `Graph::build` joins them already): the two roads, how far along
/// each, and the place (between the two nearest points), in order. A crossing, or a touch along a
/// short stretch, is one place (`CROSS_RUN`); a road laid on another for longer meets it where
/// they come together and where they part. A town file's roads should meet at their ends; a
/// country road laid across or along another meets it all the same.
pub(super) fn crossings(roads: &[Road]) -> Vec<(usize, usize, f32, f32, [f32; 2])> {
    let bounds: Vec<[f32; 4]> = roads
        .iter()
        .map(|r| {
            r.pts.iter().fold(
                [
                    f32::INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::NEG_INFINITY,
                ],
                |b, p| {
                    [
                        b[0].min(p[0]),
                        b[1].min(p[1]),
                        b[2].max(p[0]),
                        b[3].max(p[1]),
                    ]
                },
            )
        })
        .collect();
    let mut out = Vec::new();
    for r in 0..roads.len() {
        for q in r + 1..roads.len() {
            let (a, b) = (&roads[r], &roads[q]);
            let (x, y) = (bounds[r], bounds[q]);
            if !a.drivable()
                || !b.drivable()
                || x[2] + CROSS_NEAR < y[0]
                || y[2] + CROSS_NEAR < x[0]
                || x[3] + CROSS_NEAR < y[1]
                || y[3] + CROSS_NEAR < x[1]
            {
                continue;
            }
            let snap = a.width.max(b.width) / 2.0 + 1.0;
            let inside = |road: &Road, s: f32| s >= snap && s <= road.len() - snap;
            // (The nearest places of each pair of segments within reach, by the stretches along `a`
            // they make.)
            let mut near: Vec<(f32, f32, f32, [f32; 2])> = Vec::new();
            for i in 0..a.pts.len() - 1 {
                for j in 0..b.pts.len() - 1 {
                    let (ta, tb, d) =
                        segments_nearest(a.pts[i], a.pts[i + 1], b.pts[j], b.pts[j + 1]);
                    if d > CROSS_NEAR {
                        continue;
                    }
                    let sa = a.cum[i] + (a.cum[i + 1] - a.cum[i]) * ta;
                    let sb = b.cum[j] + (b.cum[j + 1] - b.cum[j]) * tb;
                    if !inside(a, sa) || !inside(b, sb) {
                        continue;
                    }
                    let (pa, pb) = (a.at(sa), b.at(sb));
                    near.push((d, sa, sb, [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0]));
                }
            }
            near.sort_by(|m, n| m.1.total_cmp(&n.1).then(m.0.total_cmp(&n.0)));
            let mut start = 0;
            while start < near.len() {
                let mut end = start + 1;
                while end < near.len() && near[end].1 - near[end - 1].1 < CROSS_GAP {
                    end += 1;
                }
                let run = &near[start..end];
                let least = run.iter().map(|m| m.0).fold(f32::INFINITY, f32::min);
                if run[run.len() - 1].1 - run[0].1 <= CROSS_RUN {
                    let m = run.iter().find(|m| m.0 == least).expect("one is the least");
                    out.push((r, q, m.1, m.2, m.3));
                } else {
                    // (Where they've come together, and the last place before they part.)
                    let met = |m: &&(f32, f32, f32, [f32; 2])| m.0 <= least + 0.3;
                    let first = run.iter().find(met).expect("one is the least");
                    let last = run.iter().rev().find(met).expect("one is the least");
                    out.push((r, q, first.1, first.2, first.3));
                    out.push((r, q, last.1, last.2, last.3));
                }
                start = end;
            }
        }
    }
    out
}

/// The nearest points of segments `a0`–`a1` and `b0`–`b1`: how far along each (0 to 1), and how
/// far apart they are (m).
fn segments_nearest(a0: [f32; 2], a1: [f32; 2], b0: [f32; 2], b1: [f32; 2]) -> (f32, f32, f32) {
    let (u, v) = (
        [a1[0] - a0[0], a1[1] - a0[1]],
        [b1[0] - b0[0], b1[1] - b0[1]],
    );
    let w0 = [a0[0] - b0[0], a0[1] - b0[1]];
    let dot = |p: [f32; 2], q: [f32; 2]| p[0] * q[0] + p[1] * q[1];
    let (aa, bb, cc, dd, ee) = (dot(u, u), dot(u, v), dot(v, v), dot(u, w0), dot(v, w0));
    let den = aa * cc - bb * bb;
    let mut s = if den > 1e-9 {
        ((bb * ee - cc * dd) / den).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = if cc > 1e-9 {
        ((bb * s + ee) / cc).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if aa > 1e-9 {
        s = ((bb * t - dd) / aa).clamp(0.0, 1.0);
    }
    if cc > 1e-9 {
        t = ((bb * s + ee) / cc).clamp(0.0, 1.0);
    }
    let p = [a0[0] + u[0] * s, a0[1] + u[1] * s];
    let q = [b0[0] + v[0] * t, b0[1] + v[1] * t];
    (s, t, dist(p, q))
}
