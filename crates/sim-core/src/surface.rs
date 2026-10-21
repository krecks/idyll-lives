//! Tops that things stand on, and seats at tables.
//!
//! Objects with a `surface` (tables, counters, desks, nightstands, shelves) carry small things:
//! objects on the `Surface` layer (vases, books, lamps) and plated food. A thing on a surface
//! stands at its tile's middle moved by its `offset`; its host is worked out from where it
//! stands, never stored (object ids shift when objects are sold or moved). The disc it takes
//! (its `radius`) lies wholly on one host's top, inside the margin kept clear along the edges,
//! and overlaps nothing else standing there.
//!
//! Seats with a `seatAt` (dining chairs, stools) are used from any free tile around them
//! (`Approach::Around`) and, facing a table of a kind they tuck under, are seats at that table.
//!
//! Everything here works on slices, so residents deciding what to do (which only see `Ctx`) can
//! use it. `Links` caches what depends only on where objects and walls stand; it is rebuilt with
//! the object index when the structure changes.

use crate::content::{Approach, Content, DEFAULT_RADIUS, Layer, Pose, StageMask, Surface};
use crate::lot::Edge;
use crate::path::NavGrid;
use crate::world::{MAX_OFFSET, ObjectInstance};
use std::f32::consts::{FRAC_PI_2, PI};

/// Slack (m) when checking that a disc stays on a top: offsets are whole centimetres.
const SLACK: f32 = 0.006;
/// How far in from a table's edge a plate goes in front of a chair (m).
const PLATE_IN: f32 = 0.3;
/// How far in front of the middle of a seat built into a table its plate goes (m).
const PLATE_AHEAD: f32 = 0.2;
/// Spacing (m) of the places tried on a top.
const STEP: f32 = 0.05;
/// How far a plate may go from right in front of its diner when that place is taken (m).
const PLATE_SEARCH: f32 = 0.6;

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

    /// The tile and offset of the point `p` (on `host`'s top, `height` high).
    pub fn at(host: u32, p: [f32; 2], height: f32) -> Self {
        let (x, z) = (p[0].floor() as i32, p[1].floor() as i32);
        let cm = |v: f32, t: i32| ((v - t as f32 - 0.5) * 100.0).round().clamp(-(MAX_OFFSET as f32), MAX_OFFSET as f32) as i8;
        SurfaceSpot { host, x, z, offset: [cm(p[0], x), cm(p[1], z)], height }
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
    /// The table it is at: the one a chair faces, or the seat itself for a table with its seats
    /// built in.
    pub table: Option<u32>,
    /// Where the plate goes (for the seat's first free slot).
    pub spot: Option<SurfaceSpot>,
}

/// What depends only on where objects and walls stand, per object id; rebuilt with the object
/// index.
#[derive(Debug, Clone, Default)]
pub struct Links {
    /// The table each seat faces (seats with a `seatAt`).
    pub table: Vec<Option<u32>>,
    /// The tile each object is used from (see `approach_tile`; a thing on a top, `rider_tiles`).
    pub approach: Vec<(i32, i32)>,
    /// The top each thing on the `Surface` layer stands on (see `host_of`).
    pub host: Vec<Option<u32>>,
}

impl Links {
    pub fn new(content: &Content, objects: &[ObjectInstance], nav: &NavGrid) -> Self {
        let lot = nav.lot;
        // Which top covers each tile (floor objects never share tiles).
        let mut tops: Vec<u32> = Vec::new();
        if objects.iter().any(|o| content.objects[o.def].seat_at != 0) {
            tops = vec![u32::MAX; lot.width * lot.depth];
            for o in objects.iter().filter(|o| content.objects[o.def].surface.is_some()) {
                for (x, z) in o.tiles(content).filter(|&(x, z)| lot.in_bounds(x, z)) {
                    tops[lot.tile_index(x, z)] = o.id;
                }
            }
        }
        let table: Vec<Option<u32>> = objects
            .iter()
            .map(|o| {
                let at = |x: i32, z: i32| {
                    let id = *tops.get(lot.tile_index(x, z))?;
                    (id != u32::MAX).then_some(id)
                };
                faced(content, objects, nav, o, at)
            })
            .collect();
        let host: Vec<Option<u32>> = objects.iter().map(|o| host_of(content, objects, o)).collect();
        let base: Vec<(i32, i32)> = objects.iter().map(|o| approach_tile(content, nav, o)).collect();
        // A thing on a top with no free side: from a seat pulled up to the top, else the top's.
        let approach = objects
            .iter()
            .map(|o| match host[o.id as usize] {
                Some(h) => rider_tiles(content, objects, nav, o, h).first().copied().unwrap_or_else(|| {
                    let seat = table.iter().position(|t| *t == Some(h));
                    base[seat.unwrap_or(h as usize)]
                }),
                None => base[o.id as usize],
            })
            .collect();
        Self { table, approach, host }
    }

