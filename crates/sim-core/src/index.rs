//! Objects by plot, so a resident deciding what to do looks at what's near them rather than at
//! every object in town.
//!
//! Residents only use what's at home, in public places and off the plots, or, as guests, what's
//! on the plot they're visiting (see `world::access`). The index lists exactly those objects per
//! plot, in id order, so choices come out as they did from a scan of the whole town. It is
//! derived from `World::object_plot` and rebuilt when `World::objects_version` moves (objects
//! bought, sold or moved, walls built, a game loaded; not a car parking); see
//! `World::refresh_index`.
//!
//! Public and off-plot objects aren't copied into every plot's list: they're kept in `CELL` m
//! cells, and residents look at those within reach of where they are (`usable_near`, content
//! `autonomy.reach`), still in id order.

use std::borrow::Cow;

use crate::world::Plot;

/// Spatial cells for public objects (m; docs/design/open-world.md).
pub(crate) const CELL: f32 = 64.0;

#[derive(Debug, Clone, Default)]
pub(crate) struct ObjectIndex {
    /// Objects standing on each plot.
    on: Vec<Vec<u32>>,
    /// Per plot, what the people living there may use: their own objects, public places' and
    /// anything off the plots.
    usable: Vec<Vec<u32>>,
    /// Seats at tables and the like (`surface::Links`), set by `World::refresh_index`.
    pub links: crate::surface::Links,
    /// The `World::objects_version` it was built for.
    pub version: u32,
    /// Open-world towns: `shared` by `CELL` m cell (row-major, `cols` a row), each in id order;
    /// `usable` then holds only each plot's own.
    cells: Vec<Vec<u32>>,
    cols: usize,
    /// The ground's depth (m). The storeys above and the basement are stacked past it, each as
    /// deep: a place there is in the cell of the ground under it (`fold`).
    ground: f32,
}

impl ObjectIndex {
    /// Public and off-plot objects by cell (their middles, `centres`, by object id), not in every
    /// plot's list.
    pub fn open(
        object_plot: &[Option<u32>],
        plots: &[Plot],
        centres: &[[f32; 2]],
        size: [f32; 2],
        version: u32,
    ) -> Self {
        let mut index = Self {
            on: vec![Vec::new(); plots.len()],
            usable: vec![Vec::new(); plots.len()],
            links: Default::default(),
            version,
            cells: Vec::new(),
            cols: (size[0].max(1.0) / CELL).ceil() as usize,
            ground: size[1].max(1.0),
        };
        let rows = (size[1].max(1.0) / CELL).ceil() as usize;
        index.cells = vec![Vec::new(); index.cols * rows];
        for (id, plot) in object_plot.iter().enumerate() {
            let id = id as u32;
            match plot.map(|p| p as usize) {
                Some(p) if !plots[p].public => {
                    index.on[p].push(id);
                    index.usable[p].push(id);
                }
                p => {
                    if let Some(p) = p {
                        index.on[p].push(id);
                    }
                    let c = centres.get(id as usize).copied().unwrap_or([0.0, 0.0]);
                    let cell = index.cell_of(c, rows);
                    index.cells[cell].push(id);
                }
            }
        }
        index
    }

    /// `z` (stacked rows) on the ground: upstairs and in the basement, the ground under it.
    fn fold(&self, z: f32) -> f32 {
        if self.ground > 0.0 && z >= self.ground {
            z.rem_euclid(self.ground)
        } else {
            z
        }
    }

    fn cell_of(&self, p: [f32; 2], rows: usize) -> usize {
        let cx = ((p[0].max(0.0) / CELL) as usize).min(self.cols.saturating_sub(1));
        let cz = ((self.fold(p[1]).max(0.0) / CELL) as usize).min(rows.saturating_sub(1));
        cz * self.cols + cx
    }

    /// What people living on `home` (or nowhere) may use from `pos`: their own things, and the
    /// public ones within about `reach` m (the cells it touches), in id order.
    pub fn usable_near(&self, home: Option<u32>, pos: [f32; 2], reach: f32) -> Cow<'_, [u32]> {
        let own: &[u32] = home
            .and_then(|p| self.usable.get(p as usize))
            .map_or(&[], Vec::as_slice);
        let rows = self.cells.len() / self.cols.max(1);
        let span = |v: f32, n: usize| {
            let lo = ((v - reach).max(0.0) / CELL) as usize;
            let hi = (((v + reach).max(0.0) / CELL) as usize).min(n.saturating_sub(1));
            lo.min(hi)..=hi
        };
        let mut near: Vec<u32> = Vec::new();
        for cz in span(self.fold(pos[1]), rows) {
            for cx in span(pos[0], self.cols) {
                near.extend_from_slice(&self.cells[cz * self.cols + cx]);
            }
        }
        near.extend_from_slice(own);
        near.sort_unstable();
        Cow::Owned(near)
    }

    /// Objects on `plot`.
    pub fn on(&self, plot: u32) -> &[u32] {
        self.on.get(plot as usize).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plot(id: u32, public: bool) -> Plot {
        Plot {
            id,
            name: String::new(),
            x: 0,
            z: 0,
            w: 1,
            d: 1,
            public,
            entry: None,
            roof: None,
        }
    }

    #[test]
    fn open_towns_keep_public_things_in_cells() {
        let plots = [plot(0, false), plot(1, true)];
        // A thing at home, a public one near, one off the plots far away.
        let object_plot = [Some(0), Some(1), None];
        let centres = [[5.0, 5.0], [70.0, 5.0], [300.0, 300.0]];
        let ix = ObjectIndex::open(&object_plot, &plots, &centres, [320.0, 320.0], 1);
        assert_eq!(&*ix.usable_near(Some(0), [5.0, 5.0], 100.0), [0, 1]);
        assert_eq!(&*ix.usable_near(Some(0), [250.0, 250.0], 100.0), [0, 2]);
        assert_eq!(&*ix.usable_near(None, [5.0, 5.0], 400.0), [1, 2]);
    }

    #[test]
    fn open_towns_find_public_things_upstairs_over_the_ground_under_them() {
        let plots = [plot(0, true)];
        // A public thing on the ground near x 300, one upstairs (a block of 320 rows on) above it.
        let object_plot = [Some(0), Some(0)];
        let centres = [[300.0, 10.0], [300.0, 330.0]];
        let ix = ObjectIndex::open(&object_plot, &plots, &centres, [320.0, 320.0], 1);
        assert_eq!(&*ix.usable_near(None, [300.0, 330.0], 10.0), [0, 1]);
        assert_eq!(&*ix.usable_near(None, [300.0, 10.0], 10.0), [0, 1]);
        assert!(ix.usable_near(None, [10.0, 330.0], 10.0).is_empty());
    }
}
