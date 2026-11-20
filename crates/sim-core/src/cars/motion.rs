//! Moving a car: its own place, its way along and its doors.
//!
//! Split from `cars/mod.rs` (see `mod.rs`).

use super::*;

impl Car {
    pub(crate) fn from_raw(raw: &CarRaw, household: u32) -> Self {
        Self {
            plot: raw.plot,
            household,
            spot: [raw.x, raw.z],
            yaw: raw.yaw,
            street: raw.street,
            len: raw.len,
            wid: raw.wid,
            body: raw.body,
            paint: raw.paint,
            age: Default::default(),
            want: None,
            owners: Vec::new(),
            state: CarState::Parked,
            since: 0,
            driver: None,
            aboard: Vec::new(),
            done: Vec::new(),
            ready_at: 0,
            blocked: false,
            route: None,
            meets: None,
            axis: None,
            no_way: false,
            out_ticks: 0.0,
            progress: 0.0,
            trip: None,
            riders: Vec::new(),
            drive_spot: None,
            waited: 0,
            on_foot: Vec::new(),
            zebra_wait: 0,
            at_red: false,
            shown: None,
            next: None,
            next_ground: forward_ground(),
            shift: 0.0,
            waits_for: None,
            moved: false,
            pulled_for: None,
            jam: (0, 0, false),
            backed_for: None,
            taxi: false,
            standing: false,
            sight: Sight::Seen,
            metered: None,
            stall: (0.0, 0),
            detour: None,
            off_run: false,
            let_by: None,
            avoided: Vec::new(),
            pass_by: None,
        }
    }

    /// A work taxi's car slot (`taxi/`), free (out of town) till a run takes it.
    pub(crate) fn taxi(at: [f32; 2], len: f32, wid: f32) -> Self {
        let raw = CarRaw {
            plot: u32::MAX,
            x: at[0],
            z: at[1],
            yaw: 0.0,
            street: at[1],
            len,
            wid,
            body: 0,
            paint: 0,
        };
        let mut car = Car::from_raw(&raw, u32::MAX);
        car.taxi = true;
        car.state = CarState::Away;
        car
    }

    /// A taxi on a run (`taxi/`): coming into town along `route` (at `progress` along it, a game
    /// just loaded).
    pub(crate) fn set_taxi_route(&mut self, route: Route, progress: f32) {
        self.state = CarState::Arriving;
        self.route = Some((true, route));
        self.progress = progress;
        self.standing = false;
        self.shift = 0.0;
        self.waited = 0;
        self.jam = (0, 0, false);
        self.backed_for = None;
        self.waits_for = None;
        self.pulled_for = None;
        // (A new way: not one it turned round to, `unjam`.)
        self.detour = None;
        self.let_by = None;
    }

    /// A taxi's route's length (ticks).
    /// A taxi's door for `seat` (`door`), else its own door's place whatever is there.
    pub(crate) fn taxi_door(&self, nav: &crate::path::NavGrid, seat: usize) -> (usize, [f32; 2]) {
        self.door(nav, seat).unwrap_or_else(|| {
            let d = [1, 3, 2, 0][seat.min(3)];
            (d, self.door_at(d))
        })
    }

    pub(crate) fn taxi_ticks(&self) -> f32 {
        self.route.as_ref().map_or(0.0, |r| r.1.ticks())
    }

    /// A taxi done with its run: out of town, free.
    pub(crate) fn park_taxi(&mut self) {
        self.off_run = false;
        self.let_by = None;
        self.detour = None;
        self.state = CarState::Away;
        self.route = None;
        self.progress = 0.0;
        self.standing = false;
        self.shown = None;
        self.next = None;
        self.shift = 0.0;
        self.waits_for = None;
    }

    /// Out of sight (`Sight`): it waits for nobody on the road or on foot, nor stays backed up or
    /// pulled over for anyone.
    pub(crate) fn unhold(&mut self) {
        self.moved = false;
        self.waits_for = None;
        self.on_foot.clear();
        self.waited = 0;
        self.zebra_wait = 0;
        self.at_red = false;
        self.jam = (0, 0, false);
        self.backed_for = None;
        self.pulled_for = None;
    }

