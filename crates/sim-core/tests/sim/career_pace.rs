//! How fast careers go on the shipped content: what a shift adds to the performance bar
//! (`careerRules`, `life::shift_performance`), when the promotions come, and what holds them
//! back. A decent worker is promoted after about four workdays at first, after one to two working
//! weeks at the top; a poor one takes clearly longer; missing skills wait however full the bar.

use crate::common;
use common::TICKS_PER_HOUR;
use serde_json::Value;
use sim_core::content::CareerRules;
use sim_core::{Command, Content, World, clock, life, social};

/// The career Mika was stuck in (journalism, digital media): Social Media Intern, grade A.
const CAREER: &str = "journalism.digital";

/// Shifts a worker in `mood` (0..1) with `fit` skill levels to spare works for a full bar.
fn shifts(rules: &CareerRules, grade: usize, mood: f32, fit: f32, late: bool) -> u32 {
    (100.0 / life::shift_performance(rules, grade, mood, fit, late)).ceil() as u32
}

#[test]
fn a_shift_is_worth_less_each_grade_up_and_in_a_bad_mood() {
    let content = Content::from_json(&common::content()).unwrap();
    let rules = &content.career_rules;
    // "Fine" (the Sim panel's word for a mood above 0.5), just the skills: the first promotion
    // within a working week; "Happy" (above 0.75, as most leave for work), sooner.
    let first = shifts(rules, 0, 0.6, 0.0, false);
    assert!(
        (3..=5).contains(&first),
        "first promotion after {first} shifts"
    );
    let happy = shifts(rules, 0, 0.85, 0.0, false);
    assert!((3..first).contains(&happy), "{happy} shifts happy");
    // Each grade up takes as long or longer; the top one a working week or two.
    let grades = content.careers[content.career_index(CAREER).unwrap()]
        .levels
        .len();
    let by_grade = |mood: f32| -> Vec<u32> {
        (0..grades - 1)
            .map(|g| shifts(rules, g, mood, 0.0, false))
            .collect()
    };
    let fine = by_grade(0.6);
    assert!(fine.windows(2).all(|w| w[0] <= w[1]), "{fine:?}");
    let top = *fine.last().unwrap();
    assert!((5..=10).contains(&top), "the top grade after {top} shifts");
    // A whole career, A to J, takes the better part of a working life (18 to 65: 94 days at two
    // days a year, 67 of them workdays).
    let career: u32 = by_grade(0.85).iter().sum();
    assert!((40..=67).contains(&career), "A to J in {career} shifts");
    // A poor worker, "Uncomfortable" (0.3 to 0.5): half as long again at least.
    let poor = shifts(rules, 0, 0.4, 0.0, false);
    assert!(
        poor * 2 >= first * 3,
        "{poor} shifts in a bad mood, {first} in a fine one"
    );
    // Late, a shift counts half; a missed one takes performance off, less of it further up.
    assert!(shifts(rules, 0, 0.6, 0.0, true) >= 2 * first - 1);
    assert_eq!(life::grade_pace(rules, 0), 1.0);
    assert!(life::grade_pace(rules, 8) < life::grade_pace(rules, 1));
    // Skills above the job's requirements help a little; short of them, it's slower.
    assert!(shifts(rules, 0, 0.6, -1.0, false) >= first);
    assert!(shifts(rules, 0, 0.6, 2.0, false) <= first);
}

/// A town on the shipped content, and the player's two residents hired at grade A of `CAREER`
/// with the skills grade B asks for.
fn hired() -> (World, [usize; 2]) {
    let content = common::content();
    let town = common::town(&content, 7, 2);
    let mut w = World::from_json(&content, &town, 7).unwrap();
    let player: Vec<usize> = (0..w.sims.len())
        .filter(|&i| w.households[w.sims[i].household as usize].player)
        .collect();
    let pair = [player[0], player[1]];
    let career = w.content.career_index(CAREER).unwrap();
    for &i in &pair {
        assert!(
            w.sims[i].adult(&w.content),
            "{} is grown up",
            w.sims[i].name
        );
        w.apply(Command::JoinCareer {
            sim: i as u32,
            career,
            level: 0,
        })
        .unwrap();
        for &(skill, level) in &w.content.careers[career].levels[1].requires.clone() {
            w.sims[i].skills[skill] = w.sims[i].skills[skill].max(level);
        }
    }
    (w, pair)
}

