//! Junction signals: their phases, their arms and where they stand.

use super::*;

/// A junction where `SIGNAL_ARMS` roads for cars or more meet (`settle`), lit or not (`lit`). A big
/// one is lit (`Travel::signals`): where `LIT_ARMS` arms or more meet, or two main roads (streets or
/// country roads) do, `LIT_MAIN_ARMS` of its arms theirs (one meeting another, or crossing it; a
/// country road's arm counted only with a street's there, and four or more only country roads'
/// not: `lit_of`), and on a roundabout's ring. Any other (a lane meeting a street or another lane,
/// country roads meeting out in the open)
/// gives way (`Travel::give_way`): cars take turns through its box by the claims as anywhere
/// (`boxes`), and on a street's arm there's a zebra (`SignalArm::crosswalk`: one of the street's
/// two arms through it, one for junctions giving way close together along it, `ZEBRA_APART`),
/// where they stop only for someone on foot on it or stepping onto it (`zebra_stops`). The
/// owner's call (2026-11-13): with lights and zebras at every junction, a quiet side street's T had
/// cars stopping at its crosswalks for nobody.
///
/// A lit one: its middle, and its arms, each with its phase (`SignalArm::phase`: two
/// arms more than `OPPOSITE` apart go together, an arm with none across from it on its own). One
/// phase at a time has green (`Light`); a group through it is asked for only on its arm's phase's
/// green (`Claim::gate`), at its arm's stop line (`SignalArm::stop`), and holding it, one goes on
/// through on amber and red; the claims keep anyone from driving into another in the box as ever
/// (one turning across the way of those coming the other way on the same green waits for them).
/// Someone on foot waiting at its main road's crosswalks (`kerb_stop`) brings its walk phase round
/// (`Signal::walk_phase`): every arm red, and the crosswalks, closed to those on foot otherwise,
/// open; a side road's crosswalk is open on the main road's green too (`Signal::opens_with`), and
/// those waiting there wait for that. A roundabout's (`Signal::roundabout`) is lit as ever: each
/// arm a phase of its own, its pole just out past the box, cars stopping at the box's edge, and no
/// crosswalks.
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
    /// Whether it has traffic lights (a big junction: see above); else it gives way.
    pub lit: bool,
    /// The phase its lights rest on with nobody waiting (`change_lights`): its main road's, the
    /// phase of its streets' and country roads' arms (a road through it counted twice), the first
    /// of two as good.
    pub rest: u8,
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

    /// The phase on whose green arm `arm`'s crosswalk is open to those on foot too, not only in the
    /// walk phase: a side road's (an arm out of the phase its lights rest on, `rest`) on its main
    /// road's. Those on foot cross the side road's mouth alongside the main road's cars, as they go
    /// straight on past it, and those turning into it stop at their stop line for anyone on it
    /// (`side_zebra_stops`, `cars::zebra_yield`). Someone walking along the main road's pavement
    /// past a lit T stopped every arm for the walk (the owner's call, 2026-11-13). None: crossed on
    /// the walk only (the main road's own arms, a roundabout's).
    pub fn opens_with(&self, arm: usize) -> Option<u8> {
        let a = self.arms.get(arm)?;
        (self.lit && !self.roundabout && a.crosswalk && a.phase != self.rest).then_some(self.rest)
    }

    /// The phase someone on foot waiting at arm `arm`'s crosswalk waits for (`Traffic::walkers`):
    /// its main road's green on a side road's (`opens_with`), else the walk.
    pub(crate) fn wants(&self, arm: usize) -> Option<u8> {
        self.opens_with(arm).or_else(|| self.walk_phase())
    }

    /// Whether `p` is on one of its crosswalks (`kerb_stop`): an arm's with one
    /// (`SignalArm::crosswalk`), from `walk_from` to `walk_to` out, in its carriageway (`KERB_IN`
    /// in from the kerb: one walking along the pavement by it isn't on it). Its box and the
    /// carriageway either side of the crosswalks nobody on foot walks on at all
    /// (`Travel::junction_tiles`). None of a roundabout's; a country road's arm is crossed as
    /// anywhere else, out past the box.
    pub fn crossing(&self, p: [f32; 2]) -> bool {
        self.crossing_arm(p).is_some()
    }

    /// The arm whose crosswalk `p` is on (`crossing`), if any.
    pub fn crossing_arm(&self, p: [f32; 2]) -> Option<usize> {
        if self.walk_clear == 0 {
            return None;
        }
        self.arms.iter().position(|a| {
            a.crosswalk && {
                let (along, across) = a.out(self.at, p);
                (a.walk_from..=a.walk_to).contains(&along) && across <= a.kerb - KERB_IN
            }
        })
    }

    /// Whether `p` is where those on foot never walk (`Travel::junction_tiles`): its box, between
    /// the kerbs out to the crosswalks (out past the box on an arm without one), and on an arm with
    /// one, from its far side out to `KEEP_OFF` past the stop line; not on a crosswalk itself.
    /// (Not a roundabout's.)
    pub(crate) fn off_limits(&self, p: [f32; 2]) -> bool {
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
    pub(crate) fn crossing_reach(&self) -> f32 {
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
/// cars stop on red with their noses (or for those on foot on its zebra). Whether it has a
/// crosswalk at all (`crosswalk`, the zebra the renderer draws, and the pedestrians' head): at a lit
/// junction a street's or a lane's (one with sidewalks), at one giving way a street's, one of its
/// two arms through it (`signal_at`); not a country road's nor a roundabout's; those on foot wait
/// for the walk phase only at those.
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
    /// Red for the cars, and walk for those on foot (the walk phase; a side road's arm on the main
    /// road's green: `Signal::opens_with`).
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

/// A junction is where this many roads for cars meet (`Signal`), and it has traffic lights where
/// `LIT_ARMS` do, or where `LIT_MAIN_ARMS` of them are streets' or country roads' (two main roads
/// meeting: a street through it has two arms, a lane meeting it one more; country roads' only with
/// a street's: `lit_of`).
pub(crate) const SIGNAL_ARMS: usize = 3;
pub(crate) const LIT_ARMS: usize = 4;
pub(crate) const LIT_MAIN_ARMS: usize = 3;
/// How long a phase's green lasts at least, its amber, and all red after while the box
/// clears (ticks of drive time, as the cars go through).
pub(crate) const MIN_GREEN: u64 = crate::drive_secs(5.0);
/// Every phase's turn comes round in this long at most (ticks of drive time): a phase's green
/// lasts no longer than its share of it (`change_lights`).
pub(crate) const CYCLE: u64 = crate::drive_secs(50.0);
pub(crate) const AMBER: u64 = crate::drive_secs(2.0);
/// Through two lit junctions too near to wait between (`Claim::gate2`): the second's green is
/// kept on for those waiting at the first's stop line or granted through both, from when one
/// first is, for `CYCLE` at most (the first's turn comes round for them meanwhile); one is
/// granted through only with `GREEN_HELD` of that still to come (its way to the second's stop
/// line).
pub(crate) const GREEN_HELD: u64 = crate::drive_secs(10.0);
pub(crate) const ALL_RED: u64 = crate::drive_secs(4.0);
/// A route this near a lit junction's middle goes through its lights (m, `gate`), and its arm is
/// the one it's on where it first comes this near (`GATE_REACH`).
pub(crate) const GATE_THROUGH: f32 = BOX_LEAST + 1.0;
pub(crate) const GATE_REACH: f32 = BOX_MOST + 2.0;
/// How far out past its arm's box a roundabout's signal pole stands (another's stands at the stop
/// line), and how far in from the kerb on the pavement (or the road's edge) any stands (m).
const POLE_OUT: f32 = 1.0;
const POLE_KERB: f32 = 0.4;
/// A pole stands this far off every road's carriageway at least (m, `signal_at`), looked for along
/// its kerb `POLE_STEP` m at a time, `POLE_STEPS` each way at most, over the roads `POLE_LOOK` m
/// round it.
const POLE_CLEAR: f32 = 0.15;
const POLE_STEP: f32 = 0.5;
const POLE_STEPS: usize = 24;
const POLE_LOOK: f32 = 10.0;
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
pub(crate) const KEEP_OFF_ROOM: i32 = 4;
const KERB_IN: f32 = 0.25;
/// An arm's crosswalk is measured along its own road (`SignalArm::span`) where that goes out within
/// 30° of its way (the cosine), its crosswalk this near as far out as the arm's (m); else along
/// `way`.
const SPAN_ALONG: f32 = 0.866;
const SPAN_OFF: f32 = 2.0;
/// The walk phase (`Signal::walk_phase`): how long those on foot may step out (ticks of real time,
/// as people walk), and the pace its clearance gives the slowest to get across the widest arm
/// (m a second; `Signal::walk_clear`), `WALK_CLEAR_LEAST` at least (s).
pub(crate) const WALK_OPEN: u64 = crate::secs(5.0);
/// The walk phase is open this long at least (ticks of real time): those waiting at the kerb step
/// out (`change_lights`).
pub(crate) const WALK_LEAST: u64 = crate::secs(1.0);
const WALK_PACE: f32 = 1.0;
const WALK_CLEAR_LEAST: f32 = 3.0;

/// An arm's road from its junction (`signal_at`): which, where along it and which way (+1, -1),
/// and the stretch of it between its ends' nodes.
type ArmRoad = (u32, f32, f32, f32, f32);

/// A junction's arm for cars as `signal_at` finds it: its way out, how far from its middle its kerb
/// is, its road from the node (`ArmRoad`), the node at the far end of its stretch, and the
/// stretch's length.
type ArmAt = ([f32; 2], f32, ArmRoad, usize, f32);

/// The arms for cars of node `n` of `g` (`ArmAt`), round the way they go (by their angle).
fn arms_at(g: &Graph, n: usize) -> Vec<ArmAt> {
    let p = g.nodes[n];
    let mut arms: Vec<ArmAt> = Vec::new();
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
                arms.push((
                    [(q[0] - p[0]) / l, (q[1] - p[1]) / l],
                    crate::cars::kerb_of(road.width, road.kind),
                    (e.road, s, k, e.s0, e.s1),
                    far as usize,
                    e.len(),
                ));
            }
        }
    }
    arms.sort_by(|a, b| a.0[0].atan2(a.0[1]).total_cmp(&b.0[0].atan2(b.0[1])));
    arms
}

