//! Cooking makes food: a recipe goes round its stations (fridge, counter, stove), serves a dish
//! on a surface, housemates sit down to a serving each, and the dirty plates are washed up.

mod common;

use sim_core::items::ItemAt;
use sim_core::social::EventKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World, snapshot};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1},
             {"id":"fun","label":"Fun","decayPerHour":0.02},
             {"id":"comfort","label":"Comfort","decayPerHour":0.02}],
    "animations":["idle","cook","eat","wash","clean","fetch","chop","stir","flip","sit"],
    "accidents":[{"id":"takeout","need":"hunger","story":"ordered takeout","cost":25,"graceMinutes":20,
      "crowdedGraceMinutes":20,"rest":{"label":"Eat takeout","minutes":20,"anim":"eat","gains":{"hunger":0.7},"tags":["food"]}}],
    "cooking":{"spoilHours":8,"storedSpoilHours":72,"eatMinutes":20,"dishesMinutes":5,"carryDishes":3},
    "items":[
      {"id":"ingredients","label":"Ingredients"},
      {"id":"prepBowl","label":"Prepped bowl"},
      {"id":"mealPot","label":"Home-cooked meal","serving":"mealPlate","emptyAs":"potDirty","radius":0.16},
      {"id":"grillPlatter","label":"Barbecue","serving":"mealPlate","emptyAs":"potDirty","radius":0.16},
      {"id":"mealPlate","label":"Plate of food","emptyAs":"plateDirty"},
      {"id":"plateDirty","label":"Dirty plate","dirty":true},
      {"id":"potDirty","label":"Dirty pot","dirty":true,"radius":0.16}],
    "recipes":[
      {"id":"meal","label":"Make a meal","item":"mealPot","servings":[1,4],"costPerServing":3,"effects":{"hunger":0.9},
       "steps":[{"label":"Get ingredients","at":["ingredients"],"minutes":5,"anim":"fetch","holds":"ingredients"},
                {"label":"Prep","at":["prep"],"minutes":10,"anim":"chop","optional":true,"holds":"prepBowl"},
                {"label":"Cook","at":["stove","microwave"],"minutes":20,"anim":"stir","optional":true}]},
      {"id":"gourmet","label":"Cook a gourmet meal","item":"mealPot","servings":[2,4],"costPerServing":6,"effects":{"hunger":1.0},
       "steps":[{"at":["ingredients"],"minutes":5,"anim":"fetch","holds":"ingredients"},
                {"at":["prep"],"minutes":20,"anim":"chop","holds":"prepBowl"},
                {"at":["stove"],"minutes":45,"anim":"stir"}]},
      {"id":"bbq","label":"Grill dinner","item":"grillPlatter","servings":[1,4],"costPerServing":4,"outdoor":true,"effects":{"hunger":0.9},
       "steps":[{"at":["ingredients"],"minutes":5,"anim":"fetch","holds":"ingredients"},
                {"at":["grill"],"minutes":25,"anim":"flip"}]}],
    "objects":[
      {"id":"fridge","name":"Fridge","price":500,"stations":["ingredients"],"stores":true,"interactions":[
        {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.15},"tags":["food"]},
        {"id":"meal","label":"Make a meal","recipe":"meal","effects":{"fun":0.05},"tags":["food","cooking"],"skills":{}},
        {"id":"gourmet","label":"Cook a gourmet meal","recipe":"gourmet","tags":["food","cooking"]}]},
      {"id":"counter","name":"Counter","price":100,"stations":["prep"],"surface":{"height":0.9,"kind":"counter"}},
      {"id":"vanity","name":"Vanity","price":100,"surface":{"height":0.9,"kind":"counter"}},
      {"id":"nightstand","name":"Nightstand","price":50,"surface":{"height":0.5,"kind":"low"}},
      {"id":"stove","name":"Stove","price":400,"stations":["stove","oven"],"interactions":[
        {"id":"cook","label":"Cook dinner","recipe":"meal","effects":{"fun":0.05},"tags":["food","cooking"]},
        {"id":"wipe","label":"Wipe it down","minutes":30,"autonomous":false,"effects":{"comfort":0.1},"tags":["chores"]}]},
      {"id":"sink","name":"Sink","price":150,"stations":["dishes"],"interactions":[
        {"id":"wash","label":"Wash hands","minutes":4,"autonomous":false,"effects":{"comfort":0.05},"tags":["hygiene"]}]},
      {"id":"dishwasher","name":"Dishwasher","price":400,"stations":["dishes"]},
      {"id":"table","name":"Dining set","price":300,"footprint":[2,1],"slots":2,"surface":{"height":0.75,"kind":"table"},
       "interactions":[{"id":"coffee","label":"Coffee","minutes":20,"pose":"sit","autonomous":false,"effects":{"comfort":0.2},"tags":["food","lounge"]}]},
      {"id":"sofa","name":"Sofa","price":300,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"relax","label":"Relax","minutes":30,"pose":"sit","autonomous":false,"effects":{"comfort":0.3},"tags":["lounge"]}]},
      {"id":"grill","name":"Grill","price":250,"outdoors":true,"stations":["grill"],"interactions":[
        {"id":"cook","label":"Grill dinner","recipe":"bbq","tags":["food","cooking","outdoors"]}]},
      {"id":"picnic","name":"Picnic table","price":300,"footprint":[2,1],"slots":2,"surface":{"height":0.75,"kind":"table"},
       "interactions":[{"id":"picnic","label":"Picnic","minutes":20,"pose":"sit","autonomous":false,"effects":{"fun":0.1},"tags":["food","lounge"]}]}],
    "activities":[{"id":"eat","label":"Eat","icon":"i","tags":["food"]},{"id":"cook","label":"Cook","icon":"i","tags":["cooking"]}],
    "economy":{"startingFunds":1000}}"#;

