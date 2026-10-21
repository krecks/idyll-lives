//! Build and buy: changing a household's home. Buying, selling, moving, restyling and
//! upgrading objects, and building walls, doors and windows.
//!
//! Edits are addressed to a household (it may have nobody living in it yet: build first, move
//! a family in later). Every change is checked against its home plot and paid from household
//! funds, except in Creative games, where building is free. Nothing
//! may cut Sims off from what they could reach before (an object's front tile, a room).
//! With diagonal walls the same rule holds: a tile crossed by a diagonal wall can't be stood
//! on, so walling across an object's front tile or around a resident is refused.

use std::collections::VecDeque;

use crate::Error;
use crate::command::{EdgeAxis, EdgeEdit, EdgeKind, FacePaint, FloorPaint};
use crate::content::Layer;
use crate::lot::{Diagonal, Edge, EdgeLook, EdgeRef, FORM_HALF, OUTDOORS};
use crate::path::NavGrid;
use crate::social::EventKind;
use crate::world::{GameMode, Layout, MAX_OFFSET, MAX_SHORTER, MAX_SIZE, MAX_TALLER, MIN_SIZE, ObjectInstance, TaskKind, World, end_activity, kind_size};

/// Tiles around a plot that still count when checking who can reach what (the path from the street).
const REACH_MARGIN: i32 = 2;
/// Build and buy edits kept for undo.
const UNDO_DEPTH: usize = 30;

/// The world as it was before a build or buy edit: everything an edit can change (walls,
/// objects and money, and residents stopped or moved by it). Time stands still while
/// edits are kept (see `World::apply`), so putting it back restores exactly that moment.
#[derive(Debug, Clone)]
pub(crate) struct HomeSnapshot {
    household: usize,
    lot: crate::lot::Lot,
    objects: Vec<ObjectInstance>,
    object_plot: Vec<Option<u32>>,
    blocked: Vec<bool>,
    sims: Vec<crate::world::Sim>,
    households: Vec<crate::world::Household>,
    plots: Vec<crate::world::Plot>,
    /// Food and dishes (they move and go with the furniture they're on).
    items: crate::items::Items,
    /// The story's latest event id when it was taken, and the events the edit after it told
    /// (an upgrade): undo takes them out of the story, redo tells them again.
    events_before: u64,
    story: Vec<crate::social::SocialEvent>,
}

fn edge_kind(kind: EdgeKind) -> Edge {
    match kind {
        EdgeKind::Wall => Edge::Wall,
        EdgeKind::Door => Edge::Door,
        EdgeKind::Window => Edge::Window,
        EdgeKind::Open => Edge::Open,
        EdgeKind::Fence => Edge::Fence,
        EdgeKind::Gate => Edge::Gate,
    }
}

/// Where an edit's wall is (after the bounds checks: coordinates are on the lot).
fn edge_ref(axis: EdgeAxis, x: i32, z: i32) -> EdgeRef {
    let (x, z) = (x as u16, z as u16);
    match axis {
        EdgeAxis::H => EdgeRef::H(x, z),
        EdgeAxis::V => EdgeRef::V(x, z),
        EdgeAxis::Dp | EdgeAxis::Dn => EdgeRef::Diag(x, z),
    }
}

/// Whether a task is at object `id` (using it, repairing it, cooking at it, sitting at it to
/// eat or to be on the phone, washing up at it): it stops when the object goes.
fn uses_object(t: &TaskKind, id: u32) -> bool {
    match *t {
        TaskKind::Use { object, .. } | TaskKind::Repair { object } => object == id,
        TaskKind::Cook { station, .. } | TaskKind::Dishes { station, .. } => station == Some(id),
        TaskKind::Eat { seat, .. } | TaskKind::Phone { seat, .. } => seat == Some(id),
        _ => false,
    }
}

/// Object `id` is gone: the task's object ids after it shift down by one (a recipe started
/// from it goes on without). Tasks hold indices into `World::objects` rather than stable ids,
/// so this stays even though player-facing ids no longer move.
fn shift_objects(t: &mut TaskKind, id: u32) {
    let shift = |o: &mut u32| {
        if *o > id {
            *o -= 1;
        }
    };
    match t {
        TaskKind::Use { object, .. } | TaskKind::Repair { object } => shift(object),
        TaskKind::Cook { station, origin, .. } => {
            if *origin == Some(id) {
                *origin = None;
            }
            station.iter_mut().chain(origin.iter_mut()).for_each(shift);
        }
        TaskKind::Dishes { station, .. } => station.iter_mut().for_each(shift),
        TaskKind::Eat { seat, .. } | TaskKind::Phone { seat, .. } => seat.iter_mut().for_each(shift),
        _ => {}
    }
}

impl World {
    /// The state a build or buy edit by `household` may change (None if it has no home).
    pub(crate) fn home_snapshot(&self, household: u32) -> Option<HomeSnapshot> {
        let (household, _) = self.home_of(household).ok()?;
        Some(HomeSnapshot {
            household,
            lot: self.lot.clone(),
            objects: self.objects.clone(),
            object_plot: self.object_plot.clone(),
            blocked: self.blocked.clone(),
            sims: self.sims.clone(),
            households: self.households.clone(),
            plots: self.plots.clone(),
            items: self.items.clone(),
            events_before: self.events.last_id(),
            story: Vec::new(),
        })
    }

    /// Keeps `before` for undo if the edit changed anything (an edit that changed nothing, like
    /// drawing a wall where one stands, leaves no step).
    pub(crate) fn remember(&mut self, mut before: HomeSnapshot) {
        let h = before.household;
        let lot_changed = before.lot != self.lot;
        if lot_changed {
            self.lot_version = self.lot_version.wrapping_add(1);
        }
        let changed = before.households[h].funds != self.households[h].funds
            || before.objects != self.objects
            || lot_changed
            || before.plots != self.plots;
        if !changed {
            return;
        }
        if self.undo.len() >= UNDO_DEPTH {
            self.undo.remove(0);
        }
        before.story = self.events.since(before.events_before);
        self.undo.push(before);
        self.redo.clear();
    }

    /// Build and buy edits the household can take back.
    pub fn undo_steps(&self, household: usize) -> usize {
        self.undo.iter().filter(|s| s.household == household).count()
    }

    /// Edits the household took back and can make again.
    pub fn redo_steps(&self, household: usize) -> usize {
        self.redo.iter().filter(|s| s.household == household).count()
    }

    /// Takes back the household's last build or buy edit: walls, furniture, money and residents
    /// are as they were before it (a sold object comes back where it stood, at its old price).
    /// It can be made again with `redo` until the next edit or until time moves on.
    pub fn undo(&mut self, household: u32) -> Result<(), Error> {
        let (h, _) = self.home_of(household)?;
        if self.undo.last().is_none_or(|s| s.household != h) {
            return Err(Error::new("nothing to undo"));
        }
        let mut now = self.home_snapshot(household).expect("the household has a home");
        let mut s = self.undo.pop().expect("checked above");
        // What the edit told goes from the story (and comes back with a redo).
        now.story = self.events.take(&s.story);
        self.keep_orders(&mut s);
        self.redo.push(now);
        self.restore(s);
        Ok(())
    }

    /// Makes the household's last undone edit again.
    pub fn redo(&mut self, household: u32) -> Result<(), Error> {
        let (h, _) = self.home_of(household)?;
        if self.redo.last().is_none_or(|s| s.household != h) {
            return Err(Error::new("nothing to redo"));
        }
        let mut now = self.home_snapshot(household).expect("the household has a home");
        let mut s = self.redo.pop().expect("checked above");
        now.story = self.events.put_back(std::mem::take(&mut s.story));
        self.keep_orders(&mut s);
        self.undo.push(now);
        self.restore(s);
        Ok(())
    }

