use crate::social::EventKind;

#[test]
fn meeting_halfway_over_the_stairs_is_by_the_stairs_not_across_town() {
    // Storeys 222 rows apart: one upstairs at (33.5, 284.5) calls one just down the stairs
    // at (37.7, 66.5), the flight's top at (36.9, 286.5). Measured in rows, the middle was
    // half of the 220 between the blocks: (37, 180.6) on the ground, a street away.
    let mut stairs = crate::path::StairMap::default();
    stairs.depth = 222;
    let from = [33.5, 284.5];
    let path = [[36.9, 286.5], [37.7, 66.5], [37.7, 60.5]];
    let (mine, middle) = super::halfway(&stairs, from, &path);
    assert!(
        stairs.distance(middle, [37.7, 66.5]) < 6.0,
        "by the stairs: {middle:?}"
    );
    assert_eq!(mine.last(), Some(&middle));
    // On one storey, as before: halfway along the legs.
    let flat = crate::path::StairMap::default();
    let (_, m) = super::halfway(&flat, [0.0, 0.0], &[[4.0, 0.0], [4.0, 6.0]]);
    assert!(
        (m[0] - 4.0).abs() < 1e-4 && (m[1] - 1.0).abs() < 1e-4,
        "{m:?}"
    );
}
use crate::world::{Phase, TICKS_PER_DAY, Task, TaskKind};
use crate::{Command, World, mins, secs};

const CONTENT: &str = r#"{
    "needs":[{"id":"social","label":"Social","decayPerHour":0.1},{"id":"fun","label":"Fun","decayPerHour":0.05}],
    "objects":[],
    "genders":[{"id":"female","label":"Female"},{"id":"male","label":"Male"}],
    "emotions":[{"id":"angry","label":"Angry","effects":{"tagPreference":{"mean":4}}},
                {"id":"happy","label":"Happy"}],
    "feelings":[{"id":"goodChat","label":"Good chat","emotion":"happy","mood":0.05,"hours":2},
                {"id":"jealous","label":"Jealous","emotion":"angry","mood":-0.15,"hours":6},
                {"id":"heartbroken","label":"Heartbroken","mood":-0.25,"hours":24},
                {"id":"lost","label":"Lost a fight","mood":-0.1,"hours":4}],
    "socials":[
      {"id":"chat","label":"Chat","minutes":10,"tags":["social","friendly"],"prefer":"liked","joinable":true,
       "acceptance":{"base":1.0},"needs":{"social":0.4},"targetNeeds":{"social":0.4},
       "success":{"friendship":6,"targetFriendship":6,"feeling":"goodChat","targetFeeling":"goodChat"}},
      {"id":"flirt","label":"Flirt","minutes":10,"tags":["social","romantic"],"animation":"flirt","prefer":"romance",
       "acceptance":{"base":1.0},"success":{"romance":10,"targetRomance":10}},
      {"id":"askPartner","label":"Ask to be partners","minutes":5,"tags":["social","romantic"],"autonomous":false,
       "requires":{"minRomance":30,"partners":false},"acceptance":{"base":1.0},"success":{"effect":"becomePartners"}},
      {"id":"fight","label":"Fight","minutes":5,"tags":["social","mean"],"animation":"fight","autonomous":false,
       "acceptance":{"base":1.0},"success":{"friendship":-20,"targetFriendship":-20,"effect":"fight","loserFeeling":"lost"}},
      {"id":"hug","label":"Hug","minutes":4,"tags":["social","friendly"],"animation":"hug","autonomous":false,
       "acceptance":{"base":0.0}}],
    "bondPresets":{"roommates":{"friendship":20},"partners":{"friendship":50,"romance":70,"partners":true}},
    "socialRules":{"defaultBond":"roommates","romanticTags":["romantic"],"jealousyFeeling":"jealous","heartbreakFeeling":"heartbroken"}}"#;

