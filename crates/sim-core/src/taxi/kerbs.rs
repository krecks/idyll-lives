//! Kerbs: where a taxi can stop, and how to get to one.

use super::*;

/// How far either side of a taxi's body the road it stands on is kept straight and clear of a
/// junction's box (m), and how far it may turn over that stretch at most (rad: about 4°, the
/// body parallel to its lane all along).
pub(super) const STAND_ROOM: f32 = 1.0;
const STAND_STRAIGHT: f32 = 0.07;
/// How far along the road from a rider's driveway mouth the taxi may stand for them at most (m),
/// tried this often (m) nearest first.
const STAND_REACH: f32 = 30.0;
const STAND_STEP: f32 = 0.5;
/// Room between two taxis standing on one stretch (m, between their bodies).
pub(super) const STAND_GAP: f32 = 3.0;
/// How far from a way out of town along its road a taxi doesn't stand (m).
const STAND_OUT: f32 = 20.0;
/// How far the claim of a taxi standing at the kerb reaches before and after its middle, past half
/// its length (m): those behind it, or turning out of a driveway by it, wait for it.
pub(super) const KERB_CLAIM: f32 = 2.0;
/// Its own lane's kerb a standing taxi keeps this far from (m: its corners too, on a lane that
/// turns a little over its length, `STAND_STRAIGHT`).
pub(super) const KERB_KEEP: f32 = 0.15;
/// Over how much of the way before it the taxi eases to where it stands (m).
pub(super) const STAND_EASE: f32 = 8.0;
/// Turning round in a dead end's turning circle (`u_turn`): how far its kerb reaches past the
/// lane's (m, `roads.ts` `TURNING`), how far over it a front corner may swing (m), the swerve's
/// radius into it and out (m), and the speed (m/s).
pub(super) const TURNING: f32 = 2.4;
pub(super) const TURN_OVER: f32 = 0.3;
pub(super) const SWERVE: f32 = 2.0;
pub(super) const TURN_SPEED: f32 = crate::cars::JUNCTION_SPEED;
/// What turning round costs the way it's weighed (s of drive time, as the drive's own:
/// `travel::Graph::path`), on top of the drive, and standing for a rider on the far side of the
/// road from their house (s: they cross it).
pub(super) const TURN_COST: f32 = 6.0;
/// What turning round in a side street costs, the way it's weighed (s of drive time: slowly in and
/// backing out).
pub(super) const SIDE_COST: f32 = 12.0;
pub(super) const FAR_SIDE: f32 = 8.0;
/// Runs coming into town by the same way are planned this far apart at least (ticks, `time_run`),
/// one sooner by as much as `ENTRY_SHIFT` for it.
pub(super) const ENTRY_GAP: u64 = 2 * TICKS_PER_MINUTE;
pub(super) const ENTRY_SHIFT: u64 = 20 * TICKS_PER_MINUTE;
/// A way in with this many cars or more queued at it (`queued`: waiting to come in, or stood
/// still within `QUEUE_NEAR` m of it) isn't taken while another will do (`route_in`), of the
/// `QUEUE_TRIES` nearest.
pub(super) const QUEUE_MOST: usize = 3;
pub(super) const QUEUE_NEAR: f32 = 40.0;
pub(super) const QUEUE_TRIES: usize = 3;
/// The most a way round a hold-up may take over the straight way (ticks, `way_for`): about as
/// long as a run waits one out (`JAM_WAIT`).
pub(super) const DETOUR_MOST: u64 = JAM_WAIT;
/// A stretch through a place cars are held up now (`jammed`) as a run's way is weighed (s of drive
/// time): driven
/// only with no other way round, a way round no longer than `DETOUR_MOST` over the straight way
/// (else that, `way_for`: a run came two hours late to its riders round a car stood still half an
/// hour on its way).
pub(super) const JAM_COST: f32 = 600.0;

/// Where a taxi stands for a halt (`kerb_at`): its middle `s` m along road `road` (the network's),
/// in its lane and parallel to it, the way it comes there deciding which side; and the rider's
/// driveway, whose mouth they wait by (`wait_spot`) and on whose side the house is.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Kerb {
    pub road: u32,
    pub s: f32,
    pub mouth: crate::travel::RoadPoint,
    /// The way the driveway runs from the house to the road (a unit vector).
    pub axis: [f32; 2],
}

/// The way a driveway runs from its inner end to the street (`cars::Drive::way`).
fn drive_axis(d: &crate::cars::Drive) -> Option<[f32; 2]> {
    let (x, z) = (d.front_x, d.front);
    let l = x.hypot(z);
    (l.is_finite() && l > 0.5).then(|| [x / l, z / l])
}

