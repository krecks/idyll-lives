//! Houses on lots turned round to face the street behind them (web/src/game/town.ts turns a
//! layout 180°): everything in them can still be used. A thing two tiles wide is used from one of
//! the two middle tiles in front of it; turned round, which of them counts as its front swaps, so
//! the one the house was furnished to keep clear is the other one. Either will do.

mod common;

use sim_core::content::Pose;
use sim_core::path::NavGrid;
use sim_core::surface;
use sim_core::{Command, World};

const W: i32 = 10;
const D: i32 = 10;

/// A bedroom (x 1..5, z 1..6) with its door at the bottom left: a double bed in the top corner,
/// a nightstand on the right of the tile in front of it (the left one kept clear to get into bed),
/// and a sofa against the right-hand wall with a nightstand on its lower middle tile (the upper
/// one kept clear to sit down). Turned round, everything as town.ts puts it on a lot facing the
/// other way.
fn town(turned: bool) -> String {
    let wall = |[x0, z0, x1, z1]: [i32; 4]| {
        if turned {
            [W - x0, D - z0, W - x1, D - z1]
        } else {
            [x0, z0, x1, z1]
        }
    };
    let point = |[x, z]: [f32; 2]| {
        if turned {
            [W as f32 - x, D as f32 - z]
        } else {
            [x, z]
        }
    };
    // Footprints are as at rot 0.
    let place = |def: &str, x: i32, z: i32, rot: i32, (w, d): (i32, i32)| {
        let (w, d) = if rot % 2 == 0 { (w, d) } else { (d, w) };
        let (x, z, rot) = if turned {
            (W - x - w, D - z - d, (rot + 2) % 4)
        } else {
            (x, z, rot)
        };
        format!(r#"{{"def":"{def}","x":{x},"z":{z},"rot":{rot}}}"#)
    };
    let walls: Vec<String> = [[1, 1, 5, 1], [1, 6, 5, 6], [1, 1, 1, 6], [5, 1, 5, 6]]
        .map(|w| format!("{:?}", wall(w)))
        .into();
    let (door_x, door_z) = if turned { (W - 1 - 1, D - 6) } else { (1, 6) };
    let objects = [
        place("bed", 1, 1, 0, (2, 2)),
        place("nightstand", 2, 3, 0, (1, 1)),
        // Facing -x (rot 3): used from (3, 4) as laid out.
        place("sofa", 4, 4, 3, (2, 1)),
        place("nightstand", 3, 5, 2, (1, 1)),
    ];
    let (entry, sim) = (point([1.5, 9.5]), point([1.5, 7.5]));
    format!(
        r#"{{"width":{W},"depth":{D},"walls":[{}],"doors":[{{"x":{door_x},"z":{door_z},"axis":"x"}}],
        "plots":[{{"name":"Home","x":0,"z":0,"w":{W},"d":{D},"entry":{entry:?}}}],
        "households":[{{"name":"Home","plot":0,"player":true}}],
        "objects":[{}],
        "sims":[{{"name":"Ada","traits":["bookworm"],"x":{},"z":{}}}]}}"#,
        walls.join(","),
        objects.join(","),
        sim[0],
        sim[1]
    )
}

const BED: u32 = 0;
const SOFA: u32 = 2;

#[test]
fn things_two_tiles_wide_on_a_turned_lot_can_be_reached() {
    let content = common::content();
    for turned in [false, true] {
        let w = World::from_json(&content, &town(turned), 3).unwrap();
        if turned {
            common::dump_state(&w, "turned-bedroom");
        }
        let nav = NavGrid {
            lot: &w.lot,
            blocked: w.blocked(),
            stairs: w.stair_map(),
        };
        let start = (
            w.sims[0].pos[0].floor() as i32,
            w.sims[0].pos[1].floor() as i32,
        );
        // The tile the house was furnished to keep clear in front of each.
        let kept = |x: i32, z: i32| {
            if turned {
                (W - 1 - x, D - 1 - z)
            } else {
                (x, z)
            }
        };
        for (object, tile) in [(BED, kept(1, 3)), (SOFA, kept(3, 4))] {
            let o = &w.objects[object as usize];
            let front = o.front_tile(&w.content);
            // Turned round, the front tile the simulation prefers is the one taken.
            assert_eq!(
                front == tile,
                !turned,
                "turned {turned}: the {} faces {front:?}",
                w.content.objects[o.def].id
            );
            assert_eq!(
                surface::approach_tile(&w.content, &nav, o),
                tile,
                "turned {turned}"
            );
            assert!(
                nav.find_path(start, tile).is_some(),
                "turned {turned}: can't get to {tile:?}"
            );
        }
    }
}

#[test]
fn with_both_middle_tiles_taken_it_is_still_used_from_the_front() {
    // Build mode marks the front then (the web's `approachTile` finds no free tile): nothing else
    // is tried, so a thing boxed in on both is out of reach.
    let content = common::content();
    for turned in [false, true] {
        let w = World::from_json(&content, &town(turned), 3).unwrap();
        let o = &w.objects[BED as usize];
        let front = o.front_tile(&w.content);
        let mut blocked = w.blocked().to_vec();
        for (x, z) in [front, (front.0 + 1, front.1)] {
            blocked[w.lot.tile_index(x, z)] = true;
        }
        let nav = NavGrid {
            lot: &w.lot,
            blocked: &blocked,
            stairs: w.stair_map(),
        };
        assert_eq!(
            surface::approach_tiles(&w.content, &nav, o).collect::<Vec<_>>(),
            vec![front],
            "turned {turned}"
        );
        assert_eq!(
            surface::approach_tile(&w.content, &nav, o),
            front,
            "turned {turned}"
        );
    }
}

#[test]
fn on_a_turned_lot_they_get_into_bed_from_the_free_side() {
    let content = common::content();
    let mut w = World::from_json(&content, &town(true), 3).unwrap();
    w.autonomy = false;
    let sleep = w.content.objects[w.objects[BED as usize].def]
        .interactions
        .iter()
        .position(|i| i.id == "sleep")
        .unwrap();
    w.apply(Command::Use {
        sim: 0,
        object: BED,
        interaction: sleep,
    })
    .unwrap();
    // Where they last stood before lying down.
    let mut stood = None;
    for _ in 0..20 * 60 {
        w.tick_once();
        if w.sims[0].pose == Pose::Lie {
            break;
        }
        stood = Some((
            w.sims[0].pos[0].floor() as i32,
            w.sims[0].pos[1].floor() as i32,
        ));
    }
    let s = &w.sims[0];
    assert_eq!(s.pose, Pose::Lie, "never got into bed");
    // (From beside it, by the foot row: its other side is against the wall; G-51.)
    assert_eq!(
        stood,
        Some((W - 1 - 3, D - 1 - 2)),
        "got into bed from the wrong side"
    );
    // Lying where they would in a bed whichever way it's turned.
    let bed = &w.objects[BED as usize];
    let lying = (0..2).map(|slot| bed.slot_position(&w.content, slot));
    assert!(
        lying
            .clone()
            .any(|p| (p[0] - s.pos[0]).abs() < 0.01 && (p[1] - s.pos[1]).abs() < 0.01),
        "at {:?}, not one of {:?}",
        s.pos,
        lying.collect::<Vec<_>>()
    );
}
