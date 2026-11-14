//! World state and the per-tick update.

use std::collections::{BTreeMap, VecDeque};
use std::f32::consts::FRAC_PI_2;

use crate::clock::{AUTO_FAST_SPEED, MAX_SPEED, TICKS_PER_STEP};
use crate::content::{
    Content, HabitTime, Layer, MAX_NEEDS, MAX_SKILLS, MAX_SLOTS, Modifiers, Pose, TagMask, stage_in,
};
use crate::index::ObjectIndex;
use crate::lot::{BondRaw, Lot, LotFile, SimSpawn};
use crate::path::{Grounds, NavGrid};
use crate::rng::Rng;
use crate::social::{self, ActiveFeeling, EventLog, Prefer, Relationships};
use crate::{
    Command, Error, MINUTES_PER_TICK, TICKS_PER_MINUTE, TICKS_PER_SECOND, ai, conversation,
};
use serde::{Deserialize, Serialize};

/// The most residents a town holds (a town file may start with this many; saves, slots, the
/// snapshot's layout).
pub const MAX_SIMS: usize = 240;

/// A set of residents by slot (`MAX_SIMS` of them, a bit each).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SimSet([u64; SIM_WORDS]);

/// Words in a `SimSet`.
const SIM_WORDS: usize = MAX_SIMS.div_ceil(64);

impl SimSet {
    pub fn insert(&mut self, sim: u32) {
        if let Some(w) = self.0.get_mut(sim as usize / 64) {
            *w |= 1 << (sim % 64);
        }
    }

    pub fn remove(&mut self, sim: u32) {
        if let Some(w) = self.0.get_mut(sim as usize / 64) {
            *w &= !(1 << (sim % 64));
        }
    }

    pub fn contains(&self, sim: u32) -> bool {
        self.0
            .get(sim as usize / 64)
            .is_some_and(|w| w & (1 << (sim % 64)) != 0)
    }

    /// Whether anyone but `sim` is in it.
    pub fn any_but(&self, sim: u32) -> bool {
        let mut rest = *self;
        rest.remove(sim);
        rest.0.iter().any(|&w| w != 0)
    }

    /// The set's words, lowest residents first (for fingerprints).
    pub fn words(&self) -> &[u64] {
        &self.0
    }
}
pub const MAX_QUEUE: usize = 6;
/// How many things a resident remembers they couldn't get to (`Sim::unreachable`).
pub(crate) const UNREACHABLE: usize = 4;
/// How long a resident remembers that someone was out of reach: people move, so not long (half a
/// real second: as long as the one they wanted takes to walk a little way).
pub(crate) const PERSON_UNREACHABLE_TICKS: u64 = crate::secs(0.5);
/// Walking speed in tiles per tick (1.5 m per real second at 1×).
pub(crate) const WALK_SPEED: f32 = 1.5 / TICKS_PER_SECOND as f32;
/// Idle time before a Sim picks something on its own: a beat seen standing there, a real second
/// (counted in their steps, so a turn at lower detail covers it, `lod.rs`). Those listening in on a
/// talk wait longer for its next social (`conversation::REGROUP_TICKS`).
pub(crate) const AUTONOMY_DELAY_TICKS: u32 = crate::secs(1.0) as u32;
/// Getting up from the upper bunk: the time to climb down its ladder (the renderer's `bunkDown`, 6.5 s), in
/// which they do nothing else (`Sim::ladder_ticks`).
pub(crate) const LADDER_DOWN_TICKS: u32 = crate::secs(6.5) as u32;
/// Stepping aside out of a car's way (`cars::make_way`): at most this far in one step (m).
const SIDESTEP_MOST: f32 = 0.3;
const MIN_AUTONOMY_SCORE: f32 = 0.01;
/// How much less a meal nobody at home would eat tempts free will (the cooking practice it
/// gives aside): cooked only with nothing much else to do.
const UNWANTED_MEAL: f32 = 0.1;
/// A baby with a need below this cries for someone (and lies awake in the crib).
const BABY_CRIES_BELOW: f32 = 0.3;
pub const TICKS_PER_DAY: u64 = crate::clock::TICKS_PER_DAY;
const TICKS_PER_HOUR: u64 = 60 * TICKS_PER_MINUTE;
/// Rooms are re-scored this often (for dirt), and whenever the structure changes.
pub(crate) const ROOM_REFRESH_TICKS: u64 = 10 * TICKS_PER_MINUTE;
/// Past this many tiles where things changed between ticks, every room is scored again for what
/// stands in it (`World::rooms_things_changed`).
const MOST_THINGS_AT: usize = 64;
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
        Layout {
            offset,
            ..Layout::default()
        }
    }

    /// The layout `o` has now.
    pub fn of(o: &ObjectInstance) -> Self {
        Layout {
            offset: o.offset,
            extent: Some(o.extent),
            tall: o.tall,
        }
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
    /// A lamp (or a fire) switched off (see `lights.rs`); bought off, on with a lot or a house's
    /// file. Meaningless for other things.
    pub off: bool,
    /// A machine's run (its kind `runs`; see `machines.rs`): None empty, the game minutes left
    /// while it runs on its own, 0 done with the load still in it.
    pub run: Option<f32>,
    /// The dirty things washed up at it that wait in it to be washed (its kind's `rack`, a
    /// dishwasher; `machines::rack_in`): they're washed when it runs, put away when it's emptied.
    pub rack: u8,
    /// Residents whose current use needs it closed (bits by sim id), so opening it under them is
    /// refused (`allows`). Derived from their tasks: set when the use reserves a slot, cleared
    /// when it ends; not saved (a reloaded use starts again and sets it).
    pub(crate) closed_users: SimSet,
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
            && (!inter.loads || self.run.is_none())
            && (inter.work_on != Some(crate::content::WorkOn::Dishes) || self.run.is_none())
            && (!inter.unloads || self.done())
            && (inter.leaves != Some(false) || self.users().all(|u| u == me))
            && (inter.leaves != Some(true) || !self.closed_users.any_but(me))
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

    /// A machine running on its own (see `run`).
    pub fn running(&self) -> bool {
        self.run.is_some_and(|m| m > 0.0)
    }

    /// A machine done, its load still in it (see `run`).
    pub fn done(&self) -> bool {
        self.run.is_some_and(|m| m <= 0.0)
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
        self.reserve_near(sim, range, None, 0)
    }

    /// Takes the slot `pick_slot` gives `sim` in `range` (`near`, `prefer`: see there).
    fn reserve_near(
        &mut self,
        sim: u32,
        range: std::ops::Range<usize>,
        near: Option<(&crate::content::ObjectDef, [f32; 2])>,
        prefer: u32,
    ) -> bool {
        if self.slot_of(sim).is_some() {
            return true;
        }
        match self.pick_slot(sim, range, near, prefer) {
            Some(k) => {
                self.users[k] = Some(sim);
                true
            }
            None => false,
        }
    }

    /// The slot in `range` `sim` gets: theirs already, else a free one, one of `prefer`'s (a
    /// mask of slots: the sofa's seat with a footstool in front, `footstool_slots`) if one is
    /// free; of a kind whose slots have their places (`near`: its def, and where `sim` is now),
    /// the one nearest them, else the first.
    pub(crate) fn pick_slot(
        &self,
        sim: u32,
        range: std::ops::Range<usize>,
        near: Option<(&crate::content::ObjectDef, [f32; 2])>,
        prefer: u32,
    ) -> Option<usize> {
        if let Some(k) = self.slot_of(sim) {
            return Some(k);
        }
        let open = |k: &usize| self.users[*k].is_none();
        let preferred = |k: &usize| *k < 32 && prefer & (1 << *k) != 0;
        let some_preferred = range.clone().filter(open).any(|k| preferred(&k));
        let mut free = range
            .filter(open)
            .filter(|k| !some_preferred || preferred(k));
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
    pub(crate) fn reserve_for(
        &mut self,
        sim: u32,
        def: &crate::content::ObjectDef,
        i: usize,
        from: [f32; 2],
        prefer: u32,
    ) -> bool {
        let range = def.slots_for(i);
        let reserved = if def.interactions[i].whole {
            if !self.can_use_for(sim, def, i) {
                return false;
            }
            self.users[range].fill(Some(sim));
            true
        } else {
            self.reserve_near(sim, range, Some((def, from)), prefer)
        };
        self.kept_closed(sim, reserved, def.interactions[i].when == Some(false))
    }

    /// The slot `reserve_for` would give `sim` (at `from`) for interaction `i` of `def`.
    pub(crate) fn slot_for(
        &self,
        sim: u32,
        def: &crate::content::ObjectDef,
        i: usize,
        from: [f32; 2],
        prefer: u32,
    ) -> Option<usize> {
        let range = def.slots_for(i);
        if def.interactions[i].whole {
            return Some(range.start);
        }
        self.pick_slot(sim, range, Some((def, from)), prefer)
    }

    /// Takes a place on it for `sim` (now at `from`) to sit while doing something else (eating,
    /// the phone, TV from the sofa). A seat only while closed (a sofa bed) then can't be opened
    /// under them.
    pub(crate) fn reserve_seat(
        &mut self,
        sim: u32,
        def: &crate::content::ObjectDef,
        from: [f32; 2],
        prefer: u32,
    ) -> bool {
        let reserved = self.reserve_near(sim, 0..def.slots as usize, Some((def, from)), prefer);
        self.kept_closed(
            sim,
            reserved,
            def.interactions
                .iter()
                .any(|i| i.pose == Pose::Sit && i.when == Some(false)),
        )
    }

    /// Passes `reserved` on; a place taken for a use that needs it `closed` keeps it so under `sim`.
    fn kept_closed(&mut self, sim: u32, reserved: bool, closed: bool) -> bool {
        if reserved && closed {
            self.closed_users.insert(sim);
        }
        reserved
    }

    /// The tags of each way someone past the crib, in life stage `stage`, can sit on it as it is
    /// now (a sofa bed pulled out is no seat; a kids' chair is none for a grown-up).
    pub fn seats_now<'a>(
        &'a self,
        def: &'a crate::content::ObjectDef,
        stage: usize,
    ) -> impl Iterator<Item = TagMask> + 'a {
        def.interactions
            .iter()
            .filter(move |i| i.pose == Pose::Sit && !i.baby && self.fits(i) && i.for_stage(stage))
            .map(|i| i.tags)
    }

    /// Whether it is a seat now for someone in `stage` with a way to sit that has one of `want`'s
    /// tags and none of `avoid`'s.
    pub fn sits(
        &self,
        def: &crate::content::ObjectDef,
        stage: usize,
        want: TagMask,
        avoid: TagMask,
    ) -> bool {
        self.seats_now(def, stage)
            .any(|t| t & want != 0 && t & avoid == 0)
    }

    /// Takes on the state `old`, this object before it was lifted out and placed again, had: its
    /// stable id, turn, wear, open or closed, on or off, crop, and a machine's run.
    pub(crate) fn carry_state(&mut self, old: &ObjectInstance) {
        (
            self.stable,
            self.turn,
            self.wear,
            self.open,
            self.off,
            self.growth,
            self.run,
            self.rack,
        ) = (
            old.stable, old.turn, old.wear, old.open, old.off, old.growth, old.run, old.rack,
        );
    }

    pub(crate) fn release(&mut self, sim: u32) {
        self.closed_users.remove(sim);
        for u in &mut self.users {
            if *u == Some(sim) {
                *u = None;
            }
        }
    }

    /// Where a Sim sits, lies or stands in `slot`: at its place (content `places`), else spread
    /// across the object's width, in the middle of its depth (but a baby in the middle of a seat
    /// for one, a high chair, whose other places are the carers' beside it).
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
        if slot == 0
            && def.places.is_empty()
            && def.interactions.iter().any(|i| i.baby && !i.baby_bed())
        {
            return c;
        }
        let [x, z] = self.slot_local(def, slot);
        // Its place turns with it, a free turn too (the place is on its model); the spread across
        // its width only by its `rot`.
        let a = if slot < def.places.len() {
            self.angle()
        } else {
            self.yaw()
        };
        [
            c[0] + a.cos() * x + a.sin() * z,
            c[1] - a.sin() * x + a.cos() * z,
        ]
    }

    /// Where `slot`'s user is in the object's own frame, before its free turn: metres to its
    /// right and to its front of the middle of its tiles (`slot_position` without the offset).
    pub(crate) fn slot_local(&self, def: &crate::content::ObjectDef, slot: usize) -> [f32; 2] {
        // Its place, moved out with its size.
        if let Some(p) = def.places.get(slot) {
            return [0, 1]
                .map(|k| p.at[k] * self.extent[k] as f32 / 100.0 / def.footprint[k].max(1) as f32);
        }
        let n = def.slots.max(1) as f32;
        // Bunks lie one above the other: both in the middle (a hair apart, so the renderer can
        // tell the lower bunk from the upper one).
        let spread = if def.bunk {
            0.04
        } else {
            self.extent[0] as f32 / 100.0 / n
        };
        [(slot as f32 - (n - 1.0) / 2.0) * spread, 0.0]
    }

    /// The point `[right, front]` metres off the middle of its tiles in its own frame (by its
    /// `rot`, not its free turn or offset: where its tiles are).
    pub(crate) fn local_point(&self, at: [f32; 2]) -> [f32; 2] {
        let (w, d) = self.turned_footprint();
        let yaw = self.yaw();
        let c = [
            self.x as f32 + w as f32 / 2.0,
            self.z as f32 + d as f32 / 2.0,
        ];
        [
            c[0] + yaw.cos() * at[0] + yaw.sin() * at[1],
            c[1] - yaw.sin() * at[0] + yaw.cos() * at[1],
        ]
    }

    /// Which way a Sim in `slot` faces (`Sim::yaw`): its place's facing (content `places`), else
    /// the way the object faces.
    pub fn slot_yaw(&self, content: &Content, slot: usize) -> f32 {
        match content.objects[self.def].places.get(slot) {
            // Kept within a half turn either way, as every yaw is (a place's facing on a turned object can add up past one).
            Some(p) => wrap_angle(self.angle() + p.facing),
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

    /// The tiles it reaches over at head height beside its own (content `overhang`, turned with
    /// it): an arc lamp's shade over the sofa.
    pub fn overhang_tiles<'a>(
        &self,
        content: &'a Content,
    ) -> impl Iterator<Item = (i32, i32)> + 'a {
        // Its right and its front, as it's turned.
        let ((rx, rz), (fx, fz)) = match self.rot % 4 {
            0 => ((1, 0), (0, 1)),
            1 => ((0, -1), (1, 0)),
            2 => ((-1, 0), (0, -1)),
            _ => ((0, 1), (-1, 0)),
        };
        let (x, z) = (self.x, self.z);
        content.objects[self.def]
            .overhang
            .iter()
            .map(move |&(r, f)| (x + r * rx + f * fx, z + r * rz + f * fz))
    }

    /// Pairs of neighbouring tiles the arm to each `overhang_tiles` crosses (along the row, then
    /// the column): no wall may stand between them.
    pub(crate) fn overhang_edges(&self, content: &Content) -> Vec<((i32, i32), (i32, i32))> {
        let mut edges = Vec::new();
        for to in self.overhang_tiles(content) {
            let mut at = (self.x, self.z);
            while at != to {
                let next = if at.0 != to.0 {
                    (at.0 + (to.0 - at.0).signum(), at.1)
                } else {
                    (at.0, at.1 + (to.1 - at.1).signum())
                };
                edges.push((at, next));
                at = next;
            }
        }
        edges
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
        let [ex, ez] = if self.rot.is_multiple_of(2) {
            self.extent
        } else {
            [self.extent[1], self.extent[0]]
        };
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
        let (xs, zs) = (
            x0.div_euclid(200)..(x1 + 199).div_euclid(200),
            z0.div_euclid(200)..(z1 + 199).div_euclid(200),
        );
        zs.flat_map(move |tz| xs.clone().map(move |tx| (tx, tz)))
    }

    /// Whether a diagonal wall across tile `(tx, tz)` would run through its body (touching its
    /// side or corner is fine, as a wall on a tile line along its side).
    pub(crate) fn body_crosses(
        &self,
        content: &Content,
        tx: i32,
        tz: i32,
        dir: crate::lot::DiagDir,
    ) -> bool {
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
    /// footprint, and from the footprint to the front tile (not for a seat behind the work,
    /// `Approach::Behind`: a desk may stand with its screen to a wall).
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
        let behind = content.objects[self.def].approach == crate::content::Approach::Behind;
        inside.chain((!behind).then_some(((bx, bz), front)))
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
    /// Carrying something to `to` and putting it there (`carry.rs`): a baby (`baby`), first over
    /// to them in `from` and lifting them out (stage 0), then carrying them to `to` and putting
    /// them in (stage 1: the high chair to be fed, or back to a crib); or, `baby` none, what's in
    /// their hands (a harvest) to the fridge `to` (stage 1; `from` is `to`).
    Carry {
        baby: Option<u32>,
        from: u32,
        to: u32,
        stage: u8,
    },
    /// Setting up something new that was bought (`setup.rs`; `object` its type): over to its
    /// parcel at the front door (`parcel`, standing at `door`) and picking it up (stage 0), then
    /// carrying it to `stand`, beside where it goes (`at`, its middle), and putting it together
    /// there (stage 1).
    SetUp {
        object: u32,
        stage: u8,
        door: [f32; 2],
        parcel: [f32; 2],
        stand: (i32, i32),
        at: [f32; 2],
    },
    /// A host bringing `guest` something to eat or drink (`hosting.rs`): over to `from` to take
    /// it out (its `interaction`, an `eat` one; stage 0), then with it in hand to the guest and
    /// handing it over (stage 1 on; each stage past 1 is one more go at getting to them).
    Serve {
        guest: u32,
        from: u32,
        interaction: usize,
        stage: u8,
    },
}

