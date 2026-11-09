//! Rooms found again round the walls that changed (`Lot::compute_rooms` after the first time)
//! agree with finding every room again (`Lot::compute_all_rooms`): the same rooms, numbered the
//! same, over 1500 random edits on a lot whose rows past the ground are chunks allocated as
//! they're built on (walls, doors, windows, fences and gates along x and z, boxes closed and
//! opened, diagonals put up, turned and taken down, several at once). And the rooms a town
//! keeps up to date (found and scored again only round what was built, where things changed,
//! for dirt where there was a mess) agree with finding and scoring them all again
//! (`World::check_rooms`), over random build and buy edits in a town, saved towns and a big world:
//! walls, boxes, diagonals, things bought, sold and moved, paint, floors, undo and redo, and time
//! passing.

use crate::common;

use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind, FacePaint, FloorPaint};
use sim_core::lot::{DiagDir, Diagonal, Edge, Lot, OUTDOORS};
use sim_core::{Command, World};

const W: usize = 160;
const D: usize = 240;
/// Rows always there; past them (an upper storey) chunks are allocated as they're built on.
const GROUND: usize = 120;

fn check(lot: &Lot, edit: usize) {
    let mut full = lot.clone();
    full.compute_all_rooms();
    assert!(
        *lot == full,
        "edit {edit}: the rooms differ from finding them all again"
    );
    assert_eq!(lot.rooms_row_major(), full.rooms_row_major(), "edit {edit}");
    assert_eq!(
        lot.room_count(),
        full.room_count(),
        "edit {edit}: room count"
    );
    for id in 1..=full.room_count() as u16 {
        assert_eq!(
            lot.room_rect(id),
            full.room_rect(id),
            "edit {edit}: room {id}'s tiles"
        );
    }
    assert!(
        lot.indoor_tiles().eq(full.indoor_tiles()),
        "edit {edit}: the chunks with rooms in them"
    );
}

#[test]
fn rooms_found_round_the_changes_agree_with_finding_them_all() {
    let mut rng = common::Lcg::new(7);
    let mut lot = Lot::with_ground(W, D, GROUND);
    lot.compute_rooms();
    let mut most_rooms = 0;
    for edit in 0..1500 {
        let batch = if rng.next() < 0.2 {
            rng.below(4) + 2
        } else {
            1
        };
        for _ in 0..batch {
            let (x, z) = (rng.below(W - 2) + 1, rng.below(D - 2) + 1);
            match rng.below(9) {
                // A run of wall, door, window, fence, gate or nothing along x or z.
                0..=2 => {
                    let kind = [
                        Edge::Wall,
                        Edge::Wall,
                        Edge::Door,
                        Edge::Window,
                        Edge::Fence,
                        Edge::Gate,
                        Edge::Open,
                    ][rng.below(7)];
                    let len = rng.below(12) + 1;
                    if rng.next() < 0.5 {
                        (x..(x + len).min(W)).for_each(|tx| lot.set_h(tx, z, kind));
                    } else {
                        (z..(z + len).min(D)).for_each(|tz| lot.set_v(x, tz, kind));
                    }
                }
                // One edge.
                3 => {
                    let kind = [Edge::Wall, Edge::Door, Edge::Open][rng.below(3)];
                    if rng.next() < 0.5 {
                        lot.set_h(x, z, kind);
                    } else {
                        lot.set_v(x, z, kind);
                    }
                }
                // A closed box (a big one now and then, over chunks not yet there), or one
                // opened up again.
                4 | 5 => {
                    let big = rng.next() < 0.1;
                    let (w, d) = if big {
                        (rng.below(40) + 20, rng.below(40) + 20)
                    } else {
                        (rng.below(6) + 2, rng.below(6) + 2)
                    };
                    let kind = if rng.next() < 0.65 {
                        Edge::Wall
                    } else {
                        Edge::Open
                    };
                    let (x1, z1) = ((x + w).min(W - 1), (z + d).min(D - 1));
                    for tx in x..x1 {
                        lot.set_h(tx, z, kind);
                        lot.set_h(tx, z1, kind);
                    }
                    for tz in z..z1 {
                        lot.set_v(x, tz, kind);
                        lot.set_v(x1, tz, kind);
                    }
                }
                // A diagonal wall, door or nothing across a tile.
                6 | 7 => {
                    let d = match rng.below(5) {
                        0 => None,
                        k => Some(Diagonal {
                            dir: if k % 2 == 1 { DiagDir::Dp } else { DiagDir::Dn },
                            edge: if k == 4 { Edge::Door } else { Edge::Wall },
                        }),
                    };
                    lot.set_diag(x, z, d);
                }
                // A diamond of diagonals (a room of half tiles), or one taken down.
                _ => {
                    let on = rng.next() < 0.7;
                    let dg = |dir| {
                        on.then_some(Diagonal {
                            dir,
                            edge: Edge::Wall,
                        })
                    };
                    let (x, z) = (x.clamp(2, W - 4), z.clamp(2, D - 4));
                    for (dx, dz, dir) in [
                        (0, 1, DiagDir::Dn),
                        (1, 0, DiagDir::Dn),
                        (2, 0, DiagDir::Dp),
                        (3, 1, DiagDir::Dp),
                        (3, 2, DiagDir::Dn),
                        (2, 3, DiagDir::Dn),
                        (1, 3, DiagDir::Dp),
                        (0, 2, DiagDir::Dp),
                    ] {
                        lot.set_diag(x + dx - 1, z + dz - 1, dg(dir));
                    }
                }
            }
        }
        lot.compute_rooms();
        check(&lot, edit);
        most_rooms = most_rooms.max(lot.room_count());
    }
    assert!(most_rooms > 20, "the edits made rooms ({most_rooms})");
    assert!(
        (0..W as i32).any(|x| (GROUND as i32..D as i32).any(|z| lot.room_at(x, z) != OUTDOORS)),
        "rooms past the ground rows"
    );
}

