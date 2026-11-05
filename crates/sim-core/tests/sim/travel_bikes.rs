//! Riding a bike to work and back (`ride.rs`, content `vehicle`): someone with a bike at home
//! rides it out of town at its speed, it goes to work with them and comes home with them, back
//! at its place; a way too short for it is walked; two can't share one bike. In an open-world
//! town the way to work is timed as it's ridden or walked (`life::Commute`): someone too near the
//! way out for their bike, or whose housemate has the one bike, leaves at their walking time.

use crate::common;

use sim_core::snapshot;
use sim_core::world::TaskKind;
use sim_core::{MINUTES_PER_TICK, World, clock, life};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],
    "objects":[
        {"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"effects":{"hunger":0.01},"tags":["lounge"]}]},
        {"id":"bike","name":"Bike","footprint":[1,1],"price":200,"interactions":[],"vehicle":{"kind":"bike","speed":3,"minRide":30}}],
    "careers":[{"id":"office","label":"Office","levels":[
        {"title":"Clerk","pay":100,"start":10,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
    "careerRules":{"commuteMinutes":60,"performancePerShift":10},
    "economy":{"startingFunds":500}}"#;

/// A long street with the way out at its west end and a house at the far east end, `workers`
/// living there with a job and `bikes` bikes in the garden.
fn town(workers: usize, bikes: usize, house_x: i32) -> World {
    let sims: Vec<String> = (0..workers)
        .map(|k| {
            format!(
                r#"{{"name":"W{k}","household":0,"job":{{"career":"office"}},"x":{}.5,"z":3.5}}"#,
                house_x + 4 + k as i32
            )
        })
        .collect();
    let mut objects = vec![format!(
        r#"{{"def":"chair","x":{},"z":2,"rot":0}}"#,
        house_x + 2
    )];
    for b in 0..bikes {
        objects.push(format!(
            r#"{{"def":"bike","x":{},"z":6,"rot":0}}"#,
            house_x + 10 + b as i32 * 2
        ));
    }
    let json = format!(
        r#"{{"width":{w},"depth":14,"exits":[[0.5,10.5]],
        "plots":[{{"name":"Home","x":{house_x},"z":0,"w":20,"d":9,"entry":[{e}.5,8.5]}}],
        "households":[{{"name":"Riders","plot":0,"player":true}}],
        "objects":[{objects}],
        "sims":[{sims}]}}"#,
        w = house_x + 20,
        e = house_x + 10,
        objects = objects.join(","),
        sims = sims.join(","),
    );
    let mut w = World::from_json(CONTENT, &json, 5).unwrap();
    w.autonomy = false;
    w
}

/// Ticks until resident `i` is away at work (None: not by `limit`).
fn until_away(w: &mut World, i: usize, limit: u64, mut each: impl FnMut(&World)) -> Option<u64> {
    let end = w.tick + limit;
    while w.tick < end {
        w.tick_once();
        each(w);
        if w.sims[i].away_until.is_some() {
            return Some(w.tick);
        }
    }
    None
}

fn bike(w: &World) -> usize {
    w.objects
        .iter()
        .position(|o| w.content.objects[o.def].id == "bike")
        .unwrap()
}

#[test]
fn someone_with_a_bike_rides_it_to_work_and_home_again() {
    let mut w = town(1, 1, 100);
    let b = bike(&w);
    let stable = w.objects[b].stable as f32;
    let mut rode = false;
    let mut shown = false;
    until_away(&mut w, 0, clock::tick_at(2, 12.0 * 60.0).unwrap(), |w| {
        if w.sims[0].riding == Some(b as u32) {
            rode = true;
            let mut out = vec![0.0; snapshot::CAPACITY];
            snapshot::write(w, &mut out);
            shown |= out[snapshot::HEADER_LEN + snapshot::sim::RIDE] == stable;
        }
    })
    .expect("never got to work");
    assert!(rode, "walked instead of riding");
    assert!(shown, "the snapshot never showed them on it");
    // It went to work with them: still theirs, nobody else's.
    assert_eq!(w.sims[0].riding, Some(b as u32));
    assert_eq!(w.objects[b].users[0], Some(0));
    // Home again after the shift: off it, at its place, free.
    let mut home_at = None;
    let end = w.tick + 10 * common::TICKS_PER_HOUR;
    while w.tick < end {
        w.tick_once();
        if w.sims[0].away_until.is_none() && w.sims[0].riding.is_none() {
            home_at = Some(w.tick);
            break;
        }
    }
    assert!(home_at.is_some(), "never got off");
    let (x, z) = w.sims[0].tile();
    assert_eq!(w.plot_at(x, z), Some(0), "got off away from home");
    assert!(
        w.objects[b].users.iter().all(Option::is_none),
        "the bike stayed taken"
    );
}

#[test]
fn a_bike_gets_them_out_of_town_in_a_third_of_the_walk() {
    let time_to_exit = |bikes: usize| {
        let mut w = town(1, bikes, 100);
        common::run_to(&mut w, 1, 8.0);
        let mut set_off = None;
        let away = until_away(&mut w, 0, clock::tick_at(2, 12.0 * 60.0).unwrap(), |w| {
            if set_off.is_none() && w.sims[0].current().is_some() {
                set_off = Some(w.tick);
            }
        })
        .expect("never got to work");
        away - set_off.unwrap()
    };
    let (walked, rode) = (time_to_exit(0), time_to_exit(1));
    assert!(
        (rode as f32) < walked as f32 * 0.5,
        "rode in {rode} ticks, walked in {walked}"
    );
}

#[test]
fn a_way_too_short_for_the_bike_is_walked() {
    // The house at the way out: under the bike's `minRide`.
    let mut w = town(1, 1, 2);
    until_away(&mut w, 0, clock::tick_at(2, 12.0 * 60.0).unwrap(), |w| {
        assert!(w.sims[0].riding.is_none(), "rode a few steps");
    })
    .expect("never got to work");
}

#[test]
fn two_off_to_work_with_one_bike_one_rides_and_one_walks() {
    let mut w = town(2, 1, 100);
    let mut riders = [false; 2];
    let end = clock::tick_at(2, 12.0 * 60.0).unwrap();
    while w.tick < end && !(w.sims[0].away_until.is_some() && w.sims[1].away_until.is_some()) {
        w.tick_once();
        assert!(
            !(w.sims[0].riding.is_some() && w.sims[0].riding == w.sims[1].riding),
            "both on one bike"
        );
        for (k, r) in riders.iter_mut().enumerate() {
            *r |= w.sims[k].riding.is_some();
        }
    }
    assert!(
        w.sims.iter().all(|s| s.away_until.is_some()),
        "not both at work"
    );
    assert_eq!(riders.iter().filter(|&&r| r).count(), 1, "{riders:?}");
}

#[test]
fn a_game_saved_mid_ride_loads_with_the_bike_at_home_and_them_on_foot() {
    let mut w = town(1, 1, 100);
    let b = bike(&w);
    while w.sims[0].riding.is_none() {
        w.tick_once();
        assert!(
            w.tick < clock::tick_at(2, 12.0 * 60.0).unwrap(),
            "never rode"
        );
    }
    let mut loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    loaded.autonomy = false;
    assert!(loaded.sims[0].riding.is_none());
    assert!(loaded.objects[b].users.iter().all(Option::is_none));
    until_away(&mut loaded, 0, 30 * common::TICKS_PER_HOUR, |_| {})
        .expect("never got to work after loading");
}

#[test]
fn a_bike_sold_while_they_ride_it_leaves_them_on_foot() {
    let mut w = town(1, 1, 100);
    let b = bike(&w);
    while w.sims[0].riding.is_none() {
        w.tick_once();
        assert!(
            w.tick < clock::tick_at(2, 12.0 * 60.0).unwrap(),
            "never rode"
        );
    }
    let stable = w.objects[b].stable;
    w.apply(sim_core::Command::Sell {
        household: 0,
        object: stable,
    })
    .unwrap();
    assert!(w.sims[0].riding.is_none());
    until_away(&mut w, 0, 30 * common::TICKS_PER_HOUR, |_| {}).expect("never got to work on foot");
}

/// One morning off to work or school for each of `w`'s residents (from day 2, the first full
/// one): when they set off, when they were out of town, whether they rode, and when they were
/// due at the way out (`commuteMinutes` before the start: the rest of the way is off the map).
#[derive(Debug, Default, Clone, Copy)]
struct Morning {
    left: Option<u64>,
    out: Option<u64>,
    rode: bool,
    due: u64,
}

fn mornings(w: &mut World) -> Vec<Morning> {
    common::run_to(w, 2, 0.0);
    let off_map = (w.content.career_rules.commute_minutes / MINUTES_PER_TICK) as u64;
    let mut days = vec![Morning::default(); w.sims.len()];
    let end = clock::tick_at(2, 14.0 * 60.0).unwrap();
    while w.tick < end && days.iter().any(|d| d.out.is_none()) {
        w.tick_once();
        for (i, d) in days.iter_mut().enumerate() {
            let s = &w.sims[i];
            if d.left.is_none()
                && s.current()
                    .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
            {
                d.left = Some(w.tick);
            }
            d.rode |= s.riding.is_some();
            if d.out.is_none() && s.away_until.is_some() {
                d.out = Some(w.tick);
                let start = match &s.job {
                    Some(job) => life::shift_today(&w.content, job, &s.skills, w.tick),
                    None => life::school_today(&w.content, s, w.tick),
                };
                d.due = start.expect("work or school today").1 - off_map;
            }
        }
    }
    assert!(days.iter().all(|d| d.out.is_some()), "{days:?}");
    days
}

/// Whether `a` and `b` set off within a minute of each other.
fn same_time(a: Morning, b: Morning) -> bool {
    let minute = (1.0 / MINUTES_PER_TICK) as u64;
    a.left.unwrap().abs_diff(b.left.unwrap()) <= minute
}

/// An open-world town on `content`: a garden walled along its west and south sides, open to the
/// street at its east end, and the way out of town west along the street, a long way round from
/// the garden (some 80 m on foot from its front door at the west end, under 40 m in a straight
/// line). Who lives there (`sims`) and what's in the garden (`objects`), as JSON.
fn walled_town(content: &str, sims: &str, objects: &str) -> World {
    let json = format!(
        r#"{{"width":64,"depth":14,"size":"normal","exits":[[0.5,10.5]],
        "walls":[[31,0,31,9],[31,9,56,9]],
        "plots":[{{"name":"Home","x":30,"z":0,"w":30,"d":9,"entry":[34.5,5.5]}}],
        "households":[{{"name":"Home","plot":0,"player":true}}],
        "objects":[{objects}],
        "sims":[{sims}]}}"#
    );
    let mut w = World::from_json(content, &json, 5).unwrap();
    w.autonomy = false;
    w
}

