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

use serde::Deserialize;

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
    #[serde(default)]
    pub objects: Vec<PlacementRaw>,
    #[serde(default)]
    pub sims: Vec<SimSpawn>,
    /// Regions of a town (residential lots, parks). Optional for single lots.
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
    /// Opaque data for the presentation layer (streets, scenery seed...). Saved unchanged.
    #[serde(default)]
    pub meta: serde_json::Value,
    // ---- Open world (sim-travel; docs/design/open-world.md) -------------------------------
    /// `size`, `roads`, `areas`, `terrain` and `parking` (`travel.rs`). All optional.
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

impl Lot {
    /// An open lot whose every row is ground (all its chunks allocated).
    pub fn new(width: usize, depth: usize) -> Self {
        Self::with_ground(width, depth, depth)
    }

    /// An open lot whose first `ground` rows are always there; the rows past them are
    /// allocated chunk by chunk as they're built on.
    pub fn with_ground(width: usize, depth: usize, ground: usize) -> Self {
        let (cols, rows) = (width / CHUNK + 1, depth / CHUNK + 1);
        let mut lot = Self {
            width,
            depth,
            cols,
            rows,
            base: vec![0; cols * rows],
            order: Vec::new(),
            built: vec![0; cols * rows],
            indoor: vec![0; cols * rows],
            h_edges: vec![Edge::Open],
            v_edges: vec![Edge::Open],
            diags: vec![None],
            rooms: vec![OUTDOORS],
            halves: vec![[OUTDOORS; 2]],
            looks: BTreeMap::new(),
            floors: BTreeMap::new(),
            room_meta: Vec::new(),
            pending: None,
            changes: RoomChanges {
                full: true,
                ..Default::default()
            },
            log: None,
        };
        lot.ensure(0, 0, width, ground.min(depth));
        lot
    }

    pub fn from_file(file: &LotFile) -> Result<Self, Error> {
        Self::from_file_styled(file, &[])
    }

    /// `from_file`, its fences' styles found among `fence_styles` (the content's fence style ids).
    pub fn from_file_styled(file: &LotFile, fence_styles: &[&str]) -> Result<Self, Error> {
        let (w, d) = (file.width as usize, file.depth as usize);
        if w == 0 || d == 0 || w > MAX_SIDE || d > MAX_SIDE {
            return Err(Error::new(format!(
                "lot size must be within 1..={MAX_SIDE}"
            )));
        }
        // Every storey's rows, and the basement's after them (see `storeys.rs`).
        let ground = d;
        let d = d * (file.storeys.unwrap_or(1).clamp(1, 4) as usize + usize::from(file.basement));
        let mut lot = Self::with_ground(w, d, ground);
        for &[x0, z0, x1, z1] in &file.walls {
            let (x0, z0, x1, z1) = (x0 as usize, z0 as usize, x1 as usize, z1 as usize);
            if z0 == z1 && z0 <= d && x0.max(x1) <= w {
                for x in x0.min(x1)..x0.max(x1) {
                    lot.set_h(x, z0, Edge::Wall);
                }
            } else if x0 == x1 && x0 <= w && z0.max(z1) <= d {
                for z in z0.min(z1)..z0.max(z1) {
                    lot.set_v(x0, z, Edge::Wall);
                }
            } else {
                return Err(Error::new(format!(
                    "wall {x0},{z0}-{x1},{z1} is diagonal or out of bounds"
                )));
            }
        }
        let openings = file.doors.iter().map(|d| (d, Edge::Door, "door"));
        let openings = openings.chain(file.windows.iter().map(|w| (w, Edge::Window, "window")));
        for (door, kind, name) in openings {
            let (x, z) = (door.x as usize, door.z as usize);
            let current = match door.axis {
                Axis::X if x < w && z <= d => lot.h_edge(x, z),
                Axis::Z if x <= w && z < d => lot.v_edge(x, z),
                _ => return Err(Error::new(format!("{name} at {x},{z} is out of bounds"))),
            };
            if current != Edge::Wall {
                return Err(Error::new(format!("{name} at {x},{z} is not on a wall")));
            }
            match door.axis {
                Axis::X => lot.set_h(x, z, kind),
                Axis::Z => lot.set_v(x, z, kind),
            }
        }
        for d in &file.diagonals {
            let (x, z) = (d.x as usize, d.z as usize);
            if x >= w || z >= lot.depth {
                return Err(Error::new(format!("diagonal at {x},{z} is out of bounds")));
            }
            let edge = match d.kind.as_deref().unwrap_or("wall") {
                "wall" => Edge::Wall,
                "door" => Edge::Door,
                "window" => Edge::Window,
                other => return Err(Error::new(format!("unknown diagonal kind '{other}'"))),
            };
            lot.set_diag(x, z, Some(Diagonal { dir: d.dir, edge }));
        }
        // Fences on the ground storey, never on a wall (they make no rooms).
        for f in &file.fences {
            let [x0, z0, x1, z1] = f.line.map(usize::from);
            let edge = if f.gate { Edge::Gate } else { Edge::Fence };
            let style = f
                .style
                .as_deref()
                .and_then(|id| fence_styles.iter().position(|s| *s == id))
                .unwrap_or(0) as u8;
            let edges: Vec<EdgeRef> = if z0 == z1 && z0 <= ground && x0.max(x1) <= w {
                (x0.min(x1)..x0.max(x1))
                    .map(|x| EdgeRef::H(x as u16, z0 as u16))
                    .collect()
            } else if x0 == x1 && x0 <= w && z0.max(z1) <= ground {
                (z0.min(z1)..z0.max(z1))
                    .map(|z| EdgeRef::V(x0 as u16, z as u16))
                    .collect()
            } else {
                return Err(Error::new(format!(
                    "fence {x0},{z0}-{x1},{z1} is diagonal or off the ground"
                )));
            };
            for e in edges {
                let (current, x, z) = match e {
                    EdgeRef::H(x, z) => (lot.h_edge(x as usize, z as usize), x, z),
                    EdgeRef::V(x, z) => (lot.v_edge(x as usize, z as usize), x, z),
                    EdgeRef::Diag(..) => unreachable!(),
                };
                if current.is_wall() {
                    return Err(Error::new(format!("fence at {x},{z} is on a wall")));
                }
                match e {
                    EdgeRef::H(..) => lot.set_h(x as usize, z as usize, edge),
                    _ => lot.set_v(x as usize, z as usize, edge),
                }
                lot.set_look(
                    e,
                    EdgeLook {
                        style,
                        ..EdgeLook::default()
                    },
                );
            }
        }
        lot.compute_rooms();
        Ok(lot)
    }

