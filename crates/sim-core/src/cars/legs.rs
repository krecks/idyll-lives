//! Legs and turns: laying a route through junctions.

use super::*;

/// A turn at a driveway's mouth, in the frame of the road there (`Route::on_roads`): from going
/// `din` to going `dout` round the corner where their lines cross, an arc as tight as `TURN` at
/// most that begins no further back from the corner than `room` (the driveway's length to it).
/// A driveway square to the road turns a quarter, as `Route::legacy` does.
pub(crate) struct Turn {
    /// Where it begins and ends, the arc's middle and radius (0: straight on).
    pub(super) from: [f32; 2],
    pub(super) to: [f32; 2],
    centre: [f32; 2],
    r: f32,
    /// How far from the corner it begins and ends (m).
    pub(crate) taken: f32,
}

impl Turn {
    pub(super) fn round(corner: [f32; 2], din: [f32; 2], dout: [f32; 2], room: f32) -> Turn {
        let cos = (din[0] * dout[0] + din[1] * dout[1]).clamp(-0.999, 1.0);
        // tan(θ/2) of the angle turned (exactly 1 for a quarter turn).
        let tan = ((1.0 - cos) / (1.0 + cos)).sqrt();
        if tan < 1e-3 {
            return Turn {
                from: corner,
                to: corner,
                centre: corner,
                r: 0.0,
                taken: 0.0,
            };
        }
        let r = TURN.min(room / tan).max(0.5);
        let taken = r * tan;
        let from = [corner[0] - din[0] * taken, corner[1] - din[1] * taken];
        let to = [corner[0] + dout[0] * taken, corner[1] + dout[1] * taken];
        // Towards the inside of the turn, square to the way in.
        let m = [dout[0] - cos * din[0], dout[1] - cos * din[1]];
        let ml = m[0].hypot(m[1]).max(1e-6);
        Turn {
            from,
            to,
            centre: [from[0] + m[0] / ml * r, from[1] + m[1] / ml * r],
            r,
            taken,
        }
    }

    /// The arc onto `legs`, on the town (`world`: from the road's frame).
    pub(super) fn drive(
        &self,
        legs: &mut Legs,
        world: &impl Fn([f32; 2]) -> [f32; 2],
        reverse: bool,
    ) {
        if self.r > 0.0 {
            legs.arc(
                world(self.centre),
                self.r,
                world(self.from),
                world(self.to),
                reverse,
            );
        }
    }
}

/// The pieces of a route as they're laid out: sampled points, whether the car backs along them,
/// and the top speed there.
#[derive(Default)]
pub(crate) struct Legs {
    pub(crate) pts: Vec<([f32; 2], bool, f32)>,
    /// At each point, how far a car can pull over there (`Route::room`), and its lane
    /// (`Route::lane`).
    pub(super) room: Vec<f32>,
    pub(super) lane: Vec<f32>,
    /// Its mouths (`Route::mouths`), from point to point (None: to the end).
    pub(crate) mouths: Vec<(usize, Option<usize>)>,
    /// Its points off the ground (`travel::Layer`: on a deck, or about a merge on one); every
    /// other on the ground (`Route::layers`).
    pub(crate) layers: Marks<crate::travel::Layer>,
    /// Its points where it merges off a ramp into a motorway's carriageway (`Route::merges`).
    pub(crate) merges: Marks<()>,
    /// Its points on an avenue, a motorway or a ramp, and which (`Route::kinds`); every other on
    /// a street, a lane, a country road or off the roads.
    pub(crate) kinds: Marks<crate::travel::RoadKind>,
}

/// Points of a route as it's laid out (`Legs`), by index, each marked with a value (once).
#[derive(Debug, Clone)]
pub(crate) struct Marks<T>(Vec<(usize, T)>);

impl<T> Default for Marks<T> {
    fn default() -> Self {
        Marks(Vec::new())
    }
}

