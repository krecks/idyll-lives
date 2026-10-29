//! Privacy (G-72): rooms kept shut while someone is in them, the household's own things, and
//! trying for a baby in bed behind a shut door.
//!
//! **Rooms kept shut.** A room of a private kind (content `roomKinds[].private`: the bathroom)
//! is kept shut while someone is in it, and so is the room of a use done behind a shut door
//! (`Interaction::private`: a double bed's *Try for a baby*) while the two are in it at it. Not
//! a room that is two kinds at once (`mixed`: a toilet beside the bed with no wall between),
//! nor one with a door to the outside (a cabin with its toilet in the open: its door is the
//! front door), nor anything on a public place (`private_rooms`, found with the rooms). `refresh`
//! finds who keeps which shut as each tick starts.
//!
//! A room of a *claimed* kind (content `roomKinds[].claimed`: a bedroom, and `private` as well)
//! is its people's own: whoever has a place in it comes and goes as they please, and its door
//! keeps everyone else out. Who has a place is what's in the room, not a name on a door: whoever
//! lies in one of its beds now (not just on their way to one), and only those who belong with them
//! (partner, parents, children, brothers and sisters: only they share a bedroom) with somewhere
//! here to lie: a bed nobody's in, a bunk's other bunk, or the other half of a couple's bed if
//! they're the partner (back from a drink of water) or a child of whoever lies in it, for the
//! night. Whoever minds the crib in there (the baby in it and its care: feeding, changing,
//! cuddling) goes in too, a lodger included. While nobody lies here yet, it's whoever has a place
//! here. The bathroom is never claimed: nobody has a place in it, so it shuts for whomever is in
//! there (`Shut::claimants`, `refresh`). At a use behind a shut door (the two *trying for a baby*)
//! the door keeps the household out all the same, claimants included. A bedroom on the way to a
//! room beyond it (an en-suite, a study through it) is walked through: it keeps out only those
//! going in.
//!
//! Someone on their way into a room kept shut by someone else stops short of its doorway
//! (`LOOK` metres before they'd step in) and waits there (`Sim::kept_out`), standing; once
//! nobody keeps it shut they go on. Whoever does the same private use as the one in there (the
//! other one trying for a baby) goes in, and so does anyone the room is *their own* (a claimed
//! room's claimants, above). After `WAIT_MINUTES` they give up; one who chose it
//! themselves doesn't choose it again for a while (`autonomy.retryMinutes`). Free will doesn't
//! choose something in a room kept shut by someone else, unless it fills a need about to run
//! out: they'd rather wait at the door. Kept out, a need that runs out counts as having nothing
//! for it at home (the accident comes after its `graceMinutes`), and a toilet in a room kept
//! shut counts as taken (a second bathroom is wished for, `rooms::opinions`). The renderer holds
//! such a room's doors shut (snapshot `sim::SHUTS`). Nobody waits on their way out to work, home,
//! a visit or the front door, nor on their way to someone to talk to or to meet someone who
//! called them: they go through.
//!
//! **The household's own things.** An object kind marked `private` (a wardrobe, a dresser, the
//! bath cabinet) is used by nobody visiting: free will doesn't choose it and an order is refused
//! (`world::guest_may`), as with beds and showers.
//!
//! **Trying for a baby.** When partners' social with the `conceive` effect goes well and the
//! content has somewhere to try (`Interaction::conceive`, the double beds), the two go to bed
//! (`after_step`): to a free double bed at home in a room of its own (indoors, with a door, nobody
//! else in it but a baby), where they lie under the covers for its time, the door shut (with
//! someone else in the room as they get there, they don't: `overlooked`). A baby
//! may come of it once it has run its time (`lifecycle::conceive`, as before). Without such a bed
//! the social isn't chosen, nor can the player ask for it.

use crate::content::{Content, Interaction};
use crate::lot::{Lot, OUTDOORS};
use crate::world::{Ctx, ObjectInstance, Phase, Sim, Task, TaskKind, World};

/// Metres ahead along their way someone stops before a room kept shut: short of its doorway.
pub const LOOK: f32 = 0.9;

/// Game minutes someone waits outside a room kept shut before giving up.
pub const WAIT_MINUTES: f32 = 60.0;

