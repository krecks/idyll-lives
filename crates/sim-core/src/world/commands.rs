//! The player's commands: what each changes, and the checks before it does.
//!
//! Split from `world.rs` (see `mod.rs`).

use super::*;

impl World {
    /// Applies a player command. Build and buy edits can be undone (`Command::Undo`) until time
    /// moves on or the player gives the household another kind of order. A command naming an
    /// object that's gone since the view showed it (sold, undone: an id handed out before and
    /// not there now) does nothing; an id never handed out is an error.
    pub fn apply(&mut self, mut cmd: Command) -> Result<(), Error> {
        // A command names objects by their stable ids; the simulation works in indices.
        if let Some(object) = cmd.object_mut() {
            let stable = *object;
            match self.index_of(stable) {
                Some(i) => *object = i as u32,
                None if stable < self.next_object_id => return Ok(()),
                None => return Err(Error::new(format!("unknown object {stable}"))),
            }
        }
        let history = match cmd {
            Command::Undo { household } => Some(self.undo(household)),
            Command::Redo { household } => Some(self.redo(household)),
            _ => None,
        };
        if let Some(result) = history {
            result?;
            self.refresh_rooms();
            return Ok(());
        }
        let snapshot = cmd.home_edit().and_then(|h| self.home_snapshot(h));
        let keeps_history = matches!(
            cmd,
            Command::SetSpeed { .. }
                | Command::SetLifespan { .. }
                | Command::SetPlayerMoves { .. }
                | Command::AddFunds { .. }
                | Command::SetSkill { .. }
                | Command::SetNeed { .. }
                | Command::SetBond { .. }
                | Command::SetAutonomy { .. }
                | Command::SetAutoFast { .. }
                | Command::SetRoutines { .. }
                | Command::SetHouseholdRoutines { .. }
                | Command::SkipHouseholdRoutine { .. }
                | Command::AddGoal { .. }
                | Command::RemoveGoal { .. }
                | Command::MoveGoal { .. }
                | Command::AcceptSuggestion { .. }
                | Command::DismissSuggestion { .. }
                | Command::DeclinePurchase { .. }
                // (The land keeps its own undo; `shape_land` carries what it changed into the
                // home edits kept.)
                | Command::ShapeLand { .. }
        );
        if let Err(e) = self.apply_command(cmd) {
            // (Nothing to keep for undo.)
            if let Some(s) = snapshot {
                self.lot.log_forget(s.lot_mark());
            }
            return Err(e);
        }
        self.catch_up_rooms();
        // Someone an order or an edit stopped (the sink they carried dishes to was moved) puts
        // down what they carry now, not when time moves on (Build mode stands still).
        // (The objects' index catches up as time moves on, if nobody has anything to put down.)
        if crate::cooking::must_settle(self) {
            crate::cooking::settle(self);
        }
        match snapshot {
            Some(s) => self.remember(s),
            None if !keeps_history => {
                self.undo.clear();
                self.redo.clear();
            }
            None => {}
        }
        Ok(())
    }