impl<T: Copy + PartialEq> Marks<T> {
    /// Point `i` marked `v`, unless it's marked already (the last marked).
    fn mark(&mut self, i: usize, v: T) {
        if self.0.last().is_none_or(|m| m.0 != i) {
            self.0.push((i, v));
        }
    }

    /// The marked points as runs along the route (`Runs`), point `i` `tick(i)` ticks along it:
    /// each run of points one after another with one value, from the first's tick to the last's.
    /// Where one point's value is followed straight away by another's, `seam` lays what's between
    /// them (the runs so far, the ticks of the two points, and their values) before the next run
    /// begins.
    fn runs(
        &self,
        tick: impl Fn(usize) -> f32,
        seam: impl Fn(&mut Vec<(f32, f32, T)>, f32, f32, T, T),
    ) -> Runs<T> {
        let mut runs: Vec<(f32, f32, T)> = Vec::new();
        let mut last: Option<(usize, T)> = None;
        for &(i, v) in &self.0 {
            let t = tick(i);
            match last {
                Some((j, k)) if j + 1 == i && k == v => {
                    if let Some(run) = runs.last_mut() {
                        run.1 = t;
                    }
                }
                Some((j, k)) if j + 1 == i => {
                    seam(&mut runs, tick(j), t, k, v);
                    runs.push((t, t, v));
                }
                _ => runs.push((t, t, v)),
            }
            last = Some((i, v));
        }
        Runs(runs)
    }
}

/// Stretches of a route with a value of their own (ticks along it, from and to, and the value), in
/// order: `Route::layers`, `Route::kinds`. Elsewhere none.
#[derive(Debug, Clone)]
pub(crate) struct Runs<T>(Vec<(f32, f32, T)>);

impl<T> Default for Runs<T> {
    fn default() -> Self {
        Runs(Vec::new())
    }
}

impl<T: Copy> Runs<T> {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The value `t` ticks along (the first run there). None: no run there.
    pub(crate) fn at(&self, t: f32) -> Option<T> {
        self.0.iter().find(|r| r.0 <= t && t <= r.1).map(|r| r.2)
    }

    /// Whether a run whose value is `keep` is anywhere from `a` to `b` ticks along (`a <= b`).
    pub(crate) fn any(&self, a: f32, b: f32, keep: impl Fn(T) -> bool) -> bool {
        self.0.iter().any(|r| keep(r.2) && r.0 <= b && a <= r.1)
    }
}

impl Legs {
    pub(crate) fn len(&self) -> usize {
        self.pts.len()
    }

    pub(crate) fn last(&self) -> Option<[f32; 2]> {
        self.pts.last().map(|p| p.0)
    }

    pub(crate) fn push(&mut self, p: [f32; 2], reverse: bool, top: f32) {
        if self
            .pts
            .last()
            .is_some_and(|q| (q.0[0] - p[0]).hypot(q.0[1] - p[1]) < 1e-3 && q.1 == reverse)
        {
            return;
        }
        self.pts.push((p, reverse, top));
        self.room.push(0.0);
        self.lane.push(LANE);
    }

    /// `push` (driving on) of a lane's point `lane` m right of the road's middle, where a car can
    /// pull over `room` m (`Route::room`, `Route::lane`).
    pub(crate) fn push_lane(&mut self, p: [f32; 2], top: f32, room: f32, lane: f32) {
        let n = self.pts.len();
        self.push(p, false, top);
        if self.pts.len() > n {
            *self.room.last_mut().expect("just pushed") = room;
            *self.lane.last_mut().expect("just pushed") = lane;
        }
    }

    /// `push_lane`, marked as on a road of `kind` (`mark_kind`) and with its marks (`mark_last`:
    /// `LaneLine::laid`) where it has them.
    pub(crate) fn push_marked(
        &mut self,
        p: [f32; 2],
        top: f32,
        room: f32,
        lane: f32,
        kind: crate::travel::RoadKind,
        mark: Option<(crate::travel::Layer, bool)>,
    ) {
        self.push_lane(p, top, room, lane);
        self.mark_kind(kind);
        if let Some(m) = mark {
            self.mark_last(m);
        }
    }

