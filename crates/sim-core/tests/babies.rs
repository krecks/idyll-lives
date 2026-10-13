//! Babies: they lie in a crib at home and grown-ups care for them there (feeding, changing,
//! playing fill the baby's needs); a baby who needs something cries, lying awake, and stays in
//! the crib (as does one who's rested); a crib comes with a baby who moves into a home without
//! one (free, money or not); and a baby who grows into a child gets out, onto their feet beside
//! it. With the shipped content, a baby with a grown-up at home is cuddled and stays
//! comfortable.

mod common;

use sim_core::items::ItemAt;
use sim_core::lot::SimSpawn;
use sim_core::planner::ThoughtKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1},{"id":"energy","label":"Energy","decayPerHour":0.02},
             {"id":"fun","label":"Fun","decayPerHour":0.0}],
    "objects":[{"id":"crib","name":"Crib","price":350,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"lie","label":"Lie in the crib","minutes":720,"pose":"lie","baby":true,"autonomous":false,"effects":{"energy":1.0},"tags":["rest"]},
        {"id":"feed","label":"Feed the baby","minutes":15,"care":{"hunger":0.8},"tags":["care"]}]},
      {"id":"armchair","name":"Armchair","price":100,"interactions":[
        {"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"fun":0.3},"tags":["lounge"]}]}],
    "life":{"daysPerYear":2,"stages":[
        {"id":"baby","label":"Baby","from":0,"baby":true},
        {"id":"child","label":"Child","from":2,"school":true},
        {"id":"adult","label":"Adult","from":18}]},
    "bondPresets":{"roommates":{"friendship":20},"parent":{"friendship":50,"kin":"parent"}},
    "economy":{"startingFunds":1000}}"#;

const TOWN: &str = r#"{"width":20,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":10,"d":12,"entry":[5.5,11.5]},
             {"name":"Next door","x":10,"z":0,"w":10,"d":12,"entry":[15.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true},{"name":"Empty","plot":1}],
    "objects":[{"def":"crib","x":2,"z":2},{"def":"armchair","x":6,"z":6}],
    "sims":[{"name":"Ada","x":4.5,"z":8.5,"age":30},{"name":"Bea","x":5.5,"z":8.5,"age":0.5}],
    "relationships":[{"a":1,"b":0,"preset":"parent"}]}"#;

fn minutes(w: &mut World, m: u32) {
    for _ in 0..m * 20 {
        w.tick_once();
    }
}

#[test]
fn babies_lie_in_the_crib_and_are_fed() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    minutes(&mut w, 1);
    assert_eq!(w.sims[1].pose, sim_core::content::Pose::Lie, "Bea is in the crib");
    assert!(w.objects[0].users().any(|u| u == 1));

    // Hungry: she cries, and Ada feeds her.
    w.sims[1].needs[0] = 0.1;
    minutes(&mut w, 1);
    assert!(w.sims[1].planner.thought.is_some_and(|t| t.kind == ThoughtKind::Crying));
    let mut fed = false;
    for _ in 0..90 {
        minutes(&mut w, 1);
        fed |= w.sims[0].current().is_some_and(|a| matches!(a.task.kind, sim_core::world::TaskKind::Use { object: 0, interaction: 1 }));
    }
    assert!(fed, "Ada fed the baby");
    assert!(w.sims[1].needs[0] > 0.6, "and Bea isn't hungry now: {}", w.sims[1].needs[0]);
    assert_eq!(w.sims[1].pose, sim_core::content::Pose::Lie, "still in the crib");

    // Two years on (four days at two a year) she's a child and gets out.
    for _ in 0..4 {
        minutes(&mut w, 24 * 60);
    }
    assert!(!w.content.life.baby(w.sims[1].age));
    assert!(!w.objects[0].users().any(|u| u == 1), "out of the crib");
}

#[test]
fn a_baby_lying_in_the_crib_grows_up_and_climbs_out() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    // Fed, and never quite rested: she lies in the crib until the night she's a child.
    w.sims[1].age = 1.99;
    let midnight = sim_core::clock::tick_at(2, 0.0).unwrap();
    while w.tick < midnight - 1 {
        w.sims[1].needs[..2].copy_from_slice(&[1.0, 0.5]);
        w.tick_once();
    }
    assert_eq!(w.sims[1].pose, sim_core::content::Pose::Lie, "in the crib at midnight");
    w.tick_once();
    assert!(!w.content.life.baby(w.sims[1].age), "a child now");
    assert_eq!(w.sims[1].pose, sim_core::content::Pose::Stand);
    let (x, z) = w.sims[1].tile();
    assert!(!(2..4).contains(&x) || z != 2, "standing in the crib at {x},{z}");
    w.apply(Command::Use { sim: 1, object: 1, interaction: 0 }).unwrap();
    minutes(&mut w, 10);
    assert_eq!(w.sims[1].pose, sim_core::content::Pose::Sit, "she walked over and sat down");
}

#[test]
fn a_baby_in_a_home_with_no_money_gets_a_crib_and_is_looked_after() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let spawn = |name: &str, age: f32| -> SimSpawn {
        serde_json::from_value(serde_json::json!({"name": name, "x": 15.5, "z": 11.5, "age": age})).unwrap()
    };
    w.households[1].funds = 0;
    w.apply(Command::MoveIn { household: 1, name: None, sims: vec![spawn("Cal", 28.0), spawn("Dot", 0.2)], bonds: vec![] }).unwrap();
    assert_eq!(w.households[1].funds, 0, "the crib was free");
    let crib = w.objects.iter().position(|o| w.content.objects[o.def].id == "crib" && o.x >= 10).expect("a crib next door");
    // Two days: Cal feeds her whenever she's hungry.
    let (mut sum, mut lowest) = (0.0, 1.0f32);
    for _ in 0..48 {
        minutes(&mut w, 60);
        sum += w.sims[3].needs[0];
        lowest = lowest.min(w.sims[3].needs[0]);
    }
    assert!(w.objects[crib].users().any(|u| u == 3), "Dot is in her crib");
    assert!(sum / 48.0 > 0.5 && lowest > 0.2, "Dot was fed: average {:.2}, lowest {lowest:.2}", sum / 48.0);
}

