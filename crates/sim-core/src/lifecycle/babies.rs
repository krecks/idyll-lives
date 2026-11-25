//! Beds for babies and residents (content `life.stages`).

use super::*;

// ---- Babies --------------------------------------------------------------------------------

/// Every baby of household `h` needs a crib at home: one is delivered for each baby without a
/// free one (story: a crib arrived). Babies with no room for a crib stay with their parents.
pub(crate) fn cribs_for_babies(w: &mut World, h: usize) {
    let Some(plot) = w.households[h].plot else {
        return;
    };
    let Some(crib) = w
        .content
        .objects
        .iter()
        .position(|d| d.price.is_some() && d.interactions.iter().any(|it| it.baby_bed()))
    else {
        return;
    };
    let babies = members(w, h)
        .into_iter()
        .filter(|&i| w.content.life.baby(w.sims[i].age))
        .count();
    let cribs = w
        .objects
        .iter()
        .zip(&w.object_plot)
        .filter(|(o, p)| {
            **p == Some(plot)
                && w.content.objects[o.def]
                    .interactions
                    .iter()
                    .any(|it| it.baby_bed())
        })
        .count();
    for _ in cribs..babies {
        if w.deliver(h as u32, crib, 0).is_none() {
            break;
        }
    }
}

/// The player's household starts in a home with fewer bed places than people past the crib (a
/// bed their age may sleep in: the race car bed is no teenager's): tell them once, for each
/// resident without a bed (on the game's first step, so it's news to the view, not the story so
/// far). The simulation's fallback is unchanged: they sleep on the sofa (`Interaction::spare`),
/// share a bed (only a partner, in a couple's bed, or a child with a parent whose partner is out)
/// or sleep on the floor until the player buys one.
pub(crate) fn notify_missing_beds(w: &mut World, h: usize) {
    let sleep = w.content.day_rhythm.sleep_tags;
    if sleep == 0 || !w.households[h].player {
        return;
    }
    let Some(plot) = w.households[h].plot else {
        return;
    };
    let sleepers = sleepers(w, h);
    let stages = stages_of(w, &sleepers);
    for k in beds(w, plot).unplaced(&stages, pairs(w, &sleepers)) {
        let i = sleepers[k];
        w.events.push(w.tick, EventKind::NoBed, i, i, None);
    }
}

/// Resident `i` is out of the crib, came to live here past it, or grew into a life stage (their
/// bed may not be for it: the race car bed is a child's): with fewer bed places at home than
/// people past the crib (`Beds::unplaced`), a household the player doesn't play buys one: the
/// cheapest bed for the stage of whoever has none and every stage after it, at the first good free spot, like `cribs_for_babies`, if
/// that leaves a week's rent and bills, like the planner's purchases; a bed nobody at home may
/// sleep in any more is sold, to make room for it where it stood. The player's household is told,
/// if it's `i` who has none (`EventKind::NoBed`), and buys its own.
pub(crate) fn bed_for(w: &mut World, i: usize) {
    let h = w.sims[i].household as usize;
    let sleep = w.content.day_rhythm.sleep_tags;
    let Some(plot) = w.households[h].plot.filter(|_| sleep != 0) else {
        return;
    };
    // (Them last: of those with as many places to choose from, they're the one without.)
    let mut sleepers = sleepers(w, h);
    sleepers.retain(|&j| j != i);
    sleepers.push(i);
    let stages = stages_of(w, &sleepers);
    let short = beds(w, plot).unplaced(&stages, pairs(w, &sleepers));
    let Some(&k) = short.last() else {
        return;
    };
    let who = sleepers[k];
    if w.households[h].player {
        if who == i {
            w.events.push(w.tick, EventKind::NoBed, i, i, None);
        }
        return;
    }
    let stage = w.content.life.stage(w.sims[who].age);
    let later =
        (stage..w.content.life.stages.len().max(stage + 1)).fold(0, |m: StageMask, s| m | 1 << s);
    let bed = (0..w.content.objects.len())
        .filter(|&d| {
            bed_stages(&w.content, &w.content.objects[d], false).is_some_and(|m| m & later == later)
        })
        .filter_map(|d| w.content.objects[d].price.map(|p| (p, d)))
        .min();
    let reserve = crate::life::weekly_costs(w, h).map_or(0, |(rent, bills)| rent + bills);
    let Some((price, bed)) = bed else {
        return;
    };
    if w.households[h].funds - w.build_price(price) < reserve {
        return;
    }
    // Beds at home nobody of the household may sleep in now (outgrown: whoever lies in one
    // tonight gets up, and goes to bed again in the new one).
    let outgrown: Vec<u32> = w
        .objects
        .iter()
        .zip(&w.object_plot)
        .filter(|(_, p)| **p == Some(plot))
        .filter(|(o, _)| {
            bed_stages(&w.content, &w.content.objects[o.def], false)
                .is_some_and(|m| !stages.iter().any(|&s| crate::content::stage_in(m, s)))
        })
        .map(|(o, _)| o.id)
        .collect();
    // (Highest id first: selling one moves the ids after it.)
    for &o in outgrown.iter().rev() {
        let _ = w.sell(h as u32, o);
    }
    if w.deliver(h as u32, bed, price).is_some() {
        w.pay(h, price);
    }
}
