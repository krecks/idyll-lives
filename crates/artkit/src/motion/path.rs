//! The paths a channel moves along, and how it eases.
//!
//! Split from `motion.rs` (see `mod.rs`).

/// What a channel keys: the part's place, its turn (Euler angles in the part's order, radians) or its scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Path {
    Location,
    Rotation,
    Scale,
}
