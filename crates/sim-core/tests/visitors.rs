//! Visitors at the door (`visits.door`): they knock, someone at home answers and lets them in or
//! turns them away, and with nobody to answer they go home again. As they leave, they say
//! goodbye to a host who's free (`visits.farewell`).

mod common;

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

/// The test content with a goodbye at the end of a visit (`visits.farewell`) and a bed (sleep
/// can't be interrupted by a social).
fn farewell_content() -> String {
    CONTENT
        .replace(
            r#""tags":["lounge"]}]}]"#,
            r#""tags":["lounge"]}]},
            {"id":"bed","name":"Bed","footprint":[2,2],"interactions":[{"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{},"tags":["sleep"]}]}]"#,
        )
        .replace(
            r#""socials":["#,
            r#""socials":[{"id":"goodbye","label":"Say goodbye","minutes":3,"animation":"wave","tags":["social"],"autonomous":false,"acceptance":{"base":1.0},"success":{"friendship":1,"targetFriendship":1}},"#,
        )
        .replace(r#""latestHour":21,"#, r#""latestHour":21,"farewell":["goodbye"],"farewellMinutes":4,"#)
        .replace(r#""economy":"#, r#""socialRules":{"busyTags":["sleep"]},"economy":"#)
}

/// Ada's bed in `visiting`.
const BED: u32 = 1;

/// Bo let in at Ada's (who has a bed), the greeting over, and nobody needing anything.
fn visiting() -> World {
    let town = town("friends").replace(
        r#"{"def":"chair","x":3,"z":2,"rot":0}"#,
        r#"{"def":"chair","x":3,"z":2,"rot":0},{"def":"bed","x":12,"z":2,"rot":0}"#,
    );
    let mut w = World::from_json(&farewell_content(), &town, 7).unwrap();
    w.autonomy = false;
    while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    w.apply(Command::Visit { sim: 1, plot: 0 }).unwrap();
    while !w.events.iter().any(|e| e.kind == EventKind::Visited) {
        w.tick_once();
        assert!(w.tick < clock::tick_at(1, 13.0 * 60.0).unwrap(), "never let in");
    }
    // (Ada greets Bo with a chat.)
    while w.sims[0].current().is_some() || w.sims[1].engaged_with.is_some() || w.sims[1].current().is_some() {
        w.tick_once();
        assert!(w.tick < clock::tick_at(1, 14.0 * 60.0).unwrap(), "the greeting never ended");
    }
    for s in &mut w.sims {
        s.needs = [1.0; sim_core::content::MAX_NEEDS];
    }
    w
}

fn goodbye(w: &World) -> usize {
    w.content.social_index("goodbye").unwrap()
}

/// Whether Bo is saying (or on his way to say) goodbye to Ada.
fn saying_goodbye(w: &World) -> bool {
    let g = goodbye(w);
    w.sims[1].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Social { target: 0, social } if social == g))
}

/// Whether Ada is lying asleep in her bed.
fn in_bed(w: &World) -> bool {
    w.sims[0].pose == sim_core::content::Pose::Lie && w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Use { object: BED, .. }))
}

/// Ends Bo's visit now.
fn time_up(w: &mut World) {
    let tick = w.tick;
    w.sims[1].visiting.as_mut().unwrap().until = tick;
}

#[test]
fn a_guest_says_goodbye_to_the_host_then_walks_home() {
    let mut w = visiting();
    let before = w.relationships.get(1, 0).friendship;
    time_up(&mut w);
    let start = w.tick;
    let (mut said, mut walked_after) = (false, false);
    while w.sims[1].visiting.is_some() {
        w.tick_once();
        assert!(w.tick < start + (60.0 / sim_core::MINUTES_PER_TICK) as u64, "Bo never got home");
        said |= saying_goodbye(&w) && w.sims[0].engaged_with == Some(1);
        let going = w.sims[1].current().is_some_and(|a| matches!(a.task.kind, TaskKind::GoHome));
        assert!(!going || said, "Bo set off without saying goodbye");
        walked_after |= going;
    }
    assert!(said, "Bo said goodbye to Ada");
    assert!(walked_after, "and then walked home");
    assert_eq!(w.plot_at(w.sims[1].tile().0, w.sims[1].tile().1), Some(1), "home");
    assert!(w.relationships.get(1, 0).friendship > before, "a little closer for it");
}

#[test]
fn a_guest_leaves_a_sleeping_host_be() {
    let mut w = visiting();
    // Ada goes to bed.
    w.apply(Command::Use { sim: 0, object: BED, interaction: 0 }).unwrap();
    while !in_bed(&w) {
        w.tick_once();
        assert!(w.tick < clock::tick_at(1, 16.0 * 60.0).unwrap(), "Ada never got into bed");
    }
    time_up(&mut w);
    let start = w.tick;
    while w.sims[1].visiting.is_some() {
        w.tick_once();
        assert!(w.tick < start + (45.0 / sim_core::MINUTES_PER_TICK) as u64, "Bo never got home");
        assert!(!saying_goodbye(&w), "Bo went to say goodbye to a sleeping host");
    }
    assert!(in_bed(&w), "Ada is still asleep");
    assert_eq!(w.sims[0].engaged_with, None);
}

#[test]
fn a_goodbye_that_cannot_happen_soon_is_skipped() {
    let mut w = visiting();
    // Bo across the room from Ada.
    w.sims[0].pos = [4.5, 4.5];
    w.sims[1].pos = [9.5, 4.5];
    time_up(&mut w);
    while !saying_goodbye(&w) {
        w.tick_once();
        assert!(w.tick < clock::tick_at(1, 16.0 * 60.0).unwrap(), "Bo never went to say goodbye");
    }
    // Ada heads off to bed just as Bo comes over.
    w.apply(Command::Use { sim: 0, object: BED, interaction: 0 }).unwrap();
    let mut waited = sim_core::MINUTES_PER_TICK;
    while w.sims[1].visiting.is_some() {
        w.tick_once();
        if saying_goodbye(&w) {
            waited += sim_core::MINUTES_PER_TICK;
        }
        assert!(waited <= 4.1, "Bo hung around for a goodbye");
        assert!(w.tick < clock::tick_at(1, 17.0 * 60.0).unwrap(), "Bo never got home");
    }
    assert_eq!(w.plot_at(w.sims[1].tile().0, w.sims[1].tile().1), Some(1), "home");
    assert!(waited > 3.5, "Bo gave Ada a few minutes ({waited:.1})");
    assert!(in_bed(&w), "Ada went to bed");
}

#[test]
fn a_guest_saying_goodbye_at_save_time_still_goes_home_after_loading() {
    let mut w = visiting();
    time_up(&mut w);
    while !saying_goodbye(&w) {
        w.tick_once();
        assert!(w.tick < clock::tick_at(1, 16.0 * 60.0).unwrap(), "Bo never went to say goodbye");
    }
    let mut loaded = World::from_save_json(&farewell_content(), &w.save_json()).unwrap();
    loaded.autonomy = false;
    minutes(&mut loaded, 60.0);
    assert!(loaded.sims[1].visiting.is_none(), "went home");
    assert_eq!(loaded.plot_at(loaded.sims[1].tile().0, loaded.sims[1].tile().1), Some(1), "home");
}

#[test]
fn goodbyes_are_not_on_the_social_menu() {
    let w = visiting();
    let g = goodbye(&w);
    let options = sim_core::conversation::options(&w, 1, 0);
    assert!(!options.is_empty());
    assert!(!options.iter().any(|&(i, _)| i == g));
}

/// With the shipped content: a guest at the player's home says goodbye as the visit ends, with a
/// hug for a close friend and a wave otherwise. (`DUMP_STATES=1` saves the moment for the game's
/// `?state=` debug view.)
#[test]
fn guests_hug_close_friends_goodbye_and_wave_to_others() {
    let content = common::content();
    for (friendship, social, name) in [(70.0, "goodbyeHug", "goodbye-hug"), (30.0, "goodbye", "goodbye-wave")] {
        let mut w = World::from_json(&content, &common::town(&content, 7, 2), 7).unwrap();
        let h = w.households.iter().position(|h| h.player).unwrap();
        let plot = w.households[h].plot.unwrap();
        let guest = w.sims.iter().position(|s| s.household as usize != h && s.adult(&w.content)).unwrap();
        w.autonomy = false;
        // (A day off for everyone.)
        for s in &mut w.sims {
            s.job = None;
        }
        while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
            w.tick_once();
        }
        for i in 0..w.sims.len() {
            let n = w.content.needs.len();
            w.sims[i].needs[..n].fill(1.0);
            if w.sims[i].household as usize == h {
                w.relationships.get_mut(guest, i).friendship = friendship;
                w.relationships.get_mut(i, guest).friendship = friendship;
            }
        }
        common::look_at(&mut w, plot);
        w.apply(Command::Visit { sim: guest as u32, plot }).unwrap();
        while !w.events.iter().any(|e| e.kind == EventKind::Visited && e.a as usize == guest) {
            w.tick_once();
            assert!(w.tick < clock::tick_at(1, 13.0 * 60.0).unwrap(), "never let in");
        }
        minutes(&mut w, 30.0);
        let tick = w.tick;
        w.sims[guest].visiting.as_mut().unwrap().until = tick;
        let farewell = w.content.social_index(social).unwrap();
        let mut said = None;
        for _ in 0..(30.0 / sim_core::MINUTES_PER_TICK) as u32 {
            w.tick_once();
            if let Some(TaskKind::Social { target, social }) = w.sims[guest].current().map(|a| a.task.kind)
                && w.sims[target as usize].engaged_with == Some(guest as u32)
                && w.content.visits.farewell.contains(&social)
            {
                said = Some((target, social));
                break;
            }
        }
        let (host, s) = said.unwrap_or_else(|| panic!("{name}: no goodbye"));
        assert_eq!(w.sims[host as usize].household as usize, h, "{name}: to one of the hosts");
        assert_eq!(s, farewell, "{name}: {social}");
        for _ in 0..sim_core::TICKS_PER_SECOND / 4 {
            w.tick_once();
        }
        common::dump_state(&w, name);
        minutes(&mut w, 60.0);
        assert!(w.sims[guest].visiting.is_none(), "{name}: went home");
    }
}

