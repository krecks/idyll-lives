//! A car's route: the legs of it and the samples along each.

use super::*;
use crate::travel::Way;

/// A drive, sampled every `STEP` metres: where the car's middle is, which way it faces, and when
/// it gets there (ticks from the start).
#[derive(Debug, Clone, Default)]
pub(crate) struct Route {
    pub(crate) points: Vec<([f32; 2], f32, f32, bool)>,
    /// At each point, how far right of its lane a car can pull over there to pass one coming the
    /// other way (m, `pull_room`; none off the lanes, turning or backing).
    pub(super) room: Vec<f32>,
    /// At each point, how far right of the road's middle its lane is (m, `lane_on`, `LaneLine`;
    /// off the lanes, `LANE`). None where nobody comes the other way in the road (`meet`).
    pub(super) lane: Vec<Option<f32>>,
    /// How far it can pull over to pass one coming the other way (m, `room`), ticks along it, as
    /// its claims have it.
    pub(super) pass: Vec<(f32, f32)>,
    /// Where it turns out of a driveway, a parking bay or a spot on the road onto the lane, or off
    /// it into one (ticks along it, from and to): claims of their own (`traffic::claims`).
    pub(super) mouths: Vec<(f32, f32)>,
    /// Where it must hold a claim to drive on (`traffic::Claim`): junctions' boxes and the mouths,
    /// worked out with the route (`Route::build`).
    pub(crate) claims: Vec<crate::traffic::Claim>,
    /// Where it stops for those on foot at the zebras of the junctions giving way it goes through,
    /// should anyone be on one or stepping onto it (`traffic::zebra_stops`, `zebra_yield`), and at
    /// a lit junction's side road it turns into on the green, for anyone on its crosswalk
    /// (`traffic::side_zebra_stops`), worked out with its claims.
    pub(crate) zebras: Vec<crate::traffic::ZebraStop>,
    /// Where it takes turns with one passing a work taxi standing at the kerb the other way
    /// (ticks along it, from and to: `pass_stands`), claims of their own.
    pub(super) spans: Vec<(f32, f32)>,
    /// Its way out round a work taxi standing at the kerb in its lane as laid on it (`PassBy`),
    /// with what it took the place of, to put back (`Leg`).
    pub(super) leg: Option<Leg>,
    /// The taxis standing it was last looked at against (`Traffic::stand_gen`; 0: never).
    pub(super) stand_gen: u64,
    /// It goes out of town (or comes in) by a motorway (`quickest_out`).
    pub(super) by_motorway: bool,
    /// Where it's off the ground (ticks along it, from and to, and the level: `travel::Layer`):
    /// on a motorway's deck, or about a merge on one. Elsewhere on the ground (`layer_at`).
    pub(crate) layers: Runs<crate::travel::Layer>,
    /// Where it merges off a ramp into a motorway's carriageway (ticks along it: at the merge),
    /// each a claim of its own (`MERGE_UP`, `MERGE_ON`).
    pub(crate) merges: Vec<f32>,
    /// Where it's on an avenue, a motorway or a ramp (ticks along it, from and to, and which:
    /// `RoadKind::fast`): there it speeds up and slows down as a car does (`Car::speed`).
    /// Elsewhere on a street, a lane, a country road or off the roads (`kind_at`).
    pub(crate) kinds: Runs<crate::travel::RoadKind>,
}

/// A car's way round a taxi standing at the kerb (`PassBy`) as laid on its route: the points
/// from `at` on it took the place of, and their rooms and lanes (`Route::room`, `Route::lane`),
/// and how many it laid there.
#[derive(Debug, Clone)]
pub(super) struct Leg {
    pub(super) pass: PassBy,
    at: usize,
    laid: usize,
    points: Vec<([f32; 2], f32, f32, bool)>,
    pub(super) room: Vec<f32>,
    pub(super) lane: Vec<Option<f32>>,
}

impl Route {
    /// The route on the town's roads (`on_roads`) where it has a network the car's driveway
    /// meets, else `legacy` (to a road out of town).
    pub(crate) fn build(
        car: &Car,
        exits: &[[f32; 2]],
        travel: Option<&Travel>,
        home: bool,
    ) -> Route {
        let mut route = match travel.filter(|t| !t.graph.is_empty()) {
            Some(t) => Self::on_roads(car, exits, t, home),
            None => None,
        }
        .unwrap_or_else(|| Self::off_roads(car, exits, home));
        route.claim(car.len, car.wid, travel);
        route
    }

