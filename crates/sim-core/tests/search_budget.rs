//! A tick's searches have a budget in an open-world town (`World::search_budget`,
//! `world::SEARCH_BUDGET`): once they've expanded that many tiles, a resident whose turn may look
//! for a way waits for the next tick, and goes then whatever it costs. The player's household
//! and whoever is watched never wait.

mod common;

use common::hamlets::two_hamlets;
use sim_core::world::Phase;
use sim_core::{Command, World};

fn walking(w: &World, i: usize) -> bool {
    matches!(
        w.sims[i].current().map(|a| &a.phase),
        Some(Phase::Routing { .. })
    )
}

#[test]
fn over_budget_they_wait_a_tick_and_the_player_goes_first() {
    let content = common::content();
    let h = two_hamlets(&content, 7);
    let mut w = World::from_json(&content, &h.town, 7).expect("the hamlets load");
    w.autonomy = false;
    for hh in 0..w.households.len() as u32 {
        w.apply(Command::SetAutonomy {
            enabled: false,
            household: Some(hh),
        })
        .unwrap();
    }
    w.tick_once();
    // Everyone off to the far end of the other hamlet at once (a long walk by road each).
    let mut sent = Vec::new();
    for i in 0..w.sims.len() {
        let home = w.households[w.sims[i].household as usize].plot.unwrap() as usize;
        let other = if h.plots[0].contains(&home) { 1 } else { 0 };
        let goal = w.plots[h.plots[other].clone().last().unwrap()].arrival_point();
        let ok = w
            .apply(Command::MoveTo {
                sim: i as u32,
                x: goal[0],
                z: goal[1],
            })
            .is_ok();
        if ok {
            sent.push(i);
        }
    }
    let player: Vec<usize> = sent
        .iter()
        .copied()
        .filter(|&i| w.households[w.sims[i].household as usize].player)
        .collect();
    assert!(!player.is_empty() && sent.len() > player.len() + 4);
    // The view on the player's home: the rest aren't watched.
    let home = &w.plots[w
        .households
        .iter()
        .find(|h| h.player)
        .unwrap()
        .plot
        .unwrap() as usize];
    w.lod = true;
    w.set_view(Some([home.x, home.z, home.x + home.w, home.z + home.d]));
    w.search_budget = 20;
    let before = sim_core::path::nodes_searched();
    w.tick_once();
    let spent = sim_core::path::nodes_searched() - before;
    let waiting: Vec<usize> = sent.iter().copied().filter(|&i| !walking(&w, i)).collect();
    eprintln!(
        "{} sent, {} waited, {spent} tiles searched in the first tick",
        sent.len(),
        waiting.len()
    );
    assert!(
        player.iter().all(|&i| walking(&w, i)),
        "the player's household waited"
    );
    assert!(!waiting.is_empty(), "nobody waited over budget");
    assert!(waiting.len() < sent.len(), "everyone waited");
    // The next tick, those who waited go, over budget or not.
    w.tick_once();
    for &i in &waiting {
        assert!(walking(&w, i), "{} waited two ticks", w.sims[i].name);
    }
    // A town without a size has no budget: it plays as it always did, budget or none.
    let town = common::town(&content, 3, 2);
    let run = |budget: u64| {
        let mut w = World::from_json(&content, &town, 3).unwrap();
        w.search_budget = budget;
        for _ in 0..1200 {
            w.tick_once();
        }
        w.sims.iter().map(|s| s.pos).collect::<Vec<_>>()
    };
    assert_eq!(run(0), run(sim_core::world::SEARCH_BUDGET));
}
