//! Shared test fixtures: the shipped content and a whole town built from the shipped house
//! layouts, laid out like the web's `game/town.ts` (two rows of plots on a street, the north
//! row turned to face it, a park in the middle) but deterministic.

#![allow(dead_code)]

use serde_json::{Value, json};
use sim_core::social::FORMER;
use sim_core::world::{Phase, Sim, TaskKind};
use sim_core::{MINUTES_PER_TICK, World, clock};

fn content_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/content")
}

fn read(name: &str) -> String {
    std::fs::read_to_string(content_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// base.json plus every file in its `"include"` list (careers, content packs), merged the
/// way the web loader does.
pub fn content() -> String {
    let base = read("base.json");
    let includes: Vec<String> =
        serde_json::from_value(serde_json::from_str::<Value>(&base).unwrap()["include"].clone())
            .expect("base.json lists its includes");
    let parts: Vec<String> = includes.iter().map(|n| read(n)).collect();
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
    sim_core::merge_content(&base, &parts).unwrap_or_else(|e| panic!("{e}"))
}

/// Looks at a lot like the game does: it and the player's home are simulated in full detail,
/// the rest of the town in less (`sim_core::lod`).
pub fn look_at(w: &mut World, plot: u32) {
    let p = &w.plots[plot as usize];
    w.set_view(Some([p.x, p.z, p.x + p.w, p.z + p.d]));
}

/// With `LOD=1` in the environment, looks at the player's home (to compare runs with lower
/// detail for the rest of the town against full detail).
pub fn lod_from_env(w: &mut World) {
    let home = w.households.iter().find(|h| h.player).and_then(|h| h.plot);
    if let Some(home) = home.filter(|_| std::env::var("LOD").is_ok_and(|v| v == "1")) {
        look_at(w, home);
    }
}

/// Buys `def` for household 0 at `x`, `z`, facing the default way: its stable id.
pub fn buy(w: &mut World, def: &str, x: i32, z: i32) -> u32 {
    w.apply(sim_core::Command::Buy { household: 0, object: def.into(), at: Some([x, z, 0]), colors: Default::default(), turn: None, offset: None, size: None, tall: None })
        .unwrap_or_else(|e| panic!("{def}: {e}"));
    w.objects.last().unwrap().stable
}

/// Everything that must hold at any moment.
pub fn check(w: &World) {
    // Things for outside (content `outdoors`) stand outside.
    for o in &w.objects {
        let def = &w.content.objects[o.def];
        assert!(
            !def.outdoors || w.lot.room_at(o.x, o.z) == sim_core::lot::OUTDOORS,
            "the {} at {},{} is indoors",
            def.name,
            o.x,
            o.z
        );
    }
    let n = w.sims.len();
    let content_needs = w.content.needs.len();
    let max_level = w.content.skill_rules.max_level;
    for (i, s) in w.sims.iter().enumerate() {
        assert_eq!(s.id as usize, i, "Sim ids are their index");
        assert!(
            s.pos.iter().all(|p| p.is_finite()),
            "{} is nowhere: {:?}",
            s.name,
            s.pos
        );
        let (x, z) = s.tile();
        assert!(w.lot.in_bounds(x, z), "{} left the town", s.name);
        for (k, v) in s.needs[..content_needs].iter().enumerate() {
            assert!((0.0..=1.0).contains(v), "{}'s need {k} is {v}", s.name);
        }
        // Every number about them is a real one in its range (NaN is in none).
        assert!(s.yaw.is_finite() && s.yaw.abs() <= std::f32::consts::TAU, "{} faces {}", s.name, s.yaw);
        assert!((0.0..=150.0).contains(&s.age), "{} is {} years old", s.name, s.age);
        for (k, v) in s.skills[..w.content.skills.len()].iter().enumerate() {
            assert!((0.0..=max_level).contains(v), "{}'s skill {k} is {v}", s.name);
        }
        let mood = s.mood(&w.content);
        assert!((0.0..=1.0).contains(&mood), "{}'s mood is {mood}", s.name);
        assert!((0.0..=100.0).contains(&s.grade), "{}'s grade is {}", s.name, s.grade);
        assert!((s.household as usize) < w.households.len());
        if let Some(t) = s.engaged_with {
            assert!((t as usize) < n && t as usize != i);
        }
        if let Some(act) = s.current() {
            task_refs(w, i, &act.task.kind, "is doing");
            if let TaskKind::Use { object, .. } = act.task.kind {
                let o = &w.objects[object as usize];
                assert!(
                    o.users().any(|u| u as usize == i),
                    "{} uses object {object} without holding a slot",
                    s.name
                );
            }
            match &act.phase {
                Phase::Routing { waypoints, next } => {
                    assert!(*next <= waypoints.len(), "{} walks past the end of their route", s.name);
                    assert!(waypoints.iter().flatten().all(|v| v.is_finite()), "{} walks to nowhere: {waypoints:?}", s.name);
                }
                Phase::Using { elapsed } | Phase::Conversing { elapsed, .. } => {
                    assert!(elapsed.is_finite() && *elapsed >= 0.0, "{} has been at it for {elapsed} minutes", s.name)
                }
                _ => {}
            }
        }
        for t in s.queue() {
            task_refs(w, i, &t.kind, "has queued");
        }
        // (A follow-up is started when they have nothing in hand: that's when it has to name
        // what's there. Reading it is slow, so only then, and once a game minute for tests that
        // check every tick.)
        if s.current().is_none()
            && s.here()
            && w.tick.is_multiple_of(20)
            && let Some(kind) = follow_up(s)
        {
            follow_up_refs(w, i, &kind);
        }
        if let Some(h) = &s.held {
            assert!(h.def < w.content.items.len(), "{} holds an unknown item", s.name);
            assert!(h.quality.is_finite() && (0.0..=10.0).contains(&h.quality), "{} holds something of quality {}", s.name, h.quality);
        }
        if let Some(v) = s.visiting {
            assert!((v.plot as usize) < w.plots.len());
        }
        if let Some(job) = &s.job {
            let career = &w.content.careers[job.career];
            assert!(job.level < career.levels.len());
            assert!((0.0..=100.0).contains(&job.performance));
            assert!((0.0..=1.0).contains(&job.satisfaction));
        }
    }
    // Objects are their index in the list, of a known kind, worn 0..1 and of a quality there is.
    let max_quality = w.content.object_rules.max_quality;
    for (k, o) in w.objects.iter().enumerate() {
        assert_eq!(o.id as usize, k, "object ids are their index");
        assert!(o.def < w.content.objects.len(), "object {k} is of an unknown kind");
        assert!((0.0..=1.0).contains(&o.wear), "object {k} is worn {}", o.wear);
        assert!(o.quality <= max_quality, "object {k} has quality {}", o.quality);
        for u in o.users() {
            assert!((u as usize) < n, "object {k} is held by unknown resident {u}");
        }
    }
    for (t, d) in w.dirt.iter().enumerate() {
        assert!((0.0..=1.0).contains(d), "tile {t} has dirt {d}");
    }
    // Relationships: friendship and romance within ±100, chemistry within ±1.
    let m = w.relationships.len();
    for a in 0..m {
        for b in 0..m {
            let r = w.relationships.get(a, b);
            let c = w.relationships.chemistry(a, b);
            assert!(
                r.friendship.abs() <= 100.0 && r.romance.abs() <= 100.0 && c.abs() <= 1.0,
                "{a} and {b}: friendship {}, romance {}, chemistry {c}",
                r.friendship,
                r.romance
            );
        }
    }
    // Households: money that makes sense, homes on the town's plots (one each), parents-to-be
    // who are residents.
    for (h, hh) in w.households.iter().enumerate() {
        assert!(hh.funds.abs() < 1_000_000_000, "{} has {} in the bank", hh.name, hh.funds);
        if let Some(p) = hh.plot {
            assert!((p as usize) < w.plots.len(), "{} lives on unknown plot {p}", hh.name);
            assert!(!w.plots[p as usize].public, "{} lives in the park", hh.name);
            assert!(!w.households[..h].iter().any(|o| o.plot == Some(p)), "two households live on plot {p}");
        }
        if let Some(e) = hh.expecting {
            let [a, b] = e.parents.map(|p| p as usize);
            assert!(a < n && b < n && a != b, "{} expects a baby from {:?}", hh.name, e.parents);
        }
    }
    // Every object slot holder is someone who is using or heading to that object (or talking
    // from its seat, what they sat down to do held: `paused`).
    for o in &w.objects {
        for u in o.users() {
            let s = &w.sims[u as usize];
            let doing = s.paused().or(s.current());
            let ok = doing.is_some_and(|a| match a.task.kind {
                TaskKind::Use { object, .. } | TaskKind::Repair { object } => object == o.id,
                TaskKind::Cook { station, .. } | TaskKind::Dishes { station, .. } => station == Some(o.id),
                TaskKind::Eat { seat, .. } | TaskKind::Phone { seat, .. } => seat == Some(o.id),
                _ => false,
            }) || (s.use_seat == Some(o.id)
                && doing
                    .is_some_and(|a| matches!(a.task.kind, TaskKind::Use { .. })));
            assert!(
                ok,
                "object {} is held by {} who isn't using it",
                o.id, s.name
            );
        }
    }
    // Everything on a top stands on one, which comes before it in the list.
    for o in &w.objects {
        if w.content.objects[o.def].layer == sim_core::content::Layer::Surface {
            let host = sim_core::surface::host_of(&w.content, &w.objects, o);
            assert!(host.is_some_and(|h| h < o.id), "object {} stands on nothing before it ({host:?})", o.id);
        }
    }
    // Only cooks, people eating and people washing up carry things (the rest put them down).
    for s in &w.sims {
        if s.held.is_some() {
            let chain = |k: &TaskKind| matches!(k, TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. });
            assert!(
                match s.current() {
                    Some(a) => chain(&a.task.kind),
                    None => s.queue().next().is_some_and(|t| matches!(t.kind, TaskKind::Cook { .. })),
                },
                "{} carries a {} while doing {:?}, then {:?}",
                s.name,
                s.held.map_or("?", |h| w.content.items[h.def].id.as_str()),
                s.current().map(|a| (a.task.kind, &a.phase)),
                s.queue().map(|t| t.kind).collect::<Vec<_>>()
            );
        }
    }
    // Items are in id order, belong to a household, and stand somewhere on the lot.
    for pair in w.items.list.windows(2) {
        assert!(pair[0].id < pair[1].id, "items out of order");
    }
    for it in &w.items.list {
        assert!((it.household as usize) < w.households.len());
        let (x, z) = it.at.tile();
        assert!(w.lot.in_bounds(x, z), "item {} is off the lot", it.id);
        assert!(it.id < w.items.next_id);
        assert!(it.def < w.content.items.len(), "item {} is of an unknown kind", it.id);
        assert!(it.quality.is_finite() && (0.0..=10.0).contains(&it.quality), "item {} has quality {}", it.id, it.quality);
    }
    // Partners are mutual and one at a time.
    for a in 0..n {
        if let Some(b) = w.relationships.partner_of(a) {
            assert_eq!(
                w.relationships.partner_of(b),
                Some(a),
                "partners aren't mutual"
            );
        }
    }
    let mut last = 0;
    for e in w.events.iter() {
        assert!(e.id > last, "event ids increase");
        last = e.id;
        // A resident slot, or a former resident whose name is kept.
        let known = |r: u32| if r & FORMER != 0 { ((r & !FORMER) as usize) < w.former.len() } else { (r as usize) < n };
        assert!(known(e.a) && known(e.b) && e.c.is_none_or(known), "{e:?} names someone unknown");
    }
}

/// What resident `i`'s task names exists: objects (and the interaction asked for), residents,
/// plots, recipes, socials, accidents and phone actions.
fn task_refs(w: &World, i: usize, kind: &TaskKind, doing: &str) {
    let (c, n) = (&w.content, w.sims.len());
    let name = &w.sims[i].name;
    let object = |o: Option<u32>| o.is_none_or(|o| (o as usize) < w.objects.len());
    let ok = match *kind {
        TaskKind::Use { object: o, interaction } => {
            w.objects.get(o as usize).is_some_and(|o| interaction < c.objects[o.def].interactions.len())
        }
        TaskKind::MoveTo { x, z } => x.is_finite() && z.is_finite() && w.lot.in_bounds(x.floor() as i32, z.floor() as i32),
        TaskKind::Social { target, social } => (target as usize) < n && target as usize != i && social < c.socials.len(),
        TaskKind::Join { leader } => (leader as usize) < n && leader as usize != i,
        TaskKind::Work | TaskKind::GoHome => true,
        TaskKind::Visit { plot } => (plot as usize) < w.plots.len(),
        TaskKind::Answer { guest } => (guest as usize) < n,
        TaskKind::Clean { x, z } => w.lot.in_bounds(x, z),
        TaskKind::Repair { object: o } => object(Some(o)),
        TaskKind::Spot { accident } => accident < c.accidents.len(),
        TaskKind::Cook { recipe, station, origin, .. } => recipe < c.recipes.len() && object(station) && object(origin),
        TaskKind::Eat { seat, .. } => object(seat),
        TaskKind::Dishes { station, .. } => object(station),
        TaskKind::Phone { action, contact, seat, .. } => {
            action < c.phone.actions.len() && contact.is_none_or(|t| (t as usize) < n) && object(seat)
        }
    };
    assert!(ok, "{name} {doing} {kind:?}, which names something that isn't there ({} objects)", w.objects.len());
}

/// What a resident does next before choosing anything else (washing hands after the toilet),
/// as its `TaskKind` debug text. The field isn't public, so it's read from the resident's debug
/// text: a follow-up isn't remapped when objects are sold or moved (R2-01), and a stale one
/// crashes the game when it's started.
pub fn follow_up(s: &Sim) -> Option<String> {
    const FIELD: &str = "follow_up: Some(Task { kind: ";
    let text = format!("{s:?}");
    let rest = &text[text.find(FIELD)? + FIELD.len()..];
    Some(rest[..rest.find(", directed: ")?].to_string())
}

/// A number field (`object: 3`, or `seat: Some(3)`) in a task's debug text.
pub fn debug_field(kind: &str, field: &str) -> Option<u32> {
    let at = kind.find(&format!("{field}: "))? + field.len() + 2;
    let rest = kind[at..].strip_prefix("Some(").unwrap_or(&kind[at..]);
    rest[..rest.find(|c: char| !c.is_ascii_digit())?].parse().ok()
}

/// A follow-up names an object there is, and an interaction it has.
fn follow_up_refs(w: &World, i: usize, kind: &str) {
    let name = &w.sims[i].name;
    for field in ["object", "station", "origin", "seat"] {
        if let Some(o) = debug_field(kind, field) {
            assert!((o as usize) < w.objects.len(), "{name}'s next thing is {kind}, an object that isn't there");
        }
    }
    if let (Some(o), Some(k)) = (debug_field(kind, "object"), debug_field(kind, "interaction")) {
        let def = &w.content.objects[w.objects[o as usize].def];
        assert!(
            (k as usize) < def.interactions.len(),
            "{name}'s next thing is {kind}, but object {o} is a {} with {} interactions",
            def.id,
            def.interactions.len()
        );
    }
}

/// A resident stuck in one state too long, as `Watch` saw it.
#[derive(Debug, Clone)]
pub struct Stuck {
    pub sim: u32,
    pub name: String,
    /// The state, with what it's about (`Busy Use readingNook.curlUp`).
    pub state: String,
    /// The tick it started, and the tick it was seen to have lasted too long.
    pub since: u64,
    pub tick: u64,
}

impl std::fmt::Display for Stuck {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let at = |t: u64| format!("day {} {:05.2}h", clock::day(t), clock::hour(t));
        write!(f, "{} (#{}) {} since {}, still at {} (tick {})", self.name, self.sim, self.state, at(self.since), at(self.tick), self.tick)
    }
}

