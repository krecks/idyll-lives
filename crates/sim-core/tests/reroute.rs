//! A house that changes under a walker: a wall across their way makes them give up (rather than
//! walk into it), and one they can go round sends them round.

use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind};
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.0}],
    "objects":[],
    "visits":{"needs":{"hunger":0.5},"hours":2,"minFriendship":-100,"earliestHour":0,"latestHour":24}}"#;

/// One long room, the front door at its left end; Ada sets off for the right end.
const LOT: &str = r#"{"width":24,"depth":10,
    "plots":[{"name":"Home","x":0,"z":0,"w":24,"d":10,"entry":[1.5,5.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[],
    "sims":[{"name":"Ada","household":0,"x":2.5,"z":5.5}]}"#;

fn wall(x: i32, z0: i32, z1: i32) -> Vec<EdgeEdit> {
    (z0..z1).map(|z| EdgeEdit::new(EdgeAxis::V, x, z, EdgeKind::Wall)).collect()
}

fn walking() -> World {
    let mut w = World::from_json(CONTENT, LOT, 3).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w.apply(Command::MoveTo { sim: 0, x: 21.5, z: 5.5 }).unwrap();
    // Far enough along to be well past the left end, short of the new wall.
    for _ in 0..20 {
        w.tick_once();
    }
    assert!(w.sims[0].current().is_some(), "should still be on the way");
    assert!(w.sims[0].pos[0] < 12.0);
    w
}

#[test]
fn a_wall_across_the_way_makes_them_give_up() {
    let mut w = walking();
    w.apply(Command::Build { household: 0, edits: wall(12, 0, 10) }).unwrap();
    w.tick_once();
    assert!(w.sims[0].current().is_none(), "no way there any more");
}

#[test]
fn a_wall_they_can_go_round_sends_them_round() {
    let mut w = walking();
    // The wall stops short of the far side, so there's still a way round it.
    w.apply(Command::Build { household: 0, edits: wall(12, 0, 8) }).unwrap();
    for _ in 0..600 {
        w.tick_once();
        if w.sims[0].current().is_none() {
            break;
        }
    }
    assert!(w.sims[0].current().is_none(), "never got there");
    assert!(w.sims[0].pos[0] > 20.0, "ended at {:?}", w.sims[0].pos);
}