    /// Rebuilds a lot from saved edge arrays (row-major, as `h_edge` / `v_edge` number them:
    /// `width × (depth + 1)` and `(width + 1) × depth`) and diagonals (row-major per tile; empty
    /// for saves made before diagonal walls existed). The first `ground` rows are always there.
    pub fn from_edges(
        width: usize,
        depth: usize,
        ground: usize,
        h: Vec<Edge>,
        v: Vec<Edge>,
        diags: Vec<Option<Diagonal>>,
    ) -> Result<Self, Error> {
        // Deeper than a town file may be: the storeys above the ground are rows too.
        if width == 0 || depth == 0 || width > MAX_SIDE || depth > MAX_ROWS {
            return Err(Error::new("saved lot size is out of range"));
        }
        if h.len() != width * (depth + 1) || v.len() != (width + 1) * depth {
            return Err(Error::new("saved lot edges have the wrong size"));
        }
        if !diags.is_empty() && diags.len() != width * depth {
            return Err(Error::new("saved diagonal walls have the wrong size"));
        }
        if diags.iter().flatten().any(|d| d.edge == Edge::Open) {
            return Err(Error::new("corrupt diagonal walls in save"));
        }
        let mut lot = Self::with_ground(width, depth, ground.clamp(1, depth));
        for (i, &e) in h.iter().enumerate().filter(|(_, e)| **e != Edge::Open) {
            lot.set_h(i % width, i / width, e);
        }
        for (i, &e) in v.iter().enumerate().filter(|(_, e)| **e != Edge::Open) {
            lot.set_v(i % (width + 1), i / (width + 1), e);
        }
        for (i, &d) in diags.iter().enumerate().filter(|(_, d)| d.is_some()) {
            lot.set_diag(i % width, i / width, d);
        }
        lot.compute_rooms();
        Ok(lot)
    }

