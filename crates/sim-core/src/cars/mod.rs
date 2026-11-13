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
/// junctions were lit: the lights and the stop lines do the stopping now). Kept with most
/// junctions giving way again (2026-11-13): there the claims take turns through the box as ever
/// and a car stops at a zebra for those on it, and 4 to 5 m/s (some 16 km/h of drive time) is a
/// pace to turn a quiet corner at.
const SIDEWAYS: f32 = 6.5;
/// A step of a route shorter than this says nothing of the way a car faces along it (m,
/// `Legs::timed`): it faces as along the step before. (Where a taxi stands at the kerb, the way on
/// began 2 to 8 mm from it and mostly across the lane, and it stood 25 to 37 degrees across its
/// lane: King'ston, Pebble Quay.) Nor does a step shorter than `KINK_STEP` turned back on the one
/// before (more than a right angle, driving on, not backing: where a taxi stands, the way on began
/// 4 cm behind it, and it stood 86 degrees across its lane, over the kerb, Oakfield).
const FACING_STEP: f32 = 0.02;
const KINK_STEP: f32 = 0.1;
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
    /// Ticks it has stood at a zebra's stop line for those on foot (`zebra_yield`). Not saved.
    #[serde(skip)]
    zebra_wait: u32,
    /// Waiting at a red light this tick (`Traffic::red_for`): those on foot may walk up to its
    /// nose (`in_the_way`), short of a crosswalk. Not saved.
    #[serde(skip)]
    at_red: bool,
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
            zebra_wait: 0,
            at_red: false,
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
            zebra_wait: 0,
            at_red: false,
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

    /// Where its middle is, for the checks that pair it with another (`Near`): as it's drawn, or
    /// its spot.
    fn middle(&self) -> [f32; 2] {
        self.shown.map_or(self.spot, |s| s.0)
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
        r.zebras.retain(|z| z.at > back);
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
                route.zebras.retain(|z| z.at > back);
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
    /// Where it stops for those on foot at the zebras of the junctions giving way it goes through,
    /// should anyone be on one or stepping onto it (`traffic::zebra_stops`, `zebra_yield`), and at
    /// a lit junction's side road it turns into on the green, for anyone on its crosswalk
    /// (`traffic::side_zebra_stops`), worked out with its claims.
    pub(crate) zebras: Vec<crate::traffic::ZebraStop>,
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
        }
    }
}

// ---- the cars' helpers, split by what they do ----------------------------------------------

mod bodies;
mod driving;
mod garages;
mod parking;
mod seats;
mod trips;

pub use bodies::*;
pub(crate) use driving::*;
pub(crate) use garages::*;
pub(crate) use parking::*;
pub(crate) use seats::*;
pub(crate) use trips::*;

#[cfg(test)]
mod tests;