/// What a resident is doing, for `Watch`: the state, and what it's with (the same state with
/// another thing is a new stretch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// In bed (not a crib).
    Asleep(Doing),
    /// Using something (not a bed or a crib), or talking.
    Busy(Doing),
    Routing(Doing),
    /// Waiting for someone to talk to, for a busy station, or at someone's door.
    Waiting(Doing),
    Away,
    /// Being talked to by this resident.
    Engaged(u32),
    /// Being talked to at home by this guest.
    Hosting(u32),
    Idle,
    /// A baby in the crib (they lie there for days): never stuck.
    Crib,
}

/// A task without how far along it is: its kind and the thing or person it's with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Doing(u8, u32, u32);

impl State {
    /// Game hours in a row after which it's a resident stuck, not a long stretch.
    fn limit(self) -> Option<f32> {
        match self {
            State::Asleep(_) => Some(13.0),
            State::Busy(_) => Some(6.0),
            State::Routing(_) => Some(4.0),
            State::Waiting(_) => Some(1.0),
            State::Away => Some(16.0),
            State::Engaged(_) => Some(2.0),
            // A guest keeps the hosts company (content `visits.company`): one chat after another
            // for most of the visit (`visits.hours`, 2.5 h; twice that when the player sent them).
            State::Hosting(_) => Some(6.0),
            State::Idle => Some(6.0),
            State::Crib => None,
        }
    }
}