    /// The top a thing on the `Surface` layer stands on (as of the last rebuild).
    pub fn host_of(&self, id: u32) -> Option<u32> {
        self.host.get(id as usize).copied().flatten()
    }

    /// The table `seat` is pulled up to.
    pub fn table_of(&self, seat: u32) -> Option<u32> {
        self.table.get(seat as usize).copied().flatten()
    }

    /// The tile object `id` is used from (as of the last rebuild).
    pub fn approach_of(&self, id: u32) -> Option<(i32, i32)> {
        self.approach.get(id as usize).copied()
    }
}

/// The top of an object with a `surface`: a rectangle (its `size`, at its `at`) turned and
/// moved with it.
#[derive(Debug, Clone, Copy)]
pub struct Top {
    pub host: u32,
    /// Its middle (tiles).
    pub centre: [f32; 2],
    /// Half its width and depth (m), before turning.
    pub half: [f32; 2],
    pub surface: Surface,
    cos: f32,
    sin: f32,
}

impl Top {
    pub fn of(content: &Content, o: &ObjectInstance) -> Option<Self> {
        let def = &content.objects[o.def];
        let surface = def.surface?;
        let a = angle(o);
        // Resized, the top grows with the footprint (its margins stay) and rises with the height.
        let grown = [0, 1].map(|i| o.extent[i] as f32 / 100.0 - def.footprint[i] as f32);
        let scale = [0, 1].map(|i| o.extent[i] as f32 / 100.0 / def.footprint[i] as f32);
        let surface = Surface {
            size: [0, 1].map(|i| (surface.size[i] + grown[i]).max(0.1)),
            at: [0, 1].map(|i| surface.at[i] * scale[i]),
            height: (surface.height + o.tall as f32 / 100.0).max(0.05),
            ..surface
        };
        let mut top = Top {
            host: o.id,
            centre: middle(content, o),
            half: surface.size.map(|v| v / 2.0),
            surface,
            cos: a.cos(),
            sin: a.sin(),
        };
        top.centre = top.world(surface.at);
        Some(top)
    }

    /// A point in the top's own frame (x to its right, z to its front).
    pub fn local(&self, p: [f32; 2]) -> [f32; 2] {
        let (dx, dz) = (p[0] - self.centre[0], p[1] - self.centre[1]);
        [dx * self.cos - dz * self.sin, dx * self.sin + dz * self.cos]
    }

    /// A point in the top's own frame, back in the world.
    pub fn world(&self, l: [f32; 2]) -> [f32; 2] {
        [
            self.centre[0] + l[0] * self.cos + l[1] * self.sin,
            self.centre[1] - l[0] * self.sin + l[1] * self.cos,
        ]
    }

    /// How far from its middle (each way, in its own frame) the middle of a disc of `radius`
    /// may stand: inside the margin along its edges. Negative if the disc doesn't fit at all.
    fn room(&self, radius: f32) -> [f32; 2] {
        let m = self.surface.inset + radius;
        [self.half[0] - m, self.half[1] - m]
    }

    /// Whether a disc of `radius` at `p` lies on this top, inside the margin along its edges.
    pub fn holds(&self, p: [f32; 2], radius: f32) -> bool {
        let l = self.local(p);
        let r = self.room(radius);
        l[0].abs() <= r[0] + SLACK && l[1].abs() <= r[1] + SLACK
    }

    /// Whether point `p` is over this top at all.
    pub fn covers(&self, p: [f32; 2]) -> bool {
        let l = self.local(p);
        l[0].abs() < self.half[0] && l[1].abs() < self.half[1]
    }
}

/// Which way an object faces, turn included (radians; see `ObjectInstance::angle`).
fn angle(o: &ObjectInstance) -> f32 {
    o.angle()
}

/// Where an object's middle is, its offset included (tiles).
pub fn middle(content: &Content, o: &ObjectInstance) -> [f32; 2] {
    let c = o.centre(content);
    [c[0] + o.offset[0] as f32 / 100.0, c[1] + o.offset[1] as f32 / 100.0]
}

/// Things standing on surfaces near a top: their middles and radii.
fn clutter_on(content: &Content, objects: &[ObjectInstance], top: &Top, out: &mut Vec<([f32; 2], f32)>) {
    out.clear();
    for o in objects {
        let def = &content.objects[o.def];
        if def.layer != Layer::Surface {
            continue;
        }
        let p = middle(content, o);
        // Near enough to touch something on this top.
        let reach = top.half[0].max(top.half[1]) * std::f32::consts::SQRT_2 + def.radius + 0.5;
        if (p[0] - top.centre[0]).abs() < reach && (p[1] - top.centre[1]).abs() < reach {
            out.push((p, def.radius));
        }
    }
}