#[test]
fn a_call_carries_across_a_room_and_through_a_doorway_but_not_a_wall() {
    // Two rooms side by side (x 1..13 and 13..23), a doorway between them at z 6 and a window
    // at z 3; outdoors below them.
    let lot = r#"{"width":30,"depth":14,
        "walls":[[1,1,23,1],[1,11,23,11],[1,1,1,11],[23,1,23,11],[13,1,13,11]],
        "doors":[{"x":13,"z":6,"axis":"z"},{"x":5,"z":11,"axis":"x"}],
        "windows":[{"x":13,"z":3,"axis":"z"}]}"#;
    let w = World::from_json(CONTENT, lot, 1).unwrap();
    let heard = |a: [f32; 2], b: [f32; 2]| w.within_earshot(a, b) && w.within_earshot(b, a);
    assert!(heard([3.5, 6.5], [11.5, 6.5]), "across a room");
    assert!(heard([3.5, 2.5], [10.5, 9.5]), "diagonally across a room");
    assert!(heard([10.5, 6.5], [14.5, 6.5]), "through the doorway");
    assert!(
        !heard([7.5, 6.5], [15.5, 6.5]),
        "through the doorway, from afar"
    );
    assert!(!heard([11.5, 2.5], [14.5, 2.5]), "through a wall");
    assert!(!heard([12.5, 3.5], [13.5, 3.5]), "through a window");
    assert!(!heard([5.5, 10.5], [5.5, 11.5]), "through the front door");
    assert!(heard([2.5, 12.5], [13.5, 12.5]), "out in the garden");
    assert!(
        !heard([2.5, 12.5], [16.5, 12.5]),
        "too far out in the garden"
    );
}

#[test]
fn two_see_each_other_through_a_doorway_but_not_a_wall_a_window_or_the_front_door() {
    // The rooms of the test above, and a diagonal wall out in the garden.
    let lot = r#"{"width":30,"depth":14,
        "walls":[[1,1,23,1],[1,11,23,11],[1,1,1,11],[23,1,23,11],[13,1,13,11]],
        "doors":[{"x":13,"z":6,"axis":"z"},{"x":5,"z":11,"axis":"x"}],
        "windows":[{"x":13,"z":3,"axis":"z"}],
        "diagonals":[{"x":20,"z":12,"dir":"dp"}]}"#;
    let w = World::from_json(CONTENT, lot, 1).unwrap();
    let seen = |a: [f32; 2], b: [f32; 2]| {
        let (there, back) = (w.in_sight(a, b), w.in_sight(b, a));
        assert_eq!(there, back, "{a:?} and {b:?} see each other one way only");
        there
    };
    assert!(seen([3.5, 6.5], [11.5, 6.5]), "across a room");
    assert!(seen([3.5, 2.5], [10.5, 9.5]), "diagonally across a room");
    assert!(seen([10.5, 6.5], [14.5, 6.5]), "through the doorway");
    assert!(seen([12.5, 5.5], [13.5, 6.5]), "past the door frame");
    assert!(!seen([11.5, 2.5], [14.5, 2.5]), "through a wall");
    assert!(!seen([12.5, 7.8], [13.5, 6.8]), "round the door frame");
    assert!(!seen([22.5, 10.5], [23.5, 11.5]), "through a corner");
    assert!(!seen([12.5, 3.5], [13.5, 3.5]), "through a window");
    assert!(!seen([5.5, 10.5], [5.5, 11.5]), "through the front door");
    assert!(seen([2.5, 12.5], [13.5, 12.5]), "out in the garden");
    assert!(!seen([19.5, 12.2], [21.5, 12.8]), "across a diagonal wall");
    assert!(
        seen([19.5, 12.6], [21.5, 13.6]),
        "by the end of a diagonal wall"
    );
}

/// Two rooms side by side (x 1..8 and 8..15), a doorway between them at z 4.
fn two_rooms(sims: &str) -> World {
    let lot = format!(
        r#"{{"width":16,"depth":12,
        "walls":[[1,1,15,1],[1,11,15,11],[1,1,1,11],[15,1,15,11],[8,1,8,11]],
        "doors":[{{"x":8,"z":4,"axis":"z"}}],"sims":[{sims}]}}"#
    );
    let mut w = World::from_json(CONTENT, &lot, 5).unwrap();
    w.autonomy = false;
    w
}

