//! How residents behave, in small scenes and a long run (from the 0.2.0 release review). Each
//! proof keeps one bug fixed: someone too tired to talk leaves others be, nobody follows someone
//! across town for a chat, practising doesn't stutter, newborns are born at age 0.
//! `long_run_probe` (ignored, a diagnostic) runs a whole town and logs non-finite numbers, needs
//! out of range, the longest stretch in one task and residents flipping between two things;
//! `trace` and `house` print the stories behind them.
//!
//! `cargo test --release -p sim-core --test behaviour -- --ignored --nocapture <name>`
//! (`long_run_probe`: `DAYS=60 SEEDS=7,11` by default; `LOD=1` looks at the player's home).

mod common;

use std::collections::{BTreeMap, VecDeque};

use sim_core::social::EventKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{World, clock};

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
    let lot = format!(
        r#"{{"width":{width},"depth":12,"objects":[{{"def":"book","x":8,"z":3,"rot":0}}],"sims":[{sims}]}}"#
    );
    World::from_json(TALK, &lot, 5).unwrap()
}

#[test]
fn someone_too_tired_to_talk_leaves_others_be() {
    let mut w = talk_town(
        16,
        r#"{"name":"Ann","x":4.5,"z":5.5},{"name":"Bob","x":8.5,"z":5.5}"#,
    );
    // Ann is about to drop (pressed: a chat would end at once) and lonely; Bob wants to read.
    let e: f32 = std::env::var("ENERGY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.05);
    w.sims[0].needs[..3].copy_from_slice(&[e, 0.1, 1.0]);
    w.sims[1].needs[..3].copy_from_slice(&[1.0, 1.0, 0.2]);
    // Chats Bob is pulled into that end straight away (his reading dropped for nothing).
    let (mut cut_short, mut since) = (0, None);
    for t in 0..20 * 30 {
        w.tick_once();
        if std::env::var("SHOW").is_ok() && t % 20 == 0 {
            eprintln!(
                "{t}: Ann {:?} | Bob {:?} engaged {:?}",
                w.sims[0]
                    .current()
                    .map(|a| (a.task.kind, phase_name(&a.phase))),
                w.sims[1]
                    .current()
                    .map(|a| (a.task.kind, phase_name(&a.phase))),
                w.sims[1].engaged_with
            );
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
    eprintln!(
        "Bob was pulled into {cut_short} chats that ended at once in 30 minutes; his fun is {:.2}",
        w.sims[1].needs[2]
    );
    assert!(
        cut_short <= 1,
        "Bob was pulled into {cut_short} chats that ended at once"
    );
}

#[test]
fn told_to_chat_while_about_to_drop_they_leave_the_other_be() {
    let mut w = talk_town(
        16,
        r#"{"name":"Ann","x":4.5,"z":5.5},{"name":"Bob","x":8.5,"z":5.5}"#,
    );
    w.apply(sim_core::Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w.sims[0].needs[..3].copy_from_slice(&[0.05, 0.1, 1.0]);
    w.apply(sim_core::Command::Use {
        sim: 1,
        object: 0,
        interaction: 0,
    })
    .unwrap();
    for _ in 0..20 * 3 {
        w.tick_once();
    }
    assert!(
        matches!(
            w.sims[1].current().map(|a| &a.phase),
            Some(Phase::Using { .. })
        ),
        "Bob is reading"
    );
    w.apply(sim_core::Command::Social {
        sim: 0,
        target: 1,
        social: 0,
    })
    .unwrap();
    for _ in 0..20 * 5 {
        w.tick_once();
        assert!(
            w.sims[1].engaged_with.is_none(),
            "Bob's reading was dropped for a chat that ends at once"
        );
    }
    assert!(w.sims[0].current().is_none(), "Ann gave up on the chat");
    assert!(
        matches!(
            w.sims[1].current().map(|a| &a.phase),
            Some(Phase::Using { .. })
        ),
        "Bob reads on"
    );
}

#[test]
fn nobody_follows_someone_over_the_fence_for_a_chat() {
    // Bob, from next door, is in Ann's garden; he heads home to watch TV as lonely Ann (within
    // earshot) calls out to him.
    let content = r#"{
        "needs":[{"id":"social","label":"Social","decayPerHour":0.1},{"id":"fun","label":"Fun","decayPerHour":0.05}],
        "objects":[{"id":"tv","name":"TV","interactions":[{"id":"watch","label":"Watch","minutes":60,"effects":{"fun":0.6}}]}],
        "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},"needs":{"social":0.4}}],
        "bondPresets":{"friends":{"friendship":40}}}"#;
    let lot = r#"{"width":40,"depth":14,
        "plots":[{"name":"West","x":0,"z":0,"w":20,"d":10,"entry":[10.5,9.5]},{"name":"East","x":20,"z":0,"w":20,"d":10,"entry":[30.5,9.5]}],
        "households":[{"name":"Ann's","plot":0,"player":true},{"name":"Bob's","plot":1}],
        "objects":[{"def":"tv","x":34,"z":2,"rot":0}],
        "sims":[{"name":"Ann","household":0,"x":10.5,"z":5.5},{"name":"Bob","household":1,"x":17.5,"z":5.5}],
        "relationships":[{"a":0,"b":1,"preset":"friends"}]}"#;
    let mut w = World::from_json(content, lot, 5).unwrap();
    w.sims[0].needs[..2].copy_from_slice(&[0.1, 1.0]);
    w.sims[1].needs[..2].copy_from_slice(&[1.0, 0.1]);
    let (mut went_after, mut talked) = (false, false);
    for _ in 0..20 * 40 {
        w.tick_once();
        went_after |= matches!(
            w.sims[0].current().map(|a| a.task.kind),
            Some(TaskKind::Social { target: 1, .. })
        );
        talked |= w.sims[1].engaged_with == Some(0);
        let (x, z) = w.sims[0].tile();
        assert_ne!(
            w.plot_at(x, z),
            Some(1),
            "Ann followed Bob into his garden for a chat"
        );
    }
    assert!(went_after, "Ann never went to chat with Bob");
    // (He turned when she called him, and they met on her side.)
    assert!(talked, "Bob never came over to talk");
}

#[test]
fn someone_calls_a_housemate_and_they_meet_halfway() {
    let mut w = talk_town(
        24,
        r#"{"name":"Ann","x":3.5,"z":5.5},{"name":"Bob","x":11.5,"z":5.5}"#,
    );
    w.sims[0].needs[..3].copy_from_slice(&[1.0, 0.1, 1.0]);
    w.sims[1].needs[..3].copy_from_slice(&[1.0, 1.0, 1.0]);
    let (mut called, mut heeded) = (false, false);
    for _ in 0..20 * 30 {
        w.tick_once();
        let phase = w.sims[0]
            .current()
            .map(|a| (a.task.kind, phase_name(&a.phase)));
        called |= matches!(phase, Some((TaskKind::Social { target: 1, .. }, "Calling")));
        heeded |= w.sims[1].heeding == Some(0);
        if matches!(phase, Some((_, "Conversing"))) {
            break;
        }
    }
    assert!(called, "Ann never called Bob");
    assert!(heeded, "Bob never turned to Ann");
    assert!(w.sims[1].engaged_with == Some(0), "they never talked");
    // Both walked, neither all the way.
    let (ann, bob) = (w.sims[0].pos[0], w.sims[1].pos[0]);
    eprintln!("They met at {ann:.1} and {bob:.1}");
    assert!(ann > 4.5 && bob < 10.5, "Ann at {ann:.1}, Bob at {bob:.1}");
}

#[test]
fn nobody_chats_with_someone_out_of_earshot() {
    let mut w = talk_town(
        48,
        r#"{"name":"Ann","x":3.5,"z":5.5},{"name":"Bob","x":30.5,"z":5.5}"#,
    );
    w.sims[0].needs[..3].copy_from_slice(&[1.0, 0.1, 1.0]);
    w.sims[1].needs[..3].copy_from_slice(&[1.0, 1.0, 1.0]);
    for _ in 0..20 * 60 {
        w.tick_once();
        let social = matches!(
            w.sims[0].current().map(|a| a.task.kind),
            Some(TaskKind::Social { .. })
        );
        assert!(!social, "Ann set off across the lot to chat with Bob");
    }
}

#[test]
fn told_to_chat_from_afar_they_walk_until_in_earshot() {
    let mut w = talk_town(
        60,
        r#"{"name":"Ann","x":3.5,"z":5.5},{"name":"Bob","x":40.5,"z":5.5}"#,
    );
    w.apply(sim_core::Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w.apply(sim_core::Command::Social {
        sim: 0,
        target: 1,
        social: 0,
    })
    .unwrap();
    let phase = |w: &World| w.sims[0].current().map(|a| phase_name(&a.phase));
    for _ in 0..20 * 60 {
        w.tick_once();
        if phase(&w) == Some("Calling") {
            break;
        }
    }
    assert_eq!(phase(&w), Some("Calling"), "Ann never called Bob");
    let gap = w.sims[1].pos[0] - w.sims[0].pos[0];
    assert!(
        gap > 2.0 && w.within_earshot(w.sims[0].pos, w.sims[1].pos),
        "Ann called from {gap:.1} m"
    );
    for _ in 0..20 * 60 {
        w.tick_once();
        if phase(&w) == Some("Conversing") {
            break;
        }
    }
    assert_eq!(phase(&w), Some("Conversing"), "they never talked");
    assert!(w.sims[1].pos[0] < 39.5, "Bob never came to meet Ann");
}

#[test]
fn nobody_follows_someone_across_town_for_a_chat() {
    let mut w = talk_town(
        80,
        r#"{"name":"Ann","x":4.5,"z":5.5},{"name":"Bob","x":6.5,"z":5.5}"#,
    );
    w.sims[0].needs[..3].copy_from_slice(&[1.0, 0.1, 1.0]);
    w.sims[1].needs[..3].copy_from_slice(&[1.0, 1.0, 1.0]);
    // Ann decides to chat with Bob...
    let chatting = |w: &World| {
        matches!(
            w.sims[0].current().map(|a| a.task.kind),
            Some(TaskKind::Social { target: 1, .. })
        )
    };
    for _ in 0..20 * 10 {
        if chatting(&w) {
            break;
        }
        w.tick_once();
    }
    assert!(chatting(&w), "Ann never went to chat with Bob");
    // ...and Bob is told to walk to the far end and back (busy: not someone to talk to).
    for x in [75.5, 3.5, 75.5] {
        w.apply(sim_core::Command::MoveTo { sim: 1, x, z: 5.5 })
            .unwrap();
    }
    let mut minutes = 0;
    while chatting(&w) && minutes < 120 {
        for _ in 0..20 {
            w.tick_once();
        }
        minutes += 1;
    }
    eprintln!(
        "Ann followed Bob for {minutes} minutes; she's at {:.1}",
        w.sims[0].pos[0]
    );
    assert!(minutes <= 15, "Ann followed Bob for {minutes} minutes");
}

/// A TV and a toilet; the bladder is urgent, and a programme is long enough to run it out.
const PRESSED: &str = r#"{
    "needs":[{"id":"fun","label":"Fun","decayPerHour":0.05},{"id":"bladder","label":"Bladder","decayPerHour":0.1},
             {"id":"energy","label":"Energy","decayPerHour":0.05}],
    "autonomy":{"urgent":["bladder"],"urgentBelow":0.1},
    "objects":[{"id":"tv","name":"TV","interactions":[
        {"id":"watch","label":"Watch","minutes":60,"effects":{"fun":0.8}}]},
      {"id":"toilet","name":"Toilet","interactions":[
        {"id":"use","label":"Use","minutes":5,"effects":{"bladder":1.0}}]}]}"#;

#[test]
fn an_urgent_need_doesnt_choose_a_programme_it_would_stop_at_once() {
    let lot = r#"{"width":12,"depth":12,
        "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[5.5,11.5]}],
        "households":[{"name":"Home","plot":0,"player":true}],
        "objects":[{"def":"tv","x":2,"z":2,"rot":0},{"def":"toilet","x":8,"z":2,"rot":0}],
        "sims":[{"name":"Ann","x":5.5,"z":7.5}]}"#;
    let mut w = World::from_json(PRESSED, lot, 5).unwrap();
    // Bored and about to burst: the TV is tempting, but watching it would be cut off at once.
    w.sims[0].needs[..3].copy_from_slice(&[0.0, 0.05, 1.0]);
    let (mut tv_starts, mut toilet, mut was) = (0, false, false);
    for _ in 0..20 * 30 {
        w.tick_once();
        let using_tv = matches!(
            w.sims[0].current().map(|a| (&a.task.kind, &a.phase)),
            Some((TaskKind::Use { object: 0, .. }, Phase::Using { .. }))
        );
        tv_starts += usize::from(using_tv && !was);
        was = using_tv;
        toilet |= matches!(
            w.sims[0].current().map(|a| (&a.task.kind, &a.phase)),
            Some((TaskKind::Use { object: 1, .. }, Phase::Using { .. }))
        );
    }
    eprintln!(
        "Ann started watching {tv_starts} times; toilet {toilet}; bladder {:.2}",
        w.sims[0].needs[1]
    );
    assert!(
        toilet && w.sims[0].needs[1] > 0.9,
        "Ann never dealt with the urgent need"
    );
    assert!(
        tv_starts <= 2,
        "Ann started watching {tv_starts} times in 30 minutes"
    );
}

#[test]
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
        let now = matches!(
            w.sims[0].current().map(|a| &a.phase),
            Some(Phase::Using { .. })
        );
        starts += usize::from(now && !was);
        was = now;
    }
    eprintln!(
        "Ann started tending {starts} times in 30 minutes (skill {:.3})",
        w.sims[0].skills[0]
    );
    assert!(
        starts <= 2,
        "Ann started tending {starts} times in 30 minutes"
    );
}

