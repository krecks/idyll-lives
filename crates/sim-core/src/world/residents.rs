//! The residents: moving in, out, and looking them up.

use super::*;

impl World {
    /// Whether the briefs (and `members`) cover every resident.
    pub(crate) fn members_cover(&self) -> bool {
        self.briefs.len() == self.sims.len()
    }

    /// Says which plot object `id` belongs to though it stands elsewhere (`ObjectInstance::keeper`:
    /// a flat's letterbox in the hall), or that it belongs where it stands (None). What it is
    /// counted on (`object_plot`) follows.
    pub(crate) fn keep(&mut self, id: u32, keeper: Option<u32>) {
        let keeper = keeper.filter(|&k| (k as usize) < self.plots.len());
        let Some(o) = self.objects.get_mut(id as usize) else {
            return;
        };
        o.keeper = keeper;
        let (x, z) = (o.x, o.z);
        let plot = keeper.or_else(|| self.plot_at(x, z));
        if self.object_plot[id as usize] != plot {
            self.object_plot[id as usize] = plot;
            self.objects_version = self.objects_version.wrapping_add(1);
        }
    }

    /// Adds a Sim with random starting needs. Identity, traits and perks are validated.
    pub fn spawn_sim(&mut self, spawn: &SimSpawn) -> Result<u32, Error> {
        self.spawn_at(spawn, false)
    }

    /// A new resident; with `reuse`, in the first slot someone gone left (their story events
    /// then name them as a former resident), else in a new slot.
    pub(crate) fn spawn_at(&mut self, spawn: &SimSpawn, reuse: bool) -> Result<u32, Error> {
        let SimSpawn { name, x, z, .. } = spawn;
        if !self.lot.in_bounds(x.floor() as i32, z.floor() as i32) {
            return Err(Error::new(format!("resident '{name}' spawns off the lot")));
        }
        let slot = if reuse {
            self.sims.iter().position(|s| !s.here())
        } else {
            None
        };
        if slot.is_none() && self.sims.len() >= MAX_SIMS {
            return Err(Error::new(format!("at most {MAX_SIMS} residents per town")));
        }
        let id = slot.unwrap_or(self.sims.len());
        if spawn.household as usize >= self.households.len() {
            return Err(Error::new(format!(
                "{name}: unknown household {}",
                spawn.household
            )));
        }
        let content = &self.content;
        let gender = spawn
            .gender
            .clone()
            .or_else(|| content.genders.first().map(|g| g.id.clone()))
            .unwrap_or_default();
        let attracted_to = spawn
            .attracted_to
            .clone()
            .unwrap_or_else(|| content.genders.iter().map(|g| g.id.clone()).collect());
        content
            .validate_identity(&gender, &attracted_to)
            .map_err(|e| Error::new(format!("{name}: {e}")))?;
        let mods = content
            .character_modifiers(&spawn.traits, &spawn.perks)
            .map_err(|e| Error::new(format!("{name}: {e}")))?;
        let gender_ix = content.genders.iter().position(|g| g.id == gender);
        let attraction = content
            .genders
            .iter()
            .enumerate()
            .filter(|(_, g)| attracted_to.contains(&g.id))
            .fold(0u32, |m, (i, _)| m | (1 << i));

        let young = !content
            .life
            .adult(spawn.age.unwrap_or(content.life.start_age[0]));
        let job = match &spawn.job {
            None => None,
            Some(_) if young => None,
            Some(j) => {
                let career = content
                    .career_index(&j.career)
                    .ok_or_else(|| Error::new(format!("{name}: unknown career '{}'", j.career)))?;
                let level = j.level.min(content.careers[career].levels.len() - 1);
                Some(crate::life::Job::new(career, level, 0))
            }
        };
        // Traits give a head start; explicit levels win; anyone placed in a job has the skills for it.
        let mut skills = content.starting_skills(&spawn.traits);
        for (id, level) in &spawn.skills {
            let s = content
                .skill_index(id)
                .ok_or_else(|| Error::new(format!("{name}: unknown skill '{id}'")))?;
            skills[s] = *level;
        }
        if let Some(j) = &job {
            for &(s, level) in &content.careers[j.career].levels[j.level].requires {
                skills[s] = skills[s].max(level);
            }
        }
        let max = content.skill_rules.max_level;
        for s in &mut skills {
            *s = s.clamp(0.0, max);
        }

        let age = spawn
            .age
            .unwrap_or_else(|| crate::lifecycle::default_age(&content.life, name, id))
            .clamp(0.0, 150.0);
        // At or past the retirement age: retired already, with the pension of a job they were
        // given (at least the basic pension).
        let retired = content.life.retire_at.is_some_and(|at| age >= at);
        let pension = if retired {
            crate::lifecycle::pension(content, job.as_ref())
        } else {
            0
        };
        let job = job.filter(|_| !retired);
        let mut needs = [0.0; MAX_NEEDS];
        for n in needs.iter_mut().take(self.content.needs.len()) {
            *n = self.rng.range(0.55, 0.95);
        }
        let sim = Sim {
            id: id as u32,
            name: name.clone(),
            appearance: spawn.appearance.clone(),
            outfit: 0,
            gender,
            attracted_to,
            traits: spawn.traits.clone(),
            perks: spawn.perks.clone(),
            base_mods: mods.clone(),
            mods,
            buffs: Vec::new(),
            pending_spend: 0,
            worked: 0,
            made: 0,
            taken: 0,
            pos: [*x, *z],
            yaw: 0.0,
            pose: Pose::Stand,
            seated_from: None,
            use_seat: None,
            needs,
            feelings: Vec::new(),
            engaged_with: None,
            heeding: None,
            gender_ix,
            attraction,
            queue: VecDeque::new(),
            current: None,
            paused: None,
            join_spot: None,
            idle_ticks: 0,
            follow_up: None,
            fetched: None,
            unreachable: [None; UNREACHABLE],
            social_waited: 0,
            called: false,
            nav_seen: 0,
            deferred: false,
            commute: None,
            on_trail: None,
            game: None,
            wants_partner: false,
            joins: None,
            partner_wait: 0,
            being_served: false,
            serve_wait: 0,
            bound_for_bike: None,
            bike_parked: None,
            bike_spot: None,
            back_at: 0,
            riding: None,
            bound_for_car: None,
            taxi: None,
            car_seat: 0,
            car_door: 0,
            car_wait: 0,
            kerb_wait: None,
            shift_from: 0,
            alone_until: 0,
            household: spawn.household,
            job,
            away_until: None,
            visiting: None,
            transition: None,
            skills,
            skill_ups: 0,
            practice_fatigue: 0.0,
            nap_until: 0,
            ladder_ticks: 0,
            job_search_from: 0,
            planner: Default::default(),
            mess: None,
            sweep: None,
            swept: None,
            wore_pressed: false,
            reshaped: false,
            lit: false,
            care_out: None,
            carry_out: None,
            carried_by: None,
            night_woken: 0,
            conceived: None,
            to_bed: None,
            kept_out: 0,
            news: None,
            mail: None,
            set_up: None,
            accident_at: [0; crate::content::MAX_ACCIDENTS],
            empty_since: [0; crate::content::MAX_ACCIDENTS],
            age,
            gone: None,
            held: None,
            invited_until: 0,
            serve_for: None,
            hand_to: None,
            expecting: 0,
            served: None,
            hung_up: None,
            retired,
            pension,
            school_day: 0,
            grade: content.life.school.as_ref().map_or(0.0, |s| s.start_grade),
            steps: 1,
            owed: 0,
        };
        if let Some(slot) = slot {
            self.retire_slot(slot);
            self.sims[slot] = sim;
            self.relationships.clear(slot);
            self.generations[slot] = self.generations[slot].wrapping_add(1);
        } else {
            self.sims.push(sim);
            self.relationships.grow(self.sims.len());
            self.generations.push(0);
        }
        // (The household index no longer matches the residents: `household_sims`; nor the turns.)
        self.members.moved = true;
        self.turns_fresh = false;
        self.refresh_base_mods(id);
        Ok(id as u32)
    }

