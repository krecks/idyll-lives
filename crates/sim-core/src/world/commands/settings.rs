//! The household's and the player's settings: speed, money, needs, skills, goals, suggestions,
//! the lamps and curtains.

use super::*;

impl World {
    /// Carries out `Command::SetSpeed`.
    pub(super) fn apply_set_speed(&mut self, speed: u8) -> Result<(), Error> {
        if speed > MAX_SPEED {
            return Err(Error::new(format!("speed must be 0..={MAX_SPEED}")));
        }
        // (The player's pick, super speed too: it stays when the quiet ends.)
        self.speed = speed;
        self.auto_super = false;
        if (1..SUPER_SPEED).contains(&self.speed) {
            self.resume_speed = self.speed;
        }
        Ok(())
    }

    /// Carries out `Command::SetAutoFast`.
    pub(super) fn apply_set_auto_fast(&mut self, enabled: bool) -> Result<(), Error> {
        // (Turned on while it's quiet: super speed starts at the next step.)
        if enabled && !self.auto_fast {
            self.was_calm = false;
        }
        self.auto_fast = enabled;
        Ok(())
    }

    /// Carries out `Command::AddFunds`.
    pub(super) fn apply_add_funds(&mut self, household: u32, amount: i64) -> Result<(), Error> {
        let h = self
            .households
            .get_mut(household as usize)
            .ok_or_else(|| Error::new(format!("unknown household {household}")))?;
        h.funds = (h.funds + amount).max(0);
        Ok(())
    }

