//! The lot: a tile grid with walls, doors and windows on tile edges.
//!
//! Coordinates: tile `(x, z)` covers `[x, x+1] × [z, z+1]` in metres (1 tile = 1 m).
//! Horizontal edges lie on the line `z = const` and run along x; vertical edges
//! lie on `x = const` and run along z.
//!
//! **Diagonal walls** run corner to corner across one tile (at most one per tile): `/`
//! ([`DiagDir::Dp`]) from `(x, z)` to `(x + 1, z + 1)`, `\` ([`DiagDir::Dn`]) from
//! `(x, z + 1)` to `(x + 1, z)`. They split the tile into two triangular halves: half 0
//! touches the tile's `-z` side, half 1 its `+z` side (see [`Lot::half_at_side`]). Rooms are
//! found per half, so the two halves can belong to different rooms (inside and outside).
//!
//! Movement stays on whole tiles: nobody stands on a tile crossed by a diagonal wall or
//! window (it blocks the whole tile; see `NavGrid`), while a diagonal *door* tile is walked
//! across like a doorway. Objects never stand on a tile with a diagonal.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::Error;
use crate::path::join;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Open,
    Wall,
    /// A wall with a doorway: Sims walk through, but it still separates rooms.
    Door,
    /// A wall with a window: blocks walking and separates rooms like a wall.
    Window,
    /// A garden fence: blocks walking but makes no rooms (a fenced garden stays outdoors).
    Fence,
    /// A gate in a fence: walked through; makes no rooms.
    Gate,
}

impl Edge {
    /// Whether Sims can't cross this edge.
    pub fn blocks(self) -> bool {
        matches!(self, Edge::Wall | Edge::Window | Edge::Fence)
    }

    /// Whether a wall stands on this edge (plain, or with a door or window in it). Walls make
    /// rooms; fences and gates don't.
    pub fn is_wall(self) -> bool {
        matches!(self, Edge::Wall | Edge::Door | Edge::Window)
    }

    /// Whether a fence or a gate stands on this edge.
    pub fn is_fence(self) -> bool {
        matches!(self, Edge::Fence | Edge::Gate)
    }
}

/// Which way a diagonal wall runs across its tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagDir {
    /// `/`: from corner `(x, z)` to `(x + 1, z + 1)`.
    Dp,
    /// `\`: from corner `(x, z + 1)` to `(x + 1, z)`.
    Dn,
}

/// A diagonal wall across a tile: its direction and what stands there (never `Open`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Diagonal {
    pub dir: DiagDir,
    pub edge: Edge,
}

impl Diagonal {
    /// Whether this diagonal ends at grid corner `(cx, cz)` of tile `(x, z)`.
    pub fn touches(self, x: i32, z: i32, cx: i32, cz: i32) -> bool {
        match self.dir {
            DiagDir::Dp => (cx, cz) == (x, z) || (cx, cz) == (x + 1, z + 1),
            DiagDir::Dn => (cx, cz) == (x, z + 1) || (cx, cz) == (x + 1, z),
        }
    }
}

/// A wall's place on the lot: an `h` or `v` grid edge (as in [`Lot::h_edge`] / [`Lot::v_edge`]),
/// or the diagonal across tile `(x, z)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdgeRef {
    H(u16, u16),
    V(u16, u16),
    Diag(u16, u16),
}

/// Half-height wall (see [`EdgeLook::form`]).
pub const FORM_HALF: u8 = 1;

/// How a wall looks. Presentation only: walking, rooms and light never depend on it (a half
/// wall blocks and separates like a full one), but it is saved and costs money to change.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EdgeLook {
    /// Covering of each face: 0 the automatic look, else an index into the wall coverings + 1.
    /// Face 0 looks towards `-z` (`h` edges), `-x` (`v` edges) or the tile's half 0
    /// (diagonals); face 1 the other way.
    pub sides: [u8; 2],
    /// 0 = full height, [`FORM_HALF`] = half wall (no doors or windows in it).
    pub form: u8,
    /// Door or window style: an index into the content's door / window styles.
    pub style: u8,
    /// Doors and windows the player resized: the sill and the top (centimetres above the
    /// floor; doors have no sill). None: the style's own.
    pub sill: Option<u16>,
    pub head: Option<u16>,
}

