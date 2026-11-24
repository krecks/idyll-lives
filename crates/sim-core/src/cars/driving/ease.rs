//! A car's own speed on an avenue, a motorway or a ramp (`Car::speed`, `travel::fast_kind`).
//!
//! Everywhere else a car keeps to its way's timing (`Legs::timed`): a tick on along it, or none
//! (held up: it stands at once). At a town's 9 m/s that's a step of 0.6 m; at the motorway's
//! 30 m/s, 2 m, and a car stopping from there stood at once, and one stopped behind another crept
//! up by turns, a tick on and a tick stood. There (and on from there till it's back at its way's
//! speed) it has a speed of its own each tick and goes on as far along its way as that takes it:
//! up to its way's speed, faster by `ACCEL` a second at most, and slower as it must, by `BRAKE`
//! a second as it plans to and `BRAKE_MOST` at most, to stop, or to keep back, behind
//!
//! - the car in front of it on its way (its body on its way ahead, going its way: `Path::reach`),
//!   as far back as it could stop if that one stopped as it does (`follow_ahead`'s ground and
//!   `EASE_ROOM` more): at speed it keeps as far back as `follow_ahead` says, and coming up to one
//!   standing it slows to a stop behind it;
//! - one about to merge onto its carriageway off a ramp, holding its merge's claim
//!   (`traffic::Claim::merge`), as if it were already where it'll be along the carriageway: nearer
//!   the merge than this one is, it's ahead of it (as a driver lets one in that was let in);
//!   and the other way round, merging, those on the carriageway nearer the merge;
//! - the start of a merge's claim, or of one through lights not green for it, it doesn't hold: it
//!   comes up to it at `CREEP` at least, and asks there (`Traffic::may_go`; for a merge, as soon
//!   as it would begin to slow down for it: `ask_ahead`, let in by then, it goes on at speed).
//!
//! Every rule that holds a car up still does, at once, as the last resort (`up_against`, the
//! claims, `guard`); easing off as above, it seldom comes to that. Classic towns have none of these
//! roads: their cars drive as they always did, tick for tick.

use super::*;
use crate::travel::dist;

/// Slowing down as a car plans to (m/s² of drive time): as it speeds up (`ACCEL`).
pub(crate) const BRAKE: f32 = 3.0;
/// The hardest it slows down (m/s² of drive time), but held up at once by the last resort.
pub(crate) const BRAKE_MOST: f32 = 6.0;
/// The room it keeps past the ground in front of it it keeps clear (`follow_ahead`, m).
const EASE_ROOM: f32 = 1.0;
/// Coming up to the start of a claim it stops at (`EASE_STOPS`), at least this fast (m/s) till its
/// nose is there and it asks.
const CREEP: f32 = 1.5;
/// Slower than this (m/s), it stands; and standing behind another, it sets off again once there's
/// this much more room in front of it than it keeps (m).
const STANDS: f32 = 0.05;
const SET_OFF: f32 = 1.0;
/// How far along its way it looks at most (m).
const EASE_LOOK: f32 = 200.0;
/// How near its way passes a merge's point for it to come to it there (m).
const MERGE_NEAR: f32 = 3.0;

/// What `ease` says of a car's tick: how far on along its way (ticks), its speed after it (m/s),
/// and the car it keeps back for (`car_first`'s key), standing.
pub(crate) struct Eased {
    pub step: f32,
    pub speed: f32,
    pub behind: Option<u64>,
    /// Which goes first at a merge, newly worked out (`merge_order`): the car merging, the key of
    /// the one along the carriageway, and whether the one merging goes first (`Car::merge_first`).
    pub decided: Vec<(usize, u64, bool)>,
}

/// How fast a car `t` ticks along `route` goes keeping to its timing (m/s of drive time).
pub(crate) fn plan_speed(route: &Route, t: f32) -> f32 {
    dist(route.at(t).0, route.at(t + 1.0).0) * DRIVE_TICKS_PER_SECOND
}

