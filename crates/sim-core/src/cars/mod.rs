//! Households' cars: parked on their driveway, driven to work (or school) and back.
//!
//! A car belongs to a household and stands on its home's driveway (the town file's `cars`; every
//! house's driveway is in its `drives`). Only its owners drive it, and only grown-ups (18 and up):
//! the household's grown-ups when the town was made (a household left with no owner hands it down
//! to its grown-ups). Whoever else in the household leaves at the same time rides along. When the
//! household moves house, the car goes with it, to the new home's driveway (`relocate`). Which car
//! it is follows the job of whoever in the household earns the most: a van for a trade, an
//! executive saloon high up in finance, newer up the ladder; a promotion or a new job buys another
//! once it stands idle (`looks.rs`).
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
//! Where its way crosses or meets another's, it takes turns with the other cars (`traffic/`):
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

/// Most cars in a town (the snapshot's rows): the households' and the work taxis'. A full city
/// world's households have a car in about one home in three (some 110 to 130), well inside it;
/// past it, cars are dropped as the town loads and no taxi is made.
pub const MAX_CARS: usize = 384;
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

/// The ground a car driving forwards keeps clear: `AHEAD` in front of it, none behind
/// (`Car::next_ground`).
fn forward_ground() -> [f32; 2] {
    [AHEAD, 0.0]
}
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

// ---- the cars' helpers, split by what they do ----

mod bay;
mod bodies;
mod driving;
mod garages;
mod lane;
mod legs;
mod looks;
mod motion;
mod parking;
mod parts;
mod route;
mod seats;
mod state;
mod trips;

pub use bay::*;
pub use bodies::*;
pub(crate) use driving::*;
pub(crate) use garages::*;
pub(crate) use lane::*;
pub(crate) use legs::*;
pub use looks::*;
pub use motion::*;
pub(crate) use parking::*;
pub use parts::*;
pub(crate) use route::*;
pub(crate) use seats::*;
pub use state::*;
pub(crate) use trips::*;

#[cfg(test)]
mod tests;
