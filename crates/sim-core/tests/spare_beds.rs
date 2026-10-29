//! A spare bed's night (content `spare`): the sofa bed pulled out for a night's sleep is a bed for
//! someone without one, and doesn't make the living room a bedroom (G-61, I-living-04).

mod common;

use sim_core::world::TaskKind;
use sim_core::{World, clock};

/// A living room (a sofa bed and a sofa) and a bedroom with one single bed, for two.
const TOWN: &str = r#"{"width":18,"depth":14,
    "plots":[{"name":"Home","x":0,"z":0,"w":18,"d":14,"entry":[8.5,13.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[1,1,10,1],[1,7,10,7],[1,1,1,7],[10,1,10,7],
             [10,1,15,1],[10,7,15,7],[15,1,15,7]],
    "doors":[{"x":4,"z":7,"axis":"x"},{"x":10,"z":4,"axis":"z"}],
    "objects":[{"def":"sofaBed","x":2,"z":2},{"def":"sofa","x":6,"z":2},{"def":"bedSingle","x":12,"z":2}],
    "sims":[{"name":"Ada","traits":["bookworm"],"x":4.5,"z":5.5},{"name":"Bea","traits":["bookworm"],"x":7.5,"z":5.5}]}"#;

const SOFA_BED: usize = 0;
const BED: usize = 2;

fn kind(w: &World, x: i32, z: i32) -> String {
    let r = w.room_at_tile(x, z).unwrap();
    r.kind.map_or("none".into(), |k| w.content.room_kinds[k].id.clone())
}

/// What `sim` is doing: the object and interaction id, while using one.
fn doing(w: &World, sim: usize) -> Option<(usize, String)> {
    match w.sims[sim].current()?.task.kind {
        TaskKind::Use { object, interaction } => {
            let def = &w.content.objects[w.objects[object as usize].def];
            Some((object as usize, def.interactions[interaction].id.clone()))
        }
        _ => None,
    }
}

/// The town at 10 pm on day 1, both tired, choosing for themselves.
fn bedtime(town: &str) -> World {
    let mut w = World::from_json(&common::content(), town, 1).unwrap();
    let evening = clock::tick_at(1, 21.5 * 60.0).unwrap();
    while w.tick < evening {
        w.tick_once();
    }
    let n = w.content.needs.len();
    for s in &mut w.sims {
        s.needs[..n].fill(1.0);
        let energy = w.content.needs.iter().position(|n| n.id == "energy").unwrap();
        s.needs[energy] = 0.25;
    }
    w
}

#[test]
fn the_sofa_bed_keeps_the_living_room_a_living_room() {
    let w = World::from_json(&common::content(), TOWN, 1).unwrap();
    assert_eq!(kind(&w, 3, 3), "living", "a sofa bed is a sofa that folds out");
    assert!(!w.room_at_tile(3, 3).unwrap().mixed, "not a bedroom too");
    assert_eq!(kind(&w, 12, 3), "bedroom");
    let def = &w.content.objects[w.objects[SOFA_BED].def];
    let sleep = def.interactions.iter().find(|i| i.id == "sleep").expect("a night on the sofa bed");
    assert!(sleep.spare && sleep.tags & w.content.day_rhythm.sleep_tags != 0);
    assert_eq!(def.tags & w.content.day_rhythm.sleep_tags, 0, "its night doesn't count towards the room");
}

#[test]
fn whoever_has_no_bed_sleeps_the_night_on_the_sofa_bed() {
    let mut w = bedtime(TOWN);
    let night = clock::tick_at(2, 2.0 * 60.0).unwrap();
    let mut slept = [None, None];
    while w.tick < night {
        w.tick_once();
        for (s, place) in slept.iter_mut().enumerate() {
            if let Some((o, id)) = doing(&w, s).filter(|(_, id)| id == "sleep")
                && w.sims[s].asleep(&w.content)
            {
                place.get_or_insert((o, id));
            }
        }
    }
    let places: Vec<usize> = slept.iter().map(|p| p.as_ref().expect("both asleep by 2 am").0).collect();
    assert!(places.contains(&BED) && places.contains(&SOFA_BED), "one in the bed, the other on the sofa bed: {places:?}");
    assert!(w.objects[SOFA_BED].open, "pulled out for the night");
    assert_eq!(kind(&w, 3, 3), "living", "still the living room, slept in");
}

#[test]
fn nobody_takes_the_sofa_bed_with_a_bed_free() {
    // Two single beds in the bedroom, for two sisters (only those who belong together share a
    // bedroom): the sofa bed stays a sofa.
    let town = TOWN
        .replace(r#"{"def":"bedSingle","x":12,"z":2}"#, r#"{"def":"bedSingle","x":12,"z":2},{"def":"bedSingle","x":14,"z":2}"#)
        .replace(r#""x":7.5,"z":5.5}]}"#, r#""x":7.5,"z":5.5}],"relationships":[{"a":0,"b":1,"preset":"siblings"}]}"#);
    let mut w = bedtime(&town);
    let night = clock::tick_at(2, 2.0 * 60.0).unwrap();
    while w.tick < night {
        w.tick_once();
        for s in 0..2 {
            assert!(doing(&w, s).is_none_or(|(o, id)| o != SOFA_BED || id != "sleep"), "{} on the sofa bed with a bed free", w.sims[s].name);
        }
    }
    assert!((0..2).all(|s| doing(&w, s).is_some_and(|(o, id)| o != SOFA_BED && id == "sleep")), "both in the beds");
}

#[test]
fn spare_is_for_a_night_s_sleep() {
    let content = r#"{"needs":[{"id":"energy","label":"Energy","decayPerHour":0.01}],
        "dayRhythm":{"sleepTags":["sleep"]},
        "objects":[{"id":"bed","name":"Bed","price":10,"interactions":[
            {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","tags":["sleep"]}]},
          {"id":"sofa","name":"Sofa","price":10,"interactions":[
            {"id":"sit","label":"Sit","minutes":30,"pose":"sit","tags":["lounge"],"spare":true}]}]}"#;
    let town = r#"{"width":4,"depth":4,"plots":[],"households":[],"sims":[]}"#;
    let err = World::from_json(content, town, 1).err().expect("refused").to_string();
    assert!(err.contains("sofa.sit") && err.contains("spare"), "{err}");
}
