//! Long walks in an open world go by road (`travel.rs`): a trip between two hamlets follows the
//! country road, and its legs to and from the road keep out of other people's gardens.

use crate::common;

use common::hamlets::{Hamlets, two_hamlets};
use sim_core::command::{EdgeAxis, EdgeEdit, EdgeKind};
use sim_core::world::Phase;
use sim_core::{Command, World};

fn world() -> (World, Hamlets) {
    let content = common::content();
    let h = two_hamlets(&content, 7);
    let mut w = World::from_json(&content, &h.town, 1).expect("the hamlets load");
    w.autonomy = false;
    for hh in 0..w.households.len() as u32 {
        w.apply(Command::SetAutonomy {
            enabled: false,
            household: Some(hh),
        })
        .unwrap();
    }
    (w, h)
}

fn waypoints(w: &World, i: usize) -> Vec<[f32; 2]> {
    match w.sims[i].current().map(|a| &a.phase) {
        Some(Phase::Routing { waypoints, .. }) => waypoints.clone(),
        other => panic!("{} isn't walking: {other:?}", w.sims[i].name),
    }
}

/// Whether `p` is on a plot other than `ok`'s and not public.
fn in_a_garden(w: &World, p: [f32; 2], ok: &[Option<u32>]) -> Option<u32> {
    let plot = w.plot_at(p[0].floor() as i32, p[1].floor() as i32)?;
    (!w.plots[plot as usize].public && !ok.contains(&Some(plot))).then_some(plot)
}

/// Distance from `p` to the line through `pts`.
fn off_line(p: [f32; 2], pts: &[[f64; 2]]) -> f32 {
    pts.windows(2)
        .map(|s| {
            sim_core::travel::line_distance(
                p,
                [s[0][0] as f32, s[0][1] as f32],
                [s[1][0] as f32, s[1][1] as f32],
            )
        })
        .fold(f32::INFINITY, f32::min)
}

#[test]
fn the_network_joins_the_hamlets() {
    let (w, _) = world();
    let g = &w.travel.graph;
    // Two streets and the country road between them: their four ends, the two where the country
    // road meets the streets shared.
    assert_eq!(g.roads.len(), 3);
    assert_eq!(g.nodes.len(), 4, "{:?}", g.nodes);
}

#[test]
fn a_trip_between_hamlets_follows_the_roads_and_keeps_out_of_gardens() {
    let (mut w, h) = world();
    let i = 0;
    let home = w.households[w.sims[i].household as usize].plot;
    // A house in the other hamlet: its front door.
    let goal_plot = h.plots[1]
        .clone()
        .find(|&p| !w.plots[p].public)
        .expect("a house") as u32;
    let target = w.plots[goal_plot as usize].arrival_point();
    w.apply(Command::MoveTo {
        sim: i as u32,
        x: target[0],
        z: target[1],
    })
    .unwrap();
    w.tick_once();
    let route = waypoints(&w, i);
    let from = w.sims[i].pos;
    let ok = [home, Some(goal_plot)];
    // Between the hamlets every waypoint is on the country road (on its verge).
    let (_, _, a_end) = h.streets[0];
    let (_, b_start, _) = h.streets[1];
    let between: Vec<_> = route
        .iter()
        .filter(|p| p[0] > a_end as f32 + 4.0 && p[0] < b_start as f32 - 4.0)
        .collect();
    assert!(
        between.len() >= 10,
        "few waypoints along the road: {route:?}"
    );
    for p in &between {
        assert!(
            off_line(**p, &h.country) <= 3.0,
            "{p:?} is off the country road"
        );
    }
    // Every step of the way: no one else's garden.
    let mut at = from;
    for &p in &route {
        let n = (sim_core::travel::dist(at, p) / 0.25).ceil().max(1.0) as usize;
        for k in 0..=n {
            let t = k as f32 / n as f32;
            let q = [at[0] + (p[0] - at[0]) * t, at[1] + (p[1] - at[1]) * t];
            assert!(
                in_a_garden(&w, q, &ok).is_none(),
                "through plot {:?}'s garden at {q:?}",
                in_a_garden(&w, q, &ok)
            );
        }
        at = p;
    }
    let len = sim_core::travel::length(from, &route);
    eprintln!(
        "{} waypoints, {len:.0} m from {from:?} to {target:?} ({:.0} min on foot)",
        route.len(),
        len / 1.5 * sim_core::GAME_MINUTES_PER_SECOND
    );
    // And they get there, walking it.
    let ticks = (len / 1.5 * 1.3 * common::TICKS_PER_SECOND as f32) as u64;
    for _ in 0..ticks {
        w.tick_once();
        if w.sims[i].current().is_none() {
            break;
        }
    }
    assert!(
        sim_core::travel::dist(w.sims[i].pos, target) < 1.5,
        "at {:?}, not {target:?}",
        w.sims[i].pos
    );
}

