//! Routines (`after`): with the shipped content, the toilet is followed by washing hands at the
//! nearest sink, for its full time even with clean hands; with no sink, nothing follows.

mod common;

use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, MINUTES_PER_TICK, World};

/// An open 16×14 plot with one resident.
const TOWN: &str = r#"{"width":16,"depth":16,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,13.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
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

/// The object and interaction resident 0 is using (not on their way to).
fn using(w: &World) -> Option<(u32, usize)> {
    match w.sims[0].current() {
        Some(a) if matches!(a.phase, Phase::Using { .. }) => match a.task.kind {
            TaskKind::Use { object, interaction } => Some((object, interaction)),
            _ => None,
        },
        _ => None,
    }
}

/// Uses the toilet (with a full-ish bladder and clean hands) and returns what follows it:
/// the next object used and for how many minutes, if anything is used within an hour.
fn after_the_toilet(w: &mut World, toilet: u32) -> Option<(u32, usize, f32)> {
    let (bladder, hygiene) = (need(w, "bladder"), need(w, "hygiene"));
    w.sims[0].needs[bladder] = 0.3;
    w.sims[0].needs[hygiene] = 1.0;
    let use_it = w.content.objects[w.objects[toilet as usize].def].interactions.iter().position(|i| i.id == "use").unwrap();
    w.apply(Command::Use { sim: 0, object: toilet, interaction: use_it }).unwrap();
    let ticks = (60.0 / MINUTES_PER_TICK) as u32;
    let mut on = false;
    for _ in 0..ticks {
        w.tick_once();
        match using(w) {
            Some((o, _)) if o == toilet => on = true,
            _ if on => break,
            _ => {}
        }
    }
    assert!(on && w.sims[0].needs[bladder] > 0.9, "used the toilet");
    let mut next = None;
    for _ in 0..ticks {
        match (using(w), next) {
            (Some((o, i)), None) => next = Some((o, i, 0.0)),
            (Some((o, i)), Some((n, j, minutes))) if (o, i) == (n, j) => next = Some((n, j, minutes + MINUTES_PER_TICK)),
            (_, Some(_)) => break,
            _ => {}
        }
        w.tick_once();
    }
    next
}

#[test]
fn after_the_toilet_hands_are_washed_at_the_nearest_sink() {
    let mut w = world();
    let toilet = buy(&mut w, "toilet", 3, 3);
    buy(&mut w, "sink", 12, 3);
    let sink = buy(&mut w, "sink", 5, 3);
    let (object, interaction, minutes) = after_the_toilet(&mut w, toilet).expect("something follows the toilet");
    assert_eq!(object, sink, "the nearer sink");
    let wash = &w.content.objects[w.objects[sink as usize].def].interactions[interaction];
    assert_eq!(wash.id, "wash");
    assert!(minutes >= wash.minutes - 0.1, "washed for {minutes} of {} minutes, though already clean", wash.minutes);
    assert!(w.sims[0].current().is_none(), "and then nothing more");
    common::check(&w);
}

#[test]
fn without_a_sink_nothing_follows_the_toilet() {
    let mut w = world();
    let toilet = buy(&mut w, "toilet", 3, 3);
    buy(&mut w, "shower", 8, 3);
    assert_eq!(after_the_toilet(&mut w, toilet), None);
}
