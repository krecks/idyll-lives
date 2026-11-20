use super::{doors, firm_leave, knocking, refresh_commute};
use crate::social::EventKind;
use crate::world::{Activity, Phase, Task, TaskKind};
use crate::{Command, World, clock, mins, secs};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02},{"id":"social","label":"Social","decayPerHour":0.02}],
    "objects":[{"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"effects":{"social":0.01},"tags":["lounge"]}]}],
    "feelings":[{"id":"promoted","label":"Promoted","mood":0.2,"hours":24}],
    "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},"needs":{"social":0.3}}],
    "bondPresets":{"friends":{"friendship":60}},
    "careers":[{"id":"office","label":"Office","workEffects":{"hunger":-0.2},"levels":[
        {"title":"Intern","pay":100,"start":9,"hours":4,"days":[0,1,2,3,4,5,6]},
        {"title":"Clerk","pay":200,"start":9,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
    "careerRules":{"promotionFeeling":"promoted","performancePerShift":60},
    "visits":{"needs":{"social":0.5},"hours":2,"minFriendship":10,"earliestHour":0,"latestHour":24},
    "economy":{"startingFunds":500}}"#;

/// Two houses on one street with an exit at the west end.
const TOWN: &str = r#"{"width":40,"depth":14,"exits":[[0.5,7.5]],
    "plots":[{"name":"West","x":0,"z":0,"w":20,"d":6,"entry":[5.5,5.5]},{"name":"East","x":20,"z":0,"w":20,"d":6,"entry":[25.5,5.5]}],
    "households":[{"name":"Player","plot":0,"player":true},{"name":"Npc","plot":1}],
    "objects":[{"def":"chair","x":3,"z":2,"rot":0},{"def":"chair","x":24,"z":2,"rot":0}],
    "sims":[{"name":"Worker","household":0,"job":{"career":"office"},"x":5.5,"z":3.5},
            {"name":"Guest","household":1,"x":25.5,"z":3.5}],
    "relationships":[{"a":0,"b":1,"preset":"friends"}]}"#;

fn ticks(w: &mut World, n: u64) {
    for _ in 0..n {
        w.tick_once();
    }
}

fn until_hour(w: &mut World, day: u32, hour: f32) {
    let target = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < target {
        w.tick_once();
    }
}

#[test]
fn workers_leave_get_paid_and_come_home() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    assert_eq!(w.households[0].funds, 500);
    until_hour(&mut w, 2, 10.0);
    assert!(w.sims[0].away_until.is_some(), "should be at work at 10:00");
    until_hour(&mut w, 2, 14.5);
    assert!(w.sims[0].away_until.is_none());
    assert_eq!(
        w.households[0].funds, 1400,
        "two shifts plus half a week's promotion bonus"
    );
    until_hour(&mut w, 2, 16.0);
    let home = w.plot_at(w.sims[0].tile().0, w.sims[0].tile().1);
    assert_eq!(home, Some(0), "walked back home");
    // 60 performance per shift at neutral mood -> promoted after two shifts.
    assert_eq!(w.sims[0].job.as_ref().unwrap().level, 1);
    assert!(w.events.iter().any(|e| e.kind == EventKind::Promoted));
    let promoted = w
        .events
        .iter()
        .find(|e| e.kind == EventKind::Promoted)
        .unwrap();
    assert_eq!(promoted.pay, Some(700), "half a week at the new grade");
}

#[test]
fn being_kept_busy_means_a_missed_shift() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.autonomy = false;
    // Make the exit unreachable by walling the worker in.
    for x in 0..40 {
        w.lot.set_h(x, 6, crate::lot::Edge::Wall);
    }
    until_hour(&mut w, 2, 11.0);
    assert!(w.events.iter().any(|e| e.kind == EventKind::MissedWork));
    assert!(w.sims[0].away_until.is_none());
}

#[test]
fn friends_visit_and_go_home() {
    let mut w = World::from_json(CONTENT, TOWN, 3).unwrap();
    w.sims[0].job = None;
    w.sims[1].needs[1] = 0.1; // lonely guest
    ticks(&mut w, mins(120.0));
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::Visited && e.a == 1),
        "guest never visited"
    );
    // The visit lasts two hours; afterwards they head home.
    ticks(&mut w, (3.0 * 60.0 * crate::TICKS_PER_MINUTE as f32) as u64);
    let (x, z) = w.sims[1].tile();
    assert_eq!(w.plot_at(x, z), Some(1), "guest went home");
    assert!(w.sims[1].visiting.is_none());
}

