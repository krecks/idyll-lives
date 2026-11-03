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

/// A house on a street along x (its middle at z = 9.5), the ways out at both ends, the car on the
/// driveway facing the street, and a box either side of the driveway's mouth: Ada drives, Bo has
/// no job.
fn driveway() -> World {
    let content = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],
        "objects":[{"id":"box","name":"Box","footprint":[1,1],"interactions":[]}],
        "careers":[{"id":"office","label":"Office","levels":[
            {"title":"Intern","pay":100,"start":9,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
        "economy":{"startingFunds":500}}"#;
    let town = r#"{"width":60,"depth":14,"exits":[[0.5,9.5],[59.5,9.5]],
        "plots":[{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}],
        "households":[{"name":"Ones","plot":0}],
        "objects":[{"def":"box","x":33,"z":6,"rot":0},{"def":"box","x":35,"z":6,"rot":0}],
        "sims":[{"name":"Ada","household":0,"job":{"career":"office"},"x":25.5,"z":3.5},
                {"name":"Bo","household":0,"x":26.5,"z":3.5}],
        "cars":[{"plot":0,"x":34.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":2}]}"#;
    let mut w = World::from_json(content, town, 1).unwrap();
    w.autonomy = false;
    w
}

/// How far someone on foot at `pos` (a quarter of a metre round) is from car `c`'s body where
/// it's drawn: below 0, how far into it. None: not drawn.
fn from_car(w: &World, c: usize, pos: [f32; 2]) -> Option<f32> {
    let (at, yaw, ..) = w.cars[c].shown?;
    let (s, k) = yaw.sin_cos();
    let (dx, dz) = (pos[0] - at[0], pos[1] - at[1]);
    let (along, across) = (dx * s + dz * k, dx * k - dz * s);
    let (ex, ez) = (
        along.abs() - w.cars[c].len / 2.0,
        across.abs() - w.cars[c].wid / 2.0,
    );
    Some(ex.max(0.0).hypot(ez.max(0.0)) + ex.max(ez).min(0.0) - 0.25)
}

#[test]
fn a_car_never_drives_through_someone_with_nowhere_to_step_aside() {
    let mut w = driveway();
    // Ada drives off (at work as far as anyone can tell)...
    let car = &mut w.cars[0];
    car.state = CarState::Leaving;
    car.progress = 0.0;
    car.driver = Some(0);
    car.aboard = vec![0];
    w.sims[0].away_until = Some(w.tick + 100_000);
    // ...and Bo stands in front of it between the boxes, with no room to step aside.
    w.sims[1].pos = [34.0, 6.5];
    let mut closest = f32::INFINITY;
    for _ in 0..2000 {
        w.tick_once();
        if let Some(gap) = from_car(&w, 0, w.sims[1].pos) {
            closest = closest.min(gap);
        }
    }
    assert_eq!(w.cars[0].state, CarState::Away, "the car never got away");
    assert!(
        closest >= 0.0,
        "it drove {:.2} m into Bo, now at {:?}",
        -closest,
        w.sims[1].pos
    );
}
