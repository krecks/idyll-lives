//! Grounds: the plot a tile belongs to and how a walk over it is charged.

use super::*;

/// Going up or down a storey counts as this many metres of walking (for choosing what to do).
pub const STOREY_WALK: f32 = 4.0;

pub(crate) const ORTHO: u32 = 10;
pub(crate) const DIAG: u32 = 14;
/// Extra cost of stepping onto a tile of someone else's plot: twenty tiles of walking.
pub(crate) const TRESPASS: u32 = ORTHO * 20;
/// Extra cost of stepping straight from one plot into another beside a garden (`Grounds::fenced`):
/// three hundred tiles, so a garden is entered from the street at its front wherever there is one,
/// and only crossed into where there isn't. Gardens are kept apart all round, though the renderer
/// draws their fences and hedges (`shared/street.ts`) only from the house's front back, and none
/// round some: next door is by the street, as a visitor would go.
pub(crate) const OVER_THE_FENCE: u32 = ORTHO * 300;

/// The stair halls of buildings with flats over a public place (a shop, a café): the flats' own
/// way in from the street, on their building's lot but no part of the place. Only those living
/// in the flats and their guests walk across one (`Grounds`); made with the rooms
/// (`World::map_halls`).
#[derive(Debug, Clone, Default)]
pub struct Halls {
    /// By plot (a building's lot), the index of its hall in `list` + 1 (0: none).
    pub of_lot: Vec<u16>,
    pub list: Vec<Hall>,
}

/// A building's stair hall (`Halls`): its lot, and which of the lot's ground tiles (row-major)
/// are the hall's (the same tiles upstairs are its landings').
#[derive(Debug, Clone)]
pub struct Hall {
    pub lot: u32,
    pub x: i32,
    pub z: i32,
    pub w: i32,
    pub tiles: Vec<bool>,
}

impl Hall {
    /// Whether ground tile (`x`, `z`) is the hall's.
    pub fn covers(&self, x: i32, z: i32) -> bool {
        let (dx, dz) = (x - self.x, z - self.z);
        dx >= 0
            && dz >= 0
            && dx < self.w
            && self
                .tiles
                .get((dz * self.w + dx) as usize)
                .copied()
                .unwrap_or(false)
    }
}

impl Halls {
    /// No halls.
    pub const NONE: Halls = Halls {
        of_lot: Vec::new(),
        list: Vec::new(),
    };

    /// The hall (its index in `list`) ground tile (`x`, `z`) of lot `lot` is in, if it is in one.
    pub fn at(&self, lot: u32, x: i32, z: i32) -> Option<usize> {
        let k = usize::from(*self.of_lot.get(lot as usize)?).checked_sub(1)?;
        self.list.get(k)?.covers(x, z).then_some(k)
    }
}

/// A hall as `Grounds::plot` gives it (its index in `Halls::list` with this bit set): not one of
/// the plots.
pub const HALL: u32 = 1 << 31;

/// Which plots one resident may walk across (their home, public places, where they're going);
/// the rest are other people's gardens, kept off unless there's no other way. A flats' stair hall
/// over a public place counts as a plot of its own (`Halls`): only for its flats' people and
/// their guests.
pub struct Grounds<'a> {
    /// Plot per tile of the ground storey, row by row (upper storeys are over the same ground;
    /// `NO_PLOT` off the plots).
    pub tile_plot: &'a [u16],
    /// Per lot on the ground (by id; `World::lots`): whether this resident may walk across it.
    pub open: Vec<bool>,
    /// Per lot on the ground (by id): whether its sides are fenced off from the plots beside it (a garden; a
    /// public place isn't).
    pub fenced: Vec<bool>,
    /// The flats' stair halls over public places, and per hall whether this resident may walk
    /// across it.
    pub halls: &'a Halls,
    pub hall_open: Vec<bool>,
}

impl Grounds<'_> {
    /// The plot a tile is on (on any storey), or the hall (`HALL` and its index) it is in.
    pub fn plot(&self, nav: &NavGrid, x: i32, z: i32) -> Option<u32> {
        let z = nav.stairs.ground_z(z);
        if x < 0 || z < 0 || x >= nav.lot.width as i32 {
            return None;
        }
        let p = self
            .tile_plot
            .get(z as usize * nav.lot.width + x as usize)
            .filter(|&&p| p != NO_PLOT)
            .map(|&p| u32::from(p))?;
        Some(match self.halls.at(p, x, z) {
            Some(k) => HALL | k as u32,
            None => p,
        })
    }

    /// Whether plot `p` (a tile's) is one this resident shouldn't walk across.
    pub(crate) fn foreign(&self, p: Option<u32>) -> bool {
        p.is_some_and(|p| {
            let open = if p & HALL != 0 {
                &self.hall_open[..]
            } else {
                &self.open[..]
            };
            !open.get((p & !HALL) as usize).copied().unwrap_or(true)
        })
    }

    /// A step from plot `a` straight into plot `b` (tiles'), over the fence between them, open
    /// or not: a garden's side or back, not between two public places.
    pub(crate) fn over_the_fence(&self, a: Option<u32>, b: Option<u32>) -> bool {
        let fenced = |p: u32| self.fenced.get(p as usize).copied().unwrap_or(false);
        matches!((a, b), (Some(a), Some(b)) if a != b && (fenced(a) || fenced(b)))
    }
}

/// No plot (`Grounds::tile_plot`, `World::tile_plot`).
pub const NO_PLOT: u16 = u16::MAX;
