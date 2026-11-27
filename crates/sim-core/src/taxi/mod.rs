//! The work taxi (content `taxi`): in an open-world town, someone off to work or school a long
//! way out of town on foot, with no seat in a car (their household's or a neighbour's lift) and
//! no bike or e-scooter for a ride of `life::BIKE_MOST` at most, rides a taxi the work (or the
//! school) sends, shared with those going the same way about the same time.
//!
//! **Who rides.** A resident whose way out of town on foot (`life::Commute::walk`) takes longer
//! than `taxi.minWalkMinutes`, and whom the taxi gets out of town sooner than the walk, its wait at
//! the kerb included (`time_run`). Not someone with a seat in a car to go in (`cars::lead_ticks`)
//! or a bike to ride (`Commute::bike`). With or without a driveway (`home_way`: else by the front
//! path).
//!
//! **Shared.** Those who leave town by the same way out, whatever their work (or school), and
//! whose work starts within `taxi.window` minutes of the first's share one (`group`), up to
//! `taxi.seats` a taxi, housemates side by side; more take more taxis. Those already too late to be
//! there in time on foot (a game begun) share one late run that way, whenever their work starts.
//!
//! **The run there** (`Run`). Planned `PLAN_LEAD` before the first of them would have set off on
//! foot: the riders in a short order from the way in, a stand at the kerb for each house
//! (`kerb_at`: in the lane, parallel to it, where the road runs straight for the body, clear of
//! junctions and of another taxi standing then), and the taxi's way in at that way out, to each
//! stand and out again as a car drives (`plan_way`, `lay_route`): on along its lane from a stand,
//! turning round only in a dead end's turning circle or nose first into a side street at a
//! junction, else round the block; round where cars are held up now (`jammed`). Timed back from when
//! it's to be out of town (the first one's work start less the part of the way off the map), each
//! stop `DWELL` long; too late for that (a game begun after they'd have set off, a rider missed
//! before), as soon as it can come, if that's still sooner than on foot (a late run). Each rider
//! leaves home in time to be at their kerb by then, or later as the taxi runs late (`leave_at`),
//! waits there ("Waiting for the taxi": `world::Phase::AtKerb`) and, as the taxi stands for them,
//! walks to its door and gets in (`door_for`, `world::Phase::Boarding`, `boarded`): their shift
//! under way, out of sight as in a household's car, the rest of the ride and the way off the map
//! still to go. Not at its door soon after it got there, they're left: it goes on without them, and
//! they're planned once more (`release`), else they make their own way. Someone just off on foot
//! when they're booked (as a game begins) goes back for it (`setting_off`, `book`).
//!
//! **Home.** Done at work, those who came by taxi, and those come on foot with a long walk home,
//! go home by one (`off_work`): the first done waits there for those of their group done within
//! `taxi.window` minutes (each `Run::depart`), then their taxi drives back into town by that way
//! out, stands at each one's stop while they get out (`cars::walk_home`) and leaves town again.
//! Riders done later than that make their own way.
//!
//! **On the road.** A taxi is a car (`cars::Car::taxi`) in every way that matters: its route
//! (`cars::Route`), its claims (its stretch of the lane held while it stands there, a turn round
//! while it turns), pulling over, the last-resort guard. A taxi's car slot is kept once it's come
//! (the snapshot's car rows, `KIND` a taxi) and taken again by the next run.
//!
//! **Saved** (`save/`'s `taxis`, absent: none): the runs planned and under way, where each taxi
//! has got to and who's at which stop; their routes, stands, slots and the bookings are worked out
//! again on load (`settle`).

use serde::{Deserialize, Serialize};

use crate::TICKS_PER_MINUTE;
use crate::world::{Phase, Task, TaskKind, World};