/// How far out along each of `arms` (`arms_at`) a lit junction's box reaches with its corners: as
/// far as two bodies in their lanes on it and another are `ARMS_APART` apart (`BOX_LEAST` to
/// `BOX_MOST`), or where the kerb's return round from the next arm ends (`R / tan(θ / 2)` past
/// where the two kerbs would meet; none with an arm across), whichever is further, `BOX_MOST` at
/// most: where its crosswalk begins (`SignalArm::walk_from`) with room for it.
fn box_edges(arms: &[ArmAt]) -> Vec<(f32, f32)> {
    let dot = |a: [f32; 2], b: [f32; 2]| a[0] * b[0] + a[1] * b[1];
    (0..arms.len())
        .map(|i| {
            let (wa, ka) = (arms[i].0, arms[i].1);
            let mut edge = BOX_LEAST;
            let mut corner = 0.0f32;
            for (j, b) in arms.iter().enumerate() {
                if j == i {
                    continue;
                }
                let cos = dot(wa, b.0).clamp(-1.0, 1.0);
                let sin = (1.0 - cos * cos).max(0.0).sqrt();
                if cos > 0.0 {
                    edge = edge.max(ARMS_APART / sin.max(1e-3));
                }
                if cos >= OPPOSITE {
                    let sin = sin.max(1e-3);
                    corner = corner.max((b.1 + ka * cos) / sin + KERB_RETURN * (1.0 + cos) / sin);
                }
            }
            let edge = edge.min(BOX_MOST);
            (edge, edge.max(corner).min(BOX_MOST))
        })
        .collect()
}