/// A bed and a toilet; energy and the bladder are urgent below 0.1.
const NIGHT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.05},{"id":"bladder","label":"Bladder","decayPerHour":0.1}],
    "autonomy":{"urgent":["energy","bladder"],"urgentBelow":0.1},
    "dayRhythm":{"sleepTags":["sleep"]},
    "objects":[{"id":"bed","name":"Bed","footprint":[1,2],"interactions":[
        {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"]}]},
      {"id":"toilet","name":"Toilet","interactions":[{"id":"use","label":"Use","minutes":5,"effects":{"bladder":1.0},"after":"wash"}]},
      {"id":"sink","name":"Sink","interactions":[{"id":"wash","label":"Wash hands","minutes":1,"effects":{"bladder":0.0},"tags":["wash"],"autonomous":false}]}]}"#;

fn night_town() -> World {
    let lot = r#"{"width":12,"depth":12,"plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[5.5,11.5]}],
        "households":[{"name":"Home","plot":0,"player":true}],
        "objects":[{"def":"bed","x":2,"z":2,"rot":0},{"def":"toilet","x":8,"z":2,"rot":0},{"def":"sink","x":10,"z":2,"rot":0}],
        "sims":[{"name":"Ann","x":5.5,"z":7.5},{"name":"Bob","x":2.5,"z":5.5}]}"#;
    World::from_json(NIGHT, lot, 5).unwrap()
}

#[test]
fn worn_out_with_every_bed_taken_they_still_finish_on_the_toilet() {
    let mut w = night_town();
    // Bob has the only bed; Ann is worn out (nothing she can do about it) and needs the toilet.
    w.apply(sim_core::Command::Use {
        sim: 1,
        object: 0,
        interaction: 0,
    })
    .unwrap();
    w.sims[0].needs[..2].copy_from_slice(&[0.05, 0.3]);
    let mut on_toilet = 0;
    for _ in 0..20 * 20 {
        w.tick_once();
        w.sims[0].needs[0] = 0.05;
        if matches!(w.sims[0].current(), Some(a) if matches!(a.task.kind, TaskKind::Use { object: 1, .. }) && matches!(a.phase, Phase::Using { .. }))
        {
            on_toilet += 1;
        }
    }
    assert!(
        matches!(
            w.sims[1].current().map(|a| a.task.kind),
            Some(TaskKind::Use { object: 0, .. })
        ),
        "Bob is in bed"
    );
    let minutes = on_toilet as f32 * sim_core::MINUTES_PER_TICK;
    assert!(
        minutes >= 2.0 && w.sims[0].needs[1] > 0.9,
        "Ann was cut off on the toilet: {minutes:.2} minutes, bladder {:.2}",
        w.sims[0].needs[1]
    );
}

#[test]
fn practice_lasts_its_time_while_the_skill_is_wanted() {
    let content = r#"{
        "needs":[{"id":"fun","label":"Fun","decayPerHour":0.05}],
        "skills":[{"id":"gardening","label":"Gardening"}],
        "skillRules":{"interest":0.05,"maxLevel":10},
        "objects":[{"id":"flowerbed","name":"Flowerbed","interactions":[
            {"id":"tend","label":"Tend","minutes":20,"effects":{"fun":0.12},"skills":{"gardening":0.3}}]}]}"#;
    let lot = r#"{"width":12,"depth":12,"objects":[{"def":"flowerbed","x":5,"z":5,"rot":0}],"sims":[{"name":"Ann","x":5.5,"z":7.5}]}"#;
    // Told to tend it with fun full: for its 20 minutes while she'd like to get better at it;
    // a master gardener stops as soon as it's no fun any more (nothing left to learn).
    for (skill, lasts) in [(0.0, 15.0), (10.0, 1.0)] {
        let mut w = World::from_json(content, lot, 5).unwrap();
        w.apply(sim_core::Command::SetAutonomy {
            enabled: false,
            household: None,
        })
        .unwrap();
        w.sims[0].needs[0] = 1.0;
        w.sims[0].skills[0] = skill;
        w.apply(sim_core::Command::Use {
            sim: 0,
            object: 0,
            interaction: 0,
        })
        .unwrap();
        let mut using = 0;
        for _ in 0..20 * 30 {
            w.tick_once();
            using += usize::from(matches!(
                w.sims[0].current().map(|a| &a.phase),
                Some(Phase::Using { .. })
            ));
        }
        let minutes = using as f32 * sim_core::MINUTES_PER_TICK;
        eprintln!("skill {skill}: tended for {minutes:.1} minutes");
        assert!(
            if skill == 0.0 {
                minutes >= lasts
            } else {
                minutes <= lasts
            },
            "skill {skill}: tended for {minutes:.1} minutes"
        );
    }
}

