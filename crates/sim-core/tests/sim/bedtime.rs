//! Who sleeps where (F8 report 2026-11-11, 12 Birch Road): a couple and their son of eighteen in
//! a house with one double bed and a race car bed (a child's). The parents' bed is theirs: the son
//! with no bed of his own sleeps on the sofa rather than in their bed (and leaving his father to
//! fall asleep on the landing, kept out of the room); a race car bed is no bed place for a
//! teenager, so a household grown out of one is told (the player's) or swaps it for a bed (the
//! neighbours'); and worn out with the bladder low, the toilet comes first and then bed, not the
//! one broken off for the other until they fall asleep on the floor.

use crate::common;

use sim_core::social::EventKind;
use sim_core::world::TaskKind;
use sim_core::{World, clock};

/// A living room with a sofa (the front door to the east), and two bedrooms off it to the south:
/// the parents' with the double bed, the son's with the race car bed. Jonah and Elif are
/// partners, Ravi (18) their son.
const HOUSE: &str = r#"{"width":16,"depth":15,
    "plots":[{"name":"12 Birch Road","x":0,"z":0,"w":16,"d":15,"entry":[14.5,4.5]}],
    "households":[{"name":"Nakamura","plot":0,"player":true}],
    "walls":[[1,1,13,1],[1,1,1,13],[13,1,13,13],[1,13,13,13],[1,7,13,7],[7,7,7,13]],
    "doors":[{"x":13,"z":4,"axis":"z"},{"x":3,"z":7,"axis":"x"},{"x":9,"z":7,"axis":"x"}],
    "objects":[{"def":"sofa","x":3,"z":2},{"def":"bed","x":2,"z":10},{"def":"kidsBedCar","x":9,"z":10}],
    "sims":[{"name":"Jonah","traits":["bookworm"],"x":5.5,"z":4.5,"age":58},
            {"name":"Elif","traits":["bookworm"],"x":6.5,"z":4.5,"age":58},
            {"name":"Ravi","traits":["bookworm"],"x":7.5,"z":4.5,"age":18}],
    "relationships":[{"a":0,"b":1,"preset":"partners"},
                     {"a":2,"b":0,"preset":"parent"},{"a":2,"b":1,"preset":"parent"}]}"#;

const SOFA: u32 = 0;
const BED: u32 = 1;
const CAR_BED: u32 = 2;

fn need(w: &World, id: &str) -> usize {
    w.content.needs.iter().position(|n| n.id == id).unwrap()
}

fn fell_asleep(w: &World) -> usize {
    w.content
        .accidents
        .iter()
        .position(|a| a.id == "fellAsleep")
        .unwrap()
}

/// Who of `sims` fell asleep on the floor from tick `since` on, from the story.
fn on_the_floor(w: &World, sims: &[u32], since: u64) -> Vec<u32> {
    let floor = fell_asleep(w) as i64;
    w.events
        .iter()
        .filter(|e| {
            e.kind == EventKind::Accident
                && e.n == Some(floor)
                && sims.contains(&e.a)
                && e.tick >= since
        })
        .map(|e| e.a)
        .collect()
}

/// The object `sim` sleeps in now (a use tagged as sleep, lying in it), if any.
fn sleeping_in(w: &World, sim: usize) -> Option<u32> {
    let s = &w.sims[sim];
    if !s.asleep(&w.content) {
        return None;
    }
    match s.current()?.task.kind {
        TaskKind::Use { object, .. } => Some(object),
        _ => None,
    }
}

/// `HOUSE` (or `town`) at half past nine on day 1, the family in the living room (a day without
/// free will, fed and rested) and fed and rested but for energy: the son worn out (off to bed at once), his parents
/// tired (to bed at bedtime).
fn evening(town: &str) -> World {
    let mut w = World::from_json(&common::content(), town, 3).unwrap();
    w.autonomy = false;
    let (n, energy) = (w.content.needs.len(), need(&w, "energy"));
    // (Fed and rested every hour: nobody falls asleep on the floor in the day.)
    for hour in 9..=21 {
        common::run_to(&mut w, 1, (hour as f32).min(21.5));
        for s in &mut w.sims {
            s.needs[..n].fill(1.0);
        }
    }
    common::run_to(&mut w, 1, 21.5);
    w.autonomy = true;
    for (i, s) in w.sims.iter_mut().enumerate() {
        s.needs[..n].fill(1.0);
        s.needs[energy] = if i == 2 { 0.1 } else { 0.45 };
    }
    w
}

