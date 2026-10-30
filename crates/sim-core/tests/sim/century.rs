//! A hundred years of a town, headless: 200 game days (content `life.daysPerYear` is 2 at the
//! normal lifespan) for seeds 7, 11 and 19, with lower detail for the lots nobody looks at
//! (`lod.rs`) and without. Every game hour the world must be consistent (`common::check`) and
//! everyone must have aged exactly once at every midnight; nobody may get stuck
//! (`common::Watch`); the town keeps between 8 and 64 residents; the story has births, deaths
//! and rent; and saving and loading at a few points gives a world that keeps going.
//!
//! `cargo test --release -p sim-core --test sim century:: -- --ignored --nocapture` (one test per seed
//! and detail; `WATCH=<name>@<day>` prints what a resident does). Two days of it and tests of
//! the checks themselves run with the suite; `the_town_still_ages_after_day_1167` jumps a town
//! to day 1166 and checks a few days there.

use crate::common;

use common::Long;
use sim_core::{World, clock};

const YEARS: f32 = 100.0;
/// The days the town runs before it's saved and loaded again (at 13:37, partway through things).
const RELOADS: [u32; 3] = [50, 100, 150];

/// Runs the town of `seed` from day 1 for `days` (`None`: a hundred years at the normal lifespan)
/// with lower detail on or off, and checks it all along. Returns what it saw.
fn century(seed: u64, lod: bool, days: Option<u32>) -> Long {
    let content = common::content();
    let town = common::town(&content, seed, 3);
    let mut w = World::from_json(&content, &town, seed as u32).expect("town loads");
    let per_year = w
        .lifespan
        .days_per_year(w.content.life.days_per_year)
        .expect("residents age");
    let days = days.unwrap_or((YEARS * per_year) as u32);
    let home = w
        .households
        .iter()
        .find(|h| h.player)
        .and_then(|h| h.plot)
        .expect("the player has a home");
    let look = |w: &mut World| {
        w.lod = lod;
        if lod {
            common::look_at(w, home);
        }
    };
    look(&mut w);
    let mut long = Long::new(&w, 8..=64);
    let end = clock::tick_at(days + 1, 0.0).unwrap();
    // (A short run saves and loads on its last day.)
    let reload_days: Vec<u32> = if days < RELOADS[0] {
        vec![days]
    } else {
        RELOADS.iter().copied().filter(|&d| d <= days).collect()
    };
    let reloads: Vec<u64> = reload_days
        .iter()
        .map(|&d| clock::tick_at(d, 13.0 * 60.0 + 37.0).unwrap())
        .collect();
    let started = std::time::Instant::now();
    let label = format!(
        "seed {seed}, lower detail {}",
        if lod { "on" } else { "off" }
    );
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        while w.tick < end {
            w.tick_once();
            long.after(&w);
            if reloads.contains(&w.tick) {
                w = common::reload(&w, &content);
                look(&mut w);
            }
        }
    }));
    if run.is_err() {
        eprintln!(
            "{label}: failed on day {} at {:.2} h (tick {})",
            clock::day(w.tick),
            clock::hour(w.tick),
            w.tick
        );
    }
    let here = w.sims.iter().filter(|s| s.here()).count();
    eprintln!(
        "{label}: {} days ({:.1} years) in {:.1} s; {here} residents at the end ({}..{} seen), {} slots, {} households",
        clock::day(w.tick) - 1,
        (clock::day(w.tick) - 1) as f32 / per_year,
        started.elapsed().as_secs_f64(),
        long.fewest,
        long.most,
        w.sims.len(),
        w.households.len()
    );
    eprintln!("{label}: story {:?}", long.story);
    for p in &long.problems {
        eprintln!("{label}: {p}");
    }
    for s in &long.watch.stuck {
        eprintln!("{label}: stuck: {s}");
    }
    if let Err(e) = run {
        std::panic::resume_unwind(e);
    }
    long.assert_ok();
    long
}

/// Lives begin and end, and the rent is paid.
fn whole_story(long: &Long) {
    for kind in ["Born", "Died", "PaidRent"] {
        assert!(
            long.told(kind),
            "nobody {kind} in a hundred years: {:?}",
            long.story
        );
    }
}

#[test]
fn two_days_of_the_century() {
    century(7, true, Some(2));
}

/// A game that's run for years of play: from day 1166 everyone still ages at every midnight
/// (and the rest of the checks hold) for a few days.
#[test]
fn the_town_still_ages_after_day_1167() {
    let content = common::content();
    let town = common::town(&content, 7, 3);
    let mut w = World::from_json(&content, &town, 7).expect("town loads");
    w.tick = clock::tick_at(1166, 12.0 * 60.0).unwrap();
    let mut long = Long::new(&w, 8..=64);
    let end = clock::tick_at(1171, 0.0).unwrap();
    while w.tick < end {
        w.tick_once();
        long.after(&w);
    }
    long.assert_ok();
}

