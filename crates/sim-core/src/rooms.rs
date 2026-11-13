//! Rooms and how good they are ("the house matters").
//!
//! A room is an enclosed area of the lot (`Lot::compute_rooms`); each plot's outdoor tiles are
//! scored as its garden. Room ids change whenever walls do, so nothing here is saved: rooms are
//! recomputed from the lot, the objects and the per-tile dirt (`World::refresh_rooms`).

use crate::content::TagMask;
use crate::lot::{Edge, EdgeRef, OUTDOORS};
use crate::world::World;

/// Scores of a room, each 0..1, and the weighted `overall` (content `roomRules.weights`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RoomScores {
    pub size: f32,
    pub light: f32,
    pub decor: f32,
    pub clean: f32,
    pub function: f32,
    pub overall: f32,
}

impl RoomScores {
    /// The factors in order (size, light, decor, cleanliness, function).
    pub fn factors(&self) -> [f32; 5] {
        [self.size, self.light, self.decor, self.clean, self.function]
    }

    /// The weakest factor (index into `factors`).
    pub fn weakest(&self) -> usize {
        let f = self.factors();
        (0..f.len())
            .min_by(|&a, &b| f[a].total_cmp(&f[b]))
            .unwrap_or(0)
    }
}

/// Factor names, in the order of `RoomScores::factors` (ids for the UI and saves).
pub const FACTORS: [&str; 5] = ["size", "light", "decor", "clean", "function"];

#[derive(Debug, Clone, PartialEq)]
pub struct RoomInfo {
    /// Lot room id (`OUTDOORS` for a plot's garden).
    pub id: u16,
    pub plot: Option<u32>,
    pub garden: bool,
    /// Room kind (index into `Content::room_kinds`); None: nothing tells what it's for.
    pub kind: Option<usize>,
    /// Things of two exclusive kinds stand in it (a bed in the kitchen).
    pub mixed: bool,
    /// The first essential of its kind nothing in it offers (index into the kind's `essentials`).
    pub missing: Option<usize>,
    /// Indoor tiles (a tile split by a diagonal wall counts half).
    pub tiles: f32,
    pub windows: u16,
    pub doors: u16,
    pub lamps: u16,
    /// Floor its lamps light at night, in all (tiles; `ObjectDef::lamp`).
    pub lit: f32,
    /// A wall of it (plain or with a window) is an outside wall, above ground: somewhere a
    /// window could be. A room without (an inside bathroom, a hall, anything in the basement)
    /// gets its daylight score from its lamps too.
    pub outside: bool,
    /// Decor points of what stands in it.
    pub decor: f32,
    /// Share of its floor and wall faces with a chosen covering.
    pub coverings: f32,
    /// Mean dirt of its tiles (0..1).
    pub dirt: f32,
    /// Centre of its tiles (for labels and the camera).
    pub centre: [f32; 2],
    /// Tags offered by what stands in it.
    pub tags: TagMask,
    pub scores: RoomScores,
}

#[derive(Default)]
struct Acc {
    tiles: f32,
    sx: f32,
    sz: f32,
    dirt: f32,
    /// Tiles and their weights, in the order their dirt was added up.
    cells: Vec<(u32, f32)>,
    windows: u16,
    doors: u16,
    outside: bool,
    /// What stands in it.
    things: Things,
    floors: f32,
    faces: f32,
    covered_faces: f32,
    first: (i32, i32),
}

/// What it takes to rescore cleanliness between structure changes without finding the rooms
/// again (see `World::refresh_rooms`): which tiles' dirt counts towards each room and garden,
/// in the order `compute` adds it up (so scores come out exactly the same), which tiles take
/// each one's score, and which had a mess since they were last scored.
///
/// Each room's lists are kept under a handle that stays while other rooms come and go (rooms
/// found again round a wall: `World::refresh_rooms_near`); `handle` gives each room's.
#[derive(Debug, Clone, Default)]
pub(crate) struct RoomTiles {
    /// Per room (index into `World::rooms`): its handle (index into the lists below).
    handle: Vec<u32>,
    /// Per handle: tiles and weights.
    cells: Vec<Vec<(u32, f32)>>,
    /// Per handle: the tiles whose `World::room_score` is its overall score.
    scored: Vec<Vec<u32>>,
    /// Per tile: the handles of the rooms its dirt counts towards (`NONE`: none).
    of_tile: Vec<[u32; 2]>,
    /// Per handle: its dirt changed since it was scored.
    stale: Vec<bool>,
    /// Handles no room has.
    free: Vec<u32>,
}

