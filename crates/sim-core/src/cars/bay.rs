//! Bays and kerbs: where a car parks, pulls over and keeps to the side.
//!
//! Split from `cars/mod.rs` (see `mod.rs`).

use super::*;

/// Where a car parks off the road, its driveway or a parking bay, in the frame of the road where
/// it meets it (`mp` on its middle line, its tangent `t` and normal `n`): where it parks (`x0`,
/// `z0` in that frame) facing `yaw`, the way it runs to the road (`u`: along its axis, else
/// square to the road), and the kerb on its side of the road (`kerb`, its `z` in that frame).
pub(crate) struct Bay {
    spot: [f32; 2],
    yaw: f32,
    mp: [f32; 2],
    pub(crate) t: [f32; 2],
    n: [f32; 2],
    x0: f32,
    z0: f32,
    u: [f32; 2],
    kerb: f32,
}

impl Bay {
    /// On `road` (its kerbs: `kerb_off`). None: it comes alongside the road rather than onto it
    /// (`OBLIQUE`).
    pub(crate) fn new(
        spot: [f32; 2],
        yaw: f32,
        axis: Option<[f32; 2]>,
        mp: [f32; 2],
        road: &crate::travel::Road,
        s_at: f32,
    ) -> Option<Bay> {
        let t = road.tangent(s_at);
        let n = [-t[1], t[0]];
        let (dx, dz) = (spot[0] - mp[0], spot[1] - mp[1]);
        let (x0, z0) = (dx * t[0] + dz * t[1], dx * n[0] + dz * n[1]);
        let s = if z0 <= 0.0 { 1.0 } else { -1.0 };
        let u = match axis {
            Some([x, z]) => {
                let u = [x * t[0] + z * t[1], x * n[0] + z * n[1]];
                if u[1] * s < OBLIQUE {
                    return None;
                }
                u
            }
            None => [0.0, s],
        };
        Some(Bay {
            spot,
            yaw,
            mp,
            t,
            n,
            x0,
            z0,
            u,
            kerb: -s * kerb_off(road),
        })
    }

    /// How far along its line from where it crosses its side's kerb it crosses the lane `lane` m
    /// off the middle line, and a little (`KERB_SLACK`): the most a turn onto (or off) that lane
    /// takes up of it, so it turns on the road, not across the sidewalk.
    fn past_kerb(&self, lane: f32) -> f32 {
        ((lane - self.kerb) / self.u[1]).max(0.0) + KERB_SLACK
    }

    pub(crate) fn local(&self, p: [f32; 2]) -> [f32; 2] {
        let (dx, dz) = (p[0] - self.mp[0], p[1] - self.mp[1]);
        [
            dx * self.t[0] + dz * self.t[1],
            dx * self.n[0] + dz * self.n[1],
        ]
    }

    fn world(&self, q: [f32; 2]) -> [f32; 2] {
        [
            self.mp[0] + q[0] * self.t[0] + q[1] * self.n[0],
            self.mp[1] + q[0] * self.t[1] + q[1] * self.n[1],
        ]
    }

    /// Where its line crosses the lane `lane` m off the middle line (along the road), and how far
    /// that is from where it parks.
    fn cross(&self, lane: f32) -> (f32, f32) {
        let k = ((lane - self.z0) / self.u[1]).max(0.0);
        (self.x0 + self.u[0] * k, k)
    }

    /// Whether `p` is a point of the lane before (or after) a turn round `at` (along the road)
    /// that takes up `r` of it, going `d` (+1: along `t`).
    pub(crate) fn near(&self, p: [f32; 2], at: f32, r: f32, d: f32) -> bool {
        (self.local(p)[0] - at) * d < r + 0.5
            && crate::travel::dist(p, self.world([at, 0.0])) < 2.0 * r + 4.0
    }

    /// Out of it onto the lane going `d` (+1: along `t`): facing the road it pulls out, facing
    /// away it backs out turning away from the way it goes (as `Route::legacy` does). Where the
    /// turn is along the road, and how much of the lane it takes up.
    pub(crate) fn leave(&self, legs: &mut Legs, d: f32) -> (f32, f32) {
        let lane_out = d * TIGHT_LANE;
        let (at, room) = self.cross(lane_out);
        let room = room.min(self.past_kerb(lane_out));
        let facing = [self.yaw.sin(), self.yaw.cos()];
        let way = [
            self.u[0] * self.t[0] + self.u[1] * self.n[0],
            self.u[0] * self.t[1] + self.u[1] * self.n[1],
        ];
        let facing_street = facing[0] * way[0] + facing[1] * way[1] > 0.0;
        let k = if facing_street { d } else { -d };
        let reverse = !facing_street;
        let turn = Turn::round([at, lane_out], self.u, [k, 0.0], room);
        legs.line(self.spot, self.world(turn.from), reverse, SLOW);
        turn.drive(legs, &|q| self.world(q), reverse);
        (at, turn.taken)
    }

    /// The turn into it from the lane, coming along it against `d`, and where it is along the road.
    pub(crate) fn entry(&self, d: f32) -> (Turn, f32) {
        let lane_in = -d * TIGHT_LANE;
        let (at, room) = self.cross(lane_in);
        let room = room.min(self.past_kerb(lane_in));
        (
            Turn::round([at, lane_in], [-d, 0.0], [-self.u[0], -self.u[1]], room),
            at,
        )
    }

    /// Into it, nose first, by `turn` (`entry`).
    pub(crate) fn enter(&self, legs: &mut Legs, turn: &Turn) {
        let from = legs.last().unwrap_or(self.world(turn.from));
        legs.line(from, self.world(turn.from), false, JUNCTION_SPEED);
        turn.drive(legs, &|q| self.world(q), false);
        legs.line(self.world(turn.to), self.spot, false, SLOW);
    }
}

