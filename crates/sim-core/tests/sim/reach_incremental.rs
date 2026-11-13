//! The walkable regions kept up to date chunk by chunk (`path::Reach::mark` and `refresh`) agree
//! with labelling the whole lot again (`Reach::build`): 500 random edits on a 256 × 256 lot
//! (things put down and taken away, walls, doors and fences, diagonal walls), each checked on
//! random pairs of tiles, and now and then against a search.

use crate::common;

use sim_core::lot::{DiagDir, Diagonal, Edge, Lot};
use sim_core::path::{NO_STAIRS, NavGrid, Reach};

const SIZE: usize = 256;

fn nav<'a>(lot: &'a Lot, blocked: &'a [bool]) -> NavGrid<'a> {
    NavGrid {
        lot,
        blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
    }
}

/// A region label per tile (row by row; `u32::MAX` where nobody stands), by a flood over every
/// tile with the steps walkers take.
fn flood(nav: &NavGrid) -> Vec<u32> {
    let mut label = vec![u32::MAX; SIZE * SIZE];
    let mut next = 0;
    for start in 0..SIZE * SIZE {
        let (x, z) = ((start % SIZE) as i32, (start / SIZE) as i32);
        if label[start] != u32::MAX || !nav.tile_free(x, z) {
            continue;
        }
        label[start] = next;
        let mut queue = vec![(x, z)];
        while let Some((x, z)) = queue.pop() {
            for (nx, nz) in nav.neighbours(x, z) {
                let i = nz as usize * SIZE + nx as usize;
                if label[i] == u32::MAX {
                    label[i] = next;
                    queue.push((nx, nz));
                }
            }
        }
        next += 1;
    }
    label
}

#[test]
#[cfg_attr(not(feature = "slow-tests"), ignore = "slow: --features slow-tests")]
fn incremental_reach_agrees_with_a_full_rebuild() {
    let mut rng = common::Lcg::new(42);
    let mut lot = Lot::new(SIZE, SIZE);
    let mut blocked = vec![false; lot.tile_slots()];
    // A start with some walls: a few closed boxes and long walls with gaps.
    for k in 0..12 {
        let (x0, z0) = (8 + k * 20, 10 + (k * 37) % 200);
        for x in x0..x0 + 10 {
            lot.set_h(x, z0, Edge::Wall);
            lot.set_h(x, z0 + 8, Edge::Wall);
        }
        for z in z0..z0 + 8 {
            lot.set_v(x0, z, Edge::Wall);
            lot.set_v(x0 + 10, z, Edge::Wall);
        }
    }
    lot.compute_rooms();
    let mut reach = Reach::build(&nav(&lot, &blocked));
    let mut regions_seen = std::collections::BTreeSet::new();
    for edit in 0..500 {
        let (x, z) = (rng.below(SIZE - 2) + 1, rng.below(SIZE - 2) + 1);
        let rect = match rng.below(7) {
            // Something put down or taken away, up to 3 × 3.
            0 | 1 => {
                let (w, d) = (rng.below(3) + 1, rng.below(3) + 1);
                let on = rng.next() < 0.6;
                for tz in z..(z + d).min(SIZE) {
                    for tx in x..(x + w).min(SIZE) {
                        blocked[lot.tile_index(tx as i32, tz as i32)] = on;
                    }
                }
                [x, z, x + w, z + d]
            }
            // A run of wall, fence, door or gate along x or z.
            2 | 3 => {
                let kind = [
                    Edge::Wall,
                    Edge::Fence,
                    Edge::Door,
                    Edge::Open,
                    Edge::Window,
                ][rng.below(5)];
                let len = rng.below(20) + 1;
                if rng.next() < 0.5 {
                    let x1 = (x + len).min(SIZE);
                    (x..x1).for_each(|tx| lot.set_h(tx, z, kind));
                    [x, z - 1, x1, z + 1]
                } else {
                    let z1 = (z + len).min(SIZE);
                    (z..z1).for_each(|tz| lot.set_v(x, tz, kind));
                    [x - 1, z, x + 1, z1]
                }
            }
            // A small closed box, or one opened up again.
            4 => {
                let (w, d) = (rng.below(5) + 2, rng.below(5) + 2);
                let kind = if rng.next() < 0.6 {
                    Edge::Wall
                } else {
                    Edge::Open
                };
                let (x1, z1) = ((x + w).min(SIZE - 1), (z + d).min(SIZE - 1));
                for tx in x..x1 {
                    lot.set_h(tx, z, kind);
                    lot.set_h(tx, z1, kind);
                }
                for tz in z..z1 {
                    lot.set_v(x, tz, kind);
                    lot.set_v(x1, tz, kind);
                }
                [x - 1, z - 1, x1 + 1, z1 + 1]
            }
            // A diagonal wall, door or nothing across a tile.
            _ => {
                let d = match rng.below(4) {
                    0 => None,
                    k => Some(Diagonal {
                        dir: if k == 1 { DiagDir::Dp } else { DiagDir::Dn },
                        edge: if k == 3 { Edge::Door } else { Edge::Wall },
                    }),
                };
                lot.set_diag(x, z, d);
                [x, z, x + 1, z + 1]
            }
        };
        lot.compute_rooms();
        blocked.resize(lot.tile_slots(), false);
        let rect = rect.map(|v| v as i32);
        reach.mark(&lot, rect);
        reach.refresh(&nav(&lot, &blocked));
        let full = Reach::build(&nav(&lot, &blocked));
        let grid = nav(&lot, &blocked);
        assert_eq!(reach.regions(), full.regions(), "edit {edit}: region count");
        regions_seen.insert(full.regions());
        for _ in 0..200 {
            let a = (rng.below(SIZE) as i32, rng.below(SIZE) as i32);
            let b = (rng.below(SIZE) as i32, rng.below(SIZE) as i32);
            assert_eq!(
                reach.reaches(&grid, a, b),
                full.reaches(&grid, a, b),
                "edit {edit}: {a:?} to {b:?}"
            );
        }
        // Now and then, against a plain flood over every tile, as the walkers step.
        if edit % 25 == 0 {
            let label = flood(&grid);
            for _ in 0..400 {
                let a = (rng.below(SIZE) as i32, rng.below(SIZE) as i32);
                let b = (rng.below(SIZE) as i32, rng.below(SIZE) as i32);
                let (la, lb) = (
                    label[a.1 as usize * SIZE + a.0 as usize],
                    label[b.1 as usize * SIZE + b.0 as usize],
                );
                if la != u32::MAX && lb != u32::MAX {
                    assert_eq!(
                        reach.reaches(&grid, a, b),
                        la == lb,
                        "edit {edit}: {a:?} to {b:?} against a flood"
                    );
                }
            }
        }
        // Now and then, against a search: reachable exactly when there's a way.
        if edit % 50 == 0 {
            for _ in 0..4 {
                let a = (rng.below(SIZE) as i32, rng.below(SIZE) as i32);
                let b = (rng.below(SIZE) as i32, rng.below(SIZE) as i32);
                if grid.tile_free(a.0, a.1) && grid.tile_free(b.0, b.1) {
                    assert_eq!(
                        reach.reaches(&grid, a, b),
                        grid.find_path(a, b).is_some(),
                        "edit {edit}: {a:?} to {b:?} against a search"
                    );
                }
            }
        }
    }
    assert!(regions_seen.len() > 1, "the edits made a difference");
}