/// How fast car `car` goes now (m/s of drive time): its own speed, else its way's if it moved
/// last tick.
fn speed_of(car: &Car) -> f32 {
    car.speed.unwrap_or_else(|| match (&car.route, car.moved) {
        (Some((_, r)), true) => plan_speed(r, car.progress),
        _ => 0.0,
    })
}

/// A stretch of a car's way on from where it is: its steps, each from `a` to `b`, `d` m along
/// from where it is at `a`, at ticks `ta` to `tb`, at speed `v`.
struct Path(Vec<Step>);

/// A step of a `Path`: from and to, how far along it begins (m), its ticks along the way, and how
/// fast its way's timing goes there (m/s of drive time): the whole step between the way's two
/// points it lies on, the first from where the car is (part of one) too. (Worked out from its own
/// part of a step, a car standing a hair's breadth short of a point, its part of the step there of
/// no length at all in f32 but some ticks long, had a speed of none there, and stood for good
/// with nobody to wait for: a taxi turning off an avenue, the queue behind it, Oatcroft.)
type Step = ([f32; 2], [f32; 2], f32, f32, f32, f32);

impl Path {
    /// `route` from `t` ticks along, `look` m of it at most (as far as it drives on, not backing).
    fn new(route: &Route, t: f32, look: f32) -> Path {
        let pts = &route.points;
        let mut i = pts.partition_point(|q| q.2 <= t);
        let (mut a, mut ta, mut d) = (route.at(t).0, t, 0.0);
        let mut out = Vec::new();
        while i < pts.len() && d < look {
            if i > 0 && pts[i - 1].3 {
                break;
            }
            let q = pts[i];
            let (p, tp) = if i > 0 {
                (pts[i - 1].0, pts[i - 1].2)
            } else {
                (a, ta)
            };
            let v = if q.2 > tp + 1e-6 {
                dist(p, q.0) / (q.2 - tp) * DRIVE_TICKS_PER_SECOND
            } else {
                f32::INFINITY
            };
            out.push((a, q.0, d, ta, q.2, v));
            d += dist(a, q.0);
            (a, ta) = (q.0, q.2);
            i += 1;
        }
        Path(out)
    }

    /// How far along it (m) it first comes within `near` of `p`, and the way it goes there (a
    /// unit vector).
    fn reach(&self, p: [f32; 2], near: f32) -> Option<(f32, [f32; 2])> {
        self.0.iter().find_map(|&(a, b, d, ..)| {
            let ab = [b[0] - a[0], b[1] - a[1]];
            let len = ab[0].hypot(ab[1]);
            if len < 1e-6 {
                return (dist(a, p) < near).then_some((d, [0.0, 0.0]));
            }
            let u = [ab[0] / len, ab[1] / len];
            let s = ((p[0] - a[0]) * u[0] + (p[1] - a[1]) * u[1]).clamp(0.0, len);
            let q = [a[0] + u[0] * s, a[1] + u[1] * s];
            (dist(q, p) < near).then_some((d + s, u))
        })
    }

    /// The fastest a car may go where it begins to slow down to each step's speed by `BRAKE`
    /// before it (m/s of drive time).
    fn tops(&self) -> f32 {
        self.0
            .iter()
            .filter(|s| s.4 > s.3 + 1e-6)
            .map(|&(.., d, _, _, v)| (v * v + 2.0 * BRAKE * d).sqrt())
            .fold(f32::INFINITY, f32::min)
    }

    /// How far along it (m) it is `t` ticks along its way; None: further than it reaches.
    fn at_tick(&self, t: f32) -> Option<f32> {
        self.0.iter().find_map(|&(a, b, d, ta, tb, _)| {
            (ta <= t && t <= tb).then(|| {
                let k = if tb > ta { (t - ta) / (tb - ta) } else { 1.0 };
                d + dist(a, b) * k
            })
        })
    }
}

/// Where car `k` merges onto a motorway's carriageway next, still ahead of it: the point and the
/// tick along its way (`Route::merges`).
fn merge_at(k: &Car) -> Option<([f32; 2], f32)> {
    let (_, r) = k.route.as_ref()?;
    let m = *r.merges.iter().find(|&&m| m >= k.progress)?;
    Some((r.at(m).0, m))
}

