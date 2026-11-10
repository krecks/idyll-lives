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
    for _ in 0..common::TICKS_PER_DAY {
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
    for _ in 0..common::TICKS_PER_DAY {
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
    w.sims[0].away_until = Some(w.tick + common::mins(5000.0));
    // ...and Bo stands in front of it between the boxes, with no room to step aside.
    w.sims[1].pos = [34.0, 6.5];
    let mut closest = f32::INFINITY;
    for _ in 0..common::secs(100.0) {
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

/// A town 200 m long with a street `width` m wide along x (its middle at z = 30, a way out at either
/// end) and a house at each of `houses` (its driveway's x, and whether it's north of the street),
/// with its driveway onto the street and a car on it, and its driver at home.
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
    w.sims[c].away_until = Some(w.tick + common::mins(5000.0));
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
    w.sims[c].away_until = Some(w.tick + common::mins(5000.0));
}

/// How far two of `w`'s cars' bodies overlap at most where they're drawn, one of them on the move
/// (m; 0 or less: apart), and which.
fn cars_overlap(w: &World) -> (f32, usize, usize) {
    overlap_where(w, |_| true)
}

/// As `cars_overlap`, of the pairs one of which is `counted`.
fn overlap_where(w: &World, counted: impl Fn(usize) -> bool) -> (f32, usize, usize) {
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
            if (a.1 || b.1) && (counted(a.0) || counted(b.0)) {
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
    for _ in 0..common::secs(200.0) {
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
        a.abs_diff(b) >= common::drive_secs(0.5),
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
    for _ in 0..common::secs(200.0) {
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
    assert!(
        stood.iter().all(|&s| s < common::drive_secs(1.0)),
        "they stood {stood:?} ticks"
    );
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

/// A town 200 by 60 m with `roads` (JSON, `kind` street) and `exits` (JSON) and a car at each of
/// `houses`, its driver at home.
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
    ticks: u64,
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
        common::secs(300.0),
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
fn turn_in_by_one_pulling_out(width: f32, houses: &[(f32, bool)], after: u64, ticks: u64) {
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
    turn_in_by_one_pulling_out(7.0, &[(60.0, true), (100.0, true)], 0, common::secs(200.0));
}

#[test]
fn on_a_6_m_lane_a_car_passing_one_pulled_over_keeps_its_distance_or_waits() {
    // (Coming home to its house, about to turn in, it can't pull over itself; the one pulled
    // out next door and pulled over for it isn't over far enough for the gap the cars keep: it
    // waits for that one to pass.)
    turn_in_by_one_pulling_out(
        6.0,
        &[(60.0, true), (70.0, true)],
        common::drive_secs(5.0),
        common::secs(200.0),
    );
}

#[test]
fn a_car_pulling_out_across_the_street_as_one_turns_in_gets_out() {
    // (On a 7 m street, the one pulling out of its driveway opposite, or a little further on,
    // still turning out as the one coming home turns in across its lane: the two come within the
    // gap the cars keep of each other, the one coming home backs up to let it finish its turn
    // (and stays back while it waits its turn), and both get on their way. They waited for each
    // other for good, the one coming home backing up and coming back up to it by turns.)
    turn_in_by_one_pulling_out(
        7.0,
        &[(60.0, true), (70.0, false)],
        common::drive_secs(5.0),
        common::secs(200.0),
    );
    turn_in_by_one_pulling_out(7.0, &[(60.0, true), (100.0, false)], 0, common::secs(200.0));
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
    for t in 0..common::secs(200.0) {
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
        common::secs(200.0),
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
            common::secs(300.0),
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
            common::secs(200.0),
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
        common::secs(180.0),
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
        ticks > common::drive_secs(10.0),
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
fn a_lit_junctions_heads_stand_at_its_stop_lines_and_arms_across_go_green_together() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, header, light};
    let mut w = busy_crossroads();
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
    let end = w.tick + common::secs(1800.0);
    let mut greens = std::collections::BTreeSet::new();
    while w.tick < end {
        keep_driving(&mut w);
        w.tick_once();
        snapshot::write(&w, &mut buf);
        assert_eq!(buf[header::LIGHT_COUNT], 4.0, "a head an arm");
        let sig = &w.travel.signals[0];
        let mut lit = std::collections::BTreeSet::new();
        for k in 0..4 {
            let o = &buf[LIGHT_BASE + k * LIGHT_STRIDE..LIGHT_BASE + (k + 1) * LIGHT_STRIDE];
            assert_eq!(o[light::SHOWN], 1.0);
            // (On a pavement by the junction: past the kerb, 2.15 m from a 7 m street's middle,
            // and at its arm's stop line.)
            let (x, z) = (o[light::X], o[light::Z]);
            let off = (x - 200.0).abs().min((z - 200.0).abs());
            assert!((2.15..3.5).contains(&off), "head {k} at {x}, {z}");
            let out = (x - 200.0).abs().max((z - 200.0).abs());
            assert!(
                (out - sig.arms[k].stop).abs() < 0.05,
                "head {k} at {x}, {z}"
            );
            let state = o[light::STATE];
            assert!([light::RED, light::AMBER, light::GREEN, light::WALK].contains(&state));
            if state == light::GREEN || state == light::AMBER {
                lit.insert(sig.arms[k].phase);
            }
            if state == light::GREEN {
                greens.insert(k);
                // (Across from it, green too: the arms across from each other go together.)
                let across = (0..4)
                    .find(|&j| {
                        let (a, b) = (sig.arms[k].way, sig.arms[j].way);
                        a[0] * b[0] + a[1] * b[1] < -0.9
                    })
                    .unwrap();
                let o = &buf[LIGHT_BASE + across * LIGHT_STRIDE..];
                assert_eq!(
                    o[light::STATE],
                    light::GREEN,
                    "head {across} across from {k}"
                );
            }
        }
        assert!(lit.len() <= 1, "two phases not red at once: {lit:?}");
    }
    assert!(greens.len() >= 2, "the lights changed: {greens:?}");
}

#[test]
fn every_junction_is_in_the_structure_with_its_arms_crosswalks_and_stop_lines() {
    // The renderer's layout of a junction (`junctions`, web `JunctionInfo`): its middle, and per
    // arm its road, its way out, its width between the kerbs, its crosswalk and stop line out
    // from the middle, and its signal head's row. A crossroads of 7 m streets: four arms, each a
    // way out along x or z, 4.3 m between the kerbs.
    let w = busy_crossroads();
    let s: serde_json::Value = serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    let junctions = s["junctions"].as_array().expect("junctions");
    assert_eq!(junctions.len(), 1, "{junctions:?}");
    let j = &junctions[0];
    assert_eq!(
        (j["x"].as_f64(), j["z"].as_f64()),
        (Some(200.0), Some(200.0))
    );
    assert!(j.get("roundabout").is_none(), "not on a roundabout");
    let arms = j["arms"].as_array().expect("arms");
    assert_eq!(arms.len(), 4);
    let mut rows = Vec::new();
    let mut ways = Vec::new();
    for a in arms {
        let keys: std::collections::BTreeSet<&str> =
            a.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "dir", "light", "road", "stop", "walkFrom", "walkTo", "width"
            ]
            .into_iter()
            .collect(),
        );
        let f = |k: &str| a[k].as_f64().unwrap() as f32;
        assert!(a["road"].as_u64().unwrap() < 4, "a road of the town's: {a}");
        let dir = [a["dir"][0].as_f64().unwrap(), a["dir"][1].as_f64().unwrap()];
        assert!(
            (dir[0].hypot(dir[1]) - 1.0).abs() < 1e-3,
            "a unit way out: {a}"
        );
        assert!(
            dir[0].abs() < 1e-3 || dir[1].abs() < 1e-3,
            "along x or z: {a}"
        );
        ways.push([dir[0].round() as i32, dir[1].round() as i32]);
        assert!((f("width") - 4.3).abs() < 1e-3, "kerb to kerb: {a}");
        // (The crosswalk from the corner, past the box's edge, 3 m wide; the stop line just past.)
        assert!(f("walkFrom") >= 6.0 - 1e-3, "{a}");
        assert!((f("walkTo") - f("walkFrom") - sim_core::traffic::CROSSWALK).abs() < 1e-3);
        assert!((f("stop") - f("walkTo") - sim_core::traffic::STOP_GAP).abs() < 1e-3);
        rows.push(a["light"].as_u64().expect("lit"));
    }
    ways.sort();
    assert_eq!(ways, [[-1, 0], [0, -1], [0, 1], [1, 0]]);
    assert_eq!(rows, [0, 1, 2, 3], "the heads' rows in order");
    // (The poles too, for the kerbside furniture.)
    assert_eq!(s["lightPoles"].as_array().map(Vec::len), Some(4));
}

#[test]
fn a_game_saved_with_the_lights_part_way_loads_with_them_where_they_were() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, light};
    let mut w = busy_crossroads();
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
    let end = w.tick + common::secs(1800.0);
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
fn a_game_saved_before_lights_were_lit_loads_with_them_on() {
    use sim_core::snapshot::{self, header};
    // A save from before the junctions were lit has no lights in it: the roads still light them.
    let w = busy_crossroads();
    let mut save: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    save.as_object_mut().unwrap().remove("lights");
    let loaded = World::from_save_json(CONTENT, &save.to_string()).unwrap();
    assert_eq!(loaded.travel.signals.len(), 1, "the crossroads is lit");
    let mut buf = vec![0.0f32; snapshot::CAPACITY];
    snapshot::write(&loaded, &mut buf);
    assert_eq!(buf[header::LIGHT_COUNT], 4.0, "a head an arm");
}

#[test]
fn every_junction_of_a_grid_bigger_than_any_generated_worlds_is_lit_a_row_a_head() {
    // A grid of 9 streets each way, 81 crossroads of four arms: 324 heads, about as many as the
    // generated big worlds' most (the snapshot has room for more). Every junction is lit, every one
    // with a head an arm, and a row in the snapshot for each; none left unlit for want of rows.
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
    let w = town_of(CONTENT, more, 400, 400, &roads, exits, &[]);
    let heads: usize = w.travel.signals.iter().map(|s| s.arms.len()).sum();
    assert_eq!(w.travel.signals.len(), 81);
    assert_eq!(heads, 324);
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

/// The crossroads' arm out along `way` (`crossroads`), its index of the lit junction's.
fn crossroads_arm(w: &World, way: [f32; 2]) -> usize {
    let sig = &w.travel.signals[0];
    (0..sig.arms.len())
        .find(|&k| sig.arms[k].way[0] * way[0] + sig.arms[k].way[1] * way[1] > 0.9)
        .expect("an arm that way")
}

/// Ends of a walk across the crossroads' north arm by its crosswalk (`crossroads`): on the pavement
/// west of it and east of it.
fn across_north(w: &World) -> ([f32; 2], [f32; 2]) {
    let sig = &w.travel.signals[0];
    let arm = sig.arms[crossroads_arm(w, [0.0, -1.0])];
    let z = sig.at[1] - (arm.walk_from + arm.walk_to) / 2.0;
    (
        [sig.at[0] - arm.kerb - 0.7, z],
        [sig.at[0] + arm.kerb + 0.7, z],
    )
}

/// Resident `i` of `w` walks to `to`.
fn walk_to(w: &mut World, i: u32, to: [f32; 2]) {
    w.apply(sim_core::Command::MoveTo {
        sim: i,
        x: to[0],
        z: to[1],
    })
    .unwrap();
}

#[test]
fn someone_on_foot_at_a_lit_crosswalk_waits_at_the_kerb_for_the_walk_then_crosses() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, light};
    // The crossroads, its lights resting on the cars' first phase: someone on the pavement by the
    // north arm's crosswalk walks across it. They wait at the kerb for the walk phase their
    // waiting brings round (the cars' green, its amber and all red first), and cross on it.
    let mut w = crossroads(&[crossroads_house(150.0, true)]);
    common::run_to(&mut w, 2, 7.0);
    let sig = w.travel.signals[0].clone();
    let north = crossroads_arm(&w, [0.0, -1.0]);
    let (from, to) = across_north(&w);
    w.sims[0].pos = from;
    walk_to(&mut w, 0, to);
    let mut buf = vec![0.0f32; snapshot::CAPACITY];
    let (mut waited, mut entered, mut across) = (0, None, false);
    let mut states = std::collections::BTreeSet::new();
    for _ in 0..common::secs(90.0) {
        w.tick_once();
        snapshot::write(&w, &mut buf);
        let head = buf[LIGHT_BASE + north * LIGHT_STRIDE + light::STATE];
        states.insert(head as i32);
        let p = w.sims[0].pos;
        if sig.crossing(p) {
            entered.get_or_insert(head);
        } else if entered.is_none() && head != light::WALK {
            waited += 1;
        }
        if (p[0] - to[0]).hypot(p[1] - to[1]) < 0.3 {
            across = true;
            break;
        }
    }
    assert_eq!(entered, Some(light::WALK), "onto the crosswalk on the walk");
    assert!(across, "never across: at {:?}", w.sims[0].pos);
    // (Amber and all red before the walk: six seconds of the cars' drive time at least at the
    // kerb, a second and a half of real time.)
    assert!(
        waited >= common::drive_secs(5.0),
        "waited {waited} ticks; the lights showed {states:?}"
    );
}

#[test]
fn no_car_drives_past_a_stop_line_on_red_or_the_walk_and_on_red_each_waits_at_it() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, light};
    // The busy crossroads, its cars coming and going all the time, and someone walking to and fro
    // across its north arm: no car's nose goes past its arm's stop line but on its phase's green
    // (or its amber, or held up just after on its way through: green a moment ago), never on
    // the walk; a car waiting at a red light stands with its nose at the stop line, short of the
    // crosswalk. And the walk comes round.
    let mut w = crossroads(&[
        crossroads_house(170.0, true),
        crossroads_house(230.0, false),
        crossroads_house(178.0, true),
        crossroads_house(222.0, false),
        crossroads_house(150.0, true),
    ]);
    common::run_to(&mut w, 2, 7.0);
    let sig = w.travel.signals[0].clone();
    let (west, east) = across_north(&w);
    let walker = 4u32;
    w.sims[walker as usize].pos = west;
    let mut buf = vec![0.0f32; snapshot::CAPACITY];
    // (Per car, its nose out along the arm it's coming in by, and the last tick that arm was green
    // as it came.)
    let mut was: Vec<Option<(usize, f32)>> = vec![None; w.cars.len()];
    let mut green_at: Vec<u64> = vec![0; w.cars.len()];
    let (mut crossed, mut waits, mut walks, mut across) = (0, 0, 0, 0);
    let end = w.tick + common::secs(1200.0);
    while w.tick < end {
        for c in 0..4 {
            match w.cars[c].state {
                CarState::Parked => set_off(&mut w, c),
                CarState::Away => come_home(&mut w, c),
                _ => {}
            }
        }
        if w.sims[walker as usize].current().is_none() {
            let p = w.sims[walker as usize].pos;
            let to = if (p[0] - west[0]).abs() < (p[0] - east[0]).abs() {
                east
            } else {
                across += 1;
                west
            };
            walk_to(&mut w, walker, to);
        }
        w.tick_once();
        snapshot::write(&w, &mut buf);
        let head = |k: usize| buf[LIGHT_BASE + k * LIGHT_STRIDE + light::STATE];
        if (0..4).any(|k| head(k) == light::WALK) {
            walks += 1;
        }
        for c in 0..w.cars.len() {
            let Some((p, yaw, _, _)) = w.cars[c].shown else {
                was[c] = None;
                continue;
            };
            let half = w.cars[c].len / 2.0;
            let nose = [p[0] + yaw.sin() * half, p[1] + yaw.cos() * half];
            let d = [nose[0] - sig.at[0], nose[1] - sig.at[1]];
            // (Coming in along an arm, in its lane, facing in.)
            let on = (0..sig.arms.len()).find_map(|k| {
                let a = sig.arms[k];
                let along = d[0] * a.way[0] + d[1] * a.way[1];
                let across = (d[0] * a.way[1] - d[1] * a.way[0]).abs();
                let inward = yaw.sin() * a.way[0] + yaw.cos() * a.way[1] < -0.7;
                (inward && across <= a.kerb && (0.0..30.0).contains(&along)).then_some((k, along))
            });
            if let Some((k, along)) = on {
                let stop = sig.arms[k].stop;
                if head(k) == light::GREEN {
                    green_at[c] = w.tick;
                }
                if let Some((j, before)) = was[c]
                    && j == k
                    && before >= stop - 0.05
                    && along < stop - 0.05
                {
                    crossed += 1;
                    assert_ne!(
                        head(k),
                        light::WALK,
                        "car {c} past the stop line on the walk: green {} ticks ago, {:?} {:?}",
                        w.tick - green_at[c],
                        w.cars[c].state,
                        sim_core::traffic::at_red(&w, sim_core::cars::car_key(c))
                    );
                    assert!(
                        head(k) != light::RED || w.tick < green_at[c] + common::drive_secs(6.0),
                        "car {c} past arm {k}'s stop line on red ({along:.2} m out)"
                    );
                }
                if sim_core::traffic::at_red(&w, sim_core::cars::car_key(c)) {
                    waits += 1;
                    assert!(
                        (stop - 0.1..stop + 0.5).contains(&along),
                        "car {c} waits at red with its nose {along:.2} m out, its stop line {stop}"
                    );
                }
            }
            was[c] = on;
        }
    }
    assert!(crossed >= 10, "{crossed} cars through");
    assert!(waits > 0, "nobody waited at a red light");
    assert!(walks > 0, "the walk never came round");
    assert!(across >= 2, "walked across {across} times");
}

