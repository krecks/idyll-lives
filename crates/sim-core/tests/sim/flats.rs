//! Flats (`Plot::of`): several households in one building, each flat a plot of its own on one
//! storey. Mill Court (`tests/towns/flats.json`): two storeys, four flats round a stair hall, the
//! letterboxes by the street door. The Corner Shop (`tests/towns/flats_over_shop.json`): two
//! flats over a shop, their own street door and stair hall beside the shop's, a door from the
//! hall into the shop, and the house next door. Each flat lives its day in its own rooms with its
//! own things, a visitor knocks at the flat's own door off the landing and is let in, the post is
//! fetched from the hall, newcomers move into a vacant flat, rent is by the flat, the building's
//! envelope is not the flat's to build on, a watched building watches its flats, and the shop's
//! customers keep out of the flats' hall, which stays the flats' way home when the shop is shut.

use crate::common;

use std::collections::BTreeSet;

use serde_json::{Value, json};
use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind};
use sim_core::social::EventKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World};

const FLATS: &str = include_str!("../towns/flats.json");
const OVER_SHOP: &str = include_str!("../towns/flats_over_shop.json");

/// Mill Court's building (plot 0); flat `k + 1` is household `k`'s, sim `k` the one living
/// there (Ada, Ben, Cara and Dev; Ada's the player's).
const BUILDING: u32 = 0;
const ADA: u32 = 0;
const CARA: u32 = 2;

/// The Corner Shop (plot 0), its two flats upstairs (plots 2 and 3: Fay's, the player's, and
/// Gus's), the house next door (plot 1, Eve's), the shop's sofa (object 0); the hall is ground
/// tiles x 15..18, z 8..17 (and the landing over it), the flats' letterboxes outside its door.
const SHOP: u32 = 0;
const FAY: u32 = 0;
const GUS: u32 = 1;
const EVE: u32 = 2;
const SOFA: u32 = 0;

/// `json` with `change` made to it.
fn town(json: &str, change: impl FnOnce(&mut Value)) -> String {
    let mut t: Value = serde_json::from_str(json).unwrap();
    change(&mut t);
    t.to_string()
}

fn world(town: &str) -> World {
    World::from_json(&common::content(), town, 1).unwrap()
}

/// Where object `o` belongs: the plot keeping it (a flat's letterbox in the hall), else the
/// plot it stands on.
fn belongs(w: &World, o: u32) -> Option<u32> {
    let obj = &w.objects[o as usize];
    obj.keeper.or_else(|| w.plot_at(obj.x, obj.z))
}

fn home(w: &World, s: usize) -> Option<u32> {
    w.households[w.sims[s].household as usize].plot
}

fn def_of(w: &World, o: u32) -> &str {
    &w.content.objects[w.objects[o as usize].def].id
}

/// The objects a task of `s`'s uses: what it's using, the seat it eats at, the station it
/// cooks or washes up at.
fn objects_used(t: &TaskKind) -> Vec<u32> {
    match *t {
        TaskKind::Use { object, .. } => vec![object],
        TaskKind::Eat { seat, .. } => seat.into_iter().collect(),
        TaskKind::Cook { station, .. } | TaskKind::Dishes { station, .. } => {
            station.into_iter().collect()
        }
        _ => Vec::new(),
    }
}

/// Mill Court's residents through a day with free will: each sleeps in their own bed and uses
/// their own toilet; nobody uses a neighbour's things, cleans a neighbour's floor or sets foot in
/// a neighbour's flat. The building's own things (the hall's) are everyone's in it.
#[test]
fn a_day_in_each_flat_with_its_own_things() {
    let mut w = world(FLATS);
    for h in &mut w.households {
        h.free_will = true;
    }
    let mut used: Vec<BTreeSet<String>> = vec![BTreeSet::new(); w.sims.len()];
    for _ in 0..common::TICKS_PER_DAY {
        w.tick_once();
        for (i, s) in w.sims.iter().enumerate() {
            if !s.here() || s.away_until.is_some() {
                continue;
            }
            let mine = home(&w, i);
            let (x, z) = s.tile();
            if let Some(p) = w.plot_at(x, z).filter(|&p| w.plots[p as usize].is_flat()) {
                assert_eq!(
                    Some(p),
                    mine,
                    "{} in {} at {x},{z}",
                    s.name,
                    w.plots[p as usize].name
                );
            }
            let Some(task) = s.current().map(|a| a.task.kind) else {
                continue;
            };
            for o in objects_used(&task) {
                let on = belongs(&w, o);
                assert!(
                    on == mine || on == Some(BUILDING),
                    "{} used the {} of {:?}",
                    s.name,
                    def_of(&w, o),
                    on.map(|p| &w.plots[p as usize].name)
                );
                if on == mine {
                    used[i].insert(def_of(&w, o).to_string());
                }
            }
            if let TaskKind::Clean { x, z } = task {
                assert_eq!(w.plot_at(x, z), mine, "{} cleaning at {x},{z}", s.name);
            }
        }
    }
    for (i, u) in used.iter().enumerate() {
        for what in ["bed", "toilet"] {
            assert!(
                u.contains(what),
                "{}: no {what} of their own in {u:?}",
                w.sims[i].name
            );
        }
    }
}

