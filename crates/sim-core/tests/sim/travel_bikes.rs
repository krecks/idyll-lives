//! Riding a bike to work and back (`ride.rs`, content `vehicle`): someone with a bike at home
//! rides it out of town at its speed, it goes to work with them and comes home with them, back
//! at its place; a way too short for it is walked; two can't share one bike.

use sim_core::snapshot;
use sim_core::{World, clock};

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

fn hours(h: f32) -> u64 {
    (h * 60.0 / sim_core::MINUTES_PER_TICK) as u64
}

fn bike(w: &World) -> usize {
    w.objects
        .iter()
        .position(|o| w.content.objects[o.def].id == "bike")
        .unwrap()
}

fn day_hour(w: &mut World, day: u32, hour: f32) {
    let t = clock::tick_at(day, hour * 60.0).unwrap();
    while w.tick < t {
        w.tick_once();
    }
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
    let end = w.tick + hours(10.0);
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
        day_hour(&mut w, 1, 8.0);
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
    until_away(&mut loaded, 0, hours(30.0), |_| {}).expect("never got to work after loading");
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
    until_away(&mut w, 0, hours(30.0), |_| {}).expect("never got to work on foot");
}
