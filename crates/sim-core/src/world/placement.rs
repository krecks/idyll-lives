//! Where objects stand and how they fit.

use super::*;

impl World {
    /// Notes the plot of every ground tile (plots never move; upper storeys are the same
    /// ground), which `plot_at` and the walks (`path::Grounds`) look up, and the flats of each
    /// building with flats by storey (`flat_tiles`). Flats leave the ground tiles to their
    /// building. Made as the plots are loaded, before anything is placed on them.
    pub(crate) fn map_plots(&mut self) {
        let (width, depth) = (self.lot.width as i32, self.storey_depth());
        if self.tile_plot.len() == (width * depth) as usize {
            return;
        }
        self.tile_plot = vec![crate::path::NO_PLOT; (width * depth) as usize];
        for p in lots_of(&self.plots).iter().rev() {
            for z in p.z.max(0)..(p.z + p.d).min(depth) {
                for x in p.x.max(0)..(p.x + p.w).min(width) {
                    self.tile_plot[(z * width + x) as usize] = p.id as u16;
                }
            }
        }
        self.flat_tiles.clear();
        self.flats_in = vec![0; self.plots.len()];
        for f in flats_of(&self.plots) {
            let Some(lot) = f.of.and_then(|of| self.plots.get(of as usize)) else {
                continue;
            };
            let k = match self.flats_in[lot.id as usize] {
                0 => {
                    self.flat_tiles.push(FlatTiles {
                        x: lot.x,
                        z: lot.z,
                        w: lot.w,
                        d: lot.d,
                        storeys: Vec::new(),
                    });
                    self.flats_in[lot.id as usize] = self.flat_tiles.len() as u16;
                    self.flat_tiles.len() - 1
                }
                k => usize::from(k) - 1,
            };
            let t = &mut self.flat_tiles[k];
            let storey = i32::from(f.storey);
            let level = match t.storeys.iter().position(|(s, _)| *s == storey) {
                Some(i) => i,
                None => {
                    let tiles = vec![crate::path::NO_PLOT; (t.w * t.d).max(0) as usize];
                    t.storeys.push((storey, tiles));
                    t.storeys.len() - 1
                }
            };
            let (x0, z0, w) = (t.x, t.z, t.w);
            let tiles = &mut t.storeys[level].1;
            for z in f.z.max(z0)..(f.z + f.d).min(z0 + t.d) {
                for x in f.x.max(x0)..(f.x + f.w).min(x0 + w) {
                    tiles[((z - z0) * w + (x - x0)) as usize] = f.id as u16;
                }
            }
        }
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
    /// The `ObjectInstance` `check_fit_among` would place, as it would be there: its footprint
    /// and extent from `layout`, where it gives them.
    fn candidate(&self, def: usize, x: i32, z: i32, rot: u8, layout: Layout) -> ObjectInstance {
        ObjectInstance {
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
            shut: false,
            run: None,
            rack: 0,
            keeper: None,
        }
    }

    pub(crate) fn check_fit(
        &self,
        def: usize,
        x: i32,
        z: i32,
        rot: u8,
        layout: Layout,
    ) -> Result<ObjectInstance, Error> {
        self.check_fit_among(def, x, z, rot, layout, None)
    }

    /// `check_fit`, looking for what's in the way only among `near` (in id order) when given:
    /// every object whose tiles or overhang reach where it would go (`ObjectInstance::reach`),
    /// found once for many tries over one plot (`find_spot`). Without it, those are found among
    /// the whole town first: a check of each tile against every object in a full town, for every
    /// spot a delivered bed was tried at, made the noon a newcomer or an adoption came half a
    /// second long.
    pub(crate) fn check_fit_among(
        &self,
        def: usize,
        x: i32,
        z: i32,
        rot: u8,
        layout: Layout,
        near: Option<&[&ObjectInstance]>,
    ) -> Result<ObjectInstance, Error> {
        let obj = self.candidate(def, x, z, rot, layout);
        let d = &self.content.objects[def];
        let name = &d.id;
        let layer = d.layer;
        if layer == Layer::Surface {
            return self.check_surface_fit(obj);
        }
        let found;
        let near = match near {
            Some(near) => near,
            None => {
                found = self.objects_reaching(obj.reach(&self.content));
                &found[..]
            }
        };
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
                near.iter()
                    .copied()
                    .any(|o| !self.content.objects[o.def].layer.mounted() && on(o, (tx, tz)))
            };
            let taken = if layer.mounted() {
                near.iter()
                    .copied()
                    .any(|o| self.content.objects[o.def].layer == layer && on(o, (tx, tz)))
                    || (d.hangs_low && floor && !furniture())
            } else {
                let hung = || {
                    near.iter().copied().any(|o| {
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
            let clash = near.iter().copied().find(|o| {
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
                near.iter()
                    .copied()
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
            if let Some(o) = near.iter().copied().find(|o| other(o)) {
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
            // (Each chunk's steps only while they're as the lot is: `Reach::steps`.)
            steps: Some(&self.reach),
        }
    }

    /// The objects whose tiles or overhang (`ObjectInstance::reach`) touch the tile rectangle
    /// `[x0, z0, x1, z1)`, in id order: all a fit there needs to look at (`check_fit_among`).
    pub(crate) fn objects_reaching(&self, rect: [i32; 4]) -> Vec<&ObjectInstance> {
        let m = self.content.overhang_reach();
        let [x0, z0, x1, z1] = rect;
        self.objects
            .iter()
            .filter(|o| {
                o.may_reach(rect, m) && {
                    let [a0, b0, a1, b1] = o.reach(&self.content);
                    a0 < x1 && x0 < a1 && b0 < z1 && z0 < b1
                }
            })
            .collect()
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
}
