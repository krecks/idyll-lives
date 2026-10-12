//! Visitors at the door (`visits.door`): they knock, someone at home answers and lets them in or
//! turns them away, and with nobody to answer they go home again.

use sim_core::social::EventKind;
use sim_core::world::TaskKind;
use sim_core::{Command, World, clock};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01},{"id":"social","label":"Social","decayPerHour":0.01}],
    "objects":[{"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"effects":{"social":0.01},"tags":["lounge"]}]}],
    "feelings":[{"id":"turnedAway","label":"Turned away","mood":-0.15,"hours":6}],
    "animations":["knock","talk"],
    "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},"needs":{"social":0.3}}],
    "bondPresets":{"friends":{"friendship":60},"enemies":{"friendship":-60}},
    "visits":{"needs":{"social":0.5},"hours":2,"minFriendship":10,"earliestHour":9,"latestHour":21,
      "door":{"waitMinutes":10,"welcomeFriendship":-20,"greet":"chat","turnedAwayFeeling":"turnedAway","knockAnim":"knock"}},
    "economy":{"startingFunds":500}}"#;

/// Two houses side by side; the host's front door is at the bottom of the west lot.
fn town(bond: &str) -> String {
    format!(
        r#"{{"width":40,"depth":14,
        "plots":[{{"name":"West","x":0,"z":0,"w":20,"d":10,"entry":[10.5,9.5]}},{{"name":"East","x":20,"z":0,"w":20,"d":10,"entry":[30.5,9.5]}}],
        "households":[{{"name":"Host","plot":0,"player":true}},{{"name":"Guest","plot":1}}],
        "objects":[{{"def":"chair","x":3,"z":2,"rot":0}},{{"def":"chair","x":24,"z":2,"rot":0}}],
        "sims":[{{"name":"Ada","household":0,"x":4.5,"z":4.5}},{{"name":"Bo","household":1,"x":25.5,"z":4.5}}],
        "relationships":[{{"a":0,"b":1,"preset":"{bond}"}}]}}"#
    )
}

fn world(bond: &str) -> World {
    let mut w = World::from_json(CONTENT, &town(bond), 7).unwrap();
    w.autonomy = false;
    // 11:00 on day 1: within visiting hours.
    while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    w
}

fn minutes(w: &mut World, m: f32) {
    let end = w.tick + (m / sim_core::MINUTES_PER_TICK) as u64;
    while w.tick < end {
        w.tick_once();
    }
}

fn knock_anim(w: &World) -> f32 {
    w.content.animations.iter().position(|a| a == "knock").unwrap() as f32
}

fn action_of(w: &World, i: usize) -> f32 {
    let mut out = vec![0.0; sim_core::snapshot::CAPACITY];
    sim_core::snapshot::write(w, &mut out);
    out[sim_core::snapshot::HEADER_LEN + i * sim_core::snapshot::SIM_STRIDE + sim_core::snapshot::sim::ACTION]
}

#[test]
fn a_friend_knocks_and_is_let_in() {
    let mut w = world("friends");
    w.apply(Command::Visit { sim: 1, plot: 0 }).unwrap();
    // Walk over and knock.
    let mut knocked = false;
    for _ in 0..(90.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        knocked |= action_of(&w, 1) == knock_anim(&w);
        if w.events.iter().any(|e| e.kind == EventKind::Visited) {
            break;
        }
    }
    assert!(knocked, "the visitor never knocked");
    let visited = w.events.iter().find(|e| e.kind == EventKind::Visited).expect("never let in");
    assert_eq!((visited.a, visited.b), (1, 0), "Ada let Bo in");
    assert_eq!(w.sims[1].visiting.map(|v| v.plot), Some(0));
    assert_ne!(action_of(&w, 1), knock_anim(&w), "inside, not knocking any more");
    assert!(!w.events.iter().any(|e| matches!(e.kind, EventKind::TurnedAway | EventKind::NobodyHome)));
}

#[test]
fn an_enemy_is_turned_away() {
    let mut w = world("enemies");
    w.apply(Command::Visit { sim: 1, plot: 0 }).unwrap();
    minutes(&mut w, 90.0);
    let turned = w.events.iter().find(|e| e.kind == EventKind::TurnedAway).expect("never turned away");
    assert_eq!((turned.a, turned.b), (1, 0));
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Visited));
    let feeling = w.content.feeling_index("turnedAway").unwrap();
    assert!(w.sims[1].feelings.iter().any(|f| f.def == feeling));
    // And home again.
    minutes(&mut w, 60.0);
    assert!(w.sims[1].visiting.is_none());
    assert_eq!(w.plot_at(w.sims[1].tile().0, w.sims[1].tile().1), Some(1));
}

