//! Taking turns where cars' and buses' ways cross or meet (`World::traffic`): reservations, not
//! give-way rules, cars and buses alike in one queue.
//!
//! **Zones and claims.** Where two ways may cross or meet so that both can't go on, there's a
//! zone (`Travel::zones`): each junction's box (`boxes`: a 6 m circle round its middle, reaching
//! on along an arm as far as needed to keep a body in its lane there `ARMS_APART` from one on
//! another arm close to it), and where a bus's body sweeps out of its lane round a sharp turn or
//! turning round (`sweeps`, circles round its body as it turns). Each route (a car's,
//! `cars::Route`; a bus line's, `bus::Line`) is looked at once, as it's laid out, for the zones it
//! drives through, and for stretches that are claims of their own: a driveway's, a bay's or a spot
//! on the road's mouth (a car turning out across the lane or in), and where it couldn't pass one
//! coming the other way, both pulled over as far as they can (`Narrow`: a car's or a bus's way
//! where their bodies, side by side or nose to nose round a bend, still come too near; a 6 m lane's
//! tight bend for two cars). Each is a part (`Claim`): the stretch of the route from just before
//! its body first touches it (`Claim::from`; a bus `bus::BUS_LEAD` before a zone, clear of where
//! one coming out of it the other way goes on) to where it's out of it again (`Claim::to`),
//! sampled every `SAMPLE` m (more often turning) with its room on the far side (its length and
//! `ROOM_GAP`). Parts its body comes to before it's out of the last make a group
//! (`Claim::group`): it can't stand clear between them, so it asks for them together. A bus's
//! group is cut where it stands at a stop clear of every junction's box: it asks for the rest
//! only as it leaves the stop.
//!
//! **Asking.** A car or bus about to drive into a group asks for it (`Traffic::may_go`) and waits
//! where it is till it's granted, all of it at once. The asks are granted once a tick
//! (`Traffic::begin`, before the cars and the buses go on), in the order they were asked (the
//! tick, then `cars::Driving::first`), each when
//! 1. none of it is in the way (`conflict`) of what's left of a part another holds,
//! 2. nor of an earlier ask still waiting,
//! 3. and nobody not holding a part is in it now, nor anyone at all (a bus not waiting for a
//!    claim itself included) on its far side ("don't block the box"), nor anyone on the move
//!    into it in the next second or so (`LOOK`).
//!
//! Two bodies are in each other's way (`conflict`) when they overlap or come within `CLEAR`; going
//! the same way (one following the other through, each keeping behind the one in front:
//! `cars::up_against`) they aren't; coming towards each other they are where pulling over as far as
//! both can there (`Sample::pass`) doesn't leave `PASS_GAP` between them. Two buses are never in
//! each other's way here (their own rules keep them apart: the signal box, pulling over, backing
//! up: `bus.rs`), but for one waiting on the far side of another's group. One right behind
//! another in its lane (`behind`) can't get anywhere before that one does: neither its parts nor
//! its ask hold that one up there.
//!
//! **Holding.** Holding a group, a vehicle drives through it, letting go of each part as its
//! body's out of it. Anyone else, holding a group or not, keeps `KEEP_SHORT` short of what's left
//! of one another holds (`Traffic::blocked`; holding one itself, it may follow another through
//! one going its way; already in another's way, it goes on out of it).
//!
//! **Why nobody waits for good.** As a tick begins, who waits for whom is a graph: the vehicles'
//! waits on the road last tick (`Mover::waits`, `cars::Car::waits_for`, the buses'
//! `bus::held_up_by`), and the asks' waits for parts held, bodies and earlier asks. Every rule
//! that makes one wait for another leaves out one that waits for it, however far round
//! (`reaches`): an earlier ask doesn't hold up a later one it waits for itself (rule 2), a body
//! that waits for the asker doesn't (rule 3), a car holding a part that waits for the asker's body
//! doesn't (rule 1), nor in `Traffic::blocked`. So the grants never close a ring of waits the
//! graph has in it: a ring forms only through waits it can't see (a pull-over, a turn sweeping
//! wider than its samples), and those are broken where they form: a car and one in its way
//! waiting for each other a while, the car backs up a little out of its way, never into anyone
//! (`cars::back_off`). An ask older than `OLD_ASK` yields only to one it waits for itself, not to
//! a stream round about: nobody waits on and on while others come and go. Should two bodies come
//! to overlap anyway, a last-resort check keeps every car from moving deeper into another or a
//! bus (`cars::guard`, counted in `Traffic::guarded`).
//!
//! Nothing here is saved: the zones and the claims are worked out again with the routes, and a
//! vehicle found in a part it doesn't hold (a game just loaded) takes it as it goes on.

use crate::cars::Rect;
use crate::travel::{Graph, dist};
use crate::world::World;

/// How far apart a route's samples are along it (m), and how far it turns between two at most
/// (radians: a car's ends swing round about 0.2 m).
pub(crate) const SAMPLE: f32 = 1.0;
const TURN_SAMPLE: f32 = 0.08;
/// The room a vehicle wants clear on the far side of a claim: its length, and this (m).
pub(crate) const ROOM_GAP: f32 = 1.0;
/// Two coming towards each other pass where pulling over takes them this far apart (m: `bus`'s
/// `PASSING` and a little).
pub(crate) const PASS_GAP: f32 = 0.08;
/// Two going within about 25° of each other's way go the same way (the cosine).
const SAME_WAY: f32 = 0.9;
/// Bodies this near (m) are in each other's way (a body's corners between two samples too).
const CLEAR: f32 = 0.25;
/// One on the move, looked ahead this many ticks along its way (`Traffic::begin`): it isn't made
/// to stop short in front of one just granted.
pub(crate) const LOOK: [f32; 3] = [10.0, 20.0, 30.0];
/// How far along its way a vehicle's ask is its own business with one right behind it (m,
/// `behind`): that one can't get there first. Further on (a bus's way round and back) it may.
const NEAR_AHEAD: f32 = 40.0;
/// A junction's box: as far from its middle as two bodies in their lanes on any two of its arms
/// are this far apart (m), between `BOX_LEAST` and `BOX_MOST`; its arms looked along this far (m).
const ARMS_APART: f32 = 4.6;
const BOX_LEAST: f32 = 6.0;
const BOX_MOST: f32 = 14.0;
const ARM_LOOK: f32 = 5.0;
/// A box reaching further along an arm than `BOX_LEAST` (one near another) does so in circles
/// this wide (m) round the arm, this far apart along it (m).
const ARM_REACH: f32 = 2.5;
const ARM_STEP: f32 = 2.0;

/// A place along a route: when it's there (ticks along it), how far along it is (m), its body
/// there, the way it goes (a unit vector: backwards, backing up), and how far it can pull over
/// there to pass one coming the other way (m: `passes`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Sample {
    pub t: f32,
    pub d: f32,
    pub rect: Rect,
    pub way: [f32; 2],
    pub pass: f32,
}

/// A stretch of a route a vehicle holds before driving into it (`from`, ticks along it) till it's
/// out of its last zone (`to`), sampled on past that over its room on the far side.
#[derive(Debug, Clone, Default)]
pub(crate) struct Claim {
    pub from: f32,
    pub to: f32,
    /// Its group: parts of a route its body comes to before it's out of the one before (`claims`).
    pub group: u32,
    samples: Vec<Sample>,
    /// The box round all its samples' bodies (x0, z0, x1, z1).
    bounds: [f32; 4],
    /// The traffic lights its group goes through, and the arm it comes in by (`Travel::signals`,
    /// `gate`): it's asked for only on that arm's green.
    pub gate: Option<(u16, u8)>,
}

