//! Privacy (G-72, `privacy.rs`), with the shipped content: a bathroom someone is in is kept shut
//! (others wait outside its door and go in after, or give up after an hour; kept out with the
//! bladder empty, the accident comes), visitors are refused the household's own things, and
//! *Try for a baby* is two in a double bed behind a shut door.
//!
//! `DUMP_STATES=1` writes `privacy-check`: `home(true)` (below) at 10:00 on day 1, needs at 0.8,
//! for looking at the doors in the game (`?state=privacy-check&debug`).

mod common;

use sim_core::social::EventKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, MINUTES_PER_TICK, World, snapshot};

/// A home on a 21×17 plot: a house of 19×12 tiles (its front door at the bottom) with a 3×3
/// bathroom (toilet, shower; its door at the bottom) and a 5×5 bedroom (a double bed, or none
/// and the bed out in the garden; its door at the bottom) off the hall, which has a fridge, a
/// sink, a sofa and a TV. Ada and Ben are partners, Gus their lodger; all three in the hall, no
/// jobs, free will off.
fn home(bed_in_bedroom: bool) -> World {
    home_with(bed_in_bedroom, true)
}

/// `home`, or without the house's outside walls (`walled` false): the bathroom and the bedroom
/// open onto the garden.
fn home_with(bed_in_bedroom: bool, walled: bool) -> World {
    let (house, front) = if walled { ("[1,1,20,1],[20,1,20,13],[20,13,1,13],[1,13,1,1],", r#"{"x":11,"z":13,"axis":"x"},"#) } else { ("", "") };
    let bed = if bed_in_bedroom { r#"{"def":"bed","x":10,"z":3}"# } else { r#"{"def":"bed","x":16,"z":14}"# };
    let lot = format!(
        r#"{{"width":21,"depth":17,
        "plots":[{{"name":"Home","x":0,"z":0,"w":21,"d":17,"entry":[11.5,16.5]}}],
        "households":[{{"name":"Player","plot":0,"player":true,"funds":100000}}],
        "walls":[{house}
                 [2,2,5,2],[5,2,5,5],[5,5,2,5],[2,5,2,2],
                 [9,2,14,2],[14,2,14,7],[14,7,9,7],[9,7,9,2]],
        "doors":[{front}{{"x":3,"z":5,"axis":"x"}},{{"x":11,"z":7,"axis":"x"}}],
        "objects":[{{"def":"toilet","x":2,"z":2}},{{"def":"shower","x":4,"z":2}},{bed},
                   {{"def":"fridge","x":17,"z":2}},{{"def":"sink","x":16,"z":2}},
                   {{"def":"sofa","x":16,"z":7}},{{"def":"tv","x":16,"z":10,"rot":2}}],
        "sims":[{{"name":"Ada","gender":"female","age":30,"x":6.5,"z":10.5,"traits":["cheerful"]}},
                {{"name":"Ben","gender":"male","age":31,"x":7.5,"z":10.5,"traits":["cheerful"]}},
                {{"name":"Gus","gender":"male","age":45,"x":8.5,"z":10.5,"traits":["cheerful"]}}],
        "relationships":[{{"a":0,"b":1,"preset":"partners"}},{{"a":2,"b":0,"preset":"friends"}},{{"a":2,"b":1,"preset":"friends"}}]}}"#
    );
    // (A baby every time they try, for the test.)
    let mut content: serde_json::Value = serde_json::from_str(&common::content()).unwrap();
    content["life"]["pregnancy"]["chance"] = serde_json::json!(1.0);
    let content = content.to_string();
    let mut w = World::from_json(&content, &lot, 4).unwrap();
    w.autonomy = false;
    for s in w.sims.iter_mut() {
        s.job = None;
        s.job_search_from = u32::MAX;
    }
    w
}

/// The bathroom's room id, and the bedroom's.
fn rooms(w: &World) -> (u16, u16) {
    (w.lot.room_at(3, 3), w.lot.room_at(11, 4))
}

fn room_of(w: &World, i: usize) -> u16 {
    let (x, z) = w.sims[i].tile();
    w.lot.room_at(x, z)
}

fn need(w: &World, id: &str) -> usize {
    w.content.needs.iter().position(|n| n.id == id).unwrap()
}

fn interaction(w: &World, object: u32, id: &str) -> usize {
    w.content.objects[w.objects[object as usize].def].interactions.iter().position(|i| i.id == id).unwrap()
}

/// What resident `i` is using now (not on their way to): the object and the interaction's id.
fn using(w: &World, i: usize) -> Option<(u32, &str)> {
    match w.sims[i].current() {
        Some(a) if matches!(a.phase, Phase::Using { .. }) => match a.task.kind {
            TaskKind::Use { object, interaction } => Some((object, w.content.objects[w.objects[object as usize].def].interactions[interaction].id.as_str())),
            _ => None,
        },
        _ => None,
    }
}

/// Resident `i`'s row in the snapshot.
fn row(w: &World, i: usize) -> Vec<f32> {
    let mut out = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut out);
    let o = snapshot::HEADER_LEN + i * snapshot::SIM_STRIDE;
    out[o..o + snapshot::SIM_STRIDE].to_vec()
}

