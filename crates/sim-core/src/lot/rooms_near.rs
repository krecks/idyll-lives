//! Finding the rooms again round the walls that changed (`Lot::compute_rooms` after the first
//! time), with the same rooms, numbered the same, as finding them all again would give.
//!
//! A room (or a stretch of outdoors) none of whose edges changed is what it was. So only the
//! rooms and the outdoors touching a changed edge are found again, by floods from the tile
//! halves either side of each changed edge, all at once and a step each in turn: a flood that
//! runs out before it reaches the lot's border is a room, one that reaches the border (or meets
//! one that did) is outdoors. When the outdoors touched a changed edge, some flood must end up
//! outdoors; once every other flood has run out, the one still going is that one, and needn't
//! cross the whole town to show it. Then the rooms that went are cleared, the rooms are numbered
//! by their first tile half again (the rooms nobody touched keep their keys), and the ones whose
//! number moved are relabelled over the tiles round them.

use std::collections::{HashMap, VecDeque};

use super::{CHUNK_TILES, EdgeRef, Lot, OUTDOORS, RoomMeta, RoomNode, Side, VOID, grow};
use crate::path::join;

/// Past this many tile halves the floods give up and every room is found again (the whole
/// outdoors split in two would take as long either way).
const MOST_NODES: usize = 1 << 18;
/// How many nodes each flood takes in its turn.
const TURN: usize = 16;
/// Room ids stay below this, so a relabelled one can be told from one still to do (the top bit).
const MOVED: u16 = 0x8000;

/// One step of a flood: off the lot, into a wall, or onto a node.
enum Step {
    Border,
    Shut,
    Node(RoomNode),
}

struct Flood {
    queue: VecDeque<RoomNode>,
    /// Tile halves (slot, half) and chunks that aren't allocated, reached so far.
    tiles: Vec<(usize, usize)>,
    voids: Vec<usize>,
    /// The key of its first tile half, row by row (as `compute_all_rooms` numbers rooms).
    key: u64,
    rect: [i32; 4],
    /// Reached the border: outdoors.
    border: bool,
}

fn node_key(n: RoomNode) -> u64 {
    match n {
        RoomNode::Tile(s, h, ..) => (s as u64) << 1 | h as u64,
        RoomNode::Void(c) => 1 << 63 | c as u64,
    }
}

const EMPTY: [i32; 4] = [i32::MAX, i32::MAX, i32::MIN, i32::MIN];

fn find(parent: &mut [usize], mut f: usize) -> usize {
    while parent[f] != f {
        parent[f] = parent[parent[f]];
        f = parent[f];
    }
    f
}

impl Lot {
    /// The key `compute_all_rooms` numbers a room by, for half `h` of tile `(x, z)`.
    fn half_key(&self, x: i32, z: i32, h: usize) -> u64 {
        (z as u64 * self.width as u64 + x as u64) * 2 + h as u64
    }

    /// The node holding the half of tile `(x, z)` on its `side` (the chunk, if it isn't there).
    fn node_on(&self, x: i32, z: i32, side: Side) -> RoomNode {
        let c = self.chunk_of(x as usize, z as usize);
        if self.base[c] == 0 {
            return RoomNode::Void(c);
        }
        RoomNode::Tile(self.tile_index(x, z), self.half_at_side(x, z, side), x, z)
    }

    fn room_step(&self, x: i32, z: i32, side: Side) -> Step {
        let (dx, dz) = side.offset();
        let (nx, nz) = (x + dx, z + dz);
        if !self.in_bounds(nx, nz) {
            return Step::Border;
        }
        if self.edge_between(x, z, nx, nz).is_wall() {
            return Step::Shut;
        }
        Step::Node(self.node_on(nx, nz, side.opposite()))
    }

