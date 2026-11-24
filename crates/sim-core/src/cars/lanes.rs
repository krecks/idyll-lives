//! Lanes on the road hierarchy (`travel::RoadKind`): which lane a car keeps to on each road, and
//! where it moves from one lane to the next.
//!
//! - **A two-way street, lane or country road** (`classic`): one lane each way, `lane_on` right of
//!   its middle line. A way along only such roads drives exactly as it always did.
//! - **An avenue**: two lanes each way between its median (`AVENUE_MEDIAN`) and its kerbs, where
//!   the renderer paints them (`markings.ts` `laneLayout`). A car keeps to the outer lane, but on
//!   a stretch at whose end it turns left at a junction: there, the inner one.
//! - **A motorway carriageway**: two lanes (`MOTORWAY_LANE`) out from its inside, towards the
//!   other carriageway (`Graph::inside`), its shoulder outside them. Everyone keeps to the outer
//!   lane; the inner one is drawn but not driven yet.
//! - **A ramp, or a one-way street, lane or country road**: one lane, down its middle (where the
//!   renderer paints its arrows). Nobody comes the other way there to keep right of.
//!
//! A car moves to its next lane only where nobody else's lane is in its way unclaimed:
//! - going straight through a junction, across its box (`Graph::boxes`, whose claim covers it);
//! - turning at a junction (or a corner of two roads' ends), as it turns there;
//! - where a ramp leaves its carriageway or merges into it (no box: `Graph::is_merge`), eased over
//!   `TAPER` m of the ramp, never across the carriageway's lanes in a step;
//! - elsewhere (where one road's end meets the next's), eased over `TAPER` m of the road on.
//!
//! `LaneLine` lays a way's lanes out; `lane_off` is a road's lane, for those that need one alone.

use super::*;
use crate::travel::{Graph, NearestPoint, Road, RoadKind, dist};

/// An avenue's median (m: `roadKinds.ts` `ROAD_KIND.avenue.median`).
pub(crate) const AVENUE_MEDIAN: f32 = 2.6;
/// A motorway carriageway's lanes (m: `motorway.ts` `MOTORWAY.inner`, `MOTORWAY.lane`): how far in
/// from its inside edge the inner lane begins, and each lane's width.
pub(crate) const MOTORWAY_INNER: f32 = 0.55;
pub(crate) const MOTORWAY_LANE: f32 = 3.5;
/// How long a change of lane where no junction's box covers it is eased over (m): a ramp's into
/// its carriageway's outer lane, or out of it.
pub(crate) const TAPER: f32 = 30.0;
/// Turning at a junction or a corner, how far past the turn's middle a car is in its new lane (m):
/// the corner is rounded from there (`round_corners`), as every turn always was.
const SWITCH: f32 = 1.0;
/// How far either side of a turn the way it comes in and goes out is taken from (m).
const LOOK: f32 = 6.0;
/// A turn at a junction further left than this (radians) takes the inner lane of an avenue up to
/// it (`TURN_AT`: a turn that slows a car).
const LEFT_AT: f32 = TURN_AT;
/// How far out from the middle of a junction its boxes are looked along a way (m), and in what
/// steps.
const BOX_LOOK: f32 = 20.0;
const BOX_STEP: f32 = 0.5;
/// `Route::lane` on a road where nobody comes the other way in it (one-way: a ramp, a motorway's
/// carriageway, a one-way street): no lane coming the other way for it to reach over.
pub(crate) const NO_ONCOMING: f32 = 50.0;

/// How many lanes a road of `kind` has each way (`roadKinds.ts` `lanesEachWay`).
pub(crate) fn lanes_each_way(kind: RoadKind) -> u8 {
    match kind {
        RoadKind::Avenue | RoadKind::Motorway => 2,
        _ => 1,
    }
}

/// Whether `road` is one cars always drove: two-way, one lane each way (a street, a lane, a
/// country road).
pub(crate) fn classic(road: &Road) -> bool {
    road.oneway.is_both() && !road.kind.fast()
}

/// Whether one can come the other way along `road` in it: two-way, and not a ramp or a motorway.
fn oncoming(road: &Road) -> bool {
    road.oneway.is_both() && !road.kind.oneway_always()
}

/// Lane `idx` (0: the outer, kerbside one; 1: the next in) of network road `road` for a car going
/// along it the way its points run (`along`) or against them: how far right of its middle line,
/// looking the way the car goes (m). An index past its lanes is its innermost.
pub(crate) fn lane_off(g: &Graph, road: u32, idx: u8, along: bool) -> f32 {
    let r = &g.roads[road as usize];
    let inside = g
        .inside
        .get(road as usize)
        .copied()
        .filter(|&v| v != 0)
        .unwrap_or(1);
    lane_of(r, inside, idx, along)
}

