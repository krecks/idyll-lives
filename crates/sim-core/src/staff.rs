//! Vendors and shopkeepers (I-community-09, Q-17): someone serves at the town's carts, stalls and
//! counters.
//!
//! Who they are: staff who come in to work at the town's public places, not residents. Each thing
//! sold from (content `staff`: the ice cream cart, the hot dog stand, a market stall, a shop's
//! counter, a bakery case, a café's coffee machine) is a post on a public lot, with a place behind
//! it where its server stands, on the server's side, and the hours it's open. Posts on one lot whose
//! places are within `service.share` metres of each other, with a straight walk between them,
//! share a server, who steps over to the one a customer is at (the café's counter and its coffee
//! machine). A post with no room behind it (against a wall) has nobody, nor has one on a
//! household's lot (their own cart): there it stays self-serve. A town has at most `MAX_STAFF`
//! servers. Nothing here is saved: who serves where follows from the objects (`refresh`), and
//! whether they're there from the clock.
//!
//! What they sell (content `selfServe` on the interaction) is then a purchase: the customer comes to
//! the front and waits for the server (`hold`; the server takes one customer at a time, first come
//! first served, and one who waits longer than `service.waitMinutes` gives up); while they're
//! served the use runs, the customer ordering, the server fetching it from their side (the
//! freezer's lid, the bakery case's back door), handing it over and taking the money (the renderer's
//! `buy` and `sell` clips, keyed on the use starting), and the customer carries the food off as
//! before (`eat`). Outside the post's hours it's closed: free will doesn't choose it and an order
//! is refused (`closed`); a café's coffee is closed with its machine. Where nobody serves, the use
//! is as it always was, under its `selfServe` label and animation.
//!
//! Why staff and not residents with a vendor career: the town's residents work away (they leave by
//! the town exit for their shift), and the few with a fitting career couldn't keep a dozen posts open
//! all day, so the stalls would stand empty most of the time; staff need no place in `MAX_SIMS`, no
//! needs, no wages from the town's households, nothing in saves, and the balance of careers and
//! money stays as it is. The renderer draws each server from `snapshot` rows after the residents'.

use serde::Deserialize;

use crate::content::{Content, Interaction, ObjectDef, in_hours};
use crate::error::Error;
use crate::world::{Activity, Ctx, ObjectInstance, Phase, Task, TaskKind, WALK_SPEED, World};

/// Most servers in a town (snapshot rows after the residents'). Posts past them stay self-serve.
pub const MAX_STAFF: usize = 32;
/// Last orders (hours): someone who set off for a post while it was open and gets there this long
/// after it closed is still served (the server waits for them), as no shop turns away the one
/// coming through the door at closing time.
pub const LAST_ORDERS: f32 = 0.5;

/// How far (radians) a server turns from the way their post faces to look at the customer they serve.
const FACE_TURN: f32 = 1.0;

/// Where a post's server stands and when it's open (content `staff`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Post {
    /// Metres to its right and front of the middle of its tiles, in its own frame (as `places`):
    /// behind it, a negative front.
    pub at: [f32; 2],
    /// Open from, to (hours of the day; past midnight when `from` is later).
    pub hours: [f32; 2],
}

/// How a sold use is done where nobody serves (content `selfServe`).
#[derive(Debug, Clone, PartialEq)]
pub struct SelfServe {
    pub label: String,
    /// The animation (index into `Content::animations`): as `Interaction::anim`.
    pub anim: Option<usize>,
    /// How long it takes helping oneself (game minutes): the interaction's own `minutes` where
    /// none is given. A sale's five minutes are the ordering, the hand-over and the paying; a
    /// household's own cart or coffee machine is quicker (`Interaction::minutes_where`).
    pub minutes: Option<f32>,
}

/// Content `service`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServiceRules {
    /// Posts whose places are this close (m) share a server.
    pub share: f32,
    /// A customer waits this long (game minutes) for the server, then gives up.
    pub wait_minutes: f32,
    /// What the server is shown doing while they serve (index into `Content::animations`).
    pub anim: Option<usize>,
}

impl Default for ServiceRules {
    fn default() -> Self {
        Self { share: 0.0, wait_minutes: 20.0, anim: None }
    }
}

#[derive(Deserialize)]
pub(crate) struct PostRaw {
    at: [f32; 2],
    hours: [f32; 2],
}

