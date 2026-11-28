//! Tests for kerbs and stopping.

use super::*;

/// A street along the town, two houses north of it, their driveways onto it 48 m apart.
fn street() -> World {
    let content = r#"{"needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],
        "objects":[],"economy":{"startingFunds":500}}"#;
    let points: Vec<String> = (0..200)
        .map(|k| format!("[{},30]", 0.5 + k as f32))
        .collect();
    let town = format!(
        r#"{{"width":200,"depth":60,
        "exits":[{{"x":0.5,"z":30,"kind":"road"}},{{"x":199.5,"z":30,"kind":"road"}}],
        "roads":[{{"points":[{}],"width":7,"kind":"street","ends":["out","out"]}}],
        "plots":[{{"name":"H0","x":92,"z":8,"w":16,"d":18,"entry":[100.5,17.5]}},
                 {{"name":"H1","x":140,"z":8,"w":16,"d":18,"entry":[148.5,17.5]}}],
        "households":[{{"name":"F0","plot":0}},{{"name":"F1","plot":1}}],
        "objects":[],
        "sims":[{{"name":"A","household":0,"x":100.5,"z":17.5}},{{"name":"B","household":1,"x":148.5,"z":17.5}}],
        "drives":[{{"plot":0,"x":100,"z":20.5,"front":1,"street":30,"road":0,"s":99.5}},
                  {{"plot":1,"x":148,"z":20.5,"front":1,"street":30,"road":0,"s":147.5}}]}}"#,
        points.join(",")
    );
    World::from_json(content, &town, 1).unwrap()
}

/// A house's stand as kept (`warm_kerbs`) is the one for the driveways as they are: one
/// moved since, it's worked out again, whoever asks (it once took `warm_kerbs` first).
#[test]
fn a_kept_stand_is_for_the_driveways_as_they_are() {
    let mut w = street();
    warm_kerbs(&mut w);
    let kept = stop_for(&w, 0).expect("a stand by A's");
    // B's driveway now meets the street just where A's taxi stood.
    let d = w.drives.iter_mut().find(|d| d.plot == 1).unwrap();
    d.s = Some(kept.s);
    let fresh = kerb_at(
        &w.travel,
        &w.meta,
        home_way(&w, 0).unwrap(),
        &[],
        &drive_mouths(&w),
    )
    .expect("a stand by A's still");
    assert!((fresh.s - kept.s).abs() >= 1.0, "the stand moved clear");
    let now = stop_for(&w, 0).expect("a stand");
    assert_eq!((now.road, now.s), (fresh.road, fresh.s));
    // Warmed again, the same.
    warm_kerbs(&mut w);
    let warm = stop_for(&w, 0).expect("a stand");
    assert_eq!((warm.road, warm.s), (fresh.road, fresh.s));
}

/// A house nobody lives in has its taxi bay by its door: the door moved (the roads and driveways
/// as they were), its bay moves with it (`warm_kerbs` once looked again only at households').
#[test]
fn an_empty_houses_bay_moves_with_its_door() {
    let mut w = street();
    // H1 empty, no driveway: its stand is by its door.
    w.households[1].plot = None;
    w.drives.retain(|d| d.plot != 1);
    warm_kerbs(&mut w);
    // (Within a stand's reach of x: H0's bay is 48 m and more from both doors.)
    let near = |w: &World, x: f32| w.taxis.kerbs.bays.iter().any(|b| (b[0] - x).abs() < 30.0);
    assert!(near(&w, 148.5), "a bay by H1's door");
    w.plots[1].entry = Some([40.5, 17.5]);
    warm_kerbs(&mut w);
    assert!(near(&w, 40.5), "a bay by its new door");
    assert!(!near(&w, 148.5), "none left by the old");
}