/// A child off to school in `walled_town` on the shipped content, nearer the way out in a straight
/// line than their kids' bike's `minRide` (60 m); with the bike at home or not.
fn child_by_the_way_out(bike: bool) -> World {
    let kid = r#"{"name":"Kid","household":0,"age":8,"traits":["cheerful"],"x":34.5,"z":3.5}"#;
    let bike = if bike {
        r#"{"def":"bicycleChild","x":33,"z":5,"rot":0}"#
    } else {
        ""
    };
    walled_town(&common::content(), kid, bike)
}

/// Two off to work in `walled_town`, with `bikes` bikes (`CONTENT`'s, a `minRide` of 30 m) at home.
fn two_by_the_way_out(bikes: usize) -> World {
    let sims = r#"{"name":"W0","household":0,"job":{"career":"office"},"x":34.5,"z":3.5},
        {"name":"W1","household":0,"job":{"career":"office"},"x":35.5,"z":3.5}"#;
    let bike = if bikes > 0 {
        r#"{"def":"bike","x":40,"z":5,"rot":0}"#
    } else {
        ""
    };
    walled_town(CONTENT, sims, bike)
}

#[test]
fn a_child_too_near_the_way_out_for_their_bike_leaves_at_their_walking_time() {
    // (Timed by the bike, they'd set off too late for the walk it turns out to be.)
    let walked = mornings(&mut child_by_the_way_out(false))[0];
    let with_bike = mornings(&mut child_by_the_way_out(true))[0];
    assert!(!with_bike.rode, "rode a few steps");
    assert!(
        same_time(walked, with_bike),
        "left at {:.2} with the bike at home, {:.2} without",
        clock::hour(with_bike.left.unwrap()),
        clock::hour(walked.left.unwrap())
    );
    assert!(
        with_bike.out.unwrap() <= with_bike.due,
        "out of town at {:.2}, due at {:.2}",
        clock::hour(with_bike.out.unwrap()),
        clock::hour(with_bike.due)
    );
}