/// Where the house on `plot` meets the road its riders wait by and the way to it from the house
/// (a unit vector): its driveway's mouth, else (no driveway) the road nearest its front door,
/// where its front path comes out.
pub(super) fn home_way(w: &World, plot: u32) -> Option<(crate::travel::RoadPoint, [f32; 2])> {
    let g = &w.travel.graph;
    if let Some(d) = w.drives.iter().find(|d| d.plot == plot)
        && let Some(axis) = drive_axis(d)
        && let Some(mouth) = d
            .road
            .zip(d.s.filter(|s| s.is_finite()))
            .and_then(|(r, s)| g.at_source(r, s))
    {
        return Some((mouth, axis));
    }
    let door = w.plots.get(plot as usize)?.arrival_point();
    let mouth = *g
        .near(door, crate::cars::ROAD_REACH, 1, crate::travel::Mode::Car)
        .first()?;
    let p = g.point(mouth);
    let l = crate::travel::dist(p, door).max(1e-3);
    Some((mouth, [(p[0] - door[0]) / l, (p[1] - door[1]) / l]))
}

/// Whether end `end` (0: its start, 1: its end) of the network's road `road` is a dead end with a
/// turning circle (the town file's `ends`, `dead`): a street's or a lane's.
pub(super) fn dead_end(
    travel: &crate::travel::Travel,
    meta: &serde_json::Value,
    road: u32,
    end: usize,
) -> bool {
    use crate::travel::RoadKind;
    let r = &travel.graph.roads[road as usize];
    matches!(r.kind, RoadKind::Street | RoadKind::Lane)
        && meta["roads"][r.source as usize]["ends"][end].as_str() == Some("dead")
}

/// Whether end `end` of the network's road `road` is a way out of town (`ends`, `out`).
fn out_end(
    meta: &serde_json::Value,
    travel: &crate::travel::Travel,
    road: u32,
    end: usize,
) -> bool {
    let r = &travel.graph.roads[road as usize];
    meta["roads"][r.source as usize]["ends"][end].as_str() == Some("out")
}

/// Where the taxi stands for a house whose way to the road meets it at `mouth`, running there
/// along `axis` (`home_way`): by it, nearest first along the road, where the road runs straight
/// for the taxi's body and a little more (`STAND_ROOM`, `STAND_STRAIGHT`), clear of junctions'
/// boxes, of a dead end's turning circle, of the way out of town, of driveways' mouths (`mouths`,
/// `DRIVE_CLEAR`), of public parking (`PARK_CLEAR`), and of another taxi standing (`taken`: (road, s), with `STAND_GAP` between
/// them). None: not a road for cars, or no such stretch within `STAND_REACH`.
pub(crate) fn kerb_at(
    travel: &crate::travel::Travel,
    meta: &serde_json::Value,
    (mouth, axis): (crate::travel::RoadPoint, [f32; 2]),
    taken: &[(u32, f32)],
    mouths: &[(u32, f32)],
) -> Option<Kerb> {
    let g = &travel.graph;
    let road = &g.roads[mouth.road as usize];
    if !road.drivable() {
        return None;
    }
    let half = TAXI_LEN / 2.0 + STAND_ROOM;
    let len = road.len();
    // (From each end: past a dead end's turning circle and the swerve into it, or the way out.)
    let keep = |end: usize| {
        if dead_end(travel, meta, mouth.road, end) {
            u_turn_reach(road.width, road.kind) + TAXI_LEN
        } else if out_end(meta, travel, mouth.road, end) {
            STAND_OUT
        } else {
            0.0
        }
    };
    let (lo, hi) = (keep(0) + half, len - keep(1) - half);
    let steps = (STAND_REACH / STAND_STEP) as i32;
    // (Where a car parks at each public parking spot (`cars::parked_at`: the spot itself), of
    // those it could be near: a stand's body is within `STAND_REACH` and its half length of the
    // mouth along the road, so no nearer to it as the crow flies.)
    let near = road.at(mouth.s);
    let parked: Vec<[f32; 2]> = travel
        .file
        .parking
        .iter()
        .map(|p| [p.x, p.z])
        .filter(|at| at.iter().all(|v| v.is_finite()))
        .filter(|&at| crate::travel::dist(at, near) < STAND_REACH + half + PARK_CLEAR + 1.0)
        .collect();
    // (The junctions' boxes a stand could come near, the same way: the rest it's clear of.)
    let boxes: Vec<([f32; 2], f32)> = g
        .boxes
        .iter()
        .copied()
        .filter(|&(m, r)| {
            crate::travel::dist(m, near) < STAND_REACH + half + r + STAND_ROOM + TAXI_WID + 1.0
        })
        .collect();
    // (Clear of driveways by a car's length and a margin where it can be within reach, else
    // clear of their mouths at least: a house in a row of them still has its taxi.)
    let find = |keep: f32| {
        (0..=2 * steps)
            .map(|k| {
                mouth.s + ((k + 1) / 2) as f32 * STAND_STEP * if k % 2 == 1 { 1.0 } else { -1.0 }
            })
            .filter(|&s| s >= lo && s <= hi)
            .find(|&s| {
                // (The cheap tests first, each only for a stand the ones before let stand: the
                // junctions' boxes and the roads near are asked about last.)
                let free = || {
                    taken
                        .iter()
                        .all(|&(r, ts)| r != mouth.road || (ts - s).abs() >= TAXI_LEN + STAND_GAP)
                };
                // (Nor across a driveway's mouth, theirs too: a car's length and a little either side
                // of it, for one coming out or turning in.)
                let drives = || {
                    mouths
                        .iter()
                        .all(|&(r, m)| r != mouth.road || (m - s).abs() >= half + keep)
                };
                let n = (2.0 * half / STAND_STEP).ceil() as usize;
                // (Nor where a car parks out on a trip, nor pulls into or out of it.)
                let parking = || {
                    (0..=n).all(|i| {
                        let p = road.at(s - half + i as f32 * 2.0 * half / n as f32);
                        parked
                            .iter()
                            .all(|&at| crate::travel::dist(at, p) >= PARK_CLEAR)
                    })
                };
                let straight = || {
                    let (a, b) = (road.at(s - half), road.at(s + half));
                    let chord = (b[1] - a[1]).atan2(b[0] - a[0]);
                    (0..=n).all(|i| {
                        let t = road.tangent(s - half + i as f32 * 2.0 * half / n as f32);
                        crate::world::wrap_angle(t[1].atan2(t[0]) - chord).abs() <= STAND_STRAIGHT
                    })
                };
                // (Clear of junctions' boxes, and of another road lying along or over this one: two
                // country roads a metre apart, its stand across the other's lane; or this road
                // itself further along, come back over it: its stand across that stretch's lane.)
                let clear = || {
                    (0..=n).all(|i| {
                        let at = s - half + i as f32 * 2.0 * half / n as f32;
                        let p = road.at(at);
                        boxes
                            .iter()
                            .all(|&(m, r)| crate::travel::dist(m, p) >= r + STAND_ROOM + TAXI_WID)
                            && g.near(p, road.width, 3, crate::travel::Mode::Car)
                                .iter()
                                .all(|q| q.road == mouth.road)
                            && !g.passes_again(mouth.road, at, p, road.width, road.width + half)
                    })
                };
                free() && drives() && parking() && straight() && clear()
            })
    };
    find(DRIVE_CLEAR)
        .or_else(|| find(DRIVE_LEAST))
        .map(|s| Kerb {
            road: mouth.road,
            s,
            mouth,
            axis,
        })
}

