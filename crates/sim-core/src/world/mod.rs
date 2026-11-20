//! World state and the per-tick update.

use crate::clock::{MAX_SPEED, SUPER_SPEED};
use crate::content::{
    Content, HabitTime, Layer, MAX_NEEDS, MAX_SKILLS, MAX_SLOTS, Modifiers, Pose, TagMask,
};
use crate::index::ObjectIndex;
use crate::lot::{BondRaw, Lot, LotFile, SimSpawn};
use crate::path::{Grounds, NavGrid};
use crate::rng::Rng;
use crate::social::{self, ActiveFeeling, EventLog, Relationships};
use crate::{Command, Error, MINUTES_PER_TICK, TICKS_PER_MINUTE, TICKS_PER_SECOND, conversation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

/// Something that happened while a resident stepped, for the story (applied after the step).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum News {
    Broke(u32),
    Repaired(u32),
    /// An accident (index into `Content::accidents`).
    Accident(usize),
    /// The player's order to close this object was dropped: someone else is in it now.
    NotClosed(u32),
}

/// How a house's roof looks: a roof style and a roof colour (indices into the content's
/// `roofStyles` and `roofColors`). Presentation only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoofLook {
    pub style: u8,
    pub color: u8,
}

/// A region of the town: a residential lot or a public place on the ground, or a flat (`of`): a
/// plot of its own inside its building's lot, on one storey (`storey`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Plot {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub z: i32,
    pub w: i32,
    pub d: i32,
    /// Public plots (parks) can be used by everyone.
    pub public: bool,
    /// Where visitors arrive (in front of the door); defaults to the plot centre.
    pub entry: Option<[f32; 2]>,
    /// The roof the player chose for the house on this plot (None: the town's own look).
    pub roof: Option<RoofLook>,
    /// When the place on it is open (None: always, as a home, a park or a plot from an older file).
    pub hours: Option<PlotHours>,
    /// What it is (the town file's `plots[].kind`; None: a file without kinds).
    pub kind: Option<crate::travel::PlotKind>,
    /// A flat: the ground lot of its building (a flats lot, or a place's with flats over it). Its
    /// rectangle is on the ground's rows, inside that lot's; its `entry` is the building's street
    /// door. None: a lot on the ground.
    pub of: Option<u32>,
    /// The storey a flat is on (0: the ground; a lot's is 0).
    pub storey: i8,
    /// A flat's landing tile before its own door, on its storey's rows (its door opens off an
    /// indoor landing, not the street).
    pub door: Option<(i32, i32)>,
    /// A flat's tile behind its door (its middle, on its storey's rows): where one home is home.
    pub inside: Option<[f32; 2]>,
}

/// A building with flats in it (`World::flat_tiles`, by its lot: `World::flats_in`): its lot's
/// rectangle (ground rows), and for each storey with flats on it the plot of each of the
/// rectangle's tiles, row-major (`path::NO_PLOT`: the building's own: its hall, its stairs, its yard).
#[derive(Debug, Clone, Default)]
pub(crate) struct FlatTiles {
    x: i32,
    z: i32,
    w: i32,
    d: i32,
    storeys: Vec<(i32, Vec<u16>)>,
}

impl FlatTiles {
    /// The flat at ground tile (`x`, `z`) of `storey`, if one is there.
    fn at(&self, x: i32, z: i32, storey: i32) -> Option<u32> {
        let (dx, dz) = (x - self.x, z - self.z);
        if dx < 0 || dz < 0 || dx >= self.w || dz >= self.d {
            return None;
        }
        let (_, tiles) = self.storeys.iter().find(|(s, _)| *s == storey)?;
        let plot = *tiles.get((dz * self.w + dx) as usize)?;
        (plot != crate::path::NO_PLOT).then_some(u32::from(plot))
    }
}

/// Checks a town's flats (`Plot::of`): each comes after every lot on the ground (a lot's storey is
/// the ground's), stands in a lot on the ground on one of its `storeys` (not the basement), inside
/// that lot's rectangle, and its door and the tile behind it, where it says them, are in that lot
/// on its storey's rows (as `stairs` stacks them, `storeys.rs`).
pub(crate) fn check_flats(
    plots: &[Plot],
    storeys: i32,
    stairs: &crate::path::StairMap,
) -> Result<(), Error> {
    let ground = plots.iter().take_while(|p| p.of.is_none()).count();
    if let Some(lot) = plots[..ground].iter().find(|p| p.storey != 0) {
        return Err(Error::new(format!(
            "lot '{}' is on a storey (only a flat is)",
            lot.name
        )));
    }
    for p in &plots[ground..] {
        let bad = |why: &str| Err(Error::new(format!("flat '{}' {why}", p.name)));
        let Some(of) = p.of else {
            return Err(Error::new(format!(
                "lot '{}' comes after the flats (flats follow every lot on the ground)",
                p.name
            )));
        };
        let Some(lot) = plots[..ground].get(of as usize) else {
            return bad("is in no lot on the ground");
        };
        if !(0..storeys).contains(&i32::from(p.storey)) {
            return bad("is on no storey the town has");
        }
        if p.x < lot.x || p.z < lot.z || p.x + p.w > lot.x + lot.w || p.z + p.d > lot.z + lot.d {
            return bad("reaches out of its building's lot");
        }
        let on_storey = |x: i32, z: i32| lot.holds(stairs, x, z) && p.on_storey(stairs, z);
        if p.door.is_some_and(|(x, z)| !on_storey(x, z))
            || p.inside
                .is_some_and(|[x, z]| !on_storey(x.floor() as i32, z.floor() as i32))
        {
            return bad("has its door out of its building on its storey");
        }
    }
    Ok(())
}

