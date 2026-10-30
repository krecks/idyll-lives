//! Retirement: at content `life.retireAt` a working resident retires with a weekly pension of
//! `life.pension` × their weekly pay, and at least `life.basicPension`. Anyone at or past the age
//! without a job is retired (with the basic pension), and the job market doesn't hire them; a job
//! the player gives them is theirs to keep.

use crate::common;

use sim_core::social::EventKind;
use sim_core::{Command, World, clock};

/// A clerk earns 100 a week (40 %: 40, below the basic pension of 150), a manager 1,000 (40 %:
/// 400). A year a day.
const CONTENT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.0}],
    "objects":[],
    "feelings":[{"id":"retired","label":"Retired","mood":0.06,"hours":48}],
    "careers":[{"id":"shop","label":"Shop","levels":[
        {"title":"Clerk","pay":20,"start":9,"hours":4,"days":[0,1,2,3,4]},
        {"title":"Manager","pay":200,"start":9,"hours":4,"days":[0,1,2,3,4]}]}],
    "careerRules":{"market":{"hour":10,"dailyChance":1.0,"minShiftsBeforeQuit":99,"quitBelow":0}},
    "life":{"daysPerYear":1,"stages":[{"id":"adult","label":"Adult","from":18},{"id":"elder","label":"Elder","from":60}],
            "retireAt":65,"pension":0.4,"basicPension":150,"retireFeeling":"retired"},
    "economy":{"startingFunds":1000}}"#;

/// One home with a resident for each of `sims` (their JSON keys besides a name and a place).
fn town(sims: &[&str]) -> String {
    let sims: Vec<String> = sims
        .iter()
        .enumerate()
        .map(|(i, s)| format!(r#"{{"name":"R{i}","x":{}.5,"z":3.5,{s}}}"#, 2 + i))
        .collect();
    format!(
        r#"{{"width":12,"depth":10,
        "plots":[{{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]}}],
        "households":[{{"name":"Home","plot":0,"player":true}}],
        "sims":[{}]}}"#,
        sims.join(",")
    )
}

