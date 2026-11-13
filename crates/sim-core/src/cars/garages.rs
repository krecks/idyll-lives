//! Arriving home, driveways and garages, and the tiles parked cars take up.
//!
//! Split from `cars.rs` (see `mod.rs`).

use super::*;

/// The car is home: it parks, nose in, its doors open and they get out and walk to the house.
pub(crate) fn arrive(w: &mut World, c: usize) {
    let tick = w.tick;
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    car.state = CarState::Parked;
    car.since = tick;
    car.trip = None;
    car.riders.clear();
    car.route = None;
    car.yaw = car.nose_in_on(Some(travel));
    car.settle(exits, Some(travel));
    let driver = car.driver.take();
    let mut out = std::mem::take(&mut car.done);
    car.aboard.clear();
    // (The tiles under it before anyone gets out beside it.)
    reblock(w);
    // The driver first, then the others, a seat each.
    out.sort_by_key(|&s| Some(s) != driver);
    for (seat, s) in out.into_iter().enumerate() {
        let i = s as usize;
        w.sims[i].away_until = None;
        walk_home(w, i, Some((c, seat)));
    }
}

/// `i` sets off home on foot: out of the car (`by`: the car and their seat in it), sat in it as
/// its door opens, then out beside it (`Phase::Alighting`); else from where they are.
pub(crate) fn walk_home(w: &mut World, i: usize, by: Option<(usize, usize)>) {
    if let Some((c, seat)) = by {
        let door = w.cars[c].door(&w.nav(), seat).map(|(d, _)| d);
        if let Some(door) = door {
            let car = &w.cars[c];
            let (at, yaw) = (car.seat_at(seat, door), car.yaw);
            let sim = &mut w.sims[i];
            sim.pos = at;
            sim.yaw = yaw;
            sim.pose = crate::content::Pose::Sit;
            sim.bound_for_car = Some(c as u32);
            sim.car_seat = seat as u8;
            sim.car_door = door as u8;
            sim.current = Some(crate::world::Activity {
                task: Task {
                    kind: TaskKind::GoHome,
                    directed: false,
                    routine: false,
                },
                phase: Phase::Alighting { ticks: 0 },
                tags: Default::default(),
            });
            return;
        }
    }
    let sim = &mut w.sims[i];
    if !sim.queue.iter().any(|t| matches!(t.kind, TaskKind::GoHome)) {
        sim.queue.push_front(Task {
            kind: TaskKind::GoHome,
            directed: false,
            routine: false,
        });
    }
}

/// While an owner goes to the car or waits in it: the rest of the household setting off now is
/// called to it (those on their way to the exit turn round, if they're still close by). Whether
/// anyone is on their way to it.
pub(crate) fn call_riders(w: &mut World, c: usize) -> bool {
    let car = &w.cars[c];
    let (household, plot) = (car.household, car.plot);
    // Who drives it off, for a neighbour's lift: only with a seat free (`lift_seat`).
    let driver = setting_off_driver(car, c as u32, &w.briefs, None)
        .filter(|&d| riders(car, c as u32, &w.briefs, d, None) + 1 < SEATS);
    let mut expected = false;
    for i in 0..w.sims.len() {
        let s = &w.sims[i];
        if !s.here() || s.away_until.is_some() || w.cars[c].aboard.contains(&(i as u32)) {
            continue;
        }
        let heading_out = s
            .current()
            .is_some_and(|a| matches!(a.task.kind, TaskKind::Work));
        // The household, still at home; or a neighbour setting off from theirs, for a lift
        // (`lift_seat`, which `may_board` asks).
        let from = if s.household == household {
            Some(plot)
        } else {
            w.households.get(s.household as usize).and_then(|h| h.plot)
        };
        // (A neighbour on their way to it may be on either lot.)
        if !heading_out
            || from.is_none()
            || w.briefs
                .get(i)
                .is_none_or(|b| b.plot.is_some_and(|p| Some(p) != from && p != plot))
        {
            continue;
        }
        if s.bound_for_car == Some(c as u32) {
            expected = true;
            continue;
        }
        // (A neighbour first: is it a lift for them at all? `lift_ok`.)
        if s.household != household {
            let door = from.map(|p| w.plots[p as usize].arrival_point());
            let (car, rels) = (&w.cars[c], &w.relationships);
            let lift = |d| {
                door.is_some_and(|door| {
                    lift_ok(car, &w.households, rels, door, i as u32, s.household, d)
                })
            };
            if !driver.is_some_and(lift) {
                continue;
            }
        }
        // On their way to the exit: to the car instead (`ride_for` picks it now).
        if may_board(w, i, c as u32) {
            let sim = &mut w.sims[i];
            if let Some(a) = sim.current.take() {
                sim.queue.push_front(a.task);
            }
            expected = true;
        }
    }
    expected
}

