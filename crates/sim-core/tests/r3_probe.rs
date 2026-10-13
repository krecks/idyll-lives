//! Release review (R3) probes, all ignored. The proofs (`every_days_gates_fire`,
//! `residents_still_age_after_day_1167`, `someone_too_tired_to_talk_leaves_others_be`,
//! `nobody_follows_someone_across_town_for_a_chat`, `practising_with_nothing_else_to_do_doesnt_stutter`,
//! `newborns_are_born_at_age_zero`) fail until their bug is fixed. `long_run_probe` runs a whole
//! town and logs non-finite numbers, needs out of range, the longest stretch in one task and
//! residents flipping between two things; `trace` and `house` print the stories behind them.
//!
//! `cargo test --release -p sim-core --test r3_probe -- --ignored --nocapture <name>`
//! (`long_run_probe`: `DAYS=60 SEEDS=7,11` by default; `LOD=1` looks at the player's home).

mod common;

use std::collections::{BTreeMap, VecDeque};

use sim_core::social::EventKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{World, clock};

/// The days on which `tick_at(day, minute)` names a tick that `day()` doesn't put on that day,
/// as ranges: every exact-tick gate (`tick == tick_at(day(tick), m)`) is dead on them.
fn dead_days(minute: f32, last: u32) -> Vec<(u32, u32)> {
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for day in 2..=last {
        let Some(t) = clock::tick_at(day, minute) else { continue };
        if clock::tick_at(clock::day(t), minute) == Some(t) {
            continue;
        }
        match ranges.last_mut() {
            Some(r) if r.1 + 1 == day => r.1 = day,
            _ => ranges.push((day, day)),
        }
    }
    ranges
}

#[test]
#[ignore = "R3 proof: fails until the clock counts whole ticks"]
fn every_days_gates_fire() {
    let mut failed = false;
    for minute in [0.0, 6.0 * 60.0, 9.0 * 60.0, 12.0 * 60.0, 18.0 * 60.0, 23.5 * 60.0] {
        let dead = dead_days(minute, 6000);
        eprintln!("minute {minute:>6}: dead on days {dead:?}");
        failed |= !dead.is_empty();
    }
    assert!(!failed, "exact-tick gates stop on some days");
}

/// Around every midnight up to day 6000: the minute of the day stays below 1440 and days don't
/// go backwards.
#[test]
#[ignore = "R3 probe: clock edges"]
fn clock_edges() {
    let (mut at_1440, mut backwards, mut first) = (0, 0, None);
    for day in 2..=6000u32 {
        let t0 = clock::tick_at(day, 0.0).unwrap();
        for t in t0 - 3..t0 + 3 {
            if clock::minute_of_day(t) >= 1440.0 {
                at_1440 += 1;
                first.get_or_insert((day, t));
            }
            if clock::day(t + 1) < clock::day(t) {
                backwards += 1;
            }
        }
    }
    eprintln!("minute_of_day >= 1440: {at_1440} (first {first:?}); day going backwards: {backwards}");
    assert_eq!(at_1440 + backwards, 0);
}

/// Residents' ages at each midnight from day `from` for `days` days (the town jumped there).
fn ages_from(from: u32, days: u32) -> Vec<(u32, f32, usize)> {
    let content = common::content();
    let town = common::town(&content, 7, 2);
    let mut w = World::from_json(&content, &town, 7).unwrap();
    w.tick = clock::tick_at(from, 23.0 * 60.0).unwrap();
    let mut out = Vec::new();
    let end = clock::tick_at(from + days, 1.0 * 60.0).unwrap();
    let mut last_day = clock::day(w.tick);
    let first = w.sims.iter().position(|s| s.here()).unwrap();
    while w.tick < end {
        w.tick_once();
        let day = clock::day(w.tick);
        if day != last_day {
            last_day = day;
            let grew = w.events.iter().filter(|e| e.kind == EventKind::GrewOlder).count();
            out.push((day, w.sims[first].age, grew));
        }
    }
    out
}

#[test]
#[ignore = "R3 proof: fails until the clock counts whole ticks"]
fn residents_still_age_after_day_1167() {
    let early = ages_from(10, 3);
    let late = ages_from(1166, 3);
    eprintln!("from day 10: {early:?}\nfrom day 1166: {late:?}");
    let grows = |a: &[(u32, f32, usize)]| a.windows(2).filter(|p| p[1].1 > p[0].1).count();
    assert_eq!(grows(&early), 2, "ages each midnight early on");
    assert_eq!(grows(&late), 2, "ages each midnight on days 1167 and 1168 too");
}

