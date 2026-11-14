//! Something a resident asked for, once the player says yes, comes as a parcel to the front door:
//! the one who asked carries it in and puts it together where it goes (`setup.rs`), someone else
//! does if they can't, and it's set up for them in the end whatever happens.

use crate::common::{self, Watch, apply, garden_home, until};

use serde_json::json;
use sim_core::content::Layer;
use sim_core::planner::{self, PurchaseRequest};
use sim_core::social::EventKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{World, clock};

/// A small thing for indoors (one tile, on the floor) household 0 doesn't have.
fn something_new(w: &World) -> usize {
    let c = &w.content;
    (0..c.objects.len())
        .find(|&d| {
            let o = &c.objects[d];
            o.price.is_some_and(|p| p > 0 && p <= 400)
                && o.footprint == [1, 1]
                && o.layer == Layer::Floor
                && !o.outdoors
                && o.interactions.iter().any(|it| it.autonomous && !it.baby)
                && !planner::has_at_home(w, 0, d)
        })
        .expect("a small thing to want")
}

/// Resident `i` asks for `object` and the player says yes.
fn yes(w: &mut World, i: usize, object: usize) {
    let id = w.content.objects[object].id.clone();
    apply(
        w,
        json!({"type": "addGoal", "sim": i, "goal": {"def": "own", "object": id}}),
    )
    .unwrap();
    w.sims[i].planner.purchase = Some(PurchaseRequest {
        object,
        open: true,
        day: clock::day(w.tick),
        declines: 0,
    });
    apply(w, json!({"type": "acceptPurchase", "sim": i})).unwrap();
}

/// Who is setting something up now (any stage), with the stage and phase.
fn setting_up(w: &World) -> Vec<(usize, u8, bool)> {
    w.sims
        .iter()
        .enumerate()
        .filter_map(|(i, s)| match s.current() {
            Some(a) => match a.task.kind {
                TaskKind::SetUp { stage, .. } => {
                    Some((i, stage, matches!(a.phase, Phase::Using { .. })))
                }
                _ => None,
            },
            None => None,
        })
        .collect()
}

/// The parcels the view draws (item ids from `setup::PARCEL_IDS`).
fn parcels_shown(w: &World) -> usize {
    let items: serde_json::Value = serde_json::from_str(&sim_core::view::items_json(w)).unwrap();
    items
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["def"] == "parcel")
        .count()
}

#[test]
fn a_yes_brings_a_parcel_they_carry_in_and_set_up() {
    let (mut w, _) = garden_home();
    let object = something_new(&w);
    let price = w.content.objects[object].price.unwrap();
    let before = w.households[0].funds;
    yes(&mut w, 0, object);

    // Paid for and on its way: not at home yet, the goal not reached yet.
    assert_eq!(w.households[0].funds, before - price, "paid when bought");
    assert!(!planner::has_at_home(&w, 0, object));
    assert_eq!(w.setups.len(), 1);
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::Bought && e.a == 0)
    );
    assert!(w.sims[0].planner.purchase.is_none());
    assert!(
        w.sims[0]
            .planner
            .goals
            .iter()
            .any(|g| g.object == Some(object)),
        "the goal waits for it to stand at home"
    );
    assert_eq!(parcels_shown(&w), 1, "a parcel at the door at once");

    // Ada goes to the door and picks it up...
    let mut watch = Watch::new();
    let mut picked = false;
    let mut carried = false;
    let mut putting = false;
    let done = until(&mut w, 120, |w| {
        watch.sample(w);
        for (i, stage, using) in setting_up(w) {
            assert_eq!(i, 0, "the one who asked does it");
            picked |= stage == 0 && using;
            if stage == 1 && !using && w.sims[0].held.is_some() {
                carried = true;
                assert_eq!(common::shown_held(w).as_deref(), Some("parcel"));
                assert_eq!(parcels_shown(w), 0, "the parcel is in her hands");
            }
            if stage == 1 && using {
                putting = true;
                assert!(w.sims[0].held.is_none(), "put down to put it together");
                assert_eq!(parcels_shown(w), 1, "the parcel beside her");
            }
        }
        planner::has_at_home(w, 0, object)
    });
    assert!(done, "set up within two hours");
    assert!(
        picked && carried && putting,
        "picked up {picked}, carried {carried}, put together {putting}"
    );
    watch.assert_quiet();

    // ...and there it is: the story says so, and the goal is reached.
    assert!(w.setups.is_empty());
    assert_eq!(
        w.households[0].funds,
        before - price,
        "paid once, not again"
    );
    w.tick_once();
    assert_eq!(parcels_shown(&w), 0);
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::SetUp && e.a == 0 && e.object == Some(object as u32))
    );
    until(&mut w, 30, |w| {
        !w.sims[0]
            .planner
            .goals
            .iter()
            .any(|g| g.object == Some(object))
    });
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::GoalReached && e.a == 0),
        "the goal reached once it stands there"
    );
    // Near where she put it together.
    let o = w.objects.iter().find(|o| o.def == object).unwrap();
    assert!(
        common::apart(w.sims[0].pos, o.centre(&w.content)) < 2.5,
        "she stands by it"
    );
    common::check(&w);
}