fn overlaps(discs: &[([f32; 2], f32)], p: [f32; 2], radius: f32) -> bool {
    discs.iter().any(|(q, r)| (q[0] - p[0]).hypot(q[1] - p[1]) < r + radius - SLACK)
}

/// The object whose top covers tile `(x, z)`.
pub fn host_at(content: &Content, objects: &[ObjectInstance], x: i32, z: i32) -> Option<u32> {
    objects.iter().find_map(|o| {
        content.objects[o.def].surface?;
        let (w, d) = o.size(content);
        (x >= o.x && x < o.x + w && z >= o.z && z < o.z + d).then_some(o.id)
    })
}

/// The object a thing on the `Surface` layer stands on (None for anything else, or a thing
/// left with nothing under it).
pub fn host_of(content: &Content, objects: &[ObjectInstance], obj: &ObjectInstance) -> Option<u32> {
    if content.objects[obj.def].layer != Layer::Surface {
        return None;
    }
    let p = middle(content, obj);
    objects
        .iter()
        .filter(|o| o.id != obj.id)
        .find_map(|o| Top::of(content, o).filter(|t| t.covers(p)).map(|t| t.host))
}

/// Things on the `Surface` layer standing on `host`, in id order.
pub fn riders(content: &Content, objects: &[ObjectInstance], host: u32) -> Vec<u32> {
    let Some(top) = objects.get(host as usize).and_then(|o| Top::of(content, o)) else {
        return Vec::new();
    };
    objects
        .iter()
        .filter(|o| content.objects[o.def].layer == Layer::Surface && top.covers(middle(content, o)))
        .map(|o| o.id)
        .collect()
}

/// Why a thing can't stand somewhere on a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Misfit {
    /// Nothing with a top under it.
    NoTop,
    /// It would hang over a top's edge (or into the margin along it).
    OverEdge,
    /// Something else stands there.
    Crowded,
}

/// Where a thing of `radius` standing with its middle at `p` would stand: the host whose top
/// takes the whole disc, with nothing else standing in it (`extra`: more things on surfaces,
/// like plated food).
pub fn fit_on_top(
    content: &Content,
    objects: &[ObjectInstance],
    p: [f32; 2],
    radius: f32,
    extra: &[([f32; 2], f32)],
) -> Result<u32, Misfit> {
    let mut under = None;
    let mut host = None;
    for o in objects {
        let Some(top) = Top::of(content, o) else { continue };
        if !top.covers(p) {
            continue;
        }
        under.get_or_insert(top.host);
        if top.holds(p, radius) {
            host = Some(top);
            break;
        }
    }
    let Some(top) = host else {
        return Err(if under.is_some() { Misfit::OverEdge } else { Misfit::NoTop });
    };
    let mut discs = Vec::new();
    clutter_on(content, objects, &top, &mut discs);
    if overlaps(&discs, p, radius) || overlaps(extra, p, radius) {
        return Err(Misfit::Crowded);
    }
    Ok(top.host)
}

/// The tile a resident stands on to use `obj`: its front tile (or, should that be taken, the
/// other of the two middle tiles in front of a thing an even number of tiles wide; a seat with
/// neither free, from beside it); for seats used from around, the first free one of the tiles in
/// front of, behind, left and right of it.
pub fn approach_tile(content: &Content, nav: &NavGrid, obj: &ObjectInstance) -> (i32, i32) {
    approach_tiles(content, nav, obj).next().unwrap_or_else(|| obj.front_tile(content))
}

