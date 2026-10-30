//! Lamps switched on and off (`ObjectInstance::off`).
//!
//! Each lamp (an object with a content `light` that doesn't say `"switch": false`) is on or
//! off on its own: the player turns one on or off with a click (`Command::SwitchLamp`). One
//! bought comes switched off. Residents switch a room's lamps together, as at the switch by the
//! door, on two moments only (so what the player set stays until someone comes or goes):
//! - someone awake comes into a room when it's dark (night, or always in the basement and in a
//!   room without windows) and none of its lamps is on: they all go on; the same as night falls
//!   on a room with someone awake in it;
//! - the last one awake leaves a room, or goes to sleep in it: its lamps go off.
//!
//! A fire (content `light.fire`: a fireplace, a fire pit) is switched as a lamp is, but nobody
//! lights it just for coming in: whoever sits down by it in the dark lights it (`world::arrive`),
//! or the player does. It goes out with the room's lamps when the last one awake leaves, and a
//! fire still burning has burnt down when the morning comes.
//!
//! Only at home where the household has free will (and in public places): with it off, the
//! lights stay as the player left them (fires still burn down). Outdoors (the garden's lanterns)
//! is the player's. The renderer shows a lamp that's off with no light and no glow (a fire with
//! no flames either); by day only the basement's and windowless rooms' lamps show.
//! Room scores count the lamps a room has, on or off (they're how well it can be lit).
//! A lamp moved (alone, on a top or with its room), undone or redone stays on or off as it was.

use crate::World;
use crate::lot::OUTDOORS;

/// Minutes of the day from which it is dark indoors (lamps come up in the renderer's look then).
pub const DARK_FROM: f32 = 19.0 * 60.0;
/// Minutes of the day until which it is still dark in the morning.
pub const DARK_UNTIL: f32 = 6.0 * 60.0 + 30.0;
/// How often (ticks) residents' comings and goings are looked at: a quarter of a game minute.
const EVERY: u64 = 5;

/// What the residents' rule saw last time (not saved: on load it starts from what's there,
/// switching nothing).
#[derive(Debug, Default, Clone)]
pub(crate) struct Watch {
    /// Rooms with someone awake in them, sorted.
    awake: Vec<u16>,
    dark: bool,
    /// The lot (walls) these room ids were for; after a rebuild the rooms are seen afresh.
    lot_version: u32,
    primed: bool,
}

/// Whether it's dark at `minute` of the day.
pub fn dark_at(minute: f32) -> bool {
    !(DARK_UNTIL..DARK_FROM).contains(&minute)
}

/// Whether object kind `def` is a lamp that can be switched.
pub fn switchable(world: &World, def: usize) -> bool {
    let def = &world.content.objects[def];
    def.switch && def.while_anims.is_empty()
}

/// Whether a light that follows a use (content `light.while`) is lit now: someone is doing that
/// animation at an object on its tiles (a cooker hood's light while the hob under it cooks).
/// Any task counts, not only a plain use: cooking a recipe is `TaskKind::Cook`, its step's
/// animation the one the hood watches.
pub fn while_on(world: &World, o: &crate::world::ObjectInstance) -> bool {
    let tags = &world.content.objects[o.def].while_anims;
    if tags.is_empty() {
        return false;
    }
    let anims: Vec<usize> = world
        .content
        .animations
        .iter()
        .enumerate()
        .filter(|(_, a)| tags.contains(a))
        .map(|(i, _)| i)
        .collect();
    let tiles: Vec<(i32, i32)> = o.tiles(&world.content).collect();
    let on_tiles = |x: i32, z: i32| tiles.contains(&(x, z));
    world.sims.iter().any(|s| {
        let doing = crate::snapshot::doing(world, s);
        if doing.action < 0.0 || !anims.contains(&(doing.action as usize)) {
            return false;
        }
        // At a station on the tiles, or standing on them (a hob with no station to speak of).
        let at = world.index_of(doing.object as u32).map(|i| {
            world.objects[i]
                .tiles(&world.content)
                .any(|t| on_tiles(t.0, t.1))
        });
        at == Some(true) || on_tiles(s.pos[0].floor() as i32, s.pos[1].floor() as i32)
    })
}

/// Keeps `World::while_lit` (the lights that follow a use) up to date, telling the renderer when
/// one comes on or goes off.
pub(crate) fn update_while(world: &mut World) {
    if !world
        .content
        .objects
        .iter()
        .any(|o| !o.while_anims.is_empty())
    {
        return;
    }
    let lit: Vec<u32> = world
        .objects
        .iter()
        .filter(|o| while_on(world, o))
        .map(|o| o.stable)
        .collect();
    if lit != world.while_lit {
        world.while_lit = lit;
        world.lights_version = world.lights_version.wrapping_add(1);
    }
}

