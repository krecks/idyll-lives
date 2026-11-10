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
/// one, one and a half, of drive time): it isn't made to stop short in front of one just granted.
pub(crate) const LOOK: [f32; 3] = [
    crate::drive_ticks(0.5),
    crate::drive_ticks(1.0),
    crate::drive_ticks(1.5),
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
    /// The traffic lights its group goes through, and the phase of the arm it comes in by
    /// (`Travel::signals`, `gate`): it's asked for only on that phase's green.
    pub gate: Option<(u16, u8)>,
    /// The next lit junction its group goes through, too near the first for a body to stand
    /// between them (each box taken out to its crosswalks: `settle`), and the phase of the arm it
    /// comes in by there: asked for only on both greens (`Traffic::green_through`). (Through one
    /// only, one went on past the second's red; or waited at the second's stop line on the first's
    /// crosswalk.)
    pub gate2: Option<(u16, u8)>,
    /// Through two lights (`gate2`): where it comes to the second's stop line (ticks along the
    /// route), its green kept on for it till it's past (`change_lights`).
    pub stop2: f32,
    /// Through lights, where its group begins (ticks along the route: the nose at the stop line,
    /// `gate`): granted, but no further on than that once the light's no longer green, its holder
    /// lets it go and waits for the next green (`Traffic::begin`).
    pub stop_at: f32,
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
            gate2: None,
            stop2: f32::NEG_INFINITY,
            stop_at: f32::NEG_INFINITY,
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

/// How far apart (ticks: a fortieth of a second of drive time, less than a tick: a car on a
/// country road goes about a metre a tick) `trace` looks along a route for its poses.
const TRACE_STEP: f32 = crate::drive_ticks(0.025);

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

/// A junction with traffic lights (`Travel::signals`: every junction where `SIGNAL_ARMS` roads or
/// more meet, `settle`): its middle, and its arms, each with its phase (`SignalArm::phase`: two
/// arms more than `OPPOSITE` apart go together, an arm with none across from it on its own). One
/// phase at a time has green (`Light`); a group through it is asked for only on its arm's phase's
/// green (`Claim::gate`), at its arm's stop line (`SignalArm::stop`), and holding it, one goes on
/// through on amber and red; the claims keep anyone from driving into another in the box as ever
/// (one turning across the way of those coming the other way on the same green waits for them).
/// Someone on foot waiting at its crosswalks (`kerb_stop`) brings its walk phase round
/// (`Signal::walk_phase`): every arm red, and the crosswalks, closed to those on foot otherwise,
/// open. A roundabout's (`Signal::roundabout`) is lit as ever: each arm a phase of its own, its
/// pole just out past the box, cars stopping at the box's edge, and no crosswalks.
#[derive(Debug, Clone)]
pub struct Signal {
    pub at: [f32; 2],
    pub arms: Vec<SignalArm>,
    /// How many phases its arms make (`SignalArm::phase`, from 0).
    pub phases: u8,
    /// On a roundabout's ring (a road of it has an island: `meta.roads[].island`).
    pub roundabout: bool,
    /// How long its walk phase's clearance lasts, once those on foot may no longer step out
    /// (`WALK_OPEN`): its widest arm with a crosswalk crossed at `WALK_PACE` (ticks of real time:
    /// people don't hurry for the cars' clock). 0: no walk phase (a roundabout's, or one with no
    /// crosswalk: country roads meeting).
    pub walk_clear: u64,
}

impl Signal {
    /// Its walk phase (`Light::phase`): the one after the cars' (none on a roundabout).
    pub fn walk_phase(&self) -> Option<u8> {
        (self.walk_clear > 0).then_some(self.phases)
    }

    /// How many turns its lights take round: the cars' phases, and the walk phase.
    pub fn turns(&self) -> u8 {
        self.phases + u8::from(self.walk_clear > 0)
    }

    /// Whether `p` is on one of its crosswalks (`kerb_stop`): an arm's with one
    /// (`SignalArm::crosswalk`), from `walk_from` to `walk_to` out, in its carriageway (`KERB_IN`
    /// in from the kerb: one walking along the pavement by it isn't on it). Its box and the
    /// carriageway either side of the crosswalks nobody on foot walks on at all
    /// (`Travel::junction_tiles`). None of a roundabout's; a country road's arm is crossed as
    /// anywhere else, out past the box.
    pub fn crossing(&self, p: [f32; 2]) -> bool {
        self.walk_clear > 0
            && self.arms.iter().filter(|a| a.crosswalk).any(|a| {
                let (along, across) = a.out(self.at, p);
                (a.walk_from..=a.walk_to).contains(&along) && across <= a.kerb - KERB_IN
            })
    }

    /// Whether `p` is where those on foot never walk (`Travel::junction_tiles`): its box, between
    /// the kerbs out to the crosswalks (out past the box on an arm without one), and on an arm with
    /// one, from its far side out to `KEEP_OFF` past the stop line; not on a crosswalk itself.
    /// (Not a roundabout's.)
    fn off_limits(&self, p: [f32; 2]) -> bool {
        if self.roundabout {
            return false;
        }
        let mut off = false;
        for a in &self.arms {
            let (along, across) = a.out(self.at, p);
            if across <= a.kerb && along >= a.walk_from {
                if a.crosswalk && along <= a.walk_to {
                    return false;
                }
                off |= a.crosswalk && along <= a.stop + KEEP_OFF;
            }
            let (along, across) = a.inside(self.at, p);
            off |= across <= a.kerb && (-0.01..a.walk_from).contains(&along);
        }
        off
    }

    /// How far from its middle its crosswalks reach at most (m).
    fn crossing_reach(&self) -> f32 {
        self.arms
            .iter()
            .filter(|a| a.crosswalk)
            .map(|a| a.walk_to.max(dist(self.at, a.span[0]) + CROSSWALK) + a.kerb)
            .fold(0.0, f32::max)
    }
}

/// An arm of a lit junction (`Signal`): the way out along it (a unit vector), and its signal head:
/// the pole on the kerb to the right of the way in, at its stop line (a roundabout's, a little out
/// past the box), and which way it faces (as a resident's yaw: towards those coming in). Its phase (`Signal::phases`), its road
/// (`Graph::roads` index) and how far its kerbs are from the middle of it (m); its crosswalk, from
/// `walk_from` to `walk_to` m out from the junction's middle (from the corner, where the kerb's
/// return round from the next arm ends, `KERB_RETURN`, or the box's edge, whichever is further
/// out: `CROSSWALK` wide), and its stop line `stop` m out (`STOP_GAP` past the crosswalk), where
/// cars stop on red with their noses. Whether it has a crosswalk at all (`crosswalk`): a street's
/// or a lane's (one with sidewalks, as the renderer draws its zebra and the pedestrians' head), not
/// a country road's nor a roundabout's; those on foot wait for the walk phase only at those.
/// Where its road's middle is at its crosswalk and out to `KEEP_OFF` past its stop line (`span`:
/// at `walk_from` and at `stop + KEEP_OFF` out along the road, round its bend): its crosswalk and
/// the carriageway either side of it are measured across from that (`SignalArm::out`), not from
/// `way`, which a road bending away from the junction's middle, or a junction's middle off its
/// road (two junctions merged), leaves metres off by then: the strip kept off by the stop line
/// took in a house's gate on the pavement, and shut it in.
#[derive(Debug, Clone, Copy)]
pub struct SignalArm {
    pub way: [f32; 2],
    pub pole: [f32; 2],
    pub yaw: f32,
    pub phase: u8,
    pub road: u32,
    pub kerb: f32,
    pub walk_from: f32,
    pub walk_to: f32,
    pub stop: f32,
    pub crosswalk: bool,
    pub span: [[f32; 2]; 2],
}

impl SignalArm {
    /// Where `p` is by its crosswalk (`span`): how far out (m from the junction's middle, as
    /// `walk_from`, `stop`) and how far from its road's middle. A span too short to have a way
    /// (its road ends there): along `way` from `at`, the junction's middle.
    fn out(&self, at: [f32; 2], p: [f32; 2]) -> (f32, f32) {
        let [a, b] = self.span;
        let l = dist(a, b);
        let (o, w, from) = if l > 0.1 {
            (a, [(b[0] - a[0]) / l, (b[1] - a[1]) / l], self.walk_from)
        } else {
            (at, self.way, 0.0)
        };
        let d = [p[0] - o[0], p[1] - o[1]];
        (
            from + d[0] * w[0] + d[1] * w[1],
            (d[0] * w[1] - d[1] * w[0]).abs(),
        )
    }

    /// Where `p` is along its box (from `at`, the junction's middle, out to where its crosswalk
    /// begins, `span`'s first): how far out, and how far from that line.
    fn inside(&self, at: [f32; 2], p: [f32; 2]) -> (f32, f32) {
        let a = self.span[0];
        let l = dist(at, a);
        let w = if l > 0.1 {
            [(a[0] - at[0]) / l, (a[1] - at[1]) / l]
        } else {
            self.way
        };
        let d = [p[0] - at[0], p[1] - at[1]];
        (d[0] * w[0] + d[1] * w[1], (d[0] * w[1] - d[1] * w[0]).abs())
    }
}

/// How far a lit junction's lights are through a phase's turn (`Light`): its green, its amber,
/// then all red while the box clears. The walk phase's (`Signal::walk_phase`): those on foot may
/// step out, then its clearance, then all red.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stage {
    #[default]
    Green,
    Amber,
    Clear,
}