fn ticks(minutes: f32) -> u32 {
    (minutes / MINUTES_PER_TICK) as u32
}

#[test]
fn someone_waits_outside_the_bathroom_in_use_and_goes_in_after() {
    let mut w = home(true);
    let (bath, _) = rooms(&w);
    let (toilet, shower) = (0, 1);
    let bladder = need(&w, "bladder");
    let hygiene = need(&w, "hygiene");
    w.sims[0].needs[hygiene] = 0.2;
    w.apply(Command::Use { sim: 0, object: shower, interaction: interaction(&w, shower, "shower") }).unwrap();
    for _ in 0..ticks(20.0) {
        w.tick_once();
        if using(&w, 0).is_some() {
            break;
        }
    }
    assert_eq!(using(&w, 0).map(|u| u.0), Some(shower), "Ada is in the shower");
    assert_eq!(row(&w, 0)[snapshot::sim::SHUTS], 1.0, "she keeps the bathroom shut");

    w.sims[1].needs[bladder] = 0.3;
    w.apply(Command::Use { sim: 1, object: toilet, interaction: interaction(&w, toilet, "use") }).unwrap();
    let (mut waited, mut sent_out, mut went) = (0, false, false);
    for _ in 0..ticks(120.0) {
        w.tick_once();
        let ada_in = room_of(&w, 0) == bath;
        if ada_in {
            assert_ne!(room_of(&w, 1), bath, "Ben went in while Ada was in there");
        }
        if w.sims[1].kept_out() {
            waited += 1;
            let r = row(&w, 1);
            assert_eq!(r[snapshot::sim::MOVING], 0.0, "standing outside, not walking");
            assert_eq!([r[snapshot::sim::WAY_X], r[snapshot::sim::WAY_Z]], [r[snapshot::sim::X], r[snapshot::sim::Z]], "no way through the door");
            let (bx, bz) = (w.sims[1].pos[0] - 3.5, w.sims[1].pos[1] - 5.0);
            assert!(bx.hypot(bz) < 2.0, "Ben waits by the door, {:?} off", (bx, bz));
        }
        // Showered: Ada comes out (free will is off, she'd stand there).
        if ada_in && w.sims[0].current().is_none() && !sent_out {
            w.apply(Command::MoveTo { sim: 0, x: 16.5, z: 11.5 }).unwrap();
            sent_out = true;
        }
        if using(&w, 1).is_some_and(|u| u.0 == toilet) {
            went = true;
            break;
        }
    }
    assert!(sent_out, "Ada finished her shower");
    assert!(waited as f32 * MINUTES_PER_TICK >= 5.0, "Ben waited outside ({} min)", waited as f32 * MINUTES_PER_TICK);
    assert!(went, "Ben went in once Ada was out");
    assert!(!w.sims[1].kept_out());
    common::check(&w);
}

