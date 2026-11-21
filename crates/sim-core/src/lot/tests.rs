use super::*;

fn boxed_lot() -> Lot {
    let file: LotFile = serde_json::from_str(
        r#"{"width":10,"depth":10,
            "walls":[[2,2,8,2],[2,8,8,8],[2,2,2,8],[8,2,8,8],[5,2,5,8]],
            "doors":[{"x":3,"z":2,"axis":"x"},{"x":5,"z":4,"axis":"z"}],
            "windows":[{"x":6,"z":2,"axis":"x"},{"x":5,"z":6,"axis":"z"}]}"#,
    )
    .unwrap();
    Lot::from_file(&file).unwrap()
}

#[test]
fn detects_two_rooms() {
    let lot = boxed_lot();
    assert_eq!(lot.room_at(0, 0), OUTDOORS);
    let a = lot.room_at(3, 3);
    let b = lot.room_at(6, 3);
    assert_ne!(a, OUTDOORS);
    assert_ne!(b, OUTDOORS);
    assert_ne!(a, b);
}

#[test]
fn walls_block_and_doors_pass() {
    let lot = boxed_lot();
    assert!(!lot.edge_walkable(4, 1, 4, 2));
    assert!(lot.edge_walkable(3, 1, 3, 2));
    assert!(lot.edge_walkable(4, 4, 5, 4));
}

#[test]
fn windows_block_walking_and_separate_rooms() {
    let lot = boxed_lot();
    assert_eq!(lot.h_edge(6, 2), Edge::Window);
    assert_eq!(lot.v_edge(5, 6), Edge::Window);
    // Outside to inside through the window, and between the rooms through the inner one.
    assert!(!lot.edge_walkable(6, 1, 6, 2));
    assert!(!lot.edge_walkable(4, 6, 5, 6));
    // The rooms stay closed: a window doesn't join them or let the outdoors in.
    assert_ne!(lot.room_at(4, 6), lot.room_at(5, 6));
    assert_ne!(lot.room_at(6, 2), OUTDOORS);
}

#[test]
fn diagonals_split_tiles_into_rooms_by_half() {
    // A diamond with corners (1,3), (3,1), (5,3), (3,5): eight diagonal tiles around four
    // whole tiles. One diagonal has a door, which separates rooms like a wall.
    let file: LotFile = serde_json::from_str(
        r#"{"width":8,"depth":8,"diagonals":[
            {"x":1,"z":2,"dir":"dn"},{"x":2,"z":1,"dir":"dn"},
            {"x":3,"z":1,"dir":"dp"},{"x":4,"z":2,"dir":"dp"},
            {"x":4,"z":3,"dir":"dn","kind":"door"},{"x":3,"z":4,"dir":"dn"},
            {"x":2,"z":4,"dir":"dp"},{"x":1,"z":3,"dir":"dp"}]}"#,
    )
    .unwrap();
    let lot = Lot::from_file(&file).unwrap();
    let inside = lot.room_at(2, 2);
    assert_ne!(inside, OUTDOORS);
    for (x, z) in [(2, 2), (3, 2), (2, 3), (3, 3)] {
        assert_eq!(lot.room_at(x, z), inside, "whole tile {x},{z}");
        assert_eq!(lot.half_room(x, z, 0), lot.half_room(x, z, 1));
    }
    // Each diagonal tile: the half towards the centre is inside, the other outside.
    let centre_half = |x: i32, z: i32| {
        let d = lot.diag(x, z).unwrap();
        // Signed side of the tile centre's offset towards the diamond centre (3, 3).
        let (cx, cz) = (3.0 - (x as f32 + 0.5), 3.0 - (z as f32 + 0.5));
        match d.dir {
            DiagDir::Dp => usize::from(cz > cx),
            DiagDir::Dn => usize::from(cx + cz > 0.0),
        }
    };
    for d in &file.diagonals {
        let (x, z) = (d.x as i32, d.z as i32);
        let h = centre_half(x, z);
        assert_eq!(lot.half_room(x, z, h), inside, "inner half of {x},{z}");
        assert_eq!(
            lot.half_room(x, z, 1 - h),
            OUTDOORS,
            "outer half of {x},{z}"
        );
        assert_eq!(
            lot.room_at(x, z),
            inside,
            "split tiles report the indoor half"
        );
    }
    // Side lookups match the halves.
    assert_eq!(lot.half_at_side(1, 2, Side::NegX), 0);
    assert_eq!(lot.half_at_side(1, 2, Side::PosX), 1);
    assert_eq!(lot.half_at_side(3, 1, Side::PosX), 0);
    assert_eq!(lot.half_at_side(3, 1, Side::NegX), 1);
    assert!(lot.diag_blocks(1, 2));
    assert!(
        !lot.diag_blocks(4, 3),
        "a diagonal door can be walked through"
    );
    assert!(lot.diag_touches(3, 1, 3, 1) && lot.diag_touches(3, 1, 4, 2));
    assert!(!lot.diag_touches(3, 1, 4, 1));
}

