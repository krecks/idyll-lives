//! Setting up something a resident asked for (`world::TaskKind::SetUp`).
//!
//! When the player says yes to what a resident asked to buy (`planner::accept_purchase`), it
//! isn't simply there. It's paid for and comes as a parcel to the front door (a `Setup`, kept in
//! `World::setups` until it's done). The one who asked goes to the door and picks the parcel up
//! (stage 0, `SetUpRules::fetch_minutes`), carries it in the hands (`Sim::held`, the content's
//! `parcel` item) to beside where the thing goes, and puts it together there (stage 1,
//! `SetUpRules::minutes`). Then it stands there (`planner::place_wanted`), the story says who set
//! it up, and the renderer shows it springing into place, as things bought in Build mode do.
//!
//! The one who asked does it when they're at home and free (`free`): not asleep, at work or out,
//! not busy with what the player asked for, in a conversation, cooking, eating or badly needing
//! something. They drop anything else they chose for themselves. A grown-up does it for a child.
//! After `SetUpRules::help_hours` anyone grown-up at home may do it instead; after
//! `SetUpRules::wait_hours` it's set up for them anyway, so nothing waits for ever. Where it goes
//! is chosen as they set off with the parcel (the free spot anything bought gets, or where the
//! older thing it replaces stands): taken meanwhile, it goes at the next free spot. With no room
//! for it anywhere at home, it goes back to the shop and the money comes back.
//!
//! Not saved mid-way: after a load the parcel is back at the door, and whoever does it sets off
//! again (`save::SetupSave` keeps what was bought).

use crate::content::{Content, Pose};
use crate::items::Held;
use crate::social::EventKind;
use crate::world::{Activity, Ctx, Layout, ObjectInstance, Phase, Sim, Task, TaskKind, World};

/// How often (ticks) waiting parcels look for someone to set them up.
const CHECK_TICKS: u64 = crate::TICKS_PER_MINUTE;
/// Someone sent off who didn't get there (no way to the door or the spot): not sent again for
/// this long (game minutes).
const RETRY_MINUTES: u64 = 30;
/// How far (m) from the front door the parcel lies, and the one picking it up stands.
const PARCEL_OUT: f32 = 0.3;
const STAND_OUT: f32 = 0.85;
/// How far (m) in front of them the parcel stands while they put it together.
const REACH: f32 = 0.5;
/// Item ids at and above this are parcels in the view (`view::items_json`), not real items.
pub const PARCEL_IDS: u32 = u32::MAX - 1024;

/// Where a thing goes, as for `World::buy_at`: its tile and facing, and its layout (the offset of a
/// thing on a top).
pub type Spot = ([i32; 3], Layout);

/// Something bought that waits to be set up.
#[derive(Debug, Clone, PartialEq)]
pub struct Setup {
    pub household: u32,
    /// Who asked for it.
    pub sim: u32,
    /// Object type.
    pub object: usize,
    /// The tick it was bought.
    pub since: u64,
    /// What was paid (given back if there's no room for it).
    pub paid: i64,
    /// Not sent again before this tick (`RETRY_MINUTES`). Not saved.
    pub(crate) retry: u64,
    /// Where it goes (as for `World::buy_at`), chosen as someone set off for it. Not saved.
    pub(crate) spot: Option<Spot>,
    /// Where its parcel lies to be seen (`view::items_json`), none while in someone's hands.
    /// Not saved.
    pub(crate) parcel: Option<[f32; 2]>,
    /// The resident last sent off for it (or found at it), to see each tick whether they're at it
    /// still (`at_it`): nobody else is, as nobody is sent while someone is. Not saved.
    by: Option<u32>,
    /// Where it waits at the door (`door`), and for which plot and `World::nav_version` and
    /// `World::lot_version`: found again only when they change. Not saved.
    door: Option<(DoorKey, Door)>,
}

/// A home's plot, and the versions of walking and of the lot, as its door was found at.
type DoorKey = (u32, u32, u32);

/// Where a parcel lies at the door, and where the one picking it up stands (`door`).
type Door = ([f32; 2], [f32; 2]);

impl Setup {
    /// Where it goes, once someone set off for it (as for `World::buy_at`).
    pub fn spot(&self) -> Option<Spot> {
        self.spot
    }

