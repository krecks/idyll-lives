//! Where a moving thing will be, sampled along its route.

use super::*;

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

/// A lit junction (`Travel::signals`) and the phase of the arm a way comes in by (`Claim::gate`).
pub(crate) type Gate = (u16, u8);

/// A stretch of a route a vehicle holds before driving into it (`from`, ticks along it) till it's
/// out of its last zone (`to`), sampled on past that over its room on the far side.
#[derive(Debug, Clone, Default)]
pub(crate) struct Claim {
    pub from: f32,
    pub to: f32,
    /// Its group: parts of a route its body comes to before it's out of the one before (`claims`).
    pub group: u32,
    pub(crate) samples: Vec<Sample>,
    /// The box round all its samples' bodies (x0, z0, x1, z1).
    pub(crate) bounds: [f32; 4],
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
    pub(crate) fn new(from: f32, to: f32, samples: Vec<Sample>) -> Claim {
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

    /// The lights its group goes through (`gate`, `gate2`) that one `at` ticks along its route
    /// still comes to: not one whose stop line it's past already, in the box (`stop_at`,
    /// `stop2`). (A taxi stood for its rider in a lit junction's box, out past the crosswalk,
    /// letting go of its group but the part it stood in, `Traffic::stand`: asking for the rest
    /// again, it waited in the box for that light's green, held up those waiting for the box to
    /// clear at the others' greens, Elderbrook.)
    pub(crate) fn lights(&self, at: f32) -> (Option<Gate>, Option<Gate>) {
        let past = |line: f32| line.is_finite() && at > line + 1e-3;
        (
            self.gate.filter(|_| !past(self.stop_at)),
            self.gate2.filter(|_| !past(self.stop2)),
        )
    }

    /// What's left of it for one `t` ticks along its route: the samples from there on (from the
    /// one before: with the next, its body there now).
    pub(crate) fn rest(&self, t: f32) -> &[Sample] {
        let k = self.samples.partition_point(|s| s.t < t);
        &self.samples[k.saturating_sub(1)..]
    }
}

/// How far from its middle a body reaches at most (m).
pub(crate) fn reach(r: &Rect) -> f32 {
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
