//! Building, buying and editing the lot (with undo and redo).

use super::*;

impl World {
    /// Carries out `Command::Buy`.
    #[allow(clippy::too_many_arguments)] // (the command's fields, one each)
    pub(super) fn apply_buy(
        &mut self,
        household: u32,
        object: String,
        at: Option<[i32; 3]>,
        colors: Colors,
        turn: Option<u8>,
        offset: Option<[i8; 2]>,
        size: Option<[u16; 2]>,
        tall: Option<i16>,
    ) -> Result<(), Error> {
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
        Ok(())
    }

    /// Carries out `Command::MoveObject`.
    #[allow(clippy::too_many_arguments)] // (the command's fields, one each)
    pub(super) fn apply_move_object(
        &mut self,
        household: u32,
        object: u32,
        x: i32,
        z: i32,
        rot: u8,
        turn: Option<u8>,
        offset: Option<[i8; 2]>,
        size: Option<[u16; 2]>,
        tall: Option<i16>,
    ) -> Result<(), Error> {
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
        Ok(())
    }
}
