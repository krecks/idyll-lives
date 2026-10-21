//! World state and the per-tick update.

use std::collections::{BTreeMap, VecDeque};
use std::f32::consts::FRAC_PI_2;

use crate::clock::{AUTO_FAST_SPEED, MAX_SPEED, TICKS_PER_STEP};
use crate::content::{Content, HabitTime, Layer, MAX_NEEDS, MAX_SKILLS, MAX_SLOTS, Modifiers, Pose, TagMask, stage_in};
use crate::index::ObjectIndex;
use crate::lot::{BondRaw, Lot, LotFile, SimSpawn};
use crate::path::{Grounds, NavGrid};
use crate::rng::Rng;
use crate::social::{self, ActiveFeeling, EventLog, Prefer, Relationships};
use crate::{
    Command, Error, MINUTES_PER_TICK, TICKS_PER_MINUTE, TICKS_PER_SECOND, ai, conversation,
};
use serde::{Deserialize, Serialize};

pub const MAX_SIMS: usize = 64;
pub const MAX_QUEUE: usize = 6;
/// How many things a resident remembers they couldn't get to (`Sim::unreachable`).
pub(crate) const UNREACHABLE: usize = 4;
/// How long a resident remembers that someone was out of reach: people move, so not long.
pub(crate) const PERSON_UNREACHABLE_TICKS: u64 = 10;
/// Walking speed in tiles per tick (1.5 m per real second at 1×).
pub(crate) const WALK_SPEED: f32 = 1.5 / TICKS_PER_SECOND as f32;
/// Idle time before a Sim picks something on its own.
const AUTONOMY_DELAY_TICKS: u32 = TICKS_PER_SECOND;
const MIN_AUTONOMY_SCORE: f32 = 0.01;
/// A baby with a need below this cries for someone (and lies awake in the crib).
const BABY_CRIES_BELOW: f32 = 0.3;
pub const TICKS_PER_DAY: u64 = crate::clock::TICKS_PER_DAY;
const TICKS_PER_HOUR: u64 = 60 * TICKS_PER_MINUTE;
/// Rooms are re-scored this often (for dirt), and whenever the structure changes.
pub(crate) const ROOM_REFRESH_TICKS: u64 = 10 * TICKS_PER_MINUTE;
/// How far (centimetres, per axis) an object can stand off the middle of its footprint: half a
/// tile, so its middle stays on its tile (see `ObjectInstance::offset`).
pub const MAX_OFFSET: i8 = 50;

/// No offset (for leaving it out of saves and views).
pub(crate) fn centred(offset: &[i8; 2]) -> bool {
    *offset == [0, 0]
}

/// The most tiles a resized object may take along either side.
pub const MAX_SIZE: u8 = 6;
/// The least a resized object may measure across or deep (centimetres).
pub const MIN_SIZE: u16 = 10;
/// How much taller (centimetres) a resized object may be than its kind, and how much shorter.
pub const MAX_TALLER: i16 = 200;
pub const MAX_SHORTER: i16 = 100;

/// The tiles an object of `size` (centimetres across and deep) takes: rounded up.
pub fn tiles_for(size: [u16; 2]) -> [u8; 2] {
    size.map(|s| s.div_ceil(100).max(1) as u8)
}

/// A kind's footprint in centimetres (its own size before any resizing).
pub fn kind_size(footprint: [u8; 2]) -> [u16; 2] {
    footprint.map(|t| t as u16 * 100)
}

/// How an object is laid out on its tiles beyond where and which way it stands: off their
/// middle, and resized (see `ObjectInstance::offset`, `extent` and `tall`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Layout {
    pub offset: [i8; 2],
    /// Its size (centimetres across and deep, before turning); None: its kind's.
    pub extent: Option<[u16; 2]>,
    pub tall: i16,
}

impl Layout {
    /// Standing `offset` off its tiles' middle, its kind's size.
    pub fn at(offset: [i8; 2]) -> Self {
        Layout { offset, ..Layout::default() }
    }

    /// The layout `o` has now.
    pub fn of(o: &ObjectInstance) -> Self {
        Layout { offset: o.offset, extent: Some(o.extent), tall: o.tall }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectInstance {
    /// Equal to the object's index in `World::objects` (internal; views and commands use `stable`).
    pub id: u32,
    /// The object's id for its whole life: it never changes when other objects are sold, moved or
    /// reloaded. `World::index_of` turns it back into an index.
    pub stable: u32,
    pub def: usize,
    /// Minimum corner of the rotated footprint.
    pub x: i32,
    pub z: i32,
    /// Facing: 0 = +z, 1 = +x, 2 = -z, 3 = -x.
    pub rot: u8,
    /// Degrees turned past `rot` (0..90), for objects that turn freely; only for looks.
    pub turn: u8,
    /// Centimetres (x, z) its middle stands off the middle of its footprint (within
    /// `MAX_OFFSET`), for anything but stairs, things on a wall only along it: they can stand
    /// anywhere on their tiles. The footprint stays on tiles; its body (`extent` about its
    /// middle) may reach past them, but no wall may run through it (`body_edges`, `body_crosses`).
    pub offset: [i8; 2],
    /// Tiles it takes across and deep, before turning: its `extent` rounded up to whole tiles
    /// (`MAX_SIZE` at most).
    pub footprint: [u8; 2],
    /// Its own size, centimetres across and deep before turning: its kind's footprint unless the
    /// player resized it. It stands in the middle of its tiles (moved by `offset`).
    pub extent: [u16; 2],
    /// Centimetres taller (negative: shorter) than its kind, the player's resizing: tops and
    /// seats rise with it.
    pub tall: i16,
    /// Sims using or walking to this object, one per slot.
    pub users: [Option<u32>; MAX_SLOTS],
    /// Upgrade level: higher quality objects satisfy needs and teach skills faster.
    pub quality: u8,
    /// The colours the player gave its parts (visual only): see `Colors`. Empty: as it's made.
    pub colors: Colors,
    /// Money put into this object (price plus upgrades), for resale.
    pub value: i64,
    /// Wear from use, 0..1; at 1 it is broken until repaired.
    pub wear: f32,
    /// Open (a wall bed folded down, a door or lid open): set by the interactions that open or
    /// close it (`Interaction::leaves`), and what some are offered for (`Interaction::when`).
    pub open: bool,
    /// A crop's growth (its kind `grows`; see `GrowthStage`): -1..0 lying fallow after it was
    /// picked, 0..1 planted and growing, 1 ripe. Bought ripe. Meaningless for other things.
    pub growth: f32,
    /// A lamp switched off (see `lights.rs`); bought on. Meaningless for other things.
    pub off: bool,
    /// Residents whose current use needs it closed (bits by sim id), so opening it under them is
    /// refused (`allows`). Derived from their tasks: set when the use reserves a slot, cleared
    /// when it ends; not saved (a reloaded use starts again and sets it).
    pub(crate) closed_users: u64,
}

/// A crop's stage, from its `growth`: what's offered (`when`: "ripe") and what the renderer
/// shows (the model's `motions.growth` clip). The numbers are the view's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrowthStage {
    Planted = 0,
    Growing = 1,
    Ripe = 2,
    Picked = 3,
}

/// Share of the way to ripe at which a crop shows as growing rather than just planted.
const GROWING_FROM: f32 = 1.0 / 3.0;
/// Game minutes between crops' growth steps.
const GROWTH_STEP_MINUTES: u64 = 15;

/// Something that happened while a resident stepped, for the story (applied after the step).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum News {
    Broke(u32),
    Repaired(u32),
    /// An accident (index into `Content::accidents`).
    Accident(usize),
    /// The player's order to close this object was dropped: someone else is in it now.
    NotClosed(u32),
}

impl ObjectInstance {
    /// Worn out: its interactions don't work until it is repaired.
    pub fn broken(&self) -> bool {
        self.wear >= 1.0
    }

    /// Whether its state lets resident `me` do `inter` now: open or closed as the interaction
    /// wants (`when`), and nothing closed on, or opened under, someone else using it (or on their
    /// way to). Opening is refused only when another occupant's use needs it to stay closed.
    pub fn allows(&self, inter: &crate::content::Interaction, me: u32) -> bool {
        self.fits(inter)
            && (!inter.ripe || self.ripe())
            && (inter.leaves != Some(false) || self.users().all(|u| u == me))
            && (inter.leaves != Some(true) || self.closed_users & !(1 << me) == 0)
    }

    /// Whether `inter` would close it on someone other than `me` using it (or on their way to):
    /// the one reason `allows` says no that the player hears of when it comes up after an order
    /// (an order to open or close what someone else already did is done).
    fn closes_on_someone(&self, inter: &crate::content::Interaction, me: u32) -> bool {
        inter.leaves == Some(false) && self.fits(inter) && self.users().any(|u| u != me)
    }

    /// Whether it's open or closed as `inter` wants (`when`).
    fn fits(&self, inter: &crate::content::Interaction) -> bool {
        inter.when.is_none_or(|w| w == self.open)
    }

    /// Its crop is ripe (see `growth`).
    pub fn ripe(&self) -> bool {
        self.growth >= 1.0
    }

    /// Its crop's stage (for a kind that `grows`).
    pub fn growth_stage(&self) -> GrowthStage {
        match self.growth {
            g if g < 0.0 => GrowthStage::Picked,
            g if g >= 1.0 => GrowthStage::Ripe,
            g if g >= GROWING_FROM => GrowthStage::Growing,
            _ => GrowthStage::Planted,
        }
    }

    pub fn users(&self) -> impl Iterator<Item = u32> + '_ {
        self.users.iter().flatten().copied()
    }

    pub fn in_use(&self) -> bool {
        self.users.iter().any(Option::is_some)
    }

    /// Whether `sim` can use this object (it has a free slot, or `sim` already holds one).
    pub fn can_use(&self, sim: u32, slots: u8) -> bool {
        self.can_use_in(sim, 0..slots as usize)
    }

    /// Whether one of the slots in `range` is free for `sim` (or already theirs).
    pub fn can_use_in(&self, sim: u32, range: std::ops::Range<usize>) -> bool {
        self.users[range].iter().any(|u| u.is_none_or(|u| u == sim))
    }

    /// Whether `sim` can start interaction `i` of `def` here: one of its slots
    /// (`ObjectDef::slots_for`) free or theirs already; all of them for one that takes the whole
    /// object (`Interaction::whole`).
    pub fn can_use_for(&self, sim: u32, def: &crate::content::ObjectDef, i: usize) -> bool {
        let range = def.slots_for(i);
        if def.interactions[i].whole {
            self.users[range].iter().all(|u| u.is_none_or(|u| u == sim))
        } else {
            self.can_use_in(sim, range)
        }
    }

    pub fn has_free_slot(&self, slots: u8) -> bool {
        self.users[..slots as usize].iter().any(Option::is_none)
    }

    pub(crate) fn slot_of(&self, sim: u32) -> Option<usize> {
        self.users.iter().position(|u| *u == Some(sim))
    }

    /// Takes a slot for `sim` (keeps the one it already has).
    pub(crate) fn reserve(&mut self, sim: u32, slots: u8) -> bool {
        self.reserve_in(sim, 0..slots as usize)
    }

    /// Takes one of the slots in `range` for `sim` (keeps the one it already has).
    pub(crate) fn reserve_in(&mut self, sim: u32, range: std::ops::Range<usize>) -> bool {
        self.reserve_near(sim, range, None)
    }

    /// Takes the slot `pick_slot` gives `sim` in `range` (`near`: see there).
    fn reserve_near(&mut self, sim: u32, range: std::ops::Range<usize>, near: Option<(&crate::content::ObjectDef, [f32; 2])>) -> bool {
        if self.slot_of(sim).is_some() {
            return true;
        }
        match self.pick_slot(sim, range, near) {
            Some(k) => {
                self.users[k] = Some(sim);
                true
            }
            None => false,
        }
    }

    /// The slot in `range` `sim` gets: theirs already, else a free one; of a kind whose slots
    /// have their places (`near`: its def, and where `sim` is now), the one nearest them, else
    /// the first.
    pub(crate) fn pick_slot(&self, sim: u32, range: std::ops::Range<usize>, near: Option<(&crate::content::ObjectDef, [f32; 2])>) -> Option<usize> {
        if let Some(k) = self.slot_of(sim) {
            return Some(k);
        }
        let mut free = range.filter(|&k| self.users[k].is_none());
        match near.filter(|(def, _)| !def.places.is_empty()) {
            Some((def, from)) => free.min_by(|&a, &b| {
                let d = |k| {
                    let p = self.place_at(def, k);
                    (p[0] - from[0]).hypot(p[1] - from[1])
                };
                d(a).total_cmp(&d(b))
            }),
            None => free.next(),
        }
    }

    /// Takes a place for `sim` (now at `from`) for interaction `i` of `def` (see `can_use_for`):
    /// one of its slots (`pick_slot`), or every one for a use that takes the whole object
    /// (`slot_of` is then the first).
    pub(crate) fn reserve_for(&mut self, sim: u32, def: &crate::content::ObjectDef, i: usize, from: [f32; 2]) -> bool {
        let range = def.slots_for(i);
        let reserved = if def.interactions[i].whole {
            if !self.can_use_for(sim, def, i) {
                return false;
            }
            self.users[range].fill(Some(sim));
            true
        } else {
            self.reserve_near(sim, range, Some((def, from)))
        };
        self.kept_closed(sim, reserved, def.interactions[i].when == Some(false))
    }

    /// The slot `reserve_for` would give `sim` (at `from`) for interaction `i` of `def`.
    pub(crate) fn slot_for(&self, sim: u32, def: &crate::content::ObjectDef, i: usize, from: [f32; 2]) -> Option<usize> {
        let range = def.slots_for(i);
        if def.interactions[i].whole {
            return Some(range.start);
        }
        self.pick_slot(sim, range, Some((def, from)))
    }

    /// Takes a place on it for `sim` (now at `from`) to sit while doing something else (eating,
    /// the phone, TV from the sofa). A seat only while closed (a sofa bed) then can't be opened
    /// under them.
    pub(crate) fn reserve_seat(&mut self, sim: u32, def: &crate::content::ObjectDef, from: [f32; 2]) -> bool {
        let reserved = self.reserve_near(sim, 0..def.slots as usize, Some((def, from)));
        self.kept_closed(sim, reserved, def.interactions.iter().any(|i| i.pose == Pose::Sit && i.when == Some(false)))
    }

    /// Passes `reserved` on; a place taken for a use that needs it `closed` keeps it so under `sim`.
    fn kept_closed(&mut self, sim: u32, reserved: bool, closed: bool) -> bool {
        if reserved && closed {
            self.closed_users |= 1 << sim;
        }
        reserved
    }

    /// The tags of each way someone past the crib, in life stage `stage`, can sit on it as it is
    /// now (a sofa bed pulled out is no seat; a kids' chair is none for a grown-up).
    pub fn seats_now<'a>(&'a self, def: &'a crate::content::ObjectDef, stage: usize) -> impl Iterator<Item = TagMask> + 'a {
        def.interactions.iter().filter(move |i| i.pose == Pose::Sit && !i.baby && self.fits(i) && i.for_stage(stage)).map(|i| i.tags)
    }

    /// Whether it is a seat now for someone in `stage` with a way to sit that has one of `want`'s
    /// tags and none of `avoid`'s.
    pub fn sits(&self, def: &crate::content::ObjectDef, stage: usize, want: TagMask, avoid: TagMask) -> bool {
        self.seats_now(def, stage).any(|t| t & want != 0 && t & avoid == 0)
    }

    /// Takes on the state `old`, this object before it was lifted out and placed again, had: its
    /// stable id, turn, wear, open or closed, on or off, and crop.
    pub(crate) fn carry_state(&mut self, old: &ObjectInstance) {
        (self.stable, self.turn, self.wear, self.open, self.off, self.growth) = (old.stable, old.turn, old.wear, old.open, old.off, old.growth);
    }

    pub(crate) fn release(&mut self, sim: u32) {
        self.closed_users &= !(1 << sim);
        for u in &mut self.users {
            if *u == Some(sim) {
                *u = None;
            }
        }
    }

    /// Where a Sim sits, lies or stands in `slot`: at its place (content `places`), else spread
    /// across the object's width, in the middle of its depth.
    pub fn slot_position(&self, content: &Content, slot: usize) -> [f32; 2] {
        self.place_at(&content.objects[self.def], slot)
    }

    /// `slot_position`, for its kind `def`.
    fn place_at(&self, def: &crate::content::ObjectDef, slot: usize) -> [f32; 2] {
        // (Where it stands on its tiles, as it's drawn.)
        let (w, d) = self.turned_footprint();
        let c = [
            self.x as f32 + w as f32 / 2.0 + self.offset[0] as f32 / 100.0,
            self.z as f32 + d as f32 / 2.0 + self.offset[1] as f32 / 100.0,
        ];
        if let Some(p) = def.places.get(slot) {
            // Its place, turned with it (a freely turned one too: the place is on its model) and
            // moved out with its size.
            let a = self.angle();
            let [x, z] = [0, 1].map(|k| p.at[k] * self.extent[k] as f32 / 100.0 / def.footprint[k].max(1) as f32);
            return [c[0] + a.cos() * x + a.sin() * z, c[1] - a.sin() * x + a.cos() * z];
        }
        let n = def.slots.max(1) as f32;
        // Bunks lie one above the other: both in the middle (a hair apart, so the renderer can
        // tell the lower bunk from the upper one).
        let spread = if def.bunk { 0.04 } else { self.extent[0] as f32 / 100.0 / n };
        let offset = (slot as f32 - (n - 1.0) / 2.0) * spread;
        let yaw = self.yaw();
        // Local +x (the object's right) after rotation.
        [c[0] + yaw.cos() * offset, c[1] - yaw.sin() * offset]
    }

    /// Which way a Sim in `slot` faces (`Sim::yaw`): its place's facing (content `places`), else
    /// the way the object faces.
    pub fn slot_yaw(&self, content: &Content, slot: usize) -> f32 {
        match content.objects[self.def].places.get(slot) {
            Some(p) => self.angle() + p.facing,
            None => self.yaw(),
        }
    }

    /// Which way it faces, its free turn included (radians; see `yaw`).
    pub fn angle(&self) -> f32 {
        self.yaw() + (self.turn as f32).to_radians()
    }
    /// Rotated footprint `(x extent, z extent)`.
    pub fn size(&self, _content: &Content) -> (i32, i32) {
        self.turned_footprint()
    }

    fn turned_footprint(&self) -> (i32, i32) {
        let [w, d] = self.footprint;
        if self.rot.is_multiple_of(2) {
            (w as i32, d as i32)
        } else {
            (d as i32, w as i32)
        }
    }

    pub fn centre(&self, content: &Content) -> [f32; 2] {
        let (w, d) = self.size(content);
        [
            self.x as f32 + w as f32 / 2.0,
            self.z as f32 + d as f32 / 2.0,
        ]
    }

    pub fn yaw(&self) -> f32 {
        (self.rot % 4) as f32 * FRAC_PI_2
    }

    /// Tile a Sim stands on to use the object: centred in front of the footprint.
    pub fn front_tile(&self, content: &Content) -> (i32, i32) {
        let (w, d) = self.size(content);
        match self.rot % 4 {
            0 => (self.x + (w - 1) / 2, self.z + d),
            1 => (self.x + w, self.z + (d - 1) / 2),
            2 => (self.x + (w - 1) / 2, self.z - 1),
            _ => (self.x - 1, self.z + (d - 1) / 2),
        }
    }

    /// The grid edges along its back (`(horizontal, x, z)`: an `h` or a `v` edge), where a thing
    /// on a wall hangs.
    pub(crate) fn back_edges(&self, content: &Content) -> Vec<(bool, i32, i32)> {
        let (w, d) = self.size(content);
        let (x, z) = (self.x, self.z);
        match self.rot % 4 {
            0 => (x..x + w).map(|tx| (true, tx, z)).collect(),
            2 => (x..x + w).map(|tx| (true, tx, z + d)).collect(),
            1 => (z..z + d).map(|tz| (false, x, tz)).collect(),
            _ => (z..z + d).map(|tz| (false, x + w, tz)).collect(),
        }
    }

    pub(crate) fn tiles(&self, content: &Content) -> impl Iterator<Item = (i32, i32)> + use<> {
        let (w, d) = self.size(content);
        let (x0, z0) = (self.x, self.z);
        (0..d).flat_map(move |dz| (0..w).map(move |dx| (x0 + dx, z0 + dz)))
    }

    /// Its body's sides, `(x0, x1, z0, z1)` in half centimetres (tile lines every 200): its own
    /// size (`extent`, turned) about its middle, `offset` off the middle of its footprint.
    fn body_box(&self, content: &Content) -> (i32, i32, i32, i32) {
        let (w, d) = self.size(content);
        let [ex, ez] = if self.rot.is_multiple_of(2) { self.extent } else { [self.extent[1], self.extent[0]] };
        let span = |at: i32, tiles: i32, off: i8, size: u16| {
            let mid = 2 * (at * 100 + off as i32) + tiles * 100;
            (mid - size as i32, mid + size as i32)
        };
        let (x0, x1) = span(self.x, w, self.offset[0], ex);
        let (z0, z1) = span(self.z, d, self.offset[1], ez);
        (x0, x1, z0, z1)
    }

    /// The tiles its body overlaps (more than touches): its footprint's, and those it reaches
    /// into off the footprint's middle.
    pub(crate) fn body_tiles(&self, content: &Content) -> impl Iterator<Item = (i32, i32)> + use<> {
        let (x0, x1, z0, z1) = self.body_box(content);
        let (xs, zs) = (x0.div_euclid(200)..(x1 + 199).div_euclid(200), z0.div_euclid(200)..(z1 + 199).div_euclid(200));
        zs.flat_map(move |tz| xs.clone().map(move |tx| (tx, tz)))
    }

    /// Whether a diagonal wall across tile `(tx, tz)` would run through its body (touching its
    /// side or corner is fine, as a wall on a tile line along its side).
    pub(crate) fn body_crosses(&self, content: &Content, tx: i32, tz: i32, dir: crate::lot::DiagDir) -> bool {
        let (x0, x1, z0, z1) = self.body_box(content);
        // The part of the body on that tile.
        let (tx0, tz0) = (tx * 200, tz * 200);
        let (a0, a1) = (x0.max(tx0), x1.min(tx0 + 200));
        let (b0, b1) = (z0.max(tz0), z1.min(tz0 + 200));
        if a0 >= a1 || b0 >= b1 {
            return false;
        }
        match dir {
            // `/`: x - z is that of the corner (x, z) all along it; strictly between the least
            // and the most of the body's, it runs through.
            crate::lot::DiagDir::Dp => (a0 - b1 + 1..a1 - b0).contains(&(tx0 - tz0)),
            // `\`: x + z is that of the corners (x, z + 1) and (x + 1, z).
            crate::lot::DiagDir::Dn => (a0 + b0 + 1..a1 + b1).contains(&(tx0 + tz0 + 200)),
        }
    }

    /// Pairs of neighbouring tiles its body stands across: off the middle of its footprint
    /// (`offset`), it can reach past the footprint's side, and no wall may run through it there.
    pub(crate) fn body_edges(&self, content: &Content) -> Vec<((i32, i32), (i32, i32))> {
        let (x0, x1, z0, z1) = self.body_box(content);
        // The tiles it overlaps, and the tile lines strictly inside it.
        let tiles = |lo: i32, hi: i32| lo.div_euclid(200)..(hi + 199).div_euclid(200);
        let lines = |lo: i32, hi: i32| lo.div_euclid(200) + 1..=(hi - 1).div_euclid(200);
        let mut edges = Vec::new();
        for lx in lines(x0, x1) {
            edges.extend(tiles(z0, z1).map(|tz| ((lx - 1, tz), (lx, tz))));
        }
        for lz in lines(z0, z1) {
            edges.extend(tiles(x0, x1).map(|tx| ((tx, lz - 1), (tx, lz))));
        }
        edges
    }

    /// Pairs of neighbouring tiles that must not have a wall between them: inside the
    /// footprint, and from the footprint to the front tile.
    pub(crate) fn wall_edges(
        &self,
        content: &Content,
    ) -> impl Iterator<Item = ((i32, i32), (i32, i32))> + use<> {
        let (w, d) = self.size(content);
        let (x0, z0) = (self.x, self.z);
        let inside = (0..d).flat_map(move |dz| {
            (0..w).flat_map(move |dx| {
                let t = (x0 + dx, z0 + dz);
                let right = (dx + 1 < w).then_some((t, (t.0 + 1, t.1)));
                let down = (dz + 1 < d).then_some((t, (t.0, t.1 + 1)));
                right.into_iter().chain(down)
            })
        });
        let front = self.front_tile(content);
        let (bx, bz) = match self.rot % 4 {
            0 => (front.0, front.1 - 1),
            1 => (front.0 - 1, front.1),
            2 => (front.0, front.1 + 1),
            _ => (front.0 + 1, front.1),
        };
        inside.chain(std::iter::once(((bx, bz), front)))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TaskKind {
    Use {
        object: u32,
        interaction: usize,
    },
    MoveTo {
        x: f32,
        z: f32,
    },
    /// Start a social interaction with another Sim.
    Social {
        target: u32,
        social: usize,
    },
    /// Join the conversation `leader` started and listen (a group conversation; see
    /// `conversation.rs`): `Routing` on the way, `Conversing` while listening, `Waiting` between
    /// two of the leader's socials.
    Join {
        leader: u32,
    },
    /// Head to the town exit for a work shift.
    Work,
    /// Walk to another household's front door and stay a while.
    Visit {
        plot: u32,
    },
    /// Walk back home.
    GoHome,
    /// Go to the front door to answer a visitor's knock (`visits.door`).
    Answer {
        guest: u32,
    },
    /// Tidy up around tile `(x, z)` (`roomRules.clean`).
    Clean {
        x: i32,
        z: i32,
    },
    /// Repair a broken object (`objectRules.repair`).
    Repair {
        object: u32,
    },
    /// An accident's little while on the spot (`Content::accidents`): asleep on the floor,
    /// eating takeout.
    Spot {
        accident: usize,
    },
    /// Cooking a recipe (`cooking.rs`; index into `Content::recipes`): at step `step`, at
    /// `station` once one is found, started from `origin` (the object whose interaction it was;
    /// its skills and feeling count). `servings` is 0 until the ingredients are paid for (at
    /// the first step), then how many it makes; `skipped` counts steps done without their
    /// station; `minutes` is how long the current step takes (until it starts, the minutes of
    /// the steps just done without, carried over to it).
    Cook {
        recipe: usize,
        step: u8,
        station: Option<u32>,
        origin: Option<u32>,
        servings: u8,
        skipped: u8,
        minutes: u16,
    },
    /// Eating: first fetching a serving of `item` (none: they hold their plate already), then
    /// at `seat` (none: standing), the plate put down there as item `plate` (none: in hand).
    Eat {
        item: Option<u32>,
        seat: Option<u32>,
        plate: Option<u32>,
        stage: u8,
    },
    /// Clearing away: picking up `item` and dirty things next to it (stage 0), scraping the
    /// food off `scraps` of them into a bin (1), washing up (2) and putting them away (3), each
    /// at `station`.
    Dishes {
        item: u32,
        station: Option<u32>,
        stage: u8,
        scraps: u8,
    },
    /// On the phone (`phone.rs`; index into `content.phone.actions`): with `contact`, if the
    /// action is with someone; on `seat` once one is found (none: standing); `answer`: how the
    /// other end took it.
    Phone {
        action: usize,
        contact: Option<u32>,
        seat: Option<u32>,
        answer: crate::phone::Answer,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Task {
    pub kind: TaskKind,
    /// Issued by the player (as opposed to chosen autonomously).
    pub directed: bool,
    /// Done as part of a routine (washing hands after the toilet; see `Interaction::after`):
    /// it takes its full time even with the needs it fills full.
    pub routine: bool,
}

#[derive(Debug, Clone)]
pub enum Phase {
    Routing {
        waypoints: Vec<[f32; 2]>,
        next: usize,
    },
    Using {
        elapsed: f32,
    },
    /// Calling out the social target's name from where they stand (they're within earshot); the
    /// two then walk to meet halfway (see `conversation::update`).
    Calling {
        ticks: u32,
    },
    /// Next to the social target, waiting for them to be free.
    Waiting {
        ticks: u32,
    },
    /// In a conversation; the outcome was decided when it started.
    Conversing {
        elapsed: f32,
        success: bool,
    },
    /// Waiting for a busy station (a cook or someone washing up; see `cooking.rs`).
    Queued {
        ticks: u32,
    },
    /// A visitor at the front door, waiting to be let in (`visits.door`, run by `life`).
    Knocking {
        ticks: u32,
        /// The resident on their way to answer.
        host: Option<u32>,
    },
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub task: Task,
    pub phase: Phase,
    /// Tags of the interaction or social being performed (for "busy" checks and autonomy).
    pub tags: TagMask,
}

#[derive(Debug, Clone)]
pub struct Sim {
    /// Equal to the Sim's index in `World::sims`.
    pub id: u32,
    pub name: String,
    /// Opaque appearance data for the renderer/UI.
    pub appearance: serde_json::Value,
    pub gender: String,
    /// Genders this Sim is romantically attracted to.
    pub attracted_to: Vec<String>,
    pub traits: Vec<String>,
    pub perks: Vec<String>,
    /// Combined effect of traits and perks.
    pub base_mods: Modifiers,
    /// Effective modifiers: `base_mods` combined with the effects of active feelings.
    pub mods: Modifiers,
    /// Feelings (defs, sorted) whose effects are folded into `mods`.
    pub(crate) buffs: Vec<usize>,
    /// Money the household owes for interactions started this tick (settled by `life::update`).
    pub(crate) pending_spend: i64,
    pub pos: [f32; 2],
    pub yaw: f32,
    pub pose: Pose,
    /// The tile they sat or lay down from (where they stand back up): the tile an object is
    /// used from can change while they're on it (see `surface::approach_tile`).
    pub(crate) seated_from: Option<(i32, i32)>,
    /// The seat they took to use another object (content `seatNear`: the sofa while watching
    /// TV). Not saved: the task finds a seat again after a load.
    pub use_seat: Option<u32>,
    pub needs: [f32; MAX_NEEDS],
    pub feelings: Vec<ActiveFeeling>,
    /// Set while another Sim is talking to this one.
    pub engaged_with: Option<u32>,
    /// Someone called out their name: they turned to them and are on their way to meet them, or
    /// wait for them where they met (see `conversation::update`). Not saved.
    pub heeding: Option<u32>,
    pub(crate) gender_ix: Option<usize>,
    pub(crate) attraction: u32,
    pub(crate) queue: VecDeque<Task>,
    pub(crate) current: Option<Activity>,
    pub(crate) idle_ticks: u32,
    /// What they do next, before choosing anything else (an `after` routine, like washing
    /// hands after the toilet; see `follow_up`). Not saved: losing it on a reload is harmless.
    pub(crate) follow_up: Option<Task>,
    /// The last few things they chose for themselves but couldn't get to (a plate on a counter
    /// nobody can reach, a bed behind a wall, a house across a river), each with until when they
    /// don't choose it again: uses, eating, cleaning, repairs, washing up, visits, and people
    /// for a short while (see `remember_unreachable`; object ids follow `remap_unreachable`).
    /// Not saved either.
    pub(crate) unreachable: [Option<(TaskKind, u64)>; UNREACHABLE],
    /// Ticks the social they're on has spent waiting for its target, and following them since
    /// it first got there (see `conversation::update`). Not saved.
    pub(crate) social_waited: u32,
    /// They called out to the target of the social they're on already (once per social). Not saved.
    pub(crate) called: bool,
    /// The `World::nav_version` their route was last checked against (see `step_sim`): a
    /// resident nobody watches takes a turn only every so often, and a change between two of
    /// them still catches up with them. Not saved (after a load, a walker routes again once).
    pub(crate) nav_seen: u32,
    pub household: u32,
    pub job: Option<crate::life::Job>,
    /// At work until this tick (not on the map).
    pub away_until: Option<u64>,
    /// Currently a guest on another household's plot.
    pub visiting: Option<crate::life::Visit>,
    /// Set by Work/Visit/GoHome tasks; handled by `life::update`.
    pub(crate) transition: Option<crate::life::Transition>,
    /// Skill levels, `0..=max_level` (the whole number is the level shown to players).
    pub skills: [f32; MAX_SKILLS],
    /// Skills that reached a new level this tick (bitmask), reported by `life::update`.
    pub(crate) skill_ups: u32,
    /// Grows while practising and fades otherwise, so Sims don't train for hours on end.
    pub(crate) practice_fatigue: f32,
    /// No nap until this tick (the last one was too recent; see `DayRhythm::nap_cooldown_hours`).
    pub(crate) nap_until: u64,
    /// The first day this Sim looks for work again (after quitting or being let go).
    pub job_search_from: u32,
    /// Routines, goals and how planned blocks went (see `planner`).
    pub planner: crate::planner::Planner,
    /// Mess made (or, negative, cleaned) around a tile this tick; applied after the step
    /// (residents can't change the dirt while stepping).
    pub(crate) mess: Option<(i32, i32, f32)>,
    /// Conceived with this partner this tick (the life cycle decides; see `lifecycle::conceive`).
    pub(crate) conceived: Option<u32>,
    /// Care given this tick: the baby it's for and what it fills (applied after the step).
    pub(crate) care_out: Option<(u32, [f32; MAX_NEEDS])>,
    /// Opened or closed an object this tick (`Interaction::leaves`): how the house looks
    /// changed (applied after the step).
    pub(crate) reshaped: bool,
    /// When they last woke someone crying (a baby), or were woken by a crying baby (a
    /// grown-up), as a tick (0: never; see `World::wake_for_crying_babies`).
    pub(crate) night_woken: u64,
    /// An object they broke or repaired, or an accident they had, this tick (applied after
    /// the step).
    pub(crate) news: Option<News>,
    /// When each accident can happen again (`Content::accidents`).
    pub(crate) accident_at: [u64; crate::content::MAX_ACCIDENTS],
    /// Since when each accident's need has been empty (0: it isn't).
    pub(crate) empty_since: [u64; crate::content::MAX_ACCIDENTS],
    /// Age in years (see `lifecycle.rs`); the life stage follows from it.
    pub age: f32,
    /// No longer in town (the slot stays, so ids don't change; see `World::depart`).
    pub gone: Option<Gone>,
    /// What they carry (ingredients, a plate, dirty dishes; see `cooking.rs`).
    pub held: Option<crate::items::Held>,
    /// Told dinner's ready: keener to eat until this tick.
    pub(crate) invited_until: u64,
    /// A dish they served this tick (an item; the household is told after the step).
    pub(crate) served: Option<u32>,
    /// A phone action that ended this tick (applied after the step, see `phone::update`).
    pub(crate) hung_up: Option<crate::phone::HungUp>,
    /// The last day they went to (or skipped) school.
    pub school_day: u32,
    /// How they do at school, 0..100 (pupils; see `SchoolRules::letter`).
    pub grade: f32,
    /// Retired from work: doesn't look for a job; the household gets `pension` a week.
    pub retired: bool,
    pub pension: i64,
    /// Ticks they catch up on this tick: 1 at full detail; in a lot nobody watches, 0 between
    /// turns and every tick since the last one on their turn (see `lod.rs`).
    pub(crate) steps: u32,
    /// Ticks since their last turn.
    pub(crate) owed: u32,
}

impl Sim {
    pub fn tile(&self) -> (i32, i32) {
        (self.pos[0].floor() as i32, self.pos[1].floor() as i32)
    }

    pub fn is_moving(&self) -> bool {
        matches!(
            self.current,
            Some(Activity {
                phase: Phase::Routing { .. },
                ..
            })
        )
    }

    pub fn current(&self) -> Option<&Activity> {
        self.current.as_ref()
    }

    /// In bed for the night (or a nap): using something tagged as sleep.
    pub fn asleep(&self, content: &Content) -> bool {
        self.current.as_ref().is_some_and(|a| {
            a.tags & content.day_rhythm.sleep_tags != 0 && matches!(a.phase, Phase::Using { .. })
        })
    }

    /// A baby in the crib lies awake while they need something (crying for it) or once they're
    /// rested in the daytime, and sleeps otherwise.
    pub fn awake_in_crib(&self, content: &Content, hour: f32) -> bool {
        let rhythm = &content.day_rhythm;
        let crying = (0..content.needs.len()).any(|n| !content.needs[n].room && self.needs[n] < BABY_CRIES_BELOW);
        let rested = rhythm.rest_need.is_none_or(|n| self.needs[n] >= 0.999);
        crying || (rested && !rhythm.is_night(hour, rhythm.wake_hour))
    }

    /// Stops whatever they're doing and drops what they meant to do (their object slots are
    /// released by the caller).
    pub(crate) fn clear_activity(&mut self) {
        self.current = None;
        self.queue.clear();
        self.follow_up = None;
        self.engaged_with = None;
        self.heeding = None;
        self.transition = None;
        self.visiting = None;
        self.pose = Pose::Stand;
        self.seated_from = None;
        self.use_seat = None;
    }

    /// Grown up (may work, fall in love, move, have children).
    pub fn adult(&self, content: &Content) -> bool {
        content.life.adult(self.age)
    }

    /// Still in town (not dead or moved away).
    pub fn here(&self) -> bool {
        self.gone.is_none()
    }

    pub fn queue(&self) -> impl Iterator<Item = &Task> {
        self.queue.iter()
    }

    /// Game minutes their update covers this tick (see `steps`).
    pub(crate) fn minutes(&self) -> f32 {
        MINUTES_PER_TICK * self.steps as f32
    }

    /// Needs average plus trait and feeling effects, 0..1.
    pub fn mood(&self, content: &Content) -> f32 {
        let n = content.needs.len();
        let needs = self.needs[..n].iter().sum::<f32>() / n as f32;
        (needs + self.mods.mood + social::feeling_mood(&self.feelings, &content.feelings))
            .clamp(0.0, 1.0)
    }

    pub fn emotion(&self, content: &Content) -> Option<usize> {
        social::dominant_emotion(&self.feelings, &content.feelings)
    }

    /// Recomputes `mods` when the set of active feelings with effects has changed.
    pub(crate) fn refresh_buffs(&mut self, content: &Content) {
        let defs = &content.feelings;
        let mut active = self
            .feelings
            .iter()
            .filter(|m| defs[m.def].effects.is_some());
        if active.clone().count() == self.buffs.len() && active.all(|m| self.buffs.contains(&m.def))
        {
            return;
        }
        self.buffs.clear();
        self.buffs.extend(
            self.feelings
                .iter()
                .filter(|m| defs[m.def].effects.is_some())
                .map(|m| m.def),
        );
        self.buffs.sort_unstable();
        self.mods = self.base_mods.clone();
        for &m in &self.buffs {
            if let Some(e) = &defs[m].effects {
                self.mods.combine(e);
            }
        }
    }

    pub fn attracted_to(&self, other: &Sim) -> bool {
        match other.gender_ix {
            Some(g) => self.attraction & (1 << g) != 0,
            None => true, // content defines no genders
        }
    }

    /// Whether a need a chat won't help with is about to run out (the bladder): no time to talk.
    pub fn pressed(&self, content: &Content) -> bool {
        let rules = &content.autonomy;
        let essentials = rules.urgent & !rules.social_fills;
        (0..content.needs.len()).any(|n| !content.needs[n].room && essentials & (1 << n) != 0 && self.needs[n] < rules.urgent_below)
    }

    /// Whether another Sim can pull this one into a conversation right now.
    pub fn available_for_social(&self, content: &Content) -> bool {
        if self.engaged_with.is_some() || self.heeding.is_some() || self.pose == Pose::Lie || self.away_until.is_some() || self.pressed(content) {
            return false;
        }
        // Busy with a planned block (unless it's for socialising).
        if let Some(run) = &self.planner.run
            && run.skipped.is_none()
            && !run.no_place
            && !content.activities.get(run.activity).is_some_and(|a| a.social || a.visit)
        {
            return false;
        }
        let Some(act) = &self.current else {
            return true;
        };
        if act.task.directed {
            return false;
        }
        match (act.task.kind, &act.phase) {
            // People on their way somewhere don't stop to chat (it would strand them), nor do
            // cooks halfway through a recipe or people eating or washing up.
            (TaskKind::Work | TaskKind::Visit { .. } | TaskKind::GoHome | TaskKind::Answer { .. }, _) => false,
            (TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. }, _) => false,
            (TaskKind::Social { .. } | TaskKind::Join { .. }, Phase::Conversing { .. }) => false,
            // A guest on their way to say goodbye (`visits.farewell`) is leaving.
            (TaskKind::Social { social, .. }, _) if content.visits.farewell.contains(&social) => false,
            (_, Phase::Routing { .. }) => true,
            _ => act.tags & content.social_rules.busy_tags == 0,
        }
    }

    /// For a Sim leading a conversation: (target, social, success, progress 0..1).
    pub fn conversation(&self, content: &Content) -> Option<(u32, usize, bool, f32)> {
        match &self.current {
            Some(Activity {
                task:
                    Task {
                        kind: TaskKind::Social { target, social },
                        ..
                    },
                phase: Phase::Conversing { elapsed, success },
                ..
            }) => {
                let s = &content.socials[*social];
                let duration = conversation::duration(s, *success);
                Some((*target, *social, *success, (elapsed / duration).min(1.0)))
            }
            _ => None,
        }
    }

    /// For a Sim listening in on someone else's conversation (a group conversation): the one
    /// who started it.
    pub fn listening_to(&self) -> Option<u32> {
        match &self.current {
            Some(Activity { task: Task { kind: TaskKind::Join { leader }, .. }, phase: Phase::Conversing { .. }, .. }) => Some(*leader),
            _ => None,
        }
    }

    /// What they couldn't get to a moment ago and won't choose again yet (`unreachable`).
    pub(crate) fn unreachable_now(&self, tick: u64) -> impl Iterator<Item = TaskKind> + '_ {
        self.unreachable.iter().flatten().filter(move |u| tick < u.1).map(|u| u.0)
    }

    /// Whether they couldn't get to object `id` a moment ago (`unreachable`), and won't try again yet.
    pub(crate) fn couldnt_reach(&self, id: u32, tick: u64) -> bool {
        self.unreachable_now(tick).any(|kind| matches!(kind, TaskKind::Use { object, .. } | TaskKind::Repair { object } if object == id))
    }

    /// They couldn't get to what they chose for themselves (`kind`): not again until `until`.
    /// Only what can't be reached for long is remembered (things, places, visits, and people,
    /// who move and so are given a short `until` by the caller); the oldest memory makes way.
    pub(crate) fn remember_unreachable(&mut self, kind: TaskKind, until: u64) {
        if !matches!(
            kind,
            TaskKind::Use { .. }
                | TaskKind::Eat { .. }
                | TaskKind::Clean { .. }
                | TaskKind::Repair { .. }
                | TaskKind::Dishes { .. }
                | TaskKind::Visit { .. }
                | TaskKind::Social { .. }
                | TaskKind::Join { .. }
        ) {
            return;
        }
        let same = self.unreachable.iter().position(|u| u.is_some_and(|u| same_target(u.0, kind)));
        let slot = same.unwrap_or_else(|| (0..UNREACHABLE).min_by_key(|&k| self.unreachable[k].map_or(0, |u| u.1)).unwrap_or(0));
        self.unreachable[slot] = Some((kind, until));
    }

    /// Objects moved (ids shifted, one gone): `remap` gives each object id in `unreachable` its
    /// new id, or None for one that's gone (forgotten then).
    pub(crate) fn remap_unreachable(&mut self, remap: impl Fn(u32) -> Option<u32>) {
        for slot in &mut self.unreachable {
            let Some((TaskKind::Use { object, .. } | TaskKind::Repair { object }, _)) = slot else { continue };
            match remap(*object) {
                Some(new) => *object = new,
                None => *slot = None,
            }
        }
    }
}

/// Whether two tasks are after the same thing (`Sim::unreachable`): the same use of an object,
/// the same food, tile, broken object, dirty dish, house or person.
fn same_target(a: TaskKind, b: TaskKind) -> bool {
    match (a, b) {
        (TaskKind::Use { object, interaction }, TaskKind::Use { object: o, interaction: i }) => (object, interaction) == (o, i),
        (TaskKind::Eat { item, .. }, TaskKind::Eat { item: it, .. }) => item == it,
        (TaskKind::Clean { x, z }, TaskKind::Clean { x: cx, z: cz }) => (x, z) == (cx, cz),
        (TaskKind::Repair { object }, TaskKind::Repair { object: o }) => object == o,
        (TaskKind::Dishes { item, .. }, TaskKind::Dishes { item: it, .. }) => item == it,
        (TaskKind::Visit { plot }, TaskKind::Visit { plot: p }) => plot == p,
        (TaskKind::Social { target, social }, TaskKind::Social { target: t, social: s }) => (target, social) == (t, s),
        (TaskKind::Join { leader }, TaskKind::Join { leader: l }) => leader == l,
        _ => false,
    }
}

/// How a house's roof looks: a roof style and a roof colour (indices into the content's
/// `roofStyles` and `roofColors`). Presentation only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoofLook {
    pub style: u8,
    pub color: u8,
}

