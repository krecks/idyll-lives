//! Approach tiles: where a resident stands to use an object or a seat.

use super::*;

/// The tile a resident stands on to use `obj`: its front tile (or, should that be taken, the
/// other of the two middle tiles in front of a thing an even number of tiles wide; a seat with
/// neither free, from beside it); for seats used from around, the first free one of the tiles in
/// front of, behind, left and right of it.
pub fn approach_tile(content: &Content, nav: &NavGrid, obj: &ObjectInstance) -> (i32, i32) {
    approach_tiles(content, nav, obj)
        .next()
        .unwrap_or_else(|| obj.front_tile(content))
}

/// The tiles a resident may use `obj` from, best first: its front tile (whether or not it is
/// free; along an even front edge, the other middle tile comes first if the front can't be stood
/// on and it can: a thing two tiles wide is used from either, as which counts as the front swaps
/// when the house it's in is turned round; a seat whose front can't be stood on, from the tiles
/// left and right of it that can first: an armchair with a footstool in front of it is sat in
/// from beside it); for seats used from around, each tile in front of, behind, left and right of its
/// middle that can be stood on, on the same storey with no wall or fence between; for a seat behind
/// the work (`Approach::Behind`: a desk, sat at from behind it), the tiles behind, left and right
/// of it that can be stood on, then its front if it can be, else (along an even front edge) the
/// other middle tile if that can (a desk may stand with its screen to a wall).
pub fn approach_tiles<'a>(
    content: &Content,
    nav: &'a NavGrid,
    obj: &ObjectInstance,
) -> impl Iterator<Item = (i32, i32)> + 'a {
    let around = content.objects[obj.def].approach == Approach::Around;
    let behind = content.objects[obj.def].approach == Approach::Behind;
    let front = obj.front_tile(content);
    let (w, d) = obj.size(content);
    let (x, z) = (obj.x, obj.z);
    let (cx, cz) = (x + (w - 1) / 2, z + (d - 1) / 2);
    // Beside each side (+z, +x, -z, -x): the tile out there and the one of its own next to it.
    let sides = [
        ((cx, z + d), (cx, z + d - 1)),
        ((x + w, cz), (x + w - 1, cz)),
        ((cx, z - 1), (cx, z)),
        ((x - 1, cz), (x, cz)),
    ];
    let r = (obj.rot % 4) as usize;
    let order = if behind {
        [(r + 2) % 4, (r + 3) % 4, (r + 1) % 4, r]
    } else {
        [r, (r + 2) % 4, (r + 3) % 4, (r + 1) % 4]
    };
    let storey = nav.stairs.storey(z);
    let standable = move |(t, inner): ((i32, i32), (i32, i32))| stands_by(nav, storey, t, inner);
    // The other middle tile along an even front edge (the front tile is the lower of the two).
    let twin = match r {
        0 | 2 if w % 2 == 0 => Some((
            (sides[r].0.0 + 1, sides[r].0.1),
            (sides[r].1.0 + 1, sides[r].1.1),
        )),
        1 | 3 if d % 2 == 0 => Some((
            (sides[r].0.0, sides[r].0.1 + 1),
            (sides[r].1.0, sides[r].1.1 + 1),
        )),
        _ => None,
    };
    let instead = twin
        .filter(|&t| !around && !standable(sides[r]) && standable(t))
        .map(|(t, _)| t);
    // (A seat with its front taken, a footstool there: from its left or right first, else still the front.)
    let seat = content.objects[obj.def]
        .interactions
        .iter()
        .any(|i| i.pose == Pose::Sit);
    let from_side = !around && !behind && instead.is_none() && seat && !standable(sides[r]);
    // (A desk's front only if it can be stood on, else the other middle tile if that can: it may
    // have its screen to a wall, and its front swaps too when the house is turned round.)
    let only_front = if behind {
        standable(sides[r]).then_some(front).or(instead)
    } else {
        (!around).then_some(front)
    };
    let (skip, take) = if around || behind {
        (0, 4 - behind as usize)
    } else if from_side {
        (2, 2)
    } else {
        (4, 0)
    };
    let beside = order
        .into_iter()
        .skip(skip)
        .take(take)
        .map(move |i| sides[i])
        .filter_map(move |s| standable(s).then_some(s.0));
    let (front_first, front_last) = if from_side || behind {
        (None, only_front)
    } else {
        (only_front, None)
    };
    instead
        .filter(|_| !behind)
        .into_iter()
        .chain(front_first)
        .chain(beside)
        .chain(front_last)
}

