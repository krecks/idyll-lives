//! Things standing on tops have uses (G-63): a kettle on the counter is used standing in front
//! of the counter, a laptop on a desk sitting on the chair pulled up to it (standing beside the
//! desk without one). Uses have conditions (G-64): an hour of the day (stargazing after dark),
//! open sky (outdoors), something to work on (washing up with dishes to wash, the laundry with
//! washing to do), kept by free will and the player's orders alike. Coffee is for teens and up.

mod common;

use sim_core::content::{Content, Pose};
use sim_core::items::{ItemAt, ItemInstance};
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World, clock};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02},
             {"id":"fun","label":"Fun","decayPerHour":0.02},
             {"id":"comfort","label":"Comfort","decayPerHour":0.02}],
    "objects":[
      {"id":"counter","name":"Counter","price":150,"surface":{"height":0.9,"kind":"counter","size":[1,0.6],"at":[0,-0.2]}},
      {"id":"desk","name":"Desk","price":150,"surface":{"height":0.75,"kind":"desk"}},
      {"id":"chair","name":"Desk chair","price":60,"seatAt":["desk"],"approach":"around",
       "interactions":[{"id":"sit","label":"Sit down","minutes":20,"pose":"sit","effects":{"comfort":0.2},"tags":["lounge"],"autonomous":false}]},
      {"id":"kettle","name":"Kettle","price":30,"layer":"surface","radius":0.13,
       "interactions":[{"id":"tea","label":"Make a cup of tea","minutes":8,"effects":{"comfort":0.3},"tags":["food"]}]},
      {"id":"laptop","name":"Laptop","price":600,"layer":"surface","radius":0.19,
       "interactions":[{"id":"browse","label":"Browse the web","minutes":45,"pose":"sit","effects":{"fun":0.4},"tags":["screen"]}]},
      {"id":"telescope","name":"Telescope","price":900,
       "interactions":[{"id":"stargaze","label":"Stargaze","minutes":45,"hours":[20,5],"sky":true,"effects":{"fun":0.4},"tags":["stargazing"]}]},
      {"id":"sink","name":"Sink","price":200,
       "interactions":[{"id":"washUp","label":"Wash the dishes","minutes":15,"workOn":"dishes","autonomous":false,"effects":{"comfort":0.06},"tags":["chores"]}]},
      {"id":"washer","name":"Washing machine","price":500,
       "interactions":[{"id":"laundry","label":"Do the laundry","minutes":30,"workOn":"laundry","makes":"washed","effects":{"comfort":0.1},"tags":["chores"]}]},
      {"id":"dryer","name":"Tumble dryer","price":400,
       "interactions":[{"id":"laundry","label":"Fold the laundry","minutes":20,"workOn":"washed","effects":{"comfort":0.1},"tags":["chores"]}]}],
    "items":[{"id":"plateDirty","label":"Dirty plate","dirty":true}],
    "laundry":{"perDay":0.5,"most":3},
    "economy":{"startingFunds":5000}}"#;

/// One room (x 1..9, z 1..6) with a door at the bottom, on a 16 × 12 plot: a counter with a
/// kettle on it, a desk with a laptop on it, a telescope indoors and one outside, a sink, a
/// washer and a dryer; Ada in the room.
const TOWN: &str = r#"{"width":16,"depth":12,
    "walls":[[1,1,9,1],[1,6,9,6],[1,1,1,6],[9,1,9,6]],
    "doors":[{"x":4,"z":6,"axis":"x"}],
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":12,"entry":[12.5,10.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"counter","x":2,"z":1,"rot":0},{"def":"kettle","x":2,"z":1,"rot":2,"offset":[0,-20]},
               {"def":"desk","x":5,"z":1,"rot":0},{"def":"laptop","x":5,"z":1,"rot":0},
               {"def":"chair","x":5,"z":2,"rot":2},
               {"def":"telescope","x":7,"z":2,"rot":0},{"def":"telescope","x":12,"z":3,"rot":0},
               {"def":"sink","x":6,"z":4,"rot":0},{"def":"washer","x":2,"z":4,"rot":0},{"def":"dryer","x":3,"z":4,"rot":0}],
    "sims":[{"name":"Ada","x":7.5,"z":4.5,"age":35}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.autonomy = false;
    w.tick = clock::tick_at(1, 12.0 * 60.0).unwrap();
    w
}

