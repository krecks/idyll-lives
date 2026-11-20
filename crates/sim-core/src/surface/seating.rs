//! Standing spots, seating and dining seats.
//!
//! Split from `surface.rs` (see `mod.rs`).

use super::*;

/// The tile `p` is on.
pub(crate) fn tile_of(p: [f32; 2]) -> (i32, i32) {
    (p[0].floor() as i32, p[1].floor() as i32)
}

/// How far (m) towards its front of its slot's middle someone sits on a bed's or a seat's side
/// to get on (and so which row of a bed two tiles long they come to it from).
const SIDE_ROW: f32 = 0.15;

/// The tile someone sitting or lying in `slot` of `obj` comes to it from (and gets up to), when
/// it isn't the object's own (`approach_tiles`): round to its place (`slot_approach`); beside
/// the slot for a thing got onto from the side (`Approach::Side`: a bed beside its sleeper's
/// middle, the slot's own side first, a bunk's lower sleeper from its ladder side first and the
/// upper one from the front, up the ladder; a bike, a chaise), behind it for
/// a seat behind the work (`Approach::Behind`: a desk's chair, the drum kit's throne), else
/// beside it; in front of its own slot on a seat for several (the end of a sofa); none for a
/// cabin got into through its door (`Approach::Door`: from its front). Each side
/// nearer `from` (where they are now) first where the slot is in the middle. None when none of
/// those can be stood on.
pub fn seat_approach(
    content: &Content,
    nav: &NavGrid,
    obj: &ObjectInstance,
    slot: usize,
    from: [f32; 2],
) -> Option<(i32, i32)> {
    let def = &content.objects[obj.def];
    // (Into a cabin through its door: from the front, whichever side the place is on.)
    if def.approach == Approach::Door {
        return None;
    }
    if !def.places.is_empty() {
        return slot_approach(content, nav, obj, slot);
    }
    let [r, f] = obj.slot_local(def, slot);
    let [hw, hd] = obj.footprint.map(|v| v as f32 / 2.0);
    let storey = nav.stairs.storey(obj.z);
    // (A hair off a tile line: the middle of a thing two tiles wide.)
    let r = if (r + hw).fract().abs() < 1e-3 {
        r - 0.01
    } else {
        r
    };
    let stand = |out: [f32; 2], inner: [f32; 2]| {
        let (t, i) = (
            tile_of(obj.local_point(out)),
            tile_of(obj.local_point(inner)),
        );
        stands_by(nav, storey, t, i).then_some(t)
    };
    let row = (f + SIDE_ROW).clamp(-hd + 0.5, hd - 0.5);
    let side = |s: f32| stand([s * (hw + 0.5), row], [s * (hw - 0.5), row]);
    let to = |s: f32| {
        let p = obj.local_point([s * (hw + 0.5), row]);
        (p[0] - from[0]).hypot(p[1] - from[1])
    };
    let first = if def.bunk {
        1.0
    } else if r.abs() > 0.05 {
        r.signum()
    } else if to(1.0) < to(-1.0) {
        1.0
    } else {
        -1.0
    };
    let beside = || side(first).or_else(|| side(-first));
    match def.approach {
        // (The upper bunk is climbed into from the tile in front, round its foot to the ladder.)
        Approach::Side if def.bunk && slot > 0 => None,
        Approach::Side => beside(),
        Approach::Behind => stand([r, -hd - 0.5], [r, -hd + 0.5]).or_else(beside),
        Approach::Front if def.slots > 1 && !def.bunk => stand([r, hd + 0.5], [r, hd - 0.5]),
        _ => None,
    }
}

