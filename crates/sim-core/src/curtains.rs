//! Curtains drawn and opened (`ObjectInstance::shut`).
//!
//! Each pair of curtains (a thing hung over a window: content `overWindow`) is open or drawn shut
//! on its own: the player draws or opens one with a click (`Command::DrawCurtains`). One bought
//! comes open. Residents draw a room's curtains together, as `lights.rs` switches its lamps, on
//! these moments only (so what the player set stays until the next one):
//! - someone goes to sleep in a room: its curtains close;
//! - night falls (`lights::dark_at`) on a room with someone awake in it, or someone awake comes
//!   into a room at night: they close;
//! - day breaks on a room with someone awake in it, or someone awake (up for the day, or from a
//!   nap) comes into a room by day: they open, unless someone is still asleep in it.
//!
//! A new game's lot is drawn as fits the hour (`Watch::fresh`): shut at night and where someone
//! sleeps, open by day. A save opens with its curtains as they were.
//!
//! Only at home, where the household living there has free will: with it off, and in public
//! places, the curtains stay as the player left them. The renderer holds a shut curtain's
//! `motions.night` clip (drawn) instead of the clock's, and a lit room's window throws less light
//! on the garden behind it. A curtain moved, undone or redone stays as it was.

use crate::World;
use crate::lot::OUTDOORS;

/// How often (ticks) residents' comings, goings and sleep are looked at: as the lamps are.
const EVERY: u64 = crate::secs(0.25);

/// What the residents' rule saw last time (not saved: on load it starts from what's there,
/// drawing nothing).
#[derive(Debug, Default, Clone)]
pub(crate) struct Watch {
    /// Rooms with someone awake in them, and with someone asleep in them, sorted.
    awake: Vec<u16>,
    asleep: Vec<u16>,
    dark: bool,
    /// The lot (walls) these room ids were for; after a rebuild the rooms are seen afresh.
    lot_version: u32,
    primed: bool,
    /// A new game's lot, not looked at yet: its curtains are drawn as fits the hour.
    pub(crate) fresh: bool,
    /// The curtains by room, found for the objects and the rooms as they stood
    /// (`World::objects_version` and the lot's version).
    by_room: Option<((u32, u32), ByRoom)>,
}

/// Whether object kind `def` is curtains (hung over a window: content `overWindow`).
pub fn is_curtain(world: &World, def: usize) -> bool {
    world.content.objects[def].over_window
}

/// Draws one pair of curtains shut or opens it; false if it was so already (or isn't curtains).
pub(crate) fn set(world: &mut World, object: u32, shut: bool) -> bool {
    let Some(o) = world.objects.get(object as usize) else {
        return false;
    };
    if !is_curtain(world, o.def) || o.shut == shut {
        return false;
    }
    world.objects[object as usize].shut = shut;
    world.curtains_version = world.curtains_version.wrapping_add(1);
    true
}

/// The curtains drawn shut (stable ids), for the renderer.
pub fn shut(world: &World) -> Vec<u32> {
    world
        .objects
        .iter()
        .filter(|o| o.shut && is_curtain(world, o.def))
        .map(|o| o.stable)
        .collect()
}

/// The residents' rule (see the module notes); runs every `EVERY` ticks.
pub(crate) fn update(world: &mut World) {
    if !world.tick.is_multiple_of(EVERY) {
        return;
    }
    let content = &world.content;
    // (One pass over the residents for both: a baby asleep counts, a baby awake draws nothing.)
    let (mut awake, mut asleep) = (Vec::new(), Vec::new());
    for s in world
        .sims
        .iter()
        .filter(|s| s.here() && s.away_until.is_none())
    {
        let (x, z) = s.tile();
        let room = world.lot.room_at(x, z);
        if room == OUTDOORS {
            continue;
        }
        if s.asleep(content) {
            asleep.push(room);
        } else if !content.life.baby(s.age) {
            awake.push(room);
        }
    }
    for rooms in [&mut awake, &mut asleep] {
        rooms.sort_unstable();
        rooms.dedup();
    }
    let dark = crate::lights::dark_at(crate::clock::minute_of_day(world.tick));
    let lot_version = world.lot_version();
    let mut watch = std::mem::take(&mut world.curtain_watch);
    let key = (world.objects_version, lot_version);
    let mut by_room = watch.by_room.take().filter(|c| c.0 == key).map(|c| c.1);
    if watch.fresh {
        // (Not the residents' doing: the lot comes so, free will or not.)
        let by_room = by_room.get_or_insert_with(|| ByRoom::new(world));
        for &(room, id) in &by_room.curtains {
            set(world, id, dark || asleep.binary_search(&room).is_ok());
        }
        watch.fresh = false;
    }
    if watch.primed && watch.lot_version == lot_version && world.autonomy {
        let turned = watch.dark != dark;
        let slept: Vec<u16> = asleep
            .iter()
            .copied()
            .filter(|r| watch.asleep.binary_search(r).is_err())
            .collect();
        let came: Vec<u16> = awake
            .iter()
            .copied()
            .filter(|r| turned || watch.awake.binary_search(r).is_err())
            .collect();
        if !slept.is_empty() || !came.is_empty() {
            let by_room = by_room.get_or_insert_with(|| ByRoom::new(world));
            for room in slept {
                draw_room(world, by_room, room, true);
            }
            for room in came {
                if dark {
                    draw_room(world, by_room, room, true);
                } else if asleep.binary_search(&room).is_err() {
                    draw_room(world, by_room, room, false);
                }
            }
        }
    }
    watch.by_room = by_room.map(|c| (key, c));
    watch.awake = awake;
    watch.asleep = asleep;
    watch.dark = dark;
    watch.lot_version = lot_version;
    watch.primed = true;
    world.curtain_watch = watch;
}

/// The curtains by the room they hang in (`room_at` their middle, the tile in front of their
/// wall), each room's in id order: found once for a pass over the rooms.
#[derive(Debug, Clone)]
struct ByRoom {
    curtains: Vec<(u16, u32)>,
}

impl ByRoom {
    fn new(world: &World) -> Self {
        let mut curtains: Vec<(u16, u32)> = world
            .objects
            .iter()
            .filter(|o| is_curtain(world, o.def))
            .map(|o| {
                let [x, z] = o.centre(&world.content);
                (world.lot.room_at(x.floor() as i32, z.floor() as i32), o.id)
            })
            .filter(|&(room, _)| room != OUTDOORS)
            .collect();
        curtains.sort_unstable();
        Self { curtains }
    }

    /// The curtains hanging in `room`, in id order.
    fn in_room(&self, room: u16) -> impl Iterator<Item = u32> + '_ {
        let from = self.curtains.partition_point(|&(r, _)| r < room);
        self.curtains[from..]
            .iter()
            .take_while(move |&&(r, _)| r == room)
            .map(|&(_, id)| id)
    }
}

/// A resident draws (or opens) every pair of curtains in `room`, if it's in a home whose
/// household has free will.
fn draw_room(world: &mut World, by_room: &ByRoom, room: u16, shut: bool) {
    let curtains: Vec<u32> = by_room.in_room(room).collect();
    let Some(&first) = curtains.first() else {
        return;
    };
    let o = &world.objects[first as usize];
    let Some(plot) = world.plot_at(o.x, o.z) else {
        return;
    };
    let mut living = world
        .households
        .iter()
        .filter(|h| h.plot == Some(plot))
        .peekable();
    if living.peek().is_none() || !living.all(|h| h.free_will) {
        return;
    }
    for id in curtains {
        set(world, id, shut);
    }
}