    /// Its claims (`Route::claims`), for a car `len` by `wid`: the junctions' boxes it drives
    /// through, its mouths, and where it couldn't pass one coming the other way pulled over (a 6 m
    /// lane's tight bend: `traffic::narrow`).
    pub(crate) fn claim(&mut self, len: f32, wid: f32, travel: Option<&Travel>) {
        let trace = crate::traffic::trace(|t| self.at(t), self.ticks());
        // (Pulling over to pass one coming the other way, out of the junctions' boxes only:
        // `pull_want`.)
        let boxes = travel.map_or(&[][..], |t| &t.graph.boxes[..]);
        let pass: Vec<f32> = trace
            .iter()
            .map(|s| {
                if in_box(boxes, s.1, len) {
                    0.0
                } else {
                    self.room_here(s.0)
                }
            })
            .collect();
        let swing = crate::traffic::swings(&trace, len);
        let over: Vec<f32> = trace
            .iter()
            // (None coming the other way: nobody to overlap.)
            .map(|s| {
                self.lane_at(s.0)
                    .map_or(f32::NEG_INFINITY, |l| wid - 2.0 * l)
            })
            .collect();
        // (A mouth from where its body first comes onto the road out of the driveway or the bay,
        // and to where it's off it into one: backing down a driveway or parking in it, it's in
        // nobody's way.)
        let on_road = |k: usize| {
            let Some(t) = travel.filter(|t| !t.graph.is_empty()) else {
                return true;
            };
            let (_, p, yaw, ..) = trace[k];
            let (s, c) = yaw.sin_cos();
            [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)]
                .into_iter()
                .any(|(a, b)| {
                    let q = [
                        p[0] + s * a * len / 2.0 + c * b * wid / 2.0,
                        p[1] + c * a * len / 2.0 - s * b * wid / 2.0,
                    ];
                    t.graph.near(q, 8.0, 1, Mode::Car).first().is_some_and(|r| {
                        r.off < kerb_off(&t.graph.roads[r.road as usize]) + KERB_SLACK
                    })
                })
        };
        let n = trace.len();
        let mut more: Vec<(f32, f32)> = self
            .mouths
            .iter()
            .map(|&(a, b)| {
                let k = trace.partition_point(|s| s.0 < a);
                let j = trace.partition_point(|s| s.0 < b).min(n - 1);
                if a <= 0.0 {
                    let first = (k..=j).find(|&i| on_road(i)).unwrap_or(j);
                    // (Till it's going along its lane, not turning: easing out to it, it follows
                    // its lane as any car does.)
                    let lane = trace[j].2;
                    let along = (first..=j)
                        .find(|&i| {
                            !trace[i].3 && crate::world::wrap_angle(trace[i].2 - lane).abs() < 0.1
                        })
                        .unwrap_or(j);
                    (
                        trace[first.saturating_sub(1)].0,
                        trace[(along + 1).min(j)].0,
                    )
                } else if j + 1 >= n {
                    let last = (k..=j).rev().find(|&i| on_road(i)).unwrap_or(k);
                    (a, trace[(last + 1).min(n - 1)].0)
                } else {
                    (a, b)
                }
            })
            .collect();
        if travel.is_some() {
            more.extend(crate::traffic::narrow(&trace, &pass, &swing, &over));
        }
        // (By a taxi standing at the kerb: alongside where one passes it the other way, begun as
        // far short as a narrow stretch's, so it waits there; and its own way round it.)
        more.extend(crate::traffic::back_by(
            &trace,
            self.spans.clone(),
            crate::traffic::NARROW_BACK,
        ));
        if let Some(l) = &self.leg {
            more.push((l.pass.from, l.pass.to));
        }
        // (Each pose's level: on a deck it goes by the junctions under it, their boxes, lights
        // and zebras.)
        let layers: Vec<crate::travel::Layer> = if self.layers.is_empty() {
            Vec::new()
        } else {
            trace.iter().map(|s| self.layer_at(s.0)).collect()
        };
        // (Onto a motorway off a ramp: from `MERGE_UP` up the ramp to `MERGE_ON` past the merge,
        // a claim of its own, asked for with room in the carriageway's traffic, `traffic::Claim`'s
        // `merge`.)
        let merges: Vec<(f32, f32)> = self
            .merges
            .iter()
            .filter_map(|&m| {
                let n = trace.len();
                let k = trace.partition_point(|s| s.0 < m).min(n.checked_sub(1)?);
                let d = trace[k].4;
                let from = trace[trace.partition_point(|s| s.4 < d - MERGE_UP).min(k)].0;
                let to = trace[trace.partition_point(|s| s.4 <= d + MERGE_ON).min(n - 1)].0;
                (to > from).then_some((from, to))
            })
            .collect();
        more.extend(&merges);
        self.claims = crate::traffic::claims(&trace, &pass, len, wid, boxes, &more, &layers);
        for c in &mut self.claims {
            c.merge = merges.iter().any(|&(a, b)| c.from < b && a < c.to);
        }
        if let Some(t) = travel {
            crate::traffic::gate(&mut self.claims, &t.signals, &trace, len);
            // (Its poses on the ground: every one, but where it's off it.)
            let ground: std::borrow::Cow<crate::traffic::Trace> = if layers.is_empty() {
                std::borrow::Cow::Borrowed(&trace)
            } else {
                std::borrow::Cow::Owned(
                    trace
                        .iter()
                        .zip(&layers)
                        .filter(|(_, l)| l.ground())
                        .map(|(s, _)| *s)
                        .collect(),
                )
            };
            self.zebras = crate::traffic::zebra_stops(&ground, &t.give_way, len);
            self.zebras
                .extend(crate::traffic::side_zebra_stops(&ground, &t.signals, len));
            self.zebras.sort_by(|a, b| a.at.total_cmp(&b.at));
        }
        self.pass = trace.iter().zip(&pass).map(|(s, &p)| (s.0, p)).collect();
    }

    /// How far it can pull over to pass one coming the other way `t` ticks along (m: `pass`).
    pub(crate) fn pass_at(&self, t: f32) -> f32 {
        let k = self.pass.partition_point(|p| p.0 <= t);
        self.pass.get(k.saturating_sub(1)).map_or(0.0, |p| p.1)
    }

    /// Lays `pass` on it (`PassBy`): its points from `pass.from` to `pass.to` a tick apart, each
    /// moved its share of `pass.off` to its right, facing on to the next, with no room to pull
    /// over. False: no such stretch on it (backing up there, or past its end).
    pub(super) fn lay_leg(&mut self, pass: PassBy) -> bool {
        let n = self.points.len();
        if self.leg.is_some()
            || n < 2
            || self.room.len() != n
            || self.lane.len() != n
            || !(pass.from < pass.out && pass.out <= pass.back && pass.back < pass.to)
            || pass.to + 1.0 >= self.ticks()
        {
            return false;
        }
        let share = |t: f32| {
            let ease = |x: f32| {
                let x = x.clamp(0.0, 1.0);
                x * x * (3.0 - 2.0 * x)
            };
            if t < pass.out {
                ease((t - pass.from) / (pass.out - pass.from))
            } else if t <= pass.back {
                1.0
            } else {
                1.0 - ease((t - pass.back) / (pass.to - pass.back))
            }
        };
        let mut ts: Vec<f32> = (0..)
            .map(|i| pass.from + i as f32)
            .take_while(|&t| t < pass.to)
            .collect();
        ts.push(pass.to);
        let mut laid = Vec::with_capacity(ts.len());
        let (mut room, mut lane) = (Vec::new(), Vec::new());
        for &t in &ts {
            let (p, yaw, back) = self.at(t);
            if back {
                return false;
            }
            laid.push((shifted(p, yaw, pass.off * share(t)), yaw, t, false));
            room.push(0.0);
            lane.push(self.lane_at(t));
        }
        // (Facing on to the next, the last as the one before it.)
        for k in 0..laid.len() {
            let (a, b) = if k + 1 < laid.len() {
                (laid[k].0, laid[k + 1].0)
            } else {
                (laid[k - 1].0, laid[k].0)
            };
            laid[k].1 = (b[0] - a[0]).atan2(b[1] - a[1]);
        }
        let i0 = self.points.partition_point(|p| p.2 < pass.from);
        let i1 = self.points.partition_point(|p| p.2 <= pass.to);
        let count = laid.len();
        let points = self.points.splice(i0..i1, laid).collect();
        let room = self.room.splice(i0..i1, room).collect();
        let lane = self.lane.splice(i0..i1, lane).collect();
        self.leg = Some(Leg {
            pass,
            at: i0,
            laid: count,
            points,
            room,
            lane,
        });
        true
    }

    /// Its way round a standing taxi taken off it again (`lay_leg`), as it was.
    pub(super) fn unlay_leg(&mut self) {
        if let Some(l) = self.leg.take() {
            let r = l.at..l.at + l.laid;
            self.points.splice(r.clone(), l.points);
            self.room.splice(r.clone(), l.room);
            self.lane.splice(r, l.lane);
        }
    }

    /// The point after the one `t` ticks along lies past, when `points` and the per-point array a
    /// caller reads (`lens` of them) agree: `Some(i)`, with `i - 1` and `i` the two either side.
    /// None: too few points to have a span, or the array doesn't match them.
    fn span(&self, lens: usize, t: f32) -> Option<usize> {
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
    fn lane_at(&self, t: f32) -> Option<f32> {
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

    /// `legacy`, to a road out of town.
    pub(super) fn off_roads(car: &Car, exits: &[[f32; 2]], home: bool) -> Route {
        Self::legacy(car, exits, home)
    }

    #[cfg(test)]
    pub(crate) fn new(car: &Car, exits: &[[f32; 2]], home: bool) -> Route {
        Self::legacy(car, exits, home)
    }

    /// Out of the driveway onto the road it meets and along the roads to the way out of town
    /// quickest by car (`quickest_out`: a motorway's end of town counts what the motorway saves
    /// beyond it), past it (`OVERSHOOT`), or (`home`) back in from there, nose first: the way out
    /// turned round where every road along it is two-way, else a way of its own in from there.
    /// The driveway's part as `legacy` lays it out, in the frame of the road at the driveway's
    /// mouth; the lanes right of the roads' middle lines (`LaneLine`); each road's top speed
    /// (`lane_tops`), slower at junctions. Sampled more where it bends (`ROUTE_TOLERANCE`). None:
    /// no road near, or no way out by road.
    pub(super) fn on_roads(
        car: &Car,
        exits: &[[f32; 2]],
        travel: &Travel,
        home: bool,
    ) -> Option<Route> {
        use crate::travel::{RoadPath, dist};
        let g = &travel.graph;
        let m = car.mouth(travel)?;
        let mp = g.point(m);
        // Where to: the parking spot of its trip, or the quickest way out by car. Home, the way
        // there turned round where every road along it is two-way, else (one-way roads) a way of
        // its own from there to the mouth; either one laid from the mouth on (and turned round
        // below).
        let speed = travel.country_speed;
        let mut by_motorway = false;
        // (On an avenue, out of its driveway and its trip's bay to the right only, and into them
        // from the right: `kerbside_way`.)
        let side = kerbside_way(g, m, car.spot);
        let bay_side = car
            .trip
            .and_then(|trip| kerbside_way(g, enterable_near(g, trip.at, ROAD_REACH)?, trip.at));
        let out = |to: RoadPoint| {
            let arrive = car.trip.and_then(|_| bay_side.map(|s| s.1));
            kerbside_path(g, m, to, speed, side.map(|s| s.0), arrive)
        };
        let back = |to: RoadPoint| {
            let toward = car.trip.and_then(|_| bay_side.map(|s| s.0));
            let mut path = kerbside_path(g, to, m, speed, toward, side.map(|s| s.1))?;
            path.reverse();
            Some(path)
        };
        // (Not on an avenue at either end: turned round, it'd come in across its median.)
        let kerbside = side.is_some() || bay_side.is_some();
        let turns_round =
            |path: Option<&RoadPath>| !kerbside && path.is_some_and(|p| two_way(g, p));
        let (path, park) = match car.trip {
            Some(trip) => {
                let p = enterable_near(g, trip.at, ROAD_REACH)?;
                let mut path = out(p);
                if home && !turns_round(path.as_ref()) {
                    path = back(p);
                }
                (path?, Some((trip.at, trip.yaw)))
            }
            None => {
                let mut path = quickest_out(g, exits, out);
                if home && !turns_round(path.as_ref().map(|p| &p.0)) {
                    path = quickest_out(g, exits, back);
                }
                let (path, by) = path?;
                by_motorway = by;
                (path, None)
            }
        };
        if path.pts.len() < 2 {
            return None;
        }
        // The driveway, and which way along the road it goes from it.
        let drive = Bay::new(
            car.spot,
            car.yaw,
            car.axis,
            mp,
            &g.roads[m.road as usize],
            m.s,
        )?;
        let ahead = drive.local(path.pts[(path.pts.len() - 1).min(3)].0);
        let d = if ahead[0] >= 0.0 { 1.0 } else { -1.0 };
        // The lane: right of the middle line, on past the exit (to the parking spot's side of
        // the road on a trip).
        let mut centre: Vec<[f32; 2]> = path.pts.iter().map(|p| p.0).collect();
        let mut on = path.roads.clone();
        if park.is_none() {
            centre.push(extend_past(
                centre[centre.len() - 2],
                centre[centre.len() - 1],
            ));
            on.push(on[on.len() - 1]);
        }
        if home {
            centre.reverse();
            on.reverse();
            for o in &mut on {
                o.1 = !o.1;
            }
        }
        let laid = LaneLine::new(g, centre, on).laid(g, car.wid);
        let lane = &laid.lane;
        if lane.len() < 2 {
            return None;
        }
        let tops = lane_tops(lane, travel, |i| laid.here[i].kind);
        // A trip's parking bay off the road (`parking_bay`), turned into nose first and backed
        // out of as a driveway is, and which way along the road it goes there (from the lane's
        // nearer end); a spot on the road itself is pulled into and out of straight.
        let bay = park.and_then(|(at, yaw)| {
            let (rp, on, to_road) = parking_bay(travel, at)?;
            let road = &g.roads[on.road as usize];
            let rt = road.tangent(on.s);
            let axis = bay_axis(yaw, to_road);
            let (a, b) = if home {
                (lane[0], lane[1])
            } else {
                (lane[lane.len() - 2], lane[lane.len() - 1])
            };
            let go = (b[0] - a[0]) * rt[0] + (b[1] - a[1]) * rt[1];
            Some((
                Bay::new(at, yaw, Some(axis), rp, road, on.s)?,
                if go >= 0.0 { 1.0 } else { -1.0 },
            ))
        });
        let mut legs = Legs::default();
        // Onto the lane: out of the driveway, or (home from a trip) out of the parking bay, or
        // pulling out of a spot on the road; or (home from work) at the lane's start, past the
        // way into town. The turn there and its place along the road, for the lane's points
        // before it to be left out.
        let start: Option<(&Bay, f32, f32, f32)> = if !home {
            let (at, taken) = drive.leave(&mut legs, d);
            Some((&drive, d, at, taken))
        } else if let Some((bay, go)) = &bay {
            let (at, taken) = bay.leave(&mut legs, *go);
            Some((bay, *go, at, taken))
        } else {
            if let Some((at, _)) = park {
                legs.push(at, false, SLOW);
            }
            None
        };
        // Off it: into the driveway, or into the parking bay (coming the other way).
        let into = if home {
            Some((&drive, d))
        } else {
            bay.as_ref().map(|(bay, go)| (bay, -go))
        }
        .map(|(bay, d)| (bay, d, bay.entry(d)));
        let mut on = false;
        // (Which way the lane goes at its point `i`, and whether that's along the road at a bay's
        // mouth going `d`: past a junction there it turns off, and its points on aren't the turn's.)
        let way = |i: usize| {
            let (a, b) = (lane[i.saturating_sub(1)], lane[(i + 1).min(lane.len() - 1)]);
            let l = dist(a, b).max(1e-6);
            [(b[0] - a[0]) / l, (b[1] - a[1]) / l]
        };
        let along = |b: &Bay, i: usize, d: f32| {
            let w = way(i);
            (w[0] * b.t[0] + w[1] * b.t[1]) * d > 0.7
        };
        for (i, &p) in lane.iter().enumerate() {
            if !on
                && let Some((b, d, at, taken)) = start
                && b.near(p, at, taken, d)
                && along(b, i, d)
            {
                continue;
            }
            if let Some((b, d, (turn, at))) = &into
                && b.near(p, *at, turn.taken, *d)
                && along(b, i, -*d)
            {
                break;
            }
            let first = !on;
            if first && start.is_some() {
                let from = legs.last().unwrap_or(p);
                legs.line(from, p, false, JUNCTION_SPEED);
            }
            on = true;
            let here = laid.here[i];
            legs.push_marked(
                p,
                tops[i],
                (here.room - bend_out(lane, i, car.len)).max(0.0),
                here.lane,
                here.kind,
                laid.mark(i),
            );
            // (Out of a driveway, a bay or a spot on the road: its mouth, up to the lane.)
            if first && (start.is_some() || park.is_some() && home) {
                let to = legs.len() - 1;
                legs.mouths.push((0, Some(to)));
            }
        }
        // (Never on the lane, from the driveway straight into the bay by it: all of it a mouth.)
        if !on && (start.is_some() || park.is_some() && home) {
            legs.mouths.push((0, None));
        }
        // (Into a driveway, a bay or a spot on the road: its mouth, from the lane.)
        let lane_end = legs.len().saturating_sub(1);
        match into {
            Some((b, _, (turn, _))) => {
                legs.mouths.push((lane_end, None));
                b.enter(&mut legs, &turn)
            }
            // (On a trip to a spot on the road: into it at the end.)
            None => {
                if let (Some((at, _)), false) = (park, home) {
                    legs.mouths.push((lane_end, None));
                    let from = legs.last().unwrap_or(at);
                    legs.line(from, at, false, SLOW);
                }
            }
        }
        let trip = park.is_some();
        let mut route = legs.timed(!home || trip, home || trip);
        route.by_motorway = by_motorway;
        Some(route)
    }

    /// Its way turned round (`Detour`): back along the way it had (`on_roads`) from where it was
    /// held up to where it turns (`Detour::back`), then on along the lanes from there, setting off
    /// the way it faces, round the stretch it was held up on (`Detour::avoid`), to where it was
    /// going: into its driveway (home), its trip's parking spot, or out of town. None: no such
    /// way (only by that stretch, say).
    pub(super) fn detour(
        car: &Car,
        exits: &[[f32; 2]],
        travel: &Travel,
        home: bool,
        d: &Detour,
        had: &Route,
    ) -> Option<Route> {
        use crate::travel::dist;
        let g = &travel.graph;
        // (Into a driveway's mouth it may back from where it stands: `mouth_behind`.)
        let backs = d.back < d.from || d.mouth.is_some() && d.back <= d.from;
        if !(backs && d.from <= had.ticks()) {
            return None;
        }
        let mut legs = Legs::default();
        // Back along the way it came, as it drove it (round the bends and turns it took).
        let mut t = d.from;
        loop {
            // (Pulled over as far as the way it came lets it, as it was: `pull_over`.)
            legs.push_back(had.at(t).0, had.room_here(t), had.lane_at(t));
            legs.mark_last((had.layer_at(t), false));
            if let Some(k) = had.kind_at(t) {
                legs.mark_kind(k);
            }
            if t <= d.back {
                break;
            }
            t = (t - 1.0).max(d.back);
        }
        // (Backed up out of the way only: on the way it had from there, as it had it.)
        if d.same {
            let i0 = had.points.partition_point(|q| q.2 <= d.back);
            for i in i0..had.points.len() {
                let (q, _, t, rev) = had.points[i];
                let top = had.points.get(i + 1).map_or(SLOW, |n| {
                    let dt = (n.2 - t).max(1e-3) / DRIVE_TICKS_PER_SECOND;
                    (dist(q, n.0) / dt).max(0.3)
                });
                legs.pts.push((q, rev, top));
                legs.room.push(had.room.get(i).copied().unwrap_or(0.0));
                legs.lane
                    .push(had.lane.get(i).copied().unwrap_or(Some(LANE)));
                let merge = had.merges.iter().any(|&m| (m - t).abs() < 1e-3);
                legs.mark_last((had.layer_at(t), merge));
                if let Some(k) = had.kind_at(t) {
                    legs.mark_kind(k);
                }
            }
            return Some(legs.timed(true, false));
        }
        // (Into a driveway's mouth backing up, round as it would pull out of it, and out of it the
        // other way.)
        let turned = match d.mouth {
            Some(m) => {
                let (bay, go) = (mouth_turn(&m, travel, car.len)?, m.go);
                let mut into = Legs::default();
                bay.leave(&mut into, go);
                for &(q, ..) in into.pts.iter().rev() {
                    legs.push(q, true, SLOW);
                }
                bay.leave(&mut legs, -go);
                let n = legs.pts.len();
                let (a, b) = (legs.pts[n.saturating_sub(2)].0, legs.pts[n - 1].0);
                Some((b, (b[0] - a[0]).atan2(b[1] - a[1])))
            }
            None => None,
        };
        // On from there, the way it faces.
        let (p, yaw) = turned.unwrap_or_else(|| {
            let (p, yaw, _) = had.at(d.back);
            (p, yaw)
        });
        let rp = *g.near(p, ROAD_REACH, 1, Mode::Car).first()?;
        let e = g.edges[rp.edge as usize];
        let tan = g.roads[rp.road as usize].tangent(rp.s);
        let toward = if tan[0] * yaw.sin() + tan[1] * yaw.cos() >= 0.0 {
            e.b
        } else {
            e.a
        };
        let mut avoid = vec![(d.avoid, DETOUR_AVOID)];
        avoid.extend(
            car.avoided
                .iter()
                .filter(|a| a.0 != d.avoid)
                .map(|a| (a.0, DETOUR_AVOID)),
        );
        let way = Way {
            toward: Some(toward),
            arrive: None,
            avoid,
        };
        let speed = travel.country_speed;
        // (A work taxi, its halts behind it: out of town where its run goes, `taxi/`.)
        let taxi = car.taxi;
        let ends = if taxi {
            vec![had.points.last()?.0]
        } else {
            exits.to_vec()
        };
        let m = car.mouth(travel);
        // (Into a driveway or a bay on an avenue from the right only, `kerbside_way`; else any
        // way in.)
        let arrive = |to: RoadPoint, spot: [f32; 2]| {
            let into = Way {
                arrive: kerbside_way(g, to, spot).map(|s| s.1),
                ..way.clone()
            };
            g.path_by(rp, to, Mode::Car, speed, into)
                .or_else(|| g.path_by(rp, to, Mode::Car, speed, way.clone()))
        };
        let (path, park) = if home && !taxi {
            (arrive(m?, car.spot)?, None)
        } else if let Some(trip) = car.trip.filter(|_| !taxi) {
            let to = enterable_near(g, trip.at, ROAD_REACH)?;
            (arrive(to, trip.at)?, Some((trip.at, trip.yaw)))
        } else {
            // (Weighed with what going along the stretches it keeps off costs it more, as the
            // search has it: by its cost alone it went back the way it turned round from, the way
            // out that way as near as another.)
            let weight = |p: &crate::travel::RoadPath| {
                p.cost
                    + p.edges
                        .iter()
                        .filter_map(|e| way.avoid.iter().find(|a| a.0 == *e))
                        .map(|a| a.1)
                        .sum::<f32>()
            };
            ends.iter()
                .flat_map(|&x| {
                    if taxi {
                        g.near(x, ROAD_REACH, 1, Mode::Car)
                            .into_iter()
                            .map(|e| (e, 0.0))
                            .collect()
                    } else {
                        exit_points(g, x)
                    }
                })
                .filter_map(|(e, saves)| {
                    let path = g.path_by(rp, e, Mode::Car, speed, way.clone())?;
                    Some((weight(&path) - saves, path))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, p)| (p, None))?
        };
        // (Round where it was held up if there's another way; else the same way, once those it
        // backed up out of the way of have gone by.)
        if path.pts.len() < 2 {
            return None;
        }
        let mut centre: Vec<[f32; 2]> = path.pts.iter().map(|p| p.0).collect();
        let mut on = path.roads.clone();
        let home = home && !taxi;
        if !home && park.is_none() {
            centre.push(extend_past(
                centre[centre.len() - 2],
                centre[centre.len() - 1],
            ));
            on.push(on[on.len() - 1]);
        }
        let laid = LaneLine::new(g, centre, on).laid(g, car.wid);
        let lane = &laid.lane;
        if lane.len() < 2 {
            return None;
        }
        let tops = lane_tops(lane, travel, |i| laid.here[i].kind);
        // Off the lane at the end: into its driveway, coming from the side the way comes in
        // from; or into its trip's parking bay.
        let into = if home {
            let m = m?;
            let drive = Bay::new(
                car.spot,
                car.yaw,
                car.axis,
                g.point(m),
                &g.roads[m.road as usize],
                m.s,
            )?;
            let n = path.pts.len();
            let side = drive.local(path.pts[n - 1 - (n - 1).min(3)].0);
            Some((drive, if side[0] >= 0.0 { 1.0 } else { -1.0 }))
        } else {
            park.and_then(|(at, yaw)| {
                let (rp, on, to_road) = parking_bay(travel, at)?;
                let road = &g.roads[on.road as usize];
                let rt = road.tangent(on.s);
                let axis = bay_axis(yaw, to_road);
                let (a, b) = (lane[lane.len() - 2], lane[lane.len() - 1]);
                let go = (b[0] - a[0]) * rt[0] + (b[1] - a[1]) * rt[1];
                Some((
                    Bay::new(at, yaw, Some(axis), rp, road, on.s)?,
                    if go >= 0.0 { -1.0 } else { 1.0 },
                ))
            })
        }
        .map(|(bay, d)| {
            let entry = bay.entry(d);
            (bay, d, entry)
        });
        let way = |i: usize| {
            let (a, b) = (lane[i.saturating_sub(1)], lane[(i + 1).min(lane.len() - 1)]);
            let l = dist(a, b).max(1e-6);
            [(b[0] - a[0]) / l, (b[1] - a[1]) / l]
        };
        let along = |b: &Bay, i: usize, d: f32| {
            let w = way(i);
            (w[0] * b.t[0] + w[1] * b.t[1]) * d > 0.7
        };
        // (Onto the lane at its first point ahead of where it turned round, the way it faces
        // there, not across to one right beside it or behind it: a car doesn't step sideways,
        // and facing along such a step it swung its tail half across the road as it drove on,
        // into the way of one coming the other way on a stretch too narrow for more, both held up
        // for good, Eaglescar. One further off it drives to as it would.)
        let from = legs.last().unwrap_or(lane[0]);
        let facing = [yaw.sin(), yaw.cos()];
        let join = (0..lane.len() - 1)
            .find(|&i| {
                let off = dist(from, lane[i]);
                off >= JOIN_NEAR || dot2(sub2(lane[i], from), facing) >= JOIN_COS * off
            })
            .unwrap_or(lane.len() - 1);
        legs.line(from, lane[join], false, JUNCTION_SPEED);
        for (i, &q) in lane.iter().enumerate() {
            if i < join {
                continue;
            }
            if let Some((b, d, (turn, at))) = &into
                && b.near(q, *at, turn.taken, *d)
                && along(b, i, -*d)
            {
                break;
            }
            let here = laid.here[i];
            legs.push_marked(
                q,
                tops[i],
                (here.room - bend_out(lane, i, car.len)).max(0.0),
                here.lane,
                here.kind,
                laid.mark(i),
            );
        }
        let lane_end = legs.len().saturating_sub(1);
        match &into {
            Some((b, _, (turn, _))) => {
                legs.mouths.push((lane_end, None));
                b.enter(&mut legs, turn);
            }
            None => {
                if let Some((at, _)) = park {
                    legs.mouths.push((lane_end, None));
                    let from = legs.last().unwrap_or(at);
                    legs.line(from, at, false, SLOW);
                }
            }
        }
        Some(legs.timed(true, home || park.is_some()))
    }

    /// Out of the driveway to the nearest end of town along its street, or (`home`) back in
    /// from there, nose first: straight along x (towns with no roads).
    fn legacy(car: &Car, exits: &[[f32; 2]], home: bool) -> Route {
        let [x0, z0] = car.spot;
        // The side of the street the driveway is on, and the way out of town.
        let s = if car.street >= z0 { 1.0 } else { -1.0 };
        let on_street: Vec<[f32; 2]> = exits
            .iter()
            .copied()
            .filter(|e| (e[1] - car.street).abs() < 1.5)
            .collect();
        let near = |list: &[[f32; 2]]| {
            list.iter()
                .copied()
                .min_by(|a, b| (a[0] - x0).abs().total_cmp(&(b[0] - x0).abs()))
        };
        let exit = near(&on_street)
            .or_else(|| near(exits))
            .unwrap_or([0.0, car.street]);
        let d = if exit[0] >= x0 { 1.0 } else { -1.0 };
        let end_x = exit[0] + d * OVERSHOOT;
        // Drive on the right: heading +x, that's the -z side (`right_of`).
        let lane_out = car.street - d * TIGHT_LANE;
        let lane_in = car.street + d * TIGHT_LANE;
        let mut legs = Legs::default();
        if home {
            let r = TURN.min((lane_in - z0).abs()).max(0.5);
            legs.line([end_x, lane_in], [x0 + d * r, lane_in], false, FAST);
            legs.mouths.push((legs.len() - 1, None));
            legs.arc(
                [x0 + d * r, lane_in - s * r],
                r,
                [x0 + d * r, lane_in],
                [x0, lane_in - s * r],
                false,
            );
            legs.line([x0, lane_in - s * r], [x0, z0], false, SLOW);
        } else {
            let r = TURN.min((lane_out - z0).abs()).max(0.5);
            // Facing the street it pulls out; facing the house it backs out, turning away from
            // the way it goes, then drives off past the driveway.
            let facing_street = (car.yaw.cos() * s) > 0.0;
            let k = if facing_street { d } else { -d };
            let reverse = !facing_street;
            legs.line([x0, z0], [x0, lane_out - s * r], reverse, SLOW);
            legs.arc(
                [x0 + k * r, lane_out - s * r],
                r,
                [x0, lane_out - s * r],
                [x0 + k * r, lane_out],
                reverse,
            );
            legs.mouths.push((0, Some(legs.len() - 1)));
            legs.line([x0 + k * r, lane_out], [end_x, lane_out], false, FAST);
        }
        legs.timed(!home, home)
    }

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
}

/// A merge off a ramp onto a motorway's carriageway (`Route::merges`) is claimed from this far up
/// the ramp (m: short of its taper, `TAPER`, where the car waits for its gap) to this far past the
/// merge (m).
pub(crate) const MERGE_UP: f32 = 40.0;
pub(crate) const MERGE_ON: f32 = 20.0;

/// What leaving town by a motorway saves, the drive beyond the end of town being shorter
/// (`MOTORWAY_OFF_MAP_TICKS`): seconds of drive time, taken off the drive to a motorway's end of
/// town when a car weighs its ways out (`quickest_out`).
const MOTORWAY_SAVES: f32 =
    (OFF_MAP_TICKS - MOTORWAY_OFF_MAP_TICKS) as f32 / DRIVE_TICKS_PER_SECOND;

/// How near its end a motorway's point by a way out of town is (m): where a carriageway leaves
/// town, not one passing by (`exit_points`).
const MOTORWAY_END: f32 = 2.0;

/// Where a car leaves town by the way out at `x`: the road nearest it, and a motorway's
/// carriageways ending there besides (both of them come near a way out; the path search takes
/// only the one going out, `Graph::path`), each with what leaving by it saves (s of drive time:
/// `MOTORWAY_SAVES` by a motorway).
pub(crate) fn exit_points(g: &crate::travel::Graph, x: [f32; 2]) -> Vec<(RoadPoint, f32)> {
    use crate::travel::RoadKind;
    let Some(&first) = g.near(x, ROAD_REACH, 1, Mode::Car).first() else {
        return Vec::new();
    };
    let motorway_end = |p: &RoadPoint| {
        let road = &g.roads[p.road as usize];
        road.kind == RoadKind::Motorway && (p.s < MOTORWAY_END || p.s > road.len() - MOTORWAY_END)
    };
    let mut points = vec![first];
    points.extend(
        g.near(x, ROAD_REACH, EXIT_LOOK, Mode::Car)
            .into_iter()
            .filter(|p| p.road != first.road && motorway_end(p)),
    );
    points
        .into_iter()
        .map(|p| {
            (
                p,
                if motorway_end(&p) {
                    MOTORWAY_SAVES
                } else {
                    0.0
                },
            )
        })
        .collect()
}

/// How many roads near a way out `exit_points` looks at: a carriageway's two ends, and one more.
const EXIT_LOOK: usize = 3;

/// The quickest way out of town by `way` (from a car's mouth to each of `exits`' points,
/// `exit_points`; the drive to a motorway's end of town less what the motorway saves), and whether
/// it leaves by a motorway. The first way out of those as quick.
fn quickest_out(
    g: &crate::travel::Graph,
    exits: &[[f32; 2]],
    way: impl Fn(RoadPoint) -> Option<crate::travel::RoadPath>,
) -> Option<(crate::travel::RoadPath, bool)> {
    exits
        .iter()
        .enumerate()
        .flat_map(|(i, &x)| exit_points(g, x).into_iter().map(move |p| (i, p)))
        .filter_map(|(i, (e, saves))| {
            let path = way(e)?;
            Some((i, path.cost - saves, path, saves > 0.0))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
        .map(|(_, _, path, by)| (path, by))
}

/// Where a car whose driveway is at `at` leaves town by `exits`, as `Route::on_roads` chooses
/// (`quickest_out`, from the road a car may come onto there, `enterable_near`): the point of the
/// road it leaves by nearest the way out, and whether that is a motorway. None: no road near, or
/// no way out. (For the tests.)
#[cfg(any(test, feature = "test-support"))]
pub fn way_out(travel: &Travel, exits: &[[f32; 2]], at: [f32; 2]) -> Option<([f32; 2], bool)> {
    let g = &travel.graph;
    let m = enterable_near(g, at, ROAD_REACH)?;
    let side = kerbside_way(g, m, at).map(|s| s.0);
    let out = |to: RoadPoint| kerbside_path(g, m, to, travel.country_speed, side, None);
    let (path, by) = quickest_out(g, exits, out)?;
    Some((path.pts.last()?.0, by))
}

/// On an avenue (two lanes each way about a median), the way a car keeps its driveway or bay at
/// `spot` on its right, `m` where that meets the road: the end of `m`'s stretch it sets off
/// towards, turning right out of it into the near lanes, and the end it comes in from, turning
/// right into it; neither across the median (one on the far side goes on to the next junction and
/// comes back, `kerbside_path`). None on any other road: a two-way street's cars cross the
/// oncoming lane as ever, a one-way road's go its way (`Graph::may_go`).
pub(crate) fn kerbside_way(
    g: &crate::travel::Graph,
    m: RoadPoint,
    spot: [f32; 2],
) -> Option<(u32, u32)> {
    let r = &g.roads[m.road as usize];
    if r.kind != crate::travel::RoadKind::Avenue || !r.oneway.is_both() {
        return None;
    }
    let (p, t) = (r.at(m.s), r.tangent(m.s));
    let r = right_of(t);
    let right = (spot[0] - p[0]) * r[0] + (spot[1] - p[1]) * r[1];
    let e = g.edges[m.edge as usize];
    Some(if right >= 0.0 { (e.b, e.a) } else { (e.a, e.b) })
}

/// The way by car from `from` to `to`, setting off towards `toward` and coming in from `arrive`
/// where given (`kerbside_way`): round by the next junction and back, as the roads have it. With
/// no such way (an avenue's dead end), any way.
fn kerbside_path(
    g: &crate::travel::Graph,
    from: RoadPoint,
    to: RoadPoint,
    speed: f32,
    toward: Option<u32>,
    arrive: Option<u32>,
) -> Option<crate::travel::RoadPath> {
    let any = || g.path_by(from, to, Mode::Car, speed, Way::default());
    if toward.is_none() && arrive.is_none() {
        return any();
    }
    let way = Way {
        toward,
        arrive,
        ..Way::default()
    };
    g.path_by(from, to, Mode::Car, speed, way).or_else(any)
}

/// Whether a car may drive `path` back the way it goes: no road along it is one-way.
fn two_way(g: &crate::travel::Graph, path: &crate::travel::RoadPath) -> bool {
    path.edges
        .iter()
        .all(|&e| g.roads[g.edges[e as usize].road as usize].oneway.is_both())
}