/// Switches one lamp; false if it was so already (or isn't a lamp).
pub(crate) fn set(world: &mut World, object: u32, on: bool) -> bool {
    let Some(o) = world.objects.get(object as usize) else {
        return false;
    };
    if !switchable(world, o.def) || o.off != on {
        return false;
    }
    world.objects[object as usize].off = !on;
    world.lights_version = world.lights_version.wrapping_add(1);
    true
}

/// The lamps that are off (object ids), for the renderer: those switched off, and the lights that
/// follow a use while nothing is being done under them.
pub fn lamps_off(world: &World) -> Vec<u32> {
    world
        .objects
        .iter()
        .filter(|o| {
            (o.off && switchable(world, o.def))
                || (!world.content.objects[o.def].while_anims.is_empty()
                    && !world.while_lit.contains(&o.stable))
        })
        .map(|o| o.stable)
        .collect()
}

/// The residents' rule (see the module notes); runs every `EVERY` ticks.
pub(crate) fn update(world: &mut World) {
    if !world.tick.is_multiple_of(EVERY) {
        return;
    }
    let content = &world.content;
    let mut awake: Vec<u16> = world
        .sims
        .iter()
        .filter(|s| {
            s.here() && s.away_until.is_none() && !content.life.baby(s.age) && !s.asleep(content)
        })
        .map(|s| {
            let (x, z) = s.tile();
            world.lot.room_at(x, z)
        })
        .filter(|&r| r != OUTDOORS)
        .collect();
    awake.sort_unstable();
    awake.dedup();
    let dark = dark_at(crate::clock::minute_of_day(world.tick));
    let lot_version = world.lot_version();
    let mut watch = std::mem::take(&mut world.light_watch);
    if watch.primed && watch.lot_version == lot_version && world.autonomy {
        let came: Vec<u16> = awake
            .iter()
            .copied()
            .filter(|r| !watch.dark && dark || watch.awake.binary_search(r).is_err())
            .collect();
        let left: Vec<u16> = watch
            .awake
            .iter()
            .copied()
            .filter(|r| awake.binary_search(r).is_err())
            .collect();
        for room in came {
            if room_dark(world, room, dark) {
                switch_room(world, room, true);
            }
        }
        for room in left {
            switch_room(world, room, false);
        }
    }
    // Morning: the fires left burning have burnt down.
    if watch.primed && watch.dark && !dark {
        let fires: Vec<u32> = world
            .objects
            .iter()
            .filter(|o| !o.off && world.content.objects[o.def].fire)
            .map(|o| o.id)
            .collect();
        for id in fires {
            set(world, id, false);
        }
    }
    watch.awake = awake;
    watch.dark = dark;
    watch.lot_version = lot_version;
    watch.primed = true;
    world.light_watch = watch;
}

/// Whether `room` is dark: it is night (or the room is in the basement, or has no windows) and
/// none of its lamps is on (a fire burning there isn't lit for its light).
fn room_dark(world: &World, room: u16, night: bool) -> bool {
    let mut lamps = lamps_in(world, room)
        .filter(|&id| !world.content.objects[world.objects[id as usize].def].fire)
        .peekable();
    let Some(&first) = lamps.peek() else {
        return false;
    };
    let o = &world.objects[first as usize];
    let below = world.stairs.storey(o.z) < 0;
    (night || below || windowless(world, room)) && lamps.all(|id| world.objects[id as usize].off)
}

/// Whether `room` has no window (to outdoors or to another room): daylight doesn't reach it.
pub fn windowless(world: &World, room: u16) -> bool {
    world
        .rooms()
        .iter()
        .find(|r| !r.garden && r.id == room)
        .is_some_and(|r| r.windows == 0)
}

/// The switchable lamps standing in `room`.
fn lamps_in(world: &World, room: u16) -> impl Iterator<Item = u32> + '_ {
    world
        .objects
        .iter()
        .filter(move |o| {
            switchable(world, o.def) && {
                let [x, z] = o.centre(&world.content);
                world.lot.room_at(x.floor() as i32, z.floor() as i32) == room
            }
        })
        .map(|o| o.id)
}

/// A resident switches every lamp in `room`, unless the household living there has free will off
/// (and puts out a fire there, but doesn't light one).
fn switch_room(world: &mut World, room: u16, on: bool) {
    let lamps: Vec<u32> = lamps_in(world, room)
        .filter(|&id| !on || !world.content.objects[world.objects[id as usize].def].fire)
        .collect();
    let Some(&first) = lamps.first() else { return };
    let o = &world.objects[first as usize];
    let plot = world.plot_at(o.x, o.z);
    if world
        .households
        .iter()
        .any(|h| h.plot.is_some() && h.plot == plot && !h.free_will)
    {
        return;
    }
    for id in lamps {
        set(world, id, on);
    }
}