#[test]
fn a_car_pulling_up_at_a_light_resting_on_another_arms_green_gets_its_turn_soon() {
    use sim_core::snapshot::{self, LIGHT_BASE, LIGHT_STRIDE, light};
    // The crossroads lit, nobody about: whichever arm rests on green. A car comes home along the
    // north arm's dead end: its arm's green within the 50 s (of drive time) every turn comes round in.
    let mut w = crossroads(&[crossroads_house(170.0, true)]);
    common::run_to(&mut w, 2, 7.0);
    // (Resting: a while with nobody at it.)
    for _ in 0..common::secs(20.0) {
        w.tick_once();
    }
    let mut buf = vec![0.0f32; snapshot::CAPACITY];
    snapshot::write(&w, &mut buf);
    let resting: Vec<f32> = (0..4)
        .map(|k| buf[LIGHT_BASE + k * LIGHT_STRIDE + light::STATE])
        .collect();
    set_off(&mut w, 0);
    let (start, mut through) = (w.tick, None);
    while w.tick < start + common::secs(3600.0) && through.is_none() {
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
        through - start < common::drive_secs(50.0) + common::drive_secs(180.0),
        "through after {} s, the lights resting at {resting:?}",
        (through - start) / common::drive_secs(1.0)
    );
}

// ---- Out of sight (`cars::sight`) ---------------------------------------------------------------

