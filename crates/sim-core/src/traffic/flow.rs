//! Traffic itself: who is moving, who is waiting and who has been asked to wait.
//!
//! Split from `traffic.rs` (see `mod.rs`).

use super::*;

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
pub(crate) struct Hold {
    pub(crate) key: u64,
    tag: u32,
    claim: Claim,
    /// How far along its route its holder was as the tick began.
    at: f32,
}

/// A vehicle's ask for the parts of a group of its route's claims (`Claim::group`), in order:
/// granted all at once, in the order the vehicles asked (`tick`).
#[derive(Debug, Clone)]
pub(crate) struct Ask {
    tick: u64,
    pub(crate) key: u64,
    route: u32,
    group: u32,
    parts: Vec<(u32, Claim)>,
    /// The last tick it waited at the start of the group (asking again: one not asked again is
    /// dropped).
    seen: u64,
    /// Whom it waited for as the tick began (their keys: their claims, bodies).
    pub(crate) by: Vec<u64>,
    /// The earlier asks it waited for as the tick began (their keys).
    pub(crate) fifo: Vec<u64>,
    /// Of `by`, those only on the move into its way, not in it: they're to stop short, it doesn't
    /// wait for them to go first (`begin`, rule 3).
    coming: Vec<u64>,
    /// Held up by nobody's claim nor an earlier ask, only by bodies: once those still to come
    /// keep clear (`blocked`), it's granted.
    bodies_only: bool,
    /// Long asked (`OLD_ASK`), whom it waits for, however far round (as the tick began, those only
    /// coming into its way left out): none of them keeps clear of it (`blocked`).
    reach: Vec<u64>,
    /// How far along its route its vehicle is (ticks, as the tick began or as it asked): past a
    /// stop line of its group's lights already, that light is behind it (`Claim::lights`).
    at: f32,
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
pub(crate) const OLD_ASK: u64 = crate::drive_secs(20.0);

/// The claims held and asked for (see the module notes). Not saved.
#[derive(Debug, Clone, Default)]
pub struct Traffic {
    /// The tick (`begin`).
    pub(crate) tick: u64,
    pub(crate) holds: Vec<Hold>,
    pub(crate) asks: Vec<Ask>,
    /// Each lit junction's lights (`Travel::signals`): the arm whose turn it is, how far through
    /// it (`Light`), and since when.
    pub(crate) lights: Vec<Light>,
    pub(crate) movers: Vec<Mover>,
    /// `movers` by key: each key and the first of them with it, sorted (`mover`).
    pub(crate) mover_ix: Vec<(u64, u32)>,
    /// Where each vehicle on its way to the start of its next claim, not yet there, will wait for
    /// it (its key, the claim's first sample: `may_go`, this tick).
    pub(crate) starts: Vec<(u64, Sample)>,
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
    pub(crate) lit_phases: Vec<[u64; 4]>,
    /// Per lit junction, since when its green is held for those through it from a lit junction
    /// just before (`Claim::gate2`, `GREEN_HELD`), and whether one may still be granted through
    /// it (`Traffic::green_through`).
    pub(crate) held: Vec<Option<u64>>,
    pub(crate) fresh: Vec<bool>,
    /// The lit junctions someone on foot waits at a crosswalk of (`Signal` indices, this tick:
    /// `kerb_stop`, `traffic::begin`), and the phase they wait for (`Signal::wants`: the walk, or
    /// on a side road's crosswalk its main road's green).
    pub(crate) walkers: Vec<Gate>,
    /// The lit junctions someone on foot is on a crosswalk of, and its arm (this tick,
    /// `on_crossings`), in order: their walk phase isn't over before they're across, nor does the
    /// phase of the arm they're on come round.
    pub(crate) crossing: Vec<(u16, u8)>,
    /// Those on foot on a zebra of a junction giving way, or stepping onto one, and those on a
    /// side road's crosswalk at a lit junction, open on its main road's green
    /// (`Signal::opens_with`), or stepping onto it then (`on_crossings`).
    pub(crate) zebra_walkers: Vec<OnZebra>,
    /// The lights cars on the road are coming up to (`Claim::gate`, `cars::movers`: within
    /// `APPROACH` of their stop line, not yet asking): its green kept on for them.
    pub(crate) approaching: Vec<Gate>,
}

/// Someone on foot on a zebra or stepping onto it (`Traffic::zebra_walkers`): at a lit junction
/// (`Travel::signals`, a side road's crosswalk: `Signal::opens_with`) or one giving way
/// (`Travel::give_way`), which, the arm, who, and whether on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OnZebra {
    pub lit: bool,
    pub junction: u16,
    pub arm: u8,
    pub id: u32,
    pub on: bool,
}