/// Two owners set off for the car at once, both to drive it: the first drives, the others find
/// themselves a seat as riders (they head for the car again, `seat` seeing the driver coming).
pub(crate) fn one_driver(w: &mut World, c: usize) {
    let drivers: Vec<usize> = (0..w.sims.len())
        .filter(|&i| {
            let s = &w.sims[i];
            s.bound_for_car == Some(c as u32)
                && s.car_seat == 0
                && s.current().is_some_and(|a| {
                    matches!(a.task.kind, TaskKind::Work)
                        && matches!(a.phase, Phase::Routing { .. })
                })
        })
        .collect();
    for &i in drivers.iter().skip(1) {
        let sim = &mut w.sims[i];
        sim.bound_for_car = None;
        if let Some(a) = sim.current.take() {
            sim.queue.push_front(a.task);
        }
    }
}

/// Owners who left town or the household no longer own it; a household left with none hands it
/// down to its grown-ups.
pub(crate) fn owners(w: &mut World, c: usize) {
    let household = w.cars[c].household;
    let sims = &w.sims;
    w.cars[c].owners.retain(|&o| {
        sims.get(o as usize)
            .is_some_and(|s| s.here() && s.household == household)
    });
    if w.cars[c].owners.is_empty() {
        let content = &w.content;
        let heirs: Vec<u32> = w
            .sims
            .iter()
            .filter(|s| s.here() && s.household == household && s.adult(content))
            .map(|s| s.id)
            .collect();
        w.cars[c].owners = heirs;
    }
}

/// The household moved house: the car goes with it, to the new home's driveway (parked there
/// nose in), once it's parked or out with them. None there (or another car on it): it stays where
/// it was, unused, out of sight.
pub(crate) fn relocate(w: &mut World, c: usize) {
    let car = &w.cars[c];
    let Some(home) = w
        .households
        .get(car.household as usize)
        .and_then(|h| h.plot)
    else {
        return;
    };
    if home == car.plot || !matches!(car.state, CarState::Parked | CarState::Away) {
        return;
    }
    let taken = w.cars.iter().enumerate().any(|(k, o)| {
        k != c
            && o.plot == home
            && w.households
                .get(o.household as usize)
                .is_some_and(|h| h.plot == Some(home))
    });
    let Some(drive) = w.drives.iter().find(|d| d.plot == home).copied() else {
        return;
    };
    if taken {
        return;
    }
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    car.plot = home;
    car.street = drive.street;
    car.meets = drive.meets();
    car.axis = drive.way();
    car.spot = drive.spot(car.len);
    car.yaw = car.nose_in_on(Some(travel));
    car.drive_spot = None;
    car.settle(exits, Some(travel));
    // (Parked as it was, somewhere else: its tiles are worked out again.)
    w.cars_parked = None;
    refresh(w);
    // (In front of the garage there, if the player built one at the end of that driveway.)
    park_by_garage(w, c);
}

/// Which cars' parking spots have been built over: a wall or a fence across one, a thing on the
/// tiles it takes up.
pub(crate) fn refresh(w: &mut World) {
    let things = things_on(w, &w.cars.iter().map(|c| c.plot).collect::<Vec<_>>());
    let lot = &w.lot;
    for car in &mut w.cars {
        car.blocked = built_over(lot, &things, &car.tiles());
    }
}

/// The tiles the things on `plots` take from walking.
pub(crate) fn things_on(w: &World, plots: &[u32]) -> std::collections::HashSet<(i32, i32)> {
    let mut things: std::collections::HashSet<(i32, i32)> = Default::default();
    for (o, plot) in w.objects.iter().zip(&w.object_plot) {
        if plot.is_some_and(|p| plots.contains(&p)) {
            things.extend(w.blocking_tiles(o));
        }
    }
    things
}

