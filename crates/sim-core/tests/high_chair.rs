//! A baby at the high chair and the changing table (G-66, I-kids-01, I-kids-05), with the shipped
//! content: by day a grown-up feeding the baby lifts them out of the crib, carries them in the arms
//! to the high chair, puts them in it and feeds them from beside it, then carries them back to the
//! crib; at night the feed stays at the crib, in the arms. The baby is changed at the changing
//! table beside the cot, the carer standing at it. Carried, the baby goes where the carer goes;
//! put down anywhere else (the carer called away), or left in the high chair, they're back in the
//! crib.

mod common;

use sim_core::content::Pose;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, TICKS_PER_MINUTE, World, clock};

/// A crib, a dining table with the high chair pulled up to it, and a parent with her baby.
const HOME: &str = r#"{"width":16,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":10,"entry":[8.5,9.5]}],
    "households":[{"name":"Home","plot":0,"player":true}],
    "objects":[{"def":"crib","x":1,"z":5,"rot":0},{"def":"tableDining4","x":8,"z":4,"rot":0},
               {"def":"highChair","x":8,"z":5,"rot":2},{"def":"fridge","x":13,"z":1,"rot":0}],
    "sims":[{"name":"Ada","traits":["kind"],"x":5.5,"z":7.5,"age":30},{"name":"Bea","traits":["cheerful"],"x":5.5,"z":6.5,"age":0.5}],
    "relationships":[{"a":1,"b":0,"preset":"parent"}]}"#;

const CRIB: u32 = 0;
const HIGH_CHAIR: u32 = 2;

fn interaction(w: &World, object: u32, id: &str) -> usize {
    let def = w.objects[object as usize].def;
    w.content.objects[def].interactions.iter().position(|i| i.id == id).unwrap_or_else(|| panic!("no {id}"))
}

/// The shipped content and `town` at `hour` on day 1, everyone's needs full, nobody choosing for
/// themselves; Bea in her crib.
fn home(town: &str, hour: f32) -> World {
    let mut w = World::from_json(&common::content(), town, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    let at = clock::tick_at(1, hour * 60.0).unwrap();
    while w.tick < at {
        w.tick_once();
    }
    let n = w.content.needs.len();
    for s in &mut w.sims {
        s.needs[..n].fill(1.0);
    }
    w.tick_once();
    assert!(lying_in_crib(&w), "Bea is in her crib");
    w
}

fn lying_in_crib(w: &World) -> bool {
    w.sims[1].pose == Pose::Lie && w.objects[CRIB as usize].users[0] == Some(1)
}

fn using(w: &World, sim: usize, object: u32, id: &str) -> bool {
    w.sims[sim].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Use { object: o, interaction } if o == object && interaction == self::interaction(w, object, id)))
        && w.sims[sim].current().is_some_and(|a| matches!(a.phase, Phase::Using { .. }))
}

/// Ticks `w` for up to `minutes`, calling `each` every tick, until `done`; true if it got there.
fn until(w: &mut World, minutes: u64, mut each: impl FnMut(&World), mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..minutes * TICKS_PER_MINUTE {
        if done(w) {
            return true;
        }
        w.tick_once();
        each(w);
    }
    done(w)
}

