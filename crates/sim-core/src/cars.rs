//! Households' cars: parked on their driveway, driven to work (or school) and back.
//!
//! A car belongs to a household and stands on its home's driveway (the town file's `cars`; every
//! house's driveway is in its `drives`). Only its owners drive it, and only grown-ups (18 and up):
//! the household's grown-ups when the town was made (a household left with no owner hands it down
//! to its grown-ups). Whoever else in the household leaves at the same time rides along. When the
//! household moves house, the car goes with it, to the new home's driveway (`relocate`).
//!
//! A day out by car:
//! - **Parked.** Those who'll go by car leave home early enough to be at work on time
//!   (`lead_ticks`: the walk to the car, getting in, the drive out of town and on to work). An owner
//!   walks to the driver's door instead of the town's exit (`ride_for`), the door opens and they get
//!   in (`Phase::Boarding`, then `Transition::Board`). Their shift starts at its hour, not as they
//!   get in (`Sim::shift_from`).
//! - **Boarding.** The driver waits a little for the rest of the household setting off now
//!   (those walking to the exit are called to the car instead), at most `BOARD_WAIT_TICKS`; each
//!   gets in by a door of their own (`Car::door`).
//! - **Leaving.** The car backs out (or pulls out) of the driveway and drives off along the street
//!   to the nearest end of town (`Route`), then it is **Away**.
//! - Whoever finishes first waits at work for the others when they finish within
//!   `RIDE_WAIT_TICKS` (a rider whose driver works later walks home, as without a car). The drive
//!   back to town takes `OFF_MAP_TICKS`.
//! - **Arriving.** The car drives back in, nose first, and parks; its doors open and they get out
//!   and walk to the house.
//!
//! A parked car takes up the tiles under it: nobody walks through it and nothing can be put there
//! (`World::blocked`, `reblock`). A car whose parking spot has been built over (a wall, a fence, a
//! thing put there while it was out) stays where it is, unused (`Car::blocked`), and the renderer
//! leaves it out.
//!
//! The route is worked out from the car's spot, the street it faces and the town's exits, the
//! same way each time, so it isn't saved; the snapshot gives the renderer the car's pose along
//! it, and which doors are open, each tick (`snapshot::car`).

use serde::{Deserialize, Serialize};

use crate::TICKS_PER_SECOND;
use crate::lot::OUTDOORS;
use crate::world::{
    Household, Phase, Sim, SimBrief, Task, TaskKind, World, nearest_exit, wrap_angle,
};

/// Most cars in a town (the snapshot's rows).
pub const MAX_CARS: usize = 64;
/// Doors: 0 front left (the driver's), 1 front right, 2 back left, 3 back right.
pub const DOORS: usize = 4;
/// How long the driver waits in the car for the others setting off at the same time.
const BOARD_WAIT_TICKS: u64 = 4 * crate::TICKS_PER_MINUTE;
/// How long getting in takes before the car can go (the doors, the belt).
const BOARD_MIN_TICKS: u64 = crate::TICKS_PER_MINUTE / 2;
/// Getting in (`Phase::Boarding`): at the door while it opens, a step in to the seat, then sat
/// while it shuts (the renderer swings a door in about 9 ticks at any speed), and in. Getting
/// out (`Phase::Alighting`): sat while the door opens, a step out to beside it, and off (it
/// shuts behind them).
pub(crate) const DOOR_OPENS: u32 = 10;
pub(crate) const STEP_IN: u32 = 14;
pub(crate) const GET_IN_TICKS: u32 = DOOR_OPENS + STEP_IN + 12;
pub(crate) const GET_OUT_TICKS: u32 = DOOR_OPENS + STEP_IN;
/// How far a door swings out from the car's side (m), to have room for (`Car::door`).
const SWING: f32 = 1.2;
/// While a car drives, the ground ahead of it it keeps clear of people (m): it waits for anyone
/// there (`drive_on`); and how close to its sides they keep (m).
const AHEAD: f32 = 2.2;
const BESIDE: f32 = 0.35;
/// Someone waiting for a car to pass for this long steps aside out of its way (`make_way`).
pub(crate) const MAKE_WAY_TICKS: u32 = 30;
/// How long someone done at work waits there for the others in their car.
const RIDE_WAIT_TICKS: u64 = 60 * crate::TICKS_PER_MINUTE;
/// The drive between the end of town and work, each way.
const OFF_MAP_TICKS: u64 = 10 * crate::TICKS_PER_MINUTE;
/// About how long the walk from the house to the car takes (for when to leave).
const WALK_TO_CAR_TICKS: u64 = 3 * crate::TICKS_PER_MINUTE;
/// The middle of a lane from the middle of the street (m): they drive on the right.
const LANE: f32 = 0.8;
/// Turning radius at the driveway's mouth (m).
const TURN: f32 = 2.4;
/// How far past the town's exit a car drives before it's gone (m).
const OVERSHOOT: f32 = 12.0;
/// Top speeds (m per second at 1×): along the street, and in the driveway, turning or backing up.
const FAST: f32 = 6.0;
const SLOW: f32 = 1.8;
/// Speeding up and slowing down (m/s²).
const ACCEL: f32 = 2.2;
/// The route's sampling step (m).
const STEP: f32 = 0.25;
/// From the side of the car to where someone gets in (m).
const DOOR_OUT: f32 = 0.45;
/// Seats.
const SEATS: usize = 5;
/// How far from the car (m) someone already on the street turns back for a ride.
const CALL_REACH: f32 = 15.0;
/// Clearance from the inner end of a driveway to the car (m), as the town parks them.
const PARK_GAP: f32 = 0.45;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CarState {
    #[default]
    Parked,
    /// The driver is in, waiting for the others (`Car::since`: since when).
    Boarding,
    /// Driving off (`Car::since`: when it set off).
    Leaving,
    /// Out of town, with whoever is at work with it.
    Away,
    /// Driving back (`Car::since`: when it set off).
    Arriving,
}

/// A household's car.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Car {
    pub plot: u32,
    pub household: u32,
    /// Where it parks (the middle of the car) and which way it faces there (as a resident's
    /// yaw: 0 towards +z).
    pub spot: [f32; 2],
    pub yaw: f32,
    /// The middle of the street its driveway leads to (z).
    pub street: f32,
    /// Its length and width (m).
    pub len: f32,
    pub wid: f32,
    /// What it looks like (the renderer's bodies and paints).
    pub body: u8,
    pub paint: u8,
    /// Who may drive it (resident ids).
    pub owners: Vec<u32>,
    #[serde(default)]
    pub state: CarState,
    /// When the state began (a tick).
    #[serde(default)]
    pub since: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver: Option<u32>,
    /// In it, or out at work with it (the driver too).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aboard: Vec<u32>,
    /// Of those, the ones done for the day: waiting for the car home, or in it on the way.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub done: Vec<u32>,
    /// Away: it sets off back to town no sooner than this (the last one done, and the drive from
    /// work to the end of town).
    #[serde(default, skip_serializing_if = "crate::save::is_zero_u64")]
    pub ready_at: u64,
    /// Something stands where it parks: it isn't used (`refresh`). Not saved.
    #[serde(skip)]
    pub blocked: bool,
    /// Its route out or (true) home, worked out once (not saved).
    #[serde(skip)]
    route: Option<(bool, Route)>,
    /// Ticks the drive out of town takes from where it parks (`settle`; not saved).
    #[serde(skip)]
    out_ticks: f32,
    /// How far along its drive it is (ticks of its route: it stops for people in its way).
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub progress: f32,
    /// Ticks it has waited for people in its way (not saved).
    #[serde(skip)]
    waited: u32,
    /// Where it is now, which way it faces, whether it's moving and which doors are open (bit
    /// `d` for door `d`); None while it's out of town or unused (`poses`; for the snapshot, not
    /// saved).
    #[serde(skip)]
    pub shown: Option<([f32; 2], f32, bool, u8)>,
}

fn is_zero_f32(v: &f32) -> bool {
    *v == 0.0
}