#[derive(Deserialize)]
pub(crate) struct SelfServeRaw {
    label: String,
    #[serde(default)]
    anim: Option<String>,
    #[serde(default)]
    minutes: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServiceRaw {
    #[serde(default)]
    share: f32,
    #[serde(default)]
    wait_minutes: Option<f32>,
    #[serde(default)]
    anim: Option<String>,
}

/// An object's `staff`, checked: a place within a metre and a half of its footprint, and hours.
pub(crate) fn post(raw: &PostRaw, footprint: [u8; 2], id: &str) -> Result<Post, Error> {
    let half = footprint.map(|f| f as f32 / 2.0 + 1.5);
    if (0..2).any(|k| !raw.at[k].is_finite() || raw.at[k].abs() > half[k]) {
        return Err(Error::new(format!("object '{id}': `staff.at` is more than 1.5 m off its footprint")));
    }
    if !raw.hours.iter().all(|h| (0.0..=24.0).contains(h)) || raw.hours[0] == raw.hours[1] {
        return Err(Error::new(format!("object '{id}': `staff.hours` is [from, to], two different hours of the day (0 to 24)")));
    }
    Ok(Post { at: raw.at, hours: raw.hours })
}

/// An interaction's `selfServe`, checked.
pub(crate) fn self_serve(raw: &SelfServeRaw, anim_index: &dyn Fn(&str) -> Option<usize>, ctx: &str) -> Result<SelfServe, Error> {
    let anim = match raw.anim.as_deref() {
        Some(a) => Some(anim_index(a).ok_or_else(|| Error::new(format!("{ctx}: selfServe: unknown anim '{a}' (declare it in \"animations\")")))?),
        None => None,
    };
    if raw.minutes.is_some_and(|m| !(m.is_finite() && m > 0.0)) {
        return Err(Error::new(format!("{ctx}: selfServe: `minutes` must be above 0")));
    }
    Ok(SelfServe { label: raw.label.clone(), anim, minutes: raw.minutes })
}

/// Content `service`, checked.
pub(crate) fn rules(raw: Option<&ServiceRaw>, anim_index: &dyn Fn(&str) -> Option<usize>) -> Result<ServiceRules, Error> {
    let Some(raw) = raw else { return Ok(ServiceRules::default()) };
    let anim = match raw.anim.as_deref() {
        Some(a) => Some(anim_index(a).ok_or_else(|| Error::new(format!("service: unknown anim '{a}' (declare it in \"animations\")")))?),
        None => None,
    };
    let wait_minutes = raw.wait_minutes.unwrap_or(ServiceRules::default().wait_minutes);
    if !(raw.share.is_finite() && raw.share >= 0.0 && wait_minutes.is_finite() && wait_minutes > 0.0) {
        return Err(Error::new("service: `share` at least 0 and `waitMinutes` above 0"));
    }
    Ok(ServiceRules { share: raw.share, wait_minutes, anim })
}

/// Every sold use is at an object someone can serve at.
pub(crate) fn validate(objects: &[ObjectDef]) -> Result<(), Error> {
    for o in objects {
        if o.staff.is_none()
            && let Some(i) = o.interactions.iter().find(|i| i.served.is_some())
        {
            return Err(Error::new(format!("object '{}' interaction '{}': `selfServe` is for something sold, at an object with `staff`", o.id, i.id)));
        }
    }
    Ok(())
}

/// Someone serving at one post or a few.
#[derive(Debug, Clone, PartialEq)]
pub struct Server {
    /// The posts they serve at (object indices), the first the one they stand at when they come in.
    pub posts: Vec<u32>,
    /// The post they're at or stepping over to (index into `posts`).
    pub at: usize,
    pub pos: [f32; 2],
    pub yaw: f32,
    /// Stepping over to another post.
    pub moving: bool,
    /// At work: one of their posts is open, or someone still waits to be served.
    pub present: bool,
    /// The customer they're serving (a resident's id).
    pub serving: Option<u32>,
    /// Those waiting, first come first.
    pub(crate) queue: Vec<u32>,
    /// What they look like (the renderer makes them from it): the first post's stable id.
    pub seed: u32,
}

/// The town's servers (`World::staff`).
#[derive(Debug, Clone, Default)]
pub struct Staff {
    /// The `World::nav_version` they were found for.
    built: Option<u32>,
    pub servers: Vec<Server>,
    /// Server per object index (posts only).
    of: Vec<Option<u16>>,
}

impl Staff {
    /// The server who serves at object `object` (an index), if anyone does.
    pub fn server_of(&self, object: u32) -> Option<usize> {
        self.of.get(object as usize).copied().flatten().map(usize::from)
    }

