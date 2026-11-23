//! The tests' own drives (`tests/sim/travel_hierarchy.rs`): cars set on their way along the roads
//! between two places, with nobody at the wheel nor any household's life about them, driven a tick
//! at a time by the traffic's rules alone (`traffic::begin`, `drive_on`), as every car on its way
//! is.

use super::*;

/// The length and width of a drill's car (m).
const LEN: f32 = 4.4;
const WID: f32 = 1.8;

/// A car put on its way from the road nearest `from` to the road nearest `to` (as
/// `empty_drive_ticks` lays it out, with its claims), `at` ticks along it already. Its index in
/// `World::cars`; None: no way.
pub fn add(w: &mut World, from: [f32; 2], to: [f32; 2], at: f32) -> Option<usize> {
    let mut route = empty_drive_route(&w.travel, from, to)?;
    route.claim(LEN, WID, Some(&w.travel));
    let at = at.clamp(0.0, route.ticks());
    let (p, yaw, _) = route.at(at);
    let mut car = Car::taxi(p, LEN, WID);
    car.taxi = false;
    car.state = CarState::Leaving;
    car.route = Some((false, route));
    car.progress = at;
    car.shown = Some((p, yaw, true, 0));
    car.moved = true;
    w.cars.push(car);
    Some(w.cars.len() - 1)
}

/// Car `c` put `t` ticks along its way, as it would be shown there.
pub fn put(w: &mut World, c: usize, t: f32) {
    let car = &mut w.cars[c];
    let Some((_, route)) = car.route.as_ref() else {
        return;
    };
    let (p, yaw, _) = route.at(t);
    car.progress = t;
    car.shown = Some((p, yaw, true, 0));
}

/// Car `c` standing `t` ticks along its way (a breakdown: `put` there, not on the move), as each
/// tick of a drill leaves it till it's let go.
pub fn hold(w: &mut World, c: usize, t: f32) {
    put(w, c, t);
    let car = &mut w.cars[c];
    car.moved = false;
}

/// Whether car `c` has turned round (`unjam`: its way turned round, backing up along it or on
/// from there).
pub fn turned_round(w: &World, c: usize) -> bool {
    w.cars[c].detour.is_some()
}

/// A tick of the drills' traffic: the claims (`traffic::begin`), then each car on its way a tick
/// on (`drive_on`); one at its way's end is gone.
pub fn step(w: &mut World) {
    w.tick += 1;
    crate::traffic::begin(w);
    w.cars_near.build(&w.cars);
    for c in 0..w.cars.len() {
        if w.cars[c].state != CarState::Leaving {
            continue;
        }
        drive_on(w, c);
        if w.cars[c].progress >= ticks(w, c) {
            let car = &mut w.cars[c];
            car.state = CarState::Away;
            car.shown = None;
            car.route = None;
            w.traffic.forget(car_first(c));
        }
    }
}

/// Car `c`'s way's length (ticks).
pub fn ticks(w: &World, c: usize) -> f32 {
    w.cars[c].route.as_ref().map_or(0.0, |r| r.1.ticks())
}

/// Where car `c` is `t` ticks along its way.
pub fn at(w: &World, c: usize, t: f32) -> [f32; 2] {
    w.cars[c].route.as_ref().map_or([0.0; 2], |r| r.1.at(t).0)
}

/// Whether car `c` is on a motorway's deck now (`travel::Layer::DECK`).
pub fn on_deck(w: &World, c: usize) -> bool {
    w.cars[c].layer() == crate::travel::Layer::DECK
}

/// The car (index) someone on foot stepping from `from` to `to` would walk into (`in_the_way`).
pub fn walks_into(w: &World, from: [f32; 2], to: [f32; 2]) -> Option<usize> {
    in_the_way(&w.cars, u32::MAX, Body::WALKER, from, to)
}

/// Car `c`'s merges onto a motorway's carriageway (ticks along its way).
pub fn merges(w: &World, c: usize) -> Vec<f32> {
    w.cars[c]
        .route
        .as_ref()
        .map_or(Vec::new(), |r| r.1.merges.clone())
}

/// The drive out of town (or, `home`, back in) of a car parked at `spot` facing `yaw` on a
/// driveway square to the road nearest it (`Route::on_roads`): where it is and which way it faces
/// each tick. None: no way.
pub fn from_driveway(
    w: &World,
    spot: [f32; 2],
    yaw: f32,
    home: bool,
) -> Option<Vec<([f32; 2], f32)>> {
    let raw = CarRaw {
        plot: u32::MAX,
        x: spot[0],
        z: spot[1],
        yaw,
        street: spot[1],
        len: LEN,
        wid: WID,
        body: 0,
        paint: 0,
    };
    let car = Car::from_raw(&raw, u32::MAX);
    let route = Route::on_roads(&car, &w.exits, &w.travel, home)?;
    let n = route.ticks().ceil() as usize;
    Some(
        (0..=n)
            .map(|t| {
                let (p, yaw, _) = route.at(t as f32);
                (p, yaw)
            })
            .collect(),
    )
}

/// A taxi in by `way_in` and out by `way_out` standing for a house whose front door is at `door`
/// (no driveway): where it stands and its yaw there. None: no stand or no way.
pub fn taxi_stand(
    w: &World,
    way_in: [f32; 2],
    door: [f32; 2],
    way_out: [f32; 2],
) -> Option<([f32; 2], f32)> {
    crate::taxi::stand_for_door(w, way_in, door, way_out)
}
