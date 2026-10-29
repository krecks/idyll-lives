//! What residents want with the shipped content: things to have (they ask, the player says yes
//! or no; neighbours buy for themselves), the skills a promotion takes, and breaks at work.

mod common;

use serde_json::json;
use sim_core::content::Layer;
use sim_core::planner::{self, GoalKind};
use sim_core::social::EventKind;
use sim_core::{Command, World, clock};

/// A town with one newcomer (resident 0) in the player's house; free will on.
fn town(seed: u64) -> World {
    let content = common::content();
    let file = common::town(&content, seed, 1);
    World::from_json(&content, &file, seed as u32).unwrap()
}

fn until(w: &mut World, day: u32, hour: f32) {
    let target = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < target {
        w.tick_once();
    }
}

fn apply(w: &mut World, cmd: serde_json::Value) -> Result<(), sim_core::Error> {
    w.apply(Command::from_json(&cmd.to_string()).unwrap())
}

/// Keeps resident `i` home and free (no job, no job hunting).
fn homebody(w: &mut World, i: usize) {
    w.sims[i].job = None;
    w.sims[i].job_search_from = u32::MAX;
}

/// Something small and cheap household `h` doesn't have, that stands on the floor indoors and
/// is of no kind they have already.
fn something_new(w: &World, h: usize) -> usize {
    let c = &w.content;
    (0..c.objects.len())
        .find(|&d| {
            let o = &c.objects[d];
            o.price.is_some_and(|p| p > 0 && p <= 400)
                && o.footprint == [1, 1]
                && o.layer == Layer::Floor
                && !o.outdoors
                && o.interactions.iter().any(|it| it.autonomous && !it.baby)
                && !planner::has_at_home(w, h, d)
        })
        .expect("a small thing to want")
}

fn has_feeling(w: &World, i: usize, id: &str) -> bool {
    let f = w.content.feeling_index(id).unwrap();
    w.sims[i].feelings.iter().any(|m| m.def == f)
}

/// Resident 0 wants `object`, can well afford it, and the morning review is next.
fn wants(w: &mut World, object: usize) {
    homebody(w, 0);
    let id = w.content.objects[object].id.clone();
    apply(w, json!({"type": "addGoal", "sim": 0, "goal": {"def": "own", "object": id}})).unwrap();
    let h = w.sims[0].household as usize;
    w.households[h].funds = 50_000;
}

fn review(w: &mut World) {
    let day = clock::day(w.tick) + 1;
    until(w, day, w.content.planner.review_hour + 0.1);
}

#[test]
fn they_ask_for_what_they_want_and_a_yes_buys_it() {
    let mut w = town(21);
    let h = w.sims[0].household as usize;
    let object = something_new(&w, h);
    wants(&mut w, object);
    review(&mut w);
    let request = w.sims[0].planner.purchase.expect("asked for it");
    assert!(request.open && request.object == object);
    assert!(w.events.iter().any(|e| e.kind == EventKind::WantsToBuy && e.a == 0));
    assert!(!planner::has_at_home(&w, h, object), "nothing bought without a yes");
    common::dump_state(&w, "wants-asking");

    let before = w.households[h].funds;
    apply(&mut w, json!({"type": "acceptPurchase", "sim": 0})).unwrap();
    assert!(planner::has_at_home(&w, h, object));
    assert_eq!(w.households[h].funds, before - w.content.objects[object].price.unwrap());
    assert!(has_feeling(&w, 0, "gotWish"), "a mood boost");
    assert!(w.sims[0].planner.goals.is_empty(), "the goal is reached and gone");
    assert!(w.sims[0].planner.purchase.is_none());
    assert!(w.events.iter().any(|e| e.kind == EventKind::Bought && e.a == 0));
    assert!(apply(&mut w, json!({"type": "acceptPurchase", "sim": 0})).is_err(), "nothing left to say yes to");
}