    /// Standing on its driveway (parked, or waiting for the others to get in).
    pub fn at_home(&self) -> bool {
        matches!(self.state, CarState::Parked | CarState::Boarding)
    }

    /// Whether `j` may drive it: an owner, grown up.
    pub(crate) fn driven_by(&self, briefs: &[SimBrief], j: u32) -> bool {
        self.owners.contains(&j) && briefs.get(j as usize).is_some_and(|b| b.adult)
    }

    /// The point `along` its length (forward) and `left` across it (to its left) from its middle,
    /// parked.
    pub(crate) fn local(&self, along: f32, left: f32) -> [f32; 2] {
        local_at(self.spot, self.yaw, along, left)
    }

    /// The car as it stands parked at `spot` facing `yaw` (its doors and seats there): a copy
    /// without its routes or who is in it.
    pub(crate) fn posed(&self, spot: [f32; 2], yaw: f32) -> Car {
        Car {
            plot: self.plot,
            household: self.household,
            spot,
            yaw,
            street: self.street,
            len: self.len,
            wid: self.wid,
            body: self.body,
            paint: self.paint,
            age: self.age,
            want: None,
            owners: Vec::new(),
            state: self.state,
            since: self.since,
            driver: self.driver,
            aboard: Vec::new(),
            done: Vec::new(),
            ready_at: self.ready_at,
            blocked: self.blocked,
            route: None,
            meets: self.meets,
            axis: self.axis,
            no_way: self.no_way,
            out_ticks: self.out_ticks,
            progress: self.progress,
            trip: self.trip,
            riders: Vec::new(),
            drive_spot: self.drive_spot,
            waited: self.waited,
            on_foot: Vec::new(),
            zebra_wait: 0,
            at_red: false,
            shown: self.shown,
            next: None,
            next_ground: forward_ground(),
            shift: 0.0,
            waits_for: None,
            moved: false,
            pulled_for: None,
            jam: (0, 0, false),
            backed_for: None,
            taxi: false,
            standing: false,
            sight: Sight::Seen,
            metered: None,
            stall: (0.0, 0),
            detour: None,
            off_run: false,
            let_by: None,
            avoided: Vec::new(),
            pass_by: None,
        }
    }

    /// Door `d`'s side (+1 left, -1 right) and how far along the car it is.
    fn door_side(&self, d: usize) -> (f32, f32) {
        let k = if d.is_multiple_of(2) { 1.0 } else { -1.0 };
        let along = if d < 2 {
            self.len * 0.08
        } else {
            -self.len * 0.19
        };
        (k, along)
    }

    /// Where someone stands to get in by door `d` (or out): beside it, outside the car.
    pub(crate) fn door_at(&self, d: usize) -> [f32; 2] {
        let (k, along) = self.door_side(d);
        self.local(along, (self.wid / 2.0 + DOOR_OUT) * k)
    }

    /// Where the one in `seat` sits, getting in by door `d`: on that door's side (the front seats a
    /// little back from the doors), the fifth (seat 4) in the middle at the back.
    pub(crate) fn seat_at(&self, seat: usize, d: usize) -> [f32; 2] {
        let (k, along) = self.door_side(d);
        if seat >= DOORS {
            return self.local(-self.len * 0.19, 0.0);
        }
        let along = if d < 2 { self.len * 0.05 } else { along };
        self.local(along, (self.wid / 2.0 - 0.5).max(0.2) * k)
    }

    /// The door someone in seat `seat` (0 the driver's) gets in by, and where they stand for it:
    /// their own, else the nearest other one with room outdoors to stand and for the door to
    /// swing open (`SWING`: not into the house, a fence or a thing).
    pub fn door(&self, nav: &crate::path::NavGrid, seat: usize) -> Option<(usize, [f32; 2])> {
        self.door_swung(nav, seat, &[0.25, 0.6, SWING])
    }