/// What `RoomTiles::summary` gives: per room its tiles (weights as bits) and the tiles taking its
/// score, and per tile the rooms its dirt counts towards.
pub(crate) type RoomTilesSummary = (Vec<(Vec<(u32, u32)>, Vec<u32>)>, Vec<Vec<u32>>);

const NONE: u32 = u32::MAX;
/// No room sets a tile's score (`World::refresh_rooms`, `RoomTiles::new`).
pub(crate) const NO_ROOM: u32 = NONE;

impl RoomTiles {
    /// `cells` per room from `compute`; `source` per tile slot: the room that sets its score
    /// (`NO_ROOM`: none).
    pub fn new(cells: Vec<Vec<(u32, f32)>>, source: &[u32]) -> Self {
        let mut scored = vec![Vec::new(); cells.len()];
        for (i, &r) in source.iter().enumerate() {
            if r != NO_ROOM {
                scored[r as usize].push(i as u32);
            }
        }
        let mut of_tile = vec![[NONE; 2]; source.len()];
        for (r, list) in cells.iter().enumerate() {
            for &(i, _) in list {
                let slot = &mut of_tile[i as usize];
                let k = usize::from(slot[0] != NONE);
                slot[k] = r as u32;
            }
        }
        let stale = vec![false; cells.len()];
        Self {
            handle: (0..cells.len() as u32).collect(),
            cells,
            scored,
            of_tile,
            stale,
            free: Vec::new(),
        }
    }

    /// Tile `i`'s dirt changed.
    pub fn touch(&mut self, i: usize) {
        for r in self.of_tile.get(i).into_iter().flatten() {
            if *r != NONE {
                self.stale[*r as usize] = true;
            }
        }
    }

    /// Scores cleanliness again where there was a mess, and updates the score per tile.
    pub fn rescore(
        &mut self,
        w: &mut [RoomInfo],
        rules: &crate::content::RoomRules,
        dirt: &[f32],
        room_score: &mut [f32],
    ) {
        for (r, room) in w.iter_mut().enumerate() {
            let h = self.handle[r] as usize;
            if !std::mem::take(&mut self.stale[h]) {
                continue;
            }
            let sum = self.cells[h].iter().fold(0.0, |s, &(i, weight)| {
                s + dirt.get(i as usize).copied().unwrap_or(0.0) * weight
            });
            score_dirt(room, sum, rules);
            for &i in &self.scored[h] {
                room_score[i as usize] = room.scores.overall;
            }
        }
    }

    /// The handles of the rooms, in order (see `set_handles`).
    pub fn take_handles(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.handle)
    }

    /// The handles of the rooms, in the order the rooms now have.
    pub fn set_handles(&mut self, handles: Vec<u32>) {
        self.handle = handles;
    }

    /// Room `h` went (or is found again): its lists go.
    pub fn drop_room(&mut self, h: u32) {
        let cells = std::mem::take(&mut self.cells[h as usize]);
        for &(i, _) in &cells {
            if let Some(slot) = self.of_tile.get_mut(i as usize) {
                for r in slot.iter_mut().filter(|r| **r == h) {
                    *r = NONE;
                }
            }
        }
        self.scored[h as usize] = Vec::new();
        self.stale[h as usize] = false;
        self.free.push(h);
    }

    /// A room found (again) with its tiles: its handle.
    pub fn add_room(&mut self, cells: Vec<(u32, f32)>, scored: Vec<u32>) -> u32 {
        let h = self.free.pop().unwrap_or_else(|| {
            self.cells.push(Vec::new());
            self.scored.push(Vec::new());
            self.stale.push(false);
            self.cells.len() as u32 - 1
        });
        for &(i, _) in &cells {
            let slot = &mut self.of_tile[i as usize];
            let k = usize::from(slot[0] != NONE);
            slot[k] = h;
        }
        self.cells[h as usize] = cells;
        self.scored[h as usize] = scored;
        self.stale[h as usize] = false;
        h
    }

    /// Per room (in order): its tiles and weights, the tiles taking its score (sorted), and per
    /// tile the rooms (indices, sorted) its dirt counts towards: what these lists say, however
    /// they're kept (for `World::check_rooms`).
    pub fn summary(&self) -> RoomTilesSummary {
        let mut index = vec![NONE; self.cells.len()];
        for (r, &h) in self.handle.iter().enumerate() {
            index[h as usize] = r as u32;
        }
        let rooms = self
            .handle
            .iter()
            .map(|&h| {
                let cells = self.cells[h as usize]
                    .iter()
                    .map(|&(i, w)| (i, w.to_bits()))
                    .collect();
                let mut scored = self.scored[h as usize].clone();
                scored.sort_unstable();
                (cells, scored)
            })
            .collect();
        let tiles = self
            .of_tile
            .iter()
            .map(|slot| {
                let mut rs: Vec<u32> = slot
                    .iter()
                    .filter(|&&h| h != NONE)
                    .map(|&h| index[h as usize])
                    .collect();
                rs.sort_unstable();
                rs
            })
            .collect();
        (rooms, tiles)
    }

    /// The per-tile list follows the lot's slots (chunks allocated since).
    pub fn grow(&mut self, slots: usize) {
        if self.of_tile.len() < slots {
            self.of_tile.resize(slots, [NONE; 2]);
        }
    }
}

