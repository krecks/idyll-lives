//! A host and their guest (`hosting.rs`): someone eating at home sits down beside a guest and
//! the two talk over the meal; a host offers a guest something, brings it from the fridge and
//! hands it over, and the guest sits down with it; a guest who isn't hungry says no; no fridge,
//! no offer; and a guest who leaves while it's on its way leaves nobody stuck or holding it.

use crate::common;

use sim_core::content::Pose;
use sim_core::social::EventKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, TICKS_PER_MINUTE, World, clock};

/// Hunger, fun, comfort and social; a fridge with a snack in it (not chosen on a whim: only
/// when the test says), dining sets to sit at and a sofa; a chat, and the host's offer (a guest
/// short of food says yes, a full one no).
const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01},
             {"id":"fun","label":"Fun","decayPerHour":0.01},
             {"id":"comfort","label":"Comfort","decayPerHour":0.01},
             {"id":"social","label":"Social","decayPerHour":0.01}],
    "animations":["idle","eat","fetch","serve","talk","sit"],
    "cooking":{"spoilHours":8,"storedSpoilHours":72,"eatMinutes":20,"dishesMinutes":5,"carryDishes":3},
    "items":[{"id":"snack","label":"Snack","emptyAs":"plateDirty"},{"id":"plateDirty","label":"Dirty plate","dirty":true}],
    "objects":[
      {"id":"fridge","name":"Fridge","price":500,"stores":true,"interactions":[
        {"id":"snack","label":"Grab a snack","anim":"fetch","minutes":2,"autonomous":false,"eat":{"item":"snack","minutes":30},
         "effects":{"hunger":0.4},"tags":["food"]}]},
      {"id":"table","name":"Dining set","price":300,"footprint":[2,1],"slots":2,"surface":{"height":0.75,"kind":"table"},
       "interactions":[{"id":"sit","label":"Sit at the table","minutes":120,"pose":"sit","effects":{"comfort":0.3},"tags":["food","lounge"]}]},
      {"id":"sofa","name":"Sofa","price":300,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"relax","label":"Relax","minutes":30,"pose":"sit","autonomous":false,"effects":{"comfort":0.3},"tags":["lounge"]}]}],
    "socials":[
      {"id":"chat","label":"Chat","minutes":10,"animation":"talk","tags":["social"],"prefer":"liked","acceptance":{"base":1.0},"needs":{"social":0.3},"targetNeeds":{"social":0.3}},
      {"id":"offer","label":"Offer a bite to eat","minutes":3,"animation":"talk","tags":["social"],"autonomous":false,
       "acceptance":{"base":0.0,"needs":{"hunger":1.6}},"needs":{"social":0.05},"success":{"targetFriendship":2,"effect":"serve"}}],
    "bondPresets":{"friends":{"friendship":60}},
    "visits":{"needs":{"social":0.5},"hours":4,"minFriendship":10,"earliestHour":6,"latestHour":23,"company":0.05,
      "offer":{"social":"offer","afterMinutes":5,"againMinutes":240,"handAnim":"serve"}},
    "economy":{"startingFunds":500}}"#;

/// Two homes side by side, open (no walls): the host's (the west one) has the fridge in a
/// corner, a dining set by it and another across the garden, and a sofa.
const TOWN: &str = r#"{"width":40,"depth":14,
    "plots":[{"name":"West","x":0,"z":0,"w":20,"d":12,"entry":[10.5,11.5]},{"name":"East","x":20,"z":0,"w":20,"d":12,"entry":[30.5,11.5]}],
    "households":[{"name":"Host","plot":0,"player":true},{"name":"Guest","plot":1}],
    "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"table","x":3,"z":3,"rot":0},
               {"def":"table","x":14,"z":6,"rot":0},{"def":"sofa","x":9,"z":8,"rot":2}],
    "sims":[{"name":"Ada","household":0,"x":3.5,"z":6.5},{"name":"Bo","household":1,"x":25.5,"z":6.5}],
    "relationships":[{"a":0,"b":1,"preset":"friends"}]}"#;