#[test]
fn nobody_talks_through_a_wall() {
    // B stands against the wall, A just the other side of it: the tile next to B nearest A
    // is A's side of the wall. A goes round by the doorway to talk.
    let mut w = two_rooms(r#"{"name":"A","x":6.5,"z":8.5},{"name":"B","x":8.5,"z":8.5}"#);
    let chat = w.content.social_index("chat").unwrap();
    w.apply(Command::Social {
        sim: 0,
        target: 1,
        social: chat,
    })
    .unwrap();
    let mut talked = false;
    for _ in 0..secs(30.0) {
        w.tick_once();
        if w.sims[0].conversation(&w.content).is_some() {
            talked = true;
            let (a, b) = (w.sims[0].pos, w.sims[1].pos);
            assert!(
                w.in_sight(a, b),
                "A at {a:?} talks to B at {b:?} through the wall"
            );
        }
    }
    assert!(talked, "A and B never talked");
}

#[test]
fn nobody_listens_in_through_a_wall() {
    // A talks to B with their back to the wall; C, the other side of it, comes round by the
    // doorway to listen in.
    let mut w = two_rooms(
        r#"{"name":"A","x":7.5,"z":8.5},{"name":"B","x":6.5,"z":8.5},{"name":"C","x":9.5,"z":8.5}"#,
    );
    let chat = w.content.social_index("chat").unwrap();
    w.apply(Command::Social {
        sim: 0,
        target: 1,
        social: chat,
    })
    .unwrap();
    ticks(&mut w, 2);
    assert!(w.sims[0].conversation(&w.content).is_some());
    join(&mut w, 2, 0);
    let mut listened = false;
    for _ in 0..secs(10.0) {
        w.tick_once();
        if w.sims[2].listening_to() == Some(0) {
            listened = true;
            let c = w.sims[2].pos;
            for m in [0, 1] {
                let p = w.sims[m].pos;
                assert!(
                    w.in_sight(c, p),
                    "C at {c:?} listens to {m} at {p:?} through the wall"
                );
            }
        }
    }
    assert!(listened, "C never listened in");
}

fn town(sims: &str, relationships: &str) -> World {
    let lot =
        format!(r#"{{"width":16,"depth":16,"sims":[{sims}],"relationships":[{relationships}]}}"#);
    World::from_json(CONTENT, &lot, 5).unwrap()
}

fn ticks(w: &mut World, n: u64) {
    for _ in 0..n {
        w.tick_once();
    }
}

#[test]
fn lonely_housemates_chat_and_grow_closer() {
    let mut w = town(
        r#"{"name":"A","gender":"female","x":3.5,"z":3.5},{"name":"B","gender":"male","x":8.5,"z":8.5}"#,
        "",
    );
    w.sims[0].needs[0] = 0.1;
    w.sims[1].needs[0] = 0.1;
    let before = w.relationships.get(0, 1).friendship;
    ticks(&mut w, mins(120.0));
    assert!(
        w.relationships.get(0, 1).friendship > before,
        "they never talked"
    );
    assert!(w.sims[0].needs[0] > 0.1);
}

#[test]
fn romance_respects_attraction() {
    let w = town(
        r#"{"name":"A","gender":"female","attractedTo":["female"],"x":3.5,"z":3.5},
           {"name":"B","gender":"male","attractedTo":["female"],"x":4.5,"z":3.5}"#,
        "",
    );
    let flirt = w.content.social_index("flirt").unwrap();
    // A isn't attracted to men: no romantic options toward B.
    assert!(!super::options(&w, 0, 1).iter().any(|(i, _)| *i == flirt));
    // B is attracted to women and may flirt, but A isn't interested.
    let chance = super::options(&w, 1, 0)
        .into_iter()
        .find(|(i, _)| *i == flirt)
        .unwrap()
        .1;
    assert_eq!(chance, 0.0);
}

#[test]
fn directed_flirt_runs_as_a_conversation() {
    let mut w = town(
        r#"{"name":"A","gender":"female","x":3.5,"z":3.5},{"name":"B","gender":"male","x":9.5,"z":3.5}"#,
        "",
    );
    w.autonomy = false;
    let flirt = w.content.social_index("flirt").unwrap();
    w.apply(Command::Social {
        sim: 0,
        target: 1,
        social: flirt,
    })
    .unwrap();
    let mut talked = false;
    for _ in 0..mins(30.0) {
        w.tick_once();
        if matches!(
            w.sims[0].current.as_ref().map(|a| &a.phase),
            Some(Phase::Conversing { .. })
        ) {
            talked = true;
            assert_eq!(w.sims[1].engaged_with, Some(0));
        }
    }
    assert!(talked);
    assert_eq!(w.relationships.get(0, 1).romance, 10.0);
    assert_eq!(w.relationships.get(1, 0).romance, 10.0);
    assert_eq!(
        w.sims[1].engaged_with, None,
        "released after the conversation"
    );
}

#[test]
fn new_partner_breaks_old_relationship_and_flirting_makes_jealous() {
    let mut w = town(
        r#"{"name":"A","gender":"female","x":3.5,"z":3.5},{"name":"B","gender":"male","x":4.5,"z":3.5},
           {"name":"C","gender":"male","x":5.5,"z":3.5}"#,
        r#"{"a":0,"b":1,"preset":"partners"}"#,
    );
    w.autonomy = false;
    let flirt = w.content.social_index("flirt").unwrap();
    let ask = w.content.social_index("askPartner").unwrap();
    w.apply(Command::Social {
        sim: 0,
        target: 2,
        social: flirt,
    })
    .unwrap();
    ticks(&mut w, mins(20.0));
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::Jealous && e.a == 1 && e.b == 0)
    );

    w.relationships.get_mut(0, 2).romance = 50.0;
    w.apply(Command::Social {
        sim: 0,
        target: 2,
        social: ask,
    })
    .unwrap();
    ticks(&mut w, mins(20.0));
    assert!(w.relationships.get(0, 2).partners);
    assert!(!w.relationships.get(0, 1).partners);
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::BrokeUp && e.b == 1)
    );
    assert!(
        w.sims[1]
            .feelings
            .iter()
            .any(|m| w.content.feelings[m.def].id == "heartbroken")
    );
}

