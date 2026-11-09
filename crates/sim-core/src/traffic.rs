//! Taking turns where cars' ways cross or meet (`World::traffic`): reservations, not give-way
//! rules, every car in one queue.
//!
//! **Zones and claims.** Where two ways may cross or meet so that both can't go on, there's a
//! zone: each junction's box (`boxes`: a 6 m circle round its middle, reaching on along an arm as
//! far as needed to keep a body in its lane there `ARMS_APART` from one on another arm close to
//! it). Each car's route (`cars::Route`) is looked at once, as it's laid out, for the zones it
//! drives through, and for stretches that are claims of their own: a driveway's, a bay's or a spot
//! on the road's mouth (a car turning out across the lane or in), and round a 6 m lane's tight
//! bend where it couldn't pass one coming the other way, both pulled over as far as they can
//! (`narrow`). Each is a part (`Claim`): the stretch of the route from just before its body first
//! touches it (`Claim::from`) to where it's out of it again (`Claim::to`), sampled every `SAMPLE` m
//! (more often turning) with its room on the far side (its length and `ROOM_GAP`). Parts its body
//! comes to before it's out of the last make a group (`Claim::group`): it can't stand clear between
//! them, so it asks for them together.
//!
//! **Asking.** A car about to drive into a group asks for it (`Traffic::may_go`) and waits where it
//! is till it's granted, all of it at once. The asks are granted once a tick (`Traffic::begin`,
//! before the cars go on), in the order they were asked (the tick, then `cars::Driving::first`),
//! each when
//! 1. none of it is in the way (`conflict`) of what's left of a part another holds,
//! 2. nor of an earlier ask still waiting,
//! 3. and nobody not holding a part is in it now, nor anyone at all on its far side ("don't block
//!    the box"), nor anyone on the move into it in the next second or so (`LOOK`).
//!
//! Two bodies are in each other's way (`conflict`) when they overlap or come within `CLEAR`; going
//! the same way (one following the other through, each keeping behind the one in front:
//! `cars::up_against`) they aren't; coming towards each other they are where pulling over as far as
//! both can there (`Sample::pass`) doesn't leave `PASS_GAP` between them. One right behind another
//! in its lane (`behind`) can't get anywhere before that one does: neither its parts nor its ask
//! hold that one up there.
//!
//! **Holding.** Holding a group, a car drives through it, letting go of each part as its body's
//! out of it. Anyone else, holding a group or not, keeps `KEEP_SHORT` short of what's left of one
//! another holds (`Traffic::blocked`; holding one itself, it may follow another through one going
//! its way; already in another's way, it goes on out of it).
//!
//! **Why nobody waits for good.** As a tick begins, who waits for whom is a graph: the cars' waits
//! on the road last tick (`Mover::waits`, `cars::Car::waits_for`), and the asks' waits for parts
//! held, bodies and earlier asks. Every rule that makes one wait for another leaves out one that
//! waits for it, however far round (`reaches`): an earlier ask doesn't hold up a later one it waits
//! for itself (rule 2), a body that waits for the asker doesn't (rule 3), one holding a part that
//! waits for the asker's body doesn't (rule 1), nor in `Traffic::blocked`. So the grants never
//! close a ring of waits the graph has in it: a ring forms only through waits it can't see (a
//! pull-over, a turn sweeping wider than its samples), and those are broken where they form: a car
//! and one in its way waiting for each other a while, the car backs up a little out of its way,
//! never into anyone (`cars::back_off`). An ask older than `OLD_ASK` yields only to one it waits
//! for itself, not to a stream round about: nobody waits on and on while others come and go; and
//! nobody comes up to wait in its way (`STOP_SHORT`, `Traffic::blocked`), to be let go first.
//! Should two bodies come to overlap anyway, a last-resort check keeps every car from moving
//! deeper into another (`cars::guard`, counted in `Traffic::guarded`).
//!
//! Nothing here is saved but the lights (`Light`): the zones and the claims are worked out again
//! with the routes, and a car found in a part it doesn't hold (a game just loaded) takes it as it
//! goes on.

use crate::cars::Rect;
use crate::travel::{Graph, dist};
use crate::world::World;

/// How far apart a route's samples are along it (m), and how far it turns between two at most
/// (radians: a car's ends swing round about 0.2 m).
pub(crate) const SAMPLE: f32 = 1.0;
const TURN_SAMPLE: f32 = 0.08;
/// The room a vehicle wants clear on the far side of a claim: its length, and this (m).
pub(crate) const ROOM_GAP: f32 = 1.0;
/// Two coming towards each other pass where pulling over takes them this far apart (m: the gap
/// the cars keep, `cars::GAP`, and a little: their ways are sampled a metre apart, and between two
/// a body turning comes a little nearer).
pub(crate) const PASS_GAP: f32 = crate::cars::GAP + 0.05;
/// Two going within about 25° of each other's way go the same way (the cosine).
const SAME_WAY: f32 = 0.9;
/// Bodies this near (m) are in each other's way (a body's corners between two samples too).
const CLEAR: f32 = 0.25;
/// One on the move, looked ahead this many ticks along its way (`Traffic::begin`: half a second,
/// one, one and a half): it isn't made to stop short in front of one just granted.
pub(crate) const LOOK: [f32; 3] = [
    crate::secs(0.5) as f32,
    crate::secs(1.0) as f32,
    crate::secs(1.5) as f32,
];
/// How far along its way a vehicle's ask is its own business with one right behind it (m,
/// `behind`): that one can't get there first. Further on (its way round and back) it may.
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