#[test]
fn of_two_with_one_bike_the_one_without_it_leaves_at_their_walking_time() {
    // (The first of the household has it: the other's way is timed on foot, as it's walked.)
    let walking = mornings(&mut two_by_the_way_out(0));
    let one_bike = mornings(&mut two_by_the_way_out(1));
    assert!(one_bike[0].rode && !one_bike[1].rode, "{one_bike:?}");
    assert!(
        same_time(walking[1], one_bike[1]),
        "left at {:.2} with a housemate on the bike, {:.2} with no bike",
        clock::hour(one_bike[1].left.unwrap()),
        clock::hour(walking[1].left.unwrap())
    );
    for (k, m) in one_bike.iter().enumerate() {
        assert!(
            m.out.unwrap() <= m.due,
            "W{k} out of town at {:.2}, due at {:.2}",
            clock::hour(m.out.unwrap()),
            clock::hour(m.due)
        );
    }
}

// ---- Keeping clear of cars ---------------------------------------------------------------

/// A city bike as the shipped one reaches (content `vehicle.reach`): its front wheel, its back
/// wheel and its handlebars.
const REACH: [f32; 3] = [1.2, 0.6, 0.3];

/// `CONTENT` with the bike as long and wide as a city bike.
fn lane_content() -> String {
    CONTENT.replace(
        r#""minRide":30}"#,
        &format!(r#""minRide":30,"reach":{REACH:?}}}"#),
    )
}

/// The middle of the lane (z): 6 m wide, along x, the way out at its east end.
const LANE: f32 = 30.0;

/// A small open-world town on a 6 m lane: a rider's house (`R`, a job and a bike in the garden) at
/// `rider_x`, and south of the lane at `car_x` a house whose car (driven by `D`) is parked nose
/// out on its driveway.
fn lane_town(rider_x: i32, car_x: i32) -> World {
    let road: Vec<String> = (0..=499).map(|x| format!("[{x}.5,{LANE}]")).collect();
    let road = format!(
        r#"{{"points":[{}],"width":6,"kind":"lane","ends":["dead","out"]}}"#,
        road.join(",")
    );
    let (cx, inner) = (car_x as f32 + 1.5, 39.5);
    let json = format!(
        r#"{{"width":500,"depth":60,"size":"normal",
        "exits":[{{"x":499.5,"z":{LANE},"kind":"road"}}],
        "roads":[{road}],"meta":{{"roads":[{road}]}},
        "plots":[{{"name":"Riders","x":{rider_x},"z":34,"w":20,"d":16,"entry":[{re}.5,34.5]}},
                 {{"name":"Drivers","x":{car_x},"z":34,"w":20,"d":16,"entry":[{ce}.5,34.5]}}],
        "households":[{{"name":"Riders","plot":0,"player":true}},{{"name":"Drivers","plot":1}}],
        "objects":[{{"def":"bike","x":{bx},"z":40,"rot":0}}],
        "sims":[{{"name":"R","household":0,"job":{{"career":"office"}},"x":{rs}.5,"z":44.5}},
                {{"name":"D","household":1,"x":{cs}.5,"z":44.5}}],
        "drives":[{{"plot":1,"x":{cx},"z":{inner},"front":-1,"street":{LANE}}}],
        "cars":[{{"plot":1,"x":{cx},"z":{spot},"yaw":{yaw},"street":{LANE},"len":4.4,"wid":1.8,"body":1,"paint":2}}]}}"#,
        re = rider_x + 10,
        ce = car_x + 10,
        bx = rider_x + 12,
        rs = rider_x + 5,
        cs = car_x + 10,
        spot = inner - 0.45 - 2.2,
        yaw = std::f32::consts::PI,
    );
    let mut w = World::from_json(&lane_content(), &json, 5).unwrap();
    w.autonomy = false;
    w
}

