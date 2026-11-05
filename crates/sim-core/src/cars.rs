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
//! A trip by car (open-world towns, `Trip`): someone off to a place beyond walking range
//! (`autonomy.reach`; a visit or an outing) with the household's car at home and public parking
//! near the place (the town file's `parking`) walks to the driver's door instead (`trip_car`),
//! gets in and drives off along the roads (`Phase::Driving`, sat at the wheel), parks at the spot
//! (`CarState::Out`) and walks on. Going home they walk back to it and drive home, and get out
//! as after work. The trip is saved with the car.
//!
//! Passengers (`Car::riders`): the household setting off at the same time for somewhere the car
//! parks near (the same parking spot) comes along. The driver gets in and waits in the driveway
//! (`CarState::Boarding` on a trip) while any of them is on the way to it (those already walking
//! off turn round while they're close by: `call_trip_riders`), up to `TRIP_WAIT_TICKS`; each gets
//! in by a door of their own (`trip_ride`, `ride`) and sits in their seat as it drives
//! (`Phase::Driving`, like the driver, drawn there by the renderer as anyone sitting). At the
//! parking spot they all get out and walk on to where they're going. When one of them goes home
//! (the driver or not) the others go home too (`call_party_home`): back to the car, in, and it
//! drives home once they're all in (or waited for long enough), where they get out as after work.
//! Anyone who misses it makes their own way, as do those in it waiting for a driver who can't come
//! (at work, out of town) or hasn't come in `TRIP_WAIT_TICKS`.
//!
//! A parked car takes up the tiles under it, at home or out on a trip in its bay: nobody walks
//! through it and nothing can be put there (`World::blocked`, `reblock`). A car whose parking
//! spot has been built over (a wall, a fence, a thing put there while it was out) stays where it
//! is, unused (`Car::blocked`), and the renderer leaves it out.
//!
//! The route is worked out from the car's spot, the street it faces and the town's exits, the
//! same way each time, so it isn't saved; the snapshot gives the renderer the car's pose along
//! it, and which doors are open, each tick (`snapshot::car`). In a town with a road network
//! (`Travel::graph`: normal and big ones, and cosy ones made since cars drive winding roads,
//! `settle_all`) it goes along its driveway (`Drive::way`), turns onto the road where the
//! driveway meets it and along the lanes (`Route::on_roads`); a car whose driveway comes
//! alongside its road rather than onto it (`OBLIQUE`), or has no way out by road, stays parked
//! (`Car::no_way`). Older towns drive straight along x (`Route::legacy`).
//!
//! **Among the traffic.** A car drives in its lane, `LANE` right of the road's middle where the
//! road has room for two to pass clear of each other (a 7 m street, a country road), `TIGHT_LANE`
//! on one too narrow for that (a 6 m lane) and in older towns (`lane_on`). It keeps behind one in
//! front of it going its way (`up_against`); of two tangled, the later keeps behind the earlier
//! till they part (`Driving::free`). Coming into town it waits out of sight at the way in till the
//! way in is clear of the one before it, the ground in front of it too, the one there longest first
//! (`room_to_come_in`): those coming home at the same time come in one after another, not on top
//! of each other. A trip's car parks only in a spot no other car is parked at or on its way to
//! (`parking_for`).
//!
//! Where its way crosses or meets another's, it takes turns with the other cars (`traffic.rs`):
//! into a junction's box, across a lane out of its driveway or a bay (or into one), and round a 6 m
//! lane's tight bend where it couldn't pass one coming the other way, only holding its claim to it
//! (`traffic::Traffic::may_go`), and never into what's left of another's
//! (`traffic::Traffic::blocked`). On a 6 m lane two cars meeting pull over onto the pavement to
//! pass (`pull_over`, the kerbside furniture `LANE_KERBSIDE` back from the edge for it), and a car
//! waits while one coming the other way is too near to pass (`too_near`), but for one waiting for
//! it. Held up nose to nose with one that waits for it for `JAM_TICKS`, it backs up a little out of
//! its way (`back_off`). And never deeper into another car than it is (`guard`).

use serde::{Deserialize, Serialize};

use crate::TICKS_PER_SECOND;
use crate::lot::OUTDOORS;
use crate::path::join;
use crate::travel::{Mode, RoadPoint, Travel};
use crate::world::{
    Household, Phase, Plot, Sim, SimBrief, Task, TaskKind, World, nearest_exit, wrap_angle,
};

/// Most cars in a town (the snapshot's rows).
pub const MAX_CARS: usize = 128;
/// Doors: 0 front left (the driver's), 1 front right, 2 back left, 3 back right.
pub const DOORS: usize = 4;
/// How long the driver waits in the car for the others setting off at the same time.
const BOARD_WAIT_TICKS: u64 = 4 * crate::TICKS_PER_MINUTE;
/// How long getting in takes before the car can go (the doors, the belt).
const BOARD_MIN_TICKS: u64 = crate::TICKS_PER_MINUTE / 2;
/// On a trip, the longest the driver waits in the car for the others coming along while they are
/// on their way to it (from the house; from the place, up to `TRIP_PARK_REACH` off, to go home).
const TRIP_WAIT_TICKS: u64 = 2 * 60 * crate::TICKS_PER_MINUTE;
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
/// there (`drive_on`); and how close to its sides and ends their bodies come (m, `Body`).
pub(crate) const AHEAD: f32 = 2.2;
pub(crate) const BESIDE: f32 = 0.1;
/// How close someone steps to a car driving by (m, `walks_into`): further off than it
/// keeps from them, so one it held up doesn't hold it up in turn as it moves on past them.
pub(crate) const KEEP: f32 = 2.0 * BESIDE;
/// How far someone on foot reaches out from where they stand (m: their shoulders and elbows).
pub(crate) const BODY: f32 = 0.25;
/// Someone waiting for a car to pass for this long steps aside out of its way (`make_way`).
pub(crate) const MAKE_WAY_TICKS: u32 = 30;
/// How long a car waits for someone who can't step aside (with nowhere to step) before they go
/// round to where it has room to pass them (`clear_of`), up to `CLEAR_STEPS` tiles off.
pub(crate) const PASS_TICKS: u32 = 3 * crate::TICKS_PER_MINUTE as u32;
const CLEAR_STEPS: usize = 6;
/// How long someone done at work waits there for the others in their car.
const RIDE_WAIT_TICKS: u64 = 60 * crate::TICKS_PER_MINUTE;
/// The drive between the end of town and work, each way.
const OFF_MAP_TICKS: u64 = 10 * crate::TICKS_PER_MINUTE;
/// About how long the walk from the house to the car takes (for when to leave).
const WALK_TO_CAR_TICKS: u64 = 3 * crate::TICKS_PER_MINUTE;
/// The middle of a car's lane from the middle of the road (m): they drive on the right, clear of
/// one coming the other way by 0.1 m (the widest car is 1.88 m across) where the road has room
/// for two (`lane_on`: a 7 m street, 4.3 m between its kerbs; a country road). On one too narrow
/// for that (a 6 m lane), and in older towns' streets (`Route::legacy`), `TIGHT_LANE`.
pub(crate) const LANE: f32 = 1.05;
pub(crate) const TIGHT_LANE: f32 = 0.8;
/// Half the widest car's width (m: the renderer's bodies), and how far in from the kerb a car
/// keeps at least.
pub(crate) const CAR_HALF: f32 = 0.94;
const KERB_KEEP: f32 = 0.05;
/// How far in from a lane's edges its kerbside furniture may reach (m; the renderer's street
/// dressing keeps to it, `pull_over_streets`) where the lane is too narrow for two cars to pass
/// in their lanes (a 6 m lane): back from the kerb, for them to pull over onto the pavement to
/// pass (`pull_room`).
pub const LANE_KERBSIDE: f32 = 0.5;
/// How far in from a street's edge its kerbside furniture reaches elsewhere (m: `street.ts` stands
/// it `SIDEWALK - KERB - 0.3` in, its widest about 0.3 across).
pub(crate) const KERBSIDE: f32 = SIDEWALK - 0.14 - 0.3 + 0.3;
/// How far apart two cars pass each other at least (m: `too_near`), and how much further apart
/// two coming towards each other pull over to pass where the lane is too narrow for both in their
/// lanes (m, `pull_want`): its measure is across a way between theirs, `too_near` their bodies as
/// they are, which on a bend come nearer.
pub(crate) const PASSING: f32 = 0.1;
pub(crate) const PULL_MARGIN: f32 = 0.3;
/// How far a car pulls over (or back) a tick (m), and for one coming the other way how far ahead
/// of its front (m, from the other's front: `pull_want`).
pub(crate) const PULL_RATE: f32 = 0.06;
pub(crate) const PULL_AHEAD: f32 = 16.0;
/// How near the furniture (or a country road's edge) a car pulled over comes (m), and how far
/// right of its lane it pulls over at most (m).
const PULL_EDGE: f32 = 0.1;
const PULL_MOST: f32 = 0.7;
/// Turning radius at the driveway's mouth (m).
const TURN: f32 = 2.4;
/// How far past the town's exit a car drives before it's gone (m).
pub(crate) const OVERSHOOT: f32 = 12.0;
/// Top speeds (m per second at 1×): along the street, and in the driveway, turning or backing up.
pub(crate) const FAST: f32 = 6.0;
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
/// On the town's roads (`Route::on_roads`): how far from the road a driveway or exit may be (m),
/// how slow cars take junctions (m/s) within how far of them (m), and the longest straight step
/// between a route's points and how far a step may stray from the lane (m).
pub(crate) const ROAD_REACH: f32 = 30.0;
pub(crate) const JUNCTION_SPEED: f32 = 3.5;
pub(crate) const JUNCTION_REACH: f32 = 8.0;
pub(crate) const ROUTE_STEP: f32 = 4.0;
pub(crate) const ROUTE_TOLERANCE: f32 = 0.08;
/// A driveway meeting its road further from square than this (the cosine of its angle to the
/// road's side) comes alongside it rather than onto it: its car stays parked (`Car::no_way`).
const OBLIQUE: f32 = 0.5;
/// A street's or lane's sidewalks (m, each side: `roads.ts` `SIDEWALK`), inside its width; and how
/// far onto the sidewalk a turn out of (or into) a driveway may begin (`Bay::past_kerb`).
pub(crate) const SIDEWALK: f32 = 1.35;
const KERB_SLACK: f32 = 0.4;
/// A trip by car (`Trip`) parks at the town's public parking spot nearest the place, if one is
/// this close to it (m).
const TRIP_PARK_REACH: f32 = 120.0;

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
    /// Parked away from home on a trip (`Car::trip`): at a public parking spot in another
    /// hamlet while its driver is out there.
    Out,
}

/// A drive to a place beyond walking range and back (`Car::trip`; open-world towns): out of the
/// driveway along the roads to a public parking spot near it (the town file's `parking`), parked
/// there (`CarState::Out`) while the driver walks to the place and is there, then back the same
/// way and home. Whoever of the household sets off with the driver rides along (`Car::riders`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trip {
    /// Who drives (and goes).
    pub driver: u32,
    /// Where they're going (a plot), and whether the player sent them.
    pub plot: u32,
    #[serde(default)]
    pub directed: bool,
    /// Where it parks there, and which way it faces parked.
    pub at: [f32; 2],
    pub yaw: f32,
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
    /// Where its driveway meets the road network, as the town file says (`Drive::road`, `s`);
    /// None: the nearest road (not saved).
    #[serde(skip)]
    meets: Option<(u32, f32)>,
    /// The way its driveway runs to the street (`Drive::way`; not saved): it drives out and in
    /// along it and parks nose in facing the other way. None: square to the road it meets.
    #[serde(skip)]
    axis: Option<[f32; 2]>,
    /// The town's cars drive its roads (`Travel::graph`) but there's no way by them from its
    /// driveway out of town: it stays parked, unused (`settle`; not saved).
    #[serde(skip)]
    pub no_way: bool,
    /// Ticks the drive out of town takes from where it parks (`settle`; not saved).
    #[serde(skip)]
    out_ticks: f32,
    /// How far along its drive it is (ticks of its route: it stops for people in its way).
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub progress: f32,
    /// On a trip to another hamlet (`Trip`); its routes then go to its parking spot and back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trip: Option<Trip>,
    /// On a trip: who rides with its driver (besides them), there and home again with it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub riders: Vec<u32>,
    /// Parked in front of a garage the player built at the end of its driveway (`park_by_garage`):
    /// where it parked before (x, z, yaw), where it goes back to once the garage is gone. Absent:
    /// it parks where it always has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drive_spot: Option<[f32; 3]>,
    /// Ticks it has waited for people in its way (not saved).
    #[serde(skip)]
    waited: u32,
    /// Those on foot or on a bike it waits for this tick (resident ids; for checks:
    /// `traffic::waits_on`). Not saved.
    #[serde(skip)]
    pub(crate) on_foot: Vec<u32>,
    /// Where it is now, which way it faces, whether it's moving and which doors are open (bit
    /// `d` for door `d`); None while it's out of town or unused (`poses`; for the snapshot, not
    /// saved).
    #[serde(skip)]
    pub shown: Option<([f32; 2], f32, bool, u8)>,
    /// Driving, where it would be a tick on and which way it would face (`drive_on`): those on foot
    /// keep clear of it there too (`in_the_way`), as it waits for anyone it would come nearer to
    /// (`go_on`); turning tight, it swings round further than it is now. Not saved.
    #[serde(skip)]
    next: Option<([f32; 2], f32)>,
    /// How far right of its lane it's pulled over (m) to pass one coming the other way on a lane
    /// too narrow for both in their lanes (`pull_over`). Not saved.
    #[serde(skip)]
    pub shift: f32,
    /// Who it waits for this tick (`Driving::first`: one in front of it, one coming the other way,
    /// one whose claim it would drive into). Not saved.
    #[serde(skip)]
    pub waits_for: Option<u64>,
    /// It moved on its way last tick (others asking for a claim look ahead along its way:
    /// `traffic::Mover`). Not saved.
    #[serde(skip)]
    moved: bool,
    /// Whom it pulls over for, last it did (`pull_want`). Not saved.
    #[serde(skip)]
    pulled_for: Option<u64>,
    /// Ticks it and one in its way have waited for each other, and ticks it has backed up since
    /// (`back_off`). Not saved.
    #[serde(skip)]
    jam: (u32, u32),
    /// A work taxi (`taxi.rs`), not a household's: it comes in at a way out on a run, drives it
    /// and goes again, and nothing of the households' (`update`) is its. Not saved with the cars
    /// (the runs are, `taxi::Taxis`).
    #[serde(skip)]
    pub taxi: bool,
    /// A taxi standing at a stop (`taxi.rs`): shown standing, its doors as its riders open them.
    /// Not saved.
    #[serde(skip)]
    pub standing: bool,
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
/// which way it runs from there to the street (`front`: +1 towards +z; `frontX`: +1 towards +x,
/// for a house whose street is east or west of it), and the middle of that street (z).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Drive {
    pub plot: u32,
    pub x: f32,
    pub z: f32,
    pub front: f32,
    /// A driveway running along x (its street east or west of the house; towns made since cars
    /// drive winding roads). Absent: 0, it runs along z (`front`), so files and saves without it
    /// stay byte for byte as they were.
    #[serde(rename = "frontX", default, skip_serializing_if = "is_zero_f32")]
    pub front_x: f32,
    pub street: f32,
    /// The road (`roads` index) the driveway meets and how far along it (m): open-world towns,
    /// and any town made since cars drive winding roads (in a cosy one, the sign that its cars
    /// drive its roads: `settle_all`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub road: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s: Option<f32>,
}

impl Drive {
    fn meets(&self) -> Option<(u32, f32)> {
        Some((self.road?, self.s.filter(|s| s.is_finite())?))
    }

    /// The way it runs from its inner end to the street (a unit vector: `front`, `front_x`).
    fn way(&self) -> Option<[f32; 2]> {
        let (x, z) = (self.front_x, self.front);
        let l = x.hypot(z);
        (l.is_finite() && l > 0.5).then(|| [x / l, z / l])
    }

    /// Where a car `len` long parks on it: its middle `PARK_GAP` on from the inner end.
    fn spot(&self, len: f32) -> [f32; 2] {
        let [x, z] = self.way().unwrap_or([0.0, 0.0]);
        let k = PARK_GAP + len / 2.0;
        [self.x + x * k, self.z + z * k]
    }
}

/// A garage at the inner end of a house's driveway (`LotFile::garages`, as the town placed it:
/// wall centre lines), or one the player built on their lot (`World::build_garage`). It stands
/// where it is; nobody walks through it and nothing is put there (`reblock`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Garage {
    pub plot: u32,
    pub x0: f32,
    pub z0: f32,
    pub x1: f32,
    pub z1: f32,
    /// Which way its door faces, as a thing's facing (0 towards +z, 1 +x, 2 -z, 3 -x): one the
    /// player built (Build mode's Garage tool). Absent: towards the street, as a town places them
    /// (and so files and saves without it stay as they were).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rot: Option<u8>,
}

/// Half a garage's wall (m), outside its centre lines (the renderer's `WALL_HALF`).
pub const GARAGE_WALL_HALF: f32 = 0.07;

impl Garage {
    /// The tiles it stands on.
    pub(crate) fn tiles(&self) -> Vec<(i32, i32)> {
        covered_tiles(self.x0, self.z0, self.x1, self.z1)
    }

    /// The middle of its door outside (on the outer face of its wall) and the way it faces out
    /// (a unit vector), for a garage with a facing (`rot`).
    pub fn door(&self) -> Option<([f32; 2], [f32; 2])> {
        let (cx, cz) = ((self.x0 + self.x1) / 2.0, (self.z0 + self.z1) / 2.0);
        let t = GARAGE_WALL_HALF;
        Some(match self.rot? % 4 {
            0 => ([cx, self.z1 + t], [0.0, 1.0]),
            1 => ([self.x1 + t, cz], [1.0, 0.0]),
            2 => ([cx, self.z0 - t], [0.0, -1.0]),
            _ => ([self.x0 - t, cz], [-1.0, 0.0]),
        })
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

/// The point `along` the length (forward) and `left` across (to the left) of a car standing at
/// `spot` facing `yaw`.
fn local_at(spot: [f32; 2], yaw: f32, along: f32, left: f32) -> [f32; 2] {
    let (s, c) = yaw.sin_cos();
    [
        spot[0] + s * along + c * left,
        spot[1] + c * along - s * left,
    ]
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
            shown: None,
            next: None,
            shift: 0.0,
            waits_for: None,
            moved: false,
            pulled_for: None,
            jam: (0, 0),
            taxi: false,
            standing: false,
        }
    }

    /// A work taxi's car slot (`taxi.rs`), free (out of town) till a run takes it.
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

    /// A taxi on a run (`taxi.rs`): coming into town along `route` (at `progress` along it, a game
    /// just loaded).
    pub(crate) fn set_taxi_route(&mut self, route: Route, progress: f32) {
        self.state = CarState::Arriving;
        self.route = Some((true, route));
        self.progress = progress;
        self.standing = false;
        self.shift = 0.0;
        self.waited = 0;
        self.jam = (0, 0);
        self.waits_for = None;
        self.pulled_for = None;
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
        self.state = CarState::Away;
        self.route = None;
        self.progress = 0.0;
        self.standing = false;
        self.shown = None;
        self.next = None;
        self.shift = 0.0;
        self.waits_for = None;
    }

    /// Standing on its driveway (parked, or waiting for the others to get in).
    pub fn at_home(&self) -> bool {
        matches!(self.state, CarState::Parked | CarState::Boarding)
    }

    /// Whether `j` may drive it: an owner, grown up.
    fn driven_by(&self, briefs: &[SimBrief], j: u32) -> bool {
        self.owners.contains(&j) && briefs.get(j as usize).is_some_and(|b| b.adult)
    }

    /// The point `along` its length (forward) and `left` across it (to its left) from its middle,
    /// parked.
    fn local(&self, along: f32, left: f32) -> [f32; 2] {
        local_at(self.spot, self.yaw, along, left)
    }