    /// `push` of a point it backs up to (at `SLOW`), on a lane where a car can pull over `room`
    /// m, `lane` m right of the road's middle (`Route::room`, `Route::lane`).
    pub(crate) fn push_back(&mut self, p: [f32; 2], room: f32, lane: f32) {
        let n = self.pts.len();
        self.push(p, true, SLOW);
        if self.pts.len() > n {
            *self.room.last_mut().expect("just pushed") = room;
            *self.lane.last_mut().expect("just pushed") = lane;
        }
    }

    /// Its last point marked as on `layer` (none: on the ground), and as where it merges into a
    /// motorway (`merge`).
    pub(crate) fn mark_last(&mut self, (layer, merge): (crate::travel::Layer, bool)) {
        let Some(i) = self.pts.len().checked_sub(1) else {
            return;
        };
        if layer != crate::travel::Layer::GROUND {
            self.layers.mark(i, layer);
        }
        if merge {
            self.merges.mark(i, ());
        }
    }

    /// Its last point marked as on a road of `kind` (`Route::kinds`: only an avenue, a motorway
    /// or a ramp is kept).
    pub(crate) fn mark_kind(&mut self, kind: crate::travel::RoadKind) {
        let Some(i) = self.pts.len().checked_sub(1) else {
            return;
        };
        if kind.fast() {
            self.kinds.mark(i, kind);
        }
    }

    pub(crate) fn line(&mut self, a: [f32; 2], b: [f32; 2], reverse: bool, top: f32) {
        let n = ((a[0] - b[0]).hypot(a[1] - b[1]) / STEP).ceil().max(1.0) as usize;
        for i in 0..=n {
            let k = i as f32 / n as f32;
            self.push(
                [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k],
                reverse,
                top,
            );
        }
    }

    /// A quarter turn about `c` from `a` to `b` (both `r` from it).
    pub(super) fn arc(&mut self, c: [f32; 2], r: f32, a: [f32; 2], b: [f32; 2], reverse: bool) {
        let a0 = (a[1] - c[1]).atan2(a[0] - c[0]);
        let a1 = a0 + wrap_angle((b[1] - c[1]).atan2(b[0] - c[0]) - a0);
        let n = ((a1 - a0).abs() * r / STEP).ceil().max(1.0) as usize;
        for i in 0..=n {
            let t = a0 + (a1 - a0) * i as f32 / n as f32;
            self.push([c[0] + r * t.cos(), c[1] + r * t.sin()], reverse, SLOW);
        }
    }