    /// Undo and redo put back the home, money and residents as an edit left them, but not the
    /// orders given since (they don't end the edits that can be undone, see `World::apply`):
    /// goals and suggestions, routines, free will and an answered purchase request (with the
    /// feeling a no brought) stay as they are now.
    fn keep_orders(&self, s: &mut HomeSnapshot) {
        for (then, now) in s.sims.iter_mut().zip(&self.sims) {
            let (p, q) = (&mut then.planner, &now.planner);
            p.routines.clone_from(&q.routines);
            p.skip_household.clone_from(&q.skip_household);
            p.goals.clone_from(&q.goals);
            p.suggestions.clone_from(&q.suggestions);
            p.run.clone_from(&q.run);
            p.purchase = q.purchase;
            then.feelings.clone_from(&now.feelings);
            then.mods = now.mods.clone();
        }
        for (then, now) in s.households.iter_mut().zip(&self.households) {
            then.free_will = now.free_will;
            then.routines.clone_from(&now.routines);
        }
    }

    pub(crate) fn restore(&mut self, s: HomeSnapshot) {
        if s.lot != self.lot {
            self.lot_version = self.lot_version.wrapping_add(1);
        }
        self.lot = s.lot;
        self.objects = s.objects;
        self.object_plot = s.object_plot;
        self.blocked = s.blocked;
        self.sims = s.sims;
        self.households = s.households;
        self.plots = s.plots;
        // (Bumped past both, so views send them again.)
        let version = self.items.version.max(s.items.version).wrapping_add(1);
        self.items = s.items;
        self.items.version = version;
        self.refresh_stairs();
        self.nav_changed();
        self.rooms_layout_changed();
    }

    /// The roof over the household's home: a style and a colour (free; only for looks).
    pub fn set_roof(&mut self, household: u32, style: u8, color: u8) -> Result<(), Error> {
        let (_, plot) = self.home_of(household)?;
        let b = &self.content.build;
        if style as usize >= b.roofs.len().max(1) || color as usize >= b.roof_colors.len().max(1) {
            return Err(Error::new("unknown roof style or colour"));
        }
        self.plots[plot as usize].roof = Some(crate::world::RoofLook { style, color });
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }

    /// The household's index and its home plot.
    pub(crate) fn home_of(&self, household: u32) -> Result<(usize, u32), Error> {
        let h = household as usize;
        let plot = self
            .households
            .get(h)
            .ok_or_else(|| Error::new(format!("unknown household {household}")))?
            .plot
            .ok_or_else(|| Error::new("this household has no home"))?;
        Ok((h, plot))
    }

    /// What a build or buy edit costs in this game: nothing in Creative.
    pub fn build_price(&self, price: i64) -> i64 {
        match self.mode {
            GameMode::Living => price,
            GameMode::Creative => 0,
        }
    }

    /// Checks that household `h` can pay `cost` for an edit (free in Creative); `Ok` means
    /// the edit may go ahead and be paid with `pay`.
    pub(crate) fn can_pay(&self, h: usize, cost: i64, refusal: impl FnOnce() -> String) -> Result<(), Error> {
        let cost = self.build_price(cost);
        if cost > 0 && self.households[h].funds < cost {
            return Err(Error::new(refusal()));
        }
        Ok(())
    }

    pub(crate) fn pay(&mut self, h: usize, cost: i64) {
        self.households[h].funds -= self.build_price(cost);
    }

    /// An angle past the facing for object type `def`: only those that turn freely take one.
    pub(crate) fn check_turn(&self, def: usize, turn: u8) -> Result<u8, Error> {
        let d = &self.content.objects[def];
        if turn == 0 {
            Ok(0)
        } else if !d.turns {
            Err(Error::new(format!("the {} only faces four ways", d.name)))
        } else if turn >= 90 {
            Err(Error::new(format!("turn {turn}° is past the next facing")))
        } else {
            Ok(turn)
        }
    }

    /// How far object type `def`, facing `rot`, stands off the middle of its footprint: by up
    /// to half a tile, things on a wall only along it. Stairs stand square.
    pub(crate) fn check_offset(&self, def: usize, rot: u8, offset: [i8; 2]) -> Result<[i8; 2], Error> {
        let d = &self.content.objects[def];
        // Across the wall (z facing 0 or 2, x facing 1 or 3) a thing on it can't move.
        let across = if rot.is_multiple_of(2) { offset[1] } else { offset[0] };
        if offset == [0, 0] {
            Ok(offset)
        } else if d.stairs {
            Err(Error::new(format!("the {} stands square on its tiles", d.name.to_lowercase())))
        } else if d.layer == Layer::Wall && across != 0 {
            Err(Error::new("that hangs flat on its wall"))
        } else if offset.iter().any(|&o| o.unsigned_abs() > MAX_OFFSET as u8) {
            Err(Error::new("that's off its tile"))
        } else {
            Ok(offset)
        }
    }

    /// A size for object type `def` (centimetres across and deep, before turning, and
    /// centimetres taller): each side `MIN_SIZE` to `MAX_SIZE` tiles. Stairs keep theirs; things
    /// on a wall keep their depth and things on a top their footprint (they can still be taller
    /// or shorter).
    pub(crate) fn check_size(&self, def: usize, size: [u16; 2], tall: i16) -> Result<([u16; 2], i16), Error> {
        let d = &self.content.objects[def];
        let name = d.name.to_lowercase();
        let own = kind_size(d.footprint);
        if d.stairs && (size != own || tall != 0) {
            Err(Error::new(format!("the {name} keep their size")))
        } else if d.layer == Layer::Surface && size != own {
            Err(Error::new(format!("the {name} only gets taller or shorter")))
        } else if d.layer == Layer::Wall && size[1] != own[1] {
            Err(Error::new(format!("the {name} hangs flat on its wall")))
        } else if size.iter().any(|&s| !(MIN_SIZE..=MAX_SIZE as u16 * 100).contains(&s)) {
            Err(Error::new(format!("things measure {MIN_SIZE} cm to {MAX_SIZE} m each way")))
        } else if !(-MAX_SHORTER..=MAX_TALLER).contains(&tall) {
            Err(Error::new(format!("things grow {} m at most, or shrink {} m", MAX_TALLER as f32 / 100.0, MAX_SHORTER as f32 / 100.0)))
        } else {
            Ok((size, tall))
        }
    }

    /// `layout` for object type `def` facing `rot`, checked (`check_offset`, `check_size`).
    pub(crate) fn check_layout(&self, def: usize, rot: u8, layout: Layout) -> Result<Layout, Error> {
        let offset = self.check_offset(def, rot, layout.offset)?;
        let (size, tall) = self.check_size(def, layout.extent.unwrap_or(kind_size(self.content.objects[def].footprint)), layout.tall)?;
        Ok(Layout { offset, extent: Some(size), tall })
    }

    fn home_object(&self, household: u32, object: u32) -> Result<(usize, u32), Error> {
        let (h, plot) = self.home_of(household)?;
        if object as usize >= self.objects.len() {
            return Err(Error::new(format!("unknown object {object}")));
        }
        if self.object_plot[object as usize] != Some(plot) {
            return Err(Error::new("that isn't at your home"));
        }
        Ok((h, plot))
    }

    /// Whether every tile of `obj` and the tile in front of it lie on `plot` (nobody uses things
    /// without interactions, like tables, or things standing on a top: their front can be off it).
    fn fits_on_plot(&self, obj: &ObjectInstance, plot: u32) -> bool {
        let p = &self.plots[plot as usize];
        let def = &self.content.objects[obj.def];
        let (fx, fz) = obj.front_tile(&self.content);
        let on = |x: i32, z: i32| p.contains(x, self.ground_z(z));
        let used = !def.interactions.is_empty() && def.layer != Layer::Surface;
        (!used || on(fx, fz)) && obj.tiles(&self.content).all(|(x, z)| on(x, z))
    }