/// Where someone using `slot` of `obj` standing stands, when not on the object's own tile in
/// front (`approach_tiles`): the tile, the point and the way they face. At the slot's place off
/// the footprint (`places`: the end of a ping-pong table, the oche); for a thing several use at
/// once standing, or two tiles wide, along its front, each in front of their own slot (two at a
/// double sink, one at each basin) or in front of its middle, facing straight in; a carer at a
/// thing with a baby's slot (the crib), in front of the baby; for a use of the `whole` of it (or
/// one that opens or shuts it), in front of its middle. None when that can't be stood on.
pub fn stand_spot(
    content: &Content,
    nav: &NavGrid,
    obj: &ObjectInstance,
    slot: usize,
    whole: bool,
) -> Option<((i32, i32), [f32; 2], f32)> {
    let def = &content.objects[obj.def];
    if !def.places.is_empty() {
        let at = obj.slot_position(content, slot);
        let on = obj.tiles(content).any(|t| t == tile_of(at));
        return (!on)
            .then(|| slot_approach(content, nav, obj, slot))
            .flatten()
            .map(|t| (t, at, obj.slot_yaw(content, slot)));
    }
    let [w, d] = obj.footprint;
    if def.approach != Approach::Front
        || def.stairs
        || def.bunk
        || !matches!(def.layer, Layer::Floor | Layer::Wall)
        || (def.slots < 2 && w % 2 == 1)
    {
        return None;
    }
    let baby = def.interactions.iter().any(|i| i.baby);
    let r = if whole || def.slots < 2 {
        0.0
    } else {
        obj.slot_local(def, if baby { 0 } else { slot })[0]
    };
    let [hw, hd] = [w as f32 / 2.0, d as f32 / 2.0];
    let nudged = if (r + hw).fract().abs() < 1e-3 {
        r - 0.01
    } else {
        r
    };
    let t = tile_of(obj.local_point([nudged, hd + 0.5]));
    let inner = tile_of(obj.local_point([nudged, hd - 0.5]));
    if !stands_by(nav, nav.stairs.storey(obj.z), t, inner) {
        return None;
    }
    // As it's drawn (its offset), on that tile.
    let p = obj.local_point([r, hd + 0.5]);
    let o = [obj.offset[0] as f32 / 100.0, obj.offset[1] as f32 / 100.0];
    let p = [0, 1].map(|k| (p[k] + o[k]).clamp([t.0, t.1][k] as f32, [t.0, t.1][k] as f32 + 1.0));
    Some((t, p, crate::world::wrap_angle(obj.angle() + PI)))
}

/// `stand_spot`, if someone at `pos` has come to it (within 0.75 m): the point and the way
/// they face there.
pub fn at_stand_spot(
    content: &Content,
    nav: &NavGrid,
    obj: &ObjectInstance,
    slot: usize,
    whole: bool,
    pos: [f32; 2],
) -> Option<([f32; 2], f32)> {
    stand_spot(content, nav, obj, slot, whole)
        .filter(|&(_, p, _)| (p[0] - pos[0]).hypot(p[1] - pos[1]) < 0.75)
        .map(|(_, p, yaw)| (p, yaw))
}

/// The table `seat` faces: the top covering its front tile, of a kind it tucks under (its
/// `seatAt`), on the same storey with no wall between (`top_at`: the top over a tile on the lot).
pub(crate) fn faced(
    content: &Content,
    objects: &[ObjectInstance],
    nav: &NavGrid,
    seat: &ObjectInstance,
    top_at: impl Fn(i32, i32) -> Option<u32>,
) -> Option<u32> {
    let kinds = content.objects[seat.def].seat_at;
    if kinds == 0 {
        return None;
    }
    let (fx, fz) = seat.front_tile(content);
    let (bx, bz) = match seat.rot % 4 {
        0 => (fx, fz - 1),
        1 => (fx - 1, fz),
        2 => (fx, fz + 1),
        _ => (fx + 1, fz),
    };
    let lot = nav.lot;
    if !lot.in_bounds(fx, fz)
        || nav.stairs.storey(fz) != nav.stairs.storey(bz)
        || lot.edge_between(bx, bz, fx, fz) != Edge::Open
    {
        return None;
    }
    let host = top_at(fx, fz)?;
    let s = content.objects[objects[host as usize].def].surface?;
    (s.kind.bit() & kinds != 0).then_some(host)
}

/// The table `seat` faces, if it is a seat that tucks under one (see `Links::table`); none at a
/// top with seats of its own (`seats_itself`).
pub fn faced_table(
    content: &Content,
    objects: &[ObjectInstance],
    nav: &NavGrid,
    seat: u32,
) -> Option<u32> {
    let o = objects.get(seat as usize)?;
    faced(content, objects, nav, o, |x, z| {
        host_at(content, objects, x, z)
    })
    .filter(|&h| !seats_itself(content, &objects[h as usize]))
}

/// A top sat at by a use of its own (a dining set with its chairs built in, a desk with its
/// chair): a seat bought facing it stands behind or beside those chairs, a metre from the top,
/// so it isn't pulled up to it (its `needsTable` uses aren't offered there).
pub fn seats_itself(content: &Content, obj: &ObjectInstance) -> bool {
    content.objects[obj.def]
        .interactions
        .iter()
        .any(|i| i.pose == Pose::Sit)
}

