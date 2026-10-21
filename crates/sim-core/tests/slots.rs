//! A place and a facing per slot (content `places`): two at a picnic table sit across it from
//! each other, coming round to the far bench; ping-pong players stand at the two ends; and a use
//! with `seatFacing` takes only a seat that looks at the thing (no sofa with its back to the TV,
//! a chair round the fire pit), else is used standing (at a place round the pit).

use sim_core::content::Pose;
use sim_core::{Command, TICKS_PER_MINUTE, World};
use std::f32::consts::{FRAC_PI_2, PI};

const CONTENT: &str = r#"{
    "needs":[{"id":"fun","label":"Fun","decayPerHour":0.01}],
    "objects":[
      {"id":"picnic","name":"Picnic table","footprint":[3,2],"slots":2,
       "places":[{"at":[-0.3,0.7],"facing":180},{"at":[-0.3,-0.7]}],
       "interactions":[{"id":"picnic","label":"Have a picnic","minutes":60,"pose":"sit","effects":{"fun":0.3},"tags":["lounge"]}]},
      {"id":"pingPong","name":"Ping-pong table","footprint":[2,1],"slots":2,
       "places":[{"at":[-1.3,0],"facing":90},{"at":[1.3,0],"facing":-90}],
       "interactions":[{"id":"play","label":"Play ping-pong","minutes":60,"effects":{"fun":0.3},"tags":["lounge"]}]},
      {"id":"bench","name":"Bench","footprint":[2,1],"slots":2,
       "interactions":[{"id":"sit","label":"Sit","minutes":60,"pose":"sit","effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"tv","name":"TV","interactions":[
        {"id":"watch","label":"Watch TV","minutes":60,"seatNear":5,"seatFacing":"front","effects":{"fun":0.6},"tags":["screen"]}]},
      {"id":"sofa","name":"Sofa","footprint":[2,1],"slots":2,"interactions":[
        {"id":"relax","label":"Relax","minutes":30,"pose":"sit","effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"chair","name":"Chair","interactions":[
        {"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"firePit","name":"Fire pit","slots":4,
       "places":[{"at":[0,0.85],"facing":180},{"at":[0.85,0],"facing":-90},{"at":[0,-0.85]},{"at":[-0.85,0],"facing":90}],
       "interactions":[{"id":"relax","label":"Warm up by the fire","minutes":30,"seatNear":3,"seatFacing":"around",
        "effects":{"fun":0.4},"tags":["cozy"]}]}]}"#;

const PICNIC: usize = 0;
const BENCH: usize = 2;

fn lot(objects: &str, sims: &str) -> String {
    format!(r#"{{"width":16,"depth":16,"objects":[{objects}],"sims":[{sims}]}}"#)
}

fn world(objects: &str, sims: &str) -> World {
    World::from_json(CONTENT, &lot(objects, sims), 3).unwrap()
}

/// Ticks `w` for up to `minutes`, until `done`; true if it got there.
fn until(w: &mut World, minutes: u64, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..minutes * TICKS_PER_MINUTE {
        if done(w) {
            return true;
        }
        w.tick_once();
    }
    done(w)
}

fn near(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).hypot(a[1] - b[1]) < 0.01
}

/// The way `yaw` faces, as a unit vector (`Sim::yaw`: 0 faces +z).
fn ahead(yaw: f32) -> [f32; 2] {
    [yaw.sin(), yaw.cos()]
}

/// How far (radians) `a` and `b` are apart as facings.
fn turn(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(2.0 * PI);
    d.min(2.0 * PI - d)
}

#[test]
fn places_parse_and_say_where_each_slot_is() {
    let w = world(r#"{"def":"picnic","x":4,"z":4,"rot":0},{"def":"picnic","x":4,"z":9,"rot":1},{"def":"bench","x":10,"z":4}"#, "");
    let def = &w.content.objects[PICNIC];
    assert_eq!(def.places.len(), 2);
    assert!((def.places[0].facing - PI).abs() < 1e-5, "degrees read as radians");
    assert!(w.content.objects[BENCH].places.is_empty(), "none unless given");
    // Facing +z: the first on the front bench facing back, the second on the far one facing out.
    let o = &w.objects[0];
    assert!(near(o.slot_position(&w.content, 0), [5.2, 5.7]));
    assert!(near(o.slot_position(&w.content, 1), [5.2, 4.3]));
    assert!(turn(o.slot_yaw(&w.content, 0), PI) < 1e-4);
    assert!(turn(o.slot_yaw(&w.content, 1), 0.0) < 1e-4);
    // Turned a quarter (facing +x, 2 wide and 3 deep): the places turn with it.
    let o = &w.objects[1];
    assert!(near(o.slot_position(&w.content, 0), [5.7, 10.8]), "{:?}", o.slot_position(&w.content, 0));
    assert!(turn(o.slot_yaw(&w.content, 0), FRAC_PI_2 + PI) < 1e-4);
    // Without places: spread across the width, facing the front, as before.
    let b = &w.objects[2];
    assert!(near(b.slot_position(&w.content, 0), [10.5, 4.5]));
    assert!(near(b.slot_position(&w.content, 1), [11.5, 4.5]));
    assert_eq!(b.slot_yaw(&w.content, 1), 0.0);
}

#[test]
fn places_must_fit_the_slots() {
    let bad = CONTENT.replace(r#""slots":2,
       "places":[{"at":[-0.3,0.7],"facing":180},{"at":[-0.3,-0.7]}]"#, r#""slots":3,
       "places":[{"at":[-0.3,0.7],"facing":180},{"at":[-0.3,-0.7]}]"#);
    assert_ne!(bad, CONTENT);
    let Err(err) = World::from_json(&bad, &lot("", ""), 1) else { panic!("refused") };
    assert!(err.to_string().contains("one place a slot"), "{err}");
    let far = CONTENT.replace(r#"{"at":[1.3,0],"facing":-90}"#, r#"{"at":[2.6,0],"facing":-90}"#);
    let Err(err) = World::from_json(&far, &lot("", ""), 1) else { panic!("refused") };
    assert!(err.to_string().contains("off its footprint"), "{err}");
    let lone = CONTENT.replace(r#""seatNear":5,"seatFacing":"front""#, r#""seatFacing":"front""#);
    let Err(err) = World::from_json(&lone, &lot("", ""), 1) else { panic!("refused") };
    assert!(err.to_string().contains("seatFacing needs a seatNear"), "{err}");
}

#[test]
fn two_at_a_picnic_table_sit_across_it_facing_each_other() {
    // The table on x 4..7, z 4..6, facing +z; Ada comes from the front, Bea from behind it.
    let mut w = world(r#"{"def":"picnic","x":4,"z":4,"rot":0}"#, r#"{"name":"Ada","x":5.5,"z":9.5},{"name":"Bea","x":5.5,"z":1.5}"#);
    for sim in 0..2 {
        w.apply(Command::Use { sim, object: 0, interaction: 0 }).unwrap();
    }
    // (Where Bea stood last before she sat down.)
    let mut from = (0, 0);
    let both = until(&mut w, 10, |w| {
        if w.sims[1].pose == Pose::Stand {
            from = w.sims[1].tile();
        }
        w.sims.iter().all(|s| s.pose == Pose::Sit)
    });
    assert!(both, "both sit down");
    let (a, b) = (&w.sims[0], &w.sims[1]);
    // Each on their own bench, the one nearer them: across the table, facing each other.
    assert!(near(a.pos, [5.2, 5.7]), "Ada on the front bench: {:?}", a.pos);
    assert!(near(b.pos, [5.2, 4.3]), "Bea on the far bench: {:?}", b.pos);
    assert!(turn(a.yaw, b.yaw + PI) < 1e-4, "facing each other: {} {}", a.yaw, b.yaw);
    let to_b = [b.pos[0] - a.pos[0], b.pos[1] - a.pos[1]];
    assert!(ahead(a.yaw)[0] * to_b[0] + ahead(a.yaw)[1] * to_b[1] > 0.0, "Ada looks at Bea");
    // Bea came round to her bench from behind it, not through the table.
    assert_eq!(from, (5, 3));
}

#[test]
fn ping_pong_players_stand_at_the_two_ends_facing_each_other() {
    let mut w = world(r#"{"def":"pingPong","x":6,"z":6,"rot":0}"#, r#"{"name":"Ada","x":2.5,"z":7.5},{"name":"Bea","x":11.5,"z":7.5}"#);
    for sim in 0..2 {
        w.apply(Command::Use { sim, object: 0, interaction: 0 }).unwrap();
    }
    let playing = |w: &World| w.sims.iter().all(|s| s.current().is_some_and(|a| matches!(a.phase, sim_core::world::Phase::Using { .. })));
    assert!(until(&mut w, 10, playing), "both play");
    let (a, b) = (&w.sims[0], &w.sims[1]);
    assert_eq!(a.pose, Pose::Stand);
    assert!(near(a.pos, [5.7, 6.5]), "Ada at the left end: {:?}", a.pos);
    assert!(near(b.pos, [8.3, 6.5]), "Bea at the right end: {:?}", b.pos);
    assert!(turn(a.yaw, FRAC_PI_2) < 1e-4 && turn(b.yaw, -FRAC_PI_2) < 1e-4, "facing each other across the net");
}

#[test]
fn a_blocked_end_is_played_from_the_front() {
    // A bench takes the tiles past the table's left end (x 4..6): Ada, the only one, gets the
    // slot nearest her there, can't stand on it, and plays from the front tile, facing the table.
    let mut w = world(r#"{"def":"pingPong","x":6,"z":6,"rot":0},{"def":"bench","x":4,"z":6,"rot":0}"#, r#"{"name":"Ada","x":2.5,"z":7.5}"#);
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    assert!(until(&mut w, 10, |w| w.sims[0].current().is_some_and(|a| matches!(a.phase, sim_core::world::Phase::Using { .. }))));
    let a = &w.sims[0];
    assert_eq!(a.tile(), (6, 7), "on the front tile: {:?}", a.pos);
    assert!(ahead(a.yaw)[1] < -0.5, "facing the table: {}", a.yaw);
}

#[test]
fn watching_tv_takes_the_seat_that_faces_the_screen() {
    // The TV at (7, 7) faces +z. Nearest, a sofa in front of it facing away (its back to the
    // screen); further off, one facing it.
    let objects = r#"{"def":"tv","x":7,"z":7,"rot":0},{"def":"sofa","x":7,"z":9,"rot":0},{"def":"sofa","x":7,"z":12,"rot":2}"#;
    let mut w = world(objects, r#"{"name":"Ada","x":3.5,"z":9.5}"#);
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    assert!(until(&mut w, 10, |w| w.sims[0].pose == Pose::Sit), "sits down");
    assert_eq!(w.sims[0].use_seat, Some(2), "the sofa that faces the screen");
}

#[test]
fn with_no_seat_facing_it_the_tv_is_watched_standing() {
    // Only the sofa with its back to the screen, and one behind the TV looking at its back.
    let objects = r#"{"def":"tv","x":7,"z":7,"rot":0},{"def":"sofa","x":7,"z":9,"rot":0},{"def":"sofa","x":7,"z":4,"rot":0}"#;
    let mut w = world(objects, r#"{"name":"Ada","x":8.5,"z":12.5}"#);
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    assert!(until(&mut w, 10, |w| w.sims[0].current().is_some_and(|a| matches!(a.phase, sim_core::world::Phase::Using { .. }))));
    assert_eq!(w.sims[0].use_seat, None);
    assert_eq!(w.sims[0].pose, Pose::Stand);
}

#[test]
fn round_the_fire_pit_from_a_chair_facing_it_or_standing_facing_in() {
    // The pit at (7, 7); a chair three tiles behind it facing it (rot 0, towards +z), and a
    // nearer one off its front corner facing away (rot 1, towards +x).
    let objects = r#"{"def":"firePit","x":7,"z":7},{"def":"chair","x":7,"z":4,"rot":0},{"def":"chair","x":8,"z":8,"rot":1}"#;
    let mut w = world(objects, r#"{"name":"Ada","x":10.5,"z":5.5},{"name":"Bea","x":7.5,"z":12.5}"#);
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    assert!(until(&mut w, 10, |w| w.sims[0].pose == Pose::Sit), "Ada sits");
    assert_eq!(w.sims[0].use_seat, Some(1), "on the chair that faces the fire, not the one turned away");
    // No other chair faces it: Bea stands at the place round it nearest her, facing the fire.
    w.apply(Command::Use { sim: 1, object: 0, interaction: 0 }).unwrap();
    assert!(until(&mut w, 10, |w| w.sims[1].current().is_some_and(|a| matches!(a.phase, sim_core::world::Phase::Using { .. }))));
    let b = &w.sims[1];
    assert_eq!((b.use_seat, b.pose), (None, Pose::Stand));
    assert!(near(b.pos, [7.5, 8.35]), "in front of the pit, off its rim: {:?}", b.pos);
    assert!(turn(b.yaw, PI) < 1e-4, "facing the fire");
}