/// Whether a car standing on `tiles` would be built over: off the lot, a thing there (`things`),
/// a diagonal wall, or a wall or a fence between two of them.
pub(crate) fn built_over(
    lot: &crate::lot::Lot,
    things: &std::collections::HashSet<(i32, i32)>,
    tiles: &[(i32, i32)],
) -> bool {
    let walled = |x: i32, z: i32| {
        // An edge inside the car's tiles (not round their outline).
        let inside = |x: i32, z: i32| tiles.contains(&(x, z));
        (inside(x, z)
            && inside(x, z - 1)
            && lot.h_edge(x as usize, z as usize) != crate::lot::Edge::Open)
            || (inside(x, z)
                && inside(x - 1, z)
                && lot.v_edge(x as usize, z as usize) != crate::lot::Edge::Open)
    };
    tiles.iter().any(|&(x, z)| {
        !lot.in_bounds(x, z) || things.contains(&(x, z)) || lot.diag_blocks(x, z) || walled(x, z)
    })
}

// ---- Garages the player builds ----------------------------------------------------------

/// How far across its driveway's line a garage's door may be from the car's middle (m) for the
/// car to drive straight in, and how much further on from where it parks (m).
pub(crate) const GARAGE_LINE: f32 = 0.8;
pub(crate) const GARAGE_REACH: f32 = 10.0;

/// Where `car`, parked at `base` (its spot and yaw) on its driveway, parks with a garage the
/// player built on its plot (`Garage::rot`) whose door faces its driveway: one in line with the
/// car, a little further from the road it meets (`GARAGE_REACH`), its door towards that road. It
/// parks in front of the door, nose in. None: no such garage. A town's own garages (no `rot`)
/// change nothing: their cars park where the town put them.
pub(crate) fn garage_spot(w: &World, car: &Car, base: ([f32; 2], f32)) -> Option<([f32; 2], f32)> {
    let (fs, fc) = car
        .posed(base.0, base.1)
        .nose_in_on(Some(&w.travel))
        .sin_cos();
    let reach = PARK_GAP + car.len / 2.0;
    w.garages
        .iter()
        .filter(|g| g.plot == car.plot)
        .find_map(|g| {
            let (m, n) = g.door()?;
            // Its door faces the road: against the way the car faces parked nose in.
            if n[0] * fs + n[1] * fc > -0.9 {
                return None;
            }
            let (dx, dz) = (base.0[0] - m[0], base.0[1] - m[1]);
            let out = dx * n[0] + dz * n[1];
            let across = dx * n[1] - dz * n[0];
            (across.abs() <= GARAGE_LINE && out > car.len / 2.0 && out <= reach + GARAGE_REACH)
                .then(|| {
                    (
                        [m[0] + n[0] * reach, m[1] + n[1] * reach],
                        (-n[0]).atan2(-n[1]),
                    )
                })
        })
}

/// The tiles `car` would take up parked at `spot` facing `yaw`.
pub(crate) fn car_tiles_at(car: &Car, spot: [f32; 2], yaw: f32) -> Vec<(i32, i32)> {
    car.tiles_at(spot, yaw)
}

/// The tiles a car takes up parked on driveway `d` (one as long as the longest car, as
/// `relocate` parks it), whether or not one is there now.
pub(crate) fn drive_tiles(d: &Drive) -> Vec<(i32, i32)> {
    const LEN: f32 = 4.8;
    const WID: f32 = 1.9;
    let Some([ax, _]) = d.way() else {
        return covered_tiles(
            d.x - WID / 2.0,
            d.z - WID / 2.0,
            d.x + WID / 2.0,
            d.z + WID / 2.0,
        );
    };
    let [x, z] = d.spot(LEN);
    // (Along x or along z: a town's driveways run square to its plots.)
    let (hx, hz) = if ax.abs() > 0.5 {
        (LEN / 2.0, WID / 2.0)
    } else {
        (WID / 2.0, LEN / 2.0)
    };
    covered_tiles(x - hx, z - hz, x + hx, z + hz)
}

