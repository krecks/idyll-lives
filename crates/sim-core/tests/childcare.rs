//! Childcare: a baby is never left alone at home for a workday. When the last grown-up leaves
//! for work, the household books a babysitter for the day (content `life.childcare`), who feeds
//! and changes the baby while nobody's home; a grandparent at home looks after the baby instead,
//! for free; and without the money for a babysitter, the last grown-up stays home. Nor does the
//! last grown-up at home go out on their own: sent out by the player, a babysitter comes too.

use sim_core::social::EventKind;
use sim_core::{Command, World, clock};

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

// ---- Going out ----------------------------------------------------------------------------
//
// The last grown-up at home with a baby doesn't go out on their own (to friends, or to the
// park); sent out by the player (visiting, to the park), a babysitter comes, and without the
// money for one they can't be sent.

const OUT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1},{"id":"bladder","label":"Bladder","decayPerHour":0.12},
             {"id":"energy","label":"Energy","decayPerHour":0.02},{"id":"social","label":"Social","decayPerHour":0.15}],
    "objects":[{"id":"crib","name":"Crib","price":350,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"lie","label":"Lie in the crib","minutes":720,"pose":"lie","baby":true,"autonomous":false,"effects":{"energy":1.0},"tags":["rest"]},
        {"id":"feed","label":"Feed the baby","minutes":15,"care":{"hunger":0.8},"tags":["care"]},
        {"id":"change","label":"Change the baby","minutes":10,"care":{"bladder":1.0},"tags":["care"]}]},
      {"id":"fridge","name":"Fridge","price":100,"interactions":[
        {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.5},"tags":["food"]}]},
      {"id":"toilet","name":"Toilet","price":100,"interactions":[
        {"id":"use","label":"Use","minutes":5,"effects":{"bladder":1.0},"tags":["bathroom"]}]},
      {"id":"bench","name":"Bench","price":100,"interactions":[
        {"id":"watch","label":"Watch people go by","minutes":60,"effects":{"social":0.6},"tags":["lounge"]}]}],
    "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},"needs":{"social":0.3}}],
    "visits":{"needs":{"social":0.8},"hours":2,"minFriendship":10,"earliestHour":9,"latestHour":21},
    "phone":{"seatRange":0,"actions":[
      {"id":"askOver","label":"Ask to come over","minutes":5,"contact":true,"meet":"there","autonomous":false,"tags":["social"],"acceptance":{"base":1.0}}]},
    "life":{"daysPerYear":2,"stages":[
        {"id":"baby","label":"Baby","from":0,"baby":true},
        {"id":"child","label":"Child","from":2,"school":true},
        {"id":"adult","label":"Adult","from":18},
        {"id":"elder","label":"Elder","from":60}],
        "childcare":{"cost":40,"below":0.45}},
    "bondPresets":{"friends":{"friendship":70},"parent":{"friendship":50,"kin":"parent"}},
    "economy":{"startingFunds":1000}}"#;

/// Ada (no job) at home with baby Cleo (`baby`; else Cleo's grown up) and `extra` anyone else;
/// the park next door (a bench, the only thing for company bar visiting) and her friend Bo's
/// house beyond it.
fn street(baby: bool, extra: &str) -> String {
    let age = if baby { 0.5 } else { 30.0 };
    format!(
        r#"{{"width":36,"depth":12,
        "plots":[{{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[5.5,9.5]}},
                 {{"name":"Park","x":12,"z":0,"w":12,"d":10,"entry":[17.5,9.5],"public":true}},
                 {{"name":"Bo's","x":24,"z":0,"w":12,"d":10,"entry":[29.5,9.5]}}],
        "households":[{{"name":"Park","plot":0,"player":true}},{{"name":"Bo","plot":2}}],
        "objects":[{{"def":"crib","x":2,"z":2}},{{"def":"fridge","x":8,"z":2}},{{"def":"toilet","x":10,"z":6}},
                   {{"def":"bench","x":17,"z":4}},{{"def":"fridge","x":26,"z":2}},{{"def":"toilet","x":34,"z":6}}],
        "sims":[{{"name":"Ada","household":0,"x":4.5,"z":8.5,"age":30}},
                {{"name":"Cleo","household":0,"x":3.5,"z":8.5,"age":{age}}},
                {{"name":"Bo","household":1,"x":28.5,"z":5.5,"age":31}}{extra}],
        "relationships":[{{"a":0,"b":2,"preset":"friends"}},{{"a":1,"b":0,"preset":"parent"}}]}}"#
    )
}