#[test]
fn nobody_home_means_going_home() {
    let mut w = world("friends");
    // Ada is out all day.
    w.sims[0].away_until = Some(w.tick + 100_000);
    w.apply(Command::Visit { sim: 1, plot: 0 }).unwrap();
    minutes(&mut w, 90.0);
    let none = w.events.iter().find(|e| e.kind == EventKind::NobodyHome).expect("no note of an empty house");
    assert_eq!((none.a, none.b), (1, 0));
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Visited));
    minutes(&mut w, 60.0);
    assert!(w.sims[1].visiting.is_none(), "went home");
}

#[test]
fn a_visitor_knocking_at_save_time_knocks_again_after_loading() {
    let mut w = world("friends");
    w.apply(Command::Visit { sim: 1, plot: 0 }).unwrap();
    while action_of(&w, 1) != knock_anim(&w) {
        w.tick_once();
        assert!(w.tick < clock::tick_at(1, 13.0 * 60.0).unwrap(), "never knocked");
    }
    let mut loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    loaded.autonomy = false;
    minutes(&mut loaded, 90.0);
    assert!(loaded.events.iter().any(|e| e.kind == EventKind::Visited), "let in after loading");
}

/// Three houses in a row with no fences between the gardens: the host's on the west, a
/// neighbour's in the middle and the guest's on the east, all facing the street (rows 10..14).
fn row_of_three(bo: [f32; 2]) -> World {
    let town = format!(
        r#"{{"width":42,"depth":14,
        "plots":[{{"name":"West","x":0,"z":0,"w":14,"d":10,"entry":[7.5,9.5]}},{{"name":"Middle","x":14,"z":0,"w":14,"d":10,"entry":[21.5,9.5]}},{{"name":"East","x":28,"z":0,"w":14,"d":10,"entry":[35.5,9.5]}}],
        "households":[{{"name":"Host","plot":0,"player":true}},{{"name":"Guest","plot":2}},{{"name":"Neighbour","plot":1}}],
        "sims":[{{"name":"Ada","household":0,"x":4.5,"z":4.5}},{{"name":"Bo","household":1,"x":{},"z":{}}},{{"name":"Cy","household":2,"x":21.5,"z":1.5}}],
        "relationships":[{{"a":0,"b":1,"preset":"friends"}}]}}"#,
        bo[0], bo[1]
    );
    let mut w = World::from_json(CONTENT, &town, 7).unwrap();
    w.autonomy = false;
    while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    w
}

#[test]
fn a_visitor_goes_round_by_the_street_not_through_the_neighbours_garden() {
    // From the back of the east garden to the west door: the straight line is across the middle one.
    let mut w = row_of_three([33.5, 1.5]);
    w.apply(Command::Visit { sim: 1, plot: 0 }).unwrap();
    let mut on_street = false;
    for _ in 0..(90.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        let (x, z) = w.sims[1].tile();
        assert_ne!(w.plot_at(x, z), Some(1), "Bo cut across the neighbours' garden at {x},{z}");
        on_street |= z >= 10;
        if w.events.iter().any(|e| e.kind == EventKind::Visited) {
            break;
        }
    }
    assert!(on_street, "by the street");
    assert!(w.events.iter().any(|e| e.kind == EventKind::Visited), "and let in at the host's door");
    assert_eq!(w.sims[1].visiting.map(|v| v.plot), Some(0));
}

#[test]
fn someone_in_a_garden_that_isnt_theirs_still_walks_out_of_it() {
    // Bo finds himself at the back of the neighbours' garden and heads home.
    let mut w = row_of_three([21.5, 4.5]);
    w.apply(Command::GoHome { sim: 1 }).unwrap();
    minutes(&mut w, 60.0);
    let (x, z) = w.sims[1].tile();
    assert_eq!(w.plot_at(x, z), Some(2), "home at {x},{z}");
}