/// Car `c` parks in front of a garage the player built at the end of its driveway
/// (`garage_spot`), or back where it parked before once that's gone (`Car::drive_spot`): while it
/// stands parked with nobody setting off in it, or while it's away at work (it comes home to its
/// new spot). Not where something stands in front of the garage. True when it moved.
pub(crate) fn park_by_garage(w: &mut World, c: usize) -> bool {
    let car = &w.cars[c];
    let idle = match car.state {
        CarState::Parked => {
            car.driver.is_none()
                && car.aboard.is_empty()
                && !w.sims.iter().any(|s| s.bound_for_car == Some(c as u32))
        }
        CarState::Away => car.trip.is_none(),
        _ => false,
    };
    if !idle {
        return false;
    }
    let base = car
        .drive_spot
        .map_or((car.spot, car.yaw), |[x, z, yaw]| ([x, z], yaw));
    let want = garage_spot(w, car, base).filter(|&(spot, yaw)| {
        let things = things_on(w, &[car.plot]);
        !built_over(&w.lot, &things, &car.posed(spot, yaw).tiles())
    });
    let (spot, yaw, before) = match want {
        Some((spot, yaw)) => (spot, yaw, Some([base.0[0], base.0[1], base.1])),
        None => (base.0, base.1, None),
    };
    if car.spot == spot && car.yaw == yaw && car.drive_spot == before {
        return false;
    }
    let World {
        cars,
        exits,
        travel,
        ..
    } = &mut *w;
    let car = &mut cars[c];
    car.spot = spot;
    car.yaw = yaw;
    car.drive_spot = before;
    car.route = None;
    car.settle(exits, Some(travel));
    // (Parked somewhere else: its tiles are worked out again.)
    w.cars_parked = None;
    refresh(w);
    true
}

/// The tiles the garages and the cars parked (`parked`: a bit each) take up, sorted.
pub(crate) fn standing_tiles(w: &World, parked: &[u64]) -> Vec<usize> {
    let mut tiles: Vec<usize> = Vec::new();
    for g in &w.garages {
        for (x, z) in g.tiles() {
            if w.lot.in_bounds(x, z) {
                tiles.push(w.lot.tile_index(x, z));
            }
        }
    }
    // (A porch post stands where it is too, as a garage does, on its own or a porch's: `porches.rs`.)
    let posts = w
        .porch_pieces
        .iter()
        .filter(|p| p.kind == crate::porches::PieceKind::Post)
        .map(|p| (p.x, p.z))
        .chain(w.porches.iter().flat_map(|p| p.post_tiles()));
    for (x, z) in posts {
        if w.lot.in_bounds(x, z) {
            tiles.push(w.lot.tile_index(x, z));
        }
    }
    let out = w.cars.len().div_ceil(64);
    for (c, car) in w.cars.iter().enumerate() {
        let home = parked[c / 64] & 1 << (c % 64) != 0;
        // (Out on a trip, it stands in its bay as at home.)
        let away = parked[out + c / 64] & 1 << (c % 64) != 0;
        let at = match car.out_at() {
            Some((at, yaw)) if away => car.tiles_at(at, yaw),
            _ if home => car.tiles(),
            _ => continue,
        };
        for (x, z) in at {
            if w.lot.in_bounds(x, z) {
                tiles.push(w.lot.tile_index(x, z));
            }
        }
    }
    tiles.sort_unstable();
    tiles.dedup();
    tiles
}

/// Which cars stand parked, a bit each: at home, then out on a trip.
pub(crate) fn parked_bits(w: &World) -> Vec<u64> {
    let out = w.cars.len().div_ceil(64);
    let mut parked = vec![0u64; 2 * out];
    for c in (0..w.cars.len()).filter(|&c| present(w, c)) {
        parked[c / 64] |= 1 << (c % 64);
    }
    for c in (0..w.cars.len()).filter(|&c| w.cars[c].out_at().is_some()) {
        parked[out + c / 64] |= 1 << (c % 64);
    }
    parked
}

/// A garage was built or taken down on `plot` (or put back by an undo): its car parks by it
/// (`park_by_garage`), and walking changes only on the tiles taken up or freed (`restand`).
pub(crate) fn garages_changed(w: &mut World, plot: u32) {
    for c in 0..w.cars.len() {
        if w.cars[c].plot == plot {
            park_by_garage(w, c);
        }
    }
    let parked = parked_bits(w);
    let tiles = standing_tiles(w, &parked);
    w.cars_parked = Some(parked);
    restand(w, tiles, false);
}

/// A porch post was built or taken down, on its own or a porch's (or a porch moved with its room):
/// walking changes only on their tiles (`restand`). A tile freed keeps a thing standing there
/// blocked: a town's porch may have had one at a post (`porches.rs`, `porch_kept`).
pub(crate) fn posts_changed(w: &mut World) {
    let parked = parked_bits(w);
    let tiles = standing_tiles(w, &parked);
    w.cars_parked = Some(parked);
    restand(w, tiles, true);
}