    /// Applies a command by handing it to the family that carries it.
    fn apply_command(&mut self, cmd: Command) -> Result<(), Error> {
        match cmd {
            Command::SetSpeed { .. }
            | Command::SetAutoFast { .. }
            | Command::SetLifespan { .. }
            | Command::SetPlayerMoves { .. }
            | Command::AddFunds { .. }
            | Command::SetNeed { .. }
            | Command::SetBond { .. }
            | Command::SetSkill { .. }
            | Command::SetAutonomy { .. }
            | Command::SetRoutines { .. }
            | Command::SetHouseholdRoutines { .. }
            | Command::SkipHouseholdRoutine { .. }
            | Command::AddGoal { .. }
            | Command::RemoveGoal { .. }
            | Command::MoveGoal { .. }
            | Command::AcceptSuggestion { .. }
            | Command::DismissSuggestion { .. }
            | Command::AcceptPurchase { .. }
            | Command::DeclinePurchase { .. }
            | Command::SwitchLamp { .. }
            | Command::DrawCurtains { .. } => self.apply_settings(cmd),
            Command::Use { .. } | Command::UseItem { .. } => self.apply_use(cmd),
            Command::MoveTo { .. }
            | Command::Social { .. }
            | Command::JoinCareer { .. }
            | Command::QuitCareer { .. }
            | Command::Visit { .. }
            | Command::GoHome { .. }
            | Command::Phone { .. }
            | Command::Cancel { .. } => self.apply_order(cmd),
            Command::Buy { .. }
            | Command::Sell { .. }
            | Command::MoveObject { .. }
            | Command::Recolor { .. }
            | Command::Upgrade { .. }
            | Command::Build { .. }
            | Command::Paint { .. }
            | Command::PaintFloor { .. }
            | Command::Undo { .. }
            | Command::Redo { .. }
            | Command::SetRoof { .. }
            | Command::Repair { .. }
            | Command::BuildBlueprint { .. }
            | Command::MoveRoom { .. }
            | Command::BuildGarage { .. }
            | Command::SellGarage { .. }
            | Command::BuildPorch { .. }
            | Command::RemovePorch { .. }
            | Command::BuildPorchPiece { .. }
            | Command::RemovePorchPiece { .. }
            | Command::ShapeLand { .. } => self.apply_edit(cmd),
            Command::Adopt { .. } | Command::MoveIn { .. } => self.apply_household(cmd),
        }
    }

    /// The household's and the player's settings: speed, money, needs, skills, goals, suggestions, the lamps and curtains.
    fn apply_settings(&mut self, cmd: Command) -> Result<(), Error> {
        match cmd {
            Command::SetSpeed { speed } => {
                if speed > MAX_SPEED {
                    return Err(Error::new(format!("speed must be 0..={MAX_SPEED}")));
                }
                // (The player's pick, super speed too: it stays when the quiet ends.)
                self.speed = speed;
                self.auto_super = false;
                if (1..SUPER_SPEED).contains(&self.speed) {
                    self.resume_speed = self.speed;
                }
            }
            Command::SetAutoFast { enabled } => {
                // (Turned on while it's quiet: super speed starts at the next step.)
                if enabled && !self.auto_fast {
                    self.was_calm = false;
                }
                self.auto_fast = enabled;
            }
            Command::SetLifespan { lifespan } => self.lifespan = lifespan,
            Command::SetPlayerMoves { enabled } => self.player_moves = enabled,
            Command::AddFunds { household, amount } => {
                let h = self
                    .households
                    .get_mut(household as usize)
                    .ok_or_else(|| Error::new(format!("unknown household {household}")))?;
                h.funds = (h.funds + amount).max(0);
            }
            Command::SetNeed { sim, need, value } => {
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
            }
            Command::SetBond {
                a,
                b,
                friendship,
                romance,
                chemistry,
                partners,
            } => {
                self.sim(a)?;
                self.sim(b)?;
                if a == b {
                    return Err(Error::new("a bond is between two residents"));
                }
                let (a, b) = (a as usize, b as usize);
                // (Only those who may court: the game can't get anyone else there, so a check can't either.)
                let grown_ups =
                    self.sims[a].adult(&self.content) && self.sims[b].adult(&self.content);
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
            }
            Command::SetSkill { sim, skill, level } => {
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
            }
            Command::SetAutonomy { enabled, household } => {
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
            }
            Command::SetRoutines { sim, routines } => {
                let routines = crate::planner::routines_from(&self.content, &routines)?;
                let s = self.sim_mut(sim)?;
                s.planner.routines = routines;
                // A changed block starts over (with today's plan).
                s.planner.run = None;
            }
            Command::SetHouseholdRoutines { sim, routines } => {
                let routines = crate::planner::routines_from(&self.content, &routines)?;
                let h = self.sim(sim)?.household as usize;
                self.households[h].routines = routines;
                for s in self.sims.iter_mut().filter(|s| s.household as usize == h) {
                    s.planner.run = None;
                }
            }
            Command::SkipHouseholdRoutine { sim, routine, skip } => {
                let skips = &mut self.sim_mut(sim)?.planner.skip_household;
                skips.retain(|&r| r != routine);
                if skip {
                    skips.push(routine);
                }
            }
            Command::AddGoal { sim, goal } => {
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
            }
            Command::RemoveGoal { sim, index } => {
                let goals = &mut self.sim_mut(sim)?.planner.goals;
                if index < goals.len() {
                    goals.remove(index);
                }
            }
            Command::MoveGoal { sim, index, to } => {
                let goals = &mut self.sim_mut(sim)?.planner.goals;
                if index < goals.len() && to < goals.len() {
                    let goal = goals.remove(index);
                    goals.insert(to, goal);
                }
            }
            Command::AcceptSuggestion { sim, index } => {
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
            }
            Command::DismissSuggestion { sim, index } => {
                let suggestions = &mut self.sim_mut(sim)?.planner.suggestions;
                if index < suggestions.len() {
                    suggestions.remove(index);
                }
            }
            Command::AcceptPurchase { sim } => {
                self.sim(sim)?;
                crate::planner::accept_purchase(self, sim as usize)?;
            }
            Command::DeclinePurchase { sim } => {
                self.sim(sim)?;
                crate::planner::decline_purchase(self, sim as usize)?;
            }
            Command::SwitchLamp { object, on } => {
                let obj = &self.objects[object as usize];
                if !crate::lights::switchable(self, obj.def) {
                    return Err(Error::new(format!(
                        "the {} has no switch",
                        self.content.objects[obj.def].name.to_lowercase()
                    )));
                }
                crate::lights::set(self, object, on);
            }
            Command::DrawCurtains { object, shut } => {
                let obj = &self.objects[object as usize];
                if !crate::curtains::is_curtain(self, obj.def) {
                    return Err(Error::new(format!(
                        "the {} has no curtains to draw",
                        self.content.objects[obj.def].name.to_lowercase()
                    )));
                }
                crate::curtains::set(self, object, shut);
            }
            _ => unreachable!("apply_settings is only given its own commands"),
        }
        Ok(())
    }

