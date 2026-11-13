//! Loading and saving cars, and the parking spots trips use.
//!
//! Split from `cars.rs` (see `mod.rs`).

use super::*;

impl World {
    /// For tests: the tiles parked cars and garages take up (x, z), sorted as stacked rows are:
    /// nobody walks through them (`reblock`).
    #[doc(hidden)]
    pub fn parked_tiles(&self) -> Vec<(i32, i32)> {
        self.car_tiles
            .iter()
            .map(|&i| self.lot.tile_xz(i))
            .collect()
    }
}

/// The cars of a saved game: those that still make sense (their plot, household and people are
/// there, their numbers are numbers), every house's driveway, and the garages.
pub(crate) fn load(w: &mut World, saved: &[Car], drives: &[Drive], garages: &[Garage]) {
    let sims = w.sims.len() as u32;
    let finite = |v: &[f32]| v.iter().all(|v| v.is_finite());
    let ok = |c: &Car| {
        (c.plot as usize) < w.plots.len()
            && (c.household as usize) < w.households.len()
            && finite(&[c.spot[0], c.spot[1], c.yaw, c.street, c.len, c.wid])
            && c.len > 0.0
            && c.wid > 0.0
    };
    w.drives = drives
        .iter()
        .filter(|d| (d.plot as usize) < w.plots.len() && finite(&[d.x, d.z, d.front, d.street]))
        .copied()
        .collect();
    w.garages = garages
        .iter()
        .filter(|g| (g.plot as usize) < w.plots.len() && finite(&[g.x0, g.z0, g.x1, g.z1]))
        .copied()
        .collect();
    w.cars = saved
        .iter()
        .filter(|c| ok(c))
        .take(MAX_CARS)
        .cloned()
        .collect();
    for car in &mut w.cars {
        // (On its way, it comes into sight as one coming in does, clear of the others: it may have
        // been driving through one out of sight as the game was saved, `sight`.)
        if matches!(car.state, CarState::Leaving | CarState::Arriving) {
            car.sight = Sight::Ghost;
        }
        car.owners.retain(|&o| o < sims);
        car.aboard.retain(|&o| o < sims);
        car.done.retain(|o| car.aboard.contains(o));
        car.trip = car
            .trip
            .filter(|t| t.driver < sims && (t.plot as usize) < w.plots.len());
        car.riders.retain(|&o| o < sims);
        if car.trip.is_none() {
            car.riders.clear();
        }
        // (On a trip, its driver even while they're out of it, parked out there.)
        let trip = car.trip.map(|t| t.driver);
        car.driver = car
            .driver
            .filter(|d| car.aboard.contains(d) || Some(*d) == trip);
    }
    settle_all(w);
}

/// Builds the town's cars from its file: each goes to the household living on its plot (none
/// there: no car), owned by its grown-ups.
pub(crate) fn from_file(w: &mut World, raw: &[CarRaw], drives: &[Drive], garages: &[Garage]) {
    w.drives = drives.to_vec();
    w.garages = garages.to_vec();
    w.cars = raw
        .iter()
        .filter_map(|r| {
            let h = w.households.iter().position(|h| h.plot == Some(r.plot))?;
            let mut car = Car::from_raw(r, h as u32);
            car.owners = w
                .sims
                .iter()
                .filter(|s| s.here() && s.household == h as u32 && s.adult(&w.content))
                .map(|s| s.id)
                .collect();
            Some(car)
        })
        .take(MAX_CARS)
        .collect();
    settle_all(w);
}

/// After the cars were made or loaded: their routes' lengths, what's built over, the tiles they
/// take up, where they are.
pub(crate) fn settle_all(w: &mut World) {
    let World {
        cars,
        exits,
        travel,
        drives,
        ..
    } = &mut *w;
    // (The public parking spots a car parked at would stand in the way of one turning into or out
    // of its driveway, or on the driveway itself (a bay off the road where a driveway runs: the
    // car coming home couldn't get by it): none parks there on a trip, `parking_for`.)
    let ways: Vec<([f32; 2], [f32; 2])> = drives
        .iter()
        .filter_map(|d| {
            let (road, s) = d.meets()?;
            let mouth = travel.graph.point(travel.graph.at_source(road, s)?);
            Some(([d.x, d.z], mouth))
        })
        .collect();
    travel.by_a_mouth = (0..travel.file.parking.len())
        .map(|k| {
            parked_at(travel, k).is_some_and(|(at, ..)| {
                ways.iter().any(|&(inner, mouth)| {
                    crate::travel::dist(mouth, at) < MOUTH_CLEAR
                        || to_segment(at, inner, mouth) < DRIVE_CLEAR
                })
            })
        })
        .collect();
    for car in cars.iter_mut() {
        let drive = drives.iter().find(|d| d.plot == car.plot);
        car.meets = drive.and_then(Drive::meets);
        car.axis = drive.and_then(Drive::way);
        car.settle(exits, Some(travel));
    }
    w.car_tiles.clear();
    w.cars_parked = None;
    refresh(w);
    reblock(w);
    poses(w);
}

