use super::*;

/// A lane that goes a step past a road's end and back before it turns off (where one road's end
/// meets another) keeps the points it came up by: rounding the corner takes out only those
/// about it. (A work taxi's lane in Applewold, seed base 19: its corner's way in was the step
/// back, and the 130 m of road before it went, the taxi driving straight across the grass.)
#[test]
fn a_corner_after_a_step_back_keeps_the_road_before_it() {
    let lane: Vec<[f32; 2]> = vec![
        [497.8137, 250.62184],
        [497.54294, 253.87134],
        [497.3321, 257.02386],
        [497.16132, 260.057],
        [496.99088, 263.9974],
        [496.8605, 267.95804],
        [496.76022, 271.9087],
        [496.70004, 275.87933],
        [496.67, 279.84],
        [496.68005, 283.81058],
        [496.73013, 287.7659],
        [496.81033, 291.7365],
        [496.92065, 295.70712],
        [497.07108, 299.6677],
        [497.26132, 303.62296],
        [497.48193, 307.5835],
        [497.74265, 311.53442],
        [498.03302, 315.4797],
        [498.36392, 319.42072],
        [498.73563, 323.38858],
        [499.08685, 326.48978],
        [499.4788, 329.6956],
        [499.92017, 332.97583],
        [500.40112, 336.3224],
        [500.92294, 339.71445],
        [501.46356, 343.12817],
        [502.02515, 346.5478],
        [502.60544, 349.93933],
        [503.1968, 353.297],
        [503.7874, 356.59033],
        [504.37732, 359.80997],
        [504.94775, 362.9223],
        [505.50842, 365.92575],
        [506.22787, 369.70285],
        [506.879, 373.18875],
        [507.59717, 377.03912],
        [508.2756, 380.8303],
        [508.36566, 381.36075],
        [510.51163, 381.43753],
        [509.77502, 378.2679],
        [507.91922, 377.908],
        [504.92474, 379.23447],
        [501.34598, 380.8985],
        [499.398, 381.86713],
        [497.2816, 381.88196],
        [498.27985, 382.7819],
        [498.2299, 379.45593],
        [498.17975, 376.0669],
        [498.09973, 372.6471],
        [498.01965, 369.19263],
        [497.91953, 365.71832],
        [497.8094, 362.2338],
        [497.6894, 358.7538],
        [497.55923, 355.2793],
        [497.4192, 351.8189],
        [497.279, 348.38397],
        [497.12897, 344.98358],
        [496.97894, 341.62277],
        [496.82867, 338.31708],
        [496.66885, 335.0712],
        [496.50854, 331.89508],
        [496.35873, 328.79837],
        [496.1586, 324.8062],
        [495.9585, 320.98364],
        [495.76865, 317.3664],
        [495.59875, 313.96884],
        [495.39883, 310.06036],
        [495.2389, 306.5716],
        [495.08926, 303.00076],
        [495.0692, 302.46964],
    ];
    let out = round_corners(&lane);
    let gap = out
        .windows(2)
        .map(|w| crate::travel::dist(w[0], w[1]))
        .fold(0.0f32, f32::max);
    assert!(gap < 8.0, "a step of {gap} m");
}

/// A body standing on the outside of a bend to the left reaches past the lane by the chord's
/// sagitta (len² / 8R); on a straight, or on the inside of a bend to the right, by none.
#[test]
fn bend_out_is_the_sagitta_on_the_outside_of_a_left_bend() {
    let (r, len) = (40.0f32, 4.7f32);
    // An arc round (0, 0) of radius `r`, a point every degree; `turn` +1 bends to the left of
    // the way it runs (forward (sin, cos), right (-cos, sin): `shifted`).
    let arc = |turn: f32| -> Vec<[f32; 2]> {
        (0..=40)
            .map(|k| {
                let a = (k as f32).to_radians();
                [-r * a.cos() * turn, r * a.sin()]
            })
            .collect()
    };
    let want = len * len / (8.0 * r);
    let left = bend_out(&arc(1.0), 20, len);
    assert!((left - want).abs() < 0.01, "{left} against {want}");
    assert_eq!(bend_out(&arc(-1.0), 20, len), 0.0);
    let straight: Vec<[f32; 2]> = (0..20).map(|k| [0.0, k as f32]).collect();
    assert!(bend_out(&straight, 10, len) < 1e-5);
}
use crate::clock;