fn doing(kind: &TaskKind) -> Doing {
    match *kind {
        TaskKind::Use { object, interaction } => Doing(0, object, interaction as u32),
        TaskKind::MoveTo { x, z } => Doing(1, x.to_bits(), z.to_bits()),
        TaskKind::Social { target, social } => Doing(2, target, social as u32),
        TaskKind::Join { leader } => Doing(3, leader, 0),
        TaskKind::Work => Doing(4, 0, 0),
        TaskKind::Visit { plot } => Doing(5, plot, 0),
        TaskKind::GoHome => Doing(6, 0, 0),
        TaskKind::Answer { guest } => Doing(7, guest, 0),
        TaskKind::Clean { x, z } => Doing(8, x as u32, z as u32),
        TaskKind::Repair { object } => Doing(9, object, 0),
        TaskKind::Spot { accident } => Doing(10, accident as u32, 0),
        TaskKind::Cook { recipe, step, .. } => Doing(11, recipe as u32, step as u32),
        TaskKind::Eat { stage, .. } => Doing(12, stage as u32, 0),
        TaskKind::Dishes { stage, .. } => Doing(13, stage as u32, 0),
        TaskKind::Phone { action, .. } => Doing(14, action as u32, 0),
    }
}

fn state(w: &World, s: &Sim) -> State {
    if s.away_until.is_some() {
        return State::Away;
    }
    if let Some(t) = s.engaged_with {
        let home = w.households[s.household as usize].plot;
        let guest = w.sims.get(t as usize).and_then(|g| g.visiting).is_some_and(|v| Some(v.plot) == home);
        return if guest { State::Hosting(t) } else { State::Engaged(t) };
    }
    let Some(act) = s.current() else { return State::Idle };
    let d = doing(&act.task.kind);
    match act.phase {
        Phase::Using { .. } => {
            let crib = match act.task.kind {
                TaskKind::Use { object, interaction } => w
                    .objects
                    .get(object as usize)
                    .and_then(|o| w.content.objects[o.def].interactions.get(interaction))
                    .is_some_and(|i| i.baby),
                _ => false,
            };
            if crib {
                State::Crib
            } else if s.asleep(&w.content) {
                State::Asleep(d)
            } else {
                State::Busy(d)
            }
        }
        Phase::Conversing { .. } => State::Busy(d),
        Phase::Routing { .. } => State::Routing(d),
        Phase::Calling { .. } | Phase::Waiting { .. } | Phase::Queued { .. } | Phase::Knocking { .. } => State::Waiting(d),
    }
}