    pub(crate) fn new(household: u32, sim: u32, object: usize, since: u64, paid: i64) -> Setup {
        Setup {
            household,
            sim,
            object,
            since,
            paid,
            retry: 0,
            spot: None,
            parcel: None,
            by: None,
            door: None,
        }
    }
}

/// Whether `kind` is setting up `object`.
fn for_object(kind: &TaskKind, object: usize) -> bool {
    matches!(*kind, TaskKind::SetUp { object: o, .. } if o as usize == object)
}

/// Whether resident `x` is at `s` now (on their way, carrying the parcel, putting it together) or
/// about to be.
fn at_it(x: &Sim, s: &Setup) -> bool {
    x.here()
        && x.household == s.household
        && (x
            .current
            .as_ref()
            .is_some_and(|a| for_object(&a.task.kind, s.object))
            || x.queue.iter().any(|t| for_object(&t.kind, s.object)))
}

/// The resident at `s` now, if anyone is (`at_it`).
fn doing(w: &World, s: &Setup) -> Option<usize> {
    w.sims.iter().position(|x| at_it(x, s))
}

/// Notes that resident `j` is at what household `h` bought of `object` (each one, should they
/// have bought two: `for_object` doesn't tell them apart).
fn mark_by(w: &mut World, h: u32, object: usize, j: usize) {
    for s in w.setups.iter_mut() {
        if s.household == h && s.object == object {
            s.by = Some(j as u32);
        }
    }
}

/// Where `setups[k]`'s parcel waits at the front door of `plot` (`door`), found again only when
/// walking or the lot changes there.
fn door_of(w: &mut World, k: usize, plot: u32) -> Door {
    let key = (plot, w.nav_version, w.lot_version);
    if let Some((seen, at)) = w.setups[k].door
        && seen == key
    {
        return at;
    }
    let at = door(w, plot);
    w.setups[k].door = Some((key, at));
    at
}

/// Where the parcel waits at the front door of `plot`: the point it lies at, and where the one
/// picking it up stands (facing it). The visitors' knocking spot (`life::knock_spot`), else where
/// people arrive at the plot.
fn door(w: &World, plot: u32) -> Door {
    let p = &w.plots[plot as usize];
    match crate::life::knock_spot(&w.nav(), p) {
        Some(((x, z), _, yaw)) => {
            // (The door is on the edge of the knocking tile the way a visitor faces.)
            let (dx, dz) = (yaw.sin(), yaw.cos());
            let middle = [x as f32 + 0.5, z as f32 + 0.5];
            let door = [middle[0] + dx * 0.5, middle[1] + dz * 0.5];
            (
                [door[0] - dx * PARCEL_OUT, door[1] - dz * PARCEL_OUT],
                [door[0] - dx * STAND_OUT, door[1] - dz * STAND_OUT],
            )
        }
        None => {
            let a = p.arrival_point();
            ([a[0], a[1] + 0.3], a)
        }
    }
}

/// Where `object` would go at household `h`'s home now, where to stand to put it together, and
/// its middle: over the older thing it replaces (`planner::replaces`), else the first good free
/// spot (`World::find_spot`). None with no room for it anywhere.
fn spot(w: &World, h: u32, object: usize) -> Option<(Spot, (i32, i32), [f32; 2])> {
    let plot = w.households.get(h as usize)?.plot?;
    let content = &w.content;
    let fit = |x: i32, z: i32, rot: u8, layout: Layout| {
        let obj = w.check_fit(object, x, z, rot, layout).ok()?;
        let stand = w
            .approach_tiles(&obj)
            .into_iter()
            .next()
            .unwrap_or_else(|| obj.front_tile(content));
        Some((([x, z, i32::from(rot)], layout), stand, obj.centre(content)))
    };
    if let Some(old) = crate::planner::replaces(w, h as usize, object) {
        let o = &w.objects[old as usize];
        let stand = w
            .object_index
            .links
            .approach_of(old)
            .unwrap_or_else(|| o.front_tile(content));
        return Some((
            ([o.x, o.z, i32::from(o.rot)], Layout::default()),
            stand,
            o.centre(content),
        ));
    }
    let on_top = content.objects[object].layer == crate::content::Layer::Surface;
    w.find_spot(plot, object)
        .into_iter()
        .find_map(|(x, z, rot, offset)| {
            let layout = if on_top {
                Layout::at(offset)
            } else {
                Layout::default()
            };
            fit(x, z, rot, layout)
        })
}