impl TaskKind {
    /// Whether whoever does it keeps what's in their hands (the rest put it down): cooking,
    /// eating, washing up (`cooking::chain`), a harvest carried to the fridge (`carry.rs`), a
    /// parcel carried in to be set up (`setup.rs`) and a bite brought to a guest (`hosting.rs`).
    pub fn carries_held(&self) -> bool {
        matches!(
            self,
            TaskKind::Cook { .. }
                | TaskKind::Eat { .. }
                | TaskKind::Dishes { .. }
                | TaskKind::Carry { baby: None, .. }
                | TaskKind::SetUp { .. }
                | TaskKind::Serve { .. }
        )
    }
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
    /// At the door of the car they go to work in (`Sim::bound_for_car`), getting in: it opens,
    /// they sit in, it shuts (`cars::getting`).
    Boarding {
        ticks: u32,
    },
    /// Back home in the car, getting out on their way in (a `GoHome` task): sat in it as its door
    /// opens, then out beside it (`cars::getting`).
    Alighting {
        ticks: u32,
    },
    /// At the wheel of their car on a trip to another hamlet or back (`cars::Trip`, a `Visit`
    /// or `GoHome` task): the car drives (`cars::update`), they sit in it.
    Driving,
    /// At their kerb on their way to work or school, waiting for the work taxi they're booked on
    /// (`Sim::taxi`, `taxi.rs`): it calls them to its door.
    AtKerb {
        ticks: u32,
    },
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub task: Task,
    pub phase: Phase,
    /// Tags of the interaction or social being performed (for "busy" checks and autonomy).
    pub tags: TagMask,
}

/// How many outfits each resident has to change between (`Sim::outfit`).
pub const OUTFITS: u8 = 3;

#[derive(Debug, Clone)]
pub struct Sim {
    /// Equal to the Sim's index in `World::sims`.
    pub id: u32,
    pub name: String,
    /// Opaque appearance data for the renderer/UI.
    pub appearance: serde_json::Value,
    /// Which of their `OUTFITS` they're dressed in (0: the one in `appearance`, as made in the
    /// creator; the renderer makes the others from the id): the next one once they change at
    /// a wardrobe (`Interaction::changes_outfit`), kept until they change again.
    pub outfit: u8,
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
    /// What they took up to work on this tick (`WorkOn` bits: a load of washing), settled with
    /// the household by `life::update`.
    pub(crate) worked: u8,
    /// Loads of washing they left to work on next this tick (`WorkOn` bits: the wet washing out
    /// of the washer; `Interaction::makes`), settled with `worked`.
    pub(crate) made: u8,
    /// The load of washing taken up for the use they're at (`WorkOn` bits): given back if the use
    /// doesn't get done (cancelled, called away: `end_activity`), none once it is
    /// (`machines::use_done`). Not saved: a save gives it back (a reloaded use starts again).
    pub(crate) taken: u8,
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
    /// Sitting, what they sat down to do, held while they talk from the seat (a chat with the
    /// one beside them on the sofa, listening in at the table; `conversation::from_seat`): they
    /// carry on with it when the talk is over, still sitting (`end_activity`). Not saved: after
    /// a reload they stand up where they sat down from.
    pub(crate) paused: Option<Activity>,
    /// Joining in: the place in the group they're walking to (`conversation::take_place`). Not
    /// saved, as listening in isn't.
    pub(crate) join_spot: Option<[f32; 2]>,
    /// Ticks without anything to do (to choose something); sitting at something that leaves
    /// room for a word, ticks since they last thought of one (`from_seat`).
    pub(crate) idle_ticks: u32,
    /// What they do next, before choosing anything else (an `after` routine, like washing
    /// hands after the toilet; see `follow_up`). Not saved: losing it on a reload is harmless.
    pub(crate) follow_up: Option<Task>,
    /// The object they fetched what a use there needs for (`Interaction::fetch`: the coffee
    /// poured at the machine for the table), so that use, the `follow_up`, goes ahead without
    /// fetching it again. Any other task started clears it. Not saved.
    pub(crate) fetched: Option<u32>,
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
    /// Open-world towns: how long the way to work or school takes them today, firmed up shortly
    /// before they set off (`life::refresh_commute`); None: as `careerRules.commuteMinutes`
    /// says. Not saved.
    pub(crate) commute: Option<crate::life::Commute>,
    /// The trailhead (object id) whose trail they last walked (`trail_step`); they're walking it
    /// while they use it (`on_trail`). Not saved.
    pub(crate) on_trail: Option<u32>,
    /// The game for two (`Interaction::partner`) they're at, or on their way to, next: its
    /// object and interaction, and whether they're at their place. Set as each tick starts
    /// (`partners::refresh`), not saved (the task it comes from is).
    pub(crate) game: Option<(u32, usize, bool)>,
    /// They set off for a game for two this tick: someone to play with is to be found
    /// (`partners::update`). Not saved.
    pub(crate) wants_partner: bool,
    /// Fetched to play with this resident (`partners::update`): the game they're sent to is
    /// that one's, joined, never one of their own. Not saved.
    pub(crate) joins: Option<u32>,
    /// Ticks they've waited at their place for the one they play with to get to theirs (see
    /// `partners::hold`; 0: not waiting). Not saved.
    pub(crate) partner_wait: u32,
    /// At a post's front, being served by its server now (set as each tick starts,
    /// `staff::update`). Not saved.
    pub(crate) being_served: bool,
    /// Ticks they've waited at a post for its server (`staff::hold`). Not saved.
    pub(crate) serve_wait: u32,
    /// The bike (or e-scooter) they're on their way to for work or school, and the one they're
    /// on (`ride.rs`): an object at home, its slot theirs until they're back. Not saved (after a
    /// load it's at its place and they walk).
    pub(crate) bound_for_bike: Option<u32>,
    pub riding: Option<u32>,
    /// Out on a visit or an outing by bike: where they left it by the place (`world::arrive`), the
    /// bike theirs in `bound_for_bike` till they ride it home. Not saved.
    pub(crate) bike_parked: Option<[f32; 2]>,
    /// Where that bike stands by the place (`ride::park_spot`: its middle, which way it faces);
    /// None while it's not worked out yet, or there was no room (then where they got off). In the
    /// snapshot for the renderer. Not saved.
    pub(crate) bike_spot: Option<([f32; 2], f32)>,
    /// When they were last back home from a visit or an outing (`visits.againMinutes`). Not saved.
    pub(crate) back_at: u64,
    /// The car they're on their way to for work or school (`cars::ride_for`), their seat in it (0:
    /// they drive) and the door they get in by (`cars::DOORS`). Not saved (after a load, they set
    /// off again).
    pub(crate) bound_for_car: Option<u32>,
    /// The work taxi they're booked on (`taxi::Booking`): to work or school, or home from it. Not
    /// saved (the runs are: `taxi::settle`).
    pub(crate) taxi: Option<crate::taxi::Booking>,
    pub(crate) car_seat: u8,
    pub(crate) car_door: u8,
    /// Ticks they've waited for a car driving by to pass (`cars::in_the_way`). Not saved.
    pub(crate) car_wait: u32,
    /// The lit junction whose crosswalk they wait at the kerb of, the arm it's on (for its walk
    /// phase, or its main road's green: `Signal::opens_with`), and the tick they last did
    /// (`traffic::kerb_stop`). Not saved.
    pub(crate) kerb_wait: Option<(u16, u8, u64)>,
    /// The work shift (or school day) they're away for starts at this tick: until then they're on
    /// their way (`life::at_work`). Not saved (after a load, it's under way).
    pub(crate) shift_from: u64,
    /// Found nobody to play a game for two with they chose: none chosen again until this tick
    /// (`partners::alone`). Not saved.
    pub(crate) alone_until: u64,
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
    /// Up from the upper bunk: the ticks left climbing down its ladder, doing nothing else meanwhile
    /// (`LADDER_DOWN_TICKS`). Not saved: a load finds them down already.
    pub(crate) ladder_ticks: u32,
    /// The first day this Sim looks for work again (after quitting or being let go).
    pub job_search_from: u32,
    /// Routines, goals and how planned blocks went (see `planner`).
    pub planner: crate::planner::Planner,
    /// Mess made (or, negative, cleaned) around a tile this tick; applied after the step
    /// (residents can't change the dirt while stepping).
    pub(crate) mess: Option<(i32, i32, f32)>,
    /// Conceived with this partner this tick (the life cycle decides; see `lifecycle::conceive`).
    pub(crate) conceived: Option<u32>,
    /// Their *Try for a baby* with this partner went well this tick: the two go to bed
    /// (`privacy::after_step`). Not saved.
    pub(crate) to_bed: Option<u32>,
    /// Ticks they've stood outside a room kept shut by someone in it, on their way in (a
    /// bathroom in use; `privacy.rs`), 0 while not. Not saved: after a load they set off again.
    pub(crate) kept_out: u32,
    /// Care given this tick: the baby it's for and what it fills (applied after the step).
    pub(crate) care_out: Option<(u32, [f32; MAX_NEEDS])>,
    /// A baby lifted out of their place or put in one this tick (applied after the step,
    /// `carry::after_step`).
    pub(crate) carry_out: Option<crate::carry::CarryOut>,
    /// A baby in someone's arms: who carries them (`carry.rs`). In no place meanwhile, they go
    /// where the one carrying them goes. Not saved (after a load they're back in the crib).
    pub carried_by: Option<u32>,
    /// Opened or closed an object this tick (`Interaction::leaves`), or changed clothes
    /// (`Interaction::changes_outfit`): how the house or they look changed (applied after the
    /// step).
    pub(crate) reshaped: bool,
    /// Lit a fire this tick (sat down by one in the dark: see `lights.rs`): the lamps' state
    /// changed (applied after the step).
    pub(crate) lit: bool,
    /// When they last woke someone crying (a baby), or were woken by a crying baby (a
    /// grown-up), as a tick (0: never; see `World::wake_for_crying_babies`).
    pub(crate) night_woken: u64,
    /// An object they broke or repaired, or an accident they had, this tick (applied after
    /// the step).
    pub(crate) news: Option<News>,
    /// Finished putting together something new this tick (an object type; `setup.rs` places it
    /// after the step). Not saved.
    pub(crate) set_up: Option<u32>,
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
    /// Their offer of something to eat or drink was taken up this tick by this resident: they go
    /// and fetch it (`hosting::update`). Not saved.
    pub(crate) serve_for: Option<u32>,
    /// Handed what they hold to this resident this tick (`hosting::update` passes it over). Not
    /// saved.
    pub(crate) hand_to: Option<u32>,
    /// Said yes to a host's offer: something to eat is on its way to them until this tick (or
    /// till it's handed over), so they don't get food of their own meanwhile (`hosting.rs`). Not
    /// saved.
    pub(crate) expecting: u64,
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
    /// Their turn last tick waited for the next, the tick's searches being over budget
    /// (`SEARCH_BUDGET`): this tick they go whatever it costs (`lod::plan` gives them a turn).
    /// Not saved.
    pub(crate) deferred: bool,
}

impl Sim {
    pub fn tile(&self) -> (i32, i32) {
        (self.pos[0].floor() as i32, self.pos[1].floor() as i32)
    }

    /// Walking: on their way somewhere, and not waiting outside a room kept shut (`kept_out`).
    pub fn is_moving(&self) -> bool {
        self.kept_out == 0
            && (matches!(
                self.current,
                Some(Activity {
                    phase: Phase::Routing { .. },
                    ..
                })
            ) || self.on_trail())
    }

    /// Walking a trail (`Content::trail_walks`): shown walking though it's a use.
    pub fn on_trail(&self) -> bool {
        self.on_trail.is_some()
            && matches!(
                self.current,
                Some(Activity {
                    phase: Phase::Using { .. },
                    task: Task {
                        kind: TaskKind::Use { object, .. },
                        ..
                    },
                    ..
                }) if Some(object) == self.on_trail
            )
    }

    /// Waiting outside a room kept shut by someone in it (a bathroom in use; `privacy.rs`).
    pub fn kept_out(&self) -> bool {
        self.kept_out > 0
    }

    pub fn current(&self) -> Option<&Activity> {
        self.current.as_ref()
    }

    /// How long their way to work or school takes today in an open-world town (`life::Commute`).
    pub fn commute(&self) -> Option<crate::life::Commute> {
        self.commute
    }

    /// Eating sitting down (at the table, on the sofa), not talking.
    pub(crate) fn eating_seated(&self) -> bool {
        self.pose == Pose::Sit
            && self.current.as_ref().is_some_and(|a| {
                matches!(
                    (a.task.kind, &a.phase),
                    (TaskKind::Eat { .. }, Phase::Using { .. })
                )
            })
    }

    /// Sitting, what they sat down to do while they talk from the seat (`paused`).
    pub fn paused(&self) -> Option<&Activity> {
        self.paused.as_ref()
    }

    /// The bike (or e-scooter) they're on, or on their way to for work or school (`ride.rs`): its
    /// slot is theirs until they're back.
    pub fn bike(&self) -> Option<u32> {
        self.riding.or(self.bound_for_bike)
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
        let crying = (0..content.needs.len())
            .any(|n| !content.needs[n].room && self.needs[n] < BABY_CRIES_BELOW);
        let rested = rhythm.rest_need.is_none_or(|n| self.needs[n] >= 0.999);
        crying || (rested && !rhythm.is_night(hour, rhythm.wake_hour))
    }

    /// Stops whatever they're doing and drops what they meant to do (their object slots are
    /// released by the caller).
    pub(crate) fn clear_activity(&mut self) {
        self.current = None;
        self.queue.clear();
        self.follow_up = None;
        self.fetched = None;
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

    /// The car they're on their way to (or in, on a trip), if any (`bound_for_car`).
    pub fn car_bound(&self) -> Option<u32> {
        self.bound_for_car
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

    /// How fast they walk, as a share of `WALK_SPEED`: their traits' and life stage's (`mods`)
    /// times their emotion's (content `emotions` `walkSpeed`: slower sad, quicker angry or
    /// energised; the renderer's body language shows the rest).
    pub fn walk_speed(&self, content: &Content) -> f32 {
        self.mods.walk_speed
            * self
                .emotion(content)
                .map_or(1.0, |e| content.emotions[e].mods.walk_speed)
    }

    /// How many metres they walk in a minute (`walk_speed`), as `travel::Travel::eta` asks.
    pub(crate) fn walk_per_minute(&self, content: &Content) -> f32 {
        WALK_SPEED * self.walk_speed(content) * TICKS_PER_MINUTE as f32
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
        // (Only the needs in the mask looked at: asked of everyone every tick, `SimBrief`.)
        let mut essentials = rules.urgent & !rules.social_fills;
        while essentials != 0 {
            let n = essentials.trailing_zeros() as usize;
            essentials &= essentials - 1;
            if n >= content.needs.len() {
                break;
            }
            if !content.needs[n].room && self.needs[n] < rules.urgent_below {
                return true;
            }
        }
        false
    }

    /// Whether another Sim can pull this one into a conversation right now.
    pub fn available_for_social(&self, content: &Content) -> bool {
        // (Nor someone at a game for two, or on their way to one: `partners.rs`.)
        if self.engaged_with.is_some()
            || self.heeding.is_some()
            || self.pose == Pose::Lie
            || self.away_until.is_some()
            || self.pressed(content)
            || self.game.is_some()
        {
            return false;
        }
        // Busy with a planned block (unless it's for socialising).
        if let Some(run) = &self.planner.run
            && run.skipped.is_none()
            && !run.no_place
            && !content
                .activities
                .get(run.activity)
                .is_some_and(|a| a.social || a.visit)
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
            // cooks halfway through a recipe or people eating or washing up. A cook on their way
            // to the first station (nothing bought yet, nothing in hand) does.
            (
                TaskKind::Work
                | TaskKind::Visit { .. }
                | TaskKind::GoHome
                | TaskKind::Answer { .. }
                | TaskKind::Serve { .. },
                _,
            ) => false,
            (
                TaskKind::Cook {
                    step: 0,
                    servings: 0,
                    ..
                },
                Phase::Routing { .. },
            ) if self.held.is_none() => true,
            // Eating sitting down: a word over the meal, from their seat (`conversation::keeps_seat`).
            (TaskKind::Eat { .. }, Phase::Using { .. }) if self.pose == Pose::Sit => true,
            (TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. }, _) => false,
            (TaskKind::Social { .. } | TaskKind::Join { .. }, Phase::Conversing { .. }) => false,
            // A guest on their way to say goodbye (`visits.farewell`) is leaving.
            (TaskKind::Social { social, .. }, _) if content.visits.farewell.contains(&social) => {
                false
            }
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
            Some(Activity {
                task:
                    Task {
                        kind: TaskKind::Join { leader },
                        ..
                    },
                phase: Phase::Conversing { .. },
                ..
            }) => Some(*leader),
            _ => None,
        }
    }

