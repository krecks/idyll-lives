//! How many residents a town grows to: births and newcomers stop at
//! `MAX_SIMS` (768) in every town now (cosy and normal were dropped in 0.8.0), which every town
//! can hold, or sooner where the town file says so (`residentsMost`), which a save keeps.

use sim_core::World;
use sim_core::world::MAX_SIMS;

const CONTENT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.0}],
    "objects":[{"id":"bed","name":"Bed","price":300,"slots":1,"interactions":[
        {"id":"sleep","label":"Sleep","minutes":480,"effects":{"energy":1.0},"tags":["sleep"]}]}],
    "dayRhythm":{"sleepTags":["sleep"]},
    "genders":[{"id":"female","label":"Female"},{"id":"male","label":"Male"}],
    "names":{"first":["Ana","Ben","Cleo","Dev"],"last":["Moss","Reed"]},
    "rules":{"maxHousehold":4},
    "economy":{"startingFunds":1000}}"#;

/// `people` residents (the player and a home for the elderly holding the rest) and a house for
/// sale with a bed, in a town of `size` (none: a town from before sizes).
fn town(size: Option<&str>, people: usize) -> String {
    town_most(size, people, None)
}

/// As `town`, the file saying the most residents it grows to when `most` is given.
fn town_most(size: Option<&str>, people: usize, most: Option<usize>) -> String {
    let mut sims = vec![r#"{"name":"Ada","x":4.5,"z":5.5,"age":30}"#.to_string()];
    sims.extend((1..people).map(|i| {
        format!(
            r#"{{"name":"Old {i}","household":1,"x":{}.5,"z":5.5,"age":80}}"#,
            10 + i % 10
        )
    }));
    let size = size.map_or(String::new(), |s| format!(r#""size":"{s}","#));
    let most = most.map_or(String::new(), |n| format!(r#""residentsMost":{n},"#));
    format!(
        r#"{{"width":30,"depth":10,{size}{most}
        "plots":[{{"name":"Home","x":0,"z":0,"w":10,"d":10,"entry":[5.5,9.5]}},
                 {{"name":"Elders","x":10,"z":0,"w":10,"d":10,"entry":[15.5,9.5]}},
                 {{"name":"For sale","x":20,"z":0,"w":10,"d":10,"entry":[25.5,9.5]}}],
        "households":[{{"name":"Player","plot":0,"player":true}},{{"name":"Elders","plot":1}}],
        "objects":[{{"def":"bed","x":22,"z":2}}],
        "sims":[{}]}}"#,
        sims.join(",")
    )
}

/// Whether a household moves into the house for sale with `people` living in town.
fn moves_in(size: Option<&str>, people: usize) -> bool {
    let mut w = World::from_json(CONTENT, &town(size, people), 1).unwrap();
    assert_eq!(w.sims.len(), people);
    sim_core::lifecycle::arrive(&mut w, 2)
}

/// Whether a household moves in with `people` living in a town whose file says `most`, as it
/// loads and once saved and loaded again.
fn moves_in_most(people: usize, most: usize) -> [bool; 2] {
    let mut w = World::from_json(CONTENT, &town_most(None, people, Some(most)), 1).unwrap();
    let mut saved = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    [&mut w, &mut saved].map(|w| sim_core::lifecycle::arrive(w, 2))
}

#[test]
fn every_town_grows_to_160() {
    for size in [None, Some("cosy"), Some("normal"), Some("big")] {
        assert!(moves_in(size, MAX_SIMS - 1), "{size:?}: room for one more");
        assert!(!moves_in(size, MAX_SIMS), "{size:?}: full at {MAX_SIMS}");
    }
}

#[test]
fn a_town_file_may_hold_its_town_smaller() {
    assert!(!moves_in_most(40, 40).contains(&true), "full at its 40");
    assert_eq!(moves_in_most(39, 40), [true, true], "room for one more");
    // (More than the sim holds counts as the sim's limit; a file that says nothing grows to it.)
    assert_eq!(moves_in_most(MAX_SIMS - 1, MAX_SIMS + 100), [true, true]);
    assert!(!moves_in_most(MAX_SIMS, MAX_SIMS + 100).contains(&true));
    let w = World::from_json(CONTENT, &town(None, 3), 1).unwrap();
    assert_eq!(w.residents_most(), MAX_SIMS);
    assert!(
        !w.save_json().contains("residentsMost"),
        "nothing saved when the file said nothing"
    );
    let w = World::from_json(CONTENT, &town_most(None, 3, Some(40)), 1).unwrap();
    assert_eq!(w.residents_most(), 40);
    assert!(w.save_json().contains(r#""residentsMost":40"#));
}
