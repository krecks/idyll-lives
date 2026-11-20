//! Which car a household has: the one its best earner's job calls for.
//!
//! Each job's grade calls for a car (`careers.json`: the field's or the track's `car`, a body from
//! each grade on, and the grade's `carAge`; `CareerLevel::car`): a trade a van, finance a saloon
//! and an executive saloon higher up, the army an off-roader, and older to newer up the ladder.
//! The household's car is the one its best earner's job calls for (the most pay a week, then the
//! first of them): a town's households have theirs from the start (`fit_town`), and a promotion
//! or a new job in the household (`job_changed`) has it buy the one that job calls for, if that's
//! another body or a newer one (`Car::want`); one who quits, is fired, retires, moves out or
//! leaves town before it's bought doesn't buy it (`job_lost`). The car changes as soon as it stands idle at home
//! (nobody in it or on their way to it) or is away at work (`renew`): it parks as one of its
//! length does there, and the story says who bought it (`EventKind::BoughtCar`). A household with
//! nobody at work (retired, out of work, at school) keeps the car it has, and so does one whose
//! new car wouldn't fit where it parks (something stands where its longer body would, or it would
//! stand further off its plot). Whether a house has a car at all is the town's, as before; its
//! paint is the town's too and stays.

use super::*;
use crate::content::{CarAge, CarLook, Content};

/// The car a promotion or a new job calls for (`Car::want`), and who the story says bought it
/// (none: nobody is told, as a town's cars are fitted).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Want {
    pub body: u8,
    #[serde(default)]
    pub age: CarAge,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<u32>,
}

impl Car {
    /// What it looks like now, but its paint.
    pub fn look(&self) -> CarLook {
        CarLook {
            body: self.body,
            age: self.age,
        }
    }
}

/// `household`'s best earner (the most pay a week; of two alike, the first) and the car their job
/// calls for; None when nobody in it works, or their job calls for none in particular.
pub fn fitting(content: &Content, sims: &[Sim], household: u32) -> Option<(u32, CarLook)> {
    sims.iter()
        .filter(|s| s.here() && s.household == household)
        .filter_map(|s| {
            let job = s.job.as_ref()?;
            let level = content.careers.get(job.career)?.levels.get(job.level)?;
            let week = level
                .pay
                .saturating_mul(i64::from(level.days.count_ones().max(1)));
            Some((week, s.id, level.car))
        })
        .max_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)))
        .and_then(|(_, id, car)| Some((id, car?)))
}

/// Whether a car looking like `now` is changed for `fit`: another body, or the same one newer.
fn calls_for(now: CarLook, fit: CarLook) -> bool {
    fit.body != now.body || fit.age.index() > now.age.index()
}

/// A promotion or a new job in `household`: its car is to be the one its best earner's job calls
/// for (`Car::want`, bought by them), if that's another or a newer one; else it stays as it is.
pub(crate) fn job_changed(cars: &mut [Car], content: &Content, sims: &[Sim], household: u32) {
    let fit = fitting(content, sims, household);
    for car in cars
        .iter_mut()
        .filter(|c| !c.taxi && c.household == household)
    {
        car.want = fit
            .filter(|&(_, look)| calls_for(car.look(), look))
            .map(|(by, look)| Want {
                body: look.body,
                age: look.age,
                by: Some(by),
            });
    }
}

/// `sim` has no job any more (quit, fired, retired, gone) or has moved out: a car their job called
/// for that their household hasn't bought yet isn't bought.
pub(crate) fn job_lost(cars: &mut [Car], sim: u32) {
    for car in cars {
        if car.want.is_some_and(|w| w.by == Some(sim)) {
            car.want = None;
        }
    }
}

/// The town's cars, each the one its household's best earner's job calls
/// for, nobody told: those standing idle at once, the others when they next do (`renew`). The
/// caller settles them after (`settle_all`).
pub(crate) fn fit_town(w: &mut World) {
    for c in 0..w.cars.len() {
        let car = &w.cars[c];
        if car.taxi {
            continue;
        }
        let fit = fitting(&w.content, &w.sims, car.household)
            .filter(|&(_, look)| look != car.look())
            .map(|(_, look)| Want {
                body: look.body,
                age: look.age,
                by: None,
            });
        w.cars[c].want = fit;
        if let Some(want) = fit
            && idle(w, c)
        {
            w.cars[c].want = None;
            if let Some(at) = refit(w, c, want) {
                put(w, c, want, at);
            }
        }
    }
}

/// Car `c` stands idle: parked with nobody in it or on their way to it, or away at work.
fn idle(w: &World, c: usize) -> bool {
    let car = &w.cars[c];
    match car.state {
        CarState::Parked => {
            car.driver.is_none()
                && car.aboard.is_empty()
                && car.trip.is_none()
                && !w.sims.iter().any(|s| s.bound_for_car == Some(c as u32))
        }
        CarState::Away => car.trip.is_none(),
        _ => false,
    }
}