/// The tiles a resident may use `obj` from, best first: its front tile (whether or not it is
/// free; along an even front edge, the other middle tile comes first if the front can't be stood
/// on and it can: a thing two tiles wide is used from either, as which counts as the front swaps
/// when the house it's in is turned round; a seat whose front can't be stood on, from the tiles
/// left and right of it that can first: an armchair with a footstool in front of it is sat in
/// from beside it); for seats used from around, each tile in front of, behind, left and right of its
/// middle that can be stood on, on the same storey with no wall or fence between.
pub fn approach_tiles<'a>(
    content: &Content,
    nav: &'a NavGrid,
    obj: &ObjectInstance,
) -> impl Iterator<Item = (i32, i32)> + 'a {
    let around = content.objects[obj.def].approach == Approach::Around;
    let front = obj.front_tile(content);
    let (w, d) = obj.size(content);
    let (x, z) = (obj.x, obj.z);
    let (cx, cz) = (x + (w - 1) / 2, z + (d - 1) / 2);
    // Beside each side (+z, +x, -z, -x): the tile out there and the one of its own next to it.
    let sides = [
        ((cx, z + d), (cx, z + d - 1)),
        ((x + w, cz), (x + w - 1, cz)),
        ((cx, z - 1), (cx, z)),
        ((x - 1, cz), (x, cz)),
    ];
    let r = (obj.rot % 4) as usize;
    let order = [r, (r + 2) % 4, (r + 3) % 4, (r + 1) % 4];
    let storey = nav.stairs.storey(z);
    let standable = move |(t, inner): ((i32, i32), (i32, i32))| stands_by(nav, storey, t, inner);
    // The other middle tile along an even front edge (the front tile is the lower of the two).
    let twin = match r {
        0 | 2 if w % 2 == 0 => Some(((sides[r].0.0 + 1, sides[r].0.1), (sides[r].1.0 + 1, sides[r].1.1))),
        1 | 3 if d % 2 == 0 => Some(((sides[r].0.0, sides[r].0.1 + 1), (sides[r].1.0, sides[r].1.1 + 1))),
        _ => None,
    };
    let instead = twin.filter(|&t| !around && !standable(sides[r]) && standable(t)).map(|(t, _)| t);
    // (A seat with its front taken, a footstool there: from its left or right first, else still the front.)
    let seat = content.objects[obj.def].interactions.iter().any(|i| i.pose == Pose::Sit);
    let from_side = !around && instead.is_none() && seat && !standable(sides[r]);
    let only_front = (!around).then_some(front);
    let skip = if around { 0 } else if from_side { 2 } else { 4 };
    let beside = order.into_iter().skip(skip).map(move |i| sides[i]).filter_map(move |s| standable(s).then_some(s.0));
    let (front_first, front_last) = if from_side { (None, only_front) } else { (only_front, None) };
    instead.into_iter().chain(front_first).chain(beside).chain(front_last)
}

/// The tiles a resident may use `obj`, a thing standing on `host`'s top, from, best first: at
/// the top's edge by it, from the floor beside the tile it stands over (the top's front first,
/// then its left, right and back), else anywhere along the top's edge, nearest it first. Each
/// can be stood on, on the same storey, with no wall or fence between it and the top. A kettle
/// is used from in front of its counter, a book on a table from beside it. Empty when the top
/// stands with no free side (then it is used from the top's own approach tile, `Links::new`).
pub fn rider_tiles(content: &Content, objects: &[ObjectInstance], nav: &NavGrid, obj: &ObjectInstance, host: u32) -> Vec<(i32, i32)> {
    let Some(h) = objects.get(host as usize) else { return Vec::new() };
    let (w, d) = h.size(content);
    let on = |x: i32, z: i32| x >= h.x && x < h.x + w && z >= h.z && z < h.z + d;
    let p = middle(content, obj);
    let under = (p[0].floor() as i32, p[1].floor() as i32);
    let under = if on(under.0, under.1) { under } else { (under.0.clamp(h.x, h.x + w - 1), under.1.clamp(h.z, h.z + d - 1)) };
    let storey = nav.stairs.storey(h.z);
    let standable = |inner: (i32, i32), t: (i32, i32)| !on(t.0, t.1) && stands_by(nav, storey, t, inner);
    // The way the top faces (+z, +x, -z, -x), then its left, right and back.
    const STEPS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    let r = (h.rot % 4) as usize;
    let mut tiles: Vec<(i32, i32)> = [r, (r + 3) % 4, (r + 1) % 4, (r + 2) % 4]
        .into_iter()
        .map(|i| (under.0 + STEPS[i].0, under.1 + STEPS[i].1))
        .filter(|&t| standable(under, t))
        .collect();
    if tiles.is_empty() {
        let mut edge: Vec<(f32, (i32, i32))> = h
            .tiles(content)
            .flat_map(|inner| STEPS.map(|s| (inner, (inner.0 + s.0, inner.1 + s.1))))
            .filter(|&(inner, t)| standable(inner, t))
            .map(|(_, t)| ((t.0 as f32 + 0.5 - p[0]).hypot(t.1 as f32 + 0.5 - p[1]), t))
            .collect();
        edge.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        edge.dedup_by_key(|e| e.1);
        tiles = edge.into_iter().map(|e| e.1).collect();
    }
    tiles
}