impl Claim {
    fn new(from: f32, to: f32, samples: Vec<Sample>) -> Claim {
        let mut b = [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ];
        for s in &samples {
            let r = reach(&s.rect);
            b = [
                b[0].min(s.rect.at[0] - r),
                b[1].min(s.rect.at[1] - r),
                b[2].max(s.rect.at[0] + r),
                b[3].max(s.rect.at[1] + r),
            ];
        }
        Claim {
            from,
            to,
            group: 0,
            samples,
            bounds: b,
            gate: None,
        }
    }

    /// What's left of it for one `t` ticks along its route: the samples from there on (from the
    /// one before: with the next, its body there now).
    fn rest(&self, t: f32) -> &[Sample] {
        let k = self.samples.partition_point(|s| s.t < t);
        &self.samples[k.saturating_sub(1)..]
    }
}

/// How far from its middle a body reaches at most (m).
fn reach(r: &Rect) -> f32 {
    r.half[0].hypot(r.half[1])
}

/// A route's poses every `SAMPLE` m (`TURN_SAMPLE` turning): when (ticks along it), where, which
/// way it faces, whether it's backing up, and how far along it it is (m).
pub(crate) type Trace = Vec<(f32, [f32; 2], f32, bool, f32)>;

/// `Trace` of a route `ticks` long whose pose `t` ticks along is `at(t)`.
pub(crate) fn trace(at: impl Fn(f32) -> ([f32; 2], f32, bool), ticks: f32) -> Trace {
    let (p, yaw, back) = at(0.0);
    let mut out: Trace = vec![(0.0, p, yaw, back, 0.0)];
    let (mut last, mut along, mut since, mut turned) = (p, 0.0f32, 0.0f32, yaw);
    let mut t = 0.0f32;
    while t < ticks {
        t = (t + 0.5).min(ticks);
        let (p, yaw, back) = at(t);
        let d = dist(last, p);
        along += d;
        since += d;
        last = p;
        if since >= SAMPLE
            || crate::world::wrap_angle(yaw - turned).abs() >= TURN_SAMPLE
            || t >= ticks
        {
            out.push((t, p, yaw, back, along));
            (since, turned) = (0.0, yaw);
        }
    }
    out
}

/// How far a body `len` long swings out on a bend at each of `trace`'s poses (m: its ends, on the
/// inside of the bend, half its length squared over twice the bend's radius, the bend measured
/// over its length). Two coming towards each other on a bend pass only where pulling over makes
/// up for the more of theirs (`Narrow`; their bodies' overlap there has it already, `conflict`).
pub(crate) fn swings(trace: &Trace, len: f32) -> Vec<f32> {
    let n = trace.len();
    (0..n)
        .map(|k| {
            let d = trace[k].4;
            let a = trace[..k].partition_point(|s| s.4 < d - len / 2.0);
            let b = (k + trace[k..].partition_point(|s| s.4 <= d + len / 2.0)).min(n - 1);
            let span = (trace[b].4 - trace[a].4).max(0.5);
            let bend = crate::world::wrap_angle(trace[b].2 - trace[a].2).abs() / span;
            len * len / 8.0 * bend
        })
        .collect()
}

/// A route's claims (`Claim`), its parts in order: for a body `len` by `wid` along `trace` (how far
/// it can pull over at each pose, `pass`), the zones it drives through (`zones`: junctions' boxes,
/// where a bus sweeps round a sharp turn; waited for `lead` m short of them), and `more` (stretches
/// of it that are parts of their own: a driveway's mouth, a bus's way where a car can't pass it, a
/// car's where it couldn't pass one coming the other way). Parts its body comes to before it's out
/// of the last make one group (`Claim::group`: it can't stand clear of them between); `cuts`
/// (ticks along it) split a group where it stands clear of every junction's box (a bus at a stop).
#[allow(clippy::too_many_arguments)]
pub(crate) fn claims(
    trace: &Trace,
    pass: &[f32],
    len: f32,
    wid: f32,
    zones: &[([f32; 2], f32)],
    more: &[(f32, f32)],
    lead: f32,
    cuts: &[f32],
) -> Vec<Claim> {
    let n = trace.len();
    if n == 0 {
        return Vec::new();
    }
    let end = trace[n - 1].0;
    let body = |k: usize| {
        let (_, p, yaw, ..) = trace[k];
        Rect {
            at: p,
            yaw,
            half: [len / 2.0, wid / 2.0],
        }
    };
    let mut spans: Vec<(f32, f32, bool)> = more
        .iter()
        .copied()
        .filter(|&(a, b)| b > a && a < end)
        .map(|(a, b)| (a, b, false))
        .collect();
    let r0 = len.hypot(wid) / 2.0;
    let touching =
        |k: usize, c: [f32; 2], r: f32| dist(trace[k].1, c) <= r + r0 && touches(&body(k), c, r);
    for &(c, r) in zones {
        spans.extend(
            runs(trace, |k| touching(k, c, r))
                .into_iter()
                .map(|(a, b)| (a, b, true)),
        );
    }
    spans.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    // (How far along the route `t` is, m.)
    let along = |t: f32| trace[trace.partition_point(|s| s.0 < t).min(n - 1)].4;
    // The parts (one inside another's is part of it), and their groups.
    let mut parts: Vec<(f32, f32, u32, bool)> = Vec::new();
    for (a, b, zone) in spans {
        match parts.last().copied() {
            Some(last) if b <= last.1 => parts.last_mut().expect("just looked").3 |= zone,
            Some(last) if along(a) < along(last.1) + SAMPLE => {
                if a <= last.0 + 1e-3 {
                    let l = parts.last_mut().expect("just looked");
                    (l.1, l.3) = (b, l.3 | zone);
                } else {
                    parts.push((a, b, last.2, zone));
                }
            }
            last => parts.push((a, b, last.map_or(0, |l| l.2 + 1), zone)),
        }
    }
    // A group begun with a zone, waited for `lead` m short of it (clear of where one coming out of
    // it the other way goes on): one with the group before, where that's no further on than that.
    let back = |t: f32| {
        let k = trace.partition_point(|s| s.0 < t).min(n - 1);
        trace[trace.partition_point(|s| s.4 < trace[k].4 - lead).min(k)].0
    };
    let orig: Vec<u32> = parts.iter().map(|p| p.2).collect();
    let (mut id, mut end) = (0u32, f32::NEG_INFINITY);
    for k in 0..parts.len() {
        let (a, b, _, zone) = parts[k];
        if k > 0 && orig[k] != orig[k - 1] {
            // (A new group: begun short of its zone, one with the last where that reaches it.)
            let a = if zone { back(a).max(parts[k - 1].0) } else { a };
            if along(a) >= along(end) + SAMPLE {
                id += 1;
            }
            parts[k].0 = a;
        } else if k == 0 && zone {
            parts[k].0 = back(a);
        }
        parts[k].2 = id;
        end = if k > 0 && parts[k - 1].2 == id {
            end.max(b)
        } else {
            b
        };
    }
    // Cut where it stands clear (renumbering the groups after).
    let mut out: Vec<(f32, f32, u32, bool)> = Vec::new();
    let (mut id, mut was) = (0, None);
    for (a, b, g, zone) in parts {
        if was.is_some_and(|w| w != g) {
            id += 1;
        }
        was = Some(g);
        let cut = cuts
            .iter()
            .copied()
            .filter(|&c| c > a + 1e-3 && c < b - 1e-3)
            .fold(None, |m: Option<f32>, c| Some(m.map_or(c, |m| m.min(c))));
        match cut {
            Some(c) => {
                // (Asking for the rest only once it's on its way again: `CUT_ON` after.)
                out.push((a, c, id, zone));
                id += 1;
                if c + CUT_ON < b {
                    out.push((c + CUT_ON, b, id, zone));
                }
            }
            None => out.push((a, b, id, zone)),
        }
    }
    let room = len + ROOM_GAP;
    out.into_iter()
        .map(|(from, to, group, _)| {
            let far = along(to) + room;
            let samples = (0..n)
                .filter(|&k| trace[k].0 >= from && trace[k].4 <= far)
                .map(|k| {
                    let (_, _, yaw, back, d) = trace[k];
                    let (s, c) = yaw.sin_cos();
                    let k_way = if back { -1.0 } else { 1.0 };
                    Sample {
                        t: trace[k].0,
                        d,
                        rect: body(k),
                        way: [s * k_way, c * k_way],
                        pass: pass.get(k).copied().unwrap_or(0.0),
                    }
                })
                .collect();
            let mut c = Claim::new(from, to, samples);
            c.group = group;
            c
        })
        .collect()
}

