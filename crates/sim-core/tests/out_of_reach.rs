//! What residents can't get to: they remember it for a while and choose something else (the next
//! dirtiest spot, not the one behind a wall again and again), and a visit with no way there
//! doesn't make them a guest at home.

use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.0},{"id":"environment","label":"Surroundings","room":true}],
    "traits":[{"id":"neat","label":"Neat","effects":{"tagPreference":{"chores":3}}}],
    "objects":[],
    "roomRules":{"clean":{"minutes":10,"threshold":0.15,"amount":0.7,"interest":0.6,"tags":["chores"]}},
    "visits":{"needs":{"hunger":0.5},"hours":2,"minFriendship":-100,"earliestHour":0,"latestHour":24}}"#;

/// A room with no way in (walls all round) on the left of the home; the neighbours' plot is
/// across a wall that runs the whole depth of the lot.
const TOWN: &str = r#"{"width":24,"depth":10,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]},{"name":"Next door","x":12,"z":0,"w":12,"d":10,"entry":[17.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true},{"name":"Neighbours","plot":1}],
    "walls":[[1,1,6,1],[1,5,6,5],[1,1,1,5],[6,1,6,5],[12,0,12,10]],
    "objects":[],
    "sims":[{"name":"Ada","household":0,"x":9.5,"z":3.5,"traits":["neat"]},{"name":"Bo","household":1,"x":17.5,"z":3.5}]}"#;

#[test]
fn a_mess_out_of_reach_doesnt_keep_them_from_the_one_they_can_get_to() {
    let mut w = World::from_json(CONTENT, TOWN, 3).unwrap();
    let (shut_in, open) = (w.lot.tile_index(3, 3), w.lot.tile_index(9, 7));
    w.dirt[shut_in] = 0.9;
    w.dirt[open] = 0.5;
    for _ in 0..30 * 20 {
        w.tick_once();
    }
    assert!(w.dirt[open] < 0.1, "the reachable mess is still there: {:.2}", w.dirt[open]);
    assert!((w.dirt[shut_in] - 0.9).abs() < 1e-4);
}

#[test]
fn a_visit_with_no_way_there_doesnt_make_them_a_guest() {
    let mut w = World::from_json(CONTENT, TOWN, 3).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w.apply(Command::Visit { sim: 0, plot: 1 }).unwrap();
    for _ in 0..20 {
        w.tick_once();
    }
    assert!(w.sims[0].current().is_none(), "no way over");
    assert!(w.sims[0].visiting.is_none(), "a guest next door while still at home");
}
