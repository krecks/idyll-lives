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
        }
    }
}

// ---- the cars' helpers, split by what they do ----------------------------------------------
