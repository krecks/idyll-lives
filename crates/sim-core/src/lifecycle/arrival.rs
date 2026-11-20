//! Newcomers arriving in town (content `life.newcomers`).
//!
//! Split from `lifecycle.rs` (see `mod.rs`).

use super::*;

// ---- Newcomers -----------------------------------------------------------------------------

/// Vacant homes (houses and flats: furnished, nobody living there, not the player's) get a new
/// household now and then. They come in at the street door (a flat's building's).
pub(crate) fn newcomers(w: &mut World, rules: &crate::content::NewcomerRules) {
    for p in 0..w.plots.len() as u32 {
        if !w.is_home(p)
            || beds(w, p).most(grown_stage(w)) == 0
            || w.households
                .iter()
                .enumerate()
                .any(|(h, hh)| hh.plot == Some(p) && (hh.player || !members(w, h).is_empty()))
        {
            continue;
        }
        if w.rng.next_f32() < 1.0 / rules.days {
            arrive(w, p);
        }
    }
}

pub(crate) fn pick<'a, T>(rng: &mut crate::rng::Rng, list: &'a [T]) -> Option<&'a T> {
    (!list.is_empty()).then(|| &list[(rng.next_u32() as usize) % list.len()])
}

/// A new household moves into the house on `plot`: a single, a couple (sometimes with a grown
/// child) or siblings, sized to its beds (only a couple shares a couple's bed), with names,
/// traits, perks and ages from content and an appearance seed (`{"seed": n}`, expanded into a
/// look by the web). Never strangers: only those who belong together share a bedroom
/// (`privacy::room_claimants`).
pub fn arrive(w: &mut World, plot: u32) -> bool {
    let content = &w.content;
    let beds = beds(w, plot);
    let grown = grown_stage(w);
    let most = beds.most(grown).min(content.rules.max_household).max(1);
    let here = w.sims.iter().filter(|s| s.here()).count();
    let cap = w.residents_most();
    if here >= cap || content.genders.is_empty() {
        return false;
    }
    let size = (1 + (w.rng.next_u32() as usize) % most).min(cap - here);
    let couple = size >= 2 && w.rng.next_f32() < 0.6;
    let size = size
        .min(beds.placed(&vec![grown; size], usize::from(couple)))
        .max(1);
    // Everyone who moves in is grown up (the first stages are a baby's and a child's).
    let adult = content
        .life
        .stages
        .iter()
        .find(|s| s.adult)
        .map_or(18.0, |s| s.from);
    let [lo, hi] = content.life.start_age;
    let mut spawns = Vec::new();
    let mut bonds = Vec::new();
    let mut taken: Vec<String> = Vec::new();
    // A family name no household in town has (any, once every one is taken).
    let used: Vec<&str> = w
        .sims
        .iter()
        .filter(|s| s.here())
        .filter_map(|s| w.households.get(s.household as usize))
        .map(|h| h.name.as_str())
        .collect();
    let free: Vec<&String> = content
        .names
        .last
        .iter()
        .filter(|n| !used.contains(&n.as_str()))
        .collect();
    let surname = match pick(&mut w.rng, &free) {
        Some(n) => (*n).clone(),
        None => pick(&mut w.rng, &content.names.last)
            .cloned()
            .unwrap_or_else(|| "Newcomer".into()),
    };
    let at = w.plots[plot as usize].arrival_point();
    for k in 0..size {
        let content = &w.content;
        let gender = pick(&mut w.rng, &content.genders)
            .map(|g| g.id.clone())
            .unwrap_or_default();
        let firsts = content
            .names
            .by_gender
            .get(&gender)
            .filter(|l| !l.is_empty())
            .unwrap_or(&content.names.first);
        let free: Vec<&String> = firsts.iter().filter(|n| !taken.contains(n)).collect();
        let name = pick(&mut w.rng, &free)
            .map(|n| (*n).clone())
            .unwrap_or_else(|| format!("Newcomer {}", k + 1));
        taken.push(name.clone());
        // Partners close in age; a grown child twenty to thirty years younger.
        let age = match k {
            0 => lo + w.rng.next_f32() * (hi - lo),
            1 if couple => spawns_age(&spawns, 0) + w.rng.range(-4.0, 4.0),
            2 if couple => (spawns_age(&spawns, 0).min(spawns_age(&spawns, 1))
                - w.rng.range(20.0, 30.0))
            .max(adult),
            _ => spawns_age(&spawns, 0) + w.rng.range(-6.0, 6.0),
        }
        .max(adult)
        .floor();
        let attracted: Vec<String> = if couple && k < 2 {
            // The couple are attracted to each other (the second is picked to fit below).
            content.genders.iter().map(|g| g.id.clone()).collect()
        } else {
            content
                .genders
                .iter()
                .filter(|g| g.id != gender || w.rng.next_f32() < 0.15)
                .map(|g| g.id.clone())
                .collect()
        };
        let (traits, perks) = random_character(w);
        let seed = w.rng.next_u32();
        spawns.push(serde_json::json!({
            "name": name, "gender": gender, "attractedTo": attracted, "traits": traits, "perks": perks,
            "age": age, "appearance": {"seed": seed}, "x": at[0], "z": at[1],
        }));
    }
    if couple {
        bonds.push((0, 1, "partners"));
        if size >= 3 {
            bonds.push((2, 0, "parent"));
            bonds.push((2, 1, "parent"));
        }
    } else {
        for b in 1..size {
            for a in 0..b {
                bonds.push((a, b, "siblings"));
            }
        }
    }
    let h = household_for(w, plot, &surname);
    w.households[h].funds = w.content.starting_funds;
    w.households[h].free_will = true;
    let mut ids = Vec::new();
    for (k, spawn) in spawns.into_iter().enumerate() {
        let mut spawn: crate::lot::SimSpawn = match serde_json::from_value(spawn) {
            Ok(s) => s,
            Err(_) => return false,
        };
        spawn.household = h as u32;
        spawn.x += (k as f32 - 1.0) * 0.6;
        match w.spawn_at(&spawn, true) {
            Ok(id) => ids.push(id as usize),
            Err(_) => return false,
        }
    }
    let bonds: Vec<crate::lot::BondRaw> = bonds
        .into_iter()
        .filter(|(_, _, preset)| w.content.bond_presets.contains_key(*preset))
        .map(|(a, b, preset)| crate::lot::BondRaw {
            a: ids[a],
            b: ids[b],
            preset: preset.into(),
        })
        .collect();
    if w.init_relationships_for(&ids, &bonds).is_err() {
        return false;
    }
    w.structure_version = w.structure_version.wrapping_add(1);
    w.events
        .push_detail(w.tick, EventKind::MovedIn, ids[0], None, None);
    true
}