/// What the things standing in a room (or a garden) give it.
#[derive(Default, Clone, Copy)]
struct Things {
    decor: f32,
    lamps: u16,
    lit: f32,
    tags: TagMask,
    /// Stairs stand in it.
    stairs: bool,
}

impl Things {
    fn add(&mut self, w: &World, o: &crate::world::ObjectInstance) {
        let content = &w.content;
        let def = &content.objects[o.def];
        self.decor += def.decor * content.object_rules.quality_factor(o.quality);
        self.lamps += u16::from(def.lamp > 0.0);
        self.lit += def.lamp;
        // Broken things don't do what they're for (a broken shower is no shower).
        if !o.broken() {
            self.tags |= def.tags;
        }
        self.stairs |= def.stairs;
    }
}

/// A room's (or garden's) kind and scores but cleanliness, from its tiles, walls and coverings
/// (as `compute` found them) and what stands in it (`things`).
fn score_things(room: &mut RoomInfo, a: &Things, content: &crate::content::Content) {
    let rules = &content.room_rules;
    room.lamps = a.lamps;
    room.lit = a.lit;
    room.decor = a.decor;
    room.tags = a.tags;
    if room.garden {
        let decor = (a.decor / (room.tiles * rules.garden_decor_per_tile)).min(1.0);
        room.scores = RoomScores {
            size: 1.0,
            light: 1.0,
            decor,
            clean: 1.0,
            function: 1.0,
            overall: 0.0,
        };
        return;
    }
    let tiles = room.tiles;
    let kinds = &content.room_kinds;
    // By what's done there, else whether it's a way through.
    let passage = room.doors >= 2 || a.stairs;
    let kind = kinds
        .iter()
        .position(|k| a.tags & k.tags != 0)
        .or_else(|| kinds.iter().position(|k| k.passage).filter(|_| passage));
    let exclusive = kinds
        .iter()
        .filter(|k| k.exclusive && a.tags & k.tags != 0)
        .count();
    let missing = kind.and_then(|k| kinds[k].essentials.iter().position(|&e| a.tags & e == 0));
    let [cramped, comfy] = kind.map_or([4.0, 9.0], |k| kinds[k].size);
    let size = if tiles < cramped {
        0.5 * tiles / cramped.max(1e-3)
    } else if tiles < comfy {
        0.5 + 0.5 * (tiles - cramped) / (comfy - cramped).max(1e-3)
    } else {
        1.0
    };
    let day = (f32::from(room.windows) * rules.window_tiles / tiles).min(1.0);
    let night = (a.lit / tiles).min(1.0);
    // No window could light a room with no outside wall: its lamps are all its light, by day too.
    let day = if room.outside { day } else { night };
    let decor = (a.decor / (tiles * rules.decor_per_tile) * 0.8 + room.coverings * 0.4).min(1.0);
    let function = match (kind, exclusive > 1, missing) {
        (None, _, _) => 0.4,
        (Some(_), true, _) => 0.6,
        (Some(_), false, Some(_)) => 0.7,
        _ => 1.0,
    };
    room.kind = kind;
    room.mixed = exclusive > 1;
    room.missing = missing;
    room.scores = RoomScores {
        size,
        light: (day + night) / 2.0,
        decor,
        clean: 1.0,
        function,
        overall: 0.0,
    };
}