#[test]
fn a_baby_moving_in_comes_with_a_crib() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let spawn = |name: &str, age: f32| -> SimSpawn {
        serde_json::from_value(serde_json::json!({"name": name, "x": 15.5, "z": 11.5, "age": age})).unwrap()
    };
    let cribs = |w: &World| w.objects.iter().filter(|o| w.content.objects[o.def].id == "crib").count();
    assert_eq!(cribs(&w), 1);
    w.apply(Command::MoveIn { household: 1, name: None, sims: vec![spawn("Cal", 28.0), spawn("Dot", 0.2)], bonds: vec![] }).unwrap();
    assert_eq!(cribs(&w), 2, "a crib for Dot next door");
    minutes(&mut w, 1);
    assert_eq!(w.sims[3].pose, sim_core::content::Pose::Lie, "Dot is in it");
}

const FAMILY: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.0}],
    "objects":[{"id":"crib","name":"Crib","price":350,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"lie","label":"Lie in the crib","minutes":720,"pose":"lie","baby":true,"autonomous":false,"effects":{"hunger":0.0},"tags":["rest"]}]}],
    "genders":[{"id":"female","label":"Female"},{"id":"male","label":"Male"}],
    "names":{"first":["Ivy","Jun"],"last":["Moss"]},
    "tags":["romantic"],
    "socials":[{"id":"tryForBaby","label":"Try for a baby","minutes":30,"tags":["romantic"],"requires":{"partners":true},
                "acceptance":{"base":1.0},"success":{"effect":"conceive"}}],
    "socialRules":{"romanticTags":["romantic"]},
    "life":{"daysPerYear":2,"stages":[
        {"id":"baby","label":"Baby","from":0,"baby":true},
        {"id":"child","label":"Child","from":2,"school":true},
        {"id":"adult","label":"Adult","from":18}],
        "pregnancy":{"chance":1.0,"days":2,"maxAge":50},"adoption":{"cost":500}},
    "bondPresets":{"partners":{"friendship":55,"romance":75,"partners":true},"parent":{"friendship":50,"kin":"parent"},
                   "siblings":{"friendship":40,"kin":"sibling"}},
    "rules":{"maxHousehold":4},
    "economy":{"startingFunds":1000}}"#;

const COUPLE: &str = r#"{"width":12,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[5.5,11.5]}],
    "households":[{"name":"Moss","plot":0,"player":true}],
    "objects":[],
    "sims":[{"name":"Ada","x":4.5,"z":8.5,"age":30,"gender":"female"},{"name":"Ben","x":5.5,"z":8.5,"age":32,"gender":"male"}],
    "relationships":[{"a":0,"b":1,"preset":"partners"}]}"#;

