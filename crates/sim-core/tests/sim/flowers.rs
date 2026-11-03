//! Cut flowers for a vase (Q-18): the flower bed gives a bunch (`cut`, in the hand while it's cut) and the
//! empty vase takes it (`putFlowers`, the bunch in the arms on the way, the vase's own flowers grown up as it
//! goes in). The
//! vase stands open with them in it, saved that way, until they are taken out again.

use crate::common;

use common::{interaction, shown_held, until};
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, TICKS_PER_MINUTE, World};

/// A room with a side table to stand the vase on, and a flower bed out in the garden.
const TOWN: &str = r#"{"width":14,"depth":14,
  "plots":[{"name":"Home","x":0,"z":0,"w":14,"d":14,"entry":[13.5,13.5]}],
  "households":[{"name":"Player","plot":0,"player":true}],
  "walls":[[1,1,9,1],[1,1,1,7],[1,7,9,7],[9,1,9,7]],
  "doors":[{"x":4,"z":7,"axis":"x"}],
  "objects":[{"def":"sideTableRound","x":4,"z":3},{"def":"vaseEmpty","x":4,"z":3},
             {"def":"flowerbed","x":12,"z":12},{"def":"garden.tulips","x":10,"z":12},
             {"def":"garden.daisies","x":2,"z":12}],
  "sims":[{"name":"Ada","x":6,"z":10,"traits":["cheerful"]}]}"#;

fn world() -> (World, String) {
    let content = common::content();
    let mut w = World::from_json(&content, TOWN, 3).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    common::run_to(&mut w, 1, 10.0);
    let n = w.content.needs.len();
    for s in &mut w.sims {
        s.needs[..n].fill(1.0);
    }
    (w, content)
}

fn object(w: &World, id: &str) -> u32 {
    w.objects
        .iter()
        .position(|o| w.content.objects[o.def].id == id)
        .unwrap_or_else(|| panic!("no {id}")) as u32
}

fn using(w: &World, i: usize) -> bool {
    at(w, i, |p| matches!(p, Phase::Using { .. }))
}

fn routing(w: &World, i: usize) -> bool {
    at(w, i, |p| matches!(p, Phase::Routing { .. }))
}

/// Whether Sim 0 is on interaction `i` of something, its phase `which`.
fn at(w: &World, i: usize, which: impl Fn(&Phase) -> bool) -> bool {
    w.sims[0].current().is_some_and(|a| {
        matches!(a.task.kind, TaskKind::Use { interaction, .. } if interaction == i)
            && which(&a.phase)
    })
}

#[test]
fn the_garden_gives_a_bunch_and_the_vase_takes_it() {
    let (mut w, content) = world();
    let bed = object(&w, "flowerbed");
    let vase = object(&w, "vaseEmpty");
    let cut = interaction(&w, bed, "cut");
    let put = interaction(&w, vase, "putFlowers");
    let out = interaction(&w, vase, "takeOut");
    let o = vase as usize;
    assert!(!w.objects[o].open, "a new vase is empty");

    // Nothing to take out of an empty vase.
    let refused = w
        .apply(Command::Use {
            sim: 0,
            object: vase,
            interaction: out,
        })
        .unwrap_err()
        .to_string();
    assert!(refused.contains("is closed"), "{refused}");

    // The flower bed: the bunch is in their hand as they cut it.
    w.apply(Command::Use {
        sim: 0,
        object: bed,
        interaction: cut,
    })
    .unwrap();
    let (mut in_hand, mut finished) = (false, false);
    for _ in 0..40 * TICKS_PER_MINUTE {
        if using(&w, cut) {
            in_hand |= shown_held(&w).as_deref() == Some("cutFlowers");
        }
        // The command lands on the next tick, so nothing yet is not the use being over.
        if in_hand && w.sims[0].current().is_none() {
            finished = true;
            break;
        }
        w.tick_once();
    }
    assert!(finished, "the cutting done");
    assert!(in_hand, "the flowers in hand as they're cut");
    assert_eq!(
        shown_held(&w),
        None,
        "the bunch is only shown while it is cut"
    );

    // The vase: the bunch in the arms on the way there, the vase's own bunch grown up as it goes in.
    w.apply(Command::Use {
        sim: 0,
        object: vase,
        interaction: put,
    })
    .unwrap();
    let (mut carried, mut empty_handed, mut opened) = (false, false, false);
    for _ in 0..40 * TICKS_PER_MINUTE {
        carried |= routing(&w, put) && shown_held(&w).as_deref() == Some("cutFlowers");
        empty_handed |= using(&w, put) && shown_held(&w).is_none();
        opened |= w.objects[o].open;
        if opened && w.sims[0].current().is_none() {
            break;
        }
        w.tick_once();
    }
    assert!(carried, "carried to the vase in the arms");
    assert!(
        empty_handed,
        "out of the arms as the vase's own bunch grows"
    );
    assert!(opened, "put the flowers in");
    assert!(w.objects[o].open, "it stands open with the flowers in it");
    assert_eq!(shown_held(&w), None, "nothing in hand once they're in");

    // Still open after saving and loading, and the flowers come back out.
    let mut w = common::reload(&w, &content);
    // (The things on tops are placed after the rest, so the vase's row can differ.)
    let o = object(&w, "vaseEmpty") as usize;
    assert!(w.objects[o].open, "still open after loading");
    w.apply(Command::Use {
        sim: 0,
        object: vase,
        interaction: out,
    })
    .unwrap();
    assert!(
        until(&mut w, 20, |w| !w.objects[o].open
            && w.sims[0].current().is_none()),
        "taken out, and shut"
    );
}