    /// `door`, for a car parked out in a public bay (`CarState::Out`, posed there): with cars in
    /// the bays either side (3 m apart, `BAY_WIDE`) none has room to swing a door wide open, so
    /// one opened only as far as where they stand (`DOOR_OUT`) does: they squeeze in or out by
    /// it, as anyone does in a full car park.
    pub fn door_in_bay(
        &self,
        nav: &crate::path::NavGrid,
        seat: usize,
    ) -> Option<(usize, [f32; 2])> {
        self.door(nav, seat)
            .or_else(|| self.door_swung(nav, seat, &[0.25, DOOR_OUT]))
    }

    /// `door` with the door swung out as far as each of `swing` (m from the car's side, along
    /// its length) clear of anything in the way.
    fn door_swung(
        &self,
        nav: &crate::path::NavGrid,
        seat: usize,
        swing: &[f32],
    ) -> Option<(usize, [f32; 2])> {
        const ORDER: [[usize; DOORS]; 4] = [[0, 1, 2, 3], [1, 3, 0, 2], [3, 1, 2, 0], [2, 0, 3, 1]];
        let own = self.tiles();
        let open = |x: i32, z: i32| {
            own.contains(&(x, z)) || (nav.tile_free(x, z) && nav.lot.room_at(x, z) == OUTDOORS)
        };
        ORDER[seat.min(3)]
            .into_iter()
            .map(|d| (d, self.door_at(d)))
            .find(|&(d, p)| {
                let (k, along) = self.door_side(d);
                let side = self.wid / 2.0;
                let (x, z) = (p[0].floor() as i32, p[1].floor() as i32);
                // The ground the door sweeps, outside the car along its length: nothing in the way,
                // nor a fence or a wall between it and where they stand.
                let swept_clear = || {
                    [-0.5f32, 0.0, 0.5].into_iter().all(|a| {
                        swing.iter().all(|&o| {
                            let q = self.local(along + a, (side + o) * k);
                            let (qx, qz) = (q[0].floor() as i32, q[1].floor() as i32);
                            let next = (qx - x).abs() + (qz - z).abs() == 1;
                            open(qx, qz)
                                && (!next
                                    || own.contains(&(qx, qz))
                                    || nav.lot.edge_walkable(x, z, qx, qz))
                        })
                    })
                };
                nav.tile_free(x, z) && nav.lot.room_at(x, z) == OUTDOORS && swept_clear()
            })
    }

    /// Where it is and which way it faces now, and whether it's moving; None while it's out
    /// of town.
    pub fn pose(&mut self, exits: &[[f32; 2]]) -> Option<([f32; 2], f32, bool)> {
        self.pose_on(exits, None)
    }

    /// `pose`, on the town's roads (`travel/`) where it has them.
    pub fn pose_on(
        &mut self,
        exits: &[[f32; 2]],
        travel: Option<&Travel>,
    ) -> Option<([f32; 2], f32, bool)> {
        match self.state {
            CarState::Parked | CarState::Boarding => Some((self.spot, self.yaw, false)),
            CarState::Away => None,
            CarState::Out => self.trip.map(|t| (t.at, t.yaw, false)),
            // (Not yet come in: out of sight at the way in.)
            CarState::Arriving if self.outside() => None,
            CarState::Leaving | CarState::Arriving => {
                let (t, shift, moving) = (self.progress, self.shift, !self.standing);
                let route = self.route(exits, travel, self.state == CarState::Arriving);
                (t < route.ticks()).then(|| {
                    let (p, yaw, _) = route.at(t);
                    (shifted(p, yaw, shift), yaw, moving)
                })
            }
        }
    }

    /// Whether it's backing up now, or just was (along its route, as `pose_on` last had it, or a
    /// tick before): out of a driveway or into one.
    pub(crate) fn backing(&self) -> bool {
        matches!(self.state, CarState::Leaving | CarState::Arriving)
            && self.route.as_ref().is_some_and(|(_, r)| {
                r.at(self.progress).2 || r.at((self.progress - 1.0).max(0.0)).2
            })
    }