    /// What they couldn't get to a moment ago and won't choose again yet (`unreachable`).
    pub(crate) fn unreachable_now(&self, tick: u64) -> impl Iterator<Item = TaskKind> + '_ {
        self.unreachable
            .iter()
            .flatten()
            .filter(move |u| tick < u.1)
            .map(|u| u.0)
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
        let same = self
            .unreachable
            .iter()
            .position(|u| u.is_some_and(|u| same_target(u.0, kind)));
        let slot = same.unwrap_or_else(|| {
            (0..UNREACHABLE)
                .min_by_key(|&k| self.unreachable[k].map_or(0, |u| u.1))
                .unwrap_or(0)
        });
        self.unreachable[slot] = Some((kind, until));
    }

    /// Objects moved (ids shifted, one gone): `remap` gives each object id in `unreachable` its
    /// new id, or None for one that's gone (forgotten then).
    pub(crate) fn remap_unreachable(&mut self, remap: impl Fn(u32) -> Option<u32>) {
        for slot in &mut self.unreachable {
            let Some((TaskKind::Use { object, .. } | TaskKind::Repair { object }, _)) = slot else {
                continue;
            };
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
        (
            TaskKind::Use {
                object,
                interaction,
            },
            TaskKind::Use {
                object: o,
                interaction: i,
            },
        ) => (object, interaction) == (o, i),
        (TaskKind::Eat { item, .. }, TaskKind::Eat { item: it, .. }) => item == it,
        (TaskKind::Clean { x, z }, TaskKind::Clean { x: cx, z: cz }) => (x, z) == (cx, cz),
        (TaskKind::Repair { object }, TaskKind::Repair { object: o }) => object == o,
        (TaskKind::Dishes { item, .. }, TaskKind::Dishes { item: it, .. }) => item == it,
        (TaskKind::Visit { plot }, TaskKind::Visit { plot: p }) => plot == p,
        (
            TaskKind::Social { target, social },
            TaskKind::Social {
                target: t,
                social: s,
            },
        ) => (target, social) == (t, s),
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
    /// Loads of washing waiting: dirty (piling up as content `laundry` says), washed, to dry,
    /// and dried, to fold (`content::WorkOn::load`).
    pub laundry: [f32; 3],
}

/// Loads of washing a new household has waiting, dirty and washed (one of each: there's always
/// a bag of washing on moving in); none dry yet.
pub const LAUNDRY_AT_START: f32 = 1.0;

/// A new household's washing waiting (`Household::laundry`).
pub const LAUNDRY_START: [f32; 3] = [LAUNDRY_AT_START, LAUNDRY_AT_START, 0.0];

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
    /// Sitting down (on a sofa, at a table): someone sitting near can talk to them from their seat.
    pub seated: bool,
    /// Lying down (asleep in bed, napping on the sofa): nobody to answer the door.
    pub lying: bool,
    /// Eating sitting down: talked to only from close by in their seat (not a hug), the meal
    /// left a moment (`conversation::keeps_seat`).
    pub eating: bool,
    /// The game for two they're at or on their way to (`Sim::game`).
    pub game: Option<(u32, usize, bool)>,
    /// Free to be fetched for one (`partners::free_now`).
    pub free: bool,
    /// On their way to this car for work or school (`Sim::bound_for_car`).
    pub bound_for_car: Option<u32>,
    /// On their way to this car for a trip, or in it (`cars::Trip`: a `Visit` or `GoHome`).
    pub trip_car: Option<u32>,
    /// The bike their way to work or school is timed by (`life::Commute::bike`): not a
    /// housemate's to take (`ride::bike_for`).
    pub bike: Option<u32>,
    /// Where their bike stands left out by a place (`Sim::bike_spot`): not another's to leave there.
    pub bike_spot: Option<[f32; 2]>,
    /// The plot they're a guest on (`Sim::visiting`).
    pub guest_at: Option<u32>,
    /// The seat they eat at, or are on their way to with something to eat (`cooking::company`).
    pub eats_at: Option<u32>,
}

/// `World::broadcast`'s per-tick buffers, kept from tick to tick like `briefs`.
#[derive(Default)]
struct BroadcastScratch {
    /// The things residents are using that broadcast to others.
    sources: Vec<BroadcastSource>,
    /// Per Sim: the room they're in, their storey, their plot, and whether they're asleep.
    rooms: Vec<u16>,
    storeys: Vec<i32>,
    plots: Vec<Option<u32>>,
    asleep: Vec<bool>,
    /// The objects one Sim has already heard this tick.
    heard: Vec<u32>,
}

/// A `Use` that broadcasts to residents nearby (`World::broadcast`).
struct BroadcastSource {
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
    /// Things bought that wait to be set up at home (see `setup.rs`).
    pub setups: Vec<crate::setup::Setup>,
    /// Plot index per object (`None` when outside every plot).
    pub(crate) object_plot: Vec<Option<u32>>,
    /// Objects by plot, for decisions (see `index.rs`); kept up to date by `refresh_index`.
    pub(crate) object_index: ObjectIndex,
    pub sims: Vec<Sim>,
    pub households: Vec<Household>,
    pub plots: Vec<Plot>,
    /// Plot per ground tile, row by row (see `map_plots`; `path::NO_PLOT` off the plots).
    pub(crate) tile_plot: Vec<u16>,
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
    /// The last changes to walking and where they were (`nav_changed_in`), so walkers far from
    /// them keep their way.
    pub(crate) nav_log: crate::path::NavLog,
    /// `path::nodes_searched` as this tick began: what its searches cost so far is the
    /// difference (`over_search_budget`). Not saved.
    pub(crate) search_mark: u64,
    /// The tiles a tick's searches may expand before turns that may search wait for the next
    /// (`SEARCH_BUDGET`; open-world towns only). Not saved.
    pub search_budget: u64,
    /// Bumped when objects come, go or move (or walls change), not when only a car parks: the
    /// objects-by-plot index, the links and the staff plan follow it.
    pub(crate) objects_version: u32,
    /// The walkable regions of the lot at `reach_version` (see `path::Reach`).
    pub(crate) reach: crate::path::Reach,
    pub(crate) reach_version: u32,
    /// Bumped when the lot itself changes (walls, doors, windows, fences, coverings, floors),
    /// not when only objects, residents or households do: views can then leave the lot out.
    pub(crate) lot_version: u32,
    pub(crate) briefs: Vec<SimBrief>,
    /// Rooms kept shut as the tick's step started (`privacy::refresh`), kept from tick to tick
    /// like `briefs`.
    pub(crate) shut: Vec<crate::privacy::Shut>,
    /// The rooms kept shut while anyone is in them (`privacy::private_room`), sorted; found
    /// with the rooms (`refresh_rooms`).
    pub(crate) private_rooms: Vec<u16>,
    /// Among those, the rooms of a claimed kind (`roomKinds[].claimed`: bedrooms), whose door its
    /// own people (`privacy::Shut::claimants`) have the run of, sorted; found with the rooms.
    pub(crate) claimed_rooms: Vec<u16>,
    /// The objects that can offer a place in each of those rooms (`privacy::claimed_places`),
    /// broken ones included, found with them, so a tick needn't look through every object in the
    /// home.
    pub(crate) claimed_places: Vec<(u16, Vec<u32>)>,
    /// The people who have a place in each claimed room, with the fingerprint they were found
    /// for (`privacy::refresh`), so a tick needn't look for them again while nothing changes.
    pub(crate) claimants_cache: Vec<(u16, u64, Vec<u32>)>,
    /// What each household has to work on as the tick's step started (`work_waiting`), kept
    /// from tick to tick like `briefs`.
    work: Vec<u8>,
    /// Who goes first when the tick's searches run over budget (`tick_once`), kept from tick to
    /// tick like `briefs`.
    first: Vec<bool>,
    /// `broadcast`'s per-tick buffers, kept from tick to tick like `briefs`.
    broadcast_scratch: BroadcastScratch,
    /// Where Sims leave town for work.
    pub exits: Vec<[f32; 2]>,
    /// The road network and the town file's other open-world fields (`travel.rs`).
    pub travel: crate::travel::Travel,
    /// Households' cars (see `cars.rs`), and every house's driveway (where a car goes when its
    /// household moves).
    pub cars: Vec<crate::cars::Car>,
    /// The claims cars hold and ask for where their ways cross or meet
    /// (`traffic::Traffic`). Not saved.
    pub traffic: crate::traffic::Traffic,
    pub drives: Vec<crate::cars::Drive>,
    /// The work taxis' runs, planned and under way (`taxi.rs`).
    pub taxis: crate::taxi::Taxis,
    /// Garages at the inner ends of driveways (`cars::Garage`).
    pub garages: Vec<crate::cars::Garage>,
    /// Porches over doors, and porch pieces standing on their own (`porches.rs`).
    pub porches: Vec<crate::porches::Porch>,
    pub porch_pieces: Vec<crate::porches::PorchPiece>,
    /// The tiles parked cars take up in `blocked` (`cars::reblock`), sorted.
    pub(crate) car_tiles: Vec<usize>,
    /// The `nav_version` the cars' parking spots were last checked at (`cars::update`).
    pub(crate) cars_nav: u32,
    /// Which cars (a bit each, 64 to a word) stood parked when `car_tiles` was worked out; None:
    /// work it out again (`cars::reblock`).
    pub(crate) cars_parked: Option<Vec<u64>>,
    /// Cars by where they are, so a check that pairs one with another need only look at those
    /// near it (`cars::Near`; built each tick, `cars::update`).
    pub(crate) cars_near: crate::cars::Near,
    /// Opaque presentation data from the lot/town file, saved unchanged.
    pub meta: serde_json::Value,
    /// The town's paths and driveways (`meta.paths`) as rects (`x0, z0, x1, z1`): where a bike left
    /// out by a place doesn't stand (`ride::park_spot`). Worked out with `meta` (`note_paths`).
    pub(crate) paths: Vec<[f32; 4]>,
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
    /// Things came, went, broke, were mended or upgraded since the rooms were scored
    /// (`rooms_things_changed`).
    pub(crate) rooms_things: bool,
    /// The tiles (`(x, z)`) where things changed (empty with `rooms_things_all`).
    pub(crate) rooms_things_at: Vec<(i32, i32)>,
    /// Things may have changed anywhere: every room is scored again for them.
    pub(crate) rooms_things_all: bool,
    /// Per room id: it has a door to the outside (`privacy::doors_out`, found with the rooms).
    pub(crate) rooms_out: Vec<bool>,
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
    /// What the renderer draws (`set_watch`, for the cars in sight: `cars::sight`); None: all of
    /// it. Not saved.
    pub(crate) watch: Option<[i32; 4]>,
    /// Per plot: watched this tick (see `lod::plan`).
    pub(crate) watched: Vec<bool>,
    /// Bumped when a lamp is switched on or off (see `lights.rs`); not a structure change.
    pub(crate) lights_version: u32,
    /// What residents switching lights saw last (see `lights::update`).
    pub(crate) light_watch: crate::lights::Watch,
    /// The lights that follow a use (content `light.while`) that are lit now (object ids): a cooker
    /// hood's light while the hob under it cooks (see `lights::update_while`). Not saved.
    pub(crate) while_lit: Vec<u32>,
    /// Who serves at the town's stalls and counters (see `staff.rs`); not saved.
    pub staff: crate::staff::Staff,
    /// The walk out of town last timed from where each resident was (`life::firm_leave`). Not
    /// saved: a memo of what it's worked out from.
    pub(crate) walk_memo: crate::life::WalkMemo,
}

/// Read-only context for one Sim's update.
pub(crate) struct Ctx<'a> {
    pub content: &'a Content,
    pub nav: NavGrid<'a>,
    /// Which tiles can reach which (see `path::Reach`): a goal elsewhere is refused at once.
    pub reach: &'a crate::path::Reach,
    /// Bumped when the house changes in a way that can cut a walker's route short.
    pub nav_version: u32,
    /// Where those changes were (`World::nav_log`).
    pub nav_log: &'a crate::path::NavLog,
    pub rels: &'a Relationships,
    pub briefs: &'a [SimBrief],
    pub households: &'a [Household],
    pub plots: &'a [Plot],
    /// Plot per ground tile (`World::tile_plot`).
    pub tile_plot: &'a [u16],
    pub object_plot: &'a [Option<u32>],
    /// Objects by plot (`World::object_index`).
    pub index: &'a ObjectIndex,
    pub exits: &'a [[f32; 2]],
    /// The road network (`World::travel`).
    pub travel: &'a crate::travel::Travel,
    /// Households' cars and the work taxis (`World::cars`).
    pub cars: &'a [crate::cars::Car],
    /// The lit junctions' lights (`Traffic::lights`): their crosswalks open on the walk phase.
    pub lights: &'a [crate::traffic::Light],
    /// The work taxis' runs (`World::taxis`).
    pub taxis: &'a crate::taxi::Taxis,
    pub autonomy: bool,
    /// Room score per tile (`World::room_score`).
    pub room_score: &'a [f32],
    /// Dirt per tile (`World::dirt`).
    pub dirt: &'a [f32],
    pub hour: f32,
    pub day: u32,
    pub minute: f32,
    pub tick: u64,
    /// What each household has to work on (`work_waiting`), by household.
    pub work: &'a [u8],
    /// Rooms kept shut now, and by whom (`privacy::refresh`).
    pub shut: &'a [crate::privacy::Shut],
    /// Every room and garden with its scores (`World::rooms`).
    pub rooms: &'a [crate::rooms::RoomInfo],
    /// Who serves at the town's stalls and counters (`World::staff`).
    pub staff: &'a crate::staff::Staff,
    /// The town's paths and driveways (`World::paths`).
    pub paths: &'a [[f32; 4]],
    /// The other residents, as far along as they are (`Others`).
    pub others: Others<'a>,
}

/// The residents but the one whose update it is (each is stepped with the rest borrowed apart):
/// what another's own plans are worked out from, only when asked for (`hosts_about_till`; worked
/// out for everyone each tick, it was a cost of every step). Outside the residents' updates, all
/// of them (`World::ctx`).
#[derive(Clone, Copy, Default)]
pub(crate) struct Others<'a> {
    before: &'a [Sim],
    after: &'a [Sim],
}

impl<'a> Others<'a> {
    /// Every resident.
    pub(crate) fn all(sims: &'a [Sim]) -> Self {
        Others {
            before: sims,
            after: &[],
        }
    }

    /// Resident `j`; None for the one whose update it is (or one there isn't).
    pub(crate) fn get(&self, j: usize) -> Option<&'a Sim> {
        let n = self.before.len();
        match j.cmp(&n) {
            std::cmp::Ordering::Less => self.before.get(j),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => self.after.get(j - n - 1),
        }
    }
}

impl Ctx<'_> {
    /// Whether someone serves at object `object`: a sold use's minutes are then the ordering, the
    /// hand-over and the paying (`Interaction::minutes_where`).
    pub(crate) fn served(&self, object: u32) -> bool {
        self.staff.server_of(object).is_some()
    }

    /// Score of the room or garden at a point (`roomRules.away` off the plots and off the lot).
    pub(crate) fn room_at(&self, pos: [f32; 2]) -> f32 {
        let (x, z) = (pos[0].floor() as i32, pos[1].floor() as i32);
        let lot = self.nav.lot;
        if !lot.in_bounds(x, z) {
            return self.content.room_rules.away;
        }
        self.room_score
            .get(lot.tile_index(x, z))
            .copied()
            .unwrap_or(self.content.room_rules.away)
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
    /// Open-world towns: off to a public place out of reach (`autonomy.reach`), a countryside
    /// spot, the park in the next hamlet; they go as visitors (`TaskKind::Visit`).
    Outing(u32),
    /// A phone action, with someone or not (`phone.rs`).
    Phone(usize, Option<u32>),
}