/// The tiles a resident may use `obj`, a thing standing on `host`'s top, from, best first: at
/// the top's edge by it, from the floor beside the tile it stands over (the top's front first,
/// then its left, right and back), else anywhere along the top's edge, nearest it first. Each
/// can be stood on, on the same storey, with no wall or fence between it and the top. A kettle
/// is used from in front of its counter, a book on a table from beside it. Empty when the top
/// stands with no free side (then it is used from the top's own approach tile, `Links::new`).
pub fn rider_tiles(
    content: &Content,
    objects: &[ObjectInstance],
    nav: &NavGrid,
    obj: &ObjectInstance,
    host: u32,
) -> Vec<(i32, i32)> {
    let Some(h) = objects.get(host as usize) else {
        return Vec::new();
    };
    let (w, d) = h.size(content);
    let on = |x: i32, z: i32| x >= h.x && x < h.x + w && z >= h.z && z < h.z + d;
    let p = middle(content, obj);
    let under = (p[0].floor() as i32, p[1].floor() as i32);
    let under = if on(under.0, under.1) {
        under
    } else {
        (
            under.0.clamp(h.x, h.x + w - 1),
            under.1.clamp(h.z, h.z + d - 1),
        )
    };
    let storey = nav.stairs.storey(h.z);
    let standable =
        |inner: (i32, i32), t: (i32, i32)| !on(t.0, t.1) && stands_by(nav, storey, t, inner);
    // The way the top faces (+z, +x, -z, -x), then its left, right and back.
    const STEPS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    let r = (h.rot % 4) as usize;
    let mut tiles: Vec<(i32, i32)> = [r, (r + 3) % 4, (r + 1) % 4, (r + 2) % 4]
        .into_iter()
        .map(|i| (under.0 + STEPS[i].0, under.1 + STEPS[i].1))
        .filter(|&t| standable(under, t))
        .collect();
    if tiles.is_empty() {
        let mut edge: Vec<(f32, (i32, i32))> = h
            .tiles(content)
            .flat_map(|inner| STEPS.map(|s| (inner, (inner.0 + s.0, inner.1 + s.1))))
            .filter(|&(inner, t)| standable(inner, t))
            .map(|(_, t)| ((t.0 as f32 + 0.5 - p[0]).hypot(t.1 as f32 + 0.5 - p[1]), t))
            .collect();
        edge.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        edge.dedup_by_key(|e| e.1);
        tiles = edge.into_iter().map(|e| e.1).collect();
    }
    tiles
}

/// Every tile a resident may use `obj` from, best first (`approach_tiles`; for a thing on a
/// top, `rider_tiles`; with the top's sides all taken, the tiles of the seats pulled up to it,
/// as they sit down to use it there, else the top's).
pub fn use_tiles(
    content: &Content,
    objects: &[ObjectInstance],
    nav: &NavGrid,
    obj: &ObjectInstance,
) -> Vec<(i32, i32)> {
    match host_of(content, objects, obj) {
        Some(h) => {
            let tiles = rider_tiles(content, objects, nav, obj, h);
            if !tiles.is_empty() {
                return tiles;
            }
            let seats: Vec<(i32, i32)> = objects
                .iter()
                .filter(|s| faced_table(content, objects, nav, s.id) == Some(h))
                .flat_map(|s| approach_tiles(content, nav, s).collect::<Vec<_>>())
                .collect();
            if seats.is_empty() {
                approach_tiles(content, nav, &objects[h as usize]).collect()
            } else {
                seats
            }
        }
        None => approach_tiles(content, nav, obj).collect(),
    }
}

