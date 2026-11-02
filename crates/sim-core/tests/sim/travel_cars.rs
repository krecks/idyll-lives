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