/// A region of the town: a residential lot or a public place.
#[derive(Debug, Clone, PartialEq)]
pub struct Plot {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub z: i32,
    pub w: i32,
    pub d: i32,
    /// Public plots (parks) can be used by everyone.
    pub public: bool,
    /// Where visitors arrive (in front of the door); defaults to the plot centre.
    pub entry: Option<[f32; 2]>,
    /// The roof the player chose for the house on this plot (None: the town's own look).
    pub roof: Option<RoofLook>,
}

impl Plot {
    pub fn contains(&self, x: i32, z: i32) -> bool {
        x >= self.x && z >= self.z && x < self.x + self.w && z < self.z + self.d
    }

    pub fn arrival_point(&self) -> [f32; 2] {
        self.entry.unwrap_or([
            self.x as f32 + self.w as f32 / 2.0,
            self.z as f32 + self.d as f32 / 2.0,
        ])
    }
}

/// How a game treats money for building: chosen once per game.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    /// The residents earn the money: build and buy are paid from household funds.
    #[default]
    Living,
    /// Building is free: build and buy cost nothing (selling brings nothing back), and the
    /// player's home pays no rent or bills.
    Creative,
}

#[derive(Debug, Clone)]
pub struct Household {
    pub id: u32,
    pub name: String,
    /// The plot this household lives on.
    pub plot: Option<u32>,
    /// Controlled by the player.
    pub player: bool,
    /// Money earned from jobs.
    pub funds: i64,
    /// "Free will": whether this household's residents choose what to do on their own.
    pub free_will: bool,
    /// Routine blocks every member follows (each may skip some; their own blocks win).
    pub routines: Vec<crate::planner::Routine>,
    /// A baby on the way (see `lifecycle::conceive`).
    pub expecting: Option<Expecting>,
    /// The day a babysitter was booked for (see `life::childcare`).
    pub sitter_day: Option<u32>,
}

/// A household expecting a baby: the parents and the day it's due.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expecting {
    pub parents: [u32; 2],
    pub due: u32,
}

/// A resident who is no longer in town: they died or moved away (see `lifecycle.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gone {
    pub why: GoneWhy,
    /// Game day it happened.
    pub day: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GoneWhy {
    Died,
    MovedAway,
}

/// Someone whose slot a newcomer took: kept so the story can still name them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Former {
    pub name: String,
    /// Their household's name at the time.
    pub household: String,
}

/// Per-tick snapshot of other Sims, so each Sim can reason about the others
/// without aliasing the mutable Sim list. Reused between ticks.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SimBrief {
    pub pos: [f32; 2],
    pub available: bool,
    pub gender_ix: Option<usize>,
    pub household: u32,
    /// Plot the Sim is standing on.
    pub plot: Option<u32>,
    pub away: bool,
    /// On their way off their home plot, now or later in their queue (`life::leaving_home`; so
    /// the last grown-up at home with a baby knows they will be: `life::minding_baby`).
    pub leaving: bool,
    /// A baby (cared for in a crib), and their needs (so carers know what's wanted).
    pub baby: bool,
    pub needs: [f32; MAX_NEEDS],
    /// Has a partner.
    pub partnered: bool,
    /// Still in town, grown up, and how old.
    pub here: bool,
    pub adult: bool,
    pub age: f32,
    /// In a conversation (leading it, being talked to or listening in): who started it and
    /// the social.
    pub conversing: Option<(u32, usize)>,
    /// The tile they're tidying up (or on their way to), if any.
    pub cleaning: Option<(i32, i32)>,
}

pub struct World {
    pub content: Content,
    /// Living or Creative (see `GameMode`).
    pub mode: GameMode,
    pub lot: Lot,
    pub objects: Vec<ObjectInstance>,
    /// The next stable object id (see `ObjectInstance::stable`); never reused.
    pub(crate) next_object_id: u32,
    /// Food, plates and pots (see `items.rs`).
    pub items: crate::items::Items,
    /// Plot index per object (`None` when outside every plot).
    pub(crate) object_plot: Vec<Option<u32>>,
    /// Objects by plot, for decisions (see `index.rs`); kept up to date by `refresh_index`.
    pub(crate) object_index: ObjectIndex,
    pub sims: Vec<Sim>,
    pub households: Vec<Household>,
    pub plots: Vec<Plot>,
    /// Plot per tile (see `map_plots`).
    pub(crate) tile_plot: Vec<Option<u32>>,
    pub relationships: Relationships,
    pub events: EventLog,
    pub(crate) blocked: Vec<bool>,
    pub tick: u64,
    pub speed: u8,
    /// Master switch for "free will" in the whole town (each household also has its own,
    /// `Household::free_will`). Off is mainly for tests.
    pub autonomy: bool,
    /// How fast residents age (a per-game option).
    pub lifespan: crate::lifecycle::Lifespan,
    /// Whether the player's residents move in with partners and out of the family home on
    /// their own (a per-game option; others always do).
    pub player_moves: bool,
    /// People whose slot someone new took, for the story (`social::FORMER` references).
    pub former: Vec<Former>,
    /// Bumped per slot when someone new takes it (views rebuild that resident).
    pub(crate) generations: Vec<u32>,
    pub(crate) rng: Rng,
    /// The seed the game started from (0: unknown, a save from before v23), for bug reports.
    pub seed: u32,
    pub(crate) structure_version: u32,
    /// Bumped only when something a walker can or can't step through changes: walls, doors,
    /// windows, stairs, storeys, and things placed or removed. Looks, repairs and crops don't
    /// bump it, so the walkable regions (and the objects-by-plot index) needn't be rebuilt for
    /// them. `structure_version` stays the one views and offers follow.
    pub(crate) nav_version: u32,
    /// The walkable regions of the lot at `reach_version` (see `path::Reach`).
    pub(crate) reach: crate::path::Reach,
    pub(crate) reach_version: u32,
    /// Bumped when the lot itself changes (walls, doors, windows, fences, coverings, floors),
    /// not when only objects, residents or households do: views can then leave the lot out.
    pub(crate) lot_version: u32,
    pub(crate) briefs: Vec<SimBrief>,
    /// Where Sims leave town for work.
    pub exits: Vec<[f32; 2]>,
    /// Opaque presentation data from the lot/town file, saved unchanged.
    pub meta: serde_json::Value,
    /// Build and buy edits that can be taken back, oldest first (see `World::undo`).
    pub(crate) undo: Vec<crate::home::HomeSnapshot>,
    /// Edits taken back that can be made again, latest last (see `World::redo`).
    pub(crate) redo: Vec<crate::home::HomeSnapshot>,
    /// "Skip quiet hours": run at `AUTO_FAST_SPEED` while the player's household is asleep
    /// or out and nothing notable happened at home in the last hour (see `World::calm`).
    pub auto_fast: bool,
    /// What each household's home (plus public places) offers, for planned activities;
    /// recomputed when the structure changes.
    pub(crate) offers: Vec<crate::planner::HouseOffer>,
    pub(crate) offers_version: u32,
    /// Dirt per tile (0..1), from using things and accidents; cleaned by residents.
    pub dirt: Vec<f32>,
    /// Every room and garden with its scores (see `rooms.rs`), and the layout version they
    /// were computed for.
    pub(crate) rooms: Vec<crate::rooms::RoomInfo>,
    pub(crate) rooms_version: u32,
    /// Bumped when the lot's rooms or their scores change structurally (walls, coverings,
    /// objects added or removed). Unlike `structure_version`, a cupboard opening or a crop
    /// growing doesn't bump it, so a busy town needn't rebuild every room for a change of looks.
    pub(crate) room_layout_version: u32,
    /// A room's scores changed without the layout changing (a thing broke, was repaired or
    /// upgraded); the next tick recomputes the rooms once.
    pub(crate) rooms_dirty: bool,
    /// Only cleanliness may have changed (dirt, or a look that doesn't move anything); the next
    /// tick rescores the rooms without walking the lot again.
    pub(crate) rooms_rescore: bool,
    /// Overall score of the room or garden each tile is in (`roomRules.away` off the plots).
    pub(crate) room_score: Vec<f32>,
    /// Which tiles make up each room, to rescore cleanliness after messes (see `rooms.rs`).
    pub(crate) room_tiles: crate::rooms::RoomTiles,
    /// How many storeys the lot holds (see `storeys.rs`); 1: only the ground.
    pub storeys: u8,
    /// Rows per storey and the stairs between them, for walking.
    pub(crate) stairs: crate::path::StairMap,
    /// Lower detail for lots nobody watches (see `lod.rs`); on unless turned off for tests and
    /// comparisons. It only applies once there's a view.
    pub lod: bool,
    /// The part of the town being looked at (`set_view`); None: all of it.
    pub(crate) view: Option<[i32; 4]>,
    /// Per plot: watched this tick (see `lod::plan`).
    pub(crate) watched: Vec<bool>,
    /// Bumped when a lamp is switched on or off (see `lights.rs`); not a structure change.
    pub(crate) lights_version: u32,
    /// What residents switching lights saw last (see `lights::update`).
    pub(crate) light_watch: crate::lights::Watch,
}

/// Read-only context for one Sim's update.
pub(crate) struct Ctx<'a> {
    pub content: &'a Content,
    pub nav: NavGrid<'a>,
    /// Which tiles can reach which (see `path::Reach`): a goal elsewhere is refused at once.
    pub reach: &'a crate::path::Reach,
    /// Bumped when the house changes in a way that can cut a walker's route short.
    pub nav_version: u32,
    pub rels: &'a Relationships,
    pub briefs: &'a [SimBrief],
    pub households: &'a [Household],
    pub plots: &'a [Plot],
    /// Plot per ground tile (`World::tile_plot`).
    pub tile_plot: &'a [Option<u32>],
    pub object_plot: &'a [Option<u32>],
    /// Objects by plot (`World::object_index`).
    pub index: &'a ObjectIndex,
    pub exits: &'a [[f32; 2]],
    pub autonomy: bool,
    /// Room score per tile (`World::room_score`).
    pub room_score: &'a [f32],
    /// Dirt per tile (`World::dirt`).
    pub dirt: &'a [f32],
    pub hour: f32,
    pub day: u32,
    pub minute: f32,
    pub tick: u64,
}

impl Ctx<'_> {
    /// Score of the room or garden at a point (`roomRules.away` off the plots and off the lot).
    pub(crate) fn room_at(&self, pos: [f32; 2]) -> f32 {
        let (x, z) = (pos[0].floor() as i32, pos[1].floor() as i32);
        let lot = self.nav.lot;
        if !lot.in_bounds(x, z) {
            return self.content.room_rules.away;
        }
        self.room_score.get(lot.tile_index(x, z)).copied().unwrap_or(self.content.room_rules.away)
    }
}

#[derive(Debug, Clone, Copy)]
enum Choice {
    Object(u32, usize),
    /// A serving of food (an item).
    Eat(u32),
    /// Washing up, starting with this item.
    Dishes(u32),
    Clean(i32, i32),
    Repair(u32),
    Social(u32, usize),
    /// Joining the conversation this Sim started.
    Join(u32),
    Visit(u32),
    /// A phone action, with someone or not (`phone.rs`).
    Phone(usize, Option<u32>),
}

impl World {
    pub fn from_json(content_json: &str, lot_json: &str, seed: u32) -> Result<Self, Error> {
        let content = Content::from_json(content_json)?;
        let lot_file: LotFile = serde_json::from_str(lot_json)?;
        Self::new(content, &lot_file, seed)
    }

    pub fn new(content: Content, lot_file: &LotFile, seed: u32) -> Result<Self, Error> {
        let mut world = Self::empty(content, Lot::from_file(lot_file)?, Rng::new(seed));
        world.seed = seed;
        // Storeys from the file before its objects (stairs, furniture upstairs).
        let storeys = lot_file.storeys.unwrap_or(1).clamp(1, 4);
        if storeys > 1 || lot_file.basement {
            world.storeys = storeys;
            world.stairs.depth = lot_file.depth as i32;
        }
        // The basement's block after the storeys' (storeys the content adds go after it).
        if lot_file.basement {
            world.stairs.basement = storeys as i32;
        }
        world.mode = lot_file.mode;
        world.lifespan = lot_file.lifespan;
        world.meta = lot_file.meta.clone();
        world.exits = lot_file.exits.clone();
        world.plots = lot_file
            .plots
            .iter()
            .enumerate()
            .map(|(i, p)| Plot {
                id: i as u32,
                name: p.name.clone(),
                x: p.x,
                z: p.z,
                w: p.w,
                d: p.d,
                public: p.public,
                entry: p.entry,
                roof: None,
            })
            .collect();
        let starting_funds = world.content.starting_funds;
        world.households = if lot_file.households.is_empty() {
            vec![Household {
                id: 0,
                name: "Household".into(),
                plot: None,
                player: true,
                funds: starting_funds,
                free_will: true,
                routines: Vec::new(),
                expecting: None,
                sitter_day: None,
            }]
        } else {
            lot_file
                .households
                .iter()
                .enumerate()
                .map(|(i, h)| Household {
                    id: i as u32,
                    name: h.name.clone(),
                    plot: h.plot,
                    player: h.player,
                    funds: h.funds.unwrap_or(starting_funds),
                    free_will: true,
                    routines: Vec::new(),
                    expecting: None,
                    sitter_day: None,
                })
                .collect()
        };
        if let Some(h) = world
            .households
            .iter()
            .find(|h| h.plot.is_some_and(|p| p as usize >= world.plots.len()))
        {
            return Err(Error::new(format!(
                "household '{}' lives on an unknown plot",
                h.name
            )));
        }
        for p in &lot_file.objects {
            let def = world
                .content
                .object_index(&p.def)
                .ok_or_else(|| Error::new(format!("unknown object '{}'", p.def)))?;
            // Turned (only what turns freely may be), set off its tile's middle or resized.
            let at = |e: Error| Error::new(format!("'{}' at {},{}: {e}", p.def, p.x, p.z));
            let turn = world.check_turn(def, p.turn).map_err(at)?;
            let layout = world.check_layout(def, p.rot, Layout { offset: p.offset, extent: p.size, tall: p.tall }).map_err(at)?;
            let value = world.content.objects[def].price.unwrap_or(0);
            let id = world.place(def, p.x, p.z, p.rot, layout, Default::default(), 0, value, None)?;
            world.objects[id as usize].turn = turn;
        }
        if lot_file.sims.len() > MAX_SIMS {
            return Err(Error::new(format!("at most {MAX_SIMS} residents per town")));
        }
        for s in &lot_file.sims {
            world.spawn_sim(s)?;
        }
        world.init_relationships(&lot_file.relationships)?;
        // Babies come with a crib (a crowded home is reported to the player as time starts).
        for h in 0..world.households.len() {
            crate::lifecycle::cribs_for_babies(&mut world, h);
        }
        world.make_storeys();
        world.refresh_rooms();
        Ok(world)
    }

    pub(crate) fn empty(content: Content, lot: Lot, rng: Rng) -> Self {
        let tiles = lot.width * lot.depth;
        Self {
            content,
            mode: GameMode::Living,
            lot,
            objects: Vec::new(),
            next_object_id: 0,
            items: Default::default(),
            object_plot: Vec::new(),
            object_index: ObjectIndex::default(),
            sims: Vec::new(),
            households: Vec::new(),
            plots: Vec::new(),
            tile_plot: Vec::new(),
            relationships: Relationships::default(),
            events: EventLog::default(),
            blocked: vec![false; tiles],
            tick: 0,
            speed: 1,
            autonomy: true,
            lifespan: Default::default(),
            player_moves: true,
            former: Vec::new(),
            generations: Vec::new(),
            rng,
            seed: 0,
            structure_version: 1,
            nav_version: 1,
            reach: Default::default(),
            reach_version: 0,
            lot_version: 1,
            briefs: Vec::new(),
            exits: Vec::new(),
            meta: serde_json::Value::Null,
            undo: Vec::new(),
            redo: Vec::new(),
            auto_fast: false,
            offers: Vec::new(),
            offers_version: 0,
            dirt: vec![0.0; tiles],
            rooms: Vec::new(),
            rooms_version: 0,
            room_layout_version: 1,
            rooms_dirty: false,
            rooms_rescore: false,
            room_score: Vec::new(),
            room_tiles: Default::default(),
            storeys: 1,
            stairs: crate::path::StairMap::default(),
            lod: true,
            view: None,
            watched: Vec::new(),
            lights_version: 0,
            light_watch: Default::default(),
        }
    }

    /// Bumped whenever walls or objects change, so renderers know to rebuild.
    pub fn structure_version(&self) -> u32 {
        self.structure_version
    }

    /// Bumped whenever a lamp is switched (see `lights::lamps_off`).
    pub fn lights_version(&self) -> u32 {
        self.lights_version
    }