const FRIDGE: u32 = 0;
const NEAR_TABLE: u32 = 1;
const FAR_TABLE: u32 = 2;

/// The town at 11:00 with Bo let in at Ada's (by the far table), everyone's needs full, free
/// will on. `content` may change the shipped test content.
fn hosting(content: &str) -> World {
    let mut w = World::from_json(content, TOWN, 11).unwrap();
    for s in &mut w.sims {
        s.needs = [1.0; 8];
    }
    while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    w.sims[1].pos = [15.5, 8.5];
    w.sims[1].visiting = Some(sim_core::life::Visit {
        plot: 0,
        until: w.tick + (6.0 * 60.0 / sim_core::MINUTES_PER_TICK) as u64,
        idle: 0,
        farewell: 0,
        offer_at: 0,
    });
    w
}

fn need(w: &World, id: &str) -> usize {
    w.content.needs.iter().position(|n| n.id == id).unwrap()
}

/// What `sim` is doing, if it's a serve: (guest, stage).
fn serving(w: &World, sim: usize) -> Option<(u32, u8)> {
    match w.sims[sim].current()?.task.kind {
        TaskKind::Serve { guest, stage, .. } => Some((guest, stage)),
        _ => None,
    }
}

fn offering(w: &World, sim: usize) -> bool {
    let offer = w.content.social_index("offer").unwrap();
    w.sims[sim]
        .current()
        .is_some_and(|a| matches!(a.task.kind, TaskKind::Social { social, .. } if social == offer))
}

/// The seat `sim` eats at, sitting down.
fn eating_at(w: &World, sim: usize) -> Option<u32> {
    let a = w.sims[sim].current()?;
    match (a.task.kind, &a.phase) {
        (TaskKind::Eat { seat, .. }, Phase::Using { .. }) if w.sims[sim].pose == Pose::Sit => seat,
        _ => None,
    }
}

fn tick(w: &mut World, watch: &mut common::Watch) {
    w.tick_once();
    common::check(w);
    watch.sample(w);
}

#[test]
fn a_host_who_eats_sits_beside_the_guest_and_they_talk_over_the_meal() {
    // No offers here: Ada gets her own snack. Bo, weary, sits down at the far table; then Ada,
    // hungry, has a snack from the fridge right by the near table, and sits down with it at
    // Bo's table, not the one by the fridge. Neither is lonely, but a guest keeps the hosts
    // company (`visits.company`): a word from their seats over the meal.
    let content = CONTENT
        .replace(
            r#","offer":{"social":"offer","afterMinutes":5,"againMinutes":240,"handAnim":"serve"}"#,
            "",
        )
        .replace(r#""autonomous":false,"eat""#, r#""eat""#);
    assert_ne!(content, CONTENT);
    let mut w = hosting(&content);
    let (hunger, comfort) = (need(&w, "hunger"), need(&w, "comfort"));
    w.sims[1].needs[comfort] = 0.2;
    let mut watch = common::Watch::new();
    let seated = |w: &World| {
        w.sims[1].pose == Pose::Sit
            && w.sims[1].current().is_some_and(|a| {
                matches!(a.task.kind, TaskKind::Use { object, .. } if object == FAR_TABLE)
                    && matches!(a.phase, Phase::Using { .. })
            })
    };
    for _ in 0..30 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
        if seated(&w) {
            break;
        }
    }
    assert!(
        seated(&w),
        "Bo sat down at the far table: {:?}",
        w.sims[1].current().map(|a| a.task.kind)
    );
    w.sims[0].needs[hunger] = 0.2;
    let mut seat = None;
    let mut talked = false;
    for _ in 0..60 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
        seat = seat.or(eating_at(&w, 0));
        // A word while Ada has her meal (from their seats: neither gets up).
        let talking = w.sims[0].conversation(&w.content).is_some_and(|c| c.0 == 1)
            || w.sims[1].conversation(&w.content).is_some_and(|c| c.0 == 0);
        if talking && seat.is_some() {
            assert_eq!(w.sims[0].pose, Pose::Sit, "Ada talks from her seat");
            assert_eq!(w.sims[1].pose, Pose::Sit, "Bo talks from his seat");
            talked = true;
            break;
        }
    }
    assert_eq!(
        seat,
        Some(FAR_TABLE),
        "Ada sat down to eat at Bo's table, not the one by the fridge ({NEAR_TABLE})"
    );
    assert!(talked, "the two talked over Ada's meal");
    watch.assert_quiet();
}

