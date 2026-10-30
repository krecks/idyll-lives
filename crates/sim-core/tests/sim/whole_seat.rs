//! A use that takes the whole object (content `whole`): a nap along a 2-seat window seat takes
//! both places, so it starts only with nobody else on the seat, and nobody sits down beside the
//! napper until they get up. Sitting on it is still for two at once.

use sim_core::world::TaskKind;
use sim_core::{Command, TICKS_PER_MINUTE, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.01},{"id":"fun","label":"Fun","decayPerHour":0.01}],
    "objects":[{"id":"windowSeat","name":"Window seat","price":300,"footprint":[2,1],"slots":2,"interactions":[
        {"id":"gaze","label":"Gaze out","minutes":60,"pose":"sit","effects":{"fun":0.5},"tags":["lounge"]},
        {"id":"doze","label":"Doze","minutes":60,"pose":"lie","effects":{"energy":0.5},"tags":["rest","nap"],"whole":true}]}],
    "economy":{"startingFunds":1000}}"#;

const TOWN: &str = r#"{"width":12,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[6.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"windowSeat","x":4,"z":1}],
    "sims":[{"name":"Ada","x":3.5,"z":7.5},{"name":"Bea","x":6.5,"z":7.5}]}"#;

const GAZE: usize = 0;
const DOZE: usize = 1;

fn world(autonomy: bool) -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: autonomy,
        household: None,
    })
    .unwrap();
    w
}

fn using(w: &World, sim: usize) -> Option<usize> {
    match w.sims[sim].current()?.task.kind {
        TaskKind::Use { interaction, .. } => Some(interaction),
        _ => None,
    }
}

/// On the seat (holding a place on it).
fn on_seat(w: &World, sim: u32) -> bool {
    w.objects[0].users().any(|u| u == sim)
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
fn the_flag_parses() {
    let w = world(false);
    let def = &w.content.objects[0];
    assert!(def.interactions[DOZE].whole);
    assert!(!def.interactions[GAZE].whole, "off unless set");
}

#[test]
fn a_whole_nap_takes_both_places_and_keeps_a_sitter_off() {
    let mut w = world(false);
    w.apply(Command::Use {
        sim: 0,
        object: 0,
        interaction: DOZE,
    })
    .unwrap();
    assert!(
        until(&mut w, 10, |w| w.sims[0].pose
            == sim_core::content::Pose::Lie),
        "Ada lies down"
    );
    assert_eq!(
        w.objects[0].users[..2],
        [Some(0), Some(0)],
        "both places hers"
    );
    // Told to sit beside her: there's no room (the order is taken, and dropped when it comes up).
    w.apply(Command::Use {
        sim: 1,
        object: 0,
        interaction: GAZE,
    })
    .unwrap();
    assert!(!until(&mut w, 20, |w| on_seat(w, 1)), "Bea never gets on");
    assert_eq!(using(&w, 0), Some(DOZE), "Ada dozes on");
}

#[test]
fn a_whole_nap_waits_for_the_seat_to_be_empty() {
    let mut w = world(false);
    w.apply(Command::Use {
        sim: 1,
        object: 0,
        interaction: GAZE,
    })
    .unwrap();
    assert!(until(&mut w, 10, |w| using(w, 1) == Some(GAZE)
        && w.sims[1].pose == sim_core::content::Pose::Sit));
    // On their own, tired: the nap isn't chosen while Bea sits there.
    w.apply(Command::SetAutonomy {
        enabled: true,
        household: None,
    })
    .unwrap();
    w.sims[0].needs[0] = 0.05;
    assert!(!until(&mut w, 30, |w| using(w, 0) == Some(DOZE)
        || w.sims[0].pose == sim_core::content::Pose::Lie));
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    // Told to: the player hears why not.
    let err = w
        .apply(Command::Use {
            sim: 0,
            object: 0,
            interaction: DOZE,
        })
        .unwrap_err();
    assert!(
        err.to_string().contains("someone is using the window seat"),
        "{err}"
    );
    // Once she's up, it's free for a nap.
    w.apply(Command::Cancel { sim: 1, index: 0 }).unwrap();
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    assert!(until(&mut w, 5, |w| !w.objects[0].in_use()), "both off it");
    w.apply(Command::Use {
        sim: 0,
        object: 0,
        interaction: DOZE,
    })
    .unwrap();
    assert!(until(&mut w, 10, |w| w.sims[0].pose
        == sim_core::content::Pose::Lie));
}

#[test]
fn getting_up_frees_both_places_and_two_sit_at_once() {
    let mut w = world(false);
    w.apply(Command::Use {
        sim: 0,
        object: 0,
        interaction: DOZE,
    })
    .unwrap();
    assert!(until(&mut w, 10, |w| w.sims[0].pose
        == sim_core::content::Pose::Lie));
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    assert!(!w.objects[0].in_use(), "both places free");
    for sim in 0..2 {
        w.apply(Command::Use {
            sim,
            object: 0,
            interaction: GAZE,
        })
        .unwrap();
    }
    assert!(
        until(&mut w, 10, |w| (0..2).all(
            |s| using(w, s) == Some(GAZE) && w.sims[s].pose == sim_core::content::Pose::Sit
        )),
        "side by side"
    );
    assert_ne!(w.sims[0].pos, w.sims[1].pos, "each in their own place");
}
