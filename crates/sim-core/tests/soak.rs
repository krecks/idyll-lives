//! Long runs of a whole town with free will on and nobody giving orders: the "aquarium". The
//! world must stay consistent hour after hour, residents must keep themselves going, and the
//! story must keep moving.
//!
//! `cargo test --release --test soak -- --ignored` runs the long version; with `LOD=1` the
//! player's home is looked at and the rest of the town runs at lower detail (`lod.rs`).
//!
//! The busy player runs are the opposite: a big town with someone at the controls every few
//! game hours (buying, selling and moving things at home, giving orders, changing the speed and
//! looking around town), with the checks of the century runs (`century.rs`).

mod common;

use sim_core::command::ItemAction;
use sim_core::social::EventKind;
use sim_core::{Command, MINUTES_PER_TICK, World, clock, life};

const TICKS_PER_HOUR: u64 = (60.0 / MINUTES_PER_TICK) as u64;

/// What happened over a whole run, by event kind (the world's log only keeps the latest).
type Story = std::collections::BTreeMap<String, usize>;

/// Per-resident record of how the run went.
#[derive(Default, Clone)]
struct Stats {
    need_sum: f64,
    samples: u32,
    lowest: f32,
    /// Ticks in a row with nothing to do while awake and not at work.
    idle: u64,
    /// Longest such stretch, in game minutes.
    worst_idle: f32,
}

fn run(days: u32, seed: u64, player_size: usize) -> (World, Vec<Stats>, Story) {
    let content = common::content();
    let town = common::town(&content, seed, player_size);
    let mut w = World::from_json(&content, &town, seed as u32).expect("town loads");
    common::lod_from_env(&mut w);
    let mut stats = vec![
        Stats {
            lowest: 1.0,
            ..Default::default()
        };
        w.sims.len()
    ];
    let end = clock::tick_at(days + 1, 8.0 * 60.0).unwrap();
    let started = std::time::Instant::now();
    let mut story = Story::new();
    while w.tick < end {
        let hour = w.tick;
        for _ in 0..TICKS_PER_HOUR {
            w.tick_once();
            let waking = (9.0..21.0).contains(&clock::hour(w.tick));
            // Babies are born and newcomers arrive: their records start now.
            if stats.len() < w.sims.len() {
                stats.resize(w.sims.len(), Stats { lowest: 1.0, ..Default::default() });
            }
            for (i, s) in w.sims.iter().enumerate() {
                let st = &mut stats[i];
                let busy = s.current().is_some() || s.engaged_with.is_some();
                if waking && s.away_until.is_none() && !busy {
                    st.idle += 1;
                    st.worst_idle = st.worst_idle.max(st.idle as f32 * MINUTES_PER_TICK);
                } else {
                    st.idle = 0;
                }
            }
        }
        for e in w.events.iter().filter(|e| e.tick > hour) {
            *story.entry(format!("{:?}", e.kind)).or_default() += 1;
        }
        common::check(&w);
        let n = w.content.needs.len();
        for (i, s) in w.sims.iter().enumerate() {
            let st = &mut stats[i];
            let avg = s.needs[..n].iter().sum::<f32>() / n as f32;
            st.need_sum += avg as f64;
            st.samples += 1;
            st.lowest = st.lowest.min(avg);
        }
    }
    let secs = started.elapsed().as_secs_f64();
    eprintln!(
        "{} residents, {days} days: {:.0} ticks/s ({secs:.1} s)",
        w.sims.len(),
        w.tick as f64 / secs
    );
    (w, stats, story)
}