#[test]
fn fights_have_a_loser_and_hurt_the_relationship() {
    let mut w = town(
        r#"{"name":"A","x":3.5,"z":3.5},{"name":"B","x":5.5,"z":3.5}"#,
        "",
    );
    w.autonomy = false;
    let fight = w.content.social_index("fight").unwrap();
    let before = w.relationships.get(0, 1).friendship;
    w.apply(Command::Social {
        sim: 0,
        target: 1,
        social: fight,
    })
    .unwrap();
    ticks(&mut w, mins(20.0));
    assert!(w.relationships.get(0, 1).friendship < before);
    let e = w
        .events
        .iter()
        .find(|e| e.kind == EventKind::Fight)
        .expect("fight event");
    assert!(
        w.sims[e.b as usize]
            .feelings
            .iter()
            .any(|m| w.content.feelings[m.def].id == "lost")
    );
}

#[test]
fn relationships_drift_daily_and_feelings_expire() {
    let mut w = town(
        r#"{"name":"A","x":3.5,"z":3.5},{"name":"B","x":12.5,"z":12.5}"#,
        "",
    );
    w.autonomy = false;
    w.relationships.get_mut(0, 1).friendship = 60.0;
    crate::social::add_feeling(&mut w.sims[0].feelings, 0, &w.content.feelings, w.tick);
    ticks(&mut w, TICKS_PER_DAY);
    assert!(w.relationships.get(0, 1).friendship < 60.0);
    assert!(w.sims[0].feelings.is_empty());
}