/// Every tile a resident may use `obj` from, best first (`approach_tiles`; for a thing on a
/// top, `rider_tiles`; with the top's sides all taken, the tiles of the seats pulled up to it,
/// as they sit down to use it there, else the top's).
pub fn use_tiles(content: &Content, objects: &[ObjectInstance], nav: &NavGrid, obj: &ObjectInstance) -> Vec<(i32, i32)> {
    match host_of(content, objects, obj) {
        Some(h) => {
            let tiles = rider_tiles(content, objects, nav, obj, h);
            if !tiles.is_empty() {
                return tiles;
            }
            let seats: Vec<(i32, i32)> = objects
                .iter()
                .filter(|s| faced_table(content, objects, nav, s.id) == Some(h))
                .flat_map(|s| approach_tiles(content, nav, s).collect::<Vec<_>>())
                .collect();
            if seats.is_empty() { approach_tiles(content, nav, &objects[h as usize]).collect() } else { seats }
        }
        None => approach_tiles(content, nav, obj).collect(),
    }
}

/// A free seat pulled up to the top thing `object` stands on (`Links::table_of`), for someone
/// in life stage `stage`, the nearest to it first: they sit there to use a laptop on a desk.
pub fn seat_at_top(content: &Content, objects: &[ObjectInstance], links: &Links, object: u32, sim: u32, stage: usize) -> Option<u32> {
    let host = links.host_of(object)?;
    let p = middle(content, objects.get(object as usize)?);
    let rules = &content.cooking;
    objects
        .iter()
        .filter(|o| links.table_of(o.id) == Some(host))
        .filter(|o| {
            let def = &content.objects[o.def];
            !o.broken() && o.can_use(sim, def.slots) && o.sits(def, stage, rules.seat_tags, rules.seat_avoid)
        })
        .map(|o| {
            let c = middle(content, o);
            ((c[0] - p[0]).hypot(c[1] - p[1]), o.id)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, o)| o)
}

/// Whether a resident can stand on tile `t` to use a thing on storey `storey` from it: free, on
/// that storey, not on a stair, and (beside its tile `inner`, on the lot) with no wall or fence
/// between.
fn stands_by(nav: &NavGrid, storey: i32, t: (i32, i32), inner: (i32, i32)) -> bool {
    nav.tile_free(t.0, t.1)
        && nav.lot.in_bounds(inner.0, inner.1)
        && nav.stairs.storey(t.1) == storey
        && nav.stairs.step(t.0, t.1).is_none()
        // (Diagonally beside it, where the place is off a corner: no one edge between.)
        && ((t.0 - inner.0).abs() + (t.1 - inner.1).abs() != 1 || nav.lot.edge_between(inner.0, inner.1, t.0, t.1) == Edge::Open)
}

/// The tile a resident uses `slot` of `obj` from when the slot has its own place (content
/// `places`): the place's own tile when it's off the footprint (standing at a ping-pong table's
/// end), else the first tile off the footprint behind it, against the way its user faces (round
/// to a picnic table's far bench). None without a place, or when that tile can't be stood on
/// (see `approach_tiles`): then they come from the object's own.
pub fn slot_approach(content: &Content, nav: &NavGrid, obj: &ObjectInstance, slot: usize) -> Option<(i32, i32)> {
    content.objects[obj.def].places.get(slot)?;
    let at = obj.slot_position(content, slot);
    let yaw = obj.slot_yaw(content, slot);
    let on = |t: (i32, i32)| obj.tiles(content).any(|f| f == t);
    let storey = nav.stairs.storey(obj.z);
    if !on(tile_of(at)) {
        // Off it: on the place's own tile (two tiles out at the oche), with nothing in the way
        // to the next tile towards the thing.
        let t = tile_of(at);
        let inner = (1..=20).map(|k| tile_of([at[0] + yaw.sin() * 0.1 * k as f32, at[1] + yaw.cos() * 0.1 * k as f32])).find(|&p| p != t)?;
        return stands_by(nav, storey, t, inner).then_some(t);
    }
    // Against the way they face from a place on it (out of it), until the footprint's edge is
    // crossed: `t` off it, `inner` on it; else from beside the place (round the end of a bench).
    let out = |a: f32| {
        let mut last = tile_of(at);
        (1..=40).find_map(|k| {
            let p = tile_of([at[0] - a.sin() * 0.1 * k as f32, at[1] - a.cos() * 0.1 * k as f32]);
            let crossed = !on(p) && on(last);
            let found = crossed.then_some((p, last));
            last = p;
            found
        })
    };
    [yaw, yaw + FRAC_PI_2, yaw - FRAC_PI_2]
        .into_iter()
        .filter_map(out)
        .find(|&(t, inner)| stands_by(nav, storey, t, inner))
        .map(|(t, _)| t)
}

/// The tile `p` is on.
fn tile_of(p: [f32; 2]) -> (i32, i32) {
    (p[0].floor() as i32, p[1].floor() as i32)
}

/// How far (m) towards its front of its slot's middle someone sits on a bed's or a seat's side
/// to get on (and so which row of a bed two tiles long they come to it from).
const SIDE_ROW: f32 = 0.15;