/// A room kept shut (a lot room id) and the resident in it who keeps it so.
#[derive(Debug, Clone, PartialEq)]
pub struct Shut {
    pub room: u16,
    pub by: u32,
    /// The private use they're at in it (an object and its interaction: a double bed's *Try for
    /// a baby*): whoever does the same one with them goes in.
    pub doing: Option<(u32, usize)>,
    /// A claimed room (`roomKinds[].claimed`: a bedroom) is its people's own: the residents who
    /// have a place in it (`refresh`: whose bed or crib it offers, and whoever lies in one now)
    /// come in even while someone else is in the room. Empty for a private room that isn't claimed
    /// (the bathroom), and while someone is at a use behind the door (`doing`).
    pub claimants: Vec<u32>,
    /// Of a claimed kind: a bedroom keeps out only those going in, not those passing through it to
    /// a room beyond (`shut_ahead`), unless someone's at a use behind its door.
    pub claimed: bool,
}

/// Whether a room is kept shut while anyone is in it: of a private kind, not mixed with another
/// (a toilet by the bed with no wall between), with no door to the outside (a cabin with its
/// toilet in the open: the front door isn't a bathroom door), not on a public place.
fn private_room(content: &Content, plots: &[crate::world::Plot], r: &crate::rooms::RoomInfo, outside: &[u16]) -> bool {
    !r.garden
        && r.id != OUTDOORS
        && !r.mixed
        && r.kind.is_some_and(|k| content.room_kinds[k].private)
        && !outside.contains(&r.id)
        && r.plot.is_none_or(|p| plots.get(p as usize).is_none_or(|p| !p.public))
}

/// The rooms of `rooms` kept shut while anyone is in them (`private_room`), sorted, and the claimed
/// (`roomKinds[].claimed`: bedrooms, `Shut::claimants`) ones among them, sorted.
pub(crate) fn private_rooms(content: &Content, plots: &[crate::world::Plot], rooms: &[crate::rooms::RoomInfo], lot: &Lot) -> (Vec<u16>, Vec<u16>) {
    if !content.room_kinds.iter().any(|k| k.private) {
        return (Vec::new(), Vec::new());
    }
    // Rooms with a door to the outside.
    let mut outside = Vec::new();
    let room = |x: i32, z: i32| lot.room_at(x, z);
    let (w, d) = (lot.width as i32, lot.depth as i32);
    let mut door = |a: u16, b: u16| {
        for (r, across) in [(a, b), (b, a)] {
            if r != OUTDOORS && across == OUTDOORS && !outside.contains(&r) {
                outside.push(r);
            }
        }
    };
    for z in 0..=d {
        for x in 0..w {
            if lot.h_edge(x as usize, z as usize) == crate::lot::Edge::Door {
                door(room(x, z - 1), room(x, z));
            }
        }
    }
    for z in 0..d {
        for x in 0..=w {
            if lot.v_edge(x as usize, z as usize) == crate::lot::Edge::Door {
                door(room(x - 1, z), room(x, z));
            }
        }
    }
    let mut ids = Vec::new();
    let mut claimed = Vec::new();
    for r in rooms.iter().filter(|r| private_room(content, plots, r, &outside)) {
        ids.push(r.id);
        if r.kind.is_some_and(|k| content.room_kinds[k].claimed) {
            claimed.push(r.id);
        }
    }
    ids.sort_unstable();
    claimed.sort_unstable();
    (ids, claimed)
}

/// The private use `s` is at or on their way to (`Interaction::private`), if any.
pub(crate) fn private_use(content: &Content, objects: &[ObjectInstance], s: &Sim) -> Option<(u32, usize)> {
    match s.current()?.task.kind {
        TaskKind::Use { object, interaction } => objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .filter(|i| i.private)
            .map(|_| (object, interaction)),
        _ => None,
    }
}

/// The room a point is in (`OUTDOORS`: none).
fn room_of(lot: &Lot, p: [f32; 2]) -> u16 {
    lot.room_at(p[0].floor() as i32, p[1].floor() as i32)
}

/// The room an object stands in (by the middle of its footprint).
pub(crate) fn room_of_object(content: &Content, lot: &Lot, o: &ObjectInstance) -> u16 {
    room_of(lot, o.centre(content))
}

