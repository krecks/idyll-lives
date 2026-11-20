//! Getting through a junction: signals, zebra crossings and narrow spots.
//!
//! Split from `traffic.rs` (see `mod.rs`).

use super::*;

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
        // (Not round a roundabout, its junctions lit as ever: its cars wait at the box's edge,
        // with no crosswalks to wait on, and its ring through the next's box. Asked for on two of
        // its junctions' greens at once, cars round one waited for hours, Bridgegate.)
        let ring = |&(i, _): &(usize, usize)| signals[i].roundabout;
        let gate2 = through
            .get(1)
            .map(arm_of)
            .filter(|g| !ring(g) && !gate.as_ref().is_some_and(ring));
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
            let nose = |j: usize| nose_in(trace, j, sig.at, a, len);
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
                // (Back into the same lit junction by another arm, a loop round from it too short
                // to wait on: through it on the green it came in on. Asked for on two of its
                // phases' greens at once, never both, one waited at its stop line for good,
                // Willowbrook.)
                let this = claims[k].gate.filter(|t| gate.is_none_or(|g| g.0 != t.0));
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

/// Where the nose of a body `len` long at pose `j` of `trace` is on arm `a` of the junction at
/// `at` (`gate`, `zebra_stops`): how far out along it, and whether it's coming in along it, between
/// its kerbs (and a metre more), not backing up.
fn nose_in(trace: &Trace, j: usize, at: [f32; 2], a: &SignalArm, len: f32) -> (f32, bool) {
    let (_, p, yaw, back, _) = trace[j];
    let (s, c) = yaw.sin_cos();
    let n = [p[0] + s * len / 2.0, p[1] + c * len / 2.0];
    let d = [n[0] - at[0], n[1] - at[1]];
    let along = d[0] * a.way[0] + d[1] * a.way[1];
    let across = (d[0] * a.way[1] - d[1] * a.way[0]).abs();
    let inward = s * a.way[0] + c * a.way[1] < -0.5;
    (along, !back && inward && across <= a.kerb + 1.0)
}

/// Where a car's route stops for those on foot at a junction giving way (`Travel::give_way`,
/// `zebra_stops`): `at` ticks along it, its nose at the stop line of the arm it comes in by (out
/// past that arm's zebra, where it has one); the junction; and the zebras it crosses there, the
/// arm's it comes in by and the one's it goes out by. It stops there only for someone on foot on
/// the first or stepping onto it, or on the second (`cars::zebra_yield`); else it drives on. At a
/// lit junction (`lit`: the phase of the arm it comes in by), turning into a side road whose
/// crosswalk is crossed on its green (`side_zebra_stops`): only the second, while it's green.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ZebraStop {
    pub at: f32,
    pub junction: u16,
    pub arms: [Option<u8>; 2],
    pub lit: Option<u8>,
}

/// A route's stops at the zebras of the junctions giving way it goes through (`ZebraStop`), for a
/// body `len` long along `trace`, in order along it. A junction it goes through as `gate` has it
/// (within `GATE_THROUGH` of its middle), its arms in and out those it's on where it first and last
/// comes within `GATE_REACH`; one it begins or ends by (a driveway by it) not crossed coming in or
/// going out.
pub(crate) fn zebra_stops(trace: &Trace, junctions: &[Signal], len: f32) -> Vec<ZebraStop> {
    stops_at(trace, junctions, len, |sig, into, out_by, crossed| {
        let zebra = |a: usize, crossed: bool| {
            (crossed && sig.arms[a].crosswalk)
                .then(|| u8::try_from(a).ok())
                .flatten()
        };
        let arms = [zebra(into, crossed[0]), zebra(out_by, crossed[1])];
        (arms != [None, None]).then_some((arms, None))
    })
}

