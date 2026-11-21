//! A household's car fits its best earner's job (`cars/looks.rs`): the body a field's grade calls
//! for (`careerCategories[].car`) and how old it is (`grades[].carAge`), from the start, after a
//! promotion or a new job (bought once it stands idle, the story told; not if the job is given up
//! first), kept by a household with nobody at work, kept where a longer one wouldn't fit, and kept
//! in the save.

use crate::common;

use sim_core::content::CarAge;
use sim_core::social::EventKind;
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
    "skills":[{"id":"strength","label":"Strength"},{"id":"writing","label":"Writing"}],
    "skillRules":{"maxLevel":10,"workGainPerHour":0.05,"slowdown":0.25},
    "objects":[{"id":"rock","name":"Rock","interactions":[]}],
    "grades":[{"id":"A","label":"Entry","payPerHour":10,"skillLevel":0,"carAge":"old"},
              {"id":"B","label":"Junior","payPerHour":20,"skillLevel":2},
              {"id":"C","label":"Senior","payPerHour":40,"skillLevel":4,"carAge":"new"}],
    "carBodies":[{"id":"saloon","label":"saloon","len":4.4,"wid":1.78},
                 {"id":"hatchback","label":"hatchback","len":4.0,"wid":1.74},
                 {"id":"van","label":"van","len":5.0,"wid":1.98},
                 {"id":"executive","label":"executive saloon","len":4.9,"wid":1.86}],
    "careerCategories":[
      {"id":"press","label":"Press","shift":{"start":9,"hours":4,"days":[0,1,2,3,4]},
       "car":{"A":"hatchback","B":"saloon","C":"executive"},
       "tracks":[{"id":"press.print","label":"Print","skills":{"writing":1,"strength":0.5},"titles":["Runner","Reporter","Editor"]}]},
      {"id":"trade","label":"Trade","shift":{"start":9,"hours":4,"days":[0,1,2,3,4]},
       "car":{"A":"van"},
       "tracks":[{"id":"trade.joinery","label":"Joinery","skills":{"strength":1},"titles":["Apprentice","Joiner","Master"]},
                 {"id":"trade.office","label":"Office","skills":{"writing":1},"titles":["Clerk"],"car":{"A":"saloon"}}]}],
    "careerRules":{"performancePerShift":60,"performancePerFit":0},
    "economy":{"startingFunds":1000}}"#;

