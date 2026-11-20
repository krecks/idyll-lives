use super::*;

impl World {
    /// Building, buying and editing the lot (with undo and redo).
    pub(super) fn apply_edit(&mut self, cmd: Command) -> Result<(), Error> {
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
}