/// Whether a body `len` by `wid` is in a junction's box (`boxes`) at each of `trace`'s poses.
pub(crate) fn in_boxes(trace: &Trace, len: f32, wid: f32, boxes: &[([f32; 2], f32)]) -> Vec<bool> {
    let r0 = len.hypot(wid) / 2.0;
    trace
        .iter()
        .map(|&(_, p, yaw, ..)| {
            let body = Rect {
                at: p,
                yaw,
                half: [len / 2.0, wid / 2.0],
            };
            boxes
                .iter()
                .any(|&(c, r)| dist(p, c) <= r + r0 && touches(&body, c, r))
        })
        .collect()
}

/// The stretches of `trace` where `on` (its poses' indices) holds: from the pose before the first
/// to the one after the last (ticks along it).
pub(crate) fn runs(trace: &Trace, on: impl Fn(usize) -> bool) -> Vec<(f32, f32)> {
    let n = trace.len();
    let mut out = Vec::new();
    let mut k = 0;
    while k < n {
        if !on(k) {
            k += 1;
            continue;
        }
        let first = k;
        while k + 1 < n && on(k + 1) {
            k += 1;
        }
        let from = if first == 0 { 0.0 } else { trace[first - 1].0 };
        let to = trace.get(k + 1).map_or(trace[n - 1].0, |s| s.0);
        out.push((from, to));
        k += 1;
    }
    out
}

/// Whether body `r` comes within `rad` of `c`.
fn touches(r: &Rect, c: [f32; 2], rad: f32) -> bool {
    let (s, co) = r.yaw.sin_cos();
    let d = [c[0] - r.at[0], c[1] - r.at[1]];
    let along = (d[0] * s + d[1] * co).abs() - r.half[0];
    let across = (d[0] * co - d[1] * s).abs() - r.half[1];
    along.max(0.0).hypot(across.max(0.0)) <= rad
}

/// The road network's junctions' boxes (`Graph::boxes`): each junction's middle, and how far round
/// it its box reaches: as far out along its arms as two bodies in their lanes on any two of them
/// are `ARMS_APART` apart (`BOX_LEAST` to `BOX_MOST`).
pub fn boxes(g: &Graph) -> Vec<([f32; 2], f32)> {
    let mut out = Vec::new();
    for (n, &p) in g.nodes.iter().enumerate() {
        let mut arms: Vec<[f32; 2]> = Vec::new();
        for e in g.edges.iter() {
            let road = &g.roads[e.road as usize];
            for (end, s, k) in [(e.a, e.s0, 1.0f32), (e.b, e.s1, -1.0)] {
                if end as usize != n {
                    continue;
                }
                let q = road.at((s + k * ARM_LOOK.min(e.len())).clamp(e.s0, e.s1));
                let l = dist(p, q);
                if l > 0.1 {
                    arms.push([(q[0] - p[0]) / l, (q[1] - p[1]) / l]);
                }
            }
        }
        if arms.len() < 3 {
            continue;
        }
        // (Its middle, and on out along an arm as far as that one needs: one near another.)
        out.push((p, BOX_LEAST));
        for a in &arms {
            let r = arms
                .iter()
                .filter(|b| *b != a)
                .map(|b| {
                    let cos = a[0] * b[0] + a[1] * b[1];
                    let sin = (1.0 - cos * cos).max(0.0).sqrt();
                    if cos > 0.0 {
                        ARMS_APART / sin.max(1e-3)
                    } else {
                        0.0
                    }
                })
                .fold(BOX_LEAST, f32::max)
                .min(BOX_MOST);
            let mut d = BOX_LEAST;
            while d < r {
                d = (d + ARM_STEP).min(r);
                let q = [p[0] + a[0] * (d - ARM_REACH), p[1] + a[1] * (d - ARM_REACH)];
                out.push((q, ARM_REACH));
            }
        }
    }
    out
}

/// A junction with traffic lights (`Travel::signals`: where `SIGNAL_ARMS` roads or more meet and
/// `SIGNAL_LINES` bus lines or more come through, `bus::settle`): its middle, and its arms. One arm
/// at a time has green (`Light`); a group through it is asked for only on its arm's green
/// (`Claim::gate`), at the stop line where its claim begins, and holding it, one goes on through
/// on amber and red; the claims keep anyone from driving into another in the box as ever.
#[derive(Debug, Clone)]
pub struct Signal {
    pub at: [f32; 2],
    pub arms: Vec<SignalArm>,
}

/// An arm of a lit junction (`Signal`): the way out along it (a unit vector), and its signal head:
/// the pole on the kerb to the right of the way in, a little out past the box, and which way it
/// faces (as a resident's yaw: towards those coming in).
#[derive(Debug, Clone, Copy)]
pub struct SignalArm {
    pub way: [f32; 2],
    pub pole: [f32; 2],
    pub yaw: f32,
}

/// What a lit junction's lights show (`Light`): an arm's green, its amber, then all red while the
/// box clears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stage {
    #[default]
    Green,
    Amber,
    Clear,
}

/// A lit junction's lights now (`Traffic::lights`): whose turn it is (`SignalArm` index), how far
/// through it, and since when (the tick).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Light {
    pub arm: u8,
    pub stage: Stage,
    pub since: u64,
}

/// A junction where this many roads for cars meet, and this many bus lines come through, has
/// traffic lights (`Signal`).
pub(crate) const SIGNAL_ARMS: usize = 3;
pub(crate) const SIGNAL_LINES: usize = 2;
/// How long an arm's green lasts at least, its amber, and all red after while the box
/// clears: a bus across it out the far side (ticks).
const MIN_GREEN: u64 = 5 * crate::TICKS_PER_MINUTE;
/// Every arm's turn comes round in this long at most (ticks): an arm's green lasts no longer than
/// its share of it (`change_lights`).
const CYCLE: u64 = 50 * crate::TICKS_PER_MINUTE;
const AMBER: u64 = 2 * crate::TICKS_PER_MINUTE;
const ALL_RED: u64 = 4 * crate::TICKS_PER_MINUTE;
/// A route this near a lit junction's middle goes through its lights (m, `gate`).
const GATE_REACH: f32 = BOX_MOST + 2.0;
/// How far out past its arm's box a signal pole stands, and in from the kerb on the pavement (or
/// the road's edge) (m).
const POLE_OUT: f32 = 1.0;
const POLE_KERB: f32 = 0.4;