#[test]
fn nobody_waits_at_a_shut_bathroom_for_ever() {
    let mut w = home(true);
    let (bath, _) = rooms(&w);
    let toilet = 0;
    // Ada stands in the bathroom (free will off: she stays there).
    w.sims[0].pos = [3.5, 3.5];
    let bladder = need(&w, "bladder");
    w.sims[1].needs[bladder] = 0.5;
    w.apply(Command::Use { sim: 1, object: toilet, interaction: interaction(&w, toilet, "use") }).unwrap();
    let mut waited = false;
    for _ in 0..ticks(sim_core::privacy::WAIT_MINUTES + 20.0) {
        w.tick_once();
        assert_ne!(room_of(&w, 1), bath, "Ben went in on Ada");
        waited |= w.sims[1].kept_out();
    }
    assert!(waited, "Ben waited outside");
    assert!(w.sims[1].current().is_none() && !w.sims[1].kept_out(), "and gave up after an hour");
}

#[test]
fn kept_out_with_an_empty_bladder_the_accident_comes() {
    let mut w = home(true);
    let toilet = 0;
    w.sims[0].pos = [3.5, 3.5];
    let bladder = need(&w, "bladder");
    w.sims[1].needs[bladder] = 0.0;
    w.apply(Command::Use { sim: 1, object: toilet, interaction: interaction(&w, toilet, "use") }).unwrap();
    let mut accident = false;
    for _ in 0..ticks(sim_core::privacy::WAIT_MINUTES) {
        w.tick_once();
        accident |= w.events.iter().any(|e| e.kind == EventKind::Accident && e.a == 1);
        if accident {
            break;
        }
    }
    assert!(accident, "Ben didn't stand at the door with an empty bladder for an hour");
}

/// Ada's home and her friend Bo's next door, with a wardrobe and a fridge at Bo's.
const STREET: &str = r#"{"width":32,"depth":16,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,13.5]},
             {"name":"Bo's","x":16,"z":0,"w":16,"d":14,"entry":[24.5,13.5]}],
    "households":[{"name":"Player","plot":0,"player":true,"funds":100000},{"name":"Bo","plot":1}],
    "objects":[{"def":"fridge","x":20,"z":3},{"def":"wardrobe","x":26,"z":3}],
    "sims":[{"name":"Ada","household":0,"x":8.5,"z":8.5,"traits":["bookworm"]},{"name":"Bo","household":1,"x":24.5,"z":8.5,"traits":["bookworm"]}],
    "relationships":[{"a":0,"b":1,"preset":"friends"}]}"#;

#[test]
fn a_visitor_is_refused_the_households_own_things() {
    let mut w = World::from_json(&common::content(), STREET, 1).unwrap();
    w.autonomy = false;
    let (fridge, wardrobe) = (0, 1);
    assert!(w.content.objects[w.objects[wardrobe as usize].def].private, "the wardrobe is the household's own");
    w.apply(Command::Visit { sim: 0, plot: 1 }).unwrap();
    // A snack at a friend's is fine; their wardrobe isn't.
    w.apply(Command::Use { sim: 0, object: fridge, interaction: interaction(&w, fridge, "snack") }).unwrap();
    let outfit = interaction(&w, wardrobe, "outfit");
    let err = w.apply(Command::Use { sim: 0, object: wardrobe, interaction: outfit }).unwrap_err();
    assert_eq!(err.to_string(), "Ada is a guest there, and the wardrobe is the household's own");
    // Bo, at home, may.
    w.apply(Command::Use { sim: 1, object: wardrobe, interaction: outfit }).unwrap();

    // Nor does a guest choose it: Ada at Bo's for a day, free will on, never at the wardrobe.
    let mut w = World::from_json(&common::content(), STREET, 2).unwrap();
    w.sims[0].pos = [22.5, 8.5];
    w.sims[0].visiting = Some(sim_core::life::Visit { plot: 1, until: u64::MAX, idle: 0, farewell: 0 });
    for _ in 0..ticks(12.0 * 60.0) {
        w.tick_once();
        assert!(using(&w, 0).is_none_or(|u| u.0 != wardrobe), "Ada changed at Bo's wardrobe");
        if w.sims[0].visiting.is_none() {
            break;
        }
    }
}

/// Partners `a` and `b` romance enough that she says yes.
fn in_love(w: &mut World) {
    for (a, b) in [(0, 1), (1, 0)] {
        w.relationships.get_mut(a, b).romance = 100.0;
    }
}

