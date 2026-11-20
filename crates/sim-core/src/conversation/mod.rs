//! Running social interactions between two Sims.
//!
//! An actor calls out the target's name when they're within earshot (`within_earshot`): the
//! target turns to them, and the two walk to meet halfway. Out of earshot (only on the player's
//! orders) the actor walks over until they are; a target too busy to come is walked up to (re-
//! routing if they move) and waited for until free. Then both face each other for the
//! interaction's duration.
//! Success is rolled when the conversation starts (so the UI can show the reaction
//! halfway through); relationship changes, feelings and events apply at the end.
//!
//! Others nearby may join in on a joinable social (a group conversation, up to `MAX_GROUP`
//! people): a `Join` task walks them over, finds them a place around the pair, and they listen,
//! filling their needs like the one talked to. When a social goes well, everyone there grows a
//! little closer. They stay for the leader's next social with someone there, and go otherwise.
//! Someone joining walks to their place in the group.
//!
//! Two talk, and others listen in, only where they see each other (`in_sight`): not through a
//! wall, a window or a shut front door. Whoever goes over to someone goes round to where they
//! can (`approach_tile`), and calling out doesn't carry through a front door either.
//!
//! People sitting stay sitting for a talk: the one talked to on the sofa or at the table keeps
//! their seat (and carries on with what they sat down to do afterwards), and people sitting near
//! each other talk, or listen in, from their seats (`from_seat`), without getting up. Only what
//! takes them closer than talking does (a hug, a kiss, a fight: `SEATED_DISTANCE`) gets them up.

use crate::MINUTES_PER_TICK;
use crate::content::{Content, MAX_NEEDS, Pose, stage_in};
use crate::lot::{DiagDir, Edge, Lot, OUTDOORS, Side};
use crate::path::{Grounds, NavGrid, STOREY_WALK, StairMap};
use crate::rng::Rng;
use crate::social::{self, Effect, EventKind, EventLog, Relationships, SocialDef};
use crate::world::{
    Activity, ObjectInstance, Phase, Sim, Task, TaskKind, World, end_activity, grounds, route,
};

/// How long an actor waits for a busy target before giving up (in all: following them about
/// counts too, see `follow`).
const WAIT_LIMIT_TICKS: u32 = (10.0 * crate::TICKS_PER_MINUTE as f32) as u32;
/// Re-route when the target is this far from the route's end. Routes end on a tile
/// *next to* the target (up to ~1.6 m from them), so this must exceed that.
const REPATH_DISTANCE: f32 = 2.4;
/// Extra distance at which a conversation can start.
const ENGAGE_SLACK: f32 = 0.8;
/// Rejected interactions end early.
pub(crate) const FAILURE_LENGTH: f32 = 0.4;
/// The most people in one conversation: the two talking and those listening in.
pub(crate) const MAX_GROUP: usize = 4;
/// How far from the middle of a conversation someone joining in can take their place in it.
const JOIN_RANGE: f32 = 2.6;
/// How long someone listening in waits between two of the leader's socials (for the next one
/// to start with someone there) before they go: long enough for the leader to choose again, three
/// real seconds of standing by (seen, as `world::AUTONOMY_DELAY_TICKS` is), past the leader's
/// pause before choosing and their next turn at lower detail (`lod::LOD_TICKS`).
const REGROUP_TICKS: u32 = crate::secs(3.0) as u32;
const _: () = assert!(
    REGROUP_TICKS > crate::world::AUTONOMY_DELAY_TICKS
        && REGROUP_TICKS as u64 > crate::lod::LOD_TICKS
);
/// How close together the people in a group stand at the least.
const GROUP_SPACING: f32 = 0.6;
/// How far a call carries (metres of open floor; see `within_earshot`): across a room, or a
/// garden.
const EARSHOT: f32 = 12.0;
/// A doorway muffles a call as much as this much more open floor: the next room is within
/// earshot near the door.
const DOOR_MUFFLE: f32 = 6.0;
/// How long after someone calls out the one called hears it and turns to them (real time: it's
/// seen).
const HEAR_TICKS: u32 = crate::secs(0.5) as u32;
/// How long someone calling out stands there before the two walk over to meet (real time).
const CALL_TICKS: u32 = crate::secs(1.5) as u32;
/// How far apart two people sitting can talk from their seats: along a sofa, across a table,
/// from an armchair to the sofa beside it.
pub(crate) const SEAT_RANGE: f32 = 2.5;
/// Socials done this close or closer (a hug, a kiss, a fight) need them on their feet; a talk
/// (`SocialDef::distance` 1 m) doesn't.
pub(crate) const SEATED_DISTANCE: f32 = 0.9;
/// How often someone sitting thinks of a word with those sitting near them, and the chance
/// they do, each time, if anything they'd say comes to mind (`from_seat`).
pub(crate) const SEAT_TALK_TICKS: u32 = (5.0 * crate::TICKS_PER_MINUTE as f32) as u32;
pub(crate) const SEAT_TALK_CHANCE: f32 = 0.5;
/// Someone joining in this close to their place in the group settles into it; further off,
/// they walk there (`take_place`).
const SETTLE: f32 = 0.3;