/// One random build or buy edit by household `h` on its home (many are refused: someone in the
/// way, shut in, nothing to stand on upstairs; what a refused one changed is put back).
fn edit(w: &mut World, rng: &mut common::Lcg, h: usize) {
    let plot = w.households[h].plot.expect("a home") as usize;
    let p = w.plots[plot].clone();
    let storeys: Vec<i32> = w.storey_range().collect();
    let storey = storeys[rng.below(storeys.len())];
    let row = w.storey_row(storey);
    let (x, z) = (
        p.x + rng.below(p.w.max(1) as usize) as i32,
        p.z + rng.below(p.d.max(1) as usize) as i32 + row,
    );
    let household = h as u32;
    let kinds = [
        EdgeKind::Wall,
        EdgeKind::Wall,
        EdgeKind::Door,
        EdgeKind::Window,
        EdgeKind::Open,
        EdgeKind::Open,
        EdgeKind::Fence,
        EdgeKind::Gate,
    ];
    let cmd = match rng.below(20) {
        // A run of edges along x or z.
        0..=4 => {
            let kind = kinds[rng.below(kinds.len())];
            let len = rng.below(8) as i32 + 1;
            let along = rng.next() < 0.5;
            let edits = (0..len)
                .map(|k| {
                    if along {
                        EdgeEdit::new(EdgeAxis::H, x + k, z, kind)
                    } else {
                        EdgeEdit::new(EdgeAxis::V, x, z + k, kind)
                    }
                })
                .collect();
            Command::Build { household, edits }
        }
        // A closed box, with a door now and then, or one taken down.
        5 | 6 => {
            let (bw, bd) = (rng.below(5) as i32 + 2, rng.below(5) as i32 + 2);
            let kind = if rng.next() < 0.7 {
                EdgeKind::Wall
            } else {
                EdgeKind::Open
            };
            let mut edits = Vec::new();
            for k in 0..bw {
                edits.push(EdgeEdit::new(EdgeAxis::H, x + k, z, kind));
                edits.push(EdgeEdit::new(EdgeAxis::H, x + k, z + bd, kind));
            }
            for k in 0..bd {
                edits.push(EdgeEdit::new(EdgeAxis::V, x, z + k, kind));
                edits.push(EdgeEdit::new(EdgeAxis::V, x + bw, z + k, kind));
            }
            if kind == EdgeKind::Wall && rng.next() < 0.5 {
                edits.push(EdgeEdit::new(EdgeAxis::H, x, z + bd, EdgeKind::Door));
            }
            Command::Build { household, edits }
        }
        // A diagonal.
        7 => {
            let axis = if rng.next() < 0.5 {
                EdgeAxis::Dp
            } else {
                EdgeAxis::Dn
            };
            let kind = [EdgeKind::Wall, EdgeKind::Door, EdgeKind::Open][rng.below(3)];
            Command::Build {
                household,
                edits: vec![EdgeEdit::new(axis, x, z, kind)],
            }
        }
        // Something bought.
        8..=10 => {
            let priced: Vec<String> = w
                .content
                .objects
                .iter()
                .filter(|o| o.price.is_some())
                .map(|o| o.id.clone())
                .collect();
            Command::Buy {
                household,
                object: priced[rng.below(priced.len())].clone(),
                at: Some([x, z, rng.below(4) as i32]),
                colors: Default::default(),
                turn: None,
                offset: None,
                size: None,
                tall: None,
            }
        }
        // Something sold or moved.
        11 | 12 => {
            let mine: Vec<u32> = w
                .objects
                .iter()
                .filter(|o| w.plot_at(o.x, o.z) == Some(plot as u32))
                .map(|o| o.stable)
                .collect();
            if mine.is_empty() {
                return;
            }
            let object = mine[rng.below(mine.len())];
            if rng.next() < 0.5 {
                Command::Sell { household, object }
            } else {
                Command::MoveObject {
                    household,
                    object,
                    x,
                    z,
                    rot: rng.below(4) as u8,
                    turn: None,
                    offset: None,
                    size: None,
                    tall: None,
                }
            }
        }
        // Walls painted, floors covered.
        13 => {
            let coverings = w.content.build.coverings.len();
            let faces = (0..rng.below(6) + 1)
                .map(|k| FacePaint {
                    axis: if k % 2 == 0 { EdgeAxis::H } else { EdgeAxis::V },
                    x: x + k as i32 / 2,
                    z,
                    side: rng.below(2) as u8,
                    covering: rng.below(coverings + 1) as u8,
                })
                .collect();
            Command::Paint { household, faces }
        }
        14 => {
            let floors = w.content.build.floors.len();
            let tiles = (0..rng.below(6) + 1)
                .map(|k| FloorPaint {
                    x: x + k as i32,
                    z,
                    covering: rng.below(floors + 1) as u8,
                })
                .collect();
            Command::PaintFloor { household, tiles }
        }
        15 | 16 => Command::Undo { household },
        17 => Command::Redo { household },
        // Time moves on: messes, breakages; the edits kept for undo go.
        _ => {
            for _ in 0..rng.below(40) + 1 {
                w.tick_once();
            }
            return;
        }
    };
    let _ = w.apply(cmd);
}