/// The rooms scored again for what stands in them, with their tiles, walls and coverings as
/// they were (`World::refresh_room_things`: things came, went, broke, were mended or upgraded,
/// and nothing was built): exactly what `compute` would give, without finding the rooms or
/// walking their tiles but to add up their dirt. Then the score per tile follows. With `only`
/// (per room: whether things changed in it), only those rooms are scored again for their
/// things, and the rest only for their dirt where there was a mess, as `RoomTiles::rescore`.
pub(crate) fn rescore_things(
    w: &World,
    rooms: &mut [RoomInfo],
    tiles: &mut RoomTiles,
    room_score: &mut [f32],
    only: Option<&[bool]>,
) {
    let picked = |k: usize| only.is_none_or(|o| o.get(k).copied().unwrap_or(false));
    let lot = &w.lot;
    let rules = &w.content.room_rules;
    let most = rooms
        .iter()
        .filter(|r| !r.garden)
        .map(|r| r.id as usize)
        .max()
        .unwrap_or(0);
    let mut by_id = vec![usize::MAX; most + 1];
    let mut by_plot = vec![usize::MAX; w.plots.len()];
    for (k, r) in rooms.iter().enumerate() {
        match (r.garden, r.plot) {
            (true, Some(p)) => {
                if let Some(slot) = by_plot.get_mut(p as usize) {
                    *slot = k;
                }
            }
            (true, None) => {}
            (false, _) => by_id[r.id as usize] = k,
        }
    }
    // (Only the gardens picked need the plot of what stands outdoors.)
    let any_garden = only.is_none() || by_plot.iter().any(|&k| picked(k));
    let mut things = vec![Things::default(); rooms.len()];
    for o in &w.objects {
        let r = if lot.in_bounds(o.x, o.z) {
            lot.room_at(o.x, o.z)
        } else {
            OUTDOORS
        };
        let k = if r == OUTDOORS {
            if !any_garden {
                continue;
            }
            let Some(plot) = w.plot_at(o.x, o.z) else {
                continue;
            };
            by_plot.get(plot as usize).copied().unwrap_or(usize::MAX)
        } else {
            by_id.get(r as usize).copied().unwrap_or(usize::MAX)
        };
        if picked(k)
            && let Some(t) = things.get_mut(k)
        {
            t.add(w, o);
        }
    }
    let dirt_at = |i: u32| w.dirt.get(i as usize).copied().unwrap_or(0.0);
    for (k, room) in rooms.iter_mut().enumerate() {
        if !picked(k) {
            continue;
        }
        let h = tiles.handle[k] as usize;
        let before = room.scores.overall;
        score_things(room, &things[k], &w.content);
        let dirt = tiles.cells[h]
            .iter()
            .fold(0.0, |s, &(i, weight)| s + dirt_at(i) * weight);
        score_dirt(room, dirt, rules);
        tiles.stale[h] = false;
        if room.scores.overall.to_bits() != before.to_bits() {
            for &i in &tiles.scored[h] {
                room_score[i as usize] = room.scores.overall;
            }
        }
    }
    if only.is_some() {
        tiles.rescore(rooms, rules, &w.dirt, room_score);
    } else {
        tiles.stale.fill(false);
    }
}

/// Weighted overall score (content `roomRules.weights`).
fn overall(s: &RoomScores, rules: &crate::content::RoomRules) -> f32 {
    let total: f32 = rules.weights.iter().sum::<f32>().max(1e-6);
    s.factors()
        .iter()
        .zip(rules.weights)
        .map(|(f, w)| f * w)
        .sum::<f32>()
        / total
}

/// Cleanliness from the room's summed dirt, and the overall score with it.
fn score_dirt(room: &mut RoomInfo, dirt: f32, rules: &crate::content::RoomRules) {
    room.dirt = dirt / room.tiles;
    room.scores.clean = 1.0 - room.dirt;
    room.scores.overall = overall(&room.scores, rules);
}

