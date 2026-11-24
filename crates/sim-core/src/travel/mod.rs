//! Getting about a big world: the road network, long trips on foot along it, how long a trip
//! takes, and the open-world fields of the town file.
//!
//! **Which towns.** Every town is laid out for the open world now (`size` is always `big`, cosy
//! and normal were dropped in 0.8.0): it has a network, times commutes by the walk, goes on
//! outings and keeps public things in cells. A town or save without open-world fields uses its
//! meta's roads, or the centre lines of its streets, as the network.
//!
//! **The network.** The town's roads (the town file's top-level `roads`; else `meta.roads`, or
//! the centre lines of `meta.streets`; none: no network) become a small graph: a node at each
//! end of every road, wherever one road's end meets another (within `width / 2 + 1` m of its
//! middle) and wherever two roads for cars cross or touch away from their ends (their middle
//! lines within `CROSS_NEAR`: a junction like any other, `crossings`), and an edge for each
//! stretch of road between two nodes, with its length and kind.
//! Trails are for people on foot only; country roads are where cars go fast, and a motorway (two
//! one-way carriageways) and its ramps are for cars only (`Road::walkable`, `Road::enterable`).
//!
//! **Long walks.** A trip on foot shorter than `ROAD_TRIP_TILES` (or in a town with no network)
//! is found by A* over the tiles as ever. A longer one walks to the road (a short A* leg to a
//! point on one of the nearest stretches of road it can reach), along the roads (A* over the
//! graph, a few hundred nodes at most), and off them to where it's going (another short leg).
//! On the road they keep to the right-hand side (`walk_side`), at a waypoint every few metres
//! and at every bend (`thin`).
//!
//! **ETA.** `Travel::eta` estimates a trip's minutes on foot or by car without searching the
//! tiles: commutes, which way out of town is nearest, and how far a place is for choosing what
//! to do (`ai::falloff`).
//!
//! Nothing here is saved except the town file's fields (`OpenWorld`): the graph is built again on
//! load.

use serde::{Deserialize, Serialize};

/// Trips on foot at least this far (octile distance in tiles) go by road when there's a network.
pub const ROAD_TRIP_TILES: f32 = 96.0;
/// How many of the nearest stretches of road a trip tries to get on at (and off at).
const ENTRIES: usize = 3;
/// How far from a point a stretch of road may be to get on it there (m).
const ENTRY_REACH: f32 = 64.0;
/// How far from a way out of town the road it's the end of is (m): a road's end is at its way out.
const EXIT_ROAD: f32 = 2.0;
/// The longest step between waypoints along a road (m), and how far from the road's middle line
/// a straight step between two of them may stray (m).
const THIN_STEP: f32 = 6.0;
const THIN_TOLERANCE: f32 = 0.3;
/// Walking, a detour factor over the straight line for the tile legs of an estimate.
const DETOUR: f32 = 1.25;
/// Cars: top speeds by kind of road (m a second of drive time, `crate::DRIVE_SCALE` times that a
/// real second at 1×); country roads come from content (`cars.countrySpeed`).
pub const TOWN_SPEED: f32 = 9.0;
pub const COUNTRY_SPEED: f32 = 16.0;
pub const LANE_SPEED: f32 = 8.3;
pub const AVENUE_SPEED: f32 = 13.9;
pub const RAMP_SPEED: f32 = 14.0;
pub const MOTORWAY_SPEED: f32 = 30.0;

/// A car's top speed on a road of `kind` (m a second of drive time): `country` (`Travel::country_speed`) on a
/// country road, `TOWN_SPEED` on a street and on a trail (no car drives one).
pub fn kind_speed(kind: RoadKind, country: f32) -> f32 {
    match kind {
        RoadKind::Street | RoadKind::Trail => TOWN_SPEED,
        RoadKind::Lane => LANE_SPEED,
        RoadKind::Country => country,
        RoadKind::Avenue => AVENUE_SPEED,
        RoadKind::Ramp => RAMP_SPEED,
        RoadKind::Motorway => MOTORWAY_SPEED,
    }
}
/// Game minutes in a minute of `Travel::eta` by car: a car's speeds are drive time (m a second of
/// it), so the eta's minutes are a drive's seconds of drive time over 60, each `DRIVE_SCALE` times
/// shorter than a real second, which is `GAME_MINUTES_PER_SECOND` of the game's minutes
/// (`world::trip_minutes`).
pub const DRIVE_MINUTES: f32 = 60.0 * crate::GAME_MINUTES_PER_SECOND / crate::DRIVE_SCALE;
/// Spatial cells for finding the road near a point (m).
const CELL: f32 = 16.0;

