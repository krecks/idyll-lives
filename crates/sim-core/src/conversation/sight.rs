//! Sight and earshot: who can see whom, who hears whom, and the tiles between.

use super::*;
/// Whether the edge from tile `a` to its neighbour `b` stands shut to voices and eyes: a wall, a
/// window, or a front door (between the house and outdoors), which stays shut but for someone
/// going through it (as the renderer swings it: `doors.ts`). A doorway inside the house stands
/// open, and fences and gates are talked over.
pub(crate) fn shut(lot: &Lot, (ax, az): (i32, i32), (bx, bz): (i32, i32)) -> bool {
    let side = match (bx - ax, bz - az) {
        (1, 0) => Side::PosX,
        (-1, 0) => Side::NegX,
        (0, 1) => Side::PosZ,
        _ => Side::NegZ,
    };
    // (The halves either side of it, on a tile a diagonal wall splits.)
    let room = |x, z, side| lot.half_room(x, z, lot.half_at_side(x, z, side));
    stops(lot.edge_between(ax, az, bx, bz), || {
        (room(ax, az, side) == OUTDOORS) != (room(bx, bz, side.opposite()) == OUTDOORS)
    })
}

/// Whether an edge of kind `edge` stops voices and eyes (`shut`): a door only if it's a front
/// door (`front`: between the house and outdoors).
fn stops(edge: Edge, front: impl FnOnce() -> bool) -> bool {
    match edge {
        Edge::Wall | Edge::Window => true,
        Edge::Door => front(),
        Edge::Open | Edge::Fence | Edge::Gate => false,
    }
}

/// Whether people at `a` and `b` see each other, as two talking (and those listening in) must:
/// on the same storey, with nothing `shut` on the straight line between them (through a grid
/// corner, open one way round it or the other), nor a diagonal wall, window or front door across
/// it. Furniture doesn't hide anyone. It walks the tiles the line crosses, so it's cheap for
/// people near each other, as it's asked of every tick someone is about to start talking.
pub(crate) fn in_sight(nav: &NavGrid, a: [f32; 2], b: [f32; 2]) -> bool {
    let lot = nav.lot;
    let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
    let (mut at, goal) = (tile(a), tile(b));
    if !lot.in_bounds(at.0, at.1) || nav.stairs.storey(at.1) != nav.stairs.storey(goal.1) {
        return false;
    }
    let open = |from: (i32, i32), to: (i32, i32)| lot.in_bounds(to.0, to.1) && !shut(lot, from, to);
    // Which side of the line from `p` to `q` the point `r` is on (0 on it).
    let side = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    // A diagonal across the tile, shut as `shut` has it, that the line crosses.
    let across = |(x, z): (i32, i32)| {
        let Some(d) = lot.diag(x, z) else {
            return false;
        };
        let shut = stops(d.edge, || {
            (lot.half_room(x, z, 0) == OUTDOORS) != (lot.half_room(x, z, 1) == OUTDOORS)
        });
        let (x, z) = (x as f32, z as f32);
        let (p, q) = match d.dir {
            DiagDir::Dp => ([x, z], [x + 1.0, z + 1.0]),
            DiagDir::Dn => ([x, z + 1.0], [x + 1.0, z]),
        };
        shut && side(p, q, a) * side(p, q, b) <= 0.0 && side(a, b, p) * side(a, b, q) <= 0.0
    };
    let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
    let (sx, sz) = (if dx > 0.0 { 1 } else { -1 }, if dz > 0.0 { 1 } else { -1 });
    // How far along the line (0 at `a`, 1 at `b`) it next crosses a grid line across x, and one
    // across z; and how far on it crosses the one after.
    let first = |p: f32, d: f32| {
        if d > 0.0 {
            (p.floor() + 1.0 - p) / d
        } else if d < 0.0 {
            (p.floor() - p) / d
        } else {
            f32::INFINITY
        }
    };
    let (mut tx, mut tz) = (first(a[0], dx), first(a[1], dz));
    let (step_x, step_z) = (1.0 / dx.abs(), 1.0 / dz.abs());
    for _ in 0..=(goal.0 - at.0).abs() + (goal.1 - at.1).abs() {
        if across(at) {
            return false;
        }
        if at == goal {
            return true;
        }
        let to = if (tx - tz).abs() < 1e-5 {
            let (side_x, side_z, to) =
                ((at.0 + sx, at.1), (at.0, at.1 + sz), (at.0 + sx, at.1 + sz));
            if !(open(at, side_x) && open(side_x, to) || open(at, side_z) && open(side_z, to)) {
                return false;
            }
            tx += step_x;
            tz += step_z;
            to
        } else {
            let to = if tx < tz {
                tx += step_x;
                (at.0 + sx, at.1)
            } else {
                tz += step_z;
                (at.0, at.1 + sz)
            };
            if !open(at, to) {
                return false;
            }
            to
        };
        at = to;
    }
    false
}