#[test]
fn guests_stay_at_the_hosts_place() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.sims[0].job = None;
    w.sims[0].needs[0] = 0.2; // hungry: the home chair... and nothing to eat anywhere
    w.apply(Command::Visit { sim: 0, plot: 1 }).unwrap();
    ticks(&mut w, secs(30.0));
    // (An hour there, looked at each minute.)
    for _ in 0..60 {
        ticks(&mut w, mins(1.0));
        let (x, z) = w.sims[0].tile();
        assert_eq!(
            w.plot_at(x, z),
            Some(1),
            "guest wandered off the host's lot"
        );
    }
}

#[test]
fn player_can_send_a_sim_to_visit_and_back() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.sims[0].job = None;
    w.apply(Command::Visit { sim: 0, plot: 1 }).unwrap();
    ticks(&mut w, secs(40.0));
    assert_eq!(w.sims[0].visiting.map(|v| v.plot), Some(1));
    w.apply(Command::GoHome { sim: 0 }).unwrap();
    ticks(&mut w, secs(40.0));
    assert!(w.sims[0].visiting.is_none());
}

/// The test content with a job market: hired the day they look, let go after two misses.
fn with_market() -> String {
    CONTENT.replace(
        r#""careerRules":{"promotionFeeling":"promoted","performancePerShift":60}"#,
        r#""careerRules":{"promotionFeeling":"promoted","performancePerShift":60,
            "market":{"hour":9.5,"dailyChance":1.0,"fireAfterMissed":2,"cooldownDays":1,
                      "hiredFeeling":"promoted"}}"#,
    )
}

#[test]
fn the_unemployed_find_work_on_their_own() {
    let mut w = World::from_json(&with_market(), TOWN, 2).unwrap();
    assert!(w.sims[1].job.is_none());
    until_hour(&mut w, 1, 10.0);
    let job = w.sims[1].job.as_ref().expect("found a job at 9:30");
    assert_eq!(job.career, 0);
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::JobFound && e.a == 1 && e.career == Some(0))
    );
    // The first shift is tomorrow's.
    until_hour(&mut w, 2, 10.0);
    assert!(w.sims[1].away_until.is_some(), "at work on day 2");
}

#[test]
fn no_free_will_no_job_search() {
    let mut w = World::from_json(&with_market(), TOWN, 2).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: Some(1),
    })
    .unwrap();
    until_hour(&mut w, 3, 12.0);
    assert!(w.sims[1].job.is_none());
}

#[test]
fn missing_shifts_gets_you_let_go() {
    let mut w = World::from_json(&with_market(), TOWN, 1).unwrap();
    // No free will: they don't look for a new job afterwards either.
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: Some(0),
    })
    .unwrap();
    // Wall the worker in so they can never reach the exit.
    for x in 0..40 {
        w.lot.set_h(x, 6, crate::lot::Edge::Wall);
    }
    until_hour(&mut w, 3, 12.0);
    assert!(w.sims[0].job.is_none(), "let go after two missed shifts");
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::Fired && e.a == 0)
    );
    // Missed on days 1 and 2; looks again from day 3.
    assert_eq!(w.sims[0].job_search_from, 3);
}