/// The lights of the junction at node `n` of `g` (`Signal`): its arms for cars, each with its pole
/// on the kerb to the right of the way in, just out past the box (`boxes`), moved on out along the
/// arm clear of `clear` (bus stops: `POLE_CLEAR`). None: fewer than `SIGNAL_ARMS` arms.
pub(crate) fn signal_at(g: &Graph, n: usize, clear: &[[f32; 2]]) -> Option<Signal> {
    let p = g.nodes[n];
    let mut arms: Vec<([f32; 2], f32)> = Vec::new();
    // (Each arm's way out, and how far from its middle its kerb is.)
    for e in g.edges.iter() {
        let road = &g.roads[e.road as usize];
        if !road.drivable() {
            continue;
        }
        for (end, s, k) in [(e.a, e.s0, 1.0f32), (e.b, e.s1, -1.0)] {
            if end as usize != n {
                continue;
            }
            let q = road.at((s + k * ARM_LOOK.min(e.len())).clamp(e.s0, e.s1));
            let l = dist(p, q);
            if l > 0.1 {
                arms.push((
                    [(q[0] - p[0]) / l, (q[1] - p[1]) / l],
                    crate::cars::kerb_of(road.width, road.kind),
                ));
            }
        }
    }
    if arms.len() < SIGNAL_ARMS {
        return None;
    }
    // (Arms by the way they go round, for the lights' turns.)
    arms.sort_by(|a, b| a.0[0].atan2(a.0[1]).total_cmp(&b.0[0].atan2(b.0[1])));
    let reach = |a: [f32; 2]| {
        arms.iter()
            .filter(|b| b.0 != a)
            .map(|b| {
                let cos = a[0] * b.0[0] + a[1] * b.0[1];
                let sin = (1.0 - cos * cos).max(0.0).sqrt();
                if cos > 0.0 {
                    ARMS_APART / sin.max(1e-3)
                } else {
                    0.0
                }
            })
            .fold(BOX_LEAST, f32::max)
            .min(BOX_MOST)
    };
    let arms = arms
        .iter()
        .map(|&(way, kerb)| {
            // (Coming in, its right is the way out's left.)
            let right = [way[1], -way[0]];
            let at = |d: f32| {
                let side = kerb + POLE_KERB;
                [
                    p[0] + way[0] * d + right[0] * side,
                    p[1] + way[1] * d + right[1] * side,
                ]
            };
            let mut d = reach(way) + POLE_OUT;
            while d < reach(way) + POLE_OUT + 12.0
                && clear.iter().any(|&c| dist(c, at(d)) < POLE_CLEAR)
            {
                d += 1.0;
            }
            SignalArm {
                way,
                pole: at(d),
                yaw: way[0].atan2(way[1]),
            }
        })
        .collect();
    Some(Signal { at: p, arms })
}

/// How far from a bus stop a signal pole stands at least (m, `signal_at`).
pub const POLE_CLEAR: f32 = 4.0;

/// The lights `claims`' groups go through (`Claim::gate`): of `signals`, the first each group
/// comes this near (`GATE_REACH`), and the arm it comes in by.
pub(crate) fn gate(claims: &mut [Claim], signals: &[Signal]) {
    let mut k = 0;
    while k < claims.len() {
        let g = claims[k].group;
        let end = k + claims[k..].iter().take_while(|c| c.group == g).count();
        let gate = claims[k..end]
            .iter()
            .flat_map(|c| &c.samples)
            .find_map(|s| {
                signals.iter().enumerate().find_map(|(i, sig)| {
                    let d = [s.rect.at[0] - sig.at[0], s.rect.at[1] - sig.at[1]];
                    (d[0].hypot(d[1]) < GATE_REACH).then(|| {
                        let arm = (0..sig.arms.len())
                            .max_by(|&a, &b| {
                                let dot = |x: usize| {
                                    sig.arms[x].way[0] * d[0] + sig.arms[x].way[1] * d[1]
                                };
                                dot(a).total_cmp(&dot(b))
                            })
                            .unwrap_or(0);
                        (i as u16, arm as u8)
                    })
                })
            });
        for c in &mut claims[k..end] {
            c.gate = gate;
        }
        k = end;
    }
}

/// How far a bus's way turns over its own length (radians) for its body to sweep out of its lane
/// round a sharp turn outside a junction's box (`sweeps`): a car right behind it would be in the
/// way of its tail swinging round.
const SWEEP_TURN: f32 = 0.8;
/// How far apart the circles round a bus's body sweeping round are (m, `sweeps`).
const SWEEP_STEP: f32 = 1.5;

/// The zones where a bus's body sweeps out of its lane round a sharp turn outside any junction's
/// box (`boxes`), along `trace` for a body `len` by `wid` (its line's way): round all its body's
/// corners there (a middle, and how far round it), with `CLEAR` to spare. Cars and buses take
/// turns through them as through a junction's box (`World::travel`'s `zones`).
pub(crate) fn sweeps(
    trace: &Trace,
    len: f32,
    wid: f32,
    boxes: &[([f32; 2], f32)],
) -> Vec<([f32; 2], f32)> {
    let n = trace.len();
    let boxed = in_boxes(trace, len, wid, boxes);
    let sharp = |k: usize| {
        let d = trace[k].4;
        let a = trace[..k].partition_point(|s| s.4 < d - len / 2.0);
        let b = (k + trace[k..].partition_point(|s| s.4 <= d + len / 2.0)).min(n - 1);
        !boxed[k] && crate::world::wrap_angle(trace[b].2 - trace[a].2).abs() > SWEEP_TURN
    };
    // (Round its body as it goes, every `SWEEP_STEP` m along it, at each pose, in circles across
    // its width.)
    let mut out: Vec<([f32; 2], f32)> = Vec::new();
    let r = wid / 2.0 + CLEAR;
    for (from, to) in runs(trace, sharp) {
        for s in trace.iter().filter(|s| s.0 >= from && s.0 <= to) {
            let (sn, cs) = s.2.sin_cos();
            let n = (len / SWEEP_STEP).ceil() as usize;
            for k in 0..=n {
                let d = -len / 2.0 + len * k as f32 / n as f32;
                let c = [s.1[0] + sn * d, s.1[1] + cs * d];
                if !out.iter().any(|&(q, _)| dist(q, c) < SWEEP_STEP / 2.0) {
                    out.push((c, r));
                }
            }
        }
    }
    out
}

/// The buses' ways (`bus::settle`), for where a car coming the other way couldn't pass a bus, both
/// pulled over: their places about every `SAMPLE` m, the way the bus goes there, how far it can
/// pull over (none stood at a stop), how far its ends swing out on a bend (`swings`), and how far
/// a car in its lane coming the other way overlaps it in its own (m), by the `NARROW_CELL` m square
/// they're in. A car's route along one where they couldn't pass claims it (`Narrow::spans`), as the
/// bus's does (`bus::Line::claims`): they take turns there.
#[derive(Debug, Clone, Default)]
pub struct Narrow {
    cells: std::collections::BTreeMap<(i32, i32), Vec<NarrowPlace>>,
}

/// A place on a bus's way (`Narrow`): where, the way the bus goes, and how far it can pull over
/// there (m).
type NarrowPlace = ([f32; 2], [f32; 2], f32);

/// The cells of `Narrow` (m).
const NARROW_CELL: f32 = 8.0;
/// How far before one a claim of it begins (m: half a car's length and half a bus's): two coming
/// towards each other wait where they pass.
pub(crate) const NARROW_BACK: f32 = 6.6;

impl Narrow {
    /// Adds the place `p` where a bus going `way` can pull over `pass` m.
    pub(crate) fn add(&mut self, p: [f32; 2], way: [f32; 2], pass: f32) {
        self.cells.entry(cell(p)).or_default().push((p, way, pass));
    }