/// Of car `ramp` merging (holding its merge's claim) at `at` (its point and tick along its way:
/// `merge_at`) and car `by` coming along the carriageway to it, each as far from it as `ramp` and
/// `by` (m along their ways), and which goes first (`ramp_first`): the one merging, if `by` can
/// keep back behind it from where it is, slowing by `BRAKE` at most (as if both on the carriageway
/// already, `ease`), or if it couldn't stop short of the taper itself any more (`TAPER`,
/// `BRAKE_MOST`); else `by`, the one merging slowing to let it by. Worked out once, as they first
/// come to it, and kept (`Car::merge_first`, `new`): the same for both cars, and nobody changing
/// their mind, both slowing for each other by turns. None: `by`'s way doesn't come to it.
fn merge_order(w: &World, ramp: usize, by: usize, at: ([f32; 2], f32)) -> Option<MergeOrder> {
    let (r, b) = (&w.cars[ramp], &w.cars[by]);
    // (Not one merging there too, up the ramp behind it: that one keeps behind it as anyone.)
    if merge_at(b).is_some_and(|(p, _)| dist(p, at.0) < MERGE_NEAR) {
        return None;
    }
    let kept = r
        .merge_first
        .iter()
        .find(|d| d.0 == car_first(by))
        .map(|d| d.1);
    let ramp_d = Path::new(&r.route.as_ref()?.1, r.progress, EASE_LOOK).at_tick(at.1)?;
    let (by_d, _) =
        Path::new(&b.route.as_ref()?.1, b.progress, EASE_LOOK).reach(at.0, MERGE_NEAR)?;
    let (vr, vb) = (speed_of(r), speed_of(b));
    let lens = (r.len + b.len) / 2.0;
    let committed = ramp_d - TAPER - r.len / 2.0 < vr * vr / (2.0 * BRAKE_MOST) + EASE_ROOM;
    let keeps_back = by_d - ramp_d
        >= (vb * vb - vr * vr).max(0.0) / (2.0 * BRAKE)
            + follow_ahead(vr, w.travel.country_speed)
            + EASE_ROOM
            + lens;
    Some(MergeOrder {
        ramp_first: kept.unwrap_or(committed || keeps_back),
        new: kept.is_none(),
        ramp: ramp_d,
        by: by_d,
    })
}

/// `merge_order`'s answer.
struct MergeOrder {
    ramp_first: bool,
    new: bool,
    ramp: f32,
    by: f32,
}