/// The tile someone sitting or lying in `slot` of `obj` comes to it from (and gets up to), when
/// it isn't the object's own (`approach_tiles`): round to its place (`slot_approach`); beside
/// the slot for a thing got onto from the side (`Approach::Side`: a bed beside its sleeper's
/// middle, the slot's own side first, a bunk's lower sleeper from its ladder side first and the
/// upper one from the front, up the ladder; a bike, a chaise), behind it for
/// a seat behind the work (`Approach::Behind`: a desk's chair, the drum kit's throne), else
/// beside it; in front of its own slot on a seat for several (the end of a sofa). Each side
/// nearer `from` (where they are now) first where the slot is in the middle. None when none of
/// those can be stood on.
pub fn seat_approach(content: &Content, nav: &NavGrid, obj: &ObjectInstance, slot: usize, from: [f32; 2]) -> Option<(i32, i32)> {
    let def = &content.objects[obj.def];
    if !def.places.is_empty() {
        return slot_approach(content, nav, obj, slot);
    }
    let [r, f] = obj.slot_local(def, slot);
    let [hw, hd] = obj.footprint.map(|v| v as f32 / 2.0);
    let storey = nav.stairs.storey(obj.z);
    // (A hair off a tile line: the middle of a thing two tiles wide.)
    let r = if (r + hw).fract().abs() < 1e-3 { r - 0.01 } else { r };
    let stand = |out: [f32; 2], inner: [f32; 2]| {
        let (t, i) = (tile_of(obj.local_point(out)), tile_of(obj.local_point(inner)));
        stands_by(nav, storey, t, i).then_some(t)
    };
    let row = (f + SIDE_ROW).clamp(-hd + 0.5, hd - 0.5);
    let side = |s: f32| stand([s * (hw + 0.5), row], [s * (hw - 0.5), row]);
    let to = |s: f32| {
        let p = obj.local_point([s * (hw + 0.5), row]);
        (p[0] - from[0]).hypot(p[1] - from[1])
    };
    let first = if def.bunk {
        1.0
    } else if r.abs() > 0.05 {
        r.signum()
    } else if to(1.0) < to(-1.0) {
        1.0
    } else {
        -1.0
    };
    let beside = || side(first).or_else(|| side(-first));
    match def.approach {
        // (The upper bunk is climbed into from the tile in front, round its foot to the ladder.)
        Approach::Side if def.bunk && slot > 0 => None,
        Approach::Side => beside(),
        Approach::Behind => stand([r, -hd - 0.5], [r, -hd + 0.5]).or_else(beside),
        Approach::Front if def.slots > 1 && !def.bunk => stand([r, hd + 0.5], [r, hd - 0.5]),
        _ => None,
    }
}

/// Where someone using `slot` of `obj` standing stands, when not on the object's own tile in
/// front (`approach_tiles`): the tile, the point and the way they face. At the slot's place off
/// the footprint (`places`: the end of a ping-pong table, the oche); for a thing several use at
/// once standing, or two tiles wide, along its front, each in front of their own slot (two at a
/// double sink, one at each basin) or in front of its middle, facing straight in; a carer at a
/// thing with a baby's slot (the crib), in front of the baby; for a use of the `whole` of it (or
/// one that opens or shuts it), in front of its middle. None when that can't be stood on.
pub fn stand_spot(content: &Content, nav: &NavGrid, obj: &ObjectInstance, slot: usize, whole: bool) -> Option<((i32, i32), [f32; 2], f32)> {
    let def = &content.objects[obj.def];
    if !def.places.is_empty() {
        let at = obj.slot_position(content, slot);
        let on = obj.tiles(content).any(|t| t == tile_of(at));
        return (!on).then(|| slot_approach(content, nav, obj, slot)).flatten().map(|t| (t, at, obj.slot_yaw(content, slot)));
    }
    let [w, d] = obj.footprint;
    if def.approach != Approach::Front || def.stairs || def.bunk || !matches!(def.layer, Layer::Floor | Layer::Wall) || (def.slots < 2 && w % 2 == 1) {
        return None;
    }
    let baby = def.interactions.iter().any(|i| i.baby);
    let r = if whole || def.slots < 2 { 0.0 } else { obj.slot_local(def, if baby { 0 } else { slot })[0] };
    let [hw, hd] = [w as f32 / 2.0, d as f32 / 2.0];
    let nudged = if (r + hw).fract().abs() < 1e-3 { r - 0.01 } else { r };
    let t = tile_of(obj.local_point([nudged, hd + 0.5]));
    let inner = tile_of(obj.local_point([nudged, hd - 0.5]));
    if !stands_by(nav, nav.stairs.storey(obj.z), t, inner) {
        return None;
    }
    // As it's drawn (its offset), on that tile.
    let p = obj.local_point([r, hd + 0.5]);
    let o = [obj.offset[0] as f32 / 100.0, obj.offset[1] as f32 / 100.0];
    let p = [0, 1].map(|k| (p[k] + o[k]).clamp([t.0, t.1][k] as f32, [t.0, t.1][k] as f32 + 1.0));
    Some((t, p, crate::world::wrap_angle(obj.angle() + PI)))
}

