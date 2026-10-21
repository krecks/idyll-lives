//! Calling out with the shipped content (`conversation.rs`): a lonely resident calls a housemate
//! a few metres off by name, the housemate turns to them, and the two meet halfway to talk.

mod common;

use sim_core::content::Pose;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World, clock};

#[test]
fn a_lonely_housemate_calls_another_over() {
    let content = common::content();
    let mut w = World::from_json(&content, &common::town(&content, 7, 3), 7).unwrap();
    let h = w.households.iter().position(|h| h.player).unwrap() as u32;
    let home: Vec<usize> = (0..w.sims.len()).filter(|&i| w.sims[i].household == h).collect();
    let (a, b) = (home[0], home[1]);
    // Morning (before visiting hours), everyone rested and fed; A feels lonely, gets on well
    // with B and knows nobody else.
    w.autonomy = false;
    while w.tick < clock::tick_at(1, 9.0 * 60.0).unwrap() {
        w.tick_once();
    }
    // B steps a few metres off, still within earshot of A.
    let from = w.sims[a].pos;
    let spots = [7.0f32, 6.0, 5.0, 8.0]
        .into_iter()
        .flat_map(|r| [(1.0f32, 0.0f32), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)].map(|(dx, dz)| [from[0] + dx * r, from[1] + dz * r]))
        .filter(|&p| w.within_earshot(from, p))
        .collect::<Vec<_>>();
    let mut there = false;
    for spot in spots {
        w.apply(Command::MoveTo { sim: b as u32, x: spot[0], z: spot[1] }).unwrap();
        for _ in 0..sim_core::TICKS_PER_SECOND * 10 {
            w.tick_once();
        }
        let p = w.sims[b].pos;
        there = (p[0] - spot[0]).hypot(p[1] - spot[1]) < 0.3 && w.within_earshot(w.sims[a].pos, p);
        if there {
            break;
        }
    }
    assert!(there, "B found nowhere to stand within earshot of A");
    for &i in &home {
        let n = w.content.needs.len();
        w.sims[i].needs[..n].fill(1.0);
    }
    let social = w.content.needs.iter().position(|n| n.id == "social").unwrap();
    w.sims[a].needs[social] = 0.05;
    // B could do with some fun, so free will finds them something to pass the time with (a book,
    // the TV) that a call can interrupt, not a meal to cook for a household that has eaten.
    let fun = w.content.needs.iter().position(|n| n.id == "fun").unwrap();
    w.sims[b].needs[fun] = 0.6;
    for x in (0..w.sims.len()).filter(|&x| x != a && x != b) {
        *w.relationships.get_mut(a, x) = Default::default();
        *w.relationships.get_mut(x, a) = Default::default();
    }
    w.relationships.get_mut(a, b).friendship = 70.0;
    w.relationships.get_mut(b, a).friendship = 70.0;
    w.autonomy = true;
    common::dump_state(&w, "calling-out");

    let (a_from, mut b_from) = (w.sims[a].pos, None);
    let mut heeded = false;
    for _ in 0..(10.0 / sim_core::MINUTES_PER_TICK) as u64 {
        w.tick_once();
        let act = w.sims[a].current();
        if act.is_some_and(|act| matches!((act.task.kind, &act.phase), (TaskKind::Social { target, .. }, Phase::Calling { .. }) if target as usize == b)) {
            b_from.get_or_insert(w.sims[b].pos);
        }
        // Turned to A as they heard their name.
        if !heeded && w.sims[b].heeding == Some(a as u32) {
            heeded = true;
            let (p, q) = (w.sims[b].pos, w.sims[a].pos);
            let towards = (q[0] - p[0]).atan2(q[1] - p[1]);
            let off = (w.sims[b].yaw - towards).rem_euclid(std::f32::consts::TAU);
            assert!(off.min(std::f32::consts::TAU - off) < 0.1, "B didn't turn to A");
        }
        if w.sims[a].conversation(&w.content).is_some_and(|c| c.0 as usize == b) {
            break;
        }
    }
    let b_from = b_from.expect("A never called B");
    assert!(heeded, "B never heard A");
    assert!(w.sims[a].conversation(&w.content).is_some(), "A and B never talked");
    let walked = |p: [f32; 2], q: [f32; 2]| (p[0] - q[0]).hypot(p[1] - q[1]);
    let (by_a, by_b) = (walked(a_from, w.sims[a].pos), walked(b_from, w.sims[b].pos));
    eprintln!("A walked {by_a:.1} m, B {by_b:.1} m");
    assert!(by_a > 1.0 && by_b > 1.0, "they didn't meet halfway: A walked {by_a:.1} m, B {by_b:.1} m");
}