#[test]
fn trying_for_a_baby_is_in_a_double_bed_behind_a_shut_door() {
    let mut w = home(true);
    let (_, bedroom) = rooms(&w);
    let bed = 2;
    in_love(&mut w);
    let social = w.content.social_index("tryForBaby").unwrap();
    w.apply(Command::Social { sim: 0, target: 1, social }).unwrap();
    // Gus wants to come into the bedroom meanwhile.
    let (mut both, mut gus_sent, mut gus_waited, mut tried) = (false, false, false, 0);
    for _ in 0..ticks(120.0) {
        w.tick_once();
        let at = |w: &World, i: usize| using(w, i).is_some_and(|u| u == (bed, "tryForBaby"));
        if at(&w, 0) && at(&w, 1) {
            both = true;
            tried += 1;
            for i in [0, 1] {
                assert_eq!(room_of(&w, i), bedroom);
                assert_eq!(w.sims[i].pose, sim_core::content::Pose::Lie, "in the bed");
                assert_eq!(row(&w, i)[snapshot::sim::SHUTS], 1.0, "the bedroom is kept shut");
            }
            if !gus_sent {
                w.apply(Command::MoveTo { sim: 2, x: 12.5, z: 5.5 }).unwrap();
                gus_sent = true;
            }
        }
        if both && (at(&w, 0) || at(&w, 1)) {
            assert_ne!(room_of(&w, 2), bedroom, "Gus walked in on them");
        }
        gus_waited |= w.sims[2].kept_out();
        if both && !at(&w, 0) && !at(&w, 1) {
            break;
        }
    }
    assert!(both, "the two went to bed together");
    let minutes = tried as f32 * MINUTES_PER_TICK;
    assert!((25.0..=31.0).contains(&minutes), "under the covers for {minutes} minutes");
    assert!(gus_waited, "Gus waited outside the door");
    assert!(w.households[0].expecting.is_some(), "a baby is on the way");
    assert!(w.events.iter().any(|e| e.kind == EventKind::Expecting));
}

#[test]
fn no_trying_for_a_baby_without_a_bed_behind_a_door() {
    // The double bed out in the garden.
    let mut w = home(false);
    in_love(&mut w);
    let social = w.content.social_index("tryForBaby").unwrap();
    let err = w.apply(Command::Social { sim: 0, target: 1, social }).unwrap_err();
    assert!(err.to_string().contains("nowhere private"), "{err}");
    // Nor with Gus in the bedroom.
    let mut w = home(true);
    in_love(&mut w);
    w.sims[2].pos = [12.5, 5.5];
    w.tick_once();
    let err = w.apply(Command::Social { sim: 0, target: 1, social }).unwrap_err();
    assert!(err.to_string().contains("nowhere private"), "{err}");
    // Not from the bed's menu either: it's asked of a partner.
    let bed = 2;
    let err = w.apply(Command::Use { sim: 0, object: bed, interaction: interaction(&w, bed, "tryForBaby") }).unwrap_err();
    assert!(err.to_string().contains("asked of a partner"), "{err}");

    // With free will, two days in the house with the bed in the garden: no baby.
    let mut w = home(false);
    in_love(&mut w);
    w.autonomy = true;
    for _ in 0..ticks(48.0 * 60.0) {
        w.tick_once();
        for i in [0, 1] {
            let trying = w.sims[i].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Social { social: s, .. } if s == social));
            assert!(!trying, "tried for a baby with no bed behind a door");
        }
    }
    assert!(w.households[0].expecting.is_none());
}

#[test]
fn with_free_will_partners_try_for_a_baby_in_bed() {
    let mut w = home(true);
    in_love(&mut w);
    w.autonomy = true;
    // (Gus is away.)
    w.sims[2].away_until = Some(u64::MAX);
    let bed = 2;
    let mut in_bed = false;
    for _ in 0..ticks(5.0 * 24.0 * 60.0) {
        w.tick_once();
        in_bed |= using(&w, 0).is_some_and(|u| u == (bed, "tryForBaby"));
        if w.households[0].expecting.is_some() {
            break;
        }
    }
    assert!(in_bed, "they went to bed to try");
    assert!(w.households[0].expecting.is_some(), "a baby is on the way within five days");
}

