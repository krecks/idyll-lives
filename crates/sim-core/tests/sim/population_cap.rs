//! How many residents a town grows to (`World::population_cap`): births and newcomers stop at
//! `MAX_SIMS` (160) in every town now (cosy and normal were dropped in 0.7.1), which every town
//! can hold.

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
    let mut sims = vec![r#"{"name":"Ada","x":4.5,"z":5.5,"age":30}"#.to_string()];
    sims.extend((1..people).map(|i| {
        format!(
            r#"{{"name":"Old {i}","household":1,"x":{}.5,"z":5.5,"age":80}}"#,
            10 + i % 10
        )
    }));
    let size = size.map_or(String::new(), |s| format!(r#""size":"{s}","#));
    format!(
        r#"{{"width":30,"depth":10,{size}
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

#[test]
fn every_town_grows_to_160() {
    for size in [None, Some("cosy"), Some("normal"), Some("big")] {
        let w = World::from_json(CONTENT, &town(size, 2), 1).unwrap();
        assert_eq!(w.population_cap(), MAX_SIMS, "{size:?}");
        assert!(moves_in(size, MAX_SIMS - 1), "{size:?}: room for one more");
        assert!(!moves_in(size, MAX_SIMS), "{size:?}: full at {MAX_SIMS}");
    }
}
