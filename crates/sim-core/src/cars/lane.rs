//! Lanes: bending, rounding corners and gentle bends.
//!
//! **Which side is right.** The town's x runs east and its z north, a yaw turning clockwise seen
//! from above (`Car::yaw`: forward is (sin, cos)): a left-handed frame, as the renderer draws it
//! (`web/src/render/pc/space.ts`). So the right of going (x, z) is (z, -x) (`right_of`): going
//! east, the south. Traffic keeps to the right as the game shows it (German rules): every right
//! and left of the traffic (`shifted`, `offset_right_by`, `bend_out`, `turned`, the bays', the
//! kerbs', the taxis', the walkers', `Graph::inside`) is this one. (The town map draws z down the
//! page, a mirror of the game's view: there, it looks the other way round.)

use super::*;

/// The right of going `d` (a way along the ground, x z): a quarter turn clockwise seen from
/// above. Going +x (east), -z (south). See the top of this file.
pub(crate) fn right_of(d: [f32; 2]) -> [f32; 2] {
    [d[1], -d[0]]
}

/// How far past `lane`'s own line at its point `i` the right side of a body `len` m long standing
/// along it there reaches (m), on the outside of a bend to the left: the sagitta of the chord its
/// length spans, which `pull_room` (worked out as if the road ran straight) leaves no room for.
/// None on a straight, or on a bend to the right, its right side on the inside. (A taxi pulled
/// over its full room at the kerb of a lane bending left stood with a corner over the kerbside
/// furniture: Primrosebrook.)
pub(crate) fn bend_out(lane: &[[f32; 2]], i: usize, len: f32) -> f32 {
    use crate::travel::dist;
    let Some(&p) = lane.get(i) else {
        return 0.0;
    };
    // The point `len / 2` m along the lane from `i`, back (`step` -1) or on (+1), or its end.
    let reach = |step: isize| -> [f32; 2] {
        let (mut at, mut left) = (i as isize, len / 2.0);
        loop {
            let next = at + step;
            if next < 0 || next as usize >= lane.len() {
                return lane[at as usize];
            }
            let (a, b) = (lane[at as usize], lane[next as usize]);
            let d = dist(a, b);
            if d >= left {
                let k = left / d.max(1e-6);
                return [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k];
            }
            left -= d;
            at = next;
        }
    };
    let (a, b) = (reach(-1), reach(1));
    let d = sub2(b, a);
    let l = d[0].hypot(d[1]);
    if l < 1e-3 {
        return 0.0;
    }
    // (Right of the way it runs, as `shifted`: `right_of`.)
    (dot2(sub2(p, a), right_of(d)) / l).max(0.0)
}

/// `p`, facing `yaw`, `shift` m to its right.
pub(crate) fn shifted(p: [f32; 2], yaw: f32, shift: f32) -> [f32; 2] {
    // (Forward is (sin, cos); its right (cos, -sin): `right_of`.)
    [p[0] + yaw.cos() * shift, p[1] - yaw.sin() * shift]
}

/// `pts` moved `offs[i]` m to the right (looking the way they run), each its own.
pub(crate) fn offset_right_by(pts: &[[f32; 2]], offs: &[f32]) -> Vec<[f32; 2]> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let (a, b) = (pts[i.saturating_sub(1)], pts[(i + 1).min(n - 1)]);
            let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
            let d = dx.hypot(dz).max(1e-6);
            let r = right_of([dx / d, dz / d]);
            [pts[i][0] + r[0] * offs[i], pts[i][1] + r[1] * offs[i]]
        })
        .collect()
}

/// `lane` (a lane's points, `thin_with`) with its corners rounded, its ends where they were: where
/// it turns sharper than `SHARP` at a point or at a few close together (a junction, a street corner,
/// where one road's end meets another's: the two lanes' lines cross there, or one goes on past the
/// other's and the way doubles back on itself a little), an arc into the line out from the line in
/// (`sharp_corners`); then at each other bend, an arc from the middle of the step before to the
/// middle of the next (`gentle_bends`).
pub(crate) fn round_corners(lane: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let (pts, fixed) = sharp_corners(lane);
    let mut out = gentle_bends(&pts, &fixed);
    out.dedup_by(|b, a| crate::travel::dist(*a, *b) < 1e-3);
    out
}

pub(super) fn sub2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

pub(super) fn dot2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

pub(super) fn cross2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

/// The way from `a` to `b`, a unit long.
pub(super) fn unit2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let l = crate::travel::dist(a, b).max(1e-6);
    [(b[0] - a[0]) / l, (b[1] - a[1]) / l]
}

/// How far going `b` is turned from going `a` (radians, + to the right, clockwise seen from above:
/// `right_of`).
pub(super) fn turned(a: [f32; 2], b: [f32; 2]) -> f32 {
    (-cross2(a, b)).atan2(dot2(a, b))
}