    /// Changes whenever the lot does (see `lot_version`); never while only objects move.
    pub fn lot_version(&self) -> u32 {
        self.lot_version
    }

    pub fn blocked(&self) -> &[bool] {
        &self.blocked
    }

    /// Whether someone at `b` hears someone at `a` call their name (see
    /// `conversation::within_earshot`).
    pub fn within_earshot(&self, a: [f32; 2], b: [f32; 2]) -> bool {
        conversation::within_earshot(&self.nav(), a, b)
    }

    /// The plot a tile is on (on any storey).
    pub fn plot_at(&self, x: i32, z: i32) -> Option<u32> {
        let z = self.ground_z(z);
        let width = self.lot.width as i32;
        if (0..width).contains(&x)
            && z >= 0
            && let Some(&plot) = self.tile_plot.get((z * width + x) as usize)
        {
            return plot;
        }
        self.plots.iter().find(|p| p.contains(x, z)).map(|p| p.id)
    }

    /// Notes the plot of every ground tile (plots never move; upper storeys are the same
    /// ground), so `plot_at` needn't search them.
    fn map_plots(&mut self) {
        let (width, depth) = (self.lot.width as i32, self.storey_depth());
        if self.tile_plot.len() == (width * depth) as usize {
            return;
        }
        self.tile_plot = vec![None; (width * depth) as usize];
        for p in self.plots.iter().rev() {
            for z in p.z.max(0)..(p.z + p.d).min(depth) {
                for x in p.x.max(0)..(p.x + p.w).min(width) {
                    self.tile_plot[(z * width + x) as usize] = Some(p.id);
                }
            }
        }
    }

    /// The index in `World::objects` of the object with stable id `stable`, if it still exists.
    pub fn index_of(&self, stable: u32) -> Option<usize> {
        self.objects.iter().position(|o| o.stable == stable)
    }

    /// The stable id of the object at `index` in `World::objects` (the inverse of `index_of`).
    pub fn stable_of(&self, index: u32) -> Option<u32> {
        self.objects.get(index as usize).map(|o| o.stable)
    }

    pub fn place_object(&mut self, def_id: &str, x: i32, z: i32, rot: u8) -> Result<u32, Error> {
        let def = self
            .content
            .object_index(def_id)
            .ok_or_else(|| Error::new(format!("unknown object '{def_id}'")))?;
        let value = self.content.objects[def].price.unwrap_or(0);
        self.place(def, x, z, rot, Layout::default(), Default::default(), 0, value, None)
    }

    /// Checks that object `def` fits at `x, z, rot` (laid out as `layout`, unchecked: see
    /// `check_layout`):
    /// inside the lot, on free tiles without a diagonal wall, facing a tile on the lot. Things
    /// on the `Surface` layer need a top to stand on instead (see `surface::fit_on_top`).
    pub(crate) fn check_fit(
        &self,
        def: usize,
        x: i32,
        z: i32,
        rot: u8,
        layout: Layout,
    ) -> Result<ObjectInstance, Error> {
        let obj = ObjectInstance {
            id: self.objects.len() as u32,
            stable: 0,
            def,
            x,
            z,
            rot: rot % 4,
            turn: 0,
            offset: layout.offset,
            footprint: layout.extent.map_or(self.content.objects[def].footprint, tiles_for),
            extent: layout.extent.unwrap_or(kind_size(self.content.objects[def].footprint)),
            tall: layout.tall,
            users: [None; MAX_SLOTS],
            closed_users: 0,
            quality: 0,
            colors: Colors::new(),
            value: 0,
            wear: 0.0,
            open: false,
            // A crop comes ripe with a lot (bought, it is just planted: `buy_at`).
            growth: 1.0,
            off: false,
        };
        let d = &self.content.objects[def];
        let name = &d.id;
        let layer = d.layer;
        if layer == Layer::Surface {
            return self.check_surface_fit(obj);
        }
        let on = |o: &ObjectInstance, t: (i32, i32)| o.tiles(&self.content).any(|q| q == t);
        let called = |o: &ObjectInstance| self.content.objects[o.def].name.to_lowercase();
        for (tx, tz) in obj.tiles(&self.content) {
            // Furniture needs free floor; things on a wall, the ceiling or rugs only a free spot
            // on their own layer (and a hanging plant that hangs low the floor as well).
            let floor = self.lot.in_bounds(tx, tz) && self.blocked[self.lot.tile_index(tx, tz)];
            let taken = if layer.mounted() {
                self.objects.iter().any(|o| self.content.objects[o.def].layer == layer && on(o, (tx, tz)))
                    || (d.hangs_low && floor && !self.objects.iter().any(|o| !self.content.objects[o.def].layer.mounted() && on(o, (tx, tz))))
            } else {
                floor && !self.objects.iter().any(|o| self.content.objects[o.def].hangs_low && on(o, (tx, tz)))
            };
            if !self.lot.in_bounds(tx, tz) || taken {
                return Err(Error::new(format!(
                    "'{name}' at {x},{z} overlaps or leaves the lot"
                )));
            }
            // Furniture as tall as the wall leaves no room on the wall or the ceiling over it; a
            // thing hanging low from the ceiling none under it.
            let clash = self.objects.iter().find(|o| {
                let other = &self.content.objects[o.def];
                on(o, (tx, tz))
                    && match (layer, other.layer) {
                        (Layer::Wall, Layer::Floor) => other.fills_wall,
                        (Layer::Ceiling, Layer::Floor) => other.fills_wall || d.hangs_low,
                        (Layer::Floor, Layer::Wall) => d.fills_wall,
                        (Layer::Floor, Layer::Ceiling) => d.fills_wall || other.hangs_low,
                        _ => false,
                    }
            });
            if let Some(o) = clash {
                let other = &self.content.objects[o.def];
                let why = match layer {
                    Layer::Floor if other.hangs_low => format!("is under the {}, which hangs too low", called(o)),
                    Layer::Floor => format!("is too tall for the {} hanging there", called(o)),
                    Layer::Ceiling if d.hangs_low && !other.fills_wall => format!("would hang too low over the {}", called(o)),
                    _ => format!("would hang into the {}", called(o)),
                };
                return Err(Error::new(format!("'{name}' at {x},{z} {why}")));
            }
            if layer == Layer::Ceiling && self.lot.room_at(tx, tz) == crate::lot::OUTDOORS {
                return Err(Error::new(format!("'{name}' at {x},{z} needs a ceiling")));
            }
            // Furniture never stands on a tile split by a diagonal wall (it would overlap it).
            if self.lot.diag(tx, tz).is_some() {
                return Err(Error::new(format!(
                    "'{name}' at {x},{z} overlaps a diagonal wall"
                )));
            }
        }
        let (fx, fz) = obj.front_tile(&self.content);
        if !self.lot.in_bounds(fx, fz) {
            return Err(Error::new(format!("'{name}' at {x},{z} faces off the lot")));
        }
        // No wall or fence may run through the footprint, through the body where it stands off
        // the footprint, or between the object and where it's used from.
        let in_lot = |(a, b): &((i32, i32), (i32, i32))| self.lot.in_bounds(a.0, a.1) && self.lot.in_bounds(b.0, b.1);
        let body = obj.body_edges(&self.content).into_iter().filter(in_lot);
        // Nor a diagonal wall across a tile it reaches into.
        let diagonal = |(tx, tz): (i32, i32)| self.lot.diag(tx, tz).is_some_and(|g| obj.body_crosses(&self.content, tx, tz, g.dir));
        if obj.wall_edges(&self.content).chain(body).any(|(a, b)| {
            self.lot.edge_between(a.0, a.1, b.0, b.1) != crate::lot::Edge::Open
        }) || obj.body_tiles(&self.content).any(diagonal)
        {
            return Err(Error::new(format!("'{name}' at {x},{z} is blocked by a wall")));
        }
        if layer == Layer::Wall {
            match self.wall_behind(&obj) {
                None => return Err(Error::new(format!("'{name}' at {x},{z} has no wall to hang on"))),
                Some(true) if !d.over_window => return Err(Error::new(format!("'{name}' at {x},{z} can't hang over a window"))),
                _ => {}
            }
        }
        self.check_storey_fit(&obj)?;
        Ok(obj)
    }

    /// `check_fit` for a thing on the `Surface` layer: on the lot, its disc wholly on one top
    /// with nothing else standing in it. It needs no front tile and no wall check.
    fn check_surface_fit(&self, obj: ObjectInstance) -> Result<ObjectInstance, Error> {
        use crate::surface::Misfit;
        let name = &self.content.objects[obj.def].id;
        let (x, z) = (obj.x, obj.z);
        if !self.lot.in_bounds(x, z) {
            return Err(Error::new(format!("'{name}' at {x},{z} overlaps or leaves the lot")));
        }
        let at = crate::surface::middle(&self.content, &obj);
        let radius = self.content.objects[obj.def].radius;
        match crate::surface::fit_on_top(&self.content, &self.objects, at, radius, &[]) {
            Ok(_) => Ok(obj),
            Err(Misfit::NoTop) => Err(Error::new(format!("'{name}' at {x},{z} has nothing to stand on"))),
            Err(Misfit::OverEdge) => Err(Error::new(format!("'{name}' at {x},{z} would hang over the edge"))),
            Err(Misfit::Crowded) => Err(Error::new(format!("'{name}' at {x},{z} overlaps or leaves the lot"))),
        }
    }

    /// The tile a resident stands on to use `obj`, as things stand now (see
    /// `surface::approach_tile`; residents deciding what to do use `Links`).
    pub(crate) fn approach_tile(&self, obj: &ObjectInstance) -> (i32, i32) {
        crate::surface::approach_tile(&self.content, &self.nav(), obj)
    }

    /// Every tile a resident may use `obj` from, as things stand now (see `surface::approach_tiles`).
    pub(crate) fn approach_tiles(&self, obj: &ObjectInstance) -> Vec<(i32, i32)> {
        crate::surface::approach_tiles(&self.content, &self.nav(), obj).collect()
    }