#[test]
fn partners_have_a_baby() {
    use sim_core::social::{EventKind, Kin};
    let mut w = World::from_json(FAMILY, COUPLE, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w.apply(Command::Social { sim: 0, target: 1, social: 0 }).unwrap();
    minutes(&mut w, 60);
    let e = w.households[0].expecting.expect("expecting");
    assert_eq!(e.parents, [0, 1]);
    assert!(w.events.iter().any(|e| e.kind == EventKind::Expecting));
    // Saved expecting.
    let loaded = World::from_save_json(FAMILY, &w.save_json()).unwrap();
    assert_eq!(loaded.households[0].expecting, Some(e));
    // Born two days later, into a crib that came for them.
    minutes(&mut w, 3 * 24 * 60);
    assert_eq!(w.sims.len(), 3, "a baby");
    let baby = &w.sims[2];
    assert!(w.content.life.baby(baby.age) && ["Ivy", "Jun"].contains(&baby.name.as_str()));
    assert_eq!(baby.appearance["parents"], serde_json::json!([0, 1]), "looks from the parents");
    assert_eq!((w.relationships.kin(2, 0), w.relationships.kin(0, 2)), (Kin::Parent, Kin::Child));
    assert!(w.events.iter().any(|e| e.kind == EventKind::Born && e.a == 2 && e.b == 0 && e.c == Some(1)));
    assert!(w.households[0].expecting.is_none());
    assert_eq!(w.objects.iter().filter(|o| w.content.objects[o.def].id == "crib").count(), 1);
    assert_eq!(w.sims[2].pose, sim_core::content::Pose::Lie, "in the crib");
}

#[test]
fn adopting_a_child() {
    use sim_core::social::Kin;
    let mut w = World::from_json(FAMILY, COUPLE, 1).unwrap();
    w.apply(Command::Adopt { household: 0, child: true }).unwrap();
    let kid = &w.sims[2];
    assert!((3.0..11.0).contains(&kid.age), "a child: {}", kid.age);
    assert_eq!((w.relationships.kin(2, 0), w.relationships.kin(2, 1)), (Kin::Parent, Kin::Parent), "both are parents");
    assert_eq!(w.households[0].funds, 500, "adoption costs");
    w.apply(Command::Adopt { household: 0, child: false }).unwrap();
    assert_eq!(w.relationships.kin(3, 2), Kin::Sibling, "a little brother or sister");
    let err = w.apply(Command::Adopt { household: 0, child: false }).unwrap_err();
    assert!(err.to_string().contains("money") || err.to_string().contains("room"), "{err}");
}

#[test]
fn neighbours_adopt_now_and_then() {
    let content = FAMILY.replace(r#""adoption":{"cost":500}"#, r#""adoption":{"cost":500,"neighbours":1.0},"newcomers":{"hour":12,"days":1000}"#);
    let town = COUPLE.replace(r#""player":true"#, r#""player":false"#)
        .replace(r#""households":[{"name":"Moss","plot":0,"player":false}]"#, r#""households":[{"name":"Moss","plot":0,"player":false},{"name":"Player","player":true}]"#);
    let mut w = World::from_json(&content, &town, 1).unwrap();
    w.households[0].funds = 5000;
    minutes(&mut w, 24 * 60);
    assert!(w.events.iter().any(|e| e.kind == sim_core::social::EventKind::Adopted), "the neighbours adopted");
    assert_eq!(w.households[0].funds, 4500);
}

/// Bladder too (and a toilet at home, which a baby can't get up for), a grown-up to change the
/// baby, and the shipped content's needs that wake sleepers and are urgent, and what a night's
/// sleep fills (energy, from a bed).
const NURSERY: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.0},{"id":"energy","label":"Energy","decayPerHour":0.02},
             {"id":"bladder","label":"Bladder","decayPerHour":0.1}],
    "objects":[{"id":"crib","name":"Crib","price":350,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"lie","label":"Lie in the crib","anim":"sleep","minutes":720,"pose":"lie","baby":true,"autonomous":false,"effects":{"energy":1.0},"tags":["rest"]},
        {"id":"change","label":"Change the baby","minutes":10,"care":{"bladder":1.0},"tags":["care"]}]},
      {"id":"armchair","name":"Armchair","price":100,"interactions":[]},
      {"id":"toilet","name":"Toilet","price":100,"interactions":[
        {"id":"use","label":"Use","minutes":5,"effects":{"bladder":1.0},"tags":["toilet"]}]},
      {"id":"bed","name":"Bed","price":300,"interactions":[
        {"id":"sleep","label":"Sleep","anim":"sleep","minutes":480,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"]}]}],
    "animations":["sleep"],
    "dayRhythm":{"sleepTags":["sleep"],"wakeFor":["bladder","hunger"]},
    "autonomy":{"urgent":["hunger","energy","bladder"],"urgentBelow":0.1},
    "life":{"daysPerYear":2,"stages":[
        {"id":"baby","label":"Baby","from":0,"baby":true},
        {"id":"adult","label":"Adult","from":18}]},
    "bondPresets":{"parent":{"friendship":50,"kin":"parent"}},
    "economy":{"startingFunds":1000}}"#;

fn nursery() -> World {
    let town = TOWN.replace(r#"{"def":"armchair","x":6,"z":6}"#, r#"{"def":"armchair","x":6,"z":6},{"def":"toilet","x":8,"z":2}"#);
    World::from_json(NURSERY, &town, 1).unwrap()
}

/// How long (minutes) Bea has been lying in the crib, while she's in it (lying there, using it).
fn in_crib(w: &World) -> Option<f32> {
    let a = w.sims[1].current()?;
    let lying = matches!(a.task.kind, sim_core::world::TaskKind::Use { object: 0, interaction: 0 });
    let sim_core::world::Phase::Using { elapsed } = a.phase else { return None };
    (lying && w.sims[1].pose == sim_core::content::Pose::Lie && w.objects[0].users().any(|u| u == 1)).then_some(elapsed)
}

/// Bea is still in the crib, the same stay as `last` (it hasn't started over): how long now.
fn still_in_crib(w: &World, last: f32, what: &str) -> f32 {
    let now = in_crib(w).unwrap_or_else(|| panic!("{what}: Bea left the crib at tick {}", w.tick));
    assert!(now > last, "{what}: Bea's crib stay started over at tick {} ({last} minutes, now {now})", w.tick);
    now
}

#[test]
fn a_baby_stays_in_the_crib_needing_something_or_rested() {
    for (need, level) in [(2, 0.05), (1, 1.0)] {
        let mut w = nursery();
        // Nobody to see to her: Ada is out of the way.
        w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
        minutes(&mut w, 1);
        let mut lain = in_crib(&w).expect("Bea is in the crib");
        // Wet (or wide awake), for two hours: she lies there, the same stay all along.
        for _ in 0..2 * 60 * 20 {
            w.sims[1].needs[need] = level;
            w.tick_once();
            lain = still_in_crib(&w, lain, &format!("need {need} at {level}"));
        }
    }
}

#[test]
fn a_wet_baby_is_changed_in_the_crib() {
    let mut w = nursery();
    minutes(&mut w, 1);
    let mut lain = in_crib(&w).expect("Bea is in the crib");
    w.sims[1].needs[2] = 0.05;
    let mut changed = false;
    for _ in 0..90 * 20 {
        w.tick_once();
        changed |= w.sims[0].current().is_some_and(|a| matches!(a.task.kind, sim_core::world::TaskKind::Use { object: 0, interaction: 1 }));
        lain = still_in_crib(&w, lain, "wet");
        if w.sims[1].needs[2] > 0.9 {
            break;
        }
    }
    assert!(changed, "Ada changed the baby");
    assert!(w.sims[1].needs[2] > 0.9, "and Bea's dry: {}", w.sims[1].needs[2]);
}

#[test]
fn a_baby_lies_awake_in_the_crib_needing_something_or_rested_by_day() {
    use sim_core::snapshot::{self, HEADER_LEN, SIM_STRIDE, sim};
    let action = |w: &World| {
        let mut out = vec![0.0; snapshot::CAPACITY];
        snapshot::write(w, &mut out);
        out[HEADER_LEN + SIM_STRIDE + sim::ACTION]
    };
    let mut w = nursery();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    let until = |w: &mut World, day: u32, hour: f32, needs: [f32; 3]| {
        let at = sim_core::clock::tick_at(day, hour * 60.0).unwrap();
        while w.tick < at {
            w.sims[1].needs[..3].copy_from_slice(&needs);
            w.tick_once();
            assert!(in_crib(w).is_some(), "Bea left the crib at tick {}", w.tick);
        }
    };
    minutes(&mut w, 1);
    // Asleep in the night, and in the day until she's rested.
    until(&mut w, 2, 2.0, [1.0, 0.8, 1.0]);
    let lain = in_crib(&w).expect("Bea is in the crib");
    assert_eq!(action(&w), 0.0, "asleep at night");
    until(&mut w, 2, 12.0, [1.0, 0.8, 1.0]);
    assert_eq!(action(&w), 0.0, "napping by day");
    // Rested by day, awake; wet in the night, awake (crying).
    until(&mut w, 2, 13.0, [1.0, 1.0, 1.0]);
    assert_eq!(action(&w), -1.0, "awake, rested, by day");
    until(&mut w, 3, 2.0, [1.0, 0.8, 0.05]);
    assert_eq!(action(&w), -1.0, "awake, crying, at night");
    still_in_crib(&w, lain, "all along");
}

/// Two parents asleep in their beds and a baby in the crib who gets hungry in the night (the
/// grown-ups don't, asleep), with a crying baby waking someone (`dayRhythm.babyWakes`).
const NIGHT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1},{"id":"energy","label":"Energy","decayPerHour":0.02}],
    "objects":[{"id":"crib","name":"Crib","price":350,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"lie","label":"Lie in the crib","anim":"sleep","minutes":720,"pose":"lie","baby":true,"autonomous":false,"effects":{"energy":1.0},"tags":["rest"]},
        {"id":"feed","label":"Feed the baby","minutes":15,"care":{"hunger":0.8},"tags":["care"]}]},
      {"id":"bed","name":"Bed","price":300,"interactions":[
        {"id":"sleep","label":"Sleep","anim":"sleep","minutes":480,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"]}]},
      {"id":"fridge","name":"Fridge","price":100,"interactions":[
        {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.6}}]}],
    "animations":["sleep"],
    "feelings":[{"id":"brokenSleep","label":"Up in the night","mood":-0.05,"hours":8}],
    "dayRhythm":{"sleepTags":["sleep"],"wakeFor":["hunger"],"asleepDecay":{"hunger":0.0},
                 "babyWakes":{"below":0.3,"cooldownMinutes":60,"feeling":"brokenSleep"}},
    "life":{"daysPerYear":2,"stages":[
        {"id":"baby","label":"Baby","from":0,"baby":true},
        {"id":"adult","label":"Adult","from":18}]},
    "bondPresets":{"partners":{"friendship":55,"romance":75,"partners":true},"parent":{"friendship":50,"kin":"parent"}},
    "economy":{"startingFunds":1000}}"#;

const NIGHT_TOWN: &str = r#"{"width":12,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[5.5,11.5]}],
    "households":[{"name":"Moss","plot":0,"player":true}],
    "objects":[{"def":"crib","x":2,"z":2},{"def":"bed","x":6,"z":2},{"def":"bed","x":9,"z":2},{"def":"fridge","x":9,"z":9}],
    "sims":[{"name":"Ada","x":4.5,"z":8.5,"age":30},{"name":"Ben","x":5.5,"z":8.5,"age":32},{"name":"Bea","x":6.5,"z":8.5,"age":0.5}],
    "relationships":[{"a":0,"b":1,"preset":"partners"},{"a":2,"b":0,"preset":"parent"},{"a":2,"b":1,"preset":"parent"}]}"#;

