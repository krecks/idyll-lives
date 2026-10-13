//! Objects and the things on them, from the release review (R2): each test failed (or
//! panicked) on 4cde174 and names the finding that fixed it. Selling and moving renumber
//! objects (ids are places in `World::objects`), so everything that names one by id has to
//! follow.

mod common;

use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, MINUTES_PER_TICK, World};

/// An open 16×14 plot with one resident.
const TOWN: &str = r#"{"width":16,"depth":16,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,13.5]}],
    "households":[{"name":"Player","plot":0,"player":true,"funds":100000}],
    "sims":[{"name":"Ada","household":0,"x":8.5,"z":8.5,"traits":["bookworm"]}]}"#;

fn world() -> World {
    let mut w = World::from_json(&common::content(), TOWN, 1).unwrap();
    w.autonomy = false;
    w
}

fn buy(w: &mut World, def: &str, x: i32, z: i32) -> u32 {
    w.apply(Command::Buy { household: 0, object: def.into(), at: Some([x, z, 0]), style: None, turn: None, offset: None, size: None, tall: None })
        .unwrap_or_else(|e| panic!("{def}: {e}"));
    w.objects.len() as u32 - 1
}

fn need(w: &World, id: &str) -> usize {
    w.content.needs.iter().position(|n| n.id == id).unwrap()
}

fn interaction(w: &World, object: u32, id: &str) -> usize {
    w.content.objects[w.objects[object as usize].def].interactions.iter().position(|i| i.id == id).unwrap()
}

/// Uses the toilet until the tick it's done with it: then washing hands at the sink is the
/// resident's `follow_up` (not started until the next tick).
fn finish_on_the_toilet(w: &mut World, toilet: u32) {
    let bladder = need(w, "bladder");
    w.sims[0].needs[bladder] = 0.3;
    let use_it = interaction(w, toilet, "use");
    w.apply(Command::Use { sim: 0, object: toilet, interaction: use_it }).unwrap();
    let mut on = false;
    for _ in 0..(60.0 / MINUTES_PER_TICK) as u32 {
        w.tick_once();
        let using = w.sims[0].current().is_some_and(|a| matches!(a.phase, Phase::Using { .. }) && matches!(a.task.kind, TaskKind::Use { object, .. } if object == toilet));
        if using {
            on = true;
        } else if on {
            assert!(w.sims[0].current().is_none(), "done with the toilet, nothing started yet");
            return;
        }
    }
    panic!("never finished on the toilet");
}

/// R2-01. `remove_object` shifts the ids in the current task and the queue but not in
/// `Sim::follow_up`. Selling anything with a lower id in the tick between the toilet and
/// washing hands leaves the follow-up pointing at the next object: here a lamp with no
/// interactions, so `start_task` indexes `interactions[0]` (`ObjectDef::slots_for`) and panics.
#[test]
fn selling_an_earlier_object_while_a_follow_up_is_pending_does_not_panic() {
    let mut w = world();
    let first = buy(&mut w, "lamp", 1, 11);
    let toilet = buy(&mut w, "toilet", 3, 3);
    let _sink = buy(&mut w, "sink", 5, 3);
    let _lamp = buy(&mut w, "lamp", 12, 11);
    finish_on_the_toilet(&mut w, toilet);
    // The player sells the first lamp now (the worker applies commands between steps).
    w.apply(Command::Sell { household: 0, object: first }).unwrap();
    for _ in 0..40 {
        w.tick_once();
    }
    common::check(&w);
}

/// R2-01 (no panic, wrong object): the stale follow-up names whatever object took the id. With
/// a second sink after the first (another room, further away), selling an earlier object
/// sends the resident to wash at the far sink instead of the near one... and with a toilet
/// after the sink, to "wash" (index 0) on the toilet, i.e. to use the toilet again.
#[test]
fn a_pending_follow_up_still_names_the_sink_after_an_earlier_sale() {
    let mut w = world();
    let first = buy(&mut w, "lamp", 1, 11);
    let toilet = buy(&mut w, "toilet", 3, 3);
    let sink = buy(&mut w, "sink", 5, 3);
    let _other_toilet = buy(&mut w, "toilet", 12, 3);
    finish_on_the_toilet(&mut w, toilet);
    w.apply(Command::Sell { household: 0, object: first }).unwrap();
    let sink = sink - 1;
    let mut used = None;
    for _ in 0..(30.0 / MINUTES_PER_TICK) as u32 {
        w.tick_once();
        if let Some(a) = w.sims[0].current()
            && let TaskKind::Use { object, .. } = a.task.kind
        {
            used = Some(object);
            break;
        }
    }
    let def = |o: u32| w.content.objects[w.objects[o as usize].def].id.clone();
    assert_eq!(used.map(def), Some(def(sink)), "washes at the sink, not the next object");
}

/// Puts a pot of the base game's meal (4 servings) on a free spot of `table`.
fn meal_on(w: &mut World, table: u32) -> u32 {
    use sim_core::items::{ItemAt, ItemInstance};
    let def = w.content.item_index("mealPot").unwrap();
    let radius = w.content.items[def].radius;
    let occupied = w.items.occupancy(&w.content);
    let centre = w.objects[table as usize].centre(&w.content);
    let spot = sim_core::surface::free_surface_spot_near(&w.content, &w.objects, &[table], centre, 3.0, 0, radius, &occupied).expect("room on the table");
    let recipe = w.content.recipes.iter().position(|r| r.id == "meal");
    w.items.add(ItemInstance { id: 0, def, at: ItemAt::on(&spot), servings: 4, quality: 1.0, made_at: w.tick, household: 0, spoiled: false, recipe })
}