#[test]
fn a_visit_the_player_asked_for_ends() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.sims[0].job = None;
    w.apply(Command::Visit { sim: 0, plot: 1 }).unwrap();
    ticks(&mut w, secs(40.0));
    assert_eq!(w.sims[0].visiting.map(|v| v.plot), Some(1));
    // Twice as long as a normal (2 h) visit, then home.
    ticks(&mut w, (5.0 * 60.0 * crate::TICKS_PER_MINUTE as f32) as u64);
    assert!(w.sims[0].visiting.is_none());
}

#[test]
fn careers_can_be_joined_and_quit() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::QuitCareer { sim: 0 }).unwrap();
    assert!(w.sims[0].job.is_none());
    w.apply(Command::JoinCareer {
        sim: 1,
        career: 0,
        level: 0,
    })
    .unwrap();
    assert_eq!(w.sims[1].job.as_ref().unwrap().level, 0);
    assert!(
        w.apply(Command::JoinCareer {
            sim: 1,
            career: 9,
            level: 0
        })
        .is_err()
    );
}

/// Visitors at a front door (`visits.door`): two houses side by side, friends.
const DOOR_CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01},{"id":"social","label":"Social","decayPerHour":0.01}],
    "objects":[{"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"effects":{"social":0.01},"tags":["lounge"]}]}],
    "animations":["knock","talk"],
    "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},"needs":{"social":0.3}}],
    "bondPresets":{"friends":{"friendship":60}},
    "visits":{"needs":{"social":0.5},"hours":2,"minFriendship":10,"earliestHour":9,"latestHour":21,
      "door":{"waitMinutes":10,"welcomeFriendship":-20,"greet":"chat","knockAnim":"knock"}},
    "economy":{"startingFunds":500}}"#;
const DOOR_TOWN: &str = r#"{"width":40,"depth":14,
    "plots":[{"name":"West","x":0,"z":0,"w":20,"d":10,"entry":[10.5,9.5]},{"name":"East","x":20,"z":0,"w":20,"d":10,"entry":[30.5,9.5]}],
    "households":[{"name":"Host","plot":0,"player":true},{"name":"Guest","plot":1}],
    "objects":[{"def":"chair","x":3,"z":2,"rot":0},{"def":"chair","x":24,"z":2,"rot":0}],
    "sims":[{"name":"Ada","household":0,"x":4.5,"z":4.5},{"name":"Bo","household":1,"x":25.5,"z":4.5}],
    "relationships":[{"a":0,"b":1,"preset":"friends"}]}"#;

/// A host on her way somewhere with the next step of it to come (sitting down with the
/// coffee she went to pour, washing her hands after the toilet) answers the door first: the
/// step waits. (It used to go first, and the visitor gave up knocking.)
#[test]
fn someone_at_the_door_is_answered_before_the_next_step_of_a_routine() {
    let mut w = World::from_json(DOOR_CONTENT, DOOR_TOWN, 7).unwrap();
    w.autonomy = false;
    until_hour(&mut w, 1, 11.0);
    w.apply(Command::Visit { sim: 1, plot: 0 }).unwrap();
    while knocking(&w.sims[1]).is_none() {
        w.tick_once();
        assert!(
            w.tick < clock::tick_at(1, 12.0 * 60.0).unwrap(),
            "never knocked"
        );
    }
    let knocked = w.tick;
    // Sent to the door (after everyone's step: `doors`) just as she's done with something,
    // with sitting in the chair (30 minutes) to come next.
    let ada = &mut w.sims[0];
    ada.current = None;
    ada.queue
        .retain(|t| !matches!(t.kind, TaskKind::Answer { .. }));
    ada.follow_up = Some(Task {
        kind: TaskKind::Use {
            object: 0,
            interaction: 0,
        },
        directed: false,
        routine: false,
    });
    if let Some(Activity {
        phase: Phase::Knocking { host, .. },
        ..
    }) = w.sims[1].current.as_mut()
    {
        *host = None;
    }
    doors(&mut w);
    assert!(matches!(
        w.sims[0].queue.front().map(|t| t.kind),
        Some(TaskKind::Answer { guest: 1 })
    ));
    let in_by = knocked + ticks_of(10.0);
    while !w.events.iter().any(|e| e.kind == EventKind::Visited) {
        w.tick_once();
        assert!(w.tick < in_by, "nobody came to the door");
    }
    assert!(
        !w.events.iter().any(|e| e.kind == EventKind::NobodyHome),
        "the visitor gave up"
    );
    // The visit's hours count from the door, not from leaving home.
    let until = w.sims[1].visiting.unwrap().until;
    assert!(
        until >= w.tick + ticks_of(119.0),
        "{} minutes left",
        (until - w.tick) / crate::TICKS_PER_MINUTE
    );
}