/// On until `hour` (of day 2: the first night's small hours) with Bea's hunger held at `hunger`
/// (None: it falls), calling `each` every tick; how low her hunger got.
fn night_until(w: &mut World, hour: f32, hunger: Option<f32>, mut each: impl FnMut(&World)) -> f32 {
    let at = sim_core::clock::tick_at(2, hour * 60.0).unwrap();
    let mut lowest = 1.0f32;
    while w.tick < at {
        if let Some(h) = hunger {
            w.sims[2].needs[0] = h;
        }
        w.tick_once();
        lowest = lowest.min(w.sims[2].needs[0]);
        each(w);
    }
    lowest
}

/// Both parents in bed at one in the morning (tired at bedtime), Bea fed until then.
fn night() -> World {
    let mut w = World::from_json(NIGHT, NIGHT_TOWN, 1).unwrap();
    let bedtime = sim_core::clock::tick_at(1, 22.0 * 60.0).unwrap();
    while w.tick < bedtime {
        w.tick_once();
    }
    w.sims[0].needs[1] = 0.3;
    w.sims[1].needs[1] = 0.35;
    night_until(&mut w, 1.0, Some(1.0), |_| {});
    let doing = |i: usize| w.sims[i].current().map(|a| a.task.kind);
    assert!(w.sims[0].asleep(&w.content) && w.sims[1].asleep(&w.content), "both asleep at one: {:?} {:?}", doing(0), doing(1));
    w
}

fn feeding(w: &World, i: usize) -> bool {
    w.sims[i].current().is_some_and(|a| matches!(a.task.kind, sim_core::world::TaskKind::Use { object: 0, interaction: 1 }))
}