/// Two people on an empty lot, a book, and a chat (acceptance certain); energy is urgent.
const TALK: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.05},{"id":"social","label":"Social","decayPerHour":0.1},
             {"id":"fun","label":"Fun","decayPerHour":0.05}],
    "autonomy":{"urgent":["energy"],"urgentBelow":0.1},
    "objects":[{"id":"book","name":"Book","interactions":[{"id":"read","label":"Read","minutes":30,"effects":{"fun":0.6}}]}],
    "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},
                "needs":{"social":0.4},"targetNeeds":{"social":0.4},"success":{"friendship":2,"targetFriendship":2}}],
    "bondPresets":{"roommates":{"friendship":20}},"socialRules":{"defaultBond":"roommates"}}"#;

fn talk_town(width: u32, sims: &str) -> World {
    let lot = format!(r#"{{"width":{width},"depth":12,"objects":[{{"def":"book","x":8,"z":3,"rot":0}}],"sims":[{sims}]}}"#);
    World::from_json(TALK, &lot, 5).unwrap()
}

#[test]
#[ignore = "R3 proof: fails until someone about to drop doesn't start chats"]
fn someone_too_tired_to_talk_leaves_others_be() {
    let mut w = talk_town(16, r#"{"name":"Ann","x":4.5,"z":5.5},{"name":"Bob","x":8.5,"z":5.5}"#);
    // Ann is about to drop (pressed: a chat would end at once) and lonely; Bob wants to read.
    let e: f32 = std::env::var("ENERGY").ok().and_then(|v| v.parse().ok()).unwrap_or(0.05);
    w.sims[0].needs[..3].copy_from_slice(&[e, 0.1, 1.0]);
    w.sims[1].needs[..3].copy_from_slice(&[1.0, 1.0, 0.2]);
    // Chats Bob is pulled into that end straight away (his reading dropped for nothing).
    let (mut cut_short, mut since) = (0, None);
    for t in 0..20 * 30 {
        w.tick_once();
        if std::env::var("SHOW").is_ok() && t % 20 == 0 {
            eprintln!("{t}: Ann {:?} | Bob {:?} engaged {:?}", w.sims[0].current().map(|a| (a.task.kind, phase_name(&a.phase))), w.sims[1].current().map(|a| (a.task.kind, phase_name(&a.phase))), w.sims[1].engaged_with);
        }
        match (w.sims[1].engaged_with, since) {
            (Some(_), None) => since = Some(w.tick),
            (None, Some(from)) => {
                cut_short += usize::from(w.tick - from <= 2);
                since = None;
            }
            _ => {}
        }
    }
    eprintln!("Bob was pulled into {cut_short} chats that ended at once in 30 minutes; his fun is {:.2}", w.sims[1].needs[2]);
    assert!(cut_short <= 1, "Bob was pulled into {cut_short} chats that ended at once");
}

#[test]
#[ignore = "R3 proof: fails until a chat gives up on someone who keeps walking off"]
fn nobody_follows_someone_across_town_for_a_chat() {
    let mut w = talk_town(80, r#"{"name":"Ann","x":4.5,"z":5.5},{"name":"Bob","x":6.5,"z":5.5}"#);
    w.sims[0].needs[..3].copy_from_slice(&[1.0, 0.1, 1.0]);
    w.sims[1].needs[..3].copy_from_slice(&[1.0, 1.0, 1.0]);
    // Ann decides to chat with Bob...
    let chatting = |w: &World| matches!(w.sims[0].current().map(|a| a.task.kind), Some(TaskKind::Social { target: 1, .. }));
    for _ in 0..20 * 10 {
        if chatting(&w) {
            break;
        }
        w.tick_once();
    }
    assert!(chatting(&w), "Ann never went to chat with Bob");
    // ...and Bob is told to walk to the far end and back (busy: not someone to talk to).
    for x in [75.5, 3.5, 75.5] {
        w.apply(sim_core::Command::MoveTo { sim: 1, x, z: 5.5 }).unwrap();
    }
    let mut minutes = 0;
    while chatting(&w) && minutes < 120 {
        for _ in 0..20 {
            w.tick_once();
        }
        minutes += 1;
    }
    eprintln!("Ann followed Bob for {minutes} minutes; she's at {:.1}", w.sims[0].pos[0]);
    assert!(minutes <= 15, "Ann followed Bob for {minutes} minutes");
}

#[test]
#[ignore = "R3 proof: fails until practising doesn't stop the moment the need it fills is full"]
fn practising_with_nothing_else_to_do_doesnt_stutter() {
    let content = r#"{
        "needs":[{"id":"fun","label":"Fun","decayPerHour":0.05}],
        "skills":[{"id":"gardening","label":"Gardening"}],
        "skillRules":{"interest":0.05},
        "objects":[{"id":"flowerbed","name":"Flowerbed","interactions":[
            {"id":"tend","label":"Tend","minutes":20,"effects":{"fun":0.12},"skills":{"gardening":0.3}}]}]}"#;
    let lot = r#"{"width":12,"depth":12,"objects":[{"def":"flowerbed","x":5,"z":5,"rot":0}],"sims":[{"name":"Ann","x":5.5,"z":7.5}]}"#;
    let mut w = World::from_json(content, lot, 5).unwrap();
    w.sims[0].needs[0] = 1.0;
    let (mut starts, mut was) = (0, false);
    for _ in 0..20 * 30 {
        w.tick_once();
        let now = matches!(w.sims[0].current().map(|a| &a.phase), Some(Phase::Using { .. }));
        starts += usize::from(now && !was);
        was = now;
    }
    eprintln!("Ann started tending {starts} times in 30 minutes (skill {:.3})", w.sims[0].skills[0]);
    assert!(starts <= 2, "Ann started tending {starts} times in 30 minutes");
}

#[test]
#[ignore = "R3 proof: fails until a newborn isn't aged the moment they're born"]
fn newborns_are_born_at_age_zero() {
    let content = common::content();
    let town = common::town(&content, 7, 2);
    let mut w = World::from_json(&content, &town, 7).unwrap();
    let h = w.households.iter().position(|h| h.player).unwrap();
    let adults: Vec<u32> = w.sims.iter().filter(|s| s.household as usize == h && s.adult(&w.content)).map(|s| s.id).collect();
    w.households[h].expecting = Some(sim_core::world::Expecting { parents: [adults[0], adults[1]], due: 2 });
    let midnight = clock::tick_at(2, 0.0).unwrap();
    while w.tick < midnight {
        w.tick_once();
    }
    let born = w.events.iter().find(|e| e.kind == EventKind::Born).expect("the baby was born");
    let age = w.sims[born.a as usize].age;
    eprintln!("born at age {age}");
    assert_eq!(age, 0.0, "a newborn is a day's ageing old already");
}

/// What a resident is doing, as something to compare: the task kind and what it's with (not
/// how far along it is).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Key {
    Idle,
    Away,
    Use(u32, usize, usize),
    MoveTo,
    Social(u32, usize),
    Join(u32),
    Work,
    Visit(u32),
    GoHome,
    Answer(u32),
    Clean(i32, i32),
    Repair(u32),
    Spot(usize),
    Cook(usize),
    Eat,
    Dishes,
    Phone(usize),
}

fn key(w: &World, i: usize) -> Key {
    let s = &w.sims[i];
    if s.away_until.is_some() {
        return Key::Away;
    }
    let Some(a) = s.current() else { return Key::Idle };
    match a.task.kind {
        TaskKind::Use { object, interaction } => Key::Use(object, w.objects.get(object as usize).map_or(usize::MAX, |o| o.def), interaction),
        TaskKind::MoveTo { .. } => Key::MoveTo,
        TaskKind::Social { target, social } => Key::Social(target, social),
        TaskKind::Join { leader } => Key::Join(leader),
        TaskKind::Work => Key::Work,
        TaskKind::Visit { plot } => Key::Visit(plot),
        TaskKind::GoHome => Key::GoHome,
        TaskKind::Answer { guest } => Key::Answer(guest),
        TaskKind::Clean { x, z } => Key::Clean(x, z),
        TaskKind::Repair { object } => Key::Repair(object),
        TaskKind::Spot { accident } => Key::Spot(accident),
        TaskKind::Cook { recipe, .. } => Key::Cook(recipe),
        TaskKind::Eat { .. } => Key::Eat,
        TaskKind::Dishes { .. } => Key::Dishes,
        TaskKind::Phone { action, .. } => Key::Phone(action),
    }
}

fn kind_name(w: &World, k: Key) -> String {
    match k {
        Key::Use(_, d, i) => {
            let Some(def) = w.content.objects.get(d) else { return "Use ?".into() };
            format!("Use {}.{}", def.id, def.interactions[i].id)
        }
        Key::Social(_, s) => format!("Social {}", w.content.socials[s].id),
        Key::Join(_) => "Join".into(),
        Key::Visit(_) => "Visit".into(),
        Key::Answer(_) => "Answer".into(),
        Key::Clean(..) => "Clean".into(),
        Key::Repair(_) => "Repair".into(),
        Key::Spot(a) => format!("Spot {}", w.content.accidents[a].id),
        Key::Cook(r) => format!("Cook {}", w.content.recipes[r].id),
        Key::Phone(a) => format!("Phone {}", w.content.phone.actions[a].id),
        k => format!("{k:?}"),
    }
}

fn phase_name(p: &Phase) -> &'static str {
    match p {
        Phase::Routing { .. } => "Routing",
        Phase::Using { .. } => "Using",
        Phase::Waiting { .. } => "Waiting",
        Phase::Conversing { .. } => "Conversing",
        Phase::Queued { .. } => "Queued",
        Phase::Knocking { .. } => "Knocking",
    }
}