/// Friends are visited only where the way isn't one of hours: the one next door, not the
/// one at the far end of a long street (`world::visit_fits`), most of a kilometre off (a third
/// of that, the taxi takes them: `taxi::leisure_eta`, on the traffic's clock).
#[test]
fn friends_hours_away_are_not_visited_on_a_whim() {
    for (east, visited) in [(20, true), (900, false)] {
        let town = TOWN
            .replace(r#""width":40"#, &format!(r#""width":{}"#, east + 40))
            .replace(
                r#""x":20,"z":0,"w":20"#,
                &format!(r#""x":{east},"z":0,"w":20"#),
            )
            .replace(r#""x":24,"z":2"#, &format!(r#""x":{},"z":2"#, east + 4))
            .replace(
                r#""x":25.5,"z":3.5"#,
                &format!(r#""x":{}.5,"z":3.5"#, east + 5),
            )
            .replace(
                r#""entry":[25.5,5.5]"#,
                &format!(r#""entry":[{}.5,5.5]"#, east + 5),
            );
        let mut w = World::from_json(CONTENT, &town, 3).unwrap();
        w.sims[0].job = None;
        w.sims[1].needs[1] = 0.1; // lonely guest
        ticks(&mut w, mins(240.0));
        let went = w
            .events
            .iter()
            .any(|e| e.kind == EventKind::Visited && e.a == 1)
            || w.sims[1].visiting.is_some();
        assert_eq!(went, visited, "friend {east} tiles away visited: {went}");
    }
}

fn ticks_of(minutes: f32) -> u64 {
    (minutes * crate::TICKS_PER_MINUTE as f32) as u64
}

#[test]
fn out_by_taxi_their_way_to_work_is_from_home_not_from_where_it_took_them() {
    // The worker, out at their neighbour's at the far end of the street from the way out:
    // there on foot, their way to work is at least the walk from there; brought by taxi, it
    // brings them home first, and their way is from home. (Timed as a walk across a big world
    // from the place the taxi took them to, their stay there ended 38 minutes in, Maple Park.)
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let i = 0usize;
    let day = clock::day(w.tick);
    refresh_commute(&mut w, i, day);
    let home = w.sims[i].commute.expect("a way to work").ticks;
    let (leave, start) = (w.tick + mins(600.0), w.tick + mins(660.0));
    w.sims[i].pos = [35.5, 3.5];
    w.sims[i].visiting = Some(super::Visit {
        plot: 1,
        until: w.tick + mins(120.0),
        idle: 0,
        farewell: 0,
        offer_at: 0,
        arrived: w.tick,
        leaving: false,
        home_by: true,
    });
    w.taxis.out_at.push((i as u32, 1));
    firm_leave(&mut w, i, leave, start, day, None);
    assert_eq!(w.sims[i].commute.unwrap().ticks, home, "by taxi: from home");
    w.taxis.out_at.clear();
    firm_leave(&mut w, i, leave, start, day, None);
    assert!(
        w.sims[i].commute.unwrap().ticks > home,
        "on foot: at least the walk from there"
    );
}
