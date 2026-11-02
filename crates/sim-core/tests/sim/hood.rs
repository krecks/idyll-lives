//! A light that follows a use (`light.while`): the cooker hood's light comes on while the hob
//! under it is at work, by day as at night, and goes off when the cooking is done. Cooking is a
//! recipe (`TaskKind::Cook`), not a plain use: the light watches the step's animation at the
//! station, and a hood over a hob lights for the stir and the bake alike.

use crate::common;

use sim_core::{Command, TICKS_PER_MINUTE, World, lights};

/// A kitchen with a wall to hang the hood on, a cooker under it and a fridge to fetch from.
const TOWN: &str = r#"{"width":9,"depth":9,
  "plots":[{"name":"Home","x":0,"z":0,"w":9,"d":9,"entry":[4.5,8.5]}],
  "households":[{"name":"Player","plot":0,"player":true}],
  "walls":[[1,1,8,1],[1,1,1,8],[1,8,8,8],[8,1,8,8],[1,3,8,3]],
  "doors":[{"x":4,"z":8,"axis":"x"}],
  "objects":[{"def":"stoveElectric","x":3,"z":3},{"def":"rangeHood","x":3,"z":3},
             {"def":"fridge","x":4,"z":3}],
  "sims":[{"name":"Ada","x":5,"z":5,"traits":["cheerful"]}]}"#;

#[test]
fn the_hood_lights_while_the_meal_cooks() {
    let mut w = World::from_json(&common::content(), TOWN, 7).unwrap();
    w.autonomy = false;
    // Dusk: the room's own lamps are on, so the hood's light is the only thing that changes.
    w.tick = TICKS_PER_MINUTE * 60 * 14;
    let stove = w
        .objects
        .iter()
        .position(|o| w.content.objects[o.def].id == "stoveElectric")
        .unwrap() as u32;
    let hood = w
        .objects
        .iter()
        .position(|o| w.content.objects[o.def].id == "rangeHood")
        .unwrap();
    let stable = w.objects[hood].stable;
    assert!(
        lights::lamps_off(&w).contains(&stable),
        "a hood with nothing under it is off"
    );
    w.apply(Command::Use {
        sim: 0,
        object: stove,
        interaction: 0,
    })
    .unwrap();
    let mut lit = 0;
    for _ in 0..TICKS_PER_MINUTE * 40 {
        w.tick_once();
        if !lights::lamps_off(&w).contains(&stable) {
            lit += 1;
        }
    }
    assert!(
        lit > TICKS_PER_MINUTE * 10,
        "the light came on: {lit} ticks"
    );
    assert!(
        lights::lamps_off(&w).contains(&stable),
        "and went off when the cooking was done"
    );
}

/// The lights that follow a use are found once for the objects as they stand (`lights.rs`): a
/// hood sold and hung again while the town plays lights for the cooking under it all the same.
#[test]
fn a_hood_hung_again_lights_too() {
    let mut w = World::from_json(&common::content(), TOWN, 7).unwrap();
    w.autonomy = false;
    w.tick = TICKS_PER_MINUTE * 60 * 14;
    let find = |w: &World, id: &str| {
        w.objects
            .iter()
            .position(|o| w.content.objects[o.def].id == id)
    };
    w.tick_once();
    let hood = w.objects[find(&w, "rangeHood").unwrap()].stable;
    w.apply(Command::Sell {
        household: 0,
        object: hood,
    })
    .unwrap();
    w.tick_once();
    let hood = common::buy(&mut w, "rangeHood", 3, 3);
    let stove = find(&w, "stoveElectric").unwrap() as u32;
    w.apply(Command::Use {
        sim: 0,
        object: stove,
        interaction: 0,
    })
    .unwrap();
    let mut lit = 0;
    for _ in 0..TICKS_PER_MINUTE * 40 {
        w.tick_once();
        if !lights::lamps_off(&w).contains(&hood) {
            lit += 1;
        }
    }
    assert!(
        lit > TICKS_PER_MINUTE * 10,
        "the light came on: {lit} ticks"
    );
}