/// Of a junction's `arms` (`arms_at`, round in order) on roads of `g`, which share a phase: of
/// two arms more than `OPPOSITE` apart, those of one road through it first, then two of one kind
/// (two streets), then the rest, each the two most nearly opposite first, of those left. So a
/// street's two arms through a junction go together, though a road leaving it at a slant is
/// further round from one of them (Canalside's j35: a country road 16° off the street and
/// straighter on than the street's own way, took the street's arm's place in its phase, and the
/// street's two arms were never green together). Each arm the phase it's in, numbered round in
/// order (on a roundabout, an arm each), and how many.
fn phases_of(g: &Graph, arms: &[ArmAt], roundabout: bool) -> (Vec<u8>, u8) {
    let dot = |a: [f32; 2], b: [f32; 2]| a[0] * b[0] + a[1] * b[1];
    let n = arms.len();
    let mut partner: Vec<Option<usize>> = vec![None; n];
    if !roundabout {
        let kind = |i: usize| g.roads[arms[i].2.0 as usize].kind;
        let rank = |i: usize, j: usize| {
            if arms[i].2.0 == arms[j].2.0 {
                0
            } else if kind(i) == kind(j) {
                1
            } else {
                2
            }
        };
        let mut pairs: Vec<(u8, f32, usize, usize)> = (0..n)
            .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
            .map(|(i, j)| (rank(i, j), dot(arms[i].0, arms[j].0), i, j))
            .filter(|&(_, d, ..)| d < OPPOSITE)
            .collect();
        pairs.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.total_cmp(&b.1))
                .then((a.2, a.3).cmp(&(b.2, b.3)))
        });
        for (.., i, j) in pairs {
            if partner[i].is_none() && partner[j].is_none() {
                partner[i] = Some(j);
                partner[j] = Some(i);
            }
        }
    }
    let mut phase = vec![u8::MAX; n];
    let mut phases = 0u8;
    for i in 0..n {
        if phase[i] != u8::MAX {
            continue;
        }
        phase[i] = phases;
        if let Some(j) = partner[i] {
            phase[j] = phases;
        }
        phases += 1;
    }
    (phase, phases)
}