fn describe(w: &World, st: State) -> String {
    let thing = |d: Doing| match d {
        Doing(0, o, i) => w.objects.get(o as usize).map_or(format!("Use object {o}"), |obj| {
            let def = &w.content.objects[obj.def];
            format!("Use {}.{}", def.id, def.interactions.get(i as usize).map_or("?", |i| i.id.as_str()))
        }),
        Doing(2, t, s) => format!("Social {} with {}", w.content.socials.get(s as usize).map_or("?", |s| s.id.as_str()), w.sims.get(t as usize).map_or("?", |s| s.name.as_str())),
        Doing(10, a, _) => format!("Spot {}", w.content.accidents.get(a as usize).map_or("?", |a| a.id.as_str())),
        Doing(k, a, b) => {
            let kinds = ["Use", "MoveTo", "Social", "Join", "Work", "Visit", "GoHome", "Answer", "Clean", "Repair", "Spot", "Cook", "Eat", "Dishes", "Phone"];
            format!("{} {a} {b}", kinds[k as usize])
        }
    };
    match st {
        State::Asleep(d) => format!("Asleep: {}", thing(d)),
        State::Busy(d) => format!("Busy {}", thing(d)),
        State::Routing(d) => format!("Routing {}", thing(d)),
        State::Waiting(d) => format!("Waiting {}", thing(d)),
        State::Engaged(t) => format!("Engaged by {}", w.sims.get(t as usize).map_or("?", |s| s.name.as_str())),
        State::Hosting(t) => format!("Engaged by guest {}", w.sims.get(t as usize).map_or("?", |s| s.name.as_str())),
        st => format!("{st:?}"),
    }
}

/// Watches for residents stuck in one state: asleep for 13 hours (a baby in the crib is
/// exempt), using something or talking for 6, walking for 4, waiting, queued or knocking for
/// 1, away for 16, being talked to for 2 (by a guest at home: 6), or idle for 6. Call `sample`
/// after every tick (or step); it looks every `Watch::EVERY` ticks. Each stretch is reported
/// once, in `stuck`.
///
/// With `WATCH=<name>` (or `<name>@<day>`) in the environment it prints what that resident
/// does (from that day), one line per change, to see the story behind a report.
#[derive(Default)]
pub struct Watch {
    /// Per resident slot: their stretch in one state as of the last look.
    last: Vec<Option<Stretch>>,
    next: u64,
    pub stuck: Vec<Stuck>,
    trace: Option<(String, u32)>,
    /// Hours of walking that count as stuck (`State::limit` unless set).
    routing_hours: Option<f32>,
}

/// A resident's stretch in one state: since when, who it is (a slot can be someone new), how
/// far they've walked in it (tiles, between looks) and where they were at the last look, and
/// whether it was reported.
#[derive(Clone)]
struct Stretch {
    state: State,
    since: u64,
    name: String,
    walked: f32,
    at: [f32; 2],
    reported: bool,
}

impl Watch {
    pub const EVERY: u64 = 20;

    pub fn new() -> Self {
        let trace = std::env::var("WATCH").ok().map(|v| match v.split_once('@') {
            Some((name, day)) => (name.to_string(), day.parse().unwrap_or(1)),
            None => (v, 1),
        });
        Watch { trace, ..Self::default() }
    }

    /// Walking for longer than `hours` counts as stuck (instead of 4: a town much bigger than
    /// the game's has longer walks).
    pub fn routing_hours(mut self, hours: f32) -> Self {
        self.routing_hours = Some(hours);
        self
    }

    pub fn sample(&mut self, w: &World) {
        if w.tick < self.next {
            return;
        }
        self.next = (w.tick / Self::EVERY + 1) * Self::EVERY;
        self.last.resize(w.sims.len(), None);
        for (i, s) in w.sims.iter().enumerate() {
            if !s.here() {
                self.last[i] = None;
                continue;
            }
            let now = state(w, s);
            if let Some((name, from)) = &self.trace
                && *name == s.name
                && clock::day(w.tick) >= *from
                && self.last[i].as_ref().is_none_or(|st| st.state != now)
            {
                let route = match s.current().map(|a| &a.phase) {
                    Some(Phase::Routing { waypoints, next }) => format!(", route {next}/{} to {:?}", waypoints.len(), waypoints.last()),
                    _ => String::new(),
                };
                let home = w.households[s.household as usize].plot;
                eprintln!("day {} {:05.2}h {name} (home plot {home:?}) at {:.1},{:.1}: {}{route}", clock::day(w.tick), clock::hour(w.tick), s.pos[0], s.pos[1], describe(w, now));
            }
            match &mut self.last[i] {
                Some(st) if st.state == now && st.name == s.name => {
                    st.walked += (s.pos[0] - st.at[0]).hypot(s.pos[1] - st.at[1]);
                    st.at = s.pos;
                    let hours = (w.tick - st.since) as f32 * MINUTES_PER_TICK / 60.0;
                    let limit = match now {
                        State::Routing(_) => self.routing_hours.or(now.limit()),
                        _ => now.limit(),
                    };
                    if !st.reported && limit.is_some_and(|l| hours > l) {
                        st.reported = true;
                        let state = format!("{} (walked {:.0} m)", describe(w, now), st.walked);
                        self.stuck.push(Stuck { sim: i as u32, name: s.name.clone(), state, since: st.since, tick: w.tick });
                    }
                }
                entry => *entry = Some(Stretch { state: now, since: w.tick, name: s.name.clone(), walked: 0.0, at: s.pos, reported: false }),
            }
        }
    }