/// No more than `LEISURE_FLEET` taxis go out on visits and outings at once, there and back
/// counted alike; the work taxi's runs don't count.
#[test]
fn the_leisure_taxis_out_at_once_are_a_fleet_of_six() {
    let run = |id: u32, place: Option<u32>, home: bool| -> Run {
        serde_json::from_value(serde_json::json!({
            "id": id, "home": home, "place": place, "group": [0, 0], "enter": 100,
            "stops": [], "progress": 0.0}))
        .unwrap()
    };
    let mut taxis = Taxis::default();
    for k in 0..10 {
        taxis.runs.push(run(k, None, false));
    }
    assert!(!taxis.fleet_out(), "work runs aren't the fleet");
    for k in 0..LEISURE_FLEET as u32 - 1 {
        taxis.runs.push(run(10 + k, Some(3), k % 2 == 0));
    }
    assert!(!taxis.fleet_out());
    taxis.runs.push(run(20, Some(4), true));
    assert!(taxis.fleet_out(), "{LEISURE_FLEET} out, there or back");
}

/// A taxi in and out by a motorway's end of town: the way in and the way out sit by both
/// carriageways' ends, nearer the one going the other way; it comes in along the one coming in
/// and leaves along the one going out (`plan_way`, `cars::exit_points`).
#[test]
fn a_taxi_comes_in_and_leaves_by_the_carriageway_going_its_way() {
    use crate::travel::{OneWay, RoadKind, RoadRaw, Travel};
    let line = |a: [f32; 2], b: [f32; 2]| -> Vec<[f32; 2]> {
        let n = crate::travel::dist(a, b).ceil() as usize;
        (0..=n)
            .map(|i| {
                let t = i as f32 / n as f32;
                [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
            })
            .collect()
    };
    let road = |pts: Vec<[f32; 2]>, width: f32, kind: RoadKind, oneway: OneWay| RoadRaw {
        points: pts,
        width,
        kind,
        area: None,
        oneway,
    };
    // Carriageways along z = 100 (east, coming in from x = 0.5) and z = 109 (west, going out);
    // ramps between them and a street at x = 150 north of both.
    let travel = Travel::new(
        crate::travel::OpenWorld {
            roads: vec![
                road(
                    line([0.5, 100.0], [299.5, 100.0]),
                    9.0,
                    RoadKind::Motorway,
                    OneWay::Along,
                ),
                road(
                    line([0.5, 109.0], [299.5, 109.0]),
                    9.0,
                    RoadKind::Motorway,
                    OneWay::Against,
                ),
                road(
                    line([60.0, 100.0], [150.0, 80.0]),
                    5.0,
                    RoadKind::Ramp,
                    OneWay::Along,
                ),
                road(
                    line([150.0, 130.0], [60.0, 109.0]),
                    5.0,
                    RoadKind::Ramp,
                    OneWay::Along,
                ),
                road(
                    line([150.0, 0.5], [150.0, 199.5]),
                    7.0,
                    RoadKind::Street,
                    OneWay::Both,
                ),
            ],
            ..Default::default()
        },
        &serde_json::json!({}),
    );
    let net = Net::new(&travel, &serde_json::json!({}), &[]);
    let road_of = |p: &(u32, f32, f32)| travel.graph.roads[p.0 as usize].source;
    // (The motorway's west end 2 m off the eastbound's end, 7 m off the westbound's.)
    let end = [0.5, 102.0];
    // In by it, out at the street's north end: along the eastbound, east.
    let legs = plan_way(&travel, &net, end, &[], [150.0, 199.5]).expect("in by the motorway");
    let first = legs.concat()[0];
    assert_eq!(road_of(&first), 0, "in along the eastbound: {legs:?}");
    assert!(first.2 > first.1, "east along it");
    // In at the street's south end, out by it: along the westbound, west.
    let legs = plan_way(&travel, &net, [150.0, 0.5], &[], end).expect("out by the motorway");
    let last = *legs.concat().last().unwrap();
    assert_eq!(road_of(&last), 1, "out along the westbound: {legs:?}");
    assert!(last.2 < last.1, "west along it");
}