#[cfg(test)]
mod kerb_at_tests {
    use super::*;

    /// A town of one street that runs east from x 100 and doubles back (a hairpin at x 220): its
    /// way back lies over its way out, a few metres off, as far as x 100.
    fn hairpin() -> World {
        let mut pts: Vec<String> = (0..=120).map(|k| format!("[{},30.0]", 100 + k)).collect();
        pts.extend((1..=6).map(|k| format!("[220.0,{}]", 30 + k)));
        pts.extend(
            (1..=120).map(|k| format!("[{},{:.2}]", 220 - k, 36.0 - 3.0 * k as f32 / 120.0)),
        );
        let road = format!(
            r#"{{"points":[{}],"width":7,"kind":"street","ends":["out","dead"]}}"#,
            pts.join(",")
        );
        let town = format!(
            r#"{{"width":300,"depth":80,"size":"normal","exits":[{{"x":100.0,"z":30.0,"kind":"road"}}],
            "roads":[{road}],"meta":{{"roads":[{road}]}},"plots":[],"households":[],"sims":[]}}"#
        );
        World::from_json(
            r#"{"needs":[{"id":"social","label":"Social","decayPerHour":0.02}],"objects":[]}"#,
            &town,
            1,
        )
        .unwrap()
    }

    #[test]
    fn a_taxi_doesnt_stand_where_its_street_comes_back_over_itself() {
        // By the street's way out, its way back a few metres off: no stand there, its body
        // across that stretch's lane (Oakwood, v4 s11: 8 degrees across, stood over a loop of
        // its own road; one road, so "another road along or over it" let it be).
        let w = hairpin();
        let g = &w.travel.graph;
        let mouth = *g
            .near([160.0, 30.0], 2.0, 1, crate::travel::Mode::Car)
            .first()
            .expect("the street");
        let road = &g.roads[mouth.road as usize];
        let back: Vec<[f32; 2]> = (1..=120)
            .map(|k| [220.0 - k as f32, 36.0 - 3.0 * k as f32 / 120.0])
            .collect();
        if let Some(k) = kerb_at(&w.travel, &w.meta, (mouth, [0.0, -1.0]), &[], &[]) {
            let half = TAXI_LEN / 2.0;
            for d in [-half, 0.0, half] {
                let p = road.at(k.s + d);
                let near = back
                    .iter()
                    .map(|&q| crate::travel::dist(p, q))
                    .fold(f32::INFINITY, f32::min);
                assert!(
                    near >= road.width,
                    "a stand at s {:.1}, {near:.1} m from the street's way back",
                    k.s
                );
            }
        }
    }
}
