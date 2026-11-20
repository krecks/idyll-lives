//! Grounds: the plot a tile belongs to and how a walk over it is charged.
//!
//! Split from `path.rs` (see `mod.rs`).

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

/// Which plots one resident may walk across (their home, public places, where they're going);
/// the rest are other people's gardens, kept off unless there's no other way.
pub struct Grounds<'a> {
    /// Plot per tile of the ground storey, row by row (upper storeys are over the same ground;
    /// `NO_PLOT` off the plots).
    pub tile_plot: &'a [u16],
    /// Per plot (by id): whether this resident may walk across it.
    pub open: Vec<bool>,
    /// Per plot (by id): whether its sides are fenced off from the plots beside it (a garden; a
    /// public place isn't).
    pub fenced: Vec<bool>,
}

impl Grounds<'_> {
    /// The plot a tile is on (on any storey).
    pub fn plot(&self, nav: &NavGrid, x: i32, z: i32) -> Option<u32> {
        let z = nav.stairs.ground_z(z);
        if x < 0 || z < 0 || x >= nav.lot.width as i32 {
            return None;
        }
        self.tile_plot
            .get(z as usize * nav.lot.width + x as usize)
            .filter(|&&p| p != NO_PLOT)
            .map(|&p| u32::from(p))
    }

    /// Whether plot `p` (a tile's) is one this resident shouldn't walk across.
    pub(crate) fn foreign(&self, p: Option<u32>) -> bool {
        p.is_some_and(|p| !self.open.get(p as usize).copied().unwrap_or(true))
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
