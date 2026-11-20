//! Taxi jams: noticing them and getting round them.
//!
//! Split from `taxi.rs` (see `mod.rs`).

use super::*;

/// How long two cars or more by a junction's box stand still on their way before it counts as
/// jammed (ticks of drive time, `jammed`), how far round its box they're by it (m), and how long a
/// taxi due into town through one waits at the edge of town for it to clear at most (ticks of
/// drive time, as the cars clear it, `run_on`: a run to work, half its spare time at most).
const JAM_STILL: u64 = crate::drive_secs(5.0);
/// How long one car on its way stands still before the place counts as held up on its own (ticks
/// of drive time).
pub(crate) const LONE_STILL: u64 = crate::drive_secs(30.0);
pub(crate) const JAM_NEAR: f32 = 8.0;
pub(crate) const JAM_WAIT: u64 = crate::drive_secs(30.0);
/// A taxi stands clear of another's stand when it's there within this long of it, either way (ticks,
/// `stand_window`): as each is timed, give or take its stand, a hold-up on its way and a wait at
/// the edge of town (`JAM_WAIT`). Real time, not drive time: the stand is its riders' (getting
/// in, `DWELL`, and coming out to it), which don't speed up with the traffic.
pub(crate) const STAND_NEAR: u64 = crate::secs(30.0);
/// A run whose stand another taxi has when it'd be there comes as much later as clears it, by
/// `STAND_NEAR` at a time, this much at most (ticks, `stands_and_way`).
pub(crate) const STAND_LATER: u64 = 3 * 60 * TICKS_PER_MINUTE;
/// A car on its way that stays within this of where it was (m) stands still, as `jammed` has it.
const STILL_REACH: f32 = 3.0;

/// Since when each car on its way has stood still (`Taxis::still`).
pub(crate) fn note_still(w: &mut World) {
    let tick = w.tick;
    let red: Vec<bool> = (0..w.cars.len())
        .map(|c| {
            w.traffic.waiting(crate::cars::car_first(c))
                && crate::traffic::at_red(w, crate::cars::car_first(c))
        })
        .collect();
    let still = &mut w.taxis.still;
    still.resize(w.cars.len(), (None, tick));
    for (c, car) in w.cars.iter().enumerate() {
        let at = car.shown.filter(|s| s.2).map(|s| s.0).filter(|_| {
            matches!(
                car.state,
                crate::cars::CarState::Leaving | crate::cars::CarState::Arriving
            )
        });
        // (Inching on or backing up a little, as one held up does, is standing still; waiting at
        // a red light isn't, it changes: a taxi given up there let its riders out in the road,
        // and waited for them standing in its way for good, Wheat Acres.)
        let moved = match (at, still[c].0) {
            (Some(p), Some(q)) => crate::travel::dist(p, q) > STILL_REACH,
            _ => true,
        } || red[c];
        if moved {
            still[c] = (at, tick);
        }
    }
}

/// Where cars are held up now (middles and how far round them): a junction's box with two cars or
/// more by it standing still on their way for `JAM_STILL` or longer (a ring of them waiting for
/// each other, or a queue into one); anywhere else two such cars stand near each other (held up
/// face to face on a stretch); and any one car standing still on its way `LONE_STILL` or longer.
pub(crate) fn jammed(w: &World) -> Vec<([f32; 2], f32)> {
    let g = &w.travel.graph;
    let mut out: Vec<([f32; 2], f32)> = w
        .taxis
        .jams
        .iter()
        .filter_map(|&b| g.boxes.get(b).copied())
        .collect();
    let held: Vec<[f32; 2]> = w
        .cars
        .iter()
        .zip(&w.taxis.still)
        .filter(|(_, s)| s.0.is_some() && w.tick >= s.1 + JAM_STILL)
        .filter_map(|(c, _)| c.shown.map(|s| s.0))
        .collect();
    // (One held up a long while on its own: there.)
    out.extend(
        w.cars
            .iter()
            .zip(&w.taxis.still)
            .filter(|(_, s)| s.0.is_some() && w.tick >= s.1 + LONE_STILL)
            .filter_map(|(c, _)| c.shown.map(|s| (s.0, JAM_NEAR))),
    );
    if held.len() >= 2 {
        let boxed = |p: [f32; 2]| {
            g.boxes
                .iter()
                .any(|&(m, r)| crate::travel::dist(p, m) < r + JAM_NEAR)
        };
        out.extend(g.boxes.iter().copied().filter(|&(m, r)| {
            held.iter()
                .filter(|p| crate::travel::dist(**p, m) < r + JAM_NEAR)
                .count()
                >= 2
        }));
        for (i, &a) in held.iter().enumerate() {
            for &b in &held[i + 1..] {
                let d = crate::travel::dist(a, b);
                if d < 2.0 * JAM_NEAR && !boxed(a) && !boxed(b) {
                    let m = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
                    out.push((m, d / 2.0 + JAM_NEAR));
                }
            }
        }
    }
    out
}

/// Whether `route` goes through one of the places held up given (`jammed`), or by it (`JAM_NEAR`:
/// the junction it holds up, the queue into it).
pub(crate) fn through(route: &crate::cars::Route, jams: &[([f32; 2], f32)]) -> bool {
    jams.iter().any(|&(m, r)| {
        route
            .points
            .iter()
            .any(|p| crate::travel::dist(p.0, m) < r + JAM_NEAR)
    })
}

// ---------------------------------------------------------------------------------------------
// The taxi for visits and outings.

/// The most a taxi ride to a visit or an outing may take, door to door, against walking there:
/// less than this share of the walk, else they walk (per cent).
pub(crate) const LEISURE_BEATS: f32 = 0.9;
/// The taxi's stands on the way and a rider's wait at the kerb (minutes, `leisure_eta`).
pub(crate) const LEISURE_KERBS: f32 = 8.0;
/// A walk to or from the kerb is about this much longer than as the crow flies (`kerb_walks`).
pub(crate) const KERB_WINDING: f32 = 1.3;
/// Those off to the same place within this long of each other share a taxi (ticks).
pub(crate) const LEISURE_WINDOW: u64 = 30 * TICKS_PER_MINUTE;
/// A taxi back from a place comes this long after its rider's visit ends (ticks: a goodbye and
/// out to the kerb).
pub(crate) const BACK_SLACK: u64 = 8 * TICKS_PER_MINUTE;
