//! Where cars are drawn and what they are in the way of: bodies, passing and touching.
//!
//! Split from `cars.rs` (see `mod.rs`).

use super::*;

/// Furthest (ticks along its way: 15 s of drive time) a car the camera came to on top of another
/// is put to come into sight clear of it (`moved_clear`, a tick at a time).
pub(crate) const MOVED_MOST: u32 = drive_secs(15.0) as u32;

/// Which of `cars` aren't drawn (`snapshot::car`): those out of sight (`Sight`: none is drawn
/// driving through the others), and any on top of one of those or touching it (`TOUCH`: at the
/// seen area's edge, out of the picture, so no two cars are ever drawn on top of each other).
pub fn hidden(cars: &[Car]) -> Vec<bool> {
    let mut out: Vec<bool> = cars.iter().map(|c| c.sight != Sight::Seen).collect();
    if !out.contains(&true) {
        return out;
    }
    let body = |c: &Car| {
        c.shown.map(|s| Rect {
            at: s.0,
            yaw: s.1,
            half: [c.len / 2.0, c.wid / 2.0],
        })
    };
    // (Out of sight by where their middles are, along x: a car seen need only look at those whose
    // middles are near enough along x to touch it, `reach` apart at most. That's the same test as
    // looking at every one of them, without the pair of every car in town.)
    let mut out_of_sight: Vec<Rect> = cars
        .iter()
        .filter(|c| c.sight != Sight::Seen)
        .filter_map(body)
        .collect();
    out_of_sight.sort_by(|a, b| a.at[0].total_cmp(&b.at[0]));
    let reach = out_of_sight.iter().map(|o| o.half[0]).fold(0.0, f32::max);
    for (k, car) in cars.iter().enumerate() {
        if out[k] {
            continue;
        }
        let Some(r) = body(car) else {
            continue;
        };
        let most = reach + r.half[0] + 0.5;
        out[k] = along_x(&out_of_sight, |o| o.at[0], r.at[0], most)
            .iter()
            .any(|o| {
                crate::travel::dist(o.at, r.at) < o.half[0] + r.half[0] + 0.5 && o.gap(&r) < TOUCH
            });
    }
    out
}