/// `lane_off` of `road`, its inside `inside` (`Graph::inside`; a motorway's only).
fn lane_of(road: &Road, inside: i8, idx: u8, along: bool) -> f32 {
    let n = lanes_each_way(road.kind);
    // (Its lanes counted out from the inside: the median's, or the other carriageway's.)
    let k = (n - 1 - idx.min(n - 1)) as f32;
    match road.kind {
        RoadKind::Avenue => {
            let (m, half) = (AVENUE_MEDIAN / 2.0, kerb_of(road.width, road.kind));
            m + (half - m) / n as f32 * (k + 0.5)
        }
        RoadKind::Motorway => {
            // (Its inside the left of the way the car goes, +1; its lanes right of that edge.)
            let inside = if along { inside } else { -inside } as f32;
            inside * (MOTORWAY_INNER + MOTORWAY_LANE * (k + 0.5) - road.width / 2.0)
        }
        _ if !classic(road) => 0.0,
        _ => lane_on(road.width, road.kind),
    }
}

/// Where a car turns onto `road` out of a driveway or a bay, or off it into one (`Bay`): m right
/// of its middle line. `TIGHT_LANE`, as ever, on a road cars always drove (`classic`); else its
/// outer lane (an avenue's), or its middle (a one-way road's).
pub(crate) fn kerb_lane(road: &Road) -> f32 {
    if classic(road) {
        TIGHT_LANE
    } else {
        lane_of(road, 1, 0, true)
    }
}

/// A way's lanes laid out (`LaneLine::new`): its middle line's points, in the order a car drives
/// them, each with how far right of it the car's lane is, and the road it's on.
pub(crate) struct LaneLine {
    /// The way's middle line: its points, and more where its lane changes (where a change is to
    /// begin or end between two of them).
    pub(crate) pts: Vec<[f32; 2]>,
    /// At each point, how far right of the middle line the car's lane is (m).
    pub(crate) offs: Vec<f32>,
    /// At each point, its road and whether the car goes along it the way its points run
    /// (`RoadPath::roads`).
    pub(crate) on: Vec<(u32, bool)>,
    /// Its points' nearest. None where every road along it is one cars always drove (`classic`):
    /// its points the way's own and its lanes `lane_on`'s, all as they always were.
    near: Option<NearestPoint>,
    /// Its points where it turns from one lane into another (`SWITCH`): its lane's corner there
    /// where the lane in meets the lane out (`line`).
    corners: Vec<usize>,
}