fn report(w: &World, stats: &[Stats]) {
    for (i, s) in w.sims.iter().enumerate() {
        let st = &stats[i];
        let job = s.job.as_ref().map_or("-".to_string(), |j| {
            w.content.careers[j.career].levels[j.level].title.clone()
        });
        eprintln!(
            "{:>4} h{} avg {:.2} low {:.2} idle {:.0} min  {}",
            s.name,
            s.household,
            st.need_sum / st.samples as f64,
            st.lowest,
            st.worst_idle,
            job
        );
    }
    for (h, hh) in w.households.iter().enumerate() {
        let weekly = life::weekly_costs(w, h).map_or(0, |(r, b)| r + b);
        eprintln!("{}: funds {} (weekly costs {weekly})", hh.name, hh.funds);
        // How the home's rooms are doing (cleanliness and overall), worst first.
        let mut rooms: Vec<_> = w.rooms().iter().filter(|r| !r.garden && r.plot == hh.plot).collect();
        rooms.sort_by(|a, b| a.scores.overall.total_cmp(&b.scores.overall));
        let line: Vec<String> = rooms
            .iter()
            .map(|r| {
                let kind = r.kind.map_or("?", |k| w.content.room_kinds[k].id.as_str());
                format!("{kind} {:.2} (clean {:.2})", r.scores.overall, r.scores.clean)
            })
            .collect();
        let broken: Vec<&str> = w
            .objects
            .iter()
            .filter(|o| o.broken() && hh.plot.is_some() && w.plot_at(o.x, o.z) == hh.plot)
            .map(|o| w.content.objects[o.def].id.as_str())
            .collect();
        eprintln!("    rooms: {}; broken now: {broken:?}", line.join(", "));
        let wishes: Vec<String> = w
            .sims
            .iter()
            .filter(|s| s.household as usize == h)
            .flat_map(|s| s.planner.home_wishes.iter().map(|wish| format!("{wish:?}")))
            .collect();
        eprintln!("    home wishes: {wishes:?}");
    }
    let mut kinds: std::collections::BTreeMap<String, usize> = Default::default();
    for e in w.events.iter() {
        *kinds.entry(format!("{:?}", e.kind)).or_default() += 1;
    }
    eprintln!("story: {kinds:?}");
    let mut accidents: std::collections::BTreeMap<&str, usize> = Default::default();
    for e in w.events.iter().filter(|e| e.kind == sim_core::social::EventKind::Accident) {
        *accidents.entry(w.content.accidents[e.n.unwrap_or(0) as usize].id.as_str()).or_default() += 1;
    }
    eprintln!("accidents (last {} story events): {accidents:?}", w.events.iter().count());
}

#[test]
fn a_town_lives_on_its_own_for_two_weeks() {
    let (w, stats, _) = run(14, 7, 2);
    report(&w, &stats);
    for (i, s) in w.sims.iter().enumerate() {
        let st = &stats[i];
        let avg = st.need_sum / st.samples as f64;
        assert!(avg > 0.4, "{} lived badly: average needs {avg:.2}", s.name);
        assert!(
            st.worst_idle <= 60.0,
            "{} stood around for {:.0} minutes",
            s.name,
            st.worst_idle
        );
    }
    // The newcomers found work on their own (the grown-ups: a baby may have come along).
    let player: Vec<_> = w
        .sims
        .iter()
        .filter(|s| w.households[s.household as usize].player && s.adult(&w.content))
        .collect();
    assert!(!player.is_empty());
    assert!(
        player.iter().all(|s| s.job.is_some()),
        "the player's household is still looking for work"
    );
    assert!(w.events.iter().any(|e| e.kind == EventKind::JobFound));
    // Working households don't sink into debt.
    for (h, hh) in w.households.iter().enumerate() {
        let working = w
            .sims
            .iter()
            .any(|s| s.household as usize == h && s.job.is_some());
        if working {
            assert!(hh.funds > 0, "{} went into debt: {}", hh.name, hh.funds);
        }
    }
    // The story keeps moving: something notable happened at the player's home in the second week.
    let week_two = clock::tick_at(8, 0.0).unwrap();
    // (As the journal's "Our home": events naming one of them in any role.)
    let ours = |id: u32| w.households[w.sims[id as usize].household as usize].player;
    let about_us = |e: &&sim_core::social::SocialEvent| [Some(e.a), Some(e.b), e.c].into_iter().flatten().any(ours);
    // Something besides the rent in the second week (a quiet week is skill-ups and visits)...
    assert!(
        w.events.iter().filter(about_us).any(|e| e.tick >= week_two && e.kind != EventKind::PaidRent),
        "nothing happened to the player's household in week two"
    );
    // ...and something notable in the two.
    assert!(
        w.events.iter().filter(about_us).any(|e| e.kind.importance() >= 1),
        "nothing notable happened to the player's household"
    );
}