    /// Panics with everyone found stuck.
    pub fn assert_quiet(&self) {
        let list: Vec<String> = self.stuck.iter().map(|s| s.to_string()).collect();
        assert!(list.is_empty(), "{} residents stuck:\n{}", list.len(), list.join("\n"));
    }
}

pub const TICKS_PER_HOUR: u64 = (60.0 / MINUTES_PER_TICK) as u64;

/// Keeps an eye on a long run: call `after` after every tick (or step). Every game hour it
/// checks the world (`check`, which panics), and notes in `problems` anyone who didn't age
/// exactly once at every midnight since the last look (or a newborn older than 0), and a town
/// outside `population` residents; every `Watch::EVERY` ticks it watches for residents stuck
/// and collects the story (the world's log keeps only the latest). `assert_ok` at the end.
pub struct Long {
    pub watch: Watch,
    /// Every story event of the run, by kind.
    pub story: std::collections::BTreeMap<String, usize>,
    /// Residents here: the fewest and the most seen.
    pub fewest: usize,
    pub most: usize,
    /// What went wrong besides `check` and `watch` (the first few of each kind).
    pub problems: Vec<String>,
    population: std::ops::RangeInclusive<usize>,
    seen: u64,
    hour: u64,
    /// At the last hourly look: the day, and everyone's (name, age) by slot.
    day: u32,
    ages: Vec<Option<(String, f32)>>,
    /// Babies born since the last look (slot, name).
    born: Vec<(usize, String)>,
    noted: std::collections::BTreeMap<&'static str, usize>,
}

impl Long {
    pub fn new(w: &World, population: std::ops::RangeInclusive<usize>) -> Self {
        let mut long = Long {
            watch: Watch::new(),
            story: Default::default(),
            fewest: usize::MAX,
            most: 0,
            problems: Vec::new(),
            population,
            seen: w.events.last_id(),
            hour: w.tick / TICKS_PER_HOUR,
            day: clock::day(w.tick),
            ages: Vec::new(),
            born: Vec::new(),
            noted: Default::default(),
        };
        long.look(w);
        long
    }

    pub fn after(&mut self, w: &World) {
        if w.tick < self.watch.next {
            return;
        }
        self.watch.sample(w);
        if w.events.last_id() > self.seen {
            let new: Vec<_> = w.events.iter().rev().take_while(|e| e.id > self.seen).collect();
            // (Ids can skip one an undo took back; the log only forgets the oldest when it's full.)
            let full = w.events.iter().count() >= sim_core::social::EVENT_LOG_CAPACITY;
            let oldest = w.events.iter().next().map_or(0, |e| e.id);
            assert!(!full || oldest <= self.seen + 1, "the story log dropped events before they were counted");
            for e in new {
                *self.story.entry(format!("{:?}", e.kind)).or_default() += 1;
                if e.kind == sim_core::social::EventKind::Born {
                    self.born.push((e.a as usize, w.sims[e.a as usize].name.clone()));
                }
            }
            self.seen = w.events.last_id();
        }
        if w.tick / TICKS_PER_HOUR > self.hour {
            self.hour = w.tick / TICKS_PER_HOUR;
            self.look(w);
        }
    }

    /// Notes a problem of `kind` (the first 10 of each).
    fn note(&mut self, kind: &'static str, what: String) {
        let n = self.noted.entry(kind).or_default();
        *n += 1;
        if *n <= 10 {
            self.problems.push(format!("{kind}: {what}"));
        }
    }

    fn look(&mut self, w: &World) {
        check(w);
        let day = clock::day(w.tick);
        let when = format!("day {day} {:05.2}h", clock::hour(w.tick));
        let year = w.lifespan.days_per_year(w.content.life.days_per_year).map(|d| 1.0 / d);
        let midnights = day - self.day;
        for (i, s) in w.sims.iter().enumerate() {
            let Some(Some((name, age))) = self.ages.get(i) else { continue };
            if !s.here() || s.name != *name {
                continue;
            }
            // (Added up the way the life cycle does, so it's exact.)
            let mut expected = *age;
            for _ in 0..midnights {
                expected += year.unwrap_or(0.0);
            }
            if s.age != expected {
                let what = format!("{} was {age} on day {}, and is {} at {when} ({midnights} midnight(s) later)", s.name, self.day, s.age);
                self.note("ageing", what);
            }
        }
        for (i, name) in std::mem::take(&mut self.born) {
            let s = &w.sims[i];
            if s.here() && s.name == name && s.age != 0.0 {
                self.note("ageing", format!("{name} was born on day {day} at the age of {}", s.age));
            }
        }
        self.day = day;
        self.ages = w.sims.iter().map(|s| s.here().then(|| (s.name.clone(), s.age))).collect();
        let here = w.sims.iter().filter(|s| s.here()).count();
        self.fewest = self.fewest.min(here);
        self.most = self.most.max(here);
        if !self.population.contains(&here) {
            self.note("population", format!("{here} residents at {when}"));
        }
    }

    /// Whether the run's story has an event of `kind` (`"Born"`).
    pub fn told(&self, kind: &str) -> bool {
        self.story.get(kind).is_some_and(|&n| n > 0)
    }

    /// Panics with every problem noted and everyone found stuck.
    pub fn assert_ok(&self) {
        let counts: Vec<String> = self.noted.iter().map(|(k, n)| format!("{n} {k}")).collect();
        assert!(self.problems.is_empty(), "problems ({}):\n{}", counts.join(", "), self.problems.join("\n"));
        self.watch.assert_quiet();
    }
}

