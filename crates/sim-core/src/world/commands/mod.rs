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
}

mod edit;
mod household;
mod order;
mod settings;
mod use_command;
