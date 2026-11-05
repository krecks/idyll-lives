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

// ---- Taking turns where ways cross (`traffic.rs`) --------------------------------------------

/// A house in `roads_town`: its plot (x, z, w, d), and its driveway's inner end (x, z), the way it
/// runs to its road (+1/-1 along z, or along x: `along_x`), the road it meets (`roads` index) and
/// how far along it.
#[derive(Clone, Copy)]
struct House {
    plot: [f32; 4],
    inner: [f32; 2],
    front: f32,
    along_x: bool,
    road: u32,
    s: f32,
}

/// A cosy town 200 by 60 m with `roads` (JSON, `kind` street) and `exits` (JSON) and a car at each
/// of `houses`, its driver at home.
fn roads_town(roads: &str, exits: &str, houses: &[House]) -> World {
    let content = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],
        "objects":[],
        "economy":{"startingFunds":500}}"#;
    town_of(content, "", 200, 60, roads, exits, houses)
}

/// A town `width` by `depth` m with `content`, `more` (JSON fields, each with a comma before it),
/// `roads` and `exits` (JSON) and a car at each of `houses`, its driver at home.
fn town_of(
    content: &str,
    more: &str,
    width: u32,
    depth: u32,
    roads: &str,
    exits: &str,
    houses: &[House],
) -> World {
    let (mut plots, mut households, mut sims, mut drives, mut cars) =
        (vec![], vec![], vec![], vec![], vec![]);
    for (k, h) in houses.iter().enumerate() {
        let [x, z, pw, pd] = h.plot;
        let (cx, cz) = (x + pw / 2.0, z + pd / 2.0);
        plots.push(format!(
            r#"{{"name":"H{k}","x":{x},"z":{z},"w":{pw},"d":{pd},"entry":[{},{}]}}"#,
            cx.floor() + 0.5,
            cz.floor() + 0.5
        ));
        households.push(format!(r#"{{"name":"F{k}","plot":{k}}}"#));
        sims.push(format!(
            r#"{{"name":"D{k}","household":{k},"x":{},"z":{}}}"#,
            cx.floor() + 0.5,
            cz.floor() + 0.5
        ));
        let k_park = 0.45 + CAR.0 / 2.0;
        let (spot, yaw, front) = if h.along_x {
            (
                [h.inner[0] + h.front * k_park, h.inner[1]],
                h.front * std::f32::consts::FRAC_PI_2,
                format!(r#""front":0,"frontX":{}"#, h.front),
            )
        } else {
            (
                [h.inner[0], h.inner[1] + h.front * k_park],
                if h.front > 0.0 {
                    0.0
                } else {
                    std::f32::consts::PI
                },
                format!(r#""front":{}"#, h.front),
            )
        };
        drives.push(format!(
            r#"{{"plot":{k},"x":{},"z":{},{front},"street":30,"road":{},"s":{}}}"#,
            h.inner[0], h.inner[1], h.road, h.s
        ));
        cars.push(format!(
            r#"{{"plot":{k},"x":{},"z":{},"yaw":{yaw},"street":30,"len":{},"wid":{},"body":2,"paint":{k}}}"#,
            spot[0], spot[1], CAR.0, CAR.1
        ));
    }
    let town = format!(
        r#"{{"width":{width},"depth":{depth},"exits":{exits},"roads":{roads}{more},
        "plots":[{}],"households":[{}],"objects":[],"sims":[{}],"drives":[{}],"cars":[{}]}}"#,
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

/// Points every metre from `a` to `b` (JSON).
fn line_pts(a: [f32; 2], b: [f32; 2]) -> String {
    let n = (a[0] - b[0]).hypot(a[1] - b[1]).round() as usize;
    (0..=n)
        .map(|k| {
            let t = k as f32 / n as f32;
            format!(
                "[{},{}]",
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Ticks `w` on (at most `ticks`) till `done`, every tick checking no two cars overlap (one of
/// them on the move); false if it never was.
fn run_clear(
    w: &mut World,
    ticks: u32,
    mut each: impl FnMut(&mut World),
    done: impl Fn(&World) -> bool,
) -> bool {
    for _ in 0..ticks {
        w.tick_once();
        let (o, a, b) = cars_overlap(w);
        assert!(o <= 0.05, "cars {a} and {b} overlap by {o:.2} m");
        each(w);
        if done(w) {
            return true;
        }
    }
    false
}

#[test]
fn two_cars_meeting_on_a_6_m_lane_pass_each_other_pulled_over() {
    // One drives off west from its house; the other comes in from the west to its home.
    let mut w = street(6.0, &[(90.0, true), (60.0, false)]);
    set_off(&mut w, 0);
    come_home(&mut w, 1);
    let mut pulled: f32 = 0.0;
    let done = run_clear(
        &mut w,
        6000,
        |w| pulled = pulled.max(w.cars[0].shift.min(w.cars[1].shift)),
        |w| w.cars[0].state == CarState::Away && w.cars[1].state == CarState::Parked,
    );
    assert!(
        done,
        "both on their way: {:?}",
        w.cars
            .iter()
            .map(|c| (c.state, c.progress))
            .collect::<Vec<_>>()
    );
    // (Both onto their pavements, clear of the kerbside furniture: `cars::kerbside`.)
    assert!(pulled > 0.2, "they pulled over {pulled:.2} m to pass");
}

#[test]
fn a_car_pulling_out_of_its_driveway_waits_for_one_coming_along() {
    // One comes in from the east end to its home; the other, on the same side further east,
    // pulls out of its driveway as it comes by, to drive off east across its lane.
    let mut w = street(7.0, &[(110.0, true), (150.0, true)]);
    come_home(&mut w, 0);
    let mut out = false;
    let done = run_clear(
        &mut w,
        4000,
        |w| {
            if !out && w.cars[0].shown.is_some_and(|s| s.2 && s.0[0] < 170.0) {
                set_off(w, 1);
                out = true;
            }
        },
        |w| w.cars[0].state == CarState::Parked && w.cars[1].state == CarState::Away,
    );
    assert!(
        done,
        "not both on their way: {:?}",
        w.cars
            .iter()
            .map(|c| (c.state, c.progress))
            .collect::<Vec<_>>()
    );
}

/// A main road along x (z = 30) and a side road along z (x = 120) from a dead end at z = 52 to it;
/// the ways out at either end of the main road.
fn t_junction(houses: &[House]) -> World {
    let roads = format!(
        r#"[{{"points":[{}],"width":7,"kind":"street","ends":["out","out"]}},
            {{"points":[{}],"width":6,"kind":"street","ends":["dead","join"]}}]"#,
        line_pts([0.5, 30.0], [199.5, 30.0]),
        line_pts([120.0, 52.0], [120.0, 30.0])
    );
    roads_town(
        &roads,
        r#"[{"x":0.5,"z":30,"kind":"road"},{"x":199.5,"z":30,"kind":"road"}]"#,
        houses,
    )
}

#[test]
fn a_car_turning_out_of_a_side_road_waits_for_one_coming_along_the_road() {
    // On the side road, west of it, a house whose car turns out onto the main road east (the
    // nearer way out), across the way of one coming home along it from the east to a house just
    // west of the junction.
    let side = House {
        plot: [102.0, 36.0, 16.0, 14.0],
        inner: [110.0, 43.0],
        front: 1.0,
        along_x: true,
        road: 1,
        s: 9.0,
    };
    let main = House {
        plot: [98.0, 8.0, 16.0, 18.0],
        inner: [106.0, 20.5],
        front: 1.0,
        along_x: false,
        road: 0,
        s: 105.5,
    };
    for trigger in [175.0, 169.0, 165.0, 150.0] {
        let mut w = t_junction(&[side, main]);
        come_home(&mut w, 1);
        let mut out = false;
        let done = run_clear(
            &mut w,
            6000,
            |w| {
                if !out && w.cars[1].shown.is_some_and(|s| s.2 && s.0[0] < trigger) {
                    set_off(w, 0);
                    out = true;
                }
            },
            |w| w.cars[0].state == CarState::Away && w.cars[1].state == CarState::Parked,
        );
        assert!(
            done,
            "({trigger}) not both on their way: {:?}",
            w.cars
                .iter()
                .map(|c| (c.state, c.progress))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn cars_from_every_arm_of_a_crossroads_at_once_all_get_through() {
    // A crossroads: the main road along x (dead ends either way), the cross road along z with the
    // ways out at its ends. Two cars set off from houses on the main road's arms, two come home
    // to houses on them from the cross road's: all four through the junction at once.
    let roads = format!(
        r#"[{{"points":[{}],"width":7,"kind":"street","ends":["dead","dead"]}},
            {{"points":[{}],"width":7,"kind":"street","ends":["out","join"]}},
            {{"points":[{}],"width":7,"kind":"street","ends":["join","out"]}}]"#,
        line_pts([40.0, 30.0], [160.0, 30.0]),
        line_pts([100.0, 0.5], [100.0, 30.0]),
        line_pts([100.0, 30.0], [100.0, 59.5])
    );
    let house = |x: f32, north: bool| {
        let (z, inner, front) = if north {
            (8.0, 20.5, 1.0)
        } else {
            (34.0, 39.5, -1.0)
        };
        House {
            plot: [x - 12.0, z, 16.0, 18.0],
            inner: [x, inner],
            front,
            along_x: false,
            road: 0,
            s: x - 40.0,
        }
    };
    let houses = [
        house(70.0, true),
        house(130.0, false),
        house(80.0, false),
        house(120.0, true),
    ];
    let mut w = roads_town(
        &roads,
        r#"[{"x":100,"z":0.5,"kind":"road"},{"x":100,"z":59.5,"kind":"road"}]"#,
        &houses,
    );
    set_off(&mut w, 0);
    set_off(&mut w, 1);
    come_home(&mut w, 2);
    come_home(&mut w, 3);
    let mut ticks = 0;
    let done = run_clear(
        &mut w,
        3 * 60 * 20,
        |_| ticks += 1,
        |w| {
            matches!(w.cars[0].state, CarState::Away)
                && matches!(w.cars[1].state, CarState::Away)
                && w.cars[2].state == CarState::Parked
                && w.cars[3].state == CarState::Parked
        },
    );
    assert!(
        done,
        "not all through in three minutes: {:?}",
        w.cars
            .iter()
            .map(|c| (c.state, c.progress))
            .collect::<Vec<_>>()
    );
    // (Through the junction one after another: none of them over a minute of game time waiting.)
    assert!(
        ticks > 200,
        "through in {ticks} ticks: did they drive at all?"
    );
}

// ---- Cars and buses taking turns (`traffic.rs`) ------------------------------------------------

/// A bus every 20 minutes from 6 to 23 o'clock.
const BUS_CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],
    "objects":[],
    "bus":{"every":20,"first":6,"last":23},
    "economy":{"startingFunds":500}}"#;

/// How far two bodies overlap (m; 0 or less: apart), each where it is facing its yaw, its length
/// and width: the least along any of their sides' ways.
fn bodies_overlap(a: ([f32; 2], f32, f32, f32), b: ([f32; 2], f32, f32, f32)) -> f32 {
    let axes = |yaw: f32| {
        let (s, c) = yaw.sin_cos();
        [[s, c], [c, -s]]
    };
    let (aa, ba) = (axes(a.1), axes(b.1));
    let d = [b.0[0] - a.0[0], b.0[1] - a.0[1]];
    let reach = |ax: &[[f32; 2]; 2], half: [f32; 2], u: [f32; 2]| {
        half[0] * (ax[0][0] * u[0] + ax[0][1] * u[1]).abs()
            + half[1] * (ax[1][0] * u[0] + ax[1][1] * u[1]).abs()
    };
    let (ha, hb) = ([a.2 / 2.0, a.3 / 2.0], [b.2 / 2.0, b.3 / 2.0]);
    aa.iter()
        .chain(ba.iter())
        .map(|&u| reach(&aa, ha, u) + reach(&ba, hb, u) - (d[0] * u[0] + d[1] * u[1]).abs())
        .fold(f32::INFINITY, f32::min)
}

/// How far a car on the move and a bus overlap at most where they're drawn (m; 0 or less: apart).
fn car_bus_overlap(w: &World) -> f32 {
    let (len, wid) = (sim_core::bus::BUS_LEN, sim_core::bus::BUS_WID);
    w.cars
        .iter()
        .filter_map(|c| c.shown.filter(|s| s.2).map(|s| (s.0, s.1, c.len, c.wid)))
        .flat_map(|c| {
            w.transit
                .buses
                .iter()
                .filter_map(|b| b.shown)
                .map(move |b| bodies_overlap(c, (b.0, b.1, len, wid)))
        })
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Ticks `w` on (at most `ticks`) till `done`, every tick checking no two cars overlap, nor a car
/// and a bus; false if it never was.
fn run_clear_of_buses(
    w: &mut World,
    ticks: u32,
    mut each: impl FnMut(&mut World),
    done: impl Fn(&World) -> bool,
) -> bool {
    run_clear(
        w,
        ticks,
        |w| {
            let o = car_bus_overlap(w);
            assert!(o <= 0.05, "a car and a bus overlap by {o:.2} m");
            each(w)
        },
        done,
    )
}

/// The bus of `w` standing at a stop with its doors open, if any: where.
fn bus_at_stop(w: &World) -> Option<[f32; 2]> {
    w.transit
        .buses
        .iter()
        .find_map(|b| b.shown.filter(|s| s.3 != 0).map(|s| s.0))
}

/// Where a bus of `w` in town is, if any.
fn bus_at(w: &World) -> Option<[f32; 2]> {
    w.transit.buses.iter().find_map(|b| b.shown.map(|s| s.0))
}

#[test]
fn a_car_behind_and_one_coming_by_a_bus_at_its_stop_wait_and_never_touch_it() {
    // A 6 m street with the bus stop near its west end (the bus going east, in from the west): one
    // car comes home from the west behind the bus at its stop, another leaves west past it the
    // other way, where neither can pass it.
    let house = |x: f32| House {
        plot: [x - 12.0, 8.0, 16.0, 18.0],
        inner: [x, 20.5],
        front: 1.0,
        along_x: false,
        road: 0,
        s: x - 0.5,
    };
    let roads = format!(
        r#"[{{"points":[{}],"width":6,"kind":"street","ends":["out","out"]}}]"#,
        line_pts([0.5, 30.0], [199.5, 30.0])
    );
    let mut w = town_of(
        BUS_CONTENT,
        "",
        200,
        60,
        &roads,
        r#"[{"x":0.5,"z":30,"kind":"road"},{"x":199.5,"z":30,"kind":"road"},{"x":30,"z":32.4,"kind":"stop"}]"#,
        &[house(60.0), house(70.0)],
    );
    assert_eq!(w.transit.lines.len(), 1);
    common::run_to(&mut w, 2, 7.0);
    let sent = std::cell::Cell::new(false);
    let done = run_clear_of_buses(
        &mut w,
        40 * 60 * 20,
        |w| {
            if !sent.get() && bus_at_stop(w).is_some_and(|p| (p[0] - 30.0).abs() < 6.0) {
                come_home(w, 0);
                set_off(w, 1);
                sent.set(true);
            }
        },
        |w| sent.get() && w.cars[0].state == CarState::Parked && w.cars[1].state == CarState::Away,
    );
    assert!(
        done,
        "not both on their way: {:?}",
        w.cars
            .iter()
            .map(|c| (c.state, c.progress))
            .collect::<Vec<_>>()
    );
    assert_eq!(w.transit.withdrawn, 0, "the bus was taken out of service");
}

#[test]
fn a_car_turning_out_of_a_side_road_takes_its_turn_with_a_bus_at_the_junction() {
    // The side road's car turns out onto the main road as a bus comes along it to the junction (a
    // stop east of it), at a few moments: never into it, nor the bus into the car, and both on.
    let side = House {
        plot: [102.0, 36.0, 16.0, 14.0],
        inner: [110.0, 43.0],
        front: 1.0,
        along_x: true,
        road: 1,
        s: 9.0,
    };
    let roads = format!(
        r#"[{{"points":[{}],"width":7,"kind":"street","ends":["out","out"]}},
            {{"points":[{}],"width":6,"kind":"street","ends":["dead","join"]}}]"#,
        line_pts([0.5, 30.0], [199.5, 30.0]),
        line_pts([120.0, 52.0], [120.0, 30.0])
    );
    for trigger in [96.0, 104.0, 110.0, 116.0] {
        let mut w = town_of(
            BUS_CONTENT,
            "",
            200,
            60,
            &roads,
            r#"[{"x":0.5,"z":30,"kind":"road"},{"x":199.5,"z":30,"kind":"road"},{"x":160,"z":32.4,"kind":"stop"}]"#,
            &[side],
        );
        common::run_to(&mut w, 2, 7.0);
        let sent = std::cell::Cell::new(false);
        let mut bus_past = false;
        let done = run_clear_of_buses(
            &mut w,
            40 * 60 * 20,
            |w| {
                if !sent.get()
                    && bus_at(w).is_some_and(|p| (trigger..trigger + 6.0).contains(&p[0]))
                {
                    set_off(w, 0);
                    sent.set(true);
                }
                bus_past |= sent.get() && bus_at(w).is_some_and(|p| p[0] > 150.0);
            },
            |w| sent.get() && w.cars[0].state == CarState::Away,
        );
        assert!(
            done && bus_past,
            "({trigger}) not both on: {:?}",
            (w.cars[0].state, w.cars[0].progress)
        );
        assert_eq!(
            w.transit.withdrawn, 0,
            "({trigger}) the bus was taken out of service"
        );
    }
}

#[test]
fn a_car_right_behind_a_bus_turning_round_keeps_out_of_its_sweep() {
    // A line in from the west to its one stop and back out west: its bus turns round there, its
    // body sweeping both lanes. A car coming home past the stop from the west comes up behind it as
    // it stands there and turns, at a few moments: it keeps out of the bus's way round (`sweeps`).
    let roads = format!(
        r#"[{{"points":[{}],"width":7,"kind":"street","ends":["out","dead"]}}]"#,
        line_pts([0.5, 30.0], [150.0, 30.0])
    );
    let house = House {
        plot: [62.0, 8.0, 16.0, 18.0],
        inner: [70.0, 20.5],
        front: 1.0,
        along_x: false,
        road: 0,
        s: 69.5,
    };
    for wait in [0, 40, 80, 120] {
        let mut w = town_of(
            BUS_CONTENT,
            "",
            200,
            60,
            &roads,
            r#"[{"x":0.5,"z":30,"kind":"road"},{"x":40,"z":32.4,"kind":"stop"}]"#,
            &[house],
        );
        assert!(
            w.travel.zones.len() > w.travel.graph.boxes.len(),
            "the bus's sweep turning round is a zone"
        );
        common::run_to(&mut w, 2, 7.0);
        let (sent, since) = (std::cell::Cell::new(false), std::cell::Cell::new(None));
        let done = run_clear_of_buses(
            &mut w,
            40 * 60 * 20,
            |w| {
                if since.get().is_none() && bus_at_stop(w).is_some() {
                    since.set(Some(w.tick));
                }
                if !sent.get() && since.get().is_some_and(|t| w.tick >= t + wait) {
                    come_home(w, 0);
                    sent.set(true);
                }
            },
            |w| sent.get() && w.cars[0].state == CarState::Parked,
        );
        assert!(
            done,
            "({wait}) the car never got home: {:?}",
            (w.cars[0].state, w.cars[0].progress)
        );
        assert_eq!(
            w.transit.withdrawn, 0,
            "({wait}) the bus was taken out of service"
        );
    }
}

#[test]
fn a_line_whose_stop_is_by_a_junction_turns_round_on_along_its_road_clear_of_it() {
    // In from the west to its one stop just past a side road's junction, and back out west: the
    // line turns round on along the road clear of the junction (`bus::turn_away`), not at the stop,
    // and is a line all the same (a town with stops has its buses).
    let roads = format!(
        r#"[{{"points":[{}],"width":7,"kind":"street","ends":["out","dead"]}},
            {{"points":[{}],"width":6,"kind":"street","ends":["dead","join"]}}]"#,
        line_pts([0.5, 30.0], [180.0, 30.0]),
        line_pts([60.0, 52.0], [60.0, 30.0])
    );
    let w = town_of(
        BUS_CONTENT,
        "",
        200,
        60,
        &roads,
        r#"[{"x":0.5,"z":30,"kind":"road"},{"x":75,"z":32.4,"kind":"stop"}]"#,
        &[],
    );
    assert_eq!(
        w.transit.lines.len(),
        1,
        "no line to the stop by the junction"
    );
    // (How far east its route comes: where it turns round.)
    let east = w.transit.lines[0]
        .route_points()
        .iter()
        .fold(f32::MIN, |m, p| m.max(p[0]));
    assert!(
        east >= 60.0 + 44.0,
        "it turns round at x = {east:.0}, by the junction at 60"
    );
}

#[test]
fn a_bus_turning_round_gets_round_soon_with_someone_standing_where_it_sweeps() {
    // (Willowley's evening: a line's bus turning round at its stop by the hub waited a tick at a
    // time for someone on foot beside its way, as its front swung round over them, and the cars
    // through the hub waited behind it a minute and more.) Someone stands on the pavement where
    // the bus sweeps turning round past its stop: it gets round soon all the same, never into them.
    let roads = format!(
        r#"[{{"points":[{}],"width":7,"kind":"street","ends":["out","dead"]}}]"#,
        line_pts([0.5, 30.0], [150.0, 30.0])
    );
    let house = House {
        plot: [62.0, 8.0, 16.0, 18.0],
        inner: [70.0, 20.5],
        front: 1.0,
        along_x: false,
        road: 0,
        s: 69.5,
    };
    let mut w = town_of(
        BUS_CONTENT,
        "",
        200,
        60,
        &roads,
        r#"[{"x":0.5,"z":30,"kind":"road"},{"x":40,"z":32.4,"kind":"stop"}]"#,
        &[house],
    );
    common::run_to(&mut w, 2, 7.0);
    // (The bus at its stop, then on its way round: where it turns, as it leaves.)
    let end = w.tick + 60 * 60 * 20;
    while bus_at_stop(&w).is_none() {
        w.tick_once();
        assert!(w.tick < end, "no bus came to the stop");
    }
    while bus_at_stop(&w).is_some() {
        w.tick_once();
    }
    let start = w.tick;
    let (at, yaw) = w.transit.buses[0]
        .shown
        .map(|s| (s.0, s.1))
        .expect("in town");
    // (Where its front swings round over as it turns, half way round.)
    let f = [yaw.sin(), yaw.cos()];
    let swung = yaw + 0.9;
    let spot = [at[0] + swung.sin() * 4.6, at[1] + swung.cos() * 4.6];
    w.sims[0].pos = spot;
    // (Turned round: facing back the way it came.)
    let mut round = None;
    while w.tick < start + 3 * 60 * 20 {
        w.tick_once();
        let Some((p, y)) = w
            .transit
            .buses
            .first()
            .and_then(|b| b.shown.map(|s| (s.0, s.1)))
        else {
            break;
        };
        let back = y.sin() * f[0] + y.cos() * f[1] < -0.9;
        if back && round.is_none() {
            round = Some(w.tick - start);
        }
        let d = (w.sims[0].pos[0] - p[0]).hypot(w.sims[0].pos[1] - p[1]);
        assert!(d > 1.0, "the bus drove over them");
    }
    let took = round.expect("the bus never got round");
    assert!(took < 30 * 20, "the bus took {} s to turn round", took / 20);
}

/// A big world's crossroads at (200, 200), its ways out east and west, two bus stops on either of
/// those arms (a line through it each way: a lit junction), dead ends north and south, and
/// `houses` along those (`crossroads_house`).
fn crossroads(houses: &[House]) -> World {
    let arm = |a: [f32; 2], b: [f32; 2], ends: &str| {
        format!(
            r#"{{"points":[{}],"width":7,"kind":"street","ends":{ends},"area":0}}"#,
            line_pts(a, b)
        )
    };
    let roads = format!(
        "[{},{},{},{}]",
        arm([0.5, 200.0], [200.0, 200.0], r#"["out","join"]"#),
        arm([200.0, 200.0], [399.5, 200.0], r#"["join","out"]"#),
        arm([200.0, 120.0], [200.0, 200.0], r#"["dead","join"]"#),
        arm([200.0, 200.0], [200.0, 280.0], r#"["join","dead"]"#)
    );
    let exits = r#"[{"x":0.5,"z":200,"kind":"road"},{"x":399.5,"z":200,"kind":"road"},
        {"x":230,"z":202.4,"kind":"stop"},{"x":250,"z":202.4,"kind":"stop"},
        {"x":170,"z":197.6,"kind":"stop"},{"x":150,"z":197.6,"kind":"stop"}]"#;
    let more = r#","size":"big","walls":[],"doors":[],"windows":[],"relationships":[],
        "areas":[{"name":"Crossroads","kind":"hamlet","x":130,"z":130,"w":140,"d":140}]"#;
    town_of(BUS_CONTENT, more, 400, 400, &roads, exits, houses)
}

/// A house by the crossroads' north arm (`z` below 200, west of it) or south arm (east of it),
/// its driveway onto it (`crossroads`).
fn crossroads_house(z: f32, west: bool) -> House {
    House {
        plot: if west {
            [180.0, z - 8.0, 14.0, 16.0]
        } else {
            [206.0, z - 8.0, 14.0, 16.0]
        },
        inner: if west { [193.0, z] } else { [207.0, z] },
        front: if west { 1.0 } else { -1.0 },
        along_x: true,
        road: if z < 200.0 { 2 } else { 3 },
        s: if z < 200.0 { z - 120.0 } else { z - 200.0 },
    }
}

#[test]
fn a_lit_junctions_signal_heads_are_in_the_snapshot_one_an_arm_on_its_kerb() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, header, light};
    let mut w = crossroads(&[]);
    assert!(w.travel.signals.is_empty(), "no lights in a game yet");
    w.light_busy_junctions();
    assert_eq!(
        w.travel.signals.len(),
        1,
        "the crossroads two lines go through is lit"
    );
    let layout: serde_json::Value =
        serde_json::from_str(&snapshot::layout_json(&w.content)).unwrap();
    assert_eq!(layout["header"]["lightCount"], header::LIGHT_COUNT);
    assert_eq!(layout["lightBase"], LIGHT_BASE);
    assert_eq!(layout["lightStride"], LIGHT_STRIDE);
    assert_eq!(layout["maxLights"], snapshot::MAX_LIGHTS);
    for (key, at) in [
        ("shown", light::SHOWN),
        ("x", light::X),
        ("z", light::Z),
        ("yaw", light::YAW),
        ("state", light::STATE),
    ] {
        assert_eq!(layout["light"][key], at, "light.{key}");
    }
    assert_eq!(layout["light"]["states"]["green"], light::GREEN as f64);
    let mut buf = vec![0.0f32; snapshot::CAPACITY];
    common::run_to(&mut w, 2, 7.0);
    let end = w.tick + 30 * 60 * 20;
    let mut greens = std::collections::BTreeSet::new();
    while w.tick < end {
        w.tick_once();
        snapshot::write(&w, &mut buf);
        assert_eq!(buf[header::LIGHT_COUNT], 4.0, "a head an arm");
        let mut lit = 0;
        for k in 0..4 {
            let o = &buf[LIGHT_BASE + k * LIGHT_STRIDE..LIGHT_BASE + (k + 1) * LIGHT_STRIDE];
            assert_eq!(o[light::SHOWN], 1.0);
            // (On a pavement by the junction: past the kerb, 2.15 m from a 7 m street's middle,
            // and on the street, near its box.)
            let (x, z) = (o[light::X], o[light::Z]);
            let off = (x - 200.0).abs().min((z - 200.0).abs());
            assert!((2.15..3.5).contains(&off), "head {k} at {x}, {z}");
            assert!((x - 200.0).hypot(z - 200.0) < 25.0, "head {k} at {x}, {z}");
            let state = o[light::STATE];
            assert!([light::RED, light::AMBER, light::GREEN].contains(&state));
            if state != light::RED {
                lit += 1;
            }
            if state == light::GREEN {
                greens.insert(k);
            }
        }
        assert!(lit <= 1, "two arms not red at once");
    }
    assert!(greens.len() >= 2, "the lights changed: {greens:?}");
}

#[test]
fn a_lit_junctions_poles_stand_clear_of_the_bus_stops_by_it() {
    // The crossroads with a stop right by its east arm, on the side its pole would stand: the pole
    // stands further out along the arm instead.
    let arm = |a: [f32; 2], b: [f32; 2], ends: &str| {
        format!(
            r#"{{"points":[{}],"width":7,"kind":"street","ends":{ends},"area":0}}"#,
            line_pts(a, b)
        )
    };
    let roads = format!(
        "[{},{},{},{}]",
        arm([0.5, 200.0], [200.0, 200.0], r#"["out","join"]"#),
        arm([200.0, 200.0], [399.5, 200.0], r#"["join","out"]"#),
        arm([200.0, 120.0], [200.0, 200.0], r#"["dead","join"]"#),
        arm([200.0, 200.0], [200.0, 280.0], r#"["join","dead"]"#)
    );
    let exits = r#"[{"x":0.5,"z":200,"kind":"road"},{"x":399.5,"z":200,"kind":"road"},
        {"x":211,"z":197.6,"kind":"stop"},{"x":250,"z":202.4,"kind":"stop"},
        {"x":170,"z":197.6,"kind":"stop"},{"x":150,"z":202.4,"kind":"stop"}]"#;
    let more = r#","size":"big","walls":[],"doors":[],"windows":[],"relationships":[],
        "areas":[{"name":"Crossroads","kind":"hamlet","x":130,"z":130,"w":140,"d":140}]"#;
    let mut w = town_of(BUS_CONTENT, more, 400, 400, &roads, exits, &[]);
    w.light_busy_junctions();
    assert_eq!(w.travel.signals.len(), 1, "the crossroads is lit");
    for arm in &w.travel.signals[0].arms {
        for st in &w.transit.stops {
            let d = (arm.pole[0] - st.at[0]).hypot(arm.pole[1] - st.at[1]);
            assert!(
                d >= sim_core::traffic::POLE_CLEAR,
                "a pole at {:?} {d:.1} m from the stop at {:?}",
                arm.pole,
                st.at
            );
        }
    }
}

#[test]
fn a_game_saved_with_the_lights_part_way_loads_with_them_where_they_were() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, light};
    let mut w = crossroads(&[]);
    w.light_busy_junctions();
    common::run_to(&mut w, 2, 7.0);
    let mut buf = vec![0.0f32; snapshot::CAPACITY];
    let states = |w: &World, buf: &mut Vec<f32>| {
        snapshot::write(w, buf);
        (0..4)
            .map(|k| buf[LIGHT_BASE + k * LIGHT_STRIDE + light::STATE])
            .collect::<Vec<_>>()
    };
    // (Some arm but the first's turn, or amber: not where they start over.)
    let first = states(&w, &mut buf);
    let end = w.tick + 30 * 60 * 20;
    while states(&w, &mut buf) == first && w.tick < end {
        w.tick_once();
    }
    let was = states(&w, &mut buf);
    assert_ne!(was, first, "the lights never changed");
    let loaded = World::from_save_json(BUS_CONTENT, &w.save_json()).unwrap();
    assert_eq!(states(&loaded, &mut buf), was);
}

#[test]
fn cars_from_every_arm_of_a_busy_crossroads_all_get_through_with_the_buses() {
    // A big world's crossroads, its ways out east and west, two bus stops on either of those arms
    // (a line through it each way): two cars leave from houses on the north and south arms, two
    // come home to houses there, all across the junction as a bus comes to it, at a few moments.
    // (Houses west of the north arm and east of the south one, their driveways onto them.)
    let houses = [
        crossroads_house(170.0, true),
        crossroads_house(230.0, false),
        crossroads_house(178.0, true),
        crossroads_house(222.0, false),
    ];
    for wait in [0, 30, 60] {
        let mut w = crossroads(&houses);
        assert!(
            w.transit.lines.len() >= 2,
            "{} lines",
            w.transit.lines.len()
        );
        common::run_to(&mut w, 2, 7.0);
        let (sent, since) = (std::cell::Cell::new(false), std::cell::Cell::new(None));
        let done = run_clear_of_buses(
            &mut w,
            60 * 60 * 20,
            |w| {
                let near = w.transit.buses.iter().any(|b| {
                    b.shown
                        .is_some_and(|s| (s.0[0] - 200.0).hypot(s.0[1] - 200.0) < 30.0)
                });
                if since.get().is_none() && near {
                    since.set(Some(w.tick));
                }
                if !sent.get() && since.get().is_some_and(|t| w.tick >= t + wait) {
                    set_off(w, 0);
                    set_off(w, 1);
                    come_home(w, 2);
                    come_home(w, 3);
                    sent.set(true);
                }
            },
            |w| {
                sent.get()
                    && w.cars[0].state == CarState::Away
                    && w.cars[1].state == CarState::Away
                    && w.cars[2].state == CarState::Parked
                    && w.cars[3].state == CarState::Parked
            },
        );
        assert!(
            done,
            "({wait}) not all through: {:?}",
            w.cars
                .iter()
                .map(|c| (c.state, c.progress))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            w.transit.withdrawn, 0,
            "({wait}) a bus was taken out of service"
        );
    }
}