/// The cars on the road this tick, for the claims (`traffic::begin`): those driving (`places`:
/// where along their routes, by their routes' tags: home or not), and those in sight (`movers`:
/// their bodies, the way they go, and if they moved last tick where they'll be along their way;
/// one still in its driveway, not yet holding its claim out, as parked), and the lights those in
/// sight are coming up to (`approaching`: within `traffic::APPROACH` of a group's start through
/// them).
pub(crate) fn movers(
    w: &mut World,
    movers: &mut Vec<crate::traffic::Mover>,
    places: &mut Vec<crate::traffic::Place>,
    approaching: &mut Vec<crate::traffic::Gate>,
) {
    let World {
        cars,
        exits,
        travel,
        traffic,
        ..
    } = w;
    for (c, car) in cars.iter_mut().enumerate() {
        // (Out of sight, it claims nothing and nobody keeps out of its way: `sight`.)
        if !matches!(car.state, CarState::Leaving | CarState::Arriving) || car.sight != Sight::Seen
        {
            continue;
        }
        let key = car_first(c);
        let home = car.state == CarState::Arriving;
        places.push((key, u32::from(home), car.progress));
        let Some((p, yaw, true, _)) = car.shown else {
            continue;
        };
        let k = if car.backing() { -1.0 } else { 1.0 };
        let half = [car.len / 2.0, car.wid / 2.0];
        let parked = !home && car.progress <= 0.0 && !traffic.holding(key);
        let (t, shift, moved) = (car.progress, car.shift, car.moved);
        let route = car.route(exits, Some(travel), home);
        if let Some(gate) = route
            .claims
            .iter()
            .take_while(|k| k.from <= t + crate::traffic::APPROACH)
            .find(|k| k.from > t + 1e-3 && k.gate.is_some())
            .and_then(|k| k.lights(t).0)
        {
            approaching.push(gate);
        }
        let ahead = if moved {
            crate::traffic::LOOK
                .iter()
                .filter(|&&dt| t + dt < route.ticks())
                .map(|&dt| {
                    let (q, qy, _) = route.at(t + dt);
                    Rect {
                        at: shifted(q, qy, shift),
                        yaw: qy,
                        half,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        // (On the move, as far as it will pull over on its way: not into a junction's box or a
        // narrow span just ahead, as `drive_on` sees it. Standing, waiting at a box's edge, as far
        // as it can where it stands: those coming out of the box pass it there.)
        const AHEAD_TOO: [f32; 3] = [0.0, drive_ticks(0.5), drive_ticks(1.0)];
        let ahead_too: &[f32] = if moved { &AHEAD_TOO } else { &[0.0] };
        let pass = (ahead_too
            .iter()
            .map(|&k| route.pass_at(t + k))
            .fold(f32::INFINITY, f32::min)
            - shift)
            .max(0.0);
        movers.push(crate::traffic::Mover {
            key,
            rect: Rect { at: p, yaw, half },
            way: [yaw.sin() * k, yaw.cos() * k],
            pass,
            moving: moved,
            ahead,
            parked,
            waits: car.waits_for,
        });
    }
}

/// Whether car `c`, out of sight at the way in (`Car::outside`), comes in now: none there longer
/// waits to come in by the same way (the earlier car, of two there as long), and its body a tick
/// on (by `GAP`) and the ground in front of it (`AHEAD`) are clear of every car in town (`others`,
/// `vehicles`: those that came in this tick where they are now) and every car parked.
pub(crate) fn room_to_come_in(w: &mut World, c: usize, others: &[Driving]) -> bool {
    let World {
        cars,
        exits,
        travel,
        ..
    } = w;
    let (at, yaw, _) = cars[c].route(exits, Some(travel), true).at(1.0);
    let start = |car: &Car| car.route.as_ref().filter(|r| r.0).map(|r| r.1.at(0.0).0);
    let mine = start(&cars[c]);
    let before = cars.iter().enumerate().any(|(o, other)| {
        o != c
            && other.outside()
            && (other.since, o) < (cars[c].since, c)
            && start(other)
                .zip(mine)
                .is_some_and(|(a, b)| crate::travel::dist(a, b) < 4.0)
    });
    let me = Driving {
        at,
        yaw,
        len: cars[c].len,
        wid: cars[c].wid,
        ahead: AHEAD,
        behind: 0.0,
        first: car_first(c),
    };
    let (body, front) = (me.body(), me.front());
    !before
        && others
            .iter()
            .copied()
            .chain(parked(cars))
            .all(|o| o.body().gap(&body) >= GAP && o.body().overlap(&front) <= 0.0)
}

/// A car driving where it is now, for who's in its way (`go_on`): its middle and yaw, its length
/// and width, and how far in front of it and behind it someone is in its way (m, past its ends);
/// and which it is (`first`: `car_first`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Driving {
    pub at: [f32; 2],
    pub yaw: f32,
    pub len: f32,
    pub wid: f32,
    pub ahead: f32,
    pub behind: f32,
    pub first: u64,
}

impl Driving {
    /// The ground from its middle to `ahead` past its front (or, backing up, `behind` past its
    /// back), as wide as it is: anything there is in its way (`up_against`). Its middle, its yaw
    /// and its half length and width.
    fn front(&self) -> Rect {
        let (s, c) = self.yaw.sin_cos();
        let (reach, k) = if self.ahead > 0.0 || self.behind <= 0.0 {
            (self.len / 2.0 + self.ahead, 1.0)
        } else {
            (self.len / 2.0 + self.behind, -1.0)
        };
        let along = k * reach / 2.0;
        Rect {
            at: [self.at[0] + s * along, self.at[1] + c * along],
            yaw: self.yaw,
            half: [reach / 2.0, self.wid / 2.0],
        }
    }

    /// How far `body` at `pos` is from it and the ground in front of it (behind it, backing up):
    /// below 0, how far into them (`Body::gap`).
    fn gap(&self, body: Body, pos: [f32; 2]) -> f32 {
        body.gap(
            self.at,
            self.yaw.sin_cos(),
            pos,
            -(self.len / 2.0 + self.behind),
            self.len / 2.0 + self.ahead,
            self.wid / 2.0,
        )
    }

    /// Its body.
    pub(crate) fn body(&self) -> Rect {
        Rect {
            at: self.at,
            yaw: self.yaw,
            half: [self.len / 2.0, self.wid / 2.0],
        }
    }

    /// Whether `d` waits for it at all: not while it backs up, out of a driveway or into one; of
    /// two tangled, the earlier (`first`) doesn't wait for the later, but the later for it, so the
    /// earlier pulls ahead and they part (going on together, neither waiting, they'd stay on top of
    /// each other).
    pub(crate) fn free(&self, d: &Driving) -> bool {
        let tangled = self.body().overlap(&d.body()) > 0.0;
        self.ahead > 0.0 && !(tangled && d.first < self.first)
    }

    /// The way it's going (a unit vector: backwards, backing up).
    pub(crate) fn heading(&self) -> [f32; 2] {
        let (s, c) = self.yaw.sin_cos();
        let k = if self.ahead > 0.0 || self.behind <= 0.0 {
            1.0
        } else {
            -1.0
        };
        [s * k, c * k]
    }
}

/// A rectangle on the ground: its middle, its yaw (its length along (sin, cos)), and its half
/// length and width (m).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Rect {
    pub at: [f32; 2],
    pub yaw: f32,
    pub half: [f32; 2],
}

impl Rect {
    /// How far the two overlap (m, the least along any of their sides' ways; 0 or less: apart).
    pub(crate) fn overlap(&self, o: &Rect) -> f32 {
        let axes = |yaw: f32| {
            let (s, c) = yaw.sin_cos();
            [[s, c], [c, -s]]
        };
        let (a, b) = (axes(self.yaw), axes(o.yaw));
        let d = [o.at[0] - self.at[0], o.at[1] - self.at[1]];
        let reach = |r: &Rect, ax: &[[f32; 2]; 2], u: [f32; 2]| {
            r.half[0] * (ax[0][0] * u[0] + ax[0][1] * u[1]).abs()
                + r.half[1] * (ax[1][0] * u[0] + ax[1][1] * u[1]).abs()
        };
        a.iter()
            .chain(b.iter())
            .map(|&u| reach(self, &a, u) + reach(o, &b, u) - (d[0] * u[0] + d[1] * u[1]).abs())
            .fold(f32::INFINITY, f32::min)
    }

    /// The gap between the two (m: the least distance from a corner of either to a side of the
    /// other; below 0, how far they overlap, `overlap`).
    pub(crate) fn gap(&self, o: &Rect) -> f32 {
        let over = self.overlap(o);
        if over > 0.0 {
            return -over;
        }
        let corners = |r: &Rect| {
            let (s, c) = r.yaw.sin_cos();
            let (f, side) = (
                [s * r.half[0], c * r.half[0]],
                [c * r.half[1], -s * r.half[1]],
            );
            [(1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (-1.0, 1.0)].map(|(a, b)| {
                [
                    r.at[0] + a * f[0] + b * side[0],
                    r.at[1] + a * f[1] + b * side[1],
                ]
            })
        };
        let to_side = |p: [f32; 2], a: [f32; 2], b: [f32; 2]| {
            let ab = [b[0] - a[0], b[1] - a[1]];
            let t = ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1])
                / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-9);
            let t = t.clamp(0.0, 1.0);
            (p[0] - a[0] - ab[0] * t).hypot(p[1] - a[1] - ab[1] * t)
        };
        let (a, b) = (corners(self), corners(o));
        let mut best = f32::INFINITY;
        for (ps, qs) in [(&a, &b), (&b, &a)] {
            for &p in ps {
                for k in 0..4 {
                    best = best.min(to_side(p, qs[k], qs[(k + 1) % 4]));
                }
            }
        }
        best
    }
}