/// The street, with Bo staying in (always home for a visit, and not coming over himself).
fn street_world(baby: bool, extra: &str) -> World {
    let mut w = World::from_json(OUT, &street(baby, extra), 1).unwrap();
    w.households[1].free_will = false;
    w
}

/// Whether `i` is somewhere other than home (plot 0), or on a visit.
fn out(w: &World, i: usize) -> bool {
    let (x, z) = w.sims[i].tile();
    w.sims[i].visiting.is_some() || w.plot_at(x, z) != Some(0)
}

/// From 9:00 to 21:00 on day 1, lonely: how many quarter hours Ada was out.
fn lonely_day(w: &mut World) -> usize {
    until(w, 1, 9.0);
    let mut away = 0;
    while clock::hour(w.tick) < 21.0 {
        w.sims[0].needs[3] = w.sims[0].needs[3].min(0.3);
        for _ in 0..15 * 20 {
            w.tick_once();
        }
        away += usize::from(out(w, 0));
    }
    away
}

/// From 10:00 to 14:00 on day 1, the baby's needs each quarter hour (the lowest and the
/// average), and whether Ada was at Bo's.
fn while_out(w: &mut World) -> (f32, f32, bool) {
    let (mut lowest, mut sum, mut n, mut visited) = (1.0f32, 0.0, 0, false);
    while clock::hour(w.tick) < 14.0 {
        for _ in 0..15 * 20 {
            w.tick_once();
        }
        visited |= w.sims[0].visiting.is_some_and(|v| v.plot == 2);
        for k in 0..2 {
            lowest = lowest.min(w.sims[1].needs[k]);
            sum += w.sims[1].needs[k];
            n += 1;
        }
    }
    (lowest, sum / n as f32, visited)
}

#[test]
fn the_only_grown_up_at_home_with_a_baby_does_not_go_out() {
    // Without a baby to look after, lonely Ada goes out: to Bo's or the park.
    let mut w = street_world(false, "");
    assert!(lonely_day(&mut w) > 0, "without a baby, Ada goes out");

    let mut w = street_world(true, "");
    assert_eq!(lonely_day(&mut w), 0, "Ada stays home with Cleo");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Babysitter), "nobody needed");
}

#[test]
fn sent_out_visiting_the_last_grown_up_gets_a_babysitter() {
    let mut w = street_world(true, "");
    until(&mut w, 1, 10.0);
    let funds = w.households[0].funds;
    // Cleo is getting hungry and wet: she'll need looking after while Ada's out.
    w.sims[1].needs[0] = 0.5;
    w.sims[1].needs[1] = 0.5;
    w.apply(Command::Visit { sim: 0, plot: 2 }).unwrap();
    let (lowest, average, visited) = while_out(&mut w);
    assert!(visited, "Ada went to Bo's");
    let booked = told(&w, EventKind::Babysitter, 1);
    assert_eq!(booked.len(), 1, "a babysitter for the day");
    assert_eq!((booked[0].a, booked[0].b, booked[0].n), (0, 1, Some(40)), "for Cleo, for $40");
    assert_eq!(w.households[0].funds, funds - 40, "paid from the household");
    assert!(average > 0.5 && lowest > 0.25, "Cleo was looked after: average {average:.2}, lowest {lowest:.2}");

    // Out again the same day: still the one booking.
    until(&mut w, 1, 15.0);
    assert!(!out(&w, 0), "back home");
    w.apply(Command::Visit { sim: 0, plot: 2 }).unwrap();
    until(&mut w, 1, 17.0);
    assert!(w.sims[0].visiting.is_some(), "out again");
    assert_eq!(told(&w, EventKind::Babysitter, 1).len(), 1, "once a day");
}