/// Where car `c` as `want` parks: where one of its length does on its driveway (its inner end as
/// far from the driveway's inner end; in front of the player's garage, `park_by_garage`, after),
/// with its size. None when it doesn't fit there: something stands where it would (a thing, a
/// wall or a fence, a garage, a porch post) and nothing where the car it has stands, or more of it
/// would stand off its plot.
fn refit(w: &World, c: usize, want: Want) -> Option<([f32; 2], f32, f32, f32)> {
    let def = w.content.car_bodies.get(usize::from(want.body))?;
    let car = &w.cars[c];
    let (base, yaw) = car
        .drive_spot
        .map_or((car.spot, car.yaw), |[x, z, yaw]| ([x, z], yaw));
    let grow = (def.len - car.len) / 2.0;
    let spot = match w
        .drives
        .iter()
        .find(|d| d.plot == car.plot)
        .and_then(Drive::way)
    {
        Some([x, z]) => [base[0] + x * grow, base[1] + z * grow],
        None => base,
    };
    let mut new = car.posed(spot, yaw);
    (new.len, new.wid) = (def.len, def.wid);
    let old_tiles = car.tiles_at(base, yaw);
    let new_tiles = new.tiles();
    let mut things = things_on(w, &[car.plot]);
    // (A garage or a porch post on the lot stands there too.)
    things.extend(
        w.garages
            .iter()
            .filter(|g| g.plot == car.plot)
            .flat_map(|g| g.tiles()),
    );
    things.extend(
        w.porches
            .iter()
            .filter(|p| p.plot == car.plot)
            .flat_map(|p| p.post_tiles()),
    );
    things.extend(
        w.porch_pieces
            .iter()
            .filter(|p| p.plot == car.plot && p.kind == crate::porches::PieceKind::Post)
            .map(|p| (p.x, p.z)),
    );
    if built_over(&w.lot, &things, &new_tiles) && !built_over(&w.lot, &things, &old_tiles) {
        return None;
    }
    let off = |tiles: &[(i32, i32)]| match w.plots.get(car.plot as usize) {
        Some(p) => tiles.iter().filter(|&&(x, z)| !p.contains(x, z)).count(),
        None => 0,
    };
    if off(&new_tiles) > off(&old_tiles) {
        return None;
    }
    Some((spot, yaw, def.len, def.wid))
}

/// Car `c` is `want` now, parked at `at` (`refit`: its spot, yaw and size), where it stands.
fn put(w: &mut World, c: usize, want: Want, (spot, yaw, len, wid): ([f32; 2], f32, f32, f32)) {
    let car = &mut w.cars[c];
    car.body = want.body;
    car.age = want.age;
    (car.len, car.wid) = (len, wid);
    let garaged = car.drive_spot.take().is_some();
    car.spot = spot;
    car.yaw = yaw;
    car.route = None;
    if garaged {
        park_by_garage(w, c);
    }
}

/// Once a tick for each household's car: the car a promotion or a new job called for
/// (`Car::want`), bought as soon as it stands idle (`idle`), parked where one as long does, the
/// story told; dropped when it wouldn't fit there (`refit`).
pub(crate) fn renew(w: &mut World, c: usize) {
    let Some(want) = w.cars[c].want else {
        return;
    };
    if !idle(w, c) {
        return;
    }
    w.cars[c].want = None;
    let Some(at) = refit(w, c, want) else {
        return;
    };
    put(w, c, want, at);
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    cars[c].settle(exits, Some(travel));
    // (Parked as another size: its tiles are worked out again, and whether it's built over, its
    // own only, as the other cars in this tick's loop are as they were; the things on the car
    // plots it stands on count, as `refresh` counts them; the renderer draws its new look.)
    w.cars_parked = None;
    let tiles = w.cars[c].tiles();
    let own = w.cars[c].plot;
    let plots: Vec<u32> = std::iter::once(own)
        .chain(w.cars.iter().map(|o| o.plot).filter(|&p| {
            p != own
                && w.plots
                    .get(p as usize)
                    .is_some_and(|q| tiles.iter().any(|&(x, z)| q.contains(x, z)))
        }))
        .collect();
    let things = things_on(w, &plots);
    w.cars[c].blocked = built_over(&w.lot, &things, &tiles);
    w.structure_version = w.structure_version.wrapping_add(1);
    if let Some(by) = want.by.filter(|&s| (s as usize) < w.sims.len()) {
        w.events.push_detail(
            w.tick,
            crate::social::EventKind::BoughtCar,
            by as usize,
            Some(i64::from(want.body)),
            None,
        );
    }
}
