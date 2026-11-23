use super::*;

impl Traffic {
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
                                layer: m.layer,
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
                        layer: m.layer,
                    };
                    next.iter().any(|c| {
                        in_bounds(c, r)
                            && c.samples.iter().any(|s| {
                                // (Its far side clear too: don't block the box. Into a
                                // merge, those coming along the carriageway its way too.)
                                conflict(s, &o, strict || c.merge || s.t > c.to)
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