/// The crossroads (`crossroads`) with four houses up either dead-end arm, their cars always coming
/// and going (`keep_driving`): they hold each other up there by turns.
fn crowded_crossroads() -> World {
    crossroads(&[
        crossroads_house(150.0, true),
        crossroads_house(160.0, true),
        crossroads_house(170.0, true),
        crossroads_house(180.0, true),
        crossroads_house(220.0, false),
        crossroads_house(230.0, false),
        crossroads_house(240.0, false),
        crossroads_house(250.0, false),
    ])
}

/// The cars of `w` waiting for each other round (by their `cars::car_key`s): a ring in who waits
/// for whom (`traffic::waits_on`), else none.
fn ring(w: &World) -> Vec<u64> {
    use sim_core::traffic::waits_on;
    for c in 0..w.cars.len() {
        let (start, mut path) = (sim_core::cars::car_key(c), Vec::new());
        let mut at = start;
        while let Some(&next) = waits_on(w, at).iter().find(|&&k| k >= 1 << 63) {
            path.push(at);
            if next == start {
                return path;
            }
            if path.contains(&next) || path.len() > w.cars.len() {
                break;
            }
            at = next;
        }
    }
    Vec::new()
}

/// Every car's state as a test compares it: where along its way, where it's drawn, whether in
/// sight, whom it waits for.
fn cars_now(w: &World) -> Vec<String> {
    w.cars
        .iter()
        .map(|c| {
            format!(
                "{:?} {:.4} {:?} {:?} {:?} {:.4}",
                c.state, c.progress, c.shown, c.sight, c.waits_for, c.shift
            )
        })
        .collect()
}