/// R2-02. `World::sell` drops every item on any tile of the sold object (`drop_items_with`
/// matches tiles, not the host): selling a fruit bowl standing on the dining table, or the rug
/// under it, throws away the dinner on the table.
#[test]
#[ignore = "release review: fails until R2-02 is fixed"]
fn selling_a_fruit_bowl_or_a_rug_keeps_the_dinner_on_the_table() {
    let mut lost = Vec::new();
    for (extra, at) in [("fruitBowl", None), ("rugLarge", Some([4, 4, 0]))] {
        let mut w = world();
        let rug = at.map(|[x, z, r]| {
            w.apply(Command::Buy { household: 0, object: extra.into(), at: Some([x, z, r]), style: None, turn: None, offset: None, size: None, tall: None }).unwrap();
            w.objects.len() as u32 - 1
        });
        let table = buy(&mut w, "tableDining4", 5, 5);
        let meal = meal_on(&mut w, table);
        let thing = match rug {
            Some(r) => r,
            None => {
                w.apply(Command::Buy { household: 0, object: extra.into(), at: None, style: None, turn: None, offset: None, size: None, tall: None }).unwrap();
                w.objects.len() as u32 - 1
            }
        };
        let (o, (mx, mz)) = (&w.objects[thing as usize], w.items.get(meal).unwrap().at.tile());
        let (ow, od) = o.size(&w.content);
        assert!((o.x..o.x + ow).contains(&mx) && (o.z..o.z + od).contains(&mz), "the {extra} shares the meal's tile");
        w.apply(Command::Sell { household: 0, object: thing }).unwrap();
        if w.items.get(meal).is_none() {
            lost.push(extra);
        }
    }
    assert!(lost.is_empty(), "selling these threw away the meal on the table: {lost:?}");
}

/// R2-02 (moving): `carry_items` also matches tiles, so moving the fruit bowl to another table
/// takes the dinner with it.
#[test]
#[ignore = "release review: fails until R2-02 is fixed"]
fn moving_a_fruit_bowl_leaves_the_dinner_on_the_table() {
    let mut w = world();
    let table = buy(&mut w, "tableDining4", 5, 5);
    let other = buy(&mut w, "tableDining4", 5, 9);
    let meal = meal_on(&mut w, table);
    let was = w.items.get(meal).unwrap().at;
    w.apply(Command::Buy { household: 0, object: "fruitBowl".into(), at: None, style: None, turn: None, offset: None, size: None, tall: None }).unwrap();
    let bowl = w.objects.len() as u32 - 1;
    assert_eq!(sim_core::surface::host_of(&w.content, &w.objects, &w.objects[bowl as usize]), Some(table), "the bowl is on the table");
    let radius = w.content.objects[w.objects[bowl as usize].def].radius;
    let centre = w.objects[other as usize].centre(&w.content);
    let spot = sim_core::surface::free_surface_spot_near(&w.content, &w.objects, &[other], centre, 3.0, 0, radius, &[]).expect("room on the other table");
    w.apply(Command::MoveObject { household: 0, object: bowl, x: spot.x, z: spot.z, rot: 0, turn: None, offset: Some(spot.offset), size: None, tall: None }).unwrap();
    assert_eq!(w.items.get(meal).map(|it| it.at), Some(was), "the meal stays where it was");
}

/// R2-05 (content). `TraitRaw` has no `rename_all = "camelCase"`, so the shipped traits'
/// `startingSkills` are dropped by serde and every resident starts with no skills from traits.
#[test]
#[ignore = "release review: fails until R2-05 is fixed"]
fn traits_give_their_starting_skills() {
    let w = world();
    let cooking = w.content.skill_index("cooking").unwrap();
    let intelligence = w.content.skill_index("intelligence").unwrap();
    let skills = w.content.starting_skills(&["bookworm".to_string()]);
    assert!(skills[intelligence] >= 2.0, "a bookworm starts with intelligence 2 (base.json), got {}", skills[intelligence]);
    let chef = w.content.traits.iter().find(|t| t.starting_skills[cooking] > 0.0);
    assert!(chef.is_some(), "some trait starts with cooking (base.json)");
}

/// R2-04 (known: commands by position aren't idempotent). Holding R in Build mode sends
/// `moveObject` again with the id the view had (`session.ts:543`): the first rotates the rug and
/// renumbers it last, the second moves whatever took its id (the lamp) onto the rug.
#[test]
#[ignore = "release review: fails until R2-04 is fixed"]
fn a_repeated_rotate_does_not_move_another_object() {
    let mut w = world();
    let rug = buy(&mut w, "rugRound", 2, 2);
    let lamp = buy(&mut w, "lamp", 11, 11);
    let (x, z) = (w.objects[rug as usize].x, w.objects[rug as usize].z);
    let rotate = Command::MoveObject { household: 0, object: rug, x, z, rot: 1, turn: None, offset: None, size: None, tall: None };
    w.apply(rotate.clone()).unwrap();
    let _ = w.apply(rotate);
    let lamp_now = w.objects.iter().find(|o| w.content.objects[o.def].id == "lamp").unwrap();
    assert_eq!((lamp_now.x, lamp_now.z), (11, 11), "the lamp (id {lamp} before) stays where it was");
}