/// Car `c`, `t` ticks along its way (`home`: the way back), its own speed `carried` (`Car::speed`),
/// on the move last tick or not (`moving`), with `others` on the road about it (`vehicles`): its
/// tick at a speed of its own, on an avenue, a motorway or a ramp or till it's back at its way's
/// speed from one (see the module notes). None: it keeps to its way's timing (not on one of those
/// roads, or backing up).
pub(crate) fn ease(
    w: &World,
    c: usize,
    t: f32,
    carried: Option<f32>,
    moving: bool,
    home: bool,
    others: &[Driving],
) -> Option<Eased> {
    let car = &w.cars[c];
    let route = car.route.as_ref().filter(|r| r.0 == home).map(|r| &r.1)?;
    if route.at(t).2 || route.at(t + 1.0).2 {
        return None;
    }
    let plan = plan_speed(route, t);
    let v0 = carried.unwrap_or(if moving { plan } else { 0.0 });
    // (Off those roads, at its way's speed again: on as its timing has it.)
    if route.kind_at(t).is_none() && (carried.is_none() || v0 >= plan - 1e-3) {
        return None;
    }
    let tick = 1.0 / DRIVE_TICKS_PER_SECOND;
    let country = w.travel.country_speed;
    let look = (v0 * v0 / (2.0 * BRAKE) + follow_ahead(v0, country) + 30.0).min(EASE_LOOK);
    let path = Path::new(route, t, look);
    let key = car_first(c);
    let half = car.len / 2.0;
    // (How fast it may go `gap` m behind one going `lead` m/s its way: to keep `room` back from
    // it, slowing by `BRAKE` as that one would; nearer than that, slower than that one, to drop
    // back. Standing, it sets off once there's `SET_OFF` more: not on and stopped by turns behind
    // one still coming to a stop.)
    let room =
        follow_ahead(v0, country) + EASE_ROOM + v0 * tick + if v0 < STANDS { SET_OFF } else { 0.0 };
    let safe = |gap: f32, lead: f32| (lead * lead + 2.0 * BRAKE * (gap - room)).max(0.0).sqrt();
    // (No faster than its way's timing ahead, slowing as it does at `BRAKE` at most: its timing
    // goes at a speed of its own between each two of its points, a step down from one to the next
    // where it slows.)
    let mut v = path.tops().min(v0 + ACCEL * tick);
    let mut behind = None;
    let mut keep = |v: &mut f32, lim: f32, k: Option<u64>| {
        if lim < *v {
            *v = lim;
            behind = k;
        }
    };
    // The car in front of it on its way.
    for o in others.iter().filter(|o| o.ahead > 0.0 && o.first != key) {
        if dist(o.at, car.shown.map_or(o.at, |s| s.0)) > look + half + o.len {
            continue;
        }
        let Some((d, way)) = path.reach(o.at, (car.wid + o.wid) / 2.0 + 0.1) else {
            continue;
        };
        let (s, cos) = o.yaw.sin_cos();
        let along = s * way[0] + cos * way[1];
        if d < half || along < 0.0 {
            continue;
        }
        let other = &w.cars[(o.first & !(1 << 63)) as usize];
        keep(
            &mut v,
            safe(d - half - o.len / 2.0, speed_of(other) * along),
            Some(o.first),
        );
    }
    // A merge: the one merging and those coming along the carriageway, as if both on it
    // already (`merge_order`).
    let merging = w.traffic.merging();
    let mut decided = Vec::new();
    let stop = |d: f32| (2.0 * BRAKE * (d - v0 * tick).max(0.0)).sqrt();
    if merging.contains(&key) {
        if let Some((m, mt)) = merge_at(car) {
            for o in others
                .iter()
                .filter(|o| o.ahead > 0.0 && !merging.contains(&o.first))
            {
                let k = (o.first & !(1 << 63)) as usize;
                if dist(o.at, m) > EASE_LOOK {
                    continue;
                }
                let Some(order) = merge_order(w, c, k, (m, mt)) else {
                    continue;
                };
                if order.new {
                    decided.push((c, o.first, order.ramp_first));
                }
                if order.ramp_first {
                    continue;
                }
                let other = &w.cars[k];
                // (That one first: behind it once it's ahead, short of the taper till then.)
                let lim = if order.by < order.ramp {
                    safe(
                        order.ramp - order.by - half - other.len / 2.0,
                        speed_of(other),
                    )
                } else {
                    stop(order.ramp - TAPER - half)
                };
                keep(&mut v, lim, Some(o.first));
            }
        }
    } else {
        for &o in &merging {
            let k = (o & !(1 << 63)) as usize;
            let Some(at) = w.cars.get(k).and_then(merge_at) else {
                continue;
            };
            if path.reach(at.0, MERGE_NEAR).is_none() {
                continue;
            }
            let Some(order) = merge_order(w, k, c, at) else {
                continue;
            };
            if order.new {
                decided.push((k, key, order.ramp_first));
            }
            if !order.ramp_first {
                continue;
            }
            let other = &w.cars[k];
            keep(
                &mut v,
                safe(
                    order.by - order.ramp - half - other.len / 2.0,
                    speed_of(other),
                ),
                Some(o),
            );
        }
    }
    // The start of a claim it stops at, not holding it.
    let tag = u32::from(home);
    for (k, claim) in route.claims.iter().enumerate() {
        if claim.from <= t + 1.0 || w.traffic.holds_part(key, tag, k) {
            continue;
        }
        let Some(d) = path.at_tick(claim.from) else {
            break;
        };
        if claim.merge || !w.traffic.green_through(claim, t) {
            let lim = stop(d).max(CREEP);
            keep(&mut v, lim, None);
            break;
        }
    }
    let v = v.max(v0 - BRAKE_MOST * tick).max(0.0);
    let v = if v < STANDS { 0.0 } else { v };
    // (As far on as that speed takes it in a tick, along its way's points: no further than its
    // timing would.)
    let step = if v <= 0.0 {
        0.0
    } else if plan < 0.3 || v >= plan {
        1.0
    } else {
        (route.ahead_by(t, v * tick) - t).clamp(0.0, 1.0)
    };
    Some(Eased {
        step,
        speed: if step >= 1.0 { plan } else { v },
        behind,
        decided,
    })
}