    /// Tiles reachable on foot from the plot's entry (a mask over the whole lot).
    pub(crate) fn reachable(&self, plot: u32) -> Vec<bool> {
        let lot = &self.lot;
        let nav = NavGrid {
            lot,
            blocked: &self.blocked,
            stairs: &self.stairs,
        };
        let p = &self.plots[plot as usize];
        let inside = |x: i32, z: i32| {
            let z = self.ground_z(z);
            x >= p.x - REACH_MARGIN
                && z >= p.z - REACH_MARGIN
                && x < p.x + p.w + REACH_MARGIN
                && z < p.z + p.d + REACH_MARGIN
        };
        let mut seen = vec![false; lot.width * lot.depth];
        let entry = p.arrival_point();
        let start = (entry[0].floor() as i32, entry[1].floor() as i32);
        if !nav.tile_free(start.0, start.1) {
            return seen;
        }
        let mut open = VecDeque::from([start]);
        seen[lot.tile_index(start.0, start.1)] = true;
        while let Some((x, z)) = open.pop_front() {
            let steps = [(1, 0), (-1, 0), (0, 1), (0, -1)].map(|(dx, dz)| (x + dx, z + dz));
            let up_or_down = self.stairs.link(x, z).filter(|&(lx, lz)| nav.tile_free(lx, lz));
            for (nx, nz) in steps.into_iter().filter(|&(nx, nz)| nav.can_step(x, z, nx, nz)).chain(up_or_down) {
                if inside(nx, nz) && !std::mem::replace(&mut seen[lot.tile_index(nx, nz)], true) {
                    open.push_back((nx, nz));
                }
            }
        }
        seen
    }

    /// Front tiles and empty tiles of `plot` that can be reached, as `(tile index)` list.
    fn reach_signature(&self, plot: u32) -> (Vec<bool>, Vec<bool>) {
        let seen = self.reachable(plot);
        let fronts = self
            .objects
            .iter()
            .map(|o| {
                let def = &self.content.objects[o.def];
                // Nobody needs to reach a picture, a rug, a table nobody uses or what stands on a
                // top; chairs pulled up to a table, from any side.
                self.object_plot[o.id as usize] != Some(plot)
                    || !def.reached()
                    || self.approach_tiles(o).iter().any(|&(fx, fz)| self.lot.in_bounds(fx, fz) && seen[self.lot.tile_index(fx, fz)])
            })
            .collect();
        (fronts, seen)
    }

    /// Whether every object and Sim that could be reached `before` still can. Empty floor may
    /// be closed off (draw a room's walls first, then add its door).
    fn keeps_reach(&self, plot: u32, before: &(Vec<bool>, Vec<bool>), skip: Option<u32>) -> bool {
        let after = self.reach_signature(plot);
        let p = &self.plots[plot as usize];
        let objects_ok = before
            .0
            .iter()
            .zip(&after.0)
            .enumerate()
            .all(|(i, (b, a))| Some(i as u32) == skip || !*b || *a);
        // No Sim at home gets shut in.
        let sims_ok = self.sims.iter().all(|s| {
            let (x, z) = s.tile();
            let gz = self.ground_z(z);
            let on_plot = x >= p.x && gz >= p.z && x < p.x + p.w && gz < p.z + p.d;
            let i = self.lot.tile_index(x, z);
            s.away_until.is_some() || !on_plot || !before.1[i] || after.1[i]
        });
        objects_ok && sims_ok
    }

    /// Whether a resident stands where `obj` would go (or would stand up there from a seat).
    fn sim_in_the_way(&self, obj: &ObjectInstance) -> bool {
        self.sims.iter().any(|s| {
            let seated = s.seated_from;
            s.away_until.is_none() && obj.tiles(&self.content).any(|t| t == s.tile() || Some(t) == seated)
        })
    }