/// The lots on the ground of `plots`: every plot before the first flat (`check_flats` keeps the
/// flats after them all). Passes over the town's ground take these, and a flat's rectangle, its
/// building's ground, is never taken for a lot of its own.
pub(crate) fn lots_of(plots: &[Plot]) -> &[Plot] {
    &plots[..plots.partition_point(|p| !p.is_flat())]
}

/// The flats of `plots`: every plot after the lots on the ground (`lots_of`).
pub(crate) fn flats_of(plots: &[Plot]) -> &[Plot] {
    &plots[lots_of(plots).len()..]
}

/// When a public place is open: minutes of the day `[open, close)` on the days in `days` (bit 0
/// Monday), as the town file and saves carry it (`plots[].hours`, the page's `TownPlace.hours`).
/// A `close` at or before `open` runs on past midnight into the next morning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlotHours {
    pub open: u16,
    pub close: u16,
    pub days: u8,
}

impl Plot {
    pub fn contains(&self, x: i32, z: i32) -> bool {
        x >= self.x && z >= self.z && x < self.x + self.w && z < self.z + self.d
    }

    /// Where one coming to it arrives from the street: its entry (a flat's: its building's street
    /// door, on the ground), else its middle. What travel, taxis, cars and outings measure from.
    pub fn arrival_point(&self) -> [f32; 2] {
        self.entry.unwrap_or([
            self.x as f32 + self.w as f32 / 2.0,
            self.z as f32 + self.d as f32 / 2.0,
        ])
    }

    /// Where one who lives on it is at home: a flat's tile behind its door (on its storey's rows),
    /// else where it is arrived at (`arrival_point`).
    pub fn home_point(&self) -> [f32; 2] {
        self.inside.unwrap_or_else(|| self.arrival_point())
    }

    /// Whether tile (`x`, `z`: stacked rows, as `stairs` has them) is on it: inside its rectangle
    /// on any storey, and a flat's on its own storey.
    pub fn holds(&self, stairs: &crate::path::StairMap, x: i32, z: i32) -> bool {
        self.contains(x, stairs.ground_z(z)) && self.on_storey(stairs, z)
    }

    /// Whether row `z` (stacked rows, as `stairs` has them) is on its storey: any storey for a
    /// lot on the ground (what stands upstairs on it is on it too), a flat's own only.
    pub fn on_storey(&self, stairs: &crate::path::StairMap, z: i32) -> bool {
        !self.is_flat() || stairs.storey(z) == i32::from(self.storey)
    }

    /// Its rows on its own storey (stacked rows, as `stairs` has them): a flat's on its storey, a
    /// lot's on the ground (a lot's storey is 0).
    pub fn rows(&self, stairs: &crate::path::StairMap) -> std::ops::Range<i32> {
        let up = stairs.storey_row(i32::from(self.storey));
        self.z + up..self.z + self.d + up
    }

    /// Where one coming to it stands at its door: a flat's landing before its own door (on its
    /// storey's rows), else where it is arrived at (`arrival_point`).
    pub fn doorstep(&self) -> [f32; 2] {
        match self.door {
            Some((x, z)) => [x as f32 + 0.5, z as f32 + 0.5],
            None => self.arrival_point(),
        }
    }

    /// Whether it is a flat (a plot inside its building's lot, `of`).
    pub fn is_flat(&self) -> bool {
        self.of.is_some()
    }

    /// The lot on the ground it stands on: a flat's building, else itself.
    pub fn lot(&self) -> u32 {
        self.of.unwrap_or(self.id)
    }

    /// Whether the place on it is open at `minute` of day `day` (`clock::day`, `clock::minute_of_day`):
    /// always without hours. Hours past midnight count for the day they opened on.
    pub fn open_at(&self, day: u32, minute: f32) -> bool {
        let Some(h) = self.hours else {
            return true;
        };
        let weekday = crate::clock::weekday(day);
        let on = |wd: u32| h.days & (1 << wd) != 0;
        let (open, close) = (f32::from(h.open), f32::from(h.close));
        if open < close {
            on(weekday) && minute >= open && minute < close
        } else {
            (on(weekday) && minute >= open) || (on((weekday + 6) % 7) && minute < close)
        }
    }

    /// `open_at` at `tick`.
    pub fn open_at_tick(&self, tick: u64) -> bool {
        self.hours.is_none()
            || self.open_at(crate::clock::day(tick), crate::clock::minute_of_day(tick))
    }