/// Whether `inter` offers a place in a claimed room: somewhere to lie (sleeping or napping,
/// `dayRhythm`), or a baby's place and the grown-up care of it.
fn offers_place(content: &Content, inter: &Interaction) -> bool {
    inter.baby || inter.cares() || inter.tags & (content.day_rhythm.sleep_tags | content.day_rhythm.nap_tags) != 0
}

/// Whether `s` lives on the plot `plot` (their own home: not a guest, and not someone from another
/// house). A lot with no plots is one home throughout (`access_in`): everyone staying there lives
/// there, guests aside.
fn lives_on(w: &World, s: &Sim, plot: Option<u32>) -> bool {
    if s.visiting.is_some() {
        return false;
    }
    match plot {
        None => true,
        Some(p) => w.households.get(s.household as usize).is_some_and(|h| h.plot == Some(p)),
    }
}

/// Whether `sim` of this world can go to the interaction `i` of `o`: a resident of the home
/// (`plot`), with a slot of that use free, their age and standing enough for the use itself, and
/// the place open to them.
fn goes_to(w: &World, o: &ObjectInstance, sim: u32, plot: Option<u32>, i: usize) -> bool {
    let s = match w.sims.get(sim as usize) {
        Some(s) if s.here() && s.carried_by.is_none() => s,
        _ => return false,
    };
    let Some(def) = w.content.objects.get(o.def) else { return false };
    let Some(inter) = def.interactions.get(i) else { return false };
    lives_on(w, s, plot)
        && o.can_use_for(sim, def, i)
        && o.allows(inter, sim)
        && crate::world::unfit(&w.content, s.age, inter).is_none()
}

/// Whether `b` is a child of `a` (a little one sleeps in their parents' room, not a box of their own).
fn is_child_of(w: &World, a: u32, b: u32) -> bool {
    w.relationships.kin(a as usize, b as usize) == crate::social::Kin::Child
}

/// Whether `a` and `b` belong together: the same one, partners, or family (parent, child,
/// sibling). Only they share a bedroom.
fn connected(w: &World, a: u32, b: u32) -> bool {
    a == b
        || w.relationships.partner_of(a as usize) == Some(b as usize)
        || !w.relationships.kin(a as usize, b as usize).is_none()
}

/// A couple's bed (one with somewhere to try for a baby: a double bed): its other half is for the
/// partner of whoever lies in it, or a child of theirs for the night. A bed for two apart (a bunk,
/// a futon) is a place for each.
pub(crate) fn couples_bed(def: &crate::content::ObjectDef) -> bool {
    def.slots > 1 && def.interactions.iter().any(|i| i.conceive)
}

/// The objects that can offer a place in a claimed room (`roomKinds[].claimed`), per claimed
/// room (`World::claimed_places`): the beds and sofas to lie in, and the cribs and their care.
/// Broken ones are listed too (skipped while broken, `room_claimants`), so a repair needn't wait
/// for the rooms to be found again. Found with the rooms, so the tick needn't look through every
/// object in the home.
pub(crate) fn claimed_places(content: &Content, lot: &Lot, claimed: &[u16], objects: &[ObjectInstance]) -> Vec<(u16, Vec<u32>)> {
    let mut out: Vec<(u16, Vec<u32>)> = claimed.iter().map(|&r| (r, Vec::new())).collect();
    for (i, o) in objects.iter().enumerate() {
        // Broken ones are listed too: `room_claimants` skips them while they're broken, and a
        // repair shouldn't need the rooms rebuilt.
        let room = room_of_object(content, lot, o);
        let Ok(k) = claimed.binary_search(&room) else {
            continue;
        };
        let Some(def) = content.objects.get(o.def) else {
            continue;
        };
        if def.interactions.iter().any(|i| offers_place(content, i)) {
            out[k].1.push(i as u32);
        }
    }
    out
}

