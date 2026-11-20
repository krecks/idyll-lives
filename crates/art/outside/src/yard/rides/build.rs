//! The bikes as parts are given them.
//!
//! Split from `yard/rides.rs` (see `mod.rs`).

use super::*;

/// A bike `S` in its nodes (the note above): `extras` and `stand` (its pivot at game (x, height) `pivot`) build what
/// this one has of its own.
fn ride(
    name_: &str,
    s: &Bike,
    extras: fn(&Bike),
    stand: Option<fn(&Bike)>,
    pivot: (f64, f64),
    centre: bool,
) -> Vec<Obj> {
    let rear = new_parts(|| wheel(s.rear, s.z, s));
    let (frame, front, cranks) = bike(s);
    let more = new_parts(|| extras(s));
    let cranks_more = new_parts(|| spindle(s));
    let kick = new_parts(|| {
        if let Some(stand) = stand {
            stand(s);
        }
    });
    let how = Settle {
        centre,
        floor: true,
    };
    if is_low() {
        // from afar: parked, all one still part
        return settle_with(&[join_all(name_)], how);
    }
    let mut body = frame;
    body.extend(more);
    join(&body, "frame", Some(P3::from((0, 0, 0))));
    join(&front, "wheel_front", Some(F.at(s.front, s.r, s.z)));
    join(&rear, "wheel_rear", Some(F.at(s.rear, s.r, s.z)));
    let mut turning = cranks;
    turning.extend(cranks_more);
    join(&turning, "cranks", Some(F.at(s.bb.0, s.bb.1, s.z)));
    if !kick.is_empty() {
        join(&kick, "stand", Some(F.at(pivot.0, pivot.1, s.z)));
    }
    settle_with(&meshes(), how)
}

pub fn bicycle() -> Vec<Obj> {
    ride(
        "bicycle",
        &city(),
        city_extras,
        Some(city_stand),
        CITY_STAND,
        false,
    )
}

pub fn mountain_bike() -> Vec<Obj> {
    ride("mountainBike", &mtb(), mtb_extras, None, (0.0, 0.0), false)
}

pub fn road_bike() -> Vec<Obj> {
    ride("roadBike", &road(), road_extras, None, (0.0, 0.0), false)
}

pub fn cargo_bike() -> Vec<Obj> {
    ride("cargoBike", &cargo(), cargo_extras, None, (0.0, 0.0), true)
}

pub fn e_bike() -> Vec<Obj> {
    ride("eBike", &ebike(), ebike_extras, None, (0.0, 0.0), false)
}

pub fn bicycle_child() -> Vec<Obj> {
    ride("bicycleChild", &kid(), kid_extras, None, (0.0, 0.0), false)
}
