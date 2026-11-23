//! Taking turns where cars' ways cross or meet (`World::traffic`): reservations, not give-way
//! rules, every car in one queue.
//!
//! **Zones and claims.** Where two ways may cross or meet so that both can't go on, there's a
//! zone: each junction's box (`boxes`: a 6 m circle round its middle, reaching on along an arm as
//! far as needed to keep a body in its lane there `ARMS_APART` from one on another arm close to
//! it). Each car's route (`cars::Route`) is looked at once, as it's laid out, for the zones it
//! drives through, and for stretches that are claims of their own: a driveway's, a bay's or a spot
//! on the road's mouth (a car turning out across the lane or in), and round a 6 m lane's tight
//! bend where it couldn't pass one coming the other way, both pulled over as far as they can
//! (`narrow`). Each is a part (`Claim`): the stretch of the route from just before its body first
//! touches it (`Claim::from`) to where it's out of it again (`Claim::to`), sampled every `SAMPLE` m
//! (more often turning) with its room on the far side (its length and `ROOM_GAP`). Parts its body
//! comes to before it's out of the last make a group (`Claim::group`): it can't stand clear between
//! them, so it asks for them together.
//!
//! **Asking.** A car about to drive into a group asks for it (`Traffic::may_go`) and waits where it
//! is till it's granted, all of it at once. The asks are granted once a tick (`Traffic::begin`,
//! before the cars go on), in the order they were asked (the tick, then `cars::Driving::first`),
//! each when
//! 1. none of it is in the way (`conflict`) of what's left of a part another holds,
//! 2. nor of an earlier ask still waiting,
//! 3. and nobody not holding a part is in it now, nor anyone at all on its far side ("don't block
//!    the box"), nor anyone on the move into it in the next second or so (`LOOK`), nor one turning
//!    round backing into it, however slowly (`Traffic::turning`).
//!
//! Two bodies are in each other's way (`conflict`) when they overlap or come within `CLEAR`; going
//! the same way (one following the other through, each keeping behind the one in front:
//! `cars::up_against`) they aren't; coming towards each other they are where pulling over as far as
//! both can there (`Sample::pass`) doesn't leave `PASS_GAP` between them. One right behind another
//! in its lane (`behind`) can't get anywhere before that one does: neither its parts nor its ask
//! hold that one up there.
//!
//! **Holding.** Holding a group, a car drives through it, letting go of each part as its body's
//! out of it. Anyone else, holding a group or not, keeps `KEEP_SHORT` short of what's left of one
//! another holds (`Traffic::blocked`; holding one itself, it may follow another through one going
//! its way; already in another's way, it goes on out of it).
//!
//! **Why nobody waits for good.** As a tick begins, who waits for whom is a graph: the cars' waits
//! on the road last tick (`Mover::waits`, `cars::Car::waits_for`), and the asks' waits for parts
//! held, bodies and earlier asks. Every rule that makes one wait for another leaves out one that
//! waits for it, however far round (`reaches`): an earlier ask doesn't hold up a later one it waits
//! for itself (rule 2), a body that waits for the asker doesn't (rule 3), one holding a part that
//! waits for the asker's body doesn't (rule 1), nor in `Traffic::blocked`. So the grants never
//! close a ring of waits the graph has in it: a ring forms only through waits it can't see (a
//! pull-over, a turn sweeping wider than its samples), and those are broken where they form: a car
//! and one in its way waiting for each other a while, the car backs up a little out of its way,
//! never into anyone (`cars::back_off`). An ask older than `OLD_ASK` yields only to one it waits
//! for itself, not to a stream round about: nobody waits on and on while others come and go; and
//! nobody comes up to wait in its way (`STOP_SHORT`, `Traffic::blocked`), to be let go first.
//! Should two bodies come to overlap anyway, a last-resort check keeps every car from moving
//! deeper into another (`cars::guard`, counted in `Traffic::guarded`).
//!
//! **Levels and merges.** Every body and every claim's sample has a level (`travel::Layer`): the
//! ground, or a motorway's deck over a road or the water (its carriageways and ramps one level
//! with the deck, on it or off it). Only those on levels that meet are in each other's way
//! (`conflict`, `cars::vehicles`, the walkers' `cars::in_the_way`): nobody under a deck waits for
//! a car on it, nor it for them; a junction's box, lights and zebras under a deck aren't the
//! deck's. A car coming off a ramp onto a motorway's carriageway claims its merge (`Claim::merge`,
//! from `cars::MERGE_UP` up the ramp to `cars::MERGE_ON` past it), granted by rule 3 only with
//! the carriageway's traffic clear of it for some 4 s of drive (`LOOK_FAST`), going its way too;
//! those on the carriageway claim nothing and never keep clear of a merge's claim (`blocked`):
//! they keep behind the car in front, the further back the faster they go
//! (`cars::follow_ahead`), as anyone. Off a carriageway onto a ramp, nobody claims anything.
//!
//! Nothing here is saved but the lights (`Light`): the zones and the claims are worked out again
//! with the routes, and a car found in a part it doesn't hold (a game just loaded) takes it as it
//! goes on.