#[test]
fn a_host_offers_the_guest_says_yes_and_has_what_is_brought() {
    let mut w = hosting(CONTENT);
    let hunger = need(&w, "hunger");
    w.sims[1].needs[hunger] = 0.35;
    let snack = w.content.item_index("snack").unwrap();
    let mut watch = common::Watch::new();
    let (mut offered, mut fetched, mut carried, mut ate_at) = (false, false, false, None);
    for _ in 0..90 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
        offered |= offering(&w, 0);
        // At the fridge, taking it out; then on the way to Bo with it in hand.
        if let Some((guest, stage)) = serving(&w, 0) {
            assert_eq!(guest, 1);
            fetched |= stage == 0
                && w.sims[0].current().is_some_and(|a| {
                    matches!(a.phase, Phase::Using { .. })
                        && matches!(a.task.kind, TaskKind::Serve { from, .. } if from == FRIDGE)
                });
            carried |= stage > 0 && w.sims[0].held.is_some_and(|h| h.def == snack);
        }
        ate_at = ate_at.or(eating_at(&w, 1));
        if ate_at.is_some() {
            break;
        }
    }
    assert!(offered, "Ada asked Bo if he'd like something");
    assert!(fetched, "Ada took it out of the fridge");
    assert!(carried, "Ada brought it over in her hands");
    let served = w
        .events
        .iter()
        .find(|e| e.kind == EventKind::Served)
        .expect("the story tells of it");
    assert_eq!((served.a, served.b, served.n), (0, 1, Some(snack as i64)));
    assert!(ate_at.is_some(), "Bo sat down with it");
    // (Put down on the table in front of him.)
    let plate = match w.sims[1].current().map(|a| a.task.kind) {
        Some(TaskKind::Eat { plate, .. }) => plate.and_then(|p| w.items.get(p)).map(|it| it.def),
        _ => None,
    };
    assert_eq!(
        plate.or(w.sims[1].held.map(|h| h.def)),
        Some(snack),
        "Bo has the snack"
    );
    assert!(w.sims[0].held.is_none(), "Ada handed it over");
    // He eats it up, and the plate is Ada's household's to wash.
    let before = w.sims[1].needs[hunger];
    for _ in 0..40 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
    }
    assert!(w.sims[1].needs[hunger] > before + 0.2, "Bo ate it");
    let dirty = w.content.item_index("plateDirty").unwrap();
    assert!(
        w.items
            .list
            .iter()
            .any(|it| it.def == dirty && it.household == 0),
        "the dirty plate is the host's: {:?}",
        w.items.list
    );
    // Once is enough for a while.
    let asked = (0..60 * TICKS_PER_MINUTE).any(|_| {
        tick(&mut w, &mut watch);
        offering(&w, 0)
    });
    assert!(!asked, "Ada doesn't ask again straight away");
    watch.assert_quiet();
}

#[test]
fn a_guest_who_is_not_hungry_says_no_and_nothing_is_brought() {
    let mut w = hosting(CONTENT);
    let mut watch = common::Watch::new();
    let mut offered = false;
    for _ in 0..60 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
        offered |= offering(&w, 0);
        assert!(
            serving(&w, 0).is_none(),
            "Ada went for something nobody wanted"
        );
        assert!(
            w.sims.iter().all(|s| s.held.is_none()),
            "nobody holds anything"
        );
    }
    assert!(offered, "Ada asked all the same");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Served));
    watch.assert_quiet();
}