/// Whether a car driving (`d`, where it would be a tick on) comes up against another of `them`
/// (`vehicles`, as they are now): one whose body is in the ground in front of it
/// (`Driving::front`), going its way. It keeps behind it. (One coming the other way passes it in
/// the other lane, both pulled over where the lane is too narrow for both: `pull_over`.) It waits
/// only behind one in front of it going its way, driving on (not backing up), so whoever waits
/// for one waits for something that moves on.
pub(crate) fn up_against(d: &Driving, them: &[Driving]) -> bool {
    against(d, them).next().is_some()
}

/// Those of `them` (indices) `d` comes up against (`up_against`).
pub(crate) fn against<'a>(d: &'a Driving, them: &'a [Driving]) -> impl Iterator<Item = usize> + 'a {
    // (Backing up, out of a driveway or into one, a car waits for nobody but those on foot.)
    let backing = d.ahead <= 0.0;
    let (front, heading) = (d.front(), d.heading());
    them.iter()
        .enumerate()
        .filter(move |(_, o)| {
            let way = o.heading();
            let cos = heading[0] * way[0] + heading[1] * way[1];
            if backing || cos < ONCOMING || !o.free(d) {
                return false;
            }
            let reach = d.len + d.ahead.max(d.behind) + o.len;
            if (o.at[0] - d.at[0]).hypot(o.at[1] - d.at[1]) > reach
                || front.overlap(&o.body()) <= 0.0
            {
                return false;
            }
            true
        })
        .map(|(k, _)| k)
}

/// Another driving towards something (the cosine of the angle between their ways, less than
/// minus this) is coming the other way (`up_against`, `passing`, `too_near`).
pub(crate) const ONCOMING: f32 = 0.7;

/// Two coming towards each other, measured along and across the way between theirs (the same
/// for both: on a bend either may look to be off the other's right): how far the second is along
/// it from the first; how far apart their middles are across it (the second on the first's left:
/// more), and the gap there is between their bodies (m: less than none, they may overlap); and
/// how far across it each moves pulling over a metre (`pull_want`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Passing {
    pub along: f32,
    pub apart: f32,
    pub gap: f32,
    pub mine: f32,
    pub theirs: f32,
}

