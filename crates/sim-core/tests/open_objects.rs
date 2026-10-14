//! Things that open and close (content `when` and `leaves`): a wall bed folds down for the night
//! and back up in the morning, or whenever someone is told to; it stays as it was left, saved.

mod common;

use sim_core::world::TaskKind;
use sim_core::{Command, TICKS_PER_MINUTE, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.01}],
    "dayRhythm":{"sleepTags":["sleep"]},
    "objects":[{"id":"wallBed","name":"Wall bed","price":900,"footprint":[2,3],"slots":2,"interactions":[
        {"id":"sleep","label":"Sleep","minutes":60,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"],"leaves":"open"},
        {"id":"foldDown","label":"Fold down","minutes":2,"autonomous":false,"when":"closed","leaves":"open"},
        {"id":"foldUp","label":"Fold up","minutes":2,"autonomous":false,"when":"open","leaves":"closed"}]}],
    "economy":{"startingFunds":1000}}"#;

const TOWN: &str = r#"{"width":12,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[6.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"wallBed","x":4,"z":1}],
    "sims":[{"name":"Ada","x":3.5,"z":7.5},{"name":"Bea","x":6.5,"z":7.5}]}"#;

const SLEEP: usize = 0;
const FOLD_DOWN: usize = 1;
const FOLD_UP: usize = 2;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w
}

fn using(w: &World, sim: usize) -> Option<usize> {
    match w.sims[sim].current()?.task.kind {
        TaskKind::Use { interaction, .. } => Some(interaction),
        _ => None,
    }
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

#[test]
fn folding_down_and_up_when_told() {
    let mut w = world();
    assert!(!w.objects[0].open, "it's bought folded up");
    assert!(w.apply(Command::Use { sim: 0, object: 0, interaction: FOLD_UP }).is_err(), "nothing to fold up");

    let version = w.structure_version();
    w.apply(Command::Use { sim: 0, object: 0, interaction: FOLD_DOWN }).unwrap();
    assert!(until(&mut w, 30, |w| w.objects[0].open), "folded down");
    assert_ne!(w.structure_version(), version, "the house looks different");
    assert_eq!(using(&w, 0), Some(FOLD_DOWN), "as the folding starts, not once it's done");
    assert!(until(&mut w, 10, |w| w.sims[0].current().is_none()), "done in a couple of minutes");
    assert!(w.objects[0].open, "and it stays down");
    assert!(w.apply(Command::Use { sim: 0, object: 0, interaction: FOLD_DOWN }).is_err(), "down already");

    let w = common::reload(&w, CONTENT);
    assert!(w.objects[0].open, "still down after loading");

    let mut w = w;
    w.apply(Command::Use { sim: 1, object: 0, interaction: FOLD_UP }).unwrap();
    assert!(until(&mut w, 30, |w| !w.objects[0].open && w.sims[1].current().is_none()), "folded up again");
}

#[test]
fn going_to_bed_folds_it_down_and_the_last_one_up_folds_it_up() {
    let mut w = world();
    // Ada is less tired: she's up first, with Bea still asleep.
    w.sims[0].needs[0] = 0.5;
    w.sims[1].needs[0] = 0.0;
    for sim in 0..2 {
        w.apply(Command::Use { sim, object: 0, interaction: SLEEP }).unwrap();
    }
    assert!(until(&mut w, 30, |w| w.objects[0].open), "it folds down as they go to bed");
    assert!(until(&mut w, 30, |w| w.sims.iter().all(|s| s.asleep(&w.content))), "both asleep");

    let mut folded_by = None;
    let mut ada_up_with_bea_asleep = false;
    let done = until(&mut w, 120, |w| {
        let asleep = |s: usize| w.sims[s].asleep(&w.content);
        assert!(w.objects[0].open || !(asleep(0) || asleep(1)), "never folded up over a sleeper");
        ada_up_with_bea_asleep |= !asleep(0) && asleep(1);
        for s in 0..2 {
            if using(w, s) == Some(FOLD_UP) {
                folded_by.get_or_insert(s);
            }
        }
        !w.objects[0].open && w.sims.iter().all(|s| s.current().is_none())
    });
    assert!(ada_up_with_bea_asleep, "Ada got up first");
    assert!(done, "folded up again once both are up");
    assert_eq!(folded_by, Some(1), "by Bea, the last one up");
}

#[test]
fn nobody_folds_it_up_over_someone_in_it() {
    let mut w = world();
    w.sims[0].needs[0] = 0.0;
    w.apply(Command::Use { sim: 0, object: 0, interaction: SLEEP }).unwrap();
    assert!(until(&mut w, 30, |w| w.sims[0].asleep(&w.content)), "Ada is asleep");
    let refused = w.apply(Command::Use { sim: 1, object: 0, interaction: FOLD_UP });
    assert!(refused.is_err_and(|e| e.to_string().contains("someone is using")), "Bea can't fold it up on her");
}