/// The `n`th object of kind `def`.
fn object(w: &World, def: &str, n: usize) -> u32 {
    w.objects.iter().filter(|o| w.content.objects[o.def].id == def).nth(n).unwrap_or_else(|| panic!("no {def} {n}")).id
}

fn order(w: &mut World, def: &str, n: usize) -> Result<(), String> {
    let object = w.objects[object(w, def, n) as usize].stable;
    w.apply(Command::Use { sim: 0, object, interaction: 0 }).map(|_| ()).map_err(|e| e.to_string())
}

/// Ticks until Ada is using `def` (or `minutes` pass): whether she got to it.
fn until_using(w: &mut World, def: &str, minutes: f32) -> bool {
    for _ in 0..(minutes / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        common::check(w);
        if using(w) == Some(def.to_string()) {
            return true;
        }
    }
    false
}

/// What Ada is using now.
fn using(w: &World) -> Option<String> {
    let a = w.sims[0].current()?;
    match (a.task.kind, &a.phase) {
        (TaskKind::Use { object, .. }, Phase::Using { .. }) => Some(w.content.objects[w.objects[object as usize].def].id.clone()),
        _ => None,
    }
}

fn tile(p: [f32; 2]) -> (i32, i32) {
    (p[0].floor() as i32, p[1].floor() as i32)
}