use crate::cars::Rect;
use crate::travel::{Graph, dist};
use crate::world::World;

/// How far apart a route's samples are along it (m), and how far it turns between two at most
/// (radians: a car's ends swing round about 0.2 m).
pub(crate) const SAMPLE: f32 = 1.0;
const TURN_SAMPLE: f32 = 0.08;
/// The room a vehicle wants clear on the far side of a claim: its length, and this (m).
pub(crate) const ROOM_GAP: f32 = 1.0;
/// Two coming towards each other pass where pulling over takes them this far apart (m: the gap
/// the cars keep, `cars::GAP`, and a little: their ways are sampled a metre apart, and between two
/// a body turning comes a little nearer).
pub(crate) const PASS_GAP: f32 = crate::cars::GAP + 0.05;
/// Two going within about 25° of each other's way go the same way (the cosine).
const SAME_WAY: f32 = 0.9;
/// Bodies this near (m) are in each other's way (a body's corners between two samples too).
const CLEAR: f32 = 0.25;
/// One on the move, looked ahead this many ticks along its way (`Traffic::begin`: half a second,
/// one, one and a half, of drive time): it isn't made to stop short in front of one just granted.
pub(crate) const LOOK: [f32; 3] = [
    crate::drive_ticks(0.5),
    crate::drive_ticks(1.0),
    crate::drive_ticks(1.5),
];
/// One on the move faster than on a country road (the motorway), looked further ahead along its
/// way (`cars::movers`, ticks: two seconds of drive time to four): a car merging onto the
/// carriageway asks for its gap with room in the traffic coming along it (`Claim::merge`), some
/// 120 m at 30 m/s.
pub(crate) const LOOK_FAST: [f32; 5] = [
    crate::drive_ticks(2.0),
    crate::drive_ticks(2.5),
    crate::drive_ticks(3.0),
    crate::drive_ticks(3.5),
    crate::drive_ticks(4.0),
];
/// How far along its way a vehicle's ask is its own business with one right behind it (m,
/// `behind`): that one can't get there first. Further on (its way round and back) it may.
const NEAR_AHEAD: f32 = 40.0;
/// A junction's box: as far from its middle as two bodies in their lanes on any two of its arms
/// are this far apart (m), between `BOX_LEAST` and `BOX_MOST`; its arms looked along this far (m).
const ARMS_APART: f32 = 4.6;
const BOX_LEAST: f32 = 6.0;
const BOX_MOST: f32 = 14.0;
const ARM_LOOK: f32 = 5.0;
/// A box reaching further along an arm than `BOX_LEAST` (one near another) does so in circles
/// this wide (m) round the arm, this far apart along it (m).
const ARM_REACH: f32 = 2.5;
const ARM_STEP: f32 = 2.0;

mod conflict;
mod crossings;
mod flow;
mod gate;
mod sample;
mod signals;
#[cfg(test)]
mod tests;
mod waits;

pub(crate) use conflict::*;
pub use crossings::*;
pub use flow::*;
pub(crate) use gate::*;
pub use sample::*;
pub use signals::*;
pub use waits::*;
