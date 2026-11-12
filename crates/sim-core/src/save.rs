//! Save games: a versioned, content-independent snapshot of the world.
//!
//! Objects, traits, perks, needs, interactions, feelings and careers are stored by id
//! (not index), so saves survive content changes; anything that no longer exists is
//! dropped on load. In-progress activities restart from the beginning on load.
//!
//! Version history: 1 = single household; 2 = town (households, plots), relationships,
//! feelings, gender and attraction; 3 = jobs, funds, visits, exits; 4 = skills, object
//! quality/style/value, household style; 5 = diagonal walls (`lot.diagonals`, absent in
//! older files, which load without any); 6 = wall looks (`lot.looks`: coverings, half walls,
//! door and window styles, by content id; absent: plain walls); 7 = floor coverings
//! (`lot.floors`, by content id; absent: automatic floors); 8 = free will per household,
//! the story log (`events`), job satisfaction and missed shifts, visits always end (absent:
//! one town-wide free-will switch, no story, a fresh start in every job); 9 = planners
//! (routines, goals, how blocks went, wishes) and household routine templates (absent: none);
//! 10 = the game mode (`mode`: Living or Creative; absent: Living), and the player's household
//! may have nobody living in it yet; 11 = objects turned freely (`turn`, absent: 0), fences
//! and gates (edges `4` and `5`, their fence style in `looks`), roofs (`plots[].roof`: roof
//! style and colour ids); 12 = per-tile dirt (`dirt`), the surroundings need, tidying up,
//! object wear and repairs; 13 = ages (`sims[].age`; absent: from content `life.startAge`)
//! and the lifespan (`lifespan`; absent: off, so older games don't start aging by surprise),
//! residents who are gone (`sims[].gone`) and former residents the story names (`former`);
//! 14 = a baby on the way (`households[].expecting`); 15 = story events name the content they
//! are about by id (`events[].refs`), so a content change can't rename them; 16 = story events
//! for visitors at the door (`turnedAway`, `nobodyHome`). Answering the door isn't saved: a
//! visitor knocking when the game was saved knocks again; 17 = storeys (`storeys`; the lot's rows
//! hold every storey, see `storeys.rs`; absent: one, and the lot grows to the content's count);
//! 18 = objects placed off the middle of their tile (`offset`, centimetres; absent: centred);
//! 19 = cooking makes food: food and dishes (`items`, by content id, and `nextItem`), what
//! residents carry (`sims[].held`) and recipes in progress (`Cook` tasks; eating and washing up
//! aren't saved, they start again from what's there; absent: none of it); 20 = objects
//! resized by the player (`size`, centimetres across and deep before turning, absent: their
//! kind's; `tall`, centimetres taller, absent: 0), and anything may stand off its tile's middle;
//! doors and windows too (`looks[].sill`, `looks[].head`, centimetres; absent: their style's);
//! 21 = story events from the phone (`invited`, `declined`); a phone action under way isn't
//! saved, nor is joining in on a conversation (`Join` tasks); 22 = basements (`basement`: the
//! block of the lot's rows holding storey -1, after the storeys above; absent: none, and the lot
//! grows one on the end if the content allows it, so nothing already built moves); 23 = the seed
//! the game started from (`seed`, for bug reports; absent: 0, unknown); 24 = objects have a colour
//! the player chose (`objects[].color`, `#rrggbb`; absent: as made) instead of a style (older
//! files' object and household `style` is ignored); 25 = objects can be open (`objects[].open`: a
//! wall bed folded down; absent: closed); 26 = a colour per part (`objects[].colors`, `#rrggbb` by
//! the model's material name; an older file's `color` goes on the `body`); 27 = crops grow
//! (`objects[].growth`: -1..0 fallow, 0..1 growing, 1 ripe; absent: ripe); 28 = lamps can be
//! switched off (`objects[].off`; absent: on); 29 = objects have a stable id (`objects[].id`)
//! that doesn't change when other objects are sold or moved (absent: assigned on load), and
//! `next_object_id` keeps ids from being reused after the highest one is sold; 30 = washing
//! piles up (`households[].laundry`: loads dirty and washed; absent: one of each); 31 = the
//! outfit a resident changed into (`sims[].outfit`; older files: 0, the creator's); 32 = the
//! day a pupil last went to school or missed it (`sims[].schoolDay`; older files: today once
//! school has begun, so a load never costs a missed day); 33 = machines run on their own
//! (`objects[].run`: game minutes left, 0 done with the load in it; absent: empty; in an older
//! file the washing out on a line is done, dry) and dried washing waits to be folded (a third
//! number in `households[].laundry`; absent: none), the dirty things in a dishwasher's rack waiting
//! for it to run (`objects[].rack`; absent: none); and the home chess table is 2 × 1 (it
//! was 1 × 1): an older file's goes over the tile it stood on where it fits, else at a free spot
//! at home, else it is sold back (`migrate_32_grown`); and an older file's posts in town (content
//! `staff`) with their back to a wall move a tile out from it, where there's room, for their server
//! (`World::nudge_post`); 34 = households' cars (`cars`: where each parks, its look, its
//! owners and where it is on its day out), every house's driveway (`drives`, where a car goes
//! when its household moves) and the garages on them (`garages`); absent: none, and the renderer draws the cars it used to draw,
//! parked for good. Older files load. Saves
//! written before feelings were renamed from "moodlets" store them under `moodlets`; a serde
//! alias still reads it.
//! 35 = open worlds (docs/design/open-world.md): the lot's edge and diagonal strings may be
//! run-length coded (a `~` and then `<char><count>` runs split by commas; dense strings still
//! load), chemistry is one string for every pair (`chemistry`: base64 of the upper triangle,
//! `(0, 1), (0, 2), …, (1, 2), …`, as little-endian f32s) and relationships list only the pairs
//! that aren't as new (no `chemistry` of their own), and the town file's open-world fields are
//! kept as they came (`openWorld`, `travel::OpenWorld`: size, roads, areas, terrain masks,
//! parking, plots' areas and kinds (and, written before the bus was dropped, which exits were bus
//! stops); driveways keep their `road` and `s` in `drives`); a car on a trip between hamlets keeps
//! it (`cars[].trip`, state `out` while it's parked there) and who rides with it
//! (`cars[].riders`; absent: nobody, as in files written before passengers, so no new version).
//! Absent: a town file without them, whose network is made from the meta's roads
//! (`travel::Travel::new`). Still 35: a garage the player built (Build mode's Garage tool)
//! keeps the way its door faces (`garages[].rot`, a thing's facing; absent: towards the street, as
//! the town placed it), and a car parked in front of one where it parked before
//! (`cars[].driveSpot`, `[x, z, yaw]`; absent: where it parks is its own spot). Both are left out
//! when absent, so a game without a garage of the player's saves as it did. Still 35: a driveway
//! running along x keeps the way it runs (`drives[].frontX`, ±1; absent: along z, `front`), so
//! files without one save as they did; a driveway's road, where it meets one (`drives[].road`), is
//! what has its cars drive the town's roads (`cars::settle_all`). Still 35: things
//! a resident asked for, bought and waiting to be set up at home (`setups`: the household, who
//! asked, the object by id, the tick it was bought and what was paid; `setup.rs`), left out when
//! there are none, so an older file has none; setting one up isn't saved mid-way (the parcel is
//! back at the front door after a load). Still 35: a file written while the town had buses (its
//! bus stops among its exits, `openWorld.stops`) loads without them: the stops are no ways out,
//! dropped from the exits as it loads, and nothing of the buses was ever saved. Still 35: the
//! traffic lights (`traffic::Signal`, worked out again on load) are saved where they'd got to (`lights`: each lit junction's phase, its
//! stage, the tick it began; `arm` before phases), left out where there are none; absent (an
//! older file, another number of lit junctions, or a phase one hasn't), they start over at their
//! first phase's green.
//! Still 35: a resident late for the shift they're at says so (`job.late`, left out when on
//! time; an older file's are on time).
//! Still 35: the story may tell of a host bringing a guest something (`served`, the item in `n`,
//! by id in `refs.item`); older builds drop it, as any kind they don't know. A host's offer, or
//! something on its way to a guest (`world::TaskKind::Serve`), isn't saved: what's in hand is put
//! down after a load.
//! 36 = ticks counted in this build's units, `TICKS_PER_SECOND` to a real second and
//! `TICKS_PER_MINUTE` to a game minute (60 and 75 from 0.8.0: a game day of 30 real minutes); a
//! file before it counted 20 and 20, and its ticks are rescaled as it loads (`Rescale`): the
//! clock's (`tick`, and the ticks things began, were made, happened or are due at) by the game
//! minute's, so it plays on at the same minute of the same day, and places along a car's or a
//! taxi's way (`cars[].progress`, its detour's and its way round a taxi's, `taxis.runs[].progress`)
//! by the real second's, as they're driven.
//! 37 = cars, the taxi, bikes and e-scooters go on the traffic's own clock, `DRIVE_SCALE` (4) times
//! as fast as real time (`DRIVE_TICKS_PER_SECOND` ticks to a second of it): a file before it timed
//! places along a car's or a taxi's way (`cars[].progress`, its detour's and its way round a
//! taxi's, `taxis.runs[].progress`) and the tick a traffic light's stage began (`lights`) in real
//! seconds, and they're rescaled as it loads (`Rescale::drive`): each place along its way is the
//! same place, as its way is now timed, and each light as many seconds into its stage.
//! 38 = the town's living-town fields (docs/design/living-town.md, World data) are in its `meta`:
//! `backdrop` (buildings nobody lives in), `places` (what the town tags name) and `centre` (each
//! main town's square and high street), as the town file had them; and its homes may be terrace
//! lots (`terrace`, narrow houses a metre apart). The simulation reads none of them: a file before
//! it has none and plays as before.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::content::{Content, Pose};
use crate::items::{Held, ItemAt, ItemInstance};
use crate::life::{Job, Visit};
use crate::lot::{DiagDir, Diagonal, Edge, EdgeLook, EdgeRef, Lot, SimSpawn};
use crate::planner::{BlockResult, Goal, HomeWish, Outcome, Planner, Reason, Routine};
use crate::rng::Rng;
use crate::social::{self, Relationship};
use crate::world::{GameMode, Household, Plot, Task, TaskKind, World};
use crate::{
    DRIVE_TICKS_PER_SECOND, Error, MINUTES_PER_TICK, TICKS_PER_MINUTE, TICKS_PER_SECOND,
    clock::MAX_SPEED,
};

pub const SAVE_VERSION: u32 = 38;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFile {
    pub version: u32,
    /// Living or Creative (absent before v10: Living).
    #[serde(default)]
    pub mode: GameMode,
    /// Dirty tiles: `[x, z, dirt × 255]` (absent before v12: clean).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dirt: Vec<[u16; 3]>,
    pub tick: u64,
    pub speed: u8,
    pub autonomy: bool,
    /// How fast residents age (absent before v13: off).
    #[serde(default)]
    pub lifespan: Option<crate::lifecycle::Lifespan>,
    /// Storeys the lot holds (its depth is that many times the town's; absent before v17: 1).
    #[serde(default)]
    pub storeys: Option<u8>,
    /// The block of rows holding the basement (its depth is then one more storey's; absent
    /// before v22, or without a basement: none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basement: Option<u8>,
    /// Whether the player's residents move on their own (absent: yes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_moves: Option<bool>,
    pub rng: u32,
    /// The seed the game started from (absent before v23: 0, unknown).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub seed: u32,
    pub lot: LotSave,
    pub objects: Vec<ObjectSave>,
    /// The next stable object id (absent before v29: the highest saved id plus one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_object_id: Option<u32>,
    pub sims: Vec<SimSave>,
    #[serde(default)]
    pub households: Vec<HouseholdSave>,
    #[serde(default)]
    pub plots: Vec<PlotSave>,
    #[serde(default)]
    pub relationships: Vec<RelationshipSave>,
    #[serde(default)]
    pub meta: serde_json::Value,
    #[serde(default)]
    pub exits: Vec<[f32; 2]>,
    /// The story so far, oldest first (absent before v8). Events name residents by index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<serde_json::Value>,
    #[serde(default)]
    pub next_event_id: u64,
    /// People whose slot someone new took (events name them with `social::FORMER`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub former: Vec<crate::world::Former>,
    /// Food and dishes (absent before v19: none).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemSave>,
    /// The next item's id (ids are never reused).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub next_item: u32,
    /// Households' cars, and every house's driveway (absent before v34: none).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cars: Vec<crate::cars::Car>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drives: Vec<crate::cars::Drive>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub garages: Vec<crate::cars::Garage>,
    /// Chemistry between every pair of residents (absent before v35: in `relationships`): base64
    /// of the upper triangle as little-endian f32s (`pack_chemistry`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub chemistry: String,
    /// The town file's open-world fields (`travel::OpenWorld`): roads, areas, terrain, parking,
    /// plots' areas and kinds (absent: none).
    #[serde(default, skip_serializing_if = "crate::travel::OpenWorld::is_empty")]
    pub open_world: crate::travel::OpenWorld,
    /// Things bought that wait to be set up at home (`setup.rs`; absent: none).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub setups: Vec<SetupSave>,
    /// The traffic lights where they'd got to (`traffic::Light`, a lit junction's each; absent:
    /// none, or they start over).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lights: Vec<LightSave>,
    /// The work taxis' runs planned and under way, and who came to work by one (`taxi::Taxis`;
    /// absent: none).
    #[serde(default, skip_serializing_if = "crate::taxi::Taxis::is_empty")]
    pub taxis: crate::taxi::Taxis,
}

/// A lit junction's lights (`traffic::Light`): whose turn it is (its phase, or its walk phase;
/// `arm` in files before phases, an arm's index: loaded as a phase if there's one, else they start
/// over), how far through it (0 green, 1 amber, 2 all red), and the tick that began.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LightSave {
    #[serde(alias = "arm")]
    pub phase: u8,
    pub stage: u8,
    pub since: u64,
}

/// Something bought waiting to be set up (`setup::Setup`), its object by content id (unknown ones
/// are dropped on load, the money given back).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupSave {
    pub household: u32,
    pub sim: u32,
    pub object: String,
    pub since: u64,
    pub paid: i64,
}