/// Whether someone at `b` hears a call from `a`: within `EARSHOT` of open floor (round walls and
/// furniture-free of them: furniture doesn't stop a voice), a doorway counting `DOOR_MUFFLE`
/// more and a flight of stairs `STOREY_WALK`. Walls, windows and the front door (`shut`) stop
/// it; garden fences don't.
///
/// Asked for many pairs a tick in a full town, so cheaply (`earshot_search`): it stops as soon as
/// `b` is reached by a way within earshot, with a hasher for small keys and its map and queue kept
/// from call to call. Asking about many people from one spot, `Earshot` searches once.
pub(crate) fn within_earshot(nav: &NavGrid, a: [f32; 2], b: [f32; 2]) -> bool {
    if nav.stairs.distance(a, b) > EARSHOT {
        return false;
    }
    let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
    let (start, goal) = (tile(a), tile(b));
    if start == goal {
        return true;
    }
    EARSHOT_BEST.with(|best| earshot_search(nav, start, Some(goal), &mut best.borrow_mut()))
}

/// The search for `within_earshot` and `Earshot` from tile `start`, cheapest first, over the
/// tiles' edges as a call carries over them, as far as `EARSHOT`: true as soon as it reaches `goal`
/// (any way within earshot is the answer, the cheapest isn't needed); with no goal, to the end,
/// leaving in `best` every tile within earshot (the cheapest way to it, tenths of a metre).
fn earshot_search(
    nav: &NavGrid,
    start: (i32, i32),
    goal: Option<(i32, i32)>,
    best: &mut EarshotMap,
) -> bool {
    let lot = nav.lot;
    // Tenths of a metre, as whole numbers for the queue.
    let tenths = |m: f32| (m * 10.0).round() as u32;
    let (limit, muffle) = (tenths(EARSHOT), tenths(DOOR_MUFFLE));
    let stairs = tenths(STOREY_WALK);
    let storey = |z: i32| nav.stairs.storey(z);
    // Sound crosses the edge between neighbouring tiles at this extra cost, or not at all.
    let open = |(ax, az): (i32, i32), (bx, bz): (i32, i32)| -> Option<u32> {
        if !lot.in_bounds(bx, bz) || lot.diag_blocks(bx, bz) || storey(az) != storey(bz) {
            return None;
        }
        match lot.edge_between(ax, az, bx, bz) {
            _ if shut(lot, (ax, az), (bx, bz)) => None,
            Edge::Door => Some(muffle),
            _ => Some(0),
        }
    };
    EARSHOT_QUEUE.with(|queue| {
        let mut queue = queue.borrow_mut();
        best.clear();
        queue.clear();
        best.insert(start, 0u32);
        queue.push(std::cmp::Reverse((0u32, start)));
        while let Some(std::cmp::Reverse((cost, at))) = queue.pop() {
            if best.get(&at).is_some_and(|&c| c < cost) {
                continue;
            }
            // A tile reached by a way within earshot: the goal heard there, or searched on from.
            let mut reach = |to: (i32, i32), step: u32| {
                let c = cost + step;
                if c > limit {
                    return false;
                }
                if Some(to) == goal {
                    return true;
                }
                if best.get(&to).is_none_or(|&b| c < b) {
                    best.insert(to, c);
                    queue.push(std::cmp::Reverse((c, to)));
                }
                false
            };
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let to = (at.0 + dx, at.1 + dz);
                if let Some(extra) = open(at, to)
                    && reach(to, 10 + extra)
                {
                    return true;
                }
            }
            // Diagonally across open floor (both ways round the corner open, no doorway).
            for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let (side_x, side_z) = ((at.0 + dx, at.1), (at.0, at.1 + dz));
                let to = (at.0 + dx, at.1 + dz);
                let clear = open(at, side_x) == Some(0)
                    && open(side_x, to) == Some(0)
                    && open(at, side_z) == Some(0)
                    && open(side_z, to) == Some(0);
                if clear && reach(to, 14) {
                    return true;
                }
            }
            // Up or down the stairs.
            if let Some(to) = nav.stairs.link(at.0, at.1)
                && reach(to, stairs)
            {
                return true;
            }
        }
        false
    })
}