/// Cara (upstairs, flat 3) visits her friend Ada (flat 1): down the stairs to the landing before
/// Ada's own door, knocks there, and is let in.
#[test]
fn a_visitor_knocks_at_the_flats_own_door_and_is_let_in() {
    let town = town(FLATS, |t| {
        t["relationships"] = json!([{"a": 0, "b": 2, "preset": "friends"},
                                    {"a": 2, "b": 0, "preset": "friends"}]);
    });
    let mut w = world(&town);
    w.autonomy = false;
    common::run_to(&mut w, 1, 11.0);
    w.apply(Command::Visit { sim: CARA, plot: 1 }).unwrap();
    let mut knocked_at = None;
    for _ in 0..common::mins(60.0) {
        w.tick_once();
        let cara = &w.sims[CARA as usize];
        if cara
            .current()
            .is_some_and(|a| matches!(a.phase, Phase::Knocking { .. }))
        {
            knocked_at.get_or_insert(cara.tile());
        }
        if w.events.iter().any(|e| e.kind == EventKind::Visited) {
            break;
        }
    }
    assert_eq!(
        knocked_at, w.plots[1].door,
        "at flat 1's own door, off the landing"
    );
    let visited = w
        .events
        .iter()
        .find(|e| e.kind == EventKind::Visited)
        .expect("let in");
    assert_eq!((visited.a, visited.b), (CARA, ADA));
    assert_eq!(w.sims[CARA as usize].visiting.map(|v| v.plot), Some(1));
    // In she comes, to Ada's sofa (a guest's to sit on).
    let sofa = (0..w.objects.len() as u32)
        .find(|&o| def_of(&w, o) == "sofa" && belongs(&w, o) == Some(1))
        .unwrap();
    let relax = common::interaction(&w, sofa, "relax");
    w.apply(Command::Use {
        sim: CARA,
        object: sofa,
        interaction: relax,
    })
    .unwrap();
    assert!(
        common::until(&mut w, 20, |w| {
            let (x, z) = w.sims[CARA as usize].tile();
            w.plot_at(x, z) == Some(1)
        }),
        "in Ada's flat"
    );
}

/// The morning's post comes to each flat's letterbox in the hall (kept by the flat): each
/// household fetches its own, and a neighbour's isn't theirs to open.
#[test]
fn the_post_is_fetched_from_the_letterboxes_in_the_hall() {
    let mut w = world(FLATS);
    for h in &mut w.households {
        h.free_will = true;
    }
    let boxes: Vec<u32> = (0..w.objects.len() as u32)
        .filter(|&o| def_of(&w, o) == "letterbox")
        .collect();
    assert_eq!(boxes.len(), 4);
    assert!(common::until(&mut w, 24 * 60, |w| boxes
        .iter()
        .all(|&b| w.objects[b as usize].open)));
    for &b in &boxes {
        assert_eq!(
            w.plot_at(w.objects[b as usize].x, w.objects[b as usize].z),
            Some(BUILDING)
        );
    }
    // Ben's letterbox (flat 2's) isn't Ada's to open.
    let bens = boxes[1];
    assert_eq!(w.objects[bens as usize].keeper, Some(2));
    let err = w
        .apply(Command::Use {
            sim: ADA,
            object: bens,
            interaction: 0,
        })
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    let mut fetched = BTreeSet::new();
    for _ in 0..4 * 60 * common::TICKS_PER_MINUTE {
        w.tick_once();
        for (i, s) in w.sims.iter().enumerate() {
            if let Some(TaskKind::Use { object, .. }) = s.current().map(|a| a.task.kind)
                && boxes.contains(&object)
            {
                assert_eq!(
                    w.objects[object as usize].keeper,
                    home(&w, i),
                    "{} at a neighbour's letterbox",
                    s.name
                );
            }
        }
        for &b in &boxes {
            if !w.objects[b as usize].open {
                fetched.insert(b);
            }
        }
    }
    assert!(fetched.len() >= 2, "fetched: {fetched:?}");
}

