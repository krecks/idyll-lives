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
    /// The tile each object is used from (see `approach_tile`).
    pub approach: Vec<(i32, i32)>,
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
        let table = objects
            .iter()
            .map(|o| {
                let at = |x: i32, z: i32| {
                    let id = *tops.get(lot.tile_index(x, z))?;
                    (id != u32::MAX).then_some(id)
                };
                faced(content, objects, nav, o, at)
            })
            .collect();
        let approach = objects.iter().map(|o| approach_tile(content, nav, o)).collect();
        Self { table, approach }
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
/// other of the two middle tiles in front of a thing an even number of tiles wide); for seats
/// used from around, the first free one of the tiles in front of, behind, left and right of it.
pub fn approach_tile(content: &Content, nav: &NavGrid, obj: &ObjectInstance) -> (i32, i32) {
    approach_tiles(content, nav, obj).next().unwrap_or_else(|| obj.front_tile(content))
}

/// The tiles a resident may use `obj` from, best first: its front tile (whether or not it is
/// free; along an even front edge, the other middle tile comes first if the front can't be stood
/// on and it can: a thing two tiles wide is used from either, as which counts as the front swaps
/// when the house it's in is turned round); for seats used from around, each tile in front of,
/// behind, left and right of its middle that can be stood on, on the same storey with no wall or
/// fence between.
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
    let only_front = (!around).then_some(front);
    let beside = around.then_some(order.map(|i| sides[i])).into_iter().flatten().filter_map(move |s| standable(s).then_some(s.0));
    instead.into_iter().chain(only_front).chain(beside)
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
    let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
    let on = |t: (i32, i32)| obj.tiles(content).any(|f| f == t);
    let off = !on(tile(at));
    // Along the way they face from a place off it (into the object), against it from one on it
    // (out of it), until the footprint's edge is crossed: `t` off it, `inner` on it.
    let step = if off { 0.1 } else { -0.1 };
    let (mut last, mut found) = (tile(at), None);
    for k in 1..=40 {
        let p = tile([at[0] + yaw.sin() * step * k as f32, at[1] + yaw.cos() * step * k as f32]);
        if on(p) != on(last) {
            found = Some(if off { (last, p) } else { (p, last) });
            break;
        }
        last = p;
    }
    let (t, inner) = found?;
    stands_by(nav, nav.stairs.storey(obj.z), t, inner).then_some(t)
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