/// How far apart (ticks: a fortieth of a second) `trace` looks along a route for its poses.
const TRACE_STEP: f32 = 0.025 * crate::TICKS_PER_SECOND as f32;

/// `Trace` of a route `ticks` long whose pose `t` ticks along is `at(t)`.
pub(crate) fn trace(at: impl Fn(f32) -> ([f32; 2], f32, bool), ticks: f32) -> Trace {
    let (p, yaw, back) = at(0.0);
    let mut out: Trace = vec![(0.0, p, yaw, back, 0.0)];
    let (mut last, mut along, mut since, mut turned) = (p, 0.0f32, 0.0f32, yaw);
    let mut t = 0.0f32;
    while t < ticks {
        t = (t + TRACE_STEP).min(ticks);
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
/// it can pull over at each pose, `pass`), the zones it drives through (`zones`: junctions' boxes),
/// and `more` (stretches of it that are parts of their own: a driveway's mouth, where it couldn't
/// pass one coming the other way). Parts its body comes to before it's out of the last make one
/// group (`Claim::group`: it can't stand clear of them between).
pub(crate) fn claims(
    trace: &Trace,
    pass: &[f32],
    len: f32,
    wid: f32,
    zones: &[([f32; 2], f32)],
    more: &[(f32, f32)],
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
    // A group begun with a zone, from its first pose there: one with the group before, where
    // that's no further on.
    let back = |t: f32| {
        let k = trace.partition_point(|s| s.0 < t).min(n - 1);
        trace[trace.partition_point(|s| s.4 < trace[k].4).min(k)].0
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
    let room = len + ROOM_GAP;
    parts
        .into_iter()
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

/// A junction with traffic lights (`Travel::signals`: where `SIGNAL_ARMS` roads or more meet,
/// `settle`): its middle, and its arms. One arm
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

/// A junction where this many roads for cars meet may have traffic lights (`Signal`).
pub(crate) const SIGNAL_ARMS: usize = 3;
/// How long an arm's green lasts at least, its amber, and all red after while the box
/// clears (ticks of real time: the lights are seen to change).
const MIN_GREEN: u64 = crate::secs(5.0);
/// Every arm's turn comes round in this long at most (ticks of real time): an arm's green lasts no
/// longer than its share of it (`change_lights`).
const CYCLE: u64 = crate::secs(50.0);
const AMBER: u64 = crate::secs(2.0);
const ALL_RED: u64 = crate::secs(4.0);
/// A route this near a lit junction's middle goes through its lights (m, `gate`).
const GATE_REACH: f32 = BOX_MOST + 2.0;
/// How far out past its arm's box a signal pole stands, and in from the kerb on the pavement (or
/// the road's edge) (m).
const POLE_OUT: f32 = 1.0;
const POLE_KERB: f32 = 0.4;

/// An arm's road from its junction (`signal_at`): which, where along it and which way (+1, -1),
/// and the stretch of it between its ends' nodes.
type ArmRoad = (u32, f32, f32, f32, f32);

/// The lights of the junction at node `n` of `g` (`Signal`): its arms for cars, each with its pole
/// on the kerb to the right of the way in, just out past the box (`boxes`). None: fewer than
/// `SIGNAL_ARMS` arms.
pub(crate) fn signal_at(g: &Graph, n: usize) -> Option<Signal> {
    let p = g.nodes[n];
    // (Each arm's way out, how far from its middle its kerb is, and its road from the node: which,
    // where along it and which way.)
    let mut arms: Vec<([f32; 2], f32, ArmRoad)> = Vec::new();
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
                    (e.road, s, k, e.s0, e.s1),
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
        .map(|&(way, kerb, (road, s, k, s0, s1))| {
            // (Along its own road, round its bend: where that's out past the box, and the way
            // out there; coming in, its right is the way out's left. On the pavement, `POLE_KERB`
            // in from the kerb; on a country road, as far out on the verge.)
            let road = &g.roads[road as usize];
            let at = (s + k * (reach(way) + POLE_OUT)).clamp(s0, s1);
            let t = road.tangent(at);
            let out = [t[0] * k, t[1] * k];
            let right = [out[1], -out[0]];
            let m = road.at(at);
            let side = kerb + POLE_KERB;
            SignalArm {
                way,
                pole: [m[0] + right[0] * side, m[1] + right[1] * side],
                yaw: out[0].atan2(out[1]),
            }
        })
        .collect();
    Some(Signal { at: p, arms })
}

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

/// The stretches of a car's route along `trace` (pulling over `pass` m at each pose, its ends
/// swinging out `swing` m, overlapping another car coming the other way in its lane `over` m) where
/// it couldn't pass one coming the other way (a 6 m lane's tight bend; a 7 m street's sharp bend
/// out of the junctions' boxes, there the two in their lanes too near each other however far
/// they pull over), each begun `NARROW_BACK` m before: a claim of their own, so that two coming
/// towards each other take turns there. (The other's body swings about as much as its own there,
/// across the bend from it: two taxis met side by side on a 7 m lane's bend by a junction, too near
/// to pass, Rowan Hollow.)
pub(crate) fn narrow(trace: &Trace, pass: &[f32], swing: &[f32], over: &[f32]) -> Vec<(f32, f32)> {
    let at = |v: &[f32], k: usize| v.get(k).copied().unwrap_or(0.0);
    let spans = runs(trace, |k| {
        let (p, s, o) = (at(pass, k), at(swing, k), at(over, k));
        (o > 0.0 || p > 0.0) && o + 2.0 * s + PASS_GAP > 2.0 * p
    });
    back_by(trace, spans, NARROW_BACK)
}

/// How far before a narrow stretch a claim of it begins (m, `narrow`): two coming towards each
/// other wait where they pass.
pub(crate) const NARROW_BACK: f32 = 6.6;

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

/// A car on the road this tick, as the others keep out of its way (`Traffic::begin`): its
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
/// route's claims (whether it's on its way home) and how far along it is (ticks).
pub(crate) type Place = (u64, u32, f32);

#[derive(Debug, Clone)]
struct Hold {
    key: u64,
    tag: u32,
    claim: Claim,
    /// How far along its route its holder was as the tick began.
    at: f32,
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
    /// Of `by`, those only on the move into its way, not in it: they're to stop short, it doesn't
    /// wait for them to go first (`begin`, rule 3).
    coming: Vec<u64>,
    /// Held up by nobody's claim nor an earlier ask, only by bodies: once those still to come
    /// keep clear (`blocked`), it's granted.
    bodies_only: bool,
    /// Long asked (`OLD_ASK`), whom it waits for, however far round (as the tick began, those only
    /// coming into its way left out): none of them keeps clear of it (`blocked`).
    reach: Vec<u64>,
}

/// How far short of what's left of another's claim one waits (m, `Traffic::blocked`).
const KEEP_SHORT: f32 = 1.5;

/// How far short of the start of its next claim one not holding a claim stops, where waiting
/// there it would be in the way of a group asked for long ago (`OLD_ASK`, m: `Traffic::blocked`).
const STOP_SHORT: f32 = 3.0;

/// An ask not asked again stands while its vehicle is no further short of its group than this
/// (ticks along its route: backed up out of one's way, `cars::JAM_BACK`, and a little more,
/// `Traffic::begin`).
const KEPT_ASK: f32 = crate::secs(3.25) as f32;

/// An ask this old (ticks of real time) yields only to those it waits for itself
/// (`Traffic::begin`, rule 2).
const OLD_ASK: u64 = crate::secs(20.0);

/// The claims held and asked for (see the module notes). Not saved.
#[derive(Debug, Clone, Default)]
pub struct Traffic {
    /// The tick (`begin`).
    tick: u64,
    holds: Vec<Hold>,
    asks: Vec<Ask>,
    /// Each lit junction's lights (`Travel::signals`): the arm whose turn it is, how far through
    /// it (`Light`), and since when.
    pub(crate) lights: Vec<Light>,
    movers: Vec<Mover>,
    /// Where each vehicle on its way to the start of its next claim, not yet there, will wait for
    /// it (its key, the claim's first sample: `may_go`, this tick).
    starts: Vec<(u64, Sample)>,
    /// Ticks a car was kept from moving deeper into another by the last-resort check
    /// (`cars::guard`), since the game was loaded.
    pub guarded: u64,
    /// Cars held up out of the picture put on along their ways (`cars::unstick`), since the game
    /// was loaded.
    pub unstuck: u64,
    /// Cars held up in sight turned round to go another way (`cars::unjam`), and rings of them
    /// none of which could, by why, since the game was loaded.
    pub unjams: u64,
    pub unjam_impossible: std::collections::BTreeMap<&'static str, u64>,
    /// Rings `cars::unjam` last tried (by their least key), and when.
    pub(crate) unjam_tried: Vec<(u64, u64)>,
    /// Work taxis held still on their way so long they gave their way up (`taxi.rs`): in sight
    /// (driving off, `cars::drive_off`) and out of it (gone), since the game was loaded.
    pub give_ups: [u64; 2],
    /// Each car turning round (`cars::unjam`): the ground it still backs up over, its bodies
    /// along it this tick.
    pub(crate) turning: Vec<(u64, Vec<crate::cars::Rect>)>,
    /// The work taxis standing at the kerb in sight (`cars::pass_stands`: their keys, where), and
    /// a count that goes up each time they change.
    pub(crate) stands: Vec<(u64, [f32; 2])>,
    pub(crate) stand_gen: u64,
    /// Each car on its way round a standing taxi (`cars::PassBy`), and the taxi's key: it keeps
    /// clear of the taxi's claim as it goes by.
    pub(crate) passing: Vec<(u64, u64)>,
    /// Per lit junction, the arms anyone waits at and comes through (`change_lights`'s own,
    /// kept so it isn't made anew every tick).
    lit_arms: Vec<[u64; 2]>,
}

/// Claim `k` of a route tagged `route` (`Place`).
fn tag(route: u32, k: usize) -> u32 {
    route << 16 | k as u32
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

    /// Whether `key` waits for a claim to be granted, at the start of it (this tick or the last).
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

    /// `key`, a taxi standing `t` ticks along its way at a stop (`cars::stand_taxi`): every claim
    /// it holds but the one it stands in let go, and its asks too; it asks again as it sets off.
    /// (Standing there for its riders, a claim on ahead held others up all the while.)
    pub(crate) fn stand(&mut self, key: u64, t: f32) {
        self.holds
            .retain(|h| h.key != key || h.claim.from <= t + 1e-3 && t < h.claim.to);
        self.asks.retain(|a| a.key != key);
        self.starts.retain(|s| s.0 != key);
        for a in &mut self.asks {
            a.by.retain(|&k| k != key);
            a.fifo.retain(|&k| k != key);
        }
    }

    /// `key` gone out of sight (`cars::sight`): its claims, held or asked for, let go, and nobody
    /// waits for it.
    pub(crate) fn forget(&mut self, key: u64) {
        self.holds.retain(|h| h.key != key);
        self.asks.retain(|a| a.key != key);
        self.starts.retain(|s| s.0 != key);
        for a in &mut self.asks {
            a.by.retain(|&k| k != key);
            a.fifo.retain(|&k| k != key);
        }
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
                });
                continue;
            }
            match self.asks.iter_mut().find(|a| a.key == key) {
                Some(a) if a.parts.first().is_some_and(|p| p.0 == tag) => a.seen = tick,
                found => {
                    let ask = Ask {
                        tick,
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
                        coming: Vec::new(),
                        bodies_only: false,
                        reach: Vec::new(),
                    };
                    match found {
                        Some(a) => *a = ask,
                        None => self.asks.push(ask),
                    }
                }
            }
            self.starts.retain(|s| s.0 != key);
            return false;
        }
        // (Where it'll wait for its next claim: `blocked`.)
        self.starts.retain(|s| s.0 != key);
        let next = claims
            .iter()
            .enumerate()
            .find(|(k, c)| c.from >= to && !self.holds(key, tag(route, *k)));
        if let Some(first) = next.and_then(|(_, c)| c.samples.first()) {
            self.starts.push((key, *first));
        }
        true
    }

    /// Whose claim `key` would drive into, its body `now` going on to `next` going `way` (pulling
    /// over `pass` m at most): what's left of a claim another holds that `next` is in the way of
    /// and `now` isn't (`counts`, even going its way: cutting in; not one right behind it,
    /// `behind`). Still in its driveway (`in_drive`: before its first claim, none held), `now` in
    /// the way too: it'd only come out deeper into it. Not holding one, nor on up to wait in the
    /// way of a group asked for long ago (`OLD_ASK`).
    pub(crate) fn blocked(
        &self,
        key: u64,
        now: &Rect,
        next: &Rect,
        way: [f32; 2],
        pass: f32,
        in_drive: bool,
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
        let raw_next = *next;
        let next = if holding { *next } else { long(next) };
        let (now, next) = (sample(now), sample(&next));
        for h in &self.holds {
            // (Going by a standing taxi, out round it: not its claim where it stands.)
            if h.key == key
                || !in_bounds(&h.claim, &next.rect)
                || self.passing.contains(&(key, h.key))
            {
                continue;
            }
            // (Not one right behind it, nor one that waits for it to go on: as for its grant.)
            if self
                .mover(h.key)
                .is_some_and(|m| behind((&now.rect, way), (&m.rect, m.way)) || m.waits == Some(key))
            {
                continue;
            }
            let rest = h.claim.rest(h.at);
            // (In its way already, ahead of it: it waits for this one, which goes on out, away
            // from it; not on towards it.)
            let away = self
                .mover(h.key)
                .is_none_or(|m| dist(next.rect.at, m.rect.at) >= dist(now.rect.at, m.rect.at));
            if !in_drive && away && rest.iter().any(|o| conflict(&now, o, true)) {
                continue;
            }
            // (Holding a claim itself, it was granted it with the other's: going its way, they
            // follow each other through, `cars::up_against`; only one not holding cuts in.)
            if rest.iter().any(|o| conflict(&next, o, !holding)) {
                return Some(h.key);
            }
        }
        // Not holding a claim, nor on up to wait at the start of its next one (`Traffic::starts`)
        // where it would be in the way of a group another has asked for long (`OLD_ASK`) and
        // waits for at its start: it stops `STOP_SHORT` short of it. (Standing there, it would hold
        // that one up, to be let go first as it asked after it: a stream of them kept the one
        // waiting there for good.)
        if holding {
            return None;
        }
        let into = |a: &Ask, r: &Sample| {
            a.parts.iter().any(|(_, c)| {
                in_bounds(c, &r.rect) && c.samples.iter().any(|o| conflict(r, o, true))
            })
        };
        let old = |a: &&Ask| {
            a.key != key
                && a.seen + 1 >= self.tick
                && self.tick > a.tick + OLD_ASK
                && self
                    .mover(a.key)
                    .is_none_or(|m| !behind((&now.rect, way), (&m.rect, m.way)))
        };
        // Not holding a claim, nor on into the way of a group another has asked for long
        // (`OLD_ASK`), held up only by bodies in its way, while out of it ("keep clear"): stood
        // there in turn, it'd hold that one up as those before it did; a stream of them kept one
        // backing out of its driveway by a busy junction, and one waiting at a junction with a
        // stream of taxis through it, waiting for good. (Not one that one waits for, however far
        // round: they'd wait for each other.)
        let clear = Sample { pass: 0.0, ..now };
        if let Some(a) = self.asks.iter().filter(old).find(|a| {
            a.bodies_only && !a.reach.contains(&key) && into(a, &next) && !into(a, &clear)
        }) {
            return Some(a.key);
        }
        let stop = self.starts.iter().find(|s| s.0 == key).map(|s| s.1)?;
        // (Nearer than half that already, it came up to it before that one's ask was old: on.)
        let (from, to) = (
            dist(now.rect.at, stop.rect.at),
            dist(raw_next.at, stop.rect.at),
        );
        if from <= STOP_SHORT / 2.0 || to > STOP_SHORT {
            return None;
        }
        // (Standing where it stops, as it stands there: not in that one's way itself.)
        let stop = Sample { pass: 0.0, ..stop };
        let short = Sample {
            rect: raw_next,
            pass: 0.0,
            ..now
        };
        self.asks
            .iter()
            .filter(old)
            .find(|a| into(a, &stop) && !into(a, &short))
            .map(|a| a.key)
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
        // (Per lit junction, the arms anyone waits at and comes through, bit `arm` of each: one
        // pass over the asks and the holds for every junction, not one per junction and arm.)
        let bit = |arm: u8| 1u64.checked_shl(u32::from(arm)).unwrap_or(0);
        let mut arms = std::mem::take(&mut self.lit_arms);
        arms.clear();
        arms.resize(signals.len(), [0, 0]);
        for a in &self.asks {
            if a.seen + 1 >= tick
                && let Some((s, arm)) = a.parts.first().and_then(|p| p.1.gate)
                && let Some(m) = arms.get_mut(usize::from(s))
            {
                m[0] |= bit(arm);
            }
        }
        for h in &self.holds {
            if h.at < h.claim.from + 1.0
                && let Some((s, arm)) = h.claim.gate
                && let Some(m) = arms.get_mut(usize::from(s))
            {
                m[1] |= bit(arm);
            }
        }
        for (i, sig) in signals.iter().enumerate() {
            let waits = |arm: u8| arms[i][0] & bit(arm) != 0;
            let coming = |arm: u8| arms[i][1] & bit(arm) != 0;
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
        self.lit_arms = arms;
    }

    /// The tick's claims (before the cars go on): the vehicles on the road
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
        self.starts.clear();
        let place = |key: u64, route: u32| {
            places
                .iter()
                .find(|p| p.0 == key && p.1 == route)
                .map(|p| p.2)
        };
        self.holds.retain_mut(|h| match place(h.key, h.tag >> 16) {
            Some(t) if t < h.claim.to => {
                h.at = t;
                true
            }
            _ => false,
        });
        let holds = &self.holds;
        self.asks.retain(|a| {
            let going = holds
                .iter()
                .any(|h| h.key == a.key && h.tag >> 16 == a.route && h.claim.group == a.group);
            // (Backed up a little out of one's way, not asking again meanwhile, `cars::back_off`:
            // its ask stands, in its place in the queue, `KEPT_ASK`.)
            let at = place(a.key, a.route);
            let short = |t: f32| a.parts.first().is_some_and(|p| t >= p.1.from - KEPT_ASK);
            (a.seen + 1 >= tick || going || at.is_some_and(short))
                && at.is_some_and(|t| a.parts.first().is_some_and(|p| t <= p.1.from + 1e-3))
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
            // (Not those only coming into its way: they keep clear of it, `blocked`, and are no
            // reason for it to let them go first.)
            let e = graph.entry(a.key).or_default();
            e.extend(
                a.by.iter()
                    .filter(|k| !a.coming.contains(k))
                    .chain(&a.fifo)
                    .copied(),
            );
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
                self.holds.push(Hold {
                    key,
                    tag,
                    at: claim.from,
                    claim,
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
                    // (Not one's that waits for this one's body where it stands, in its way, or
                    // for the body of one that does, and so on, as in a queue behind it: it's to go
                    // on first; but for its way through crossing where that one stands.)
                    let holder = self.mover(h.key);
                    let through = holder.is_some_and(|m| {
                        let body = Sample {
                            t: 0.0,
                            d: 0.0,
                            rect: m.rect,
                            way: m.way,
                            pass: m.pass,
                        };
                        next.iter().any(|c| {
                            in_bounds(c, &m.rect)
                                && c.samples.iter().any(|s| conflict(s, &body, true))
                        })
                    });
                    let waits = !through
                        && (holder.is_some_and(|m| m.waits == Some(a.key))
                            || reaches(&on_road, h.key, a.key));
                    h.key != a.key
                        && !waits
                        && !self.passing.contains(&(a.key, h.key))
                        && next
                            .iter()
                            .any(|c| overlaps((c, &c.samples[mine(c, h.key)]), (&h.claim, rest)))
                })
                .map(|h| h.key)
                .collect();
            // 2. Earlier asks' next parts still waiting (but for one that waits for this one's
            // body).
            let conflicting = |(j, bodies_of): &&(usize, Vec<u64>)| {
                let e = &self.asks[*j];
                // (Asked long ago, only one it waits for itself, however far round, goes first:
                // by claims, bodies or the earlier asks it waits its turn after (one in the way of
                // an ask before it is to go on first); not one it doesn't, on this tick's bodies
                // alone. No stream of others waiting their turn after it round about holds it up
                // for good, and no ring of them either.)
                let old = tick > e.tick + OLD_ASK;
                let first = if old {
                    reaches(&graph, e.key, a.key)
                } else {
                    bodies_of.contains(&a.key) || reaches(&graph, e.key, a.key)
                };
                !first
                    && next.iter().any(|c| {
                        e.parts.iter().any(|(_, o)| {
                            overlaps((c, &c.samples[mine(c, e.key)]), (o, &o.samples))
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
            let mut coming = Vec::new();
            for m in &self.movers {
                if m.key == a.key
                    || m.parked
                    || behind_me(m.key)
                    || self.passing.contains(&(a.key, m.key))
                {
                    continue;
                }
                let strict = !self.holding(m.key);
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
                                // (Its far side clear too: don't block the box.)
                                conflict(s, &o, strict || s.t > c.to)
                            })
                    })
                };
                // (Not one that waits for this one, however far round: it's to go on first; but
                // for one standing where its way goes through, as for a claim's holder (rule 1).)
                if reaches(&graph, m.key, a.key) && !hit(&m.rect, true) {
                    continue;
                }
                let inside = hit(&m.rect, strict);
                if !inside && m.moving && m.ahead.iter().any(|r| hit(r, false)) {
                    coming.push(m.key);
                }
                if inside || m.moving && m.ahead.iter().any(|r| hit(r, false)) {
                    bodies.push(m.key);
                }
            }
            let free = by_holds.is_empty() && !earlier && bodies.is_empty();
            let bodies_only = by_holds.is_empty() && !earlier;
            let mut by = by_holds;
            by.extend(bodies);
            if free {
                // (Held from now: the asks after it this tick see it held.)
                let key = self.asks[i].key;
                for (tag, claim) in std::mem::take(&mut self.asks[i].parts) {
                    self.holds.push(Hold {
                        key,
                        tag,
                        at: claim.from,
                        claim,
                    });
                }
                by.clear();
            }
            if !self.asks[i].parts.is_empty() {
                // (The bodies it waits for, and those the earlier asks it waits for wait for: not
                // those only coming into its way.)
                let mut bodies_of: Vec<u64> =
                    by.iter().filter(|k| !coming.contains(k)).copied().collect();
                bodies_of.extend(after);
                pending.push((i, bodies_of));
            }
            self.asks[i].coming = coming;
            self.asks[i].bodies_only = bodies_only;
            waits.push((i, by));
        }
        for (i, e) in fifo {
            self.asks[i].fifo = e;
        }
        for (i, by) in waits {
            self.asks[i].by = by;
        }
        // (Whom each long-asked waits for however far round: `blocked`.)
        for i in 0..self.asks.len() {
            let a = &self.asks[i];
            let reach = if tick > a.tick + OLD_ASK {
                let mut seen = vec![a.key];
                let mut k = 0;
                while k < seen.len() {
                    for &x in graph.get(&seen[k]).into_iter().flatten() {
                        if !seen.contains(&x) {
                            seen.push(x);
                        }
                    }
                    k += 1;
                }
                seen
            } else {
                Vec::new()
            };
            self.asks[i].reach = reach;
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

/// Whether body `r` is within reach of `claim`'s samples at all (its box).
fn in_bounds(claim: &Claim, r: &Rect) -> bool {
    let b = claim.bounds;
    let k = reach(r) + CLEAR;
    !(r.at[0] + k < b[0] || r.at[0] - k > b[2] || r.at[1] + k < b[1] || r.at[1] - k > b[3])
}

/// Whether any of samples `a` (of claim `ca`) is in the way of any of `b` (of `cb`: `conflict`).
fn overlaps(a: (&Claim, &[Sample]), b: (&Claim, &[Sample])) -> bool {
    let ((ca, a), (cb, b)) = (a, b);
    let (x, y) = (ca.bounds, cb.bounds);
    if x[2] < y[0] || y[2] < x[0] || x[3] < y[1] || y[3] < x[1] {
        return false;
    }
    a.iter()
        .any(|s| in_bounds(cb, &s.rect) && b.iter().any(|o| conflict(s, o, false)))
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

/// Whom car `key` (`cars::Driving::first`) waits for this tick: one in its way (another vehicle,
/// or those on foot or on a bike: their resident ids, below `1 << 63`), those holding
/// claims it asks for and the earlier asks it waits for. For checks (the generated worlds': a car
/// is stuck when nothing it waits for, however far round, goes on).
pub fn waits_on(w: &World, key: u64) -> Vec<u64> {
    let car = w.cars.get((key & !(1 << 63)) as usize);
    let mut out: Vec<u64> = car.and_then(|c| c.waits_for).into_iter().collect();
    // (Those on foot or on a bike in its way: their resident ids, no vehicle's key.)
    out.extend(
        car.into_iter()
            .flat_map(|c| c.on_foot.iter().map(|&i| u64::from(i))),
    );
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

/// How far along its way car `key` is (ticks); none, not on its way. For checks, with `waits_on`.
pub fn on_its_way(w: &World, key: u64) -> Option<f32> {
    let car = w.cars.get((key & !(1 << 63)) as usize)?;
    matches!(
        car.state,
        crate::cars::CarState::Leaving | crate::cars::CarState::Arriving
    )
    .then_some(car.progress)
}

/// The junctions' traffic lights (`Travel::signals`: every junction where `SIGNAL_ARMS` roads or
/// more meet, while their heads fit the snapshot's rows), and every car's claims worked out again
/// with them (`Claim::gate`). Once the cars are made or loaded.
pub(crate) fn settle(w: &mut World) {
    let g = &w.travel.graph;
    let mut signals: Vec<Signal> = Vec::new();
    let mut heads = 0;
    for n in 0..g.nodes.len() {
        let Some(sig) = signal_at(g, n) else { continue };
        // (No junction lit without a head for each arm in the snapshot's rows.)
        if heads + sig.arms.len() > crate::snapshot::MAX_LIGHTS {
            break;
        }
        heads += sig.arms.len();
        signals.push(sig);
    }
    w.travel.signals = signals;
    // (Green on the first arm before the first tick changes them: `change_lights`.)
    w.traffic
        .lights
        .resize(w.travel.signals.len(), Light::default());
    for car in &mut w.cars {
        car.reclaim(&w.travel);
    }
}

impl World {
    /// The junctions' traffic lights worked out again (`traffic::settle`), and the cars' claims
    /// with them: for a test that changes the roads after the world is made.
    #[doc(hidden)]
    pub fn settle_lights(&mut self) {
        settle(self);
    }
}

/// The tick's claims, before the cars go on (`Traffic::begin`).
pub(crate) fn begin(w: &mut World) {
    crate::cars::track_stall(w);
    crate::cars::sight(w);
    crate::cars::pass_stands(w);
    crate::cars::unjam(w);
    let mut movers = Vec::new();
    let mut places = Vec::new();
    crate::cars::movers(w, &mut movers, &mut places);
    let tick = w.tick;
    w.traffic.begin(tick, movers, &places, &w.travel.signals);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A car's body (4.6 by 1.88 m) at `at` facing `yaw`.
    fn rect(at: [f32; 2], yaw: f32) -> Rect {
        Rect {
            at,
            yaw,
            half: [2.3, 0.94],
        }
    }

    /// A claim from 10 to 20 ticks along its route through `points` (each facing on to the next).
    fn claim(points: &[[f32; 2]]) -> Claim {
        let n = points.len();
        let samples = (0..n)
            .map(|k| {
                let (a, b) = if k + 1 < n {
                    (points[k], points[k + 1])
                } else {
                    (points[k - 1], points[k])
                };
                let yaw = (b[0] - a[0]).atan2(b[1] - a[1]);
                Sample {
                    t: 10.0 + 10.0 * k as f32 / (n - 1) as f32,
                    d: k as f32,
                    rect: rect(points[k], yaw),
                    way: [yaw.sin(), yaw.cos()],
                    pass: 0.0,
                }
            })
            .collect();
        Claim::new(10.0, 20.0, samples)
    }

    /// One standing at `at` facing `yaw`, waiting for nobody on the road.
    fn standing(key: u64, at: [f32; 2], yaw: f32) -> Mover {
        Mover {
            key,
            rect: rect(at, yaw),
            way: [yaw.sin(), yaw.cos()],
            pass: 0.0,
            moving: false,
            ahead: Vec::new(),
            parked: false,
            waits: None,
        }
    }

    #[test]
    fn an_old_ask_waiting_its_turn_after_one_held_up_by_a_later_one_lets_that_one_go() {
        // (Mirrormere's: the first, its way across where the third stands; the second, asked a
        // tick after it, its way across the first's; the third, asked long after, its way
        // across the second's only. Each waited for the next for good: the third for the second,
        // an old ask, the second its turn after the first, the first for the third's body.)
        let (first, second, third) = (1 << 63 | 11, 1 << 63 | 18, 1 << 63 | 9);
        let c_first = claim(&[
            [-12.0, 0.0],
            [-6.0, 0.0],
            [0.0, 0.0],
            [6.0, 0.0],
            [12.0, 0.0],
        ]);
        let c_second = claim(&[
            [6.0, -6.0],
            [6.0, 0.0],
            [6.0, 6.0],
            [6.0, 12.0],
            [6.0, 18.0],
        ]);
        let c_third = claim(&[
            [0.0, 6.0],
            [0.0, 9.0],
            [2.0, 12.0],
            [6.0, 14.0],
            [12.0, 14.0],
        ]);
        let movers = || {
            vec![
                standing(first, [-15.0, 0.0], std::f32::consts::FRAC_PI_2),
                standing(second, [6.0, -9.0], 0.0),
                standing(third, [0.0, 2.0], 0.0),
            ]
        };
        let places = [(first, 0, 9.5), (second, 0, 9.5), (third, 0, 9.5)];
        let mut traffic = Traffic::default();
        let late = 2 + OLD_ASK + 10;
        for tick in 1..late + 20 {
            traffic.may_go(tick, first, 0, std::slice::from_ref(&c_first), 9.5, 10.5);
            if tick >= 2 {
                traffic.may_go(tick, second, 0, std::slice::from_ref(&c_second), 9.5, 10.5);
            }
            if tick >= late && !traffic.holding(third) {
                traffic.may_go(tick, third, 0, std::slice::from_ref(&c_third), 9.5, 10.5);
            }
            traffic.begin(tick + 1, movers(), &places, &[]);
        }
        assert!(
            traffic.waits_for(first).contains(&third),
            "the first waits for the third's body"
        );
        assert!(
            traffic.holding(third),
            "the third goes first: waits {:?}",
            traffic.waits_for(third)
        );
    }

    #[test]
    fn one_coming_up_to_wait_in_the_way_of_an_old_ask_stops_short() {
        // (Crown Quarter's: a taxi waiting to turn across a junction's mouth, where those coming
        // up the other arm stand at its edge, each in its way and let go first as it asked: one
        // after another kept it waiting an hour. Asked long ago, the next stops short.)
        let (waiting, coming) = (1 << 63 | 40, 1 << 63 | 18);
        let across = claim(&[
            [-12.0, 0.0],
            [-6.0, 0.0],
            [0.0, 0.0],
            [6.0, 0.0],
            [12.0, 0.0],
        ]);
        let mine = claim(&[
            [0.0, -2.0],
            [0.0, 2.0],
            [0.0, 6.0],
            [0.0, 10.0],
            [0.0, 14.0],
        ]);
        // (Held up meanwhile by one standing in its way further on.)
        let right = std::f32::consts::FRAC_PI_2;
        let movers = || {
            vec![
                standing(waiting, [-15.0, 0.0], right),
                standing(1 << 63 | 7, [10.0, 0.0], right),
            ]
        };
        let mut traffic = Traffic::default();
        let mut stops = Vec::new();
        for tick in 1..OLD_ASK + 20 {
            traffic.may_go(tick, waiting, 0, std::slice::from_ref(&across), 9.5, 10.5);
            traffic.begin(tick + 1, movers(), &[(waiting, 0, 9.5)], &[]);
            // (Coming up, 3.4 m short of where it would wait, a tick on 2.9 m.)
            assert!(traffic.may_go(tick + 1, coming, 0, std::slice::from_ref(&mine), 5.0, 6.0));
            let (now, next) = (rect([0.0, -5.4], 0.0), rect([0.0, -4.9], 0.0));
            stops.push(traffic.blocked(coming, &now, &next, [0.0, 1.0], 0.7, false));
        }
        assert_eq!(stops[0], None, "a young ask: on up to it");
        assert_eq!(
            stops.last().copied().flatten(),
            Some(waiting),
            "an old one: it stops short"
        );
        // (Nearer than half that already, it came up before the ask was old: on.)
        let (now, next) = (rect([0.0, -3.4], 0.0), rect([0.0, -2.9], 0.0));
        assert_eq!(
            traffic.blocked(coming, &now, &next, [0.0, 1.0], 0.7, false),
            None
        );
    }
}
