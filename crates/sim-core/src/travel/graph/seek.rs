//! Seeking: the nearest stretch, and those within reach.

use super::*;

impl Graph {
    /// Whether network road `road` comes within `reach` of `p` somewhere else than about `s` along
    /// it (more than `apart` further on or back): a road come back over itself, a hairpin's way
    /// back beside its way out (`near` gives each road's nearest point only: there, `s` itself).
    pub fn passes_again(&self, road: u32, s: f32, p: [f32; 2], reach: f32, apart: f32) -> bool {
        if self.is_empty() {
            return false;
        }
        let (x0, x1) = (
            self.cell(p[0] - reach, self.cols),
            self.cell(p[0] + reach, self.cols),
        );
        let (z0, z1) = (
            self.cell(p[1] - reach, self.rows),
            self.cell(p[1] + reach, self.rows),
        );
        (z0..=z1).any(|cz| {
            (x0..=x1).any(|cx| {
                self.cells[cz * self.cols + cx]
                    .iter()
                    .filter(|&&(r, _)| r == road)
                    .any(|&(r, i)| {
                        let (at, d) = self.roads[r as usize].project_segment(i as usize, p);
                        d <= reach && (at - s).abs() > apart
                    })
            })
        })
    }

    /// The nearest points on up to `n` different roads within `reach` m of `p` that `mode` may
    /// use, nearest first.
    pub fn near(&self, p: [f32; 2], reach: f32, n: usize, mode: Mode) -> Vec<RoadPoint> {
        if n == 1 && p.iter().all(|v| v.is_finite()) {
            return self.nearest(p, reach, mode).into_iter().collect();
        }
        let mut near = Near {
            graph: self,
            n,
            best: Vec::new(),
        };
        self.seek(p, reach, mode, &mut near);
        let mut best = near.best;
        // (Not a road too short for an edge: not in the network.)
        best.retain_mut(|b| self.edge_at(b.road, b.s).map(|e| b.edge = e).is_some());
        best.sort_by(|a, b| a.off.total_cmp(&b.off).then(a.road.cmp(&b.road)));
        best.truncate(n);
        best
    }

    /// `near`'s nearest point (`n` 1), without gathering every road's: the least of (how far,
    /// road, how far along), as `near`'s sort ends up with.
    pub fn nearest(&self, p: [f32; 2], reach: f32, mode: Mode) -> Option<RoadPoint> {
        // (Not a point: `near`'s way, whatever that makes of it.)
        if !p.iter().all(|v| v.is_finite()) {
            return self.near(p, reach, 1, mode).first().copied();
        }
        let mut nearest = Nearest {
            graph: self,
            best: None,
        };
        self.seek(p, reach, mode, &mut nearest);
        let (off, road, s) = nearest.best?;
        Some(RoadPoint {
            edge: self.edge_at(road, s)?,
            road,
            s,
            off,
        })
    }

    /// Shows `seeker` the road segments within `reach` m of `p` that `mode` may use
    /// (`Seeker::visit`), the cells nearest `p` first, a ring of cells at a time, until no
    /// segment further out can count (`Seeker::enough`).
    ///
    /// A segment's nearest point to `p` is in one of the cells it's listed in (`index`), and one
    /// `d` m off in a cell `d` m off at most; so cells further than `reach`, or than `enough`,
    /// from `p` (and a metre more, for rounding) aren't looked in, and once a ring's next is that
    /// far the search ends. A segment in several cells comes up once from each. Segments surely
    /// out of either (by the square of the distance) are passed over before the square root.
    fn seek<S: Seeker>(&self, p: [f32; 2], reach: f32, mode: Mode, seeker: &mut S) {
        if self.is_empty() {
            return;
        }
        let (x0, x1) = (
            self.cell(p[0] - reach, self.cols),
            self.cell(p[0] + reach, self.cols),
        );
        let (z0, z1) = (
            self.cell(p[1] - reach, self.rows),
            self.cell(p[1] + reach, self.rows),
        );
        let (px, pz) = (
            self.cell(p[0], self.cols).clamp(x0, x1),
            self.cell(p[1], self.rows).clamp(z0, z1),
        );
        // How far `v` is outside cell `c` of `n` along an axis (the first and last reach on out:
        // `cell` clamps).
        let gap = |v: f32, c: usize, n: usize| {
            let lo = if c == 0 {
                f32::NEG_INFINITY
            } else {
                c as f32 * CELL
            };
            let hi = if c + 1 >= n {
                f32::INFINITY
            } else {
                (c + 1) as f32 * CELL
            };
            (lo - v).max(v - hi).max(0.0)
        };
        let look = |cx: usize, cz: usize, limit: f32, seeker: &mut S| {
            let (gx, gz) = (gap(p[0], cx, self.cols), gap(p[1], cz, self.rows));
            if gx * gx + gz * gz > (limit + 1.0) * (limit + 1.0) {
                return;
            }
            let far2 = limit * limit * 1.001 + 1e-3;
            for &(r, i) in &self.cells[cz * self.cols + cx] {
                let road = &self.roads[r as usize];
                let allowed = match mode {
                    Mode::Foot => road.walkable(),
                    Mode::Car => road.drivable(),
                };
                if !allowed {
                    continue;
                }
                // (`Road::project_segment`, step by step.)
                let i = i as usize;
                let (a, b) = (road.pts[i], road.pts[i + 1]);
                let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
                let len2 = (dx * dx + dz * dz).max(1e-9);
                let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dz) / len2).clamp(0.0, 1.0);
                let q = [a[0] + dx * t, a[1] + dz * t];
                let (ex, ez) = (p[0] - q[0], p[1] - q[1]);
                if ex * ex + ez * ez > far2 {
                    continue;
                }
                let d = dist(p, q);
                if d > reach {
                    continue;
                }
                seeker.visit(r, road.cum[i] + t * len2.sqrt(), d);
            }
        };
        let rings = (px - x0).max(x1 - px).max(pz - z0).max(z1 - pz);
        for k in 0..=rings {
            let limit = reach.min(seeker.enough());
            // (The cells of ring `k` are `k - 1` cells off at least.)
            if k > 0 && (k - 1) as f32 * CELL > limit + 1.0 {
                break;
            }
            // (Ring `k`: the cells `k` from `p`'s, along a row or a column or both.)
            for cz in pz.saturating_sub(k).max(z0)..=(pz + k).min(z1) {
                if cz + k == pz || cz == pz + k {
                    for cx in px.saturating_sub(k).max(x0)..=(px + k).min(x1) {
                        look(cx, cz, limit, seeker);
                    }
                } else {
                    if px >= x0 + k {
                        look(px - k, cz, limit, seeker);
                    }
                    if k > 0 && px + k <= x1 {
                        look(px + k, cz, limit, seeker);
                    }
                }
            }
        }
    }

    /// Cost of `len` metres of a road of `kind` for `mode` (metres on foot, seconds of drive time by
    /// car).
    pub(super) fn cost(kind: RoadKind, len: f32, mode: Mode, country: f32) -> f32 {
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
}