    /// How many minutes from `minute` of day `day` the place stays open: 0 if it's shut then,
    /// unending without hours (or open round the clock). One opening running into the next (to
    /// midnight, and from it on the next day) counts as one.
    pub fn open_for(&self, day: u32, minute: f32) -> f32 {
        let Some(h) = self.hours else {
            return f32::INFINITY;
        };
        let (open, close) = (f32::from(h.open), f32::from(h.close));
        let (mut day, mut minute, mut total) = (day, minute, 0.0);
        // (A week of openings back to back: open all the time.)
        for _ in 0..8 {
            if !self.open_at(day, minute) {
                return total;
            }
            // To the close of the opening it's in: today's, or (past midnight) the one that
            // began the day before.
            let left = if open < close || minute < close {
                close - minute
            } else {
                MINUTES_PER_DAY - minute + close
            };
            total += left;
            let end = minute + left;
            day += (end / MINUTES_PER_DAY).floor() as u32;
            minute = end.rem_euclid(MINUTES_PER_DAY);
        }
        f32::INFINITY
    }

    /// When the place next opens after `minute` of day `day`: the day and the minute of it. None
    /// without hours, or with no day it opens on.
    pub fn opens_next(&self, day: u32, minute: f32) -> Option<(u32, u16)> {
        let h = self.hours?;
        (0..8u32)
            .map(|k| day + k)
            .find(|&d| {
                h.days & (1 << crate::clock::weekday(d)) != 0
                    && (d > day || f32::from(h.open) > minute)
            })
            .map(|d| (d, h.open))
    }
}

/// Minutes in a day (`Plot::open_for`).
const MINUTES_PER_DAY: f32 = 24.0 * 60.0;

/// The days of the week from Monday (`clock::weekday`), as a place shut till another day says when
/// it opens.
const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

/// How a game treats money for building: chosen once per game.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    /// The residents earn the money: build and buy are paid from household funds.
    #[default]
    Living,
    /// Building is free: build and buy cost nothing (selling brings nothing back), and the
    /// player's home pays no rent or bills.
    Creative,
}

#[derive(Debug, Clone)]
pub struct Household {
    pub id: u32,
    pub name: String,
    /// The plot this household lives on.
    pub plot: Option<u32>,
    /// Controlled by the player.
    pub player: bool,
    /// Money earned from jobs.
    pub funds: i64,
    /// "Free will": whether this household's residents choose what to do on their own.
    pub free_will: bool,
    /// Routine blocks every member follows (each may skip some; their own blocks win).
    pub routines: Vec<crate::planner::Routine>,
    /// A baby on the way (see `lifecycle::conceive`).
    pub expecting: Option<Expecting>,
    /// The day a babysitter was booked for (see `life::childcare`).
    pub sitter_day: Option<u32>,
    /// Loads of washing waiting: dirty (piling up as content `laundry` says), washed, to dry,
    /// dried, to fold (or iron), and ironed, to wear (`content::WorkOn::load`).
    pub laundry: [f32; LOADS],
    /// Letters on their way to the household's letterbox, by who wrote them, the oldest first
    /// (`post::send`): read as the post is fetched (`post::read`).
    pub letters: Vec<u32>,
}

/// The piles of washing a household has (`Household::laundry`, `content::WorkOn::load`).
pub const LOADS: usize = 4;

/// Loads of washing a new household has waiting, dirty and washed (one of each: there's always
/// a bag of washing on moving in); none dry yet.
pub const LAUNDRY_AT_START: f32 = 1.0;

/// A new household's washing waiting (`Household::laundry`).
pub const LAUNDRY_START: [f32; LOADS] = [LAUNDRY_AT_START, LAUNDRY_AT_START, 0.0, 0.0];

/// A household expecting a baby: the parents and the day it's due.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expecting {
    pub parents: [u32; 2],
    pub due: u32,
}

/// A resident who is no longer in town: they died or moved away (see `lifecycle.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gone {
    pub why: GoneWhy,
    /// Game day it happened.
    pub day: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GoneWhy {
    Died,
    MovedAway,
}

/// Someone whose slot a newcomer took: kept so the story can still name them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Former {
    pub name: String,
    /// Their household's name at the time.
    pub household: String,
}