/// An order holds against needs: told to watch television, a resident watches it to the end,
/// with free will off even as the essentials run low (`urgentBelow`), and with free will on while
/// none is urgent. (Only a need running out ends it: an accident.)
#[test]
fn an_order_holds_against_needs() {
    use sim_core::Command;
    let content = common::content();
    let town = common::town(&content, 7, 2);
    for (free_will, low) in [(false, 0.12), (true, 0.4)] {
        let mut w = World::from_json(&content, &town, 7).unwrap();
        let h = w.households.iter().position(|h| h.player).unwrap();
        w.households[h].funds = 100_000;
        w.apply(Command::SetAutonomy {
            enabled: free_will,
            household: Some(h as u32),
        })
        .unwrap();
        let noon = clock::tick_at(1, 12.0 * 60.0).unwrap();
        while w.tick < noon {
            w.tick_once();
        }
        w.apply(Command::Buy {
            household: h as u32,
            object: "tv".into(),
            at: None,
            colors: Default::default(),
            turn: None,
            offset: None,
            size: None,
            tall: None,
        })
        .unwrap();
        let tv = w.objects.last().unwrap().stable;
        let tv_at = w.index_of(tv).unwrap() as u32;
        let sim = w
            .sims
            .iter()
            .position(|s| s.household as usize == h)
            .unwrap();
        let n = w.content.needs.len();
        w.sims[sim].needs[..n].fill(low);
        let fun = w.content.needs.iter().position(|d| d.id == "fun").unwrap();
        w.sims[sim].needs[fun] = 0.0;
        w.apply(Command::Use {
            sim: sim as u32,
            object: tv,
            interaction: 0,
        })
        .unwrap();
        let (mut watched, mut other) = (0u64, None);
        // (Time for the walk home first: at noon with free will on they may be out in town.)
        for _ in 0..150 * sim_core::TICKS_PER_MINUTE {
            w.tick_once();
            match w.sims[sim].current().map(|a| (a.task.kind, &a.phase)) {
                Some((TaskKind::Use { object, .. }, Phase::Using { .. })) if object == tv_at => {
                    watched += 1
                }
                Some((TaskKind::Use { object, .. }, _)) if object == tv_at => {}
                None if watched > 0 => break,
                what => {
                    other = Some(format!("{what:?}"));
                    break;
                }
            }
        }
        let minutes = watched / sim_core::TICKS_PER_MINUTE;
        assert!(
            other.is_none(),
            "free will {free_will}: {other:?} after {minutes} minutes of television"
        );
        assert!(
            minutes >= 59,
            "free will {free_will}: watched for {minutes} minutes"
        );
    }
}