/// Car `c`, `t` ticks along its way (`home`: the way back) at a speed of its own (`carried`, on the
/// move last tick or not: `moving`), coming up to a merge's claim it doesn't hold
/// (`traffic::Claim::merge`): it asks for it as soon as it would begin to slow down to stop at its
/// start (`BRAKE`), not only there. Let in by then, it goes on onto the carriageway at its speed;
/// else it slows down to stop there (`ease`), asking on.
pub(crate) fn ask_ahead(
    w: &mut World,
    c: usize,
    t: f32,
    carried: Option<f32>,
    moving: bool,
    home: bool,
) {
    let World {
        cars,
        traffic,
        tick,
        ..
    } = w;
    let Some((h, route)) = cars[c].route.as_ref() else {
        return;
    };
    if *h != home || route.merges.is_empty() {
        return;
    }
    let v0 = carried.unwrap_or(if moving { plan_speed(route, t) } else { 0.0 });
    let tick_s = 1.0 / DRIVE_TICKS_PER_SECOND;
    let reach = v0 * v0 / (2.0 * BRAKE) + 2.0 * v0 * tick_s + ASK_AHEAD;
    let key = car_first(c);
    let tag = u32::from(home);
    let Some(k) = route
        .claims
        .iter()
        .position(|claim| claim.from > t + 1.0)
        .filter(|&k| route.claims[k].merge && !traffic.holds_part(key, tag, k))
    else {
        return;
    };
    let from = route.claims[k].from;
    let path = Path::new(route, t, reach);
    if path.at_tick(from).is_some() {
        traffic.may_go(*tick, key, tag, &route.claims, t, from + 1e-3);
    }
}

/// How much further off than it would begin to slow down to stop at it a car asks for its merge
/// (m, `ask_ahead`).
const ASK_AHEAD: f32 = 5.0;

#[cfg(test)]
mod tests {
    use super::*;

    /// A car standing a hair's breadth short of a point of its way (where its part of the step
    /// there is of no length in f32, but some ticks long) may still set off: the step goes at its
    /// way's speed, as the whole of it does. (It had a speed of none, and a taxi turning off an
    /// avenue stood there for good with nobody to wait for, the queue behind it too: Oatcroft.)
    #[test]
    fn a_car_a_hairs_breadth_short_of_a_point_sets_off() {
        let route = Route {
            points: vec![
                ([506.4, 464.9], 0.0, 870.0, false),
                ([507.020_1, 465.380_4], 0.8, 877.98, false),
                ([507.071_9, 465.424_9], 0.9, 878.474_2, false),
                ([507.130_7, 465.466], 1.0, 878.968_7, false),
                ([507.193_3, 465.501_1], 1.1, 879.463_9, false),
            ],
            ..Default::default()
        };
        let b = route.points[2];
        // (The latest tick short of it where it is, in f32, on it.)
        let mut t = b.2;
        let mut short = None;
        for _ in 0..256 {
            t = f32::from_bits(t.to_bits() - 1);
            if route.at(t).0 != b.0 {
                break;
            }
            short = Some(t);
        }
        let t = short.expect("a tick short of the point where the car is on it");
        let path = Path::new(&route, t, 10.0);
        assert!(
            path.0[0].0 == path.0[0].1,
            "its first step there is of no length"
        );
        let speed = plan_speed(&route, b.2);
        assert!(
            path.tops() >= speed * 0.9,
            "it may go {} m/s there, not {speed}",
            path.tops()
        );
    }
}
