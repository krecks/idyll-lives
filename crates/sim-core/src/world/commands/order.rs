//! Orders to a resident: where to go, whom to talk to, what to do.

use super::*;

impl World {
    /// Where a player sends someone on foot: a resident of the town, on the lot, and not leaving a
    /// baby alone (`check_cover_at`).
    fn check_walk(&self, sim: u32, x: f32, z: f32) -> Result<(), Error> {
        self.sim(sim)?;
        let (tx, tz) = (x.floor() as i32, z.floor() as i32);
        if !self.lot.in_bounds(tx, tz) {
            return Err(Error::new("move target is off the lot"));
        }
        crate::life::check_cover_at(self, sim as usize, self.plot_at(tx, tz))
    }

    /// Carries out `Command::MoveTo`.
    pub(super) fn apply_move_to(&mut self, sim: u32, x: f32, z: f32) -> Result<(), Error> {
        self.check_walk(sim, x, z)?;
        self.enqueue(sim, TaskKind::MoveTo { x, z })
    }

    /// Carries out `Command::Steer`: what they were doing and had queued is dropped for the walk.
    pub(super) fn apply_steer(&mut self, sim: u32, x: f32, z: f32) -> Result<(), Error> {
        self.check_walk(sim, x, z)?;
        // Into a wall or a thing: they stop at it, not go the long way round to what's behind it.
        let from = self.sims[sim as usize].pos;
        let [x, z] = if self.nav().line_walkable(from, [x, z]) {
            [x, z]
        } else {
            from
        };
        let World {
            sims,
            objects,
            content,
            ..
        } = self;
        let s = sims
            .get_mut(sim as usize)
            .filter(|s| s.here())
            .ok_or_else(|| Error::new(format!("unknown resident {sim}")))?;
        // (Twice at most: a talk from the seat ended gives back what they sat down to do, `end_activity`.)
        for _ in 0..2 {
            if s.current.is_some() {
                end_activity(s, content, objects);
            }
        }
        s.heeding = None;
        s.engaged_with = None;
        s.queue.clear();
        self.enqueue(sim, TaskKind::MoveTo { x, z })
    }

    /// Carries out `Command::Social`.
    pub(super) fn apply_social(
        &mut self,
        sim: u32,
        target: u32,
        social: usize,
    ) -> Result<(), Error> {
        let (a, t) = (self.sim(sim)?, self.sim(target)?);
        let def = self
            .content
            .socials
            .get(social)
            .ok_or_else(|| Error::new(format!("unknown social {social}")))?;
        if !conversation::social_allowed(&self.content, &self.relationships, a, t, def) {
            return Err(Error::new(format!(
                "{} isn't possible right now",
                def.label
            )));
        }
        // Trying for a baby needs a bed behind a shut door (`privacy.rs`).
        if def.success.effect == social::Effect::Conceive
            && crate::privacy::beds_in_content(&self.content)
            && crate::privacy::bed_in_world(self, sim as usize, target as usize).is_none()
        {
            return Err(Error::new(format!(
                "{} and {} have nowhere private to try for a baby: a double bed in a room of its own",
                a.name, t.name
            )));
        }
        // Walking over to someone out of the house (in the park, say).
        let (tx, tz) = t.tile();
        crate::life::check_cover_at(self, sim as usize, self.plot_at(tx, tz))?;
        self.enqueue(sim, TaskKind::Social { target, social })?;
        Ok(())
    }

    /// Carries out `Command::JoinCareer`.
    pub(super) fn apply_join_career(
        &mut self,
        sim: u32,
        career: usize,
        level: usize,
    ) -> Result<(), Error> {
        let def = self
            .content
            .careers
            .get(career)
            .ok_or_else(|| Error::new(format!("unknown career {career}")))?;
        let position = def
            .levels
            .get(level)
            .ok_or_else(|| Error::new(format!("{} has no level {level}", def.label)))?;
        if !self.sim(sim)?.adult(&self.content) {
            return Err(Error::new(format!(
                "{} is too young to work",
                self.sims[sim as usize].name
            )));
        }
        let s = &mut self.sims[sim as usize];
        if !crate::life::can_join(&self.content, position, &s.skills) {
            return Err(Error::new(format!(
                "{} doesn't have the skills to be a {} yet",
                s.name, position.title
            )));
        }
        s.job = Some(crate::life::Job::new(
            career,
            level,
            crate::clock::day(self.tick),
        ));
        // Back to work: no more pension.
        (s.retired, s.pension) = (false, 0);
        // (The car the new job calls for: `cars/looks.rs`.)
        let household = s.household;
        crate::cars::job_changed(&mut self.cars, &self.content, &self.sims, household);
        Ok(())
    }

    /// Carries out `Command::Visit`.
    pub(super) fn apply_visit(&mut self, sim: u32, plot: u32) -> Result<(), Error> {
        if plot as usize >= self.plots.len() {
            return Err(Error::new(format!("unknown plot {plot}")));
        }
        self.sim(sim)?;
        self.check_open(sim as usize, plot)?;
        // Not leaving a baby home alone with no money for a babysitter.
        crate::life::check_cover_at(self, sim as usize, Some(plot))?;
        self.enqueue(sim, TaskKind::Visit { plot })?;
        Ok(())
    }

    /// Carries out `Command::GoHome`.
    pub(super) fn apply_go_home(&mut self, sim: u32) -> Result<(), Error> {
        // A guest sent home says goodbye to one of the hosts first, if one is free (as at
        // the end of any visit), leaving whatever they chose to do themselves.
        let s = self.sim(sim)?;
        let goodbye = s
            .visiting
            .and_then(|v| crate::life::farewell(self, sim as usize, v.plot));
        if let Some((host, social)) =
            goodbye.filter(|_| self.sims[sim as usize].queue.len() + 1 < MAX_QUEUE)
        {
            let World {
                sims,
                content,
                objects,
                ..
            } = self;
            let s = &mut sims[sim as usize];
            if s.current.as_ref().is_some_and(|a| !a.task.directed) {
                end_activity(s, content, objects);
            }
            s.queue.push_back(Task {
                kind: TaskKind::Social {
                    target: host as u32,
                    social,
                },
                directed: false,
                routine: false,
            });
        }
        self.enqueue(sim, TaskKind::GoHome)?;
        Ok(())
    }

    /// Carries out `Command::Phone`.
    pub(super) fn apply_phone(
        &mut self,
        sim: u32,
        action: usize,
        contact: Option<u32>,
    ) -> Result<(), Error> {
        self.sim(sim)?;
        crate::phone::check_order(self, sim as usize, action, contact)?;
        self.enqueue(
            sim,
            TaskKind::Phone {
                action,
                contact,
                seat: None,
                answer: crate::phone::Answer::Pending,
            },
        )?;
        Ok(())
    }

    /// Carries out `Command::Cancel`.
    pub(super) fn apply_cancel(&mut self, sim: u32, index: usize) -> Result<(), Error> {
        let World {
            sims,
            objects,
            content,
            ..
        } = self;
        let s = sims
            .get_mut(sim as usize)
            .ok_or_else(|| Error::new(format!("unknown resident {sim}")))?;
        let offset =
            (s.current.is_some() || s.engaged_with.is_some() || s.heeding.is_some()) as usize;
        if index < offset {
            // (Called over: they don't go after all.)
            s.heeding = None;
            if s.current.is_some() {
                end_activity(s, content, objects);
            } else {
                s.engaged_with = None;
            }
        } else if index - offset < s.queue.len() {
            s.queue.remove(index - offset);
        }
        Ok(())
    }
}
