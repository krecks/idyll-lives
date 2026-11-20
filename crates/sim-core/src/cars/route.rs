//! A car's route: the legs of it and the samples along each.
//!
//! Split from `cars/mod.rs` (see `mod.rs`).

use super::*;

/// A drive, sampled every `STEP` metres: where the car's middle is, which way it faces, and when
/// it gets there (ticks from the start).
#[derive(Debug, Clone, Default)]
pub(crate) struct Route {
    pub(crate) points: Vec<([f32; 2], f32, f32, bool)>,
    /// At each point, how far right of its lane a car can pull over there to pass one coming the
    /// other way (m, `pull_room`; none off the lanes, turning or backing).
    pub(crate) room: Vec<f32>,
    /// At each point, how far right of the road's middle its lane is (m, `lane_on`; off the lanes,
    /// `LANE`).
    pub(crate) lane: Vec<f32>,
    /// How far it can pull over to pass one coming the other way (m, `room`), ticks along it, as
    /// its claims have it.
    pub(crate) pass: Vec<(f32, f32)>,
    /// Where it turns out of a driveway, a parking bay or a spot on the road onto the lane, or off
    /// it into one (ticks along it, from and to): claims of their own (`traffic::claims`).
    pub(crate) mouths: Vec<(f32, f32)>,
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
    pub(crate) spans: Vec<(f32, f32)>,
    /// Its way out round a work taxi standing at the kerb in its lane as laid on it (`PassBy`),
    /// with what it took the place of, to put back (`Leg`).
    pub(crate) leg: Option<Leg>,
    /// The taxis standing it was last looked at against (`Traffic::stand_gen`; 0: never).
    pub(crate) stand_gen: u64,
}