    /// Whether a car's body `car` going `way`, pulling over `pass` m there, couldn't pass a bus
    /// coming the other way there: their bodies as they'd be, side by side or nose to nose on a
    /// bend, both pulled over as far as they can (`conflict`).
    fn stuck(&self, car: &Rect, way: [f32; 2], pass: f32) -> bool {
        let (cx, cz) = cell(car.at);
        let mine = Sample {
            t: 0.0,
            d: 0.0,
            rect: *car,
            way,
            pass,
        };
        (cx - 1..=cx + 1).any(|x| {
            (cz - 1..=cz + 1).any(|z| {
                self.cells
                    .get(&(x, z))
                    .into_iter()
                    .flatten()
                    .any(|&(q, theirs, room)| {
                        // (Coming towards each other, if round a bend.)
                        if way[0] * theirs[0] + way[1] * theirs[1] >= 0.0 {
                            return false;
                        }
                        let bus = Sample {
                            t: 0.0,
                            d: 0.0,
                            rect: Rect {
                                at: q,
                                yaw: theirs[0].atan2(theirs[1]),
                                half: [crate::bus::BUS_LEN / 2.0, crate::bus::BUS_WID / 2.0],
                            },
                            way: theirs,
                            pass: room,
                        };
                        conflict(&mine, &bus, false)
                    })
            })
        })
    }

    /// The stretches of a car's route along `trace` (a body `len` by `wid`, pulling over `pass` m
    /// at each pose, its ends swinging out `swing` m, overlapping another car coming the other way
    /// in its lane `over` m) where it couldn't pass one coming the other way: a bus, here, or
    /// another car (a 6 m lane's tight bend).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn spans(
        &self,
        trace: &Trace,
        len: f32,
        wid: f32,
        pass: &[f32],
        swing: &[f32],
        over: &[f32],
    ) -> Vec<(f32, f32)> {
        let at = |v: &[f32], k: usize| v.get(k).copied().unwrap_or(0.0);
        let spans = runs(trace, |k| {
            let (p, s, o) = (at(pass, k), at(swing, k), at(over, k));
            let (_, q, yaw, back, _) = trace[k];
            let w = if back { -1.0 } else { 1.0 };
            let car = Rect {
                at: q,
                yaw,
                half: [len / 2.0, wid / 2.0],
            };
            (o > 0.0 && o + s + PASS_GAP > 2.0 * p)
                || self.stuck(&car, [yaw.sin() * w, yaw.cos() * w], p)
        });
        back_by(trace, spans, NARROW_BACK)
    }
}

/// The cell of `Narrow` a place is in.
fn cell(p: [f32; 2]) -> (i32, i32) {
    (
        (p[0] / NARROW_CELL).floor() as i32,
        (p[1] / NARROW_CELL).floor() as i32,
    )
}

/// `spans` of `trace` (ticks along it), each begun `back` m earlier.
pub(crate) fn back_by(trace: &Trace, spans: Vec<(f32, f32)>, back: f32) -> Vec<(f32, f32)> {
    spans
        .into_iter()
        .map(|(from, to)| {
            let k = trace.partition_point(|s| s.0 < from).min(trace.len() - 1);
            let d = trace[k].4 - back;
            let j = trace.partition_point(|s| s.4 < d).min(k);
            (trace[j].0, to)
        })
        .collect()
}

/// A car or bus on the road this tick, as the others keep out of its way (`Traffic::begin`): its
/// key (`cars::Driving::first`), its body, the way it goes and how far it can still pull over
/// (`Sample::pass`); whether it moved last tick, and where it'll be along its way in the next ticks
/// (`LOOK`); and whether it stands parked to all intents (a car still in its driveway, not yet let
/// out).
#[derive(Debug, Clone)]
pub(crate) struct Mover {
    pub key: u64,
    pub rect: Rect,
    pub way: [f32; 2],
    pub pass: f32,
    pub moving: bool,
    pub ahead: Vec<Rect>,
    pub parked: bool,
    /// Whom it waited for last tick (`Driving::first`), if anyone.
    pub waits: Option<u64>,
}

/// Where a vehicle is on its route, for its claims (`Traffic::begin`): its key, the tag of its
/// route's claims (a car's: whether it's on its way home; a bus's 0) and how far along it is
/// (ticks).
pub(crate) type Place = (u64, u32, f32);

#[derive(Debug, Clone)]
struct Hold {
    key: u64,
    tag: u32,
    claim: Claim,
    /// How far along its route its holder was as the tick began.
    at: f32,
    /// The tick it last came on along its route (`HOLD_STILL`), and the tick it asked for it.
    since: u64,
    asked: u64,
}

/// A vehicle's ask for the parts of a group of its route's claims (`Claim::group`), in order:
/// granted all at once, in the order the vehicles asked (`tick`).
#[derive(Debug, Clone)]
struct Ask {
    tick: u64,
    key: u64,
    route: u32,
    group: u32,
    parts: Vec<(u32, Claim)>,
    /// The last tick it waited at the start of the group (asking again: one not asked again is
    /// dropped).
    seen: u64,
    /// Whom it waited for as the tick began (their keys: their claims, bodies).
    by: Vec<u64>,
    /// The earlier asks it waited for as the tick began (their keys).
    fifo: Vec<u64>,
}

/// How far past where a group is cut (`claims`) the rest of it begins (ticks): standing there,
/// it asks for it only once it's on its way again.
const CUT_ON: f32 = 1.0;

/// How far short of what's left of another's claim one waits (m, `Traffic::blocked`).
const KEEP_SHORT: f32 = 1.5;
/// A bus that hasn't come on in this long (ticks) lets go of the parts it isn't in yet
/// (`Traffic::begin`).
const HOLD_STILL: u64 = 10 * crate::TICKS_PER_MINUTE;

/// An ask this old (ticks) yields only to those it waits for itself (`Traffic::begin`, rule 2).
const OLD_ASK: u64 = 20 * crate::TICKS_PER_MINUTE;

/// The claims held and asked for (see the module notes). Not saved.
#[derive(Debug, Clone, Default)]
pub struct Traffic {
    /// The tick (`begin`).
    tick: u64,
    holds: Vec<Hold>,
    asks: Vec<Ask>,
    /// Those that let go of parts not come on to (`HOLD_STILL`), and when they had asked: asking
    /// again, they keep their place.
    kept: Vec<(u64, u64)>,
    /// Each lit junction's lights (`Travel::signals`): the arm whose turn it is, how far through
    /// it (`Light`), and since when.
    pub(crate) lights: Vec<Light>,
    movers: Vec<Mover>,
    /// Ticks a car was kept from moving deeper into another car or a bus by the last-resort check
    /// (`cars::guard`), since the game was loaded.
    pub guarded: u64,
}

/// Claim `k` of a route tagged `route` (`Place`).
fn tag(route: u32, k: usize) -> u32 {
    route << 16 | k as u32
}

/// Whether `key` is a bus's (`cars::Driving::first`: a car's has its top bit set).
fn is_bus(key: u64) -> bool {
    key < 1 << 63
}

impl Traffic {
    /// Whether `key` holds its claim `tag`.
    fn holds(&self, key: u64, tag: u32) -> bool {
        self.holds.iter().any(|h| h.key == key && h.tag == tag)
    }

    /// Whether `key` holds a claim.
    pub fn holding(&self, key: u64) -> bool {
        self.holds.iter().any(|h| h.key == key)
    }

    /// Whether `key` waits for a claim to be granted, at the start of it (this tick or the last:
    /// the cars go on before the buses).
    pub fn waiting(&self, key: u64) -> bool {
        self.asks
            .iter()
            .any(|a| a.key == key && a.seen + 1 >= self.tick)
    }

    /// Whom `key` waits for its claim for (`Ask::by`): those holding claims in its way, or on the
    /// road in it.
    pub fn waits_for(&self, key: u64) -> &[u64] {
        self.asks
            .iter()
            .find(|a| a.key == key && a.seen + 1 >= self.tick)
            .map_or(&[][..], |a| &a.by[..])
    }

