use super::*;

impl Traffic {
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
}
