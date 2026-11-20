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
//! 35 = open worlds: the lot's edge and diagonal strings may be
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
//! 38 = the town's living-town fields are in its `meta`:
//! `backdrop` (buildings nobody lives in), `places` (what the town tags name) and `centre` (each
//! main town's square and high street), as the town file had them; and its homes may be terrace
//! lots (`terrace`, narrow houses a metre apart). The simulation reads none of them: a file before
//! it has none and plays as before.
//! 39 = porches are the simulation's (`porches.rs`): the porches over doors (`porches`: each its
//! plot, its door's edge, `axis`, `x`, `z`, and the side it stands on, `out`) and the porch pieces
//! standing on their own (`porchPieces`: a post, a canopy or a step platform, its tiles and its
//! facing), built and taken down in Build mode. A file before it had its porches drawn by the
//! renderer over the front door the garden path met: they're worked out again from its doors and
//! its paths as it loads (`World::guess_porches`), so the town looks as it did; it has no pieces.
//! Still 39: the land may be shaped in play (Build mode's Land tab, `land.rs`). Its heights, water
//! levels and cover are the page's, carried in the meta (`meta.land`: the land's size and its
//! tiles, `h` and `wl` coded strings by tile, `cover` run-length; `meta.zones` worked out again
//! from it), saved and loaded as the rest of the meta is, without the simulation reading it. The
//! town file's water mask (`openWorld.terrain.water`) may now change in play too, and is saved as
//! it is then; who may walk where is worked out from it again as the game loads. A file without
//! `meta.land` has its land drawn from the landscape, as before, so nothing changes version.
//! Still 39: a town holds up to 768 residents (`world::MAX_SIMS`, 240 before) and 384 cars
//! (`cars::MAX_CARS`, 128 before). Nothing saved is sized by either: residents, cars and
//! relationships are lists, the chemistry string is as long as the residents there are, and the
//! sets sized by the limit (`world::SimSet`) and the snapshot's layout are not saved. A town may
//! say the most residents it grows to (`residentsMost`, optional): saved with the town when it
//! does, and a file without it (every file before it) grows to the limit.
//! 40 = four speeds: 0 paused, 1 normal, 2 fast, 3 faster (10×) and 4 super speed (as fast as
//! the machine goes, up to 60×, only while the player's household is all asleep or out), with
//! the speed super speed drops back to (`resumeSpeed`). A file before it had 4 and 5 for 30×
//! and 60×: it loads at super speed with 3 to drop back to, which the first step leaves if
//! someone at home is up.
//! Still 40: a public place's plot may say when it is open (`plots[].hours`: minutes of the day it
//! opens and closes and the days it opens, bit 0 Monday, as the town file gave them; `Plot::open_at`).
//! Left out where it has none, so a file without them (every file before it) has its places open
//! at any hour, as before. A town holds up to 128 servers (`staff::MAX_STAFF`, 48 before); nothing
//! of them is saved.
//! Still 40: curtains may be drawn shut (`objects[].shut`, `curtains.rs`), left out while open, so
//! a file without it (every file before it) has its curtains open, as the renderer's clock used to
//! draw them by day, and residents draw them at the next moment they would.
//! Still 40: a household's washing has a fourth pile, the ironed (`households[].laundry[3]`,
//! pressed clothes to wear: `content::Pressed`); a file with three has none ironed, and an older
//! build reads the first three. Where a tool is taken to clean the floor (`sweep.rs`) isn't saved:
//! the use starts again after a load, as any.
//! Still 40: a household may have letters on their way to its letterbox (`households[].letters`, by
//! who wrote them, `post.rs`). Left out where there are none, so a file without them has none.
//! 41 = a household's car fits its best earner's job (`cars/looks.rs`): its body (`cars[].body`,
//! now any of `carBodies`, a van or an off-roader too) and how old it is (`cars[].age`: `old` or
//! `new`, left out when used), and the car a promotion or a new job called for that it hasn't
//! bought yet (`cars[].want`: `body`, `age`, and `by`, who the story says bought it). The story
//! may tell of it (`boughtCar`, the body in `n`, by id in `refs.car`). A file before it has its
//! cars as the town picked them: each is fitted as it loads, as a new town's are, nobody told.

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
    clock::{MAX_SPEED, SUPER_SPEED},
};

mod migrate;
mod pack;
mod read;
mod write;

