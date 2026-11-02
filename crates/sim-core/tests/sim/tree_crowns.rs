//! A tree's crown (content `crown`, m from its middle) keeps clear of the house: buying or moving
//! one where a wall stands closer is refused (`World::place_at_home`, `crown_meets_wall`), further
//! off it goes in, and a garden fence close by doesn't matter.

use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind};
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"comfort","label":"Comfort","decayPerHour":0.01}],
    "objects":[
      {"id":"pine","name":"Pine tree","price":120,"outdoors":true,"crown":3.0,"interactions":[]},
      {"id":"bush","name":"Bush","price":20,"outdoors":true,"interactions":[]}],
    "economy":{"startingFunds":20000}}"#;

/// A house 8 × 6 m (walls from x 4 to 12, z 4 to 10) on a 24 × 24 lot.
const TOWN: &str = r#"{"width":24,"depth":24,
    "plots":[{"name":"Home","x":0,"z":0,"w":24,"d":24,"entry":[12.5,23.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[4,4,12,4],[12,4,12,10],[12,10,4,10],[4,10,4,4]],
    "doors":[{"x":8,"z":10,"axis":"x"}],
    "sims":[{"name":"Ada","x":8.5,"z":20.5}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w
}

fn buy(w: &mut World, def: &str, x: i32, z: i32) -> Result<(), String> {
    w.apply(Command::Buy {
        household: 0,
        object: def.into(),
        at: Some([x, z, 0]),
        colors: Default::default(),
        turn: None,
        offset: None,
        size: None,
        tall: None,
    })
    .map_err(|e| e.to_string())
}

#[test]
fn a_tree_is_not_planted_where_its_crown_meets_the_house() {
    let mut w = world();
    // 1.5 m from the east wall (x = 12): the crown would be in the house.
    let e = buy(&mut w, "pine", 13, 6).expect_err("too near the wall");
    assert!(e.contains("crown"), "{e}");
    // Beyond a corner, but less than 3 m from it (12, 10): (14.5, 11.5) is 2.9 m away.
    assert!(buy(&mut w, "pine", 14, 11).is_err(), "too near the corner");
    // 3.5 m off: fine.
    buy(&mut w, "pine", 15, 6).expect("clear of the house");
    // A bush has no crown: right by the wall is fine.
    buy(&mut w, "bush", 12, 6).expect("a bush by the wall");
}

#[test]
fn a_fence_close_by_is_no_wall() {
    let mut w = world();
    let fence: Vec<EdgeEdit> = (14..20)
        .map(|z| EdgeEdit::new(EdgeAxis::V, 19, z, EdgeKind::Fence))
        .collect();
    w.build(0, &fence).unwrap();
    buy(&mut w, "pine", 17, 16).expect("by a fence");
}