    /// The steps out of `node`, as `compute_all_rooms` takes them.
    fn room_steps(&self, node: RoomNode, out: &mut Vec<Step>) {
        out.clear();
        match node {
            RoomNode::Tile(i, h, x, z) => {
                for side in Side::ALL {
                    if self.diags[i].is_none() || self.half_at_side(x, z, side) == h {
                        out.push(self.room_step(x, z, side));
                    }
                }
            }
            RoomNode::Void(c) => {
                let [x0, z0, x1, z1] = self.chunk_rect(c);
                for z in z0..z1 {
                    out.push(self.room_step(x0, z, Side::NegX));
                    out.push(self.room_step(x1 - 1, z, Side::PosX));
                }
                for x in x0..x1 {
                    out.push(self.room_step(x, z0, Side::NegZ));
                    out.push(self.room_step(x, z1 - 1, Side::PosZ));
                }
            }
        }
    }

    /// A node joins flood `f`.
    fn claim(&self, f: &mut Flood, n: RoomNode) {
        match n {
            RoomNode::Tile(s, h, x, z) => {
                f.tiles.push((s, h));
                f.key = f.key.min(self.half_key(x, z, h));
                grow(&mut f.rect, x, z);
            }
            RoomNode::Void(c) => {
                f.voids.push(c);
                let [x0, z0, x1, z1] = self.chunk_rect(c);
                f.key = f.key.min(self.half_key(x0, z0, 0));
                f.rect = join(f.rect, [x0, z0, x1, z1]);
            }
        }
        f.queue.push_back(n);
    }

    /// The room id slot `s` reports (an indoor half's), and its chunk's indoor count with it.
    fn settle_slot(&mut self, s: usize) {
        let h = self.halves[s];
        let r = if h[0] != OUTDOORS { h[0] } else { h[1] };
        let (was, is) = (self.rooms[s] != OUTDOORS, r != OUTDOORS);
        self.rooms[s] = r;
        if was != is {
            let c = self.order[(s - 1) / CHUNK_TILES] as usize;
            if is {
                self.indoor[c] += 1;
            } else {
                self.indoor[c] -= 1;
            }
        }
    }

    /// The slots of the tiles in `rect` that are there.
    fn rect_slots(&self, [x0, z0, x1, z1]: [i32; 4]) -> Vec<usize> {
        let mut out = Vec::new();
        for z in z0.max(0)..z1.min(self.depth as i32) {
            for x in x0.max(0)..x1.min(self.width as i32) {
                let s = self.tile_index(x, z);
                if s != VOID {
                    out.push(s);
                }
            }
        }
        out
    }