    /// Using something: the object a command names, or the item carried.
    fn apply_use(&mut self, cmd: Command) -> Result<(), Error> {
        match cmd {
            Command::Use {
                sim,
                object,
                interaction,
            } => {
                // (Who serves where, as the objects are now: `staff::closed`.)
                crate::staff::refresh(self);
                let obj = &self.objects[object as usize];
                let Some(inter) = self.content.objects[obj.def].interactions.get(interaction)
                else {
                    return Err(Error::new(format!(
                        "object {object} has no interaction {interaction}"
                    )));
                };
                if obj.broken() {
                    return Err(Error::new(format!(
                        "the {} is broken — repair it first",
                        self.content.objects[obj.def].name
                    )));
                }
                // Trying for a baby is asked of a partner (the social), not done at the bed alone.
                if inter.conceive {
                    return Err(Error::new(format!(
                        "{} is asked of a partner: click them",
                        inter.label
                    )));
                }
                if let Some(required) = missing_requirement(
                    &self.content,
                    &self.object_plot,
                    None,
                    &self.objects,
                    object,
                    inter,
                ) {
                    return Err(Error::new(format!(
                        "{} needs a working object on this lot that provides {required}",
                        inter.label
                    )));
                }
                if !obj.allows(inter, sim) {
                    let name = self.content.objects[obj.def].name.to_lowercase();
                    return Err(Error::new(match inter.when {
                        Some(w) if w != obj.open && obj.open => {
                            format!("the {name} is already open")
                        }
                        Some(w) if w != obj.open => format!("the {name} is closed"),
                        _ if inter.ripe && !obj.ripe() => self.content.objects[obj.def]
                            .not_ripe
                            .clone()
                            .unwrap_or_else(|| format!("the {name} isn't ripe yet")),
                        _ if (inter.loads
                            || inter.unloads
                            || inter.work_on == Some(crate::content::WorkOn::Dishes))
                            && obj.running() =>
                        {
                            format!("the {name} is still running")
                        }
                        _ if inter.work_on == Some(crate::content::WorkOn::Dishes)
                            && obj.done() =>
                        {
                            format!("the {name} is done: empty it first")
                        }
                        _ if inter.loads && obj.done() => {
                            format!("the {name} is done: take the washing out first")
                        }
                        _ if inter.unloads && obj.run.is_none() => {
                            format!("there's nothing in the {name}")
                        }
                        _ => format!("someone is using the {name}"),
                    }));
                }
                // (Lying along the whole window seat: not with someone else on it.)
                if inter.whole && !obj.can_use_for(sim, &self.content.objects[obj.def], interaction)
                {
                    let name = self.content.objects[obj.def].name.to_lowercase();
                    return Err(Error::new(format!("someone is using the {name}")));
                }
                if inter.needs_table
                    && crate::surface::faced_table(
                        &self.content,
                        &self.objects,
                        &self.nav(),
                        object,
                    )
                    .is_none()
                {
                    return Err(Error::new(format!(
                        "{} needs a table: pull the {} up to one",
                        inter.label,
                        self.content.objects[obj.def].name.to_lowercase()
                    )));
                }
                // Only their own home's things and public places', and what guests may use where
                // they're a guest (or on their way to be one).
                let s = self.sim(sim)?;
                let plot = self.object_plot.get(object as usize).copied().flatten();
                let visiting = |p: u32| {
                    let visit = |t: &Task| matches!(t.kind, TaskKind::Visit { plot } if plot == p);
                    s.current.as_ref().is_some_and(|a| visit(&a.task)) || s.queue.iter().any(visit)
                };
                let allowed = match order_access(&self.households, &self.plots, s, plot) {
                    Access::None if plot.is_some_and(visiting) => Access::Guest,
                    a => a,
                };
                let thing = self.content.objects[obj.def].name.to_lowercase();
                if allowed == Access::None {
                    return Err(Error::new(format!("the {thing} isn't {}'s to use", s.name)));
                }
                if allowed == Access::Guest && self.content.objects[obj.def].private {
                    return Err(Error::new(format!(
                        "{} is a guest there, and the {thing} is the household's own",
                        s.name
                    )));
                }
                if allowed == Access::Guest
                    && !guest_may(&self.content, &self.content.objects[obj.def], inter)
                {
                    return Err(Error::new(format!(
                        "{} is a guest there, and guests don't {}",
                        s.name,
                        inter.label.to_lowercase()
                    )));
                }
                // Who may use what, as free will keeps it: wine and the stove not for a child,
                // the kids' things not for grown-ups, homework for pupils, the baby's care for
                // grown-ups (and only with the baby there).
                if let Some(why) = unfit(&self.content, s.age, inter) {
                    return Err(Error::new(match why {
                        Unfit::TooYoung => {
                            format!("{} is too young to {}", s.name, inter.label.to_lowercase())
                        }
                        Unfit::TooBig => format!("{} is too big for the {thing}", s.name),
                        Unfit::NoHomework => format!("{} has no homework to do", s.name),
                        Unfit::TooYoungToCare => {
                            format!("{} is too young to look after a baby", s.name)
                        }
                    }));
                }
                // Its conditions, as free will keeps them: stargazing after dark under open sky,
                // washing up with dishes to wash.
                let work = self
                    .work_waiting()
                    .get(s.household as usize)
                    .copied()
                    .unwrap_or(0);
                if let Some(why) = unmet(
                    &self.lot,
                    crate::clock::hour(self.tick),
                    obj,
                    &self.content,
                    inter,
                    work,
                ) {
                    let label = inter.label.to_lowercase();
                    return Err(Error::new(match why {
                        Unmet::Hours => {
                            let [from, to] = inter.hours.unwrap_or_default();
                            format!(
                                "{} can only {label} from {} to {}",
                                s.name,
                                clock_time(from),
                                clock_time(to)
                            )
                        }
                        Unmet::Sky => format!(
                            "{} can only {label} under open sky: the {thing} has to stand outside",
                            s.name
                        ),
                        Unmet::Nothing(w) => format!("{}: {}", inter.label, w.none()),
                    }));
                }
                // A tool that cleans the floor: only at home, with a floor there it cleans.
                if let Some(sweep) = &inter.cleans {
                    let home = self.households[s.household as usize].plot;
                    if plot.is_none() || plot != home {
                        return Err(Error::new(format!("{}: only at home", inter.label)));
                    }
                    if decide::sweep_spot(s, &self.ctx(), sweep, decide::SWEEP_ORDERED).is_none() {
                        return Err(Error::new(format!(
                            "{}: the floors it cleans are clean",
                            inter.label
                        )));
                    }
                }
                // Sold at a post outside its hours (or a coffee from a café that's shut): closed.
                // (As `closed_at`: the objects on its plot, none off the plots.)
                let plot = self.object_plot.get(object as usize).copied().flatten();
                let on: Vec<u32> = match plot {
                    Some(p) => self
                        .objects
                        .iter()
                        .filter(|o| {
                            self.object_plot.get(o.id as usize).copied().flatten() == Some(p)
                        })
                        .map(|o| o.id)
                        .collect(),
                    None => Vec::new(),
                };
                if let Some(post) = crate::staff::closed(
                    &self.content,
                    &self.staff,
                    &self.objects,
                    &on,
                    object,
                    inter,
                    crate::clock::hour(self.tick),
                ) {
                    let name = self.content.objects[self.objects[post as usize].def]
                        .name
                        .to_lowercase();
                    let opens = crate::staff::opens(&self.content, &self.objects, post)
                        .map_or(String::new(), |h| {
                            format!(": it opens at {}", clock_time(h))
                        });
                    return Err(Error::new(format!("the {name} is closed{opens}")));
                }
                // A game for two: its places free (or one taken by someone there for the same
                // game, to play along with), and someone free to play, or a use for one to do
                // instead (`partners.rs`).
                if inter.partner {
                    let def = &self.content.objects[obj.def];
                    let mut others = obj.users[def.slots_for(interaction)]
                        .iter()
                        .flatten()
                        .filter(|&&u| u != sim);
                    let (first, more) = (others.next().copied(), others.next().is_some());
                    let along = |u: u32| {
                        crate::partners::game(self, u)
                            .is_some_and(|(o, i, _)| (o, i) == (object, interaction))
                    };
                    if more || first.is_some_and(|u| !along(u)) {
                        return Err(Error::new(format!("someone is using the {thing}")));
                    }
                    if first.is_none()
                        && inter.alone.is_none()
                        && crate::partners::partner_for(self, sim as usize, object, interaction)
                            .is_none()
                    {
                        return Err(Error::new(format!(
                            "nobody is free to {} with {}",
                            inter.label.to_lowercase(),
                            s.name
                        )));
                    }
                }
                if inter.cares()
                    && !obj.users().any(|u| {
                        self.sims
                            .get(u as usize)
                            .is_some_and(|b| self.content.life.baby(b.age))
                    })
                {
                    return Err(Error::new(format!("there's no baby in the {thing}")));
                }
                if upper_barred(&self.content, obj, sim, s.age) {
                    let young = crate::content::all_older(
                        self.content.objects[obj.def].upper_stages,
                        self.content.life.stage(s.age),
                    );
                    return Err(Error::new(format!(
                        "{} is too {} for the top bunk",
                        s.name,
                        if young { "young" } else { "old" }
                    )));
                }
                let household = s.household as usize;
                if !affordable(inter.cost, self.households[household].funds) {
                    return Err(Error::new(format!(
                        "not enough money: {} costs {}",
                        inter.label, inter.cost
                    )));
                }
                // Something in a public place or at someone else's: not leaving a baby home alone.
                crate::life::check_cover_at(
                    self,
                    sim as usize,
                    self.object_plot.get(object as usize).copied().flatten(),
                )?;
                self.enqueue(
                    sim,
                    TaskKind::Use {
                        object,
                        interaction,
                    },
                )?;
            }
            Command::UseItem { sim, item, action } => {
                let household = self.sim(sim)?.household;
                let it = self
                    .items
                    .get(item)
                    .filter(|it| it.household == household)
                    .ok_or_else(|| Error::new("that isn't theirs"))?;
                let kind = match action {
                    crate::command::ItemAction::Eat => {
                        if !it.edible() {
                            return Err(Error::new(if it.spoiled {
                                "that's gone off"
                            } else {
                                "there's nothing left to eat"
                            }));
                        }
                        TaskKind::Eat {
                            item: Some(item),
                            seat: None,
                            plate: None,
                            stage: 0,
                        }
                    }
                    crate::command::ItemAction::Clear => {
                        // (With no sink, into the bin.)
                        let dishes = self.content.cooking.dishes | self.content.cooking.bin;
                        let home = self.households[household as usize].plot;
                        let sink = self.objects.iter().any(|o| {
                            self.content.objects[o.def].stations & dishes != 0
                                && !o.broken()
                                && self.object_plot[o.id as usize] == home
                        });
                        if !sink {
                            return Err(Error::new(
                                "there's nowhere to wash up: buy a sink, a dishwasher or a bin",
                            ));
                        }
                        TaskKind::Dishes {
                            item,
                            station: None,
                            stage: 0,
                            scraps: 0,
                        }
                    }
                };
                self.enqueue(sim, kind)?;
            }
            _ => unreachable!("apply_use is only given its own commands"),
        }
        Ok(())
    }