// ---- Trips by car (`Trip`) ---------------------------------------------------------------

/// How near a driveway's mouth (where it meets its road) the middle of a car parked out on a trip
/// may not be (m: a car's length and the swing of one turning in or out by it).
pub(crate) const MOUTH_CLEAR: f32 = 7.0;

/// How near a driveway (the line from its inner end to its mouth) the middle of a car parked out
/// on a trip may not be (m: half of each of two cars side by side, the gap between them and some).
pub(crate) const DRIVE_CLEAR: f32 = 2.5;

/// How far `p` is from the segment `a`..`b` (m).
pub(crate) fn to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let t =
        ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-6);
    let t = t.clamp(0.0, 1.0);
    crate::travel::dist(p, [a[0] + ab[0] * t, a[1] + ab[1] * t])
}

/// The public parking spot (`parking`) nearest `door` within `TRIP_PARK_REACH`: where it is and
/// which way a car faces parked there. Of `cars`, but car `me`, none parks where another is on a
/// trip to (on its way there, parked there, or still pulling out of it home: they'd stand one on
/// top of the other), nor at one by a driveway's mouth (`Travel::by_a_mouth`): with all near
/// taken, none.
pub(crate) fn parking_for(
    travel: &Travel,
    door: [f32; 2],
    cars: &[Car],
    me: Option<usize>,
) -> Option<([f32; 2], f32)> {
    let taken = |at: [f32; 2]| {
        cars.iter().enumerate().any(|(c, car)| {
            let there = || match car.state {
                CarState::Arriving => car
                    .shown
                    .is_some_and(|s| crate::travel::dist(s.0, at) < car.len),
                _ => true,
            };
            Some(c) != me
                && car
                    .trip
                    .is_some_and(|t| crate::travel::dist(t.at, at) < 0.5)
                && there()
        })
    };
    (0..travel.file.parking.len())
        .filter(|&k| !travel.by_a_mouth.get(k).copied().unwrap_or(false))
        .filter_map(|k| parked_at(travel, k))
        .map(|(at, yaw, _)| (crate::travel::dist(at, door), at, yaw))
        .filter(|&(d, at, _)| d <= TRIP_PARK_REACH && !taken(at))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, at, yaw)| (at, yaw))
}

/// Where a car parked at public parking spot `k` (`OpenWorld::parking`) stands, which way it
/// faces and whether it's a bay off the road (nose in, facing away from it: `parking_bay`; else
/// as the spot, on the road).
pub(crate) fn parked_at(travel: &Travel, k: usize) -> Option<([f32; 2], f32, bool)> {
    let p = travel.file.parking.get(k)?;
    let at = [p.x, p.z];
    at.iter().all(|v| v.is_finite()).then_some(())?;
    Some(match parking_bay(travel, at) {
        Some((_, _, [x, z])) => (at, (0.0 - x).atan2(0.0 - z), true),
        None => (at, p.rot, false),
    })
}

/// Whether there's public parking near `door` for a trip by car (`TRIP_PARK_REACH`), taken or not.
pub(crate) fn parks_near(travel: &Travel, door: [f32; 2]) -> bool {
    parking_near(travel, door).is_some()
}

/// The public parking spot nearest `door` for a trip by car (`TRIP_PARK_REACH`), taken or not:
/// the walk from it to there is part of the trip.
pub(crate) fn parking_near(travel: &Travel, door: [f32; 2]) -> Option<[f32; 2]> {
    parking_for(travel, door, &[], None).map(|(at, _)| at)
}
