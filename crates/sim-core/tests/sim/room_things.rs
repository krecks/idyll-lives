//! Things bought, sold, moved, upgraded and mended score the rooms again for what stands in
//! them without finding the rooms again (`World::catch_up_rooms`, `rooms::rescore_things`):
//! exactly what finding them again (`World::refresh_rooms`) gives, room by room, tile by tile,
//! after every edit, with time passing in between (dirt, which the rescore adds up afresh).

use crate::common;

use sim_core::{Command, World};

#[test]
fn rescoring_for_things_matches_finding_the_rooms_again() {
    let content = common::content();
    let town = common::town(&content, 3, 2);
    let mut a = World::from_json(&content, &town, 3).unwrap();
    let mut b = World::from_json(&content, &town, 3).unwrap();
    let mut rng = common::Lcg::new(9);
    let home = a.plots[a.households[0].plot.unwrap() as usize].clone();
    let defs = ["plant", "bookshelf", "chessTable", "bedBunk", "crib"];
    let mut edits = 0;
    for step in 0..300 {
        let roll = rng.next();
        // (By stable id, as commands name them.)
        let mine: Vec<u32> = a
            .objects
            .iter()
            .filter(|o| {
                o.x >= home.x && o.z >= home.z && o.x < home.x + home.w && o.z < home.z + home.d
            })
            .map(|o| o.stable)
            .collect();
        let pick = |rng: &mut common::Lcg, n: usize| (rng.next() * n as f64) as usize % n.max(1);
        let cmd = if roll < 0.45 || mine.is_empty() {
            let def = defs[pick(&mut rng, defs.len())];
            let x = home.x + pick(&mut rng, home.w as usize) as i32;
            let z = home.z + pick(&mut rng, home.d as usize) as i32;
            Command::Buy {
                household: 0,
                object: def.into(),
                at: Some([x, z, 0]),
                colors: Default::default(),
                turn: None,
                offset: None,
                size: None,
                tall: None,
            }
        } else {
            let object = mine[pick(&mut rng, mine.len())];
            if roll < 0.65 {
                Command::Sell {
                    household: 0,
                    object,
                }
            } else if roll < 0.85 {
                Command::MoveObject {
                    household: 0,
                    object,
                    x: home.x + pick(&mut rng, home.w as usize) as i32,
                    z: home.z + pick(&mut rng, home.d as usize) as i32,
                    rot: pick(&mut rng, 4) as u8,
                    turn: None,
                    offset: None,
                    size: None,
                    tall: None,
                }
            } else {
                Command::Upgrade {
                    household: 0,
                    object,
                }
            }
        };
        let (ra, rb) = (a.apply(cmd.clone()), b.apply(cmd));
        assert_eq!(ra.is_ok(), rb.is_ok(), "step {step}");
        if ra.is_ok() {
            edits += 1;
            // `a` caught up by rescoring for what stands where (as `apply` does); `b` finds its
            // rooms again from scratch.
            b.refresh_rooms();
            let (sa, sb) = (a.room_state(), b.room_state());
            if sa != sb {
                let room = (0..sa.0.len().max(sb.0.len()))
                    .find(|&k| sa.0.get(k) != sb.0.get(k))
                    .map(|k| (sa.0.get(k).cloned(), sb.0.get(k).cloned()));
                let tile = (0..sa.1.len()).find(|&i| sa.1[i].to_bits() != sb.1[i].to_bits());
                panic!(
                    "step {step}: the rooms differ: {room:#?}, tile {tile:?}, private {} {}",
                    sa.2 == sb.2,
                    sa.3 == sb.3
                );
            }
        }
        // A little time between edits: dirt, things used (the same in both, as they're the same).
        for _ in 0..(rng.next() * 40.0) as usize {
            a.tick_once();
            b.tick_once();
        }
    }
    assert!(edits > 100, "only {edits} edits went through");
}