#[derive(Default)]
struct Track {
    key: Option<Key>,
    since: u64,
    /// The last few things done: (what, from, to).
    done: VecDeque<(Key, u64, u64)>,
    phase: Option<(&'static str, Key)>,
    phase_since: u64,
    /// Social task: times it went from waiting back to walking after the target (a chase).
    chases: u32,
    redo_chain: u32,
    engaged_since: Option<u64>,
    still_since: Option<(u64, [f32; 2])>,
}

#[derive(Default)]
struct Probe {
    non_finite: BTreeMap<String, usize>,
    first_non_finite: Vec<String>,
    out_of_range: BTreeMap<String, usize>,
    need_min: f32,
    need_max: f32,
    /// Longest stretch per kind name: (ticks, resident, from tick).
    longest: BTreeMap<String, (u64, String, u64)>,
    /// Longest stretch per (task kind, phase).
    longest_phase: BTreeMap<String, (u64, String, u64)>,
    aba: BTreeMap<String, usize>,
    abab: BTreeMap<String, usize>,
    quick_redo: BTreeMap<String, usize>,
    short: BTreeMap<String, usize>,
    starts: usize,
    most_chases: (u32, String),
    longest_redo_chain: (u32, String),
    longest_engaged: u64,
    short_engaged: usize,
    spots: BTreeMap<String, usize>,
    short_engaged_hour: BTreeMap<(usize, u64), usize>,
    stuck_walking: usize,
    born_ages: Vec<f32>,
    story: BTreeMap<String, usize>,
    waking_starts: BTreeMap<String, usize>,
    waking_hours: f64,
}

fn note(map: &mut BTreeMap<String, usize>, k: String) {
    *map.entry(k).or_default() += 1;
}

fn longest(map: &mut BTreeMap<String, (u64, String, u64)>, k: String, ticks: u64, who: &str, from: u64) {
    let e = map.entry(k).or_insert((0, String::new(), 0));
    if ticks > e.0 {
        *e = (ticks, who.to_string(), from);
    }
}

/// Window for flips: the same thing again within this many ticks (10 game minutes).
const FLIP: u64 = 200;

fn finite_checks(w: &World, p: &mut Probe, full: bool) {
    let n = w.content.needs.len();
    let max_level = w.content.skill_rules.max_level;
    let bad = |p: &mut Probe, what: &str, who: &str, v: f32| {
        note(&mut p.non_finite, what.to_string());
        if p.first_non_finite.len() < 10 {
            p.first_non_finite.push(format!("tick {} {who}: {what} = {v}", w.tick));
        }
    };
    for s in w.sims.iter().filter(|s| s.here()) {
        for (k, &v) in s.needs[..n].iter().enumerate() {
            if !v.is_finite() {
                bad(p, "need", &s.name, v);
            } else {
                p.need_min = p.need_min.min(v);
                p.need_max = p.need_max.max(v);
                if !(0.0..=1.0).contains(&v) {
                    note(&mut p.out_of_range, format!("need {}", w.content.needs[k].id));
                }
            }
        }
        for (what, v) in [("pos.x", s.pos[0]), ("pos.z", s.pos[1]), ("yaw", s.yaw), ("age", s.age), ("grade", s.grade)] {
            if !v.is_finite() {
                bad(p, what, &s.name, v);
            }
        }
        if !(0.0..=100.0).contains(&s.grade) {
            note(&mut p.out_of_range, "grade".into());
        }
        if s.age < 0.0 {
            note(&mut p.out_of_range, "age".into());
        }
        for &v in &s.skills[..w.content.skills.len()] {
            if !v.is_finite() {
                bad(p, "skill", &s.name, v);
            } else if !(0.0..=max_level).contains(&v) {
                note(&mut p.out_of_range, "skill".into());
            }
        }
        if let Some(a) = s.current() {
            let e = match a.phase {
                Phase::Using { elapsed } | Phase::Conversing { elapsed, .. } => elapsed,
                _ => 0.0,
            };
            if !e.is_finite() {
                bad(p, "elapsed", &s.name, e);
            }
            if let Phase::Routing { waypoints, .. } = &a.phase
                && waypoints.iter().any(|w| !w[0].is_finite() || !w[1].is_finite())
            {
                bad(p, "waypoint", &s.name, f32::NAN);
            }
        }
    }
    if !full {
        return;
    }
    for o in &w.objects {
        if !o.wear.is_finite() {
            bad(p, "wear", "object", o.wear);
        } else if !(0.0..=1.0).contains(&o.wear) {
            note(&mut p.out_of_range, "wear".into());
        }
    }
    for &d in &w.dirt {
        if !d.is_finite() {
            bad(p, "dirt", "tile", d);
        } else if !(0.0..=1.0).contains(&d) {
            note(&mut p.out_of_range, "dirt".into());
        }
    }
    let m = w.relationships.len();
    for a in 0..m {
        for b in 0..m {
            let r = w.relationships.get(a, b);
            let c = w.relationships.chemistry(a, b);
            for (what, v, lim) in [("friendship", r.friendship, 100.0), ("romance", r.romance, 100.0), ("chemistry", c, 1.0)] {
                if !v.is_finite() {
                    bad(p, what, "pair", v);
                } else if v.abs() > lim {
                    note(&mut p.out_of_range, what.into());
                }
            }
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn probe(days: u32, seed: u64) -> Probe {
    let content = common::content();
    let town = common::town(&content, seed, 2);
    let mut w = World::from_json(&content, &town, seed as u32).expect("town loads");
    common::lod_from_env(&mut w);
    let mut p = Probe { need_min: 1.0, need_max: 0.0, ..Default::default() };
    let mut tracks: Vec<Track> = Vec::new();
    let end = clock::tick_at(days + 1, 8.0 * 60.0).unwrap();
    let started = std::time::Instant::now();
    let hour = (60.0 / sim_core::MINUTES_PER_TICK) as u64;
    while w.tick < end {
        w.tick_once();
        let tick = w.tick;
        while tracks.len() < w.sims.len() {
            tracks.push(Track { since: tick, ..Default::default() });
        }
        for e in w.events.iter().rev().take_while(|e| e.tick == tick) {
            note(&mut p.story, format!("{:?}", e.kind));
            if e.kind == EventKind::Born {
                p.born_ages.push(w.sims[e.a as usize].age);
            }
        }
        finite_checks(&w, &mut p, tick.is_multiple_of(hour));
        let waking = (8.0..22.0).contains(&clock::hour(tick));
        for i in 0..w.sims.len() {
            let s = &w.sims[i];
            if !s.here() {
                tracks[i] = Track { since: tick, ..Default::default() };
                continue;
            }
            let k = key(&w, i);
            let name = s.name.clone();
            let t = &mut tracks[i];
            if waking && !w.content.life.baby(s.age) && k != Key::Away {
                p.waking_hours += 1.0 / hour as f64;
            }
            if t.key != Some(k) {
                if let Some(old) = t.key {
                    let ticks = tick - t.since;
                    longest(&mut p.longest, kind_name(&w, old), ticks, &name, t.since);
                    if !matches!(old, Key::Idle | Key::Away) {
                        if ticks <= 2 {
                            note(&mut p.short, kind_name(&w, old));
                        }
                        t.done.push_back((old, t.since, tick));
                        if t.done.len() > 4 {
                            t.done.pop_front();
                        }
                    }
                }
                if !matches!(k, Key::Idle | Key::Away) {
                    p.starts += 1;
                    if matches!(k, Key::Spot(_)) {
                        note(&mut p.spots, kind_name(&w, k));
                    }
                    if waking && !w.content.life.baby(s.age) {
                        note(&mut p.waking_starts, name.clone());
                    }
                    let d: Vec<_> = t.done.iter().copied().collect();
                    // The same again right after a stretch of it that ended within a minute.
                    if let Some(&(x, from, to)) = d.last()
                        && x == k
                        && tick - to <= 20
                        && to - from <= 20
                    {
                        note(&mut p.quick_redo, kind_name(&w, k));
                        t.redo_chain += 1;
                        if t.redo_chain > p.longest_redo_chain.0 {
                            p.longest_redo_chain = (t.redo_chain, format!("{name} {} until day {} {:.0} min", kind_name(&w, k), clock::day(tick), clock::minute_of_day(tick)));
                        }
                    } else {
                        t.redo_chain = 0;
                    }
                    // X, Y, X again within the window.
                    if d.len() >= 2 {
                        let (x, _, x_end) = d[d.len() - 2];
                        let (y, ..) = d[d.len() - 1];
                        if x == k && y != k && tick - x_end <= FLIP {
                            note(&mut p.aba, format!("{} <> {}", kind_name(&w, x), kind_name(&w, y)));
                        }
                    }
                    // X, Y, X, Y.
                    if d.len() >= 3 {
                        let (x, x_start, _) = d[d.len() - 3];
                        let (y, ..) = d[d.len() - 2];
                        let (x2, ..) = d[d.len() - 1];
                        if x == x2 && y == k && x != y && tick - x_start <= 3 * FLIP {
                            note(&mut p.abab, format!("{} <> {}", kind_name(&w, x), kind_name(&w, y)));
                        }
                    }
                }
                t.key = Some(k);
                t.since = tick;
                t.chases = 0;
            }
            // Phases: the longest stretch of each, and chases after a social's target.
            let ph = s.current().map(|a| (phase_name(&a.phase), k));
            if ph != t.phase {
                if let Some((name_p, pk)) = t.phase {
                    longest(&mut p.longest_phase, format!("{} / {}", kind_name(&w, pk), name_p), tick - t.phase_since, &name, t.phase_since);
                    if matches!(pk, Key::Social(..) | Key::Join(_)) && name_p == "Waiting" && ph.is_some_and(|(n, k2)| n == "Routing" && k2 == pk) {
                        t.chases += 1;
                        if t.chases > p.most_chases.0 {
                            p.most_chases = (t.chases, format!("{name} {} from tick {}", kind_name(&w, pk), t.since));
                        }
                    }
                }
                t.phase = ph;
                t.phase_since = tick;
            }
            // Engaged (talked to) for how long.
            match (s.engaged_with, t.engaged_since) {
                (Some(_), None) => t.engaged_since = Some(tick),
                (None, Some(from)) => {
                    p.longest_engaged = p.longest_engaged.max(tick - from);
                    if tick - from <= 2 {
                        p.short_engaged += 1;
                        *p.short_engaged_hour.entry((i, tick / hour)).or_default() += 1;
                    }
                    t.engaged_since = None;
                }
                _ => {}
            }
            // Walking without moving.
            if s.is_moving() {
                match t.still_since {
                    Some((from, at)) if at == s.pos => {
                        if tick - from == 100 {
                            p.stuck_walking += 1;
                        }
                    }
                    _ => t.still_since = Some((tick, s.pos)),
                }
            } else {
                t.still_since = None;
            }
        }
        if tick.is_multiple_of(hour * 24) {
            common::check(&w);
        }
    }
    eprintln!(
        "seed {seed}: {} residents ({} here), {days} days in {:.1} s",
        w.sims.len(),
        w.sims.iter().filter(|s| s.here()).count(),
        started.elapsed().as_secs_f64()
    );
    p
}

fn top<V: Ord + Copy + std::fmt::Debug>(map: &BTreeMap<String, V>, n: usize) -> Vec<(String, V)> {
    let mut v: Vec<(String, V)> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    v.sort_by_key(|e| std::cmp::Reverse(e.1));
    v.truncate(n);
    v
}

#[test]
#[ignore = "R3 probe: long run, prints numbers"]
fn long_run_probe() {
    let days: u32 = std::env::var("DAYS").ok().and_then(|d| d.parse().ok()).unwrap_or(60);
    let seeds: Vec<u64> = std::env::var("SEEDS").ok().map_or(vec![7, 11], |s| s.split(',').filter_map(|x| x.parse().ok()).collect());
    let minutes = |t: u64| t as f32 * sim_core::MINUTES_PER_TICK;
    for seed in seeds {
        let p = probe(days, seed);
        eprintln!("== seed {seed}, {days} days ==");
        eprintln!("non-finite: {:?} first: {:?}", p.non_finite, p.first_non_finite);
        eprintln!("out of range: {:?}; needs seen {:.4}..{:.4}", p.out_of_range, p.need_min, p.need_max);
        eprintln!("activity starts: {}; waking hours (non-babies, here): {:.0}; starts per waking hour {:.2}", p.starts, p.waking_hours, p.waking_starts.values().sum::<usize>() as f64 / p.waking_hours.max(1.0));
        let mut by_len: Vec<_> = p.longest.iter().collect();
        by_len.sort_by_key(|e| std::cmp::Reverse(e.1.0));
        eprintln!("longest stretch per kind (minutes):");
        for (k, (t, who, from)) in by_len.iter().take(25) {
            eprintln!("  {:>7.0}  {k}  ({who}, day {} {:.0}:00)", minutes(*t), clock::day(*from), clock::hour(*from).floor());
        }
        let mut by_len: Vec<_> = p.longest_phase.iter().filter(|(k, _)| !k.ends_with("/ Using")).collect();
        by_len.sort_by_key(|e| std::cmp::Reverse(e.1.0));
        eprintln!("longest non-Using phase stretches (minutes):");
        for (k, (t, who, from)) in by_len.iter().take(15) {
            eprintln!("  {:>7.1}  {k}  ({who}, day {} {:.0}:00)", minutes(*t), clock::day(*from), clock::hour(*from).floor());
        }
        eprintln!("A-B-A within {} min: {} total; top {:?}", minutes(FLIP), p.aba.values().sum::<usize>(), top(&p.aba, 12));
        eprintln!("A-B-A-B within {} min: {} total; top {:?}", minutes(3 * FLIP), p.abab.values().sum::<usize>(), top(&p.abab, 12));
        eprintln!("same thing again within 1 min after <=1 min of it: {} total; top {:?}", p.quick_redo.values().sum::<usize>(), top(&p.quick_redo, 10));
        eprintln!("activities over in <=2 ticks: {} total; top {:?}", p.short.values().sum::<usize>(), top(&p.short, 10));
        eprintln!("most chases in one social: {:?}; longest engaged {:.1} min; walking without moving 100+ ticks: {}", p.most_chases, minutes(p.longest_engaged), p.stuck_walking);
        let worst = p.short_engaged_hour.iter().max_by_key(|(_, n)| **n);
        eprintln!(
            "engagements over in <=2 ticks (the target's activity dropped for nothing): {}; resident-hours with 10+: {}; worst hour {worst:?}",
            p.short_engaged,
            p.short_engaged_hour.values().filter(|&&n| n >= 10).count()
        );
        eprintln!("accidents on the spot: {:?}", p.spots);
        eprintln!("longest run of quick restarts of one thing: {:?}", p.longest_redo_chain);
        eprintln!("newborn ages at birth: {:?}", p.born_ages);
        eprintln!("story: {:?}", p.story);
    }
}

fn describe(w: &World, i: usize) -> String {
    let s = &w.sims[i];
    let c = &w.content;
    let needs: Vec<String> = c.needs.iter().zip(&s.needs).map(|(d, v)| format!("{}={v:.3}", d.id)).collect();
    let stage = c.life.stage_at(s.age).map_or("?", |st| st.id.as_str());
    let run = s.planner.run.as_ref().map(|r| format!("block {} skipped {:?} no_place {}", c.activities[r.activity].id, r.skipped, r.no_place));
    let goals: Vec<&str> = s.planner.goals.iter().map(|g| c.goals[g.def].id.as_str()).collect();
    let act = s.current().map(|a| format!("{:?} {}", a.task.kind, phase_name(&a.phase)));
    format!(
        "{} (#{i}, {stage} {:.1}, h{}) at {:.1},{:.1} visiting {:?} engaged {:?} | {} | run {run:?} goals {goals:?} | now {act:?} queue {}",
        s.name,
        s.age,
        s.household,
        s.pos[0],
        s.pos[1],
        s.visiting.map(|v| v.plot),
        s.engaged_with,
        needs.join(" "),
        s.queue().count()
    )
}

/// Prints the stories behind the probe's numbers: who restarts what, what lasts for hours.
/// `SEED`, `DAYS`, `TRACE` (part of a kind name, e.g. `flowerbed`), `LIMIT`, `WHO` (a name).
#[test]
#[ignore = "R3 probe: traces"]
fn trace() {
    let seed: u64 = std::env::var("SEED").ok().and_then(|d| d.parse().ok()).unwrap_or(7);
    let days: u32 = std::env::var("DAYS").ok().and_then(|d| d.parse().ok()).unwrap_or(10);
    let pat = std::env::var("TRACE").unwrap_or_default();
    let who = std::env::var("WHO").unwrap_or_default();
    let limit: usize = std::env::var("LIMIT").ok().and_then(|d| d.parse().ok()).unwrap_or(12);
    let content = common::content();
    let town = common::town(&content, seed, 2);
    let mut w = World::from_json(&content, &town, seed as u32).unwrap();
    let end = clock::tick_at(days + 1, 8.0 * 60.0).unwrap();
    let mut last: Vec<Option<(Key, u64)>> = vec![None; w.sims.len()];
    let mut ended: Vec<Option<(Key, u64, u64)>> = vec![None; w.sims.len()];
    let mut printed = 0;
    let mut prev_desc: Vec<String> = vec![String::new(); w.sims.len()];
    while w.tick < end && printed < limit {
        for (i, d) in prev_desc.iter_mut().enumerate() {
            if w.sims[i].here() && (who.is_empty() || w.sims[i].name == who) {
                *d = describe(&w, i);
            }
        }
        w.tick_once();
        let tick = w.tick;
        last.resize(w.sims.len(), None);
        ended.resize(w.sims.len(), None);
        prev_desc.resize(w.sims.len(), String::new());
        for i in 0..w.sims.len() {
            if !w.sims[i].here() || !(who.is_empty() || w.sims[i].name == who) {
                continue;
            }
            let k = key(&w, i);
            let when = format!("day {} {:06.1}", clock::day(tick), clock::minute_of_day(tick));
            let window = |name: &str, default: f32| std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default);
            if let Ok(every) = std::env::var("WATCH")
                && clock::day(tick) == days
                && (window("FROM", 0.0)..window("TO", 1440.0)).contains(&clock::minute_of_day(tick))
                && tick.is_multiple_of(every.parse().unwrap_or(20))
            {
                let target = match w.sims[i].current().map(|a| a.task.kind) {
                    Some(TaskKind::Social { target, .. } | TaskKind::Join { leader: target }) => Some(describe(&w, target as usize)),
                    _ => None,
                };
                eprintln!("{when} WATCH {} || target: {target:?}", describe(&w, i));
            }
            match last[i] {
                Some((old, since)) if old != k => {
                    let name = kind_name(&w, old);
                    if std::env::var("LOG").is_ok() && w.tick >= clock::tick_at(days, 0.0).unwrap() {
                        eprintln!("{when} {name} ({} ticks) -> {}: {}", tick - since, kind_name(&w, k), describe(&w, i));
                    }
                    if !matches!(old, Key::Idle | Key::Away) {
                        if name.contains(&pat) && tick - since <= 20 {
                            eprintln!("{when} SHORT {name} lasted {} ticks; the tick before: {}", tick - since, prev_desc[i]);
                            printed += 1;
                        }
                        ended[i] = Some((old, since, tick));
                    }
                    if !matches!(k, Key::Idle | Key::Away)
                        && let Some((x, from, to)) = ended[i]
                        && x == k
                        && tick - to <= 20
                        && kind_name(&w, k).contains(&pat)
                    {
                        eprintln!("{when} REDO {} after {} ticks of it: {}", kind_name(&w, k), to - from, describe(&w, i));
                        printed += 1;
                    }
                    last[i] = Some((k, tick));
                }
                None => last[i] = Some((k, tick)),
                _ => {}
            }
            if let Some((k, since)) = last[i]
                && !matches!(k, Key::Idle | Key::Away)
                && (tick - since).is_multiple_of(3 * 60 * 20)
                && tick > since
                && kind_name(&w, k).contains(&pat)
            {
                eprintln!("{when} LONG {} for {} h: {}", kind_name(&w, k), (tick - since) / 1200, describe(&w, i));
                printed += 1;
            }
        }
    }
}

/// A household's people and beds at `DAYS` `FROM` (minute): `SEED`, `HOUSE`.
#[test]
#[ignore = "R3 probe: traces"]
fn house() {
    let env = |name: &str, default: f32| std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default);
    let seed = env("SEED", 7.0) as u64;
    let h = env("HOUSE", 0.0) as u32;
    let content = common::content();
    let town = common::town(&content, seed, 2);
    let mut w = World::from_json(&content, &town, seed as u32).unwrap();
    let at = clock::tick_at(env("DAYS", 10.0) as u32, env("FROM", 0.0)).unwrap();
    while w.tick < at {
        w.tick_once();
    }
    // Every household: people past the crib against bed places at home.
    let sleep = w.content.day_rhythm.sleep_tags;
    for (hh, household) in w.households.iter().enumerate() {
        let people = w.sims.iter().filter(|s| s.here() && s.household as usize == hh && !w.content.life.baby(s.age)).count();
        let places: u8 = w
            .objects
            .iter()
            .filter(|o| household.plot.is_some() && w.plot_at(o.x, o.z) == household.plot)
            .filter(|o| w.content.objects[o.def].interactions.iter().any(|i| i.tags & sleep != 0 && !i.baby))
            .map(|o| w.content.objects[o.def].slots)
            .sum();
        eprintln!("household {hh} {}: {people} past the crib, {places} bed places{}", household.name, if usize::from(places) < people { "  <-- short" } else { "" });
    }
    let plot = w.households[h as usize].plot;
    for (i, s) in w.sims.iter().enumerate().filter(|(_, s)| s.here() && s.household == h) {
        eprintln!("{}", describe(&w, i));
        let _ = s;
    }
    let sleep = w.content.day_rhythm.sleep_tags;
    for o in w.objects.iter().filter(|o| w.plot_at(o.x, o.z) == plot) {
        let def = &w.content.objects[o.def];
        if def.interactions.iter().any(|i| i.tags & sleep != 0 || i.baby) {
            eprintln!("bed-like {} #{} slots {} users {:?} wear {:.2}", def.id, o.id, def.slots, o.users, o.wear);
        }
    }
}
