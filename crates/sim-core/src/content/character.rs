//! Traits, perks, genders and the character rules.

use super::*;

pub(super) fn each_tag(mask: TagMask) -> impl Iterator<Item = usize> {
    let mut bits = mask;
    std::iter::from_fn(move || {
        if bits == 0 {
            return None;
        }
        let i = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        Some(i)
    })
}

#[derive(Debug)]
pub struct TraitDef {
    pub id: String,
    pub label: String,
    pub conflicts: Vec<String>,
    pub mods: Modifiers,
    /// Skill levels a new Sim with this trait starts with.
    pub starting_skills: [f32; MAX_SKILLS],
}

#[derive(Debug)]
pub struct PerkDef {
    pub id: String,
    pub label: String,
    pub cost: u32,
    pub mods: Modifiers,
}

#[derive(Debug)]
pub struct GenderDef {
    pub id: String,
    pub label: String,
}

/// Limits for character creation.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CharacterRules {
    pub min_traits: usize,
    pub max_traits: usize,
    pub perk_points: u32,
    pub max_household: usize,
}

impl Default for CharacterRules {
    fn default() -> Self {
        Self {
            min_traits: 0,
            max_traits: 3,
            perk_points: 5,
            max_household: 4,
        }
    }
}