#[test]
fn things_on_tops_load_with_uses_but_are_not_lain_on_or_stood_in() {
    for (bad, why) in [(r#""pose":"lie""#, "standing or sitting"), (r#""inside":true"#, "standing or sitting"), (r#""needsTable":true"#, "standing or sitting")] {
        let content = CONTENT.replace(r#""minutes":8,"effects":{"comfort":0.3}"#, &format!(r#""minutes":8,{bad},"effects":{{"comfort":0.3}}"#));
        let err = Content::from_json(&content).map(|_| ()).unwrap_err().to_string();
        assert!(err.contains("kettle.tea") && err.contains(why), "{bad}: {err}");
    }
    let w = world();
    let kettle = &w.content.objects[w.objects[object(&w, "kettle", 0) as usize].def];
    assert_eq!(kettle.interactions.len(), 1);
    assert!(kettle.wear_per_use > 0.0, "used, it wears");
}

#[test]
fn a_kettle_on_the_counter_is_used_standing_in_front_of_the_counter() {
    let mut w = world();
    let kettle = object(&w, "kettle", 0);
    // Turned the other way on the counter, it is still used from the counter's front.
    assert_eq!(w.objects[kettle as usize].front_tile(&w.content), (2, 0));
    order(&mut w, "kettle", 0).unwrap();
    assert!(until_using(&mut w, "kettle", 30.0), "she got to the kettle");
    let ada = &w.sims[0];
    assert_eq!(tile(ada.pos), (2, 2), "in front of the counter");
    assert_eq!(ada.pose, Pose::Stand);
    // Facing the kettle (towards -z).
    assert!((ada.yaw.cos() + 1.0).abs() < 0.05, "facing the kettle: yaw {}", ada.yaw);
    let before = w.sims[0].needs[2];
    for _ in 0..(10.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
    }
    assert!(w.sims[0].needs[2] > before + 0.2, "the tea did her good");
    assert_eq!(tile(w.sims[0].pos), (2, 2), "she stays where she stood");
}

#[test]
fn with_the_front_taken_it_is_used_from_beside_the_top() {
    let mut w = world();
    // A chair in front of the counter: the kettle is used from the side.
    w.apply(Command::Buy { household: 0, object: "chair".into(), at: Some([2, 2, 0]), colors: Default::default(), turn: None, offset: None, size: None, tall: None }).unwrap();
    let kettle = object(&w, "kettle", 0);
    let links = sim_core::surface::Links::new(&w.content, &w.objects, &sim_core::path::NavGrid { lot: &w.lot, blocked: w.blocked(), stairs: w.stair_map() });
    assert_eq!(links.approach_of(kettle), Some((1, 1)), "beside the counter");
    order(&mut w, "kettle", 0).unwrap();
    assert!(until_using(&mut w, "kettle", 30.0));
    assert_eq!(tile(w.sims[0].pos), (1, 1));
}

#[test]
fn a_laptop_on_a_desk_is_used_sitting_at_it_or_standing_without_a_chair() {
    let mut w = world();
    let chair = object(&w, "chair", 0);
    order(&mut w, "laptop", 0).unwrap();
    assert!(until_using(&mut w, "laptop", 30.0));
    let ada = &w.sims[0];
    assert_eq!(ada.use_seat, Some(chair), "on the desk chair");
    assert_eq!(ada.pose, Pose::Sit);
    assert_eq!(ada.pos, w.objects[chair as usize].slot_position(&w.content, 0));
    // Done: up off the chair, and the chair is free again.
    for _ in 0..(50.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        common::check(&w);
    }
    assert!(using(&w).is_none());
    assert!(w.objects[chair as usize].users().next().is_none(), "the chair is free");

    // With the chair sold: standing at the desk, beside it (the front is free now).
    let mut w = world();
    let chair = w.objects[object(&w, "chair", 0) as usize].stable;
    w.apply(Command::Sell { household: 0, object: chair }).unwrap();
    order(&mut w, "laptop", 0).unwrap();
    assert!(until_using(&mut w, "laptop", 30.0));
    let ada = &w.sims[0];
    assert_eq!((ada.use_seat, ada.pose), (None, Pose::Stand), "standing at the desk");
    assert_eq!(tile(ada.pos), (5, 2), "in front of the desk");
}

#[test]
fn at_a_top_with_chairs_all_round_they_sit_down_to_use_it() {
    let mut w = world();
    // The desk between two counters, its chair in front, the wall behind: no side free.
    let buy = |w: &mut World, def: &str, x: i32, z: i32| {
        w.apply(Command::Buy { household: 0, object: def.into(), at: Some([x, z, 0]), colors: Default::default(), turn: None, offset: None, size: None, tall: None }).unwrap();
    };
    let laptop = w.objects[object(&w, "laptop", 0) as usize].stable;
    w.apply(Command::Sell { household: 0, object: laptop }).unwrap();
    buy(&mut w, "counter", 4, 1);
    buy(&mut w, "counter", 6, 1);
    buy(&mut w, "kettle", 5, 1);
    let chair = object(&w, "chair", 0);
    let kettle = object(&w, "kettle", 1);
    assert_eq!(w.objects[kettle as usize].x, 5);
    order(&mut w, "kettle", 1).unwrap();
    assert!(until_using(&mut w, "kettle", 30.0));
    assert_eq!((w.sims[0].use_seat, w.sims[0].pose), (Some(chair), Pose::Sit), "sat at the desk for it");
}

#[test]
fn free_will_uses_things_on_tops() {
    let mut w = world();
    w.autonomy = true;
    w.sims[0].needs[..3].copy_from_slice(&[1.0, 1.0, 0.2]);
    let mut kettle = false;
    for _ in 0..(120.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        common::check(&w);
        kettle |= using(&w).as_deref() == Some("kettle");
    }
    assert!(kettle, "a cup of tea of her own accord");
}

#[test]
fn a_thing_on_a_top_nobody_can_reach_may_still_be_put_there() {
    let mut w = world();
    // A table boxed in by the walls and two dressers: a book on it is a way nobody needs.
    w.apply(Command::Buy { household: 0, object: "desk".into(), at: Some([8, 1, 0]), colors: Default::default(), turn: None, offset: None, size: None, tall: None }).unwrap();
    w.apply(Command::Buy { household: 0, object: "counter".into(), at: Some([8, 2, 0]), colors: Default::default(), turn: None, offset: None, size: None, tall: None }).unwrap();
    w.apply(Command::Buy { household: 0, object: "counter".into(), at: Some([7, 1, 0]), colors: Default::default(), turn: None, offset: None, size: None, tall: None }).unwrap();
    w.apply(Command::Buy { household: 0, object: "kettle".into(), at: Some([8, 1, 0]), colors: Default::default(), turn: None, offset: None, size: None, tall: None })
        .expect("a kettle can go on a top nobody can get to");
}

#[test]
fn stargazing_is_after_dark_under_open_sky() {
    let mut w = world();
    // At noon: not yet, indoors or out.
    let err = order(&mut w, "telescope", 1).unwrap_err();
    assert!(err.contains("Ada can only stargaze from 20:00 to 5:00"), "{err}");
    // At night: not indoors.
    w.tick = clock::tick_at(1, 22.0 * 60.0).unwrap();
    let err = order(&mut w, "telescope", 0).unwrap_err();
    assert!(err.contains("under open sky"), "{err}");
    assert!(w.sims[0].queue().next().is_none(), "nothing refused was queued");
    // Outside, after dark (and past midnight too): yes.
    order(&mut w, "telescope", 1).unwrap();
    assert!(until_using(&mut w, "telescope", 30.0));
    let mut w = world();
    w.tick = clock::tick_at(2, 3.0 * 60.0).unwrap();
    order(&mut w, "telescope", 1).unwrap();
    // Sent out at 4:55, it's light by the time she'd start: nothing.
    let mut w = world();
    w.tick = clock::tick_at(2, 4.0 * 60.0 + 55.0).unwrap();
    order(&mut w, "telescope", 1).unwrap();
    w.sims[0].pos = [2.5, 2.5];
    assert!(!until_using(&mut w, "telescope", 20.0), "not after 5:00");
}

#[test]
fn free_will_stargazes_only_at_night() {
    for (hour, wanted) in [(12.0, false), (22.0, true)] {
        let mut w = world();
        w.autonomy = true;
        w.tick = clock::tick_at(1, hour * 60.0).unwrap();
        // (Not the laptop instead.)
        let laptop = w.objects[object(&w, "laptop", 0) as usize].stable;
        w.apply(Command::Sell { household: 0, object: laptop }).unwrap();
        w.sims[0].needs[..3].copy_from_slice(&[1.0, 0.1, 1.0]);
        let mut stargazed = None;
        for _ in 0..(90.0 / sim_core::MINUTES_PER_TICK) as u32 {
            w.tick_once();
            common::check(&w);
            if using(&w).as_deref() == Some("telescope") {
                let a = w.sims[0].current().unwrap();
                let TaskKind::Use { object, .. } = a.task.kind else { unreachable!() };
                stargazed = Some(object);
            }
        }
        assert_eq!(stargazed.is_some(), wanted, "at {hour}: {stargazed:?}");
        if let Some(o) = stargazed {
            assert_eq!(o, object(&w, "telescope", 1), "the one outside");
        }
    }
}

#[test]
fn bad_hours_are_refused_at_load() {
    for hours in ["[5,5]", "[25,3]", "[-1,4]"] {
        let content = CONTENT.replace(r#""hours":[20,5]"#, &format!(r#""hours":{hours}"#));
        let err = Content::from_json(&content).map(|_| ()).unwrap_err().to_string();
        assert!(err.contains("telescope.stargaze") && err.contains("hours"), "{hours}: {err}");
    }
    let content = CONTENT.replace(r#""workOn":"dishes""#, r#""workOn":"socks""#);
    assert!(Content::from_json(&content).is_err(), "an unknown workOn");
    let content = CONTENT.replace(r#""laundry":{"perDay":0.5,"most":3},"#, "");
    let err = Content::from_json(&content).map(|_| ()).unwrap_err().to_string();
    assert!(err.contains("washer.laundry") && err.contains("laundry"), "{err}");
}

fn dirty_plate(w: &mut World) -> u32 {
    let def = w.content.item_index("plateDirty").unwrap();
    w.items.add(ItemInstance { id: 0, def, at: ItemAt::Floor { x: 3, z: 3, offset: [0, 0] }, servings: 0, quality: 1.0, made_at: w.tick, household: 0, spoiled: false, recipe: None })
}

#[test]
fn washing_up_needs_dishes_to_wash() {
    let mut w = world();
    let err = order(&mut w, "sink", 0).unwrap_err();
    assert!(err.contains("Wash the dishes: there are no dishes to wash"), "{err}");
    dirty_plate(&mut w);
    order(&mut w, "sink", 0).unwrap();
    assert!(until_using(&mut w, "sink", 30.0));
}

#[test]
fn the_laundry_is_washed_then_dried_and_piles_up_again() {
    let mut w = world();
    assert_eq!(w.households[0].laundry, [1.0, 1.0, 0.0], "a load dirty and one washed to start with");
    // Fold the washed load...
    order(&mut w, "dryer", 0).unwrap();
    assert!(until_using(&mut w, "dryer", 30.0));
    w.tick_once();
    assert_eq!(w.households[0].laundry[1], 0.0, "taken up to dry and fold");
    for _ in 0..(25.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
    }
    // ...and there's no more to fold until a wash.
    let err = order(&mut w, "dryer", 0).unwrap_err();
    assert!(err.contains("no washed laundry"), "{err}");
    order(&mut w, "washer", 0).unwrap();
    assert!(until_using(&mut w, "washer", 30.0));
    w.tick_once();
    assert!(w.households[0].laundry[0] < 0.1, "taken up to wash: {:?}", w.households[0].laundry);
    // Washed once the use is done (content `makes`), and carried on to the dryer.
    assert!(until_using(&mut w, "dryer", 35.0), "on to the dryer with it");
    w.tick_once();
    let [dirty, washed, _] = w.households[0].laundry;
    assert!(dirty < 0.1 && washed == 0.0, "washed and taken up to dry: {dirty}, {washed}");
    for _ in 0..(25.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
    }
    let err = order(&mut w, "washer", 0).unwrap_err();
    assert!(err.contains("no washing to do"), "{err}");
    // A day on, Ada (perDay 0.5) has half a load more; two days, a load to wash.
    for _ in 0..(2.0 * 1440.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
    }
    assert!(w.households[0].laundry[0] >= 1.0, "{:?}", w.households[0].laundry);
    order(&mut w, "washer", 0).unwrap();
    // Saved and loaded, the washing waiting is kept; a save from before it starts with a load of each.
    let saved = w.save_json();
    let loaded = World::from_save_json(CONTENT, &saved).unwrap();
    assert_eq!(loaded.households[0].laundry, w.households[0].laundry);
    let mut old: serde_json::Value = serde_json::from_str(&saved).unwrap();
    old["version"] = 29.into();
    old["households"][0].as_object_mut().unwrap().remove("laundry");
    let loaded = World::from_save_json(CONTENT, &old.to_string()).unwrap();
    assert_eq!(loaded.households[0].laundry, [1.0, 1.0, 0.0]);
}

#[test]
fn free_will_does_no_chore_with_nothing_to_work_on() {
    let mut w = world();
    w.autonomy = true;
    w.households[0].laundry = [0.0, 0.0, 0.0];
    w.sims[0].needs[..3].copy_from_slice(&[1.0, 1.0, 0.3]);
    for _ in 0..(240.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.sims[0].needs[2] = w.sims[0].needs[2].min(0.3);
        w.tick_once();
        let u = using(&w);
        assert!(u.as_deref() != Some("washer") && u.as_deref() != Some("dryer"), "nothing to wash: {u:?}");
    }
    // With washing to do, she does it.
    w.households[0].laundry = [2.0, 0.0, 0.0];
    let mut washed = false;
    for _ in 0..(240.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.sims[0].needs[2] = w.sims[0].needs[2].min(0.3);
        w.tick_once();
        washed |= using(&w).as_deref() == Some("washer");
    }
    assert!(washed, "the washing done of her own accord");
}

/// Coffee (making one, having one, pulling an espresso, latte art) is for teens and up
/// (I-systems-11).
#[test]
fn coffee_is_for_teens_and_up() {
    let content = Content::from_json(&common::content()).unwrap();
    let stage = |id: &str| content.life.stages.iter().position(|s| s.id == id).unwrap();
    let mut coffees = 0;
    for o in &content.objects {
        for i in o.interactions.iter().filter(|i| ["coffee", "espresso", "latte"].iter().any(|c| i.label.to_lowercase().contains(c))) {
            coffees += 1;
            assert!(!i.for_stage(stage("child")), "{}.{}: a child's coffee", o.id, i.id);
            assert!(i.for_stage(stage("teen")) && i.for_stage(stage("adult")), "{}.{}", o.id, i.id);
        }
    }
    assert!(coffees >= 20, "{coffees} coffees");
}