    /// Adds rows at the far end, all open (room for the storeys above the ground; see
    /// `World::storeys`). Edges and tiles already there keep their places, and nothing is
    /// allocated for the new rows until something is built there.
    pub fn grow_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.rows = depth / CHUNK + 1;
        self.base.resize(self.cols * self.rows, 0);
        self.built.resize(self.cols * self.rows, 0);
        self.indoor.resize(self.cols * self.rows, 0);
        self.depth = depth;
        self.pending = None;
        self.compute_rooms();
    }

    /// The chunk holding tile `(x, z)` (either may be one past the lot, for far edges).
    fn chunk_of(&self, x: usize, z: usize) -> usize {
        (z >> SHIFT) * self.cols + (x >> SHIFT)
    }

    /// The slot of tile `(x, z)` (either may be one past the lot); `VOID` if its chunk isn't
    /// allocated.
    fn slot(&self, x: usize, z: usize) -> usize {
        match self.base[self.chunk_of(x, z)] as usize {
            0 => VOID,
            b => b + ((z & MASK) << SHIFT | (x & MASK)),
        }
    }

    /// Allocates chunk `c` (its slots go on the end, all open and outdoors).
    fn alloc(&mut self, c: usize) {
        if self.base[c] != 0 {
            return;
        }
        self.base[c] = (1 + self.order.len() * CHUNK_TILES) as u32;
        self.order.push(c as u32);
        let n = self.h_edges.len() + CHUNK_TILES;
        self.h_edges.resize(n, Edge::Open);
        self.v_edges.resize(n, Edge::Open);
        self.diags.resize(n, None);
        self.rooms.resize(n, OUTDOORS);
        self.halves.resize(n, [OUTDOORS; 2]);
    }

    /// The slot of tile `(x, z)` to write to, allocating its chunk first.
    fn slot_mut(&mut self, x: usize, z: usize) -> usize {
        let c = self.chunk_of(x, z);
        self.alloc(c);
        let s = self.slot(x, z);
        debug_assert!(s != VOID, "never written: the void slot");
        s
    }

    /// Makes sure the chunks over tiles `[x0, x1) × [z0, z1)` are allocated (clamped to the lot,
    /// far edges included). Returns whether any was new: per-tile arrays elsewhere must then
    /// grow to `tile_slots` (`World::sync_tile_arrays`).
    pub fn ensure(&mut self, x0: usize, z0: usize, x1: usize, z1: usize) -> bool {
        let (x1, z1) = (x1.min(self.width + 1), z1.min(self.depth + 1));
        if x0 >= x1 || z0 >= z1 {
            return false;
        }
        let before = self.order.len();
        for cz in z0 >> SHIFT..=(z1 - 1) >> SHIFT {
            for cx in x0 >> SHIFT..=(x1 - 1) >> SHIFT {
                self.alloc(cz * self.cols + cx);
            }
        }
        self.order.len() != before
    }

    /// `ensure` for one tile (one past the lot either way is fine, for far edges).
    pub fn ensure_tile(&mut self, x: i32, z: i32) -> bool {
        if x < 0 || z < 0 {
            return false;
        }
        self.ensure(x as usize, z as usize, x as usize + 1, z as usize + 1)
    }

    /// The chunk grid: chunks across and down (one tile more than the lot each way).
    pub fn chunk_grid(&self) -> (usize, usize) {
        (self.cols, self.rows)
    }

    /// The chunk (row-major over `chunk_grid`) holding tile `(x, z)`, which must be on the
    /// chunk grid.
    pub fn chunk_at(&self, x: i32, z: i32) -> usize {
        self.chunk_of(x as usize, z as usize)
    }

    /// Whether anything (a wall, door, window, fence, gate or diagonal) is built in chunk `c`.
    pub fn chunk_built(&self, c: usize) -> bool {
        self.built.get(c).is_some_and(|&n| n > 0)
    }

    /// Whether chunk `c` is allocated.
    pub fn chunk_allocated(&self, c: usize) -> bool {
        self.base.get(c).is_some_and(|&b| b != 0)
    }

    /// The tiles `[x0, z0, x1, z1)` chunk `c` covers on the lot (empty past its sides).
    pub fn chunk_rect(&self, c: usize) -> [i32; 4] {
        let (x0, z0) = ((c % self.cols) * CHUNK, (c / self.cols) * CHUNK);
        [
            x0 as i32,
            z0 as i32,
            (x0 + CHUNK).min(self.width) as i32,
            (z0 + CHUNK).min(self.depth) as i32,
        ]
    }

    /// The tiles of chunk `c` on the lot, row by row: `(x, z, slot)` (none if it isn't
    /// allocated).
    pub fn chunk_tiles(&self, c: usize) -> impl Iterator<Item = (i32, i32, usize)> + '_ {
        let b = self.base.get(c).copied().unwrap_or(0) as usize;
        let [x0, z0, x1, z1] = if b == 0 { [0; 4] } else { self.chunk_rect(c) };
        (z0..z1).flat_map(move |z| {
            (x0..x1).map(move |x| {
                (
                    x,
                    z,
                    b + ((z as usize & MASK) << SHIFT | (x as usize & MASK)),
                )
            })
        })
    }

    /// The tiles round the sides of chunk `c` on the lot (none if it isn't allocated).
    pub fn chunk_rim(&self, c: usize) -> impl Iterator<Item = (i32, i32)> + '_ {
        let [x0, z0, x1, z1] = self.chunk_rect(c);
        self.chunk_tiles(c)
            .filter(move |&(x, z, _)| x == x0 || z == z0 || x == x1 - 1 || z == z1 - 1)
            .map(|(x, z, _)| (x, z))
    }

    /// Whether tile `(x, z)`'s chunk is allocated.
    pub fn allocated(&self, x: i32, z: i32) -> bool {
        self.tile_index(x, z) != VOID
    }

    /// How many slots per-tile arrays need (the void slot and every allocated chunk's).
    pub fn tile_slots(&self) -> usize {
        1 + self.order.len() * CHUNK_TILES
    }

    /// The tile a slot holds (`(-1, -1)` for the void slot). It may be past the lot's sides: a
    /// chunk on the edge has slots for tiles off the lot, which stay open and outdoors.
    pub fn tile_xz(&self, slot: usize) -> (i32, i32) {
        if slot == VOID || slot >= self.tile_slots() {
            return (-1, -1);
        }
        let c = self.order[(slot - 1) / CHUNK_TILES] as usize;
        let local = (slot - 1) % CHUNK_TILES;
        (
            ((c % self.cols) * CHUNK + (local & MASK)) as i32,
            ((c / self.cols) * CHUNK + (local >> SHIFT)) as i32,
        )
    }

    /// Every tile of the chunks with a room in them, row by row: `(x, z, slot)` (outdoor tiles
    /// of those chunks too). Whole-lot passes over rooms cost the houses, not the land.
    pub fn indoor_tiles(&self) -> impl Iterator<Item = (i32, i32, usize)> + '_ {
        (0..self.depth).flat_map(move |z| {
            let first = (z >> SHIFT) * self.cols;
            (0..self.cols)
                .filter(move |&cx| self.indoor[first + cx] > 0)
                .flat_map(move |cx| {
                    let b = self.base[first + cx] as usize + ((z & MASK) << SHIFT);
                    (cx * CHUNK..(cx * CHUNK + CHUNK).min(self.width))
                        .map(move |x| (x as i32, z as i32, b + (x & MASK)))
                })
        })
    }

    /// Every allocated tile on the lot, row by row: `(x, z, slot)`.
    pub fn tiles(&self) -> impl Iterator<Item = (i32, i32, usize)> + '_ {
        (0..self.depth).flat_map(move |z| self.row(z, self.width))
    }

    /// The allocated tiles of row `z` with `x < width`, left to right: `(x, z, slot)`.
    fn row(&self, z: usize, width: usize) -> impl Iterator<Item = (i32, i32, usize)> + '_ {
        let first = (z >> SHIFT) * self.cols;
        (0..self.cols)
            .filter_map(move |cx| match self.base[first + cx] as usize {
                0 => None,
                b => Some((cx, b + ((z & MASK) << SHIFT))),
            })
            .flat_map(move |(cx, b)| {
                (cx * CHUNK..(cx * CHUNK + CHUNK).min(width))
                    .map(move |x| (x as i32, z as i32, b + (x & MASK)))
            })
    }

    /// Row `z`'s slots with `x < width` in chunks with something built, left to right.
    fn built_row(&self, z: usize, width: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        let first = (z >> SHIFT) * self.cols;
        (0..self.cols)
            .filter(move |&cx| self.built[first + cx] > 0)
            .flat_map(move |cx| {
                let b = self.base[first + cx] as usize + ((z & MASK) << SHIFT);
                (cx * CHUNK..(cx * CHUNK + CHUNK).min(width)).map(move |x| (x, b + (x & MASK)))
            })
    }

    /// Every wall, door, window, fence and gate, row by row: the horizontal edges first (`z` to
    /// `depth`), then the vertical ones (`x` to `width`), then the diagonals. Chunks with
    /// nothing built are skipped, so this costs what's built, not the lot's size.
    pub fn walls(&self) -> impl Iterator<Item = (EdgeRef, Edge)> + '_ {
        let h = (0..=self.depth).flat_map(move |z| {
            self.built_row(z, self.width).filter_map(move |(x, s)| {
                let e = self.h_edges[s];
                (e != Edge::Open).then_some((EdgeRef::H(x as u16, z as u16), e))
            })
        });
        let v = (0..self.depth).flat_map(move |z| {
            self.built_row(z, self.width + 1).filter_map(move |(x, s)| {
                let e = self.v_edges[s];
                (e != Edge::Open).then_some((EdgeRef::V(x as u16, z as u16), e))
            })
        });
        h.chain(v).chain(
            self.diagonal_walls()
                .map(|(x, z, d)| (EdgeRef::Diag(x as u16, z as u16), d.edge)),
        )
    }

    /// The walls, doors, windows, fences, gates and diagonals stored on tiles `[x0, z0, x1, z1)`
    /// (a tile holds the edges on its `-z` and `-x` sides; far edges are on the tiles one past the
    /// lot), chunk by chunk and row by row in each: what `structure_rect` sends.
    pub fn walls_in(&self, [x0, z0, x1, z1]: [i32; 4]) -> Vec<(EdgeRef, Edge)> {
        let (x0, z0) = (x0.max(0) as usize, z0.max(0) as usize);
        let (x1, z1) = (
            (x1.max(0) as usize).min(self.width + 1),
            (z1.max(0) as usize).min(self.depth + 1),
        );
        let mut out = Vec::new();
        if x0 >= x1 || z0 >= z1 {
            return out;
        }
        for cz in z0 >> SHIFT..=(z1 - 1) >> SHIFT {
            for cx in x0 >> SHIFT..=(x1 - 1) >> SHIFT {
                let c = cz * self.cols + cx;
                if self.built[c] == 0 {
                    continue;
                }
                let b = self.base[c] as usize;
                for z in (cz * CHUNK).max(z0)..(cz * CHUNK + CHUNK).min(z1) {
                    for x in (cx * CHUNK).max(x0)..(cx * CHUNK + CHUNK).min(x1) {
                        let s = b + ((z & MASK) << SHIFT | (x & MASK));
                        let (xu, zu) = (x as u16, z as u16);
                        if x < self.width && self.h_edges[s] != Edge::Open {
                            out.push((EdgeRef::H(xu, zu), self.h_edges[s]));
                        }
                        if z < self.depth && self.v_edges[s] != Edge::Open {
                            out.push((EdgeRef::V(xu, zu), self.v_edges[s]));
                        }
                        if let Some(d) = self.diags[s].filter(|_| x < self.width && z < self.depth)
                        {
                            out.push((EdgeRef::Diag(xu, zu), d.edge));
                        }
                    }
                }
            }
        }
        out
    }

    /// Every diagonal wall, row by row: `(x, z, diagonal)`.
    pub fn diagonal_walls(&self) -> impl Iterator<Item = (usize, usize, Diagonal)> + '_ {
        (0..self.depth).flat_map(move |z| {
            self.built_row(z, self.width)
                .filter_map(move |(x, s)| self.diags[s].map(|d| (x, z, d)))
        })
    }

    /// Horizontal and vertical edges per slot (each tile's `-z` and `-x` side), for counting.
    pub fn edges(&self) -> (&[Edge], &[Edge]) {
        (&self.h_edges, &self.v_edges)
    }

    /// Diagonal wall per slot (see `tile_index`).
    pub fn diagonals(&self) -> &[Option<Diagonal>] {
        &self.diags
    }

    /// Every edge and diagonal row by row, as `from_edges` takes them (for saving): the
    /// horizontal edges `width × (depth + 1)`, the vertical ones `(width + 1) × depth`, the
    /// diagonals `width × depth`.
    pub fn row_major_edges(&self) -> (Vec<Edge>, Vec<Edge>, Vec<Option<Diagonal>>) {
        let (w, d) = (self.width, self.depth);
        let mut h = vec![Edge::Open; w * (d + 1)];
        let mut v = vec![Edge::Open; (w + 1) * d];
        let mut diags = vec![None; w * d];
        for (e, edge) in self.walls() {
            match e {
                EdgeRef::H(x, z) => h[z as usize * w + x as usize] = edge,
                EdgeRef::V(x, z) => v[z as usize * (w + 1) + x as usize] = edge,
                EdgeRef::Diag(x, z) => {
                    diags[z as usize * w + x as usize] = self.diags[self.slot(x.into(), z.into())]
                }
            }
        }
        (h, v, diags)
    }

    /// The diagonal wall across tile `(x, z)`, if any (none off the lot).
    pub fn diag(&self, x: i32, z: i32) -> Option<Diagonal> {
        if self.in_bounds(x, z) {
            self.diags[self.tile_index(x, z)]
        } else {
            None
        }
    }

    /// Notes a change between built and open in chunk `c`.
    fn count(&mut self, c: usize, was: bool, is: bool) {
        match (was, is) {
            (false, true) => self.built[c] += 1,
            (true, false) => self.built[c] -= 1,
            _ => {}
        }
    }

    /// Sets or clears (`None`) the diagonal across a tile. Call `compute_rooms` afterwards.
    pub fn set_diag(&mut self, x: usize, z: usize, d: Option<Diagonal>) {
        let d = d.filter(|d| d.edge != Edge::Open);
        let c = self.chunk_of(x, z);
        if d.is_none() && self.base[c] == 0 {
            return;
        }
        let s = self.slot_mut(x, z);
        let old = std::mem::replace(&mut self.diags[s], d);
        self.count(c, old.is_some(), d.is_some());
        if old.map(|d| d.dir) != d.map(|d| d.dir) {
            self.note(EdgeRef::Diag(x as u16, z as u16));
        }
        if old != d {
            self.changed(LotChange::Diag(x as u16, z as u16, old));
        }
    }

    /// Notes an edge whose wall-ness changed (or a diagonal): the rooms round it are found again.
    fn note(&mut self, e: EdgeRef) {
        if let Some(p) = &mut self.pending {
            if p.len() >= PENDING_MAX {
                self.pending = None;
            } else {
                p.push(e);
            }
        }
    }

    /// Something changed: the tiles round it may score differently (`RoomChanges::rect`), and
    /// while the log is on it keeps what was there.
    fn changed(&mut self, change: LotChange) {
        if !self.changes.full {
            let edge = |e: EdgeRef| match e {
                EdgeRef::H(x, z) => [x as i32, z as i32 - 1, x as i32 + 1, z as i32 + 1],
                EdgeRef::V(x, z) => [x as i32 - 1, z as i32, x as i32 + 1, z as i32 + 1],
                EdgeRef::Diag(x, z) => [x as i32, z as i32, x as i32 + 1, z as i32 + 1],
            };
            let at = match change {
                LotChange::H(x, z, _) => edge(EdgeRef::H(x, z)),
                LotChange::V(x, z, _) => edge(EdgeRef::V(x, z)),
                LotChange::Diag(x, z, _) | LotChange::Floor(x, z, _) => edge(EdgeRef::Diag(x, z)),
                LotChange::Look(e, _) => edge(e),
            };
            self.changes.rect = Some(self.changes.rect.map_or(at, |r| join(r, at)));
        }
        if let Some(log) = &mut self.log {
            log.push(change);
        }
    }

    /// Starts keeping what's changed on the lot, as it was before (the log may be on already:
    /// an edit within an edit). `log_end` gives what changed since, `log_rollback` puts it back.
    pub fn log_begin(&mut self) -> LogMark {
        match &self.log {
            Some(log) => LogMark {
                at: log.len(),
                started: false,
            },
            None => {
                self.log = Some(Vec::new());
                LogMark {
                    at: 0,
                    started: true,
                }
            }
        }
    }

    /// What changed since `mark`, as it was before, oldest first; the log goes off if `mark`
    /// turned it on.
    pub fn log_end(&mut self, mark: LogMark) -> Vec<LotChange> {
        if mark.started {
            return self.log.take().unwrap_or_default();
        }
        self.log
            .as_ref()
            .and_then(|log| log.get(mark.at..))
            .map(<[LotChange]>::to_vec)
            .unwrap_or_default()
    }

    /// Puts back what changed since `mark` (call `compute_rooms` afterwards), and says whether
    /// anything had; the log goes off if `mark` turned it on.
    pub fn log_rollback(&mut self, mark: LogMark) -> bool {
        let changes = match &mut self.log {
            Some(log) if mark.at <= log.len() => log.split_off(mark.at),
            _ => Vec::new(),
        };
        let log = self.log.take();
        self.put_back(&changes);
        self.log = if mark.started { None } else { log };
        !changes.is_empty()
    }

    /// The edit begun at `mark` stands and nobody keeps it: the log goes off if `mark` turned
    /// it on.
    pub fn log_forget(&mut self, mark: LogMark) {
        if mark.started {
            self.log = None;
        }
    }

    /// Stops the log (time moves on: edits can no longer be taken back).
    pub fn log_off(&mut self) {
        self.log = None;
    }

    /// Puts back changes as `log_end` gave them, newest first (call `compute_rooms` afterwards).
    /// With the log on, it keeps what they replace: the changes that make them again.
    pub fn put_back(&mut self, changes: &[LotChange]) {
        for c in changes.iter().rev() {
            match *c {
                LotChange::H(x, z, e) => self.set_h(x.into(), z.into(), e),
                LotChange::V(x, z, e) => self.set_v(x.into(), z.into(), e),
                LotChange::Diag(x, z, d) => self.set_diag(x.into(), z.into(), d),
                LotChange::Look(at, look) => self.set_look(at, look),
                LotChange::Floor(x, z, covering) => self.set_floor(x, z, covering),
            }
        }
    }

    /// How the rooms changed since this was last asked (see `RoomChanges`).
    pub fn take_room_changes(&mut self) -> RoomChanges {
        std::mem::take(&mut self.changes)
    }

    /// The tiles `[x0, z0, x1, z1)` round room `id` (stacked rows), if there is one.
    pub fn room_rect(&self, id: u16) -> Option<[i32; 4]> {
        (id as usize)
            .checked_sub(1)
            .and_then(|k| self.room_meta.get(k))
            .map(RoomMeta::rect)
    }

    /// How many rooms the lot has (their ids are `1..=room_count`).
    pub fn room_count(&self) -> usize {
        self.room_meta.len()
    }

    /// Whether a diagonal wall or window stands across the tile: nobody can stand on it.
    pub fn diag_blocks(&self, x: i32, z: i32) -> bool {
        self.diag(x, z).is_some_and(|d| d.edge.blocks())
    }

    /// Whether tile `(x, z)` has a diagonal ending at grid corner `(cx, cz)`.
    pub fn diag_touches(&self, x: i32, z: i32, cx: i32, cz: i32) -> bool {
        self.diag(x, z).is_some_and(|d| d.touches(x, z, cx, cz))
    }

    /// Which half (0 or 1) of tile `(x, z)` touches `side`; always 0 without a diagonal.
    pub fn half_at_side(&self, x: i32, z: i32, side: Side) -> usize {
        let Some(d) = self.diag(x, z) else {
            return 0;
        };
        match (side, d.dir) {
            (Side::NegZ, _) | (Side::PosX, DiagDir::Dp) | (Side::NegX, DiagDir::Dn) => 0,
            _ => 1,
        }
    }

    /// Room of half `half` (0 or 1) of a tile; `OUTDOORS` off the lot.
    pub fn half_room(&self, x: i32, z: i32, half: usize) -> u16 {
        if self.in_bounds(x, z) {
            self.halves[self.tile_index(x, z)][half & 1]
        } else {
            OUTDOORS
        }
    }

    /// Rooms of both halves of every slot (see `tile_index`; equal for tiles without a diagonal).
    pub fn half_rooms(&self) -> &[[u16; 2]] {
        &self.halves
    }

    pub fn h_edge(&self, x: usize, z: usize) -> Edge {
        self.h_edges[self.slot(x, z)]
    }

    pub fn v_edge(&self, x: usize, z: usize) -> Edge {
        self.v_edges[self.slot(x, z)]
    }

    pub fn set_h(&mut self, x: usize, z: usize, e: Edge) {
        let c = self.chunk_of(x, z);
        if e == Edge::Open && self.base[c] == 0 {
            return;
        }
        let s = self.slot_mut(x, z);
        let old = std::mem::replace(&mut self.h_edges[s], e);
        self.count(c, old != Edge::Open, e != Edge::Open);
        if old.is_wall() != e.is_wall() {
            self.note(EdgeRef::H(x as u16, z as u16));
        }
        if old != e {
            self.changed(LotChange::H(x as u16, z as u16, old));
        }
    }

    pub fn set_v(&mut self, x: usize, z: usize, e: Edge) {
        let c = self.chunk_of(x, z);
        if e == Edge::Open && self.base[c] == 0 {
            return;
        }
        let s = self.slot_mut(x, z);
        let old = std::mem::replace(&mut self.v_edges[s], e);
        self.count(c, old != Edge::Open, e != Edge::Open);
        if old.is_wall() != e.is_wall() {
            self.note(EdgeRef::V(x as u16, z as u16));
        }
        if old != e {
            self.changed(LotChange::V(x as u16, z as u16, old));
        }
    }

    /// How the wall at `e` looks (the default look where there's no wall).
    pub fn look(&self, e: EdgeRef) -> EdgeLook {
        self.looks.get(&e).copied().unwrap_or_default()
    }

    /// Sets the look of the wall at `e` (the default look is not stored).
    pub fn set_look(&mut self, e: EdgeRef, look: EdgeLook) {
        let old = self.look(e);
        if old != look {
            self.changed(LotChange::Look(e, old));
        }
        if look == EdgeLook::default() {
            self.looks.remove(&e);
        } else {
            self.looks.insert(e, look);
        }
    }

    /// Every wall that doesn't have the default look, in a stable order (for saving and views).
    pub fn looks(&self) -> impl Iterator<Item = (EdgeRef, EdgeLook)> + '_ {
        self.looks.iter().map(|(e, l)| (*e, *l))
    }

    /// Floor covering of tile `(x, z)` (0: the automatic look).
    pub fn floor(&self, x: u16, z: u16) -> u8 {
        self.floors.get(&(x, z)).copied().unwrap_or(0)
    }

    /// Sets the floor covering of tile `(x, z)` (0, the automatic look, is not stored).
    pub fn set_floor(&mut self, x: u16, z: u16, covering: u8) {
        let old = self.floor(x, z);
        if old != covering {
            self.changed(LotChange::Floor(x, z, old));
        }
        if covering == 0 {
            self.floors.remove(&(x, z));
        } else {
            self.floors.insert((x, z), covering);
        }
    }

    /// Every tile with a floor covering, as `(x, z, covering)` in a stable order.
    pub fn floors(&self) -> impl Iterator<Item = (u16, u16, u8)> + '_ {
        self.floors.iter().map(|(&(x, z), &c)| (x, z, c))
    }

    pub fn in_bounds(&self, x: i32, z: i32) -> bool {
        x >= 0 && z >= 0 && (x as usize) < self.width && (z as usize) < self.depth
    }

    /// The slot of tile `(x, z)` (see `Lot`): where per-tile arrays keep its values. A tile
    /// whose chunk isn't allocated, or off the chunk grid, has the void slot.
    pub fn tile_index(&self, x: i32, z: i32) -> usize {
        let (x, z) = (x as usize, z as usize);
        let c = (z >> SHIFT)
            .wrapping_mul(self.cols)
            .wrapping_add(x >> SHIFT);
        match self.base.get(c).copied().unwrap_or(0) as usize {
            b if b != 0 && x <= self.width && z <= self.depth => {
                b + ((z & MASK) << SHIFT | (x & MASK))
            }
            _ => VOID,
        }
    }

    /// Edge between two orthogonally adjacent in-bounds tiles.
    pub(crate) fn edge_between(&self, ax: i32, az: i32, bx: i32, bz: i32) -> Edge {
        match (bx - ax, bz - az) {
            (1, 0) => self.v_edge(bx as usize, az as usize),
            (-1, 0) => self.v_edge(ax as usize, az as usize),
            (0, 1) => self.h_edge(ax as usize, bz as usize),
            (0, -1) => self.h_edge(ax as usize, az as usize),
            _ => unreachable!("tiles are not orthogonally adjacent"),
        }
    }

    /// Whether a Sim can walk across the edge between two orthogonally adjacent tiles.
    pub fn edge_walkable(&self, ax: i32, az: i32, bx: i32, bz: i32) -> bool {
        self.in_bounds(bx, bz) && !self.edge_between(ax, az, bx, bz).blocks()
    }

    pub fn room_at(&self, x: i32, z: i32) -> u16 {
        if self.in_bounds(x, z) {
            self.rooms[self.tile_index(x, z)]
        } else {
            OUTDOORS
        }
    }

    /// Room id per slot (see `tile_index`).
    pub fn rooms(&self) -> &[u16] {
        &self.rooms
    }

    /// Room id per tile, row by row (`width × depth`), for views.
    pub fn rooms_row_major(&self) -> Vec<u16> {
        let mut out = vec![OUTDOORS; self.width * self.depth];
        for (x, z, s) in self.tiles() {
            out[z as usize * self.width + x as usize] = self.rooms[s];
        }
        out
    }

    /// Finds the rooms: over the whole lot the first time (`compute_all_rooms`), then only round
    /// the walls built or taken down since (`rooms_near`, the same rooms numbered the same).
    pub fn compute_rooms(&mut self) {
        let done = match self.pending.take() {
            Some(edges) => self.rooms_near(&edges),
            None => false,
        };
        if !done {
            self.compute_all_rooms();
        }
        self.pending = Some(Vec::new());
    }

    /// Flood-fills rooms over tile halves. Walls, doors and windows (straight or diagonal)
    /// separate rooms; any area touching the lot border is outdoors. Rooms are numbered in the
    /// order of their first tile row by row, as if every chunk were there: a chunk that isn't
    /// is one open square, walked across as a whole, and allocated if it turns out to be inside
    /// a room.
    pub fn compute_all_rooms(&mut self) {
        struct Region {
            key: u64,
            nodes: std::ops::Range<usize>,
            voids: std::ops::Range<usize>,
        }
        self.halves.fill([UNSET; 2]);
        let width = self.width;
        let key = |x: i32, z: i32, h: usize| (z as u64 * width as u64 + x as u64) * 2 + h as u64;
        let mut void_seen = vec![false; self.base.len()];
        let mut regions: Vec<Region> = Vec::new();
        let (mut nodes, mut voids) = (Vec::new(), Vec::new());
        let mut queue = VecDeque::new();
        for z in 0..self.depth {
            for cx in 0..self.cols {
                let b = self.base[(z >> SHIFT) * self.cols + cx] as usize;
                if b == 0 {
                    continue;
                }
                for x in cx * CHUNK..(cx * CHUNK + CHUNK).min(width) {
                    let si = b + ((z & MASK) << SHIFT | (x & MASK));
                    for sh in 0..2 {
                        if (sh == 1 && self.diags[si].is_none()) || self.halves[si][sh] != UNSET {
                            continue;
                        }
                        let (n0, v0) = (nodes.len(), voids.len());
                        let mut border = false;
                        let mut first = u64::MAX;
                        self.halves[si][sh] = SEEN;
                        queue.push_back(RoomNode::Tile(si, sh, x as i32, z as i32));
                        while let Some(node) = queue.pop_front() {
                            let mut go = |lot: &mut Lot, x: i32, z: i32, side: Side| {
                                border |= lot.flood_step(x, z, side, &mut queue, &mut void_seen);
                            };
                            match node {
                                RoomNode::Tile(i, h, tx, tz) => {
                                    nodes.push((i, h));
                                    first = first.min(key(tx, tz, h));
                                    for side in Side::ALL {
                                        if self.diags[i].is_none()
                                            || self.half_at_side(tx, tz, side) == h
                                        {
                                            go(self, tx, tz, side);
                                        }
                                    }
                                }
                                RoomNode::Void(c) => {
                                    voids.push(c);
                                    let (x0, z0) =
                                        ((c % self.cols) * CHUNK, (c / self.cols) * CHUNK);
                                    let (x1, z1) =
                                        ((x0 + CHUNK).min(width), (z0 + CHUNK).min(self.depth));
                                    first = first.min(key(x0 as i32, z0 as i32, 0));
                                    for tz in z0 as i32..z1 as i32 {
                                        go(self, x0 as i32, tz, Side::NegX);
                                        go(self, x1 as i32 - 1, tz, Side::PosX);
                                    }
                                    for tx in x0 as i32..x1 as i32 {
                                        go(self, tx, z0 as i32, Side::NegZ);
                                        go(self, tx, z1 as i32 - 1, Side::PosZ);
                                    }
                                }
                            }
                        }
                        if border {
                            for &(i, h) in &nodes[n0..] {
                                self.halves[i][h] = OUTDOORS;
                            }
                            nodes.truncate(n0);
                            voids.truncate(v0);
                        } else {
                            regions.push(Region {
                                key: first,
                                nodes: n0..nodes.len(),
                                voids: v0..voids.len(),
                            });
                        }
                    }
                }
            }
        }
        // Numbered by their first tile, row by row.
        regions.sort_unstable_by_key(|r| r.key);
        for (k, r) in regions.iter().enumerate() {
            let id = k as u16 + 1;
            for &(i, h) in &nodes[r.nodes.clone()] {
                self.halves[i][h] = id;
            }
            // A chunk inside a room has a floor to stand on: it's there from now on.
            for &c in &voids[r.voids.clone()] {
                self.alloc(c);
                let b = self.base[c] as usize;
                let (x0, z0) = ((c % self.cols) * CHUNK, (c / self.cols) * CHUNK);
                for tz in z0..(z0 + CHUNK).min(self.depth) {
                    for tx in x0..(x0 + CHUNK).min(width) {
                        self.halves[b + ((tz & MASK) << SHIFT | (tx & MASK))] = [id, UNSET];
                    }
                }
            }
        }
        self.indoor.fill(0);
        for (i, h) in self.halves.iter_mut().enumerate() {
            if h[0] == UNSET {
                // The void slot, and tiles of edge chunks off the lot.
                *h = [OUTDOORS; 2];
            }
            if self.diags[i].is_none() {
                h[1] = h[0];
            }
            self.rooms[i] = if h[0] != OUTDOORS { h[0] } else { h[1] };
            if self.rooms[i] != OUTDOORS {
                self.indoor[self.order[(i - 1) / CHUNK_TILES] as usize] += 1;
            }
        }
        // Each room's key and the tiles round it, over the chunks with rooms in them.
        let mut rects = vec![[i32::MAX, i32::MAX, i32::MIN, i32::MIN]; regions.len()];
        for c in 0..self.indoor.len() {
            if self.indoor[c] == 0 {
                continue;
            }
            let [x0, z0, x1, z1] = self.chunk_rect(c);
            let b = self.base[c] as usize;
            for z in z0..z1 {
                for x in x0..x1 {
                    let s = b + ((z as usize & MASK) << SHIFT | (x as usize & MASK));
                    for id in self.halves[s] {
                        if id != OUTDOORS {
                            grow(&mut rects[id as usize - 1], x, z);
                        }
                    }
                }
            }
        }
        self.room_meta = regions
            .iter()
            .zip(rects)
            .map(|(r, rect)| RoomMeta::new(r.key, rect))
            .collect();
        self.changes = RoomChanges {
            full: true,
            ..Default::default()
        };
    }

    /// One step of the room flood: from tile `(x, z)` across `side`. Queues the tile there (or
    /// its chunk, if that isn't allocated) unless a wall is in the way or it was seen already;
    /// returns whether the step leaves the lot.
    fn flood_step(
        &mut self,
        x: i32,
        z: i32,
        side: Side,
        queue: &mut VecDeque<RoomNode>,
        void_seen: &mut [bool],
    ) -> bool {
        let (dx, dz) = side.offset();
        let (nx, nz) = (x + dx, z + dz);
        if !self.in_bounds(nx, nz) {
            return true;
        }
        if self.edge_between(x, z, nx, nz).is_wall() {
            return false;
        }
        let nc = self.chunk_of(nx as usize, nz as usize);
        if self.base[nc] == 0 {
            if !std::mem::replace(&mut void_seen[nc], true) {
                queue.push_back(RoomNode::Void(nc));
            }
            return false;
        }
        let n = self.tile_index(nx, nz);
        let nh = self.half_at_side(nx, nz, side.opposite());
        if self.halves[n][nh] == UNSET {
            self.halves[n][nh] = SEEN;
            queue.push_back(RoomNode::Tile(n, nh, nx, nz));
        }
        false
    }
}