/// A watch far off the crossroads: nothing near it is in sight.
const ELSEWHERE: [i32; 4] = [0, 0, 64, 64];

#[test]
fn cars_holding_each_other_up_out_of_sight_go_on_at_once() {
    // The crowded crossroads looked at (a watch over all of it) till cars there wait for each
    // other round a while; then the camera goes elsewhere.
    let mut w = crowded_crossroads();
    w.set_watch(Some([0, 0, 400, 400]));
    let mut held = 0;
    let mut cars = Vec::new();
    for _ in 0..common::secs(180.0) {
        keep_driving(&mut w);
        w.tick_once();
        let r = ring(&w);
        held = if r.is_empty() { 0 } else { held + 1 };
        if held >= common::drive_secs(0.5) {
            cars = r;
            break;
        }
    }
    assert!(
        !cars.is_empty(),
        "no ring of cars waiting for each other formed"
    );
    let at = |w: &World, k: u64| w.cars[(k & !(1 << 63)) as usize].progress;
    let from: Vec<f32> = cars.iter().map(|&k| at(&w, k)).collect();
    w.set_watch(Some(ELSEWHERE));
    // Out of sight they drive through each other: in a couple of seconds each has gone on
    // well along its way, and nobody waits for anyone.
    for t in 0..common::secs(2.0) {
        keep_driving(&mut w);
        w.tick_once();
        for c in 0..w.cars.len() {
            let car = &w.cars[c];
            if matches!(car.state, CarState::Leaving | CarState::Arriving) {
                assert_eq!(
                    car.sight,
                    sim_core::cars::Sight::Ghost,
                    "car {c} out of sight"
                );
                assert!(
                    sim_core::traffic::waits_on(&w, sim_core::cars::car_key(c)).is_empty(),
                    "car {c} waits out of sight, tick {t}"
                );
            }
        }
    }
    for (k, from) in cars.iter().zip(from) {
        let car = &w.cars[(k & !(1 << 63)) as usize];
        assert!(
            car.progress >= from + 30.0
                || !matches!(car.state, CarState::Leaving | CarState::Arriving),
            "car {k:x} went from {from} to {} only",
            car.progress
        );
    }
}