#[test]
fn newborns_are_born_at_age_zero() {
    let content = common::content();
    let town = common::town(&content, 7, 2);
    let mut w = World::from_json(&content, &town, 7).unwrap();
    let h = w.households.iter().position(|h| h.player).unwrap();
    let adults: Vec<u32> = w
        .sims
        .iter()
        .filter(|s| s.household as usize == h && s.adult(&w.content))
        .map(|s| s.id)
        .collect();
    w.households[h].expecting = Some(sim_core::world::Expecting {
        parents: [adults[0], adults[1]],
        due: 2,
    });
    let midnight = clock::tick_at(2, 0.0).unwrap();
    while w.tick < midnight {
        w.tick_once();
    }
    let born = w
        .events
        .iter()
        .find(|e| e.kind == EventKind::Born)
        .expect("the baby was born");
    let (baby, parent) = (born.a as usize, adults[0] as usize);
    let age = w.sims[baby].age;
    eprintln!("born at age {age}");
    assert_eq!(age, 0.0, "a newborn is a day's ageing old already");
    // From the next midnight on, a day older each night, like everyone else.
    let before = w.sims[parent].age;
    let midnight = clock::tick_at(3, 0.0).unwrap();
    while w.tick < midnight {
        w.tick_once();
    }
    assert!(
        w.sims[baby].age > 0.0 && w.sims[baby].age == w.sims[parent].age - before,
        "the baby ages from the next night"
    );
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
    Carry(u32, u8),
}