/// What kind of way a road is (`RoadPlan.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoadKind {
    #[default]
    Street,
    Lane,
    /// Between areas: where cars drive fast.
    Country,
    /// Two metres wide, on foot only.
    Trail,
    /// Two lanes each way about a median: a city's main road.
    Avenue,
    /// One of a motorway's two carriageways (one-way, two lanes): cars only, joined only by ramps.
    Motorway,
    /// One lane onto the motorway or off it (one-way): cars only.
    Ramp,
}

impl RoadKind {
    /// Whether it's the motorway's, grade separated: a carriageway or a ramp (`roadKinds.ts`
    /// `walkers: false`). Nobody on foot is on it, no car comes onto it but at its ends (no
    /// driveway, bay or kerb: `Road::enterable`), and it has a level of its own
    /// (`Graph::layer_at`).
    pub fn separated(self) -> bool {
        matches!(self, RoadKind::Motorway | RoadKind::Ramp)
    }

    /// Whether cars go only one way along it, whatever its `OneWay` says (`roadKinds.ts`
    /// `oneway`): a motorway's carriageway, a ramp. Nobody comes the other way there.
    pub fn oneway_always(self) -> bool {
        matches!(self, RoadKind::Motorway | RoadKind::Ramp)
    }

    /// Whether its width takes in sidewalks (`roadKinds.ts` `sidewalk`): a street's, a lane's, an
    /// avenue's. Not a country road (its verge), a trail, nor the motorway (its shoulders): their
    /// kerbs are their edges (`cars::kerb_of`).
    pub fn has_sidewalk(self) -> bool {
        !matches!(
            self,
            RoadKind::Country | RoadKind::Trail | RoadKind::Motorway | RoadKind::Ramp
        )
    }

    /// Whether cars on it speed up and slow down as cars do, each at a speed of its own
    /// (`cars::Car::speed`): an avenue, a motorway and a ramp. Every classic town's roads are none
    /// of them: there each car keeps to its way's timing as it always did.
    pub fn fast(self) -> bool {
        matches!(self, RoadKind::Avenue | RoadKind::Motorway | RoadKind::Ramp)
    }
}

/// Which way cars may go along a road (`RoadPlan.oneway`): both ways, or one way only, along its
/// points (1) or against them (-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "i8", into = "i8")]
pub enum OneWay {
    #[default]
    Both,
    Along,
    Against,
}

impl OneWay {
    pub fn is_both(&self) -> bool {
        *self == OneWay::Both
    }

    /// Whether a car may go along the road the way its points run (`forward`) or against them.
    pub fn may_go(self, forward: bool) -> bool {
        match self {
            OneWay::Both => true,
            OneWay::Along => forward,
            OneWay::Against => !forward,
        }
    }
}

impl TryFrom<i8> for OneWay {
    type Error = String;

    fn try_from(v: i8) -> Result<Self, String> {
        match v {
            0 => Ok(OneWay::Both),
            1 => Ok(OneWay::Along),
            -1 => Ok(OneWay::Against),
            _ => Err(format!("oneway {v}: 1 or -1")),
        }
    }
}

impl From<OneWay> for i8 {
    fn from(o: OneWay) -> i8 {
        match o {
            OneWay::Both => 0,
            OneWay::Along => 1,
            OneWay::Against => -1,
        }
    }
}

/// How someone travels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Foot,
    Car,
}

/// A road as the town file gives it (`RoadPlan`): its middle, a point about every metre (town
/// tiles), and its width with the sidewalks.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoadRaw {
    pub points: Vec<[f32; 2]>,
    pub width: f32,
    #[serde(default)]
    pub kind: RoadKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<u32>,
    /// One-way (`RoadPlan.oneway`); left out when it is two-way, so a town without one-way roads
    /// saves as it did.
    #[serde(default, skip_serializing_if = "OneWay::is_both")]
    pub oneway: OneWay,
}

/// A way out of town (`TownExit`): a road out (older town files may list bus stops too, `Stop`:
/// no longer ways out, dropped as the town loads).
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum ExitRaw {
    Point([f32; 2]),
    Typed {
        x: f32,
        z: f32,
        #[serde(default)]
        kind: ExitKind,
    },
}

impl ExitRaw {
    pub fn point(&self) -> [f32; 2] {
        match *self {
            ExitRaw::Point(p) => p,
            ExitRaw::Typed { x, z, .. } => [x, z],
        }
    }

