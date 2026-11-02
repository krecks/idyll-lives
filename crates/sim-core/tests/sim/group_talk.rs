//! Group conversations with the shipped content: while two residents chat, a lonely third one
//! comes over and joins in (`conversation.rs`, socials with `joinable`); two sitting on the sofa
//! talk from their seats, and the third walks over to listen. One listening in doesn't stay to wait
//! by a hug the leader is taken into (A-social-02).

use crate::common;

use common::apart;
use sim_core::content::Pose;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World, clock};

#[test]
fn two_on_the_sofa_talk_from_their_seats_and_a_third_walks_over_to_listen() {
    let (mut w, mut home, content) = common::household_of_three();
    // On the sofa two who aren't a couple (so what they talk about is something to join in on);
    // A feels lonely.
    let k = (0..3)
        .find(|&k| {
            !w.relationships
                .get(home[(k + 1) % 3], home[(k + 2) % 3])
                .partners
        })
        .unwrap();
    home.rotate_left((k + 1) % 3);
    let (a, b, c) = (home[0], home[1], home[2]);
    let social = w
        .content
        .needs
        .iter()
        .position(|n| n.id == "social")
        .unwrap();
    w.sims[a].needs[social] = 0.05;
    let mut w = common::sit_on_the_sofa(w, &content, &[a, b]);
    let seats = [w.sims[a].pos, w.sims[b].pos];
    w.autonomy = true;
    common::dump_state(&w, "group-seated");
    // (For the browser: the same with C lonely as well.)
    let before = w.sims[c].needs[social];
    w.sims[c].needs[social] = 0.05;
    common::dump_state(&w, "group-seated-lonely");
    w.sims[c].needs[social] = before;
    // A word from the seat: one of them talks to the other, neither getting up.
    let mut talking = None;
    for _ in 0..(30.0 / sim_core::MINUTES_PER_TICK) as u64 {
        w.tick_once();
        talking = [a, b].into_iter().find(|&x| {
            w.sims[x]
                .conversation(&w.content)
                .is_some_and(|t| [a, b].contains(&(t.0 as usize)))
        });
        if talking.is_some() {
            break;
        }
    }
    let leader = talking.unwrap_or_else(|| {
        panic!(
            "nobody talked on the sofa: {:?} / {:?}",
            w.sims[a].current().map(|x| x.task.kind),
            w.sims[b].current().map(|x| x.task.kind)
        )
    });
    let other = if leader == a { b } else { a };
    // C comes into the room, a few metres off, and feels lonely too.
    let from = w.sims[leader].pos;
    let spots: Vec<[f32; 2]> = [4.0f32, 3.5, 3.0, 5.0]
        .into_iter()
        .flat_map(|r| {
            [(0.0f32, 1.0f32), (1.0, 0.0), (-1.0, 0.0), (0.0, -1.0)]
                .map(|(dx, dz)| [from[0] + dx * r, from[1] + dz * r])
        })
        .filter(|&p| w.within_earshot(from, p))
        .collect();
    let mut there = false;
    for spot in spots {
        if w.apply(Command::MoveTo {
            sim: c as u32,
            x: spot[0],
            z: spot[1],
        })
        .is_err()
        {
            continue;
        }
        for _ in 0..(10.0 / sim_core::MINUTES_PER_TICK) as u64 {
            w.tick_once();
            if apart(w.sims[c].pos, spot) < 0.3 {
                there = true;
                break;
            }
        }
        if there {
            break;
        }
    }
    assert!(there, "C found nowhere to stand near the sofa");
    let n = w.content.needs.len();
    w.sims[c].needs[..n].fill(1.0);
    w.sims[c].needs[social] = 0.05;
    eprintln!(
        "C {:.1} m from {leader} on the sofa",
        apart(w.sims[c].pos, from)
    );
    // C comes over to listen, on foot: never more than a step a tick.
    let mut last = w.sims[c].pos;
    let mut stride: f32 = 0.0;
    let mut joined = false;
    for _ in 0..(15.0 / sim_core::MINUTES_PER_TICK) as u64 {
        w.tick_once();
        for (x, seat) in [a, b].into_iter().zip(seats) {
            assert_eq!(w.sims[x].pose, Pose::Sit, "{x} got up to talk");
            assert!(apart(w.sims[x].pos, seat) < 0.05, "{x} left the seat");
        }
        stride = stride.max(apart(last, w.sims[c].pos));
        last = w.sims[c].pos;
        if w.sims[leader].conversation(&w.content).is_none() {
            break;
        }
        if w.sims[c].listening_to() == Some(leader as u32) {
            joined = true;
            common::dump_state(&w, "group-seated-joined");
            break;
        }
        // (Off to do something else: what they'd rather do is join in, so they think again.)
        if w.sims[c]
            .current()
            .is_some_and(|act| !matches!(act.task.kind, TaskKind::Join { .. }))
        {
            w.apply(Command::Cancel {
                sim: c as u32,
                index: 0,
            })
            .unwrap();
        }
    }
    let doing = |w: &World, x: usize| match w.sims[x].current().map(|act| act.task.kind) {
        Some(TaskKind::Use {
            object,
            interaction,
        }) => format!(
            "{}.{}",
            w.content.objects[w.objects[object as usize].def].id, interaction
        ),
        other => format!("{other:?}"),
    };
    assert!(
        joined,
        "C didn't join in: {} (pose {:?}; {leader}: {})",
        doing(&w, c),
        w.sims[c].pose,
        doing(&w, leader)
    );
    assert!(
        stride < 0.35,
        "C jumped {stride:.2} m in a tick to their place"
    );
    assert!(
        apart(w.sims[c].pos, w.sims[leader].pos) < 2.6
            && apart(w.sims[c].pos, w.sims[other].pos) < 2.6
    );
    // The talk over, the two on the sofa carry on sitting there.
    for _ in 0..(30.0 / sim_core::MINUTES_PER_TICK) as u64 {
        w.tick_once();
        if w.sims[leader].conversation(&w.content).is_none() {
            break;
        }
    }
    for x in [a, b] {
        assert_eq!(w.sims[x].pose, Pose::Sit, "{x} stood up after the talk");
        assert!(
            matches!(
                w.sims[x].current().map(|act| (act.task.kind, &act.phase)),
                Some((TaskKind::Use { .. }, Phase::Using { .. }))
                    | Some((TaskKind::Social { .. }, _))
            ),
            "{x}: {:?}",
            w.sims[x].current().map(|act| act.task.kind)
        );
    }
}