impl World {
    pub fn from_json(content_json: &str, lot_json: &str, seed: u32) -> Result<Self, Error> {
        let content = Content::from_json(content_json)?;
        let lot_file: LotFile = serde_json::from_str(lot_json)?;
        let world = Self::new(content, &lot_file, seed)?;
        Ok(world)
    }

    pub fn new(content: Content, lot_file: &LotFile, seed: u32) -> Result<Self, Error> {
        let styles: Vec<&str> = content.build.fences.iter().map(|s| s.id.as_str()).collect();
        let lot = Lot::from_file_styled(lot_file, &styles)?;
        let mut world = Self::empty(content, lot, Rng::new(seed));
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
        world.note_paths();
        // (A bus stop in an older town file is no way out: left out.)
        world.exits = lot_file
            .exits
            .iter()
            .filter(|e| e.kind() != crate::travel::ExitKind::Stop)
            .map(|e| e.point())
            .collect();
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
                laundry: LAUNDRY_START,
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
                    laundry: LAUNDRY_START,
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
        // The road network and the town's other open-world fields (`travel.rs`).
        let mut open_world = lot_file.open_world.clone();
        if lot_file
            .plots
            .iter()
            .any(|p| p.area.is_some() || p.kind.is_some())
        {
            open_world.plot_areas = lot_file.plots.iter().map(|p| p.area).collect();
            open_world.plot_kinds = lot_file.plots.iter().map(|p| p.kind).collect();
        }
        world.set_travel(open_world);
        for p in &lot_file.objects {
            let def = world
                .content
                .object_index(&p.def)
                .ok_or_else(|| Error::new(format!("unknown object '{}'", p.def)))?;
            // Turned (only what turns freely may be), set off its tile's middle or resized.
            let at = |e: Error| Error::new(format!("'{}' at {},{}: {e}", p.def, p.x, p.z));
            let turn = world.check_turn(def, p.turn).map_err(at)?;
            let layout = world
                .check_layout(
                    def,
                    p.rot,
                    Layout {
                        offset: p.offset,
                        extent: p.size,
                        tall: p.tall,
                    },
                )
                .map_err(at)?;
            let value = world.content.objects[def].price.unwrap_or(0);
            let id = world.place(
                def,
                p.x,
                p.z,
                p.rot,
                layout,
                Default::default(),
                0,
                value,
                None,
            )?;
            world.objects[id as usize].turn = turn;
        }
        if lot_file.sims.len() > MAX_SIMS {
            return Err(Error::new(format!("at most {MAX_SIMS} residents per town")));
        }
        for s in &lot_file.sims {
            world.spawn_sim(s)?;
        }
        world.init_relationships(&lot_file.relationships)?;
        // Porches as the file has them (a file from before them: as they were drawn), and the
        // pieces (before the cars: a post's tile is kept as a garage's, `cars::reblock`).
        world.porches = match &lot_file.porches {
            Some(list) => list
                .iter()
                .filter(|p| (p.plot as usize) < world.plots.len())
                .copied()
                .collect(),
            None => world.guess_porches(),
        };
        world.porch_pieces = lot_file
            .porch_pieces
            .iter()
            .filter(|p| (p.plot as usize) < world.plots.len())
            .copied()
            .collect();
        crate::cars::from_file(
            &mut world,
            &lot_file.cars,
            &lot_file.drives,
            &lot_file.garages,
        );
        if !world.porch_pieces.is_empty() {
            crate::cars::posts_changed(&mut world);
        }
        crate::traffic::settle(&mut world);
        // Babies come with a crib (a crowded home is reported to the player as time starts).
        for h in 0..world.households.len() {
            crate::lifecycle::cribs_for_babies(&mut world, h);
        }
        world.make_storeys();
        world.refresh_rooms();
        // A new lot's lamps come on only where someone is (`lights::Watch::fresh`).
        world.light_watch.fresh = true;
        Ok(world)
    }

    pub(crate) fn empty(content: Content, lot: Lot, rng: Rng) -> Self {
        let tiles = lot.tile_slots();
        Self {
            content,
            mode: GameMode::Living,
            lot,
            objects: Vec::new(),
            next_object_id: 0,
            items: Default::default(),
            setups: Vec::new(),
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
            nav_log: Default::default(),
            search_mark: 0,
            search_budget: SEARCH_BUDGET,
            objects_version: 1,
            reach: Default::default(),
            reach_version: 0,
            lot_version: 1,
            briefs: Vec::new(),
            shut: Vec::new(),
            private_rooms: Vec::new(),
            claimed_rooms: Vec::new(),
            claimed_places: Vec::new(),
            claimants_cache: Vec::new(),
            work: Vec::new(),
            first: Vec::new(),
            broadcast_scratch: BroadcastScratch::default(),
            exits: Vec::new(),
            travel: Default::default(),
            cars: Vec::new(),
            traffic: Default::default(),
            drives: Vec::new(),
            taxis: Default::default(),
            garages: Vec::new(),
            porches: Vec::new(),
            porch_pieces: Vec::new(),
            car_tiles: Vec::new(),
            cars_nav: 0,
            cars_parked: None,
            cars_near: Default::default(),
            meta: serde_json::Value::Null,
            paths: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            auto_fast: false,
            offers: Vec::new(),
            offers_version: 0,
            dirt: vec![0.0; tiles],
            rooms: Vec::new(),
            rooms_version: 0,
            room_layout_version: 1,
            rooms_things: false,
            rooms_things_at: Vec::new(),
            rooms_things_all: false,
            rooms_out: Vec::new(),
            rooms_dirty: false,
            rooms_rescore: false,
            room_score: Vec::new(),
            room_tiles: Default::default(),
            storeys: 1,
            stairs: crate::path::StairMap::default(),
            lod: true,
            view: None,
            watch: None,
            watched: Vec::new(),
            lights_version: 0,
            light_watch: Default::default(),
            while_lit: Vec::new(),
            staff: Default::default(),
            walk_memo: Vec::new(),
        }
    }

    /// The road network and the rest of the travel state from the town file's open-world
    /// fields (on load too: the graph isn't saved).
    pub(crate) fn set_travel(&mut self, file: crate::travel::OpenWorld) {
        let mut travel = crate::travel::Travel::new(file, &self.meta);
        travel.country_speed = self.content.cars.country_speed;
        // Water and other ground nobody walks on: blocked underneath everything (`base_blocked`).
        if let Some(terrain) = &travel.file.terrain {
            let (w, d) = (
                self.lot.width,
                if self.stairs.depth > 0 {
                    self.stairs.depth as usize
                } else {
                    self.lot.depth
                },
            );
            let mask = terrain.mask(w, d);
            travel.base_blocked = (0..mask.len())
                .filter(|&i| mask[i])
                .map(|i| self.lot.tile_index((i % w) as i32, (i / w) as i32))
                .collect();
            travel.base_blocked.sort_unstable();
            for &i in &travel.base_blocked {
                if let Some(b) = self.blocked.get_mut(i) {
                    *b = true;
                }
            }
        }
        self.travel = travel;
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

    /// Sizes the per-tile arrays (by slot: `Lot::tile_index`) to the lot's slots, after the lot
    /// got new chunks (or was put back as it was before some, by an undo): new slots start
    /// unblocked and clean.
    pub(crate) fn sync_tile_arrays(&mut self) {
        let slots = self.lot.tile_slots();
        if self.blocked.len() != slots {
            self.blocked.resize(slots, false);
        }
        if self.dirt.len() != slots {
            self.dirt.resize(slots, 0.0);
        }
    }

    /// The slot of tile `(x, z)` to write a per-tile value to (`blocked`, `dirt`): its chunk is
    /// allocated first if it wasn't, and the per-tile arrays follow. Off the lot, the void slot
    /// (which nothing may write to).
    pub(crate) fn tile_slot_mut(&mut self, x: i32, z: i32) -> usize {
        if self.lot.in_bounds(x, z) && self.lot.ensure_tile(x, z) {
            self.sync_tile_arrays();
        }
        self.lot.tile_index(x, z)
    }

    /// Whether someone at `b` hears someone at `a` call their name (see
    /// `conversation::within_earshot`).
    pub fn within_earshot(&self, a: [f32; 2], b: [f32; 2]) -> bool {
        conversation::within_earshot(&self.nav(), a, b)
    }

    /// `paths` from `meta.paths`.
    pub(crate) fn note_paths(&mut self) {
        self.paths = self.meta["paths"]
            .as_array()
            .map_or_else(Vec::new, |paths| {
                paths
                    .iter()
                    .map(|r| {
                        let k = |key: &str| r[key].as_f64().unwrap_or(0.0) as f32;
                        [k("x"), k("z"), k("x") + k("w"), k("z") + k("d")]
                    })
                    .collect()
            });
    }

    /// Whether people at `a` and `b` see each other, as two talking must (see
    /// `conversation::in_sight`).
    pub fn in_sight(&self, a: [f32; 2], b: [f32; 2]) -> bool {
        conversation::in_sight(&self.nav(), a, b)
    }

    /// The context a resident's update has (`Ctx`), outside the residents' updates: everyone
    /// else as they are now.
    pub(crate) fn ctx(&self) -> Ctx<'_> {
        let tick = self.tick;
        Ctx {
            content: &self.content,
            nav: self.nav(),
            reach: &self.reach,
            nav_version: self.nav_version,
            nav_log: &self.nav_log,
            rels: &self.relationships,
            briefs: &self.briefs,
            households: &self.households,
            plots: &self.plots,
            tile_plot: &self.tile_plot,
            object_plot: &self.object_plot,
            index: &self.object_index,
            exits: &self.exits,
            travel: &self.travel,
            cars: &self.cars,
            lights: &self.traffic.lights,
            taxis: &self.taxis,
            autonomy: self.autonomy,
            room_score: &self.room_score,
            dirt: &self.dirt,
            hour: crate::clock::hour(tick),
            day: crate::clock::day(tick),
            minute: crate::clock::minute_of_day(tick),
            tick,
            work: &self.work,
            shut: &self.shut,
            rooms: &self.rooms,
            staff: &self.staff,
            paths: &self.paths,
            others: Others::all(&self.sims),
        }
    }

    /// Whether resident `i` would set off now to visit `plot` (an invitation over the phone):
    /// the way isn't too long, as they'd go (`leisure_way`), they'd be there in visiting hours
    /// with time for the stay before they leave for work or school themselves (`visit_fits`,
    /// `free_before_work`), and a household living there isn't all off to work or school before
    /// they could stay `visits.leastMinutes` (`hosts_about_till`).
    pub(crate) fn visit_fits_now(&self, i: usize, plot: u32) -> bool {
        let (Some(sim), Some(p)) = (self.sims.get(i), self.plots.get(plot as usize)) else {
            return false;
        };
        let pos = self.nav().stairs.ground_pos(sim.pos);
        let goer = Goer {
            drives: crate::cars::can_drive(
                &self.travel,
                &self.households,
                &self.briefs,
                &self.cars,
                sim,
            ),
            bike: crate::ride::leisure_bike(
                &self.content,
                &self.object_index,
                &self.briefs,
                &self.households,
                &self.objects,
                sim,
            )
            .map(|(_, speed, min)| (speed, min)),
        };
        let door = p.arrival_point();
        let taxi = self
            .taxis
            .takes(sim)
            .then(|| {
                crate::taxi::leisure_eta(&self.travel, &self.exits, &self.content, sim, pos, door)
            })
            .flatten();
        let (_, eta) = leisure_way(
            &self.travel,
            &self.content,
            pos,
            sim.walk_per_minute(&self.content),
            plot,
            door,
            goer,
            taxi,
            &mut [None, None],
        );
        let stay = crate::mins(eta + self.content.visits.least_minutes);
        // (With time for it before they leave for work or school, as a visit they'd choose for
        // themselves: invited round, a guest let in was off to work 39 minutes on, Brightton.)
        let ctx = self.ctx();
        visit_fits(&self.content, eta, ctx.hour, free_before_work(sim, &ctx))
            && self
                .households
                .iter()
                .find(|h| h.plot == Some(plot))
                .is_none_or(|h| hosts_about_till(&ctx, h.id) >= self.tick + stay)
    }