/// A free seat pulled up to the top thing `object` stands on (`Links::table_of`), for someone
/// in life stage `stage`, the nearest to it first: they sit there to use a laptop on a desk.
pub fn seat_at_top(
    content: &Content,
    objects: &[ObjectInstance],
    links: &Links,
    object: u32,
    sim: u32,
    stage: usize,
) -> Option<u32> {
    let host = links.host_of(object)?;
    let p = middle(content, objects.get(object as usize)?);
    let rules = &content.cooking;
    links
        .seats_at(host)
        .iter()
        .filter_map(|&id| objects.get(id as usize))
        .filter(|o| {
            let def = &content.objects[o.def];
            !o.broken()
                && o.can_use(sim, def.slots)
                && o.sits(def, stage, rules.seat_tags, rules.seat_avoid)
        })
        .map(|o| {
            let c = middle(content, o);
            ((c[0] - p[0]).hypot(c[1] - p[1]), o.id)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, o)| o)
}

/// Whether a resident can stand on tile `t` to use a thing on storey `storey` from it: free, on
/// that storey, not on a stair, and (beside its tile `inner`, on the lot) with no wall or fence
/// between.
pub(crate) fn stands_by(nav: &NavGrid, storey: i32, t: (i32, i32), inner: (i32, i32)) -> bool {
    nav.tile_free(t.0, t.1)
        && nav.lot.in_bounds(inner.0, inner.1)
        && nav.stairs.storey(t.1) == storey
        && nav.stairs.step(t.0, t.1).is_none()
        // (Diagonally beside it, where the place is off a corner: no one edge between.)
        && ((t.0 - inner.0).abs() + (t.1 - inner.1).abs() != 1 || nav.lot.edge_between(inner.0, inner.1, t.0, t.1) == Edge::Open)
}

/// The tile a resident uses `slot` of `obj` from when the slot has its own place (content
/// `places`): the place's own tile when it's off the footprint (standing at a ping-pong table's
/// end), else the first tile off the footprint behind it, against the way its user faces (round
/// to a picnic table's far bench). None without a place, or when that tile can't be stood on
/// (see `approach_tiles`): then they come from the object's own.
pub fn slot_approach(
    content: &Content,
    nav: &NavGrid,
    obj: &ObjectInstance,
    slot: usize,
) -> Option<(i32, i32)> {
    content.objects[obj.def].places.get(slot)?;
    let at = obj.slot_position(content, slot);
    let yaw = obj.slot_yaw(content, slot);
    let on = |t: (i32, i32)| obj.tiles(content).any(|f| f == t);
    let storey = nav.stairs.storey(obj.z);
    if !on(tile_of(at)) {
        // Off it: on the place's own tile (two tiles out at the oche), with nothing in the way
        // to the next tile towards the thing.
        let t = tile_of(at);
        let inner = (1..=20)
            .map(|k| {
                tile_of([
                    at[0] + yaw.sin() * 0.1 * k as f32,
                    at[1] + yaw.cos() * 0.1 * k as f32,
                ])
            })
            .find(|&p| p != t)?;
        return stands_by(nav, storey, t, inner).then_some(t);
    }
    // Against the way they face from a place on it (out of it), until the footprint's edge is
    // crossed: `t` off it, `inner` on it; else from beside the place (round the end of a bench).
    let out = |a: f32| {
        let mut last = tile_of(at);
        (1..=40).find_map(|k| {
            let p = tile_of([
                at[0] - a.sin() * 0.1 * k as f32,
                at[1] - a.cos() * 0.1 * k as f32,
            ]);
            let crossed = !on(p) && on(last);
            let found = crossed.then_some((p, last));
            last = p;
            found
        })
    };
    [yaw, yaw + FRAC_PI_2, yaw - FRAC_PI_2]
        .into_iter()
        .filter_map(out)
        .find(|&(t, inner)| stands_by(nav, storey, t, inner))
        .map(|(t, _)| t)
}