#[test]
fn told_no_again_and_again_they_get_cross() {
    let mut w = town(22);
    let h = w.sims[0].household as usize;
    let object = something_new(&w, h);
    wants(&mut w, object);
    review(&mut w);
    apply(&mut w, json!({"type": "declinePurchase", "sim": 0})).unwrap();
    assert!(!has_feeling(&w, 0, "gotWish") && !has_feeling(&w, 0, "deniedAgain"), "a first no: no boost, no anger");
    assert!(!w.sims[0].planner.purchase.unwrap().open);

    // They wait a few days before asking again.
    review(&mut w);
    assert!(!w.sims[0].planner.purchase.unwrap().open, "not again the next day");
    let wait = w.content.planner.ask_again_days;
    for _ in 1..wait {
        review(&mut w);
    }
    let request = w.sims[0].planner.purchase.unwrap();
    assert!(request.open && request.declines == 1, "asking again: {request:?}");
    apply(&mut w, json!({"type": "declinePurchase", "sim": 0})).unwrap();
    assert!(has_feeling(&w, 0, "deniedAgain"), "cross at the second no");
    common::dump_state(&w, "wants-cross");
    assert!(w.events.iter().any(|e| e.kind == EventKind::DeniedAgain && e.a == 0));
    assert!(!planner::has_at_home(&w, h, object));
}

#[test]
fn nobody_asks_without_free_will_or_the_money_for_rent_too() {
    let mut w = town(23);
    let h = w.sims[0].household as usize;
    let object = something_new(&w, h);
    wants(&mut w, object);
    w.households[h].free_will = false;
    review(&mut w);
    assert!(w.sims[0].planner.purchase.is_none(), "no free will");

    w.households[h].free_will = true;
    let (rent, bills) = sim_core::life::weekly_costs(&w, h).unwrap_or((0, 0));
    w.households[h].funds = w.content.objects[object].price.unwrap() + rent + bills - 1;
    review(&mut w);
    assert!(w.sims[0].planner.purchase.is_none(), "it would eat into the rent");
}

#[test]
fn neighbours_buy_what_they_want_themselves() {
    let mut w = town(24);
    let i = (1..w.sims.len())
        .find(|&i| {
            let h = w.sims[i].household as usize;
            !w.households[h].player && w.households[h].plot.is_some() && w.sims[i].here()
        })
        .unwrap();
    homebody(&mut w, i);
    let h = w.sims[i].household as usize;
    let object = something_new(&w, h);
    let id = w.content.objects[object].id.clone();
    apply(&mut w, json!({"type": "addGoal", "sim": i, "goal": {"def": "own", "object": id}})).unwrap();
    w.households[h].funds = 50_000;
    review(&mut w);
    assert!(planner::has_at_home(&w, h, object), "bought it");
    assert!(w.events.iter().any(|e| e.kind == EventKind::Bought && e.a == i as u32));
    let (day, hour) = (clock::day(w.tick), w.content.planner.review_hour + 0.5);
    until(&mut w, day, hour);
    assert!(
        !w.sims[i].planner.goals.iter().any(|g| g.object == Some(object)),
        "and the goal is gone"
    );
}

#[test]
fn a_better_one_replaces_the_old() {
    let mut w = town(25);
    let h = w.sims[0].household as usize;
    let plot = w.households[h].plot.unwrap();
    // Something at home with a pricier one of its kind in the catalog.
    let c = &w.content;
    let (old, new) = w
        .objects
        .iter()
        .filter(|o| w.plot_at(o.x, o.z) == Some(plot) && !c.objects[o.def].group.is_empty())
        .find_map(|o| {
            let d = &c.objects[o.def];
            let better = (0..c.objects.len()).find(|&n| {
                let b = &c.objects[n];
                b.group == d.group
                    && b.layer == d.layer
                    && b.footprint == d.footprint
                    && b.price.is_some_and(|p| d.price.is_some_and(|q| p > q))
                    && b.interactions.iter().any(|x| d.interactions.iter().any(|y| x.id == y.id))
                    && !planner::has_at_home(&w, h, n)
            })?;
            Some((o.def, better))
        })
        .expect("an upgrade to have");
    let count = |w: &World, d: usize| w.objects.iter().filter(|o| o.def == d && w.plot_at(o.x, o.z) == Some(plot)).count();
    let olds = count(&w, old);
    wants(&mut w, new);
    review(&mut w);
    apply(&mut w, json!({"type": "acceptPurchase", "sim": 0})).unwrap();
    assert_eq!(count(&w, new), 1);
    assert_eq!(count(&w, old), olds - 1, "the old one was sold");
}