    /// Where its middle is, for the checks that pair it with another (`Near`): along its way, as
    /// `pull_want` has it (not as drawn: one waiting to come into town isn't, yet comes in this
    /// tick); else as it's drawn, or its spot.
    pub(crate) fn middle(&self) -> [f32; 2] {
        match self.route.as_ref() {
            Some((_, r)) if r.points.len() >= 2 => r.at(self.progress).0,
            _ => self.shown.map_or(self.spot, |s| s.0),
        }
    }

    /// Coming into town from the way in (not from a trip's parking spot), it hasn't yet: out of
    /// sight there, waiting for room to come in (`drive_on`).
    pub(crate) fn outside(&self) -> bool {
        self.state == CarState::Arriving && self.trip.is_none() && self.progress <= 0.0
    }

    /// Whether it waits out of sight to come into town by the way in at `at` (`outside`; the
    /// taxis count those, `taxi::queued`).
    pub(crate) fn waits_to_come_in_at(&self, at: [f32; 2]) -> bool {
        self.outside()
            && self
                .route
                .as_ref()
                .is_some_and(|r| r.0 && crate::travel::dist(r.1.at(0.0).0, at) < 4.0)
    }

    /// Its route out of town or (`home`) back (worked out when first asked for).
    pub(crate) fn route(
        &mut self,
        exits: &[[f32; 2]],
        travel: Option<&Travel>,
        home: bool,
    ) -> &Route {
        if self.route.as_ref().is_none_or(|(h, _)| *h != home) {
            // (Turned round on this way, `unjam`: that way, as it was then.)
            let detour = self
                .detour
                .filter(|d| d.home == home)
                .zip(travel.filter(|t| !t.graph.is_empty()))
                .and_then(|(d, t)| {
                    let had = Route::on_roads(self, exits, t, home)?;
                    self.detoured(exits, t, home, &d, &had)
                });
            let route = detour.unwrap_or_else(|| Route::build(self, exits, travel, home));
            self.route = Some((home, route));
        }
        &self.route.as_ref().expect("just made").1
    }

    /// Its way turned round `d` from the way it had (`Route::detour`), with its claims: none
    /// backing up out of the way it came (it backs into room that's clear, `unjam`, never deeper
    /// into another, `guard`).
    pub(crate) fn detoured(
        &self,
        exits: &[[f32; 2]],
        travel: &Travel,
        home: bool,
        d: &Detour,
        had: &Route,
    ) -> Option<Route> {
        let mut r = Route::detour(self, exits, travel, home, d, had)?;
        // (It sets off backing up facing as it faces now: no turn on the spot.)
        if let Some(p) = r.points.first_mut() {
            p.1 = had.at(d.from).1;
        }
        r.claim(self.len, self.wid, Some(travel));
        let back = r.points.iter().find(|p| !p.3).map_or(0.0, |p| p.2);
        r.claims.retain(|k| k.to > back);
        for k in &mut r.claims {
            k.from = k.from.max(back);
        }
        r.zebras.retain(|z| z.at > back);
        Some(r)
    }

    /// Works out again what follows from where it parks (after it came home, moved, or loaded).
    pub(crate) fn settle(&mut self, exits: &[[f32; 2]], travel: Option<&Travel>) {
        let roads = travel.filter(|t| !t.graph.is_empty());
        let out = match roads.and_then(|t| Route::on_roads(self, exits, t, false)) {
            Some(out) => {
                self.no_way = false;
                out
            }
            None => {
                // (Home, with no way out by road in a town whose cars drive its roads.)
                if self.trip.is_none() {
                    self.no_way = roads.is_some();
                }
                // (`on_roads` just failed: not again, as `build` would.)
                Route::off_roads(self, exits, false)
            }
        };
        let mut out = out;
        out.claim(self.len, self.wid, travel);
        self.out_ticks = out.ticks();
        self.route = Some((false, out));
    }