/// `Passing` for `a` and `b` (each where, facing which way, how long and wide), if they're coming
/// towards each other (else none).
pub(crate) fn passing(
    a: ([f32; 2], f32, f32, f32),
    b: ([f32; 2], f32, f32, f32),
) -> Option<Passing> {
    let (ahead, theirs) = ([a.1.sin(), a.1.cos()], [b.1.sin(), b.1.cos()]);
    if ahead[0] * theirs[0] + ahead[1] * theirs[1] >= -ONCOMING {
        return None;
    }
    let u = [ahead[0] - theirs[0], ahead[1] - theirs[1]];
    let n = u[0].hypot(u[1]).max(1e-6);
    let (u, left) = ([u[0] / n, u[1] / n], [u[1] / n, -u[0] / n]);
    let d = [b.0[0] - a.0[0], b.0[1] - a.0[1]];
    let (along, apart) = (d[0] * u[0] + d[1] * u[1], d[0] * left[0] + d[1] * left[1]);
    // (How far each reaches across that way, turned as it is; and moves, pulling over to its
    // right.)
    let reach = |way: [f32; 2], len: f32, wid: f32| {
        let k = way[0] * left[0] + way[1] * left[1];
        len / 2.0 * k.abs() + wid / 2.0 * (1.0 - k * k).max(0.0).sqrt()
    };
    let pulls = |way: [f32; 2]| (way[0] * u[0] + way[1] * u[1]).abs().max(0.3);
    Some(Passing {
        along,
        apart,
        gap: apart.abs() - reach(ahead, a.2, a.3) - reach(theirs, b.2, b.3),
        mine: pulls(ahead),
        theirs: pulls(theirs),
    })
}

/// Whether `a` (where it would be a tick on) would come too near `b` coming the other way: nearer
/// its body than `GAP`, as the guard has it (`Rect::gap`). (Pulled over as far as `passing` says,
/// they're further apart than that: it measures across a way between theirs.)
pub(crate) fn too_near(a: ([f32; 2], f32, f32, f32), b: ([f32; 2], f32, f32, f32)) -> bool {
    let (ahead, theirs) = ([a.1.sin(), a.1.cos()], [b.1.sin(), b.1.cos()]);
    let body = |(at, yaw, len, wid): ([f32; 2], f32, f32, f32)| Rect {
        at,
        yaw,
        half: [len / 2.0, wid / 2.0],
    };
    ahead[0] * theirs[0] + ahead[1] * theirs[1] < -ONCOMING && body(a).gap(&body(b)) < GAP
}

/// Someone's body as cars keep clear of it, and it of them: how far it reaches in front
/// of where they are and behind them, and out to each side (m), facing `yaw`. On foot, `BODY`
/// all round; on a bike, its wheels in front and behind and its handlebars to the sides (content
/// `vehicle.reach`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Body {
    pub ahead: f32,
    pub behind: f32,
    pub side: f32,
    pub yaw: f32,
}

impl Body {
    /// Someone on foot.
    pub(crate) const WALKER: Body = Body {
        ahead: BODY,
        behind: BODY,
        side: BODY,
        yaw: 0.0,
    };

    /// `sim`'s, facing `yaw`: on foot, or on what they ride.
    pub(crate) fn of(
        sim: &Sim,
        content: &crate::content::Content,
        objects: &[crate::world::ObjectInstance],
        yaw: f32,
    ) -> Body {
        let ride = sim
            .riding
            .and_then(|o| objects.get(o as usize))
            .and_then(|o| content.objects[o.def].vehicle);
        let Some([ahead, behind, side]) = ride.map(|v| v.reach) else {
            return Body {
                yaw,
                ..Body::WALKER
            };
        };
        Body {
            ahead,
            behind,
            side,
            yaw,
        }
    }

    /// How far it is, at `pos`, from the box `back..front` along and `half` either side across
    /// of something at `at` turned `turn` (its yaw's sine and cosine): below 0, how far into it.
    fn gap(
        self,
        at: [f32; 2],
        turn: (f32, f32),
        pos: [f32; 2],
        back: f32,
        front: f32,
        half: f32,
    ) -> f32 {
        // (Points along its middle a fifth of a metre apart at most, as far as its ends less
        // its sides: round each, as far as its sides.)
        let (s, c) = self.yaw.sin_cos();
        let a = (self.ahead - self.side).max(0.0);
        let b = (self.behind - self.side).max(0.0);
        let n = ((a + b) / 0.2).ceil() as usize;
        let (mid, long) = ((front + back) / 2.0, (front - back) / 2.0);
        (0..=n)
            .map(|k| {
                let t = -b + (a + b) * k as f32 / n.max(1) as f32;
                let (along, across) = local(at, turn, [pos[0] + s * t, pos[1] + c * t]);
                let (dx, dz) = ((along - mid).abs() - long, across.abs() - half);
                dx.max(0.0).hypot(dz.max(0.0)) + dx.max(dz).min(0.0) - self.side
            })
            .fold(f32::INFINITY, f32::min)
    }
}

