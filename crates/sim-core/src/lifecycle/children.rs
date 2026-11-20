//! Conceiving, births and adoption (content `life.pregnancy`, `life.adoption`).
//!
//! Split from `lifecycle.rs` (see `mod.rs`).

use super::*;

// ---- Having children -----------------------------------------------------------------------

/// Partners `a` and `b` tried for a baby and it went well: with content `pregnancy.chance`, one
/// is on the way, if they live together, are grown-ups young enough, there's room for one more
/// and none is on the way already.
pub(crate) fn conceive(w: &mut World, a: usize, b: usize) {
    let Some(rules) = w.content.life.pregnancy else {
        return;
    };
    let (sa, sb) = (&w.sims[a], &w.sims[b]);
    let h = sa.household as usize;
    let fits = |s: &crate::world::Sim| s.here() && s.adult(&w.content) && s.age <= rules.max_age;
    if !fits(sa) || !fits(sb) || sb.household as usize != h || !w.relationships.get(a, b).partners {
        return;
    }
    if w.households[h].expecting.is_some()
        || !room_for_one(w, h)
        || w.rng.next_f32() >= rules.chance
    {
        return;
    }
    let due = clock::day(w.tick) + rules.days;
    w.households[h].expecting = Some(crate::world::Expecting {
        parents: [a as u32, b as u32],
        due,
    });
    w.events.push(w.tick, EventKind::Expecting, a, b, None);
}

/// Room in household `h` for one more (and in the town, `World::room_in_town`).
pub(crate) fn room_for_one(w: &World, h: usize) -> bool {
    members(w, h).len() < w.content.rules.max_household && w.room_in_town()
}

/// At midnight: babies who are due are born (returned).
pub(crate) fn births(w: &mut World) -> Vec<usize> {
    let today = clock::day(w.tick);
    let mut born = Vec::new();
    for h in 0..w.households.len() {
        let Some(e) = w.households[h].expecting.filter(|e| e.due <= today) else {
            continue;
        };
        w.households[h].expecting = None;
        let parents: Vec<usize> = e
            .parents
            .iter()
            .map(|&p| p as usize)
            .filter(|&p| w.sims[p].here())
            .collect();
        if !members(w, h).iter().any(|&i| w.sims[i].adult(&w.content)) || !room_for_one(w, h) {
            continue;
        }
        if let Some(baby) = add_child(w, h, 0.0, &parents, true) {
            w.events.push(
                w.tick,
                EventKind::Born,
                baby,
                parents.first().copied().unwrap_or(baby),
                parents.get(1).copied(),
            );
            born.push(baby);
        }
    }
    born
}

/// Adopts a baby (or a child) into household `h`, for content `adoption` (free in Creative).
pub fn adopt(w: &mut World, household: u32, child: bool) -> Result<u32, crate::Error> {
    use crate::Error;
    let cost = w
        .content
        .life
        .adoption_cost
        .ok_or_else(|| Error::new("adoption isn't possible here"))?;
    let (h, _) = w.home_of(household)?;
    let adults: Vec<usize> = members(w, h)
        .into_iter()
        .filter(|&i| w.sims[i].adult(&w.content))
        .collect();
    if adults.is_empty() {
        return Err(Error::new("it takes a grown-up to adopt"));
    }
    if !room_for_one(w, h) {
        return Err(Error::new("there's no room for another"));
    }
    w.can_pay(h, cost, || "not enough money to adopt".into())?;
    // The parents: a couple if there is one, else the first grown-up.
    let parents: Vec<usize> = adults
        .iter()
        .find_map(|&a| {
            adults
                .iter()
                .find(|&&b| b != a && w.relationships.get(a, b).partners)
                .map(|&b| vec![a, b])
        })
        .unwrap_or_else(|| vec![adults[0]]);
    let age = if child {
        3.0 + (w.rng.next_u32() % 8) as f32
    } else {
        0.0
    };
    let id = add_child(w, h, age, &parents, false)
        .ok_or_else(|| Error::new("there's no room for another"))?;
    w.pay(h, cost);
    w.events.push(
        w.tick,
        EventKind::Adopted,
        id,
        parents[0],
        parents.get(1).copied(),
    );
    if !w.content.life.baby(age) {
        bed_for(w, id);
    }
    Ok(id as u32)
}