#[test]
fn the_privacy_check_state() {
    let mut w = home(true);
    while w.tick < sim_core::clock::tick_at(1, 10.0 * 60.0).unwrap() {
        w.tick_once();
    }
    for s in w.sims.iter_mut() {
        for n in s.needs.iter_mut() {
            *n = 0.8;
        }
    }
    assert!(w.sims.iter().all(|s| s.here() && s.away_until.is_none()));
    common::dump_state(&w, "privacy-check");
}

#[test]
fn a_bathroom_opening_onto_the_garden_isnt_kept_shut() {
    // (Its door is the way in from outside, as a cabin's front door with the toilet in the open.)
    let mut w = home_with(true, false);
    let (bath, _) = rooms(&w);
    w.sims[0].pos = [3.5, 3.5];
    w.tick_once();
    assert_eq!(room_of(&w, 0), bath);
    assert_eq!(row(&w, 0)[snapshot::sim::SHUTS], 0.0);
    let toilet = 0;
    w.apply(Command::Use { sim: 1, object: toilet, interaction: interaction(&w, toilet, "use") }).unwrap();
    let mut went = false;
    for _ in 0..ticks(20.0) {
        w.tick_once();
        assert!(!w.sims[1].kept_out());
        went |= using(&w, 1).is_some_and(|u| u.0 == toilet);
    }
    assert!(went, "Ben went in");
}

#[test]
fn not_with_someone_else_in_the_bedroom() {
    let mut w = home(true);
    in_love(&mut w);
    let bed = 2;
    let social = w.content.social_index("tryForBaby").unwrap();
    w.apply(Command::Social { sim: 0, target: 1, social }).unwrap();
    // On their way to bed, Gus is in the bedroom already (he got there first).
    let mut moved = false;
    for _ in 0..ticks(60.0) {
        w.tick_once();
        let going = w.sims[0].current().is_some_and(|a| matches!(a.task.kind, TaskKind::Use { object, .. } if object == bed));
        if going && !moved {
            w.sims[2].pos = [12.5, 5.5];
            moved = true;
        }
        for i in [0, 1] {
            let trying = w.sims[i].current().is_some_and(|a| {
                matches!(a.task.kind, TaskKind::Use { object, .. } if object == bed) && matches!(a.phase, Phase::Using { elapsed } if elapsed > 0.0)
            });
            assert!(!trying, "they went ahead with Gus in the room");
        }
    }
    assert!(moved, "they set off for the bed");
    assert!(w.households[0].expecting.is_none());
}

/// Ada and Ben sleep in the bedroom, Gus the lodger in the front room's bed. The bedroom's their
/// own (`roomKinds[].claimed`): its door is open to whoever has a place in it — their bed — and
/// keeps the rest of the household out.
#[test]
fn their_bedroom_is_theirs_and_its_door_shuts_the_others_out() {
    let mut w = home(true);
    let (_, bedroom) = rooms(&w);
    let bed = 2;
    // Ada goes to their bed: her room keeps its door shut.
    w.apply(Command::Use { sim: 0, object: bed, interaction: interaction(&w, bed, "sleep") }).unwrap();
    for _ in 0..ticks(60.0) {
        if using(&w, 0).is_some() {
            break;
        }
        w.tick_once();
    }
    assert_eq!(using(&w, 0).map(|u| u.0), Some(bed), "Ada sleeps in their bed");
    assert_eq!(room_of(&w, 0), bedroom, "and in their bedroom");
    assert_eq!(row(&w, 0)[snapshot::sim::SHUTS], 1.0, "she keeps its door shut");
    assert_ne!(room_of(&w, 2), bedroom, "Gus sleeps in the front room's bed, out of their way");
    // Ben, her partner, has the other half of the same bed: its door is open to him, and he comes
    // in without a step of waiting.
    w.apply(Command::Use { sim: 1, object: bed, interaction: interaction(&w, bed, "sleep") }).unwrap();
    let mut ben_in = false;
    for _ in 0..ticks(60.0) {
        w.tick_once();
        assert!(!w.sims[1].kept_out(), "Ben waited at the door of his own bedroom");
        if using(&w, 1).is_some() {
            ben_in = true;
            break;
        }
    }
    assert!(ben_in, "Ben went to bed in the bedroom as well");
    assert_eq!(room_of(&w, 1), bedroom, "he slept in their bed with Ada");
    // The bed's two halves are theirs now: Gus, who has no place in their room, waits at its door
    // and never goes in.
    send_down(&mut w, 2);
    w.apply(Command::MoveTo { sim: 2, x: 12.5, z: 5.5 }).unwrap(); // and he wants back to their room
    let (mut gus_waited, mut gus_in) = (false, false);
    for _ in 0..ticks(30.0) {
        w.tick_once();
        gus_waited |= w.sims[2].kept_out();
        gus_in |= room_of(&w, 2) == bedroom;
    }
    assert!(using(&w, 0).is_some() && using(&w, 1).is_some(), "Ada and Ben are still in their bed");
    assert!(gus_waited, "Gus waited outside their door");
    assert!(!gus_in, "and never went in: the bedroom is the sleepers' own");
    common::check(&w);
}

