//! Buying what a goal asks for, and the resident's answer.
//!
//! Split from `planner.rs` (see `mod.rs`).

use super::*;

/// Buys `object` for resident `i`'s home: over the cheaper one it replaces, if there is one, in
/// the place that one stood (their old bed goes, sold, and the new one stands where it stood, so
/// the room they know stays as it was). Only where they can't be swapped there does the old one
/// go and the new one find a free spot somewhere else.
pub(crate) fn buy_wanted(w: &mut World, i: usize, object: usize) -> Result<u32, Error> {
    let bought = place_wanted(w, w.sims[i].household, object, None, true)?;
    w.events
        .push_object(w.tick, EventKind::Bought, i, object, None);
    w.sims[i].planner.recheck = true;
    Ok(bought)
}

/// Buys `object` for household `h`'s home as `buy_wanted` does: over the cheaper one it
/// replaces, else at `at` (as for `World::buy_at`) if it fits there, else at a free spot.
/// Paid for now (`charge`), or already (a parcel set up). Returns the new object's id.
pub(crate) fn place_wanted(
    w: &mut World,
    h: u32,
    object: usize,
    at: Option<crate::setup::Spot>,
    charge: bool,
) -> Result<u32, Error> {
    let id = w.content.objects[object].id.clone();
    let old = replaces(w, h as usize, object);
    // Their own thing goes first, and where it stood: not a new bed into some free corner.
    match old.and_then(|o| swap_in(w, h, o, &id, charge)) {
        Some(new) => Ok(new),
        None => at_a_free_spot(w, h, object, &id, at, charge),
    }
}

/// Swaps `old` out for `id`: `old` is sold and `id` bought over the tiles it stood on, so the new
/// thing stands in the old one's place. If it won't go there, it's put anywhere now free; if even
/// that doesn't fit, `old` goes back as it was. Only for something nothing stands on and nobody
/// uses, standing on their own plot: not one a flat keeps outside it (its letterbox by the
/// building's door, `ObjectInstance::keeper`), whose place is not the flat's to buy on nor to put
/// it back on (`replace_at_home`), so it would be sold and lost.
fn swap_in(w: &mut World, h: u32, old: u32, id: &str, charge: bool) -> Option<u32> {
    let o = w.objects.get(old as usize)?.clone();
    if o.keeper.is_some()
        || o.users.iter().any(Option::is_some)
        || !crate::surface::riders(&w.content, &w.objects, old).is_empty()
    {
        return None;
    }
    let (hh, plot) = w.home_of(h).ok()?;
    let before = w.households[hh].funds;
    w.sell(h, old).ok()?;
    let refund = w.households[hh].funds - before;
    let buy = |w: &mut World, at| {
        w.buy_charging(h, id, at, Default::default(), Default::default(), charge)
    };
    let placed = buy(w, Some([o.x, o.z, o.rot as i32])).or_else(|_| buy(w, None));
    if let Ok(new) = placed {
        return Some(new);
    }
    // Put it back (and the money).
    let _ = w.replace_at_home(plot, &o, o.x, o.z, o.rot, Layout::of(&o));
    w.households[hh].funds -= refund;
    None
}

/// The new one goes where there's room rather than over their old one: buys `id` at `at` if it
/// fits there, else at any free spot, and then sells what it replaces, if the household has one.
/// Returns the new object's id (the old one going shifts the ids after it down). Called when
/// swapping them in their own place can't be done, and when there's nothing at home to take the
/// place of.
fn at_a_free_spot(
    w: &mut World,
    h: u32,
    object: usize,
    id: &str,
    at: Option<crate::setup::Spot>,
    charge: bool,
) -> Result<u32, Error> {
    let old = replaces(w, h as usize, object);
    let buy =
        |w: &mut World, at, layout| w.buy_charging(h, id, at, Default::default(), layout, charge);
    let new = match at {
        Some((spot, layout)) => {
            buy(w, Some(spot), layout).or_else(|_| buy(w, None, Default::default()))
        }
        None => buy(w, None, Default::default()),
    }?;
    // The old one goes too (ids after it shift down).
    match old {
        Some(o) if w.sell(h, o).is_ok() && o < new => Ok(new - 1),
        _ => Ok(new),
    }
}

/// The player says yes to what resident `i` asked for: it's bought, and comes as a parcel to the
/// front door for them to set up (`setup.rs`; their goal is reached once it stands at home).
pub(crate) fn accept_purchase(w: &mut World, i: usize) -> Result<(), Error> {
    let Some(request) = w.sims[i].planner.purchase.filter(|p| p.open) else {
        return Err(Error::new(format!(
            "{} isn't asking for anything",
            w.sims[i].name
        )));
    };
    let object = request.object;
    let h = w.sims[i].household;
    let (hh, _) = w.home_of(h)?;
    let def = &w.content.objects[object];
    let (name, price) = (def.name.clone(), def.price.unwrap_or(0));
    w.can_pay(hh, price, || format!("not enough money for the {name}"))?;
    if !crate::setup::room_for(w, h, object) {
        return Err(Error::new("there's no free spot for that at home"));
    }
    let paid = w.build_price(price);
    w.pay(hh, price);
    w.setups
        .push(crate::setup::Setup::new(h, i as u32, object, w.tick, paid));
    // (The parcel at the door at once, even with the game paused.)
    crate::setup::refresh_parcels(w);
    w.events
        .push_object(w.tick, EventKind::Bought, i, object, None);
    let p = &mut w.sims[i].planner;
    p.purchase = None;
    // (Asked and answered: no more wishing for it over their head.)
    if p.thought.is_some_and(|t| t.kind == ThoughtKind::WantsToBuy) {
        p.thought = None;
    }
    Ok(())
}

/// The player says no: they'll ask again in a few days, and from the second no for the same
/// thing on, they're cross about it.
pub(crate) fn decline_purchase(w: &mut World, i: usize) -> Result<(), Error> {
    let day = clock::day(w.tick);
    let tick = w.tick;
    let content = &w.content;
    let sim = &mut w.sims[i];
    let Some(request) = sim.planner.purchase.as_mut().filter(|p| p.open) else {
        return Err(Error::new(format!(
            "{} isn't asking for anything",
            sim.name
        )));
    };
    request.open = false;
    request.day = day;
    request.declines = request.declines.saturating_add(1);
    let (object, declines) = (request.object, request.declines);
    let denied = &content.planner.denied_feelings;
    if declines >= 2
        && let Some(&f) = denied.get((declines as usize - 2).min(denied.len().saturating_sub(1)))
    {
        social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        w.events.push_object(
            tick,
            EventKind::DeniedAgain,
            i,
            object,
            Some(declines as i64),
        );
    }
    Ok(())
}

// ---- How plans and goals steer choices (used by `World::pick_autonomous`) ----------------