/// The nearest place to `near` on `top` where a disc of `radius` fits among `discs`, no
/// further than `max_m`, with how far it is.
fn nearest_on(
    top: &Top,
    near: [f32; 2],
    max_m: f32,
    radius: f32,
    discs: &[([f32; 2], f32)],
    extra: &[([f32; 2], f32)],
) -> Option<(f32, SurfaceSpot)> {
    let room = top.room(radius);
    if room[0] < 0.0 || room[1] < 0.0 {
        return None;
    }
    let fits = |p: [f32; 2]| -> Option<SurfaceSpot> {
        let spot = SurfaceSpot::at(top.host, p, top.surface.height);
        let c = spot.centre();
        (top.holds(c, radius) && !overlaps(discs, c, radius) && !overlaps(extra, c, radius))
            .then_some(spot)
    };
    // Right there (as near as the edges allow) if it's free.
    let l = top.local(near);
    let first = top.world([l[0].clamp(-room[0], room[0]), l[1].clamp(-room[1], room[1])]);
    let dist = |p: [f32; 2]| (p[0] - near[0]).hypot(p[1] - near[1]);
    if dist(first) <= max_m
        && let Some(s) = fits(first)
    {
        return Some((dist(s.centre()), s));
    }
    // Else the nearest free point on a grid over the top.
    let (nx, nz) = (
        (room[0] / STEP).floor() as i32,
        (room[1] / STEP).floor() as i32,
    );
    let mut best: Option<(f32, SurfaceSpot)> = None;
    for iz in -nz..=nz {
        for ix in -nx..=nx {
            let p = top.world([ix as f32 * STEP, iz as f32 * STEP]);
            let dp = dist(p);
            if dp > max_m || best.is_some_and(|b| dp >= b.0) {
                continue;
            }
            if let Some(s) = fits(p) {
                best = Some((dp, s));
            }
        }
    }
    best
}

/// A free place on a surface near `near` (within `max_m`), for something of `radius` on one of
/// the surface `kinds` (a mask of `SurfaceKind::bit`; 0: any), among `candidates` (object ids, e.g.
/// what the household may use). `extra` lists more things already standing on surfaces (plated
/// food: middle and radius). Nearest first.
#[allow(clippy::too_many_arguments)]
pub fn free_surface_spot_near(
    content: &Content,
    objects: &[ObjectInstance],
    candidates: &[u32],
    near: [f32; 2],
    max_m: f32,
    kinds: u8,
    radius: f32,
    extra: &[([f32; 2], f32)],
) -> Option<SurfaceSpot> {
    let mut discs = Vec::new();
    let mut best: Option<(f32, SurfaceSpot)> = None;
    for &id in candidates {
        let Some(top) = objects.get(id as usize).and_then(|o| Top::of(content, o)) else {
            continue;
        };
        if kinds != 0 && kinds & top.surface.kind.bit() == 0 {
            continue;
        }
        // Too far even to its nearest corner.
        let reach = top.half[0].hypot(top.half[1]);
        if (top.centre[0] - near[0]).hypot(top.centre[1] - near[1]) - reach
            > max_m.min(best.map_or(f32::INFINITY, |b| b.0))
        {
            continue;
        }
        clutter_on(content, objects, &top, &mut discs);
        if let Some(found) = nearest_on(&top, near, max_m, radius, &discs, extra)
            && best.is_none_or(|b| found.0 < b.0)
        {
            best = Some(found);
        }
    }
    best.map(|b| b.1)
}

/// The top a seat's diner eats at, and where on it right in front of them (`slot`): a table it
/// is pulled up to, else one it faces (a sofa at a coffee table), else its own top (a dining
/// set with its seats built in).
fn plate_target(
    content: &Content,
    objects: &[ObjectInstance],
    links: &Links,
    seat: u32,
    slot: usize,
) -> Option<(Top, [f32; 2])> {
    let s = objects.get(seat as usize)?;
    let def = &content.objects[s.def];
    let slot = slot.min(def.slots.max(1) as usize - 1);
    let at = s.slot_position(content, slot);
    // (The way they face in it: across the table from a place of their own, `places`.)
    let a = if def.places.is_empty() {
        angle(s)
    } else {
        s.slot_yaw(content, slot)
    };
    let forward = [a.sin(), a.cos()];
    let ahead = |t: f32| [at[0] + forward[0] * t, at[1] + forward[1] * t];
    if let Some(top) = Top::of(content, s) {
        return Some((top, ahead(PLATE_AHEAD)));
    }
    let (fx, fz) = s.front_tile(content);
    let host = links.table_of(seat).or_else(|| {
        let host = host_at(content, objects, fx, fz)?;
        (content.objects[s.def]
            .interactions
            .iter()
            .any(|i| i.pose == Pose::Sit))
        .then_some(host)
    })?;
    let top = Top::of(content, objects.get(host as usize)?)?;
    // Where the line straight ahead of the diner meets the top's edge, then a little way in.
    let (o, f) = (top.local(at), top.local(ahead(1.0)));
    let dir = [f[0] - o[0], f[1] - o[1]];
    let mut enter: f32 = 0.0;
    for k in 0..2 {
        if dir[k].abs() > 1e-4 {
            let (t1, t2) = (
                (-top.half[k] - o[k]) / dir[k],
                (top.half[k] - o[k]) / dir[k],
            );
            enter = enter.max(t1.min(t2));
        }
    }
    Some((top, ahead(enter + PLATE_IN)))
}