/// What a signal head shows (`Traffic::light`, `snapshot::light::STATE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Head {
    Red,
    Amber,
    Green,
    /// Red for the cars, and walk for those on foot (the walk phase).
    Walk,
}

/// A lit junction's lights now (`Traffic::lights`): whose turn it is (a phase, `SignalArm::phase`,
/// or the walk phase, `Signal::walk_phase`), how far through it, and since when (the tick).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Light {
    pub phase: u8,
    pub stage: Stage,
    pub since: u64,
}

/// A junction where this many roads for cars meet may have traffic lights (`Signal`).
pub(crate) const SIGNAL_ARMS: usize = 3;
/// How long a phase's green lasts at least, its amber, and all red after while the box
/// clears (ticks of drive time, as the cars go through).
const MIN_GREEN: u64 = crate::drive_secs(5.0);
/// Every phase's turn comes round in this long at most (ticks of drive time): a phase's green
/// lasts no longer than its share of it (`change_lights`).
const CYCLE: u64 = crate::drive_secs(50.0);
const AMBER: u64 = crate::drive_secs(2.0);
/// Through two lit junctions too near to wait between (`Claim::gate2`): the second's green is
/// kept on for those waiting at the first's stop line or granted through both, from when one
/// first is, for `CYCLE` at most (the first's turn comes round for them meanwhile); one is
/// granted through only with `GREEN_HELD` of that still to come (its way to the second's stop
/// line).
const GREEN_HELD: u64 = crate::drive_secs(10.0);
const ALL_RED: u64 = crate::drive_secs(4.0);
/// A route this near a lit junction's middle goes through its lights (m, `gate`), and its arm is
/// the one it's on where it first comes this near (`GATE_REACH`).
const GATE_THROUGH: f32 = BOX_LEAST + 1.0;
const GATE_REACH: f32 = BOX_MOST + 2.0;
/// How far out past its arm's box a roundabout's signal pole stands (another's stands at the stop
/// line), and how far in from the kerb on the pavement (or the road's edge) any stands (m).
const POLE_OUT: f32 = 1.0;
const POLE_KERB: f32 = 0.4;
/// Two arms of a junction whose ways out are further apart than 150° (the cosine) are across from
/// each other: green together (`SignalArm::phase`).
const OPPOSITE: f32 = -0.866;
/// The radius of a junction's kerbs where they turn round from one arm to the next (m, the
/// renderer's), how wide a crosswalk is along its arm, and how far out past it the stop line is
/// (m, `SignalArm`).
pub const KERB_RETURN: f32 = 3.0;
pub const CROSSWALK: f32 = 3.0;
pub const STOP_GAP: f32 = 0.5;
/// How far out past an arm's stop line its carriageway is off-limits to those on foot (m,
/// `Signal::off_limits`): they cross on the zebra, not just past it; and how far in from the kerb
/// one on the crosswalk is in the road (m, `Signal::crossing`).
pub const KEEP_OFF: f32 = 3.0;
/// How far round a lit junction's tiles kept off (tiles) the ground is looked over for any they
/// would cut off (`left_open`).
const KEEP_OFF_ROOM: i32 = 4;
const KERB_IN: f32 = 0.25;
/// An arm's crosswalk is measured along its own road (`SignalArm::span`) where that goes out within
/// 30° of its way (the cosine), its crosswalk this near as far out as the arm's (m); else along
/// `way`.
const SPAN_ALONG: f32 = 0.866;
const SPAN_OFF: f32 = 2.0;
/// The walk phase (`Signal::walk_phase`): how long those on foot may step out (ticks of real time,
/// as people walk), and the pace its clearance gives the slowest to get across the widest arm
/// (m a second; `Signal::walk_clear`), `WALK_CLEAR_LEAST` at least (s).
const WALK_OPEN: u64 = crate::secs(5.0);
const WALK_PACE: f32 = 1.0;
const WALK_CLEAR_LEAST: f32 = 3.0;

/// An arm's road from its junction (`signal_at`): which, where along it and which way (+1, -1),
/// and the stretch of it between its ends' nodes.
type ArmRoad = (u32, f32, f32, f32, f32);