#[test]
fn the_parents_share_their_bed_and_their_grown_son_sleeps_on_the_sofa() {
    let mut w = evening(HOUSE);
    let start = w.tick;
    assert_eq!(w.content.objects[w.objects[BED as usize].def].id, "bed");
    let mut slept: [Option<u32>; 3] = [None; 3];
    while w.tick < clock::tick_at(2, 3.0 * 60.0).unwrap() {
        w.tick_once();
        for (s, place) in slept.iter_mut().enumerate() {
            if let Some(o) = sleeping_in(&w, s) {
                assert!(
                    s == 2 || o == BED,
                    "{} sleeps somewhere but the bed",
                    w.sims[s].name
                );
                assert!(s < 2 || o != BED, "Ravi in his parents' bed");
                place.get_or_insert(o);
            }
        }
    }
    assert_eq!(
        slept,
        [Some(BED), Some(BED), Some(SOFA)],
        "the parents in their bed, their son on the sofa"
    );
    assert!(
        on_the_floor(&w, &[0, 1, 2], start).is_empty(),
        "nobody on the floor"
    );
    let sofa = &w.content.objects[w.objects[SOFA as usize].def];
    assert!(
        sofa.interactions.iter().any(|i| {
            i.spare && i.id == "sleep" && i.whole && i.pose == sim_core::content::Pose::Lie
        }),
        "a night on the sofa is a spare bed's, lain along the whole of it"
    );
}

#[test]
fn a_sofa_somebody_sits_on_is_still_the_son_s_for_the_night() {
    // His father watching TV on the sofa (sat on, it isn't free to lie along): the son waits for it
    // rather than take his parents' bed (he took it, and his father, kept out of the room, fell asleep
    // on the landing).
    let mut w = evening(HOUSE);
    let energy = need(&w, "energy");
    w.sims[0].needs[energy] = 0.9;
    let relax = common::interaction(&w, SOFA, "relax");
    w.apply(sim_core::Command::Use {
        sim: 0,
        object: SOFA,
        interaction: relax,
    })
    .unwrap();
    let mut sat = false;
    let mut his = None;
    while w.tick < clock::tick_at(2, 1.0 * 60.0).unwrap() {
        w.tick_once();
        sat |= w.objects[SOFA as usize].users().any(|u| u == 0);
        assert_ne!(sleeping_in(&w, 2), Some(BED), "Ravi in his parents' bed");
        his = his.or(sleeping_in(&w, 2));
    }
    assert!(sat, "his father sat on the sofa");
    assert_eq!(his, Some(SOFA), "Ravi on the sofa once it's free");
}