impl LaneLine {
    /// The lanes along the way through `pts` (its middle line, in the order a car drives it),
    /// each point on the road `on` says (see the module notes).
    pub(crate) fn new(g: &Graph, pts: Vec<[f32; 2]>, on: Vec<(u32, bool)>) -> LaneLine {
        let n = pts.len();
        let road = |i: usize| &g.roads[on[i].0 as usize];
        if n < 2 || (0..n).all(|i| classic(road(i))) {
            let offs = (0..n)
                .map(|i| lane_on(road(i).width, road(i).kind))
                .collect();
            return LaneLine {
                pts,
                offs,
                on,
                near: None,
                corners: Vec::new(),
            };
        }
        // How far along the way each point is, and the point `x` m along it.
        let mut s = vec![0.0f32; n];
        for i in 1..n {
            s[i] = s[i - 1] + dist(pts[i - 1], pts[i]);
        }
        let at = |x: f32| -> [f32; 2] {
            let x = x.clamp(0.0, s[n - 1]);
            let i = s.partition_point(|&v| v < x).clamp(1, n - 1);
            let (a, b) = (pts[i - 1], pts[i]);
            let k = ((x - s[i - 1]) / (s[i] - s[i - 1]).max(1e-6)).clamp(0.0, 1.0);
            [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k]
        };
        // How far it turns at its point `i` (radians, + to the right: `turned`).
        let turn_at = |i: usize| {
            let (a, p, b) = (at(s[i] - LOOK), pts[i], at(s[i] + LOOK));
            if dist(a, p) < 0.5 || dist(p, b) < 0.5 {
                return 0.0;
            }
            turned(unit2(a, p), unit2(p, b))
        };
        // Its points at a junction's middle.
        let at_junction = |p: [f32; 2]| g.junction_near(p).is_some_and(|j| dist(j, p) < 0.5);
        // Each point's lane: the outer, but on an avenue's stretch at whose end it turns left.
        let mut base = vec![0.0f32; n];
        let mut left_ahead: Option<u32> = None;
        for i in (0..n).rev() {
            let (r, along) = on[i];
            if at_junction(pts[i]) {
                // (Turning there off the road it comes in by.)
                let came = on[i.saturating_sub(1)].0;
                left_ahead = (turn_at(i) < -LEFT_AT).then_some(came);
            } else if left_ahead.is_some_and(|l| l != r) {
                left_ahead = None;
            }
            let idx = u8::from(road(i).kind == RoadKind::Avenue && left_ahead == Some(r));
            base[i] = lane_off(g, r, idx, along);
        }
        // Where its lane changes, from its point `i` to the next: the stretch along it it moves
        // over in, from and to.
        let changes: Vec<usize> = (0..n - 1)
            .filter(|&i| {
                (base[i + 1] - base[i]).abs() > 1e-3 && !(classic(road(i)) && classic(road(i + 1)))
            })
            .collect();
        let boxed = |p: [f32; 2]| in_box(&g.boxes, p, 0.0);
        let mut windows: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(changes.len());
        let mut turns: Vec<usize> = Vec::new();
        for (c, &i) in changes.iter().enumerate() {
            let (a, b) = if turn_at(i).abs() > TURN_AT {
                // Turning: as it turns.
                turns.push(i);
                (s[i], (s[i] + SWITCH).min(s[i + 1]))
            } else if boxed(pts[i]) || boxed(pts[i + 1]) {
                // Straight through a junction: across its box.
                let mid = if boxed(pts[i]) { s[i] } else { s[i + 1] };
                let (mut a, mut b) = (mid, mid);
                while a > mid - BOX_LOOK && boxed(at(a - BOX_STEP)) {
                    a -= BOX_STEP;
                }
                while b < mid + BOX_LOOK && boxed(at(b + BOX_STEP)) {
                    b += BOX_STEP;
                }
                (a, b)
            } else if road(i + 1).kind == RoadKind::Motorway && road(i).kind != RoadKind::Motorway {
                // Merging into a carriageway: eased along the ramp, in its lane at the merge.
                (s[i] - TAPER, s[i])
            } else {
                // Off a carriageway (or on from one road's end to the next's): eased along the
                // road on.
                (s[i], s[i] + TAPER)
            };
            // (Not into the change before's, nor past where the next one begins.)
            let lo = windows.last().map_or(0.0, |w| w.1);
            let hi = changes.get(c + 1).map_or(s[n - 1], |&j| s[j]);
            let (a, b) = (a.max(lo), b.min(hi));
            if b - a > 1e-3 {
                windows.push((a, b, base[i], base[i + 1]));
            }
        }
        // The lane at `x` m along it: in a change's stretch, eased across; else `own`.
        let off_at = |x: f32, own: f32| {
            windows
                .iter()
                .find(|w| x >= w.0 && x <= w.1)
                .map_or(own, |w| w.2 + (w.3 - w.2) * (x - w.0) / (w.1 - w.0))
        };
        let cuts: Vec<f32> = windows.iter().flat_map(|w| [w.0, w.1]).collect();
        let mut out = LaneLine {
            pts: Vec::with_capacity(n + cuts.len()),
            offs: Vec::with_capacity(n + cuts.len()),
            on: Vec::with_capacity(n + cuts.len()),
            near: None,
            corners: Vec::with_capacity(turns.len()),
        };
        let mut c = 0;
        for k in 0..n {
            // (Where a change begins or ends between the last point and this: a point of its own.)
            while c < cuts.len() && cuts[c] < s[k] - 0.05 {
                if k > 0 && cuts[c] > s[k - 1] + 0.05 {
                    out.pts.push(at(cuts[c]));
                    out.offs.push(off_at(cuts[c], base[k]));
                    out.on.push(on[k]);
                }
                c += 1;
            }
            while c < cuts.len() && cuts[c] <= s[k] + 0.05 {
                c += 1;
            }
            if turns.contains(&k) {
                out.corners.push(out.pts.len());
            }
            out.pts.push(pts[k]);
            out.offs.push(off_at(s[k], base[k]));
            out.on.push(on[k]);
        }
        out.near = Some(NearestPoint::new(out.pts.clone()));
        out
    }

    /// Its lane's line: its middle line's points moved right by their lanes (`offset_right_by`);
    /// where it turns from one lane into another (`corners`), the corner where the lane in meets
    /// the lane out (not the turn's middle moved by the lane in, which cut the corner by more the
    /// further out the lane: an avenue's outer lane began its turn 10 m short).
    pub(crate) fn line(&self) -> Vec<[f32; 2]> {
        let mut out = offset_right_by(&self.pts, &self.offs);
        for &m in &self.corners {
            if m == 0 || m + 1 >= out.len() {
                continue;
            }
            let (din, dout) = (
                sub2(self.pts[m], self.pts[m - 1]),
                sub2(self.pts[m + 1], self.pts[m]),
            );
            let cross = cross2(din, dout);
            let sin = cross / (dot2(din, din) * dot2(dout, dout)).sqrt().max(1e-6);
            if sin.abs() < 0.2 {
                continue;
            }
            let (a, b) = (out[m - 1], out[m + 1]);
            let t = cross2(sub2(b, a), dout) / cross;
            let x = [a[0] + din[0] * t, a[1] + din[1] * t];
            let reach = self.offs[m - 1].abs().max(self.offs[m + 1].abs());
            if dist(x, self.pts[m]) < 2.0 * reach + 1.0 {
                out[m] = x;
            }
        }
        out
    }