#[test]
fn same_seed_same_town() {
    let (a, ..) = run(2, 3, 2);
    let (b, ..) = run(2, 3, 2);
    assert_eq!(a.save_json(), b.save_json());
}

#[test]
fn saving_and_loading_mid_run_keeps_the_town() {
    let content = common::content();
    let town = common::town(&content, 5, 2);
    let mut w = World::from_json(&content, &town, 5).unwrap();
    for _ in 0..(TICKS_PER_HOUR * 30) {
        w.tick_once();
    }
    let json = w.save_json();
    let loaded = World::from_save_json(&content, &json).unwrap();
    common::check(&loaded);
    assert_eq!(loaded.sims.len(), w.sims.len());
    assert_eq!(
        loaded.events.iter().map(|e| e.id).collect::<Vec<_>>(),
        w.events.iter().map(|e| e.id).collect::<Vec<_>>(),
        "the story survives saving"
    );
    let jobs = |w: &World| w.sims.iter().map(|s| s.job.clone()).collect::<Vec<_>>();
    assert_eq!(jobs(&loaded), jobs(&w));
}

#[test]
#[ignore = "long: run with --release -- --ignored"]
fn a_town_lives_on_its_own_for_two_months() {
    let (w, stats, _) = run(60, 11, 3);
    report(&w, &stats);
    // Nobody's life collapses (crowded houses do wear people down).
    for (i, s) in w.sims.iter().enumerate() {
        let avg = stats[i].need_sum / stats[i].samples as f64;
        assert!(avg > 0.3, "{} lived badly: average needs {avg:.2}", s.name);
    }
}

/// A whole stretch of life at the normal pace (150 days: 75 years): people grow old, retire and
/// die, children move out, partners move in, and newcomers fill the houses left empty.
#[test]
#[ignore = "long: run with --release -- --ignored"]
fn a_town_over_a_lifetime() {
    let (w, _, story) = run(150, 7, 3);
    let here: Vec<_> = w.sims.iter().filter(|s| s.here()).collect();
    let mut kinds: std::collections::BTreeMap<String, usize> = Default::default();
    for e in w.events.iter() {
        *kinds.entry(format!("{:?}", e.kind)).or_default() += 1;
    }
    eprintln!("after 150 days: {} residents here, {} slots, {} former names kept", here.len(), w.sims.len(), w.former.len());
    eprintln!("ages: {:?}", here.iter().map(|s| s.age.floor() as u32).collect::<Vec<_>>());
    let lived = w.households.iter().enumerate().filter(|(h, _)| here.iter().any(|s| s.household as usize == *h)).count();
    eprintln!("households with someone: {lived} of {}", w.households.len());
    eprintln!("story (last {} events): {kinds:?}", w.events.iter().count());
    let broken: Vec<&str> = w.objects.iter().filter(|o| o.broken()).map(|o| w.content.objects[o.def].id.as_str()).collect();
    eprintln!("broken now: {broken:?}");
    assert!(here.len() >= 8, "the town doesn't empty out");
    assert!(w.sims.len() <= sim_core::world::MAX_SIMS);
    eprintln!("whole run: {story:?}");
    assert!(story.contains_key("Died") && story.contains_key("MovedIn"), "lives end and new ones begin");
}

// ---- A busy player in a big town -------------------------------------------------------------

/// As big a town as `big_town.rs`: 34 houses on two streets with furnished yards (thousands of
/// objects), and as many residents as a town holds.
const BIG: common::Layout = common::Layout { columns: 9, streets: 2, yards: true, max_residents: sim_core::world::MAX_SIMS };

/// Someone at the controls: every few game hours a session of orders, or of edits at home
/// (paused, as build mode is), from a seeded random number generator.
struct Player {
    rng: common::Lcg,
    household: u32,
    /// The plot being looked at (the game's camera; lower detail is for the rest).
    view: u32,
    next: u64,
    /// Per command: tried, and done (the world said yes).
    tried: std::collections::BTreeMap<&'static str, (usize, usize)>,
    /// Residents whose next thing or current use names another kind of thing after an edit.
    stale: Vec<String>,
}