    /// Orders to a resident: where to go, whom to talk to, what to do.
    fn apply_order(&mut self, cmd: Command) -> Result<(), Error> {
        match cmd {
            Command::MoveTo { sim, x, z } => {
                if !self.lot.in_bounds(x.floor() as i32, z.floor() as i32) {
                    return Err(Error::new("move target is off the lot"));
                }
                crate::life::check_cover_at(
                    self,
                    sim as usize,
                    self.plot_at(x.floor() as i32, z.floor() as i32),
                )?;
                self.enqueue(sim, TaskKind::MoveTo { x, z })?;
            }
            Command::Social {
                sim,
                target,
                social,
            } => {
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
            }
            Command::JoinCareer { sim, career, level } => {
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
            }
            Command::QuitCareer { sim } => {
                self.sim_mut(sim)?.job = None;
                crate::cars::job_lost(&mut self.cars, sim);
            }
            Command::Visit { sim, plot } => {
                if plot as usize >= self.plots.len() {
                    return Err(Error::new(format!("unknown plot {plot}")));
                }
                self.sim(sim)?;
                self.check_open(sim as usize, plot)?;
                // Not leaving a baby home alone with no money for a babysitter.
                crate::life::check_cover_at(self, sim as usize, Some(plot))?;
                self.enqueue(sim, TaskKind::Visit { plot })?;
            }
            Command::GoHome { sim } => {
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
            }
            Command::Phone {
                sim,
                action,
                contact,
            } => {
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
            }
            Command::Cancel { sim, index } => {
                let World {
                    sims,
                    objects,
                    content,
                    ..
                } = self;
                let s = sims
                    .get_mut(sim as usize)
                    .ok_or_else(|| Error::new(format!("unknown resident {sim}")))?;
                let offset = (s.current.is_some()
                    || s.engaged_with.is_some()
                    || s.heeding.is_some()) as usize;
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
            }
            _ => unreachable!("apply_order is only given its own commands"),
        }
        Ok(())
    }