#[test]
fn a_crying_baby_wakes_one_parent_at_night_who_feeds_her_and_goes_back_to_bed() {
    let mut w = night();
    // Getting hungry: below the crying level in half an hour.
    w.sims[2].needs[0] = 0.35;
    let (mut woke, mut fed) = ([false; 2], [false; 2]);
    let lowest = night_until(&mut w, 3.0, None, |w| {
        for i in 0..2 {
            woke[i] |= !w.sims[i].asleep(&w.content);
            fed[i] |= feeding(w, i);
        }
    });
    assert_eq!(woke.iter().filter(|&&x| x).count(), 1, "one parent woke: {woke:?}");
    let (up, other) = if woke[0] { (0, 1) } else { (1, 0) };
    assert!(fed[up], "{} fed the baby", w.sims[up].name);
    assert!(!fed[other] && w.sims[other].asleep(&w.content), "{} slept on", w.sims[other].name);
    assert!(w.sims[up].asleep(&w.content), "{} is back in bed", w.sims[up].name);
    assert!(w.sims[up].feelings.iter().any(|f| w.content.feelings[f.def].id == "brokenSleep"), "up in the night");
    assert!(w.sims[2].needs[0] > 0.8, "Bea is fed: {}", w.sims[2].needs[0]);
    // And she's looked after till morning (she gets hungry again before then).
    let lowest = lowest.min(night_until(&mut w, 6.5, None, |_| {}));
    assert!(lowest > 0.2, "Bea's hunger never fell far: {lowest}");
}

#[test]
fn partners_take_turns_getting_up_for_the_baby() {
    let mut w = night();
    let mut up = Vec::new();
    for hour in [1.0, 2.5, 4.0] {
        night_until(&mut w, hour, Some(1.0), |_| {});
        w.sims[2].needs[0] = 0.25;
        let mut who = None;
        night_until(&mut w, hour + 1.0, None, |w| {
            who = who.or((0..2).find(|&i| feeding(w, i)));
        });
        up.push(who.expect("someone fed her"));
    }
    assert!(up[0] != up[1] && up[1] != up[2], "turn and turn about: {up:?}");
}

#[test]
fn nobody_wakes_when_the_baby_needs_nothing() {
    let mut w = night();
    // Not hungry enough to cry: everyone sleeps through.
    night_until(&mut w, 6.0, Some(0.4), |w| {
        assert!(w.sims[0].asleep(&w.content) && w.sims[1].asleep(&w.content), "woken at tick {}", w.tick);
    });
}

/// `NIGHT` with a television for a night owl (and a 4 am shift for an early riser).
fn night_owl_content() -> String {
    NIGHT
        .replace(
            r#"{"id":"energy","label":"Energy","decayPerHour":0.02}]"#,
            r#"{"id":"energy","label":"Energy","decayPerHour":0.02},{"id":"fun","label":"Fun","decayPerHour":0.0}]"#,
        )
        .replace(
            r#""animations":["sleep"],"#,
            r#"{"id":"tv","name":"Television","price":200,"interactions":[
                {"id":"watch","label":"Watch TV","minutes":240,"effects":{"fun":0.2},"tags":["lounge"]}]}],
            "skills":[{"id":"writing","label":"Writing"}],
            "grades":[{"id":"A","label":"Entry","payPerHour":10,"skillLevel":0}],
            "careerCategories":[{"id":"press","label":"Press","shift":{"start":4,"hours":8,"days":[0,1,2,3,4,5,6]},
              "tracks":[{"id":"press.print","label":"Print","skills":{"writing":1},"titles":["Runner"]}]}],
            "animations":["sleep"],"#,
        )
        .replacen(r#""effects":{"hunger":0.6}}]}],"#, r#""effects":{"hunger":0.6}}]},"#, 1)
}

/// Ben in bed at one in the morning, Ada up watching television (on her own: nobody told her
/// to; with `job`, she leaves for work at half past three), Bea fed until `hour`.
fn night_owl(job: &str, hour: f32) -> World {
    night_owl_in(&night_owl_content(), "", job, hour)
}

