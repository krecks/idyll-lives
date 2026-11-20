//! Traits, perks and genders.

use super::*;

type People = (Vec<TraitDef>, Vec<PerkDef>, Vec<GenderDef>);

pub(super) fn parse(
    raw: &ContentFile,
    ix: &Indices,
    skill_array: &impl Fn(&HashMap<String, f32>, &str) -> Result<[f32; MAX_SKILLS], Error>,
) -> Result<People, Error> {
    let mut traits = Vec::with_capacity(raw.traits.len());
    for t in &raw.traits {
        traits.push(TraitDef {
            id: t.id.clone(),
            label: t.label.clone(),
            conflicts: t.conflicts.clone(),
            mods: build_modifiers(&t.effects, ix, &t.id)?,
            starting_skills: skill_array(&t.starting_skills, &t.id)?,
        });
    }
    let trait_ids: HashSet<&str> = traits.iter().map(|t| t.id.as_str()).collect();
    if trait_ids.len() != traits.len() {
        return Err(Error::new("duplicate trait id"));
    }
    for t in &traits {
        if let Some(c) = t.conflicts.iter().find(|c| !trait_ids.contains(c.as_str())) {
            return Err(Error::new(format!(
                "trait '{}' conflicts with unknown trait '{c}'",
                t.id
            )));
        }
    }

    let mut perks = Vec::with_capacity(raw.perks.len());
    for p in &raw.perks {
        perks.push(PerkDef {
            id: p.id.clone(),
            label: p.label.clone(),
            cost: p.cost,
            mods: build_modifiers(&p.effects, ix, &p.id)?,
        });
    }
    if perks
        .iter()
        .map(|p| p.id.as_str())
        .collect::<HashSet<_>>()
        .len()
        != perks.len()
    {
        return Err(Error::new("duplicate perk id"));
    }

    // (Who someone is attracted to is a bit per gender.)
    if raw.genders.len() > MAX_GENDERS {
        return Err(Error::new(format!("at most {MAX_GENDERS} genders")));
    }
    let genders: Vec<GenderDef> = raw
        .genders
        .iter()
        .map(|g| GenderDef {
            id: g.id.clone(),
            label: g.label.clone(),
        })
        .collect();
    Ok((traits, perks, genders))
}