/// A small catalog: a fridge, a bigger one that does all it does and more, a range cooker that
/// shares the gourmet dinner with the fridges but is no fridge, and a bed and a double bed.
const CATALOG: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.0},{"id":"energy","label":"Energy","decayPerHour":0.0},
             {"id":"fun","label":"Fun","decayPerHour":0.0}],
    "items":[{"id":"ingredients","label":"Ingredients"},{"id":"mealPot","label":"Meal","serving":"mealPlate"},
             {"id":"mealPlate","label":"Plate of food"}],
    "recipes":[
      {"id":"meal","label":"Make a meal","item":"mealPot","servings":[1,4],"effects":{"hunger":0.9},
       "steps":[{"at":["ingredients"],"minutes":5,"holds":"ingredients"},{"at":["stove","oven"],"minutes":20,"optional":true}]},
      {"id":"gourmet","label":"Cook a gourmet meal","item":"mealPot","servings":[2,4],"effects":{"hunger":1.0},
       "steps":[{"at":["ingredients"],"minutes":5,"holds":"ingredients"},{"at":["stove"],"minutes":45}]}],
    "objects":[
      {"id":"fridge","name":"Fridge","price":600,"group":"appliances","stations":["ingredients"],"stores":true,"interactions":[
        {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.3}},
        {"id":"meal","label":"Make a meal","recipe":"meal"},
        {"id":"gourmet","label":"Cook a gourmet meal","recipe":"gourmet"}]},
      {"id":"fridgeBig","name":"Big fridge","price":1500,"group":"appliances","stations":["ingredients"],"stores":true,"interactions":[
        {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.35}},
        {"id":"meal","label":"Make a meal","recipe":"meal"},
        {"id":"gourmet","label":"Cook a gourmet meal","recipe":"gourmet"},
        {"id":"coldDrink","label":"Cold drink","minutes":5,"effects":{"fun":0.05}}]},
      {"id":"rangeCooker","name":"Range cooker","price":1600,"group":"appliances","stations":["stove","oven"],"interactions":[
        {"id":"cook","label":"Cook dinner","recipe":"meal"},
        {"id":"gourmet","label":"Cook a gourmet meal","recipe":"gourmet"}]},
      {"id":"bed","name":"Bed","price":300,"group":"beds","footprint":[1,2],"interactions":[
        {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0}}]},
      {"id":"bedDouble","name":"Double bed","price":900,"group":"beds","footprint":[2,2],"slots":2,"interactions":[
        {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0}},
        {"id":"read","label":"Read in bed","minutes":30,"pose":"lie","effects":{"fun":0.2}}]}],
    "goals":[{"id":"own","label":"Have a {object}","kind":"own"}]}"#;

/// Two homes on a street, each with a fridge and a bed: the player's (Ada) and the neighbours'
/// (Ned), who buy for themselves.
const STREET: &str = r#"{"width":24,"depth":10,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]},{"name":"Next door","x":12,"z":0,"w":12,"d":10,"entry":[18.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true,"funds":50000},{"name":"Neighbours","plot":1,"funds":50000}],
    "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"bed","x":4,"z":1,"rot":0},
               {"def":"fridge","x":13,"z":1,"rot":0},{"def":"bed","x":16,"z":1,"rot":0}],
    "sims":[{"name":"Ada","household":0,"x":6.5,"z":6.5},{"name":"Ned","household":1,"x":18.5,"z":6.5}]}"#;

/// The same two homes, the beds hemmed in: fridges stand beside them, so nothing wider than the
/// bed itself has room where the bed stands.
const HEMMED: &str = r#"{"width":24,"depth":10,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]},{"name":"Next door","x":12,"z":0,"w":12,"d":10,"entry":[18.5,9.5]}],
    "households":[{"name":"Player","plot":0,"player":true,"funds":50000},{"name":"Neighbours","plot":1,"funds":50000}],
    "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"bed","x":4,"z":1,"rot":0},
               {"def":"fridge","x":5,"z":1,"rot":0},{"def":"fridge","x":5,"z":2,"rot":0},
               {"def":"fridge","x":13,"z":1,"rot":0},{"def":"bed","x":16,"z":1,"rot":0}],
    "sims":[{"name":"Ada","household":0,"x":6.5,"z":6.5},{"name":"Ned","household":1,"x":18.5,"z":6.5}]}"#;