/// A car this near its next lights' stop line (ticks along its way: two seconds of drive time,
/// some 18 m at a town's 9 m/s) counts as coming to its green (`Traffic::approaching`).
pub(crate) const APPROACH: f32 = crate::drive_ticks(2.0);

/// Claim `k` of a route tagged `route` (`Place`).
pub(crate) fn tag(route: u32, k: usize) -> u32 {
    route << 16 | k as u32
}

impl Traffic {
    /// Whether `key` holds its claim `tag`.
    pub(crate) fn holds(&self, key: u64, tag: u32) -> bool {
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

    /// `key`'s claims worked out again `t` ticks along its route (`route`, `claims`: a way laid
    /// round a taxi standing at the kerb, `cars::pass_stands`): what it held and asked for let go,
    /// as `forget`, but every part of the group it's in the middle of held at once, as it held it.
    /// It can't stand clear between them: let go, the rest of a way through two lit junctions
    /// joined as one was granted to one coming the other way, in its way, and the two stood nose
    /// to nose in the box for good (Willowley).
    pub(crate) fn reclaimed(&mut self, key: u64, route: u32, claims: &[Claim], t: f32) {
        self.forget(key);
        let Some(group) = claims
            .iter()
            .find(|c| c.from <= t + 1e-3 && t < c.to)
            .map(|c| c.group)
        else {
            return;
        };
        for (k, c) in claims.iter().enumerate() {
            if c.group == group && t < c.to {
                self.holds.push(Hold {
                    key,
                    tag: tag(route, k),
                    claim: c.clone(),
                    at: t,
                });
            }
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
        let found = self
            .mover_ix
            .binary_search_by_key(&key, |e| e.0)
            .ok()
            .map(|k| &self.movers[self.mover_ix[k].1 as usize]);
        debug_assert!(
            found.map(|m| m as *const Mover)
                == self
                    .movers
                    .iter()
                    .find(|m| m.key == key)
                    .map(|m| m as *const Mover)
        );
        found
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
                Some(a) if a.parts.first().is_some_and(|p| p.0 == tag) => {
                    a.seen = tick;
                    a.at = t;
                }
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
                        at: t,
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

    /// Whether vehicle `key` waits at a red light this tick (`at_red`).
    pub(crate) fn red_for(&self, key: u64) -> bool {
        self.asks.iter().any(|a| {
            a.key == key
                && a.seen + 1 >= self.tick
                && a.parts
                    .first()
                    .is_some_and(|p| !self.green_through(&p.1, a.at))
        })
    }

    /// Whether a group through claim `c`'s lights may be asked for now by one `at` ticks along its
    /// route: green at each of them it still comes to (`Claim::lights`; none: no lights).
    pub(crate) fn green_through(&self, c: &Claim, at: f32) -> bool {
        let (gate, gate2) = c.lights(at);
        self.green(gate)
            && self.green(gate2)
            && gate2.is_none_or(|(s, _)| self.fresh.get(usize::from(s)).copied().unwrap_or(true))
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
            Stage::Green if sig.opens_with(arm) == Some(l.phase) => Head::Walk,
            Stage::Green if l.phase == phase => Head::Green,
            Stage::Amber if l.phase == phase => Head::Amber,
            _ => Head::Red,
        })
    }

    /// The lights a tick on (`Light`): a phase's green `MIN_GREEN` at least, on while anyone comes
    /// to it (an ask through it, one holding a claim through it still out of the box, or one
    /// coming up to its stop line, `Traffic::approaching`) and its share of `CYCLE` at most, and
    /// only while someone waits at another (or on foot at the crosswalks, `Traffic::walkers`);
    /// then amber (`AMBER`), all red while the box clears (`ALL_RED`, and while someone's on a
    /// crosswalk of an arm the next lets in: a side road's, crossed on the main road's green), and
    /// the next phase round that anyone waits at. The walk phase: those on foot may step out for
    /// `WALK_OPEN` (once nobody waits at the kerbs nor is on the crosswalks, after `WALK_LEAST`,
    /// no longer), then its
    /// clearance (`Signal::walk_clear`, over once nobody's on them), then all red, and the cars'
    /// phase after it. Nobody waiting anywhere nor coming, it goes back to the main road's green
    /// and rests there (`Signal::rest`; a roundabout's stays where it is).
    pub(crate) fn change_lights(&mut self, tick: u64, signals: &[Signal]) {
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
                let (gate, gate2) = first.1.lights(a.at);
                for (s, phase) in [gate, gate2].into_iter().flatten() {
                    if let Some(m) = phases.get_mut(usize::from(s)) {
                        m[0] |= bit(phase);
                    }
                }
                // (Waiting through two: the one of the two listed first leads, the other follows.
                // The leader on its green for it, that's held, and the other's phase it needs is
                // wanted there, next. Each holding the other's green for one waiting at its own
                // red, two lights in step kept two cars crossing each other's way at red for
                // hours: Hollyglade.)
                if let (Some(a), Some(b)) = (gate, gate2) {
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
        for &(s, phase) in &self.walkers {
            if let Some(m) = phases.get_mut(usize::from(s)) {
                m[0] |= bit(phase);
            }
        }
        // (Cars coming up to a light, not yet at its stop line: coming to its green, as those
        // granted through it and still out of the box are.)
        for &(s, phase) in &self.approaching {
            if let Some(m) = phases.get_mut(usize::from(s)) {
                m[1] |= bit(phase);
            }
        }
        for (i, sig) in signals.iter().enumerate() {
            let waits = |phase: u8| phases[i][0] & bit(phase) != 0;
            let coming = |phase: u8| phases[i][1] & bit(phase) != 0;
            let n = sig.turns();
            let mut l = self.lights[i];
            // (Loaded with another junction's, or the roads changed: the green it rests on.)
            if l.phase >= n {
                l = Light {
                    phase: sig.rest,
                    ..Light::default()
                };
            }
            // (Someone on foot on one of its crosswalks; on one of a phase's arms.)
            let on = {
                let j = i as u16;
                let from = self.crossing.partition_point(|c| c.0 < j);
                let to = self.crossing.partition_point(|c| c.0 <= j);
                &self.crossing[from..to]
            };
            let on_it = !on.is_empty();
            let on_arms = |phase: u8| {
                on.iter().any(|&(_, a)| {
                    sig.arms
                        .get(usize::from(a))
                        .is_some_and(|a| a.phase == phase)
                })
            };
            let walk = sig.walk_phase() == Some(l.phase);
            let on = tick.saturating_sub(l.since);
            let next_up = |l: Light| {
                (1..=n)
                    .map(|k| (l.phase + k) % n)
                    .find(|&x| phases[i][3] & bit(x) != 0 && x != l.phase)
                    .filter(|_| phases[i][3] & bit(l.phase) == 0)
                    .or_else(|| (1..n).map(|k| (l.phase + k) % n).find(|&x| waits(x)))
                    .unwrap_or(if sig.roundabout && !walk {
                        l.phase
                    } else {
                        sig.rest
                    })
            };
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
                // (The walk: over once nobody waits at its kerbs nor is on its crosswalks, its
                // clearance with it, `WALK_LEAST` at least, so those waiting have stepped out.)
                Stage::Green if walk => {
                    if on >= WALK_OPEN || on >= WALK_LEAST && !waits(l.phase) && !on_it {
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
                        // (Nobody waiting anywhere, nor coming to it, off the main road's phase:
                        // back to that, to rest on, so its cars don't stop for nobody.)
                        None if l.phase != sig.rest
                            && !sig.roundabout
                            && on >= MIN_GREEN
                            && !busy
                            && !kept =>
                        {
                            stage(Stage::Amber)
                        }
                        _ => l,
                    }
                }
                // (The walk's clearance: over as soon as nobody's on its crosswalks.)
                Stage::Amber if walk && (on >= sig.walk_clear || !on_it) => stage(Stage::Clear),
                Stage::Amber if !walk && on >= AMBER => stage(Stage::Clear),
                // (Nobody waiting: the phase it rests on, its main road's. One wanted by a lit
                // junction next to it, first; not again straight after its own turn: those at the
                // others' reds waited for good, one stopped in the box among them, Wheat Bottom.
                // Not while someone on foot is on the crosswalk of an arm whose cars it lets in,
                // a side road's crossed on the main road's green (`Signal::opens_with`), its walk's
                // clearance long at most.)
                Stage::Clear
                    if on >= ALL_RED
                        && (!on_arms(next_up(l)) || on >= ALL_RED + sig.walk_clear) =>
                {
                    Light {
                        phase: next_up(l),
                        stage: Stage::Green,
                        since: tick,
                    }
                }
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
        // (Looked up by key from here on: `mover`, the first with each key.)
        self.mover_ix.clear();
        self.mover_ix.extend(
            self.movers
                .iter()
                .enumerate()
                .map(|(i, m)| (m.key, i as u32)),
        );
        self.mover_ix.sort_by_key(|&(key, i)| (key, i));
        self.mover_ix.dedup_by_key(|e| e.0);
        self.starts.clear();
        // (Where each holder or asker is, by key and route: the first place given for them.)
        let mut at: Vec<(u64, u32, usize)> = places
            .iter()
            .enumerate()
            .map(|(i, p)| (p.0, p.1, i))
            .collect();
        at.sort_unstable();
        at.dedup_by_key(|e| (e.0, e.1));
        let place = |key: u64, route: u32| {
            let found = at
                .binary_search_by_key(&(key, route), |e| (e.0, e.1))
                .ok()
                .map(|k| places[at[k].2].2);
            debug_assert_eq!(
                found.map(f32::to_bits),
                places
                    .iter()
                    .find(|p| p.0 == key && p.1 == route)
                    .map(|p| p.2.to_bits())
            );
            found
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
        for a in &mut self.asks {
            if let Some(t) = place(a.key, a.route) {
                a.at = t;
            }
        }
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
        let on_road = Reaches::new(graph.clone());
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
        // (Neither changes from here on: whom each waits for however far round is worked out
        // once for each, `Reaches`.)
        let waits_round = Reaches::new(graph);
        let graph = &waits_round.graph;
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
            let red = !self.green_through(next[0], a.at);
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
                    let through = || {
                        holder.is_some_and(|m| {
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
                        })
                    };
                    let waits = || {
                        !through()
                            && (holder.is_some_and(|m| m.waits == Some(a.key))
                                || on_road.reaches(h.key, a.key))
                    };
                    // (Whether it's in the way at all first: the rest only for those that are.)
                    h.key != a.key
                        && next
                            .iter()
                            .any(|c| overlaps((c, &c.samples[mine(c, h.key)]), (&h.claim, rest)))
                        && !self.passing.contains(&(a.key, h.key))
                        && !waits()
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
                let first = || {
                    if old {
                        waits_round.reaches(e.key, a.key)
                    } else {
                        bodies_of.contains(&a.key) || waits_round.reaches(e.key, a.key)
                    }
                };
                // (Whether it's in the way at all first: who goes first only for those that are.)
                next.iter().any(|c| {
                    e.parts
                        .iter()
                        .any(|(_, o)| overlaps((c, &c.samples[mine(c, e.key)]), (o, &o.samples)))
                }) && !first()
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
                // (Nowhere near any of its parts, nor on the move or backing up into them: it's in
                // no way of this one, `hit` below finding nothing.)
                let turning = self.turning.iter().find(|t| t.0 == m.key);
                let near = |r: &Rect| next.iter().any(|c| in_bounds(c, r));
                if !near(&m.rect)
                    && !m.ahead.iter().any(near)
                    && !turning.is_some_and(|(_, path)| path.iter().any(near))
                {
                    continue;
                }
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
                if waits_round.reaches(m.key, a.key) && !hit(&m.rect, true) {
                    continue;
                }
                let inside = hit(&m.rect, strict);
                // (One turning round (`cars::unjam`) is on its way into the ground it still backs
                // up over, however slowly, and stands at its end till it's on its way out of it:
                // it has the way there (`cars::drive_on`). Let into a narrow stretch whose far end
                // that was, one came up nose to nose with it there, the one turning round waiting
                // for its claim and that one for its body, for good, Mirrorshore.)
                let backs_in =
                    !inside && turning.is_some_and(|(_, path)| path.iter().any(|r| hit(r, true)));
                let on_into = backs_in || m.moving && m.ahead.iter().any(|r| hit(r, false));
                if !inside && on_into {
                    coming.push(m.key);
                }
                if inside || on_into {
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