#[test]
fn short_trips_are_found_over_the_tiles_as_before() {
    let (mut w, _) = world();
    let i = 0;
    let p = w.sims[i].pos;
    let target = [p[0] + 20.0, p[1]];
    let goal = (target[0].floor() as i32, target[1].floor() as i32);
    w.apply(Command::MoveTo {
        sim: i as u32,
        x: target[0],
        z: target[1],
    })
    .unwrap();
    w.tick_once();
    let route = waypoints(&w, i);
    let tile = (p[0].floor() as i32, p[1].floor() as i32);
    let nav = sim_core::path::NavGrid {
        lot: &w.lot,
        blocked: w.blocked(),
        stairs: w.stair_map(),
        kept_off: &[],
    };
    // The same as A* would give (the way they set off may have been cut short by a step).
    let path = nav.find_path(tile, goal).expect("a way");
    let smooth = nav.smooth(&path);
    assert_eq!(route.last(), Some(&target));
    assert!(route.len() <= smooth.len() + 1, "{route:?} vs {smooth:?}");
}

/// The town records where walking changed (`World::nav_changed_since`, a build by the tiles it
/// was on): a walker whose way lies far from it keeps their way (`world::route_touched`), even
/// with the change inside the box round the rest of the trip, in a back garden it passes by.
#[test]
fn a_far_away_change_does_not_reroute() {
    let (mut w, h) = world();
    let i = 0;
    let goal_plot = h.plots[1].clone().find(|&p| !w.plots[p].public).unwrap() as u32;
    let target = w.plots[goal_plot as usize].arrival_point();
    w.apply(Command::MoveTo {
        sim: i as u32,
        x: target[0],
        z: target[1],
    })
    .unwrap();
    // Some way along it.
    for _ in 0..common::secs(20.0) {
        w.tick_once();
    }
    let next = |w: &World| match w.sims[i].current().map(|a| &a.phase) {
        Some(Phase::Routing { next, .. }) => *next,
        _ => 0,
    };
    let (before, at) = (waypoints(&w, i), next(&w));
    assert!(at > 0);
    let ahead = &before[at..];
    let pos = w.sims[i].pos;
    let lo = |k: usize| ahead.iter().map(|q| q[k]).fold(pos[k], f32::min);
    let hi = |k: usize| ahead.iter().map(|q| q[k]).fold(pos[k], f32::max);
    // A wall across the back garden of a house in their own hamlet that they're still to pass
    // (on the street's far side from it), and one in the other hamlet's last house.
    let behind_the_street = h.plots[0]
        .clone()
        .find(|&p| {
            let p = &w.plots[p];
            !p.public && p.z as f32 > h.streets[0].0 as f32 && (p.x as f32) > pos[0] + 10.0
        })
        .expect("a house on the street's south side ahead of them")
        as u32;
    let last = h.plots[1]
        .clone()
        .rev()
        .find(|&p| !w.plots[p].public)
        .unwrap() as u32;
    for plot in [behind_the_street, last] {
        let p = w.plots[plot as usize].clone();
        let (x, z) = (p.x + 2, p.z + p.d - 4);
        if plot == behind_the_street {
            let inside = (x as f32) > lo(0) && (x as f32) < hi(0);
            let inside = inside && (z as f32) > lo(1) && (z as f32) < hi(1);
            assert!(inside, "the wall at {x}, {z} is inside the trip's box");
        }
        let household = w
            .households
            .iter()
            .find(|hh| hh.plot == Some(plot))
            .unwrap()
            .id;
        w.apply(Command::Build {
            household,
            edits: (0..3)
                .map(|k| EdgeEdit::new(EdgeAxis::V, x, z + k, EdgeKind::Wall))
                .collect(),
        })
        .unwrap_or_else(|e| panic!("a wall on {}: {e}", p.name));
        w.tick_once();
        // Not routed again from where they are (that starts a new list of waypoints).
        assert_eq!(
            waypoints(&w, i),
            before,
            "re-routed for a wall on {}",
            p.name
        );
        assert!(next(&w) >= at, "re-routed for a wall on {}", p.name);
    }
}
