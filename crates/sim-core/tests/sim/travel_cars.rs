//! Cars on the roads of an open-world town (`cars::Route::on_roads`): out of the driveway onto
//! the road it meets, along the lanes to the way out, and back in nose first, never off the road.

use crate::common;

use common::hamlets::{two_hamlets, with_cars};
use sim_core::World;
use sim_core::cars::CarState;
use sim_core::travel::Mode;

fn world(seed: u64) -> World {
    let content = common::content();
    let h = two_hamlets(&content, seed);
    World::from_json(&content, &with_cars(&h), seed as u32).expect("the hamlets load")
}

#[test]
fn cars_keep_to_the_road_and_come_home_nose_in() {
    let mut w = world(21);
    assert!(!w.cars.is_empty());
    let (width, depth) = (w.lot.width as f32, w.lot.depth as f32);
    let mut drives = vec![0u32; w.cars.len()];
    let mut worst: f32 = 0.0;
    let mut moving_ticks = 0u64;
    for _ in 0..(24 * 60 * 20) {
        w.tick_once();
        for (c, car) in w.cars.iter().enumerate() {
            let Some((at, _, moving, _)) = car.shown else {
                continue;
            };
            if !moving {
                continue;
            }
            moving_ticks += 1;
            if car.state == CarState::Leaving && car.progress < 2.0 {
                drives[c] += 1;
            }
            // Off the map past the exit, or on its own driveway: nothing to check.
            let (x, z) = (at[0].floor() as i32, at[1].floor() as i32);
            if at[0] < 0.0 || at[1] < 0.0 || at[0] >= width || at[1] >= depth {
                continue;
            }
            if w.plot_at(x, z) == Some(car.plot) {
                continue;
            }
            assert_eq!(w.plot_at(x, z), None, "car {c} on someone's plot at {at:?}");
            let near = w.travel.graph.near(at, 20.0, 1, Mode::Car);
            let p = near
                .first()
                .unwrap_or_else(|| panic!("car {c} far off the road at {at:?}"));
            let half = w.travel.graph.roads[p.road as usize].width / 2.0;
            worst = worst.max(p.off);
            assert!(
                p.off <= half,
                "car {c} {:.2} m off the road's middle at {at:?} (half its width {half})",
                p.off
            );
        }
    }
    eprintln!(
        "{} drives out, {moving_ticks} car-ticks on the move, at most {worst:.2} m off the middle",
        drives.iter().filter(|&&d| d > 0).count()
    );
    assert!(
        drives.iter().filter(|&&d| d > 0).count() >= 2,
        "hardly anyone drove: {drives:?}"
    );
    // Back home: parked where they were, nose in (facing away from the road).
    for car in &w.cars {
        if car.state != CarState::Parked || car.blocked {
            continue;
        }
        let m = w.travel.graph.near(car.spot, 30.0, 1, Mode::Car)[0];
        let mp = w.travel.graph.point(m);
        let away = (car.spot[0] - mp[0]).atan2(car.spot[1] - mp[1]);
        let turned = sim_core::world::wrap_angle(car.yaw - away).abs();
        assert!(
            turned < 0.2 || turned > std::f32::consts::PI - 0.2,
            "parked across its driveway: yaw {} vs {away}",
            car.yaw
        );
    }
}

/// A car pulling out of its driveway or parking in it changes walking only round it
/// (`cars::restand`): walkers elsewhere keep their way, and the walkable regions are labelled
/// again only there, not over the box round every car in town.
#[test]
fn a_car_coming_or_going_changes_walking_only_round_it() {
    let mut w = world(21);
    assert!(w.cars.len() >= 2);
    let mut checked = 0;
    for _ in 0..(24 * 60 * 20) {
        let (seen, before) = (w.nav_version(), w.parked_tiles());
        let spots: Vec<[f32; 2]> = w.cars.iter().map(|c| c.spot).collect();
        w.tick_once();
        let after = w.parked_tiles();
        if w.nav_version() == seen || before == after {
            continue;
        }
        // Between the two cars furthest apart (inside the box round them all), well away from
        // every car: nothing changed there.
        let spots: Vec<[f32; 2]> = spots
            .into_iter()
            .chain(w.cars.iter().map(|c| c.spot))
            .collect();
        let far = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
        let (a, b) = spots
            .iter()
            .flat_map(|&a| spots.iter().map(move |&b| (a, b)))
            .max_by(|p, q| far(p.0, p.1).total_cmp(&far(q.0, q.1)))
            .unwrap();
        let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
        if spots.iter().any(|&s| far(s, mid) < 20.0) {
            continue;
        }
        let (x, z) = (mid[0].floor() as i32, mid[1].floor() as i32);
        assert!(
            !w.nav_changed_since(seen, [x - 2, z - 2, x + 3, z + 3]),
            "walking changed at {mid:?}, far from every car"
        );
        // Round the tiles that changed, it did.
        let (x, z) = before
            .iter()
            .chain(&after)
            .copied()
            .find(|t| before.contains(t) != after.contains(t))
            .unwrap();
        assert!(w.nav_changed_since(seen, [x, z, x + 1, z + 1]));
        checked += 1;
    }
    assert!(checked >= 2, "hardly any car came or went: {checked}");
}