/// Per-tick snapshot of other Sims, so each Sim can reason about the others
/// without aliasing the mutable Sim list. Reused between ticks.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SimBrief {
    pub pos: [f32; 2],
    pub available: bool,
    pub gender_ix: Option<usize>,
    pub household: u32,
    /// Plot the Sim is standing on.
    pub plot: Option<u32>,
    pub away: bool,
    /// On their way off their home plot, now or later in their queue (`life::leaving_home`; so
    /// the last grown-up at home with a baby knows they will be: `life::minding_baby`).
    pub leaving: bool,
    /// A baby (cared for in a crib), and their needs (so carers know what's wanted).
    pub baby: bool,
    pub needs: [f32; MAX_NEEDS],
    /// Has a partner.
    pub partnered: bool,
    /// Still in town, grown up, and how old.
    pub here: bool,
    pub adult: bool,
    pub age: f32,
    /// In a conversation (leading it, being talked to or listening in): who started it and
    /// the social.
    pub conversing: Option<(u32, usize)>,
    /// The tile they're tidying up (or on their way to), if any.
    pub cleaning: Option<(i32, i32)>,
    /// Sitting down (on a sofa, at a table): someone sitting near can talk to them from their seat.
    pub seated: bool,
    /// Lying down (asleep in bed, napping on the sofa): nobody to answer the door.
    pub lying: bool,
    /// Eating sitting down: talked to only from close by in their seat (not a hug), the meal
    /// left a moment (`conversation::keeps_seat`).
    pub eating: bool,
    /// The game for two they're at or on their way to (`Sim::game`).
    pub game: Option<(u32, usize, bool)>,
    /// Free to be fetched for one (`partners::free_now`).
    pub free: bool,
    /// On their way to this car for work or school (`Sim::bound_for_car`).
    pub bound_for_car: Option<u32>,
    /// On their way to this car for a trip, or in it (`cars::Trip`: a `Visit` or `GoHome`).
    pub trip_car: Option<u32>,
    /// The bike their way to work or school is timed by (`life::Commute::bike`): not a
    /// housemate's to take (`ride::bike_for`).
    pub bike: Option<u32>,
    /// Where their bike stands left out by a place (`Sim::bike_spot`): not another's to leave there.
    pub bike_spot: Option<[f32; 2]>,
    /// The plot they're a guest on (`Sim::visiting`).
    pub guest_at: Option<u32>,
    /// The seat they eat at, or are on their way to with something to eat (`cooking::company`).
    pub eats_at: Option<u32>,
    /// Carried by someone (`Sim::carried_by`), and the use behind a shut door they're at
    /// (`privacy::private_use`): for the rooms kept shut, found from the briefs rather than
    /// from the whole of every resident again (`privacy::refresh`).
    pub carried: bool,
    pub private: Option<(u32, usize)>,
    /// In the stair hall (or on its landings) of a public place with flats over it
    /// (`path::Halls`): on the place's lot, but not in the place (`world::at_home`).
    pub hall: bool,
}

impl SimBrief {
    /// Whether they stand at home, for those living on `home`: on a plot that is home to them
    /// (`at_home`), in its stair hall or not as they stand (`hall`). Off every plot, not.
    pub fn at_home(&self, plots: &[Plot], home: Option<u32>) -> bool {
        self.plot
            .is_some_and(|p| at_home(plots, home, p, self.hall))
    }
}

/// Who belongs to each household by the tick's briefs (`SimBrief::household`), built with them:
/// a look through one household's people needn't go through the whole town's. Each household's
/// in the residents' order, as a look through the briefs finds them, so what it finds is the same.
#[derive(Debug, Default)]
pub(crate) struct Members {
    /// Where each household's run starts in `list` (one more entry than households: the end).
    start: Vec<u32>,
    list: Vec<u32>,
    /// A resident moved house or came to town since (`lifecycle::move_to`, `World::spawn_at`):
    /// the residents as they are now no longer match it (`World::household_sims`).
    pub moved: bool,
}

impl Members {
    /// Sorts the briefs' residents by household (a counting sort, so each household keeps the
    /// residents' order).
    pub(crate) fn build(&mut self, briefs: &[SimBrief]) {
        let n = briefs
            .iter()
            .map(|b| b.household as usize + 1)
            .max()
            .unwrap_or(0);
        self.start.clear();
        self.start.resize(n + 1, 0);
        for b in briefs {
            self.start[b.household as usize + 1] += 1;
        }
        for h in 0..n {
            self.start[h + 1] += self.start[h];
        }
        self.list.clear();
        self.list.resize(briefs.len(), 0);
        // (Each household's next free place, counted up from its start and back after.)
        for (j, b) in briefs.iter().enumerate() {
            let h = b.household as usize;
            self.list[self.start[h] as usize] = j as u32;
            self.start[h] += 1;
        }
        for h in (1..=n).rev() {
            self.start[h] = self.start[h - 1];
        }
        if let Some(first) = self.start.first_mut() {
            *first = 0;
        }
        self.moved = false;
    }

    /// The residents of household `h` by the briefs (here or not), in the residents' order.
    pub(crate) fn of(&self, h: u32) -> &[u32] {
        let h = h as usize;
        if h + 1 >= self.start.len() {
            return &[];
        }
        &self.list[self.start[h] as usize..self.start[h + 1] as usize]
    }
}

/// `World::broadcast`'s per-tick buffers, kept from tick to tick like `briefs`.
#[derive(Default)]
struct BroadcastScratch {
    /// The things residents are using that broadcast to others.
    sources: Vec<BroadcastSource>,
    /// Per Sim: the room they're in, their storey, their plot, and whether they're asleep.
    rooms: Vec<u16>,
    storeys: Vec<i32>,
    plots: Vec<Option<u32>>,
    asleep: Vec<bool>,
    /// The objects one Sim has already heard this tick.
    heard: Vec<u32>,
}