    /// What is walkable now.
    pub(crate) fn nav(&self) -> NavGrid<'_> {
        NavGrid { lot: &self.lot, blocked: &self.blocked, stairs: &self.stairs }
    }

    /// What runs along the whole back edge of `obj`: None if not a wall (plain or with a window)
    /// all along, else whether a window is part of it.
    fn wall_behind(&self, obj: &ObjectInstance) -> Option<bool> {
        use crate::lot::Edge;
        let lot = &self.lot;
        obj.back_edges(&self.content).into_iter().try_fold(false, |window, (horizontal, x, z)| {
            let e = if horizontal {
                (x >= 0 && (x as usize) < lot.width && z >= 0 && (z as usize) <= lot.depth).then(|| lot.h_edge(x as usize, z as usize))
            } else {
                (x >= 0 && (x as usize) <= lot.width && z >= 0 && (z as usize) < lot.depth).then(|| lot.v_edge(x as usize, z as usize))
            };
            match e? {
                Edge::Wall => Some(window),
                Edge::Window => Some(true),
                _ => None,
            }
        })
    }

    /// What hangs on the wall at this grid edge (`horizontal`: an `h` edge).
    pub(crate) fn hung_on(&self, horizontal: bool, x: i32, z: i32) -> impl Iterator<Item = &ObjectInstance> {
        self.objects.iter().filter(move |o| {
            self.content.objects[o.def].layer == Layer::Wall && o.back_edges(&self.content).contains(&(horizontal, x, z))
        })
    }

    /// Whether something hangs on the wall at this grid edge (`horizontal`: an `h` edge).
    pub(crate) fn hangs_on(&self, horizontal: bool, x: i32, z: i32) -> bool {
        self.hung_on(horizontal, x, z).next().is_some()
    }

    /// Places object `def` if it fits. `stable`: its id from a save; otherwise it gets a fresh one.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn place(
        &mut self,
        def: usize,
        x: i32,
        z: i32,
        rot: u8,
        layout: Layout,
        colors: Colors,
        quality: u8,
        value: i64,
        stable: Option<u32>,
    ) -> Result<u32, Error> {
        let mut obj = self.check_fit(def, x, z, rot, layout)?;
        obj.stable = stable.unwrap_or_else(|| {
            let id = self.next_object_id;
            self.next_object_id = id.wrapping_add(1);
            id
        });
        obj.colors = colors;
        obj.quality = quality.min(self.content.object_rules.max_quality);
        obj.value = value;
        for (tx, tz) in self.blocking_tiles(&obj) {
            let i = self.lot.tile_index(tx, tz);
            self.blocked[i] = true;
        }
        self.object_plot.push(self.plot_at(x, z));
        let stairs = self.content.objects[def].stairs;
        self.objects.push(obj);
        if stairs {
            self.refresh_stairs();
        }
        self.nav_changed();
        self.rooms_layout_changed();
        Ok(self.objects.len() as u32 - 1)
    }

    /// Adds a Sim with random starting needs. Identity, traits and perks are validated.
    pub fn spawn_sim(&mut self, spawn: &SimSpawn) -> Result<u32, Error> {
        self.spawn_at(spawn, false)
    }

    /// A new resident; with `reuse`, in the first slot someone gone left (their story events
    /// then name them as a former resident), else in a new slot.
    pub(crate) fn spawn_at(&mut self, spawn: &SimSpawn, reuse: bool) -> Result<u32, Error> {
        let SimSpawn { name, x, z, .. } = spawn;
        if !self.lot.in_bounds(x.floor() as i32, z.floor() as i32) {
            return Err(Error::new(format!("resident '{name}' spawns off the lot")));
        }
        let slot = if reuse { self.sims.iter().position(|s| !s.here()) } else { None };
        if slot.is_none() && self.sims.len() >= MAX_SIMS {
            return Err(Error::new(format!("at most {MAX_SIMS} residents per town")));
        }
        let id = slot.unwrap_or(self.sims.len());
        if spawn.household as usize >= self.households.len() {
            return Err(Error::new(format!(
                "{name}: unknown household {}",
                spawn.household
            )));
        }
        let content = &self.content;
        let gender = spawn
            .gender
            .clone()
            .or_else(|| content.genders.first().map(|g| g.id.clone()))
            .unwrap_or_default();
        let attracted_to = spawn
            .attracted_to
            .clone()
            .unwrap_or_else(|| content.genders.iter().map(|g| g.id.clone()).collect());
        content
            .validate_identity(&gender, &attracted_to)
            .map_err(|e| Error::new(format!("{name}: {e}")))?;
        let mods = content
            .character_modifiers(&spawn.traits, &spawn.perks)
            .map_err(|e| Error::new(format!("{name}: {e}")))?;
        let gender_ix = content.genders.iter().position(|g| g.id == gender);
        let attraction = content
            .genders
            .iter()
            .enumerate()
            .filter(|(_, g)| attracted_to.contains(&g.id))
            .fold(0u32, |m, (i, _)| m | (1 << i));

        let young = !content.life.adult(spawn.age.unwrap_or(content.life.start_age[0]));
        let job = match &spawn.job {
            None => None,
            Some(_) if young => None,
            Some(j) => {
                let career = content
                    .career_index(&j.career)
                    .ok_or_else(|| Error::new(format!("{name}: unknown career '{}'", j.career)))?;
                let level = j.level.min(content.careers[career].levels.len() - 1);
                Some(crate::life::Job::new(career, level, 0))
            }
        };
        // Traits give a head start; explicit levels win; anyone placed in a job has the skills for it.
        let mut skills = content.starting_skills(&spawn.traits);
        for (id, level) in &spawn.skills {
            let s = content
                .skill_index(id)
                .ok_or_else(|| Error::new(format!("{name}: unknown skill '{id}'")))?;
            skills[s] = *level;
        }
        if let Some(j) = &job {
            for &(s, level) in &content.careers[j.career].levels[j.level].requires {
                skills[s] = skills[s].max(level);
            }
        }
        let max = content.skill_rules.max_level;
        for s in &mut skills {
            *s = s.clamp(0.0, max);
        }

        let age = spawn
            .age
            .unwrap_or_else(|| crate::lifecycle::default_age(&content.life, name, id))
            .clamp(0.0, 150.0);
        // At or past the retirement age: retired already, with the pension of a job they were
        // given (at least the basic pension).
        let retired = content.life.retire_at.is_some_and(|at| age >= at);
        let pension = if retired { crate::lifecycle::pension(content, job.as_ref()) } else { 0 };
        let job = job.filter(|_| !retired);
        let mut needs = [0.0; MAX_NEEDS];
        for n in needs.iter_mut().take(self.content.needs.len()) {
            *n = self.rng.range(0.55, 0.95);
        }
        let sim = Sim {
            id: id as u32,
            name: name.clone(),
            appearance: spawn.appearance.clone(),
            gender,
            attracted_to,
            traits: spawn.traits.clone(),
            perks: spawn.perks.clone(),
            base_mods: mods.clone(),
            mods,
            buffs: Vec::new(),
            pending_spend: 0,
            pos: [*x, *z],
            yaw: 0.0,
            pose: Pose::Stand,
            seated_from: None,
            use_seat: None,
            needs,
            feelings: Vec::new(),
            engaged_with: None,
            heeding: None,
            gender_ix,
            attraction,
            queue: VecDeque::new(),
            current: None,
            idle_ticks: 0,
            follow_up: None,
            unreachable: [None; UNREACHABLE],
            social_waited: 0,
            called: false,
            nav_seen: 0,
            household: spawn.household,
            job,
            away_until: None,
            visiting: None,
            transition: None,
            skills,
            skill_ups: 0,
            practice_fatigue: 0.0,
            nap_until: 0,
            job_search_from: 0,
            planner: Default::default(),
            mess: None,
            reshaped: false,
            care_out: None,
            night_woken: 0,
            conceived: None,
            news: None,
            accident_at: [0; crate::content::MAX_ACCIDENTS],
            empty_since: [0; crate::content::MAX_ACCIDENTS],
            age,
            gone: None,
            held: None,
            invited_until: 0,
            served: None,
            hung_up: None,
            retired,
            pension,
            school_day: 0,
            grade: content.life.school.as_ref().map_or(0.0, |s| s.start_grade),
            steps: 1,
            owed: 0,
        };
        if let Some(slot) = slot {
            self.retire_slot(slot);
            self.sims[slot] = sim;
            self.relationships.clear(slot);
            self.generations[slot] = self.generations[slot].wrapping_add(1);
        } else {
            self.sims.push(sim);
            self.relationships.grow(self.sims.len());
            self.generations.push(0);
        }
        self.refresh_base_mods(id);
        Ok(id as u32)
    }

    /// Before someone new takes `slot`: story events naming its former occupant name them as a
    /// former resident instead; former residents no event names any more are forgotten.
    fn retire_slot(&mut self, slot: usize) {
        use crate::social::FORMER;
        let gone = &self.sims[slot];
        let household = self.households.get(gone.household as usize).map_or_else(String::new, |h| h.name.clone());
        self.former.push(Former { name: gone.name.clone(), household });
        let k = (self.former.len() - 1) as u32;
        self.events.rewrite_refs(|r| {
            if *r == slot as u32 {
                *r = FORMER | k;
            }
        });
        let mut used = vec![false; self.former.len()];
        self.events.rewrite_refs(|r| {
            if *r & FORMER != 0 {
                used[(*r & !FORMER) as usize] = true;
            }
        });
        let mut remap = vec![0u32; self.former.len()];
        let mut kept = Vec::new();
        for (i, f) in self.former.drain(..).enumerate() {
            if used[i] {
                remap[i] = kept.len() as u32;
                kept.push(f);
            }
        }
        self.former = kept;
        self.events.rewrite_refs(|r| {
            if *r & FORMER != 0 {
                *r = FORMER | remap[(*r & !FORMER) as usize];
            }
        });
    }

    /// A resident leaves town for good (`why`): whatever they were doing stops, nobody waits
    /// for them, their relationships are forgotten, and their slot is kept (ids don't change)
    /// until a newcomer takes it.
    pub fn depart(&mut self, i: usize, why: GoneWhy) {
        let id = i as u32;
        for o in &mut self.objects {
            o.release(id);
        }
        let day = crate::clock::day(self.tick);
        let sim = &mut self.sims[i];
        sim.clear_activity();
        sim.away_until = None;
        sim.job = None;
        sim.planner.run = None;
        sim.planner.thought = None;
        sim.gone = Some(Gone { why, day });
        // Nobody talks to them, listens in, opens the door to them or rings them any more (their
        // slot may soon be someone else's), and a visitor waits for someone else to come.
        let with_them = |t: &Task| match t.kind {
            TaskKind::Social { target, .. } | TaskKind::Join { leader: target } | TaskKind::Answer { guest: target } => target == id,
            TaskKind::Phone { contact, .. } => contact == Some(id),
            _ => false,
        };
        let World { sims, content, objects, households, .. } = self;
        for other in sims.iter_mut() {
            if other.engaged_with == Some(id) {
                other.engaged_with = None;
            }
            if other.heeding == Some(id) {
                other.heeding = None;
            }
            other.queue.retain(|t| !with_them(t));
            if other.current.as_ref().is_some_and(|a| with_them(&a.task)) {
                end_activity(other, content, objects);
            }
            if let Some(Activity { phase: Phase::Knocking { host, .. }, .. }) = other.current.as_mut()
                && *host == Some(id)
            {
                *host = None;
            }
        }
        // A baby on the way with them is no longer (nobody's named as a parent who isn't here).
        for h in households.iter_mut() {
            if h.expecting.is_some_and(|e| e.parents.contains(&id)) {
                h.expecting = None;
            }
        }
        self.relationships.clear(i);
        self.structure_version = self.structure_version.wrapping_add(1);
        self.rooms_need_clean_rescore();
    }

    /// A resident's own modifiers: traits and perks, and what their life stage does (then the
    /// active feelings on top, as `Sim::refresh_buffs` does).
    pub(crate) fn refresh_base_mods(&mut self, i: usize) {
        let content = &self.content;
        let sim = &mut self.sims[i];
        let mut mods = content.character_modifiers(&sim.traits, &sim.perks).unwrap_or_default();
        if let Some(stage) = content.life.stages.get(content.life.stage(sim.age)) {
            mods.combine(&stage.mods);
        }
        sim.mods = mods.clone();
        for &m in &sim.buffs {
            if let Some(e) = &content.feelings[m].effects {
                sim.mods.combine(e);
            }
        }
        sim.base_mods = mods;
    }

    /// New residents move into household `household`'s home (see `Command::MoveIn`): all of
    /// them or, if any can't (unknown traits, a bad bond), none.
    pub fn move_in(
        &mut self,
        household: u32,
        name: Option<String>,
        sims: &[SimSpawn],
        bonds: &[BondRaw],
    ) -> Result<(), Error> {
        let (h, _) = self.home_of(household)?;
        if sims.is_empty() {
            return Err(Error::new("nobody to move in"));
        }
        let members = self.sims.iter().filter(|s| s.here() && s.household as usize == h).count();
        let most = self.content.rules.max_household;
        if members + sims.len() > most {
            return Err(Error::new(format!("at most {most} residents per household")));
        }
        if self.sims.iter().filter(|s| s.here()).count() + sims.len() > MAX_SIMS {
            return Err(Error::new(format!("at most {MAX_SIMS} residents per town")));
        }
        if bonds.iter().any(|b| b.a >= sims.len() || b.b >= sims.len()) {
            return Err(Error::new("a bond refers to someone who isn't moving in"));
        }
        let life = &self.content.life;
        let grown_up = |s: &SimSpawn| s.age.is_none_or(|a| life.adult(a));
        let adult_at_home = self.sims.iter().any(|s| s.here() && s.household as usize == h && s.adult(&self.content));
        if !adult_at_home && !sims.iter().any(grown_up) {
            return Err(Error::new("children need a grown-up to live with"));
        }
        // Newcomers may take slots of people who are gone, so a failure restores all of it.
        let before = (
            self.sims.clone(),
            self.relationships.clone(),
            self.rng.clone(),
            self.former.clone(),
            self.generations.clone(),
            self.events.clone(),
        );
        let mut ids = Vec::with_capacity(sims.len());
        let moved = sims
            .iter()
            .try_for_each(|s| {
                let spawn = SimSpawn { household, ..s.clone() };
                ids.push(self.spawn_at(&spawn, true)? as usize);
                Ok(())
            })
            .and_then(|()| {
                let bonds: Vec<BondRaw> = bonds
                    .iter()
                    .map(|b| BondRaw { a: ids[b.a], b: ids[b.b], preset: b.preset.clone() })
                    .collect();
                self.init_relationships_for(&ids, &bonds)
            });
        if let Err(e) = moved {
            (self.sims, self.relationships, self.rng, self.former, self.generations, self.events) = before;
            return Err(e);
        }
        let first = ids[0];
        crate::lifecycle::cribs_for_babies(self, h);
        if let Some(name) = name.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
            self.households[h].name = name.to_string();
        }
        self.structure_version = self.structure_version.wrapping_add(1);
        self.rooms_need_clean_rescore();
        self.events
            .push_detail(self.tick, social::EventKind::MovedIn, first, None, None);
        Ok(())
    }

    /// Chemistry for every pair, household bonds, then explicit bonds from the lot file.
    pub(crate) fn init_relationships(&mut self, bonds: &[BondRaw]) -> Result<(), Error> {
        let all: Vec<usize> = (0..self.sims.len()).collect();
        self.init_relationships_for(&all, bonds)
    }

    /// Chemistry and household bonds for every pair with one of the `new` residents in it (the
    /// others already have theirs; the gone have none), then `bonds` (indices into the list).
    pub(crate) fn init_relationships_for(&mut self, new: &[usize], bonds: &[BondRaw]) -> Result<(), Error> {
        let n = self.sims.len();
        let default_bond = self.content.social_rules.default_bond;
        for a in 0..n {
            for b in a + 1..n {
                if !(new.contains(&a) || new.contains(&b)) || !self.sims[a].here() || !self.sims[b].here() {
                    continue;
                }
                let (sa, sb) = (&self.sims[a], &self.sims[b]);
                let shared = sa.traits.iter().filter(|t| sb.traits.contains(t)).count() as f32;
                let clashes = sa
                    .traits
                    .iter()
                    .filter_map(|t| self.content.traits.iter().find(|d| &d.id == t))
                    .map(|d| d.conflicts.iter().filter(|c| sb.traits.contains(c)).count())
                    .sum::<usize>() as f32;
                let same_household = sa.household == sb.household;
                let chemistry = self.rng.range(-0.6, 0.6) + shared * 0.15 - clashes * 0.3;
                self.relationships.set_chemistry(a, b, chemistry);
                if same_household && let Some(p) = default_bond {
                    self.relationships.bond(a, b, &p);
                }
            }
        }
        for bond in bonds {
            let preset = self
                .content
                .bond_presets
                .get(&bond.preset)
                .ok_or_else(|| Error::new(format!("unknown bond preset '{}'", bond.preset)))?;
            if bond.a >= n || bond.b >= n || bond.a == bond.b {
                return Err(Error::new(format!(
                    "bond {}-{} refers to unknown residents",
                    bond.a, bond.b
                )));
            }
            if preset.partners
                && [bond.a, bond.b].iter().any(|&s| {
                    self.relationships
                        .partner_of(s)
                        .is_some_and(|p| p != bond.a && p != bond.b)
                })
            {
                return Err(Error::new("a resident can only have one partner"));
            }
            self.relationships.bond(bond.a, bond.b, preset);
        }
        Ok(())
    }

    /// Advances one real-time step (1 / TICKS_PER_SECOND s) at the current speed, faster
    /// while it's calm at home and `auto_fast` is on.
    pub fn advance(&mut self) {
        let normal = TICKS_PER_STEP[self.speed as usize];
        let most = if self.auto_fast && normal > 0 {
            normal.max(TICKS_PER_STEP[AUTO_FAST_SPEED as usize])
        } else {
            normal
        };
        for i in 0..most {
            if i >= normal && !self.calm() {
                break;
            }
            self.tick_once();
        }
    }

    /// Whether time may skip ahead: everyone in the player's household is asleep or away at
    /// work, and nothing notable happened to them in the last game hour.
    pub fn calm(&self) -> bool {
        let home = |h: u32| self.households[h as usize].player;
        let mut members = self.sims.iter().filter(|s| s.here() && home(s.household)).peekable();
        if members.peek().is_none() {
            return false;
        }
        let quiet = members.all(|s| s.away_until.is_some() || s.asleep(&self.content));
        let hour_ago = self.tick.saturating_sub(TICKS_PER_HOUR);
        quiet
            && !self.events.iter().rev().take_while(|e| e.tick >= hour_ago).any(|e| {
                e.kind.importance() >= 1
                    && [Some(e.a), Some(e.b), e.c]
                        .into_iter()
                        .flatten()
                        .any(|s| self.sims.get(s as usize).is_some_and(|s| home(s.household)))
            })
    }

    /// Every room and garden of the town with its scores (current as of the last refresh).
    pub fn rooms(&self) -> &[crate::rooms::RoomInfo] {
        &self.rooms
    }

    /// The room (or garden) a tile belongs to.
    pub fn room_at_tile(&self, x: i32, z: i32) -> Option<&crate::rooms::RoomInfo> {
        if !self.lot.in_bounds(x, z) {
            return None;
        }
        let id = self.lot.room_at(x, z);
        if id == crate::lot::OUTDOORS {
            let plot = self.plot_at(x, z)?;
            self.rooms.iter().find(|r| r.garden && r.plot == Some(plot))
        } else {
            self.rooms.iter().find(|r| !r.garden && r.id == id)
        }
    }

    /// Mess residents made, and cleaning they finished, this tick: a mess spreads over the
    /// tile and (less) the tiles around it in the same room; cleaning clears them all.
    fn apply_messes(&mut self) {
        for i in 0..self.sims.len() {
            if let Some(partner) = self.sims[i].conceived.take() {
                crate::lifecycle::conceive(self, i, partner as usize);
            }
            if let Some((baby, gains)) = self.sims[i].care_out.take()
                && let Some(b) = self.sims.get_mut(baby as usize)
            {
                for (n, g) in gains.iter().enumerate() {
                    b.needs[n] = (b.needs[n] + g).min(1.0);
                }
            }
            if std::mem::take(&mut self.sims[i].reshaped) {
                // An object opened or closed: how it looks changes.
                self.structure_version = self.structure_version.wrapping_add(1);
                self.rooms_need_clean_rescore();
            }
            if let Some(news) = self.sims[i].news.take() {
                // Broken or mended: what the house offers and how it looks change.
                self.structure_version = self.structure_version.wrapping_add(1);
                self.rooms_need_rescore();
                let (kind, n) = match news {
                    News::Broke(o) => (social::EventKind::Broke, self.objects[o as usize].def as i64),
                    News::Repaired(o) => (social::EventKind::Repaired, self.objects[o as usize].def as i64),
                    News::Accident(a) => (social::EventKind::Accident, a as i64),
                    News::NotClosed(o) => (social::EventKind::NotClosed, self.objects[o as usize].def as i64),
                };
                self.events.push_detail(self.tick, kind, i, Some(n), None);
            }
            let Some((x, z, amount)) = self.sims[i].mess.take() else {
                continue;
            };
            if !self.lot.in_bounds(x, z) {
                continue;
            }
            let room = self.lot.room_at(x, z);
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let (tx, tz) = (x + dx, z + dz);
                    if !self.lot.in_bounds(tx, tz) || self.lot.room_at(tx, tz) != room {
                        continue;
                    }
                    let share = if amount < 0.0 || (dx, dz) == (0, 0) { 1.0 } else if dx == 0 || dz == 0 { 0.5 } else { 0.25 };
                    let t = self.lot.tile_index(tx, tz);
                    self.dirt[t] = (self.dirt[t] + amount * share).clamp(0.0, 1.0);
                    self.room_tiles.touch(t);
                }
            }
        }
    }

    /// A baby crying (content `dayRhythm.babyWakes`) in the crib at home with every grown-up of
    /// the household there asleep wakes one of them: family first, then someone not worn out,
    /// then whoever's turn it is (woken longest ago: partners take turns), then the most rested.
    /// At night, a grown-up there who's up and free to drop what they're doing
    /// (`free_for_baby`) comes at once instead (a cook nearly done with a step finishes it
    /// first): family first, then the nearest; with nobody up who can (in the shower, told to
    /// do something, about to leave for work), one asleep is woken. They do the crib's care for
    /// each need that's low (the woken go back to bed after: night still picks it); and the same
    /// baby calls nobody again for a while. In the daytime, with someone at home awake, they see
    /// to the baby themselves (autonomy, `pick_autonomous`).
    fn wake_for_crying_babies(&mut self) {
        let Some(rules) = self.content.day_rhythm.baby_wakes else { return };
        for b in 0..self.sims.len() {
            let Some((carer, tasks)) = self.woken_for(b, &rules) else { continue };
            let content = &self.content;
            let sim = &mut self.sims[carer];
            let woken = sim.asleep(content);
            // A cook halfway through a step finishes it first (`cooking::called_away`).
            if !crate::cooking::called_away(sim, content, &mut self.objects) {
                end_activity(sim, content, &mut self.objects);
            }
            sim.follow_up = None;
            let care = tasks.len();
            for task in tasks.into_iter().rev() {
                sim.queue.push_front(task);
            }
            trim_queue(&mut sim.queue, care);
            if woken {
                sim.night_woken = self.tick;
                if let Some(f) = rules.feeling {
                    social::add_feeling(&mut sim.feelings, f, &content.feelings, self.tick);
                }
            }
            self.sims[b].night_woken = self.tick;
        }
    }

    /// Who sees to baby `b`'s crying (`wake_for_crying_babies`), and the care they do.
    fn woken_for(&self, b: usize, rules: &crate::content::BabyWakes) -> Option<(usize, Vec<Task>)> {
        let content = &self.content;
        let baby = &self.sims[b];
        let cooling = baby.night_woken != 0 && self.tick < baby.night_woken + rules.cooldown_ticks;
        if !baby.here() || baby.away_until.is_some() || !content.life.baby(baby.age) || cooling {
            return None;
        }
        let wants = |n: usize, below: f32| rules.needs & (1 << n) != 0 && baby.needs[n] < below;
        if !(0..content.needs.len()).any(|n| wants(n, rules.below)) {
            return None;
        }
        let h = baby.household as usize;
        let plot = self.households[h].plot.filter(|_| self.autonomy && self.households[h].free_will)?;
        // In a crib at home, with care for what they're crying for.
        let crib = self.objects.iter().find(|o| o.users().any(|u| u as usize == b) && self.object_plot[o.id as usize] == Some(plot))?;
        let def = &content.objects[crib.def];
        let care = |i: &crate::content::Interaction, below: f32| {
            i.cares() && (0..content.needs.len()).any(|n| i.care[n] > 0.0 && wants(n, below))
        };
        if !def.interactions.iter().any(|i| care(i, rules.below)) || !crib.has_free_slot(def.slots) {
            return None;
        }
        // The grown-ups at home.
        let home: Vec<usize> = (0..self.sims.len())
            .filter(|&j| {
                let s = &self.sims[j];
                let (x, z) = s.tile();
                s.here() && s.household as usize == h && s.away_until.is_none() && s.visiting.is_none()
                    && s.adult(content) && self.plot_at(x, z) == Some(plot)
            })
            .collect();
        // Nothing to do if someone's already doing the crib's care (or on their way to).
        let tending = |t: &Task| matches!(t.kind, TaskKind::Use { object, interaction } if object == crib.id && def.interactions[interaction].cares());
        if home.iter().any(|&j| self.sims[j].current().is_some_and(|a| tending(&a.task)) || self.sims[j].queue.iter().any(tending)) {
            return None;
        }
        let carer = if home.iter().all(|&j| self.sims[j].asleep(content)) {
            self.to_wake(b, &home)?
        } else {
            // Up at night: whoever's free drops what they're doing; by day, autonomy.
            let rhythm = &content.day_rhythm;
            if !rhythm.is_night(crate::clock::hour(self.tick), rhythm.wake_hour) {
                return None;
            }
            let family = |j: usize| self.relationships.kin(j, b) != crate::social::Kin::None;
            let (cx, cz) = crib.front_tile(content);
            let far = |j: usize| {
                let p = self.sims[j].pos;
                (p[0] - cx as f32 - 0.5).powi(2) + (p[1] - cz as f32 - 0.5).powi(2)
            };
            let up = home
                .iter()
                .copied()
                .filter(|&j| self.free_for_baby(j, rules))
                .max_by(|&x, &y| family(x).cmp(&family(y)).then(far(y).total_cmp(&far(x))));
            up.or_else(|| self.to_wake(b, &home))?
        };
        let tasks = def
            .interactions
            .iter()
            .enumerate()
            .filter(|(_, i)| care(i, rules.tend_below))
            .map(|(i, _)| Task { kind: TaskKind::Use { object: crib.id, interaction: i }, directed: false, routine: false })
            .collect();
        Some((carer, tasks))
    }

    /// Which of the grown-ups at `home` who are asleep baby `b`'s crying wakes
    /// (`wake_for_crying_babies`).
    fn to_wake(&self, b: usize, home: &[usize]) -> Option<usize> {
        let content = &self.content;
        let rhythm = &content.day_rhythm;
        let rest = |j: usize| rhythm.rest_need.map_or(1.0, |n| self.sims[j].needs[n]);
        let key = |j: usize| {
            let family = self.relationships.kin(j, b) != crate::social::Kin::None;
            (family, rest(j) >= rhythm.exhausted_below, std::cmp::Reverse(self.sims[j].night_woken))
        };
        home.iter()
            .copied()
            .filter(|&j| self.sims[j].asleep(content))
            .max_by(|&x, &y| key(x).cmp(&key(y)).then(rest(x).total_cmp(&rest(y))))
    }

    /// Whether grown-up `j`, up at home, can see to a crying baby now: doing nothing, or
    /// something they picked themselves (on their way to bed too; eating, the plate left where
    /// it is; cooking, with at most `BabyWakes::finish_minutes` of the step left, see
    /// `cooking::called_away`; not the toilet or shower, washing up, or a conversation), with
    /// nothing the player asked for next, not pressed themselves, and not leaving for work or
    /// school within `DayRhythm::ready_minutes`.
    fn free_for_baby(&self, j: usize, rules: &crate::content::BabyWakes) -> bool {
        let content = &self.content;
        let sim = &self.sims[j];
        if sim.asleep(content) || sim.engaged_with.is_some() || sim.pressed(content) {
            return false;
        }
        let picked = sim.current().is_none_or(|a| {
            let drop = match a.task.kind {
                TaskKind::Use { .. } | TaskKind::MoveTo { .. } | TaskKind::Clean { .. } | TaskKind::Eat { .. } => true,
                TaskKind::Cook { .. } => crate::cooking::step_left(sim).is_some_and(|m| m <= rules.finish_minutes),
                _ => false,
            };
            !a.task.directed && drop && a.tags & content.social_rules.busy_tags == 0
        });
        let next = sim.queue.front().is_some_and(|t| t.directed || matches!(t.kind, TaskKind::Work));
        let ready = social::minutes_to_ticks(content.day_rhythm.ready_minutes);
        let leaving = leave_at(sim, content, self.tick).is_some_and(|leave| leave <= self.tick + ready);
        picked && !next && !leaving
    }

    /// Rebuilds the objects-by-plot index if objects were added, removed or changed since.
    pub(crate) fn refresh_index(&mut self) {
        if self.object_index.version != self.nav_version {
            self.object_index = ObjectIndex::new(&self.object_plot, &self.plots, self.nav_version);
            let nav = NavGrid { lot: &self.lot, blocked: &self.blocked, stairs: &self.stairs };
            self.object_index.links = crate::surface::Links::new(&self.content, &self.objects, &nav);
        }
    }

    /// Labels the walkable regions of the lot again if the navigation changed (see
    /// `path::Reach`): an unreachable goal is then refused before a search can flood the town.
    fn refresh_reach(&mut self) {
        if self.reach_version == self.nav_version {
            return;
        }
        let reach = {
            let nav = NavGrid { lot: &self.lot, blocked: &self.blocked, stairs: &self.stairs };
            crate::path::Reach::build(&nav)
        };
        self.reach = reach;
        self.reach_version = self.nav_version;
    }

    /// Notes that something walkers can or can't step through changed, which views also see.
    pub(crate) fn nav_changed(&mut self) {
        self.structure_version = self.structure_version.wrapping_add(1);
        self.nav_version = self.nav_version.wrapping_add(1);
    }

    /// Notes that the lot's rooms or their scores changed structurally (walls, coverings,
    /// objects added or removed): the next tick recomputes the rooms.
    pub(crate) fn rooms_layout_changed(&mut self) {
        self.room_layout_version = self.room_layout_version.wrapping_add(1);
    }

    /// Notes that a room's scores changed without its layout (something broke, was repaired
    /// or upgraded): the next tick recomputes the rooms once.
    pub(crate) fn rooms_need_rescore(&mut self) {
        self.rooms_dirty = true;
    }

    /// Notes that only cleanliness may have changed (a crop grew, a thing opened): the next
    /// tick rescores the rooms' cleanliness without walking the whole lot.
    pub(crate) fn rooms_need_clean_rescore(&mut self) {
        self.rooms_rescore = true;
    }

    /// Recomputes rooms and their scores (see `rooms.rs`). Call it after changing `dirt`
    /// directly; messes residents make are rescored on their own.
    pub fn refresh_rooms(&mut self) {
        if self.dirt.len() != self.lot.width * self.lot.depth {
            self.dirt = vec![0.0; self.lot.width * self.lot.depth];
        }
        self.map_plots();
        let (rooms, cells) = crate::rooms::compute(self);
        self.rooms = rooms;
        self.rooms_version = self.room_layout_version;
        self.rooms_dirty = false;
        self.rooms_rescore = false;
        // The room (index into `rooms`) whose score each tile takes: its plot's garden, then
        // indoor tiles anywhere their room; off the plots, none (`roomRules.away`).
        let rooms = &self.rooms;
        let mut source = vec![None; self.lot.width * self.lot.depth];
        for (pi, p) in self.plots.iter().enumerate() {
            let garden = rooms.iter().position(|r| r.garden && r.plot == Some(pi as u32));
            for z in p.z..p.z + p.d {
                for x in p.x..p.x + p.w {
                    if self.lot.in_bounds(x, z) {
                        source[self.lot.tile_index(x, z)] = garden;
                    }
                }
            }
        }
        let max_id = rooms.iter().filter(|r| !r.garden).map(|r| r.id as usize).max().unwrap_or(0);
        let mut by_id = vec![None; max_id + 1];
        for (k, r) in rooms.iter().enumerate().filter(|(_, r)| !r.garden) {
            by_id[r.id as usize] = Some(k);
        }
        for (i, &id) in self.lot.rooms().iter().enumerate() {
            if let Some(Some(k)) = by_id.get(id as usize).filter(|_| id != crate::lot::OUTDOORS) {
                source[i] = Some(*k);
            }
        }
        let away = self.content.room_rules.away;
        self.room_score = source.iter().map(|s| s.map_or(away, |k| rooms[k].scores.overall)).collect();
        self.room_tiles = crate::rooms::RoomTiles::new(cells, &source);
    }

    /// Scores cleanliness again in rooms that had a mess since: between structure changes
    /// only the dirt changes, so the rooms needn't be found again (a whole-town job).
    fn rescore_rooms(&mut self) {
        self.room_tiles.rescore(&mut self.rooms, &self.content.room_rules, &self.dirt, &mut self.room_score);
    }

    pub fn tick_once(&mut self) {
        // Time moves on: edits can no longer be taken back.
        self.undo.clear();
        self.redo.clear();
        if self.rooms_version != self.room_layout_version || self.rooms_dirty {
            self.refresh_rooms();
        } else if self.rooms_rescore {
            self.rooms_rescore = false;
            self.rescore_rooms();
        } else if self.tick.is_multiple_of(ROOM_REFRESH_TICKS) {
            self.rescore_rooms();
        }
        self.refresh_reach();
        self.refresh_index();
        // A new game's first step: the player hears of anyone without a bed (told as the world was
        // made, it would be the story so far, which the view doesn't show as news).
        if self.tick == 0 {
            for h in 0..self.households.len() {
                crate::lifecycle::notify_missing_beds(self, h);
            }
        }
        self.tick += 1;
        let tick = self.tick;

        let mut briefs = std::mem::take(&mut self.briefs);
        briefs.clear();
        for s in &self.sims {
            let (x, z) = s.tile();
            let plot = self.plot_at(x, z);
            briefs.push(SimBrief {
                pos: s.pos,
                available: s.here() && s.available_for_social(&self.content),
                gender_ix: s.gender_ix,
                household: s.household,
                plot: plot.filter(|_| s.here()),
                away: s.away_until.is_some() || !s.here(),
                leaving: crate::life::leaving_home(self, s),
                baby: s.here() && self.content.life.baby(s.age),
                needs: s.needs,
                here: s.here(),
                partnered: self.relationships.partner_of(s.id as usize).is_some(),
                adult: s.adult(&self.content),
                age: s.age,
                conversing: None,
                cleaning: s.current().and_then(|a| if let TaskKind::Clean { x, z } = a.task.kind { Some((x, z)) } else { None }),
            });
        }
        // Who's in a conversation, and whose (for joining in).
        for (i, s) in self.sims.iter().enumerate() {
            if let Some((t, social, ..)) = s.conversation(&self.content) {
                briefs[i].conversing = Some((i as u32, social));
                briefs[t as usize].conversing = Some((i as u32, social));
            }
        }
        for (i, s) in self.sims.iter().enumerate() {
            if let Some(l) = s.listening_to() {
                briefs[i].conversing = briefs[l as usize].conversing.filter(|c| c.0 == l);
            }
        }
        self.briefs = briefs;
        crate::lod::plan(self);
        let hour = crate::clock::hour(tick);
        {
            let World {
                content,
                lot,
                objects,
                items,
                sims,
                blocked,
                rng,
                relationships,
                briefs,
                autonomy,
                households,
                plots,
                tile_plot,
                object_plot,
                object_index,
                exits,
                room_score,
                dirt,
                stairs,
                reach,
                nav_version,
                ..
            } = self;
            let ctx = Ctx {
                room_score,
                dirt,
                content,
                nav: NavGrid { lot, blocked, stairs },
                reach,
                nav_version: *nav_version,
                rels: relationships,
                briefs,
                households,
                plots,
                tile_plot,
                object_plot,
                index: object_index,
                exits,
                autonomy: *autonomy,
                hour,
                day: crate::clock::day(tick),
                minute: crate::clock::minute_of_day(tick),
                tick,
            };
            // (In lots nobody watches, only those whose turn it is; see `lod.rs`.)
            for sim in sims.iter_mut().filter(|s| s.here() && s.steps > 0) {
                step_sim(sim, &ctx, objects, items, rng);
            }
        }
        self.broadcast();
        self.apply_messes();
        self.wake_for_crying_babies();
        conversation::update(self);
        crate::phone::update(self);
        crate::life::update(self);
        crate::lifecycle::update(self);
        crate::planner::update(self);
        crate::cooking::after_step(self);
        crate::lights::update(self);
        if self.tick.is_multiple_of(TICKS_PER_HOUR) {
            crate::rooms::opinions(self);
        }

        for s in self.sims.iter_mut().filter(|s| s.steps > 0) {
            s.feelings.retain(|m| m.expires > tick);
            if self.content.buff_feelings {
                s.refresh_buffs(&self.content);
            }
        }
        if tick.is_multiple_of(TICKS_PER_DAY) {
            self.relationships.decay(&self.content.social_rules);
        }
        if tick.is_multiple_of(GROWTH_STEP_MINUTES * TICKS_PER_MINUTE) {
            self.grow(GROWTH_STEP_MINUTES as f32);
        }
    }

    /// Gives nearby residents a share of what a use gives (content `broadcast`: hearing the TV
    /// from the sofa, or while eating at the table). Only positive gains carry, and not to the
    /// resident using it, who gets the whole of it.
    fn broadcast(&mut self) {
        let content = &self.content;
        struct Source {
            object: u32,
            room: u16,
            storey: i32,
            plot: Option<u32>,
            pos: [f32; 2],
            gains: [f32; MAX_NEEDS],
            share: f32,
            range: f32,
            user: u32,
        }
        let mut sources: Vec<Source> = Vec::new();
        for sim in &self.sims {
            if !sim.here() || sim.away_until.is_some() {
                continue;
            }
            let Some(act) = sim.current() else { continue };
            if !matches!(act.phase, Phase::Using { .. }) {
                continue;
            }
            let TaskKind::Use {
                object,
                interaction,
            } = act.task.kind
            else {
                continue;
            };
            let Some(obj) = self.objects.get(object as usize) else {
                continue;
            };
            let inter = &content.objects[obj.def].interactions[interaction];
            if inter.broadcast <= 0.0 {
                continue;
            }
            let pos = obj.centre(content);
            let (x, z) = (pos[0].floor() as i32, pos[1].floor() as i32);
            let ground = self.ground_z(z);
            let room = if self.lot.in_bounds(x, ground) {
                self.lot.room_at(x, ground)
            } else {
                crate::lot::OUTDOORS
            };
            let quality = content.object_rules.quality_factor(obj.quality);
            sources.push(Source {
                object,
                room,
                storey: self.storey_of(z),
                plot: self.object_plot.get(object as usize).copied().flatten(),
                pos,
                gains: inter.gain_per_minute.map(|g| g * quality),
                share: inter.broadcast,
                range: inter.broadcast_range,
                user: sim.id,
            });
        }
        if sources.is_empty() {
            return;
        }
        let rooms: Vec<u16> = self
            .sims
            .iter()
            .map(|s| {
                let (x, z) = s.tile();
                let ground = self.ground_z(z);
                if self.lot.in_bounds(x, ground) {
                    self.lot.room_at(x, ground)
                } else {
                    crate::lot::OUTDOORS
                }
            })
            .collect();
        let storeys: Vec<i32> = self
            .sims
            .iter()
            .map(|s| self.storey_of(s.tile().1))
            .collect();
        let plots: Vec<Option<u32>> = self
            .sims
            .iter()
            .map(|s| {
                let (x, z) = s.tile();
                self.plot_at(x, z)
            })
            .collect();
        let asleep: Vec<bool> = self.sims.iter().map(|s| s.asleep(content)).collect();
        for (i, sim) in self.sims.iter_mut().enumerate() {
            if !sim.here() || sim.away_until.is_some() || sim.steps == 0 || asleep[i] {
                continue;
            }
            let minutes = sim.minutes();
            // Two residents watching the same screen are one source, not two.
            let mut heard: Vec<u32> = Vec::new();
            for src in &sources {
                if src.user == sim.id
                    || heard.contains(&src.object)
                    || (src.plot.is_some() && plots[i] != src.plot)
                {
                    continue;
                }
                // Another resident watching the same screen has their own share of it already.
                if let Some(act) = sim.current()
                    && let TaskKind::Use { object, .. } = act.task.kind
                    && object == src.object
                {
                    continue;
                }
                if storeys[i] != src.storey {
                    continue;
                }
                let same_room = src.room != crate::lot::OUTDOORS && rooms[i] == src.room;
                let near = (sim.pos[0] - src.pos[0]).hypot(sim.pos[1] - src.pos[1]) <= src.range;
                if !same_room && !near {
                    continue;
                }
                heard.push(src.object);
                for n in 0..content.needs.len() {
                    let gain = src.gains[n] * src.share * minutes * sim.mods.need_gain[n];
                    if gain > 0.0 {
                        sim.needs[n] = (sim.needs[n] + gain).clamp(0.0, 1.0);
                    }
                }
            }
        }
    }

    /// Crops grow by `minutes`: fallow ground is planted again, what's planted ripens. A crop
    /// that shows another stage changes how the lot looks.
    fn grow(&mut self, minutes: f32) {
        let mut changed = false;
        for o in &mut self.objects {
            let Some(g) = self.content.objects[o.def].grows else { continue };
            if o.growth >= 1.0 {
                continue;
            }
            let before = o.growth_stage();
            o.growth = if o.growth < 0.0 {
                // Fallow, then planted afresh (from nothing: 0).
                let fallow = o.growth + minutes / (g.fallow_days.max(1.0 / 1440.0) * 1440.0);
                if fallow >= 0.0 { 0.0 } else { fallow }
            } else {
                (o.growth + minutes / (g.days * 1440.0)).min(1.0)
            };
            changed |= o.growth_stage() != before;
        }
        if changed {
            self.structure_version = self.structure_version.wrapping_add(1);
            self.rooms_need_clean_rescore();
        }
    }

    /// Applies a player command. Build and buy edits can be undone (`Command::Undo`) until time
    /// moves on or the player gives the household another kind of order. A command naming an
    /// object that's gone since the view showed it (sold, undone: an id handed out before and
    /// not there now) does nothing; an id never handed out is an error.
    pub fn apply(&mut self, mut cmd: Command) -> Result<(), Error> {
        // A command names objects by their stable ids; the simulation works in indices.
        if let Some(object) = cmd.object_mut() {
            let stable = *object;
            match self.index_of(stable) {
                Some(i) => *object = i as u32,
                None if stable < self.next_object_id => return Ok(()),
                None => return Err(Error::new(format!("unknown object {stable}"))),
            }
        }
        let history = match cmd {
            Command::Undo { household } => Some(self.undo(household)),
            Command::Redo { household } => Some(self.redo(household)),
            _ => None,
        };
        if let Some(result) = history {
            result?;
            self.refresh_rooms();
            return Ok(());
        }
        let snapshot = cmd.home_edit().and_then(|h| self.home_snapshot(h));
        let keeps_history = matches!(
            cmd,
            Command::SetSpeed { .. }
                | Command::SetLifespan { .. }
                | Command::SetPlayerMoves { .. }
                | Command::AddFunds { .. }
                | Command::SetSkill { .. }
                | Command::SetAutonomy { .. }
                | Command::SetAutoFast { .. }
                | Command::SetRoutines { .. }
                | Command::SetHouseholdRoutines { .. }
                | Command::SkipHouseholdRoutine { .. }
                | Command::AddGoal { .. }
                | Command::RemoveGoal { .. }
                | Command::MoveGoal { .. }
                | Command::AcceptSuggestion { .. }
                | Command::DismissSuggestion { .. }
                | Command::DeclinePurchase { .. }
        );
        self.apply_command(cmd)?;
        if self.rooms_version != self.room_layout_version || self.rooms_dirty {
            self.refresh_rooms();
        } else if self.rooms_rescore {
            self.rooms_rescore = false;
            self.rescore_rooms();
        }
        // Someone an order or an edit stopped (the sink they carried dishes to was moved) puts
        // down what they carry now, not when time moves on (Build mode stands still).
        if self.sims.iter().any(|s| s.held.is_some()) {
            crate::cooking::settle(self);
        }
        match snapshot {
            Some(s) => self.remember(s),
            None if !keeps_history => {
                self.undo.clear();
                self.redo.clear();
            }
            None => {}
        }
        Ok(())
    }

    fn apply_command(&mut self, cmd: Command) -> Result<(), Error> {
        match cmd {
            Command::SetSpeed { speed } => {
                if speed > MAX_SPEED {
                    return Err(Error::new(format!("speed must be 0..={MAX_SPEED}")));
                }
                self.speed = speed;
            }
            Command::SetAutoFast { enabled } => self.auto_fast = enabled,
            Command::SetRoutines { sim, routines } => {
                let routines = crate::planner::routines_from(&self.content, &routines)?;
                let s = self.sim_mut(sim)?;
                s.planner.routines = routines;
                // A changed block starts over (with today's plan).
                s.planner.run = None;
            }
            Command::SetHouseholdRoutines { sim, routines } => {
                let routines = crate::planner::routines_from(&self.content, &routines)?;
                let h = self.sim(sim)?.household as usize;
                self.households[h].routines = routines;
                for s in self.sims.iter_mut().filter(|s| s.household as usize == h) {
                    s.planner.run = None;
                }
            }
            Command::SkipHouseholdRoutine { sim, routine, skip } => {
                let skips = &mut self.sim_mut(sim)?.planner.skip_household;
                skips.retain(|&r| r != routine);
                if skip {
                    skips.push(routine);
                }
            }
            Command::AddGoal { sim, goal } => {
                let day = crate::clock::day(self.tick);
                let goal = crate::planner::goal_from(&self.content, &goal, self.sim(sim)?, day)?;
                if crate::planner::progress(self, sim as usize, &goal) >= 1.0 {
                    return Err(Error::new(format!("{} is there already", self.sims[sim as usize].name)));
                }
                let max = self.content.planner.max_goals;
                let s = self.sim_mut(sim)?;
                if s.planner.goals.len() >= max {
                    return Err(Error::new(format!("{} has {max} goals already", s.name)));
                }
                if s.planner.goals.iter().any(|g| g.same(&goal)) {
                    return Err(Error::new(format!("{} has that goal already", s.name)));
                }
                s.planner.goals.push(goal);
            }
            Command::RemoveGoal { sim, index } => {
                let goals = &mut self.sim_mut(sim)?.planner.goals;
                if index < goals.len() {
                    goals.remove(index);
                }
            }
            Command::MoveGoal { sim, index, to } => {
                let goals = &mut self.sim_mut(sim)?.planner.goals;
                if index < goals.len() && to < goals.len() {
                    let goal = goals.remove(index);
                    goals.insert(to, goal);
                }
            }
            Command::AcceptSuggestion { sim, index } => {
                let max = self.content.planner.max_goals;
                let day = crate::clock::day(self.tick);
                let s = self.sim_mut(sim)?;
                if index < s.planner.suggestions.len() {
                    if s.planner.goals.len() >= max {
                        return Err(Error::new(format!("{} has {max} goals already", s.name)));
                    }
                    let mut goal = s.planner.suggestions.remove(index);
                    goal.since_day = day;
                    // Already on the list (the player added it meanwhile): the idea just goes.
                    if !s.planner.goals.iter().any(|g| g.same(&goal)) {
                        s.planner.goals.push(goal);
                    }
                }
            }
            Command::DismissSuggestion { sim, index } => {
                let suggestions = &mut self.sim_mut(sim)?.planner.suggestions;
                if index < suggestions.len() {
                    suggestions.remove(index);
                }
            }
            Command::AcceptPurchase { sim } => {
                self.sim(sim)?;
                crate::planner::accept_purchase(self, sim as usize)?;
            }
            Command::DeclinePurchase { sim } => {
                self.sim(sim)?;
                crate::planner::decline_purchase(self, sim as usize)?;
            }
            Command::SetLifespan { lifespan } => self.lifespan = lifespan,
            Command::SetPlayerMoves { enabled } => self.player_moves = enabled,
            Command::AddFunds { household, amount } => {
                let h = self.households.get_mut(household as usize).ok_or_else(|| Error::new(format!("unknown household {household}")))?;
                h.funds = (h.funds + amount).max(0);
            }
            Command::SetSkill { sim, skill, level } => {
                self.sim(sim)?;
                let s = self.content.skill_index(&skill).ok_or_else(|| Error::new(format!("unknown skill '{skill}'")))?;
                let level = if level.is_finite() { level.clamp(0.0, self.content.skill_rules.max_level) } else { 0.0 };
                let sim = &mut self.sims[sim as usize];
                sim.skills[s] = level;
                sim.planner.recheck = true;
            }
            Command::Adopt { household, child } => {
                crate::lifecycle::adopt(self, household, child)?;
            }
            Command::SetAutonomy { enabled, household } => {
                if let Some(h) = household {
                    self.households
                        .get_mut(h as usize)
                        .ok_or_else(|| Error::new(format!("unknown household {h}")))?
                        .free_will = enabled;
                } else {
                    for h in self.households.iter_mut().filter(|h| h.player) {
                        h.free_will = enabled;
                    }
                }
            }
            Command::SwitchLamp { object, on } => {
                let obj = &self.objects[object as usize];
                if !crate::lights::switchable(self, obj.def) {
                    return Err(Error::new(format!("the {} has no switch", self.content.objects[obj.def].name.to_lowercase())));
                }
                crate::lights::set(self, object, on);
            }
            Command::Use {
                sim,
                object,
                interaction,
            } => {
                let obj = &self.objects[object as usize];
                let Some(inter) = self.content.objects[obj.def].interactions.get(interaction)
                else {
                    return Err(Error::new(format!(
                        "object {object} has no interaction {interaction}"
                    )));
                };
                if obj.broken() {
                    return Err(Error::new(format!(
                        "the {} is broken — repair it first",
                        self.content.objects[obj.def].name
                    )));
                }
                if let Some(required) = missing_requirement(&self.content, &self.object_plot, None, &self.objects, object, inter) {
                    return Err(Error::new(format!(
                        "{} needs a working object on this lot that provides {required}",
                        inter.label
                    )));
                }
                if !obj.allows(inter, sim) {
                    let name = self.content.objects[obj.def].name.to_lowercase();
                    return Err(Error::new(match inter.when {
                        Some(w) if w != obj.open && obj.open => format!("the {name} is already open"),
                        Some(w) if w != obj.open => format!("the {name} is closed"),
                        _ if inter.ripe && !obj.ripe() => format!("the {name} isn't ripe yet"),
                        _ => format!("someone is using the {name}"),
                    }));
                }
                // (Lying along the whole window seat: not with someone else on it.)
                if inter.whole && !obj.can_use_for(sim, &self.content.objects[obj.def], interaction) {
                    let name = self.content.objects[obj.def].name.to_lowercase();
                    return Err(Error::new(format!("someone is using the {name}")));
                }
                if inter.needs_table && crate::surface::faced_table(&self.content, &self.objects, &self.nav(), object).is_none() {
                    return Err(Error::new(format!(
                        "{} needs a table: pull the {} up to one",
                        inter.label,
                        self.content.objects[obj.def].name.to_lowercase()
                    )));
                }
                // Only their own home's things and public places', and what guests may use where
                // they're a guest (or on their way to be one).
                let s = self.sim(sim)?;
                let plot = self.object_plot.get(object as usize).copied().flatten();
                let visiting = |p: u32| {
                    let visit = |t: &Task| matches!(t.kind, TaskKind::Visit { plot } if plot == p);
                    s.current.as_ref().is_some_and(|a| visit(&a.task)) || s.queue.iter().any(visit)
                };
                let allowed = match order_access(&self.households, &self.plots, s, plot) {
                    Access::None if plot.is_some_and(visiting) => Access::Guest,
                    a => a,
                };
                let thing = self.content.objects[obj.def].name.to_lowercase();
                if allowed == Access::None {
                    return Err(Error::new(format!("the {thing} isn't {}'s to use", s.name)));
                }
                if allowed == Access::Guest && !guest_may(&self.content, inter) {
                    return Err(Error::new(format!("{} is a guest there, and guests don't {}", s.name, inter.label.to_lowercase())));
                }
                // Who may use what, as free will keeps it: wine and the stove not for a child,
                // the kids' things not for grown-ups, homework for pupils, the baby's care for
                // grown-ups (and only with the baby there).
                if let Some(why) = unfit(&self.content, s.age, inter) {
                    return Err(Error::new(match why {
                        Unfit::TooYoung => format!("{} is too young to {}", s.name, inter.label.to_lowercase()),
                        Unfit::TooBig => format!("{} is too big for the {thing}", s.name),
                        Unfit::NoHomework => format!("{} has no homework to do", s.name),
                        Unfit::TooYoungToCare => format!("{} is too young to look after a baby", s.name),
                    }));
                }
                if inter.cares() && !obj.users().any(|u| self.sims.get(u as usize).is_some_and(|b| self.content.life.baby(b.age))) {
                    return Err(Error::new(format!("there's no baby in the {thing}")));
                }
                if upper_barred(&self.content, obj, sim, s.age) {
                    let young = crate::content::all_older(self.content.objects[obj.def].upper_stages, self.content.life.stage(s.age));
                    return Err(Error::new(format!("{} is too {} for the top bunk", s.name, if young { "young" } else { "old" })));
                }
                let household = s.household as usize;
                if !affordable(inter.cost, self.households[household].funds) {
                    return Err(Error::new(format!(
                        "not enough money: {} costs {}",
                        inter.label, inter.cost
                    )));
                }
                // Something in a public place or at someone else's: not leaving a baby home alone.
                crate::life::check_cover_at(self, sim as usize, self.object_plot.get(object as usize).copied().flatten())?;
                self.enqueue(
                    sim,
                    TaskKind::Use {
                        object,
                        interaction,
                    },
                )?;
            }
            Command::UseItem { sim, item, action } => {
                let household = self.sim(sim)?.household;
                let it = self
                    .items
                    .get(item)
                    .filter(|it| it.household == household)
                    .ok_or_else(|| Error::new("that isn't theirs"))?;
                let kind = match action {
                    crate::command::ItemAction::Eat => {
                        if !it.edible() {
                            return Err(Error::new(if it.spoiled { "that's gone off" } else { "there's nothing left to eat" }));
                        }
                        TaskKind::Eat { item: Some(item), seat: None, plate: None, stage: 0 }
                    }
                    crate::command::ItemAction::Clear => {
                        // (With no sink, into the bin.)
                        let dishes = self.content.cooking.dishes | self.content.cooking.bin;
                        let home = self.households[household as usize].plot;
                        let sink = self.objects.iter().any(|o| {
                            self.content.objects[o.def].stations & dishes != 0 && !o.broken() && self.object_plot[o.id as usize] == home
                        });
                        if !sink {
                            return Err(Error::new("there's nowhere to wash up: buy a sink, a dishwasher or a bin"));
                        }
                        TaskKind::Dishes { item, station: None, stage: 0, scraps: 0 }
                    }
                };
                self.enqueue(sim, kind)?;
            }
            Command::MoveTo { sim, x, z } => {
                if !self.lot.in_bounds(x.floor() as i32, z.floor() as i32) {
                    return Err(Error::new("move target is off the lot"));
                }
                crate::life::check_cover_at(self, sim as usize, self.plot_at(x.floor() as i32, z.floor() as i32))?;
                self.enqueue(sim, TaskKind::MoveTo { x, z })?;
            }
            Command::Social {
                sim,
                target,
                social,
            } => {
                let (a, t) = (self.sim(sim)?, self.sim(target)?);
                let def = self
                    .content
                    .socials
                    .get(social)
                    .ok_or_else(|| Error::new(format!("unknown social {social}")))?;
                if !conversation::social_allowed(&self.content, &self.relationships, a, t, def) {
                    return Err(Error::new(format!(
                        "{} isn't possible right now",
                        def.label
                    )));
                }
                // Walking over to someone out of the house (in the park, say).
                let (tx, tz) = t.tile();
                crate::life::check_cover_at(self, sim as usize, self.plot_at(tx, tz))?;
                self.enqueue(sim, TaskKind::Social { target, social })?;
            }
            Command::JoinCareer { sim, career, level } => {
                let def = self
                    .content
                    .careers
                    .get(career)
                    .ok_or_else(|| Error::new(format!("unknown career {career}")))?;
                let position = def
                    .levels
                    .get(level)
                    .ok_or_else(|| Error::new(format!("{} has no level {level}", def.label)))?;
                if !self.sim(sim)?.adult(&self.content) {
                    return Err(Error::new(format!("{} is too young to work", self.sims[sim as usize].name)));
                }
                let s = &mut self.sims[sim as usize];
                if !crate::life::can_join(&self.content, position, &s.skills) {
                    return Err(Error::new(format!(
                        "{} doesn't have the skills to be a {} yet",
                        s.name, position.title
                    )));
                }
                s.job = Some(crate::life::Job::new(
                    career,
                    level,
                    crate::clock::day(self.tick),
                ));
                // Back to work: no more pension.
                (s.retired, s.pension) = (false, 0);
            }
            Command::QuitCareer { sim } => {
                self.sim_mut(sim)?.job = None;
            }
            Command::Visit { sim, plot } => {
                if plot as usize >= self.plots.len() {
                    return Err(Error::new(format!("unknown plot {plot}")));
                }
                // Not leaving a baby home alone with no money for a babysitter.
                crate::life::check_cover_at(self, sim as usize, Some(plot))?;
                self.enqueue(sim, TaskKind::Visit { plot })?;
            }
            Command::GoHome { sim } => {
                // A guest sent home says goodbye to one of the hosts first, if one is free (as at
                // the end of any visit), leaving whatever they chose to do themselves.
                let s = self.sim(sim)?;
                let goodbye = s.visiting.and_then(|v| crate::life::farewell(self, sim as usize, v.plot));
                if let Some((host, social)) = goodbye.filter(|_| self.sims[sim as usize].queue.len() + 1 < MAX_QUEUE) {
                    let World { sims, content, objects, .. } = self;
                    let s = &mut sims[sim as usize];
                    if s.current.as_ref().is_some_and(|a| !a.task.directed) {
                        end_activity(s, content, objects);
                    }
                    s.queue.push_back(Task { kind: TaskKind::Social { target: host as u32, social }, directed: false, routine: false });
                }
                self.enqueue(sim, TaskKind::GoHome)?;
            }
            Command::Phone { sim, action, contact } => {
                self.sim(sim)?;
                crate::phone::check_order(self, sim as usize, action, contact)?;
                self.enqueue(sim, TaskKind::Phone { action, contact, seat: None, answer: crate::phone::Answer::Pending })?;
            }
            Command::Buy {
                household,
                object,
                at,
                colors,
                turn,
                offset,
                size,
                tall,
            } => {
                let def = self.content.object_index(&object);
                let turn = def.map(|d| self.check_turn(d, turn.unwrap_or(0))).transpose()?;
                let rot = at.map_or(0, |a| a[2] as u8);
                let layout = Layout { offset: offset.unwrap_or([0, 0]), extent: size, tall: tall.unwrap_or(0) };
                let layout = def.map(|d| self.check_layout(d, rot, layout)).transpose()?;
                let colors = check_colors(colors)?;
                let id = self.buy_at(household, &object, at, colors, layout.unwrap_or_default())?;
                self.objects[id as usize].turn = turn.unwrap_or(0);
            }
            Command::Sell { household, object } => self.sell(household, object)?,
            Command::MoveObject {
                household,
                object,
                x,
                z,
                rot,
                turn,
                offset,
                size,
                tall,
            } => {
                let old = self.objects.get(object as usize).map(Layout::of);
                let def = self.objects.get(object as usize).map(|o| o.def);
                let turn = match (def, turn) {
                    (Some(d), Some(t)) => Some(self.check_turn(d, t)?),
                    _ => None,
                };
                // What isn't given stays as it was (checked against the new facing).
                let layout = match (def, old) {
                    (Some(d), Some(old)) => {
                        // An offset it keeps that doesn't suit its new facing (along another wall) goes.
                        let kept = if self.check_offset(d, rot, old.offset).is_ok() { old.offset } else { [0, 0] };
                        let wanted = Layout { offset: offset.unwrap_or(kept), extent: size.or(old.extent), tall: tall.unwrap_or(old.tall) };
                        Some(self.check_layout(d, rot, wanted)?)
                    }
                    _ => None,
                };
                let (offset, size, tall) = (layout.map(|l| l.offset), layout.and_then(|l| l.extent), layout.map(|l| l.tall));
                self.move_object_to(household, object, x, z, rot, turn, offset, size, tall)?;
            }
            Command::Recolor { household, object, colors } => self.recolor(household, object, check_colors(colors)?)?,
            Command::Upgrade { household, object } => {
                self.upgrade(household, object)?;
            }
            Command::Build { household, edits } => self.build(household, &edits)?,
            Command::Paint { household, faces } => self.paint(household, &faces)?,
            Command::PaintFloor { household, tiles } => self.paint_floor(household, &tiles)?,
            Command::Undo { household } => self.undo(household)?,
            Command::Redo { household } => self.redo(household)?,
            Command::SetRoof {
                household,
                style,
                color,
            } => self.set_roof(household, style, color)?,
            Command::Repair { household, object } => self.repair(household, object)?,
            Command::BuildBlueprint { household, blueprint } => self.build_blueprint(household, &blueprint)?,
            Command::MoveRoom { household, x, z, dx, dz } => self.move_room(household, x, z, dx, dz)?,
            Command::MoveIn {
                household,
                name,
                sims,
                bonds,
            } => self.move_in(household, name, &sims, &bonds)?,
            Command::Cancel { sim, index } => {
                let World {
                    sims,
                    objects,
                    content,
                    ..
                } = self;
                let s = sims
                    .get_mut(sim as usize)
                    .ok_or_else(|| Error::new(format!("unknown resident {sim}")))?;
                let offset = (s.current.is_some() || s.engaged_with.is_some() || s.heeding.is_some()) as usize;
                if index < offset {
                    // (Called over: they don't go after all.)
                    s.heeding = None;
                    if s.current.is_some() {
                        end_activity(s, content, objects);
                    } else {
                        s.engaged_with = None;
                    }
                } else if index - offset < s.queue.len() {
                    s.queue.remove(index - offset);
                }
            }
        }
        Ok(())
    }

    fn sim(&self, id: u32) -> Result<&Sim, Error> {
        self.sims
            .get(id as usize)
            .filter(|s| s.here())
            .ok_or_else(|| Error::new(format!("unknown resident {id}")))
    }

    fn sim_mut(&mut self, id: u32) -> Result<&mut Sim, Error> {
        self.sims
            .get_mut(id as usize)
            .filter(|s| s.here())
            .ok_or_else(|| Error::new(format!("unknown resident {id}")))
    }

    fn enqueue(&mut self, sim: u32, kind: TaskKind) -> Result<(), Error> {
        let s = self.sim_mut(sim)?;
        if s.queue.len() >= MAX_QUEUE {
            return Err(Error::new("action queue is full"));
        }
        s.queue.push_back(Task {
            kind,
            directed: true,
            routine: false,
        });
        Ok(())
    }
}

