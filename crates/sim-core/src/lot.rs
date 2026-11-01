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
    /// Where Sims leave town (for work) and come back: `[x, z]` (a road out) or `{ x, z, kind }`
    /// (`road` or `stop`, a bus stop for people on foot). Optional.
    #[serde(default)]
    pub exits: Vec<TownExit>,
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
    /// The open-world fields (docs/design/open-world.md; kept in `TownPlan`). How big the world
    /// is (absent: `cosy`).
    #[serde(default)]
    pub size: Option<WorldSize>,
    /// The roads, for the sim (the same objects as `meta.roads`).
    #[serde(default)]
    pub roads: Vec<RoadPlan>,
    /// The hamlets (area 0: the main town) and stretches of countryside.
    #[serde(default)]
    pub areas: Vec<TownArea>,
    /// Water and blocked land, as run-length masks over the ground grid.
    #[serde(default)]
    pub terrain: Option<Terrain>,
    /// Public parking for car trips between areas.
    #[serde(default)]
    pub parking: Vec<ParkingSpot>,
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
    /// The area (`LotFile::areas` index) it is in.
    #[serde(default)]
    pub area: Option<u32>,
    /// What it is for.
    #[serde(default)]
    pub kind: Option<PlotKind>,
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

/// How big a world is (docs/design/open-world.md); towns without one are `cosy`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorldSize {
    #[default]
    Cosy,
    Normal,
    Big,
}

/// What kind of way a road is: `street` (the default), `lane`, `country` (between areas) or
/// `trail` (2 m, on foot only).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoadKind {
    #[default]
    Street,
    Lane,
    Country,
    Trail,
}

/// A road of the town (`RoadPlan` in `web/src/core/protocol.ts`): its middle, a point about
/// every metre (town tiles), its width with the sidewalks, its kind and the area it belongs to.
/// What the sim doesn't use of it (the side its lots are on, a roundabout's island) stays in
/// `meta.roads` for the renderer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoadPlan {
    pub points: Vec<[f32; 2]>,
    pub width: f32,
    #[serde(default, skip_serializing_if = "is_default")]
    pub kind: RoadKind,
    /// The area (`TownPlan::areas` index) it belongs to; absent between areas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// What is at each end: `out`, `dead`, `join` or `loop`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ends: Option<[String; 2]>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AreaKind {
    #[default]
    Hamlet,
    Countryside,
}

/// A hamlet (area 0: the main town) or a stretch of countryside.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TownArea {
    pub name: String,
    #[serde(default)]
    pub kind: AreaKind,
    pub x: i32,
    pub z: i32,
    pub w: i32,
    pub d: i32,
}

/// A way out of town: a road out, or a bus stop that counts as one for people on foot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExitKind {
    #[default]
    Road,
    Stop,
}

/// Where people leave town; a town file gives `[x, z]` (a road out) or `{ x, z, kind }`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TownExit {
    pub x: f32,
    pub z: f32,
    pub kind: ExitKind,
}

impl<'de> Deserialize<'de> for TownExit {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Point([f32; 2]),
            Typed {
                x: f32,
                z: f32,
                #[serde(default)]
                kind: ExitKind,
            },
        }
        Ok(match Raw::deserialize(d)? {
            Raw::Point([x, z]) => TownExit {
                x,
                z,
                kind: ExitKind::Road,
            },
            Raw::Typed { x, z, kind } => TownExit { x, z, kind },
        })
    }
}

/// Run-length masks over the ground grid, row by row from tile (0, 0): `[run, value, run,
/// value, ...]` with value 0 or 1. Set tiles are water, or blocked, for walkers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terrain {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked: Option<Vec<u32>>,
}

impl Terrain {
    /// A mask's tiles (`width × depth`, row by row; past its runs, unset). None without one.
    pub fn mask(rle: Option<&[u32]>, width: usize, depth: usize) -> Option<Vec<bool>> {
        let rle = rle?;
        let mut out = vec![false; width * depth];
        let mut at = 0usize;
        for pair in rle.chunks(2) {
            let (run, value) = (pair[0] as usize, pair.get(1).copied().unwrap_or(0));
            let end = (at + run).min(out.len());
            out[at..end].fill(value != 0);
            at = end;
        }
        Some(out)
    }

    /// Where nobody walks: water or blocked land (`width × depth`); None without either mask.
    pub fn unwalkable(&self, width: usize, depth: usize) -> Option<Vec<bool>> {
        let water = Self::mask(self.water.as_deref(), width, depth);
        let blocked = Self::mask(self.blocked.as_deref(), width, depth);
        match (water, blocked) {
            (None, None) => None,
            (Some(m), None) | (None, Some(m)) => Some(m),
            (Some(a), Some(b)) => Some(a.iter().zip(&b).map(|(a, b)| *a || *b).collect()),
        }
    }
}

/// A public parking spot for car trips between areas.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ParkingSpot {
    pub x: f32,
    pub z: f32,
    #[serde(default)]
    pub rot: f32,
    #[serde(default)]
    pub area: u32,
}

/// What a plot is for (`plots[].kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlotKind {
    Home,
    Farm,
    Public,
    Shop,
    Countryside,
}

/// The road a driveway meets and how far along its line (m), from `drives[].road` / `.s`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct DriveRoad {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub road: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s: Option<f32>,
}