/// Room flood markers (`Lot::compute_rooms`): not reached yet, and reached by this flood.
const UNSET: u16 = u16::MAX;
const SEEN: u16 = u16::MAX - 1;

/// A node of the room flood: half `h` of the tile in slot `i` at `(x, z)`, or a whole chunk
/// that isn't allocated (all open).
#[derive(Clone, Copy)]
enum RoomNode {
    Tile(usize, usize, i32, i32),
    Void(usize),
}

mod rooms_near;

/// Grows tiles `r` (`[x0, z0, x1, z1)`) over tile `(x, z)`.
fn grow(r: &mut [i32; 4], x: i32, z: i32) {
    r[0] = r[0].min(x);
    r[1] = r[1].min(z);
    r[2] = r[2].max(x + 1);
    r[3] = r[3].max(z + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed_lot() -> Lot {
        let file: LotFile = serde_json::from_str(
            r#"{"width":10,"depth":10,
                "walls":[[2,2,8,2],[2,8,8,8],[2,2,2,8],[8,2,8,8],[5,2,5,8]],
                "doors":[{"x":3,"z":2,"axis":"x"},{"x":5,"z":4,"axis":"z"}],
                "windows":[{"x":6,"z":2,"axis":"x"},{"x":5,"z":6,"axis":"z"}]}"#,
        )
        .unwrap();
        Lot::from_file(&file).unwrap()
    }

    #[test]
    fn detects_two_rooms() {
        let lot = boxed_lot();
        assert_eq!(lot.room_at(0, 0), OUTDOORS);
        let a = lot.room_at(3, 3);
        let b = lot.room_at(6, 3);
        assert_ne!(a, OUTDOORS);
        assert_ne!(b, OUTDOORS);
        assert_ne!(a, b);
    }

    #[test]
    fn walls_block_and_doors_pass() {
        let lot = boxed_lot();
        assert!(!lot.edge_walkable(4, 1, 4, 2));
        assert!(lot.edge_walkable(3, 1, 3, 2));
        assert!(lot.edge_walkable(4, 4, 5, 4));
    }

    #[test]
    fn windows_block_walking_and_separate_rooms() {
        let lot = boxed_lot();
        assert_eq!(lot.h_edge(6, 2), Edge::Window);
        assert_eq!(lot.v_edge(5, 6), Edge::Window);
        // Outside to inside through the window, and between the rooms through the inner one.
        assert!(!lot.edge_walkable(6, 1, 6, 2));
        assert!(!lot.edge_walkable(4, 6, 5, 6));
        // The rooms stay closed: a window doesn't join them or let the outdoors in.
        assert_ne!(lot.room_at(4, 6), lot.room_at(5, 6));
        assert_ne!(lot.room_at(6, 2), OUTDOORS);
    }

    #[test]
    fn diagonals_split_tiles_into_rooms_by_half() {
        // A diamond with corners (1,3), (3,1), (5,3), (3,5): eight diagonal tiles around four
        // whole tiles. One diagonal has a door, which separates rooms like a wall.
        let file: LotFile = serde_json::from_str(
            r#"{"width":8,"depth":8,"diagonals":[
                {"x":1,"z":2,"dir":"dn"},{"x":2,"z":1,"dir":"dn"},
                {"x":3,"z":1,"dir":"dp"},{"x":4,"z":2,"dir":"dp"},
                {"x":4,"z":3,"dir":"dn","kind":"door"},{"x":3,"z":4,"dir":"dn"},
                {"x":2,"z":4,"dir":"dp"},{"x":1,"z":3,"dir":"dp"}]}"#,
        )
        .unwrap();
        let lot = Lot::from_file(&file).unwrap();
        let inside = lot.room_at(2, 2);
        assert_ne!(inside, OUTDOORS);
        for (x, z) in [(2, 2), (3, 2), (2, 3), (3, 3)] {
            assert_eq!(lot.room_at(x, z), inside, "whole tile {x},{z}");
            assert_eq!(lot.half_room(x, z, 0), lot.half_room(x, z, 1));
        }
        // Each diagonal tile: the half towards the centre is inside, the other outside.
        let centre_half = |x: i32, z: i32| {
            let d = lot.diag(x, z).unwrap();
            // Signed side of the tile centre's offset towards the diamond centre (3, 3).
            let (cx, cz) = (3.0 - (x as f32 + 0.5), 3.0 - (z as f32 + 0.5));
            match d.dir {
                DiagDir::Dp => usize::from(cz > cx),
                DiagDir::Dn => usize::from(cx + cz > 0.0),
            }
        };
        for d in &file.diagonals {
            let (x, z) = (d.x as i32, d.z as i32);
            let h = centre_half(x, z);
            assert_eq!(lot.half_room(x, z, h), inside, "inner half of {x},{z}");
            assert_eq!(
                lot.half_room(x, z, 1 - h),
                OUTDOORS,
                "outer half of {x},{z}"
            );
            assert_eq!(
                lot.room_at(x, z),
                inside,
                "split tiles report the indoor half"
            );
        }
        // Side lookups match the halves.
        assert_eq!(lot.half_at_side(1, 2, Side::NegX), 0);
        assert_eq!(lot.half_at_side(1, 2, Side::PosX), 1);
        assert_eq!(lot.half_at_side(3, 1, Side::PosX), 0);
        assert_eq!(lot.half_at_side(3, 1, Side::NegX), 1);
        assert!(lot.diag_blocks(1, 2));
        assert!(
            !lot.diag_blocks(4, 3),
            "a diagonal door can be walked through"
        );
        assert!(lot.diag_touches(3, 1, 3, 1) && lot.diag_touches(3, 1, 4, 2));
        assert!(!lot.diag_touches(3, 1, 4, 1));
    }

    #[test]
    fn half_open_diagonal_rooms_stay_outdoors() {
        // Three sides of the diamond only: the area leaks out and stays outdoors.
        let file: LotFile = serde_json::from_str(
            r#"{"width":8,"depth":8,"diagonals":[
                {"x":1,"z":2,"dir":"dn"},{"x":2,"z":1,"dir":"dn"},
                {"x":3,"z":1,"dir":"dp"},{"x":4,"z":2,"dir":"dp"},
                {"x":4,"z":3,"dir":"dn"},{"x":3,"z":4,"dir":"dn"}]}"#,
        )
        .unwrap();
        let lot = Lot::from_file(&file).unwrap();
        assert!(lot.half_rooms().iter().all(|h| *h == [OUTDOORS; 2]));
        assert!(lot.rooms().iter().all(|&r| r == OUTDOORS));
    }

    /// The same walls on a lot whose rows past `ground` are chunks allocated as they're built
    /// on, and on one with every chunk there: the same rooms, numbered the same.
    fn both_ways(width: usize, depth: usize, ground: usize, walls: &[[usize; 4]]) -> (Lot, Lot) {
        let mut lazy = Lot::with_ground(width, depth, ground);
        let mut dense = Lot::new(width, depth);
        for lot in [&mut lazy, &mut dense] {
            for &[x0, z0, x1, z1] in walls {
                if z0 == z1 {
                    (x0..x1).for_each(|x| lot.set_h(x, z0, Edge::Wall));
                } else {
                    (z0..z1).for_each(|z| lot.set_v(x0, z, Edge::Wall));
                }
            }
            lot.compute_rooms();
        }
        (lazy, dense)
    }

    #[test]
    fn rooms_over_chunks_not_yet_there_are_found_and_numbered_as_if_they_were() {
        // Upstairs (rows 40..), a hall 40 tiles square: whole chunks inside it have no walls,
        // so they aren't allocated until the rooms are found. A small room beside it starts on
        // a later row than the hall's first tile.
        let walls = [
            [4, 44, 44, 44],
            [4, 84, 44, 84],
            [4, 44, 4, 84],
            [44, 44, 44, 84],
            [50, 46, 54, 46],
            [50, 50, 54, 50],
            [50, 46, 50, 50],
            [54, 46, 54, 50],
        ];
        let (lazy, dense) = both_ways(64, 100, 40, &walls);
        assert_eq!(lazy, dense);
        assert!(lazy.tile_slots() < dense.tile_slots(), "less is stored");
        let hall = lazy.room_at(20, 60);
        assert_ne!(hall, OUTDOORS, "the middle of the hall is a room");
        assert!(lazy.allocated(20, 60), "and is there now");
        assert_eq!(hall, dense.room_at(20, 60));
        assert_eq!(lazy.room_at(52, 48), dense.room_at(52, 48));
        assert_ne!(lazy.room_at(52, 48), hall);
        assert_eq!(lazy.rooms_row_major(), dense.rooms_row_major());
        assert!(!lazy.allocated(60, 90), "open air stays unallocated");
        assert_eq!(lazy.room_at(60, 90), OUTDOORS);
    }

    #[test]
    fn slots_map_back_to_their_tiles() {
        let (lot, _) = both_ways(40, 60, 20, &[[3, 30, 9, 30]]);
        for (x, z, s) in lot.tiles() {
            assert_eq!(lot.tile_index(x, z), s);
            assert_eq!(lot.tile_xz(s), (x, z));
            assert_ne!(s, VOID);
        }
        assert_eq!(lot.tile_index(30, 50), VOID, "nothing built there");
        assert_eq!(lot.tile_index(-1, 3), VOID);
        let rows: Vec<i32> = lot.tiles().map(|(_, z, _)| z).collect();
        assert!(rows.windows(2).all(|w| w[0] <= w[1]), "row by row");
        let walls: Vec<_> = lot.walls().collect();
        assert_eq!(walls.len(), 6);
        assert!(
            walls
                .iter()
                .all(|(e, w)| matches!(e, EdgeRef::H(_, 30)) && *w == Edge::Wall)
        );
        let (h, v, _) = lot.row_major_edges();
        let again = Lot::from_edges(40, 60, 20, h, v, Vec::new()).unwrap();
        assert_eq!(again, lot);
    }

    #[test]
    fn windows_need_a_wall() {
        let file: LotFile = serde_json::from_str(
            r#"{"width":6,"depth":6,"walls":[[1,1,4,1]],"windows":[{"x":4,"z":1,"axis":"x"}]}"#,
        )
        .unwrap();
        let err = Lot::from_file(&file).unwrap_err();
        assert!(err.to_string().contains("window"), "{err}");
    }
}