#[test]
fn someone_on_the_sofa_stays_sitting_when_called_and_is_walked_over_to() {
    let (mut w, home, content) = common::household_of_three();
    // A feels lonely and knows nobody else to talk to; B is on the sofa.
    let (a, b) = (home[0], home[1]);
    for &x in &home[2..] {
        *w.relationships.get_mut(a, x) = Default::default();
        *w.relationships.get_mut(x, a) = Default::default();
    }
    let mut w = common::sit_on_the_sofa(w, &content, &[b]);
    let seat = w.sims[b].pos;
    // A a few metres off, within earshot.
    let spots: Vec<[f32; 2]> = [5.0f32, 4.0, 6.0, 3.0]
        .into_iter()
        .flat_map(|r| [(0.0f32, 1.0f32), (1.0, 0.0), (-1.0, 0.0), (0.0, -1.0)].map(|(dx, dz)| [seat[0] + dx * r, seat[1] + dz * r]))
        .filter(|&p| w.within_earshot(seat, p))
        .collect();
    let mut there = false;
    for spot in spots {
        if w.apply(Command::MoveTo { sim: a as u32, x: spot[0], z: spot[1] }).is_err() {
            continue;
        }
        for _ in 0..(10.0 / sim_core::MINUTES_PER_TICK) as u64 {
            w.tick_once();
            if common::apart(w.sims[a].pos, spot) < 0.3 {
                there = true;
                break;
            }
        }
        if there {
            break;
        }
    }
    assert!(there, "A found nowhere to stand within earshot of the sofa");
    let social = w.content.needs.iter().position(|n| n.id == "social").unwrap();
    let n = w.content.needs.len();
    w.sims[a].needs[..n].fill(1.0);
    w.sims[a].needs[social] = 0.05;
    let from = w.sims[a].pos;
    w.autonomy = true;
    let mut talked = false;
    for _ in 0..(10.0 / sim_core::MINUTES_PER_TICK) as u64 {
        w.tick_once();
        assert_eq!(w.sims[b].pose, Pose::Sit, "B got up");
        assert!(common::apart(w.sims[b].pos, seat) < 0.05, "B left the sofa");
        assert_eq!(w.sims[b].heeding, None, "B set off to meet A");
        if w.sims[a].conversation(&w.content).is_some_and(|c| c.0 as usize == b) {
            talked = true;
            break;
        }
    }
    assert!(talked, "A and B never talked: {:?}", w.sims[a].current().map(|act| act.task.kind));
    assert!(common::apart(from, w.sims[a].pos) > 1.0, "A didn't come over");
    // The talk over, B is still on the sofa, at what they sat down to do.
    for _ in 0..(30.0 / sim_core::MINUTES_PER_TICK) as u64 {
        w.tick_once();
        assert_eq!(w.sims[b].pose, Pose::Sit, "B got up");
        if w.sims[a].conversation(&w.content).is_none() {
            break;
        }
    }
    assert!(matches!(w.sims[b].current().map(|act| (act.task.kind, &act.phase)), Some((TaskKind::Use { .. }, Phase::Using { .. }))), "B: {:?}", w.sims[b].current().map(|act| act.task.kind));
}