    /// Before someone new takes `slot`: story events naming its former occupant name them as a
    /// former resident instead; former residents no event names any more are forgotten.
    fn retire_slot(&mut self, slot: usize) {
        use crate::social::FORMER;
        let gone = &self.sims[slot];
        let household = self
            .households
            .get(gone.household as usize)
            .map_or_else(String::new, |h| h.name.clone());
        self.former.push(Former {
            name: gone.name.clone(),
            household,
        });
        let k = (self.former.len() - 1) as u32;
        // (Letters of theirs still on the way are lost in the post.)
        for h in &mut self.households {
            h.letters.retain(|&w| w != slot as u32);
        }
        self.events.rewrite_refs(|r| {
            if *r == slot as u32 {
                *r = FORMER | k;
            }
        });
        let mut used = vec![false; self.former.len()];
        self.events.rewrite_refs(|r| {
            if *r & FORMER != 0 {
                used[(*r & !FORMER) as usize] = true;
            }
        });
        let mut remap = vec![0u32; self.former.len()];
        let mut kept = Vec::new();
        for (i, f) in self.former.drain(..).enumerate() {
            if used[i] {
                remap[i] = kept.len() as u32;
                kept.push(f);
            }
        }
        self.former = kept;
        self.events.rewrite_refs(|r| {
            if *r & FORMER != 0 {
                *r = FORMER | remap[(*r & !FORMER) as usize];
            }
        });
    }

