//! The way each car went over the last few real steps (`car::TRAIL`): at a faster speed a step
//! runs many ticks (at super speed up to 60, some 50 m of driving), and a car drawn on a straight
//! line from one snapshot to the next would cut across the bends and turns it took, onto the
//! wrong side of the road and into the cars beside it. The renderer draws it along these points
//! instead: the poses it had tick by tick, as few kept as draw it within `TRAIL_TOLERANCE` of
//! every one. Not saved: a game loaded draws its cars from the snapshots it makes from then on.

use std::collections::VecDeque;

/// Real steps back a car's way is kept for (`TRAIL_STEPS`): a frame drawn this many steps after
/// the one before (some 10 frames a second at 60 steps a second) still draws its cars along
/// their way; a longer one draws them on the straight line from the snapshot before, as ever.
pub const TRAIL_STEPS: u32 = 6;
/// Most points of one car's way in a snapshot (`car::TRAIL_LEN`), and of all of them together
/// (`TRAIL_POINTS`: the pool `car::TRAIL` points into; a car left without room is drawn on the
/// straight line).
pub const TRAIL_MOST: usize = 24;
pub const TRAIL_POINTS: usize = 4096;
/// How far (m) a car drawn along its kept points strays at most from where the sim had it at
/// any tick (its middle, and its ends turned about it: `YAW_LEVER`), unless it has more turns
/// than `TRAIL_MOST` points draw.
pub const TRAIL_TOLERANCE: f32 = 0.08;
/// How far from its middle (m) a car's yaw is judged by: about half a car's length.
const YAW_LEVER: f32 = 2.0;
/// How far (m) a car may be from where it was the tick before and its way go on: further, it was
/// put there (no car drives so far in a tick: on a motorway some 2 m), and its way starts again
/// there, not drawn sliding across from where it was.
const TRAIL_JUMP: f32 = 10.0;

/// A car's pose after a tick: the step it was in (`Trails::step`), the tick of that step (from
/// 1), where its middle was and which way it faced.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Point {
    step: u32,
    tick: u32,
    at: [f32; 2],
    yaw: f32,
}

/// Each car's poses over the last `TRAIL_STEPS` steps, and how many ticks each of those ran.
#[derive(Default, Debug)]
pub struct Trails {
    /// Steps begun (`begin_step`): the one under way.
    step: u32,
    /// The ticks each of the steps before it ran, newest last: (step, ticks).
    ran: VecDeque<(u32, u32)>,
    /// Per car (`World::cars` order, `remap`ped as that changes): its poses, oldest first, one
    /// before the window at most.
    cars: Vec<VecDeque<Point>>,
}

impl Trails {
    /// A real step begins (`World::start_step`); the one before ran `ticks`.
    pub(crate) fn begin_step(&mut self, ticks: u32) {
        if self.step > 0 {
            self.ran.push_back((self.step, ticks));
            while self.ran.len() > TRAIL_STEPS as usize + 1 {
                self.ran.pop_front();
            }
        }
        self.step += 1;
    }

    /// Every car's pose after tick `tick` (from 1) of the step under way: `shown`, each car's
    /// (`Car::shown`; none: out of sight, its way starts again when it's back).
    pub(crate) fn record(
        &mut self,
        tick: u32,
        shown: impl ExactSizeIterator<Item = Option<([f32; 2], f32)>>,
    ) {
        let step = self.step;
        self.cars.resize_with(shown.len(), VecDeque::new);
        for (way, pose) in self.cars.iter_mut().zip(shown) {
            let Some((at, yaw)) = pose else {
                way.clear();
                continue;
            };
            let p = Point {
                step,
                tick,
                at,
                yaw,
            };
            if way
                .back()
                .is_some_and(|q| (q.at[0] - at[0]).hypot(q.at[1] - at[1]) > TRAIL_JUMP)
            {
                way.clear();
            }
            // (Standing still: the run of the same pose is its first point and its last.)
            let n = way.len();
            if n >= 2 && same(&way[n - 1], &p) && same(&way[n - 2], &p) {
                way[n - 1] = p;
            } else {
                way.push_back(p);
            }
            // (One point before the window is kept: where the way into it starts.)
            while way.len() > 1 && way[1].step + TRAIL_STEPS <= step {
                way.pop_front();
            }
        }
    }