pub const SAVE_VERSION: u32 = 41;

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
    /// The speed super speed drops back to (absent before v40: 1, or 3 for a file saved at 30×
    /// or 60×).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_speed: Option<u8>,
    /// The game went to super speed by itself (absent: the player picked the speed).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub auto_super: bool,
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
    /// The most residents the town grows to by newcomers and births, as its town file asked
    /// (`World::residents_most_given`; absent: `world::MAX_SIMS`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residents_most: Option<u32>,
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
    /// Porches over doors (absent before v39: worked out from the doors and the garden paths,
    /// `World::guess_porches`; from v39, absent: none) and porch pieces (absent: none).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub porches: Vec<crate::porches::Porch>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub porch_pieces: Vec<crate::porches::PorchPiece>,
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
    /// Curtains drawn shut (absent: open; see `curtains.rs`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub shut: bool,
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
    /// Loads of washing waiting: dirty, washed to dry, dried to fold and ironed to wear (see
    /// `Household::laundry`; absent before v30: one dirty and one washed; before v33 without
    /// the dried; without the ironed, none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub laundry: Option<Vec<f32>>,
    /// Letters on their way to the household's letterbox, by who wrote them (see
    /// `Household::letters`; absent: none).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub letters: Vec<u32>,
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
    /// When the place on it is open (`world::PlotHours`; absent: always).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hours: Option<crate::world::PlotHours>,
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

#[cfg(test)]
use migrate::*;
#[cfg(test)]
use pack::*;

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
    pub(super) fn a_run_too_long_for_any_lot_is_corrupt() {
        assert!(super::expand(&format!("~01,0{}", usize::MAX)).is_err());
        assert_eq!(&*super::expand("~03,1,02").unwrap(), "000100");
    }

    #[test]
    pub(super) fn round_trip_preserves_state() {
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
    pub(super) fn appearance_is_kept_whole() {
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
    pub(super) fn lying_sim_is_saved_standing_in_front_of_the_bed() {
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
    pub(super) fn drops_objects_missing_from_content() {
        let w = World::from_json(CONTENT, LOT, 3).unwrap();
        let json = w
            .save_json()
            .replace(r#""def":"fridge""#, r#""def":"jukebox""#);
        let loaded = World::from_save_json(CONTENT, &json).unwrap();
        assert_eq!(loaded.objects.len(), 1);
    }

    #[test]
    pub(super) fn old_upgrade_tasks_are_dropped_on_load() {
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
    pub(super) fn saves_with_moodlets_still_load() {
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
    pub(super) fn version_one_saves_still_load() {
        let v1 = r#"{"version":1,"tick":10,"speed":1,"autonomy":true,"rng":5,
            "lot":{"width":4,"depth":4,"hEdges":"00000000000000000000","vEdges":"00000000000000000000"},
            "objects":[],"sims":[{"name":"Old","appearance":null,"traits":[],"perks":[],"pos":[1.5,1.5],"yaw":0,
              "needs":{"hunger":0.5},"tasks":[]}]}"#;
        let w = World::from_save_json(CONTENT, v1).unwrap();
        assert_eq!(w.sims[0].gender, "female", "defaults to the first gender");
        assert_eq!(w.sims[0].attracted_to.len(), 2);
    }

    #[test]
    pub(super) fn the_speeds_of_older_saves() {
        // Before v40, 4 and 5 were 30× and 60×: super speed now (30× with Old awake), 10× under it;
        // the speed under super speed is saved.
        for (version, speed, now, back) in
            [(39, 5, 4, 3), (39, 4, 4, 3), (39, 2, 2, 2), (40, 4, 4, 1)]
        {
            let file = format!(
                r#"{{"version":{version},"tick":10,"speed":{speed},"autonomy":true,"rng":5,
                "lot":{{"width":4,"depth":4,"hEdges":"00000000000000000000","vEdges":"00000000000000000000"}},
                "objects":[],"sims":[{{"name":"Old","appearance":null,"traits":[],"perks":[],"pos":[1.5,1.5],"yaw":0,
                  "needs":{{"hunger":0.5}},"tasks":[]}}]}}"#
            );
            let mut w = World::from_save_json(CONTENT, &file).unwrap();
            assert_eq!(
                (w.speed, w.resume_speed),
                (now, back),
                "v{version} at {speed}"
            );
            let again = World::from_save_json(CONTENT, &w.save_json()).unwrap();
            assert_eq!((again.speed, again.resume_speed), (now, back));
            w.advance();
            assert_eq!(w.speed, now, "Old is up: the speed stays");
        }
    }

    #[test]
    pub(super) fn diagonal_walls_survive_saving() {
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
    pub(super) fn saves_without_diagonals_still_load() {
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
    pub(super) fn windows_survive_saving() {
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
    pub(super) fn edited(f: impl FnOnce(&mut serde_json::Value)) -> String {
        let w = World::from_json(CONTENT, LOT, 3).unwrap();
        let mut saved: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
        f(&mut saved);
        saved.to_string()
    }

    /// Damaged saves are refused when they're read, not by a panic later: a household on a plot
    /// the town doesn't have, a baby expected by someone who isn't there, an item id past the
    /// last one there can be.
    #[test]
    pub(super) fn damaged_saves_are_refused_on_loading() {
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
    pub(super) fn numbers_out_of_range_load_in_range() {
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
    pub(super) fn broken_numbers_never_make_a_save_unloadable() {
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
    pub(super) fn a_v35_file_loads_as_it_was_saved() {
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
    pub(super) fn older_ticks_keep_their_minute_at_another_rate() {
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
    pub(super) fn the_starting_seed_is_saved() {
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