/// A `Use` that broadcasts to residents nearby (`World::broadcast`).
struct BroadcastSource {
    object: u32,
    room: u16,
    storey: i32,
    plot: Option<u32>,
    pos: [f32; 2],
    gains: [f32; MAX_NEEDS],
    share: f32,
    range: f32,
    user: u32,
}

pub struct World {
    pub content: Content,
    /// Living or Creative (see `GameMode`).
    pub mode: GameMode,
    pub lot: Lot,
    pub objects: Vec<ObjectInstance>,
    /// The next stable object id (see `ObjectInstance::stable`); never reused.
    pub(crate) next_object_id: u32,
    /// Food, plates and pots (see `items.rs`).
    pub items: crate::items::Items,
    /// Things bought that wait to be set up at home (see `setup.rs`).
    pub setups: Vec<crate::setup::Setup>,
    /// Plot index per object (`None` when outside every plot).
    pub(crate) object_plot: Vec<Option<u32>>,
    /// Objects by plot, for decisions (see `index.rs`); kept up to date by `refresh_index`.
    pub(crate) object_index: ObjectIndex,
    pub sims: Vec<Sim>,
    pub households: Vec<Household>,
    pub plots: Vec<Plot>,
    /// Plot per ground tile, row by row (see `map_plots`; `path::NO_PLOT` off the plots). A flats
    /// building's tiles hold its lot; its flats are found per storey in `flat_tiles`.
    pub(crate) tile_plot: Vec<u16>,
    /// Each building with flats in it, the plot of its tiles on each storey (`FlatTiles`), and by
    /// ground lot which of them is its (index + 1; 0: none). Made with `tile_plot`.
    pub(crate) flat_tiles: Vec<FlatTiles>,
    pub(crate) flats_in: Vec<u16>,
    /// The flats' stair halls over public places (`path::Halls`), found with the rooms
    /// (`map_halls`).
    pub(crate) halls: crate::path::Halls,
    pub relationships: Relationships,
    pub events: EventLog,
    pub(crate) blocked: Vec<bool>,
    pub tick: u64,
    /// The game's speed (`clock::TICKS_PER_STEP`'s index): 0 paused .. `SUPER_SPEED`.
    pub speed: u8,
    /// The speed below super speed last chosen (1 to 3): super speed gone to by itself drops back
    /// to it when the household stops being quiet.
    pub resume_speed: u8,
    /// Whether the game went to super speed by itself (`auto_fast`) rather than the player picking
    /// it: only then does it drop back as the quiet ends.
    pub(crate) auto_super: bool,
    /// Whether it was quiet at home (`calm`) at the last step: super speed by itself starts as
    /// it falls quiet, so a speed the player picks while it is quiet holds.
    pub(crate) was_calm: bool,
    /// Ticks the last step ran (the snapshot's `RATE` at super speed).
    pub(crate) step_ticks: u32,
    /// Master switch for "free will" in the whole town (each household also has its own,
    /// `Household::free_will`). Off is mainly for tests.
    pub autonomy: bool,
    /// How fast residents age (a per-game option).
    pub lifespan: crate::lifecycle::Lifespan,
    /// Whether the player's residents move in with partners and out of the family home on
    /// their own (a per-game option; others always do).
    pub player_moves: bool,
    /// The most residents the town grows to by newcomers and births, as its town file asks
    /// (`residentsMost`; none: `MAX_SIMS`). Kept as given and saved with the town, so a town made
    /// for fewer stays so on a machine that could hold more. Read through `residents_most`.
    pub residents_most_given: Option<u32>,
    /// People whose slot someone new took, for the story (`social::FORMER` references).
    pub former: Vec<Former>,
    /// Bumped per slot when someone new takes it (views rebuild that resident).
    pub(crate) generations: Vec<u32>,
    pub(crate) rng: Rng,
    /// The seed the game started from (0: unknown), for bug reports.
    pub seed: u32,
    pub(crate) structure_version: u32,
    /// Bumped only when something a walker can or can't step through changes: walls, doors,
    /// windows, stairs, storeys, and things placed or removed. Looks, repairs and crops don't
    /// bump it, so the walkable regions (and the objects-by-plot index) needn't be rebuilt for
    /// them. `structure_version` stays the one views and offers follow.
    pub(crate) nav_version: u32,
    /// The last changes to walking and where they were (`nav_changed_in`), so walkers far from
    /// them keep their way.
    pub(crate) nav_log: crate::path::NavLog,
    /// `path::nodes_searched` as this tick began: what its searches cost so far is the
    /// difference (`over_search_budget`). Not saved.
    pub(crate) search_mark: u64,
    /// The tiles a tick's searches may expand before turns that may search wait for the next
    /// (`SEARCH_BUDGET`; open-world towns only). Not saved.
    pub search_budget: u64,
    /// Bumped when objects come, go or move (or walls change), not when only a car parks: the
    /// objects-by-plot index, the links and the staff plan follow it.
    pub(crate) objects_version: u32,
    /// The walkable regions of the lot at `reach_version` (see `path::Reach`).
    pub(crate) reach: crate::path::Reach,
    pub(crate) reach_version: u32,
    /// Bumped when the lot itself changes (walls, doors, windows, fences, coverings, floors),
    /// not when only objects, residents or households do: views can then leave the lot out.
    pub(crate) lot_version: u32,
    pub(crate) briefs: Vec<SimBrief>,
    /// The briefs' residents by household (`Members`), built with them.
    pub(crate) members: Members,
    /// Rooms kept shut as the tick's step started (`privacy::refresh`), kept from tick to tick
    /// like `briefs`.
    pub(crate) shut: Vec<crate::privacy::Shut>,
    /// Per resident: whether they keep a room in `shut` shut (`privacy::keeps_shut`).
    pub(crate) shut_by: Vec<bool>,
    /// The rooms kept shut while anyone is in them (`privacy::private_room`), sorted; found
    /// with the rooms (`refresh_rooms`).
    pub(crate) private_rooms: Vec<u16>,
    /// Among those, the rooms of a claimed kind (`roomKinds[].claimed`: bedrooms), whose door its
    /// own people (`privacy::Shut::claimants`) have the run of, sorted; found with the rooms.
    pub(crate) claimed_rooms: Vec<u16>,
    /// The objects that can offer a place in each of those rooms (`privacy::claimed_places`),
    /// broken ones included, found with them, so a tick needn't look through every object in the
    /// home.
    pub(crate) claimed_places: Vec<(u16, Vec<u32>)>,
    /// The people who have a place in each claimed room, with the fingerprint they were found
    /// for and the tick it was last checked in (`privacy::refresh`), so a tick needn't look for
    /// them again while nothing changes. In the rooms' order.
    pub(crate) claimants_cache: Vec<(u16, u64, std::sync::Arc<[u32]>, u64)>,
    /// What each household has to work on as the tick's step started (`work_waiting`), kept
    /// from tick to tick like `briefs`.
    work: Vec<u8>,
    /// Who goes first when the tick's searches run over budget (`tick_once`), kept from tick to
    /// tick like `briefs`.
    pub(crate) first: Vec<bool>,
    /// The residents taking a turn this tick (`Sim::steps` over 0 as `lod::plan` gave them), in
    /// order: the passes after the step that are only for them go through these, not the town
    /// (`life::update`). Not while `turns_fresh` is off: someone came to town since.
    pub(crate) turns: Vec<u32>,
    pub(crate) turns_fresh: bool,
    /// `broadcast`'s per-tick buffers, kept from tick to tick like `briefs`.
    broadcast_scratch: BroadcastScratch,
    /// Where Sims leave town for work.
    pub exits: Vec<[f32; 2]>,
    /// The road network and the town file's other open-world fields (`travel/`).
    pub travel: crate::travel::Travel,
    /// Households' cars (see `cars.rs`), and every house's driveway (where a car goes when its
    /// household moves).
    pub cars: Vec<crate::cars::Car>,
    /// The claims cars hold and ask for where their ways cross or meet
    /// (`traffic::Traffic`). Not saved.
    pub traffic: crate::traffic::Traffic,
    pub drives: Vec<crate::cars::Drive>,
    /// The work taxis' runs, planned and under way (`taxi/`).
    pub taxis: crate::taxi::Taxis,
    /// Garages at the inner ends of driveways (`cars::Garage`).
    pub garages: Vec<crate::cars::Garage>,
    /// Porches over doors, and porch pieces standing on their own (`porches.rs`).
    pub porches: Vec<crate::porches::Porch>,
    pub porch_pieces: Vec<crate::porches::PorchPiece>,
    /// The tiles parked cars take up in `blocked` (`cars::reblock`), sorted.
    pub(crate) car_tiles: Vec<usize>,
    /// The `nav_version` the cars' parking spots were last checked at (`cars::update`).
    pub(crate) cars_nav: u32,
    /// Which cars (a bit each, 64 to a word) stood parked when `car_tiles` was worked out; None:
    /// work it out again (`cars::reblock`).
    pub(crate) cars_parked: Option<Vec<u64>>,
    /// Cars by where they are, so a check that pairs one with another need only look at those
    /// near it (`cars::Near`; built each tick, `cars::update`).
    pub(crate) cars_near: crate::cars::Near,
    /// Opaque presentation data from the lot/town file, saved unchanged.
    pub meta: serde_json::Value,
    /// The town's paths and driveways (`meta.paths`) as rects (`x0, z0, x1, z1`): where a bike left
    /// out by a place doesn't stand (`ride::park_spot`). Worked out with `meta` (`note_paths`).
    pub(crate) paths: Vec<[f32; 4]>,
    /// Build and buy edits that can be taken back, oldest first (see `World::undo`).
    pub(crate) undo: Vec<crate::home::HomeSnapshot>,
    /// Edits taken back that can be made again, latest last (see `World::redo`).
    pub(crate) redo: Vec<crate::home::HomeSnapshot>,
    /// Super speed by itself: the game goes to `SUPER_SPEED` as the player's household falls
    /// asleep or goes out with nothing notable at home in the last hour (`World::calm`).
    pub auto_fast: bool,
    /// What each household's home (plus public places) offers, for planned activities;
    /// recomputed when the structure changes.
    pub(crate) offers: Vec<crate::planner::HouseOffer>,
    pub(crate) offers_version: u32,
    /// Dirt per tile (0..1), from using things and accidents; cleaned by residents.
    pub dirt: Vec<f32>,
    /// Every room and garden with its scores (see `rooms.rs`), and the layout version they
    /// were computed for.
    pub(crate) rooms: Vec<crate::rooms::RoomInfo>,
    pub(crate) rooms_version: u32,
    /// Bumped when the lot's rooms or their scores change structurally (walls, coverings,
    /// objects added or removed). Unlike `structure_version`, a cupboard opening or a crop
    /// growing doesn't bump it, so a busy town needn't rebuild every room for a change of looks.
    pub(crate) room_layout_version: u32,
    /// Things came, went, broke, were mended or upgraded since the rooms were scored
    /// (`rooms_things_changed`).
    pub(crate) rooms_things: bool,
    /// The tiles (`(x, z)`) where things changed (empty with `rooms_things_all`).
    pub(crate) rooms_things_at: Vec<(i32, i32)>,
    /// Things may have changed anywhere: every room is scored again for them.
    pub(crate) rooms_things_all: bool,
    /// Per room id: it has a door to the outside (`privacy::doors_out`, found with the rooms).
    pub(crate) rooms_out: Vec<bool>,
    /// A room's scores changed without the layout changing (a thing broke, was repaired or
    /// upgraded); the next tick recomputes the rooms once.
    pub(crate) rooms_dirty: bool,
    /// Only cleanliness may have changed (dirt, or a look that doesn't move anything); the next
    /// tick rescores the rooms without walking the lot again.
    pub(crate) rooms_rescore: bool,
    /// Overall score of the room or garden each tile is in (`roomRules.away` off the plots).
    pub(crate) room_score: Vec<f32>,
    /// Which tiles make up each room, to rescore cleanliness after messes (see `rooms.rs`).
    pub(crate) room_tiles: crate::rooms::RoomTiles,
    /// How many storeys the lot holds (see `storeys.rs`); 1: only the ground.
    pub storeys: u8,
    /// Rows per storey and the stairs between them, for walking.
    pub(crate) stairs: crate::path::StairMap,
    /// Lower detail for lots nobody watches (see `lod.rs`); on unless turned off for tests and
    /// comparisons. It only applies once there's a view.
    pub lod: bool,
    /// The part of the town being looked at (`set_view`); None: all of it.
    pub(crate) view: Option<[i32; 4]>,
    /// What the renderer draws (`set_watch`, for the cars in sight: `cars::sight`); None: all of
    /// it. Not saved.
    pub(crate) watch: Option<[i32; 4]>,
    /// Per plot: watched this tick (see `lod::plan`).
    pub(crate) watched: Vec<bool>,
    /// Bumped when a lamp is switched on or off (see `lights.rs`); not a structure change.
    pub(crate) lights_version: u32,
    /// What residents switching lights saw last (see `lights::update`).
    pub(crate) light_watch: crate::lights::Watch,
    /// Bumped when curtains are drawn or opened (see `curtains.rs`); not a structure change.
    pub(crate) curtains_version: u32,
    /// What residents drawing curtains saw last (see `curtains::update`).
    pub(crate) curtain_watch: crate::curtains::Watch,
    /// The lights that follow a use (content `light.while`) that are lit now (object ids): a cooker
    /// hood's light while the hob under it cooks (see `lights::update_while`). Not saved.
    pub(crate) while_lit: Vec<u32>,
    /// Who serves at the town's stalls and counters (see `staff.rs`); not saved.
    pub staff: crate::staff::Staff,
    /// The walk out of town last timed from where each resident was (`life::firm_leave`). Not
    /// saved: a memo of what it's worked out from.
    pub(crate) walk_memo: crate::life::WalkMemo,
}