/// A and B chatting (A started it, the player's order), the others elsewhere on the lot,
/// nobody choosing anything for themselves yet.
fn chatting(others: &str) -> World {
    let mut w = town(
        &format!(r#"{{"name":"A","x":5.5,"z":5.5}},{{"name":"B","x":7.5,"z":5.5}}{others}"#),
        "",
    );
    w.autonomy = false;
    let chat = w.content.social_index("chat").unwrap();
    w.apply(Command::Social {
        sim: 0,
        target: 1,
        social: chat,
    })
    .unwrap();
    for _ in 0..secs(5.0) {
        w.tick_once();
        if w.sims[0].conversation(&w.content).is_some() {
            return w;
        }
    }
    panic!("A and B never started talking");
}

fn join(w: &mut World, sim: usize, leader: u32) {
    w.sims[sim].queue.push_back(Task {
        kind: TaskKind::Join { leader },
        directed: false,
        routine: false,
    });
}

fn listeners(w: &World, leader: u32) -> usize {
    w.sims
        .iter()
        .filter(|s| s.listening_to() == Some(leader))
        .count()
}

fn apart(p: [f32; 2], q: [f32; 2]) -> f32 {
    (p[0] - q[0]).hypot(p[1] - q[1])
}

#[test]
fn a_lonely_third_joins_in_listens_and_is_released_afterwards() {
    let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
    w.sims[2].needs[0] = 0.05;
    // C chooses for themselves: A is talking and B is being talked to, so joining in is all
    // there is.
    w.autonomy = true;
    let mut joined = false;
    for _ in 0..secs(6.0) {
        w.tick_once();
        if w.sims[2].listening_to() == Some(0) {
            joined = true;
            break;
        }
    }
    assert!(joined, "C never joined in");
    w.autonomy = false;
    let before = w.sims[2].needs[0];
    let (c, a, b) = (w.sims[2].pos, w.sims[0].pos, w.sims[1].pos);
    assert!(
        apart(c, a) < 2.0 && apart(c, b) < 2.0,
        "C listens from {c:?}, A at {a:?}, B at {b:?}"
    );
    assert!(
        apart(c, a) >= 0.6 && apart(c, b) >= 0.6,
        "C stands on someone"
    );
    // The snapshot: C listens to A's chat (role 3, A as partner).
    let mut out = vec![0.0; crate::snapshot::CAPACITY];
    crate::snapshot::write(&w, &mut out);
    let row = |i: usize| crate::snapshot::HEADER_LEN + i * crate::snapshot::SIM_STRIDE;
    use crate::snapshot::sim;
    assert_eq!(out[row(2) + sim::ROLE], 3.0);
    assert_eq!(out[row(2) + sim::PARTNER], 0.0);
    assert_eq!(out[row(2) + sim::SOCIAL], out[row(0) + sim::SOCIAL]);
    assert!(!w.sims[2].available_for_social(&w.content));

    let friends = (
        w.relationships.get(2, 0).friendship,
        w.relationships.get(1, 2).friendship,
    );
    let mut ended = false;
    for _ in 0..mins(15.0) {
        w.tick_once();
        if w.sims[0].conversation(&w.content).is_none() {
            ended = true;
            break;
        }
    }
    assert!(ended);
    assert!(
        w.sims[2].needs[0] > before,
        "listening filled C's social need"
    );
    let gain = w.content.social_rules.join_friendship;
    assert_eq!(w.relationships.get(2, 0).friendship, friends.0 + gain);
    assert_eq!(w.relationships.get(1, 2).friendship, friends.1 + gain);
    assert!(
        w.sims[2]
            .feelings
            .iter()
            .any(|f| w.content.feelings[f.def].id == "goodChat")
    );
    // A doesn't talk again (nobody chooses): C goes after a little while.
    ticks(&mut w, super::REGROUP_TICKS as u64 + 2);
    assert!(
        w.sims[2].current.is_none(),
        "C still {:?}",
        w.sims[2].current.as_ref().map(|a| a.task.kind)
    );
    assert!(w.sims[2].available_for_social(&w.content));
}

#[test]
fn listeners_stay_for_the_next_social_with_the_group() {
    let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
    join(&mut w, 2, 0);
    ticks(&mut w, secs(5.0));
    assert_eq!(listeners(&w, 0), 1);
    // The player has A chat with B again right after: C stays for it.
    let chat = w.content.social_index("chat").unwrap();
    w.apply(Command::Social {
        sim: 0,
        target: 1,
        social: chat,
    })
    .unwrap();
    let mut gap = false;
    for _ in 0..mins(30.0) {
        w.tick_once();
        if w.sims[0].conversation(&w.content).is_none() {
            gap = true;
            assert!(
                matches!(
                    w.sims[2].current.as_ref().map(|a| a.task.kind),
                    Some(TaskKind::Join { leader: 0 })
                ),
                "C left between the two"
            );
        } else if gap {
            break;
        }
    }
    assert!(gap && w.sims[0].conversation(&w.content).is_some());
    ticks(&mut w, 2);
    assert_eq!(listeners(&w, 0), 1, "C listens to the second one too");
}

#[test]
fn cancelling_or_leaving_releases_those_listening() {
    // The leader's conversation cancelled: the listener goes.
    let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
    join(&mut w, 2, 0);
    ticks(&mut w, secs(5.0));
    assert_eq!(listeners(&w, 0), 1);
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    ticks(&mut w, super::REGROUP_TICKS as u64 + 2);
    assert!(w.sims[2].current.is_none());
    assert_eq!(w.sims[1].engaged_with, None);

    // The listener called away: the others talk on.
    let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
    join(&mut w, 2, 0);
    ticks(&mut w, secs(5.0));
    assert_eq!(listeners(&w, 0), 1);
    w.apply(Command::Cancel { sim: 2, index: 0 }).unwrap();
    ticks(&mut w, 2);
    assert_eq!(listeners(&w, 0), 0);
    assert!(w.sims[2].current.is_none());
    assert!(w.sims[0].conversation(&w.content).is_some());
    assert_eq!(w.sims[1].engaged_with, Some(0));

    // The one who started it leaves town: nobody waits for them.
    let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
    join(&mut w, 2, 0);
    ticks(&mut w, secs(5.0));
    assert_eq!(listeners(&w, 0), 1);
    w.depart(0, crate::world::GoneWhy::MovedAway);
    assert!(w.sims[2].current.is_none());
    ticks(&mut w, 2);
    assert!(w.sims[2].current.is_none() && w.sims[1].engaged_with.is_none());
}

#[test]
fn a_conversation_takes_four_at_most() {
    let mut w = chatting(
        r#",{"name":"C","x":9.5,"z":9.5},{"name":"D","x":2.5,"z":9.5},{"name":"E","x":5.5,"z":1.5}"#,
    );
    for k in 2..5 {
        join(&mut w, k, 0);
    }
    for _ in 0..secs(6.0) {
        w.tick_once();
        assert!(listeners(&w, 0) <= 2);
    }
    assert_eq!(listeners(&w, 0), 2);
    let group = super::group(&w.sims, &w.content, 0);
    assert_eq!(group.len(), super::MAX_GROUP);
    // The one left out does something else.
    let out = (2..5)
        .find(|&k| w.sims[k].listening_to().is_none())
        .unwrap();
    assert!(w.sims[out].current.is_none());
    // Everyone in the group stands apart.
    for (x, &a) in group.iter().enumerate() {
        for &b in &group[x + 1..] {
            assert!(
                apart(w.sims[a].pos, w.sims[b].pos) >= 0.59,
                "{a} and {b} too close"
            );
        }
    }
}

/// How far (radians) `sim`'s yaw is from facing `to`.
fn off(w: &World, sim: usize, to: [f32; 2]) -> f32 {
    let p = w.sims[sim].pos;
    let want = (to[0] - p[0]).atan2(to[1] - p[1]);
    let d = (w.sims[sim].yaw - want).rem_euclid(std::f32::consts::TAU);
    d.min(std::f32::consts::TAU - d)
}

fn centroid(w: &World, members: &[usize]) -> [f32; 2] {
    let n = members.len() as f32;
    let sum = members.iter().fold([0.0, 0.0], |c, &m| {
        [c[0] + w.sims[m].pos[0], c[1] + w.sims[m].pos[1]]
    });
    [sum[0] / n, sum[1] / n]
}

#[test]
fn the_two_talking_open_up_to_those_listening() {
    let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
    // Just the two: face to face.
    ticks(&mut w, 2);
    let (a, b) = (w.sims[0].pos, w.sims[1].pos);
    assert!(
        off(&w, 0, b) < 0.01 && off(&w, 1, a) < 0.01,
        "A and B don't face each other"
    );

    join(&mut w, 2, 0);
    ticks(&mut w, secs(5.0));
    assert_eq!(listeners(&w, 0), 1);
    let group = super::group(&w.sims, &w.content, 0);
    assert_eq!(group.len(), 3);
    let centre = centroid(&w, &group);
    for m in group {
        assert!(
            off(&w, m, centre) < 0.05,
            "{m} doesn't face the middle of the group"
        );
    }
    // Not straight at each other any more: both turned part way towards C (about 30 degrees,
    // standing in a triangle).
    let (a, b, c) = (w.sims[0].pos, w.sims[1].pos, w.sims[2].pos);
    assert!(
        off(&w, 0, b) > 0.3 && off(&w, 1, a) > 0.3,
        "A and B still face each other"
    );
    let between = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| {
        let d = ((r[0] - p[0]).atan2(r[1] - p[1]) - (q[0] - p[0]).atan2(q[1] - p[1]))
            .rem_euclid(std::f32::consts::TAU);
        d.min(std::f32::consts::TAU - d)
    };
    assert!(
        off(&w, 0, c) < between(a, b, c) && off(&w, 1, c) < between(b, a, c),
        "they don't turn towards C"
    );

    // C goes: back to face to face.
    w.apply(Command::Cancel { sim: 2, index: 0 }).unwrap();
    ticks(&mut w, 2);
    assert_eq!(listeners(&w, 0), 0);
    let (a, b) = (w.sims[0].pos, w.sims[1].pos);
    assert!(
        off(&w, 0, b) < 0.01 && off(&w, 1, a) < 0.01,
        "A and B don't face each other again"
    );
}

#[test]
fn listening_in_is_not_saved() {
    let mut w = chatting(r#",{"name":"C","x":9.5,"z":9.5}"#);
    join(&mut w, 2, 0);
    ticks(&mut w, secs(5.0));
    assert_eq!(listeners(&w, 0), 1);
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert!(loaded.sims[2].current.is_none() && loaded.sims[2].queue.is_empty());
}

/// The snapshot's `OUTCOME` for Sim `i`, ticking on until the two are in the social.
fn outcome_shown(w: &World, i: usize) -> f32 {
    let mut out = vec![0.0; crate::snapshot::CAPACITY];
    crate::snapshot::write(w, &mut out);
    out[crate::snapshot::HEADER_LEN
        + i * crate::snapshot::SIM_STRIDE
        + crate::snapshot::sim::OUTCOME]
}

#[test]
fn a_refused_hug_shows_at_once_a_chat_halfway_through() {
    // A hug is answered by the body, from the start (the renderer's step back, `characters.ts`).
    let mut w = town(
        r#"{"name":"A","x":5.5,"z":5.5},{"name":"B","x":7.5,"z":5.5}"#,
        "",
    );
    w.autonomy = false;
    let hug = w.content.social_index("hug").unwrap();
    w.apply(Command::Social {
        sim: 0,
        target: 1,
        social: hug,
    })
    .unwrap();
    let mut seen = None;
    for _ in 0..secs(5.0) {
        w.tick_once();
        if let Some((_, _, success, progress)) = w.sims[0].conversation(&w.content) {
            seen = Some((
                success,
                progress,
                outcome_shown(&w, 0),
                outcome_shown(&w, 1),
            ));
            break;
        }
    }
    let (success, progress, a, b) = seen.expect("A never started the hug");
    assert!(
        !success && progress < 0.1,
        "refused, just begun: {progress}"
    );
    assert_eq!((a, b), (2.0, 2.0), "the refusal shows from the start");
    // A chat's outcome shows only halfway through (the answer after the opener).
    let w = chatting("");
    assert!(
        w.sims[0]
            .conversation(&w.content)
            .is_some_and(|c| c.3 < 0.5)
    );
    assert_eq!(outcome_shown(&w, 1), 0.0);
}

/// `within_earshot` as it was before it stopped at the first way within earshot and kept its
/// search's room (a cheapest-first search to the end, over a SipHash map): the answers to keep.
fn earshot_as_before(nav: &crate::path::NavGrid, a: [f32; 2], b: [f32; 2]) -> bool {
    use super::{DOOR_MUFFLE, EARSHOT, Edge, STOREY_WALK, shut};
    if nav.stairs.distance(a, b) > EARSHOT {
        return false;
    }
    let lot = nav.lot;
    let tile = |p: [f32; 2]| (p[0].floor() as i32, p[1].floor() as i32);
    let (start, goal) = (tile(a), tile(b));
    if start == goal {
        return true;
    }
    let tenths = |m: f32| (m * 10.0).round() as u32;
    let (limit, muffle) = (tenths(EARSHOT), tenths(DOOR_MUFFLE));
    let storey = |z: i32| nav.stairs.storey(z);
    let open = |(ax, az): (i32, i32), (bx, bz): (i32, i32)| -> Option<u32> {
        if !lot.in_bounds(bx, bz) || lot.diag_blocks(bx, bz) || storey(az) != storey(bz) {
            return None;
        }
        match lot.edge_between(ax, az, bx, bz) {
            _ if shut(lot, (ax, az), (bx, bz)) => None,
            Edge::Door => Some(muffle),
            _ => Some(0),
        }
    };
    let mut best = std::collections::HashMap::from([(start, 0u32)]);
    let mut queue = std::collections::BinaryHeap::from([std::cmp::Reverse((0u32, start))]);
    while let Some(std::cmp::Reverse((cost, at))) = queue.pop() {
        if at == goal {
            return true;
        }
        if best.get(&at).is_some_and(|&c| c < cost) {
            continue;
        }
        let mut next: Vec<((i32, i32), u32)> = Vec::with_capacity(9);
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let to = (at.0 + dx, at.1 + dz);
            if let Some(extra) = open(at, to) {
                next.push((to, 10 + extra));
            }
        }
        for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let (side_x, side_z) = ((at.0 + dx, at.1), (at.0, at.1 + dz));
            let to = (at.0 + dx, at.1 + dz);
            let clear = [
                open(at, side_x),
                open(side_x, to),
                open(at, side_z),
                open(side_z, to),
            ]
            .iter()
            .all(|e| *e == Some(0));
            if clear {
                next.push((to, 14));
            }
        }
        if let Some(to) = nav.stairs.link(at.0, at.1) {
            next.push((to, tenths(STOREY_WALK)));
        }
        for (to, step) in next {
            let c = cost + step;
            if c <= limit && best.get(&to).is_none_or(|&b| c < b) {
                best.insert(to, c);
                queue.push(std::cmp::Reverse((c, to)));
            }
        }
    }
    false
}