/// Keeps a queue within `MAX_QUEUE` by dropping what was chosen for them from the back, but
/// never the first `keep` tasks (just put there), the player's orders, nor a recipe whose
/// ingredients are paid for: for those, it goes over the cap.
fn trim_queue(queue: &mut VecDeque<Task>, keep: usize) {
    while queue.len() > MAX_QUEUE {
        let paid = |t: &Task| matches!(t.kind, TaskKind::Cook { servings, .. } if servings > 0);
        let Some(k) = queue.iter().rposition(|t| !t.directed && !paid(t)).filter(|&k| k >= keep) else { break };
        queue.remove(k);
    }
}

/// Whether a household with `funds` can pay `cost` (free things are always affordable).
pub fn affordable(cost: i64, funds: i64) -> bool {
    cost <= 0 || funds >= cost
}

/// Hour a Sim gets up today: the usual time, or earlier for an early shift.
fn wake_hour(sim: &Sim, ctx: &Ctx) -> f32 {
    let rhythm = &ctx.content.day_rhythm;
    match leave_today(sim, ctx) {
        Some(leave) => (crate::clock::hour(leave) - rhythm.wake_before_work_minutes / 60.0).clamp(0.0, rhythm.wake_hour),
        None => rhythm.wake_hour,
    }
}

/// Whether it is night for this Sim (time to be in bed): their planned sleep, else the usual
/// day rhythm.
fn is_night(sim: &Sim, ctx: &Ctx) -> bool {
    let routines = &ctx.households[sim.household as usize].routines;
    if let Some(night) = crate::planner::planned_night(ctx.content, sim, routines, ctx.day, ctx.minute) {
        return night;
    }
    let rhythm = &ctx.content.day_rhythm;
    if ctx.hour >= rhythm.bed_hour {
        return true;
    }
    if ctx.hour >= rhythm.wake_hour {
        return false;
    }
    rhythm.is_night(ctx.hour, wake_hour(sim, ctx))
}

/// What an object's quality adds to an interaction: a bed's stars make a night only a little
/// shorter (`DayRhythm::sleep_quality_bonus`), anything else's do more.
fn quality_boost(content: &Content, inter: &crate::content::Interaction, quality: u8) -> f32 {
    let rhythm = &content.day_rhythm;
    if inter.tags & rhythm.sleep_tags != 0 {
        rhythm.sleep_quality_factor(quality)
    } else {
        content.object_rules.quality_factor(quality)
    }
}

/// Worn out: what a night's sleep fills is nearly empty (bed, not a nap, whatever the hour).
fn exhausted(sim: &Sim, content: &Content) -> bool {
    let rhythm = &content.day_rhythm;
    rhythm.rest_need.is_some_and(|n| sim.needs[n] < rhythm.exhausted_below)
}

/// Time to get up for work or school (`DayRhythm::wake_before_work_minutes` before leaving).
fn getting_up(sim: &Sim, ctx: &Ctx) -> bool {
    let early = social::minutes_to_ticks(ctx.content.day_rhythm.wake_before_work_minutes);
    leave_today(sim, ctx).is_some_and(|leave| leave <= ctx.tick + early)
}

/// The middle of the night: a rested sleeper stays in bed until a little before they'd get up.
fn before_dawn(sim: &Sim, ctx: &Ctx) -> bool {
    is_night(sim, ctx) && {
        let wake = wake_hour(sim, ctx);
        !(wake - ctx.content.day_rhythm.lie_in_hours..wake).contains(&ctx.hour)
    }
}

/// Target skill levels (what probation and the next promotion need, and skill goals) and how
/// keen they are on each (the job's fully, a goal's by its place on the list).
pub(crate) fn job_goals(sim: &Sim, content: &Content) -> ([f32; MAX_SKILLS], [f32; MAX_SKILLS]) {
    let mut goals = [0.0f32; MAX_SKILLS];
    let mut keen = [0.0f32; MAX_SKILLS];
    if let Some(job) = &sim.job {
        let levels = &content.careers[job.career].levels;
        for level in levels[job.level..].iter().take(2) {
            for &(s, l) in &level.requires {
                goals[s] = goals[s].max(l);
                keen[s] = 1.0;
            }
        }
    }
    crate::planner::skill_targets(content, sim, &mut goals, &mut keen);
    (goals, keen)
}

/// How much an idle Sim wants to practise with `inter` (0 when needs come first).
fn training_interest(sim: &Sim, content: &Content, inter: &crate::content::Interaction) -> f32 {
    let rules = &content.skill_rules;
    let lowest = sim.needs[..content.needs.len()]
        .iter()
        .copied()
        .fold(1.0f32, f32::min);
    if lowest < 0.35 {
        return 0.0;
    }
    let (goals, keen) = job_goals(sim, content);
    let mut best = 0.0f32;
    for (s, &gain) in inter.skill_gain.iter().enumerate() {
        if gain <= 0.0 || sim.skills[s] >= rules.max_level {
            continue;
        }
        let mut want = rules.interest * sim.mods.skill_gain[s];
        if goals[s] > sim.skills[s] {
            want += rules.goal_interest * keen[s];
        }
        best = best.max(want);
    }
    best * lowest / (1.0 + sim.practice_fatigue)
}

/// Adds `minutes` of practice to a skill (a tick's, or more when catching up on a turn, see
/// `lod.rs`) and notes a new level.
pub(crate) fn practise(sim: &mut Sim, content: &Content, skill: usize, per_hour: f32, minutes: f32) {
    let rules = &content.skill_rules;
    let before = sim.skills[skill];
    let gain = rules.gain(
        per_hour * sim.mods.skill_gain[skill],
        minutes / 60.0,
        before,
    );
    let after = (before + gain).min(rules.max_level);
    sim.skills[skill] = after;
    if after.floor() > before.floor() {
        sim.skill_ups |= 1 << skill;
        sim.planner.recheck = true;
    }
}

/// A wall or something they walk through moved while they were on their way: their waypoints
/// may cross it now. Route again from where they stand, and if there's no way there any more,
/// give up (remembering it, if they chose it themselves).
fn reroute_after_change(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let Some(act) = sim.current.as_ref() else { return };
    let Phase::Routing { waypoints, .. } = &act.phase else { return };
    let Some(&target) = waypoints.last() else { return };
    let task = act.task;
    let goal = (target[0].floor() as i32, target[1].floor() as i32);
    if let Some(waypoints) = walk_route(ctx, sim, goal, target) {
        if let Some(act) = sim.current.as_mut() {
            act.phase = Phase::Routing { waypoints, next: 0 };
        }
    } else {
        let directed = task.directed;
        end_activity(sim, ctx.content, objects);
        if !directed {
            let retry = if matches!(task.kind, TaskKind::Social { .. } | TaskKind::Join { .. }) {
                PERSON_UNREACHABLE_TICKS
            } else {
                social::minutes_to_ticks(ctx.content.autonomy.retry_minutes)
            };
            sim.remember_unreachable(task.kind, ctx.tick + retry);
        }
    }
}

fn step_sim(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut crate::items::Items, rng: &mut Rng) {
    let content = ctx.content;
    let minutes = sim.minutes();
    let rhythm = &content.day_rhythm;
    let asleep = sim.asleep(content);
    let night = !asleep && rhythm.sleep_tags != 0 && is_night(sim, ctx);
    let practising = sim.current.as_ref().is_some_and(|a| {
        matches!(a.phase, Phase::Using { .. })
            && match a.task.kind {
                TaskKind::Use { object, interaction } => {
                    content.objects[objects[object as usize].def].interactions[interaction].trains_skills()
                }
                TaskKind::Cook { recipe, origin, .. } => {
                    crate::cooking::origin_interaction(content, objects, origin, recipe).is_some_and(|i| i.trains_skills())
                }
                _ => false,
            }
    });
    let skill_rules = &content.skill_rules;
    sim.practice_fatigue = if practising {
        sim.practice_fatigue + skill_rules.fatigue_per_hour * minutes / 60.0
    } else {
        (sim.practice_fatigue - skill_rules.fatigue_recovery_per_hour * minutes / 60.0)
            .max(0.0)
    };
    for (n, def) in content.needs.iter().enumerate() {
        let rhythm_factor = if sim.away_until.is_some() {
            rhythm.work_decay[n]
        } else if asleep {
            rhythm.asleep_decay[n]
        } else if night {
            rhythm.night_decay[n]
        } else {
            1.0
        };
        if def.room {
            // Surroundings: towards how good the room is (a neutral value away from home).
            let rules = &content.room_rules;
            let target = if sim.away_until.is_some() { rules.away } else { ctx.room_at(sim.pos) };
            let k = (rules.drift * minutes / 60.0).min(1.0);
            sim.needs[n] = (sim.needs[n] + (target - sim.needs[n]) * k).clamp(0.0, 1.0);
            continue;
        }
        let decay =
            def.decay_per_minute * sim.mods.need_decay[n] * rhythm_factor * minutes;
        sim.needs[n] = (sim.needs[n] - decay).clamp(0.0, 1.0);
    }

    // At work: off the map until `life::update` brings them back.
    if sim.away_until.is_some() {
        return;
    }

    // The house changed under them since their last turn: a wall where they were going, or a
    // door opened.
    if sim.nav_seen != ctx.nav_version {
        sim.nav_seen = ctx.nav_version;
        reroute_after_change(sim, ctx, objects);
    }

    // A baby whose needs run low cries for someone (a thought bubble).
    if content.life.baby(sim.age) {
        let lowest = (0..content.needs.len())
            .filter(|&n| !content.needs[n].room)
            .min_by(|&a, &b| sim.needs[a].total_cmp(&sim.needs[b]));
        if let Some(n) = lowest
            && sim.needs[n] < BABY_CRIES_BELOW
            && sim.planner.thought.is_none()
        {
            crate::planner::think(sim, crate::planner::ThoughtKind::Crying, n, ctx.tick);
        }
    }

    // A need ran out with nothing being done about it: an accident (a puddle, asleep on the
    // floor, takeout). Not for babies: that's what carers are for.
    let baby = content.life.baby(sim.age);
    for (a, def) in content.accidents.iter().enumerate().filter(|_| !baby) {
        if sim.needs[def.need] > 0.0 || filling(sim, content, objects, items, def.need) {
            sim.empty_since[a] = 0;
            continue;
        }
        if sim.empty_since[a] == 0 {
            sim.empty_since[a] = ctx.tick.max(1);
        }
        // When home has something for it (it's busy, or they got caught up in something), only
        // if the content says so; without (no toilet, no bed), after a short while.
        let grace = if home_fills(sim, ctx, objects, def.need) { def.crowded_grace_ticks } else { Some(def.grace_ticks) };
        let Some(grace) = grace else { continue };
        if ctx.tick < sim.accident_at[a] || ctx.tick < sim.empty_since[a] + grace {
            continue;
        }
        sim.empty_since[a] = 0;
        accident(sim, ctx, objects, a);
        break;
    }

    // Someone is talking to this Sim: stay put unless the player gives an order.
    if sim.engaged_with.is_some() {
        if sim.queue.front().is_some_and(|t| t.directed) {
            sim.engaged_with = None;
        } else {
            return;
        }
    }
    // Someone called them: on their way to meet them, or waiting there (unless the player gives
    // an order).
    if sim.heeding.is_some() {
        if sim.queue.front().is_some_and(|t| t.directed) {
            sim.heeding = None;
        } else if sim.current.is_none() {
            return;
        }
    }

    // Player commands interrupt whatever the Sim chose on its own.
    let autonomous_running = sim.current.as_ref().is_some_and(|a| !a.task.directed);
    if autonomous_running && sim.queue.front().is_some_and(|t| t.directed) {
        end_activity(sim, content, objects);
    }

    // Off to work or school soon: the toilet first, out of habit.
    if let Some(task) = ready_to_leave(sim, ctx, objects) {
        end_activity(sim, content, objects);
        sim.follow_up = Some(task);
    }

    // Ticks what they're doing moves on by (fewer than `steps` if it starts partway through).
    let mut ticks = sim.steps;
    if sim.current.is_none() {
        // A routine's next step (washing hands after the toilet) before anything else they'd
        // do, unless the player asks for something.
        let follow_up = sim.follow_up.take().filter(|_| !sim.queue.front().is_some_and(|t| t.directed));
        if let Some(task) = follow_up {
            start_task(sim, task, ctx, objects, items);
        } else if let Some(task) = sim.queue.pop_front() {
            start_task(sim, task, ctx, objects, items);
        } else {
            sim.idle_ticks = sim.idle_ticks.saturating_add(sim.steps);
            // Babies don't choose anything: they lie in a crib and are cared for.
            if content.life.baby(sim.age) {
                if let Some(task) = crib_for(sim, ctx, objects) {
                    start_task(sim, task, ctx, objects, items);
                }
                return;
            }
            let free_will = ctx.autonomy && ctx.households[sim.household as usize].free_will;
            if free_will && sim.idle_ticks >= AUTONOMY_DELAY_TICKS {
                let since = sim.idle_ticks - AUTONOMY_DELAY_TICKS;
                sim.idle_ticks = 0;
                if let Some(task) = pick_autonomous(sim, ctx, objects, items, rng) {
                    start_task(sim, task, ctx, objects, items);
                    if sim.current.is_none() && sim.follow_up.is_none() {
                        // People move: don't hold a grudge against where they were for long.
                        let retry = if matches!(task.kind, TaskKind::Social { .. } | TaskKind::Join { .. }) {
                            PERSON_UNREACHABLE_TICKS
                        } else {
                            social::minutes_to_ticks(content.autonomy.retry_minutes)
                        };
                        sim.remember_unreachable(task.kind, ctx.tick + retry);
                    }
                    // Catching up on a turn (`lod.rs`): they were idle until it was time to
                    // choose, so what they chose has only had the ticks since.
                    ticks = ticks.min(since + 1);
                }
            }
        }
    }

    progress(sim, ctx, objects, items, rng, ticks);
}

/// What a Sim may use on a plot: everything at home or in public places; as a guest,
/// anything that isn't private (bed, bathroom...); nothing on other people's lots.
#[derive(PartialEq)]
pub(crate) enum Access {
    Full,
    Guest,
    None,
}

pub(crate) fn access(ctx: &Ctx, sim: &Sim, plot: Option<u32>) -> Access {
    access_in(ctx.households, ctx.plots, sim, plot)
}

/// Whether a guest may use `inter` at the host's: nothing private (sleeping, washing; the
/// toilet is fine), no cooking and no caring for the host's baby.
pub(crate) fn guest_may(content: &Content, inter: &crate::content::Interaction) -> bool {
    let rules = &content.social_rules;
    inter.tags & rules.busy_tags & !rules.guest_tags == 0 && inter.recipe.is_none() && !inter.cares()
}

/// Why someone may not take an interaction at their age, wherever it is (see `unfit`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unfit {
    /// Every stage it is for (`Interaction::stages`) is older than theirs: wine for a child.
    TooYoung,
    /// It is for younger ones only: a toddler's spring rider for a grown-up.
    TooBig,
    /// Homework is a pupil's.
    NoHomework,
    /// Looking after a baby is a grown-up's.
    TooYoungToCare,
}

/// Why someone of `age` may not take `inter` (None: they may): its life stages, homework only
/// at a school stage, and caring for a baby only grown up. Free will and the player's orders
/// keep it alike.
pub(crate) fn unfit(content: &Content, age: f32, inter: &crate::content::Interaction) -> Option<Unfit> {
    let life = &content.life;
    let stage = life.stage(age);
    if !inter.for_stage(stage) {
        return Some(if crate::content::all_older(inter.stages, stage) { Unfit::TooYoung } else { Unfit::TooBig });
    }
    if inter.homework > 0.0 && !life.stage_at(age).is_some_and(|s| s.school) {
        return Some(Unfit::NoHomework);
    }
    (inter.cares() && !life.adult(age)).then_some(Unfit::TooYoungToCare)
}

/// Whether `sim` (of `age`) could only take the upper bunk of `obj` (the lower one someone
/// else's) and may not (`ObjectDef::upper_stages`): an elder, say.
pub(crate) fn upper_barred(content: &Content, obj: &ObjectInstance, sim: u32, age: f32) -> bool {
    let def = &content.objects[obj.def];
    def.bunk && !stage_in(def.upper_stages, content.life.stage(age)) && obj.users[0].is_some_and(|u| u != sim)
}

/// What the player may have `sim` use on `plot`: as `access`, but their own home and public
/// places stay theirs to use while they're a guest somewhere.
pub(crate) fn order_access(households: &[Household], plots: &[Plot], sim: &Sim, plot: Option<u32>) -> Access {
    let home = households[sim.household as usize].plot;
    match plot {
        None => Access::Full,
        Some(p) if plots[p as usize].public || home == Some(p) => Access::Full,
        _ => access_in(households, plots, sim, plot),
    }
}

/// `access` outside a step.
pub(crate) fn access_in(households: &[Household], plots: &[Plot], sim: &Sim, plot: Option<u32>) -> Access {
    let home = households[sim.household as usize].plot;
    // A guest stays at the host's place until it's time to go home.
    if let Some(visit) = sim.visiting {
        return if plot == Some(visit.plot) {
            Access::Guest
        } else {
            Access::None
        };
    }
    match plot {
        None => Access::Full,
        Some(p) if plots[p as usize].public || home == Some(p) => Access::Full,
        Some(p) if sim.visiting.is_some_and(|v| v.plot == p) => Access::Guest,
        Some(_) => Access::None,
    }
}

/// The first thing `inter` requires that no working object on `object`'s plot provides, if any
/// (content `requires` / `provides`): a table's coffee needs a coffee machine. With `index`
/// (current during a tick) only that plot's objects are looked at.
fn missing_requirement<'a>(
    content: &'a Content,
    object_plot: &[Option<u32>],
    index: Option<&ObjectIndex>,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> Option<&'a str> {
    if inter.requires == 0 {
        return None;
    }
    let plot = object_plot.get(object as usize).copied().flatten();
    let provides = |o: &ObjectInstance| if o.broken() { 0 } else { content.objects[o.def].provides };
    let provided = match (plot, index) {
        (Some(p), Some(index)) => index.on(p).iter().fold(0, |m, &o| m | provides(&objects[o as usize])),
        _ => objects
            .iter()
            .filter(|o| object_plot.get(o.id as usize).copied().flatten() == plot)
            .fold(0, |m, o| m | provides(o)),
    };
    let missing = inter.requires & !provided;
    if missing == 0 {
        return None;
    }
    (0..content.tags.len()).find(|&i| missing & (1 << i) != 0).map(|i| content.tags[i].as_str())
}

fn requirements_met(ctx: &Ctx, objects: &[ObjectInstance], object: u32, inter: &crate::content::Interaction) -> bool {
    missing_requirement(ctx.content, ctx.object_plot, Some(ctx.index), objects, object, inter).is_none()
}

