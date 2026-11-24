use super::*;

fn road(points: &[[f32; 2]], width: f32, kind: RoadKind) -> RoadRaw {
    RoadRaw {
        points: points.to_vec(),
        width,
        kind,
        area: None,
        oneway: OneWay::Both,
    }
}

/// A line from `a` to `b` with a point every metre.
fn line(a: [f32; 2], b: [f32; 2]) -> Vec<[f32; 2]> {
    let n = dist(a, b).ceil() as usize;
    (0..=n).map(|i| lerp(a, b, i as f32 / n as f32)).collect()
}

/// `Graph::near` and `nearest` (cells nearest first, a ring at a time, stopping once
/// nothing further out counts) find what looking at every segment of every road finds.
#[test]
fn near_finds_what_every_segment_says() {
    // (A little generator of its own: the same roads every run.)
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 40) as f32 / (1u64 << 24) as f32
    };
    let kinds = [
        RoadKind::Street,
        RoadKind::Lane,
        RoadKind::Country,
        RoadKind::Trail,
    ];
    let mut roads = Vec::new();
    for k in 0..24 {
        // Winding roads of a few dozen points, 1 to 9 m apart.
        let mut at = [next() * 300.0, next() * 300.0];
        let mut heading = next() * std::f32::consts::TAU;
        let mut pts = vec![at];
        for _ in 0..(10 + (next() * 40.0) as usize) {
            heading += (next() - 0.5) * 0.8;
            let step = 1.0 + next() * 8.0;
            at = [at[0] + heading.cos() * step, at[1] + heading.sin() * step];
            pts.push(at);
        }
        roads.push(road(&pts, 4.0 + next() * 6.0, kinds[k % kinds.len()]));
    }
    // (A road too short for an edge: not in the network.)
    roads.push(road(
        &[[150.0, 150.0], [150.2, 150.1]],
        6.0,
        RoadKind::Street,
    ));
    let g = Graph::build(&roads);
    let every = |p: [f32; 2], reach: f32, n: usize, mode: Mode| {
        let mut best: Vec<RoadPoint> = Vec::new();
        for (r, road) in g.roads.iter().enumerate() {
            if mode == Mode::Car && !road.drivable() {
                continue;
            }
            for i in 0..road.pts.len() - 1 {
                let (s, d) = road.project_segment(i, p);
                if d > reach {
                    continue;
                }
                match best.iter_mut().find(|b| b.road == r as u32) {
                    Some(b) if d < b.off || (d == b.off && s < b.s) => {
                        b.s = s;
                        b.off = d;
                    }
                    Some(_) => {}
                    None => best.push(RoadPoint {
                        edge: 0,
                        road: r as u32,
                        s,
                        off: d,
                    }),
                }
            }
        }
        best.retain_mut(|b| g.edge_at(b.road, b.s).map(|e| b.edge = e).is_some());
        best.sort_by(|a, b| a.off.total_cmp(&b.off).then(a.road.cmp(&b.road)));
        best.truncate(n);
        best
    };
    let mut found = 0;
    for k in 0..3000 {
        // (Around the roads and past the cells' edges, on and off them.)
        let p = if k % 7 == 0 {
            let r = &g.roads[k % g.roads.len()];
            r.at(next() * r.len())
        } else {
            [next() * 420.0 - 60.0, next() * 420.0 - 60.0]
        };
        let reach = [2.0, 8.0, 30.0, 64.0, 150.0][k % 5];
        for mode in [Mode::Foot, Mode::Car] {
            for n in [1, 3, 6] {
                let want = every(p, reach, n, mode);
                assert_eq!(
                    g.near(p, reach, n, mode),
                    want,
                    "{p:?} {reach} {n} {mode:?}"
                );
                found += want.len();
            }
            assert_eq!(
                g.nearest(p, reach, mode),
                every(p, reach, 1, mode).first().copied(),
            );
        }
    }
    assert!(found > 1000, "few roads found: {found}");
}

