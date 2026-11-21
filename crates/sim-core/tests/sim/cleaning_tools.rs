//! Cleaning with a tool (content `cleans`, `sweep.rs`) and ironing (content `laundry.pressed`), on
//! the shipped content: the vacuum cleaner, the mop and bucket and the broom are taken to the
//! dirtiest floor at home they clean and clean the room there (the mop and the broom leave a
//! carpet; only the broom sweeps the garden); residents pick them up on their own, the neat
//! sooner; the ironing board turns a dry load into pressed clothes, which feel crisp to dress in.

use crate::common;

use common::interaction;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World};

/// A hall (x 2..7) and a bedroom (x 8..13, carpeted by its kind: the bed), both with a door to
/// the garden below them; the cleaning things in the hall, the ironing board and the wardrobe in
/// the bedroom.
const TOWN: &str = r#"{"width":16,"depth":14,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,13.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "walls":[[1,1,13,1],[1,7,13,7],[1,1,1,7],[7,1,7,7],[13,1,13,7]],
    "doors":[{"x":3,"z":7,"axis":"x"},{"x":10,"z":7,"axis":"x"}],
    "objects":[{"def":"vacuumCleaner","x":2,"z":1},{"def":"mopBucket","x":3,"z":1},
               {"def":"broomDustpan","x":4,"z":1},{"def":"bedSingle","x":11,"z":1},
               {"def":"wardrobe","x":8,"z":1},{"def":"ironingBoard","x":8,"z":4}],
    "sims":[{"name":"Ada","traits":["neat"],"x":4.5,"z":10.5}]}"#;

const VACUUM: u32 = 0;
const MOP: u32 = 1;
const BROOM: u32 = 2;
const WARDROBE: u32 = 4;
const BOARD: u32 = 5;

