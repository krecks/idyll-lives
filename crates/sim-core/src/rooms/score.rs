use super::*;

/// What the things standing in a room (or a garden) give it.
#[derive(Default, Clone, Copy)]
pub(super) struct Things {
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
pub(super) fn score_dirt(room: &mut RoomInfo, dirt: f32, rules: &crate::content::RoomRules) {
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
    // Gardens: each plot's outdoor tiles (a flat has none: its building's yard is the building's).
    for (pi, p) in w.lots().iter().enumerate() {
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

/// Rooms `ids` (lot ids, each once) and the gardens of `plots` (lots on the ground, each once), exactly as `compute`
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
    // Gardens: each lot's outdoor tiles (as `compute`).
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