    /// Places object `def` on the home plot (laid out as `layout`) if it fits there and blocks
    /// nothing.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn place_at_home(
        &mut self,
        plot: u32,
        def: usize,
        x: i32,
        z: i32,
        rot: u8,
        layout: Layout,
        colors: crate::world::Colors,
        quality: u8,
        value: i64,
    ) -> Result<u32, Error> {
        let obj = self.check_fit(def, x, z, rot, layout).map_err(|e| {
            let e = e.to_string();
            // The storeys' rules say what's wrong in words already (`check_storey_fit`).
            if !e.starts_with('\'') {
                return Error::new(e);
            }
            // Why it doesn't fit beside something else, as it says it.
            for why in ["would hang into ", "would hang too low over ", "is too tall for ", "is under ", "can't hang over a window"] {
                if let Some(i) = e.find(why) {
                    return Error::new(format!("that {}", &e[i..]));
                }
            }
            Error::new(if e.contains("diagonal") {
                "furniture can't stand on a tile with a diagonal wall"
            } else if e.contains("no wall to hang on") {
                "that hangs on a wall: put its back to one"
            } else if e.contains("needs a ceiling") {
                "that hangs from a ceiling: put it in a room"
            } else if e.contains("nothing to stand on") {
                "that goes on a table, a counter, a desk or a shelf"
            } else if e.contains("over the edge") {
                "that would hang over the edge"
            } else {
                "there's no room for that here"
            })
        })?;
        if !self.fits_on_plot(&obj, plot) {
            return Err(Error::new("that has to go on your own lot"));
        }
        if self.content.objects[def].outdoors && !self.all_outdoors(&obj) {
            return Err(Error::new("that has to go outside"));
        }
        // Residents walk under pictures and lamps and over rugs (not into a plant hanging low).
        let d = &self.content.objects[def];
        let mounted = d.layer.mounted() && !d.hangs_low;
        if !mounted && self.sim_in_the_way(&obj) {
            return Err(Error::new("someone is standing there"));
        }
        let before = self.reach_signature(plot);
        let id = self.place(def, x, z, rot, layout, colors, quality, value, None)?;
        let approach = self.approach_tiles(&self.objects[id as usize]);
        let (fx, fz) = approach.first().copied().unwrap_or_else(|| self.objects[id as usize].front_tile(&self.content));
        let seen = self.reachable(plot);
        // Upstairs before there are stairs, nothing in the room can be reached yet anyway.
        let cut_off = || {
            let room = self.lot.room_at(fx, fz);
            let p = &self.plots[plot as usize];
            let k = self.storey_row(self.storey_of(fz));
            room != OUTDOORS
                && !(p.z + k..p.z + p.d + k).any(|z| (p.x..p.x + p.w).any(|x| self.lot.room_at(x, z) == room && seen[self.lot.tile_index(x, z)]))
        };
        // A picture, a rug, a table nobody uses or a thing on a top can go anywhere (over a bed,
        // behind a sofa, with chairs all round); a chair pulled up to a table needs one free side.
        let unused = !self.content.objects[def].reached();
        let reached = approach.iter().any(|&(ax, az)| self.lot.in_bounds(ax, az) && seen[self.lot.tile_index(ax, az)]);
        let front_ok = unused || reached || (!approach.is_empty() && self.storey_of(fz) != 0 && cut_off());
        if !front_ok || !self.keeps_reach(plot, &before, Some(id)) {
            self.remove_object(id);
            return Err(Error::new("that would block the way"));
        }
        Ok(id)
    }

    /// Places `old`, lifted out, back on the home plot at `x`, `z` (see `place_at_home`), keeping
    /// what it was (`ObjectInstance::carry_state`).
    pub(crate) fn replace_at_home(&mut self, plot: u32, old: &ObjectInstance, x: i32, z: i32, rot: u8, layout: Layout) -> Result<u32, Error> {
        let id = self.place_at_home(plot, old.def, x, z, rot, layout, old.colors.clone(), old.quality, old.value)?;
        self.objects[id as usize].carry_state(old);
        Ok(id)
    }

    /// Whether every tile of `obj` is outdoors (a tile split by a diagonal wall counts as indoors
    /// when either half is).
    fn all_outdoors(&self, obj: &ObjectInstance) -> bool {
        obj.tiles(&self.content)
            .all(|(x, z)| self.lot.room_at(x, z) == OUTDOORS)
    }

    /// Whether a tile has a door on one of its sides.
    fn by_door(&self, x: i32, z: i32) -> bool {
        if !self.lot.in_bounds(x, z) {
            return false;
        }
        let (ux, uz, lot) = (x as usize, z as usize, &self.lot);
        lot.h_edge(ux, uz) == Edge::Door
            || lot.h_edge(ux, uz + 1) == Edge::Door
            || lot.v_edge(ux, uz) == Edge::Door
            || lot.v_edge(ux + 1, uz) == Edge::Door
    }

    /// Furniture on the floor (not pictures, ceiling lamps, rugs, things on tops or stairs)
    /// doesn't go just inside a door, where it would make the way through a squeeze: a lamp
    /// nobody needs to reach could otherwise stand anywhere that leaves the rest reachable.
    fn keep_doorway_clear(&self, def: usize, x: i32, z: i32, rot: u8, layout: Layout) -> Result<(), Error> {
        let d = &self.content.objects[def];
        if (d.layer.mounted() && !d.hangs_low) || d.stairs {
            return Ok(());
        }
        // Where it doesn't fit at all, `place_at_home` says why.
        let Ok(obj) = self.check_fit(def, x, z, rot, layout) else {
            return Ok(());
        };
        if obj.tiles(&self.content).any(|(tx, tz)| self.by_door(tx, tz)) {
            return Err(Error::new("that would stand in a doorway"));
        }
        Ok(())
    }

    /// Good free spots for `def` on the plot, best first: indoors (outdoors for garden things),
    /// back to a wall, away from doors. Things on a surface go on the first top with room for
    /// them, in the middle if it's free.
    fn find_spot(&self, plot: u32, def: usize) -> Vec<(i32, i32, u8, [i8; 2])> {
        if self.content.objects[def].layer == Layer::Surface {
            let radius = self.content.objects[def].radius;
            return self
                .objects
                .iter()
                .filter(|o| self.object_plot[o.id as usize] == Some(plot))
                .filter_map(|o| {
                    let c = crate::surface::middle(&self.content, o);
                    crate::surface::free_surface_spot_near(&self.content, &self.objects, &[o.id], c, f32::INFINITY, 0, radius, &[])
                })
                .map(|s| (s.x, s.z, 0, s.offset))
                .collect();
        }
        let p = &self.plots[plot as usize];
        let lot = &self.lot;
        let mut spots = Vec::new();
        for z in p.z..p.z + p.d {
            for x in p.x..p.x + p.w {
                for rot in 0..4u8 {
                    let Ok(obj) = self.check_fit(def, x, z, rot, Layout::default()) else {
                        continue;
                    };
                    if !self.fits_on_plot(&obj, plot)
                        || obj.tiles(&self.content).any(|(tx, tz)| self.by_door(tx, tz))
                    {
                        continue;
                    }
                    let (w, d) = obj.size(&self.content);
                    let wall = |e: Edge| e.blocks();
                    let back_to_wall = match rot {
                        0 => (x..x + w).all(|tx| wall(lot.h_edge(tx as usize, z as usize))),
                        2 => (x..x + w).all(|tx| wall(lot.h_edge(tx as usize, (z + d) as usize))),
                        1 => (z..z + d).all(|tz| wall(lot.v_edge(x as usize, tz as usize))),
                        _ => (z..z + d).all(|tz| wall(lot.v_edge((x + w) as usize, tz as usize))),
                    };
                    let indoors = lot.room_at(x, z) != OUTDOORS;
                    let wanted = indoors != self.content.objects[def].outdoors;
                    let score = wanted as i32 * 10 + back_to_wall as i32 * 5;
                    spots.push((score, x, z, rot));
                }
            }
        }
        spots.sort_by_key(|s| -s.0);
        spots.into_iter().map(|(_, x, z, r)| (x, z, r, [0, 0])).collect()
    }

    /// Buys an object for the household's home. Returns the new object's id.
    pub fn buy(
        &mut self,
        household: u32,
        def_id: &str,
        at: Option<[i32; 3]>,
        colors: crate::world::Colors,
    ) -> Result<u32, Error> {
        self.buy_at(household, def_id, at, colors, Layout::default())
    }

    /// Buys an object for the household's home, laid out as `layout` (unchecked: see
    /// `check_layout`; its offset is where it goes on a top, for things on a surface). Without
    /// `at`, it goes at the first good free spot (the offset kept unless it stands on a
    /// surface). Returns the new object's id.
    pub fn buy_at(
        &mut self,
        household: u32,
        def_id: &str,
        at: Option<[i32; 3]>,
        colors: crate::world::Colors,
        layout: Layout,
    ) -> Result<u32, Error> {
        let (h, plot) = self.home_of(household)?;
        let def = self
            .content
            .object_index(def_id)
            .ok_or_else(|| Error::new(format!("unknown object '{def_id}'")))?;
        let d = &self.content.objects[def];
        let price = d
            .price
            .ok_or_else(|| Error::new(format!("{} isn't for sale", d.name)))?;
        self.can_pay(h, price, || format!("not enough money for the {}", d.name))?;
        let on_top = d.layer == Layer::Surface;
        let id = match at {
            Some([x, z, rot]) => {
                self.keep_doorway_clear(def, x, z, rot as u8, layout)?;
                self.place_at_home(plot, def, x, z, rot as u8, layout, colors, 0, price)?
            }
            None => self
                .find_spot(plot, def)
                .into_iter()
                .find_map(|(x, z, rot, found)| {
                    let layout = if on_top { Layout::at(found) } else { layout };
                    self.place_at_home(plot, def, x, z, rot, layout, colors.clone(), 0, price).ok()
                })
                .ok_or_else(|| Error::new("there's no free spot for that at home"))?,
        };
        // A crop bought new is just planted: it ripens over its days.
        if self.content.objects[def].grows.is_some() {
            self.objects[id as usize].growth = 0.0;
        }
        self.pay(h, price);
        Ok(id)
    }

    /// Puts `def` into the household's home at the first good free spot, free of charge (a
    /// delivery: a crib for a new baby). None if there's no room.
    pub(crate) fn deliver(&mut self, household: u32, def: usize) -> Option<u32> {
        let (_, plot) = self.home_of(household).ok()?;
        let value = self.content.objects[def].price.unwrap_or(0);
        self.find_spot(plot, def)
            .into_iter()
            .find_map(|(x, z, rot, offset)| self.place_at_home(plot, def, x, z, rot, Layout::at(offset), Default::default(), 0, value).ok())
    }

    /// Sells an object at home for part of what was spent on it (nothing in Creative). What
    /// stands on its top is sold with it.
    pub(crate) fn sell(&mut self, household: u32, object: u32) -> Result<(), Error> {
        let (h, _) = self.home_object(household, object)?;
        let obj = &self.objects[object as usize];
        if self.content.objects[obj.def].price.is_none() {
            return Err(Error::new("that can't be sold"));
        }
        let mut sold = crate::surface::riders(&self.content, &self.objects, object);
        sold.push(object);
        sold.sort_unstable();
        // Food and dishes on it (or kept in it) go too, not what's on the table under it.
        let held: Vec<u32> = self.items_held_by(&sold).into_iter().map(|(item, _)| item).collect();
        self.drop_items(&held);
        let mut refund = 0;
        // Highest id first: the ids before it stay put.
        for &id in sold.iter().rev() {
            let obj = &self.objects[id as usize];
            // Something broken brings back half.
            let broken = if obj.broken() { 0.5 } else { 1.0 };
            refund += (obj.value as f32 * self.content.object_rules.resale * broken).round() as i64;
            self.remove_object(id);
        }
        self.households[h].funds += self.build_price(refund);
        Ok(())
    }

    /// Moves or rotates an object at home. Its look, quality and value stay. Returns its new id
    /// (it's placed again at the end of the list, so the ids after it shift down).
    pub fn move_object(
        &mut self,
        household: u32,
        object: u32,
        x: i32,
        z: i32,
        rot: u8,
    ) -> Result<u32, Error> {
        self.move_object_to(household, object, x, z, rot, None, None, None, None)
    }

    /// Moves, rotates or resizes an object at home, with a new free turn, offset, size
    /// (centimetres across and deep, before turning) and height (centimetres taller) if given, else it keeps
    /// its own (all unchecked: see `check_turn`, `check_layout`). Things standing on its top go
    /// with it, turned with it. Its look, quality and value stay. Returns its new id: it's placed again at the end of the list (its riders
    /// after it), so the ids after it shift down. If anything doesn't fit, nothing moves.
    #[allow(clippy::too_many_arguments)]
    pub fn move_object_to(
        &mut self,
        household: u32,
        object: u32,
        x: i32,
        z: i32,
        rot: u8,
        turn: Option<u8>,
        offset: Option<[i8; 2]>,
        size: Option<[u16; 2]>,
        tall: Option<i16>,
    ) -> Result<u32, Error> {
        let (_, plot) = self.home_object(household, object)?;
        let before = self.home_snapshot(household).expect("the household has a home");
        let riders = crate::surface::riders(&self.content, &self.objects, object);
        let old_top = crate::surface::Top::of(&self.content, &self.objects[object as usize]);
        // Lift it and its riders out, highest id first (the ids before each stay put).
        let mut ids = riders.clone();
        ids.push(object);
        ids.sort_unstable();
        // Food and dishes on or in them (not what's on the table under it) come along.
        let held = self.items_held_by(&ids);
        let mut lifted: Vec<ObjectInstance> = ids.iter().rev().map(|&id| self.remove_object(id)).collect();
        lifted.reverse();
        let old = lifted.iter().find(|o| o.id == object).expect("lifted it").clone();
        let result = (|| {
            let layout = Layout { offset: offset.unwrap_or(old.offset), extent: Some(size.unwrap_or(old.extent)), tall: tall.unwrap_or(old.tall) };
            self.keep_doorway_clear(old.def, x, z, rot, layout)?;
            let id = self.replace_at_home(plot, &old, x, z, rot, layout)?;
            self.objects[id as usize].turn = turn.unwrap_or(old.turn);
            let new_top = crate::surface::Top::of(&self.content, &self.objects[id as usize]);
            let turned = |o: &ObjectInstance| (o.rot as i32 * 90 + o.turn as i32).rem_euclid(360);
            let delta = turned(&self.objects[id as usize]) - turned(&old);
            let mut placed = vec![(old.clone(), id)];
            for r in lifted.iter().filter(|o| o.id != object) {
                let (x, z, offset) = match (old_top, new_top) {
                    (Some(a), Some(b)) => {
                        // Where it stood on the old top, on the new one.
                        let p = b.world(a.local(crate::surface::middle(&self.content, r)));
                        let spot = crate::surface::SurfaceSpot::at(id, p, b.surface.height);
                        (spot.x, spot.z, spot.offset)
                    }
                    _ => (r.x, r.z, r.offset),
                };
                let facing = (turned(r) + delta).rem_euclid(360);
                let name = self.content.objects[r.def].name.to_lowercase();
                let rid = self
                    .replace_at_home(plot, r, x, z, (facing / 90) as u8, Layout { offset, ..Layout::of(r) })
                    .map_err(|_| Error::new(format!("the {name} on it wouldn't fit there")))?;
                self.objects[rid as usize].turn = if self.content.objects[r.def].turns { (facing % 90) as u8 } else { 0 };
                placed.push((r.clone(), rid));
            }
            for (was, now) in placed {
                let items: Vec<u32> = held.iter().filter(|&&(_, holder)| holder == was.id).map(|&(item, _)| item).collect();
                if !items.is_empty() {
                    let new = self.objects[now as usize].clone();
                    self.carry_items(&was, &new, &items);
                }
            }
            Ok(id)
        })();
        if result.is_err() {
            self.restore(before);
        }
        result
    }

    pub(crate) fn recolor(&mut self, household: u32, object: u32, colors: crate::world::Colors) -> Result<(), Error> {
        self.home_object(household, object)?;
        self.objects[object as usize].colors = colors;
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }

    /// A quick fix in Buy mode: as good as new, for a share of the price times how worn it is
    /// (free in Creative).
    pub(crate) fn repair(&mut self, household: u32, object: u32) -> Result<(), Error> {
        let (h, _) = self.home_object(household, object)?;
        let obj = &self.objects[object as usize];
        let def = &self.content.objects[obj.def];
        if obj.wear <= 0.0 {
            return Err(Error::new(format!("the {} is as good as new", def.name)));
        }
        let cost = repair_cost(&self.content, obj);
        let name = def.name.clone();
        self.can_pay(h, cost, || format!("not enough money to repair the {name}"))?;
        self.pay(h, cost);
        self.objects[object as usize].wear = 0.0;
        self.structure_version = self.structure_version.wrapping_add(1);
        self.rooms_need_rescore();
        Ok(())
    }

    /// Buys the next quality level for an object at home (Buy mode, instant). Returns the new quality.
    pub(crate) fn upgrade(&mut self, household: u32, object: u32) -> Result<u8, Error> {
        let (h, _) = self.home_object(household, object)?;
        let rules = &self.content.object_rules;
        let obj = &self.objects[object as usize];
        let def = &self.content.objects[obj.def];
        let price = def
            .price
            .ok_or_else(|| Error::new(format!("the {} can't be upgraded", def.name)))?;
        if obj.quality >= rules.max_quality {
            return Err(Error::new(format!(
                "the {} is as good as it gets",
                def.name
            )));
        }
        let cost = rules.upgrade_price(price);
        let name = def.name.clone();
        self.can_pay(h, cost, || format!("not enough money to upgrade the {name}"))?;
        self.pay(h, cost);
        let obj = &mut self.objects[object as usize];
        obj.quality += 1;
        obj.value += cost;
        let quality = obj.quality;
        self.structure_version = self.structure_version.wrapping_add(1);
        self.rooms_need_rescore();
        // Told about the household's first resident (nobody may live there yet).
        if let Some(member) = self.sims.iter().position(|s| s.here() && s.household as usize == h) {
            self.events.push_detail(
                self.tick,
                EventKind::Upgraded,
                member,
                Some(quality as i64),
                None,
            );
        }
        Ok(quality)
    }

    /// Changes wall edges on the home plot, paying per changed edge.
    ///
    /// As in classic life-simulation games, doors and windows go into walls: their edge must already
    /// have a wall (plain, or with a door or window to replace), unless the same edit draws the
    /// wall around it (a wall edge next to it on the same line).
    ///
    /// Diagonal walls (`dp` / `dn`) cross one tile corner to corner and cost
    /// [`BuildRules::diagonal_wall`](crate::content::BuildRules::diagonal_wall). A tile holds at
    /// most one diagonal; it can't go across furniture or a resident, and furniture never
    /// stands on a tile with a diagonal. Doors and windows go into diagonal walls too.
    ///
    /// Looks: a new wall takes the edit's `form` (full or half height) and `cover` (both faces);
    /// a new door or window its `style`. On a standing wall, a different `form` rebuilds it (the
    /// wall price); on a standing door or window, a different `style` replaces it (its price).
    /// Doors and windows only go into full-height walls. Faces are re-covered with [`World::paint`].
    pub fn build(&mut self, household: u32, edits: &[EdgeEdit]) -> Result<(), Error> {
        let (h, plot) = self.home_of(household)?;
        let edge_at = |lot: &crate::lot::Lot, axis: EdgeAxis, x: i32, z: i32| match axis {
            EdgeAxis::H => lot.h_edge(x as usize, z as usize),
            EdgeAxis::V => lot.v_edge(x as usize, z as usize),
            // What stands across the tile in this direction (a diagonal the other way is
            // reported separately).
            EdgeAxis::Dp | EdgeAxis::Dn => lot
                .diag(x, z)
                .filter(|d| Some(d.dir) == axis.diagonal())
                .map_or(Edge::Open, |d| d.edge),
        };
        // Tiles on either side of an edge (the tile itself, twice, for a diagonal).
        let sides = |e: &EdgeEdit| match e.axis {
            EdgeAxis::H => ((e.x, e.z - 1), (e.x, e.z)),
            EdgeAxis::V => ((e.x - 1, e.z), (e.x, e.z)),
            EdgeAxis::Dp | EdgeAxis::Dn => ((e.x, e.z), (e.x, e.z)),
        };
        let object_at = |x: i32, z: i32| {
            self.objects.iter().position(|o| {
                let (w, d) = o.size(&self.content);
                x >= o.x && z >= o.z && x < o.x + w && z < o.z + d
            })
        };
        // A wall drawn by this edit on the same line, right next to `e`.
        let walled_beside = |e: &EdgeEdit| {
            let (dx, dz) = match e.axis {
                EdgeAxis::H => (1, 0),
                EdgeAxis::V => (0, 1),
                EdgeAxis::Dp => (1, 1),
                EdgeAxis::Dn => (1, -1),
            };
            edits.iter().any(|o| {
                o.kind == EdgeKind::Wall
                    && o.axis == e.axis
                    && ((o.x == e.x + dx && o.z == e.z + dz)
                        || (o.x == e.x - dx && o.z == e.z - dz))
            })
        };
        let rules = &self.content.build;
        let mut cost = 0;
        // The look each changed wall ends up with (applied once everything checks out).
        let mut looks: Vec<(EdgeRef, EdgeLook)> = Vec::new();
        for e in edits {
            if !self.edge_on_plot(plot, e.axis, e.x, e.z) {
                return Err(Error::new("you can only build on your own lot"));
            }
            check_look(rules, e)?;
            let target = edge_kind(e.kind);
            let diagonal = e.axis.diagonal();
            if target.is_fence() && diagonal.is_some() {
                return Err(Error::new("fences and gates run along the grid"));
            }
            let (a, b) = sides(e);
            if matches!(target, Edge::Window | Edge::Fence | Edge::Gate) && (self.storey_of(a.1) < 0 || self.storey_of(b.1) < 0) {
                return Err(Error::new("a basement is underground: no windows, fences or gates down there"));
            }
            if e.kind != EdgeKind::Open
                && let Some(dir) = diagonal
                && (object_at(a.0, a.1).is_some() || self.objects.iter().any(|o| o.body_crosses(&self.content, e.x, e.z, dir)))
            {
                return Err(Error::new("a wall can't go through furniture"));
            }
            if e.kind != EdgeKind::Open
                && diagonal.is_none()
                && self.objects.iter().any(|o| {
                    o.wall_edges(&self.content)
                        .chain(o.body_edges(&self.content))
                        .any(|(p, q)| (p, q) == (a, b) || (q, p) == (a, b))
                })
            {
                return Err(Error::new(
                    "a wall can't go through furniture or in front of it",
                ));
            }
            if !matches!(target, Edge::Wall | Edge::Window)
                && diagonal.is_none()
                && self.hangs_on(e.axis == EdgeAxis::H, e.x, e.z)
            {
                return Err(Error::new("something hangs on that wall: move it first"));
            }
            // Only curtains and the like hang over a window.
            if target == Edge::Window
                && diagonal.is_none()
                && let Some(o) = self.hung_on(e.axis == EdgeAxis::H, e.x, e.z).find(|o| !self.content.objects[o.def].over_window)
            {
                let name = self.content.objects[o.def].name.to_lowercase();
                return Err(Error::new(format!("the {name} hangs there, and it can't hang over a window: move it first")));
            }
            if let Some(dir) = diagonal
                && target != Edge::Open
            {
                if self.lot.diag(e.x, e.z).is_some_and(|d| d.dir != dir) {
                    return Err(Error::new(
                        "a wall already crosses that tile the other way — remove it first",
                    ));
                }
                let someone = self
                    .sims
                    .iter()
                    .any(|s| s.away_until.is_none() && s.tile() == (e.x, e.z));
                if target.blocks() && someone {
                    return Err(Error::new("someone is standing there"));
                }
            }
            let current = edge_at(&self.lot, e.axis, e.x, e.z);
            if target.is_fence() && current.is_wall() {
                return Err(Error::new(
                    "a wall stands there — take it down before putting up a fence",
                ));
            }
            if matches!(target, Edge::Door | Edge::Window)
                && !current.is_wall()
                && !walled_beside(e)
            {
                let what = if target == Edge::Door {
                    "door"
                } else {
                    "window"
                };
                return Err(Error::new(format!(
                    "a {what} has to go into a wall — build one there first"
                )));
            }
            // The wall's look now (a diagonal running the other way isn't this wall).
            let at = edge_ref(e.axis, e.x, e.z);
            let old = if current == Edge::Open {
                EdgeLook::default()
            } else {
                self.lot.look(at)
            };
            let look = next_look(current, target, old, e);
            if matches!(target, Edge::Door | Edge::Window)
                && look.form == 0
                && old.form == FORM_HALF
                && current == Edge::Wall
            {
                return Err(Error::new("doors and windows need a full-height wall"));
            }
            let wall_price = if diagonal.is_some() {
                rules.diagonal_wall()
            } else {
                rules.wall
            };
            if current != target {
                cost += match target {
                    Edge::Wall => {
                        wall_price
                            + 2 * rules.covering_price(look.sides[0])
                                * i64::from(current == Edge::Open)
                    }
                    Edge::Door => rules.door_price(look.style),
                    Edge::Window => rules.window_price(look.style),
                    Edge::Fence => rules.fence_price(look.style),
                    Edge::Gate => rules.gate_price(look.style),
                    Edge::Open => rules.remove,
                };
            } else if target != Edge::Open && look != old && (look.style, look.form, look.sides) != (old.style, old.form, old.sides) {
                // Rebuilt in another form, or replaced by another style (resizing a door or window is free).
                cost += match target {
                    Edge::Door => rules.door_price(look.style),
                    Edge::Window => rules.window_price(look.style),
                    Edge::Fence => rules.fence_price(look.style),
                    Edge::Gate => rules.gate_price(look.style),
                    _ => wall_price,
                };
            }
            if current != target || look != old {
                looks.push((at, look));
            }
        }
        self.can_pay(h, cost, || "not enough money to build that".into())?;
        let before = self.reach_signature(plot);
        let saved = self.lot.clone();
        for e in edits {
            let target = edge_kind(e.kind);
            match e.axis {
                EdgeAxis::H => self.lot.set_h(e.x as usize, e.z as usize, target),
                EdgeAxis::V => self.lot.set_v(e.x as usize, e.z as usize, target),
                EdgeAxis::Dp | EdgeAxis::Dn => {
                    let dir = e.axis.diagonal().expect("diagonal axis");
                    let current = self.lot.diag(e.x, e.z);
                    // Removing only clears a diagonal running the same way.
                    if target != Edge::Open || current.is_some_and(|d| d.dir == dir) {
                        let d = (target != Edge::Open).then_some(Diagonal { dir, edge: target });
                        self.lot.set_diag(e.x as usize, e.z as usize, d);
                    }
                }
            }
        }
        for (at, look) in looks {
            self.lot.set_look(at, look);
        }
        // Rooms first: upstairs, where there's floor depends on them.
        self.lot.compute_rooms();
        if !self.keeps_reach(plot, &before, None) {
            self.lot = saved;
            return Err(Error::new(
                "that would shut a resident or an object in — add a door or a gate",
            ));
        }
        if let Err(e) = self.storeys_stand(plot) {
            self.lot = saved;
            return Err(e);
        }
        self.pay(h, cost);
        self.nav_changed();
        self.rooms_layout_changed();
        Ok(())
    }

    /// Covers wall faces on the home plot (paint, wallpaper, brick...), paying each covering's
    /// price per face that changes. Every face must belong to a wall (with or without a door or
    /// window); a face listed twice counts once.
    pub fn paint(&mut self, household: u32, faces: &[FacePaint]) -> Result<(), Error> {
        let (h, plot) = self.home_of(household)?;
        let rules = &self.content.build;
        let mut changes: Vec<(EdgeRef, usize, u8)> = Vec::new();
        let mut cost = 0;
        for f in faces {
            if !self.edge_on_plot(plot, f.axis, f.x, f.z) {
                return Err(Error::new("you can only paint walls on your own lot"));
            }
            if f.side > 1 {
                return Err(Error::new("a wall has two faces (0 and 1)"));
            }
            if f.covering as usize > rules.coverings.len() {
                return Err(Error::new(format!("unknown wall covering {}", f.covering)));
            }
            let standing = match f.axis {
                EdgeAxis::H => self.lot.h_edge(f.x as usize, f.z as usize).is_wall(),
                EdgeAxis::V => self.lot.v_edge(f.x as usize, f.z as usize).is_wall(),
                EdgeAxis::Dp | EdgeAxis::Dn => self
                    .lot
                    .diag(f.x, f.z)
                    .is_some_and(|d| Some(d.dir) == f.axis.diagonal()),
            };
            if !standing {
                return Err(Error::new("there's no wall there to paint"));
            }
            let at = edge_ref(f.axis, f.x, f.z);
            let side = f.side as usize;
            if changes.iter().any(|&(e, s, _)| e == at && s == side) {
                continue;
            }
            if self.lot.look(at).sides[side] != f.covering {
                cost += rules.covering_price(f.covering);
                changes.push((at, side, f.covering));
            }
        }
        self.can_pay(h, cost, || "not enough money to paint that".into())?;
        if changes.is_empty() {
            return Ok(());
        }
        for (at, side, covering) in changes {
            let mut look = self.lot.look(at);
            look.sides[side] = covering;
            self.lot.set_look(at, look);
        }
        self.pay(h, cost);
        self.structure_version = self.structure_version.wrapping_add(1);
        self.rooms_need_rescore();
        Ok(())
    }

    /// Covers floor tiles on the home plot (wood, tile, carpet...), paying each covering's price
    /// per tile that changes. Every tile must be indoors (a tile split by a diagonal wall counts
    /// when either half is); a tile listed twice counts once.
    pub fn paint_floor(&mut self, household: u32, tiles: &[FloorPaint]) -> Result<(), Error> {
        let (h, plot) = self.home_of(household)?;
        let rules = &self.content.build;
        let p = &self.plots[plot as usize];
        let mut changes: Vec<(u16, u16, u8)> = Vec::new();
        let mut cost = 0;
        for t in tiles {
            if !p.contains(t.x, self.ground_z(t.z)) || !self.lot.in_bounds(t.x, t.z) {
                return Err(Error::new("you can only lay floors on your own lot"));
            }
            if t.covering as usize > rules.floors.len() {
                return Err(Error::new(format!("unknown floor covering {}", t.covering)));
            }
            if self.lot.room_at(t.x, t.z) == OUTDOORS {
                return Err(Error::new("floors go inside rooms"));
            }
            let (x, z) = (t.x as u16, t.z as u16);
            if changes.iter().any(|&(cx, cz, _)| (cx, cz) == (x, z)) {
                continue;
            }
            if self.lot.floor(x, z) != t.covering {
                cost += rules.floor_price(t.covering);
                changes.push((x, z, t.covering));
            }
        }
        self.can_pay(h, cost, || "not enough money for that floor".into())?;
        if changes.is_empty() {
            return Ok(());
        }
        for (x, z, covering) in changes {
            self.lot.set_floor(x, z, covering);
        }
        self.pay(h, cost);
        self.structure_version = self.structure_version.wrapping_add(1);
        self.rooms_need_rescore();
        Ok(())
    }

    /// Whether a wall edge (or the diagonal across a tile) lies on plot `plot` and on the lot.
    fn edge_on_plot(&self, plot: u32, axis: EdgeAxis, x: i32, z: i32) -> bool {
        let p = &self.plots[plot as usize];
        let (lw, ld) = (self.lot.width as i32, self.lot.depth as i32);
        if x < 0 || z < 0 || z > ld {
            return false;
        }
        // On any storey: compare with the plot on the ground rows.
        let (z, ld) = (self.ground_z(z), ld.min(self.storey_depth()));
        match axis {
            EdgeAxis::H => {
                x >= p.x && x < p.x + p.w && z >= p.z && z <= p.z + p.d && x < lw && z <= ld
            }
            EdgeAxis::V => {
                x >= p.x && x <= p.x + p.w && z >= p.z && z < p.z + p.d && x <= lw && z < ld
            }
            EdgeAxis::Dp | EdgeAxis::Dn => p.contains(x, z) && self.lot.in_bounds(x, z),
        }
    }

    /// Upstairs stands on downstairs: every room above the ground has a room under each of its
    /// tiles, and every wall above the ground stands on a room's ceiling (a room on one side of
    /// it below). Nothing stands on the stairwell either. Stairs up from the basement keep
    /// coming up inside a room.
    pub(crate) fn storeys_stand(&self, plot: u32) -> Result<(), Error> {
        let indoor = |x: i32, z: i32| self.lot.in_bounds(x, z) && self.lot.room_at(x, z) != OUTDOORS;
        for (o, _) in self.objects.iter().zip(&self.object_plot).filter(|(_, on)| **on == Some(plot)) {
            if self.content.objects[o.def].stairs && self.storey_of(o.z) < 0 {
                let rise = self.stairs.rise(-1);
                if o.tiles(&self.content).any(|(x, z)| !indoor(x, z + rise)) {
                    return Err(Error::new("the stairs down to the basement need a room over them"));
                }
            }
        }
        if self.storeys < 2 {
            return Ok(());
        }
        let p = &self.plots[plot as usize];
        for k in 1..self.storeys as i32 {
            // This storey's first row, and the rows down to the storey below.
            let (up, d) = (self.storey_row(k), self.stairs.rise(k - 1));
            for z in p.z + up..p.z + p.d + up {
                for x in p.x..p.x + p.w {
                    if indoor(x, z) && !indoor(x, z - d) {
                        return Err(Error::new("a room upstairs needs a room under it"));
                    }
                }
            }
            for z in p.z + up..=p.z + p.d + up {
                for x in p.x..=p.x + p.w {
                    let h = x < p.x + p.w && self.lot.h_edge(x as usize, z as usize) != Edge::Open;
                    let v = z < p.z + p.d + up && self.lot.v_edge(x as usize, z as usize) != Edge::Open;
                    let diag = x < p.x + p.w && z < p.z + p.d + up && self.lot.diag(x, z).is_some();
                    if h && !(indoor(x, z - 1 - d) || indoor(x, z - d)) || v && !(indoor(x - 1, z - d) || indoor(x, z - d)) || diag && !indoor(x, z - d) {
                        return Err(Error::new("walls upstairs have to stand on a room below"));
                    }
                }
            }
        }
        Ok(())
    }

    /// Removes an object. Sims using it stop, and later object ids shift down by one (in what
    /// they do, have queued, do next out of habit, and couldn't get to).
    pub(crate) fn remove_object(&mut self, id: u32) -> ObjectInstance {
        let World {
            sims,
            objects,
            content,
            ..
        } = self;
        for sim in sims.iter_mut() {
            if sim.current.as_ref().is_some_and(|a| uses_object(&a.task.kind, id)) {
                end_activity(sim, content, objects);
            }
            sim.queue.retain(|t| !uses_object(&t.kind, id));
            // (A routine's next step, washing hands at this sink, goes with it.)
            sim.follow_up = sim.follow_up.take().filter(|t| !uses_object(&t.kind, id));
            let current = sim.current.as_mut().map(|a| &mut a.task);
            for t in current.into_iter().chain(sim.queue.iter_mut()).chain(sim.follow_up.iter_mut()) {
                shift_objects(&mut t.kind, id);
            }
            sim.remap_unreachable(|o| if o == id { None } else { Some(if o > id { o - 1 } else { o }) });
            // A seat taken to use something else (content `seatNear`): ids shift down too, and
            // if the seat itself goes they stand up but may carry on with what they were using.
            match sim.use_seat {
                Some(seat) if seat == id => {
                    sim.use_seat = None;
                    if sim.pose == crate::content::Pose::Sit {
                        sim.seated_from = None;
                        sim.pose = crate::content::Pose::Stand;
                    }
                }
                Some(seat) if seat > id => sim.use_seat = Some(seat - 1),
                _ => {}
            }
        }
        let obj = self.objects.remove(id as usize);
        self.object_plot.remove(id as usize);
        for (i, o) in self.objects.iter_mut().enumerate() {
            o.id = i as u32;
        }
        for (x, z) in self.blocking_tiles(&obj) {
            let i = self.lot.tile_index(x, z);
            self.blocked[i] = false;
        }
        if self.content.objects[obj.def].stairs {
            self.refresh_stairs();
        }
        self.nav_changed();
        self.rooms_layout_changed();
        obj
    }
}