/// A car as the town file gives it (`LotFile::cars`): owners are the household's grown-ups.
#[derive(Debug, Clone, Deserialize)]
pub struct CarRaw {
    pub plot: u32,
    pub x: f32,
    pub z: f32,
    pub yaw: f32,
    pub street: f32,
    pub len: f32,
    pub wid: f32,
    #[serde(default)]
    pub body: u8,
    #[serde(default)]
    pub paint: u8,
}

/// A house's driveway, where a car parks (`LotFile::drives`): the middle of its inner end (x, z),
/// which way it runs from there to the street (`front`: +1 towards +z), and the middle of that
/// street (z).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Drive {
    pub plot: u32,
    pub x: f32,
    pub z: f32,
    pub front: f32,
    pub street: f32,
}

/// A garage at the inner end of a house's driveway (`LotFile::garages`, as the town placed it:
/// wall centre lines). It stands where it is; nobody walks through it and nothing is put there
/// (`reblock`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Garage {
    pub plot: u32,
    pub x0: f32,
    pub z0: f32,
    pub x1: f32,
    pub z1: f32,
}

impl Garage {
    /// The tiles it stands on.
    fn tiles(&self) -> Vec<(i32, i32)> {
        covered_tiles(self.x0, self.z0, self.x1, self.z1)
    }
}