#[test]
fn a_lonely_housemate_joins_a_chat_at_home() {
    let content = common::content();
    let mut w = World::from_json(&content, &common::town(&content, 7, 3), 7).unwrap();
    let h = w.households.iter().position(|h| h.player).unwrap() as u32;
    let home: Vec<usize> = (0..w.sims.len())
        .filter(|&i| w.sims[i].household == h)
        .collect();
    assert_eq!(home.len(), 3);
    let (a, b, c) = (home[0], home[1], home[2]);
    // Morning (before visiting hours), everyone rested and fed; C feels lonely, gets on well
    // with the other two and knows nobody else to call.
    w.autonomy = false;
    while w.tick < clock::tick_at(1, 9.0 * 60.0).unwrap() {
        w.tick_once();
    }
    for &i in &home {
        let n = w.content.needs.len();
        w.sims[i].needs[..n].fill(1.0);
    }
    let social = w
        .content
        .needs
        .iter()
        .position(|n| n.id == "social")
        .unwrap();
    w.sims[c].needs[social] = 0.05;
    for x in (0..w.sims.len()).filter(|x| !home.contains(x)) {
        *w.relationships.get_mut(c, x) = Default::default();
        *w.relationships.get_mut(x, c) = Default::default();
    }
    for x in [a, b] {
        w.relationships.get_mut(c, x).friendship = 70.0;
        w.relationships.get_mut(x, c).friendship = 70.0;
    }
    let chat = w.content.social_index("chat").unwrap();
    w.apply(Command::Social {
        sim: a as u32,
        target: b as u32,
        social: chat,
    })
    .unwrap();
    w.autonomy = true;
    let mut joined = false;
    // Over to the group on foot, into their place in it: never more than a step a tick.
    let (mut last, mut stride) = (w.sims[c].pos, 0.0f32);
    for _ in 0..(15.0 / sim_core::MINUTES_PER_TICK) as u64 {
        w.tick_once();
        stride = stride.max(apart(last, w.sims[c].pos));
        last = w.sims[c].pos;
        if w.sims[c].listening_to() == Some(a as u32) {
            joined = true;
            break;
        }
    }
    assert!(
        joined,
        "C didn't join in: {:?}",
        w.sims[c].current().map(|act| act.task.kind)
    );
    assert!(matches!(
        w.sims[c].current().map(|act| act.task.kind),
        Some(TaskKind::Join { .. })
    ));
    assert!(
        stride < 0.35,
        "C jumped {stride:.2} m in a tick to their place"
    );
    let near = |p: [f32; 2], q: [f32; 2]| (p[0] - q[0]).hypot(p[1] - q[1]) < 2.2;
    assert!(near(w.sims[c].pos, w.sims[a].pos) && near(w.sims[c].pos, w.sims[b].pos));
    for _ in 0..sim_core::TICKS_PER_SECOND * 3 {
        w.tick_once();
    }
    common::dump_state(&w, "group-talk");
    // The talk broken off, and B hugs A (A-social-02): C, who was listening in, doesn't stand by
    // the hug waiting for A's next word.
    w.autonomy = false;
    w.relationships.get_mut(a, b).friendship = 70.0;
    w.relationships.get_mut(b, a).friendship = 70.0;
    w.apply(Command::Cancel {
        sim: a as u32,
        index: 0,
    })
    .unwrap();
    let hug = w.content.social_index("hug").unwrap();
    w.apply(Command::Social {
        sim: b as u32,
        target: a as u32,
        social: hug,
    })
    .unwrap();
    let hugging = |w: &World| matches!(w.sims[b].current().map(|act| (act.task.kind, &act.phase)), Some((TaskKind::Social { social, .. }, Phase::Conversing { .. })) if social == hug);
    let started = (0..(5.0 / sim_core::MINUTES_PER_TICK) as u64).find(|_| {
        w.tick_once();
        hugging(&w)
    });
    assert!(
        started.is_some(),
        "B hugs A: {:?}",
        w.sims[b].current().map(|act| act.task.kind)
    );
    for _ in 0..2 {
        w.tick_once();
    }
    assert!(hugging(&w));
    assert!(
        !matches!(
            w.sims[c].current().map(|act| act.task.kind),
            Some(TaskKind::Join { .. })
        ),
        "C still waits for A beside the hug"
    );
}