/// Dev moves out of flat 4: newcomers move in there (not into the building, nobody's home), in
/// at the building's street door, and are home in their flat soon after.
#[test]
fn newcomers_move_into_a_vacant_flat() {
    let town = town(FLATS, |t| {
        t["sims"].as_array_mut().unwrap().remove(3);
    });
    let mut content: Value = serde_json::from_str(&common::content()).unwrap();
    content["life"]["newcomers"]["days"] = json!(1);
    let mut w = World::from_json(&content.to_string(), &town, 1).unwrap();
    let before = w.sims.len();
    assert!(
        common::until(&mut w, 2 * 24 * 60, |w| w.sims.len() > before),
        "nobody came"
    );
    let new = before;
    let h = w.sims[new].household as usize;
    assert_eq!(w.households[h].plot, Some(4), "into the vacant flat");
    assert!(w.households.iter().all(|hh| hh.plot != Some(BUILDING)));
    let door = w.plots[4].arrival_point();
    assert!(
        common::apart(w.sims[new].pos, door) < 1.0,
        "in at the street door"
    );
    assert!(
        common::until(&mut w, 4 * 60, |w| {
            let (x, z) = w.sims[new].tile();
            w.plot_at(x, z) == Some(4)
        }),
        "home in flat 4"
    );
}

/// Rent is the flat's: each pays for its own floor, not the building's lot.
#[test]
fn rent_is_by_the_flat() {
    let w = world(FLATS);
    let rent = w
        .content
        .rent
        .clone()
        .expect("the shipped content charges rent");
    for h in 0..4 {
        let (paid, _) = sim_core::life::weekly_costs(&w, h).unwrap();
        assert_eq!(paid, rent.amount(7 * 9), "{}", w.households[h].name);
    }
    assert!(rent.amount(7 * 9) < rent.amount(20 * 26));
}

fn edge(axis: EdgeAxis, x: i32, z: i32, kind: EdgeKind) -> EdgeEdit {
    EdgeEdit::new(axis, x, z, kind)
}

