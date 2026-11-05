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

/// The least gap between two of `w`'s cars' bodies where they're drawn, one of them on the move (m:
/// the least distance from a corner of either to a side of the other; below 0, how far they
/// overlap), and which.
fn cars_gap(w: &World) -> (f32, usize, usize) {
    let corners = |p: [f32; 2], yaw: f32, len: f32, wid: f32| {
        let (s, c) = yaw.sin_cos();
        let (f, r) = (
            [s * len / 2.0, c * len / 2.0],
            [c * wid / 2.0, -s * wid / 2.0],
        );
        [(1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (-1.0, 1.0)]
            .map(|(a, b)| [p[0] + a * f[0] + b * r[0], p[1] + a * f[1] + b * r[1]])
    };
    let to_side = |p: [f32; 2], a: [f32; 2], b: [f32; 2]| {
        let ab = [b[0] - a[0], b[1] - a[1]];
        let t = ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]);
        let t = t.clamp(0.0, 1.0);
        (p[0] - a[0] - ab[0] * t).hypot(p[1] - a[1] - ab[1] * t)
    };
    let (worst, a, b) = cars_overlap(w);
    if worst > 0.0 {
        return (-worst, a, b);
    }
    let shown: Vec<_> = w
        .cars
        .iter()
        .enumerate()
        .filter_map(|(k, c)| c.shown.map(|s| (k, s.2, corners(s.0, s.1, c.len, c.wid))))
        .collect();
    let mut least = (f32::INFINITY, 0, 0);
    for (i, a) in shown.iter().enumerate() {
        for b in &shown[i + 1..] {
            if !(a.1 || b.1) {
                continue;
            }
            for (p, q) in [(&a.2, &b.2), (&b.2, &a.2)] {
                for &v in p {
                    for k in 0..4 {
                        let g = to_side(v, q[k], q[(k + 1) % 4]);
                        if g < least.0 {
                            least = (g, a.0, b.0);
                        }
                    }
                }
            }
        }
    }
    least
}

/// How near the cars come at least (m between their bodies: `cars::GAP`, less its tolerance).
const CAR_GAP: f32 = 0.149;