#[test]
fn nearest_point_is_the_scans_ties_and_all() {
    // What `NearestPoint` gives is what a scan of every point gives (`min_by`: the first of
    // those as near): over a winding line of points (one going back over itself, points
    // repeated), on whole metres too (many as near), asked about near it, across its box and
    // far outside it.
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut rand = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 40) as f32 / (1u64 << 24) as f32
    };
    for whole in [false, true] {
        let mut pts: Vec<[f32; 2]> = Vec::new();
        let (mut at, mut heading) = ([1200.0f32, -300.0f32], 0.0f32);
        for _ in 0..1300 {
            heading += (rand() - 0.5) * 0.6;
            at = [at[0] + 2.5 * heading.cos(), at[1] + 2.5 * heading.sin()];
            pts.push(if whole { at.map(f32::round) } else { at });
        }
        let back: Vec<[f32; 2]> = pts[200..400].iter().rev().copied().collect();
        pts.extend(back);
        pts.extend_from_within(10..20);
        let grid = NearestPoint::new(pts.clone());
        assert!(!grid.scan, "cells over that many points");
        let scan = |p: [f32; 2]| {
            (0..pts.len()).min_by(|&a, &b| dist(pts[a], p).total_cmp(&dist(pts[b], p)))
        };
        let (lo, hi) = pts
            .iter()
            .fold(([f32::MAX; 2], [f32::MIN; 2]), |(lo, hi), p| {
                (
                    [lo[0].min(p[0]), lo[1].min(p[1])],
                    [hi[0].max(p[0]), hi[1].max(p[1])],
                )
            });
        let mut ties = 0;
        for i in (0..pts.len()).step_by(2) {
            let p = pts[i];
            let near = [
                p,
                [p[0] + (rand() - 0.5) * 9.0, p[1] + (rand() - 0.5) * 9.0],
                // (Half way between two points: as near to each, on whole metres.)
                lerp(p, pts[(i + 1) % pts.len()], 0.5),
                [p[0].round() + 0.5, p[1].round()],
            ];
            let across = [
                lo[0] + rand() * (hi[0] - lo[0]),
                lo[1] + rand() * (hi[1] - lo[1]),
            ];
            let far = [
                lo[0] + (rand() * 3.0 - 1.0) * (hi[0] - lo[0]) - 50.0,
                hi[1] + rand() * 4000.0,
            ];
            for q in near.into_iter().chain([across, far]) {
                let want = scan(q);
                assert_eq!(grid.nearest(q), want, "nearest {q:?}");
                let d = dist(pts[want.expect("points")], q);
                if pts[want.expect("points") + 1..]
                    .iter()
                    .any(|&p| dist(p, q) == d)
                {
                    ties += 1;
                }
            }
        }
        assert!(ties > 500, "ties asked about: {ties}");
    }
    // (Few points, none, and one not finite: scanned.)
    assert_eq!(NearestPoint::new(Vec::new()).nearest([0.0, 0.0]), None);
    let few = vec![[0.0, 0.0], [2.0, 0.0], [1.0, 5.0]];
    assert_eq!(NearestPoint::new(few).nearest([1.0, 0.0]), Some(0));
}