/// Something driving (`d`, where it would be a tick on) a tick further along its way, unless
/// someone is in its way: it waits for them (`waited`, ticks). Someone standing there a while (a
/// chat on the pavement) steps aside for it (`make_way`); someone with nowhere to step goes
/// round to where they're out of its way, past `PASS_TICKS` (`clear_of`). It never drives
/// through anyone. Someone at something or at a wheel isn't in its way; nor is anyone whose body (`Body`, on foot or on a bike) clears its sides by
/// `BESIDE`, walking by it or along beside it; nor anyone the tick on brings it no nearer (`now`:
/// where it is, as `d`; nearer measured from its whole body, its corners and ends as it turns, and
/// the ground ahead of it), as someone beside it waits for it (`walks_into`): they'd hold each
/// other up. Nor does it drive into another car in front of it (`up_against`): it waits behind it
/// (true), not counted in `waited`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn go_on(
    sims: &mut [Sim],
    nav: &crate::path::NavGrid,
    bodies: (&crate::content::Content, &[crate::world::ObjectInstance]),
    d: Driving,
    now: Driving,
    others: &[Driving],
    waited: &mut u32,
    progress: &mut f32,
    whom: &mut Vec<u32>,
) -> bool {
    let turn = d.yaw.sin_cos();
    whom.clear();
    // (Nobody further off than its corners and the ground ahead of or behind them, and the
    // longest bike, is in its way.)
    let reach = d.len / 2.0 + d.ahead.max(d.behind) + d.wid / 2.0 + BESIDE + 2.0;
    let mut waiting = false;
    for s in sims.iter_mut() {
        let (dx, dz) = (s.pos[0] - d.at[0], s.pos[1] - d.at[1]);
        if dx * dx + dz * dz >= reach * reach || !s.here() || s.away_until.is_some() {
            continue;
        }
        let body = Body::of(s, bodies.0, bodies.1, s.yaw);
        let phase = s.current().map(|a| &a.phase);
        // (Standing at the kerb for the taxi, they won't step out in front of it: in its way only
        // where its body goes, not the ground ahead.)
        let (ahead, behind) = if matches!(phase, Some(Phase::AtKerb { .. })) {
            (0.0, 0.0)
        } else {
            (d.ahead, d.behind)
        };
        // (Beside it, no nearer a tick on, clear of its body and the ground ahead: they wait for
        // it. Never into anyone. Nearer by under a millimetre is only rounding, as it drives on
        // alongside them: held up there a tick in two, it would keep pace with them.)
        let next = d.gap(body, s.pos);
        if !covers(d.at, turn, d.len, d.wid, body, s.pos, ahead, behind)
            || next >= 0.0 && next >= now.gap(body, s.pos) - 1e-3
        {
            continue;
        }
        if matches!(phase, Some(Phase::Using { .. }) | Some(Phase::Driving)) {
            continue;
        }
        waiting = true;
        whom.push(s.id);
        // (Walking on across its way, not waiting for it: they're on their way out of it, not
        // to be stepped back where they came from. Not one walking along its way, ahead of it or
        // towards it: they'd hold it up all the way, so they step aside in the end as anyone.)
        if matches!(phase, Some(Phase::Routing { .. }))
            && s.car_wait == 0
            && (s.yaw - d.yaw).sin().abs() >= ALONG
        {
            continue;
        }
        // (Not into the way of another, nor where this one comes on to: `clear`.)
        let clear = |p: [f32; 2]| clear(others, body, p);
        // (Stepping aside that long hasn't got them out of its way.)
        let round = (*waited >= PASS_TICKS)
            .then(|| clear_of(&d, nav, body, s.pos, clear))
            .flatten();
        if let Some(p) = round.or_else(|| {
            (*waited > MAKE_WAY_TICKS)
                .then(|| make_way(d.at, d.yaw, nav, s.pos, STEP_ASIDE, clear))
                .flatten()
        }) {
            s.pos = p;
        }
    }
    let blocked = up_against(&d, others);
    if waiting {
        *waited = waited.saturating_add(1);
    } else {
        // (Held up most ticks, by one walking ahead of it on the road and on again: it still
        // counts, so they step aside for it in the end, `make_way`.)
        *waited = waited.saturating_sub(2);
        if !blocked {
            *progress += 1.0;
        }
    }
    blocked
}