/// A bladder and a toilet at Ada's (`bathroom` makes a resident busy); guests may use it when
/// `guestTags` says so.
fn toilet_content(guest_tags: &str) -> String {
    bored_content()
        .replace(
            r#"{"id":"social","label":"Social","decayPerHour":0.01}]"#,
            r#"{"id":"social","label":"Social","decayPerHour":0.01},{"id":"bladder","label":"Bladder","decayPerHour":0.01}]"#,
        )
        .replace(
            r#""tags":["study"]}]}]"#,
            r#""tags":["study"]}]},
            {"id":"toilet","name":"Toilet","interactions":[{"id":"use","label":"Use","minutes":5,"effects":{"bladder":0.9},"tags":["bathroom"]}]}]"#,
        )
        .replace(r#""economy":"#, &format!(r#""socialRules":{{"busyTags":["bathroom"],"guestTags":{guest_tags}}},"economy":"#))
}

fn guest_uses_the_toilet(content: &str) -> bool {
    let town = town("friends").replace(
        r#"{"def":"chair","x":3,"z":2,"rot":0}"#,
        r#"{"def":"chair","x":3,"z":2,"rot":0},{"def":"desk","x":8,"z":2,"rot":0},{"def":"toilet","x":14,"z":2,"rot":0}"#,
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
    let toilet = w.objects.iter().find(|o| w.content.objects[o.def].id == "toilet").unwrap().id;
    let bladder = w.content.needs.iter().position(|n| n.id == "bladder").unwrap();
    w.sims[1].needs[bladder] = 0.2;
    let mut used = false;
    for _ in 0..(30.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        used |= w.sims[1].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Use { object, .. } if object == toilet));
    }
    used
}

#[test]
fn a_guest_may_use_the_hosts_toilet() {
    assert!(guest_uses_the_toilet(&toilet_content(r#"["bathroom"]"#)), "Bo never used Ada's toilet");
    assert!(!guest_uses_the_toilet(&toilet_content("[]")), "without guestTags the toilet is the household's");
}
