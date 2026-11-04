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

// ---- Cars keeping their distance -----------------------------------------------------------

/// The widest and longest of the renderer's cars (m).
const CAR: (f32, f32) = (4.6, 1.88);

/// A cosy town 200 m long with a street `width` m wide along x (its middle at z = 30, a way out at
/// either end) and a house at each of `houses` (its driveway's x, and whether it's north of the
/// street), with its driveway onto the street and a car on it, and its driver at home.
fn street(width: f32, houses: &[(f32, bool)]) -> World {
    let content = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],
        "objects":[],
        "economy":{"startingFunds":500}}"#;
    let road: Vec<String> = (0..=199).map(|x| format!("[{x}.5,30]")).collect();
    let (mut plots, mut households, mut sims, mut drives, mut cars) =
        (vec![], vec![], vec![], vec![], vec![]);
    for (k, &(x, north)) in houses.iter().enumerate() {
        let (z0, inner, front, yaw) = if north {
            (8.0, 20.5, 1.0, 0.0)
        } else {
            (34.0, 39.5, -1.0, std::f32::consts::PI)
        };
        let spot = inner + front * (0.45 + CAR.0 / 2.0);
        let door = if north { z0 + 17.5 } else { z0 + 0.5 };
        plots.push(format!(
            r#"{{"name":"H{k}","x":{},"z":{z0},"w":16,"d":18,"entry":[{},{door}]}}"#,
            x - 12.0,
            x - 6.5
        ));
        households.push(format!(r#"{{"name":"F{k}","plot":{k}}}"#));
        sims.push(format!(
            r#"{{"name":"D{k}","household":{k},"x":{},"z":{}}}"#,
            x - 6.5,
            z0 + 9.5
        ));
        drives.push(format!(
            r#"{{"plot":{k},"x":{x},"z":{inner},"front":{front},"street":30,"road":0,"s":{}}}"#,
            x - 0.5
        ));
        cars.push(format!(
            r#"{{"plot":{k},"x":{x},"z":{spot},"yaw":{yaw},"street":30,"len":{},"wid":{},"body":2,"paint":{k}}}"#,
            CAR.0, CAR.1
        ));
    }
    let town = format!(
        r#"{{"width":200,"depth":60,
        "exits":[{{"x":0.5,"z":30,"kind":"road"}},{{"x":199.5,"z":30,"kind":"road"}}],
        "roads":[{{"points":[{}],"width":{width},"kind":"street","ends":["out","out"]}}],
        "plots":[{}],"households":[{}],"objects":[],"sims":[{}],"drives":[{}],"cars":[{}]}}"#,
        road.join(","),
        plots.join(","),
        households.join(","),
        sims.join(","),
        drives.join(","),
        cars.join(",")
    );
    let mut w = World::from_json(content, &town, 1).unwrap();
    w.autonomy = false;
    assert!(w.cars.iter().all(|c| !c.no_way), "every car has a way out");
    w
}

/// Car `c` sets off now, its driver in (at work as far as anyone can tell).
fn set_off(w: &mut World, c: usize) {
    let car = &mut w.cars[c];
    car.state = CarState::Leaving;
    car.since = w.tick;
    car.progress = 0.0;
    car.driver = Some(c as u32);
    car.aboard = vec![c as u32];
    w.sims[c].away_until = Some(w.tick + 100_000);
}

/// Car `c`, away with its driver done at work, comes home now.
fn come_home(w: &mut World, c: usize) {
    let car = &mut w.cars[c];
    car.state = CarState::Away;
    car.since = w.tick;
    car.driver = Some(c as u32);
    car.aboard = vec![c as u32];
    car.done = vec![c as u32];
    car.ready_at = w.tick;
    w.sims[c].away_until = Some(w.tick + 100_000);
}