    /// `World::cars` put back in another order or as they were before (`World::restore`): car `c`
    /// now is the one at `from(c)` before, its way going on; None: a car put back elsewhere, its
    /// way starting again from where it is now.
    pub(crate) fn remap(&mut self, n: usize, from: impl Fn(usize) -> Option<usize>) {
        let mut old = std::mem::take(&mut self.cars);
        self.cars = (0..n)
            .map(|c| {
                from(c)
                    .and_then(|o| old.get_mut(o))
                    .map(std::mem::take)
                    .unwrap_or_default()
            })
            .collect();
    }

    /// When point `p` was, in steps from the end of the step under way (which ran `now` ticks):
    /// 0 its last tick, -1 the end of the step before. None: from before the steps kept.
    fn when(&self, p: &Point, now: u32) -> Option<f32> {
        let ticks = if p.step == self.step {
            now
        } else {
            // (`ran` holds one step after another, oldest first.)
            let back = p.step.checked_sub(self.ran.front()?.0)?;
            self.ran.get(back as usize).filter(|r| r.0 == p.step)?.1
        };
        let into = if ticks == 0 {
            1.0
        } else {
            p.tick as f32 / ticks as f32
        };
        Some((i64::from(p.step) - i64::from(self.step)) as f32 - 1.0 + into)
    }

    /// Car `c`'s way into the snapshot (the step under way having run `now` ticks): as few of its
    /// poses as draw it within `TRAIL_TOLERANCE` of all of them (`TRAIL_MOST` at most), oldest
    /// first, each `[t, x, z, yaw]` with `t` in steps before the snapshot (below 0; the snapshot's
    /// own pose, at 0, is the car's row), into `out`. How many. (Nothing allocated but `points`
    /// grown, kept by the caller for every car: it's written every step.)
    pub(crate) fn write(
        &self,
        c: usize,
        now: u32,
        out: &mut [[f32; 4]],
        points: &mut Vec<[f32; 4]>,
    ) -> usize {
        let Some(way) = self.cars.get(c) else {
            return 0;
        };
        // (Its points from the steps kept: all but one before them, where it stood since. Each
        // worked out once: `simplify` looks at most of them several times.)
        let at = |p: &Point| self.when(p, now).map(|t| [t, p.at[0], p.at[1], p.yaw]);
        let first = way
            .iter()
            .position(|p| at(p).is_some())
            .unwrap_or(way.len());
        points.clear();
        points.extend(way.iter().skip(first).map(|p| at(p).unwrap_or_default()));
        let point = |i: usize| points[i];
        let mut kept = [0; TRAIL_MOST + 1];
        let most = out.len().min(TRAIL_MOST) + 1;
        let mut n = simplify(points.len(), point, &mut kept[..most]);
        // (The last is the snapshot's pose when it's at 0: the car's row has it.)
        if n > 0 && point(kept[n - 1])[0] >= 0.0 {
            n -= 1;
        }
        let n = n.min(out.len());
        for (o, &k) in out.iter_mut().zip(&kept[..n]) {
            *o = point(k);
        }
        n
    }
}

fn same(a: &Point, b: &Point) -> bool {
    a.at == b.at && a.yaw == b.yaw
}