#[test]
fn a_car_coming_into_sight_never_overlaps_one_there() {
    // Two cars coming home by the same way at the same tick, the way in out of sight: there they
    // drive in on top of each other (out of sight nobody waits for anyone).
    let mut w = street(7.0, &[(80.0, true), (90.0, true)]);
    let watch = [70, 0, 200, 60];
    w.set_watch(Some(watch));
    come_home(&mut w, 0);
    come_home(&mut w, 1);
    let mut together = false;
    let mut waited = false;
    for _ in 0..common::secs(200.0) {
        w.tick_once();
        let (a, b) = (w.cars[0].shown, w.cars[1].shown);
        together |= a.zip(b).is_some_and(|(a, b)| a.0 == b.0 && a.2);
        waited |= w
            .cars
            .iter()
            .any(|c| matches!(c.sight, sim_core::cars::Sight::Wait(_)));
        // Never on top of each other where either is drawn, nor both in sight.
        let drawn = common::drawn(&w, Some(watch));
        let (o, a, b) = overlap_where(&w, |c| drawn[c]);
        assert!(
            o <= 0.0,
            "cars {a} and {b} overlap by {o:.2} m where they're drawn"
        );
        let (o, a, b) = overlap_where(&w, |c| w.cars[c].sight == sim_core::cars::Sight::Seen);
        let both = w.cars[a].sight == sim_core::cars::Sight::Seen
            && w.cars[b].sight == sim_core::cars::Sight::Seen;
        assert!(
            o <= 0.0 || !both,
            "cars {a} and {b}, both in sight, overlap by {o:.2} m"
        );
        if w.cars.iter().all(|c| c.state == CarState::Parked) {
            break;
        }
    }
    assert!(together, "they came in on top of each other out of sight");
    assert!(
        waited,
        "one waited outside the seen area for its way in to clear"
    );
    assert!(
        w.cars.iter().all(|c| c.state == CarState::Parked),
        "both home"
    );
}