/// The test content with guests who keep their hosts company and go home when there's nothing
/// to do, and a desk for a long stretch of study.
fn bored_content() -> String {
    CONTENT
        .replace(
            r#""tags":["lounge"]}]}]"#,
            r#""tags":["lounge"]}]},
            {"id":"desk","name":"Desk","interactions":[{"id":"study","label":"Study","minutes":240,"effects":{},"tags":["study"]}]}]"#,
        )
        .replace(r#""acceptance":{"base":1.0}"#, r#""acceptance":{"base":1.0},"prefer":"liked""#)
        .replace(r#""latestHour":21,"#, r#""latestHour":21,"company":0.05,"boredMinutes":30,"#)
}

/// Bo let in at Ada's (who has a desk), free will on from then on, and neither needing
/// anything (so nothing appeals to the guest but company).
fn let_in(content: &str) -> World {
    let town = town("friends").replace(
        r#"{"def":"chair","x":3,"z":2,"rot":0}"#,
        r#"{"def":"chair","x":3,"z":2,"rot":0},{"def":"desk","x":8,"z":2,"rot":0}"#,
    );
    let mut w = World::from_json(content, &town, 7).unwrap();
    w.autonomy = false;
    while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    w.apply(Command::Visit { sim: 1, plot: 0 }).unwrap();
    while !w.events.iter().any(|e| e.kind == EventKind::Visited) {
        w.tick_once();
        assert!(w.tick < clock::tick_at(1, 13.0 * 60.0).unwrap(), "never let in");
    }
    w.autonomy = true;
    for s in &mut w.sims {
        s.needs = [1.0; sim_core::content::MAX_NEEDS];
    }
    w
}

/// Whether Bo has nothing to do.
fn idle(w: &World) -> bool {
    w.sims[1].current().is_none() && w.sims[1].engaged_with.is_none()
}

#[test]
fn a_guest_with_nothing_to_do_while_the_host_is_busy_goes_home() {
    let mut w = let_in(&bored_content());
    let arrived = w.tick;
    let until = w.sims[1].visiting.unwrap().until;
    // Ada settles down at her desk for hours.
    w.apply(Command::Use { sim: 0, object: 1, interaction: 0 }).unwrap();
    let (mut stood, mut worst, mut walked, mut jump) = (0.0f32, 0.0f32, false, 0.0f32);
    let mut prev = w.sims[1].pos;
    while w.sims[1].visiting.is_some() {
        w.tick_once();
        assert!(w.tick < arrived + (90.0 / sim_core::MINUTES_PER_TICK) as u64, "Bo stayed on with nothing to do");
        stood = if idle(&w) { stood + sim_core::MINUTES_PER_TICK } else { 0.0 };
        worst = worst.max(stood);
        walked |= w.sims[1].current().is_some_and(|a| matches!(a.task.kind, TaskKind::GoHome));
        let pos = w.sims[1].pos;
        jump = jump.max((pos[0] - prev[0]).hypot(pos[1] - prev[1]));
        prev = pos;
    }
    assert!(w.tick < until, "left before the visit was over");
    assert!(worst <= 35.0, "Bo stood around for {worst:.0} minutes");
    assert!(walked && jump < 0.5, "walked home (no teleporting)");
    assert_eq!(w.plot_at(w.sims[1].tile().0, w.sims[1].tile().1), Some(1), "home");
    let studying = w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Use { object: 1, .. }));
    assert!(studying, "Ada is still at her desk");
}

#[test]
fn a_guest_keeps_the_host_company() {
    let mut w = let_in(&bored_content());
    // Nothing to talk about (Bo needs nothing), but Ada is free: Bo goes over for a chat.
    let mut chatted = false;
    for _ in 0..(30.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        chatted |= w.sims[1].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Social { target: 0, .. }));
    }
    assert!(chatted, "Bo never talked to Ada");
    assert_eq!(w.sims[1].visiting.map(|v| v.plot), Some(0), "still visiting");
}

#[test]
fn without_free_will_a_guest_waits_for_the_player() {
    let mut w = let_in(&bored_content());
    w.apply(Command::SetAutonomy { enabled: false, household: Some(1) }).unwrap();
    w.apply(Command::Use { sim: 0, object: 1, interaction: 0 }).unwrap();
    minutes(&mut w, 60.0);
    assert_eq!(w.sims[1].visiting.map(|v| v.plot), Some(0), "still visiting");
}