/// Car 0 comes home along the street from its west end, car 1 sets off from its house further
/// east `after` ticks on, west for the way out: both on their way within `ticks`, never nearer
/// each other than `CAR_GAP`.
fn turn_in_by_one_pulling_out(width: f32, houses: &[(f32, bool)], after: u32, ticks: u32) {
    let mut w = street(width, houses);
    come_home(&mut w, 0);
    for t in 0..ticks {
        if t == after {
            set_off(&mut w, 1);
        }
        w.tick_once();
        let (g, a, b) = cars_gap(&w);
        assert!(
            g >= CAR_GAP,
            "cars {a} and {b} came {g:.3} m from each other at tick {t}: {:?}",
            w.cars
                .iter()
                .map(|c| (c.state, c.shown.map(|s| s.0), c.shift, c.waits_for))
                .collect::<Vec<_>>()
        );
        if t > after && w.cars[0].state == CarState::Parked && w.cars[1].state == CarState::Away {
            return;
        }
    }
    panic!(
        "not both on their way: {:?}",
        w.cars
            .iter()
            .map(|c| (c.state, c.progress))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_car_turning_in_home_across_one_pulling_out_next_door_keeps_its_distance() {
    // (On a 7 m street: the one coming home turns in across the other's lane as it comes by, the
    // one pulling out pulled over as far as it can: it waits for that one to pass, never nearer
    // than the cars keep. They came within a hair of each other.)
    turn_in_by_one_pulling_out(7.0, &[(60.0, true), (100.0, true)], 0, 4000);
}

#[test]
fn on_a_6_m_lane_a_car_passing_one_pulled_over_keeps_its_distance_or_waits() {
    // (Coming home to its house, about to turn in, it can't pull over itself; the one pulled
    // out next door and pulled over for it isn't over far enough for the gap the cars keep: it
    // waits for that one to pass.)
    turn_in_by_one_pulling_out(6.0, &[(60.0, true), (70.0, true)], 100, 4000);
}

#[test]
fn a_car_pulling_out_across_the_street_as_one_turns_in_gets_out() {
    // (On a 7 m street, the one pulling out of its driveway opposite, or a little further on,
    // still turning out as the one coming home turns in across its lane: the two come within the
    // gap the cars keep of each other, the one coming home backs up to let it finish its turn
    // (and stays back while it waits its turn), and both get on their way. They waited for each
    // other for good, the one coming home backing up and coming back up to it by turns.)
    turn_in_by_one_pulling_out(7.0, &[(60.0, true), (70.0, false)], 100, 4000);
    turn_in_by_one_pulling_out(7.0, &[(60.0, true), (100.0, false)], 0, 4000);
}

#[test]
fn a_taxi_waiting_on_a_bend_for_a_car_turning_out_lets_it_by_clear() {
    // (Oak Clearing, a normal town of the 0.7.0 samples: about 18:34 a car turns out of its
    // driveway onto a 6 m lane's bend and on along it, as the work taxi coming the other way waits
    // its turn after it there, pulled over. The car came within 0.1 m of it; kept clear of it, it
    // waited for the taxi, which pulled back in for it, into its way, and the two waited for each
    // other for good. Standing, waiting for it, the taxi pulls over as far as it can.)
    let content = common::content();
    let town = include_str!("../towns/oak_clearing.json");
    let mut w = World::from_json(&content, town, 46).unwrap();
    let end = sim_core::clock::tick_at(1, 19.5 * 60.0).unwrap();
    assert!(
        w.tick < end,
        "the town begins at {:.2}h",
        sim_core::clock::hour(w.tick)
    );
    while w.tick < end {
        w.tick_once();
        let (g, a, b) = cars_gap(&w);
        assert!(
            g >= CAR_GAP,
            "cars {a} and {b} came {g:.3} m from each other at {:.2}h",
            sim_core::clock::hour(w.tick)
        );
    }
    // (It set off about 18:34: out of town by now, as every household's car on its way set off
    // since.)
    for (c, car) in w.cars.iter().enumerate().filter(|(_, c)| !c.taxi) {
        let on = matches!(car.state, CarState::Leaving | CarState::Arriving);
        assert!(
            !on || w.tick - car.since < 45 * sim_core::TICKS_PER_MINUTE,
            "car {c} {:?} since {:.2}h at {:?}",
            car.state,
            sim_core::clock::hour(car.since),
            car.shown
        );
    }
}

#[test]
fn a_car_gets_past_someone_walking_ahead_of_it_along_its_lane() {
    // (As on Barleystead's road out of town: one walking out along the lane in front of a car,
    // the way it goes, steps aside for it in the end, not followed at their pace all the way.)
    let mut w = street(7.0, &[(90.0, true), (170.0, true)]);
    set_off(&mut w, 0);
    let mut walking = None;
    let mut away = None;
    for t in 0..4000u32 {
        w.tick_once();
        let Some((at, yaw, true, _)) = w.cars[0].shown else {
            if w.cars[0].state == CarState::Away && walking.is_some() {
                away = Some(t);
                break;
            }
            continue;
        };
        // On the street, westbound: D1 in its lane 6 m in front, off west along it.
        if walking.is_none() && (at[1] - 30.0).abs() < 2.5 && yaw.sin() < -0.99 {
            w.sims[1].pos = [at[0] - 6.0, at[1]];
            w.apply(sim_core::Command::MoveTo {
                sim: 1,
                x: 2.5,
                z: at[1],
            })
            .unwrap();
            walking = Some((t, at[0]));
        }
    }
    let (from, x) = walking.expect("the car came out onto the street, westbound");
    let gone = away.expect("the car never got away") - from;
    // (Following them at their pace, it would be out of town about when they are: past them,
    // it's long gone while they're still on the first half of the way.)
    assert!(
        w.sims[1].pos[0] > x / 2.0,
        "the car followed D1 for {gone} ticks, out of town with them at {:?}",
        w.sims[1].pos
    );
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
fn one_turning_out_of_a_side_road_and_one_coming_home_past_it_both_get_through_its_box() {
    // (As at Pebble Bay: one coming home along the main road from the west to a house just east
    // of the junction, as one turns out of the side road through its box: the claims let them in
    // together where they'd pass no further apart than the gap the cars keep, and in the box,
    // where nobody pulls over, they waited for each other for good. One of them waits.)
    let side = |s: f32, west: bool| House {
        plot: [if west { 102.0 } else { 122.0 }, 45.0 - s, 16.0, 14.0],
        inner: [if west { 110.0 } else { 130.0 }, 52.0 - s],
        front: if west { 1.0 } else { -1.0 },
        along_x: true,
        road: 1,
        s,
    };
    let home = House {
        plot: [133.0, 8.0, 16.0, 18.0],
        inner: [141.0, 20.5],
        front: 1.0,
        along_x: false,
        road: 0,
        s: 140.5,
    };
    for out in [side(12.0, true), side(6.0, false), side(12.0, false)] {
        let mut w = t_junction(&[home, out]);
        come_home(&mut w, 0);
        set_off(&mut w, 1);
        let mut least = f32::INFINITY;
        let done = run_clear(
            &mut w,
            4000,
            |w| least = least.min(cars_gap(w).0),
            |w| w.cars[0].state == CarState::Parked && w.cars[1].state == CarState::Away,
        );
        assert!(
            done,
            "({:?}) not both on their way: {:?}",
            out.inner,
            w.cars
                .iter()
                .map(|c| (c.state, c.progress))
                .collect::<Vec<_>>()
        );
        assert!(
            least >= CAR_GAP,
            "({:?}) they came {least:.3} m apart",
            out.inner
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

// ---- Traffic lights (`traffic::Signal`) ---------------------------------------------------------

/// A town's content.
const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],
    "objects":[],
    "economy":{"startingFunds":500}}"#;

/// A big world's crossroads at (200, 200), its ways out east and west, dead ends north and south,
/// and `houses` along those (`crossroads_house`).
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
    let exits = r#"[{"x":0.5,"z":200,"kind":"road"},{"x":399.5,"z":200,"kind":"road"}]"#;
    let more = r#","size":"big","walls":[],"doors":[],"windows":[],"relationships":[],
        "areas":[{"name":"Crossroads","kind":"hamlet","x":130,"z":130,"w":140,"d":140}]"#;
    town_of(CONTENT, more, 400, 400, &roads, exits, houses)
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

/// The crossroads with a house on each of its dead-end arms either side, their cars coming and
/// going all the time (`keep_driving`): someone waiting at the lights on every arm.
fn busy_crossroads() -> World {
    crossroads(&[
        crossroads_house(170.0, true),
        crossroads_house(230.0, false),
        crossroads_house(178.0, true),
        crossroads_house(222.0, false),
    ])
}

/// Every car of `w` parked at home sets off, every one away comes home.
fn keep_driving(w: &mut World) {
    for c in 0..w.cars.len() {
        match w.cars[c].state {
            CarState::Parked => set_off(w, c),
            CarState::Away => come_home(w, c),
            _ => {}
        }
    }
}

#[test]
fn a_lit_junctions_signal_heads_are_in_the_snapshot_one_an_arm_on_its_kerb() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, header, light};
    let mut w = busy_crossroads();
    assert!(w.travel.signals.is_empty(), "no lights in a game yet");
    w.light_busy_junctions();
    assert_eq!(w.travel.signals.len(), 1, "the crossroads is lit");
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
        keep_driving(&mut w);
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
fn a_game_saved_with_the_lights_part_way_loads_with_them_where_they_were() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, light};
    let mut w = busy_crossroads();
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
        keep_driving(&mut w);
        w.tick_once();
    }
    let was = states(&w, &mut buf);
    assert_ne!(was, first, "the lights never changed");
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(states(&loaded, &mut buf), was);
}