/// How far `p` (`[t, x, z, yaw]`) is from where the straight way from `a` to `b` has it at its
/// time (m; its yaw's turn at `YAW_LEVER`). `turn`: how far it turns from `a` to `b`
/// (`wrap_angle`), the same for every `p` between them.
fn off(a: &[f32; 4], b: &[f32; 4], turn: f32, p: &[f32; 4]) -> f32 {
    let k = if b[0] > a[0] {
        ((p[0] - a[0]) / (b[0] - a[0])).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let x = a[1] + (b[1] - a[1]) * k;
    let z = a[2] + (b[2] - a[2]) * k;
    let yaw = crate::world::wrap_angle(p[3] - (a[3] + turn * k));
    (p[1] - x).hypot(p[2] - z) + yaw.abs() * YAW_LEVER
}

/// Which of `n` points (`point(i)`, oldest first) are kept to draw them: the first and the last,
/// and those furthest off the way between them in turn until every one is within
/// `TRAIL_TOLERANCE` or `kept` is full. Their indices, in order, into `kept`; how many.
pub(crate) fn simplify(n: usize, point: impl Fn(usize) -> [f32; 4], kept: &mut [usize]) -> usize {
    if n <= 2 || kept.len() < 2 {
        for (i, k) in kept.iter_mut().take(n).enumerate() {
            *k = i;
        }
        return n.min(kept.len());
    }
    // The point furthest off the straight way between `a` and `b`, and how far.
    let worst = |a: usize, b: usize| -> (usize, f32) {
        let (pa, pb) = (point(a), point(b));
        let turn = crate::world::wrap_angle(pb[3] - pa[3]);
        (a + 1..b)
            .map(|i| (i, off(&pa, &pb, turn, &point(i))))
            .fold((a, 0.0), |m, x| if x.1 > m.1 { x } else { m })
    };
    // Each stretch between two kept points (`kept[s]`, `kept[s + 1]`): its worst point.
    let mut worsts = [(0, 0.0); TRAIL_MOST + 1];
    kept[0] = 0;
    kept[1] = n - 1;
    worsts[0] = worst(0, n - 1);
    let mut count = 2;
    while count < kept.len() {
        let Some(s) = (0..count - 1)
            .filter(|&s| worsts[s].1 > TRAIL_TOLERANCE)
            .max_by(|&a, &b| worsts[a].1.total_cmp(&worsts[b].1))
        else {
            break;
        };
        let (a, i, b) = (kept[s], worsts[s].0, kept[s + 1]);
        kept.copy_within(s + 1..count, s + 2);
        worsts.copy_within(s + 1..count - 1, s + 2);
        kept[s + 1] = i;
        worsts[s] = worst(a, i);
        worsts[s + 1] = worst(i, b);
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trails_of(poses: &[Vec<([f32; 2], f32)>]) -> Trails {
        // (One car, a step per entry, a tick per pose in it.)
        let mut t = Trails::default();
        let mut ran = 0;
        for step in poses {
            t.begin_step(ran);
            for (k, &(at, yaw)) in step.iter().enumerate() {
                t.record(k as u32 + 1, std::iter::once(Some((at, yaw))));
            }
            ran = step.len() as u32;
        }
        t
    }

    #[test]
    fn a_straight_drive_keeps_no_point_between() {
        let step: Vec<_> = (0..60).map(|k| ([0.0, k as f32 * 0.9], 0.0)).collect();
        let t = trails_of(&[step]);
        let mut out = [[0.0; 4]; TRAIL_MOST];
        // (Its first tick only: the last is the row's.)
        assert_eq!(t.write(0, 60, &mut out, &mut Vec::new()), 1);
        assert!((out[0][0] - (-1.0 + 1.0 / 60.0)).abs() < 1e-6);
    }

    #[test]
    fn a_turn_keeps_the_corner() {
        // Up +z 20 ticks, then along +x 20 ticks, in one step.
        let mut step: Vec<_> = (0..20).map(|k| ([0.0, k as f32], 0.0)).collect();
        step.extend((0..20).map(|k| ([k as f32, 20.0], std::f32::consts::FRAC_PI_2)));
        let t = trails_of(&[step.clone()]);
        let mut out = [[0.0; 4]; TRAIL_MOST];
        let n = t.write(0, 40, &mut out, &mut Vec::new());
        assert!(n >= 2, "{n}");
        // Every tick's pose is within the tolerance of the drawn way.
        for (k, &(at, _)) in step.iter().enumerate() {
            let tk = -1.0 + (k as f32 + 1.0) / 40.0;
            let mut way: Vec<[f32; 4]> = out[..n].to_vec();
            way.push([0.0, 19.0, 20.0, std::f32::consts::FRAC_PI_2]);
            let j = way.iter().position(|p| p[0] >= tk - 1e-6).unwrap();
            let p = if j == 0 {
                way[0]
            } else {
                let (a, b) = (way[j - 1], way[j]);
                let f = (tk - a[0]) / (b[0] - a[0]);
                [tk, a[1] + (b[1] - a[1]) * f, a[2] + (b[2] - a[2]) * f, 0.0]
            };
            assert!(
                (p[1] - at[0]).hypot(p[2] - at[1]) <= TRAIL_TOLERANCE + 1e-4,
                "tick {k}"
            );
        }
    }

    #[test]
    fn steps_before_are_timed_by_their_ticks() {
        // Two steps: 10 ticks, then 30; the first step's last tick is at -1.
        let a: Vec<_> = (0..10).map(|k| ([0.0, k as f32], 0.0)).collect();
        let b: Vec<_> = (0..30).map(|k| ([k as f32, 10.0], 1.0)).collect();
        let t = trails_of(&[a, b]);
        let way = &t.cars[0];
        assert_eq!(t.when(&way[9], 30), Some(-1.0));
        assert_eq!(t.when(&way[0], 30), Some(-2.0 + 0.1));
        assert_eq!(t.when(&way[39], 30), Some(0.0));
    }

    #[test]
    fn only_the_steps_kept_and_one_point_before() {
        let steps: Vec<Vec<_>> = (0..20).map(|s| vec![([0.0, s as f32], 0.0)]).collect();
        let t = trails_of(&steps);
        let way = &t.cars[0];
        assert_eq!(way.len(), TRAIL_STEPS as usize + 1);
        assert_eq!(way[0].step + TRAIL_STEPS, t.step);
    }

    #[test]
    fn standing_still_is_two_points() {
        let step: Vec<_> = (0..50).map(|_| ([3.0, 4.0], 0.5)).collect();
        let t = trails_of(&[step]);
        assert_eq!(t.cars[0].len(), 2);
        assert_eq!(t.cars[0][1].tick, 50);
    }

    #[test]
    fn a_car_put_elsewhere_starts_again() {
        let mut t = Trails::default();
        t.begin_step(0);
        t.record(1, std::iter::once(Some(([0.0, 0.0], 0.0))));
        t.record(2, std::iter::once(Some(([1.5, 0.0], 0.0))));
        t.record(3, std::iter::once(Some(([40.0, 0.0], 0.0))));
        assert_eq!(t.cars[0].len(), 1);
        assert_eq!(t.cars[0][0].at, [40.0, 0.0]);
    }

    #[test]
    fn remapped_ways_go_with_their_cars() {
        let mut t = Trails::default();
        t.begin_step(0);
        let poses = [
            Some(([0.0, 0.0], 0.0)),
            Some(([100.0, 0.0], 0.0)),
            Some(([0.0, 100.0], 0.0)),
        ];
        t.record(1, poses.into_iter());
        // (The first put back elsewhere, the other two swapped.)
        t.remap(3, |c| [None, Some(2), Some(1)][c]);
        assert!(t.cars[0].is_empty());
        assert_eq!(t.cars[1][0].at, [0.0, 100.0]);
        assert_eq!(t.cars[2][0].at, [100.0, 0.0]);
    }

    #[test]
    fn out_of_sight_starts_again() {
        let mut t = Trails::default();
        t.begin_step(0);
        t.record(1, std::iter::once(Some(([0.0, 0.0], 0.0))));
        t.record(2, std::iter::once(None));
        t.record(3, std::iter::once(Some(([5.0, 0.0], 0.0))));
        assert_eq!(t.cars[0].len(), 1);
    }
}