/// `night_owl` with `content`, and `objects` (JSON, each with a comma before) after the
/// television.
fn night_owl_in(content: &str, objects: &str, job: &str, hour: f32) -> World {
    let town = NIGHT_TOWN
        .replace(r#"{"def":"fridge","x":9,"z":9}"#, &format!(r#"{{"def":"fridge","x":9,"z":9}},{{"def":"tv","x":6,"z":9}}{objects}"#))
        .replace(r#""age":30}"#, &format!(r#""age":30{job}}}"#));
    let mut w = World::from_json(content, &town, 1).unwrap();
    let bedtime = sim_core::clock::tick_at(1, 22.0 * 60.0).unwrap();
    while w.tick < bedtime {
        w.tick_once();
    }
    w.sims[0].needs[1] = 1.0;
    w.sims[0].needs[2] = 0.0;
    w.sims[1].needs[1] = 0.35;
    w.sims[1].needs[2] = 1.0;
    night_until(&mut w, hour, Some(1.0), |_| {});
    let doing = |i: usize| w.sims[i].current().map(|a| a.task);
    assert!(w.sims[1].asleep(&w.content), "Ben asleep: {:?}", doing(1));
    assert!(
        matches!(doing(0), Some(sim_core::world::Task { kind: sim_core::world::TaskKind::Use { object: 4, .. }, directed: false, .. })),
        "Ada watching TV: {:?}",
        doing(0)
    );
    w
}

#[test]
fn a_parent_up_at_night_sees_to_the_crying_baby_and_the_other_sleeps_on() {
    let mut w = night_owl("", 1.0);
    // Crying with hunger: Ada leaves the television for her within minutes.
    w.sims[2].needs[0] = 0.25;
    let start = w.tick;
    let mut fed_at = None;
    let lowest = night_until(&mut w, 4.0, None, |w| {
        assert!(w.sims[1].asleep(&w.content), "Ben woken at tick {}", w.tick);
        if fed_at.is_none() && feeding(w, 0) {
            fed_at = Some(w.tick);
        }
    });
    let fed_at = fed_at.expect("Ada fed Bea");
    let waited = (fed_at - start) as f32 * sim_core::MINUTES_PER_TICK;
    assert!(waited < 5.0, "Ada saw to Bea promptly: {waited} minutes");
    assert!(lowest > 0.2, "Bea's hunger never fell far: {lowest}");
    assert!(!w.sims[0].feelings.iter().any(|f| w.content.feelings[f.def].id == "brokenSleep"), "Ada wasn't woken");
}

#[test]
fn a_parent_about_to_leave_for_work_leaves_the_crying_baby_to_the_one_asleep() {
    let mut w = night_owl(r#","job":{"career":"press.print","level":0}"#, 3.1);
    // Crying with hunger as Ada is about to set off (at half past three): Ben is woken at once.
    w.sims[2].needs[0] = 0.25;
    let (ben, ada) = first_to_feed(&mut w, 4.0);
    assert!(ben.is_some_and(|m| m < 5.0) && ada.is_none(), "Ben fed Bea promptly (Ben {ben:?}, Ada {ada:?} minutes on)");
    assert!(w.sims[2].needs[0] > 0.8, "Bea is fed: {}", w.sims[2].needs[0]);
    assert!(w.sims[0].away_until.is_some() || w.sims[0].current().is_some_and(|a| matches!(a.task.kind, sim_core::world::TaskKind::Work)), "Ada went to work");
}

#[test]
fn a_parent_told_to_watch_tv_leaves_the_crying_baby_to_the_one_asleep() {
    let mut w = night_owl("", 1.0);
    w.apply(Command::Use { sim: 0, object: 4, interaction: 0 }).unwrap();
    w.sims[2].needs[0] = 0.25;
    let (ben, ada) = first_to_feed(&mut w, 2.0);
    assert!(ben.is_some_and(|m| m < 5.0) && ada.is_none(), "Ben fed Bea promptly (Ben {ben:?}, Ada {ada:?} minutes on)");
}

/// `night_owl_content` with a kitchen: a fridge with `fridge` (its interactions) that keeps
/// food and gives ingredients, a snack that's a plate to sit down with (half an hour's eating),
/// a meal made at the fridge, a counter (ten minutes) and the stove (half an hour), and a
/// table to eat at.
fn night_kitchen_content(fridge: &str) -> String {
    night_owl_content()
        .replace(
            r#"{"id":"fridge","name":"Fridge","price":100,"interactions":["#,
            r#"{"id":"fridge","name":"Fridge","price":100,"stations":["ingredients"],"stores":true,"interactions":["#,
        )
        .replace(r#"{"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.6}}"#, fridge)
        .replace(
            r#""tags":["lounge"]}]}],"#,
            r#""tags":["lounge"]}]},
            {"id":"table","name":"Dining set","price":300,"footprint":[2,1],"slots":2,"surface":{"height":0.75,"kind":"table"},
             "interactions":[{"id":"coffee","label":"Coffee","minutes":20,"pose":"sit","autonomous":false,"effects":{"energy":0.05},"tags":["food"]}]},
            {"id":"counter","name":"Counter","price":100,"stations":["prep"],"surface":{"height":0.9,"kind":"counter"}},
            {"id":"stove","name":"Stove","price":400,"stations":["stove"]}],
            "items":[{"id":"snack","label":"Snack","emptyAs":"plateDirty"},{"id":"plateDirty","label":"Dirty plate","dirty":true},
              {"id":"ingredients","label":"Ingredients"},{"id":"prepBowl","label":"Prepped bowl"},
              {"id":"mealPot","label":"Home-cooked meal","serving":"mealPlate","emptyAs":"potDirty","radius":0.16},
              {"id":"mealPlate","label":"Plate of food","emptyAs":"plateDirty"},{"id":"potDirty","label":"Dirty pot","dirty":true,"radius":0.16}],
            "recipes":[{"id":"meal","label":"Make a meal","item":"mealPot","servings":[1,4],"costPerServing":3,"effects":{"hunger":0.9},
              "steps":[{"at":["ingredients"],"minutes":5,"holds":"ingredients"},{"at":["prep"],"minutes":10,"holds":"prepBowl"},
                       {"at":["stove"],"minutes":30}]}],"#,
        )
}

const SNACK: &str = r#"{"id":"snack","label":"Snack","minutes":2,"eat":{"item":"snack","minutes":30},"effects":{"hunger":0.6},"tags":["food"]}"#;
const MEAL: &str = r#"{"id":"meal","label":"Make a meal","recipe":"meal","tags":["food","cooking"]}"#;
/// The table, counter and stove for `night_kitchen_content` (`night_owl_in`).
const KITCHEN: &str = r#",{"def":"table","x":2,"z":6},{"def":"counter","x":10,"z":9},{"def":"stove","x":11,"z":9}"#;

/// On (Bea fed) until `done`; panics after two hours.
fn until(w: &mut World, what: &str, done: impl Fn(&World) -> bool) {
    let limit = w.tick + 2 * 60 * 20;
    while !done(w) {
        w.sims[2].needs[0] = 1.0;
        w.tick_once();
        assert!(w.tick < limit, "{what}: Ada {:?}", w.sims[0].current().map(|a| a.task));
    }
}

/// With Ada at step `step` of the meal (at its station), how many minutes of it are left.
fn cooking(w: &World, step: u8) -> Option<f32> {
    w.sims[0].current().and_then(|a| match (a.task.kind, &a.phase) {
        (TaskKind::Cook { step: s, station: Some(_), minutes, .. }, Phase::Using { elapsed }) if s == step => {
            Some(f32::from(minutes) - elapsed)
        }
        _ => None,
    })
}

#[test]
fn a_parent_eating_at_night_leaves_her_plate_for_the_crying_baby_and_finishes_it_after() {
    let mut w = night_owl_in(&night_kitchen_content(SNACK), KITCHEN, "", 1.0);
    // Hungry: Ada leaves the television for a snack and sits down to it at the table.
    w.sims[0].needs[0] = 0.1;
    w.sims[0].needs[2] = 1.0;
    let at_table = |w: &World| {
        w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Eat { plate: Some(_), .. }) && matches!(a.phase, Phase::Using { .. }))
    };
    until(&mut w, "Ada sits down to a snack", at_table);
    minutes(&mut w, 5);
    assert!(at_table(&w), "still eating: {:?}", w.sims[0].current().map(|a| a.task));
    let plate = w.items.list.iter().find(|it| it.edible()).expect("a plate on the table").id;
    // Bea cries: Ada leaves her plate on the table and feeds her; Ben sleeps on.
    w.sims[2].needs[0] = 0.25;
    let start = w.tick;
    let (mut fed_at, mut left) = (None, true);
    night_until(&mut w, 4.0, None, |w| {
        assert!(w.sims[1].asleep(&w.content), "Ben woken at tick {}", w.tick);
        // (The first feed: she feeds Bea again before morning.)
        if feeding(w, 0) && *fed_at.get_or_insert(w.tick) + 20 * 20 > w.tick {
            left &= w.items.get(plate).is_some_and(|it| it.edible() && matches!(it.at, ItemAt::Surface { .. }));
        }
    });
    let fed_at = fed_at.expect("Ada fed Bea");
    let waited = (fed_at - start) as f32 * sim_core::MINUTES_PER_TICK;
    assert!(waited < 5.0, "Ada saw to Bea promptly: {waited} minutes");
    assert!(left, "the snack waited on the table while she fed Bea");
    // And she came back to it after.
    assert!(w.items.get(plate).is_none_or(|it| !it.edible()), "the snack was eaten: {:?}", w.items.get(plate));
    assert!(w.sims[0].needs[0] > 0.5, "Ada isn't hungry now: {}", w.sims[0].needs[0]);
}

#[test]
fn a_parent_cooking_at_night_finishes_the_step_sees_to_the_crying_baby_and_goes_on_after() {
    let mut w = night_owl_in(&night_kitchen_content(MEAL), KITCHEN, "", 1.0);
    w.items.list.retain(|it| !it.edible());
    w.sims[0].needs[0] = 0.3;
    w.sims[0].needs[2] = 1.0;
    // Hungry: Ada makes a meal. Bea cries with a few minutes of chopping to go.
    until(&mut w, "Ada at the counter", |w| cooking(w, 1).is_some_and(|m| m <= 6.0));
    w.sims[2].needs[0] = 0.25;
    let start = w.tick;
    let (mut fed_at, mut stove_first) = (None, false);
    night_until(&mut w, 4.0, None, |w| {
        assert!(w.sims[1].asleep(&w.content), "Ben woken at tick {}", w.tick);
        if feeding(w, 0) {
            fed_at.get_or_insert(w.tick);
        }
        stove_first |= fed_at.is_none() && cooking(w, 2).is_some();
    });
    let fed_at = fed_at.expect("Ada fed Bea");
    let waited = (fed_at - start) as f32 * sim_core::MINUTES_PER_TICK;
    assert!(waited > 5.0 && waited < 10.0, "Ada finished chopping, then saw to Bea: {waited} minutes");
    assert!(!stove_first, "Bea before the stove");
    // Then back to the stove: the meal is made (and she has some).
    let made = w.items.list.iter().any(|it| w.content.items[it.def].id == "mealPot" || w.content.items[it.def].id == "potDirty");
    assert!(made && w.sims[0].needs[0] > 0.5, "the meal was made and eaten: hunger {}", w.sims[0].needs[0]);
}

#[test]
fn a_parent_with_long_to_go_on_the_stove_leaves_the_crying_baby_to_the_one_asleep() {
    let mut w = night_owl_in(&night_kitchen_content(MEAL), KITCHEN, "", 1.0);
    w.items.list.retain(|it| !it.edible());
    w.sims[0].needs[0] = 0.3;
    w.sims[0].needs[2] = 1.0;
    // Bea cries with most of the half hour on the stove to go: Ben is woken.
    until(&mut w, "Ada at the stove", |w| cooking(w, 2).is_some_and(|m| m >= 25.0));
    w.sims[2].needs[0] = 0.25;
    let (ben, ada) = first_to_feed(&mut w, 2.5);
    assert!(ben.is_some_and(|m| m < 5.0) && ada.is_none(), "Ben fed Bea promptly (Ben {ben:?}, Ada {ada:?} minutes on)");
}

#[test]
fn whose_turn_it_is_at_night_survives_a_reload() {
    let mut w = night();
    w.sims[2].needs[0] = 0.25;
    let mut first = None;
    night_until(&mut w, 1.5, None, |w| first = first.or((0..2).find(|&i| feeding(w, i))));
    let first = first.expect("someone fed Bea");
    let mut w = World::from_save_json(NIGHT, &w.save_json()).unwrap();
    // Crying again at once: she's quiet for her hour (from one o'clock) and then the other
    // parent gets up.
    w.sims[2].needs[0] = 0.25;
    let mut next = None;
    night_until(&mut w, 3.0, None, |w| next = next.or((0..2).find(|&i| feeding(w, i)).map(|i| (i, w.tick))));
    let (next, at) = next.expect("someone fed Bea again");
    let hour = sim_core::clock::minute_of_day(at) as f32 / 60.0;
    assert!(hour > 1.9, "Bea's quiet spell carried over: fed again at {hour:.2}");
    assert_ne!(next, first, "turn and turn about after the reload");
}

/// On until `hour` (`night_until`): how many minutes from now Ben, and Ada, first fed Bea.
fn first_to_feed(w: &mut World, hour: f32) -> (Option<f32>, Option<f32>) {
    let start = w.tick;
    let mut first = [None; 2];
    night_until(w, hour, None, |w| {
        for (i, first) in first.iter_mut().enumerate() {
            if first.is_none() && feeding(w, i) {
                *first = Some((w.tick - start) as f32 * sim_core::MINUTES_PER_TICK);
            }
        }
    });
    (first[1], first[0])
}

/// The shipped content: a home with a crib, and what a grown-up needs for themselves.
const HOME: &str = r#"{"width":16,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":10,"entry":[8.5,9.5]}],
    "households":[{"name":"Home","plot":0,"player":true}],
    "objects":[{"def":"crib","x":1,"z":5,"rot":0},{"def":"toilet","x":5,"z":1,"rot":0},{"def":"sink","x":7,"z":1,"rot":0},
               {"def":"shower","x":9,"z":1,"rot":0},{"def":"fridge","x":13,"z":1,"rot":0},{"def":"bed","x":1,"z":1,"rot":0},
               {"def":"sofa","x":10,"z":6,"rot":0},{"def":"tv","x":10,"z":4,"rot":0}],
    "sims":[{"name":"Ada","traits":["kind"],"x":6.5,"z":6.5,"age":30},{"name":"Bea","traits":["cheerful"],"x":5.5,"z":6.5,"age":0.5}],
    "relationships":[{"a":1,"b":0,"preset":"parent"}]}"#;