/// The table `seat` faces: the top covering its front tile, of a kind it tucks under (its
/// `seatAt`), on the same storey with no wall between (`top_at`: the top over a tile on the lot).
fn faced(
    content: &Content,
    objects: &[ObjectInstance],
    nav: &NavGrid,
    seat: &ObjectInstance,
    top_at: impl Fn(i32, i32) -> Option<u32>,
) -> Option<u32> {
    let kinds = content.objects[seat.def].seat_at;
    if kinds == 0 {
        return None;
    }
    let (fx, fz) = seat.front_tile(content);
    let (bx, bz) = match seat.rot % 4 {
        0 => (fx, fz - 1),
        1 => (fx - 1, fz),
        2 => (fx, fz + 1),
        _ => (fx + 1, fz),
    };
    let lot = nav.lot;
    if !lot.in_bounds(fx, fz) || nav.stairs.storey(fz) != nav.stairs.storey(bz) || lot.edge_between(bx, bz, fx, fz) != Edge::Open {
        return None;
    }
    let host = top_at(fx, fz)?;
    let s = content.objects[objects[host as usize].def].surface?;
    (s.kind.bit() & kinds != 0).then_some(host)
}

/// The table `seat` faces, if it is a seat that tucks under one (see `Links::table`).
pub fn faced_table(content: &Content, objects: &[ObjectInstance], nav: &NavGrid, seat: u32) -> Option<u32> {
    let o = objects.get(seat as usize)?;
    faced(content, objects, nav, o, |x, z| host_at(content, objects, x, z))
}

/// The nearest place to `near` on `top` where a disc of `radius` fits among `discs`, no
/// further than `max_m`, with how far it is.
fn nearest_on(top: &Top, near: [f32; 2], max_m: f32, radius: f32, discs: &[([f32; 2], f32)], extra: &[([f32; 2], f32)]) -> Option<(f32, SurfaceSpot)> {
    let room = top.room(radius);
    if room[0] < 0.0 || room[1] < 0.0 {
        return None;
    }
    let fits = |p: [f32; 2]| -> Option<SurfaceSpot> {
        let spot = SurfaceSpot::at(top.host, p, top.surface.height);
        let c = spot.centre();
        (top.holds(c, radius) && !overlaps(discs, c, radius) && !overlaps(extra, c, radius)).then_some(spot)
    };
    // Right there (as near as the edges allow) if it's free.
    let l = top.local(near);
    let first = top.world([l[0].clamp(-room[0], room[0]), l[1].clamp(-room[1], room[1])]);
    let dist = |p: [f32; 2]| (p[0] - near[0]).hypot(p[1] - near[1]);
    if dist(first) <= max_m
        && let Some(s) = fits(first)
    {
        return Some((dist(s.centre()), s));
    }
    // Else the nearest free point on a grid over the top.
    let (nx, nz) = ((room[0] / STEP).floor() as i32, (room[1] / STEP).floor() as i32);
    let mut best: Option<(f32, SurfaceSpot)> = None;
    for iz in -nz..=nz {
        for ix in -nx..=nx {
            let p = top.world([ix as f32 * STEP, iz as f32 * STEP]);
            let dp = dist(p);
            if dp > max_m || best.is_some_and(|b| dp >= b.0) {
                continue;
            }
            if let Some(s) = fits(p) {
                best = Some((dp, s));
            }
        }
    }
    best
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
    let mut discs = Vec::new();
    let mut best: Option<(f32, SurfaceSpot)> = None;
    for &id in candidates {
        let Some(top) = objects.get(id as usize).and_then(|o| Top::of(content, o)) else { continue };
        if kinds != 0 && kinds & top.surface.kind.bit() == 0 {
            continue;
        }
        // Too far even to its nearest corner.
        let reach = top.half[0].hypot(top.half[1]);
        if (top.centre[0] - near[0]).hypot(top.centre[1] - near[1]) - reach > max_m.min(best.map_or(f32::INFINITY, |b| b.0)) {
            continue;
        }
        clutter_on(content, objects, &top, &mut discs);
        if let Some(found) = nearest_on(&top, near, max_m, radius, &discs, extra)
            && best.is_none_or(|b| found.0 < b.0)
        {
            best = Some(found);
        }
    }
    best.map(|b| b.1)
}

