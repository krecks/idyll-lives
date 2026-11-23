use super::*;

impl Traffic {
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
    /// way of a group asked for long ago (`OLD_ASK`). Only claims on its level (`layer`) count.
    pub(crate) fn blocked(
        &self,
        key: u64,
        now: &Rect,
        next: &Rect,
        way: [f32; 2],
        pass: f32,
        in_drive: bool,
        layer: crate::travel::Layer,
    ) -> Option<u64> {
        let sample = |rect: &Rect| Sample {
            t: 0.0,
            d: 0.0,
            rect: *rect,
            way,
            pass,
            layer,
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
}
