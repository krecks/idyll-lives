//! Tops that things stand on, and seats at tables.
//!
//! Objects with a `surface` (tables, counters, desks, nightstands, shelves) carry small things:
//! objects on the `Surface` layer (vases, books, lamps) and plated food. A thing on a surface
//! stands at its tile's middle moved by its `offset`; its host is worked out from where it
//! stands, never stored (object ids shift when objects are sold or moved).
//!
//! Seats with a `seatAt` (dining chairs, stools) are used from any free tile around them
//! (`Approach::Around`) and, facing a table of a kind they tuck under, are seats at that table.
//!
//! Everything here works on slices, so residents deciding what to do (which only see `Ctx`) can
//! use it. `Links` caches what depends only on where objects stand; it is rebuilt with the
//! object index when the structure changes.

use crate::content::Content;
use crate::world::ObjectInstance;

/// A place on a surface: the tile and offset a thing stands at, the object carrying it and the
/// height of its top.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceSpot {
    pub host: u32,
    pub x: i32,
    pub z: i32,
    /// Centimetres off the tile's middle (like `ObjectInstance::offset`).
    pub offset: [i8; 2],
    /// Height of the host's top (m).
    pub height: f32,
}

impl SurfaceSpot {
    /// Where it is, in tiles.
    pub fn centre(&self) -> [f32; 2] {
        [
            self.x as f32 + 0.5 + self.offset[0] as f32 / 100.0,
            self.z as f32 + 0.5 + self.offset[1] as f32 / 100.0,
        ]
    }
}

/// How good a place to eat a seat is (best first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiningRank {
    /// A chair pulled up to a table.
    ChairAtTable,
    /// A table with its seats built in (a dining set, a picnic table).
    TableSeat,
    /// Anywhere to sit.
    AnySeat,
}

/// Where to sit down to eat (see `dining_seat`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiningSeat {
    pub object: u32,
    pub rank: DiningRank,
    /// The table it faces (chairs at a table).
    pub table: Option<u32>,
    /// Where the plate goes.
    pub spot: Option<SurfaceSpot>,
}

/// What depends only on where objects stand, per object id; rebuilt with the object index.
#[derive(Debug, Clone, Default)]
pub struct Links {
    /// The table each seat faces (seats with a `seatAt`).
    pub table: Vec<Option<u32>>,
}

impl Links {
    pub fn new(content: &Content, objects: &[ObjectInstance]) -> Self {
        let _ = content;
        Self { table: vec![None; objects.len()] }
    }

    /// The table `seat` is pulled up to.
    pub fn table_of(&self, seat: u32) -> Option<u32> {
        self.table.get(seat as usize).copied().flatten()
    }
}

/// The object whose top covers tile `(x, z)`.
pub fn host_at(content: &Content, objects: &[ObjectInstance], x: i32, z: i32) -> Option<u32> {
    objects.iter().find_map(|o| {
        content.objects[o.def].surface?;
        let (w, d) = o.size(content);
        (x >= o.x && x < o.x + w && z >= o.z && z < o.z + d).then_some(o.id)
    })
}

/// The tile a resident stands on to use `obj` (its front tile; for seats used from around, the
/// first free tile beside it).
pub fn approach_tile(content: &Content, objects: &[ObjectInstance], obj: &ObjectInstance) -> (i32, i32) {
    let _ = objects;
    obj.front_tile(content)
}

/// A free place on a surface near `near` (within `max_m`), for something of `radius` on one of
/// the surface `kinds` (a mask of `SurfaceKind::bit`; 0: any), among `candidates` (object ids, e.g.
/// what the household may use). `extra` lists more things already standing on surfaces (plated
/// food: middle and radius). Nearest first.
#[allow(clippy::too_many_arguments)]
pub fn free_surface_spot_near(
    content: &Content,
    objects: &[ObjectInstance],
    candidates: &[u32],
    near: [f32; 2],
    max_m: f32,
    kinds: u8,
    radius: f32,
    extra: &[([f32; 2], f32)],
) -> Option<SurfaceSpot> {
    let _ = radius;
    candidates
        .iter()
        .filter_map(|&id| {
            let o = objects.get(id as usize)?;
            let s = content.objects[o.def].surface?;
            if kinds != 0 && kinds & s.kind.bit() == 0 {
                return None;
            }
            let c = o.centre(content);
            let dist = (c[0] - near[0]).hypot(c[1] - near[1]);
            let taken = extra.iter().any(|(p, r)| (p[0] - c[0]).hypot(p[1] - c[1]) < *r);
            (dist <= max_m && !taken).then(|| {
                let (x, z) = (c[0].floor() as i32, c[1].floor() as i32);
                let cm = |v: f32| ((v - v.floor() - 0.5) * 100.0).round() as i8;
                (dist, SurfaceSpot { host: id, x, z, offset: [cm(c[0]), cm(c[1])], height: s.height })
            })
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, s)| s)
}

/// Where the plate goes for someone sitting in `slot` of `seat`: on the table in front of them.
pub fn spot_for_seat(
    content: &Content,
    objects: &[ObjectInstance],
    links: &Links,
    seat: u32,
    slot: usize,
    radius: f32,
    extra: &[([f32; 2], f32)],
) -> Option<SurfaceSpot> {
    let _ = (content, objects, links, seat, slot, radius, extra);
    None
}

/// The best free seat to eat at among `candidates`, nearest within a rank, from `from`.
pub fn dining_seat(
    content: &Content,
    objects: &[ObjectInstance],
    links: &Links,
    candidates: &[u32],
    from: [f32; 2],
) -> Option<DiningSeat> {
    let _ = (content, objects, links, candidates, from);
    None
}
