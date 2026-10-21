//! Where residents stand, sit and lie to use things. Two standing at one thing stand apart,
//! each in front of their own slot (a basin each at a double sink, the pool players on the
//! table's two long sides), and the one user of a thing two tiles wide at its middle; a carer at
//! the crib in front of the baby; a dart thrower at the oche; a visitor at the front door,
//! facing it. Beds are got into from the side (and out of on that side, facing away), a desk's
//! chair from behind or beside it, a bike from its side, a sofa's end seat from in front of it;
//! someone on their own on a sofa takes the seat with the footstool in front. And every shipped thing several use at once standing keeps its users apart.

mod common;

use sim_core::content::Pose;
use sim_core::world::Phase;
use sim_core::{Command, TICKS_PER_MINUTE, World};
use std::f32::consts::{FRAC_PI_2, PI};

const CONTENT: &str = r#"{
    "needs":[{"id":"fun","label":"Fun","decayPerHour":0.01},{"id":"energy","label":"Energy","decayPerHour":0.01}],
    "objects":[
      {"id":"sink2","name":"Double sink","footprint":[2,1],"slots":2,
       "interactions":[{"id":"wash","label":"Wash","minutes":30,"effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"wardrobe","name":"Wardrobe","footprint":[2,1],
       "interactions":[{"id":"dress","label":"Dress","minutes":30,"effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"crib","name":"Crib","footprint":[2,1],"slots":2,
       "interactions":[{"id":"lie","label":"Lie","minutes":60,"pose":"lie","baby":true,"autonomous":false,"effects":{"energy":0.5},"tags":["rest"]},
                       {"id":"play","label":"Play","minutes":30,"effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"pool","name":"Pool table","footprint":[3,2],"slots":2,
       "places":[{"at":[-0.55,1.5],"facing":180},{"at":[0.55,-1.5]}],
       "interactions":[{"id":"pool","label":"Pool","minutes":30,"effects":{"fun":0.3},"tags":["lounge"]}]},
      {"id":"dartboard","name":"Dartboard","layer":"wall","places":[{"at":[0,2.24],"facing":180}],
       "interactions":[{"id":"darts","label":"Darts","minutes":30,"effects":{"fun":0.3},"tags":["lounge"]}]},
      {"id":"bed","name":"Double bed","footprint":[2,2],"slots":2,"approach":"side",
       "interactions":[{"id":"sleep","label":"Sleep","minutes":60,"pose":"lie","effects":{"energy":0.5},"tags":["rest"]}]},
      {"id":"single","name":"Single bed","footprint":[1,2],"approach":"side",
       "interactions":[{"id":"sleep","label":"Sleep","minutes":60,"pose":"lie","effects":{"energy":0.5},"tags":["rest"]}]},
      {"id":"desk","name":"Desk","footprint":[2,1],"approach":"behind",
       "interactions":[{"id":"work","label":"Work","minutes":30,"pose":"sit","effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"bike","name":"Exercise bike","approach":"side",
       "interactions":[{"id":"cycle","label":"Cycle","minutes":30,"pose":"sit","effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"sofa3","name":"Sofa","footprint":[3,1],"slots":3,
       "interactions":[{"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"ottoman","name":"Ottoman","footstool":true,
       "interactions":[{"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"fun":0.1},"tags":["lounge"]}]},
      {"id":"box","name":"Box"}]}"#;

fn world(walls: &str, objects: &str, sims: &str) -> World {
    let lot = format!(r#"{{"width":16,"depth":16,"walls":[{walls}],"objects":[{objects}],"sims":[{sims}]}}"#);
    let mut w = World::from_json(CONTENT, &lot, 3).unwrap();
    w.autonomy = false;
    w
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

fn using(w: &World, sim: usize) -> bool {
    w.sims[sim].current().is_some_and(|a| matches!(a.phase, Phase::Using { .. }))
}

/// Sends each of `sims` to use interaction `i` of object `object`, until all are using it.
fn all_use(w: &mut World, sims: &[u32], object: u32, i: usize) {
    for &sim in sims {
        w.apply(Command::Use { sim, object, interaction: i }).unwrap();
    }
    assert!(until(w, 15, |w| sims.iter().all(|&s| using(w, s as usize))), "all of {sims:?} use it");
}

fn near(a: [f32; 2], b: [f32; 2], within: f32) -> bool {
    (a[0] - b[0]).hypot(a[1] - b[1]) < within
}

/// How far (radians) `a` and `b` are apart as facings.
fn turn(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(2.0 * PI);
    d.min(2.0 * PI - d)
}

#[test]
fn two_at_a_double_sink_each_stand_at_a_basin() {
    // The sink on x 6..8 at z 6, facing +z: a basin in front of each tile.
    let mut w = world("", r#"{"def":"sink2","x":6,"z":6,"rot":0}"#, r#"{"name":"Ada","x":2.5,"z":9.5},{"name":"Bea","x":12.5,"z":9.5}"#);
    all_use(&mut w, &[0, 1], 0, 0);
    let (a, b) = (&w.sims[0], &w.sims[1]);
    assert!(near(a.pos, [6.5, 7.5], 0.01) && near(b.pos, [7.5, 7.5], 0.01), "one in front of each half: {:?} {:?}", a.pos, b.pos);
    assert!(turn(a.yaw, PI) < 1e-4 && turn(b.yaw, PI) < 1e-4, "facing straight in");
    // Turned round (facing -x, on z 6..8): the same, along its new front.
    let mut w = world("", r#"{"def":"sink2","x":6,"z":6,"rot":3}"#, r#"{"name":"Ada","x":2.5,"z":2.5},{"name":"Bea","x":2.5,"z":12.5}"#);
    all_use(&mut w, &[0, 1], 0, 0);
    let (a, b) = (w.sims[0].pos, w.sims[1].pos);
    assert!((a[0] - 5.5).abs() < 0.01 && (b[0] - 5.5).abs() < 0.01 && (a[1] - b[1]).abs() > 0.99, "{a:?} {b:?}");
}

#[test]
fn the_one_user_of_a_thing_two_tiles_wide_stands_at_its_middle() {
    for rot in 0..4 {
        let mut w = world("", &format!(r#"{{"def":"wardrobe","x":6,"z":6,"rot":{rot}}}"#), r#"{"name":"Ada","x":2.5,"z":12.5}"#);
        all_use(&mut w, &[0], 0, 0);
        let o = &w.objects[0];
        let c = o.centre(&w.content);
        let a = &w.sims[0];
        // Straight out in front of its middle, a tile's middle off its edge.
        let ahead = [o.yaw().sin(), o.yaw().cos()];
        let off = [a.pos[0] - c[0], a.pos[1] - c[1]];
        assert!((off[0] * ahead[1] - off[1] * ahead[0]).abs() < 0.02, "rot {rot}: in front of the middle, {:?}", a.pos);
        assert!((off[0] * ahead[0] + off[1] * ahead[1] - 1.0).abs() < 0.02, "rot {rot}: {:?}", a.pos);
        assert!(turn(a.yaw, o.yaw() + PI) < 1e-4, "rot {rot}: facing it");
    }
}

#[test]
fn a_carer_at_the_crib_stands_in_front_of_the_baby() {
    // The baby's slot is the crib's left half (slot 0); the carer gets the other slot, and in
    // every facing stands in front of the baby, not of the changing table.
    for rot in 0..4 {
        let mut w = world("", &format!(r#"{{"def":"crib","x":6,"z":6,"rot":{rot}}}"#), r#"{"name":"Ada","x":2.5,"z":12.5}"#);
        all_use(&mut w, &[0], 0, 1);
        let o = &w.objects[0];
        assert_eq!(o.users[1], Some(0), "rot {rot}: the carer's slot");
        let baby = o.slot_position(&w.content, 0);
        let a = &w.sims[0];
        let ahead = [o.yaw().sin(), o.yaw().cos()];
        let off = [a.pos[0] - baby[0], a.pos[1] - baby[1]];
        assert!((off[0] * ahead[1] - off[1] * ahead[0]).abs() < 0.02, "rot {rot}: in front of the baby, {:?} {baby:?}", a.pos);
    }
}

#[test]
fn pool_players_stand_on_the_two_long_sides_facing_each_other() {
    // The table on x 5..8, z 6..8, facing +z.
    let mut w = world("", r#"{"def":"pool","x":5,"z":6,"rot":0}"#, r#"{"name":"Ada","x":6.5,"z":12.5},{"name":"Bea","x":6.5,"z":2.5}"#);
    all_use(&mut w, &[0, 1], 0, 0);
    let (a, b) = (&w.sims[0], &w.sims[1]);
    assert!(near(a.pos, [5.95, 8.5], 0.01), "Ada on the near side: {:?}", a.pos);
    assert!(near(b.pos, [7.05, 5.5], 0.01), "Bea on the far side: {:?}", b.pos);
    assert!(turn(a.yaw, PI) < 1e-4 && turn(b.yaw, 0.0) < 1e-4, "facing each other across it");
}

#[test]
fn darts_are_thrown_from_the_oche() {
    // The board on the wall along z 4 (x 6), facing +z: its face 0.37 m in front of the wall
    // edge's... the thrower 2.6 m from it, two tiles out.
    let mut w = world("[0,4,16,4]", r#"{"def":"dartboard","x":6,"z":4,"rot":0}"#, r#"{"name":"Ada","x":2.5,"z":12.5}"#);
    all_use(&mut w, &[0], 0, 0);
    let a = &w.sims[0];
    let face = 4.5 - 0.367;
    assert_eq!(a.tile(), (6, 6), "two tiles out: {:?}", a.pos);
    assert!(((a.pos[1] - face) - 2.61).abs() < 0.01, "{:?}", a.pos);
    assert!(turn(a.yaw, PI) < 1e-4, "facing the board");
}

#[test]
fn beds_are_got_into_from_the_side_and_left_that_way() {
    // The double bed on x 6..8, z 6..8, its head at -z (facing +z). Ada (left) and Bea (right)
    // come to the tiles beside their own halves, by the foot row (z 7), not the foot end.
    let mut w = world("", r#"{"def":"bed","x":6,"z":6,"rot":0}"#, r#"{"name":"Ada","x":2.5,"z":12.5},{"name":"Bea","x":12.5,"z":12.5}"#);
    let mut from = [(0, 0); 2];
    for sim in 0..2 {
        w.apply(Command::Use { sim, object: 0, interaction: 0 }).unwrap();
    }
    assert!(until(&mut w, 15, |w| {
        for s in 0..2 {
            if w.sims[s].pose == Pose::Stand {
                from[s] = w.sims[s].tile();
            }
        }
        (0..2).all(|s| w.sims[s].pose == Pose::Lie)
    }));
    assert_eq!(from, [(5, 7), (8, 7)], "each from beside their half");
    // Woken: out on the side they got in, facing away from the bed.
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    assert!(until(&mut w, 1, |w| w.sims[0].pose == Pose::Stand));
    assert_eq!(w.sims[0].tile(), (5, 7));
    assert!(turn(w.sims[0].yaw, -FRAC_PI_2) < 1e-4, "facing out of the bed: {}", w.sims[0].yaw);

    // A single bed against a wall on its left: from its right, whichever side Ada starts on.
    let mut w = world("[6,0,6,16]", r#"{"def":"single","x":6,"z":6,"rot":0}"#, r#"{"name":"Ada","x":2.5,"z":12.5}"#);
    w.sims[0].pos = [7.5, 12.5];
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    let mut last = (0, 0);
    assert!(until(&mut w, 15, |w| {
        if w.sims[0].pose == Pose::Stand {
            last = w.sims[0].tile();
        }
        w.sims[0].pose == Pose::Lie
    }));
    assert_eq!(last, (7, 7), "from the open side");
    // Both sides blocked: over the foot end, as before.
    let mut w = world("", r#"{"def":"single","x":6,"z":6,"rot":0},{"def":"box","x":5,"z":7},{"def":"box","x":7,"z":7}"#, r#"{"name":"Ada","x":2.5,"z":12.5}"#);
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    assert!(until(&mut w, 15, |w| {
        if w.sims[0].pose == Pose::Stand {
            last = w.sims[0].tile();
        }
        w.sims[0].pose == Pose::Lie
    }));
    assert_eq!(last, (6, 8), "from the foot end");
}

#[test]
fn a_desk_chair_is_reached_from_behind_or_beside_it_not_through_the_desk() {
    // Open all round: from behind the chair.
    let mut w = world("", r#"{"def":"desk","x":6,"z":6,"rot":0}"#, r#"{"name":"Ada","x":6.5,"z":12.5}"#);
    let mut last = (0, 0);
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    assert!(until(&mut w, 15, |w| {
        if w.sims[0].pose == Pose::Stand {
            last = w.sims[0].tile();
        }
        w.sims[0].pose == Pose::Sit
    }));
    assert_eq!(last.1, 5, "from behind it: {last:?}");
    // Its back to a wall: from beside it, the side nearer them.
    let mut w = world("[0,6,16,6]", r#"{"def":"desk","x":6,"z":6,"rot":0}"#, r#"{"name":"Ada","x":12.5,"z":12.5}"#);
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    assert!(until(&mut w, 15, |w| {
        if w.sims[0].pose == Pose::Stand {
            last = w.sims[0].tile();
        }
        w.sims[0].pose == Pose::Sit
    }));
    assert_eq!(last, (8, 6), "from its right end");
    // Back out where they came from.
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    assert!(until(&mut w, 1, |w| w.sims[0].pose == Pose::Stand));
    assert_eq!(w.sims[0].tile(), (8, 6));
}

#[test]
fn a_bike_is_got_onto_from_its_side_and_a_sofas_end_seat_from_in_front_of_it() {
    let mut w = world("", r#"{"def":"bike","x":6,"z":6,"rot":0}"#, r#"{"name":"Ada","x":2.5,"z":12.5}"#);
    let mut last = (0, 0);
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    assert!(until(&mut w, 15, |w| {
        if w.sims[0].pose == Pose::Stand {
            last = w.sims[0].tile();
        }
        w.sims[0].pose == Pose::Sit
    }));
    assert_eq!(last, (5, 6), "from beside it, the side nearer");
    // Three on a sofa for three: each from the tile in front of their own seat.
    let mut w = world("", r#"{"def":"sofa3","x":5,"z":6,"rot":0}"#, r#"{"name":"Ada","x":2.5,"z":12.5},{"name":"Bea","x":6.5,"z":12.5},{"name":"Cy","x":10.5,"z":12.5}"#);
    let mut from = [(0, 0); 3];
    for sim in 0..3 {
        w.apply(Command::Use { sim, object: 0, interaction: 0 }).unwrap();
    }
    assert!(until(&mut w, 15, |w| {
        for s in 0..3 {
            if w.sims[s].pose == Pose::Stand {
                from[s] = w.sims[s].tile();
            }
        }
        (0..3).all(|s| w.sims[s].pose == Pose::Sit)
    }));
    let mut tiles = from.to_vec();
    tiles.sort();
    assert_eq!(tiles, [(5, 7), (6, 7), (7, 7)], "each in front of their own seat");
}

#[test]
fn someone_alone_on_a_sofa_takes_the_seat_with_the_footstool_in_front() {
    // The ottoman in front of the sofa's third seat: Ada, on her own, sits there; Bea, after
    // her, in the first.
    let sofa = r#"{"def":"sofa3","x":5,"z":6,"rot":0},{"def":"ottoman","x":7,"z":7,"rot":0}"#;
    let mut w = world("", sofa, r#"{"name":"Ada","x":2.5,"z":12.5},{"name":"Bea","x":6.5,"z":12.5}"#);
    all_use(&mut w, &[0], 0, 0);
    assert_eq!(w.objects[0].users[2], Some(0), "Ada before the footstool: {:?}", w.objects[0].users);
    all_use(&mut w, &[1], 0, 0);
    assert_eq!(w.objects[0].users[0], Some(1), "Bea in the first seat: {:?}", w.objects[0].users);
    // Turned round (its front to -z), the ottoman in front of the seat at x 7.5.
    let sofa = r#"{"def":"sofa3","x":5,"z":6,"rot":2},{"def":"ottoman","x":7,"z":5,"rot":0}"#;
    let mut w = world("", sofa, r#"{"name":"Ada","x":2.5,"z":1.5}"#);
    all_use(&mut w, &[0], 0, 0);
    let seat = (0..3).find(|&k| w.objects[0].users[k] == Some(0)).unwrap();
    let at = w.objects[0].slot_position(&w.content, seat);
    assert!(near(at, [7.5, 6.5], 0.1), "Ada before the footstool: seat {seat} at {at:?}");
    // No footstool: the first seat.
    let mut w = world("", r#"{"def":"sofa3","x":5,"z":6,"rot":0}"#, r#"{"name":"Ada","x":2.5,"z":12.5}"#);
    all_use(&mut w, &[0], 0, 0);
    assert_eq!(w.objects[0].users[0], Some(0));
}

/// A house with its front door on the street side: the walls round 2..12 × 1..8, the door at
/// x 7 in the front wall (z 8); the plot's entry a tile and a half out in front of it.
const VISITS: &str = r#"{
    "needs":[{"id":"social","label":"Social","decayPerHour":0.01}],
    "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},"needs":{"social":0.3}}],
    "bondPresets":{"friends":{"friendship":60}},
    "visits":{"needs":{"social":0.5},"hours":2,"minFriendship":10,"earliestHour":9,"latestHour":21,"door":{"waitMinutes":10,"welcomeFriendship":-20}},
    "objects":[],
    "economy":{"startingFunds":500}}"#;

const STREET: &str = r#"{"width":40,"depth":14,
    "walls":[[2,1,12,1],[2,8,12,8],[2,1,2,8],[12,1,12,8]],
    "doors":[{"x":7,"z":8,"axis":"x"}],
    "plots":[{"name":"West","x":0,"z":0,"w":20,"d":11,"entry":[7.5,9.5]},{"name":"East","x":20,"z":0,"w":20,"d":11,"entry":[30.5,9.5]}],
    "households":[{"name":"Ada","plot":0,"player":true},{"name":"Bo","plot":1}],
    "sims":[{"name":"Ada","household":0,"x":4.5,"z":4.5},{"name":"Bo","household":1,"x":25.5,"z":5.5}],
    "relationships":[{"a":0,"b":1,"preset":"friends"}]}"#;

#[test]
fn a_visitor_knocks_at_the_front_door_facing_it() {
    let mut w = World::from_json(VISITS, STREET, 7).unwrap();
    w.autonomy = false;
    while w.tick < sim_core::clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    w.apply(Command::Visit { sim: 1, plot: 0 }).unwrap();
    let knocking = |w: &World| w.sims[1].current().is_some_and(|a| matches!(a.phase, Phase::Knocking { .. }));
    assert!(until(&mut w, 30, knocking), "Bo knocks");
    let b = &w.sims[1];
    // An arm's length out from the doorway (its middle at 7.5, 8), facing it.
    assert!(near(b.pos, [7.5, 8.45], 0.01), "at the door: {:?}", b.pos);
    assert!(turn(b.yaw, PI) < 1e-4, "facing the door: {}", b.yaw);
}

/// The shipped things several use at once standing (content `slots` and a standing use): with
/// as many residents as slots using one, nobody stands within `APART` of another.
#[test]
fn shipped_things_used_standing_by_several_keep_them_apart() {
    const APART: f32 = 0.45;
    let content = common::content();
    let c = sim_core::Content::from_json(&content).unwrap();
    let mut checked = Vec::new();
    let mut skipped = Vec::new();
    for (d, def) in c.objects.iter().enumerate() {
        if def.slots < 2 {
            continue;
        }
        let Some(i) = def.interactions.iter().position(|it| {
            it.pose == Pose::Stand && !it.inside && !it.whole && !it.baby && it.recipe.is_none() && it.seat_near <= 0.0 && it.when.is_none() && it.leaves.is_none() && it.care.iter().all(|v| *v == 0.0)
        }) else {
            continue;
        };
        let n = def.slots.min(3) as usize;
        let outdoors = def.outdoors;
        let names = ["Ada", "Bea", "Cy"];
        let sims: Vec<String> = (0..n).map(|k| format!(r#"{{"name":"{}","household":0,"traits":["bookworm"],"x":{}.5,"z":17.5}}"#, names[k], 4 + 5 * k)).collect();
        let lot = format!(
            r#"{{"width":20,"depth":20,"plots":[{{"name":"Home","x":0,"z":0,"w":20,"d":20,"entry":[10.5,18.5],"public":{public}}}],
            "households":[{{"name":"P","plot":0,"player":true}}],"objects":[{{"def":"{id}","x":8,"z":8,"rot":0}}],"sims":[{sims}]}}"#,
            public = outdoors,
            id = def.id,
            sims = sims.join(","),
        );
        let mut w = match World::from_json(&content, &lot, 1) {
            Ok(w) => w,
            Err(e) => {
                skipped.push(format!("{}: {e}", def.id));
                continue;
            }
        };
        w.autonomy = false;
        w.households[0].funds = 100_000;
        for s in &mut w.sims {
            s.needs.iter_mut().for_each(|v| *v = 0.5);
        }
        let _ = d;
        for k in 0..n {
            let _ = w.apply(Command::Use { sim: k as u32, object: 0, interaction: i });
        }
        if !until(&mut w, 15, |w| (0..n).all(|k| using(w, k))) {
            skipped.push(format!("{}: not all of {n} got to use it", def.id));
            continue;
        }
        for a in 0..n {
            for b in a + 1..n {
                let (p, q) = (w.sims[a].pos, w.sims[b].pos);
                assert!(!near(p, q, APART), "{}: {} at {p:?} and {} at {q:?}", def.id, names[a], names[b]);
            }
        }
        checked.push(def.id.clone());
    }
    println!("checked {}: {checked:?}\nskipped: {skipped:#?}", checked.len());
    for id in ["poolTable", "foosball", "pingPongTable", "counterSinkDouble", "vanityDouble", "shopCounter", "marketStall", "bakeryCase", "basketballHoop", "trainTable"] {
        assert!(checked.iter().any(|c| c == id), "{id} was checked ({skipped:#?})");
    }
}