/// In a flat the household builds inside it only: inner walls and furniture; not its outer walls,
/// the flat upstairs, the hall, the roof, a garage, a porch or stairs.
#[test]
fn building_in_a_flat_is_inside_it_only() {
    let mut w = world(FLATS);
    w.households[0].funds = 100_000;
    let refused = |w: &mut World, edits: &[EdgeEdit]| w.build(0, edits).unwrap_err().to_string();
    // A wall inside Ada's flat.
    w.build(
        0,
        &(14..17)
            .map(|z| edge(EdgeAxis::V, 8, z, EdgeKind::Wall))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    // (The rooms kept up to date round it are the rooms found again.)
    w.check_rooms().unwrap();
    // Its outer walls: the side, the back, the front, the one onto the hall.
    for e in [
        edge(EdgeAxis::V, 5, 12, EdgeKind::Window),
        edge(EdgeAxis::H, 9, 8, EdgeKind::Door),
        edge(EdgeAxis::H, 9, 17, EdgeKind::Window),
        edge(EdgeAxis::V, 12, 12, EdgeKind::Door),
    ] {
        let err = refused(&mut w, &[e]);
        assert!(err.contains("outer walls"), "{err}");
    }
    // Cara's flat over it, and the hall.
    let up = w.storey_row(1);
    for e in [
        edge(EdgeAxis::V, 8, 14 + up, EdgeKind::Wall),
        edge(EdgeAxis::V, 14, 12, EdgeKind::Wall),
    ] {
        let err = refused(&mut w, &[e]);
        assert!(err.contains("own lot"), "{err}");
    }
    assert!(
        w.set_roof(0, 1, 1)
            .unwrap_err()
            .to_string()
            .contains("in a flat")
    );
    assert!(
        w.build_garage(0, [6.0, 20.0, 9.0, 26.0], 0)
            .unwrap_err()
            .to_string()
            .contains("in a flat")
    );
    assert!(
        w.build_porch(0, EdgeAxis::H, 13, 17)
            .unwrap_err()
            .to_string()
            .contains("in a flat")
    );
    // Furniture in the flat; not stairs, not in the flat upstairs.
    let buy = |w: &mut World, def: &str, x: i32, z: i32| {
        w.apply(Command::Buy {
            household: 0,
            object: def.into(),
            at: Some([x, z, 0]),
            colors: Default::default(),
            turn: None,
            offset: None,
            size: None,
            tall: None,
        })
    };
    buy(&mut w, "diningChair", 7, 15).unwrap();
    assert!(buy(&mut w, "stairs", 6, 9).is_err());
    assert!(buy(&mut w, "diningChair", 7, 15 + up).is_err());
}

/// The player's flat is watched (Ada's): so is its building, and with it every flat in it.
#[test]
fn a_watched_building_watches_its_flats() {
    let mut w = world(FLATS);
    w.lod = true;
    w.set_view(Some([30, 30, 40, 40]));
    w.tick_once();
    for s in 0..4 {
        assert!(w.full_detail(s), "{}", w.sims[s].name);
    }
}

/// The ground tile a position is over.
fn ground_tile(w: &World, s: u32) -> (i32, i32) {
    let (x, z) = w.sims[s as usize].tile();
    (x, w.ground_z(z))
}

fn in_hall(t: (i32, i32)) -> bool {
    (15..18).contains(&t.0) && (8..17).contains(&t.1)
}

/// Eve from next door sits on the shop's sofa: the way in through the flats' hall (its door into
/// the shop) is the shorter, but it's the flats', so she goes in at the shop's door.
#[test]
fn the_shops_customers_keep_out_of_the_flats_hall() {
    let mut w = world(OVER_SHOP);
    w.autonomy = false;
    common::run_to(&mut w, 1, 11.0);
    assert!(w.plots[SHOP as usize].open_at_tick(w.tick));
    w.sims[EVE as usize].needs.iter_mut().for_each(|n| *n = 1.0);
    w.sims[EVE as usize].pos = [16.5, 30.5];
    let sit = common::interaction(&w, SOFA, "relax");
    w.apply(Command::Use {
        sim: EVE,
        object: SOFA,
        interaction: sit,
    })
    .unwrap();
    let mut sat = false;
    for _ in 0..common::mins(30.0) {
        w.tick_once();
        assert!(
            !in_hall(ground_tile(&w, EVE)),
            "Eve in the hall at {:?}",
            w.sims[EVE as usize].tile()
        );
        if w.sims[EVE as usize]
            .current()
            .is_some_and(|a| matches!(a.phase, Phase::Using { .. }))
        {
            sat = true;
            break;
        }
    }
    assert!(sat, "Eve got to the sofa");
}

/// At night the shop is shut, and the flats' street door is still theirs: Fay comes home through
/// the hall to her flat, and Eve, visiting Gus, knocks at his door upstairs and is let in.
#[test]
fn the_flats_door_is_theirs_when_the_shop_is_shut() {
    let town = town(OVER_SHOP, |t| {
        t["relationships"] = json!([{"a": 1, "b": 2, "preset": "friends"},
                                    {"a": 2, "b": 1, "preset": "friends"}]);
    });
    let mut w = world(&town);
    w.autonomy = false;
    common::run_to(&mut w, 1, 19.5);
    assert!(
        !w.plots[SHOP as usize].open_at_tick(w.tick),
        "shut at night"
    );
    // (Fresh, so nothing on the way calls them aside.)
    for s in &mut w.sims {
        s.needs.iter_mut().for_each(|n| *n = 1.0);
    }
    w.sims[FAY as usize].pos = [12.5, 30.5];
    w.apply(Command::GoHome { sim: FAY }).unwrap();
    let mut through = false;
    let home = common::until(&mut w, 30, |w| {
        through |= in_hall(ground_tile(w, FAY));
        let (x, z) = w.sims[FAY as usize].tile();
        w.plot_at(x, z) == Some(2)
    });
    assert!(
        home && through,
        "home through the hall (home {home}, through {through})"
    );
    w.apply(Command::Visit { sim: EVE, plot: 3 }).unwrap();
    let mut knocked_at = None;
    for _ in 0..common::mins(30.0) {
        w.tick_once();
        let eve = &w.sims[EVE as usize];
        if eve
            .current()
            .is_some_and(|a| matches!(a.phase, Phase::Knocking { .. }))
        {
            knocked_at.get_or_insert(eve.tile());
        }
        if w.events.iter().any(|e| e.kind == EventKind::Visited) {
            break;
        }
    }
    assert_eq!(knocked_at, w.plots[3].door, "at Gus's door upstairs");
    let visited = w
        .events
        .iter()
        .find(|e| e.kind == EventKind::Visited)
        .expect("let in");
    assert_eq!((visited.a, visited.b), (EVE, GUS));
}