/// The most separate patches of changed tiles one `restand` notes for walkers (more: the box
/// round them all). The navigation log keeps a tick's patches together (`path::NavLog`), so
/// however many cars come and go in a tick, walkers that last looked a few ticks ago still find
/// what changed there rather than re-planning for anything.
pub(crate) const MOST_PATCHES: usize = 8;

/// Tiles of change this close together (or closer) make one patch.
pub(crate) const PATCH_GAP: i32 = 2;

/// The garages and parked cars now stand on `tiles` (sorted): walking changes only on the tiles
/// taken up or freed, each patch of them noted on its own. (One box round every car in town
/// would have every walker plan again and the walkable regions of most of a big world labelled
/// again, for one car pulling out of a driveway.)
pub(crate) fn restand(w: &mut World, tiles: Vec<usize>, things: bool) {
    // The tiles in one list and not the other (both are sorted).
    let (old, new) = (&w.car_tiles, &tiles);
    let (mut i, mut j) = (0, 0);
    let (mut freed, mut taken) = (Vec::new(), Vec::new());
    while i < old.len() || j < new.len() {
        match (old.get(i).copied(), new.get(j).copied()) {
            (Some(a), Some(b)) if a == b => {
                i += 1;
                j += 1;
            }
            (Some(a), Some(b)) if a < b => {
                freed.push(a);
                i += 1;
            }
            (Some(a), None) => {
                freed.push(a);
                i += 1;
            }
            (_, Some(b)) => {
                taken.push(b);
                j += 1;
            }
            (None, None) => break,
        }
    }
    w.car_tiles = tiles;
    if freed.is_empty() && taken.is_empty() {
        return;
    }
    for &i in &freed {
        w.blocked[i] = w.travel.ground_blocked(i);
    }
    // (A thing may stand where a porch's post stood: cars and garages never stand on one.)
    if things && !freed.is_empty() {
        let at: Vec<(i32, i32)> = freed.iter().map(|&i| w.lot.tile_xz(i)).collect();
        let under: Vec<usize> = w
            .objects
            .iter()
            .flat_map(|o| w.blocking_tiles(o))
            .filter_map(|t| at.iter().position(|&a| a == t))
            .collect();
        for k in under {
            w.blocked[freed[k]] = true;
        }
    }
    for &i in &taken {
        w.blocked[i] = true;
    }
    let changed: Vec<(i32, i32)> = freed
        .iter()
        .chain(&taken)
        .map(|&i| w.lot.tile_xz(i))
        .collect();
    for rect in patches(&changed) {
        w.nav_changed_in(rect);
    }
    w.cars_nav = w.nav_version;
}

/// Tiles `tiles` gathered into patches (`[x0, z0, x1, z1)`, in a fixed order): those within
/// `PATCH_GAP` of each other share one; past `MOST_PATCHES`, one box round them all.
pub(crate) fn patches(tiles: &[(i32, i32)]) -> Vec<[i32; 4]> {
    let near = |a: [i32; 4], b: [i32; 4]| {
        a[0] - PATCH_GAP < b[2]
            && b[0] - PATCH_GAP < a[2]
            && a[1] - PATCH_GAP < b[3]
            && b[1] - PATCH_GAP < a[3]
    };
    let mut out: Vec<[i32; 4]> = Vec::new();
    for &(x, z) in tiles {
        let mut r = [x, z, x + 1, z + 1];
        // (Joining two patches can bring a third near: gather until none is.)
        while let Some(k) = out.iter().position(|&o| near(o, r)) {
            r = join(r, out.swap_remove(k));
        }
        out.push(r);
    }
    if out.len() > MOST_PATCHES {
        let all = out.iter().copied().reduce(join).unwrap_or_default();
        return vec![all];
    }
    out
}

/// The tiles parked cars and garages take up: nobody walks through a car or a garage, and nothing is put where one stands
/// (`World::blocked`; things never stand on them, see `refresh`). Walkers find their way again
/// near where that changes (the walkable regions, not the views: `nav_version` only; `restand`).
pub(crate) fn reblock(w: &mut World) {
    // (Nothing to do while the same cars stand parked: a bit each.)
    let parked = parked_bits(w);
    if w.cars_parked.as_ref() == Some(&parked) {
        return;
    }
    // (Garages too, as they stand.)
    let tiles = standing_tiles(w, &parked);
    w.cars_parked = Some(parked);
    restand(w, tiles, false);
}