#[test]
fn not_asked_twice_while_it_waits() {
    let (mut w, _) = garden_home();
    w.households[0].free_will = true;
    let object = something_new(&w);
    yes(&mut w, 0, object);
    // Away all day: it waits at the door, and the morning review doesn't ask for it again.
    w.sims[0].away_until = Some(w.tick + 30 * 60 * sim_core::TICKS_PER_MINUTE);
    let (day, hour) = (clock::day(w.tick) + 1, w.content.planner.review_hour);
    common::run_to(&mut w, day, hour + 0.2);
    assert!(
        w.sims[0].planner.purchase.is_none(),
        "no second request for what's on its way"
    );
}

#[test]
fn someone_else_does_it_while_the_one_who_asked_is_away() {
    let (mut w, _) = garden_home();
    let object = something_new(&w);
    yes(&mut w, 0, object);
    // Ada is out at work for the day.
    w.sims[0].away_until = Some(w.tick + 10 * 60 * sim_core::TICKS_PER_MINUTE);
    let help = w.content.object_rules.set_up.help_hours;
    // Not straight away: Ada may be back in a minute.
    until(&mut w, (help * 60.0) as u64 - 5, |w| {
        !setting_up(w).is_empty()
    });
    assert!(setting_up(&w).is_empty(), "they wait for her a while");
    // Then Ben (the other grown-up; not the children) does it.
    let done = until(&mut w, 120, |w| {
        for (i, ..) in setting_up(w) {
            assert_eq!(w.sims[i].name, "Ben", "a grown-up at home helps");
        }
        planner::has_at_home(w, 0, object)
    });
    assert!(done, "Ben set it up");
    let e = w
        .events
        .iter()
        .find(|e| e.kind == EventKind::SetUp)
        .unwrap();
    assert_eq!(w.sims[e.a as usize].name, "Ben");
}

#[test]
fn nobody_free_it_is_set_up_for_them_in_the_end() {
    let (mut w, _) = garden_home();
    let object = something_new(&w);
    yes(&mut w, 0, object);
    // Every grown-up away for longer than it waits.
    let hours = w.content.object_rules.set_up.wait_hours;
    let back = w.tick + ((hours + 2.0) * 60.0) as u64 * sim_core::TICKS_PER_MINUTE;
    for sim in &mut w.sims[..2] {
        sim.away_until = Some(back);
    }
    let done = until(&mut w, ((hours + 1.0) * 60.0) as u64, |w| {
        assert!(setting_up(w).is_empty(), "the children leave it be");
        planner::has_at_home(w, 0, object)
    });
    assert!(done, "it doesn't wait for ever");
    assert!(w.setups.is_empty());
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::SetUpForThem && e.a == 0)
    );
}