/// Food or a dish somewhere, by content id (unknown ones are dropped on load).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSave {
    pub id: u32,
    pub def: String,
    /// `surface`, `stored` or `floor` (see `items::ItemAt`).
    pub at: String,
    pub x: i32,
    pub z: i32,
    #[serde(default, skip_serializing_if = "crate::world::centred")]
    pub offset: [i8; 2],
    #[serde(default, skip_serializing_if = "is_zero")]
    pub servings: u8,
    pub quality: f32,
    pub made_at: u64,
    pub household: u32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub spoiled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
}

/// What a resident carries, by content id.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeldSave {
    pub def: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub servings: u8,
    pub quality: f32,
    pub made_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LotSave {
    pub width: usize,
    pub depth: usize,
    /// One character per edge: `0` open, `1` wall, `2` door, `3` window.
    pub h_edges: String,
    pub v_edges: String,
    /// One character per tile (row-major): `0` none; `/` diagonals `1` wall, `2` door,
    /// `3` window; `\` diagonals `4` wall, `5` door, `6` window. Empty when the lot has no
    /// diagonal walls (and in saves from before they existed).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub diagonals: String,
    /// Walls that don't look plain (absent in older saves).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub looks: Vec<LookSave>,
    /// Tiles with a floor covering (absent in older saves).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub floors: Vec<FloorSave>,
}

/// A tile's floor covering, by content id (unknown ids load as the automatic floor).
#[derive(Debug, Serialize, Deserialize)]
pub struct FloorSave {
    pub x: u16,
    pub z: u16,
    pub covering: String,
}