    /// The car as it stands parked at `spot` facing `yaw` (its doors and seats there): a copy
    /// without its routes or who is in it.
    fn posed(&self, spot: [f32; 2], yaw: f32) -> Car {
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
            shown: self.shown,
            next: None,
            shift: 0.0,
            waits_for: None,
            moved: false,
            pulled_for: None,
            jam: (0, 0),
            taxi: false,
            standing: false,
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
    fn door_at(&self, d: usize) -> [f32; 2] {
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

    /// Where it is and which way it faces now, and whether it's moving; None while it's out
    /// of town.
    pub fn pose(&mut self, exits: &[[f32; 2]]) -> Option<([f32; 2], f32, bool)> {
        self.pose_on(exits, None)
    }

    /// `pose`, on the town's roads (`travel.rs`) where it has them.
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

    /// Coming into town from the way in (not from a trip's parking spot), it hasn't yet: out of
    /// sight there, waiting for room to come in (`drive_on`).
    fn outside(&self) -> bool {
        self.state == CarState::Arriving && self.trip.is_none() && self.progress <= 0.0
    }

    /// Its route out of town or (`home`) back (worked out when first asked for).
    fn route(&mut self, exits: &[[f32; 2]], travel: Option<&Travel>, home: bool) -> &Route {
        if self.route.as_ref().is_none_or(|(h, _)| *h != home) {
            self.route = Some((home, Route::build(self, exits, travel, home)));
        }
        &self.route.as_ref().expect("just made").1
    }

    /// Works out again what follows from where it parks (after it came home, moved, or loaded).
    fn settle(&mut self, exits: &[[f32; 2]], travel: Option<&Travel>) {
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
    fn mouth(&self, travel: &Travel) -> Option<RoadPoint> {
        let g = &travel.graph;
        self.meets
            .and_then(|(road, s)| g.at_source(road, s))
            .filter(|p| g.roads[p.road as usize].kind != crate::travel::RoadKind::Trail)
            .or_else(|| g.near(self.spot, ROAD_REACH, 1, Mode::Car).first().copied())
    }

    /// Its route's claims worked out again (the junctions' lights settled: `traffic::settle`).
    pub(crate) fn reclaim(&mut self, travel: &Travel) {
        let (len, wid) = (self.len, self.wid);
        if let Some((_, route)) = self.route.as_mut() {
            route.claim(len, wid, Some(travel));
        }
    }

    /// The yaw it parks with after coming home, nose in: facing up its driveway, away from the
    /// street (`axis`), else away from the road it meets (the street, in towns with no roads).
    fn nose_in_on(&self, travel: Option<&Travel>) -> f32 {
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
    fn nose_in(&self) -> f32 {
        if self.street > self.spot[1] {
            std::f32::consts::PI
        } else {
            0.0
        }
    }

    /// The tiles it takes up parked.
    fn tiles(&self) -> Vec<(i32, i32)> {
        self.tiles_at(self.spot, self.yaw)
    }

    /// The tiles it takes up parked at `at` facing `yaw`.
    fn tiles_at(&self, at: [f32; 2], yaw: f32) -> Vec<(i32, i32)> {
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
    fn out_at(&self) -> Option<([f32; 2], f32)> {
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

/// The top speed on the roads at `p`, on a road of `kind` (m/s): `FAST` in the hamlets,
/// `Travel::country_speed` on country roads, `JUNCTION_SPEED` near any of `junctions`.
pub(crate) fn top_speed(
    p: [f32; 2],
    kind: crate::travel::RoadKind,
    travel: &Travel,
    junctions: &[[f32; 2]],
) -> f32 {
    let fast = if kind == crate::travel::RoadKind::Country {
        travel.country_speed
    } else {
        FAST
    };
    if junctions
        .iter()
        .any(|&j| crate::travel::dist(j, p) < JUNCTION_REACH)
    {
        fast.min(JUNCTION_SPEED)
    } else {
        fast
    }
}

/// A drive, sampled every `STEP` metres: where the car's middle is, which way it faces, and when
/// it gets there (ticks from the start).
#[derive(Debug, Clone, Default)]
pub(crate) struct Route {
    pub(crate) points: Vec<([f32; 2], f32, f32, bool)>,
    /// At each point, how far right of its lane a car can pull over there to pass one coming the
    /// other way (m, `pull_room`; none off the lanes, turning or backing).
    room: Vec<f32>,
    /// At each point, how far right of the road's middle its lane is (m, `lane_on`; off the lanes,
    /// `LANE`).
    lane: Vec<f32>,
    /// How far it can pull over to pass one coming the other way (m, `room`), ticks along it, as
    /// its claims have it.
    pass: Vec<(f32, f32)>,
    /// Where it turns out of a driveway, a parking bay or a spot on the road onto the lane, or off
    /// it into one (ticks along it, from and to): claims of their own (`traffic::claims`).
    mouths: Vec<(f32, f32)>,
    /// Where it must hold a claim to drive on (`traffic::Claim`): junctions' boxes and the mouths,
    /// worked out with the route (`Route::build`).
    pub(crate) claims: Vec<crate::traffic::Claim>,
}

impl Route {
    /// The route on the town's roads (`on_roads`) where it has a network the car's driveway
    /// meets, else `legacy` (to a road out of town).
    fn build(car: &Car, exits: &[[f32; 2]], travel: Option<&Travel>, home: bool) -> Route {
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
    fn claim(&mut self, len: f32, wid: f32, travel: Option<&Travel>) {
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
        self.claims = crate::traffic::claims(&trace, &pass, len, wid, boxes, &more);
        if let Some(t) = travel {
            crate::traffic::gate(&mut self.claims, &t.signals);
        }
        self.pass = trace.iter().zip(&pass).map(|(s, &p)| (s.0, p)).collect();
    }

    /// How far it can pull over to pass one coming the other way `t` ticks along (m: `pass`).
    pub(crate) fn pass_at(&self, t: f32) -> f32 {
        let k = self.pass.partition_point(|p| p.0 <= t);
        self.pass.get(k.saturating_sub(1)).map_or(0.0, |p| p.1)
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
    fn off_roads(car: &Car, exits: &[[f32; 2]], home: bool) -> Route {
        Self::legacy(car, exits, home)
    }

    #[cfg(test)]
    fn new(car: &Car, exits: &[[f32; 2]], home: bool) -> Route {
        Self::legacy(car, exits, home)
    }

    /// Out of the driveway onto the road it meets and along the roads to the way out of town
    /// quickest by car, past it (`OVERSHOOT`), or (`home`) back in from there, nose first. The
    /// driveway's part as `legacy` lays it out, in the frame of the road at the driveway's
    /// mouth; the lanes right of the roads' middle lines (`lane_on`); `FAST` in the hamlets,
    /// `Travel::country_speed` on country roads, slower at junctions. Sampled more where it
    /// bends (`ROUTE_TOLERANCE`). None: no road near, or no way out by road.
    fn on_roads(car: &Car, exits: &[[f32; 2]], travel: &Travel, home: bool) -> Option<Route> {
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
        let lane = crate::travel::thin_with(
            &offset_right_by(&centre, &offs),
            ROUTE_STEP,
            ROUTE_TOLERANCE,
        );
        if lane.len() < 2 {
            return None;
        }
        let junctions: Vec<[f32; 2]> = g.junctions().collect();
        // (The road at `p`: its width and kind, those of the path's point nearest it.)
        let road_at = |p: [f32; 2]| {
            path.pts
                .iter()
                .min_by(|x, y| dist(x.0, p).total_cmp(&dist(y.0, p)))
                .map_or((7.0, RoadKind::Street), |x| (x.1, x.2))
        };
        let top = |p: [f32; 2]| top_speed(p, road_at(p).1, travel, &junctions);
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
                top(p),
                pull_room(width, kind, car.wid),
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

/// Where a car parks off the road, its driveway or a parking bay, in the frame of the road where
/// it meets it (`mp` on its middle line, its tangent `t` and normal `n`): where it parks (`x0`,
/// `z0` in that frame) facing `yaw`, the way it runs to the road (`u`: along its axis, else
/// square to the road), and the kerb on its side of the road (`kerb`, its `z` in that frame).
struct Bay {
    spot: [f32; 2],
    yaw: f32,
    mp: [f32; 2],
    t: [f32; 2],
    n: [f32; 2],
    x0: f32,
    z0: f32,
    u: [f32; 2],
    kerb: f32,
}

impl Bay {
    /// On `road` (its kerbs: `kerb_off`). None: it comes alongside the road rather than onto it
    /// (`OBLIQUE`).
    fn new(
        spot: [f32; 2],
        yaw: f32,
        axis: Option<[f32; 2]>,
        mp: [f32; 2],
        road: &crate::travel::Road,
        s_at: f32,
    ) -> Option<Bay> {
        let t = road.tangent(s_at);
        let n = [-t[1], t[0]];
        let (dx, dz) = (spot[0] - mp[0], spot[1] - mp[1]);
        let (x0, z0) = (dx * t[0] + dz * t[1], dx * n[0] + dz * n[1]);
        let s = if z0 <= 0.0 { 1.0 } else { -1.0 };
        let u = match axis {
            Some([x, z]) => {
                let u = [x * t[0] + z * t[1], x * n[0] + z * n[1]];
                if u[1] * s < OBLIQUE {
                    return None;
                }
                u
            }
            None => [0.0, s],
        };
        Some(Bay {
            spot,
            yaw,
            mp,
            t,
            n,
            x0,
            z0,
            u,
            kerb: -s * kerb_off(road),
        })
    }

    /// How far along its line from where it crosses its side's kerb it crosses the lane `lane` m
    /// off the middle line, and a little (`KERB_SLACK`): the most a turn onto (or off) that lane
    /// takes up of it, so it turns on the road, not across the sidewalk.
    fn past_kerb(&self, lane: f32) -> f32 {
        ((lane - self.kerb) / self.u[1]).max(0.0) + KERB_SLACK
    }

    fn local(&self, p: [f32; 2]) -> [f32; 2] {
        let (dx, dz) = (p[0] - self.mp[0], p[1] - self.mp[1]);
        [
            dx * self.t[0] + dz * self.t[1],
            dx * self.n[0] + dz * self.n[1],
        ]
    }

    fn world(&self, q: [f32; 2]) -> [f32; 2] {
        [
            self.mp[0] + q[0] * self.t[0] + q[1] * self.n[0],
            self.mp[1] + q[0] * self.t[1] + q[1] * self.n[1],
        ]
    }

    /// Where its line crosses the lane `lane` m off the middle line (along the road), and how far
    /// that is from where it parks.
    fn cross(&self, lane: f32) -> (f32, f32) {
        let k = ((lane - self.z0) / self.u[1]).max(0.0);
        (self.x0 + self.u[0] * k, k)
    }

    /// Whether `p` is a point of the lane before (or after) a turn round `at` (along the road)
    /// that takes up `r` of it, going `d` (+1: along `t`).
    fn near(&self, p: [f32; 2], at: f32, r: f32, d: f32) -> bool {
        (self.local(p)[0] - at) * d < r + 0.5
            && crate::travel::dist(p, self.world([at, 0.0])) < 2.0 * r + 4.0
    }

    /// Out of it onto the lane going `d` (+1: along `t`): facing the road it pulls out, facing
    /// away it backs out turning away from the way it goes (as `Route::legacy` does). Where the
    /// turn is along the road, and how much of the lane it takes up.
    fn leave(&self, legs: &mut Legs, d: f32) -> (f32, f32) {
        let lane_out = d * TIGHT_LANE;
        let (at, room) = self.cross(lane_out);
        let room = room.min(self.past_kerb(lane_out));
        let facing = [self.yaw.sin(), self.yaw.cos()];
        let way = [
            self.u[0] * self.t[0] + self.u[1] * self.n[0],
            self.u[0] * self.t[1] + self.u[1] * self.n[1],
        ];
        let facing_street = facing[0] * way[0] + facing[1] * way[1] > 0.0;
        let k = if facing_street { d } else { -d };
        let reverse = !facing_street;
        let turn = Turn::round([at, lane_out], self.u, [k, 0.0], room);
        legs.line(self.spot, self.world(turn.from), reverse, SLOW);
        turn.drive(legs, &|q| self.world(q), reverse);
        (at, turn.taken)
    }

    /// The turn into it from the lane, coming along it against `d`, and where it is along the road.
    fn entry(&self, d: f32) -> (Turn, f32) {
        let lane_in = -d * TIGHT_LANE;
        let (at, room) = self.cross(lane_in);
        let room = room.min(self.past_kerb(lane_in));
        (
            Turn::round([at, lane_in], [-d, 0.0], [-self.u[0], -self.u[1]], room),
            at,
        )
    }

    /// Into it, nose first, by `turn` (`entry`).
    fn enter(&self, legs: &mut Legs, turn: &Turn) {
        let from = legs.last().unwrap_or(self.world(turn.from));
        legs.line(from, self.world(turn.from), false, JUNCTION_SPEED);
        turn.drive(legs, &|q| self.world(q), false);
        legs.line(self.world(turn.to), self.spot, false, SLOW);
    }
}

/// A public parking spot at `at` as a bay off the road (one of a big world's, `macro.ts`: square
/// to the road, off its verge): the point of the road it's nearest, the road's tangent there and
/// the way from the spot to it. None: no road near, or the spot is on the road itself (pulled into
/// along it).
fn parking_bay(travel: &Travel, at: [f32; 2]) -> Option<([f32; 2], RoadPoint, [f32; 2])> {
    let g = &travel.graph;
    let p = *g.near(at, ROAD_REACH, 1, Mode::Car).first()?;
    let road = &g.roads[p.road as usize];
    let rp = g.point(p);
    let (dx, dz) = (rp[0] - at[0], rp[1] - at[1]);
    let l = dx.hypot(dz);
    (l > road.width / 2.0 + 0.5).then_some((rp, p, [dx / l, dz / l]))
}

/// How far from its middle line a road's kerbs are (m): its width takes in the sidewalks of a
/// street or lane (`roads.ts` `SIDEWALK`); a country road has none.
fn kerb_off(road: &crate::travel::Road) -> f32 {
    kerb_of(road.width, road.kind)
}

/// `kerb_off` for a road `width` m wide of `kind`.
pub(crate) fn kerb_of(width: f32, kind: crate::travel::RoadKind) -> f32 {
    match kind {
        crate::travel::RoadKind::Country | crate::travel::RoadKind::Trail => width / 2.0,
        _ => (width / 2.0 - SIDEWALK).max(TIGHT_LANE + 0.5),
    }
}

/// Where a car drives on a road `width` m wide of `kind` (m right of its middle line): `LANE` where
/// that leaves it `KERB_KEEP` from the kerb, else `TIGHT_LANE` (a 6 m lane, 3.3 m between its
/// kerbs: too narrow for two cars to pass either way, and not up on the kerb, as the kerbside
/// furniture stands about 0.15 m in from it). It turns out of a driveway onto the road, and off it
/// into one, `TIGHT_LANE` out as ever (`Bay`: a turn there at a bend in the road cuts the bend's
/// edge the more the further out it ends), easing out to its lane on along the road.
pub(crate) fn lane_on(width: f32, kind: crate::travel::RoadKind) -> f32 {
    if kerb_of(width, kind) - CAR_HALF - KERB_KEEP >= LANE {
        LANE
    } else {
        TIGHT_LANE
    }
}

/// How far in from the edges of a road `width` m wide of `kind` its kerbside furniture may reach
/// (m): `LANE_KERBSIDE` on a lane too narrow for two cars to pass in their lanes (`lane_on`), where
/// they pull over onto the pavement to pass; elsewhere where it always stood (`KERBSIDE`).
pub fn kerbside(width: f32, kind: crate::travel::RoadKind) -> f32 {
    if lane_on(width, kind) == TIGHT_LANE {
        LANE_KERBSIDE
    } else {
        KERBSIDE
    }
}

/// The streets whose kerbside furniture keeps back from the kerb (`kerbside`): every lane too
/// narrow for two cars to pass in their lanes, where they pull over onto the pavement to pass. The
/// town file's road (`meta.roads` index) and how far in from its edges the furniture may reach (m),
/// by road: for the renderer's street dressing (the world's structure, `pullOverStreets`).
pub fn pull_over_streets(travel: &Travel) -> Vec<(u32, f32)> {
    use crate::travel::RoadKind;
    let mut out: Vec<(u32, f32)> = travel
        .graph
        .roads
        .iter()
        .filter(|r| !matches!(r.kind, RoadKind::Country | RoadKind::Trail))
        .filter_map(|r| {
            let keep = kerbside(r.width, r.kind);
            (keep < KERBSIDE).then_some((r.source, keep))
        })
        .collect();
    out.sort_by_key(|&(r, _)| r);
    out.dedup_by_key(|&mut (r, _)| r);
    out
}

/// How far right of its lane (`lane_on`) a car `wid` m wide can pull over on a road `width` m wide
/// of `kind` (m, `PULL_MOST` at most): to `PULL_EDGE` short of the kerbside furniture
/// (`kerbside`), or of a country road's edge. On a 6 m lane two cars pulled over pass each other.
pub(crate) fn pull_room(width: f32, kind: crate::travel::RoadKind, wid: f32) -> f32 {
    use crate::travel::RoadKind;
    let edge = match kind {
        RoadKind::Country | RoadKind::Trail => width / 2.0,
        _ => width / 2.0 - kerbside(width, kind),
    } - PULL_EDGE;
    (edge - wid / 2.0 - lane_on(width, kind)).clamp(0.0, PULL_MOST)
}

/// `p`, facing `yaw`, `shift` m to its right.
pub(crate) fn shifted(p: [f32; 2], yaw: f32, shift: f32) -> [f32; 2] {
    // (Forward is (sin, cos); its right (-cos, sin).)
    [p[0] - yaw.cos() * shift, p[1] + yaw.sin() * shift]
}

/// `pts` moved `offs[i]` m to the right (looking the way they run), each its own.
fn offset_right_by(pts: &[[f32; 2]], offs: &[f32]) -> Vec<[f32; 2]> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let (a, b) = (pts[i.saturating_sub(1)], pts[(i + 1).min(n - 1)]);
            let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
            let d = dx.hypot(dz).max(1e-6);
            [pts[i][0] - dz / d * offs[i], pts[i][1] + dx / d * offs[i]]
        })
        .collect()
}

/// A turn at a driveway's mouth, in the frame of the road there (`Route::on_roads`): from going
/// `din` to going `dout` round the corner where their lines cross, an arc as tight as `TURN` at
/// most that begins no further back from the corner than `room` (the driveway's length to it).
/// A driveway square to the road turns a quarter, as `Route::legacy` does.
struct Turn {
    /// Where it begins and ends, the arc's middle and radius (0: straight on).
    from: [f32; 2],
    to: [f32; 2],
    centre: [f32; 2],
    r: f32,
    /// How far from the corner it begins and ends (m).
    taken: f32,
}

impl Turn {
    fn round(corner: [f32; 2], din: [f32; 2], dout: [f32; 2], room: f32) -> Turn {
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
    fn drive(&self, legs: &mut Legs, world: &impl Fn([f32; 2]) -> [f32; 2], reverse: bool) {
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
    pts: Vec<([f32; 2], bool, f32)>,
    /// At each point, how far a car can pull over there (`Route::room`), and its lane
    /// (`Route::lane`).
    room: Vec<f32>,
    lane: Vec<f32>,
    /// Its mouths (`Route::mouths`), from point to point (None: to the end).
    mouths: Vec<(usize, Option<usize>)>,
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
    fn push_lane(&mut self, p: [f32; 2], top: f32, room: f32, lane: f32) {
        let n = self.pts.len();
        self.push(p, false, top);
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
                yaw = wrap_angle(yaw + std::f32::consts::PI);
            }
            points.push((pts[i].0, yaw, t, reverse));
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
        }
    }
}

/// The car `sim` would take to work (or school) now, and their seat in it (0: they drive): an
/// owner 18 or older takes their household's car from home when it's parked there; anyone else in
/// the household rides along when an owner is about to drive off in it (or on their way to).
pub(crate) fn seat(
    cars: &[Car],
    briefs: &[SimBrief],
    households: &[Household],
    plots: &[Plot],
    rels: &crate::social::Relationships,
    sim: &Sim,
) -> Option<(u32, usize)> {
    let me = briefs.get(sim.id as usize)?;
    let home = households.get(sim.household as usize)?.plot?;
    if !sim.here() || me.baby {
        return None;
    }
    if me.plot.is_none_or(|p| p == home)
        && let Some(seat) = own_seat(cars, briefs, sim, me, home)
    {
        return Some(seat);
    }
    // A lift from a neighbour leaving now (`lift_seat`): from home, or on their way to it.
    me.plot?;
    let door = plots.get(home as usize)?.arrival_point();
    lift_seat(cars, briefs, households, rels, door, sim, me, home)
}

/// Who drives car `c` off now, other than `me`: the driver in it waiting for the others, or (it
/// still waits for one) an owner who drives it on their way to it.
fn setting_off_driver(car: &Car, c: u32, briefs: &[SimBrief], me: Option<u32>) -> Option<u32> {
    match (car.state, car.driver) {
        (CarState::Boarding, Some(d)) => Some(d).filter(|&d| Some(d) != me),
        (CarState::Parked, _) | (CarState::Boarding, None) => briefs
            .iter()
            .enumerate()
            .map(|(j, b)| (j as u32, b))
            .find(|&(j, b)| Some(j) != me && b.bound_for_car == Some(c) && car.driven_by(briefs, j))
            .map(|(j, _)| j),
        _ => None,
    }
}

/// How many ride in car `c` with `driver`, other than `me`: those in it and on their way to it.
fn riders(car: &Car, c: u32, briefs: &[SimBrief], driver: u32, me: Option<u32>) -> usize {
    car.aboard.iter().filter(|&&a| a != driver).count()
        + briefs
            .iter()
            .enumerate()
            .map(|(j, b)| (j as u32, b))
            .filter(|&(j, b)| Some(j) != me && j != driver && b.bound_for_car == Some(c))
            .count()
}

/// A seat in a car of `sim`'s own household (`seat`).
fn own_seat(
    cars: &[Car],
    briefs: &[SimBrief],
    sim: &Sim,
    me: &SimBrief,
    home: u32,
) -> Option<(u32, usize)> {
    cars.iter().enumerate().find_map(|(c, car)| {
        // At home, or still close by on the street.
        let near = me.plot == Some(home)
            || (sim.pos[0] - car.spot[0]).hypot(sim.pos[1] - car.spot[1]) < CALL_REACH;
        // (Not one setting off on a trip: `trip_ride`.)
        if car.household != sim.household
            || car.plot != home
            || car.blocked
            || car.no_way
            || !near
            || car.trip.is_some()
        {
            return None;
        }
        let c = c as u32;
        let waiting = matches!(
            (car.state, car.driver),
            (CarState::Parked, _) | (CarState::Boarding, None)
        );
        match setting_off_driver(car, c, briefs, Some(sim.id)) {
            // Nobody else to drive it: they do.
            None => (waiting && car.driven_by(briefs, sim.id)).then_some((c, 0)),
            // A seat of their own, past the driver and the riders before them.
            Some(d) => {
                let rider = riders(car, c, briefs, d, Some(sim.id)) + 1;
                (rider < SEATS).then_some((c, rider))
            }
        }
    })
}

/// Neighbours give lifts: a car of another household whose driver is setting off now (on their
/// way to it, or in it and waiting) and gives `sim` a lift (`lift_ok`; `door`: their home's), with
/// a seat free.
#[allow(clippy::too_many_arguments)]
fn lift_seat(
    cars: &[Car],
    briefs: &[SimBrief],
    households: &[Household],
    rels: &crate::social::Relationships,
    door: [f32; 2],
    sim: &Sim,
    me: &SimBrief,
    home: u32,
) -> Option<(u32, usize)> {
    cars.iter().enumerate().find_map(|(c, car)| {
        // (The car's own tests first: who drives it is a look through everyone.)
        if me.plot.is_some_and(|p| p != home && p != car.plot)
            || car.trip.is_some()
            || !lift_car(car, households, door, sim.household)
        {
            return None;
        }
        let c = c as u32;
        let driver = setting_off_driver(car, c, briefs, Some(sim.id))?;
        if !lift_friend(rels, sim.id, driver) {
            return None;
        }
        let rider = riders(car, c, briefs, driver, Some(sim.id)) + 1;
        (rider < SEATS).then_some((c, rider))
    })
}

/// Whether `driver` gives `i` (of `household`, whose home's door is `door`) a lift in `car`:
/// another household's, standing on its own driveway (usable) within `LIFT_REACH` m of the door,
/// and the driver a friend (`LIFT_FRIENDSHIP`).
fn lift_ok(
    car: &Car,
    households: &[Household],
    rels: &crate::social::Relationships,
    door: [f32; 2],
    i: u32,
    household: u32,
    driver: u32,
) -> bool {
    lift_car(car, households, door, household) && lift_friend(rels, i, driver)
}

/// `lift_ok`'s test of the car: another household's than `household`, standing on its own
/// driveway (usable), within `LIFT_REACH` m of `door`.
fn lift_car(car: &Car, households: &[Household], door: [f32; 2], household: u32) -> bool {
    car.household != household
        && households.get(car.household as usize).and_then(|h| h.plot) == Some(car.plot)
        && !car.blocked
        && !car.no_way
        && (door[0] - car.spot[0]).hypot(door[1] - car.spot[1]) <= LIFT_REACH
}

/// `lift_ok`'s test of the driver: someone else, and a friend of `i`'s (`LIFT_FRIENDSHIP`).
fn lift_friend(rels: &crate::social::Relationships, i: u32, driver: u32) -> bool {
    driver != i && rels.get(i as usize, driver as usize).friendship >= LIFT_FRIENDSHIP
}

/// Neighbours give a lift (`lift_ok`) from a car this close (m) to the door of someone's home...
pub(crate) const LIFT_REACH: f32 = 25.0;
/// ...to a friend this good.
pub(crate) const LIFT_FRIENDSHIP: f32 = 30.0;

/// Where `sim` heads for work (or school) when they go by car: the car, the door they get in by
/// and where they stand for it. None: on foot.
pub(crate) fn ride_for(
    ctx: &crate::world::Ctx,
    sim: &Sim,
) -> Option<(u32, usize, usize, [f32; 2])> {
    let (c, seat) = seat(
        ctx.cars,
        ctx.briefs,
        ctx.households,
        ctx.plots,
        ctx.rels,
        sim,
    )?;
    let (door, at) = ctx.cars[c as usize].door(&ctx.nav, seat)?;
    Some((c, seat, door, at))
}

/// Whether `i` can get into car `c` now (as `seat` found it on their way).
pub(crate) fn may_board(w: &World, i: usize, c: u32) -> bool {
    seat(
        &w.cars,
        &w.briefs,
        &w.households,
        &w.plots,
        &w.relationships,
        &w.sims[i],
    )
    .is_some_and(|(k, _)| k == c)
}

/// How much earlier than on foot `i` leaves for work or school starting at `start` when their
/// household's car is at home to take them: the walk to it, getting in, the drive out of town and
/// on to work. 0 without one, and for one who can't drive it unless someone who can starts about
/// when they do (so they set off together: `seat`).
pub(crate) fn lead_ticks(w: &World, i: usize, start: u64) -> u64 {
    let s = &w.sims[i];
    let me = i as u32;
    let home = w.households.get(s.household as usize).and_then(|h| h.plot);
    let usable = |c: &&Car| c.at_home() && !c.blocked && !c.no_way;
    // An owner who drives it and starts about when they do.
    let sets_off = |car: &Car, j: u32| {
        j != me
            && car.driven_by(&w.briefs, j)
            && w.sims.get(j as usize).is_some_and(|o| o.here())
            && starts_today(w, j as usize).is_some_and(|st| st.abs_diff(start) <= BOARD_WAIT_TICKS)
    };
    let own = w
        .cars
        .iter()
        .filter(usable)
        .find(|c| c.household == s.household && Some(c.plot) == home)
        .filter(|car| car.driven_by(&w.briefs, me) || car.owners.iter().any(|&j| sets_off(car, j)));
    // Else a lift from a neighbour leaving with them (`lift_seat`).
    let door = home.map(|p| w.plots[p as usize].arrival_point());
    let car = own.or_else(|| {
        let door = door?;
        let rels = &w.relationships;
        w.cars.iter().filter(|c| c.at_home()).find(|c| {
            c.owners.iter().any(|&j| {
                lift_ok(c, &w.households, rels, door, me, s.household, j) && sets_off(c, j)
            })
        })
    });
    let Some(car) = car else {
        return 0;
    };
    WALK_TO_CAR_TICKS + u64::from(GET_IN_TICKS) + drive_out_ticks(car) + off_map_ticks(w)
}

/// Once in `car` for work or school: the wait for it to go and the drive out of town.
pub(crate) fn drive_out_ticks(car: &Car) -> u64 {
    BOARD_MIN_TICKS + car.out_ticks.ceil() as u64
}

/// The drive between the end of town and work: in an open-world town `commuteMinutes` (the
/// part of the way off the map), else `OFF_MAP_TICKS` as ever.
pub(crate) fn off_map_ticks(w: &World) -> u64 {
    if w.travel.open {
        (w.content.career_rules.commute_minutes / crate::MINUTES_PER_TICK) as u64
    } else {
        OFF_MAP_TICKS
    }
}

/// When `j`'s work (or school) starts today, if they have it.
fn starts_today(w: &World, j: usize) -> Option<u64> {
    crate::life::start_on(&w.content, &w.sims[j], w.tick)
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
    if let Some(k) = nearest_exit(&w.exits, w.sims[i].pos) {
        w.sims[i].pos = w.exits[k];
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
    // Someone who set off for a car and turned to using something instead isn't coming: the car
    // doesn't wait for them, nor is it kept from the others.
    for s in &mut w.sims {
        if s.bound_for_car.is_some()
            && s.current()
                .is_some_and(|a| matches!(a.phase, Phase::Using { .. }))
        {
            s.bound_for_car = None;
        }
    }
    for c in 0..w.cars.len() {
        // (A work taxi is the taxis' own: `taxi.rs`.)
        if w.cars[c].taxi {
            continue;
        }
        owners(w, c);
        relocate(w, c);
        let gone = |w: &World, s: u32| w.sims.get(s as usize).is_none_or(|s| !s.here());
        // Nobody to drive it home (the driver died or moved away while out): it's back.
        if !w.cars[c].at_home() && w.cars[c].driver.is_none_or(|d| gone(w, d)) {
            if w.cars[c].trip.take().is_some() {
                // (Those riding with them get out and make their own way.)
                let aboard = w.cars[c].aboard.clone();
                for s in aboard {
                    let_out(w, s as usize);
                }
                let World {
                    cars,
                    exits,
                    travel,
                    ..
                } = &mut *w;
                cars[c].route = None;
                cars[c].riders.clear();
                cars[c].settle(exits, Some(travel));
            }
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
        if w.cars[c].trip.is_some() {
            trip_update(w, c);
            continue;
        }
        let state = w.cars[c].state;
        let since = w.cars[c].since;
        match state {
            // (Only on a trip, above.)
            CarState::Out => {}
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
                    // (A garage built or taken down while it was leaving: it comes home to it.)
                    let plot = w.cars[c].plot;
                    if w.cars[c].drive_spot.is_some()
                        || w.garages.iter().any(|g| g.plot == plot && g.rot.is_some())
                    {
                        park_by_garage(w, c);
                    }
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
    let World {
        cars,
        exits,
        travel,
        ..
    } = w;
    cars[c].route(exits, Some(travel), home).ticks()
}

/// A tick further along its drive, unless someone is in its way: it waits for them (`go_on`).
/// Someone at something (a bench, a letterbox) is beside its way at most, nothing stands on a
/// street or a driveway: it doesn't wait for them. Coming into town, it waits out of sight at the
/// way in till the way in is clear of the one before it (`room_to_come_in`), the one there longest
/// first: cars coming home at the same time come in one after another. Into a junction's box or
/// across a lane out of (or into) a driveway or a bay only holding the claim to it
/// (`traffic::Traffic::may_go`), nor into another's (`Traffic::blocked`). Pulled over where the
/// lane is narrow, it passes one coming the other way (`pull_over`; it waits while they're too
/// near, `too_near`). And never deeper into another car than it is (`guard`).
fn drive_on(w: &mut World, c: usize) {
    let home = w.cars[c].state == CarState::Arriving;
    let t = w.cars[c].progress;
    let moving = std::mem::take(&mut w.cars[c].moved);
    // Held up for good, nose to nose with one waiting for it (`JAM_TICKS`), it backs up a little
    // out of its way (`back_off`).
    let key = car_first(c);
    // (Short of the group it asked for, backing up out of one's way or coming back up to it: its
    // ask stands, its place in the queue kept.)
    w.traffic.still_asking(w.tick, key, u32::from(home), t);
    let jammed = w.cars[c].waits_for.is_some_and(|k| waits_back(w, k, key));
    // (Not on its way since, it's still held up: two pulling over for each other by turns wait
    // for each other every other tick.)
    let jam = &mut w.cars[c].jam;
    *jam = if jammed {
        (jam.0.saturating_add(1), jam.1)
    } else if moving {
        (0, 0)
    } else {
        *jam
    };
    let jam = *jam;
    if jammed && jam.0 > JAM_TICKS && jam.1 < JAM_BACK && !w.cars[c].outside() && back_off(w, c) {
        return;
    }
    // (Backed up out of its way, it stays there while that one still waits for it: not on into
    // its way again.)
    if jammed && jam.1 > 0 && !w.cars[c].outside() {
        return;
    }
    w.cars[c].waits_for = None;
    w.cars[c].on_foot.clear();
    let others = vehicles(&w.cars, Some(c));
    if w.cars[c].outside() && !room_to_come_in(w, c, &others) {
        return;
    }
    let tick = w.tick;
    if !w.cars[c].outside() {
        pull_over(w, c, &others, moving);
    }
    {
        let World {
            cars,
            exits,
            travel,
            traffic,
            ..
        } = &mut *w;
        let route = cars[c].route(exits, Some(travel), home);
        if !traffic.may_go(tick, key, u32::from(home), &route.claims, t, t + 1.0) {
            // (Waiting for its claim: for one in its way that waits for it in turn, on the road
            // or for a claim of its own, as one in its way on the road, so that the later of them
            // backs up out of the other's way, `back_off`.)
            let by = traffic.waits_for(key).to_vec();
            cars[c].waits_for = by
                .iter()
                .copied()
                .find(|&k| {
                    cars.get((k & !(1 << 63)) as usize)
                        .is_some_and(|o| o.waits_for == Some(key))
                        || traffic.waits_for(k).contains(&key)
                })
                .or(cars[c].waits_for);
            return;
        }
    }
    // (Pulled over further than it has room on: back towards its lane first.)
    if w.cars[c].shift > route_room(&w.cars[c], t + 1.0) + 1e-3 {
        // (Pulled over for one coming the other way, it waits for that one to pass; for one
        // waiting for it, it pulls back in first.)
        if w.cars[c].waits_for.is_none()
            && w.cars[c]
                .pulled_for
                .is_none_or(|k| !w.traffic.waits_for(k).contains(&key))
        {
            w.cars[c].waits_for = w.cars[c].pulled_for;
        }
        return;
    }
    let standing: Vec<Driving> = parked(&w.cars).collect();
    let World {
        cars,
        exits,
        travel,
        sims,
        lot,
        blocked,
        stairs,
        content,
        objects,
        traffic,
        ..
    } = w;
    let car = &mut cars[c];
    let shift = car.shift;
    // Where it would be a tick on, its body and the ground in front of it (behind it while it
    // backs up): not round its tail, as people may follow it close (on their way the same way),
    // but all along its sides, as its tail swings out on a turn.
    let route = car.route(exits, Some(travel), home);
    let ((at, yaw, reverse), (was, was_yaw, _)) = (route.at(t + 1.0), route.at(t));
    let (at, was) = (shifted(at, yaw, shift), shifted(was, was_yaw, shift));
    let ticks = route.ticks();
    let first_claim = route.claims.first().map(|c| c.from);
    let (ahead, behind) = if reverse { (0.0, AHEAD) } else { (AHEAD, 0.0) };
    let me = Driving {
        at,
        yaw,
        len: car.len,
        wid: car.wid,
        ahead,
        behind,
        first: key,
    };
    let now = Driving {
        at: was,
        yaw: was_yaw,
        ..me
    };
    car.next = Some((at, yaw));
    // Not into what's left of a claim another holds. (Pulling over to pass one in it only as far
    // as it will on its way: not into a junction's box just ahead, `pull_want`.)
    let (body_now, body_next) = (now.body(), me.body());
    let pass = ([1.0, 10.0, 20.0]
        .into_iter()
        .map(|k| route_pass(car, t + k))
        .fold(f32::INFINITY, f32::min)
        - shift)
        .max(0.0);
    let holding_here = traffic.holding(key);
    let in_drive = !home && !holding_here && first_claim.is_some_and(|from| t < from);
    if let Some(k) = traffic.blocked(key, &body_now, &body_next, me.heading(), pass, in_drive) {
        car.waits_for = Some(k);
        return;
    }
    // Not on past one coming the other way too near to pass (pulling over meanwhile); but for
    // one waiting for it (for its claim, or its body in the way of its own): it goes on by, as
    // near as it can (never into it, `guard`). Nor in a junction's box holding its claim there:
    // there the claims say who goes (one coming the other way let in with it can pass it in their
    // lanes, nobody pulls over there; one not let in waits for it), and waiting for one coming
    // would let that one's claim through this one's (`Traffic::begin`'s rule 1).
    let boxed = holding_here && in_box(&travel.graph.boxes, at, car.len);
    if !reverse
        && !boxed
        && let Some(o) = others.iter().find(|o| {
            o.free(&me)
                && !traffic.waits_for(o.first).contains(&key)
                && too_near((at, yaw, car.len, car.wid), (o.at, o.yaw, o.len, o.wid))
                && !too_near(
                    (was, was_yaw, car.len, car.wid),
                    (o.at, o.yaw, o.len, o.wid),
                )
        })
    {
        car.waits_for = Some(o.first);
        return;
    }
    let nav = crate::path::NavGrid {
        lot,
        blocked,
        stairs,
    };
    let mut progress = t;
    let held_up = go_on(
        sims,
        &nav,
        (content, objects),
        me,
        now,
        &others,
        &mut car.waited,
        &mut progress,
        &mut car.on_foot,
    );
    if held_up {
        car.waits_for = against(&me, &others).next().map(|k| others[k].first);
    }
    if progress > t {
        // (The last resort: never deeper into another car than it is.)
        match guard(&body_now, &body_next, others.iter().chain(&standing)) {
            Some(k) => {
                traffic.guarded += 1;
                car.waits_for = Some(k);
            }
            None => {
                car.progress = progress;
                car.moved = true;
            }
        }
    }
    // (Where it is now, for those after it this tick: one coming in behind it sees it there.)
    let progress = car.progress;
    let shift = car.shift;
    if progress > t && progress < ticks {
        let (p, yaw, _) = car.route(exits, Some(travel), home).at(progress);
        car.shown = Some((shifted(p, yaw, shift), yaw, true, 0));
    }
}

/// A work taxi a tick on along its run (`taxi.rs`): as any car drives on.
pub(crate) fn drive_taxi(w: &mut World, c: usize) {
    w.cars[c].standing = false;
    drive_on(w, c);
}

/// A work taxi standing at a stop (`taxi.rs`): pulled over for one coming the other way as far as
/// it has room, as any car standing.
pub(crate) fn stand_taxi(w: &mut World, c: usize) {
    let car = &mut w.cars[c];
    car.standing = true;
    car.waits_for = None;
    // (Where it stands, for its doors and seats: `Car::door`, `getting`.)
    if let Some((_, route)) = car.route.as_ref() {
        let (p, yaw, _) = route.at(car.progress);
        car.spot = shifted(p, yaw, car.shift);
        car.yaw = yaw;
    }
    let others = vehicles(&w.cars, Some(c));
    pull_over(w, c, &others, false);
}

/// How long a car and one in its way wait for each other before it backs up (ticks,
/// `back_off`), and how far back it goes at most (ticks along its route).
const JAM_TICKS: u32 = 5 * crate::TICKS_PER_MINUTE as u32;
const JAM_BACK: u32 = 60;

/// Whether car `k` (its `Driving::first`) waited for car `me` last tick, so that car `me`, waiting
/// for it, should back up (`back_off`): of two cars, the one not holding a claim (holding one,
/// the other was let through where it waits: it's that one that's in the way); else the later.
fn waits_back(w: &World, k: u64, me: u64) -> bool {
    let o = (k & !(1 << 63)) as usize;
    let (mine, theirs) = (w.traffic.holding(me), w.traffic.holding(k));
    let mine_to_go = if mine == theirs { me > k } else { theirs };
    mine_to_go
        && (w.cars.get(o).is_some_and(|c| c.waits_for == Some(me))
            || w.traffic.waits_for(k).contains(&me))
}

/// Car `c` a tick back along its route, out of the way of one it holds up as that one holds it
/// up (`JAM_TICKS`): never into another (`guard`) nor into what's left of another's claim
/// (`traffic::Traffic::blocked`). Whether it moved.
fn back_off(w: &mut World, c: usize) -> bool {
    let key = car_first(c);
    let others = vehicles(&w.cars, Some(c));
    let standing: Vec<Driving> = parked(&w.cars).collect();
    let car = &w.cars[c];
    let Some((home, route)) = car.route.as_ref() else {
        return false;
    };
    let (t, shift, half) = (car.progress, car.shift, [car.len / 2.0, car.wid / 2.0]);
    let to = (t - 1.0).max(0.0);
    if to >= t || route.at(to).2 != route.at(t).2 {
        return false;
    }
    let body = |t: f32| {
        let (p, yaw, _) = route.at(t);
        Rect {
            at: shifted(p, yaw, shift),
            yaw,
            half,
        }
    };
    let (now, next) = (body(t), body(to));
    let way = [-next.yaw.sin(), -next.yaw.cos()];
    // (Back out of the claim of the one it's held up with, too: away from it is out of its way.)
    let partner = car.waits_for;
    if guard(&now, &next, others.iter().chain(&standing)).is_some()
        || w.traffic
            .blocked(key, &now, &next, way, 0.0, false)
            .is_some_and(|k| Some(k) != partner)
    {
        return false;
    }
    let home = *home;
    let car = &mut w.cars[c];
    car.progress = to;
    car.jam.1 += 1;
    if home == (car.state == CarState::Arriving) {
        car.shown = Some((next.at, next.yaw, true, 0));
    }
    true
}

/// How far right of its lane car `car` has room to pull over `t` ticks along its route
/// (`Route::room_at`).
fn route_room(car: &Car, t: f32) -> f32 {
    car.route.as_ref().map_or(0.0, |(_, r)| r.room_at(t))
}

/// How far car `car` can pull over `t` ticks along its route (`Route::pass_at`).
fn route_pass(car: &Car, t: f32) -> f32 {
    car.route.as_ref().map_or(0.0, |(_, r)| r.pass_at(t))
}

/// The last-resort check on a car's move from body `now` to `next`: the first of `them` (the cars
/// on the road and parked) it would be deeper into than it is now. None: it may move.
fn guard<'a>(now: &Rect, next: &Rect, them: impl Iterator<Item = &'a Driving>) -> Option<u64> {
    let deeper = |r: &Rect| {
        let after = next.overlap(r);
        after > 1e-3 && after > now.overlap(r).max(0.0) + 1e-3
    };
    let near = |at: [f32; 2], len: f32| {
        crate::travel::dist(at, next.at) < (len + next.half[0] * 2.0) + AHEAD
    };
    them.into_iter()
        .find(|o| near(o.at, o.len) && deeper(&o.body()))
        .map(|o| o.first)
}

/// Car `c` a tick nearer pulled over as far as it would be (`pull_want`), `PULL_RATE` a tick:
/// back towards its lane only clear of those coming the other way (`too_near`), and never
/// into another (`guard`). Where its room runs out it's back in its lane.
fn pull_over(w: &mut World, c: usize, others: &[Driving], moving: bool) {
    // (Pulled over for one that waits for it, it pulls over only as far as it can on its way.)
    let me = car_first(c);
    let waits_me = |k: u64| {
        w.traffic.waits_for(k).contains(&me)
            || w.cars
                .get((k & !(1 << 63)) as usize)
                .is_some_and(|o| o.waits_for == Some(me))
    };
    let waited = w.cars[c].pulled_for.is_some_and(waits_me);
    let (mut want, mut whom) = pull_want(w, c, moving || waited);
    if !(moving || waited) && whom.is_some_and(waits_me) {
        (want, whom) = pull_want(w, c, true);
    }
    w.cars[c].pulled_for = whom;
    let standing: Vec<Driving> = parked(&w.cars).collect();
    let car = &w.cars[c];
    let Some((_, route)) = car.route.as_ref() else {
        return;
    };
    let t = car.progress;
    let room = route.room_here(t);
    let (p, yaw, _) = route.at(t);
    let half = [car.len / 2.0, car.wid / 2.0];
    let body = |s: f32| Rect {
        at: shifted(p, yaw, s),
        yaw,
        half,
    };
    let rate = PULL_RATE;
    let mut shift = car.shift + (want - car.shift).clamp(-rate, rate);
    // (Back towards its lane only clear of those coming the other way; but for one waiting for
    // it, which won't pass: it pulls back in to go on by, never into it, `guard`.)
    let (at, len, wid) = (body(shift).at, car.len, car.wid);
    let key = car_first(c);
    let waits_for_me = |o: &Driving| {
        w.traffic.waits_for(o.first).contains(&key)
            || w.cars
                .get((o.first & !(1 << 63)) as usize)
                .is_some_and(|x| x.waits_for == Some(key))
    };
    if shift < car.shift
        && let Some(o) = others.iter().find(|o| {
            too_near((at, yaw, len, wid), (o.at, o.yaw, o.len, o.wid)) && !waits_for_me(o)
        })
    {
        shift = car.shift;
        // (It waits for that one to pass: one that waits for it goes by, `drive_on`.)
        w.cars[c].waits_for = Some(o.first);
    }
    let car = &w.cars[c];
    // (Its room run out: back in its lane, whatever.)
    let shift = shift.min(room).max(0.0);
    if (shift - car.shift).abs() < 1e-5 {
        return;
    }
    // (Out, or back in: never into another, one going by that waited for it too. Kept from
    // pulling back in by one, it waits for that one: who waits for whom sees it, `back_off`.)
    if let Some(k) = guard(
        &body(car.shift),
        &body(shift),
        others.iter().chain(&standing),
    ) {
        if shift < car.shift {
            w.cars[c].waits_for = Some(k);
        }
        return;
    }
    w.cars[c].shift = shift;
}

/// How far right of its lane car `c` would be pulled over (m) to pass those coming the other way in
/// front of it (`PULL_AHEAD`) where the lane is too narrow for both in their lanes: half the
/// way past another car (it pulls over the other half, as far as it can: `Route::room`); as far as
/// its room ahead allows (back in its lane before the room runs out).
fn pull_want(w: &World, c: usize, moving: bool) -> (f32, Option<u64>) {
    let car = &w.cars[c];
    let Some((_, route)) = car.route.as_ref() else {
        return (0.0, None);
    };
    let t = car.progress;
    // (On the move, back in its lane before its room runs out; standing, as far as it has room
    // where it stands.)
    let room = if moving {
        [0.0, 1.0, 10.0, 20.0]
            .into_iter()
            .map(|k| route.room_at(t + k))
            .fold(f32::INFINITY, f32::min)
    } else {
        route.room_here(t)
    };
    let (at, yaw, back) = route.at(t);
    // (Not in a junction's box: there they take turns, `traffic.rs`, and turning there it faces
    // any way, those following it in too. For one coming the other way in it, as anywhere: the
    // claims let that one in counting on this one's pulling over.)
    let boxes = &w.travel.graph.boxes;
    if room <= 0.0 || back || in_box(boxes, at, car.len) {
        return (0.0, None);
    }
    let short = |p: [f32; 2], facing: f32, len: f32, wid: f32| {
        let m = passing((at, yaw, car.len, car.wid), (p, facing, len, wid))?;
        let ends = (car.len + len) / 2.0;
        if m.along < -ends || m.along > ends + PULL_AHEAD || m.apart < 0.0 {
            return None;
        }
        Some(((PASSING + PULL_MARGIN - m.gap).max(0.0), m.mine, m.theirs))
    };
    let (mut want, mut whom): (f32, Option<u64>) = (0.0, None);
    for (o, other) in w.cars.iter().enumerate() {
        if o == c || other.shown.is_none_or(|s| !s.2) || other.backing() {
            continue;
        }
        let Some((_, theirs)) = other.route.as_ref() else {
            continue;
        };
        let (p, facing, _) = theirs.at(other.progress);
        if let Some((all, mine, their)) = short(p, facing, other.len, other.wid) {
            let share = (all / 2.0).max(all - theirs.room_at(other.progress) * their);
            if share / mine > want {
                (want, whom) = (share / mine, Some(car_first(o)));
            }
        }
    }
    (want.min(room), whom)
}

/// A work taxi's stop (`taxi.rs`): by a house's driveway, `mouth` where it meets the road. The
/// taxi stands at the kerb in its lane there; where it has to turn round to go on, it turns there
/// as a car does, nose first into the driveway's mouth (its middle to `spot`, facing `yaw`, the
/// driveway running to the street along `axis`) and backing out, after it has stood
/// (`turn`: none, no room in the mouth for that).
#[derive(Debug, Clone, Copy)]
pub(crate) struct TaxiStop {
    pub spot: [f32; 2],
    pub yaw: f32,
    pub axis: [f32; 2],
    pub mouth: RoadPoint,
    pub turn: bool,
}

/// How far into a driveway from the road's edge a taxi turning in it goes at most, past half its
/// length (m), and how far over the pavement its back may stay (m) where the household's car
/// parked up the driveway leaves no more room (`taxi_stop`).
const TAXI_IN: f32 = 0.3;
const TAXI_OVER: f32 = 1.0;
/// How far from a public parking bay's middle a taxi turning in a driveway's mouth keeps its own
/// middle, past half its length (m: a car's length and its turn's swing).
const BAY_CLEAR: f32 = 6.0;
/// Room kept between a taxi turning in a driveway's mouth and the car parked up it (m).
const TAXI_GAP: f32 = 0.4;

/// Where a taxi `len` long stops at a house's driveway (`TaxiStop`), and whether it can turn in its
/// mouth, clear of the household's car parked up it (`parked`: its length, if it's there). None:
/// no driveway meeting a road for cars.
pub(crate) fn taxi_stop(
    travel: &Travel,
    drive: &Drive,
    len: f32,
    parked: Option<f32>,
) -> Option<TaxiStop> {
    let g = &travel.graph;
    let way = drive.way()?;
    let (road, s) = drive.meets()?;
    let mouth = g.at_source(road, s)?;
    let r = &g.roads[mouth.road as usize];
    if !r.drivable() {
        return None;
    }
    let mp = g.point(mouth);
    // (From the inner end along the driveway to the road's edge.)
    let inner = [drive.x, drive.z];
    let to_edge = (mp[0] - inner[0]) * way[0] + (mp[1] - inner[1]) * way[1] - r.width / 2.0;
    let free = to_edge - parked.map_or(0.0, |l| PARK_GAP + l + TAXI_GAP);
    let depth = (len / 2.0 + TAXI_IN).min(free - len / 2.0);
    let k = to_edge - depth;
    let spot = [inner[0] + way[0] * k, inner[1] + way[1] * k];
    let yaw = (-way[0]).atan2(-way[1]);
    // (Not where a car parked out in a public bay by the mouth may stand in the way of its turn:
    // its body would be under the taxi's path, held up for good by it.)
    let bay_by = travel
        .file
        .parking
        .iter()
        .any(|p| crate::travel::dist([p.x, p.z], spot) < len / 2.0 + BAY_CLEAR);
    let turn = depth >= len / 2.0 - TAXI_OVER
        && !bay_by
        && Bay::new(spot, yaw, Some(way), mp, r, mouth.s).is_some();
    Some(TaxiStop {
        spot,
        yaw,
        axis: way,
        mouth,
        turn,
    })
}

/// How far before and after where a taxi stands at a stop its claim there reaches (m,
/// `taxi_route`): those behind it, or turning into or out of a driveway by it, wait for it.
const TAXI_KERB_CLAIM: f32 = 4.0;

/// A work taxi's route (`taxi.rs`), `len` by `wid`: in from `way_in` (a way out of town, from off
/// the map as a car coming home is) along the roads to each of `stops`, standing at the kerb in its
/// lane by each (a claim of the lane there, held as it stands), then out at `way_out` and on off the
/// map. Where it goes on back the way it came, it turns at the stop, nose first into the
/// driveway's mouth and backing out (its claim held from the kerb till it's back in the lane); its
/// claims worked out (`Route::claim`). With the ticks along it where it stands at each stop. None:
/// no way by the roads, or no room to turn where it has to.
pub(crate) fn taxi_route(
    travel: &Travel,
    way_in: [f32; 2],
    stops: &[TaxiStop],
    way_out: [f32; 2],
    len: f32,
    wid: f32,
) -> Option<(Route, Vec<f32>)> {
    use crate::travel::{RoadKind, dist};
    let g = &travel.graph;
    let entry = *g.near(way_in, ROAD_REACH, 1, Mode::Car).first()?;
    let exit = *g.near(way_out, ROAD_REACH, 1, Mode::Car).first()?;
    let junctions: Vec<[f32; 2]> = g.junctions().collect();
    // (Each leg's way, and where it turns round: going on back the way it came from a stop.)
    let mut paths = Vec::with_capacity(stops.len() + 1);
    let mut from = entry;
    for k in 0..=stops.len() {
        let to = if k == stops.len() {
            exit
        } else {
            stops[k].mouth
        };
        let path = g.path(from, to, Mode::Car, travel.country_speed)?;
        if path.pts.len() < 2 {
            return None;
        }
        paths.push(path);
        from = to;
    }
    let seg = |pts: &[([f32; 2], f32, RoadKind)], end: bool| {
        let n = pts.len();
        let (a, b) = if end {
            (pts[n.saturating_sub(4)].0, pts[n - 1].0)
        } else {
            (pts[0].0, pts[3.min(n - 1)].0)
        };
        [b[0] - a[0], b[1] - a[1]]
    };
    let turns: Vec<bool> = (0..stops.len())
        .map(|k| {
            let (a, b) = (seg(&paths[k].pts, true), seg(&paths[k + 1].pts, false));
            a[0] * b[0] + a[1] * b[1] < 0.0
        })
        .collect();
    let mut bays: Vec<Option<Bay>> = Vec::with_capacity(stops.len());
    for (k, st) in stops.iter().enumerate() {
        if !turns[k] {
            bays.push(None);
            continue;
        }
        if !st.turn {
            return None;
        }
        bays.push(Some(Bay::new(
            st.spot,
            st.yaw,
            Some(st.axis),
            g.point(st.mouth),
            &g.roads[st.mouth.road as usize],
            st.mouth.s,
        )?));
    }
    let mut legs = Legs::default();
    let mut at_stops = Vec::new();
    // (Where it stood at the last stop, for its claim there, closed once the next leg is laid; and
    // leaving a turn there: its bay, the way it goes, where its turn is and how much it takes.)
    let mut stood: Option<usize> = None;
    let mut start: Option<(usize, f32, f32, f32)> = None;
    for (k, path) in paths.iter().enumerate() {
        let last = k == stops.len();
        let mut centre: Vec<[f32; 2]> = path.pts.iter().map(|p| p.0).collect();
        let mut offs: Vec<f32> = path.pts.iter().map(|p| lane_on(p.1, p.2)).collect();
        if k == 0 {
            // (In from off the map.)
            centre.insert(0, extend_past(centre[1], centre[0]));
            offs.insert(0, offs[0]);
        }
        if last {
            centre.push(extend_past(
                centre[centre.len() - 2],
                centre[centre.len() - 1],
            ));
            offs.push(offs[offs.len() - 1]);
        }
        let lane = crate::travel::thin_with(
            &offset_right_by(&centre, &offs),
            ROUTE_STEP,
            ROUTE_TOLERANCE,
        );
        if lane.len() < 2 {
            return None;
        }
        let road_at = |p: [f32; 2]| {
            path.pts
                .iter()
                .min_by(|x, y| dist(x.0, p).total_cmp(&dist(y.0, p)))
                .map_or((7.0, RoadKind::Street), |x| (x.1, x.2))
        };
        let top = |p: [f32; 2]| top_speed(p, road_at(p).1, travel, &junctions);
        // (Turning at the last stop: into the driveway's mouth and backing out the way the lane
        // goes on.)
        if let Some((j, ..)) = start {
            let bay = bays[j].as_ref().expect("a turn");
            let ahead = bay.local(lane[(lane.len() - 1).min(3)]);
            let d = if ahead[0] >= bay.local(lane[0])[0] {
                1.0
            } else {
                -1.0
            };
            let (at, taken) = bay.leave(&mut legs, d);
            start = Some((j, d, at, taken));
        }
        // (Turning at the next stop: it stands at the kerb short of its turn in.)
        let into = bays
            .get(k)
            .filter(|_| !last)
            .and_then(Option::as_ref)
            .map(|bay| {
                let road = &g.roads[stops[k].mouth.road as usize];
                let rt = road.tangent(stops[k].mouth.s);
                let (a, b) = (lane[lane.len() - 2], lane[lane.len() - 1]);
                let go = (b[0] - a[0]) * rt[0] + (b[1] - a[1]) * rt[1];
                let d = if go >= 0.0 { -1.0 } else { 1.0 };
                (k, d, bay.entry(d))
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
        let mut on = false;
        for (i, &p) in lane.iter().enumerate() {
            if !on
                && let Some((j, d, at, taken)) = start
                && let Some(bay) = &bays[j]
                && bay.near(p, at, taken, d)
                && along(bay, i, d)
            {
                continue;
            }
            if let Some((j, d, (turn, at))) = &into
                && let Some(bay) = &bays[*j]
                && bay.near(p, *at, turn.taken, *d)
                && along(bay, i, -*d)
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
            legs.push_lane(p, top(p), pull_room(width, kind, wid), lane_on(width, kind));
            if first && let Some(s) = stood {
                // (Its claim from where it stood, through its turn, back to the lane.)
                legs.mouths.push((s, Some(legs.len() - 1)));
            }
        }
        if !on && let Some(s) = stood {
            legs.mouths.push((s, None));
        }
        // (Standing at the last stop with no turn: its stretch of the lane, from just before to
        // just after.)
        if start.is_none()
            && let Some(s) = stood
        {
            let n = (TAXI_KERB_CLAIM / STEP).ceil() as usize;
            legs.mouths.push((
                s.saturating_sub(n),
                Some((s + n).min(legs.len().saturating_sub(1))),
            ));
        }
        start = None;
        if last {
            break;
        }
        // At the kerb: it slows to a stand at the lane's end there.
        if let Some(p) = legs.pts.last_mut() {
            p.2 = 0.0;
        }
        let s = legs.len() - 1;
        at_stops.push(s);
        stood = Some(s);
        if let Some((j, _, (turn, _))) = into {
            bays[j].as_ref().expect("a turn").enter(&mut legs, &turn);
            start = Some((j, 0.0, 0.0, 0.0));
        }
    }
    let mut route = legs.timed(false, false);
    let ticks = at_stops.iter().map(|&i| route.points[i].2).collect();
    route.claim(len, wid, Some(travel));
    Some((route, ticks))
}

/// Whether a body `len` long with its middle at `p` is in one of the junctions' `boxes`, about.
fn in_box(boxes: &[([f32; 2], f32)], p: [f32; 2], len: f32) -> bool {
    boxes
        .iter()
        .any(|&(m, r)| crate::travel::dist(m, p) < r + len / 2.0)
}

/// The cars on the road this tick, for the claims (`traffic::begin`): those driving (`places`:
/// where along their routes, by their routes' tags: home or not), and those in sight (`movers`:
/// their bodies, the way they go, and if they moved last tick where they'll be along their way;
/// one still in its driveway, not yet holding its claim out, as parked).
pub(crate) fn movers(
    w: &mut World,
    movers: &mut Vec<crate::traffic::Mover>,
    places: &mut Vec<crate::traffic::Place>,
) {
    let World {
        cars,
        exits,
        travel,
        traffic,
        ..
    } = w;
    for (c, car) in cars.iter_mut().enumerate() {
        if !matches!(car.state, CarState::Leaving | CarState::Arriving) {
            continue;
        }
        let key = car_first(c);
        let home = car.state == CarState::Arriving;
        places.push((key, u32::from(home), car.progress));
        let Some((p, yaw, true, _)) = car.shown else {
            continue;
        };
        let k = if car.backing() { -1.0 } else { 1.0 };
        let half = [car.len / 2.0, car.wid / 2.0];
        let parked = !home && car.progress <= 0.0 && !traffic.holding(key);
        let (t, shift, moved) = (car.progress, car.shift, car.moved);
        let route = car.route(exits, Some(travel), home);
        let ahead = if moved {
            crate::traffic::LOOK
                .iter()
                .filter(|&&dt| t + dt < route.ticks())
                .map(|&dt| {
                    let (q, qy, _) = route.at(t + dt);
                    Rect {
                        at: shifted(q, qy, shift),
                        yaw: qy,
                        half,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        // (On the move, as far as it will pull over on its way: not into a junction's box or a
        // narrow span just ahead, as `drive_on` sees it. Standing, waiting at a box's edge, as far
        // as it can where it stands: those coming out of the box pass it there.)
        let ahead_too: &[f32] = if moved { &[0.0, 10.0, 20.0] } else { &[0.0] };
        let pass = (ahead_too
            .iter()
            .map(|&k| route.pass_at(t + k))
            .fold(f32::INFINITY, f32::min)
            - shift)
            .max(0.0);
        movers.push(crate::traffic::Mover {
            key,
            rect: Rect { at: p, yaw, half },
            way: [yaw.sin() * k, yaw.cos() * k],
            pass,
            moving: moved,
            ahead,
            parked,
            waits: car.waits_for,
        });
    }
}

/// Whether car `c`, out of sight at the way in (`Car::outside`), comes in now: none there longer
/// waits to come in by the same way (the earlier car, of two there as long), and its body a tick
/// on and the ground in front of it (`AHEAD`) are clear of every car in town (`others`,
/// `vehicles`: those that came in this tick where they are now) and every car parked.
fn room_to_come_in(w: &mut World, c: usize, others: &[Driving]) -> bool {
    let World {
        cars,
        exits,
        travel,
        ..
    } = w;
    let (at, yaw, _) = cars[c].route(exits, Some(travel), true).at(1.0);
    let start = |car: &Car| car.route.as_ref().filter(|r| r.0).map(|r| r.1.at(0.0).0);
    let mine = start(&cars[c]);
    let before = cars.iter().enumerate().any(|(o, other)| {
        o != c
            && other.outside()
            && (other.since, o) < (cars[c].since, c)
            && start(other)
                .zip(mine)
                .is_some_and(|(a, b)| crate::travel::dist(a, b) < 4.0)
    });
    let me = Driving {
        at,
        yaw,
        len: cars[c].len,
        wid: cars[c].wid,
        ahead: AHEAD,
        behind: 0.0,
        first: car_first(c),
    };
    let (body, front) = (me.body(), me.front());
    !before
        && others
            .iter()
            .copied()
            .chain(parked(cars))
            .all(|o| o.body().overlap(&body) <= 0.0 && o.body().overlap(&front) <= 0.0)
}

/// A car driving where it is now, for who's in its way (`go_on`): its middle and yaw, its length
/// and width, and how far in front of it and behind it someone is in its way (m, past its ends);
/// and which it is (`first`: `car_first`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Driving {
    pub at: [f32; 2],
    pub yaw: f32,
    pub len: f32,
    pub wid: f32,
    pub ahead: f32,
    pub behind: f32,
    pub first: u64,
}

impl Driving {
    /// The ground from its middle to `ahead` past its front (or, backing up, `behind` past its
    /// back), as wide as it is: anything there is in its way (`up_against`). Its middle, its yaw
    /// and its half length and width.
    fn front(&self) -> Rect {
        let (s, c) = self.yaw.sin_cos();
        let (reach, k) = if self.ahead > 0.0 || self.behind <= 0.0 {
            (self.len / 2.0 + self.ahead, 1.0)
        } else {
            (self.len / 2.0 + self.behind, -1.0)
        };
        let along = k * reach / 2.0;
        Rect {
            at: [self.at[0] + s * along, self.at[1] + c * along],
            yaw: self.yaw,
            half: [reach / 2.0, self.wid / 2.0],
        }
    }

    /// How far `body` at `pos` is from it and the ground in front of it (behind it, backing up):
    /// below 0, how far into them (`Body::gap`).
    fn gap(&self, body: Body, pos: [f32; 2]) -> f32 {
        body.gap(
            self.at,
            self.yaw.sin_cos(),
            pos,
            -(self.len / 2.0 + self.behind),
            self.len / 2.0 + self.ahead,
            self.wid / 2.0,
        )
    }

    /// Its body.
    fn body(&self) -> Rect {
        Rect {
            at: self.at,
            yaw: self.yaw,
            half: [self.len / 2.0, self.wid / 2.0],
        }
    }

    /// Whether `d` waits for it at all: not while it backs up, out of a driveway or into one; of
    /// two tangled, the earlier (`first`) doesn't wait for the later, but the later for it, so the
    /// earlier pulls ahead and they part (going on together, neither waiting, they'd stay on top of
    /// each other).
    pub(crate) fn free(&self, d: &Driving) -> bool {
        let tangled = self.body().overlap(&d.body()) > 0.0;
        self.ahead > 0.0 && !(tangled && d.first < self.first)
    }

    /// The way it's going (a unit vector: backwards, backing up).
    pub(crate) fn heading(&self) -> [f32; 2] {
        let (s, c) = self.yaw.sin_cos();
        let k = if self.ahead > 0.0 || self.behind <= 0.0 {
            1.0
        } else {
            -1.0
        };
        [s * k, c * k]
    }
}

/// A rectangle on the ground: its middle, its yaw (its length along (sin, cos)), and its half
/// length and width (m).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Rect {
    pub at: [f32; 2],
    pub yaw: f32,
    pub half: [f32; 2],
}

impl Rect {
    /// How far the two overlap (m, the least along any of their sides' ways; 0 or less: apart).
    pub(crate) fn overlap(&self, o: &Rect) -> f32 {
        let axes = |yaw: f32| {
            let (s, c) = yaw.sin_cos();
            [[s, c], [c, -s]]
        };
        let (a, b) = (axes(self.yaw), axes(o.yaw));
        let d = [o.at[0] - self.at[0], o.at[1] - self.at[1]];
        let reach = |r: &Rect, ax: &[[f32; 2]; 2], u: [f32; 2]| {
            r.half[0] * (ax[0][0] * u[0] + ax[0][1] * u[1]).abs()
                + r.half[1] * (ax[1][0] * u[0] + ax[1][1] * u[1]).abs()
        };
        a.iter()
            .chain(b.iter())
            .map(|&u| reach(self, &a, u) + reach(o, &b, u) - (d[0] * u[0] + d[1] * u[1]).abs())
            .fold(f32::INFINITY, f32::min)
    }
}

/// Whether a car driving (`d`, where it would be a tick on) comes up against another of `them`
/// (`vehicles`, as they are now): one whose body is in the ground in front of it
/// (`Driving::front`), going its way. It keeps behind it. (One coming the other way passes it in
/// the other lane, both pulled over where the lane is too narrow for both: `pull_over`.) It waits
/// only behind one in front of it going its way, driving on (not backing up), so whoever waits
/// for one waits for something that moves on.
pub(crate) fn up_against(d: &Driving, them: &[Driving]) -> bool {
    against(d, them).next().is_some()
}

/// Those of `them` (indices) `d` comes up against (`up_against`).
pub(crate) fn against<'a>(d: &'a Driving, them: &'a [Driving]) -> impl Iterator<Item = usize> + 'a {
    // (Backing up, out of a driveway or into one, a car waits for nobody but those on foot.)
    let backing = d.ahead <= 0.0;
    let (front, heading) = (d.front(), d.heading());
    them.iter()
        .enumerate()
        .filter(move |(_, o)| {
            let way = o.heading();
            let cos = heading[0] * way[0] + heading[1] * way[1];
            if backing || cos < ONCOMING || !o.free(d) {
                return false;
            }
            let reach = d.len + d.ahead.max(d.behind) + o.len;
            if (o.at[0] - d.at[0]).hypot(o.at[1] - d.at[1]) > reach
                || front.overlap(&o.body()) <= 0.0
            {
                return false;
            }
            true
        })
        .map(|(k, _)| k)
}

/// Another driving towards something (the cosine of the angle between their ways, less than
/// minus this) is coming the other way (`up_against`, `passing`, `too_near`; and for the claims,
/// `traffic::conflict`).
pub(crate) const ONCOMING: f32 = 0.7;

/// Two coming towards each other, measured along and across the way between theirs (the same
/// for both: on a bend either may look to be off the other's right): how far the second is along
/// it from the first; how far apart their middles are across it (the second on the first's left:
/// more), and the gap there is between their bodies (m: less than none, they may overlap); and
/// how far across it each moves pulling over a metre (`pull_want`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Passing {
    pub along: f32,
    pub apart: f32,
    pub gap: f32,
    pub mine: f32,
    pub theirs: f32,
}

/// `Passing` for `a` and `b` (each where, facing which way, how long and wide), if they're coming
/// towards each other (else none).
pub(crate) fn passing(
    a: ([f32; 2], f32, f32, f32),
    b: ([f32; 2], f32, f32, f32),
) -> Option<Passing> {
    let (ahead, theirs) = ([a.1.sin(), a.1.cos()], [b.1.sin(), b.1.cos()]);
    if ahead[0] * theirs[0] + ahead[1] * theirs[1] >= -ONCOMING {
        return None;
    }
    let u = [ahead[0] - theirs[0], ahead[1] - theirs[1]];
    let n = u[0].hypot(u[1]).max(1e-6);
    let (u, left) = ([u[0] / n, u[1] / n], [u[1] / n, -u[0] / n]);
    let d = [b.0[0] - a.0[0], b.0[1] - a.0[1]];
    let (along, apart) = (d[0] * u[0] + d[1] * u[1], d[0] * left[0] + d[1] * left[1]);
    // (How far each reaches across that way, turned as it is; and moves, pulling over to its
    // right.)
    let reach = |way: [f32; 2], len: f32, wid: f32| {
        let k = way[0] * left[0] + way[1] * left[1];
        len / 2.0 * k.abs() + wid / 2.0 * (1.0 - k * k).max(0.0).sqrt()
    };
    let pulls = |way: [f32; 2]| (way[0] * u[0] + way[1] * u[1]).abs().max(0.3);
    Some(Passing {
        along,
        apart,
        gap: apart.abs() - reach(ahead, a.2, a.3) - reach(theirs, b.2, b.3),
        mine: pulls(ahead),
        theirs: pulls(theirs),
    })
}

/// Whether `a` (where it would be a tick on) would come too near `b` coming the other way: its
/// body, half `PASSING` wider either side, on the other's. (Pulled over as far as `passing` says,
/// they're further apart than that: it measures across a way between theirs.)
pub(crate) fn too_near(a: ([f32; 2], f32, f32, f32), b: ([f32; 2], f32, f32, f32)) -> bool {
    let (ahead, theirs) = ([a.1.sin(), a.1.cos()], [b.1.sin(), b.1.cos()]);
    let mine = Rect {
        at: a.0,
        yaw: a.1,
        half: [a.2 / 2.0, a.3 / 2.0 + PASSING / 2.0],
    };
    let other = Rect {
        at: b.0,
        yaw: b.1,
        half: [b.2 / 2.0, b.3 / 2.0],
    };
    ahead[0] * theirs[0] + ahead[1] * theirs[1] < -ONCOMING && mine.overlap(&other) > 0.0
}

/// Someone's body as cars keep clear of it, and it of them: how far it reaches in front
/// of where they are and behind them, and out to each side (m), facing `yaw`. On foot, `BODY`
/// all round; on a bike, its wheels in front and behind and its handlebars to the sides (content
/// `vehicle.reach`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Body {
    pub ahead: f32,
    pub behind: f32,
    pub side: f32,
    pub yaw: f32,
}

impl Body {
    /// Someone on foot.
    pub(crate) const WALKER: Body = Body {
        ahead: BODY,
        behind: BODY,
        side: BODY,
        yaw: 0.0,
    };

    /// `sim`'s, facing `yaw`: on foot, or on what they ride.
    pub(crate) fn of(
        sim: &Sim,
        content: &crate::content::Content,
        objects: &[crate::world::ObjectInstance],
        yaw: f32,
    ) -> Body {
        let ride = sim
            .riding
            .and_then(|o| objects.get(o as usize))
            .and_then(|o| content.objects[o.def].vehicle);
        let Some([ahead, behind, side]) = ride.map(|v| v.reach) else {
            return Body {
                yaw,
                ..Body::WALKER
            };
        };
        Body {
            ahead,
            behind,
            side,
            yaw,
        }
    }

    /// How far it is, at `pos`, from the box `back..front` along and `half` either side across
    /// of something at `at` turned `turn` (its yaw's sine and cosine): below 0, how far into it.
    fn gap(
        self,
        at: [f32; 2],
        turn: (f32, f32),
        pos: [f32; 2],
        back: f32,
        front: f32,
        half: f32,
    ) -> f32 {
        // (Points along its middle a fifth of a metre apart at most, as far as its ends less
        // its sides: round each, as far as its sides.)
        let (s, c) = self.yaw.sin_cos();
        let a = (self.ahead - self.side).max(0.0);
        let b = (self.behind - self.side).max(0.0);
        let n = ((a + b) / 0.2).ceil() as usize;
        let (mid, long) = ((front + back) / 2.0, (front - back) / 2.0);
        (0..=n)
            .map(|k| {
                let t = -b + (a + b) * k as f32 / n.max(1) as f32;
                let (along, across) = local(at, turn, [pos[0] + s * t, pos[1] + c * t]);
                let (dx, dz) = ((along - mid).abs() - long, across.abs() - half);
                dx.max(0.0).hypot(dz.max(0.0)) + dx.max(dz).min(0.0) - self.side
            })
            .fold(f32::INFINITY, f32::min)
    }
}

/// Something driving (`d`, where it would be a tick on) a tick further along its way, unless
/// someone is in its way: it waits for them (`waited`, ticks). Someone standing there a while (a
/// chat on the pavement) steps aside for it (`make_way`); someone with nowhere to step goes
/// round to where they're out of its way, past `PASS_TICKS` (`clear_of`). It never drives
/// through anyone. Someone at something or at a wheel isn't in its way; nor is anyone whose body (`Body`, on foot or on a bike) clears its sides by
/// `BESIDE`, walking by it or along beside it; nor anyone the tick on brings it no nearer (`now`:
/// where it is, as `d`; nearer measured from its whole body, its corners and ends as it turns, and
/// the ground ahead of it), as someone beside it waits for it (`walks_into`): they'd hold each
/// other up. Nor does it drive into another car in front of it (`up_against`): it waits behind it
/// (true), not counted in `waited`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn go_on(
    sims: &mut [Sim],
    nav: &crate::path::NavGrid,
    bodies: (&crate::content::Content, &[crate::world::ObjectInstance]),
    d: Driving,
    now: Driving,
    others: &[Driving],
    waited: &mut u32,
    progress: &mut f32,
    whom: &mut Vec<u32>,
) -> bool {
    let turn = d.yaw.sin_cos();
    whom.clear();
    // (Nobody further off than its corners and the ground ahead of or behind them, and the
    // longest bike, is in its way.)
    let reach = d.len / 2.0 + d.ahead.max(d.behind) + d.wid / 2.0 + BESIDE + 2.0;
    let mut waiting = false;
    for s in sims.iter_mut() {
        let (dx, dz) = (s.pos[0] - d.at[0], s.pos[1] - d.at[1]);
        if dx * dx + dz * dz >= reach * reach || !s.here() || s.away_until.is_some() {
            continue;
        }
        let body = Body::of(s, bodies.0, bodies.1, s.yaw);
        let phase = s.current().map(|a| &a.phase);
        // (Standing at the kerb for the taxi, they won't step out in front of it: in its way only
        // where its body goes, not the ground ahead.)
        let (ahead, behind) = if matches!(phase, Some(Phase::AtKerb { .. })) {
            (0.0, 0.0)
        } else {
            (d.ahead, d.behind)
        };
        // (Beside it, no nearer a tick on, clear of its body and the ground ahead: they wait for
        // it. Never into anyone.)
        let next = d.gap(body, s.pos);
        if !covers(d.at, turn, d.len, d.wid, body, s.pos, ahead, behind)
            || next >= 0.0 && next >= now.gap(body, s.pos)
        {
            continue;
        }
        if matches!(phase, Some(Phase::Using { .. }) | Some(Phase::Driving)) {
            continue;
        }
        waiting = true;
        whom.push(s.id);
        // (Walking on through its way, not waiting for it: they're on their way out of it, not
        // to be stepped back where they came from.)
        if matches!(phase, Some(Phase::Routing { .. })) && s.car_wait == 0 {
            continue;
        }
        // (Not into the way of another, nor where this one comes on to: `clear`.)
        let clear = |p: [f32; 2]| clear(others, body, p);
        // (Stepping aside that long hasn't got them out of its way.)
        let round = (*waited >= PASS_TICKS)
            .then(|| clear_of(&d, nav, body, s.pos, clear))
            .flatten();
        if let Some(p) = round.or_else(|| {
            (*waited > MAKE_WAY_TICKS)
                .then(|| make_way(d.at, d.yaw, nav, s.pos, 0.06, clear))
                .flatten()
        }) {
            s.pos = p;
        }
    }
    let blocked = up_against(&d, others);
    if waiting {
        *waited = waited.saturating_add(1);
    } else {
        // (Held up most ticks, by one walking ahead of it on the road and on again: it still
        // counts, so they step aside for it in the end, `make_way`.)
        *waited = waited.saturating_sub(2);
        if !blocked {
            *progress += 1.0;
        }
    }
    blocked
}

/// Every car driving by, as `walks_into` keeps clear of them (its body and the ground just in front
/// of it), but car `car`.
pub(crate) fn vehicles(cars: &[Car], car: Option<usize>) -> Vec<Driving> {
    let at = |(at, yaw): ([f32; 2], f32), len: f32, wid: f32, first: u64| Driving {
        at,
        yaw,
        len,
        wid,
        ahead: AHEAD,
        behind: 0.0,
        first,
    };
    let cars = cars.iter().enumerate().filter_map(|(c, k)| match k.shown {
        Some((p, yaw, true, _)) if Some(c) != car => {
            let mut d = at((p, yaw), k.len, k.wid, car_first(c));
            if k.backing() {
                (d.ahead, d.behind) = (0.0, AHEAD);
            }
            Some(d)
        }
        _ => None,
    });
    cars.collect()
}

/// Every car parked where it can be seen (at home, out on a trip): as `vehicles` has those on the
/// road, but standing.
pub(crate) fn parked(cars: &[Car]) -> impl Iterator<Item = Driving> + '_ {
    cars.iter().enumerate().filter_map(|(c, k)| match k.shown {
        Some((at, yaw, false, _)) => Some(Driving {
            at,
            yaw,
            len: k.len,
            wid: k.wid,
            ahead: AHEAD,
            behind: 0.0,
            first: car_first(c),
        }),
        _ => None,
    })
}

pub fn car_key(c: usize) -> u64 {
    car_first(c)
}

/// Car `c`'s `Driving::first` (its top bit set, as ever).
pub(crate) fn car_first(c: usize) -> u64 {
    (1 << 63) | c as u64
}

/// Whether `body` at `p` is out of the way of all of `them` (`vehicles`): somewhere to step aside
/// to, or to go round to, for someone in the way of another.
pub(crate) fn clear(them: &[Driving], body: Body, p: [f32; 2]) -> bool {
    them.iter().all(|d| {
        !covers(
            d.at,
            d.yaw.sin_cos(),
            d.len,
            d.wid,
            body,
            p,
            d.ahead,
            d.behind,
        )
    })
}

/// Someone at `pos` in the way of something driving (`d`) with nowhere to step aside: the middle
/// of the nearest tile they can walk to, `CLEAR_STEPS` at most, where their `body` is out of its
/// way, and (`clear`) of any other's. None: nowhere that near (it waits on).
fn clear_of(
    d: &Driving,
    nav: &crate::path::NavGrid,
    body: Body,
    pos: [f32; 2],
    clear: impl Fn([f32; 2]) -> bool,
) -> Option<[f32; 2]> {
    let turn = d.yaw.sin_cos();
    let start = (pos[0].floor() as i32, pos[1].floor() as i32);
    let mut seen = vec![start];
    let mut ring = vec![start];
    for _ in 0..CLEAR_STEPS {
        let mut next = Vec::new();
        for &(x, z) in &ring {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let t = (x + dx, z + dz);
                if !seen.contains(&t) && nav.can_step(x, z, t.0, t.1) {
                    seen.push(t);
                    next.push(t);
                }
            }
        }
        let far = |p: &[f32; 2]| (p[0] - pos[0]).hypot(p[1] - pos[1]);
        let clear = next
            .iter()
            .map(|&(x, z)| [x as f32 + 0.5, z as f32 + 0.5])
            .filter(|&p| !covers(d.at, turn, d.len, d.wid, body, p, d.ahead, d.behind) && clear(p))
            .min_by(|a, b| far(a).total_cmp(&far(b)));
        if clear.is_some() {
            return clear;
        }
        ring = next;
    }
    None
}

/// `pos` from something at `at` turned `turn` (its yaw's sine and cosine): how far along it,
/// ahead, and how far across it.
fn local(at: [f32; 2], turn: (f32, f32), pos: [f32; 2]) -> (f32, f32) {
    let (s, c) = turn;
    let (dx, dz) = (pos[0] - at[0], pos[1] - at[1]);
    (dx * s + dz * c, dx * c - dz * s)
}

/// Whether `body` at `pos` comes within `BESIDE` of something `len` by `wid` m (a car)
/// at `at`, turned `turn` (its yaw's sine and cosine), or of the ground `ahead` in front of it
/// or `behind` behind it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn covers(
    at: [f32; 2],
    turn: (f32, f32),
    len: f32,
    wid: f32,
    body: Body,
    pos: [f32; 2],
    ahead: f32,
    behind: f32,
) -> bool {
    body.gap(
        at,
        turn,
        pos,
        -(len / 2.0 + behind),
        len / 2.0 + ahead,
        wid / 2.0,
    ) < BESIDE
}

/// Whether a step from `from` to `to` with `body` (turned the way of the step) walks into
/// something `len` by `wid` m driving by at `at` facing `yaw`, within `KEEP` of it, or into the
/// ground just in front of it that it keeps clear (`AHEAD`): stepping out of its way is fine.
/// (Stopped there, waiting for another to pass, they'd hold it up as it held them.)
pub(crate) fn walks_into(
    at: [f32; 2],
    yaw: f32,
    len: f32,
    wid: f32,
    body: Body,
    from: [f32; 2],
    to: [f32; 2],
) -> bool {
    let turn = yaw.sin_cos();
    let body = Body {
        yaw: (to[0] - from[0]).atan2(to[1] - from[1]),
        ..body
    };
    let gap = |p: [f32; 2]| body.gap(at, turn, p, -len / 2.0, len / 2.0 + AHEAD, wid / 2.0);
    let g = gap(to);
    g < KEEP && g <= gap(from)
}

/// Whether `body` at `p` is in the way of a car (`walks_into`'s ground: its body and the ground in
/// front of it, `KEEP` round).
fn in_way_of(at: [f32; 2], yaw: f32, len: f32, wid: f32, body: Body, p: [f32; 2]) -> bool {
    body.gap(
        at,
        yaw.sin_cos(),
        p,
        -len / 2.0,
        len / 2.0 + AHEAD,
        wid / 2.0,
    ) < KEEP
}

/// Getting into `car` (`inward`, `Phase::Boarding`) or out of it (`Phase::Alighting`) by their
/// door, `t` ticks in: while it opens at the door going in (sat in their seat going out), then a
/// step between the two, and in, sat while it shuts.
pub(crate) fn getting(sim: &mut Sim, car: &Car, t: u32, inward: bool) {
    let d = usize::from(sim.car_door).min(DOORS - 1);
    let (door, seat) = (car.door_at(d), car.seat_at(usize::from(sim.car_seat), d));
    let k = (t.saturating_sub(DOOR_OPENS) as f32 / STEP_IN as f32).min(1.0);
    if t < DOOR_OPENS || inward && k >= 1.0 {
        if inward == (t >= DOOR_OPENS) {
            sim.pos = seat;
            sim.yaw = car.yaw;
            sim.pose = crate::content::Pose::Sit;
        }
        return;
    }
    let (a, b) = if inward { (door, seat) } else { (seat, door) };
    sim.pos = [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k];
    sim.yaw = (b[0] - a[0]).atan2(b[1] - a[1]);
    sim.pose = crate::content::Pose::Stand;
}

/// The car driving by that a step from `from` to `to` with `body` would walk into (stepping out
/// of its way is fine).
pub(crate) fn in_the_way(cars: &[Car], body: Body, from: [f32; 2], to: [f32; 2]) -> Option<usize> {
    cars.iter().position(|car| {
        let Some((at, yaw, true, _)) = car.shown else {
            return false;
        };
        // (Already in the way of one waiting for those in its way: they go on out of it, where
        // they're going, as it waits; those not yet in it wait for it, so it gets its gap.)
        if car.waited > 0 && in_way_of(at, yaw, car.len, car.wid, body, from) {
            return false;
        }
        walks_into(at, yaw, car.len, car.wid, body, from, to)
            || car
                .next
                .is_some_and(|(at, yaw)| walks_into(at, yaw, car.len, car.wid, body, from, to))
    })
}

/// Someone has waited a while for a car (at `at`, facing `yaw`) to pass, standing in its way: a
/// step aside (across the way it drives, away from it), where there's ground to stand on and it
/// isn't into another's way (`clear`). None: nowhere to go.
pub(crate) fn make_way(
    at: [f32; 2],
    yaw: f32,
    nav: &crate::path::NavGrid,
    pos: [f32; 2],
    step: f32,
    clear: impl Fn([f32; 2]) -> bool,
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
            nav.tile_free(x, z)
                && near
                && ((x, z) == (fx, fz) || nav.can_step(fx, fz, x, z))
                && clear(*p)
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
    let World {
        cars,
        exits,
        travel,
        ..
    } = w;
    for (c, car) in cars.iter_mut().enumerate() {
        let open = doors[c];
        car.shown = if car.at_home() && !present[c] {
            None
        } else {
            car.pose_on(exits, Some(travel))
                .map(|(p, yaw, moving)| (p, yaw, moving, if moving { 0 } else { open }))
        };
        if !car.shown.is_some_and(|s| s.2) {
            car.next = None;
        }
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
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    car.state = CarState::Parked;
    car.since = tick;
    car.trip = None;
    car.riders.clear();
    car.route = None;
    car.yaw = car.nose_in_on(Some(travel));
    car.settle(exits, Some(travel));
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
pub(crate) fn walk_home(w: &mut World, i: usize, by: Option<(usize, usize)>) {
    if let Some((c, seat)) = by {
        let door = w.cars[c].door(&w.nav(), seat).map(|(d, _)| d);
        if let Some(door) = door {
            let car = &w.cars[c];
            let (at, yaw) = (car.seat_at(seat, door), car.yaw);
            let sim = &mut w.sims[i];
            sim.pos = at;
            sim.yaw = yaw;
            sim.pose = crate::content::Pose::Sit;
            sim.bound_for_car = Some(c as u32);
            sim.car_seat = seat as u8;
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
    // Who drives it off, for a neighbour's lift: only with a seat free (`lift_seat`).
    let driver = setting_off_driver(car, c as u32, &w.briefs, None)
        .filter(|&d| riders(car, c as u32, &w.briefs, d, None) + 1 < SEATS);
    let mut expected = false;
    for i in 0..w.sims.len() {
        let s = &w.sims[i];
        if !s.here() || s.away_until.is_some() || w.cars[c].aboard.contains(&(i as u32)) {
            continue;
        }
        let heading_out = s
            .current()
            .is_some_and(|a| matches!(a.task.kind, TaskKind::Work));
        // The household, still at home; or a neighbour setting off from theirs, for a lift
        // (`lift_seat`, which `may_board` asks).
        let from = if s.household == household {
            Some(plot)
        } else {
            w.households.get(s.household as usize).and_then(|h| h.plot)
        };
        // (A neighbour on their way to it may be on either lot.)
        if !heading_out
            || from.is_none()
            || w.briefs
                .get(i)
                .is_none_or(|b| b.plot.is_some_and(|p| Some(p) != from && p != plot))
        {
            continue;
        }
        if s.bound_for_car == Some(c as u32) {
            expected = true;
            continue;
        }
        // (A neighbour first: is it a lift for them at all? `lift_ok`.)
        if s.household != household {
            let door = from.map(|p| w.plots[p as usize].arrival_point());
            let (car, rels) = (&w.cars[c], &w.relationships);
            let lift = |d| {
                door.is_some_and(|door| {
                    lift_ok(car, &w.households, rels, door, i as u32, s.household, d)
                })
            };
            if !driver.is_some_and(lift) {
                continue;
            }
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
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    car.plot = home;
    car.street = drive.street;
    car.meets = drive.meets();
    car.axis = drive.way();
    car.spot = drive.spot(car.len);
    car.yaw = car.nose_in_on(Some(travel));
    car.drive_spot = None;
    car.settle(exits, Some(travel));
    // (Parked as it was, somewhere else: its tiles are worked out again.)
    w.cars_parked = None;
    refresh(w);
    // (In front of the garage there, if the player built one at the end of that driveway.)
    park_by_garage(w, c);
}

/// Which cars' parking spots have been built over: a wall or a fence across one, a thing on the
/// tiles it takes up.
fn refresh(w: &mut World) {
    let things = things_on(w, &w.cars.iter().map(|c| c.plot).collect::<Vec<_>>());
    let lot = &w.lot;
    for car in &mut w.cars {
        car.blocked = built_over(lot, &things, &car.tiles());
    }
}

/// The tiles the things on `plots` take from walking.
fn things_on(w: &World, plots: &[u32]) -> std::collections::HashSet<(i32, i32)> {
    let mut things: std::collections::HashSet<(i32, i32)> = Default::default();
    for (o, plot) in w.objects.iter().zip(&w.object_plot) {
        if plot.is_some_and(|p| plots.contains(&p)) {
            things.extend(w.blocking_tiles(o));
        }
    }
    things
}

/// Whether a car standing on `tiles` would be built over: off the lot, a thing there (`things`),
/// a diagonal wall, or a wall or a fence between two of them.
fn built_over(
    lot: &crate::lot::Lot,
    things: &std::collections::HashSet<(i32, i32)>,
    tiles: &[(i32, i32)],
) -> bool {
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
    tiles.iter().any(|&(x, z)| {
        !lot.in_bounds(x, z) || things.contains(&(x, z)) || lot.diag_blocks(x, z) || walled(x, z)
    })
}

// ---- Garages the player builds ----------------------------------------------------------

/// How far across its driveway's line a garage's door may be from the car's middle (m) for the
/// car to drive straight in, and how much further on from where it parks (m).
const GARAGE_LINE: f32 = 0.8;
const GARAGE_REACH: f32 = 10.0;

/// Where `car`, parked at `base` (its spot and yaw) on its driveway, parks with a garage the
/// player built on its plot (`Garage::rot`) whose door faces its driveway: one in line with the
/// car, a little further from the road it meets (`GARAGE_REACH`), its door towards that road. It
/// parks in front of the door, nose in. None: no such garage. A town's own garages (no `rot`)
/// change nothing: their cars park where the town put them.
fn garage_spot(w: &World, car: &Car, base: ([f32; 2], f32)) -> Option<([f32; 2], f32)> {
    let (fs, fc) = car
        .posed(base.0, base.1)
        .nose_in_on(Some(&w.travel))
        .sin_cos();
    let reach = PARK_GAP + car.len / 2.0;
    w.garages
        .iter()
        .filter(|g| g.plot == car.plot)
        .find_map(|g| {
            let (m, n) = g.door()?;
            // Its door faces the road: against the way the car faces parked nose in.
            if n[0] * fs + n[1] * fc > -0.9 {
                return None;
            }
            let (dx, dz) = (base.0[0] - m[0], base.0[1] - m[1]);
            let out = dx * n[0] + dz * n[1];
            let across = dx * n[1] - dz * n[0];
            (across.abs() <= GARAGE_LINE && out > car.len / 2.0 && out <= reach + GARAGE_REACH)
                .then(|| {
                    (
                        [m[0] + n[0] * reach, m[1] + n[1] * reach],
                        (-n[0]).atan2(-n[1]),
                    )
                })
        })
}

/// The tiles `car` would take up parked at `spot` facing `yaw`.
pub(crate) fn car_tiles_at(car: &Car, spot: [f32; 2], yaw: f32) -> Vec<(i32, i32)> {
    car.tiles_at(spot, yaw)
}

/// The tiles a car takes up parked on driveway `d` (one as long as the longest car, as
/// `relocate` parks it), whether or not one is there now.
pub(crate) fn drive_tiles(d: &Drive) -> Vec<(i32, i32)> {
    const LEN: f32 = 4.8;
    const WID: f32 = 1.9;
    let Some([ax, _]) = d.way() else {
        return covered_tiles(
            d.x - WID / 2.0,
            d.z - WID / 2.0,
            d.x + WID / 2.0,
            d.z + WID / 2.0,
        );
    };
    let [x, z] = d.spot(LEN);
    // (Along x or along z: a town's driveways run square to its plots.)
    let (hx, hz) = if ax.abs() > 0.5 {
        (LEN / 2.0, WID / 2.0)
    } else {
        (WID / 2.0, LEN / 2.0)
    };
    covered_tiles(x - hx, z - hz, x + hx, z + hz)
}

/// Car `c` parks in front of a garage the player built at the end of its driveway
/// (`garage_spot`), or back where it parked before once that's gone (`Car::drive_spot`): while it
/// stands parked with nobody setting off in it, or while it's away at work (it comes home to its
/// new spot). Not where something stands in front of the garage. True when it moved.
pub(crate) fn park_by_garage(w: &mut World, c: usize) -> bool {
    let car = &w.cars[c];
    let idle = match car.state {
        CarState::Parked => {
            car.driver.is_none()
                && car.aboard.is_empty()
                && !w.sims.iter().any(|s| s.bound_for_car == Some(c as u32))
        }
        CarState::Away => car.trip.is_none(),
        _ => false,
    };
    if !idle {
        return false;
    }
    let base = car
        .drive_spot
        .map_or((car.spot, car.yaw), |[x, z, yaw]| ([x, z], yaw));
    let want = garage_spot(w, car, base).filter(|&(spot, yaw)| {
        let things = things_on(w, &[car.plot]);
        !built_over(&w.lot, &things, &car.posed(spot, yaw).tiles())
    });
    let (spot, yaw, before) = match want {
        Some((spot, yaw)) => (spot, yaw, Some([base.0[0], base.0[1], base.1])),
        None => (base.0, base.1, None),
    };
    if car.spot == spot && car.yaw == yaw && car.drive_spot == before {
        return false;
    }
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    car.spot = spot;
    car.yaw = yaw;
    car.drive_spot = before;
    car.route = None;
    car.settle(exits, Some(travel));
    // (Parked somewhere else: its tiles are worked out again.)
    w.cars_parked = None;
    refresh(w);
    true
}

/// The tiles the garages and the cars parked (`parked`: a bit each) take up, sorted.
fn standing_tiles(w: &World, parked: &[u64]) -> Vec<usize> {
    let mut tiles: Vec<usize> = Vec::new();
    for g in &w.garages {
        for (x, z) in g.tiles() {
            if w.lot.in_bounds(x, z) {
                tiles.push(w.lot.tile_index(x, z));
            }
        }
    }
    let out = w.cars.len().div_ceil(64);
    for (c, car) in w.cars.iter().enumerate() {
        let home = parked[c / 64] & 1 << (c % 64) != 0;
        // (Out on a trip, it stands in its bay as at home.)
        let away = parked[out + c / 64] & 1 << (c % 64) != 0;
        let at = match car.out_at() {
            Some((at, yaw)) if away => car.tiles_at(at, yaw),
            _ if home => car.tiles(),
            _ => continue,
        };
        for (x, z) in at {
            if w.lot.in_bounds(x, z) {
                tiles.push(w.lot.tile_index(x, z));
            }
        }
    }
    tiles.sort_unstable();
    tiles.dedup();
    tiles
}

/// Which cars stand parked, a bit each: at home, then out on a trip.
fn parked_bits(w: &World) -> Vec<u64> {
    let out = w.cars.len().div_ceil(64);
    let mut parked = vec![0u64; 2 * out];
    for c in (0..w.cars.len()).filter(|&c| present(w, c)) {
        parked[c / 64] |= 1 << (c % 64);
    }
    for c in (0..w.cars.len()).filter(|&c| w.cars[c].out_at().is_some()) {
        parked[out + c / 64] |= 1 << (c % 64);
    }
    parked
}

/// A garage was built or taken down on `plot` (or put back by an undo): its car parks by it
/// (`park_by_garage`), and walking changes only on the tiles taken up or freed (`restand`).
pub(crate) fn garages_changed(w: &mut World, plot: u32) {
    for c in 0..w.cars.len() {
        if w.cars[c].plot == plot {
            park_by_garage(w, c);
        }
    }
    let parked = parked_bits(w);
    let tiles = standing_tiles(w, &parked);
    w.cars_parked = Some(parked);
    restand(w, tiles);
}

/// The most separate patches of changed tiles one `restand` notes for walkers (more: the box
/// round them all). The navigation log keeps a tick's patches together (`path::NavLog`), so
/// however many cars come and go in a tick, walkers that last looked a few ticks ago still find
/// what changed there rather than re-planning for anything.
const MOST_PATCHES: usize = 8;

/// Tiles of change this close together (or closer) make one patch.
const PATCH_GAP: i32 = 2;

/// The garages and parked cars now stand on `tiles` (sorted): walking changes only on the tiles
/// taken up or freed, each patch of them noted on its own. (One box round every car in town
/// would have every walker plan again and the walkable regions of most of a big world labelled
/// again, for one car pulling out of a driveway.)
fn restand(w: &mut World, tiles: Vec<usize>) {
    // The tiles in one list and not the other (both are sorted).
    let (old, new) = (&w.car_tiles, &tiles);
    let (mut i, mut j) = (0, 0);
    let (mut freed, mut taken) = (Vec::new(), Vec::new());
    while i < old.len() || j < new.len() {
        match (old.get(i).copied(), new.get(j).copied()) {
            (Some(a), Some(b)) if a == b => {
                i += 1;
                j += 1;
            }
            (Some(a), Some(b)) if a < b => {
                freed.push(a);
                i += 1;
            }
            (Some(a), None) => {
                freed.push(a);
                i += 1;
            }
            (_, Some(b)) => {
                taken.push(b);
                j += 1;
            }
            (None, None) => break,
        }
    }
    w.car_tiles = tiles;
    if freed.is_empty() && taken.is_empty() {
        return;
    }
    for &i in &freed {
        w.blocked[i] = w.travel.ground_blocked(i);
    }
    for &i in &taken {
        w.blocked[i] = true;
    }
    let changed: Vec<(i32, i32)> = freed
        .iter()
        .chain(&taken)
        .map(|&i| w.lot.tile_xz(i))
        .collect();
    for rect in patches(&changed) {
        w.nav_changed_in(rect);
    }
    w.cars_nav = w.nav_version;
}

/// Tiles `tiles` gathered into patches (`[x0, z0, x1, z1)`, in a fixed order): those within
/// `PATCH_GAP` of each other share one; past `MOST_PATCHES`, one box round them all.
fn patches(tiles: &[(i32, i32)]) -> Vec<[i32; 4]> {
    let near = |a: [i32; 4], b: [i32; 4]| {
        a[0] - PATCH_GAP < b[2]
            && b[0] - PATCH_GAP < a[2]
            && a[1] - PATCH_GAP < b[3]
            && b[1] - PATCH_GAP < a[3]
    };
    let mut out: Vec<[i32; 4]> = Vec::new();
    for &(x, z) in tiles {
        let mut r = [x, z, x + 1, z + 1];
        // (Joining two patches can bring a third near: gather until none is.)
        while let Some(k) = out.iter().position(|&o| near(o, r)) {
            r = join(r, out.swap_remove(k));
        }
        out.push(r);
    }
    if out.len() > MOST_PATCHES {
        let all = out.iter().copied().reduce(join).unwrap_or_default();
        return vec![all];
    }
    out
}

/// The tiles parked cars and garages take up: nobody walks through a car or a garage, and nothing is put where one stands
/// (`World::blocked`; things never stand on them, see `refresh`). Walkers find their way again
/// near where that changes (the walkable regions, not the views: `nav_version` only; `restand`).
pub(crate) fn reblock(w: &mut World) {
    // (Nothing to do while the same cars stand parked: a bit each.)
    let parked = parked_bits(w);
    if w.cars_parked.as_ref() == Some(&parked) {
        return;
    }
    // (Garages too, as they stand.)
    let tiles = standing_tiles(w, &parked);
    w.cars_parked = Some(parked);
    restand(w, tiles);
}

impl World {
    /// For tests: the tiles parked cars and garages take up (x, z), sorted as stacked rows are:
    /// nobody walks through them (`reblock`).
    #[doc(hidden)]
    pub fn parked_tiles(&self) -> Vec<(i32, i32)> {
        self.car_tiles
            .iter()
            .map(|&i| self.lot.tile_xz(i))
            .collect()
    }
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
        car.trip = car
            .trip
            .filter(|t| t.driver < sims && (t.plot as usize) < w.plots.len());
        car.riders.retain(|&o| o < sims);
        if car.trip.is_none() {
            car.riders.clear();
        }
        // (On a trip, its driver even while they're out of it, parked out there.)
        let trip = car.trip.map(|t| t.driver);
        car.driver = car
            .driver
            .filter(|d| car.aboard.contains(d) || Some(*d) == trip);
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
    // A cosy town made since cars drive winding roads (its driveways say where they meet them,
    // `Drive::road`): the road network, for its cars only (nobody walks it: `Travel::by_road`).
    // Older ones drive along x as ever.
    if !w.travel.open && w.travel.graph.is_empty() && w.drives.iter().any(|d| d.road.is_some()) {
        w.travel.graph = crate::travel::Graph::from_town(&w.travel.file, &w.meta);
    }
    let World {
        cars,
        exits,
        travel,
        drives,
        ..
    } = &mut *w;
    for car in cars.iter_mut() {
        let drive = drives.iter().find(|d| d.plot == car.plot);
        car.meets = drive.and_then(Drive::meets);
        car.axis = drive.and_then(Drive::way);
        car.settle(exits, Some(travel));
    }
    w.car_tiles.clear();
    w.cars_parked = None;
    refresh(w);
    reblock(w);
    poses(w);
}

// ---- Trips by car (`Trip`) ---------------------------------------------------------------

/// The public parking spot (`parking`) nearest `door` within `TRIP_PARK_REACH`: where it is and
/// which way a car faces parked there. Of `cars`, but car `me`, none parks where another is on a
/// trip to (on its way there, parked there, or still pulling out of it home: they'd stand one on
/// top of the other): with all near taken, none.
fn parking_for(
    travel: &Travel,
    door: [f32; 2],
    cars: &[Car],
    me: Option<usize>,
) -> Option<([f32; 2], f32)> {
    let taken = |at: [f32; 2]| {
        cars.iter().enumerate().any(|(c, car)| {
            let there = || match car.state {
                CarState::Arriving => car
                    .shown
                    .is_some_and(|s| crate::travel::dist(s.0, at) < car.len),
                _ => true,
            };
            Some(c) != me
                && car
                    .trip
                    .is_some_and(|t| crate::travel::dist(t.at, at) < 0.5)
                && there()
        })
    };
    (0..travel.file.parking.len())
        .filter_map(|k| parked_at(travel, k))
        .map(|(at, yaw, _)| (crate::travel::dist(at, door), at, yaw))
        .filter(|&(d, at, _)| d <= TRIP_PARK_REACH && !taken(at))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, at, yaw)| (at, yaw))
}

/// Where a car parked at public parking spot `k` (`OpenWorld::parking`) stands, which way it
/// faces and whether it's a bay off the road (nose in, facing away from it: `parking_bay`; else
/// as the spot, on the road).
pub(crate) fn parked_at(travel: &Travel, k: usize) -> Option<([f32; 2], f32, bool)> {
    let p = travel.file.parking.get(k)?;
    let at = [p.x, p.z];
    at.iter().all(|v| v.is_finite()).then_some(())?;
    Some(match parking_bay(travel, at) {
        Some((_, _, [x, z])) => (at, (0.0 - x).atan2(0.0 - z), true),
        None => (at, p.rot, false),
    })
}

/// Whether there's public parking near `door` for a trip by car (`TRIP_PARK_REACH`), taken or not.
pub(crate) fn parks_near(travel: &Travel, door: [f32; 2]) -> bool {
    parking_for(travel, door, &[], None).is_some()
}

/// `can_drive`, in a tick (from its `Ctx`).
pub(crate) fn could_drive(ctx: &crate::world::Ctx, sim: &Sim) -> bool {
    can_drive(ctx.travel, ctx.households, ctx.briefs, ctx.cars, sim)
}

/// Whether `sim` could drive somewhere now (`trip_car` without the place): an open-world town
/// with parking, a grown-up owner at home with the household's car parked there. From the
/// world's parts, so it's asked outside a tick too (`World::visit_fits_now`); in one, `could_drive`.
pub(crate) fn can_drive(
    travel: &Travel,
    households: &[crate::world::Household],
    briefs: &[crate::world::SimBrief],
    cars: &[Car],
    sim: &Sim,
) -> bool {
    let Some(home) = households.get(sim.household as usize).and_then(|h| h.plot) else {
        return false;
    };
    travel.open
        && !travel.file.parking.is_empty()
        && sim.visiting.is_none()
        && briefs
            .get(sim.id as usize)
            .is_some_and(|b| b.adult && b.plot == Some(home))
        && cars.iter().any(|car| {
            car.household == sim.household
                && car.plot == home
                && car.state == CarState::Parked
                && car.trip.is_none()
                && !car.blocked
                && car.owners.contains(&sim.id)
        })
}

/// Whether `car`'s driveway has a way by road to `at`.
fn drives_to(car: &Car, travel: &Travel, at: [f32; 2]) -> bool {
    let g = &travel.graph;
    let (Some(m), Some(&p)) = (
        car.mouth(travel),
        g.near(at, ROAD_REACH, 1, Mode::Car).first(),
    ) else {
        return false;
    };
    g.path(m, p, Mode::Car, travel.country_speed).is_some()
}

/// The car `sim` drives to plot `plot` in an open-world town, and the door they get in by and
/// where they stand for it: the place is out of walking range (`autonomy.reach`), there's
/// public parking near it and a way there by road, and they're a grown-up owner at home with
/// the household's car parked there and nobody else heading for it. None: they walk.
pub(crate) fn trip_car(
    ctx: &crate::world::Ctx,
    sim: &Sim,
    plot: u32,
) -> Option<(u32, usize, [f32; 2])> {
    let travel = ctx.travel;
    if !travel.open || travel.file.parking.is_empty() || sim.visiting.is_some() {
        return None;
    }
    let me = ctx.briefs.get(sim.id as usize)?;
    let home = ctx.households.get(sim.household as usize)?.plot?;
    if !me.adult || me.plot != Some(home) {
        return None;
    }
    let door = ctx.plots.get(plot as usize)?.arrival_point();
    if crate::travel::dist(sim.pos, door) <= ctx.content.autonomy.reach {
        return None;
    }
    let (at, _) = parking_for(travel, door, ctx.cars, None)?;
    let (c, car) = ctx.cars.iter().enumerate().find(|(_, car)| {
        car.household == sim.household
            && car.plot == home
            && car.state == CarState::Parked
            && car.trip.is_none()
            && !car.blocked
            && !car.no_way
            && car.owners.contains(&sim.id)
    })?;
    let c = c as u32;
    if ctx.briefs.iter().any(|b| b.bound_for_car == Some(c)) || !drives_to(car, travel, at) {
        return None;
    }
    let (d, stand) = car.door(&ctx.nav, 0)?;
    Some((c, d, stand))
}

/// Whether `sim` can set off in car `c` now (at its door, as `trip_car` found it).
pub(crate) fn may_set_off(cars: &[Car], c: u32, sim: u32) -> bool {
    cars.get(c as usize).is_some_and(|car| {
        car.state == CarState::Parked
            && car.trip.is_none()
            && !car.blocked
            && car.owners.contains(&sim)
    })
}

/// The car `sim` is at the wheel of, on a trip there or back (`Leaving`, `Arriving`), or sat in
/// on one (driving or riding, or waiting in it for the others).
pub(crate) fn driving(cars: &[Car], sim: u32) -> Option<u32> {
    cars.iter()
        .position(|car| {
            (car.trip.is_some_and(|t| t.driver == sim)
                && matches!(car.state, CarState::Leaving | CarState::Arriving))
                || (car.trip.is_some() && car.aboard.contains(&sim))
        })
        .map(|c| c as u32)
}

/// Whether car `c` is parked out on a trip `sim` drives.
pub(crate) fn parked_out(cars: &[Car], c: u32, sim: u32) -> bool {
    cars.get(c as usize)
        .is_some_and(|car| car.state == CarState::Out && car.trip.is_some_and(|t| t.driver == sim))
}

/// `sim`'s car parked out on a trip, the door they get in by and where they stand for it: where
/// they go home from.
pub(crate) fn car_out_for(ctx: &crate::world::Ctx, sim: &Sim) -> Option<(u32, usize, [f32; 2])> {
    let (c, car) = ctx
        .cars
        .iter()
        .enumerate()
        .find(|(c, _)| parked_out(ctx.cars, *c as u32, sim.id))?;
    let trip = car.trip?;
    let (d, stand) = car.posed(trip.at, trip.yaw).door(&ctx.nav, 0)?;
    Some((c as u32, d, stand))
}

/// `i` gets in at car `c`'s door and drives off to plot `plot` (`Transition::Drive`): out of the
/// driveway, along the roads to the parking spot near it. Gone or taken since: they walk.
pub(crate) fn start_trip(w: &mut World, i: usize, c: u32, plot: u32, directed: bool) {
    let visit = Task {
        kind: TaskKind::Visit { plot },
        directed,
        routine: false,
    };
    let door = w.plots.get(plot as usize).map(|p| p.arrival_point());
    let park = door.and_then(|d| parking_for(&w.travel, d, &w.cars, Some(c as usize)));
    let (Some((at, yaw)), true) = (park, may_set_off(&w.cars, c, i as u32)) else {
        w.sims[i].bound_for_car = None;
        w.sims[i].queue.push_front(visit);
        return;
    };
    let tick = w.tick;
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c as usize];
    car.trip = Some(Trip {
        driver: i as u32,
        plot,
        directed,
        at,
        yaw,
    });
    car.route = None;
    if Route::on_roads(car, exits, travel, false).is_none() {
        car.trip = None;
        car.settle(exits, Some(travel));
        w.sims[i].bound_for_car = None;
        w.sims[i].queue.push_front(visit);
        return;
    }
    car.since = tick;
    car.progress = 0.0;
    car.driver = Some(i as u32);
    car.aboard = vec![i as u32];
    car.riders.clear();
    car.done.clear();
    // Those of the household setting off for somewhere it parks near now come along: it waits
    // for them in the driveway (`CarState::Boarding`).
    car.state = CarState::Boarding;
    let sim = &mut w.sims[i];
    sim.bound_for_car = Some(c);
    sim.current = Some(crate::world::Activity {
        task: visit,
        phase: Phase::Driving,
        tags: Default::default(),
    });
    if !call_trip_riders(w, c as usize) {
        w.cars[c as usize].state = CarState::Leaving;
    }
    seat_party(w, c as usize);
}

/// `i` gets into car `c` parked out on their trip and drives home (`Transition::DriveHome`):
/// at once, or once those who came with them are in (`call_party_home`).
pub(crate) fn drive_home(w: &mut World, i: usize, c: u32) {
    if !parked_out(&w.cars, c, i as u32) {
        w.sims[i].bound_for_car = None;
        walk_home(w, i, None);
        return;
    }
    let tick = w.tick;
    let car = &mut w.cars[c as usize];
    car.since = tick;
    car.route = None;
    car.driver = Some(i as u32);
    // (At the wheel, before any of the others in already.)
    car.aboard.retain(|&a| a != i as u32);
    car.aboard.insert(0, i as u32);
    let sim = &mut w.sims[i];
    sim.bound_for_car = Some(c);
    sim.current = Some(crate::world::Activity {
        task: Task {
            kind: TaskKind::GoHome,
            directed: false,
            routine: false,
        },
        phase: Phase::Driving,
        tags: Default::default(),
    });
    if !call_party_home(w, c as usize) {
        set_off_home(w, c as usize);
    }
    seat_party(w, c as usize);
}

/// Car `c`, parked out on a trip with its driver in, sets off home with whoever is in it (the
/// rest of those who came make their own way).
fn set_off_home(w: &mut World, c: usize) {
    let tick = w.tick;
    let car = &mut w.cars[c];
    car.state = CarState::Arriving;
    car.since = tick;
    car.progress = 0.0;
    car.route = None;
    let aboard = car.aboard.clone();
    car.riders.retain(|r| aboard.contains(r));
}

/// The door the one in seat `seat` gets in by on their own side (as `Car::door` tries first).
fn own_door(seat: usize) -> usize {
    [0, 1, 3, 2][seat.min(DOORS - 1)]
}

/// Those in car `c` on a trip sat in their seats where it is now: the driver at the wheel, the
/// others a seat each in the order they got in.
fn seat_party(w: &mut World, c: usize) {
    let World {
        cars,
        exits,
        travel,
        sims,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    if car.trip.is_none() {
        return;
    }
    let pose = car.pose_on(exits, Some(travel));
    let Some((at, yaw, _)) = pose.or(car.trip.map(|t| (t.at, t.yaw, false))) else {
        return;
    };
    let posed = car.posed(at, yaw);
    for (seat, &a) in car.aboard.iter().enumerate() {
        if let Some(s) = sims.get_mut(a as usize) {
            s.pos = posed.seat_at(seat, own_door(seat));
            s.yaw = yaw;
            s.pose = crate::content::Pose::Sit;
        }
    }
}

/// Once a tick for a car on a trip: on along the roads, parked out there, or back home.
fn trip_update(w: &mut World, c: usize) {
    let Some(trip) = w.cars[c].trip else { return };
    let d = trip.driver as usize;
    // At the wheel; or (just loaded, or not their turn yet) about to be.
    let phase = w.sims[d].current().map(|a| a.phase.clone());
    let at_wheel = matches!(phase, None | Some(Phase::Driving));
    // (Those riding with them who left town or were called away get out.)
    let gone: Vec<u32> = w.cars[c]
        .aboard
        .iter()
        .copied()
        .filter(|&a| {
            a != trip.driver
                && w.sims.get(a as usize).is_none_or(|s| {
                    !s.here()
                        || !s
                            .current()
                            .is_some_and(|a| matches!(a.phase, Phase::Driving))
                })
        })
        .collect();
    if !gone.is_empty() {
        w.cars[c].aboard.retain(|a| !gone.contains(a));
        w.cars[c].riders.retain(|a| !gone.contains(a));
    }
    match w.cars[c].state {
        // In the driveway, the driver in: waiting for the others coming along.
        CarState::Boarding => {
            let waited = w.tick - w.cars[c].since;
            if !at_wheel || !w.sims[d].here() {
                // (The driver called away after all: the trip is off.)
                let aboard = std::mem::take(&mut w.cars[c].aboard);
                for a in aboard {
                    let_out(w, a as usize);
                }
                let World {
                    cars,
                    exits,
                    travel,
                    ..
                } = &mut *w;
                let car = &mut cars[c];
                car.state = CarState::Parked;
                car.trip = None;
                car.riders.clear();
                car.driver = None;
                car.route = None;
                car.settle(exits, Some(travel));
                return;
            }
            let expected = call_trip_riders(w, c);
            if (!expected && waited >= BOARD_MIN_TICKS) || waited >= TRIP_WAIT_TICKS {
                let tick = w.tick;
                let car = &mut w.cars[c];
                car.state = CarState::Leaving;
                car.since = tick;
                car.progress = 0.0;
            }
            seat_party(w, c);
        }
        CarState::Leaving => {
            let ticks = route_ticks(w, c, false);
            if at_wheel {
                drive_on(w, c);
                seat_party(w, c);
            } else {
                // (Called away at the wheel, as they can't be: the car is there.)
                w.cars[c].progress = ticks;
            }
            if w.cars[c].progress >= ticks {
                park_out(w, c);
            }
        }
        CarState::Arriving => {
            let ticks = route_ticks(w, c, true);
            if at_wheel {
                drive_on(w, c);
                seat_party(w, c);
            } else {
                w.cars[c].progress = ticks;
            }
            if w.cars[c].progress >= ticks {
                let car = &mut w.cars[c];
                // Everyone in it gets out at home, the driver first (`arrive`).
                car.done = car
                    .aboard
                    .iter()
                    .copied()
                    .filter(|&a| at_wheel || a != d as u32)
                    .collect();
                car.driver = Some(d as u32);
                arrive(w, c);
            }
        }
        CarState::Out => {
            // (The driver, in it to go home, called away after all, to work say: out of it. They
            // come back to it going home, or it goes home when they get there.)
            if !at_wheel && w.cars[c].aboard.contains(&(d as u32)) {
                w.cars[c].aboard.retain(|&a| a != d as u32);
                let sim = &mut w.sims[d];
                if sim.bound_for_car == Some(c as u32) {
                    sim.bound_for_car = None;
                }
                sim.pose = crate::content::Pose::Stand;
            }
            let car = &w.cars[c];
            let driver_in = at_wheel && car.aboard.contains(&(d as u32));
            if driver_in {
                // The driver in to go home: once the others who came are in too (or have been
                // waited for long enough).
                let waited = w.tick - w.cars[c].since;
                let expected = call_party_home(w, c);
                if !expected || waited >= TRIP_WAIT_TICKS {
                    set_off_home(w, c);
                }
                seat_party(w, c);
                return;
            }
            // One of them in it, or on their way to it, to go home: the rest go home too.
            let going = !car.aboard.is_empty()
                || car
                    .riders
                    .iter()
                    .any(|&r| w.sims[r as usize].bound_for_car == Some(c as u32));
            // Unless the driver can't come (at work, out of town): those in it get out and make
            // their own way, and so do the rest of them. Or the driver hasn't come for as long as
            // a driver waits for them (`TRIP_WAIT_TICKS` since the first of them got in): those in
            // it get out, and the rest may still ride home with the driver.
            if going {
                let driver_away = w.sims[d].away_until.is_some() || !w.sims[d].here();
                let waited = !car.aboard.is_empty() && w.tick - car.since >= TRIP_WAIT_TICKS;
                if driver_away || waited {
                    let aboard = std::mem::take(&mut w.cars[c].aboard);
                    for &a in &aboard {
                        let_out(w, a as usize);
                    }
                    if driver_away {
                        w.cars[c].riders.clear();
                    } else {
                        w.cars[c].riders.retain(|r| !aboard.contains(r));
                    }
                } else {
                    call_party_home(w, c);
                }
            }
            seat_party(w, c);
            // Home some other way (and not on their way back to it): it's home too.
            let home = w
                .households
                .get(w.sims[d].household as usize)
                .and_then(|h| h.plot);
            let there = home.is_some() && w.briefs.get(d).and_then(|b| b.plot) == home;
            if there && w.sims[d].bound_for_car != Some(c as u32) {
                let car = &mut w.cars[c];
                car.done = car.aboard.clone();
                car.aboard.clear();
                arrive(w, c);
            }
        }
        // (A trip on a car that isn't on its way, out or back: done with.)
        CarState::Parked | CarState::Away => {
            let aboard = std::mem::take(&mut w.cars[c].aboard);
            for a in aboard {
                let_out(w, a as usize);
            }
            let World {
                cars,
                exits,
                travel,
                ..
            } = &mut *w;
            cars[c].trip = None;
            cars[c].riders.clear();
            cars[c].route = None;
            cars[c].settle(exits, Some(travel));
        }
    }
}

/// `i`, sat in a car on a trip that won't take them now, gets out where they are and goes on
/// their own way (on foot).
fn let_out(w: &mut World, i: usize) {
    let Some(sim) = w.sims.get_mut(i) else {
        return;
    };
    if let Some(a) = sim.current.take_if(|a| matches!(a.phase, Phase::Driving)) {
        sim.queue.push_front(a.task);
    }
    sim.bound_for_car = None;
    sim.pose = crate::content::Pose::Stand;
}

/// Whether household member `j` sets off now for somewhere car `c`, about to leave on a trip,
/// parks near (`Trip::at`): at home (or still close by), on their way there on foot, or about to
/// be. The place they're off to.
fn sets_off_with(w: &World, j: usize, c: usize) -> Option<(u32, bool)> {
    let car = &w.cars[c];
    let trip = car.trip?;
    let s = &w.sims[j];
    if s.household != car.household
        || j as u32 == trip.driver
        || !s.here()
        || s.away_until.is_some()
        || car.aboard.contains(&(j as u32))
        || w.briefs.get(j).is_none_or(|b| b.baby)
    {
        return None;
    }
    let near = w.briefs.get(j).and_then(|b| b.plot) == Some(car.plot)
        || crate::travel::dist(s.pos, car.spot) < CALL_REACH;
    let task = match s.current() {
        Some(a) if matches!(a.phase, Phase::Routing { .. }) => a.task,
        Some(_) => return None,
        None => *s.queue.front()?,
    };
    let TaskKind::Visit { plot } = task.kind else {
        return None;
    };
    // (A guest there already as they set off from home: `Transition::Visit`; not from a visit
    // elsewhere.)
    if s.visiting.is_some_and(|v| v.plot != plot) {
        return None;
    }
    let door = w.plots.get(plot as usize)?.arrival_point();
    let far = crate::travel::dist(s.pos, door) > w.content.autonomy.reach;
    let parks = parking_for(&w.travel, door, &w.cars, Some(c)).is_some_and(|(at, _)| at == trip.at);
    (near && far && parks).then_some((plot, task.directed))
}

/// While car `c` waits in the driveway on a trip: the rest of the household setting off now for
/// somewhere it parks near is called to it (those on their way on foot turn round, while they're
/// still close by), as many as it has seats. Whether anyone is on their way to it.
fn call_trip_riders(w: &mut World, c: usize) -> bool {
    let me = c as u32;
    let mut coming = 0;
    for j in 0..w.sims.len() {
        let s = &w.sims[j];
        if s.bound_for_car == Some(me)
            && !w.cars[c].aboard.contains(&(j as u32))
            && s.here()
            && s.current()
                .is_some_and(|a| matches!(a.task.kind, TaskKind::Visit { .. }))
        {
            coming += 1;
        }
    }
    for j in 0..w.sims.len() {
        if w.cars[c].aboard.len() + coming >= SEATS {
            break;
        }
        if w.sims[j].bound_for_car == Some(me) || sets_off_with(w, j, c).is_none() {
            continue;
        }
        // To the car instead (`trip_ride` picks it now).
        let sim = &mut w.sims[j];
        if let Some(a) = sim.current.take() {
            sim.queue.push_front(a.task);
        }
        coming += 1;
    }
    coming > 0
}

/// Car `c` parked out on a trip, one of those who came in it (or on their way to it) to go home:
/// the others who came go home too (with it: `ride_home_for`), leaving what they chose to do.
/// Whether any of them is still to get in.
fn call_party_home(w: &mut World, c: usize) -> bool {
    let me = c as u32;
    let Some(trip) = w.cars[c].trip else {
        return false;
    };
    let party: Vec<u32> = std::iter::once(trip.driver)
        .chain(w.cars[c].riders.iter().copied())
        .collect();
    let home = w
        .households
        .get(w.cars[c].household as usize)
        .and_then(|h| h.plot);
    let mut expected = false;
    for j in party {
        if w.cars[c].aboard.contains(&j) {
            continue;
        }
        let i = j as usize;
        // (Home some other way already: not coming.)
        if j != trip.driver && home.is_some() && w.briefs.get(i).and_then(|b| b.plot) == home {
            w.cars[c].riders.retain(|&r| r != j);
            continue;
        }
        let World {
            sims,
            content,
            objects,
            ..
        } = &mut *w;
        let Some(s) = sims
            .get_mut(i)
            .filter(|s| s.here() && s.away_until.is_none())
        else {
            continue;
        };
        let homeward = |t: &Task| matches!(t.kind, TaskKind::GoHome);
        let going = s.current().is_some_and(|a| homeward(&a.task)) || s.queue.iter().any(homeward);
        if j != trip.driver {
            expected = true;
        }
        if s.bound_for_car == Some(me) || going {
            continue;
        }
        // What they chose for themselves ends; what they were told to do comes first.
        if s.current.as_ref().is_some_and(|a| !a.task.directed) {
            crate::world::end_activity(s, content, objects);
        }
        s.queue.push_front(Task {
            kind: TaskKind::GoHome,
            directed: false,
            routine: false,
        });
    }
    expected
}

/// The car `sim` rides in to plot `plot` in an open-world town, their seat in it, and the door
/// they get in by and where they stand for it: their household's car waiting in the driveway on
/// a trip to park near there (`CarState::Boarding`), with a seat for them, the place out of
/// walking range and they at home (or still close by). None: they walk (or drive: `trip_car`).
pub(crate) fn trip_ride(
    ctx: &crate::world::Ctx,
    sim: &Sim,
    plot: u32,
) -> Option<(u32, usize, usize, [f32; 2])> {
    let travel = ctx.travel;
    // (A guest there already as they set off from home: `Transition::Visit`.)
    if !travel.open
        || travel.file.parking.is_empty()
        || sim.visiting.is_some_and(|v| v.plot != plot)
    {
        return None;
    }
    let me = ctx.briefs.get(sim.id as usize)?;
    let home = ctx.households.get(sim.household as usize)?.plot?;
    let door = ctx.plots.get(plot as usize)?.arrival_point();
    if me.baby || crate::travel::dist(sim.pos, door) <= ctx.content.autonomy.reach {
        return None;
    }
    let (c, car) = ctx.cars.iter().enumerate().find(|&(c, car)| {
        car.household == sim.household
            && car.plot == home
            && car.state == CarState::Boarding
            && car.trip.is_some_and(|t| {
                t.driver != sim.id
                    && parking_for(travel, door, ctx.cars, Some(c))
                        .is_some_and(|(at, _)| at == t.at)
            })
            && !car.blocked
            && !car.aboard.contains(&sim.id)
    })?;
    let near = me.plot == Some(home) || crate::travel::dist(sim.pos, car.spot) < CALL_REACH;
    let c = c as u32;
    // A seat of their own: after those in it and on their way to it.
    let seat = car.aboard.len()
        + ctx
            .briefs
            .iter()
            .enumerate()
            .filter(|(j, b)| {
                *j as u32 != sim.id && b.trip_car == Some(c) && !car.aboard.contains(&(*j as u32))
            })
            .count();
    if !near || seat >= SEATS {
        return None;
    }
    let (d, stand) = car.door(&ctx.nav, seat)?;
    Some((c, seat, d, stand))
}

/// Whether `sim` can get into car `c` now to ride along on its trip: out (`home` false), it
/// waits in the driveway for them; home, it's parked out there and they came in it.
pub(crate) fn may_ride(cars: &[Car], c: u32, sim: u32, home: bool) -> bool {
    cars.get(c as usize).is_some_and(|car| {
        car.trip.is_some_and(|t| t.driver != sim)
            && car.aboard.len() < SEATS
            && !car.aboard.contains(&sim)
            && if home {
                car.state == CarState::Out && car.riders.contains(&sim)
            } else {
                car.state == CarState::Boarding
            }
    })
}

/// The car `sim` came in on a trip, parked out there, to ride home in: the car, their seat,
/// the door they get in by and where they stand for it.
pub(crate) fn ride_home_for(
    ctx: &crate::world::Ctx,
    sim: &Sim,
) -> Option<(u32, usize, usize, [f32; 2])> {
    let (c, car) = ctx
        .cars
        .iter()
        .enumerate()
        .find(|(c, _)| may_ride(ctx.cars, *c as u32, sim.id, true))?;
    let trip = car.trip?;
    let seat = 1 + car.riders.iter().position(|&r| r == sim.id)?;
    let (d, stand) = car
        .posed(trip.at, trip.yaw)
        .door(&ctx.nav, seat.min(SEATS - 1))?;
    Some((c as u32, seat, d, stand))
}

/// `i` at car `c`'s door gets in to ride along (`Transition::Ride`): off to `plot` with it, or
/// (None) home in it. Gone since, or full: they make their own way.
pub(crate) fn ride(w: &mut World, i: usize, c: u32, plot: Option<u32>, directed: bool) {
    let task = Task {
        kind: plot.map_or(TaskKind::GoHome, |plot| TaskKind::Visit { plot }),
        directed,
        routine: false,
    };
    let me = i as u32;
    if !may_ride(&w.cars, c, me, plot.is_none()) {
        w.sims[i].bound_for_car = None;
        w.sims[i].queue.push_front(task);
        return;
    }
    let tick = w.tick;
    let car = &mut w.cars[c as usize];
    // (Parked out there, the first of them in to go home: how long they've waited for the driver.)
    if plot.is_none() && car.aboard.is_empty() {
        car.since = tick;
    }
    car.aboard.push(me);
    if plot.is_some() && !car.riders.contains(&me) {
        car.riders.push(me);
    }
    let sim = &mut w.sims[i];
    if plot.is_none() {
        sim.visiting = None;
    }
    sim.bound_for_car = Some(c);
    sim.current = Some(crate::world::Activity {
        task,
        phase: Phase::Driving,
        tags: Default::default(),
    });
    seat_party(w, c as usize);
}

/// Car `c` is at its trip's parking spot: parked there (`CarState::Out`), and its driver gets
/// out beside it and walks on to the place.
fn park_out(w: &mut World, c: usize) {
    let Some(trip) = w.cars[c].trip else { return };
    let tick = w.tick;
    let at_wheel = matches!(
        w.sims[trip.driver as usize].current().map(|a| &a.phase),
        None | Some(Phase::Driving)
    );
    let (at, yaw) = {
        let World {
            cars,
            exits,
            travel,
            ..
        } = &mut *w;
        let route = cars[c].route(exits, Some(travel), false);
        let (p, yaw, _) = route.at(route.ticks());
        (p, yaw)
    };
    let car = &mut w.cars[c];
    car.state = CarState::Out;
    car.since = tick;
    car.route = None;
    let aboard = std::mem::take(&mut car.aboard);
    car.trip = Some(Trip { at, yaw, ..trip });
    let posed = car.posed(at, yaw);
    // Those who came along get out by their doors and walk on to where they're going (they come
    // home with it: `Car::riders`).
    let riders: Vec<(usize, u32)> = aboard
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, a)| a != trip.driver)
        .collect();
    w.cars[c].riders = riders.iter().map(|&(_, a)| a).collect();
    for (seat, a) in riders {
        let door = posed.door(&w.nav(), seat).map(|(_, p)| p);
        let sim = &mut w.sims[a as usize];
        sim.pos = door.unwrap_or(posed.door_at(own_door(seat)));
        sim.yaw = yaw;
        sim.pose = crate::content::Pose::Stand;
        sim.bound_for_car = None;
        if let Some(act) = sim.current.take() {
            sim.queue.push_front(act.task);
        }
    }
    if !at_wheel {
        return;
    }
    let door = posed.door(&w.nav(), 0).map(|(_, p)| p);
    let sim = &mut w.sims[trip.driver as usize];
    sim.pos = door.unwrap_or(posed.door_at(0));
    sim.yaw = yaw;
    sim.pose = crate::content::Pose::Stand;
    sim.bound_for_car = None;
    sim.current = None;
    sim.queue.push_front(Task {
        kind: TaskKind::Visit { plot: trip.plot },
        directed: trip.directed,
        routine: false,
    });
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

    /// Ada's house with her car, and Cy's next door without one: both at the same office.
    fn neighbours(bond: &str) -> World {
        let content = CONTENT.replace(
            r#""economy""#,
            r#""bondPresets":{"friends":{"friendship":60},"strangers":{"friendship":0}},"economy""#,
        );
        let town = format!(
            r#"{{"width":60,"depth":14,"exits":[[0.5,9.5],[59.5,9.5]],
            "plots":[{{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}},
                     {{"name":"Next","x":40,"z":0,"w":16,"d":8,"entry":[45.5,7.5]}}],
            "households":[{{"name":"Ones","plot":0}},{{"name":"Twos","plot":1}}],
            "sims":[{{"name":"Ada","household":0,"job":{{"career":"office"}},"x":25.5,"z":3.5}},
                    {{"name":"Cy","household":1,"job":{{"career":"office"}},"x":45.5,"z":3.5}}],
            "relationships":[{{"a":0,"b":1,"preset":"{bond}"}}],
            "cars":[{{"plot":0,"x":34.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":2}}]}}"#
        );
        World::from_json(&content, &town, 1).unwrap()
    }

    #[test]
    fn a_friend_next_door_gets_a_lift_to_work_and_back() {
        let mut w = neighbours("friends");
        w.autonomy = false;
        until(&mut w, 2, 10.0);
        let mut aboard = w.cars[0].aboard.clone();
        aboard.sort();
        assert_eq!(aboard, vec![0, 1], "Cy went in Ada's car");
        assert!(w.sims.iter().all(|s| s.away_until.is_some()));
        until(&mut w, 2, 15.0);
        assert_eq!(w.cars[0].state, CarState::Parked, "home again");
        let (x, z) = w.sims[1].tile();
        assert_eq!(w.plot_at(x, z), Some(1), "Cy walked home from next door");
        assert!(w.sims[1].away_until.is_none());
    }

    #[test]
    fn a_neighbour_who_is_no_friend_walks() {
        let mut w = neighbours("strangers");
        w.autonomy = false;
        until(&mut w, 2, 10.0);
        assert_eq!(w.cars[0].aboard, vec![0], "only Ada in her car");
        assert!(w.sims[1].away_until.is_some(), "Cy got to work on foot");
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
    fn someone_crossing_in_front_of_a_waiting_car_gets_across() {
        // (As at Crown Quarter: walking across the ground in front of a car that waits for them,
        // they go on across, not stepped back where they came from, and it goes after.)
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        w.autonomy = false;
        w.cars[0].state = CarState::Leaving;
        w.cars[0].progress = 0.0;
        w.cars[0].driver = Some(0);
        w.cars[0].aboard = vec![0];
        w.sims[0].away_until = Some(w.tick + 100_000);
        w.sims[1].pos = [31.5, 6.5];
        // (It has waited a while already, for others before them.)
        w.tick_once();
        w.cars[0].waited = 4 * MAKE_WAY_TICKS;
        let _ = w.apply(crate::Command::MoveTo {
            sim: 1,
            x: 37.0,
            z: 6.5,
        });
        let mut across = None;
        for t in 0..1200 {
            w.tick_once();
            if t < 400 {
                w.cars[0].waited = w.cars[0].waited.max(4 * MAKE_WAY_TICKS);
            }
            if across.is_none() && w.sims[1].pos[0] > 36.0 {
                across = Some(t);
            }
        }
        assert!(
            across.is_some_and(|t| t < 400),
            "Bo never got across: {across:?} {:?}",
            w.sims[1].pos
        );
        assert!(w.cars[0].progress > 0.0, "and the car went after");
    }

    #[test]
    fn nobody_walks_into_a_car_driving_by() {
        let mut c = car([10.0, 10.0], 0.0, 20.0);
        c.shown = Some(([10.0, 10.0], 0.0, true, 0));
        let cars = [c];
        let on_foot = Body::WALKER;
        assert_eq!(
            in_the_way(&cars, on_foot, [12.0, 10.0], [11.0, 10.0]),
            Some(0),
            "into its side"
        );
        assert_eq!(
            in_the_way(&cars, on_foot, [10.8, 10.0], [11.5, 10.0]),
            None,
            "out of its way"
        );
        assert_eq!(
            in_the_way(&cars, on_foot, [14.0, 10.0], [13.5, 10.0]),
            None,
            "well clear"
        );
        // (A bike's front wheel well in front of its rider.)
        let bike = Body {
            ahead: 1.2,
            behind: 0.6,
            side: 0.3,
            yaw: 0.0,
        };
        assert_eq!(
            in_the_way(&cars, on_foot, [12.4, 10.0], [12.0, 10.0]),
            None,
            "on foot, still clear"
        );
        assert_eq!(
            in_the_way(&cars, bike, [12.4, 10.0], [12.0, 10.0]),
            Some(0),
            "but not on a bike"
        );
        // Nor into the ground just in front of it (it faces +z, its front at z = 12.2): stopped
        // there, they'd hold it up. Out of there is fine.
        assert_eq!(
            in_the_way(&cars, on_foot, [10.0, 15.5], [10.0, 14.0]),
            Some(0),
            "in front of it"
        );
        assert_eq!(
            in_the_way(&cars, on_foot, [10.0, 13.5], [10.0, 14.0]),
            None,
            "out from in front of it"
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
        assert!((last[1] - (20.0 + TIGHT_LANE)).abs() < 1e-3, "in its lane");
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

    #[test]
    fn a_car_faces_within_a_half_turn_either_way_backing_out_too() {
        // (Its riders sit facing as it does: a yaw past a full turn was a resident facing 6.3.)
        let exits = [[0.5, 20.0], [99.5, 20.0]];
        for yaw in [std::f32::consts::PI, -std::f32::consts::FRAC_PI_2, 0.0, 3.0] {
            let c = car([30.0, 10.0], yaw, 20.0);
            for home in [false, true] {
                let r = Route::new(&c, &exits, home);
                let mut t = 0.0;
                while t <= r.ticks() {
                    let (_, a, _) = r.at(t);
                    assert!(
                        (-std::f32::consts::PI..=std::f32::consts::PI).contains(&a),
                        "parked at {yaw}, home {home}: {a} at {t}"
                    );
                    t += 0.25;
                }
            }
        }
    }
}