/// A house (walls round 0..10 × 0..8, a door at the bottom) on a 16 × 14 plot with a garden.
fn town(objects: &str, sims: usize) -> String {
    let names = ["Ada", "Ben", "Cleo", "Dev"];
    let sims: Vec<String> = (0..sims)
        .map(|i| format!(r#"{{"name":"{}","household":0,"x":{}.5,"z":6.5}}"#, names[i], 4 + i))
        .collect();
    format!(
        r#"{{"width":16,"depth":16,
        "walls":[[0,0,10,0],[0,8,10,8],[0,0,0,8],[10,0,10,8]],
        "doors":[{{"x":5,"z":8,"axis":"x"}}],
        "plots":[{{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,12.5]}}],
        "households":[{{"name":"Player","plot":0,"player":true}}],
        "objects":[{objects}],
        "sims":[{}]}}"#,
        sims.join(",")
    )
}

const KITCHEN: &str = r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"counter","x":2,"z":1,"rot":0},
    {"def":"stove","x":3,"z":1,"rot":0},{"def":"sink","x":6,"z":1,"rot":0},
    {"def":"table","x":2,"z":5,"rot":0},{"def":"sofa","x":7,"z":5,"rot":0},
    {"def":"grill","x":12,"z":3,"rot":0},{"def":"picnic","x":12,"z":8,"rot":0}"#;

/// Object ids in `KITCHEN`.
const FRIDGE: u32 = 0;
const COUNTER: u32 = 1;
const STOVE: u32 = 2;
const SINK: u32 = 3;
const GRILL: u32 = 6;
const PICNIC: u32 = 7;

fn world_with(objects: &str, sims: usize, autonomy: bool) -> World {
    let mut w = World::from_json(CONTENT, &town(objects, sims), 3).unwrap();
    w.autonomy = autonomy;
    for s in &mut w.sims {
        s.needs = [1.0; 8];
    }
    w
}

fn ticks(w: &mut World, minutes: f32) {
    for _ in 0..(minutes / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        common::check(w);
    }
}

fn interaction(w: &World, object: u32, id: &str) -> usize {
    w.content.objects[w.objects[object as usize].def].interactions.iter().position(|i| i.id == id).unwrap()
}

fn order(w: &mut World, sim: u32, object: u32, id: &str) {
    let interaction = interaction(w, object, id);
    w.apply(Command::Use { sim, object, interaction }).unwrap();
}

fn cooking(w: &World, sim: usize) -> bool {
    w.sims[sim].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Cook { .. }))
}

/// The snapshot's columns for a resident.
fn row(w: &World, sim: usize) -> Vec<f32> {
    let mut buf = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut buf);
    let at = snapshot::HEADER_LEN + sim * snapshot::SIM_STRIDE;
    buf[at..at + snapshot::SIM_STRIDE].to_vec()
}