    /// Where its driveway meets the road (`meets`, else the nearest road it can drive on).
    pub(crate) fn mouth(&self, travel: &Travel) -> Option<RoadPoint> {
        let g = &travel.graph;
        self.meets
            .and_then(|(road, s)| g.at_source(road, s))
            .filter(|p| g.roads[p.road as usize].kind != crate::travel::RoadKind::Trail)
            .or_else(|| g.near(self.spot, ROAD_REACH, 1, Mode::Car).first().copied())
    }

    /// Its route's claims worked out again (the junctions' lights settled: `traffic::settle`).
    pub(crate) fn reclaim(&mut self, travel: &Travel) {
        let (len, wid) = (self.len, self.wid);
        let turned = self.detour.is_some();
        if let Some((_, route)) = self.route.as_mut() {
            route.claim(len, wid, Some(travel));
            // (Turned round: none backing up out of the way it came, as `detoured` has them.)
            if turned && route.points.first().is_some_and(|p| p.3) {
                let back = route.points.iter().find(|p| !p.3).map_or(0.0, |p| p.2);
                route.claims.retain(|k| k.to > back);
                for k in &mut route.claims {
                    k.from = k.from.max(back);
                }
                route.zebras.retain(|z| z.at > back);
            }
        }
    }

    /// The yaw it parks with after coming home, nose in: facing up its driveway, away from the
    /// street (`axis`), else away from the road it meets (the street, in towns with no roads).
    pub(crate) fn nose_in_on(&self, travel: Option<&Travel>) -> f32 {
        if let Some([x, z]) = self.axis {
            // (0.0 - …: never -0.0, so a driveway along z gives the yaw `nose_in` does.)
            return (0.0 - x).atan2(0.0 - z);
        }
        if let Some(m) = travel
            .filter(|t| !t.graph.is_empty())
            .and_then(|t| self.mouth(t).map(|m| t.graph.point(m)))
        {
            let (dx, dz) = (self.spot[0] - m[0], self.spot[1] - m[1]);
            if dx.hypot(dz) > 0.5 {
                return dx.atan2(dz);
            }
        }
        self.nose_in()
    }

    /// The yaw it parks with after coming home: nose in, facing away from the street.
    pub(crate) fn nose_in(&self) -> f32 {
        if self.street > self.spot[1] {
            std::f32::consts::PI
        } else {
            0.0
        }
    }

    /// The tiles it takes up parked.
    pub(crate) fn tiles(&self) -> Vec<(i32, i32)> {
        self.tiles_at(self.spot, self.yaw)
    }

    /// The tiles it takes up parked at `at` facing `yaw`.
    pub(crate) fn tiles_at(&self, at: [f32; 2], yaw: f32) -> Vec<(i32, i32)> {
        let (s, c) = yaw.sin_cos();
        // Axis-aligned enough: a car parks along its driveway, or square to the road in a bay.
        let (hw, hl) = if s.abs() > c.abs() {
            (self.len / 2.0, self.wid / 2.0)
        } else {
            (self.wid / 2.0, self.len / 2.0)
        };
        let [x, z] = at;
        covered_tiles(x - hw, z - hl, x + hw, z + hl)
    }

    /// Where it stands parked out on a trip (`CarState::Out`), and which way it faces.
    pub(crate) fn out_at(&self) -> Option<([f32; 2], f32)> {
        self.trip
            .filter(|_| self.state == CarState::Out)
            .map(|t| (t.at, t.yaw))
    }
}

/// On `OVERSHOOT` past `b`, the way from `a` to `b`: where a drive out of town ends, off the map.
pub(crate) fn extend_past(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let len = crate::travel::dist(a, b).max(1e-3);
    [
        b[0] + (b[0] - a[0]) / len * OVERSHOOT,
        b[1] + (b[1] - a[1]) / len * OVERSHOOT,
    ]
}