fn world_with(town: &str) -> World {
    let content = common::content();
    let mut w = World::from_json(&content, town, 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    common::run_to(&mut w, 1, 9.0);
    fill(&mut w);
    w
}

fn world() -> World {
    world_with(TOWN)
}

fn fill(w: &mut World) {
    let n = w.content.needs.len();
    for s in &mut w.sims {
        s.needs[..n].fill(1.0);
    }
}

fn order(w: &mut World, object: u32, id: &str) -> Result<(), String> {
    let interaction = interaction(w, object, id);
    let object = w.objects[object as usize].stable;
    w.apply(Command::Use {
        sim: 0,
        object,
        interaction,
    })
    .map_err(|e| e.to_string())
}

/// Ticks up to `minutes` until `f` holds: whether it did.
fn until(w: &mut World, minutes: u32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..minutes * sim_core::TICKS_PER_MINUTE as u32 {
        w.tick_once();
        common::check(w);
        if f(w) {
            return true;
        }
    }
    false
}

/// Dirt `d` on every tile of the rectangle (x0..x1, z0..z1).
fn soil(w: &mut World, (x0, x1): (i32, i32), (z0, z1): (i32, i32), d: f32) {
    for z in z0..z1 {
        for x in x0..x1 {
            let i = w.lot.tile_index(x, z);
            w.dirt[i] = d;
        }
    }
}

/// The most dirt on a tile of the rectangle.
fn worst(w: &World, (x0, x1): (i32, i32), (z0, z1): (i32, i32)) -> f32 {
    (z0..z1)
        .flat_map(|z| (x0..x1).map(move |x| (x, z)))
        .map(|(x, z)| w.dirt[w.lot.tile_index(x, z)])
        .fold(0.0, f32::max)
}

const HALL: ((i32, i32), (i32, i32)) = ((2, 7), (2, 7));
const BEDROOM: ((i32, i32), (i32, i32)) = ((8, 13), (2, 7));

/// The object Ada is using, and where she stands, once she's at it.
fn using(w: &World) -> Option<(u32, [f32; 2])> {
    let a = w.sims[0].current()?;
    match (a.task.kind, &a.phase) {
        (TaskKind::Use { object, .. }, Phase::Using { .. }) => Some((object, w.sims[0].pos)),
        _ => None,
    }
}

#[test]
fn the_mop_cleans_the_hard_floor_of_a_whole_room_and_leaves_the_carpet() {
    let mut w = world();
    soil(&mut w, HALL.0, HALL.1, 0.5);
    soil(&mut w, BEDROOM.0, BEDROOM.1, 0.6);
    let i = w.lot.tile_index(5, 5);
    w.dirt[i] = 0.8;
    // The bedroom is dirtier, but carpeted: the mop goes to the hall, the hard floor with the worst
    // of it, and mops it from the middle.
    order(&mut w, MOP, "mop").unwrap();
    let mut at = None;
    assert!(until(&mut w, 60, |w| {
        if let Some((MOP, p)) = using(w) {
            at = Some(p);
        }
        w.sims[0].current().is_none() && at.is_some()
    }));
    let at = at.unwrap();
    let tile = (at[0].floor() as i32, at[1].floor() as i32);
    assert!(
        (2..7).contains(&tile.0) && (2..7).contains(&tile.1) && tile.1 >= 3,
        "used in the hall it mops, out in the room, not where it's kept: {tile:?}"
    );
    assert!(
        worst(&w, HALL.0, HALL.1) < 0.01,
        "the whole hall mopped: {}",
        worst(&w, HALL.0, HALL.1)
    );
    assert!(
        (worst(&w, BEDROOM.0, BEDROOM.1) - 0.6).abs() < 1e-4,
        "not the bedroom's carpet"
    );
    // Only the carpet left dirty: nothing for the mop, the player is told.
    let err = order(&mut w, MOP, "mop").unwrap_err();
    assert!(err.contains("clean"), "{err}");
    // The vacuum does carpets.
    order(&mut w, VACUUM, "vacuum").unwrap();
    assert!(until(&mut w, 60, |w| worst(w, BEDROOM.0, BEDROOM.1) < 0.01));
    assert!(w.sims[0].skills.iter().any(|&s| s > 0.0));
}

#[test]
fn the_broom_sweeps_round_the_spot_and_out_in_the_garden() {
    let mut w = world();
    // Crumbs on the patio below the hall's door: only the broom goes out for them.
    soil(&mut w, (2, 7), (9, 12), 0.4);
    let i = w.lot.tile_index(4, 10);
    w.dirt[i] = 0.7;
    assert!(
        order(&mut w, VACUUM, "vacuum").is_err(),
        "no vacuuming the garden"
    );
    order(&mut w, BROOM, "sweep").unwrap();
    assert!(until(&mut w, 60, |w| w.sims[0].current().is_none()
        && w.dirt[w.lot.tile_index(4, 10)] < 0.2));
    // Within its reach (two tiles each way), not past it.
    assert!(w.dirt[w.lot.tile_index(2, 9)] < 0.01);
    assert!((w.dirt[w.lot.tile_index(6, 11)] - 0.0).abs() < 1e-4);
    soil(&mut w, (8, 16), (12, 14), 0.4);
    let far = w.lot.tile_index(15, 13);
    assert!((w.dirt[far] - 0.4).abs() < 1e-4, "out of its reach");
}

#[test]
fn neat_residents_take_the_tools_to_a_dirty_room_on_their_own() {
    let mut w = world();
    w.apply(Command::SetAutonomy {
        enabled: true,
        household: None,
    })
    .unwrap();
    soil(&mut w, HALL.0, HALL.1, 0.45);
    let mut tool = None;
    for _ in 0..4 * 60 {
        for _ in 0..sim_core::TICKS_PER_MINUTE {
            w.tick_once();
            if let Some((o, _)) = using(&w)
                && w.content.objects[w.objects[o as usize].def]
                    .interactions
                    .iter()
                    .any(|i| i.cleans.is_some())
            {
                tool = Some(o);
            }
        }
        common::check(&w);
        fill_but_fun(&mut w);
        if tool.is_some() && worst(&w, HALL.0, HALL.1) < 0.2 {
            break;
        }
    }
    assert!(worst(&w, HALL.0, HALL.1) < 0.2, "the hall cleaned");
    assert!(
        [VACUUM, MOP, BROOM].contains(&tool.unwrap()),
        "a tool: {tool:?}"
    );
}

/// Needs kept full but fun, so nothing pulls her away from the chores for long.
fn fill_but_fun(w: &mut World) {
    let n = w.content.needs.len();
    let fun = w.content.needs.iter().position(|d| d.id == "fun");
    for (k, v) in w.sims[0].needs[..n].iter_mut().enumerate() {
        if Some(k) != fun {
            *v = v.max(0.9);
        }
    }
}

/// How long (game minutes) a resident of `trait_id` takes to clean a dirty hall with free will,
/// bored, the television and the bookshelf in the bedroom.
fn minutes_to_clean(trait_id: &str, dirt: f32) -> u32 {
    let town = TOWN
        .replace(r#""traits":["neat"]"#, &format!(r#""traits":["{trait_id}"]"#))
        .replace(
            r#"{"def":"wardrobe","x":8,"z":1}"#,
            r#"{"def":"wardrobe","x":8,"z":1},{"def":"tv","x":10,"z":1},{"def":"bookshelf","x":12,"z":4}"#,
        );
    let mut w = world_with(&town);
    w.apply(Command::SetAutonomy {
        enabled: true,
        household: None,
    })
    .unwrap();
    let fun = w.content.needs.iter().position(|d| d.id == "fun").unwrap();
    w.sims[0].needs[fun] = 0.2;
    soil(&mut w, HALL.0, HALL.1, dirt);
    for m in 0..8 * 60 {
        for _ in 0..sim_core::TICKS_PER_MINUTE {
            w.tick_once();
        }
        fill_but_fun(&mut w);
        if worst(&w, HALL.0, HALL.1) < dirt / 2.0 {
            return m;
        }
    }
    8 * 60
}

#[test]
fn the_neat_clean_sooner_than_slobs() {
    // (Bored, the slob watches television first; the neat get the chores done.)
    let (neat, slob) = (minutes_to_clean("neat", 0.3), minutes_to_clean("slob", 0.3));
    assert!(neat < 2 * 60, "neat: {neat} minutes");
    assert!(neat < slob, "neat {neat}, slob {slob} minutes");
}

#[test]
fn ironing_presses_the_dry_washing_and_dressing_in_it_feels_crisp() {
    let mut w = world();
    let has = |w: &World| {
        w.sims[0]
            .feelings
            .iter()
            .any(|m| w.content.feelings[m.def].id == "pressedClothes")
    };
    // No pressed clothes: dressing is as ever.
    w.households[0].laundry = [0.0, 0.0, 1.0, 0.0];
    order(&mut w, WARDROBE, "outfit").unwrap();
    assert!(until(&mut w, 30, |w| w.sims[0].current().is_none()));
    assert!(!has(&w));
    // The dry load ironed: pressed clothes, and deft hands for it.
    let deft = w.sims[0].skills;
    order(&mut w, BOARD, "iron").unwrap();
    assert!(until(&mut w, 60, |w| w.sims[0].current().is_none()));
    assert_eq!(
        w.households[0].laundry[2..],
        [0.0, 1.0],
        "the dry load ironed"
    );
    let dexterity = w
        .content
        .skills
        .iter()
        .position(|s| s.id == "dexterity")
        .unwrap();
    assert!(w.sims[0].skills[dexterity] > deft[dexterity]);
    // Saved and loaded, they wait.
    let loaded = World::from_save_json(&common::content(), &w.save_json()).unwrap();
    assert_eq!(loaded.households[0].laundry[3], 1.0);
    // Dressed in them: crisp, an outfit's worth taken.
    order(&mut w, WARDROBE, "outfit").unwrap();
    assert!(until(&mut w, 30, |w| w.sims[0].current().is_none()));
    assert!(has(&w), "feels crisply pressed");
    assert!((w.households[0].laundry[3] - 0.75).abs() < 1e-4);
    // Nothing dry to iron: the board waits.
    assert!(order(&mut w, BOARD, "iron").is_err());
}