/// The checks catch what they're for: a town with one thing wrong fails `check`.
#[test]
fn the_checks_catch_a_town_gone_wrong() {
    let content = common::content();
    let town = common::town(&content, 7, 2);
    common::check(&World::from_json(&content, &town, 7).unwrap());
    type Wrong = (&'static str, fn(&mut World));
    let wrongs: [Wrong; 13] = [
        ("a need that's no number", |w| w.sims[0].needs[0] = f32::NAN),
        ("someone facing nowhere", |w| w.sims[0].yaw = f32::INFINITY),
        ("someone 151 years old", |w| w.sims[0].age = 151.0),
        ("a skill past the top", |w| w.sims[0].skills[0] = 99.0),
        ("a grade past 100", |w| w.sims[0].grade = 100.5),
        ("a mood that's no number", |w| {
            w.sims[0].needs[1] = f32::INFINITY
        }),
        ("friendship past 100", |w| {
            w.relationships.get_mut(0, 1).friendship = 150.0
        }),
        ("something worn past broken", |w| w.objects[0].wear = 1.5),
        ("dirt that's no number", |w| w.dirt[0] = f32::NAN),
        ("a billion in the bank", |w| {
            w.households[0].funds = 1_000_000_000
        }),
        ("a home on a plot there isn't", |w| {
            w.households[0].plot = Some(999)
        }),
        ("two households in one house", |w| {
            w.households[1].plot = w.households[0].plot
        }),
        ("a baby on the way from someone who isn't there", |w| {
            w.households[0].expecting = Some(sim_core::world::Expecting {
                parents: [0, 999],
                due: 3,
            })
        }),
    ];
    for (what, wrong) in wrongs {
        let mut w = World::from_json(&content, &town, 7).unwrap();
        wrong(&mut w);
        let caught =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| common::check(&w))).is_err();
        assert!(caught, "check doesn't catch {what}");
    }
}

/// The checks read a resident's follow-up from their debug text (it isn't public): after the
/// toilet it's washing hands at the sink. If this fails, the field was renamed and
/// `common::follow_up` needs to follow.
#[test]
fn the_checks_see_a_follow_up() {
    let town = r#"{"width":16,"depth":16,
        "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,13.5]}],
        "households":[{"name":"Player","plot":0,"player":true}],
        "sims":[{"name":"Ada","household":0,"x":8.5,"z":8.5,"traits":["bookworm"]}]}"#;
    let mut w = World::from_json(&common::content(), town, 1).unwrap();
    w.autonomy = false;
    for (def, x) in [("toilet", 3), ("sink", 6)] {
        let cmd = sim_core::Command::Buy {
            household: 0,
            object: def.into(),
            at: Some([x, 3, 0]),
            colors: Default::default(),
            turn: None,
            offset: None,
            size: None,
            tall: None,
        };
        w.apply(cmd).unwrap();
    }
    let (toilet, sink) = (0, 1);
    let bladder = w
        .content
        .needs
        .iter()
        .position(|n| n.id == "bladder")
        .unwrap();
    w.sims[0].needs[bladder] = 0.3;
    let interaction = w.content.objects[w.objects[toilet].def]
        .interactions
        .iter()
        .position(|i| i.id == "use")
        .unwrap();
    w.apply(sim_core::Command::Use {
        sim: 0,
        object: toilet as u32,
        interaction,
    })
    .unwrap();
    let mut next = None;
    for _ in 0..common::TICKS_PER_HOUR {
        w.tick_once();
        common::check(&w);
        if let Some(kind) = common::follow_up(&w.sims[0]) {
            next = Some(kind);
            break;
        }
    }
    let next = next.expect("a follow-up after the toilet");
    assert_eq!(common::debug_field(&next, "object"), Some(sink), "{next}");
}

/// The watch sees residents stuck: with free will off everyone stands about, and someone sent
/// away for good stays away.
#[test]
fn the_watch_sees_residents_stuck() {
    let content = common::content();
    let town = common::town(&content, 7, 2);
    let mut w = World::from_json(&content, &town, 7).unwrap();
    w.autonomy = false;
    w.sims[1].away_until = Some(u64::MAX);
    let mut watch = common::Watch::new();
    for _ in 0..17 * common::TICKS_PER_HOUR {
        w.tick_once();
        watch.sample(&w);
    }
    let seen = |who: usize, state: &str| {
        watch.stuck.iter().any(|s| {
            s.sim as usize == who && s.state.starts_with(state) && s.name == w.sims[who].name
        })
    };
    assert!(seen(0, "Idle"), "{:?}", watch.stuck);
    assert!(seen(1, "Away"), "{:?}", watch.stuck);
    let idle = watch.stuck.iter().find(|s| s.sim == 0).unwrap();
    assert!(
        (idle.tick - idle.since) as f32 * sim_core::MINUTES_PER_TICK > 6.0 * 60.0,
        "after 6 hours: {idle}"
    );
}

#[test]
#[ignore = "long: run with --release -- --ignored"]
fn a_century_seed_7() {
    whole_story(&century(7, false, None));
}

#[test]
#[ignore = "long: run with --release -- --ignored"]
fn a_century_seed_7_lower_detail() {
    whole_story(&century(7, true, None));
}

#[test]
#[ignore = "long: run with --release -- --ignored"]
fn a_century_seed_11() {
    whole_story(&century(11, false, None));
}

#[test]
#[ignore = "long: run with --release -- --ignored"]
fn a_century_seed_11_lower_detail() {
    whole_story(&century(11, true, None));
}

#[test]
#[ignore = "long: run with --release -- --ignored"]
fn a_century_seed_19() {
    whole_story(&century(19, false, None));
}

#[test]
#[ignore = "long: run with --release -- --ignored"]
fn a_century_seed_19_lower_detail() {
    whole_story(&century(19, true, None));
}