    fn mover(&self, key: u64) -> Option<&Mover> {
        self.movers.iter().find(|m| m.key == key)
    }

    /// Whether `key`, `t` ticks along its route of `claims` (tagged `route`), may drive on to `to`:
    /// it holds the part it's driving into or is in (else it asks for it and the rest of its group,
    /// and waits; in it already, it takes it).
    pub(crate) fn may_go(
        &mut self,
        tick: u64,
        key: u64,
        route: u32,
        claims: &[Claim],
        t: f32,
        to: f32,
    ) -> bool {
        for (k, c) in claims.iter().enumerate() {
            if to <= c.from || t >= c.to {
                continue;
            }
            let tag = tag(route, k);
            if self.holds(key, tag) {
                continue;
            }
            if t > c.from + 1e-3 {
                for a in &mut self.asks {
                    if a.key == key {
                        a.parts.retain(|p| p.0 != tag);
                    }
                }
                self.asks.retain(|a| !a.parts.is_empty());
                self.holds.push(Hold {
                    key,
                    tag,
                    claim: c.clone(),
                    at: t,
                    since: tick,
                    asked: tick,
                });
                continue;
            }
            match self.asks.iter_mut().find(|a| a.key == key) {
                Some(a) if a.parts.first().is_some_and(|p| p.0 == tag) => a.seen = tick,
                found => {
                    // (Asking again for what it let go of, standing still: in its place.)
                    let asked = match self.kept.iter().position(|k| k.0 == key) {
                        Some(i) => self.kept.remove(i).1,
                        None => tick,
                    };
                    let ask = Ask {
                        tick: asked,
                        key,
                        route,
                        group: c.group,
                        parts: (k..claims.len())
                            .take_while(|&j| claims[j].group == c.group)
                            .map(|j| (crate::traffic::tag(route, j), claims[j].clone()))
                            .collect(),
                        seen: tick,
                        by: Vec::new(),
                        fifo: Vec::new(),
                    };
                    match found {
                        Some(a) => *a = ask,
                        None => self.asks.push(ask),
                    }
                }
            }
            return false;
        }
        true
    }

    /// Whose claim `key` would drive into, its body `now` going on to `next` going `way` (pulling
    /// over `pass` m at most): what's left of a claim another holds that `next` is in the way of
    /// and `now` isn't (`counts`, even going its way: cutting in; not one right behind it,
    /// `behind`).
    pub(crate) fn blocked(
        &self,
        key: u64,
        now: &Rect,
        next: &Rect,
        way: [f32; 2],
        pass: f32,
    ) -> Option<u64> {
        let sample = |rect: &Rect| Sample {
            t: 0.0,
            d: 0.0,
            rect: *rect,
            way,
            pass,
        };
        // (Kept `KEEP_SHORT` m short of it, its body a tick on taken that much longer in front: the
        // other has room to straighten up and pull over.)
        let long = |r: &Rect| Rect {
            at: [
                r.at[0] + way[0] * KEEP_SHORT / 2.0,
                r.at[1] + way[1] * KEEP_SHORT / 2.0,
            ],
            half: [r.half[0] + KEEP_SHORT / 2.0, r.half[1]],
            ..*r
        };
        // (Holding a claim itself, granted it with the others in mind, as it is.)
        let holding = self.holding(key);
        let next = if holding { *next } else { long(next) };
        let (now, next) = (sample(now), sample(&next));
        let me = is_bus(key);
        for h in &self.holds {
            if h.key == key || !in_bounds(&h.claim, &next.rect) {
                continue;
            }
            // (Not one right behind it, nor a car that waits for it to go on: as for its grant.)
            if self.mover(h.key).is_some_and(|m| {
                behind((&now.rect, way), (&m.rect, m.way)) || !is_bus(h.key) && m.waits == Some(key)
            }) {
                continue;
            }
            let rest = h.claim.rest(h.at);
            // (In its way already, ahead of it: it waits for this one, which goes on out.)
            if rest
                .iter()
                .any(|o| counts(&now, me, o, is_bus(h.key), true))
            {
                continue;
            }
            // (Holding a claim itself, it was granted it with the other's: going its way, they
            // follow each other through, `cars::up_against`; only one not holding cuts in.)
            if rest
                .iter()
                .any(|o| counts(&next, me, o, is_bus(h.key), !holding))
            {
                return Some(h.key);
            }
        }
        None
    }

    /// Whether a group through the lights `gate` (`Claim::gate`) may be asked for now: its arm's
    /// green (none: no lights).
    pub(crate) fn green(&self, gate: Option<(u16, u8)>) -> bool {
        gate.is_none_or(|(s, arm)| {
            self.lights
                .get(s as usize)
                .is_none_or(|l| l.stage == Stage::Green && l.arm == arm)
        })
    }

    /// What arm `arm` of lit junction `s` shows (`Stage::Green` or `Amber` on its turn, else red).
    pub(crate) fn light(&self, s: usize, arm: usize) -> Option<Stage> {
        let l = self.lights.get(s)?;
        Some(if usize::from(l.arm) == arm && l.stage != Stage::Clear {
            l.stage
        } else {
            Stage::Clear
        })
    }

    /// The lights a tick on (`Light`): green on an arm `MIN_GREEN` at least, on while anyone comes
    /// to it (an ask through it, or one holding a claim through it still out of the box) and
    /// its share of `CYCLE` at most, and only while someone waits at another; then amber (`AMBER`),
    /// all red while the box clears (`ALL_RED`), and the next arm round that anyone waits at. Nobody waiting
    /// anywhere, it stays green where it is.
    fn change_lights(&mut self, tick: u64, signals: &[Signal]) {
        self.lights.resize(signals.len(), Light::default());
        for (i, sig) in signals.iter().enumerate() {
            let waits = |arm: u8| {
                self.asks.iter().any(|a| {
                    a.seen + 1 >= tick
                        && a.parts.first().and_then(|p| p.1.gate) == Some((i as u16, arm))
                })
            };
            let coming = |arm: u8| {
                self.holds
                    .iter()
                    .any(|h| h.claim.gate == Some((i as u16, arm)) && h.at < h.claim.from + 1.0)
            };
            let n = sig.arms.len() as u8;
            let l = self.lights[i];
            let on = tick.saturating_sub(l.since);
            // (Every arm round in `CYCLE` at most: nobody waits at a red light much longer.)
            let most = (CYCLE / u64::from(n.max(2) - 1))
                .saturating_sub(AMBER + ALL_RED)
                .max(MIN_GREEN);
            let next = |arm: u8| (1..n).map(|k| (arm + k) % n).find(|&x| waits(x));
            self.lights[i] = match l.stage {
                Stage::Green => {
                    let busy = waits(l.arm) || coming(l.arm);
                    match next(l.arm) {
                        Some(_) if on >= most || on >= MIN_GREEN && !busy => Light {
                            stage: Stage::Amber,
                            since: tick,
                            ..l
                        },
                        _ => l,
                    }
                }
                Stage::Amber if on >= AMBER => Light {
                    stage: Stage::Clear,
                    since: tick,
                    ..l
                },
                Stage::Clear if on >= ALL_RED => Light {
                    arm: next(l.arm).unwrap_or(l.arm),
                    stage: Stage::Green,
                    since: tick,
                },
                _ => l,
            };
        }
    }

