//! Cut flowers for a vase (Q-18): the flower bed gives a bunch (`cut`, in the hand while it's cut) and the
//! empty vase takes it (`putFlowers`, the bunch in the arms on the way, the vase's own flowers grown up as it
//! goes in). The
//! vase stands open with them in it, saved that way, until they are taken out again.

mod common;

use sim_core::snapshot::{self, HEADER_LEN, sim};
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, TICKS_PER_MINUTE, World, clock};

/// A room with a side table to stand the vase on, and a flower bed out in the garden.
const TOWN: &str = r#"{"width":14,"depth":14,
  "plots":[{"name":"Home","x":0,"z":0,"w":14,"d":14,"entry":[13.5,13.5]}],
  "households":[{"name":"Player","plot":0,"player":true}],
  "walls":[[1,1,9,1],[1,1,1,7],[1,7,9,7],[9,1,9,7]],
  "doors":[{"x":4,"z":7,"axis":"x"}],
  "objects":[{"def":"sideTableRound","x":4,"z":3},{"def":"vaseEmpty","x":4,"z":3},
             {"def":"flowerbed","x":12,"z":12}],
  "sims":[{"name":"Ada","x":6,"z":10,"traits":["cheerful"]}]}"#;

fn world() -> (World, String) {
    let content = common::content();
    let mut w = World::from_json(&content, TOWN, 3).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    let morning = clock::tick_at(1, 10.0 * 60.0).unwrap();
    while w.tick < morning {
        w.tick_once();
    }
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

fn interaction(w: &World, object: u32, id: &str) -> usize {
    let def = w.objects[object as usize].def;
    w.content.objects[def]
        .interactions
        .iter()
        .position(|i| i.id == id)
        .unwrap_or_else(|| panic!("no {id}"))
}

/// The item Sim 0 is shown holding this tick (the snapshot's `held`: what they carry, or what the use
/// they're on has them bring or hold), as an id.
fn shown(w: &World) -> Option<String> {
    let mut out = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut out);
    let at = HEADER_LEN + sim::HELD;
    let n = out[at] as usize;
    (n > 0).then(|| w.content.items[n - 1].id.clone())
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

/// Ticks `w` for up to `minutes`, until `done`; true if it got there.
fn until(w: &mut World, minutes: u64, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..minutes * TICKS_PER_MINUTE {
        if done(w) {
            return true;
        }
        w.tick_once();
    }
    done(w)
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
            in_hand |= shown(&w).as_deref() == Some("cutFlowers");
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
    assert_eq!(shown(&w), None, "the bunch is only shown while it is cut");

    // The vase: the bunch in the arms on the way there, the vase's own bunch grown up as it goes in.
    w.apply(Command::Use {
        sim: 0,
        object: vase,
        interaction: put,
    })
    .unwrap();
    let (mut carried, mut empty_handed, mut opened) = (false, false, false);
    for _ in 0..40 * TICKS_PER_MINUTE {
        carried |= routing(&w, put) && shown(&w).as_deref() == Some("cutFlowers");
        empty_handed |= using(&w, put) && shown(&w).is_none();
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
    assert_eq!(shown(&w), None, "nothing in hand once they're in");

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