/// A car's way round a taxi standing at the kerb (`PassBy`) as laid on its route: the points
/// from `at` on it took the place of, and their rooms and lanes (`Route::room`, `Route::lane`),
/// and how many it laid there.
#[derive(Debug, Clone)]
pub(crate) struct Leg {
    pub(crate) pass: PassBy,
    at: usize,
    laid: usize,
    points: Vec<([f32; 2], f32, f32, bool)>,
    pub(crate) room: Vec<f32>,
    pub(crate) lane: Vec<f32>,
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
            .map(|s| wid - 2.0 * self.lane_at(s.0))
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
        self.claims = crate::traffic::claims(&trace, &pass, len, wid, boxes, &more);
        if let Some(t) = travel {
            crate::traffic::gate(&mut self.claims, &t.signals, &trace, len);
            self.zebras = crate::traffic::zebra_stops(&trace, &t.give_way, len);
            self.zebras
                .extend(crate::traffic::side_zebra_stops(&trace, &t.signals, len));
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
    pub(crate) fn lay_leg(&mut self, pass: PassBy) -> bool {
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
    pub(crate) fn unlay_leg(&mut self) {
        if let Some(l) = self.leg.take() {
            let r = l.at..l.at + l.laid;
            self.points.splice(r.clone(), l.points);
            self.room.splice(r.clone(), l.room);
            self.lane.splice(r, l.lane);
        }
    }

    /// How far right of its lane a car standing `t` ticks along can pull over (m: the more of the
    /// two points either side: it's on the lane up to the one where its turn off it begins).
    pub(crate) fn room_here(&self, t: f32) -> f32 {
        let pts = &self.points;
        if pts.len() < 2 || self.room.len() != pts.len() {
            return 0.0;
        }
        let i = pts.partition_point(|p| p.2 <= t).clamp(1, pts.len() - 1);
        self.room[i - 1].max(self.room[i])
    }

    /// How far right of the road's middle its lane is `t` ticks along (m: the more of the two
    /// points either side).
    fn lane_at(&self, t: f32) -> f32 {
        let pts = &self.points;
        if pts.len() < 2 || self.lane.len() != pts.len() {
            return LANE;
        }
        let i = pts.partition_point(|p| p.2 <= t).clamp(1, pts.len() - 1);
        self.lane[i - 1].max(self.lane[i])
    }

    /// How far right of its lane a car can pull over `t` ticks along (m: the less of the two
    /// points either side).
    pub(crate) fn room_at(&self, t: f32) -> f32 {
        let pts = &self.points;
        if pts.len() < 2 || self.room.len() != pts.len() {
            return 0.0;
        }
        let i = pts.partition_point(|p| p.2 <= t).clamp(1, pts.len() - 1);
        self.room[i - 1].min(self.room[i])
    }

    /// `legacy`, to a road out of town.
    pub(crate) fn off_roads(car: &Car, exits: &[[f32; 2]], home: bool) -> Route {
        Self::legacy(car, exits, home)
    }

    #[cfg(test)]
    pub(crate) fn new(car: &Car, exits: &[[f32; 2]], home: bool) -> Route {
        Self::legacy(car, exits, home)
    }

    /// Out of the driveway onto the road it meets and along the roads to the way out of town
    /// quickest by car, past it (`OVERSHOOT`), or (`home`) back in from there, nose first. The
    /// driveway's part as `legacy` lays it out, in the frame of the road at the driveway's
    /// mouth; the lanes right of the roads' middle lines (`lane_on`); `FAST` in the hamlets,
    /// `Travel::country_speed` on country roads, slower at junctions. Sampled more where it
    /// bends (`ROUTE_TOLERANCE`). None: no road near, or no way out by road.
    pub(crate) fn on_roads(
        car: &Car,
        exits: &[[f32; 2]],
        travel: &Travel,
        home: bool,
    ) -> Option<Route> {
        use crate::travel::{RoadKind, dist};
        let g = &travel.graph;
        let m = car.mouth(travel)?;
        let mp = g.point(m);
        // Where to: the parking spot of its trip, or the quickest way out by car.
        let (path, park) = match car.trip {
            Some(trip) => {
                let p = *g.near(trip.at, ROAD_REACH, 1, Mode::Car).first()?;
                (
                    g.path(m, p, Mode::Car, travel.country_speed)?,
                    Some((trip.at, trip.yaw)),
                )
            }
            None => {
                let (_, path) = (0..exits.len())
                    .filter_map(|i| {
                        let e = *g.near(exits[i], ROAD_REACH, 1, Mode::Car).first()?;
                        let path = g.path(m, e, Mode::Car, travel.country_speed)?;
                        Some((i, path))
                    })
                    .min_by(|a, b| a.1.cost.total_cmp(&b.1.cost).then(a.0.cmp(&b.0)))?;
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
        let mut offs: Vec<f32> = path.pts.iter().map(|p| lane_on(p.1, p.2)).collect();
        if park.is_none() {
            centre.push(extend_past(
                centre[centre.len() - 2],
                centre[centre.len() - 1],
            ));
            offs.push(offs[offs.len() - 1]);
        }
        if home {
            centre.reverse();
            offs.reverse();
        }
        let lane = round_corners(&crate::travel::thin_with(
            &offset_right_by(&centre, &offs),
            ROUTE_STEP,
            ROUTE_TOLERANCE,
        ));
        if lane.len() < 2 {
            return None;
        }
        let junctions: Vec<[f32; 2]> = g.junctions().collect();
        // (The road at `p`: its width and kind, those of the path's point nearest it.)
        let near_path = crate::travel::NearestPoint::new(path.pts.iter().map(|x| x.0).collect());
        let road_at = |p: [f32; 2]| {
            near_path
                .nearest(p)
                .map_or((7.0, RoadKind::Street), |i| (path.pts[i].1, path.pts[i].2))
        };
        let tops = lane_tops(&lane, travel, &junctions, |i| road_at(lane[i]).1);
        // A trip's parking bay off the road (`parking_bay`), turned into nose first and backed
        // out of as a driveway is, and which way along the road it goes there (from the lane's
        // nearer end); a spot on the road itself is pulled into and out of straight.
        let bay = park.and_then(|(at, yaw)| {
            let (rp, on, axis) = parking_bay(travel, at)?;
            let road = &g.roads[on.road as usize];
            let rt = road.tangent(on.s);
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
            let (width, kind) = road_at(p);
            legs.push_lane(
                p,
                tops[i],
                (pull_room(width, kind, car.wid) - bend_out(&lane, i, car.len)).max(0.0),
                lane_on(width, kind),
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
        Some(legs.timed(!home || trip, home || trip))
    }

    /// Its way turned round (`Detour`): back along the way it had (`on_roads`) from where it was
    /// held up to where it turns (`Detour::back`), then on along the lanes from there, setting off
    /// the way it faces, round the stretch it was held up on (`Detour::avoid`), to where it was
    /// going: into its driveway (home), its trip's parking spot, or out of town. None: no such
    /// way (only by that stretch, say).
    pub(crate) fn detour(
        car: &Car,
        exits: &[[f32; 2]],
        travel: &Travel,
        home: bool,
        d: &Detour,
        had: &Route,
    ) -> Option<Route> {
        use crate::travel::{RoadKind, Way, dist};
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
                legs.lane.push(had.lane.get(i).copied().unwrap_or(LANE));
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
            avoid,
        };
        let speed = travel.country_speed;
        // (A work taxi, its halts behind it: out of town where its run goes, `taxi.rs`.)
        let taxi = car.taxi;
        let ends = if taxi {
            vec![had.points.last()?.0]
        } else {
            exits.to_vec()
        };
        let m = car.mouth(travel);
        let (path, park) = if home && !taxi {
            (g.path_by(rp, m?, Mode::Car, speed, way)?, None)
        } else if let Some(trip) = car.trip.filter(|_| !taxi) {
            let to = *g.near(trip.at, ROAD_REACH, 1, Mode::Car).first()?;
            (
                g.path_by(rp, to, Mode::Car, speed, way)?,
                Some((trip.at, trip.yaw)),
            )
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
                .filter_map(|&x| {
                    let e = *g.near(x, ROAD_REACH, 1, Mode::Car).first()?;
                    g.path_by(rp, e, Mode::Car, speed, way.clone())
                })
                .min_by(|a, b| weight(a).total_cmp(&weight(b)))
                .map(|p| (p, None))?
        };
        // (Round where it was held up if there's another way; else the same way, once those it
        // backed up out of the way of have gone by.)
        if path.pts.len() < 2 {
            return None;
        }
        let mut centre: Vec<[f32; 2]> = path.pts.iter().map(|p| p.0).collect();
        let mut offs: Vec<f32> = path.pts.iter().map(|p| lane_on(p.1, p.2)).collect();
        let home = home && !taxi;
        if !home && park.is_none() {
            centre.push(extend_past(
                centre[centre.len() - 2],
                centre[centre.len() - 1],
            ));
            offs.push(offs[offs.len() - 1]);
        }
        let lane = round_corners(&crate::travel::thin_with(
            &offset_right_by(&centre, &offs),
            ROUTE_STEP,
            ROUTE_TOLERANCE,
        ));
        if lane.len() < 2 {
            return None;
        }
        let junctions: Vec<[f32; 2]> = g.junctions().collect();
        let near_path = crate::travel::NearestPoint::new(path.pts.iter().map(|x| x.0).collect());
        let road_at = |q: [f32; 2]| {
            near_path
                .nearest(q)
                .map_or((7.0, RoadKind::Street), |i| (path.pts[i].1, path.pts[i].2))
        };
        let tops = lane_tops(&lane, travel, &junctions, |i| road_at(lane[i]).1);
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
                let (rp, on, axis) = parking_bay(travel, at)?;
                let road = &g.roads[on.road as usize];
                let rt = road.tangent(on.s);
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
        for (i, &q) in lane.iter().enumerate().skip(join) {
            if let Some((b, d, (turn, at))) = &into
                && b.near(q, *at, turn.taken, *d)
                && along(b, i, -*d)
            {
                break;
            }
            let (width, kind) = road_at(q);
            legs.push_lane(
                q,
                tops[i],
                (pull_room(width, kind, car.wid) - bend_out(&lane, i, car.len)).max(0.0),
                lane_on(width, kind),
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
        // Drive on the right: heading +x, that's the +z side.
        let lane_out = car.street + d * TIGHT_LANE;
        let lane_in = car.street - d * TIGHT_LANE;
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