    /// Building, buying and editing the lot (with undo and redo).
    fn apply_edit(&mut self, cmd: Command) -> Result<(), Error> {
        match cmd {
            Command::Buy {
                household,
                object,
                at,
                colors,
                turn,
                offset,
                size,
                tall,
            } => {
                let def = self.content.object_index(&object);
                let turn = def
                    .map(|d| self.check_turn(d, turn.unwrap_or(0)))
                    .transpose()?;
                let rot = at.map_or(0, |a| a[2] as u8);
                let layout = Layout {
                    offset: offset.unwrap_or([0, 0]),
                    extent: size,
                    tall: tall.unwrap_or(0),
                };
                let layout = def.map(|d| self.check_layout(d, rot, layout)).transpose()?;
                let colors = check_colors(colors)?;
                let id = self.buy_at(household, &object, at, colors, layout.unwrap_or_default())?;
                self.objects[id as usize].turn = turn.unwrap_or(0);
            }
            Command::Sell { household, object } => self.sell(household, object)?,
            Command::MoveObject {
                household,
                object,
                x,
                z,
                rot,
                turn,
                offset,
                size,
                tall,
            } => {
                let old = self.objects.get(object as usize).map(Layout::of);
                let def = self.objects.get(object as usize).map(|o| o.def);
                let turn = match (def, turn) {
                    (Some(d), Some(t)) => Some(self.check_turn(d, t)?),
                    _ => None,
                };
                // What isn't given stays as it was (checked against the new facing).
                let layout = match (def, old) {
                    (Some(d), Some(old)) => {
                        // An offset it keeps that doesn't suit its new facing (along another wall) goes.
                        let kept = if self.check_offset(d, rot, old.offset).is_ok() {
                            old.offset
                        } else {
                            [0, 0]
                        };
                        let wanted = Layout {
                            offset: offset.unwrap_or(kept),
                            extent: size.or(old.extent),
                            tall: tall.unwrap_or(old.tall),
                        };
                        Some(self.check_layout(d, rot, wanted)?)
                    }
                    _ => None,
                };
                let (offset, size, tall) = (
                    layout.map(|l| l.offset),
                    layout.and_then(|l| l.extent),
                    layout.map(|l| l.tall),
                );
                self.move_object_to(household, object, x, z, rot, turn, offset, size, tall)?;
            }
            Command::Recolor {
                household,
                object,
                colors,
            } => self.recolor(household, object, check_colors(colors)?)?,
            Command::Upgrade { household, object } => {
                self.upgrade(household, object)?;
            }
            Command::Build { household, edits } => self.build(household, &edits)?,
            Command::Paint { household, faces } => self.paint(household, &faces)?,
            Command::PaintFloor { household, tiles } => self.paint_floor(household, &tiles)?,
            Command::Undo { household } => self.undo(household)?,
            Command::Redo { household } => self.redo(household)?,
            Command::SetRoof {
                household,
                style,
                color,
            } => self.set_roof(household, style, color)?,
            Command::Repair { household, object } => self.repair(household, object)?,
            Command::BuildBlueprint {
                household,
                blueprint,
            } => self.build_blueprint(household, &blueprint)?,
            Command::MoveRoom {
                household,
                x,
                z,
                dx,
                dz,
            } => self.move_room(household, x, z, dx, dz)?,
            Command::BuildGarage {
                household,
                x0,
                z0,
                x1,
                z1,
                rot,
            } => self.build_garage(household, [x0, z0, x1, z1], rot)?,
            Command::SellGarage { household } => self.sell_garage(household)?,
            Command::BuildPorch {
                household,
                axis,
                x,
                z,
            } => self.build_porch(household, axis, x, z)?,
            Command::RemovePorch {
                household,
                axis,
                x,
                z,
            } => self.remove_porch(household, axis, x, z)?,
            Command::BuildPorchPiece {
                household,
                kind,
                x,
                z,
                w,
                d,
                rot,
            } => self.build_porch_piece(household, kind, x, z, w, d, rot)?,
            Command::RemovePorchPiece {
                household,
                kind,
                x,
                z,
            } => self.remove_porch_piece(household, kind, x, z)?,
            Command::ShapeLand {
                household,
                cost,
                meta,
                water,
            } => self.shape_land(household, cost, &meta, water)?,
            _ => unreachable!("apply_edit is only given its own commands"),
        }
        Ok(())
    }

    /// A household's people: who moves in, and who joins it.
    fn apply_household(&mut self, cmd: Command) -> Result<(), Error> {
        match cmd {
            Command::Adopt { household, child } => {
                crate::lifecycle::adopt(self, household, child)?;
            }
            Command::MoveIn {
                household,
                name,
                sims,
                bonds,
            } => self.move_in(household, name, &sims, &bonds)?,
            _ => unreachable!("apply_household is only given its own commands"),
        }
        Ok(())
    }
}