    /// The plot a tile is on (on any storey).
    pub fn plot_at(&self, x: i32, z: i32) -> Option<u32> {
        let z = self.ground_z(z);
        let width = self.lot.width as i32;
        if (0..width).contains(&x)
            && z >= 0
            && let Some(&plot) = self.tile_plot.get((z * width + x) as usize)
        {
            return (plot != crate::path::NO_PLOT).then_some(plot as u32);
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
        self.tile_plot = vec![crate::path::NO_PLOT; (width * depth) as usize];
        for p in self.plots.iter().rev() {
            for z in p.z.max(0)..(p.z + p.d).min(depth) {
                for x in p.x.max(0)..(p.x + p.w).min(width) {
                    self.tile_plot[(z * width + x) as usize] = p.id as u16;
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
        self.place(
            def,
            x,
            z,
            rot,
            Layout::default(),
            Default::default(),
            0,
            value,
            None,
        )
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
            footprint: layout
                .extent
                .map_or(self.content.objects[def].footprint, tiles_for),
            extent: layout
                .extent
                .unwrap_or(kind_size(self.content.objects[def].footprint)),
            tall: layout.tall,
            users: [None; MAX_SLOTS],
            closed_users: SimSet::default(),
            quality: 0,
            colors: Colors::new(),
            value: 0,
            wear: 0.0,
            open: false,
            // A crop comes ripe with a lot (bought, it is just planted: `buy_at`).
            growth: 1.0,
            off: false,
            run: None,
            rack: 0,
        };
        let d = &self.content.objects[def];
        let name = &d.id;
        let layer = d.layer;
        if layer == Layer::Surface {
            return self.check_surface_fit(obj);
        }
        let on = |o: &ObjectInstance, t: (i32, i32)| o.tiles(&self.content).any(|q| q == t);
        // (Reaching over it at head height: an arc lamp's shade.)
        let over =
            |o: &ObjectInstance, t: (i32, i32)| o.overhang_tiles(&self.content).any(|q| q == t);
        let called = |o: &ObjectInstance| self.content.objects[o.def].name.to_lowercase();
        for (tx, tz) in obj.tiles(&self.content) {
            // Furniture needs free floor; things on a wall, the ceiling or rugs only a free spot
            // on their own layer (and a hanging plant that hangs low the floor as well). A lamp at
            // head height takes the floor from walking only: furniture goes under it.
            let floor = self.lot.in_bounds(tx, tz) && self.blocked[self.lot.tile_index(tx, tz)];
            let furniture = || {
                self.objects
                    .iter()
                    .any(|o| !self.content.objects[o.def].layer.mounted() && on(o, (tx, tz)))
            };
            let taken = if layer.mounted() {
                self.objects
                    .iter()
                    .any(|o| self.content.objects[o.def].layer == layer && on(o, (tx, tz)))
                    || (d.hangs_low && floor && !furniture())
            } else {
                let hung = || {
                    self.objects.iter().any(|o| {
                        ((self.content.objects[o.def].hangs_low
                            || self.content.objects[o.def].head_height)
                            && on(o, (tx, tz)))
                            || over(o, (tx, tz))
                    })
                };
                floor && (furniture() || !hung())
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
            // Nor under (or into) a lamp's shade reaching over from beside.
            let clash = clash.or_else(|| {
                self.objects
                    .iter()
                    .find(|o| over(o, (tx, tz)) && (d.fills_wall || d.hangs_low || d.head_height))
            });
            if let Some(o) = clash {
                let other = &self.content.objects[o.def];
                let why = match layer {
                    Layer::Floor if other.hangs_low => {
                        format!("is under the {}, which hangs too low", called(o))
                    }
                    Layer::Floor => format!("is too tall for the {} hanging there", called(o)),
                    Layer::Ceiling if d.hangs_low && !other.fills_wall => {
                        format!("would hang too low over the {}", called(o))
                    }
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
        // Where it reaches over (an arc lamp's shade): on the lot, off the stairs, no wall in the
        // way from its own tile, and nothing tall or hanging there to run into. Furniture may be
        // under it.
        for (tx, tz) in obj.overhang_tiles(&self.content) {
            if !self.lot.in_bounds(tx, tz)
                || self.lot.diag(tx, tz).is_some()
                || self.stairs.step(tx, tz).is_some()
            {
                return Err(Error::new(format!(
                    "'{name}' at {x},{z} would hang over the edge"
                )));
            }
            let other = |o: &ObjectInstance| {
                let def = &self.content.objects[o.def];
                (on(o, (tx, tz)) && (def.fills_wall || def.hangs_low || def.head_height))
                    || over(o, (tx, tz))
            };
            if let Some(o) = self.objects.iter().find(|o| other(o)) {
                return Err(Error::new(format!(
                    "'{name}' at {x},{z} would hang into the {}",
                    called(o)
                )));
            }
        }
        let (fx, fz) = obj.front_tile(&self.content);
        if !self.lot.in_bounds(fx, fz) {
            return Err(Error::new(format!("'{name}' at {x},{z} faces off the lot")));
        }
        // No wall or fence may run through the footprint, through the body where it stands off
        // the footprint, or between the object and where it's used from.
        let in_lot = |(a, b): &((i32, i32), (i32, i32))| {
            self.lot.in_bounds(a.0, a.1) && self.lot.in_bounds(b.0, b.1)
        };
        let body = obj.body_edges(&self.content).into_iter().filter(in_lot);
        // Nor a diagonal wall across a tile it reaches into.
        let diagonal = |(tx, tz): (i32, i32)| {
            self.lot
                .diag(tx, tz)
                .is_some_and(|g| obj.body_crosses(&self.content, tx, tz, g.dir))
        };
        if obj
            .wall_edges(&self.content)
            .chain(body)
            .chain(obj.overhang_edges(&self.content))
            .any(|(a, b)| self.lot.edge_between(a.0, a.1, b.0, b.1) != crate::lot::Edge::Open)
            || obj.body_tiles(&self.content).any(diagonal)
        {
            return Err(Error::new(format!(
                "'{name}' at {x},{z} is blocked by a wall"
            )));
        }
        if layer == Layer::Wall {
            match self.wall_behind(&obj) {
                None => {
                    return Err(Error::new(format!(
                        "'{name}' at {x},{z} has no wall to hang on"
                    )));
                }
                Some(true) if !d.over_window => {
                    return Err(Error::new(format!(
                        "'{name}' at {x},{z} can't hang over a window"
                    )));
                }
                _ => {}
            }
        }
        // Built to stand back to a wall (a wall-hung toilet's cistern box): plain wall behind it.
        if d.against_wall {
            match self.wall_behind(&obj) {
                None => {
                    return Err(Error::new(format!(
                        "'{name}' at {x},{z} needs a wall behind it"
                    )));
                }
                Some(true) => {
                    return Err(Error::new(format!(
                        "'{name}' at {x},{z} can't stand against a window"
                    )));
                }
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
            return Err(Error::new(format!(
                "'{name}' at {x},{z} overlaps or leaves the lot"
            )));
        }
        let at = crate::surface::middle(&self.content, &obj);
        let radius = self.content.objects[obj.def].radius;
        match crate::surface::fit_on_top(&self.content, &self.objects, at, radius, &[]) {
            Ok(_) => Ok(obj),
            Err(Misfit::NoTop) => Err(Error::new(format!(
                "'{name}' at {x},{z} has nothing to stand on"
            ))),
            Err(Misfit::OverEdge) => Err(Error::new(format!(
                "'{name}' at {x},{z} would hang over the edge"
            ))),
            Err(Misfit::Crowded) => Err(Error::new(format!(
                "'{name}' at {x},{z} overlaps or leaves the lot"
            ))),
        }
    }

    /// The tile a resident stands on to use `obj`, as things stand now (see
    /// `surface::approach_tile`; residents deciding what to do use `Links`).
    pub(crate) fn approach_tile(&self, obj: &ObjectInstance) -> (i32, i32) {
        self.approach_tiles(obj)
            .first()
            .copied()
            .unwrap_or_else(|| obj.front_tile(&self.content))
    }

    /// Every tile a resident may use `obj` from, as things stand now (see `surface::use_tiles`:
    /// a thing on a top from beside its top).
    pub(crate) fn approach_tiles(&self, obj: &ObjectInstance) -> Vec<(i32, i32)> {
        crate::surface::use_tiles(&self.content, &self.objects, &self.nav(), obj)
    }

    /// What is walkable now.
    pub(crate) fn nav(&self) -> NavGrid<'_> {
        NavGrid {
            lot: &self.lot,
            blocked: &self.blocked,
            stairs: &self.stairs,
            kept_off: &self.travel.junction_tiles,
        }
    }

    /// What runs along the whole back edge of `obj`: None if not a wall (plain or with a window)
    /// all along, else whether a window is part of it.
    pub(crate) fn wall_behind(&self, obj: &ObjectInstance) -> Option<bool> {
        use crate::lot::Edge;
        let lot = &self.lot;
        obj.back_edges(&self.content)
            .into_iter()
            .try_fold(false, |window, (horizontal, x, z)| {
                let e = if horizontal {
                    (x >= 0 && (x as usize) < lot.width && z >= 0 && (z as usize) <= lot.depth)
                        .then(|| lot.h_edge(x as usize, z as usize))
                } else {
                    (x >= 0 && (x as usize) <= lot.width && z >= 0 && (z as usize) < lot.depth)
                        .then(|| lot.v_edge(x as usize, z as usize))
                };
                match e? {
                    Edge::Wall => Some(window),
                    Edge::Window => Some(true),
                    _ => None,
                }
            })
    }

    /// What hangs on the wall at this grid edge (`horizontal`: an `h` edge).
    pub(crate) fn hung_on(
        &self,
        horizontal: bool,
        x: i32,
        z: i32,
    ) -> impl Iterator<Item = &ObjectInstance> {
        self.objects.iter().filter(move |o| {
            self.content.objects[o.def].layer == Layer::Wall
                && o.back_edges(&self.content).contains(&(horizontal, x, z))
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
            let i = self.tile_slot_mut(tx, tz);
            debug_assert!(i != crate::lot::VOID);
            self.blocked[i] = true;
        }
        self.object_plot.push(self.plot_at(x, z));
        let stairs = self.content.objects[def].stairs;
        let rect = self.object_rect(&obj);
        let at = (obj.x, obj.z);
        self.objects.push(obj);
        if stairs {
            self.refresh_stairs();
            self.nav_changed();
        } else {
            self.objects_changed_in(rect);
        }
        self.rooms_things_changed(at);
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
        let slot = if reuse {
            self.sims.iter().position(|s| !s.here())
        } else {
            None
        };
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

        let young = !content
            .life
            .adult(spawn.age.unwrap_or(content.life.start_age[0]));
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
        let pension = if retired {
            crate::lifecycle::pension(content, job.as_ref())
        } else {
            0
        };
        let job = job.filter(|_| !retired);
        let mut needs = [0.0; MAX_NEEDS];
        for n in needs.iter_mut().take(self.content.needs.len()) {
            *n = self.rng.range(0.55, 0.95);
        }
        let sim = Sim {
            id: id as u32,
            name: name.clone(),
            appearance: spawn.appearance.clone(),
            outfit: 0,
            gender,
            attracted_to,
            traits: spawn.traits.clone(),
            perks: spawn.perks.clone(),
            base_mods: mods.clone(),
            mods,
            buffs: Vec::new(),
            pending_spend: 0,
            worked: 0,
            made: 0,
            taken: 0,
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
            paused: None,
            join_spot: None,
            idle_ticks: 0,
            follow_up: None,
            fetched: None,
            unreachable: [None; UNREACHABLE],
            social_waited: 0,
            called: false,
            nav_seen: 0,
            deferred: false,
            commute: None,
            on_trail: None,
            game: None,
            wants_partner: false,
            joins: None,
            partner_wait: 0,
            being_served: false,
            serve_wait: 0,
            bound_for_bike: None,
            bike_parked: None,
            bike_spot: None,
            back_at: 0,
            riding: None,
            bound_for_car: None,
            taxi: None,
            car_seat: 0,
            car_door: 0,
            car_wait: 0,
            kerb_wait: None,
            shift_from: 0,
            alone_until: 0,
            household: spawn.household,
            job,
            away_until: None,
            visiting: None,
            transition: None,
            skills,
            skill_ups: 0,
            practice_fatigue: 0.0,
            nap_until: 0,
            ladder_ticks: 0,
            job_search_from: 0,
            planner: Default::default(),
            mess: None,
            reshaped: false,
            lit: false,
            care_out: None,
            carry_out: None,
            carried_by: None,
            night_woken: 0,
            conceived: None,
            to_bed: None,
            kept_out: 0,
            news: None,
            set_up: None,
            accident_at: [0; crate::content::MAX_ACCIDENTS],
            empty_since: [0; crate::content::MAX_ACCIDENTS],
            age,
            gone: None,
            held: None,
            invited_until: 0,
            serve_for: None,
            hand_to: None,
            expecting: 0,
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
        let household = self
            .households
            .get(gone.household as usize)
            .map_or_else(String::new, |h| h.name.clone());
        self.former.push(Former {
            name: gone.name.clone(),
            household,
        });
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
            TaskKind::Social { target, .. }
            | TaskKind::Join { leader: target }
            | TaskKind::Answer { guest: target } => target == id,
            TaskKind::Phone { contact, .. } => contact == Some(id),
            TaskKind::Carry { baby, .. } => baby == Some(id),
            TaskKind::Serve { guest, .. } => guest == id,
            _ => false,
        };
        let World {
            sims,
            content,
            objects,
            households,
            ..
        } = self;
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
            if let Some(Activity {
                phase: Phase::Knocking { host, .. },
                ..
            }) = other.current.as_mut()
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
        let mut mods = content
            .character_modifiers(&sim.traits, &sim.perks)
            .unwrap_or_default();
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
        let members = self
            .sims
            .iter()
            .filter(|s| s.here() && s.household as usize == h)
            .count();
        let most = self.content.rules.max_household;
        if members + sims.len() > most {
            return Err(Error::new(format!(
                "at most {most} residents per household"
            )));
        }
        let cap = MAX_SIMS;
        if self.sims.iter().filter(|s| s.here()).count() + sims.len() > cap {
            return Err(Error::new(format!("at most {cap} residents in this town")));
        }
        if bonds.iter().any(|b| b.a >= sims.len() || b.b >= sims.len()) {
            return Err(Error::new("a bond refers to someone who isn't moving in"));
        }
        let life = &self.content.life;
        let grown_up = |s: &SimSpawn| s.age.is_none_or(|a| life.adult(a));
        let adult_at_home = self
            .sims
            .iter()
            .any(|s| s.here() && s.household as usize == h && s.adult(&self.content));
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
                let spawn = SimSpawn {
                    household,
                    ..s.clone()
                };
                ids.push(self.spawn_at(&spawn, true)? as usize);
                Ok(())
            })
            .and_then(|()| {
                let bonds: Vec<BondRaw> = bonds
                    .iter()
                    .map(|b| BondRaw {
                        a: ids[b.a],
                        b: ids[b.b],
                        preset: b.preset.clone(),
                    })
                    .collect();
                self.init_relationships_for(&ids, &bonds)
            });
        if let Err(e) = moved {
            (
                self.sims,
                self.relationships,
                self.rng,
                self.former,
                self.generations,
                self.events,
            ) = before;
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
    pub(crate) fn init_relationships_for(
        &mut self,
        new: &[usize],
        bonds: &[BondRaw],
    ) -> Result<(), Error> {
        let n = self.sims.len();
        let default_bond = self.content.social_rules.default_bond;
        for a in 0..n {
            for b in a + 1..n {
                if !(new.contains(&a) || new.contains(&b))
                    || !self.sims[a].here()
                    || !self.sims[b].here()
                {
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
        let mut members = self
            .sims
            .iter()
            .filter(|s| s.here() && home(s.household))
            .peekable();
        if members.peek().is_none() {
            return false;
        }
        let quiet = members.all(|s| s.away_until.is_some() || s.asleep(&self.content));
        let hour_ago = self.tick.saturating_sub(TICKS_PER_HOUR);
        quiet
            && !self
                .events
                .iter()
                .rev()
                .take_while(|e| e.tick >= hour_ago)
                .any(|e| {
                    e.kind.importance() >= 1
                        && [Some(e.a), Some(e.b), e.c]
                            .into_iter()
                            .flatten()
                            .any(|s| self.sims.get(s as usize).is_some_and(|s| home(s.household)))
                })
    }

    /// Every room and garden of the town with its scores (current as of the last refresh).
    /// Everything found with the rooms, for tests that compare two ways of finding them: the
    /// rooms, the score per tile, the private and claimed rooms and the places in those.
    #[doc(hidden)]
    pub fn room_state(&self) -> (Vec<crate::rooms::RoomInfo>, Vec<f32>, Vec<u16>, Vec<u16>) {
        (
            self.rooms.clone(),
            self.room_score.clone(),
            self.private_rooms.clone(),
            self.claimed_rooms.clone(),
        )
    }

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
                // (Both done trying at once: one chance, not two.)
                if let Some(p) = self
                    .sims
                    .get_mut(partner as usize)
                    .filter(|p| p.conceived == Some(i as u32))
                {
                    p.conceived = None;
                }
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
                // An object opened or closed, or someone changed clothes: how it looks changes.
                self.structure_version = self.structure_version.wrapping_add(1);
                self.rooms_need_clean_rescore();
            }
            if std::mem::take(&mut self.sims[i].lit) {
                self.lights_version = self.lights_version.wrapping_add(1);
            }
            if let Some(news) = self.sims[i].news.take() {
                // Broken or mended: what the house offers and how it looks change.
                self.structure_version = self.structure_version.wrapping_add(1);
                let at = match news {
                    News::Broke(o) | News::Repaired(o) | News::NotClosed(o) => {
                        let o = &self.objects[o as usize];
                        (o.x, o.z)
                    }
                    News::Accident(_) => self.sims[i].tile(),
                };
                self.rooms_things_changed(at);
                let (kind, n) = match news {
                    News::Broke(o) => (
                        social::EventKind::Broke,
                        self.objects[o as usize].def as i64,
                    ),
                    News::Repaired(o) => (
                        social::EventKind::Repaired,
                        self.objects[o as usize].def as i64,
                    ),
                    News::Accident(a) => (social::EventKind::Accident, a as i64),
                    News::NotClosed(o) => (
                        social::EventKind::NotClosed,
                        self.objects[o as usize].def as i64,
                    ),
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
                    let share = if amount < 0.0 || (dx, dz) == (0, 0) {
                        1.0
                    } else if dx == 0 || dz == 0 {
                        0.5
                    } else {
                        0.25
                    };
                    let t = self.tile_slot_mut(tx, tz);
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
        let Some(rules) = self.content.day_rhythm.baby_wakes else {
            return;
        };
        for b in 0..self.sims.len() {
            let Some((carer, tasks)) = self.woken_for(b, &rules) else {
                continue;
            };
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
        let plot = self.households[h]
            .plot
            .filter(|_| self.autonomy && self.households[h].free_will)?;
        // In a crib at home, with care for what they're crying for.
        let crib = self.objects.iter().find(|o| {
            o.users().any(|u| u as usize == b) && self.object_plot[o.id as usize] == Some(plot)
        })?;
        let def = &content.objects[crib.def];
        let care = |i: &crate::content::Interaction, below: f32| {
            i.cares() && (0..content.needs.len()).any(|n| i.care[n] > 0.0 && wants(n, below))
        };
        if !def.interactions.iter().any(|i| care(i, rules.below)) || !crib.has_free_slot(def.slots)
        {
            return None;
        }
        // The grown-ups at home.
        let home: Vec<usize> = (0..self.sims.len())
            .filter(|&j| {
                let s = &self.sims[j];
                let (x, z) = s.tile();
                s.here()
                    && s.household as usize == h
                    && s.away_until.is_none()
                    && s.visiting.is_none()
                    && s.adult(content)
                    && self.plot_at(x, z) == Some(plot)
            })
            .collect();
        // Nothing to do if someone's already doing the crib's care (or on their way to).
        let tending = |t: &Task| matches!(t.kind, TaskKind::Use { object, interaction } if object == crib.id && def.interactions[interaction].cares());
        if home.iter().any(|&j| {
            self.sims[j].current().is_some_and(|a| tending(&a.task))
                || self.sims[j].queue.iter().any(tending)
        }) {
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
            .map(|(i, _)| Task {
                kind: TaskKind::Use {
                    object: crib.id,
                    interaction: i,
                },
                directed: false,
                routine: false,
            })
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
            (
                family,
                rest(j) >= rhythm.exhausted_below,
                std::cmp::Reverse(self.sims[j].night_woken),
            )
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
                TaskKind::Use { .. }
                | TaskKind::MoveTo { .. }
                | TaskKind::Clean { .. }
                | TaskKind::Eat { .. } => true,
                TaskKind::Cook { .. } => {
                    crate::cooking::step_left(sim).is_some_and(|m| m <= rules.finish_minutes)
                }
                _ => false,
            };
            !a.task.directed && drop && a.tags & content.social_rules.busy_tags == 0
        });
        let next = sim
            .queue
            .front()
            .is_some_and(|t| t.directed || matches!(t.kind, TaskKind::Work));
        let ready = social::minutes_to_ticks(content.day_rhythm.ready_minutes);
        let leaving = leaves_by(sim, content, self.tick, self.tick + ready);
        picked && !next && !leaving
    }

    /// Rebuilds the objects-by-plot index if objects were added, removed or changed since.
    pub(crate) fn refresh_index(&mut self) {
        if self.object_index.version != self.objects_version {
            let centres: Vec<[f32; 2]> = self
                .objects
                .iter()
                .map(|o| o.centre(&self.content))
                .collect();
            let ground = if self.stairs.depth > 0 {
                self.stairs.depth as usize
            } else {
                self.lot.depth
            };
            let size = [self.lot.width as f32, ground as f32];
            self.object_index = ObjectIndex::open(
                &self.object_plot,
                &self.plots,
                &centres,
                size,
                self.objects_version,
            );
            let nav = NavGrid {
                lot: &self.lot,
                blocked: &self.blocked,
                stairs: &self.stairs,
                kept_off: &self.travel.junction_tiles,
            };
            self.object_index.links =
                crate::surface::Links::new(&self.content, &self.objects, &nav);
        }
    }

    /// The tiles `[x0, z0, x1, z1)` an object takes up or blocks, for `objects_changed_in`.
    pub(crate) fn object_rect(&self, obj: &ObjectInstance) -> [i32; 4] {
        let mut r = [i32::MAX, i32::MAX, i32::MIN, i32::MIN];
        for (x, z) in obj.tiles(&self.content).chain(self.blocking_tiles(obj)) {
            r = [r[0].min(x), r[1].min(z), r[2].max(x + 1), r[3].max(z + 1)];
        }
        if r[0] > r[2] {
            [obj.x, obj.z, obj.x + 1, obj.z + 1]
        } else {
            r
        }
    }

    /// Whether this tick's searches so far have expanded more than `search_budget` tiles: what can
    /// wait for the next tick does (residents' turns, the day's commutes worked out again).
    pub(crate) fn over_search_budget(&self) -> bool {
        crate::path::nodes_searched() - self.search_mark > self.search_budget
    }

    /// The navigation version: bumped whenever something walkers can or can't step through
    /// changes (see `nav_changed_in`).
    pub fn nav_version(&self) -> u32 {
        self.nav_version
    }

    /// Whether walking may have changed within tiles `[x0, z0, x1, z1)` (stacked rows, as
    /// positions are) since navigation version `seen`: a change there was logged since, or the
    /// log (`path::NAV_LOG` changes) doesn't go back that far. A walker whose way lies outside
    /// every change since it planned it can keep it.
    pub fn nav_changed_since(&self, seen: u32, rect: [i32; 4]) -> bool {
        self.nav_log.changed_since(seen, self.nav_version, rect)
    }

    /// Catches up on what changed for walking since the last tick (the walkable regions, the
    /// objects-by-plot index and the links), as the next tick would: for benchmarks and tools.
    pub fn refresh_navigation(&mut self) {
        self.refresh_reach();
        self.refresh_index();
    }

    /// Catches up on the walkable regions only (`refresh_navigation` without the indexes).
    pub fn refresh_walkable(&mut self) {
        self.refresh_reach();
    }

    /// Labels the walkable regions of the lot again if the navigation changed (see
    /// `path::Reach`): an unreachable goal is then refused before a search can flood the town.
    fn refresh_reach(&mut self) {
        if self.reach_version == self.nav_version && !self.reach.stale() {
            return;
        }
        if !self.reach.stale() {
            // (Bumped without saying where: anywhere.)
            self.reach.mark_all();
        }
        let nav = NavGrid {
            lot: &self.lot,
            blocked: &self.blocked,
            stairs: &self.stairs,
            kept_off: &self.travel.junction_tiles,
        };
        self.reach.refresh(&nav);
        self.reach_version = self.nav_version;
    }

    /// Notes that something walkers can or can't step through changed, anywhere (walls, storeys,
    /// stairs, an undo), which views and the objects' indexes also see.
    pub(crate) fn nav_changed(&mut self) {
        self.structure_version = self.structure_version.wrapping_add(1);
        self.objects_version = self.objects_version.wrapping_add(1);
        let all = [0, 0, self.lot.width as i32 + 1, self.lot.depth as i32 + 1];
        self.nav_changed_in(all);
        self.reach.mark_all();
    }

    /// Notes that an object came, went or moved over tiles `[x0, z0, x1, z1)`: views and the
    /// objects' indexes see it, walkers near it find their way again.
    pub(crate) fn objects_changed_in(&mut self, rect: [i32; 4]) {
        self.structure_version = self.structure_version.wrapping_add(1);
        self.objects_version = self.objects_version.wrapping_add(1);
        self.nav_changed_in(rect);
    }

    /// Notes that walls, doors, windows, fences or diagonals were built or taken down on plot
    /// `plot`, on the grid lines round tiles `edits` (`[x0, z0, x1, z1)`, stacked rows): as
    /// `objects_changed_in` there, and over the plot on every block of rows past the ground's (its
    /// rooms say where there's floor above them), so the walkable regions are labelled again only
    /// there and walkers whose way doesn't pass it keep their way.
    pub(crate) fn walls_changed_on(&mut self, plot: u32, edits: [i32; 4]) {
        let Some(p) = self.plots.get(plot as usize) else {
            self.nav_changed();
            return;
        };
        let (x0, z0, x1, z1) = (p.x - 1, p.z - 1, p.x + p.w + 1, p.z + p.d + 1);
        let ground = if self.stairs.depth > 0 {
            self.stairs.depth
        } else {
            self.lot.depth as i32
        };
        let blocks = (self.lot.depth as i32 / ground.max(1)).max(1);
        self.objects_changed_in(edits);
        for b in 1..blocks {
            let dz = b * ground;
            self.nav_changed_in([x0, z0 + dz, x1, z1 + dz]);
        }
    }

    /// Notes that walking changed within tiles `[x0, z0, x1, z1)` only (a car parked): the
    /// walkable regions are labelled again there, and walkers whose way goes near it find their
    /// way again (`nav_changed_since`). Views and the objects' indexes don't follow it.
    pub(crate) fn nav_changed_in(&mut self, rect: [i32; 4]) {
        self.nav_version = self.nav_version.wrapping_add(1);
        self.nav_log.push(self.tick, self.nav_version, rect);
        self.reach.mark(&self.lot, rect);
    }

    /// Notes that the lot's rooms changed (walls, storeys, a blueprint, an undo): the next tick
    /// finds and scores the rooms again.
    pub(crate) fn rooms_layout_changed(&mut self) {
        self.room_layout_version = self.room_layout_version.wrapping_add(1);
    }

    /// Notes that a room's scores changed without its layout (a wall painted, a floor covered):
    /// the next tick recomputes the rooms once.
    pub(crate) fn rooms_need_rescore(&mut self) {
        self.rooms_dirty = true;
    }

    /// Notes that a thing standing on tile `at` came, went, broke, was mended or upgraded, and
    /// nothing else about the rooms changed: the next tick scores the room (or garden) there
    /// again for what stands in it (`refresh_room_things`) without finding it again.
    pub(crate) fn rooms_things_changed(&mut self, at: (i32, i32)) {
        self.rooms_things = true;
        if self.rooms_things_at.len() < MOST_THINGS_AT {
            self.rooms_things_at.push(at);
        } else {
            self.rooms_things_all = true;
        }
    }

    /// Notes that things may have changed anywhere (an undo put them back): every room is scored
    /// again for what stands in it.
    pub(crate) fn rooms_things_changed_everywhere(&mut self) {
        self.rooms_things = true;
        self.rooms_things_all = true;
    }

    /// Notes that only cleanliness may have changed (a crop grew, a thing opened): the next
    /// tick rescores the rooms' cleanliness without walking the whole lot.
    pub(crate) fn rooms_need_clean_rescore(&mut self) {
        self.rooms_rescore = true;
    }

    /// Recomputes rooms and their scores (see `rooms.rs`). Call it after changing `dirt`
    /// directly; messes residents make are rescored on their own.
    pub fn refresh_rooms(&mut self) {
        self.sync_tile_arrays();
        self.map_plots();
        // (Everything is found again: what the lot says changed since doesn't matter.)
        self.lot.take_room_changes();
        let (rooms, cells) = crate::rooms::compute(self);
        self.rooms = rooms;
        self.rooms_out = crate::privacy::doors_out(&self.content, &self.rooms, &self.lot);
        self.refresh_privacy();
        self.rooms_version = self.room_layout_version;
        self.rooms_dirty = false;
        self.rooms_things = false;
        self.rooms_things_all = false;
        self.rooms_things_at.clear();
        self.rooms_rescore = false;
        // The room (index into `rooms`) whose score each tile takes: its plot's garden, then
        // indoor tiles anywhere their room; off the plots, none (`roomRules.away`).
        let rooms = &self.rooms;
        let mut source = vec![crate::rooms::NO_ROOM; self.lot.tile_slots()];
        for (pi, p) in self.plots.iter().enumerate() {
            let garden = rooms
                .iter()
                .position(|r| r.garden && r.plot == Some(pi as u32))
                .map_or(crate::rooms::NO_ROOM, |k| k as u32);
            for z in p.z..p.z + p.d {
                for x in p.x..p.x + p.w {
                    if self.lot.in_bounds(x, z) {
                        source[self.lot.tile_index(x, z)] = garden;
                    }
                }
            }
        }
        source[crate::lot::VOID] = crate::rooms::NO_ROOM;
        let max_id = rooms
            .iter()
            .filter(|r| !r.garden)
            .map(|r| r.id as usize)
            .max()
            .unwrap_or(0);
        let mut by_id = vec![None; max_id + 1];
        for (k, r) in rooms.iter().enumerate().filter(|(_, r)| !r.garden) {
            by_id[r.id as usize] = Some(k);
        }
        for (_, _, i) in self.lot.indoor_tiles() {
            let id = self.lot.rooms()[i];
            if let Some(Some(k)) = by_id
                .get(id as usize)
                .filter(|_| id != crate::lot::OUTDOORS)
            {
                source[i] = *k as u32;
            }
        }
        let away = self.content.room_rules.away;
        self.room_score = source
            .iter()
            .map(|&k| match k {
                crate::rooms::NO_ROOM => away,
                k => rooms[k as usize].scores.overall,
            })
            .collect();
        self.room_tiles = crate::rooms::RoomTiles::new(cells, &source);
    }

    /// Catches up on the rooms after walls, coverings, things or dirt changed: as `refresh_rooms`
    /// would give, but finding and scoring again only the rooms round what was built
    /// (`refresh_rooms_near`) and where things changed (`refresh_room_things`), and the rest only
    /// for their dirt where there was a mess.
    fn update_rooms(&mut self) {
        // The per-tile lists follow the lot's slots (chunks allocated since: outdoors).
        let slots = self.lot.tile_slots();
        if self.room_score.len() < slots {
            self.room_score.resize(slots, self.content.room_rules.away);
        }
        self.room_tiles.grow(slots);
        let layout = self.rooms_version != self.room_layout_version || self.rooms_dirty;
        if layout {
            let changes = self.lot.take_room_changes();
            if changes.full {
                self.refresh_rooms();
                return;
            }
            self.refresh_rooms_near(&changes);
            self.rooms_version = self.room_layout_version;
            self.rooms_dirty = false;
        }
        let things = self.rooms_things;
        if things {
            self.refresh_room_things();
        } else {
            self.rescore_rooms();
        }
        self.rooms_rescore = false;
        if layout || things {
            self.refresh_privacy();
        }
    }

    /// The rooms found and scored again inside `changes.rect` (grown a tile, for the rooms across
    /// a wall from it) and the gardens of the plots there; the rest keep their scores, renumbered
    /// as the lot renumbered them (`Lot::take_room_changes`). What `refresh_rooms` would give.
    fn refresh_rooms_near(&mut self, changes: &crate::lot::RoomChanges) {
        use crate::rooms::room_order;
        self.sync_tile_arrays();
        self.map_plots();
        let away = self.content.room_rules.away;
        let Some([x0, z0, x1, z1]) = changes.rect else {
            return;
        };
        let rect = [x0 - 1, z0 - 1, x1 + 1, z1 + 1];
        let meets =
            |r: [i32; 4]| r[0] < rect[2] && rect[0] < r[2] && r[1] < rect[3] && rect[1] < r[3];
        // The plots there, on any storey (what stands upstairs outdoors is in the garden).
        let depth = self.storey_depth().max(1);
        let blocks = (self.lot.depth as i32 / depth).max(1);
        let plots: Vec<u32> = self
            .plots
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                (0..blocks).any(|b| meets([p.x, p.z + b * depth, p.x + p.w, p.z + p.d + b * depth]))
            })
            .map(|(k, _)| k as u32)
            .collect();
        // The rooms there now, and the ones kept (renumbered).
        let ids: Vec<u16> = (1..=self.lot.room_count() as u16)
            .filter(|&id| self.lot.room_rect(id).is_some_and(meets))
            .collect();
        let rooms = std::mem::take(&mut self.rooms);
        let handles = self.room_tiles.take_handles();
        let old_out = std::mem::take(&mut self.rooms_out);
        let mut kept: Vec<(crate::rooms::RoomInfo, u32)> = Vec::with_capacity(rooms.len());
        let mut out_kept: Vec<(u16, bool)> = Vec::new();
        for (mut room, h) in rooms.into_iter().zip(handles) {
            let keep = if room.garden {
                room.plot.is_some_and(|p| plots.binary_search(&p).is_err())
            } else {
                let id = changes
                    .remap
                    .as_ref()
                    .map_or(room.id, |m| m.get(room.id as usize).copied().unwrap_or(0));
                if id != crate::lot::OUTDOORS {
                    out_kept.push((id, old_out.get(room.id as usize).copied().unwrap_or(false)));
                }
                room.id = id;
                id != crate::lot::OUTDOORS && ids.binary_search(&id).is_err()
            };
            if keep {
                kept.push((room, h));
            } else {
                self.room_tiles.drop_room(h);
            }
        }
        // Found and scored again, with the tiles that take each one's score.
        let found = crate::rooms::compute_near(self, &ids, &plots);
        let lot = &self.lot;
        // The last plot over each tile sets its garden's score there (as `refresh_rooms`).
        let last_plot = |x: i32, z: i32| {
            plots
                .iter()
                .rev()
                .copied()
                .find(|&p| self.plots[p as usize].contains(x, z))
        };
        let gardens: Vec<u32> = found
            .iter()
            .filter(|(r, _)| r.garden)
            .filter_map(|(r, _)| r.plot)
            .collect();
        for (room, cells) in found {
            let mut scored = Vec::new();
            if room.garden {
                let pi = room.plot.expect("a garden is a plot's");
                let p = &self.plots[pi as usize];
                // (A later plot over the same tiles scores them: as `refresh_rooms`.)
                let later: Vec<&Plot> = self.plots[pi as usize + 1..]
                    .iter()
                    .filter(|q| {
                        q.x < p.x + p.w && p.x < q.x + q.w && q.z < p.z + p.d && p.z < q.z + q.d
                    })
                    .collect();
                for z in p.z..p.z + p.d {
                    for x in p.x..p.x + p.w {
                        if !lot.in_bounds(x, z) {
                            continue;
                        }
                        let i = lot.tile_index(x, z);
                        if i != crate::lot::VOID
                            && lot.rooms()[i] == crate::lot::OUTDOORS
                            && !later.iter().any(|q| q.contains(x, z))
                        {
                            scored.push(i as u32);
                        }
                    }
                }
            } else if let Some([rx0, rz0, rx1, rz1]) = lot.room_rect(room.id) {
                for z in rz0..rz1 {
                    for x in rx0..rx1 {
                        let i = lot.tile_index(x, z);
                        if i != crate::lot::VOID && lot.rooms()[i] == room.id {
                            scored.push(i as u32);
                        }
                    }
                }
            }
            for &i in &scored {
                self.room_score[i as usize] = room.scores.overall;
            }
            let h = self.room_tiles.add_room(cells, scored);
            kept.push((room, h));
        }
        // Tiles there no room or garden scores: away from home.
        for z in rect[1].max(0)..rect[3].min(lot.depth as i32) {
            for x in rect[0].max(0)..rect[2].min(lot.width as i32) {
                let i = lot.tile_index(x, z);
                if i == crate::lot::VOID || lot.rooms()[i] != crate::lot::OUTDOORS {
                    continue;
                }
                let garden = (z < depth)
                    .then(|| last_plot(x, z))
                    .flatten()
                    .filter(|p| gardens.contains(p));
                if garden.is_none() {
                    self.room_score[i] = away;
                }
            }
        }
        kept.sort_by_key(|(r, _)| room_order(r));
        let (rooms, handles): (Vec<_>, Vec<_>) = kept.into_iter().unzip();
        self.rooms = rooms;
        self.room_tiles.set_handles(handles);
        // Doors to the outside, per room id (as `privacy::doors_out`).
        if self.content.room_kinds.iter().any(|k| k.private) {
            let most = self.rooms.iter().map(|r| r.id as usize).max().unwrap_or(0);
            let mut out = vec![false; most + 1];
            for (id, o) in out_kept {
                if let Some(slot) = out.get_mut(id as usize) {
                    *slot = o;
                }
            }
            for &id in &ids {
                if let Some(slot) = out.get_mut(id as usize) {
                    *slot = crate::privacy::door_out(&self.lot, id);
                }
            }
            self.rooms_out = out;
        }
    }

    /// Scores the rooms again for what stands in them, the rooms themselves as they are (things
    /// came, went, broke, were mended or upgraded, and nothing was built: `rooms_things_changed`):
    /// the rooms where things changed, or all of them if things may have changed anywhere, and
    /// the rest for their dirt where there was a mess. The same as `refresh_rooms` would give,
    /// at the cost of the things and the rooms' dirt, not of finding the rooms and walking every
    /// tile.
    fn refresh_room_things(&mut self) {
        let mut rooms = std::mem::take(&mut self.rooms);
        let mut tiles = std::mem::take(&mut self.room_tiles);
        let mut score = std::mem::take(&mut self.room_score);
        let at = std::mem::take(&mut self.rooms_things_at);
        let only = (!self.rooms_things_all).then(|| {
            let mut picked = vec![false; rooms.len()];
            for &(x, z) in &at {
                let id = self.lot.room_at(x, z);
                let k = if id == crate::lot::OUTDOORS {
                    let plot = self.plot_at(x, z);
                    rooms
                        .iter()
                        .position(|r| r.garden && plot.is_some() && r.plot == plot)
                } else {
                    rooms.iter().position(|r| !r.garden && r.id == id)
                };
                if let Some(k) = k {
                    picked[k] = true;
                }
            }
            picked
        });
        crate::rooms::rescore_things(self, &mut rooms, &mut tiles, &mut score, only.as_deref());
        (self.rooms, self.room_tiles, self.room_score) = (rooms, tiles, score);
        self.rooms_things = false;
        self.rooms_things_all = false;
        self.rooms_rescore = false;
    }

    /// The rooms kept shut and the claimed ones among them, and the places in those (`privacy`),
    /// from the rooms as they are.
    fn refresh_privacy(&mut self) {
        let (private, claimed) =
            crate::privacy::private_rooms(&self.content, &self.plots, &self.rooms, &self.rooms_out);
        self.private_rooms = private;
        self.claimed_rooms = claimed;
        self.claimed_places = crate::privacy::claimed_places(
            &self.content,
            &self.lot,
            &self.claimed_rooms,
            &self.objects,
        );
        self.claimants_cache.clear();
    }

    /// Catches up on the rooms as the next tick would (found again, scored again for what stands
    /// in them, or for their dirt), for benchmarks and tools.
    pub fn catch_up_rooms(&mut self) {
        if self.rooms_version != self.room_layout_version
            || self.rooms_dirty
            || self.rooms_things
            || self.rooms_rescore
        {
            self.update_rooms();
        }
    }

    /// For tests: catches up on the rooms as the next tick would, and checks that what that gave
    /// (found again only round what changed, scored again only where things changed) is what
    /// finding and scoring every room again gives: the lot's rooms and their numbers, every room
    /// and garden with its scores, the score per tile, the rooms' tiles, doors to the outside,
    /// private and claimed rooms, and `structure_json`. The rooms are left as they were kept.
    #[doc(hidden)]
    pub fn check_rooms(&mut self) -> Result<(), String> {
        self.catch_up_rooms();
        // (Messes since the tick began are scored at the next refresh, as any refresh does.)
        self.rescore_rooms();
        // (Chunks allocated since, by an edit given up: outdoors, away, as a short list reads.)
        let slots = self.lot.tile_slots();
        if self.room_score.len() < slots {
            self.room_score.resize(slots, self.content.room_rules.away);
        }
        self.room_tiles.grow(slots);
        let mut lot = self.lot.clone();
        lot.compute_all_rooms();
        if lot != self.lot || lot.rooms_row_major() != self.lot.rooms_row_major() {
            return Err("the lot's rooms differ from finding them all again".into());
        }
        if lot.room_count() != self.lot.room_count()
            || (1..=lot.room_count() as u16).any(|id| lot.room_rect(id) != self.lot.room_rect(id))
            || !lot.indoor_tiles().eq(self.lot.indoor_tiles())
        {
            return Err("the lot's rooms are kept differently".into());
        }
        let kept = (
            std::mem::take(&mut self.rooms),
            std::mem::take(&mut self.room_score),
            std::mem::take(&mut self.room_tiles),
            std::mem::take(&mut self.rooms_out),
            std::mem::take(&mut self.private_rooms),
            std::mem::take(&mut self.claimed_rooms),
            std::mem::take(&mut self.claimed_places),
        );
        self.rooms.clone_from(&kept.0);
        self.room_score.clone_from(&kept.1);
        self.rooms_out.clone_from(&kept.3);
        self.private_rooms.clone_from(&kept.4);
        self.claimed_rooms.clone_from(&kept.5);
        self.claimed_places.clone_from(&kept.6);
        let structure = crate::view::structure_json(self);
        let tiles = kept.2.summary();
        self.refresh_rooms();
        let mut wrong = Vec::new();
        if self.rooms != kept.0 {
            let n = self.rooms.len().max(kept.0.len());
            let first = (0..n).find(|&k| self.rooms.get(k) != kept.0.get(k));
            wrong.push(format!(
                "rooms ({} kept, {} found; first apart: {:?} kept, {:?} found)",
                kept.0.len(),
                self.rooms.len(),
                first.and_then(|k| kept.0.get(k)),
                first.and_then(|k| self.rooms.get(k)),
            ));
        }
        let bits = |v: &[f32]| v.iter().map(|f| f.to_bits()).collect::<Vec<_>>();
        if bits(&self.room_score) != bits(&kept.1) {
            let at = (0..self.room_score.len().max(kept.1.len())).find(|&i| {
                self.room_score.get(i).map(|f| f.to_bits()) != kept.1.get(i).map(|f| f.to_bits())
            });
            wrong.push(format!(
                "score per tile (first at slot {at:?}: {:?} kept, {:?} found)",
                at.and_then(|i| kept.1.get(i)),
                at.and_then(|i| self.room_score.get(i)),
            ));
        }
        let full = self.room_tiles.summary();
        if full.0 != tiles.0 {
            wrong.push("the rooms' tiles".into());
        }
        if full.1 != tiles.1 {
            wrong.push("the rooms per tile".into());
        }
        if self.rooms_out != kept.3 {
            wrong.push("doors to the outside".into());
        }
        if self.private_rooms != kept.4 || self.claimed_rooms != kept.5 {
            wrong.push("private rooms".into());
        }
        if self.claimed_places != kept.6 {
            wrong.push("claimed places".into());
        }
        if crate::view::structure_json(self) != structure {
            wrong.push("structure_json".into());
        }
        (
            self.rooms,
            self.room_score,
            self.room_tiles,
            self.rooms_out,
            self.private_rooms,
            self.claimed_rooms,
            self.claimed_places,
        ) = kept;
        if wrong.is_empty() {
            Ok(())
        } else {
            Err(wrong.join("; "))
        }
    }

    /// Scores cleanliness again in rooms that had a mess since: between structure changes
    /// only the dirt changes, so the rooms needn't be found again (a whole-town job).
    fn rescore_rooms(&mut self) {
        self.room_tiles.rescore(
            &mut self.rooms,
            &self.content.room_rules,
            &self.dirt,
            &mut self.room_score,
        );
    }

    pub fn tick_once(&mut self) {
        self.sync_tile_arrays();
        // Time moves on: edits can no longer be taken back.
        self.undo.clear();
        self.redo.clear();
        // (An edit still under way can't be taken back now: nothing more to keep of it.)
        self.lot.log_off();
        if self.rooms_version != self.room_layout_version
            || self.rooms_dirty
            || self.rooms_things
            || self.rooms_rescore
        {
            self.update_rooms();
        } else if self.tick.is_multiple_of(ROOM_REFRESH_TICKS) {
            self.rescore_rooms();
        }
        self.refresh_reach();
        self.refresh_index();
        self.travel.note_doors(&self.plots);
        // A new game's first step: the player hears of anyone without a bed (told as the world was
        // made, it would be the story so far, which the view doesn't show as news).
        if self.tick == 0 {
            for h in 0..self.households.len() {
                crate::lifecycle::notify_missing_beds(self, h);
            }
        }
        self.tick += 1;
        let tick = self.tick;
        self.search_mark = crate::path::nodes_searched();

        crate::partners::refresh(self);
        crate::staff::update(self);
        let mut briefs = std::mem::take(&mut self.briefs);
        briefs.clear();
        for s in &self.sims {
            let (x, z) = s.tile();
            let plot = self.plot_at(x, z);
            let available = s.here() && s.available_for_social(&self.content);
            briefs.push(SimBrief {
                pos: s.pos,
                available,
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
                cleaning: s.current().and_then(|a| {
                    if let TaskKind::Clean { x, z } = a.task.kind {
                        Some((x, z))
                    } else {
                        None
                    }
                }),
                seated: s.pose == Pose::Sit && !s.is_moving(),
                lying: s.pose == Pose::Lie,
                eating: s.eating_seated(),
                game: s.game,
                // (`partners::free_now`, its start just above.)
                free: {
                    let free = available && crate::partners::free_besides(&self.content, s, tick);
                    debug_assert_eq!(free, crate::partners::free_now(&self.content, s, tick));
                    free
                },
                bound_for_car: s.bound_for_car.filter(|_| {
                    s.current()
                        .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
                }),
                trip_car: s.bound_for_car.filter(|_| {
                    s.current().is_some_and(|a| {
                        matches!(a.task.kind, TaskKind::Visit { .. } | TaskKind::GoHome)
                    })
                }),
                bike: s.commute.and_then(|c| c.bike),
                bike_spot: s.bike_spot.map(|(at, _)| at),
                guest_at: s.visiting.map(|v| v.plot),
                eats_at: match s.current().map(|a| a.task.kind) {
                    Some(TaskKind::Eat { seat, stage: 1, .. }) => seat,
                    _ => None,
                },
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
        crate::privacy::refresh(self);
        crate::lod::plan(self);
        // Those who go first when the tick's searches run over budget: the player's household
        // and whoever is watched (see `SEARCH_BUDGET`).
        let mut first = std::mem::take(&mut self.first);
        first.clear();
        first.extend((0..self.sims.len()).map(|i| {
            self.full_detail(i)
                || self
                    .households
                    .get(self.sims[i].household as usize)
                    .is_some_and(|h| h.player)
        }));
        let (search_mark, budget) = (self.search_mark, self.search_budget);
        let hour = crate::clock::hour(tick);
        let mut work = std::mem::take(&mut self.work);
        self.fill_work_waiting(&mut work);
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
                travel,
                cars,
                traffic,
                taxis,
                room_score,
                dirt,
                stairs,
                reach,
                nav_version,
                nav_log,
                shut,
                rooms,
                staff,
                paths,
                ..
            } = self;
            let base = Ctx {
                room_score,
                dirt,
                content,
                nav: NavGrid {
                    lot,
                    blocked,
                    stairs,
                    kept_off: &travel.junction_tiles,
                },
                reach,
                nav_version: *nav_version,
                nav_log,
                rels: relationships,
                briefs,
                households,
                plots,
                tile_plot,
                object_plot,
                index: object_index,
                exits,
                travel,
                cars,
                lights: &traffic.lights,
                taxis,
                autonomy: *autonomy,
                hour,
                day: crate::clock::day(tick),
                minute: crate::clock::minute_of_day(tick),
                tick,
                work: &work,
                shut,
                rooms,
                staff,
                paths,
                others: Others::default(),
            };
            // (In lots nobody watches, only those whose turn it is; see `lod.rs`.)
            for i in 0..sims.len() {
                let (before, rest) = sims.split_at_mut(i);
                let Some((sim, after)) = rest.split_first_mut() else {
                    break;
                };
                if !sim.here() || sim.steps == 0 {
                    continue;
                }
                let ctx = Ctx {
                    others: Others { before, after },
                    ..base
                };
                // The tick's searches over budget: one who may look for a way now waits for
                // the next tick, unless they go first or waited last tick.
                if first.get(sim.id as usize) == Some(&false)
                    && !sim.deferred
                    && crate::path::nodes_searched() - search_mark > budget
                    && may_search(sim, &ctx)
                {
                    sim.owed += sim.steps;
                    sim.steps = 0;
                    sim.deferred = true;
                    continue;
                }
                sim.deferred = false;
                step_sim(sim, &ctx, objects, items, rng);
            }
        }
        self.work = work;
        self.first = first;
        crate::partners::update(self);
        self.broadcast();
        self.apply_messes();
        crate::carry::after_step(self);
        crate::setup::update(self);
        self.wake_for_crying_babies();
        conversation::update(self);
        crate::privacy::after_step(self);
        crate::hosting::update(self);
        crate::phone::update(self);
        crate::life::update(self);
        crate::lifecycle::update(self);
        crate::traffic::begin(self);
        crate::cars::update(self);
        crate::taxi::update(self);
        crate::planner::update(self);
        crate::cooking::after_step(self);
        crate::lights::update(self);
        crate::lights::update_while(self);
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
        if tick.is_multiple_of(crate::machines::STEP_MINUTES * TICKS_PER_MINUTE) {
            crate::machines::run(self, crate::machines::STEP_MINUTES as f32);
        }
    }

    /// Gives nearby residents a share of what a use gives (content `broadcast`: hearing the TV
    /// from the sofa, or while eating at the table). Only positive gains carry, and not to the
    /// resident using it, who gets the whole of it.
    fn broadcast(&mut self) {
        let mut scratch = std::mem::take(&mut self.broadcast_scratch);
        self.broadcast_with(&mut scratch);
        self.broadcast_scratch = scratch;
    }

    /// `broadcast` with its buffers (`BroadcastScratch`) out of `self`.
    fn broadcast_with(&mut self, scratch: &mut BroadcastScratch) {
        let BroadcastScratch {
            sources,
            rooms,
            storeys,
            plots,
            asleep,
            heard,
        } = scratch;
        let content = &self.content;
        sources.clear();
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
            sources.push(BroadcastSource {
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
        rooms.clear();
        rooms.extend(self.sims.iter().map(|s| {
            let (x, z) = s.tile();
            let ground = self.ground_z(z);
            if self.lot.in_bounds(x, ground) {
                self.lot.room_at(x, ground)
            } else {
                crate::lot::OUTDOORS
            }
        }));
        storeys.clear();
        storeys.extend(self.sims.iter().map(|s| self.storey_of(s.tile().1)));
        plots.clear();
        plots.extend(self.sims.iter().map(|s| {
            let (x, z) = s.tile();
            self.plot_at(x, z)
        }));
        asleep.clear();
        asleep.extend(self.sims.iter().map(|s| s.asleep(content)));
        for (i, sim) in self.sims.iter_mut().enumerate() {
            if !sim.here() || sim.away_until.is_some() || sim.steps == 0 || asleep[i] {
                continue;
            }
            let minutes = sim.minutes();
            // Two residents watching the same screen are one source, not two.
            heard.clear();
            for src in sources.iter() {
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
            let Some(g) = self.content.objects[o.def].grows else {
                continue;
            };
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
        self.pile_laundry(minutes);
    }

    /// Dirty washing piles up over `minutes`: each household's, as much as content `laundry` says
    /// for each of its residents in town, up to the most it says.
    fn pile_laundry(&mut self, minutes: f32) {
        let Some(rules) = self.content.laundry else {
            return;
        };
        for s in self.sims.iter().filter(|s| s.here()) {
            if let Some(h) = self.households.get_mut(s.household as usize) {
                h.laundry[0] = (h.laundry[0] + rules.per_day * minutes / 1440.0).min(rules.most);
            }
        }
    }

    /// What each household has to work on (`WorkOn` bits, by household): dirty dishes or food
    /// gone off of theirs, a load of dirty washing, a load washed.
    pub(crate) fn work_waiting(&self) -> Vec<u8> {
        let mut work = Vec::new();
        self.fill_work_waiting(&mut work);
        work
    }

    /// `work_waiting`, into `work`.
    fn fill_work_waiting(&self, work: &mut Vec<u8>) {
        use crate::content::WorkOn;
        work.clear();
        work.extend(self.households.iter().map(|h| {
            let has = |kind: WorkOn, loads: f32| if loads >= 1.0 { kind.bit() } else { 0 };
            has(WorkOn::Laundry, h.laundry[0])
                | has(WorkOn::Washed, h.laundry[1])
                | has(WorkOn::Dried, h.laundry[2])
        }));
        for it in self
            .items
            .list
            .iter()
            .filter(|it| it.to_clear(&self.content))
        {
            if let Some(w) = work.get_mut(it.household as usize) {
                *w |= WorkOn::Dishes.bit();
            }
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
                | Command::SetNeed { .. }
                | Command::SetBond { .. }
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
        if let Err(e) = self.apply_command(cmd) {
            // (Nothing to keep for undo.)
            if let Some(s) = snapshot {
                self.lot.log_forget(s.lot_mark());
            }
            return Err(e);
        }
        self.catch_up_rooms();
        // Someone an order or an edit stopped (the sink they carried dishes to was moved) puts
        // down what they carry now, not when time moves on (Build mode stands still).
        // (The objects' index catches up as time moves on, if nobody has anything to put down.)
        if crate::cooking::must_settle(self) {
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
                    return Err(Error::new(format!(
                        "{} is there already",
                        self.sims[sim as usize].name
                    )));
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
                let h = self
                    .households
                    .get_mut(household as usize)
                    .ok_or_else(|| Error::new(format!("unknown household {household}")))?;
                h.funds = (h.funds + amount).max(0);
            }
            Command::SetNeed { sim, need, value } => {
                self.sim(sim)?;
                let n = self.content.needs.len();
                let which: Vec<usize> = if need == "all" {
                    (0..n).collect()
                } else {
                    vec![
                        self.content
                            .needs
                            .iter()
                            .position(|d| d.id == need)
                            .ok_or_else(|| Error::new(format!("unknown need '{need}'")))?,
                    ]
                };
                let value = if value.is_finite() {
                    value.clamp(0.0, 1.0)
                } else {
                    0.5
                };
                let sim = &mut self.sims[sim as usize];
                for k in which {
                    sim.needs[k] = value;
                }
                sim.planner.recheck = true;
            }
            Command::SetBond {
                a,
                b,
                friendship,
                romance,
                chemistry,
                partners,
            } => {
                self.sim(a)?;
                self.sim(b)?;
                if a == b {
                    return Err(Error::new("a bond is between two residents"));
                }
                let (a, b) = (a as usize, b as usize);
                // (Only those who may court: the game can't get anyone else there, so a check can't either.)
                let grown_ups =
                    self.sims[a].adult(&self.content) && self.sims[b].adult(&self.content);
                if !conversation::may_court(grown_ups, self.relationships.get(a, b))
                    && (romance.is_some_and(|f| f > 0.0) || partners == Some(true))
                {
                    return Err(Error::new(
                        "only grown-ups who aren't family can be partners or in love",
                    ));
                }
                let r = &mut self.relationships;
                for (x, y) in [(a, b), (b, a)] {
                    let rel = r.get_mut(x, y);
                    rel.met = true;
                    if let Some(f) = friendship.filter(|f| f.is_finite()) {
                        rel.friendship = f.clamp(-100.0, 100.0);
                    }
                    if let Some(f) = romance.filter(|f| f.is_finite()) {
                        rel.romance = f.clamp(-100.0, 100.0);
                    }
                }
                if let Some(c) = chemistry.filter(|c| c.is_finite()) {
                    r.set_chemistry(a, b, c);
                }
                if let Some(p) = partners {
                    // (One partner each: a new one ends what either had.)
                    if p {
                        for x in [a, b] {
                            if let Some(old) = r.partner_of(x).filter(|&o| o != a && o != b) {
                                r.set_partners(x, old, false);
                            }
                        }
                    }
                    r.set_partners(a, b, p);
                }
                self.sims[a].planner.recheck = true;
                self.sims[b].planner.recheck = true;
            }
            Command::SetSkill { sim, skill, level } => {
                self.sim(sim)?;
                let s = self
                    .content
                    .skill_index(&skill)
                    .ok_or_else(|| Error::new(format!("unknown skill '{skill}'")))?;
                let level = if level.is_finite() {
                    level.clamp(0.0, self.content.skill_rules.max_level)
                } else {
                    0.0
                };
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
                    return Err(Error::new(format!(
                        "the {} has no switch",
                        self.content.objects[obj.def].name.to_lowercase()
                    )));
                }
                crate::lights::set(self, object, on);
            }
            Command::Use {
                sim,
                object,
                interaction,
            } => {
                // (Who serves where, as the objects are now: `staff::closed`.)
                crate::staff::refresh(self);
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
                // Trying for a baby is asked of a partner (the social), not done at the bed alone.
                if inter.conceive {
                    return Err(Error::new(format!(
                        "{} is asked of a partner: click them",
                        inter.label
                    )));
                }
                if let Some(required) = missing_requirement(
                    &self.content,
                    &self.object_plot,
                    None,
                    &self.objects,
                    object,
                    inter,
                ) {
                    return Err(Error::new(format!(
                        "{} needs a working object on this lot that provides {required}",
                        inter.label
                    )));
                }
                if !obj.allows(inter, sim) {
                    let name = self.content.objects[obj.def].name.to_lowercase();
                    return Err(Error::new(match inter.when {
                        Some(w) if w != obj.open && obj.open => {
                            format!("the {name} is already open")
                        }
                        Some(w) if w != obj.open => format!("the {name} is closed"),
                        _ if inter.ripe && !obj.ripe() => format!("the {name} isn't ripe yet"),
                        _ if (inter.loads
                            || inter.unloads
                            || inter.work_on == Some(crate::content::WorkOn::Dishes))
                            && obj.running() =>
                        {
                            format!("the {name} is still running")
                        }
                        _ if inter.work_on == Some(crate::content::WorkOn::Dishes)
                            && obj.done() =>
                        {
                            format!("the {name} is done: empty it first")
                        }
                        _ if inter.loads && obj.done() => {
                            format!("the {name} is done: take the washing out first")
                        }
                        _ if inter.unloads && obj.run.is_none() => {
                            format!("there's nothing in the {name}")
                        }
                        _ => format!("someone is using the {name}"),
                    }));
                }
                // (Lying along the whole window seat: not with someone else on it.)
                if inter.whole && !obj.can_use_for(sim, &self.content.objects[obj.def], interaction)
                {
                    let name = self.content.objects[obj.def].name.to_lowercase();
                    return Err(Error::new(format!("someone is using the {name}")));
                }
                if inter.needs_table
                    && crate::surface::faced_table(
                        &self.content,
                        &self.objects,
                        &self.nav(),
                        object,
                    )
                    .is_none()
                {
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
                if allowed == Access::Guest && self.content.objects[obj.def].private {
                    return Err(Error::new(format!(
                        "{} is a guest there, and the {thing} is the household's own",
                        s.name
                    )));
                }
                if allowed == Access::Guest
                    && !guest_may(&self.content, &self.content.objects[obj.def], inter)
                {
                    return Err(Error::new(format!(
                        "{} is a guest there, and guests don't {}",
                        s.name,
                        inter.label.to_lowercase()
                    )));
                }
                // Who may use what, as free will keeps it: wine and the stove not for a child,
                // the kids' things not for grown-ups, homework for pupils, the baby's care for
                // grown-ups (and only with the baby there).
                if let Some(why) = unfit(&self.content, s.age, inter) {
                    return Err(Error::new(match why {
                        Unfit::TooYoung => {
                            format!("{} is too young to {}", s.name, inter.label.to_lowercase())
                        }
                        Unfit::TooBig => format!("{} is too big for the {thing}", s.name),
                        Unfit::NoHomework => format!("{} has no homework to do", s.name),
                        Unfit::TooYoungToCare => {
                            format!("{} is too young to look after a baby", s.name)
                        }
                    }));
                }
                // Its conditions, as free will keeps them: stargazing after dark under open sky,
                // washing up with dishes to wash.
                let work = self
                    .work_waiting()
                    .get(s.household as usize)
                    .copied()
                    .unwrap_or(0);
                if let Some(why) = unmet(
                    &self.lot,
                    crate::clock::hour(self.tick),
                    obj,
                    &self.content,
                    inter,
                    work,
                ) {
                    let label = inter.label.to_lowercase();
                    return Err(Error::new(match why {
                        Unmet::Hours => {
                            let [from, to] = inter.hours.unwrap_or_default();
                            format!(
                                "{} can only {label} from {} to {}",
                                s.name,
                                clock_time(from),
                                clock_time(to)
                            )
                        }
                        Unmet::Sky => format!(
                            "{} can only {label} under open sky: the {thing} has to stand outside",
                            s.name
                        ),
                        Unmet::Nothing(w) => format!("{}: {}", inter.label, w.none()),
                    }));
                }
                // Sold at a post outside its hours (or a coffee from a café that's shut): closed.
                // (As `closed_at`: the objects on its plot, none off the plots.)
                let plot = self.object_plot.get(object as usize).copied().flatten();
                let on: Vec<u32> = match plot {
                    Some(p) => self
                        .objects
                        .iter()
                        .filter(|o| {
                            self.object_plot.get(o.id as usize).copied().flatten() == Some(p)
                        })
                        .map(|o| o.id)
                        .collect(),
                    None => Vec::new(),
                };
                if let Some(post) = crate::staff::closed(
                    &self.content,
                    &self.staff,
                    &self.objects,
                    &on,
                    object,
                    inter,
                    crate::clock::hour(self.tick),
                ) {
                    let name = self.content.objects[self.objects[post as usize].def]
                        .name
                        .to_lowercase();
                    let opens = crate::staff::opens(&self.content, &self.objects, post)
                        .map_or(String::new(), |h| {
                            format!(": it opens at {}", clock_time(h))
                        });
                    return Err(Error::new(format!("the {name} is closed{opens}")));
                }
                // A game for two: its places free (or one taken by someone there for the same
                // game, to play along with), and someone free to play, or a use for one to do
                // instead (`partners.rs`).
                if inter.partner {
                    let def = &self.content.objects[obj.def];
                    let mut others = obj.users[def.slots_for(interaction)]
                        .iter()
                        .flatten()
                        .filter(|&&u| u != sim);
                    let (first, more) = (others.next().copied(), others.next().is_some());
                    let along = |u: u32| {
                        crate::partners::game(self, u)
                            .is_some_and(|(o, i, _)| (o, i) == (object, interaction))
                    };
                    if more || first.is_some_and(|u| !along(u)) {
                        return Err(Error::new(format!("someone is using the {thing}")));
                    }
                    if first.is_none()
                        && inter.alone.is_none()
                        && crate::partners::partner_for(self, sim as usize, object, interaction)
                            .is_none()
                    {
                        return Err(Error::new(format!(
                            "nobody is free to {} with {}",
                            inter.label.to_lowercase(),
                            s.name
                        )));
                    }
                }
                if inter.cares()
                    && !obj.users().any(|u| {
                        self.sims
                            .get(u as usize)
                            .is_some_and(|b| self.content.life.baby(b.age))
                    })
                {
                    return Err(Error::new(format!("there's no baby in the {thing}")));
                }
                if upper_barred(&self.content, obj, sim, s.age) {
                    let young = crate::content::all_older(
                        self.content.objects[obj.def].upper_stages,
                        self.content.life.stage(s.age),
                    );
                    return Err(Error::new(format!(
                        "{} is too {} for the top bunk",
                        s.name,
                        if young { "young" } else { "old" }
                    )));
                }
                let household = s.household as usize;
                if !affordable(inter.cost, self.households[household].funds) {
                    return Err(Error::new(format!(
                        "not enough money: {} costs {}",
                        inter.label, inter.cost
                    )));
                }
                // Something in a public place or at someone else's: not leaving a baby home alone.
                crate::life::check_cover_at(
                    self,
                    sim as usize,
                    self.object_plot.get(object as usize).copied().flatten(),
                )?;
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
                            return Err(Error::new(if it.spoiled {
                                "that's gone off"
                            } else {
                                "there's nothing left to eat"
                            }));
                        }
                        TaskKind::Eat {
                            item: Some(item),
                            seat: None,
                            plate: None,
                            stage: 0,
                        }
                    }
                    crate::command::ItemAction::Clear => {
                        // (With no sink, into the bin.)
                        let dishes = self.content.cooking.dishes | self.content.cooking.bin;
                        let home = self.households[household as usize].plot;
                        let sink = self.objects.iter().any(|o| {
                            self.content.objects[o.def].stations & dishes != 0
                                && !o.broken()
                                && self.object_plot[o.id as usize] == home
                        });
                        if !sink {
                            return Err(Error::new(
                                "there's nowhere to wash up: buy a sink, a dishwasher or a bin",
                            ));
                        }
                        TaskKind::Dishes {
                            item,
                            station: None,
                            stage: 0,
                            scraps: 0,
                        }
                    }
                };
                self.enqueue(sim, kind)?;
            }
            Command::MoveTo { sim, x, z } => {
                if !self.lot.in_bounds(x.floor() as i32, z.floor() as i32) {
                    return Err(Error::new("move target is off the lot"));
                }
                crate::life::check_cover_at(
                    self,
                    sim as usize,
                    self.plot_at(x.floor() as i32, z.floor() as i32),
                )?;
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
                // Trying for a baby needs a bed behind a shut door (`privacy.rs`).
                if def.success.effect == social::Effect::Conceive
                    && crate::privacy::beds_in_content(&self.content)
                    && crate::privacy::bed_in_world(self, sim as usize, target as usize).is_none()
                {
                    return Err(Error::new(format!(
                        "{} and {} have nowhere private to try for a baby: a double bed in a room of its own",
                        a.name, t.name
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
                    return Err(Error::new(format!(
                        "{} is too young to work",
                        self.sims[sim as usize].name
                    )));
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
                let goodbye = s
                    .visiting
                    .and_then(|v| crate::life::farewell(self, sim as usize, v.plot));
                if let Some((host, social)) =
                    goodbye.filter(|_| self.sims[sim as usize].queue.len() + 1 < MAX_QUEUE)
                {
                    let World {
                        sims,
                        content,
                        objects,
                        ..
                    } = self;
                    let s = &mut sims[sim as usize];
                    if s.current.as_ref().is_some_and(|a| !a.task.directed) {
                        end_activity(s, content, objects);
                    }
                    s.queue.push_back(Task {
                        kind: TaskKind::Social {
                            target: host as u32,
                            social,
                        },
                        directed: false,
                        routine: false,
                    });
                }
                self.enqueue(sim, TaskKind::GoHome)?;
            }
            Command::Phone {
                sim,
                action,
                contact,
            } => {
                self.sim(sim)?;
                crate::phone::check_order(self, sim as usize, action, contact)?;
                self.enqueue(
                    sim,
                    TaskKind::Phone {
                        action,
                        contact,
                        seat: None,
                        answer: crate::phone::Answer::Pending,
                    },
                )?;
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
                let turn = def
                    .map(|d| self.check_turn(d, turn.unwrap_or(0)))
                    .transpose()?;
                let rot = at.map_or(0, |a| a[2] as u8);
                let layout = Layout {
                    offset: offset.unwrap_or([0, 0]),
                    extent: size,
                    tall: tall.unwrap_or(0),
                };
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
                        let kept = if self.check_offset(d, rot, old.offset).is_ok() {
                            old.offset
                        } else {
                            [0, 0]
                        };
                        let wanted = Layout {
                            offset: offset.unwrap_or(kept),
                            extent: size.or(old.extent),
                            tall: tall.unwrap_or(old.tall),
                        };
                        Some(self.check_layout(d, rot, wanted)?)
                    }
                    _ => None,
                };
                let (offset, size, tall) = (
                    layout.map(|l| l.offset),
                    layout.and_then(|l| l.extent),
                    layout.map(|l| l.tall),
                );
                self.move_object_to(household, object, x, z, rot, turn, offset, size, tall)?;
            }
            Command::Recolor {
                household,
                object,
                colors,
            } => self.recolor(household, object, check_colors(colors)?)?,
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
            Command::BuildBlueprint {
                household,
                blueprint,
            } => self.build_blueprint(household, &blueprint)?,
            Command::MoveRoom {
                household,
                x,
                z,
                dx,
                dz,
            } => self.move_room(household, x, z, dx, dz)?,
            Command::BuildGarage {
                household,
                x0,
                z0,
                x1,
                z1,
                rot,
            } => self.build_garage(household, [x0, z0, x1, z1], rot)?,
            Command::SellGarage { household } => self.sell_garage(household)?,
            Command::BuildPorch {
                household,
                axis,
                x,
                z,
            } => self.build_porch(household, axis, x, z)?,
            Command::RemovePorch {
                household,
                axis,
                x,
                z,
            } => self.remove_porch(household, axis, x, z)?,
            Command::BuildPorchPiece {
                household,
                kind,
                x,
                z,
                w,
                d,
                rot,
            } => self.build_porch_piece(household, kind, x, z, w, d, rot)?,
            Command::RemovePorchPiece {
                household,
                kind,
                x,
                z,
            } => self.remove_porch_piece(household, kind, x, z)?,
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
                let offset = (s.current.is_some()
                    || s.engaged_with.is_some()
                    || s.heeding.is_some()) as usize;
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

// ---- the tick's helpers, split by what they do ---------------------------------------------

mod colors;
mod decide;
mod objects;
mod outings;
mod route;
mod tick;

pub use colors::*;
pub(crate) use decide::*;
pub(crate) use objects::*;
pub use outings::*;
pub(crate) use route::*;
pub use tick::*;

#[cfg(test)]
mod tests;