    /// A resident leaves town for good (`why`): whatever they were doing stops, nobody waits
    /// for them, their relationships are forgotten, and their slot is kept (ids don't change)
    /// until a newcomer takes it.
    pub fn depart(&mut self, i: usize, why: GoneWhy) {
        let id = i as u32;
        for o in &mut self.objects {
            o.release(id);
        }
        let day = crate::clock::day(self.tick);
        let sim = &mut self.sims[i];
        sim.clear_activity();
        sim.away_until = None;
        sim.job = None;
        sim.planner.run = None;
        sim.planner.thought = None;
        sim.gone = Some(Gone { why, day });
        crate::cars::job_lost(&mut self.cars, id);
        // Nobody talks to them, listens in, opens the door to them or rings them any more (their
        // slot may soon be someone else's), and a visitor waits for someone else to come.
        let with_them = |t: &Task| match t.kind {
            TaskKind::Social { target, .. }
            | TaskKind::Join { leader: target }
            | TaskKind::Answer { guest: target } => target == id,
            TaskKind::Phone { contact, .. } => contact == Some(id),
            TaskKind::Carry { baby, .. } => baby == Some(id),
            TaskKind::Serve { guest, .. } => guest == id,
            _ => false,
        };
        let World {
            sims,
            content,
            objects,
            households,
            ..
        } = self;
        for other in sims.iter_mut() {
            if other.engaged_with == Some(id) {
                other.engaged_with = None;
            }
            if other.heeding == Some(id) {
                other.heeding = None;
            }
            other.queue.retain(|t| !with_them(t));
            if other.current.as_ref().is_some_and(|a| with_them(&a.task)) {
                end_activity(other, content, objects);
            }
            if let Some(Activity {
                phase: Phase::Knocking { host, .. },
                ..
            }) = other.current.as_mut()
                && *host == Some(id)
            {
                *host = None;
            }
        }
        // A baby on the way with them is no longer (nobody's named as a parent who isn't here).
        for h in households.iter_mut() {
            if h.expecting.is_some_and(|e| e.parents.contains(&id)) {
                h.expecting = None;
            }
        }
        self.relationships.clear(i);
        self.structure_version = self.structure_version.wrapping_add(1);
        self.rooms_need_clean_rescore();
    }

    /// A resident's own modifiers: traits and perks, and what their life stage does (then the
    /// active feelings on top, as `Sim::refresh_buffs` does).
    pub(crate) fn refresh_base_mods(&mut self, i: usize) {
        let content = &self.content;
        let sim = &mut self.sims[i];
        let mut mods = content
            .character_modifiers(&sim.traits, &sim.perks)
            .unwrap_or_default();
        if let Some(stage) = content.life.stages.get(content.life.stage(sim.age)) {
            mods.combine(&stage.mods);
        }
        sim.mods = mods.clone();
        for &m in &sim.buffs {
            if let Some(e) = &content.feelings[m].effects {
                sim.mods.combine(e);
            }
        }
        sim.base_mods = mods;
    }

