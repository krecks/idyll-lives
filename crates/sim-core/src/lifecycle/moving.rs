//! Moving house and newcomers (content `life.moving`, `life.newcomers`).

use super::*;

pub(super) fn members(w: &World, h: usize) -> Vec<usize> {
    (0..w.sims.len())
        .filter(|&i| w.sims[i].here() && w.sims[i].household as usize == h)
        .collect()
}

/// Residents of household `h` who are here and past the crib (who need a bed).
pub(super) fn sleepers(w: &World, h: usize) -> Vec<usize> {
    members(w, h)
        .into_iter()
        .filter(|&i| !w.content.life.baby(w.sims[i].age))
        .collect()
}

/// Bed places in the home on `plot`, broken ones too (whoever lives there fixes them; a house
/// whose only bed broke while empty would otherwise never be lived in again): one for each bed
/// and each bunk of a bed for two apart, and the couple's beds (a double bed: somewhere to try
/// for a baby, `privacy::couples_bed`), whose other half is only a partner's. Each place is for
/// the life stages its bed is for (content `stages`: the race car bed is a child's). A spare bed
/// opened out for the night (`Interaction::spare` and `leaves` open: the sofa bed pulled out) is a
/// place; a night on the sofa isn't.
pub(super) struct Beds {
    /// A place each, and the stages that may sleep there: a bed, a bunk, or a couple's bed slept
    /// in alone.
    places: Vec<StageMask>,
    /// Couple's beds' other halves, each a place more for a couple.
    couples: Vec<StageMask>,
}

impl Beds {
    /// Which of the sleepers of life stages `stages` have no place here (indices into it), with
    /// `pairs` couples among them: each takes a place their stage may sleep in, those with the
    /// fewest to choose from first and the place for the fewest stages first; then a couple's
    /// bed's other half for each couple.
    pub(super) fn unplaced(&self, stages: &[usize], pairs: usize) -> Vec<usize> {
        let fits = |mask: StageMask, s: usize| crate::content::stage_in(mask, s);
        let mut places = self.places.clone();
        places.sort_by_key(|m| m.count_ones());
        let mut order: Vec<usize> = (0..stages.len()).collect();
        order.sort_by_key(|&k| places.iter().filter(|&&m| fits(m, stages[k])).count());
        let mut taken = vec![false; places.len()];
        let mut out = Vec::new();
        for k in order {
            match (0..places.len()).find(|&p| !taken[p] && fits(places[p], stages[k])) {
                Some(p) => taken[p] = true,
                None => out.push(k),
            }
        }
        let mut halves = self.couples.clone();
        let mut pairs = pairs;
        out.retain(|&k| {
            let half = halves.iter().position(|&m| fits(m, stages[k]));
            match half.filter(|_| pairs > 0) {
                Some(h) => {
                    halves.swap_remove(h);
                    pairs -= 1;
                    false
                }
                None => true,
            }
        });
        out.sort_unstable();
        out
    }

    /// How many of `stages` have a place here, with `pairs` couples among them.
    pub(super) fn placed(&self, stages: &[usize], pairs: usize) -> usize {
        stages.len() - self.unplaced(stages, pairs).len()
    }

    /// The most of life stage `stage` who could sleep here: couples in every couple's bed.
    pub(super) fn most(&self, stage: usize) -> usize {
        let fits = |m: &&StageMask| crate::content::stage_in(**m, stage);
        self.places.iter().filter(fits).count() + self.couples.iter().filter(fits).count()
    }
}

/// The stages a bed of kind `def` is for (its night's sleep, content `stages`); None: no bed (no
/// night's sleep, or only a baby's; a spare bed's only if `spare` and it opens out into a bed for
/// the night, the sofa bed pulled out, not a night curled up on the sofa).
pub(super) fn bed_stages(
    content: &crate::content::Content,
    def: &crate::content::ObjectDef,
    spare: bool,
) -> Option<StageMask> {
    let sleep = content.day_rhythm.sleep_tags;
    def.interactions
        .iter()
        .filter(|it| {
            it.tags & sleep != 0 && !it.baby && (!it.spare || (spare && it.leaves == Some(true)))
        })
        .map(|it| it.stages)
        .reduce(|a, b| a | b)
}

pub(super) fn beds(w: &World, plot: u32) -> Beds {
    let mut beds = Beds {
        places: Vec::new(),
        couples: Vec::new(),
    };
    for (o, _) in w
        .objects
        .iter()
        .zip(&w.object_plot)
        .filter(|(_, p)| **p == Some(plot))
    {
        let def = &w.content.objects[o.def];
        let Some(stages) = bed_stages(&w.content, def, true) else {
            continue;
        };
        if crate::privacy::couples_bed(def) {
            beds.places.push(stages);
            beds.couples
                .extend(std::iter::repeat_n(stages, usize::from(def.slots) - 1));
        } else {
            beds.places
                .extend(std::iter::repeat_n(stages, usize::from(def.slots)));
        }
    }
    beds
}