#[test]
fn two_side_roads_meeting_a_road_a_metre_apart_are_one_crossroads() {
    // A 7 m street along x; a lane off it north at x = 50 and another south at x = 51.1, each
    // beginning at the street's kerb: two T's 1.1 m apart were two junctions, each with its
    // lights in the middle of the other. One crossroads, its node half way between on the
    // street's middle line, four arms, two phases; the street's stretches either side of it
    // meet there, with nothing left out of the street.
    let g = Graph::build(&[
        road(&line([0.0, 10.0], [100.0, 10.0]), 7.0, RoadKind::Street),
        road(&line([50.0, 12.15], [50.0, 60.0]), 6.0, RoadKind::Lane),
        road(&line([51.1, 7.85], [51.1, -40.0]), 6.0, RoadKind::Lane),
        // (Farther apart, 12 m, two T's as before.)
        road(&line([80.0, 12.15], [80.0, 60.0]), 6.0, RoadKind::Lane),
        road(&line([92.0, 7.85], [92.0, -40.0]), 6.0, RoadKind::Lane),
    ]);
    let junctions: Vec<usize> = (0..g.nodes.len()).filter(|&n| g.degree(n) >= 3).collect();
    assert_eq!(junctions.len(), 3, "{:?}", g.nodes);
    let cross = junctions
        .iter()
        .copied()
        .find(|&n| g.degree(n) == 4)
        .expect("a crossroads");
    assert!(
        dist(g.nodes[cross], [50.55, 10.0]) < 0.01,
        "{:?}",
        g.nodes[cross]
    );
    let sig = crate::traffic::signal_at(&g, cross, &[]).expect("a junction");
    assert_eq!((sig.arms.len(), sig.phases), (4, 2));
    assert!(sig.lit, "a crossroads is lit");
    // (The two T's further on, a lane meeting the street: giving way.)
    for &n in junctions.iter().filter(|&&n| n != cross) {
        let t = crate::traffic::signal_at(&g, n, &[]).expect("a junction");
        assert!(!t.lit, "a lane's T gives way");
    }
    // (The street's stretches, end to end along it: none missing, none overlapping.)
    let street: Vec<&Edge> = g.road_edges[0]
        .iter()
        .map(|&e| &g.edges[e as usize])
        .collect();
    assert_eq!(street.first().unwrap().s0, 0.0);
    assert!((street.last().unwrap().s1 - 100.0).abs() < 1e-3);
    for w in street.windows(2) {
        assert_eq!(w[0].s1, w[1].s0);
        assert_eq!(w[0].b, w[1].a);
    }
    for (k, e) in g.edges.iter().enumerate() {
        assert!(g.adj[e.a as usize].contains(&(k as u32)));
        assert!(g.adj[e.b as usize].contains(&(k as u32)));
    }
    // (From one lane to the other across the street, through the crossroads.)
    let (a, b) = (
        g.near([50.0, 40.0], 5.0, 1, Mode::Car)[0],
        g.near([51.1, -20.0], 5.0, 1, Mode::Car)[0],
    );
    let path = g
        .path(a, b, Mode::Car, COUNTRY_SPEED)
        .expect("a way across");
    assert!(path.len < 62.0, "{}", path.len);
}

#[test]
fn a_road_too_short_for_an_edge_is_left_out_not_a_panic() {
    // (Half a metre or less end to end, as an old save or a hand-built town may have: on its
    // own off the street, and as a stub from it.)
    let g = Graph::build(&[
        road(&line([0.0, 10.0], [100.0, 10.0]), 6.0, RoadKind::Street),
        road(&[[50.0, 40.0], [50.3, 40.0]], 6.0, RoadKind::Lane),
        road(&[[20.0, 10.0], [20.0, 10.4]], 6.0, RoadKind::Lane),
    ]);
    for (p, mode) in [
        ([50.1, 40.0], Mode::Car),
        ([50.1, 40.0], Mode::Foot),
        ([20.0, 10.2], Mode::Car),
    ] {
        for q in g.near(p, 40.0, 3, mode) {
            assert_eq!(g.roads[q.road as usize].source, 0, "{q:?}");
        }
    }
    assert!(g.at_source(1, 0.1).is_none());
    assert!(g.at_source(2, 0.2).is_none());
    assert!(g.at_source(0, 50.0).is_some());
}

