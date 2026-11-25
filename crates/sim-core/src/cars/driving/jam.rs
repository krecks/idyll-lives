//! Holding up: jams, backing off, and the room a car keeps.

use super::*;

/// How long a car and one in its way wait for each other before it backs up (ticks of drive time,
/// `back_off`), and how far back it goes at most (ticks along its route, a tick back each tick).
pub(crate) const JAM_TICKS: u32 = drive_secs(5.0) as u32;
pub(crate) const JAM_BACK: u32 = drive_secs(3.0) as u32;

/// Whether car `k` (its `Driving::first`) waited for car `me` last tick, so that car `me`, waiting
/// for it, should back up (`back_off`): of two cars, the one not holding a claim (holding one,
/// the other was let through where it waits: it's that one that's in the way); else the later.
pub(crate) fn waits_back(w: &World, k: u64, me: u64) -> bool {
    let o = (k & !(1 << 63)) as usize;
    let (mine, theirs) = (w.traffic.holding(me), w.traffic.holding(k));
    // (Turning out of its driveway, it has its way: the other backs up to give it room to finish
    // its turn.)
    let (out, their_out) = (turning_out(w, me), turning_out(w, k));
    // (Of two not holding one, the one waiting its turn for a claim the other is in the way of:
    // that one can't go on out of its way till it's backed up out of that one's.)
    let (asks, their_asks) = (
        !w.traffic.waits_for(me).is_empty(),
        !w.traffic.waits_for(k).is_empty(),
    );
    // (Of the two, the one that can still back up: the other can't, or no further.)
    let mine_to_go = if out != their_out {
        their_out
    } else if mine != theirs {
        theirs
    } else if !mine && asks != their_asks {
        asks
    } else {
        me > k
    };
    // (One backing up out of the other's way already goes on: not the other in turn as their
    // claims change, the one going on by letting go of its claim behind it, say; nor one, then the
    // other, a tick each.)
    let backing = |x: u64| {
        w.cars
            .get((x & !(1 << 63)) as usize)
            .is_some_and(|c| c.jam.1 > 0)
            && !stuck_back(w, x)
    };
    let mine_to_go = match (backing(me), backing(k)) {
        (true, false) => true,
        (false, true) => false,
        _ => mine_to_go,
    };
    let mine_to_go = if mine_to_go {
        !stuck_back(w, me)
    } else {
        stuck_back(w, k) && !stuck_back(w, me)
    };
    mine_to_go
        && (w.cars.get(o).is_some_and(|c| c.waits_for == Some(me))
            || w.traffic.waits_for(k).contains(&me))
}

/// Whether car `k` (its `Driving::first`) is a household's car still turning out of its driveway
/// onto the road: in its route's first mouth (`Route::mouths`).
pub(crate) fn turning_out(w: &World, k: u64) -> bool {
    w.cars.get((k & !(1 << 63)) as usize).is_some_and(|c| {
        !c.taxi
            && c.state == CarState::Leaving
            && c.route.as_ref().is_some_and(|(_, r)| {
                r.mouths
                    .first()
                    .is_some_and(|&(a, b)| a <= 0.0 && c.progress <= b)
            })
    })
}

/// Whether car `k` (its `Driving::first`) can back up no further out of the way of one it holds
/// up as that one holds it up (`back_off`, `waits_back`).
pub(crate) fn stuck_back(w: &World, k: u64) -> bool {
    w.cars
        .get((k & !(1 << 63)) as usize)
        .is_some_and(|c| c.jam.2)
}

/// Car `c` a tick back along its route, out of the way of one it holds up as that one holds it
/// up (`JAM_TICKS`): never into another (`guard`) nor into what's left of another's claim
/// (`traffic::Traffic::blocked`). Whether it moved.
pub(crate) fn back_off(w: &mut World, c: usize) -> bool {
    let key = car_first(c);
    let others = vehicles(&w.cars, Some(c));
    let car = &w.cars[c];
    let Some((home, route)) = car.route.as_ref() else {
        return false;
    };
    let (t, shift, half) = (car.progress, car.shift, [car.len / 2.0, car.wid / 2.0]);
    let to = (t - 1.0).max(0.0);
    // (Turned round, still on the stretch it backs up along, its last fraction of a tick before it
    // drives on: a tick back along it is forward, the way it backed, and there it's turning round
    // again (`drive_on`), with no jam rule of its own: it'd neither back up nor say it can't, and
    // the other would wait for it for good. It can back up no further: the other does.)
    let turned = car.detour.is_some() && route.at(t).2;
    // (Never on a motorway's carriageway or a ramp, nor back onto one: `Route::on_motorway`.)
    if to >= t || route.at(to).2 != route.at(t).2 || turned || route.on_motorway(to, t) {
        return false;
    }
    let body = |t: f32| {
        let (p, yaw, _) = route.at(t);
        Rect {
            at: shifted(p, yaw, shift),
            yaw,
            half,
        }
    };
    let (now, next) = (body(t), body(to));
    // (Not back into a junction's box it's out of: it holds no claim to it, and standing there it
    // would hold up those taking their turn through it, one it's held up with among them: they'd
    // wait for each other for good, Rowan Hollow.)
    let boxes = &w.travel.graph.boxes;
    if in_box(boxes, next.at, car.len) && !in_box(boxes, now.at, car.len) {
        return false;
    }
    let way = [-next.yaw.sin(), -next.yaw.cos()];
    // (Back out of the claim of the one it's held up with, too: away from it is out of its way.)
    let partner = car.waits_for;
    let layer = car.layer();
    let standing: Vec<Driving> = parked(&w.cars).collect();
    if guard(&now, &next, layer, others.iter().chain(&standing)).is_some()
        || w.traffic
            .blocked(key, &now, &next, way, 0.0, false, layer)
            .is_some_and(|k| Some(k) != partner)
    {
        return false;
    }
    let home = *home;
    let car = &mut w.cars[c];
    car.progress = to;
    car.jam.1 += 1;
    car.backed_for = partner;
    if home == (car.state == CarState::Arriving) {
        car.shown = Some((next.at, next.yaw, true, 0));
    }
    true
}

/// How far right of its lane car `car` has room to pull over `t` ticks along its route
/// (`Route::room_at`).
pub(crate) fn route_room(car: &Car, t: f32) -> f32 {
    car.route.as_ref().map_or(0.0, |(_, r)| r.room_at(t))
}

/// How far car `car` can pull over `t` ticks along its route (`Route::pass_at`).
pub(crate) fn route_pass(car: &Car, t: f32) -> f32 {
    car.route.as_ref().map_or(0.0, |(_, r)| r.pass_at(t))
}
