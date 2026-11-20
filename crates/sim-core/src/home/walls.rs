//! Walls, floors and paint: building on the home plot and checking it holds.
//!
//! Split from `home.rs` (see `mod.rs`).

use super::*;

impl World {
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
            if matches!(target, Edge::Window | Edge::Fence | Edge::Gate)
                && (self.storey_of(a.1) < 0 || self.storey_of(b.1) < 0)
            {
                return Err(Error::new(
                    "a basement is underground: no windows, fences or gates down there",
                ));
            }
            if e.kind != EdgeKind::Open
                && let Some(dir) = diagonal
                && (object_at(a.0, a.1).is_some()
                    || self
                        .objects
                        .iter()
                        .any(|o| o.body_crosses(&self.content, e.x, e.z, dir)))
            {
                return Err(Error::new("a wall can't go through furniture"));
            }
            if e.kind != EdgeKind::Open
                && let Some(what) = self.porch_across(e.axis, e.x, e.z)
            {
                return Err(Error::new(if target.is_fence() {
                    format!("a fence can't go through the {what}")
                } else {
                    format!("a wall can't go through the {what}")
                }));
            }
            if e.kind != EdgeKind::Open && self.garage_across(e.axis, e.x, e.z) {
                return Err(Error::new(if target.is_fence() {
                    "a fence can't go through the garage"
                } else {
                    "a wall can't go through the garage"
                }));
            }
            if e.kind != EdgeKind::Open
                && diagonal.is_none()
                && self.objects.iter().any(|o| {
                    o.wall_edges(&self.content)
                        .chain(o.body_edges(&self.content))
                        .chain(o.overhang_edges(&self.content))
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
            // Nor does the wall come down (or become a door or a window) behind what stands against it.
            if target != Edge::Wall
                && diagonal.is_none()
                && let Some(o) = self.objects.iter().find(|o| {
                    self.content.objects[o.def].against_wall
                        && o.back_edges(&self.content)
                            .contains(&(e.axis == EdgeAxis::H, e.x, e.z))
                })
            {
                let name = self.content.objects[o.def].name.to_lowercase();
                return Err(Error::new(format!(
                    "the {name} stands against that wall: move it first"
                )));
            }
            // Only curtains and the like hang over a window.
            if target == Edge::Window
                && diagonal.is_none()
                && let Some(o) = self
                    .hung_on(e.axis == EdgeAxis::H, e.x, e.z)
                    .find(|o| !self.content.objects[o.def].over_window)
            {
                let name = self.content.objects[o.def].name.to_lowercase();
                return Err(Error::new(format!(
                    "the {name} hangs there, and it can't hang over a window: move it first"
                )));
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
            } else if target != Edge::Open
                && look != old
                && (look.style, look.form, look.sides) != (old.style, old.form, old.sides)
            {
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
        // (What it changes is put back if it turns out it can't stand.)
        let mark = self.lot.log_begin();
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
        self.sync_tile_arrays();
        if !self.keeps_reach(plot, &before, None) {
            self.lot.log_rollback(mark);
            self.lot.compute_rooms();
            self.sync_tile_arrays();
            return Err(Error::new(
                "that would shut a resident or an object in — add a door or a gate",
            ));
        }
        if let Err(e) = self
            .storeys_stand(plot)
            .and_then(|()| self.walls_still_hold(plot))
        {
            self.lot.log_rollback(mark);
            self.lot.compute_rooms();
            self.sync_tile_arrays();
            return Err(e);
        }
        self.lot.log_forget(mark);
        self.pay(h, cost);
        // A porch whose door went goes with it.
        self.porches_follow(plot);
        // The tiles either side of every edge built on (a diagonal's own tile), and upstairs the
        // plot (its rooms say where there's floor).
        let rect = edits
            .iter()
            .fold([i32::MAX, i32::MAX, i32::MIN, i32::MIN], |r, e| {
                [
                    r[0].min(e.x - 1),
                    r[1].min(e.z - 1),
                    r[2].max(e.x + 2),
                    r[3].max(e.z + 2),
                ]
            });
        if edits.is_empty() {
            self.nav_changed();
        } else {
            self.walls_changed_on(plot, rect);
        }
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
    pub(crate) fn edge_on_plot(&self, plot: u32, axis: EdgeAxis, x: i32, z: i32) -> bool {
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
        let indoor =
            |x: i32, z: i32| self.lot.in_bounds(x, z) && self.lot.room_at(x, z) != OUTDOORS;
        for (o, _) in self
            .objects
            .iter()
            .zip(&self.object_plot)
            .filter(|(_, on)| **on == Some(plot))
        {
            if self.content.objects[o.def].stairs && self.storey_of(o.z) < 0 {
                let rise = self.stairs.rise(-1);
                if o.tiles(&self.content).any(|(x, z)| !indoor(x, z + rise)) {
                    return Err(Error::new(
                        "the stairs down to the basement need a room over them",
                    ));
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
                    let v =
                        z < p.z + p.d + up && self.lot.v_edge(x as usize, z as usize) != Edge::Open;
                    let diag = x < p.x + p.w && z < p.z + p.d + up && self.lot.diag(x, z).is_some();
                    if h && !(indoor(x, z - 1 - d) || indoor(x, z - d))
                        || v && !(indoor(x - 1, z - d) || indoor(x, z - d))
                        || diag && !indoor(x, z - d)
                    {
                        return Err(Error::new("walls upstairs have to stand on a room below"));
                    }
                }
            }
        }
        Ok(())
    }

    /// After walls change on the plot, what they held still stands as `check_fit` and
    /// `check_storey_fit` have it: what hangs from a ceiling, and what stands upstairs or in the
    /// basement, inside a room (a wall taken down doesn't leave a ceiling lamp hanging from
    /// nothing outdoors), and what stands against a wall with plain wall behind it.
    pub(crate) fn walls_still_hold(&self, plot: u32) -> Result<(), Error> {
        for (o, _) in self
            .objects
            .iter()
            .zip(&self.object_plot)
            .filter(|(_, on)| **on == Some(plot))
        {
            let def = &self.content.objects[o.def];
            let name = || def.name.to_lowercase();
            let needs_room = def.layer == Layer::Ceiling || self.storey_of(o.z) != 0;
            if needs_room
                && o.tiles(&self.content)
                    .any(|(x, z)| self.lot.room_at(x, z) == OUTDOORS)
            {
                return Err(Error::new(format!(
                    "the {} would be left outdoors: move it first",
                    name()
                )));
            }
            if def.against_wall && self.wall_behind(o) != Some(false) {
                return Err(Error::new(format!(
                    "the {} stands against that wall: move it first",
                    name()
                )));
            }
        }
        Ok(())
    }
}