/// Every room and garden of the town, with scores, and each one's tiles (as `RoomTiles::cells`).
pub(crate) fn compute(w: &World) -> (Vec<RoomInfo>, Vec<Vec<(u32, f32)>>) {
    let lot = &w.lot;
    // Rooms by lot id, gardens by plot.
    let halves = lot.half_rooms();
    let max_id = halves.iter().flatten().copied().max().unwrap_or(0) as usize;
    let mut rooms: Vec<Acc> = (0..=max_id).map(|_| Acc::default()).collect();
    let mut gardens: Vec<Acc> = (0..w.plots.len()).map(|_| Acc::default()).collect();
    let dirt_at = |i: usize| w.dirt.get(i).copied().unwrap_or(0.0);
    let add = |acc: &mut Acc, x: i32, z: i32, i: usize, weight: f32| {
        if acc.tiles == 0.0 {
            acc.first = (x, z);
        }
        acc.tiles += weight;
        acc.sx += (x as f32 + 0.5) * weight;
        acc.sz += (z as f32 + 0.5) * weight;
        acc.dirt += dirt_at(i) * weight;
        acc.cells.push((i as u32, weight));
    };
    let parts = |[r0, r1]: [u16; 2]| -> [(u16, f32); 2] {
        if r0 == r1 {
            [(r0, 1.0), (OUTDOORS, 0.0)]
        } else {
            [(r0, 0.5), (r1, 0.5)]
        }
    };
    for (x, z, i) in lot.indoor_tiles() {
        for (r, weight) in parts(halves[i]) {
            if r == OUTDOORS || weight == 0.0 {
                continue;
            }
            let acc = &mut rooms[r as usize];
            add(acc, x, z, i, weight);
            if lot.floor(x as u16, z as u16) != 0 {
                acc.floors += weight;
            }
        }
    }
    // Gardens: each plot's outdoor tiles.
    for (pi, p) in w.plots.iter().enumerate() {
        for z in p.z..p.z + p.d {
            for x in p.x..p.x + p.w {
                if !lot.in_bounds(x, z) {
                    continue;
                }
                let i = lot.tile_index(x, z);
                let [r0, r1] = halves[i];
                // A tile split by a diagonal wall: each half counts half.
                let half = |r: u16| if r == OUTDOORS { 0.5 } else { 0.0 };
                let outdoor = if r0 == r1 {
                    2.0 * half(r0)
                } else {
                    half(r0) + half(r1)
                };
                if outdoor > 0.0 {
                    add(&mut gardens[pi], x, z, i, outdoor);
                }
            }
        }
    }

    // Walls, doors and windows on each room's boundary (a window between two rooms lights both),
    // and which rooms have an outside wall (with outdoors across it, not underground).
    let mut boundary = |r: u16, across: u16, z: i32, edge: Edge, covered: bool| {
        if r == OUTDOORS {
            return;
        }
        let Some(acc) = rooms.get_mut(r as usize) else {
            return;
        };
        if across == OUTDOORS && matches!(edge, Edge::Wall | Edge::Window) && w.storey_of(z) >= 0 {
            acc.outside = true;
        }
        match edge {
            Edge::Window => acc.windows += 1,
            Edge::Door => acc.doors += 1,
            _ => {}
        }
        if edge.is_wall() {
            acc.faces += 1.0;
            if covered {
                acc.covered_faces += 1.0;
            }
        }
    };
    let room = |x: i32, z: i32| {
        if lot.in_bounds(x, z) {
            lot.room_at(x, z)
        } else {
            OUTDOORS
        }
    };
    for (at, e) in lot.walls() {
        if !e.is_wall() {
            continue;
        }
        let look = lot.look(at);
        match at {
            EdgeRef::H(x, z) => {
                let (x, z) = (x as i32, z as i32);
                let (a, b) = (room(x, z - 1), room(x, z));
                boundary(a, b, z - 1, e, look.sides[0] != 0);
                boundary(b, a, z, e, look.sides[1] != 0);
            }
            EdgeRef::V(x, z) => {
                let (x, z) = (x as i32, z as i32);
                let (a, b) = (room(x - 1, z), room(x, z));
                boundary(a, b, z, e, look.sides[0] != 0);
                boundary(b, a, z, e, look.sides[1] != 0);
            }
            EdgeRef::Diag(x, z) => {
                let (x, z) = (x as i32, z as i32);
                let [h0, h1] = halves[lot.tile_index(x, z)];
                boundary(h0, h1, z, e, look.sides[0] != 0);
                boundary(h1, h0, z, e, look.sides[1] != 0);
            }
        }
    }

    // What stands in each room.
    for o in &w.objects {
        let r = room(o.x, o.z);
        let acc = if r == OUTDOORS {
            let Some(plot) = w.plot_at(o.x, o.z) else {
                continue;
            };
            &mut gardens[plot as usize]
        } else {
            let Some(acc) = rooms.get_mut(r as usize) else {
                continue;
            };
            acc
        };
        acc.things.add(w, o);
    }

    let mut out: Vec<(RoomInfo, Vec<(u32, f32)>)> = Vec::with_capacity(rooms.len() + gardens.len());
    for (id, a) in rooms.into_iter().enumerate().filter(|(_, a)| a.tiles > 0.0) {
        out.push(finish_room(w, id as u16, a));
    }
    for (plot, a) in gardens
        .into_iter()
        .enumerate()
        .filter(|(_, a)| a.tiles > 0.0)
    {
        out.push(finish_garden(w, plot as u32, a));
    }
    // Deterministic order: by plot, gardens last, then by room id.
    out.sort_by_key(|(r, _)| room_order(r));
    out.into_iter().unzip()
}

/// The order `compute` gives the rooms in: by plot, gardens last, then by room id.
pub(crate) fn room_order(r: &RoomInfo) -> (u32, bool, u16) {
    (r.plot.unwrap_or(u32::MAX), r.garden, r.id)
}

