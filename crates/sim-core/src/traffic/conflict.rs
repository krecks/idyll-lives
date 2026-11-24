//! Whether two claims on the same road actually conflict.

use super::*;

/// Who waits for whom (`graph`), and whom each waits for however far round, worked out once for
/// each asked about (`reaches`).
pub(super) struct Reaches {
    pub(super) graph: std::collections::BTreeMap<u64, Vec<u64>>,
    found: std::cell::RefCell<std::collections::HashMap<u64, std::collections::HashSet<u64>>>,
}

impl Reaches {
    pub(super) fn new(graph: std::collections::BTreeMap<u64, Vec<u64>>) -> Reaches {
        Reaches {
            graph,
            found: Default::default(),
        }
    }

    /// Whether `from` waits for `to`, however far round (`reaches`).
    pub(super) fn reaches(&self, from: u64, to: u64) -> bool {
        let mut found = self.found.borrow_mut();
        let round = found.entry(from).or_insert_with(|| {
            let mut seen = std::collections::HashSet::new();
            let mut todo = vec![from];
            while let Some(k) = todo.pop() {
                for &x in self.graph.get(&k).into_iter().flatten() {
                    if seen.insert(x) {
                        todo.push(x);
                    }
                }
            }
            seen
        });
        let out = round.contains(&to);
        debug_assert_eq!(out, reaches(&self.graph, from, to));
        out
    }
}

/// Whether `from` waits for `to`, however far round, in `graph` (who waits for whom).
pub(super) fn reaches(
    graph: &std::collections::BTreeMap<u64, Vec<u64>>,
    from: u64,
    to: u64,
) -> bool {
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
pub(super) fn in_bounds(claim: &Claim, r: &Rect) -> bool {
    let b = claim.bounds;
    let k = reach(r) + CLEAR;
    !(r.at[0] + k < b[0] || r.at[0] - k > b[2] || r.at[1] + k < b[1] || r.at[1] - k > b[3])
}

/// Whether any of samples `a` (of claim `ca`) is in the way of any of `b` (of `cb`: `conflict`).
pub(super) fn overlaps(a: (&Claim, &[Sample]), b: (&Claim, &[Sample])) -> bool {
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
    // (On levels that don't meet, one over the other: `travel::Layer`.)
    if !a.layer.meets(b.layer) {
        return false;
    }
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