/// The top a seat's diner eats at, and where on it right in front of them (`slot`): a table it
/// is pulled up to, else one it faces (a sofa at a coffee table), else its own top (a dining
/// set with its seats built in).
fn plate_target(content: &Content, objects: &[ObjectInstance], links: &Links, seat: u32, slot: usize) -> Option<(Top, [f32; 2])> {
    let s = objects.get(seat as usize)?;
    let def = &content.objects[s.def];
    let slot = slot.min(def.slots.max(1) as usize - 1);
    let at = s.slot_position(content, slot);
    // (The way they face in it: across the table from a place of their own, `places`.)
    let a = if def.places.is_empty() { angle(s) } else { s.slot_yaw(content, slot) };
    let forward = [a.sin(), a.cos()];
    let ahead = |t: f32| [at[0] + forward[0] * t, at[1] + forward[1] * t];
    if let Some(top) = Top::of(content, s) {
        return Some((top, ahead(PLATE_AHEAD)));
    }
    let (fx, fz) = s.front_tile(content);
    let host = links.table_of(seat).or_else(|| {
        let host = host_at(content, objects, fx, fz)?;
        (content.objects[s.def].interactions.iter().any(|i| i.pose == Pose::Sit)).then_some(host)
    })?;
    let top = Top::of(content, objects.get(host as usize)?)?;
    // Where the line straight ahead of the diner meets the top's edge, then a little way in.
    let (o, f) = (top.local(at), top.local(ahead(1.0)));
    let dir = [f[0] - o[0], f[1] - o[1]];
    let mut enter: f32 = 0.0;
    for k in 0..2 {
        if dir[k].abs() > 1e-4 {
            let (t1, t2) = ((-top.half[k] - o[k]) / dir[k], (top.half[k] - o[k]) / dir[k]);
            enter = enter.max(t1.min(t2));
        }
    }
    Some((top, ahead(enter + PLATE_IN)))
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
    let (top, target) = plate_target(content, objects, links, seat, slot)?;
    let mut discs = Vec::new();
    clutter_on(content, objects, &top, &mut discs);
    nearest_on(&top, target, PLATE_SEARCH, radius, &discs, extra).map(|f| f.1)
}

/// The best free seat to eat at among `candidates` for someone in life stage `stage`, nearest
/// within a rank, from `from`: a chair pulled up to a table, else a table with its seats built
/// in, else anywhere comfortable to sit (not a toilet or a desk chair). Within a rank, a seat
/// made for their age comes first: a child has a snack at the kids' table.
pub fn dining_seat(
    content: &Content,
    objects: &[ObjectInstance],
    links: &Links,
    candidates: &[u32],
    from: [f32; 2],
    stage: usize,
) -> Option<DiningSeat> {
    let tag = |name: &str| content.tags.iter().position(|t| t == name).map_or(0, |i| 1u64 << i);
    let food = tag("food");
    let comfy = food | tag("lounge");
    // (Every stage but a baby's: a seat for fewer is made for them.)
    let anyone: StageMask = content.life.stages.iter().enumerate().filter(|(_, s)| !s.baby).fold(0, |m, (i, _)| m | 1 << i);
    let mut best: Option<(DiningRank, bool, f32, u32, Option<u32>)> = None;
    for &id in candidates {
        let Some(o) = objects.get(id as usize) else { continue };
        let def = &content.objects[o.def];
        if o.broken() || !o.has_free_slot(def.slots) {
            continue;
        }
        let sits = |mask: u64| o.seats_now(def, stage).any(|t| mask == 0 || t & mask != 0);
        let (rank, table) = match links.table_of(id) {
            Some(t) if def.seat_at != 0 && sits(0) => (DiningRank::ChairAtTable, Some(t)),
            _ if def.surface.is_some() && food != 0 && sits(food) => (DiningRank::TableSeat, Some(id)),
            _ if sits(comfy) => (DiningRank::AnySeat, None),
            _ => continue,
        };
        let anyones = !def.interactions.iter().any(|i| i.pose == Pose::Sit && i.for_stage(stage) && i.stages & anyone != anyone);
        let c = o.centre(content);
        let d = (c[0] - from[0]).hypot(c[1] - from[1]);
        if best.is_none_or(|b| (rank, anyones, d) < (b.0, b.1, b.2)) {
            best = Some((rank, anyones, d, id, table));
        }
    }
    let (rank, _, _, object, table) = best?;
    let o = &objects[object as usize];
    let slot = o.users[..content.objects[o.def].slots as usize].iter().position(Option::is_none).unwrap_or(0);
    Some(DiningSeat { object, rank, table, spot: spot_for_seat(content, objects, links, object, slot, DEFAULT_RADIUS, &[]) })
}