#[test]
fn half_open_diagonal_rooms_stay_outdoors() {
    // Three sides of the diamond only: the area leaks out and stays outdoors.
    let file: LotFile = serde_json::from_str(
        r#"{"width":8,"depth":8,"diagonals":[
            {"x":1,"z":2,"dir":"dn"},{"x":2,"z":1,"dir":"dn"},
            {"x":3,"z":1,"dir":"dp"},{"x":4,"z":2,"dir":"dp"},
            {"x":4,"z":3,"dir":"dn"},{"x":3,"z":4,"dir":"dn"}]}"#,
    )
    .unwrap();
    let lot = Lot::from_file(&file).unwrap();
    assert!(lot.half_rooms().iter().all(|h| *h == [OUTDOORS; 2]));
    assert!(lot.rooms().iter().all(|&r| r == OUTDOORS));
}

/// The same walls on a lot whose rows past `ground` are chunks allocated as they're built
/// on, and on one with every chunk there: the same rooms, numbered the same.
fn both_ways(width: usize, depth: usize, ground: usize, walls: &[[usize; 4]]) -> (Lot, Lot) {
    let mut lazy = Lot::with_ground(width, depth, ground);
    let mut dense = Lot::new(width, depth);
    for lot in [&mut lazy, &mut dense] {
        for &[x0, z0, x1, z1] in walls {
            if z0 == z1 {
                (x0..x1).for_each(|x| lot.set_h(x, z0, Edge::Wall));
            } else {
                (z0..z1).for_each(|z| lot.set_v(x0, z, Edge::Wall));
            }
        }
        lot.compute_rooms();
    }
    (lazy, dense)
}

#[test]
fn rooms_over_chunks_not_yet_there_are_found_and_numbered_as_if_they_were() {
    // Upstairs (rows 40..), a hall 40 tiles square: whole chunks inside it have no walls,
    // so they aren't allocated until the rooms are found. A small room beside it starts on
    // a later row than the hall's first tile.
    let walls = [
        [4, 44, 44, 44],
        [4, 84, 44, 84],
        [4, 44, 4, 84],
        [44, 44, 44, 84],
        [50, 46, 54, 46],
        [50, 50, 54, 50],
        [50, 46, 50, 50],
        [54, 46, 54, 50],
    ];
    let (lazy, dense) = both_ways(64, 100, 40, &walls);
    assert_eq!(lazy, dense);
    assert!(lazy.tile_slots() < dense.tile_slots(), "less is stored");
    let hall = lazy.room_at(20, 60);
    assert_ne!(hall, OUTDOORS, "the middle of the hall is a room");
    assert!(lazy.allocated(20, 60), "and is there now");
    assert_eq!(hall, dense.room_at(20, 60));
    assert_eq!(lazy.room_at(52, 48), dense.room_at(52, 48));
    assert_ne!(lazy.room_at(52, 48), hall);
    assert_eq!(lazy.rooms_row_major(), dense.rooms_row_major());
    assert!(!lazy.allocated(60, 90), "open air stays unallocated");
    assert_eq!(lazy.room_at(60, 90), OUTDOORS);
}

#[test]
fn slots_map_back_to_their_tiles() {
    let (lot, _) = both_ways(40, 60, 20, &[[3, 30, 9, 30]]);
    for (x, z, s) in lot.tiles() {
        assert_eq!(lot.tile_index(x, z), s);
        assert_eq!(lot.tile_xz(s), (x, z));
        assert_ne!(s, VOID);
    }
    assert_eq!(lot.tile_index(30, 50), VOID, "nothing built there");
    assert_eq!(lot.tile_index(-1, 3), VOID);
    let rows: Vec<i32> = lot.tiles().map(|(_, z, _)| z).collect();
    assert!(rows.windows(2).all(|w| w[0] <= w[1]), "row by row");
    let walls: Vec<_> = lot.walls().collect();
    assert_eq!(walls.len(), 6);
    assert!(
        walls
            .iter()
            .all(|(e, w)| matches!(e, EdgeRef::H(_, 30)) && *w == Edge::Wall)
    );
    let (h, v, _) = lot.row_major_edges();
    let again = Lot::from_edges(40, 60, 20, h, v, Vec::new()).unwrap();
    assert_eq!(again, lot);
}

#[test]
fn windows_need_a_wall() {
    let file: LotFile = serde_json::from_str(
        r#"{"width":6,"depth":6,"walls":[[1,1,4,1]],"windows":[{"x":4,"z":1,"axis":"x"}]}"#,
    )
    .unwrap();
    let err = Lot::from_file(&file).unwrap_err();
    assert!(err.to_string().contains("window"), "{err}");
}
