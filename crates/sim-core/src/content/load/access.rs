//! Index lookups and character rules on a loaded `Content`.

use super::*;

impl Content {
    pub fn object_index(&self, id: &str) -> Option<usize> {
        self.object_index.get(id).copied()
    }

    /// How far past its footprint any object reaches over (content `overhang`, in tiles, along
    /// either axis however it's turned): for `ObjectInstance::may_reach`.
    pub(crate) fn overhang_reach(&self) -> i32 {
        self.objects
            .iter()
            .flat_map(|d| &d.overhang)
            .map(|&(r, f)| r.abs().max(f.abs()))
            .max()
            .unwrap_or(0)
    }

    pub fn recipe_index(&self, id: &str) -> Option<usize> {
        self.recipes.iter().position(|r| r.id == id)
    }

    pub fn item_index(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|i| i.id == id)
    }

    pub fn feeling_index(&self, id: &str) -> Option<usize> {
        self.feelings.iter().position(|m| m.id == id)
    }

    pub fn career_index(&self, id: &str) -> Option<usize> {
        self.careers.iter().position(|c| c.id == id)
    }

    pub fn skill_index(&self, id: &str) -> Option<usize> {
        self.skills.iter().position(|s| s.id == id)
    }

    /// Starting skill levels for a Sim with these traits (highest trait value per skill).
    pub fn starting_skills(&self, traits: &[String]) -> [f32; MAX_SKILLS] {
        let mut out = [0.0f32; MAX_SKILLS];
        for t in traits
            .iter()
            .filter_map(|id| self.traits.iter().find(|d| &d.id == id))
        {
            for (o, s) in out.iter_mut().zip(t.starting_skills) {
                *o = o.max(s);
            }
        }
        out
    }

    /// Combined autonomy modifiers of every schedule window active at `hour`.
    pub fn schedule_at(&self, hour: f32) -> Modifiers {
        let mut mods = Modifiers::default();
        for w in self.schedule.iter().filter(|w| w.contains(hour)) {
            mods.combine(&w.mods);
        }
        mods
    }

    pub fn social_index(&self, id: &str) -> Option<usize> {
        self.socials.iter().position(|s| s.id == id)
    }

    /// Validates gender and attraction ids. With no genders defined, any value is accepted.
    pub fn validate_identity(&self, gender: &str, attracted_to: &[String]) -> Result<(), Error> {
        if self.genders.is_empty() {
            return Ok(());
        }
        let known = |g: &str| self.genders.iter().any(|d| d.id == g);
        if !known(gender) {
            return Err(Error::new(format!("unknown gender '{gender}'")));
        }
        if let Some(g) = attracted_to.iter().find(|g| !known(g)) {
            return Err(Error::new(format!("unknown gender '{g}' in attraction")));
        }
        Ok(())
    }

    /// Whether a new character has as many traits as the rules ask. (Not asked of a saved one:
    /// it keeps the traits the content still knows, however few.)
    pub fn check_trait_count(&self, traits: &[String]) -> Result<(), Error> {
        let rules = self.rules;
        if traits.len() < rules.min_traits || traits.len() > rules.max_traits {
            return Err(Error::new(format!(
                "pick {}..={} traits (got {})",
                rules.min_traits,
                rules.max_traits,
                traits.len()
            )));
        }
        Ok(())
    }

    /// Validates a character's traits and perks against the rules (but their count,
    /// `check_trait_count`) and returns their combined effect.
    pub fn character_modifiers(
        &self,
        traits: &[String],
        perks: &[String],
    ) -> Result<Modifiers, Error> {
        let rules = self.rules;
        let mut mods = Modifiers::default();
        for (i, id) in traits.iter().enumerate() {
            if traits[..i].contains(id) {
                return Err(Error::new(format!("trait '{id}' picked twice")));
            }
            let def = self
                .traits
                .iter()
                .find(|t| &t.id == id)
                .ok_or_else(|| Error::new(format!("unknown trait '{id}'")))?;
            if let Some(c) = def.conflicts.iter().find(|c| traits.contains(c)) {
                return Err(Error::new(format!("traits '{id}' and '{c}' conflict")));
            }
            mods.combine(&def.mods);
        }
        let mut spent = 0;
        for (i, id) in perks.iter().enumerate() {
            if perks[..i].contains(id) {
                return Err(Error::new(format!("perk '{id}' picked twice")));
            }
            let def = self
                .perks
                .iter()
                .find(|p| &p.id == id)
                .ok_or_else(|| Error::new(format!("unknown perk '{id}'")))?;
            spent += def.cost;
            mods.combine(&def.mods);
        }
        if spent > rules.perk_points {
            return Err(Error::new(format!(
                "perks cost {spent} points, budget is {}",
                rules.perk_points
            )));
        }
        Ok(mods)
    }
}