/// Whether there's room for `object` at household `h`'s home (`spot`).
pub(crate) fn room_for(w: &World, h: u32, object: usize) -> bool {
    spot(w, h, object).is_some()
}

/// Whether resident `s` can set something up at home (`plot`) now: there, awake, up, not out,
/// carrying nothing, not told to do anything, not talking, and not at something that can't be
/// dropped (cooking, eating, a baby in the arms, the phone, the door, the bathroom, seeing to a
/// need that's run low).
fn free(content: &Content, w: &World, s: &Sim, plot: u32) -> bool {
    let (x, z) = s.tile();
    s.here()
        && s.away_until.is_none()
        && s.visiting.is_none()
        && s.transition.is_none()
        && s.bound_for_car.is_none()
        && s.carried_by.is_none()
        && s.held.is_none()
        && s.engaged_with.is_none()
        && s.heeding.is_none()
        && s.pose != Pose::Lie
        && !s.asleep(content)
        && !content.life.baby(s.age)
        && w.plot_at(x, z) == Some(plot)
        && !crate::life::leaving_home(w, s)
        && !s.queue.iter().any(|t| t.directed)
        && s.current.as_ref().is_none_or(|a| {
            !a.task.directed
                && !matches!(
                    a.phase,
                    Phase::Conversing { .. } | Phase::Calling { .. } | Phase::Waiting { .. }
                )
                && match a.task.kind {
                    TaskKind::Use {
                        object,
                        interaction,
                    } => w
                        .objects
                        .get(object as usize)
                        .and_then(|o| content.objects[o.def].interactions.get(interaction))
                        .is_some_and(|it| {
                            !it.private
                                && (0..content.needs.len()).all(|n| {
                                    it.total_gain[n] <= 0.0
                                        || s.needs[n] >= content.planner.urgent_below
                                })
                        }),
                    TaskKind::MoveTo { .. } | TaskKind::Clean { .. } | TaskKind::Repair { .. } => {
                        true
                    }
                    _ => false,
                }
        })
}

/// Who sets `s` up now: the one who asked, if they're a grown-up and free; else (`others`, or
/// a child's) the free grown-up at home nearest the door.
fn who(w: &World, s: &Setup, plot: u32, others: bool) -> Option<usize> {
    let content = &w.content;
    let asker = w.sims.get(s.sim as usize);
    let theirs = asker.is_some_and(|a| a.here() && a.household == s.household);
    if let Some(a) = asker.filter(|a| theirs && a.adult(content))
        && free(content, w, a, plot)
    {
        return Some(s.sim as usize);
    }
    if !others && theirs && asker.is_some_and(|a| a.adult(content)) {
        return None;
    }
    let (_, stand) = door(w, plot);
    w.sims
        .iter()
        .enumerate()
        .filter(|(_, x)| {
            x.household == s.household && x.adult(content) && free(content, w, x, plot)
        })
        .map(|(j, x)| (j, (x.pos[0] - stand[0]).hypot(x.pos[1] - stand[1])))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(j, _)| j)
}

/// Each tick, after the step: what was put together this tick is placed, and now and then each
/// parcel waiting at a door finds someone to set it up (or, waited long enough, is set up anyway).
pub(crate) fn update(w: &mut World) {
    for i in 0..w.sims.len() {
        if let Some(object) = w.sims[i].set_up.take() {
            let h = w.sims[i].household;
            if let Some(k) = w
                .setups
                .iter()
                .position(|s| s.household == h && s.object == object as usize)
            {
                place(w, k, Some(i));
            }
        }
    }
    if w.setups.is_empty() {
        return;
    }
    refresh_parcels(w);
    if !w.tick.is_multiple_of(CHECK_TICKS) {
        return;
    }
    let tick = w.tick;
    let rules = &w.content.object_rules.set_up;
    let hours = |h: f32| (h * 60.0 * crate::TICKS_PER_MINUTE as f32) as u64;
    let (help, wait) = (hours(rules.help_hours), hours(rules.wait_hours));
    let mut k = 0;
    while k < w.setups.len() {
        let s = w.setups[k].clone();
        let Some(plot) = w.households.get(s.household as usize).and_then(|h| h.plot) else {
            // (Moved out meanwhile: the money back.)
            sent_back(w, k);
            continue;
        };
        if let Some(j) = doing(w, &s) {
            mark_by(w, s.household, s.object, j);
            k += 1;
            continue;
        }
        // Nobody got to it: set up for them.
        if tick.saturating_sub(s.since) >= wait {
            place(w, k, None);
            continue;
        }
        if tick < s.retry {
            k += 1;
            continue;
        }
        let others = tick.saturating_sub(s.since) >= help;
        if let Some(j) = who(w, &s, plot, others) {
            send(w, k, j, plot);
        }
        k += 1;
    }
}