/// Whether an edit's look exists in the content (form, covering, door or window style).
fn check_look(rules: &crate::content::BuildRules, e: &EdgeEdit) -> Result<(), Error> {
    if e.form.is_some_and(|f| f > FORM_HALF) {
        return Err(Error::new("a wall is full height (0) or a half wall (1)"));
    }
    if e.cover.is_some_and(|c| c as usize > rules.coverings.len()) {
        return Err(Error::new("unknown wall covering"));
    }
    let styles = match e.kind {
        EdgeKind::Door => rules.doors.len(),
        EdgeKind::Window => rules.windows.len(),
        EdgeKind::Fence | EdgeKind::Gate => rules.fences.len(),
        _ => return Ok(()),
    };
    if e.style.is_some_and(|s| s as usize >= styles.max(1)) {
        return Err(Error::new("unknown style for that"));
    }
    if matches!(e.kind, EdgeKind::Door | EdgeKind::Window) && e.form == Some(FORM_HALF) {
        return Err(Error::new("doors and windows need a full-height wall"));
    }
    use crate::command::{MAX_HEAD, MIN_DOOR_HEAD, MIN_WINDOW};
    match e.kind {
        _ if e.sill.is_none() && e.head.is_none() => {}
        EdgeKind::Door if e.sill.is_some() => return Err(Error::new("a door starts at the floor")),
        EdgeKind::Door if e.head.is_some_and(|h| !(MIN_DOOR_HEAD..=MAX_HEAD).contains(&h)) => {
            return Err(Error::new(format!("a door is {} to {} m tall", MIN_DOOR_HEAD as f32 / 100.0, MAX_HEAD as f32 / 100.0)));
        }
        EdgeKind::Window if e.head.is_some_and(|h| h > MAX_HEAD) || e.sill.is_some_and(|s| s + MIN_WINDOW > MAX_HEAD) => {
            return Err(Error::new(format!("a window reaches {} m at most", MAX_HEAD as f32 / 100.0)));
        }
        EdgeKind::Window if matches!((e.sill, e.head), (Some(s), Some(h)) if s + MIN_WINDOW > h) => {
            return Err(Error::new(format!("a window is at least {MIN_WINDOW} cm tall")));
        }
        EdgeKind::Door | EdgeKind::Window => {}
        _ => return Err(Error::new("only doors and windows have a sill or a top")),
    }
    Ok(())
}