/// A child of `parents` (at `age`) joins household `h`: a name for a random gender, the
/// household's name, random traits, a look (from the parents if `born` to them: an appearance
/// seed with them; adopted children look like themselves), family links to the parents and
/// their other children, and a crib for a baby.
pub(crate) fn add_child(
    w: &mut World,
    h: usize,
    age: f32,
    parents: &[usize],
    born: bool,
) -> Option<usize> {
    let content = &w.content;
    let gender = pick(&mut w.rng, &content.genders).map(|g| g.id.clone())?;
    let firsts = content
        .names
        .by_gender
        .get(&gender)
        .filter(|l| !l.is_empty())
        .unwrap_or(&content.names.first);
    let taken: Vec<&str> = w
        .sims
        .iter()
        .filter(|s| s.here() && s.household as usize == h)
        .map(|s| s.name.as_str())
        .collect();
    let free: Vec<&String> = firsts
        .iter()
        .filter(|n| !taken.contains(&n.as_str()))
        .collect();
    let name = pick(&mut w.rng, &free)
        .map(|n| (*n).clone())
        .unwrap_or_else(|| "Baby".into());
    // Beside the first parent (who may be on the bed, the sofa: somewhere nobody stands up from
    // but the one using it), else at the door.
    let beside = |pos: [f32; 2]| {
        let nav = w.nav();
        if nav.tile_free(pos[0].floor() as i32, pos[1].floor() as i32) {
            return Some(pos);
        }
        crate::conversation::approach_tile(&nav, pos, pos)
            .map(|(x, z)| [x as f32 + 0.5, z as f32 + 0.5])
    };
    let at = parents
        .first()
        .and_then(|&p| beside(w.sims[p].pos))
        .or_else(|| {
            w.households[h]
                .plot
                .map(|p| w.plots[p as usize].arrival_point())
        })?;
    let (traits, perks) = random_character(w);
    let seed = w.rng.next_u32();
    let spawn = serde_json::json!({
        "name": name, "gender": gender, "traits": traits, "perks": perks, "age": age, "household": h,
        "appearance": if born { serde_json::json!({"seed": seed, "parents": parents}) } else { serde_json::json!({"seed": seed}) },
        "x": at[0], "z": at[1],
    });
    let spawn: crate::lot::SimSpawn = serde_json::from_value(spawn).ok()?;
    let id = w.spawn_at(&spawn, true).ok()? as usize;
    // Family: the parents, and their other children are brothers and sisters.
    let mut bonds = Vec::new();
    if w.content.bond_presets.contains_key("parent") {
        for &p in parents {
            bonds.push(crate::lot::BondRaw {
                a: id,
                b: p,
                preset: "parent".into(),
            });
        }
    }
    if w.content.bond_presets.contains_key("siblings") {
        for j in 0..w.sims.len() {
            let child_of_ours = parents
                .iter()
                .any(|&p| w.relationships.kin(j, p) == Kin::Parent);
            if j != id && w.sims[j].here() && child_of_ours {
                bonds.push(crate::lot::BondRaw {
                    a: id,
                    b: j,
                    preset: "siblings".into(),
                });
            }
        }
    }
    let _ = w.init_relationships_for(&[id], &bonds);
    cribs_for_babies(w, h);
    w.structure_version = w.structure_version.wrapping_add(1);
    Some(id)
}

/// Now and then a neighbour household of grown-ups with no children, room and the money for it
/// adopts (content `adoption.neighbours`, a daily chance).
pub(crate) fn neighbours_adopt(w: &mut World) {
    let (Some(cost), chance) = (w.content.life.adoption_cost, w.content.life.adoption_chance)
    else {
        return;
    };
    let max_age = w.content.life.pregnancy.map_or(55.0, |p| p.max_age + 5.0);
    for h in 0..w.households.len() {
        let hh = &w.households[h];
        if hh.player || !hh.free_will || hh.plot.is_none() || hh.funds < cost * 3 {
            continue;
        }
        let people = members(w, h);
        let grown = people
            .iter()
            .filter(|&&i| w.sims[i].adult(&w.content))
            .count();
        let young_enough = people.iter().any(|&i| w.sims[i].age <= max_age);
        if grown == 0 || grown < people.len() || !young_enough || !room_for_one(w, h) {
            continue;
        }
        if w.rng.next_f32() < chance {
            let child = w.rng.next_f32() < 0.5;
            let _ = adopt(w, h as u32, child);
        }
    }
}