/// The crib in their room is the baby's place, and the grown-ups' to mind it at: whoever could go
/// to the crib has the run of the room, whoever of the two sleeps there.
#[test]
fn the_crib_in_a_bedroom_opens_its_door_to_whoever_minds_the_baby() {
    let mut w = home(true);
    let (_, bedroom) = rooms(&w);
    common::buy(&mut w, "crib", 12, 5); // the baby's place, by the parents' bed
    assert!(
        w.objects.iter().any(|o| w.content.objects[o.def].id == "crib" && w.lot.room_at(o.x, o.z) == bedroom),
        "the crib stands in their bedroom"
    );
    let bed = 2;
    w.apply(Command::Use { sim: 0, object: bed, interaction: interaction(&w, bed, "sleep") }).unwrap();
    for _ in 0..ticks(60.0) {
        if using(&w, 0).is_some() {
            break;
        }
        w.tick_once();
    }
    assert_eq!(row(&w, 0)[snapshot::sim::SHUTS], 1.0, "the bedroom stays shut while Ada sleeps in it");
    send_down(&mut w, 2);
    w.apply(Command::MoveTo { sim: 2, x: 12.5, z: 6.5 }).unwrap(); // and up to the baby, beside the crib
    let mut gus_in = false;
    for _ in 0..ticks(30.0) {
        w.tick_once();
        if room_of(&w, 2) == bedroom {
            gus_in = true;
            break;
        }
    }
    assert!(gus_in, "whoever can go to the crib goes in to the baby");
    common::check(&w);
}

/// Puts resident `i` (who starts in the bedroom) down in the living room, and says so.
fn send_down(w: &mut World, i: usize) {
    let living = w.lot.room_at(5, 2);
    w.apply(Command::MoveTo { sim: i as u32, x: 5.5, z: 2.5 }).unwrap();
    for _ in 0..ticks(120.0) {
        w.tick_once();
        if room_of(w, i) == living {
            return;
        }
    }
    panic!("resident {i} never got down to the living room");
}

/// What the shipped content says: the bedroom is its people's own, the bathroom nobody's (it shuts
/// for whomever is in there). And content that claims a room it doesn't keep shut is wrong: a room
/// nothing shuts has no door to keep.
#[test]
fn a_shipped_bedroom_is_claimed_and_a_bathroom_never_is() {
    let w = home(true);
    let kind = |id: &str| w.content.room_kinds.iter().find(|k| k.id == id).expect(id).clone();
    let (bedroom, bathroom) = (kind("bedroom"), kind("bathroom"));
    assert!(bedroom.private && bedroom.claimed, "a bedroom is kept shut, and its door is the sleepers' own");
    assert!(bathroom.private && !bathroom.claimed, "nobody owns the bathroom");
    assert!(w.content.room_kinds.iter().all(|k| !k.claimed || k.private), "every claimed room is one kept shut");

    let mut content: serde_json::Value = serde_json::from_str(&common::content()).unwrap();
    let kinds = content["roomKinds"].as_array_mut().unwrap();
    let living = kinds.iter_mut().find(|k| k["id"] == "living").expect("the living room's kind is in the content");
    living["claimed"] = serde_json::json!(true);
    let lot = serde_json::json!({
        "width": 6, "depth": 6,
        "plots": [{"name": "Home", "x": 0, "z": 0, "w": 6, "d": 6, "entry": [3.5, 5.5]}],
        "households": [{"name": "Player", "plot": 0, "player": true}],
        "sims": [{"name": "Ada", "household": 0, "x": 3.5, "z": 5.5}],
    });
    let wrong =
        World::from_json(&content.to_string(), &lot.to_string(), 1).map(|_| ()).map_err(|e| e.to_string()).unwrap_err();
    assert!(wrong.contains("claimed"), "{wrong}");
}