/// A wall's look, by content id so it survives content changes (unknown ids load as the
/// default look). `at`: `h`, `v` or `d` (the diagonal across tile `x, z`).
#[derive(Debug, Serialize, Deserialize)]
pub struct LookSave {
    pub at: String,
    pub x: u16,
    pub z: u16,
    /// Wall covering id of each face; empty for the automatic look.
    #[serde(default)]
    pub faces: [String; 2],
    #[serde(default, skip_serializing_if = "is_zero")]
    pub form: u8,
    /// Door or window style id (empty for none).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub style: String,
    /// Doors and windows the player resized: sill and top, centimetres (absent before v20, or the style's own).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sill: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<u16>,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

/// A number as a save can hold it: JSON has no NaN or infinity, and one (written as `null`)
/// would make the whole save unloadable, so a broken value is written as `or`.
fn finite(x: f32, or: f32) -> f32 {
    if x.is_finite() { x } else { or }
}

/// `Some(x)` if it can be saved (else left out, and loaded as the default).
fn some_finite(x: f32) -> Option<f32> {
    x.is_finite().then_some(x)
}

fn is_unworn(w: &f32) -> bool {
    *w <= 0.0
}

fn is_zero_i64(n: &i64) -> bool {
    *n == 0
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

pub(crate) fn is_zero_u64(n: &u64) -> bool {
    *n == 0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectSave {
    /// The object's stable id (absent before v29: assigned on load).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    pub def: String,
    pub x: i32,
    pub z: i32,
    pub rot: u8,
    #[serde(default)]
    pub quality: u8,
    /// The colours the player chose, `#rrggbb` by part (absent: as made).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub colors: BTreeMap<String, String>,
    /// Before v26: its one colour, on the `body`.
    #[serde(default, skip_serializing)]
    pub color: Option<String>,
    /// Money put into it; defaults to the price.
    #[serde(default)]
    pub value: Option<i64>,
    /// Degrees past the facing (objects that turn freely; absent before v11: 0).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub turn: u8,
    /// Centimetres off the middle of its footprint (objects that turn freely; absent before
    /// v18: centred).
    #[serde(default, skip_serializing_if = "crate::world::centred")]
    pub offset: [i8; 2],
    /// Wear, 0..1 (broken at 1; absent before v12: new).
    #[serde(default, skip_serializing_if = "is_unworn")]
    pub wear: f32,
    /// Its own size, centimetres across and deep before turning (absent before v20, or its kind's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<[u16; 2]>,
    /// Centimetres taller (negative: shorter) than its kind (absent before v20: 0).
    #[serde(default, skip_serializing_if = "is_level")]
    pub tall: i16,
    /// Open (a wall bed folded down; absent before v25, or closed).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub open: bool,
    /// A crop's growth (see `ObjectInstance::growth`; absent before v27, or not a crop: ripe).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub growth: Option<f32>,
    /// A lamp switched off (absent before v28, or on).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub off: bool,
    /// A machine's run (see `ObjectInstance::run`): game minutes left while it runs, 0 done
    /// with its load in it (absent before v33, or empty).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<f32>,
    /// The dirty things in a dishwasher's rack (see `ObjectInstance::rack`; absent before v33,
    /// or none).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rack: u8,
}

fn is_level(tall: &i16) -> bool {
    *tall == 0
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimSave {
    pub name: String,
    pub appearance: serde_json::Value,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub attracted_to: Option<Vec<String>>,
    #[serde(default)]
    pub household: u32,
    pub traits: Vec<String>,
    pub perks: Vec<String>,
    /// Age in years (absent before v13).
    #[serde(default)]
    pub age: Option<f32>,
    /// Died or moved away (the slot stays so ids don't change).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gone: Option<crate::world::Gone>,
    /// Retired, with this weekly pension.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub retired: bool,
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub pension: i64,
    /// How a pupil does at school (0..100).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grade: Option<f32>,
    /// The outfit they're in (`Sim::outfit`; absent before v31, or the creator's: 0).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub outfit: u8,
    /// The day a pupil last went to school or missed it (`Sim::school_day`; absent before v32).
    #[serde(default, rename = "schoolDay", skip_serializing_if = "Option::is_none")]
    pub school_day: Option<u32>,
    pub pos: [f32; 2],
    pub yaw: f32,
    /// Sorted so identical worlds produce identical save files.
    pub needs: BTreeMap<String, f32>,
    #[serde(default)]
    #[serde(alias = "moodlets")] // key before the moodlet → feeling rename
    pub feelings: Vec<FeelingSave>,
    /// Skill levels by id (sorted). Missing in old saves: traits decide.
    #[serde(default)]
    pub skills: BTreeMap<String, f32>,
    #[serde(default)]
    pub job: Option<JobSave>,
    /// First day to look for work again (after quitting or being let go).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub job_search_from: u32,
    /// Minutes until they'd nap again (absent: any time).
    #[serde(default, skip_serializing_if = "is_unworn")]
    pub nap_minutes: f32,
    /// Minutes left at work, if away.
    #[serde(default)]
    pub away_minutes: Option<f32>,
    /// Plot being visited, and minutes left (`None` in saves before v8: the player's visits
    /// had no end).
    #[serde(default)]
    pub visiting: Option<(u32, Option<f32>)>,
    /// Current activity first, then the queue.
    pub tasks: Vec<TaskSave>,
    /// Routines and goals (absent before v9). A block in progress starts over on load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planner: Option<PlannerSave>,
    /// What they carry (absent before v19: nothing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held: Option<HeldSave>,
    /// The tick a crying baby last woke them (a grown-up), or they last called someone (a
    /// baby), so turns at night and the baby's quiet spell go on after a reload (absent: never).
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub night_woken: u64,
}

/// A resident's planner, by content id.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlannerSave {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub routines: Vec<RoutineSave>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_household: Vec<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub goals: Vec<GoalSave>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<GoalSave>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<BlockResultSave>,
    /// `(activity, skill)` ids.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub wishes: Vec<(String, Option<String>)>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub home_wishes: Vec<HomeWishSave>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waited_day: Option<u32>,
    pub reviewed: u32,
    /// What they asked to buy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase: Option<PurchaseSave>,
}

/// Something a resident asked to buy (`object`: object id).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseSave {
    pub object: String,
    #[serde(default)]
    pub open: bool,
    pub day: u32,
    #[serde(default)]
    pub declines: u8,
}

/// A wish about the home: a better room (`room`: room kind id, "garden", or absent for a room
/// nothing marks; `factor`: "size", "light", ...), a fix (`fix`: object id) or another room of
/// a kind (`another`: room kind id).
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeWishSave {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub another: Option<String>,
}

pub(crate) fn home_wish_save(content: &Content, w: &HomeWish) -> HomeWishSave {
    let kind = |k: usize| content.room_kinds[k].id.clone();
    match *w {
        HomeWish::Room {
            kind: k,
            garden,
            factor,
        } => HomeWishSave {
            room: if garden {
                Some("garden".into())
            } else {
                k.map(kind)
            },
            factor: Some(crate::rooms::FACTORS[factor as usize].into()),
            ..Default::default()
        },
        HomeWish::Fix { def } => HomeWishSave {
            fix: Some(content.objects[def].id.clone()),
            ..Default::default()
        },
        HomeWish::Another { kind: k } => HomeWishSave {
            another: Some(kind(k)),
            ..Default::default()
        },
    }
}

fn home_wish_load(content: &Content, w: &HomeWishSave) -> Option<HomeWish> {
    let kind = |id: &str| content.room_kinds.iter().position(|k| k.id == id);
    if let Some(f) = &w.fix {
        return Some(HomeWish::Fix {
            def: content.object_index(f)?,
        });
    }
    if let Some(a) = &w.another {
        return Some(HomeWish::Another { kind: kind(a)? });
    }
    let factor = crate::rooms::FACTORS
        .iter()
        .position(|f| Some(*f) == w.factor.as_deref())? as u8;
    let garden = w.room.as_deref() == Some("garden");
    let k = match w.room.as_deref() {
        Some(id) if !garden => Some(kind(id)?),
        _ => None,
    };
    Some(HomeWish::Room {
        kind: k,
        garden,
        factor,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutineSave {
    pub id: u16,
    pub activity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    pub days: u8,
    pub start: u16,
    pub minutes: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalSave {
    pub def: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Object id, for things they want.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    pub target: f32,
    pub since_day: u32,
    pub start: f32,
    pub progress: f32,
    pub history: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockResultSave {
    pub routine: u16,
    pub household: bool,
    pub activity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    pub day: u32,
    pub start: u16,
    pub minutes: u16,
    /// `kept`, `cut`, `skipped`, `noPlace`.
    pub outcome: String,
    /// `trait:<id>`, `need:<id>`, `mood`, `away`, `noPlace`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub done: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSave {
    pub career: String,
    pub level: usize,
    pub performance: f32,
    pub last_shift_day: u32,
    #[serde(default)]
    pub satisfaction: Option<f32>,
    #[serde(default)]
    pub shifts: u32,
    #[serde(default)]
    pub missed: u8,
    /// Mood when leaving for the current shift (absent before v8).
    #[serde(default)]
    pub shift_mood: Option<f32>,
    /// Late for the current shift (`Job::late`; absent: on time).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub late: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeelingSave {
    pub id: String,
    pub minutes_left: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TaskSave {
    Use {
        object: u32,
        interaction: String,
        directed: bool,
        /// A routine's step (washing hands after the toilet): it takes its full time.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        routine: bool,
    },
    MoveTo {
        x: f32,
        z: f32,
        directed: bool,
    },
    Social {
        target: u32,
        social: String,
        directed: bool,
    },
    Work,
    Visit {
        plot: u32,
        directed: bool,
    },
    GoHome {
        directed: bool,
    },
    Clean {
        x: i32,
        z: i32,
        directed: bool,
    },
    Repair {
        object: u32,
        directed: bool,
    },
    Spot {
        accident: String,
    },
    /// Cooking a recipe (by id), at step `step` (from 0), started from object `origin`;
    /// `servings` paid for (0: not yet), `skipped` steps done without their station (v19).
    Cook {
        recipe: String,
        step: u8,
        #[serde(default)]
        origin: Option<u32>,
        #[serde(default)]
        servings: u8,
        #[serde(default)]
        skipped: u8,
        directed: bool,
    },
    /// Old saves only (upgrades are instant now); dropped on load.
    Upgrade {
        #[allow(dead_code)]
        object: u32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HouseholdSave {
    pub name: String,
    pub plot: Option<u32>,
    pub player: bool,
    #[serde(default)]
    pub funds: Option<i64>,
    /// Absent before v8, when free will was one switch for the whole town (`autonomy`).
    #[serde(default, rename = "freeWill", skip_serializing_if = "Option::is_none")]
    pub free_will: Option<bool>,
    /// Routine template for every member (absent before v9).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routines: Vec<RoutineSave>,
    /// A baby on the way (absent before v14: none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expecting: Option<crate::world::Expecting>,
    /// The day a babysitter was booked for (absent: none).
    #[serde(default, rename = "sitterDay", skip_serializing_if = "Option::is_none")]
    pub sitter_day: Option<u32>,
    /// Loads of washing waiting: dirty, washed to dry, and dried to fold (see
    /// `Household::laundry`; absent before v30: one dirty and one washed; before v33 without
    /// the dried).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub laundry: Option<Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlotSave {
    pub name: String,
    pub x: i32,
    pub z: i32,
    pub w: i32,
    pub d: i32,
    pub public: bool,
    #[serde(default)]
    pub entry: Option<[f32; 2]>,
    /// The roof the player chose: roof style and colour ids (absent before v11: the town's look).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roof: Option<[String; 2]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelationshipSave {
    pub a: u32,
    pub b: u32,
    #[serde(flatten)]
    pub rel: Relationship,
    /// Before v35 (since: in `SaveFile::chemistry`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chemistry: Option<f32>,
}

impl World {
    pub fn to_save(&self) -> SaveFile {
        let content = &self.content;
        let (h, v, diagonals) = self.lot.row_major_edges();
        let minutes_left = |until: u64| until.saturating_sub(self.tick) as f32 * MINUTES_PER_TICK;
        let objects = self
            .objects
            .iter()
            .map(|o| ObjectSave {
                id: Some(o.stable),
                def: content.objects[o.def].id.clone(),
                x: o.x,
                z: o.z,
                rot: o.rot,
                quality: o.quality,
                colors: crate::world::colors_hex(&o.colors),
                color: None,
                value: Some(o.value),
                turn: o.turn,
                offset: o.offset,
                wear: finite(o.wear, 0.0),
                size: (o.extent != crate::world::kind_size(content.objects[o.def].footprint))
                    .then_some(o.extent),
                tall: o.tall,
                open: o.open,
                growth: content.objects[o.def].grows.is_some().then_some(o.growth),
                off: o.off && content.objects[o.def].switch,
                run: o.run.filter(|_| content.objects[o.def].runs.is_some()),
                rack: o.rack.min(content.objects[o.def].rack),
            })
            .collect();
        let sims = self
            .sims
            .iter()
            .map(|s| {
                let mut pos = s.pos;
                // A Sim on a bed or chair (or in the shower) stands back up where they came from.
                if (s.pose != Pose::Stand || s.seated_from.is_some())
                    && let Some(act) = s.current()
                    && let TaskKind::Use { object, .. }
                    | TaskKind::Eat {
                        seat: Some(object), ..
                    } = act.task.kind
                {
                    let (fx, fz) = s
                        .seated_from
                        .unwrap_or_else(|| self.approach_tile(&self.objects[object as usize]));
                    pos = [fx as f32 + 0.5, fz as f32 + 0.5];
                } else if let Some((fx, fz)) = s.seated_from {
                    // (Talking from their seat: up where they sat down from, as what they sat
                    // down to do isn't saved.)
                    pos = [fx as f32 + 0.5, fz as f32 + 0.5];
                }
                // (Lost somewhere: back at home's front door, or the middle of town.)
                if !pos.iter().all(|p| p.is_finite()) {
                    let home = self
                        .households
                        .get(s.household as usize)
                        .and_then(|h| h.plot);
                    pos = home.map_or(
                        [self.lot.width as f32 / 2.0, self.lot.depth as f32 / 2.0],
                        |p| self.plots[p as usize].arrival_point(),
                    );
                }
                let tasks = s
                    .current()
                    .map(|a| &a.task)
                    .into_iter()
                    .chain(s.queue())
                    .filter_map(|t| self.task_save(t))
                    .collect();
                let feelings = s
                    .feelings
                    .iter()
                    .map(|m| FeelingSave {
                        id: content.feelings[m.def].id.clone(),
                        minutes_left: minutes_left(m.expires),
                    })
                    .collect();
                SimSave {
                    name: s.name.clone(),
                    appearance: s.appearance.clone(),
                    gender: Some(s.gender.clone()),
                    attracted_to: Some(s.attracted_to.clone()),
                    household: s.household,
                    traits: s.traits.clone(),
                    perks: s.perks.clone(),
                    age: some_finite(s.age),
                    gone: s.gone,
                    retired: s.retired,
                    pension: s.pension,
                    grade: content
                        .life
                        .stage_at(s.age)
                        .is_some_and(|st| st.school)
                        .then_some(s.grade)
                        .and_then(some_finite),
                    outfit: s.outfit,
                    school_day: content
                        .life
                        .stage_at(s.age)
                        .is_some_and(|st| st.school)
                        .then_some(s.school_day),
                    pos,
                    yaw: finite(s.yaw, 0.0),
                    needs: content
                        .needs
                        .iter()
                        .enumerate()
                        .map(|(i, n)| (n.id.clone(), finite(s.needs[i], 0.75)))
                        .collect(),
                    feelings,
                    skills: content
                        .skills
                        .iter()
                        .enumerate()
                        .map(|(i, k)| (k.id.clone(), finite(s.skills[i], 0.0)))
                        .collect(),
                    job: s.job.as_ref().map(|j| JobSave {
                        career: content.careers[j.career].id.clone(),
                        level: j.level,
                        performance: finite(j.performance, 0.0),
                        last_shift_day: j.last_shift_day,
                        satisfaction: some_finite(j.satisfaction),
                        shifts: j.shifts,
                        missed: j.missed,
                        shift_mood: some_finite(j.shift_mood),
                        late: j.late,
                    }),
                    job_search_from: s.job_search_from,
                    nap_minutes: minutes_left(s.nap_until),
                    away_minutes: s.away_until.map(minutes_left),
                    visiting: s.visiting.map(|v| (v.plot, Some(minutes_left(v.until)))),
                    tasks,
                    planner: planner_save(content, &s.planner),
                    held: s.held.map(|h| HeldSave {
                        def: content.items[h.def].id.clone(),
                        servings: h.servings,
                        quality: finite(h.quality, 1.0),
                        made_at: h.made_at,
                        recipe: h.recipe.map(|r| content.recipes[r].id.clone()),
                    }),
                    night_woken: s.night_woken,
                }
            })
            .collect();
        let n = self.sims.len();
        let mut relationships = Vec::new();
        for a in 0..n {
            for b in 0..n {
                let rel = *self.relationships.get(a, b);
                if a != b && rel != Relationship::default() {
                    let rel = Relationship {
                        friendship: finite(rel.friendship, 0.0),
                        romance: finite(rel.romance, 0.0),
                        ..rel
                    };
                    relationships.push(RelationshipSave {
                        a: a as u32,
                        b: b as u32,
                        rel,
                        chemistry: None,
                    });
                }
            }
        }
        let chemistry = pack_chemistry(n, |a, b| finite(self.relationships.chemistry(a, b), 0.0));
        SaveFile {
            version: SAVE_VERSION,
            mode: self.mode,
            dirt: self
                .lot
                .tiles()
                .filter_map(|(x, z, i)| {
                    let d = self.dirt.get(i).copied().unwrap_or(0.0);
                    (d >= 1.0 / 255.0).then(|| [x as u16, z as u16, (d * 255.0).round() as u16])
                })
                .collect(),
            tick: self.tick,
            speed: self.speed,
            autonomy: self.autonomy,
            lifespan: Some(self.lifespan),
            storeys: Some(self.storeys),
            basement: (self.stairs.basement > 0).then_some(self.stairs.basement as u8),
            player_moves: (!self.player_moves).then_some(false),
            rng: self.rng.state(),
            seed: self.seed,
            lot: LotSave {
                width: self.lot.width,
                depth: self.lot.depth,
                h_edges: run_length(&encode(&h)),
                v_edges: run_length(&encode(&v)),
                diagonals: run_length(&encode_diagonals(&diagonals)),
                looks: encode_looks(&self.lot, &self.content),
                floors: encode_floors(&self.lot, &self.content),
            },
            objects,
            next_object_id: Some(self.next_object_id),
            sims,
            households: self
                .households
                .iter()
                .map(|h| HouseholdSave {
                    name: h.name.clone(),
                    plot: h.plot,
                    player: h.player,
                    funds: Some(h.funds),
                    free_will: Some(h.free_will),
                    routines: h
                        .routines
                        .iter()
                        .map(|r| routine_save(content, r))
                        .collect(),
                    expecting: h.expecting,
                    sitter_day: h.sitter_day,
                    // (With a load taken up for a use under way: the use starts again on load.)
                    laundry: Some(
                        crate::machines::with_taken(
                            h.laundry,
                            self.sims
                                .iter()
                                .filter(|s| s.household == h.id)
                                .map(|s| s.taken),
                            content.laundry.map_or(1.0, |l| l.most),
                        )
                        .to_vec(),
                    ),
                })
                .collect(),
            plots: self
                .plots
                .iter()
                .map(|p| PlotSave {
                    name: p.name.clone(),
                    x: p.x,
                    z: p.z,
                    w: p.w,
                    d: p.d,
                    public: p.public,
                    entry: p.entry.filter(|e| e.iter().all(|v| v.is_finite())),
                    roof: p.roof.map(|r| {
                        let b = &content.build;
                        let id = |list: &[crate::content::BuildStyle], i: u8| {
                            list.get(i as usize).map_or(String::new(), |s| s.id.clone())
                        };
                        [id(&b.roofs, r.style), id(&b.roof_colors, r.color)]
                    }),
                })
                .collect(),
            relationships,
            chemistry,
            meta: self.meta.clone(),
            exits: self
                .exits
                .iter()
                .copied()
                .filter(|e| e.iter().all(|v| v.is_finite()))
                .collect(),
            events: self
                .events
                .iter()
                .map(|e| {
                    let mut v = serde_json::to_value(e).expect("event serializes");
                    let refs = event_refs(content, e);
                    if !refs.is_empty() {
                        v["refs"] = serde_json::Value::Object(refs);
                    }
                    v
                })
                .collect(),
            next_event_id: self.events.last_id(),
            former: self.former.clone(),
            // (The households' cars: the work taxis' slots are the runs', `taxis`.)
            cars: self.cars.iter().filter(|c| !c.taxi).cloned().collect(),
            taxis: self.taxis.clone(),
            drives: self.drives.clone(),
            garages: self.garages.clone(),
            open_world: self.travel.file.clone(),
            items: self
                .items
                .list
                .iter()
                .map(|it| {
                    let (at, offset) = match it.at {
                        ItemAt::Surface { offset, .. } => ("surface", offset),
                        ItemAt::Stored { .. } => ("stored", [0, 0]),
                        ItemAt::Floor { offset, .. } => ("floor", offset),
                    };
                    let (x, z) = it.at.tile();
                    ItemSave {
                        id: it.id,
                        def: content.items[it.def].id.clone(),
                        at: at.to_owned(),
                        x,
                        z,
                        offset,
                        servings: it.servings,
                        quality: finite(it.quality, 1.0),
                        made_at: it.made_at,
                        household: it.household,
                        spoiled: it.spoiled,
                        recipe: it.recipe.map(|r| content.recipes[r].id.clone()),
                    }
                })
                .collect(),
            next_item: self.items.next_id,
            setups: self
                .setups
                .iter()
                .map(|s| SetupSave {
                    household: s.household,
                    sim: s.sim,
                    object: content.objects[s.object].id.clone(),
                    since: s.since,
                    paid: s.paid,
                })
                .collect(),
            lights: self
                .traffic
                .lights
                .iter()
                .map(|l| LightSave {
                    phase: l.phase,
                    stage: match l.stage {
                        crate::traffic::Stage::Green => 0,
                        crate::traffic::Stage::Amber => 1,
                        crate::traffic::Stage::Clear => 2,
                    },
                    since: l.since,
                })
                .collect(),
        }
    }

    pub fn save_json(&self) -> String {
        serde_json::to_string(&self.to_save()).expect("save serializes")
    }

    pub fn from_save_json(content_json: &str, save_json: &str) -> Result<Self, Error> {
        let content = Content::from_json(content_json)?;
        let save: SaveFile = serde_json::from_str(save_json)?;
        Self::from_save(content, &save)
    }

    pub fn from_save(content: Content, save: &SaveFile) -> Result<Self, Error> {
        if save.version == 0 || save.version > SAVE_VERSION {
            return Err(Error::new(format!(
                "unsupported save version {}",
                save.version
            )));
        }
        // Storeys before the furniture: what may stand upstairs depends on them.
        let storeys = save.storeys.unwrap_or(1).max(1);
        // The basement's block comes after the ground's, at most one past the storeys above.
        let basement = save.basement.filter(|&b| b > 0 && b <= storeys);
        let blocks = storeys as usize + usize::from(basement.is_some());
        let stacked = blocks > 1 && save.lot.depth.is_multiple_of(blocks);
        let ground = if stacked {
            save.lot.depth / blocks
        } else {
            save.lot.depth
        };
        let mut lot = Lot::from_edges(
            save.lot.width,
            save.lot.depth,
            ground,
            decode(&expand(&save.lot.h_edges)?)?,
            decode(&expand(&save.lot.v_edges)?)?,
            decode_diagonals(&expand(&save.lot.diagonals)?)?,
        )?;
        decode_looks(&mut lot, &content, &save.lot.looks);
        decode_floors(&mut lot, &content, &save.lot.floors);
        let mut world = World::empty(content, lot, Rng::new(save.rng));
        if stacked {
            world.storeys = storeys;
            world.stairs.depth = (world.lot.depth / blocks) as i32;
            world.stairs.basement = basement.map_or(0, i32::from);
        }
        world.mode = save.mode;
        world.seed = save.seed;
        // (Before v36: ticks counted at 20 a real second and 20 a game minute; before v37, cars
        // timed in real seconds.)
        let ticks = Rescale::of(save.version);
        world.tick = ticks.tick(save.tick);
        world.speed = save.speed.min(MAX_SPEED);
        // Before v8 the one free-will switch was the player's setting; now each household has one.
        let legacy_free_will = save.version < 8;
        world.autonomy = save.autonomy || legacy_free_will;
        world.lifespan = save.lifespan.unwrap_or(crate::lifecycle::Lifespan::Off);
        world.player_moves = save.player_moves.unwrap_or(true);
        world.meta = save.meta.clone();
        world.note_paths();
        // (Bus stops, in a file written while the town had buses, are no ways out.)
        world.exits = save
            .exits
            .iter()
            .enumerate()
            .filter(|&(i, _)| !save.open_world.stops.contains(&(i as u32)))
            .map(|(_, e)| *e)
            .collect();
        world.set_travel(save.open_world.clone());
        world.plots = save
            .plots
            .iter()
            .enumerate()
            .map(|(i, p)| Plot {
                id: i as u32,
                name: p.name.clone(),
                x: p.x,
                z: p.z,
                w: p.w,
                d: p.d,
                public: p.public,
                entry: p.entry,
                roof: p.roof.as_ref().and_then(|[style, color]| {
                    let b = &world.content.build;
                    let index = |list: &[crate::content::BuildStyle], id: &str| {
                        list.iter().position(|s| s.id == id).map(|i| i as u8)
                    };
                    // A style or colour the content no longer has falls back to the first.
                    let style = index(&b.roofs, style).unwrap_or(0);
                    let color = index(&b.roof_colors, color).unwrap_or(0);
                    (!b.roofs.is_empty()).then_some(crate::world::RoofLook { style, color })
                }),
            })
            .collect();
        let starting_funds = world.content.starting_funds;
        world.households = if save.households.is_empty() {
            vec![Household {
                id: 0,
                name: "Household".into(),
                plot: None,
                player: true,
                funds: starting_funds,
                free_will: save.autonomy,
                routines: Vec::new(),
                expecting: None,
                sitter_day: None,
                laundry: crate::world::LAUNDRY_START,
            }]
        } else {
            save.households
                .iter()
                .enumerate()
                .map(|(i, h)| Household {
                    id: i as u32,
                    name: h.name.clone(),
                    plot: h.plot,
                    player: h.player,
                    funds: h.funds.unwrap_or(starting_funds),
                    free_will: h
                        .free_will
                        .unwrap_or(!h.player || save.autonomy || !legacy_free_will),
                    routines: h
                        .routines
                        .iter()
                        .filter_map(|r| routine_load(&world.content, r))
                        .collect(),
                    expecting: h.expecting,
                    sitter_day: h.sitter_day,
                    laundry: h.laundry.as_ref().map_or(crate::world::LAUNDRY_START, |l| {
                        std::array::from_fn(|k| {
                            finite(l.get(k).copied().unwrap_or(0.0), 0.0).max(0.0)
                        })
                    }),
                })
                .collect()
        };
        if let Some(h) = world
            .households
            .iter()
            .find(|h| h.plot.is_some_and(|p| p as usize >= world.plots.len()))
        {
            return Err(Error::new(format!(
                "corrupt save: household '{}' lives on an unknown plot",
                h.name
            )));
        }

        // Object ids may change from the save's (an object left out, or tops placed last): tasks are remapped.
        // Things standing on tops go in once everything else stands.
        let mut object_ids = HashMap::new();
        // Saved stable ids are kept (once each); old saves and anything else get fresh ones, past
        // every saved id.
        world.next_object_id = save
            .objects
            .iter()
            .filter_map(|o| o.id)
            .max()
            .map_or(0, |m| m + 1);
        let mut used = HashSet::new();
        let on_top = |o: &ObjectSave| {
            world
                .content
                .object_index(&o.def)
                .is_some_and(|d| world.content.objects[d].layer == crate::content::Layer::Surface)
        };
        let tops: Vec<bool> = save.objects.iter().map(on_top).collect();
        // (v33: kinds that grew go in once everything else stands, where they now fit: `migrate_32_grown`.)
        let grown: Vec<bool> = save
            .objects
            .iter()
            .map(|o| grown_since_32(save.version, o))
            .collect();
        // (v33: the town's posts go in after the rest, so one with its back to a wall can move out
        // from it where there's room: `migrate_32_post`.)
        let posts: Vec<bool> = (0..tops.len())
            .map(|i| !tops[i] && !grown[i] && post_since_32(&world.content, save, i, &tops))
            .collect();
        let order: Vec<usize> = (0..tops.len())
            .filter(|&i| !tops[i] && !grown[i] && !posts[i])
            .chain((0..tops.len()).filter(|&i| posts[i]))
            .chain((0..tops.len()).filter(|&i| grown[i]))
            .chain((0..tops.len()).filter(|&i| tops[i]))
            .collect();
        for i in order {
            let o = &save.objects[i];
            // (A corrupt save's second use of an id gets a fresh one.)
            let keep = o.id.filter(|s| !used.contains(s));
            let placed = match world.content.object_index(&o.def) {
                Some(def) if grown[i] => migrate_32_grown(&mut world, def, o, keep),
                Some(def) => {
                    let value = o
                        .value
                        .unwrap_or(world.content.objects[def].price.unwrap_or(0));
                    let colors = crate::world::parse_colors(&o.colors, o.color.as_deref());
                    // A size the content no longer allows goes back to its kind's.
                    let own = crate::world::kind_size(world.content.objects[def].footprint);
                    let (size, tall) = world
                        .check_size(def, o.size.unwrap_or(own), o.tall)
                        .unwrap_or((own, 0));
                    let layout = crate::world::Layout {
                        offset: o.offset,
                        extent: Some(size),
                        tall,
                    };
                    world.place(def, o.x, o.z, o.rot, layout, colors, o.quality, value, keep)
                }
                None => world.place_object(&o.def, o.x, o.z, o.rot),
            };
            // What can't be placed is left out, and the rest of the game loads: a kind the content
            // no longer has, a thing whose top is gone, or one that doesn't fit where it was any
            // more (building can leave a ceiling lamp outdoors, or something upstairs outside a room).
            if let Ok(id) = placed {
                let obj = &world.objects[id as usize];
                let turn = world.check_turn(obj.def, o.turn).unwrap_or(0);
                let offset = world
                    .check_offset(obj.def, obj.rot, o.offset)
                    .unwrap_or([0, 0]);
                let obj = &mut world.objects[id as usize];
                obj.turn = turn;
                obj.offset = offset;
                obj.wear = o.wear.clamp(0.0, 1.0);
                obj.open = o.open;
                obj.off = o.off && world.content.objects[obj.def].switch;
                obj.growth = finite(o.growth.unwrap_or(1.0), 1.0).clamp(-1.0, 1.0);
                let runs = world.content.objects[obj.def].runs;
                obj.run = match o.run {
                    Some(m) => runs.map(|full| finite(m, 0.0).clamp(0.0, full)),
                    // Before v33 nothing ran on its own: washing left out on the line (open) is
                    // dry by now, ready to take in.
                    None if save.version < 33 && obj.open => runs.map(|_| 0.0),
                    None => None,
                };
                obj.rack = o.rack.min(world.content.objects[obj.def].rack);
                let id = if posts[i] { world.nudge_post(id) } else { id };
                used.extend(keep);
                object_ids.insert(i as u32, id);
            }
        }
        world.next_object_id = world.next_object_id.max(save.next_object_id.unwrap_or(0));
        for s in &save.sims {
            let spawn = SimSpawn {
                name: s.name.clone(),
                gender: s.gender.clone(),
                attracted_to: s.attracted_to.clone(),
                household: s.household,
                job: None,
                appearance: s.appearance.clone(),
                traits: s.traits.clone(),
                perks: s.perks.clone(),
                age: s.age.filter(|a| a.is_finite()),
                skills: Default::default(),
                x: s.pos[0],
                z: s.pos[1],
            };
            world.spawn_sim(&spawn)?;
        }
        let n = world.sims.len();
        if let Some(h) = world.households.iter().find(|h| {
            h.expecting
                .is_some_and(|e| e.parents.iter().any(|&p| p as usize >= n))
        }) {
            return Err(Error::new(format!(
                "corrupt save: household '{}' expects a baby by someone who isn't in town",
                h.name
            )));
        }
        let plot_count = world.plots.len();
        let tick = world.tick;
        let after =
            |minutes: f32| tick.saturating_add(social::minutes_to_ticks(finite(minutes, 0.0)));
        for (id, s) in save.sims.iter().enumerate() {
            let content = &world.content;
            let sim = &mut world.sims[id];
            sim.yaw = finite(s.yaw, 0.0);
            sim.gone = s.gone;
            sim.retired = s.retired;
            sim.pension = s.pension.max(0);
            if let Some(g) = s.grade {
                sim.grade = g.clamp(0.0, 100.0);
            }
            // (v31: residents change outfits; before it everyone wore the creator's.)
            sim.outfit = if save.version >= 31 {
                s.outfit % crate::world::OUTFITS
            } else {
                0
            };
            // (v32: the school day is kept; before it, once today's school has begun it counts as
            // seen to, or the load would mark the day missed.)
            sim.school_day = match s.school_day {
                Some(day) if save.version >= 32 => day,
                _ => match crate::life::school_today(content, sim, tick) {
                    Some((_, start, _)) if tick > start => crate::clock::day(tick),
                    _ => 0,
                },
            };
            for (i, need) in content.needs.iter().enumerate() {
                sim.needs[i] = s
                    .needs
                    .get(&need.id)
                    .copied()
                    .unwrap_or(0.75)
                    .clamp(0.0, 1.0);
            }
            for m in &s.feelings {
                if let Some(def) = content.feeling_index(&m.id) {
                    let expires = after(m.minutes_left.min(content.feelings[def].minutes));
                    sim.feelings.push(social::ActiveFeeling { def, expires });
                }
            }
            for (i, skill) in content.skills.iter().enumerate() {
                if let Some(&v) = s.skills.get(&skill.id) {
                    sim.skills[i] = v.clamp(0.0, content.skill_rules.max_level);
                }
            }
            sim.job = s.job.as_ref().and_then(|j| {
                let career = content.career_index(&j.career)?;
                let level = j.level.min(content.careers[career].levels.len() - 1);
                let mut job = Job::new(career, level, j.last_shift_day);
                job.performance = j.performance;
                job.satisfaction = j.satisfaction.unwrap_or(job.satisfaction);
                job.shifts = j.shifts;
                job.missed = j.missed;
                job.shift_mood = j.shift_mood.unwrap_or(job.shift_mood);
                job.late = j.late;
                Some(job)
            });
            sim.job_search_from = s.job_search_from;
            if s.nap_minutes > 0.0 {
                sim.nap_until = after(s.nap_minutes);
            }
            if let Some(p) = &s.planner {
                sim.planner = planner_load(content, p);
            }
            sim.held = s.held.as_ref().and_then(|h| {
                Some(Held {
                    def: content.item_index(&h.def)?,
                    servings: h.servings,
                    quality: h.quality,
                    made_at: ticks.tick(h.made_at),
                    recipe: h.recipe.as_deref().and_then(|r| content.recipe_index(r)),
                })
            });
            sim.night_woken = ticks.tick(s.night_woken);
            // At work, or a pupil at school.
            let pupil = content
                .life
                .stage_at(s.age.unwrap_or(30.0))
                .is_some_and(|st| st.school);
            sim.away_until = s
                .away_minutes
                .filter(|_| sim.job.is_some() || pupil)
                .map(after);
            sim.visiting = s.visiting.and_then(|(plot, left)| {
                ((plot as usize) < plot_count).then(|| Visit {
                    plot,
                    // Visits without an end (player visits before v8) last as long as a
                    // player visit does now.
                    until: after(left.unwrap_or(content.visits.directed_hours * 60.0)),
                    idle: 0,
                    farewell: 0,
                    offer_at: 0,
                    arrived: 0,
                    leaving: false,
                    home_by: false,
                })
            });
            for t in &s.tasks {
                let task = match t {
                    TaskSave::Use {
                        object,
                        interaction,
                        directed,
                        routine,
                    } => {
                        let Some(&object) = object_ids.get(object) else {
                            continue;
                        };
                        let def = &content.objects[world.objects[object as usize].def];
                        let Some(interaction) =
                            def.interactions.iter().position(|i| &i.id == interaction)
                        else {
                            continue;
                        };
                        Task {
                            kind: TaskKind::Use {
                                object,
                                interaction,
                            },
                            directed: *directed,
                            routine: *routine,
                        }
                    }
                    TaskSave::MoveTo { x, z, directed } => Task {
                        kind: TaskKind::MoveTo { x: *x, z: *z },
                        directed: *directed,
                        routine: false,
                    },
                    TaskSave::Social {
                        target,
                        social,
                        directed,
                    } => {
                        let Some(social) = content.social_index(social) else {
                            continue;
                        };
                        if *target as usize >= n || *target as usize == id {
                            continue;
                        }
                        Task {
                            kind: TaskKind::Social {
                                target: *target,
                                social,
                            },
                            directed: *directed,
                            routine: false,
                        }
                    }
                    TaskSave::Work => Task {
                        kind: TaskKind::Work,
                        directed: true,
                        routine: false,
                    },
                    TaskSave::Visit { plot, directed } => {
                        if *plot as usize >= plot_count {
                            continue;
                        }
                        Task {
                            kind: TaskKind::Visit { plot: *plot },
                            directed: *directed,
                            routine: false,
                        }
                    }
                    TaskSave::GoHome { directed } => Task {
                        kind: TaskKind::GoHome,
                        directed: *directed,
                        routine: false,
                    },
                    TaskSave::Clean { x, z, directed } => Task {
                        kind: TaskKind::Clean { x: *x, z: *z },
                        directed: *directed,
                        routine: false,
                    },
                    // (An accident's while on the spot doesn't outlast a reload.)
                    TaskSave::Spot { .. } => continue,
                    TaskSave::Repair { object, directed } => {
                        let Some(&object) = object_ids.get(object) else {
                            continue;
                        };
                        Task {
                            kind: TaskKind::Repair { object },
                            directed: *directed,
                            routine: false,
                        }
                    }
                    // Upgrades used to be a Sim task; they are instant purchases now.
                    TaskSave::Upgrade { .. } => continue,
                    TaskSave::Cook {
                        recipe,
                        step,
                        origin,
                        servings,
                        skipped,
                        directed,
                    } => {
                        let Some(recipe) = content.recipe_index(recipe) else {
                            continue;
                        };
                        // (At most one past the last step: the dish on its way to the table.)
                        if *step as usize > content.recipes[recipe].steps.len() {
                            continue;
                        }
                        Task {
                            kind: TaskKind::Cook {
                                recipe,
                                step: *step,
                                station: None,
                                origin: origin.and_then(|o| object_ids.get(&o).copied()),
                                servings: *servings,
                                skipped: *skipped,
                                minutes: 0,
                            },
                            directed: *directed,
                            routine: false,
                        }
                    }
                };
                sim.queue.push_back(task);
            }
        }

        // (Since v35 a save whose pairs are all as new lists none, but keeps their chemistry.)
        if save.version < 2 || (save.relationships.is_empty() && save.chemistry.is_empty()) {
            // Older saves have no relationships: start the household as housemates.
            world.init_relationships(&[])?;
        } else {
            for r in &save.relationships {
                let (a, b) = (r.a as usize, r.b as usize);
                if a < n && b < n && a != b {
                    let rel = Relationship {
                        friendship: r.rel.friendship.clamp(-100.0, 100.0),
                        romance: r.rel.romance.clamp(-100.0, 100.0),
                        ..r.rel
                    };
                    world.relationships.set(a, b, rel);
                    if let Some(c) = r.chemistry {
                        world.relationships.set_chemistry(a, b, c);
                    }
                }
            }
        }
        if !save.chemistry.is_empty() {
            let chemistry = unpack_chemistry(&save.chemistry, n)?;
            let mut k = 0;
            for a in 0..n {
                for b in a + 1..n {
                    world
                        .relationships
                        .set_chemistry(a, b, finite(chemistry[k], 0.0));
                    k += 1;
                }
            }
        }
        // Buffs from saved feelings apply straight away.
        for sim in &mut world.sims {
            sim.refresh_buffs(&world.content);
        }
        // Spawning consumed random numbers; restore the saved stream.
        world.rng = Rng::new(save.rng);
        // The story: events about residents who no longer exist (or of kinds this version
        // doesn't know) are dropped.
        let content = &world.content;
        let events = save.events.iter().filter_map(|v| {
            let mut e: social::SocialEvent = serde_json::from_value(v.clone()).ok()?;
            e.tick = ticks.tick(e.tick);
            match v.get("refs") {
                Some(refs) => resolve_refs(content, &mut e, refs)?,
                None => migrate_refs(content, &mut e, save.version),
            }
            let formers = save.former.len();
            let known = |s: u32| {
                if s & social::FORMER != 0 {
                    ((s & !social::FORMER) as usize) < formers
                } else {
                    (s as usize) < n
                }
            };
            (known(e.a) && known(e.b) && e.c.is_none_or(known)).then_some(e)
        });
        world.events = social::EventLog::restore(save.next_event_id, events);
        world.former = save.former.clone();
        world.dirt = vec![0.0; world.lot.tile_slots()];
        for &[x, z, d] in &save.dirt {
            let (x, z) = (x as i32, z as i32);
            if world.lot.in_bounds(x, z) {
                let i = world.tile_slot_mut(x, z);
                world.dirt[i] = (f32::from(d) / 255.0).min(1.0);
            }
        }
        let households = world.households.len();
        let content = &world.content;
        world.items.list = save
            .items
            .iter()
            .filter_map(|it| {
                let at = match it.at.as_str() {
                    "surface" => ItemAt::Surface {
                        x: it.x,
                        z: it.z,
                        offset: it.offset,
                    },
                    "stored" => ItemAt::Stored { x: it.x, z: it.z },
                    "floor" => ItemAt::Floor {
                        x: it.x,
                        z: it.z,
                        offset: it.offset,
                    },
                    _ => return None,
                };
                ((it.household as usize) < households).then_some(())?;
                Some(ItemInstance {
                    id: it.id,
                    def: content.item_index(&it.def)?,
                    at,
                    servings: it.servings,
                    quality: it.quality,
                    made_at: ticks.tick(it.made_at),
                    household: it.household,
                    spoiled: it.spoiled,
                    recipe: it.recipe.as_deref().and_then(|r| content.recipe_index(r)),
                })
            })
            .collect();
        world.items.list.sort_by_key(|it| it.id);
        world.items.list.dedup_by_key(|it| it.id);
        let after_last = match world.items.list.last() {
            Some(it) => it
                .id
                .checked_add(1)
                .ok_or_else(|| Error::new("corrupt save: an item id is out of range"))?,
            None => 0,
        };
        world.items.next_id = save.next_item.max(after_last);
        for s in &save.setups {
            let object = world.content.object_index(&s.object);
            let Some(h) = world.households.get_mut(s.household as usize) else {
                continue;
            };
            match object {
                Some(object) if (s.sim as usize) < world.sims.len() => {
                    world.setups.push(crate::setup::Setup::new(
                        s.household,
                        s.sim,
                        object,
                        ticks.tick(s.since),
                        s.paid,
                    ))
                }
                // (Gone from the content: the money back.)
                _ => h.funds += s.paid,
            }
        }
        world.make_storeys();
        world.refresh_rooms();
        let cars = ticks.cars(&save.cars);
        crate::cars::load(&mut world, &cars, &save.drives, &save.garages);
        // (The junctions lit are worked out again from the roads: `traffic::settle`.)
        crate::traffic::settle(&mut world);
        // (The lights where they'd got to, for the same lit junctions: each a turn it has.)
        if save.lights.len() == world.travel.signals.len()
            && save
                .lights
                .iter()
                .zip(&world.travel.signals)
                .all(|(l, s)| l.phase < s.turns())
        {
            world.traffic.lights = save
                .lights
                .iter()
                .map(|l| crate::traffic::Light {
                    phase: l.phase,
                    stage: match l.stage {
                        1 => crate::traffic::Stage::Amber,
                        2 => crate::traffic::Stage::Clear,
                        _ => crate::traffic::Stage::Green,
                    },
                    since: ticks.drive_since(l.since, save.tick),
                })
                .collect();
        }
        // The work taxis' runs, their riders booked and the taxis under way where they were.
        world.taxis = save.taxis.clone();
        ticks.taxis(&mut world.taxis);
        crate::taxi::settle(&mut world);
        // What someone was eating or washing up is put down (those don't go on after loading).
        crate::cooking::settle(&mut world);
        Ok(world)
    }

    /// `None` for tasks that aren't saved: answering the door (the visitor knocks again after loading).
    fn task_save(&self, t: &Task) -> Option<TaskSave> {
        let content = &self.content;
        Some(match t.kind {
            TaskKind::Use {
                object,
                interaction,
            } => TaskSave::Use {
                object,
                interaction: content.objects[self.objects[object as usize].def].interactions
                    [interaction]
                    .id
                    .clone(),
                directed: t.directed,
                routine: t.routine,
            },
            // (A walk to nowhere isn't saved.)
            TaskKind::MoveTo { x, z } if !(x.is_finite() && z.is_finite()) => return None,
            TaskKind::MoveTo { x, z } => TaskSave::MoveTo {
                x,
                z,
                directed: t.directed,
            },
            TaskKind::Social { target, social } => TaskSave::Social {
                target,
                social: content.socials[social].id.clone(),
                directed: t.directed,
            },
            TaskKind::Work => TaskSave::Work,
            TaskKind::Visit { plot } => TaskSave::Visit {
                plot,
                directed: t.directed,
            },
            TaskKind::GoHome => TaskSave::GoHome {
                directed: t.directed,
            },
            TaskKind::Clean { x, z } => TaskSave::Clean {
                x,
                z,
                directed: t.directed,
            },
            TaskKind::Repair { object } => TaskSave::Repair {
                object,
                directed: t.directed,
            },
            TaskKind::Spot { accident } => TaskSave::Spot {
                accident: content.accidents[accident].id.clone(),
            },
            TaskKind::Cook {
                recipe,
                step,
                origin,
                servings,
                skipped,
                ..
            } => TaskSave::Cook {
                recipe: content.recipes[recipe].id.clone(),
                step,
                origin,
                servings,
                skipped,
                directed: t.directed,
            },
            // (Eating and washing up start again from what's there after loading; a few minutes
            // on the phone are simply over, and so is listening in on a conversation.)
            // (Nor is a carry: a baby is back in the crib after a load, a harvest put down; nor
            // a host bringing a guest something, which is put down.)
            TaskKind::Answer { .. }
            | TaskKind::Eat { .. }
            | TaskKind::Dishes { .. }
            | TaskKind::Phone { .. }
            | TaskKind::Join { .. }
            | TaskKind::Carry { .. }
            | TaskKind::Serve { .. } => return None,
            // (Nor is setting something up: the parcel is back at the door, `setup.rs`.)
            TaskKind::SetUp { .. } => return None,
        })
    }
}

/// Runs of a character longer than this are given a count (`run_length`).
const RUN: usize = 1;
/// Strings of edges or diagonals this long or shorter stay as they are (`run_length`).
const DENSE: usize = 4096;

/// A string of edge or diagonal characters, run-length coded (v35) when it is long and that makes
/// it shorter: `~` and then `<char><count>` runs split by commas (`~0120,1,04` is 120 `0`s, a `1`
/// and four `0`s). Short ones (a single lot's) stay dense, as before.
fn run_length(dense: &str) -> String {
    if dense.len() <= DENSE {
        return dense.to_string();
    }
    let mut out = String::from("~");
    let bytes = dense.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let mut j = i + 1;
        while j < bytes.len() && bytes[j] == c {
            j += 1;
        }
        if i > 0 {
            out.push(',');
        }
        out.push(char::from(c));
        if j - i > RUN {
            out.push_str(&(j - i).to_string());
        }
        i = j;
    }
    if out.len() < dense.len() {
        out
    } else {
        dense.to_string()
    }
}

/// The dense string a saved one stands for: run-length coded (`~`, v35) or as it is (older).
fn expand(s: &str) -> Result<std::borrow::Cow<'_, str>, Error> {
    let Some(runs) = s.strip_prefix('~') else {
        return Ok(std::borrow::Cow::Borrowed(s));
    };
    let corrupt = || Error::new("corrupt lot edges in save");
    let mut out = String::new();
    for run in runs.split(',').filter(|r| !r.is_empty()) {
        let mut chars = run.chars();
        let c = chars.next().ok_or_else(corrupt)?;
        let count: usize = match chars.as_str() {
            "" => 1,
            n => n.parse().map_err(|_| corrupt())?,
        };
        // (No lot is bigger than this: a corrupt count can't ask for more.)
        if out.len().saturating_add(count) > (crate::lot::MAX_SIDE + 1) * (crate::lot::MAX_ROWS + 1)
        {
            return Err(corrupt());
        }
        out.extend(std::iter::repeat_n(c, count));
    }
    Ok(std::borrow::Cow::Owned(out))
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = u32::from(b[0]) << 16 | u32::from(b[1]) << 8 | u32::from(b[2]);
        for k in 0..4 {
            if k <= chunk.len() {
                out.push(char::from(BASE64[(n >> (18 - 6 * k) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn unbase64(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let digit = |c: u8| BASE64.iter().position(|&d| d == c).map(|v| v as u32);
    for chunk in s.as_bytes().chunks(4) {
        if chunk.len() != 4 {
            return None;
        }
        let pad = chunk.iter().rev().take_while(|&&c| c == b'=').count();
        let mut n = 0u32;
        for &c in &chunk[..4 - pad] {
            n = n << 6 | digit(c)?;
        }
        n <<= 6 * pad as u32;
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    Some(out)
}

/// Chemistry between every pair `a < b` of `n` residents, row by row, as base64 of little-endian
/// f32s (empty for fewer than two).
fn pack_chemistry(n: usize, chemistry: impl Fn(usize, usize) -> f32) -> String {
    let mut bytes = Vec::with_capacity(n * n.saturating_sub(1) * 2);
    for a in 0..n {
        for b in a + 1..n {
            bytes.extend_from_slice(&chemistry(a, b).to_le_bytes());
        }
    }
    base64(&bytes)
}

fn unpack_chemistry(s: &str, n: usize) -> Result<Vec<f32>, Error> {
    let corrupt = || Error::new("corrupt chemistry in save");
    let bytes = unbase64(s).ok_or_else(corrupt)?;
    if bytes.len() != n * n.saturating_sub(1) / 2 * 4 {
        return Err(corrupt());
    }
    Ok(bytes
        .chunks(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}

fn encode(edges: &[Edge]) -> String {
    edges
        .iter()
        .map(|e| match e {
            Edge::Open => '0',
            Edge::Wall => '1',
            Edge::Door => '2',
            Edge::Window => '3',
            Edge::Fence => '4',
            Edge::Gate => '5',
        })
        .collect()
}

/// The kind of wall standing at `at` (open where there's none).
fn edge_at(lot: &Lot, at: EdgeRef) -> Edge {
    match at {
        EdgeRef::H(x, z) => lot.h_edge(x as usize, z as usize),
        EdgeRef::V(x, z) => lot.v_edge(x as usize, z as usize),
        EdgeRef::Diag(x, z) => lot.diag(x as i32, z as i32).map_or(Edge::Open, |d| d.edge),
    }
}

fn encode_looks(lot: &Lot, content: &Content) -> Vec<LookSave> {
    let b = &content.build;
    let covering = |c: u8| {
        c.checked_sub(1)
            .and_then(|i| b.coverings.get(i as usize))
            .map_or(String::new(), |s| s.id.clone())
    };
    lot.looks()
        .filter_map(|(at, look)| {
            let edge = edge_at(lot, at);
            if edge == Edge::Open {
                return None;
            }
            let styles = match edge {
                Edge::Door => &b.doors,
                Edge::Window => &b.windows,
                Edge::Fence | Edge::Gate => &b.fences,
                _ => &Vec::new(),
            };
            let (tag, x, z) = match at {
                EdgeRef::H(x, z) => ("h", x, z),
                EdgeRef::V(x, z) => ("v", x, z),
                EdgeRef::Diag(x, z) => ("d", x, z),
            };
            Some(LookSave {
                at: tag.into(),
                x,
                z,
                faces: look.sides.map(covering),
                form: look.form,
                style: styles
                    .get(look.style as usize)
                    .map_or(String::new(), |s| s.id.clone()),
                sill: look.sill,
                head: look.head,
            })
        })
        .collect()
}

/// Puts saved looks back on the walls they belong to; unknown ids, and looks of walls that
/// aren't there, are dropped.
fn decode_looks(lot: &mut Lot, content: &Content, looks: &[LookSave]) {
    let b = &content.build;
    let index =
        |list: &[crate::content::BuildStyle], id: &str| list.iter().position(|s| s.id == id);
    for l in looks {
        let at = match l.at.as_str() {
            "h" if (l.x as usize) < lot.width && (l.z as usize) <= lot.depth => {
                EdgeRef::H(l.x, l.z)
            }
            "v" if (l.x as usize) <= lot.width && (l.z as usize) < lot.depth => {
                EdgeRef::V(l.x, l.z)
            }
            "d" if lot.in_bounds(l.x as i32, l.z as i32) => EdgeRef::Diag(l.x, l.z),
            _ => continue,
        };
        let edge = edge_at(lot, at);
        if edge == Edge::Open {
            continue;
        }
        let styles: &[crate::content::BuildStyle] = match edge {
            Edge::Door => &b.doors,
            Edge::Window => &b.windows,
            Edge::Fence | Edge::Gate => &b.fences,
            _ => &[],
        };
        let side = |id: &str| index(&b.coverings, id).map_or(0, |i| i as u8 + 1);
        lot.set_look(
            at,
            EdgeLook {
                sides: [side(&l.faces[0]), side(&l.faces[1])],
                form: if edge == Edge::Wall {
                    l.form.min(crate::lot::FORM_HALF)
                } else {
                    0
                },
                style: index(styles, &l.style).map_or(0, |i| i as u8),
                // A size no door or window may have is dropped.
                sill: l.sill.filter(|&s| {
                    edge == Edge::Window
                        && s + crate::command::MIN_WINDOW <= crate::command::MAX_HEAD
                }),
                head: l.head.filter(|&h| {
                    matches!(edge, Edge::Door | Edge::Window) && h <= crate::command::MAX_HEAD
                }),
            },
        );
    }
}

fn encode_floors(lot: &Lot, content: &Content) -> Vec<FloorSave> {
    let floors = &content.build.floors;
    lot.floors()
        .filter_map(|(x, z, c)| {
            let id = &floors.get(c.checked_sub(1)? as usize)?.id;
            Some(FloorSave {
                x,
                z,
                covering: id.clone(),
            })
        })
        .collect()
}

/// Puts saved floor coverings back; unknown ids, and tiles off the lot, are dropped.
fn decode_floors(lot: &mut Lot, content: &Content, floors: &[FloorSave]) {
    let list = &content.build.floors;
    for f in floors {
        if !lot.in_bounds(i32::from(f.x), i32::from(f.z)) {
            continue;
        }
        if let Some(i) = list.iter().position(|s| s.id == f.covering) {
            lot.set_floor(f.x, f.z, i as u8 + 1);
        }
    }
}

fn encode_diagonals(diags: &[Option<Diagonal>]) -> String {
    if diags.iter().all(Option::is_none) {
        return String::new();
    }
    diags
        .iter()
        .map(|d| {
            let Some(d) = d else { return '0' };
            let base = match d.dir {
                DiagDir::Dp => 0,
                DiagDir::Dn => 3,
            };
            let kind = match d.edge {
                Edge::Door => 2,
                Edge::Window => 3,
                _ => 1,
            };
            char::from(b'0' + base + kind)
        })
        .collect()
}

fn decode_diagonals(s: &str) -> Result<Vec<Option<Diagonal>>, Error> {
    s.chars()
        .map(|c| {
            let n = c
                .to_digit(10)
                .filter(|&n| n <= 6)
                .ok_or_else(|| Error::new("corrupt diagonal walls in save"))?;
            if n == 0 {
                return Ok(None);
            }
            let dir = if n <= 3 { DiagDir::Dp } else { DiagDir::Dn };
            let edge = match (n - 1) % 3 {
                0 => Edge::Wall,
                1 => Edge::Door,
                _ => Edge::Window,
            };
            Ok(Some(Diagonal { dir, edge }))
        })
        .collect()
}

fn decode(s: &str) -> Result<Vec<Edge>, Error> {
    s.chars()
        .map(|c| match c {
            '0' => Ok(Edge::Open),
            '1' => Ok(Edge::Wall),
            '2' => Ok(Edge::Door),
            '3' => Ok(Edge::Window),
            '4' => Ok(Edge::Fence),
            '5' => Ok(Edge::Gate),
            _ => Err(Error::new("corrupt lot edges in save")),
        })
        .collect()
}

// ---- Planner ------------------------------------------------------------------------------

fn skill_id(content: &Content, skill: Option<usize>) -> Option<String> {
    skill.map(|s| content.skills[s].id.clone())
}

fn routine_save(content: &Content, r: &Routine) -> RoutineSave {
    RoutineSave {
        id: r.id,
        activity: content.activities[r.activity].id.clone(),
        skill: skill_id(content, r.skill),
        days: r.days,
        start: r.start,
        minutes: r.minutes,
    }
}

/// A saved routine, if its activity (and skill) still exist.
fn routine_load(content: &Content, r: &RoutineSave) -> Option<Routine> {
    let activity = content.activities.iter().position(|a| a.id == r.activity)?;
    Some(Routine {
        id: r.id,
        activity,
        skill: r.skill.as_deref().and_then(|s| content.skill_index(s)),
        days: r.days & 0x7f,
        start: r.start.min(1439),
        minutes: r.minutes,
    })
}

fn goal_save(content: &Content, g: &Goal) -> GoalSave {
    GoalSave {
        def: content.goals[g.def].id.clone(),
        skill: skill_id(content, g.skill),
        category: g.category.map(|c| content.career_categories[c].id.clone()),
        object: g.object.map(|o| content.objects[o].id.clone()),
        target: finite(g.target, 0.0),
        since_day: g.since_day,
        start: finite(g.start, 0.0),
        progress: finite(g.progress, 0.0),
        history: g.history.map(|h| finite(h, 0.0)),
    }
}

fn goal_load(content: &Content, g: &GoalSave) -> Option<Goal> {
    Some(Goal {
        def: content.goals.iter().position(|d| d.id == g.def)?,
        skill: match &g.skill {
            Some(s) => Some(content.skill_index(s)?),
            None => None,
        },
        category: g
            .category
            .as_deref()
            .and_then(|c| content.career_categories.iter().position(|d| d.id == c)),
        object: match &g.object {
            Some(o) => Some(content.object_index(o)?),
            None => None,
        },
        target: g.target,
        since_day: g.since_day,
        start: g.start,
        progress: g.progress,
        history: g.history,
    })
}

fn reason_save(content: &Content, r: Reason) -> String {
    match r {
        Reason::Trait(t) => format!("trait:{}", content.traits[t].id),
        Reason::Need(n) => format!("need:{}", content.needs[n].id),
        Reason::Mood => "mood".into(),
        Reason::Away => "away".into(),
        Reason::NoPlace => "noPlace".into(),
    }
}

fn reason_load(content: &Content, r: &str) -> Option<Reason> {
    Some(match r.split_once(':') {
        Some(("trait", id)) => Reason::Trait(content.traits.iter().position(|t| t.id == id)?),
        Some(("need", id)) => Reason::Need(content.needs.iter().position(|n| n.id == id)?),
        _ => match r {
            "away" => Reason::Away,
            "noPlace" => Reason::NoPlace,
            _ => Reason::Mood,
        },
    })
}

fn planner_save(content: &Content, p: &Planner) -> Option<PlannerSave> {
    let save = PlannerSave {
        routines: p
            .routines
            .iter()
            .map(|r| routine_save(content, r))
            .collect(),
        skip_household: p.skip_household.clone(),
        goals: p.goals.iter().map(|g| goal_save(content, g)).collect(),
        suggestions: p
            .suggestions
            .iter()
            .map(|g| goal_save(content, g))
            .collect(),
        history: p
            .history
            .iter()
            .map(|b| BlockResultSave {
                routine: b.routine,
                household: b.household,
                activity: content.activities[b.activity].id.clone(),
                skill: skill_id(content, b.skill),
                day: b.day,
                start: b.start,
                minutes: b.minutes,
                outcome: match b.outcome {
                    Outcome::Kept => "kept",
                    Outcome::Cut => "cut",
                    Outcome::Skipped => "skipped",
                    Outcome::NoPlace => "noPlace",
                }
                .into(),
                reason: b.reason.map(|r| reason_save(content, r)),
                done: finite(b.done, 0.0),
            })
            .collect(),
        wishes: p
            .wishes
            .iter()
            .map(|&(a, s)| (content.activities[a].id.clone(), skill_id(content, s)))
            .collect(),
        home_wishes: p
            .home_wishes
            .iter()
            .map(|w| home_wish_save(content, w))
            .collect(),
        waited_day: p.waited_day,
        reviewed: p.reviewed,
        purchase: p.purchase.map(|r| PurchaseSave {
            object: content.objects[r.object].id.clone(),
            open: r.open,
            day: r.day,
            declines: r.declines,
        }),
    };
    let empty = save.routines.is_empty()
        && save.skip_household.is_empty()
        && save.goals.is_empty()
        && save.suggestions.is_empty()
        && save.history.is_empty()
        && save.wishes.is_empty()
        && save.home_wishes.is_empty()
        && save.waited_day.is_none()
        && save.purchase.is_none();
    (!empty || save.reviewed != 0).then_some(save)
}

fn planner_load(content: &Content, p: &PlannerSave) -> Planner {
    let activity = |id: &str| content.activities.iter().position(|a| a.id == id);
    Planner {
        routines: p
            .routines
            .iter()
            .filter_map(|r| routine_load(content, r))
            .collect(),
        skip_household: p.skip_household.clone(),
        // Older saves may hold the same goal twice: keep the first.
        goals: p.goals.iter().filter_map(|g| goal_load(content, g)).fold(
            Vec::new(),
            |mut goals: Vec<Goal>, g| {
                if !goals.iter().any(|o| o.same(&g)) {
                    goals.push(g);
                }
                goals
            },
        ),
        suggestions: p
            .suggestions
            .iter()
            .filter_map(|g| goal_load(content, g))
            .collect(),
        run: None,
        history: p
            .history
            .iter()
            .filter_map(|b| {
                Some(BlockResult {
                    routine: b.routine,
                    household: b.household,
                    activity: activity(&b.activity)?,
                    skill: b.skill.as_deref().and_then(|s| content.skill_index(s)),
                    day: b.day,
                    start: b.start,
                    minutes: b.minutes,
                    outcome: match b.outcome.as_str() {
                        "kept" => Outcome::Kept,
                        "cut" => Outcome::Cut,
                        "noPlace" => Outcome::NoPlace,
                        _ => Outcome::Skipped,
                    },
                    reason: b.reason.as_deref().and_then(|r| reason_load(content, r)),
                    done: b.done,
                })
            })
            .collect(),
        wishes: p
            .wishes
            .iter()
            .filter_map(|(a, s)| {
                Some((
                    activity(a)?,
                    s.as_deref().and_then(|s| content.skill_index(s)),
                ))
            })
            .collect(),
        home_wishes: p
            .home_wishes
            .iter()
            .filter_map(|w| home_wish_load(content, w))
            .collect(),
        waited_day: p.waited_day,
        thought: None,
        reviewed: p.reviewed,
        recheck: false,
        purchase: p.purchase.as_ref().and_then(|r| {
            Some(crate::planner::PurchaseRequest {
                object: content.object_index(&r.object)?,
                open: r.open,
                day: r.day,
                declines: r.declines,
            })
        }),
    }
}

/// The content a story event names, by id (its indices change when content does): the object,
/// accident or life stage in `n`, the skill, the career (or career category, for goals) and the
/// goal.
fn event_refs(
    content: &Content,
    e: &social::SocialEvent,
) -> serde_json::Map<String, serde_json::Value> {
    use social::EventKind::*;
    let mut refs = serde_json::Map::new();
    let mut put = |key: &str, id: Option<&String>| {
        if let Some(id) = id {
            refs.insert(key.into(), id.clone().into());
        }
    };
    let n = e.n.and_then(|n| usize::try_from(n).ok());
    match e.kind {
        Broke | Repaired => put(
            "object",
            n.and_then(|n| content.objects.get(n)).map(|d| &d.id),
        ),
        Accident => put(
            "accident",
            n.and_then(|n| content.accidents.get(n)).map(|d| &d.id),
        ),
        GrewOlder => put(
            "stage",
            n.and_then(|n| content.life.stages.get(n)).map(|d| &d.id),
        ),
        Served => put("item", n.and_then(|n| content.items.get(n)).map(|d| &d.id)),
        _ => {}
    }
    put(
        "skill",
        e.skill
            .and_then(|s| content.skills.get(s as usize))
            .map(|d| &d.id),
    );
    if matches!(e.kind, GoalReached | GoalSuggested) {
        put(
            "category",
            e.career
                .and_then(|c| content.career_categories.get(c as usize))
                .map(|d| &d.id),
        );
    } else {
        put(
            "career",
            e.career
                .and_then(|c| content.careers.get(c as usize))
                .map(|d| &d.id),
        );
    }
    put(
        "goal",
        e.goal
            .and_then(|g| content.goals.get(g as usize))
            .map(|d| &d.id),
    );
    refs
}

/// Points a loaded event at the content it names (by the ids saved with it). None when the
/// content no longer has one of them: the event is dropped.
fn resolve_refs(
    content: &Content,
    e: &mut social::SocialEvent,
    refs: &serde_json::Value,
) -> Option<()> {
    /// The index of the saved id for `key` among `ids`: Some(None) without one, None if gone.
    fn find<'a>(
        refs: &serde_json::Value,
        key: &str,
        mut ids: impl Iterator<Item = &'a String>,
    ) -> Option<Option<u32>> {
        match refs.get(key).and_then(|v| v.as_str()) {
            None => Some(None),
            Some(want) => ids.position(|i| i == want).map(|p| Some(p as u32)),
        }
    }
    let n = [
        find(refs, "object", content.objects.iter().map(|d| &d.id))?,
        find(refs, "accident", content.accidents.iter().map(|d| &d.id))?,
        find(refs, "stage", content.life.stages.iter().map(|d| &d.id))?,
        find(refs, "item", content.items.iter().map(|d| &d.id))?,
    ];
    if let Some(i) = n.into_iter().flatten().next() {
        e.n = Some(i64::from(i));
    }
    if let Some(s) = find(refs, "skill", content.skills.iter().map(|d| &d.id))? {
        e.skill = Some(s);
    }
    if let Some(c) = find(refs, "career", content.careers.iter().map(|d| &d.id))? {
        e.career = Some(c);
    }
    if let Some(c) = find(
        refs,
        "category",
        content.career_categories.iter().map(|d| &d.id),
    )? {
        e.career = Some(c);
    }
    if let Some(g) = find(refs, "goal", content.goals.iter().map(|d| &d.id))? {
        e.goal = Some(g);
    }
    Some(())
}

/// Ticks as files before v36 counted them: 20 to a real second and 20 to a game minute.
const OLD_TICKS_PER_SECOND: u32 = 20;
const OLD_TICKS_PER_MINUTE: u64 = 20;

/// How a file's ticks become this build's (v36: before it, `OLD_TICKS_PER_SECOND` and
/// `OLD_TICKS_PER_MINUTE`; v37: before it, cars timed in real seconds). The clock's ticks
/// (`World::tick`, and the ticks things began, were made, happened or are due at) by the game
/// minute's ticks, now over then: tick 0 is 08:00 on day 1 either way, so each is the same minute
/// of the same day. Places along a car's or a taxi's way (ticks of its route, timed by its speeds)
/// by a second of drive time's ticks (`DRIVE_TICKS_PER_SECOND`) over the file's (its real second's
/// before v37: the same place along the same way). The tick a traffic light's stage began
/// (`traffic::Light`, timed in drive time, `drive_secs`) as far into its stage as it was
/// (`drive_since`). A car's state and a taxi standing at its stop began at a tick of the clock:
/// what they wait for is game minutes (a car boarding `cars::BOARD_WAIT_TICKS`, a taxi
/// `taxi.waitMinutes` and `taxi::COMING_WAIT`), and their seconds' floors and dwells
/// (`BOARD_MIN_TICKS`, `DWELL`, `PASS_BY_MOST`) end within the second. Nothing else is saved in
/// ticks: what's left of a nap, a visit, a feeling or a machine's run is saved in game minutes,
/// and the day things happened on in days.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rescale {
    /// Ticks in a game minute, now and in the file.
    game: (u64, u64),
    /// Ticks in a second of drive time now over those in the file (its real second's before v37).
    drive: f32,
}

impl Rescale {
    /// For a file of save version `version`.
    fn of(version: u32) -> Self {
        // (Ticks to the second cars were timed in, in the file.)
        let file_drive = match version {
            ..36 => OLD_TICKS_PER_SECOND as f32,
            36 => TICKS_PER_SECOND as f32,
            _ => DRIVE_TICKS_PER_SECOND,
        };
        Self {
            game: if version < 36 {
                (TICKS_PER_MINUTE, OLD_TICKS_PER_MINUTE)
            } else {
                (1, 1)
            },
            drive: DRIVE_TICKS_PER_SECOND / file_drive,
        }
    }

    /// A tick of the clock, to the nearest (0, never, stays 0; the same tick stays the same).
    fn tick(self, t: u64) -> u64 {
        let (now, then) = self.game;
        if now == then {
            return t;
        }
        let scaled = (u128::from(t) * u128::from(now) + u128::from(then / 2)) / u128::from(then);
        u64::try_from(scaled).unwrap_or(u64::MAX)
    }

    /// A place along a way, in ticks of its route.
    fn route(self, t: f32) -> f32 {
        t * self.drive
    }

    /// The tick something timed in drive time began (`since`; in real time before v37), the
    /// file's clock at `now`: as many of its seconds before the clock's tick (`tick`) as it was.
    /// (By the clock's ticks, a traffic light green for 4 seconds was at it for 5.)
    fn drive_since(self, since: u64, now: u64) -> u64 {
        if since >= now {
            return self.tick(since);
        }
        let ago = ((now - since) as f64 * f64::from(self.drive)).round() as u64;
        self.tick(now).saturating_sub(ago)
    }

    fn detour(self, d: &mut crate::cars::Detour) {
        d.from = self.route(d.from);
        d.back = self.route(d.back);
    }

    /// The households' cars: when their state began and when one away sets off back, where they
    /// are along their way, and their way turned round or round a taxi.
    fn cars<'a>(self, cars: &'a [crate::cars::Car]) -> std::borrow::Cow<'a, [crate::cars::Car]> {
        if self.game.0 == self.game.1 && self.drive == 1.0 {
            return cars.into();
        }
        let mut cars = cars.to_vec();
        for c in &mut cars {
            c.since = self.tick(c.since);
            c.ready_at = self.tick(c.ready_at);
            c.progress = self.route(c.progress);
            if let Some(d) = c.detour.as_mut() {
                self.detour(d);
            }
            if let Some(p) = c.pass_by.as_mut() {
                p.from = self.route(p.from);
                p.out = self.route(p.out);
                p.back = self.route(p.back);
                p.to = self.route(p.to);
            }
        }
        cars.into()
    }

    /// The work taxis' runs: when each comes in, is due out (and was planned to be), leaves work
    /// and stands at its stop since, when each stop is due and its rider leaves home for it, and
    /// where it is along its way.
    fn taxis(self, taxis: &mut crate::taxi::Taxis) {
        for r in &mut taxis.runs {
            r.enter = self.tick(r.enter);
            r.due_out = self.tick(r.due_out);
            r.planned_out = self.tick(r.planned_out);
            r.depart = self.tick(r.depart);
            r.since = self.tick(r.since);
            r.progress = self.route(r.progress);
            if let Some(d) = r.detour.as_mut() {
                self.detour(d);
            }
            for st in &mut r.stops {
                st.due = self.tick(st.due);
                st.leave = self.tick(st.leave);
            }
        }
    }
}

/// Kinds whose footprint grew in save version 33: the home chess table, 1 × 1 to 2 × 1 (a chair
/// at each end, M-hobbies-11).
const GROWN_IN_33: [&str; 1] = ["chessTable"];

/// Whether a saved object is one of `GROWN_IN_33` from an older file, at its kind's old size (a
/// resized one keeps its own size and loads as any).
fn grown_since_32(version: u32, o: &ObjectSave) -> bool {
    version < 33 && o.size.is_none() && GROWN_IN_33.contains(&o.def.as_str())
}

/// Whether saved object `i` is one of the town's posts (content `staff`) from a file older than
/// v33, with nothing standing on it: placed after the rest, it may move out from a wall
/// (`World::nudge_post`).
fn post_since_32(content: &Content, save: &SaveFile, i: usize, tops: &[bool]) -> bool {
    let o = &save.objects[i];
    let Some(def) = content.object_index(&o.def) else {
        return false;
    };
    let d = &content.objects[def];
    if save.version >= 33 || d.staff.is_none() {
        return false;
    }
    // (Anything on its top stays where it was saved: so does the post.)
    let (w, dd) = if o.rot.is_multiple_of(2) {
        (d.footprint[0], d.footprint[1])
    } else {
        (d.footprint[1], d.footprint[0])
    };
    let under =
        |t: &ObjectSave| t.x >= o.x && t.x < o.x + w as i32 && t.z >= o.z && t.z < o.z + dd as i32;
    !save
        .objects
        .iter()
        .enumerate()
        .any(|(j, t)| tops[j] && under(t))
}

/// v32 to v33: a thing whose kind grew (`grown_since_32`) goes over the tile it stood on where it
/// now fits and blocks nothing, else at its home's first good free spot (`World::place_grown`);
/// with no room for it anywhere there, it is sold back to the household living on its plot for
/// what it was worth, and left out. Never placed overlapping anything, nor shutting anyone or
/// anything off.
fn migrate_32_grown(
    world: &mut World,
    def: usize,
    o: &ObjectSave,
    keep: Option<u32>,
) -> Result<u32, crate::Error> {
    let value = o
        .value
        .unwrap_or(world.content.objects[def].price.unwrap_or(0));
    let colors = crate::world::parse_colors(&o.colors, o.color.as_deref());
    if let Some(id) = world.place_grown(def, o.x, o.z, o.rot, colors, o.quality, value) {
        if let Some(stable) = keep {
            world.objects[id as usize].stable = stable;
        }
        return Ok(id);
    }
    let plot = world.plot_at(o.x, o.z);
    if let Some(h) = world
        .households
        .iter_mut()
        .find(|h| plot.is_some() && h.plot == plot)
    {
        h.funds += value;
    }
    Err(crate::Error::new(format!(
        "no room for the {} any more",
        world.content.objects[def].name.to_lowercase()
    )))
}

/// Events from before content ids were saved name content by index. The one change since: the
/// base game's crib (0.13, save 14) came at the end of the base objects, so objects from packs
/// (after it) moved up one.
fn migrate_refs(content: &Content, e: &mut social::SocialEvent, version: u32) {
    use social::EventKind::*;
    if version < 14
        && matches!(e.kind, Broke | Repaired)
        && let (Some(crib), Some(n)) = (content.object_index("crib"), e.n)
        && n >= crib as i64
    {
        e.n = Some(n + 1);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Command, World};

    const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1},{"id":"energy","label":"Energy","decayPerHour":0.05}],
        "objects":[
          {"id":"fridge","name":"Fridge","interactions":[{"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.4}}]},
          {"id":"bed","name":"Bed","footprint":[2,2],"interactions":[{"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0}}]}],
        "genders":[{"id":"female","label":"Female"},{"id":"male","label":"Male"}],
        "feelings":[{"id":"happy","label":"Happy","mood":0.1,"hours":3}],
        "bondPresets":{"friends":{"friendship":60}},
        "traits":[{"id":"lazy","label":"Lazy","effects":{"walkSpeed":0.8}}]}"#;

    const LOT: &str = r##"{"width":12,"depth":12,"walls":[[6,0,6,10]],"doors":[{"x":6,"z":4,"axis":"z"}],
        "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"bed","x":9,"z":2,"rot":3}],
        "households":[{"name":"Park","player":true},{"name":"Next door"}],
        "sims":[{"name":"Ada","gender":"female","attractedTo":["male"],"appearance":{"body":"#fff"},"traits":["lazy"],"x":3.5,"z":3.5},
                {"name":"Bo","gender":"male","household":1,"x":4.5,"z":6.5}],
        "relationships":[{"a":0,"b":1,"preset":"friends"}]}"##;

    #[test]
    fn a_run_too_long_for_any_lot_is_corrupt() {
        assert!(super::expand(&format!("~01,0{}", usize::MAX)).is_err());
        assert_eq!(&*super::expand("~03,1,02").unwrap(), "000100");
    }

    #[test]
    fn round_trip_preserves_state() {
        let mut w = World::from_json(CONTENT, LOT, 3).unwrap();
        w.apply(Command::Use {
            sim: 1,
            object: 0,
            interaction: 0,
        })
        .unwrap();
        w.apply(Command::SetAutonomy {
            enabled: false,
            household: None,
        })
        .unwrap();
        crate::social::add_feeling(&mut w.sims[0].feelings, 0, &w.content.feelings, w.tick);
        // (Seven game minutes less a few ticks: not on a whole minute.)
        for _ in 0..crate::mins(7.0) - 3 {
            w.tick_once();
        }
        let json = w.save_json();
        let loaded = World::from_save_json(CONTENT, &json).unwrap();

        assert_eq!(loaded.tick, w.tick);
        assert!(loaded.autonomy, "the town-wide switch stays on");
        assert!(
            !loaded.households[0].free_will,
            "the player's household has free will off"
        );
        assert!(loaded.households[1].free_will);
        assert_eq!(loaded.objects.len(), 2);
        assert_eq!(loaded.households.len(), 2);
        assert_eq!(loaded.sims[1].household, 1);
        assert_eq!(loaded.lot.rooms(), w.lot.rooms());
        assert_eq!(loaded.sims[0].traits, ["lazy"]);
        assert_eq!(loaded.sims[0].gender, "female");
        assert_eq!(loaded.sims[0].attracted_to, ["male"]);
        assert_eq!(loaded.sims[0].appearance["body"], "#fff");
        assert!((loaded.sims[0].mods.walk_speed - 0.8).abs() < 1e-6);
        assert_eq!(loaded.sims[0].feelings.len(), 1);
        assert_eq!(loaded.relationships.get(0, 1), w.relationships.get(0, 1));
        assert_eq!(
            loaded.relationships.chemistry(0, 1),
            w.relationships.chemistry(0, 1)
        );
        for (a, b) in loaded.sims.iter().zip(&w.sims) {
            assert_eq!(a.needs, b.needs);
        }
        assert_eq!(loaded.sims[0].pos, w.sims[0].pos);
        assert_eq!(
            loaded.sims[1].queue().count() + loaded.sims[1].current().is_some() as usize,
            1
        );
        // Saving the loaded world again is stable.
        assert_eq!(
            World::from_save_json(CONTENT, &loaded.save_json())
                .unwrap()
                .save_json(),
            loaded.save_json()
        );
    }

    #[test]
    fn appearance_is_kept_whole() {
        // The web keeps more than colours in the appearance (a voice chosen in the creator).
        let look =
            serde_json::json!({"body":"#fff","voice":{"pitch":1.08,"speed":0.95,"seed":12345}});
        let lot = LOT.replace(r##"{"body":"#fff"}"##, &look.to_string());
        let w = World::from_json(CONTENT, &lot, 3).unwrap();
        assert_eq!(w.sims[0].appearance, look);
        let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
        assert_eq!(loaded.sims[0].appearance, look);
    }

    #[test]
    fn lying_sim_is_saved_standing_in_front_of_the_bed() {
        let mut w = World::from_json(CONTENT, LOT, 3).unwrap();
        w.autonomy = false;
        w.sims[0].needs[1] = 0.1;
        w.apply(Command::Use {
            sim: 0,
            object: 1,
            interaction: 0,
        })
        .unwrap();
        for _ in 0..crate::secs(40.0) {
            w.tick_once();
        }
        assert_eq!(w.sims[0].pose, crate::content::Pose::Lie);
        let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
        let (x, z) = loaded.sims[0].tile();
        assert!(
            !loaded.blocked()[loaded.lot.tile_index(x, z)],
            "must not spawn inside the bed"
        );
    }

    #[test]
    fn drops_objects_missing_from_content() {
        let w = World::from_json(CONTENT, LOT, 3).unwrap();
        let json = w
            .save_json()
            .replace(r#""def":"fridge""#, r#""def":"jukebox""#);
        let loaded = World::from_save_json(CONTENT, &json).unwrap();
        assert_eq!(loaded.objects.len(), 1);
    }

    #[test]
    fn old_upgrade_tasks_are_dropped_on_load() {
        let w = World::from_json(CONTENT, LOT, 3).unwrap();
        let json = w.save_json().replacen(
            r#""tasks":[]"#,
            r#""tasks":[{"type":"upgrade","object":1},{"type":"moveTo","x":2.5,"z":2.5,"directed":true}]"#,
            1,
        );
        let loaded = World::from_save_json(CONTENT, &json).unwrap();
        assert_eq!(loaded.sims[0].queue().count(), 1, "only the move is kept");
    }

    #[test]
    fn saves_with_moodlets_still_load() {
        // Saves written before the moodlet → feeling rename store `moodlets` per Sim.
        let mut w = World::from_json(CONTENT, LOT, 3).unwrap();
        crate::social::add_feeling(&mut w.sims[0].feelings, 0, &w.content.feelings, w.tick);
        let mut saved: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
        let sim = saved["sims"][0].as_object_mut().unwrap();
        let feelings = sim.remove("feelings").expect("new saves write `feelings`");
        assert_eq!(feelings[0]["id"], "happy");
        sim.insert("moodlets".into(), feelings);
        let loaded = World::from_save_json(CONTENT, &saved.to_string()).unwrap();
        assert_eq!(loaded.sims[0].feelings.len(), 1);
        assert_eq!(
            loaded.content.feelings[loaded.sims[0].feelings[0].def].id,
            "happy"
        );
    }

    #[test]
    fn version_one_saves_still_load() {
        let v1 = r#"{"version":1,"tick":10,"speed":1,"autonomy":true,"rng":5,
            "lot":{"width":4,"depth":4,"hEdges":"00000000000000000000","vEdges":"00000000000000000000"},
            "objects":[],"sims":[{"name":"Old","appearance":null,"traits":[],"perks":[],"pos":[1.5,1.5],"yaw":0,
              "needs":{"hunger":0.5},"tasks":[]}]}"#;
        let w = World::from_save_json(CONTENT, v1).unwrap();
        assert_eq!(w.sims[0].gender, "female", "defaults to the first gender");
        assert_eq!(w.sims[0].attracted_to.len(), 2);
    }

    #[test]
    fn diagonal_walls_survive_saving() {
        let lot = LOT.replacen(
            r#""doors":"#,
            r#""diagonals":[{"x":2,"z":8,"dir":"dp"},{"x":3,"z":9,"dir":"dp","kind":"window"},
                {"x":4,"z":8,"dir":"dn","kind":"door"}],"doors":"#,
            1,
        );
        let w = World::from_json(CONTENT, &lot, 3).unwrap();
        let json = w.save_json();
        let saved: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(saved["version"], super::SAVE_VERSION);
        let diags = saved["lot"]["diagonals"].as_str().unwrap();
        assert_eq!(diags.len(), 12 * 12);
        let loaded = World::from_save_json(CONTENT, &json).unwrap();
        use crate::lot::{DiagDir, Diagonal, Edge};
        assert_eq!(
            loaded.lot.diag(3, 9),
            Some(Diagonal {
                dir: DiagDir::Dp,
                edge: Edge::Window
            })
        );
        assert_eq!(
            loaded.lot.diag(4, 8),
            Some(Diagonal {
                dir: DiagDir::Dn,
                edge: Edge::Door
            })
        );
        assert_eq!(loaded.lot.half_rooms(), w.lot.half_rooms());
        assert_eq!(loaded.save_json(), json);
    }

    #[test]
    fn saves_without_diagonals_still_load() {
        // A version 4 save (before diagonal walls): no `diagonals` in the lot.
        let w = World::from_json(CONTENT, LOT, 3).unwrap();
        let mut saved: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
        assert!(
            saved["lot"].get("diagonals").is_none(),
            "lots without diagonals don't write them"
        );
        saved["version"] = 4.into();
        let loaded = World::from_save_json(CONTENT, &saved.to_string()).unwrap();
        assert!(loaded.lot.diagonals().iter().all(Option::is_none));
        assert_eq!(loaded.lot.rooms(), w.lot.rooms());
        // A corrupt or wrongly sized diagonal string is an error, not a silent change.
        saved["lot"]["diagonals"] = "01".into();
        assert!(World::from_save_json(CONTENT, &saved.to_string()).is_err());
    }

    #[test]
    fn windows_survive_saving() {
        let lot = LOT.replacen(
            r#""doors":"#,
            r#""windows":[{"x":6,"z":8,"axis":"z"}],"doors":"#,
            1,
        );
        let w = World::from_json(CONTENT, &lot, 3).unwrap();
        assert_eq!(w.lot.v_edge(6, 8), crate::lot::Edge::Window);
        let json = w.save_json();
        let saved: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(saved["lot"]["vEdges"].as_str().unwrap().contains('3'));
        let loaded = World::from_save_json(CONTENT, &json).unwrap();
        assert_eq!(loaded.lot.v_edge(6, 8), crate::lot::Edge::Window);
        assert_eq!(loaded.lot.v_edge(6, 4), crate::lot::Edge::Door);
        assert_eq!(loaded.save_json(), json);
    }

    /// A saved game as JSON, edited by `f`.
    fn edited(f: impl FnOnce(&mut serde_json::Value)) -> String {
        let w = World::from_json(CONTENT, LOT, 3).unwrap();
        let mut saved: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
        f(&mut saved);
        saved.to_string()
    }

    /// Damaged saves are refused when they're read, not by a panic later: a household on a plot
    /// the town doesn't have, a baby expected by someone who isn't there, an item id past the
    /// last one there can be.
    #[test]
    fn damaged_saves_are_refused_on_loading() {
        let refused = |json: String, why: &str| {
            let err = World::from_save_json(CONTENT, &json)
                .err()
                .unwrap_or_else(|| panic!("{why}: loaded"));
            assert!(err.to_string().starts_with("corrupt save"), "{why}: {err}");
        };
        refused(
            edited(|s| s["households"][0]["plot"] = 7.into()),
            "unknown plot",
        );
        refused(
            edited(|s| {
                s["households"][0]["expecting"] = serde_json::json!({"parents": [0, 9], "due": 3})
            }),
            "unknown parent",
        );
        let item = serde_json::json!([{"id": u32::MAX, "def": "x", "at": "floor", "x": 1, "z": 1, "quality": 1.0, "madeAt": 0, "household": 0}]);
        // (An item the content doesn't know is dropped: give the content one.)
        let content = CONTENT.replacen(
            r#""needs":"#,
            r#""items":[{"id":"x","label":"X","model":"x"}],"needs":"#,
            1,
        );
        let json = edited(|s| s["items"] = item);
        let err = World::from_save_json(&content, &json)
            .err()
            .map(|e| e.to_string());
        assert_eq!(
            err.as_deref(),
            Some("corrupt save: an item id is out of range")
        );
        // An expected baby by residents who are there loads.
        let json = edited(|s| {
            s["households"][0]["expecting"] = serde_json::json!({"parents": [0, 1], "due": 3})
        });
        assert!(World::from_save_json(CONTENT, &json).is_ok());
    }

    /// Numbers out of range in a save (here written as 1e39, which reads as infinity) are brought
    /// back in: a facing, friendship and romance (±100), a nap or a visit's end.
    #[test]
    fn numbers_out_of_range_load_in_range() {
        let json = edited(|s| {
            s["sims"][0]["yaw"] = serde_json::json!(1e39);
            s["sims"][0]["napMinutes"] = serde_json::json!(1e39);
            s["sims"][0]["visiting"] = serde_json::json!([0, 1e39]);
            for r in s["relationships"].as_array_mut().unwrap() {
                r["friendship"] = serde_json::json!(1e39);
                r["romance"] = serde_json::json!(-1e39);
            }
        });
        let w = World::from_save_json(CONTENT, &json).unwrap();
        assert_eq!(w.sims[0].yaw, 0.0);
        let r = w.relationships.get(0, 1);
        assert_eq!((r.friendship, r.romance), (100.0, -100.0));
        assert_eq!(w.sims[0].nap_until, w.tick + 1, "no nap for ever");
    }

    /// Whatever goes wrong while playing, the save can be loaded: a value that isn't a number
    /// (NaN, infinity) is written as a usable one, never as `null`.
    #[test]
    fn broken_numbers_never_make_a_save_unloadable() {
        let mut w = World::from_json(CONTENT, LOT, 3).unwrap();
        let s = &mut w.sims[0];
        s.pos = [f32::NAN, 3.0];
        s.yaw = f32::INFINITY;
        s.age = f32::NAN;
        s.needs[0] = f32::NAN;
        s.skills[0] = f32::NEG_INFINITY;
        w.objects[0].wear = f32::NAN;
        w.relationships.adjust(0, 1, f32::NAN, 0.0);
        w.exits.push([f32::NAN, 1.0]);
        let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
        assert!(loaded.sims[0].pos.iter().all(|p| p.is_finite()));
        assert_eq!(loaded.sims[0].needs[0], 0.75);
        assert_eq!(loaded.relationships.get(0, 1).friendship, 0.0);
        assert!(loaded.exits.is_empty());
    }

    /// v36: an older file's ticks are rescaled as it loads, from 20 to a real second and 20 to a
    /// game minute (`Rescale`); at those rates still, a v35 file loads just as it was saved, a car
    /// on its way and the story as they were.
    #[test]
    fn a_v35_file_loads_as_it_was_saved() {
        const CONTENT: &str = r#"{
            "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
            "objects":[{"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"tags":["lounge"]}]}],
            "careers":[{"id":"office","label":"Office","workEffects":{"hunger":-0.2},"levels":[
                {"title":"Intern","pay":100,"start":9,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
            "economy":{"startingFunds":500}}"#;
        // A house on a street, its car parked beside it; two grown-ups at the same office.
        const TOWN: &str = r#"{"width":60,"depth":14,"exits":[[0.5,9.5],[59.5,9.5]],
            "plots":[{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}],
            "households":[{"name":"Ones","plot":0}],
            "sims":[{"name":"Ada","household":0,"job":{"career":"office"},"x":25.5,"z":3.5},
                    {"name":"Bo","household":0,"job":{"career":"office"},"x":26.5,"z":3.5}],
            "cars":[{"plot":0,"x":34.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":2}]}"#;
        let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
        w.autonomy = false;
        let by = crate::clock::tick_at(1, 10.0 * 60.0).unwrap();
        while !(w.cars[0].state == crate::cars::CarState::Leaving && w.cars[0].progress > 0.0) {
            w.tick_once();
            assert!(w.tick < by, "the car never set off");
        }
        w.events
            .push(w.tick - 7, crate::social::EventKind::Met, 0, 1, None);
        let json = w.save_json();
        let mut old: serde_json::Value = serde_json::from_str(&json).unwrap();
        old["version"] = 35.into();
        let loaded = World::from_save_json(CONTENT, &old.to_string()).unwrap();
        // (As this build counts them: the same at 20 and 20.)
        let r = super::Rescale::of(35);
        assert_eq!(loaded.tick, r.tick(w.tick));
        assert_eq!(loaded.cars[0].since, r.tick(w.cars[0].since));
        assert_eq!(
            loaded.cars[0].progress.to_bits(),
            r.route(w.cars[0].progress).to_bits()
        );
        let ticks = |w: &World| w.events.iter().map(|e| e.tick).collect::<Vec<_>>();
        assert_eq!(
            ticks(&loaded),
            ticks(&w).into_iter().map(|t| r.tick(t)).collect::<Vec<_>>()
        );
        if r == super::Rescale::of(super::SAVE_VERSION) {
            assert_eq!(
                loaded.save_json(),
                World::from_save_json(CONTENT, &json).unwrap().save_json()
            );
        }
    }

    /// From 20 ticks a game minute to 75 (and 20 a real second to 60): each tick of the clock is
    /// the same minute of the same day, the same tick stays the same and never stays never; places
    /// along a way are three times the ticks.
    #[test]
    fn older_ticks_keep_their_minute_at_another_rate() {
        let r = super::Rescale {
            game: (75, 20),
            drive: 3.0,
        };
        // (The clock by hand: tick 0 is 08:00 on day 1.)
        let at = |t: u64, per_minute: u64| {
            let m = t + 8 * 60 * per_minute;
            (
                m / (1440 * per_minute),
                m % (1440 * per_minute) / per_minute,
            )
        };
        for t in [0, 1, 19, 20, 21, 1_199, 9_599, 9_600, 28_813, 123_456_789] {
            assert_eq!(at(t, 20), at(r.tick(t), 75), "tick {t}");
        }
        assert_eq!(r.tick(0), 0);
        assert_eq!(r.tick(1_000), 3_750);
        // A traffic light's stage, at it for 100 ticks (5 seconds) of the file's clock at 1000: 5
        // seconds before the clock's tick still (300 ticks, not the game minutes' 375).
        assert_eq!(r.drive_since(900, 1_000), 3_450);
        assert_eq!(r.drive_since(1_000, 1_000), 3_750);
        assert_eq!(r.drive_since(0, 10), 8);
        let car: crate::cars::Car = serde_json::from_value(serde_json::json!({
            "plot":0,"household":0,"spot":[34.0,3.0],"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,
            "body":1,"paint":2,"owners":[0],"state":"leaving","since":1000,"readyAt":2000,
            "progress":40.5,"detour":{"home":false,"from":30.0,"back":12.0,"avoid":3},
            "passBy":{"home":false,"taxi":1,"from":50.0,"out":55.0,"back":70.0,"to":75.0,"off":-2.0,"at":[1.0,2.0]}}))
        .unwrap();
        let cars = r.cars(std::slice::from_ref(&car));
        let c = &cars[0];
        assert_eq!((c.since, c.ready_at), (3_750, 7_500));
        assert_eq!(c.progress, 121.5);
        let d = c.detour.unwrap();
        assert_eq!((d.from, d.back, d.avoid), (90.0, 36.0, 3));
        let p = c.pass_by.unwrap();
        assert_eq!(
            (p.from, p.out, p.back, p.to, p.off),
            (150.0, 165.0, 210.0, 225.0, -2.0)
        );
        let mut taxis: crate::taxi::Taxis = serde_json::from_value(serde_json::json!({
            "runs":[{"id":0,"group":[0,1],"enter":100,"dueOut":220,"plannedOut":200,"depart":0,
                "stops":[{"sim":0,"plot":0,"due":160,"leave":140,"state":"booked"}],
                "progress":12.0,"since":160}]}))
        .unwrap();
        r.taxis(&mut taxis);
        let run = &taxis.runs[0];
        assert_eq!(
            (
                run.enter,
                run.due_out,
                run.planned_out,
                run.depart,
                run.since
            ),
            (375, 825, 750, 0, 600)
        );
        assert_eq!((run.stops[0].due, run.stops[0].leave), (600, 525));
        assert_eq!(run.progress, 36.0);
        // Today's file needs nothing.
        let today = super::Rescale::of(super::SAVE_VERSION);
        assert_eq!(today.drive_since(900, 1_000), 900);
        assert_eq!(
            super::Rescale::of(super::SAVE_VERSION),
            super::Rescale {
                game: (1, 1),
                drive: 1.0
            }
        );
        // (A v36 file's cars timed in real seconds: a place along the way a quarter of the ticks
        // on, the clock as it was; a v35 file's both.)
        assert_eq!(
            super::Rescale::of(36),
            super::Rescale {
                game: (1, 1),
                drive: 1.0 / crate::DRIVE_SCALE
            }
        );
        assert_eq!(
            super::Rescale::of(35).drive,
            crate::DRIVE_TICKS_PER_SECOND / super::OLD_TICKS_PER_SECOND as f32
        );
    }

    /// v23: the seed the game started from is kept (absent in older saves: 0, unknown).
    #[test]
    fn the_starting_seed_is_saved() {
        let w = World::from_json(CONTENT, LOT, 1234).unwrap();
        assert_eq!(w.seed, 1234);
        let json = w.save_json();
        assert_eq!(World::from_save_json(CONTENT, &json).unwrap().seed, 1234);
        let mut old: serde_json::Value = serde_json::from_str(&json).unwrap();
        old["version"] = 22.into();
        old.as_object_mut().unwrap().remove("seed");
        assert_eq!(
            World::from_save_json(CONTENT, &old.to_string())
                .unwrap()
                .seed,
            0
        );
    }
}