/// Room `id` from what was added up over its tiles, walls and things.
fn finish_room(w: &World, id: u16, a: Acc) -> (RoomInfo, Vec<(u32, f32)>) {
    let content = &w.content;
    let coverings = 0.5 * (a.floors / a.tiles) + 0.5 * (a.covered_faces / a.faces.max(1.0));
    let mut room = RoomInfo {
        id,
        plot: w.plot_at(a.first.0, a.first.1),
        garden: false,
        kind: None,
        mixed: false,
        missing: None,
        tiles: a.tiles,
        windows: a.windows,
        doors: a.doors,
        lamps: 0,
        lit: 0.0,
        outside: a.outside,
        decor: 0.0,
        coverings,
        dirt: 0.0,
        centre: [a.sx / a.tiles, a.sz / a.tiles],
        tags: 0,
        scores: RoomScores::default(),
    };
    score_things(&mut room, &a.things, content);
    score_dirt(&mut room, a.dirt, &content.room_rules);
    (room, a.cells)
}

/// Plot `plot`'s garden from what was added up over its outdoor tiles and things.
fn finish_garden(w: &World, plot: u32, a: Acc) -> (RoomInfo, Vec<(u32, f32)>) {
    let content = &w.content;
    let mut garden = RoomInfo {
        id: OUTDOORS,
        plot: Some(plot),
        garden: true,
        kind: None,
        mixed: false,
        missing: None,
        tiles: a.tiles,
        windows: 0,
        doors: 0,
        lamps: 0,
        lit: 0.0,
        outside: true,
        decor: 0.0,
        coverings: 0.0,
        dirt: 0.0,
        centre: [a.sx / a.tiles, a.sz / a.tiles],
        tags: 0,
        scores: RoomScores::default(),
    };
    score_things(&mut garden, &a.things, content);
    score_dirt(&mut garden, a.dirt, &content.room_rules);
    (garden, a.cells)
}