    /// Carries out `Command::SetNeed`.
    pub(super) fn apply_set_need(
        &mut self,
        sim: u32,
        need: String,
        value: f32,
    ) -> Result<(), Error> {
        self.sim(sim)?;
        let n = self.content.needs.len();
        let which: Vec<usize> = if need == "all" {
            (0..n).collect()
        } else {
            vec![
                self.content
                    .needs
                    .iter()
                    .position(|d| d.id == need)
                    .ok_or_else(|| Error::new(format!("unknown need '{need}'")))?,
            ]
        };
        let value = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.5
        };
        let sim = &mut self.sims[sim as usize];
        for k in which {
            sim.needs[k] = value;
        }
        sim.planner.recheck = true;
        Ok(())
    }

    /// Carries out `Command::SetBond`.
    pub(super) fn apply_set_bond(
        &mut self,
        a: u32,
        b: u32,
        friendship: Option<f32>,
        romance: Option<f32>,
        chemistry: Option<f32>,
        partners: Option<bool>,
    ) -> Result<(), Error> {
        self.sim(a)?;
        self.sim(b)?;
        if a == b {
            return Err(Error::new("a bond is between two residents"));
        }
        let (a, b) = (a as usize, b as usize);
        // (Only those who may court: the game can't get anyone else there, so a check can't either.)
        let grown_ups = self.sims[a].adult(&self.content) && self.sims[b].adult(&self.content);
        if !conversation::may_court(grown_ups, self.relationships.get(a, b))
            && (romance.is_some_and(|f| f > 0.0) || partners == Some(true))
        {
            return Err(Error::new(
                "only grown-ups who aren't family can be partners or in love",
            ));
        }
        let r = &mut self.relationships;
        for (x, y) in [(a, b), (b, a)] {
            let rel = r.get_mut(x, y);
            rel.met = true;
            if let Some(f) = friendship.filter(|f| f.is_finite()) {
                rel.friendship = f.clamp(-100.0, 100.0);
            }
            if let Some(f) = romance.filter(|f| f.is_finite()) {
                rel.romance = f.clamp(-100.0, 100.0);
            }
        }
        if let Some(c) = chemistry.filter(|c| c.is_finite()) {
            r.set_chemistry(a, b, c);
        }
        if let Some(p) = partners {
            // (One partner each: a new one ends what either had.)
            if p {
                for x in [a, b] {
                    if let Some(old) = r.partner_of(x).filter(|&o| o != a && o != b) {
                        r.set_partners(x, old, false);
                    }
                }
            }
            r.set_partners(a, b, p);
        }
        self.sims[a].planner.recheck = true;
        self.sims[b].planner.recheck = true;
        Ok(())
    }

    /// Carries out `Command::SetSkill`.
    pub(super) fn apply_set_skill(
        &mut self,
        sim: u32,
        skill: String,
        level: f32,
    ) -> Result<(), Error> {
        self.sim(sim)?;
        let s = self
            .content
            .skill_index(&skill)
            .ok_or_else(|| Error::new(format!("unknown skill '{skill}'")))?;
        let level = if level.is_finite() {
            level.clamp(0.0, self.content.skill_rules.max_level)
        } else {
            0.0
        };
        let sim = &mut self.sims[sim as usize];
        sim.skills[s] = level;
        sim.planner.recheck = true;
        Ok(())
    }

    /// Carries out `Command::SetAutonomy`.
    pub(super) fn apply_set_autonomy(
        &mut self,
        enabled: bool,
        household: Option<u32>,
    ) -> Result<(), Error> {
        if let Some(h) = household {
            self.households
                .get_mut(h as usize)
                .ok_or_else(|| Error::new(format!("unknown household {h}")))?
                .free_will = enabled;
        } else {
            for h in self.households.iter_mut().filter(|h| h.player) {
                h.free_will = enabled;
            }
        }
        Ok(())
    }

    /// Carries out `Command::SetRoutines`.
    pub(super) fn apply_set_routines(
        &mut self,
        sim: u32,
        routines: Vec<crate::planner::RoutineIn>,
    ) -> Result<(), Error> {
        let routines = crate::planner::routines_from(&self.content, &routines)?;
        let s = self.sim_mut(sim)?;
        s.planner.routines = routines;
        // A changed block starts over (with today's plan).
        s.planner.run = None;
        Ok(())
    }

    /// Carries out `Command::SetHouseholdRoutines`.
    pub(super) fn apply_set_household_routines(
        &mut self,
        sim: u32,
        routines: Vec<crate::planner::RoutineIn>,
    ) -> Result<(), Error> {
        let routines = crate::planner::routines_from(&self.content, &routines)?;
        let h = self.sim(sim)?.household as usize;
        self.households[h].routines = routines;
        for s in self.sims.iter_mut().filter(|s| s.household as usize == h) {
            s.planner.run = None;
        }
        Ok(())
    }

    /// Carries out `Command::SkipHouseholdRoutine`.
    pub(super) fn apply_skip_household_routine(
        &mut self,
        sim: u32,
        routine: u16,
        skip: bool,
    ) -> Result<(), Error> {
        let skips = &mut self.sim_mut(sim)?.planner.skip_household;
        skips.retain(|&r| r != routine);
        if skip {
            skips.push(routine);
        }
        Ok(())
    }

    /// Carries out `Command::AddGoal`.
    pub(super) fn apply_add_goal(
        &mut self,
        sim: u32,
        goal: crate::planner::GoalIn,
    ) -> Result<(), Error> {
        let day = crate::clock::day(self.tick);
        let goal = crate::planner::goal_from(&self.content, &goal, self.sim(sim)?, day)?;
        if crate::planner::progress(self, sim as usize, &goal) >= 1.0 {
            return Err(Error::new(format!(
                "{} is there already",
                self.sims[sim as usize].name
            )));
        }
        let max = self.content.planner.max_goals;
        let s = self.sim_mut(sim)?;
        if s.planner.goals.len() >= max {
            return Err(Error::new(format!("{} has {max} goals already", s.name)));
        }
        if s.planner.goals.iter().any(|g| g.same(&goal)) {
            return Err(Error::new(format!("{} has that goal already", s.name)));
        }
        s.planner.goals.push(goal);
        Ok(())
    }

    /// Carries out `Command::RemoveGoal`.
    pub(super) fn apply_remove_goal(&mut self, sim: u32, index: usize) -> Result<(), Error> {
        let goals = &mut self.sim_mut(sim)?.planner.goals;
        if index < goals.len() {
            goals.remove(index);
        }
        Ok(())
    }

    /// Carries out `Command::MoveGoal`.
    pub(super) fn apply_move_goal(
        &mut self,
        sim: u32,
        index: usize,
        to: usize,
    ) -> Result<(), Error> {
        let goals = &mut self.sim_mut(sim)?.planner.goals;
        if index < goals.len() && to < goals.len() {
            let goal = goals.remove(index);
            goals.insert(to, goal);
        }
        Ok(())
    }

    /// Carries out `Command::AcceptSuggestion`.
    pub(super) fn apply_accept_suggestion(&mut self, sim: u32, index: usize) -> Result<(), Error> {
        let max = self.content.planner.max_goals;
        let day = crate::clock::day(self.tick);
        let s = self.sim_mut(sim)?;
        if index < s.planner.suggestions.len() {
            if s.planner.goals.len() >= max {
                return Err(Error::new(format!("{} has {max} goals already", s.name)));
            }
            let mut goal = s.planner.suggestions.remove(index);
            goal.since_day = day;
            // Already on the list (the player added it meanwhile): the idea just goes.
            if !s.planner.goals.iter().any(|g| g.same(&goal)) {
                s.planner.goals.push(goal);
            }
        }
        Ok(())
    }

    /// Carries out `Command::DismissSuggestion`.
    pub(super) fn apply_dismiss_suggestion(&mut self, sim: u32, index: usize) -> Result<(), Error> {
        let suggestions = &mut self.sim_mut(sim)?.planner.suggestions;
        if index < suggestions.len() {
            suggestions.remove(index);
        }
        Ok(())
    }

    /// Carries out `Command::SwitchLamp`.
    pub(super) fn apply_switch_lamp(&mut self, object: u32, on: bool) -> Result<(), Error> {
        let obj = &self.objects[object as usize];
        if !crate::lights::switchable(self, obj.def) {
            return Err(Error::new(format!(
                "the {} has no switch",
                self.content.objects[obj.def].name.to_lowercase()
            )));
        }
        crate::lights::set(self, object, on);
        Ok(())
    }

    /// Carries out `Command::DrawCurtains`.
    pub(super) fn apply_draw_curtains(&mut self, object: u32, shut: bool) -> Result<(), Error> {
        let obj = &self.objects[object as usize];
        if !crate::curtains::is_curtain(self, obj.def) {
            return Err(Error::new(format!(
                "the {} has no curtains to draw",
                self.content.objects[obj.def].name.to_lowercase()
            )));
        }
        crate::curtains::set(self, object, shut);
        Ok(())
    }
}