/// Who hears a call from one spot (`within_earshot` from it to each of them, the same answers),
/// for asking about many: the search from the spot runs once, to everywhere within earshot, at
/// the first asked about who is near enough as the crow flies, and each after is a look-up.
/// (Someone choosing what to do asks about everyone available: in a full town, dozens.) The lot
/// mustn't change while it's in use.
pub(crate) struct Earshot<'a> {
    nav: &'a NavGrid<'a>,
    from: [f32; 2],
    /// Every tile within earshot of `from`'s (filled at the first need).
    heard: Option<EarshotMap>,
}

impl<'a> Earshot<'a> {
    pub(crate) fn new(nav: &'a NavGrid<'a>, from: [f32; 2]) -> Self {
        Self {
            nav,
            from,
            heard: None,
        }
    }

    /// `within_earshot(nav, from, b)`.
    pub(crate) fn hears(&mut self, b: [f32; 2]) -> bool {
        let nav = self.nav;
        // (Further apart across than earshot: no walk between them is any shorter. Most of a big
        // town is, for each one asking about everyone, and is passed over without the walk.)
        if (self.from[0] - b[0]).abs() > EARSHOT || nav.stairs.distance(self.from, b) > EARSHOT {
            return false;
        }
        let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
        let (start, goal) = (tile(self.from), tile(b));
        if start == goal {
            return true;
        }
        self.heard
            .get_or_insert_with(|| {
                let mut best = EARSHOT_MAPS
                    .with(|m| m.borrow_mut().pop())
                    .unwrap_or_default();
                best.clear();
                earshot_search(nav, start, None, &mut best);
                best
            })
            .contains_key(&goal)
    }
}

impl Drop for Earshot<'_> {
    fn drop(&mut self) {
        // (Its map's room kept for the next.)
        if let Some(map) = self.heard.take() {
            EARSHOT_MAPS.with(|m| m.borrow_mut().push(map));
        }
    }
}

thread_local! {
    /// `earshot_search`'s queue and `within_earshot`'s map, kept from one call to the next
    /// (cleared, their room kept), so a call allocates nothing once the first few have run.
    static EARSHOT_QUEUE: std::cell::RefCell<EarshotQueue> = std::cell::RefCell::default();
    static EARSHOT_BEST: std::cell::RefCell<EarshotMap> = std::cell::RefCell::default();
    /// `Earshot`'s maps, kept for the next one.
    static EARSHOT_MAPS: std::cell::RefCell<Vec<EarshotMap>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The tiles within earshot found so far, each with the cheapest way to it (tenths of a metre).
type EarshotMap = std::collections::HashMap<(i32, i32), u32, TileHash>;
/// The tiles still to search on from, cheapest first.
type EarshotQueue = std::collections::BinaryHeap<std::cmp::Reverse<(u32, (i32, i32))>>;

/// Hashing for maps keyed by tiles: a multiply and a rotate a word (FxHash's), much cheaper than
/// the standard SipHash, which guards against keys chosen to collide (none are here).
pub(crate) type TileHash = std::hash::BuildHasherDefault<TileHasher>;

#[derive(Default, Clone, Copy)]
pub(crate) struct TileHasher(u64);

impl TileHasher {
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
}

impl std::hash::Hasher for TileHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.add(u64::from(b));
        }
    }

    fn write_i32(&mut self, v: i32) {
        self.add(u64::from(v as u32));
    }

    fn write_u32(&mut self, v: u32) {
        self.add(u64::from(v));
    }

    fn write_u64(&mut self, v: u64) {
        self.add(v);
    }

    fn write_usize(&mut self, v: usize) {
        self.add(v as u64);
    }
}

/// Whether someone at `from` would call out to start `s` with someone at `to`: within earshot,
/// but not near enough to just start talking.
pub(crate) fn calls_out(nav: &NavGrid, s: &SocialDef, from: [f32; 2], to: [f32; 2]) -> bool {
    dist(from, to) > s.distance + ENGAGE_SLACK && within_earshot(nav, from, to)
}