/// The tiles the box from (x0, z0) to (x1, z1) covers at least half of each way.
fn covered_tiles(x0: f32, z0: f32, x1: f32, z1: f32) -> Vec<(i32, i32)> {
    let covers = |a: f32, b: f32, t: i32| (b.min(t as f32 + 1.0) - a.max(t as f32)) >= 0.5;
    let mut out = Vec::new();
    for z in z0.floor() as i32..=z1.floor() as i32 {
        for x in x0.floor() as i32..=x1.floor() as i32 {
            if covers(x0, x1, x) && covers(z0, z1, z) {
                out.push((x, z));
            }
        }
    }
    out
}

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
            owners: Vec::new(),
            state: CarState::Parked,
            since: 0,
            driver: None,
            aboard: Vec::new(),
            done: Vec::new(),
            ready_at: 0,
            blocked: false,
            route: None,
            out_ticks: 0.0,
            progress: 0.0,
            waited: 0,
            shown: None,
        }
    }

    /// Standing on its driveway (parked, or waiting for the others to get in).
    pub fn at_home(&self) -> bool {
        matches!(self.state, CarState::Parked | CarState::Boarding)
    }

    /// The point `along` its length (forward) and `left` across it (to its left) from its middle,
    /// parked.
    fn local(&self, along: f32, left: f32) -> [f32; 2] {
        let (s, c) = self.yaw.sin_cos();
        [
            self.spot[0] + s * along + c * left,
            self.spot[1] + c * along - s * left,
        ]
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
    fn door_at(&self, d: usize) -> [f32; 2] {
        let (k, along) = self.door_side(d);
        self.local(along, (self.wid / 2.0 + DOOR_OUT) * k)
    }

    /// Where the one getting in by door `d` sits (the front seats a little back from the doors).
    pub(crate) fn seat_at(&self, d: usize) -> [f32; 2] {
        let (k, along) = self.door_side(d);
        let along = if d < 2 { self.len * 0.05 } else { along };
        self.local(along, (self.wid / 2.0 - 0.5).max(0.2) * k)
    }

    /// The door someone in seat `seat` (0 the driver's) gets in by, and where they stand for it:
    /// their own, else the nearest other one with room outdoors to stand and for the door to
    /// swing open (`SWING`: not into the house, a fence or a thing).
    pub(crate) fn door(
        &self,
        nav: &crate::path::NavGrid,
        seat: usize,
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
                        [0.25f32, 0.6, SWING].into_iter().all(|o| {
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

    /// Whether `pos` is on the car standing at `at` facing `yaw`, close beside it (`BESIDE`), or
    /// within `ahead` in front of it or `behind` behind it.
    fn covers(&self, at: [f32; 2], yaw: f32, pos: [f32; 2], ahead: f32, behind: f32) -> bool {
        let (s, c) = yaw.sin_cos();
        let (dx, dz) = (pos[0] - at[0], pos[1] - at[1]);
        let along = dx * s + dz * c;
        let across = dx * c - dz * s;
        across.abs() < self.wid / 2.0 + BESIDE
            && along < self.len / 2.0 + ahead
            && along > -(self.len / 2.0 + behind)
    }

    /// Where it is and which way it faces now, and whether it's moving; None while it's out
    /// of town.
    pub fn pose(&mut self, exits: &[[f32; 2]]) -> Option<([f32; 2], f32, bool)> {
        match self.state {
            CarState::Parked | CarState::Boarding => Some((self.spot, self.yaw, false)),
            CarState::Away => None,
            CarState::Leaving | CarState::Arriving => {
                let t = self.progress;
                let route = self.route(exits, self.state == CarState::Arriving);
                (t < route.ticks()).then(|| {
                    let (p, yaw, _) = route.at(t);
                    (p, yaw, true)
                })
            }
        }
    }

    /// Its route out of town or (`home`) back (worked out when first asked for).
    fn route(&mut self, exits: &[[f32; 2]], home: bool) -> &Route {
        if self.route.as_ref().is_none_or(|(h, _)| *h != home) {
            self.route = Some((home, Route::new(self, exits, home)));
        }
        &self.route.as_ref().expect("just made").1
    }

    /// Works out again what follows from where it parks (after it came home, moved, or loaded).
    fn settle(&mut self, exits: &[[f32; 2]]) {
        let out = Route::new(self, exits, false);
        self.out_ticks = out.ticks();
        self.route = Some((false, out));
    }

    /// The yaw it parks with after coming home: nose in, facing away from the street.
    fn nose_in(&self) -> f32 {
        if self.street > self.spot[1] {
            std::f32::consts::PI
        } else {
            0.0
        }
    }

    /// The tiles it takes up parked.
    fn tiles(&self) -> Vec<(i32, i32)> {
        let (s, c) = self.yaw.sin_cos();
        // Axis-aligned enough: a car parks along its driveway.
        let (hw, hl) = if s.abs() > c.abs() {
            (self.len / 2.0, self.wid / 2.0)
        } else {
            (self.wid / 2.0, self.len / 2.0)
        };
        let [x, z] = self.spot;
        covered_tiles(x - hw, z - hl, x + hw, z + hl)
    }
}

/// A drive, sampled every `STEP` metres: where the car's middle is, which way it faces, and when
/// it gets there (ticks from the start).
#[derive(Debug, Clone, Default)]
struct Route {
    points: Vec<([f32; 2], f32, f32, bool)>,
}

impl Route {
    /// Out of the driveway to the nearest end of town along its street, or (`home`) back in
    /// from there, nose first.
    fn new(car: &Car, exits: &[[f32; 2]], home: bool) -> Route {
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
        let lane_out = car.street + d * LANE;
        let lane_in = car.street - d * LANE;
        let mut legs = Legs::default();
        if home {
            let r = TURN.min((lane_in - z0).abs()).max(0.5);
            legs.line([end_x, lane_in], [x0 + d * r, lane_in], false, FAST);
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
            legs.line([x0 + k * r, lane_out], [end_x, lane_out], false, FAST);
        }
        legs.timed(!home, home)
    }

    fn ticks(&self) -> f32 {
        self.points.last().map_or(0.0, |p| p.2)
    }

    /// Where the car is `t` ticks in, which way it faces, and whether it's backing up.
    fn at(&self, t: f32) -> ([f32; 2], f32, bool) {
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
        (p, a.1 + wrap_angle(b.1 - a.1) * k, a.3)
    }
}

/// The pieces of a route as they're laid out: sampled points, whether the car backs along them,
/// and the top speed there.
#[derive(Default)]
struct Legs {
    pts: Vec<([f32; 2], bool, f32)>,
}

impl Legs {
    fn push(&mut self, p: [f32; 2], reverse: bool, top: f32) {
        if self
            .pts
            .last()
            .is_some_and(|q| (q.0[0] - p[0]).hypot(q.0[1] - p[1]) < 1e-3 && q.1 == reverse)
        {
            return;
        }
        self.pts.push((p, reverse, top));
    }

    fn line(&mut self, a: [f32; 2], b: [f32; 2], reverse: bool, top: f32) {
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
    fn arc(&mut self, c: [f32; 2], r: f32, a: [f32; 2], b: [f32; 2], reverse: bool) {
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
    fn timed(self, stop_start: bool, stop_end: bool) -> Route {
        let pts = self.pts;
        let n = pts.len();
        if n < 2 {
            return Route {
                points: pts.iter().map(|p| (p.0, 0.0, 0.0, false)).collect(),
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
        for i in 1..n {
            v[i] = v[i].min((v[i - 1] * v[i - 1] + 2.0 * ACCEL * ds[i - 1]).sqrt());
        }
        for i in (0..n - 1).rev() {
            v[i] = v[i].min((v[i + 1] * v[i + 1] + 2.0 * ACCEL * ds[i]).sqrt());
        }
        let per_second = TICKS_PER_SECOND as f32;
        let mut t = 0.0;
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            if i > 0 {
                t += ds[i - 1] / ((v[i - 1] + v[i]) / 2.0).max(0.3) * per_second;
            }
            // Facing: the way to the next point (the last, from the one before).
            let (a, b) = if i + 1 < n {
                (pts[i].0, pts[i + 1].0)
            } else {
                (pts[i - 1].0, pts[i].0)
            };
            let reverse = if i + 1 < n { pts[i + 1].1 } else { pts[i].1 };
            let mut yaw = (b[0] - a[0]).atan2(b[1] - a[1]);
            if reverse {
                yaw += std::f32::consts::PI;
            }
            points.push((pts[i].0, yaw, t, reverse));
        }
        Route { points }
    }
}

/// The car `sim` would take to work (or school) now, and their seat in it (0: they drive): an
/// owner 18 or older takes their household's car from home when it's parked there; anyone else in
/// the household rides along when an owner is about to drive off in it (or on their way to).
pub(crate) fn seat(
    cars: &[Car],
    briefs: &[SimBrief],
    households: &[Household],
    sim: &Sim,
) -> Option<(u32, usize)> {
    let me = briefs.get(sim.id as usize)?;
    let home = households.get(sim.household as usize)?.plot?;
    if !sim.here() || me.baby || me.plot.is_some_and(|p| p != home) {
        return None;
    }
    cars.iter().enumerate().find_map(|(c, car)| {
        // At home, or still close by on the street.
        let near = me.plot == Some(home)
            || (sim.pos[0] - car.spot[0]).hypot(sim.pos[1] - car.spot[1]) < CALL_REACH;
        if car.household != sim.household || car.plot != home || car.blocked || !near {
            return None;
        }
        let c = c as u32;
        let drives =
            |j: u32| car.owners.contains(&j) && briefs.get(j as usize).is_some_and(|b| b.adult);
        // An owner on their way to drive it.
        let coming = || {
            briefs
                .iter()
                .enumerate()
                .any(|(j, b)| j as u32 != sim.id && b.bound_for_car == Some(c) && drives(j as u32))
        };
        // A seat of their own: after those in it and on their way to it as riders.
        let riders = car
            .aboard
            .iter()
            .filter(|&&a| Some(a) != car.driver)
            .count()
            + briefs
                .iter()
                .enumerate()
                .filter(|(j, b)| {
                    *j as u32 != sim.id && b.bound_for_car == Some(c) && !drives(*j as u32)
                })
                .count();
        let rider = riders + 1;
        // (Room for them past the driver and the riders before them, in it or on their way.)
        let room = rider < SEATS;
        match (car.state, car.driver) {
            (CarState::Parked, _) | (CarState::Boarding, None) if drives(sim.id) && !coming() => {
                Some((c, 0))
            }
            (CarState::Parked, _) | (CarState::Boarding, None) => {
                (room && coming()).then_some((c, rider))
            }
            (CarState::Boarding, Some(d)) if d != sim.id && room => Some((c, rider)),
            _ => None,
        }
    })
}

/// Where `sim` heads for work (or school) when they go by car: the car, the door they get in by
/// and where they stand for it. None: on foot.
pub(crate) fn ride_for(
    ctx: &crate::world::Ctx,
    sim: &Sim,
) -> Option<(u32, usize, usize, [f32; 2])> {
    let (c, seat) = seat(ctx.cars, ctx.briefs, ctx.households, sim)?;
    let (door, at) = ctx.cars[c as usize].door(&ctx.nav, seat)?;
    Some((c, seat, door, at))
}

/// Whether `i` can get into car `c` now (as `seat` found it on their way).
pub(crate) fn may_board(w: &World, i: usize, c: u32) -> bool {
    seat(&w.cars, &w.briefs, &w.households, &w.sims[i]).is_some_and(|(k, _)| k == c)
}

/// How much earlier than on foot `i` leaves for work or school when their household's car is
/// at home to take them: the walk to it, getting in, the drive out of town and on to work. 0
/// without one.
pub(crate) fn lead_ticks(w: &World, i: usize) -> u64 {
    let s = &w.sims[i];
    let home = w.households.get(s.household as usize).and_then(|h| h.plot);
    w.cars
        .iter()
        .find(|c| c.household == s.household && Some(c.plot) == home && c.at_home() && !c.blocked)
        .map_or(0, |c| {
            WALK_TO_CAR_TICKS
                + u64::from(GET_IN_TICKS)
                + BOARD_MIN_TICKS
                + c.out_ticks.ceil() as u64
                + OFF_MAP_TICKS
        })
}

/// `i` gets in (their shift is under way): the first owner in drives (someone in before them waits
/// for them). Should they come home some other way, it's from the end of town (as they'd have
/// walked there).
pub(crate) fn board(w: &mut World, i: usize, c: u32) {
    let tick = w.tick;
    to_exit(w, i);
    let Some(car) = w.cars.get_mut(c as usize) else {
        return;
    };
    if car.state == CarState::Parked {
        car.state = CarState::Boarding;
        car.since = tick;
        car.driver = None;
        car.aboard.clear();
        car.done.clear();
        car.ready_at = 0;
    }
    let adult = w.content.life.adult(w.sims[i].age);
    if car.driver.is_none() && adult && car.owners.contains(&(i as u32)) {
        car.driver = Some(i as u32);
    }
    if !car.aboard.contains(&(i as u32)) {
        car.aboard.push(i as u32);
    }
}

/// The car `i` is in, waiting for or riding home in after work (they're away, but not at work).
pub(crate) fn homeward(w: &World, i: usize) -> bool {
    w.cars.iter().any(|c| c.done.contains(&(i as u32)))
}

/// `i` is done at work (paid, or out of school): true when they wait there for their car home
/// (or go in it now), false when they make their own way (no car, or its driver works on for
/// longer than `RIDE_WAIT_TICKS`).
pub(crate) fn off_work(w: &mut World, i: usize) -> bool {
    let tick = w.tick;
    let me = i as u32;
    let Some(c) = w.cars.iter().position(|c| {
        c.aboard.contains(&me) && matches!(c.state, CarState::Leaving | CarState::Away)
    }) else {
        return false;
    };
    let driver = w.cars[c].driver;
    let wait = driver.is_some_and(|d| {
        d == me
            || w.cars[c].done.contains(&d)
            || w.sims[d as usize]
                .away_until
                .is_some_and(|until| until <= tick + RIDE_WAIT_TICKS)
    });
    let car = &mut w.cars[c];
    if !wait {
        car.aboard.retain(|&a| a != me);
        return false;
    }
    car.done.push(me);
    car.ready_at = car.ready_at.max(tick + OFF_MAP_TICKS);
    w.sims[i].away_until = Some(tick + 1);
    true
}

/// `i` stands at the town's exit nearest them (out of sight).
fn to_exit(w: &mut World, i: usize) {
    if let Some(at) = nearest_exit(&w.exits, w.sims[i].pos) {
        w.sims[i].pos = at;
    }
}

/// Once a tick (after `life::update`): owners, cars going with their household, setting off and
/// coming home, what can't be used, and the tiles parked cars take up.
pub(crate) fn update(w: &mut World) {
    if w.cars.is_empty() {
        return;
    }
    let tick = w.tick;
    if w.cars_nav != w.nav_version {
        w.cars_nav = w.nav_version;
        w.cars_parked = None;
        refresh(w);
    }
    for c in 0..w.cars.len() {
        owners(w, c);
        relocate(w, c);
        let gone = |w: &World, s: u32| w.sims.get(s as usize).is_none_or(|s| !s.here());
        // Nobody to drive it home (the driver died or moved away while out): it's back.
        if !w.cars[c].at_home() && w.cars[c].driver.is_none_or(|d| gone(w, d)) {
            let car = &mut w.cars[c];
            let left = std::mem::take(&mut car.done);
            car.aboard.clear();
            car.state = CarState::Parked;
            car.driver = None;
            for s in left {
                w.sims[s as usize].away_until = None;
                walk_home(w, s as usize, None);
            }
            continue;
        }
        let state = w.cars[c].state;
        let since = w.cars[c].since;
        match state {
            CarState::Parked => {
                // An owner on their way to drive it: the others setting off now come along.
                let bound = |s: &Sim| {
                    s.bound_for_car == Some(c as u32)
                        && s.current()
                            .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
                };
                if w.cars[c].owners.iter().any(|&o| bound(&w.sims[o as usize])) {
                    one_driver(w, c);
                    call_riders(w, c);
                }
            }
            CarState::Boarding => {
                let driver = w.cars[c].driver;
                let waited = tick - since;
                // No driver after all (the one coming was called away, or left town): those in it
                // have gone on (they're at work).
                if driver.is_some_and(|d| gone(w, d))
                    || (driver.is_none() && waited >= BOARD_WAIT_TICKS)
                {
                    let car = &mut w.cars[c];
                    car.state = CarState::Parked;
                    car.driver = None;
                    car.aboard.clear();
                    continue;
                }
                let expected = call_riders(w, c);
                if driver.is_none() {
                    continue;
                }
                if (!expected && waited >= BOARD_MIN_TICKS) || waited >= BOARD_WAIT_TICKS {
                    let car = &mut w.cars[c];
                    car.state = CarState::Leaving;
                    car.since = tick;
                    car.progress = 0.0;
                }
            }
            CarState::Leaving => {
                let ticks = route_ticks(w, c, false);
                drive_on(w, c);
                if w.cars[c].progress >= ticks {
                    w.cars[c].state = CarState::Away;
                    w.cars[c].since = tick;
                }
            }
            CarState::Away => {
                // Those still at work who won't be done in time make their own way home.
                let car = &w.cars[c];
                let driver_done = car.driver.is_some_and(|d| car.done.contains(&d));
                let stays: Vec<u32> = car
                    .aboard
                    .iter()
                    .copied()
                    .filter(|a| {
                        !car.done.contains(a)
                            && (gone(w, *a)
                                || (driver_done
                                    && w.sims[*a as usize]
                                        .away_until
                                        .is_none_or(|u| u > tick + RIDE_WAIT_TICKS)))
                    })
                    .collect();
                w.cars[c].aboard.retain(|a| !stays.contains(a));
                let car = &w.cars[c];
                if driver_done
                    && car.aboard.iter().all(|a| car.done.contains(a))
                    && tick >= car.ready_at
                {
                    let car = &mut w.cars[c];
                    car.state = CarState::Arriving;
                    car.since = tick;
                    car.progress = 0.0;
                }
                // Home once the last of them is done, back in town and driven home.
                let car = &w.cars[c];
                let last = car
                    .aboard
                    .iter()
                    .filter(|a| !car.done.contains(a))
                    .filter_map(|&a| w.sims[a as usize].away_until)
                    .map(|u| u + OFF_MAP_TICKS)
                    .max()
                    .unwrap_or(tick)
                    .max(car.ready_at);
                let drive = route_ticks(w, c, true);
                keep_away(w, c, last.max(tick) + drive.ceil() as u64);
            }
            CarState::Arriving => {
                let ticks = route_ticks(w, c, true);
                drive_on(w, c);
                let left = ticks - w.cars[c].progress;
                if left <= 0.0 {
                    arrive(w, c);
                } else {
                    keep_away(w, c, tick + left.ceil() as u64);
                }
            }
        }
    }
    reblock(w);
    poses(w);
}

/// Ticks car `c`'s drive out of town or (`home`) back takes.
fn route_ticks(w: &mut World, c: usize, home: bool) -> f32 {
    let World { cars, exits, .. } = w;
    cars[c].route(exits, home).ticks()
}

/// A tick further along its drive, unless someone is in its way: it waits for them. Someone
/// standing there a while (a chat on the pavement) steps aside for it (`make_way`), unless
/// they're at something.
fn drive_on(w: &mut World, c: usize) {
    let home = w.cars[c].state == CarState::Arriving;
    let t = w.cars[c].progress;
    let World {
        cars,
        exits,
        sims,
        lot,
        blocked,
        stairs,
        ..
    } = w;
    let car = &mut cars[c];
    let (at, yaw, reverse) = car.route(exits, home).at(t);
    // (Ahead is behind it while it backs up.)
    let (ahead, behind) = if reverse { (0.2, AHEAD) } else { (AHEAD, 0.2) };
    let mut waiting = false;
    for s in sims.iter_mut() {
        if !s.here() || s.away_until.is_some() || !car.covers(at, yaw, s.pos, ahead, behind) {
            continue;
        }
        waiting = true;
        let busy = s
            .current()
            .is_some_and(|a| matches!(a.phase, Phase::Using { .. }));
        if car.waited > MAKE_WAY_TICKS && !busy {
            let nav = crate::path::NavGrid {
                lot,
                blocked,
                stairs,
            };
            if let Some(p) = make_way(at, yaw, &nav, s.pos, 0.06) {
                s.pos = p;
            }
        }
    }
    if waiting {
        car.waited += 1;
    } else {
        car.waited = 0;
        car.progress += 1.0;
    }
}

/// Getting into `car` by their door, `t` ticks in (`Phase::Boarding`): at the door while it
/// opens, a step in, sat in their seat while it shuts.
pub(crate) fn getting_in(sim: &mut Sim, car: &Car, t: u32) {
    let d = usize::from(sim.car_door).min(DOORS - 1);
    let (door, seat) = (car.door_at(d), car.seat_at(d));
    if t < DOOR_OPENS {
        return;
    }
    let k = ((t - DOOR_OPENS) as f32 / STEP_IN as f32).min(1.0);
    if k < 1.0 {
        sim.pos = [
            door[0] + (seat[0] - door[0]) * k,
            door[1] + (seat[1] - door[1]) * k,
        ];
        sim.yaw = (seat[0] - door[0]).atan2(seat[1] - door[1]);
        sim.pose = crate::content::Pose::Stand;
    } else {
        sim.pos = seat;
        sim.yaw = car.yaw;
        sim.pose = crate::content::Pose::Sit;
    }
}

/// Getting out of `car` by their door, `t` ticks in (`Phase::Alighting`): sat while it opens,
/// then a step out to beside it.
pub(crate) fn getting_out(sim: &mut Sim, car: &Car, t: u32) {
    let d = usize::from(sim.car_door).min(DOORS - 1);
    let (door, seat) = (car.door_at(d), car.seat_at(d));
    if t < DOOR_OPENS {
        sim.pos = seat;
        sim.yaw = car.yaw;
        sim.pose = crate::content::Pose::Sit;
        return;
    }
    let k = ((t - DOOR_OPENS) as f32 / STEP_IN as f32).min(1.0);
    sim.pos = [
        seat[0] + (door[0] - seat[0]) * k,
        seat[1] + (door[1] - seat[1]) * k,
    ];
    sim.yaw = (door[0] - seat[0]).atan2(door[1] - seat[1]);
    sim.pose = crate::content::Pose::Stand;
}

/// The car driving by that a step from `from` to `to` would walk into (stepping out of its way
/// is fine).
pub(crate) fn in_the_way(cars: &[Car], from: [f32; 2], to: [f32; 2]) -> Option<usize> {
    cars.iter().position(|car| {
        let Some((at, yaw, true, _)) = car.shown else {
            return false;
        };
        let d = |p: [f32; 2]| (p[0] - at[0]).hypot(p[1] - at[1]);
        car.covers(at, yaw, to, 0.0, 0.0)
            && !(car.covers(at, yaw, from, 0.0, 0.0) && d(to) > d(from))
    })
}

/// Someone has waited a while for a car (at `at`, facing `yaw`) to pass, standing in its way: a
/// step aside (across the way it drives, away from it), where there's ground to stand on. None:
/// nowhere to go.
pub(crate) fn make_way(
    at: [f32; 2],
    yaw: f32,
    nav: &crate::path::NavGrid,
    pos: [f32; 2],
    step: f32,
) -> Option<[f32; 2]> {
    let (s, c) = yaw.sin_cos();
    let left = [c, -s];
    let across = (pos[0] - at[0]) * left[0] + (pos[1] - at[1]) * left[1];
    let k = if across >= 0.0 { 1.0 } else { -1.0 };
    let (fx, fz) = (pos[0].floor() as i32, pos[1].floor() as i32);
    [k, -k]
        .into_iter()
        .map(|k| [pos[0] + left[0] * step * k, pos[1] + left[1] * step * k])
        .find(|p| {
            let (x, z) = (p[0].floor() as i32, p[1].floor() as i32);
            // (A step to the next tile at most.)
            let near = (x - fx).abs() <= 1 && (z - fz).abs() <= 1;
            nav.tile_free(x, z) && near && ((x, z) == (fx, fz) || nav.can_step(fx, fz, x, z))
        })
}

/// Whether car `c` stands on its household's driveway, to be seen and used (not built over).
fn present(w: &World, c: usize) -> bool {
    let car = &w.cars[c];
    car.at_home()
        && !car.blocked
        && w.households
            .get(car.household as usize)
            .is_some_and(|h| h.plot == Some(car.plot))
}

/// Where every car is now and which doors are open, for the snapshot (`Car::shown`): the doors
/// of those getting in, and of those who just got in or out.
pub(crate) fn poses(w: &mut World) {
    let mut doors = vec![0u8; w.cars.len()];
    for s in &w.sims {
        // Open from when they're at it until they're sat in (or out and stood up).
        let open = s.current().is_some_and(|a| match a.phase {
            Phase::Boarding { ticks } => ticks < DOOR_OPENS + STEP_IN,
            Phase::Alighting { ticks } => ticks < GET_OUT_TICKS,
            _ => false,
        });
        if let Some(c) = s.bound_for_car.filter(|_| open)
            && let Some(bits) = doors.get_mut(c as usize)
        {
            *bits |= 1 << s.car_door.min(DOORS as u8 - 1);
        }
    }
    let present: Vec<bool> = (0..w.cars.len()).map(|c| present(w, c)).collect();
    let World { cars, exits, .. } = w;
    for (c, car) in cars.iter_mut().enumerate() {
        let open = doors[c];
        car.shown = if car.at_home() && !present[c] {
            None
        } else {
            car.pose(exits)
                .map(|(p, yaw, moving)| (p, yaw, moving, if moving { 0 } else { open }))
        };
    }
}

/// Those done at work for the car's way home stay away until it's back (`until`: the time shown
/// for them).
fn keep_away(w: &mut World, c: usize, until: u64) {
    let until = until.max(w.tick + 1);
    for &s in &w.cars[c].done {
        w.sims[s as usize].away_until = Some(until);
    }
}

/// The car is home: it parks, nose in, its doors open and they get out and walk to the house.
fn arrive(w: &mut World, c: usize) {
    let tick = w.tick;
    let World { cars, exits, .. } = &mut *w;
    let car = &mut cars[c];
    car.state = CarState::Parked;
    car.since = tick;
    car.yaw = car.nose_in();
    car.settle(exits);
    let driver = car.driver.take();
    let mut out = std::mem::take(&mut car.done);
    car.aboard.clear();
    // (The tiles under it before anyone gets out beside it.)
    reblock(w);
    // The driver first, then the others, a seat each.
    out.sort_by_key(|&s| Some(s) != driver);
    for (seat, s) in out.into_iter().enumerate() {
        let i = s as usize;
        w.sims[i].away_until = None;
        walk_home(w, i, Some((c, seat)));
    }
}

/// `i` sets off home on foot: out of the car (`by`: the car and their seat in it), sat in it as
/// its door opens, then out beside it (`Phase::Alighting`); else from where they are.
fn walk_home(w: &mut World, i: usize, by: Option<(usize, usize)>) {
    if let Some((c, seat)) = by {
        let door = w.cars[c].door(&w.nav(), seat).map(|(d, _)| d);
        if let Some(door) = door {
            let car = &w.cars[c];
            let (at, yaw) = (car.seat_at(door), car.yaw);
            let sim = &mut w.sims[i];
            sim.pos = at;
            sim.yaw = yaw;
            sim.pose = crate::content::Pose::Sit;
            sim.bound_for_car = Some(c as u32);
            sim.car_door = door as u8;
            sim.current = Some(crate::world::Activity {
                task: Task {
                    kind: TaskKind::GoHome,
                    directed: false,
                    routine: false,
                },
                phase: Phase::Alighting { ticks: 0 },
                tags: Default::default(),
            });
            return;
        }
    }
    let sim = &mut w.sims[i];
    if !sim.queue.iter().any(|t| matches!(t.kind, TaskKind::GoHome)) {
        sim.queue.push_front(Task {
            kind: TaskKind::GoHome,
            directed: false,
            routine: false,
        });
    }
}

/// While an owner goes to the car or waits in it: the rest of the household setting off now is
/// called to it (those on their way to the exit turn round, if they're still close by). Whether
/// anyone is on their way to it.
fn call_riders(w: &mut World, c: usize) -> bool {
    let car = &w.cars[c];
    let (household, plot) = (car.household, car.plot);
    let mut expected = false;
    for i in 0..w.sims.len() {
        let s = &w.sims[i];
        if !s.here()
            || s.household != household
            || s.away_until.is_some()
            || w.cars[c].aboard.contains(&(i as u32))
        {
            continue;
        }
        let heading_out = s
            .current()
            .is_some_and(|a| matches!(a.task.kind, TaskKind::Work));
        if !heading_out
            || w.briefs
                .get(i)
                .is_none_or(|b| b.plot.is_some_and(|p| p != plot))
        {
            continue;
        }
        if s.bound_for_car == Some(c as u32) {
            expected = true;
            continue;
        }
        // On their way to the exit: to the car instead (`ride_for` picks it now).
        if may_board(w, i, c as u32) {
            let sim = &mut w.sims[i];
            if let Some(a) = sim.current.take() {
                sim.queue.push_front(a.task);
            }
            expected = true;
        }
    }
    expected
}

/// Two owners set off for the car at once, both to drive it: the first drives, the others find
/// themselves a seat as riders (they head for the car again, `seat` seeing the driver coming).
fn one_driver(w: &mut World, c: usize) {
    let drivers: Vec<usize> = (0..w.sims.len())
        .filter(|&i| {
            let s = &w.sims[i];
            s.bound_for_car == Some(c as u32)
                && s.car_seat == 0
                && s.current().is_some_and(|a| {
                    matches!(a.task.kind, TaskKind::Work)
                        && matches!(a.phase, Phase::Routing { .. })
                })
        })
        .collect();
    for &i in drivers.iter().skip(1) {
        let sim = &mut w.sims[i];
        sim.bound_for_car = None;
        if let Some(a) = sim.current.take() {
            sim.queue.push_front(a.task);
        }
    }
}

/// Owners who left town or the household no longer own it; a household left with none hands it
/// down to its grown-ups.
fn owners(w: &mut World, c: usize) {
    let household = w.cars[c].household;
    let sims = &w.sims;
    w.cars[c].owners.retain(|&o| {
        sims.get(o as usize)
            .is_some_and(|s| s.here() && s.household == household)
    });
    if w.cars[c].owners.is_empty() {
        let content = &w.content;
        let heirs: Vec<u32> = w
            .sims
            .iter()
            .filter(|s| s.here() && s.household == household && s.adult(content))
            .map(|s| s.id)
            .collect();
        w.cars[c].owners = heirs;
    }
}

/// The household moved house: the car goes with it, to the new home's driveway (parked there
/// nose in), once it's parked or out with them. None there (or another car on it): it stays where
/// it was, unused, out of sight.
fn relocate(w: &mut World, c: usize) {
    let car = &w.cars[c];
    let Some(home) = w
        .households
        .get(car.household as usize)
        .and_then(|h| h.plot)
    else {
        return;
    };
    if home == car.plot || !matches!(car.state, CarState::Parked | CarState::Away) {
        return;
    }
    let taken = w.cars.iter().enumerate().any(|(k, o)| {
        k != c
            && o.plot == home
            && w.households
                .get(o.household as usize)
                .is_some_and(|h| h.plot == Some(home))
    });
    let Some(drive) = w.drives.iter().find(|d| d.plot == home).copied() else {
        return;
    };
    if taken {
        return;
    }
    let World { cars, exits, .. } = &mut *w;
    let car = &mut cars[c];
    car.plot = home;
    car.street = drive.street;
    car.spot = [drive.x, drive.z + drive.front * (PARK_GAP + car.len / 2.0)];
    car.yaw = car.nose_in();
    car.settle(exits);
    // (Parked as it was, somewhere else: its tiles are worked out again.)
    w.cars_parked = None;
    refresh(w);
}

/// Which cars' parking spots have been built over: a wall or a fence across one, a thing on the
/// tiles it takes up.
fn refresh(w: &mut World) {
    let lot = &w.lot;
    let mut things: std::collections::HashSet<(i32, i32)> = Default::default();
    let plots: Vec<u32> = w.cars.iter().map(|c| c.plot).collect();
    for (o, plot) in w.objects.iter().zip(&w.object_plot) {
        if plot.is_some_and(|p| plots.contains(&p)) {
            things.extend(w.blocking_tiles(o));
        }
    }
    for car in &mut w.cars {
        let tiles = car.tiles();
        let walled = |x: i32, z: i32| {
            // An edge inside the car's tiles (not round their outline).
            let inside = |x: i32, z: i32| tiles.contains(&(x, z));
            (inside(x, z)
                && inside(x, z - 1)
                && lot.h_edge(x as usize, z as usize) != crate::lot::Edge::Open)
                || (inside(x, z)
                    && inside(x - 1, z)
                    && lot.v_edge(x as usize, z as usize) != crate::lot::Edge::Open)
        };
        car.blocked = tiles.iter().any(|&(x, z)| {
            !lot.in_bounds(x, z)
                || things.contains(&(x, z))
                || lot.diag_blocks(x, z)
                || walled(x, z)
        });
    }
}

/// The tiles parked cars and garages take up: nobody walks through a car or a garage, and nothing is put where one stands
/// (`World::blocked`; things never stand on them, see `refresh`). Walkers find their way again
/// when that changes (the walkable regions, not the views: `nav_version` only).
pub(crate) fn reblock(w: &mut World) {
    // (Nothing to do while the same cars stand parked: `MAX_CARS` bits.)
    let parked = (0..w.cars.len())
        .filter(|&c| present(w, c))
        .fold(0u64, |m, c| m | 1 << c);
    if w.cars_parked == Some(parked) {
        return;
    }
    w.cars_parked = Some(parked);
    let mut tiles: Vec<usize> = Vec::new();
    // (Garages too, as they stand.)
    for g in &w.garages {
        for (x, z) in g.tiles() {
            if w.lot.in_bounds(x, z) {
                tiles.push(w.lot.tile_index(x, z));
            }
        }
    }
    for c in 0..w.cars.len() {
        if parked & 1 << c != 0 {
            for (x, z) in w.cars[c].tiles() {
                if w.lot.in_bounds(x, z) {
                    tiles.push(w.lot.tile_index(x, z));
                }
            }
        }
    }
    tiles.sort_unstable();
    tiles.dedup();
    if tiles == w.car_tiles {
        return;
    }
    for &i in &w.car_tiles {
        w.blocked[i] = false;
    }
    for &i in &tiles {
        w.blocked[i] = true;
    }
    w.car_tiles = tiles;
    w.nav_version = w.nav_version.wrapping_add(1);
    w.cars_nav = w.nav_version;
}

/// The cars of a saved game: those that still make sense (their plot, household and people are
/// there, their numbers are numbers), every house's driveway, and the garages.
pub(crate) fn load(w: &mut World, saved: &[Car], drives: &[Drive], garages: &[Garage]) {
    let sims = w.sims.len() as u32;
    let finite = |v: &[f32]| v.iter().all(|v| v.is_finite());
    let ok = |c: &Car| {
        (c.plot as usize) < w.plots.len()
            && (c.household as usize) < w.households.len()
            && finite(&[c.spot[0], c.spot[1], c.yaw, c.street, c.len, c.wid])
            && c.len > 0.0
            && c.wid > 0.0
    };
    w.drives = drives
        .iter()
        .filter(|d| (d.plot as usize) < w.plots.len() && finite(&[d.x, d.z, d.front, d.street]))
        .copied()
        .collect();
    w.garages = garages
        .iter()
        .filter(|g| (g.plot as usize) < w.plots.len() && finite(&[g.x0, g.z0, g.x1, g.z1]))
        .copied()
        .collect();
    w.cars = saved
        .iter()
        .filter(|c| ok(c))
        .take(MAX_CARS)
        .cloned()
        .collect();
    for car in &mut w.cars {
        car.owners.retain(|&o| o < sims);
        car.aboard.retain(|&o| o < sims);
        car.done.retain(|o| car.aboard.contains(o));
        car.driver = car.driver.filter(|d| car.aboard.contains(d));
    }
    settle_all(w);
}

/// Builds the town's cars from its file: each goes to the household living on its plot (none
/// there: no car), owned by its grown-ups.
pub(crate) fn from_file(w: &mut World, raw: &[CarRaw], drives: &[Drive], garages: &[Garage]) {
    w.drives = drives.to_vec();
    w.garages = garages.to_vec();
    w.cars = raw
        .iter()
        .filter_map(|r| {
            let h = w.households.iter().position(|h| h.plot == Some(r.plot))?;
            let mut car = Car::from_raw(r, h as u32);
            car.owners = w
                .sims
                .iter()
                .filter(|s| s.here() && s.household == h as u32 && s.adult(&w.content))
                .map(|s| s.id)
                .collect();
            Some(car)
        })
        .take(MAX_CARS)
        .collect();
    settle_all(w);
}

/// After the cars were made or loaded: their routes' lengths, what's built over, the tiles they
/// take up, where they are.
fn settle_all(w: &mut World) {
    let World { cars, exits, .. } = &mut *w;
    for car in cars.iter_mut() {
        car.settle(exits);
    }
    w.car_tiles.clear();
    w.cars_parked = None;
    refresh(w);
    reblock(w);
    poses(w);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock;

    const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
        "objects":[{"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"tags":["lounge"]}]}],
        "careers":[{"id":"office","label":"Office","workEffects":{"hunger":-0.2},"levels":[
            {"title":"Intern","pay":100,"start":9,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
        "economy":{"startingFunds":500}}"#;

    /// A house on a street running along x (its middle at z = 9.5), the exits at both ends; the
    /// car parked beside the house facing the street. Two grown-ups at the same office.
    const TOWN: &str = r#"{"width":60,"depth":14,"exits":[[0.5,9.5],[59.5,9.5]],
        "plots":[{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}],
        "households":[{"name":"Ones","plot":0}],
        "sims":[{"name":"Ada","household":0,"job":{"career":"office"},"x":25.5,"z":3.5},
                {"name":"Bo","household":0,"job":{"career":"office"},"x":26.5,"z":3.5}],
        "cars":[{"plot":0,"x":34.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":2}]}"#;

    fn until(w: &mut World, day: u32, hour: f32) {
        let target = clock::tick_at(day, hour * 60.0).unwrap();
        while w.tick < target {
            w.tick_once();
        }
    }

    #[test]
    fn owners_drive_to_work_together_and_come_home_nose_in() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        assert_eq!(w.cars.len(), 1);
        assert_eq!(w.cars[0].owners, vec![0, 1], "the grown-ups own it");
        until(&mut w, 2, 0.0);
        let funds = w.households[0].funds;
        let mut seen = Vec::new();
        let target = clock::tick_at(2, 10.0 * 60.0).unwrap();
        while w.tick < target {
            w.tick_once();
            let s = w.cars[0].state;
            if seen.last() != Some(&s) {
                seen.push(s);
            }
        }
        assert_eq!(
            seen,
            [
                CarState::Parked,
                CarState::Boarding,
                CarState::Leaving,
                CarState::Away
            ],
            "off to work by car"
        );
        assert_eq!(w.cars[0].aboard.len(), 2, "both went in it");
        assert!(w.sims.iter().all(|s| s.away_until.is_some()));
        until(&mut w, 2, 13.0 + 1.0 / 60.0);
        assert_eq!(
            w.households[0].funds,
            funds + 200,
            "a shift each, paid once"
        );
        assert!(
            homeward(&w, 0) && homeward(&w, 1),
            "waiting for the car home"
        );
        until(&mut w, 2, 14.0);
        assert_eq!(w.cars[0].state, CarState::Parked, "home again");
        assert!(
            (w.cars[0].yaw.abs() - std::f32::consts::PI).abs() < 1e-3,
            "nose in"
        );
        assert!(w.sims.iter().all(|s| s.away_until.is_none()));
        assert_eq!(w.households[0].funds, funds + 200);
        until(&mut w, 2, 14.5);
        for s in &w.sims {
            assert_eq!(
                w.plot_at(s.tile().0, s.tile().1),
                Some(0),
                "{} walked in from the car",
                s.name
            );
        }
    }

    #[test]
    fn only_owners_drive() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        // Bo owns it but has no job; Ada may not drive it, so walks.
        w.cars[0].owners = vec![1];
        w.sims[1].job = None;
        until(&mut w, 2, 10.0);
        assert!(w.sims[0].away_until.is_some(), "at work");
        assert_eq!(w.cars[0].state, CarState::Parked, "nobody drove it");
        assert!(w.cars[0].owners == vec![1]);
    }

    #[test]
    fn the_rest_of_the_household_rides_along() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        // Only Bo owns it: Bo drives, Ada (setting off at the same time) rides.
        w.cars[0].owners = vec![1];
        until(&mut w, 2, 10.0);
        assert_eq!(w.cars[0].state, CarState::Away);
        assert_eq!(w.cars[0].driver, Some(1));
        assert!(w.cars[0].aboard.contains(&0), "Ada went with Bo");
    }

    #[test]
    fn a_teenager_never_drives() {
        let content = CONTENT.replace(
            r#""economy""#,
            r#""life":{"stages":[{"id":"teen","label":"Teen","from":13,"adult":false},{"id":"youngAdult","label":"Young adult","from":18,"adult":true}]},"economy""#,
        );
        let town = TOWN.replace(
            r#""name":"Ada","household":0,"#,
            r#""name":"Ada","household":0,"age":16,"#,
        );
        let mut w = World::from_json(&content, &town, 1).unwrap();
        assert_eq!(w.cars[0].owners, vec![1], "only the grown-up owns it");
        // Even listed as an owner, a sixteen-year-old with a job doesn't drive; Bo stays home.
        w.cars[0].owners = vec![0, 1];
        w.sims[0].job = Some(crate::life::Job::new(0, 0, 0));
        w.sims[1].job = None;
        until(&mut w, 2, 10.0);
        assert!(w.sims[0].away_until.is_some(), "Ada went, on foot");
        assert_eq!(w.cars[0].state, CarState::Parked);
    }

    #[test]
    fn a_parked_car_takes_up_its_tiles() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        let tiles: Vec<usize> = w.cars[0]
            .tiles()
            .iter()
            .map(|&(x, z)| w.lot.tile_index(x, z))
            .collect();
        assert!(tiles.len() >= 6, "{tiles:?}");
        assert!(
            tiles.iter().all(|&i| w.blocked[i]),
            "nobody walks through it, nothing goes there"
        );
        assert!(
            w.place(
                0,
                34,
                3,
                0,
                Default::default(),
                Default::default(),
                0,
                0,
                None
            )
            .is_err(),
            "no chair on it"
        );
        until(&mut w, 2, 9.5);
        assert_eq!(w.cars[0].state, CarState::Away);
        assert!(tiles.iter().all(|&i| !w.blocked[i]), "free while it's out");
        until(&mut w, 2, 14.0);
        assert_eq!(w.cars[0].state, CarState::Parked);
        assert!(tiles.iter().all(|&i| w.blocked[i]), "taken again");
    }

    #[test]
    fn they_open_its_doors_and_set_off_before_work_starts() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        until(&mut w, 2, 0.0);
        let start = clock::tick_at(2, 9.0 * 60.0).unwrap();
        let mut opened = 0u8;
        let mut getting_in = false;
        while w.tick < start {
            w.tick_once();
            opened |= w.cars[0].shown.map_or(0, |s| s.3);
            getting_in |= w.sims.iter().any(|s| {
                s.current()
                    .is_some_and(|a| matches!(a.phase, Phase::Boarding { .. }))
            });
        }
        assert!(getting_in, "they stood at its doors");
        assert_eq!(opened & 1, 1, "the driver's door opened");
        assert!(opened.count_ones() >= 2, "and the rider's: {opened:b}");
        assert_eq!(w.cars[0].state, CarState::Away, "out of town before nine");
        assert!(
            !w.events
                .iter()
                .any(|e| e.kind == crate::social::EventKind::MissedWork)
        );
        // On the road, not at work yet: the shift starts at nine.
        assert!(w.sims.iter().all(|s| s.shift_from == start));
    }

    #[test]
    fn the_car_moves_house_with_its_household() {
        let town = TOWN
            .replace(
                r#"{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}"#,
                r#"{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]},{"name":"Next","x":40,"z":0,"w":20,"d":8,"entry":[45.5,7.5]}"#,
            )
            .replace(
                r#""cars":["#,
                r#""drives":[{"plot":0,"x":34.0,"z":0.55,"front":1.0,"street":9.5},{"plot":1,"x":54.0,"z":1.0,"front":1.0,"street":9.5}],"cars":["#,
            );
        let mut w = World::from_json(CONTENT, &town, 1).unwrap();
        w.households[0].plot = Some(1);
        w.tick_once();
        let car = &w.cars[0];
        assert_eq!(car.plot, 1, "it went along");
        assert!(
            (car.spot[0] - 54.0).abs() < 1e-3
                && (car.spot[1] - (1.0 + PARK_GAP + 2.2)).abs() < 1e-3,
            "{:?}",
            car.spot
        );
        assert!(
            (car.yaw.abs() - std::f32::consts::PI).abs() < 1e-3,
            "nose in"
        );
        let old = w.lot.tile_index(34, 3);
        assert!(!w.blocked[old], "the old driveway is free");
        assert!(
            car.tiles()
                .iter()
                .all(|&(x, z)| w.blocked[w.lot.tile_index(x, z)])
        );
    }

    #[test]
    fn a_door_with_no_room_to_open_isnt_used() {
        // The house's wall a metre from the car's left (driver's) side.
        let town = TOWN
            .replace(r#""x":34.0,"z":3.0"#, r#""x":34.5,"z":3.0"#)
            .replace(
                r#""sims":"#,
                r#""walls":[[36,0,40,0],[40,0,40,6],[36,6,40,6],[36,0,36,6]],"sims":"#,
            );
        let w = World::from_json(CONTENT, &town, 1).unwrap();
        let (door, _) = w.cars[0].door(&w.nav(), 0).unwrap();
        assert_eq!(door, 1, "the driver gets in on the other side");
    }

    #[test]
    fn the_door_shuts_before_they_are_gone() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        until(&mut w, 2, 0.0);
        let start = clock::tick_at(2, 9.0 * 60.0).unwrap();
        let (mut last_open, mut gone, mut sat) = (0, None, false);
        while w.tick < start && gone.is_none() {
            w.tick_once();
            if w.cars[0].shown.is_some_and(|s| s.3 != 0) {
                last_open = w.tick;
            }
            sat |= w
                .sims
                .iter()
                .any(|s| s.away_until.is_none() && s.pose == crate::content::Pose::Sit);
            if w.sims.iter().any(|s| s.away_until.is_some()) {
                gone = Some(w.tick);
            }
        }
        assert!(sat, "sat in the car first");
        assert!(
            gone.unwrap() >= last_open + 10,
            "the door shut ({last_open}) before they went ({gone:?})"
        );
    }

    #[test]
    fn a_car_waits_for_someone_in_its_way_who_then_steps_aside() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        w.autonomy = false;
        // Ada drives off (at work as far as anyone can tell)...
        w.cars[0].state = CarState::Leaving;
        w.cars[0].progress = 0.0;
        w.cars[0].driver = Some(0);
        w.cars[0].aboard = vec![0];
        w.sims[0].away_until = Some(w.tick + 100_000);
        // ...and Bo stands right in front of it (it faces the street).
        w.sims[1].pos = [34.0, 6.5];
        for _ in 0..20 {
            w.tick_once();
        }
        assert_eq!(w.cars[0].progress, 0.0, "it waits");
        for _ in 0..600 {
            w.tick_once();
        }
        assert!(
            w.cars[0].progress > 0.0,
            "and goes once Bo has stepped aside"
        );
        assert!(
            (w.sims[1].pos[0] - 34.0).abs() > 1.0,
            "Bo stepped aside: {:?}",
            w.sims[1].pos
        );
    }

    #[test]
    fn nobody_walks_into_a_car_driving_by() {
        let mut c = car([10.0, 10.0], 0.0, 20.0);
        c.shown = Some(([10.0, 10.0], 0.0, true, 0));
        let cars = [c];
        assert_eq!(
            in_the_way(&cars, [12.0, 10.0], [11.0, 10.0]),
            Some(0),
            "into its side"
        );
        assert_eq!(
            in_the_way(&cars, [10.8, 10.0], [11.5, 10.0]),
            None,
            "out of its way"
        );
        assert_eq!(
            in_the_way(&cars, [14.0, 10.0], [13.5, 10.0]),
            None,
            "well clear"
        );
    }

    #[test]
    fn a_garage_takes_up_its_ground() {
        let town = TOWN.replace(
            r#""cars":["#,
            r#""garages":[{"plot":0,"x0":40.0,"z0":1.0,"x1":43.2,"z1":6.4}],"cars":["#,
        );
        let mut w = World::from_json(CONTENT, &town, 1).unwrap();
        for (x, z) in [(40, 1), (42, 5), (41, 3)] {
            assert!(
                w.blocked[w.lot.tile_index(x, z)],
                "({x}, {z}) is the garage's"
            );
        }
        assert!(
            w.place(
                0,
                41,
                3,
                0,
                Default::default(),
                Default::default(),
                0,
                0,
                None
            )
            .is_err(),
            "no chair in it"
        );
        // Not lost on a save, nor freed as cars come and go.
        until(&mut w, 2, 10.0);
        let back = World::from_save_json(CONTENT, &w.save_json()).unwrap();
        assert_eq!(back.garages, w.garages);
        assert!(back.blocked[back.lot.tile_index(41, 3)]);
    }

    #[test]
    fn a_car_built_over_stays_put_and_survives_a_save() {
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        for x in 30..38 {
            w.lot.set_h(x, 3, crate::lot::Edge::Fence);
        }
        w.nav_version += 1;
        until(&mut w, 2, 10.0);
        assert!(w.cars[0].blocked);
        assert_eq!(w.cars[0].state, CarState::Parked);
        assert!(w.cars[0].shown.is_none(), "not drawn");
        assert!(w.sims.iter().all(|s| s.away_until.is_some()), "they walked");

        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        until(&mut w, 2, 9.5);
        assert_eq!(w.cars[0].state, CarState::Away);
        let saved = w.save_json();
        let mut back = World::from_save_json(CONTENT, &saved).unwrap();
        assert_eq!(back.cars[0].state, CarState::Away);
        assert_eq!(back.cars[0].aboard, w.cars[0].aboard);
        until(&mut back, 2, 14.0);
        assert_eq!(
            back.cars[0].state,
            CarState::Parked,
            "back after the load too"
        );
    }

    fn car(spot: [f32; 2], yaw: f32, street: f32) -> Car {
        Car::from_raw(
            &CarRaw {
                plot: 0,
                x: spot[0],
                z: spot[1],
                yaw,
                street,
                len: 4.4,
                wid: 1.8,
                body: 0,
                paint: 0,
            },
            0,
        )
    }

    #[test]
    fn pulls_out_and_drives_to_the_nearer_end() {
        // Street along x at z = 20, exits at both ends; the car faces the street.
        let exits = [[0.5, 20.0], [99.5, 20.0]];
        let mut c = car([70.0, 10.0], 0.0, 20.0);
        let r = Route::new(&c, &exits, false);
        let last = r.points.last().unwrap().0;
        assert!(last[0] > 99.5, "drives to the east end: {last:?}");
        assert!((last[1] - (20.0 + LANE)).abs() < 1e-3, "in its lane");
        // Starts from the spot facing the street, ends heading east.
        assert!((r.points[0].0[0] - 70.0).abs() < 1e-3);
        assert!(r.points[0].1.abs() < 0.2);
        assert!((r.points.last().unwrap().1 - std::f32::consts::FRAC_PI_2).abs() < 0.1);
        // Takes a while, but not forever.
        let secs = r.ticks() / TICKS_PER_SECOND as f32;
        assert!(secs > 5.0 && secs < 40.0, "{secs} s");
        // The pose follows the route.
        c.state = CarState::Leaving;
        assert!(
            c.pose(&exits)
                .is_some_and(|(p, _, moving)| moving && (p[0] - 70.0).abs() < 1e-3)
        );
        c.progress = r.ticks() + 1.0;
        assert!(c.pose(&exits).is_none());
    }

    #[test]
    fn backs_out_when_parked_nose_in_and_comes_home_nose_in() {
        let exits = [[0.5, 20.0], [99.5, 20.0]];
        // Facing the house (away from the street at +z).
        let c = car([30.0, 10.0], std::f32::consts::PI, 20.0);
        let r = Route::new(&c, &exits, false);
        // Backing out, it faces the house still.
        assert!(
            (r.points[1].1.abs() - std::f32::consts::PI).abs() < 0.2,
            "{}",
            r.points[1].1
        );
        assert!(r.points.last().unwrap().0[0] < 0.5, "west, the nearer end");
        let home = Route::new(&c, &exits, true);
        let end = *home.points.last().unwrap();
        assert!((end.0[0] - 30.0).abs() < 1e-3 && (end.0[1] - 10.0).abs() < 1e-3);
        assert!(
            (end.1.abs() - c.nose_in()).abs() < 0.1,
            "nose in: {}",
            end.1
        );
    }
}