/// Two houses on a street along x (its middle at z = 9.5, the ways out at both ends), each with a
/// driveway beside it and a saloon on it, as the town parked them: Ada (an editor, the best earner)
/// and Bo (a runner) in the first, Cy (out of work) in the second.
fn town(cy_job: &str, objects: &str) -> String {
    format!(
        r#"{{"width":40,"depth":14,"exits":[[0.5,9.5],[39.5,9.5]],
        "plots":[{{"name":"Home","x":0,"z":0,"w":20,"d":8,"entry":[5.5,7.5]}},
                 {{"name":"Next","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}}],
        "households":[{{"name":"Ones","plot":0}},{{"name":"Twos","plot":1}}],
        "objects":[{objects}],
        "sims":[{{"name":"Ada","household":0,"job":{{"career":"press.print","level":2}},"x":5.5,"z":3.5}},
                {{"name":"Bo","household":0,"job":{{"career":"press.print"}},"x":6.5,"z":3.5}},
                {{"name":"Cy","household":1,{cy_job}"x":25.5,"z":3.5,"skills":{{"writing":2,"strength":1}}}}],
        "drives":[{{"plot":0,"x":16.0,"z":0.55,"front":1.0,"street":9.5}},
                  {{"plot":1,"x":36.0,"z":0.55,"front":1.0,"street":9.5}}],
        "cars":[{{"plot":0,"x":16.0,"z":3.2,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.78,"body":0,"paint":2}},
                {{"plot":1,"x":36.0,"z":3.2,"yaw":3.1416,"street":9.5,"len":4.4,"wid":1.78,"body":0,"paint":5}}]}}"#
    )
}

fn world(cy_job: &str, objects: &str) -> World {
    World::from_json(CONTENT, &town(cy_job, objects), 1).unwrap()
}

/// The car's inner end is `PARK_GAP` (0.45 m) from its driveway's inner end, whatever its length.
fn parked_from_inner_end(w: &World, c: usize) -> f32 {
    let car = &w.cars[c];
    car.spot[1] - car.len / 2.0 - 0.55
}

#[test]
fn a_new_towns_cars_fit_their_best_earners_jobs() {
    let w = world("", "");
    let ada = &w.cars[0];
    assert_eq!(
        (ada.body, ada.age),
        (3, CarAge::New),
        "Ada edits: a new executive saloon"
    );
    assert_eq!((ada.len, ada.wid), (4.9, 1.86), "its size the body's");
    assert!((parked_from_inner_end(&w, 0) - 0.45).abs() < 1e-4);
    assert_eq!(ada.paint, 2, "the town's paint");
    let cy = &w.cars[1];
    assert_eq!(
        (cy.body, cy.age, cy.len),
        (0, CarAge::Used, 4.4),
        "nobody at work next door: the town's car"
    );
    assert!(w.cars.iter().all(|c| c.want.is_none()));
    // The renderer's look.
    let s: serde_json::Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    assert_eq!(s["cars"][0]["body"], 3);
    assert_eq!(s["cars"][0]["age"], 2);
    assert_eq!(s["cars"][1]["age"], 1);
}

#[test]
fn a_new_job_buys_the_car_it_calls_for_and_out_of_work_they_keep_it() {
    let mut w = world("", "");
    w.autonomy = false;
    w.apply(Command::JoinCareer {
        sim: 2,
        career: 1,
        level: 0,
    })
    .unwrap();
    // (Parked with nobody in it: bought at once.)
    w.tick_once();
    let car = &w.cars[1];
    assert_eq!(
        (car.body, car.age),
        (2, CarAge::Old),
        "an old van for an apprentice joiner"
    );
    assert_eq!((car.len, car.wid), (5.0, 1.98));
    assert!((parked_from_inner_end(&w, 1) - 0.45).abs() < 1e-4);
    let told = w
        .events
        .iter()
        .find(|e| e.kind == EventKind::BoughtCar)
        .expect("the story tells of it");
    assert_eq!((told.a, told.n), (2, Some(2)));
    // The van's tiles are walked round, the saloon's that stuck out less freed.
    assert!(
        w.parked_tiles().contains(&(36, 5)),
        "{:?}",
        w.parked_tiles()
    );
    w.apply(Command::QuitCareer { sim: 2 }).unwrap();
    w.tick_once();
    assert_eq!(w.cars[1].body, 2, "out of work: the van stays");
    // A job whose car is no better than the one they have: the same body, older.
    w.cars[1].age = CarAge::New;
    w.apply(Command::JoinCareer {
        sim: 2,
        career: 1,
        level: 0,
    })
    .unwrap();
    w.tick_once();
    assert_eq!(
        (w.cars[1].body, w.cars[1].age),
        (2, CarAge::New),
        "a new van kept"
    );
    // Another field's: the car it calls for.
    w.apply(Command::JoinCareer {
        sim: 2,
        career: 2,
        level: 0,
    })
    .unwrap();
    w.tick_once();
    assert_eq!((w.cars[1].body, w.cars[1].age), (0, CarAge::Old));
}

#[test]
fn a_car_called_for_by_a_job_given_up_before_it_is_bought_is_not_bought() {
    let mut w = world("", "");
    w.autonomy = false;
    w.apply(Command::JoinCareer {
        sim: 2,
        career: 1,
        level: 0,
    })
    .unwrap();
    assert!(w.cars[1].want.is_some(), "a van wanted");
    // (Quit before the car stood idle for it to be bought.)
    w.apply(Command::QuitCareer { sim: 2 }).unwrap();
    assert!(w.cars[1].want.is_none());
    w.tick_once();
    assert_eq!(w.cars[1].body, 0, "the saloon kept");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::BoughtCar));
}

#[test]
fn a_promotion_buys_a_better_car_on_the_way_home_from_work() {
    let mut w = world(r#""job":{"career":"press.print"},"#, "");
    w.autonomy = false;
    assert_eq!(
        (w.cars[1].body, w.cars[1].age),
        (1, CarAge::Old),
        "a runner's old hatchback"
    );
    // Cy has the skills for the next grade; two shifts fill the bar.
    let mut away = false;
    let mut bought_away = false;
    while w.tick < sim_core::clock::tick_at(3, 15.0 * 60.0).unwrap() {
        w.tick_once();
        let car = &w.cars[1];
        away |= car.state == sim_core::cars::CarState::Away;
        if car.body == 0 && !bought_away {
            bought_away = car.state == sim_core::cars::CarState::Away;
            assert!(bought_away, "bought while it was away: {:?}", car.state);
        }
    }
    assert!(away, "Cy drove to work");
    assert_eq!(w.sims[2].job.as_ref().unwrap().level, 1, "promoted");
    assert_eq!(
        (w.cars[1].body, w.cars[1].age),
        (0, CarAge::Used),
        "a reporter's saloon"
    );
    assert_eq!(
        w.cars[1].state,
        sim_core::cars::CarState::Parked,
        "home in it"
    );
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::BoughtCar && e.a == 2 && e.n == Some(0))
    );
}

#[test]
fn a_longer_car_that_would_not_fit_where_it_parks_is_not_bought() {
    // A rock where the end of a 4.9 m car would be, clear of the 4.4 m one there.
    let w = world("", r#"{"def":"rock","x":16,"z":5,"rot":0}"#);
    assert_eq!(w.cars[0].body, 0, "the town's saloon kept");
    assert!(!w.cars[0].blocked);
}

#[test]
fn the_cars_look_and_the_one_wanted_are_saved() {
    let mut w = world("", "");
    w.autonomy = false;
    w.cars[1].want = Some(sim_core::cars::Want {
        body: 2,
        age: CarAge::Old,
        by: Some(2),
    });
    let loaded = common::reload(&w, CONTENT);
    assert_eq!((loaded.cars[0].body, loaded.cars[0].age), (3, CarAge::New));
    assert_eq!(loaded.cars[1].want, w.cars[1].want);
}