/// The lights of the junction at node `n` of `g` (`Signal`): its arms for cars, each with its pole
/// on the kerb to the right of the way in, at its stop line, its phase and its crosswalk
/// (`SignalArm`). `ring`: per road of `g`, whether it's a roundabout's. None: fewer
/// than `SIGNAL_ARMS` arms.
pub(crate) fn signal_at(g: &Graph, n: usize, ring: &[bool]) -> Option<Signal> {
    let p = g.nodes[n];
    // (Each arm's way out, how far from its middle its kerb is, its road from the node: which,
    // where along it and which way; and how far out along it the junction may reach: half way to
    // the next junction, else its stretch's length.)
    let mut arms: Vec<([f32; 2], f32, ArmRoad, f32)> = Vec::new();
    for e in g.edges.iter() {
        let road = &g.roads[e.road as usize];
        if !road.drivable() {
            continue;
        }
        for (end, s, k, far) in [(e.a, e.s0, 1.0f32, e.b), (e.b, e.s1, -1.0, e.a)] {
            if end as usize != n {
                continue;
            }
            let q = road.at((s + k * ARM_LOOK.min(e.len())).clamp(e.s0, e.s1));
            let l = dist(p, q);
            if l > 0.1 {
                let room = if g.degree(far as usize) >= SIGNAL_ARMS {
                    e.len() / 2.0
                } else {
                    e.len()
                };
                arms.push((
                    [(q[0] - p[0]) / l, (q[1] - p[1]) / l],
                    crate::cars::kerb_of(road.width, road.kind),
                    (e.road, s, k, e.s0, e.s1),
                    room,
                ));
            }
        }
    }
    if arms.len() < SIGNAL_ARMS {
        return None;
    }
    let roundabout = arms
        .iter()
        .any(|a| ring.get(a.2.0 as usize).copied().unwrap_or(false));
    // (Arms by the way they go round, for the lights' turns.)
    arms.sort_by(|a, b| a.0[0].atan2(a.0[1]).total_cmp(&b.0[0].atan2(b.0[1])));
    let dot = |a: [f32; 2], b: [f32; 2]| a[0] * b[0] + a[1] * b[1];
    // (Phases in that order round: an arm, and the one furthest round from it past `OPPOSITE`
    // not yet in one; on a roundabout, an arm each.)
    let mut phase = vec![u8::MAX; arms.len()];
    let mut phases = 0u8;
    for i in 0..arms.len() {
        if phase[i] != u8::MAX {
            continue;
        }
        phase[i] = phases;
        let across = (i + 1..arms.len())
            .filter(|&j| phase[j] == u8::MAX && dot(arms[i].0, arms[j].0) < OPPOSITE)
            .min_by(|&a, &b| dot(arms[i].0, arms[a].0).total_cmp(&dot(arms[i].0, arms[b].0)));
        if let Some(j) = across.filter(|_| !roundabout) {
            phase[j] = phases;
        }
        phases += 1;
    }
    let reach = |a: [f32; 2]| {
        arms.iter()
            .filter(|b| b.0 != a)
            .map(|b| {
                let cos = dot(a, b.0);
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
    // (Where an arm's kerb turns round into the next's ends, out along it: where the two kerbs
    // would meet, and on round the return's radius, `R / tan(θ / 2)`; none with an arm across.)
    let corner = |a: usize| {
        let (wa, ka) = (arms[a].0, arms[a].1);
        arms.iter()
            .enumerate()
            .filter(|&(b, _)| b != a)
            .map(|(_, &(wb, kb, ..))| {
                let cos = dot(wa, wb).clamp(-1.0, 1.0);
                if cos < OPPOSITE {
                    return 0.0;
                }
                let sin = (1.0 - cos * cos).sqrt().max(1e-3);
                (kb + ka * cos) / sin + KERB_RETURN * (1.0 + cos) / sin
            })
            .fold(0.0, f32::max)
    };
    let arms: Vec<SignalArm> = arms
        .iter()
        .enumerate()
        .map(|(i, &(way, kerb, (road, s, k, s0, s1), room))| {
            // (The crosswalk from the corner or the box's edge, the stop line past it; no further
            // out than the junction may reach along it, the box's edge at least.)
            let edge = reach(way);
            let walk_from = edge.max(corner(i)).min(BOX_MOST);
            let mut walk_to = walk_from + CROSSWALK;
            let mut stop = walk_to + STOP_GAP;
            if stop > room {
                stop = room.max(edge).min(stop);
                walk_to = walk_to.min(stop - STOP_GAP);
            }
            let walk_from = walk_from.min(walk_to - CROSSWALK).max(0.0);
            // (Along its own road, round its bend: at the stop line (a roundabout's, out past the
            // box), and the way out there; coming in, its right is the way out's left. On the
            // pavement, `POLE_KERB` in from the kerb; on a country road, as far out on the verge.
            // A road ending at another's kerb, a T's arm, measured from the junction's middle.)
            let r = &g.roads[road as usize];
            let short = dist(p, r.at(s));
            let raw = |out: f32| s + k * (out - short).max(0.0);
            let on = |out: f32| raw(out).clamp(s0, s1);
            // (Along its own road where that goes out the arm's way, about as far out; else out
            // along `way` as ever: past the end of its stretch, or from a junction's middle merged
            // off it, the box went off towards a garden's chair, Frostridge.)
            let straight =
                [walk_from, stop + KEEP_OFF].map(|o| [p[0] + way[0] * o, p[1] + way[1] * o]);
            let along = [r.at(on(walk_from)), r.at(on(stop + KEEP_OFF))];
            let l = dist(along[0], along[1]);
            let span = if [walk_from, stop + KEEP_OFF]
                .iter()
                .all(|&o| (s0..=s1).contains(&raw(o)))
                && l > 0.1
                && ((along[1][0] - along[0][0]) * way[0] + (along[1][1] - along[0][1]) * way[1]) / l
                    > SPAN_ALONG
                && (dist(p, along[0]) - walk_from).abs() < SPAN_OFF
            {
                along
            } else {
                straight
            };
            let out = if roundabout { edge + POLE_OUT } else { stop };
            let at = on(out);
            let t = r.tangent(at);
            let out = [t[0] * k, t[1] * k];
            let right = [out[1], -out[0]];
            let m = r.at(at);
            let side = kerb + POLE_KERB;
            SignalArm {
                way,
                pole: [m[0] + right[0] * side, m[1] + right[1] * side],
                yaw: out[0].atan2(out[1]),
                phase: phase[i],
                road,
                kerb,
                walk_from,
                walk_to,
                stop,
                span,
                crosswalk: !roundabout
                    && matches!(
                        r.kind,
                        crate::travel::RoadKind::Street | crate::travel::RoadKind::Lane
                    ),
            }
        })
        .collect();
    // (Its widest crosswalk crossed at an old person's pace, once they may no longer step out;
    // none, no walk phase.)
    let widest = arms
        .iter()
        .filter(|a| a.crosswalk)
        .map(|a| 2.0 * a.kerb)
        .fold(0.0, f32::max);
    let walk_clear = if widest <= 0.0 {
        0
    } else {
        crate::secs((widest / WALK_PACE).max(WALK_CLEAR_LEAST))
    };
    Some(Signal {
        at: p,
        arms,
        phases,
        roundabout,
        walk_clear,
    })
}

/// The lights `claims`' groups go through (`Claim::gate`), and the phase of the arm each comes in
/// by: of `signals`, the first along its way whose middle a group comes by (within `GATE_THROUGH`:
/// through its box; not the first in the list it comes within `GATE_REACH` of, two lit junctions
/// close together; nor one it only comes near, out of a driveway by a junction, which kept a car
/// coming out of one waiting for the junction's green in the way of those turning in). A group through lights begins where a body `len` long coming
/// in along `trace` has its nose at its arm's stop line (`SignalArm::stop`; but a roundabout's, at
/// the box's edge as ever): it waits there on red, short of the crosswalk.
pub(crate) fn gate(claims: &mut [Claim], signals: &[Signal], trace: &Trace, len: f32) {
    // (A lit junction a group goes through: when it first comes within reach of it and how near
    // it comes, which, and where it was then.)
    type Near = ((f32, f32), usize, [f32; 2]);
    // (The group before, through lights: its first part, and where it begins.)
    let mut before: Option<(usize, f32)> = None;
    let mut k = 0;
    while k < claims.len() {
        let g = claims[k].group;
        let end = k + claims[k..].iter().take_while(|c| c.group == g).count();
        let b = claims[k..end].iter().fold(
            [
                f32::INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::NEG_INFINITY,
            ],
            |b, c| {
                [
                    b[0].min(c.bounds[0]),
                    b[1].min(c.bounds[1]),
                    b[2].max(c.bounds[2]),
                    b[3].max(c.bounds[3]),
                ]
            },
        );
        // (Each lit junction near it: whether it goes through its box, when it first comes within
        // reach of it and where, and how near it comes.)
        let mut through: Vec<Near> = Vec::new();
        for (i, sig) in signals.iter().enumerate() {
            let p = sig.at;
            if p[0] < b[0] - GATE_REACH
                || p[0] > b[2] + GATE_REACH
                || p[1] < b[1] - GATE_REACH
                || p[1] > b[3] + GATE_REACH
            {
                continue;
            }
            let mut first: Option<(f32, [f32; 2])> = None;
            let mut near = f32::INFINITY;
            for s in claims[k..end].iter().flat_map(|c| &c.samples) {
                let d = dist(s.rect.at, p);
                if d < GATE_REACH {
                    near = near.min(d);
                    if first.is_none_or(|f| s.t < f.0) {
                        first = Some((s.t, s.rect.at));
                    }
                }
            }
            let Some((t, at)) = first.filter(|_| near < GATE_THROUGH) else {
                continue;
            };
            through.push(((t, near), i, at));
        }
        through.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
        let arm_of = |&(_, i, at): &Near| {
            let sig = &signals[i];
            let d = [at[0] - sig.at[0], at[1] - sig.at[1]];
            let arm = (0..sig.arms.len())
                .max_by(|&a, &b| {
                    let dot = |x: usize| sig.arms[x].way[0] * d[0] + sig.arms[x].way[1] * d[1];
                    dot(a).total_cmp(&dot(b))
                })
                .unwrap_or(0);
            (i, arm)
        };
        let gate = through.first().map(arm_of);
        let gate2 = through.get(1).map(arm_of);
        // (Through a second, where its nose comes to that one's stop line.)
        let stop2 = gate2.and_then(|(i, arm)| {
            let (at, out) = (signals[i].at, signals[i].arms[arm].stop + len / 2.0);
            claims[k..end]
                .iter()
                .flat_map(|c| &c.samples)
                .filter(|s| dist(s.rect.at, at) <= out)
                .map(|s| s.t)
                .reduce(f32::min)
        });
        for c in &mut claims[k..end] {
            c.gate = gate.map(|(i, arm)| (i as u16, signals[i].arms[arm].phase));
            c.gate2 = gate2.map(|(i, arm)| (i as u16, signals[i].arms[arm].phase));
            c.stop2 = stop2.unwrap_or(f32::NEG_INFINITY);
        }
        if let Some((i, arm)) = gate
            && !signals[i].roundabout
            && !trace.is_empty()
        {
            let (sig, a) = (&signals[i], &signals[i].arms[arm]);
            // (Back along its way from where its body first touches the box, while it's coming
            // in along the arm, between its kerbs, its nose short of the stop line; no further
            // back than the end of the part before.)
            let least = if k > 0 { claims[k - 1].to } else { 0.0 };
            let nose = |j: usize| {
                let (_, p, yaw, back, _) = trace[j];
                let (s, c) = yaw.sin_cos();
                let n = [p[0] + s * len / 2.0, p[1] + c * len / 2.0];
                let d = [n[0] - sig.at[0], n[1] - sig.at[1]];
                let along = d[0] * a.way[0] + d[1] * a.way[1];
                let across = (d[0] * a.way[1] - d[1] * a.way[0]).abs();
                let inward = s * a.way[0] + c * a.way[1] < -0.5;
                (along, !back && inward && across <= a.kerb + 1.0)
            };
            let start = trace
                .partition_point(|s| s.0 < claims[k].from)
                .min(trace.len() - 1);
            let mut j = start;
            let mut short = false;
            while j > 0 {
                let (along, on) = nose(j);
                if !on || along >= a.stop {
                    break;
                }
                if trace[j - 1].0 < least {
                    short = true;
                    break;
                }
                j -= 1;
            }
            // (Between that pose and the next, where the nose is at the line.)
            let mut from = trace[j].0;
            if let ((at, true), Some((next, true))) =
                (nose(j), trace.get(j + 1).map(|_| nose(j + 1)))
                && at >= a.stop
                && next < a.stop
            {
                from += (trace[j + 1].0 - from) * (at - a.stop) / (at - next);
            }
            // (Its body first touching the box out past the line, taken out to the crosswalks, a
            // step before its nose is at the line: at the line all the same.)
            let from = if j == start {
                from
            } else {
                claims[k].from.min(from)
            };
            let short = short || from < least;
            // (Its stop line before it's out of the group before through lights: no room to wait
            // between the two. One group with that one, asked for on both greens, from its stop
            // line: out of the one, it went on past the other's stop line on red.)
            if let Some((first, at)) =
                before.filter(|&(first, _)| short && claims[first].gate.is_some())
            {
                let (group, gate) = (claims[first].group, claims[first].gate);
                let this = claims[k].gate;
                for c in &mut claims[first..k] {
                    c.gate2 = c.gate2.or(this);
                    c.stop2 = from;
                }
                for c in &mut claims[k..end] {
                    c.group = group;
                    c.gate = gate;
                    c.gate2 = this;
                    c.stop_at = at;
                    c.stop2 = from;
                }
                k = end;
                continue;
            }

            claims[k].from = from;
            for c in &mut claims[k..end] {
                c.stop_at = from;
            }
            before = Some((k, from));
        } else {
            before = None;
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
const KEPT_ASK: f32 = crate::drive_ticks(3.25);

/// An ask this old (ticks of drive time) yields only to those it waits for itself
/// (`Traffic::begin`, rule 2).
const OLD_ASK: u64 = crate::drive_secs(20.0);

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
    /// Per lit junction, the phases anyone waits at and comes through (`change_lights`'s own,
    /// kept so it isn't made anew every tick).
    lit_phases: Vec<[u64; 4]>,
    /// Per lit junction, since when its green is held for those through it from a lit junction
    /// just before (`Claim::gate2`, `GREEN_HELD`), and whether one may still be granted through
    /// it (`Traffic::green_through`).
    held: Vec<Option<u64>>,
    fresh: Vec<bool>,
    /// The lit junctions someone on foot waits at a crosswalk of (`Signal` indices, this tick:
    /// `kerb_stop`, `traffic::begin`), for their walk phase.
    pub(crate) walkers: Vec<u16>,
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
    /// phase's green (none: no lights).
    /// Whether a group through claim `c`'s lights may be asked for now: green at each of them
    /// (`Claim::gate`, `Claim::gate2`).
    pub(crate) fn green_through(&self, c: &Claim) -> bool {
        self.green(c.gate)
            && self.green(c.gate2)
            && c.gate2
                .is_none_or(|(s, _)| self.fresh.get(usize::from(s)).copied().unwrap_or(true))
    }

    pub(crate) fn green(&self, gate: Option<(u16, u8)>) -> bool {
        gate.is_none_or(|(s, phase)| {
            self.lights
                .get(s as usize)
                .is_none_or(|l| l.stage == Stage::Green && l.phase == phase)
        })
    }

    /// What arm `arm` of lit junction `s` (`signals[s]`) shows: its phase's green or amber, else
    /// red; in the walk phase (`Signal::walk_phase`), while those on foot may step out, walk.
    pub(crate) fn light(&self, signals: &[Signal], s: usize, arm: usize) -> Option<Head> {
        let (l, sig) = (self.lights.get(s)?, signals.get(s)?);
        let phase = sig.arms.get(arm)?.phase;
        Some(match l.stage {
            Stage::Green if sig.walk_phase() == Some(l.phase) => Head::Walk,
            Stage::Green if l.phase == phase => Head::Green,
            Stage::Amber if l.phase == phase => Head::Amber,
            _ => Head::Red,
        })
    }

    /// The lights a tick on (`Light`): a phase's green `MIN_GREEN` at least, on while anyone comes
    /// to it (an ask through it, or one holding a claim through it still out of the box) and its
    /// share of `CYCLE` at most, and only while someone waits at another (or on foot at the
    /// crosswalks, `Traffic::walkers`); then amber (`AMBER`), all red while the box clears
    /// (`ALL_RED`), and the next phase round that anyone waits at. The walk phase: those on foot
    /// may step out for `WALK_OPEN`, then its clearance (`Signal::walk_clear`), then all red, and
    /// the cars' phase after it. Nobody waiting anywhere, it stays green where it is (the cars').
    fn change_lights(&mut self, tick: u64, signals: &[Signal]) {
        self.lights.resize(signals.len(), Light::default());
        // (Per lit junction, the phases anyone waits at and comes through, bit `phase` of each:
        // one pass over the asks and the holds for every junction, not one per junction and
        // phase; and those on foot, at its walk phase's bit.)
        let bit = |phase: u8| 1u64.checked_shl(u32::from(phase)).unwrap_or(0);
        self.fresh.resize(signals.len(), true);
        self.held.resize(signals.len(), None);
        let mut phases = std::mem::take(&mut self.lit_phases);
        phases.clear();
        phases.resize(signals.len(), [0, 0, 0, 0]);
        for a in &self.asks {
            if a.seen + 1 >= tick
                && let Some(first) = a.parts.first()
            {
                for (s, phase) in [first.1.gate, first.1.gate2].into_iter().flatten() {
                    if let Some(m) = phases.get_mut(usize::from(s)) {
                        m[0] |= bit(phase);
                    }
                }
                // (Waiting through two: the one of the two listed first leads, the other follows.
                // The leader on its green for it, that's held, and the other's phase it needs is
                // wanted there, next. Each holding the other's green for one waiting at its own
                // red, two lights in step kept two cars crossing each other's way at red for
                // hours: Hollyglade.)
                if let (Some(a), Some(b)) = (first.1.gate, first.1.gate2) {
                    let (lead, follow) = if a.0 <= b.0 { (a, b) } else { (b, a) };
                    if self.green(Some(lead)) {
                        if let Some(m) = phases.get_mut(usize::from(lead.0)) {
                            m[2] |= bit(lead.1);
                        }
                        if let Some(m) = phases.get_mut(usize::from(follow.0)) {
                            m[3] |= bit(follow.1);
                        }
                    }
                }
            }
        }
        for h in &self.holds {
            if h.at < h.claim.from + 1.0
                && let Some((s, phase)) = h.claim.gate
                && let Some(m) = phases.get_mut(usize::from(s))
            {
                m[1] |= bit(phase);
            }
            // (Through two lights, granted on both greens: the second's on till it's past its
            // stop line.)
            if h.at < h.claim.stop2
                && let Some((s, phase)) = h.claim.gate2
                && let Some(m) = phases.get_mut(usize::from(s))
            {
                m[2] |= bit(phase);
            }
        }
        for &s in &self.walkers {
            if let (Some(m), Some(walk)) = (
                phases.get_mut(usize::from(s)),
                signals.get(usize::from(s)).and_then(Signal::walk_phase),
            ) {
                m[0] |= bit(walk);
            }
        }
        for (i, sig) in signals.iter().enumerate() {
            let waits = |phase: u8| phases[i][0] & bit(phase) != 0;
            let coming = |phase: u8| phases[i][1] & bit(phase) != 0;
            let n = sig.turns();
            let mut l = self.lights[i];
            // (Loaded with another junction's, or the roads changed: the first phase's green.)
            if l.phase >= n {
                l = Light::default();
            }
            let walk = sig.walk_phase() == Some(l.phase);
            let on = tick.saturating_sub(l.since);
            // (Every phase round in `CYCLE` at most: nobody waits at a red light much longer.)
            let most = (CYCLE / u64::from(n.max(2) - 1))
                .saturating_sub(AMBER + ALL_RED)
                .max(MIN_GREEN);
            let next = |phase: u8| (1..n).map(|k| (phase + k) % n).find(|&x| waits(x));
            let stage = |stage: Stage| Light {
                stage,
                since: tick,
                ..l
            };
            self.lights[i] = match l.stage {
                Stage::Green if walk => {
                    if on >= WALK_OPEN {
                        stage(Stage::Amber)
                    } else {
                        l
                    }
                }
                Stage::Green => {
                    let held = phases[i][2] & bit(l.phase) != 0;
                    let wanted = phases[i][3];
                    let busy = waits(l.phase) || coming(l.phase) || held;
                    // (Held, on till `CYCLE` from when it first was; or wanted, by a lit junction
                    // next to it on its green for one through both, on while that is, `CYCLE` at
                    // most: one at its red, in the way of one through both, waited for good,
                    // Wheat Bottom.)
                    let kept = held && self.held[i].is_none_or(|t| tick < t + CYCLE)
                        || wanted & bit(l.phase) != 0 && on < CYCLE;
                    // (Another phase wanted: on to it as soon as may be.)
                    let follow = wanted != 0 && !kept && on >= MIN_GREEN;
                    match next(l.phase) {
                        Some(_) if !kept && on >= most || on >= MIN_GREEN && !busy || follow => {
                            stage(Stage::Amber)
                        }
                        _ => l,
                    }
                }
                Stage::Amber if on >= if walk { sig.walk_clear } else { AMBER } => {
                    stage(Stage::Clear)
                }
                // (After the walk phase, nobody waiting: the cars' next phase, to rest on. One
                // wanted by a lit junction next to it, first; not again straight after its own
                // turn: those at the others' reds waited for good, one stopped in the box among
                // them, Wheat Bottom.)
                Stage::Clear if on >= ALL_RED => Light {
                    phase: (1..=n)
                        .map(|k| (l.phase + k) % n)
                        .find(|&x| phases[i][3] & bit(x) != 0 && x != l.phase)
                        .filter(|_| phases[i][3] & bit(l.phase) == 0)
                        .or_else(|| next(l.phase))
                        .unwrap_or(if walk { 0 } else { l.phase }),
                    stage: Stage::Green,
                    since: tick,
                },
                _ => l,
            };
            let now = self.lights[i];
            let held = now.stage == Stage::Green && phases[i][2] & bit(now.phase) != 0;
            // (A new hold as its green begins, or its last one over and it's still on: nobody
            // waiting at another phase.)
            self.held[i] = if held {
                Some(
                    self.held[i]
                        .filter(|&t| now.since < tick && tick < t + CYCLE)
                        .unwrap_or(tick),
                )
            } else {
                None
            };
            // (Granted through only with its green on a while yet: young, or held long enough
            // still; resting, nobody waiting elsewhere, it stays on till someone is, and then held.)
            self.fresh[i] = now.stage == Stage::Green
                && match self.held[i] {
                    Some(t) => tick + GREEN_HELD < t + CYCLE,
                    None => true,
                };
        }
        self.lit_phases = phases;
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
        // (Granted through lights, but not yet past the stop line, held up there meanwhile: the
        // light no longer green, it stops after all, and asks again on the next green.)
        let lights = &self.lights;
        let green = |gate: Option<(u16, u8)>| {
            gate.is_none_or(|(s, phase)| {
                lights
                    .get(usize::from(s))
                    .is_none_or(|l| l.stage == Stage::Green && l.phase == phase)
            })
        };
        self.holds.retain(|h| {
            h.at > h.claim.stop_at + 1e-3 || green(h.claim.gate) && green(h.claim.gate2)
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
            // (At the lights, not its phase's green: it waits at the stop line, holding nobody up
            // and granted nothing; but who it would wait for on its green, the parts held and the
            // bodies in its way, it names (`by`), as on its green: one standing in its way waiting
            // in turn for it at another phase's red is a ring the waits see, rule 3's and
            // `cars::unjam`'s, not two standing for good by turns, Highholm.)
            let red = !self.green_through(next[0]);
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
                if red {
                    Vec::new()
                } else {
                    pending
                        .iter()
                        .filter(conflicting)
                        .map(|(j, _)| self.asks[*j].key)
                        .collect()
                },
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
            let holds_none = by_holds.is_empty();
            let mut by = by_holds;
            by.extend(bodies);
            if red {
                self.asks[i].coming = coming;
                self.asks[i].bodies_only = false;
                waits.push((i, by));
                continue;
            }
            let free = by.is_empty() && !earlier;
            let bodies_only = holds_none && !earlier;
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

/// Whether vehicle `key` waits at a red light (its ask through lights not on its phase's green): it
/// isn't held up for good, the light changes (for checks).
pub fn at_red(w: &World, key: u64) -> bool {
    w.traffic.asks.iter().any(|a| {
        a.key == key
            && a.seen + 1 >= w.traffic.tick
            && a.parts
                .first()
                .is_some_and(|p| !w.traffic.green_through(&p.1))
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

/// The tiles the lit junctions keep those on foot off (`Signal::off_limits`, `Travel::
/// junction_tiles`), as `signals` have them: blocked, those freed since the last time given back,
/// and walkers finding their way again round where that changed. Never ground cut off: a way
/// over them left open from any they'd shut in (`left_open`).
fn keep_off(w: &mut World, signals: &[Signal]) {
    let mut tiles: Vec<usize> = Vec::new();
    let mut rects: Vec<[i32; 4]> = Vec::new();
    let mut taken = std::collections::HashSet::new();
    let gates: Vec<(i32, i32)> = w
        .plots
        .iter()
        .map(|p| {
            let a = p.arrival_point();
            (a[0].floor() as i32, a[1].floor() as i32)
        })
        .collect();
    for sig in signals.iter().filter(|s| !s.roundabout) {
        let r = sig
            .arms
            .iter()
            .map(|a| (a.stop + KEEP_OFF).max(dist(sig.at, a.span[1])) + a.kerb)
            .fold(0.0, f32::max)
            .ceil();
        let (x0, z0) = (
            (sig.at[0] - r).floor() as i32,
            (sig.at[1] - r).floor() as i32,
        );
        let (x1, z1) = ((sig.at[0] + r).ceil() as i32, (sig.at[1] + r).ceil() as i32);
        let mut mine = Vec::new();
        // (Its arms' roads' middle lines near it: a tile kept off is on one's carriageway, within
        // its kerb of it, whatever the arms' straight ways out say. Out along a way off its road,
        // the box took in a garden's planter, Applestead.)
        let g = &w.travel.graph;
        let lines: Vec<([f32; 2], [f32; 2], f32)> = sig
            .arms
            .iter()
            .flat_map(|a| {
                let pts = &g.roads[a.road as usize].pts;
                pts.windows(2)
                    .filter(|q| dist(q[0], sig.at).min(dist(q[1], sig.at)) <= r + 2.0)
                    .map(|q| (q[0], q[1], a.kerb))
                    .collect::<Vec<_>>()
            })
            .collect();
        let on_road = |q: [f32; 2]| {
            lines
                .iter()
                .any(|&(a, b, kerb)| crate::cars::to_segment(q, a, b) <= kerb)
        };
        for z in z0..z1 {
            for x in x0..x1 {
                let q = [x as f32 + 0.5, z as f32 + 0.5];
                if w.lot.in_bounds(x, z) && sig.off_limits(q) && on_road(q) {
                    mine.push(w.lot.tile_index(x, z));
                }
            }
        }
        if mine.is_empty() {
            continue;
        }
        mine.sort_unstable();
        let open = left_open(w, [x0, z0, x1, z1], &mine, &taken, &gates);
        mine.retain(|i| open.binary_search(i).is_err());
        taken.extend(mine.iter().copied());
        tiles.extend(mine);
        rects.push([x0, z0, x1, z1]);
    }
    tiles.sort_unstable();
    tiles.dedup();
    let old = std::mem::replace(&mut w.travel.junction_tiles, tiles);
    if old == w.travel.junction_tiles {
        return;
    }
    for &i in &old {
        if w.travel.junction_tiles.binary_search(&i).is_err()
            && let Some(b) = w.blocked.get_mut(i)
        {
            *b = w.travel.base_blocked.binary_search(&i).is_ok();
        }
    }
    for &i in &w.travel.junction_tiles {
        if let Some(b) = w.blocked.get_mut(i) {
            *b = true;
        }
    }
    for &i in &old {
        let (x, z) = w.lot.tile_xz(i);
        rects.push([x, z, x + 1, z + 1]);
    }
    for r in rects {
        w.nav_changed_in(r);
    }
}

/// Of `mine` (a lit junction's tiles to keep those on foot off, sorted, within `rect`), those left
/// open so that it cuts no ground off. Of the ground in `rect` and `KEEP_OFF_ROOM` round it,
/// walked as ever (`NavGrid` on the ground, but for the tiles the junctions keep them off: those
/// looked at before it, `taken`, and not the ones they kept them off till now): a
/// piece of it once `mine` are kept off that was joined to the edge's ground and no longer is (a
/// pocket of pavement walled round but for its side onto the road by the stop line, a gate
/// opening onto it) gets the shortest way over `mine` back to it; a place's gate on one of them
/// (`gates`), the same from there.
fn left_open(
    w: &World,
    rect: [i32; 4],
    mine: &[usize],
    taken: &std::collections::HashSet<usize>,
    gates: &[(i32, i32)],
) -> Vec<usize> {
    let [x0, z0, x1, z1] = rect;
    let (x0, z0) = ((x0 - KEEP_OFF_ROOM).max(0), (z0 - KEEP_OFF_ROOM).max(0));
    let (x1, z1) = (
        (x1 + KEEP_OFF_ROOM).min(w.lot.width as i32),
        (z1 + KEEP_OFF_ROOM).min(w.lot.depth as i32),
    );
    if x1 <= x0 || z1 <= z0 {
        return Vec::new();
    }
    let (cols, rows) = ((x1 - x0) as usize, (z1 - z0) as usize);
    let blocked = w.blocked();
    let lot = &w.lot;
    let xz = |k: usize| (x0 + (k % cols) as i32, z0 + (k / cols) as i32);
    let tile = |k: usize| {
        let (x, z) = xz(k);
        lot.tile_index(x, z)
    };
    // (As `NavGrid::tile_free` has it on the ground, but for the junctions' tiles.)
    let wall: Vec<bool> = (0..cols * rows)
        .map(|k| {
            let (x, z) = xz(k);
            let i = lot.tile_index(x, z);
            !lot.allocated(x, z)
                || lot.diag_blocks(x, z)
                || taken.contains(&i)
                || if w.travel.junction_tiles.binary_search(&i).is_ok() {
                    w.travel.base_blocked.binary_search(&i).is_ok()
                } else {
                    blocked.get(i).copied().unwrap_or(true)
                }
        })
        .collect();
    let kept: Vec<bool> = (0..cols * rows)
        .map(|k| mine.binary_search(&tile(k)).is_ok())
        .collect();
    let next = |k: usize| {
        let (x, z) = (k % cols, k / cols);
        let (ax, az) = xz(k);
        [
            (x > 0).then(|| k - 1),
            (x + 1 < cols).then(|| k + 1),
            (z > 0).then(|| k - cols),
            (z + 1 < rows).then(|| k + cols),
        ]
        .into_iter()
        .flatten()
        .filter(move |&m| {
            let (bx, bz) = xz(m);
            lot.edge_walkable(ax, az, bx, bz)
        })
    };
    // (Each tile's piece of ground, numbered from 1; 0 shut.)
    let pieces = |shut: &dyn Fn(usize) -> bool| {
        let mut piece = vec![0u32; cols * rows];
        let mut n = 0;
        let mut todo = Vec::new();
        for start in 0..piece.len() {
            if piece[start] != 0 || shut(start) {
                continue;
            }
            n += 1;
            piece[start] = n;
            todo.push(start);
            while let Some(k) = todo.pop() {
                for m in next(k) {
                    if piece[m] == 0 && !shut(m) {
                        piece[m] = n;
                        todo.push(m);
                    }
                }
            }
        }
        (piece, n as usize)
    };
    let (before, nb) = pieces(&|k| wall[k]);
    let (after, na) = pieces(&|k| wall[k] || kept[k]);
    let edge = |k: usize| {
        let (x, z) = (k % cols, k / cols);
        x == 0 || z == 0 || x + 1 == cols || z + 1 == rows
    };
    // (Of each piece before, the pieces after on the edge, and those that aren't.)
    let mut out = vec![false; na + 1];
    let mut joined = vec![false; nb + 1];
    for k in 0..cols * rows {
        if edge(k) && after[k] != 0 {
            out[after[k] as usize] = true;
            joined[before[k] as usize] = true;
        }
    }
    let mut cut: Vec<usize> = (0..cols * rows)
        .filter(|&k| after[k] != 0 && !out[after[k] as usize] && joined[before[k] as usize])
        .collect();
    // (A gate on the junction's tiles: a piece of its own.)
    cut.extend(gates.iter().filter_map(|&(x, z)| {
        ((x0..x1).contains(&x) && (z0..z1).contains(&z))
            .then(|| (z - z0) as usize * cols + (x - x0) as usize)
            .filter(|&k| kept[k])
    }));
    let mut open = Vec::new();
    let mut from = vec![usize::MAX; cols * rows];
    while let Some(&start) = cut.first() {
        // (The piece, all of it, and the shortest way over the junction's tiles to the edge's
        // ground.)
        let piece = after[start];
        let seeds: Vec<usize> = if piece == 0 {
            vec![start]
        } else {
            cut.iter().copied().filter(|&k| after[k] == piece).collect()
        };
        cut.retain(|k| !seeds.contains(k));
        from.iter_mut().for_each(|f| *f = usize::MAX);
        let mut todo = std::collections::VecDeque::new();
        for &k in &seeds {
            from[k] = k;
            todo.push_back(k);
        }
        let mut found = None;
        while let Some(k) = todo.pop_front() {
            if after[k] != 0 && out[after[k] as usize] {
                found = Some(k);
                break;
            }
            for m in next(k) {
                if from[m] == usize::MAX && !wall[m] && (kept[m] || after[m] != 0) {
                    from[m] = k;
                    todo.push_back(m);
                }
            }
        }
        let Some(mut k) = found else { continue };
        while from[k] != k {
            if kept[k] {
                open.push(tile(k));
            }
            k = from[k];
        }
        if kept[k] {
            open.push(tile(k));
        }
        // (Joined now: the piece is the edge's.)
        for &k in &seeds {
            if after[k] != 0 {
                out[after[k] as usize] = true;
            }
        }
    }
    open.sort_unstable();
    open.dedup();
    open
}

/// Per road of `g`, whether it's a roundabout's ring: the town file's road (`meta.roads`) has an
/// island in the middle.
fn rings(g: &Graph, meta: &serde_json::Value) -> Vec<bool> {
    let roads = meta.get("roads").and_then(|r| r.as_array());
    g.roads
        .iter()
        .map(|r| {
            roads
                .and_then(|list| list.get(r.source as usize))
                .is_some_and(|m| m.get("island").is_some_and(|i| !i.is_null()))
        })
        .collect()
}

/// The junctions' traffic lights (`Travel::signals`: every junction where `SIGNAL_ARMS` roads or
/// more meet), and every car's claims worked out again with them (`Claim::gate`). Once the cars
/// are made or loaded. Every head has a row in the snapshot (`snapshot::MAX_LIGHTS`, made room
/// for the biggest generated worlds' and then some); past that, a junction is still lit, its
/// heads only not drawn, and a debug build says so.
pub(crate) fn settle(w: &mut World) {
    let g = &w.travel.graph;
    let ring = rings(g, &w.meta);
    let signals: Vec<Signal> = (0..g.nodes.len())
        .filter_map(|n| signal_at(g, n, &ring))
        .collect();
    let heads: usize = signals.iter().map(|s| s.arms.len()).sum();
    debug_assert!(
        heads <= crate::snapshot::MAX_LIGHTS,
        "{heads} signal heads, past the snapshot's {} rows",
        crate::snapshot::MAX_LIGHTS
    );
    w.travel.crossings = Crossings::new(&signals);
    keep_off(w, &signals);
    // (Each lit junction's box taken out to the far side of its crosswalks: nobody goes onto one
    // without room to clear it beyond, and a way between two lit junctions too near each other
    // for a body to stand between them is one group, through both lights: `gate`.)
    let mut zones = boxes(&w.travel.graph);
    for sig in &signals {
        for a in sig.arms.iter().filter(|a| a.crosswalk) {
            let mut d = BOX_LEAST;
            while d < a.walk_to {
                d = (d + ARM_STEP).min(a.walk_to);
                let q = [
                    sig.at[0] + a.way[0] * (d - ARM_REACH),
                    sig.at[1] + a.way[1] * (d - ARM_REACH),
                ];
                zones.push((q, ARM_REACH));
            }
        }
    }
    w.travel.graph.boxes = zones;
    w.travel.signals = signals;
    // (Green on the first phase before the first tick changes them: `change_lights`.)
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

/// The lit junctions' crosswalks by where they are (`Travel::crossings`, `kerb_stop`): a grid of
/// `CROSS_CELL` m cells from the town's corner, each with the junctions whose crosswalks reach
/// into it.
#[derive(Debug, Clone, Default)]
pub struct Crossings {
    cols: usize,
    rows: usize,
    cells: Vec<Vec<u16>>,
}

/// `Crossings`' cells (m), and how far apart a step is looked along for crosswalks
/// (`kerb_stop`, m).
const CROSS_CELL: f32 = 16.0;
const KERB_STEP: f32 = 0.25;

impl Crossings {
    fn new(signals: &[Signal]) -> Self {
        let lit = || signals.iter().filter(|s| s.walk_clear > 0);
        let far = lit().fold([0.0f32; 2], |m, s| {
            let r = s.crossing_reach();
            [m[0].max(s.at[0] + r), m[1].max(s.at[1] + r)]
        });
        let cell = |v: f32| (v.max(0.0) / CROSS_CELL) as usize;
        let (cols, rows) = (cell(far[0]) + 1, cell(far[1]) + 1);
        if lit().next().is_none() {
            return Self::default();
        }
        let mut cells = vec![Vec::new(); cols * rows];
        for (i, s) in signals.iter().enumerate() {
            if s.walk_clear == 0 {
                continue;
            }
            let r = s.crossing_reach();
            for z in cell(s.at[1] - r)..=cell(s.at[1] + r).min(rows - 1) {
                for x in cell(s.at[0] - r)..=cell(s.at[0] + r).min(cols - 1) {
                    cells[z * cols + x].push(i as u16);
                }
            }
        }
        Self { cols, rows, cells }
    }
}

/// Whether lit junction `s`'s crosswalks are open to those on foot: its walk phase, while they may
/// step out (`WALK_OPEN`).
fn walk_open(signals: &[Signal], lights: &[Light], s: usize) -> bool {
    signals[s].walk_phase().is_some_and(|walk| {
        lights
            .get(s)
            .is_some_and(|l| l.phase == walk && l.stage == Stage::Green)
    })
}

/// Where someone on foot (riding, they go on with the traffic) stepping from `from` to `to` stops
/// at a lit junction's crosswalks (`Signal::crossing`: its box too, between the kerbs, and its arms
/// out to the far side of their crosswalks) while they're closed to them, out of its walk phase: the junction (of those
/// whose crosswalks are there, the nearest), and how far along the step they come (0 to 1), at the
/// kerb. None: on (out of reach of any; one whose crosswalks are there in its walk phase; or on a
/// junction's crosswalks already, on across, out of it and over any next to it: two junctions a
/// few metres apart, their walks apart, kept someone waiting on one for the other's). One crossing
/// a road away from the junctions goes as ever.
pub(crate) fn kerb_stop(
    travel: &crate::travel::Travel,
    lights: &[Light],
    from: [f32; 2],
    to: [f32; 2],
) -> Option<(u16, f32)> {
    let c = &travel.crossings;
    if c.cells.is_empty() {
        return None;
    }
    let cell = |v: f32, n: usize| ((v.max(0.0) / CROSS_CELL) as usize).min(n - 1);
    let (x0, x1) = (
        cell(from[0].min(to[0]), c.cols),
        cell(from[0].max(to[0]), c.cols),
    );
    let (z0, z1) = (
        cell(from[1].min(to[1]), c.rows),
        cell(from[1].max(to[1]), c.rows),
    );
    // (The junctions near the step, once each.)
    let mut near: [u16; 8] = [0; 8];
    let mut count = 0;
    for z in z0..=z1 {
        for x in x0..=x1 {
            for &s in &c.cells[z * c.cols + x] {
                if !near[..count].contains(&s) && count < near.len() {
                    near[count] = s;
                    count += 1;
                }
            }
        }
    }
    let signals = &travel.signals;
    let near = &near[..count];
    if near.is_empty() || near.iter().any(|&s| signals[usize::from(s)].crossing(from)) {
        return None;
    }
    let n = (dist(from, to) / KERB_STEP).ceil().max(1.0) as usize;
    for k in 1..=n {
        let f = k as f32 / n as f32;
        let p = [
            from[0] + (to[0] - from[0]) * f,
            from[1] + (to[1] - from[1]) * f,
        ];
        let mut here = near
            .iter()
            .copied()
            .filter(|&s| signals[usize::from(s)].crossing(p))
            .peekable();
        if here.peek().is_none() {
            continue;
        }
        let mut stop: Option<(u16, f32)> = None;
        for s in here {
            if walk_open(signals, lights, usize::from(s)) {
                return None;
            }
            let d = dist(p, signals[usize::from(s)].at);
            if stop.is_none_or(|(_, e)| d < e) {
                stop = Some((s, d));
            }
        }
        return stop.map(|(s, _)| (s, (k - 1) as f32 / n as f32));
    }
    None
}

/// The tick's claims, before the cars go on (`Traffic::begin`).
pub(crate) fn begin(w: &mut World) {
    // (Those on foot waiting at a lit junction's crosswalks, `kerb_stop`: those at full detail this
    // tick or last, the others on their last turn, `lod.rs`.)
    let mut walkers = std::mem::take(&mut w.traffic.walkers);
    walkers.clear();
    if !w.travel.signals.is_empty() {
        let tick = w.tick;
        walkers.extend(w.sims.iter().filter_map(|s| {
            s.kerb_wait
                .filter(|&(_, t)| t + crate::lod::FAR_TICKS >= tick)
                .map(|(j, _)| j)
        }));
        walkers.sort_unstable();
        walkers.dedup();
    }
    w.traffic.walkers = walkers;
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

    /// A lit junction at `at` with arms out along `ways`, each of phase `phases[k]`.
    fn lit(at: [f32; 2], ways: &[[f32; 2]], phases: &[u8]) -> Signal {
        Signal {
            at,
            arms: ways
                .iter()
                .zip(phases)
                .map(|(&way, &phase)| SignalArm {
                    way,
                    pole: at,
                    yaw: 0.0,
                    phase,
                    road: 0,
                    kerb: 2.15,
                    walk_from: 6.0,
                    walk_to: 9.0,
                    stop: 9.5,
                    crosswalk: true,
                    span: [
                        [at[0] + way[0] * 6.0, at[1] + way[1] * 6.0],
                        [
                            at[0] + way[0] * (9.5 + KEEP_OFF),
                            at[1] + way[1] * (9.5 + KEEP_OFF),
                        ],
                    ],
                })
                .collect(),
            phases: phases.iter().max().map_or(0, |p| p + 1),
            roundabout: false,
            walk_clear: crate::secs(4.3),
        }
    }

    #[test]
    fn a_crosswalk_and_the_road_kept_off_by_it_follow_its_road_not_the_way_out_from_the_middle() {
        // A junction's east arm, its road leaving the middle 20° off the way it goes on (a road
        // bending away, a middle merged off it): its crosswalk and the road kept off by its stop
        // line are where its road is, not along the way out from the middle (by then metres off
        // it, over the pavement, where it shut a house's gate in).
        let mut sig = lit(
            [0.0, 0.0],
            &[[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0]],
            &[0, 1, 0],
        );
        let (s, c) = 20f32.to_radians().sin_cos();
        sig.arms[0].way = [c, s];
        sig.arms[0].span = [[6.0, 0.0], [9.5 + KEEP_OFF, 0.0]];
        let kerb = sig.arms[0].kerb;
        assert!(sig.crossing([7.5, -1.0]), "on its zebra");
        assert!(!sig.crossing([7.5, kerb + 0.5]), "on the pavement by it");
        assert!(sig.off_limits([11.0, 0.0]), "on the road by the stop line");
        assert!(!sig.off_limits([11.0, kerb + 1.0]), "on the pavement by it");
        assert!(sig.off_limits([2.0, 0.0]), "in the box");
    }

    #[test]
    fn a_group_goes_by_the_lights_of_the_junction_it_drives_through_not_the_first_near() {
        // Two lit junctions near each other: the first listed 12 m off the way, the second on it.
        // A group coming down through the second from the north takes its lights, on the phase
        // of the arm it comes in by (its north arm's, across from none: a phase of its own).
        let near = lit(
            [8.0, -6.0],
            &[[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0]],
            &[0, 0, 1],
        );
        let through = lit(
            [20.0, 0.0],
            &[[-1.0, 0.0], [1.0, 0.0], [0.0, -1.0]],
            &[0, 0, 1],
        );
        let mut claims = vec![claim(&[
            [20.0, -12.0],
            [20.0, -6.0],
            [20.0, 0.0],
            [20.0, 6.0],
            [20.0, 12.0],
        ])];
        gate(&mut claims, &[near, through], &Trace::new(), 4.6);
        assert_eq!(claims[0].gate, Some((1, 1)));
    }

    #[test]
    fn a_group_through_lights_begins_with_the_nose_at_its_arms_stop_line() {
        // Coming down the north arm (its stop line 9.5 m out) of a junction at the origin, its
        // body first touching the box (6 m round) with its middle about 8 m out: it waits further
        // back, its nose at the stop line, its middle 11.8 m out.
        let sig = lit(
            [0.0, 0.0],
            &[[0.0, -1.0], [0.0, 1.0], [1.0, 0.0]],
            &[0, 0, 1],
        );
        let trace: Trace = (0..=60)
            .map(|k| {
                let z = -30.0 + k as f32;
                (k as f32, [-1.05, z], 0.0, false, k as f32)
            })
            .collect();
        let mut claims = claims(&trace, &[], 4.6, 1.9, &[([0.0, 0.0], 6.0)], &[]);
        let before = claims[0].from;
        gate(&mut claims, std::slice::from_ref(&sig), &trace, 4.6);
        assert_eq!(claims[0].gate, Some((0, 0)));
        let middle = |t: f32| trace[t as usize].1[1];
        assert!(
            middle(before) >= -9.0,
            "it touched the box at {}",
            middle(before)
        );
        let nose = middle(claims[0].from) + 2.3;
        assert!((-10.5..=-9.5).contains(&nose), "its nose at {nose}");
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
    fn two_at_the_lights_each_in_the_others_way_wait_for_each_other_on_red_and_green() {
        // (Highholm's: a car and a taxi at their stop lines on two phases of a lit junction, each
        // standing in the other's way through it. On its green each waited for the other's body;
        // the other, at its red, named nobody: no ring for the waits to see, and they stood by
        // turns for good.) At red too each names whom it would wait for: the ring is seen.
        let (a, b) = (1 << 63 | 5, 1 << 63 | 22);
        let mut c_a = claim(&[
            [-6.0, 0.0],
            [-2.0, 0.0],
            [2.0, 0.0],
            [6.0, 0.0],
            [10.0, 0.0],
        ]);
        let mut c_b = claim(&[
            [6.0, 0.0],
            [2.0, 0.0],
            [-2.0, 0.0],
            [-6.0, 0.0],
            [-10.0, 0.0],
        ]);
        c_a.gate = Some((0, 0));
        c_b.gate = Some((0, 1));
        let sig = lit(
            [0.0, 0.0],
            &[[-1.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            &[0, 1, 1],
        );
        let movers = || {
            vec![
                standing(a, [-9.0, 0.0], std::f32::consts::FRAC_PI_2),
                standing(b, [9.0, 0.0], -std::f32::consts::FRAC_PI_2),
            ]
        };
        let places = [(a, 0, 9.5), (b, 0, 9.5)];
        let mut traffic = Traffic::default();
        let mut both = false;
        for tick in 1..CYCLE * 3 {
            traffic.may_go(tick, a, 0, std::slice::from_ref(&c_a), 9.5, 10.5);
            traffic.may_go(tick, b, 0, std::slice::from_ref(&c_b), 9.5, 10.5);
            traffic.begin(tick + 1, movers(), &places, std::slice::from_ref(&sig));
            assert!(
                !traffic.holding(a) && !traffic.holding(b),
                "one went into the other"
            );
            both |= traffic.waits_for(a).contains(&b) && traffic.waits_for(b).contains(&a);
        }
        assert!(
            both,
            "they never both waited for each other: a ring nobody sees"
        );
    }

    #[test]
    fn two_through_two_close_lit_junctions_each_way_both_get_their_greens() {
        // (Hollyglade's: two lit junctions too near to wait between. One waits at the first's red
        // to go through it on phase 1 and the second on phase 0; another the other way round.
        // Each light held its phase 0 for the one waiting through it from the other, and the two
        // went round in step for good: neither ever had both its greens.) One of the two leads,
        // the other follows it: each gets through on both its greens.
        let (x, y) = (1 << 63 | 3, 1 << 63 | 29);
        let mut c_x = claim(&[[-6.0, 50.0], [-2.0, 50.0], [2.0, 50.0], [6.0, 50.0]]);
        let mut c_y = claim(&[[-6.0, -50.0], [-2.0, -50.0], [2.0, -50.0], [6.0, -50.0]]);
        c_x.gate = Some((0, 1));
        c_x.gate2 = Some((1, 0));
        c_y.gate = Some((1, 1));
        c_y.gate2 = Some((0, 0));
        let ways = [[-1.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let sigs = [
            lit([0.0, 0.0], &ways, &[0, 0, 1]),
            lit([20.0, 0.0], &ways, &[0, 0, 1]),
        ];
        let movers = || {
            vec![
                standing(x, [-9.0, 50.0], std::f32::consts::FRAC_PI_2),
                standing(y, [-9.0, -50.0], std::f32::consts::FRAC_PI_2),
            ]
        };
        let places = [(x, 0, 9.5), (y, 0, 9.5)];
        let mut traffic = Traffic::default();
        let (mut went_x, mut went_y) = (None, None);
        for tick in 1..CYCLE * 6 {
            if went_x.is_none() {
                traffic.may_go(tick, x, 0, std::slice::from_ref(&c_x), 9.5, 10.5);
            }
            if went_y.is_none() {
                traffic.may_go(tick, y, 0, std::slice::from_ref(&c_y), 9.5, 10.5);
            }
            traffic.begin(tick + 1, movers(), &places, &sigs);
            if went_x.is_none() && traffic.holding(x) {
                went_x = Some(tick);
            }
            if went_y.is_none() && traffic.holding(y) {
                went_y = Some(tick);
            }
        }
        assert!(
            went_x.is_some_and(|t| t < 2 * CYCLE) && went_y.is_some_and(|t| t < 2 * CYCLE),
            "through both lights: {went_x:?}, {went_y:?}"
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