#[test]
fn ends_and_joins_become_nodes() {
    // A street along x, a lane from its middle going +z (ending 2 m off its middle line,
    // within reach of its edge), and a trail on from the lane's far end.
    let g = Graph::build(&[
        road(&line([0.0, 10.0], [100.0, 10.0]), 6.0, RoadKind::Street),
        road(&line([50.0, 12.0], [50.0, 60.0]), 5.0, RoadKind::Lane),
        road(&line([50.0, 60.0], [90.0, 90.0]), 2.0, RoadKind::Trail),
    ]);
    // Street ends 2, the T (the lane's near end), the lane's far end shared with the trail,
    // the trail's far end.
    assert_eq!(g.nodes.len(), 5, "{:?}", g.nodes);
    // The street in two at the T, the lane, the trail.
    assert_eq!(g.edges.len(), 4);
    let degree: Vec<usize> = g.adj.iter().map(Vec::len).collect();
    assert_eq!(degree.iter().filter(|&&d| d == 3).count(), 1, "{degree:?}");
    // (The T on the street's middle line, where the lane meets it, not at the lane's end 2 m
    // off it: its box and lights in the middle of the road.)
    let t = degree.iter().position(|&d| d == 3).unwrap();
    assert!(dist(g.nodes[t], [50.0, 10.0]) < 1e-3, "{:?}", g.nodes[t]);
    assert!(
        g.boxes
            .iter()
            .any(|&(c, r)| dist(c, [50.0, 10.0]) < 1e-3 && r >= 6.0),
        "{:?}",
        g.boxes
    );
    assert_eq!(g.networks(), 1);
    // A lane ending 5 m short of the street is a network of its own.
    let apart = Graph::build(&[
        road(&line([0.0, 10.0], [100.0, 10.0]), 6.0, RoadKind::Street),
        road(&line([50.0, 18.0], [50.0, 60.0]), 6.0, RoadKind::Lane),
    ]);
    assert_eq!(apart.networks(), 2);
    assert_eq!(Graph::default().networks(), 0);
}

#[test]
fn roads_crossing_or_touching_away_from_their_ends_meet_there() {
    // A street along x; a country road across it, and another bending down to touch it (its
    // middle line 0.3 m from the street's) and away again; a trail across it.
    let mut touch = line([60.0, -30.0], [75.0, 9.7]);
    touch.extend(line([75.0, 9.7], [90.0, -30.0]).into_iter().skip(1));
    let g = Graph::build(&[
        road(&line([0.0, 10.0], [100.0, 10.0]), 7.0, RoadKind::Street),
        road(&line([30.0, -30.0], [30.0, 50.0]), 7.0, RoadKind::Country),
        road(&touch, 7.0, RoadKind::Country),
        road(&line([50.0, -30.0], [50.0, 50.0]), 2.0, RoadKind::Trail),
    ]);
    let junctions: Vec<[f32; 2]> = g.junctions().collect();
    assert_eq!(junctions.len(), 2, "{junctions:?}");
    assert!(junctions.iter().any(|&p| dist(p, [30.0, 10.0]) < 0.5));
    assert!(junctions.iter().any(|&p| dist(p, [75.0, 9.85]) < 0.5));
    // A box at each, for the cars to take turns there; the trail crosses as ever.
    assert!(g.boxes.iter().any(|&(c, _)| dist(c, [30.0, 10.0]) < 0.5));
    assert!(g.boxes.iter().any(|&(c, _)| dist(c, [75.0, 9.85]) < 0.5));
    assert_eq!(g.networks(), 2, "the trail apart");
}

#[test]
fn a_path_goes_round_by_the_roads_and_cars_keep_off_trails() {
    let g = Graph::build(&[
        road(&line([0.0, 10.0], [100.0, 10.0]), 6.0, RoadKind::Street),
        road(&line([100.0, 10.0], [100.0, 100.0]), 6.0, RoadKind::Country),
        road(&line([0.0, 10.0], [0.0, 100.0]), 2.0, RoadKind::Trail),
        road(&line([0.0, 100.0], [100.0, 100.0]), 6.0, RoadKind::Street),
    ]);
    let from = g.near([2.0, 60.0], 10.0, 1, Mode::Foot)[0];
    let to = g.near([50.0, 100.0], 10.0, 1, Mode::Foot)[0];
    let walk = g.path(from, to, Mode::Foot, COUNTRY_SPEED).expect("a way");
    // Down the trail and along the far street: 40 + 50.
    assert!((walk.len - 90.0).abs() < 1.0, "{}", walk.len);
    // By car: no trail, so round by the near street and the country road.
    let from = g.near([20.0, 12.0], 10.0, 1, Mode::Car)[0];
    let drive = g.path(from, to, Mode::Car, COUNTRY_SPEED).expect("a drive");
    assert!(
        (drive.len - (80.0 + 90.0 + 50.0)).abs() < 1.0,
        "{}",
        drive.len
    );
    assert!(g.near([1.0, 60.0], 3.0, 1, Mode::Car).is_empty());
}