/// What's at home for household `h`, by object type id (sorted).
fn at_home(w: &World, h: usize) -> Vec<&str> {
    let plot = w.households[h].plot;
    let mut ids: Vec<&str> = w.objects.iter().filter(|o| w.plot_at(o.x, o.z) == plot).map(|o| w.content.objects[o.def].id.as_str()).collect();
    ids.sort_unstable();
    ids
}

/// Ada (the player says yes to her request) and Ned (buys it himself) each get `id` at the
/// morning review.
fn both_buy(id: &str) -> World {
    let mut w = World::from_json(CATALOG, STREET, 9).unwrap();
    buy_both(&mut w, id);
    w
}

/// The same, on a world the test set up first: Ada says yes to her request, Ned buys for himself.
fn buy_both(w: &mut World, id: &str) {
    for i in 0..2 {
        apply(w, json!({"type": "addGoal", "sim": i, "goal": {"def": "own", "object": id}})).unwrap();
    }
    review(w);
    assert!(w.sims[0].planner.purchase.is_some_and(|p| p.open), "Ada asks for the {id}");
    apply(w, json!({"type": "acceptPurchase", "sim": 0})).unwrap();
    for h in 0..2 {
        assert!(planner::has_at_home(w, h, w.content.object_index(id).unwrap()), "household {h} bought the {id}");
    }
}

/// Where the object of type `id` at home for household `h` stands (its tile and how it's turned),
/// if they have one.
fn standing(w: &World, h: usize, id: &str) -> Option<(i32, i32, u8)> {
    let plot = w.households[h].plot?;
    let o = w.objects.iter().find(|o| w.plot_at(o.x, o.z) == Some(plot) && w.content.objects[o.def].id == id)?;
    Some((o.x, o.z, o.rot))
}

#[test]
fn the_new_one_stands_where_the_old_one_stood() {
    // A swap, not a new thing in some free corner of the home: what they had goes, and the new one
    // takes its place, so the room they know stays as it was (the bed they slept in stays in the
    // corner it has always lain in).
    let mut w = World::from_json(CATALOG, STREET, 9).unwrap();
    let had: Vec<(i32, i32, u8)> = (0..2).map(|h| standing(&w, h, "bed").expect("a bed at home")).collect();
    buy_both(&mut w, "bedDouble");
    for (h, stood) in had.iter().enumerate() {
        assert_eq!(at_home(&w, h), ["bedDouble", "fridge"], "household {h} sold the old bed");
        assert_eq!(standing(&w, h, "bedDouble"), Some(*stood), "household {h}: the new bed stands where the old one stood");
    }
}

#[test]
fn a_bigger_one_that_wont_fit_their_spot_goes_its_own_way() {
    // There's no room for it over the tiles the old one had: then, and only then, the old one is
    // sold and the new one finds a free spot somewhere else.
    let mut w = World::from_json(CATALOG, HEMMED, 9).unwrap();
    let had = standing(&w, 0, "bed").expect("a bed at home");
    apply(&mut w, json!({"type": "addGoal", "sim": 0, "goal": {"def": "own", "object": "bedDouble"}})).unwrap();
    review(&mut w);
    apply(&mut w, json!({"type": "acceptPurchase", "sim": 0})).unwrap();
    let home = at_home(&w, 0);
    assert!(home.contains(&"bedDouble"), "the new bed came in: {home:?}");
    assert!(!home.contains(&"bed"), "and the old one was sold: {home:?}");
    assert_ne!(standing(&w, 0, "bedDouble"), Some(had), "where there was room for it, elsewhere");
}

#[test]
fn a_range_cooker_goes_in_beside_the_fridge() {
    // It shares the gourmet dinner with the fridge, but selling the fridge for it would leave no
    // snacks and nothing to cook with.
    let w = both_buy("rangeCooker");
    for h in 0..2 {
        assert_eq!(at_home(&w, h), ["bed", "fridge", "rangeCooker"], "household {h} keeps its fridge");
    }
}

#[test]
fn a_better_fridge_or_bed_still_replaces_the_old_one() {
    let w = both_buy("fridgeBig");
    for h in 0..2 {
        assert_eq!(at_home(&w, h), ["bed", "fridgeBig"], "household {h} sold the old fridge");
    }
    let w = both_buy("bedDouble");
    for h in 0..2 {
        assert_eq!(at_home(&w, h), ["bedDouble", "fridge"], "household {h} sold the old bed");
    }
}