impl Player {
    fn new(seed: u64, w: &World) -> Self {
        let household = w.households.iter().position(|h| h.player).expect("a player household") as u32;
        let view = w.households[household as usize].plot.expect("the player has a home");
        Player { rng: common::Lcg::new(seed ^ 0x5eed), household, view, next: 0, tried: Default::default(), stale: Vec::new() }
    }

    fn pick<T: Copy>(&mut self, list: &[T]) -> Option<T> {
        (!list.is_empty()).then(|| list[self.rng.below(list.len())])
    }

    fn chance(&mut self, p: f64) -> bool {
        self.rng.next() < p
    }

    fn apply(&mut self, w: &mut World, what: &'static str, cmd: Command) {
        let edit = cmd.home_edit().is_some() || matches!(cmd, Command::Undo { .. } | Command::Redo { .. });
        let before = edit.then(|| uses(w));
        let ok = w.apply(cmd.clone()).is_ok();
        let e = self.tried.entry(what).or_default();
        e.0 += 1;
        e.1 += usize::from(ok);
        let Some(before) = before else { return };
        for (i, (was, now)) in before.iter().zip(uses(w)).enumerate() {
            for (k, label) in ["next thing", "current use"].into_iter().enumerate() {
                let (Some((o, Some(def))), Some((o2, def2))) = (was[k], now[k]) else { continue };
                if def2 != Some(def) {
                    let name = |d: Option<usize>| d.map_or("nothing".to_string(), |d| w.content.objects[d].id.clone());
                    self.stale.push(format!(
                        "day {} {:.2}h: {}'s {label} was object {o} ({}); after {cmd:?} it's object {o2} ({})",
                        clock::day(w.tick),
                        clock::hour(w.tick),
                        w.sims[i].name,
                        name(Some(def)),
                        name(def2)
                    ));
                }
            }
        }
    }

    /// The things at home (not the stairs: people upstairs need them).
    fn at_home(&self, w: &World) -> Vec<u32> {
        let Some(plot) = w.households[self.household as usize].plot else { return vec![] };
        w.objects.iter().filter(|o| w.plot_at(o.x, o.z) == Some(plot) && !w.content.objects[o.def].stairs).map(|o| o.id).collect()
    }

