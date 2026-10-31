//! The held foods (`docs/missing-items.md` §4): a pot of food, a tasting board, a pizza, a pie, a tray of
//! baked treats, a sandwich and a barbecue platter are each carried out of the kitchen in the resident's
//! hand, and a serving taken from a pizza is a slice. Each is the item the shipped pack has a model for
//! (`cargo check-glb`), so the game draws a held model rather than an empty hand.
//!
//! The dish is carried only as far as the household's table, and only if the table is one they can sit at:
//! a bare table has a top but no seats, so `cooking::serving_table` skips it and the dish is put down on a
//! free surface within `cooking.serveRange` instead. That is why the house below has two chairs pulled up
//! to the dining table, and a picnic table out in the yard for the two recipes cooked outdoors.

mod common;

use sim_core::items::ItemAt;
use sim_core::snapshot::{self, HEADER_LEN, sim};
use sim_core::{Command, World};

/// A one-room house with the shipped appliances each recipe asks for, and a yard for the two that are
/// cooked outdoors.
const TOWN: &str = r#"{
    "width":22,"depth":22,
    "plots":[{"name":"Home","x":0,"z":0,"w":22,"d":22,"entry":[10.5,1.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[1,1,12,1],[1,1,1,10],[1,10,12,10],[12,1,12,10]],
    "doors":[{"x":6,"z":10,"axis":"x"}],
    "objects":[
      {"def":"fridge","x":2,"z":2},
      {"def":"counter","x":3,"z":2},
      {"def":"stoveElectric","x":5,"z":2},
      {"def":"tableDining4","x":10,"z":8},
      {"def":"chairWood","x":10,"z":7},
      {"def":"chairWood","x":11,"z":7},
      {"def":"foodie.chefsIsland","x":8,"z":7},
      {"def":"foodie.pizzaOven","x":16,"z":4},
      {"def":"kind.neighbourlyOven","x":16,"z":8},
      {"def":"grill","x":16,"z":12},
      {"def":"picnicTable","x":19,"z":12}],
    "sims":[{"name":"Ada","x":8,"z":4,"traits":["cheerful"]}]}"#;

fn world() -> World {
    let content = common::content();
    let mut w = World::from_json(&content, TOWN, 3).unwrap();
    w.autonomy = false;
    for s in &mut w.sims {
        s.needs = [1.0; 8];
    }
    w
}

/// The object that offers a recipe, and the index of that interaction.
fn offers(w: &World, recipe: &str) -> (u32, usize) {
    let want = w.content.recipe_index(recipe).unwrap();
    for (i, o) in w.objects.iter().enumerate() {
        let def = &w.content.objects[o.def];
        if let Some(k) = def
            .interactions
            .iter()
            .position(|it| it.recipe == Some(want))
        {
            return (i as u32, k);
        }
    }
    panic!("no object in the lot offers the {recipe} recipe");
}

/// What the snapshot says the resident is holding, or `None`.
fn held(w: &World) -> Option<String> {
    let mut buf = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut buf);
    let n = buf[HEADER_LEN + sim::HELD];
    if n <= 0.0 {
        return None;
    }
    Some(w.content.items[n as usize - 1].id.clone())
}

/// Whether the dish is standing on the dining table at `(10, 8)` (2 × 1 tiles), which is where the
/// carrying out of the kitchen ends.
fn where_is(w: &World, dish: &str) -> String {
    let def = w.content.item_index(dish).unwrap();
    w.items
        .list
        .iter()
        .filter(|it| it.def == def)
        .map(|it| format!("{:?}", it.at))
        .collect::<Vec<_>>()
        .join(", ")
}

fn on_the_table(w: &World, dish: &str) -> bool {
    let def = w.content.item_index(dish).unwrap();
    w.items.list.iter().any(|it| {
        let (x, z) = it.at.tile();
        it.def == def
            && matches!(it.at, ItemAt::Surface { .. })
            && (10..12).contains(&x)
            && (8..9).contains(&z)
    })
}

/// Whether the dish is standing on the picnic table at `(19, 12)` (3 × 2 tiles).
fn on_the_picnic(w: &World, dish: &str) -> bool {
    let def = w.content.item_index(dish).unwrap();
    w.items.list.iter().any(|it| {
        let (x, z) = it.at.tile();
        it.def == def
            && matches!(it.at, ItemAt::Surface { .. })
            && (19..22).contains(&x)
            && (12..14).contains(&z)
    })
}