/// A side of a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    NegX,
    PosX,
    NegZ,
    PosZ,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::PosX, Side::NegX, Side::PosZ, Side::NegZ];

    pub fn offset(self) -> (i32, i32) {
        match self {
            Side::NegX => (-1, 0),
            Side::PosX => (1, 0),
            Side::NegZ => (0, -1),
            Side::PosZ => (0, 1),
        }
    }

    pub fn opposite(self) -> Side {
        match self {
            Side::NegX => Side::PosX,
            Side::PosX => Side::NegX,
            Side::NegZ => Side::PosZ,
            Side::PosZ => Side::NegZ,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    /// Edge runs along x (horizontal, on a `z` grid line).
    X,
    /// Edge runs along z (vertical, on an `x` grid line).
    Z,
}

/// Lot description as stored in lot JSON files.
#[derive(Debug, Deserialize)]
pub struct LotFile {
    /// Living (default) or Creative.
    #[serde(default)]
    pub mode: crate::world::GameMode,
    /// How fast residents age (default: normal).
    #[serde(default)]
    pub lifespan: crate::lifecycle::Lifespan,
    /// The most residents the town grows to by newcomers and births (optional; absent:
    /// `world::MAX_SIMS`, and more than it counts as it). Saved with the town.
    #[serde(default, rename = "residentsMost")]
    pub residents_most: Option<u32>,
    pub width: u16,
    pub depth: u16,
    /// Storeys the file's houses have (default 1). With more, walls, doors, windows, diagonals and
    /// objects of storey `k` are given on rows `k * depth ..` (see `storeys.rs`).
    #[serde(default)]
    pub storeys: Option<u8>,
    /// Whether the file's houses have a basement (default not): its walls, doors and objects are
    /// given on the block of rows after the storeys', `storeys * depth ..` (see `storeys.rs`).
    #[serde(default)]
    pub basement: bool,
    /// Wall segments `[x0, z0, x1, z1]` along grid lines.
    #[serde(default)]
    pub walls: Vec<[u16; 4]>,
    #[serde(default)]
    pub doors: Vec<DoorRaw>,
    /// Windows, like doors: each must sit on a wall.
    #[serde(default)]
    pub windows: Vec<DoorRaw>,
    /// Diagonal walls (optional), one per tile.
    #[serde(default)]
    pub diagonals: Vec<DiagRaw>,
    /// Garden fences, hedges and gates the town put up (optional; on the ground storey, never on a
    /// wall): runs along grid lines, as walls are given.
    #[serde(default)]
    pub fences: Vec<FenceRaw>,
    /// Floor coverings the town laid (optional): rectangles of tiles on any storey's rows, by
    /// content id (`floorCoverings`; an unknown id leaves its tiles to the automatic floor).
    #[serde(default)]
    pub floors: Vec<FloorRaw>,
    #[serde(default)]
    pub objects: Vec<PlacementRaw>,
    #[serde(default)]
    pub sims: Vec<SimSpawn>,
    /// Regions of a town (residential lots, parks). Optional for single lots. The lots on the ground
    /// come first, in slot order; then every flat (`PlotRaw::of`), each a plot of its own inside its
    /// building's lot on one storey, its id following the last ground lot's.
    #[serde(default)]
    pub plots: Vec<PlotRaw>,
    #[serde(default)]
    pub households: Vec<HouseholdRaw>,
    /// Starting relationships between Sims (indices into `sims`).
    #[serde(default)]
    pub relationships: Vec<BondRaw>,
    /// Where Sims leave town (for work) and come back. Optional. `[x, z]`, or (open-world towns)
    /// `{ x, z, kind }` with `kind` `road` or `stop` (`travel::ExitRaw`).
    #[serde(default)]
    pub exits: Vec<crate::travel::ExitRaw>,
    /// Households' cars on their driveways (`cars.rs`). Optional.
    #[serde(default)]
    pub cars: Vec<crate::cars::CarRaw>,
    /// Every house's driveway, where a car parks (`cars.rs`). Optional.
    #[serde(default)]
    pub drives: Vec<crate::cars::Drive>,
    /// Garages at the inner ends of driveways (`cars.rs`). Optional.
    #[serde(default)]
    pub garages: Vec<crate::cars::Garage>,
    /// Porches over doors (`porches.rs`): the town puts one over the front door the garden path
    /// meets. Absent (a file from before porches): worked out from the doors and the garden paths
    /// as they were drawn (`World::guess_porches`); `[]`: none.
    #[serde(default)]
    pub porches: Option<Vec<crate::porches::Porch>>,
    /// Porch pieces standing on their own (`porches.rs`). Optional.
    #[serde(default, rename = "porchPieces")]
    pub porch_pieces: Vec<crate::porches::PorchPiece>,
    /// Opaque data for the presentation layer (streets, scenery seed...). Saved unchanged.
    #[serde(default)]
    pub meta: serde_json::Value,
    // ---- Open world (sim-travel) -------------------------------
    /// `size`, `roads`, `areas`, `terrain`, `parking` (marked bays, each with the pose a car parks
    /// at) and `carParks` (the rows the bays make) (`travel/`). All optional.
    #[serde(flatten)]
    pub open_world: crate::travel::OpenWorld,
}

#[derive(Debug, Deserialize)]
pub struct PlotRaw {
    pub name: String,
    pub x: i32,
    pub z: i32,
    pub w: i32,
    pub d: i32,
    #[serde(default)]
    pub public: bool,
    /// Where visitors arrive (in front of the door).
    #[serde(default)]
    pub entry: Option<[f32; 2]>,
    // ---- Open world (sim-travel) ----
    /// The area (`areas` index) it is in, and what kind of plot it is.
    #[serde(default)]
    pub area: Option<u32>,
    #[serde(default)]
    pub kind: Option<crate::travel::PlotKind>,
    /// When a public place is open (`world::PlotHours`, as its place's `TownPlace.hours` on the
    /// page); absent: always.
    #[serde(default)]
    pub hours: Option<crate::world::PlotHours>,
    // ---- Flats (kind `flat`) ----
    /// A flat: the ground lot of its building (a flats lot, or a place's with flats over it). Its
    /// rectangle is on the ground's rows, inside that lot's; its `entry` is the building's street
    /// door, on the ground, which its flats share.
    #[serde(default)]
    pub of: Option<u32>,
    /// The storey a flat is on (0: the ground).
    #[serde(default)]
    pub storey: i8,
    /// A flat's landing tile before its own door, and the tile behind that door (its middle), both
    /// on its storey's rows (`storeys.rs`).
    #[serde(default)]
    pub door: Option<[i32; 2]>,
    #[serde(default)]
    pub inside: Option<[f32; 2]>,
}

#[derive(Debug, Deserialize)]
pub struct HouseholdRaw {
    pub name: String,
    #[serde(default)]
    pub plot: Option<u32>,
    #[serde(default)]
    pub player: bool,
    /// Defaults to the content's starting funds.
    #[serde(default)]
    pub funds: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JobRaw {
    pub career: String,
    #[serde(default)]
    pub level: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BondRaw {
    pub a: usize,
    pub b: usize,
    /// Key into the content's `bondPresets`.
    pub preset: String,
}

/// A run of fence (or a gate in one) along a grid line, `[x0, z0, x1, z1]` as walls are given,
/// in a fence style (`fenceStyles` id; an unknown one or none: the first).
#[derive(Debug, Deserialize)]
pub struct FenceRaw {
    pub line: [u16; 4],
    #[serde(default)]
    pub style: Option<String>,
    /// A gate: walked through.
    #[serde(default)]
    pub gate: bool,
}

/// A rectangle of tiles with a floor covering: `w` × `d` tiles from `(x, z)` (one tile when
/// absent), by its `floorCoverings` id. A town file lays them so; a save keeps them so too.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorRaw {
    pub x: u16,
    pub z: u16,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub w: u16,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub d: u16,
    pub covering: String,
}

fn one() -> u16 {
    1
}

fn is_one(v: &u16) -> bool {
    *v == 1
}

/// A door or window on the wall edge at `(x, z)` (see `Axis`).
#[derive(Debug, Deserialize)]
pub struct DoorRaw {
    pub x: u16,
    pub z: u16,
    pub axis: Axis,
}

/// A diagonal wall across tile `(x, z)`, optionally with a door or window in it.
#[derive(Debug, Deserialize)]
pub struct DiagRaw {
    pub x: u16,
    pub z: u16,
    pub dir: DiagDir,
    /// `wall` (default), `door` or `window`.
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlacementRaw {
    pub def: String,
    pub x: i32,
    pub z: i32,
    /// Facing: 0 = +z, 1 = +x, 2 = -z, 3 = -x.
    #[serde(default)]
    pub rot: u8,
    /// Degrees past the facing (0..90), for things that turn freely.
    #[serde(default)]
    pub turn: u8,
    /// How far it stands off the middle of its footprint, in hundredths of a tile (up to
    /// `MAX_OFFSET` each way; things on a wall only along it).
    #[serde(default)]
    pub offset: [i8; 2],
    /// Its own size: centimetres across and deep before turning; absent: its kind's.
    #[serde(default)]
    pub size: Option<[u16; 2]>,
    /// Centimetres taller (negative: shorter) than its kind.
    #[serde(default)]
    pub tall: i16,
    /// The plot it belongs to though it stands elsewhere (a flat's letterbox in the hall, its bike
    /// in the yard): `world::ObjectInstance::keeper`. Absent: the plot it stands on.
    #[serde(default)]
    pub plot: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimSpawn {
    pub name: String,
    /// Gender id; defaults to the first gender in the content.
    #[serde(default)]
    pub gender: Option<String>,
    /// Gender ids this Sim is attracted to; `None` means all.
    #[serde(default)]
    pub attracted_to: Option<Vec<String>>,
    #[serde(default)]
    pub household: u32,
    #[serde(default)]
    pub job: Option<JobRaw>,
    /// Opaque appearance data passed through to the renderer/UI unchanged.
    #[serde(default)]
    pub appearance: serde_json::Value,
    #[serde(default)]
    pub traits: Vec<String>,
    #[serde(default)]
    pub perks: Vec<String>,
    /// Age in years (default: content `life.startAge`, by name).
    #[serde(default)]
    pub age: Option<f32>,
    /// Starting skill levels by id (on top of what traits give).
    #[serde(default)]
    pub skills: std::collections::HashMap<String, f32>,
    pub x: f32,
    pub z: f32,
}

pub const OUTDOORS: u16 = 0;

/// Tiles along each side of a storage chunk (see `Lot`).
pub const CHUNK: usize = 16;
const SHIFT: usize = 4;
const MASK: usize = CHUNK - 1;
/// Tiles (slots) per chunk.
pub const CHUNK_TILES: usize = CHUNK * CHUNK;
/// The slot every tile of a chunk that isn't allocated reads (see `Lot`): open edges, no
/// diagonal, outdoors. Nothing is ever written there.
pub const VOID: usize = 0;
/// The widest a town file may be, and the deepest (each storey is as deep again).
pub const MAX_SIDE: usize = 1024;
/// The deepest a saved grid may be: the ground, three storeys above it and a basement (and one
/// more for content with more storeys).
pub const MAX_ROWS: usize = MAX_SIDE * 5;

/// The tile grid, stored in chunks of `CHUNK × CHUNK` tiles.
///
/// Every per-tile value (the edges on a tile's `-z` and `-x` sides, its diagonal, its rooms) is
/// kept by *slot*: [`Lot::tile_index`] maps a tile to its slot, and per-tile arrays elsewhere
/// (`World::blocked`, `World::dirt`, search buffers) are indexed by the same slots, sized
/// [`Lot::tile_slots`]. The ground storey's chunks are always there; chunks of the rows past it
/// (the storeys above and the basement, see `storeys.rs`) are allocated on first write, and
/// until then every tile of theirs reads slot [`VOID`]: all open, outdoors, nothing standing
/// there. Slots only ever grow (a new chunk's go on the end), so a slot keeps its tile. Whole-grid
/// passes go row by row ([`Lot::tiles`]), so what they find (room numbers, orders) doesn't
/// depend on where chunks are stored.
///
/// The chunk grid is one tile wider and deeper than the lot, for the edges along its far sides
/// (`h_edge(x, depth)`, `v_edge(width, z)`).
#[derive(Debug, Clone)]
pub struct Lot {
    pub width: usize,
    pub depth: usize,
    /// Chunks across and down.
    cols: usize,
    rows: usize,
    /// Per chunk (row-major): its first slot, 0 while it isn't allocated.
    base: Vec<u32>,
    /// The allocated chunks in slot order.
    order: Vec<u32>,
    /// Per chunk: edges and diagonals in it that aren't open (whole-lot wall scans skip the rest).
    built: Vec<u16>,
    /// Per chunk: tiles in it with a room (either half), from the last `compute_rooms`.
    indoor: Vec<u16>,
    /// Per slot: the horizontal edge on the tile's `-z` side (`h_edge(x, z)`).
    h_edges: Vec<Edge>,
    /// Per slot: the vertical edge on the tile's `-x` side (`v_edge(x, z)`).
    v_edges: Vec<Edge>,
    /// Diagonal wall per slot, if any.
    diags: Vec<Option<Diagonal>>,
    /// Room id per slot; `OUTDOORS` for outside. A tile split by a diagonal reports the
    /// room of an indoor half if it has one (see `halves` for both).
    rooms: Vec<u16>,
    /// Room id of each half of each slot (equal for tiles without a diagonal).
    halves: Vec<[u16; 2]>,
    /// Looks of the walls that aren't plain (absent: the default look).
    looks: BTreeMap<EdgeRef, EdgeLook>,
    /// Floor covering per tile `(x, z)` (a floor covering + 1; absent: the automatic look). Kept
    /// when the room around it is torn down, so rebuilding it brings the floor back.
    floors: BTreeMap<(u16, u16), u8>,
    /// Per room id from 1 (index `id - 1`): the key rooms are numbered by and the tiles round
    /// it, from the last `compute_rooms`.
    room_meta: Vec<RoomMeta>,
    /// The edges whose wall-ness changed, and the diagonals put up, taken down or turned, since
    /// the rooms were last found: `compute_rooms` finds them again only round these (see
    /// `lot/rooms_near.rs`). None: everywhere.
    pending: Option<Vec<EdgeRef>>,
    /// How the rooms changed since `take_room_changes` (for the rooms' scores).
    changes: RoomChanges,
    /// While an edit can be taken back: what it changed, as it was before (`log_begin`).
    log: Option<Vec<LotChange>>,
}

/// A room's numbering key (its first tile half, row by row: `compute_rooms`) and the tiles
/// `[x0, z0, x1, z1)` round it.
#[derive(Debug, Clone, Copy)]
struct RoomMeta {
    key: u64,
    /// (Tiles fit in 16 bits: `MAX_SIDE`, `MAX_ROWS`.)
    rect: [u16; 4],
}

impl RoomMeta {
    fn new(key: u64, rect: [i32; 4]) -> Self {
        Self {
            key,
            rect: rect.map(|v| v.clamp(0, u16::MAX as i32) as u16),
        }
    }

    fn rect(&self) -> [i32; 4] {
        self.rect.map(i32::from)
    }
}

/// How the rooms changed since they were last taken (`Lot::take_room_changes`), for whatever
/// scores them (`World::refresh_rooms`): found everywhere again, or only round `rect`, with the
/// rooms elsewhere as they were, renumbered by `remap`.
#[derive(Debug, Clone, Default)]
pub struct RoomChanges {
    /// Found everywhere again: nothing else here says anything.
    pub full: bool,
    /// The tiles `[x0, z0, x1, z1)` (stacked rows) whose room may have changed: every room found
    /// again lies inside, and every room that went lay inside.
    pub rect: Option<[i32; 4]>,
    /// Per room id as it was (index), the id it has now; `OUTDOORS` for a room that went (or
    /// was found again, inside `rect`). None: every id is as it was.
    pub remap: Option<Vec<u16>>,
}

/// One change to the lot, as it was before (see `Lot::log_begin`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LotChange {
    H(u16, u16, Edge),
    V(u16, u16, Edge),
    Diag(u16, u16, Option<Diagonal>),
    Look(EdgeRef, EdgeLook),
    Floor(u16, u16, u8),
}

/// Where the log of an edit started (`Lot::log_begin`).
#[derive(Debug, Clone, Copy)]
pub struct LogMark {
    at: usize,
    /// This mark turned the log on (and ends it).
    started: bool,
}

/// At most this many edges wait for `compute_rooms` before it finds every room again instead
/// (a whole house laid at once, a lot loaded).
const PENDING_MAX: usize = 4096;

impl PartialEq for Lot {
    fn eq(&self, o: &Self) -> bool {
        if (self.width, self.depth) != (o.width, o.depth)
            || self.looks != o.looks
            || self.floors != o.floors
        {
            return false;
        }
        if self.base == o.base {
            return self.h_edges == o.h_edges
                && self.v_edges == o.v_edges
                && self.diags == o.diags
                && self.halves == o.halves
                && self.rooms == o.rooms;
        }
        // Stored differently: tile by tile, far edges included.
        (0..=self.depth).all(|z| {
            (0..=self.width).all(|x| {
                let (a, b) = (self.slot(x, z), o.slot(x, z));
                self.h_edges[a] == o.h_edges[b]
                    && self.v_edges[a] == o.v_edges[b]
                    && self.diags[a] == o.diags[b]
                    && self.halves[a] == o.halves[b]
                    && self.rooms[a] == o.rooms[b]
            })
        })
    }
}

mod build;
mod chunks;
mod edges;
mod log;
mod rooms;
mod rooms_near;
mod walls;

use rooms::*;

#[cfg(test)]
mod tests;