/// Saves `w` and loads it again: the loaded world passes `check`, has the same residents,
/// households, objects and story, and saves the same again. Returns it (with no view: look at
/// the lot again for lower detail).
pub fn reload(w: &World, content: &str) -> World {
    let json = w.save_json();
    let loaded = World::from_save_json(content, &json).unwrap_or_else(|e| panic!("the save on day {} doesn't load: {e}", clock::day(w.tick)));
    check(&loaded);
    assert_eq!(loaded.tick, w.tick);
    // The first difference, if any.
    fn same<T: PartialEq + std::fmt::Debug>(what: &str, saved: Vec<T>, loaded: Vec<T>) {
        if let Some(i) = (0..saved.len().max(loaded.len())).find(|&i| saved.get(i) != loaded.get(i)) {
            panic!("{what} don't survive saving: {} saved, {} loaded; #{i} was {:?}, is {:?}", saved.len(), loaded.len(), saved.get(i), loaded.get(i));
        }
    }
    let who = |w: &World| w.sims.iter().map(|s| (s.name.clone(), s.here(), s.household, s.age.to_bits())).collect::<Vec<_>>();
    same("the residents", who(w), who(&loaded));
    let homes = |w: &World| w.households.iter().map(|h| (h.plot, h.funds, h.expecting.map(|e| (e.parents, e.due)))).collect::<Vec<_>>();
    same("the households", homes(w), homes(&loaded));
    // (Loading puts what stands on tops in last, so the order, and the ids, can change.)
    let things = |w: &World| {
        let mut list: Vec<_> = w.objects.iter().map(|o| (w.content.objects[o.def].id.clone(), o.x, o.z, o.rot, o.quality, o.wear.to_bits(), o.open, o.growth.to_bits())).collect();
        list.sort();
        list
    };
    same("the objects", things(w), things(&loaded));
    let story = |w: &World| w.events.iter().map(|e| e.id).collect::<Vec<_>>();
    same("the story's events", story(w), story(&loaded));
    // Some things in hand start again from what's there (eating, washing up, a phone call, a
    // visitor at the door aren't saved), so the first load can tidy up; from then on a save
    // loads as it was.
    let again = loaded.save_json();
    let twice = World::from_save_json(content, &again).expect("a loaded game's save loads").save_json();
    if twice != again {
        // Where they part, with a little before it.
        let at = again.bytes().zip(twice.bytes()).position(|(a, b)| a != b).unwrap_or(again.len().min(twice.len()));
        let around = |s: &str| s[at.saturating_sub(300)..(at + 200).min(s.len())].to_string();
        panic!("a loaded game saves differently from character {at}:\nsaved:   …{}\nresaved: …{}", around(&again), around(&twice));
    }
    loaded
}

/// A small deterministic random number generator for building fixtures.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(6364136223846793005).wrapping_add(1))
    }

    pub fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next() * n as f64) as usize % n.max(1)
    }
}

const MARGIN: i64 = 8;
const STREET: i64 = 6;

/// How big a town `town_with` builds.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Plots on each side of a street (one in the middle of each south row is a park).
    pub columns: i64,
    /// Streets, one behind the other (a lot is at most 256 tiles across).
    pub streets: i64,
    /// Fills back yards and the strips beside each house with garden things (a town with
    /// thousands of objects).
    pub yards: bool,
    /// Residents stop moving in at this many; later houses stand empty.
    pub max_residents: usize,
}

/// A large town: 9 houses and the park. The player's household (`player_size` newcomers
/// without jobs, or none) lives on the first plot; every other house has neighbours, most of
/// them with a job. Returns the town file.
pub fn town(content: &str, seed: u64, player_size: usize) -> String {
    town_with(content, seed, player_size, Layout { columns: 5, streets: 1, yards: false, max_residents: usize::MAX })
}