/// A route's stops at lit junctions' side roads' crosswalks (`ZebraStop`, `Signal::opens_with`):
/// where it turns into a side road on the green whose crosswalk is crossed on, it stops at its
/// stop line for anyone on that one (`cars::zebra_yield`), in order along it.
pub(crate) fn side_zebra_stops(trace: &Trace, signals: &[Signal], len: f32) -> Vec<ZebraStop> {
    stops_at(trace, signals, len, |sig, into, out_by, crossed| {
        let phase = sig.arms[into].phase;
        let out = u8::try_from(out_by).ok()?;
        (crossed[1] && out_by != into && sig.opens_with(out_by) == Some(phase))
            .then_some(([None, Some(out)], Some(phase)))
    })
}

/// The stops along `trace` (a body `len` long) at `junctions` (`zebra_stops`): where it goes
/// through one, coming in by one arm and out by another (crossing the zebras on them, or not: it
/// begins or ends by the junction), `pick` says which zebras it stops for (and at a lit junction,
/// the phase it's let through on), its nose at the stop line of the arm it comes in by.
fn stops_at(
    trace: &Trace,
    junctions: &[Signal],
    len: f32,
    pick: impl Fn(&Signal, usize, usize, [bool; 2]) -> Option<([Option<u8>; 2], Option<u8>)>,
) -> Vec<ZebraStop> {
    let mut out = Vec::new();
    let n = trace.len();
    if n == 0 {
        return out;
    }
    let b = trace.iter().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |b, s| {
            [
                b[0].min(s.1[0]),
                b[1].min(s.1[1]),
                b[2].max(s.1[0]),
                b[3].max(s.1[1]),
            ]
        },
    );
    for (i, sig) in junctions.iter().enumerate() {
        let p = sig.at;
        if sig.walk_clear == 0
            || p[0] < b[0] - GATE_REACH
            || p[0] > b[2] + GATE_REACH
            || p[1] < b[1] - GATE_REACH
            || p[1] > b[3] + GATE_REACH
        {
            continue;
        }
        let arm_of = |q: [f32; 2]| {
            let d = [q[0] - p[0], q[1] - p[1]];
            (0..sig.arms.len())
                .max_by(|&x, &y| {
                    let dot = |a: usize| sig.arms[a].way[0] * d[0] + sig.arms[a].way[1] * d[1];
                    dot(x).total_cmp(&dot(y))
                })
                .unwrap_or(0)
        };
        let mut k = 0;
        while k < n {
            if dist(trace[k].1, p) >= GATE_REACH {
                k += 1;
                continue;
            }
            let first = k;
            let mut near = f32::INFINITY;
            while k < n && dist(trace[k].1, p) < GATE_REACH {
                near = near.min(dist(trace[k].1, p));
                k += 1;
            }
            let last = k - 1;
            if near >= GATE_THROUGH {
                continue;
            }
            let (into, out_by) = (arm_of(trace[first].1), arm_of(trace[last].1));
            let Some((arms, lit)) = pick(sig, into, out_by, [first > 0, last + 1 < n]) else {
                continue;
            };
            // (Back along its way from where it first comes near, while it's coming in along the
            // arm with its nose short of the stop line; or on from there while its nose is still
            // out past it, coming in: it first comes near `GATE_REACH` out, its nose some 14 m
            // out, and a car stopped for those on a zebra 4 m short of its stop line, 8 m short of
            // the zebra. Between that pose and the next, where the nose is at the line.)
            let a = &sig.arms[into];
            let mut j = first;
            while j > 0 {
                let (along, on) = nose_in(trace, j, p, a, len);
                if !on || along >= a.stop {
                    break;
                }
                j -= 1;
            }
            if j == first {
                while j < last
                    && let (along, true) = nose_in(trace, j + 1, p, a, len)
                    && along >= a.stop
                {
                    j += 1;
                }
            }
            let mut at = trace[j].0;
            if let ((was, true), Some((next, true))) = (
                nose_in(trace, j, p, a, len),
                (j + 1 < n).then(|| nose_in(trace, j + 1, p, a, len)),
            ) && was >= a.stop
                && next < a.stop
            {
                at += (trace[j + 1].0 - at) * (was - a.stop) / (was - next);
            }
            out.push(ZebraStop {
                at,
                junction: i as u16,
                arms,
                lit,
            });
        }
    }
    out.sort_by(|a, b| a.at.total_cmp(&b.at));
    out
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