/// The other half of the bed has the run of the room they slept in, whether they're in it or not:
/// up in the night for a drink of water, and the door stays open to them on the way back. Their
/// place there is the bed their partner lies in, not a half of it that happens to stand free.
#[test]
fn ben_wakes_in_the_night_and_walks_back_into_his_own_bedroom() {
    let mut w = home(true);
    let (bath, bedroom) = rooms(&w);
    let bed = 2;
    let sleep = interaction(&w, bed, "sleep");
    // The couple both lie down in the bedroom: its door keeps the household out behind them.
    for i in 0..2 {
        w.apply(Command::Use { sim: i, object: bed, interaction: sleep }).unwrap();
    }
    for _ in 0..ticks(60.0) {
        if using(&w, 0).is_some() && using(&w, 1).is_some() {
            break;
        }
        w.tick_once();
    }
    assert_eq!(using(&w, 0).map(|u| u.0), Some(bed), "Ada sleeps in their bed");
    assert_eq!(using(&w, 1).map(|u| u.0), Some(bed), "and Ben lies next to her in it");
    // Ben up in the night, and down to the front room (the bathroom door shuts for whoever's in it).
    send_down(&mut w, 1);
    assert_eq!(w.lot.room_at(5, 2), room_of(&w, 1), "he's down the hall in the front room");
    assert!(using(&w, 0).is_some(), "Ada stays asleep in their bed");
    // Back to bed: their door is open to him, and he waits at it as long as a step.
    w.apply(Command::Use { sim: 1, object: bed, interaction: sleep }).unwrap();
    let mut back = false;
    for _ in 0..ticks(60.0) {
        w.tick_once();
        assert!(!w.sims[1].kept_out(), "Ben waited at the door of the bedroom he slept in");
        if using(&w, 1).is_some() {
            back = true;
            break;
        }
    }
    assert!(back, "Ben went back to bed");
    assert_eq!(room_of(&w, 1), bedroom, "he lay down in their bed again");
    assert_ne!(bath, bedroom);
    common::check(&w);
}

/// A spare half of the couple's bed stands free when one of them is out of bed: that's nothing to a
/// grown-up who has no place in their room. He's kept out of it the same as when they're all of them
/// asleep in it.
#[test]
fn a_lodger_has_no_place_in_the_couples_bedroom_when_one_of_them_is_out_of_bed() {
    let mut w = home(true);
    let (_, bedroom) = rooms(&w);
    let bed = 2;
    let sleep = interaction(&w, bed, "sleep");
    // Ada down to their bed, and up again for the night: one half of the bed lies empty now.
    w.apply(Command::Use { sim: 0, object: bed, interaction: sleep }).unwrap();
    for _ in 0..ticks(60.0) {
        if using(&w, 0).is_some() {
            break;
        }
        w.tick_once();
    }
    assert_eq!(using(&w, 0).map(|u| u.0), Some(bed), "Ada sleeps in their bed");
    send_down(&mut w, 1);
    assert!(using(&w, 1).is_none(), "Ben's out of bed and down the hall");
    // Gus, who has no bed of his own in their room, wants whatever's going on in there.
    w.apply(Command::MoveTo { sim: 2, x: 12.5, z: 5.5 }).unwrap();
    let (mut waited, mut in_room) = (false, false);
    for _ in 0..ticks(60.0) {
        w.tick_once();
        waited |= w.sims[2].kept_out();
        in_room |= room_of(&w, 2) == bedroom;
    }
    assert!(using(&w, 0).is_some(), "Ada stays in their bed");
    assert!(waited, "Gus waited outside their door");
    assert!(!in_room, "and never went in: the empty half of their bed is not his place in their room");
    common::check(&w);
}