/// Rooms `ids` (lot ids, each once) and the gardens of `plots` (each once), exactly as `compute`
/// finds them, walking only the tiles round each (`Lot::room_rect`) and the plots: for
/// `World::refresh_rooms_near`, after a wall was built. A room or garden with no tiles is left
/// out, as `compute` leaves it out; the rest come in no particular order.
pub(crate) fn compute_near(
    w: &World,
    ids: &[u16],
    plots: &[u32],
) -> Vec<(RoomInfo, Vec<(u32, f32)>)> {
    let lot = &w.lot;
    let halves = lot.half_rooms();
    let most = ids.iter().copied().max().unwrap_or(0) as usize;
    let mut room_of = vec![usize::MAX; most + 1];
    for (k, &id) in ids.iter().enumerate() {
        room_of[id as usize] = k;
    }
    let mut garden_of = vec![usize::MAX; w.plots.len()];
    for (k, &p) in plots.iter().enumerate() {
        garden_of[p as usize] = k;
    }
    let mut rooms: Vec<Acc> = ids.iter().map(|_| Acc::default()).collect();
    let mut gardens: Vec<Acc> = plots.iter().map(|_| Acc::default()).collect();
    let dirt_at = |i: usize| w.dirt.get(i).copied().unwrap_or(0.0);
    let add = |acc: &mut Acc, x: i32, z: i32, i: usize, weight: f32| {
        if acc.tiles == 0.0 {
            acc.first = (x, z);
        }
        acc.tiles += weight;
        acc.sx += (x as f32 + 0.5) * weight;
        acc.sz += (z as f32 + 0.5) * weight;
        acc.dirt += dirt_at(i) * weight;
        acc.cells.push((i as u32, weight));
    };
    let room = |x: i32, z: i32| {
        if lot.in_bounds(x, z) {
            lot.room_at(x, z)
        } else {
            OUTDOORS
        }
    };
    for (&id, acc) in ids.iter().zip(&mut rooms) {
        let Some([x0, z0, x1, z1]) = lot.room_rect(id) else {
            continue;
        };
        // Its tiles, row by row (as `compute` adds them up).
        for z in z0..z1 {
            for x in x0..x1 {
                let i = lot.tile_index(x, z);
                if i == crate::lot::VOID {
                    continue;
                }
                let [r0, r1] = halves[i];
                if r0 != id && r1 != id {
                    continue;
                }
                let weight = if r0 == r1 { 1.0 } else { 0.5 };
                add(acc, x, z, i, weight);
                if lot.floor(x as u16, z as u16) != 0 {
                    acc.floors += weight;
                }
            }
        }
        // Its walls, doors and windows, and whether one is an outside wall (as `compute`).
        let mut boundary = |r: u16, across: u16, z: i32, edge: Edge, covered: bool| {
            if r != id {
                return;
            }
            if across == OUTDOORS
                && matches!(edge, Edge::Wall | Edge::Window)
                && w.storey_of(z) >= 0
            {
                acc.outside = true;
            }
            match edge {
                Edge::Window => acc.windows += 1,
                Edge::Door => acc.doors += 1,
                _ => {}
            }
            if edge.is_wall() {
                acc.faces += 1.0;
                if covered {
                    acc.covered_faces += 1.0;
                }
            }
        };
        for (at, e) in lot.walls_in([x0, z0, x1 + 1, z1 + 1]) {
            if !e.is_wall() {
                continue;
            }
            let look = lot.look(at);
            match at {
                EdgeRef::H(x, z) => {
                    let (x, z) = (x as i32, z as i32);
                    let (a, b) = (room(x, z - 1), room(x, z));
                    boundary(a, b, z - 1, e, look.sides[0] != 0);
                    boundary(b, a, z, e, look.sides[1] != 0);
                }
                EdgeRef::V(x, z) => {
                    let (x, z) = (x as i32, z as i32);
                    let (a, b) = (room(x - 1, z), room(x, z));
                    boundary(a, b, z, e, look.sides[0] != 0);
                    boundary(b, a, z, e, look.sides[1] != 0);
                }
                EdgeRef::Diag(x, z) => {
                    let (x, z) = (x as i32, z as i32);
                    let [h0, h1] = halves[lot.tile_index(x, z)];
                    boundary(h0, h1, z, e, look.sides[0] != 0);
                    boundary(h1, h0, z, e, look.sides[1] != 0);
                }
            }
        }
    }
    // Gardens: each plot's outdoor tiles (as `compute`).
    for (&pi, acc) in plots.iter().zip(&mut gardens) {
        let p = &w.plots[pi as usize];
        for z in p.z..p.z + p.d {
            for x in p.x..p.x + p.w {
                if !lot.in_bounds(x, z) {
                    continue;
                }
                let i = lot.tile_index(x, z);
                let [r0, r1] = halves[i];
                let half = |r: u16| if r == OUTDOORS { 0.5 } else { 0.0 };
                let outdoor = if r0 == r1 {
                    2.0 * half(r0)
                } else {
                    half(r0) + half(r1)
                };
                if outdoor > 0.0 {
                    add(acc, x, z, i, outdoor);
                }
            }
        }
    }
    // What stands in each.
    for o in &w.objects {
        let r = room(o.x, o.z);
        let acc = if r == OUTDOORS {
            if plots.is_empty() {
                continue;
            }
            let Some(plot) = w.plot_at(o.x, o.z) else {
                continue;
            };
            match garden_of.get(plot as usize) {
                Some(&k) if k != usize::MAX => &mut gardens[k],
                _ => continue,
            }
        } else {
            match room_of.get(r as usize) {
                Some(&k) if k != usize::MAX => &mut rooms[k],
                _ => continue,
            }
        };
        acc.things.add(w, o);
    }
    let mut out = Vec::with_capacity(ids.len() + plots.len());
    for (&id, a) in ids.iter().zip(rooms) {
        if a.tiles > 0.0 {
            out.push(finish_room(w, id, a));
        }
    }
    for (&plot, a) in plots.iter().zip(gardens) {
        if a.tiles > 0.0 {
            out.push(finish_garden(w, plot, a));
        }
    }
    out
}

/// Thought subject for a room: its kind (garden 0, kind + 1, or 63 for a room nothing marks)
/// times 8 plus a factor (`FACTORS`).
pub fn room_subject(room: &RoomInfo, factor: usize) -> usize {
    let kind = if room.garden {
        0
    } else {
        room.kind.map_or(63, |k| k + 1)
    };
    kind * 8 + factor.min(7)
}