/// A town laid out like `town`, of any size.
pub fn town_with(content: &str, seed: u64, player_size: usize, layout: Layout) -> String {
    let columns = layout.columns;
    let content: Value = serde_json::from_str(content).unwrap();
    let houses: Value = serde_json::from_str(&read("houses.json")).unwrap();
    let (pw, pd) = (
        houses["plot"]["width"].as_i64().unwrap(),
        houses["plot"]["depth"].as_i64().unwrap(),
    );
    let templates = houses["houses"].as_array().unwrap();
    let park = &houses["parks"][0];
    let footprint = |def: &str| -> (i64, i64) {
        content["objects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["id"] == def)
            .and_then(|o| o["footprint"].as_array())
            .map_or((1, 1), |f| (f[0].as_i64().unwrap(), f[1].as_i64().unwrap()))
    };
    let mut rng = Lcg::new(seed);
    let width = MARGIN * 2 + columns * pw;
    let depth = MARGIN * 2 + (pd * 2 + STREET) * layout.streets;
    let park_index = columns + columns / 2;

    let (mut walls, mut doors, mut windows, mut objects) = (vec![], vec![], vec![], vec![]);
    let (mut plots, mut households, mut sims, mut relationships) = (vec![], vec![], vec![], vec![]);
    let traits: Vec<(String, Vec<String>)> = content["traits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            let conflicts = t["conflicts"].as_array().map_or(vec![], |c| {
                c.iter().map(|c| c.as_str().unwrap().to_string()).collect()
            });
            (t["id"].as_str().unwrap().to_string(), conflicts)
        })
        .collect();
    let genders: Vec<String> = content["genders"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["id"].as_str().unwrap().to_string())
        .collect();
    let tracks: Vec<String> = content["careerCategories"]
        .as_array()
        .map(|cats| {
            cats.iter()
                .flat_map(|c| c["tracks"].as_array().unwrap())
                .map(|t| t["id"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default();
    // One-tile garden things people can do something with, for the yards: by `outdoors`, not by
    // category (a finished one is in Validate until it's signed off).
    let yard_defs: Vec<String> = content["objects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| {
            o["outdoors"].as_bool() == Some(true)
                && o["interactions"].as_array().is_some_and(|i| !i.is_empty())
                && o["footprint"].as_array().is_none_or(|f| f[0] == 1 && f[1] == 1)
        })
        .map(|o| o["id"].as_str().unwrap().to_string())
        .collect();

    let mut house_number = 0;
    // Streets one behind the other, back yards touching.
    for (street, row) in (0..layout.streets).flat_map(|s| [(s, 0), (s, 1)]) {
        let block = MARGIN + street * (pd * 2 + STREET);
        let street_name = ["Elm", "Oak", "Ash", "Birch"][street as usize % 4];
        for col in 0..columns {
            let index = row * columns + col;
            let (x, z) = (
                MARGIN + col * pw,
                if row == 0 {
                    block
                } else {
                    block + pd + STREET
                },
            );
            let rotated = row == 0;
            // Plot coordinates to town coordinates (the north row is turned 180°).
            let point = |p: [f64; 2]| -> [f64; 2] {
                if rotated {
                    [(x + pw) as f64 - p[0], (z + pd) as f64 - p[1]]
                } else {
                    [x as f64 + p[0], z as f64 + p[1]]
                }
            };
            let mut place = |list: &Value| {
                for o in list.as_array().unwrap() {
                    let def = o["def"].as_str().unwrap();
                    let (ox, oz, rot) = (
                        o["x"].as_i64().unwrap(),
                        o["z"].as_i64().unwrap(),
                        o["rot"].as_i64().unwrap(),
                    );
                    let (fw, fd) = footprint(def);
                    let (w, d) = if rot % 2 == 0 { (fw, fd) } else { (fd, fw) };
                    objects.push(if rotated {
                        json!({"def": def, "x": x + pw - ox - w, "z": z + pd - oz - d, "rot": (rot + 2) % 4})
                    } else {
                        json!({"def": def, "x": x + ox, "z": z + oz, "rot": rot})
                    });
                }
            };
            if index == park_index {
                place(&park["objects"]);
                plots.push(
                    json!({"name": "Park", "x": x, "z": z, "w": pw, "d": pd, "public": true}),
                );
                continue;
            }
            let h = &templates[house_number % templates.len()];
            house_number += 1;
            for wall in h["walls"].as_array().unwrap() {
                let c: Vec<f64> = wall
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect();
                let (a, b) = (point([c[0], c[1]]), point([c[2], c[3]]));
                walls.push(json!([a[0] as i64, a[1] as i64, b[0] as i64, b[1] as i64]));
            }
            let edge = |d: &Value| {
                let (dx, dz, axis) = (
                    d["x"].as_i64().unwrap(),
                    d["z"].as_i64().unwrap(),
                    d["axis"].as_str().unwrap(),
                );
                if !rotated {
                    json!({"x": x + dx, "z": z + dz, "axis": axis})
                } else if axis == "x" {
                    json!({"x": x + pw - 1 - dx, "z": z + pd - dz, "axis": "x"})
                } else {
                    json!({"x": x + pw - dx, "z": z + pd - 1 - dz, "axis": "z"})
                }
            };
            doors.extend(h["doors"].as_array().unwrap().iter().map(edge));
            if let Some(w) = h["windows"].as_array() {
                windows.extend(w.iter().map(edge));
            }
            place(&h["objects"]);
            if layout.yards {
                place(&yard(h, house_number, &yard_defs, (pw, pd), footprint));
            }
            let spawns: Vec<[f64; 2]> = h["spawns"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| point([p[0].as_f64().unwrap(), p[1].as_f64().unwrap()]))
                .collect();
            let plot = plots.len();
            plots.push(
                json!({"name": format!("{} {street_name} Street", house_number * 2 - row as usize),
                "x": x, "z": z, "w": pw, "d": pd, "entry": spawns[0]}),
            );

            let player = households.is_empty();
            // Neighbours fit their house, as in the game (`town.ts`): its bedrooms, maybe one more.
            let bedrooms = h["bedrooms"].as_u64().unwrap_or(1) as usize;
            let size = if player {
                player_size
            } else {
                (bedrooms + rng.below(2)).clamp(1, 4)
            };
            let size = size.min(layout.max_residents.saturating_sub(sims.len()));
            if size == 0 {
                let name = if player { "Player".to_string() } else { format!("House {}", households.len()) };
                households.push(json!({"name": name, "plot": plot, "player": player}));
                continue;
            }
            let household = households.len();
            households.push(
                json!({"name": format!("House {household}"), "plot": plot, "player": player}),
            );
            let first = sims.len();
            // A couple shares the house when there are two adults, sometimes with a grown child.
            let couple = size >= 2 && rng.next() < 0.6;
            let family = couple && size >= 3 && rng.next() < 0.6;
            let parents_age = 44.0 + rng.next() * 12.0;
            for m in 0..size {
                let mut chosen: Vec<String> = Vec::new();
                for _ in 0..2 {
                    let (t, conflicts) = &traits[rng.below(traits.len())];
                    if !chosen.contains(t) && !chosen.iter().any(|c| conflicts.contains(c)) {
                        chosen.push(t.clone());
                    }
                }
                let spawn = spawns[m % spawns.len()];
                let mut sim = json!({
                    "name": format!("R{}", sims.len()),
                    "household": household,
                    "gender": genders[rng.below(genders.len())],
                    "traits": chosen,
                    "x": spawn[0], "z": spawn[1],
                });
                if family && m < 3 {
                    // A child of any age from three to grown up.
                    sim["age"] = json!(if m < 2 { parents_age.floor() } else { (parents_age - 20.0 - rng.next() * 30.0).max(3.0).floor() });
                }
                if !player && !tracks.is_empty() && rng.next() < 0.75 {
                    let level = (rng.next().powi(2) * 6.0) as usize;
                    sim["job"] = json!({"career": tracks[rng.below(tracks.len())], "level": level});
                }
                sims.push(sim);
            }
            // Housemates are close; the couple are partners, and their grown child is family.
            for a in first..sims.len() {
                for b in a + 1..sims.len() {
                    if couple && a == first && b == first + 1 {
                        relationships.push(json!({"a": a, "b": b, "preset": "partners"}));
                    } else if family && b == first + 2 && a < b {
                        relationships.push(json!({"a": b, "b": a, "preset": "parent"}));
                    } else {
                        relationships.push(json!({"a": a, "b": b, "preset": "friends"}));
                    }
                }
            }
        }
    }
    // Neighbours know each other a little.
    let n = sims.len();
    for a in 0..n {
        for b in a + 1..n {
            if sims[a]["household"] == sims[b]["household"] {
                continue;
            }
            let roll = rng.next();
            let preset = if roll < 0.03 {
                "rivals"
            } else if roll < 0.09 {
                "friends"
            } else if roll < 0.3 {
                "acquaintances"
            } else {
                continue;
            };
            relationships.push(json!({"a": a, "b": b, "preset": preset}));
        }
    }
    // A way out of town at both ends of every street.
    let exits: Vec<Value> = (0..layout.streets)
        .flat_map(|s| {
            let street_z = (MARGIN + s * (pd * 2 + STREET) + pd) as f64 + STREET as f64 / 2.0;
            [json!([0.5, street_z]), json!([width as f64 - 0.5, street_z])]
        })
        .collect();
    json!({
        "width": width, "depth": depth,
        "walls": walls, "doors": doors, "windows": windows, "objects": objects,
        "plots": plots, "households": households, "sims": sims, "relationships": relationships,
        "exits": exits,
    })
    .to_string()
}

/// Garden things for a house's back yard and the strips beside it (plot coordinates, plot
/// `w × d`): rows with a free lane in front of each, clear of the house's own objects.
fn yard(house: &Value, number: usize, defs: &[String], (w, d): (i64, i64), footprint: impl Fn(&str) -> (i64, i64)) -> Value {
    // Tiles the house's own objects stand on, and one around them (where they're used from).
    let mut taken = std::collections::HashSet::new();
    for o in house["objects"].as_array().unwrap() {
        let (x, z, rot) = (o["x"].as_i64().unwrap(), o["z"].as_i64().unwrap(), o["rot"].as_i64().unwrap());
        let (fw, fd) = footprint(o["def"].as_str().unwrap());
        let (ow, od) = if rot % 2 == 0 { (fw, fd) } else { (fd, fw) };
        for dz in -1..=od {
            for dx in -1..=ow {
                taken.insert((x + dx, z + dz));
            }
        }
    }
    // Two rows across the back facing the house, one down each side facing out.
    let back = [d - 3, d - 1].into_iter().flat_map(|z| (1..w).step_by(2).map(move |x| (x, z, 2)));
    let sides = (5..d - 4).step_by(2).flat_map(|z| [(1, z, 3), (w - 2, z, 1)]);
    let list = back
        .chain(sides)
        .filter(|&(x, z, _)| !taken.contains(&(x, z)))
        .enumerate()
        .map(|(k, (x, z, rot))| json!({"def": defs[(number * 7 + k) % defs.len()], "x": x, "z": z, "rot": rot}))
        .collect();
    Value::Array(list)
}

/// With `DUMP_STATES` set, writes the world's state to `<repo>/debug-states/<name>.json`, to
/// look at in the game: `?state=<name>` on the dev server, or paste it in Settings → Interface →
/// Diagnostics.
pub fn dump_state(w: &World, name: &str) {
    if std::env::var_os("DUMP_STATES").is_none() {
        return;
    }
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../debug-states");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{name}.json")), w.save_json()).unwrap();
}