#[test]
fn wants_and_requests_survive_saving() {
    let mut w = town(26);
    let h = w.sims[0].household as usize;
    let object = something_new(&w, h);
    wants(&mut w, object);
    review(&mut w);
    apply(&mut w, json!({"type": "declinePurchase", "sim": 0})).unwrap();
    let loaded = World::from_save_json(&common::content(), &w.save_json()).unwrap();
    let goal = loaded.sims[0].planner.goals.iter().find(|g| g.object.is_some()).unwrap();
    assert_eq!(goal.object, Some(object));
    assert_eq!(loaded.content.goals[goal.def].kind, GoalKind::Own);
    assert_eq!(loaded.sims[0].planner.purchase, w.sims[0].planner.purchase);
}

#[test]
fn residents_come_up_with_things_to_want() {
    let mut w = town(27);
    until(&mut w, 10, 8.0);
    let wanted = w.sims.iter().any(|s| {
        s.planner.goals.iter().chain(&s.planner.suggestions).any(|g| g.object.is_some())
    }) || w.events.iter().any(|e| matches!(e.kind, EventKind::WantsToBuy | EventKind::Bought));
    assert!(wanted, "someone wants something");
}

/// Resident 0 in a job whose next grade needs some skill they lack.
fn employed(w: &mut World) -> usize {
    homebody(w, 0);
    let career = (0..w.content.careers.len())
        .find(|&c| w.content.careers[c].levels.get(1).is_some_and(|l| l.requires.iter().any(|r| r.1 >= 1.0)))
        .unwrap();
    w.autonomy = false;
    apply(w, json!({"type": "joinCareer", "sim": 0, "career": career, "level": 0})).unwrap();
    w.sims[0].skills = [0.0; 16];
    career
}

#[test]
fn a_promotion_takes_the_next_grades_skills() {
    let mut w = town(28);
    let career = employed(&mut w);
    apply(&mut w, json!({"type": "addGoal", "sim": 0, "goal": {"def": "promotion"}})).unwrap();
    w.sims[0].job.as_mut().unwrap().performance = 100.0;
    let goal = w.sims[0].planner.goals[0];
    let needs = planner::goal_skills(&w.content, &w.sims[0], &goal);
    assert_eq!(needs.len(), w.content.careers[career].levels[1].requires.iter().filter(|r| r.1 > 0.0).count());
    let p = planner::progress(&w, 0, &goal);
    assert!(p < 0.5, "a full bar but none of the skills: {p}");
    for &(s, l) in &needs {
        w.sims[0].skills[s] = l;
    }
    assert!(planner::progress(&w, 0, &goal) >= 0.89, "skills and a full bar: all but there");

    // A higher grade goal takes every grade's skills on the way.
    let levels = w.content.careers[career].levels.len().min(4);
    apply(&mut w, json!({"type": "addGoal", "sim": 0, "goal": {"def": "jobLevel", "target": levels}})).unwrap();
    let far = planner::goal_skills(&w.content, &w.sims[0], &w.sims[0].planner.goals[1]);
    for (s, l) in &needs {
        assert!(far.iter().any(|(t, m)| t == s && m >= l));
    }

    // Promoted: the goal goes straight away.
    w.sims[0].job.as_mut().unwrap().level = 1;
    w.sims[0].planner.recheck = true;
    w.tick_once();
    assert!(
        !w.sims[0].planner.goals.iter().any(|g| w.content.goals[g.def].kind == GoalKind::Promoted),
        "a reached goal is deleted"
    );
}

#[test]
fn reached_ideas_just_go_and_reached_goals_cant_be_added() {
    let mut w = town(29);
    homebody(&mut w, 0);
    w.sims[0].skills[0] = 5.0;
    let skill = w.content.skills[0].id.clone();
    assert!(apply(&mut w, json!({"type": "addGoal", "sim": 0, "goal": {"def": "skill", "skill": skill, "target": 3}})).is_err());
    apply(&mut w, json!({"type": "addGoal", "sim": 0, "goal": {"def": "skill", "skill": skill, "target": 7}})).unwrap();
    let mut idea = w.sims[0].planner.goals.pop().unwrap();
    idea.target = 3.0;
    w.sims[0].planner.suggestions = vec![idea];
    let reached = w.events.iter().filter(|e| e.kind == EventKind::GoalReached).count();
    for _ in 0..200 {
        w.tick_once();
    }
    assert!(w.sims[0].planner.suggestions.is_empty());
    assert_eq!(w.events.iter().filter(|e| e.kind == EventKind::GoalReached).count(), reached, "no fuss");
}

