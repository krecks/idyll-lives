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
//! A new game's lot (its file's lamps all on) is first seen as if everyone had just come in: the
//! lamps of the rooms with nobody awake in them go off, those of a dark room with someone awake
//! in it on (`Watch::fresh`). A save opens with its lamps as they were.
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
    /// A new game's lot, not looked at yet: its rooms' lamps are switched for who's in them.
    pub(crate) fresh: bool,
    /// The lights that follow a use (`WhileLights`), found for the objects as they stood
    /// (`World::objects_version`, which changes whenever objects come, go or move).
    while_lights: Option<(u32, WhileLights)>,
    /// The lamps by room, found for the objects and the rooms as they stood (`objects_version`
    /// and `lot_version`).
    lamps: Option<((u32, u32), Lamps)>,
}

/// What `cached` holds if it was found for `key` (the versions it's kept for, `Watch`).
fn kept<K: PartialEq, T>(cached: Option<(K, T)>, key: K) -> Option<T> {
    cached.filter(|c| c.0 == key).map(|c| c.1)
}

/// The lights that follow a use (content `light.while`; object ids, in id order) and the
/// animations any of them watches (sorted): a big town has thousands of objects and a handful
/// of these, looked at every tick.
#[derive(Debug, Clone)]
struct WhileLights {
    lights: Vec<u32>,
    watched: Vec<usize>,
}

impl WhileLights {
    fn new(world: &World) -> Self {
        let content = &world.content;
        let lights: Vec<u32> = world
            .objects
            .iter()
            .filter(|o| !content.objects[o.def].while_anims.is_empty())
            .map(|o| o.id)
            .collect();
        // (A handful: most lights are of one kind.)
        let mut watched: Vec<usize> = lights
            .iter()
            .flat_map(|&id| {
                content.objects[world.objects[id as usize].def]
                    .while_anims
                    .iter()
                    .copied()
            })
            .collect();
        watched.sort_unstable();
        watched.dedup();
        Self { lights, watched }
    }
}

/// Whether it's dark at `minute` of the day.
pub fn dark_at(minute: f32) -> bool {
    !(DARK_UNTIL..DARK_FROM).contains(&minute)
}

/// Whether object kind `def` is a lamp that can be switched (a light that follows a use has no
/// switch: content loading leaves its `switch` off).
pub fn switchable(world: &World, def: usize) -> bool {
    world.content.objects[def].switch
}

/// The lights that follow a use (content `light.while`) lit now (stable ids): someone is doing one
/// of their animations at an object on their tiles, or standing on them (a cooker hood's light
/// while the hob under it cooks). Any task counts, not only a plain use: cooking a recipe is
/// `TaskKind::Cook`, its step's animation the one the hood watches.
fn while_lit(world: &World, cached: &WhileLights) -> Vec<u32> {
    let content = &world.content;
    let watched = &cached.watched;
    if cached.lights.is_empty() {
        return Vec::new();
    }
    // Each sim doing one of them (once a tick, not once a light): the animation, the footprint of
    // the object they're at (none when at no object: `object` is -1), and the tile they stand on.
    let doing: Vec<_> = world
        .sims
        .iter()
        .filter_map(|s| {
            let d = crate::snapshot::doing(world, s);
            (d.action >= 0.0 && watched.binary_search(&(d.action as usize)).is_ok()).then(|| {
                let at = (d.object >= 0.0)
                    .then(|| world.index_of(d.object as u32))
                    .flatten()
                    .map(|i| footprint(&world.objects[i], content));
                (
                    d.action as usize,
                    at,
                    (s.pos[0].floor() as i32, s.pos[1].floor() as i32),
                )
            })
        })
        .collect();
    if doing.is_empty() {
        return Vec::new();
    }
    cached
        .lights
        .iter()
        .map(|&id| &world.objects[id as usize])
        .filter(|o| {
            let anims = &content.objects[o.def].while_anims;
            let [x0, z0, x1, z1] = footprint(o, content);
            let on = |(x, z): (i32, i32)| x >= x0 && x < x1 && z >= z0 && z < z1;
            doing.iter().any(|&(anim, at, here)| {
                anims.contains(&anim)
                    && (on(here)
                        || at.is_some_and(|[a0, b0, a1, b1]| {
                            a0 < x1 && x0 < a1 && b0 < z1 && z0 < b1
                        }))
            })
        })
        .map(|o| o.stable)
        .collect()
}