/// How far `p` is off the line through `c` going `d`.
fn off_line(p: [f32; 2], c: [f32; 2], d: [f32; 2]) -> f32 {
    cross2(d, sub2(p, c)).abs()
}

/// An arc of radius `r` from `from` going `din`, turning `theta` (`turned`): a point every
/// `ARC_STEP` of it, its ends too, onto `out`.
fn arc_onto(out: &mut Vec<[f32; 2]>, from: [f32; 2], din: [f32; 2], theta: f32, r: f32) {
    // (About its middle, towards the inside of the turn square to the way in: right of it on a
    // turn to the right.)
    let rt = right_of(din);
    let inside = if theta > 0.0 { rt } else { [-rt[0], -rt[1]] };
    let centre = [from[0] + inside[0] * r, from[1] + inside[1] * r];
    let v = sub2(from, centre);
    let k = (theta.abs() / ARC_STEP).ceil().max(1.0) as usize;
    for s in 0..=k {
        // (Turned from x towards z, the other way round from `turned`'s.)
        let (sn, cs) = (-theta * s as f32 / k as f32).sin_cos();
        let p = [
            centre[0] + v[0] * cs - v[1] * sn,
            centre[1] + v[0] * sn + v[1] * cs,
        ];
        if out.last().is_none_or(|q| crate::travel::dist(*q, p) > 1e-3) {
            out.push(p);
        }
    }
}

/// A curve from `from` to `to` round the corner `c` (a quadratic Bézier: along the lines from
/// `from` to `c` and from `c` to `to` at its ends), a point every `ARC_STEP` it turns, onto `out`.
fn curve_onto(out: &mut Vec<[f32; 2]>, from: [f32; 2], c: [f32; 2], to: [f32; 2]) {
    const STEPS: usize = 128;
    let (a, b) = (sub2(c, from), sub2(to, c));
    let way = |s: f32| [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s];
    let at = |s: f32| {
        let (u, v, w) = ((1.0 - s) * (1.0 - s), 2.0 * s * (1.0 - s), s * s);
        [
            from[0] * u + c[0] * v + to[0] * w,
            from[1] * u + c[1] * v + to[1] * w,
        ]
    };
    if out
        .last()
        .is_none_or(|q| crate::travel::dist(*q, from) > 1e-3)
    {
        out.push(from);
    }
    let mut was = way(0.0);
    for k in 1..=STEPS {
        let s = k as f32 / STEPS as f32;
        let d = way(s);
        if k == STEPS || turned(was, d).abs() >= ARC_STEP {
            out.push(at(s));
            was = d;
        }
    }
}