    /// The tick's claims (before the cars and the buses go on): the vehicles on the road
    /// (`movers`), and where those holding or asking are (`places`); a part its holder is out of,
    /// or no longer on (another route, gone), let go; asks neither asked again nor going on through
    /// their group dropped; then the asks' next parts granted in order (see the module notes).
    pub(crate) fn begin(
        &mut self,
        tick: u64,
        movers: Vec<Mover>,
        places: &[Place],
        signals: &[Signal],
    ) {
        self.tick = tick;
        self.movers = movers;
        let place = |key: u64, route: u32| {
            places
                .iter()
                .find(|p| p.0 == key && p.1 == route)
                .map(|p| p.2)
        };
        // (A bus that hasn't come on `HOLD_STILL` and isn't in a part yet lets it go, and asks
        // for it again as it comes on: one held up on its way doesn't keep a junction from
        // everyone else.)
        let mut kept = Vec::new();
        self.holds.retain_mut(|h| match place(h.key, h.tag >> 16) {
            Some(t) if t < h.claim.to => {
                if t > h.at + 0.5 {
                    h.since = tick;
                }
                h.at = t;
                let stood = is_bus(h.key) && t < h.claim.from && tick > h.since + HOLD_STILL;
                if stood {
                    kept.push((h.key, h.asked));
                }
                !stood
            }
            _ => false,
        });
        for (key, asked) in kept {
            match self.kept.iter_mut().find(|k| k.0 == key) {
                Some(k) => k.1 = k.1.min(asked),
                None => self.kept.push((key, asked)),
            }
        }
        self.kept.retain(|k| places.iter().any(|p| p.0 == k.0));
        let holds = &self.holds;
        self.asks.retain(|a| {
            let going = holds
                .iter()
                .any(|h| h.key == a.key && h.tag >> 16 == a.route && h.claim.group == a.group);
            (a.seen + 1 >= tick || going)
                && place(a.key, a.route)
                    .is_some_and(|t| a.parts.first().is_some_and(|p| t <= p.1.from + 1e-3))
        });
        self.asks.sort_by_key(|a| (a.tick, a.key));
        self.change_lights(tick, signals);
        // Who waited for whom as the tick began: those on the road, for one in their way; the
        // asks, for the claims, bodies and earlier asks in theirs. An earlier ask doesn't hold up
        // one it waits for itself, however far round (`reaches`): no two asks wait for each other.
        let mut graph: std::collections::BTreeMap<u64, Vec<u64>> = Default::default();
        for m in &self.movers {
            graph.entry(m.key).or_default().extend(m.waits);
        }
        // (Who waits for whose body on the road, however far round: the asks' waits left out.)
        let on_road = graph.clone();
        for a in &self.asks {
            let e = graph.entry(a.key).or_default();
            e.extend(a.by.iter().chain(&a.fifo).copied());
        }
        // (Earlier asks with parts not granted, and whose bodies they wait for.)
        let mut pending: Vec<(usize, Vec<u64>)> = Vec::new();
        let mut waits: Vec<(usize, Vec<u64>)> = Vec::new();
        let mut fifo: Vec<(usize, Vec<u64>)> = Vec::new();
        for i in 0..self.asks.len() {
            let a = &self.asks[i];
            // (Its group's parts: it goes into the first only with all clear, `Ask`.)
            let next: Vec<&Claim> = a.parts.iter().map(|p| &p.1).collect();
            // (Nothing to it: it's its own.)
            let Some(first) = next[0].samples.first() else {
                let (tag, claim) = self.asks[i].parts.remove(0);
                let key = self.asks[i].key;
                let asked = self.asks[i].tick;
                self.holds.push(Hold {
                    key,
                    tag,
                    at: claim.from,
                    claim,
                    since: tick,
                    asked,
                });
                continue;
            };
            // (At the lights, not its arm's green: it waits at the stop line, and holds nobody up.)
            if !self.green(next[0].gate) {
                waits.push((i, Vec::new()));
                fifo.push((i, Vec::new()));
                continue;
            }
            let me = self
                .mover(a.key)
                .map_or((first.rect, first.way), |m| (m.rect, m.way));
            let me = (&me.0, me.1);
            let behind_me = |key: u64| {
                self.mover(key)
                    .is_some_and(|m| behind(me, (&m.rect, m.way)))
            };
            // (Of a part, what one right behind it may still come up against: further on.)
            let reach = first.d + NEAR_AHEAD;
            let far = |c: &'_ Claim| -> usize { c.samples.partition_point(|s| s.d <= reach) };
            let mine = |c: &'_ Claim, key: u64| -> std::ops::Range<usize> {
                if behind_me(key) {
                    far(c)..c.samples.len()
                } else {
                    0..c.samples.len()
                }
            };
            // 1. What's left of the parts held.
            let by_holds: Vec<u64> = self
                .holds
                .iter()
                .filter(|h| {
                    let rest = h.claim.rest(h.at);
                    // (Not a car's that waits for this one's body where it stands, in its way, or
                    // for the body of one that does, and so on, as in a queue behind it: it's to go
                    // on first.)
                    let waits = !is_bus(h.key)
                        && (self.mover(h.key).is_some_and(|m| m.waits == Some(a.key))
                            || reaches(&on_road, h.key, a.key));
                    h.key != a.key
                        && !waits
                        && next.iter().any(|c| {
                            overlaps(
                                (c, &c.samples[mine(c, h.key)], is_bus(a.key)),
                                (&h.claim, rest, is_bus(h.key)),
                            )
                        })
                })
                .map(|h| h.key)
                .collect();
            // 2. Earlier asks' next parts still waiting (but for one that waits for this one's
            // body).
            let conflicting = |(j, bodies_of): &&(usize, Vec<u64>)| {
                let e = &self.asks[*j];
                // (Asked long ago, only one it waits for itself goes first, a bus's or a car's:
                // no stream of others waiting for it round about holds it up for good.)
                let old = tick > e.tick + OLD_ASK;
                let first = if old {
                    graph.get(&e.key).is_some_and(|g| g.contains(&a.key))
                } else {
                    bodies_of.contains(&a.key) || reaches(&graph, e.key, a.key)
                };
                !first
                    && next.iter().any(|c| {
                        e.parts.iter().any(|(_, o)| {
                            overlaps(
                                (c, &c.samples[mine(c, e.key)], is_bus(a.key)),
                                (o, &o.samples, is_bus(e.key)),
                            )
                        })
                    })
            };
            fifo.push((
                i,
                pending
                    .iter()
                    .filter(conflicting)
                    .map(|(j, _)| self.asks[*j].key)
                    .collect(),
            ));
            // (Whose bodies those it waits for wait for, it doesn't wait for: it'd wait for itself.)
            let after: Vec<u64> = pending
                .iter()
                .filter(conflicting)
                .flat_map(|(_, b)| b.iter().copied())
                .collect();
            let earlier = pending.iter().any(|p| conflicting(&p));
            // 3. Those on the road not holding a claim: none in it (going its way too: don't
            // block the box), nor on the move into it; those holding one, not in its way; on its
            // far side, past its zone, nobody.
            let mut bodies = Vec::new();
            for m in &self.movers {
                // (Not one that waits for this one, however far round: it's to go on first.)
                if m.key == a.key || m.parked || behind_me(m.key) || reaches(&graph, m.key, a.key) {
                    continue;
                }
                let strict = !self.holding(m.key);
                let asking = self
                    .asks
                    .iter()
                    .any(|x| x.key == m.key && x.seen + 1 >= tick);
                let hit = |r: &Rect, strict: bool| {
                    let o = Sample {
                        t: 0.0,
                        d: 0.0,
                        rect: *r,
                        way: m.way,
                        pass: m.pass,
                    };
                    next.iter().any(|c| {
                        in_bounds(c, r)
                            && c.samples.iter().any(|s| {
                                // (Its far side clear of buses too, a bus's of one not itself
                                // waiting for a claim: don't block the box.)
                                let far = s.t > c.to;
                                let ahead = far && !asking;
                                counts(s, is_bus(a.key) && !ahead, &o, is_bus(m.key), strict || far)
                            })
                    })
                };
                if hit(&m.rect, strict) || m.moving && m.ahead.iter().any(|r| hit(r, false)) {
                    bodies.push(m.key);
                }
            }
            let free = by_holds.is_empty() && !earlier && bodies.is_empty();
            let mut by = by_holds;
            by.extend(bodies);
            if free {
                // (Held from now: the asks after it this tick see it held.)
                let key = self.asks[i].key;
                for (tag, claim) in std::mem::take(&mut self.asks[i].parts) {
                    self.holds.push(Hold {
                        key,
                        tag,
                        since: tick,
                        asked: self.asks[i].tick,
                        at: claim.from,
                        claim,
                    });
                }
                by.clear();
            }
            if !self.asks[i].parts.is_empty() {
                // (The bodies it waits for, and those the earlier asks it waits for wait for.)
                let mut bodies_of = by.clone();
                bodies_of.extend(after);
                pending.push((i, bodies_of));
            }
            waits.push((i, by));
        }
        for (i, e) in fifo {
            self.asks[i].fifo = e;
        }
        for (i, by) in waits {
            self.asks[i].by = by;
        }
        self.asks.retain(|a| !a.parts.is_empty());
    }
}

