//! Using something: the object a command names, or the item carried.

use super::*;

impl World {
    /// Carries out `Command::Use`.
    pub(super) fn apply_use(
        &mut self,
        sim: u32,
        object: u32,
        interaction: usize,
    ) -> Result<(), Error> {
        // (Who serves where, as the objects are now: `staff::closed`.)
        crate::staff::refresh(self);
        let obj = &self.objects[object as usize];
        let Some(inter) = self.content.objects[obj.def].interactions.get(interaction) else {
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
                _ if inter.work_on == Some(crate::content::WorkOn::Dishes) && obj.done() => {
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
        if inter.whole && !obj.can_use_for(sim, &self.content.objects[obj.def], interaction) {
            let name = self.content.objects[obj.def].name.to_lowercase();
            return Err(Error::new(format!("someone is using the {name}")));
        }
        if inter.needs_table
            && crate::surface::faced_table(&self.content, &self.objects, &self.nav(), object)
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
                .filter(|o| self.object_plot.get(o.id as usize).copied().flatten() == Some(p))
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
                && crate::partners::partner_for(self, sim as usize, object, interaction).is_none()
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
        Ok(())
    }

    /// Carries out `Command::UseItem`.
    pub(super) fn apply_use_item(
        &mut self,
        sim: u32,
        item: u32,
        action: crate::command::ItemAction,
    ) -> Result<(), Error> {
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
                if !self.items.free_for(item, sim) {
                    return Err(Error::new("someone else is having that"));
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
        Ok(())
    }
}
