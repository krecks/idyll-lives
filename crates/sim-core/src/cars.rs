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
//! (`Travel::graph`: every town has one, `settle_all`) it goes along its driveway (`Drive::way`),
//! turns onto the road where the driveway meets it and along the lanes (`Route::on_roads`); a car
//! whose driveway comes alongside its road rather than onto it (`OBLIQUE`), or has no way out by
//! road, stays parked (`Car::no_way`). Older towns drive straight along x (`Route::legacy`).
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
//!
//! **Out of sight** (`sight`; the owner's call for big towns). All that is only where the player
//! can see it: a car takes turns with the others, keeps clear of them and waits for anyone in its
//! way only while it's in the seen area, what the renderer draws (`World::set_watch`: the
//! camera's streamed chunks) and `SEEN_MARGIN` round it (`Sight::Seen`). Out of it a car drives on
//! through the others, through junctions and past people, claiming nothing (`Sight::Ghost`):
//! nobody draws it there, and a hold-up nobody can see is gone at once. Those in sight never wait
//! for one driving out of it. One driving into the seen area comes under the rules some 20 m short
//! of it (`COMING_IN`), and short of any claim of its reaching into it, once clear of those in it:
//! till then it waits outside (`Sight::Wait`). One the seen area came to (the camera moved, nobody
//! has seen it yet) on top of or touching one in it is put back along its way, or on, to the
//! nearest place clear of them (`moved_clear`) and comes into sight there: no car is drawn on top
//! of another, and none the player sees is hidden (`hidden`, `snapshot::car`, leaves out those out
//! of sight and any touching one). Without a watch (headless runs) every car is seen.

use serde::{Deserialize, Serialize};

use crate::lot::OUTDOORS;
use crate::path::join;
use crate::travel::{Mode, RoadPoint, Travel};
use crate::world::{
    Household, Phase, Plot, Sim, SimBrief, Task, TaskKind, World, nearest_exit, wrap_angle,
};
use crate::{DRIVE_TICKS_PER_SECOND, drive_secs, drive_ticks, secs};

/// Most cars in a town (the snapshot's rows).
pub const MAX_CARS: usize = 128;
/// Doors: 0 front left (the driver's), 1 front right, 2 back left, 3 back right.
pub const DOORS: usize = 4;
/// How long the driver waits in the car for the others setting off at the same time.
const BOARD_WAIT_TICKS: u64 = 4 * crate::TICKS_PER_MINUTE;
/// How long getting in takes before the car can go (the doors, the belt: real time).
const BOARD_MIN_TICKS: u64 = secs(0.5);
/// On a trip, the longest the driver waits in the car for the others coming along while they are
/// on their way to it (from the house; from the place, up to `TRIP_PARK_REACH` off, to go home).
const TRIP_WAIT_TICKS: u64 = 2 * 60 * crate::TICKS_PER_MINUTE;
/// Getting in (`Phase::Boarding`): at the door while it opens, a step in to the seat, then sat
/// while it shuts (the renderer swings a door in about 9 ticks at any speed), and in. Getting
/// out (`Phase::Alighting`): sat while the door opens, a step out to beside it, and off (it
/// shuts behind them). Ticks of real time, as the door is seen to swing.
pub(crate) const DOOR_OPENS: u32 = secs(0.5) as u32;
pub(crate) const STEP_IN: u32 = secs(0.7) as u32;
pub(crate) const GET_IN_TICKS: u32 = DOOR_OPENS + STEP_IN + secs(0.6) as u32;
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
/// Someone waiting for a car to pass for this long steps aside out of its way (ticks of real
/// time, `make_way`), this far a tick (m: 1.2 m a second).
pub(crate) const MAKE_WAY_TICKS: u32 = secs(1.5) as u32;
pub(crate) const STEP_ASIDE: f32 = (1.2 / crate::TICKS_PER_SECOND as f64) as f32;
/// Someone walking within about 30° of a car's way (the sine of the angle between) walks along
/// it, not across it out of it (`go_on`).
const ALONG: f32 = 0.5;
/// How long a car waits for someone who can't step aside (with nowhere to step) before they go
/// round to where it has room to pass them (`clear_of`), up to `CLEAR_STEPS` tiles off (ticks of
/// real time).
pub(crate) const PASS_TICKS: u32 = secs(3.0) as u32;
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
/// The least gap between two cars' bodies (m): two coming towards each other pass no nearer
/// (`too_near`; the claims a little further apart, `traffic::PASS_GAP`), and the last-resort check
/// never lets one come nearer another, whoever waits for whom (`guard`). Two in their lanes on a
/// 7 m street are 0.22 m apart at least (the widest cars).
pub(crate) const GAP: f32 = 0.15;
/// How far apart two coming towards each other pull over to pass where the lane is too narrow for
/// both in their lanes (m, `pull_want`: the two together): its measure is across a way between
/// theirs, `too_near` their bodies as they are, which on a bend come nearer.
pub(crate) const PASSING: f32 = 0.1;
pub(crate) const PULL_MARGIN: f32 = 0.3;
/// How far a car pulls over (or back) a tick (m: 1.2 m a second of drive time), and for one coming
/// the other way how far ahead of its front (m, from the other's front: `pull_want`).
pub(crate) const PULL_RATE: f32 = (1.2 / DRIVE_TICKS_PER_SECOND as f64) as f32;
pub(crate) const PULL_AHEAD: f32 = 16.0;
/// How near the furniture (or a country road's edge) a car pulled over comes (m), and how far
/// right of its lane it pulls over at most (m).
const PULL_EDGE: f32 = 0.1;
const PULL_MOST: f32 = 0.7;
/// Turning radius at the driveway's mouth (m).
const TURN: f32 = 2.4;
/// How far past the town's exit a car drives before it's gone (m).
pub(crate) const OVERSHOOT: f32 = 12.0;
/// Top speeds (m per second of drive time, `crate::DRIVE_SCALE`): along the street
/// (`travel::TOWN_SPEED`, 32 km/h), and in the driveway, turning or backing up.
pub(crate) const FAST: f32 = crate::travel::TOWN_SPEED;
const SLOW: f32 = 1.8;
/// Speeding up and slowing down (m/s², drive time: from standing to `FAST` in 3 s).
const ACCEL: f32 = 3.0;
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
/// how slow cars turn through junctions (m/s) within how far of them (m; straight through one, at
/// their speed: `lane_tops`), and the longest straight step
/// between a route's points and how far a step may stray from the lane (m).
pub(crate) const ROAD_REACH: f32 = 30.0;
pub(crate) const JUNCTION_SPEED: f32 = 3.5;
pub(crate) const JUNCTION_REACH: f32 = 8.0;
/// A way turning through a junction by more than this (radians, between the way it comes in and
/// the way it goes out) takes it at `JUNCTION_SPEED` (`lane_tops`); straighter, at its speed.
const TURN_AT: f32 = 0.35;
pub(crate) const ROUTE_STEP: f32 = 4.0;
pub(crate) const ROUTE_TOLERANCE: f32 = 0.08;
/// A lane's corners rounded (`round_corners`): where it turns sharper than `SHARP` (radians) at a
/// point, or at a few within `CORNER_SPAN` m of each other (a junction, a street corner, where one
/// road's end meets another's: their lanes cross or fall short there), an arc as tight as
/// `RIGHT_TURN` turning right (round the near kerb) and `LEFT_TURN` turning left (across the
/// junction, its middle on the inside) at most, ending in the lanes' lines; a gentler bend, an arc
/// from the middle of the step before to the middle of the next (as wide as `BEND` at most). Each
/// sampled a point every `ARC_STEP` radians of the turn: the car's nose, facing the way the points
/// run on either side (`Legs::timed`), within half that of the way it goes.
const SHARP: f32 = 0.44;
const CORNER_SPAN: f32 = 3.0;
const RIGHT_TURN: f32 = TURN;
const LEFT_TURN: f32 = 4.0;
const BEND: f32 = 30.0;
const ARC_STEP: f32 = 0.1;
/// How much longer than it's off a corner's line a lead into its curve (or out of it) from a point
/// a little off it is (`round_corners`): the step turns no more than about `ARC_STEP`.
const LEAD: f32 = 10.0;
/// The most a car's way pulls it sideways turning (m/s² of drive time, `Legs::timed`): round a
/// junction's corner it slows to about 4 to 5 m/s, a country road's bend to 14 (4 before the
/// junctions were lit: the lights and the stop lines do the stopping now).
const SIDEWAYS: f32 = 6.5;
/// A step of a route shorter than this says nothing of the way a car faces along it (m,
/// `Legs::timed`): it faces as along the step before. (Where a taxi stands at the kerb, the way on
/// began 2 to 8 mm from it and mostly across the lane, and it stood 25 to 37 degrees across its
/// lane: King'ston, Pebble Quay.)
const FACING_STEP: f32 = 0.02;
/// A car turned round joins its new way's lane at the first point it drives on to within this of
/// the way it faces (the cosine of the angle, `Route::detour`), of those nearer than `JOIN_NEAR`
/// (m): one further off it drives to however it lies.
const JOIN_COS: f32 = 0.7;
const JOIN_NEAR: f32 = 2.5;
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

/// A car's way turned round (`unjam`, `Route::detour`): it was held up `from` ticks along the way
/// it had (`home`: its way home), backs up along it to `back` and goes on from there another way,
/// round edge `avoid` of the roads (`travel::Graph::edges`). Saved with the car, so its way after a
/// load is the same.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Detour {
    pub home: bool,
    pub from: f32,
    pub back: f32,
    pub avoid: u32,
    /// It only backs up out of the way, and goes on the way it had: a work taxi with its stops
    /// still ahead on it (`taxi.rs`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub same: bool,
    /// It turns round in a driveway's mouth it came by (`MouthTurn`), backing into it and out the
    /// other way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mouth: Option<MouthTurn>,
}

/// A driveway's mouth a car turns round in (`Detour::mouth`): the road it meets and how far along
/// it (the network's: `travel::Graph::roads`), the way it runs to the road (`Drive::way`), and
/// the way along the road (+1: the road's own) the car came going, backing into it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MouthTurn {
    pub road: u32,
    pub s: f32,
    pub way: [f32; 2],
    pub go: f32,
}

/// A car's way round a work taxi standing at the kerb in its lane (`pass_stands`): its way's
/// points from `from` to `to` (ticks along it) moved `off` m to their right (out into the other
/// lane: less than 0), all of it between `out` and `back` (alongside the taxi and clear of it at
/// either end), eased out before and back in after. A claim of its own, taken in turns with
/// those coming the other way there (`Route::spans`). Made on its way as it was then (`home`, and
/// `at`: where it is `from` along it), taxi `taxi`'s (its index in `World::cars`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassBy {
    pub home: bool,
    pub taxi: u32,
    pub from: f32,
    pub out: f32,
    pub back: f32,
    pub to: f32,
    pub off: f32,
    pub at: [f32; 2],
}

/// What going along the stretch a car turned round from costs it more, finding its new way
/// (seconds: `Route::detour`): only if there's no other way does it go back that way.
const DETOUR_AVOID: f32 = 1.0e5;

/// Whether a car on its way is in sight (`sight`, see the module notes): under the rules (`Seen`),
/// out of sight driving through the others (`Ghost`), or just outside the seen area waiting for its
/// way into it to clear of one in sight (`Wait`: that one's `Driving::first`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sight {
    #[default]
    Seen,
    Ghost,
    Wait(u64),
}

/// The seen area reaches this far (m) round the watch (`World::set_watch`): those coming in are
/// under the rules this far short of where anyone could see them, a camera moving on finds them so
/// there, and one out of sight driving through another is never near what's drawn.
pub(crate) const SEEN_MARGIN: f32 = 32.0;

/// One in sight goes out of it only this much further out (m): backed up a little out of
/// another's way at the edge of the seen area, or pulled over there, it isn't in and out of sight
/// by turns, letting go of its claims each time (`sight`).
const SEEN_SLACK: f32 = 12.0;

/// One out of sight comes under the rules once it's this near coming into sight along its way
/// (ticks: the next, then each second of drive time up to 4 s, some 20 m at a town's speed),
/// clear of those in sight (`sight`).
const COMING_IN: [f32; 5] = [
    1.0,
    drive_ticks(1.0),
    drive_ticks(2.0),
    drive_ticks(3.0),
    drive_ticks(4.0),
];
/// How far apart (ticks: a quarter of a second of drive time, 1.5 m at a town's speed, 4 m on a
/// country road straight through a junction: less than the smallest box) a car's way into sight
/// is looked along (`sight`: its claims reaching into sight, and `METER`'s queue).
const SIGHT_STEP: f32 = drive_ticks(0.25);

/// Coming into sight (`sight`), a car out of it waits outside while one in sight standing still
/// (`METER_STILL`, or a taxi at its stop) is on its way in up to the first junction it takes
/// turns at in sight (else `ahead` ticks along it; `most` at most; sampled every `SIGHT_STEP`),
/// this near it past `GAP` (`gap`, m): the queue there has no room for it. Out of sight nobody
/// waits, so those coming would all come in at once and fill the seen area past what the rules
/// would have let through (Fernglade).
struct Meter {
    ahead: f32,
    most: f32,
    gap: f32,
}
const METER: Meter = Meter {
    ahead: drive_ticks(6.0),
    most: drive_ticks(75.0),
    gap: 1.0,
};
/// How long one in sight on its way has stood still for its queue to count as full (ticks of drive
/// time).
const METER_STILL: u64 = drive_secs(1.0);
/// How long one waits outside for room at most (ticks of drive time, `METER`): then it comes in
/// all the same.
const METER_MOST: u64 = drive_secs(5.0);

/// Two cars this near (m) touch: one the camera comes to touching one in sight is put clear
/// (`sight`), and one in sight touching one out of it isn't drawn (`hidden`). (Those under the
/// rules keep `GAP` apart.)
const TOUCH: f32 = 0.05;

