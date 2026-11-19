//! Cleaning the floor with a tool (content `cleans`, `content::Sweep`): the vacuum cleaner, the mop
//! and bucket, the broom.
//!
//! Tidying up (`roomRules.clean`) takes the dirt off a spot and the tiles round it with the bare
//! hands. A use that `cleans` is tidying up with a tool: free will offers it only with a floor at
//! home dirtier than `roomRules.clean.threshold` that the tool cleans (a mop and a broom leave a
//! carpet; only a broom goes out into the garden), the keener the dirtier and the more of the room
//! is dirty. The resident takes the tool (the renderer draws it in their hands, content `held`, and
//! hides it where it's kept) to the dirtiest such spot (`spot`, every storey of the home), and uses
//! it there; done, the dirt comes off every tile of that room the tool cleans within its `reach`
//! (`clean`, after the step). Two residents never clean the same spot at once: the spot is
//! someone's while they're at it (`World::cleaning_spot`, the briefs' `cleaning`), as tidying up's.
//! Where it's used isn't saved: a use under way starts again after a load, and finds the dirtiest
//! spot then.

use crate::content::{Content, Sweep};
use crate::lot::{Lot, OUTDOORS};
use crate::path::StairMap;
use crate::rooms::RoomInfo;
use crate::world::{Plot, World};

/// Where a tool is taken to clean: the dirtiest tile it cleans, its dirt, and how many more tiles
/// as dirty as is worth cleaning it would clean there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Spot {
    pub x: i32,
    pub z: i32,
    pub dirt: f32,
    pub more: u32,
}

/// How much keener a resident is on a tool for each more tile it would clean (`keen`), up to
/// `MOST_MORE` of them: a room dirty all over is worth the vacuum more than a crumb.
const PER_MORE: f32 = 0.1;
const MOST_MORE: u32 = 5;

/// How keen on cleaning `spot` (as tidying up a spot that dirty is keen, `roomRules.clean`'s
/// `interest` times its dirt), the keener the more of the room it cleans.
pub(crate) fn keen(content: &Content, spot: &Spot) -> f32 {
    content.room_rules.clean.interest
        * spot.dirt
        * (1.0 + PER_MORE * spot.more.min(MOST_MORE) as f32)
}

/// Rows of plot `p` on every storey (each storey's block of rows, `storeys.rs`), the ground's
/// first: `(z, ground)`.
fn plot_rows(lot: &Lot, stairs: &StairMap, p: &Plot) -> impl Iterator<Item = (i32, bool)> {
    let depth = stairs.depth;
    let blocks = if depth > 0 {
        (lot.depth as i32 / depth).max(1)
    } else {
        1
    };
    (0..blocks).flat_map(move |b| (p.z..p.z + p.d).map(move |z| (z + b * depth, b == 0)))
}

/// Whether the floor of room `room` is soft where no covering was laid: its kind's `carpet` (a
/// bedroom). Kept per room as asked (`seen`).
fn carpet_room(
    content: &Content,
    rooms: &[RoomInfo],
    seen: &mut Vec<(u16, bool)>,
    room: u16,
) -> bool {
    if let Some(&(_, c)) = seen.iter().find(|(r, _)| *r == room) {
        return c;
    }
    let c = rooms
        .iter()
        .find(|r| !r.garden && r.id == room)
        .and_then(|r| r.kind)
        .is_some_and(|k| content.room_kinds[k].carpet);
    seen.push((room, c));
    c
}

/// Whether `sweep` cleans tile `(x, z)` (room `room`; `ground`: on the ground storey): a floor
/// indoors, hard for a mop or a broom (`Sweep::hard`: no carpet laid, nor a carpeted room's
/// own floor), and the garden for a broom (`Sweep::outdoors`).
#[allow(clippy::too_many_arguments)]
fn cleans(
    content: &Content,
    lot: &Lot,
    rooms: &[RoomInfo],
    seen: &mut Vec<(u16, bool)>,
    sweep: &Sweep,
    x: i32,
    z: i32,
    room: u16,
    ground: bool,
) -> bool {
    if room == OUTDOORS {
        return sweep.outdoors && ground;
    }
    if !sweep.hard {
        return true;
    }
    match lot.floor(x as u16, z as u16) {
        0 => !carpet_room(content, rooms, seen, room),
        c => !content
            .build
            .floors
            .get(c as usize - 1)
            .is_some_and(|f| f.soft),
    }
}