    /// New residents move into household `household`'s home (see `Command::MoveIn`): all of
    /// them or, if any can't (unknown traits, a bad bond), none.
    pub fn move_in(
        &mut self,
        household: u32,
        name: Option<String>,
        sims: &[SimSpawn],
        bonds: &[BondRaw],
    ) -> Result<(), Error> {
        let (h, _) = self.home_of(household)?;
        if sims.is_empty() {
            return Err(Error::new("nobody to move in"));
        }
        let members = self
            .sims
            .iter()
            .filter(|s| s.here() && s.household as usize == h)
            .count();
        let most = self.content.rules.max_household;
        if members + sims.len() > most {
            return Err(Error::new(format!(
                "at most {most} residents per household"
            )));
        }
        // (The player's own choice: up to the sim's limit, past a town file's `residentsMost`,
        // which holds back only newcomers and births.)
        let cap = MAX_SIMS;
        if self.sims.iter().filter(|s| s.here()).count() + sims.len() > cap {
            return Err(Error::new(format!("at most {cap} residents in this town")));
        }
        if bonds.iter().any(|b| b.a >= sims.len() || b.b >= sims.len()) {
            return Err(Error::new("a bond refers to someone who isn't moving in"));
        }
        let life = &self.content.life;
        let grown_up = |s: &SimSpawn| s.age.is_none_or(|a| life.adult(a));
        let adult_at_home = self
            .sims
            .iter()
            .any(|s| s.here() && s.household as usize == h && s.adult(&self.content));
        if !adult_at_home && !sims.iter().any(grown_up) {
            return Err(Error::new("children need a grown-up to live with"));
        }
        // Newcomers may take slots of people who are gone, so a failure restores all of it.
        let before = (
            self.sims.clone(),
            self.relationships.clone(),
            self.rng.clone(),
            self.former.clone(),
            self.generations.clone(),
            self.events.clone(),
        );
        let mut ids = Vec::with_capacity(sims.len());
        let moved = sims
            .iter()
            .try_for_each(|s| {
                let spawn = SimSpawn {
                    household,
                    ..s.clone()
                };
                ids.push(self.spawn_at(&spawn, true)? as usize);
                Ok(())
            })
            .and_then(|()| {
                let bonds: Vec<BondRaw> = bonds
                    .iter()
                    .map(|b| BondRaw {
                        a: ids[b.a],
                        b: ids[b.b],
                        preset: b.preset.clone(),
                    })
                    .collect();
                self.init_relationships_for(&ids, &bonds)
            });
        if let Err(e) = moved {
            (
                self.sims,
                self.relationships,
                self.rng,
                self.former,
                self.generations,
                self.events,
            ) = before;
            return Err(e);
        }
        let first = ids[0];
        crate::lifecycle::cribs_for_babies(self, h);
        if let Some(name) = name.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
            self.households[h].name = name.to_string();
        }
        self.structure_version = self.structure_version.wrapping_add(1);
        self.rooms_need_clean_rescore();
        self.events
            .push_detail(self.tick, social::EventKind::MovedIn, first, None, None);
        Ok(())
    }

    /// Chemistry for every pair, household bonds, then explicit bonds from the lot file.
    pub(crate) fn init_relationships(&mut self, bonds: &[BondRaw]) -> Result<(), Error> {
        let all: Vec<usize> = (0..self.sims.len()).collect();
        self.init_relationships_for(&all, bonds)
    }

    /// Chemistry and household bonds for every pair with one of the `new` residents in it (the
    /// others already have theirs; the gone have none), then `bonds` (indices into the list).
    pub(crate) fn init_relationships_for(
        &mut self,
        new: &[usize],
        bonds: &[BondRaw],
    ) -> Result<(), Error> {
        let n = self.sims.len();
        let default_bond = self.content.social_rules.default_bond;
        // (Who is new, by slot: a whole town is, as it loads, and a search of the list for every
        // pair grew with the cube of the town.)
        let mut is_new = vec![false; n];
        for &i in new.iter().filter(|&&i| i < n) {
            is_new[i] = true;
        }
        for a in 0..n {
            for b in a + 1..n {
                if !(is_new[a] || is_new[b]) || !self.sims[a].here() || !self.sims[b].here() {
                    continue;
                }
                let (sa, sb) = (&self.sims[a], &self.sims[b]);
                let shared = sa.traits.iter().filter(|t| sb.traits.contains(t)).count() as f32;
                let clashes = sa
                    .traits
                    .iter()
                    .filter_map(|t| self.content.traits.iter().find(|d| &d.id == t))
                    .map(|d| d.conflicts.iter().filter(|c| sb.traits.contains(c)).count())
                    .sum::<usize>() as f32;
                let same_household = sa.household == sb.household;
                let chemistry = self.rng.range(-0.6, 0.6) + shared * 0.15 - clashes * 0.3;
                self.relationships.set_chemistry(a, b, chemistry);
                if same_household && let Some(p) = default_bond {
                    self.relationships.bond(a, b, &p);
                }
            }
        }
        for bond in bonds {
            let preset = self
                .content
                .bond_presets
                .get(&bond.preset)
                .ok_or_else(|| Error::new(format!("unknown bond preset '{}'", bond.preset)))?;
            if bond.a >= n || bond.b >= n || bond.a == bond.b {
                return Err(Error::new(format!(
                    "bond {}-{} refers to unknown residents",
                    bond.a, bond.b
                )));
            }
            if preset.partners
                && [bond.a, bond.b].iter().any(|&s| {
                    self.relationships
                        .partner_of(s)
                        .is_some_and(|p| p != bond.a && p != bond.b)
                })
            {
                return Err(Error::new("a resident can only have one partner"));
            }
            self.relationships.bond(bond.a, bond.b, preset);
        }
        Ok(())
    }

    pub(crate) fn sim(&self, id: u32) -> Result<&Sim, Error> {
        self.sims
            .get(id as usize)
            .filter(|s| s.here())
            .ok_or_else(|| Error::new(format!("unknown resident {id}")))
    }

    pub(crate) fn sim_mut(&mut self, id: u32) -> Result<&mut Sim, Error> {
        self.sims
            .get_mut(id as usize)
            .filter(|s| s.here())
            .ok_or_else(|| Error::new(format!("unknown resident {id}")))
    }

    pub(crate) fn enqueue(&mut self, sim: u32, kind: TaskKind) -> Result<(), Error> {
        let s = self.sim_mut(sim)?;
        if s.queue.len() >= MAX_QUEUE {
            return Err(Error::new("action queue is full"));
        }
        s.queue.push_back(Task {
            kind,
            directed: true,
            routine: false,
        });
        Ok(())
    }
}