fn key(w: &World, i: usize) -> Key {
    let s = &w.sims[i];
    if s.away_until.is_some() {
        return Key::Away;
    }
    let Some(a) = s.current() else {
        return Key::Idle;
    };
    match a.task.kind {
        TaskKind::Use {
            object,
            interaction,
        } => Key::Use(
            object,
            w.objects.get(object as usize).map_or(usize::MAX, |o| o.def),
            interaction,
        ),
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
        TaskKind::Carry { to, stage, .. } => Key::Carry(to, stage),
    }
}

fn kind_name(w: &World, k: Key) -> String {
    match k {
        Key::Use(_, d, i) => {
            let Some(def) = w.content.objects.get(d) else {
                return "Use ?".into();
            };
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
        Phase::Calling { .. } => "Calling",
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

fn longest(
    map: &mut BTreeMap<String, (u64, String, u64)>,
    k: String,
    ticks: u64,
    who: &str,
    from: u64,
) {
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
            p.first_non_finite
                .push(format!("tick {} {who}: {what} = {v}", w.tick));
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
                    note(
                        &mut p.out_of_range,
                        format!("need {}", w.content.needs[k].id),
                    );
                }
            }
        }
        for (what, v) in [
            ("pos.x", s.pos[0]),
            ("pos.z", s.pos[1]),
            ("yaw", s.yaw),
            ("age", s.age),
            ("grade", s.grade),
        ] {
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
                && waypoints
                    .iter()
                    .any(|w| !w[0].is_finite() || !w[1].is_finite())
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
            for (what, v, lim) in [
                ("friendship", r.friendship, 100.0),
                ("romance", r.romance, 100.0),
                ("chemistry", c, 1.0),
            ] {
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
    let mut p = Probe {
        need_min: 1.0,
        need_max: 0.0,
        ..Default::default()
    };
    let mut tracks: Vec<Track> = Vec::new();
    let end = clock::tick_at(days + 1, 8.0 * 60.0).unwrap();
    let started = std::time::Instant::now();
    let hour = (60.0 / sim_core::MINUTES_PER_TICK) as u64;
    while w.tick < end {
        w.tick_once();
        let tick = w.tick;
        while tracks.len() < w.sims.len() {
            tracks.push(Track {
                since: tick,
                ..Default::default()
            });
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
                tracks[i] = Track {
                    since: tick,
                    ..Default::default()
                };
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
                            p.longest_redo_chain = (
                                t.redo_chain,
                                format!(
                                    "{name} {} until day {} {:.0} min",
                                    kind_name(&w, k),
                                    clock::day(tick),
                                    clock::minute_of_day(tick)
                                ),
                            );
                        }
                    } else {
                        t.redo_chain = 0;
                    }
                    // X, Y, X again within the window.
                    if d.len() >= 2 {
                        let (x, _, x_end) = d[d.len() - 2];
                        let (y, ..) = d[d.len() - 1];
                        if x == k && y != k && tick - x_end <= FLIP {
                            note(
                                &mut p.aba,
                                format!("{} <> {}", kind_name(&w, x), kind_name(&w, y)),
                            );
                        }
                    }
                    // X, Y, X, Y.
                    if d.len() >= 3 {
                        let (x, x_start, _) = d[d.len() - 3];
                        let (y, ..) = d[d.len() - 2];
                        let (x2, ..) = d[d.len() - 1];
                        if x == x2 && y == k && x != y && tick - x_start <= 3 * FLIP {
                            note(
                                &mut p.abab,
                                format!("{} <> {}", kind_name(&w, x), kind_name(&w, y)),
                            );
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
                    longest(
                        &mut p.longest_phase,
                        format!("{} / {}", kind_name(&w, pk), name_p),
                        tick - t.phase_since,
                        &name,
                        t.phase_since,
                    );
                    if matches!(pk, Key::Social(..) | Key::Join(_))
                        && name_p == "Waiting"
                        && ph.is_some_and(|(n, k2)| n == "Routing" && k2 == pk)
                    {
                        t.chases += 1;
                        if t.chases > p.most_chases.0 {
                            p.most_chases = (
                                t.chases,
                                format!("{name} {} from tick {}", kind_name(&w, pk), t.since),
                            );
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
    let days: u32 = std::env::var("DAYS")
        .ok()
        .and_then(|d| d.parse().ok())
        .unwrap_or(60);
    let seeds: Vec<u64> = std::env::var("SEEDS").ok().map_or(vec![7, 11], |s| {
        s.split(',').filter_map(|x| x.parse().ok()).collect()
    });
    let minutes = |t: u64| t as f32 * sim_core::MINUTES_PER_TICK;
    for seed in seeds {
        let p = probe(days, seed);
        eprintln!("== seed {seed}, {days} days ==");
        eprintln!(
            "non-finite: {:?} first: {:?}",
            p.non_finite, p.first_non_finite
        );
        eprintln!(
            "out of range: {:?}; needs seen {:.4}..{:.4}",
            p.out_of_range, p.need_min, p.need_max
        );
        eprintln!(
            "activity starts: {}; waking hours (non-babies, here): {:.0}; starts per waking hour {:.2}",
            p.starts,
            p.waking_hours,
            p.waking_starts.values().sum::<usize>() as f64 / p.waking_hours.max(1.0)
        );
        let mut by_len: Vec<_> = p.longest.iter().collect();
        by_len.sort_by_key(|e| std::cmp::Reverse(e.1.0));
        eprintln!("longest stretch per kind (minutes):");
        for (k, (t, who, from)) in by_len.iter().take(25) {
            eprintln!(
                "  {:>7.0}  {k}  ({who}, day {} {:.0}:00)",
                minutes(*t),
                clock::day(*from),
                clock::hour(*from).floor()
            );
        }
        let mut by_len: Vec<_> = p
            .longest_phase
            .iter()
            .filter(|(k, _)| !k.ends_with("/ Using"))
            .collect();
        by_len.sort_by_key(|e| std::cmp::Reverse(e.1.0));
        eprintln!("longest non-Using phase stretches (minutes):");
        for (k, (t, who, from)) in by_len.iter().take(15) {
            eprintln!(
                "  {:>7.1}  {k}  ({who}, day {} {:.0}:00)",
                minutes(*t),
                clock::day(*from),
                clock::hour(*from).floor()
            );
        }
        eprintln!(
            "A-B-A within {} min: {} total; top {:?}",
            minutes(FLIP),
            p.aba.values().sum::<usize>(),
            top(&p.aba, 12)
        );
        eprintln!(
            "A-B-A-B within {} min: {} total; top {:?}",
            minutes(3 * FLIP),
            p.abab.values().sum::<usize>(),
            top(&p.abab, 12)
        );
        eprintln!(
            "same thing again within 1 min after <=1 min of it: {} total; top {:?}",
            p.quick_redo.values().sum::<usize>(),
            top(&p.quick_redo, 10)
        );
        eprintln!(
            "activities over in <=2 ticks: {} total; top {:?}",
            p.short.values().sum::<usize>(),
            top(&p.short, 10)
        );
        eprintln!(
            "most chases in one social: {:?}; longest engaged {:.1} min; walking without moving 100+ ticks: {}",
            p.most_chases,
            minutes(p.longest_engaged),
            p.stuck_walking
        );
        let worst = p.short_engaged_hour.iter().max_by_key(|(_, n)| **n);
        eprintln!(
            "engagements over in <=2 ticks (the target's activity dropped for nothing): {}; resident-hours with 10+: {}; worst hour {worst:?}",
            p.short_engaged,
            p.short_engaged_hour.values().filter(|&&n| n >= 10).count()
        );
        eprintln!("accidents on the spot: {:?}", p.spots);
        eprintln!(
            "longest run of quick restarts of one thing: {:?}",
            p.longest_redo_chain
        );
        eprintln!("newborn ages at birth: {:?}", p.born_ages);
        eprintln!("story: {:?}", p.story);
    }
}

fn describe(w: &World, i: usize) -> String {
    let s = &w.sims[i];
    let c = &w.content;
    let needs: Vec<String> = c
        .needs
        .iter()
        .zip(&s.needs)
        .map(|(d, v)| format!("{}={v:.3}", d.id))
        .collect();
    let stage = c.life.stage_at(s.age).map_or("?", |st| st.id.as_str());
    let run = s.planner.run.as_ref().map(|r| {
        format!(
            "block {} skipped {:?} no_place {}",
            c.activities[r.activity].id, r.skipped, r.no_place
        )
    });
    let goals: Vec<&str> = s
        .planner
        .goals
        .iter()
        .map(|g| c.goals[g.def].id.as_str())
        .collect();
    let act = s
        .current()
        .map(|a| format!("{:?} {}", a.task.kind, phase_name(&a.phase)));
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
    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|d| d.parse().ok())
        .unwrap_or(7);
    let days: u32 = std::env::var("DAYS")
        .ok()
        .and_then(|d| d.parse().ok())
        .unwrap_or(10);
    let pat = std::env::var("TRACE").unwrap_or_default();
    let who = std::env::var("WHO").unwrap_or_default();
    let limit: usize = std::env::var("LIMIT")
        .ok()
        .and_then(|d| d.parse().ok())
        .unwrap_or(12);
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
            let when = format!(
                "day {} {:06.1}",
                clock::day(tick),
                clock::minute_of_day(tick)
            );
            let window = |name: &str, default: f32| {
                std::env::var(name)
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(default)
            };
            if let Ok(every) = std::env::var("WATCH")
                && clock::day(tick) == days
                && (window("FROM", 0.0)..window("TO", 1440.0)).contains(&clock::minute_of_day(tick))
                && tick.is_multiple_of(every.parse().unwrap_or(20))
            {
                let target = match w.sims[i].current().map(|a| a.task.kind) {
                    Some(TaskKind::Social { target, .. } | TaskKind::Join { leader: target }) => {
                        Some(describe(&w, target as usize))
                    }
                    _ => None,
                };
                eprintln!("{when} WATCH {} || target: {target:?}", describe(&w, i));
            }
            match last[i] {
                Some((old, since)) if old != k => {
                    let name = kind_name(&w, old);
                    if std::env::var("LOG").is_ok() && w.tick >= clock::tick_at(days, 0.0).unwrap()
                    {
                        eprintln!(
                            "{when} {name} ({} ticks) -> {}: {}",
                            tick - since,
                            kind_name(&w, k),
                            describe(&w, i)
                        );
                    }
                    if !matches!(old, Key::Idle | Key::Away) {
                        if name.contains(&pat) && tick - since <= 20 {
                            eprintln!(
                                "{when} SHORT {name} lasted {} ticks; the tick before: {}",
                                tick - since,
                                prev_desc[i]
                            );
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
                        eprintln!(
                            "{when} REDO {} after {} ticks of it: {}",
                            kind_name(&w, k),
                            to - from,
                            describe(&w, i)
                        );
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
                eprintln!(
                    "{when} LONG {} for {} h: {}",
                    kind_name(&w, k),
                    (tick - since) / 1200,
                    describe(&w, i)
                );
                printed += 1;
            }
        }
    }
}

/// A household's people and beds at `DAYS` `FROM` (minute): `SEED`, `HOUSE`.
#[test]
#[ignore = "R3 probe: traces"]
fn house() {
    let env = |name: &str, default: f32| {
        std::env::var(name)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    };
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
        let people = w
            .sims
            .iter()
            .filter(|s| s.here() && s.household as usize == hh && !w.content.life.baby(s.age))
            .count();
        let places: u8 = w
            .objects
            .iter()
            .filter(|o| household.plot.is_some() && w.plot_at(o.x, o.z) == household.plot)
            .filter(|o| {
                w.content.objects[o.def]
                    .interactions
                    .iter()
                    .any(|i| i.tags & sleep != 0 && !i.baby)
            })
            .map(|o| w.content.objects[o.def].slots)
            .sum();
        eprintln!(
            "household {hh} {}: {people} past the crib, {places} bed places{}",
            household.name,
            if usize::from(places) < people {
                "  <-- short"
            } else {
                ""
            }
        );
    }
    let plot = w.households[h as usize].plot;
    for (i, s) in w
        .sims
        .iter()
        .enumerate()
        .filter(|(_, s)| s.here() && s.household == h)
    {
        eprintln!("{}", describe(&w, i));
        let _ = s;
    }
    let sleep = w.content.day_rhythm.sleep_tags;
    for o in w.objects.iter().filter(|o| w.plot_at(o.x, o.z) == plot) {
        let def = &w.content.objects[o.def];
        if def
            .interactions
            .iter()
            .any(|i| i.tags & sleep != 0 || i.baby)
        {
            eprintln!(
                "bed-like {} #{} slots {} users {:?} wear {:.2}",
                def.id, o.id, def.slots, o.users, o.wear
            );
        }
    }
}
