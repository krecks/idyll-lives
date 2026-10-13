//! Retirement: at content `life.retireAt` a working resident retires with a weekly pension of
//! `life.pension` × their weekly pay, and at least `life.basicPension`.

use sim_core::social::EventKind;
use sim_core::{Command, World};

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
    let sims: Vec<String> = sims.iter().enumerate().map(|(i, s)| format!(r#"{{"name":"R{i}","x":{}.5,"z":3.5,{s}}}"#, 2 + i)).collect();
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
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w
}

fn days(w: &mut World, d: u32) {
    for _ in 0..d * 24 * 60 * 20 {
        w.tick_once();
    }
}

fn retired_from_work(w: &World, i: u32) -> bool {
    w.events.iter().any(|e| e.kind == EventKind::Retired && e.a == i)
}

#[test]
fn an_entry_level_worker_retires_with_at_least_the_basic_pension() {
    let mut w = world(&[r#""age":64.5,"job":{"career":"shop","level":0}"#]);
    days(&mut w, 1);
    assert!(w.sims[0].retired && w.sims[0].job.is_none(), "65: retired");
    assert_eq!(w.sims[0].pension, 150, "the basic pension: 40 % of 100 is less");
    assert!(retired_from_work(&w, 0));
}

#[test]
fn a_high_earner_retires_with_their_share_of_their_pay() {
    let mut w = world(&[r#""age":64.5,"job":{"career":"shop","level":1}"#]);
    days(&mut w, 1);
    assert!(w.sims[0].retired && w.sims[0].job.is_none(), "65: retired");
    assert_eq!(w.sims[0].pension, 400, "40 % of 1,000, more than the basic pension");
}