/// Read-only context for one Sim's update.
pub(crate) struct Ctx<'a> {
    pub content: &'a Content,
    pub nav: NavGrid<'a>,
    /// Which tiles can reach which (see `path::Reach`): a goal elsewhere is refused at once.
    pub reach: &'a crate::path::Reach,
    /// Bumped when the house changes in a way that can cut a walker's route short.
    pub nav_version: u32,
    /// Where those changes were (`World::nav_log`).
    pub nav_log: &'a crate::path::NavLog,
    pub rels: &'a Relationships,
    pub briefs: &'a [SimBrief],
    /// The briefs' residents by household (`World::members`).
    pub members: &'a Members,
    pub households: &'a [Household],
    pub plots: &'a [Plot],
    /// Plot per ground tile (`World::tile_plot`).
    pub tile_plot: &'a [u16],
    /// The flats' stair halls over public places (`World::halls`).
    pub halls: &'a crate::path::Halls,
    pub object_plot: &'a [Option<u32>],
    /// Objects by plot (`World::object_index`).
    pub index: &'a ObjectIndex,
    pub exits: &'a [[f32; 2]],
    /// The road network (`World::travel`).
    pub travel: &'a crate::travel::Travel,
    /// Households' cars and the work taxis (`World::cars`).
    pub cars: &'a [crate::cars::Car],
    /// The lit junctions' lights (`Traffic::lights`): their crosswalks open on the walk phase.
    pub lights: &'a [crate::traffic::Light],
    /// The work taxis' runs (`World::taxis`).
    pub taxis: &'a crate::taxi::Taxis,
    pub autonomy: bool,
    /// Whether the town has room for one more by a birth (`World::room_in_town`), worked out
    /// once a tick rather than by every pair asking.
    pub room_in_town: bool,
    /// Room score per tile (`World::room_score`).
    pub room_score: &'a [f32],
    /// Dirt per tile (`World::dirt`).
    pub dirt: &'a [f32],
    pub hour: f32,
    pub day: u32,
    pub minute: f32,
    pub tick: u64,
    /// What each household has to work on (`work_waiting`), by household.
    pub work: &'a [u8],
    /// Rooms kept shut now, and by whom (`privacy::refresh`).
    pub shut: &'a [crate::privacy::Shut],
    /// Every room and garden with its scores (`World::rooms`).
    pub rooms: &'a [crate::rooms::RoomInfo],
    /// Who serves at the town's stalls and counters (`World::staff`).
    pub staff: &'a crate::staff::Staff,
    /// The town's paths and driveways (`World::paths`).
    pub paths: &'a [[f32; 4]],
    /// The other residents, as far along as they are (`Others`).
    pub others: Others<'a>,
}

