//! The build rules: what a household may pay for and where a piece may go.
//!
//! Split from `home.rs` (see `mod.rs`).

use super::*;

impl World {
    /// The roof over the household's home: a style and a colour (free; only for looks).
    pub fn set_roof(&mut self, household: u32, style: u8, color: u8) -> Result<(), Error> {
        let (_, plot) = self.home_of(household)?;
        self.not_a_flat(plot, "roof")?;
        let b = &self.content.build;
        if style as usize >= b.roofs.len().max(1) || color as usize >= b.roof_colors.len().max(1) {
            return Err(Error::new("unknown roof style or colour"));
        }
        self.plots[plot as usize].roof = Some(crate::world::RoofLook { style, color });
        self.structure_version = self.structure_version.wrapping_add(1);
        Ok(())
    }

    /// Refuses an edit of `what` (a roof, a garage, a porch, a blueprint) on `plot` if it is a flat:
    /// its building's envelope is the building's, and a flat is built on inside only.
    pub(crate) fn not_a_flat(&self, plot: u32, what: &str) -> Result<(), Error> {
        if self.plots.get(plot as usize).is_some_and(|p| p.is_flat()) {
            return Err(Error::new(format!(
                "in a flat you build inside it only: no {what}"
            )));
        }
        Ok(())
    }

    /// The household's index and its home plot.
    pub(crate) fn home_of(&self, household: u32) -> Result<(usize, u32), Error> {
        let h = household as usize;
        let plot = self
            .households
            .get(h)
            .ok_or_else(|| Error::new(format!("unknown household {household}")))?
            .plot
            .ok_or_else(|| Error::new("this household has no home"))?;
        Ok((h, plot))
    }

    /// What a build or buy edit costs in this game: nothing in Creative.
    pub fn build_price(&self, price: i64) -> i64 {
        match self.mode {
            GameMode::Living => price,
            GameMode::Creative => 0,
        }
    }

    /// Checks that household `h` can pay `cost` for an edit (free in Creative); `Ok` means
    /// the edit may go ahead and be paid with `pay`.
    pub(crate) fn can_pay(
        &self,
        h: usize,
        cost: i64,
        refusal: impl FnOnce() -> String,
    ) -> Result<(), Error> {
        let cost = self.build_price(cost);
        if cost > 0 && self.households[h].funds < cost {
            return Err(Error::new(refusal()));
        }
        Ok(())
    }

    pub(crate) fn pay(&mut self, h: usize, cost: i64) {
        self.households[h].funds -= self.build_price(cost);
    }

    /// An angle past the facing for object type `def`: only those that turn freely take one.
    pub(crate) fn check_turn(&self, def: usize, turn: u8) -> Result<u8, Error> {
        let d = &self.content.objects[def];
        if turn == 0 {
            Ok(0)
        } else if !d.turns {
            Err(Error::new(format!("the {} only faces four ways", d.name)))
        } else if turn >= 90 {
            Err(Error::new(format!("turn {turn}° is past the next facing")))
        } else {
            Ok(turn)
        }
    }

    /// How far object type `def`, facing `rot`, stands off the middle of its footprint: by up
    /// to half a tile, things on a wall only along it. Stairs stand square.
    pub(crate) fn check_offset(
        &self,
        def: usize,
        rot: u8,
        offset: [i8; 2],
    ) -> Result<[i8; 2], Error> {
        let d = &self.content.objects[def];
        // Across the wall (z facing 0 or 2, x facing 1 or 3) a thing on it can't move.
        let across = if rot.is_multiple_of(2) {
            offset[1]
        } else {
            offset[0]
        };
        if offset == [0, 0] {
            Ok(offset)
        } else if d.stairs {
            Err(Error::new(format!(
                "the {} stands square on its tiles",
                d.name.to_lowercase()
            )))
        } else if d.layer == Layer::Wall && across != 0 {
            Err(Error::new("that hangs flat on its wall"))
        } else if offset.iter().any(|&o| o.unsigned_abs() > MAX_OFFSET as u8) {
            Err(Error::new("that's off its tile"))
        } else {
            Ok(offset)
        }
    }

    /// A size for object type `def` (centimetres across and deep, before turning, and
    /// centimetres taller): each side `MIN_SIZE` to `MAX_SIZE` tiles. Stairs keep theirs; things
    /// on a wall keep their depth and things on a top their footprint (they can still be taller
    /// or shorter).
    pub(crate) fn check_size(
        &self,
        def: usize,
        size: [u16; 2],
        tall: i16,
    ) -> Result<([u16; 2], i16), Error> {
        let d = &self.content.objects[def];
        let name = d.name.to_lowercase();
        let own = kind_size(d.footprint);
        if d.stairs && (size != own || tall != 0) {
            Err(Error::new(format!("the {name} keep their size")))
        } else if d.layer == Layer::Surface && size != own {
            Err(Error::new(format!(
                "the {name} only gets taller or shorter"
            )))
        } else if d.layer == Layer::Wall && size[1] != own[1] {
            Err(Error::new(format!("the {name} hangs flat on its wall")))
        } else if size
            .iter()
            .any(|&s| !(MIN_SIZE..=MAX_SIZE as u16 * 100).contains(&s))
        {
            Err(Error::new(format!(
                "things measure {MIN_SIZE} cm to {MAX_SIZE} m each way"
            )))
        } else if !(-MAX_SHORTER..=MAX_TALLER).contains(&tall) {
            Err(Error::new(format!(
                "things grow {} m at most, or shrink {} m",
                MAX_TALLER as f32 / 100.0,
                MAX_SHORTER as f32 / 100.0
            )))
        } else {
            Ok((size, tall))
        }
    }

    /// `layout` for object type `def` facing `rot`, checked (`check_offset`, `check_size`).
    pub(crate) fn check_layout(
        &self,
        def: usize,
        rot: u8,
        layout: Layout,
    ) -> Result<Layout, Error> {
        let offset = self.check_offset(def, rot, layout.offset)?;
        let (size, tall) = self.check_size(
            def,
            layout
                .extent
                .unwrap_or(kind_size(self.content.objects[def].footprint)),
            layout.tall,
        )?;
        Ok(Layout {
            offset,
            extent: Some(size),
            tall,
        })
    }
}