    fn session(&mut self, w: &mut World) {
        let h = self.household;
        let mine: Vec<u32> = w.sims.iter().filter(|s| s.here() && s.household == h).map(|s| s.id).collect();
        let everyone: Vec<u32> = w.sims.iter().filter(|s| s.here()).map(|s| s.id).collect();
        let plot = w.households[h as usize].plot;
        if let Some(home) = plot.filter(|_| self.chance(0.4)) {
            // Build and buy mode: time stands still while the home changes.
            self.apply(w, "pause", Command::SetSpeed { speed: 0 });
            for _ in 0..1 + self.rng.below(5) {
                let things = self.at_home(w);
                let Some(object) = self.pick(&things) else { break };
                let o = w.objects[object as usize].clone();
                let roll = self.rng.below(100);
                if roll < 20 {
                    let funds = w.households[h as usize].funds;
                    let priced: Vec<usize> = (0..w.content.objects.len()).filter(|&d| w.content.objects[d].price.is_some_and(|p| p <= funds)).collect();
                    if let Some(d) = self.pick(&priced) {
                        let object = w.content.objects[d].id.clone();
                        self.apply(w, "buy", Command::Buy { household: h, object, at: None, style: None, turn: None, offset: None, size: None, tall: None });
                    }
                } else if roll < 40 {
                    self.apply(w, "sell", Command::Sell { household: h, object });
                } else if roll < 65 {
                    // Picked up and put down where it was, or turned.
                    let rot = if self.chance(0.5) { o.rot } else { self.rng.below(4) as u8 };
                    self.apply(w, "move in place", Command::MoveObject { household: h, object, x: o.x, z: o.z, rot, turn: None, offset: None, size: None, tall: None });
                } else if roll < 80 {
                    let p = w.plots[home as usize].clone();
                    let (x, z) = (p.x + self.rng.below(p.w as usize) as i32, p.z + self.rng.below(p.d as usize) as i32);
                    let rot = self.rng.below(4) as u8;
                    self.apply(w, "move", Command::MoveObject { household: h, object, x, z, rot, turn: None, offset: None, size: None, tall: None });
                } else if roll < 85 {
                    self.apply(w, "upgrade", Command::Upgrade { household: h, object });
                } else if roll < 90 {
                    self.apply(w, "repair", Command::Repair { household: h, object });
                } else if roll < 95 {
                    let style = self.rng.below(3) as u8;
                    self.apply(w, "restyle", Command::Restyle { household: h, object, style });
                } else {
                    self.apply(w, "undo", Command::Undo { household: h });
                }
            }
            let speed = 1 + self.rng.below(5) as u8;
            self.apply(w, "speed", Command::SetSpeed { speed });
        } else {
            // Live mode: orders for the household.
            for _ in 0..1 + self.rng.below(4) {
                let Some(sim) = self.pick(&mine) else { break };
                let roll = self.rng.below(100);
                if roll < 35 {
                    // Something at home or in the park.
                    let mut usable = self.at_home(w);
                    usable.extend(w.objects.iter().filter(|o| w.plot_at(o.x, o.z).is_some_and(|p| w.plots[p as usize].public)).map(|o| o.id));
                    let Some(object) = self.pick(&usable) else { continue };
                    let count = w.content.objects[w.objects[object as usize].def].interactions.len();
                    if count > 0 {
                        let interaction = self.rng.below(count);
                        self.apply(w, "use", Command::Use { sim, object, interaction });
                    }
                } else if roll < 45 {
                    let p = w.plots[self.rng.below(w.plots.len())].clone();
                    let (x, z) = (p.x as f32 + self.rng.next() as f32 * p.w as f32, p.z as f32 + self.rng.next() as f32 * p.d as f32);
                    self.apply(w, "walk", Command::MoveTo { sim, x, z });
                } else if roll < 60 {
                    let others: Vec<u32> = everyone.iter().copied().filter(|&t| t != sim).collect();
                    if let Some(target) = self.pick(&others) {
                        let social = self.rng.below(w.content.socials.len());
                        self.apply(w, "social", Command::Social { sim, target, social });
                    }
                } else if roll < 68 {
                    let plot = self.rng.below(w.plots.len()) as u32;
                    self.apply(w, "visit", Command::Visit { sim, plot });
                } else if roll < 74 {
                    self.apply(w, "go home", Command::GoHome { sim });
                } else if roll < 82 && !w.content.phone.actions.is_empty() {
                    let action = self.rng.below(w.content.phone.actions.len());
                    let contact = self.pick(&everyone).filter(|&c| c != sim);
                    self.apply(w, "phone", Command::Phone { sim, action, contact });
                } else if roll < 88 {
                    let items: Vec<u32> = w.items.list.iter().filter(|it| it.household == h).map(|it| it.id).collect();
                    if let Some(item) = self.pick(&items) {
                        let action = if self.chance(0.5) { ItemAction::Eat } else { ItemAction::Clear };
                        self.apply(w, "item", Command::UseItem { sim, item, action });
                    }
                } else if roll < 95 {
                    let index = self.rng.below(3);
                    self.apply(w, "cancel", Command::Cancel { sim, index });
                } else {
                    let enabled = self.chance(0.5);
                    self.apply(w, "skip quiet hours", Command::SetAutoFast { enabled });
                }
            }
            if self.chance(0.3) {
                let speed = 1 + self.rng.below(5) as u8;
                self.apply(w, "speed", Command::SetSpeed { speed });
            }
        }
        // The camera: mostly at home, now and then another lot.
        if self.chance(0.2) {
            self.view = match plot.filter(|_| self.chance(0.5)) {
                Some(home) => home,
                None => self.rng.below(w.plots.len()) as u32,
            };
            common::look_at(w, self.view);
        }
        // An hour or four until the next.
        self.next = w.tick + common::TICKS_PER_HOUR * (1 + self.rng.below(4) as u64);
    }
}

/// What a resident's next thing (the follow-up) and their current use name: the object and its
/// kind (None: there's no such object).
type Uses = [Option<(u32, Option<usize>)>; 2];