/// Cook a recipe and give back every item the resident held on the way, in order.
fn cook(w: &mut World, recipe: &str, minutes: u32) -> Vec<String> {
    let (object, interaction) = offers(w, recipe);
    w.apply(Command::Use {
        sim: 0,
        object,
        interaction,
    })
    .unwrap();
    let mut carried: Vec<String> = Vec::new();
    for _ in 0..(u64::from(minutes) * sim_core::TICKS_PER_MINUTE) {
        w.tick_once();
        common::check(w);
        if let Some(id) = held(w)
            && carried.last() != Some(&id)
        {
            carried.push(id);
        }
    }
    carried
}

#[test]
fn a_tasting_board_is_carried_out_of_the_kitchen() {
    let mut w = world();
    let carried = cook(&mut w, "tastingMenu", 120);
    assert!(
        carried.last() == Some(&"tastingBoard".to_string()),
        "carried the board: {carried:?}"
    );
    assert!(
        on_the_table(&w, "tastingBoard"),
        "the board is on the table"
    );
}

#[test]
fn a_pizza_is_carried_out_of_the_kitchen() {
    let mut w = world();
    let carried = cook(&mut w, "pizza", 120);
    assert!(
        carried.last() == Some(&"pizza".to_string()),
        "carried the pizza: {carried:?}"
    );
    // A wood-fired pizza is cooked outdoors: it goes out to the picnic table.
    assert!(
        on_the_picnic(&w, "pizza"),
        "the pizza is at {}",
        where_is(&w, "pizza")
    );
}

#[test]
fn a_pie_is_carried_out_of_the_kitchen() {
    let mut w = world();
    let carried = cook(&mut w, "pie", 120);
    assert!(
        carried.last() == Some(&"pieDish".to_string()),
        "carried the pie: {carried:?}"
    );
    assert!(
        on_the_table(&w, "pieDish"),
        "the pie is at {}",
        where_is(&w, "pieDish")
    );
}

#[test]
fn a_sandwich_is_carried_out_of_the_kitchen() {
    let mut w = world();
    let carried = cook(&mut w, "sandwich", 60);
    assert!(
        carried.last() == Some(&"sandwich".to_string()),
        "carried the sandwich: {carried:?}"
    );
    // A sandwich is eaten where it is made (`eatNow`): the dirty plate is what is left on the table.
    assert!(
        on_the_table(&w, "plateDirty"),
        "the plate is at {}",
        where_is(&w, "plateDirty")
    );
}

#[test]
fn a_tray_of_baked_treats_is_carried_out_of_the_kitchen() {
    let mut w = world();
    let carried = cook(&mut w, "bakedTreats", 120);
    assert!(
        carried.last() == Some(&"bakingTray".to_string()),
        "carried the tray: {carried:?}"
    );
    assert!(on_the_table(&w, "bakingTray"), "the tray is on the table");
}

#[test]
fn a_barbecue_is_carried_out_of_the_kitchen() {
    let mut w = world();
    let carried = cook(&mut w, "bbq", 120);
    assert!(
        carried.last() == Some(&"grillPlatter".to_string()),
        "carried the platter: {carried:?}"
    );
    assert!(
        on_the_picnic(&w, "grillPlatter"),
        "the platter is at {}",
        where_is(&w, "grillPlatter")
    );
}

#[test]
fn a_meal_from_the_fridge_is_carried_to_the_table() {
    let mut w = world();
    // Hungry, so that the pot of food is dished out onto a plate and eaten at the table.
    w.sims[0].needs[0] = 0.2;
    let carried = cook(&mut w, "meal", 200);
    assert_eq!(
        carried,
        ["ingredients", "prepBowl", "mealPot", "mealPlate"],
        "the makings from the fridge, the bowl, the pot to the table, then a plate"
    );
    assert!(
        on_the_table(&w, "mealPlate") || on_the_table(&w, "plateDirty"),
        "the plate is at {}",
        where_is(&w, "mealPlate")
    );
}

#[test]
fn a_serving_taken_from_a_pizza_is_a_slice_in_the_hand() {
    let mut w = world();
    // Hungry, so that once the pizza is on the table they have some of it.
    w.sims[0].needs[0] = 0.2;
    let carried = cook(&mut w, "pizza", 200);
    assert!(
        carried.last() == Some(&"pizzaSlice".to_string()),
        "carried a slice: {carried:?}"
    );
}