/// D drives off now (at work as far as anyone can tell).
fn drive_off(w: &mut World) {
    let car = &mut w.cars[0];
    car.state = sim_core::cars::CarState::Leaving;
    car.since = w.tick;
    car.progress = 0.0;
    car.driver = Some(1);
    car.aboard = vec![1];
    w.sims[1].away_until = Some(w.tick + 100_000);
}

/// How far R's bike (its wheels and handlebars, facing the way R does) is from the car's body,
/// where it is now: below 0, how far into it. None: the car isn't on the move, or R isn't riding.
fn bike_from_car(w: &World) -> Option<f32> {
    let (at, yaw, true, _) = w.cars[0].shown? else {
        return None;
    };
    let s = &w.sims[0];
    s.riding?;
    let (len, wid) = (w.cars[0].len / 2.0, w.cars[0].wid / 2.0);
    let [ahead, behind, side] = REACH;
    let (fs, fc) = s.yaw.sin_cos();
    let (cs, cc) = yaw.sin_cos();
    // (Points along the bike's middle, each as far out as its handlebars.)
    let gap = (0..=20)
        .map(|k| {
            let t = -(behind - side) + (ahead + behind - 2.0 * side) * k as f32 / 20.0;
            let (dx, dz) = (s.pos[0] + fs * t - at[0], s.pos[1] + fc * t - at[1]);
            let (along, across) = (dx * cs + dz * cc, dx * cc - dz * cs);
            let (ex, ez) = (along.abs() - len, across.abs() - wid);
            ex.max(0.0).hypot(ez.max(0.0)) + ex.max(ez).min(0.0) - side
        })
        .fold(f32::INFINITY, f32::min);
    Some(gap)
}

