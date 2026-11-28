//! The player's commands: what each changes, and the checks before it does.

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

    /// Applies a command: one arm each, a longer one by its own method beside the others of its
    /// kind (settings.rs, use_command.rs, order.rs, edit.rs).
    fn apply_command(&mut self, cmd: Command) -> Result<(), Error> {
        match cmd {
            // The household's and the player's settings: speed, money, needs, skills, goals,
            // suggestions, the lamps and curtains.
            Command::SetSpeed { speed } => self.apply_set_speed(speed)?,
            Command::SetAutoFast { enabled } => self.apply_set_auto_fast(enabled)?,
            Command::SetLifespan { lifespan } => self.lifespan = lifespan,
            Command::SetPlayerMoves { enabled } => self.player_moves = enabled,
            Command::AddFunds { household, amount } => self.apply_add_funds(household, amount)?,
            Command::SetNeed { sim, need, value } => self.apply_set_need(sim, need, value)?,
            Command::SetBond {
                a,
                b,
                friendship,
                romance,
                chemistry,
                partners,
            } => self.apply_set_bond(a, b, friendship, romance, chemistry, partners)?,
            Command::SetSkill { sim, skill, level } => self.apply_set_skill(sim, skill, level)?,
            Command::SetAutonomy { enabled, household } => {
                self.apply_set_autonomy(enabled, household)?
            }
            Command::SetRoutines { sim, routines } => self.apply_set_routines(sim, routines)?,
            Command::SetHouseholdRoutines { sim, routines } => {
                self.apply_set_household_routines(sim, routines)?
            }
            Command::SkipHouseholdRoutine { sim, routine, skip } => {
                self.apply_skip_household_routine(sim, routine, skip)?
            }
            Command::AddGoal { sim, goal } => self.apply_add_goal(sim, goal)?,
            Command::RemoveGoal { sim, index } => self.apply_remove_goal(sim, index)?,
            Command::MoveGoal { sim, index, to } => self.apply_move_goal(sim, index, to)?,
            Command::AcceptSuggestion { sim, index } => self.apply_accept_suggestion(sim, index)?,
            Command::DismissSuggestion { sim, index } => {
                self.apply_dismiss_suggestion(sim, index)?
            }
            Command::AcceptPurchase { sim } => {
                self.sim(sim)?;
                crate::planner::accept_purchase(self, sim as usize)?;
            }
            Command::DeclinePurchase { sim } => {
                self.sim(sim)?;
                crate::planner::decline_purchase(self, sim as usize)?;
            }
            Command::SwitchLamp { object, on } => self.apply_switch_lamp(object, on)?,
            Command::DrawCurtains { object, shut } => self.apply_draw_curtains(object, shut)?,
            // Using something: the object a command names, or the item carried.
            Command::Use {
                sim,
                object,
                interaction,
            } => self.apply_use(sim, object, interaction)?,
            Command::UseItem { sim, item, action } => self.apply_use_item(sim, item, action)?,
            // Orders to a resident: where to go, whom to talk to, what to do.
            Command::MoveTo { sim, x, z } => self.apply_move_to(sim, x, z)?,
            Command::Steer { sim, x, z, run } => {
                self.apply_steer(sim, x, z)?;
                self.sim_mut(sim)?.running = run;
            }
            Command::Hold { sim, on } => self.sim_mut(sim)?.steered = on,
            Command::Social {
                sim,
                target,
                social,
            } => self.apply_social(sim, target, social)?,
            Command::JoinCareer { sim, career, level } => {
                self.apply_join_career(sim, career, level)?
            }
            Command::QuitCareer { sim } => {
                self.sim_mut(sim)?.job = None;
                crate::cars::job_lost(&mut self.cars, sim);
            }
            Command::Visit { sim, plot } => self.apply_visit(sim, plot)?,
            Command::GoHome { sim } => self.apply_go_home(sim)?,
            Command::Phone {
                sim,
                action,
                contact,
            } => self.apply_phone(sim, action, contact)?,
            Command::Cancel { sim, index } => self.apply_cancel(sim, index)?,
            // Building, buying and editing the lot (with undo and redo).
            Command::Buy {
                household,
                object,
                at,
                colors,
                turn,
                offset,
                size,
                tall,
            } => self.apply_buy(household, object, at, colors, turn, offset, size, tall)?,
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
            } => self.apply_move_object(household, object, x, z, rot, turn, offset, size, tall)?,
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
            // A household's people: who moves in, and who joins it.
            Command::Adopt { household, child } => {
                crate::lifecycle::adopt(self, household, child)?;
            }
            Command::MoveIn {
                household,
                name,
                sims,
                bonds,
            } => self.move_in(household, name, &sims, &bonds)?,
        }
        Ok(())
    }
}

mod edit;
mod order;
mod settings;
mod use_command;