/// The life stage of `sims`, each.
pub(super) fn stages_of(w: &World, sims: &[usize]) -> Vec<usize> {
    sims.iter()
        .map(|&i| w.content.life.stage(w.sims[i].age))
        .collect()
}

/// The first grown-up life stage (whoever moves in is one).
pub(super) fn grown_stage(w: &World) -> usize {
    w.content
        .life
        .stages
        .iter()
        .position(|s| s.adult)
        .unwrap_or(0)
}

/// Couples among `sims` (both of them in it).
pub(super) fn pairs(w: &World, sims: &[usize]) -> usize {
    sims.iter()
        .filter(|&&a| {
            w.relationships
                .partner_of(a)
                .is_some_and(|b| b > a && sims.contains(&b))
        })
        .count()
}

/// Whether household `h`'s home has a bed and room for one more resident, the partner of one of
/// them moving in.
pub(super) fn has_room_for_partner(w: &World, h: usize) -> bool {
    let Some(plot) = w.households[h].plot else {
        return false;
    };
    let members = members(w, h);
    let sleepers = sleepers(w, h);
    let mut stages = stages_of(w, &sleepers);
    stages.push(grown_stage(w));
    members.len() < w.content.rules.max_household
        && beds(w, plot)
            .unplaced(&stages, pairs(w, &sleepers) + 1)
            .is_empty()
}

/// Whether a resident's household lets them move on their own (the player can say no). Being
/// at work doesn't matter: they come back to the new home. Out on a visit or an outing, not
/// that day (moved, they were taken from the host's home ten minutes in), nor out on a trip by
/// car, there or back (`cars::on_trip`: moved at the wheel, they got out where the car stood, and
/// it waited in the road for them for good). Nor the last grown-up at home with a child: the
/// child would be left on their own (no sitter booked, the rent run up with nobody earning).
pub(super) fn may_move(w: &World, i: usize) -> bool {
    let s = &w.sims[i];
    let adult = |j: &usize| w.sims[*j].adult(&w.content);
    s.here()
        && s.visiting.is_none()
        && !crate::cars::on_trip(&w.cars, s)
        && s.adult(&w.content)
        && (w.player_moves || !w.households[s.household as usize].player)
        && {
            let home = members(w, s.household as usize);
            home.iter().all(adult) || home.iter().any(|&j| j != i && adult(&j))
        }
}

/// A home nobody lives in (a house or a flat; not a park, not the player's home), with beds for
/// `n`: one, or two partners.
pub(super) fn vacant_plot(w: &World, n: usize) -> Option<u32> {
    w.plots
        .iter()
        .filter(|p| w.is_home(p.id))
        .filter(|p| {
            !w.households
                .iter()
                .enumerate()
                .any(|(h, hh)| hh.plot == Some(p.id) && (hh.player || !members(w, h).is_empty()))
        })
        .find(|p| beds(w, p.id).placed(&vec![grown_stage(w); n], n / 2) >= n)
        .map(|p| p.id)
}

/// A household for the house on `plot`: the empty one already there, else a new one. Either way
/// it starts afresh: nothing of the family who lived there before is the new one's (their baby on
/// the way, letters, washing, parcels to set up), and the story names those of them who left as
/// the family they were (`World::retire_slot`), not by the new family's name.
pub(super) fn household_for(w: &mut World, plot: u32, name: &str) -> usize {
    let h = w
        .households
        .iter()
        .position(|hh| hh.plot == Some(plot))
        .unwrap_or(w.households.len());
    let fresh = Household {
        id: h as u32,
        name: name.to_string(),
        plot: Some(plot),
        player: false,
        funds: 0,
        free_will: true,
        routines: Vec::new(),
        expecting: None,
        sitter_day: None,
        laundry: crate::world::LAUNDRY_START,
        letters: Vec::new(),
    };
    if h == w.households.len() {
        w.households.push(fresh);
        return h;
    }
    for j in 0..w.sims.len() {
        if !w.sims[j].here() && w.sims[j].household as usize == h {
            w.retire_slot(j);
        }
    }
    w.households[h] = fresh;
    w.setups.retain(|s| s.household as usize != h);
    w.items.version = w.items.version.wrapping_add(1);
    h
}