    /// Finds the rooms again round `edges` (see the module docs). Returns false, having changed
    /// nothing, where finding them all again is the better way.
    pub(super) fn rooms_near(&mut self, edges: &[EdgeRef]) -> bool {
        if edges.is_empty() {
            return true;
        }
        // The halves either side of each edge, and the rooms they were in.
        let mut seeds = Vec::new();
        let mut old = Vec::new();
        let mut touched = EMPTY;
        for &e in edges {
            let (x, z) = match e {
                EdgeRef::H(x, z) | EdgeRef::V(x, z) | EdgeRef::Diag(x, z) => (x as i32, z as i32),
            };
            let sides: &[(i32, i32, Side)] = match e {
                EdgeRef::H(..) => &[(x, z - 1, Side::PosZ), (x, z, Side::NegZ)],
                EdgeRef::V(..) => &[(x - 1, z, Side::PosX), (x, z, Side::NegX)],
                EdgeRef::Diag(..) => &[],
            };
            for &(tx, tz, side) in sides {
                if self.in_bounds(tx, tz) {
                    let n = self.node_on(tx, tz, side);
                    old.push(match n {
                        RoomNode::Tile(s, h, ..) => self.halves[s][h],
                        RoomNode::Void(_) => OUTDOORS,
                    });
                    seeds.push(n);
                    grow(&mut touched, tx, tz);
                }
            }
            if matches!(e, EdgeRef::Diag(..)) && self.in_bounds(x, z) {
                // Both halves as they were (put up, taken down or turned: either may have
                // been anything), and each half there is now.
                let s = self.tile_index(x, z);
                old.extend(self.halves[s]);
                grow(&mut touched, x, z);
                if s == VOID {
                    seeds.push(self.node_on(x, z, Side::NegZ));
                    continue;
                }
                let halves = if self.diags[s].is_some() { 2 } else { 1 };
                for h in 0..halves {
                    seeds.push(RoomNode::Tile(s, h, x, z));
                }
            }
        }
        let need_outdoor = old.contains(&OUTDOORS);
        old.sort_unstable();
        old.dedup();
        old.retain(|&l| l != OUTDOORS);

        // The floods, a turn each.
        let mut owner: HashMap<u64, usize> = HashMap::new();
        let mut floods: Vec<Flood> = Vec::new();
        let mut parent: Vec<usize> = Vec::new();
        for n in seeds {
            let k = node_key(n);
            if owner.contains_key(&k) {
                continue;
            }
            let f = floods.len();
            owner.insert(k, f);
            parent.push(f);
            let mut flood = Flood {
                queue: VecDeque::new(),
                tiles: Vec::new(),
                voids: Vec::new(),
                key: u64::MAX,
                rect: EMPTY,
                border: false,
            };
            self.claim(&mut flood, n);
            floods.push(flood);
        }
        let mut outdoor_found = false;
        let mut steps = Vec::new();
        let mut active: Vec<usize> = (0..floods.len()).collect();
        loop {
            active.retain(|&f| parent[f] == f && !floods[f].border && !floods[f].queue.is_empty());
            if active.is_empty() {
                break;
            }
            if need_outdoor && !outdoor_found && active.len() == 1 {
                // Every other flood ran out: this is the outdoors.
                floods[active[0]].border = true;
                break;
            }
            if owner.len() > MOST_NODES {
                return false;
            }
            for &start in &active {
                let mut f = find(&mut parent, start);
                for _ in 0..TURN {
                    if floods[f].border {
                        break;
                    }
                    let Some(node) = floods[f].queue.pop_front() else {
                        break;
                    };
                    self.room_steps(node, &mut steps);
                    for step in steps.drain(..) {
                        match step {
                            Step::Border => floods[f].border = true,
                            Step::Shut => {}
                            Step::Node(n) => match owner.get(&node_key(n)) {
                                Some(&g) => {
                                    let g = find(&mut parent, g);
                                    if g != f {
                                        f = merge(&mut floods, &mut parent, f, g);
                                    }
                                }
                                None => {
                                    owner.insert(node_key(n), f);
                                    self.claim(&mut floods[f], n);
                                }
                            },
                        }
                    }
                }
                if floods[f].border {
                    floods[f].queue.clear();
                    outdoor_found = true;
                }
            }
        }
        let enclosed: Vec<usize> = (0..floods.len())
            .filter(|&f| parent[f] == f && !floods[f].border)
            .collect();
        let before = self.room_meta.len();
        if before - old.len() + enclosed.len() >= MOVED as usize - 1 {
            return false;
        }

        // The rooms that went (or are found again) are cleared.
        for &l in &old {
            let rect = self.room_meta[l as usize - 1].rect();
            touched = join(touched, rect);
            for s in self.rect_slots(rect) {
                let mut h = self.halves[s];
                for half in &mut h {
                    if *half == l {
                        *half = OUTDOORS;
                    }
                }
                if h != self.halves[s] {
                    self.halves[s] = h;
                    self.settle_slot(s);
                }
            }
        }

        // Numbered again by their keys: the rooms kept, and the ones found.
        enum Was {
            Kept(u16),
            Found(usize),
        }
        let mut order: Vec<(u64, Was)> = (1..=before as u16)
            .filter(|id| old.binary_search(id).is_err())
            .map(|id| (self.room_meta[id as usize - 1].key, Was::Kept(id)))
            .chain(enclosed.iter().map(|&f| (floods[f].key, Was::Found(f))))
            .collect();
        order.sort_unstable_by_key(|(key, _)| *key);
        let mut remap = vec![OUTDOORS; before + 1];
        let mut meta = Vec::with_capacity(order.len());
        let mut moved = Vec::new();
        for (k, (key, was)) in order.iter().enumerate() {
            let id = k as u16 + 1;
            match *was {
                Was::Kept(o) => {
                    remap[o as usize] = id;
                    let m = self.room_meta[o as usize - 1];
                    if o != id {
                        moved.push((o, id, m.rect()));
                    }
                    meta.push(m);
                }
                Was::Found(f) => meta.push(RoomMeta::new(*key, floods[f].rect)),
            }
        }
        // Rooms whose number moved: marked first (so a room isn't taken for another's old
        // number), then unmarked.
        for &(o, id, rect) in &moved {
            for s in self.rect_slots(rect) {
                for half in &mut self.halves[s] {
                    if *half == o {
                        *half = id | MOVED;
                    }
                }
            }
        }
        for &(_, id, rect) in &moved {
            for s in self.rect_slots(rect) {
                let mut hit = false;
                for half in &mut self.halves[s] {
                    if *half == id | MOVED {
                        *half = id;
                        hit = true;
                    }
                }
                if hit {
                    self.settle_slot(s);
                }
            }
        }
        // The rooms found.
        for (k, (_, was)) in order.iter().enumerate() {
            let Was::Found(f) = *was else {
                continue;
            };
            let id = k as u16 + 1;
            touched = join(touched, floods[f].rect);
            for &c in &floods[f].voids {
                // A chunk inside a room has a floor to stand on: it's there from now on.
                self.alloc(c);
                let tiles: Vec<usize> = self.rect_slots(self.chunk_rect(c));
                for s in tiles {
                    self.halves[s] = [id, id];
                    self.settle_slot(s);
                }
            }
            for &(s, h) in &floods[f].tiles {
                self.halves[s][h] = id;
            }
        }
        // Tiles without a diagonal have one room (a diagonal taken down joins its halves).
        for f in &floods {
            for &(s, _) in &f.tiles {
                if self.diags[s].is_none() {
                    self.halves[s][1] = self.halves[s][0];
                }
                self.settle_slot(s);
            }
        }
        self.room_meta = meta;

        if !self.changes.full {
            let ids_moved = !old.is_empty() || !moved.is_empty() || !enclosed.is_empty();
            self.changes.remap = match self.changes.remap.take() {
                Some(prev) => Some(
                    prev.iter()
                        .map(|&m| remap.get(m as usize).copied().unwrap_or(OUTDOORS))
                        .collect(),
                ),
                None if ids_moved => Some(remap),
                None => None,
            };
            self.changes.rect = Some(match self.changes.rect {
                Some(r) => join(r, touched),
                None => touched,
            });
        }
        true
    }
}

/// Floods `a` and `b` met: one flood from now on (the bigger one's), which it returns.
fn merge(floods: &mut [Flood], parent: &mut [usize], a: usize, b: usize) -> usize {
    let (keep, gone) = if floods[a].tiles.len() >= floods[b].tiles.len() {
        (a, b)
    } else {
        (b, a)
    };
    parent[gone] = keep;
    let mut g = std::mem::replace(
        &mut floods[gone],
        Flood {
            queue: VecDeque::new(),
            tiles: Vec::new(),
            voids: Vec::new(),
            key: u64::MAX,
            rect: EMPTY,
            border: false,
        },
    );
    let k = &mut floods[keep];
    k.queue.append(&mut g.queue);
    k.tiles.append(&mut g.tiles);
    k.voids.append(&mut g.voids);
    k.key = k.key.min(g.key);
    k.rect = join(k.rect, g.rect);
    k.border |= g.border;
    keep
}