#[test]
fn the_camera_coming_to_two_cars_on_top_of_each_other_shows_them_apart() {
    // Two cars coming home by the same way at the same tick, out of sight: on top of each other.
    let mut w = street(7.0, &[(60.0, true), (70.0, true)]);
    w.set_watch(Some([190, 0, 200, 60]));
    come_home(&mut w, 0);
    come_home(&mut w, 1);
    let together = |w: &World| {
        w.cars[0]
            .shown
            .zip(w.cars[1].shown)
            .is_some_and(|(a, b)| a.0 == b.0 && a.2)
    };
    for _ in 0..common::secs(10.0) {
        w.tick_once();
        if together(&w) && w.cars[0].shown.is_some_and(|s| s.0[0] > 20.0) {
            break;
        }
    }
    assert!(
        together(&w),
        "they drove in on top of each other out of sight"
    );
    // The camera comes to them: the next picture has both, apart.
    w.set_watch(Some([0, 0, 200, 60]));
    w.tick_once();
    let hidden = sim_core::cars::hidden(&w.cars);
    assert!(!hidden[0] && !hidden[1], "both drawn: {hidden:?}");
    assert!(
        w.cars
            .iter()
            .all(|c| c.sight == sim_core::cars::Sight::Seen),
        "both in sight"
    );
    let (o, a, b) = cars_overlap(&w);
    assert!(o <= -0.15, "cars {a} and {b} {:.2} m apart at most", -o);
    let done = run_clear(
        &mut w,
        common::secs(200.0),
        |_| {},
        |w| w.cars.iter().all(|c| c.state == CarState::Parked),
    );
    assert!(done, "both home");
}

