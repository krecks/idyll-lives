//! Reading a route: where a car is `t` ticks along it, and what the road is like there.
use super::*;

impl Route {
    pub(crate) fn ticks(&self) -> f32 {
        self.points.last().map_or(0.0, |p| p.2)
    }

    /// Its level `t` ticks along (`layers`: the ground but where it says).
    pub(crate) fn layer_at(&self, t: f32) -> crate::travel::Layer {
        self.layers.at(t).unwrap_or(crate::travel::Layer::GROUND)
    }

    /// The kind of road it's on `t` ticks along, of an avenue, a motorway and a ramp (`kinds`);
    /// None: any other, or off the roads.
    pub(crate) fn kind_at(&self, t: f32) -> Option<crate::travel::RoadKind> {
        self.kinds.at(t)
    }

    /// Whether it's on a motorway's carriageway or a ramp anywhere from `a` to `b` ticks along
    /// (`kinds`): nobody backs up or turns round there (`turn_round`, `back_off`).
    pub(crate) fn on_motorway(&self, a: f32, b: f32) -> bool {
        self.kinds.any(a.min(b), a.max(b), |k| k.separated())
    }

    /// How far along it (ticks) a car `t` ticks along is `d` m further on (as far as its end).
    pub(crate) fn ahead_by(&self, t: f32, d: f32) -> f32 {
        let pts = &self.points;
        let mut i = pts.partition_point(|q| q.2 <= t);
        let (mut a, mut ta, mut left) = (self.at(t).0, t, d);
        while i < pts.len() {
            let q = pts[i];
            let len = crate::travel::dist(a, q.0);
            if len >= left {
                return if len > 1e-6 {
                    ta + (q.2 - ta) * left / len
                } else {
                    q.2
                };
            }
            left -= len;
            (a, ta) = (q.0, q.2);
            i += 1;
        }
        self.ticks()
    }

    /// Where the car is `t` ticks in, which way it faces, and whether it's backing up.
    pub(crate) fn at(&self, t: f32) -> ([f32; 2], f32, bool) {
        let pts = &self.points;
        let i = pts
            .partition_point(|p| p.2 <= t)
            .clamp(1, pts.len().max(2) - 1);
        let (a, b) = (pts[i - 1], pts[i]);
        let k = if b.2 > a.2 {
            ((t - a.2) / (b.2 - a.2)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let p = [
            a.0[0] + (b.0[0] - a.0[0]) * k,
            a.0[1] + (b.0[1] - a.0[1]) * k,
        ];
        (p, wrap_angle(a.1 + wrap_angle(b.1 - a.1) * k), a.3)
    }

    /// How far it can pull over to pass one coming the other way `t` ticks along (m: `pass`).
    pub(crate) fn pass_at(&self, t: f32) -> f32 {
        let k = self.pass.partition_point(|p| p.0 <= t);
        self.pass.get(k.saturating_sub(1)).map_or(0.0, |p| p.1)
    }

    /// The point after the one `t` ticks along lies past, when `points` and the per-point array a
    /// caller reads (`lens` of them) agree: `Some(i)`, with `i - 1` and `i` the two either side.
    /// None: too few points to have a span, or the array doesn't match them.
    pub(super) fn span(&self, lens: usize, t: f32) -> Option<usize> {
        let pts = &self.points;
        if pts.len() < 2 || lens != pts.len() {
            return None;
        }
        Some(pts.partition_point(|p| p.2 <= t).clamp(1, pts.len() - 1))
    }

    /// How far right of its lane a car standing `t` ticks along can pull over (m: the more of the
    /// two points either side: it's on the lane up to the one where its turn off it begins).
    pub(crate) fn room_here(&self, t: f32) -> f32 {
        let Some(i) = self.span(self.room.len(), t) else {
            return 0.0;
        };
        self.room[i - 1].max(self.room[i])
    }

    /// How far right of the road's middle its lane is `t` ticks along (m: the more of the two
    /// points either side). None: nobody comes the other way at either.
    pub(super) fn lane_at(&self, t: f32) -> Option<f32> {
        // (A route whose points and lanes disagree keeps its car in its lane: `Some(LANE)`.)
        let Some(i) = self.span(self.lane.len(), t) else {
            return Some(LANE);
        };
        Some(self.lane[i - 1]?.max(self.lane[i]?))
    }

    /// How far right of its lane a car can pull over `t` ticks along (m: the less of the two
    /// points either side).
    pub(crate) fn room_at(&self, t: f32) -> f32 {
        let Some(i) = self.span(self.room.len(), t) else {
            return 0.0;
        };
        self.room[i - 1].min(self.room[i])
    }
}