/// Whether `p` is on a road's carriageway in `g` (within its kerb of its middle line and `margin`
/// more): no signal pole stands there (`signal_at`).
fn on_carriageway(g: &Graph, p: [f32; 2], margin: f32) -> bool {
    g.near(p, POLE_LOOK, 6, crate::travel::Mode::Car)
        .iter()
        .any(|rp| {
            let r = &g.roads[rp.road as usize];
            rp.off < crate::cars::kerb_of(r.width, r.kind) + margin
        })
}

/// Whether a road of `ring` (per road of a graph: a roundabout's) is among `arms`'.
fn on_ring(ring: &[bool], arms: &[ArmAt]) -> bool {
    arms.iter()
        .any(|a| ring.get(a.2.0 as usize).copied().unwrap_or(false))
}

/// Whether a junction with `arms` (`arms_at`) on roads of `g` is lit (`Signal::lit`): on a
/// roundabout's ring, `LIT_ARMS` arms or more (not all country roads'), or `LIT_MAIN_ARMS` of main
/// roads'; a country road's arm counts as a main road's only with a street's there too. Where
/// only country roads meet, out in the open, it gives way: 151 of seed base 0's 408 lit junctions
/// were country roads' T's and crossroads, their lights changing for the odd car (the owner's
/// call, 2026-11-13).
fn lit_of(g: &Graph, arms: &[ArmAt], ring: &[bool]) -> bool {
    use crate::travel::RoadKind;
    let count = |k: RoadKind| {
        arms.iter()
            .filter(|a| g.roads[a.2.0 as usize].kind == k)
            .count()
    };
    let streets = count(RoadKind::Street);
    let country = if streets > 0 {
        count(RoadKind::Country)
    } else {
        0
    };
    let all_country = count(RoadKind::Country) == arms.len();
    on_ring(ring, arms)
        || arms.len() >= LIT_ARMS && !all_country
        || streets + country >= LIT_MAIN_ARMS
}

/// What a junction giving way has along a street's arm (`signal_at`), within `ZEBRA_APART`: no
/// junction, a lit one, or one giving way before or after it along the street.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Near {
    Far,
    Lit,
    Before,
    After,
}

/// Junctions giving way this near each other along a street (m, from middle to middle) have one
/// zebra (`signal_at`): between them where it has room, its crosswalk no more than
/// `ZEBRA_SQUEEZE` m further in than at a T on its own (its stop line half way between them: two
/// T's 17 m apart, 1 m in).
pub const ZEBRA_APART: f32 = 40.0;
const ZEBRA_SQUEEZE: f32 = 1.5;