/// Every car driving by, as `walks_into` keeps clear of them (its body and the ground just in front
/// of it), but car `car`.
pub(crate) fn vehicles(cars: &[Car], car: Option<usize>) -> Vec<Driving> {
    let at = |(at, yaw): ([f32; 2], f32), len: f32, wid: f32, first: u64| Driving {
        at,
        yaw,
        len,
        wid,
        ahead: AHEAD,
        behind: 0.0,
        first,
    };
    // (For one in sight, those in sight: `sight`; one out of sight keeps out of nobody's way.)
    if car.is_some_and(|c| cars[c].sight != Sight::Seen) {
        return Vec::new();
    }
    let cars = cars.iter().enumerate().filter_map(|(c, k)| match k.shown {
        Some((p, yaw, true, _)) if Some(c) != car && (car.is_none() || k.sight == Sight::Seen) => {
            let mut d = at((p, yaw), k.len, k.wid, car_first(c));
            if k.backing() {
                (d.ahead, d.behind) = (0.0, AHEAD);
            }
            Some(d)
        }
        _ => None,
    });
    cars.collect()
}

/// Every car parked where it can be seen (at home, out on a trip): as `vehicles` has those on the
/// road, but standing.
pub(crate) fn parked(cars: &[Car]) -> impl Iterator<Item = Driving> + '_ {
    cars.iter().enumerate().filter_map(|(c, k)| match k.shown {
        // (A taxi standing out of sight, out of sight: `sight`.)
        Some((at, yaw, false, _)) if k.sight == Sight::Seen => Some(Driving {
            at,
            yaw,
            len: k.len,
            wid: k.wid,
            ahead: AHEAD,
            behind: 0.0,
            first: car_first(c),
        }),
        _ => None,
    })
}

pub fn car_key(c: usize) -> u64 {
    car_first(c)
}

/// Car `c`'s `Driving::first` (its top bit set, as ever).
pub(crate) fn car_first(c: usize) -> u64 {
    (1 << 63) | c as u64
}

/// Whether `body` at `p` is out of the way of all of `them` (`vehicles`): somewhere to step aside
/// to, or to go round to, for someone in the way of another.
pub(crate) fn clear(them: &[Driving], body: Body, p: [f32; 2]) -> bool {
    them.iter().all(|d| {
        !covers(
            d.at,
            d.yaw.sin_cos(),
            d.len,
            d.wid,
            body,
            p,
            d.ahead,
            d.behind,
        )
    })
}

/// Someone at `pos` in the way of something driving (`d`) with nowhere to step aside: the middle
/// of the nearest tile they can walk to, `CLEAR_STEPS` at most, where their `body` is out of its
/// way, and (`clear`) of any other's. None: nowhere that near (it waits on).
pub(crate) fn clear_of(
    d: &Driving,
    nav: &crate::path::NavGrid,
    body: Body,
    pos: [f32; 2],
    clear: impl Fn([f32; 2]) -> bool,
) -> Option<[f32; 2]> {
    let turn = d.yaw.sin_cos();
    let start = (pos[0].floor() as i32, pos[1].floor() as i32);
    let mut seen = vec![start];
    let mut ring = vec![start];
    for _ in 0..CLEAR_STEPS {
        let mut next = Vec::new();
        for &(x, z) in &ring {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let t = (x + dx, z + dz);
                if !seen.contains(&t) && nav.can_step(x, z, t.0, t.1) {
                    seen.push(t);
                    next.push(t);
                }
            }
        }
        let far = |p: &[f32; 2]| (p[0] - pos[0]).hypot(p[1] - pos[1]);
        let clear = next
            .iter()
            .map(|&(x, z)| [x as f32 + 0.5, z as f32 + 0.5])
            .filter(|&p| !covers(d.at, turn, d.len, d.wid, body, p, d.ahead, d.behind) && clear(p))
            .min_by(|a, b| far(a).total_cmp(&far(b)));
        if clear.is_some() {
            return clear;
        }
        ring = next;
    }
    None
}

/// `pos` from something at `at` turned `turn` (its yaw's sine and cosine): how far along it,
/// ahead, and how far across it.
pub(crate) fn local(at: [f32; 2], turn: (f32, f32), pos: [f32; 2]) -> (f32, f32) {
    let (s, c) = turn;
    let (dx, dz) = (pos[0] - at[0], pos[1] - at[1]);
    (dx * s + dz * c, dx * c - dz * s)
}