/// Everyone's `Uses`, by slot.
fn uses(w: &World) -> Vec<Uses> {
    let def = |o: u32| w.objects.get(o as usize).map(|o| o.def);
    w.sims
        .iter()
        .map(|s| {
            let next = common::follow_up(s).and_then(|k| common::debug_field(&k, "object")).map(|o| (o, def(o)));
            let now = match s.current().map(|a| a.task.kind) {
                Some(sim_core::world::TaskKind::Use { object, .. }) => Some((object, def(object))),
                _ => None,
            };
            [next, now]
        })
        .collect()
}

/// A big town for `days` with a busy player, checked as the century runs are; saved and loaded
/// twice on the way.
fn busy(days: u32, seed: u64) {
    let content = common::content();
    let town = common::town_with(&content, seed, 3, BIG);
    let mut w = World::from_json(&content, &town, seed as u32).expect("the big town loads");
    let mut player = Player::new(seed, &w);
    common::look_at(&mut w, player.view);
    let mut long = common::Long::new(&w, 8..=sim_core::world::MAX_SIMS);
    // The streets' back yards touch, so getting from one street to the other goes round the
    // end of the row: across town and over is some 300 m, over 4 hours for a tired walker.
    long.watch = common::Watch::new().routing_hours(6.0);
    let end = clock::tick_at(days + 1, 0.0).unwrap();
    let mut reloads: Vec<u64> = [days / 3, 2 * days / 3].iter().map(|&d| clock::tick_at(d, 13.0 * 60.0 + 37.0).unwrap()).collect();
    let started = std::time::Instant::now();
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        while w.tick < end {
            w.advance();
            long.after(&w);
            if w.tick >= player.next {
                // (Checked on the hour, after a tick: some of what an order changes settles
                // on the next one, like putting down what someone holds.)
                player.session(&mut w);
                // (Build mode ends with the session: time never stands still for long.)
                if w.speed == 0 {
                    w.apply(Command::SetSpeed { speed: 1 }).unwrap();
                }
            }
            if reloads.first().is_some_and(|&t| w.tick >= t) {
                reloads.remove(0);
                w = common::reload(&w, &content);
                common::look_at(&mut w, player.view);
            }
        }
    }));
    let label = format!("busy player, seed {seed}");
    if run.is_err() {
        eprintln!("{label}: failed on day {} at {:.2} h (tick {})", clock::day(w.tick), clock::hour(w.tick), w.tick);
    }
    eprintln!(
        "{label}: {} days in {:.1} s; {} residents at the end ({}..{} seen), {} objects",
        clock::day(w.tick) - 1,
        started.elapsed().as_secs_f64(),
        w.sims.iter().filter(|s| s.here()).count(),
        long.fewest,
        long.most,
        w.objects.len()
    );
    eprintln!("{label}: commands (tried, done) {:?}", player.tried);
    eprintln!("{label}: story {:?}", long.story);
    for s in &player.stale {
        eprintln!("{label}: stale: {s}");
    }
    for p in &long.problems {
        eprintln!("{label}: {p}");
    }
    for s in &long.watch.stuck {
        eprintln!("{label}: stuck: {s}");
    }
    if let Err(e) = run {
        std::panic::resume_unwind(e);
    }
    assert!(player.stale.is_empty(), "{} times a resident's next thing or current use named another kind of thing after an edit", player.stale.len());
    long.assert_ok();
    for kind in ["Born", "Died", "PaidRent"] {
        assert!(long.told(kind), "nobody {kind}: {:?}", long.story);
    }
}

#[test]
#[ignore = "long (--release -- --ignored); fails on main until R2-01 (a stale follow-up panics after furniture moves, day 6), then on T1-01 (in a full town a couple tries for a baby back to back, 2 h)"]
fn a_big_town_with_a_busy_player_for_two_months() {
    busy(60, 7);
}

#[test]
#[ignore = "long (--release -- --ignored); fails on main until R2-01 (stale follow-ups), R3-03 (chasing a departing guest) and R3-08 (newborns half a year old)"]
fn a_big_town_with_a_busy_player_over_a_lifetime() {
    busy(150, 11);
}