/// The people who have a place in claimed room `room`, over the places in it
/// (`World::claimed_places`):
///
/// - whoever of this home lies in one of them now (holding it and in the room: not just on their
///   way to it, or anyone sent to a bed would make the room theirs);
/// - while someone lies here, those who belong with one of them (`connected`: partner or family)
///   and have a place here they may take: a bed nobody lies in, a bunk's other bunk, or the other
///   half of a couple's bed if they're the partner (back from a drink of water) or a child of whoever
///   lies in it;
/// - whoever can mind a crib here, whether or not the baby is in it (any adult of this home, a
///   lodger included); and
/// - while nobody lies here yet (nobody has gone to bed, or the crib's empty), whoever of this
///   home has a place here.
///
/// Guests, and anyone from another house, are nobody's own. As each tick starts, before anyone's
/// free will has chosen anything.
fn room_claimants(w: &World, room: u16) -> Vec<u32> {
    let mut out = Vec::new();
    let Ok(k) = w.claimed_places.binary_search_by_key(&room, |(r, _)| *r) else {
        return out;
    };
    let places: Vec<(&ObjectInstance, &crate::content::ObjectDef, Option<u32>)> = w.claimed_places[k]
        .1
        .iter()
        .filter_map(|&oi| w.objects.get(oi as usize))
        .filter(|o| !o.broken())
        .filter_map(|o| Some((o, w.content.objects.get(o.def)?, w.plot_at(o.x, o.z))))
        .collect();
    let lying_in = |o: &ObjectInstance, plot: Option<u32>| -> Vec<u32> {
        o.users()
            .filter(|&u| w.sims.get(u as usize).is_some_and(|s| lives_on(w, s, plot) && room_of(&w.lot, s.pos) == room))
            .collect()
    };
    let lying: Vec<u32> = places.iter().flat_map(|&(o, _, plot)| lying_in(o, plot)).collect();
    out.extend_from_slice(&lying);
    for &(o, def, plot) in &places {
        let here = lying_in(o, plot);
        for (i, inter) in def.interactions.iter().enumerate() {
            if !offers_place(&w.content, inter) {
                continue;
            }
            for s in w.sims.iter().filter(|s| s.here() && s.carried_by.is_none()) {
                if !goes_to(w, o, s.id, plot, i) {
                    continue;
                }
                let own = inter.cares()
                    || lying.is_empty()
                    || (lying.iter().any(|&l| connected(w, l, s.id))
                        && (here.is_empty()
                            || !couples_bed(def)
                            || here.iter().any(|&l| w.relationships.partner_of(l as usize) == Some(s.id as usize) || is_child_of(w, l, s.id))));
                if own {
                    out.push(s.id);
                }
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

fn mix(h: &mut u64, v: u64) {
    *h ^= v;
    *h = h.wrapping_mul(0x100_0000_01b3);
}

/// Everything the people of a claimed room can depend on, apart from the room's own beds: who is
/// here and where they live, how old they are, and the family links and partnerships
/// (`room_claimants`).
fn presence_hash(w: &World) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325;
    mix(&mut h, w.relationships.version());
    for s in &w.sims {
        mix(&mut h, s.id as u64);
        mix(&mut h, s.age.to_bits() as u64);
        mix(&mut h, s.gone.is_none() as u64);
        mix(&mut h, s.visiting.is_some() as u64);
        mix(&mut h, s.carried_by.map_or(0, |c| c as u64 + 1));
        mix(&mut h, s.household as u64);
        let plot = w.households.get(s.household as usize).and_then(|h| h.plot).map_or(u64::MAX, |p| p as u64);
        mix(&mut h, plot);
    }
    h
}

/// A claimed room's place objects, their state and who lies in them, over `presence_hash`: the
/// same number means the same people have a place there (`World::claimants_cache`).
fn room_hash(w: &World, room: u16, presence: u64) -> u64 {
    let mut h = presence;
    mix(&mut h, room as u64);
    let Ok(k) = w.claimed_places.binary_search_by_key(&room, |(r, _)| *r) else {
        return h;
    };
    for &oi in &w.claimed_places[k].1 {
        let Some(o) = w.objects.get(oi as usize) else {
            continue;
        };
        mix(&mut h, oi as u64);
        mix(&mut h, o.broken() as u64);
        mix(&mut h, o.open as u64);
        mix(&mut h, o.ripe() as u64);
        mix(&mut h, o.run.map_or(0, |m| if m > 0.0 { 1 } else { 2 }));
        mix(&mut h, o.closed_users);
        let mut n = 0;
        for u in o.users() {
            mix(&mut h, u as u64);
            // In the room, or still on their way to it (`room_claimants`).
            mix(&mut h, w.sims.get(u as usize).is_some_and(|s| room_of(&w.lot, s.pos) == room) as u64);
            n += 1;
        }
        mix(&mut h, n);
    }
    h
}

/// Finds the rooms kept shut as a tick starts (`World::shut`).
pub(crate) fn refresh(w: &mut World) {
    let mut shut = std::mem::take(&mut w.shut);
    // The people who have a place in each claimed room (`room_claimants`), kept from tick to
    // tick while nothing they depend on changes (`presence_hash`, `room_hash`).
    let mut cache = std::mem::take(&mut w.claimants_cache);
    let presence = presence_hash(w);
    shut.clear();
    for s in &w.sims {
        if !s.here() || s.away_until.is_some() || s.carried_by.is_some() || w.content.life.baby(s.age) {
            continue;
        }
        let room = room_of(&w.lot, s.pos);
        if room == OUTDOORS {
            continue;
        }
        let doing = private_use(&w.content, &w.objects, s).filter(|&(o, _)| room_of_object(&w.content, &w.lot, &w.objects[o as usize]) == room);
        if doing.is_some() || w.private_rooms.binary_search(&room).is_ok() {
            // Whoever is at a use behind a shut door (two trying for a baby) keeps the household
            // out, claimants included: only while the room is shut by itself being used do its
            // own people have the run of it.
            let claimed = w.claimed_rooms.binary_search(&room).is_ok();
            let claimants = if doing.is_none() && claimed {
                let hash = room_hash(w, room, presence);
                match cache.iter_mut().find(|(r, _, _)| *r == room) {
                    Some((_, cached, people)) if *cached == hash => people.clone(),
                    Some((_, cached, people)) => {
                        *cached = hash;
                        *people = room_claimants(w, room);
                        people.clone()
                    }
                    None => {
                        let people = room_claimants(w, room);
                        cache.push((room, hash, people.clone()));
                        people
                    }
                }
            } else {
                Vec::new()
            };
            shut.push(Shut { room, by: s.id, doing, claimants, claimed });
        }
    }
    w.shut = shut;
    w.claimants_cache = cache;
}

/// Whether `room` is kept shut to resident `me` (doing private use `mine`, if any): by someone
/// else in it, not at the same use, and not one of the room's own people (`claimants`) if it's a
/// claimed room.
pub(crate) fn shut_to(shut: &[Shut], me: u32, mine: Option<(u32, usize)>, room: u16) -> bool {
    room != OUTDOORS
        && shut.iter().any(|s| {
            s.room == room && s.by != me && (s.doing.is_none() || s.doing != mine) && !s.claimants.contains(&me)
        })
}

/// Whether `room`, kept shut, may be walked through to a room beyond: a bedroom (claimed), unless
/// someone in it is at a use behind its door (trying for a baby).
fn passable(shut: &[Shut], room: u16) -> bool {
    shut.iter().filter(|s| s.room == room).all(|s| s.claimed && s.doing.is_none())
}

/// Whether `sim` keeps a room shut now.
pub fn keeps_shut(world: &World, sim: u32) -> bool {
    world.shut.iter().any(|s| s.by == sim)
}

/// Whether the thing at tile `at` (where it's used from) is in a room kept shut to `sim` by
/// someone else, and they aren't in that room already.
pub(crate) fn shut_away(ctx: &Ctx, sim: &Sim, at: (i32, i32)) -> bool {
    if ctx.shut.is_empty() {
        return false;
    }
    let lot = ctx.nav.lot;
    let room = lot.room_at(at.0, at.1);
    room != room_of(lot, sim.pos) && shut_to(ctx.shut, sim.id, None, room)
}

/// Whether `sim`, walking, is about to step into a room kept shut to them: within `LOOK` metres
/// along their way, into another room than the one they're in. Not on the way to someone to
/// talk to or to meet someone who called them (`conversation.rs` walks them), nor on their way out to work, home, a visit or the
/// door (a house whose only way out is through the bathroom: they go through rather than miss
/// the shift).
pub(crate) fn shut_ahead(sim: &Sim, ctx: &Ctx, objects: &[ObjectInstance]) -> bool {
    if ctx.shut.is_empty() {
        return false;
    }
    let Some(act) = sim.current() else { return false };
    let Phase::Routing { waypoints, next } = &act.phase else { return false };
    // (Nor someone called over, on their way to meet the one who called: `Sim::heeding`.)
    if sim.heeding.is_some() || matches!(act.task.kind, TaskKind::Social { .. } | TaskKind::Join { .. } | TaskKind::Work | TaskKind::GoHome | TaskKind::Visit { .. } | TaskKind::Answer { .. }) {
        return false;
    }
    let lot = ctx.nav.lot;
    let stairs = &ctx.nav.stairs;
    let here = room_of(lot, sim.pos);
    let storey = stairs.storey(sim.pos[1].floor() as i32);
    let mine = private_use(ctx.content, objects, sim);
    // Where they're going: a bedroom on the way to a room beyond it is walked through.
    let to = waypoints.last().map_or(here, |&wp| room_of(lot, wp));
    let (mut at, mut left) = (sim.pos, LOOK);
    for &wp in waypoints.iter().skip(*next) {
        if stairs.storey(wp[1].floor() as i32) != storey {
            break;
        }
        let d = (wp[0] - at[0]).hypot(wp[1] - at[1]);
        let along = d.min(left);
        // (Every quarter metre: no doorway is passed between two looks.)
        let steps = (along / 0.25).ceil().max(1.0) as u32;
        for k in 1..=steps {
            let t = if d > 1e-4 { (along * k as f32 / steps as f32) / d } else { 1.0 };
            let p = [at[0] + (wp[0] - at[0]) * t, at[1] + (wp[1] - at[1]) * t];
            let room = room_of(lot, p);
            if room != here && shut_to(ctx.shut, sim.id, mine, room) && (room == to || !passable(ctx.shut, room)) {
                return true;
            }
        }
        if d >= left {
            break;
        }
        left -= d;
        at = wp;
    }
    false
}

/// A walker's step outside a room kept shut (`world::progress`, before they move): true while
/// they wait there (`ticks` more), false to walk on. After `WAIT_MINUTES` they give up what
/// they were on their way to.
pub(crate) fn wait_outside(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance], ticks: u32) -> bool {
    if !shut_ahead(sim, ctx, objects) {
        sim.kept_out = 0;
        return false;
    }
    sim.kept_out = sim.kept_out.saturating_add(ticks.max(1));
    if sim.kept_out as f32 * crate::MINUTES_PER_TICK < WAIT_MINUTES {
        return true;
    }
    sim.kept_out = 0;
    let Some(task) = sim.current().map(|a| a.task) else { return true };
    crate::world::end_activity(sim, ctx.content, objects);
    if !task.directed {
        let retry = crate::social::minutes_to_ticks(ctx.content.autonomy.retry_minutes);
        sim.remember_unreachable(task.kind, ctx.tick + retry);
    }
    true
}

/// Whether someone else is in the room of private use `doing` (`(object, interaction)`) besides
/// `me`: not whoever does it with them, nor a baby. Then it isn't done (`world::progress`).
pub(crate) fn overlooked(ctx: &Ctx, objects: &[ObjectInstance], me: u32, doing: (u32, usize)) -> bool {
    let Some(o) = objects.get(doing.0 as usize) else { return false };
    let lot = ctx.nav.lot;
    let room = room_of_object(ctx.content, lot, o);
    room != OUTDOORS
        && ctx.briefs.iter().enumerate().any(|(j, b)| {
            j as u32 != me && b.here && !b.away && !b.baby && room_of(lot, b.pos) == room && b.game.is_none_or(|g| (g.0, g.1) != doing)
        })
}

/// Whether the content has somewhere to try for a baby (`Interaction::conceive`): then the
/// social goes on in bed, and needs one.
pub(crate) fn beds_in_content(content: &Content) -> bool {
    content.conceive_beds
}

/// Where partners `a` and `b` could try for a baby now: a working double bed at home with its
/// use for it (`Interaction::conceive`), nobody else on it, in a room indoors with a door,
/// kept shut by nobody and with nobody else in it (`others_in`: whether someone but the two,
/// and babies, is in a room); the nearest to `from`. None: nowhere.
#[allow(clippy::too_many_arguments)]
pub(crate) fn bed_for(
    content: &Content,
    lot: &Lot,
    rooms: &[crate::rooms::RoomInfo],
    objects: &[ObjectInstance],
    on_home: &[u32],
    shut: &[Shut],
    (a, b): (u32, u32),
    from: [f32; 2],
    others_in: impl Fn(u16) -> bool,
) -> Option<(u32, usize)> {
    let mut best: Option<(f32, u32, usize)> = None;
    for &id in on_home {
        // (Free, or lain on by one of the two: a nap they got up from to be asked.)
        let Some(o) = objects.get(id as usize).filter(|o| !o.broken() && o.users().all(|u| u == a || u == b)) else { continue };
        let def = &content.objects[o.def];
        let Some(i) = def.interactions.iter().position(|i: &Interaction| i.conceive && o.allows(i, a)) else { continue };
        let room = room_of_object(content, lot, o);
        let walled = room != OUTDOORS && rooms.iter().any(|r| !r.garden && r.id == room && r.doors > 0);
        if !walled || shut.iter().any(|s| s.room == room && s.by != a && s.by != b) || others_in(room) {
            continue;
        }
        let c = o.centre(content);
        let d = (c[0] - from[0]).hypot(c[1] - from[1]);
        if best.is_none_or(|(e, ..)| d < e) {
            best = Some((d, id, i));
        }
    }
    best.map(|(_, o, i)| (o, i))
}

/// `bed_for` partners `a` and `b` as a tick's step sees them (free will choosing the social).
pub(crate) fn bed_in_step(ctx: &Ctx, objects: &[ObjectInstance], a: usize, b: usize) -> Option<(u32, usize)> {
    let home = ctx.households[ctx.briefs[a].household as usize].plot?;
    let lot = ctx.nav.lot;
    let others_in = |room: u16| ctx.briefs.iter().enumerate().any(|(j, s)| j != a && j != b && s.here && !s.away && !s.baby && room_of(lot, s.pos) == room);
    bed_for(ctx.content, lot, ctx.rooms, objects, ctx.index.on(home), ctx.shut, (a as u32, b as u32), ctx.briefs[a].pos, others_in)
}

/// `bed_for` partners `a` and `b` in `world` as it is now.
pub(crate) fn bed_in_world(w: &World, a: usize, b: usize) -> Option<(u32, usize)> {
    let home = w.households[w.sims[a].household as usize].plot?;
    let others_in = |room: u16| {
        w.sims.iter().enumerate().any(|(j, s)| {
            j != a && j != b && s.here() && s.away_until.is_none() && !w.content.life.baby(s.age) && room_of(&w.lot, s.pos) == room
        })
    };
    let on: Vec<u32> = (0..w.objects.len() as u32).filter(|&o| w.object_plot.get(o as usize).copied().flatten() == Some(home)).collect();
    bed_for(&w.content, &w.lot, &w.rooms, &w.objects, &on, &w.shut, (a as u32, b as u32), w.sims[a].pos, others_in)
}

/// After the step: partners whose *Try for a baby* went well (`Sim::to_bed`) go to bed together
/// (with the content's beds for it; without, a baby may come of the social itself, as it did
/// before beds had a use for it). With no bed free for them now, nothing comes of it.
pub(crate) fn after_step(w: &mut World) {
    for a in 0..w.sims.len() {
        let Some(b) = w.sims[a].to_bed.take() else { continue };
        let b = b as usize;
        if !beds_in_content(&w.content) {
            w.sims[a].conceived = Some(b as u32);
            continue;
        }
        let Some((object, interaction)) = bed_in_world(w, a, b) else { continue };
        let World { content, objects, sims, .. } = w;
        for s in [a, b] {
            let sim = &mut sims[s];
            if sim.current().is_some() {
                crate::world::end_activity(sim, content, objects);
            }
            sim.follow_up = None;
            sim.queue.push_front(Task { kind: TaskKind::Use { object, interaction }, directed: false, routine: false });
            sim.game = Some((object, interaction, false));
        }
    }
}
