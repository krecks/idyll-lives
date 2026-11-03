//! Rearranging homes while people live in them, with the shipped content: furniture sold, moved,
//! turned and bought at random every few game minutes for two game days (at home and at the
//! neighbours', with undo and redo, while NPC households replace things on their own), and the
//! same command sent twice, never panics or breaks what must hold (`common::check`). Object ids
//! are places in the list, so each sale or move renumbers what comes after it: what residents
//! do, have queued and will do next out of habit, and the food on the tables, must follow.

use crate::common;

use sim_core::world::TICKS_PER_DAY;
use sim_core::{Command, World};

/// The town, looking at the player's home (the rest in less detail, so residents there choose
/// their next step less often), with money for anything.
fn town(seed: u64) -> World {
    let content = common::content();
    let mut w = World::from_json(&content, &common::town(&content, seed, 2), seed as u32).unwrap();
    for h in &mut w.households {
        h.funds = 10_000_000;
    }
    let home = w
        .households
        .iter()
        .find(|h| h.player)
        .and_then(|h| h.plot)
        .unwrap();
    common::look_at(&mut w, home);
    w
}

/// Things on `plot` that can be sold or moved (they have a price).
fn owned(w: &World, plot: u32) -> Vec<u32> {
    w.objects
        .iter()
        .filter(|o| w.content.objects[o.def].price.is_some() && w.plot_at(o.x, o.z) == Some(plot))
        .map(|o| o.id)
        .collect()
}

fn move_to(household: u32, object: u32, x: i32, z: i32, rot: u8) -> Command {
    Command::MoveObject {
        household,
        object,
        x,
        z,
        rot,
        turn: None,
        offset: None,
        size: None,
        tall: None,
    }
}

/// One random edit of a random household's home (mostly the player's), sometimes undone or
/// undone and made again. Edits that don't fit are refused; that's fine.
fn edit(w: &mut World, rng: &mut common::Lcg) {
    let homes: Vec<(u32, u32)> = w
        .households
        .iter()
        .filter_map(|h| Some((h.id, h.plot?)))
        .collect();
    let (household, plot) = if rng.below(3) > 0 {
        homes[0]
    } else {
        homes[rng.below(homes.len())]
    };
    let things = owned(w, plot);
    let any = |rng: &mut common::Lcg| things[rng.below(things.len())];
    let _ = match rng.below(5) {
        0 if !things.is_empty() => w.apply(Command::Sell {
            household,
            object: any(rng),
        }),
        1 | 2 if !things.is_empty() => {
            let o = &w.objects[any(rng) as usize];
            let (id, x, z, rot) = (o.id, o.x, o.z, rng.below(4) as u8);
            // Turned where it stands, or somewhere else at home.
            let p = &w.plots[plot as usize];
            let (x, z) = if rng.below(2) == 0 {
                (x, z)
            } else {
                (
                    p.x + rng.below(p.w as usize) as i32,
                    p.z + rng.below(p.d as usize) as i32,
                )
            };
            w.apply(move_to(household, id, x, z, rot))
        }
        _ => {
            let defs: Vec<&str> = w
                .content
                .objects
                .iter()
                .filter(|d| d.price.is_some() && !d.stairs)
                .map(|d| d.id.as_str())
                .collect();
            let def = defs[rng.below(defs.len())].to_string();
            w.apply(Command::Buy {
                household,
                object: def,
                at: None,
                colors: Default::default(),
                turn: None,
                offset: None,
                size: None,
                tall: None,
            })
        }
    };
    match rng.below(10) {
        0 | 1 => {
            let _ = w.apply(Command::Undo { household });
        }
        2 => {
            let _ = w.apply(Command::Undo { household });
            let _ = w.apply(Command::Redo { household });
        }
        _ => {}
    }
}

/// A day of a town's homes rearranged from `seed`: an edit every few game minutes, nothing breaks.
fn rearranging_homes_for_a_day(seed: u64) {
    let mut w = town(seed);
    let mut rng = common::Lcg::new(seed);
    let end = w.tick + TICKS_PER_DAY;
    let mut edits = 0;
    while w.tick < end {
        w.tick_once();
        // An edit every few game minutes, at a random tick (a follow-up or a conversation
        // may be pending).
        if rng.below(30) == 0 {
            edit(&mut w, &mut rng);
            edits += 1;
            common::check(&w);
        } else if w.tick.is_multiple_of(20) {
            common::check(&w);
        }
    }
    assert!(edits > 750, "seed {seed}: {edits} edits");
}

// (Four towns, a test each: they run side by side.)
#[test]
fn rearranging_homes_for_a_day_breaks_nothing() {
    rearranging_homes_for_a_day(5);
}

#[test]
fn rearranging_a_second_town_for_a_day_breaks_nothing() {
    rearranging_homes_for_a_day(6);
}

#[test]
fn rearranging_a_third_town_for_a_day_breaks_nothing() {
    rearranging_homes_for_a_day(7);
}

#[test]
fn rearranging_a_fourth_town_for_a_day_breaks_nothing() {
    rearranging_homes_for_a_day(8);
}

/// The same command sent twice before the view caught up (a held-down key, a double click):
/// the object's stable id means the second still names it, never something else.
#[test]
fn the_same_move_sent_twice_moves_nothing_else() {
    let mut w = town(7);
    let mut rng = common::Lcg::new(7);
    let (household, plot) = w
        .households
        .iter()
        .find(|h| h.player)
        .map(|h| (h.id, h.plot.unwrap()))
        .unwrap();
    let layout = |w: &World, but: (i32, i32)| {
        let mut all: Vec<(usize, i32, i32, u8)> = w
            .objects
            .iter()
            .filter(|o| (o.x, o.z) != but)
            .map(|o| (o.def, o.x, o.z, o.rot))
            .collect();
        all.sort_unstable();
        all
    };
    for _ in 0..200 {
        let things = owned(&w, plot);
        let o = &w.objects[things[rng.below(things.len())] as usize];
        let (id, x, z) = (o.stable, o.x, o.z);
        let others = layout(&w, (x, z));
        let turn = move_to(household, id, x, z, rng.below(4) as u8);
        if w.apply(turn.clone()).is_err() {
            continue;
        }
        // (A refusal is fine: the turn may not fit twice.)
        let _ = w.apply(turn);
        assert_eq!(layout(&w, (x, z)), others, "something else moved");
        for _ in 0..rng.below(40) {
            w.tick_once();
        }
        common::check(&w);
    }
}