pub(crate) fn spawns_age(spawns: &[serde_json::Value], k: usize) -> f32 {
    spawns
        .get(k)
        .and_then(|s| s["age"].as_f64())
        .unwrap_or(30.0) as f32
}

/// Traits (no two that clash) and perks within the points, at random.
pub(crate) fn random_character(w: &mut World) -> (Vec<String>, Vec<String>) {
    let rules = w.content.rules;
    let span = rules.max_traits.saturating_sub(rules.min_traits) + 1;
    let target = rules.min_traits + (w.rng.next_u32() as usize) % span;
    let mut order: Vec<usize> = (0..w.content.traits.len()).collect();
    for i in (1..order.len()).rev() {
        order.swap(i, (w.rng.next_u32() as usize) % (i + 1));
    }
    let mut traits: Vec<String> = Vec::new();
    for t in order {
        if traits.len() >= target {
            break;
        }
        let def = &w.content.traits[t];
        let clash = traits.iter().any(|x| def.conflicts.contains(x))
            || w.content
                .traits
                .iter()
                .any(|d| traits.contains(&d.id) && d.conflicts.contains(&def.id));
        if !clash {
            traits.push(def.id.clone());
        }
    }
    let mut perks = Vec::new();
    let mut points = 0;
    for p in 0..w.content.perks.len() {
        let def = &w.content.perks[p];
        if points + def.cost <= rules.perk_points && w.rng.next_f32() < 0.4 {
            points += def.cost;
            perks.push(def.id.clone());
        }
    }
    (traits, perks)
}