    /// Times each point: the car speeds up and slows down (`ACCEL`), stands still where it
    /// changes between backing and driving on, and at the start (`stop_start`) and the end
    /// (`stop_end`). Yaw: the way it goes, turned round while it backs.
    pub(crate) fn timed(self, stop_start: bool, stop_end: bool) -> Route {
        let pts = self.pts;
        let n = pts.len();
        if n < 2 {
            return Route {
                points: pts.iter().map(|p| (p.0, 0.0, 0.0, false)).collect(),
                ..Default::default()
            };
        }
        let ds: Vec<f32> = (1..n)
            .map(|i| (pts[i].0[0] - pts[i - 1].0[0]).hypot(pts[i].0[1] - pts[i - 1].0[1]))
            .collect();
        let mut v: Vec<f32> = pts.iter().map(|p| p.2).collect();
        for i in 1..n - 1 {
            if pts[i].1 != pts[i + 1].1 {
                v[i] = 0.0;
            }
        }
        if stop_start {
            v[0] = 0.0;
        }
        if stop_end {
            v[n - 1] = 0.0;
        }
        // (Round a bend, no faster than pulls it sideways `SIDEWAYS`: its curve, the way turned
        // at a point over the steps either side.)
        for i in 1..n - 1 {
            if pts[i].1 == pts[i + 1].1 && ds[i - 1] > 1e-4 && ds[i] > 1e-4 {
                let (a, b, c) = (pts[i - 1].0, pts[i].0, pts[i + 1].0);
                let turned =
                    wrap_angle((c[0] - b[0]).atan2(c[1] - b[1]) - (b[0] - a[0]).atan2(b[1] - a[1]));
                let curve = turned.abs() / ((ds[i - 1] + ds[i]) / 2.0);
                if curve > 1e-4 {
                    v[i] = v[i].min((SIDEWAYS / curve).sqrt());
                }
            }
        }
        for i in 1..n {
            v[i] = v[i].min((v[i - 1] * v[i - 1] + 2.0 * ACCEL * ds[i - 1]).sqrt());
        }
        for i in (0..n - 1).rev() {
            v[i] = v[i].min((v[i + 1] * v[i + 1] + 2.0 * ACCEL * ds[i]).sqrt());
        }
        // (Ticks of drive time: `crate::DRIVE_SCALE` times as fast as real time.)
        let per_second = DRIVE_TICKS_PER_SECOND;
        // Facing along each step: the way it goes, turned round while it backs; along a step of
        // none (where it changes between backing and driving on) or next to none (`FACING_STEP`),
        // as along the one before.
        let mut facing: Vec<f32> = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            let (a, b) = (pts[i].0, pts[i + 1].0);
            let way = (b[0] - a[0]).atan2(b[1] - a[1]);
            let way = if pts[i + 1].1 {
                wrap_angle(way + std::f32::consts::PI)
            } else {
                way
            };
            let kink = i > 0
                && ds[i] < KINK_STEP
                && pts[i + 1].1 == pts[i].1
                && wrap_angle(way - facing[i - 1]).abs() > std::f32::consts::FRAC_PI_2;
            let f = if (ds[i] > FACING_STEP || i == 0) && !kink {
                way
            } else {
                facing[i - 1]
            };
            facing.push(f);
        }
        let mut t = 0.0;
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            if i > 0 {
                t += ds[i - 1] / ((v[i - 1] + v[i]) / 2.0).max(0.3) * per_second;
            }
            // Facing: along the step on (the last, the one before); between two it goes the same
            // way (driving on, or backing), half way from the one to the other: along its curve,
            // as a corner's arc is sampled (`round_corners`).
            let reverse = if i + 1 < n { pts[i + 1].1 } else { pts[i].1 };
            let yaw = if i + 1 == n {
                facing[n - 2]
            } else if i > 0 && pts[i].1 == pts[i + 1].1 {
                facing[i - 1] + wrap_angle(facing[i] - facing[i - 1]) / 2.0
            } else {
                facing[i]
            };
            points.push((pts[i].0, wrap_angle(yaw), t, reverse));
        }
        let at = |i: Option<usize>| i.map_or(t, |i| points[i.min(n - 1)].2);
        let mouths = self
            .mouths
            .iter()
            .map(|&(a, b)| (at(Some(a)), at(b)))
            .collect();
        let tick = |i: usize| points[i.min(n - 1)].2;
        // (Runs of points on one level off the ground; from one point to the next on another,
        // both: no stretch between two runs on the ground, where one on the deck and one coming
        // off it wouldn't meet.)
        let layers = self.layers.runs(tick, |runs, was, t, a, b| {
            runs.push((was, t, a.and(b)));
        });
        let merges = self.merges.0.iter().map(|&(i, _)| tick(i)).collect();
        // (Runs of points on one kind of road; one run on to the next of another kind straight
        // after it, up to its first: no stretch between them.)
        let kinds = self.kinds.runs(tick, |runs, _, t, _, _| {
            if let Some(run) = runs.last_mut() {
                run.1 = t;
            }
        });
        Route {
            points,
            room: self.room,
            lane: self.lane,
            pass: Vec::new(),
            mouths,
            claims: Vec::new(),
            zebras: Vec::new(),
            spans: Vec::new(),
            leg: None,
            stand_gen: 0,
            by_motorway: false,
            layers,
            merges,
            kinds,
        }
    }
}

// ---- the cars' helpers, split by what they do ----------------------------------------------