/// What residents make of their home, once an hour: a room they love or dislike (a thought,
/// a feeling, and for a disliked one a wish about its weakest factor), broken things to fix,
/// and a busy essential they had to wait for (another bathroom). Wishes come true once the
/// room is better, the thing is fixed, or there is another room of that kind.
pub(crate) fn opinions(w: &mut World) {
    use crate::planner::{
        HomeWish, MAX_HOME_WISHES, ThoughtKind, WAIT_FORGET_DAYS, WAIT_REPEAT_DAYS, think,
    };
    let tick = w.tick;
    let today = crate::clock::day(tick);
    for i in 0..w.sims.len() {
        let s = &w.sims[i];
        let Some(home) = w.households[s.household as usize].plot.filter(|_| s.here()) else {
            continue;
        };
        let (x, z) = s.tile();
        let at_home =
            s.away_until.is_none() && !s.asleep(&w.content) && w.plot_at(x, z) == Some(home);
        let rooms: Vec<&RoomInfo> = w.rooms.iter().filter(|r| r.plot == Some(home)).collect();
        let broken: Vec<usize> = w
            .objects
            .iter()
            .zip(&w.object_plot)
            .filter(|(o, p)| o.broken() && **p == Some(home))
            .map(|(o, _)| o.def)
            .collect();
        // Wishes that came true.
        let good = |kind: Option<usize>, garden: bool, factor: u8| {
            rooms.iter().any(|r| {
                r.garden == garden && r.kind == kind && r.scores.factors()[factor as usize] >= 0.6
            })
        };
        let kept: Vec<HomeWish> = w.sims[i]
            .planner
            .home_wishes
            .iter()
            .copied()
            .filter(|wish| match *wish {
                HomeWish::Room {
                    kind,
                    garden,
                    factor,
                } => !good(kind, garden, factor),
                HomeWish::Fix { def } => broken.contains(&def),
                HomeWish::Another { kind } => {
                    rooms
                        .iter()
                        .filter(|r| !r.garden && r.kind == Some(kind))
                        .count()
                        < 2
                        && s.planner
                            .waited_day
                            .is_some_and(|d| today <= d + WAIT_FORGET_DAYS)
                }
            })
            .collect();
        let mut wishes = kept;
        let mut waited = w.sims[i].planner.waited_day;
        let mut thought: Option<(ThoughtKind, usize)> = None;
        let mut feeling = None;
        if at_home {
            let rules = &w.content.room_rules;
            let roll = w.rng.next_f32();
            if let Some(room) = w.room_at_tile(x, z) {
                let s = room.scores;
                if s.overall >= rules.love && roll < 0.3 {
                    thought = Some((ThoughtKind::RoomLoved, room_subject(room, 0)));
                    feeling = rules.love_feeling;
                } else if s.overall <= rules.dislike && roll < 0.5 {
                    let factor = s.weakest();
                    thought = Some((ThoughtKind::RoomDisliked, room_subject(room, factor)));
                    feeling = rules.dislike_feeling;
                    let wish = HomeWish::Room {
                        kind: room.kind,
                        garden: room.garden,
                        factor: factor as u8,
                    };
                    if !wishes.contains(&wish) {
                        wishes.push(wish);
                    }
                }
            }
            // Something broke: they'd like it fixed.
            for &def in &broken {
                let wish = HomeWish::Fix { def };
                if !wishes.contains(&wish) {
                    wishes.push(wish);
                    thought = Some((ThoughtKind::Broken, def));
                }
            }
            // Badly needing what's taken (the toilet): another room for it, where content asks.
            let me = i as u32;
            for acc in &w.content.accidents {
                let Some(kind) = acc.another_room else {
                    continue;
                };
                let have = rooms
                    .iter()
                    .filter(|r| !r.garden && r.kind == Some(kind))
                    .count();
                if w.sims[i].needs[acc.need] >= 0.15 || have >= 2 {
                    continue;
                }
                let fillers: Vec<&crate::world::ObjectInstance> = w
                    .objects
                    .iter()
                    .zip(&w.object_plot)
                    .filter(|(o, p)| {
                        **p == Some(home)
                            && !o.broken()
                            && w.content.objects[o.def]
                                .interactions
                                .iter()
                                .any(|it| it.autonomous && it.total_gain[acc.need] > 0.0)
                    })
                    .map(|(o, _)| o)
                    .collect();
                // (Taken, or in a room someone else keeps shut: the bathroom with someone in it.)
                let here = w.lot.room_at(x, z);
                let shut = |o: &crate::world::ObjectInstance| {
                    let room = crate::privacy::room_of_object(&w.content, &w.lot, o);
                    room != here && crate::privacy::shut_to(&w.shut, me, None, room)
                };
                let busy = !fillers.is_empty()
                    && fillers.iter().all(|o| {
                        (o.users().any(|u| u != me)
                            && !o.has_free_slot(w.content.objects[o.def].slots))
                            || shut(o)
                    });
                if busy {
                    // Once is bad luck; again on another day this week, they'd like a second one.
                    let again = waited.is_some_and(|d| d < today && today <= d + WAIT_REPEAT_DAYS);
                    let wish = HomeWish::Another { kind };
                    if again && !wishes.contains(&wish) {
                        wishes.push(wish);
                    }
                    waited = Some(today);
                }
            }
        }
        while wishes.len() > MAX_HOME_WISHES {
            wishes.remove(0);
        }
        let content = &w.content;
        let sim = &mut w.sims[i];
        sim.planner.home_wishes = wishes;
        sim.planner.waited_day = waited;
        if let Some((kind, subject)) = thought
            && sim.planner.thought.is_none()
        {
            think(sim, kind, subject, tick);
        }
        if let Some(f) = feeling {
            crate::social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        }
    }
}