#[test]
fn junctions_past_the_snapshots_room_for_signal_heads_are_never_lit() {
    // A grid of 9 streets each way, 81 crossroads of four arms: 324 heads were they all lit, past
    // the snapshot's rows. Those lit fit, every one with a head an arm.
    let lines: Vec<String> = (0..9)
        .flat_map(|k| {
            let c = 40.0 + 40.0 * k as f32;
            let road = |a: [f32; 2], b: [f32; 2], ends: &str| {
                format!(
                    r#"{{"points":[{}],"width":7,"kind":"street","ends":{ends},"area":0}}"#,
                    line_pts(a, b)
                )
            };
            [
                road([0.5, c], [399.5, c], r#"["out","out"]"#),
                road([c, 0.5], [c, 399.5], r#"["out","out"]"#),
            ]
        })
        .collect();
    let roads = format!("[{}]", lines.join(","));
    let exits = r#"[{"x":0.5,"z":40,"kind":"road"}]"#;
    let more = r#","size":"big","walls":[],"doors":[],"windows":[],"relationships":[],
        "areas":[{"name":"Grid","kind":"hamlet","x":0,"z":0,"w":400,"d":400}]"#;
    let mut w = town_of(CONTENT, more, 400, 400, &roads, exits, &[]);
    assert!(w.travel.signals.is_empty(), "nothing lit in a game");
    w.light_busy_junctions();
    let heads: usize = w.travel.signals.iter().map(|s| s.arms.len()).sum();
    assert!(!w.travel.signals.is_empty());
    assert!(heads <= sim_core::snapshot::MAX_LIGHTS, "{heads} heads");
    assert!(w.travel.signals.iter().all(|s| s.arms.len() == 4));
    let mut buf = vec![0.0f32; sim_core::snapshot::CAPACITY];
    sim_core::snapshot::write(&w, &mut buf);
    assert_eq!(
        buf[sim_core::snapshot::header::LIGHT_COUNT] as usize,
        heads,
        "a row a head"
    );
}

#[test]
fn a_car_pulling_up_at_a_light_resting_on_another_arms_green_gets_its_turn_soon() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, light};
    // The crossroads lit, nobody about: whichever arm rests on green. A car comes home along the
    // north arm's dead end: its arm's green within the 50 s every turn comes round in.
    let mut w = crossroads(&[crossroads_house(170.0, true)]);
    w.light_busy_junctions();
    common::run_to(&mut w, 2, 7.0);
    // (Resting: a while with nobody at it.)
    for _ in 0..20 * 20 {
        w.tick_once();
    }
    let mut buf = vec![0.0f32; snapshot::CAPACITY];
    snapshot::write(&w, &mut buf);
    let resting: Vec<f32> = (0..4)
        .map(|k| buf[LIGHT_BASE + k * LIGHT_STRIDE + light::STATE])
        .collect();
    set_off(&mut w, 0);
    let (start, mut through) = (w.tick, None);
    while w.tick < start + 60 * 60 * 20 && through.is_none() {
        w.tick_once();
        // (Past the junction's middle on its way out east or west.)
        if w.cars[0]
            .shown
            .is_some_and(|s| (s.0[0] - 200.0).abs() > 15.0)
            && w.cars[0].progress > 0.0
        {
            through = Some(w.tick);
        }
    }
    let through = through.expect("never through the junction");
    // (Its drive out of its driveway and down the arm, and the wait for its turn: a minute or so
    // in all, the turn itself 50 s at most.)
    assert!(
        through - start < 50 * 20 + 3 * 60 * 20,
        "through after {} s, the lights resting at {resting:?}",
        (through - start) / 20
    );
}