#[test]
fn without_money_for_a_babysitter_the_last_grown_up_cannot_be_sent_out() {
    let mut w = street_world(true, "");
    until(&mut w, 1, 10.0);
    w.households[0].funds = 10;
    let err = w.apply(Command::Visit { sim: 0, plot: 2 }).unwrap_err();
    assert!(err.to_string().contains("Cleo"), "told why: {err}");
    until(&mut w, 1, 12.0);
    assert!(!out(&w, 0), "Ada stayed home");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Babysitter));
}

#[test]
fn with_a_grandparent_at_home_nobody_books_a_babysitter_for_a_visit() {
    let grandma = r#",{"name":"Dora","household":0,"x":6.5,"z":8.5,"age":70}"#;
    let mut w = street_world(true, grandma);
    until(&mut w, 1, 10.0);
    w.sims[1].needs[0] = 0.5;
    w.sims[1].needs[1] = 0.5;
    w.apply(Command::Visit { sim: 0, plot: 2 }).unwrap();
    let (lowest, average, visited) = while_out(&mut w);
    assert!(visited, "Ada went to Bo's");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Babysitter), "no babysitter with Dora at home");
    assert!(average > 0.5 && lowest > 0.25, "Dora looked after Cleo: average {average:.2}, lowest {lowest:.2}");
}

/// From now until `hour` on day 1: whether Ada was ever in the park (plot 1).
fn at_the_park_until(w: &mut World, hour: f32) -> bool {
    let mut seen = false;
    while clock::hour(w.tick) < hour {
        for _ in 0..5 * 20 {
            w.tick_once();
        }
        let (x, z) = w.sims[0].tile();
        seen |= w.plot_at(x, z) == Some(1);
    }
    seen
}

#[test]
fn without_money_for_a_babysitter_the_last_grown_up_cannot_be_sent_anywhere_else() {
    let mut w = street_world(true, "");
    until(&mut w, 1, 10.0);
    w.households[0].funds = 10;
    let refused = |err: sim_core::Error| assert_eq!(err.to_string(), "nobody to look after Cleo: not enough money for a babysitter ($40)");
    // The bench in the park, a walk to the park, a chat with Bo at his, asking Bo on the phone
    // to come over to his.
    refused(w.apply(Command::Use { sim: 0, object: 3, interaction: 0 }).unwrap_err());
    refused(w.apply(Command::MoveTo { sim: 0, x: 17.5, z: 6.5 }).unwrap_err());
    refused(w.apply(Command::Social { sim: 0, target: 2, social: 0 }).unwrap_err());
    refused(w.apply(Command::Phone { sim: 0, action: 0, contact: Some(2) }).unwrap_err());
    assert!(!at_the_park_until(&mut w, 12.0) && !out(&w, 0), "Ada stayed home");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Babysitter));
    // Things at home are fine.
    w.apply(Command::MoveTo { sim: 0, x: 6.5, z: 6.5 }).unwrap();
    w.apply(Command::Use { sim: 0, object: 1, interaction: 0 }).unwrap();
}

#[test]
fn sent_to_the_park_the_last_grown_up_gets_a_babysitter() {
    let mut w = street_world(true, "");
    until(&mut w, 1, 10.0);
    w.households[0].funds = 500;
    w.apply(Command::Use { sim: 0, object: 3, interaction: 0 }).unwrap();
    assert!(at_the_park_until(&mut w, 12.0), "Ada went to the park");
    let booked = told(&w, EventKind::Babysitter, 1);
    assert_eq!(booked.len(), 1, "a babysitter for the day");
    assert_eq!((booked[0].a, booked[0].b, booked[0].n), (0, 1, Some(40)), "for Cleo, for $40");
    assert_eq!(w.households[0].funds, 460, "paid from the household");
}

#[test]
fn with_a_grandparent_at_home_the_last_parent_goes_to_the_park_without_a_babysitter() {
    let grandma = r#",{"name":"Dora","household":0,"x":6.5,"z":8.5,"age":70}"#;
    let mut w = street_world(true, grandma);
    until(&mut w, 1, 10.0);
    w.households[0].funds = 10;
    w.apply(Command::Use { sim: 0, object: 3, interaction: 0 }).unwrap();
    assert!(at_the_park_until(&mut w, 12.0), "Ada went to the park");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Babysitter), "no babysitter with Dora at home");
}