/// Whether `body` at `pos` comes within `BESIDE` of something `len` by `wid` m (a car)
/// at `at`, turned `turn` (its yaw's sine and cosine), or of the ground `ahead` in front of it
/// or `behind` behind it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn covers(
    at: [f32; 2],
    turn: (f32, f32),
    len: f32,
    wid: f32,
    body: Body,
    pos: [f32; 2],
    ahead: f32,
    behind: f32,
) -> bool {
    body.gap(
        at,
        turn,
        pos,
        -(len / 2.0 + behind),
        len / 2.0 + ahead,
        wid / 2.0,
    ) < BESIDE
}

/// Whether a step from `from` to `to` with `body` (turned the way of the step) walks into
/// something `len` by `wid` m driving by at `at` facing `yaw`, within `KEEP` of it, or into the
/// ground `ahead` in front of it that it keeps clear (`AHEAD`): stepping out of its way is fine.
/// (Stopped there, waiting for another to pass, they'd hold it up as it held them.)
#[allow(clippy::too_many_arguments)]
pub(crate) fn walks_into(
    at: [f32; 2],
    yaw: f32,
    len: f32,
    wid: f32,
    ahead: f32,
    body: Body,
    from: [f32; 2],
    to: [f32; 2],
) -> bool {
    let turn = yaw.sin_cos();
    let body = Body {
        yaw: (to[0] - from[0]).atan2(to[1] - from[1]),
        ..body
    };
    let gap = |p: [f32; 2]| body.gap(at, turn, p, -len / 2.0, len / 2.0 + ahead, wid / 2.0);
    let g = gap(to);
    g < KEEP && g <= gap(from)
}

/// Whether `body` at `p` is in the way of a car (`walks_into`'s ground: its body and the ground in
/// front of it, `KEEP` round).
pub(crate) fn in_way_of(
    at: [f32; 2],
    yaw: f32,
    len: f32,
    wid: f32,
    body: Body,
    p: [f32; 2],
) -> bool {
    body.gap(
        at,
        yaw.sin_cos(),
        p,
        -len / 2.0,
        len / 2.0 + AHEAD,
        wid / 2.0,
    ) < KEEP
}

/// Getting into `car` (`inward`, `Phase::Boarding`) or out of it (`Phase::Alighting`) by their
/// door, `t` ticks in: while it opens at the door going in (sat in their seat going out), then a
/// step between the two, and in, sat while it shuts.
pub(crate) fn getting(sim: &mut Sim, car: &Car, t: u32, inward: bool) {
    let d = usize::from(sim.car_door).min(DOORS - 1);
    let (door, seat) = (car.door_at(d), car.seat_at(usize::from(sim.car_seat), d));
    let k = (t.saturating_sub(DOOR_OPENS) as f32 / STEP_IN as f32).min(1.0);
    if t < DOOR_OPENS || inward && k >= 1.0 {
        if inward == (t >= DOOR_OPENS) {
            sim.pos = seat;
            sim.yaw = car.yaw;
            sim.pose = crate::content::Pose::Sit;
        }
        return;
    }
    let (a, b) = if inward { (door, seat) } else { (seat, door) };
    sim.pos = [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k];
    sim.yaw = (b[0] - a[0]).atan2(b[1] - a[1]);
    sim.pose = crate::content::Pose::Stand;
}

/// The car driving by that a step from `from` to `to` with `body` would walk into (stepping out
/// of its way is fine).
pub(crate) fn in_the_way(
    cars: &[Car],
    id: u32,
    body: Body,
    from: [f32; 2],
    to: [f32; 2],
) -> Option<usize> {
    // (Already in the way of one waiting for those in its way: they go on out of it, where
    // they're going, as it waits; those not yet in it wait for it, so it gets its gap.)
    let holds_up = |car: &Car| {
        car.waited > 0
            && car.shown.is_some_and(|(at, yaw, on, _)| {
                on && in_way_of(at, yaw, car.len, car.wid, body, from)
            })
    };
    // (Holding one up so, on out of its way past the nose of one standing still, waiting its
    // turn: into its body never, but not waiting for it to come on, as that one waits for the
    // first, which waits for them.)
    let out = cars.iter().any(holds_up);
    cars.iter().position(|car| {
        let Some((at, yaw, true, _)) = car.shown else {
            return false;
        };
        if holds_up(car) {
            return false;
        }
        // (One it waits for by name (`Car::on_foot`) doesn't wait for it to come on, nor for where
        // it backs up to: only never into its body. Behind one backing out of a bay, a cyclist
        // waited for it as it waited for them, for good, Swanmere.)
        if car.on_foot.contains(&id) {
            return walks_into(at, yaw, car.len, car.wid, 0.0, body, from, to);
        }
        // (Nor the ground ahead of one waiting at a red light, its nose at the stop line: someone
        // on the crosswalk just in front of it waited there for it, on the walk, till the walk
        // was over and it came on at them.)
        let ahead = if out && !car.moved || car.at_red {
            0.0
        } else {
            AHEAD
        };
        walks_into(at, yaw, car.len, car.wid, ahead, body, from, to)
            || car.next.is_some_and(|(at, yaw)| {
                walks_into(at, yaw, car.len, car.wid, ahead, body, from, to)
            })
    })
}