#[test]
fn at_work_they_eat_use_the_toilet_and_pay_for_lunch() {
    let mut w = town(30);
    employed(&mut w);
    let h = w.sims[0].household as usize;
    let need = |w: &World, id: &str| w.content.needs.iter().position(|n| n.id == id).unwrap();
    let (hunger, bladder) = (need(&w, "hunger"), need(&w, "bladder"));
    // Off at work for a while.
    w.sims[0].away_until = Some(w.tick + 1000);
    w.sims[0].needs[hunger] = 0.2;
    w.sims[0].needs[bladder] = 0.1;
    let funds = w.households[h].funds;
    w.tick_once();
    assert!(w.sims[0].needs[hunger] > 0.5, "lunch: {}", w.sims[0].needs[hunger]);
    assert!(w.sims[0].needs[bladder] > 0.9, "the toilet: {}", w.sims[0].needs[bladder]);
    assert!(w.households[h].funds < funds, "lunch isn't free");

    // School lunch is.
    w.sims[0].job = None;
    w.sims[0].needs[hunger] = 0.2;
    let funds = w.households[h].funds;
    w.tick_once();
    assert!(w.sims[0].needs[hunger] > 0.5);
    assert_eq!(w.households[h].funds, funds);
}

/// Resident 0's plan as the UI gets it (`SimView.plan` in `protocol.ts`).
fn ui_plan(w: &World) -> serde_json::Value {
    let ui: serde_json::Value = serde_json::from_str(&sim_core::view::ui_state_json(w)).unwrap();
    let sims = ui["sims"].as_array().expect("sims in the UI state");
    sims.iter().find(|s| s["id"] == 0).expect("resident 0")["plan"].clone()
}

#[test]
fn the_ui_gets_the_request_and_what_goals_still_take() {
    let mut w = town(31);
    let h = w.sims[0].household as usize;
    let object = something_new(&w, h);
    wants(&mut w, object);
    review(&mut w);
    let plan = ui_plan(&w);
    let id = &w.content.objects[object].id;
    assert_eq!(plan["purchase"]["object"], json!(id), "the Buy / No request: {}", plan["purchase"]);
    assert_eq!(plan["purchase"]["price"], json!(w.content.objects[object].price.unwrap()));
    let own = plan["goals"].as_array().unwrap().iter().find(|g| g["object"] == json!(id)).unwrap();
    assert_eq!(own["steps"][0]["kind"], "funds");
    apply(&mut w, json!({"type": "acceptPurchase", "sim": 0})).unwrap();
    assert!(ui_plan(&w)["purchase"].is_null(), "gone once bought");

    employed(&mut w);
    apply(&mut w, json!({"type": "addGoal", "sim": 0, "goal": {"def": "promotion"}})).unwrap();
    let plan = ui_plan(&w);
    let promo = plan["goals"].as_array().unwrap().iter().find(|g| g["kind"] == "promoted").unwrap();
    let kinds: Vec<&str> = promo["steps"].as_array().unwrap().iter().map(|s| s["kind"].as_str().unwrap()).collect();
    assert!(kinds.contains(&"skill") && kinds.contains(&"performance"), "{kinds:?}");
    w.autonomy = true;
    common::dump_state(&w, "goals-steps");
}

#[test]
fn outdoor_things_go_outside_and_pot_plants_inside() {
    let mut w = town(32);
    let h = w.sims[0].household as usize;
    let outdoors = sim_core::lot::OUTDOORS;
    let where_is = |w: &World, def: usize| {
        let o = w.objects.iter().find(|o| o.def == def && w.plot_at(o.x, o.z) == w.households[h].plot).unwrap();
        w.lot.room_at(o.x, o.z)
    };
    for (id, outside) in [("flowerbed", true), ("hammock", true), ("foodie.pizzaOven", true), ("garden.monstera", false)] {
        let def = w.content.object_index(id).unwrap();
        wants(&mut w, def);
        review(&mut w);
        apply(&mut w, json!({"type": "acceptPurchase", "sim": 0})).unwrap();
        assert_eq!(where_is(&w, def) == outdoors, outside, "{id}");
    }
}