fn item_def(w: &World, id: &str) -> usize {
    w.content.item_index(id).unwrap()
}

#[test]
fn a_meal_goes_from_the_fridge_to_the_counter_to_the_stove_and_is_served() {
    let mut w = world_with(KITCHEN, 2, false);
    let funds = w.households[0].funds;
    order(&mut w, 0, STOVE, "cook");
    let mut stations = Vec::new();
    let mut held_between = Vec::new();
    for _ in 0..(90.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        common::check(&w);
        let r = row(&w, 0);
        let object = r[snapshot::sim::OBJECT];
        if cooking(&w, 0) {
            assert_eq!(r[snapshot::sim::RECIPE], w.content.recipe_index("meal").unwrap() as f32 + 1.0);
            if object >= 0.0 && stations.last() != Some(&(object as u32)) {
                stations.push(object as u32);
            }
            if w.sims[0].is_moving() {
                held_between.push(w.sims[0].held.map(|h| w.content.items[h.def].id.clone()));
                assert_eq!(r[snapshot::sim::HELD], w.sims[0].held.map_or(0.0, |h| h.def as f32 + 1.0));
            }
        }
    }
    assert_eq!(stations, [FRIDGE, COUNTER, STOVE], "fridge, counter, then the stove that was clicked");
    assert!(held_between.contains(&Some("ingredients".into())), "carried the ingredients: {held_between:?}");
    assert!(held_between.contains(&Some("prepBowl".into())), "carried the prepped bowl: {held_between:?}");
    // Paid once, for a serving each (two at home).
    assert_eq!(funds - w.households[0].funds, 6, "3 a serving, two servings");
    let pot = w.items.list.iter().find(|it| it.def == item_def(&w, "mealPot")).expect("a pot of food");
    assert!(matches!(pot.at, ItemAt::Surface { .. }), "served on a surface: {:?}", pot.at);
    assert_eq!(pot.servings, 2);
    assert!(!cooking(&w, 0));
}