/// Where the plate goes for someone sitting in `slot` of `seat`: on the table in front of them.
pub fn spot_for_seat(
    content: &Content,
    objects: &[ObjectInstance],
    links: &Links,
    seat: u32,
    slot: usize,
    radius: f32,
    extra: &[([f32; 2], f32)],
) -> Option<SurfaceSpot> {
    let (top, target) = plate_target(content, objects, links, seat, slot)?;
    let mut discs = Vec::new();
    clutter_on(content, objects, &top, &mut discs);
    nearest_on(&top, target, PLATE_SEARCH, radius, &discs, extra).map(|f| f.1)
}

/// The best free seat to eat at among `candidates` for someone in life stage `stage`, nearest
/// within a rank, from `from`: a chair pulled up to a table, else a table with its seats built
/// in, else anywhere comfortable to sit (not a toilet or a desk chair). Within a rank, a seat
/// made for their age comes first: a child has a snack at the kids' table, and then one whose
/// place at the table is clear of `taken` (things standing on tops, `Items::occupancy`): a plate
/// goes in front of them, not beside the dirty one left there.
pub fn dining_seat(
    content: &Content,
    objects: &[ObjectInstance],
    links: &Links,
    candidates: &[u32],
    from: [f32; 2],
    stage: usize,
    taken: &[([f32; 2], f32)],
) -> Option<DiningSeat> {
    let tag = |name: &str| {
        content
            .tags
            .iter()
            .position(|t| t == name)
            .map_or(0, |i| 1u64 << i)
    };
    let food = tag("food");
    let comfy = food | tag("lounge");
    // (Every stage but a baby's: a seat for fewer is made for them.)
    let anyone = content.life.not_baby;
    let mut best: Option<(DiningRank, bool, bool, f32, u32, Option<u32>)> = None;
    for &id in candidates {
        let Some(o) = objects.get(id as usize) else {
            continue;
        };
        let def = &content.objects[o.def];
        if o.broken() || !o.has_free_slot(def.slots) {
            continue;
        }
        let sits = |mask: u64| o.seats_now(def, stage).any(|t| mask == 0 || t & mask != 0);
        let (rank, table) = match links.table_of(id) {
            Some(t) if def.seat_at != 0 && sits(0) => (DiningRank::ChairAtTable, Some(t)),
            _ if def.surface.is_some() && food != 0 && sits(food) => {
                (DiningRank::TableSeat, Some(id))
            }
            _ if sits(comfy) => (DiningRank::AnySeat, None),
            _ => continue,
        };
        let anyones = !def
            .interactions
            .iter()
            .any(|i| i.pose == Pose::Sit && i.for_stage(stage) && i.stages & anyone != anyone);
        let slot = o.users[..def.slots as usize]
            .iter()
            .position(Option::is_none)
            .unwrap_or(0);
        let cluttered = table.is_some()
            && plate_target(content, objects, links, id, slot).is_some_and(|(_, at)| {
                taken
                    .iter()
                    .any(|(c, r)| (c[0] - at[0]).hypot(c[1] - at[1]) < r + DEFAULT_RADIUS * 0.5)
            });
        let c = o.centre(content);
        let d = (c[0] - from[0]).hypot(c[1] - from[1]);
        if best.is_none_or(|b| (rank, anyones, cluttered, d) < (b.0, b.1, b.2, b.3)) {
            best = Some((rank, anyones, cluttered, d, id, table));
        }
    }
    let (rank, _, _, _, object, table) = best?;
    let o = &objects[object as usize];
    let slot = o.users[..content.objects[o.def].slots as usize]
        .iter()
        .position(Option::is_none)
        .unwrap_or(0);
    Some(DiningSeat {
        object,
        rank,
        table,
        spot: spot_for_seat(content, objects, links, object, slot, DEFAULT_RADIUS, &[]),
    })
}