#[test]
fn a_baby_with_a_grown_up_at_home_is_cuddled_and_stays_comfortable() {
    let mut w = World::from_json(&common::content(), HOME, 1).unwrap();
    let comfort = w.content.needs.iter().position(|n| n.id == "comfort").unwrap();
    let cuddle = w.content.objects[w.objects[0].def].interactions.iter().position(|i| i.id == "cuddle").unwrap();
    // Two days, Ada at home (no job): she sees to Bea, cuddling her when she needs it.
    let (mut sum, mut lowest, mut n, mut cuddled) = (0.0, 1.0f32, 0, false);
    for _ in 0..2 * 24 * 4 {
        for _ in 0..15 * 20 {
            w.tick_once();
            cuddled |= w.sims[0].current().is_some_and(|a| matches!(a.task.kind, sim_core::world::TaskKind::Use { object: 0, interaction } if interaction == cuddle));
        }
        sum += w.sims[1].needs[comfort];
        lowest = lowest.min(w.sims[1].needs[comfort]);
        n += 1;
    }
    let average = sum / n as f32;
    assert!(w.objects[0].users().any(|u| u == 1), "Bea is in her crib");
    assert!(cuddled, "Ada cuddled Bea");
    assert!(average > 0.35 && lowest > 0.0, "Bea stayed comfortable: average {average:.2}, lowest {lowest:.2}");
}