#[test]
fn in_sight_the_cars_go_as_without_a_watch() {
    // The crowded crossroads with a watch over all of it, and with none: tick for tick the same.
    let mut a = crowded_crossroads();
    let mut b = crowded_crossroads();
    b.set_watch(Some([0, 0, 400, 400]));
    for t in 0..common::secs(120.0) {
        keep_driving(&mut a);
        keep_driving(&mut b);
        a.tick_once();
        b.tick_once();
        assert_eq!(cars_now(&a), cars_now(&b), "tick {t}");
    }
}

#[test]
fn with_the_same_watches_the_cars_go_the_same_way_and_none_in_sight_waits_for_one_out_of_it() {
    // The camera moving round the crowded crossroads (and off it, and the watch cleared), the same
    // way twice: tick for tick the same. Nobody waits for a car out of sight, and no two cars
    // overlap where one of them is drawn.
    let watches: [Option<[i32; 4]>; 6] = [
        Some([0, 0, 400, 400]),
        Some([150, 150, 250, 250]),
        Some(ELSEWHERE),
        None,
        Some([192, 100, 256, 192]),
        Some([136, 192, 200, 300]),
    ];
    let run = || {
        let mut w = crowded_crossroads();
        let mut states = Vec::new();
        for t in 0..common::secs(180.0) {
            let watch = watches[(t / common::drive_secs(7.5)) as usize % watches.len()];
            w.set_watch(watch);
            keep_driving(&mut w);
            w.tick_once();
            for c in 0..w.cars.len() {
                for k in sim_core::traffic::waits_on(&w, sim_core::cars::car_key(c)) {
                    let other = w
                        .cars
                        .get((k & !(1 << 63)) as usize)
                        .filter(|_| k >= 1 << 63);
                    assert!(
                        other.is_none_or(|o| o.sight == sim_core::cars::Sight::Seen),
                        "car {c} waits for car {k:x}, out of sight, tick {t}"
                    );
                }
            }
            let drawn = common::drawn(&w, watch);
            let (o, a, b) = overlap_where(&w, |c| drawn[c]);
            assert!(
                o <= 0.05,
                "cars {a} and {b} overlap by {o:.2} m where drawn, tick {t}"
            );
            states.push(cars_now(&w));
        }
        states
    };
    let (first, second) = (run(), run());
    assert!(first == second, "the two runs went differently");
}