/// Whether resident `id` (with `body`, at `pos`) holds up a car driving by: it waits for them
/// (`Car::on_foot`), or they stand in the way of one waiting for those in its way (`in_the_way`'s
/// `holds_up`). They're to go on out of its way, not stand there: at a crosswalk closed to them
/// (`traffic::kerb_stop`) the two waited for each other for good.
pub(crate) fn holding_up(cars: &[Car], id: u32, body: Body, pos: [f32; 2]) -> bool {
    cars.iter().any(|car| {
        car.shown.is_some_and(|(at, yaw, on, _)| {
            on && (car.on_foot.contains(&id)
                || car.waited > 0 && in_way_of(at, yaw, car.len, car.wid, body, pos))
        })
    })
}

/// Someone has waited a while for a car (at `at`, facing `yaw`) to pass, standing in its way: a
/// step aside (across the way it drives, away from it), where there's ground to stand on and it
/// isn't into another's way (`clear`). None: nowhere to go.
pub(crate) fn make_way(
    at: [f32; 2],
    yaw: f32,
    nav: &crate::path::NavGrid,
    pos: [f32; 2],
    step: f32,
    clear: impl Fn([f32; 2]) -> bool,
) -> Option<[f32; 2]> {
    let (s, c) = yaw.sin_cos();
    let left = [c, -s];
    let across = (pos[0] - at[0]) * left[0] + (pos[1] - at[1]) * left[1];
    let k = if across >= 0.0 { 1.0 } else { -1.0 };
    let (fx, fz) = (pos[0].floor() as i32, pos[1].floor() as i32);
    [k, -k]
        .into_iter()
        .map(|k| [pos[0] + left[0] * step * k, pos[1] + left[1] * step * k])
        .find(|p| {
            let (x, z) = (p[0].floor() as i32, p[1].floor() as i32);
            // (A step to the next tile at most.)
            let near = (x - fx).abs() <= 1 && (z - fz).abs() <= 1;
            nav.tile_free(x, z)
                && near
                && ((x, z) == (fx, fz) || nav.can_step(fx, fz, x, z))
                && clear(*p)
        })
}

/// Whether car `c` stands on its household's driveway, to be seen and used (not built over).
pub(crate) fn present(w: &World, c: usize) -> bool {
    let car = &w.cars[c];
    car.at_home()
        && !car.blocked
        && w.households
            .get(car.household as usize)
            .is_some_and(|h| h.plot == Some(car.plot))
}

/// Where every car is now and which doors are open, for the snapshot (`Car::shown`): the doors
/// of those getting in, and of those who just got in or out.
pub(crate) fn poses(w: &mut World) {
    let mut doors = vec![0u8; w.cars.len()];
    for s in &w.sims {
        // Open from when they're at it until they're sat in (or out and stood up).
        let open = s.current().is_some_and(|a| match a.phase {
            Phase::Boarding { ticks } => ticks < DOOR_OPENS + STEP_IN,
            Phase::Alighting { ticks } => ticks < GET_OUT_TICKS,
            _ => false,
        });
        if let Some(c) = s.bound_for_car.filter(|_| open)
            && let Some(bits) = doors.get_mut(c as usize)
        {
            *bits |= 1 << s.car_door.min(DOORS as u8 - 1);
        }
    }
    let present: Vec<bool> = (0..w.cars.len()).map(|c| present(w, c)).collect();
    let World {
        cars,
        exits,
        travel,
        ..
    } = w;
    for (c, car) in cars.iter_mut().enumerate() {
        let open = doors[c];
        car.shown = if car.at_home() && !present[c] {
            None
        } else {
            car.pose_on(exits, Some(travel))
                .map(|(p, yaw, moving)| (p, yaw, moving, if moving { 0 } else { open }))
        };
        if !car.shown.is_some_and(|s| s.2) {
            car.next = None;
        }
    }
}

/// Those done at work for the car's way home stay away until it's back (`until`: the time shown
/// for them).
pub(crate) fn keep_away(w: &mut World, c: usize, until: u64) {
    let until = until.max(w.tick + 1);
    for &s in &w.cars[c].done {
        w.sims[s as usize].away_until = Some(until);
    }
}