/// The look a wall has after an edit: new walls take the edit's form and covering, new doors
/// and windows its style; standing ones keep their faces and change only what the edit names.
fn next_look(current: Edge, target: Edge, old: EdgeLook, e: &EdgeEdit) -> EdgeLook {
    match target {
        Edge::Open => EdgeLook::default(),
        // Fences and gates have only a style; a gate put into a fence keeps the fence's.
        Edge::Fence | Edge::Gate => EdgeLook {
            style: e
                .style
                .unwrap_or(if current.is_fence() { old.style } else { 0 }),
            ..EdgeLook::default()
        },
        _ if current == Edge::Open => {
            let cover = e.cover.unwrap_or(0);
            EdgeLook {
                sides: [cover, cover],
                form: if target == Edge::Wall {
                    e.form.unwrap_or(0)
                } else {
                    0
                },
                style: if target == Edge::Wall {
                    0
                } else {
                    e.style.unwrap_or(0)
                },
                sill: if target == Edge::Window { e.sill } else { None },
                head: if target == Edge::Wall { None } else { e.head },
            }
        }
        Edge::Wall => EdgeLook {
            form: e
                .form
                .unwrap_or(if current == Edge::Wall { old.form } else { 0 }),
            style: 0,
            sill: None,
            head: None,
            ..old
        },
        // A door or window keeps its size when restyled; one put in for another kind starts as its style is.
        _ => EdgeLook {
            form: 0,
            style: e
                .style
                .unwrap_or(if current == target { old.style } else { 0 }),
            sill: if target == Edge::Window { e.sill.or(if current == target { old.sill } else { None }) } else { None },
            head: e.head.or(if current == target { old.head } else { None }),
            ..old
        },
    }
}