pub fn duration(s: &SocialDef, success: bool) -> f32 {
    if success {
        s.minutes
    } else {
        s.minutes * FAILURE_LENGTH
    }
}

/// Whether two can court at all: both grown up and not family (`rel` between them; attraction is
/// each social's and spark's own). Free will, the player's socials and `SetBond` keep to it alike.
pub(crate) fn may_court(grown_ups: bool, rel: &social::Relationship) -> bool {
    grown_ups && rel.kin.is_none()
}

/// Whether `actor` can start `s` with `target` (relationship requirements, both their life
/// stages and attraction).
pub fn social_allowed(
    content: &Content,
    rels: &Relationships,
    actor: &Sim,
    target: &Sim,
    s: &SocialDef,
) -> bool {
    let rel = rels.get(actor.id as usize, target.id as usize);
    let romantic = s.tags & content.social_rules.romantic_tags != 0;
    let grown_ups = actor.adult(content) && target.adult(content);
    let stage = |sim: &Sim| content.life.stage(sim.age);
    actor.id != target.id
        && s.requires.allows(rel)
        && stage_in(s.stages, stage(actor))
        && stage_in(s.target_stages, stage(target))
        && (!romantic || (may_court(grown_ups, rel) && actor.attracted_to(target)))
}

/// Probability that `target` responds well to `actor` doing `s`.
pub fn acceptance_chance(
    content: &Content,
    rels: &Relationships,
    actor: &Sim,
    target: &Sim,
    s: &SocialDef,
) -> f32 {
    let romantic = s.tags & content.social_rules.romantic_tags != 0;
    if romantic && !target.attracted_to(actor) {
        return 0.0; // not interested in the actor's gender
    }
    accepts(content, rels, actor, target, &s.acceptance, s.tags)
}

/// Probability that `target` says yes to `actor` (terms `acc`, for something with `tags`): how
/// the target feels about the actor, their chemistry, the target's mood, traits and emotion.
/// Socials use it, and so do phone calls (`phone.rs`).
pub(crate) fn accepts(
    content: &Content,
    rels: &Relationships,
    actor: &Sim,
    target: &Sim,
    acc: &social::Acceptance,
    tags: crate::content::TagMask,
) -> f32 {
    let (a, t) = (actor.id as usize, target.id as usize);
    let view = rels.get(t, a);
    let mut p = acc.base
        + acc.friendship * view.friendship
        + acc.romance * view.romance
        + acc.chemistry * rels.chemistry(a, t)
        + acc.mood * (target.mood(content) - 0.5)
        + target.mods.acceptance(tags)
        + actor.mods.success(tags);
    if let Some(e) = target.emotion(content) {
        p += content.emotions[e].mods.acceptance(tags);
    }
    if acc.taken_penalty > 0.0 && rels.partner_of(t).is_some_and(|p| p != a) {
        p -= acc.taken_penalty;
    }
    for (n, k) in acc.needs.iter().enumerate().take(content.needs.len()) {
        p += k * (1.0 - target.needs[n]);
    }
    p.clamp(0.0, 1.0)
}

/// Social options `actor` has toward `target`: `(social index, success chance)`. (Goodbyes at
/// the end of a visit, `visits.farewell`, aren't on offer: guests say them as they leave; nor is
/// a host's offer of something to eat or drink, `visits.offer`, which hosts make of their own
/// accord.)
pub fn options(w: &World, actor: usize, target: usize) -> Vec<(usize, f32)> {
    let (Some(a), Some(t)) = (w.sims.get(actor), w.sims.get(target)) else {
        return Vec::new();
    };
    let offer = w.content.visits.offer.as_ref().map(|o| o.social);
    w.content
        .socials
        .iter()
        .enumerate()
        .filter(|&(i, s)| {
            !w.content.visits.farewell.contains(&i)
                && offer != Some(i)
                && social_allowed(&w.content, &w.relationships, a, t, s)
        })
        .map(|(i, s)| (i, acceptance_chance(&w.content, &w.relationships, a, t, s)))
        .collect()
}

/// A free tile next to `target` (or under it), closest to `from`, from where `target` is in
/// sight (`in_sight`): never the one on the far side of the wall, nor indoors behind the front
/// door from someone on the doorstep (whoever answers it steps out to them).
pub(crate) fn approach_tile(nav: &NavGrid, target: [f32; 2], from: [f32; 2]) -> Option<(i32, i32)> {
    let (tx, tz) = (target[0].floor() as i32, target[1].floor() as i32);
    let mut best = None;
    let mut best_d = f32::INFINITY;
    for dz in -1..=1 {
        for dx in -1..=1 {
            let (x, z) = (tx + dx, tz + dz);
            if !nav.tile_free(x, z) || !in_sight(nav, [x as f32 + 0.5, z as f32 + 0.5], target) {
                continue;
            }
            let d = (x as f32 + 0.5 - from[0]).hypot(z as f32 + 0.5 - from[1]);
            if d < best_d {
                best_d = d;
                best = Some((x, z));
            }
        }
    }
    best
}

mod meet;
mod outcome;
mod sight;
mod update;

use meet::*;
pub(crate) use outcome::*;
pub(crate) use sight::*;
pub(crate) use update::*;

#[cfg(test)]
mod tests;