/// The residents but the one whose update it is (each is stepped with the rest borrowed apart):
/// what another's own plans are worked out from, only when asked for (`hosts_about_till`; worked
/// out for everyone each tick, it was a cost of every step). Outside the residents' updates, all
/// of them (`World::ctx`).
#[derive(Clone, Copy, Default)]
pub(crate) struct Others<'a> {
    before: &'a [Sim],
    after: &'a [Sim],
}

impl<'a> Others<'a> {
    /// Every resident.
    pub(crate) fn all(sims: &'a [Sim]) -> Self {
        Others {
            before: sims,
            after: &[],
        }
    }

    /// Resident `j`; None for the one whose update it is (or one there isn't).
    pub(crate) fn get(&self, j: usize) -> Option<&'a Sim> {
        let n = self.before.len();
        match j.cmp(&n) {
            std::cmp::Ordering::Less => self.before.get(j),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => self.after.get(j - n - 1),
        }
    }
}

impl Ctx<'_> {
    /// Whether someone serves at object `object`: a sold use's minutes are then the ordering, the
    /// hand-over and the paying (`Interaction::minutes_where`).
    pub(crate) fn served(&self, object: u32) -> bool {
        self.staff.server_of(object).is_some()
    }

    /// Score of the room or garden at a point (`roomRules.away` off the plots and off the lot).
    pub(crate) fn room_at(&self, pos: [f32; 2]) -> f32 {
        let (x, z) = (pos[0].floor() as i32, pos[1].floor() as i32);
        let lot = self.nav.lot;
        if !lot.in_bounds(x, z) {
            return self.content.room_rules.away;
        }
        self.room_score
            .get(lot.tile_index(x, z))
            .copied()
            .unwrap_or(self.content.room_rules.away)
    }
}

#[derive(Debug, Clone, Copy)]
enum Choice {
    Object(u32, usize),
    /// A serving of food (an item).
    Eat(u32),
    /// Washing up, starting with this item.
    Dishes(u32),
    Clean(i32, i32),
    Repair(u32),
    Social(u32, usize),
    /// Joining the conversation this Sim started.
    Join(u32),
    Visit(u32),
    /// Open-world towns: off to a public place out of reach (`autonomy.reach`), a countryside
    /// spot, the park in the next hamlet; they go as visitors (`TaskKind::Visit`).
    Outing(u32),
    /// A phone action, with someone or not (`phone.rs`).
    Phone(usize, Option<u32>),
}

impl World {}

// ---- the world's helpers, split by what they do --------------------------------------------

mod colors;
mod commands;
mod decide;
mod navigation;
mod object;
mod objects;
mod outings;
mod placement;
mod residents;
mod rooms;
mod route;
mod sim;
mod state;
mod task;
mod tick;

pub use colors::*;
pub(crate) use decide::*;
pub use object::*;
pub(crate) use objects::*;
pub use outings::*;
pub(crate) use route::*;
pub use sim::*;
pub use task::*;
pub use tick::*;

#[cfg(test)]
mod tests;