// ---- Held up in sight: turning round (`cars::unjam`) -------------------------------------------

#[test]
fn a_car_held_up_for_good_in_sight_backs_out_of_the_junction_and_goes_another_way() {
    // A car off to work from up the crossroads' north arm; past the junction, a neighbour's car is left
    // across the arm it took, never to be driven: held up behind it a few minutes, it backs up
    // out of the junction it came through (into clear room, never into anyone) and goes out of
    // town the other way.
    let mut w = crossroads(&[
        crossroads_house(150.0, true),
        crossroads_house(250.0, false),
        crossroads_house(260.0, false),
    ]);
    set_off(&mut w, 0);
    let mut past = None;
    for _ in 0..common::secs(150.0) {
        w.tick_once();
        if let Some((p, yaw, true, _)) = w.cars[0].shown
            && (p[0] - 200.0).abs() > 12.0
            && (p[1] - 200.0).abs() < 4.0
        {
            past = Some((p, yaw));
            break;
        }
    }
    let (p, yaw) = past.unwrap_or_else(|| {
        panic!(
            "it drove out past the junction: {:?} {:?} prog {} waits {:?} unjams {}",
            w.cars[0].state,
            w.cars[0].shown,
            w.cars[0].progress,
            sim_core::traffic::waits_on(&w, sim_core::cars::car_key(0)),
            w.traffic.unjams
        )
    });
    // (Across the arm 15 m on, never driven: another house's car, parked there.)
    let way = [yaw.sin(), yaw.cos()];
    w.cars[1].spot = [p[0] + way[0] * 15.0, 200.0];
    w.cars[1].yaw = 0.0;
    let went = run_clear(
        &mut w,
        common::secs(1200.0),
        |_| {},
        |w| w.cars[0].state == CarState::Away,
    );
    assert!(w.traffic.unjams >= 1, "it never turned round");
    assert!(went, "it never got out of town: {:?}", w.cars[0].shown);
    // (Out the other way: it backed out of the arm it was held up on.)
    assert!(
        w.cars[0].detour.is_none() || w.cars[0].state == CarState::Away,
        "still turned round"
    );
}

#[test]
fn a_car_held_up_with_no_room_behind_it_to_turn_round_is_counted_and_never_backs_into_anyone() {
    // As above, but another car right behind it, the way it came, standing (left there, never
    // driven): it can't back up, and nobody turns round; the ring is counted, with why.
    let mut w = crossroads(&[
        crossroads_house(150.0, true),
        crossroads_house(250.0, false),
        crossroads_house(260.0, false),
    ]);
    set_off(&mut w, 0);
    let mut past = None;
    for _ in 0..common::secs(150.0) {
        w.tick_once();
        if let Some((p, yaw, true, _)) = w.cars[0].shown
            && (p[0] - 200.0).abs() > 12.0
            && (p[1] - 200.0).abs() < 4.0
        {
            past = Some((p, yaw));
            break;
        }
    }
    let (p, yaw) = past.expect("it drove out past the junction");
    let way = [yaw.sin(), yaw.cos()];
    w.cars[1].spot = [p[0] + way[0] * 15.0, 200.0];
    w.cars[1].yaw = 0.0;
    // (Right behind it, across its lane.)
    w.cars[2].spot = [p[0] - way[0] * 4.0, 200.0];
    w.cars[2].yaw = 0.0;
    let went = run_clear(
        &mut w,
        common::secs(1200.0),
        |_| {},
        |w| w.cars[0].state == CarState::Away,
    );
    assert!(!went, "it got by");
    assert_eq!(w.traffic.unjams, 0, "it turned round with no room");
    assert!(
        w.traffic.unjam_impossible.values().sum::<u64>() >= 1,
        "the ring wasn't counted"
    );
}
