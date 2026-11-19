//! A cabin got into through its door (content `"approach": "door"`, the sauna): its places at the
//! back are come to from the tile in front at every turn, never from the tile behind through its
//! back wall, as an open bench's far side is (a picnic table's).

use sim_core::content::Pose;
use sim_core::{Command, TICKS_PER_MINUTE, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"comfort","label":"Comfort","decayPerHour":0.01}],
    "objects":[
      {"id":"cabin","name":"Cabin","footprint":[2,2],"slots":3,"approach":"door",
       "places":[{"at":[-0.55,-0.35]},{"at":[0,-0.35]},{"at":[0.55,-0.35]}],
       "interactions":[{"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"comfort":0.5}}]},
      {"id":"bench","name":"Bench","footprint":[2,2],"slots":3,
       "places":[{"at":[-0.55,-0.35]},{"at":[0,-0.35]},{"at":[0.55,-0.35]}],
       "interactions":[{"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"comfort":0.5}}]}],
    "economy":{"startingFunds":1000}}"#;

/// The tile `def` turned `rot` at (6, 6) is sat in from: the last tile stood on before sitting.
fn sat_in_from(def: &str, rot: u8) -> ((i32, i32), (i32, i32)) {
    let town = format!(
        r#"{{"width":14,"depth":14,
        "plots":[{{"name":"Home","x":0,"z":0,"w":14,"d":14,"entry":[7.5,13.5]}}],
        "households":[{{"name":"Player","plot":0,"player":true}}],
        "objects":[{{"def":"{def}","x":6,"z":6,"rot":{rot}}}],
        "sims":[{{"name":"Ada","x":1.5,"z":1.5,"age":35}}]}}"#
    );
    let mut w = World::from_json(CONTENT, &town, 1).unwrap();
    w.autonomy = false;
    w.apply(Command::Use {
        sim: 0,
        object: 0,
        interaction: 0,
    })
    .unwrap();
    let mut stood = (0, 0);
    for _ in 0..30 * TICKS_PER_MINUTE {
        let s = &w.sims[0];
        if s.pose == Pose::Sit {
            break;
        }
        stood = (s.pos[0].floor() as i32, s.pos[1].floor() as i32);
        w.tick_once();
    }
    assert_eq!(w.sims[0].pose, Pose::Sit, "{def} rot {rot}: sat down");
    (stood, w.objects[0].front_tile(&w.content))
}

#[test]
fn a_cabins_bench_is_got_to_through_its_door_at_every_turn() {
    for rot in 0..4 {
        let (stood, front) = sat_in_from("cabin", rot);
        assert_eq!(stood, front, "rot {rot}: from its front tile");
    }
}

#[test]
fn an_open_benchs_far_side_is_still_got_to_from_behind() {
    for rot in 0..4 {
        let (stood, front) = sat_in_from("bench", rot);
        assert_ne!(
            stood, front,
            "rot {rot}: round the back, not through the bench"
        );
    }
}