#[test]
fn a_bed_of_tulips_gives_a_bunch_too() {
    let (mut w, _content) = world();
    let bed = object(&w, "garden.tulips");
    let cut = interaction(&w, bed, "cut");
    w.apply(Command::Use {
        sim: 0,
        object: bed,
        interaction: cut,
    })
    .unwrap();
    let mut in_hand = false;
    for _ in 0..40 * TICKS_PER_MINUTE {
        if using(&w, cut) {
            in_hand |= shown_held(&w).as_deref() == Some("cutFlowers");
        }
        if in_hand && w.sims[0].current().is_none() {
            break;
        }
        w.tick_once();
    }
    assert!(in_hand, "the tulips in hand as they're cut");
}

/// The daisies' chain (Q-18): the one real use the beds were still missing. It is the bed's own daisies,
/// made at the bed, and the chain is in the hand while it's made.
#[test]
fn the_daisies_make_a_daisy_chain() {
    let (mut w, _content) = world();
    let bed = object(&w, "garden.daisies");
    let make = interaction(&w, bed, "daisyChain");
    w.apply(Command::Use {
        sim: 0,
        object: bed,
        interaction: make,
    })
    .unwrap();
    let (mut in_hand, mut finished) = (false, false);
    for _ in 0..40 * TICKS_PER_MINUTE {
        if using(&w, make) {
            in_hand |= shown_held(&w).as_deref() == Some("daisyChain");
        }
        if in_hand && w.sims[0].current().is_none() {
            finished = true;
            break;
        }
        w.tick_once();
    }
    assert!(finished, "the chain made");
    assert!(in_hand, "the chain in hand as it's made");
    assert_eq!(
        shown_held(&w),
        None,
        "the chain is only shown while it is made"
    );
}

/// Every bed of flowers the garden sells can be cut for a vase, and each cut is the same act: the bunch
/// in the hand, the shears' work (`tend`), and never something a resident chooses on their own.
#[test]
fn every_bed_can_be_cut() {
    let content = sim_core::content::Content::from_json(&common::content()).unwrap();
    let tend = content
        .animations
        .iter()
        .position(|a| a == "tend")
        .expect("the shears' work is an animation");
    let bunch = content
        .items
        .iter()
        .position(|i| i.id == "cutFlowers")
        .expect("the bunch is an item");
    let mut beds = 0;
    for def in &content.objects {
        if !def.id.starts_with("garden.") || def.group != "flowers" {
            continue;
        }
        let Some(cut) = def.interactions.iter().find(|i| i.id == "cut") else {
            continue;
        };
        beds += 1;
        assert_eq!(cut.anim, Some(tend), "{}", def.id);
        assert_eq!(cut.holds, Some(bunch), "{}", def.id);
        assert!(!cut.autonomous, "{}: cutting is not a habit", def.id);
        assert!(cut.minutes > 0.0 && cut.minutes <= 10.0, "{}", def.id);
    }
    assert_eq!(
        beds, 5,
        "the five beds of flowers: tulips, daffodils, lilies, cosmos, daisies"
    );
}