fn pick_autonomous(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    items: &crate::items::Items,
    rng: &mut Rng,
) -> Option<Task> {
    let content = ctx.content;
    let needs = &sim.needs[..content.needs.len()];
    // How hard each need pulls (more below the level they like to keep it at).
    let felt = ai::felt(needs, &content.needs, &content.autonomy);
    let emotion = sim.emotion(content).map(|e| &content.emotions[e].mods);
    let schedule = content.schedule_at(ctx.hour);
    // Traits, the current emotion and the time of day all bias what feels right.
    let feel = |tags: TagMask| {
        sim.mods.preference(tags)
            * emotion.map_or(1.0, |m| m.preference(tags))
            * schedule.preference(tags)
    };
    let me = sim.id as usize;
    let here = ctx.briefs[me].plot;
    let mut candidates: Vec<(f32, Choice)> = Vec::new();
    let routines = &ctx.households[sim.household as usize].routines;
    let night = crate::planner::planned_night(content, sim, routines, ctx.day, ctx.minute);

    let object_rules = &content.object_rules;
    let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
    let environment = content.needs.iter().position(|n| n.room).map_or(1.0, |n| sim.needs[n]);
    // A need that wakes sleepers (bladder, hunger) is urgent: deal with it first, as a sleep
    // started now would end at once (see `use_object`), over and over.
    let rhythm = &content.day_rhythm;
    let woken = (0..content.needs.len()).any(|n| needs[n] < content.autonomy.urgent_below && rhythm.wake_for & (1 << n) != 0);
    // (Not a nap either: the sofa won't help.) Worn out, it's bed whatever the hour, not a nap;
    // and no nap again too soon after the last.
    let rest = rhythm.sleep_tags | rhythm.nap_tags;
    let exhausted = exhausted(sim, content);
    let napped = ctx.tick < sim.nap_until;
    // Nor back to bed rested, unless it's the middle of the night (it would end at once), nor
    // once the alarm's gone off for work or school.
    let rested = (!before_dawn(sim, ctx) && rhythm.rest_need.is_some_and(|n| needs[n] >= 0.95)) || getting_up(sim, ctx);
    // Only what they may use: at home and in public places, or where they're a guest.
    let usable = match sim.visiting {
        Some(visit) => ctx.index.on(visit.plot),
        None => ctx.index.usable_from(ctx.households[sim.household as usize].plot),
    };
    // Cooking: the stations they have (recipes need theirs), and whether food is ready.
    let stations = crate::cooking::station_mask(sim, ctx, objects);
    let fresh = items.list.iter().any(|it| it.household == sim.household && it.edible());
    // Needs about to run out (see `use_object`'s `drained`); the essentials among them that
    // home has something for; and all needs home has something for (what they look ahead to).
    let low = content.autonomy.urgent_below;
    let short: u32 = (0..content.needs.len()).filter(|&n| !content.needs[n].room && needs[n] < low).fold(0, |m, n| m | 1 << n);
    let walk_per_minute = WALK_SPEED * sim.mods.walk_speed / MINUTES_PER_TICK;
    let urgent = urgent_needs(sim, ctx, objects);
    let fillable = home_needs(sim, ctx, objects, (0..content.needs.len()).filter(|&n| !content.needs[n].room).fold(0, |m, n| m | 1 << n), false);
    // How far away the nearest thing for each need is, and how long until they leave for work
    // or school (minutes).
    let reach = reach(sim, ctx, objects, usable, walk_per_minute);
    let until_leave = leave_today(sim, ctx).map(|t| (t - ctx.tick) as f32 * MINUTES_PER_TICK);
    // The last grown-up at home with a baby doesn't go out (`life::minding_baby`), unless for
    // an essential need about to run out that home has nothing for.
    let minding = crate::life::minding_baby(sim, ctx);
    let own = ctx.households[sim.household as usize].plot;
    let stranded = short & !fillable & content.autonomy.urgent;
    for obj in usable
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| o.has_free_slot(content.objects[o.def].slots))
    {
        let plot = ctx.object_plot[obj.id as usize];
        let allowed = access(ctx, sim, plot);
        // (Off the home plot: a public place or the street.)
        let outing = || plot != own && !content.objects[obj.def].interactions.iter().any(|i| i.fills() & stranded != 0);
        if allowed == Access::None || (minding && outing()) {
            continue;
        }
        if obj.broken() {
            // Broken things at home get repaired (handier residents are keener).
            if allowed == Access::Full {
                let rules = &object_rules.repair;
                let (fx, fz) = ctx.index.links.approach_of(obj.id).unwrap_or_else(|| obj.front_tile(content));
                let distance = ctx.nav.stairs.distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos);
                let level = rules.skill.map_or(0.0, |s| sim.skills[s]);
                let (plan, floor) = crate::planner::tags_factor(content, sim, rules.tags);
                // More urgent the more they need what it gives (a broken toilet, bursting).
                let urgency = content.objects[obj.def]
                    .interactions
                    .iter()
                    .flat_map(|it| (0..content.needs.len()).filter(move |&n| it.total_gain[n] > 0.0))
                    .map(|n| 1.0 - needs[n])
                    .fold(0.0, f32::max);
                let s = (rules.interest * (1.0 + level * 0.1) * (1.0 + 2.0 * urgency) / (1.0 + distance * 0.08)).max(floor)
                    * feel(rules.tags)
                    * plan;
                if s > MIN_AUTONOMY_SCORE {
                    candidates.push((s, Choice::Repair(obj.id)));
                }
            }
            continue;
        }
        let (fx, fz) = ctx.index.links.approach_of(obj.id).unwrap_or_else(|| obj.front_tile(content));
        let distance = ctx.nav.stairs.distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos);
        let at_table = ctx.index.links.table_of(obj.id).is_some();
        // Nicer rooms draw residents, the more so the less they like their surroundings.
        let place = 1.0
            + content.room_rules.preference
                * (ctx.room_at([fx as f32 + 0.5, fz as f32 + 0.5]) - 0.5)
                * 2.0
                * (1.0 - environment);
        for (i, inter) in content.objects[obj.def].interactions.iter().enumerate() {
            // A baby's place is only ever a baby's; caring for one is for grown-ups at home with
            // a baby in it, the more the baby needs it. Nothing their age isn't for (`unfit`).
            let mut care = 0.0;
            // Dining at a table only with a table to dine at.
            // (Nor one that takes the whole thing while someone else is on it.)
            if inter.baby
                || unfit(content, sim.age, inter).is_some()
                || upper_barred(content, obj, sim.id, sim.age)
                || (inter.needs_table && !at_table)
                || !requirements_met(ctx, objects, obj.id, inter)
                || !obj.allows(inter, sim.id)
                || !obj.can_use_for(sim.id, &content.objects[obj.def], i)
            {
                continue;
            }
            // Costing a need about to run out, or planned with one it doesn't fill: it would stop
            // at once (`use_object`), and be chosen again, over and over.
            let costs = (0..content.needs.len()).any(|n| short & (1 << n) != 0 && inter.total_gain[n] < 0.0);
            let leaves = (0..content.needs.len()).any(|n| short & (1 << n) != 0 && inter.total_gain[n] <= 0.0);
            if costs || (leaves && crate::planner::planned_now(content, sim, inter)) {
                continue;
            }
            if inter.homework > 0.0 {
                care += inter.homework / 10.0 * (100.0 - sim.grade) / 100.0 * 1.5 / (1.0 + distance * 0.08);
            }
            if inter.cares() {
                let baby = obj.users().find(|&u| ctx.briefs.get(u as usize).is_some_and(|b| b.baby));
                let Some(baby) = baby.filter(|_| allowed == Access::Full) else { continue };
                let b = &ctx.briefs[baby as usize];
                let family = if ctx.rels.kin(me, baby as usize) != crate::social::Kin::None { 1.5 } else { 1.0 };
                care = ai::score_gains(&b.needs[..content.needs.len()], &inter.care, distance) * family;
                if care <= 0.0 {
                    continue;
                }
            }
            if !inter.autonomous
                || (woken && inter.tags & rest != 0)
                || (rested && inter.tags & rhythm.sleep_tags != 0)
                || (napped && !exhausted && inter.tags & rhythm.nap_tags != 0)
                || (allowed == Access::Guest && inter.tags & content.social_rules.busy_tags & !content.social_rules.guest_tags != 0)
                || !affordable(inter.cost, funds)
            {
                continue;
            }
            let sleep = inter.tags & rhythm.sleep_tags != 0;
            let fills = inter.fills();
            let there = distance / walk_per_minute;
            let minutes = inter.minutes + there;
            // A chosen use that would stop at once for an urgent need (`use_object`), then get
            // chosen again: don't choose it at all.
            if !sleep && breaks_off(sim, content, urgent, fills, minutes, inter, low) && !crate::planner::planned_now(content, sim, inter) {
                continue;
            }
            // Better objects and more skilled Sims get more out of it, and know it.
            let boost = quality_boost(content, inter, obj.quality) * inter.skill_factor(&content.skill_rules, &sim.skills);
            let gains = ai::score_felt(needs, &felt, &inter.total_gain.map(|g| g * boost), distance);
            let mut s = match inter.recipe {
                // A recipe: only at home with every station it needs. Long ones tempt less, a
                // hungry family more, and with food ready hardly at all.
                Some(r) => {
                    let r = &content.recipes[r];
                    let at_home = ctx.object_plot[obj.id as usize].is_some_and(|p| Some(p) == ctx.households[sim.household as usize].plot);
                    // (Nobody starts on a long recipe that something would run out before it's
                    // done for, hunger too, as it's eaten only at the end; nor shortly before
                    // leaving for work.)
                    let minutes = r.minutes_with(stations);
                    let urgent = runs_out(sim, content, fillable & content.autonomy.urgent, minutes, false, &[0.0; MAX_NEEDS], &reach, low)
                        || until_leave.is_some_and(|m| minutes > m);
                    if urgent || !at_home || allowed != Access::Full || !r.possible(stations) {
                        continue;
                    }
                    let hungry = crate::cooking::hungry_housemates(sim, ctx, r) as f32;
                    let s = gains / (1.0 + r.minutes_with(stations) / 60.0) + content.cooking.feed_family * hungry;
                    if fresh { s * 0.2 } else { s }
                }
                None => gains,
            } + care;
            if inter.trains_skills() && allowed == Access::Full {
                s += training_interest(sim, content, inter) * boost / (1.0 + distance * 0.08);
            }
            // The plan: a block's activity is the favourite (even with full needs); goals.
            let (plan, floor) = crate::planner::object_factor(content, sim, inter, night);
            let mut s = s.max(floor) * feel(inter.tags) * plan * place;
            if exhausted && inter.tags & rhythm.sleep_tags != 0 {
                // (The schedule's daytime damping on sleep doesn't apply.)
                s /= schedule.preference(inter.tags & rhythm.sleep_tags).clamp(1e-3, 1.0);
            } else if exhausted && inter.tags & rhythm.nap_tags != 0 {
                s *= 0.25;
            }
            // Not something long another need would run out before the end of, counting the way
            // there and from there to what fills it (only what would wake them counts for a
            // night's sleep): that first. Nor what they couldn't finish before leaving for work.
            let watch = fillable & !fills & !urgent & if sleep { rhythm.wake_for } else { u32::MAX };
            if runs_out(sim, content, watch, minutes, sleep, &inter.gain_per_minute, &reach.map(|r| r + there), low)
                || (!sleep && until_leave.is_some_and(|m| minutes > m))
            {
                s *= content.autonomy.look_ahead;
            }
            if s > MIN_AUTONOMY_SCORE {
                candidates.push((s, Choice::Object(obj.id, i)));
            }
        }
    }

    // About to drop (or burst): no time to talk, as a chat would break off at once and leave the
    // other one's activity dropped for nothing (`available_for_social` asks the same of them).
    let pressed = sim.pressed(content);
    for (j, other) in ctx.briefs.iter().enumerate() {
        if j == me || !other.available || pressed {
            continue;
        }
        // Chat with people within earshot (they call out to them); nobody crosses the house or
        // the lot for a chat, and crossing town is a visit.
        let distance = ctx.nav.stairs.distance(other.pos, sim.pos);
        let heard = conversation::within_earshot(&ctx.nav, sim.pos, other.pos);
        // Nor out on the street, minding the baby.
        if !heard || (minding && other.plot != own) {
            continue;
        }
        // Not over the fence into someone else's garden to chat (a guest is welcome at the host's).
        if let Some(b) = other.plot
            && here != Some(b)
            && access(ctx, sim, Some(b)) == Access::None
        {
            continue;
        }
        let rel = ctx.rels.get(me, j);
        let chemistry = ctx.rels.chemistry(me, j);
        let attracted = other
            .gender_ix
            .is_none_or(|g| sim.attraction & (1 << g) != 0);
        let single = !sim.planner.goals.is_empty() && ctx.rels.partner_of(j).is_none();
        let grown_ups = ctx.briefs[me].adult && other.adult;
        // A guest keeps the hosts company, even with nothing in particular to talk about.
        let company = match sim.visiting {
            Some(v) if here == Some(v.plot) && ctx.households[other.household as usize].plot == Some(v.plot) => content.visits.company,
            _ => 0.0,
        };
        // Two singles who like the look of each other: a spark before there's any romance.
        let spark = if grown_ups && attracted && rel.met && !ctx.briefs[me].partnered && !other.partnered && rel.kin.is_none() {
            content.social_rules.single_spark
        } else {
            0.0
        };
        let (my_stage, their_stage) = (content.life.stage(sim.age), content.life.stage(other.age));
        for (k, s) in content.socials.iter().enumerate() {
            // (Nor with someone their age isn't for: no fights with a child.)
            if !s.autonomous || !s.requires.allows(rel) || !stage_in(s.stages, my_stage) || !stage_in(s.target_stages, their_stage) {
                continue;
            }
            // Romance only between grown-ups who aren't family.
            if s.tags & content.social_rules.romantic_tags != 0 && (!attracted || !grown_ups || !rel.kin.is_none()) {
                continue;
            }
            // Trying for a baby only where one could come of it.
            if s.success.effect == crate::social::Effect::Conceive && !could_conceive(ctx, me, j) {
                continue;
            }
            let romantic = s.tags & content.social_rules.romantic_tags != 0;
            let friendly = matches!(s.prefer, Prefer::Any | Prefer::Liked);
            let gains = ai::score_felt(needs, &felt, &s.actor_gain, distance);
            let gains = if s.prefer == Prefer::Liked && !romantic { gains.max(company) } else { gains };
            let score = gains
                * s.autonomy_weight
                * prefer_factor(s.prefer, rel, chemistry, spark)
                * feel(s.tags)
                * crate::planner::social_factor(content, sim, romantic, friendly, rel.friendship, single);
            if score > MIN_AUTONOMY_SCORE {
                candidates.push((score, Choice::Social(j as u32, k)));
            }
        }
    }

    // Joining in where people are talking (nearby, a friendly social, room for one more): for
    // the social it brings, more so with people they like.
    for (l, leader) in ctx.briefs.iter().enumerate() {
        let Some((_, social)) = leader.conversing.filter(|c| c.0 as usize == l && l != me) else {
            continue;
        };
        let s = &content.socials[social];
        if !s.joinable || !leader.here || leader.away || pressed {
            continue;
        }
        // (Only where they can hear them talking.)
        let distance = ctx.nav.stairs.distance(leader.pos, sim.pos);
        let heard = conversation::within_earshot(&ctx.nav, sim.pos, leader.pos);
        if !heard || (minding && leader.plot != own) {
            continue;
        }
        if let Some(b) = leader.plot
            && here != Some(b)
            && access(ctx, sim, Some(b)) == Access::None
        {
            continue;
        }
        let members: Vec<usize> = (0..ctx.briefs.len()).filter(|&m| ctx.briefs[m].conversing.is_some_and(|c| c.0 as usize == l)).collect();
        if members.len() >= conversation::MAX_GROUP || members.contains(&me) {
            continue;
        }
        let count = members.len() as f32;
        let liking = members.iter().map(|&m| prefer_factor(Prefer::Liked, ctx.rels.get(me, m), 0.0, 0.0)).sum::<f32>() / count;
        let friendship = members.iter().map(|&m| ctx.rels.get(me, m).friendship).sum::<f32>() / count;
        // A guest joins in where the hosts are talking (keeping them company).
        let company = match sim.visiting {
            Some(v) if here == Some(v.plot) && members.iter().any(|&m| ctx.households[ctx.briefs[m].household as usize].plot == Some(v.plot)) => content.visits.company,
            _ => 0.0,
        };
        let score = ai::score_felt(needs, &felt, &s.target_gain, distance).max(company)
            * s.autonomy_weight
            * liking
            * feel(s.tags)
            * crate::planner::social_factor(content, sim, false, true, friendship, false);
        if score > MIN_AUTONOMY_SCORE {
            candidates.push((score, Choice::Join(l as u32)));
        }
    }

    // Tidying up: the dirtiest spot at home, if it's dirty enough. The worse their surroundings
    // feel, the keener they are; traits and planned chores do the rest.
    let home = ctx.households[sim.household as usize].plot;
    if let Some(plot) = home.filter(|&p| here == Some(p)) {
        let rules = &content.room_rules.clean;
        let p = &ctx.plots[plot as usize];
        let lot = ctx.nav.lot;
        let mut best: Option<(f32, i32, i32)> = None;
        // (Not where they couldn't get to a moment ago, nor where someone else is tidying up:
        // the next dirtiest spot instead.)
        let mut out_of_reach: Vec<(i32, i32)> = sim.unreachable_now(ctx.tick).filter_map(|k| if let TaskKind::Clean { x, z } = k { Some((x, z)) } else { None }).collect();
        out_of_reach.extend(ctx.briefs.iter().enumerate().filter(|&(j, _)| j != me).filter_map(|(_, b)| b.cleaning));
        for z in p.z..p.z + p.d {
            for x in p.x..p.x + p.w {
                if !lot.in_bounds(x, z) || out_of_reach.contains(&(x, z)) {
                    continue;
                }
                let i = lot.tile_index(x, z);
                let d = ctx.dirt.get(i).copied().unwrap_or(0.0);
                if d > rules.threshold && !ctx.nav.blocked[i] && !lot.diag_blocks(x, z) && best.is_none_or(|b| d > b.0) {
                    best = Some((d, x, z));
                }
            }
        }
        if let Some((d, x, z)) = best {
            let distance = ctx.nav.stairs.distance([x as f32 + 0.5, z as f32 + 0.5], sim.pos);
            let (plan, floor) = crate::planner::tags_factor(content, sim, rules.tags);
            let s = (rules.interest * d * (1.5 - environment) / (1.0 + distance * 0.08)).max(floor)
                * feel(rules.tags)
                * plan;
            if s > MIN_AUTONOMY_SCORE {
                candidates.push((s, Choice::Clean(x, z)));
            }
        }
    }

    // Food at home: a serving of what's been cooked (leftovers, or the dinner they were called
    // to), and washing up where there's a sink or dishwasher.
    if home.is_some_and(|p| here == Some(p)) {
        let rules = &content.cooking;
        let invited = if sim.invited_until > ctx.tick { rules.invited_boost } else { 1.0 };
        let mut best: Option<(f32, u32)> = None;
        for it in items.list.iter().filter(|it| it.household == sim.household && it.edible()) {
            let Some(r) = it.recipe else { continue };
            let distance = ctx.nav.stairs.distance(it.at.centre(), sim.pos);
            let gains = content.recipes[r].effects.map(|g| g * it.quality);
            let s = ai::score_felt(needs, &felt, &gains, distance) * rules.leftover_preference * invited;
            if best.is_none_or(|b| s > b.0) {
                best = Some((s, it.id));
            }
        }
        if let Some((s, item)) = best {
            let (plan, floor) = crate::planner::tags_factor(content, sim, rules.eat_tags);
            let s = s.max(floor) * feel(rules.eat_tags) * plan;
            if s > MIN_AUTONOMY_SCORE {
                candidates.push((s, Choice::Eat(item)));
            }
        }
        // (With no sink, the dirty things go in the bin.)
        if stations & (rules.dishes | rules.bin) != 0 {
            let mut best: Option<(f32, u32)> = None;
            let mut count = 0;
            for it in items.list.iter().filter(|it| it.household == sim.household && it.to_clear(content)) {
                count += 1;
                let distance = ctx.nav.stairs.distance(it.at.centre(), sim.pos);
                if best.is_none_or(|b| distance < b.0) {
                    best = Some((distance, it.id));
                }
            }
            if let Some((distance, item)) = best {
                let (plan, floor) = crate::planner::tags_factor(content, sim, rules.dishes_tags);
                // The bigger the pile, the keener (a plate or two can wait).
                let pile = (count as f32 / f32::from(rules.carry_dishes)).clamp(0.4, 3.0);
                let s = (rules.dishes_interest * pile * (1.5 - environment) / (1.0 + distance * 0.08)).max(floor)
                    * feel(rules.dishes_tags)
                    * plan;
                if s > MIN_AUTONOMY_SCORE {
                    candidates.push((s, Choice::Dishes(item)));
                }
            }
        }
    }

    // The phone: texting, calling or inviting people who aren't here, scrolling (`phone.rs`).
    // (Nor a call or a text, pressed.)
    for (s, action, contact) in crate::phone::candidates(sim, ctx, &felt, &feel) {
        if s > MIN_AUTONOMY_SCORE && !(pressed && contact.is_some()) {
            candidates.push((s, Choice::Phone(action, contact)));
        }
    }

    // Residents visit friends who are home.
    let household = &ctx.households[sim.household as usize];
    let visits = &content.visits;
    let plans_soon = || {
        crate::planner::plans_soon(content, sim, routines, ctx.day, ctx.minute, visits.hours * 60.0)
    };
    if sim.visiting.is_none()
        && here.is_some()
        && here == household.plot
        && !minding
        && (visits.earliest_hour..visits.latest_hour).contains(&ctx.hour)
    {
        let soon = plans_soon();
        for other in ctx.households.iter().filter(|h| h.id != household.id) {
            let Some(plot) = other.plot else { continue };
            let friendship = ctx
                .briefs
                .iter()
                .enumerate()
                .filter(|(_, b)| b.household == other.id && b.plot == Some(plot) && !b.away)
                .map(|(j, _)| ctx.rels.get(me, j).friendship)
                .fold(f32::NEG_INFINITY, f32::max);
            if friendship < visits.min_friendship {
                continue;
            }
            let door = ctx.plots[plot as usize].arrival_point();
            let distance = (door[0] - sim.pos[0]).hypot(door[1] - sim.pos[1]);
            // Travel is less of a deterrent than for a chat in the same room.
            let score = ai::score_felt(needs, &felt, &visits.gains, distance * 0.25)
                * (0.4 + friendship / 100.0)
                * feel(visits.tags)
                * crate::planner::visit_factor(content, sim, soon);
            if score > MIN_AUTONOMY_SCORE {
                candidates.push((score, Choice::Visit(plot)));
            }
        }
    }

    // Not what they couldn't get to a moment ago (and would try again and again, each time
    // looking for a way there).
    for kind in sim.unreachable_now(ctx.tick) {
        candidates.retain(|&(_, choice)| match (choice, kind) {
            (Choice::Object(o, i), TaskKind::Use { object, interaction }) => (o, i) != (object, interaction),
            (Choice::Eat(it), TaskKind::Eat { item, .. }) => Some(it) != item,
            (Choice::Clean(x, z), TaskKind::Clean { x: cx, z: cz }) => (x, z) != (cx, cz),
            (Choice::Repair(o), TaskKind::Repair { object }) => o != object,
            (Choice::Dishes(it), TaskKind::Dishes { item, .. }) => it != item,
            (Choice::Visit(p), TaskKind::Visit { plot }) => p != plot,
            (Choice::Social(t, s), TaskKind::Social { target, social }) => (t, s) != (target, social),
            (Choice::Join(l), TaskKind::Join { leader }) => l != leader,
            _ => true,
        });
    }

    // Needs about to run out come first: whatever doesn't fill one is put off.
    if urgent != 0 {
        for (s, choice) in &mut candidates {
            let gains = match *choice {
                Choice::Object(o, i) => content.objects[objects[o as usize].def].interactions[i].total_gain,
                Choice::Eat(item) => items.get(item).and_then(|it| it.recipe).map_or([0.0; MAX_NEEDS], |r| content.recipes[r].effects),
                _ => [0.0; MAX_NEEDS],
            };
            if !(0..content.needs.len()).any(|n| urgent & (1 << n) != 0 && gains[n] > 0.0) {
                *s *= content.autonomy.look_ahead;
            }
        }
    }

    ai::choose(&mut candidates, rng.next_f32(), 3).map(|choice| Task {
        kind: match choice {
            Choice::Object(object, interaction) => TaskKind::Use {
                object,
                interaction,
            },
            Choice::Social(target, social) => TaskKind::Social { target, social },
            Choice::Join(leader) => TaskKind::Join { leader },
            Choice::Visit(plot) => TaskKind::Visit { plot },
            Choice::Clean(x, z) => TaskKind::Clean { x, z },
            Choice::Repair(object) => TaskKind::Repair { object },
            Choice::Eat(item) => TaskKind::Eat { item: Some(item), seat: None, plate: None, stage: 0 },
            Choice::Dishes(item) => TaskKind::Dishes { item, station: None, stage: 0, scraps: 0 },
            Choice::Phone(action, contact) => TaskKind::Phone { action, contact, seat: None, answer: crate::phone::Answer::Pending },
        },
        directed: false,
        routine: false,
    })
}

/// How much a relationship invites a social of the given kind.
pub(crate) fn prefer_factor(prefer: Prefer, rel: &social::Relationship, chemistry: f32, spark: f32) -> f32 {
    match prefer {
        Prefer::Any => 1.0,
        Prefer::Liked => (0.5 + rel.friendship / 100.0).clamp(0.1, 1.5),
        Prefer::Disliked => (0.4 - rel.friendship / 100.0).clamp(0.0, 1.4),
        Prefer::Romance => (rel.romance / 50.0 + chemistry * 0.4 + spark).clamp(0.0, 1.6),
        Prefer::Unloved => {
            if rel.partners && rel.romance < 25.0 {
                (25.0 - rel.romance) / 25.0
            } else {
                0.0
            }
        }
    }
}

fn start_task(sim: &mut Sim, task: Task, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut crate::items::Items) {
    let content = ctx.content;
    let nav = &ctx.nav;
    // Nothing their age isn't for, however it was chosen (`unfit`; an order was refused already).
    if let TaskKind::Use { object, interaction } = task.kind
        && let Some(o) = objects.get(object as usize)
        && content.objects[o.def].interactions.get(interaction).is_some_and(|i| unfit(content, sim.age, i).is_some())
    {
        return;
    }
    // A recipe isn't made where it was picked: the cook goes round its stations, starting with
    // the first (see `cooking.rs`).
    if let TaskKind::Use { object, interaction } = task.kind
        && let Some(recipe) = objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .and_then(|i| i.recipe)
    {
        let kind = TaskKind::Cook { recipe, step: 0, station: None, origin: Some(object), servings: 0, skipped: 0, minutes: 0 };
        return crate::cooking::start(sim, Task { kind, ..task }, ctx, objects, items);
    }
    let mut reserved_seat = None;
    let (target, goal, tags) = match task.kind {
        TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. } => {
            return crate::cooking::start(sim, task, ctx, objects, items);
        }
        TaskKind::Phone { .. } => return crate::phone::start(sim, task, ctx, objects),
        TaskKind::Use {
            object,
            interaction,
        } => {
            let Some(obj) = objects.get(object as usize) else {
                return;
            };
            // (Not one it doesn't have: an id from before something was sold or moved.)
            let Some(inter) = content.objects[obj.def].interactions.get(interaction) else {
                return;
            };
            // An order to close it on someone who got in (or set off for it) after it was given:
            // the player is told.
            if task.directed && obj.closes_on_someone(inter, sim.id) {
                sim.news.get_or_insert(News::NotClosed(object));
                return;
            }
            if !obj.can_use_for(sim.id, &content.objects[obj.def], interaction) || obj.broken() || upper_barred(content, obj, sim.id, sim.age) {
                return;
            }
            // Never other people's things; what the player asks a guest to do, only what guests
            // may (washing hands at the host's sink after the toilet is fine).
            let plot = ctx.object_plot.get(object as usize).copied().flatten();
            let allowed = if task.directed { order_access(ctx.households, ctx.plots, sim, plot) } else { access(ctx, sim, plot) };
            if allowed == Access::None || (allowed == Access::Guest && task.directed && !guest_may(content, inter)) {
                return;
            }
            if inter.needs_table && ctx.index.links.table_of(object).is_none() {
                return;
            }
            if !obj.allows(inter, sim.id) {
                return;
            }
            if inter.seat_near > 0.0 {
                reserved_seat = seat_near_object(sim, ctx, objects, object, inter);
            }
            // From the tile by the slot they'll get, when it has a place of its own (`places`).
            let t = match reserved_seat {
                Some(seat) => {
                    let s = &objects[seat as usize];
                    let def = &content.objects[s.def];
                    s.pick_slot(sim.id, 0..def.slots as usize, Some((def, sim.pos)))
                        .and_then(|k| crate::surface::slot_approach(content, nav, s, k))
                        .or_else(|| ctx.index.links.approach_of(seat))
                        .unwrap_or_else(|| s.front_tile(content))
                }
                None => obj
                    .slot_for(sim.id, &content.objects[obj.def], interaction, sim.pos)
                    .and_then(|k| crate::surface::slot_approach(content, nav, obj, k))
                    .or_else(|| ctx.index.links.approach_of(object))
                    .unwrap_or_else(|| obj.front_tile(content)),
            };
            let tags = inter.tags;
            ([t.0 as f32 + 0.5, t.1 as f32 + 0.5], t, tags)
        }
        TaskKind::MoveTo { x, z } => ([x, z], (x.floor() as i32, z.floor() as i32), 0),
        TaskKind::Clean { x, z } => ([x as f32 + 0.5, z as f32 + 0.5], (x, z), content.room_rules.clean.tags),
        // Accidents start on the spot (see `accident`), never from the queue.
        TaskKind::Spot { .. } => return,
        TaskKind::Repair { object } => {
            let Some(obj) = objects.get(object as usize).filter(|o| o.broken()) else {
                return;
            };
            let t = ctx.index.links.approach_of(object).unwrap_or_else(|| obj.front_tile(content));
            ([t.0 as f32 + 0.5, t.1 as f32 + 0.5], t, content.object_rules.repair.tags)
        }
        TaskKind::Work => {
            // Nearest way out of town; with no exits, work starts on the spot.
            let exit = ctx.exits.iter().copied().min_by(|a, b| {
                let d = |p: &[f32; 2]| (p[0] - sim.pos[0]).hypot(p[1] - sim.pos[1]);
                d(a).total_cmp(&d(b))
            });
            let Some(exit) = exit else {
                sim.transition = Some(crate::life::Transition::Work);
                return;
            };
            (exit, (exit[0].floor() as i32, exit[1].floor() as i32), 0)
        }
        // (Their visit starts once there's a way there: see below.)
        TaskKind::Visit { plot } => {
            let door = ctx.plots[plot as usize].arrival_point();
            (
                door,
                (door[0].floor() as i32, door[1].floor() as i32),
                content.visits.tags,
            )
        }
        TaskKind::GoHome => {
            let home = ctx.households[sim.household as usize].plot;
            let Some(door) = home.map(|p| ctx.plots[p as usize].arrival_point()) else {
                sim.transition = Some(crate::life::Transition::Home);
                return;
            };
            (door, (door[0].floor() as i32, door[1].floor() as i32), 0)
        }
        TaskKind::Answer { guest } => {
            let Some(other) = ctx.briefs.get(guest as usize) else {
                return;
            };
            let Some(goal) = conversation::approach_tile(nav, other.pos, sim.pos) else {
                return;
            };
            ([goal.0 as f32 + 0.5, goal.1 as f32 + 0.5], goal, 0)
        }
        TaskKind::Social { target, social } => {
            let Some(other) = ctx.briefs.get(target as usize) else {
                return;
            };
            if target == sim.id {
                return;
            }
            sim.social_waited = 0;
            // Within earshot: they call out to them, and the two meet halfway (see
            // `conversation::update`); otherwise over to them until they are (or right there).
            sim.called = other.available && conversation::calls_out(nav, &content.socials[social], sim.pos, other.pos);
            if sim.called {
                sim.pose = Pose::Stand;
                let tags = content.socials[social].tags;
                sim.current = Some(Activity { task, phase: Phase::Calling { ticks: 0 }, tags });
                return;
            }
            let Some(goal) = conversation::approach_tile(nav, other.pos, sim.pos) else {
                return;
            };
            (
                [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5],
                goal,
                content.socials[social].tags,
            )
        }
        // Over to the group (`conversation::update` finds them a place in it).
        TaskKind::Join { leader } => {
            let Some(other) = ctx.briefs.get(leader as usize) else {
                return;
            };
            let Some((_, social)) = other.conversing.filter(|c| c.0 == leader && leader != sim.id) else {
                return;
            };
            let Some(goal) = conversation::approach_tile(nav, other.pos, sim.pos) else {
                return;
            };
            ([goal.0 as f32 + 0.5, goal.1 as f32 + 0.5], goal, content.socials[social].tags)
        }
    };
    // A place on the seat for the use (two housemates don't both sit on the same sofa).
    if let Some(seat) = reserved_seat.take() {
        let def = &content.objects[objects[seat as usize].def];
        if objects[seat as usize].reserve_seat(sim.id, def, sim.pos) {
            sim.use_seat = Some(seat);
        }
    }
    let Some(waypoints) = walk_route(ctx, sim, goal, target) else {
        if let Some(seat) = sim.use_seat.take() {
            objects[seat as usize].release(sim.id);
        }
        return;
    };
    if let TaskKind::Visit { plot } = task.kind {
        sim.transition = Some(crate::life::Transition::Visit {
            plot,
            directed: task.directed,
        });
    }
    // A place at it for the repair (two housemates don't both set off for it).
    if let TaskKind::Repair { object } = task.kind {
        let obj = &mut objects[object as usize];
        if !obj.reserve(sim.id, content.objects[obj.def].slots) {
            return;
        }
    }
    if let TaskKind::Use { object, interaction } = task.kind {
        let obj = &mut objects[object as usize];
        if !obj.reserve_for(sim.id, &content.objects[obj.def], interaction, sim.pos) {
            if let Some(seat) = sim.use_seat.take() {
                objects[seat as usize].release(sim.id);
            }
            return;
        }
        // Off to bed (they can get there): the toilet first, out of habit, and then they
        // choose bed again.
        let sleep = content.objects[obj.def].interactions[interaction].tags & content.day_rhythm.sleep_tags != 0;
        if sleep
            && !task.directed
            && let Some(first) = habit(sim, ctx, objects, HabitTime::Bed, ctx.object_plot[object as usize])
        {
            objects[object as usize].release(sim.id);
            if let Some(seat) = sim.use_seat.take() {
                objects[seat as usize].release(sim.id);
            }
            return start_task(sim, first, ctx, objects, items);
        }
    }
    sim.pose = Pose::Stand;
    sim.current = Some(Activity {
        task,
        phase: Phase::Routing { waypoints, next: 0 },
        tags,
    });
}