/// How far two of `w`'s cars' bodies overlap at most where they're drawn, one of them on the move
/// (m; 0 or less: apart), and which.
fn cars_overlap(w: &World) -> (f32, usize, usize) {
    let corners = |p: [f32; 2], yaw: f32, len: f32, wid: f32| {
        let (s, c) = yaw.sin_cos();
        let (f, r) = (
            [s * len / 2.0, c * len / 2.0],
            [c * wid / 2.0, -s * wid / 2.0],
        );
        [
            [p[0] + f[0] + r[0], p[1] + f[1] + r[1]],
            [p[0] + f[0] - r[0], p[1] + f[1] - r[1]],
            [p[0] - f[0] - r[0], p[1] - f[1] - r[1]],
            [p[0] - f[0] + r[0], p[1] - f[1] + r[1]],
        ]
    };
    // (Separating axes: the least overlap along any side's normal.)
    let depth = |a: &[[f32; 2]; 4], b: &[[f32; 2]; 4]| {
        let mut best = f32::INFINITY;
        for poly in [a, b] {
            for k in 0..2 {
                let (p, q) = (poly[k], poly[k + 1]);
                let l = (q[1] - p[1]).hypot(p[0] - q[0]);
                let n = [(q[1] - p[1]) / l, (p[0] - q[0]) / l];
                let span = |r: &[[f32; 2]; 4]| {
                    r.iter()
                        .map(|v| v[0] * n[0] + v[1] * n[1])
                        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), x| {
                            (lo.min(x), hi.max(x))
                        })
                };
                let ((a0, a1), (b0, b1)) = (span(a), span(b));
                best = best.min(a1.min(b1) - a0.max(b0));
            }
        }
        best
    };
    let shown: Vec<_> = w
        .cars
        .iter()
        .enumerate()
        .filter_map(|(k, c)| c.shown.map(|s| (k, s.2, corners(s.0, s.1, c.len, c.wid))))
        .collect();
    let mut worst = (f32::NEG_INFINITY, 0, 0);
    for (i, a) in shown.iter().enumerate() {
        for b in &shown[i + 1..] {
            if a.1 || b.1 {
                let d = depth(&a.2, &b.2);
                if d > worst.0 {
                    worst = (d, a.0, b.0);
                }
            }
        }
    }
    worst
}

#[test]
fn two_cars_coming_in_by_the_same_way_at_the_same_tick_come_in_one_after_the_other() {
    // (Both homes nearer the west end: both come in by it, the first to come in (of two there as
    // long, the first car) on past the other's home.)
    let mut w = street(7.0, &[(90.0, true), (60.0, true)]);
    come_home(&mut w, 0);
    come_home(&mut w, 1);
    let mut first_seen = [None::<u64>; 2];
    for _ in 0..4000 {
        w.tick_once();
        let (o, a, b) = cars_overlap(&w);
        assert!(o <= 0.0, "cars {a} and {b} overlap by {o:.2} m");
        for (seen, car) in first_seen.iter_mut().zip(&w.cars) {
            if car.shown.is_some_and(|s| s.2) && seen.is_none() {
                *seen = Some(w.tick);
            }
        }
        if w.cars.iter().all(|c| c.state == CarState::Parked) {
            break;
        }
    }
    assert!(
        w.cars.iter().all(|c| c.state == CarState::Parked),
        "both home"
    );
    let (a, b) = (first_seen[0].unwrap(), first_seen[1].unwrap());
    // (The second waits out of sight till the first is a car's length and the gap in front on.)
    assert!(
        a.abs_diff(b) >= 10,
        "they came in {} ticks apart",
        a.abs_diff(b)
    );
}

#[test]
fn two_cars_meeting_on_a_7_m_street_pass_each_other_in_their_lanes() {
    // One drives off west from its house; the other comes in from the west to its home.
    let mut w = street(7.0, &[(90.0, true), (60.0, false)]);
    set_off(&mut w, 0);
    come_home(&mut w, 1);
    let mut met = false;
    let mut stood = vec![0u64; 2];
    let mut was = [None::<[f32; 2]>; 2];
    for _ in 0..4000 {
        w.tick_once();
        let (o, a, b) = cars_overlap(&w);
        assert!(o <= 0.0, "cars {a} and {b} overlap by {o:.2} m");
        let at: Vec<_> = (0..2)
            .map(|c| w.cars[c].shown.filter(|s| s.2).map(|s| s.0))
            .collect();
        if let (Some(p), Some(q)) = (at[0], at[1]) {
            met |= (p[0] - q[0]).abs() < 3.0;
        }
        for c in 0..2 {
            if at[c].is_some() && at[c] == was[c] {
                stood[c] += 1;
            }
            was[c] = at[c];
        }
        if w.cars[0].state == CarState::Away && w.cars[1].state == CarState::Parked {
            break;
        }
    }
    assert_eq!(w.cars[0].state, CarState::Away, "the one leaving got away");
    assert_eq!(w.cars[1].state, CarState::Parked, "the other got home");
    assert!(met, "they passed each other on the street");
    // (Where they meet, neither stops for the other: the street has room for both.)
    assert!(stood.iter().all(|&s| s < 20), "they stood {stood:?} ticks");
}