/// Till when a car turned round waits out of the way (`Car::let_by`), and those it waits for to
/// go on: each car, how far along its way it was, in which state.
type LetBy = (u64, Vec<(u32, f32, CarState)>);

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
    /// Ticks it and one in its way have waited for each other, ticks it has backed up since
    /// (`back_off`), and whether it can back up no further (the other backs up instead:
    /// `waits_back`). Not saved.
    #[serde(skip)]
    jam: (u32, u32, bool),
    /// Backed up out of the way of this one (`back_off`): it stays back while that one is near.
    /// Not saved.
    #[serde(skip)]
    backed_for: Option<u64>,
    /// A work taxi (`taxi.rs`), not a household's: it comes in at a way out on a run, drives it
    /// and goes again, and nothing of the households' (`update`) is its. Not saved with the cars
    /// (the runs are, `taxi::Taxis`).
    #[serde(skip)]
    pub taxi: bool,
    /// A taxi standing at a stop (`taxi.rs`): shown standing, its doors as its riders open them.
    /// Not saved.
    #[serde(skip)]
    pub standing: bool,
    /// On its way, whether it's in sight (`sight`; every car parked or out of town is). Not saved.
    #[serde(skip)]
    pub sight: Sight,
    /// Since when it waits outside the seen area for room in the queue on its way in (`METER`;
    /// a tick). Not saved.
    #[serde(skip)]
    metered: Option<u64>,
    /// The furthest along its way it has been, in sight, and since when (`unstick`, `unjam`).
    /// Not saved.
    #[serde(skip)]
    stall: (f32, u64),
    /// Its way turned round, held up (`unjam`); none: the way it always takes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detour: Option<Detour>,
    /// A work taxi that gave its way up in sight (`taxi.rs`): its run over, it turns round and
    /// drives off out of town (`drive_off`) rather than leave the road where it is. Not saved.
    #[serde(skip)]
    pub(crate) off_run: bool,
    /// Turned round with no other way than back where it was held up (`unjam`): it waits out of
    /// the way, where it turned, till those it was held up with have gone on (car, how far along,
    /// in which state), or till the tick given. Not saved.
    #[serde(skip)]
    let_by: Option<LetBy>,
    /// The stretches of road it turned round from lately (`travel::Graph::edges`) and till when it
    /// keeps off them, turning round again (`AVOID_FOR`). Not saved.
    #[serde(skip)]
    avoided: Vec<(u32, u64)>,
    /// Its way out round a work taxi standing at the kerb in its lane, back in past it
    /// (`pass_stands`), laid on its way; saved, so a game loaded part way round is the same.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass_by: Option<PassBy>,
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
    /// The road (`roads` index) the driveway meets and how far along it (m): the sign that its
    /// cars drive the town's roads (`settle_all`).
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
    fn unhold(&mut self) {
        self.moved = false;
        self.waits_for = None;
        self.on_foot.clear();
        self.waited = 0;
        self.jam = (0, 0, false);
        self.backed_for = None;
        self.pulled_for = None;
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
    fn route(&mut self, exits: &[[f32; 2]], travel: Option<&Travel>, home: bool) -> &Route {
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
    fn detoured(
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
        Some(r)
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
            }
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

/// The top speed at each of `lane`'s points (m/s of drive time; `kind_at` the kind of road at
/// each): `FAST` in the hamlets, `Travel::country_speed` on country roads, and `JUNCTION_SPEED`
/// through a junction (within `JUNCTION_REACH` of it, `junctions`) where the way turns there by
/// more than `TURN_AT`. Straight through one it keeps its speed (its bends limit it,
/// `Legs::timed`'s `SIDEWAYS`; the lights and the stop lines stop it, `traffic.rs`).
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
    use crate::travel::dist;
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
    let road_at = |p: [f32; 2]| {
        path.pts
            .iter()
            .min_by(|x, y| dist(x.0, p).total_cmp(&dist(y.0, p)))
            .map(|x| (x.1, x.2))
    };
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
    /// Where it takes turns with one passing a work taxi standing at the kerb the other way
    /// (ticks along it, from and to: `pass_stands`), claims of their own.
    spans: Vec<(f32, f32)>,
    /// Its way out round a work taxi standing at the kerb in its lane as laid on it (`PassBy`),
    /// with what it took the place of, to put back (`Leg`).
    leg: Option<Leg>,
    /// The taxis standing it was last looked at against (`Traffic::stand_gen`; 0: never).
    stand_gen: u64,
}

/// A car's way round a taxi standing at the kerb (`PassBy`) as laid on its route: the points
/// from `at` on it took the place of, and their rooms and lanes (`Route::room`, `Route::lane`),
/// and how many it laid there.
#[derive(Debug, Clone)]
struct Leg {
    pass: PassBy,
    at: usize,
    laid: usize,
    points: Vec<([f32; 2], f32, f32, bool)>,
    room: Vec<f32>,
    lane: Vec<f32>,
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
    fn lay_leg(&mut self, pass: PassBy) -> bool {
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
    fn unlay_leg(&mut self) {
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
        let road_at = |p: [f32; 2]| {
            path.pts
                .iter()
                .min_by(|x, y| dist(x.0, p).total_cmp(&dist(y.0, p)))
                .map_or((7.0, RoadKind::Street), |x| (x.1, x.2))
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
    fn detour(
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
        let road_at = |q: [f32; 2]| {
            path.pts
                .iter()
                .min_by(|x, y| dist(x.0, q).total_cmp(&dist(y.0, q)))
                .map_or((7.0, RoadKind::Street), |x| (x.1, x.2))
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

/// Where a car parks off the road, its driveway or a parking bay, in the frame of the road where
/// it meets it (`mp` on its middle line, its tangent `t` and normal `n`): where it parks (`x0`,
/// `z0` in that frame) facing `yaw`, the way it runs to the road (`u`: along its axis, else
/// square to the road), and the kerb on its side of the road (`kerb`, its `z` in that frame).
pub(crate) struct Bay {
    spot: [f32; 2],
    yaw: f32,
    mp: [f32; 2],
    pub(crate) t: [f32; 2],
    n: [f32; 2],
    x0: f32,
    z0: f32,
    u: [f32; 2],
    kerb: f32,
}

impl Bay {
    /// On `road` (its kerbs: `kerb_off`). None: it comes alongside the road rather than onto it
    /// (`OBLIQUE`).
    pub(crate) fn new(
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
    pub(crate) fn near(&self, p: [f32; 2], at: f32, r: f32, d: f32) -> bool {
        (self.local(p)[0] - at) * d < r + 0.5
            && crate::travel::dist(p, self.world([at, 0.0])) < 2.0 * r + 4.0
    }

    /// Out of it onto the lane going `d` (+1: along `t`): facing the road it pulls out, facing
    /// away it backs out turning away from the way it goes (as `Route::legacy` does). Where the
    /// turn is along the road, and how much of the lane it takes up.
    pub(crate) fn leave(&self, legs: &mut Legs, d: f32) -> (f32, f32) {
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
    pub(crate) fn entry(&self, d: f32) -> (Turn, f32) {
        let lane_in = -d * TIGHT_LANE;
        let (at, room) = self.cross(lane_in);
        let room = room.min(self.past_kerb(lane_in));
        (
            Turn::round([at, lane_in], [-d, 0.0], [-self.u[0], -self.u[1]], room),
            at,
        )
    }

    /// Into it, nose first, by `turn` (`entry`).
    pub(crate) fn enter(&self, legs: &mut Legs, turn: &Turn) {
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

/// How far past `lane`'s own line at its point `i` the right side of a body `len` m long standing
/// along it there reaches (m), on the outside of a bend to the left: the sagitta of the chord its
/// length spans, which `pull_room` (worked out as if the road ran straight) leaves no room for.
/// None on a straight, or on a bend to the right, its right side on the inside. (A taxi pulled
/// over its full room at the kerb of a lane bending left stood with a corner over the kerbside
/// furniture: Primrosebrook.)
pub(crate) fn bend_out(lane: &[[f32; 2]], i: usize, len: f32) -> f32 {
    use crate::travel::dist;
    let Some(&p) = lane.get(i) else {
        return 0.0;
    };
    // The point `len / 2` m along the lane from `i`, back (`step` -1) or on (+1), or its end.
    let reach = |step: isize| -> [f32; 2] {
        let (mut at, mut left) = (i as isize, len / 2.0);
        loop {
            let next = at + step;
            if next < 0 || next as usize >= lane.len() {
                return lane[at as usize];
            }
            let (a, b) = (lane[at as usize], lane[next as usize]);
            let d = dist(a, b);
            if d >= left {
                let k = left / d.max(1e-6);
                return [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k];
            }
            left -= d;
            at = next;
        }
    };
    let (a, b) = (reach(-1), reach(1));
    let d = sub2(b, a);
    let l = d[0].hypot(d[1]);
    if l < 1e-3 {
        return 0.0;
    }
    // (Right of the way it runs, as `shifted`: (-z, x).)
    (dot2(sub2(p, a), [-d[1], d[0]]) / l).max(0.0)
}

/// `p`, facing `yaw`, `shift` m to its right.
pub(crate) fn shifted(p: [f32; 2], yaw: f32, shift: f32) -> [f32; 2] {
    // (Forward is (sin, cos); its right (-cos, sin).)
    [p[0] - yaw.cos() * shift, p[1] + yaw.sin() * shift]
}

/// `pts` moved `offs[i]` m to the right (looking the way they run), each its own.
pub(crate) fn offset_right_by(pts: &[[f32; 2]], offs: &[f32]) -> Vec<[f32; 2]> {
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

/// `lane` (a lane's points, `thin_with`) with its corners rounded, its ends where they were: where
/// it turns sharper than `SHARP` at a point or at a few close together (a junction, a street corner,
/// where one road's end meets another's: the two lanes' lines cross there, or one goes on past the
/// other's and the way doubles back on itself a little), an arc into the line out from the line in
/// (`sharp_corners`); then at each other bend, an arc from the middle of the step before to the
/// middle of the next (`gentle_bends`).
pub(crate) fn round_corners(lane: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let (pts, fixed) = sharp_corners(lane);
    let mut out = gentle_bends(&pts, &fixed);
    out.dedup_by(|b, a| crate::travel::dist(*a, *b) < 1e-3);
    out
}

fn sub2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn dot2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

fn cross2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

/// The way from `a` to `b`, a unit long.
fn unit2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let l = crate::travel::dist(a, b).max(1e-6);
    [(b[0] - a[0]) / l, (b[1] - a[1]) / l]
}

/// How far going `b` is turned from going `a` (radians, + to the right: the town's x runs right
/// and z down).
fn turned(a: [f32; 2], b: [f32; 2]) -> f32 {
    cross2(a, b).atan2(dot2(a, b))
}

/// How far `p` is off the line through `c` going `d`.
fn off_line(p: [f32; 2], c: [f32; 2], d: [f32; 2]) -> f32 {
    cross2(d, sub2(p, c)).abs()
}

/// An arc of radius `r` from `from` going `din`, turning `theta` (`turned`): a point every
/// `ARC_STEP` of it, its ends too, onto `out`.
fn arc_onto(out: &mut Vec<[f32; 2]>, from: [f32; 2], din: [f32; 2], theta: f32, r: f32) {
    // (About its middle, towards the inside of the turn square to the way in.)
    let inside = if theta > 0.0 {
        [-din[1], din[0]]
    } else {
        [din[1], -din[0]]
    };
    let centre = [from[0] + inside[0] * r, from[1] + inside[1] * r];
    let v = sub2(from, centre);
    let k = (theta.abs() / ARC_STEP).ceil().max(1.0) as usize;
    for s in 0..=k {
        let (sn, cs) = (theta * s as f32 / k as f32).sin_cos();
        let p = [
            centre[0] + v[0] * cs - v[1] * sn,
            centre[1] + v[0] * sn + v[1] * cs,
        ];
        if out.last().is_none_or(|q| crate::travel::dist(*q, p) > 1e-3) {
            out.push(p);
        }
    }
}

/// A curve from `from` to `to` round the corner `c` (a quadratic Bézier: along the lines from
/// `from` to `c` and from `c` to `to` at its ends), a point every `ARC_STEP` it turns, onto `out`.
fn curve_onto(out: &mut Vec<[f32; 2]>, from: [f32; 2], c: [f32; 2], to: [f32; 2]) {
    const STEPS: usize = 128;
    let (a, b) = (sub2(c, from), sub2(to, c));
    let way = |s: f32| [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s];
    let at = |s: f32| {
        let (u, v, w) = ((1.0 - s) * (1.0 - s), 2.0 * s * (1.0 - s), s * s);
        [
            from[0] * u + c[0] * v + to[0] * w,
            from[1] * u + c[1] * v + to[1] * w,
        ]
    };
    if out
        .last()
        .is_none_or(|q| crate::travel::dist(*q, from) > 1e-3)
    {
        out.push(from);
    }
    let mut was = way(0.0);
    for k in 1..=STEPS {
        let s = k as f32 / STEPS as f32;
        let d = way(s);
        if k == STEPS || turned(was, d).abs() >= ARC_STEP {
            out.push(at(s));
            was = d;
        }
    }
}

/// `round_corners`' sharp corners: its points, and which stay as they are (`gentle_bends`): its
/// ends and the arcs'.
fn sharp_corners(lane: &[[f32; 2]]) -> (Vec<[f32; 2]>, Vec<bool>) {
    use crate::travel::dist;
    let n = lane.len();
    if n < 3 {
        return (lane.to_vec(), vec![true; n]);
    }
    let turn_at = |i: usize| turned(unit2(lane[i - 1], lane[i]), unit2(lane[i], lane[i + 1]));
    let mut out: Vec<[f32; 2]> = vec![lane[0]];
    let mut fixed = vec![true];
    // (The points laid so far that stay: the start, and every arc.)
    let mut locked = 1;
    let mut i = 1;
    while i + 1 < n {
        if turn_at(i).abs() < SHARP {
            out.push(lane[i]);
            fixed.push(false);
            i += 1;
            continue;
        }
        // The corner: this point and the sharp ones close on after it.
        let mut j = i;
        while j + 2 < n && dist(lane[j], lane[j + 1]) < CORNER_SPAN && turn_at(j + 1).abs() >= SHARP
        {
            j += 1;
        }
        let din = unit2(lane[i - 1], lane[i]);
        let dout = unit2(lane[j], lane[j + 1]);
        let theta = turned(din, dout);
        // (Its way in a step back against the way it came, where the way goes a little past a
        // road's end and back: as it is. Rounded from that step, the curve began on the far side
        // of the corner, and the road before it, ahead of the curve's start that way, was taken
        // out with it: a taxi drove 130 m straight across the grass, Applewold.)
        if i >= 2 && dot2(unit2(lane[i - 2], lane[i - 1]), din) < 0.0 {
            out.extend_from_slice(&lane[i..=j]);
            fixed.resize(out.len(), false);
            i = j + 1;
            continue;
        }
        let mut keep = |out: &mut Vec<[f32; 2]>| {
            out.extend_from_slice(&lane[i..=j]);
            fixed.resize(out.len(), false);
        };
        // (A jog back to the line it was on: left out; to another line, or turning back on
        // itself, as it is.)
        if theta.abs() < 0.05 {
            if j == i || off_line(lane[j + 1], lane[i - 1], din) > 0.3 {
                keep(&mut out);
            }
            i = j + 1;
            continue;
        }
        // Where its lines in and out cross.
        let c = if j == i {
            lane[i]
        } else {
            let u = cross2(sub2(lane[j], lane[i]), dout) / cross2(din, dout);
            [lane[i][0] + din[0] * u, lane[i][1] + din[1] * u]
        };
        if theta.abs() > std::f32::consts::PI - 0.17
            || dist(c, lane[i]) > 2.0 * CORNER_SPAN
            || dist(c, lane[j]) > 2.0 * CORNER_SPAN
        {
            keep(&mut out);
            i = j + 1;
            continue;
        }
        // How far back along the line in it may begin (back over the points on that line, not
        // into an arc), and how far on along the line out it may end (over the points on that line
        // to where the lane turns again, half way there).
        let mut m = out.len() - 1;
        while m >= locked && off_line(out[m - 1], c, din) < 0.2 {
            m -= 1;
        }
        // (Where it begins and ends, the steps from and to the points either side that stay
        // turn little: those a little off its lines, a lead in or out long enough.)
        let back = dot2(sub2(c, out[m]), din) - LEAD * off_line(out[m], c, din);
        let mut q = j + 1;
        while q + 1 < n && turn_at(q).abs() < 0.02 && off_line(lane[q + 1], c, dout) < 0.2 {
            q += 1;
        }
        let ahead = dot2(sub2(lane[q], c), dout);
        let share = if q + 1 < n { ahead / 2.0 } else { ahead };
        let fwd = share.min(ahead - LEAD * off_line(lane[q], c, dout));
        // (As far from the corner either way as an arc as wide as `most` takes, or as there's
        // room for: short of it on one side, a curve wider on the other.)
        let tan = (theta.abs() / 2.0).tan();
        let most = if theta > 0.0 { RIGHT_TURN } else { LEFT_TURN };
        let (into, outof) = ((most * tan).min(back - 0.01), (most * tan).min(fwd - 0.01));
        if into.min(outof) < 0.02 {
            keep(&mut out);
            i = j + 1;
            continue;
        }
        let from = [c[0] - din[0] * into, c[1] - din[1] * into];
        let to = [c[0] + dout[0] * outof, c[1] + dout[1] * outof];
        // (Only those about the corner, never the road before it.)
        let reach = into + 3.0 * CORNER_SPAN;
        while out.len() > locked
            && dot2(sub2(*out.last().expect("one"), from), din) > -0.01
            && dist(*out.last().expect("one"), c) <= reach
        {
            out.pop();
            fixed.pop();
        }
        if (into - outof).abs() < 1e-3 {
            arc_onto(&mut out, from, din, theta, into / tan);
        } else {
            curve_onto(&mut out, from, c, to);
        }
        fixed.resize(out.len(), true);
        locked = out.len();
        // On from the first point past its end.
        i = j + 1;
        while i + 1 < n && dot2(sub2(lane[i], to), dout) < 0.01 {
            i += 1;
        }
    }
    let last = lane[n - 1];
    if out.last().is_none_or(|p| dist(*p, last) > 1e-3) {
        out.push(last);
        fixed.push(true);
    }
    if let Some(f) = fixed.last_mut() {
        *f = true;
    }
    (out, fixed)
}

/// `round_corners`' gentler bends: at each of `pts`' points but the `fixed` ones, an arc as wide as
/// `BEND` at most from the middle of the step before (or the point before, fixed or straight on) to
/// the middle of the next (or the next point, likewise).
fn gentle_bends(pts: &[[f32; 2]], fixed: &[bool]) -> Vec<[f32; 2]> {
    use crate::travel::dist;
    let n = pts.len();
    if n < 3 {
        return pts.to_vec();
    }
    let turn_at = |i: usize| turned(unit2(pts[i - 1], pts[i]), unit2(pts[i], pts[i + 1]));
    let bends = |i: usize| {
        !fixed[i] && i + 1 < n && {
            let t = turn_at(i).abs();
            t > 1e-3 && t < std::f32::consts::PI - 0.17
        }
    };
    let mut out: Vec<[f32; 2]> = vec![pts[0]];
    for i in 1..n - 1 {
        if !bends(i) {
            out.push(pts[i]);
            continue;
        }
        let din = unit2(pts[i - 1], pts[i]);
        let dout = unit2(pts[i], pts[i + 1]);
        let theta = turned(din, dout);
        let back = dist(*out.last().expect("one"), pts[i]);
        let step = dist(pts[i], pts[i + 1]);
        let fwd = if bends(i + 1) { step / 2.0 } else { step };
        let tan = (theta.abs() / 2.0).tan();
        let taken = (BEND * tan).min(back).min(fwd);
        if taken < 0.01 {
            out.push(pts[i]);
            continue;
        }
        let from = [pts[i][0] - din[0] * taken, pts[i][1] - din[1] * taken];
        arc_onto(&mut out, from, din, theta, taken / tan);
    }
    out.push(pts[n - 1]);
    out
}

/// A turn at a driveway's mouth, in the frame of the road there (`Route::on_roads`): from going
/// `din` to going `dout` round the corner where their lines cross, an arc as tight as `TURN` at
/// most that begins no further back from the corner than `room` (the driveway's length to it).
/// A driveway square to the road turns a quarter, as `Route::legacy` does.
pub(crate) struct Turn {
    /// Where it begins and ends, the arc's middle and radius (0: straight on).
    from: [f32; 2],
    to: [f32; 2],
    centre: [f32; 2],
    r: f32,
    /// How far from the corner it begins and ends (m).
    pub(crate) taken: f32,
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
    pub(crate) pts: Vec<([f32; 2], bool, f32)>,
    /// At each point, how far a car can pull over there (`Route::room`), and its lane
    /// (`Route::lane`).
    room: Vec<f32>,
    lane: Vec<f32>,
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
            let f = if ds[i] > FACING_STEP || i == 0 {
                let way = (b[0] - a[0]).atan2(b[1] - a[1]);
                if pts[i + 1].1 {
                    wrap_angle(way + std::f32::consts::PI)
                } else {
                    way
                }
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
            spans: Vec::new(),
            leg: None,
            stand_gen: 0,
        }
    }
}

/// The car `sim` would take to work (or school) now, and their seat in it (0: they drive): an
/// owner 18 or older takes their household's car from home when it's parked there; anyone else in
/// the household rides along when an owner is about to drive off in it (or on their way to). Not
/// one booked on a work taxi (`taxi.rs`): it comes for them as planned.
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
    // (Called to the household's car on their way out to the kerb, a rider left the taxi to
    // come in for nobody, held up by that car and turned round into the next taxi's way.)
    if !sim.here() || me.baby || sim.taxi.is_some() {
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
    lead_by(
        &w.content,
        &w.cars,
        &w.briefs,
        &w.households,
        &w.plots,
        &w.relationships,
        w.tick,
        &w.sims[i],
        |j| w.sims.get(j),
        start,
    )
}

/// `lead_ticks` for `s` in a resident's update (`s` that one, or another of `ctx.others`): those
/// it doesn't have, the one being updated, drive nobody.
pub(crate) fn lead_ticks_in(ctx: &crate::world::Ctx, s: &Sim, start: u64) -> u64 {
    lead_by(
        ctx.content,
        ctx.cars,
        ctx.briefs,
        ctx.households,
        ctx.plots,
        ctx.rels,
        ctx.tick,
        s,
        |j| ctx.others.get(j),
        start,
    )
}

/// `lead_ticks` for `s`, the others by index from `sim_of`.
#[allow(clippy::too_many_arguments)]
fn lead_by<'a>(
    content: &crate::content::Content,
    cars: &[Car],
    briefs: &[SimBrief],
    households: &[Household],
    plots: &[Plot],
    rels: &crate::social::Relationships,
    tick: u64,
    s: &Sim,
    sim_of: impl Fn(usize) -> Option<&'a Sim>,
    start: u64,
) -> u64 {
    let me = s.id;
    let home = households.get(s.household as usize).and_then(|h| h.plot);
    let usable = |c: &&Car| c.at_home() && !c.blocked && !c.no_way;
    // An owner who drives it and starts about when they do.
    let sets_off = |car: &Car, j: u32| {
        j != me
            && car.driven_by(briefs, j)
            && sim_of(j as usize).is_some_and(|o| {
                o.here()
                    && crate::life::start_on(content, o, tick)
                        .is_some_and(|st| st.abs_diff(start) <= BOARD_WAIT_TICKS)
            })
    };
    let own = cars
        .iter()
        .filter(usable)
        .find(|c| c.household == s.household && Some(c.plot) == home)
        .filter(|car| car.driven_by(briefs, me) || car.owners.iter().any(|&j| sets_off(car, j)));
    // Else a lift from a neighbour leaving with them (`lift_seat`).
    let door = home
        .and_then(|p| plots.get(p as usize))
        .map(|p| p.arrival_point());
    let car = own.or_else(|| {
        let door = door?;
        cars.iter().filter(|c| c.at_home()).find(|c| {
            c.owners
                .iter()
                .any(|&j| lift_ok(c, households, rels, door, me, s.household, j) && sets_off(c, j))
        })
    });
    let Some(car) = car else {
        return 0;
    };
    WALK_TO_CAR_TICKS + u64::from(GET_IN_TICKS) + drive_out_ticks(car) + off_map(content)
}

/// Once in `car` for work or school: the wait for it to go and the drive out of town.
pub(crate) fn drive_out_ticks(car: &Car) -> u64 {
    BOARD_MIN_TICKS + car.out_ticks.ceil() as u64
}

/// The drive between the end of town and work: `commuteMinutes` (the part of the way off the
/// map).
pub(crate) fn off_map_ticks(w: &World) -> u64 {
    off_map(&w.content)
}

/// `off_map_ticks` for a content alone (`lead_by` has no world to hand).
fn off_map(content: &crate::content::Content) -> u64 {
    crate::life::ticks(content.career_rules.commute_minutes)
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
        // (A work taxi is the taxis' own: `taxi.rs`; but one driving off, its run given up in
        // sight.)
        if w.cars[c].taxi {
            if w.cars[c].off_run {
                drive_off(w, c);
            }
            continue;
        }
        owners(w, c);
        relocate(w, c);
        let gone = |w: &World, s: u32| w.sims.get(s as usize).is_none_or(|s| !s.here());
        // Nobody to drive it home (the driver died or moved away while out): it's back. (Not
        // while it's in the picture, nor its spot at home: it would jump there, `in_picture`;
        // meanwhile it goes on.)
        let seen =
            w.cars[c].shown.is_some_and(|s| in_picture(w, s.0)) || in_picture(w, w.cars[c].spot);
        if !w.cars[c].at_home() && w.cars[c].driver.is_none_or(|d| gone(w, d)) && !seen {
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
    match w.cars[c].sight {
        Sight::Seen => {}
        Sight::Ghost => return drive_through(w, c, home),
        Sight::Wait(k) => {
            let car = &mut w.cars[c];
            car.unhold();
            car.waits_for = Some(k);
            return;
        }
    }
    let t = w.cars[c].progress;
    let moving = std::mem::take(&mut w.cars[c].moved);
    // Held up for good, nose to nose with one waiting for it (`JAM_TICKS`), it backs up a little
    // out of its way (`back_off`).
    let key = car_first(c);
    let jammed = w.cars[c].waits_for.is_some_and(|k| waits_back(w, k, key));
    // (Not on its way since, it's still held up: two pulling over for each other by turns wait
    // for each other every other tick.)
    let jam = &mut w.cars[c].jam;
    *jam = if jammed {
        (jam.0.saturating_add(1), jam.1, jam.2)
    } else if moving {
        (0, 0, false)
    } else {
        *jam
    };
    let jam = *jam;
    if jam.1 == 0 {
        w.cars[c].backed_for = None;
    }
    // (The other can back up no further, and waits for it as it does: it backs up now.)
    let theirs = w.cars[c].waits_for.is_some_and(|k| stuck_back(w, k));
    // (Backed up as far as it goes, `JAM_BACK`, it can back up no further: the other does.)
    // (Backing up a tick on, turned round: `unjam`.)
    let turning = w.cars[c].detour.is_some()
        && w.cars[c]
            .route
            .as_ref()
            .is_some_and(|(_, r)| r.at(t + 1.0).2);
    // (Backed up out of the way to let those it was held up with by, `unjam`: out of the way it
    // waits till they've gone on.)
    if !turning && let Some((until, by)) = w.cars[c].let_by.as_ref() {
        // (Not for one that waits for it in turn, behind it say: that one can't go on first.)
        let me = car_first(c);
        let still = (w.tick < *until)
            .then(|| {
                by.iter().find(|&&(o, p, st)| {
                    w.cars.get(o as usize).is_some_and(|x| {
                        x.state == st
                            && x.progress <= p + 1.0
                            && x.sight == Sight::Seen
                            && !crate::traffic::waits_on(w, car_first(o as usize)).contains(&me)
                    })
                })
            })
            .flatten()
            .map(|&(o, ..)| o);
        match still {
            Some(o) => {
                let car = &mut w.cars[c];
                car.waits_for = Some(car_first(o as usize));
                car.moved = false;
                return;
            }
            None => w.cars[c].let_by = None,
        }
    }
    if jammed && (jam.0 > JAM_TICKS || theirs) && !w.cars[c].outside() && !turning {
        if jam.1 < JAM_BACK && back_off(w, c) {
            return;
        }
        w.cars[c].jam.2 = true;
    }
    // (Backed up out of its way, it stays there while that one still waits for it, or while it
    // still waits its turn after it, or that one is still by it (held up there by others now and
    // then, on foot too): not on into its way again. Not one that could back up no further
    // (`Car::jam`'s third, the other backs instead): it goes on as soon as the other gives it
    // room, else both stood back for each other for good, the other coming up again to wait on
    // it, Tide Sands.)
    let asking = !w.traffic.waits_for(key).is_empty();
    let by = w.cars[c].backed_for.is_some_and(|k| {
        let me = &w.cars[c];
        w.cars
            .get((k & !(1 << 63)) as usize)
            .and_then(|o| Some((o, o.shown.filter(|s| s.2)?)))
            .zip(me.shown)
            .is_some_and(|((o, s), m)| {
                crate::travel::dist(m.0, s.0) < (me.len + o.len) / 2.0 + 2.0 * AHEAD
            })
    });
    if (jammed || asking || by) && jam.1 > 0 && !jam.2 && !w.cars[c].outside() {
        return;
    }
    // (Whom it waited for last tick: pulled over for that one, it doesn't pull back in into its
    // way, `pull_over`.)
    let waited = w.cars[c].waits_for.take();
    w.cars[c].on_foot.clear();
    let others = vehicles(&w.cars, Some(c));
    if w.cars[c].outside() && !room_to_come_in(w, c, &others) {
        return;
    }
    let tick = w.tick;
    // (Turning round, it backs up as it stands, pulled over as far as it was: that's the room it
    // found clear, `turn_round`.)
    if !w.cars[c].outside() && !turning {
        pull_over(w, c, &others, moving, waited);
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
    // (Pulled over further than it has room on: back towards its lane first. Not turning round:
    // it backs up as it stands, into a driveway's mouth too, `turn_round`.)
    if !turning && w.cars[c].shift > route_room(&w.cars[c], t + 1.0) + 1e-3 {
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
    // (Those waiting for it, their bodies or claims in its way: it goes on by into the ground one
    // of them turning round still backs up over.)
    let waiting: Vec<u64> = traffic
        .turning
        .iter()
        .map(|t| t.0)
        .filter(|&k| {
            cars.get((k & !(1 << 63)) as usize)
                .is_some_and(|o| o.waits_for == Some(key))
                || traffic.waits_for(k).contains(&key)
        })
        .collect();
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
    // (Turning round, `unjam`, it backs slowly into room it found clear: only what's right behind
    // it holds it up, `TURN_BEHIND`.)
    let back = if reverse && car.detour.is_some() {
        TURN_BEHIND
    } else {
        AHEAD
    };
    let (ahead, behind) = if reverse { (0.0, back) } else { (AHEAD, 0.0) };
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
    let pass = ([1.0, drive_ticks(0.5), drive_ticks(1.0)]
        .into_iter()
        .map(|k| route_pass(car, t + k))
        .fold(f32::INFINITY, f32::min)
        - shift)
        .max(0.0);
    let holding_here = traffic.holding(key);
    let in_drive =
        !home && !holding_here && car.detour.is_none() && first_claim.is_some_and(|from| t < from);
    // (Backing up out of where it was held up, turned round, `unjam`: into clear room, never into
    // another, as `guard` keeps it; not kept by the claims of those it backs away from.)
    let turning = reverse && car.detour.is_some();
    if let Some(k) = traffic
        .blocked(key, &body_now, &body_next, me.heading(), pass, in_drive)
        .filter(|_| !turning)
    {
        car.waits_for = Some(k);
        return;
    }
    // Not on past one coming the other way too near to pass (pulling over meanwhile); but for
    // one waiting for it (for its claim, or its body in the way of its own): it goes on by, as
    // near as it can, never nearer than `GAP` (`closes_in`, as the guard). Nor in a junction's box
    // holding its claim there: there the claims say who goes (one coming the other way let in with
    // it can pass it in their lanes, nobody pulls over there; one not let in waits for it), and
    // waiting for one coming would let that one's claim through this one's (`Traffic::begin`'s
    // rule 1).
    let boxed = holding_here && in_box(&travel.graph.boxes, at, car.len);
    if !reverse
        && !boxed
        && let Some(o) = others.iter().find(|o| {
            if !o.free(&me) {
                return false;
            }
            if traffic.waits_for(o.first).contains(&key) {
                let (h, way) = (me.heading(), o.heading());
                return h[0] * way[0] + h[1] * way[1] < -ONCOMING
                    && closes_in(&body_now, &body_next, &o.body());
            }
            too_near((at, yaw, car.len, car.wid), (o.at, o.yaw, o.len, o.wid))
                && !too_near(
                    (was, was_yaw, car.len, car.wid),
                    (o.at, o.yaw, o.len, o.wid),
                )
        })
    {
        car.waits_for = Some(o.first);
        return;
    }
    // Nor into the ground one turning round still backs up over (`unjam`): it has the way; but
    // for one waiting for this one (its body in that one's way), which goes on by, never nearer
    // than `GAP` (each kept off the other, one backing up and one it had turned round for or
    // turning round itself stood there for good: Elm Park, Willowbrook). Even so not nearer the
    // ground past where that one stands now, still to back over: going on by is past its front,
    // not cutting in behind its tail (a taxi came by one backing round a junction's turn, cut in
    // behind it into its turn and stood there, each waiting for the other for good, Willowby).
    // Nor, going the way it backs, on past it (`overtakes`): crept along beside the ground it backs
    // over, a car came out ahead of it into its way.
    if let Some(k) = traffic.turning.iter().find_map(|(k, path)| {
        let on = if waiting.contains(k) {
            path.iter()
                .position(|r| path[0].overlap(r) <= 0.0)
                .unwrap_or(path.len())
        } else {
            0
        };
        (*k != key
            && (overtakes(path[0], &body_now, &body_next, me.heading())
                || path[on..].iter().any(|r| {
                    crate::travel::dist(r.at, body_next.at) < r.half[0] + body_next.half[0] + 1.0
                        && {
                            // (Kept `TURN_KEEP` off it, so the one backing up comes by clear.)
                            let after = body_next.gap(r);
                            after < GAP + TURN_KEEP && after < body_now.gap(r) - 1e-3
                        }
                })))
        .then_some(*k)
    }) {
        car.waits_for = Some(k);
        return;
    }
    let nav = crate::path::NavGrid {
        lot,
        blocked,
        stairs,
        kept_off: &[],
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

/// Car `c` out of sight (`Sight::Ghost`) a tick on along its way (`home`: back): through anyone,
/// claiming nothing; back into its lane as it goes.
fn drive_through(w: &mut World, c: usize, home: bool) {
    let World {
        cars,
        exits,
        travel,
        ..
    } = w;
    let car = &mut cars[c];
    car.unhold();
    let t = car.progress;
    let shift = car.shift;
    let route = car.route(exits, Some(travel), home);
    let ticks = route.ticks();
    let shift = (shift - PULL_RATE).min(route.room_at(t + 1.0)).max(0.0);
    let (p, yaw, _) = route.at(t + 1.0);
    let at = shifted(p, yaw, shift);
    car.shift = shift;
    car.next = Some((at, yaw));
    car.progress = t + 1.0;
    car.moved = true;
    if car.progress < ticks {
        car.shown = Some((at, yaw, true, 0));
    }
}

/// A work taxi whose run is over, given up in sight (`Car::off_run`), a tick on: it turns round
/// (`turn_round`, as one held up does, tried each `UNJAM_AGAIN`) and drives out of town, where
/// it's parked off the map (`Car::park_taxi`).
fn drive_off(w: &mut World, c: usize) {
    let tick = w.tick;
    if w.cars[c].detour.is_none()
        && (tick.is_multiple_of(UNJAM_AGAIN) || w.cars[c].stall.1 + 1 >= tick)
        && let Ok((_, d)) = turn_round(w, c)
    {
        turn_round_by(w, c, d);
    }
    drive_taxi(w, c);
    if w.cars[c].progress >= w.cars[c].taxi_ticks() {
        w.cars[c].park_taxi();
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
    // (Standing, it holds no claim but the one it stands in: none on ahead.)
    let t = w.cars[c].progress;
    w.traffic.stand(car_first(c), t);
    let car = &mut w.cars[c];
    car.standing = true;
    car.waits_for = None;
    // (Where it stands, for its doors and seats: `Car::door`, `getting`.)
    if let Some((_, route)) = car.route.as_ref() {
        let (p, yaw, _) = route.at(car.progress);
        car.spot = shifted(p, yaw, car.shift);
        car.yaw = yaw;
    }
    // (Out of sight, back in its lane: nobody to pull over for.)
    if car.sight != Sight::Seen {
        car.unhold();
        car.shift = (car.shift - PULL_RATE).max(0.0);
        return;
    }
    let others = vehicles(&w.cars, Some(c));
    pull_over(w, c, &others, false, None);
}

/// How long a car and one in its way wait for each other before it backs up (ticks of drive time,
/// `back_off`), and how far back it goes at most (ticks along its route, a tick back each tick).
const JAM_TICKS: u32 = drive_secs(5.0) as u32;
const JAM_BACK: u32 = drive_secs(3.0) as u32;

/// Whether car `k` (its `Driving::first`) waited for car `me` last tick, so that car `me`, waiting
/// for it, should back up (`back_off`): of two cars, the one not holding a claim (holding one,
/// the other was let through where it waits: it's that one that's in the way); else the later.
fn waits_back(w: &World, k: u64, me: u64) -> bool {
    let o = (k & !(1 << 63)) as usize;
    let (mine, theirs) = (w.traffic.holding(me), w.traffic.holding(k));
    // (Turning out of its driveway, it has its way: the other backs up to give it room to finish
    // its turn.)
    let (out, their_out) = (turning_out(w, me), turning_out(w, k));
    // (Of two not holding one, the one waiting its turn for a claim the other is in the way of:
    // that one can't go on out of its way till it's backed up out of that one's.)
    let (asks, their_asks) = (
        !w.traffic.waits_for(me).is_empty(),
        !w.traffic.waits_for(k).is_empty(),
    );
    // (Of the two, the one that can still back up: the other can't, or no further.)
    let mine_to_go = if out != their_out {
        their_out
    } else if mine != theirs {
        theirs
    } else if !mine && asks != their_asks {
        asks
    } else {
        me > k
    };
    // (One backing up out of the other's way already goes on: not the other in turn as their
    // claims change, the one going on by letting go of its claim behind it, say; nor one, then the
    // other, a tick each.)
    let backing = |x: u64| {
        w.cars
            .get((x & !(1 << 63)) as usize)
            .is_some_and(|c| c.jam.1 > 0)
            && !stuck_back(w, x)
    };
    let mine_to_go = match (backing(me), backing(k)) {
        (true, false) => true,
        (false, true) => false,
        _ => mine_to_go,
    };
    let mine_to_go = if mine_to_go {
        !stuck_back(w, me)
    } else {
        stuck_back(w, k) && !stuck_back(w, me)
    };
    mine_to_go
        && (w.cars.get(o).is_some_and(|c| c.waits_for == Some(me))
            || w.traffic.waits_for(k).contains(&me))
}

/// Whether car `k` (its `Driving::first`) is a household's car still turning out of its driveway
/// onto the road: in its route's first mouth (`Route::mouths`).
fn turning_out(w: &World, k: u64) -> bool {
    w.cars.get((k & !(1 << 63)) as usize).is_some_and(|c| {
        !c.taxi
            && c.state == CarState::Leaving
            && c.route.as_ref().is_some_and(|(_, r)| {
                r.mouths
                    .first()
                    .is_some_and(|&(a, b)| a <= 0.0 && c.progress <= b)
            })
    })
}

/// Whether car `k` (its `Driving::first`) can back up no further out of the way of one it holds
/// up as that one holds it up (`back_off`, `waits_back`).
fn stuck_back(w: &World, k: u64) -> bool {
    w.cars
        .get((k & !(1 << 63)) as usize)
        .is_some_and(|c| c.jam.2)
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
    // (Turned round, still on the stretch it backs up along, its last fraction of a tick before it
    // drives on: a tick back along it is forward, the way it backed, and there it's turning round
    // again (`drive_on`), with no jam rule of its own: it'd neither back up nor say it can't, and
    // the other would wait for it for good. It can back up no further: the other does.)
    let turned = car.detour.is_some() && route.at(t).2;
    if to >= t || route.at(to).2 != route.at(t).2 || turned {
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
    // (Not back into a junction's box it's out of: it holds no claim to it, and standing there it
    // would hold up those taking their turn through it, one it's held up with among them: they'd
    // wait for each other for good, Rowan Hollow.)
    let boxes = &w.travel.graph.boxes;
    if in_box(boxes, next.at, car.len) && !in_box(boxes, now.at, car.len) {
        return false;
    }
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
    car.backed_for = partner;
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
/// on the road and parked) it would come nearer than `GAP` to, and nearer than it is now (one it's
/// already that near, from before, it may draw away from or keep beside, never nearer). None: it
/// may move.
fn guard<'a>(now: &Rect, next: &Rect, them: impl Iterator<Item = &'a Driving>) -> Option<u64> {
    let near = |at: [f32; 2], len: f32| {
        crate::travel::dist(at, next.at) < (len + next.half[0] * 2.0) + AHEAD + GAP
    };
    them.into_iter()
        .find(|o| near(o.at, o.len) && closes_in(now, next, &o.body()))
        .map(|o| o.first)
}

/// Whether a car going `heading`, its move from body `now` to `next`, goes on past one turning
/// round as it backs (`turning`, where it is now, `Traffic::turning`): going the way that one
/// backs, beside it or behind it (within a lane or so either side), its front further past that
/// one's nose and nearer than `GAP` behind it or beyond. (Ahead of it already, it's on its way out
/// of the way: `drive_on`'s keep off the ground still backed over.)
fn overtakes(turning: Rect, now: &Rect, next: &Rect, heading: [f32; 2]) -> bool {
    let back = [-turning.yaw.sin(), -turning.yaw.cos()];
    let rel = |r: &Rect| sub2(r.at, turning.at);
    let (d_now, d_next) = (rel(now), rel(next));
    let front = |d: [f32; 2], r: &Rect| dot2(d, back) + r.half[0];
    let nose = -turning.half[0] - GAP;
    dot2(heading, back) > 0.5
        && cross2(back, d_next).abs() < turning.half[1] + next.half[1] + 2.0 * LANE
        && dot2(d_now, back) < 0.0
        && front(d_next, next) > nose
        && front(d_next, next) > front(d_now, now) + 1e-3
}

/// Whether a car's move from body `now` to `next` brings it nearer `o` than `GAP`, and nearer than
/// it is (`guard`).
fn closes_in(now: &Rect, next: &Rect, o: &Rect) -> bool {
    let after = next.gap(o);
    after < GAP - 1e-3 && after < now.gap(o) - 1e-3
}

/// Car `c` a tick nearer pulled over as far as it would be (`pull_want`), `PULL_RATE` a tick:
/// back towards its lane only clear of those coming the other way (`too_near`), and never
/// into another (`guard`). Where its room runs out it's back in its lane.
fn pull_over(w: &mut World, c: usize, others: &[Driving], moving: bool, waited: Option<u64>) {
    // (Pulled over for one that waits for it, it pulls over only as far as it can on its way: not
    // for one it waits for in turn (`waited`, or its turn after it), held up by it there, which it
    // would pull back in into the way of, unless that one has backed up clear of it to give it the
    // way.)
    let me = car_first(c);
    // (Not one backed up out of its way, clear of it now by more than it's pulled over and the
    // gap the cars keep: that one gives it the way, and it pulls back in to go on.)
    let body = |car: &Car| {
        car.shown.map(|s| Rect {
            at: s.0,
            yaw: s.1,
            half: [car.len / 2.0, car.wid / 2.0],
        })
    };
    let backed = |k: u64| {
        w.cars.get((k & !(1 << 63)) as usize).is_some_and(|o| {
            o.jam.1 > 0
                && body(o)
                    .zip(body(&w.cars[c]))
                    .is_some_and(|(a, b)| a.gap(&b) > w.cars[c].shift + GAP)
        })
    };
    let after = |k: u64| (waited == Some(k) || w.traffic.waits_for(me).contains(&k)) && !backed(k);
    let waits_me = |k: u64| {
        (w.traffic.waits_for(k).contains(&me)
            || w.cars
                .get((k & !(1 << 63)) as usize)
                .is_some_and(|o| o.waits_for == Some(me)))
            && !after(k)
    };
    let held = w.cars[c].pulled_for.is_some_and(waits_me);
    let (mut want, mut whom) = pull_want(w, c, moving || held, &after);
    if !(moving || held) && whom.is_some_and(waits_me) {
        (want, whom) = pull_want(w, c, true, &after);
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
    // (Where it is now, for those after it this tick, as `drive_on` has it once on its way.)
    let at = body(shift).at;
    let car = &mut w.cars[c];
    car.shift = shift;
    if let Some(s) = car.shown.as_mut() {
        s.0 = at;
    }
}

/// How far right of its lane car `c` would be pulled over (m) to pass those coming the other way in
/// front of it (`PULL_AHEAD`) where the lane is too narrow for both in their lanes: half the
/// way past another car (it pulls over the other half, as far as it can: `Route::room`); as far as
/// its room ahead allows (back in its lane before the room runs out). Standing, as far as it can
/// for one it waits for (`after`).
fn pull_want(w: &World, c: usize, moving: bool, after: &dyn Fn(u64) -> bool) -> (f32, Option<u64>) {
    let car = &w.cars[c];
    let Some((_, route)) = car.route.as_ref() else {
        return (0.0, None);
    };
    let t = car.progress;
    // (On the move, back in its lane before its room runs out; standing, as far as it has room
    // where it stands.)
    let room = if moving {
        [0.0, 1.0, drive_ticks(0.5), drive_ticks(1.0)]
            .into_iter()
            .map(|k| route.room_at(t + k))
            .fold(f32::INFINITY, f32::min)
    } else {
        route.room_here(t)
    };
    let (at, yaw, back) = route.at(t);
    // (In a junction's box only for one coming the other way beside it now, their bodies nearer
    // than `GAP` and `PASSING`: there they take turns, `traffic.rs`, and turning there it faces
    // any way, those following it in too; but two let in together pass each other in their lanes,
    // and round a bend there their ends swing across towards each other further than the claims'
    // samples have it: neither could go on, `guard`. For one coming the other way into it, as
    // anywhere: the claims let that one in counting on this one's pulling over.)
    let boxed = in_box(&w.travel.graph.boxes, at, car.len);
    if room <= 0.0 || back {
        return (0.0, None);
    }
    let body = |p: [f32; 2], yaw: f32, len: f32, wid: f32| Rect {
        at: p,
        yaw,
        half: [len / 2.0, wid / 2.0],
    };
    let mine = body(at, yaw, car.len, car.wid);
    let short = |p: [f32; 2], facing: f32, len: f32, wid: f32| {
        let m = passing((at, yaw, car.len, car.wid), (p, facing, len, wid))?;
        let ends = (car.len + len) / 2.0;
        let ahead = if boxed { 0.0 } else { PULL_AHEAD };
        if m.along < -ends || m.along > ends + ahead || m.apart < 0.0 {
            return None;
        }
        // (Beside it, as near as their bodies are: on a bend nearer than across their ways.)
        let gap = m.gap.min(mine.gap(&body(p, facing, len, wid)));
        if boxed && gap >= GAP + PASSING {
            return None;
        }
        Some(((PASSING + PULL_MARGIN - gap).max(0.0), m.mine, m.theirs))
    };
    let (mut want, mut whom): (f32, Option<u64>) = (0.0, None);
    for (o, other) in w.cars.iter().enumerate() {
        // (Standing, waiting for that one (`after`: its turn after it too), it pulls over as far as
        // it can, that one backing out of its driveway too: that one goes by, as it comes on round
        // a bend or out of a driveway.)
        let after = !moving && !boxed && after(car_first(o));
        // (A taxi standing at the kerb for its riders too, once it waits for it: it pulls over as
        // far as it can to get by, `stand_taxi`.)
        let standing_taxi = other.taxi && other.standing && after;
        if o == c
            || other.shown.is_none_or(|s| !s.2) && !standing_taxi
            || other.backing() && !after
            || other.sight != Sight::Seen
        {
            continue;
        }
        let Some((_, theirs)) = other.route.as_ref() else {
            continue;
        };
        let (p, facing, _) = theirs.at(other.progress);
        if let Some((all, mine, their)) = short(p, facing, other.len, other.wid) {
            let share = if after {
                f32::INFINITY
            } else {
                (all / 2.0).max(all - theirs.room_at(other.progress) * their)
            };
            if share / mine > want {
                (want, whom) = (share / mine, Some(car_first(o)));
            }
        }
    }
    (want.min(room), whom)
}

/// Whether a body `len` long with its middle at `p` is in one of the junctions' `boxes`, about.
fn in_box(boxes: &[([f32; 2], f32)], p: [f32; 2], len: f32) -> bool {
    boxes
        .iter()
        .any(|&(m, r)| crate::travel::dist(m, p) < r + len / 2.0)
}

/// Which cars on their way are in sight this tick (`Car::sight`, before the claims: `traffic::begin`;
/// see the module notes). With a watch (`World::set_watch`), one whose body is in the seen area
/// (the watch and `SEEN_MARGIN` round it), now or soon (`COMING_IN`, or a claim of its reaching
/// into it), is seen if it was (and till it's `SEEN_SLACK` out of it); one coming into it is seen
/// once clear of every car seen (`GAP`; those in sight first, then the others in turn as they are
/// now), else it waits outside for the one it isn't clear of (`Sight::Wait`), or, the seen area
/// come to it where it touches one, is put where it's clear (`moved_clear`). The rest are out
/// of sight (`Sight::Ghost`); their claims are let go. Without a watch the seen area is the whole
/// town.
pub(crate) fn sight(w: &mut World) {
    // (No watch: every car in sight, as it always was; but one out of sight on top of another
    // still till it's clear of them.)
    if w.watch.is_none() && w.cars.iter().all(|c| c.sight == Sight::Seen) {
        return;
    }
    let m = SEEN_MARGIN;
    let area = w
        .watch
        .map(|[x0, z0, x1, z1]| (x0 as f32 - m, z0 as f32 - m, x1 as f32 + m, z1 as f32 + m));
    let near = |p: [f32; 2], reach: f32| {
        area.is_none_or(|(x0, z0, x1, z1)| {
            let dx = (x0 - p[0]).max(p[0] - x1).max(0.0);
            let dz = (z0 - p[1]).max(p[1] - z1).max(0.0);
            dx.hypot(dz) <= reach
        })
    };
    // (How far on along its way each may be put, `moved_clear`.)
    let stops = crate::taxi::stops_ahead(w);
    // (Those in sight first, which stay or go; then those out of it, coming in clear of them as
    // they are now.)
    let order: Vec<usize> = (0..w.cars.len())
        .filter(|&c| w.cars[c].sight == Sight::Seen)
        .chain((0..w.cars.len()).filter(|&c| w.cars[c].sight != Sight::Seen))
        .collect();
    for c in order {
        let most = stops[c];
        if !matches!(w.cars[c].state, CarState::Leaving | CarState::Arriving) {
            w.cars[c].sight = Sight::Seen;
            continue;
        }
        let home = w.cars[c].state == CarState::Arriving;
        let tick = w.tick;
        let World {
            cars,
            exits,
            travel,
            ..
        } = &mut *w;
        let car = &mut cars[c];
        let (t, shift, half) = (car.progress, car.shift, [car.len / 2.0, car.wid / 2.0]);
        let (was, shown) = (car.sight, car.shown);
        // (Its corners and the ground in front of it.)
        let reach = half[0] + half[1] + AHEAD;
        let route = car.route(exits, Some(travel), home);
        let (p, yaw, _) = route.at(t + 1.0);
        let next = Rect {
            at: shifted(p, yaw, shift),
            yaw,
            half,
        };
        let now = shown.map(|s| Rect {
            at: s.0,
            yaw: s.1,
            half,
        });
        // (In sight, it stays so till `SEEN_SLACK` further out.)
        let reach = if was == Sight::Seen {
            reach + SEEN_SLACK
        } else {
            reach
        };
        let in_now = now.is_some_and(|r| near(r.at, reach));
        // (Coming into sight, it's under the rules a little way short of it, `COMING_IN`: up to a
        // junction there taking turns, not cutting in part way across it. So too short of a claim
        // of its, there or that near, that reaches into sight: it asks for it as anyone there,
        // not driving through on into sight.)
        let last = COMING_IN[COMING_IN.len() - 1];
        let reaches = |k: &crate::traffic::Claim| {
            let (a, b) = (k.from.max(t), k.to);
            let n = ((b - a) / SIGHT_STEP).ceil().max(1.0) as usize;
            (0..=n).any(|i| near(route.at(a + (b - a) * i as f32 / n as f32).0, reach))
        };
        let coming = COMING_IN.iter().any(|&k| near(route.at(t + k).0, reach))
            || route
                .claims
                .iter()
                .filter(|k| k.to > t && k.from <= t + last)
                .any(reaches);
        let mut moved = None;
        let sight = if was == Sight::Seen {
            if in_now || coming {
                Sight::Seen
            } else {
                Sight::Ghost
            }
        } else if !in_now && !coming {
            Sight::Ghost
        } else {
            let on = |o: &Car| {
                o.shown.map(|s| Rect {
                    at: s.0,
                    yaw: s.1,
                    half: [o.len / 2.0, o.wid / 2.0],
                })
            };
            let seen = || {
                cars.iter()
                    .enumerate()
                    .filter(|&(o, other)| o != c && other.sight == Sight::Seen)
                    .filter_map(|(o, other)| Some((car_first(o), on(other)?)))
            };
            // (On top of one in sight where it is, or touching it, the seen area come to it (the
            // camera moved there, nobody has seen it yet): it's put back along its way, or on, to
            // the nearest place clear of those in sight (`moved_clear`), and comes into sight
            // there. Coming in, it waits outside till its way in is clear by `GAP`.)
            let on_top = now.is_some_and(|r| seen().any(|(_, b)| b.gap(&r) < TOUCH));
            let by = seen()
                .find(|(_, b)| b.gap(&next) < GAP || now.is_some_and(|r| b.gap(&r) < GAP))
                .map(|(k, _)| k);
            match by {
                Some(_) if in_now && on_top => {
                    let clear = |r: &Rect| {
                        seen().all(|(_, b)| {
                            crate::travel::dist(b.at, r.at) > b.half[0] + r.half[0] + GAP + 0.5
                                || b.gap(r) >= GAP
                        })
                    };
                    match cars[c]
                        .route
                        .as_ref()
                        .and_then(|(_, r)| moved_clear(r, t, half, &[-1.0, 1.0], most, &clear))
                    {
                        Some(to) => {
                            moved = Some(to);
                            Sight::Seen
                        }
                        None => Sight::Ghost,
                    }
                }
                // (Without a watch the way into town is in sight, where it waits its turn,
                // `room_to_come_in`.)
                Some(k) if !in_now && area.is_some() => Sight::Wait(k),
                // (Its way in held up just inside: out of sight nobody waits, so those coming
                // would all come in at once. It waits outside till the queue there has room for
                // it, as the rules would have let it come, `METER`.)
                None if !in_now && area.is_some() => {
                    let bodies: Vec<(u64, Rect)> = cars
                        .iter()
                        .enumerate()
                        .filter(|&(o, k)| {
                            o != c
                                && k.sight == Sight::Seen
                                && matches!(k.state, CarState::Leaving | CarState::Arriving)
                                && (k.standing || tick >= k.stall.1 + METER_STILL)
                        })
                        .filter_map(|(o, k)| Some((car_first(o), on(k)?)))
                        .collect();
                    let route = &cars[c].route.as_ref().expect("its way").1;
                    // (Its place on its way in, up to the first junction it takes turns at in
                    // sight: the queue for it.)
                    let upto = route
                        .claims
                        .iter()
                        .map(|k| k.from)
                        .find(|&f| f > t && near(route.at(f).0, 0.0))
                        .unwrap_or(t + METER.ahead)
                        .min(t + METER.most);
                    let full = (0..)
                        .map(|i| t + 1.0 + i as f32 * SIGHT_STEP)
                        .take_while(|&s| s < upto)
                        .find_map(|s| {
                            let (p, yaw, _) = route.at(s);
                            let r = Rect {
                                at: shifted(p, yaw, shift),
                                yaw,
                                half,
                            };
                            bodies
                                .iter()
                                .find(|(_, b)| b.gap(&r) < GAP + METER.gap)
                                .map(|(k, _)| *k)
                        });
                    // (Not for good: waited `METER_MOST`, it comes in as it would have.)
                    let since = *cars[c].metered.get_or_insert(tick);
                    match full {
                        Some(k) if tick < since + METER_MOST => Sight::Wait(k),
                        _ => Sight::Seen,
                    }
                }
                _ => Sight::Seen,
            }
        };
        if let Some(to) = moved {
            let car = &mut w.cars[c];
            car.progress = to;
            car.shift = 0.0;
            car.unhold();
            if let Some((_, r)) = car.route.as_ref() {
                let (p, yaw, _) = r.at(to);
                car.shown = Some((p, yaw, true, 0));
                car.next = Some((r.at(to + 1.0).0, r.at(to + 1.0).1));
            }
        }
        if !matches!(sight, Sight::Wait(_)) {
            w.cars[c].metered = None;
        }
        if sight != Sight::Seen && was == Sight::Seen {
            w.traffic.forget(car_first(c));
        }
        w.cars[c].sight = sight;
    }
    unstick(w);
    // (One in sight waits for, stays backed up for or pulled over for nobody out of it.)
    let out: Vec<bool> = w.cars.iter().map(|c| c.sight != Sight::Seen).collect();
    let gone =
        |k: &Option<u64>| k.is_some_and(|k| out.get((k & !(1 << 63)) as usize) == Some(&true));
    for car in w.cars.iter_mut().filter(|c| c.sight == Sight::Seen) {
        if gone(&car.waits_for) {
            car.waits_for = None;
        }
        if gone(&car.backed_for) {
            car.backed_for = None;
        }
        if gone(&car.pulled_for) {
            car.pulled_for = None;
        }
    }
}

/// How far along `route` (ticks) a car `half` its length and width, `t` along it, is put to come
/// into sight clear of those in sight (`clear`, now and a tick on), the camera come to it on top of
/// one of them (`sight`): the nearest such place back along its way, else on along it, up to
/// `MOVED_MOST` either way, out of its claims (part way through one, it would cut in), and no further on than `most` (a
/// work taxi's next stop, `taxi::stops_ahead`: put on past it, it stood there mid-turn). None:
/// none.
fn moved_clear(
    route: &Route,
    t: f32,
    half: [f32; 2],
    ways: &[f32],
    most: f32,
    clear: &dyn Fn(&Rect) -> bool,
) -> Option<f32> {
    let ticks = route.ticks();
    let claiming = |s: f32| route.claims.iter().any(|k| k.from + 1e-3 < s && s < k.to);
    let body = |s: f32| {
        let (at, yaw, _) = route.at(s);
        Rect { at, yaw, half }
    };
    (1..=MOVED_MOST)
        .flat_map(|d| ways.iter().map(move |k| t + k * d as f32))
        .filter(|&s| s >= 1.0 && s + 1.0 < ticks && s <= most)
        .find(|&s| !claiming(s) && clear(&body(s)) && clear(&body(s + 1.0)))
}

/// Cars held up in the seen area but out of the picture (further than `UNSEEN` from the watch, the
/// renderer's: `World::set_watch`), with all they wait for, standing still `STUCK_UNSEEN`: each is
/// put on along its way to the nearest place clear of those in sight (`moved_clear`), out of the
/// picture too and out of its claims, letting go of those it held. Nobody sees it go: a hold-up
/// nobody can see is gone, as one out of sight (`sight`), but no car drives through another where
/// the camera could come any moment.
fn unstick(w: &mut World) {
    let Some([x0, z0, x1, z1]) = w.watch else {
        return;
    };
    let tick = w.tick;
    let unseen = |p: [f32; 2]| {
        let dx = (x0 as f32 - p[0]).max(p[0] - x1 as f32).max(0.0);
        let dz = (z0 as f32 - p[1]).max(p[1] - z1 as f32).max(0.0);
        dx.hypot(dz) > UNSEEN
    };
    let on_way = |c: &Car| {
        matches!(c.state, CarState::Leaving | CarState::Arriving) && c.sight == Sight::Seen
    };
    let stuck = |c: &Car| {
        on_way(c) && tick >= c.stall.1 + STUCK_UNSEEN && c.shown.is_some_and(|s| s.2 && unseen(s.0))
    };
    let mut out = Vec::new();
    for c in 0..w.cars.len() {
        if !stuck(&w.cars[c]) {
            continue;
        }
        // (All it waits for, however far round, as held up and as far out of the picture.)
        let mut seen = vec![car_first(c)];
        let mut todo = vec![car_first(c)];
        let mut all = true;
        while let Some(k) = todo.pop() {
            for o in crate::traffic::waits_on(w, k) {
                if o < 1 << 63 || seen.contains(&o) {
                    continue;
                }
                let car = &w.cars[(o & !(1 << 63)) as usize];
                if !stuck(car) {
                    all = false;
                }
                seen.push(o);
                todo.push(o);
            }
        }
        if all {
            out.push(c);
        }
    }
    let stops = crate::taxi::stops_ahead(w);
    for c in out {
        let body = |o: &Car| {
            o.shown.map(|s| Rect {
                at: s.0,
                yaw: s.1,
                half: [o.len / 2.0, o.wid / 2.0],
            })
        };
        let others: Vec<Rect> = w
            .cars
            .iter()
            .enumerate()
            .filter(|&(o, k)| o != c && k.sight == Sight::Seen)
            .filter_map(|(_, k)| body(k))
            .collect();
        let car = &w.cars[c];
        let half = [car.len / 2.0, car.wid / 2.0];
        let clear = |r: &Rect| {
            unseen(r.at)
                && others.iter().all(|b| {
                    crate::travel::dist(b.at, r.at) > b.half[0] + r.half[0] + GAP + 0.5
                        || b.gap(r) >= GAP
                })
        };
        let Some(to) = car
            .route
            .as_ref()
            .and_then(|(_, r)| moved_clear(r, car.progress, half, &[1.0], stops[c], &clear))
        else {
            continue;
        };
        w.traffic.forget(car_first(c));
        w.traffic.unstuck += 1;
        let car = &mut w.cars[c];
        car.progress = to;
        car.shift = 0.0;
        car.unhold();
        car.stall = (to, tick);
        if let Some((_, r)) = car.route.as_ref() {
            let (p, yaw, _) = r.at(to);
            car.shown = Some((p, yaw, true, 0));
            let (q, qy, _) = r.at(to + 1.0);
            car.next = Some((q, qy));
        }
    }
}

/// Where each car on its way is furthest along it in sight and since when (`Car::stall`), before
/// the claims each tick (`unstick`, `unjam`): standing there since, it's held up.
pub(crate) fn track_stall(w: &mut World) {
    let tick = w.tick;
    for car in w.cars.iter_mut() {
        let going = matches!(car.state, CarState::Leaving | CarState::Arriving);
        if !going {
            car.detour = None;
            car.let_by = None;
        }
        let on_way = going && car.sight == Sight::Seen;
        // (Well back along its way: a new way, begun afresh.)
        let restarted = car.progress < car.stall.0 - drive_ticks(10.0);
        if !on_way || car.progress > car.stall.0 + 1e-3 || restarted {
            car.stall = (car.progress, tick);
        }
    }
}

/// How far ahead along its way (ticks of drive time) a car in sight is looked along for a work taxi
/// standing at the kerb in its lane, or alongside one the other way (`pass_stands`); one further
/// off than `PASS_FAR` (m: well past where a car gets in `PASS_LOOK` at any speed) isn't.
const PASS_LOOK: f32 = drive_ticks(45.0);
const PASS_FAR: f32 = 900.0;
/// How far apart (ticks: a tenth of a second of drive time) a way past a standing taxi is looked
/// along (`leg_past`, `span_by`), and how far on from where it comes up against it its middle is
/// looked for alongside the taxi's (ticks: 4 s of drive time).
const PASS_STEP: f32 = drive_ticks(0.1);
const PASS_ALONGSIDE: usize = drive_secs(4.0) as usize;
/// Passing a work taxi standing at the kerb in its lane (`PassBy`): how far clear of it its body
/// keeps alongside (m), how far behind it and past it (m) it's out of its lane all the way, and
/// how far it eases out before and back in after (m; out, `PASS_EASE_LEAST` at least, right
/// behind it).
const PASS_SIDE: f32 = 0.35;
const PASS_ENDS: f32 = 0.5;
const PASS_EASE: f32 = 8.0;
const PASS_EASE_LEAST: f32 = 3.0;
/// How long a car waits for its way round a work taxi standing in its lane, or behind one with
/// no way round it, before it turns round as one held up by a car parked in its way (ticks,
/// `unjam`; drive time).
const PASS_WAIT: u64 = drive_secs(3.0);

/// How much of its way out round a standing taxi a car is at `t` ticks along (`PassBy`: 0 in its
/// lane, 1 out alongside it), easing out and back in.
fn pass_share(pass: &PassBy, t: f32) -> f32 {
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
}

/// Work taxis standing at the kerb in sight (`stand_taxi`), before the claims each tick: each
/// such stretch of road is narrow for a while, taken in turns. A car in sight on its way up behind
/// one in its lane goes out round it into the other lane and back in past it (`PassBy`, laid on
/// its way: `Route::lay_leg`), a claim of its own; one coming the other way alongside it takes its
/// turn for that stretch (`Route::spans`), begun short of it as a narrow stretch's is: they go
/// through in the order they came, as on a 6 m lane's tight bend. Once the taxi pulls away, a car
/// not yet out round it keeps to its lane, and none takes turns there. (No way round it, or none
/// clear for `PASS_WAIT`: it turns round, `unjam`.) The cars are looked at again as the taxis
/// standing change (`Traffic::stand_gen`), or their ways do.
pub(crate) fn pass_stands(w: &mut World) {
    let stands: Vec<(usize, Rect)> = w
        .cars
        .iter()
        .enumerate()
        .filter(|(_, k)| {
            k.taxi
                && k.standing
                && k.sight == Sight::Seen
                && matches!(k.state, CarState::Leaving | CarState::Arriving)
        })
        .filter_map(|(o, k)| {
            // (Where it stands in its lane, as it stood still: not as it pulls over for one coming
            // the other way.)
            let (_, route) = k.route.as_ref()?;
            let (p, yaw, _) = route.at(k.progress);
            Some((
                o,
                Rect {
                    at: p,
                    yaw,
                    half: [k.len / 2.0, k.wid / 2.0],
                },
            ))
        })
        .collect();
    let now: Vec<(u64, [f32; 2])> = stands.iter().map(|(o, r)| (car_first(*o), r.at)).collect();
    if w.traffic.stand_gen == 0 || now != w.traffic.stands {
        w.traffic.stand_gen += 1;
        w.traffic.stands = now;
    }
    let now_gen = w.traffic.stand_gen;
    let mut passing = Vec::new();
    for c in 0..w.cars.len() {
        let car = &w.cars[c];
        if !matches!(car.state, CarState::Leaving | CarState::Arriving)
            || car.sight != Sight::Seen
            || car.standing
        {
            continue;
        }
        let Some((home, route)) = car.route.as_ref() else {
            continue;
        };
        let (home, t) = (*home, car.progress);
        let mut lay = None;
        let (mut unlay, mut drop) = (false, false);
        // Its way round one laid already: kept while it's on it or the taxi stands; laid again on
        // its way worked out again (a game loaded) where it was made on it.
        if let Some(pass) = car.pass_by {
            let fits = pass.home == home
                && match route.leg.as_ref() {
                    Some(l) => l.pass == pass,
                    None => crate::travel::dist(route.at(pass.from).0, pass.at) < 0.3,
                };
            let gone = !stands.iter().any(|(o, _)| *o as u32 == pass.taxi);
            if !fits || t > pass.to {
                drop = true;
            } else if gone && t < pass.from {
                unlay = true;
            } else {
                if route.leg.is_none() {
                    lay = Some(pass);
                }
                passing.push((car_first(c), car_first(pass.taxi as usize)));
            }
        }
        let fresh = route.stand_gen != now_gen;
        let spans = if fresh {
            stands
                .iter()
                .filter(|(o, _)| *o != c)
                .filter_map(|(_, r)| span_by(route, t, car.len, r))
                .collect()
        } else {
            route.spans.clone()
        };
        if fresh && (car.pass_by.is_none() || drop) && !unlay {
            lay = stands
                .iter()
                .filter(|(o, _)| *o != c)
                .find_map(|&(o, r)| leg_past(w, c, home, o, &r));
            if let Some(p) = lay {
                passing.push((car_first(c), car_first(p.taxi as usize)));
            }
        }
        let World {
            cars,
            travel,
            traffic,
            ..
        } = &mut *w;
        let car = &mut cars[c];
        let Some((_, route)) = car.route.as_mut() else {
            continue;
        };
        route.stand_gen = now_gen;
        if drop {
            car.pass_by = None;
        }
        let mut changed = spans != route.spans;
        route.spans = spans;
        if unlay {
            route.unlay_leg();
            car.pass_by = None;
            changed = true;
        }
        if let Some(p) = lay {
            route.unlay_leg();
            if route.lay_leg(p) {
                car.pass_by = Some(p);
            } else {
                car.pass_by = None;
                passing.retain(|x| x.0 != car_first(c));
            }
            changed = true;
        }
        // (Its claims worked out again: what it held and asked for let go, taken again as it goes
        // on, `traffic::Traffic::may_go`.)
        if changed {
            car.reclaim(travel);
            traffic.forget(car_first(c));
        }
    }
    w.traffic.passing = passing;
}

/// Car `c`'s way out round work taxi `o` standing at the kerb (its body `taxi`) in its lane on its
/// way (`home`) ahead (`PassBy`): out into the other lane, clear of it by `PASS_SIDE`, from
/// `PASS_ENDS` behind it to as far past it, easing out and back in over `PASS_EASE`. None: the taxi
/// isn't in its lane ahead, or there's no such way: too near behind it to ease out, a junction's
/// box, its driveway or a stop of its on it, off the paving, or not clear of another standing.
fn leg_past(w: &World, c: usize, home: bool, o: usize, taxi: &Rect) -> Option<PassBy> {
    let car = &w.cars[c];
    let route = &car.route.as_ref()?.1;
    let t = car.progress;
    let half = [car.len / 2.0, car.wid / 2.0];
    let end = route.ticks().min(t + PASS_LOOK);
    if t + 2.0 >= end || crate::travel::dist(route.at(t).0, taxi.at) > PASS_FAR {
        return None;
    }
    // (Ticks along it from here a tick apart, and how far along that is, m.)
    let mut ts = vec![t];
    let mut ds = vec![0.0f32];
    let mut s = t;
    while s + 1.0 < end {
        let a = route.at(s).0;
        s += 1.0;
        ts.push(s);
        ds.push(ds[ds.len() - 1] + crate::travel::dist(a, route.at(s).0));
    }
    let body = |s: f32, off: f32| {
        let (p, yaw, _) = route.at(s);
        Rect {
            at: shifted(p, yaw, off),
            yaw,
            half,
        }
    };
    // (Where it would come up against it, going its way in its lane.)
    let near = half[0] + taxi.half[0] + 1.0;
    let hit = (0..ts.len()).find(|&k| {
        let b = body(ts[k], 0.0);
        crate::travel::dist(b.at, taxi.at) < near && b.gap(taxi) < PASS_SIDE
    })?;
    let (_, yaw, back) = route.at(ts[hit]);
    let way = [taxi.yaw.sin(), taxi.yaw.cos()];
    if back || yaw.sin() * way[0] + yaw.cos() * way[1] < 0.9 {
        return None;
    }
    // (Its middle alongside the taxi's, and how far right of it the taxi's is.)
    let kc = (hit..ts.len().min(hit + PASS_ALONGSIDE)).min_by(|&a, &b| {
        crate::travel::dist(route.at(ts[a]).0, taxi.at)
            .total_cmp(&crate::travel::dist(route.at(ts[b]).0, taxi.at))
    })?;
    let (pc, yc, _) = route.at(ts[kc]);
    let lat = (taxi.at[0] - pc[0]) * -yc.cos() + (taxi.at[1] - pc[1]) * yc.sin();
    if lat.abs() > 1.2 {
        return None;
    }
    let off = lat - taxi.half[1] - half[1] - PASS_SIDE;
    let dc = ds[kc];
    let d_out = dc - taxi.half[0] - half[0] - PASS_ENDS;
    let d_back = dc + taxi.half[0] + half[0] + PASS_ENDS;
    let d_from = (d_out - PASS_EASE).max(0.3);
    if d_out - d_from < PASS_EASE_LEAST {
        return None;
    }
    let at_d = |d: f32| {
        let k = ds.partition_point(|x| *x < d);
        ts.get(k).copied()
    };
    let (from, out, back, to) = (
        at_d(d_from)?,
        at_d(d_out)?,
        at_d(d_back)?,
        at_d(d_back + PASS_EASE)?,
    );
    if !(t < from && from < out && out <= back && back < to) {
        return None;
    }
    // (Not where it turns into or out of a driveway or a bay, nor a stop of its, a taxi.)
    if route.mouths.iter().any(|&(a, b)| a < to && b > from) {
        return None;
    }
    if car.taxi && crate::taxi::halts_within(w, c, from - 1.0, to + 1.0) {
        return None;
    }
    let pass = PassBy {
        home,
        taxi: o as u32,
        from,
        out,
        back,
        to,
        off,
        at: route.at(from).0,
    };
    // (Every pose of it out of the junctions' boxes, on the paving, and clear of those standing.)
    let g = &w.travel.graph;
    let standing: Vec<Rect> = parked(&w.cars)
        .filter(|d| d.first != car_first(c))
        .map(|d| d.body())
        .collect();
    let mut s = from;
    while s <= to {
        let (p, yaw0, _) = route.at(s);
        let (q, _, _) = route.at(s + 1.0);
        let (a, b) = (
            shifted(p, yaw0, off * pass_share(&pass, s)),
            shifted(q, yaw0, off * pass_share(&pass, s + 1.0)),
        );
        let r = Rect {
            at: a,
            yaw: (b[0] - a[0]).atan2(b[1] - a[1]),
            half,
        };
        if in_box(&g.boxes, r.at, car.len) {
            return None;
        }
        let (sn, cs) = r.yaw.sin_cos();
        let paved = [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)]
            .into_iter()
            .all(|(i, j)| {
                let k = [
                    r.at[0] + sn * i * half[0] + cs * j * half[1],
                    r.at[1] + cs * i * half[0] - sn * j * half[1],
                ];
                g.near(k, 8.0, 1, Mode::Car)
                    .first()
                    .is_some_and(|x| x.off <= kerb_off(&g.roads[x.road as usize]) + KERB_SLACK)
            });
        if !paved || standing.iter().any(|o| o.gap(&r) < PASS_SIDE - 0.1) {
            return None;
        }
        s += PASS_STEP;
    }
    Some(pass)
}

/// The stretch of `route` ahead of `t` (ticks along it) alongside taxi `taxi` standing at the kerb,
/// going the other way, where one passing the taxi (`PassBy`) would be out in its lane, begun and
/// ended a car of `len` short of that: its turn there with them (`pass_stands`). None: its way
/// doesn't go by it so.
fn span_by(route: &Route, t: f32, len: f32, taxi: &Rect) -> Option<(f32, f32)> {
    let u = [taxi.yaw.sin(), taxi.yaw.cos()];
    let n = [-taxi.yaw.cos(), taxi.yaw.sin()];
    let reach = taxi.half[0] + len + PASS_ENDS + PASS_EASE + 1.0;
    let end = route.ticks().min(t + PASS_LOOK);
    let mut s = t + 1.0;
    let (mut first, mut last) = (None, None);
    while s < end {
        let (q, yaw, back) = route.at(s);
        let rel = [q[0] - taxi.at[0], q[1] - taxi.at[1]];
        let along = rel[0] * u[0] + rel[1] * u[1];
        let lat = rel[0] * n[0] + rel[1] * n[1];
        let facing = yaw.sin() * u[0] + yaw.cos() * u[1];
        if !back && along.abs() <= reach && (-4.5..-0.5).contains(&lat) && facing < -0.9 {
            first.get_or_insert(s);
            last = Some(s);
        } else if last.is_some() {
            break;
        }
        s += PASS_STEP;
    }
    Some((first?, last?))
}

/// The cars held up round car `c` (`unjam`): it and those it waits for (`waits`), however far
/// round, each held up (`kind`: `Some(false)`) or a car parked in its way (`Some(true)`, not one of
/// them); with whether they're held up by each other alone, and whether one is by a car parked.
/// Not by each other alone: someone on foot among those waited for (they step aside first), one
/// on its way or out of sight (`kind`: None), or one at a red light (`red`) waiting for nothing
/// else (it goes on the green). One there that a car's in the way of waits for it on the green too
/// (`Traffic::begin`): with the others on another phase, one of them would always be at a red
/// light, and they'd stand there for good.
fn held_round(
    c: usize,
    waits: impl Fn(u64) -> Vec<u64>,
    red: impl Fn(u64) -> bool,
    kind: impl Fn(usize) -> Option<bool>,
) -> (Vec<usize>, bool, bool) {
    let mut ring = vec![c];
    let mut todo = vec![car_first(c)];
    let mut ok = true;
    let mut parked = false;
    while let Some(k) = todo.pop() {
        let by = waits(k);
        if by.is_empty() && red(k) {
            ok = false;
        }
        for o in by {
            if o < 1 << 63 {
                ok = false;
                continue;
            }
            let o = (o & !(1 << 63)) as usize;
            if ring.contains(&o) {
                continue;
            }
            match kind(o) {
                Some(true) => parked = true,
                Some(false) => {
                    ring.push(o);
                    todo.push(car_first(o));
                }
                None => ok = false,
            }
        }
    }
    (ring, ok, parked)
}

/// Cars in sight held up by each other (`traffic::waits_on`, round in a ring or on to one that
/// waits for nothing), or by a car parked in the way, none of them on `UNJAM` (`UNJAM_TICKS`),
/// none at a red light waiting for nothing else (one a car's in the way of there waits for it on
/// the green too) and nobody on foot among them (they step aside first, `go_on`): the one of them
/// with the most clear room behind it does as a driver would (`Detour`). It backs up along the way
/// it came, round the turn it took, till it's out of the last junction it came through, and goes
/// on from there another way, round the stretch it was held up on (`Route::detour`). Of those with
/// as much room: one with nobody riding with it first, then one on a trip before one on its way to
/// or from work, then one with riders, a work taxi last, and only once its stops are behind it
/// (they're along its way). None can (no junction behind it within the room it has, or no other
/// way): counted with why (`Traffic::unjam_impossible`), tried again `UNJAM_AGAIN` later.
pub(crate) fn unjam(w: &mut World) {
    let tick = w.tick;
    let held = |c: &Car| {
        matches!(c.state, CarState::Leaving | CarState::Arriving)
            && c.sight == Sight::Seen
            && tick >= c.stall.1 + UNJAM_TICKS
            && c.shown.is_some_and(|s| s.2)
    };
    let mut rings: Vec<Vec<usize>> = Vec::new();
    let mut taken = vec![false; w.cars.len()];
    // (Held up `PASS_WAIT` by a work taxi standing at the kerb in its way with no way out round it,
    // or waiting for its way round it while none of those it waits for goes on either: as by a
    // car parked there. Waiting its turn behind those coming the other way, it gets it.)
    let standing = |k: u64| {
        w.cars
            .get((k & !(1 << 63)) as usize)
            .is_some_and(|x| x.taxi && x.standing)
    };
    let still = |k: u64| {
        w.cars
            .get((k & !(1 << 63)) as usize)
            .is_some_and(|x| x.standing || tick >= x.stall.1 + PASS_WAIT)
    };
    for (c, car) in w.cars.iter().enumerate() {
        let by = crate::traffic::waits_on(w, car_first(c));
        let waiting = matches!(car.state, CarState::Leaving | CarState::Arriving)
            && car.sight == Sight::Seen
            && !car.standing
            && tick >= car.stall.1 + PASS_WAIT
            && car.shown.is_some_and(|s| s.2)
            && match car.pass_by {
                Some(p) => car.progress <= p.from && by.iter().all(|&k| k >= 1 << 63 && still(k)),
                None => by.iter().any(|&k| standing(k)),
            };
        let key = car_first(c);
        if waiting
            && w.traffic
                .unjam_tried
                .iter()
                .all(|&(k, at)| k != key || tick >= at + UNJAM_AGAIN)
        {
            taken[c] = true;
            rings.push(vec![c]);
        }
    }
    for c in 0..w.cars.len() {
        if taken[c] || !held(&w.cars[c]) {
            continue;
        }
        let (ring, ok, parked) = held_round(
            c,
            |k| crate::traffic::waits_on(w, k),
            |k| crate::traffic::at_red(w, k),
            |o| {
                if w.cars
                    .get(o)
                    .is_some_and(|x| x.state == CarState::Parked && !x.taxi)
                {
                    Some(true)
                } else {
                    w.cars.get(o).is_some_and(held).then_some(false)
                }
            },
        );
        for &o in &ring {
            taken[o] = true;
        }
        let key = ring.iter().map(|&o| car_first(o)).min().unwrap_or(0);
        if ok
            && (ring.len() >= 2 || parked)
            && w.traffic
                .unjam_tried
                .iter()
                .all(|&(k, at)| k != key || tick >= at + UNJAM_AGAIN)
        {
            rings.push(ring);
        }
    }
    for ring in rings {
        let key = ring.iter().map(|&o| car_first(o)).min().unwrap_or(0);
        // (A ring tried a little while ago and still there: counted once.)
        let again = w.traffic.unjam_tried.iter().any(|&(k, _)| k == key);
        w.traffic
            .unjam_tried
            .retain(|&(k, at)| k != key && tick < at + 4 * UNJAM_AGAIN);
        w.traffic.unjam_tried.push((key, tick));
        // Who of them can turn round, and how much room each has behind it: the most room
        // first, then by who's in it (`unjam`).
        let mut can: Vec<((f32, u8), usize, Detour)> = Vec::new();
        let mut why = "";
        for &c in &ring {
            match turn_round(w, c) {
                Ok((room, d)) => {
                    let car = &w.cars[c];
                    let riders = car.aboard.len() > 1 || !car.riders.is_empty();
                    let rank = match (car.taxi, riders, car.trip.is_some()) {
                        (true, ..) => 3,
                        (_, false, true) => 0,
                        (_, false, false) => 1,
                        (_, true, _) => 2,
                    };
                    can.push((((room * 2.0).floor() / 2.0, rank), c, d));
                }
                Err(e) => {
                    // (Why the household's cars couldn't, before that one of them was a taxi.)
                    let rank = |e: &str| UNJAM_WHY.iter().position(|x| *x == e).unwrap_or(0);
                    if why.is_empty() || rank(e) > rank(why) {
                        why = e;
                    }
                }
            }
        }
        can.sort_by(|a, b| {
            b.0.0
                .total_cmp(&a.0.0)
                .then(a.0.1.cmp(&b.0.1))
                .then(a.1.cmp(&b.1))
        });
        let mut fired = false;
        for (_, c, d) in can {
            if turn_round_by(w, c, d) {
                fired = true;
                // (No other way than back where it was held up: it lets the others by first.)
                let blocked = w.cars[c]
                    .route
                    .as_ref()
                    .and_then(|(_, r)| {
                        let back = r.points.iter().position(|q| !q.3)?;
                        let at = r.points.first()?.0;
                        Some(
                            r.points[back..]
                                .iter()
                                .any(|q| crate::travel::dist(q.0, at) < 3.0),
                        )
                    })
                    .unwrap_or(false);
                if blocked {
                    let by = ring
                        .iter()
                        .filter(|&&o| o != c)
                        .map(|&o| (o as u32, w.cars[o].progress, w.cars[o].state))
                        .collect();
                    w.cars[c].let_by = Some((tick + LET_BY_MOST, by));
                }
                break;
            }
            why = "no way";
        }
        if !fired && !again {
            *w.traffic.unjam_impossible.entry(why).or_default() += 1;
        }
    }
    // The ground each car turning round backs up over still (`Traffic::turning`): the others
    // keep out of it till it has (`drive_on`).
    let mut turning = Vec::new();
    for (c, car) in w.cars.iter().enumerate() {
        let (Some(_), Some((_, r))) = (car.detour, car.route.as_ref()) else {
            continue;
        };
        // (Backed up as far as it goes, driving on from the next tick: no ground left to back
        // over, only where it stands, which the others keep out of anyway (`guard`). Kept as
        // its, one backing up out of its way waited for it there for good while it waited for
        // that one, Oakview.)
        if !r.at(car.progress + 1.0).2 {
            continue;
        }
        let half = [car.len / 2.0, car.wid / 2.0];
        let mut t = car.progress;
        let mut path = Vec::new();
        while r.at(t).2 && t < r.ticks() {
            let (p, yaw, _) = r.at(t);
            path.push(Rect {
                at: shifted(p, yaw, car.shift),
                yaw,
                half,
            });
            t += 1.0;
        }
        if !path.is_empty() {
            turning.push((car_first(c), path));
        }
    }
    w.traffic.turning = turning;
}

/// Car `c` turned round `d` (`Detour`, `unjam`): its way that way from now on, its claims let go.
/// False: there's no such way (as it was).
fn turn_round_by(w: &mut World, c: usize, d: Detour) -> bool {
    let tick = w.tick;
    let home = d.home;
    let World {
        cars,
        exits,
        travel,
        traffic,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    car.avoided.retain(|a| a.1 > tick);
    let old = car.route.take();
    let had = car.detour;
    let twice = had.is_some();
    let passing = car.pass_by.take();
    car.detour = Some(d);
    let mut halts: Option<Box<dyn Fn(f32) -> f32>> = None;
    // (Turned round from its way as it is: a work taxi's is its run's, `taxi.rs`; one turned round
    // before, the way it turned round to. After a load that's worked out from the way it always
    // takes, `Car::route`: a car turned round twice is where the second way has it then. So too
    // one whose way isn't the way it always takes as it backs up along it, as one set off from
    // where it parked before stands: from that, it jumped there, tens of metres.)
    let plain = (!travel.graph.is_empty())
        .then(|| Route::on_roads(car, exits, travel, home))
        .flatten();
    let off_plain = match (old.as_ref(), plain.as_ref()) {
        (Some((_, had)), Some(p)) => [d.back, (d.back + d.from) / 2.0, d.from]
            .iter()
            .any(|&t| crate::travel::dist(had.at(t).0, p.at(t).0) > 0.05),
        _ => false,
    };
    if car.taxi || twice || off_plain {
        let new = old
            .as_ref()
            .and_then(|(_, had)| car.detoured(exits, travel, home, &d, had));
        // (Its stops still ahead, on the way it had: where they are along it now.)
        if let (true, Some(new), Some((_, had))) = (d.same, new.as_ref(), old.as_ref()) {
            halts = Some(Box::new(moved_along(had, new, d.back)));
        }
        car.route = new.map(|r| (home, r));
    } else {
        // (As `Car::route` has it after a load; but no such way, it isn't turned round: not off
        // on its way afresh from where it set off, which it jumped back to.)
        car.route = plain
            .as_ref()
            .and_then(|p| car.detoured(exits, travel, home, &d, p))
            .map(|r| (home, r));
    }
    // (Backing up as the new way has it, every pose of it clear of the others by the gap, as it
    // found the room, and of the ground any other turning round still backs up over
    // (`Traffic::turning`): two backing up into each other's way would wait for each other for
    // good. Else not.)
    let shift = car.shift;
    let half = [car.len / 2.0, car.wid / 2.0];
    let me = car_first(c);
    let others: Vec<Rect> = cars
        .iter()
        .enumerate()
        .filter(|&(o, k)| o != c && k.sight == Sight::Seen)
        .filter_map(|(_, k)| {
            k.shown.map(|s| Rect {
                at: s.0,
                yaw: s.1,
                half: [k.len / 2.0, k.wid / 2.0],
            })
        })
        .chain(
            traffic
                .turning
                .iter()
                .filter(|(k, _)| *k != me)
                .flat_map(|(_, path)| path.iter().copied()),
        )
        .collect();
    let car = &mut cars[c];
    let clear = car.route.as_ref().is_some_and(|(_, r)| {
        let body = |t: f32| {
            let (p, yaw, _) = r.at(t);
            Rect {
                at: shifted(p, yaw, shift),
                yaw,
                half,
            }
        };
        let now = body(0.0);
        let mut t = 0.0;
        let mut ok = true;
        while ok && t < r.ticks() && r.at(t).2 {
            let b = body(t);
            ok = others.iter().all(|o| {
                crate::travel::dist(o.at, b.at) > o.half[0] + b.half[0] + GAP + 0.5
                    || o.gap(&b) >= GAP.min(o.gap(&now))
            });
            t += 1.0;
        }
        ok
    });
    if car
        .route
        .as_ref()
        .is_none_or(|(h, r)| *h != home || r.ticks() <= 1.0)
        || !clear
    {
        // (As it was: on the way it had, turned round before or not. Not left on the way it
        // turned round to with no turn round, as on its way: backing up along it, it waited on
        // the one it backed into for good, Tide Sands.)
        car.detour = had;
        car.route = old;
        car.pass_by = passing;
        return false;
    }
    traffic.forget(car_first(c));
    traffic.unjams += 1;
    car.avoided.retain(|a| a.1 > tick && a.0 != d.avoid);
    car.avoided.push((d.avoid, tick + AVOID_FOR));
    car.progress = 0.01;
    car.unhold();
    car.stall = (car.progress, tick);
    if let Some(f) = halts {
        crate::taxi::moved_halts(w, c, &*f);
    }
    true
}

/// Turned round only backing up out of the way (`Detour::same`) from `had` to `new`, backing up
/// to `back`: where a point so far along `had` (ticks) is along `new` (the stretch from `back` on
/// is the same, after the stretch it backs up).
fn moved_along(had: &Route, new: &Route, back: f32) -> impl Fn(f32) -> f32 + use<> {
    let i0 = had.points.partition_point(|q| q.2 <= back);
    let n_rev = new.points.len().saturating_sub(had.points.len() - i0);
    let (hp, np): (Vec<f32>, Vec<f32>) = (
        had.points.iter().map(|q| q.2).collect(),
        new.points.iter().map(|q| q.2).collect(),
    );
    move |a: f32| {
        let i = hp.partition_point(|&t| t <= a).saturating_sub(1).max(i0);
        let j = (n_rev + i - i0).min(np.len().saturating_sub(1));
        let (t0, t1) = (hp[i], hp.get(i + 1).copied().unwrap_or(hp[i]));
        let (u0, u1) = (np[j], np.get(j + 1).copied().unwrap_or(np[j]));
        let k = if t1 > t0 {
            ((a - t0) / (t1 - t0)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        u0 + (u1 - u0) * k
    }
}

/// Work taxi `c`, its run loaded (`taxi::settle`) on the way it always takes, as far along it as
/// it had come on the way it turned round to (`d`, `Run::detour`): on that way, its halts still
/// ahead where they are along it. (Turned round twice, the second way from the way it always
/// takes: as a household's car, `Car::route`.)
pub(crate) fn turned_round(w: &mut World, c: usize, d: Detour) {
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    let Some((_, had)) = car.route.take() else {
        return;
    };
    let progress = car.progress;
    let Some(new) = car.detoured(exits, travel, d.home, &d, &had) else {
        car.route = Some((true, had));
        return;
    };
    let halts = d.same.then(|| moved_along(&had, &new, d.back));
    car.detour = Some(d);
    car.route = Some((d.home, new));
    car.progress = progress;
    if let Some(f) = halts {
        crate::taxi::moved_halts(w, c, &f);
    }
}

/// How far car `c` can back up along the way it came (m) clear of every other car (`GAP`), and
/// where it would turn round (`Detour`): out of the last junction's box it came through, along the
/// stretch before it, within that room. Err: why it can't.
fn turn_round(w: &World, c: usize) -> Result<(f32, Detour), &'static str> {
    let car = &w.cars[c];
    // (A work taxi with its stops still ahead backs up out of the way only, and goes on its way.)
    let same = car.taxi && crate::taxi::halts_ahead(w, c);
    // (Turned round already, backing up still or letting the others by: not again.)
    if car.let_by.is_some() {
        return Err("turning round");
    }
    if car.detour.is_some()
        && car
            .route
            .as_ref()
            .is_some_and(|(_, r)| r.at(car.progress).2)
    {
        return Err("turning round");
    }
    let Some((home, route)) = car.route.as_ref() else {
        return Err("no way");
    };
    let t = car.progress;
    let half = [car.len / 2.0, car.wid / 2.0];
    let body = |s: f32| {
        let (p, yaw, _) = route.at(s);
        Rect {
            at: shifted(p, yaw, car.shift),
            yaw,
            half,
        }
    };
    let others: Vec<Rect> = w
        .cars
        .iter()
        .enumerate()
        .filter(|&(o, k)| o != c && k.sight == Sight::Seen)
        .filter_map(|(_, k)| {
            k.shown.map(|s| Rect {
                at: s.0,
                yaw: s.1,
                half: [k.len / 2.0, k.wid / 2.0],
            })
        })
        .collect();
    let clear = |r: &Rect| {
        others.iter().all(|b| {
            crate::travel::dist(b.at, r.at) > b.half[0] + r.half[0] + GAP + 0.5 || b.gap(r) >= GAP
        })
    };
    // (Not back into the driveway it came out of, nor out of town.)
    let first = route
        .mouths
        .first()
        .filter(|m| m.0 <= 0.0 && !*home)
        .map_or(1.0, |m| m.1.max(1.0));
    let boxes = &w.travel.graph.boxes;
    // (The stretch it was held up on: where it is, or on along its way.)
    let (mut ahead, step) = (t, drive_ticks(0.2));
    while ahead + step < route.ticks() && crate::travel::dist(route.at(ahead).0, body(t).at) <= 6.0
    {
        ahead += step;
    }
    let on = *w
        .travel
        .graph
        .near(route.at(ahead).0, ROAD_REACH, 1, Mode::Car)
        .first()
        .ok_or("no way")?;
    let turn = |back: f32, room: f32| {
        Ok((
            room,
            Detour {
                home: *home,
                from: t,
                back,
                avoid: on.edge,
                same,
                mouth: None,
            },
        ))
    };
    // Back along the way it came, while that's clear: out of the last junction's box it came
    // through and on a little (`TURN_PAST`), where it turns into another way; or, short of one, the
    // furthest place back where the road is wide enough for one to pass it pulled over
    // (`PASS_ROOM`) and it's a length and more back, where it waits for the others to go by.
    let (mut s, mut room, mut was) = (t, 0.0f32, body(t).at);
    let mut boxed = in_box(boxes, was, car.len);
    let mut seen_box = false;
    let mut wide: Option<(f32, f32)> = None;
    let mut why = "no junction behind";
    loop {
        let next = (s - 1.0).max(first);
        if next >= s {
            break;
        }
        let r = body(next);
        if !clear(&r) {
            why = if seen_box { "no room" } else { why };
            break;
        }
        room += crate::travel::dist(was, r.at);
        was = r.at;
        s = next;
        if room > TURN_ROOM {
            break;
        }
        let inside = in_box(boxes, r.at, car.len);
        if inside && !boxed {
            seen_box = true;
        }
        boxed = inside;
        if !inside && room > car.len + TURN_PAST && route.room_at(s) >= PASS_ROOM {
            wide = Some((s, room));
        }
        // (Out of the box it came through, and on a little: there it turns.)
        if seen_box && !inside {
            let mut back = s;
            let mut more = 0.0;
            while more < TURN_PAST {
                let n = (back - 1.0).max(first);
                if n >= back || !clear(&body(n)) {
                    break;
                }
                more += crate::travel::dist(body(back).at, body(n).at);
                back = n;
            }
            return turn(back, room + more);
        }
    }
    // Short of a junction, a driveway's mouth it came by within that clear room (not a taxi on
    // its run's way: it backs up only).
    if !same && let Some((back, m)) = mouth_behind(w, c, route, s, t, &clear) {
        let room = crate::travel::dist(body(back).at, body(t).at);
        return Ok((
            room,
            Detour {
                home: *home,
                from: t,
                back,
                avoid: on.edge,
                same,
                mouth: Some(m),
            },
        ));
    }
    match wide {
        Some((back, room)) => turn(back, room),
        None => Err(why),
    }
}

/// The nearest driveway's mouth behind car `c` (`t` ticks along `route`, clear back to `lo`) it can
/// turn round in (`MouthTurn`): one its way came by, empty (`clear`, every pose backing into it
/// and pulling out of it the other way), and where along its way it backs up to before backing
/// into it.
fn mouth_behind(
    w: &World,
    c: usize,
    route: &Route,
    lo: f32,
    t: f32,
    clear: &dyn Fn(&Rect) -> bool,
) -> Option<(f32, MouthTurn)> {
    let car = &w.cars[c];
    let g = &w.travel.graph;
    let half = [car.len / 2.0, car.wid / 2.0];
    let at = |s: f32| route.at(s);
    let mut best: Option<(f32, f32, MouthTurn)> = None;
    for d in &w.drives {
        let Some(way) = d.way() else { continue };
        let Some(p) = d.meets().and_then(|(r, s)| g.at_source(r, s)) else {
            continue;
        };
        let road = &g.roads[p.road as usize];
        let mp = road.at(p.s);
        // (Where its way passes the mouth, behind it and within the clear room.)
        let mut s = t;
        let mut near = None;
        while s >= lo {
            if crate::travel::dist(at(s).0, mp) < road.width / 2.0 + 1.0 {
                near = Some(s);
                break;
            }
            s -= 1.0;
        }
        let Some(sm) = near else { continue };
        if best.is_some_and(|b| b.0 >= sm) {
            continue;
        }
        let (_, yaw, _) = at(sm);
        let tan = road.tangent(p.s);
        let go = if tan[0] * yaw.sin() + tan[1] * yaw.cos() >= 0.0 {
            1.0
        } else {
            -1.0
        };
        let m = MouthTurn {
            road: p.road,
            s: p.s,
            way,
            go,
        };
        let Some(bay) = mouth_turn(&m, &w.travel, car.len) else {
            continue;
        };
        let (mut into, mut out) = (Legs::default(), Legs::default());
        bay.leave(&mut into, go);
        bay.leave(&mut out, -go);
        let Some(l) = into.last() else { continue };
        // (Where its way passes where backing into the mouth begins: it backs up to there.)
        let mut back = None;
        let mut s = t;
        while s >= lo {
            if crate::travel::dist(at(s).0, l) < 1.0 {
                back = Some(s);
                break;
            }
            s -= 1.0;
        }
        let Some(back) = back else { continue };
        // (Every pose of it, backing in and pulling out, clear of the others.)
        let poses = |pts: &[([f32; 2], bool, f32)]| {
            pts.windows(2).all(|k| {
                let (a, b) = (k[0].0, k[1].0);
                let r = Rect {
                    at: b,
                    yaw: (b[0] - a[0]).atan2(b[1] - a[1]),
                    half,
                };
                crate::travel::dist(a, b) < 1e-3 || clear(&r)
            })
        };
        if poses(&into.pts) && poses(&out.pts) {
            best = Some((sm, back, m));
        }
    }
    best.map(|(_, back, m)| (back, m))
}

/// Why none of a ring of cars could turn round (`unjam`), the most telling last: the one counted.
const UNJAM_WHY: [&str; 4] = ["turning round", "no room", "no junction behind", "no way"];

/// Longest a car turned round with no other way waits out of the way for those it was held up
/// with to go on (ticks of drive time, `unjam`).
const LET_BY_MOST: u64 = drive_secs(15.0);

/// How long a car keeps off a stretch it turned round from, turning round again (ticks of drive
/// time, `unjam`).
const AVOID_FOR: u64 = drive_secs(30.0);

/// How far the others keep off the ground a car turning round backs up over, past `GAP` (m,
/// `drive_on`).
const TURN_KEEP: f32 = 0.5;

/// The ground right behind a car turning round that holds it up (m, `drive_on`).
const TURN_BEHIND: f32 = 0.3;
/// A road where a car can pull over this far (m, `Route::room_at`) is wide enough for one to pass
/// it: a car held up with no junction behind it backs up to such a place (`turn_round`).
const PASS_ROOM: f32 = 0.6;
/// Furthest a car backs up to turn round (m, `turn_round`).
const TURN_ROOM: f32 = 60.0;
/// How far on past a junction's box a car backs up to turn round there (m).
const TURN_PAST: f32 = 3.0;
/// Held up this long (ticks of drive time), in sight, a car turns round (`unjam`): well before a
/// check counts it held up for good (10 game minutes).
const UNJAM_TICKS: u64 = drive_secs(4.0);
/// Cars none of which could turn round are tried again this much later (ticks of drive time,
/// `unjam`).
const UNJAM_AGAIN: u64 = drive_secs(2.0);

/// Whether car `c` on a trip, its driver no longer at the wheel, is in the picture, or where it
/// parks out or at home is (`in_picture`): it drives on there, not to be seen to jump there.
fn trip_seen(w: &World, c: usize) -> bool {
    let car = &w.cars[c];
    car.shown.is_some_and(|s| in_picture(w, s.0))
        || car.trip.is_some_and(|t| in_picture(w, t.at))
        || in_picture(w, car.spot)
}

/// Whether `p` is in the picture or near it (within `UNSEEN` of the watch, `World::set_watch`).
/// Without a watch nothing is: every car is seen, as in a headless run.
fn in_picture(w: &World, p: [f32; 2]) -> bool {
    w.watch.is_some_and(|[x0, z0, x1, z1]| {
        let dx = (x0 as f32 - p[0]).max(p[0] - x1 as f32).max(0.0);
        let dz = (z0 as f32 - p[1]).max(p[1] - z1 as f32).max(0.0);
        dx.hypot(dz) <= UNSEEN
    })
}

/// A car this far (m, its middle) from the watch is out of the picture (`unstick`): its body and
/// the ground in front of it well clear of what's drawn.
const UNSEEN: f32 = 8.0;

/// Standing still this long (ticks of drive time), with all it waits for, out of the picture, a
/// car is put on (`unstick`).
const STUCK_UNSEEN: u64 = drive_secs(3.0);

/// The bay a car `len` long turns round in at driveway mouth `m` (`Detour::mouth`): just inside
/// the driveway, off the road, facing out (as if parked there nose out). None: no road there, or
/// the driveway too short for the car off the road.
fn mouth_turn(m: &MouthTurn, travel: &Travel, len: f32) -> Option<Bay> {
    let g = &travel.graph;
    let road = g.roads.get(m.road as usize)?;
    let mp = road.at(m.s);
    let depth = kerb_off(road) + len / 2.0 + MOUTH_IN;
    let spot = [mp[0] - m.way[0] * depth, mp[1] - m.way[1] * depth];
    Bay::new(spot, m.way[0].atan2(m.way[1]), Some(m.way), mp, road, m.s)
}

/// How far into a driveway past the kerb a car turning round in its mouth backs (m, past its
/// length).
const MOUTH_IN: f32 = 0.5;

/// Furthest (ticks along its way: 15 s of drive time) a car the camera came to on top of another
/// is put to come into sight clear of it (`moved_clear`, a tick at a time).
const MOVED_MOST: u32 = drive_secs(15.0) as u32;

/// Which of `cars` aren't drawn (`snapshot::car`): those out of sight (`Sight`: none is drawn
/// driving through the others), and any on top of one of those or touching it (`TOUCH`: at the
/// seen area's edge, out of the picture, so no two cars are ever drawn on top of each other).
pub fn hidden(cars: &[Car]) -> Vec<bool> {
    let mut out: Vec<bool> = cars.iter().map(|c| c.sight != Sight::Seen).collect();
    if !out.contains(&true) {
        return out;
    }
    let body = |c: &Car| {
        c.shown.map(|s| Rect {
            at: s.0,
            yaw: s.1,
            half: [c.len / 2.0, c.wid / 2.0],
        })
    };
    let out_of_sight: Vec<Rect> = cars
        .iter()
        .filter(|c| c.sight != Sight::Seen)
        .filter_map(body)
        .collect();
    for (k, car) in cars.iter().enumerate() {
        if out[k] {
            continue;
        }
        let Some(r) = body(car) else {
            continue;
        };
        out[k] = out_of_sight.iter().any(|o| {
            crate::travel::dist(o.at, r.at) < o.half[0] + r.half[0] + 0.5 && o.gap(&r) < TOUCH
        });
    }
    out
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
        // (Out of sight, it claims nothing and nobody keeps out of its way: `sight`.)
        if !matches!(car.state, CarState::Leaving | CarState::Arriving) || car.sight != Sight::Seen
        {
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
        const AHEAD_TOO: [f32; 3] = [0.0, drive_ticks(0.5), drive_ticks(1.0)];
        let ahead_too: &[f32] = if moved { &AHEAD_TOO } else { &[0.0] };
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
/// on (by `GAP`) and the ground in front of it (`AHEAD`) are clear of every car in town (`others`,
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
            .all(|o| o.body().gap(&body) >= GAP && o.body().overlap(&front) <= 0.0)
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

    /// The gap between the two (m: the least distance from a corner of either to a side of the
    /// other; below 0, how far they overlap, `overlap`).
    pub(crate) fn gap(&self, o: &Rect) -> f32 {
        let over = self.overlap(o);
        if over > 0.0 {
            return -over;
        }
        let corners = |r: &Rect| {
            let (s, c) = r.yaw.sin_cos();
            let (f, side) = (
                [s * r.half[0], c * r.half[0]],
                [c * r.half[1], -s * r.half[1]],
            );
            [(1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (-1.0, 1.0)].map(|(a, b)| {
                [
                    r.at[0] + a * f[0] + b * side[0],
                    r.at[1] + a * f[1] + b * side[1],
                ]
            })
        };
        let to_side = |p: [f32; 2], a: [f32; 2], b: [f32; 2]| {
            let ab = [b[0] - a[0], b[1] - a[1]];
            let t = ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1])
                / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-9);
            let t = t.clamp(0.0, 1.0);
            (p[0] - a[0] - ab[0] * t).hypot(p[1] - a[1] - ab[1] * t)
        };
        let (a, b) = (corners(self), corners(o));
        let mut best = f32::INFINITY;
        for (ps, qs) in [(&a, &b), (&b, &a)] {
            for &p in ps {
                for k in 0..4 {
                    best = best.min(to_side(p, qs[k], qs[(k + 1) % 4]));
                }
            }
        }
        best
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
/// minus this) is coming the other way (`up_against`, `passing`, `too_near`).
const ONCOMING: f32 = 0.7;

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

/// Whether `a` (where it would be a tick on) would come too near `b` coming the other way: nearer
/// its body than `GAP`, as the guard has it (`Rect::gap`). (Pulled over as far as `passing` says,
/// they're further apart than that: it measures across a way between theirs.)
pub(crate) fn too_near(a: ([f32; 2], f32, f32, f32), b: ([f32; 2], f32, f32, f32)) -> bool {
    let (ahead, theirs) = ([a.1.sin(), a.1.cos()], [b.1.sin(), b.1.cos()]);
    let body = |(at, yaw, len, wid): ([f32; 2], f32, f32, f32)| Rect {
        at,
        yaw,
        half: [len / 2.0, wid / 2.0],
    };
    ahead[0] * theirs[0] + ahead[1] * theirs[1] < -ONCOMING && body(a).gap(&body(b)) < GAP
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
        // it. Never into anyone. Nearer by under a millimetre is only rounding, as it drives on
        // alongside them: held up there a tick in two, it would keep pace with them.)
        let next = d.gap(body, s.pos);
        if !covers(d.at, turn, d.len, d.wid, body, s.pos, ahead, behind)
            || next >= 0.0 && next >= now.gap(body, s.pos) - 1e-3
        {
            continue;
        }
        if matches!(phase, Some(Phase::Using { .. }) | Some(Phase::Driving)) {
            continue;
        }
        waiting = true;
        whom.push(s.id);
        // (Walking on across its way, not waiting for it: they're on their way out of it, not
        // to be stepped back where they came from. Not one walking along its way, ahead of it or
        // towards it: they'd hold it up all the way, so they step aside in the end as anyone.)
        if matches!(phase, Some(Phase::Routing { .. }))
            && s.car_wait == 0
            && (s.yaw - d.yaw).sin().abs() >= ALONG
        {
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
                .then(|| make_way(d.at, d.yaw, nav, s.pos, STEP_ASIDE, clear))
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
    // (For one in sight, those in sight: `sight`; one out of sight keeps out of nobody's way.)
    if car.is_some_and(|c| cars[c].sight != Sight::Seen) {
        return Vec::new();
    }
    let cars = cars.iter().enumerate().filter_map(|(c, k)| match k.shown {
        Some((p, yaw, true, _)) if Some(c) != car && (car.is_none() || k.sight == Sight::Seen) => {
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
        // (A taxi standing out of sight, out of sight: `sight`.)
        Some((at, yaw, false, _)) if k.sight == Sight::Seen => Some(Driving {
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
/// ground `ahead` in front of it that it keeps clear (`AHEAD`): stepping out of its way is fine.
/// (Stopped there, waiting for another to pass, they'd hold it up as it held them.)
#[allow(clippy::too_many_arguments)]
pub(crate) fn walks_into(
    at: [f32; 2],
    yaw: f32,
    len: f32,
    wid: f32,
    ahead: f32,
    body: Body,
    from: [f32; 2],
    to: [f32; 2],
) -> bool {
    let turn = yaw.sin_cos();
    let body = Body {
        yaw: (to[0] - from[0]).atan2(to[1] - from[1]),
        ..body
    };
    let gap = |p: [f32; 2]| body.gap(at, turn, p, -len / 2.0, len / 2.0 + ahead, wid / 2.0);
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
pub(crate) fn in_the_way(
    cars: &[Car],
    id: u32,
    body: Body,
    from: [f32; 2],
    to: [f32; 2],
) -> Option<usize> {
    // (Already in the way of one waiting for those in its way: they go on out of it, where
    // they're going, as it waits; those not yet in it wait for it, so it gets its gap.)
    let holds_up = |car: &Car| {
        car.waited > 0
            && car.shown.is_some_and(|(at, yaw, on, _)| {
                on && in_way_of(at, yaw, car.len, car.wid, body, from)
            })
    };
    // (Holding one up so, on out of its way past the nose of one standing still, waiting its
    // turn: into its body never, but not waiting for it to come on, as that one waits for the
    // first, which waits for them.)
    let out = cars.iter().any(holds_up);
    cars.iter().position(|car| {
        let Some((at, yaw, true, _)) = car.shown else {
            return false;
        };
        if holds_up(car) {
            return false;
        }
        // (One it waits for by name (`Car::on_foot`) doesn't wait for it to come on, nor for where
        // it backs up to: only never into its body. Behind one backing out of a bay, a cyclist
        // waited for it as it waited for them, for good, Swanmere.)
        if car.on_foot.contains(&id) {
            return walks_into(at, yaw, car.len, car.wid, 0.0, body, from, to);
        }
        let ahead = if out && !car.moved { 0.0 } else { AHEAD };
        walks_into(at, yaw, car.len, car.wid, ahead, body, from, to)
            || car.next.is_some_and(|(at, yaw)| {
                walks_into(at, yaw, car.len, car.wid, ahead, body, from, to)
            })
    })
}

/// Whether resident `id` (with `body`, at `pos`) holds up a car driving by: it waits for them
/// (`Car::on_foot`), or they stand in the way of one waiting for those in its way (`in_the_way`'s
/// `holds_up`). They're to go on out of its way, not stand there: at a crosswalk closed to them
/// (`traffic::kerb_stop`) the two waited for each other for good.
pub(crate) fn holding_up(cars: &[Car], id: u32, body: Body, pos: [f32; 2]) -> bool {
    cars.iter().any(|car| {
        car.shown.is_some_and(|(at, yaw, on, _)| {
            on && (car.on_foot.contains(&id)
                || car.waited > 0 && in_way_of(at, yaw, car.len, car.wid, body, pos))
        })
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
    // (A porch post stands where it is too, as a garage does: `porches.rs`.)
    for p in w
        .porch_pieces
        .iter()
        .filter(|p| p.kind == crate::porches::PieceKind::Post)
    {
        if w.lot.in_bounds(p.x, p.z) {
            tiles.push(w.lot.tile_index(p.x, p.z));
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

/// A porch post was built or taken down (or put back by an undo): walking changes only on its
/// tile (`restand`).
pub(crate) fn posts_changed(w: &mut World) {
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
        // (On its way, it comes into sight as one coming in does, clear of the others: it may have
        // been driving through one out of sight as the game was saved, `sight`.)
        if matches!(car.state, CarState::Leaving | CarState::Arriving) {
            car.sight = Sight::Ghost;
        }
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
    let World {
        cars,
        exits,
        travel,
        drives,
        ..
    } = &mut *w;
    // (The public parking spots a car parked at would stand in the way of one turning into or out
    // of its driveway, or on the driveway itself (a bay off the road where a driveway runs: the
    // car coming home couldn't get by it): none parks there on a trip, `parking_for`.)
    let ways: Vec<([f32; 2], [f32; 2])> = drives
        .iter()
        .filter_map(|d| {
            let (road, s) = d.meets()?;
            let mouth = travel.graph.point(travel.graph.at_source(road, s)?);
            Some(([d.x, d.z], mouth))
        })
        .collect();
    travel.by_a_mouth = (0..travel.file.parking.len())
        .map(|k| {
            parked_at(travel, k).is_some_and(|(at, ..)| {
                ways.iter().any(|&(inner, mouth)| {
                    crate::travel::dist(mouth, at) < MOUTH_CLEAR
                        || to_segment(at, inner, mouth) < DRIVE_CLEAR
                })
            })
        })
        .collect();
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

/// How near a driveway's mouth (where it meets its road) the middle of a car parked out on a trip
/// may not be (m: a car's length and the swing of one turning in or out by it).
const MOUTH_CLEAR: f32 = 7.0;

/// How near a driveway (the line from its inner end to its mouth) the middle of a car parked out
/// on a trip may not be (m: half of each of two cars side by side, the gap between them and some).
const DRIVE_CLEAR: f32 = 2.5;

/// How far `p` is from the segment `a`..`b` (m).
pub(crate) fn to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let t =
        ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-6);
    let t = t.clamp(0.0, 1.0);
    crate::travel::dist(p, [a[0] + ab[0] * t, a[1] + ab[1] * t])
}

/// The public parking spot (`parking`) nearest `door` within `TRIP_PARK_REACH`: where it is and
/// which way a car faces parked there. Of `cars`, but car `me`, none parks where another is on a
/// trip to (on its way there, parked there, or still pulling out of it home: they'd stand one on
/// top of the other), nor at one by a driveway's mouth (`Travel::by_a_mouth`): with all near
/// taken, none.
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
        .filter(|&k| !travel.by_a_mouth.get(k).copied().unwrap_or(false))
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
    parking_near(travel, door).is_some()
}

/// The public parking spot nearest `door` for a trip by car (`TRIP_PARK_REACH`), taken or not:
/// the walk from it to there is part of the trip.
pub(crate) fn parking_near(travel: &Travel, door: [f32; 2]) -> Option<[f32; 2]> {
    parking_for(travel, door, &[], None).map(|(at, _)| at)
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
    !travel.file.parking.is_empty()
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
    if travel.file.parking.is_empty() || sim.visiting.is_some() {
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
            if at_wheel || trip_seen(w, c) {
                drive_on(w, c);
                seat_party(w, c);
            } else {
                // (Called away at the wheel, as they can't be: the car is there. Not in the
                // picture, nor where it's going: it would be seen to jump there, `trip_seen`.)
                w.cars[c].progress = ticks;
            }
            if w.cars[c].progress >= ticks {
                park_out(w, c);
            }
        }
        CarState::Arriving => {
            let ticks = route_ticks(w, c, true);
            if at_wheel || trip_seen(w, c) {
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
            // (Not while the camera is on where it's parked out or where it parks at home: it
            // isn't seen to jump there, `in_picture`.)
            let seen = {
                let car = &w.cars[c];
                car.trip.is_some_and(|t| in_picture(w, t.at)) || in_picture(w, car.spot)
            };
            if there && w.sims[d].bound_for_car != Some(c as u32) && !seen {
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
    if travel.file.parking.is_empty() || sim.visiting.is_some_and(|v| v.plot != plot) {
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

    /// A lane that goes a step past a road's end and back before it turns off (where one road's end
    /// meets another) keeps the points it came up by: rounding the corner takes out only those
    /// about it. (A work taxi's lane in Applewold, seed base 19: its corner's way in was the step
    /// back, and the 130 m of road before it went, the taxi driving straight across the grass.)
    #[test]
    fn a_corner_after_a_step_back_keeps_the_road_before_it() {
        let lane: Vec<[f32; 2]> = vec![
            [497.8137, 250.62184],
            [497.54294, 253.87134],
            [497.3321, 257.02386],
            [497.16132, 260.057],
            [496.99088, 263.9974],
            [496.8605, 267.95804],
            [496.76022, 271.9087],
            [496.70004, 275.87933],
            [496.67, 279.84],
            [496.68005, 283.81058],
            [496.73013, 287.7659],
            [496.81033, 291.7365],
            [496.92065, 295.70712],
            [497.07108, 299.6677],
            [497.26132, 303.62296],
            [497.48193, 307.5835],
            [497.74265, 311.53442],
            [498.03302, 315.4797],
            [498.36392, 319.42072],
            [498.73563, 323.38858],
            [499.08685, 326.48978],
            [499.4788, 329.6956],
            [499.92017, 332.97583],
            [500.40112, 336.3224],
            [500.92294, 339.71445],
            [501.46356, 343.12817],
            [502.02515, 346.5478],
            [502.60544, 349.93933],
            [503.1968, 353.297],
            [503.7874, 356.59033],
            [504.37732, 359.80997],
            [504.94775, 362.9223],
            [505.50842, 365.92575],
            [506.22787, 369.70285],
            [506.879, 373.18875],
            [507.59717, 377.03912],
            [508.2756, 380.8303],
            [508.36566, 381.36075],
            [510.51163, 381.43753],
            [509.77502, 378.2679],
            [507.91922, 377.908],
            [504.92474, 379.23447],
            [501.34598, 380.8985],
            [499.398, 381.86713],
            [497.2816, 381.88196],
            [498.27985, 382.7819],
            [498.2299, 379.45593],
            [498.17975, 376.0669],
            [498.09973, 372.6471],
            [498.01965, 369.19263],
            [497.91953, 365.71832],
            [497.8094, 362.2338],
            [497.6894, 358.7538],
            [497.55923, 355.2793],
            [497.4192, 351.8189],
            [497.279, 348.38397],
            [497.12897, 344.98358],
            [496.97894, 341.62277],
            [496.82867, 338.31708],
            [496.66885, 335.0712],
            [496.50854, 331.89508],
            [496.35873, 328.79837],
            [496.1586, 324.8062],
            [495.9585, 320.98364],
            [495.76865, 317.3664],
            [495.59875, 313.96884],
            [495.39883, 310.06036],
            [495.2389, 306.5716],
            [495.08926, 303.00076],
            [495.0692, 302.46964],
        ];
        let out = round_corners(&lane);
        let gap = out
            .windows(2)
            .map(|w| crate::travel::dist(w[0], w[1]))
            .fold(0.0f32, f32::max);
        assert!(gap < 8.0, "a step of {gap} m");
    }

    /// A body standing on the outside of a bend to the left reaches past the lane by the chord's
    /// sagitta (len² / 8R); on a straight, or on the inside of a bend to the right, by none.
    #[test]
    fn bend_out_is_the_sagitta_on_the_outside_of_a_left_bend() {
        let (r, len) = (40.0f32, 4.7f32);
        // An arc round (0, 0) of radius `r`, a point every degree; `turn` +1 bends to the left of
        // the way it runs (forward (sin, cos), right (-cos, sin): `shifted`).
        let arc = |turn: f32| -> Vec<[f32; 2]> {
            (0..=40)
                .map(|k| {
                    let a = (k as f32).to_radians();
                    [-r * a.cos() * turn, r * a.sin()]
                })
                .collect()
        };
        let want = len * len / (8.0 * r);
        let left = bend_out(&arc(1.0), 20, len);
        assert!((left - want).abs() < 0.01, "{left} against {want}");
        assert_eq!(bend_out(&arc(-1.0), 20, len), 0.0);
        let straight: Vec<[f32; 2]> = (0..20).map(|k| [0.0, k as f32]).collect();
        assert!(bend_out(&straight, 10, len) < 1e-5);
    }
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
            gone.unwrap() >= last_open + secs(0.5),
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
        for _ in 0..secs(1.0) {
            w.tick_once();
        }
        assert_eq!(w.cars[0].progress, 0.0, "it waits");
        for _ in 0..secs(30.0) {
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
    fn someone_whose_way_turns_right_in_front_of_a_waiting_car_gets_by() {
        // (As at Crown Quarter: a walk by road turned in a junction's box right in front of the
        // nose of a car waiting there, a while already, for others before them. At each step on
        // into the ground in front of it, it stepped them back out of it to the middle of the
        // tile beside, where it doesn't wait for them but they're still in its way, so on they
        // step into it again: they never got round, and it never went on.) They walk on round the
        // turn and out of its way, and it goes after. With the car a cm further out each time:
        // just where it stands decides which tiles round it it doesn't wait for them at.
        let mut held = Vec::new();
        for k in 0..=30 {
            let z = 3.4 + k as f32 * 0.01;
            let town = TOWN.replace(r#""z":3.0,"yaw""#, &format!(r#""z":{z},"yaw""#));
            let mut w = World::from_json(CONTENT, &town, 1).unwrap();
            w.autonomy = false;
            w.cars[0].state = CarState::Leaving;
            w.cars[0].progress = 0.0;
            w.cars[0].driver = Some(0);
            w.cars[0].aboard = vec![0];
            w.sims[0].away_until = Some(w.tick + 100_000);
            // (No shift of Bo's to set off for in the while: his way there ends where it's told.)
            w.sims[1].job = None;
            // Bo, just in front of it, on his way along the front of the house...
            w.sims[1].pos = [34.5, 8.5];
            let _ = w.apply(crate::Command::MoveTo {
                sim: 1,
                x: 40.5,
                z: 8.5,
            });
            // (It has waited a while already, for others before them.)
            w.cars[0].waited = PASS_TICKS + 5;
            w.tick_once();
            // ...by the ground right in front of its nose, where his way turns.
            let Some(Phase::Routing { waypoints, next }) =
                w.sims[1].current.as_mut().map(|a| &mut a.phase)
            else {
                panic!("Bo isn't on his way");
            };
            *waypoints = vec![[34.0, 7.6], [40.5, 8.5]];
            *next = 0;
            let mut by = None;
            for t in 0..secs(20.0) {
                w.tick_once();
                if by.is_none() && w.sims[1].pos[0] > 40.0 {
                    by = Some(t);
                }
            }
            if !by.is_some_and(|t| t < secs(10.0)) || w.cars[0].progress < secs(1.0) as f32 {
                held.push((z, by, w.sims[1].pos, w.cars[0].progress));
            }
        }
        assert!(
            held.is_empty(),
            "Bo never got round in front of the car, nor it on (its z, when he got by, where he \
             is, its progress): {held:?}"
        );
    }

    #[test]
    fn nobody_walks_into_a_car_driving_by() {
        let mut c = car([10.0, 10.0], 0.0, 20.0);
        c.shown = Some(([10.0, 10.0], 0.0, true, 0));
        let cars = [c];
        let on_foot = Body::WALKER;
        assert_eq!(
            in_the_way(&cars, u32::MAX, on_foot, [12.0, 10.0], [11.0, 10.0]),
            Some(0),
            "into its side"
        );
        assert_eq!(
            in_the_way(&cars, u32::MAX, on_foot, [10.8, 10.0], [11.5, 10.0]),
            None,
            "out of its way"
        );
        assert_eq!(
            in_the_way(&cars, u32::MAX, on_foot, [14.0, 10.0], [13.5, 10.0]),
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
            in_the_way(&cars, u32::MAX, on_foot, [12.4, 10.0], [12.0, 10.0]),
            None,
            "on foot, still clear"
        );
        assert_eq!(
            in_the_way(&cars, u32::MAX, bike, [12.4, 10.0], [12.0, 10.0]),
            Some(0),
            "but not on a bike"
        );
        // Nor into the ground just in front of it (it faces +z, its front at z = 12.2): stopped
        // there, they'd hold it up. Out of there is fine.
        assert_eq!(
            in_the_way(&cars, u32::MAX, on_foot, [10.0, 15.5], [10.0, 14.0]),
            Some(0),
            "in front of it"
        );
        assert_eq!(
            in_the_way(&cars, u32::MAX, on_foot, [10.0, 13.5], [10.0, 14.0]),
            None,
            "out from in front of it"
        );
    }

    #[test]
    fn one_holding_up_a_car_goes_on_past_the_nose_of_one_standing_still() {
        // One waiting for those in its way, facing +z at z = 10 (its front at z = 12.2); one
        // standing still beside it facing -x at x = 16 (its front at x = 13.8, the ground it keeps
        // clear to x = 11.6).
        let mut waiting = car([10.0, 10.0], 0.0, 20.0);
        waiting.shown = Some(([10.0, 10.0], 0.0, true, 0));
        waiting.waited = 40;
        let mut standing = car([16.0, 13.0], -std::f32::consts::FRAC_PI_2, 20.0);
        standing.shown = Some(([16.0, 13.0], -std::f32::consts::FRAC_PI_2, true, 0));
        let on_foot = Body::WALKER;
        // In the first's way, on out of it across the ground in front of the second: it's not
        // coming on (it waits its turn), they don't wait for it.
        let (from, to) = ([10.6, 13.0], [11.8, 13.0]);
        let cars = [waiting.clone(), standing.clone()];
        assert_eq!(
            in_the_way(&cars, u32::MAX, on_foot, from, to),
            None,
            "past its nose"
        );
        // Never into its body.
        assert_eq!(
            in_the_way(&cars, u32::MAX, on_foot, [13.0, 13.0], [13.6, 13.0]),
            Some(1),
            "into it"
        );
        // On the move, they wait for it as ever; nor, holding nobody up, into the ground in
        // front of one standing.
        let mut moving = standing.clone();
        moving.moved = true;
        assert_eq!(
            in_the_way(&[waiting.clone(), moving], u32::MAX, on_foot, from, to),
            Some(1),
            "it's coming on"
        );
        let mut clear = waiting;
        clear.waited = 0;
        assert_eq!(
            in_the_way(&[clear, standing], u32::MAX, on_foot, from, to),
            Some(1),
            "holding nobody up"
        );
    }

    #[test]
    fn one_a_car_backing_up_waits_for_goes_on_out_of_its_way() {
        // (Swanmere's: a car backing out of a bay by a lit junction, facing +z and backing towards
        // -z, waited for a cyclist behind it, who waited for it in turn: for good.) Waited for by
        // name, they go on, but never into its body; anyone else waits for it as ever.
        let mut backing = car([10.0, 10.0], 0.0, 20.0);
        backing.shown = Some(([10.0, 10.0], 0.0, true, 0));
        backing.next = Some(([10.0, 9.3], 0.0));
        backing.on_foot = vec![7];
        let (from, to) = ([9.0, 7.0], [9.6, 7.0]);
        let cars = [backing];
        assert_eq!(
            in_the_way(&cars, 8, Body::WALKER, from, to),
            Some(0),
            "another waits for it"
        );
        assert_eq!(
            in_the_way(&cars, 7, Body::WALKER, from, to),
            None,
            "the one it waits for"
        );
        assert_eq!(
            in_the_way(&cars, 7, Body::WALKER, [9.0, 7.95], [9.6, 8.0]),
            Some(0),
            "into its body"
        );
    }

    #[test]
    fn no_trip_parks_on_a_driveway() {
        // (Pebble Sands, a big world of the 0.7.0 samples: a bay off the road where a house's
        // driveway runs, 12 m from its mouth; a car parked out there on a trip, the car of the
        // house coming home couldn't get by it.) A spot on the driveway, one by its mouth, and one
        // down the street clear of it.
        let road: Vec<String> = (0..=199).map(|x| format!("[{x}.5,30]")).collect();
        let town = format!(
            r#"{{"width":200,"depth":60,
            "exits":[{{"x":0.5,"z":30,"kind":"road"}},{{"x":199.5,"z":30,"kind":"road"}}],
            "roads":[{{"points":[{}],"width":7,"kind":"street","ends":["out","out"]}}],
            "plots":[{{"name":"H","x":78,"z":8,"w":16,"d":18,"entry":[83.5,25.5]}}],
            "households":[{{"name":"F","plot":0}}],
            "sims":[{{"name":"D","household":0,"x":83.5,"z":17.5}}],
            "drives":[{{"plot":0,"x":90,"z":8.5,"front":1,"street":30,"road":0,"s":89.5}}],
            "cars":[{{"plot":0,"x":90,"z":11.25,"yaw":0,"street":30,"len":4.6,"wid":1.88,"body":2,"paint":0}}],
            "parking":[{{"x":91.0,"z":20.0,"rot":0,"area":0}},{{"x":93.0,"z":25.0,"rot":0,"area":0}},
                       {{"x":140.0,"z":24.0,"rot":0,"area":0}}]}}"#,
            road.join(",")
        );
        let w = World::from_json(CONTENT, &town, 1).unwrap();
        assert_eq!(w.travel.by_a_mouth, [true, true, false]);
    }

    #[test]
    fn a_ring_with_one_at_a_red_light_someone_is_in_the_way_of_is_held_up() {
        // (Willowshore, seed base 22: a taxi at a red light with another's body in its way through
        // the junction, it in the way of one at the green on the other phase, that one in front
        // of the other's. Whatever the light, one of them was at a red one, so the ring was never
        // turned round: 16 cars stood in view for good.) Three cars round in a ring, one at a red
        // light: it waits for one in its way too, so they're held up by each other; waiting for
        // the light alone, it goes on the green, and they aren't.
        let (a, b, c) = (car_first(0), car_first(1), car_first(2));
        let ring = |a_waits: Vec<u64>| {
            move |k: u64| {
                if k == a {
                    a_waits.clone()
                } else if k == b {
                    vec![c]
                } else {
                    vec![a]
                }
            }
        };
        let red = |k: u64| k == a;
        let held = |_: usize| Some(false);
        let (cars, ok, parked) = held_round(1, ring(vec![b]), red, held);
        assert_eq!(cars, [1, 2, 0]);
        assert!(ok && !parked, "held up by each other");
        let (_, ok, _) = held_round(1, ring(vec![]), red, held);
        assert!(!ok, "it waits for the light alone");
        let (_, ok, _) = held_round(1, ring(vec![b]), red, |o| (o != 2).then_some(false));
        assert!(!ok, "one of them on its way");
    }

    #[test]
    fn of_two_held_up_by_each_other_the_one_that_can_back_up_does() {
        // Two cars on their way, each waiting for the other, neither holding a claim: the later
        // backs up (`waits_back`); can it back up no further, the earlier does instead.
        let town = TOWN.replace(
            r#""cars":[{"plot":0"#,
            r#""cars":[{"plot":0,"x":30.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":1},{"plot":0"#,
        );
        let mut w = World::from_json(CONTENT, &town, 1).unwrap();
        assert_eq!(w.cars.len(), 2);
        let (a, b) = (car_first(0), car_first(1));
        w.cars[0].waits_for = Some(b);
        w.cars[1].waits_for = Some(a);
        assert!(waits_back(&w, a, b), "the later backs up");
        assert!(!waits_back(&w, b, a), "the earlier doesn't");
        w.cars[1].jam.2 = true;
        assert!(!waits_back(&w, a, b), "the later can't");
        assert!(waits_back(&w, b, a), "so the earlier does");
        w.cars[0].jam.2 = true;
        assert!(
            !waits_back(&w, b, a) && !waits_back(&w, a, b),
            "neither can"
        );
    }

    #[test]
    fn of_two_held_up_by_each_other_the_one_backing_up_goes_on() {
        // (Willowbrook, a big world of the 0.7.0 samples: a car and a taxi meeting round a bend
        // took turns backing up a tick each as their claims changed, for good.) The earlier backing
        // up already goes on backing up, though the later would otherwise; backed up as far as it
        // can, the later does.
        let town = TOWN.replace(
            r#""cars":[{"plot":0"#,
            r#""cars":[{"plot":0,"x":30.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":1},{"plot":0"#,
        );
        let mut w = World::from_json(CONTENT, &town, 1).unwrap();
        let (a, b) = (car_first(0), car_first(1));
        w.cars[0].waits_for = Some(b);
        w.cars[1].waits_for = Some(a);
        w.cars[0].jam = (200, 3, false);
        assert!(waits_back(&w, b, a), "the earlier goes on backing up");
        assert!(!waits_back(&w, a, b), "the later doesn't");
        w.cars[0].jam.2 = true;
        assert!(waits_back(&w, a, b), "the earlier can't: the later does");
        assert!(!waits_back(&w, b, a));
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

    /// On the traffic's clock a car on a country road goes about a metre a tick
    /// (`crate::DRIVE_SCALE`): one driven straight through the smallest junction's box at that
    /// speed, never slowing for it, still has its route sampled finer than the box, and asks for
    /// the box's claim before its body is in it, whichever tick it gets there.
    #[test]
    fn a_car_at_country_speed_cannot_skip_a_junction_box() {
        let speed = crate::travel::COUNTRY_SPEED;
        assert!(
            speed / crate::DRIVE_TICKS_PER_SECOND > 1.0,
            "a metre a tick or more"
        );
        // (Along x through a box at x = 100, the smallest: `traffic::boxes`' 6 m.)
        let (centre, r) = ([100.0, 0.0], 6.0);
        let mut legs = Legs::default();
        for k in 0..=50 {
            legs.push_lane([k as f32 * 4.0, 0.0], speed, 0.0, LANE);
        }
        let route = legs.timed(false, false);
        let (len, wid) = (4.4, 1.8);
        // The way sampled no further apart than `SAMPLE` and a little: finer than the box.
        let trace = crate::traffic::trace(|t| route.at(t), route.ticks());
        let most = trace
            .windows(2)
            .map(|w| crate::travel::dist(w[0].1, w[1].1))
            .fold(0.0f32, f32::max);
        assert!(
            most < crate::traffic::SAMPLE + 0.5 && most < r,
            "{most} m between samples"
        );
        assert!(SIGHT_STEP / crate::DRIVE_TICKS_PER_SECOND * speed < r);
        let pass = vec![0.0; trace.len()];
        let claims = crate::traffic::claims(&trace, &pass, len, wid, &[(centre, r)], &[]);
        assert_eq!(claims.len(), 1, "the box's claim");
        let c = &claims[0];
        // (Whether its body `t` ticks along touches the box.)
        let touches = |t: f32| {
            let (p, yaw, _) = route.at(t);
            let (dx, dz) = (centre[0] - p[0], centre[1] - p[1]);
            let along = dx * yaw.sin() + dz * yaw.cos();
            let across = dx * yaw.cos() - dz * yaw.sin();
            let off =
                ((along.abs() - len / 2.0).max(0.0)).hypot((across.abs() - wid / 2.0).max(0.0));
            off <= r
        };
        // Each tick it goes on from `t` to `t + 1` (`drive_on`, `Traffic::may_go`): into the box
        // only asking for its claim first.
        let mut asked = false;
        let mut t = 0.0;
        while t < route.ticks() {
            let asks = t + 1.0 > c.from && t < c.to;
            asked |= asks;
            assert!(
                !touches(t + 1.0) || touches(t) || asks,
                "at {t} it drives into the box ({:?}) unasked",
                route.at(t + 1.0).0
            );
            if touches(t + 1.0) {
                assert!(asked, "in the box at {} before asking", t + 1.0);
            }
            t += 1.0;
        }
        assert!(asked);
    }

    /// (Eaglescar, world 231441 of seed base 16: a car turned round where the way it came ran a
    /// little off the lane it turned round to; it stepped sideways onto that lane and, facing along
    /// the step, swung its tail half across a road too narrow for more as it drove on, into one
    /// coming the other way: both held up for good.) A car that came east along a street on its
    /// middle line backs up and drives on east: onto its lane ahead of where it stands, facing
    /// the way it goes all along.
    #[test]
    fn a_car_turned_round_off_its_lane_drives_on_facing_the_way_it_goes() {
        let road: Vec<String> = (0..=99).map(|x| format!("[{x}.5,30]")).collect();
        let town = format!(
            r#"{{"width":100,"depth":60,
            "exits":[{{"x":0.5,"z":30,"kind":"road"}},{{"x":99.5,"z":30,"kind":"road"}}],
            "roads":[{{"points":[{}],"width":7,"kind":"street","ends":["out","out"]}}],
            "plots":[],"households":[],"objects":[],"sims":[],"cars":[]}}"#,
            road.join(",")
        );
        let w = World::from_json(CONTENT, &town, 1).unwrap();
        let c = car([50.0, 10.0], 0.0, 30.0);
        let mut legs = Legs::default();
        for k in 0..=20 {
            legs.push_lane([20.0 + k as f32 * 2.0, 30.0], 5.0, 0.0, LANE);
        }
        let had = legs.timed(true, true);
        let at = |x: f32| had.points.iter().find(|p| p.0[0] >= x).unwrap().2;
        let rp = w.travel.graph.near([40.0, 30.0], ROAD_REACH, 1, Mode::Car)[0];
        let d = Detour {
            home: false,
            from: at(50.0),
            back: at(36.0),
            avoid: rp.edge,
            same: false,
            mouth: None,
        };
        let r = c
            .detoured(&w.exits, &w.travel, false, &d, &had)
            .expect("a way on");
        let on = r.points.iter().position(|p| !p.3).expect("it drives on");
        assert!(
            r.points[on..].iter().all(|p| !p.3),
            "backs up, then drives on"
        );
        // (At each point it drives on from, it faces the way it goes, as near as a step's curve.)
        let mut lane = 0.0f32;
        for k in r.points[on..].windows(2) {
            let (a, b) = (k[0], k[1]);
            lane = lane.max((b.0[1] - 30.0).abs());
            if crate::travel::dist(a.0, b.0) < 0.05 {
                continue;
            }
            let way = (b.0[0] - a.0[0]).atan2(b.0[1] - a.0[1]);
            assert!(
                wrap_angle(a.1 - way).abs() < 0.2,
                "at {:?} it faces {} going {way}",
                a.0,
                a.1
            );
        }
        assert!(lane > 0.5, "it comes to its lane ({lane} m off the middle)");
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
        // Takes a while, but not forever (seconds of drive time).
        let secs = r.ticks() / DRIVE_TICKS_PER_SECOND;
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

    #[test]
    fn pulling_back_in_beside_one_waiting_for_it_a_car_keeps_the_gap() {
        // Pulled over 0.5 m to its right beside one coming the other way that waits for it, it
        // pulls back in a step at a time (`pull_over`: nothing but the guard keeps it from that
        // one). In its lane its body would touch the other's: it stops short by the gap the cars
        // keep, waiting for that one.
        let (len, wid) = (4.6, 1.88);
        let them = [Driving {
            at: [wid, 0.0],
            yaw: std::f32::consts::PI,
            len,
            wid,
            ahead: AHEAD,
            behind: 0.0,
            first: car_first(1),
        }];
        let body = |shift: f32| Rect {
            at: shifted([0.0, 0.0], 0.0, shift),
            yaw: 0.0,
            half: [len / 2.0, wid / 2.0],
        };
        let mut shift: f32 = 0.5;
        let mut held = None;
        while shift > 0.0 && held.is_none() {
            let to = (shift - PULL_RATE).max(0.0);
            held = guard(&body(shift), &body(to), them.iter());
            if held.is_none() {
                shift = to;
            }
        }
        // (Side by side, as far apart as it's pulled over.)
        assert!(
            shift >= 0.149,
            "pulled back in to {shift:.3} m from it (`GAP`)"
        );
        assert_eq!(held, Some(car_first(1)), "never held for it");
        // (And on along its lane so near it: no nearer, as near as it is.)
        let on = |r: Rect| Rect {
            at: [r.at[0], r.at[1] + 0.3],
            ..r
        };
        assert_eq!(guard(&body(shift), &on(body(shift)), them.iter()), None);
    }

    /// The way along `pts`: the turn at each point between its steps (radians), and whether any
    /// step goes back against the one before.
    fn turns(pts: &[[f32; 2]]) -> (f32, bool) {
        let mut most = 0.0f32;
        let mut back = false;
        for w in pts.windows(3) {
            let (a, b) = (unit2(w[0], w[1]), unit2(w[1], w[2]));
            most = most.max(turned(a, b).abs());
            back |= dot2(a, b) < 0.0;
        }
        (most, back)
    }

    #[test]
    fn a_lane_turns_a_corner_on_an_arc() {
        // Along x, then right (+z) at (12, 0): a street corner, a point every 4 m.
        let lane = [
            [0.0, 0.0],
            [4.0, 0.0],
            [8.0, 0.0],
            [12.0, 0.0],
            [12.0, 4.0],
            [12.0, 8.0],
            [12.0, 12.0],
        ];
        let round = round_corners(&lane);
        assert_eq!(round.first(), lane.first());
        assert_eq!(round.last(), lane.last());
        let (most, back) = turns(&round);
        assert!(!back);
        assert!(most <= ARC_STEP + 1e-3, "turns {most} at a point");
        // On an arc as wide as a right turn's, its middle in from the corner.
        let r = RIGHT_TURN;
        let mid = round
            .iter()
            .min_by(|a, b| {
                crate::travel::dist(**a, [12.0, 0.0])
                    .total_cmp(&crate::travel::dist(**b, [12.0, 0.0]))
            })
            .copied()
            .unwrap();
        let inset = crate::travel::dist(mid, [12.0, 0.0]);
        assert!(
            (inset - r * (std::f32::consts::SQRT_2 - 1.0)).abs() < 0.05,
            "{inset}"
        );
        // Turning left, wider.
        let left: Vec<[f32; 2]> = lane.iter().map(|p| [p[0], -p[1]]).collect();
        let round = round_corners(&left);
        let mid = round
            .iter()
            .min_by(|a, b| {
                crate::travel::dist(**a, [12.0, 0.0])
                    .total_cmp(&crate::travel::dist(**b, [12.0, 0.0]))
            })
            .copied()
            .unwrap();
        let inset = crate::travel::dist(mid, [12.0, 0.0]);
        assert!(
            (inset - LEFT_TURN * (std::f32::consts::SQRT_2 - 1.0)).abs() < 0.05,
            "{inset}"
        );
    }

    #[test]
    fn a_lane_through_a_junction_where_it_doubles_back_turns_smoothly() {
        // Two roads' lanes meeting at a junction: the one in goes on 0.8 m past the corner, the
        // one out begins 0.6 m short of the line in, and the way doubles back between them.
        let lane = [
            [0.0, 0.0],
            [4.0, 0.0],
            [8.0, 0.0],
            [12.8, 0.0],
            [12.0, -0.6],
            [12.0, 3.0],
            [12.0, 7.0],
            [12.0, 11.0],
        ];
        let round = round_corners(&lane);
        let (most, back) = turns(&round);
        assert!(!back, "{round:?}");
        assert!(
            most <= ARC_STEP + 1e-3,
            "turns {most} at a point: {round:?}"
        );
        // On through the corner, no point further than the lines either side.
        assert!(
            round.iter().all(|p| p[0] <= 12.0 + 1e-3 && p[1] >= -1e-3),
            "{round:?}"
        );
    }

    #[test]
    fn a_car_faces_along_its_lane_where_the_next_step_is_a_hair_across_it() {
        // (King'ston and Pebble Quay: where a taxi stood at the kerb, the way on began 2 to 8 mm
        // from it, mostly across the lane, and it stood 25 to 37 degrees across its lane.) Along
        // +z, standing at z = 8, the next point 5 mm across: it faces along the lane there.
        let mut legs = Legs::default();
        for p in [
            [0.0, 0.0],
            [0.0, 4.0],
            [0.0, 8.0],
            [0.005, 8.001],
            [0.0, 12.0],
            [0.0, 16.0],
        ] {
            legs.push_lane(p, FAST, 0.0, LANE);
        }
        let route = legs.timed(true, true);
        for i in 1..route.points.len() - 1 {
            let yaw = route.points[i].1;
            assert!(
                yaw.abs().to_degrees() < 4.0,
                "{}° across its lane at {:?}",
                yaw.to_degrees(),
                route.points[i].0
            );
        }
    }

    #[test]
    fn a_car_faces_the_way_it_goes_round_a_corner() {
        let lane = round_corners(&[
            [0.0, 0.0],
            [4.0, 0.0],
            [8.0, 0.0],
            [12.0, 0.0],
            [12.0, 4.0],
            [12.0, 8.0],
        ]);
        let mut legs = Legs::default();
        for p in lane {
            legs.push_lane(p, FAST, 0.0, LANE);
        }
        let route = legs.timed(true, true);
        let mut was = route.at(0.0).0;
        let mut t = 1.0;
        let mut worst = 0.0f32;
        while t < route.ticks() {
            let (p, yaw, _) = route.at(t);
            let (dx, dz) = (p[0] - was[0], p[1] - was[1]);
            if dx.hypot(dz) > 1e-3 {
                worst = worst.max(wrap_angle(dx.atan2(dz) - yaw).abs());
            }
            was = p;
            t += 1.0;
        }
        assert!(
            worst.to_degrees() < 4.0,
            "its nose {}° off the way it goes",
            worst.to_degrees()
        );
    }
}