/// The shipped content, merged as the web loads it.
fn shipped_content() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/content");
    let read = |n: &str| std::fs::read_to_string(dir.join(n)).unwrap();
    let base = read("base.json");
    let includes: Vec<String> = serde_json::from_value(
        serde_json::from_str::<serde_json::Value>(&base).unwrap()["include"].clone(),
    )
    .unwrap();
    let parts: Vec<String> = includes.iter().map(|n| read(n)).collect();
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
    crate::merge_content(&base, &parts).unwrap()
}

#[test]
fn earshot_answers_as_it_did_before_it_was_made_cheap() {
    // Saved towns with houses of storeys, stairs, doors, windows and gardens: from every
    // resident, and from spots round them, to spots up to a little past earshot away (both
    // ways), each answered as the search to the end answered it.
    let content = shipped_content();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/saves");
    let (mut asked, mut heard) = (0, 0);
    for name in ["v042_village.json", "v035_normal.json", "live_day5.json"] {
        let json = std::fs::read_to_string(dir.join(name)).unwrap();
        let w = World::from_save_json(&content, &json).unwrap();
        let nav = w.nav();
        let mut rng = crate::rng::Rng::new(7);
        for s in &w.sims {
            for _ in 0..12 {
                let a = [
                    s.pos[0] + rng.range(-6.0, 6.0),
                    s.pos[1] + rng.range(-6.0, 6.0),
                ];
                // (One `Earshot` from each spot, asked about every other.)
                for from in [a, s.pos] {
                    let mut earshot = super::Earshot::new(&nav, from);
                    for _ in 0..10 {
                        let b = [a[0] + rng.range(-14.0, 14.0), a[1] + rng.range(-14.0, 14.0)];
                        for (p, q) in [(from, b), (b, from)] {
                            let want = earshot_as_before(&nav, p, q);
                            assert_eq!(
                                super::within_earshot(&nav, p, q),
                                want,
                                "{name}: {p:?} to {q:?}"
                            );
                            asked += 1;
                            heard += usize::from(want);
                        }
                        assert_eq!(
                            earshot.hears(b),
                            earshot_as_before(&nav, from, b),
                            "{name}: from {from:?} to {b:?}"
                        );
                    }
                }
            }
        }
    }
    // (Enough of both answers to mean something.)
    assert!(
        asked > 3000 && heard > 300 && asked - heard > 300,
        "{heard} of {asked}"
    );
}