/// The junction at node `n` of `g` (`Signal`): whether it's lit (a big one: `LIT_ARMS` arms or
/// `LIT_MAIN_ARMS` of main roads, or on a roundabout), its arms for cars, each with its pole on the
/// kerb to the right of the way in, at its stop line, its phase and its crosswalk (`SignalArm`: a
/// lit one's on its streets' and lanes' arms, one giving way's on one of a street's two arms
/// through it; the lanes meeting there are crossed as anywhere), and the phase its lights rest on
/// (`Signal::rest`).
/// `ring`: per road of `g`, whether it's a roundabout's. None: fewer than `SIGNAL_ARMS` arms.
pub(crate) fn signal_at(g: &Graph, n: usize, ring: &[bool]) -> Option<Signal> {
    let p = g.nodes[n];
    let arms = arms_at(g, n);
    if arms.len() < SIGNAL_ARMS {
        return None;
    }
    let on_ring = |arms: &[ArmAt]| on_ring(ring, arms);
    let roundabout = on_ring(&arms);
    let (phase, phases) = phases_of(g, &arms, roundabout);
    let kind = |a: &ArmAt| g.roads[a.2.0 as usize].kind;
    let main = |a: &ArmAt| {
        matches!(
            kind(a),
            crate::travel::RoadKind::Street | crate::travel::RoadKind::Country
        )
    };
    let lit = lit_of(g, &arms, ring);
    // (Its main road's phase: of its streets' and country roads' arms the most; a lane's counts
    // half. A roundabout's, its first as ever.)
    let rest = if roundabout {
        0
    } else {
        (0..phases)
            .map(|k| {
                let score: u32 = (0..arms.len())
                    .filter(|&i| phase[i] == k)
                    .map(|i| if main(&arms[i]) { 2 } else { 1 })
                    .sum();
                (score, k)
            })
            .max_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)))
            .map_or(0, |(_, k)| k)
    };
    let edges = box_edges(&arms);
    // (Per arm, how far out along it the junction may reach: half way to the next junction, else
    // its stretch's length. And whether that next junction is lit too and so near that this arm's
    // crosswalk and stop line, and that one's back along it, can't both be out of the other's box
    // (`joined`): the two are one junction there, the stretch between them its box, no crosswalk
    // on it and the stop line half way. At Canalside's knot, j35's crosswalk and stop line stood
    // in j77's box, j77's in j35's.)
    let room: Vec<(f32, bool)> = arms
        .iter()
        .enumerate()
        .map(|(i, &(_, _, (road, _, k, s0, s1), far, len))| {
            if g.degree(far) < SIGNAL_ARMS {
                return (len, false);
            }
            let there = arms_at(g, far);
            let far_s = if k > 0.0 { s1 } else { s0 };
            let back = there
                .iter()
                .position(|b| b.2.0 == road && (b.2.1 - far_s).abs() < 1e-3 && b.2.2 == -k);
            let joined = !roundabout
                && far != n
                && there.len() >= SIGNAL_ARMS
                && !on_ring(&there)
                && back.is_some_and(|b| {
                    edges[i].1 + box_edges(&there)[b].1 + CROSSWALK + STOP_GAP > len
                });
            (len / 2.0, joined)
        })
        .collect();
    // (Giving way, a street through it has a zebra on one of its two arms, the first round that
    // has room for one: one either side of a lane's mouth, a quiet street had two for nobody.)
    let across = |i: usize| {
        (0..i).any(|j| {
            phase[j] == phase[i] && kind(&arms[j]) == crate::travel::RoadKind::Street && !room[j].1
        })
    };
    // (And one zebra for junctions giving way close together along a street (`ZEBRA_APART`):
    // two T's 17 m apart, their side roads either side of the street or on one side, had a zebra
    // each, on the stretch between them and the street's arms out past them, and read as one
    // junction with a zebra either side. The first of them along the street has it, on the
    // stretch to the next where that has room for it (between the two), else on its other arm;
    // the others none, nor one by a lit junction, whose crosswalk is there. Those on foot cross a
    // street anywhere off the lit junctions as ever: a zebra is only where cars stop for them.)
    let street = |i: usize| kind(&arms[i]) == crate::travel::RoadKind::Street;
    let near: Vec<Near> = arms
        .iter()
        .enumerate()
        .map(|(i, &(_, _, (_, _, k, ..), far, len))| {
            if lit || !street(i) || far == n || len > ZEBRA_APART || g.degree(far) < SIGNAL_ARMS {
                return Near::Far;
            }
            let there = arms_at(g, far);
            if there.len() < SIGNAL_ARMS {
                Near::Far
            } else if lit_of(g, &there, ring) {
                Near::Lit
            } else if k < 0.0 {
                Near::Before
            } else {
                Near::After
            }
        })
        .collect();
    let shared = near.iter().any(|x| matches!(x, Near::Lit | Near::Before));
    let fits = |i: usize| edges[i].1 + CROSSWALK + STOP_GAP <= room[i].0 + ZEBRA_SQUEEZE;
    let toward = (0..arms.len())
        .find(|&i| near[i] == Near::After && fits(i))
        .or_else(|| {
            near.contains(&Near::After)
                .then(|| (0..arms.len()).find(|&i| street(i) && near[i] == Near::Far))
                .flatten()
        });
    let zebra = |i: usize| !shared && toward.map_or(!across(i), |t| t == i);
    let arms: Vec<SignalArm> = arms
        .iter()
        .enumerate()
        .map(|(i, &(way, kerb, (road, s, k, s0, s1), ..))| {
            // (The crosswalk from the corner or the box's edge, the stop line past it; no further
            // out than the junction may reach along it, the box's edge at least. Joined to the
            // next lit junction (`room`): no crosswalk, the stop line half way.)
            let (edge, walk_from) = edges[i];
            let (room, joined) = room[i];
            let mut walk_to = walk_from + CROSSWALK;
            let mut stop = walk_to + STOP_GAP;
            if joined {
                stop = room;
                walk_to = room;
            } else if stop > room {
                stop = room.max(edge).min(stop);
                walk_to = walk_to.min(stop - STOP_GAP);
            }
            let walk_from = if joined {
                room
            } else {
                walk_from.min(walk_to - CROSSWALK).max(0.0)
            };
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
            // (Its pole: on its right at the stop line; where that's on another road's
            // carriageway, further out along its kerb (onto the island between it and a road
            // leaving at a slant), else back towards the junction as far as its crosswalk, else
            // on its left. Canalside's j35: a country road 16° off the street, the street arm's
            // head stood on its asphalt.)
            let base = if roundabout { edge + POLE_OUT } else { stop };
            let pole_at = |out: f32, side: f32| {
                let at = on(out);
                let t = r.tangent(at);
                let o = [t[0] * k, t[1] * k];
                let right = [o[1] * side, -o[0] * side];
                let m = r.at(at);
                let d = kerb + POLE_KERB;
                ([m[0] + right[0] * d, m[1] + right[1] * d], o)
            };
            let (pole, out) = [1.0f32, -1.0]
                .into_iter()
                .flat_map(|side| {
                    let outward = (0..=POLE_STEPS).map(move |j| base + POLE_STEP * j as f32);
                    let inward = (1..=POLE_STEPS)
                        .map(move |j| base - POLE_STEP * j as f32)
                        .take_while(move |&o| o >= walk_from.min(base));
                    outward.chain(inward).map(move |o| (o, side))
                })
                .map(|(o, side)| pole_at(o, side))
                .find(|&(q, _)| !on_carriageway(g, q, POLE_CLEAR))
                .unwrap_or_else(|| pole_at(base, 1.0));
            SignalArm {
                way,
                pole,
                yaw: out[0].atan2(out[1]),
                phase: phase[i],
                road,
                kerb,
                walk_from,
                walk_to,
                stop,
                span,
                crosswalk: !roundabout
                    && !joined
                    && match r.kind {
                        crate::travel::RoadKind::Street => lit || zebra(i),
                        crate::travel::RoadKind::Lane => lit,
                        _ => false,
                    },
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
        lit,
        rest,
    })
}
