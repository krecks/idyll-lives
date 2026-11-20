//! The post at home: a letterbox (content `post`), letters written to friends (content
//! `letter`), and the hose's round of the garden (content `waters`).
//!
//! Every morning at `POST_MINUTE` the post comes to the letterboxes on each household's home
//! plot, a flat's kept for it in its building's hall (`ObjectInstance::keeper`) (`deliver`): each one empty (closed) is opened, a letter in its slot (`ObjectInstance::open`,
//! saved as any object's). Someone at home fetches it (`Interaction::post`, offered while it's
//! open, closing it as they start): a household with free will sends the nearest one at home
//! who is up and between things, from then until `FETCH_UNTIL` (`prompt`), as short a use would
//! seldom win against the rest on its own. As they finish they've read it (`read`): a letter waiting
//! for the household (`Household::letters`), or now and then (`FRIEND_CHANCE`) one from a friend
//! of theirs anyway, gives them a little social (`LETTER_SOCIAL`) and friendship
//! (`LETTER_FRIENDSHIP`), and the story says who wrote; the rest is bills and flyers, the use's
//! own fun.
//!
//! A letter written to the end at a desk (`Interaction::letter`, `send`) goes to the writer's
//! closest friend outside their household (the most friendship, a friend at least) whose home
//! has a letterbox. It waits in that household's `letters` (at most `MOST_LETTERS`, saved) for
//! the post to be fetched there; with no friend who has a letterbox it isn't sent.

use crate::content::Content;
use crate::social::{EventKind, LEVEL_FRIEND, friend_level};
use crate::world::{ObjectInstance, Sim, Task, TaskKind, World};

/// When the post comes (minute of the day: 7 o'clock).
pub const POST_MINUTE: f32 = 7.0 * 60.0;
/// Social a letter from a friend gives the one who reads it.
pub const LETTER_SOCIAL: f32 = 0.15;
/// Friendship a letter adds, both ways.
pub const LETTER_FRIENDSHIP: f32 = 2.0;
/// Chance a morning's post holds a letter from a friend when none was written to the household.
pub const FRIEND_CHANCE: f32 = 0.3;
/// Letters that wait at most for a household (the oldest go first).
pub const MOST_LETTERS: usize = 5;
/// Until when (minute of the day) someone at home is sent to fetch the post (`prompt`): nine in
/// the evening. The post that is still in then waits for the morning.
pub const FETCH_UNTIL: f32 = 21.0 * 60.0;

/// What a resident did with the post this tick (`Sim::mail`), seen to after the step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mail {
    /// Fetched and read the post.
    Read,
    /// Wrote a letter to a friend.
    Wrote,
}

/// Whether `def` is a letterbox (it has a use that fetches the post).
fn letterbox(content: &Content, o: &ObjectInstance) -> bool {
    content.objects[o.def].interactions.iter().any(|i| i.post)
}

/// After the residents' step: the morning's post comes, and what they read or wrote is seen to.
pub(crate) fn after_step(world: &mut World) {
    if world.tick.is_multiple_of(60 * crate::TICKS_PER_MINUTE)
        && (crate::clock::minute_of_day(world.tick) - POST_MINUTE).abs() < 0.5
    {
        deliver(world);
    }
    if world.tick.is_multiple_of(crate::TICKS_PER_MINUTE) {
        prompt(world);
    }
    for i in 0..world.sims.len() {
        match world.sims[i].mail.take() {
            Some(Mail::Read) => read(world, i),
            Some(Mail::Wrote) => send(world, i),
            None => {}
        }
    }
}

/// The morning's post: every empty letterbox on a household's home plot opens, a letter in its slot.
pub(crate) fn deliver(world: &mut World) {
    let mut changed = false;
    for h in 0..world.households.len() {
        let Some(plot) = world.households[h].plot else {
            continue;
        };
        for &o in world.object_index.on(plot) {
            let obj = &mut world.objects[o as usize];
            if !obj.open && !obj.broken() && letterbox(&world.content, obj) {
                obj.open = true;
                changed = true;
            }
        }
    }
    if changed {
        world.structure_version = world.structure_version.wrapping_add(1);
        world.rooms_need_clean_rescore();
    }
}