/// A taxi's length and width (m: a family estate's).
pub const TAXI_LEN: f32 = 4.7;
pub const TAXI_WID: f32 = 1.85;
/// A run there is planned this long before the first of its riders would have set off on foot
/// (ticks: before their walk is firmed up and they're off, `life::firm_leave`), and for work or
/// school starting no later than `PLAN_HORIZON` on.
const PLAN_LEAD: u64 = 3 * 60 * TICKS_PER_MINUTE;
/// Someone already off to work on foot this near their door is still booked on a taxi (m: off
/// to their kerb instead, `setting_off`).
const SETTING_OFF: f32 = 25.0;
const PLAN_HORIZON: u64 = 24 * 60 * TICKS_PER_MINUTE;
/// How long a taxi stands at a stop, in all, as its run is timed (ticks of real time: the door,
/// getting in or out, the door again).
const DWELL: u64 = crate::cars::GET_IN_TICKS as u64 + crate::secs(1.0);
/// How long before the taxi is due a rider is at their kerb (ticks), and how much spare time a run
/// is timed with (ticks).
const KERB_EARLY: u64 = 4 * TICKS_PER_MINUTE;
const SLACK: u64 = 4 * TICKS_PER_MINUTE;
/// And this share of the ride more (per cent: held up at junctions and by those in its way).
const SLACK_SHARE: u64 = 30;
/// How long the rider's walk from the house to the kerb takes, at least (ticks).
const TO_KERB_LEAST: u64 = 3 * TICKS_PER_MINUTE;
/// How far along the road from their driveway's middle a rider waits for the taxi at least (m,
/// `wait_spot`).
const WAIT_ASIDE: f32 = 2.0;
/// How far past the end of the taxi's stand a rider waits for it (m, `wait_spot`).
const WAIT_CLEAR: f32 = 0.6;
/// A walk to the taxi's door is about this much longer than straight there; and from where a
/// rider waits at their kerb to its far door it's this far at most (m): a rider called to its door
/// is waited for that walk at their pace after `taxi.waitMinutes` (`stand`), else left.
const WAY_ROUND: f32 = 1.3;
const DOOR_WALK: f32 = 6.0;
/// How long a taxi waits for a rider already on their way out to it (ticks, `stand`): at its
/// door within this at their pace, they're called to it, and waited for; else left (a rider
/// left walks the long way, as the taxi was brought in to spare them).
const COMING_WAIT: u64 = 5 * TICKS_PER_MINUTE;
/// A storey up or down on the way out of the house is about this much more walk (m: the
/// flight and round the landing, `indoor_lead`).
const STOREY_WALK: f32 = 6.0;

/// The runs planned and under way (`World::taxis`), and who rode one to work today (for the run
/// home).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Taxis {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runs: Vec<Run>,
    #[serde(default)]
    pub next_id: u32,
    /// Those who came to work by taxi, by their group (`group`): they go home by one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rode: Vec<(u32, (u32, u32))>,
    /// Those planned for no taxi for the start of work given (none fits them), not to be planned
    /// again for it. Not saved.
    #[serde(skip)]
    declined: Vec<(u32, u64)>,
    /// Those a taxi went on without for the start of work given, once: planned again (a late
    /// run, sooner out of town than on foot), but not after a second. Not saved.
    #[serde(skip)]
    missed: Vec<(u32, u64)>,
    /// Each car on its way: where it stood still last (none: not on its way), and since when (a tick), for the
    /// places held up (`jammed`). Not saved.
    #[serde(skip)]
    still: Vec<(Option<[f32; 2]>, u64)>,
    /// For tests: junctions' boxes held to be jammed whatever (`jammed`). Not saved.
    #[doc(hidden)]
    #[serde(skip)]
    pub jams: Vec<usize>,
    /// For checks: each rider's arrival at work by taxi against when their work started (ticks
    /// late; less than none, early), the latest 4096, of the runs timed to be there in time; and
    /// of the late runs (`time_run`: too late for that, sooner than on foot). Not saved.
    #[serde(skip)]
    pub arrivals: Vec<(u32, i64)>,
    #[serde(skip)]
    pub late_arrivals: Vec<(u32, i64)>,
    /// Those put down at a place by a leisure taxi, walking in (`alight_at`): a taxi back is booked
    /// once they're there (`book_backs`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub out_at: Vec<(u32, u32)>,
    /// Those turned down for a taxi to a place just now (`book_leisure` found none), walking there
    /// instead: not asked again till it's cleared (each hour). Not saved.
    #[serde(skip)]
    no_leisure: Vec<(u32, u32)>,
    /// Those a leisure taxi left at their kerb and booked again (sim, place: `release`), once:
    /// left again, they go their own way. Cleared each hour. Not saved.
    #[serde(skip)]
    left: Vec<(u32, u32)>,
    /// Those off home early from a place, their taxi back brought forward (sim, the run they were
    /// on: `bring_backs_forward`), once for each run. Cleared each hour. Not saved.
    #[serde(skip)]
    early: Vec<(u32, u32)>,
    /// Each house's stand with no other taxi about (`stop_for`), kept once found. Not saved.
    #[serde(skip)]
    kerbs: Kerbs,
}

mod jams;
#[cfg(test)]
mod kerb_tests;
mod kerbs;
mod leisure;
#[cfg(test)]
mod leisure_leaving_tests;
mod plan;
mod routes;
mod runs;
mod way;

use jams::*;
pub use kerbs::taxi_bays;
pub(crate) use kerbs::*;
pub(crate) use leisure::*;
pub use plan::*;
pub(crate) use routes::*;
pub use runs::*;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use way::stand_for_door;
use way::*;