#[test]
fn by_day_the_baby_is_carried_to_the_high_chair_fed_there_and_carried_back() {
    let mut w = home(HOME, 10.0);
    let hunger = w.content.needs.iter().position(|n| n.id == "hunger").unwrap();
    w.sims[1].needs[hunger] = 0.2;
    let feed = interaction(&w, CRIB, "feed");
    w.apply(Command::Use { sim: 0, object: CRIB, interaction: feed }).unwrap();

    // Over to the crib, Bea lifted out and carried in Ada's arms, going where she goes.
    let (mut lifted, mut carried_with) = (false, 0);
    let mut held_in_snapshot = false;
    let seated = until(
        &mut w,
        30,
        |w| {
            if w.sims[1].carried_by == Some(0) {
                lifted = true;
                assert_eq!(w.sims[1].pos, w.sims[0].pos, "carried, Bea is where Ada is");
                assert!(w.objects.iter().all(|o| o.users().all(|u| u != 1)), "and in no place");
                carried_with += 1;
                let mut out = vec![0.0; sim_core::snapshot::CAPACITY];
                sim_core::snapshot::write(w, &mut out);
                held_in_snapshot |= out[sim_core::snapshot::HEADER_LEN + sim_core::snapshot::sim::CARRY] == 2.0;
            }
        },
        |w| using(w, 1, HIGH_CHAIR, "sit"),
    );
    assert!(lifted && carried_with > 20, "carried over in the arms ({carried_with} ticks)");
    assert!(held_in_snapshot, "the renderer hears who's in Ada's arms");
    assert!(seated, "Bea sits in the high chair");
    let chair = w.objects[HIGH_CHAIR as usize].clone();
    assert_eq!(w.sims[1].pose, Pose::Sit);
    assert_eq!(chair.users[0], Some(1), "in its seat");
    assert_eq!(w.sims[1].pos, chair.slot_position(&w.content, 0));
    assert!(w.objects[CRIB as usize].users().all(|u| u != 1), "out of the crib");

    // Fed there, from beside it (the table in front of it, its back to the room).
    assert!(until(&mut w, 10, |_| {}, |w| using(w, 0, HIGH_CHAIR, "feed")), "Ada feeds her at the high chair");
    let (tx, tz) = w.sims[0].tile();
    assert_eq!(tz, 5, "beside the high chair (at {tx},{tz}), not behind it");
    assert!(tx == 7 || tx == 9, "beside it: {tx},{tz}");
    let c = chair.centre(&w.content);
    let facing = (c[0] - w.sims[0].pos[0]).atan2(c[1] - w.sims[0].pos[1]);
    assert!((w.sims[0].yaw - facing).abs() < 1e-3, "facing Bea");
    assert!(until(&mut w, 20, |_| {}, |w| !using(w, 0, HIGH_CHAIR, "feed")));
    assert!(w.sims[1].needs[hunger] > 0.9, "Bea's fed: {}", w.sims[1].needs[hunger]);

    // And back to the crib in the arms.
    let mut carried_back = false;
    let back = until(&mut w, 30, |w| carried_back |= w.sims[1].carried_by == Some(0), lying_in_crib);
    assert!(carried_back && back, "carried back and laid in the crib");
    assert!(w.objects[HIGH_CHAIR as usize].users().next().is_none(), "the high chair's free");
    assert!(until(&mut w, 5, |_| {}, |w| w.sims[0].current().is_none()), "Ada's done");
}

#[test]
fn at_night_the_feed_stays_at_the_crib() {
    let mut w = home(HOME, 23.0);
    let hunger = w.content.needs.iter().position(|n| n.id == "hunger").unwrap();
    w.sims[1].needs[hunger] = 0.2;
    let feed = interaction(&w, CRIB, "feed");
    w.apply(Command::Use { sim: 0, object: CRIB, interaction: feed }).unwrap();
    let fed = until(
        &mut w,
        30,
        |w| assert!(w.sims[1].carried_by.is_none() && lying_in_crib(w), "Bea stays in the crib"),
        |w| using(w, 0, CRIB, "feed"),
    );
    assert!(fed, "fed at the crib, in the arms");
}

