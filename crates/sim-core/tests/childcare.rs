//! Childcare: a baby is never left alone at home for a workday. When the last grown-up leaves
//! for work, the household books a babysitter for the day (content `life.childcare`), who feeds
//! and changes the baby while nobody's home; a grandparent at home looks after the baby instead,
//! for free; and without the money for a babysitter, the last grown-up stays home.

use sim_core::social::EventKind;
use sim_core::{World, clock};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1},{"id":"bladder","label":"Bladder","decayPerHour":0.12},
             {"id":"energy","label":"Energy","decayPerHour":0.02}],
    "objects":[{"id":"crib","name":"Crib","price":350,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"lie","label":"Lie in the crib","minutes":720,"pose":"lie","baby":true,"autonomous":false,"effects":{"energy":1.0},"tags":["rest"]},
        {"id":"feed","label":"Feed the baby","minutes":15,"care":{"hunger":0.8},"tags":["care"]},
        {"id":"change","label":"Change the baby","minutes":10,"care":{"bladder":1.0},"tags":["care"]}]},
      {"id":"fridge","name":"Fridge","price":100,"interactions":[
        {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.5},"tags":["food"]}]},
      {"id":"toilet","name":"Toilet","price":100,"interactions":[
        {"id":"use","label":"Use","minutes":5,"effects":{"bladder":1.0},"tags":["bathroom"]}]}],
    "skills":[{"id":"writing","label":"Writing"}],
    "grades":[{"id":"A","label":"Entry","payPerHour":10,"skillLevel":0}],
    "careerCategories":[{"id":"press","label":"Press","shift":{"start":9,"hours":8,"days":[0,1,2,3,4]},
      "tracks":[{"id":"press.print","label":"Print","skills":{"writing":1},"titles":["Runner"]}]}],
    "life":{"daysPerYear":2,"stages":[
        {"id":"baby","label":"Baby","from":0,"baby":true},
        {"id":"child","label":"Child","from":2,"school":true},
        {"id":"adult","label":"Adult","from":18},
        {"id":"elder","label":"Elder","from":60}],
        "childcare":{"cost":40,"below":0.45}},
    "bondPresets":{"partners":{"friendship":55,"romance":75,"partners":true},"parent":{"friendship":50,"kin":"parent"}},
    "economy":{"startingFunds":1000}}"#;

/// Ada and Ben both work 9 to 5; Cleo is their baby (and `extra` anyone else at home).
fn town(extra: &str) -> String {
    format!(
        r#"{{"width":12,"depth":12,
        "plots":[{{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[5.5,11.5]}}],
        "households":[{{"name":"Park","plot":0,"player":true}}],
        "objects":[{{"def":"crib","x":2,"z":2}},{{"def":"fridge","x":8,"z":2}},{{"def":"toilet","x":10,"z":6}}],
        "exits":[[5.5,11.9]],
        "sims":[{{"name":"Ada","x":4.5,"z":8.5,"age":30,"job":{{"career":"press.print","level":0}}}},
                {{"name":"Ben","x":5.5,"z":8.5,"age":32,"job":{{"career":"press.print","level":0}}}},
                {{"name":"Cleo","x":3.5,"z":8.5,"age":0.5}}{extra}],
        "relationships":[{{"a":0,"b":1,"preset":"partners"}},{{"a":2,"b":0,"preset":"parent"}},{{"a":2,"b":1,"preset":"parent"}}]}}"#
    )
}

fn until(w: &mut World, day: u32, hour: f32) {
    let end = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < end {
        w.tick_once();
    }
}

/// The baby's needs (the lowest and the average) each quarter hour from 10:00 to 16:00 on day 2 (a Tuesday).
fn workday(w: &mut World) -> (f32, f32) {
    until(w, 2, 10.0);
    let (mut lowest, mut sum, mut n) = (1.0f32, 0.0, 0);
    while clock::hour(w.tick) < 16.0 {
        for _ in 0..15 * 20 {
            w.tick_once();
        }
        for k in 0..2 {
            lowest = lowest.min(w.sims[2].needs[k]);
            sum += w.sims[2].needs[k];
            n += 1;
        }
    }
    (lowest, sum / n as f32)
}

/// Story events of `kind` on `day`.
fn told(w: &World, kind: EventKind, day: u32) -> Vec<&sim_core::social::SocialEvent> {
    w.events.iter().filter(|e| e.kind == kind && clock::day(e.tick) == day).collect()
}

#[test]
fn a_babysitter_looks_after_the_baby_while_both_parents_work() {
    let mut w = World::from_json(CONTENT, &town(""), 1).unwrap();
    until(&mut w, 2, 7.0);
    let funds = w.households[0].funds;
    until(&mut w, 2, 12.0);
    assert!(w.sims[0].away_until.is_some() && w.sims[1].away_until.is_some(), "both at work");
    let booked = told(&w, EventKind::Babysitter, 2);
    assert_eq!(booked.len(), 1, "a babysitter for the day");
    assert_eq!((booked[0].b, booked[0].n), (2, Some(40)), "for Cleo, for $40");
    assert_eq!(w.households[0].funds, funds - 40, "paid from the household");

    let (lowest, average) = workday(&mut w);
    assert!(average > 0.5 && lowest > 0.25, "Cleo was looked after: average {average:.2}, lowest {lowest:.2}");
    // Booked once for the day, and again the next workday.
    until(&mut w, 3, 12.0);
    assert_eq!(told(&w, EventKind::Babysitter, 2).len(), 1, "once a day");
    assert_eq!(told(&w, EventKind::Babysitter, 3).len(), 1, "and again the next workday");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::StayedHome || e.kind == EventKind::MissedWork));
}

#[test]
fn a_grandparent_at_home_looks_after_the_baby_for_free() {
    let grandma = r#",{"name":"Dora","x":6.5,"z":8.5,"age":70}"#;
    let mut w = World::from_json(CONTENT, &town(grandma), 1).unwrap();
    let (lowest, average) = workday(&mut w);
    assert!(w.sims[0].away_until.is_some() && w.sims[1].away_until.is_some(), "both at work");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Babysitter), "no babysitter with Dora at home");
    assert!(average > 0.5 && lowest > 0.25, "Dora looked after Cleo: average {average:.2}, lowest {lowest:.2}");
}

#[test]
fn without_money_for_a_babysitter_someone_stays_home() {
    let mut w = World::from_json(CONTENT, &town(""), 1).unwrap();
    until(&mut w, 2, 7.0);
    w.households[0].funds = 10;
    let (lowest, average) = workday(&mut w);
    assert!(told(&w, EventKind::Babysitter, 2).is_empty(), "no money for one");
    let stayed = told(&w, EventKind::StayedHome, 2);
    assert_eq!(stayed.len(), 1, "one of them stayed home");
    let (home, away) = if stayed[0].a == 0 { (0, 1) } else { (1, 0) };
    assert_eq!(stayed[0].b, 2, "to look after Cleo");
    assert!(w.sims[home].away_until.is_none() && w.sims[away].away_until.is_some(), "the other went to work");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::MissedWork), "not held against them");
    assert!(average > 0.5 && lowest > 0.25, "Cleo was looked after: average {average:.2}, lowest {lowest:.2}");
}