/// The post is in at a household with free will, and nobody is fetching it: the nearest one at
/// home who may, up and between things (doing nothing, nothing next), goes for it, as their own
/// choice (`Task::directed` false).
fn prompt(world: &mut World) {
    let minute = crate::clock::minute_of_day(world.tick);
    if !(POST_MINUTE..FETCH_UNTIL).contains(&minute) {
        return;
    }
    let tick = world.tick;
    for h in 0..world.households.len() {
        let Some(plot) = world.households[h]
            .plot
            .filter(|_| world.households[h].free_will)
        else {
            continue;
        };
        for &o in world.object_index.on(plot) {
            let obj = &world.objects[o as usize];
            let def = &world.content.objects[obj.def];
            let Some(k) = def.interactions.iter().position(|i| i.post) else {
                continue;
            };
            if !obj.open || obj.broken() || obj.users().next().is_some() {
                continue;
            }
            let kind = TaskKind::Use {
                object: o,
                interaction: k,
            };
            let at = obj.centre(&world.content);
            let inter = &def.interactions[k];
            let fetching = |t: &TaskKind| matches!(*t, TaskKind::Use { object, .. } if object == o);
            let going = |s: &Sim| {
                s.current().is_some_and(|a| fetching(&a.task.kind))
                    || s.queue().any(|t| fetching(&t.kind))
            };
            if world.sims.iter().any(going) {
                continue;
            }
            let far = |s: &Sim| (s.pos[0] - at[0]).hypot(s.pos[1] - at[1]);
            let who = (0..world.sims.len())
                .filter(|&j| {
                    let s = &world.sims[j];
                    let (x, z) = s.tile();
                    s.household as usize == h
                        && s.current().is_none()
                        && s.queue().next().is_none()
                        && world.plot_at(x, z) == Some(plot)
                        && crate::partners::free_now(&world.content, s, tick)
                        && crate::world::unfit(&world.content, s.age, inter).is_none()
                        && !s.couldnt_reach(o, tick)
                })
                .min_by(|&a, &b| far(&world.sims[a]).total_cmp(&far(&world.sims[b])));
            if let Some(j) = who {
                world.sims[j].queue.push_back(Task {
                    kind,
                    directed: false,
                    routine: false,
                });
            }
        }
    }
}

/// Resident `i` read the post: a letter waiting for their household, or now and then one from a
/// friend of theirs.
fn read(world: &mut World, i: usize) {
    let h = world.sims[i].household as usize;
    let waiting = world
        .households
        .get_mut(h)
        .filter(|hh| !hh.letters.is_empty())
        .map(|hh| hh.letters.remove(0))
        .filter(|&w| (w as usize) < world.sims.len() && w as usize != i);
    let from = waiting.or_else(|| {
        let friends = friends_of(world, i);
        let roll = world.rng.next_f32();
        (!friends.is_empty() && roll < FRIEND_CHANCE).then(|| {
            let k = (world.rng.next_u32() as usize) % friends.len();
            friends[k]
        })
    });
    let Some(w) = from else { return };
    let w = w as usize;
    if let Some(n) = world.content.needs.iter().position(|n| n.id == "social") {
        let s = &mut world.sims[i];
        s.needs[n] = (s.needs[n] + LETTER_SOCIAL).min(1.0);
    }
    world.relationships.adjust(i, w, LETTER_FRIENDSHIP, 0.0);
    world.relationships.adjust(w, i, LETTER_FRIENDSHIP, 0.0);
    world
        .events
        .push(world.tick, EventKind::GotLetter, i, w, None);
}

/// Friends of resident `i` outside their household (a friend at least), still in town.
fn friends_of(world: &World, i: usize) -> Vec<u32> {
    let h = world.sims[i].household;
    (0..world.sims.len())
        .filter(|&j| {
            j != i
                && world.sims[j].household != h
                && world.sims[j].gone.is_none()
                && friend_level(world.relationships.get(i, j).friendship) >= LEVEL_FRIEND
        })
        .map(|j| j as u32)
        .collect()
}

/// Resident `i` wrote a letter: to their closest friend outside the household whose home has a
/// letterbox, there by the next morning's post.
fn send(world: &mut World, i: usize) {
    let friends = friends_of(world, i);
    let has_box = |world: &World, j: u32| {
        let h = world.sims[j as usize].household as usize;
        world
            .households
            .get(h)
            .and_then(|hh| hh.plot)
            .is_some_and(|p| {
                world
                    .object_index
                    .on(p)
                    .iter()
                    .any(|&o| letterbox(&world.content, &world.objects[o as usize]))
            })
    };
    let Some(to) = friends
        .iter()
        .copied()
        .filter(|&j| has_box(world, j))
        .max_by(|&a, &b| {
            let f = |j: u32| world.relationships.get(i, j as usize).friendship;
            f(a).total_cmp(&f(b)).then(b.cmp(&a))
        })
    else {
        return;
    };
    let h = world.sims[to as usize].household as usize;
    let letters = &mut world.households[h].letters;
    if letters.len() >= MOST_LETTERS {
        letters.remove(0);
    }
    letters.push(i as u32);
}

/// A hose's round (`Interaction::waters`) at `object`: every growing crop on its plot comes on
/// `share` of its way to ripe. Whether a crop shows another stage now.
pub(crate) fn water_garden(
    content: &Content,
    objects: &mut [ObjectInstance],
    object_plot: &[Option<u32>],
    object: u32,
    share: f32,
) -> bool {
    let Some(plot) = object_plot.get(object as usize).copied().flatten() else {
        return false;
    };
    let mut reshaped = false;
    for (k, o) in objects.iter_mut().enumerate() {
        if object_plot.get(k).copied().flatten() != Some(plot)
            || content.objects[o.def].grows.is_none()
            || !(0.0..1.0).contains(&o.growth)
            || !content.objects[o.def]
                .interactions
                .iter()
                .any(|i| i.tends > 0.0)
        {
            continue;
        }
        let before = o.growth_stage();
        o.growth = (o.growth + share).min(1.0);
        reshaped |= o.growth_stage() != before;
    }
    reshaped
}