#[test]
fn a_spot_taken_meanwhile_it_goes_in_elsewhere() {
    let (mut w, _) = garden_home();
    let object = something_new(&w);
    yes(&mut w, 0, object);
    // Ada sets off with the parcel; meanwhile the spot she's taking it to gets taken.
    assert!(
        until(&mut w, 60, |w| setting_up(w)
            .iter()
            .any(|&(_, stage, using)| stage == 1 && !using)),
        "off with the parcel"
    );
    let ([x, z, rot], _) = w.setups[0].spot().expect("a spot chosen");
    let filler = w.content.objects[object].id.clone();
    apply(
        &mut w,
        json!({"type": "buy", "household": 0, "object": filler, "at": [x, z, rot]}),
    )
    .unwrap();
    let taken = w
        .objects
        .iter()
        .filter(|o| o.def == object)
        .map(|o| (o.x, o.z))
        .collect::<Vec<_>>();
    assert_eq!(taken, [(x, z)]);
    let done = until(&mut w, 120, |w| {
        w.objects.iter().filter(|o| o.def == object).count() == 2
    });
    assert!(done, "it went in all the same");
    assert!(w.setups.is_empty());
    assert!(
        w.objects
            .iter()
            .filter(|o| o.def == object)
            .any(|o| (o.x, o.z) != (x, z)),
        "at another free spot"
    );
    assert!(w.events.iter().any(|e| e.kind == EventKind::SetUp));
    common::check(&w);
}

#[test]
fn saved_while_carrying_it_the_parcel_is_back_at_the_door() {
    let (mut w, content) = garden_home();
    let object = something_new(&w);
    let before = w.households[0].funds;
    yes(&mut w, 0, object);
    assert!(
        until(&mut w, 60, |w| w.sims[0].held.is_some()),
        "carrying the parcel"
    );
    let saved = w.save_json();
    assert!(saved.contains("\"setups\""), "what was bought is saved");
    let mut w = common::reload(&w, &content);
    assert_eq!(w.setups.len(), 1, "it waits on");
    assert!(w.sims[0].held.is_none(), "the parcel is out of her hands");
    assert_eq!(w.setups[0].object, object);
    let done = until(&mut w, 120, |w| planner::has_at_home(w, 0, object));
    assert!(done, "and she carries on after the load");
    assert_eq!(
        w.households[0].funds,
        before - w.content.objects[object].price.unwrap(),
        "paid once"
    );
    assert!(
        !w.save_json().contains("\"setups\""),
        "nothing waits now: a save without it, as before"
    );
}

#[test]
fn with_nowhere_to_put_it_a_yes_buys_nothing() {
    let (mut w, _) = garden_home();
    let object = something_new(&w);
    let id = w.content.objects[object].id.clone();
    apply(
        &mut w,
        json!({"type": "addGoal", "sim": 0, "goal": {"def": "own", "object": id}}),
    )
    .unwrap();
    w.sims[0].planner.purchase = Some(PurchaseRequest {
        object,
        open: true,
        day: clock::day(w.tick),
        declines: 0,
    });
    let before = w.households[0].funds;
    // (The household moves out meanwhile: no home, no room.)
    let plot = w.households[0].plot.take();
    assert!(apply(&mut w, json!({"type": "acceptPurchase", "sim": 0})).is_err());
    w.households[0].plot = plot;
    assert_eq!(w.households[0].funds, before, "nothing paid");
    assert!(w.setups.is_empty());
}

#[test]
fn a_need_running_low_doesnt_keep_them_from_it() {
    // A home without a shower: hygiene stays low all day. That's no reason never to set it up.
    let (mut w, _) = garden_home();
    let hygiene = w
        .content
        .needs
        .iter()
        .position(|n| n.id == "hygiene")
        .unwrap();
    w.sims[0].needs[hygiene] = 0.02;
    let object = something_new(&w);
    yes(&mut w, 0, object);
    assert!(
        until(&mut w, 5, |w| setting_up(w).iter().any(|&(i, ..)| i == 0)),
        "she sets off for it"
    );
}
