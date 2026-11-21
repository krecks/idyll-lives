//! Surface spots, hosts and the things standing on them.

use super::*;

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
        let cm = |v: f32, t: i32| {
            ((v - t as f32 - 0.5) * 100.0)
                .round()
                .clamp(-(MAX_OFFSET as f32), MAX_OFFSET as f32) as i8
        };
        SurfaceSpot {
            host,
            x,
            z,
            offset: [cm(p[0], x), cm(p[1], z)],
            height,
        }
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
    /// The seats facing each table (`table` the other way round), in id order.
    seats: Vec<Vec<u32>>,
}

impl Links {
    pub fn new(content: &Content, objects: &[ObjectInstance], nav: &NavGrid) -> Self {
        let lot = nav.lot;
        // Which top covers each tile (floor objects never share tiles).
        let mut tops: Vec<u32> = Vec::new();
        if objects.iter().any(|o| content.objects[o.def].seat_at != 0) {
            tops = vec![u32::MAX; lot.tile_slots()];
            for o in objects
                .iter()
                .filter(|o| content.objects[o.def].surface.is_some())
            {
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
        let mut seats = vec![Vec::new(); objects.len()];
        for (seat, t) in table.iter().enumerate() {
            if let Some(list) = t.and_then(|t| seats.get_mut(t as usize)) {
                list.push(seat as u32);
            }
        }
        let tops: Vec<Top> = objects.iter().filter_map(|o| Top::of(content, o)).collect();
        let host = hosts_among(content, objects, &tops);
        let base: Vec<(i32, i32)> = objects
            .iter()
            .map(|o| approach_tile(content, nav, o))
            .collect();
        // A thing on a top with no free side: from a seat pulled up to the top, else the top's.
        let approach = objects
            .iter()
            .map(|o| match host[o.id as usize] {
                Some(h) => rider_tiles(content, objects, nav, o, h)
                    .first()
                    .copied()
                    .unwrap_or_else(|| {
                        let seat = seats.get(h as usize).and_then(|s| s.first());
                        base[seat.map_or(h as usize, |&s| s as usize)]
                    }),
                None => base[o.id as usize],
            })
            .collect();
        Self {
            table,
            approach,
            host,
            seats,
        }
    }

    /// The top a thing on the `Surface` layer stands on (as of the last rebuild).
    pub fn host_of(&self, id: u32) -> Option<u32> {
        self.host.get(id as usize).copied().flatten()
    }

    /// The table `seat` is pulled up to.
    pub fn table_of(&self, seat: u32) -> Option<u32> {
        self.table.get(seat as usize).copied().flatten()
    }

    /// The seats pulled up to `table`, in id order.
    pub fn seats_at(&self, table: u32) -> &[u32] {
        self.seats.get(table as usize).map_or(&[], Vec::as_slice)
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
    pub(super) fn room(&self, radius: f32) -> [f32; 2] {
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
