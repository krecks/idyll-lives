//! What a car is, and what it is about: its state, its trip and what it has seen.
//!
//! Split from `cars/mod.rs` (see `mod.rs`).

use super::*;

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
pub(crate) const DETOUR_AVOID: f32 = 1.0e5;

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
pub(crate) const SEEN_SLACK: f32 = 12.0;

/// One out of sight comes under the rules once it's this near coming into sight along its way
/// (ticks: the next, then each second of drive time up to 4 s, some 20 m at a town's speed),
/// clear of those in sight (`sight`).
pub(crate) const COMING_IN: [f32; 5] = [
    1.0,
    drive_ticks(1.0),
    drive_ticks(2.0),
    drive_ticks(3.0),
    drive_ticks(4.0),
];
/// How far apart (ticks: a quarter of a second of drive time, 1.5 m at a town's speed, 4 m on a
/// country road straight through a junction: less than the smallest box) a car's way into sight
/// is looked along (`sight`: its claims reaching into sight, and `METER`'s queue).
pub(crate) const SIGHT_STEP: f32 = drive_ticks(0.25);

/// Coming into sight (`sight`), a car out of it waits outside while one in sight standing still
/// (`METER_STILL`, or a taxi at its stop) is on its way in up to the first junction it takes
/// turns at in sight (else `ahead` ticks along it; `most` at most; sampled every `SIGHT_STEP`),
/// this near it past `GAP` (`gap`, m): the queue there has no room for it. Out of sight nobody
/// waits, so those coming would all come in at once and fill the seen area past what the rules
/// would have let through (Fernglade).
pub(crate) struct Meter {
    pub(crate) ahead: f32,
    pub(crate) most: f32,
    pub(crate) gap: f32,
}
pub(crate) const METER: Meter = Meter {
    ahead: drive_ticks(6.0),
    most: drive_ticks(75.0),
    gap: 1.0,
};
/// How long one in sight on its way has stood still for its queue to count as full (ticks of drive
/// time).
pub(crate) const METER_STILL: u64 = drive_secs(1.0);
/// How long one waits outside for room at most (ticks of drive time, `METER`): then it comes in
/// all the same.
pub(crate) const METER_MOST: u64 = drive_secs(5.0);

/// Two cars this near (m) touch: one the camera comes to touching one in sight is put clear
/// (`sight`), and one in sight touching one out of it isn't drawn (`hidden`). (Those under the
/// rules keep `GAP` apart.)
pub(crate) const TOUCH: f32 = 0.05;

/// Till when a car turned round waits out of the way (`Car::let_by`), and those it waits for to
/// go on: each car, how far along its way it was, in which state.
pub(crate) type LetBy = (u64, Vec<(u32, f32, CarState)>);

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
    /// What it looks like (the renderer's bodies and paints, `Content::car_bodies`): its body
    /// fits its household's best earner's job (`looks.rs`), its paint is the town's.
    pub body: u8,
    pub paint: u8,
    /// How old it is (by the grade of the job it fits). Left out when used, as every car was
    /// before cars fitted the job.
    #[serde(default, skip_serializing_if = "crate::content::CarAge::is_used")]
    pub age: crate::content::CarAge,
    /// The car a promotion or a new job calls for, bought as soon as this one stands idle at home
    /// or away at work (`looks::renew`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub want: Option<Want>,
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
    pub(crate) route: Option<(bool, Route)>,
    /// Where its driveway meets the road network, as the town file says (`Drive::road`, `s`);
    /// None: the nearest road (not saved).
    #[serde(skip)]
    pub(crate) meets: Option<(u32, f32)>,
    /// The way its driveway runs to the street (`Drive::way`; not saved): it drives out and in
    /// along it and parks nose in facing the other way. None: square to the road it meets.
    #[serde(skip)]
    pub(crate) axis: Option<[f32; 2]>,
    /// The town's cars drive its roads (`Travel::graph`) but there's no way by them from its
    /// driveway out of town: it stays parked, unused (`settle`; not saved).
    #[serde(skip)]
    pub no_way: bool,
    /// Ticks the drive out of town takes from where it parks (`settle`; not saved).
    #[serde(skip)]
    pub(crate) out_ticks: f32,
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
    pub(crate) waited: u32,
    /// Those on foot or on a bike it waits for this tick (resident ids; for checks:
    /// `traffic::waits_on`). Not saved.
    #[serde(skip)]
    pub(crate) on_foot: Vec<u32>,
    /// Ticks it has stood at a zebra's stop line for those on foot (`zebra_yield`). Not saved.
    #[serde(skip)]
    pub(crate) zebra_wait: u32,
    /// Waiting at a red light this tick (`Traffic::red_for`): those on foot may walk up to its
    /// nose (`in_the_way`), short of a crosswalk. Not saved.
    #[serde(skip)]
    pub(crate) at_red: bool,
    /// Where it is now, which way it faces, whether it's moving and which doors are open (bit
    /// `d` for door `d`); None while it's out of town or unused (`poses`; for the snapshot, not
    /// saved).
    #[serde(skip)]
    pub shown: Option<([f32; 2], f32, bool, u8)>,
    /// Driving, where it would be a tick on and which way it would face (`drive_on`): those on foot
    /// keep clear of it there too (`in_the_way`), as it waits for anyone it would come nearer to
    /// (`go_on`); turning tight, it swings round further than it is now. Not saved.
    #[serde(skip)]
    pub(crate) next: Option<([f32; 2], f32)>,
    /// The ground it keeps clear there in front of it and behind it (m: `Driving::ahead` and
    /// `behind`, as `go_on` waits for those in it), for those it waits for by name, who don't come
    /// back into it (`in_the_way`). Not saved.
    #[serde(skip, default = "forward_ground")]
    pub(crate) next_ground: [f32; 2],
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
    pub(crate) moved: bool,
    /// Whom it pulls over for, last it did (`pull_want`). Not saved.
    #[serde(skip)]
    pub(crate) pulled_for: Option<u64>,
    /// Ticks it and one in its way have waited for each other, ticks it has backed up since
    /// (`back_off`), and whether it can back up no further (the other backs up instead:
    /// `waits_back`). Not saved.
    #[serde(skip)]
    pub(crate) jam: (u32, u32, bool),
    /// Backed up out of the way of this one (`back_off`): it stays back while that one is near.
    /// Not saved.
    #[serde(skip)]
    pub(crate) backed_for: Option<u64>,
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
    pub(crate) metered: Option<u64>,
    /// The furthest along its way it has been, in sight, and since when (`unstick`, `unjam`).
    /// Not saved.
    #[serde(skip)]
    pub(crate) stall: (f32, u64),
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
    pub(crate) let_by: Option<LetBy>,
    /// The stretches of road it turned round from lately (`travel::Graph::edges`) and till when it
    /// keeps off them, turning round again (`AVOID_FOR`). Not saved.
    #[serde(skip)]
    pub(crate) avoided: Vec<(u32, u64)>,
    /// Its way out round a work taxi standing at the kerb in its lane, back in past it
    /// (`pass_stands`), laid on its way; saved, so a game loaded part way round is the same.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass_by: Option<PassBy>,
}

pub(crate) fn is_zero_f32(v: &f32) -> bool {
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