/// `path_cost` is `path`'s length and cost, to the bit, from and to anywhere on the roads
/// (a search after another reuses the last one's state, `RoadScratch`).
#[test]
fn a_paths_cost_alone_is_the_paths() {
    let g = Graph::build(&[
        road(&line([0.0, 10.0], [100.0, 10.0]), 6.0, RoadKind::Street),
        road(&line([100.0, 10.0], [100.0, 100.0]), 6.0, RoadKind::Country),
        road(&line([0.0, 10.0], [0.0, 100.0]), 2.0, RoadKind::Trail),
        road(&line([0.0, 100.0], [100.0, 100.0]), 6.0, RoadKind::Street),
        road(&line([50.0, 10.0], [50.0, 100.0]), 4.0, RoadKind::Lane),
    ]);
    let points: Vec<[f32; 2]> = (0..12)
        .flat_map(|i| (0..12).map(move |j| [i as f32 * 9.0 + 1.0, j as f32 * 9.0 + 1.0]))
        .collect();
    let mut compared = 0;
    for mode in [Mode::Foot, Mode::Car] {
        for &a in &points {
            for &b in points.iter().step_by(7) {
                let (Some(&from), Some(&to)) = (
                    g.near(a, 30.0, 1, mode).first(),
                    g.near(b, 30.0, 1, mode).first(),
                ) else {
                    continue;
                };
                let path = g.path(from, to, mode, COUNTRY_SPEED);
                let cost = g.path_cost(from, to, mode, COUNTRY_SPEED);
                assert_eq!(
                    path.map(|p| (p.len.to_bits(), p.cost.to_bits())),
                    cost.map(|(l, c)| (l.to_bits(), c.to_bits()))
                );
                compared += 1;
            }
        }
    }
    assert!(compared > 100);
}

#[test]
fn thinning_keeps_bends() {
    let mut pts = line([0.0, 0.0], [20.0, 0.0]);
    pts.extend(line([20.0, 0.0], [20.0, 20.0]).into_iter().skip(1));
    let t = thin(&pts);
    assert!(t.contains(&[20.0, 0.0]), "{t:?}");
    assert!(t.len() <= 10, "{t:?}");
    assert!(t.windows(2).all(|w| dist(w[0], w[1]) <= THIN_STEP + 1e-3));
}

#[test]
fn rle_masks() {
    let t = Terrain {
        water: Some(vec![3, 0, 2, 1, 10, 0]),
        blocked: Some(vec![14, 0, 1, 1]),
    };
    let m = t.mask(5, 3);
    let set: Vec<usize> = (0..15).filter(|&i| m[i]).collect();
    assert_eq!(set, [3, 4, 14]);
}

#[test]
fn each_kind_has_its_top_speed() {
    let country = 17.0;
    let speeds = [
        (RoadKind::Lane, LANE_SPEED),
        (RoadKind::Street, TOWN_SPEED),
        (RoadKind::Avenue, AVENUE_SPEED),
        (RoadKind::Ramp, RAMP_SPEED),
        (RoadKind::Country, country),
        (RoadKind::Motorway, MOTORWAY_SPEED),
    ];
    for w in speeds.windows(2) {
        assert!(w[0].1 < w[1].1, "{:?} not slower than {:?}", w[0].0, w[1].0);
    }
    for (kind, top) in speeds {
        assert_eq!(kind_speed(kind, country), top, "{kind:?}");
    }
}
