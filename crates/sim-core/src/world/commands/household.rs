use super::*;

impl World {
    /// A household's people: who moves in, and who joins it.
    pub(super) fn apply_household(&mut self, cmd: Command) -> Result<(), Error> {
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