/// The tiles `o` stands on as `[x0, z0, x1, z1]` (x1, z1 past the last).
fn footprint(o: &crate::world::ObjectInstance, content: &crate::content::Content) -> [i32; 4] {
    let (w, d) = o.size(content);
    [o.x, o.z, o.x + w, o.z + d]
}

/// Keeps `World::while_lit` (the lights that follow a use) up to date, telling the renderer when
/// one comes on or goes off.
pub(crate) fn update_while(world: &mut World) {
    let version = world.objects_version;
    let cached = kept(world.light_watch.while_lights.take(), version)
        .unwrap_or_else(|| WhileLights::new(world));
    let lit = while_lit(world, &cached);
    world.light_watch.while_lights = Some((version, cached));
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
    // (The lamps by room, found once for all the rooms looked at, and kept while the objects and
    // the rooms stand as they did.)
    let key = (world.objects_version, lot_version);
    let mut lamps = kept(watch.lamps.take(), key);
    if watch.fresh && world.autonomy {
        let rooms: Vec<u16> = world
            .rooms()
            .iter()
            .filter(|r| !r.garden && r.id != OUTDOORS)
            .map(|r| r.id)
            .collect();
        let lamps = lamps.get_or_insert_with(|| Lamps::new(world));
        for room in rooms {
            switch_room(world, lamps, room, false);
            if awake.binary_search(&room).is_ok() && room_dark(world, lamps, room, dark) {
                switch_room(world, lamps, room, true);
            }
        }
        watch.fresh = false;
    }
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
        if !came.is_empty() || !left.is_empty() {
            let lamps = lamps.get_or_insert_with(|| Lamps::new(world));
            for room in came {
                if room_dark(world, lamps, room, dark) {
                    switch_room(world, lamps, room, true);
                }
            }
            for room in left {
                switch_room(world, lamps, room, false);
            }
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
    watch.lamps = lamps.map(|lamps| (key, lamps));
    watch.awake = awake;
    watch.dark = dark;
    watch.lot_version = lot_version;
    watch.primed = true;
    world.light_watch = watch;
}

/// Whether `room` is dark: it is night (or the room is in the basement, or has no windows) and
/// none of its lamps is on (a fire burning there isn't lit for its light).
fn room_dark(world: &World, lamps: &Lamps, room: u16, night: bool) -> bool {
    let mut lamps = lamps
        .in_room(room)
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

/// The switchable lamps by the room they stand in (`room_at` their middle), each room's in id
/// order: found once for a pass over the rooms rather than every lamp for every room.
#[derive(Debug, Clone)]
struct Lamps {
    by_room: Vec<(u16, u32)>,
}

impl Lamps {
    fn new(world: &World) -> Self {
        let mut by_room: Vec<(u16, u32)> = world
            .objects
            .iter()
            .filter(|o| switchable(world, o.def))
            .map(|o| {
                let [x, z] = o.centre(&world.content);
                (world.lot.room_at(x.floor() as i32, z.floor() as i32), o.id)
            })
            .collect();
        by_room.sort_unstable();
        Self { by_room }
    }

    /// The switchable lamps standing in `room`, in id order.
    fn in_room(&self, room: u16) -> impl Iterator<Item = u32> + '_ {
        let from = self.by_room.partition_point(|&(r, _)| r < room);
        self.by_room[from..]
            .iter()
            .take_while(move |&&(r, _)| r == room)
            .map(|&(_, id)| id)
    }
}

/// A resident switches every lamp in `room`, unless the household living there has free will off
/// (and puts out a fire there, but doesn't light one).
fn switch_room(world: &mut World, lamps: &Lamps, room: u16, on: bool) {
    let lamps: Vec<u32> = lamps
        .in_room(room)
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
