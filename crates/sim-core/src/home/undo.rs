//! Undo and redo: snapshots of a household's home, and restoring them.

use super::*;

impl World {
    /// The state a build or buy edit by `household` may change (None if it has no home).
    pub(crate) fn home_snapshot(&mut self, household: u32) -> Option<HomeSnapshot> {
        let (household, _) = self.home_of(household).ok()?;
        Some(HomeSnapshot {
            household,
            lot_mark: self.lot.log_begin(),
            lot: None,
            objects: self.objects.clone(),
            object_plot: self.object_plot.clone(),
            blocked: self.blocked.clone(),
            sims: self.sims.clone(),
            households: self.households.clone(),
            plots: self.plots.clone(),
            items: self.items.clone(),
            garages: self.garages.clone(),
            porches: self.porches.clone(),
            porch_pieces: self.porch_pieces.clone(),
            cars: self.cars.clone(),
            car_tiles: self.car_tiles.clone(),
            cars_parked: self.cars_parked.clone(),
            events_before: self.events.last_id(),
            story: Vec::new(),
        })
    }

    /// Keeps `before` for undo if the edit changed anything (an edit that changed nothing, like
    /// drawing a wall where one stands, leaves no step).
    pub(crate) fn remember(&mut self, mut before: HomeSnapshot) {
        let h = before.household;
        let changes = self.lot.log_end(before.lot_mark);
        let lot_changed = !changes.is_empty();
        before.lot = Some(changes);
        if lot_changed {
            self.lot_version = self.lot_version.wrapping_add(1);
        }
        let changed = before.households[h].funds != self.households[h].funds
            || before.objects != self.objects
            || lot_changed
            || before.plots != self.plots
            || before.garages != self.garages
            || before.porches != self.porches
            || before.porch_pieces != self.porch_pieces;
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

    /// The land was shaped (`World::shape_land`) while home edits are kept for undo and redo: they
    /// put back the money and who may walk where as they were, so what the land took stays taken
    /// through them. Household `h` paid `paid` (negative: was refunded), the tiles `wet` are under
    /// water now and `dried` aren't (`Lot::tile_index`), walked on or not as they are now.
    pub(crate) fn keep_land_in_edits(
        &mut self,
        h: usize,
        paid: i64,
        wet: &[usize],
        dried: &[usize],
    ) {
        let now: Vec<bool> = dried
            .iter()
            .map(|&i| self.blocked.get(i).copied().unwrap_or(false))
            .collect();
        for s in self.undo.iter_mut().chain(self.redo.iter_mut()) {
            if let Some(hh) = s.households.get_mut(h) {
                hh.funds -= paid;
            }
            for &i in wet {
                if let Some(b) = s.blocked.get_mut(i) {
                    *b = true;
                }
            }
            for (&i, &b) in dried.iter().zip(&now) {
                if let Some(was) = s.blocked.get_mut(i) {
                    *was = b;
                }
            }
        }
    }

    /// Build and buy edits the household can take back.
    pub fn undo_steps(&self, household: usize) -> usize {
        self.undo
            .iter()
            .filter(|s| s.household == household)
            .count()
    }

    /// Edits the household took back and can make again.
    pub fn redo_steps(&self, household: usize) -> usize {
        self.redo
            .iter()
            .filter(|s| s.household == household)
            .count()
    }

    /// Takes back the household's last build or buy edit: walls, furniture, money and residents
    /// are as they were before it (a sold object comes back where it stood, at its old price).
    /// It can be made again with `redo` until the next edit or until time moves on.
    pub fn undo(&mut self, household: u32) -> Result<(), Error> {
        let (h, _) = self.home_of(household)?;
        if self.undo.last().is_none_or(|s| s.household != h) {
            return Err(Error::new("nothing to undo"));
        }
        let mut now = self
            .home_snapshot(household)
            .expect("the household has a home");
        let mut s = self.undo.pop().expect("checked above");
        // What the edit told goes from the story (and comes back with a redo).
        now.story = self.events.take(&s.story);
        self.keep_orders(&mut s);
        self.restore(s);
        now.lot = Some(self.lot.log_end(now.lot_mark));
        self.redo.push(now);
        Ok(())
    }

    /// Makes the household's last undone edit again.
    pub fn redo(&mut self, household: u32) -> Result<(), Error> {
        let (h, _) = self.home_of(household)?;
        if self.redo.last().is_none_or(|s| s.household != h) {
            return Err(Error::new("nothing to redo"));
        }
        let mut now = self
            .home_snapshot(household)
            .expect("the household has a home");
        let mut s = self.redo.pop().expect("checked above");
        now.story = self.events.put_back(std::mem::take(&mut s.story));
        self.keep_orders(&mut s);
        self.restore(s);
        now.lot = Some(self.lot.log_end(now.lot_mark));
        self.undo.push(now);
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
        let changed = match &s.lot {
            // A finished edit taken back (undo, redo): what it changed goes back.
            Some(changes) => {
                self.lot.put_back(changes);
                !changes.is_empty()
            }
            // An edit under way given up: what it changed since it began goes back.
            None => self.lot.log_rollback(s.lot_mark),
        };
        if changed {
            self.lot_version = self.lot_version.wrapping_add(1);
        }
        self.lot.compute_rooms();
        self.objects = s.objects;
        self.object_plot = s.object_plot;
        self.blocked = s.blocked;
        self.garages = s.garages;
        if (&self.porches, &self.porch_pieces) != (&s.porches, &s.porch_pieces) {
            self.structure_version = self.structure_version.wrapping_add(1);
        }
        self.porches = s.porches;
        self.porch_pieces = s.porch_pieces;
        // (The work taxis as they are now: the runs drive them, `taxi/`.)
        let taxis: Vec<crate::cars::Car> = self.cars.iter().filter(|c| c.taxi).cloned().collect();
        self.cars = s
            .cars
            .into_iter()
            .filter(|c| !c.taxi)
            .chain(taxis)
            .collect();
        self.car_tiles = s.car_tiles;
        self.cars_parked = s.cars_parked;
        self.sync_tile_arrays();
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
        self.rooms_things_changed_everywhere();
    }
}