    /// Its point nearest `p` (one of its lane's points), if it isn't `classic` (whose callers look
    /// along the way's own points, as they always did).
    pub(crate) fn nearest(&self, p: [f32; 2]) -> Option<usize> {
        self.near.as_ref()?.nearest(p)
    }

    /// At its point `i`: the road's width and kind, and `meet`'s room and lane for a car `wid` m
    /// wide in its lane there.
    pub(crate) fn point(&self, g: &Graph, i: usize, wid: f32) -> (f32, RoadKind, f32, f32) {
        let road = &g.roads[self.on[i].0 as usize];
        let (room, lane) = meet(road, self.offs[i], wid);
        (road.width, road.kind, room, lane)
    }
}

/// What a way's middle-line points (`pts`, each on the road `on` says: `LaneLine`) are to the cars on
/// it: each point's level (`Graph::layer_at`), and whether it's where the way merges off a ramp
/// into a motorway's carriageway (its last point on the ramp: the merge). None where the network has
/// neither decks nor merges: every point on the ground, no merge.
pub(crate) fn way_marks(
    g: &Graph,
    pts: &[[f32; 2]],
    on: &[(u32, bool)],
) -> Option<Vec<(crate::travel::Layer, bool)>> {
    if !g.has_decks() && !g.has_merges() {
        return None;
    }
    let kind = |i: usize| g.roads[on[i].0 as usize].kind;
    Some(
        (0..pts.len().min(on.len()))
            .map(|i| {
                let (s, _) = g.roads[on[i].0 as usize].project(pts[i]);
                let merge = kind(i) == RoadKind::Ramp
                    && on.get(i + 1).is_some()
                    && kind(i + 1) == RoadKind::Motorway;
                (g.layer_at(on[i].0, s), merge)
            })
            .collect(),
    )
}

/// How far on along a line `Along` looks for the nearest of its points to the next (points).
const ALONG_LOOK: usize = 64;

/// A walk along a line's points (`line`), finding for each point of a lane laid from it (thinned,
/// its corners rounded), in order, the line's point it was laid from: the nearest a little way on
/// from the last found, never back. (Where a way passes the same place twice, under a deck and
/// later on it, the nearest point alone could be either.)
pub(crate) struct Along<'a> {
    line: &'a [[f32; 2]],
    at: usize,
    /// The line's points up to here are marked on the lane already (`mark`).
    marked: usize,
}

impl<'a> Along<'a> {
    pub(crate) fn new(line: &'a [[f32; 2]]) -> Self {
        Along {
            line,
            at: 0,
            marked: 0,
        }
    }

    /// The marks (`way_marks`) of the lane's next point `p`: the level of the line's point it was
    /// laid from (`next`), and whether it merges there or at a point of the line since the last's
    /// (a ramp meeting its carriageway at a shallow angle: thinning the lane left out its point).
    pub(crate) fn mark(
        &mut self,
        marks: &[(crate::travel::Layer, bool)],
        p: [f32; 2],
    ) -> (crate::travel::Layer, bool) {
        let i = self.next(p);
        let merge = marks[self.marked.min(i)..=i].iter().any(|m| m.1);
        self.marked = i + 1;
        (marks[i].0, merge)
    }

    /// The line's point the lane's next point `p` was laid from (`Along`).
    pub(crate) fn next(&mut self, p: [f32; 2]) -> usize {
        let end = (self.at + ALONG_LOOK).min(self.line.len());
        let mut best = (f32::INFINITY, self.at);
        for (i, &q) in self.line.iter().enumerate().take(end).skip(self.at) {
            let d = dist(q, p);
            if d < best.0 {
                best = (d, i);
            }
        }
        self.at = best.1;
        best.1
    }
}

/// On `road`, a car `wid` m wide whose lane is `off` m right of its middle line: how far right of
/// its lane it can pull over to pass one coming the other way (`pull_room`; none where nobody comes
/// the other way in its lane, or the road isn't one cars always drove), and how far right of the
/// road's middle its lane is as one coming the other way meets it (`Route::lane`: `off`, or
/// `NO_ONCOMING`).
pub(crate) fn meet(road: &Road, off: f32, wid: f32) -> (f32, f32) {
    let room = if classic(road) {
        pull_room(road.width, road.kind, wid)
    } else {
        0.0
    };
    let lane = if oncoming(road) { off } else { NO_ONCOMING };
    (room, lane)
}