#[test]
fn with_nowhere_else_the_son_sleeps_in_the_bed_rather_than_on_the_floor() {
    // No sofa: the parents' bed is all there is for him (they aren't tired yet).
    let town = HOUSE.replace(r#"{"def":"sofa","x":3,"z":2},"#, "");
    let mut w = evening(&town);
    let energy = need(&w, "energy");
    for s in &mut w.sims[..2] {
        s.needs[energy] = 0.9;
    }
    let bed = BED - 1;
    let mut his = None;
    while w.tick < clock::tick_at(2, 1.0 * 60.0).unwrap() {
        w.tick_once();
        his = his.or(sleeping_in(&w, 2));
    }
    assert_eq!(
        his,
        Some(bed),
        "Ravi takes a half of the bed before the floor"
    );
}

#[test]
fn a_race_car_bed_is_no_bed_place_for_a_teenager() {
    let at = |age: f32| HOUSE.replace(r#""age":18"#, &format!(r#""age":{age}"#));
    let told = |town: &str| {
        let mut w = World::from_json(&common::content(), town, 3).unwrap();
        w.tick_once();
        w.events
            .iter()
            .filter(|e| e.kind == EventKind::NoBed)
            .map(|e| e.a)
            .collect::<Vec<_>>()
    };
    assert!(told(&at(8.0)).is_empty(), "a child of eight sleeps in it");
    assert_eq!(told(&at(15.0)), [2], "at fifteen Ravi has no bed");
    assert_eq!(told(&at(18.0)), [2], "nor at eighteen");
}

/// The house next door to the player's, a child about to be a teenager in the race car bed.
fn next_door() -> String {
    HOUSE.replace(r#""age":18"#, r#""age":12.6"#).replace(
        r#""households":[{"name":"Nakamura","plot":0,"player":true}]"#,
        r#""households":[{"name":"Nakamura","plot":0}]"#,
    )
}

#[test]
fn neighbours_swap_an_outgrown_race_car_bed_for_a_bed() {
    let mut w = World::from_json(&common::content(), &next_door(), 3).unwrap();
    let funds = w.households[0].funds;
    let teen = w
        .content
        .life
        .stages
        .iter()
        .position(|s| s.id == "teen")
        .unwrap();
    assert!(w.content.life.stage(w.sims[2].age) < teen);
    common::run_to(&mut w, 2, 1.0);
    assert_eq!(
        w.content.life.stage(w.sims[2].age),
        teen,
        "Ravi is a teenager now"
    );
    let defs: Vec<&str> = w
        .objects
        .iter()
        .map(|o| w.content.objects[o.def].id.as_str())
        .collect();
    assert!(
        !defs.contains(&"kidsBedCar"),
        "the race car bed is gone: {defs:?}"
    );
    let sleep = w.content.day_rhythm.sleep_tags;
    let new: Vec<&str> = defs
        .iter()
        .copied()
        .filter(|&d| !["sofa", "bed"].contains(&d))
        .filter(|&d| {
            let def = w.content.objects.iter().find(|o| o.id == d).unwrap();
            def.interactions.iter().any(|i| {
                i.tags & sleep != 0
                    && !i.spare
                    && (teen..w.content.life.stages.len()).all(|s| i.for_stage(s))
            })
        })
        .collect();
    assert_eq!(
        new.len(),
        1,
        "a bed for him now and when he's grown: {defs:?}"
    );
    assert!(w.households[0].funds < funds, "bought (the old one sold)");
    assert!(
        !w.events.iter().any(|e| e.kind == EventKind::NoBed),
        "nobody's told: it's done"
    );
}

#[test]
fn the_player_is_told_a_teenager_has_outgrown_the_race_car_bed() {
    let town = HOUSE.replace(r#""age":18"#, r#""age":12.6"#);
    let mut w = World::from_json(&common::content(), &town, 3).unwrap();
    common::run_to(&mut w, 2, 1.0);
    assert_eq!(
        w.content.objects[w.objects[CAR_BED as usize].def].id, "kidsBedCar",
        "the player's things are the player's"
    );
    let told: Vec<u32> = w
        .events
        .iter()
        .filter(|e| e.kind == EventKind::NoBed)
        .map(|e| e.a)
        .collect();
    assert_eq!(told, [2], "the player is told Ravi has no bed, once");
}

#[test]
fn worn_out_with_the_bladder_low_the_toilet_comes_first_then_bed() {
    // One resident, a bed and a toilet: the toilet isn't broken off for bed (which a bladder
    // that low keeps them from: up from one for the other, round and round, in Ravi's two-storey
    // home until he fell asleep on the floor).
    let town = r#"{"width":14,"depth":12,
        "plots":[{"name":"Home","x":0,"z":0,"w":14,"d":12,"entry":[7.5,11.5]}],
        "households":[{"name":"Home","plot":0,"player":true}],
        "objects":[{"def":"bedSingle","x":1,"z":1},{"def":"toilet","x":10,"z":1}],
        "sims":[{"name":"Ravi","traits":["bookworm"],"x":6.5,"z":6.5,"age":17}]}"#;
    let mut w = World::from_json(&common::content(), town, 3).unwrap();
    // (Up and about at 11 pm: a day without free will.)
    w.autonomy = false;
    common::run_to(&mut w, 1, 23.0);
    w.autonomy = true;
    let n = w.content.needs.len();
    let (energy, bladder) = (need(&w, "energy"), need(&w, "bladder"));
    w.sims[0].needs[..n].fill(1.0);
    w.sims[0].needs[energy] = 0.0;
    w.sims[0].needs[bladder] = 0.05;
    let start = w.tick;
    let on_toilet = |w: &World| {
        w.sims[0].current().is_some_and(|a| {
            matches!(a.task.kind, TaskKind::Use { object: 1, .. })
                && matches!(a.phase, sim_core::world::Phase::Using { .. })
        })
    };
    assert!(common::until(&mut w, 60, on_toilet), "the toilet first");
    assert!(common::until(&mut w, 30, |w| !on_toilet(w)), "and up again");
    assert!(
        w.sims[0].needs[bladder] > 0.9,
        "not up from it at once for bed: bladder {}",
        w.sims[0].needs[bladder]
    );
    let mut asleep = false;
    while w.tick < clock::tick_at(2, 2.0 * 60.0).unwrap() && !asleep {
        w.tick_once();
        asleep = sleeping_in(&w, 0) == Some(0);
    }
    assert!(asleep, "in bed by two");
    assert!(on_the_floor(&w, &[0], start).is_empty(), "not on the floor");
}

/// The owner's town (Sunnybury, F8 report 2026-11-11 14:47, the same game 30 minutes of play
/// before the report at 15:16): 20:38 on day 1, the Nakamuras of 12 Birch Road (household 36:
/// Jonah 109 and Elif 110, partners of 57, their son Ravi 111, 17) on their way home. Their home
/// has a double bed and a race car bed: Ravi, too old for that, fell asleep on the floor the
/// first night and his father the next two (kept out of the bedroom Ravi was in). To 1 am on day
/// 3, after Ravi turns 18 at midnight: the parents in their bed, Ravi on the sofa, nobody on the
/// floor. (Since most junctions give way, Ravi's walk home goes along the roads, not over open
/// ground as when every junction's box was kept off those on foot: home at half past one, not a
/// quarter to twelve. Worn out for two hours of it, he dropped on the doorstep; he sets off with
/// energy enough to get home on, as he did in the owner's game: what's tested is where they
/// sleep there. With 0.4 he ran out at 1:17, a quarter of an hour short of the door, and what he
/// did there with none left (the toilet and the sofa, or an espresso at the café and off to work
/// at four, asleep on the way) came down to the town's one dice roll, which every resident's
/// choices anywhere move: the junctions' changes moved it, 2026-11-13. With 0.6 he gets in with
/// some left.)
#[test]
fn nobody_on_birch_road_sleeps_on_the_floor() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/saves/v037_birch_road.json");
    let json = std::fs::read_to_string(&path).unwrap();
    let mut w = World::from_save_json(&common::content(), &json).unwrap();
    let [jonah, elif, ravi] = [109, 110, 111];
    assert_eq!(
        [&w.sims[jonah].name, &w.sims[elif].name, &w.sims[ravi].name],
        ["Jonah", "Elif", "Ravi"]
    );
    let energy = need(&w, "energy");
    w.sims[ravi].needs[energy] = w.sims[ravi].needs[energy].max(0.6);
    let plot = w.households[36].plot.unwrap();
    common::look_at(&mut w, plot);
    let end = clock::tick_at(3, 60.0).unwrap();
    let mut beds: [Vec<String>; 3] = Default::default();
    while w.tick < end {
        w.tick_once();
        for (k, s) in [jonah, elif, ravi].into_iter().enumerate() {
            if let Some(o) = sleeping_in(&w, s) {
                let def = &w.content.objects[w.objects[o as usize].def].id;
                if !beds[k].contains(def) {
                    beds[k].push(def.clone());
                }
            }
        }
    }
    assert!(
        on_the_floor(&w, &[jonah as u32, elif as u32, ravi as u32], 0).is_empty(),
        "nobody of the household on the floor"
    );
    assert_eq!(
        beds,
        [vec!["bed"], vec!["bed"], vec!["sofa"]],
        "who slept where"
    );
    for s in [jonah, elif, ravi] {
        assert!(
            w.sims[s].asleep(&w.content),
            "{} asleep at 1 am",
            w.sims[s].name
        );
    }
    assert!(
        w.events
            .iter()
            .any(|e| e.kind == EventKind::NoBed && e.a == ravi as u32),
        "the player is told Ravi has no bed"
    );
}