#[test]
fn without_a_fridge_nobody_offers_anything() {
    let mut w = World::from_json(
        CONTENT,
        &TOWN.replace(r#"{"def":"fridge","x":1,"z":1,"rot":0},"#, ""),
        11,
    )
    .unwrap();
    for s in &mut w.sims {
        s.needs = [1.0; 8];
    }
    w.sims[1].needs[0] = 0.35;
    while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    w.sims[1].pos = [15.5, 8.5];
    w.sims[1].visiting = Some(sim_core::life::Visit {
        plot: 0,
        until: w.tick + (6.0 * 60.0 / sim_core::MINUTES_PER_TICK) as u64,
        idle: 0,
        farewell: 0,
        offer_at: 0,
    });
    let mut watch = common::Watch::new();
    for _ in 0..60 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
        assert!(!offering(&w, 0), "Ada offered something she hasn't got");
        assert!(serving(&w, 0).is_none());
    }
    watch.assert_quiet();
}

#[test]
fn a_guest_who_leaves_while_it_is_on_its_way_leaves_nobody_stuck() {
    let mut w = hosting(CONTENT);
    w.sims[1].needs[0] = 0.35;
    let mut watch = common::Watch::new();
    let on_its_way = |w: &World| serving(w, 0).is_some_and(|(_, stage)| stage > 0);
    for _ in 0..90 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
        if on_its_way(&w) {
            break;
        }
    }
    assert!(on_its_way(&w), "Ada set off with it");
    // Bo is off home.
    w.apply(Command::GoHome { sim: 1 }).unwrap();
    let mut dropped = false;
    for _ in 0..60 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
        let ada = &w.sims[0];
        let serves = |k: &TaskKind| matches!(k, TaskKind::Serve { .. });
        if !ada.current().is_some_and(|a| serves(&a.task.kind))
            && !ada.queue().any(|t| serves(&t.kind))
            && ada.held.is_none()
        {
            dropped = true;
        }
        if dropped && w.sims[1].visiting.is_none() {
            break;
        }
    }
    assert!(dropped, "Ada gave it up and put it down");
    assert!(w.sims[1].held.is_none(), "Bo took nothing home");
    assert!(w.sims[1].visiting.is_none(), "Bo got home");
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Served));
    // What she held is put down at home, as anything left is.
    let snack = w.content.item_index("snack").unwrap();
    assert!(
        w.items
            .list
            .iter()
            .any(|it| it.def == snack && it.household == 0),
        "the snack was put down at Ada's: {:?}",
        w.items.list
    );
    for _ in 0..30 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
    }
    watch.assert_quiet();
}

#[test]
fn a_guest_whose_host_is_called_away_mid_serve_gets_their_own_snack() {
    let mut w = hosting(CONTENT);
    let hunger = need(&w, "hunger");
    w.sims[1].needs[hunger] = 0.35;
    let mut watch = common::Watch::new();
    for _ in 0..90 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
        if serving(&w, 0).is_some() {
            break;
        }
    }
    assert!(serving(&w, 0).is_some(), "Ada never went for it");
    // Ada is sent across the garden: nothing is coming for Bo now. Still hungry, he gets a snack
    // of his own from the fridge (one a guest may have on a whim from now on).
    w.apply(Command::MoveTo {
        sim: 0,
        x: 18.5,
        z: 1.5,
    })
    .unwrap();
    let fridge = w.objects[FRIDGE as usize].def;
    w.content.objects[fridge].interactions[0].autonomous = true;
    let own = |w: &World| {
        w.sims[1].current().is_some_and(|a| match a.task.kind {
            TaskKind::Eat { .. } => true,
            TaskKind::Use { object, .. } => object == FRIDGE,
            _ => false,
        })
    };
    let mut got = false;
    for _ in 0..10 * TICKS_PER_MINUTE {
        tick(&mut w, &mut watch);
        assert!(serving(&w, 0).is_none(), "Ada carried on serving");
        if own(&w) {
            got = true;
            break;
        }
    }
    assert!(
        got,
        "Bo waited for what isn't coming: {:?}",
        w.sims[1].current().map(|a| a.task.kind)
    );
    assert!(!w.events.iter().any(|e| e.kind == EventKind::Served));
}
