//! The navigation grid: rebuilding it when the world changes.

use super::*;

impl World {
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
                steps: None,
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
    pub(crate) fn refresh_reach(&mut self) {
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
            steps: None,
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
}