/// The open-world fields of a town (docs/design/open-world.md), as the sim keeps them: read
/// from the town file, saved with the game, and made up for towns and saves from before them
/// (`TownPlan::fill_in`). How travel uses them is `travel.rs`'s; the terrain masks are kept as
/// given (`Terrain::unwalkable`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TownPlan {
    #[serde(default, skip_serializing_if = "is_default")]
    pub size: WorldSize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roads: Vec<RoadPlan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub areas: Vec<TownArea>,
    /// Every way out, with its kind (`World::exits` has their places).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exits: Vec<TownExit>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub terrain: Terrain,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parking: Vec<ParkingSpot>,
    /// Per plot (by id): its area and kind, if the file gave them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plot_areas: Vec<Option<u32>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plot_kinds: Vec<Option<PlotKind>>,
    /// Per driveway (as `World::drives`): the road it meets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drive_roads: Vec<DriveRoad>,
}

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

impl TownPlan {
    /// The fields a town file gives (the rest are made up by `fill_in`).
    pub fn from_file(file: &LotFile) -> Self {
        let any = |f: &dyn Fn(&PlotRaw) -> bool| file.plots.iter().any(f);
        TownPlan {
            size: file.size.unwrap_or_default(),
            roads: file.roads.clone(),
            areas: file.areas.clone(),
            exits: file.exits.clone(),
            terrain: file.terrain.clone().unwrap_or_default(),
            parking: file.parking.clone(),
            plot_areas: if any(&|p| p.area.is_some()) {
                file.plots.iter().map(|p| p.area).collect()
            } else {
                Vec::new()
            },
            plot_kinds: if any(&|p| p.kind.is_some()) {
                file.plots.iter().map(|p| p.kind).collect()
            } else {
                Vec::new()
            },
            drive_roads: Vec::new(),
        }
    }

    /// Fills in what a town or save from before these fields lacks, from what it has: the roads
    /// from `meta.roads` (or the straight `meta.streets` of older towns, by their middle lines),
    /// one hamlet over the whole town (`ground` rows deep) and the exits as roads out. The size
    /// stays as given: a town or save without one is `cosy` (and plays as it always did).
    pub fn fill_in(
        &mut self,
        meta: &serde_json::Value,
        exits: &[[f32; 2]],
        width: usize,
        ground: usize,
    ) {
        if self.roads.is_empty() {
            if let Some(roads) = meta.get("roads").and_then(|r| r.as_array()) {
                self.roads = roads
                    .iter()
                    .filter_map(|r| serde_json::from_value(r.clone()).ok())
                    .collect();
            } else if let Some(streets) = meta.get("streets").and_then(|r| r.as_array()) {
                let num = |v: &serde_json::Value, k: &str| v.get(k).and_then(|n| n.as_f64());
                self.roads = streets
                    .iter()
                    .filter_map(|s| {
                        let (x, z, w, d) = (num(s, "x")?, num(s, "z")?, num(s, "w")?, num(s, "d")?);
                        let mid = (z + d / 2.0) as f32;
                        Some(RoadPlan {
                            points: vec![[x as f32, mid], [(x + w) as f32, mid]],
                            width: d as f32,
                            kind: RoadKind::Street,
                            area: Some(0),
                            name: None,
                            ends: None,
                        })
                    })
                    .collect();
            }
        }
        if self.areas.is_empty() {
            let name = meta
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("Town")
                .to_string();
            self.areas.push(TownArea {
                name,
                kind: AreaKind::Hamlet,
                x: 0,
                z: 0,
                w: width as i32,
                d: ground as i32,
            });
        }
        if self.exits.len() != exits.len() {
            self.exits = exits
                .iter()
                .map(|&[x, z]| TownExit {
                    x,
                    z,
                    kind: ExitKind::Road,
                })
                .collect();
        }
    }
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
}

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
        };
        lot.ensure(0, 0, width, ground.min(depth));
        lot
    }

    pub fn from_file(file: &LotFile) -> Result<Self, Error> {
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
        let was = std::mem::replace(&mut self.diags[s], d).is_some();
        self.count(c, was, d.is_some());
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
        let was = std::mem::replace(&mut self.h_edges[s], e) != Edge::Open;
        self.count(c, was, e != Edge::Open);
    }

    pub fn set_v(&mut self, x: usize, z: usize, e: Edge) {
        let c = self.chunk_of(x, z);
        if e == Edge::Open && self.base[c] == 0 {
            return;
        }
        let s = self.slot_mut(x, z);
        let was = std::mem::replace(&mut self.v_edges[s], e) != Edge::Open;
        self.count(c, was, e != Edge::Open);
    }

    /// How the wall at `e` looks (the default look where there's no wall).
    pub fn look(&self, e: EdgeRef) -> EdgeLook {
        self.looks.get(&e).copied().unwrap_or_default()
    }

    /// Sets the look of the wall at `e` (the default look is not stored).
    pub fn set_look(&mut self, e: EdgeRef, look: EdgeLook) {
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

    /// Flood-fills rooms over tile halves. Walls, doors and windows (straight or diagonal)
    /// separate rooms; any area touching the lot border is outdoors. Rooms are numbered in the
    /// order of their first tile row by row, as if every chunk were there: a chunk that isn't
    /// is one open square, walked across as a whole, and allocated if it turns out to be inside
    /// a room.
    pub fn compute_rooms(&mut self) {
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