#[test]
fn a_rider_on_a_narrow_lane_keeps_clear_of_a_car_passing_them() {
    // R rides east along the lane to the way out; D's car, west of them, overtakes.
    let mut w = lane_town(140, 110);
    let end = clock::tick_at(2, 12.0 * 60.0).unwrap();
    while !(w.sims[0].riding.is_some() && w.sims[0].pos[1] < 34.0) {
        w.tick_once();
        assert!(w.tick < end, "never rode out onto the lane");
    }
    drive_off(&mut w);
    let (mut closest, mut passed) = (f32::INFINITY, false);
    while w.sims[0].away_until.is_none() && w.tick < end {
        w.tick_once();
        if let Some(gap) = bike_from_car(&w) {
            closest = closest.min(gap);
            passed |= w.cars[0].shown.unwrap().0[0] > w.sims[0].pos[0] + 5.0;
        }
    }
    assert!(passed, "the car never overtook them");
    // Their handlebars clear its side by a hand's width at least.
    assert!(
        closest >= 0.1,
        "the bike came within {closest:.2} m of the car (into it below 0)"
    );
}

#[test]
fn a_rider_heading_at_a_cars_side_stops_with_the_front_wheel_outside_it() {
    // R rides east along the lane past D's driveway, east of them, as the car pulls out of it
    // across the pavement in front of them.
    let mut w = lane_town(140, 200);
    let end = clock::tick_at(2, 12.0 * 60.0).unwrap();
    let car_x = w.cars[0].spot[0];
    while !(w.sims[0].riding.is_some()
        && w.sims[0].pos[1] < 34.0
        && w.sims[0].pos[0] > car_x - 13.0)
    {
        w.tick_once();
        assert!(w.tick < end, "never rode along the lane");
    }
    drive_off(&mut w);
    let (mut closest, mut stopped) = (f32::INFINITY, false);
    while w.sims[0].away_until.is_none() && w.tick < end {
        let was = w.sims[0].pos;
        w.tick_once();
        if let Some(gap) = bike_from_car(&w) {
            closest = closest.min(gap);
            // (Held up by it, up against its side.)
            stopped |= gap < 0.6 && w.sims[0].pos == was;
        }
    }
    assert!(
        stopped,
        "never waited at the car's side (closest {closest:.2} m)"
    );
    assert!(
        closest >= 0.0,
        "the front wheel went {:.2} m into the car",
        -closest
    );
}

/// Holly Clearing, a generated Normal forest town (`bun tools/worlds/sample.ts`, seed 209464, as
/// the 0.7.0 samples had it): at 8:36 a car turns in tight from the lane to the narrow road to
/// the dead end, its tail swinging round, as four riders come out of it. The car waited for those
/// beside it its turn would come nearer to, they for it (they keep clear of where it would be a
/// tick on too), and nobody went anywhere for hours.
#[test]
fn a_car_turning_tight_among_riders_beside_it_turns_and_they_ride_on() {
    let content = common::content();
    let town = include_str!("../towns/holly_clearing.json");
    let mut w = World::from_json(&content, town, 47).unwrap();
    let end = w.tick + 3 * 60 * 20;
    let mut riding = vec![0u64; w.sims.len()];
    let mut still: Vec<(Option<[f32; 2]>, u64)> = vec![(None, 0); w.cars.len()];
    while w.tick < end {
        w.tick_once();
        riding.resize(w.sims.len(), 0);
        for (i, s) in w.sims.iter().enumerate() {
            riding[i] = if s.riding.is_some() && s.away_until.is_none() {
                riding[i] + 1
            } else {
                0
            };
            assert!(
                riding[i] < 60 * 20,
                "{} on a bike in town for an hour, at {:?}",
                s.name,
                s.pos
            );
        }
        for (c, car) in w.cars.iter().enumerate() {
            let at = car.shown.filter(|s| s.2).map(|s| s.0);
            still[c] = match (at, still[c]) {
                (Some(p), (Some(q), n)) if p == q => (Some(p), n + 1),
                (p, _) => (p, 0),
            };
            assert!(
                still[c].1 < 30 * 20,
                "car {c} stood still half an hour on its way, at {at:?}"
            );
        }
    }
}
