use super::*;
use crate::lot::LotFile;

fn lot_with_wall() -> Lot {
    // Wall along x = 5 from z = 0 to z = 8, door-less: must go around at z >= 8.
    let file: LotFile =
        serde_json::from_str(r#"{"width":10,"depth":10,"walls":[[5,0,5,8]]}"#).unwrap();
    Lot::from_file(&file).unwrap()
}

/// A 10×10 lot with a closed 4×4 room in its middle.
fn lot_with_sealed_room(door: bool) -> Lot {
    let bottom = if door {
        r#"[1,5,3,5],[4,5,5,5]"#
    } else {
        r#"[1,5,5,5]"#
    };
    let json =
        format!(r#"{{"width":10,"depth":10,"walls":[[1,1,5,1],{bottom},[1,1,1,5],[5,1,5,5]]}}"#);
    let file: LotFile = serde_json::from_str(&json).unwrap();
    Lot::from_file(&file).unwrap()
}

#[test]
fn reach_labels_separate_regions() {
    let lot = lot_with_sealed_room(false);
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    let reach = Reach::build(&nav);
    assert_eq!(reach.regions(), 2, "the room is sealed off");
    assert!(reach.reaches(&nav, (2, 2), (3, 3)));
    assert!(!reach.reaches(&nav, (2, 2), (8, 8)));
    assert!(!reach.reaches(&nav, (8, 8), (2, 2)));
    // Standing on a tile something now blocks: a free neighbour decides.
    let mut blocked = blocked;
    blocked[lot.tile_index(2, 2)] = true;
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    assert!(reach.reaches(&nav, (2, 2), (3, 3)));
    assert!(!reach.reaches(&nav, (2, 2), (8, 8)));
}

#[test]
fn a_door_joins_the_regions() {
    let lot = lot_with_sealed_room(true);
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    let reach = Reach::build(&nav);
    assert_eq!(reach.regions(), 1);
    assert!(reach.reaches(&nav, (2, 2), (8, 8)));
    assert!(nav.find_path((2, 2), (8, 8)).is_some());
}

#[test]
fn routes_around_wall() {
    let lot = lot_with_wall();
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    let path = nav.find_path((2, 2), (7, 2)).expect("path exists");
    assert!(
        path.iter().any(|&(_, z)| z >= 8),
        "must go around the wall end"
    );
    for w in path.windows(2) {
        assert!(nav.can_step(w[0].0, w[0].1, w[1].0, w[1].1));
    }
}

#[test]
fn smoothing_keeps_walkability() {
    let lot = lot_with_wall();
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    let path = nav.find_path((2, 2), (7, 2)).unwrap();
    let smooth = nav.smooth(&path);
    assert!(smooth.len() < path.len());
    for w in smooth.windows(2) {
        assert!(nav.line_walkable(w[0], w[1]));
    }
}

fn nav_of(lot: &Lot) -> Vec<bool> {
    vec![false; lot.tile_slots()]
}

/// Every step of a path is allowed and no tile on it is crossed by a diagonal wall.
fn assert_walkable(nav: &NavGrid, path: &[(i32, i32)]) {
    for &(x, z) in path {
        assert!(
            !nav.lot.diag_blocks(x, z),
            "path enters diagonal wall tile {x},{z}"
        );
    }
    for w in path.windows(2) {
        assert!(nav.can_step(w[0].0, w[0].1, w[1].0, w[1].1), "{w:?}");
    }
}

/// A diagonal wall `/` from (2, 0) to (8, 6) across tiles (2,0)..(7,5), then straight up
/// x = 8 to z = 10: the lower-right side is closed off except around the far end.
fn lot_with_diagonal(door: bool) -> Lot {
    let mut diags: Vec<String> = (0..6)
        .map(|k| format!(r#"{{"x":{},"z":{},"dir":"dp"}}"#, 2 + k, k))
        .collect();
    if door {
        diags[3] = r#"{"x":5,"z":3,"dir":"dp","kind":"door"}"#.into();
    }
    let json = format!(
        r#"{{"width":12,"depth":12,"walls":[[8,6,8,11],[2,0,0,0]],"diagonals":[{}]}}"#,
        diags.join(",")
    );
    let file: LotFile = serde_json::from_str(&json).unwrap();
    Lot::from_file(&file).unwrap()
}

#[test]
fn diagonal_wall_tiles_block_and_steps_cannot_cross_it() {
    let lot = lot_with_diagonal(false);
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    assert!(!nav.tile_free(4, 2), "nobody stands on a diagonal wall");
    // Orthogonal steps into the wall tile are refused from both halves.
    assert!(!nav.can_step(4, 1, 4, 2));
    assert!(!nav.can_step(3, 2, 4, 2));
    // Diagonal steps across the joint of two diagonal tiles are refused both ways.
    assert!(
        !nav.can_step(5, 2, 4, 3),
        "crosses the wall at corner (5, 3)"
    );
    assert!(
        !nav.can_step(6, 3, 5, 4),
        "crosses the wall at corner (6, 4)"
    );
    assert!(!nav.can_step(5, 4, 6, 3));
    // Beside the wall (parallel to it) is fine.
    assert!(nav.can_step(5, 2, 6, 3));
    // Past a free end (the wall stops at (8, 6) and the h wall at x = 0..2, z = 0).
    assert!(
        !nav.can_step(7, 6, 8, 5),
        "would clip the wall's end at (8, 6)"
    );
}

#[test]
fn routes_around_a_diagonal_wall() {
    let lot = lot_with_diagonal(false);
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    // From the lower-right side (below the wall) to the upper-left side.
    let path = nav.find_path((6, 1), (2, 4)).expect("path around the end");
    assert_walkable(&nav, &path);
    assert!(
        path.iter().any(|&(x, z)| x >= 8 && z >= 11) || path.iter().any(|&(x, _)| x >= 9),
        "must go around via x > 8: {path:?}"
    );
    let smooth = nav.smooth(&path);
    for w in smooth.windows(2) {
        assert!(nav.line_walkable(w[0], w[1]));
    }
}

#[test]
fn diagonal_door_lets_residents_through() {
    let lot = lot_with_diagonal(true);
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    assert!(nav.tile_free(5, 3), "a doorway can be walked");
    let path = nav.find_path((6, 2), (4, 4)).expect("through the door");
    assert!(path.contains(&(5, 3)), "{path:?}");
    assert!(path.len() <= 5, "short way through the door: {path:?}");
    assert_walkable(&nav, &path);
    // Not along the wall through the door's ends.
    assert!(!nav.can_step(4, 2, 5, 3));
    assert!(!nav.can_step(5, 3, 6, 4));
}

#[test]
fn corridor_between_parallel_diagonals_is_walkable() {
    // Two `/` walls one tile apart: (0,0)..(3,3) and (2,0)..(5,3) (x - z = 0 and 2).
    let diags: Vec<String> = (0..3)
        .flat_map(|k| {
            [
                format!(r#"{{"x":{k},"z":{k},"dir":"dp"}}"#),
                format!(r#"{{"x":{},"z":{k},"dir":"dp"}}"#, k + 2),
            ]
        })
        .collect();
    let json = format!(
        r#"{{"width":8,"depth":8,"diagonals":[{}]}}"#,
        diags.join(",")
    );
    let lot = Lot::from_file(&serde_json::from_str::<LotFile>(&json).unwrap()).unwrap();
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    assert!(nav.can_step(1, 0, 2, 1), "along the corridor");
    assert!(nav.can_step(2, 1, 3, 2));
    assert!(!nav.can_step(1, 0, 0, 1), "across the left wall");
}

#[test]
fn no_path_into_closed_box() {
    let file: LotFile = serde_json::from_str(
        r#"{"width":6,"depth":6,"walls":[[2,2,4,2],[2,4,4,4],[2,2,2,4],[4,2,4,4]]}"#,
    )
    .unwrap();
    let lot = Lot::from_file(&file).unwrap();
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    assert!(nav.find_path((0, 0), (3, 3)).is_none());
}

/// Three gardens side by side, x 0..10, 10..20 and 20..30 (plots 0, 1, 2) over rows
/// 0..10, with the street along rows 10..14. No fences: only the plots tell them apart.
fn three_gardens(walls: &str) -> (Lot, Vec<u16>) {
    let json = format!(r#"{{"width":30,"depth":14,"walls":[{walls}]}}"#);
    let lot = Lot::from_file(&serde_json::from_str::<LotFile>(&json).unwrap()).unwrap();
    let tile_plot = (0..30 * 14)
        .map(|i| {
            if i / 30 < 10 {
                (i % 30 / 10) as u16
            } else {
                NO_PLOT
            }
        })
        .collect();
    (lot, tile_plot)
}

fn in_middle_garden((x, z): (i32, i32)) -> bool {
    (10..20).contains(&x) && z < 10
}

#[test]
fn residents_go_round_by_the_street_not_through_the_neighbours_garden() {
    let (lot, tile_plot) = three_gardens("");
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    // Without plots to mind, straight across the middle garden.
    let direct = nav.find_path((5, 8), (25, 8)).unwrap();
    assert!(direct.iter().any(|&t| in_middle_garden(t)), "{direct:?}");
    // From the front of the first to the front of the third: the middle one isn't theirs.
    let grounds = Grounds {
        tile_plot: &tile_plot,
        open: vec![true, false, true],
        fenced: vec![true; 3],
        halls: &Halls::NONE,
        hall_open: Vec::new(),
    };
    let path = nav.find_path_with((5, 8), (25, 8), Some(&grounds)).unwrap();
    assert_walkable(&nav, &path);
    assert!(
        !path.iter().any(|&t| in_middle_garden(t)),
        "through the neighbours' garden: {path:?}"
    );
    assert!(
        path.iter().any(|&(_, z)| z >= 10),
        "by the street: {path:?}"
    );
    // The smoothed line doesn't cut back across it either.
    let smooth = nav.smooth_with(&path, Some(&grounds));
    for w in smooth.windows(2) {
        assert!(nav.line_walkable_with(w[0], w[1], Some(&grounds)), "{w:?}");
        for k in 0..=100 {
            let t = k as f32 / 100.0;
            let p = [
                w[0][0] + (w[1][0] - w[0][0]) * t,
                w[0][1] + (w[1][1] - w[0][1]) * t,
            ];
            assert!(
                !in_middle_garden((p[0].floor() as i32, p[1].floor() as i32)),
                "smoothed across at {p:?}"
            );
        }
    }
    assert!(!nav.line_walkable_with([5.5, 8.5], [25.5, 8.5], Some(&grounds)));
    assert!(
        nav.line_walkable([5.5, 8.5], [25.5, 8.5]),
        "only the plots stand in the way"
    );
}

#[test]
fn into_the_garden_next_door_by_the_street_not_over_the_hedge() {
    let (lot, tile_plot) = three_gardens("");
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    // From their own garden to the neighbours' (both open to them: they're going there), the
    // two side by side: out at the front, along the street and in at the neighbours' front.
    let grounds = Grounds {
        tile_plot: &tile_plot,
        open: vec![true, true, false],
        fenced: vec![true; 3],
        halls: &Halls::NONE,
        hall_open: Vec::new(),
    };
    let path = nav.find_path_with((8, 3), (12, 3), Some(&grounds)).unwrap();
    assert_walkable(&nav, &path);
    assert!(
        path.windows(2)
            .all(|w| (w[0].0 < 10) == (w[1].0 < 10) || w[0].1 >= 10),
        "over the boundary between the gardens: {path:?}"
    );
    assert!(
        path.iter().any(|&(_, z)| z >= 10),
        "by the street: {path:?}"
    );
    let smooth = nav.smooth_with(&path, Some(&grounds));
    for w in smooth.windows(2) {
        assert!(nav.line_walkable_with(w[0], w[1], Some(&grounds)), "{w:?}");
    }
    assert!(
        !nav.line_walkable_with([8.5, 3.5], [12.5, 3.5], Some(&grounds)),
        "no straight line over it"
    );
    // Between plots nobody fences (public places side by side), straight across.
    let grounds = Grounds {
        tile_plot: &tile_plot,
        open: vec![true; 3],
        fenced: vec![false; 3],
        halls: &Halls::NONE,
        hall_open: Vec::new(),
    };
    assert_eq!(
        nav.find_path_with((8, 3), (12, 3), Some(&grounds))
            .unwrap()
            .len(),
        5
    );
    assert!(nav.line_walkable_with([8.5, 3.5], [12.5, 3.5], Some(&grounds)));
}

#[test]
fn a_garden_is_crossed_when_there_is_no_other_way() {
    // The street is closed off in front of the middle garden.
    let (lot, tile_plot) = three_gardens("[10,10,10,14],[20,10,20,14]");
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    let grounds = Grounds {
        tile_plot: &tile_plot,
        open: vec![true, false, true],
        fenced: vec![true; 3],
        halls: &Halls::NONE,
        hall_open: Vec::new(),
    };
    let path = nav
        .find_path_with((5, 12), (25, 12), Some(&grounds))
        .expect("through the garden");
    assert_walkable(&nav, &path);
    assert!(path.iter().any(|&t| in_middle_garden(t)));
    // Someone standing in a garden they shouldn't be in can still get out of it, the
    // shortest way (not across the gardens either side).
    let grounds = Grounds {
        tile_plot: &tile_plot,
        open: vec![false; 3],
        fenced: vec![true; 3],
        halls: &Halls::NONE,
        hall_open: Vec::new(),
    };
    let out = nav
        .find_path_with((15, 2), (15, 12), Some(&grounds))
        .expect("out to the street");
    assert_eq!(out.len(), 11, "straight down: {out:?}");
    assert_eq!(
        nav.smooth_with(&out, Some(&grounds)).len(),
        2,
        "in one straight line"
    );
}

#[test]
fn a_long_way_round_is_found_through_a_corridor() {
    // A wall down the middle of a big open lot with one gap at the far end: straight at the
    // goal, a search would try most of the near half before turning back.
    let mut lot = Lot::new(400, 400);
    for z in 0..396 {
        lot.set_v(200, z, crate::lot::Edge::Wall);
    }
    lot.compute_rooms();
    let blocked = nav_of(&lot);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    take_search_stats();
    let before = nodes_searched();
    let path = nav.find_path((100, 10), (300, 10)).expect("round the end");
    assert_walkable(&nav, &path);
    assert!(path.iter().any(|&(_, z)| z >= 396), "through the gap");
    let stats = take_search_stats();
    assert_eq!(
        nodes_searched() - before,
        stats.nodes,
        "the budget counter follows"
    );
    assert!(
        stats.searches >= 2,
        "capped, then kept to a corridor: {stats:?}"
    );
    assert!(stats.max_nodes <= CORRIDOR_CAP + 1, "{stats:?}");
    // As long as the full search's, near enough (the corridor holds the best way here).
    let shortest = 2 * 386 + 200;
    assert!(path.len() <= shortest + 40, "{} tiles", path.len());
}

#[test]
fn stairs_are_found_by_tile() {
    let mut map = StairMap {
        depth: 10,
        ..Default::default()
    };
    let flight = |x| Stair::new(&[(x, 5), (x, 4), (x, 3)], (x, 6), false, (x, 12));
    map.set_stairs(vec![flight(2), flight(5)]);
    assert_eq!(
        map.step(5, 4).map(|(s, i)| (s.bottom(), i)),
        Some(((5, 5), 1))
    );
    assert_eq!(map.step(5, 2), None);
    assert_eq!(map.link(2, 3), Some((2, 12)));
    assert_eq!(map.link(2, 12), Some((2, 3)));
    assert_eq!(map.link(3, 3), None);
    // Every tile as a scan of the flights finds it, near them or not (`near_stairs`).
    let far = Stair::new(&[(70, 41), (71, 41)], (69, 41), false, (72, 51));
    map.set_stairs(vec![flight(2), flight(5), far]);
    let scan = StairMap {
        stairs: map.stairs.clone(),
        ..Default::default()
    };
    for z in -2..60 {
        for x in -2..90 {
            assert_eq!(map.step(x, z), scan.step(x, z), "({x}, {z})");
            assert_eq!(map.link(x, z), scan.link(x, z), "({x}, {z})");
        }
    }
}

#[test]
fn a_spiral_is_walked_round_and_onto_from_any_side() {
    let mut lot = lot_with_wall();
    lot.grow_depth(20);
    let blocked = nav_of(&lot);
    // A spiral in the middle of the ground storey: front left, front right, back right,
    // back left, leading to the landing one tile left of it on the storey above.
    let steps = [(3, 5), (4, 5), (4, 4), (3, 4)];
    let mut map = StairMap {
        depth: 10,
        ..Default::default()
    };
    map.set_stairs(vec![Stair::new(&steps, (3, 5), true, (2, 14))]);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &map,
        kept_off: &[],
        steps: None,
    };
    // Onto the bottom step from any side, and round in order.
    assert!(nav.can_step(2, 5, 3, 5));
    assert!(nav.can_step(3, 6, 3, 5));
    assert!(nav.can_step(3, 5, 4, 5));
    assert!(nav.can_step(4, 5, 4, 4));
    assert!(nav.can_step(4, 4, 3, 4));
    // Never across the winding: not from the bottom step to the top one, nor onto a step
    // out of turn.
    assert!(!nav.can_step(3, 5, 3, 4));
    assert!(!nav.can_step(4, 5, 3, 4));
    // Down again the way it was climbed, and off the bottom step any way.
    assert!(nav.can_step(4, 5, 3, 5));
    assert!(nav.can_step(3, 4, 4, 4));
    // Off the bottom step any way, back down to the ground.
    assert!(nav.can_step(3, 5, 2, 5));
    assert!(nav.can_step(3, 5, 3, 6));
    // The top step leads to the landing on the storey above, and back down.
    assert_eq!(map.link(3, 4), Some((2, 14)));
    assert_eq!(map.link(2, 14), Some((3, 4)));
    // A flight is only entered at its foot.
    map.set_stairs(vec![Stair::new(
        &[(3, 5), (3, 4), (3, 3)],
        (3, 6),
        false,
        (3, 12),
    )]);
    let nav = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &map,
        kept_off: &[],
        steps: None,
    };
    assert!(nav.can_step(3, 6, 3, 5));
    assert!(!nav.can_step(2, 5, 3, 5));
    assert!(!nav.can_step(3, 5, 2, 5));
}

#[test]
fn one_at_a_way_out_on_the_grounds_far_side_steps_back_onto_it() {
    // A lot a chunk deep grown by a storey whose rows nobody has built on (their chunks not
    // there): someone at the ground's far edge (`z` its depth, as a way out of town there
    // is) is on the storey above's first row, and their search reads the steps the regions
    // worked out. Their step back onto the ground is open, as `can_step` has it.
    let file: LotFile = serde_json::from_str(r#"{"width":16,"depth":16}"#).unwrap();
    let mut lot = Lot::from_file(&file).unwrap();
    lot.grow_depth(32);
    assert!(!lot.allocated(4, 16), "the storey above isn't there");
    let blocked = nav_of(&lot);
    let stairs = StairMap {
        depth: 16,
        ..Default::default()
    };
    let plain = NavGrid {
        lot: &lot,
        blocked: &blocked,
        stairs: &stairs,
        kept_off: &[],
        steps: None,
    };
    let reach = Reach::build(&plain);
    let nav = NavGrid {
        steps: Some(&reach),
        ..plain
    };
    assert!(nav.can_step(4, 16, 4, 15));
    assert_eq!(
        nav.find_path((4, 16), (4, 8)),
        plain.find_path((4, 16), (4, 8))
    );
    assert!(nav.find_path((4, 16), (4, 8)).is_some());
}

#[test]
fn search_buffers_follow_the_lot_size() {
    // The same thread routes on a small lot, a lot grown by two storeys, and the small one
    // again: the kept buffers are resized each time and earlier searches don't leak in.
    let small = lot_with_wall();
    let mut tall = lot_with_wall();
    tall.grow_depth(30);
    // (The new rows are built on: their chunks are there.)
    tall.ensure(0, 10, 10, 30);
    let (bs, bt) = (nav_of(&small), nav_of(&tall));
    let flat = NavGrid {
        lot: &small,
        blocked: &bs,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    let up = NavGrid {
        lot: &tall,
        blocked: &bt,
        stairs: &NO_STAIRS,
        kept_off: &[],
        steps: None,
    };
    let first = flat.find_path((2, 2), (7, 2)).unwrap();
    assert!(
        up.find_path((2, 22), (7, 22)).is_some(),
        "rows on the third storey are searched"
    );
    assert_eq!(flat.find_path((2, 2), (7, 2)).unwrap(), first);
}

#[test]
fn a_ticks_changes_to_walking_share_a_place_in_the_log() {
    let mut log = NavLog::default();
    let far = [500, 500, 501, 501];
    // Tick 1: more patches than a place keeps, all round (0, 0) to (40, 0), then one at
    // (100, 0).
    for v in 1..=40u32 {
        let x = v as i32;
        log.push(1, v, [x, 0, x + 1, 1]);
    }
    log.push(1, 41, [100, 0, 101, 1]);
    assert!(log.changed_since(0, 41, [100, 0, 101, 1]));
    assert!(!log.changed_since(0, 41, far));
    // Looked after the first 40: only the newest is news.
    assert!(!log.changed_since(40, 41, [0, 0, 41, 1]));
    assert!(log.changed_since(40, 41, [100, 0, 101, 1]));
    // More ticks of changes than the log keeps: anything may have changed since tick 1.
    for t in 0..NAV_LOG as u32 {
        log.push(2 + u64::from(t), 42 + t, [0, 0, 1, 1]);
    }
    let now = 41 + NAV_LOG as u32;
    assert!(!log.changed_since(41, now, far), "every tick since is kept");
    assert!(log.changed_since(40, now, far), "tick 1 isn't");
    // A change nobody noted where (the version bumped without the log): anything.
    assert!(log.changed_since(now, now + 1, far));
    assert!(!log.changed_since(now, now, far), "nothing since");
}