/// The nearest free seat within `inter`'s `seatNear` metres of `object`, one they may sit on
/// here. Seats are the sitting interactions residents eat in (`cooking.seatTags`). For a use
/// with `seatFacing` the slot they'd get on it looks at the object (within `SEAT_LOOKS`), and
/// for a screen (`SeatFacing::Front`) sits in front of it (within `SEAT_IN_FRONT` of the way it
/// faces): no sofa with its back to the TV.
fn seat_near_object(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    object: u32,
    inter: &crate::content::Interaction,
) -> Option<u32> {
    let content = ctx.content;
    let plot = ctx.object_plot.get(object as usize).copied().flatten();
    let rules = &content.cooking;
    let it = &objects[object as usize];
    let (tx, tz) = ctx
        .index
        .links
        .approach_of(object)
        .unwrap_or_else(|| it.front_tile(content));
    let target = [tx as f32 + 0.5, tz as f32 + 0.5];
    let middle = crate::surface::middle(content, it);
    let faces = |o: &ObjectInstance| {
        let Some(facing) = inter.seat_facing else { return true };
        let def = &content.objects[o.def];
        let Some(slot) = o.pick_slot(sim.id, 0..def.slots as usize, Some((def, sim.pos))) else { return false };
        let at = o.slot_position(content, slot);
        let yaw = o.slot_yaw(content, slot);
        let to = [middle[0] - at[0], middle[1] - at[1]];
        let d = to[0].hypot(to[1]).max(1e-3);
        let looks = (yaw.sin() * to[0] + yaw.cos() * to[1]) / d >= SEAT_LOOKS.cos();
        let a = it.angle();
        let in_front = facing == crate::content::SeatFacing::Around || -(a.sin() * to[0] + a.cos() * to[1]) / d >= SEAT_IN_FRONT.cos();
        looks && in_front
    };
    objects
        .iter()
        .filter(|o| o.id != object)
        .filter(|o| {
            plot.is_none_or(|p| ctx.object_plot.get(o.id as usize).copied().flatten() == Some(p))
        })
        .filter(|o| {
            let def = &content.objects[o.def];
            !o.broken() && o.can_use(sim.id, def.slots) && o.sits(def, content.life.stage(sim.age), rules.seat_tags, rules.seat_avoid)
        })
        .filter(|o| access(ctx, sim, ctx.object_plot[o.id as usize]) != Access::None)
        .filter(|o| faces(o))
        .map(|o| {
            let (fx, fz) = ctx
                .index
                .links
                .approach_of(o.id)
                .unwrap_or_else(|| o.front_tile(content));
            (
                o.id,
                ctx.nav
                    .stairs
                    .distance([fx as f32 + 0.5, fz as f32 + 0.5], target),
            )
        })
        .filter(|&(_, d)| d <= inter.seat_near)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(o, _)| o)
}

/// A seat taken for a `seatFacing` use looks at the thing within this of straight ahead...
const SEAT_LOOKS: f32 = std::f32::consts::FRAC_PI_3;
/// ...and, for a screen, is within this of the way it faces.
const SEAT_IN_FRONT: f32 = std::f32::consts::FRAC_PI_3;

/// Where `sim` may walk on the way to tile `goal` (see `path::Grounds`): home, the place
/// they're a guest at, public places, the plot they're on now (nobody is stuck in a garden)
/// and the one they're heading for. Everyone else's garden only when there's no other way.
pub(crate) fn grounds<'a>(
    tile_plot: &'a [Option<u32>],
    plots: &[Plot],
    households: &[Household],
    nav: &NavGrid,
    sim: &Sim,
    goal: (i32, i32),
) -> Grounds<'a> {
    let mut grounds = Grounds { tile_plot, open: plots.iter().map(|p| p.public).collect() };
    let here = sim.tile();
    let ends = [here, goal].map(|(x, z)| grounds.plot(nav, x, z));
    let home = households.get(sim.household as usize).and_then(|h| h.plot);
    for p in ends.into_iter().chain([home, sim.visiting.map(|v| v.plot)]).flatten() {
        if let Some(open) = grounds.open.get_mut(p as usize) {
            *open = true;
        }
    }
    grounds
}

/// `route` for `sim`, keeping out of other people's gardens (see `grounds`).
pub(crate) fn walk_route(ctx: &Ctx, sim: &Sim, goal: (i32, i32), target: [f32; 2]) -> Option<Vec<[f32; 2]>> {
    let grounds = grounds(ctx.tile_plot, ctx.plots, ctx.households, &ctx.nav, sim, goal);
    route(&ctx.nav, Some(ctx.reach), sim.pos, sim.tile(), goal, target, Some(&grounds))
}

/// Smoothed waypoints from `pos` to `target`, or None if unreachable; round the gardens
/// `grounds` keeps them out of, if any. With `reach`, a goal in another walkable region is
/// refused before the search starts (it would flood the town and still find nothing).
pub(crate) fn route(
    nav: &NavGrid,
    reach: Option<&crate::path::Reach>,
    pos: [f32; 2],
    start: (i32, i32),
    goal: (i32, i32),
    target: [f32; 2],
    grounds: Option<&Grounds>,
) -> Option<Vec<[f32; 2]>> {
    if let Some(reach) = reach
        && !reach.reaches(nav, start, goal)
    {
        return None;
    }
    let path = nav.find_path_with(start, goal, grounds)?;
    let mut waypoints = nav.smooth_with(&path, grounds);
    // The first waypoint is the centre of the current tile; skip it when we can head on directly.
    if waypoints.len() > 1 && nav.line_walkable_with(pos, waypoints[1], grounds) {
        waypoints.remove(0);
    }
    match waypoints.last_mut() {
        Some(last) => *last = target,
        None => waypoints.push(target),
    }
    Some(waypoints)
}

fn progress(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut crate::items::Items, rng: &mut Rng, ticks: u32) {
    let content = ctx.content;
    let minutes = MINUTES_PER_TICK * ticks as f32;
    let Some(act) = sim.current.as_mut() else {
        return;
    };
    // A use that ends on its first step, short of its time: given up straight away (see `after`).
    let mut given_up = false;
    let finished = match &mut act.phase {
        // Waiting at the door; `life::doors` lets them in or sends them home.
        Phase::Knocking { ticks: waited, .. } => {
            *waited += ticks;
            false
        }
        // Waiting for a busy station: try again.
        Phase::Queued { ticks: waited } => {
            *waited += ticks;
            let waited = *waited;
            return crate::cooking::queued(sim, ctx, objects, items, waited);
        }
        Phase::Routing { waypoints, next } => {
            let per_tick = WALK_SPEED * sim.mods.walk_speed;
            let mut budget = per_tick * ticks as f32;
            while budget > 0.0 && *next < waypoints.len() {
                let wp = waypoints[*next];
                // Off the top step onto the landing (or back): a storey's rows are far apart.
                let stairs = &ctx.nav.stairs;
                if stairs.depth > 0 && stairs.storey(wp[1].floor() as i32) != stairs.storey(sim.pos[1].floor() as i32) {
                    sim.pos = wp;
                    budget -= 1.0;
                    *next += 1;
                    continue;
                }
                let (dx, dz) = (wp[0] - sim.pos[0], wp[1] - sim.pos[1]);
                let dist = dx.hypot(dz);
                if dist > 1e-4 {
                    sim.yaw = dx.atan2(dz);
                }
                if dist <= budget {
                    sim.pos = wp;
                    budget -= dist;
                    *next += 1;
                } else {
                    sim.pos[0] += dx / dist * budget;
                    sim.pos[1] += dz / dist * budget;
                    budget = 0.0;
                }
            }
            if *next >= waypoints.len() {
                arrive(sim, ctx, objects, items);
                // Catching up on a turn (`lod.rs`): they got there partway through it, and
                // what they came for gets the rest.
                let rest = ((budget / per_tick) as u32).min(ticks - 1);
                let using = sim.current.as_ref().is_some_and(|a| matches!(a.phase, Phase::Using { .. }));
                if rest > 0 && using {
                    progress(sim, ctx, objects, items, rng, rest);
                }
            }
            false
        }
        Phase::Using { elapsed } => {
            let first = *elapsed == 0.0;
            *elapsed += minutes;
            let elapsed = *elapsed;
            match act.task.kind {
                TaskKind::Use {
                    object,
                    interaction,
                } => {
                    let urgent = urgent_needs(sim, ctx, objects);
                    let done = use_object(sim, ctx, &objects[object as usize], interaction, elapsed, minutes, urgent);
                    given_up = done && first && content.objects[objects[object as usize].def].interactions.get(interaction).is_some_and(|i| elapsed < i.minutes);
                    // Every finished use wears it a little, by how much of a full use it was (a
                    // nap cut short when they're rested is a fraction); better quality wears slower.
                    if done {
                        let o = &mut objects[object as usize];
                        let def = &content.objects[o.def];
                        let rules = &content.object_rules;
                        let was = o.broken();
                        let share = def.interactions.get(interaction).map_or(1.0, |it| (elapsed / it.minutes.max(1.0)).min(1.0));
                        let wear = def.wear_per_use * share * (1.0 - rules.wear_per_quality * f32::from(o.quality));
                        o.wear = (o.wear + wear.max(0.0)).min(1.0);
                        if !was && o.broken() {
                            sim.news = Some(News::Broke(object));
                        }
                    }
                    done
                }
                TaskKind::Repair { object } => {
                    let rules = &content.object_rules.repair;
                    if let Some(s) = rules.skill {
                        let max = content.skill_rules.max_level;
                        sim.skills[s] = (sim.skills[s] + rules.skill_gain * minutes / 60.0).min(max);
                    }
                    let done = elapsed >= rules.minutes || !objects[object as usize].broken();
                    if done && objects[object as usize].broken() {
                        let level = rules.skill.map_or(0.0, |s| sim.skills[s]);
                        if rng.next_f32() < rules.chance_at(level) {
                            objects[object as usize].wear = 0.0;
                            sim.news = Some(News::Repaired(object));
                        }
                    }
                    done
                }
                TaskKind::Spot { accident } => {
                    let Some(rest) = content.accidents[accident].rest.as_ref() else {
                        return end_activity(sim, content, objects);
                    };
                    for (n, g) in rest.gain_per_minute.iter().enumerate().take(content.needs.len()) {
                        sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
                    }
                    elapsed >= rest.minutes
                }
                TaskKind::Clean { x, z } => {
                    let rules = &content.room_rules.clean;
                    let done = elapsed >= rules.minutes;
                    if done {
                        sim.mess = Some((x, z, -rules.amount));
                    }
                    done
                }
                TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. } => {
                    crate::cooking::using(sim, ctx, objects, items, elapsed, minutes)
                }
                TaskKind::Phone { .. } => {
                    let task = act.task;
                    crate::phone::using(sim, content, task, elapsed, minutes)
                }
                _ => unreachable!("only Use, Repair, Spot, Clean, Cook, Eat, Dishes and Phone tasks have a Using phase"),
            }
        }
        // Conversations are driven by `conversation::update`, which sees both Sims.
        Phase::Calling { .. } | Phase::Waiting { .. } | Phase::Conversing { .. } => false,
    };
    if finished {
        // A recipe's step, eating, washing up: on to what comes next.
        if sim.current.as_ref().is_some_and(|a| crate::cooking::chain(&a.task.kind)) {
            let ate = sim.current.as_ref().is_some_and(|a| matches!(a.task.kind, TaskKind::Eat { stage: 1, .. }));
            crate::cooking::step_done(sim, ctx, objects, items);
            if sim.current.is_some() {
                return;
            }
            // After a meal: the toilet, out of habit.
            if ate && sim.follow_up.is_none() {
                sim.follow_up = habit(sim, ctx, objects, HabitTime::Meal, ctx.briefs[sim.id as usize].plot);
            }
        } else {
            let used = sim.current.as_ref().map(|a| a.task);
            end_activity(sim, content, objects);
            if let Some(task) = used
                && let TaskKind::Use { object, interaction } = task.kind
                && let Some(o) = objects.get(object as usize)
                && let Some(inter) = content.objects[o.def].interactions.get(interaction)
            {
                // Food fetched: they sit down with it (at a table if there is one) and eat.
                if let Some(recipe) = inter.eats {
                    let quality = content.object_rules.quality_factor(o.quality) * inter.skill_factor(&content.skill_rules, &sim.skills);
                    crate::cooking::eat_now(sim, Task { routine: false, ..task }, recipe, quality, ctx, objects, items);
                    if sim.current.is_some() {
                        return;
                    }
                } else if inter.after != 0 {
                    // (Not after a use given up the moment it began: no hands to wash after a toilet
                    // visit that didn't happen.)
                    if !given_up {
                        sim.follow_up = follow_up(sim, ctx, objects, object, inter.after);
                    }
                } else if inter.close_after || inter.tags & (content.day_rhythm.sleep_tags | content.day_rhythm.nap_tags) != 0 {
                    // Up from a night's sleep: the toilet first, out of habit.
                    if inter.tags & content.day_rhythm.sleep_tags != 0 {
                        sim.follow_up = habit(sim, ctx, objects, HabitTime::Wake, ctx.object_plot[object as usize]);
                    }
                    // Then what sleeping or napping opened is closed again (a wall bed folded
                    // up, a sofa bed's bed put away; a hot tub's cover put on after a soak,
                    // `close_after`), unless someone is still in it: whoever gets up last does it.
                    if let Some(close) = closing(sim, content, o, object) {
                        match sim.follow_up {
                            None => sim.follow_up = Some(close),
                            Some(_) => sim.queue.push_front(close),
                        }
                    }
                }
            }
        }
        // Catching up on a turn: it ended partway through, on average half a turn ago.
        sim.idle_ticks = (ticks - 1) / 2;
    }
}

/// One tick of using an object (`minutes` long: more when catching up on a turn, see
/// `lod.rs`), with needs `pressing` (`urgent_needs`). Returns whether the Sim is done.
fn use_object(
    sim: &mut Sim,
    ctx: &Ctx,
    obj: &ObjectInstance,
    interaction: usize,
    elapsed: f32,
    minutes: f32,
    pressing: u32,
) -> bool {
    let content = ctx.content;
    let inter = &content.objects[obj.def].interactions[interaction];
    // Quality and skill scale what the Sim gets out of it (not what it costs).
    let boost = quality_boost(content, inter, obj.quality) * inter.skill_factor(&content.skill_rules, &sim.skills);
    let rhythm = &content.day_rhythm;
    // Napping: no other nap until a while after this one.
    if inter.tags & rhythm.nap_tags != 0 {
        sim.nap_until = ctx.tick + social::minutes_to_ticks(rhythm.nap_cooldown_hours * 60.0);
    }
    if inter.cares()
        && let Some(baby) = obj.users().find(|&u| u != sim.id)
    {
        let per_tick = inter.care.map(|g| g / inter.minutes * minutes);
        sim.care_out = Some((baby, per_tick));
    }
    let (mut any_gain, mut satisfied, mut urgent, mut wakes) = (false, true, false, false);
    let (mut fills, mut emptying) = (0u32, false);
    let low = content.autonomy.urgent_below;
    for n in 0..content.needs.len() {
        let g = inter.gain_per_minute[n];
        if g > 0.0 {
            any_gain = true;
            fills |= 1 << n;
            let g = g * sim.mods.need_gain[n] * boost;
            sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
            satisfied &= sim.needs[n] >= 0.999;
        } else {
            if g < 0.0 {
                // Costs (a workout makes you hungry and sweaty).
                sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
                emptying |= sim.needs[n] < low && !content.needs[n].room;
            }
            // (Surroundings are never urgent: they follow the room, nothing fills them. Nor is
            // what it's for: a snack fetched to eat sitting down fills hunger only then.)
            urgent |= sim.needs[n] < low && !content.needs[n].room && inter.total_gain[n] <= 0.0;
            wakes |= sim.needs[n] < low && content.day_rhythm.wake_for & (1 << n) != 0;
        }
    }
    for s in 0..content.skills.len() {
        if inter.skill_gain[s] > 0.0 {
            practise(sim, content, s, inter.skill_gain[s] * boost, minutes);
        }
    }
    let sleep = inter.tags & rhythm.sleep_tags != 0;
    let done = if inter.baby {
        // A baby lies in the crib until they grow out of it (`lifecycle.rs`), asleep or awake:
        // needing something doesn't get them up (they can't: they cry, and a carer comes), nor
        // does being rested.
        false
    } else if sleep && crate::planner::planned_now(content, sim, inter) {
        // A planned night's sleep lasts the block unless a need that wakes people gets urgent.
        wakes
    } else if sleep {
        // One sleep until rested (only the need it's for counts: about 8 hours from tired,
        // less in a better bed), but nobody gets up in the middle of the night: once rested,
        // they lie in until a little before they'd get up anyway. On a work or school day the
        // alarm goes off in time to get ready, rested or not.
        let rested = rhythm.rest_need.is_none_or(|n| sim.needs[n] >= 0.999);
        let longest = f32::from(content.planner.max_sleep_minutes);
        wakes || elapsed >= longest || (rested && !before_dawn(sim, ctx)) || getting_up(sim, ctx)
    } else {
        // Workouts and other draining activities stop before they empty the needs they cost.
        // Planned activities go on for their time even once the needs they fill are full.
        // Whatever they chose themselves (not a routine, nor for one of them) stops for a need
        // about to run out that home has something for, if it would before the end (off to the
        // toilet halfway through a film, not off the toilet for a bite to eat).
        let planned = crate::planner::planned_now(content, sim, inter);
        let chosen = sim.current.as_ref().is_some_and(|a| !a.task.directed && !a.task.routine);
        let left = inter.minutes - elapsed;
        let drained = emptying
            || (urgent && planned)
            || (chosen && breaks_off(sim, content, pressing, fills, left, inter, low));
        let routine = sim.current.as_ref().is_some_and(|a| a.task.routine);
        // Practising a skill they still want to (as when choosing: at home or in a public place)
        // goes on for its time too, until practice gets boring; stopping it as soon as the needs
        // are full would have them back at it a moment later, over and over.
        let practising = satisfied
            && inter.trains_skills()
            && access(ctx, sim, ctx.object_plot[obj.id as usize]) == Access::Full
            && training_interest(sim, content, inter) > MIN_AUTONOMY_SCORE;
        elapsed >= inter.minutes || (any_gain && satisfied && !planned && !routine && !practising) || drained
    };
    // Homework done: better at school (by how much of it they did).
    if done && inter.homework > 0.0 && content.life.stage_at(sim.age).is_some_and(|s| s.school) {
        let share = (elapsed / inter.minutes.max(1.0)).min(1.0);
        sim.grade = (sim.grade + inter.homework * share).clamp(0.0, 100.0);
    }
    // A finished cook (or wash, or snack) leaves a mess around the object.
    if done && inter.dirt > 0.0 {
        let (fx, fz) = ctx.index.links.approach_of(obj.id).unwrap_or_else(|| obj.front_tile(content));
        sim.mess = Some((fx, fz, inter.dirt));
    }
    if done && let Some(m) = inter.earns_feeling(elapsed, &sim.skills) {
        social::add_feeling(&mut sim.feelings, m, &content.feelings, ctx.tick);
    }
    done
}

fn arrive(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], items: &mut crate::items::Items) {
    let content = ctx.content;
    // Paid on arrival; if the money ran out on the way, never mind. Nor is something used that
    // broke on the way (someone else wore it out).
    if let Some(TaskKind::Use {
        object,
        interaction,
    }) = sim.current.as_ref().map(|a| a.task.kind)
    {
        // Nor is something opened that someone opened on the way, or closed on someone in it.
        let inter = &content.objects[objects[object as usize].def].interactions[interaction];
        if objects[object as usize].broken()
            || !requirements_met(ctx, objects, object, inter)
            || !objects[object as usize].allows(inter, sim.id)
        {
            // (An order to close it on someone who got there first: the player is told.)
            let directed = sim.current.as_ref().is_some_and(|a| a.task.directed);
            if directed && !objects[object as usize].broken() && objects[object as usize].closes_on_someone(inter, sim.id) {
                sim.news.get_or_insert(News::NotClosed(object));
            }
            end_activity(sim, content, objects);
            return;
        }
        let cost = inter.cost;
        let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
        if !affordable(cost, funds) {
            end_activity(sim, content, objects);
            return;
        }
        sim.pending_spend += cost.max(0);
    }
    let Some(act) = sim.current.as_mut() else {
        return;
    };
    match act.task.kind {
        TaskKind::MoveTo { .. } => sim.current = None,
        TaskKind::Clean { .. } => {
            sim.pose = Pose::Stand;
            act.phase = Phase::Using { elapsed: 0.0 };
        }
        TaskKind::Spot { .. } => {}
        TaskKind::Repair { object } => {
            let c = objects[object as usize].centre(content);
            sim.yaw = (c[0] - sim.pos[0]).atan2(c[1] - sim.pos[1]);
            sim.pose = Pose::Stand;
            act.phase = Phase::Using { elapsed: 0.0 };
        }
        TaskKind::Work => {
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Work);
        }
        TaskKind::Visit { plot } => {
            // At a house (not a park) with door rules: knock and wait (`life::doors`).
            if content.visits.door.is_some() && !ctx.plots[plot as usize].public {
                let p = &ctx.plots[plot as usize];
                let centre = [p.x as f32 + p.w as f32 / 2.0, p.z as f32 + p.d as f32 / 2.0];
                sim.yaw = (centre[0] - sim.pos[0]).atan2(centre[1] - sim.pos[1]);
                sim.pose = Pose::Stand;
                act.phase = Phase::Knocking { ticks: 0, host: None };
            } else {
                sim.current = None;
            }
        }
        TaskKind::Answer { guest } => {
            if let Some(other) = ctx.briefs.get(guest as usize) {
                sim.yaw = (other.pos[0] - sim.pos[0]).atan2(other.pos[1] - sim.pos[1]);
            }
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Answered { guest });
        }
        TaskKind::GoHome => {
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Home);
            // Home again: the toilet first, out of habit.
            sim.follow_up = habit(sim, ctx, objects, HabitTime::Home, ctx.households[sim.household as usize].plot);
        }
        TaskKind::Social { .. } | TaskKind::Join { .. } => act.phase = Phase::Waiting { ticks: 0 },
        TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. } => crate::cooking::arrive(sim, ctx, objects, items),
        TaskKind::Phone { .. } => crate::phone::arrive(sim, ctx, objects),
        TaskKind::Use {
            object,
            interaction,
        } => {
            let obj = &objects[object as usize];
            let inter = &content.objects[obj.def].interactions[interaction];
            let c = obj.centre(content);
            if let Some(seat) = sim.use_seat {
                // Watching from the sofa: sit on the seat, facing the way it faces, and step
                // back off where they came from (see `end_activity`).
                let s = &objects[seat as usize];
                let slot = s.slot_of(sim.id).unwrap_or(0);
                sim.seated_from = Some((sim.pos[0].floor() as i32, sim.pos[1].floor() as i32));
                sim.pos = s.slot_position(content, slot);
                sim.yaw = s.slot_yaw(content, slot);
                sim.pose = Pose::Sit;
            } else if inter.pose == Pose::Stand && !inter.inside {
                // At their slot's own place, come to its tile (the end of a ping-pong table),
                // facing the way it says; else facing the thing.
                let slot = obj.slot_of(sim.id).unwrap_or(0);
                let here = (sim.pos[0].floor() as i32, sim.pos[1].floor() as i32);
                let place = Some(obj.slot_position(content, slot))
                    .filter(|p| !content.objects[obj.def].places.is_empty() && (p[0].floor() as i32, p[1].floor() as i32) == here);
                if let Some(p) = place {
                    sim.pos = p;
                    sim.yaw = obj.slot_yaw(content, slot);
                } else {
                    sim.yaw = (c[0] - sim.pos[0]).atan2(c[1] - sim.pos[1]);
                }
                sim.pose = inter.pose;
            } else {
                // Onto the seat or bed, or into the shower, facing the way it faces (or its
                // slot's place does); they step back off where they came from (see `end_activity`).
                let slot = obj.slot_of(sim.id).unwrap_or(0);
                sim.seated_from = Some((sim.pos[0].floor() as i32, sim.pos[1].floor() as i32));
                sim.pos = obj.slot_position(content, slot);
                sim.yaw = obj.slot_yaw(content, slot);
                sim.pose = inter.pose;
            }
            act.phase = Phase::Using { elapsed: 0.0 };
            // Opened or closed as the use starts (a wall bed folds down as they lie down).
            if let Some(open) = inter.leaves
                && objects[object as usize].open != open
            {
                objects[object as usize].open = open;
                sim.reshaped = true;
            }
            // A crop picked as the harvest starts: it lies fallow, then grows again.
            if inter.picks && objects[object as usize].growth >= 0.0 {
                objects[object as usize].growth = -1.0;
                sim.reshaped = true;
            }
        }
    }
}

/// Just before they leave for work or school (`DayRhythm::ready_minutes`): what they do out of
/// habit first (`HabitTime::Leave`), dropping what they chose themselves for it (not what
/// they were told to do, a routine, cooking or eating, or something for that habit anyway).
fn ready_to_leave(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> Option<Task> {
    let content = ctx.content;
    let rhythm = &content.day_rhythm;
    let low = rhythm.habits.iter().filter(|h| h.at == HabitTime::Leave && sim.needs[h.need] < h.below).fold(0u32, |m, h| m | 1 << h.need);
    let free_will = ctx.autonomy && ctx.households[sim.household as usize].free_will;
    if low == 0 || !free_will || sim.follow_up.is_some() || sim.queue.front().is_some_and(|t| t.directed) || sim.asleep(content) {
        return None;
    }
    let leave = leave_today(sim, ctx)?;
    if leave > ctx.tick + social::minutes_to_ticks(rhythm.ready_minutes) {
        return None;
    }
    match sim.current.as_ref().map(|a| a.task) {
        None => {}
        Some(Task { kind: TaskKind::Use { object, interaction }, directed: false, routine: false }) => {
            let inter = &content.objects[objects[object as usize].def].interactions[interaction];
            if inter.fills() & low != 0 {
                return None;
            }
        }
        Some(_) => return None,
    }
    habit(sim, ctx, objects, HabitTime::Leave, ctx.briefs[sim.id as usize].plot)
}

/// Whether their home has something working that fills need `n` (a toilet, a bed, a fridge),
/// busy or not.
fn home_fills(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], n: usize) -> bool {
    home_needs(sim, ctx, objects, 1 << n, false) != 0
}