/// The way a car parked in a bay at `yaw` (nose in, as the town file lays it: `park_spot`) runs
/// out of it to the road: straight back the way it faces, so it drives in along the bay's own
/// line and stands at the bay's yaw, the one drawn, even where the bay isn't quite square to the
/// road nearest it. Where that would take it away from the road, `to_road` (`parking_bay`).
pub(crate) fn bay_axis(yaw: f32, to_road: [f32; 2]) -> [f32; 2] {
    let back = [-yaw.sin(), -yaw.cos()];
    // (Within a thousandth of a radian of the way to the road, that way, as it always was: a bay
    // the town file gives the yaw the simulation would have worked out drives in as before.)
    let cos = back[0] * to_road[0] + back[1] * to_road[1];
    if cos > 0.0 && cos < (1e-3f32).cos() {
        back
    } else {
        to_road
    }
}

/// A public parking spot at `at` as a bay off the road (one of a big world's, `macro.ts`: square
/// to the road, off its verge): the point of the road it's nearest, the road's tangent there and
/// the way from the spot to it. None: no road near, or the spot is on the road itself (pulled into
/// along it).
pub(crate) fn parking_bay(
    travel: &Travel,
    at: [f32; 2],
) -> Option<([f32; 2], RoadPoint, [f32; 2])> {
    let g = &travel.graph;
    let p = *g.near(at, ROAD_REACH, 1, Mode::Car).first()?;
    let road = &g.roads[p.road as usize];
    let rp = g.point(p);
    let (dx, dz) = (rp[0] - at[0], rp[1] - at[1]);
    let l = dx.hypot(dz);
    (l > road.width / 2.0 + 0.5).then_some((rp, p, [dx / l, dz / l]))
}

/// How far from its middle line a road's kerbs are (m): its width takes in the sidewalks of a
/// street or lane (`roads.ts` `SIDEWALK`); a country road has none.
pub(crate) fn kerb_off(road: &crate::travel::Road) -> f32 {
    kerb_of(road.width, road.kind)
}

/// `kerb_off` for a road `width` m wide of `kind`.
pub(crate) fn kerb_of(width: f32, kind: crate::travel::RoadKind) -> f32 {
    match kind {
        crate::travel::RoadKind::Country | crate::travel::RoadKind::Trail => width / 2.0,
        _ => (width / 2.0 - SIDEWALK).max(TIGHT_LANE + 0.5),
    }
}

/// Where a car drives on a road `width` m wide of `kind` (m right of its middle line): `LANE` where
/// that leaves it `KERB_KEEP` from the kerb, else `TIGHT_LANE` (a 6 m lane, 3.3 m between its
/// kerbs: too narrow for two cars to pass either way, and not up on the kerb, as the kerbside
/// furniture stands about 0.15 m in from it). It turns out of a driveway onto the road, and off it
/// into one, `TIGHT_LANE` out as ever (`Bay`: a turn there at a bend in the road cuts the bend's
/// edge the more the further out it ends), easing out to its lane on along the road.
pub(crate) fn lane_on(width: f32, kind: crate::travel::RoadKind) -> f32 {
    if kerb_of(width, kind) - CAR_HALF - KERB_KEEP >= LANE {
        LANE
    } else {
        TIGHT_LANE
    }
}

/// How far in from the edges of a road `width` m wide of `kind` its kerbside furniture may reach
/// (m): `LANE_KERBSIDE` on a lane too narrow for two cars to pass in their lanes (`lane_on`), where
/// they pull over onto the pavement to pass; elsewhere where it always stood (`KERBSIDE`).
pub fn kerbside(width: f32, kind: crate::travel::RoadKind) -> f32 {
    if lane_on(width, kind) == TIGHT_LANE {
        LANE_KERBSIDE
    } else {
        KERBSIDE
    }
}

/// The streets whose kerbside furniture keeps back from the kerb (`kerbside`): every lane too
/// narrow for two cars to pass in their lanes, where they pull over onto the pavement to pass. The
/// town file's road (`meta.roads` index) and how far in from its edges the furniture may reach (m),
/// by road: for the renderer's street dressing (the world's structure, `pullOverStreets`).
pub fn pull_over_streets(travel: &Travel) -> Vec<(u32, f32)> {
    use crate::travel::RoadKind;
    let mut out: Vec<(u32, f32)> = travel
        .graph
        .roads
        .iter()
        .filter(|r| !matches!(r.kind, RoadKind::Country | RoadKind::Trail))
        .filter_map(|r| {
            let keep = kerbside(r.width, r.kind);
            (keep < KERBSIDE).then_some((r.source, keep))
        })
        .collect();
    out.sort_by_key(|&(r, _)| r);
    out.dedup_by_key(|&mut (r, _)| r);
    out
}

/// How far right of its lane (`lane_on`) a car `wid` m wide can pull over on a road `width` m wide
/// of `kind` (m, `PULL_MOST` at most): to `PULL_EDGE` short of the kerbside furniture
/// (`kerbside`), or of a country road's edge. On a 6 m lane two cars pulled over pass each other.
pub(crate) fn pull_room(width: f32, kind: crate::travel::RoadKind, wid: f32) -> f32 {
    use crate::travel::RoadKind;
    let edge = match kind {
        RoadKind::Country | RoadKind::Trail => width / 2.0,
        _ => width / 2.0 - kerbside(width, kind),
    } - PULL_EDGE;
    (edge - wid / 2.0 - lane_on(width, kind)).clamp(0.0, PULL_MOST)
}