/// Moves resident `i` into household `to`, with their share of their old household's money.
pub(super) fn move_to(w: &mut World, i: usize, to: usize) {
    let from = w.sims[i].household as usize;
    if from == to {
        return;
    }
    let share = w.households[from].funds.max(0) / members(w, from).len().max(1) as i64;
    w.households[from].funds -= share;
    w.households[to].funds += share;
    let id = i as u32;
    for o in &mut w.objects {
        o.release(id);
    }
    // (A car their job called for that the old household hasn't bought yet isn't bought.)
    crate::cars::job_lost(&mut w.cars, id);
    // Their baby on the way comes with them, born in their new home.
    if w.households[to].expecting.is_none()
        && w.households[from]
            .expecting
            .is_some_and(|e| e.parents.contains(&id))
    {
        w.households[to].expecting = w.households[from].expecting.take();
    }
    let arrive = w.households[to]
        .plot
        .map(|p| w.plots[p as usize].home_point());
    // (The household index no longer matches the residents: `World::household_sims`.)
    w.members.moved = true;
    let sim = &mut w.sims[i];
    sim.household = to as u32;
    sim.clear_activity();
    sim.planner.run = None;
    // At work they come back to the new home; otherwise they're there now.
    if let Some(pos) = arrive.filter(|_| sim.away_until.is_none()) {
        sim.pos = pos;
    }
    w.structure_version = w.structure_version.wrapping_add(1);
}

/// Partners move in together; grown children move out (or away).
pub(super) fn moving(w: &mut World, rules: &crate::content::MovingRules) {
    let tick = w.tick;
    let n = w.sims.len();
    // Partners in different homes.
    for a in 0..n {
        for b in a + 1..n {
            let (ra, rb) = (*w.relationships.get(a, b), *w.relationships.get(b, a));
            if !ra.partners
                || !may_move(w, a)
                || !may_move(w, b)
                || w.sims[a].household == w.sims[b].household
            {
                continue;
            }
            if ra.romance.min(rb.romance) < rules.romance || w.rng.next_f32() >= rules.partners {
                continue;
            }
            let (ha, hb) = (w.sims[a].household as usize, w.sims[b].household as usize);
            let into_b = has_room_for_partner(w, hb);
            let into_a = has_room_for_partner(w, ha);
            // The player's home first, else the one from the fuller home moves.
            let mover = match (into_a, into_b) {
                (true, true) if w.households[ha].player => Some(b),
                (true, true) if w.households[hb].player => Some(a),
                (true, true) => Some(if members(w, ha).len() >= members(w, hb).len() {
                    a
                } else {
                    b
                }),
                (true, false) => Some(b),
                (false, true) => Some(a),
                (false, false) => None,
            };
            if let Some(m) = mover {
                let partner = if m == a { b } else { a };
                let to = w.sims[partner].household as usize;
                move_to(w, m, to);
                w.events
                    .push(tick, EventKind::MovedInWith, m, partner, None);
            } else if let Some(plot) = vacant_plot(w, 2) {
                let name = w.households[ha].name.clone();
                let h = household_for(w, plot, &name);
                move_to(w, a, h);
                move_to(w, b, h);
                w.events.push(tick, EventKind::MovedInWith, a, b, None);
            }
        }
    }
    // Grown children living with a parent.
    for i in 0..n {
        let (age, h, working) = {
            let s = &w.sims[i];
            (s.age, s.household as usize, s.job.is_some() || s.retired)
        };
        if !may_move(w, i) || age < rules.leave_home_age || !working {
            continue;
        }
        let with_parent = (0..n).any(|j| {
            w.sims[j].here()
                && w.sims[j].household as usize == h
                && w.relationships.kin(i, j) == Kin::Parent
        });
        if !with_parent || w.rng.next_f32() >= rules.leave_home {
            continue;
        }
        let partner = (0..n).find(|&j| {
            w.sims[j].here()
                && w.sims[j].household as usize == h
                && w.relationships.get(i, j).partners
        });
        let going = 1 + usize::from(partner.is_some());
        if let Some(plot) = vacant_plot(w, going) {
            let name = w.households[h].name.clone();
            let new = household_for(w, plot, &name);
            move_to(w, i, new);
            if let Some(p) = partner {
                move_to(w, p, new);
            }
            w.events
                .push_detail(tick, EventKind::MovedOut, i, None, None);
        } else if age >= rules.leave_town_age
            && !w.households[h].player
            && w.rng.next_f32() < rules.leave_town
        {
            w.events
                .push_detail(tick, EventKind::MovedAway, i, None, None);
            w.depart(i, GoneWhy::MovedAway);
            if let Some(p) = partner {
                w.events
                    .push_detail(tick, EventKind::MovedAway, p, None, None);
                w.depart(p, GoneWhy::MovedAway);
            }
        }
    }
}