    /// Whether any of server `s`'s posts is open at `hour`.
    pub fn open(&self, content: &Content, objects: &[ObjectInstance], s: usize, hour: f32) -> bool {
        self.servers[s].posts.iter().any(|&o| post_open(content, &objects[o as usize], hour))
    }
}

/// Whether the post `o` is open at `hour` (its kind's `staff.hours`).
fn post_open(content: &Content, o: &ObjectInstance, hour: f32) -> bool {
    content.objects[o.def].staff.is_some_and(|p| in_hours(p.hours, hour))
}

/// Where the server of post `o` stands, and which way they face (the way it faces: over it, to the
/// customers in front).
pub fn place(content: &Content, o: &ObjectInstance) -> Option<([f32; 2], f32)> {
    let def = &content.objects[o.def];
    let post = def.staff?;
    let (w, d) = o.size(content);
    let c = [o.x as f32 + w as f32 / 2.0 + o.offset[0] as f32 / 100.0, o.z as f32 + d as f32 / 2.0 + o.offset[1] as f32 / 100.0];
    // Moved out with its size, as a slot's place.
    let [x, z] = [0, 1].map(|k| post.at[k] * o.extent[k] as f32 / 100.0 / def.footprint[k].max(1) as f32);
    let a = o.angle();
    Some(([c[0] + a.cos() * x + a.sin() * z, c[1] - a.sin() * x + a.cos() * z], crate::world::wrap_angle(a)))
}

/// Finds the town's posts and their servers again if the objects or the walls changed.
pub(crate) fn refresh(w: &mut World) {
    if w.staff.built == Some(w.nav_version) {
        return;
    }
    let mut staff = plan(w);
    // Someone already at work carries on where they are (an object placed elsewhere on the lot).
    for s in &mut staff.servers {
        if let Some(was) = w.staff.servers.iter().find(|p| p.seed == s.seed && p.present) {
            (s.pos, s.yaw, s.present) = (was.pos, was.yaw, true);
        }
    }
    w.staff = staff;
}

/// The town's posts and their servers as the objects and walls are now (nobody at work yet):
/// what `refresh` keeps, and what views show before the next tick has it.
pub fn plan(w: &World) -> Staff {
    let nav = w.nav();
    let content = &w.content;
    // Posts with room for their server: on a public lot, their place on a free tile of that lot,
    // with no wall between it and the post.
    let mut posts: Vec<(u32, u32, [f32; 2])> = Vec::new();
    for o in &w.objects {
        let Some(plot) = w.object_plot.get(o.id as usize).copied().flatten() else { continue };
        if !w.plots[plot as usize].public {
            continue;
        }
        let Some((at, _)) = place(content, o) else { continue };
        let (x, z) = (at[0].floor() as i32, at[1].floor() as i32);
        if !nav.tile_free(x, z) || w.plot_at(x, z) != Some(plot) {
            continue;
        }
        // (Beside one of its tiles, across an edge with no wall.)
        let beside = o.tiles(content).any(|(tx, tz)| (tx - x).abs() + (tz - z).abs() == 1 && nav.lot.edge_walkable(tx, tz, x, z));
        if beside {
            posts.push((o.id, plot, at));
        }
    }
    // Posts that share a server: on one lot, within `share` of one already in the group, a
    // straight walk apart.
    let share = content.service.share;
    let mut group: Vec<usize> = (0..posts.len()).collect();
    for i in 0..posts.len() {
        for j in 0..i {
            let (a, b) = (posts[i], posts[j]);
            let near = a.1 == b.1 && (a.2[0] - b.2[0]).hypot(a.2[1] - b.2[1]) <= share && nav.line_walkable(a.2, b.2);
            if near && group[i] != group[j] {
                let (from, to) = (group[i].max(group[j]), group[i].min(group[j]));
                group.iter_mut().filter(|g| **g == from).for_each(|g| *g = to);
            }
        }
    }
    let mut servers: Vec<Server> = Vec::new();
    let mut of = vec![None; w.objects.len()];
    for (i, &(o, ..)) in posts.iter().enumerate() {
        let lead = group[i];
        let k = match servers.iter().position(|s| s.posts[0] == posts[lead].0) {
            Some(k) => k,
            None if servers.len() < MAX_STAFF => {
                let first = &w.objects[posts[lead].0 as usize];
                let (pos, yaw) = place(content, first).unwrap_or(([0.0; 2], 0.0));
                servers.push(Server { posts: vec![posts[lead].0], at: 0, pos, yaw, moving: false, present: false, serving: None, queue: Vec::new(), seed: first.stable });
                servers.len() - 1
            }
            None => continue,
        };
        if !servers[k].posts.contains(&o) {
            servers[k].posts.push(o);
        }
        of[o as usize] = Some(k as u16);
    }
    Staff { built: Some(w.nav_version), servers, of }
}

/// The served use a resident is at (object, interaction), standing at the front of a post with a
/// server, if that's what they're doing.
fn at_post(content: &Content, staff: &Staff, objects: &[ObjectInstance], a: &Activity) -> Option<(u32, usize)> {
    let Activity { task: Task { kind: TaskKind::Use { object, interaction }, .. }, phase: Phase::Using { .. }, .. } = a else {
        return None;
    };
    let o = objects.get(*object as usize)?;
    content.objects[o.def].interactions.get(*interaction)?.served.as_ref()?;
    staff.server_of(*object).map(|k| (*object, k))
}

/// The server of the post a resident is on their way to buy at, if that's what they're doing.
fn heading_to(content: &Content, staff: &Staff, objects: &[ObjectInstance], a: &Activity) -> Option<usize> {
    let Activity { task: Task { kind: TaskKind::Use { object, interaction }, .. }, phase: Phase::Routing { .. }, .. } = a else {
        return None;
    };
    let o = objects.get(*object as usize)?;
    content.objects[o.def].interactions.get(*interaction)?.served.as_ref()?;
    staff.server_of(*object)
}

/// Whether any of `posts` is open at `hour`.
fn staff_open(content: &Content, objects: &[ObjectInstance], posts: &[u32], hour: f32) -> bool {
    posts.iter().any(|&o| post_open(content, &objects[o as usize], hour))
}

/// As a tick starts, before residents step: who's at work, who they serve next, and their steps
/// over to them. Sets `Sim::being_served` for the one each serves once they're at that post.
pub(crate) fn update(w: &mut World) {
    refresh(w);
    let hour = crate::clock::hour(w.tick);
    let World { content, objects, sims, staff, .. } = w;
    // Those at a post's front, waiting or being served, by server; and servers someone is on their
    // way to (last orders).
    let mut waiting: Vec<(usize, u32)> = Vec::new();
    let mut coming = vec![false; staff.servers.len()];
    for s in sims.iter_mut() {
        s.being_served = false;
        let Some(a) = s.current.as_ref().filter(|_| s.here()) else { continue };
        if let Some((_, k)) = at_post(content, staff, objects, a) {
            waiting.push((k, s.id));
        } else if let Some(k) = heading_to(content, staff, objects, a) {
            coming[k] = true;
        }
    }
    let late = (hour - LAST_ORDERS).rem_euclid(24.0);
    let open: Vec<bool> = (0..staff.servers.len()).map(|k| staff.open(content, objects, k, hour)).collect();
    for (k, server) in staff.servers.iter_mut().enumerate() {
        let here = |c: u32| waiting.contains(&(k, c));
        server.serving = server.serving.filter(|&c| here(c));
        server.queue.retain(|&c| here(c) && server.serving != Some(c));
        for &(_, c) in waiting.iter().filter(|&&(s, c)| s == k && server.serving != Some(c)) {
            if !server.queue.contains(&c) {
                server.queue.push(c);
            }
        }
        let present = open[k] || server.serving.is_some() || !server.queue.is_empty() || (coming[k] && staff_open(content, objects, &server.posts, late));
        if !present {
            // Gone home: back at the first post when they come in again.
            if server.present {
                let first = &objects[server.posts[0] as usize];
                if let Some((pos, yaw)) = place(content, first) {
                    (server.pos, server.yaw) = (pos, yaw);
                }
                (server.at, server.moving) = (0, false);
            }
            server.present = false;
            continue;
        }
        server.present = true;
        if server.serving.is_none() && !server.queue.is_empty() {
            server.serving = Some(server.queue.remove(0));
        }
        // Over to the post their customer is at.
        if let Some(c) = server.serving
            && let Some(Activity { task: Task { kind: TaskKind::Use { object, .. }, .. }, .. }) = sims[c as usize].current.as_ref()
            && let Some(i) = server.posts.iter().position(|p| p == object)
        {
            server.at = i;
        }
        let Some((to, yaw)) = place(content, &objects[server.posts[server.at] as usize]) else { continue };
        let (dx, dz) = (to[0] - server.pos[0], to[1] - server.pos[1]);
        let dist = dx.hypot(dz);
        if dist <= WALK_SPEED {
            (server.pos, server.yaw, server.moving) = (to, yaw, false);
        } else {
            server.pos = [server.pos[0] + dx / dist * WALK_SPEED, server.pos[1] + dz / dist * WALK_SPEED];
            (server.yaw, server.moving) = (dx.atan2(dz), true);
        }
        if let Some(c) = server.serving.filter(|_| !server.moving) {
            sims[c as usize].being_served = true;
            // Turned to them, over the post (the one at a two-wide stall's end), not round past its side.
            let to = sims[c as usize].pos;
            let at = (to[0] - server.pos[0]).atan2(to[1] - server.pos[1]);
            let off = crate::world::wrap_angle(at - yaw);
            server.yaw = crate::world::wrap_angle(yaw + off.clamp(-FACE_TURN, FACE_TURN));
        }
    }
}

/// What a customer at a post does this step (`world::progress`).
pub(crate) enum Hold {
    /// Being served: the use runs.
    Serve,
    /// Waiting for the server.
    Wait,
    /// Waited too long, or nobody's there any more: given up.
    GiveUp,
}

/// For a resident using `inter` at `object`: whether they're served, wait or give up (None: nobody
/// serves there, the use runs as any other). `waited`: ticks they've waited so far.
pub(crate) fn hold(ctx: &Ctx, served: bool, waited: &mut u32, object: u32, inter: &Interaction, ticks: u32) -> Option<Hold> {
    inter.served.as_ref()?;
    let k = ctx.staff.server_of(object)?;
    if served {
        *waited = 0;
        return Some(Hold::Serve);
    }
    *waited += ticks;
    let limit = (ctx.content.service.wait_minutes * crate::TICKS_PER_MINUTE as f32) as u32;
    Some(if *waited > limit || !ctx.staff.servers[k].present { Hold::GiveUp } else { Hold::Wait })
}

/// The closed post that keeps `inter` at `object` from being started now, if any: the object
/// itself, where it's sold outside its post's hours; or, for what it needs fetched (a café table's
/// coffee), the nearest of the posts it's sold at, all of them closed.
pub(crate) fn closed(content: &Content, staff: &Staff, objects: &[ObjectInstance], on_plot: &[u32], object: u32, inter: &Interaction, hour: f32) -> Option<u32> {
    if inter.served.is_some() && staff.server_of(object).is_some() {
        return (!post_open(content, &objects[object as usize], hour)).then_some(object);
    }
    let fetch = inter.fetch.as_deref()?;
    // (Every source closed: a post someone serves at, out of its hours; one nobody serves at is
    // always open.)
    let mut first = None;
    for &o in on_plot.iter().filter(|&&o| o != object) {
        let obj = &objects[o as usize];
        let def = &content.objects[obj.def];
        let Some(i) = def.interactions.iter().find(|i| i.id == fetch) else { continue };
        if def.provides & inter.requires != inter.requires || obj.broken() {
            continue;
        }
        if i.served.is_none() || staff.server_of(o).is_none() || post_open(content, obj, hour) {
            return None;
        }
        first.get_or_insert(o);
    }
    first
}

/// When post `object` opens next, as the hour of the day (for the player's refused order).
pub(crate) fn opens(content: &Content, objects: &[ObjectInstance], object: u32) -> Option<f32> {
    content.objects[objects.get(object as usize)?.def].staff.map(|p| p.hours[0])
}

/// The item a customer is handed while served at a post (the food they then eat), as the snapshot
/// shows it in their hands and the server's: the serving `eat` gives (`cooking::eat_now`).
pub(crate) fn handed(content: &Content, inter: &Interaction) -> Option<usize> {
    let r = &content.recipes[inter.eats?];
    Some(content.items[r.item].serving.unwrap_or(r.item))
}
