//! Group conversations with the shipped content: while two residents chat, a lonely third one
//! comes over and joins in (`conversation.rs`, socials with `joinable`).

mod common;

use sim_core::world::TaskKind;
use sim_core::{Command, World, clock};

#[test]
fn a_lonely_housemate_joins_a_chat_at_home() {
    let content = common::content();
    let mut w = World::from_json(&content, &common::town(&content, 7, 3), 7).unwrap();
    let h = w.households.iter().position(|h| h.player).unwrap() as u32;
    let home: Vec<usize> = (0..w.sims.len()).filter(|&i| w.sims[i].household == h).collect();
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
    let social = w.content.needs.iter().position(|n| n.id == "social").unwrap();
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
    w.apply(Command::Social { sim: a as u32, target: b as u32, social: chat }).unwrap();
    w.autonomy = true;
    let mut joined = false;
    for _ in 0..(15.0 / sim_core::MINUTES_PER_TICK) as u64 {
        w.tick_once();
        if w.sims[c].listening_to() == Some(a as u32) {
            joined = true;
            break;
        }
    }
    assert!(joined, "C didn't join in: {:?}", w.sims[c].current().map(|act| act.task.kind));
    assert!(matches!(w.sims[c].current().map(|act| act.task.kind), Some(TaskKind::Join { .. })));
    let near = |p: [f32; 2], q: [f32; 2]| (p[0] - q[0]).hypot(p[1] - q[1]) < 2.2;
    assert!(near(w.sims[c].pos, w.sims[a].pos) && near(w.sims[c].pos, w.sims[b].pos));
    for _ in 0..sim_core::TICKS_PER_SECOND * 3 {
        w.tick_once();
    }
    common::dump_state(&w, "group-talk");
}