/// The player's household of three in `town(content, 7, 3)` at 9:00 on day one, free will off,
/// everyone rested and fed; they get on well and know nobody else. (World, its residents, content.)
pub fn household_of_three() -> (World, Vec<usize>, String) {
    let content = content();
    let mut w = World::from_json(&content, &town(&content, 7, 3), 7).unwrap();
    let h = w.households.iter().position(|h| h.player).unwrap() as u32;
    let home: Vec<usize> = (0..w.sims.len()).filter(|&i| w.sims[i].household == h).collect();
    assert_eq!(home.len(), 3);
    w.autonomy = false;
    while w.tick < clock::tick_at(1, 9.0 * 60.0).unwrap() {
        w.tick_once();
    }
    for &i in &home {
        let n = w.content.needs.len();
        w.sims[i].needs[..n].fill(1.0);
        for x in (0..w.sims.len()).filter(|x| !home.contains(x)) {
            *w.relationships.get_mut(i, x) = Default::default();
            *w.relationships.get_mut(x, i) = Default::default();
        }
        for &x in home.iter().filter(|&&x| x != i) {
            w.relationships.get_mut(i, x).friendship = 70.0;
        }
    }
    (w, home, content)
}

/// Sits `who` on the home's sofa as if they'd chosen to (the player's order to sit would be theirs
/// to keep: nobody talks to them then), with the sofa's comfort still to come. Free will off.
pub fn sit_on_the_sofa(w: World, content: &str, who: &[usize]) -> World {
    use sim_core::content::Pose;
    let plot = w.households.iter().find(|h| h.player).and_then(|h| h.plot).unwrap() as usize;
    let p = &w.plots[plot];
    let sofa_def = w.content.object_index("sofa").unwrap();
    let sofa = w.objects.iter().find(|o| o.def == sofa_def && o.x >= p.x && o.x < p.x + p.w && o.z >= p.z && o.z < p.z + p.d).unwrap().id;
    let relax = w.content.objects[sofa_def].interactions.iter().position(|i| i.pose == Pose::Sit).unwrap();
    let mut w = w;
    let comfort = w.content.needs.iter().position(|n| n.id == "comfort").unwrap();
    for &i in who {
        w.sims[i].needs[comfort] = 0.2;
        w.apply(sim_core::Command::Use { sim: i as u32, object: sofa, interaction: relax }).unwrap();
    }
    let seated = |w: &World| who.iter().all(|&i| w.sims[i].pose == Pose::Sit && w.sims[i].current().is_some_and(|a| matches!(a.phase, Phase::Using { .. })));
    for _ in 0..(20.0 / MINUTES_PER_TICK) as u64 {
        if seated(&w) {
            break;
        }
        w.tick_once();
    }
    assert!(seated(&w), "they didn't sit down");
    // Saved and loaded with the orders made their own: they sit down again.
    let mut save: Value = serde_json::from_str(&w.save_json()).unwrap();
    for s in save["sims"].as_array_mut().unwrap() {
        for t in s["tasks"].as_array_mut().into_iter().flatten() {
            if t.get("directed").is_some() {
                t["directed"] = false.into();
            }
        }
    }
    let mut w = World::from_save_json(content, &save.to_string()).unwrap();
    w.autonomy = false;
    for _ in 0..(5.0 / MINUTES_PER_TICK) as u64 {
        if seated(&w) {
            break;
        }
        w.tick_once();
    }
    assert!(seated(&w), "they didn't sit down again");
    for &i in who {
        assert!(w.sims[i].current().is_some_and(|a| !a.task.directed));
        w.sims[i].needs[comfort] = 0.2;
    }
    w
}

/// Metres between two points.
pub fn apart(p: [f32; 2], q: [f32; 2]) -> f32 {
    (p[0] - q[0]).hypot(p[1] - q[1])
}