/// The top speed at each of `lane`'s points (m/s of drive time; `kind_at` the kind of road at
/// each): `FAST` in the hamlets, `Travel::country_speed` on country roads, and `JUNCTION_SPEED`
/// through a junction (within `JUNCTION_REACH` of it, `junctions`) where the way turns there by
/// more than `TURN_AT`. Straight through one it keeps its speed (its bends limit it,
/// `Legs::timed`'s `SIDEWAYS`; the lights and the stop lines stop it, `traffic/`).
pub(crate) fn lane_tops(
    lane: &[[f32; 2]],
    travel: &Travel,
    junctions: &[[f32; 2]],
    kind_at: impl Fn(usize) -> crate::travel::RoadKind,
) -> Vec<f32> {
    use crate::travel::dist;
    let n = lane.len();
    let mut tops: Vec<f32> = (0..n)
        .map(|i| {
            if kind_at(i) == crate::travel::RoadKind::Country {
                travel.country_speed
            } else {
                FAST
            }
        })
        .collect();
    let heading = |a: usize, b: usize| {
        let (p, q) = (lane[a], lane[b]);
        (q[0] - p[0]).atan2(q[1] - p[1])
    };
    for &j in junctions {
        // (Each pass by it: the points within reach, one run after another.)
        let mut i = 0;
        while i < n {
            if dist(lane[i], j) >= JUNCTION_REACH {
                i += 1;
                continue;
            }
            let i0 = i;
            while i + 1 < n && dist(lane[i + 1], j) < JUNCTION_REACH {
                i += 1;
            }
            let i1 = i;
            i += 1;
            // (The way it comes in and the way it goes out, from the points just outside.)
            let (a0, b0) = if i0 > 0 {
                (i0 - 1, i0)
            } else {
                (0, 1.min(n - 1))
            };
            let (a1, b1) = if i1 + 1 < n {
                (i1, i1 + 1)
            } else {
                (i1.saturating_sub(1), i1)
            };
            let turned = crate::world::wrap_angle(heading(a1, b1) - heading(a0, b0)).abs();
            if turned > TURN_AT || a0 == b0 || a1 == b1 {
                for t in &mut tops[i0..=i1] {
                    *t = t.min(JUNCTION_SPEED);
                }
            }
        }
    }
    tops
}

/// How many ticks a car takes along the roads from the road nearest `from` to the road nearest
/// `to` with them to itself (every light green, nobody in its way): in its lane, its corners
/// rounded, slowing for junctions and bends as `Route::on_roads` lays a drive out, from standing
/// to standing. None: no road near either, or no way between by road. (A world's drive across,
/// timed: the tests.)
pub fn empty_drive_ticks(travel: &Travel, from: [f32; 2], to: [f32; 2]) -> Option<f32> {
    let g = &travel.graph;
    let a = *g.near(from, ROAD_REACH, 1, Mode::Car).first()?;
    let b = *g.near(to, ROAD_REACH, 1, Mode::Car).first()?;
    let path = g.path(a, b, Mode::Car, travel.country_speed)?;
    if path.pts.len() < 2 {
        return None;
    }
    let centre: Vec<[f32; 2]> = path.pts.iter().map(|p| p.0).collect();
    let offs: Vec<f32> = path.pts.iter().map(|p| lane_on(p.1, p.2)).collect();
    let lane = round_corners(&crate::travel::thin_with(
        &offset_right_by(&centre, &offs),
        ROUTE_STEP,
        ROUTE_TOLERANCE,
    ));
    let junctions: Vec<[f32; 2]> = g.junctions().collect();
    let near_path = crate::travel::NearestPoint::new(path.pts.iter().map(|x| x.0).collect());
    let road_at = |p: [f32; 2]| near_path.nearest(p).map(|i| (path.pts[i].1, path.pts[i].2));
    let tops = lane_tops(&lane, travel, &junctions, |i| {
        road_at(lane[i]).map_or(crate::travel::RoadKind::Street, |r| r.1)
    });
    let mut legs = Legs::default();
    for (i, &p) in lane.iter().enumerate() {
        let (width, kind) = road_at(p)?;
        let top = tops[i];
        legs.push_lane(
            p,
            top,
            pull_room(width, kind, 2.0 * CAR_HALF),
            lane_on(width, kind),
        );
    }
    Some(legs.timed(true, true).ticks())
}
