//! Walking the roads: sides, offsets, thinning and distances.

use super::*;
/// Where a long walk may get on and off the roads (`Travel::walk_plan`).
#[derive(Debug, Clone)]
pub struct WalkPlan {
    pub ons: Vec<RoadPoint>,
    pub offs: Vec<RoadPoint>,
}

/// How far to the right of a road's middle line people walk: on the pavement of a street (its
/// width takes in the sidewalks), on the verge of a country road, down the middle of a trail.
/// Half a metre in from its edge, 2.5 m out at most: on a 6 m lane, a bike's handlebars clear a
/// car going by (`cars::BESIDE`). Beside a country road there's open ground: half a metre out on
/// the verge.
/// How far round a point the roads are looked over for one whose carriageway it's in (m,
/// `Travel::on_carriageway`): more than half the widest road's.
pub(super) const CARRIAGEWAY_LOOK: f32 = 8.0;

pub(crate) fn side_offset(width: f32, kind: RoadKind) -> f32 {
    match kind {
        RoadKind::Trail => 0.0,
        RoadKind::Country => width / 2.0 + 0.5,
        _ => (width / 2.0 - 0.5).clamp(0.0, 2.5),
    }
}

/// `pts` moved to the walkers' side (the right, looking the way they go, as the traffic keeps:
/// `cars::offset_right_with`).
pub(super) fn walk_side(pts: &[([f32; 2], f32, RoadKind)]) -> Vec<[f32; 2]> {
    let line: Vec<[f32; 2]> = pts.iter().map(|p| p.0).collect();
    crate::cars::offset_right_with(&line, |i| side_offset(pts[i].1, pts[i].2))
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

/// The nearest of a line of points (a route's path) to points asked about: what a scan of them
/// all with `dist` gives (`min_by`: the lowest index on a tie), found through cells over them (a
/// route laid asked for every lane point's nearest twice, a scan of 1,200 points each time, and
/// was most of a big world's slowest ticks).
#[derive(Debug, Clone)]
pub(crate) struct NearestPoint {
    pts: Vec<[f32; 2]>,
    /// Where cell (0, 0) begins, and the cells' size (m).
    origin: [f32; 2],
    cell: f32,
    cols: i64,
    rows: i64,
    /// Each cell's points (indices, in order) are `items[start[c]..start[c + 1]]`.
    start: Vec<u32>,
    items: Vec<u32>,
    /// Too few points for cells to pay, or one not finite: every point scanned.
    pub(super) scan: bool,
}

impl Default for NearestPoint {
    fn default() -> Self {
        NearestPoint::new(Vec::new())
    }
}

/// Fewer points than this are scanned.
const NEAREST_SCAN: usize = 48;

impl NearestPoint {
    pub(crate) fn new(pts: Vec<[f32; 2]>) -> NearestPoint {
        let mut out = NearestPoint {
            pts,
            origin: [0.0; 2],
            cell: 1.0,
            cols: 0,
            rows: 0,
            start: Vec::new(),
            items: Vec::new(),
            scan: true,
        };
        let pts = &out.pts;
        let n = pts.len();
        if n < NEAREST_SCAN || !pts.iter().all(|p| p[0].is_finite() && p[1].is_finite()) {
            return out;
        }
        let (mut lo, mut hi) = (pts[0], pts[0]);
        let mut len = 0.0f64;
        for (i, p) in pts.iter().enumerate() {
            lo = [lo[0].min(p[0]), lo[1].min(p[1])];
            hi = [hi[0].max(p[0]), hi[1].max(p[1])];
            if i > 0 {
                len += f64::from(dist(pts[i - 1], *p));
            }
        }
        // (A few points' spacing along the line a side; fewer, larger cells where that'd be more
        // cells than a few for each point.)
        let mut cell = ((4.0 * len / n as f64) as f32).max(1.0);
        let (cols, rows) = loop {
            let cols = (((hi[0] - lo[0]) / cell) as i64).saturating_add(1);
            let rows = (((hi[1] - lo[1]) / cell) as i64).saturating_add(1);
            if cols.saturating_mul(rows) <= 4 * n as i64 + 64 {
                break (cols, rows);
            }
            cell *= 2.0;
        };
        out.origin = lo;
        out.cell = cell;
        out.cols = cols;
        out.rows = rows;
        let at: Vec<usize> = pts
            .iter()
            .map(|&p| {
                let (cx, cz) = out.cell_of(p);
                (cz.clamp(0, rows - 1) * cols + cx.clamp(0, cols - 1)) as usize
            })
            .collect();
        let mut start = vec![0u32; (cols * rows) as usize + 1];
        for &c in &at {
            start[c + 1] += 1;
        }
        for c in 0..start.len() - 1 {
            start[c + 1] += start[c];
        }
        let mut fill = start.clone();
        let mut items = vec![0u32; n];
        for (i, &c) in at.iter().enumerate() {
            items[fill[c] as usize] = i as u32;
            fill[c] += 1;
        }
        out.start = start;
        out.items = items;
        out.scan = false;
        out
    }

    fn cell_of(&self, p: [f32; 2]) -> (i64, i64) {
        (
            ((p[0] - self.origin[0]) / self.cell).floor() as i64,
            ((p[1] - self.origin[1]) / self.cell).floor() as i64,
        )
    }

    /// The index of the point nearest `p` (`dist`; the lowest of those as near). None: no points.
    pub(crate) fn nearest(&self, p: [f32; 2]) -> Option<usize> {
        if self.scan || !(p[0].is_finite() && p[1].is_finite()) {
            return (0..self.pts.len())
                .min_by(|&a, &b| dist(self.pts[a], p).total_cmp(&dist(self.pts[b], p)));
        }
        let (qx, qz) = self.cell_of(p);
        // (Rings of cells round `p`'s, out from the first with any of the grid in it to the last.)
        let first = [-qx, qx - (self.cols - 1), -qz, qz - (self.rows - 1), 0]
            .into_iter()
            .max()
            .unwrap_or(0);
        let last = [qx, self.cols - 1 - qx, qz, self.rows - 1 - qz]
            .into_iter()
            .max()
            .unwrap_or(0);
        let mut best: Option<(f32, u32)> = None;
        let visit = |cx: i64, cz: i64, best: &mut Option<(f32, u32)>| {
            let c = (cz * self.cols + cx) as usize;
            for &i in &self.items[self.start[c] as usize..self.start[c + 1] as usize] {
                let d = dist(self.pts[i as usize], p);
                if best.is_none_or(|(bd, bi)| d < bd || d == bd && i < bi) {
                    *best = Some((d, i));
                }
            }
        };
        for r in first..=last {
            // (A point `r` cells round is `r - 1` cells away at least; a margin for rounding.)
            if let Some((bd, _)) = best
                && (r - 1) as f32 * self.cell * 0.99 - 0.01 > bd
            {
                break;
            }
            for cz in (qz - r).max(0)..=(qz + r).min(self.rows - 1) {
                if (cz - qz).abs() == r {
                    for cx in (qx - r).max(0)..=(qx + r).min(self.cols - 1) {
                        visit(cx, cz, &mut best);
                    }
                } else {
                    for cx in [qx - r, qx + r] {
                        if (0..self.cols).contains(&cx) {
                            visit(cx, cz, &mut best);
                        }
                    }
                }
            }
        }
        best.map(|(_, i)| i as usize)
    }
}

pub(super) fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
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