/// Whether `from` waits for `to`, however far round, in `graph` (who waits for whom).
fn reaches(graph: &std::collections::BTreeMap<u64, Vec<u64>>, from: u64, to: u64) -> bool {
    let mut seen = vec![from];
    let mut todo = vec![from];
    while let Some(k) = todo.pop() {
        for &x in graph.get(&k).into_iter().flatten() {
            if x == to {
                return true;
            }
            if !seen.contains(&x) {
                seen.push(x);
                todo.push(x);
            }
        }
    }
    false
}

/// Whether sample `a` (a bus's, `abus`, or a car's) is in the way of `b` (`bbus`'s): `conflict`
/// (`strict`: going the same way too); never a bus's of a bus's (their own rules: the signal box,
/// pulling over, backing up).
fn counts(a: &Sample, abus: bool, b: &Sample, bbus: bool, strict: bool) -> bool {
    !(abus && bbus) && conflict(a, b, strict)
}

/// Whether body `r` is within reach of `claim`'s samples at all (its box).
fn in_bounds(claim: &Claim, r: &Rect) -> bool {
    let b = claim.bounds;
    let k = reach(r) + CLEAR;
    !(r.at[0] + k < b[0] || r.at[0] - k > b[2] || r.at[1] + k < b[1] || r.at[1] - k > b[3])
}

/// Whether any of samples `a` (of claim `ca`, a bus's or not) is in the way of any of `b` (of
/// `cb`: `counts`).
fn overlaps(a: (&Claim, &[Sample], bool), b: (&Claim, &[Sample], bool)) -> bool {
    let ((ca, a, pa), (cb, b, pb)) = (a, b);
    if pa && pb {
        return false;
    }
    let (x, y) = (ca.bounds, cb.bounds);
    if x[2] < y[0] || y[2] < x[0] || x[3] < y[1] || y[3] < x[1] {
        return false;
    }
    a.iter()
        .any(|s| in_bounds(cb, &s.rect) && b.iter().any(|o| counts(s, pa, o, pb, false)))
}

/// Whether two bodies (samples `a` and `b`) are in each other's way: overlapping, or within
/// `CLEAR`; not going the same way (one following the other), unless `strict`; coming towards each
/// other only where pulling over as far as both can there (`Sample::pass`) leaves less than
/// `PASS_GAP` between them.
pub(crate) fn conflict(a: &Sample, b: &Sample, strict: bool) -> bool {
    // (Backing up, it follows nobody, nor passes anyone: going the way it faces, it's going on.)
    let back = |s: &Sample| s.way[0] * s.rect.yaw.sin() + s.way[1] * s.rect.yaw.cos() < 0.0;
    let backing = back(a) || back(b);
    let cos = a.way[0] * b.way[0] + a.way[1] * b.way[1];
    if cos > SAME_WAY && !strict && !backing {
        return false;
    }
    if dist(a.rect.at, b.rect.at) > reach(&a.rect) + reach(&b.rect) + CLEAR {
        return false;
    }
    let over = a.rect.overlap(&b.rect);
    if cos < -SAME_WAY && !backing {
        over + PASS_GAP > a.pass + b.pass
    } else {
        over > -CLEAR
    }
}

/// How far to one side of one in front a vehicle right behind it may be (m, `behind`: on a bend
/// too, in its lane).
const BEHIND_ACROSS: f32 = 1.5;

/// Whether `o` (its body and way) is right behind `me` in its lane, going its way: it can't get
/// anywhere before `me` does.
pub(crate) fn behind(me: (&Rect, [f32; 2]), o: (&Rect, [f32; 2])) -> bool {
    let (w, ow) = (me.1, o.1);
    if w[0] * ow[0] + w[1] * ow[1] <= SAME_WAY {
        return false;
    }
    let d = [o.0.at[0] - me.0.at[0], o.0.at[1] - me.0.at[1]];
    let along = d[0] * w[0] + d[1] * w[1];
    let across = (d[0] * w[1] - d[1] * w[0]).abs();
    along < -0.5 && across < BEHIND_ACROSS
}

/// Whom vehicle `key` (`cars::Driving::first`: a car's or a bus's) waits for this tick: one in
/// its way, those holding claims it asks for and the earlier asks it waits for. For checks (the
/// generated worlds': a vehicle is stuck when nothing it waits for, however far round, goes on).
pub fn waits_on(w: &World, key: u64) -> Vec<u64> {
    let mut out: Vec<u64> = if is_bus(key) {
        w.transit
            .way_of(key)
            .and_then(|b| b.2)
            .into_iter()
            .collect()
    } else {
        w.cars
            .get((key & !(1 << 63)) as usize)
            .and_then(|c| c.waits_for)
            .into_iter()
            .collect()
    };
    if let Some(a) = w.traffic.asks.iter().find(|a| a.key == key) {
        out.extend(a.by.iter().chain(&a.fifo));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Whether vehicle `key` waits at a red light (its ask through lights not on its arm's green): it
/// isn't held up for good, the light changes (for checks).
pub fn at_red(w: &World, key: u64) -> bool {
    w.traffic.asks.iter().any(|a| {
        a.key == key
            && a.seen + 1 >= w.traffic.tick
            && !w.traffic.green(a.parts.first().and_then(|p| p.1.gate))
    })
}

/// How far along its way vehicle `key` is (ticks), and whether it's a bus standing at a stop (on
/// its way all the same); none, not on its way. For checks, with `waits_on`.
pub fn on_its_way(w: &World, key: u64) -> Option<(f32, bool)> {
    if is_bus(key) {
        return w.transit.way_of(key).map(|b| (b.0, b.1));
    }
    let car = w.cars.get((key & !(1 << 63)) as usize)?;
    matches!(
        car.state,
        crate::cars::CarState::Leaving | crate::cars::CarState::Arriving
    )
    .then_some((car.progress, false))
}

/// The tick's claims, before the cars and buses go on (`Traffic::begin`).
pub(crate) fn begin(w: &mut World) {
    let mut movers = Vec::new();
    let mut places = Vec::new();
    crate::cars::movers(w, &mut movers, &mut places);
    crate::bus::movers(w, &mut movers, &mut places);
    let tick = w.tick;
    w.traffic.begin(tick, movers, &places, &w.travel.signals);
}