/// Whether `(x, z)` is within `sweep`'s reach of `(sx, sz)` (every tile with a reach of 0).
fn within(sweep: &Sweep, sx: i32, sz: i32, x: i32, z: i32) -> bool {
    sweep.reach == 0 || ((x - sx).abs().max((z - sz).abs()) <= i32::from(sweep.reach))
}

/// The dirtiest tile at home (plot `p`, every storey) that `sweep` cleans, dirtier than
/// `threshold`, that a resident can stand on and isn't one of `skip` (someone else's, or out of
/// reach a moment ago); with how many more tiles over `threshold` it would clean there.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spot(
    content: &Content,
    nav: &crate::path::NavGrid,
    rooms: &[RoomInfo],
    dirt: &[f32],
    p: &Plot,
    sweep: &Sweep,
    threshold: f32,
    skip: &[(i32, i32)],
) -> Option<Spot> {
    let lot = nav.lot;
    let mut seen = Vec::new();
    let at = |x: i32, z: i32| dirt.get(lot.tile_index(x, z)).copied().unwrap_or(0.0);
    let mut best: Option<Spot> = None;
    for (z, ground) in plot_rows(lot, nav.stairs, p) {
        for x in p.x..p.x + p.w {
            if !nav.tile_free(x, z) || skip.contains(&(x, z)) {
                continue;
            }
            let d = at(x, z);
            if d > threshold
                && best.is_none_or(|b| d > b.dirt)
                && cleans(
                    content,
                    lot,
                    rooms,
                    &mut seen,
                    sweep,
                    x,
                    z,
                    lot.room_at(x, z),
                    ground,
                )
            {
                best = Some(Spot {
                    x,
                    z,
                    dirt: d,
                    more: 0,
                });
            }
        }
    }
    let mut best = best?;
    let room = lot.room_at(best.x, best.z);
    for (z, ground) in plot_rows(lot, nav.stairs, p) {
        for x in p.x..p.x + p.w {
            if (x, z) != (best.x, best.z)
                && lot.allocated(x, z)
                && lot.room_at(x, z) == room
                && within(sweep, best.x, best.z, x, z)
                && at(x, z) > threshold
                && cleans(content, lot, rooms, &mut seen, sweep, x, z, room, ground)
            {
                best.more += 1;
            }
        }
    }
    Some(best)
}

/// A floor cleaned with a tool from `(x, z)` (applied after the step, as tidying up's): the
/// dirt comes off every tile of its room on the plot that `sweep` cleans and reaches, by its
/// `amount`.
pub(crate) fn clean(w: &mut World, x: i32, z: i32, sweep: &Sweep) {
    if !w.lot.in_bounds(x, z) {
        return;
    }
    let Some(plot) = w.plot_at(x, w.ground_z(z)) else {
        return;
    };
    let p = w.plots[plot as usize].clone();
    let room = w.lot.room_at(x, z);
    let mut seen = Vec::new();
    let rows: Vec<(i32, bool)> = plot_rows(&w.lot, &w.stairs, &p).collect();
    for (tz, ground) in rows {
        for tx in p.x..p.x + p.w {
            if !w.lot.allocated(tx, tz)
                || w.lot.room_at(tx, tz) != room
                || !within(sweep, x, z, tx, tz)
                || !cleans(
                    &w.content, &w.lot, &w.rooms, &mut seen, sweep, tx, tz, room, ground,
                )
            {
                continue;
            }
            let t = w.lot.tile_index(tx, tz);
            if w.dirt.get(t).is_some_and(|&d| d > 0.0) {
                w.dirt[t] = (w.dirt[t] - sweep.amount).max(0.0);
                w.room_tiles.touch(t);
            }
        }
    }
}