fn world(sims: &[&str]) -> World {
    let mut w = World::from_json(CONTENT, &town(sims), 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w
}

fn days(w: &mut World, d: u32) {
    for _ in 0..d * 24 * 60 * 20 {
        w.tick_once();
    }
}

fn until(w: &mut World, day: u32, hour: f32) {
    let end = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < end {
        w.tick_once();
    }
}

fn feels_retired(w: &World, i: usize) -> bool {
    w.sims[i]
        .feelings
        .iter()
        .any(|f| w.content.feelings[f.def].id == "retired")
}

fn retired_from_work(w: &World, i: u32) -> bool {
    w.events
        .iter()
        .any(|e| e.kind == EventKind::Retired && e.a == i)
}

#[test]
fn an_entry_level_worker_retires_with_at_least_the_basic_pension() {
    let mut w = world(&[r#""age":64.5,"job":{"career":"shop","level":0}"#]);
    days(&mut w, 1);
    assert!(w.sims[0].retired && w.sims[0].job.is_none(), "65: retired");
    assert_eq!(
        w.sims[0].pension, 150,
        "the basic pension: 40 % of 100 is less"
    );
    assert!(retired_from_work(&w, 0));
}

#[test]
fn a_high_earner_retires_with_their_share_of_their_pay() {
    let mut w = world(&[r#""age":64.5,"job":{"career":"shop","level":1}"#]);
    days(&mut w, 1);
    assert!(w.sims[0].retired && w.sims[0].job.is_none(), "65: retired");
    assert_eq!(
        w.sims[0].pension, 400,
        "40 % of 1,000, more than the basic pension"
    );
}

#[test]
fn elders_start_out_retired() {
    // Made at 70 in the household creator (no job), placed at 70 in a job, and 40.
    let mut w = world(&[
        r#""age":70"#,
        r#""age":70,"job":{"career":"shop","level":1}"#,
        r#""age":40"#,
    ]);
    assert!(w.sims[0].retired && w.sims[0].job.is_none());
    assert_eq!(
        w.sims[0].pension, 150,
        "the basic pension: no job to have a share of"
    );
    assert!(w.sims[1].retired && w.sims[1].job.is_none());
    assert_eq!(w.sims[1].pension, 400, "40 % of the job they were given");
    assert_eq!((w.sims[2].retired, w.sims[2].pension), (false, 0));
    // Moving in at 68 too.
    let spawn = serde_json::from_str(r#"{"name":"Mo","x":8.5,"z":3.5,"age":68}"#).unwrap();
    w.apply(Command::MoveIn {
        household: 0,
        name: None,
        sims: vec![spawn],
        bonds: vec![],
    })
    .unwrap();
    assert_eq!((w.sims[3].retired, w.sims[3].pension), (true, 150));
    // Nothing to tell: they were retired already. Saved and loaded with their pensions.
    days(&mut w, 2);
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Retired));
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    let pensions: Vec<_> = loaded.sims.iter().map(|s| (s.retired, s.pension)).collect();
    assert_eq!(
        pensions,
        [(true, 150), (true, 400), (false, 0), (true, 150)]
    );
}

#[test]
fn an_unemployed_resident_retires_at_the_age() {
    let mut w = world(&[r#""age":64.5"#]);
    assert!(!w.sims[0].retired);
    days(&mut w, 1);
    assert!(w.sims[0].retired && w.sims[0].age >= 65.0, "65: retired");
    assert_eq!(w.sims[0].pension, 150, "the basic pension");
    assert!(feels_retired(&w, 0));
    assert!(
        !retired_from_work(&w, 0),
        "the story tells of leaving a job; there was none"
    );
}

#[test]
fn the_job_market_doesnt_hire_from_the_retirement_age() {
    let mut w = World::from_json(CONTENT, &town(&[r#""age":70"#, r#""age":40"#]), 1).unwrap();
    // Short of money: they'd look twice as hard.
    w.households[0].funds = 0;
    // Back to work and out again before the job market opens: 70, out of work, not retired.
    w.apply(Command::JoinCareer {
        sim: 0,
        career: 0,
        level: 0,
    })
    .unwrap();
    w.apply(Command::QuitCareer { sim: 0 }).unwrap();
    assert!(!w.sims[0].retired && w.sims[0].job.is_none());
    until(&mut w, 1, 11.0);
    assert!(w.sims[1].job.is_some(), "the job market hires at 40");
    assert!(w.sims[0].job.is_none(), "not at 70");
    // At midnight they retire again, on the basic pension; no job comes after that either.
    until(&mut w, 2, 0.5);
    assert_eq!((w.sims[0].retired, w.sims[0].pension), (true, 150));
    until(&mut w, 8, 0.5);
    assert!(w.sims[0].job.is_none() && w.sims[0].retired);
    assert!(
        !w.events
            .iter()
            .any(|e| e.kind == EventKind::JobFound && e.a == 0)
    );
}

#[test]
fn a_job_the_player_gives_a_retiree_sticks() {
    let mut w = world(&[r#""age":70"#]);
    w.apply(Command::JoinCareer {
        sim: 0,
        career: 0,
        level: 0,
    })
    .unwrap();
    assert_eq!(
        (w.sims[0].retired, w.sims[0].pension),
        (false, 0),
        "back to work: no pension"
    );
    days(&mut w, 4);
    assert!(
        w.sims[0].job.is_some(),
        "still working four midnights later"
    );
    assert_eq!((w.sims[0].retired, w.sims[0].pension), (false, 0));
    assert!(!retired_from_work(&w, 0));
    // Quitting: retired again at midnight, on the basic pension (40 % of the job is less).
    w.apply(Command::QuitCareer { sim: 0 }).unwrap();
    days(&mut w, 1);
    assert_eq!((w.sims[0].retired, w.sims[0].pension), (true, 150));
}

/// The base game's basic pension pays a home of one's rent and bills: someone made at 70, alone
/// in the first house of a town, doesn't lose money over four weeks.
#[test]
fn the_basic_pension_pays_for_a_home_of_one() {
    // Four weeks age a resident by 14 years here (`life.daysPerYear` 2): no deaths, so this
    // checks the pension and the rent, not whether an old resident happens to survive.
    let mut content: serde_json::Value = serde_json::from_str(&common::content()).unwrap();
    content["life"]["death"] = serde_json::Value::Null;
    let content = content.to_string();
    let mut town: serde_json::Value = serde_json::from_str(&common::town(&content, 7, 1)).unwrap();
    // Just the player's household in town (faster), alone and 70.
    town["sims"]
        .as_array_mut()
        .unwrap()
        .retain(|s| s["household"] == 0);
    town["relationships"] = serde_json::json!([]);
    assert_eq!(town["sims"].as_array().unwrap().len(), 1);
    town["sims"][0]["age"] = 70.into();
    let mut w = World::from_json(&content, &town.to_string(), 7).unwrap();
    // (Nobody spends on anything else.)
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    assert!(w.sims[0].retired);
    common::dump_state(&w, "retired-alone");
    let pension = w.sims[0].pension;
    let (rent, bills) = sim_core::life::weekly_costs(&w, 0).unwrap();
    eprintln!(
        "basic pension {pension} a week; rent {rent} + bills {bills} = {}",
        rent + bills
    );
    assert_eq!(pension, w.content.life.basic_pension);
    assert!(pension >= rent + bills);
    let funds = w.households[0].funds;
    // Read week by week: the log keeps the town's last few hundred events, and a busy town
    // pushes the first weeks' out of it by day 29.
    let mut paid = std::collections::HashSet::new();
    for day in [8, 15, 22, 29] {
        until(&mut w, day, 0.0);
        paid.extend(
            w.events
                .iter()
                .filter(|e| e.kind == EventKind::PaidRent)
                .map(|e| e.id),
        );
    }
    assert_eq!(paid.len(), 4, "four weeks of rent and bills, all paid");
    assert!(
        w.households[0].funds >= funds,
        "{} after four weeks, from {funds}",
        w.households[0].funds
    );
}