/// Where each parcel is seen (`Setup::parcel`): at the door, in someone's hands (nowhere), or put
/// down by where it goes while they put it together. The view's items change with it. Every
/// tick: only the one sent off for it is looked at (`Setup::by`), and its door is kept
/// (`door_of`).
pub(crate) fn refresh_parcels(w: &mut World) {
    for k in 0..w.setups.len() {
        let s = &w.setups[k];
        let Some(plot) = w.households.get(s.household as usize).and_then(|h| h.plot) else {
            continue;
        };
        let by =
            s.by.and_then(|j| w.sims.get(j as usize))
                .filter(|x| at_it(x, s));
        let parcel = match by.and_then(|x| x.current.as_ref().map(|a| (x, a))) {
            // (On the floor in front of them, towards where it goes: within reach.)
            Some((
                x,
                Activity {
                    task:
                        Task {
                            kind: TaskKind::SetUp { stage: 1, at, .. },
                            ..
                        },
                    phase: Phase::Using { .. },
                    ..
                },
            )) => {
                let (dx, dz) = (at[0] - x.pos[0], at[1] - x.pos[1]);
                let d = dx.hypot(dz);
                let k = if d > REACH { REACH / d } else { 1.0 };
                Some([x.pos[0] + dx * k, x.pos[1] + dz * k])
            }
            Some((x, _)) if x.held.is_some() => None,
            _ => Some(door_of(w, k, plot).0),
        };
        if w.setups[k].parcel != parcel {
            w.setups[k].parcel = parcel;
            w.items.version = w.items.version.wrapping_add(1);
        }
    }
}

/// Sends resident `j` off to set up `setups[k]`: what they're doing dropped, off to the door.
fn send(w: &mut World, k: usize, j: usize, plot: u32) {
    let s = &w.setups[k];
    let (household, object) = (s.household, s.object);
    let Some((spot, stand, at)) = spot(w, household, object) else {
        // No room for it anywhere at home.
        sent_back(w, k);
        return;
    };
    let (parcel, by_door) = door_of(w, k, plot);
    mark_by(w, household, object, j);
    let s = &mut w.setups[k];
    s.spot = Some(spot);
    s.retry = w.tick + RETRY_MINUTES * crate::TICKS_PER_MINUTE;
    let World {
        sims,
        content,
        objects,
        ..
    } = &mut *w;
    let sim = &mut sims[j];
    if sim.current.is_some() {
        crate::world::end_activity(sim, content, objects);
    }
    sim.follow_up = None;
    sim.queue.push_front(Task {
        kind: TaskKind::SetUp {
            object: object as u32,
            stage: 0,
            door: by_door,
            parcel,
            stand,
            at,
        },
        directed: false,
        routine: false,
    });
}

/// Puts `setups[k]` up at home, at its spot if it still fits there (else the next free one), and
/// says so: set up by resident `by`, or (none) for them, nobody having got to it. No room for it
/// anywhere: it goes back.
fn place(w: &mut World, k: usize, by: Option<usize>) {
    let s = w.setups[k].clone();
    let at = s
        .spot
        .or_else(|| spot(w, s.household, s.object).map(|(at, ..)| at));
    // (Paid for already.)
    let placed = crate::planner::place_wanted(w, s.household, s.object, at, false);
    if placed.is_err() {
        sent_back(w, k);
        return;
    }
    w.setups.remove(k);
    w.items.version = w.items.version.wrapping_add(1);
    let tick = w.tick;
    let asker = s.sim as usize;
    match by {
        Some(by) => w
            .events
            .push_object(tick, EventKind::SetUp, by, s.object, None),
        None => w
            .events
            .push_object(tick, EventKind::SetUpForThem, asker, s.object, None),
    }
    if let Some(a) = w.sims.get_mut(asker) {
        a.planner.recheck = true;
    }
}

