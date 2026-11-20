//! What an object is: its tiles, size, layout and the state it keeps (open, worn, growing).

use std::f32::consts::FRAC_PI_2;

use super::*;

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
    /// Curtains (a thing over a window, content `overWindow`) drawn shut (see `curtains.rs`);
    /// bought open. Meaningless for other things.
    pub shut: bool,
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
    /// The plot it belongs to though it stands on another (a flat's letterbox in its building's
    /// hall, its bike in the yard: the town file's `objects[].plot`). None: the plot it stands on.
    pub keeper: Option<u32>,
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
pub(super) const GROWTH_STEP_MINUTES: u64 = 15;

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
    pub(super) fn closes_on_someone(&self, inter: &crate::content::Interaction, me: u32) -> bool {
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
    /// stable id, turn, wear, open or closed, on or off, curtains drawn or not, crop, and a
    /// machine's run.
    pub(crate) fn carry_state(&mut self, old: &ObjectInstance) {
        (
            self.stable,
            self.turn,
            self.wear,
            self.open,
            self.off,
            self.shut,
            self.growth,
            self.run,
            self.rack,
        ) = (
            old.stable, old.turn, old.wear, old.open, old.off, old.shut, old.growth, old.run,
            old.rack,
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
    pub(super) fn place_at(&self, def: &crate::content::ObjectDef, slot: usize) -> [f32; 2] {
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

    /// The tiles it stands on and reaches over (`tiles`, `overhang_tiles`) as a rectangle
    /// `[x0, z0, x1, z1)`.
    pub(crate) fn reach(&self, content: &Content) -> [i32; 4] {
        let (w, d) = self.size(content);
        let mut r = [self.x, self.z, self.x + w, self.z + d];
        for (x, z) in self.overhang_tiles(content) {
            r = [r[0].min(x), r[1].min(z), r[2].max(x + 1), r[3].max(z + 1)];
        }
        r
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