#[test]
fn with_no_high_chair_the_feed_is_at_the_crib() {
    let town = HOME.replace(r#"{"def":"highChair","x":8,"z":5,"rot":2},"#, "");
    let mut w = home(&town, 10.0);
    let feed = interaction(&w, CRIB, "feed");
    w.apply(Command::Use { sim: 0, object: CRIB, interaction: feed }).unwrap();
    assert!(until(&mut w, 30, |w| assert!(lying_in_crib(w)), |w| using(w, 0, CRIB, "feed")), "at the crib");
}

#[test]
fn the_baby_is_changed_at_the_changing_table() {
    let mut w = home(HOME, 10.0);
    let bladder = w.content.needs.iter().position(|n| n.id == "bladder").unwrap();
    w.sims[1].needs[bladder] = 0.1;
    let change = interaction(&w, CRIB, "change");
    w.apply(Command::Use { sim: 0, object: CRIB, interaction: change }).unwrap();
    assert!(until(&mut w, 30, |w| assert!(lying_in_crib(w)), |w| using(w, 0, CRIB, "change")), "changing her");
    // The cot is the crib's left tile, the changing table its right: Ada stands in front of the table.
    let crib = &w.objects[CRIB as usize];
    assert_eq!(w.sims[0].tile(), (2, 6), "at the changing table");
    let pad = crib.slot_position(&w.content, 1);
    let facing = (pad[0] - w.sims[0].pos[0]).atan2(pad[1] - w.sims[0].pos[1]);
    assert!((w.sims[0].yaw - facing).abs() < 1e-3, "facing the pad");
    assert!(until(&mut w, 20, |_| {}, |w| !using(w, 0, CRIB, "change")));
    assert!(w.sims[1].needs[bladder] > 0.9, "Bea's dry");
}

#[test]
fn a_baby_put_down_anywhere_else_is_back_in_the_crib() {
    let mut w = home(HOME, 10.0);
    let feed = interaction(&w, CRIB, "feed");
    w.apply(Command::Use { sim: 0, object: CRIB, interaction: feed }).unwrap();
    assert!(until(&mut w, 30, |_| {}, |w| w.sims[1].carried_by == Some(0)), "carried");
    // The player cancels it halfway: Bea isn't left in the air.
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    assert!(until(&mut w, 2, |_| {}, lying_in_crib), "back in the crib at once");
    assert!(w.sims[1].carried_by.is_none());
    common::check(&w);
}

#[test]
fn left_to_herself_a_parent_feeds_the_hungry_baby_at_the_high_chair() {
    let mut w = home(HOME, 10.0);
    let hunger = w.content.needs.iter().position(|n| n.id == "hunger").unwrap();
    w.sims[1].needs[hunger] = 0.15;
    w.apply(Command::SetAutonomy { enabled: true, household: None }).unwrap();
    assert!(until(&mut w, 60, |_| {}, |w| using(w, 0, HIGH_CHAIR, "feed")), "fed at the high chair");
    assert!(until(&mut w, 60, |_| {}, lying_in_crib), "and back in the crib after");
    common::check(&w);
}

#[test]
fn a_baby_left_in_the_high_chair_is_back_in_the_crib_when_its_time_is_up() {
    let mut w = home(HOME, 10.0);
    let feed = interaction(&w, CRIB, "feed");
    w.apply(Command::Use { sim: 0, object: CRIB, interaction: feed }).unwrap();
    assert!(until(&mut w, 30, |_| {}, |w| using(w, 1, HIGH_CHAIR, "sit")), "in the high chair");
    // Ada goes off before feeding her, and nobody comes for her.
    w.apply(Command::MoveTo { sim: 0, x: 12.5, z: 8.5 }).unwrap();
    let sit = &w.content.objects[w.objects[HIGH_CHAIR as usize].def].interactions[interaction(&w, HIGH_CHAIR, "sit")];
    let minutes = sit.minutes as u64 + 5;
    assert!(until(&mut w, minutes, |_| {}, lying_in_crib), "back in the crib");
}

#[test]
fn a_baby_in_the_high_chair_saved_and_loaded_is_back_in_the_crib_or_the_chair() {
    let mut w = home(HOME, 10.0);
    let feed = interaction(&w, CRIB, "feed");
    w.apply(Command::Use { sim: 0, object: CRIB, interaction: feed }).unwrap();
    for done in [|w: &World| w.sims[1].carried_by.is_some(), |w: &World| using(w, 1, HIGH_CHAIR, "sit")] {
        let mut w = World::from_save_json(&common::content(), &w.save_json()).unwrap();
        let _ = until(&mut w, 30, |_| {}, done);
        let mut loaded = World::from_save_json(&common::content(), &w.save_json()).unwrap();
        loaded.tick_once();
        loaded.tick_once();
        let s = &loaded.sims[1];
        assert!(s.carried_by.is_none(), "nobody carries her after a load");
        assert!(lying_in_crib(&loaded) || using(&loaded, 1, HIGH_CHAIR, "sit"), "in the crib or the high chair: {:?}", s.current().map(|a| a.task.kind));
        common::check(&loaded);
    }
}