/// Keeps `i` unhappy: grieving and fed up, whatever else happens.
fn keep_unhappy(w: &mut World, i: usize) {
    let tick = w.tick;
    for id in ["grieving", "fedUp"] {
        let def = w.content.feeling_index(id).unwrap();
        social::add_feeling(&mut w.sims[i].feelings, def, &w.content.feelings, tick);
    }
}

#[test]
#[cfg_attr(not(feature = "slow-tests"), ignore = "slow: --features slow-tests")]
fn a_decent_worker_is_promoted_within_a_working_week_and_a_poor_one_later() {
    let (mut w, [decent, poor]) = hired();
    // Shifts worked when each got their first promotion.
    let mut promoted = [None, None];
    let end = clock::tick_at(22, 0.0).unwrap();
    while w.tick < end && promoted.iter().any(Option::is_none) {
        keep_unhappy(&mut w, poor);
        for _ in 0..TICKS_PER_HOUR {
            w.tick_once();
        }
        for (k, &i) in [decent, poor].iter().enumerate() {
            let job = w.sims[i].job.as_ref().expect("still employed");
            if promoted[k].is_none() && job.level > 0 {
                promoted[k] = Some(job.shifts);
            }
        }
    }
    let [Some(decent), Some(poor)] = promoted else {
        panic!("not both promoted in three weeks: {promoted:?}");
    };
    assert!(
        (3..=5).contains(&decent),
        "a decent worker was promoted after {decent} shifts"
    );
    assert!(
        poor >= decent + 2,
        "an unhappy one after {poor} shifts, {decent} for the decent one"
    );
}

/// The Career tab's view of resident `i`'s job.
fn job_view(w: &World, i: usize) -> Value {
    let ui: Value = serde_json::from_str(&sim_core::view::ui_state_json(w)).unwrap();
    let sim = ui["sims"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == i)
        .expect("in the UI state")
        .clone();
    sim["job"].clone()
}

#[test]
#[cfg_attr(not(feature = "slow-tests"), ignore = "slow: --features slow-tests")]
fn missing_skills_hold_a_promotion_back_and_the_career_tab_says_which() {
    let (mut w, [i, _]) = hired();
    // Grade D (Professional) with just its skills: grade E asks for a level more of each.
    let career = w.content.career_index(CAREER).unwrap();
    let job = w.sims[i].job.as_mut().unwrap();
    job.level = 3;
    job.performance = 90.0;
    for (skill, level) in w.content.careers[career].levels[3].requires.clone() {
        w.sims[i].skills[skill] = level;
    }
    let view = job_view(&w, i);
    assert!(view["shiftsToFullBar"].as_u64().unwrap() >= 1);
    assert!(view["shiftPerformance"].as_f64().unwrap() > 0.0);
    // Two workdays: the bar fills, but the skills aren't there yet (work trains them slowly).
    let start = clock::day(w.tick);
    let mut day = start + 1;
    let mut worked = 0;
    while worked < 2 {
        day += 1;
        common::run_to(&mut w, day, 0.0);
        worked = w.sims[i].job.as_ref().unwrap().shifts;
    }
    let job = w.sims[i].job.as_ref().unwrap();
    assert_eq!(job.level, 3, "not promoted without the skills");
    assert_eq!(job.performance, 100.0);
    let view = job_view(&w, i);
    assert_eq!(view["shiftsToFullBar"], 0);
    let short: Vec<&str> = view["next"]["requires"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["have"].as_f64() < r["level"].as_f64())
        .map(|r| r["skill"].as_str().unwrap())
        .collect();
    assert!(!short.is_empty(), "the tab names the skills still short");
    // Given them, the next shift promotes.
    for (skill, level) in w.content.careers[career].levels[4].requires.clone() {
        w.sims[i].skills[skill] = w.sims[i].skills[skill].max(level);
    }
    let before = worked;
    while w.sims[i].job.as_ref().unwrap().shifts == before {
        day += 1;
        common::run_to(&mut w, day, 0.0);
    }
    assert_eq!(
        w.sims[i].job.as_ref().unwrap().level,
        4,
        "promoted once skilled"
    );
}