/// `setups[k]` can't be set up at home (no room, or the household moved out): back to the shop,
/// the money back.
fn sent_back(w: &mut World, k: usize) {
    let s = w.setups.remove(k);
    w.items.version = w.items.version.wrapping_add(1);
    if let Some(h) = w.households.get_mut(s.household as usize) {
        h.funds += s.paid;
    }
    if w.sims.get(s.sim as usize).is_some_and(|a| a.here()) {
        w.events
            .push_object(w.tick, EventKind::SentBack, s.sim as usize, s.object, None);
    }
}

// ---- The task (run by `world::step_sim`) ----------------------------------------------------

/// Sets off: to the parcel at the door (stage 0), or with it to beside where it goes (stage 1).
pub(crate) fn start(sim: &mut Sim, task: Task, ctx: &Ctx) {
    let TaskKind::SetUp {
        stage, door, stand, ..
    } = task.kind
    else {
        return;
    };
    sim.current = None;
    let (goal, target) = match stage {
        0 => ((door[0].floor() as i32, door[1].floor() as i32), door),
        _ => (stand, [stand.0 as f32 + 0.5, stand.1 as f32 + 0.5]),
    };
    let Some(waypoints) = crate::world::walk_route(ctx, sim, goal, target) else {
        // (No way there: the parcel stays where it was.)
        if stage > 0 {
            sim.held = None;
        }
        return;
    };
    sim.pose = Pose::Stand;
    sim.current = Some(Activity {
        task,
        phase: Phase::Routing { waypoints, next: 0 },
        tags: ctx.content.object_rules.set_up.tags,
    });
}

/// Got there: facing the parcel, picking it up (stage 0); or facing where it goes, the parcel put
/// down, putting it together (stage 1).
pub(crate) fn arrive(sim: &mut Sim) {
    let Some(act) = sim.current.as_mut() else {
        return;
    };
    let TaskKind::SetUp {
        stage, parcel, at, ..
    } = act.task.kind
    else {
        return;
    };
    let face = if stage == 0 { parcel } else { at };
    sim.yaw = (face[0] - sim.pos[0]).atan2(face[1] - sim.pos[1]);
    sim.pose = Pose::Stand;
    act.phase = Phase::Using { elapsed: 0.0 };
    if stage > 0 {
        sim.held = None;
    }
}

/// Picking up or putting together: whether it's done.
pub(crate) fn using(sim: &Sim, content: &Content, elapsed: f32) -> bool {
    let rules = &content.object_rules.set_up;
    match sim.current.as_ref().map(|a| a.task.kind) {
        Some(TaskKind::SetUp { stage: 0, .. }) => elapsed >= rules.fetch_minutes,
        Some(TaskKind::SetUp { .. }) => elapsed >= rules.minutes,
        _ => true,
    }
}

/// Picked up: off with the parcel in the hands (stage 1). Put together: it's placed after the
/// step (`update`).
pub(crate) fn step_done(sim: &mut Sim, ctx: &Ctx, objects: &mut [ObjectInstance]) {
    let Some(task) = sim.current.as_ref().map(|a| a.task) else {
        return;
    };
    let TaskKind::SetUp { object, stage, .. } = task.kind else {
        return;
    };
    if stage == 0 {
        sim.current = None;
        sim.held = ctx.content.object_rules.set_up.parcel.map(Held::of);
        let mut next = task;
        if let TaskKind::SetUp { stage, .. } = &mut next.kind {
            *stage = 1;
        }
        start(sim, next, ctx);
        return;
    }
    crate::world::end_activity(sim, ctx.content, objects);
    sim.set_up = Some(object);
}

/// The parcels to draw (`view::items_json`): each waiting one's place, with an id of its own
/// (from `PARCEL_IDS` up), its item and its household.
pub fn parcels(w: &World) -> Vec<(u32, usize, [f32; 2], u32)> {
    let Some(item) = w.content.object_rules.set_up.parcel else {
        return Vec::new();
    };
    w.setups
        .iter()
        .enumerate()
        .filter_map(|(k, s)| Some((PARCEL_IDS + k as u32, item, s.parcel?, s.household)))
        .collect()
}