/// Bea lying in the crib (asleep or awake) and Ada feeding her, saved and loaded: Bea lies back in
/// the crib, not on the changing table beside it, though Ada (who comes first) sets off for it
/// before Bea is put back in. The crib turned every way.
#[test]
fn a_baby_loaded_from_a_save_lies_back_in_the_crib() {
    use sim_core::snapshot::{self, HEADER_LEN, SIM_STRIDE, sim};
    // Where Bea is and which way she faces, in the sim and as the renderer gets it, and her slot.
    let bea = |w: &World| {
        let mut out = vec![0.0; snapshot::CAPACITY];
        snapshot::write(w, &mut out);
        let row = &out[HEADER_LEN + SIM_STRIDE..];
        let s = &w.sims[1];
        let slot = w.objects[0].users.iter().position(|u| *u == Some(1));
        (s.pos, s.yaw, s.pose, slot, [row[sim::X], row[sim::Z], row[sim::YAW], row[sim::POSE]])
    };
    for rot in 0..4 {
        let town = TOWN.replace(r#"{"def":"crib","x":2,"z":2}"#, &format!(r#"{{"def":"crib","x":2,"z":2,"rot":{rot}}}"#));
        for awake in [false, true] {
            let mut w = World::from_json(CONTENT, &town, 1).unwrap();
            minutes(&mut w, 1);
            assert_eq!(w.sims[1].pose, sim_core::content::Pose::Lie, "rot {rot}: Bea is in the crib");
            if awake {
                w.sims[1].needs[0] = 0.1;
            }
            w.apply(Command::Use { sim: 0, object: 0, interaction: 1 }).unwrap();
            minutes(&mut w, 1);
            let before = bea(&w);
            assert_eq!(before.3, Some(0), "rot {rot}: Bea lies in the crib's slot");
            let mut loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
            minutes(&mut loaded, 1);
            assert_eq!(bea(&loaded), before, "rot {rot}, awake {awake}: Bea lies where she lay before the save");
        }
    }
}