/// Random edits on the homes of a town, each checked against finding and scoring every room
/// again (`World::check_rooms`), the rooms kept as they were kept between checks.
fn edits_agree(name: &str, mut w: World, seed: u64, edits: usize) {
    let homes: Vec<usize> = (0..w.households.len())
        .filter(|&h| w.households[h].plot.is_some())
        .collect();
    assert!(!homes.is_empty(), "{name}: homes");
    w.tick_once();
    w.check_rooms()
        .unwrap_or_else(|e| panic!("{name}, as loaded: {e}"));
    let mut rng = common::Lcg::new(seed);
    let mut changed = 0;
    // Mostly one home, so its edits build on each other; now and then another.
    let mut h = homes[0];
    for k in 0..edits {
        if rng.next() < 0.1 {
            h = homes[rng.below(homes.len())];
        }
        let before = w.lot.room_count();
        edit(&mut w, &mut rng, h);
        if rng.next() < 0.2 {
            // Several edits before the rooms catch up.
            edit(&mut w, &mut rng, h);
        }
        w.check_rooms()
            .unwrap_or_else(|e| panic!("{name}, edit {k}: {e}"));
        changed += usize::from(before != w.lot.room_count());
    }
    assert!(
        changed > edits / 50,
        "{name}: the edits changed the rooms ({changed} times)"
    );
}

#[test]
fn rooms_kept_up_to_date_agree_with_finding_them_all_in_a_town() {
    let content = common::content();
    let town = common::town(&content, 3, 2);
    let w = World::from_json(&content, &town, 3).unwrap();
    edits_agree("a town", w, 11, 600);
}

#[test]
fn rooms_kept_up_to_date_agree_with_finding_them_all_in_saved_towns() {
    let content = common::content();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/saves");
    for (k, name) in ["v042_village.json", "live_day5.json"].iter().enumerate() {
        let json = std::fs::read_to_string(dir.join(name)).unwrap();
        let w = World::from_save_json(&content, &json).unwrap();
        edits_agree(name, w, 20 + k as u64, 300);
    }
}

#[test]
fn rooms_kept_up_to_date_agree_with_finding_them_all_in_a_big_world() {
    let content = common::content();
    let (town, _) = common::hamlets::grid(&content, 5, 256, 6, 40, true);
    let w = World::from_json(&content, &town, 5).unwrap();
    edits_agree("a big world", w, 31, 400);
}