const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
        "objects":[{"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"tags":["lounge"]}]}],
        "careers":[{"id":"office","label":"Office","workEffects":{"hunger":-0.2},"levels":[
            {"title":"Intern","pay":100,"start":9,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
        "economy":{"startingFunds":500}}"#;

/// A house on a street running along x (its middle at z = 9.5), the exits at both ends; the
/// car parked beside the house facing the street. Two grown-ups at the same office.
const TOWN: &str = r#"{"width":60,"depth":14,"exits":[[0.5,9.5],[59.5,9.5]],
        "plots":[{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}],
        "households":[{"name":"Ones","plot":0}],
        "sims":[{"name":"Ada","household":0,"job":{"career":"office"},"x":25.5,"z":3.5},
                {"name":"Bo","household":0,"job":{"career":"office"},"x":26.5,"z":3.5}],
        "cars":[{"plot":0,"x":34.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":2}]}"#;

fn until(w: &mut World, day: u32, hour: f32) {
    let target = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < target {
        w.tick_once();
    }
}

/// Ada's house with her car, and Cy's next door without one: both at the same office.
fn neighbours(bond: &str) -> World {
    let content = CONTENT.replace(
        r#""economy""#,
        r#""bondPresets":{"friends":{"friendship":60},"strangers":{"friendship":0}},"economy""#,
    );
    let town = format!(
        r#"{{"width":60,"depth":14,"exits":[[0.5,9.5],[59.5,9.5]],
            "plots":[{{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}},
                     {{"name":"Next","x":40,"z":0,"w":16,"d":8,"entry":[45.5,7.5]}}],
            "households":[{{"name":"Ones","plot":0}},{{"name":"Twos","plot":1}}],
            "sims":[{{"name":"Ada","household":0,"job":{{"career":"office"}},"x":25.5,"z":3.5}},
                    {{"name":"Cy","household":1,"job":{{"career":"office"}},"x":45.5,"z":3.5}}],
            "relationships":[{{"a":0,"b":1,"preset":"{bond}"}}],
            "cars":[{{"plot":0,"x":34.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":2}}]}}"#
    );
    World::from_json(&content, &town, 1).unwrap()
}

#[test]
fn a_friend_next_door_gets_a_lift_to_work_and_back() {
    let mut w = neighbours("friends");
    w.autonomy = false;
    until(&mut w, 2, 10.0);
    let mut aboard = w.cars[0].aboard.clone();
    aboard.sort();
    assert_eq!(aboard, vec![0, 1], "Cy went in Ada's car");
    assert!(w.sims.iter().all(|s| s.away_until.is_some()));
    until(&mut w, 2, 15.0);
    assert_eq!(w.cars[0].state, CarState::Parked, "home again");
    let (x, z) = w.sims[1].tile();
    assert_eq!(w.plot_at(x, z), Some(1), "Cy walked home from next door");
    assert!(w.sims[1].away_until.is_none());
}

#[test]
fn a_neighbour_who_is_no_friend_walks() {
    let mut w = neighbours("strangers");
    w.autonomy = false;
    until(&mut w, 2, 10.0);
    assert_eq!(w.cars[0].aboard, vec![0], "only Ada in her car");
    assert!(w.sims[1].away_until.is_some(), "Cy got to work on foot");
}

#[test]
fn owners_drive_to_work_together_and_come_home_nose_in() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    assert_eq!(w.cars.len(), 1);
    assert_eq!(w.cars[0].owners, vec![0, 1], "the grown-ups own it");
    until(&mut w, 2, 0.0);
    let funds = w.households[0].funds;
    let mut seen = Vec::new();
    let target = clock::tick_at(2, 10.0 * 60.0).unwrap();
    while w.tick < target {
        w.tick_once();
        let s = w.cars[0].state;
        if seen.last() != Some(&s) {
            seen.push(s);
        }
    }
    assert_eq!(
        seen,
        [
            CarState::Parked,
            CarState::Boarding,
            CarState::Leaving,
            CarState::Away
        ],
        "off to work by car"
    );
    assert_eq!(w.cars[0].aboard.len(), 2, "both went in it");
    assert!(w.sims.iter().all(|s| s.away_until.is_some()));
    until(&mut w, 2, 13.0 + 1.0 / 60.0);
    assert_eq!(
        w.households[0].funds,
        funds + 200,
        "a shift each, paid once"
    );
    assert!(
        homeward(&w, 0) && homeward(&w, 1),
        "waiting for the car home"
    );
    until(&mut w, 2, 14.0);
    assert_eq!(w.cars[0].state, CarState::Parked, "home again");
    assert!(
        (w.cars[0].yaw.abs() - std::f32::consts::PI).abs() < 1e-3,
        "nose in"
    );
    assert!(w.sims.iter().all(|s| s.away_until.is_none()));
    assert_eq!(w.households[0].funds, funds + 200);
    until(&mut w, 2, 14.5);
    for s in &w.sims {
        assert_eq!(
            w.plot_at(s.tile().0, s.tile().1),
            Some(0),
            "{} walked in from the car",
            s.name
        );
    }
}

#[test]
fn only_owners_drive() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    // Bo owns it but has no job; Ada may not drive it, so walks.
    w.cars[0].owners = vec![1];
    w.sims[1].job = None;
    until(&mut w, 2, 10.0);
    assert!(w.sims[0].away_until.is_some(), "at work");
    assert_eq!(w.cars[0].state, CarState::Parked, "nobody drove it");
    assert!(w.cars[0].owners == vec![1]);
}

#[test]
fn the_rest_of_the_household_rides_along() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    // Only Bo owns it: Bo drives, Ada (setting off at the same time) rides.
    w.cars[0].owners = vec![1];
    until(&mut w, 2, 10.0);
    assert_eq!(w.cars[0].state, CarState::Away);
    assert_eq!(w.cars[0].driver, Some(1));
    assert!(w.cars[0].aboard.contains(&0), "Ada went with Bo");
}

#[test]
fn a_teenager_never_drives() {
    let content = CONTENT.replace(
            r#""economy""#,
            r#""life":{"stages":[{"id":"teen","label":"Teen","from":13,"adult":false},{"id":"youngAdult","label":"Young adult","from":18,"adult":true}]},"economy""#,
        );
    let town = TOWN.replace(
        r#""name":"Ada","household":0,"#,
        r#""name":"Ada","household":0,"age":16,"#,
    );
    let mut w = World::from_json(&content, &town, 1).unwrap();
    assert_eq!(w.cars[0].owners, vec![1], "only the grown-up owns it");
    // Even listed as an owner, a sixteen-year-old with a job doesn't drive; Bo stays home.
    w.cars[0].owners = vec![0, 1];
    w.sims[0].job = Some(crate::life::Job::new(0, 0, 0));
    w.sims[1].job = None;
    until(&mut w, 2, 10.0);
    assert!(w.sims[0].away_until.is_some(), "Ada went, on foot");
    assert_eq!(w.cars[0].state, CarState::Parked);
}

#[test]
fn a_parked_car_takes_up_its_tiles() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let tiles: Vec<usize> = w.cars[0]
        .tiles()
        .iter()
        .map(|&(x, z)| w.lot.tile_index(x, z))
        .collect();
    assert!(tiles.len() >= 6, "{tiles:?}");
    assert!(
        tiles.iter().all(|&i| w.blocked[i]),
        "nobody walks through it, nothing goes there"
    );
    assert!(
        w.place(
            0,
            34,
            3,
            0,
            Default::default(),
            Default::default(),
            0,
            0,
            None
        )
        .is_err(),
        "no chair on it"
    );
    until(&mut w, 2, 9.5);
    assert_eq!(w.cars[0].state, CarState::Away);
    assert!(tiles.iter().all(|&i| !w.blocked[i]), "free while it's out");
    until(&mut w, 2, 14.0);
    assert_eq!(w.cars[0].state, CarState::Parked);
    assert!(tiles.iter().all(|&i| w.blocked[i]), "taken again");
}

#[test]
fn they_open_its_doors_and_set_off_before_work_starts() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    until(&mut w, 2, 0.0);
    let start = clock::tick_at(2, 9.0 * 60.0).unwrap();
    let mut opened = 0u8;
    let mut getting_in = false;
    while w.tick < start {
        w.tick_once();
        opened |= w.cars[0].shown.map_or(0, |s| s.3);
        getting_in |= w.sims.iter().any(|s| {
            s.current()
                .is_some_and(|a| matches!(a.phase, Phase::Boarding { .. }))
        });
    }
    assert!(getting_in, "they stood at its doors");
    assert_eq!(opened & 1, 1, "the driver's door opened");
    assert!(opened.count_ones() >= 2, "and the rider's: {opened:b}");
    assert_eq!(w.cars[0].state, CarState::Away, "out of town before nine");
    assert!(
        !w.events
            .iter()
            .any(|e| e.kind == crate::social::EventKind::MissedWork)
    );
    // On the road, not at work yet: the shift starts at nine.
    assert!(w.sims.iter().all(|s| s.shift_from == start));
}

#[test]
fn the_car_moves_house_with_its_household() {
    let town = TOWN
            .replace(
                r#"{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}"#,
                r#"{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]},{"name":"Next","x":40,"z":0,"w":20,"d":8,"entry":[45.5,7.5]}"#,
            )
            .replace(
                r#""cars":["#,
                r#""drives":[{"plot":0,"x":34.0,"z":0.55,"front":1.0,"street":9.5},{"plot":1,"x":54.0,"z":1.0,"front":1.0,"street":9.5}],"cars":["#,
            );
    let mut w = World::from_json(CONTENT, &town, 1).unwrap();
    w.households[0].plot = Some(1);
    w.tick_once();
    let car = &w.cars[0];
    assert_eq!(car.plot, 1, "it went along");
    assert!(
        (car.spot[0] - 54.0).abs() < 1e-3 && (car.spot[1] - (1.0 + PARK_GAP + 2.2)).abs() < 1e-3,
        "{:?}",
        car.spot
    );
    assert!(
        (car.yaw.abs() - std::f32::consts::PI).abs() < 1e-3,
        "nose in"
    );
    let old = w.lot.tile_index(34, 3);
    assert!(!w.blocked[old], "the old driveway is free");
    assert!(
        car.tiles()
            .iter()
            .all(|&(x, z)| w.blocked[w.lot.tile_index(x, z)])
    );
}

#[test]
fn a_door_with_no_room_to_open_isnt_used() {
    // The house's wall a metre from the car's left (driver's) side.
    let town = TOWN
        .replace(r#""x":34.0,"z":3.0"#, r#""x":34.5,"z":3.0"#)
        .replace(
            r#""sims":"#,
            r#""walls":[[36,0,40,0],[40,0,40,6],[36,6,40,6],[36,0,36,6]],"sims":"#,
        );
    let w = World::from_json(CONTENT, &town, 1).unwrap();
    let (door, _) = w.cars[0].door(&w.nav(), 0).unwrap();
    assert_eq!(door, 1, "the driver gets in on the other side");
}

#[test]
fn the_door_shuts_before_they_are_gone() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    until(&mut w, 2, 0.0);
    let start = clock::tick_at(2, 9.0 * 60.0).unwrap();
    let (mut last_open, mut gone, mut sat) = (0, None, false);
    while w.tick < start && gone.is_none() {
        w.tick_once();
        if w.cars[0].shown.is_some_and(|s| s.3 != 0) {
            last_open = w.tick;
        }
        sat |= w
            .sims
            .iter()
            .any(|s| s.away_until.is_none() && s.pose == crate::content::Pose::Sit);
        if w.sims.iter().any(|s| s.away_until.is_some()) {
            gone = Some(w.tick);
        }
    }
    assert!(sat, "sat in the car first");
    assert!(
        gone.unwrap() >= last_open + secs(0.5),
        "the door shut ({last_open}) before they went ({gone:?})"
    );
}

#[test]
fn a_car_waits_for_someone_in_its_way_who_then_steps_aside() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.autonomy = false;
    // Ada drives off (at work as far as anyone can tell)...
    w.cars[0].state = CarState::Leaving;
    w.cars[0].progress = 0.0;
    w.cars[0].driver = Some(0);
    w.cars[0].aboard = vec![0];
    w.sims[0].away_until = Some(w.tick + 100_000);
    // ...and Bo stands right in front of it (it faces the street).
    w.sims[1].pos = [34.0, 6.5];
    for _ in 0..secs(1.0) {
        w.tick_once();
    }
    assert_eq!(w.cars[0].progress, 0.0, "it waits");
    for _ in 0..secs(30.0) {
        w.tick_once();
    }
    assert!(
        w.cars[0].progress > 0.0,
        "and goes once Bo has stepped aside"
    );
    assert!(
        (w.sims[1].pos[0] - 34.0).abs() > 1.0,
        "Bo stepped aside: {:?}",
        w.sims[1].pos
    );
}

#[test]
fn someone_whose_way_turns_right_in_front_of_a_waiting_car_gets_by() {
    // (As at Crown Quarter: a walk by road turned in a junction's box right in front of the
    // nose of a car waiting there, a while already, for others before them. At each step on
    // into the ground in front of it, it stepped them back out of it to the middle of the
    // tile beside, where it doesn't wait for them but they're still in its way, so on they
    // step into it again: they never got round, and it never went on.) They walk on round the
    // turn and out of its way, and it goes after. With the car a cm further out each time:
    // just where it stands decides which tiles round it it doesn't wait for them at.
    let mut held = Vec::new();
    for k in 0..=30 {
        let z = 3.4 + k as f32 * 0.01;
        let town = TOWN.replace(r#""z":3.0,"yaw""#, &format!(r#""z":{z},"yaw""#));
        let mut w = World::from_json(CONTENT, &town, 1).unwrap();
        w.autonomy = false;
        w.cars[0].state = CarState::Leaving;
        w.cars[0].progress = 0.0;
        w.cars[0].driver = Some(0);
        w.cars[0].aboard = vec![0];
        w.sims[0].away_until = Some(w.tick + 100_000);
        // (No shift of Bo's to set off for in the while: his way there ends where it's told.)
        w.sims[1].job = None;
        // Bo, just in front of it, on his way along the front of the house...
        w.sims[1].pos = [34.5, 8.5];
        let _ = w.apply(crate::Command::MoveTo {
            sim: 1,
            x: 40.5,
            z: 8.5,
        });
        // (It has waited a while already, for others before them.)
        w.cars[0].waited = PASS_TICKS + 5;
        w.tick_once();
        // ...by the ground right in front of its nose, where his way turns.
        let Some(Phase::Routing { waypoints, next }) =
            w.sims[1].current.as_mut().map(|a| &mut a.phase)
        else {
            panic!("Bo isn't on his way");
        };
        *waypoints = vec![[34.0, 7.6], [40.5, 8.5]];
        *next = 0;
        let mut by = None;
        for t in 0..secs(20.0) {
            w.tick_once();
            if by.is_none() && w.sims[1].pos[0] > 40.0 {
                by = Some(t);
            }
        }
        if !by.is_some_and(|t| t < secs(10.0)) || w.cars[0].progress < secs(1.0) as f32 {
            held.push((z, by, w.sims[1].pos, w.cars[0].progress));
        }
    }
    assert!(
        held.is_empty(),
        "Bo never got round in front of the car, nor it on (its z, when he got by, where he \
             is, its progress): {held:?}"
    );
}

#[test]
fn nobody_walks_into_a_car_driving_by() {
    let mut c = car([10.0, 10.0], 0.0, 20.0);
    c.shown = Some(([10.0, 10.0], 0.0, true, 0));
    let cars = [c];
    let on_foot = Body::WALKER;
    assert_eq!(
        in_the_way(&cars, u32::MAX, on_foot, [12.0, 10.0], [11.0, 10.0]),
        Some(0),
        "into its side"
    );
    assert_eq!(
        in_the_way(&cars, u32::MAX, on_foot, [10.8, 10.0], [11.5, 10.0]),
        None,
        "out of its way"
    );
    assert_eq!(
        in_the_way(&cars, u32::MAX, on_foot, [14.0, 10.0], [13.5, 10.0]),
        None,
        "well clear"
    );
    // (A bike's front wheel well in front of its rider.)
    let bike = Body {
        ahead: 1.2,
        behind: 0.6,
        side: 0.3,
        yaw: 0.0,
    };
    assert_eq!(
        in_the_way(&cars, u32::MAX, on_foot, [12.4, 10.0], [12.0, 10.0]),
        None,
        "on foot, still clear"
    );
    assert_eq!(
        in_the_way(&cars, u32::MAX, bike, [12.4, 10.0], [12.0, 10.0]),
        Some(0),
        "but not on a bike"
    );
    // Nor into the ground just in front of it (it faces +z, its front at z = 12.2): stopped
    // there, they'd hold it up. Out of there is fine.
    assert_eq!(
        in_the_way(&cars, u32::MAX, on_foot, [10.0, 15.5], [10.0, 14.0]),
        Some(0),
        "in front of it"
    );
    assert_eq!(
        in_the_way(&cars, u32::MAX, on_foot, [10.0, 13.5], [10.0, 14.0]),
        None,
        "out from in front of it"
    );
}

#[test]
fn one_holding_up_a_car_goes_on_past_the_nose_of_one_standing_still() {
    // One waiting for those in its way, facing +z at z = 10 (its front at z = 12.2); one
    // standing still beside it facing -x at x = 16 (its front at x = 13.8, the ground it keeps
    // clear to x = 11.6).
    let mut waiting = car([10.0, 10.0], 0.0, 20.0);
    waiting.shown = Some(([10.0, 10.0], 0.0, true, 0));
    waiting.waited = 40;
    let mut standing = car([16.0, 13.0], -std::f32::consts::FRAC_PI_2, 20.0);
    standing.shown = Some(([16.0, 13.0], -std::f32::consts::FRAC_PI_2, true, 0));
    let on_foot = Body::WALKER;
    // In the first's way, on out of it across the ground in front of the second: it's not
    // coming on (it waits its turn), they don't wait for it.
    let (from, to) = ([10.6, 13.0], [11.8, 13.0]);
    let cars = [waiting.clone(), standing.clone()];
    assert_eq!(
        in_the_way(&cars, u32::MAX, on_foot, from, to),
        None,
        "past its nose"
    );
    // Never into its body.
    assert_eq!(
        in_the_way(&cars, u32::MAX, on_foot, [13.0, 13.0], [13.6, 13.0]),
        Some(1),
        "into it"
    );
    // On the move, they wait for it as ever; nor, holding nobody up, into the ground in
    // front of one standing.
    let mut moving = standing.clone();
    moving.moved = true;
    assert_eq!(
        in_the_way(&[waiting.clone(), moving], u32::MAX, on_foot, from, to),
        Some(1),
        "it's coming on"
    );
    let mut clear = waiting;
    clear.waited = 0;
    assert_eq!(
        in_the_way(&[clear, standing], u32::MAX, on_foot, from, to),
        Some(1),
        "holding nobody up"
    );
}

#[test]
fn one_a_car_backing_up_waits_for_goes_on_out_of_its_way() {
    // (Swanmere's: a car backing out of a bay by a lit junction, facing +z and backing towards
    // -z, waited for a cyclist behind it, who waited for it in turn: for good.) Waited for by
    // name, they go on, but never into its body; anyone else waits for it as ever.
    let mut backing = car([10.0, 10.0], 0.0, 20.0);
    backing.shown = Some(([10.0, 10.0], 0.0, true, 0));
    backing.next = Some(([10.0, 9.3], 0.0));
    backing.on_foot = vec![7];
    let (from, to) = ([9.0, 7.0], [9.6, 7.0]);
    let cars = [backing];
    assert_eq!(
        in_the_way(&cars, 8, Body::WALKER, from, to),
        Some(0),
        "another waits for it"
    );
    assert_eq!(
        in_the_way(&cars, 7, Body::WALKER, from, to),
        None,
        "the one it waits for"
    );
    assert_eq!(
        in_the_way(&cars, 7, Body::WALKER, [9.0, 7.95], [9.6, 8.0]),
        Some(0),
        "into its body"
    );
}

#[test]
fn no_trip_parks_on_a_driveway() {
    // (Pebble Sands, a big world of the 0.7.0 samples: a bay off the road where a house's
    // driveway runs, 12 m from its mouth; a car parked out there on a trip, the car of the
    // house coming home couldn't get by it.) A spot on the driveway, one by its mouth, and one
    // down the street clear of it.
    let road: Vec<String> = (0..=199).map(|x| format!("[{x}.5,30]")).collect();
    let town = format!(
        r#"{{"width":200,"depth":60,
            "exits":[{{"x":0.5,"z":30,"kind":"road"}},{{"x":199.5,"z":30,"kind":"road"}}],
            "roads":[{{"points":[{}],"width":7,"kind":"street","ends":["out","out"]}}],
            "plots":[{{"name":"H","x":78,"z":8,"w":16,"d":18,"entry":[83.5,25.5]}}],
            "households":[{{"name":"F","plot":0}}],
            "sims":[{{"name":"D","household":0,"x":83.5,"z":17.5}}],
            "drives":[{{"plot":0,"x":90,"z":8.5,"front":1,"street":30,"road":0,"s":89.5}}],
            "cars":[{{"plot":0,"x":90,"z":11.25,"yaw":0,"street":30,"len":4.6,"wid":1.88,"body":2,"paint":0}}],
            "parking":[{{"x":91.0,"z":20.0,"rot":0,"area":0}},{{"x":93.0,"z":25.0,"rot":0,"area":0}},
                       {{"x":140.0,"z":24.0,"rot":0,"area":0}}]}}"#,
        road.join(",")
    );
    let w = World::from_json(CONTENT, &town, 1).unwrap();
    assert_eq!(w.travel.by_a_mouth, [true, true, false]);
}

#[test]
fn a_ring_with_one_at_a_red_light_someone_is_in_the_way_of_is_held_up() {
    // (Willowshore, seed base 22: a taxi at a red light with another's body in its way through
    // the junction, it in the way of one at the green on the other phase, that one in front
    // of the other's. Whatever the light, one of them was at a red one, so the ring was never
    // turned round: 16 cars stood in view for good.) Three cars round in a ring, one at a red
    // light: it waits for one in its way too, so they're held up by each other; waiting for
    // the light alone, it goes on the green, and they aren't.
    let (a, b, c) = (car_first(0), car_first(1), car_first(2));
    let ring = |a_waits: Vec<u64>| {
        move |k: u64| {
            if k == a {
                a_waits.clone()
            } else if k == b {
                vec![c]
            } else {
                vec![a]
            }
        }
    };
    let red = |k: u64| k == a;
    let held = |_: usize| Some(false);
    let (cars, ok, parked) = held_round(1, ring(vec![b]), red, held);
    assert_eq!(cars, [1, 2, 0]);
    assert!(ok && !parked, "held up by each other");
    let (_, ok, _) = held_round(1, ring(vec![]), red, held);
    assert!(!ok, "it waits for the light alone");
    let (_, ok, _) = held_round(1, ring(vec![b]), red, |o| (o != 2).then_some(false));
    assert!(!ok, "one of them on its way");
}

#[test]
fn of_two_held_up_by_each_other_the_one_that_can_back_up_does() {
    // Two cars on their way, each waiting for the other, neither holding a claim: the later
    // backs up (`waits_back`); can it back up no further, the earlier does instead.
    let town = TOWN.replace(
            r#""cars":[{"plot":0"#,
            r#""cars":[{"plot":0,"x":30.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":1},{"plot":0"#,
        );
    let mut w = World::from_json(CONTENT, &town, 1).unwrap();
    assert_eq!(w.cars.len(), 2);
    let (a, b) = (car_first(0), car_first(1));
    w.cars[0].waits_for = Some(b);
    w.cars[1].waits_for = Some(a);
    assert!(waits_back(&w, a, b), "the later backs up");
    assert!(!waits_back(&w, b, a), "the earlier doesn't");
    w.cars[1].jam.2 = true;
    assert!(!waits_back(&w, a, b), "the later can't");
    assert!(waits_back(&w, b, a), "so the earlier does");
    w.cars[0].jam.2 = true;
    assert!(
        !waits_back(&w, b, a) && !waits_back(&w, a, b),
        "neither can"
    );
}

#[test]
fn of_two_held_up_by_each_other_the_one_backing_up_goes_on() {
    // (Willowbrook, a big world of the 0.7.0 samples: a car and a taxi meeting round a bend
    // took turns backing up a tick each as their claims changed, for good.) The earlier backing
    // up already goes on backing up, though the later would otherwise; backed up as far as it
    // can, the later does.
    let town = TOWN.replace(
            r#""cars":[{"plot":0"#,
            r#""cars":[{"plot":0,"x":30.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":1},{"plot":0"#,
        );
    let mut w = World::from_json(CONTENT, &town, 1).unwrap();
    let (a, b) = (car_first(0), car_first(1));
    w.cars[0].waits_for = Some(b);
    w.cars[1].waits_for = Some(a);
    w.cars[0].jam = (200, 3, false);
    assert!(waits_back(&w, b, a), "the earlier goes on backing up");
    assert!(!waits_back(&w, a, b), "the later doesn't");
    w.cars[0].jam.2 = true;
    assert!(waits_back(&w, a, b), "the earlier can't: the later does");
    assert!(!waits_back(&w, b, a));
}

#[test]
fn a_garage_takes_up_its_ground() {
    let town = TOWN.replace(
        r#""cars":["#,
        r#""garages":[{"plot":0,"x0":40.0,"z0":1.0,"x1":43.2,"z1":6.4}],"cars":["#,
    );
    let mut w = World::from_json(CONTENT, &town, 1).unwrap();
    for (x, z) in [(40, 1), (42, 5), (41, 3)] {
        assert!(
            w.blocked[w.lot.tile_index(x, z)],
            "({x}, {z}) is the garage's"
        );
    }
    assert!(
        w.place(
            0,
            41,
            3,
            0,
            Default::default(),
            Default::default(),
            0,
            0,
            None
        )
        .is_err(),
        "no chair in it"
    );
    // Not lost on a save, nor freed as cars come and go.
    until(&mut w, 2, 10.0);
    let back = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(back.garages, w.garages);
    assert!(back.blocked[back.lot.tile_index(41, 3)]);
}

#[test]
fn a_car_built_over_stays_put_and_survives_a_save() {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    for x in 30..38 {
        w.lot.set_h(x, 3, crate::lot::Edge::Fence);
    }
    w.nav_version += 1;
    until(&mut w, 2, 10.0);
    assert!(w.cars[0].blocked);
    assert_eq!(w.cars[0].state, CarState::Parked);
    assert!(w.cars[0].shown.is_none(), "not drawn");
    assert!(w.sims.iter().all(|s| s.away_until.is_some()), "they walked");

    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    until(&mut w, 2, 9.5);
    assert_eq!(w.cars[0].state, CarState::Away);
    let saved = w.save_json();
    let mut back = World::from_save_json(CONTENT, &saved).unwrap();
    assert_eq!(back.cars[0].state, CarState::Away);
    assert_eq!(back.cars[0].aboard, w.cars[0].aboard);
    until(&mut back, 2, 14.0);
    assert_eq!(
        back.cars[0].state,
        CarState::Parked,
        "back after the load too"
    );
}

fn car(spot: [f32; 2], yaw: f32, street: f32) -> Car {
    Car::from_raw(
        &CarRaw {
            plot: 0,
            x: spot[0],
            z: spot[1],
            yaw,
            street,
            len: 4.4,
            wid: 1.8,
            body: 0,
            paint: 0,
        },
        0,
    )
}

/// On the traffic's clock a car on a country road goes about a metre a tick
/// (`crate::DRIVE_SCALE`): one driven straight through the smallest junction's box at that
/// speed, never slowing for it, still has its route sampled finer than the box, and asks for
/// the box's claim before its body is in it, whichever tick it gets there.
#[test]
fn a_car_at_country_speed_cannot_skip_a_junction_box() {
    let speed = crate::travel::COUNTRY_SPEED;
    assert!(
        speed / crate::DRIVE_TICKS_PER_SECOND > 1.0,
        "a metre a tick or more"
    );
    // (Along x through a box at x = 100, the smallest: `traffic::boxes`' 6 m.)
    let (centre, r) = ([100.0, 0.0], 6.0);
    let mut legs = Legs::default();
    for k in 0..=50 {
        legs.push_lane([k as f32 * 4.0, 0.0], speed, 0.0, LANE);
    }
    let route = legs.timed(false, false);
    let (len, wid) = (4.4, 1.8);
    // The way sampled no further apart than `SAMPLE` and a little: finer than the box.
    let trace = crate::traffic::trace(|t| route.at(t), route.ticks());
    let most = trace
        .windows(2)
        .map(|w| crate::travel::dist(w[0].1, w[1].1))
        .fold(0.0f32, f32::max);
    assert!(
        most < crate::traffic::SAMPLE + 0.5 && most < r,
        "{most} m between samples"
    );
    assert!(SIGHT_STEP / crate::DRIVE_TICKS_PER_SECOND * speed < r);
    let pass = vec![0.0; trace.len()];
    let claims = crate::traffic::claims(&trace, &pass, len, wid, &[(centre, r)], &[]);
    assert_eq!(claims.len(), 1, "the box's claim");
    let c = &claims[0];
    // (Whether its body `t` ticks along touches the box.)
    let touches = |t: f32| {
        let (p, yaw, _) = route.at(t);
        let (dx, dz) = (centre[0] - p[0], centre[1] - p[1]);
        let along = dx * yaw.sin() + dz * yaw.cos();
        let across = dx * yaw.cos() - dz * yaw.sin();
        let off = ((along.abs() - len / 2.0).max(0.0)).hypot((across.abs() - wid / 2.0).max(0.0));
        off <= r
    };
    // Each tick it goes on from `t` to `t + 1` (`drive_on`, `Traffic::may_go`): into the box
    // only asking for its claim first.
    let mut asked = false;
    let mut t = 0.0;
    while t < route.ticks() {
        let asks = t + 1.0 > c.from && t < c.to;
        asked |= asks;
        assert!(
            !touches(t + 1.0) || touches(t) || asks,
            "at {t} it drives into the box ({:?}) unasked",
            route.at(t + 1.0).0
        );
        if touches(t + 1.0) {
            assert!(asked, "in the box at {} before asking", t + 1.0);
        }
        t += 1.0;
    }
    assert!(asked);
}

/// (Eaglescar, world 231441 of seed base 16: a car turned round where the way it came ran a
/// little off the lane it turned round to; it stepped sideways onto that lane and, facing along
/// the step, swung its tail half across a road too narrow for more as it drove on, into one
/// coming the other way: both held up for good.) A car that came east along a street on its
/// middle line backs up and drives on east: onto its lane ahead of where it stands, facing
/// the way it goes all along.
#[test]
fn a_car_turned_round_off_its_lane_drives_on_facing_the_way_it_goes() {
    let road: Vec<String> = (0..=99).map(|x| format!("[{x}.5,30]")).collect();
    let town = format!(
        r#"{{"width":100,"depth":60,
            "exits":[{{"x":0.5,"z":30,"kind":"road"}},{{"x":99.5,"z":30,"kind":"road"}}],
            "roads":[{{"points":[{}],"width":7,"kind":"street","ends":["out","out"]}}],
            "plots":[],"households":[],"objects":[],"sims":[],"cars":[]}}"#,
        road.join(",")
    );
    let w = World::from_json(CONTENT, &town, 1).unwrap();
    let c = car([50.0, 10.0], 0.0, 30.0);
    let mut legs = Legs::default();
    for k in 0..=20 {
        legs.push_lane([20.0 + k as f32 * 2.0, 30.0], 5.0, 0.0, LANE);
    }
    let had = legs.timed(true, true);
    let at = |x: f32| had.points.iter().find(|p| p.0[0] >= x).unwrap().2;
    let rp = w.travel.graph.near([40.0, 30.0], ROAD_REACH, 1, Mode::Car)[0];
    let d = Detour {
        home: false,
        from: at(50.0),
        back: at(36.0),
        avoid: rp.edge,
        same: false,
        mouth: None,
    };
    let r = c
        .detoured(&w.exits, &w.travel, false, &d, &had)
        .expect("a way on");
    let on = r.points.iter().position(|p| !p.3).expect("it drives on");
    assert!(
        r.points[on..].iter().all(|p| !p.3),
        "backs up, then drives on"
    );
    // (At each point it drives on from, it faces the way it goes, as near as a step's curve.)
    let mut lane = 0.0f32;
    for k in r.points[on..].windows(2) {
        let (a, b) = (k[0], k[1]);
        lane = lane.max((b.0[1] - 30.0).abs());
        if crate::travel::dist(a.0, b.0) < 0.05 {
            continue;
        }
        let way = (b.0[0] - a.0[0]).atan2(b.0[1] - a.0[1]);
        assert!(
            wrap_angle(a.1 - way).abs() < 0.2,
            "at {:?} it faces {} going {way}",
            a.0,
            a.1
        );
    }
    assert!(lane > 0.5, "it comes to its lane ({lane} m off the middle)");
}

#[test]
fn pulls_out_and_drives_to_the_nearer_end() {
    // Street along x at z = 20, exits at both ends; the car faces the street.
    let exits = [[0.5, 20.0], [99.5, 20.0]];
    let mut c = car([70.0, 10.0], 0.0, 20.0);
    let r = Route::new(&c, &exits, false);
    let last = r.points.last().unwrap().0;
    assert!(last[0] > 99.5, "drives to the east end: {last:?}");
    assert!((last[1] - (20.0 + TIGHT_LANE)).abs() < 1e-3, "in its lane");
    // Starts from the spot facing the street, ends heading east.
    assert!((r.points[0].0[0] - 70.0).abs() < 1e-3);
    assert!(r.points[0].1.abs() < 0.2);
    assert!((r.points.last().unwrap().1 - std::f32::consts::FRAC_PI_2).abs() < 0.1);
    // Takes a while, but not forever (seconds of drive time).
    let secs = r.ticks() / DRIVE_TICKS_PER_SECOND;
    assert!(secs > 5.0 && secs < 40.0, "{secs} s");
    // The pose follows the route.
    c.state = CarState::Leaving;
    assert!(
        c.pose(&exits)
            .is_some_and(|(p, _, moving)| moving && (p[0] - 70.0).abs() < 1e-3)
    );
    c.progress = r.ticks() + 1.0;
    assert!(c.pose(&exits).is_none());
}

#[test]
fn backs_out_when_parked_nose_in_and_comes_home_nose_in() {
    let exits = [[0.5, 20.0], [99.5, 20.0]];
    // Facing the house (away from the street at +z).
    let c = car([30.0, 10.0], std::f32::consts::PI, 20.0);
    let r = Route::new(&c, &exits, false);
    // Backing out, it faces the house still.
    assert!(
        (r.points[1].1.abs() - std::f32::consts::PI).abs() < 0.2,
        "{}",
        r.points[1].1
    );
    assert!(r.points.last().unwrap().0[0] < 0.5, "west, the nearer end");
    let home = Route::new(&c, &exits, true);
    let end = *home.points.last().unwrap();
    assert!((end.0[0] - 30.0).abs() < 1e-3 && (end.0[1] - 10.0).abs() < 1e-3);
    assert!(
        (end.1.abs() - c.nose_in()).abs() < 0.1,
        "nose in: {}",
        end.1
    );
}

#[test]
fn a_car_faces_within_a_half_turn_either_way_backing_out_too() {
    // (Its riders sit facing as it does: a yaw past a full turn was a resident facing 6.3.)
    let exits = [[0.5, 20.0], [99.5, 20.0]];
    for yaw in [std::f32::consts::PI, -std::f32::consts::FRAC_PI_2, 0.0, 3.0] {
        let c = car([30.0, 10.0], yaw, 20.0);
        for home in [false, true] {
            let r = Route::new(&c, &exits, home);
            let mut t = 0.0;
            while t <= r.ticks() {
                let (_, a, _) = r.at(t);
                assert!(
                    (-std::f32::consts::PI..=std::f32::consts::PI).contains(&a),
                    "parked at {yaw}, home {home}: {a} at {t}"
                );
                t += 0.25;
            }
        }
    }
}

#[test]
fn pulling_back_in_beside_one_waiting_for_it_a_car_keeps_the_gap() {
    // Pulled over 0.5 m to its right beside one coming the other way that waits for it, it
    // pulls back in a step at a time (`pull_over`: nothing but the guard keeps it from that
    // one). In its lane its body would touch the other's: it stops short by the gap the cars
    // keep, waiting for that one.
    let (len, wid) = (4.6, 1.88);
    let them = [Driving {
        at: [wid, 0.0],
        yaw: std::f32::consts::PI,
        len,
        wid,
        ahead: AHEAD,
        behind: 0.0,
        first: car_first(1),
    }];
    let body = |shift: f32| Rect {
        at: shifted([0.0, 0.0], 0.0, shift),
        yaw: 0.0,
        half: [len / 2.0, wid / 2.0],
    };
    let mut shift: f32 = 0.5;
    let mut held = None;
    while shift > 0.0 && held.is_none() {
        let to = (shift - PULL_RATE).max(0.0);
        held = guard(&body(shift), &body(to), them.iter());
        if held.is_none() {
            shift = to;
        }
    }
    // (Side by side, as far apart as it's pulled over.)
    assert!(
        shift >= 0.149,
        "pulled back in to {shift:.3} m from it (`GAP`)"
    );
    assert_eq!(held, Some(car_first(1)), "never held for it");
    // (And on along its lane so near it: no nearer, as near as it is.)
    let on = |r: Rect| Rect {
        at: [r.at[0], r.at[1] + 0.3],
        ..r
    };
    assert_eq!(guard(&body(shift), &on(body(shift)), them.iter()), None);
}

/// The way along `pts`: the turn at each point between its steps (radians), and whether any
/// step goes back against the one before.
fn turns(pts: &[[f32; 2]]) -> (f32, bool) {
    let mut most = 0.0f32;
    let mut back = false;
    for w in pts.windows(3) {
        let (a, b) = (unit2(w[0], w[1]), unit2(w[1], w[2]));
        most = most.max(turned(a, b).abs());
        back |= dot2(a, b) < 0.0;
    }
    (most, back)
}

#[test]
fn a_lane_turns_a_corner_on_an_arc() {
    // Along x, then right (+z) at (12, 0): a street corner, a point every 4 m.
    let lane = [
        [0.0, 0.0],
        [4.0, 0.0],
        [8.0, 0.0],
        [12.0, 0.0],
        [12.0, 4.0],
        [12.0, 8.0],
        [12.0, 12.0],
    ];
    let round = round_corners(&lane);
    assert_eq!(round.first(), lane.first());
    assert_eq!(round.last(), lane.last());
    let (most, back) = turns(&round);
    assert!(!back);
    assert!(most <= ARC_STEP + 1e-3, "turns {most} at a point");
    // On an arc as wide as a right turn's, its middle in from the corner.
    let r = RIGHT_TURN;
    let mid = round
        .iter()
        .min_by(|a, b| {
            crate::travel::dist(**a, [12.0, 0.0]).total_cmp(&crate::travel::dist(**b, [12.0, 0.0]))
        })
        .copied()
        .unwrap();
    let inset = crate::travel::dist(mid, [12.0, 0.0]);
    assert!(
        (inset - r * (std::f32::consts::SQRT_2 - 1.0)).abs() < 0.05,
        "{inset}"
    );
    // Turning left, wider.
    let left: Vec<[f32; 2]> = lane.iter().map(|p| [p[0], -p[1]]).collect();
    let round = round_corners(&left);
    let mid = round
        .iter()
        .min_by(|a, b| {
            crate::travel::dist(**a, [12.0, 0.0]).total_cmp(&crate::travel::dist(**b, [12.0, 0.0]))
        })
        .copied()
        .unwrap();
    let inset = crate::travel::dist(mid, [12.0, 0.0]);
    assert!(
        (inset - LEFT_TURN * (std::f32::consts::SQRT_2 - 1.0)).abs() < 0.05,
        "{inset}"
    );
}

#[test]
fn a_lane_through_a_junction_where_it_doubles_back_turns_smoothly() {
    // Two roads' lanes meeting at a junction: the one in goes on 0.8 m past the corner, the
    // one out begins 0.6 m short of the line in, and the way doubles back between them.
    let lane = [
        [0.0, 0.0],
        [4.0, 0.0],
        [8.0, 0.0],
        [12.8, 0.0],
        [12.0, -0.6],
        [12.0, 3.0],
        [12.0, 7.0],
        [12.0, 11.0],
    ];
    let round = round_corners(&lane);
    let (most, back) = turns(&round);
    assert!(!back, "{round:?}");
    assert!(
        most <= ARC_STEP + 1e-3,
        "turns {most} at a point: {round:?}"
    );
    // On through the corner, no point further than the lines either side.
    assert!(
        round.iter().all(|p| p[0] <= 12.0 + 1e-3 && p[1] >= -1e-3),
        "{round:?}"
    );
}

#[test]
fn a_car_faces_along_its_lane_where_the_next_step_is_a_hair_across_it() {
    // (King'ston and Pebble Quay: where a taxi stood at the kerb, the way on began 2 to 8 mm
    // from it, mostly across the lane, and it stood 25 to 37 degrees across its lane.) Along
    // +z, standing at z = 8, the next point 5 mm across: it faces along the lane there.
    let mut legs = Legs::default();
    for p in [
        [0.0, 0.0],
        [0.0, 4.0],
        [0.0, 8.0],
        [0.005, 8.001],
        [0.0, 12.0],
        [0.0, 16.0],
    ] {
        legs.push_lane(p, FAST, 0.0, LANE);
    }
    let route = legs.timed(true, true);
    for &(at, yaw, ..) in &route.points[1..route.points.len() - 1] {
        assert!(
            yaw.abs().to_degrees() < 4.0,
            "{}° across its lane at {:?}",
            yaw.to_degrees(),
            at
        );
    }
}

#[test]
fn a_car_faces_the_way_it_goes_round_a_corner() {
    let lane = round_corners(&[
        [0.0, 0.0],
        [4.0, 0.0],
        [8.0, 0.0],
        [12.0, 0.0],
        [12.0, 4.0],
        [12.0, 8.0],
    ]);
    let mut legs = Legs::default();
    for p in lane {
        legs.push_lane(p, FAST, 0.0, LANE);
    }
    let route = legs.timed(true, true);
    let mut was = route.at(0.0).0;
    let mut t = 1.0;
    let mut worst = 0.0f32;
    while t < route.ticks() {
        let (p, yaw, _) = route.at(t);
        let (dx, dz) = (p[0] - was[0], p[1] - was[1]);
        if dx.hypot(dz) > 1e-3 {
            worst = worst.max(wrap_angle(dx.atan2(dz) - yaw).abs());
        }
        was = p;
        t += 1.0;
    }
    assert!(
        worst.to_degrees() < 4.0,
        "its nose {}° off the way it goes",
        worst.to_degrees()
    );
}