/// `round_corners`' sharp corners: its points, and which stay as they are (`gentle_bends`): its
/// ends and the arcs'.
fn sharp_corners(lane: &[[f32; 2]]) -> (Vec<[f32; 2]>, Vec<bool>) {
    use crate::travel::dist;
    let n = lane.len();
    if n < 3 {
        return (lane.to_vec(), vec![true; n]);
    }
    let turn_at = |i: usize| turned(unit2(lane[i - 1], lane[i]), unit2(lane[i], lane[i + 1]));
    let mut out: Vec<[f32; 2]> = vec![lane[0]];
    let mut fixed = vec![true];
    // (The points laid so far that stay: the start, and every arc.)
    let mut locked = 1;
    let mut i = 1;
    while i + 1 < n {
        if turn_at(i).abs() < SHARP {
            out.push(lane[i]);
            fixed.push(false);
            i += 1;
            continue;
        }
        // The corner: this point and the sharp ones close on after it.
        let mut j = i;
        while j + 2 < n && dist(lane[j], lane[j + 1]) < CORNER_SPAN && turn_at(j + 1).abs() >= SHARP
        {
            j += 1;
        }
        let din = unit2(lane[i - 1], lane[i]);
        let dout = unit2(lane[j], lane[j + 1]);
        let theta = turned(din, dout);
        // (Its way in a step back against the way it came, where the way goes a little past a
        // road's end and back: as it is. Rounded from that step, the curve began on the far side
        // of the corner, and the road before it, ahead of the curve's start that way, was taken
        // out with it: a taxi drove 130 m straight across the grass, Applewold.)
        if i >= 2 && dot2(unit2(lane[i - 2], lane[i - 1]), din) < 0.0 {
            out.extend_from_slice(&lane[i..=j]);
            fixed.resize(out.len(), false);
            i = j + 1;
            continue;
        }
        let mut keep = |out: &mut Vec<[f32; 2]>| {
            out.extend_from_slice(&lane[i..=j]);
            fixed.resize(out.len(), false);
        };
        // (A jog back to the line it was on: left out; to another line, or turning back on
        // itself, as it is.)
        if theta.abs() < 0.05 {
            if j == i || off_line(lane[j + 1], lane[i - 1], din) > 0.3 {
                keep(&mut out);
            }
            i = j + 1;
            continue;
        }
        // Where its lines in and out cross.
        let c = if j == i {
            lane[i]
        } else {
            let u = cross2(sub2(lane[j], lane[i]), dout) / cross2(din, dout);
            [lane[i][0] + din[0] * u, lane[i][1] + din[1] * u]
        };
        if theta.abs() > std::f32::consts::PI - 0.17
            || dist(c, lane[i]) > 2.0 * CORNER_SPAN
            || dist(c, lane[j]) > 2.0 * CORNER_SPAN
        {
            keep(&mut out);
            i = j + 1;
            continue;
        }
        // How far back along the line in it may begin (back over the points on that line, not
        // into an arc), and how far on along the line out it may end (over the points on that line
        // to where the lane turns again, half way there).
        let mut m = out.len() - 1;
        while m >= locked && off_line(out[m - 1], c, din) < 0.2 {
            m -= 1;
        }
        // (Where it begins and ends, the steps from and to the points either side that stay
        // turn little: those a little off its lines, a lead in or out long enough.)
        let back = dot2(sub2(c, out[m]), din) - LEAD * off_line(out[m], c, din);
        let mut q = j + 1;
        while q + 1 < n && turn_at(q).abs() < 0.02 && off_line(lane[q + 1], c, dout) < 0.2 {
            q += 1;
        }
        let ahead = dot2(sub2(lane[q], c), dout);
        let share = if q + 1 < n { ahead / 2.0 } else { ahead };
        let fwd = share.min(ahead - LEAD * off_line(lane[q], c, dout));
        // (As far from the corner either way as an arc as wide as `most` takes, or as there's
        // room for: short of it on one side, a curve wider on the other.)
        let tan = (theta.abs() / 2.0).tan();
        let most = if theta > 0.0 { RIGHT_TURN } else { LEFT_TURN };
        let (into, outof) = ((most * tan).min(back - 0.01), (most * tan).min(fwd - 0.01));
        if into.min(outof) < 0.02 {
            keep(&mut out);
            i = j + 1;
            continue;
        }
        let from = [c[0] - din[0] * into, c[1] - din[1] * into];
        let to = [c[0] + dout[0] * outof, c[1] + dout[1] * outof];
        // (Only those about the corner, never the road before it.)
        let reach = into + 3.0 * CORNER_SPAN;
        while out.len() > locked
            && dot2(sub2(*out.last().expect("one"), from), din) > -0.01
            && dist(*out.last().expect("one"), c) <= reach
        {
            out.pop();
            fixed.pop();
        }
        if (into - outof).abs() < 1e-3 {
            arc_onto(&mut out, from, din, theta, into / tan);
        } else {
            curve_onto(&mut out, from, c, to);
        }
        fixed.resize(out.len(), true);
        locked = out.len();
        // On from the first point past its end.
        i = j + 1;
        while i + 1 < n && dot2(sub2(lane[i], to), dout) < 0.01 {
            i += 1;
        }
    }
    let last = lane[n - 1];
    if out.last().is_none_or(|p| dist(*p, last) > 1e-3) {
        out.push(last);
        fixed.push(true);
    }
    if let Some(f) = fixed.last_mut() {
        *f = true;
    }
    (out, fixed)
}

/// `round_corners`' gentler bends: at each of `pts`' points but the `fixed` ones, an arc as wide as
/// `BEND` at most from the middle of the step before (or the point before, fixed or straight on) to
/// the middle of the next (or the next point, likewise).
fn gentle_bends(pts: &[[f32; 2]], fixed: &[bool]) -> Vec<[f32; 2]> {
    use crate::travel::dist;
    let n = pts.len();
    if n < 3 {
        return pts.to_vec();
    }
    let turn_at = |i: usize| turned(unit2(pts[i - 1], pts[i]), unit2(pts[i], pts[i + 1]));
    let bends = |i: usize| {
        !fixed[i] && i + 1 < n && {
            let t = turn_at(i).abs();
            t > 1e-3 && t < std::f32::consts::PI - 0.17
        }
    };
    let mut out: Vec<[f32; 2]> = vec![pts[0]];
    for i in 1..n - 1 {
        if !bends(i) {
            out.push(pts[i]);
            continue;
        }
        let din = unit2(pts[i - 1], pts[i]);
        let dout = unit2(pts[i], pts[i + 1]);
        let theta = turned(din, dout);
        let back = dist(*out.last().expect("one"), pts[i]);
        let step = dist(pts[i], pts[i + 1]);
        let fwd = if bends(i + 1) { step / 2.0 } else { step };
        let tan = (theta.abs() / 2.0).tan();
        let taken = (BEND * tan).min(back).min(fwd);
        if taken < 0.01 {
            out.push(pts[i]);
            continue;
        }
        let from = [pts[i][0] - din[0] * taken, pts[i][1] - din[1] * taken];
        arc_onto(&mut out, from, din, theta, taken / tan);
    }
    out.push(pts[n - 1]);
    out
}