#[test]
fn clicking_one_of_two_stoves_starts_at_the_fridge_and_cooks_at_that_stove() {
    let kitchen = format!(r#"{KITCHEN},{{"def":"stove","x":8,"z":1,"rot":0}}"#);
    let far = 8;
    let mut w = world_with(&kitchen, 1, false);
    order(&mut w, 0, far, "cook");
    let mut first = None;
    let mut cooked_at = None;
    for _ in 0..(60.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        if let Some(a) = w.sims[0].current()
            && let TaskKind::Cook { step, station: Some(s), .. } = a.task.kind
        {
            first.get_or_insert(s);
            if step == 2 {
                cooked_at = Some(s);
            }
        }
    }
    assert_eq!(first, Some(FRIDGE), "the ingredients first");
    assert_eq!(cooked_at, Some(far), "the stove that was clicked, not the nearer one");
}

#[test]
fn the_household_sits_down_to_dinner_and_the_plates_get_washed() {
    let mut w = world_with(KITCHEN, 4, true);
    for s in &mut w.sims {
        s.needs[0] = 0.25;
    }
    ticks(&mut w, 240.0);
    for s in &w.sims {
        assert!(s.needs[0] > 0.5, "{} is still hungry: {}", s.name, s.needs[0]);
    }
    assert!(
        !w.events.iter().any(|e| e.kind == EventKind::Accident),
        "nobody ordered takeout while dinner was on its way"
    );
    let washed = |w: &World| !w.items.list.iter().any(|it| it.to_clear(&w.content));
    ticks(&mut w, 120.0);
    assert!(washed(&w), "dirty things left: {:?}", w.items.list);
}

#[test]
fn a_dishwasher_does_the_washing_up_too() {
    // No sink: only the dishwasher washes up.
    let kitchen = KITCHEN.replace(r#"{"def":"sink","x":6,"z":1,"rot":0}"#, r#"{"def":"dishwasher","x":6,"z":1,"rot":0}"#);
    let mut w = world_with(&kitchen, 2, true);
    let plate = item_def(&w, "plateDirty");
    for x in [2, 3] {
        w.items.add(sim_core::items::ItemInstance {
            id: 0,
            def: plate,
            at: ItemAt::Surface { x, z: 5, offset: [0, 0] },
            servings: 0,
            quality: 1.0,
            made_at: 0,
            household: 0,
            spoiled: false,
            recipe: None,
        });
    }
    let mut washing = false;
    for _ in 0..(120.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        washing |= w.sims.iter().any(|s| {
            s.current().is_some_and(|a| matches!(a.task.kind, TaskKind::Dishes { station: Some(SINK), .. }) && matches!(a.phase, Phase::Using { .. }))
        });
    }
    assert!(washing, "washed up at the dishwasher");
    assert!(w.items.list.is_empty(), "left: {:?}", w.items.list);
}

#[test]
fn without_a_stove_a_gourmet_meal_is_never_made_but_a_simple_one_is() {
    let kitchen = KITCHEN.replace(r#"{"def":"stove","x":3,"z":1,"rot":0},"#, "");
    let gourmet = 1;
    // Never chosen.
    let mut w = world_with(&kitchen, 2, true);
    for s in &mut w.sims {
        s.needs[0] = 0.1;
    }
    for _ in 0..(180.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        for s in &w.sims {
            if let Some(TaskKind::Cook { recipe, .. }) = s.current().map(|a| a.task.kind) {
                assert_ne!(recipe, gourmet, "{} cooks a gourmet meal without a stove", s.name);
            }
        }
    }
    // Ordered: refused before anything is bought.
    let mut w = world_with(&kitchen, 1, false);
    let funds = w.households[0].funds;
    order(&mut w, 0, FRIDGE, "gourmet");
    ticks(&mut w, 5.0);
    assert!(!cooking(&w, 0) && w.sims[0].held.is_none());
    assert_eq!(w.households[0].funds, funds, "nothing was bought");
    assert!(w.sims[0].planner.thought.is_some_and(|t| t.kind == sim_core::planner::ThoughtKind::NoPlace));
    // A simple meal is made all the same (on the counter, no stove).
    order(&mut w, 0, FRIDGE, "meal");
    ticks(&mut w, 60.0);
    assert!(w.items.list.iter().any(|it| it.def == item_def(&w, "mealPot")), "{:?}", w.items.list);
}

#[test]
fn a_kitchen_with_only_a_fridge_keeps_the_leftovers_in_it() {
    let mut w = world_with(r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"sink","x":6,"z":1,"rot":0}"#, 3, false);
    order(&mut w, 0, 0, "meal");
    ticks(&mut w, 50.0);
    let pot = w.items.list.iter().find(|it| it.servings > 0).expect("a meal");
    assert!(matches!(pot.at, ItemAt::Stored { x: 1, z: 1 }), "in the fridge: {:?}", pot.at);
    assert_eq!(pot.servings, 3);
    // Made at the fridge (both later steps done there), a little plainer.
    assert!(pot.quality < 1.0);
}

#[test]
fn a_busy_stove_means_waiting_or_the_other_stove() {
    // Two stoves, one busy: the other at once.
    let kitchen = format!(r#"{KITCHEN},{{"def":"stove","x":8,"z":1,"rot":0}}"#);
    let mut w = world_with(&kitchen, 2, false);
    w.sims[1].needs[2] = 0.0;
    order(&mut w, 1, STOVE, "wipe");
    ticks(&mut w, 1.0);
    order(&mut w, 0, FRIDGE, "meal");
    let mut used = None;
    for _ in 0..(40.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        if let Some(TaskKind::Cook { step: 2, station: Some(s), .. }) = w.sims[0].current().map(|a| a.task.kind) {
            used = Some(s);
        }
    }
    assert_eq!(used, Some(8), "the free stove");

    // One stove, busy: they wait their turn (then cook there, or do without after a while).
    let mut w = world_with(KITCHEN, 2, false);
    w.sims[1].needs[2] = 0.0;
    order(&mut w, 1, STOVE, "wipe");
    ticks(&mut w, 1.0);
    order(&mut w, 0, FRIDGE, "meal");
    let mut queued = false;
    for _ in 0..(60.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        queued |= w.sims[0].current().is_some_and(|a| matches!(a.phase, Phase::Queued { .. }));
    }
    assert!(queued, "waited for the stove");
    assert!(w.items.list.iter().any(|it| it.servings > 0), "and made dinner in the end");
}

#[test]
fn a_stove_that_broke_on_the_way_is_left_for_the_other_one() {
    let kitchen = format!(r#"{KITCHEN},{{"def":"stove","x":8,"z":1,"rot":0}}"#);
    let mut w = world_with(&kitchen, 1, false);
    order(&mut w, 0, FRIDGE, "meal");
    let (mut broke, mut used) = (None, None);
    for _ in 0..(60.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        let Some(a) = w.sims[0].current() else { continue };
        if let TaskKind::Cook { step: 2, station: Some(s), .. } = a.task.kind {
            // On the way to the stove, someone else wears it out.
            if broke.is_none() && matches!(a.phase, Phase::Routing { .. }) {
                w.objects[s as usize].wear = 1.0;
                broke = Some(s);
            }
            if matches!(a.phase, Phase::Using { .. }) {
                assert!(!w.objects[s as usize].broken(), "cooking at a broken stove");
                used = Some(s);
            }
        }
    }
    let broke = broke.expect("went to a stove");
    assert!(used.is_some_and(|s| s != broke), "cooked at the other stove: {used:?}");
    assert!(w.items.list.iter().any(|it| it.servings > 0), "and dinner was made");
}

#[test]
fn cancelling_after_the_fridge_drops_the_ingredients_and_frees_the_counter() {
    let mut w = world_with(KITCHEN, 1, false);
    let funds = w.households[0].funds;
    order(&mut w, 0, STOVE, "cook");
    while !w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Cook { step: 1, .. })) {
        w.tick_once();
    }
    assert!(w.sims[0].held.is_some(), "holds the ingredients");
    assert!(w.objects[COUNTER as usize].in_use(), "the counter is theirs");
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    ticks(&mut w, 1.0);
    assert!(w.sims[0].held.is_none(), "the ingredients are gone");
    assert!(!w.objects[COUNTER as usize].in_use());
    assert!(w.items.list.is_empty(), "no food: {:?}", w.items.list);
    assert_eq!(funds - w.households[0].funds, 3, "the ingredients were bought");
}

#[test]
fn food_goes_off_and_makes_a_mess() {
    let mut w = world_with(KITCHEN, 1, false);
    order(&mut w, 0, STOVE, "cook");
    ticks(&mut w, 50.0);
    let pot = w.items.list.iter().position(|it| it.servings > 0).expect("dinner");
    assert!(w.items.list[pot].edible());
    let (x, z) = w.items.list[pot].at.tile();
    let dirt = w.dirt[w.lot.tile_index(x, z)];
    w.items.list[pot].made_at = 0;
    w.tick = 9 * 60 * 20;
    ticks(&mut w, 30.0);
    let it = &w.items.list[pot];
    assert!(it.spoiled && !it.edible(), "gone off: {it:?}");
    assert!(w.dirt[w.lot.tile_index(x, z)] > dirt, "and the counter's getting dirty");
}

#[test]
fn a_barbecue_is_served_outdoors() {
    let mut w = world_with(KITCHEN, 1, false);
    order(&mut w, 0, GRILL, "cook");
    ticks(&mut w, 60.0);
    let platter = w.items.list.iter().find(|it| it.def == item_def(&w, "grillPlatter")).expect("a barbecue");
    let host = sim_core::items::holder(&w.content, &w.objects, &platter.at);
    assert_eq!(host, Some(PICNIC), "on the picnic table: {:?}", platter.at);
}

/// Dinner served on the dining table (ordered with nobody else around, so nobody eats it).
fn dinner_on_the_table() -> (World, u32) {
    let kitchen = KITCHEN.replace(r#"{"def":"counter","x":2,"z":1,"rot":0},"#, "");
    let mut w = world_with(&kitchen, 1, false);
    order(&mut w, 0, 0, "meal");
    ticks(&mut w, 50.0);
    let table = 3;
    assert_eq!(w.content.objects[w.objects[table].def].id, "table");
    let pot = w.items.list.iter().find(|it| it.servings > 0).expect("dinner").clone();
    assert_eq!(sim_core::items::holder(&w.content, &w.objects, &pot.at), Some(table as u32), "{:?}", pot.at);
    (w, pot.id)
}

#[test]
fn food_moves_with_its_table_goes_when_it_is_sold_and_comes_back_on_undo() {
    let (mut w, pot) = dinner_on_the_table();
    let table = 3;
    let before = w.items.get(pot).unwrap().at;
    w.apply(Command::MoveObject { household: 0, object: table, x: 2, z: 3, rot: 0, turn: None, offset: None, size: None, tall: None }).unwrap();
    let moved = w.items.get(pot).expect("still there").at;
    let host = sim_core::items::holder(&w.content, &w.objects, &moved).expect("on something");
    assert_eq!(w.content.objects[w.objects[host as usize].def].id, "table", "on the table where it went: {moved:?}");
    assert_ne!(moved, before);
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!(w.items.get(pot).unwrap().at, before, "undo puts it back");
    w.apply(Command::Sell { household: 0, object: table }).unwrap();
    assert!(w.items.get(pot).is_none(), "sold with the table");
    w.apply(Command::Undo { household: 0 }).unwrap();
    assert_eq!(w.items.get(pot).unwrap().at, before, "and back with it");
}

#[test]
fn saving_mid_recipe_and_with_food_about_keeps_it_all() {
    let mut w = world_with(KITCHEN, 2, false);
    order(&mut w, 0, STOVE, "cook");
    while !w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Cook { step: 1, .. })) {
        w.tick_once();
    }
    let json = w.save_json();
    assert!(json.contains("\"held\""), "carries the ingredients");
    let mut loaded = World::from_save_json(CONTENT, &json).unwrap();
    assert_eq!(loaded.save_json(), json, "saves the same again");
    common::check(&loaded);
    // It goes on: the recipe at the counter, then the stove, and dinner's served once.
    let funds = loaded.households[0].funds;
    ticks(&mut loaded, 50.0);
    assert_eq!(loaded.households[0].funds, funds, "paid for already");
    assert!(loaded.items.list.iter().any(|it| it.servings == 2), "{:?}", loaded.items.list);
    // With food and dirty dishes about.
    let plate = item_def(&loaded, "plateDirty");
    loaded.items.add(sim_core::items::ItemInstance {
        id: 0,
        def: plate,
        at: ItemAt::Floor { x: 5, z: 6, offset: [10, -20] },
        servings: 0,
        quality: 1.0,
        made_at: loaded.tick,
        household: 0,
        spoiled: false,
        recipe: None,
    });
    let json = loaded.save_json();
    let again = World::from_save_json(CONTENT, &json).unwrap();
    assert_eq!((&again.items.list, again.items.next_id), (&loaded.items.list, loaded.items.next_id));
    assert_eq!(again.save_json(), json);
}

#[test]
fn version_18_saves_load_without_food() {
    let w = world_with(KITCHEN, 1, false);
    let mut save: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    save["version"] = 18.into();
    let w = World::from_save_json(CONTENT, &save.to_string()).unwrap();
    assert!(w.items.list.is_empty());
}

#[test]
fn cooking_is_deterministic() {
    let run = || {
        let mut w = world_with(KITCHEN, 4, true);
        for (i, s) in w.sims.iter_mut().enumerate() {
            s.needs[0] = 0.2 + 0.1 * i as f32;
        }
        for _ in 0..(300.0 / sim_core::MINUTES_PER_TICK) as u32 {
            w.tick_once();
        }
        w.save_json()
    };
    assert_eq!(run(), run());
}

#[test]
fn the_snapshot_says_what_is_carried_cooked_and_when_food_changes() {
    let mut w = world_with(KITCHEN, 1, false);
    let mut buf = vec![0.0; snapshot::CAPACITY];
    snapshot::write(&w, &mut buf);
    let version = buf[snapshot::header::ITEMS_VERSION];
    order(&mut w, 0, STOVE, "cook");
    let mut saw = (false, false);
    for _ in 0..(50.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        let r = row(&w, 0);
        saw.0 |= r[snapshot::sim::HELD] == item_def(&w, "ingredients") as f32 + 1.0;
        saw.1 |= r[snapshot::sim::RECIPE] == 1.0 && r[snapshot::sim::ACTION] == w.content.animations.iter().position(|a| a == "stir").unwrap() as f32;
    }
    assert!(saw.0, "carrying the ingredients");
    assert!(saw.1, "stirring the meal");
    snapshot::write(&w, &mut buf);
    assert_ne!(buf[snapshot::header::ITEMS_VERSION], version, "dinner's served: the items changed");
    let layout: serde_json::Value = serde_json::from_str(&snapshot::layout_json(&w.content)).unwrap();
    assert_eq!(layout["sim"]["held"], snapshot::sim::HELD);
    assert_eq!(layout["header"]["itemsVersion"], snapshot::header::ITEMS_VERSION);
    let items: serde_json::Value = serde_json::from_str(&sim_core::view::items_json(&w)).unwrap();
    assert_eq!(items[0]["def"], "mealPot");
    assert_eq!(items[0]["y"], 0.9, "on the counter");
}

#[test]
fn the_player_can_have_a_resident_eat_or_clear_away_food() {
    let mut w = world_with(KITCHEN, 2, false);
    order(&mut w, 0, STOVE, "cook");
    ticks(&mut w, 50.0);
    let pot = w.items.list.iter().find(|it| it.servings > 0).unwrap().id;
    w.sims[1].needs[0] = 0.3;
    w.apply(Command::UseItem { sim: 1, item: pot, action: sim_core::command::ItemAction::Eat }).unwrap();
    ticks(&mut w, 40.0);
    assert!(w.sims[1].needs[0] > 0.8, "ate: {}", w.sims[1].needs[0]);
    assert_eq!(w.items.get(pot).map(|it| it.servings), Some(1), "one serving left");
    w.apply(Command::UseItem { sim: 1, item: pot, action: sim_core::command::ItemAction::Clear }).unwrap();
    ticks(&mut w, 40.0);
    assert!(w.items.get(pot).is_none(), "cleared away");
}

/// Each step of a recipe ordered from `object`, as it starts: (step, station, minutes).
fn step_minutes(w: &mut World, object: u32, id: &str) -> Vec<(u8, u32, u16)> {
    order(w, 0, object, id);
    let mut steps: Vec<(u8, u32, u16)> = Vec::new();
    for _ in 0..(90.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        common::check(w);
        if let Some(a) = w.sims[0].current()
            && let (TaskKind::Cook { step, station: Some(s), minutes, .. }, Phase::Using { .. }) = (a.task.kind, &a.phase)
            && steps.last().is_none_or(|l| l.0 != step)
        {
            steps.push((step, s, minutes));
        }
    }
    steps
}

#[test]
fn without_a_counter_the_chopping_happens_at_the_stove_not_the_fridge() {
    // The shipped meal: ingredients 3 min, prep 10 (optional), cook 20.
    let content = common::content();
    let all: serde_json::Value = serde_json::from_str(&content).unwrap();
    let meal = all["recipes"].as_array().unwrap().iter().find(|r| r["id"] == "meal").unwrap();
    let minutes: Vec<f64> = meal["steps"].as_array().unwrap().iter().map(|s| s["minutes"].as_f64().unwrap()).collect();
    assert_eq!(minutes, [3.0, 10.0, 20.0]);
    let traits = format!(r#""household":0,"traits":[{}]"#, all["traits"][0]["id"]);
    let world = |objects: &str| {
        let mut w = World::from_json(&content, &town(objects, 1).replace(r#""household":0"#, &traits), 3).unwrap();
        w.autonomy = false;
        w.sims[0].needs = [1.0; 8];
        w
    };
    // Fridge and stove, no counter: a quick trip to the fridge, then chopping and cooking at the stove.
    let mut w = world(r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"stoveGas","x":3,"z":1,"rot":0}"#);
    assert_eq!(step_minutes(&mut w, 1, "cook"), [(0, 0, 3), (2, 1, 30)]);
    assert!(w.items.list.iter().any(|it| it.servings > 0), "dinner's made: {:?}", w.items.list);
    // With a counter, each step at its own station.
    let mut w = world(
        r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"counter","x":2,"z":1,"rot":0},{"def":"stoveGas","x":3,"z":1,"rot":0}"#,
    );
    assert_eq!(step_minutes(&mut w, 2, "cook"), [(0, 0, 3), (1, 1, 10), (2, 2, 20)]);
    // Only a fridge: the whole meal is made there.
    let mut w = world(r#"{"def":"fridge","x":1,"z":1,"rot":0}"#);
    assert_eq!(step_minutes(&mut w, 0, "meal"), [(0, 0, 33)]);
    assert!(w.items.list.iter().any(|it| it.servings > 0), "dinner's made: {:?}", w.items.list);
}

#[test]
fn the_last_serving_from_the_fridge_brings_the_empty_pot_out_to_be_washed() {
    use sim_core::command::ItemAction;
    for (objects, on_counter) in [
        (r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"counter","x":2,"z":1,"rot":0},{"def":"sink","x":6,"z":1,"rot":0}"#, true),
        (r#"{"def":"fridge","x":1,"z":1,"rot":0},{"def":"sink","x":6,"z":1,"rot":0}"#, false),
    ] {
        let mut w = world_with(objects, 1, false);
        order(&mut w, 0, FRIDGE, "meal");
        ticks(&mut w, 50.0);
        // Leftovers in the fridge, one serving left.
        let pot = w.items.list.iter_mut().find(|it| it.servings > 0).expect("a meal");
        let id = pot.id;
        pot.at = ItemAt::Stored { x: 1, z: 1 };
        pot.servings = 1;
        w.sims[0].needs[0] = 0.3;
        w.apply(Command::UseItem { sim: 0, item: id, action: ItemAction::Eat }).unwrap();
        ticks(&mut w, 40.0);
        let dirty = item_def(&w, "potDirty");
        let pot = w.items.list.iter().find(|it| it.def == dirty).expect("the empty pot, to be washed up");
        assert!(!matches!(pot.at, ItemAt::Stored { .. }), "not left in the fridge: {:?}", pot.at);
        assert_eq!(matches!(pot.at, ItemAt::Surface { .. }), on_counter, "on the counter if there is one: {:?}", pot.at);
        // Out where it can be seen, and cleared away.
        let pot = pot.id;
        w.apply(Command::UseItem { sim: 0, item: pot, action: ItemAction::Clear }).unwrap();
        ticks(&mut w, 30.0);
        assert!(w.items.get(pot).is_none(), "washed up");
    }
}

/// A house clear of the lot's edge (walls round 1..11 × 1..9; one touching the edge would be
/// outdoors) split down the middle by a wall at x = 6 (a door at z = 7): a kitchen on the left
/// (the stove at 4,2) and a bathroom on the right, its vanity just through the wall from the stove.
fn split_house(kitchen: &str) -> World {
    let town = format!(
        r#"{{"width":16,"depth":16,
        "walls":[[1,1,11,1],[1,9,11,9],[1,1,1,9],[11,1,11,9],[6,1,6,9]],
        "doors":[{{"x":3,"z":9,"axis":"x"}},{{"x":6,"z":7,"axis":"z"}}],
        "plots":[{{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,12.5]}}],
        "households":[{{"name":"Player","plot":0,"player":true}}],
        "objects":[{kitchen},{{"def":"vanity","x":6,"z":2,"rot":0}}],
        "sims":[{{"name":"Ada","household":0,"x":4.5,"z":7.5}}]}}"#
    );
    let mut w = World::from_json(CONTENT, &town, 3).unwrap();
    w.autonomy = false;
    for s in &mut w.sims {
        s.needs = [1.0; 8];
    }
    assert_ne!(w.lot.room_at(5, 2), w.lot.room_at(6, 2), "the wall makes two rooms");
    w
}

#[test]
fn cooks_put_the_food_down_in_the_kitchen_not_through_the_wall() {
    // A counter across the kitchen, further than the vanity next door (and than a nightstand by
    // the stove, if there is one): the food goes on the counter.
    let kitchen = r#"{"def":"fridge","x":2,"z":2,"rot":0},{"def":"stove","x":4,"z":2,"rot":0},{"def":"counter","x":2,"z":6,"rot":0}"#;
    for objects in [kitchen.to_string(), format!(r#"{kitchen},{{"def":"nightstand","x":5,"z":2,"rot":0}}"#)] {
        let mut w = split_house(&objects);
        order(&mut w, 0, 1, "cook");
        ticks(&mut w, 60.0);
        let pot = w.items.list.iter().find(|it| it.servings > 0).expect("dinner");
        assert_eq!(sim_core::items::holder(&w.content, &w.objects, &pot.at), Some(2), "on the kitchen counter: {:?}", pot.at);
    }
    // Nowhere to put it in the kitchen: in the fridge, not next door.
    let mut w = split_house(r#"{"def":"fridge","x":2,"z":2,"rot":0},{"def":"stove","x":4,"z":2,"rot":0}"#);
    order(&mut w, 0, 1, "cook");
    ticks(&mut w, 60.0);
    let pot = w.items.list.iter().find(|it| it.servings > 0).expect("dinner");
    assert!(matches!(pot.at, ItemAt::Stored { x: 2, z: 2 }), "in the kitchen fridge: {:?}", pot.at);
}