/// Which of the needs in `mask` (bit per need) their home has something working for (not
/// counting what's only done with a baby there, nor a recipe the home lacks a station for: a
/// stove is no food without a fridge). `free`: only what they could go and use now, with a
/// place on it free (or theirs already), and not something they couldn't get to a moment ago
/// (every bed taken is no bed for them tonight).
fn home_needs(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], mask: u32, free: bool) -> u32 {
    let Some(home) = ctx.households[sim.household as usize].plot else {
        return 0;
    };
    let usable = |o: &ObjectInstance| {
        !free || ((o.has_free_slot(ctx.content.objects[o.def].slots) || o.slot_of(sim.id).is_some()) && !sim.couldnt_reach(o.id, ctx.tick))
    };
    // (Busy or not: a cook waits for a station.)
    let stations = crate::cooking::home_stations(ctx, sim.household, objects);
    let possible = |r: usize| ctx.content.recipes[r].possible(stations);
    let mut found = 0;
    for o in ctx.index.on(home).iter().map(|&o| &objects[o as usize]).filter(|o| !o.broken() && usable(o)) {
        let def = &ctx.content.objects[o.def];
        let startable = |&(n, _): &(usize, &crate::content::Interaction)| !free || o.can_use_for(sim.id, def, n);
        for (_, i) in def.interactions.iter().enumerate().filter(startable).filter(|(_, i)| {
            i.autonomous && !i.cares() && i.recipe.is_none_or(possible)
                && unfit(ctx.content, sim.age, i).is_none()
                && requirements_met(ctx, objects, o.id, i)
        }) {
            found |= i.fills() & mask;
        }
        if found == mask {
            break;
        }
    }
    found
}

/// Essentials about to run out (`AutonomyRules::urgent`) that something at home they could use
/// now fills (bit per need): residents see to them before anything else.
fn urgent_needs(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> u32 {
    let content = ctx.content;
    let rules = &content.autonomy;
    let low = (0..content.needs.len())
        .filter(|&n| !content.needs[n].room && rules.urgent & (1 << n) != 0 && sim.needs[n] < rules.urgent_below)
        .fold(0, |m, n| m | 1 << n);
    if low == 0 { 0 } else { home_needs(sim, ctx, objects, low, true) }
}

/// Whether something they chose themselves, `inter`, with `minutes` to go, stops for one of the
/// `pressing` needs: one it doesn't fill (`fills`, bit per need) that would run out before then.
fn breaks_off(sim: &Sim, content: &Content, pressing: u32, fills: u32, minutes: f32, inter: &crate::content::Interaction, low: f32) -> bool {
    pressing != 0 && pressing & fills == 0 && runs_out(sim, content, pressing, minutes, false, &inter.gain_per_minute, &[0.0; MAX_NEEDS], low * 0.5)
}

/// Whether one of the needs in `mask` would run down below `floor` in `minutes` of something
/// (asleep or not) that costs `costs` a minute (negative gains), and the walk `after` it to
/// what fills each.
#[allow(clippy::too_many_arguments)]
fn runs_out(
    sim: &Sim,
    content: &Content,
    mask: u32,
    minutes: f32,
    asleep: bool,
    costs: &[f32; MAX_NEEDS],
    after: &[f32; MAX_NEEDS],
    floor: f32,
) -> bool {
    let rhythm = &content.day_rhythm;
    (0..content.needs.len()).filter(|&n| mask & (1 << n) != 0).any(|n| {
        let factor = if asleep { rhythm.asleep_decay[n] } else { 1.0 };
        let per_minute = content.needs[n].decay_per_minute * sim.mods.need_decay[n];
        let used = (per_minute * factor - costs[n].min(0.0)) * minutes + per_minute * after[n];
        sim.needs[n] - used < floor
    })
}

/// How many minutes' walk the nearest thing for each need is (one they may use, not just for
/// babies): where they'd go for it after whatever they're about to do.
fn reach(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], usable: &[u32], per_minute: f32) -> [f32; MAX_NEEDS] {
    let content = ctx.content;
    let mut nearest = [f32::INFINITY; MAX_NEEDS];
    for o in usable.iter().map(|&o| &objects[o as usize]).filter(|o| !o.broken()) {
        if access(ctx, sim, ctx.object_plot[o.id as usize]) == Access::None {
            continue;
        }
        let fills = content.objects[o.def]
            .interactions
            .iter()
            .filter(|i| i.autonomous && !i.baby && !i.cares() && requirements_met(ctx, objects, o.id, i) && unfit(content, sim.age, i).is_none())
            .fold(0, |m, i| m | i.fills());
        if fills == 0 {
            continue;
        }
        let (fx, fz) = ctx.index.links.approach_of(o.id).unwrap_or_else(|| o.front_tile(content));
        let minutes = ctx.nav.stairs.distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos) / per_minute;
        for n in (0..content.needs.len()).filter(|&n| fills & (1 << n) != 0) {
            nearest[n] = nearest[n].min(minutes);
        }
    }
    nearest.map(|m| if m.is_finite() { m } else { 0.0 })
}

/// When they leave for work or school today (if they haven't yet).
fn leave_today(sim: &Sim, ctx: &Ctx) -> Option<u64> {
    leave_at(sim, ctx.content, ctx.tick)
}

/// When they leave for work or school later today (`leave_today`).
fn leave_at(sim: &Sim, content: &Content, tick: u64) -> Option<u64> {
    sim.job
        .as_ref()
        .and_then(|job| crate::life::shift_today(content, job, &sim.skills, tick))
        .or_else(|| crate::life::school_today(content, sim, tick))
        .map(|(leave, ..)| leave)
        .filter(|&leave| leave > tick)
}

/// What they do out of habit at a turning point of their day (`DayRhythm::habits`): for the
/// habit for `at` whose need is lowest (for it), the nearest quick thing on `plot` that fills
/// it. None: nothing to see to, or nothing there for it.
fn habit(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], at: HabitTime, plot: Option<u32>) -> Option<Task> {
    let content = ctx.content;
    let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
    // (The lowest first, for its habit.)
    let mut due: Vec<&crate::content::Habit> = content.day_rhythm.habits.iter().filter(|h| h.at == at && sim.needs[h.need] < h.below).collect();
    due.sort_by(|a, b| (sim.needs[a.need] / a.below).total_cmp(&(sim.needs[b.need] / b.below)));
    due.into_iter().find_map(|h| {
        nearest(sim, ctx, objects, plot, None, |i| {
            i.autonomous
                && i.total_gain[h.need] > 0.0
                && i.minutes <= content.autonomy.quick_minutes
                && !i.baby
                && !i.needs_table
                && i.recipe.is_none()
                && affordable(i.cost, funds)
        })
    })
}

/// Whether what they're doing right now fills need `n` (on the toilet, in bed, eating). A meal
/// on its way counts all along: while it cooks, while they fetch it and sit down.
fn filling(sim: &Sim, content: &Content, objects: &[ObjectInstance], items: &crate::items::Items, n: usize) -> bool {
    let Some(act) = sim.current.as_ref() else {
        return false;
    };
    let using = matches!(act.phase, Phase::Using { .. });
    match act.task.kind {
        TaskKind::Use { object, interaction } if using => objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .is_some_and(|i| i.total_gain[n] > 0.0),
        TaskKind::Spot { accident } if using => content.accidents[accident]
            .rest
            .as_ref()
            .is_some_and(|r| r.gain_per_minute[n] > 0.0),
        TaskKind::Cook { recipe, .. } => content.recipes[recipe].effects[n] > 0.0,
        TaskKind::Eat { .. } => crate::cooking::eating(sim, items).is_some_and(|r| content.recipes[r].effects[n] > 0.0),
        _ => false,
    }
}

/// Closing `o` again after sleeping or napping in it, if that opened it (`Interaction::leaves`):
/// its interaction that closes it, if `sim` may do it now. A routine, like washing hands.
fn closing(sim: &Sim, content: &Content, o: &ObjectInstance, object: u32) -> Option<Task> {
    if !o.open {
        return None;
    }
    let interactions = &content.objects[o.def].interactions;
    let interaction = interactions.iter().position(|i| i.leaves == Some(false) && o.allows(i, sim.id))?;
    Some(Task { kind: TaskKind::Use { object, interaction }, directed: false, routine: true })
}

/// What follows using `object` when its interaction has an `after` (`tags`): the nearest
/// working object on the same plot with an interaction of those tags, one they may use and can
/// get to (a guest washes their hands at the host's sink). None: there's nothing like it.
fn follow_up(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance], object: u32, tags: TagMask) -> Option<Task> {
    let plot = ctx.object_plot.get(object as usize).copied().flatten();
    nearest(sim, ctx, objects, plot, Some(object), |i| i.tags & tags != 0 && !i.baby && i.recipe.is_none())
}

/// The nearest working object on `plot` (but `except`) with an interaction that `fits`, one
/// they may use and can get to, as a routine task.
fn nearest(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    plot: Option<u32>,
    except: Option<u32>,
    fits: impl Fn(&crate::content::Interaction) -> bool,
) -> Option<Task> {
    let content = ctx.content;
    if access(ctx, sim, plot) == Access::None {
        return None;
    }
    let usable = match sim.visiting {
        Some(visit) => ctx.index.on(visit.plot),
        None => ctx.index.usable_from(ctx.households[sim.household as usize].plot),
    };
    let approach = |o: &ObjectInstance| ctx.index.links.approach_of(o.id).unwrap_or_else(|| o.front_tile(content));
    let mut near: Vec<(f32, u32, usize)> = usable
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| Some(o.id) != except && ctx.object_plot[o.id as usize] == plot)
        .filter(|o| !o.broken() && o.has_free_slot(content.objects[o.def].slots))
        .filter_map(|o| {
            let def = &content.objects[o.def];
            let i = (0..def.interactions.len()).find(|&i| {
                fits(&def.interactions[i])
                    && unfit(content, sim.age, &def.interactions[i]).is_none()
                    && !upper_barred(content, o, sim.id, sim.age)
                    && requirements_met(ctx, objects, o.id, &def.interactions[i])
                    && o.allows(&def.interactions[i], sim.id)
                    && o.can_use_for(sim.id, def, i)
            })?;
            let (fx, fz) = approach(o);
            Some((ctx.nav.stairs.distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos), o.id, i))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (_, object, interaction) = near.into_iter().take(3).find(|&(_, o, _)| {
        let (fx, fz) = approach(&objects[o as usize]);
        route(&ctx.nav, Some(ctx.reach), sim.pos, sim.tile(), (fx, fz), [fx as f32 + 0.5, fz as f32 + 0.5], None).is_some()
    })?;
    Some(Task { kind: TaskKind::Use { object, interaction }, directed: false, routine: true })
}

/// Accident `a` happens to `sim` where they are (see `Content::accidents`).
fn accident(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], a: usize) {
    let content = ctx.content;
    let def = &content.accidents[a];
    sim.accident_at[a] = ctx.tick + def.cooldown_ticks;
    for (n, e) in def.effects.iter().enumerate().take(content.needs.len()) {
        sim.needs[n] = (sim.needs[n] + e).clamp(0.0, 1.0);
    }
    if def.dirt > 0.0 {
        let (x, z) = sim.tile();
        sim.mess = Some((x, z, def.dirt));
    }
    if let Some(f) = def.feeling {
        social::add_feeling(&mut sim.feelings, f, &content.feelings, ctx.tick);
    }
    sim.pending_spend += def.cost;
    if def.story {
        sim.news = Some(News::Accident(a));
        crate::planner::think(sim, crate::planner::ThoughtKind::Accident, a, ctx.tick);
    }
    if let Some(rest) = &def.rest {
        end_activity(sim, content, objects);
        sim.queue.retain(|t| t.directed);
        sim.pose = rest.pose;
        sim.current = Some(Activity {
            task: Task {
                kind: TaskKind::Spot { accident: a },
                directed: false,
                routine: false,
            },
            phase: Phase::Using { elapsed: 0.0 },
            tags: rest.tags,
        });
    }
}

/// Stops what they're doing: the object they used (or cooked at, sat at to eat, washed up at)
/// is free again, and they stand up in front of it (or step out of the shower). What they carry is put down after the step
/// (`cooking::after_step`).
pub(crate) fn end_activity(sim: &mut Sim, content: &Content, objects: &mut [ObjectInstance]) {
    let held = sim.current.take().and_then(|act| match act.task.kind {
        TaskKind::Use { object, .. } | TaskKind::Repair { object } => Some(object),
        TaskKind::Cook { station, .. } | TaskKind::Dishes { station, .. } => station,
        TaskKind::Eat { seat, .. } | TaskKind::Phone { seat, .. } => seat,
        _ => None,
    });
    if let Some(o) = held.map(|o| o as usize).filter(|&o| o < objects.len()) {
        objects[o].release(sim.id);
        if sim.pose != Pose::Stand || sim.seated_from.is_some() {
            let (fx, fz) = sim.seated_from.unwrap_or_else(|| objects[o].front_tile(content));
            sim.pos = [fx as f32 + 0.5, fz as f32 + 0.5];
        }
    }
    if let Some(seat) = sim.use_seat.take()
        && let Some(o) = objects.get_mut(seat as usize)
    {
        o.release(sim.id);
    }
    sim.seated_from = None;
    sim.pose = Pose::Stand;
    sim.idle_ticks = 0;
}

/// A free crib at home for a baby (one with no other baby in it): they're put straight in.
fn crib_for(sim: &mut Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> Option<Task> {
    let home = ctx.households[sim.household as usize].plot?;
    let content = ctx.content;
    let (object, interaction) = ctx.index.on(home).iter().find_map(|&o| {
        let o = &objects[o as usize];
        let def = &content.objects[o.def];
        let i = def.interactions.iter().position(|it| it.baby)?;
        let taken = o.users().any(|u| ctx.briefs.get(u as usize).is_some_and(|b| b.baby));
        (!o.broken() && !taken && o.can_use_for(sim.id, def, i)).then_some((o.id, i))
    })?;
    let (fx, fz) = ctx.index.links.approach_of(object).unwrap_or_else(|| objects[object as usize].front_tile(content));
    sim.pos = [fx as f32 + 0.5, fz as f32 + 0.5];
    Some(Task { kind: TaskKind::Use { object, interaction }, directed: false, routine: false })
}

/// Whether partners `a` and `b` could have a baby now: at home together, grown-ups young enough,
/// none on the way, and room for one more (in the home and in the town).
fn could_conceive(ctx: &Ctx, a: usize, b: usize) -> bool {
    let Some(rules) = ctx.content.life.pregnancy else { return false };
    let (x, y) = (&ctx.briefs[a], &ctx.briefs[b]);
    let h = x.household as usize;
    let fits = |s: &SimBrief| s.adult && s.age <= rules.max_age;
    let members = ctx.briefs.iter().filter(|s| s.here && s.household as usize == h).count();
    fits(x) && fits(y) && y.household == x.household && ctx.households[h].expecting.is_none()
        && members < ctx.content.rules.max_household
        && ctx.briefs.iter().filter(|s| s.here).count() < MAX_SIMS
}

/// The colours a player gives an object's parts, 0xRRGGBB by the name of the model's material
/// they go on (`body`, `stone`, `chrome`); a part not named is as it's made. The sim doesn't know
/// the parts: the renderer colours only those the model lets the player paint.
pub type Colors = BTreeMap<String, u32>;

/// Parts one object may colour at most.
pub const MAX_COLORS: usize = 16;

/// An object's colours as a command gives them: at most `MAX_COLORS` parts, each named by a
/// material (`[a-z0-9_]`, up to 32 characters), each 0xRRGGBB.
pub(crate) fn check_colors(colors: Colors) -> Result<Colors, Error> {
    if colors.len() > MAX_COLORS {
        return Err(Error::new(format!("too many colours: {} (at most {MAX_COLORS})", colors.len())));
    }
    for (part, &color) in &colors {
        if !part_name(part) {
            return Err(Error::new(format!("no such part: {part:?}")));
        }
        if color > 0xFF_FFFF {
            return Err(Error::new(format!("no such colour: {color:#x}")));
        }
    }
    Ok(colors)
}

fn part_name(part: &str) -> bool {
    (1..=32).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// An object's colours as saves and the view write them: `#rrggbb` by part.
pub fn colors_hex(colors: &Colors) -> BTreeMap<String, String> {
    colors.iter().map(|(part, &c)| (part.clone(), color_hex(c))).collect()
}

/// `colors_hex` read back, with an older file's one `color` on the `body` (the part every
/// paintable model's colour went on); what isn't a part or a colour is left out.
pub fn parse_colors(colors: &BTreeMap<String, String>, color: Option<&str>) -> Colors {
    let old = color.and_then(parse_color).filter(|_| colors.is_empty()).map(|c| ("body".to_string(), c));
    let parts = colors.iter().filter(|(part, _)| part_name(part)).filter_map(|(part, hex)| Some((part.clone(), parse_color(hex)?)));
    old.into_iter().chain(parts).take(MAX_COLORS).collect()
}

/// An object's colour as saves and the view write it: `#rrggbb`.
pub fn color_hex(color: u32) -> String {
    format!("#{color:06x}")
}

/// `color_hex` read back; None if it isn't one.
pub fn parse_color(hex: &str) -> Option<u32> {
    let digits = hex.strip_prefix('#')?;
    if digits.len() == 6 { u32::from_str_radix(digits, 16).ok() } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1},
                 {"id":"energy","label":"Energy","decayPerHour":0.05}],
        "objects":[
          {"id":"fridge","name":"Fridge","interactions":[
            {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.4}}]},
          {"id":"bed","name":"Bed","footprint":[2,2],"interactions":[
            {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0}}]}]}"#;

    const LOT: &str = r#"{"width":12,"depth":12,
        "walls":[[6,0,6,10]],
        "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"bed","x":9,"z":2,"rot":3}],
        "sims":[{"name":"Ada","x":3.5,"z":3.5}]}"#;

    fn world() -> World {
        World::from_json(CONTENT, LOT, 7).unwrap()
    }

    fn run_ticks(w: &mut World, n: u32) {
        for _ in 0..n {
            w.tick_once();
        }
    }

    #[test]
    fn objects_block_tiles_and_overlaps_fail() {
        let mut w = world();
        assert!(w.blocked()[w.lot.tile_index(1, 1)]);
        assert!(w.place_object("fridge", 1, 1, 0).is_err());
    }

    #[test]
    fn hungry_sim_eats_autonomously() {
        let mut w = world();
        w.sims[0].needs[0] = 0.05;
        w.sims[0].needs[1] = 1.0;
        run_ticks(&mut w, TICKS_PER_SECOND * 60);
        assert!(
            w.sims[0].needs[0] > 0.3,
            "hunger was {}",
            w.sims[0].needs[0]
        );
    }

    #[test]
    fn directed_command_walks_around_wall_and_lies_down() {
        let mut w = world();
        w.sims[0].needs[1] = 0.1;
        w.apply(Command::Use {
            sim: 0,
            object: 1,
            interaction: 0,
        })
        .unwrap();
        run_ticks(&mut w, TICKS_PER_SECOND * 30);
        let s = &w.sims[0];
        assert_eq!(s.pose, Pose::Lie);
        assert_eq!(w.objects[1].users().next(), Some(0));
        assert!(s.needs[1] > 0.1);
    }

    #[test]
    fn cancel_releases_object() {
        let mut w = world();
        w.apply(Command::Use {
            sim: 0,
            object: 1,
            interaction: 0,
        })
        .unwrap();
        run_ticks(&mut w, 2);
        assert_eq!(w.objects[1].users().next(), Some(0));
        w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
        assert!(!w.objects[1].in_use());
        assert_eq!(w.sims[0].pose, Pose::Stand);
    }

    #[test]
    fn watching_tv_sits_on_a_nearby_sofa_and_broadcasts() {
        const TV: &str = r#"{
            "needs":[{"id":"fun","label":"Fun","decayPerHour":0.1},
                     {"id":"hunger","label":"Hunger","decayPerHour":0.1}],
            "objects":[
              {"id":"tv","name":"TV","interactions":[
                {"id":"watch","label":"Watch TV","minutes":60,"seatNear":5,"broadcast":0.5,
                 "broadcastRange":6,"effects":{"fun":0.6},"tags":["screen"]}]},
              {"id":"sofa","name":"Sofa","interactions":[
                {"id":"relax","label":"Relax","minutes":30,"pose":"sit",
                 "effects":{"fun":0.1},"tags":["lounge"]}]}]}"#;
        const TV_LOT: &str = r#"{"width":12,"depth":12,
            "objects":[{"def":"tv","x":2,"z":2,"rot":0},{"def":"sofa","x":5,"z":2,"rot":0}],
            "sims":[{"name":"Ada","x":7.5,"z":7.5},{"name":"Bo","x":3.5,"z":3.5}]}"#;
        let mut w = World::from_json(TV, TV_LOT, 7).unwrap();
        w.apply(Command::Use {
            sim: 0,
            object: 0,
            interaction: 0,
        })
        .unwrap();
        run_ticks(&mut w, TICKS_PER_SECOND * 30);
        assert_eq!(w.sims[0].use_seat, Some(1), "sat on the sofa");
        assert_eq!(w.sims[0].pose, Pose::Sit);
        assert_eq!(w.objects[0].users().next(), Some(0));
        // Bo, in the room doing something else, hears it too.
        w.sims[1].needs[0] = 0.5;
        w.broadcast();
        assert!(
            w.sims[1].needs[0] > 0.5,
            "heard the TV: {}",
            w.sims[1].needs[0]
        );

        // Saved mid-watch, the watch goes on after loading: the seat is found again.
        let mut loaded = World::from_save_json(TV, &w.save_json()).unwrap();
        run_ticks(&mut loaded, 5);
        assert_eq!(loaded.sims[0].use_seat, Some(1), "sat on the sofa again");
        assert_eq!(loaded.sims[0].pose, Pose::Sit);
    }

    #[test]
    fn furniture_cannot_straddle_or_face_a_wall() {
        let mut w = world();
        // The wall runs along x = 6 for z 0..10: a 2-wide bed across it doesn't fit.
        let err = w.place_object("bed", 5, 5, 0).unwrap_err();
        assert!(err.to_string().contains("wall"), "{err}");
        // Facing +x (rot 1) from x = 5 means using it from across the wall.
        let err = w.place_object("fridge", 5, 7, 1).unwrap_err();
        assert!(err.to_string().contains("wall"), "{err}");
        // Turned away from the wall it's fine.
        w.place_object("fridge", 5, 7, 3).unwrap();
    }

    #[test]
    fn speed_controls_tick_rate() {
        let mut w = world();
        w.apply(Command::SetSpeed { speed: 0 }).unwrap();
        w.advance();
        assert_eq!(w.tick, 0);
        w.apply(Command::SetSpeed { speed: 3 }).unwrap();
        w.advance();
        assert_eq!(w.tick, TICKS_PER_STEP[3] as u64);
        assert!(w.apply(Command::SetSpeed { speed: 9 }).is_err());
    }

    #[test]
    fn quiet_hours_skip_ahead() {
        let content = CONTENT
            .replace(r#""pose":"lie","#, r#""pose":"lie","tags":["sleep"],"#)
            .replace(r#""needs":["#, r#""dayRhythm":{"sleepTags":["sleep"]},"needs":["#);
        let mut w = World::from_json(&content, LOT, 7).unwrap();
        w.apply(Command::SetSpeed { speed: 1 }).unwrap();
        w.apply(Command::SetAutoFast { enabled: true }).unwrap();
        w.advance();
        assert_eq!(w.tick, 1, "awake: normal speed");
        w.sims[0].needs[1] = 0.0;
        w.apply(Command::Use {
            sim: 0,
            object: 1,
            interaction: 0,
        })
        .unwrap();
        while !w.sims[0].asleep(&w.content) {
            w.tick_once();
        }
        let before = w.tick;
        w.advance();
        let fast = TICKS_PER_STEP[AUTO_FAST_SPEED as usize] as u64;
        assert_eq!(w.tick, before + fast, "asleep: time-lapse");
        w.apply(Command::SetSpeed { speed: 0 }).unwrap();
        w.advance();
        assert_eq!(w.tick, before + fast, "paused stays paused");
    }

    #[test]
    fn deterministic_for_same_seed() {
        let (mut a, mut b) = (world(), world());
        run_ticks(&mut a, 2000);
        run_ticks(&mut b, 2000);
        assert_eq!(a.sims[0].pos, b.sims[0].pos);
        assert_eq!(a.sims[0].needs, b.sims[0].needs);
    }

    #[test]
    fn neighbours_do_not_use_other_households_objects() {
        let lot = r#"{"width":20,"depth":10,
            "plots":[{"name":"A","x":0,"z":0,"w":10,"d":10},{"name":"B","x":10,"z":0,"w":10,"d":10}],
            "households":[{"name":"A","plot":0,"player":true},{"name":"B","plot":1}],
            "objects":[{"def":"fridge","x":2,"z":2,"rot":0}],
            "sims":[{"name":"Neighbour","household":1,"x":14.5,"z":4.5}]}"#;
        let mut w = World::from_json(CONTENT, lot, 1).unwrap();
        w.sims[0].needs[0] = 0.0;
        run_ticks(&mut w, TICKS_PER_SECOND * 60);
        assert!(!w.objects[0].in_use());
        assert!(w.sims[0].pos[0] > 10.0, "stayed home");
    }

    #[test]
    fn showering_happens_in_the_shower() {
        let content = r#"{
            "needs":[{"id":"hygiene","label":"Hygiene","decayPerHour":0.05}],
            "objects":[{"id":"shower","name":"Shower","interactions":[
              {"id":"shower","label":"Shower","minutes":20,"inside":true,"effects":{"hygiene":1.0}}]}]}"#;
        let lot = r#"{"width":10,"depth":10,
            "objects":[{"def":"shower","x":2,"z":2,"rot":0}],
            "sims":[{"name":"Ada","x":6.5,"z":6.5}]}"#;
        let mut w = World::from_json(content, lot, 1).unwrap();
        w.autonomy = false;
        w.sims[0].needs[0] = 0.1;
        w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
        let using = |w: &World| w.sims[0].current().is_some_and(|a| matches!(a.phase, Phase::Using { .. }));
        for _ in 0..TICKS_PER_SECOND * 60 {
            if using(&w) {
                break;
            }
            w.tick_once();
        }
        assert!(using(&w), "never got to the shower");
        let front = w.objects[0].front_tile(&w.content);
        let s = &w.sims[0];
        assert_eq!(s.tile(), (2, 2), "showers in the shower, not in front of it");
        assert_eq!(s.pose, Pose::Stand);
        assert!((s.yaw - w.objects[0].yaw()).abs() < 1e-4, "faces out of the shower");
        assert_eq!(s.seated_from, Some(front));

        // Saved mid-shower: back out in front of it.
        let loaded = World::from_save_json(content, &w.save_json()).unwrap();
        assert_eq!(loaded.sims[0].tile(), front);

        // Done: out where they stepped in from.
        for _ in 0..TICKS_PER_SECOND * 600 {
            if !using(&w) {
                break;
            }
            w.tick_once();
        }
        assert!(!using(&w) && !w.objects[0].in_use(), "still showering");
        assert!(w.sims[0].needs[0] > 0.5);
        assert_eq!(w.sims[0].tile(), front);
        assert_eq!(w.sims[0].seated_from, None);
    }

    #[test]
    fn a_full_queue_drops_neither_orders_nor_a_paid_recipe() {
        let task = |kind, directed| Task { kind, directed, routine: false };
        let walk = TaskKind::MoveTo { x: 1.0, z: 1.0 };
        let cook = |servings| TaskKind::Cook { recipe: 0, step: 1, station: None, origin: None, servings, skipped: 0, minutes: 0 };
        let care = task(TaskKind::Use { object: 0, interaction: 1 }, false);
        // Chosen for them, unpaid: the last of those goes.
        let mut q: VecDeque<Task> = [care, task(walk, true), task(cook(2), false), task(walk, false), task(walk, true), task(cook(0), false), task(walk, true)].into();
        trim_queue(&mut q, 1);
        assert_eq!(q.len(), MAX_QUEUE);
        assert!(matches!(q[4].kind, TaskKind::MoveTo { .. }) && q[4].directed && matches!(q[2].kind, TaskKind::Cook { servings: 2, .. }));
        // Nothing else to drop: over the cap.
        let mut q: VecDeque<Task> = [care, task(cook(3), false)].into_iter().chain([task(walk, true); MAX_QUEUE]).collect();
        trim_queue(&mut q, 1);
        assert_eq!(q.len(), MAX_QUEUE + 2);
    }

    /// A one-room home with a household and a fridge, for build and recolour.
    const HOME_LOT: &str = r#"{"width":12,"depth":12,
        "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[6.5,11.5]}],
        "households":[{"name":"Ada","plot":0,"player":true}],
        "objects":[{"def":"fridge","x":1,"z":1,"rot":0}],
        "sims":[{"name":"Ada","household":0,"x":3.5,"z":5.5}]}"#;

    #[test]
    fn only_what_walkers_step_through_moves_the_navigation() {
        let mut w = World::from_json(CONTENT, HOME_LOT, 7).unwrap();
        let (nav, structure) = (w.nav_version, w.structure_version);
        // A new colour changes how it looks, not where anyone can walk.
        w.recolor(0, 0, Colors::new()).unwrap();
        assert_eq!(w.nav_version, nav, "a recolour isn't a change to walking");
        assert_ne!(w.structure_version, structure, "but views must rebuild");
        // A wall is.
        let wall = crate::command::EdgeEdit::new(crate::command::EdgeAxis::V, 6, 0, crate::command::EdgeKind::Wall);
        w.build(0, &[wall]).unwrap();
        assert_eq!(w.nav_version, nav + 1);
        assert_eq!(w.structure_version, structure + 2);
    }

    #[test]
    fn what_they_couldnt_get_to_is_remembered_a_while() {
        let mut w = world();
        let s = &mut w.sims[0];
        s.remember_unreachable(TaskKind::Use { object: 3, interaction: 1 }, 100);
        s.remember_unreachable(TaskKind::Repair { object: 1 }, 100);
        s.remember_unreachable(TaskKind::Visit { plot: 2 }, 50);
        s.remember_unreachable(TaskKind::Clean { x: 4, z: 5 }, 200);
        // People are remembered too, for as short a while as the caller gives them.
        s.remember_unreachable(TaskKind::Social { target: 1, social: 0 }, 80);
        assert_eq!(s.unreachable_now(60).count(), 4);
        assert!(s.unreachable_now(60).any(|k| matches!(k, TaskKind::Social { target: 1, .. })));
        assert!(!s.unreachable_now(60).any(|k| matches!(k, TaskKind::Visit { .. })), "the shortest-lived made way");
        assert!(s.couldnt_reach(3, 60) && s.couldnt_reach(1, 99) && !s.couldnt_reach(1, 100) && !s.couldnt_reach(2, 0));
        // Saying the same thing again just lasts longer.
        s.remember_unreachable(TaskKind::Repair { object: 1 }, 300);
        assert!(s.couldnt_reach(1, 299));
        // Full: the one that runs out first makes way.
        s.remember_unreachable(TaskKind::Eat { item: Some(7), seat: None, plate: None, stage: 0 }, 300);
        assert!(!s.unreachable_now(0).any(|k| matches!(k, TaskKind::Social { .. })), "the shortest-lived made way");
        // Object 1 is gone and the ids after it shift down.
        s.remap_unreachable(|o| if o == 1 { None } else { Some(if o > 1 { o - 1 } else { o }) });
        assert!(s.couldnt_reach(2, 0) && !s.couldnt_reach(3, 0) && !s.couldnt_reach(1, 0));
        assert_eq!(s.unreachable_now(0).count(), 3);
    }
}