/// What the player's quick fix costs (Buy mode): a share of the price, by wear.
pub(crate) fn repair_cost(content: &crate::content::Content, obj: &crate::world::ObjectInstance) -> i64 {
    let price = content.objects[obj.def].price.unwrap_or(0) as f32;
    (price * content.object_rules.repair.cost * obj.wear).round().max(1.0) as i64
}

#[cfg(test)]
mod tests {
    use crate::World;
    use crate::world::{Task, TaskKind};

    const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1}],
        "objects":[{"id":"fridge","name":"Fridge","interactions":[
            {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.4}}]}]}"#;

    const LOT: &str = r#"{"width":12,"depth":12,
        "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"fridge","x":4,"z":1,"rot":0},{"def":"fridge","x":7,"z":1,"rot":0}],
        "sims":[{"name":"Ada","x":3.5,"z":5.5}]}"#;

    fn snack(object: u32) -> TaskKind {
        TaskKind::Use { object, interaction: 0 }
    }

    fn object(t: Option<TaskKind>) -> Option<u32> {
        match t {
            Some(TaskKind::Use { object, .. }) => Some(object),
            _ => None,
        }
    }

    /// What a resident does next out of habit, and what they couldn't get to, follow the
    /// object they name when ids shift, and go with it.
    #[test]
    fn follow_ups_and_what_they_couldnt_reach_follow_the_object() {
        let mut w = World::from_json(CONTENT, LOT, 7).unwrap();
        w.sims[0].follow_up = Some(Task { kind: snack(2), directed: false, routine: true });
        w.sims[0].remember_unreachable(snack(1), 1000);
        w.remove_object(0);
        assert_eq!(object(w.sims[0].follow_up.map(|t| t.kind)), Some(1));
        assert_eq!(object(w.sims[0].unreachable[0].map(|s| s.0)), Some(0));
        w.remove_object(0);
        assert_eq!(object(w.sims[0].follow_up.map(|t| t.kind)), Some(0));
        assert!(w.sims[0].unreachable[0].is_none(), "the one they couldn't reach is gone");
        w.remove_object(0);
        assert!(w.sims[0].follow_up.is_none(), "the one they'd use next is gone");
    }
}