    pub fn kind(&self) -> ExitKind {
        match *self {
            ExitRaw::Point(_) => ExitKind::Road,
            ExitRaw::Typed { kind, .. } => kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExitKind {
    #[default]
    Road,
    Stop,
}

/// A hamlet (area 0: the main town) or a stretch of countryside (`TownArea`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: AreaKind,
    pub x: f32,
    pub z: f32,
    pub w: f32,
    pub d: f32,
}

impl Area {
    pub fn contains(&self, p: [f32; 2]) -> bool {
        p[0] >= self.x && p[0] < self.x + self.w && p[1] >= self.z && p[1] < self.z + self.d
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AreaKind {
    #[default]
    Hamlet,
    Countryside,
}

/// A marked public parking bay, for car trips between areas (the town file's `parking`): its
/// middle, the way a car parked in it faces (`yaw`, nose in, away from the road: `cars::park_spot`
/// parks at exactly this pose, the one the renderer draws), its area and the car park it is one of
/// (`OpenWorld::car_parks` index; none: a bay on its own by the verge).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Parking {
    pub x: f32,
    pub z: f32,
    /// The way a car parked in it faces (rad, nose in, away from the road).
    pub yaw: f32,
    #[serde(default)]
    pub area: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub park: Option<u32>,
}

/// A car park (the town file's `carParks`): a row of nose-in bays (`Parking::park`) off a straight
/// stretch of street on one apron. Its area, its asphalt (4 corners, counter-clockwise seen from
/// above), where its P sign stands (`x`, `z` and the way it faces), the plot of the place it is by
/// (`serves`), and whether it is the square's. The simulation reads none of it but the bays: it
/// keeps it for the renderer (`view/`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarPark {
    #[serde(default)]
    pub area: u32,
    pub poly: Vec<[f32; 2]>,
    pub sign: [f32; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serves: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub square: bool,
}

/// Run-length masks over the ground grid (`[run, value, run, value, ...]`, row-major from tile
/// (0, 0)): water, and other ground nobody walks on.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Terrain {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked: Option<Vec<u32>>,
}

impl Terrain {
    /// The tiles of a `width × depth` ground grid either mask sets (row-major).
    pub fn mask(&self, width: usize, depth: usize) -> Vec<bool> {
        let mut out = vec![false; width * depth];
        for rle in [&self.water, &self.blocked].into_iter().flatten() {
            let mut i = 0usize;
            for pair in rle.chunks(2) {
                let run = pair[0] as usize;
                let set = pair.get(1).is_some_and(|&v| v != 0);
                let end = (i + run).min(out.len());
                if set {
                    out[i..end].iter_mut().for_each(|t| *t = true);
                }
                i = end;
            }
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.water.is_none() && self.blocked.is_none()
    }
}

/// What a plot is (`plots[].kind`): a lot on the ground, or a flat (`Flat`, a plot of its own inside
/// a building's lot: `world::Plot::of`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlotKind {
    Home,
    Farm,
    Public,
    Shop,
    Countryside,
    /// A building of flats: its hall, stairs, yard and front strip, nobody's home.
    Flats,
    /// A flat: one storey of its building's lot.
    Flat,
}

/// The open-world fields of a town file, all optional, read along
/// with the rest of it (`LotFile::open_world`) and saved as they came (`SaveFile::open_world`).
/// Typed exits, `plots[].area/kind` and `drives[].road/s` are read where those are.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenWorld {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roads: Vec<RoadRaw>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub areas: Vec<Area>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<Terrain>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parking: Vec<Parking>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub car_parks: Vec<CarPark>,
    /// Saves only: each plot's area and kind (`plots[].area/kind`), by plot index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plot_areas: Vec<Option<u32>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plot_kinds: Vec<Option<PlotKind>>,
}

impl OpenWorld {
    pub fn is_empty(&self) -> bool {
        self.size.is_none()
            && self.roads.is_empty()
            && self.areas.is_empty()
            && self.terrain.as_ref().is_none_or(Terrain::is_empty)
            && self.parking.is_empty()
            && self.car_parks.is_empty()
            && self.plot_areas.iter().all(Option::is_none)
            && self.plot_kinds.iter().all(Option::is_none)
    }
}

mod graph;
mod roads;
mod state;
mod walk;

pub use graph::*;
pub use roads::*;
pub use state::*;
pub use walk::*;

#[cfg(test)]
mod tests;
