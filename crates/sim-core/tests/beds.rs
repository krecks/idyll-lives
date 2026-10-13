//! A child out of the crib needs a bed: with fewer bed places at home than people past the crib,
//! a household in town buys one (the cheapest); the player's household is told, once, and buys
//! its own.

use sim_core::social::EventKind;
use sim_core::{Command, World, clock};

const CONTENT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.02}],
    "dayRhythm":{"sleepTags":["sleep"]},
    "objects":[{"id":"crib","name":"Crib","price":350,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"lie","label":"Lie in the crib","minutes":720,"pose":"lie","baby":true,"autonomous":false,"effects":{"energy":1.0},"tags":["rest"]}]},
      {"id":"bed","name":"Bed","price":400,"footprint":[1,2],"interactions":[
        {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"]}]},
      {"id":"bigBed","name":"Big bed","price":900,"footprint":[2,2],"slots":2,"interactions":[
        {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"]}]}],
    "life":{"daysPerYear":2,"stages":[
        {"id":"baby","label":"Baby","from":0,"baby":true},
        {"id":"child","label":"Child","from":2},
        {"id":"adult","label":"Adult","from":18}],
      "adoption":{"cost":100}},
    "genders":[{"id":"female","label":"Female"},{"id":"male","label":"Male"}],
    "names":{"first":["Ivy","Jun"],"last":["Moss"]},
    "bondPresets":{"parent":{"friendship":50,"kin":"parent"}},
    "economy":{"startingFunds":1000}}"#;

/// The player's home on the left, the neighbours' on the right: a grown-up, a bed for them and a
/// baby about to grow out of the crib in each.
const TOWN: &str = r#"{"width":24,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[5.5,11.5]},
             {"name":"Next door","x":12,"z":0,"w":12,"d":12,"entry":[17.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true},{"name":"Neighbours","plot":1}],
    "objects":[{"def":"crib","x":2,"z":2},{"def":"bed","x":8,"z":2},{"def":"crib","x":14,"z":2},{"def":"bed","x":20,"z":2}],
    "sims":[{"name":"Ada","household":0,"x":4.5,"z":8.5,"age":30},{"name":"Bea","household":0,"x":5.5,"z":8.5,"age":1.99},
            {"name":"Cal","household":1,"x":16.5,"z":8.5,"age":30},{"name":"Dot","household":1,"x":17.5,"z":8.5,"age":1.99}],
    "relationships":[{"a":1,"b":0,"preset":"parent"},{"a":3,"b":2,"preset":"parent"}]}"#;

fn beds(w: &World, from_x: i32) -> usize {
    w.objects.iter().filter(|o| o.x >= from_x && o.x < from_x + 12 && w.content.objects[o.def].id.ends_with("ed")).count()
}

fn until(w: &mut World, day: u32) {
    let end = clock::tick_at(day, 0.0).unwrap();
    while w.tick < end {
        w.tick_once();
    }
}

#[test]
fn neighbours_buy_a_bed_for_a_child_out_of_the_crib() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let funds = w.households[1].funds;
    until(&mut w, 2);
    assert!(!w.content.life.baby(w.sims[3].age), "Dot is a child now");
    assert_eq!(beds(&w, 12), 2, "a bed for Dot next door");
    assert_eq!(w.households[1].funds, funds - 400, "the cheapest bed, paid for");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::NoBed && e.a == 3), "nobody's told: it's done");
}

#[test]
fn the_player_is_told_once_that_a_child_has_no_bed() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let funds = w.households[0].funds;
    until(&mut w, 2);
    assert!(!w.content.life.baby(w.sims[1].age), "Bea is a child now");
    assert_eq!(beds(&w, 0), 1, "nothing bought for the player");
    assert_eq!(w.households[0].funds, funds);
    let told = |w: &World| w.events.iter().filter(|e| e.kind == EventKind::NoBed && e.a == 1).count();
    assert_eq!(told(&w), 1, "the player is told Bea has no bed");
    assert!(EventKind::NoBed.importance() >= 1, "it shows in the feed");
    until(&mut w, 4);
    assert_eq!(told(&w), 1, "once, not every night");
}

#[test]
fn a_child_adopted_into_a_home_without_a_bed_for_them() {
    // Bed places for everyone past the crib: nothing to say or buy.
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::Adopt { household: 1, child: false }).unwrap();
    assert_eq!(beds(&w, 12), 1, "a baby needs a crib, not a bed");
    // A child: a bed (next door), a notice (at home).
    w.apply(Command::Adopt { household: 1, child: true }).unwrap();
    assert_eq!(beds(&w, 12), 2, "a bed for the child next door");
    w.apply(Command::Adopt { household: 0, child: true }).unwrap();
    assert_eq!(w.events.iter().filter(|e| e.kind == EventKind::NoBed).count(), 1, "the player is told");
}

#[test]
fn neighbours_short_of_money_dont_buy_into_debt() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.households[1].funds = 300;
    until(&mut w, 2);
    assert!(!w.content.life.baby(w.sims[3].age), "Dot is a child now");
    assert_eq!(beds(&w, 12), 1, "no bed they can't pay for");
    assert_eq!(w.households[1].funds, 300);
}
